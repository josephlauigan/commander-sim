//! Tier 6 batch `redirects`: the redirect spells, the damage / removal instants and the small white and green
//! instants (rust/agents/t6-batches.md). Cards only: no engine file is touched, so what needs the engine (answering a
//! spell or ability on the stack, alternative costs, "can't cast" locks, flash for a creature card) is written as a
//! plain `pub fn` the engine can call later, and listed in rust/agents/engine-needs-redirects.md.
//!
//! - Redirect Lightning, Ricochet Trap, Untimely Malfunction (redirect mode): `redirect_removal`, the same response
//!   as Deflecting Swat / Return the Favor in `mine.rs`, plus `redirect_spell` for a spell on the stack.

use crate::engine::cast::on_cast;
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, pay};
use crate::engine::removal::apply_removal;
use crate::engine::values::{pval, untargetable};
use crate::engine::values::{epow, etgh};
use crate::engine::zones::{die, max_by};
use crate::state::Ctx;
use crate::flow::Res;
use crate::hooks::Registry;
use crate::ids::{CardId, PermId, PlayerId};
use crate::cards::CardDb;
use crate::state::Game;
use crate::sym::Sym;

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn remove_card(v: &mut Vec<CardId>, c: CardId) {
    if let Some(i) = v.iter().position(|&x| x == c) {
        v.remove(i);
    }
}

/// p casts `card` from the hand as a response: the cast is counted (storm, magecraft, cast triggers) and the card
/// goes to the graveyard; the cost was paid by the caller
fn cast_from_hand(g: &mut Game, p: PlayerId, card: CardId) -> Res {
    remove_card(&mut g.player_mut(p).hand, card);
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.cast_names.insert(card);
    pl.stat("spells_cast", 1);
    on_cast(g, p, card)?;
    g.player_mut(p).gy.push(card);
    Ok(())
}

// ================================================================== redirecting removal
/// the permanents of `actor` a redirected spell of this kind may be aimed at (a damage spell needs a creature)
fn redirect_targets(g: &Game, actor: PlayerId, kind: &str, not: PermId) -> Vec<PermId> {
    g.player(actor)
        .perms
        .iter()
        .copied()
        .filter(|&x| {
            x != not
                && g.perm(x).on_bf
                && !g.perm(x).phased
                && !untargetable(g, x)
                && (g.is_creature(x) || !kind.starts_with("dmg"))
        })
        .collect()
}

/// Redirect Lightning's additional cost: how p pays "5 life or {2}". True = the life. The AI pays the mana when it
/// has it ({R} plus {2}), else 5 life when that leaves it above 10; None if neither can be paid.
pub fn redirect_lightning_life(g: &Game, p: PlayerId) -> Option<bool> {
    if can_pay(g, p, 2, "R", false) {
        Some(false)
    } else if can_pay(g, p, 0, "R", false) && g.player(p).life > 10 {
        Some(true)
    } else {
        None
    }
}

/// the permanent the AI redirects a targeted spell of `kind` to: the best of the caster's own (not `m`)
pub fn redirect_to(g: &Game, actor: PlayerId, kind: &str, m: PermId) -> Option<PermId> {
    max_by(&redirect_targets(g, actor, kind, m), |x| pval(g, x))
}

/// Redirect Lightning / Untimely Malfunction / Ricochet Trap answering removal aimed at the owner's permanent `m` (the
/// response `ai::protect_response` would call, like Deflecting Swat in `mine.rs`): the best card the owner holds pays
/// its cost, and the spell is aimed at the best of the caster's own permanents instead. Redirect Lightning costs {R}
/// and {2} or 5 life, Untimely Malfunction {1}{R}, Ricochet Trap {R} after a blue spell by an opponent or {3}{R}.
/// True if the removal was redirected (and so has been applied to the new target).
pub fn redirect_removal(
    g: &mut Game,
    owner: PlayerId,
    m: PermId,
    kind: Sym,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    let (Some(actor), Some(spell)) = (actor, spell) else { return Ok(false) };
    if actor == owner || matches!(&*kind, "edict" | "wipe") || (pval(g, m) < 4.0 && !g.perm(m).is_cmd) {
        return Ok(false);
    }
    let Some(t) = redirect_to(g, actor, &kind, m) else { return Ok(false) };
    for name in ["Redirect Lightning", "Untimely Malfunction", "Ricochet Trap"] {
        let Some(c) = g.db.id(name).filter(|c| g.player(owner).hand.contains(c)) else { continue };
        match name {
            "Redirect Lightning" => {
                let Some(life) = redirect_lightning_life(g, owner) else { continue };
                if life {
                    pay(g, owner, 0, "R", false)?;
                    lose_life(g, owner, 5, Some(owner), "other", None)?;
                } else {
                    pay(g, owner, 2, "R", false)?;
                }
            }
            "Untimely Malfunction" => {
                if !can_pay(g, owner, 1, "R", false) {
                    continue;
                }
                pay(g, owner, 1, "R", false)?;
            }
            _ => {
                // Ricochet Trap: change the target of a spell; {R} if an opponent cast a blue spell this turn
                let (gn, pips) = if blue_spell_this_turn(g, owner) { (0, "R") } else { (3, "R") };
                if !can_pay(g, owner, gn, pips, false) {
                    continue;
                }
                pay(g, owner, gn, pips, false)?;
            }
        }
        cast_from_hand(g, owner, c)?;
        crate::glog!(g, "    {} casts {}: {} now targets {}", pname(g, owner), name, cname(g, spell), g.perm(t).name);
        apply_removal(g, Some(actor), t, &kind, Some(spell))?;
        return Ok(true);
    }
    Ok(false)
}

/// Ricochet Trap's condition: an opponent of p cast a blue spell this turn
pub fn blue_spell_this_turn(g: &Game, p: PlayerId) -> bool {
    g.opps(p).any(|q| match &g.player(q).turn_casts {
        Some((st, cs)) if *st == g.turn_stamp() => cs.iter().any(|&c| g.db.get(c).pips.contains('U')),
        _ => false,
    })
}


// ================================================================== Untimely Malfunction
/// Untimely Malfunction resolves. The modes: destroy target artifact; change the target of a spell or ability with a
/// single target (`redirect_removal`, the response); one or two target creatures can't block this turn. With an
/// artifact as the chosen target it is destroyed; without one, up to two of the opponents' best blockers can't block
/// (the creatures that stop the most damage: toughness then power).
fn malfunction_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf && g.perm(t).owner != p) {
        let artifact = g.perm(t).cd.is_some_and(|c| g.db.get(c).types.has(crate::cards::Types::ARTIFACT));
        if artifact {
            apply_removal(g, Some(p), t, "destroy", None)?;
            return Ok("gy");
        }
    }
    let mut blockers: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.clone())
        .filter(|&m| {
            let x = g.perm(m);
            x.on_bf && !x.phased && !x.tapped && g.is_creature(m) && !untargetable(g, m)
        })
        .collect();
    blockers.sort_by_key(|&m| std::cmp::Reverse(etgh(g, m) * 4 + epow(g, m)));
    for m in blockers.into_iter().take(2) {
        super::partials::eot_kw(g, m, "cant_block");
        crate::glog!(g, "    {} can't block this turn (Untimely Malfunction)", g.perm(m).name);
    }
    Ok("gy")
}


// ================================================================== Pyrokinesis
/// the red cards in p's hand other than `card` (Pyrokinesis exiles one instead of paying {4}{R}{R})
pub fn red_pitch(g: &Game, p: PlayerId, card: CardId) -> Vec<CardId> {
    g.player(p).hand.iter().copied().filter(|&x| x != card && g.db.get(x).pips.contains('R')).collect()
}

/// Pyrokinesis' alternative cost: exile a red card from p's hand (the least useful one). False if there is none.
pub fn pay_pyro_pitch(g: &mut Game, p: PlayerId, card: CardId) -> Res<bool> {
    let pitch = red_pitch(g, p, card);
    let Some(x) = crate::engine::zones::min_by(&pitch, |c| crate::engine::tutors::card_worth(g, p, c, false)) else {
        return Ok(false);
    };
    let pl = g.player_mut(p);
    remove_card(&mut pl.hand, x);
    pl.exile.push(x);
    crate::glog!(g, "    {} exiles {} (Pyrokinesis)", pname(g, p), cname(g, x));
    Ok(true)
}

/// how the AI divides Pyrokinesis' 4 damage: lethal damage to the opposing creatures worth the most (the chosen
/// target first), each only if what is left kills it
pub fn pyro_split(g: &Game, p: PlayerId, first: Option<PermId>) -> Vec<(PermId, i32)> {
    let mut cs: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.clone())
        .filter(|&m| {
            let x = g.perm(m);
            x.on_bf && !x.phased && g.is_creature(m) && !untargetable(g, m) && pval(g, m) >= 0.5
        })
        .collect();
    cs.sort_by(|&a, &b| {
        let key = |m: PermId| (Some(m) == first, pval(g, m));
        key(b).partial_cmp(&key(a)).unwrap()
    });
    let mut left = 4;
    let mut out = vec![];
    for m in cs {
        let need = etgh(g, m).max(1);
        if need <= left {
            left -= need;
            out.push((m, need));
        }
    }
    out
}

/// Pyrokinesis: 4 damage divided among any number of target creatures
fn pyro_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    for (m, n) in pyro_split(g, p, ctx.target) {
        if g.perm(m).on_bf {
            apply_removal(g, Some(p), m, &format!("dmg{n}"), Some(c))?;
        }
    }
    Ok("gy")
}


// ================================================================== turn flags
fn set_flag(g: &mut Game, p: PlayerId, key: Sym) {
    let stamp = g.turn_stamp();
    g.player_mut(p).flag_turn.insert(key, stamp);
}

fn flag_now(g: &Game, p: PlayerId, key: Sym) -> bool {
    g.player(p).flag_turn.get(key) == Some(&g.turn_stamp())
}

// ================================================================== Scout's Warning
/// p's next creature card this turn can be played as though it had flash (set by Scout's Warning; the engine's
/// "can this card be cast now" check reads it, and calls `scout_flash_used` when a creature card is cast with it)
pub fn scout_flash_active(g: &Game, p: PlayerId) -> bool {
    flag_now(g, p, "scout_flash")
}

/// a creature card was played: the Warning's grant is spent
pub fn scout_flash_used(g: &mut Game, p: PlayerId) {
    g.player_mut(p).flag_turn.shift_remove("scout_flash");
}

/// Scout's Warning: the next creature card you play this turn can be played as though it had flash; draw a card
fn scout_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    set_flag(g, p, "scout_flash");
    crate::engine::zones::draw(g, p, 1, false)?;
    Ok("gy")
}


// ================================================================== Kozilek's Command
/// the opposing creature Kozilek's Command (or any exile-by-mana-value effect) would exile for X: the best one whose
/// mana value is X or less (a token has mana value 0)
fn exile_target_for(g: &Game, p: PlayerId, x: i32) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.clone())
        .filter(|&m| {
            let o = g.perm(m);
            o.on_bf
                && !o.phased
                && g.is_creature(m)
                && !untargetable(g, m)
                && o.cd.map_or(0, |c| g.db.get(c).cmc as i32) <= x
        })
        .collect();
    max_by(&cs, |m| pval(g, m))
}

/// the cards Kozilek's Command's graveyard mode would exile: up to X from the opponents' graveyards, creatures (the
/// reanimation targets) with the highest mana value first
fn gy_exile_picks(g: &Game, p: PlayerId, x: i32) -> Vec<(PlayerId, CardId)> {
    let mut all: Vec<(PlayerId, CardId)> =
        g.opps(p).flat_map(|q| g.player(q).gy.iter().map(move |&c| (q, c)).collect::<Vec<_>>()).collect();
    all.retain(|&(_, c)| g.db.get(c).creature);
    all.sort_by_key(|&(_, c)| std::cmp::Reverse(g.db.get(c).cmc));
    all.truncate(x.max(0) as usize);
    all
}

/// Kozilek's Command ({X}{C}{C}, choose two): the AI scores the four modes and takes the best two. Exile a creature
/// with mana value X or less: the creature's value; scry X then draw: 2; X Eldrazi Spawn: 0.4 per Spawn (up to 5);
/// exile up to X cards from graveyards: 1.5 when an opponent has a creature card there.
fn kozilek_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let x = ctx.x.max(0);
    let target = exile_target_for(g, p, x);
    let picks = gy_exile_picks(g, p, x);
    let mut modes: Vec<(f64, u8)> = vec![
        (2.0 + 0.01, 1),                                   // scry X, draw a card
        (0.4 * x.min(5) as f64, 2),                        // X Spawn
        (if picks.is_empty() { 0.0 } else { 1.5 }, 3),     // graveyards
        (target.map_or(0.0, |m| pval(g, m).max(2.5)), 0),  // exile a creature
    ];
    modes.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    for &(_, mode) in modes.iter().take(2) {
        match mode {
            0 => {
                if let Some(m) = target.filter(|&m| g.perm(m).on_bf) {
                    apply_removal(g, Some(p), m, "exile", Some(c))?;
                }
            }
            1 => {
                crate::cardcode::scry(g, p, x, false)?;
                crate::engine::zones::draw(g, p, 1, false)?;
            }
            2 => {
                if x > 0 {
                    let spec = crate::engine::zones::Tokens {
                        tgh: Some(1),
                        color: Some(crate::cards::Colors::NONE),
                        types: vec!["eldrazi", "spawn"],
                        ..crate::engine::zones::Tokens::new(x as u32, 0)
                    };
                    crate::engine::zones::make_tokens(g, p, spec)?;
                }
            }
            _ => {
                for (q, card) in picks.clone() {
                    let pl = g.player_mut(q);
                    if pl.gy.contains(&card) {
                        remove_card(&mut pl.gy, card);
                        pl.exile.push(card);
                    }
                }
                crate::glog!(g, "    {} exiles {} cards from graveyards (Kozilek's Command)", pname(g, p), picks.len());
            }
        }
    }
    Ok("gy")
}


// ================================================================== Eldrazi Confluence
/// Eldrazi Confluence ({2}{C}{C}, choose three, the same mode may be chosen more than once): the AI spends its three
/// choices like this. +3/-3 on the opposing creatures worth the most per choice (a creature of toughness 4 to 6 takes
/// two choices, 7 to 9 three), each only if the choices left can kill it; then each remaining choice exiles and returns
/// (tapped) a different permanent of its own with an enters effect worth at least 3 (`t2::blink_value`), else it makes
/// a 1/1 Eldrazi Scion.
fn confluence_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mut left = 3;
    let mut cs: Vec<(PermId, i32, f64)> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.clone())
        .filter(|&m| {
            let x = g.perm(m);
            x.on_bf && !x.phased && g.is_creature(m) && !untargetable(g, m)
        })
        .map(|m| (m, (etgh(g, m).max(1) + 2) / 3, pval(g, m)))
        .collect();
    cs.sort_by(|a, b| (b.2 / b.1 as f64).partial_cmp(&(a.2 / a.1 as f64)).unwrap());
    for (m, k, v) in cs {
        if k <= left && v >= 0.5 {
            left -= k;
            for _ in 0..k {
                super::common::eot(g, m, 3, -3);
            }
            crate::glog!(g, "    {} gets +{}/-{} (Eldrazi Confluence)", g.perm(m).name, 3 * k, 3 * k);
            if etgh(g, m) <= 0 {
                die(g, m, "sba")?; // toughness 0 or less: no regeneration
            }
        }
    }
    // the rest: blink our best enters-effect permanents, else Scions
    let mut own: Vec<(PermId, i32)> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).on_bf && !g.perm(m).phased && !g.perm(m).token && !untargetable(g, m))
        .map(|m| (m, crate::impls::t2::blink_value(g, p, m)))
        .filter(|&(m, v)| v >= 3 && g.perm(m).cd.is_some_and(|c| !g.db.get(c).land))
        .collect();
    own.sort_by_key(|&(_, v)| std::cmp::Reverse(v));
    for _ in 0..left {
        if let Some((m, _)) = own.first().copied() {
            own.remove(0);
            if let Some(n) = crate::impls::t2::blink(g, p, m)? {
                g.perm_mut(n).tapped = true; // "return it to the battlefield tapped"
            }
        } else {
            let spec = crate::engine::zones::Tokens {
                tgh: Some(1),
                color: Some(crate::cards::Colors::NONE),
                types: vec!["eldrazi", "scion"],
                ..crate::engine::zones::Tokens::new(1, 1)
            };
            crate::engine::zones::make_tokens(g, p, spec)?;
        }
    }
    Ok("gy")
}


// ================================================================== removal with a price
/// the best opposing permanent of one of these types (creature, enchantment, planeswalker ...) p may target
fn best_opposing(g: &Game, p: PlayerId, ok: impl Fn(&Game, PermId) -> bool) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.clone())
        .filter(|&m| g.perm(m).on_bf && !g.perm(m).phased && !untargetable(g, m) && ok(g, m))
        .collect();
    max_by(&cs, |m| pval(g, m))
}

fn has_type(g: &Game, m: PermId, t: crate::cards::Types) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(t))
}

// ================================================================== Get Lost
/// Get Lost: destroy target creature, enchantment, or planeswalker. The engine aims it at a creature (its `tgt=c`
/// tag); with none chosen it takes the best opposing enchantment or planeswalker. (The two Map tokens its controller
/// creates are not modeled: the engine has no noncreature artifact tokens or explore-on-Map.)
fn get_lost_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    use crate::cards::Types;
    let t = ctx.target.filter(|&t| g.perm(t).on_bf).or_else(|| {
        best_opposing(g, p, |g, m| has_type(g, m, Types::ENCHANTMENT) || has_type(g, m, Types::PLANESWALKER))
    });
    if let Some(t) = t {
        apply_removal(g, Some(p), t, "destroy", Some(c))?;
    }
    Ok("gy")
}


// ================================================================== Erode
/// Erode: destroy target creature or planeswalker; its controller may search for a basic land, put it onto the
/// battlefield tapped, then shuffle (the AI always searches)
fn erode_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) else { return Ok("gy") };
    let owner = g.perm(t).owner;
    apply_removal(g, Some(p), t, "destroy", Some(c))?;
    crate::engine::zones::land_ramp(g, owner, 1, true)?;
    let mut lib = std::mem::take(&mut g.player_mut(owner).library);
    g.rng.shuffle(&mut lib);
    g.player_mut(owner).library = lib;
    Ok("gy")
}


// ================================================================== Soul Partition
/// Soul Partition: exile target nonland permanent; for as long as that card remains exiled its owner may play it, and
/// a spell an opponent of the caster casts that way costs {2} more. Modeled as exile, then the owner's card returns to
/// the owner's hand (a token is gone for good): the owner replays it at normal cost, since the engine has no
/// "play from exile" zone or cost tax (engine need).
fn soul_partition_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) else { return Ok("gy") };
    let (owner, card, token) = (g.perm(t).owner, g.perm(t).cd, g.perm(t).token);
    apply_removal(g, Some(p), t, "exile", Some(c))?;
    if let Some(card) = card.filter(|_| !token && !g.perm(t).on_bf)
        && g.player(owner).exile.contains(&card)
    {
        let pl = g.player_mut(owner);
        remove_card(&mut pl.exile, card);
        pl.hand.push(card);
        crate::glog!(g, "    {} may play {} again (Soul Partition)", pname(g, owner), cname(g, card));
    }
    Ok("gy")
}


// ================================================================== Calamity's Wake
/// "Players can't cast noncreature spells this turn" is on (Calamity's Wake; the flag sits on whoever cast it): the
/// engine's cast check reads this for every player
pub fn noncreature_lock(g: &Game) -> bool {
    g.players.iter().any(|q| flag_now(g, q.id, "calamity_lock"))
}

/// Calamity's Wake: exile all graveyards; players can't cast noncreature spells this turn; exile Calamity's Wake
fn calamity_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mut n = 0;
    for i in 0..g.players.len() {
        let pl = &mut g.players[i];
        n += pl.gy.len();
        let gy = std::mem::take(&mut pl.gy);
        pl.exile.extend(gy);
    }
    crate::glog!(g, "    {} exiles {} cards from all graveyards (Calamity's Wake)", pname(g, p), n);
    set_flag(g, p, "calamity_lock");
    Ok("exile")
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Untimely Malfunction")?.resolve = Some(malfunction_resolve);
    r.card(db, "Pyrokinesis")?.resolve = Some(pyro_resolve);
    r.card(db, "Scout's Warning")?.resolve = Some(scout_resolve);
    r.card(db, "Kozilek's Command")?.resolve = Some(kozilek_resolve);
    r.card(db, "Eldrazi Confluence")?.resolve = Some(confluence_resolve);
    r.card(db, "Get Lost")?.resolve = Some(get_lost_resolve);
    r.card(db, "Erode")?.resolve = Some(erode_resolve);
    r.card(db, "Soul Partition")?.resolve = Some(soul_partition_resolve);
    r.card(db, "Calamity's Wake")?.resolve = Some(calamity_resolve);
    Ok(())
}
