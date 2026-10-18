//! Tier 6 plan slot for `rograkh-silas-turbo-breach` (SPEC: audit/t6/rograkh-silas-turbo-breach/SPEC.md). Contract:
//! `t6_plans.rs`.
//!
//! The deck is TURBO: Rograkh ({0}) is on the table on turn 1, the mana comes from Moxen, Petal, Sol Ring and Vault, the
//! tutors find Thassa's Oracle + Demonic Consultation / Tainted Pact (main line) or the Breach pieces, and the
//! protection (free counters) is held for the turn of the attempt. Silas is "mostly there for the colours".

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
const SLOW: [&str; 6] = [
    "Mox Opal",
    "Grim Monolith",
    "Arcane Signet",
    "Talisman of Dominance",
    "Talisman of Indulgence",
    "Springleaf Drum",
];
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
const PAYOFFS: [&str; 7] = [
    "Ad Nauseam",
    "Necropotence",
    "Wheel of Fortune",
    "Windfall",
    "Timetwister",
    "Yawgmoth's Will",
    "Underworld Breach",
];
const ENGINES: [&str; 2] = ["Rhystic Study", "Mystic Remora"];

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
/// cast priority: no opinion. A turbo-development priority (Rograkh first, rocks first, rituals held, Squelcher before the
/// Oracle line) measured WORSE in 300-game A/Bs (15-17% against 21-25% with the keep rule alone), so the shared AI decides.
pub fn prio(_g: &Game, _p: PlayerId, _c: CardId) -> Option<f64> {
    None
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
    // mana by turn 2 from this hand: Mox Diamond needs a second land, Chrome Mox a spare card, Sol Ring a land
    let m = crate::impls::t6_keep::mana(g, p);
    if m.settle() {
        return Some(true);
    }
    let fast = m.n_mana as f64;
    let mana = m.eff;
    let tutors = count_in(g, p, &TUTORS);
    let payoffs = count_in(g, p, &PAYOFFS);
    let engines = if fast >= 1.0 { count_in(g, p, &ENGINES) } else { 0 }; // Rhystic/Remora alone do not win
    let oracle = in_hand(g, p, "Thassa's Oracle");
    let action = tutors + payoffs + engines + usize::from(oracle);
    if lands >= 6 {
        return Some(false); // ship all-land hands
    }
    if lands == 0 {
        // no colours from lands: only with three fast mana pieces and a tutor or payoff to cast
        return Some(m.free >= 3 && tutors + payoffs >= 1);
    }
    if lands == 1 && (mana < m.need_one_lander() || action == 0) {
        return Some(false); // a 1-lander needs 2.5 more mana from the hand and a payoff or tutor to spend it on
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
/// hold-up: no opinion (a develop-first value measured worse)
pub fn hold_up(_g: &Game, _p: PlayerId) -> Option<f64> {
    None
}

/// go-now: no opinion; the shared table rules decide
pub fn go_now(_g: &Game, _p: PlayerId, _combo: &str) -> Option<bool> {
    None
}
