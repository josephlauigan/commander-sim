//! The cloud deck (decklists/JD/cloud-midgar-equipment.md): mono-white Equipment around Cloud, Midgar Mercenary. Its
//! card code and AI plan. The brief is rust/agents/archive/cloud-core.md; the verdicts are in data/card_overrides.toml.
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
//!   Arm, Trailblazer's Boots, Magitek Scythe's "must be blocked" in `must_be_blocked`) and Cloud's search order
//!   (`cloud_pick`: the card's own choice, also read by `reenter_value`).
//!
//! The deck's own AI (cast priorities, protection and wipe answers, equip weights, attack and keep rules, the
//! look-ahead bonuses) was deleted on 2026-11-01: it plays the generic policy and `data/plans/cloud.toml`
//! (`audit/phase4/hand-off-design.md`, "Deleted, part 2").

use crate::engine::repl;
use crate::cards::{CardDb, Types};
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::{ability_window, may_window, may_window_card, trigger_window};
use crate::engine::values::{epow, etgh, has_type, pval};
use crate::engine::zones::{Enter, Tokens, draw, enter, make_tokens, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Sacrificed, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;
use crate::sym::intern;

pub const CLOUD: &str = "Cloud, Midgar Mercenary";
const CLOGGED_HAND: usize = 7;
const KEY: &str = "cloud";

/// The Swords the list doesn't run, in the order Cloud's search takes them after Fire and Ice, Hearth and Home and
/// Feast and Famine: War and Peace (its hand damage is doubled), Forge and Frontier (cards and a land), Truth and Justice
/// (counters, proliferate), Light and Shadow, Sinew and Steel, Once and Future (few cheap instants here), Body and Mind
const MORE_SWORDS: [&str; 7] = [
    "Sword of War and Peace",
    "Sword of Forge and Frontier",
    "Sword of Truth and Justice",
    "Sword of Light and Shadow",
    "Sword of Sinew and Steel",
    "Sword of Once and Future",
    "Sword of Body and Mind",
];

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
    if matches!(name_of(g, m), "Balan, Wandering Knight" | "Kor Duelist") {
        s += 3.0; // double strike when equipped: the backup carriers while Cloud is away
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
    n = n.saturating_sub(crate::impls::t3_aurelia::equip_discount(g, p)); // Fighter Class, level 2
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
                if d != p && g.player(d).lands.iter().any(|&l| g.perm(l).on_bf && !g.db.get(g.perm(l).card()).basic) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// the equipped creature loses flying (Colossus Hammer): `chars::loses_keyword`, from the table of named effects
pub fn loses_flying(g: &Game, m: PermId) -> bool {
    crate::engine::chars::loses_keyword(g, m, "flying")
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
    // mana the next turn can spend: the lands out, a land drop, the mana rocks
    let rocks = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| live(g, m) && g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(crate::tag::Tag::Rock)))
        .count();
    let mana = g.player(p).lands.iter().filter(|&&l| g.perm(l).on_bf).count() as u32 + 1 + rocks as u32;
    let (free_equips, shapers) = equip_engine(g, p);
    // cast + equip cost of a card (Cid is in the cast cost; free-equip engines and Steelshapers lower the equip cost)
    let total = |c: CardId| -> u32 {
        let cast = crate::engine::mana::cost_of(g, p, c).0;
        let d = g.db.get(c);
        let eq = d
            .abilities
            .iter()
            .find_map(|a| match a {
                crate::dsl::model::Ability::Static { kind, mana, .. } if kind == "equip_cost" => {
                    Some(mana.unwrap_or(2))
                }
                _ => None,
            })
            .unwrap_or(match &*d.name {
                "Champion's Helm" => 1,
                _ => 2, // Sword of Feast and Famine
            });
        cast + if free_equips { 0 } else { eq.saturating_sub(shapers) }
    };
    if free_equips {
        // equips are (nearly) free: the biggest payoff the mana can cast
        let big = ["Sword of Kaldra", "Helm of Kaldra", "Shield of Kaldra", "Wrecking Ball Arm", "Colossus Hammer"];
        let cast = |c: CardId| cast_cost(g, p, c);
        if let Some(c) =
            big.iter().filter(|n| !have(n)).filter_map(|n| g.db.id(n)).find(|c| lib.contains(c) && cast(*c) <= mana)
        {
            return Some(c);
        }
    }
    let exposed = crate::ai::brain::removal_risk(g, p) >= 0.3;
    // Sword of Feast and Famine untaps the lands (the mana to equip or cast the next piece) and its discard is cheap
    // with a clogged hand; it also brings a different protection pair, so it comes before Hearth and Home once a Sword
    // is already out or held, and first of all with a truly clogged hand (7+ cards)
    let sword_held = ["Sword of Fire and Ice", "Sword of Hearth and Home"].iter().chain(&MORE_SWORDS).any(|n| have(n));
    let clogged = g.player(p).hand.len() >= CLOGGED_HAND; // a typical turn-2 hand is 5-6 cards: not clogged
    let mut order: Vec<&str> = if clogged {
        vec!["Sword of Feast and Famine", "Sword of Fire and Ice", "Sword of Hearth and Home"]
    } else if sword_held {
        vec!["Sword of Fire and Ice", "Sword of Feast and Famine", "Sword of Hearth and Home"]
    } else {
        vec!["Sword of Fire and Ice", "Sword of Hearth and Home", "Sword of Feast and Famine"]
    };
    order.extend(MORE_SWORDS); // the other Swords, after the deck's three (none is in the list today)
    if exposed && held >= 1 {
        order.insert(0, "Champion's Helm"); // hexproof for the legend
    } else {
        order.push("Champion's Helm");
    }
    if held == 0 {
        order.extend(["Wrecking Ball Arm", "Hero's Blade", "Sword of Kaldra", "Mask of Memory"]);
    } else {
        order.extend(["Mask of Memory", "Sword of Kaldra", "Wrecking Ball Arm", "Hero's Blade"]);
    }
    order.extend(["Colossus Hammer", "Hulkbuster Armor", "Shield of Kaldra", "Helm of Kaldra"]);
    // the first piece that can be cast next turn and equipped within a turn after (cast + equip <= mana + 2), in order
    let ready =
        order.iter().filter(|n| !have(n)).filter_map(|n| g.db.id(n)).find(|c| lib.contains(c) && total(*c) <= mana + 2);
    if let Some(c) = ready {
        return Some(c);
    }
    const REST: [&str; 14] = [
        "Conformer Shuriken",
        "Medic's Kitesail",
        "Magitek Scythe",
        "White Mage's Staff",
        "Veteran's Powerblade",
        "Explorer's Scope",
        "Trailblazer's Boots",
        "Squire's Lightblade",
        "Nim Deathmantle",
        "Hero's Heirloom",
        "Hulkbuster Armor",
        "Wrecking Ball Arm",
        "Sword of Kaldra",
        "Mask of Memory",
    ];
    if let Some(c) =
        REST.iter().filter(|n| !have(n)).filter_map(|n| g.db.id(n)).find(|c| lib.contains(c) && total(*c) <= mana + 2)
    {
        return Some(c);
    }
    // nothing fits: the cheapest piece to cast and equip
    let mut left: Vec<CardId> = lib.iter().copied().filter(|&c| !have(card_name(g, c))).collect();
    left.sort_by_key(|&c| total(c));
    left.first().copied()
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
    super::t1::tutor_named(g, o, &|g, c| is_equipment_card(g, c) && (human || Some(c) == pick), 1, "hand", CLOUD)?;
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
    if (better || crate::human::is_human(g, o))
        && may_window(g, o, Some(src), "attach to the legendary creature", Some(6.0))?
    {
        attach(g, src, m);
    }
    Ok(())
}

/// Equipment worth taking onto `to`: loose, or on a creature that is a worse holder
pub(crate) fn pullable(g: &Game, p: PlayerId, to: PermId) -> Vec<PermId> {
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
    if es.is_empty() || !may_window(g, o, Some(src), "attach Equipment to itself", Some(5.0))? {
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
    if es.is_empty() || !may_window(g, p, Some(src), "attach Equipment", Some(5.0))? {
        return Ok(vec![]);
    }
    for e in es {
        attach(g, e, to);
    }
    Ok(vec![])
}

// ------------------------------------------------------------------ Kor Outfitter, Brass Squire, Auriok Steelshaper
const STEELSHAPER: &str = "Auriok Steelshaper";

/// the Equipment p most wants moved onto `to`: a loose one first
fn best_pull(g: &Game, p: PlayerId, to: PermId) -> Option<PermId> {
    let es: Vec<PermId> = pullable(g, p, to).into_iter().filter(|&e| g.perm(e).attached != Some(to)).collect();
    max_by(&es, |e| {
        let loose = g.perm(e).attached.is_none_or(|t| !live(g, t));
        if loose { 10.0 } else { 0.0 }
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
    if best_pull(g, o, to).is_none() || !may_window(g, o, Some(src), "attach Equipment", Some(5.0))? {
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

// ------------------------------------------------------------------ Puresteel Paladin
/// Puresteel Paladin: whenever an Equipment you control enters, you may draw a card
fn puresteel_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if owner(g, m) == o && live(g, src) && is_equipment(g, m) && may_window(g, o, Some(src), "draw a card", Some(4.0))?
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
    if may_window(g, o, Some(src), "attach to a creature", Some(5.0))? && attach(g, src, to) {
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

/// Job select (White Mage's Staff, Dragoon's Lance, Samurai's Katana): when it enters, create a 1/1 colorless Hero
/// creature token, then attach this to it
pub fn staff_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
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
        repl::put_plus(g, me, diff);
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
        repl::put_plus(g, src, 1);
    }
    Ok(())
}

/// Jocasta: whenever your commander deals combat damage to a player, put a +1/+1 counter on Jocasta
fn jocasta_damage(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, dmg: i32) -> Res {
    if p == owner(g, src) && dmg > 0 && live(g, src) && g.perm(a).is_cmd && owner(g, a) == p {
        repl::put_plus(g, src, 1);
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
    if !may_window_card(
        g,
        p,
        c,
        "return Jocasta attacking",
        "Jocasta, Automaton Avenger: return her to the battlefield tapped and attacking?",
    )? {
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
    repl::put_plus(g, m, 1);
    Ok(())
}

/// {1}, Sacrifice Zack Fair: target creature you control gains indestructible until end of turn; put Zack's counters
/// on that creature and attach an Equipment that was attached to Zack to that creature. No AI path activates it
/// since the hand AI's deletion (`cloud_protect` did); the rules are tested in `tests/cards_cloud.rs`.
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
        repl::put_plus(g, t, counters);
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
    let c = r.card(db, NARSIL)?;
    c.attack = Some(narsil_attack);
    c.prio = Some(narsil_prio);
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
    r.card(db, "Open the Armory")?.resolve = Some(open_the_armory);
    register_ds(r, db)?;
    Ok(())
}

// ------------------------------------------------------------------ the AI plan
fn creatures_out(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| live(g, m) && g.is_creature(m)).count()
}

/// the Equipment that attach as they enter: only with a creature to wear them (registered as the card's own `prio`,
/// which also keeps the compiled value from casting them without one)
fn body_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if creatures_out(g, p) > 0 { 66 } else { 0 }
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
    g.player(owner).key == KEY || g.player(owner).key == crate::impls::garland::KEY
}

/// what makes equipping cheap for p right now: (equips are about free, Auriok Steelshapers out). Puresteel Paladin
/// with metalcraft (counting the piece about to be cast), Ardenn on the battlefield (re-attaches every combat) and
/// Kor Outfitter in hand (attaches on entering) make equips free.
fn equip_engine(g: &Game, p: PlayerId) -> (bool, u32) {
    let artifacts = g.player(p).perms.iter().filter(|&&m| live(g, m) && is_artifact(g, m)).count();
    let puresteel = creature_named(g, p, "Puresteel Paladin").is_some() && artifacts + 1 >= 3;
    let ardenn = creature_named(g, p, "Ardenn, Intrepid Archaeologist").is_some();
    let outfitter = in_hand(g, p, "Kor Outfitter");
    let shapers = g.player(p).perms.iter().filter(|&&s| live(g, s) && name_of(g, s) == STEELSHAPER).count() as u32;
    (puresteel || ardenn || outfitter, shapers)
}

/// mana value of casting c now (generic plus coloured pips, after cost changes)
fn cast_cost(g: &Game, p: PlayerId, c: CardId) -> u32 {
    let (n, pips) = crate::engine::mana::cost_of(g, p, c);
    n + pips.chars().filter(|ch| ch.is_ascii_alphabetic()).count() as u32
}

// ------------------------------------------------------------------ double strike package (cloud-ds)
const HERITAGE: &str = "Duelist's Heritage";
const WILL: &str = "Akroma's Will";

fn has_trigger_sword(g: &Game, p: PlayerId) -> bool {
    cloud_of(g, p).is_some_and(|cl| {
        equipment_on(g, cl).iter().any(|&e| {
            matches!(name_of(g, e), "Sword of Fire and Ice" | "Sword of Feast and Famine" | "Sword of Hearth and Home")
        })
    })
}

/// Duelist's Heritage: "Whenever one or more creatures attack, you may have target attacking creature gain double
/// strike until end of turn." The AI always takes it, on the attacker that gains most: Cloud, then the biggest.
/// (Any attack of its controller's; fired once per attack declaration.)
fn heritage_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner != p || !live(g, src) {
        return Ok(vec![]);
    }
    let pool: Vec<PermId> = atk
        .iter()
        .copied()
        .filter(|&m| {
            live(g, m) && !crate::engine::combat::double_strike(g, m) && !crate::engine::values::untargetable(g, m)
        })
        .collect();
    let score = |g: &Game, m: PermId| {
        let mut s = epow(g, m) as f64;
        if name_of(g, m) == CLOUD {
            s += 20.0 + 10.0 * (!equipment_on(g, m).is_empty()) as i32 as f64; // the doubled Sword triggers
        }
        s
    };
    let Some(t) = max_by(&pool, |m| score(g, m)) else { return Ok(vec![]) };
    if trigger_window(g, p, Some(src), "double strike", None)? && live(g, t) {
        crate::impls::partials::eot_kw(g, t, intern("double strike"));
        crate::glog!(g, "    {} gains double strike (Duelist's Heritage)", g.perm(t).name);
    }
    Ok(vec![])
}

/// Akroma's Will ({3}{W}, instant): after blockers, on the attacker's turn, when double strike adds real damage (Cloud
/// wearing a Sword, or at least 4 extra points from unblocked attackers). Mode one always (flying, vigilance, double
/// strike); mode two too while a commander is on the battlefield (lifelink, indestructible, protection from each
/// color). Not modeled: casting it on an opponent's turn for the protection mode alone.
fn will_blocks(
    g: &mut Game,
    c: CardId,
    p: PlayerId,
    atk: &[PermId],
    _d: PlayerId,
    assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    if card_name(g, c) != WILL || !g.player(p).hand.contains(&c) || !can_pay(g, p, 3, "W", false) {
        return Ok(());
    }
    let open: Vec<PermId> = atk
        .iter()
        .copied()
        .filter(|&m| live(g, m) && g.is_creature(m) && !assign.iter().any(|x| x.0 == m))
        .filter(|&m| !crate::engine::combat::double_strike(g, m))
        .collect();
    let extra: i32 = open.iter().map(|&m| epow(g, m)).sum();
    let cloud_swung = open.iter().any(|&m| name_of(g, m) == CLOUD && has_trigger_sword(g, p));
    if !(cloud_swung || extra >= 4) {
        return Ok(());
    }
    if !crate::engine::cast::castable(g, p, c, "hand") || !pay(g, p, 3, "W", false)? {
        return Ok(());
    }
    crate::engine::cast::cast_card(g, p, c, "hand", crate::state::Ctx::default())?;
    Ok(())
}

fn will_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &crate::state::Ctx) -> Res<crate::sym::Sym> {
    let both = g.player(p).perms.iter().any(|&m| live(g, m) && g.perm(m).is_cmd);
    for m in g.player(p).perms.clone() {
        if !live(g, m) || !g.is_creature(m) {
            continue;
        }
        for k in ["flying", "vigilance", "double strike"] {
            crate::impls::partials::eot_kw(g, m, intern(k));
        }
        if both {
            for k in ["lifelink", "indestructible", "pro_colors"] {
                crate::impls::partials::eot_kw(g, m, intern(k));
            }
        }
    }
    crate::glog!(
        g,
        "    {}'s creatures gain double strike{}",
        g.player(p).name,
        if both { " and protection" } else { "" }
    );
    Ok(intern("gy"))
}

/// cast priority of the package: Heritage once Cloud is out, Fireshrieker as Equipment (more with a Sword to fire
/// twice), the Will never in a main phase (it is cast from `will_blocks`)
fn ds_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    match card_name(g, c) {
        WILL => 0,
        HERITAGE => {
            if cloud_of(g, p).is_some() {
                66
            } else {
                40
            }
        }
        _ => {
            if has_trigger_sword(g, p) {
                68
            } else {
                54
            }
        }
    }
}

pub fn register_ds(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, HERITAGE)?;
    c.attack = Some(heritage_attack);
    c.prio = Some(ds_prio);
    crate::impls::partials::at_once(c, crate::hooks::Event::Attack, false);
    let c = r.card(db, WILL)?;
    c.resolve = Some(will_resolve);
    c.hand_blocks = Some(will_blocks);
    c.prio = Some(ds_prio);
    r.card(db, "Fireshrieker")?.prio = Some(ds_prio);
    Ok(())
}

// ------------------------------------------------------------------ audit fixes (ai-audit cv)
/// Open the Armory: search for an Aura or Equipment card (the DSL row had no filter and fetched Sol Ring, Cid ...).
fn open_the_armory(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &crate::state::Ctx) -> Res<crate::sym::Sym> {
    let ok = |g: &Game, c: CardId| g.db.get(c).has_subtype("equipment") || g.db.get(c).has_subtype("aura");
    super::t1::tutor_named(g, p, &ok, 1, "hand", "Open the Armory")?;
    Ok(intern("gy"))
}

// ------------------------------------------------------------------ Andúril, Narsil Reforged
const NARSIL: &str = "Andúril, Narsil Reforged";

/// Ascend: ten or more permanents (lands count; the engine's one rule for the city's blessing) give the city's
/// blessing for the rest of the game, remembered in the player's stats once seen
fn city_blessing(g: &mut Game, p: PlayerId) -> bool {
    let key = intern("city_blessing");
    if g.player(p).stats.get(&key).is_some_and(|&n| n > 0) {
        return true;
    }
    if g.player(p).perms.len() + g.player(p).lands.len() >= 10 {
        g.player_mut(p).stat(key, 1);
        crate::glog!(g, "    {} has the city's blessing", g.player(p).name);
        return true;
    }
    false
}

/// **Andúril, Narsil Reforged**: "Ascend. Whenever equipped creature attacks, put a +1/+1 counter on each creature you
/// control. If you have the city's blessing, put two +1/+1 counters on each creature you control instead. Equip {3}."
/// (Cloud's doubling queues this trigger twice while the Equipment is on him.)
fn narsil_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    let Some(host) = g.perm(src).attached else { return Ok(vec![]) };
    if g.perm(src).owner != p || !atk.contains(&host) || !live(g, host) {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "+1/+1 counters on each creature", Some(3.0))? {
        return Ok(vec![]);
    }
    let n = if city_blessing(g, p) { 2 } else { 1 };
    let mine: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect();
    for m in mine {
        repl::put_plus(g, m, n);
    }
    Ok(vec![])
}

fn narsil_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    // two mana for a counter on every creature each swing: early, and better the wider the team (and Cloud's doubling)
    let team = g.player(p).perms.iter().filter(|&&m| live(g, m) && g.is_creature(m)).count();
    if cloud_of(g, p).is_some() || team >= 2 { 79 } else { 66 }
}
