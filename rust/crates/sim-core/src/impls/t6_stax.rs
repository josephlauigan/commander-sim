//! Tier 6 "stax" batch: the stax and Silence-effect cards of JD's Dack, Kraum & Tymna and Sisay decks that the compiled
//! Oracle text and the engine's tags don't cover (rust/agents/t6-batches.md). Cards only: no engine file is touched, so
//! what needs the shared engine (a player-level "can't cast spells" check, split second, a kicker choice, "can't lose",
//! enters tapped for opponents, a counter-on-cast hook) is written as a plain `pub fn` the engine can call later and
//! listed in rust/agents/engine-needs-stax.md. Each card's doc comment says how it is modeled and what is left out.
//!
//! - Silence, Orim's Chant, Mandate of Peace: the target player(s) get the engine's player cast lock
//!   (`engine::cast::lock_this_turn`, `LockKind::AllSpells`).
//! - Orim's Chant also sets `stax_no_attack` when kicked (`attack_lock`).
//! - Angel's Grace: the turn flag `grace_active`.

use crate::cards::CardDb;
use crate::engine::zones::max_by;
use crate::flow::Res;
use crate::engine::life::{gain, lose_life};
use crate::engine::mana::{can_pay, pay, total_mana};
use crate::engine::stack::trigger_window;
use crate::engine::cast::{casts_this_turn, lock_this_turn};
use crate::hooks::{Event, Registry, Src};
use crate::state::{DataKey, LockKind, Val};
use crate::tag::Tag;
use crate::ids::PermId;
use super::t5::is_artifact;
use crate::ids::{CardId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::{Sym, intern};

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn set_flag(g: &mut Game, p: PlayerId, key: &str) {
    let stamp = g.turn_stamp();
    g.player_mut(p).flag_turn.insert(intern(key), stamp);
}

fn flag_now(g: &Game, p: PlayerId, key: &str) -> bool {
    g.player(p).flag_turn.get(intern(key)) == Some(&g.turn_stamp())
}

fn never_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

// ================================================================== Silence effects
/// p can't cast spells this turn (Silence, Orim's Chant, Mandate of Peace): the engine's player cast lock
fn lock_player(g: &mut Game, p: PlayerId) {
    lock_this_turn(g, p, LockKind::AllSpells);
}

/// **Silence** ({W} instant): your opponents can't cast spells this turn (`LockKind::AllSpells` on each, so a flash
/// creature is refused too). The AI never casts it (priority 0): when it wins a combo turn is the deck plan's call.
fn silence_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    for q in g.opps(p).collect::<Vec<_>>() {
        lock_player(g, q);
    }
    crate::glog!(g, "    {}'s opponents can't cast spells this turn (Silence)", pname(g, p));
    Ok("gy")
}


/// the turn flag a kicked Orim's Chant sets on its caster: creatures can't attack this turn (the engine's attack
/// declaration should return no attackers for anyone while it is set)
pub fn attack_lock(g: &Game) -> bool {
    g.players.iter().any(|q| flag_now(g, q.id, "stax_no_attack"))
}

/// **Orim's Chant** ({W} instant, kicker {W}): target player can't cast spells this turn; if kicked, creatures can't
/// attack this turn. The target is the face the spell was aimed at, else the opponent with the most cards in hand.
/// Partial: the lock is real; the kicker needs the engine to offer and pay it (a
/// kicked cast sets `ctx.kicks`, which nothing does yet), and the no-attack rule needs an attack check
/// (engine-needs-stax.md). Kicked, the flag `attack_lock` is set. The AI never casts it (priority 0).
fn chant_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let target = ctx.face.or_else(|| max_by(&opps, |o| g.player(o).hand.len() as f64));
    if let Some(t) = target {
        lock_player(g, t);
        crate::glog!(g, "    {} can't cast spells this turn (Orim's Chant)", pname(g, t));
    }
    if ctx.kicks > 0 {
        set_flag(g, p, "stax_no_attack");
        crate::glog!(g, "    creatures can't attack this turn (kicked Orim's Chant)");
    }
    Ok("gy")
}


// ================================================================== Mandate of Peace
/// is it combat (creatures are attacking), the only time Mandate of Peace may be cast; for the engine's cast check
/// and the AI once either offers it
pub fn in_combat_now(g: &Game) -> bool {
    !g.in_combat.is_empty()
}

/// **Mandate of Peace** ({1}{W} instant, only during combat): your opponents can't cast spells this turn; end the
/// combat phase. The lock is real; ending the combat phase is real (`Game::end_combat`: the
/// engine skips the rest of that combat, attackers and blockers leave combat, no damage; later additional combat
/// phases still happen); "cast only during combat" is not enforced
/// by the engine for a card in hand (`in_combat_now` says when). The AI never casts it (priority 0).
fn mandate_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    for q in g.opps(p).collect::<Vec<_>>() {
        lock_player(g, q);
    }
    if !g.in_combat.is_empty() {
        g.end_combat = true;
    }
    crate::glog!(g, "    {}'s opponents can't cast spells and combat ends (Mandate of Peace)", pname(g, p));
    Ok("gy")
}


// ================================================================== Angel's Grace
/// Angel's Grace is in force for p this turn: p can't lose the game and p's opponents can't win it (the engine's
/// `check_state` should skip eliminating p, as it does for Herald of Eternal Dawn via `t0_giada::cant_lose`)
pub fn grace_active(g: &Game, p: PlayerId) -> bool {
    flag_now(g, p, "stax_grace")
}

/// life total after damage under Angel's Grace: damage that would reduce p's life total to less than 1 reduces it to 1
/// instead (the engine's `lose_life` for damage should clamp with this)
pub fn grace_floor(g: &Game, p: PlayerId, life_after: i32) -> i32 {
    if grace_active(g, p) { life_after.max(1) } else { life_after }
}

/// **Angel's Grace** ({W} instant, split second): you can't lose the game this turn, your opponents can't win this
/// turn, and damage can't take you below 1 life this turn. Split second is real (the card's keyword: `cast::split_second_up`). Partial: resolving it sets the turn flag
/// (`grace_active`, `grace_floor`) but the engine does not read it yet ("can't lose" is a later feature)
/// (engine-needs-stax.md). The AI never casts it (priority 0): a card that does nothing today is not worth casting.
fn grace_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    set_flag(g, p, "stax_grace");
    crate::glog!(g, "    {} can't lose the game this turn (Angel's Grace)", pname(g, p));
    Ok("gy")
}


// ================================================================== Blind Obedience
fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

/// **Blind Obedience** ({1}{W} enchantment): extort, and artifacts and creatures your opponents control enter tapped.
/// Extort: whenever its controller casts a spell they may pay {W/B}; if they do, each opponent loses 1 life and they
/// gain that much (the AI pays when it has spare mana, the rule Crypt Ghast's extort uses in `partials.rs`; each
/// Blind Obedience triggers separately). Partial: "enter tapped" is a replacement the engine lacks, so the opponents'
/// creatures and artifacts (tokens too) are tapped by this enters hook right after they enter (like Thalia, Heretic
/// Cathar's): a haste attacker is still stopped, but an enters trigger that untaps it or uses "if it entered tapped"
/// sees it untapped (engine-needs-stax.md). Artifact tokens (Treasure, Food, Clue) are counters in this engine, so
/// "enters tapped" has nothing to act on there.
fn obedience_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let x = g.perm(m);
    if m == src || !g.perm(src).on_bf || owner(g, m) == owner(g, src) || !x.on_bf {
        return Ok(());
    }
    let artifact_token = x.token && x.ttypes.contains(&"artifact");
    if g.is_creature(m) || artifact_token || is_artifact(g, m) {
        g.perm_mut(m).tapped = true;
    }
    Ok(())
}

/// the opponents' creature tokens that just entered (a token made by `make_tokens` does not go through the enters hook)
fn obedience_tokens(g: &mut Game, src: Src, p: PlayerId, toks: &[PermId]) -> Res {
    if !g.perm(src).on_bf || p == owner(g, src) {
        return Ok(());
    }
    for &m in toks {
        if g.perm(m).on_bf && g.is_creature(m) {
            g.perm_mut(m).tapped = true;
        }
    }
    Ok(())
}

/// extort: pay {W/B} (white if it can, else black) when casting a spell with spare mana
fn obedience_extort(g: &mut Game, src: Src, caster: PlayerId, _c: CardId) -> Res {
    if caster != owner(g, src) || g.perm(src).phased || !g.perm(src).on_bf {
        return Ok(());
    }
    let hybrid = |g: &Game| -> Option<&'static str> {
        if total_mana(g, caster, false) < 3 {
            None
        } else if can_pay(g, caster, 0, "W", false) {
            Some("W")
        } else if can_pay(g, caster, 0, "B", false) {
            Some("B")
        } else {
            None
        }
    };
    if hybrid(g).is_none() || !trigger_window(g, caster, Some(src), "extort", None)? {
        return Ok(());
    }
    let Some(sym) = hybrid(g) else { return Ok(()) };
    pay(g, caster, 0, sym, false)?;
    let mut n = 0;
    for q in g.opps(caster).collect::<Vec<_>>() {
        lose_life(g, q, 1, Some(caster), intern("drain"), None)?;
        n += 1;
    }
    gain(g, caster, n)?;
    Ok(())
}


// ================================================================== Damping Sphere
/// is `src` the Damping Sphere that counts: with two out, the land rule applies once (it is one replacement,
/// "produces {C} instead", not two reductions), so only the earliest one acts
fn first_sphere(g: &Game, src: Src) -> bool {
    !g.players.iter().any(|q| {
        q.perms.iter().any(|&m| {
            let x = g.perm(m);
            m.0 < src.0 && x.on_bf && !x.phased && x.cd.is_some_and(|c| &*g.db.get(c).name == "Damping Sphere")
        })
    })
}

/// **Damping Sphere** ({2} artifact): a land tapped for two or more mana produces {C} instead of any other type and
/// amount; each spell a player casts costs {1} more for each other spell that player has cast this turn.
/// This is the land half: a land that taps for two or more makes one mana (the `land_mana` slot, as Blood Moon's cut
/// in `t6_godo.rs`; it sees only the land's own amount, so another static adding mana to the same land is not
/// cut). The colour is not changed: a two-mana land of another colour (a bounce land) should make {C}, and card code
/// has no colour override for a land. Partial (engine-needs-stax.md).
fn sphere_land(g: &Game, src: Src, p: PlayerId, l: crate::ids::LandId) -> i32 {
    if !g.perm(src).on_bf || !first_sphere(g, src) {
        return 0;
    }
    let base = crate::cardcode::dyn_mana_land(g, p, l)
        .map_or_else(|| g.db.get(g.land(l).cd).tags.int(Tag::Amt).unwrap_or(1), |n| n as i32);
    if base >= 2 { 1 - base } else { 0 }
}

/// Damping Sphere's cost half (exact): each spell costs {1} more for each other spell its caster has cast this turn,
/// for every player, the Sphere's controller included; lands are not spells
fn sphere_cost(g: &Game, _src: Src, caster: PlayerId, c: CardId) -> i32 {
    if g.db.get(c).land { 0 } else { casts_this_turn(g, caster) }
}


// ================================================================== Pithing Needle
/// the card name a Pithing Needle was set to as it entered (`DataKey::Imprint` holds the named card), if it named one
pub fn needle_name(g: &Game, m: PermId) -> Option<CardId> {
    match g.perm(m).data.get(DataKey::Imprint) {
        Some(Val::Card(c)) => Some(*c),
        _ => None,
    }
}

/// do the activated abilities of sources named `name` stop (not mana abilities): a Pithing Needle on the battlefield
/// names it. For the engine's ability checks (`engine-needs-stax.md`); `values::stopped` is the Flute's check and also
/// taxes spells, so it can't just be extended.
pub fn needle_stops(g: &Game, name: &str) -> bool {
    g.players.iter().any(|q| {
        q.perms.iter().any(|&m| {
            let x = g.perm(m);
            x.on_bf
                && !x.phased
                && !x.neutered
                && x.cd.is_some_and(|c| &*g.db.get(c).name == "Pithing Needle")
                && needle_name(g, m).is_some_and(|c| &*g.db.get(c).name == name)
        })
    })
}

/// **Pithing Needle** ({1} artifact): as it enters, choose a card name; activated abilities of sources with that name
/// can't be activated unless they're mana abilities. The AI names what Disruptor Flute would name (`flute_pick`: the
/// opposing commander or engine permanent that hurts most). Partial: the name is chosen and kept (`needle_name`), and
/// `needle_stops` answers the rule, but the engine's activation checks don't ask it yet (engine-needs-stax.md).
fn needle_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if let Some((name, _)) = crate::ai::decks::flute_pick(g, o) {
        g.perm_mut(src).data.set(DataKey::Imprint, Val::Card(name));
        crate::glog!(g, "    Pithing Needle names {}", g.db.get(name).name);
    }
    Ok(())
}


// ================================================================== Chalice of the Void
/// the X the AI casts Chalice of the Void for: one, the stax pilot's usual setting (it stops Sol Ring, the one-mana
/// Moxen, Mana Vault ...; the spec's line D). The card costs {X}{X}, so two mana.
const CHALICE_X: i32 = 1;

/// the charge counters on a Chalice (`DataKey::Charge`)
pub fn chalice_counters(g: &Game, m: PermId) -> i32 {
    g.perm(m).data.int(DataKey::Charge) as i32
}

/// {X}{X} with X fixed at one: two generic mana (the data has no mana cost for it)
fn chalice_cost(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    2 * CHALICE_X
}

/// enters with X charge counters
fn chalice_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.perm_mut(src).data.set(DataKey::Charge, Val::Int(CHALICE_X as i64));
        crate::glog!(g, "    Chalice of the Void enters with {} charge counter(s)", CHALICE_X);
    }
    Ok(())
}

/// **Chalice of the Void** ({X}{X} artifact): enters with X charge counters; whenever a player casts a spell with
/// mana value equal to the number of charge counters, counter that spell. Modeled as a lock like Vexing Bauble's
/// (`can_cast`, `t6_godo.rs`): nobody can cast a nonland spell whose mana value equals the counters. Partial: X is
/// always 1 (the engine has no X choice for a card with no X tag, and the data gives it no mana cost; `self_cost`
/// makes it cost {2}); the real card lets the cast happen and counters it (cast triggers happen, the card goes to the
/// graveyard, a free alternative cost is caught too), which needs a counter-on-cast hook (engine-needs-stax.md).
fn chalice_can_cast(g: &Game, src: Src, _p: PlayerId, c: CardId, _zone: Sym) -> bool {
    let d = g.db.get(c);
    if !g.perm(src).on_bf || d.land {
        return true;
    }
    d.cmc as i32 != chalice_counters(g, src)
}


// ================================================================== Void Mirror
/// **Void Mirror** ({2} artifact): whenever a player casts a spell, if no coloured mana was spent to cast it, counter
/// that spell. Modeled as a lock like Vexing Bauble's (`can_cast`): a spell with a coloured mana symbol in its cost
/// always spends coloured mana and is allowed; a spell that costs nothing spends none and is refused; a spell with
/// only generic mana is refused when the caster has no untapped source of coloured mana (Sol Ring paid from Ancient
/// Tomb and City of Traitors), and allowed when it has one (it can pay with coloured mana; the engine's payment may
/// still use colourless sources, which the real card would counter). Partial: the real card lets the spell be cast and
/// counters it by the mana actually spent, and catches free alternative costs: needs the counter-on-cast hook
/// (engine-needs-stax.md).
fn mirror_can_cast(g: &Game, src: Src, p: PlayerId, c: CardId, _zone: Sym) -> bool {
    if !g.perm(src).on_bf || g.db.get(c).land {
        return true;
    }
    let (gn, pips) = crate::engine::mana::cost_of(g, p, c);
    if pips.chars().any(|ch| "WUBRG".contains(ch)) {
        return true;
    }
    gn > 0 && "WUBRG".chars().any(|ch| can_pay(g, p, 0, &ch.to_string(), false))
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Silence")?;
    c.resolve = Some(silence_resolve);
    c.prio = Some(never_prio);
    let c = r.card(db, "Orim's Chant")?;
    c.resolve = Some(chant_resolve);
    c.prio = Some(never_prio);
    let c = r.card(db, "Mandate of Peace")?;
    c.resolve = Some(mandate_resolve);
    c.prio = Some(never_prio);
    let c = r.card(db, "Angel's Grace")?;
    c.resolve = Some(grace_resolve);
    c.prio = Some(never_prio);
    let c = r.card(db, "Blind Obedience")?;
    c.etb = Some(obedience_etb);
    c.tokens_enter = Some(obedience_tokens);
    *c = c.at_once(Event::TokensEnter);
    *c = c.at_once(Event::Etb);
    c.cast = Some(obedience_extort);
    let c = r.card(db, "Damping Sphere")?;
    c.land_mana = Some(sphere_land);
    c.cost = Some(sphere_cost);
    let c = r.card(db, "Pithing Needle")?;
    c.etb = Some(needle_etb);
    *c = c.at_once(Event::Etb);
    let c = r.card(db, "Chalice of the Void")?;
    c.etb = Some(chalice_etb);
    *c = c.at_once(Event::Etb);
    c.can_cast = Some(chalice_can_cast);
    c.self_cost = Some(chalice_cost);
    r.card(db, "Void Mirror")?.can_cast = Some(mirror_can_cast);
    Ok(())
}
