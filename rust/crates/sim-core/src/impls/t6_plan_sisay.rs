//! Tier 6 plan slot for `sisay-legends-toolbox` (audit/t6/sisay-legends-toolbox/SPEC.md). Contract: see `t6_plans.rs`.
//!
//! The deck ramps hard, lands Sisay on turns 2-4, then uses her {W}{U}{B}{R}{G} ability to put the line's legends onto
//! the battlefield (Derevi + Emiel + Cradle / Bloom Tender, Marvin + Ioreth + a mana legend, Mount Doom to finish).
//! `fetch` is the one hook outside this file: `legends::sisay_pick` asks it which legend Sisay fetches.

use crate::ids::{CardId, PlayerId};
use crate::state::Game;
use crate::tag::Tag;

const DOOM: &str = "Mount Doom";
const CRADLE: &str = "Gaea's Cradle";
const TENDER: &str = "Bloom Tender";
const SISAY: &str = "Sisay, Weatherlight Captain";

fn nm(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

/// a card of this name is on p's battlefield (permanent or land)
fn out(g: &Game, p: PlayerId, name: &str) -> bool {
    let pl = g.player(p);
    pl.perms.iter().any(|&m| {
        let x = g.perm(m);
        !x.phased && x.cd.is_some_and(|c| nm(g, c) == name)
    }) || pl.lands.iter().any(|&l| g.land(l).on_bf && nm(g, g.land(l).card()) == name)
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| nm(g, c) == name)
}

/// on the battlefield or in hand (a card we already have: not worth a tutor)
fn have(g: &Game, p: PlayerId, name: &str) -> bool {
    out(g, p, name) || in_hand(g, p, name)
}

fn creatures(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| !g.perm(m).phased && g.is_creature(m)).count()
}

/// How much the line wants this card (0 = nothing). The deck's lines: Derevi + Emiel + Cradle / Bloom Tender + Mount
/// Doom (Flash Unicorn); Marvin + Ioreth + a legend that taps for mana + Mount Doom (DDR).
fn want(g: &Game, p: PlayerId, name: &str) -> f64 {
    if have(g, p, name) {
        return 0.0;
    }
    let uni = out(g, p, "Derevi, Empyrial Tactician") as u8 + out(g, p, "Emiel the Blessed") as u8;
    let ddr = out(g, p, "Marvin, Murderous Mimic") as u8 + out(g, p, "Ioreth of the Healing House") as u8;
    match name {
        DOOM => 8.0,
        "Derevi, Empyrial Tactician" => 9.0,
        "Emiel the Blessed" => 8.5,
        CRADLE => {
            if creatures(g, p) >= 3 && uni >= 1 {
                8.0
            } else {
                2.0
            }
        }
        "Marvin, Murderous Mimic" => 6.0 + ddr as f64,
        "Ioreth of the Healing House" => 6.0 + ddr as f64,
        "Selvala, Heart of the Wilds" => 5.5,
        "Shang-Chi, Master of Kung Fu" => 4.5,
        "Kinnan, Bonder Prodigy" => 5.0,
        "Teferi, Time Raveler" => 4.0,
        "Tyvar, Jubilant Brawler" => 3.5,
        "Lavinia, Azorius Renegade" | "Esper Sentinel" | "Agatha's Soul Cauldron" => 3.0,
        "Lotho, Corrupt Shirriff" | "Tataru Taru, The Cabbage Merchant" | "Ragavan, Nimble Pilferer" => 2.5,
        "Ertai Resurrected" | "Deadpool, Trading Card" => 2.0,
        _ => 1.0,
    }
}

/// Which legendary permanent card Sisay fetches (MV below `power`; lands count: Mount Doom and Cradle are the
/// finisher and the engine). None = no opinion. A colour the board lacks raises her power, so it is worth a little.
pub fn fetch(g: &Game, p: PlayerId, power: i32) -> Option<CardId> {
    let mut best: Option<(f64, CardId)> = None;
    let mut have_cols = crate::cards::Colors::NONE;
    for &m in &g.player(p).perms {
        let x = g.perm(m);
        if !x.phased && x.cd.is_some_and(|c| nm(g, c) != SISAY && g.db.get(c).tag(Tag::Leg)) {
            have_cols = have_cols.union(crate::engine::values::colors_of(g, m));
        }
    }
    for c in crate::engine::zones::searchable(g, p) {
        let d = g.db.get(c);
        if !d.tag(Tag::Leg) || !(d.perm || d.creature || d.land) || (d.cmc as i32) >= power {
            continue;
        }
        let mut s = want(g, p, &d.name);
        if s <= 0.0 {
            continue;
        }
        s += completes(g, p, &d.name);
        let new = have_cols.union(d.colors).count() as i32 - have_cols.count() as i32;
        if power < 5 {
            s += 0.8 * new.max(0) as f64; // a new colour is +1 power: it opens the next fetch
        }
        if best.is_none_or(|b| s > b.0) {
            best = Some((s, c));
        }
    }
    best.map(|b| b.1)
}

/// cast priority of `c` (0-100 scale like `deck_prio`; 0 or less keeps it in hand); None = no opinion
pub fn prio(g: &Game, p: PlayerId, c: CardId) -> Option<f64> {
    let pl = g.player(p);
    let d = g.db.get(c);
    if pl.is_commander(c) {
        // Sisay on turn 2-3 as soon as the mana is there; she is the tutor
        return Some(if pl.turns >= 2 { 92.0 } else { 0.0 });
    }
    match &*d.name {
        // the cheap tutors find the line's pieces: before the value spells
        "Demonic Tutor" | "Vampiric Tutor" | "Worldly Tutor" | "Enlightened Tutor" | "Eladamri's Call"
        | "Green Sun's Zenith" | "Chord of Calling" => Some(76.0),
        // the draw/tax engines come out early, while they are cheap
        "Rhystic Study" | "Smothering Tithe" | "Mystic Remora" => Some(if pl.turns <= 4 { 72.0 } else { 55.0 }),
        // protection and answers are held, not cast into an empty board
        "Silence" | "Deafening Silence" => Some(0.0),
        _ => None,
    }
}

/// tutor wish list, best first: the line's missing pieces that are in the library
pub fn wish(g: &Game, p: PlayerId) -> Option<Vec<CardId>> {
    let lib = &g.player(p).library;
    let mut v: Vec<(f64, CardId)> = vec![];
    for &c in lib {
        let name = nm(g, c);
        let mut s = want(g, p, name);
        if name == TENDER && !have(g, p, TENDER) {
            // the rainbow dork: ramp, and the Unicorn engine without Cradle
            s = if out(g, p, SISAY) { 6.0 } else { 8.5 };
        }
        if s > 0.0 {
            s += completes(g, p, name);
        }
        if s > 1.0 {
            v.push((s, c));
        }
    }
    if v.is_empty() {
        return None;
    }
    v.sort_by(|a, b| b.0.total_cmp(&a.0));
    Some(v.into_iter().map(|x| x.1).collect())
}

/// Mulligan (spec: "if it makes a mana it is probably good enough"): keep a hand with 1-5 lands and two or more mana
/// makers, or a land, a mana maker and a tutor/Sisay-line piece; ship hands of lands only, no mana, or five lands with
/// nothing else.
pub fn keep(g: &Game, p: PlayerId) -> Option<bool> {
    let hand = &g.player(p).hand;
    // mana by turn 2 from this hand: Mox Diamond needs a second land, Chrome Mox a spare card, Sol Ring a land,
    // a dork is summoning sick
    let m = crate::impls::t6_keep::mana(g, p);
    if m.settle() {
        return Some(true);
    }
    let lands = m.lands;
    let makers = m.n_mana;
    let tutors = hand.iter().filter(|&&c| g.db.get(c).tag(Tag::Tut)).count();
    let spells = hand.len() - lands;
    let ok = if lands == 0 {
        false
    } else if lands >= 5 {
        spells >= 3 && makers >= 1
    } else if lands == 1 {
        m.eff >= m.need_one_lander() && spells >= 3
    } else {
        (m.eff >= 3.0 && spells >= 2) || (m.eff >= 2.0 && (makers >= 1 || tutors >= 1) && spells >= 2)
    };
    Some(ok)
}

/// extra value of passing with mana up this turn: the early turns are for developing
pub fn hold_up(g: &Game, p: PlayerId) -> Option<f64> {
    (g.player(p).turns <= 3).then_some(-2.0)
}

/// go for the combo named `combo` now: no opinion, the shared table check decides
pub fn go_now(g: &Game, p: PlayerId, combo: &str) -> Option<bool> {
    // No opinion (A/B at 600 games: "go whenever the line needs no spell" changed neither tries nor wins; the shared
    // table check stays in charge). `t6_combos_t6::line_needs_cast` is there for a later rule.
    let _ = (g, p, combo);
    None
}

/// the cards still missing from a line (a group counts as done when any of its cards is out or in hand)
fn missing(g: &Game, p: PlayerId, groups: &[&[&'static str]]) -> Vec<&'static str> {
    groups.iter().filter(|grp| !grp.iter().any(|n| have(g, p, n))).map(|grp| grp[0]).collect()
}

/// bonus for a card that brings the nearest line closer: the fewer pieces the line still lacks, the more each is worth
fn completes(g: &Game, p: PlayerId, name: &str) -> f64 {
    let lines: [&[&[&'static str]]; 2] = [
        &[&["Derevi, Empyrial Tactician"], &["Emiel the Blessed"], &[CRADLE, TENDER], &[DOOM]],
        &[
            &["Ioreth of the Healing House"],
            &["Marvin, Murderous Mimic"],
            &["Selvala, Heart of the Wilds", "Shang-Chi, Master of Kung Fu"],
            &[DOOM],
        ],
    ];
    let mut best = 0.0f64;
    for l in lines {
        let m = missing(g, p, l);
        let in_line = l.iter().any(|grp| grp.contains(&name));
        if in_line && !m.is_empty() && l.iter().any(|grp| grp.iter().any(|n| have(g, p, n))) {
            best = best.max(6.0 / m.len() as f64);
        }
    }
    best
}
