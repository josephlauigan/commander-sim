//! Tier 6, batch `green1`: the creature-engine cards of the Rograkh / Thrasios deck (Oboro Breezecaller, Badgermole
//! Cub, Biomancer's Familiar, Earthcraft, Training Grounds, Candelabra of Tawnos, Magus of the Candelabra,
//! Formidable Speaker, Springheart Nantuko, Vibrance). Cards only: no shared engine code is changed here (what the
//! engine still has to do is in rust/agents/engine-needs-green1.md).
//!
//! Most of them untap lands for the deck's mana engine (Gaea's Cradle taps for one {G} per creature), so the AI uses
//! an untap effect only when the land it untaps makes clearly more mana than the effect costs.

use super::partials::{pack, unpack};
use crate::engine::pool::Kind;
use crate::cards::CardDb;
use crate::engine::hooks as eh;
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::turn::remove_land;
use crate::engine::zones::min_by;
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;
use crate::tag::Tag;

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased && !g.perm(m).neutered
}

fn name_of(g: &Game, m: PermId) -> &str {
    g.perm(m).cd.map_or("", |c| &g.db.get(c).name)
}

fn ability(utility: f64, label: &str, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label: label.into(), act: Some(Action::Ability { src, f, arg }) }
}

/// the mana land l makes when tapped now (Gaea's Cradle counts creatures; Talon Gates of Madara makes {C})
fn land_yield(g: &Game, p: PlayerId, l: PermId) -> u32 {
    let base = crate::cardcode::dyn_mana_land(g, p, l)
        .map_or_else(|| g.db.get(g.perm(l).card()).tags.int(Tag::Amt).unwrap_or(1).max(0) as u32, |n| n);
    (base as i32 + eh::total_land_mana(g, p, l)).max(0) as u32
}

fn land_on(g: &Game, p: PlayerId, l: PermId) -> bool {
    g.perm(l).on_bf && g.player(p).lands.contains(&l)
}

/// {2} less per Training Grounds / Biomancer's Familiar p controls (each: "activated abilities of creatures you
/// control cost {2} less to activate; can't reduce the mana in that cost to less than one mana")
pub fn ability_discount(g: &Game, p: PlayerId) -> u32 {
    let n = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| live(g, m) && matches!(name_of(g, m), "Training Grounds" | "Biomancer's Familiar"))
        .count();
    2 * n as u32
}

/// what a creature's activated ability of printed generic cost `cost` costs p now (never below {1}, unless it was
/// free); the discount applies to creature abilities only
pub fn creature_ability_cost(g: &Game, p: PlayerId, cost: u32) -> u32 {
    if cost == 0 { 0 } else { cost.saturating_sub(ability_discount(g, p)).max(1) }
}

// ================================================================== Oboro Breezecaller
/// the best tapped land to untap and the land to return for it (cheapest to lose: a tapped one costs nothing, an
/// untapped one costs the mana it would make); None when the untap does not pay for its cost
fn oboro_plan(g: &Game, p: PlayerId, cost: u32) -> Option<(PermId, PermId)> {
    let lands = g.player(p).lands.clone();
    let tapped: Vec<PermId> = lands.iter().copied().filter(|&l| g.perm(l).tapped).collect();
    let t = crate::engine::zones::max_by(&tapped, |l| land_yield(g, p, l) as f64)?;
    let others: Vec<PermId> = lands.iter().copied().filter(|&l| l != t).collect();
    let b = min_by(&others, |l| if g.perm(l).tapped { 0.0 } else { land_yield(g, p, l) as f64 })?;
    let lost = if g.perm(b).tapped { 0 } else { land_yield(g, p, b) };
    // the gain: the untapped land's mana, less the ability's cost and the mana the returned land would have made
    (land_yield(g, p, t) as i32 - cost as i32 - lost as i32 >= 2).then_some((t, b))
}

/// whether p has anything to spend extra mana on: a spell in hand or Thrasios's mana sink on the battlefield
fn has_sink(g: &Game, p: PlayerId) -> bool {
    g.player(p).hand.iter().any(|&c| !g.db.get(c).land)
        || g.player(p).perms.iter().any(|&m| live(g, m) && name_of(g, m).starts_with("Thrasios"))
}

/// **Oboro Breezecaller** ({1}{U}, 1/1 flying): {2}, return a land you control to its owner's hand: untap target
/// land. The cost is reduced by Training Grounds / Biomancer's Familiar (`creature_ability_cost`). The AI uses it on
/// its tapped land that makes the most mana (Gaea's Cradle) when that pays back the cost, the returned land and at
/// least {2} more, and something is waiting for the mana; it returns a tapped land if it has one. Not offered while
/// the Breezecaller is phased out, neutered or locked.
fn oboro_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !live(g, src) || owner(g, src) != p || !has_sink(g, p) {
        return Ok(vec![]);
    }
    let cost = creature_ability_cost(g, p, 2);
    if !can_pay(g, p, cost, "", false) {
        return Ok(vec![]);
    }
    let Some((t, b)) = oboro_plan(g, p, cost) else { return Ok(vec![]) };
    Ok(vec![ability(2.5, "untap a land (Oboro Breezecaller)", src, oboro_go, pack(t.0, b.0))])
}

fn oboro_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (t, b) = unpack(arg);
    let (t, b) = (PermId(t), PermId(b));
    let cost = creature_ability_cost(g, p, 2);
    if !live(g, src) || owner(g, src) != p || !land_on(g, p, t) || !land_on(g, p, b) || t == b {
        return Ok(false);
    }
    if !can_pay(g, p, cost, "", false) {
        return Ok(false);
    }
    pay(g, p, cost, "", false)?;
    // the cost: return a land you control to its owner's hand
    if !land_on(g, p, b) {
        return Ok(true);
    }
    let cd = g.perm(b).card();
    remove_land(g, p, b);
    g.player_mut(p).hand.push(cd);
    if !ability_window(g, p, Some(src), "untap target land", None, None)? {
        return Ok(true);
    }
    if land_on(g, p, t) {
        g.perm_mut(t).tapped = false;
        crate::glog!(
            g,
            "    {} returns {} and untaps {} (Oboro Breezecaller)",
            g.player(p).name,
            g.db.get(cd).name,
            g.db.get(g.perm(t).card()).name
        );
    }
    Ok(true)
}

// ================================================================== Badgermole Cub
/// **Badgermole Cub** ({1}{G}, 2/2): "whenever you tap a creature for mana, add an additional {G}" (a floating {G}
/// each time one of your creatures is tapped for mana, for each Cub; Gaea's Cradle is a land, not a creature, so it
/// does not count). The planner does not count the extra {G} when it plans a payment; it floats afterwards. Left out:
/// "when this creature enters, earthbend 1" (a land becomes a 0/0 creature with haste and a +1/+1 counter, and
/// returns tapped when it dies or is exiled), which needs lands that are creatures (engine-needs-green1.md).
fn cub_tapped(g: &mut Game, src: Src, p: PlayerId, m: PermId, _n: u32) -> Res {
    if live(g, src) && owner(g, src) == p && g.is_creature(m) {
        g.player_mut(p).mana.add(Kind::G, 1);
        crate::glog!(g, "    Badgermole Cub adds an additional {{G}}");
    }
    Ok(())
}

// ================================================================== Earthcraft
fn is_basic_land(g: &Game, l: PermId) -> bool {
    crate::engine::mana::BASIC_NAMES.contains(&&*g.db.get(g.perm(l).card()).name)
}

/// the creature the AI taps for Earthcraft: one that has nothing better to do (summoning sick, or can't attack, or
/// any untapped one after combat), never a mana creature (tapping it for mana is as good), the cheapest to idle
fn earthcraft_creature(g: &Game, p: PlayerId, src: PermId, post: bool) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            let d = x.cd.map(|c| g.db.get(c));
            live(g, m)
                && g.is_creature(m)
                && m != src
                && !x.tapped
                && (post || x.sick || x.noatk)
                && !d.is_some_and(|d| d.tags.str(Tag::Dork).is_some() || d.tags.str(Tag::Rock).is_some())
        })
        .collect();
    min_by(&cs, |m| crate::engine::values::pval(g, m))
}

/// **Earthcraft** ({1}{G} enchantment): tap an untapped creature you control: untap target basic land. The AI uses it
/// with a tapped basic land, a spell (or Thrasios) waiting for the mana, and a creature with nothing better to do
/// (see `earthcraft_creature`: before combat only a summoning-sick one).
fn earthcraft_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let Some(post) = post else { return Ok(vec![]) };
    if !live(g, src) || owner(g, src) != p || !has_sink(g, p) {
        return Ok(vec![]);
    }
    let Some(l) = g.player(p).lands.iter().copied().find(|&l| g.perm(l).tapped && is_basic_land(g, l)) else {
        return Ok(vec![]);
    };
    let Some(c) = earthcraft_creature(g, p, src, post) else { return Ok(vec![]) };
    Ok(vec![ability(1.5, "untap a basic land (Earthcraft)", src, earthcraft_go, pack(c.0, l.0))])
}

fn earthcraft_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, l) = unpack(arg);
    let (c, l) = (PermId(c), PermId(l));
    if !live(g, src) || owner(g, src) != p || !live(g, c) || g.perm(c).tapped || !land_on(g, p, l) {
        return Ok(false);
    }
    g.perm_mut(c).tapped = true; // the cost
    if !ability_window(g, p, Some(src), "untap target basic land", None, None)? {
        return Ok(true);
    }
    if land_on(g, p, l) && is_basic_land(g, l) {
        g.perm_mut(l).tapped = false;
        crate::glog!(g, "    Earthcraft: {} taps to untap {}", g.perm(c).name, g.db.get(g.perm(l).card()).name);
    }
    Ok(true)
}

// ================================================================== Candelabra / Magus of the Candelabra
/// whether permanent m's {T} ability can be used now (untapped; a creature must not be summoning sick)
fn can_tap(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    live(g, m) && !x.tapped && (!x.sick || !g.is_creature(m) || crate::dsl::has_kw(g, m, "haste") || crate::impls::t6_legends1b::ability_haste(g, x.owner))
}

/// the tapped lands to untap for "{X},{T}: untap X target lands": the X best ones, and X, chosen to maximise the mana
/// they make less the ability's cost; None unless that nets at least {2} and p can pay it. `creature`: the cost is
/// reduced by Training Grounds / Biomancer's Familiar (Magus), `false` for the artifact.
fn candelabra_plan(g: &Game, p: PlayerId, creature: bool) -> Option<Vec<PermId>> {
    let mut tapped: Vec<(u32, PermId)> =
        g.player(p).lands.iter().copied().filter(|&l| g.perm(l).tapped).map(|l| (land_yield(g, p, l), l)).collect();
    tapped.sort_by_key(|x| std::cmp::Reverse(x.0));
    let (mut best, mut sum) = (None, 0i64);
    for (i, &(y, _)) in tapped.iter().enumerate() {
        sum += y as i64;
        let x = i as u32 + 1;
        let cost = if creature { creature_ability_cost(g, p, x) } else { x };
        let net = sum - cost as i64;
        if net >= 2 && can_pay(g, p, cost, "", false) && best.is_none_or(|(_, n)| net > n) {
            best = Some((x, net));
        }
    }
    best.map(|(x, _)| tapped.iter().take(x as usize).map(|t| t.1).collect())
}

fn candelabra_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || owner(g, src) != p || !can_tap(g, src) || !has_sink(g, p) {
        return Ok(vec![]);
    }
    let creature = g.is_creature(src);
    let Some(ls) = candelabra_plan(g, p, creature) else { return Ok(vec![]) };
    Ok(vec![ability(2.2, "untap lands (Candelabra)", src, candelabra_go, ls.len() as i64)])
}

/// **Candelabra of Tawnos** ({1} artifact) and **Magus of the Candelabra** ({G}, 1/2 creature): "{X}, {T}: untap X
/// target lands". The AI uses it when the lands it untaps (Gaea's Cradle first) make at least {2} more than X cost
/// (Magus: less with Training Grounds / Biomancer's Familiar, never under {1}), and a spell or Thrasios waits.
fn candelabra_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let creature = g.is_creature(src);
    if owner(g, src) != p || !can_tap(g, src) {
        return Ok(false);
    }
    let Some(ls) = candelabra_plan(g, p, creature).filter(|l| l.len() as i64 == arg) else { return Ok(false) };
    let cost = if creature { creature_ability_cost(g, p, ls.len() as u32) } else { ls.len() as u32 };
    pay(g, p, cost, "", false)?;
    g.perm_mut(src).tapped = true; // the cost
    if !ability_window(g, p, Some(src), "untap target lands", None, None)? {
        return Ok(true);
    }
    for l in ls {
        if land_on(g, p, l) {
            g.perm_mut(l).tapped = false;
        }
    }
    crate::glog!(g, "    {} untaps {} lands ({})", g.player(p).name, arg, g.perm(src).name);
    Ok(true)
}

// ================================================================== Formidable Speaker
/// **Formidable Speaker** ({2}{G}, 2/4): when it enters, you may discard a card; if you do, search for a creature
/// card (the deck's wish list first, `tutor_named`). The AI discards only a card its discard rule would give up
/// (`discard_worst`) and only when the library has a creature. "{1},{T}: untap another target permanent": modelled
/// Candelabra / Magus / mana rock (the AI chains onto Candelabra when its lands net more).
fn speaker_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let p = owner(g, src);
    if m != src || g.player(p).hand.is_empty() {
        return Ok(());
    }
    if !g.player(p).library.iter().any(|&c| g.db.get(c).creature) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "discard a card, search for a creature", None)? {
        return Ok(());
    }
    let h = g.player(p).hand.len();
    crate::engine::cast::discard_worst(g, p, 1)?;
    if g.player(p).hand.len() < h {
        super::t1::tutor_named(g, p, &|g, c| g.db.get(c).creature, 1, "hand")?;
    }
    Ok(())
}

/// untap permanent m (a creature or artifact; lands go through `PermId`): clears `tapped`, so a tapped {T} ability
/// (Candelabra, Magus, a mana rock or dork) can be used again. False when it is not a live tapped permanent.
pub fn untap_perm(g: &mut Game, m: PermId) -> bool {
    if !live(g, m) || !g.perm(m).tapped {
        return false;
    }
    g.perm_mut(m).tapped = false;
    true
}

/// what untapping p's tapped permanent m is worth in mana now (0 when nothing): Candelabra / Magus when lands are
/// tapped to untap with it (the plan's mana less its cost), a mana rock or dork by what it makes
fn untap_value(g: &Game, p: PlayerId, m: PermId) -> i64 {
    let x = g.perm(m);
    if !live(g, m) || !x.tapped || x.owner != p {
        return 0;
    }
    let Some(c) = x.cd else { return 0 };
    let name = &*g.db.get(c).name;
    if matches!(name, "Candelabra of Tawnos" | "Magus of the Candelabra") {
        let creature = g.is_creature(m);
        return candelabra_plan(g, p, creature).map_or(0, |ls| {
            let sum: i64 = ls.iter().map(|&l| land_yield(g, p, l) as i64).sum();
            let n = ls.len() as u32;
            sum - if creature { creature_ability_cost(g, p, n) } else { n } as i64
        });
    }
    let t = &g.db.get(c).tags;
    match t.str(Tag::Rock).or_else(|| t.str(Tag::Dork)) {
        Some(s) => cardcode_amt(g, p, m, s),
        None => 0,
    }
}

/// Speaker's arg: a land id (>= 0), or -(permanent id + 1) for a creature or artifact
fn speaker_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || owner(g, src) != p || !can_tap(g, src) || !has_sink(g, p) {
        return Ok(vec![]);
    }
    let cost = creature_ability_cost(g, p, 1);
    if !can_pay(g, p, cost, "", false) {
        return Ok(vec![]);
    }
    // chain onto a tapped Candelabra / Magus / mana rock when that nets more than the best land
    let perms: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| m != src).collect();
    let best_perm = perms.iter().map(|&m| (untap_value(g, p, m), m)).max_by_key(|x| x.0);
    let tapped: Vec<PermId> = g.player(p).lands.iter().copied().filter(|&l| g.perm(l).tapped).collect();
    let best_land = crate::engine::zones::max_by(&tapped, |l| land_yield(g, p, l) as f64);
    let land_net = best_land.map_or(0, |t| land_yield(g, p, t) as i64);
    if let Some((_, m)) = best_perm.filter(|&(v, _)| v - cost as i64 >= 2 && v > land_net) {
        return Ok(vec![ability(2.2, "untap a permanent (Formidable Speaker)", src, speaker_go, -(m.0 as i64) - 1)]);
    }
    let Some(t) = best_land else { return Ok(vec![]) };
    if land_net - (cost as i64) < 2 {
        return Ok(vec![]);
    }
    Ok(vec![ability(2.2, "untap a land (Formidable Speaker)", src, speaker_go, t.0 as i64)])
}

/// the mana a rock or dork with tag value `s` ("amt:colours" or just colours) makes
fn cardcode_amt(g: &Game, p: PlayerId, m: PermId, s: &str) -> i64 {
    match crate::cardcode::dyn_mana_perm(g, p, m) {
        Some(n) => n as i64,
        None => s.split_once(':').map_or(1, |a| a.0.parse().unwrap_or(1)),
    }
}

fn speaker_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let cost = creature_ability_cost(g, p, 1);
    if owner(g, src) != p || !can_tap(g, src) || !can_pay(g, p, cost, "", false) {
        return Ok(false);
    }
    let target = if arg < 0 { Some(PermId((-arg - 1) as u32)) } else { None };
    match target {
        Some(m) if m == src || !live(g, m) || g.perm(m).owner != p || !g.perm(m).tapped => return Ok(false),
        None if !land_on(g, p, PermId(arg as _)) => return Ok(false),
        _ => {}
    }
    pay(g, p, cost, "", false)?;
    g.perm_mut(src).tapped = true;
    if !ability_window(g, p, Some(src), "untap target permanent", None, None)? {
        return Ok(true);
    }
    if let Some(m) = target {
        if untap_perm(g, m) {
            crate::glog!(g, "    Formidable Speaker untaps {}", g.perm(m).name);
        }
    } else {
        let l = PermId(arg as _);
        if land_on(g, p, l) {
            g.perm_mut(l).tapped = false;
            crate::glog!(g, "    Formidable Speaker untaps {}", g.db.get(g.perm(l).card()).name);
        }
    }
    Ok(true)
}

// ================================================================== Vibrance
thread_local! {
    /// the mode an evoked Vibrance was paid for (1 = {R}{R}, 2 = {G}{G}); 0 when it was cast normally
    static VIBRANCE_MODE: std::cell::Cell<u8> = const { std::cell::Cell::new(0) };
}

/// **Vibrance** ({3}{R/G}{R/G}, 4/4, evoke {R/G}{R/G}): when it enters, if {R}{R} was spent, 3 damage to any target;
/// if {G}{G} was spent, search for a land card (to hand) and gain 2 life. The card data prices the hybrid as {R}{R},
/// so a normal cast always spent {R}{R} (3 damage, no land); the evoke options below pay for one mode and sacrifice it.
/// The AI evokes for {R}{R} when 3 damage kills a creature worth killing or finishes a player, and for {G}{G} when it
/// wants a land from the library (Gaea's Cradle or a Gate not yet in hand or play).
fn vibrance_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src || !g.last_cast_etb {
        return Ok(()); // "if {R}{R} was spent to cast it": not when it entered without being cast
    }
    let p = owner(g, src);
    let mode = VIBRANCE_MODE.with(|c| c.replace(0));
    if mode != 2 && trigger_window(g, p, Some(src), "3 damage to any target", None)? {
        super::t2::best_target_any(g, p, 3)?;
    }
    if mode == 2 && trigger_window(g, p, Some(src), "search for a land, gain 2 life", None)? {
        let want = vibrance_land_wanted(g, p);
        let key = |g: &Game, c: CardId| {
            let n = &*g.db.get(c).name;
            n == "Gaea's Cradle" || (n == "Talon Gates of Madara" && !g.player(p).library.iter().any(|&x| &*g.db.get(x).name == "Gaea's Cradle"))
        };
        super::t1::tutor_named(g, p, &|g, c| g.db.get(c).land && (!want || key(g, c)), 1, "hand")?;
        crate::engine::life::gain(g, p, 2)?;
    }
    Ok(())
}

/// the land Vibrance's {G}{G} mode would find: a wished land not yet held or in play (Gaea's Cradle, Talon Gates)
fn vibrance_land_wanted(g: &Game, p: PlayerId) -> bool {
    let held = |name: &str| {
        g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
            || g.player(p).lands.iter().any(|&l| &*g.db.get(g.perm(l).card()).name == name)
    };
    g.player(p)
        .library
        .iter()
        .any(|&c| matches!(&*g.db.get(c).name, "Gaea's Cradle" | "Talon Gates of Madara") && !held(&g.db.get(c).name))
}

fn vibrance_hand(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut o = vec![];
    if post.is_none() || !crate::engine::cast::castable(g, p, c, "hand") {
        return Ok(o);
    }
    // the full cast ({3}{R}{R}) is the engine's own; evoke is for when that is out of reach
    let (fg, fp) = crate::engine::mana::cost_of(g, p, c);
    if can_pay(g, p, fg, &fp, false) {
        return Ok(o);
    }
    let plan = |u: f64, l: &str, mode: i64| Opt {
        utility: u,
        label: l.into(),
        act: Some(Action::Plan { f: vibrance_evoke, arg: mode }),
    };
    if can_pay(g, p, 0, "RR", false) {
        let kill = g
            .opps(p)
            .flat_map(|q| g.player(q).perms.clone())
            .filter(|&m| g.is_creature(m) && crate::engine::values::etgh(g, m) <= 3)
            .map(|m| crate::engine::values::pval(g, m))
            .fold(0.0, f64::max);
        let lethal = g.opps(p).any(|q| g.player(q).life <= 3);
        if kill >= 3.0 || lethal {
            o.push(plan(if lethal { 6.0 } else { kill - 0.5 }, "evoke Vibrance ({R}{R}: 3 damage)", 1));
        }
    }
    if can_pay(g, p, 0, "GG", false) && vibrance_land_wanted(g, p) {
        o.push(plan(2.6, "evoke Vibrance ({G}{G}: land, 2 life)", 2));
    }
    Ok(o)
}

fn vibrance_evoke(g: &mut Game, p: PlayerId, mode: i64) -> Res<bool> {
    let Some(c) = g.db.id("Vibrance") else { return Ok(false) };
    let pips = if mode == 2 { "GG" } else { "RR" };
    let Some(i) = g.player(p).hand.iter().position(|&x| x == c) else { return Ok(false) };
    if !can_pay(g, p, 0, pips, false) {
        return Ok(false);
    }
    g.player_mut(p).hand.remove(i);
    pay(g, p, 0, pips, false)?;
    g.player_mut(p).spells_this_turn += 1;
    crate::engine::cast::on_cast(g, p, c)?;
    crate::glog!(g, "  {} evokes Vibrance ({})", g.player(p).name, pips);
    VIBRANCE_MODE.with(|v| v.set(mode as u8));
    let m = crate::engine::zones::enter(
        g,
        p,
        c,
        crate::engine::zones::Enter { was_cast: true, ..crate::engine::zones::Enter::default() },
    );
    VIBRANCE_MODE.with(|v| v.set(0));
    let m = m?;
    if live(g, m) {
        crate::engine::zones::die(g, m, "sac".into())?; // the evoke sacrifice
    }
    Ok(true)
}

// ================================================================== Springheart Nantuko
/// **Springheart Nantuko** ({1}{G} enchantment creature, 1/1): landfall: "you may pay {1}{G} if this is attached to a
/// creature you control: create a token copy of that creature; if you didn't, create a 1/1 green Insect". Modelled as
/// the unattached creature: each land you play makes a 1/1 Insect. Missing: bestow {1}{G} (cast attached to a
/// creature, +1/+1, the {1}{G} token copy of the enchanted creature, e.g. Eternal Witness) and the Aura falling off
/// as a creature (engine-needs-green1.md).
fn nantuko_landfall(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if !live(g, src) || owner(g, src) != p {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "create a 1/1 Insect", None)? {
        let spec = crate::engine::zones::Tokens {
            color: Some(crate::cards::Colors::from_letters("G")),
            types: vec!["insect"],
            sick: true,
            ..crate::engine::zones::Tokens::new(1, 1)
        };
        crate::engine::zones::make_tokens(g, p, spec)?;
    }
    Ok(())
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Oboro Breezecaller")?.options = Some(oboro_options);
    r.card(db, "Badgermole Cub")?.mana_tapped = Some(cub_tapped);
    r.card(db, "Earthcraft")?.options = Some(earthcraft_options);
    r.card(db, "Candelabra of Tawnos")?.options = Some(candelabra_options);
    r.card(db, "Magus of the Candelabra")?.options = Some(candelabra_options);
    r.card(db, "Springheart Nantuko")?.landfall = Some(nantuko_landfall);
    let v = r.card(db, "Vibrance")?;
    v.etb = Some(vibrance_etb);
    v.hand_options = Some(vibrance_hand);
    let s = r.card(db, "Formidable Speaker")?;
    s.etb = Some(speaker_etb);
    s.options = Some(speaker_options);
    Ok(())
}
