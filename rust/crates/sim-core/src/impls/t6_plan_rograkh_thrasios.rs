//! Tier 6 plan slot for `rograkh-thrasios-creature-combo` (Rograkh & Thrasios, green creature combo), from
//! `audit/t6/rograkh-thrasios-creature-combo/SPEC.md`. Contract: see `t6_plans.rs`.
//!
//! The plan: Rograkh ({0}) on turn 1, fast mana, cheap creatures to raise Gaea's Cradle, Thrasios; HOLD Oboro
//! Breezecaller, Eternal Witness, Snap, Finale and Green Sun's Zenith for the winning turn (the spec: "do not cast
//! Oboro on a turn the deck is not going for the win"); tutors fetch Cradle first, then the pieces of the line that
//! is closest. Mulligan on fast mana and a plan, not on the land count.

use crate::ids::{CardId, PlayerId};
use crate::state::Game;

const CRADLE: &str = "Gaea's Cradle";
const OBORO: &str = "Oboro Breezecaller";
const GATES: &str = "Talon Gates of Madara";
const WITNESS: &str = "Eternal Witness";
const SNAP: &str = "Snap";
const FINALE: &str = "Finale of Devastation";
const GSZ: &str = "Green Sun's Zenith";
const MASTERMIND: &str = "Faerie Mastermind";
const THRASIOS: &str = "Thrasios, Triton Hero";
const ROGRAKH: &str = "Rograkh, Son of Rohgahh";

/// ways to get Cradle or the engine going
const ENGINE: [&str; 9] = [
    "Crop Rotation",
    "Gamble",
    "Chord of Calling",
    "Spellseeker",
    "Formidable Speaker",
    "Rhystic Study",
    "Mystic Remora",
    "Gaea's Cradle",
    "Wan Shi Tong, Librarian",
];
/// Cradle, Talon Gates and the cards that fetch them
const FINDERS: [&str; 9] = [
    "Gaea's Cradle",
    "Talon Gates of Madara",
    "Crop Rotation",
    "Gamble",
    "Expedition Map",
    "Sowing Mycospawn",
    "Vibrance",
    "Chord of Calling",
    "Spellseeker",
];
/// purely reactive cards: two of these and no acceleration is a ship
const REACTIVE: [&str; 16] = [
    "Force of Will",
    "Force of Negation",
    "Pact of Negation",
    "Swan Song",
    "Flusterstorm",
    "Mental Misstep",
    "Mindbreak Trap",
    "Misdirection",
    "Flare of Denial",
    "Fierce Guardianship",
    "Deflecting Swat",
    "Commandeer",
    "Louisoix's Sacrifice",
    "Veil of Summer",
    "Chain of Vapor",
    "Flare of Duplication",
];

/// rthras4: some opponent has mana open and probably an instant answer in hand
fn removal_open(g: &Game, p: PlayerId) -> bool {
    use crate::ai::brain::{open_mana, prob_holding};
    g.opps(p).any(|q| {
        open_mana(g, q, None) >= 1
            && prob_holding(g, q, |c| { let d = g.db.get(c); d.instant && d.tag(crate::tag::Tag::Rem) }) >= 0.4
    })
}

fn name(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn in_hand(g: &Game, p: PlayerId, n: &str) -> bool {
    g.player(p).hand.iter().any(|&c| name(g, c) == n)
}

fn on_bf(g: &Game, p: PlayerId, n: &str) -> bool {
    // lands are not permanents in this engine: look in the land list too
    crate::ai::plans::on_bf(g, p, n).is_some()
        || g.player(p).lands.iter().any(|&l| g.land(l).on_bf && name(g, g.land(l).cd) == n)
}

fn in_gy(g: &Game, p: PlayerId, f: impl Fn(&str) -> bool) -> bool {
    g.player(p).gy.iter().any(|&c| f(name(g, c)))
}

/// Cradle is in play
fn cradle_out(g: &Game, p: PlayerId) -> bool {
    on_bf(g, p, CRADLE)
}

/// cast priority of `c` (0-100 scale like `deck_prio`; 0 or less keeps it in hand); None = no opinion
pub fn prio(g: &Game, p: PlayerId, c: CardId) -> Option<f64> {
    let n = name(g, c);
    if n == ROGRAKH {
        // rthras4 A: the third and later casts from the command zone cost 4+ extra for a 0/1: let the search decide
        if std::env::var("RT4_A").is_ok() && g.player(p).tax_of(c) >= 4 {
            return Some(40.0);
        }
        return Some(97.0); // {0}: turn 1, it turns on Fierce Guardianship, Swat and Mox Amber
    }
    if n == THRASIOS {
        // rthras4 B: after it was removed once, do not walk it back into open removal
        if std::env::var("RT4_B").is_ok() && g.player(p).tax_of(c) >= 2 && removal_open(g, p) {
            return Some(if std::env::var("RT4_B0").is_ok() { 0.0 } else { 60.0 });
        }
        return Some(93.0); // the engine, and a creature for Cradle
    }
    match n {
        // held for the winning turn: the line's own pieces and the finishers
        OBORO | FINALE | GSZ | SNAP => Some(0.0),
        // the clones are pieces of lines C and D (a copy of the earthbent land / of Badgermole Cub), not plays of their own
        "Mockingbird" | "Phantasmal Image" | "Flesh Duplicate" | "Clever Impersonator" => Some(0.0),
        // Witness is the line B piece; cast it alone only to rebuy a counter or protection
        WITNESS => {
            let value = in_gy(g, p, |x| REACTIVE.contains(&x) || x == "Crop Rotation" || x == "Noxious Revival");
            Some(if value && !cradle_out(g, p) { 55.0 } else { 0.0 })
        }
        // the finisher / backup: Mastermind stays unless it is a body for Cradle and GSZ is not in hand
        MASTERMIND => Some(if in_hand(g, p, GSZ) { 0.0 } else { 60.0 }),
        // bodies for Cradle: the cheap creature and token makers, as early as possible
        "Chatterstorm"
        | "Springheart Nantuko"
        | "Gene Pollinator"
        | "The Cabbage Merchant"
        | "Allosaurus Shepherd"
        | "Biomancer's Familiar"
        | "Cloud of Faeries"
        | "Valley Floodcaller" => Some(82.0),
        // the Cradle tutors: at once while Cradle is still in the library; never with Cradle in hand (Gamble discards at random)
        "Gamble" | "Crop Rotation" => {
            let have = cradle_out(g, p) || in_hand(g, p, CRADLE);
            let lib = g.player(p).library.iter().any(|&x| name(g, x) == CRADLE);
            // with Cradle out, the same tutor fetches Talon Gates (Oboro line); Gamble only while Cradle is not in hand
            let gates_lib = g.player(p).library.iter().any(|&x| name(g, x) == GATES);
            let gates_need = cradle_out(g, p) && gates_lib && !on_bf(g, p, GATES) && !in_hand(g, p, GATES);
            if gates_need && (n == "Gamble" || g.player(p).lands.len() >= 2) {
                Some(86.0)
            } else if have || (n == "Crop Rotation" && (!lib || g.player(p).lands.len() < 2)) {
                Some(0.0)
            } else if lib {
                Some(88.0)
            } else {
                None
            }
        }
        "Formidable Speaker" => Some(78.0),
        _ => None,
    }
}

/// tutor wish list, best first; None = no opinion (the closest combo's missing pieces)
pub fn wish(g: &Game, p: PlayerId) -> Option<Vec<CardId>> {
    let pl = g.player(p);
    let mana = pl.lands.len()
        + pl.perms.iter().filter(|&&m| g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(crate::tag::Tag::Rock))).count();
    let mut w: Vec<&str> = vec![];
    if !cradle_out(g, p) && !in_hand(g, p, CRADLE) {
        w.push(CRADLE);
    }
    if mana <= 2 {
        w.extend(["Sol Ring", "Mana Vault", "Mox Diamond", "Chrome Mox"]);
    }
    if !on_bf(g, p, GATES) && !in_hand(g, p, GATES) {
        w.push(GATES);
    }
    if !on_bf(g, p, OBORO) && !in_hand(g, p, OBORO) {
        w.push(OBORO);
    }
    // lines C (Cub + Mockingbird / Image + Oboro + Cradle) and D (Cub + clone + Witness + Snap)
    w.extend(["Badgermole Cub", "Mockingbird", "Phantasmal Image"]);
    w.extend([WITNESS, SNAP, FINALE, "Spellseeker", MASTERMIND, GSZ, "Flesh Duplicate", "Clever Impersonator", "Veil of Summer", "Sol Ring", "Mana Vault"]);
    Some(w.iter().filter_map(|n| g.db.id(n)).collect())
}

/// mulligan keep rule for the current opening hand; None = no opinion (2-5 lands)
pub fn keep(g: &Game, p: PlayerId) -> Option<bool> {
    let hand = &g.player(p).hand;
    let lands = hand.iter().filter(|&&c| g.db.get(c).land).count();
    let names: Vec<&str> = hand.iter().map(|&c| name(g, c)).collect();
    // mana by turn 2 from this hand: Mox Diamond needs a second land, Chrome Mox a spare card, Sol Ring a land,
    // a dork is summoning sick
    let m = crate::impls::t6_keep::mana(g, p);
    if m.settle() {
        return Some(true);
    }
    // rthras3: a hand that holds Cradle, Gates or a way to fetch them is kept with 2+ lands whatever else it holds
    if (2..6).contains(&lands) && names.iter().any(|n| FINDERS.contains(n))
    {
        return Some(true);
    }
    let accel = m.n_mana + usize::from(names.contains(&"Ancient Tomb"));
    let engine = names.iter().filter(|n| ENGINE.contains(n)).count();
    let reactive = names.iter().filter(|n| REACTIVE.contains(n)).count();
    // Ancient Tomb counts as a land and as acceleration, so the sum can pass the hand size
    let other = hand.len().saturating_sub(lands + accel + reactive);
    // Ancient Tomb / Gemstone Caverns / Cradle are lands (counted above); 6+ lands is flood
    if lands >= 6 {
        return Some(false);
    }
    if lands >= 2 && accel >= 1 {
        return Some(true); // "at least two lands and a piece of acceleration"
    }
    if lands == 1 {
        // fast mana and a free commander replace lands, but a 1-lander needs 2.5 more mana from the hand
        return Some(m.eff >= m.need_one_lander() && reactive < 2 && other >= 2);
    }
    // two or more lands and something to do (an engine card, or three other plays) without two dead reactive cards
    if lands >= 2 && reactive < 2 && (engine >= 1 || other >= 3) {
        return Some(true);
    }
    if lands >= 3 && reactive < 3 {
        return Some(true);
    }
    Some(false)
}

/// extra value of passing with mana up this turn (added to the shared hold value); None = no opinion
pub fn hold_up(_g: &Game, _p: PlayerId) -> Option<f64> {
    None
}

/// go for the combo named `combo` now (Some(true)), wait (Some(false)), or no opinion (None)
pub fn go_now(g: &Game, p: PlayerId, combo: &str) -> Option<bool> {
    let oboro = combo.contains("Oboro");
    if !oboro && !combo.contains("Eternal Witness") {
        return None;
    }
    let pl = g.player(p);
    let shield = pl.hand.iter().any(|&c| {
        matches!(
            name(g, c),
            "Force of Will" | "Fierce Guardianship" | "Pact of Negation" | "Veil of Summer" | "Deflecting Swat"
        )
    }) || on_bf(g, p, "Hexing Squelcher");
    if oboro {
        // line A casts one creature spell and then only abilities: go with any cover, or once the game is a few turns old
        if shield || g.round >= 3 {
            return Some(true);
        }
        return None;
    }
    // line B casts two spells: needs cover, unless the game is late
    if shield || g.round >= 4 {
        return Some(true);
    }
    None
}
