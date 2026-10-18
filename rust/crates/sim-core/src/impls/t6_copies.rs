//! Tier 6, batch `copies`: the copy cards (clones, token-copy sorceries, spell copiers, Peter Parker's Camera,
//! Volatile Stormdrake).
//!
//! - **Clones** (Clever Impersonator, Phantasmal Image, Flesh Duplicate, Copy Artifact, Mirrormade, Sculpting Steel,
//!   Mockingbird): the `enter_copy` slot (engine/zones.rs `enter`) names the best legal target and the exceptions
//!   (`CopyExc`: Bird and flying, enchantment in addition, Illusion) as the permanent is created; the clone's own card
//!   stays in `phys` (so it goes to the graveyard as the clone) and the copied card's enters effects are the ones that
//!   fire. The AI casts one only when something worth copying is out (`clone_prio`). **Mockingbird** is cast through an
//!   option that picks X and the target (`Ctx::target`, read back from `cast_target`).
//! - **Token-copy sorceries** (Heat Shimmer, Molten Duplication, Electroduplicate, Flash Photography): `resolve` +
//!   `prio`; flashback is the `fb=` tag.
//! - **Twincast, Flare of Duplication**: cast from hand in response to a worthwhile instant or sorcery, as Dualcaster
//!   Mage is (`hand_cast`); Flare's alternative cost sacrifices a red nontoken creature.

use super::mine::{etb_value, is_legendary, spell_copy_value};
use crate::cards::{CardDb, Types};
use crate::engine::cast::{cast_card, copy_spell, on_cast};
use crate::engine::life::check_state;
use crate::engine::mana::{can_pay, pay, total_mana};
use crate::engine::stack::counter_window;
use crate::engine::values::{epow, pval, untargetable};
use crate::engine::zones::{die, enter_token_copy_with, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{CopyExc, CopyOf, Ctx, DataKey, Game, Val};
use crate::sym::Sym;
use crate::tag::Tag;

use super::partials::at_once;

// ------------------------------------------------------------------ what can be copied
#[derive(Clone, Copy, PartialEq, Debug)]
enum Kind {
    /// any nonland permanent (Clever Impersonator, Flash Photography)
    Any,
    /// any creature (Phantasmal Image, Flesh Duplicate, Heat Shimmer, Mockingbird)
    Creature,
    /// an artifact (Copy Artifact, Sculpting Steel)
    Artifact,
    /// an artifact or an enchantment (Mirrormade)
    ArtOrEnch,
    /// a creature you control (Electroduplicate)
    OwnCreature,
    /// an artifact or creature you control (Molten Duplication)
    OwnArtOrCreature,
}

fn kind_of(name: &str) -> Kind {
    match name {
        "Clever Impersonator" | "Flash Photography" => Kind::Any,
        "Copy Artifact" | "Sculpting Steel" => Kind::Artifact,
        "Mirrormade" => Kind::ArtOrEnch,
        "Electroduplicate" => Kind::OwnCreature,
        "Molten Duplication" => Kind::OwnArtOrCreature,
        _ => Kind::Creature,
    }
}

/// may a copy of permanent x be made for p (a nontoken card on the battlefield, not a commander, not a legendary
/// permanent p already has: the legend rule would put the copy straight back)
fn copyable(g: &Game, p: PlayerId, x: PermId, kind: Kind) -> bool {
    let pm = g.perm(x);
    let Some(cd) = pm.cd else { return false };
    if !pm.on_bf || pm.phased || pm.token || g.players.iter().any(|q| q.is_commander(cd)) {
        return false;
    }
    let creature = g.is_creature(x);
    let ty = crate::engine::values::ptypes(g, x); // a copy may have types in addition
    let art = ty.has(Types::ARTIFACT);
    let ench = ty.has(Types::ENCHANTMENT);
    let mine = pm.owner == p;
    let ok = match kind {
        Kind::Any => true,
        Kind::Creature => creature,
        Kind::Artifact => art,
        Kind::ArtOrEnch => art || ench,
        Kind::OwnCreature => creature && mine,
        Kind::OwnArtOrCreature => (creature || art) && mine,
    };
    if !ok {
        return false;
    }
    if is_legendary(g, x) && g.player(p).perms.iter().any(|&y| y != x && g.perm(y).on_bf && g.perm(y).cd == Some(cd)) {
        return false;
    }
    !(is_legendary(g, x) && mine)
}

/// what a copy of x is worth to p: its own worth, plus what entering again does
fn copy_value(g: &Game, p: PlayerId, x: PermId) -> f64 {
    let mut v = pval(g, x) + 0.5 * etb_value(g, p, x).max(0.0);
    if let Some(r) = g.perm(x).cd.and_then(|c| g.db.get(c).tags.str(Tag::Rock)) {
        v += 0.5 * r.split(':').next().and_then(|s| s.parse::<f64>().ok()).unwrap_or(1.0); // a rock: mana
    }
    v
}

/// the best permanent for p to copy, with its value (`skip`: the clone itself; `max_mv`: Mockingbird's limit)
fn best_copy(g: &Game, p: PlayerId, kind: Kind, skip: Option<PermId>, max_mv: Option<u32>) -> Option<(PermId, f64)> {
    let cands: Vec<PermId> = g
        .players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&x| {
            Some(x) != skip
                && copyable(g, p, x, kind)
                && max_mv.is_none_or(|m| g.perm(x).cd.is_some_and(|c| g.db.get(c).cmc <= m))
        })
        .collect();
    let b = max_by(&cands, |x| copy_value(g, p, x))?;
    Some((b, copy_value(g, p, b)))
}

fn name_of(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

// ------------------------------------------------------------------ clones
/// the exceptions of a clone card's copy
fn clone_exc(name: &str) -> CopyExc {
    match name {
        "Mockingbird" => CopyExc { add_subtype: Some("bird"), fly: true, ..CopyExc::default() },
        "Copy Artifact" => CopyExc { add_types: Types::ENCHANTMENT, ..CopyExc::default() },
        // Illusion in addition; the sacrifice when it becomes the target of a spell or ability needs that event
        "Phantasmal Image" => CopyExc { add_subtype: Some("illusion"), ..CopyExc::default() },
        _ => CopyExc::default(),
    }
}

/// You may have this enter as a copy of a permanent: the best legal one (the Oracle text's kind of target). Mockingbird's
/// is the creature chosen as it was cast. With nothing to copy it enters as itself (a printed 0/0 dies).
fn clone_copy(g: &mut Game, p: PlayerId, c: CardId) -> Option<CopyOf> {
    let name = name_of(g, c);
    let t = if name == MOCKINGBIRD {
        g.cast_target.filter(|&t| copyable(g, p, t, Kind::Creature))?
    } else {
        best_copy(g, p, kind_of(name), None, None)?.0
    };
    let exc = clone_exc(name);
    crate::glog!(g, "    {} enters as a copy of {}", name, g.perm(t).name);
    Some(CopyOf { cd: g.perm(t).cd?, exc })
}

/// the AI casts a clone only when something worth copying is out
fn clone_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let kind = kind_of(name_of(g, c));
    match best_copy(g, p, kind, None, None) {
        Some((_, v)) if v >= 3.0 => 50 + (3.0 * v).min(25.0) as i32,
        _ => 0,
    }
}

fn zero_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

// ------------------------------------------------------------------ Mockingbird
const MOCKINGBIRD: &str = "Mockingbird";

/// the cheapest X that reaches the best creature: (X, the creature, its value). The mana spent is X + {U}.
fn mockingbird_plan(g: &Game, p: PlayerId, c: CardId) -> Option<(u32, PermId, f64)> {
    let total = total_mana(g, p, false);
    if total < 1 || !can_pay(g, p, 0, "U", false) {
        return None;
    }
    let (t, v) = best_copy(g, p, Kind::Creature, None, Some(total))?;
    let _ = c;
    let mv = g.db.get(g.perm(t).cd?).cmc;
    let x = mv.saturating_sub(1);
    if v < 3.0 || !can_pay(g, p, x, "U", false) {
        return None;
    }
    Some((x, t, v))
}

fn mockingbird_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !g.player(p).hand.contains(&c) {
        return Ok(vec![]);
    }
    let Some((x, t, v)) = mockingbird_plan(g, p, c) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: 1.0 + 0.5 * v,
        label: format!("Mockingbird (X = {x}, copy {})", g.perm(t).name),
        act: Some(Action::Plan { f: mockingbird_go, arg: c.0 as i64 }),
    }])
}

fn mockingbird_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) {
        return Ok(false);
    }
    let Some((x, t, _)) = mockingbird_plan(g, p, c) else { return Ok(false) };
    pay(g, p, x, "U", false)?;
    // it enters as a copy of the creature chosen as it was cast (`clone_copy`)
    cast_card(g, p, c, "hand", Ctx { x: x as i32, target: Some(t), ..Ctx::default() })?;
    Ok(true)
}

// ------------------------------------------------------------------ the token-copy sorceries
/// the copy a sorcery makes: (the permanent copied, its value to p); a hasty copy that goes away is worth its
/// enters effects and one attack, a permanent copy (Flash Photography) its whole worth
fn token_value(g: &Game, p: PlayerId, x: PermId, lasting: bool) -> f64 {
    if lasting {
        return copy_value(g, p, x);
    }
    let mut v = etb_value(g, p, x).max(0.0);
    if g.is_creature(x) && !g.perm(x).noatk {
        v += 0.3 * epow(g, x).max(0) as f64;
    }
    v
}

fn best_token(g: &Game, p: PlayerId, kind: Kind, lasting: bool, only: Option<PermId>) -> Option<(PermId, f64)> {
    let cands: Vec<PermId> = g
        .players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&x| {
            copyable(g, p, x, kind) && (g.perm(x).owner == p || !untargetable(g, x)) && only.is_none_or(|o| o == x)
        })
        .collect();
    let b = max_by(&cands, |x| token_value(g, p, x, lasting))?;
    Some((b, token_value(g, p, b, lasting)))
}

fn spell_lasting(name: &str) -> bool {
    name == "Flash Photography"
}

fn token_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let n = name_of(g, c);
    match best_token(g, p, kind_of(n), spell_lasting(n), None) {
        Some((_, v)) if v >= if spell_lasting(n) { 3.5 } else { 2.5 } => 40 + (4.0 * v).min(25.0) as i32,
        _ => 0,
    }
}

/// Create a token that's a copy of a permanent. Heat Shimmer (exiled at the end step) and Electroduplicate (sacrificed
/// at the end step) and Molten Duplication (sacrificed at the next end step) give it haste; Flash Photography's copy
/// stays. A target chosen when the spell was cast (`ctx.target`) is kept if it is still there.
fn token_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let n_owned = name_of(g, c).to_string();
    let n = n_owned.as_str();
    let lasting = spell_lasting(n);
    let chosen = ctx.target.filter(|&t| copyable(g, p, t, kind_of(n)));
    let Some((t, _)) = best_token(g, p, kind_of(n), lasting, chosen) else {
        crate::glog!(g, "    {n}: no target");
        return Ok("gy");
    };
    let cd = g.perm(t).cd.unwrap();
    crate::glog!(g, "    {n}: a token copy of {}", g.perm(t).name);
    // Molten Duplication's token is an artifact in addition to its other types
    let exc = (n == "Molten Duplication").then(|| CopyExc { add_types: Types::ARTIFACT, ..CopyExc::default() });
    if let Some(m) = enter_token_copy_with(g, p, cd, exc)?
        && g.perm(m).on_bf
    {
        if !lasting {
            let x = g.perm_mut(m);
            x.sick = false; // haste
            if n == "Heat Shimmer" {
                x.temp = true; // exiled at the end step
            } else {
                x.data.set(DataKey::EotSac, Val::Bool(true)); // sacrificed at the end step
            }
        }
    }
    Ok("gy")
}

/// Flash Photography: it may be cast as though it had flash if it targets a permanent you control, so on an
/// opponent's turn (or at the end of yours) the AI copies its own best permanent
const PHOTO: &str = "Flash Photography";

fn photo_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() && g.active == Some(p) {
        return Ok(vec![]); // its own main phase: the usual cast
    }
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "UU", false) {
        return Ok(vec![]);
    }
    let mine: Vec<PermId> = g.player(p).perms.clone();
    let cands: Vec<PermId> = mine.into_iter().filter(|&x| copyable(g, p, x, Kind::Any)).collect();
    let Some(b) = max_by(&cands, |x| copy_value(g, p, x)) else { return Ok(vec![]) };
    let v = copy_value(g, p, b);
    if v < 4.0 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 0.8 + 0.3 * v,
        label: format!("{PHOTO} (flash, copy {})", g.perm(b).name),
        act: Some(Action::Plan { f: photo_go, arg: c.0 as i64 | (b.0 as i64) << 32 }),
    }])
}

fn photo_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, t) = (CardId(arg as u16), PermId((arg >> 32) as u32));
    if !g.player(p).hand.contains(&c) || !copyable(g, p, t, Kind::Any) || g.perm(t).owner != p {
        return Ok(false);
    }
    if !pay(g, p, 2, "UU", false)? {
        return Ok(false);
    }
    cast_card(g, p, c, "hand", Ctx { target: Some(t), ..Ctx::default() })?;
    Ok(true)
}

// ------------------------------------------------------------------ Twincast, Flare of Duplication
const TWINCAST: &str = "Twincast";
const FLARE: &str = "Flare of Duplication";

/// a nontoken red creature p can sacrifice to Flare of Duplication that isn't worth more than the copy
fn flare_fodder(g: &Game, p: PlayerId, worth: f64) -> Option<PermId> {
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            x.on_bf
                && !x.token
                && g.is_creature(m)
                && x.cd.is_some_and(|c| g.db.get(c).pips.contains('R') && !g.player(p).is_commander(c))
                && pval(g, m) + 1.0 < worth
        })
        .collect();
    cands.iter().copied().min_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b)))
}

/// an instant or sorcery spell was cast: copy it with the card in hand when the copy is worth it
fn copier_hook(g: &mut Game, c: CardId, p: PlayerId, spell: CardId) -> Res {
    let sd = g.db.get(spell);
    if g.in_rtf || !g.player(p).hand.contains(&c) || !(sd.instant || sd.sorcery) || spell == c {
        return Ok(());
    }
    let worth = spell_copy_value(g, p, spell);
    let flare = name_of(g, c) == FLARE;
    if worth < if flare { 4.0 } else { 3.0 } {
        return Ok(());
    }
    let by_mana = if flare { can_pay(g, p, 1, "RR", false) } else { can_pay(g, p, 0, "UU", false) };
    let fodder = if flare && !by_mana { flare_fodder(g, p, worth) } else { None };
    if !by_mana && fodder.is_none() {
        return Ok(());
    }
    g.in_rtf = true;
    let r = copier_go(g, c, p, spell, flare, if by_mana { None } else { fodder });
    g.in_rtf = false;
    r
}

fn copier_go(g: &mut Game, c: CardId, p: PlayerId, spell: CardId, flare: bool, sac: Option<PermId>) -> Res {
    let ctx = g.cur_cast.as_ref().filter(|x| x.0 == spell).map(|x| x.1.clone());
    if let Some(m) = sac {
        crate::glog!(g, "  {} sacrifices {} to cast {}", g.player(p).name, g.perm(m).name, name_of(g, c));
        die(g, m, "sac")?;
    } else if flare {
        pay(g, p, 1, "RR", false)?;
    } else {
        pay(g, p, 0, "UU", false)?;
    }
    let pl = g.player_mut(p);
    let Some(i) = pl.hand.iter().position(|&x| x == c) else { return Ok(()) };
    pl.hand.remove(i);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    crate::glog!(g, "  {} casts {}: copy {}", g.player(p).name, name_of(g, c), name_of(g, spell));
    on_cast(g, p, c)?;
    if g.over || !g.player(p).alive {
        return Ok(());
    }
    if counter_window(g, p, c, 3.0, vec![])? {
        copy_spell(g, p, spell, ctx.as_ref(), false)?;
        g.player_mut(p).gy.push(c);
    } else {
        g.player_mut(p).gy.push(c);
    }
    check_state(g)
}

// ------------------------------------------------------------------ Peter Parker's Camera
/// enters with three film counters
fn camera_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.perm_mut(src).data.set(DataKey::Counters, Val::Int(3));
    }
    Ok(())
}

/// {2}, {T}, remove a film counter: copy target activated or triggered ability you control. The AI uses it on its own
/// important triggered ability on top of the stack (`stack::ai_respond`, importance 6 or more): the copy goes on the
/// stack (it can be countered), and its effect runs once more before the original resolves (`stack::copy_trigger`).
/// The abilities that can be copied are the hook-run triggers (a stack item with its `Trigger`, e.g. Godo's enters);
/// the copies make no new choices of targets. Returns true if a copy resolved.
pub fn camera_copy(g: &mut Game, q: PlayerId, item: u32) -> Res<bool> {
    let Some(cam) = g.player(q).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        x.on_bf
            && !x.phased
            && !x.tapped
            && !x.neutered
            && &*x.name == "Peter Parker's Camera"
            && x.data.int(DataKey::Counters) > 0
    }) else {
        return Ok(false);
    };
    let Some(it) = g.stack.iter().find(|x| x.id == item).cloned() else { return Ok(false) };
    if it.controller != q || it.countered || it.ctx.trigger.is_none() || !can_pay(g, q, 2, "", false) {
        return Ok(false);
    }
    pay(g, q, 2, "", false)?;
    g.perm_mut(cam).tapped = true;
    let n = g.perm(cam).data.int(DataKey::Counters);
    g.perm_mut(cam).data.set(DataKey::Counters, Val::Int(n - 1));
    crate::glog!(g, "    {} activates Peter Parker's Camera: copies {}", g.player(q).name, it.name);
    // the ability itself is on the stack too: if it is countered, nothing is copied
    if !crate::engine::stack::ability_window(g, q, Some(cam), "copy target triggered ability", Some(3.0), None)? {
        return Ok(false);
    }
    crate::engine::stack::copy_trigger(g, q, item)
}

// ------------------------------------------------------------------ Volatile Stormdrake
/// When it enters, exchange control of it and target creature an opponent controls. If you do, you get {E}{E}{E}{E},
/// then sacrifice that creature unless you pay {E} equal to its mana value. The AI takes the best creature it can pay
/// the energy for.
fn stormdrake_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src || !g.perm(src).on_bf {
        return Ok(());
    }
    let o = g.perm(src).owner;
    let energy = g.player(o).energy + 4;
    let cands: Vec<PermId> = g
        .opps(o)
        .flat_map(|q| g.player(q).perms.clone())
        .filter(|&x| {
            let y = g.perm(x);
            y.on_bf
                && !y.phased
                && y.cd.is_some()
                && g.is_creature(x)
                && !untargetable(g, x)
                && y.cd.is_some_and(|c| g.db.get(c).cmc as i32 <= energy)
        })
        .collect();
    let Some(t) = max_by(&cands, |x| pval(g, x)) else { return Ok(()) };
    if pval(g, t) < 2.0 {
        return Ok(()); // a creature not worth giving a 3/2 flyer for
    }
    if !crate::engine::stack::trigger_window(g, o, Some(src), "exchange control", Some(5.0))?
        || !g.perm(t).on_bf
        || !g.perm(src).on_bf
    {
        return Ok(());
    }
    let q = g.perm(t).owner;
    {
        use crate::engine::control::{Duration, gain_control};
        gain_control(g, o, t, Duration::Permanent); // you control the creature
        gain_control(g, q, src, Duration::Permanent); // and they control the Drake
    }
    g.player_mut(o).energy += 4;
    let mv = g.perm(t).cd.map_or(0, |c| g.db.get(c).cmc as i32);
    if g.player(o).energy >= mv {
        g.player_mut(o).energy -= mv;
        crate::glog!(g, "    Volatile Stormdrake: {} pays {{E}} x{mv} for {}", g.player(o).name, g.perm(t).name);
    } else {
        crate::glog!(g, "    Volatile Stormdrake: {} sacrifices {}", g.player(o).name, g.perm(t).name);
        die(g, t, "sac")?;
    }
    Ok(())
}

// ------------------------------------------------------------------ registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    for n in
        ["Clever Impersonator", "Phantasmal Image", "Flesh Duplicate", "Copy Artifact", "Mirrormade", "Sculpting Steel"]
    {
        let c = r.card(db, n)?;
        c.enter_copy = Some(clone_copy);
        c.prio = Some(clone_prio);
    }
    let c = r.card(db, MOCKINGBIRD)?;
    c.enter_copy = Some(clone_copy);
    c.hand_options = Some(mockingbird_options);
    c.prio = Some(zero_prio);
    for n in ["Heat Shimmer", "Molten Duplication", "Electroduplicate", PHOTO] {
        let c = r.card(db, n)?;
        c.resolve = Some(token_resolve);
        c.prio = Some(token_prio);
    }
    r.card(db, PHOTO)?.hand_options = Some(photo_options);
    for n in [TWINCAST, FLARE] {
        let c = r.card(db, n)?;
        c.hand_cast = Some(copier_hook);
        c.hand_opp_cast = Some(copier_hook);
        c.prio = Some(zero_prio);
    }
    let c = r.card(db, "Peter Parker's Camera")?;
    c.etb = Some(camera_etb);
    at_once(c, Event::Etb, true);
    r.card(db, "Imposter Mech")?.prio = Some(zero_prio);
    let c = r.card(db, "Volatile Stormdrake")?;
    c.etb = Some(stormdrake_etb);
    Ok(())
}
