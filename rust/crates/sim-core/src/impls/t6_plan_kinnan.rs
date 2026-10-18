//! Tier 6 plan slot for `kinnan-bonder-mana-combo` (audit/t6/kinnan-bonder-mana-combo/SPEC.md). Contract: see
//! `t6_plans.rs`.
//!
//! The deck (primer: "thrives on patience and timing"): fast mana and mana dorks, Kinnan on turn 2 before the rocks,
//! a draw engine (Rhystic Study, Mystic Remora, The One Ring) while it waits, then Kinnan + Basalt Monolith (or
//! Hullbreaker Horror + two rocks) once it holds the cover for the loop. Mulligan order from the primer: draw engine,
//! an early win attempt, mana.

use crate::ai::brain::open_mana;
use crate::ids::{CardId, PlayerId};
use crate::state::Game;
use crate::tag::Tag;

const KINNAN: &str = "Kinnan, Bonder Prodigy";
const MONOLITH: &str = "Basalt Monolith";

fn nm(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

/// a card of this name is on p's battlefield
fn out(g: &Game, p: PlayerId, name: &str) -> bool {
    let pl = g.player(p);
    pl.perms.iter().any(|&m| {
        let x = g.perm(m);
        !x.phased && x.cd.is_some_and(|c| nm(g, c) == name)
    })
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| nm(g, c) == name)
}

fn have(g: &Game, p: PlayerId, name: &str) -> bool {
    out(g, p, name) || in_hand(g, p, name)
}

/// a nonland card that makes mana for the opening count
fn mana_maker(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    !d.land
        && (d.tag(Tag::Rock)
            || d.tag(Tag::Dork)
            || d.tag(Tag::Fastmana)
            || d.tag(Tag::Lr)
            || matches!(&*d.name, "Lotus Petal" | "Elvish Spirit Guide" | "Mana Vault" | "Mox Amber" | "Mox Opal"))
        && d.cmc <= 3
}

/// the primer's first item: a draw engine
fn engine(name: &str) -> bool {
    matches!(name, "Rhystic Study" | "Mystic Remora" | "The One Ring" | "Consecrated Sphinx" | "Sylvan Library")
}

/// the primer's second item: an early win attempt
fn win_piece(name: &str) -> bool {
    matches!(name, MONOLITH | "Transmute Artifact" | "Hullbreaker Horror" | "Trophy Mage" | "Thrasios, Triton Hero")
}

/// a counter that costs no mana (or is paid by pitching a card)
fn free_counter_name(name: &str) -> bool {
    matches!(
        name,
        "Force of Will"
            | "Fierce Guardianship"
            | "Pact of Negation"
            | "Misdirection"
            | "Mental Misstep"
            | "Disrupting Shoal"
            | "Force of Negation"
            | "Snapback"
            | "Mindbreak Trap"
    )
}

/// cast priority: Kinnan before the rocks (each taps for double), the draw engines while they are cheap
pub fn prio(g: &Game, p: PlayerId, c: CardId) -> Option<f64> {
    let pl = g.player(p);
    if pl.is_commander(c) {
        return Some(92.0 - 3.0 * pl.tax_of(c) as f64);
    }
    match nm(g, c) {
        "Rhystic Study" | "Mystic Remora" => Some(if pl.turns <= 4 { 84.0 } else { 55.0 }),
        "The One Ring" => Some(if pl.turns <= 6 { 66.0 } else { 55.0 }),
        "Consecrated Sphinx" => Some(70.0),
        "Sylvan Library" => Some(60.0),
        // the counters and bounce are held, never cast into an empty table
        "Force of Will"
        | "Fierce Guardianship"
        | "Force of Negation"
        | "Pact of Negation"
        | "Mental Misstep"
        | "Misdirection"
        | "Disrupting Shoal"
        | "Mindbreak Trap"
        | "Snapback"
        | "Chain of Vapor"
        | "Dispel"
        | "Flusterstorm"
        | "Swan Song"
        | "An Offer You Can't Refuse" => Some(0.0),
        _ => None,
    }
}

/// tutor wish list, best first: the win line's missing pieces (Monolith, then an outlet, then the Horror line), then
/// the draw engines and the cover
pub fn wish(g: &Game, p: PlayerId) -> Option<Vec<CardId>> {
    let lib = &g.player(p).library;
    let kinnan_out = out(g, p, KINNAN);
    let outlet = have(g, p, "Thrasios, Triton Hero") || have(g, p, "Mirage Mirror");
    let rocks = g.player(p).perms.iter().filter(|&&m| g.perm(m).cd.is_some_and(|c| mana_maker(g, c))).count();
    let mut v: Vec<(f64, CardId)> = vec![];
    for &c in lib {
        let name = nm(g, c);
        let s = match name {
            MONOLITH if !have(g, p, MONOLITH) => 9.0,
            "Transmute Artifact" if !have(g, p, MONOLITH) => 8.5,
            "Thrasios, Triton Hero" if !outlet => 8.0,
            "Mirage Mirror" if !outlet => 7.0,
            "Hullbreaker Horror" if kinnan_out && rocks >= 2 => 7.5,
            "Rhystic Study" | "Mystic Remora" if g.player(p).turns <= 5 => 6.5,
            "Force of Will" | "Fierce Guardianship" | "Pact of Negation" if have(g, p, MONOLITH) => 7.2,
            "Faerie Mastermind" | "Endurance" | "Chain of Vapor" => 4.0,
            "Consecrated Sphinx" | "The One Ring" => 5.5,
            "Force of Will" | "Fierce Guardianship" | "Pact of Negation" => 4.5,
            "Finale of Devastation" => 3.5,
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

/// Mulligan (primer: draw engine, then an early win attempt, then mana; ship one-land hands without free mana, keep
/// fast mana with Kinnan): ship no land, six or more lands, and hands with nothing to do.
pub fn keep(g: &Game, p: PlayerId) -> Option<bool> {
    let hand = &g.player(p).hand;
    let lands = hand.iter().filter(|&&c| g.db.get(c).land).count();
    // Mox Diamond needs a land to discard: it makes mana only with a second land
    let makers = hand.iter().filter(|&&c| mana_maker(g, c) && (nm(g, c) != "Mox Diamond" || lands >= 2)).count();
    let engines = hand.iter().filter(|&&c| engine(nm(g, c))).count();
    let cheap_engine = hand.iter().any(|&c| engine(nm(g, c)) && g.db.get(c).cmc <= 2);
    let wins = hand.iter().filter(|&&c| win_piece(nm(g, c))).count();
    let free = hand.iter().filter(|&&c| free_counter_name(nm(g, c))).count();
    let action = makers + engines + wins;
    let ok = match lands {
        0 => false,
        1 => makers >= 2,
        2 => makers >= 1 || cheap_engine || engines + wins >= 1 || free >= 2,
        3 | 4 => action + free >= 1,
        5 => action >= 2,
        _ => false,
    };
    Some(ok)
}

/// extra value of passing with mana up: develop on turns 1-2, then hold the cover while the draw engines work
pub fn hold_up(g: &Game, p: PlayerId) -> Option<f64> {
    let pl = g.player(p);
    if pl.turns <= 2 {
        return Some(-2.0);
    }
    let cover = pl.hand.iter().filter(|&&c| free_counter_name(nm(g, c))).count();
    (cover >= 1 && out(g, p, KINNAN)).then_some(1.0)
}

/// Go for Kinnan's loops only with cover: a free counter or a paid one for the open blue mana, and no stax (Null Rod,
/// Stony Silence, Collector Ouphe) on the table. Late games (round 6) fall to the shared rule.
pub fn go_now(g: &Game, p: PlayerId, combo: &str) -> Option<bool> {
    if !(combo.starts_with("Kinnan + Basalt Monolith") || combo.starts_with("Hullbreaker Horror + Kinnan")) {
        return None;
    }
    if g.round >= 6 {
        return None;
    }
    let stax = g.opps(p).any(|q| {
        g.player(q).perms.iter().any(|&m| {
            g.perm(m)
                .cd
                .is_some_and(|c| matches!(nm(g, c), "Null Rod" | "Stony Silence" | "Collector Ouphe" | "Cursed Totem"))
        })
    });
    if stax {
        return Some(false);
    }
    let pl = g.player(p);
    let free = pl.hand.iter().filter(|&&c| free_counter_name(nm(g, c))).count();
    let paid = pl.hand.iter().filter(|&&c| g.db.get(c).tag(Tag::Ctr) && !free_counter_name(nm(g, c))).count();
    let blue_up = g.opps(p).filter(|&q| open_mana(g, q, Some('U')) >= 2 && !g.player(q).hand.is_empty()).count();
    if free + paid == 0 && blue_up >= 1 {
        return Some(false); // bait first or wait: no cover and an opponent can counter
    }
    if free >= 1 && blue_up <= 1 {
        return Some(true);
    }
    None
}
