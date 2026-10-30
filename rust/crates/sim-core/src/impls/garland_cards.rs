//! Cards of the Garland deck that the compiled Oracle text and the engine's tags don't cover (the audit verdicts are
//! in data/card_overrides.toml; each function's doc comment says how the card is modeled and what is left out).
//!
//! - **Coerced to Kill**: a control Aura (a `zur.rs` lock kind "coerce": `coerce_attach` / `coerce_detach`).
//! - **Skycoach Conductor // All Aboard**: enters prepared; while prepared, {U} casts a copy of All Aboard (blink a
//!   non-Pilot creature of ours: Garland most of all).
//! - **Specimen Freighter**: enter: return up to two non-Spacecraft creatures; station; 9+ a 4/7 flier that mills four.
//! - **Tragic Lesson**, **Amphin Pathmage**, **Lazav, Wearer of Faces**.

use super::partials::eot_kw;
use crate::cards::CardDb;
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::trigger_window;
use crate::engine::values::{epow, pval, untargetable};
use crate::engine::zones::{bounce, draw, max_by, mill};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
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

// ================================================================== Coerced to Kill
/// **Coerced to Kill** (Aura, {3}{U}{B}): you control enchanted creature; it has base power and toughness 1/1, has
/// deathtouch and is an Assassin. `zur::lock_attach` calls `coerce_attach` once the Aura is on the battlefield: the
/// creature changes sides (the same move as Garland's steal, but for good), and its old state is kept on the Aura.
/// Leaving the battlefield (the Aura destroyed or exiled) gives the creature back to its owner as it was. The AI casts
/// it on the best opposing creature (`zur::lock_host`, the same pick as the lock Auras).
pub fn coerce_attach(g: &mut Game, a: PermId, host: PermId) {
    let p = owner(g, a);
    let x = g.perm(host);
    let saved = vec![Val::Int(x.pow as i64), Val::Int(x.tgh as i64), Val::Bool(x.dt)];
    g.perm_mut(a).data.set(DataKey::Held, Val::List(saved));
    if g.perm(host).owner != p {
        super::marchesa::steal(g, p, host, false);
    }
    g.perm_mut(a).attached = Some(host);
    let x = g.perm_mut(host);
    (x.pow, x.tgh, x.dt) = (1, 1, true);
    g.bf_ver += 1;
}

/// the Aura left: the creature is itself again and goes back to its owner
pub fn coerce_detach(g: &mut Game, a: PermId) {
    if g.perm(a).cd.is_none_or(|c| &*g.db.get(c).name != "Coerced to Kill") {
        return;
    }
    let Some(Val::List(v)) = g.perm(a).data.get(DataKey::Held).cloned() else { return };
    g.perm_mut(a).data.set(DataKey::Held, Val::None);
    let Some(host) = g.perm(a).attached.filter(|&h| g.perm(h).on_bf) else { return };
    let int = |i: usize| if let Some(Val::Int(n)) = v.get(i) { *n as i32 } else { 0 };
    let dt = matches!(v.get(2), Some(Val::Bool(true)));
    let x = g.perm_mut(host);
    (x.pow, x.tgh, x.dt) = (int(0), int(1), dt);
    let (cur, orig) = (g.perm(host).owner, g.perm(host).orig);
    if cur != orig {
        g.player_mut(cur).perms.retain(|&m| m != host);
        if g.player(orig).alive {
            g.perm_mut(host).owner = orig;
            g.player_mut(orig).perms.push(host);
            crate::glog!(g, "    {} returns to {} (Coerced to Kill left)", g.perm(host).name, g.player(orig).name);
        } else {
            g.perm_mut(host).on_bf = false;
        }
    }
    g.bf_ver += 1;
}

// ================================================================== Skycoach Conductor // All Aboard
const CONDUCTOR: &str = "Skycoach Conductor // All Aboard";

/// enters prepared
fn conductor_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.perm_mut(src).data.set(DataKey::Prepared, Val::Bool(true));
    }
    Ok(())
}

/// the creature All Aboard blinks: ours (it returns under its owner's control), not a Pilot, worth entering again
fn aboard_target(g: &Game, p: PlayerId, src: PermId) -> Option<(PermId, f64)> {
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            m != src
                && live(g, m)
                && g.is_creature(m)
                && g.perm(m).owner == p
                && g.perm(m).orig == p
                && !g.perm(m).token
                && g.perm(m).cd.is_some_and(|c| !g.db.get(c).has_subtype("pilot"))
        })
        .collect();
    let best = max_by(&cands, |m| super::blue_cards::etb_score(g, p, m))?;
    Some((best, super::blue_cards::etb_score(g, p, best)))
}

fn conductor_options(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src) || g.perm(src).phased || !g.perm(src).data.truthy(DataKey::Prepared) {
        return Ok(vec![]);
    }
    let Some((_, s)) = aboard_target(g, p, src) else { return Ok(vec![]) };
    if s < 2.5 || !can_pay(g, p, 0, "U", false) {
        return Ok(vec![]);
    }
    Ok(vec![ability(s - 0.5, "All Aboard (Skycoach Conductor, prepared copy)", src, aboard_go, 0)])
}

/// cast a copy of All Aboard ({U}): blink the target, unprepare
fn aboard_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let Some((t, s)) = aboard_target(g, p, src) else { return Ok(false) };
    if s < 2.5 || !g.perm(src).data.truthy(DataKey::Prepared) || !pay(g, p, 0, "U", false)? {
        return Ok(false);
    }
    g.perm_mut(src).data.set(DataKey::Prepared, Val::Bool(false));
    super::mine::cast_copy_with(g, p, None, None, true, 3.0, &mut |g| {
        if live(g, t) {
            crate::glog!(g, "    All Aboard: {} is exiled and returns", g.perm(t).name);
            super::t2::blink(g, p, t)?;
        }
        Ok(())
    })?;
    Ok(true)
}

// ================================================================== Specimen Freighter
const FREIGHTER: &str = "Specimen Freighter";
const STATION: i64 = 9;

fn charge(g: &Game, m: PermId) -> i64 {
    if let Some(Val::Int(n)) = g.perm(m).data.get(DataKey::Charge) { *n } else { 0 }
}

/// **Specimen Freighter** ({5}{U} Spacecraft 4/7): when it enters, return up to two target non-Spacecraft creatures to
/// their owners' hands: our creature with the best enter trigger (Garland most of all, to crown again) and the
/// opposing creature worth most. Station: tap another creature of ours, one charge counter per power; at 9 it is a
/// 4/7 flying artifact creature, and whenever it attacks the defending player mills four.
fn freighter_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || p != owner(g, src) {
        return Ok(());
    }
    let mut picks: Vec<PermId> = vec![];
    if let Some((t, s)) = super::blue_cards::best_own_etb(g, p, Some(src), false)
        && s >= 2.0
    {
        picks.push(t);
    }
    if let Some(t) = super::common::best_opp_creature(g, p, |x| !picks.contains(&x))
        && pval(g, t) >= 4.0
    {
        picks.push(t);
    }
    if picks.is_empty() || !trigger_window(g, p, Some(src), "return up to two creatures to their owners' hands", Some(4.0))? {
        return Ok(());
    }
    for t in picks {
        if live(g, t) && (owner(g, t) == p || !untargetable(g, t)) {
            bounce(g, t)?;
        }
    }
    Ok(())
}

fn freighter_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(true) || p != owner(g, src) || g.active != Some(p) || charge(g, src) >= STATION {
        return Ok(vec![]);
    }
    if station_pool(g, p, src).is_empty() {
        return Ok(vec![]);
    }
    Ok(vec![ability(1.2, "Station (Specimen Freighter)", src, freighter_go, 0)])
}

/// the creatures that can station: untapped, ours, not the ones that should stay up (Garland, a stolen creature with
/// no summoning sickness left is better attacking, but after combat everything untapped is free to tap)
fn station_pool(g: &Game, p: PlayerId, src: PermId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| m != src && live(g, m) && g.is_creature(m) && !g.perm(m).tapped && epow(g, m) > 0)
        .collect()
}

fn freighter_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let pool = station_pool(g, p, src);
    if pool.is_empty() || !live(g, src) {
        return Ok(false);
    }
    // the weakest first: each tap is a blocker fewer, so stop once nine are reached
    let mut pool = pool;
    pool.sort_by_key(|&m| epow(g, m));
    let mut n = charge(g, src);
    for m in pool {
        if n >= STATION {
            break;
        }
        g.perm_mut(m).tapped = true;
        n += epow(g, m) as i64;
        crate::glog!(g, "    {} stations {} (Specimen Freighter)", g.player(p).name, g.perm(m).name);
    }
    g.perm_mut(src).data.set(DataKey::Charge, Val::Int(n));
    if n >= STATION && !g.perm(src).data.truthy(DataKey::Anim) {
        let x = g.perm_mut(src);
        x.data.set(DataKey::Anim, Val::Bool(true));
        (x.pow, x.tgh, x.fly) = (4, 7, true);
        crate::glog!(g, "    Specimen Freighter is a 4/7 flying artifact creature");
    }
    g.bf_ver += 1;
    Ok(true)
}

/// whenever it attacks, defending player mills four cards
fn freighter_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    if atk.contains(&src) && p == owner(g, src) && trigger_window(g, p, Some(src), "defending player mills four", Some(2.0))? {
        mill(g, d, 4)?;
    }
    Ok(vec![])
}

// ================================================================== Tragic Lesson
/// **Tragic Lesson**: draw two cards, then discard a card unless you return a land you control to its owner's hand.
/// The AI returns a tapped land (replayed if the land drop is still there, or just kept) when the hand is short of
/// lands or the land count is high; otherwise it discards its worst card.
fn lesson_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    draw(g, p, 2, false)?;
    let pl = g.player(p);
    let lands_in_hand = pl.hand.iter().filter(|&&x| g.db.get(x).land).count();
    let tapped = pl.lands.iter().copied().find(|&l| g.perm(l).tapped);
    let spare = pl.lands.len() >= 6 || lands_in_hand == 0;
    match tapped {
        Some(l) if spare && pl.lands_played < 1 => {
            let cd = g.perm(l).card();
            crate::engine::turn::remove_land(g, p, l);
            g.player_mut(p).hand.push(cd);
            crate::glog!(g, "    {} returns {} (Tragic Lesson)", g.player(p).name, g.db.get(cd).name);
        }
        _ => crate::engine::cast::discard_worst(g, p, 1)?,
    }
    Ok("gy")
}

// ================================================================== Amphin Pathmage
/// **Amphin Pathmage**: {2}{U}: target creature can't be blocked this turn. Before combat the AI makes its biggest
/// attacker that can attack unblockable (a stolen creature first), when the opponents have untapped blockers.
fn pathmage_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || p != owner(g, src) || g.active != Some(p) || g.perm(src).phased {
        return Ok(vec![]);
    }
    if !can_pay(g, p, 2, "U", false) || pathmage_target(g, p).is_none() {
        return Ok(vec![]);
    }
    Ok(vec![ability(2.0, "Unblockable (Amphin Pathmage)", src, pathmage_go, 0)])
}

fn pathmage_target(g: &Game, p: PlayerId) -> Option<PermId> {
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
            live(g, m) && g.is_creature(m) && !x.tapped && !x.sick && !x.noatk && x.data.get(DataKey::Unbl) != Some(&Val::Stamp(st))
                && epow(g, m) >= 3
                && !x.fly
        })
        .collect();
    max_by(&cs, |m| epow(g, m) as f64 + if g.perm(m).orig != p { 2.0 } else { 0.0 })
}

fn pathmage_go(g: &mut Game, _src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let Some(t) = pathmage_target(g, p) else { return Ok(false) };
    if !pay(g, p, 2, "U", false)? {
        return Ok(false);
    }
    let st = g.turn_stamp();
    g.perm_mut(t).data.set(DataKey::Unbl, Val::Stamp(st));
    crate::glog!(g, "    {} can't be blocked this turn (Amphin Pathmage)", g.perm(t).name);
    Ok(true)
}

// ================================================================== Lazav, Wearer of Faces
/// **Lazav**: whenever it attacks, exile target card from a graveyard (the best creature card, an opponent's first),
/// then investigate. Whenever you sacrifice a Clue, Lazav may become a copy of a creature card exiled with it until
/// end of turn: here it takes that card's power, toughness and evasion keywords (flying, menace, trample, deathtouch,
/// lifelink, haste, vigilance); the copied creature's triggered and activated abilities are not copied.
fn lazav_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if !atk.contains(&src) || p != owner(g, src) {
        return Ok(vec![]);
    }
    let mut best: Option<(f64, PlayerId, CardId)> = None;
    for q in g.players.iter().filter(|x| x.alive).map(|x| x.id) {
        for &c in &g.player(q).gy {
            let d = g.db.get(c);
            let v = if d.creature { 10.0 + d.pow as f64 + d.cmc as f64 } else { 1.0 } + if q != p { 5.0 } else { 0.0 };
            if best.is_none_or(|b| v > b.0) {
                best = Some((v, q, c));
            }
        }
    }
    let Some((_, q, c)) = best else { return Ok(vec![]) };
    if !trigger_window(g, p, Some(src), "exile a card from a graveyard, then investigate", Some(3.0))? {
        return Ok(vec![]);
    }
    if let Some(i) = g.player(q).gy.iter().position(|&x| x == c) {
        g.player_mut(q).gy.remove(i);
        g.player_mut(q).exile.push(c);
        crate::glog!(g, "    Lazav exiles {}", g.db.get(c).name);
        if g.db.get(c).creature {
            let mut v = match g.perm(src).data.get(DataKey::Held) {
                Some(Val::List(v)) => v.clone(),
                _ => vec![],
            };
            v.push(Val::Card(c));
            g.perm_mut(src).data.set(DataKey::Held, Val::List(v));
        }
    }
    crate::engine::zones::make_artifact_tokens(g, p, "Clue", 1)?;
    Ok(vec![])
}

fn lazav_sacrifice(g: &mut Game, src: Src, p: PlayerId, what: crate::hooks::Sacrificed) -> Res {
    if p != owner(g, src) || what != crate::hooks::Sacrificed::Token("Clue") || !live(g, src) {
        return Ok(());
    }
    let Some(Val::List(v)) = g.perm(src).data.get(DataKey::Held).cloned() else { return Ok(()) };
    let cards: Vec<CardId> = v.iter().filter_map(|x| if let Val::Card(c) = x { Some(*c) } else { None }).collect();
    let cur = (epow(g, src), crate::engine::values::etgh(g, src));
    let Some(c) = max_by(&cards, |c| {
        let d = g.db.get(c);
        (d.pow + d.tgh) as f64
    }) else {
        return Ok(());
    };
    let db = g.db.clone();
    let d = db.get(c);
    if d.pow + d.tgh <= cur.0 + cur.1 || !trigger_window(g, p, Some(src), "become a copy of a creature card exiled with it", Some(3.0))? {
        return Ok(());
    }
    let (dp, dt) = (d.pow - cur.0, d.tgh - cur.1);
    g.perm_mut(src).eot_pt.0 += dp;
    g.perm_mut(src).eot_pt.1 += dt;
    for kw in ["flying", "menace", "trample", "deathtouch", "lifelink", "haste", "vigilance"] {
        if d.has_kw(kw) {
            eot_kw(g, src, kw);
        }
    }
    crate::glog!(g, "    Lazav becomes a copy of {} until end of turn", g.db.get(c).name);
    g.bf_ver += 1;
    Ok(())
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, CONDUCTOR)?;
    c.etb = Some(conductor_etb);
    c.options = Some(conductor_options);
    let c = r.card(db, FREIGHTER)?;
    c.etb = Some(freighter_etb);
    c.options = Some(freighter_options);
    c.attack = Some(freighter_attack);
    r.card(db, "Tragic Lesson")?.resolve = Some(lesson_resolve);
    r.card(db, "Amphin Pathmage")?.options = Some(pathmage_options);
    let c = r.card(db, "Lazav, Wearer of Faces")?;
    c.attack = Some(lazav_attack);
    c.sacrifice = Some(lazav_sacrifice);
    Ok(())
}

// ================================================================== Xande, Dark Mage
/// **Xande** (menace): +1/+1 for each noncreature, nonland card in its owner's graveyard
fn xande_pt(g: &Game, p: PlayerId, _m: PermId) -> (i32, i32) {
    let n = g.player(p).gy.iter().filter(|&&c| !g.db.get(c).creature && !g.db.get(c).land).count() as i32;
    (n, n)
}

fn xande_enters(g: &mut Game, _p: PlayerId, _m: PermId) -> Res {
    g.selfpt = true;
    Ok(())
}

fn xande_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.selfpt = true;
    }
    Ok(())
}

// ================================================================== Potioner's Trove
/// **Potioner's Trove**: {T}: any colour of mana (the rock tag); {T}: gain 2 life, only if we cast an instant or
/// sorcery this turn. The AI gains the life at the end of the turn, when the mana was not needed.
fn trove_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if caster == owner(g, src) && (g.db.get(c).instant || g.db.get(c).sorcery) {
        let st = g.turn_stamp();
        g.perm_mut(src).data.set(DataKey::Hit, Val::Stamp(st));
    }
    Ok(())
}

fn trove_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let st = g.turn_stamp();
    if p != owner(g, src) || g.perm(src).tapped || g.perm(src).phased || g.perm(src).data.get(DataKey::Hit) != Some(&Val::Stamp(st)) {
        return Ok(vec![]);
    }
    let u = if post.is_none() { 1.5 } else { 0.4 };
    Ok(vec![ability(u, "gain 2 life (Potioner's Trove)", src, trove_go, 0)])
}

fn trove_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let st = g.turn_stamp();
    if g.perm(src).tapped || g.perm(src).data.get(DataKey::Hit) != Some(&Val::Stamp(st)) {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    crate::engine::life::gain(g, p, 2)?;
    Ok(true)
}

pub fn register_more(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Xande, Dark Mage")?;
    c.self_pt = Some(xande_pt);
    c.as_enters = Some(xande_enters);
    c.etb = Some(xande_etb);
    let c = r.card(db, "Potioner's Trove")?;
    c.cast = Some(trove_cast);
    c.options = Some(trove_options);
    Ok(())
}
