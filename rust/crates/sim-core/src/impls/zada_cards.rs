//! The Zada deck's supporting cards (deck key `zada`; Zada himself, the Zada spells and the spell payoffs are
//! `zada.rs`'s): the small creatures, the burn, the Sagas and a few artifacts. The mechanics built here are
//! - prepared (`mine.rs`'s PREPARED table: Blazing Firesinger, Goblin Glasswright and Strife Scholar),
//! - intimidate (`rules::evasion_blocked`) and
//! - plot (Irascible Wolverine: `plot_*` below).
//!
//! Cards whose text is fully covered by their tags (Orcish Cannonade, Reiterating Bolt ...) have no code here, only
//! an entry in data/card_overrides.toml; the tests in tests/cards_zada_cards.rs cover them too.

use super::lands::{can_pay_without, pay_without};
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{bolt_something, cast_card, discard_worst};
use crate::engine::life::lose_life;
use crate::engine::mana::{Source, Unit, can_pay, mana_units, pay, spend_unit};
use crate::engine::removal::apply_removal;
use crate::engine::removal::legal_targets;
use crate::engine::stack::{ability_window, ability_window_card, trigger_window};
use crate::engine::values::{epow, etgh, has_type, pval, untargetable};
use crate::engine::zones::{Tokens, bounce, die, draw, make_tokens, max_by, min_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::Sym;
use crate::tag::Tag;

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

/// 1 damage (or n) to any target: a creature worth killing, else the weakest opponent
fn ping(g: &mut Game, p: PlayerId, n: i32) -> Res {
    bolt_something(g, p, n)
}

/// p's artifacts: the permanents, plus the Treasure, Food and Clue tokens (kept as counts)
pub fn artifacts(g: &Game, p: PlayerId) -> i32 {
    let perms = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| {
            let x = g.perm(m);
            !x.phased
                && (x.cd.is_some_and(|c| g.db.get(c).types.has(crate::cards::Types::ARTIFACT))
                    || x.ttypes.contains(&"artifact")
                    || x.data.truthy(DataKey::Artifact))
        })
        .count();
    let pl = g.player(p);
    (perms as u32 + pl.treasures + pl.foods + pl.clues) as i32
}

fn legendary_creature(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| {
        let x = g.perm(m);
        !x.phased && g.is_creature(m) && x.cd.is_some_and(|c| g.db.get(c).tag(Tag::Leg))
    })
}

/// the top card of p's library exiled; None if the library is empty
fn exile_top(g: &mut Game, p: PlayerId) -> Option<CardId> {
    let c = g.player_mut(p).library.pop()?;
    g.player_mut(p).seen.insert(c);
    Some(c)
}

/// a card exiled "you may play it until the end of your next turn"
fn playable_until_next_turn(g: &mut Game, p: PlayerId, c: CardId) {
    let pl = g.player_mut(p);
    pl.hand.push(c);
    let until = pl.turns + 1;
    pl.impulse_long.push((c, until));
}

// ------------------------------------------------------------------ Goblin Arsonist, Bomber Corps
/// Goblin Arsonist: when it dies, you may have it deal 1 damage to any target
fn arsonist_dies(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, m);
    if trigger_window(g, o, Some(m), "1 damage to any target", None)? {
        ping(g, o, 1)?;
    }
    Ok(())
}

/// Bomber Corps, battalion: it and at least two other creatures attack: 1 damage to any target
fn bomber_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) == p
        && atk.contains(&src)
        && atk.len() >= 3
        && g.perm(src).on_bf
        && trigger_window(g, p, Some(src), "battalion: 1 damage to any target", None)?
    {
        ping(g, p, 1)?;
    }
    Ok(vec![])
}

// ------------------------------------------------------------------ Glitterfang, Goblin Rimerunner, Battlecarver
/// Glitterfang: at the beginning of the end step, return it to its owner's hand (haste is in its keywords)
fn glitterfang_end(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    if g.perm(src).on_bf && trigger_window(g, owner(g, src), Some(src), "return to hand", None)? && g.perm(src).on_bf {
        bounce(g, src)?;
    }
    Ok(())
}

/// Goblin Rimerunner: {T}: target creature can't block this turn. The AI uses it as it attacks, on the best
/// untapped creature that could block. ({S}: haste needs snow mana, which no card of the deck makes.)
fn rimerunner_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    let x = g.perm(src);
    if x.owner != p || !x.on_bf || x.tapped || x.phased || atk.contains(&src) || x.neutered {
        return Ok(vec![]);
    }
    if x.sick && !crate::dsl::has_kw(g, src, "haste") {
        return Ok(vec![]);
    }
    let blockers: Vec<PermId> = g
        .player(d)
        .perms
        .iter()
        .copied()
        .filter(|&b| {
            g.is_creature(b)
                && !g.perm(b).tapped
                && !g.perm(b).phased
                && !untargetable(g, b)
                && atk.iter().any(|&a| crate::engine::combat::can_block(g, b, a))
        })
        .collect();
    let Some(b) = max_by(&blockers, |b| pval(g, b)) else { return Ok(vec![]) };
    g.perm_mut(src).tapped = true;
    if ability_window(g, p, Some(src), "target creature can't block this turn", Some(2.0), Some(b))? && g.perm(b).on_bf
    {
        g.dsl_on = true; // the engine reads "can't block" only while the ability language is on
        let k = &mut g.perm_mut(b).eot_kw;
        if !k.contains(&"cant_block") {
            k.push("cant_block");
        }
        crate::glog!(g, "    {} can't block this turn", g.perm(b).name);
    }
    Ok(vec![])
}

/// Skilled Battlecarver: first strike during its controller's turn
fn battlecarver_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    m == src && kw == "first strike" && g.active == Some(owner(g, src))
}

// ------------------------------------------------------------------ Rundvelt Hordemaster
/// the top card of the library exiled; a Goblin creature card may be cast until the end of your next turn
fn hordemaster_exile(g: &mut Game, src: Src, dead: PermId) -> Res {
    let o = owner(g, dead);
    if g.player(o).library.is_empty() || !trigger_window(g, o, Some(src), "exile the top card", None)? {
        return Ok(());
    }
    let Some(c) = exile_top(g, o) else { return Ok(()) };
    let d = g.db.get(c);
    let goblin = d.creature && d.subtypes.iter().any(|s| &**s == "goblin");
    crate::glog!(g, "    Rundvelt Hordemaster exiles {}{}", d.name, if goblin { " (castable)" } else { "" });
    if goblin {
        playable_until_next_turn(g, o, c);
    } else {
        g.player_mut(o).exile.push(c);
    }
    Ok(())
}

/// another Goblin you control dies (the Hordemaster itself is `hordemaster_self_dies`)
fn hordemaster_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if m != src && owner(g, m) == owner(g, src) && has_type(g, m, "goblin") {
        hordemaster_exile(g, src, m)?;
    }
    Ok(())
}

fn hordemaster_self_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    hordemaster_exile(g, src, m)
}

// ------------------------------------------------------------------ Pia, Determined Rebuilder
/// When Pia enters, a 1/1 colourless Thopter artifact creature token with flying
fn pia_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, p, Some(src), "create a 1/1 Thopter", None)? {
        let spec = Tokens {
            fly: true,
            color: Some(Colors::from_letters("")),
            types: vec!["thopter", "artifact"],
            ..Tokens::new(1, 1)
        };
        make_tokens(g, p, spec)?;
    }
    Ok(())
}

/// {5}{R}: target creature gets +X/+0 until end of turn, X = artifacts you control. The AI pays it as it attacks,
/// for the attacker that gains the most (an evasive one first), when X is at least 2.
fn pia_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) != p || !g.perm(src).on_bf || g.perm(src).neutered || g.perm(src).phased {
        return Ok(vec![]);
    }
    let x = artifacts(g, p);
    if x < 2 || !crate::engine::mana::can_pay(g, p, 5, "R", false) {
        return Ok(vec![]);
    }
    let Some(t) = max_by(atk, |a| {
        let m = g.perm(a);
        (if m.fly { 3.0 } else { 0.0 }) + (if m.dt { 1.0 } else { 0.0 }) + epow(g, a) as f64 * 0.1
    }) else {
        return Ok(vec![]);
    };
    crate::engine::mana::pay(g, p, 5, "R", false)?;
    if ability_window(g, p, Some(src), "+X/+0 (X = artifacts)", Some(3.0), Some(t))? && g.perm(t).on_bf {
        crate::impls::common::eot(g, t, x, 0);
        crate::glog!(g, "    Pia gives {} +{x}/+0", g.perm(t).name);
    }
    Ok(vec![])
}

// ------------------------------------------------------------------ the burn
/// Wrath of the Bloodmane costs {1} less with a legendary creature
fn wrath_cost(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if legendary_creature(g, p) { -1 } else { 0 }
}

/// Suplex: 3 damage to a creature (exiled instead if it would die), or exile an artifact. The engine's removal logic
/// picks the target (creatures first; `alsoart` lets it pick an artifact when no creature is worth the burn).
fn suplex(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = ctx.target
        && g.perm(t).on_bf
    {
        if g.is_creature(t) {
            apply_removal(g, Some(p), t, "dmg3", Some(c))?;
        } else {
            apply_removal(g, Some(p), t, "exile", Some(c))?;
        }
    }
    Ok("gy")
}

/// Heated Argument: 6 damage to a creature; you may exile a card from your graveyard, and if you do, 2 damage to
/// that creature's controller. The AI exiles the least valuable card (and always does when it has one).
fn heated_argument(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target else { return burn_face(g, p, c, ctx, 6) };
    let victim = owner(g, t);
    apply_removal(g, Some(p), t, "dmg6", Some(c))?;
    let gy: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&x| x != c).collect();
    if let Some(x) = min_by(&gy, |x| crate::engine::tutors::card_worth(g, p, x, true)) {
        let pl = g.player_mut(p);
        let i = pl.gy.iter().position(|&y| y == x).unwrap();
        pl.gy.remove(i);
        pl.exile.push(x);
        crate::glog!(
            g,
            "    Heated Argument exiles {} from the graveyard: 2 damage to {}",
            g.db.get(x).name,
            g.player(victim).name
        );
        lose_life(g, victim, 2, Some(p), "burn", None)?;
    }
    Ok("gy")
}

fn burn_face(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx, n: i32) -> Res<Sym> {
    if let Some(q) = ctx.face {
        lose_life(g, q, n, Some(p), "burn", None)?;
    }
    Ok("gy")
}

/// Violent Echoes: 6 damage to a creature or planeswalker; excess damage empowers Jace by that much
fn violent_echoes(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) else { return Ok("gy") };
    let toughness = if g.is_creature(t) { etgh(g, t) } else { g.perm(t).loyalty.unwrap_or(0) };
    let excess = 6 - toughness;
    apply_removal(g, Some(p), t, "dmg6", Some(c))?;
    if excess > 0 && !g.perm(t).on_bf {
        crate::impls::alela::empower(g, p, excess);
    }
    Ok("gy")
}

// ------------------------------------------------------------------ Sagas
const ROMAN: [&str; 5] = ["", "I", "II", "III", "IV"];

/// a Saga's chapter n (the lore counter is already added): the chapter's trigger, then the sacrifice after the last
fn chapter(
    g: &mut Game,
    src: PermId,
    p: PlayerId,
    n: i64,
    last: i64,
    what: &str,
    run: fn(&mut Game, PermId, PlayerId, i64) -> Res,
) -> Res {
    let name = format!("chapter {}: {what}", ROMAN[n as usize]);
    if trigger_window(g, p, Some(src), &name, None)? && g.perm(src).on_bf {
        run(g, src, p, n)?;
    }
    if n >= last && g.perm(src).on_bf {
        die(g, src, "sac")?; // the final chapter is gone: sacrifice the Saga
    }
    Ok(())
}

/// as the Saga enters: one lore counter (chapter I), then each of its controller's precombat main phases
fn saga_enter(
    g: &mut Game,
    src: PermId,
    p: PlayerId,
    m: PermId,
    last: i64,
    what: &str,
    run: fn(&mut Game, PermId, PlayerId, i64) -> Res,
) -> Res {
    if m != src {
        return Ok(());
    }
    g.perm_mut(src).data.set(DataKey::Lore, Val::Int(1));
    chapter(g, src, p, 1, last, what, run)
}

fn saga_main(
    g: &mut Game,
    src: PermId,
    p: PlayerId,
    last: i64,
    what: [&str; 4],
    run: fn(&mut Game, PermId, PlayerId, i64) -> Res,
) -> Res {
    if p != owner(g, src) || !g.perm(src).on_bf || g.perm(src).data.get(DataKey::Lore).is_none() {
        return Ok(());
    }
    let n = g.perm(src).data.int(DataKey::Lore) + 1;
    if n > last {
        return Ok(());
    }
    g.perm_mut(src).data.set(DataKey::Lore, Val::Int(n));
    chapter(g, src, p, n, last, what[n as usize - 1], run)
}

/// Burn, Burn, Tree and Fern: I 6 damage to an opposing creature, II destroy an opposing artifact, III and IV add {R}
fn burn_run(g: &mut Game, _src: PermId, p: PlayerId, n: i64) -> Res {
    match n {
        1 => {
            let t = legal_targets(g, p, "dmg6", "c", false, None);
            if let Some(t) = max_by(&t, |t| pval(g, t)) {
                apply_removal(g, Some(p), t, "dmg6", None)?;
            }
        }
        2 => {
            let t = legal_targets(g, p, "destroy", "a", false, None);
            if let Some(t) = max_by(&t, |t| pval(g, t)) {
                apply_removal(g, Some(p), t, "destroy", None)?;
            }
        }
        _ => {
            g.player_mut(p).floating.r += 1;
            crate::glog!(g, "    Burn, Burn, Tree and Fern adds {{R}}");
        }
    }
    Ok(())
}

const BURN_TEXT: [&str; 4] = ["6 damage to a creature", "destroy an artifact", "add {R}", "add {R}"];

fn burn_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    saga_enter(g, src, p, m, 4, BURN_TEXT[0], burn_run)
}

fn burn_main(g: &mut Game, src: Src, p: PlayerId) -> Res {
    saga_main(g, src, p, 4, BURN_TEXT, burn_run)
}

fn burn_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if legal_targets(g, p, "dmg6", "c", false, None).is_empty() { 20 } else { 58 }
}

/// Ral and the Implicit Maze: I 2 damage to each creature and planeswalker opponents control, II may discard a card:
/// if you do, exile the top two (playable until the end of your next turn), III a Spellgorger Weird
fn ral_run(g: &mut Game, _src: PermId, p: PlayerId, n: i64) -> Res {
    match n {
        1 => {
            for q in g.opps(p).collect::<Vec<_>>() {
                for m in g.player(q).perms.clone() {
                    let walker = g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::PLANESWALKER));
                    if !g.perm(m).on_bf || g.perm(m).phased {
                        continue;
                    }
                    // the engine's damage removal is lethal, so a creature is hit only when 2 damage kills it
                    // (damage that doesn't kill is not tracked); a planeswalker loses 2 loyalty
                    if g.is_creature(m) {
                        if etgh(g, m) <= 2 {
                            apply_removal(g, Some(p), m, "dmg2", None)?;
                        }
                    } else if walker {
                        let l = g.perm(m).loyalty.unwrap_or(0);
                        if l <= 2 {
                            apply_removal(g, Some(p), m, "dmg6", None)?;
                        } else {
                            g.perm_mut(m).loyalty = Some(l - 2);
                        }
                    }
                }
            }
        }
        2 => {
            if !g.player(p).hand.is_empty() && g.player(p).library.len() >= 2 {
                discard_worst(g, p, 1)?;
                for _ in 0..2 {
                    if let Some(c) = exile_top(g, p) {
                        crate::glog!(g, "    Ral and the Implicit Maze exiles {}", g.db.get(c).name);
                        playable_until_next_turn(g, p, c);
                    }
                }
            }
        }
        _ => {
            let spec = Tokens { color: Some(Colors::from_letters("R")), types: vec!["weird"], ..Tokens::new(1, 2) };
            for t in make_tokens(g, p, Tokens { tgh: Some(2), ..spec })? {
                g.perm_mut(t).data.set(DataKey::WeirdToken, Val::Bool(true));
            }
        }
    }
    Ok(())
}

const RAL_TEXT: [&str; 4] =
    ["2 damage to each opposing creature and planeswalker", "discard, exile two", "Spellgorger Weird", ""];

fn ral_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    saga_enter(g, src, p, m, 3, RAL_TEXT[0], ral_run)
}

fn ral_main(g: &mut Game, src: Src, p: PlayerId) -> Res {
    saga_main(g, src, p, 3, RAL_TEXT, ral_run)
}

fn ral_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let hurt = g.opps(p).any(|q| g.player(q).perms.iter().any(|&m| g.is_creature(m) && etgh(g, m) <= 2));
    if hurt { 64 } else { 50 }
}

// ------------------------------------------------------------------ Command the Stage, Glimpse the Impossible
/// Command the Stage: a 2/2 Wizard Soldier Cadet, then a +1/+1 counter on each other Wizard token you control
fn command_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let spec = Tokens {
        tgh: Some(2),
        types: vec!["wizard", "soldier"],
        color: Some(Colors::from_letters("")),
        ..Tokens::new(1, 2)
    };
    let new = make_tokens(g, p, spec)?;
    for m in g.player(p).perms.clone() {
        let x = g.perm(m);
        if x.token && x.on_bf && x.ttypes.contains(&"wizard") && !new.contains(&m) {
            g.perm_mut(m).plus += 1;
            crate::glog!(g, "    a +1/+1 counter on a Wizard token");
        }
    }
    Ok("gy")
}

/// at each upkeep, if an opponent was dealt noncombat damage last turn, Command the Stage returns to its owner's hand
fn command_upkeep(g: &mut Game, c: CardId, q: PlayerId) -> Res {
    if !g.opps(q).any(|o| g.player(o).nc_dmg_last) || !g.player(q).gy.contains(&c) {
        return Ok(());
    }
    if !trigger_window(g, q, None, "return Command the Stage to hand", None)? {
        return Ok(());
    }
    let st = g.turn_stamp();
    let pl = g.player_mut(q);
    if let Some(i) = pl.gy.iter().position(|&x| x == c) {
        pl.gy.remove(i);
        pl.hand.push(c);
        pl.gy_left = Some(st);
        crate::glog!(g, "    Command the Stage returns to {}'s hand", g.player(q).name);
    }
    Ok(())
}

fn command_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    54
}

/// Glimpse the Impossible: exile the top three, playable this turn; at the end step the unplayed ones go to the
/// graveyard and each makes a 0/1 Eldrazi Spawn (engine/turn.rs calls `glimpse_spawn` for each)
fn glimpse_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mut names = vec![];
    for _ in 0..3 {
        let Some(c) = exile_top(g, p) else { break };
        names.push(g.db.get(c).name.to_string());
        let pl = g.player_mut(p);
        pl.hand.push(c);
        pl.impulse.push(c);
        pl.glimpse_exiled.push(c);
    }
    crate::glog!(g, "    Glimpse the Impossible exiles {} (playable this turn)", names.join(", "));
    Ok("gy")
}

/// a card left exiled by Glimpse went to the graveyard: a 0/1 Eldrazi Spawn
pub fn glimpse_spawn(g: &mut Game, p: PlayerId) -> Res {
    let spec = Tokens {
        tgh: Some(1),
        types: vec!["eldrazi", "spawn"],
        color: Some(Colors::from_letters("")),
        ..Tokens::new(1, 0)
    };
    make_tokens(g, p, spec)?;
    Ok(())
}

fn glimpse_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if g.player(p).library.len() >= 3 { 50 } else { 0 }
}

// ------------------------------------------------------------------ Living History
fn history_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, p, Some(src), "create a 2/2 Spirit", None)? {
        let spec = Tokens {
            tgh: Some(2),
            types: vec!["spirit"],
            color: Some(Colors::from_letters("RW")),
            ..Tokens::new(1, 2)
        };
        make_tokens(g, p, spec)?;
    }
    Ok(())
}

/// whenever you attack, if a card left your graveyard this turn, target attacking creature gets +2/+0
fn history_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) != p || g.player(p).gy_left != Some(g.turn_stamp()) || !g.perm(src).on_bf {
        return Ok(vec![]);
    }
    let Some(t) = max_by(atk, |a| {
        let m = g.perm(a);
        (if m.fly { 3.0 } else { 0.0 }) + (if m.dt { 1.0 } else { 0.0 }) + epow(g, a) as f64 * 0.1
    }) else {
        return Ok(vec![]);
    };
    if trigger_window(g, p, Some(src), "target attacking creature gets +2/+0", None)? && g.perm(t).on_bf {
        crate::impls::common::eot(g, t, 2, 0);
        crate::glog!(g, "    Living History gives {} +2/+0", g.perm(t).name);
    }
    Ok(vec![])
}

fn history_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    58
}

// ------------------------------------------------------------------ Irascible Wolverine (plot)
/// a card plotted on turn t is marked in `impulse_long` with `u32::MAX - 1 - t` (it stays in the hand; the engine
/// keeps cards that may be played from exile there)
fn plotted_turn(g: &Game, p: PlayerId, c: CardId) -> Option<u32> {
    g.player(p)
        .impulse_long
        .iter()
        .find(|x| x.0 == c && x.1 < u32::MAX && x.1 > u32::MAX / 2)
        .map(|x| u32::MAX - 1 - x.1)
}

/// When it enters, exile the top card of your library; you may play that card this turn
fn wolverine_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src
        && !g.player(p).library.is_empty()
        && trigger_window(g, p, Some(src), "exile the top card, may play it this turn", None)?
        && let Some(c) = exile_top(g, p)
    {
        crate::glog!(g, "    Irascible Wolverine exiles {} (playable this turn)", g.db.get(c).name);
        let pl = g.player_mut(p);
        pl.hand.push(c);
        pl.impulse.push(c);
    }
    Ok(())
}

/// a plotted Wolverine is only cast by its own option (free, on a later turn)
fn wolverine_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    if plotted_turn(g, p, c).is_some() { 0 } else { 55 }
}

/// Plot {2}{R} (main phase after combat, when the mana is spare); cast it free from the plot on a later turn
fn wolverine_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let Some(post) = post else { return Ok(vec![]) };
    if g.active != Some(p) {
        return Ok(vec![]);
    }
    let turns = g.player(p).turns;
    Ok(match plotted_turn(g, p, c) {
        Some(t) if turns > t => vec![Opt {
            utility: 6.5,
            label: "Irascible Wolverine (plotted, free)".into(),
            act: Some(Action::Plan { f: wolverine_free, arg: c.0 as i64 }),
        }],
        Some(_) => vec![],
        None if post && can_pay(g, p, 2, "R", false) => vec![Opt {
            utility: 1.2,
            label: "plot Irascible Wolverine".into(),
            act: Some(Action::Plan { f: wolverine_plot, arg: c.0 as i64 }),
        }],
        None => vec![],
    })
}

fn wolverine_plot(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || plotted_turn(g, p, c).is_some() || !can_pay(g, p, 2, "R", false) {
        return Ok(false);
    }
    pay(g, p, 2, "R", false)?;
    let pl = g.player_mut(p);
    let mark = u32::MAX - 1 - pl.turns;
    pl.impulse_long.push((c, mark));
    crate::glog!(g, "  {} plots Irascible Wolverine", g.player(p).name);
    Ok(true)
}

fn wolverine_free(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || plotted_turn(g, p, c).is_none_or(|t| g.player(p).turns <= t) {
        return Ok(false);
    }
    g.player_mut(p).impulse_long.retain(|x| x.0 != c);
    crate::glog!(g, "  {} casts the plotted Irascible Wolverine for free", g.player(p).name);
    cast_card(g, p, c, "hand", Ctx::default())?;
    Ok(true)
}

// ------------------------------------------------------------------ Lightning, Security Sergeant
/// Whenever Lightning deals combat damage to a player, exile the top card of your library; you may play it for as
/// long as you control Lightning (kept in the hand, marked `u32::MAX` in `impulse_long`)
fn lightning_hit(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if a != src || owner(g, src) != p || g.player(p).library.is_empty() {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "exile the top card, may play it while you control Lightning", None)?
        && let Some(c) = exile_top(g, p)
    {
        crate::glog!(g, "    Lightning exiles {}", g.db.get(c).name);
        let pl = g.player_mut(p);
        pl.hand.push(c);
        pl.impulse_long.push((c, u32::MAX));
    }
    Ok(())
}

/// Lightning left the battlefield: the cards it exiled can no longer be played
fn lightning_leaves(g: &mut Game, src: Src, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    let pl = g.player_mut(o);
    let linked: Vec<CardId> = pl.impulse_long.iter().filter(|x| x.1 == u32::MAX).map(|x| x.0).collect();
    pl.impulse_long.retain(|x| x.1 != u32::MAX);
    for c in linked {
        if let Some(i) = pl.hand.iter().position(|&x| x == c) {
            pl.hand.remove(i);
            pl.exile.push(c);
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ Spawn-Gang Commander
/// When you cast this spell, create three 0/1 colourless Eldrazi Spawn tokens
fn spawn_gang_cast(g: &mut Game, p: PlayerId, _c: CardId) -> Res {
    for _ in 0..3 {
        glimpse_spawn(g, p)?;
    }
    Ok(())
}

/// the Eldrazi Spawn tokens of p that can be sacrificed
fn spawns(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            x.token && x.on_bf && !x.phased && x.ttypes.contains(&"eldrazi")
        })
        .collect()
}

/// the colourless unit that pays {C}, and the Spawn sacrificed as the cost, with {1} left over for the generic part
fn spawn_gang_plan(g: &Game, p: PlayerId) -> Option<(Unit, PermId)> {
    let sp = spawns(g, p);
    let units = mana_units(g, p, false);
    for &s in sp.iter().rev() {
        for u in units.iter().filter(|u| u.cols.is_empty() && u.src != Source::Scion(s)) {
            let rest: u32 = units.iter().filter(|w| *w != u && w.src != Source::Scion(s)).map(|w| w.amt).sum();
            if u.amt >= 1 && rest >= 1 {
                return Some((*u, s));
            }
        }
    }
    None
}

/// {1}{C}, sacrifice an Eldrazi: 2 damage to any target. The AI sacrifices a Spawn: to kill a creature, or to hit an
/// opponent at the end of the turn when it has spare Spawn.
fn spawn_gang_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if owner(g, src) != p || !g.perm(src).on_bf || g.perm(src).phased || g.perm(src).neutered {
        return Ok(vec![]);
    }
    if spawn_gang_plan(g, p).is_none() {
        return Ok(vec![]);
    }
    let kill = legal_targets(g, p, "dmg2", "c", false, None).iter().map(|&t| pval(g, t)).fold(0.0, f64::max);
    let u = if kill >= 1.5 {
        2.0 + kill * 0.3
    } else if post.is_none() && spawns(g, p).len() >= 2 {
        0.9
    } else {
        return Ok(vec![]);
    };
    Ok(vec![Opt {
        utility: u,
        label: "Spawn-Gang Commander: 2 damage".into(),
        act: Some(Action::Ability { src, f: spawn_gang_go, arg: 0 }),
    }])
}

fn spawn_gang_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let Some((u, s)) = spawn_gang_plan(g, p) else { return Ok(false) };
    spend_unit(g, p, u, 1, "")?;
    if g.perm(s).on_bf {
        die(g, s, "sac")?;
    }
    if !can_pay(g, p, 1, "", false) {
        return Ok(true);
    }
    pay(g, p, 1, "", false)?;
    if ability_window(g, p, Some(src), "2 damage to any target", Some(3.0), None)? {
        ping(g, p, 2)?;
    }
    Ok(true)
}

// ------------------------------------------------------------------ Relic of Legends
/// Tap an untapped legendary creature you control: add one mana of any colour (summoning sickness doesn't matter).
/// The AI offers a legend that is summoning sick, or any legend once combat is over or on the opponents' turns.
fn relic_mana(g: &Game, src: Src, p: PlayerId, units: &[Unit]) -> Vec<Unit> {
    if owner(g, src) != p {
        return vec![];
    }
    let used: Vec<PermId> =
        units.iter().filter_map(|w| if let Source::Perm(m) = w.src { Some(m) } else { None }).collect();
    let free_now = g.active != Some(p) || matches!(g.step, crate::state::Step::Main2 | crate::state::Step::End);
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            !x.tapped
                && !x.phased
                && g.is_creature(m)
                && !used.contains(&m)
                && x.cd.is_some_and(|c| g.db.get(c).tag(Tag::Leg))
                && (x.sick || free_now)
        })
        .map(|m| Unit { src: Source::Perm(m), cols: g.player(p).ident, amt: 1 })
        .collect()
}

// ------------------------------------------------------------------ Bonders' Enclave
/// {3}, {T}: draw a card; activate only if you control a creature with power 4 or greater (at the end of the turn)
fn enclave_options(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || g.land(l).tapped || !can_pay_without(g, p, l, 3, "") || g.player(p).library.is_empty() {
        return Ok(vec![]);
    }
    if !g.player(p).perms.iter().any(|&m| g.is_creature(m) && !g.perm(m).phased && epow(g, m) >= 4) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.3,
        label: "Bonders' Enclave: draw".into(),
        act: Some(Action::Plan { f: enclave_go, arg: l.0 as i64 }),
    }])
}

fn enclave_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId(arg as u32);
    if !g.player(p).lands.contains(&l)
        || g.land(l).tapped
        || !g.player(p).perms.iter().any(|&m| g.is_creature(m) && !g.perm(m).phased && epow(g, m) >= 4)
        || !pay_without(g, p, l, 3, "")?
    {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    if ability_window_card(g, p, g.land(l).cd, "draw a card", None, None)? {
        draw(g, p, 1, false)?;
    }
    Ok(true)
}

// ------------------------------------------------------------------ Generator Servant
/// {T}, sacrifice: add {C}{C}
fn servant_mana(_g: &Game, _p: PlayerId, _m: PermId) -> u32 {
    2
}

/// tapped for mana: sacrificed; mana not spent is kept; a creature spell it pays for gains haste
fn servant_tap(g: &mut Game, p: PlayerId, m: PermId, n: u32) -> Res {
    if n < 2 {
        g.player_mut(p).floating.c += 2 - n;
    }
    if n >= 1
        && let Some(c) = g.pay_for.filter(|&c| g.db.get(c).creature)
    {
        g.player_mut(p).haste_next.push(c);
    }
    if g.perm(m).on_bf {
        die(g, m, "sac")?;
    }
    Ok(())
}

// ------------------------------------------------------------------ registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Goblin Arsonist")?.self_dies = Some(arsonist_dies);
    r.card(db, "Bomber Corps")?.attack = Some(bomber_attack);
    r.card(db, "Glitterfang")?.end_step = Some(glitterfang_end);
    r.card(db, "Goblin Rimerunner")?.attack = Some(rimerunner_attack);
    r.card(db, "Skilled Battlecarver")?.grant_kw = Some(battlecarver_kw);
    let c = r.card(db, "Rundvelt Hordemaster")?;
    c.dies = Some(hordemaster_dies);
    c.self_dies = Some(hordemaster_self_dies);
    let c = r.card(db, "Pia, Determined Rebuilder")?;
    c.etb = Some(pia_etb);
    c.attack = Some(pia_attack);
    r.card(db, "Wrath of the Bloodmane")?.self_cost = Some(wrath_cost);
    r.card(db, "Suplex")?.resolve = Some(suplex);
    r.card(db, "Heated Argument")?.resolve = Some(heated_argument);
    r.card(db, "Violent Echoes")?.resolve = Some(violent_echoes);
    let c = r.card(db, "Burn, Burn, Tree and Fern")?;
    c.etb = Some(burn_etb);
    c.main1 = Some(burn_main);
    c.prio = Some(burn_prio);
    let c = r.card(db, "Ral and the Implicit Maze")?;
    c.etb = Some(ral_etb);
    c.main1 = Some(ral_main);
    c.prio = Some(ral_prio);
    let c = r.card(db, "Command the Stage")?;
    c.resolve = Some(command_resolve);
    c.gy_upkeep = Some(command_upkeep);
    c.prio = Some(command_prio);
    let c = r.card(db, "Glimpse the Impossible")?;
    c.resolve = Some(glimpse_resolve);
    c.prio = Some(glimpse_prio);
    let c = r.card(db, "Living History")?;
    c.etb = Some(history_etb);
    c.attack = Some(history_attack);
    c.prio = Some(history_prio);
    let c = r.card(db, "Irascible Wolverine")?;
    c.etb = Some(wolverine_etb);
    c.prio = Some(wolverine_prio);
    c.hand_options = Some(wolverine_options);
    let c = r.card(db, "Lightning, Security Sergeant")?;
    c.combat_damage = Some(lightning_hit);
    c.leaves = Some(lightning_leaves);
    let c = r.card(db, "Spawn-Gang Commander")?;
    c.self_cast = Some(spawn_gang_cast);
    c.options = Some(spawn_gang_options);
    r.card(db, "Relic of Legends")?.extra_mana = Some(relic_mana);
    r.card(db, "Bonders' Enclave")?.land_options = Some(enclave_options);
    let c = r.card(db, "Generator Servant")?;
    c.dyn_mana_perm = Some(servant_mana);
    c.on_tap_perm = Some(servant_tap);
    Ok(())
}
