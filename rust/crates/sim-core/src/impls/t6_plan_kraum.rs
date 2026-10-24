//! Tier 6 plan slot for `kraum-tymna-blue-breach-midrange` (Kraum & Tymna, blue midrange), from
//! `audit/t6/kraum-tymna-blue-breach-midrange/SPEC.md`. Contract: see `t6_plans.rs`.
//!
//! The plan: fast mana and a cheap engine on turns 1-2 (Esper Sentinel, Rhystic Study, Remora, Tymna), counters held
//! for the table's win attempts and kept up from turn 3, Kraum only if a counter stays up, and the win late: Oracle
//! + Consultation/Pact only with cover (a creature counter or Silence/Orim's Chant) or when the table cannot answer,
//! never with Kraum / Esper Sentinel on its own board, Ad Nauseam from a healthy life total.

use crate::ai::brain::open_mana;
use crate::ai::t6_table::stop_chance;
use crate::ids::{CardId, PlayerId};
use crate::state::Game;
use crate::tag::Tag;

const ORACLE: &str = "Thassa's Oracle";

fn nm(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| nm(g, c) == name)
}

fn out(g: &Game, p: PlayerId, name: &str) -> bool {
    crate::ai::plans::on_bf(g, p, name).is_some()
}

/// fast mana (a land that is also a mana source for the opening: Tomb, Caverns, City of Traitors)
fn fast_mana(name: &str) -> bool {
    matches!(
        name,
        "Sol Ring"
            | "Mana Vault"
            | "Mox Amber"
            | "Mox Diamond"
            | "Mox Opal"
            | "Chrome Mox"
            | "Lotus Petal"
            | "Simian Spirit Guide"
            | "Dark Ritual"
            | "Cabal Ritual"
            | "Arcane Signet"
            | "Fellwar Stone"
            | "Lion's Eye Diamond"
    )
}

fn engine(name: &str) -> bool {
    matches!(
        name,
        "Rhystic Study"
            | "Mystic Remora"
            | "Esper Sentinel"
            | "Smothering Tithe"
            | "Orcish Bowmasters"
            | "Ragavan, Nimble Pilferer"
            | "Necropotence"
    )
}

fn tutor(name: &str) -> bool {
    matches!(
        name,
        "Demonic Tutor"
            | "Vampiric Tutor"
            | "Mystical Tutor"
            | "Enlightened Tutor"
            | "Imperial Seal"
            | "Gamble"
            | "Intuition"
            | "Beseech the Mirror"
            | "Wishclaw Talisman"
    )
}

fn payoff(name: &str) -> bool {
    matches!(
        name,
        "Thassa's Oracle"
            | "Demonic Consultation"
            | "Tainted Pact"
            | "Ad Nauseam"
            | "Underworld Breach"
            | "Brain Freeze"
    )
}

/// a counter that costs no mana (or is paid by pitching a card)
fn free_counter(name: &str) -> bool {
    matches!(
        name,
        "Force of Will"
            | "Fierce Guardianship"
            | "Pact of Negation"
            | "Force of Negation"
            | "Mental Misstep"
            | "Deflecting Swat"
            | "Misdirection"
            | "Mindbreak Trap"
    )
}

/// a counter that stops a creature spell (Oracle): Force of Will, Pact, or a Silence effect
fn creature_cover(g: &Game, p: PlayerId) -> bool {
    let h = &g.player(p).hand;
    let blue_cards = h.iter().filter(|&&c| g.db.get(c).colors.has('U')).count();
    h.iter().any(|&c| match nm(g, c) {
        "Pact of Negation" | "Silence" => true,
        "Orim's Chant" => true,
        "Force of Will" => blue_cards >= 2,
        _ => false,
    })
}

/// counters / protection in hand that are not part of the win itself
fn cover_count(g: &Game, p: PlayerId) -> usize {
    g.player(p)
        .hand
        .iter()
        .filter(|&&c| {
            let d = g.db.get(c);
            (d.tag(Tag::Ctr) || matches!(&*d.name, "Silence" | "Orim's Chant")) && !payoff(&d.name)
        })
        .count()
}

/// cast priority: fast mana, then the cheap engines; the win pieces and the counters wait in hand
pub fn prio(g: &Game, p: PlayerId, c: CardId) -> Option<f64> {
    let pl = g.player(p);
    if pl.is_commander(c) {
        let n = nm(g, c);
        if n.starts_with("Tymna") {
            return Some(86.0 - 3.0 * pl.tax_of(c) as f64);
        }
        // Kraum (5 mana): only with a counter still available after it, or when the table has nothing open
        let spare = cover_count(g, p) >= 1 || g.opps(p).all(|q| open_mana(g, q, None) == 0);
        return Some(if spare { 72.0 } else { 20.0 });
    }
    let n = nm(g, c);
    if fast_mana(n) && !matches!(n, "Dark Ritual" | "Cabal Ritual" | "Lion's Eye Diamond") {
        return Some(95.0);
    }
    match n {
        "Esper Sentinel" => Some(90.0),
        "Rhystic Study" => Some(if pl.turns <= 4 { 88.0 } else { 60.0 }),
        "Mystic Remora" => Some(if pl.turns <= 4 { 84.0 } else { 45.0 }),
        "Smothering Tithe" => Some(70.0),
        // the win pieces and the protection: used by the combo framework, never cast alone
        "Thassa's Oracle"
        | "Demonic Consultation"
        | "Tainted Pact"
        | "Brain Freeze"
        | "Final Fortune"
        | "Silence"
        | "Orim's Chant"
        | "Rain of Filth" => Some(0.0),
        // the counters are answers: never cast into an empty table
        "Force of Will"
        | "Fierce Guardianship"
        | "Pact of Negation"
        | "Force of Negation"
        | "Mental Misstep"
        | "Deflecting Swat"
        | "Misdirection"
        | "Mindbreak Trap"
        | "Snapback"
        | "Dispel"
        | "Flusterstorm"
        | "Swan Song"
        | "An Offer You Can't Refuse"
        | "Red Elemental Blast"
        | "Chain of Vapor"
        | "Into the Flood Maw"
        | "Commandeer" => Some(0.0),
        _ => None,
    }
}

/// tutor wish list: the missing half of the Oracle line first, then cover, then the engine
pub fn wish(g: &Game, p: PlayerId) -> Option<Vec<CardId>> {
    let have_oracle = in_hand(g, p, ORACLE) || out(g, p, ORACLE);
    let have_exiler = in_hand(g, p, "Demonic Consultation") || in_hand(g, p, "Tainted Pact");
    let cover = cover_count(g, p);
    let engine_out = out(g, p, "Rhystic Study") || out(g, p, "Mystic Remora") || out(g, p, "Esper Sentinel");
    let mut v: Vec<(f64, CardId)> = vec![];
    for &c in &g.player(p).library {
        let n = nm(g, c);
        let s = match n {
            // without cover, protection first: the pieces are cheap to find later
            "Force of Will" | "Pact of Negation" | "Silence" if have_oracle && have_exiler && cover == 0 => 9.5,
            "Thassa's Oracle" if !have_oracle => 9.0,
            "Tainted Pact" if !have_exiler => 8.8,
            "Demonic Consultation" if !have_exiler => 8.4, // as a tutor-free piece only; never cast to tutor
            "Force of Will" | "Pact of Negation" if cover == 0 => 7.0,
            "Rhystic Study" | "Esper Sentinel" | "Mystic Remora" if !engine_out && g.player(p).turns <= 4 => 8.0,
            "Silence" | "Orim's Chant" | "Fierce Guardianship" if cover == 0 => 6.0,
            "Ad Nauseam" => 5.0,
            "Underworld Breach" | "Lion's Eye Diamond" => 4.0,
            "Mana Vault" | "Sol Ring" | "Mox Diamond" | "Chrome Mox" => 3.0,
            _ => 0.0,
        };
        if s > 0.0 {
            v.push((s, c));
        }
    }
    if v.is_empty() {
        return None;
    }
    v.sort_by(|a, b| b.0.total_cmp(&a.0));
    Some(v.into_iter().map(|x| x.1).collect())
}

/// Mulligan (spec: keep fast mana, an early engine or a tutor with a counter; a lone engine or lands and spells is a
/// ship; go low). Lands include Ancient Tomb, Gemstone Caverns and City of Traitors.
pub fn keep(g: &Game, p: PlayerId) -> Option<bool> {
    let hand = &g.player(p).hand;
    // mana by turn 2 from this hand: Mox Diamond needs a second land, Chrome Mox a spare card, Sol Ring a land
    let m = crate::impls::t6_keep::mana(g, p);
    if m.settle() {
        return Some(true);
    }
    let lands = m.lands;
    let names: Vec<&str> = hand.iter().filter(|&&c| !g.db.get(c).land).map(|&c| nm(g, c)).collect();
    let makers = m.n_mana;
    let engines = names.iter().filter(|&&n| engine(n)).count();
    let tutors = names.iter().filter(|&&n| tutor(n)).count();
    let pay = names.iter().filter(|&&n| payoff(n)).count();
    let free = names.iter().filter(|&&n| free_counter(n)).count();
    let action = makers + engines + tutors + pay;
    let ok = match lands {
        0 => false,
        1 => m.eff >= m.need_one_lander() && (engines + tutors + pay >= 1),
        2 => action >= 2 || (action >= 1 && free >= 1) || free >= 3,
        // spec: a lone engine is not a keep (Ondas); an engine needs a free counter or a second action with it
        3 | 4 => makers + tutors + pay >= 1 || engines >= 2 || (engines >= 1 && free >= 1) || free >= 2,
        5 => action >= 1 && action + free >= 2,
        _ => false,
    };
    Some(ok)
}

/// extra value of passing with mana up: develop on turns 1-2, keep the counters up once the engine is out or a
/// rival is close to winning
pub fn hold_up(g: &Game, p: PlayerId) -> Option<f64> {
    let pl = g.player(p);
    let cover = cover_count(g, p);
    if pl.turns <= 2 {
        return Some(-2.0);
    }
    if cover == 0 || crate::impls::combos::line_progress(g, p).1 {
        return None; // a line is ready: no bonus for passing
    }
    let near = g.opps(p).any(|q| crate::ai::t6_table::archenemy_score(g, q) >= 5.0);
    let engine_out = out(g, p, "Rhystic Study") || out(g, p, "Mystic Remora") || out(g, p, "Esper Sentinel");
    if near {
        Some(2.5)
    } else if engine_out || pl.turns >= 4 {
        Some(1.0)
    } else {
        None
    }
}

/// Go for the Oracle line only with cover (a creature counter or a Silence effect) when an opponent can answer, never
/// with Kraum / Esper Sentinel on my own board while an opponent can force a draw; Ad Nauseam from 26+ life; the Breach
/// line is not held back.
pub fn go_now(g: &Game, p: PlayerId, combo: &str) -> Option<bool> {
    let pl = g.player(p);
    let can_answer = g.opps(p).filter(|&q| stop_chance(g, q) >= 0.5).count();
    if combo.starts_with("Thassa's Oracle") {
        // an empty-library draw in response to the Oracle trigger loses. Esper Sentinel draws on an opponent's first
        // noncreature spell: any opponent with a card and a mana can force it. Kraum draws on an opponent's SECOND
        // spell each turn (spec scenario 6, "any opponent second spell"): an opponent who already cast one this turn
        // and still holds a card, or one with two cards and two mana up. The old rule held on one card and one mana,
        // which can't cast two spells, so Kraum on the battlefield kept a ready line from going in a third of the games
        let cards_up = |q: PlayerId| !g.player(q).hand.is_empty() && open_mana(g, q, None) >= 1;
        let second_spell = |q: PlayerId| {
            let (cards, mana) = (g.player(q).hand.len(), open_mana(g, q, None));
            (crate::engine::cast::casts_this_turn(g, q) >= 1 && cards >= 1) || (cards >= 2 && mana >= 2)
        };
        let sentinel = out(g, p, "Esper Sentinel") && g.opps(p).any(cards_up);
        let kraum = crate::ai::plans::on_bf(g, p, "Kraum, Ludevic's Opus").is_some() && g.opps(p).any(second_spell);
        if sentinel || kraum {
            return Some(false);
        }
        // spec (conditions to go): a creature counter or Silence effect (Oracle is a creature spell), any one piece of
        // protection, or no opponent likely to answer. JD (2026-10-24): wait only when an opponent is LIKELY to stop
        // it (stop chance 0.5+). The old rule also waited whenever any opponent had a card and a mana up, which at a
        // cEDH table is nearly always: a ready line went in only 32 of 43 ready games, and waiting buys little when
        // the table's free counters do not go away
        let worst = g.opps(p).map(|q| stop_chance(g, q)).fold(0.0_f64, f64::max);
        let tapped_low = worst < 0.2 && g.opps(p).all(|q| open_mana(g, q, None) <= 2);
        let one_cover = cover_count(g, p) >= 1 && worst < 0.5;
        // the spec's plain case still waits: an opponent with cards and two blue up can pay for a counter
        let blue_up = g.opps(p).any(|q| !g.player(q).hand.is_empty() && open_mana(g, q, Some('U')) >= 2);
        if (can_answer >= 1 || blue_up) && !creature_cover(g, p) && !tapped_low && !one_cover {
            // nothing covers Oracle: wait unless the game is late and the table is about to win
            let near = g.opps(p).any(|q| crate::ai::t6_table::archenemy_score(g, q) >= 6.0);
            return if near && g.round >= 5 { None } else { Some(false) };
        }
        return None;
    }
    if combo.starts_with("Ad Nauseam") {
        if pl.life < 26 {
            return Some(false);
        }
        return None;
    }
    None
}
