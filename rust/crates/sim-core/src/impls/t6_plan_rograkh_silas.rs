//! Tier 6 plan slot for `rograkh-silas-turbo-breach` (SPEC: audit/t6/rograkh-silas-turbo-breach/SPEC.md). Contract:
//! `t6_plans.rs`.
//!
//! The deck is TURBO: Rograkh ({0}) is on the table on turn 1, the mana comes from Moxen, Petal, Sol Ring and Vault, the
//! tutors find Thassa's Oracle + Demonic Consultation / Tainted Pact (main line) or the Breach pieces, and the
//! protection (free counters) is held for the turn of the attempt. Silas is "mostly there for the colours".

use crate::ai::plans::on_bf;
use crate::engine::mana::can_pay;
use crate::ids::{CardId, PlayerId};
use crate::state::Game;

const FAST: [&str; 13] = [
    "Lotus Petal",
    "Mox Amber",
    "Chrome Mox",
    "Mox Diamond",
    "Sol Ring",
    "Mana Vault",
    "Dark Ritual",
    "Cabal Ritual",
    "Simian Spirit Guide",
    "Rite of Flame",
    "Jeska's Will",
    "Culling the Weak",
    "Infernal Plunge",
];
const SLOW: [&str; 6] = ["Mox Opal", "Grim Monolith", "Arcane Signet", "Talisman of Dominance", "Talisman of Indulgence", "Springleaf Drum"];
const TUTORS: [&str; 11] = [
    "Demonic Tutor",
    "Vampiric Tutor",
    "Mystical Tutor",
    "Imperial Seal",
    "Gamble",
    "Diabolic Intent",
    "Beseech the Mirror",
    "Demonic Counsel",
    "Wishclaw Talisman",
    "Demonic Consultation",
    "Tainted Pact",
];
const PAYOFFS: [&str; 7] =
    ["Ad Nauseam", "Necropotence", "Wheel of Fortune", "Windfall", "Timetwister", "Yawgmoth's Will", "Underworld Breach"];
const ENGINES: [&str; 2] = ["Rhystic Study", "Mystic Remora"];
const RITUALS: [&str; 7] = [
    "Dark Ritual",
    "Cabal Ritual",
    "Rite of Flame",
    "Simian Spirit Guide",
    "Jeska's Will",
    "Culling the Weak",
    "Infernal Plunge",
];
/// the free or cheap counters kept for the attempt
const FREE_PROT: [&str; 6] =
    ["Force of Will", "Fierce Guardianship", "Pact of Negation", "Deflecting Swat", "Mental Misstep", "Misdirection"];

fn name(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn in_hand(g: &Game, p: PlayerId, n: &str) -> bool {
    g.player(p).hand.iter().any(|&c| name(g, c) == n)
}

fn count_in(g: &Game, p: PlayerId, set: &[&str]) -> usize {
    g.player(p).hand.iter().filter(|&&c| set.contains(&name(g, c))).count()
}

/// the legendary Rograkh (or Silas) is on the battlefield
fn commander_out(g: &Game, p: PlayerId) -> bool {
    on_bf(g, p, "Rograkh, Son of Rohgahh").is_some() || on_bf(g, p, "Silas Renn, Seeker Adept").is_some()
}

/// Oracle plus an exile piece, both in hand
fn oracle_line_in_hand(g: &Game, p: PlayerId) -> bool {
    in_hand(g, p, "Thassa's Oracle") && (in_hand(g, p, "Demonic Consultation") || in_hand(g, p, "Tainted Pact"))
}

/// Ad Nauseam in hand with a finish in hand, but not enough mana for both yet: a ritual closes the gap
fn ad_wants_ritual(g: &Game, p: PlayerId) -> bool {
    in_hand(g, p, "Ad Nauseam")
        && (oracle_line_in_hand(g, p) || (in_hand(g, p, "Underworld Breach") && in_hand(g, p, "Lion's Eye Diamond")))
        && !can_pay(g, p, 3, "BBUUB", false)
}

/// cast priority of `c` (0-100 scale like `deck_prio`; 0 or less keeps it in hand); None = no opinion
pub fn prio(g: &Game, p: PlayerId, c: CardId) -> Option<f64> {
    let n = name(g, c);
    let pl = g.player(p);
    if pl.is_commander(c) {
        if n == "Rograkh, Son of Rohgahh" {
            // free: always first (turn 1), it turns on Mox Amber, Fierce Guardianship and Deflecting Swat; a recast
            // costs {2} per cast so far: still worth it for a few
            return Some(if pl.tax_of(c) <= 2 { 100.0 } else { 45.0 });
        }
        // Silas is mostly there for the colours: only with Rograkh out and mana to spare
        let spare = crate::engine::mana::total_mana(g, p, false) >= 7;
        return Some(if commander_out(g, p) && spare && pl.turns >= 4 { 25.0 } else { 0.0 });
    }
    if n == "Mox Amber" {
        return Some(if commander_out(g, p) { 96.0 } else { 0.0 }); // a Mox only with a legend in play
    }
    if RITUALS.contains(&n) {
        // rituals only to close a gap for an Ad Nauseam turn; otherwise held (mana would just be lost)
        let sac = matches!(n, "Culling the Weak" | "Infernal Plunge");
        return Some(if ad_wants_ritual(g, p) && (!sac || commander_out(g, p)) { 97.0 } else { 0.0 });
    }
    match n {
        "Sol Ring" | "Mana Vault" => Some(95.0),
        "Lotus Petal" => Some(if oracle_line_in_hand(g, p) || pl.turns >= 1 { 93.0 } else { 80.0 }),
        // Dark Floodcaller, Wishclaw (an opponent gets it), Bauble, Silas: no value in a turbo turn
        "Valley Floodcaller" | "Vexing Bauble" | "Grinding Station" | "Chromatic Star" => Some(0.0),
        "Wishclaw Talisman" => Some(if oracle_line_in_hand(g, p) { 0.0 } else { 30.0 }),
        "Hexing Squelcher" | "Defense Grid" => Some(0.0), // cast before the attempt (protection), not as development
        _ => None,
    }
}

/// tutor wish list, best first; None = no opinion (the closest combo's missing pieces)
pub fn wish(g: &Game, p: PlayerId) -> Option<Vec<CardId>> {
    let has_o = in_hand(g, p, "Thassa's Oracle");
    let has_x = in_hand(g, p, "Demonic Consultation") || in_hand(g, p, "Tainted Pact");
    let names: &[&str] = match (has_o, has_x) {
        (true, true) => return None, // the line is in hand
        (true, false) => &["Demonic Consultation", "Tainted Pact"],
        (false, true) => &["Thassa's Oracle"],
        (false, false) => &["Thassa's Oracle", "Demonic Consultation", "Tainted Pact"],
    };
    let ids: Vec<CardId> = names.iter().filter_map(|n| g.db.id(n)).collect();
    Some(ids)
}

/// mulligan keep rule: fast mana and a plan beat land count (the deck plays 23 lands in 99 and 18 fast-mana pieces)
pub fn keep(g: &Game, p: PlayerId) -> Option<bool> {
    let hand = &g.player(p).hand;
    let lands = hand.iter().filter(|&&c| g.db.get(c).land).count();
    let fast = count_in(g, p, &FAST) as f64;
    let slow = count_in(g, p, &SLOW) as f64 * 0.5;
    let mana = lands as f64 + fast + slow;
    let tutors = count_in(g, p, &TUTORS);
    let payoffs = count_in(g, p, &PAYOFFS);
    let engines = if fast >= 1.0 { count_in(g, p, &ENGINES) } else { 0 }; // Rhystic/Remora alone do not win
    let oracle = in_hand(g, p, "Thassa's Oracle");
    let action = tutors + payoffs + engines + usize::from(oracle);
    if lands == 0 || lands >= 6 {
        return Some(false); // ship all-land hands; a no-land hand has no colours
    }
    if mana < 2.0 || (action == 0 && fast < 2.0) {
        return Some(false); // lands-and-spells hands with no payoff or tutor, or no mana to cast it
    }
    if lands >= 5 && fast == 0.0 && action < 2 {
        return Some(false);
    }
    Some(true)
}

/// extra value of passing with mana up this turn (added to the shared hold value); None = no opinion
pub fn hold_up(g: &Game, p: PlayerId) -> Option<f64> {
    // develop first: Rograkh, a Mox or Sol Ring that can be cast now beats passing with a Pyroblast up
    let pl = g.player(p);
    let out = commander_out(g, p);
    let develop = pl.hand.iter().any(|&c| {
        let n = name(g, c);
        matches!(n, "Sol Ring" | "Mana Vault" | "Lotus Petal") || (n == "Mox Amber" && out)
    });
    if develop || (!out && pl.turns <= 2) { Some(-50.0) } else { None }
}

/// go for the combo named `combo` now (Some(true)), wait (Some(false)), or no opinion (None)
pub fn go_now(g: &Game, p: PlayerId, combo: &str) -> Option<bool> {
    let _ = (g, p, combo, FREE_PROT);
    None
}
