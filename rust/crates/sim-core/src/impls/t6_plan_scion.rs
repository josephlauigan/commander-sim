//! Tier 6 plan slot for `scion`, JD's Scion of the Ur-Dragon turbo deck (v1: decklists/JD/scion-ur-dragon-turbo.md).
//! Contract: `t6_plans.rs`.
//!
//! The deck is TURBO: fast mana, rituals and dorks into Thassa's Oracle + Demonic Consultation / Tainted Pact or
//! Underworld Breach + Lion's Eye Diamond + Brain Freeze, with free counters and Silence / Orim's Chant held for the
//! turn of the attempt. Scion of the Ur-Dragon is the five-colour commander and an engine: {2} turns it into a copy of
//! a Dragon from the library (Prismari, the Inspiration for storm, Ancient Copper Dragon for Treasures, Korvold for
//! cards); the activation and its choice of Dragon are card code (`t6_scion.rs`), not this plan.
//!
//! Only `keep` has an opinion (a starting point); `prio`, `wish`, `hold_up` and `go_now` leave every decision to the
//! shared Tier 6 table rules and the generic AI, so later versions of the deck are measured against a plain baseline.

use crate::ids::{CardId, PlayerId};
use crate::state::Game;

/// one-shot mana: rituals and spirit guides (Tinder Wall is a dork the shared mana count already sees)
const RITUALS: [&str; 6] =
    ["Dark Ritual", "Cabal Ritual", "Rite of Flame", "Manamorphose", "Elvish Spirit Guide", "Simian Spirit Guide"];
const TUTORS: [&str; 12] = [
    "Demonic Tutor",
    "Vampiric Tutor",
    "Mystical Tutor",
    "Enlightened Tutor",
    "Imperial Seal",
    "Gamble",
    "Diabolic Intent",
    "Beseech the Mirror",
    "Intuition",
    "Demonic Consultation",
    "Tainted Pact",
    "Crop Rotation",
];
const PAYOFFS: [&str; 4] = ["Necropotence", "Underworld Breach", "Brain Freeze", "Final Fortune"];
const ENGINES: [&str; 5] = ["Rhystic Study", "Mystic Remora", "Esper Sentinel", "Smothering Tithe", "Tataru Taru"];

fn name(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn in_hand(g: &Game, p: PlayerId, n: &str) -> bool {
    g.player(p).hand.iter().any(|&c| name(g, c) == n)
}

fn count_in(g: &Game, p: PlayerId, set: &[&str]) -> usize {
    g.player(p).hand.iter().filter(|&&c| set.contains(&name(g, c))).count()
}

/// cast priority: no opinion (the shared Tier 6 rules and the generic priority decide)
pub fn prio(_g: &Game, _p: PlayerId, _c: CardId) -> Option<f64> {
    None
}

/// tutor wish list: no opinion (the closest combo's missing pieces)
pub fn wish(_g: &Game, _p: PlayerId) -> Option<Vec<CardId>> {
    None
}

/// Mulligan keep rule, PROVISIONAL until JD decides the mulligans (D11): the Rograkh/Silas turbo rule
/// (`t6_plan_rograkh_silas::keep`) with this list's rituals, tutors, payoffs and engines. Fast mana and a plan beat land
/// count; the shared `t6_keep::mana` counts what the hand's mana really makes by turn 2.
pub fn keep(g: &Game, p: PlayerId) -> Option<bool> {
    let hand = &g.player(p).hand;
    let lands = hand.iter().filter(|&&c| g.db.get(c).land).count();
    let m = crate::impls::t6_keep::mana(g, p);
    if m.settle() {
        return Some(true);
    }
    let rituals = count_in(g, p, &RITUALS);
    let fast = (m.n_mana + rituals) as f64;
    let mana = m.eff + rituals as f64;
    let tutors = count_in(g, p, &TUTORS);
    let payoffs = count_in(g, p, &PAYOFFS);
    let engines = if fast >= 1.0 { count_in(g, p, &ENGINES) } else { 0 }; // an engine alone does not win
    let oracle = in_hand(g, p, "Thassa's Oracle");
    let action = tutors + payoffs + engines + usize::from(oracle);
    if lands >= 6 {
        return Some(false); // ship all-land hands
    }
    if lands == 0 {
        // no colours from lands: only with three fast mana pieces and a tutor or payoff to cast
        return Some(m.n_mana + rituals >= 3 && tutors + payoffs >= 1);
    }
    if lands == 1 && (mana < m.need_one_lander() || action == 0) {
        return Some(false); // a 1-lander needs the mana (rituals count) and a payoff or tutor to spend it on
    }
    if mana < 2.0 || (action == 0 && fast < 2.0) {
        return Some(false); // lands-and-spells hands with no payoff or tutor, or no mana to cast it
    }
    if lands >= 5 && fast == 0.0 && action < 2 {
        return Some(false);
    }
    Some(true)
}

/// hold-up: no opinion
pub fn hold_up(_g: &Game, _p: PlayerId) -> Option<f64> {
    None
}

/// go-now: no opinion; the shared table rules decide
pub fn go_now(_g: &Game, _p: PlayerId, _combo: &str) -> Option<bool> {
    None
}
