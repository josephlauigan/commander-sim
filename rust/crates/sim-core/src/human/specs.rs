//! What the person chooses as they cast a spell that resolves by its own code (a `resolve` hook) and has no `Rem` tag.
//!
//! The AI's plan for such a card fills a `Ctx` (`target`, `face`, `x`, `mode` ...) and casts with it; the card's
//! resolve hook reads it. For a person the cast flow (`human::cast_inner`) used to hand the hook an empty `Ctx`, so
//! the card did nothing or the code picked for the person. A card lists here, in the order the person is asked, what
//! it needs: a target, a mode, cards from the graveyard, an additional cost, X. The cast flow runs `choose_pre`
//! before paying, `pay_costs` once the mana is paid, and `choose_post` for what comes out of the mana left in the pool.
//! The result lands in the same `Ctx` fields the AI's plan fills, so the hook reads both alike.
//!
//! Never reached for an AI seat.

use super::{Target, choices, legal, mana};
use crate::cards::{Colors, Types};
use crate::engine::values::{protected_from, untargetable};
use crate::engine::zones;
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::intern;

/// What a `Target` ask may aim at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tk {
    /// any creature
    Creature,
    /// any creature or planeswalker
    CreatureOrPw,
    /// a creature you control
    OwnCreature,
    /// a player (you too)
    Player,
}

/// One thing the person is asked as the spell is cast.
#[derive(Debug, Clone, Copy)]
pub enum Ask {
    /// a target: `ctx.target` for a permanent, `ctx.face` for a player
    Target(Tk),
    /// card(s) in your graveyard -> `ctx.cards` (creature cards, or planeswalkers too; at most `max_cmc` if set).
    /// `n` are chosen; fewer than `min` available means the spell can't be cast.
    Gy { n: usize, min: usize, max_cmc: Option<u32>, pw: bool },
    /// a mode, chosen at cast time: (label, `ctx.mode`)
    Mode(&'static [(&'static str, &'static str)]),
    /// additional cost: sacrifice a creature (not the target); `ctx.x` = 1 if it was modified
    SacModified,
    /// additional cost: sacrifice a creature (not the target)
    Sac,
    /// additional cost: discard a card; `ctx.x` = 1 if it wasn't a land
    DiscardNonland,
    /// X, from what is left in the pool
    X,
    /// Scout the City: mill-and-gain, or destroy a creature with flying
    Scout,
    /// Comet Storm: kicks, X, and one distinct target for each
    Comet,
    /// a kicker of these coloured pips, from what is left in the pool; `ctx.kicks` = 1 if paid (Orim's Chant)
    Kick(&'static str),
}

const SPLATTER: &[(&str, &str)] =
    &[("draw four cards", "draw"), ("4 damage to each creature and each planeswalker", "wipe")];
const BLOOD: &[(&str, &str)] = &[("destroy all creatures", "creatures"), ("destroy all planeswalkers", "walkers")];

/// The asks for a card, if the person's cast flow needs any.
pub fn spec(name: &str) -> Option<&'static [Ask]> {
    Some(match name {
        "Lethal Throwdown" => &[Ask::Target(Tk::CreatureOrPw), Ask::SacModified],
        "Presumed Dead" | "Dark Endurance" => &[Ask::Target(Tk::Creature)],
        "Stargaze" => &[Ask::X],
        "Grab the Prize" => &[Ask::DiscardNonland],
        "Splatter Technique" => &[Ask::Mode(SPLATTER)],
        "Blood on the Snow" => &[Ask::Mode(BLOOD)],
        "Comet Storm" => &[Ask::Comet],
        "Cost of Brilliance" | "Painful Lesson" | "Peer into the Abyss" | "Decorum Dissertation" => {
            &[Ask::Target(Tk::Player)]
        }
        "Rewrite Regrets" => &[Ask::Gy { n: 1, min: 0, max_cmc: Some(6), pw: true }],
        "Helping Hand" => &[Ask::Gy { n: 1, min: 0, max_cmc: Some(3), pw: false }],
        "Victimize" => &[Ask::Gy { n: 2, min: 2, max_cmc: None, pw: false }, Ask::Sac],
        "Daydream" => &[Ask::Target(Tk::OwnCreature)],
        "Scout the City" => &[Ask::Scout],
        "Orim's Chant" => &[Ask::Target(Tk::Player), Ask::Kick("W")],
        _ => return None,
    })
}

/// The cost choices made before paying, carried to `pay_costs`.
#[derive(Debug, Default)]
pub struct Costs {
    pub sac: Option<PermId>,
    pub discard: Option<CardId>,
}

/// The outcome of the questions before payment.
pub enum Pre {
    /// the person cancelled
    Cancel,
    /// the spell can't be cast (and why)
    Problem(String),
    Ok(Costs),
}

fn perm_cands(g: &Game, p: PlayerId, c: CardId, kind: Tk) -> Vec<PermId> {
    let pips = Colors::from_letters(&g.db.get(c).pips);
    let mut out = vec![];
    for q in g.players.iter().filter(|q| q.alive) {
        for &m in &q.perms {
            let x = g.perm(m);
            let pw = x.cd.is_some_and(|cd| g.db.get(cd).types.has(Types::PLANESWALKER));
            let cr = g.is_creature(m);
            let ok = match kind {
                Tk::Creature => cr,
                Tk::CreatureOrPw => cr || pw,
                Tk::OwnCreature => cr && x.owner == p,
                Tk::Player => false,
            };
            if !ok || x.phased || (x.owner != p && (untargetable(g, m) || protected_from(g, m, pips))) {
                continue;
            }
            out.push(m);
        }
    }
    out
}

fn gy_cands(g: &Game, p: PlayerId, max_cmc: Option<u32>, pw: bool) -> Vec<CardId> {
    g.player(p)
        .gy
        .iter()
        .copied()
        .filter(|&x| {
            let d = g.db.get(x);
            (d.creature || (pw && d.types.has(Types::PLANESWALKER))) && max_cmc.is_none_or(|m| d.cmc <= m)
        })
        .collect()
}

fn my_creatures(g: &Game, p: PlayerId, not: Option<PermId>) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased && Some(m) != not).collect()
}

fn players(g: &Game) -> Vec<PlayerId> {
    g.players.iter().filter(|q| legal::player_targetable(g, q.id) && q.alive).map(|q| q.id).collect()
}

/// A creature with a counter, or Equipment/an Aura of yours attached (Lethal Throwdown's "modified").
fn modified(g: &Game, p: PlayerId, m: PermId) -> bool {
    crate::impls::sheoldred::modified(g, p, m)
}

/// Why card c (with a spec) can't be cast right now, if so; checked before the person is asked anything.
pub fn problem(g: &Game, p: PlayerId, c: CardId) -> Option<String> {
    let d = g.db.get(c);
    let asks = spec(&d.name)?;
    for a in asks {
        match *a {
            Ask::Target(Tk::Player) if players(g).is_empty() => {
                return Some(format!("{} has no legal target.", d.name));
            }
            Ask::Target(k) if k != Tk::Player && perm_cands(g, p, c, k).is_empty() => {
                return Some(format!("{} has no legal target right now.", d.name));
            }
            Ask::Gy { min, max_cmc, pw, .. } if gy_cands(g, p, max_cmc, pw).len() < min => {
                return Some(format!("{} needs {min} creature cards in your graveyard.", d.name));
            }
            Ask::Sac | Ask::SacModified if my_creatures(g, p, None).is_empty() => {
                return Some(format!("{} needs a creature to sacrifice as you cast it, and you control none.", d.name));
            }
            Ask::DiscardNonland if !g.player(p).hand.iter().any(|&x| x != c) => {
                return Some(format!("{} needs a card to discard as you cast it.", d.name));
            }
            _ => {}
        }
    }
    None
}

fn labels_of(g: &Game, p: PlayerId, t: &[Target]) -> Vec<String> {
    t.iter().map(|&x| legal::describe_target(g, p, x)).collect()
}

/// Ask what must be chosen before paying. Fills `ctx`; returns the costs to pay once the mana is paid.
pub fn choose_pre(g: &Game, p: PlayerId, c: CardId, ctx: &mut Ctx) -> Res<Pre> {
    let d = g.db.get(c);
    let name = d.name.to_string();
    let Some(asks) = spec(&name) else { return Ok(Pre::Ok(Costs::default())) };
    if let Some(why) = problem(g, p, c) {
        return Ok(Pre::Problem(why));
    }
    let mut costs = Costs::default();
    for a in asks {
        match *a {
            Ask::Target(Tk::Player) => {
                let ps = players(g);
                let t: Vec<Target> = ps.iter().map(|&q| Target::Player(q)).collect();
                let pr = format!("{name}: choose a target (player)");
                let Some(k) = choices::choose(g, p, "target", &pr, &labels_of(g, p, &t), Some("cancel"), None)? else {
                    return Ok(Pre::Cancel);
                };
                ctx.face = Some(ps[k]);
            }
            Ask::Target(kind) => {
                let ms = perm_cands(g, p, c, kind);
                let t: Vec<Target> = ms.iter().map(|&m| Target::Perm(m)).collect();
                let what = match kind {
                    Tk::CreatureOrPw => "creature or planeswalker",
                    Tk::OwnCreature => "creature you control",
                    _ => "creature",
                };
                let pr = format!("{name}: choose a target ({what})");
                let Some(k) = choices::choose(g, p, "target", &pr, &labels_of(g, p, &t), Some("cancel"), None)? else {
                    return Ok(Pre::Cancel);
                };
                ctx.target = Some(ms[k]);
            }
            Ask::Gy { n, max_cmc, pw, .. } => {
                let mut left = gy_cands(g, p, max_cmc, pw);
                let what = if pw { "creature or planeswalker card" } else { "creature card" };
                for i in 0..n.min(left.len()) {
                    let labels: Vec<String> = left.iter().map(|&x| choices::card_label(g, x)).collect();
                    let pr = format!("{name}: choose a {what} in your graveyard ({} of {n})", i + 1);
                    let Some(k) = choices::choose(g, p, "choose", &pr, &labels, Some("cancel"), None)? else {
                        return Ok(Pre::Cancel);
                    };
                    ctx.cards.push(left.remove(k));
                }
            }
            Ask::Mode(modes) => {
                let labels: Vec<String> = modes.iter().map(|m| m.0.to_string()).collect();
                let Some(k) =
                    choices::choose(g, p, "choose", &format!("{name}: choose a mode"), &labels, Some("cancel"), None)?
                else {
                    return Ok(Pre::Cancel);
                };
                ctx.mode = Some(intern(modes[k].1));
            }
            Ask::Scout => {
                let fliers: Vec<PermId> = perm_cands(g, p, c, Tk::Creature)
                    .into_iter()
                    .filter(|&m| crate::dsl::has_kw(g, m, "flying") && g.perm(m).owner != p)
                    .collect();
                let mut labels =
                    vec!["mill three cards, put a permanent card from them into your hand, gain 3 life".to_string()];
                if !fliers.is_empty() {
                    labels.push("destroy target creature with flying".into());
                }
                let Some(k) =
                    choices::choose(g, p, "choose", &format!("{name}: choose a mode"), &labels, Some("cancel"), None)?
                else {
                    return Ok(Pre::Cancel);
                };
                if k == 1 {
                    let t: Vec<Target> = fliers.iter().map(|&m| Target::Perm(m)).collect();
                    let pr = format!("{name}: choose a target (creature with flying)");
                    let Some(j) = choices::choose(g, p, "target", &pr, &labels_of(g, p, &t), Some("cancel"), None)?
                    else {
                        return Ok(Pre::Cancel);
                    };
                    ctx.target = Some(fliers[j]);
                    ctx.mode = Some(intern("flier"));
                } else {
                    ctx.mode = Some(intern("mill"));
                }
            }
            Ask::Sac | Ask::SacModified => {
                let cre = my_creatures(g, p, ctx.target);
                if cre.is_empty() {
                    return Ok(Pre::Problem(format!("{name} needs another creature to sacrifice as you cast it.")));
                }
                let t: Vec<Target> = cre.iter().map(|&m| Target::Perm(m)).collect();
                let pr = format!("{name}: sacrifice which creature?");
                let Some(k) = choices::choose(g, p, "choose", &pr, &labels_of(g, p, &t), Some("cancel"), None)? else {
                    return Ok(Pre::Cancel);
                };
                costs.sac = Some(cre[k]);
                if matches!(a, Ask::SacModified) {
                    ctx.x = modified(g, p, cre[k]) as i32;
                }
            }
            Ask::DiscardNonland => {
                let hand: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&x| x != c).collect();
                let labels: Vec<String> = hand.iter().map(|&x| choices::card_label(g, x)).collect();
                let pr = format!("{name}: discard which card?");
                let Some(k) = choices::choose(g, p, "choose", &pr, &labels, Some("cancel"), None)? else {
                    return Ok(Pre::Cancel);
                };
                costs.discard = Some(hand[k]);
                ctx.x = !g.db.get(hand[k]).land as i32;
            }
            Ask::X | Ask::Comet | Ask::Kick(_) => {} // after the mana is paid (choose_post)
        }
    }
    Ok(Pre::Ok(costs))
}

/// Pay the additional costs chosen in `choose_pre`, once the mana is paid.
pub fn pay_costs(g: &mut Game, p: PlayerId, c: CardId, costs: &Costs) -> Res {
    let name = g.db.get(c).name.to_string();
    if let Some(f) = costs.sac {
        crate::glog!(g, "  {} sacrifices {} for {name}", g.player(p).name, g.perm(f).name);
        zones::die(g, f, "sac")?;
    }
    if let Some(x) = costs.discard {
        zones::discard_cards(g, p, &[x])?;
    }
    Ok(())
}

/// What comes out of the mana left in the pool (X, kicks) and the targets that depend on it. Pays that mana.
pub fn choose_post(g: &mut Game, p: PlayerId, c: CardId, ctx: &mut Ctx) -> Res {
    let name = g.db.get(c).name.to_string();
    let Some(asks) = spec(&name) else { return Ok(()) };
    for a in asks {
        match *a {
            Ask::X => {
                let top = g.player(p).pool.total();
                let x = if top > 0 {
                    let labels: Vec<String> = (0..=top).map(|x| format!("X = {x}")).collect();
                    let pr = format!("{name}: choose X (paid from what is left in your pool)");
                    choices::choose(g, p, "choose", &pr, &labels, None, None)?.unwrap_or(0) as u32
                } else {
                    0
                };
                mana::pay_from_pool(g, p, x, "");
                ctx.x = x as i32;
            }
            Ask::Kick(pips) => {
                let ok = mana::cost_problem(g, p, 0, pips).is_none();
                let pr = format!("{name}: pay the kicker {{{pips}}}?");
                let labels = vec![format!("kick it ({{{pips}}})"), "no kicker".to_string()];
                if ok && choices::choose(g, p, "choose", &pr, &labels, None, None)? == Some(0) {
                    mana::pay_from_pool(g, p, 0, pips);
                    ctx.kicks = 1;
                }
            }
            Ask::Comet => {
                // {X}{R}{R}, multikicker {1}: X damage to each of 1 + kicks distinct targets
                let pool = g.player(p).pool.total();
                let tg = choices::damage_targets(g, p, Colors::from_letters("R"));
                let max_k = (pool as usize).min(tg.len().saturating_sub(1));
                let labels: Vec<String> = (0..=max_k)
                    .map(|k| match k {
                        0 => "no kicker: 1 target".to_string(),
                        1 => "kick once ({1}): 2 targets".to_string(),
                        _ => format!("kick {k} times ({{{k}}}): {} targets", k + 1),
                    })
                    .collect();
                let pr = format!("{name}: multikicker {{1}}: how many times?");
                let k = choices::choose(g, p, "choose", &pr, &labels, None, None)?.unwrap_or(0);
                let rest = pool - k as u32;
                let xl: Vec<String> = (0..=rest).map(|x| format!("X = {x}")).collect();
                let pr = format!("{name}: choose X (damage to each target)");
                let x = choices::choose(g, p, "choose", &pr, &xl, None, None)?.unwrap_or(0) as u32;
                mana::pay_from_pool(g, p, x + k as u32, "");
                ctx.x = x as i32;
                ctx.kicks = k as u32;
                let mut left = tg;
                for i in 0..=k {
                    if left.is_empty() {
                        break;
                    }
                    let pr = format!("{name}: choose target {} of {} (it takes {x})", i + 1, k + 1);
                    let j = choices::choose_one(g, p, "target", &pr, &labels_of(g, p, &left))?;
                    match left.remove(j) {
                        Target::Perm(m) => ctx.perms.push(m),
                        Target::Player(q) => ctx.faces.push(q),
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// The person names a player (any alive, targetable one; you too).
pub fn ask_player(g: &Game, p: PlayerId, name: &str) -> Res<PlayerId> {
    let ps = players(g);
    let t: Vec<Target> = ps.iter().map(|&q| Target::Player(q)).collect();
    let k = choices::choose_one(g, p, "target", &format!("{name}: choose a target (player)"), &labels_of(g, p, &t))?;
    Ok(ps[k])
}
