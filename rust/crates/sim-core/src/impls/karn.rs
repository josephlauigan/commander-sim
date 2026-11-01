//! The karn deck (key `karn`, JD's colourless Urza and Mishra artifacts deck; decklists/JD/karn-colorless-artifacts.md):
//! the commander, the restricted-mana lands, the Tron lands, the creature lands and the look-ahead bonus.
//!
//! - **Karn, Legacy Reforged** (Full): power and toughness each equal the greatest mana value among the artifacts you
//!   control (`karn_pt`, a bonus on top of the printed 1/1); at the beginning of your upkeep, {C} for each artifact you
//!   control goes into the pool as `pool::Key::ART` (artifact spells and abilities only; kept all turn as steps end,
//!   emptied when the next turn starts, `engine::turn`).
//! - **Restricted mana** is one mechanism: `Restr::NotNonartifact` in the pool (the AI engine, `engine::pool`) and
//!   `ManaPool::art` (practice mode, `human::mana`). It pays artifact spells and anything that isn't a spell (abilities,
//!   taxes); a nonartifact spell never sees it (`engine::mana::restr_ok`, the spell being paid for is `Game::pay_for`).
//!   The Mightstone and Weakstone's {C}{C} and the Powerstone token's {C} use the same test (tag `pstone`). Mishra's
//!   Workshop is stricter (artifact spells only), as printed.
//! - **Tron** (Urza's Mine / Power-Plant / Tower: 2 / 2 / 3 with all three, else 1) and Urza's Workshop (metalcraft:
//!   {C} for each Urza's land with three artifacts, else {C}): `dyn_mana_land` slots.
//! - **Mishra's Factory / Foundry** are creature lands (`lands::animate`, a token standing in for the land) that are
//!   also artifacts while animated; the AI animates one to attack when the way is open, and one to block a dangerous
//!   attack (`block_response`, from `brain::attack_response`; any deck since generic-paths, 2026-11-01).
//! - **The plan** (`karn_prio`, the Bauble hold, the Tron land choice, the land searches) was the deck's hand AI,
//!   deleted on 2026-11-01 (`audit/phase4/hand-off-design.md`, "Deleted"). `search_bonus` stays: the look-ahead
//!   (`ai/search.rs`, out of that change's scope) still names it under a "karn" key it never sees now.

use crate::cards::{CardDb, Types};
use crate::engine::mana::total_mana;
use crate::engine::removal::apply_removal;
use crate::engine::stack::trigger_window;
use crate::engine::values::{epow, etgh, pval, untargetable};
use crate::engine::zones::draw;
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;

pub const KARN: &str = "Karn, Legacy Reforged";
fn live(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased
}

fn land_name(g: &Game, l: PermId) -> &str {
    &g.db.get(g.perm(l).card()).name
}

fn is_art_card(g: &Game, c: CardId) -> bool {
    g.db.get(c).types.has(Types::ARTIFACT)
}

/// p's Karn on the battlefield, if any
pub fn karn_of(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == KARN))
}

/// the artifacts p controls: artifact permanents (cards and artifact tokens), Treasure / Food / Clue / Blood tokens,
/// artifact lands (Treasure Vault, Darksteel Citadel); an animated Mishra's Factory is one of the permanents
pub fn artifacts(g: &Game, p: PlayerId) -> u32 {
    let lands = g.player(p).lands.iter().filter(|&&l| is_art_card(g, g.perm(l).card())).count() as u32;
    crate::impls::sheoldred::artifact_count(g, p) as u32 + lands
}

/// the greatest mana value among the artifacts p controls
pub fn max_mv(g: &Game, p: PlayerId) -> u32 {
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| live(g, m))
        .filter_map(|&m| g.perm(m).cd.filter(|&c| is_art_card(g, c)))
        .map(|c| g.db.get(c).cmc)
        .max()
        .unwrap_or(0)
}

// ================================================================== Karn, Legacy Reforged
/// Karn's size: each equal to the greatest mana value among your artifacts. The printed 1/1 is the base the engine
/// starts from, so this is the difference.
fn karn_pt(g: &Game, p: PlayerId, _m: PermId) -> (i32, i32) {
    let v = max_mv(g, p) as i32;
    (v - 1, v - 1)
}

fn selfpt_as(g: &mut Game, _p: PlayerId, _m: PermId) -> Res {
    g.selfpt = true; // the engine reads this creature's size from its own rule from here on
    Ok(())
}

/// at the beginning of your upkeep, add {C} for each artifact you control: artifact spells and abilities only, and
/// it stays until the turn ends
fn karn_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != g.perm(src).owner || !live(g, src) {
        return Ok(());
    }
    let n = artifacts(g, p);
    if n == 0
        || !trigger_window(g, p, Some(src), "add {C} for each artifact (artifact spells and abilities)", Some(5.0))?
    {
        return Ok(());
    }
    g.player_mut(p).mana.add_art(n);
    let turns = g.player(p).turns;
    g.player_mut(p).milestone.entry("karn").or_insert(turns); // the plan report: Karn's first mana
    crate::glog!(g, "  {} adds {} for Karn (artifacts only)", g.player(p).name, "{C}".repeat(n as usize));
    Ok(())
}

// ================================================================== The Mightstone and Weakstone
pub const MIGHTSTONE: &str = "The Mightstone and Weakstone";

/// -5/-5 until end of turn on m (a lethal one goes through the removal code)
fn shrink5(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    if !g.perm(m).on_bf {
        return Ok(());
    }
    if etgh(g, m) <= 5 {
        return apply_removal(g, Some(p), m, "shrink5", None);
    }
    let x = g.perm_mut(m);
    x.eot_pt = (x.eot_pt.0 - 5, x.eot_pt.1 - 5);
    Ok(())
}

/// When it enters, choose one: draw two cards; or target creature gets -5/-5 until end of turn. (The {T}: add {C}{C}
/// that can't be spent on nonartifact spells is the `rock` tag with `pstone`: `engine::mana`.) The AI kills the best
/// opposing creature the -5/-5 handles when it is worth a card (a threat of 5 or more), else it draws.
fn mightstone_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || g.perm(src).owner != p {
        return Ok(());
    }
    let victims: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.clone())
        .filter(|&x| live(g, x) && g.is_creature(x) && !untargetable(g, x))
        .collect();
    let human = crate::human::is_human(g, p);
    let best = victims.iter().copied().filter(|&x| etgh(g, x) <= 5).max_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b)));
    if !trigger_window(g, p, Some(src), "draw two cards, or -5/-5 on a creature", Some(5.0))? {
        return Ok(());
    }
    if human {
        let labels = vec!["Draw two cards".to_string(), "Target creature gets -5/-5 until end of turn".to_string()];
        let k = crate::human::choices::choose_one(g, p, "mode", "The Mightstone and Weakstone: choose one", &labels)?;
        if k == 1 {
            let all: Vec<PermId> = g
                .players
                .iter()
                .flat_map(|q| q.perms.clone())
                .filter(|&x| live(g, x) && g.is_creature(x) && !untargetable(g, x))
                .collect();
            if let Some(t) = crate::human::cards::pick_perm(g, p, "target", "-5/-5 on which creature?", &all, None)? {
                return shrink5(g, p, t);
            }
        }
        return draw(g, p, 2, false);
    }
    match best {
        Some(t) if pval(g, t) >= 5.0 => shrink5(g, p, t),
        _ => draw(g, p, 2, false),
    }
}

// ================================================================== Tron and Urza's Workshop
fn has_land(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).lands.iter().any(|&l| land_name(g, l) == name)
}

const TRON: [&str; 3] = ["Urza's Mine", "Urza's Power Plant", "Urza's Tower"];

/// the three Tron lands are all out
pub fn tron_on(g: &Game, p: PlayerId) -> bool {
    TRON.iter().all(|n| has_land(g, p, n))
}

/// Urza's Mine: {C}; {C}{C} instead with a Power-Plant and a Tower
fn mine_mana(g: &Game, p: PlayerId, _l: PermId) -> u32 {
    if has_land(g, p, "Urza's Power Plant") && has_land(g, p, "Urza's Tower") { 2 } else { 1 }
}

/// Urza's Power-Plant: {C}; {C}{C} instead with a Mine and a Tower
fn plant_mana(g: &Game, p: PlayerId, _l: PermId) -> u32 {
    if has_land(g, p, "Urza's Mine") && has_land(g, p, "Urza's Tower") { 2 } else { 1 }
}

/// Urza's Tower: {C}; {C}{C}{C} instead with a Mine and a Power-Plant
fn tower_mana(g: &Game, p: PlayerId, _l: PermId) -> u32 {
    if has_land(g, p, "Urza's Mine") && has_land(g, p, "Urza's Power Plant") { 3 } else { 1 }
}

/// Urza's Workshop: {C}, or with three or more artifacts {C} for each Urza's land you control
fn workshop_mana(g: &Game, p: PlayerId, _l: PermId) -> u32 {
    if artifacts(g, p) >= 3 {
        let n = g.player(p).lands.iter().filter(|&&l| g.db.get(g.perm(l).card()).has_subtype("urza's")).count();
        (n as u32).max(1)
    } else {
        1
    }
}

// ================================================================== Mishra's Factory and Mishra's Foundry
/// (name, the cost to animate)
const MANLANDS: [(&str, u32); 2] = [("Mishra's Factory", 1), ("Mishra's Foundry", 2)];

pub(crate) fn cost_of_land(g: &Game, l: PermId) -> u32 {
    MANLANDS.iter().find(|m| m.0 == land_name(g, l)).map_or(1, |m| m.1)
}

fn opt(utility: f64, label: String, f: crate::hooks::PlanFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Plan { f, arg }) }
}

/// the land becomes a 2/2 Assembly-Worker artifact creature (a token standing in for the land, see `lands::animate`)
pub(crate) fn animate(g: &mut Game, p: PlayerId, l: PermId) -> PermId {
    let m = super::lands::animate(g, p, l, 2, 2, &[], false);
    let x = g.perm_mut(m);
    x.ttypes.push("artifact");
    x.ttypes.push("assembly-worker");
    m
}

/// --- generic-paths (2026-11-01): d is attacked by p: animate untapped Factories and Foundries to block when the
/// attack is dangerous (40% of d's life, or 4+ at 12 life or less), one per attacker that no creature can meet, paid
/// with d's other mana. It was the karn deck's hand AI (`brain::attack_response`), deleted with it; any deck now.
pub fn block_response(g: &mut Game, d: PlayerId, p: PlayerId, atk: &[PermId]) -> Res<bool> {
    let manland = |g: &Game, l: PermId| MANLANDS.iter().any(|m| m.0 == land_name(g, l));
    if crate::human::is_human(g, d) || !g.player(d).lands.iter().any(|&l| manland(g, l)) {
        return Ok(false);
    }
    let incoming: i32 = atk.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).owner == p).map(|&m| epow(g, m)).sum();
    let life = g.player(d).life;
    if (incoming as f64) < life as f64 * 0.4 && !(life <= 12 && incoming >= 4) {
        return Ok(false);
    }
    let mut done = false;
    let mut blockers = g.player(d).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).tapped).count();
    for l in g.player(d).lands.clone() {
        if blockers >= atk.len() {
            break;
        }
        if !MANLANDS.iter().any(|m| m.0 == land_name(g, l)) || g.perm(l).tapped {
            continue;
        }
        let cost = cost_of_land(g, l);
        if !super::lands::pay_without(g, d, l, cost, "")? {
            continue;
        }
        animate(g, d, l);
        blockers += 1;
        done = true;
    }
    Ok(done)
}

/// {1} / {2}: animate to attack when the way is open and mana is to spare (before combat)
fn manland_options(g: &mut Game, l: PermId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let cost = cost_of_land(g, l);
    if post != Some(false) || g.perm(l).tapped || !super::lands::can_pay_without(g, p, l, cost, "") {
        return Ok(vec![]);
    }
    let pl = g.player(p);
    if pl.land_turn == pl.turns as i32 && pl.lands.last() == Some(&l) {
        return Ok(vec![]); // just played: it would be summoning sick
    }
    let spare = total_mana(g, p, false) as i64 - cost as i64 - 1;
    if !super::lands::open_attack(g, p, 2, false) {
        return Ok(vec![]);
    }
    let u = 1.5 + if spare >= 2 { 0.5 } else { -0.8 };
    Ok(vec![opt(u, format!("animate {}", land_name(g, l)), manland_go, l.0 as i64)])
}

fn manland_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = PermId(arg as u32);
    let cost = cost_of_land(g, l);
    if !g.player(p).lands.contains(&l) || g.perm(l).tapped || !super::lands::pay_without(g, p, l, cost, "")? {
        return Ok(false);
    }
    if !crate::engine::stack::ability_window_card(g, p, g.perm(l).card(), "becomes a creature", None, None)? {
        return Ok(true);
    }
    animate(g, p, l);
    Ok(true)
}

// ================================================================== the plan
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    if db.id(KARN).is_some() {
        let c = r.card(db, KARN)?;
        c.self_pt = Some(karn_pt);
        c.as_enters = Some(selfpt_as);
        c.upkeep = Some(karn_upkeep);
    }
    if db.id(MIGHTSTONE).is_some() {
        r.card(db, MIGHTSTONE)?.etb = Some(mightstone_etb);
    }
    if db.id("Urza's Mine").is_some() {
        r.card(db, "Urza's Mine")?.dyn_mana_land = Some(mine_mana);
        r.card(db, "Urza's Power Plant")?.dyn_mana_land = Some(plant_mana);
        r.card(db, "Urza's Tower")?.dyn_mana_land = Some(tower_mana);
        r.card(db, "Urza's Workshop")?.dyn_mana_land = Some(workshop_mana);
    }
    for m in MANLANDS.iter() {
        if db.id(m.0).is_some() {
            r.card(db, m.0)?.land_options = Some(manland_options);
        }
    }
    Ok(())
}

/// look-ahead score bonus for a main-phase option of the karn deck (added to the rollout average in
/// `search::choose_inner`): the position score can't see what Karn's upkeep mana is worth, so casting him gets a flat
/// bonus (as early as possible: the paper plan), and each cheap artifact cast with Karn out (another {C} every
/// upkeep) a smaller one
pub fn search_bonus(g: &Game, p: PlayerId, o: &Opt) -> f64 {
    let Some(Action::Cast { card, .. }) = &o.act else { return 0.0 };
    let d = g.db.get(*card);
    if &*d.name == KARN {
        return 10.0;
    }
    if karn_of(g, p).is_some() && d.types.has(Types::ARTIFACT) && !d.land && d.cmc <= 2 {
        return 1.5;
    }
    0.0
}
