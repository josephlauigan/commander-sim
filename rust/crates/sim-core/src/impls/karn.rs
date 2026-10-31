//! The karn deck (key `karn`, JD's colourless Urza and Mishra artifacts deck; decklists/JD/karn-colorless-artifacts.md):
//! the commander, the restricted-mana lands, the Tron lands, the creature lands and the AI plan.
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
//!   also artifacts while animated; the AI animates one to attack when the way is open and to block a dangerous attack.
//! - **The plan**: `karn_prio` (Karn first, cheap artifacts early, Greaves/Boots once Karn is out, the sweepers only when
//!   behind or when the opponents' board is coloured), `hold_ability` (Baubles stay on the battlefield: each one is
//!   an artifact, so one more {C} a turn), `land_adjust` (the missing Tron piece), `search_pick` (the land that
//!   Expedition Map and Urza's Cave fetch).

use crate::ai::hand_key; // --- phase4-hand-off
use crate::cards::{CardDb, Types};
use crate::engine::mana::total_mana;
use crate::engine::removal::apply_removal;
use crate::engine::stack::trigger_window;
use crate::engine::values::{colors_of, epow, etgh, pval, untargetable};
use crate::engine::zones::draw;
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;

pub const KARN: &str = "Karn, Legacy Reforged";
const KEY: &str = "karn";

pub fn is_deck(key: &str) -> bool {
    key == KEY
}

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

/// the Tron pieces p lacks
fn missing_tron(g: &Game, p: PlayerId) -> Vec<&'static str> {
    TRON.iter().copied().filter(|n| !has_land(g, p, n)).collect()
}

/// engine::turn::land_score: a Tron land is worth playing first while the set isn't whole (the land drop that
/// completes it most of all); Urza's Workshop once the artifacts turn it on
pub fn land_adjust(g: &Game, p: PlayerId, c: CardId) -> f64 {
    if !is_deck(hand_key(g, p)) {
        return 0.0;
    }
    let n = &*g.db.get(c).name;
    if TRON.contains(&n) {
        let have = TRON.iter().filter(|t| has_land(g, p, t)).count();
        return if have == 2 { 8.0 } else { 3.0 };
    }
    if n == "Urza's Workshop" && artifacts(g, p) >= 3 {
        return 3.0;
    }
    0.0
}

/// the land a search (Expedition Map, Urza's Cave) takes: the Tron piece the deck lacks, then a land that does
/// something, else Wastes
pub fn search_pick(g: &Game, p: PlayerId, cands: &[CardId]) -> Option<CardId> {
    if !is_deck(hand_key(g, p)) {
        return None;
    }
    let named = |n: &str| cands.iter().copied().find(|&c| &*g.db.get(c).name == n);
    for n in missing_tron(g, p) {
        if let Some(c) = named(n) {
            return Some(c);
        }
    }
    for n in ["Urza's Workshop", "Mishra's Workshop", "Mishra's Factory", "Mishra's Foundry", "Urza's Factory"] {
        if !has_land(g, p, n)
            && let Some(c) = named(n)
        {
            return Some(c);
        }
    }
    named("Wastes").or_else(|| cands.iter().copied().find(|&c| g.db.get(c).land))
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

/// `ai::brain::attack_response`: d (a karn seat) is attacked with priority in the declare attackers step: animate
/// untapped Factories and Foundries (as many as the attackers call for) to block when the attack is dangerous.
/// True if one was.
pub fn block_response(g: &mut Game, d: PlayerId, p: PlayerId, atk: &[PermId]) -> Res<bool> {
    if !is_deck(hand_key(g, d)) {
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

// ================================================================== the plan
/// the opponents' permanents that are one or more colours (All Is Dust and Ugin hit these; Karn's list is colourless)
fn coloured_opp_perms(g: &Game, p: PlayerId) -> usize {
    g.opps(p)
        .map(|q| g.player(q).perms.iter().filter(|&&m| live(g, m) && !colors_of(g, m).is_empty()).count())
        .max()
        .unwrap_or(0)
}

fn nonland_perms(g: &Game, q: PlayerId) -> usize {
    g.player(q).perms.iter().filter(|&&m| live(g, m)).count()
}

/// the table is ahead of us: more permanents, or creatures that threaten a third of our life
fn behind(g: &Game, p: PlayerId) -> bool {
    let mine = nonland_perms(g, p);
    let life = g.player(p).life;
    g.opps(p).any(|q| {
        let power: i32 =
            g.player(q).perms.iter().filter(|&&m| live(g, m) && g.is_creature(m)).map(|&m| epow(g, m)).sum();
        nonland_perms(g, q) >= mine + 3 || power * 3 >= life
    })
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

/// Karn is out, or about to be cast
fn karn_soon(g: &Game, p: PlayerId) -> bool {
    karn_of(g, p).is_some() || g.player(p).cmd_in_zone || in_hand(g, p, KARN)
}

/// ais.deck_prio for the karn deck: its own order for the plan's cards, the generic priority for the rest (the cards'
/// own priority slots, rocks, the commander)
/// Can p win with the infinite-mana combo this turn: Basalt Monolith, Rings of Brighthearth and Walking Ballista each in
/// hand or on the battlefield, and mana for the pieces still in hand plus Basalt's own {C}{C}{C} and the {2} the loop
/// needs to start (Ballista is then cast with the infinite mana)
pub fn combo_this_turn(g: &Game, p: PlayerId) -> bool {
    let pl = g.player(p);
    let in_hand = |n: &str| pl.hand.iter().any(|&c| &*g.db.get(c).name == n);
    let out = |n: &str| pl.perms.iter().any(|&m| live(g, m) && g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == n));
    let pieces = ["Basalt Monolith", "Rings of Brighthearth", "Walking Ballista"];
    if !pieces.iter().all(|n| in_hand(n) || out(n)) {
        return false;
    }
    let need =
        5 + if in_hand("Basalt Monolith") { 3 } else { 0 } + if in_hand("Rings of Brighthearth") { 3 } else { 0 };
    total_mana(g, p, false) >= need
}

pub fn karn_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let pl = g.player(p);
    if c == pl.cmd {
        if combo_this_turn(g, p) {
            return 0; // Basalt Monolith + Rings of Brighthearth + Walking Ballista wins now: keep the mana for it
        }
        // as early as possible: well above anything else
        return if pl.turns >= 1 { (130 - 3 * pl.tax as i32).max(70) } else { 0 };
    }
    let d = g.db.get(c);
    let out = karn_of(g, p).is_some();
    match &*d.name {
        // Lightning Greaves / Swiftfoot Boots: with Karn (shroud / hexproof for the commander the table answers)
        "Lightning Greaves" | "Swiftfoot Boots" => {
            if out {
                88
            } else if karn_soon(g, p) {
                44
            } else {
                30
            }
        }
        // the sweepers take Karn and the artifacts too: when behind. All Is Dust and Ugin when the table is coloured
        "Oblivion Stone" | "Nevinyrral's Disk" => {
            if behind(g, p) && nonland_perms(g, p) <= 6 {
                72
            } else {
                0
            }
        }
        "All Is Dust" => {
            if coloured_opp_perms(g, p) >= 4 || (behind(g, p) && coloured_opp_perms(g, p) >= 2) {
                86
            } else {
                0
            }
        }
        "Ugin, the Spirit Dragon" => {
            if coloured_opp_perms(g, p) >= 4 || (behind(g, p) && coloured_opp_perms(g, p) >= 2) { 92 } else { 38 }
        }
        _ => {
            let v = crate::ai::pool::generic_prio(g, p, c);
            // cheap artifacts first: each one is another {C} from Karn every turn
            if v > 0 && d.types.has(Types::ARTIFACT) && !d.land && d.cmc <= 2 { v.max(74) } else { v }
        }
    }
}

/// `dsl::ability_options`: the Baubles stay on the battlefield while Karn is out or coming (each is an artifact, so one
/// more {C} for Karn's mana every turn), unless the hand has run dry
pub fn hold_ability(g: &Game, p: PlayerId, card: &str) -> bool {
    is_deck(hand_key(g, p))
        && matches!(card, "Mishra's Bauble" | "Urza's Bauble" | "Wayfarer's Bauble")
        && karn_soon(g, p)
        && g.player(p).hand.len() >= 2
}

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
