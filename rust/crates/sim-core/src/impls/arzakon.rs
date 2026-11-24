//! Arzakon ({W}{U}{B}{R}{G}, Legendary Planeswalker, loyalty 4, "can be your commander"; the `arzakon` Cali deck's
//! commander). Oracle: "+2: Arzakon deals 3 damage to any other target. -3: Create a Black Lotus token. (It's a {0}
//! artifact with "{T}, Sacrifice this token: Add three mana of any one color.") -7: Each player shuffles their hand and
//! graveyard into their library, then draws seven cards. Arzakon can be your commander."
//!
//! Card code on the walker framework (`common::walker`, `dsl = []`): the three abilities are `ARZAKON_AB`; the +2 commits
//! its target on activation (CR 602.2b, as the Theorist's -2 does) through `arz_plus_go`, so `options` swaps its action.
//!
//! - **Command zone**: nothing here. The engine casts, taxes and returns any commander card, whatever its type (Lord
//!   Windgrace is a planeswalker commander); a planeswalker deals no commander damage (CR 903.10a), as it should.
//! - **+2**: `model_gaps_a4::ping` (3 damage to a player, creature, planeswalker or battle that is not Arzakon: hexproof
//!   and protection from its colours respected; a planeswalker loses loyalty, a battle loses defense counters). The
//!   target is re-checked as the ability resolves and never re-picked (CR 608.2b); the loyalty is paid either way.
//! - **-3**: `zones::enter_token_copy` of the Black Lotus card (the token Garth One-Eye makes: three mana of one colour,
//!   sacrificed as it is tapped, `rules::petal_sac`). It is made even when the -3 takes Arzakon to 0.
//! - **-7**: `t6_blue::twister_all` (Timetwister's resolution).
//!
//! AI rules (provisional, never tuned by a sim):
//! - +2: an opponent at 3 life or less (20, lethal); else the best of an opposing creature that 3 damage kills (its
//!   `pval`, 2 more for a commander or a combo piece), an opposing planeswalker at loyalty 3 or less (`pval` + 1), and a
//!   planeswalker it can only hurt (0.3 `pval`), taken at a score of 3 or more (utility score - 0.5); else an opponent's
//!   face (2.5, 3.0 at 9 life or less, the lowest life first). Always on offer while a target exists.
//! - -3: scored by what three more mana of one colour lets p do this turn (`brain::extra_mana_play`: a catalogued line, a
//!   tutor, the commander or a good spell; that play's utility - 0.25, and 2 less if the -3 would take Arzakon to 0 for a
//!   play that is no line); with nothing to spend it on, banked: 3.6 after 2 or more loyalty (+1 for Mox Opal at metalcraft
//!   with exactly two other artifacts, +1 for Tinker in hand with no other artifact to sacrifice), 1.5 after 1, and after 0
//!   only when Arzakon would die anyway (the opposing creatures' power is at least its loyalty + 2): 2.5.
//! - -7: only when no opponent punishes the draws (Narset, Spirit of the Labyrinth, Notion Thief, Sheoldred, the
//!   Apocalypse, Orcish Bowmasters, Smothering Tithe), p holds at most 2 nonland cards, the largest opposing hand is at least
//!   3 cards bigger than p's, Underworld Breach is neither in hand nor out, and 2 or more mana is left to use the new
//!   cards; worth 1.5 per card of that difference (at most 9), 3 less if it takes Arzakon to 0.
//!
//! Left out (documented in the card's note in data/card_overrides.toml): Rings of Brighthearth never copies the abilities
//! for the AI (a person's copy asks for a new target).

use super::common::{WalkerAb, walker, walker_options, walker_pay_run};
use super::model_gaps_a4::{Tgt, ping, ping_legal};
use crate::ai::brain::{RITUAL_DISCOUNT, Situation, extra_mana_play};
use crate::cards::{CardDb, Types};
use crate::engine::mana::{Shape, Source, Unit, mana_units, total_mana};
use crate::engine::pool::Kind;
use crate::engine::values::{epow, etgh, hand_count, has, indestructible, is_artifact_perm, pval};
use crate::engine::zones::enter_token_copy;
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{PermId, PlayerId};
use crate::state::Game;
use crate::tag::Tag;

pub const ARZAKON: &str = "Arzakon";
const LOTUS: &str = super::garth::LOTUS;
/// the damage of the +2
const DAMAGE: i32 = 3;
/// the loyalty cost of the -7
const WHEEL: i32 = -7;

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

// ------------------------------------------------------------------ +2: 3 damage to any other target
/// the targets the +2 may be aimed at, for the AI and a person alike: the living players, and the creatures, planeswalkers
/// and battles on the table other than Arzakon itself ("any other target"), that p's Arzakon `src` can legally hit
pub fn plus_targets(g: &Game, p: PlayerId, src: PermId) -> Vec<Tgt> {
    let mut out: Vec<Tgt> = vec![];
    for q in g.players.iter().filter(|q| q.alive) {
        let t = Tgt::Player(q.id);
        if ping_legal(g, p, src, t) {
            out.push(t);
        }
    }
    for q in g.players.iter().filter(|q| q.alive) {
        for &m in &q.perms {
            if m != src && ping_legal(g, p, src, Tgt::Perm(m)) {
                out.push(Tgt::Perm(m));
            }
        }
    }
    out
}

/// is m an opposing planeswalker
fn is_walker(g: &Game, m: PermId) -> bool {
    g.perm(m).loyalty.is_some() && g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::PLANESWALKER))
}

/// The AI's +2 target and its utility (None: nothing to aim at). Lethal on an opponent first; else the best permanent
/// worth 3 or more; else an opponent's face.
fn plus_plan(g: &Game, p: PlayerId, src: PermId) -> Option<(Tgt, f64)> {
    let legal = plus_targets(g, p, src);
    let opp_players: Vec<PlayerId> =
        legal.iter().filter_map(|&t| if let Tgt::Player(q) = t { Some(q) } else { None }).filter(|&q| q != p).collect();
    // an opponent the 3 damage kills
    if let Some(&q) = opp_players.iter().filter(|&&q| g.player(q).life <= DAMAGE).min_by_key(|&&q| g.player(q).life) {
        return Some((Tgt::Player(q), 20.0 - 0.5));
    }
    let mut best: Option<(Tgt, f64)> = None;
    for &t in &legal {
        let Tgt::Perm(m) = t else { continue };
        let x = g.perm(m);
        if x.owner == p {
            continue;
        }
        let score = if g.is_creature(m) {
            if etgh(g, m) > DAMAGE || indestructible(g, m) || crate::engine::repl::no_damage(g, m) {
                continue;
            }
            let combo = x.cd.is_some_and(|c| crate::cardcode::is_combo_piece(g, c));
            pval(g, m) + if x.is_cmd || combo { 2.0 } else { 0.0 }
        } else if is_walker(g, m) {
            let l = x.loyalty.unwrap_or(0);
            if l <= DAMAGE { pval(g, m) + 1.0 } else { 0.3 * pval(g, m) }
        } else {
            continue; // a battle: the AI never aims at one
        };
        if best.is_none_or(|b| score > b.1) {
            best = Some((t, score));
        }
    }
    if let Some((t, score)) = best.filter(|b| b.1 >= 3.0) {
        return Some((t, score - 0.5));
    }
    let face = opp_players.iter().copied().min_by_key(|&q| g.player(q).life)?;
    Some((Tgt::Player(face), if g.player(face).life <= 9 { 3.0 } else { 2.5 }))
}

fn arz_plus(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    // the table's `eff`, never reached by the AI (`arz_options` swaps in `arz_plus_go`, which commits the target); the
    // best target now
    if let Some((t, _)) = plus_plan(g, p, src) {
        ping(g, p, src, t, DAMAGE)?;
        crate::engine::life::check_state(g)?;
    }
    Ok(())
}

fn plus_val(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    plus_plan(g, p, src).map(|x| x.1)
}

/// the AI's +2 (index 0 of `ARZAKON_AB`): the target is chosen now, on activation (CR 602.2b), before the loyalty is paid
/// and the ability can be responded to; as it resolves it is hit only if it is still legal (CR 608.2b)
fn arz_plus_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = Tgt::unpack(arg);
    if !ping_legal(g, p, src, t) {
        return Ok(false);
    }
    let perm = if let Tgt::Perm(m) = t { Some(m) } else { None };
    walker_pay_run(g, src, p, 0, perm, &mut |g| {
        ping(g, p, src, t, DAMAGE)?;
        crate::engine::life::check_state(g)
    })
}

// ------------------------------------------------------------------ -3: a Black Lotus token
fn lotus_unit(g: &Game, p: PlayerId) -> Unit {
    Unit { src: Source::float(Kind::Any), cols: g.player(p).ident, amt: 3, shape: Shape::One }
}

/// -3's value: what the three mana of one colour let p do this turn; else banking the token while the loyalty allows
fn lotus_val(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    let loyalty = g.perm(src).loyalty.unwrap_or(0);
    let after = loyalty - 3;
    let u = mana_units(g, p, false);
    let mut with = u.clone();
    with.push(lotus_unit(g, p));
    let s = Situation::new(g, p);
    let me = g.perm(src).cd?;
    if let Some((v, line)) = extra_mana_play(g, p, &s, me, &u, &with) {
        return Some(v - RITUAL_DISCOUNT - if after <= 0 && !line { 2.0 } else { 0.0 });
    }
    match after {
        a if a >= 2 => {
            let pl = g.player(p);
            let artifacts = pl.perms.iter().filter(|&&m| live(g, m) && is_artifact_perm(g, m)).count();
            let named = |n: &str| pl.perms.iter().any(|&m| live(g, m) && &*g.perm(m).name == n);
            // Mox Opal's metalcraft needs three artifacts: with Opal and one other, the Lotus is the third
            let opal = named("Mox Opal") && artifacts == 2;
            let tinker = pl.hand.iter().any(|&c| &*g.db.get(c).name == "Tinker") && artifacts == 0;
            Some(3.6 + if opal { 1.0 } else { 0.0 } + if tinker { 1.0 } else { 0.0 })
        }
        1 => Some(1.5),
        _ => {
            // the last loyalty: only when Arzakon would die to the table anyway
            let power: i32 = g
                .opps(p)
                .flat_map(|q| g.player(q).perms.iter().copied())
                .filter(|&m| live(g, m) && g.is_creature(m))
                .map(|m| epow(g, m))
                .sum();
            (power >= loyalty + 2).then_some(2.5)
        }
    }
}

/// -3's effect (also the person's): the Black Lotus token
pub fn lotus_token(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let Some(lotus) = g.db.id(LOTUS) else { return Ok(()) }; // (the card record stays in cards.json: export.rs)
    enter_token_copy(g, p, lotus)?;
    Ok(())
}

// ------------------------------------------------------------------ -7: the wheel
/// does an opponent of p punish the draws the wheel gives: Narset and Spirit of the Labyrinth (all but the first draw),
/// Notion Thief (extra draws), Sheoldred, the Apocalypse (2 life a draw), Orcish Bowmasters (extra draws), Smothering
/// Tithe (a Treasure)
fn draw_punisher(g: &Game, p: PlayerId) -> bool {
    g.opps(p).any(|q| [Tag::Narset, Tag::SheoA, Tag::Bowmasters, Tag::Tithe].iter().any(|&t| has(g, q, t)))
        || g.players.iter().any(|q| q.alive && has(g, q.id, Tag::Labyrinth))
        || crate::engine::hooks::hooked(g, Event::StealDraw).iter().any(|&(m, _)| g.perm(m).owner != p)
}

fn nonland_in_hand(g: &Game, p: PlayerId) -> usize {
    g.player(p).hand.iter().filter(|&&c| !g.db.get(c).land).count()
}

fn wheel_val(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    if draw_punisher(g, p) || nonland_in_hand(g, p) > 2 || total_mana(g, p, false) < 2 {
        return None;
    }
    let pl = g.player(p);
    let breach = |c| &*g.db.get(c).name == "Underworld Breach";
    if pl.hand.iter().any(|&c| breach(c)) || pl.perms.iter().any(|&m| g.perm(m).cd.is_some_and(breach)) {
        return None; // a Breach line is fuelled by the graveyard the wheel would shuffle away
    }
    let theirs = g.opps(p).map(|q| hand_count(g, q)).max()?;
    let gap = theirs as i32 - hand_count(g, p) as i32;
    if gap < 3 {
        return None;
    }
    let dies = g.perm(src).loyalty.unwrap_or(0) + WHEEL <= 0;
    Some((1.5 * gap as f64).min(9.0) - if dies { 3.0 } else { 0.0 })
}

fn wheel_eff(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    super::t6_blue::twister_all(g, p)
}

static ARZAKON_AB: [WalkerAb; 3] = [
    WalkerAb { delta: 2, label: "3 damage to any other target", val: plus_val, eff: arz_plus },
    WalkerAb { delta: -3, label: "a Black Lotus token", val: lotus_val, eff: lotus_token },
    WalkerAb {
        delta: WHEEL,
        label: "each player shuffles hand and graveyard into the library, then draws seven",
        val: wheel_val,
        eff: wheel_eff,
    },
];

/// the framework's options, with the +2 replaced by one that names its target (its utility is the plan's, not the
/// framework's `val + 0.15 delta`, and the target rides in the action's `arg`)
fn arz_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut out = walker_options(g, src, p, post)?;
    for o in &mut out {
        if !matches!(o.act, Some(Action::Ability { arg: 0, .. })) {
            continue;
        }
        let Some((t, u)) = plus_plan(g, p, src) else { continue };
        let name = match t {
            Tgt::Player(q) => g.player(q).name.to_string(),
            Tgt::Perm(m) => g.perm(m).name.to_string(),
        };
        o.utility = u;
        o.label = format!("Arzakon +2 ({name})");
        o.act = Some(Action::Ability { src, f: arz_plus_go, arg: t.pack() });
    }
    Ok(out)
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    walker(r, db, ARZAKON, &ARZAKON_AB)?;
    r.card(db, ARZAKON)?.options = Some(arz_options);
    Ok(())
}
