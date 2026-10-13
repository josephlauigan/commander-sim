//! Card code for the white cards of the cloud deck (decklists/JD/cloud-midgar-equipment.md), and face-down (morph, disguise) machinery.
//! Filled in by the white-rest agent (rust/agents/white-rest.md).
//!
//! - **Face down** (morph, disguise): **Exalted Angel**, **Forum Familiar**, **Unyielding Gatekeeper**, **Exit
//!   Specialist** (the Elrond deck's). A face-down creature is the galadriel module's 2/2 with no abilities (`neutered`);
//!   a disguise creature also has ward {2} while face down (`extra_ward`). `down_options` casts one face down for {3},
//!   `face_up_options` (called by `brain::hook_options` for every deck) turns one face up for its cost and runs its
//!   "when turned face up" ability.
//! - **Enter again**: **Ennis, Debate Moderator** and **Stormfront Riders** (and, through `bounce`, every Riders). They
//!   choose by `reenter_value`, the one place a deck plan can say what re-entering is worth (Cloud's search above all).
//! - **Recursion**: **Call a Surprise Witness**, **Recommission**.
//! - **Protection**: **Patriot, Shield Wielder**, **Shardmage's Rescue**, **Cho-Manno's Blessing**, **Ultimate Magic:
//!   Holy** answer removal through `white_protect` and wipes through `white_wipe_response` (the cloud deck's responses
//!   in `ai/decks.rs` call them).
//! - **Others**: **Bronze Guardian**, **Fate of the Sun-Cryst**, **Harsh Annotation**, **Lay Down Arms**,
//!   **Recruitment Officer**, **Repel the Darkness**.

use super::common::{AURA, AuraSpec, aura};
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{cast_card, castable};
use crate::engine::life::gain;
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::{ability_window, counter_window, trigger_window};
use crate::engine::tutors::shuffle_library;
use crate::engine::values::{card_name, epow, pval, untargetable};
use crate::engine::zones::{
    Enter, Tokens, bounce, draw, enter, exile_perm, landfall, leave, make_tokens, max_by, searchable,
};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

// ------------------------------------------------------------------ names
const EXALTED: &str = "Exalted Angel";
const FAMILIAR: &str = "Forum Familiar";
const GATEKEEPER: &str = "Unyielding Gatekeeper";
const EXIT: &str = "Exit Specialist";
const ENNIS: &str = "Ennis, Debate Moderator";
const RIDERS: &str = "Stormfront Riders";
const WITNESS: &str = "Call a Surprise Witness";
const RECOMMISSION: &str = "Recommission";
const CHO_MANNO: &str = "Cho-Manno's Blessing";
const RESCUE: &str = "Shardmage's Rescue";
const GUARDIAN: &str = "Bronze Guardian";
const FATE: &str = "Fate of the Sun-Cryst";
const ANNOTATION: &str = "Harsh Annotation";
const LAY_DOWN: &str = "Lay Down Arms";
const PATRIOT: &str = "Patriot, Shield Wielder";
const OFFICER: &str = "Recruitment Officer";
const REPEL: &str = "Repel the Darkness";
const HOLY: &str = "Ultimate Magic: Holy";
const CLOUD: &str = "Cloud, Midgar Mercenary";

// ------------------------------------------------------------------ small helpers
fn live(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased
}

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn name_of(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect()
}

fn is_artifact(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT)) || x.ttypes.contains(&"artifact")
}

fn is_equipment(g: &Game, m: PermId) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).has_subtype("equipment"))
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> Option<CardId> {
    g.player(p).hand.iter().copied().find(|&c| name_of(g, c) == name)
}

fn zero_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

fn plan(utility: f64, label: String, f: crate::hooks::PlanFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Plan { f, arg }) }
}

/// the Plains p controls (any land with the Plains type)
pub fn plains(g: &Game, p: PlayerId) -> u32 {
    g.player(p).lands.iter().filter(|&&l| g.db.get(g.land(l).cd).has_subtype("plains")).count() as u32
}

/// a flag set for the rest of this turn
fn flag_now(g: &mut Game, p: PlayerId, key: &str) {
    let st = g.turn_stamp();
    g.player_mut(p).flag_turn.insert(intern(key), st);
}

fn flag_is_now(g: &Game, p: PlayerId, key: &str) -> bool {
    g.player(p).flag_turn.get(&intern(key)) == Some(&g.turn_stamp())
}

// ------------------------------------------------------------------ what re-entering is worth
/// **What it is worth to p to have the permanent m leave and come back** (a blink, a bounce and recast): the enter
/// effects it would run again (`t2::blink_value`), Cloud, Midgar Mercenary's search for an Equipment above all, less
/// what falls off it (each Equipment attached). A token is gone for good: 0. The deck plans of the cloud deck (and
/// the Elrond deck) may add to this; Ennis, Stormfront Riders, Forum Familiar, Unyielding Gatekeeper and Exit
/// Specialist all choose their targets by it.
pub fn reenter_value(g: &Game, p: PlayerId, m: PermId) -> f64 {
    let x = g.perm(m);
    let Some(cd) = x.cd else { return 0.0 };
    if x.token {
        return 0.0;
    }
    let mut v = if x.is_cmd { 0.0 } else { super::t2::blink_value(g, p, m) as f64 };
    if name_of(g, cd) == CLOUD && g.player(x.owner).library.iter().any(|&c| g.db.get(c).has_subtype("equipment")) {
        v += 6.0;
    }
    let worn = g.player(x.owner).perms.iter().filter(|&&e| g.perm(e).attached == Some(m) && is_equipment(g, e)).count();
    v - 1.5 * worn as f64
}

// ================================================================== face down: morph and disguise
/// (generic, pips) to turn it face up, for the cards this module handles
fn up_cost(name: &str) -> Option<(u32, &'static str)> {
    match name {
        EXALTED => Some((2, "WW")),
        FAMILIAR | GATEKEEPER => Some((1, "W")),
        EXIT => Some((1, "U")),
        _ => None,
    }
}

/// m is a face-down creature of this module's cards
fn is_face_down(g: &Game, m: PermId) -> bool {
    live(g, m) && g.perm(m).data.truthy(DataKey::Facedown) && card_name(g, m).is_some_and(|n| up_cost(n).is_some())
}

/// ward {2} extra: a face-down disguise creature, and each other Bronze Guardian's controller's artifacts
/// (`green_cards::extra_ward`, which `cardcode::aura_ward` adds to the card's own ward)
pub fn extra_ward(g: &Game, m: PermId) -> u32 {
    let x = g.perm(m);
    let mut w = 0;
    if x.data.truthy(DataKey::Facedown) && x.cd.is_some_and(|c| g.db.get(c).has_kw("disguise")) {
        w += 2;
    }
    if x.cd.is_some() || x.token {
        let others = g
            .player(x.owner)
            .perms
            .iter()
            .filter(|&&e| e != m && live(g, e) && !g.perm(e).neutered && card_name(g, e) == Some(GUARDIAN))
            .count() as u32;
        if others > 0 && is_artifact(g, m) {
            w += 2 * others;
        }
    }
    w
}

/// Exit Specialist can't be blocked by creatures with power 3 or greater
pub fn exit_blocked(g: &Game, b: PermId, a: PermId) -> bool {
    let xa = g.perm(a);
    xa.cd.is_some()
        && !xa.neutered
        && !xa.data.truthy(DataKey::Facedown)
        && card_name(g, a) == Some(EXIT)
        && epow(g, b) >= 3
}

/// cast c face down for {3}: a 2/2 with no abilities (and ward {2} for a disguise card); `arg` is the card
fn cast_down(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 3, "", false) || !castable(g, p, c, "hand") {
        return Ok(false);
    }
    if !pay(g, p, 3, "", false)? {
        return Ok(false);
    }
    let pl = g.player_mut(p);
    if let Some(i) = pl.hand.iter().position(|&x| x == c) {
        pl.hand.remove(i);
    }
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    crate::glog!(
        g,
        "  {} casts a face-down creature ({})",
        pname(g, p),
        if up_cost(name_of(g, c)) == Some((2, "WW")) { "morph" } else { "disguise" }
    );
    crate::engine::cast::on_cast(g, p, c)?;
    if !counter_window(g, p, c, 3.0, vec![])? {
        g.player_mut(p).gy.push(c);
        return Ok(true);
    }
    let m = super::galadriel::enter_face_down(g, p, c)?;
    // a face-down creature has none of the card's keywords (turned face up they come back from its tags)
    let x = g.perm_mut(m);
    (x.fly, x.dt, x.vig, x.lifelink) = (false, false, false, false);
    Ok(true)
}

/// the best own permanent for Forum Familiar to send home: (its value, it)
fn familiar_target(g: &Game, p: PlayerId, fam: Option<PermId>) -> Option<(f64, PermId)> {
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| live(g, m) && Some(m) != fam && g.perm(m).cd.is_some() && !g.perm(m).token)
        .collect();
    let m = max_by(&cands, |m| reenter_value(g, p, m) - 0.05 * pval(g, m))?;
    Some((reenter_value(g, p, m), m))
}

/// the best permanent for Unyielding Gatekeeper to exile: (its value, it). Your own comes back tapped (a blink);
/// an opponent's is gone and its controller gets a 2/2 Detective
fn gatekeeper_target(g: &Game, p: PlayerId, keeper: Option<PermId>) -> Option<(f64, PermId)> {
    let mut best: Option<(f64, PermId)> = None;
    for q in g.players.iter().filter(|x| x.alive).map(|x| x.id).collect::<Vec<_>>() {
        for &m in &g.player(q).perms {
            if !live(g, m) || Some(m) == keeper || g.perm(m).cd.is_none() && !g.perm(m).token {
                continue;
            }
            let v = if q == p {
                reenter_value(g, p, m) - 0.5 // it comes back tapped
            } else {
                if untargetable(g, m) || g.perm(m).phased {
                    continue;
                }
                0.8 * pval(g, m) - 1.2 // a 2/2 Detective for them
            };
            if best.is_none_or(|b| v > b.0) {
                best = Some((v, m));
            }
        }
    }
    best
}

/// the best creature for Exit Specialist to bounce: (its value, it)
fn exit_target(g: &Game, p: PlayerId, me: Option<PermId>) -> Option<(f64, PermId)> {
    let mut best: Option<(f64, PermId)> = None;
    for q in g.players.iter().filter(|x| x.alive).map(|x| x.id).collect::<Vec<_>>() {
        for &m in &g.player(q).perms {
            if !live(g, m) || Some(m) == me || !g.is_creature(m) {
                continue;
            }
            let v = if q == p {
                reenter_value(g, p, m)
            } else {
                if untargetable(g, m) {
                    continue;
                }
                let x = g.perm(m);
                // a token is removed; a commander just pays its tax; anything else costs them the mana again
                0.5 * pval(g, m) + if x.token { 2.0 } else { 0.0 } - if x.is_cmd { 1.0 } else { 0.0 }
            };
            if best.is_none_or(|b| v > b.0) {
                best = Some((v, m));
            }
        }
    }
    best
}

/// what turning a face-down creature up is worth now (0: nothing to do with it)
fn up_value(g: &Game, p: PlayerId, m: PermId) -> f64 {
    match card_name(g, m) {
        Some(EXALTED) => 4.0,
        Some(FAMILIAR) => familiar_target(g, p, Some(m)).map_or(0.0, |x| x.0),
        Some(GATEKEEPER) => gatekeeper_target(g, p, Some(m)).map_or(0.0, |x| x.0),
        Some(EXIT) => exit_target(g, p, Some(m)).map_or(0.0, |x| x.0),
        _ => 0.0,
    }
}

/// **Turn face up** (any time, for its cost: a special action) and run "when this creature is turned face up".
fn face_up_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let m = PermId(arg as u32);
    if !is_face_down(g, m) || owner(g, m) != p {
        return Ok(false);
    }
    let name = card_name(g, m).unwrap().to_string();
    let (gn, pips) = up_cost(&name).unwrap();
    if !can_pay(g, p, gn, pips, false) || !pay(g, p, gn, pips, false)? {
        return Ok(false);
    }
    super::galadriel::turn_face_up(g, p, m);
    if let Some(cd) = g.perm(m).cd {
        let t = g.db.get(cd).tags.clone();
        let x = g.perm_mut(m);
        (x.fly, x.dt, x.vig, x.lifelink) = (t.has(Tag::Fly), t.has(Tag::Dt), t.has(Tag::Vig), t.has(Tag::Lifelink));
    }
    match name.as_str() {
        FAMILIAR => {
            // return another target permanent you control to its owner's hand and put a +1/+1 counter on this creature
            if let Some((_, t)) = familiar_target(g, p, Some(m))
                && ability_window(g, p, Some(m), "return a permanent, +1/+1 counter", Some(3.0), Some(t))?
                && live(g, t)
                && live(g, m)
            {
                bounce(g, t)?;
                crate::cardcode::add_counters(g, m, 1);
            }
        }
        GATEKEEPER => {
            if let Some((_, t)) = gatekeeper_target(g, p, Some(m))
                && ability_window(g, p, Some(m), "exile another nonland permanent", Some(4.0), Some(t))?
                && live(g, t)
            {
                let ctl = owner(g, t);
                if ctl == p {
                    // you controlled it: it returns to the battlefield tapped (a token is gone)
                    if let Some(n) = super::t2::blink(g, p, t)? {
                        g.perm_mut(n).tapped = true;
                    }
                } else {
                    exile_perm(g, t)?;
                    let spec = Tokens {
                        color: Some(Colors::from_letters("WU")),
                        types: vec!["detective"],
                        ..Tokens::new(1, 2)
                    };
                    make_tokens(g, ctl, spec)?;
                }
            }
        }
        EXIT => {
            if let Some((_, t)) = exit_target(g, p, Some(m))
                && ability_window(g, p, Some(m), "return another creature to its owner's hand", Some(3.0), Some(t))?
                && live(g, t)
            {
                bounce(g, t)?;
            }
        }
        _ => {}
    }
    Ok(true)
}

/// Turning a face-down creature up, on your own turn when the ability has a good target (Exalted Angel always: a 4/5
/// flying lifelinker). Called by `brain::hook_options` for every deck.
pub fn face_up_options(g: &mut Game, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut out = vec![];
    let Some(post) = post else { return Ok(out) };
    if g.active != Some(p) {
        return Ok(out);
    }
    for m in g.player(p).perms.clone() {
        if !is_face_down(g, m) {
            continue;
        }
        let name = card_name(g, m).unwrap().to_string();
        let (gn, pips) = up_cost(&name).unwrap();
        if !can_pay(g, p, gn, pips, false) {
            continue;
        }
        let v = up_value(g, p, m);
        if v < 2.0 {
            continue;
        }
        let before_combat = if post { 0.0 } else { 0.8 };
        let u = if name == EXALTED { 3.4 + before_combat } else { 1.2 + 0.4 * v.min(8.0) };
        out.push(plan(u, format!("{name} (turn face up)"), face_up_go, m.0 as i64));
    }
    Ok(out)
}

/// Casting a morph or disguise card face down for {3}: Exalted Angel when it can't be cast for {4}{W}{W}; a disguise
/// creature when turning it up has a good target (a normal cast is cheaper otherwise).
fn down_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !can_pay(g, p, 3, "", false) || !castable(g, p, c, "hand") {
        return Ok(vec![]);
    }
    let name = name_of(g, c).to_string();
    let (cg, cp) = cost_of(g, p, c);
    let normal = can_pay(g, p, cg, &cp, false);
    let (gn, pips) = up_cost(&name).unwrap();
    let u = if name == EXALTED {
        if normal {
            return Ok(vec![]); // cast it for its mana cost
        }
        // next turn it is turned up for {2}{W}{W}
        3.6 + if total_mana(g, p, false) >= 3 + gn + pips.len() as u32 { 1.0 } else { 0.0 }
    } else {
        let v = match name.as_str() {
            FAMILIAR => familiar_target(g, p, None).map_or(0.0, |x| x.0),
            GATEKEEPER => gatekeeper_target(g, p, None).map_or(0.0, |x| x.0),
            _ => exit_target(g, p, None).map_or(0.0, |x| x.0),
        };
        if v < 2.0 {
            return Ok(vec![]);
        }
        1.0 + 0.35 * v.min(8.0)
    };
    Ok(vec![plan(u, format!("{name} (face down)"), cast_down, c.0 as i64)])
}

// ================================================================== Ennis, Debate Moderator
/// the turn some card was put into exile (any player's): Ennis's counter. Called when a permanent goes to exile.
pub fn note_exile(g: &mut Game, m: PermId) {
    let x = g.perm(m);
    if x.token {
        return;
    }
    let o = x.orig;
    flag_now(g, o, "white_exiled_card");
}

fn exiled_this_turn(g: &Game) -> bool {
    g.players.iter().any(|q| flag_is_now(g, q.id, "white_exiled_card"))
}

/// exile m until the beginning of the next end step, when its card returns under its owner's control (a token is gone)
fn exile_until_end_step(g: &mut Game, m: PermId) -> Res {
    let (cd, orig, tok) = (g.perm(m).cd, g.perm(m).orig, g.perm(m).token);
    leave(g, m)?;
    if tok {
        return Ok(());
    }
    if let Some(cd) = cd {
        g.player_mut(orig).exile.push(cd);
        g.eot_returns.push((orig, cd));
        note_exile(g, m);
    }
    Ok(())
}

/// the creature p would exile with Ennis (or send home with Stormfront Riders): the best by `reenter_value`, if
/// it is worth anything
pub fn ennis_target(g: &Game, p: PlayerId, ennis: PermId) -> Option<PermId> {
    let cands: Vec<PermId> = creatures(g, p).into_iter().filter(|&m| m != ennis && !g.perm(m).token).collect();
    let m = max_by(&cands, |m| reenter_value(g, p, m))?;
    (reenter_value(g, p, m) >= 1.5).then_some(m)
}

/// Ennis enters: exile up to one other target creature you control, back at the next end step
fn ennis_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    let Some(t) = ennis_target(g, o, src) else { return Ok(()) };
    if !trigger_window(g, o, Some(src), "exile a creature until the end step", Some(3.0))? || !live(g, t) {
        return Ok(());
    }
    crate::glog!(g, "    Ennis exiles {} until the beginning of the next end step", g.perm(t).name);
    exile_until_end_step(g, t)
}

/// at the beginning of your end step, if one or more cards were put into exile this turn: a +1/+1 counter on Ennis
fn ennis_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == owner(g, src)
        && live(g, src)
        && exiled_this_turn(g)
        && trigger_window(g, p, Some(src), "a +1/+1 counter", Some(1.0))?
    {
        crate::cardcode::add_counters(g, src, 1);
    }
    Ok(())
}

// ================================================================== Stormfront Riders
/// p's Stormfront Riders that see a creature leave to hand (one per Riders, abilities on)
fn riders_of(g: &Game, p: PlayerId) -> u32 {
    g.player(p).perms.iter().filter(|&&e| live(g, e) && !g.perm(e).neutered && card_name(g, e) == Some(RIDERS)).count()
        as u32
}

/// before a permanent leaves for the hand: (its controller, their Riders) if it is a creature and there is one
pub fn bounce_note(g: &Game, m: PermId) -> Option<(PlayerId, u32)> {
    if !live(g, m) || !g.is_creature(m) {
        return None;
    }
    let o = owner(g, m);
    let n = riders_of(g, o);
    (n > 0).then_some((o, n))
}

/// did card or token m go to its owner's hand (a commander goes to the command zone instead, a token ceases to exist
/// but was returned)
fn went_to_hand(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    if x.token {
        return true;
    }
    let o = x.orig;
    !(x.cd == Some(g.player(o).cmd) && g.player(o).cmd_in_zone)
}

/// after a creature went to the hand: a 1/1 Soldier for each Stormfront Riders its controller had
pub fn bounced(g: &mut Game, m: PermId, note: Option<(PlayerId, u32)>) -> Res {
    let Some((o, n)) = note else { return Ok(()) };
    if went_to_hand(g, m) {
        soldiers(g, o, n)?;
    }
    Ok(())
}

fn soldiers(g: &mut Game, p: PlayerId, n: u32) -> Res {
    let spec = Tokens { color: Some(Colors::from_letters("W")), types: vec!["soldier"], ..Tokens::new(n, 1) };
    make_tokens(g, p, spec)?;
    Ok(())
}

/// Riders enters: return two creatures you control to their owner's hand (itself if need be), then a Soldier for
/// each creature that came back to the hand, for each Riders (they all return at once)
fn riders_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "return two creatures you control", Some(3.0))? {
        return Ok(());
    }
    let mine = creatures(g, o);
    let score = |x: PermId| -> f64 {
        if x == src {
            -50.0
        } else {
            reenter_value(g, o, x) - 0.4 * pval(g, x) - if g.perm(x).token { 1.0 } else { 0.0 }
        }
    };
    let mut ranked = mine.clone();
    ranked.sort_by(|&a, &b| score(b).partial_cmp(&score(a)).unwrap_or(std::cmp::Ordering::Equal));
    let riders = riders_of(g, o);
    let mut back = 0;
    for x in ranked.into_iter().take(2) {
        if !live(g, x) {
            continue;
        }
        crate::glog!(g, "    Stormfront Riders returns {} to its owner's hand", g.perm(x).name);
        leave(g, x)?;
        crate::engine::zones::to_zone_card(g, x, crate::engine::zones::Zone::Hand);
        if went_to_hand(g, x) {
            back += 1;
        }
    }
    soldiers(g, o, riders * back)
}

// ================================================================== Call a Surprise Witness, Recommission
/// what putting card c (from p's graveyard) onto the battlefield is worth: None if it is not a legal target
fn gy_value(g: &Game, p: PlayerId, c: CardId, artifacts: bool) -> Option<f64> {
    let d = g.db.get(c);
    if d.cmc > 3 || !(d.creature || (artifacts && d.types.has(Types::ARTIFACT))) {
        return None;
    }
    let mut v = 0.6 * d.bomb as f64 + crate::dsl::etb_value(g, c) as f64 + 0.3 * d.pow.max(0) as f64;
    v += if d.creature { 1.0 } else { 0.0 };
    if d.has_subtype("equipment") {
        v += 2.5; // the deck's engine
    }
    if g.registry.get(c).is_some_and(|i| i.etb.is_some()) {
        v += 3.0;
    }
    let _ = p;
    Some(v)
}

fn best_gy(g: &Game, p: PlayerId, artifacts: bool) -> Option<(f64, CardId)> {
    let cands: Vec<CardId> =
        g.player(p).gy.iter().copied().filter(|&c| gy_value(g, p, c, artifacts).is_some()).collect();
    let c = max_by(&cands, |c| gy_value(g, p, c, artifacts).unwrap_or(0.0))?;
    Some((gy_value(g, p, c, artifacts)?, c))
}

/// Call a Surprise Witness: return target creature card with mana value 3 or less from your graveyard to the
/// battlefield, with a flying counter (it flies), a Spirit in addition to its other types.
fn witness_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let t = ctx.rean_target.filter(|&t| g.player(p).gy.contains(&t) && gy_value(g, p, t, false).is_some());
    let Some(t) = t.or_else(|| best_gy(g, p, false).map(|x| x.1)) else { return Ok(intern("gy")) };
    super::partials::remove_card(&mut g.player_mut(p).gy, t);
    let n = enter(g, p, t, Enter::default())?;
    g.perm_mut(n).fly = true;
    g.perm_mut(n).ttypes.push(intern("spirit"));
    crate::glog!(g, "    {} returns {} with a flying counter", pname(g, p), g.perm(n).name);
    Ok(intern("gy"))
}

/// Recommission: return target artifact or creature card with mana value 3 or less from your graveyard to the
/// battlefield; a creature enters with an additional +1/+1 counter.
fn recommission_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let t = ctx.rean_target.filter(|&t| g.player(p).gy.contains(&t) && gy_value(g, p, t, true).is_some());
    let Some(t) = t.or_else(|| best_gy(g, p, true).map(|x| x.1)) else { return Ok(intern("gy")) };
    super::partials::remove_card(&mut g.player_mut(p).gy, t);
    let plus = if g.db.get(t).creature { 1 } else { 0 };
    let n = enter(g, p, t, Enter { plus, ..Enter::default() })?;
    crate::glog!(g, "    {} returns {}", pname(g, p), g.perm(n).name);
    Ok(intern("gy"))
}

fn rean_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId((arg & 0xffff) as u16);
    let t = CardId((arg >> 16) as u16);
    if !g.player(p).gy.contains(&t) {
        return Ok(false);
    }
    let v = if name_of(g, c) == RECOMMISSION { gy_value(g, p, t, true) } else { gy_value(g, p, t, false) };
    let Some(v) = v else { return Ok(false) };
    let ctx = Ctx { rean_target: Some(t), rean_value: v, rean_src: Some(p), ..Ctx::default() };
    crate::ai::brain::do_cast_with(g, p, c, None, ctx)
}

fn rean_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !castable(g, p, c, "hand") {
        return Ok(vec![]);
    }
    let (cg, cp) = cost_of(g, p, c);
    if !can_pay(g, p, cg, &cp, false) {
        return Ok(vec![]);
    }
    let Some((v, t)) = best_gy(g, p, name_of(g, c) == RECOMMISSION) else { return Ok(vec![]) };
    if v < 1.2 {
        return Ok(vec![]);
    }
    Ok(vec![plan(
        0.5 + 0.6 * v.min(9.0),
        format!("{} -> {}", name_of(g, c), name_of(g, t)),
        rean_go,
        c.0 as i64 | (t.0 as i64) << 16,
    )])
}

// ================================================================== Auras: Cho-Manno's Blessing, Shardmage's Rescue
/// the colours an Aura's controller chose (Cho-Manno's Blessing); none for any other Aura
pub fn chosen_color(g: &Game, a: PermId) -> Colors {
    match g.perm(a).data.get(DataKey::ChosenColor) {
        Some(Val::Str(s)) => Colors::from_letters(s),
        _ => Colors::NONE,
    }
}

/// the colour the AI would protect from with no removal spell to go by: the colour most of the opponents' permanents share
fn default_color(g: &Game, p: PlayerId) -> char {
    let mut best = ('W', 0usize);
    for ch in ['W', 'U', 'B', 'R', 'G'] {
        let n = g
            .opps(p)
            .map(|q| {
                g.player(q)
                    .perms
                    .iter()
                    .filter(|&&m| live(g, m) && crate::engine::values::colors_of(g, m).has(ch))
                    .count()
            })
            .sum::<usize>();
        if n > best.1 {
            best = (ch, n);
        }
    }
    best.0
}

/// As it enters, choose a colour: the one waiting on its host (a response to a spell of that colour), else the
/// opponents' most common
fn chomanno_etb(g: &mut Game, p: PlayerId, a: PermId, host: PermId) -> Res {
    let pending = g.perm_mut(host).data.remove(DataKey::ChosenColor);
    let ch = match pending {
        Some(Val::Str(s)) => s,
        _ => intern(&default_color(g, p).to_string()),
    };
    g.perm_mut(a).data.set(DataKey::ChosenColor, Val::Str(ch));
    crate::glog!(g, "    {} chooses {ch}: {} has protection from it", pname(g, p), g.perm(host).name);
    Ok(())
}

static CHO_MANNO_SPEC: AuraSpec = AuraSpec { on_etb: Some(chomanno_etb), ..AURA };

fn rescue_etb(g: &mut Game, _p: PlayerId, a: PermId, _host: PermId) -> Res {
    let st = g.turn_stamp();
    g.perm_mut(a).data.set(DataKey::Entered, Val::Stamp(st));
    Ok(())
}

static RESCUE_SPEC: AuraSpec = AuraSpec { pow: 1, tgh: 1, on_etb: Some(rescue_etb), ..AURA };

/// As long as Shardmage's Rescue entered this turn, enchanted creature has hexproof
fn rescue_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    let a = g.perm(src);
    kw == "hexproof" && a.attached == Some(m) && a.data.get(DataKey::Entered) == Some(&Val::Stamp(g.turn_stamp()))
}

/// cast the Aura c (flash) onto m, in response; true if it resolved and is on m
fn cast_aura_on(
    g: &mut Game,
    p: PlayerId,
    c: CardId,
    m: PermId,
    gn: u32,
    pips: &str,
    color: Option<char>,
) -> Res<bool> {
    if !castable(g, p, c, "hand") || !can_pay(g, p, gn, pips, false) || !pay(g, p, gn, pips, false)? {
        return Ok(false);
    }
    if let Some(ch) = color {
        g.perm_mut(m).data.set(DataKey::ChosenColor, Val::Str(intern(&ch.to_string())));
    }
    g.attach_to = Some(m);
    let r = cast_card(g, p, c, "hand", Ctx { target: Some(m), ..Ctx::default() });
    g.attach_to = None;
    g.perm_mut(m).data.remove(DataKey::ChosenColor);
    Ok(r? && live(g, m))
}

// ------------------------------------------------------------------ protection from removal and wipes
fn destroyish(kind: &str) -> bool {
    kind == "destroy" || kind.starts_with("dmg")
}

/// the first colour (W U B R G) of a spell's mana cost
fn spell_color(g: &Game, spell: Option<CardId>) -> Option<char> {
    let s = spell?;
    ['W', 'U', 'B', 'R', 'G'].into_iter().find(|&ch| g.db.get(s).pips.contains(ch))
}

/// **Answers to targeted removal** aimed at the cloud deck's permanent m: Patriot (hexproof until end of turn, no
/// card spent), Shardmage's Rescue (hexproof as it enters, +1/+1), Cho-Manno's Blessing (protection from the
/// spell's colour), Ultimate Magic: Holy (indestructible, for destroy and damage). True when the removal is stopped.
pub fn white_protect(
    g: &mut Game,
    owner_p: PlayerId,
    m: PermId,
    kind: Sym,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    if actor == Some(owner_p) || !live(g, m) || (!g.perm(m).is_cmd && pval(g, m) < 3.0) {
        return Ok(false);
    }
    let creature = g.is_creature(m);
    // Patriot: {2},{T}: another target creature you control gets +2/+0 and hexproof until end of turn
    if creature
        && let Some(pat) = g.player(owner_p).perms.iter().copied().find(|&x| {
            let y = g.perm(x);
            x != m && live(g, x) && !y.neutered && !y.tapped && !y.sick && card_name(g, x) == Some(PATRIOT)
        })
        && can_pay(g, owner_p, 2, "", false)
        && patriot_go(g, pat, owner_p, m.0 as i64)?
    {
        return Ok(true);
    }
    if creature
        && let Some(c) = in_hand(g, owner_p, RESCUE)
        && cast_aura_on(g, owner_p, c, m, 0, "W", None)?
    {
        return Ok(true);
    }
    if creature
        && let Some(ch) = spell_color(g, spell)
        && let Some(c) = in_hand(g, owner_p, CHO_MANNO)
        && cast_aura_on(g, owner_p, c, m, 0, "WW", Some(ch))?
    {
        return Ok(true);
    }
    if destroyish(&kind)
        && let Some(c) = in_hand(g, owner_p, HOLY)
        && cast_holy(g, owner_p, c)?
    {
        return Ok(crate::engine::values::indestructible(g, m));
    }
    Ok(false)
}

/// the cloud deck's answer to a board wipe that destroys: Ultimate Magic: Holy, when the wipe takes a lot
pub fn white_wipe_response(g: &mut Game, q: PlayerId, kind: Sym, caster: PlayerId) -> Res<Option<Sym>> {
    const WIPE_INDES: [&str; 6] = ["destroy", "dmg13", "austere", "austere2", "nib", "vandal"];
    if !WIPE_INDES.contains(&&*kind) {
        return Ok(None);
    }
    let Some(c) = in_hand(g, q, HOLY) else { return Ok(None) };
    if crate::ai::decks::wipe_loss(g, q, &kind, caster) < 6.0 {
        return Ok(None);
    }
    Ok(cast_holy(g, q, c)?.then(|| intern("indes")))
}

// ================================================================== Ultimate Magic: Holy
/// the turn a foretold card was set aside: `impulse_long` marks, `u32::MAX - 1 - turn` (as green_cards does)
fn foretold_turn(g: &Game, p: PlayerId, c: CardId) -> Option<u32> {
    g.player(p)
        .impulse_long
        .iter()
        .find(|x| x.0 == c && x.1 < u32::MAX && x.1 > u32::MAX / 2)
        .map(|x| u32::MAX - 1 - x.1)
}

/// foretold on an earlier turn (or it is not our turn: cast from exile at instant speed on a later turn)
fn foretold_ready(g: &Game, p: PlayerId, c: CardId) -> bool {
    foretold_turn(g, p, c).is_some_and(|t| g.player(p).turns > t || g.active != Some(p))
}

/// cast Holy for {2}{W} (foretold or not): the rider only when it was foretold
pub fn cast_holy(g: &mut Game, p: PlayerId, c: CardId) -> Res<bool> {
    let from_exile = foretold_ready(g, p, c);
    if foretold_turn(g, p, c).is_some() && !from_exile {
        return Ok(false); // foretold this turn: not yet
    }
    if !castable(g, p, c, "hand") || !can_pay(g, p, 2, "W", false) || !pay(g, p, 2, "W", false)? {
        return Ok(false);
    }
    g.player_mut(p).impulse_long.retain(|x| x.0 != c);
    if from_exile {
        flag_now(g, p, "white_foretold_cast");
    }
    let r = cast_card(g, p, c, "hand", Ctx::default());
    g.player_mut(p).flag_turn.shift_remove(&intern("white_foretold_cast"));
    r
}

/// Permanents you control gain indestructible until end of turn. If this spell was cast from exile, prevent all damage
/// that would be dealt to you this turn.
fn holy_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    for m in g.player(p).perms.clone() {
        let x = g.perm_mut(m);
        if !x.eot_kw.contains(&"indestructible") {
            x.eot_kw.push(intern("indestructible"));
        }
    }
    if flag_is_now(g, p, "white_foretold_cast") {
        flag_now(g, p, "white_damage_shield");
        crate::glog!(g, "    all damage to {} is prevented this turn", pname(g, p));
    }
    Ok(intern("gy"))
}

/// all damage to p is prevented this turn (Holy cast from exile); `engine::life::prevents_damage`
pub fn damage_shield(g: &Game, p: PlayerId) -> bool {
    flag_is_now(g, p, "white_damage_shield")
}

/// foretell {2} (your turn, after combat, mana to spare), cast later from exile for {2}{W}
fn holy_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(true) || g.active != Some(p) || foretold_turn(g, p, c).is_some() || !can_pay(g, p, 2, "", false) {
        return Ok(vec![]);
    }
    if g.player(p).hand.iter().filter(|&&x| name_of(g, x) == HOLY).count() > 1 {
        return Ok(vec![]);
    }
    Ok(vec![plan(0.7, "Ultimate Magic: Holy (foretell)".into(), holy_foretell, c.0 as i64)])
}

fn holy_foretell(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || foretold_turn(g, p, c).is_some() || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    if !pay(g, p, 2, "", false)? {
        return Ok(false);
    }
    let pl = g.player_mut(p);
    let mark = u32::MAX - 1 - pl.turns;
    pl.impulse_long.push((c, mark));
    crate::glog!(g, "  {} foretells Ultimate Magic: Holy", pname(g, p));
    Ok(true)
}

// ================================================================== Patriot, Shield Wielder
/// {2},{T}: another target creature you control gets +2/+0 and gains hexproof until end of turn (arg: the creature)
fn patriot_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    let s = g.perm(src);
    if !live(g, src) || s.tapped || s.sick || s.neutered || !live(g, t) || t == src || owner(g, t) != p {
        return Ok(false);
    }
    if !can_pay(g, p, 2, "", false) || !pay(g, p, 2, "", false)? {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if ability_window(g, p, Some(src), "+2/+0 and hexproof", Some(2.0), Some(t))? && live(g, t) {
        let x = g.perm_mut(t);
        x.eot_pt.0 += 2;
        if !x.eot_kw.contains(&"hexproof") {
            x.eot_kw.push(intern("hexproof"));
        }
        crate::glog!(g, "    {} gets +2/+0 and hexproof until end of turn", g.perm(t).name);
    }
    Ok(true)
}

fn patriot_ability(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    patriot_go(g, src, p, arg)
}

/// before combat: pump the creature that will attack (it is not summoning sick, power 3 or more)
fn patriot_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if owner(g, src) != p
        || !live(g, src)
        || x.tapped
        || x.sick
        || x.neutered
        || post != Some(false)
        || g.active != Some(p)
    {
        return Ok(vec![]);
    }
    if !can_pay(g, p, 2, "", false) {
        return Ok(vec![]);
    }
    let cands: Vec<PermId> = creatures(g, p)
        .into_iter()
        .filter(|&m| m != src && !g.perm(m).sick && !g.perm(m).tapped && !g.perm(m).noatk && epow(g, m) >= 3)
        .collect();
    let Some(t) = max_by(&cands, |m| epow(g, m) as f64 + pval(g, m)) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: 1.2 + 0.15 * epow(g, t) as f64,
        label: format!("Patriot: {} gets +2/+0 and hexproof", g.perm(t).name),
        act: Some(Action::Ability { src, f: patriot_ability, arg: t.0 as i64 }),
    }])
}

// ================================================================== Recruitment Officer
/// {3}{W}: look at the top four cards of your library. You may reveal a creature card with mana value 3 or less from
/// among them and put it into your hand. Put the rest on the bottom of your library in a random order.
fn officer_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || g.perm(src).neutered || !can_pay(g, p, 3, "W", false) || g.player(p).library.is_empty() {
        return Ok(false);
    }
    if !pay(g, p, 3, "W", false)? {
        return Ok(false);
    }
    if !ability_window(g, p, Some(src), "look at the top four cards", Some(2.0), None)? {
        return Ok(true);
    }
    // the top of the library is the end of the list
    let n = g.player(p).library.len().min(4);
    let at = g.player(p).library.len() - n;
    let mut seen: Vec<CardId> = g.player_mut(p).library.split_off(at);
    let pick = max_by(
        &seen.iter().copied().filter(|&c| g.db.get(c).creature && g.db.get(c).cmc <= 3).collect::<Vec<_>>(),
        |c| crate::engine::tutors::card_worth(g, p, c, false) + crate::dsl::etb_value(g, c) as f64,
    );
    if let Some(c) = pick {
        super::partials::remove_card(&mut seen, c);
        g.player_mut(p).hand.push(c);
        g.player_mut(p).seen.insert(c);
        crate::glog!(g, "    {} puts {} into their hand", pname(g, p), name_of(g, c));
    }
    g.rng.shuffle(&mut seen);
    let lib = std::mem::take(&mut g.player_mut(p).library);
    g.player_mut(p).library = seen.into_iter().chain(lib).collect();
    Ok(true)
}

fn officer_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if owner(g, src) != p
        || !live(g, src)
        || x.neutered
        || !can_pay(g, p, 3, "W", false)
        || g.player(p).library.len() < 4
    {
        return Ok(vec![]);
    }
    // spare mana: after combat on your turn, or at the end of the turn before yours
    let u = match post {
        None => 1.9,
        Some(true) if g.active == Some(p) => 1.3,
        _ => return Ok(vec![]),
    };
    let u = u - if g.player(p).hand.len() >= 6 { 1.2 } else { 0.0 };
    Ok(vec![Opt {
        utility: u,
        label: "Recruitment Officer: look at the top four".into(),
        act: Some(Action::Ability { src, f: officer_go, arg: 0 }),
    }])
}

// ================================================================== Fate of the Sun-Cryst, Harsh Annotation, Lay Down Arms
/// destroy target nonland permanent (`ctx.target`)
fn fate_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = ctx.target.filter(|&t| live(g, t)) {
        apply_removal(g, Some(p), t, "destroy", Some(c))?;
    }
    Ok(intern("gy"))
}

/// costs {2} less to cast if it targets a tapped creature: the cheap cast, offered when a tapped creature is worth
/// destroying (the full-cost cast goes through the generic removal options)
fn fate_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let _ = post;
    if !can_pay(g, p, 2, "W", false) || !castable(g, p, c, "hand") {
        return Ok(vec![]);
    }
    let tg: Vec<PermId> = legal_targets(g, p, "destroy", "nl", false, Some(c))
        .into_iter()
        .filter(|&m| g.is_creature(m) && g.perm(m).tapped)
        .collect();
    let Some(best) = max_by(&tg, |m| pval(g, m)) else { return Ok(vec![]) };
    let v = pval(g, best);
    if v < 2.5 {
        return Ok(vec![]);
    }
    let main = g.active == Some(p);
    let u = v - 2.7 - if main { 0.5 } else { 0.0 };
    Ok(vec![plan(u, format!("{FATE} -> {} (tapped)", g.perm(best).name), fate_go, c.0 as i64 | (best.0 as i64) << 16)])
}

fn fate_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId((arg & 0xffff) as u16);
    let t = PermId((arg >> 16) as u32);
    if !g.player(p).hand.contains(&c) || !live(g, t) || !g.perm(t).tapped || !g.is_creature(t) || untargetable(g, t) {
        return Ok(false);
    }
    if !can_pay(g, p, 2, "W", false) || !pay(g, p, 2, "W", false)? {
        return Ok(false);
    }
    cast_card(g, p, c, "hand", Ctx { target: Some(t), ..Ctx::default() })
}

/// Destroy target creature. Its controller creates a 1/1 white and black Inkling creature token with flying.
fn annotation_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target.filter(|&t| live(g, t) && g.is_creature(t) && !untargetable(g, t)) else {
        return Ok(intern("gy"));
    };
    let ctl = owner(g, t);
    apply_removal(g, Some(p), t, "destroy", Some(c))?;
    let spec =
        Tokens { fly: true, color: Some(Colors::from_letters("WB")), types: vec!["inkling"], ..Tokens::new(1, 1) };
    make_tokens(g, ctl, spec)?;
    Ok(intern("gy"))
}

/// Exile target creature with mana value less than or equal to the number of Plains you control. Its controller gains
/// 3 life. (The mana value limit is in `legal_targets`.)
fn lay_down_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target.filter(|&t| live(g, t) && g.is_creature(t)) else { return Ok(intern("gy")) };
    let mv = g.perm(t).cd.map_or(0, |cd| g.db.get(cd).cmc);
    if mv > plains(g, p) {
        return Ok(intern("gy"));
    }
    let ctl = owner(g, t);
    apply_removal(g, Some(p), t, "exile", Some(c))?;
    if !live(g, t) {
        gain(g, ctl, 3)?;
    }
    Ok(intern("gy"))
}

// ================================================================== Repel the Darkness
/// the creatures Repel the Darkness taps: the opponents' most dangerous untapped ones (at most two)
fn repel_targets(g: &Game, p: PlayerId) -> Vec<PermId> {
    let mut cands: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied().collect::<Vec<_>>())
        .filter(|&m| live(g, m) && g.is_creature(m) && !g.perm(m).tapped && !untargetable(g, m))
        .collect();
    cands.sort_by(|&a, &b| epow(g, b).cmp(&epow(g, a)));
    cands.truncate(2);
    cands
}

/// Tap up to two target creatures. Draw a card.
fn repel_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let tg: Vec<PermId> = match ctx.target {
        Some(t) if live(g, t) => {
            let mut v = vec![t];
            v.extend(repel_targets(g, p).into_iter().filter(|&m| m != t).take(1));
            v
        }
        _ => repel_targets(g, p),
    };
    for m in tg {
        if live(g, m) && !untargetable(g, m) {
            g.perm_mut(m).tapped = true;
            crate::glog!(g, "    {} is tapped", g.perm(m).name);
        }
    }
    draw(g, p, 1, false)?;
    Ok(intern("gy"))
}

fn repel_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let (cg, cp) = cost_of(g, p, c);
    if !can_pay(g, p, cg, &cp, false) || !castable(g, p, c, "hand") {
        return Ok(vec![]);
    }
    let blockers = repel_targets(g, p);
    let attackers =
        creatures(g, p).into_iter().filter(|&m| !g.perm(m).sick && !g.perm(m).tapped && epow(g, m) >= 2).count();
    let u = match post {
        // before combat: clear the way
        Some(false) if g.active == Some(p) && attackers > 0 && !blockers.is_empty() => {
            1.4 + 0.5 * blockers.len() as f64 + 0.2 * attackers.min(3) as f64
        }
        // the end of the turn before yours: a card
        None => 0.9,
        _ => return Ok(vec![]),
    };
    Ok(vec![plan(u, "Repel the Darkness".into(), repel_go, c.0 as i64)])
}

fn repel_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    crate::ai::brain::do_cast_with(g, p, CardId(arg as u16), None, Ctx::default())
}

// ================================================================== Bronze Guardian
/// Bronze Guardian's power is equal to the number of artifacts you control
fn guardian_pt(g: &Game, p: PlayerId, _m: PermId) -> (i32, i32) {
    (super::sheoldred::artifact_count(g, p), 0)
}

fn selfpt_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.selfpt = true;
    }
    Ok(())
}

fn selfpt_as(g: &mut Game, _p: PlayerId, _m: PermId) -> Res {
    g.selfpt = true;
    Ok(())
}

// ================================================================== the catch-up ramp
// Keeper of the Accord, Loyal Warhound, Gift of Estates, Archaeomancer's Map, Oreskos Explorer. "More lands" counts
// the lands on the battlefield (`lands.len()`); each condition is checked when the ability triggers and again when it
// resolves (an intervening "if"), and Oreskos Explorer counts at resolution.
const KEEPER: &str = "Keeper of the Accord";
const WARHOUND: &str = "Loyal Warhound";
const GIFT: &str = "Gift of Estates";
const MAP: &str = "Archaeomancer's Map";
const EXPLORER: &str = "Oreskos Explorer";

/// the opponents of p that control more lands than p
fn land_leaders(g: &Game, p: PlayerId) -> u32 {
    let n = g.player(p).lands.len();
    g.opps(p).filter(|&q| g.player(q).lands.len() > n).count() as u32
}

fn behind_on_lands(g: &Game, p: PlayerId) -> bool {
    land_leaders(g, p) > 0
}

/// Search p's library for up to `n` Plains cards (`basic`: only basic Plains), then shuffle. They go to the hand
/// (revealed) or onto the battlefield, tapped. Returns how many were found.
fn fetch_plains(g: &mut Game, p: PlayerId, n: u32, basic: bool, to_hand: bool) -> Res<u32> {
    let mut found = 0;
    for _ in 0..n {
        let Some(c) = searchable(g, p).into_iter().find(|&c| {
            let d = g.db.get(c);
            d.land && d.has_subtype("plains") && (!basic || d.basic)
        }) else {
            break;
        };
        let lib = &mut g.player_mut(p).library;
        let i = lib.iter().position(|&x| x == c).unwrap();
        lib.remove(i);
        found += 1;
        if to_hand {
            g.player_mut(p).hand.push(c);
            g.player_mut(p).seen.insert(c);
        } else {
            g.add_land_entering(p, c, true);
            landfall(g, p)?;
        }
    }
    shuffle_library(g, p);
    Ok(found)
}

/// Loyal Warhound: when it enters, if an opponent controls more lands than you, search for a basic Plains, put it
/// onto the battlefield tapped
fn warhound_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if behind_on_lands(g, o)
        && trigger_window(g, o, Some(src), "search for a basic Plains", Some(4.0))?
        && behind_on_lands(g, o)
    {
        fetch_plains(g, o, 1, true, false)?;
    }
    Ok(())
}

/// Oreskos Explorer: when it enters, search for up to X Plains cards (X = the players who control more lands than
/// you, counted as it resolves), into your hand
fn explorer_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "search for Plains", Some(4.0))? {
        return Ok(());
    }
    let x = land_leaders(g, o);
    fetch_plains(g, o, x, false, true)?;
    Ok(())
}

/// Gift of Estates: if an opponent controls more lands than you, search for up to three Plains cards into your hand
fn gift_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    if behind_on_lands(g, p) {
        fetch_plains(g, p, 3, false, true)?;
    }
    Ok(intern("gy"))
}

/// Keeper of the Accord: at each opponent's end step, if they control more creatures than you, a 1/1 white Soldier;
/// then if they control more lands than you, you may search for a basic Plains onto the battlefield tapped
fn keeper_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = owner(g, src);
    if o == p || !live(g, src) || !g.player(p).alive {
        return Ok(());
    }
    let fewer = |g: &Game| creatures(g, p).len() > creatures(g, o).len();
    if fewer(g) && trigger_window(g, o, Some(src), "create a Soldier", Some(4.0))? && fewer(g) {
        let spec = Tokens { color: Some(Colors::from_letters("W")), types: vec!["soldier"], ..Tokens::new(1, 1) };
        make_tokens(g, o, spec)?;
    }
    let smaller = |g: &Game| g.player(p).lands.len() > g.player(o).lands.len();
    if smaller(g) && trigger_window(g, o, Some(src), "search for a basic Plains", Some(4.0))? && smaller(g) {
        fetch_plains(g, o, 1, true, false)?;
    }
    Ok(())
}

/// Archaeomancer's Map: when it enters, search for up to two basic Plains cards into your hand
fn map_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if trigger_window(g, o, Some(src), "search for two basic Plains", Some(4.0))? {
        fetch_plains(g, o, 2, true, true)?;
    }
    Ok(())
}

/// Archaeomancer's Map: whenever a land an opponent controls enters, if that player controls more lands than you, you
/// may put a land card from your hand onto the battlefield (a Plains, else one that doesn't enter tapped)
fn map_landfall(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = owner(g, src);
    if o == p || !live(g, src) || !g.player(p).alive {
        return Ok(());
    }
    let ahead = |g: &Game| g.player(p).lands.len() > g.player(o).lands.len();
    let pick = |g: &Game| {
        g.player(o)
            .hand
            .iter()
            .copied()
            .filter(|&c| g.db.get(c).land)
            .max_by_key(|&c| (g.db.get(c).has_subtype("plains") as i32 * 2) + !g.db.get(c).tag(Tag::T) as i32)
    };
    if !ahead(g) || pick(g).is_none() {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "put a land onto the battlefield", Some(4.0))? || !ahead(g) {
        return Ok(());
    }
    if let Some(c) = pick(g) {
        let hand = &mut g.player_mut(o).hand;
        let i = hand.iter().position(|&x| x == c).unwrap();
        hand.remove(i);
        g.add_land_entering(o, c, false);
        landfall(g, o)?;
    }
    Ok(())
}

// ================================================================== registry
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    // face down
    for name in [EXALTED, FAMILIAR, GATEKEEPER, EXIT] {
        r.card(db, name)?.hand_options = Some(down_options);
    }
    // enter again
    r.card(db, ENNIS)?.etb = Some(ennis_etb);
    r.card(db, ENNIS)?.end_step = Some(ennis_end);
    r.card(db, RIDERS)?.etb = Some(riders_etb);
    // recursion
    let c = r.card(db, WITNESS)?;
    c.resolve = Some(witness_resolve);
    c.hand_options = Some(rean_options);
    c.prio = Some(zero_prio);
    let c = r.card(db, RECOMMISSION)?;
    c.resolve = Some(recommission_resolve);
    c.hand_options = Some(rean_options);
    c.prio = Some(zero_prio);
    // Auras
    aura(r, db, CHO_MANNO, &CHO_MANNO_SPEC)?;
    r.card(db, CHO_MANNO)?.prio = Some(zero_prio);
    aura(r, db, RESCUE, &RESCUE_SPEC)?;
    let c = r.card(db, RESCUE)?;
    c.prio = Some(zero_prio);
    c.grant_kw = Some(rescue_kw);
    // Holy, Patriot, the Officer
    let c = r.card(db, HOLY)?;
    c.resolve = Some(holy_resolve);
    c.hand_options = Some(holy_options);
    c.prio = Some(zero_prio);
    r.card(db, PATRIOT)?.options = Some(patriot_options);
    r.card(db, OFFICER)?.options = Some(officer_options);
    // removal and tempo
    let c = r.card(db, FATE)?;
    c.resolve = Some(fate_resolve);
    c.hand_options = Some(fate_options);
    r.card(db, ANNOTATION)?.resolve = Some(annotation_resolve);
    r.card(db, LAY_DOWN)?.resolve = Some(lay_down_resolve);
    let c = r.card(db, REPEL)?;
    c.resolve = Some(repel_resolve);
    c.hand_options = Some(repel_options);
    c.prio = Some(zero_prio);
    // Bronze Guardian
    let c = r.card(db, GUARDIAN)?;
    c.etb = Some(selfpt_etb);
    c.as_enters = Some(selfpt_as);
    c.self_pt = Some(guardian_pt);
    // the catch-up ramp
    r.card(db, WARHOUND)?.etb = Some(warhound_etb);
    r.card(db, EXPLORER)?.etb = Some(explorer_etb);
    r.card(db, GIFT)?.resolve = Some(gift_resolve);
    r.card(db, KEEPER)?.end_step = Some(keeper_end);
    let c = r.card(db, MAP)?;
    c.etb = Some(map_etb);
    c.landfall = Some(map_landfall);
    Ok(())
}
