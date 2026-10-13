//! The cloud deck (decklists/JD/cloud-midgar-equipment.md): mono-white Equipment around Cloud, Midgar Mercenary. Its
//! card code and AI plan. The brief is rust/agents/cloud-core.md; the verdicts are in data/card_overrides.toml.
//!
//! What lives here:
//! - **Cloud's trigger doubling**: `doubles`, asked by `stack::queue_triggers` for every trigger it queues. While Cloud
//!   is equipped, a trigger whose source is Cloud, or an Equipment attached to him, goes on the queue twice. (An
//!   Equipment's "equipped creature has 'whenever this creature attacks ...'" is queued with the Equipment as its
//!   source, so it counts as long as the Equipment is on Cloud.)
//! - **Attaching**: `attach` (free, no cost), `equip_cost` (the equip cost of an Equipment for one creature: Wrecking
//!   Ball Arm for a legend {3}, Hulkbuster Armor for a Hero {3}, Veteran's Powerblade for a Soldier {W}, Puresteel
//!   Paladin's metalcraft {0}, Helitrooper -{2}), read by `dsl::equip_options`.
//! - The Equipment and the creatures of the brief (see `register`), the evasion rules (`evasion_blocked`: Wrecking Ball
//!   Arm, Trailblazer's Boots, Magitek Scythe's "must be blocked" in `must_be_blocked`) and the deck's AI plan.
//!
//! The plan: Cloud on turn 2, his search by need (`cloud_pick`), every Equipment onto Cloud (`holder_bonus` biases
//! `dsl::equip_options`), the protection pieces kept for him (`cloud_protect`).

use crate::cards::{CardDb, Types};
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::values::{epow, etgh, has_type, pval};
use crate::engine::zones::{Enter, Tokens, draw, enter, make_tokens, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Sacrificed, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;
use crate::sym::intern;

pub const CLOUD: &str = "Cloud, Midgar Mercenary";
const KEY: &str = "cloud";

// ------------------------------------------------------------------ small helpers
fn live(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased
}

fn name_of(g: &Game, m: PermId) -> &str {
    g.perm(m).cd.map_or("", |c| &g.db.get(c).name)
}

fn card_name(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

/// m is an Equipment permanent
pub fn is_equipment(g: &Game, m: PermId) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).has_subtype("equipment"))
}

fn is_equipment_card(g: &Game, c: CardId) -> bool {
    g.db.get(c).has_subtype("equipment")
}

fn is_artifact(g: &Game, m: PermId) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT))
}

/// the Equipment attached to m (all of its controller's)
pub fn equipment_on(g: &Game, m: PermId) -> Vec<PermId> {
    g.player(owner(g, m))
        .perms
        .iter()
        .copied()
        .filter(|&e| g.perm(e).attached == Some(m) && g.perm(e).on_bf && is_equipment(g, e))
        .collect()
}

/// p's Equipment on the battlefield
pub fn my_equipment(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&e| live(g, e) && is_equipment(g, e)).collect()
}

/// p's Cloud, if he is out
pub fn cloud_of(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && name_of(g, m) == CLOUD)
}

/// p's creature named `name`, if out
fn creature_named(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && name_of(g, m) == name)
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| card_name(g, c) == name)
}

// ------------------------------------------------------------------ Cloud: the trigger doubling
/// the Cloud whose doubling applies to a trigger sourced by `s` (Cloud himself, or an Equipment attached to him),
/// when he is equipped
fn doubling_cloud(g: &Game, s: PermId) -> Option<PermId> {
    let x = g.perm(s);
    let cd = x.cd?;
    let cloud = if &*g.db.get(cd).name == CLOUD {
        s
    } else if let Some(t) = x.attached
        && g.perm(t).on_bf
        && g.db.get(cd).has_subtype("equipment")
        && g.perm(t).cd.is_some_and(|c| &*g.db.get(c).name == CLOUD)
    {
        t
    } else {
        return None;
    };
    (g.perm(cloud).on_bf && !g.perm(cloud).phased && !equipment_on(g, cloud).is_empty()).then_some(cloud)
}

/// a trigger of `s` triggers an additional time (Cloud, Midgar Mercenary): asked by `stack::queue_triggers`
pub fn doubles(g: &Game, s: PermId) -> bool {
    doubling_cloud(g, s).is_some()
}

// ------------------------------------------------------------------ attaching
/// attach Equipment e to creature m, free: both must be out and under the same control
pub fn attach(g: &mut Game, e: PermId, m: PermId) -> bool {
    if !live(g, e) || !live(g, m) || !g.is_creature(m) || owner(g, e) != owner(g, m) || e == m {
        return false;
    }
    if g.perm(e).attached == Some(m) {
        return true;
    }
    crate::glog!(g, "    {} attaches to {}", g.perm(e).name, g.perm(m).name);
    g.perm_mut(e).attached = Some(m);
    true
}

/// how good a creature is as the one wearing p's Equipment: Cloud above all, then size, evasion and being ready
pub fn holder_score(g: &Game, m: PermId) -> f64 {
    let x = g.perm(m);
    let mut s = 0.3 * epow(g, m) as f64 + 0.1 * etgh(g, m) as f64;
    if name_of(g, m) == CLOUD {
        s += 50.0;
    }
    if x.fly || crate::dsl::has_kw(g, m, "flying") {
        s += 2.0;
    }
    if x.sick {
        s -= 1.0;
    }
    if x.noatk || !g.is_creature(m) {
        s -= 100.0;
    }
    s
}

/// p's best creature to wear Equipment
pub fn best_holder(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m) && !g.perm(m).noatk).collect();
    max_by(&cs, |m| holder_score(g, m))
}

/// the extra weight the cloud deck gives a creature as the target of an equip (read by `dsl::equip_options`)
pub fn holder_bonus(g: &Game, p: PlayerId, m: PermId) -> f64 {
    if g.player(p).key != KEY {
        return 0.0;
    }
    if name_of(g, m) == CLOUD { 20.0 } else { 0.0 }
}

/// the extra utility the cloud deck gives equipping this piece to m (read by `dsl::equip_options`)
pub fn equip_utility_bonus(g: &Game, p: PlayerId, e: PermId, m: PermId) -> f64 {
    if g.player(p).key != KEY {
        return 0.0;
    }
    let mut u = 0.0;
    if name_of(g, m) == CLOUD {
        u += 4.5; // the plan: Cloud wears everything, every turn
        if matches!(name_of(g, e), "Wrecking Ball Arm" | "Hulkbuster Armor" | "Colossus Hammer") {
            u += 1.5; // the base or the size is worth more than the +N/+N the formula counts
        }
        if g.perm(e).cd.is_some_and(|c| {
            g.db.get(c).abilities.iter().any(|a| matches!(a, crate::dsl::model::Ability::Triggered { .. }))
        }) {
            u += 1.5; // a trigger he doubles
        }
    }
    u
}

/// an Equipment (attached to something other than Cloud) the cloud deck wants to move onto Cloud
pub fn wants_move(g: &Game, p: PlayerId, e: PermId) -> bool {
    g.player(p).key == KEY && cloud_of(g, p).is_some_and(|c| g.perm(e).attached != Some(c))
}

fn metalcraft(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().filter(|&&m| live(g, m) && is_artifact(g, m)).count() >= 3
}

/// the equip cost of Equipment e (printed `base`) onto creature m, with the cards that change it
pub fn equip_cost(g: &Game, p: PlayerId, e: PermId, m: PermId, base: u32) -> u32 {
    let mut n = base;
    match name_of(g, e) {
        "Wrecking Ball Arm" if crate::cardcode::is_legendary(g, m) => n = n.min(3), // Equip legendary creature {3}
        "Hulkbuster Armor" if has_type(g, m, "hero") => n = n.min(3),               // Equip Hero {3}
        "Veteran's Powerblade" if has_type(g, m, "soldier") => n = n.min(1),        // Equip Soldier {W}
        _ => {}
    }
    if metalcraft(g, p) && creature_named(g, p, "Puresteel Paladin").is_some() {
        n = 0; // Puresteel Paladin: equip {0} with three or more artifacts
    }
    if name_of(g, m) == "Helitrooper" {
        n = n.saturating_sub(2); // Equip abilities that target Helitrooper cost {2} less
    }
    // Auriok Steelshaper: equip costs you pay cost {1} less each (generic only: Veteran's Powerblade's {W} stays)
    let colored = name_of(g, e) == "Veteran's Powerblade" && has_type(g, m, "soldier");
    if !colored {
        let shapers = g.player(p).perms.iter().filter(|&&s| live(g, s) && name_of(g, s) == STEELSHAPER).count() as u32;
        n = n.saturating_sub(shapers);
    }
    n
}

// ------------------------------------------------------------------ evasion
/// can't-be-blocked rules of the Equipment of the deck: b can't block a (read by `cardcode::evasion_blocked`)
pub fn evasion_blocked(g: &Game, b: PermId, a: PermId) -> bool {
    let p = owner(g, a);
    for e in equipment_on(g, a) {
        match name_of(g, e) {
            // Equipped creature can't be blocked by creatures with power 2 or less
            "Wrecking Ball Arm" if epow(g, b) <= 2 => return true,
            // nonbasic landwalk: unblockable while the defending player controls a nonbasic land
            "Trailblazer's Boots" => {
                let d = owner(g, b);
                if d != p && g.player(d).lands.iter().any(|&l| g.land(l).on_bf && !g.db.get(g.land(l).cd).basic) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// the equipped creature loses flying (Colossus Hammer)
pub fn loses_flying(g: &Game, m: PermId) -> bool {
    equipment_on(g, m).iter().any(|&e| name_of(g, e) == "Colossus Hammer")
}

/// base power and toughness an Equipment sets on its creature (Wrecking Ball Arm 7/7, Hulkbuster Armor 9/9), the
/// largest of those on it
pub fn base_pt(g: &Game, m: PermId) -> Option<(i32, i32)> {
    equipment_on(g, m)
        .iter()
        .filter_map(|&e| match name_of(g, e) {
            "Wrecking Ball Arm" => Some((7, 7)),
            "Hulkbuster Armor" => Some((9, 9)),
            _ => None,
        })
        .max()
}

/// attackers marked "must be blocked if able" (Magitek Scythe) get a blocker if any can block them: `ai_blocks` calls
/// this with the blocks it chose and adds the missing ones
pub fn must_be_blocked(g: &Game, a: PermId) -> bool {
    g.perm(a).eot_kw.contains(&"must be blocked")
}

/// an attacker that must be blocked if able (Magitek Scythe) gets the cheapest blocker that can block it, when the
/// blocks the AI chose left it alone
pub fn force_blocks(g: &Game, d: PlayerId, atk: &[PermId], unbl: &[PermId], assign: &mut Vec<(PermId, PermId)>) {
    for &a in atk {
        if !must_be_blocked(g, a) || unbl.contains(&a) || assign.iter().any(|x| x.0 == a) || !g.perm(a).on_bf {
            continue;
        }
        let cands: Vec<PermId> = g
            .player(d)
            .perms
            .iter()
            .copied()
            .filter(|&b| g.is_creature(b) && !g.perm(b).tapped && !g.perm(b).phased)
            .filter(|&b| !assign.iter().any(|x| x.1 == b) && crate::engine::combat::can_block(g, b, a))
            .collect();
        if let Some(b) = crate::engine::zones::min_by(&cands, |b| pval(g, b)) {
            assign.push((a, b));
        }
    }
}

// ------------------------------------------------------------------ Cloud, Midgar Mercenary: the search
/// the order the deck wants its Equipment found in, by need
fn cloud_pick(g: &Game, p: PlayerId) -> Option<CardId> {
    let lib: Vec<CardId> =
        crate::engine::zones::searchable(g, p).into_iter().filter(|&c| is_equipment_card(g, c)).collect();
    if lib.is_empty() {
        return None;
    }
    let out = |n: &str| {
        creature_named(g, p, n).is_some() || g.player(p).perms.iter().any(|&m| live(g, m) && name_of(g, m) == n)
    };
    let have = |n: &str| out(n) || in_hand(g, p, n);
    let first = |ns: &[&str]| -> Option<CardId> {
        ns.iter().filter(|n| !have(n)).filter_map(|n| g.db.id(n)).find(|c| lib.contains(c))
    };
    let held = my_equipment(g, p).len() + g.player(p).hand.iter().filter(|&&c| is_equipment_card(g, c)).count();
    let kaldra = ["Sword of Kaldra", "Shield of Kaldra", "Helm of Kaldra"];
    let kaldra_out = kaldra.iter().filter(|n| have(n)).count();
    if kaldra_out == 2 {
        // the two other Kaldra pieces out: the third
        if let Some(c) = first(&kaldra) {
            return Some(c);
        }
    }
    if let Some(cl) = cloud_of(g, p)
        && !equipment_on(g, cl).iter().any(|&e| name_of(g, e) == "Shield of Kaldra")
        && crate::ai::brain::removal_risk(g, p) >= 0.35
        && let Some(c) = first(&["Shield of Kaldra"])
    {
        return Some(c); // Cloud is threatened
    }
    let order: &[&str] = if held == 0 {
        &["Wrecking Ball Arm", "Hero's Blade", "Sword of Kaldra", "Mask of Memory"]
    } else if held == 1 {
        &["Sword of Kaldra", "Mask of Memory", "Wrecking Ball Arm", "Hero's Blade"]
    } else {
        &[
            "Mask of Memory",
            "Sword of Kaldra",
            "Colossus Hammer",
            "Hulkbuster Armor",
            "Shield of Kaldra",
            "Helm of Kaldra",
        ]
    };
    if let Some(c) = first(order) {
        return Some(c);
    }
    const REST: [&str; 18] = [
        "Mask of Memory",
        "Sword of Kaldra",
        "Wrecking Ball Arm",
        "Hulkbuster Armor",
        "Colossus Hammer",
        "Conformer Shuriken",
        "Shield of Kaldra",
        "Helm of Kaldra",
        "Hero's Blade",
        "Medic's Kitesail",
        "Magitek Scythe",
        "White Mage's Staff",
        "Veteran's Powerblade",
        "Explorer's Scope",
        "Trailblazer's Boots",
        "Squire's Lightblade",
        "Nim Deathmantle",
        "Hero's Heirloom",
    ];
    first(&REST).or_else(|| lib.iter().copied().find(|&c| !have(card_name(g, c))))
}

/// When Cloud enters, search your library for an Equipment card, reveal it, put it into your hand, then shuffle.
fn cloud_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "search for an Equipment", Some(7.0))? {
        return Ok(());
    }
    let pick = if crate::human::is_human(g, o) { None } else { cloud_pick(g, o) };
    let human = crate::human::is_human(g, o);
    let got = super::t1::tutor_named(g, o, &|g, c| is_equipment_card(g, c) && (human || Some(c) == pick), 1, "hand")?;
    for c in got {
        crate::glog!(g, "    {} reveals {}", g.player(o).name, card_name(g, c));
    }
    Ok(())
}

// ------------------------------------------------------------------ Hero's Blade, Armory Automaton, Beatrix
/// Hero's Blade: whenever a legendary creature you control enters, you may attach it to that creature
fn blade_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if m == src || owner(g, m) != o || !live(g, src) || !live(g, m) || !g.is_creature(m) {
        return Ok(());
    }
    if !crate::cardcode::is_legendary(g, m) {
        return Ok(());
    }
    // worth it when the Blade is loose, or the new legend is the better one to wear it
    let better = match g.perm(src).attached.filter(|&t| live(g, t)) {
        None => true,
        Some(t) => t != m && holder_score(g, m) > holder_score(g, t) + 1.0,
    };
    if better && trigger_window(g, o, Some(src), "attach to the legendary creature", Some(6.0))? {
        attach(g, src, m);
    }
    Ok(())
}

/// Equipment worth taking onto `to`: loose, or on a creature that is a worse holder
fn pullable(g: &Game, p: PlayerId, to: PermId) -> Vec<PermId> {
    my_equipment(g, p)
        .into_iter()
        .filter(|&e| match g.perm(e).attached.filter(|&t| live(g, t)) {
            None => true,
            Some(t) => t != to && holder_score(g, to) > holder_score(g, t) + 1.0,
        })
        .collect()
}

/// Armory Automaton: whenever it enters or attacks, you may attach any number of target Equipment to it
fn armory_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    armory_attach(g, src)
}

fn armory_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) == p && atk.contains(&src) {
        armory_attach(g, src)?;
    }
    Ok(vec![])
}

fn armory_attach(g: &mut Game, src: PermId) -> Res {
    let o = owner(g, src);
    let es: Vec<PermId> = pullable(g, o, src).into_iter().filter(|&e| g.perm(e).attached != Some(src)).collect();
    if es.is_empty() || !trigger_window(g, o, Some(src), "attach Equipment to itself", Some(5.0))? {
        return Ok(());
    }
    for e in es {
        attach(g, e, src);
    }
    Ok(())
}

/// Beatrix: at the beginning of combat on your turn, you may attach any number of Equipment you control to target
/// creature you control (the best holder)
fn beatrix_combat(g: &mut Game, src: Src, p: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) != p || !live(g, src) {
        return Ok(vec![]);
    }
    let Some(to) = best_holder(g, p) else { return Ok(vec![]) };
    let es = pullable(g, p, to);
    if es.is_empty() || !trigger_window(g, p, Some(src), "attach Equipment", Some(5.0))? {
        return Ok(vec![]);
    }
    for e in es {
        attach(g, e, to);
    }
    Ok(vec![])
}

// ------------------------------------------------------------------ Kor Outfitter, Brass Squire, Auriok Steelshaper
const STEELSHAPER: &str = "Auriok Steelshaper";

/// the Equipment p most wants moved onto `to`: loose ones first, then by what it gives the wearer
fn best_pull(g: &Game, p: PlayerId, to: PermId) -> Option<PermId> {
    let es: Vec<PermId> = pullable(g, p, to).into_iter().filter(|&e| g.perm(e).attached != Some(to)).collect();
    max_by(&es, |e| {
        let loose = g.perm(e).attached.is_none_or(|t| !live(g, t));
        (if loose { 10.0 } else { 0.0 }) + equip_utility_bonus(g, p, e, to)
    })
}

/// Kor Outfitter: when it enters, you may attach target Equipment you control to target creature you control (the
/// best holder, Cloud when he is out; a loose Equipment, or one worn by a worse creature)
fn outfitter_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    let Some(to) = best_holder(g, o) else { return Ok(()) };
    if best_pull(g, o, to).is_none() || !trigger_window(g, o, Some(src), "attach Equipment", Some(5.0))? {
        return Ok(());
    }
    // targets are chosen again as it resolves: the same choice, if still legal
    if let Some(to) = best_holder(g, o)
        && let Some(e) = best_pull(g, o, to)
    {
        attach(g, e, to);
    }
    Ok(())
}

/// Brass Squire's Equipment and the creature to put it on: the best holder other than Squire itself
fn squire_target(g: &Game, p: PlayerId, src: PermId) -> Option<(PermId, PermId)> {
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| m != src && live(g, m) && g.is_creature(m) && !g.perm(m).noatk)
        .collect();
    let to = max_by(&cs, |m| holder_score(g, m))?;
    Some((best_pull(g, p, to)?, to))
}

/// {T}: attach target Equipment you control to target creature you control
fn squire_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || g.perm(src).tapped || g.perm(src).sick {
        return Ok(false);
    }
    let Some((e, to)) = squire_target(g, p, src) else { return Ok(false) };
    g.perm_mut(src).tapped = true;
    if ability_window(g, p, Some(src), "attach Equipment", Some(5.0), Some(to))? && live(g, e) && live(g, to) {
        attach(g, e, to);
    }
    Ok(true)
}

/// before combat on your turn, Brass Squire moves an Equipment onto the best holder (a free equip, no sorcery timing)
fn squire_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if owner(g, src) != p || !live(g, src) || x.tapped || x.sick || post != Some(false) || g.active != Some(p) {
        return Ok(vec![]);
    }
    let Some((e, to)) = squire_target(g, p, src) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: 5.5 + if name_of(g, to) == CLOUD { 1.5 } else { 0.0 },
        label: format!("Brass Squire: attach {} to {}", name_of(g, e), name_of(g, to)),
        act: Some(Action::Ability { src, f: squire_go, arg: 0 }),
    }])
}

/// Auriok Steelshaper: as long as it is equipped, each Soldier and Knight you control gets +1/+1 (read by `dsl::pt`)
pub fn steelshaper_bonus(g: &Game, m: PermId) -> i32 {
    if !(has_type(g, m, "soldier") || has_type(g, m, "knight")) {
        return 0;
    }
    g.player(owner(g, m))
        .perms
        .iter()
        .filter(|&&s| live(g, s) && name_of(g, s) == STEELSHAPER && !equipment_on(g, s).is_empty())
        .count() as i32
}

// ------------------------------------------------------------------ Puresteel Paladin
/// Puresteel Paladin: whenever an Equipment you control enters, you may draw a card
fn puresteel_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if owner(g, m) == o
        && live(g, src)
        && is_equipment(g, m)
        && trigger_window(g, o, Some(src), "draw a card", Some(4.0))?
    {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ the enter-and-attach Equipment
/// Magitek Scythe: when it enters, you may attach it to target creature you control; if you do, that creature gains
/// first strike until end of turn and must be blocked this turn if able
fn scythe_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    let Some(to) = best_holder(g, o) else { return Ok(()) };
    if trigger_window(g, o, Some(src), "attach to a creature", Some(5.0))? && attach(g, src, to) {
        super::partials::eot_kw(g, to, "first strike");
        super::partials::eot_kw(g, to, "must be blocked");
    }
    Ok(())
}

/// Squire's Lightblade: when it enters, attach it to target creature you control; that creature gains first strike
/// until end of turn
fn lightblade_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    let Some(to) = best_holder(g, o) else { return Ok(()) };
    if trigger_window(g, o, Some(src), "attach to a creature", Some(5.0))? && attach(g, src, to) {
        super::partials::eot_kw(g, to, "first strike");
    }
    Ok(())
}

/// White Mage's Staff, job select: when it enters, create a 1/1 colorless Hero creature token, then attach this to it
fn staff_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "job select", Some(5.0))? {
        return Ok(());
    }
    let spec =
        Tokens { tgh: Some(1), color: Some(crate::cards::Colors::NONE), types: vec!["hero"], ..Tokens::new(1, 1) };
    if let Some(&hero) = make_tokens(g, o, spec)?.first() {
        attach(g, src, hero);
    }
    Ok(())
}

// ------------------------------------------------------------------ Cid, Helitrooper
/// Cid, Freeflier Pilot: Equipment and Vehicle spells you cast cost {1} less
fn cid_cost(g: &Game, src: Src, caster: PlayerId, c: CardId) -> i32 {
    let d = g.db.get(c);
    if caster == owner(g, src) && live(g, src) && (d.has_subtype("equipment") || d.has_subtype("vehicle")) {
        -1
    } else {
        0
    }
}

/// Jump: during your turn, Cid has flying
fn cid_flying(g: &Game, src: Src, m: PermId, kw: crate::sym::Sym) -> bool {
    m == src && kw == "flying" && g.active == Some(owner(g, src))
}

/// the Equipment or Vehicle card Cid should fetch back: the best by worth, one worth casting
fn cid_target(g: &Game, p: PlayerId) -> Option<CardId> {
    let cs: Vec<CardId> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&c| g.db.get(c).has_subtype("equipment") || g.db.get(c).has_subtype("vehicle"))
        .collect();
    max_by(&cs, |c| crate::engine::tutors::card_worth(g, p, c, true))
}

/// {2}, {T}: return target Equipment or Vehicle card from your graveyard to your hand
fn cid_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if p != owner(g, src) || post.is_none() || x.tapped || x.sick || !live(g, src) || !can_pay(g, p, 2, "", false) {
        return Ok(vec![]);
    }
    if cid_target(g, p).is_none() {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 5.5,
        label: "Cid: return an Equipment".into(),
        act: Some(Action::Ability { src, f: cid_regrow, arg: 0 }),
    }])
}

fn cid_regrow(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let Some(c) = cid_target(g, p) else { return Ok(false) };
    if !can_pay(g, p, 2, "", false) || g.perm(src).tapped {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    g.perm_mut(src).tapped = true;
    if ability_window(g, p, Some(src), "return an Equipment", None, None)?
        && let Some(i) = g.player(p).gy.iter().position(|&x| x == c)
    {
        g.player_mut(p).gy.remove(i);
        g.player_mut(p).hand.push(c);
        g.player_mut(p).seen.insert(c);
        crate::glog!(g, "    Cid returns {}", card_name(g, c));
    }
    Ok(true)
}

/// Helitrooper: whenever it attacks, another target attacking creature gains flying until end of turn
fn helitrooper_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) != p || !atk.contains(&src) {
        return Ok(vec![]);
    }
    let cs: Vec<PermId> = atk
        .iter()
        .copied()
        .filter(|&m| m != src && live(g, m) && !g.perm(m).fly && !crate::dsl::has_kw(g, m, "flying"))
        .collect();
    let Some(t) = max_by(&cs, |m| epow(g, m) as f64) else { return Ok(vec![]) };
    if trigger_window(g, p, Some(src), "another attacker gains flying", Some(4.0))? && live(g, t) {
        super::partials::eot_kw(g, t, "flying");
    }
    Ok(vec![])
}

// ------------------------------------------------------------------ Conformer Shuriken, Explorer's Scope
/// the creature wearing Equipment e, if it is among the attackers of p
fn wearer_attacking(g: &Game, e: PermId, p: PlayerId, atk: &[PermId]) -> Option<PermId> {
    let t = g.perm(e).attached?;
    (owner(g, e) == p && live(g, e) && live(g, t) && atk.contains(&t)).then_some(t)
}

/// Conformer Shuriken: the equipped creature has "whenever this creature attacks, tap target creature defending
/// player controls. If that creature has greater power than this creature, put a number of +1/+1 counters on this
/// creature equal to the difference"
fn shuriken_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    let Some(me) = wearer_attacking(g, src, p, atk) else { return Ok(vec![]) };
    let cands: Vec<PermId> = g
        .player(d)
        .perms
        .iter()
        .copied()
        .filter(|&m| live(g, m) && g.is_creature(m) && !crate::engine::values::untargetable(g, m))
        .collect();
    let gain = |g: &Game, m: PermId| (epow(g, m) - epow(g, me)).max(0) as f64;
    let Some(t) = max_by(&cands, |m| (if g.perm(m).tapped { 0.0 } else { pval(g, m) }) + gain(g, m)) else {
        return Ok(vec![]);
    };
    if !trigger_window(g, p, Some(src), "tap a creature", Some(5.0))? || !live(g, t) || !live(g, me) {
        return Ok(vec![]);
    }
    g.perm_mut(t).tapped = true;
    let diff = epow(g, t) - epow(g, me);
    crate::glog!(g, "    Conformer Shuriken taps {}", g.perm(t).name);
    if diff > 0 {
        g.perm_mut(me).plus += diff;
        crate::glog!(g, "    {} gets {} +1/+1 counter(s)", g.perm(me).name, diff);
    }
    Ok(vec![])
}

/// Explorer's Scope: whenever equipped creature attacks, look at the top card of your library; if it's a land card,
/// you may put it onto the battlefield tapped
fn scope_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if wearer_attacking(g, src, p, atk).is_none() {
        return Ok(vec![]);
    }
    if g.player(p).library.is_empty() || !trigger_window(g, p, Some(src), "look at the top card", Some(3.0))? {
        return Ok(vec![]);
    }
    let Some(&top) = g.player(p).library.last() else { return Ok(vec![]) };
    if g.db.get(top).land {
        g.player_mut(p).library.pop();
        g.add_land_entering(p, top, true);
        crate::glog!(g, "    Explorer's Scope puts {} onto the battlefield tapped", card_name(g, top));
        crate::engine::zones::landfall(g, p)?;
    }
    Ok(vec![])
}

// ------------------------------------------------------------------ Magnetic Snuffler, Jocasta, Zack Fair
/// Magnetic Snuffler: when it enters, return target Equipment card from your graveyard to the battlefield attached to
/// it
fn snuffler_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    let cs: Vec<CardId> = g.player(o).gy.iter().copied().filter(|&c| is_equipment_card(g, c)).collect();
    let Some(c) = max_by(&cs, |c| crate::engine::tutors::card_worth(g, o, c, true)) else { return Ok(()) };
    if !trigger_window(g, o, Some(src), "return an Equipment", Some(5.0))? {
        return Ok(());
    }
    let Some(i) = g.player(o).gy.iter().position(|&x| x == c) else { return Ok(()) };
    g.player_mut(o).gy.remove(i);
    let e = enter(g, o, c, Enter { sick: false, ..Default::default() })?;
    attach(g, e, src);
    Ok(())
}

/// Whenever you sacrifice an artifact, put a +1/+1 counter on this creature
fn snuffler_sacrifice(g: &mut Game, src: Src, p: PlayerId, what: Sacrificed) -> Res {
    if p != owner(g, src) || !live(g, src) {
        return Ok(());
    }
    let artifact = match what {
        Sacrificed::Perm(m) => g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT)),
        Sacrificed::Token(k) => matches!(&*k, "Treasure" | "Food" | "Clue"),
        Sacrificed::Card(_) => false,
    };
    if artifact {
        g.perm_mut(src).plus += 1;
    }
    Ok(())
}

/// Jocasta: whenever your commander deals combat damage to a player, put a +1/+1 counter on Jocasta
fn jocasta_damage(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, dmg: i32) -> Res {
    if p == owner(g, src) && dmg > 0 && live(g, src) && g.perm(a).is_cmd && owner(g, a) == p {
        g.perm_mut(src).plus += 1;
    }
    Ok(())
}

/// Jocasta in the graveyard: whenever you attack with your commander, you may return her to the battlefield tapped
/// and attacking (called from `combat::attack_triggers_once`)
pub fn jocasta_attack(g: &mut Game, p: PlayerId, atk: &[PermId]) -> Res<Vec<PermId>> {
    if g.player(p).key != KEY || !atk.iter().any(|&m| g.perm(m).is_cmd && owner(g, m) == p) {
        return Ok(vec![]);
    }
    let Some(c) = g.player(p).gy.iter().copied().find(|&c| card_name(g, c) == "Jocasta, Automaton Avenger") else {
        return Ok(vec![]);
    };
    if !crate::engine::stack::trigger_window_card(g, p, c, "return Jocasta attacking")? {
        return Ok(vec![]);
    }
    let Some(i) = g.player(p).gy.iter().position(|&x| x == c) else { return Ok(vec![]) };
    g.player_mut(p).gy.remove(i);
    let m = enter(g, p, c, Enter { sick: false, ..Default::default() })?;
    g.perm_mut(m).tapped = true;
    Ok(vec![m])
}

/// Zack Fair enters with a +1/+1 counter on it
fn zack_enters(g: &mut Game, _p: PlayerId, m: PermId) -> Res {
    g.perm_mut(m).plus += 1;
    Ok(())
}

/// {1}, Sacrifice Zack Fair: target creature you control gains indestructible until end of turn; put Zack's counters
/// on that creature and attach an Equipment that was attached to Zack to that creature. Used by `cloud_protect`.
pub fn zack_sacrifice(g: &mut Game, src: PermId, p: PlayerId, target: i64) -> Res<bool> {
    let t = PermId(target as u32);
    if !live(g, src) || !live(g, t) || t == src || owner(g, t) != p || !can_pay(g, p, 1, "", false) {
        return Ok(false);
    }
    pay(g, p, 1, "", false)?;
    let counters = g.perm(src).plus.max(0);
    let worn = equipment_on(g, src);
    crate::engine::zones::die(g, src, intern("sac"))?; // Zack is sacrificed as a cost: it happens first
    if !ability_window(g, p, Some(src), "sacrifice: indestructible", Some(8.0), Some(t))? {
        return Ok(true);
    }
    if live(g, t) {
        super::partials::eot_kw(g, t, "indestructible");
        g.perm_mut(t).plus += counters;
        for e in worn {
            attach(g, e, t);
        }
        crate::glog!(g, "    {} gains indestructible; Zack's counters and Equipment move to it", g.perm(t).name);
    }
    Ok(true)
}

// ------------------------------------------------------------------ Unfinished Business
/// Unfinished Business: return target creature card from your graveyard to the battlefield, then return up to two
/// target Aura and/or Equipment cards from your graveyard to the battlefield attached to that creature
fn unfinished_business(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &crate::state::Ctx) -> Res<crate::sym::Sym> {
    let cre: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    let attach_value = |g: &Game, cd: CardId| -> f64 {
        g.player(p)
            .gy
            .iter()
            .filter(|&&e| is_equipment_card(g, e))
            .map(|&e| crate::engine::tutors::card_worth(g, p, e, true))
            .sum::<f64>()
            .min(20.0)
            + if &*g.db.get(cd).name == CLOUD { 30.0 } else { 0.0 }
            + g.db.get(cd).bomb as f64
            + g.db.get(cd).cmc as f64
    };
    let Some(cd) = max_by(&cre, |c| attach_value(g, c)) else { return Ok(intern("gy")) };
    let i = g.player(p).gy.iter().position(|&x| x == cd).unwrap();
    g.player_mut(p).gy.remove(i);
    let m = enter(g, p, cd, Enter::default())?;
    // up to two Auras and/or Equipment, the most useful first
    let mut cands: Vec<CardId> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&e| {
            is_equipment_card(g, e)
                || (g.db.get(e).has_subtype("aura")
                    && g.registry.get(e).and_then(|i| i.aura).is_some_and(|a| a.target == "own"))
        })
        .collect();
    cands.sort_by(|&a, &b| {
        crate::engine::tutors::card_worth(g, p, b, true)
            .partial_cmp(&crate::engine::tutors::card_worth(g, p, a, true))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for e in cands.into_iter().take(2) {
        if !live(g, m) {
            break;
        }
        let Some(i) = g.player(p).gy.iter().position(|&x| x == e) else { continue };
        g.player_mut(p).gy.remove(i);
        if is_equipment_card(g, e) {
            let em = enter(g, p, e, Enter { sick: false, ..Default::default() })?;
            attach(g, em, m);
        } else {
            g.attach_to = Some(m);
            let r = enter(g, p, e, Enter::default());
            g.attach_to = None;
            r?;
        }
    }
    Ok(intern("gy"))
}

// ------------------------------------------------------------------ Registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, CLOUD)?.etb = Some(cloud_etb);
    r.card(db, "Hero's Blade")?.etb = Some(blade_etb);
    let c = r.card(db, "Armory Automaton")?;
    c.etb = Some(armory_etb);
    c.attack = Some(armory_attack);
    r.card(db, "Beatrix, Loyal General")?.combat_start = Some(beatrix_combat);
    r.card(db, "Puresteel Paladin")?.etb = Some(puresteel_etb);
    let c = r.card(db, "Magitek Scythe")?;
    c.etb = Some(scythe_etb);
    c.prio = Some(body_prio);
    let c = r.card(db, "Squire's Lightblade")?;
    c.etb = Some(lightblade_etb);
    c.prio = Some(body_prio);
    r.card(db, "White Mage's Staff")?.etb = Some(staff_etb);
    let c = r.card(db, "Cid, Freeflier Pilot")?;
    c.cost = Some(cid_cost);
    c.grant_kw = Some(cid_flying);
    c.options = Some(cid_options);
    r.card(db, "Helitrooper")?.attack = Some(helitrooper_attack);
    r.card(db, "Kor Outfitter")?.etb = Some(outfitter_etb);
    r.card(db, "Brass Squire")?.options = Some(squire_options);
    r.card(db, "Conformer Shuriken")?.attack = Some(shuriken_attack);
    r.card(db, "Explorer's Scope")?.attack = Some(scope_attack);
    let c = r.card(db, "Magnetic Snuffler")?;
    c.etb = Some(snuffler_etb);
    c.sacrifice = Some(snuffler_sacrifice);
    let c = r.card(db, "Jocasta, Automaton Avenger")?;
    c.combat_damage = Some(jocasta_damage);
    let c = r.card(db, "Zack Fair")?;
    c.as_enters = Some(zack_enters);
    r.card(db, "Unfinished Business")?.resolve = Some(unfinished_business);
    Ok(())
}

// ------------------------------------------------------------------ the AI plan
fn creatures_out(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| live(g, m) && g.is_creature(m)).count()
}

fn equipment_cards_in_hand(g: &Game, p: PlayerId) -> usize {
    g.player(p).hand.iter().filter(|&&c| is_equipment_card(g, c)).count()
}

/// Cloud is out of play and could be cast now or later: in the command zone, or in hand
fn cloud_waiting(g: &Game, p: PlayerId) -> bool {
    let pl = g.player(p);
    cloud_of(g, p).is_none() && (pl.cmd_in_zone || pl.hand.contains(&pl.cmd))
}

/// the Kaldra pieces already out or in hand
fn kaldra_count(g: &Game, p: PlayerId) -> usize {
    ["Sword of Kaldra", "Shield of Kaldra", "Helm of Kaldra"]
        .iter()
        .filter(|n| g.player(p).perms.iter().any(|&m| live(g, m) && name_of(g, m) == **n) || in_hand(g, p, n))
        .count()
}

/// cloud's cast priority: Cloud on turn 2 (97, ahead of anything else he'd do); Cid and Puresteel Paladin first
/// when Equipment waits in the hand; the Equipment by what it gives Cloud (the size or evasion pieces, the doubled
/// triggers, then the rest); a Hero's Blade waits for Cloud to be recast; Zack Fair, the protection instants and
/// Unfinished Business as the plan needs them
pub fn cloud_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let pl = g.player(p);
    if c == pl.cmd {
        // well above anything else, so the sampled choice is Cloud (not a Page) on turn 2
        return if pl.turns >= 2 { (130 - 3 * pl.tax as i32).max(70) } else { 0 };
    }
    let name = card_name(g, c);
    let cloud_out = cloud_of(g, p).is_some();
    let eq_hand = equipment_cards_in_hand(g, p);
    let eq_out = my_equipment(g, p).len();
    match name {
        // the Equipment
        "Wrecking Ball Arm" => 78,
        "Sword of Kaldra" => 76,
        "Mask of Memory" => 72,
        "Conformer Shuriken" => 68,
        "Hulkbuster Armor" => 64,
        "Medic's Kitesail" => 62,
        "Veteran's Powerblade" => 58,
        "Explorer's Scope" => 55,
        "Trailblazer's Boots" => 50,
        "Helm of Kaldra" => {
            if kaldra_count(g, p) >= 2 {
                70
            } else {
                56
            }
        }
        "Shield of Kaldra" => {
            if cloud_out && crate::ai::brain::removal_risk(g, p) >= 0.35 {
                80
            } else {
                58
            }
        }
        "Colossus Hammer" => {
            if eq_out >= 1 {
                52
            } else {
                36
            }
        }
        // attaches itself when a legend enters: best with Cloud still to come
        "Hero's Blade" => {
            if cloud_waiting(g, p) && crate::engine::mana::total_mana(g, p, false) >= 4 {
                84
            } else {
                58
            }
        }
        // the Equipment that attach as they enter need a body
        "Magitek Scythe" | "Squire's Lightblade" => body_prio(g, p, c),
        "White Mage's Staff" => 66, // brings its own Hero
        "Nim Deathmantle" => 48,
        // the creatures that work the Equipment
        "Cid, Freeflier Pilot" => {
            if eq_hand > 0 {
                82
            } else {
                56
            }
        }
        "Puresteel Paladin" => {
            if eq_hand > 0 {
                81
            } else {
                58
            }
        }
        "Armory Automaton" => {
            if eq_out > 0 {
                64
            } else {
                50
            }
        }
        "Beatrix, Loyal General" => {
            if eq_out > 0 {
                70
            } else {
                55
            }
        }
        "Helitrooper" => 56,
        "Bastion Protector" => {
            if cloud_out {
                74
            } else {
                52
            }
        }
        "Zack Fair" => {
            if cloud_out {
                66
            } else {
                54
            }
        }
        "Magnetic Snuffler" => {
            if pl.gy.iter().any(|&x| is_equipment_card(g, x)) {
                70
            } else {
                44
            }
        }
        "Jocasta, Automaton Avenger" => 52,
        // the catch-up ramp: worth casting once an opponent has more lands
        "Gift of Estates" => {
            if g.opps(p).any(|q| g.player(q).lands.len() > pl.lands.len()) {
                64
            } else {
                0
            }
        }
        "Loyal Warhound" | "Oreskos Explorer" => {
            if g.opps(p).any(|q| g.player(q).lands.len() > pl.lands.len()) {
                60
            } else {
                36
            }
        }
        "Archaeomancer's Map" => 58,
        "Keeper of the Accord" => 56,
        // the creatures that move Equipment
        "Kor Outfitter" => {
            if eq_out > 0 && cloud_out {
                66
            } else {
                46
            }
        }
        "Brass Squire" => {
            if eq_out > 0 {
                60
            } else {
                44
            }
        }
        "Auriok Steelshaper" => {
            if eq_out > 0 {
                58
            } else {
                46
            }
        }
        "Unfinished Business" => {
            let best = pl.gy.iter().any(|&x| card_name(g, x) == CLOUD);
            let any = pl.gy.iter().any(|&x| g.db.get(x).creature && g.db.get(x).bomb >= 3);
            if best {
                80
            } else if any {
                60
            } else {
                0
            }
        }
        _ => crate::ai::pool::generic_prio(g, p, c),
    }
}

/// the Equipment that attach as they enter: only with a creature to wear them (registered as the card's own `prio`,
/// which also keeps the compiled value from casting them without one)
fn body_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if creatures_out(g, p) > 0 { 66 } else { 0 }
}

/// worth saving when removal is aimed at it: Cloud and the commander above everything, then a creature wearing
/// Equipment
fn worth_saving(g: &Game, m: PermId) -> bool {
    if !g.is_creature(m) {
        return false;
    }
    name_of(g, m) == CLOUD || g.perm(m).is_cmd || (!equipment_on(g, m).is_empty() && pval(g, m) >= 4.0)
}

/// cast an instant from the hand to answer removal; true if it resolved (the caller applies its effect)
fn cast_instant(g: &mut Game, p: PlayerId, c: CardId) -> Res<bool> {
    let (gn, pips) = (g.db.get(c).generic, g.db.get(c).pips.to_string());
    if !g.player(p).hand.contains(&c)
        || !can_pay(g, p, gn, &pips, false)
        || !crate::engine::cast::castable(g, p, c, "hand")
    {
        return Ok(false);
    }
    crate::ai::decks::pay_card(g, p, c, 0)
}

/// a flash Aura cast on m in response to removal; it only counts as a save if m ends up out of reach (`proof`)
fn cast_aura(g: &mut Game, p: PlayerId, c: CardId, m: PermId, proof: &dyn Fn(&Game) -> bool) -> Res<bool> {
    let (gn, pips) = (g.db.get(c).generic, g.db.get(c).pips.to_string());
    if !g.player(p).hand.contains(&c)
        || !can_pay(g, p, gn, &pips, false)
        || !crate::engine::cast::castable(g, p, c, "hand")
    {
        return Ok(false);
    }
    crate::ai::decks::remove_from_hand(g, p, c);
    pay(g, p, gn, &pips, false)?;
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    crate::glog!(g, "  {} casts {} -> {}", g.player(p).name, card_name(g, c), g.perm(m).name);
    crate::engine::cast::on_cast(g, p, c)?;
    if !crate::engine::stack::counter_window(g, p, c, 6.0, vec![])? {
        g.player_mut(p).gy.push(c);
        return Ok(false);
    }
    g.attach_to = Some(m);
    let r = enter(g, p, c, Enter { was_cast: true, ..Default::default() });
    g.attach_to = None;
    r?;
    Ok(proof(g))
}

/// removal that destroys (or deals damage): indestructible answers it
fn destroyish(kind: &str) -> bool {
    kind == "destroy" || kind.starts_with("dmg")
}

/// cloud's responses to removal aimed at its permanents: Cloud (and a creature wearing Equipment) is saved by, in
/// order: Shardmage's Rescue (hexproof the turn it enters, any kind of targeted removal), Take Up the Shield and Zack
/// Fair's sacrifice (indestructible, against destroy and damage), Cho-Manno's Blessing (protection from the spell's
/// colours), then the whole-board Unbreakable Formation, Rootborn Defenses and Ultimate Magic: Holy. Bastion
/// Protector and Shield of Kaldra are static.
pub fn cloud_protect(
    g: &mut Game,
    p: PlayerId,
    m: PermId,
    kind: &str,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    if actor.is_none() || actor == Some(p) || !live(g, m) || owner(g, m) != p || !worth_saving(g, m) {
        return Ok(false);
    }
    if matches!(kind, "edict" | "sacrifice") {
        return Ok(false);
    }
    let targeted = kind != "wipe";
    let hand = |g: &Game, n: &str| g.player(p).hand.iter().copied().find(|&c| card_name(g, c) == n);
    // Shardmage's Rescue: hexproof, so the removal has no target left
    if targeted && let Some(c) = hand(g, "Shardmage's Rescue") {
        let proof = |g: &Game| crate::engine::values::untargetable(g, m);
        if cast_aura(g, p, c, m, &proof)? {
            return Ok(true);
        }
    }
    if destroyish(kind) {
        if let Some(c) = hand(g, "Take Up the Shield")
            && cast_instant(g, p, c)?
        {
            g.perm_mut(m).plus += 1;
            super::partials::eot_kw(g, m, "lifelink");
            super::partials::eot_kw(g, m, "indestructible");
            return Ok(true);
        }
        if let Some(z) = creature_named(g, p, "Zack Fair")
            && z != m
            && zack_sacrifice(g, z, p, m.0 as i64)?
        {
            return Ok(true);
        }
    }
    // Cho-Manno's Blessing: protection from the colours of the spell that aims at m
    if targeted
        && let Some(sp) = spell
        && let Some(c) = hand(g, "Cho-Manno's Blessing")
    {
        let cols = g.db.get(sp).pips.to_string();
        if !cols.is_empty() {
            let source = crate::cards::Colors::from_letters(&cols);
            let proof = |g: &Game| crate::engine::values::protected_from(g, m, source);
            if cast_aura(g, p, c, m, &proof)? {
                return Ok(true);
            }
        }
    }
    if destroyish(kind) || kind == "wipe" {
        for n in ["Unbreakable Formation", "Rootborn Defenses", "Ultimate Magic: Holy"] {
            if let Some(c) = hand(g, n)
                && cast_instant(g, p, c)?
            {
                let main =
                    g.active == Some(p) && matches!(g.step, crate::state::Step::Main1 | crate::state::Step::Main2);
                for x in g.player(p).perms.clone() {
                    if live(g, x) && (n == "Ultimate Magic: Holy" || g.is_creature(x)) {
                        super::partials::eot_kw(g, x, "indestructible");
                        if n == "Unbreakable Formation" && main && g.is_creature(x) {
                            g.perm_mut(x).plus += 1;
                            super::partials::eot_kw(g, x, "vigilance");
                        }
                    }
                }
                return Ok(true);
            }
        }
    }
    // the generic one-card answers the outside decks use (Gods Willing, Benevolent Bodyguard, Selfless Spirit, ...)
    crate::ai::pool::protect(g, p, m, crate::sym::intern(kind), actor, spell)
}

/// cloud's own plays (brain.special_options): Cid's regrow and Zack's sacrifice are card options and the equip
/// moves are the generic equip options (`dsl::equip_options`, with `holder_bonus`). Nothing else yet.
pub fn cloud_special(_g: &mut Game, _p: PlayerId, _post: bool) -> Res<Vec<Opt>> {
    Ok(vec![])
}

/// what entering again is worth for Cloud (read by the blink choices in `common::blink_value` and
/// `mine::etb_value`): his search finds an Equipment, but he lets go of what he wears. 0 for anything else.
pub fn reenter_value(g: &Game, p: PlayerId, m: PermId) -> f64 {
    if name_of(g, m) != CLOUD || g.perm(m).phased {
        return 0.0;
    }
    let found = cloud_pick(g, p).map_or(0.0, |c| crate::engine::tutors::card_worth(g, p, c, false));
    let worn: f64 = equipment_on(g, m)
        .iter()
        .map(|&e| 0.5 * g.perm(e).cd.map_or(0.0, |c| crate::engine::tutors::card_worth(g, p, c, false)))
        .sum();
    (found - worn).max(0.0)
}

/// who attacks: the generic outside-deck rule (a creature a good blocker would eat stays home; evasive ones, lethal
/// swings and attacks with more attackers than good blockers go ahead), and Cloud himself only when he wears
/// Equipment or no creature can block him: he is the engine, and a 2/1 that dies is a recast at a higher tax
pub fn attack_filter(g: &mut Game, p: PlayerId, atk: Vec<PermId>, d: PlayerId) -> Vec<PermId> {
    let mut out = crate::ai::pool::attack_filter(g, p, atk, d);
    let Some(cl) = cloud_of(g, p) else { return out };
    if !out.contains(&cl) || !equipment_on(g, cl).is_empty() || crate::engine::values::indestructible(g, cl) {
        return out;
    }
    let total: i32 = out.iter().map(|&m| epow(g, m)).sum();
    if total >= g.player(d).life {
        return out; // lethal: everyone goes
    }
    let blocked =
        g.player(d).perms.iter().any(|&b| {
            live(g, b) && g.is_creature(b) && !g.perm(b).tapped && crate::engine::combat::can_block(g, b, cl)
        });
    if blocked {
        out.retain(|&m| m != cl);
        crate::glog!(g, "      [{} keeps Cloud home: nothing on him yet]", g.player(p).name);
    }
    out
}

/// look-ahead score bonus for a main-phase option of cloud (added to the rollout average in `search::choose_inner`):
/// the position score can't see what Cloud's search for an Equipment is worth, so casting him gets a flat bonus
pub fn search_bonus(g: &Game, p: PlayerId, o: &Opt) -> f64 {
    match &o.act {
        Some(Action::Cast { .. }) if o.label == CLOUD && cloud_of(g, p).is_none() => 10.0,
        _ => 0.0,
    }
}

/// the plan milestone `cloud`: the first attack by an equipped Cloud (read by `report::plan_turn`)
pub fn plan_milestone(g: &mut Game, p: PlayerId, atk: &[PermId]) {
    if g.player(p).key == KEY && atk.iter().any(|&m| name_of(g, m) == CLOUD && !equipment_on(g, m).is_empty()) {
        let turns = g.player(p).turns;
        g.player_mut(p).milestone.entry("cloud").or_insert(turns);
    }
}

/// rule 903.9b: when Cloud would go to his owner's hand, the owner may leave him there, and casting him from the hand
/// costs no commander tax
pub fn cmd_stays_in_hand(g: &Game, owner: PlayerId) -> bool {
    g.player(owner).key == KEY
}
