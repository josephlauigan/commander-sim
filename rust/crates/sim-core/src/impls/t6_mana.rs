//! Tier 6 "mana" batch: the mana rocks, rituals, dorks and mana-adjacent artifacts of the Tier 6 decks that the
//! compiled Oracle text and the engine's tags don't cover (the audit verdicts are in data/card_overrides.toml; each
//! card's doc comment says how it is modeled and what is left out; what needs the shared engine is in
//! rust/agents/engine-needs-mana.md).

use super::partials::at_once;
use crate::cards::CardDb;
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::values::epow;
use crate::impls::t5::is_artifact;
use crate::tag::Tag;
use crate::engine::zones::{die, draw, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::Sym;

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn ability(utility: f64, label: &str, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label: label.into(), act: Some(Action::Ability { src, f, arg }) }
}

// ================================================================== Chromatic Star
/// **Chromatic Star** ({1} artifact): {1}, {T}, sacrifice: one mana of any colour (a mana ability: floating mana of
/// any colour of the player's identity); when it is put into a graveyard from the battlefield, draw a card (any way
/// it dies: sacrificed or destroyed). The AI cracks it for the card in the second main phase or the end-of-turn
/// window when it has a spare {1}. The data override drops the old `rock` tag: Star is not a free mana rock.
fn star_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post == Some(false) || !live(g, src) || owner(g, src) != p || g.perm(src).tapped || !can_pay(g, p, 1, "", false)
    {
        return Ok(vec![]);
    }
    Ok(vec![ability(1.0, "crack Chromatic Star", src, star_go, 0)])
}

fn star_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).tapped {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if !pay(g, p, 1, "", false)? {
        g.perm_mut(src).tapped = false;
        return Ok(false);
    }
    crate::glog!(g, "  {} cracks Chromatic Star", g.player(p).name);
    die(g, src, "sac")?;
    g.player_mut(p).floating.any += 1;
    Ok(true)
}

/// when it is put into a graveyard from the battlefield: draw a card
fn star_dies(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, m);
    if trigger_window(g, o, Some(m), "draw a card", None)? {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

// ================================================================== Manifold Key
/// how much mana tapping artifact m makes (its `rock` tag, or the card code's own count), 0 if it makes none
fn rock_amt(g: &Game, p: PlayerId, m: PermId) -> u32 {
    let Some(c) = g.perm(m).cd else { return 0 };
    let Some(rock) = g.db.get(c).tags.str(Tag::Rock) else { return 0 };
    let a = rock.split_once(':').map_or(rock, |x| x.0);
    crate::cardcode::dyn_mana_perm(g, p, m).unwrap_or_else(|| a.parse().unwrap_or(1))
}

/// the tapped mana artifact worth untapping: it must make two or more mana (untapping pays back the {1} and more)
fn key_untap_target(g: &Game, p: PlayerId, key: PermId) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| m != key && live(g, m) && g.perm(m).tapped && is_artifact(g, m) && rock_amt(g, p, m) >= 2)
        .collect();
    max_by(&cs, |m| rock_amt(g, p, m) as f64)
}

/// the creature that gets "can't be blocked" before combat: the biggest untapped attacker (three or more power) when
/// the opponents have an untapped blocker (the same pick as Amphin Pathmage's)
fn key_unblockable_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let blockers = g.opps(p).any(|q| g.player(q).perms.iter().any(|&m| live(g, m) && g.is_creature(m) && !g.perm(m).tapped));
    if !blockers {
        return None;
    }
    let st = g.turn_stamp();
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            live(g, m)
                && g.is_creature(m)
                && !x.tapped
                && !x.sick
                && !x.noatk
                && x.data.get(DataKey::Unbl) != Some(&Val::Stamp(st))
                && epow(g, m) >= 3
        })
        .collect();
    max_by(&cs, |m| epow(g, m) as f64)
}

/// **Manifold Key** ({1} artifact): {1}, {T}: untap another target artifact (the AI untaps a tapped mana artifact
/// that makes two or more mana: Sol Ring, Basalt Monolith, Grim Monolith ... when it holds a spell); {3}, {T}: target
/// creature can't be blocked this turn (before combat, on the biggest attacker, when blockers are untapped).
fn key_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !live(g, src) || owner(g, src) != p || g.perm(src).tapped {
        return Ok(vec![]);
    }
    let mut o = vec![];
    if can_pay(g, p, 1, "", false)
        && key_untap_target(g, p, src).is_some()
        && g.player(p).hand.iter().any(|&c| !g.db.get(c).land)
    {
        o.push(ability(2.0, "untap an artifact (Manifold Key)", src, key_untap, 0));
    }
    if post == Some(false) && g.active == Some(p) && can_pay(g, p, 3, "", false) && key_unblockable_target(g, p).is_some()
    {
        o.push(ability(2.0, "unblockable (Manifold Key)", src, key_unblock, 0));
    }
    Ok(o)
}

fn key_untap(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).tapped {
        return Ok(false);
    }
    let Some(t) = key_untap_target(g, p, src) else { return Ok(false) };
    g.perm_mut(src).tapped = true;
    if !pay(g, p, 1, "", false)? {
        g.perm_mut(src).tapped = false;
        return Ok(false);
    }
    if !ability_window(g, p, Some(src), "untap an artifact", None, None)? {
        return Ok(true);
    }
    if live(g, t) {
        g.perm_mut(t).tapped = false;
        crate::glog!(g, "    Manifold Key untaps {}", g.perm(t).name);
    }
    Ok(true)
}

fn key_unblock(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).tapped {
        return Ok(false);
    }
    let Some(t) = key_unblockable_target(g, p) else { return Ok(false) };
    g.perm_mut(src).tapped = true;
    if !pay(g, p, 3, "", false)? {
        g.perm_mut(src).tapped = false;
        return Ok(false);
    }
    if !ability_window(g, p, Some(src), "can't be blocked", None, None)? {
        return Ok(true);
    }
    let st = g.turn_stamp();
    g.perm_mut(t).data.set(DataKey::Unbl, Val::Stamp(st));
    crate::glog!(g, "    {} can't be blocked this turn (Manifold Key)", g.perm(t).name);
    Ok(true)
}

// ================================================================== Desperate Ritual
/// **Desperate Ritual** ({1}{R} Arcane instant): add {R}{R}{R} (floating red mana). Splice onto Arcane is not
/// modeled: no other Arcane spell is played alongside it in any deck (Godo is the only deck that plays it). Like
/// Cabal Ritual, the AI casts it only through a deck plan (priority 0).
fn desperate_ritual(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    g.player_mut(p).floating.r += 3;
    crate::glog!(g, "    {} adds RRR (Desperate Ritual)", g.player(p).name);
    Ok("gy")
}

// ================================================================== Rite of Flame
/// **Rite of Flame** ({R} sorcery): add {R}{R}, then {R} for each card named Rite of Flame in each graveyard (the
/// copies already there: this one is still on the stack). Floating red mana; the AI casts it only through a deck plan.
fn rite_of_flame(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let copies = g.players.iter().flat_map(|q| q.gy.iter()).filter(|&&x| &*g.db.get(x).name == "Rite of Flame").count();
    g.player_mut(p).floating.r += 2 + copies as u32;
    crate::glog!(g, "    {} adds {} red (Rite of Flame)", g.player(p).name, 2 + copies);
    Ok("gy")
}

fn zero_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Chromatic Star")?;
    c.options = Some(star_options);
    c.self_dies = Some(star_dies);
    at_once(c, Event::SelfDies, false);
    let c = r.card(db, "Desperate Ritual")?;
    c.resolve = Some(desperate_ritual);
    c.prio = Some(zero_prio);
    r.card(db, "Manifold Key")?.options = Some(key_options);
    let c = r.card(db, "Rite of Flame")?;
    c.resolve = Some(rite_of_flame);
    c.prio = Some(zero_prio);
    Ok(())
}
