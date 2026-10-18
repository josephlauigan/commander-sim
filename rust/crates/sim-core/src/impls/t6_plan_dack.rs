//! Tier 6 plan slot for `dack-fayden-white-stax` (audit/t6/dack-fayden-white-stax/SPEC.md). Contract: see `t6_plans.rs`.
//!
//! The deck is a combo deck with a stax fallback. Dack ({4}{W}{W}) flips three of the seven creatures; the plan:
//! 1. fast mana first (before Trinisphere / Vexing Bauble / Void Mirror, which tax or counter the deck's own free
//!    spells), 2. Silence (two or more opponents could answer) or Orim's Chant (one) BEFORE Dack is cast, 3. Dack only
//!    when the table cannot stop it (protected, or the opponents' open mana and cards are low; the bar drops as the
//!    game ages), otherwise a lock piece chosen against the table (Chalice at the X that hits the opponents' cards,
//!    Cursed Totem against creature decks, Defense Grid against counterspells), 4. banishers and wipes are held for a
//!    real threat, not cast to no purpose (the creatures are the flip's pool).

use crate::ai::t6_table::{archenemy_score, stop_chance};
use crate::engine::cast::spell_locked;
use crate::engine::mana::{cost_of, total_mana};
use crate::ids::{CardId, PlayerId};
use crate::state::Game;
use crate::tag::Tag;

const DACK: &str = "Dack Fayden, Helping Hand";
const SEVEN: [&str; 7] = [
    "Abdel Adrian, Gorion's Ward",
    "All-Fates Stalker",
    "Felidar Guardian",
    "Fiend Hunter",
    "Preston, the Vanisher",
    "Radiant Solar",
    "Werefox Bodyguard",
];
/// the casting-protection spells cast BEFORE Dack
const PRE: [&str; 2] = ["Silence", "Orim's Chant"];
/// fast mana that comes down before any tax piece
const FAST: [&str; 10] = [
    "Sol Ring",
    "Mana Vault",
    "Chrome Mox",
    "Mox Diamond",
    "Mox Opal",
    "Grim Monolith",
    "Basalt Monolith",
    "Arcane Signet",
    "Fellwar Stone",
    "The Mind Stone",
];
/// free / zero-mana plays that Trinisphere, Bauble and Void Mirror would hit
const FREE_PLAYS: [&str; 4] = ["Lotus Petal", "Lion's Eye Diamond", "Mox Diamond", "Chrome Mox"];
/// lands that make white (the deck is mono-white: the commander needs WW)
const WHITE_LANDS: [&str; 9] = [
    "Plains",
    "Snow-Covered Plains",
    "Ancient Den",
    "Command Tower",
    "Emergence Zone",
    "Eiganjo, Seat of the Empire",
    "Remote Farm",
    "Ruins of Trokair",
    "Petrified Hamlet",
];

fn nm(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| nm(g, c) == name)
}

fn hand_card(g: &Game, p: PlayerId, name: &str) -> Option<CardId> {
    g.player(p).hand.iter().copied().find(|&c| nm(g, c) == name)
}

fn out(g: &Game, p: PlayerId, name: &str) -> bool {
    crate::ai::plans::on_bf(g, p, name).is_some()
}

fn is_dack(g: &Game, p: PlayerId, c: CardId) -> bool {
    g.player(p).is_commander(c) && nm(g, c) == DACK
}

/// Dack is waiting in the command zone
fn dack_in_zone(g: &Game, p: PlayerId) -> bool {
    let pl = g.player(p);
    pl.commanders().any(|c| pl.cmd_zone_of(c) && nm(g, c) == DACK)
}

fn dack_cost(g: &Game, p: PlayerId) -> u32 {
    let pl = g.player(p);
    pl.commanders().find(|&c| nm(g, c) == DACK).map_or(6, |c| {
        let (gn, pips) = cost_of(g, p, c);
        gn + pips.len() as u32 + pl.tax_of(c)
    })
}

/// creature cards of the seven still in the library (the flip's pool)
fn pool_left(g: &Game, p: PlayerId) -> usize {
    g.player(p).library.iter().filter(|&&c| SEVEN.contains(&nm(g, c))).count()
}

/// every opponent is under a cast lock this turn (Silence / Chant resolved)
fn protected(g: &Game, p: PlayerId) -> bool {
    let mut any = false;
    for q in g.opps(p) {
        any = true;
        let cmd = g.player(q).cmd;
        if !spell_locked(g, q, cmd) {
            return false;
        }
    }
    any
}

/// opponents who could answer a spell right now (stop chance above 15%)
fn answerers(g: &Game, p: PlayerId) -> usize {
    g.opps(p).filter(|&q| stop_chance(g, q) >= 0.15).count()
}

/// P(the table stops Dack's cast or the first flip's triggers)
fn table_risk(g: &Game, p: PlayerId) -> f64 {
    let mut miss = 1.0;
    for q in g.opps(p) {
        miss *= 1.0 - stop_chance(g, q);
    }
    1.0 - miss
}

/// the pre-cast protection we hold and could pay for on top of Dack this turn
fn pre_cast(g: &Game, p: PlayerId) -> Option<CardId> {
    let want_silence = answerers(g, p) >= 2;
    let name = if want_silence { "Silence" } else { "Orim's Chant" };
    let c = hand_card(g, p, name).or_else(|| PRE.iter().find_map(|n| hand_card(g, p, n)))?;
    let (gn, pips) = cost_of(g, p, c);
    (total_mana(g, p, false) >= dack_cost(g, p) + gn + pips.len() as u32).then_some(c)
}

/// the lowest risk at which an unprotected Dack is cast, by round
fn dack_bar(g: &Game, p: PlayerId) -> f64 {
    let mut bar: f64 = match g.round {
        0..=2 => 0.3,
        3 => 0.4,
        4 => 0.5,
        5 => 0.65,
        _ => 1.0,
    };
    if pool_left(g, p) >= 7 {
        bar -= 0.1; // the flip is a long shot with every creature still in the library
    }
    // someone else is about to win: waiting is worse
    if g.opps(p).map(|q| archenemy_score(g, q)).fold(0.0, f64::max) >= 6.0 {
        bar = 1.0;
    }
    bar
}

/// the net count of nonland cards at some X the opponents hold or will draw, less ours: how good Chalice is
fn chalice_net(g: &Game, p: PlayerId) -> i32 {
    let count = |q: PlayerId, x: i32| {
        let pl = g.player(q);
        pl.hand.iter().chain(pl.library.iter()).filter(|&&c| !g.db.get(c).land && g.db.get(c).cmc as i32 == x).count()
            as i32
    };
    (1..=3).map(|x| g.opps(p).map(|q| count(q, x)).sum::<i32>() - count(p, x)).max().unwrap_or(0)
}

/// opponents' decks lean on creatures (share of creature cards, or a creature engine in play)
fn creature_table(g: &Game, p: PlayerId) -> bool {
    let mut engine = false;
    let mut share = 0.0;
    let mut n = 0.0;
    for q in g.opps(p) {
        let pl = g.player(q);
        share +=
            pl.deck_names.iter().filter(|&&c| g.db.get(c).creature).count() as f64 / pl.deck_names.len().max(1) as f64;
        n += 1.0;
        engine |=
            pl.perms.iter().any(|&m| g.is_creature(m) && g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Dork)));
    }
    engine || (n > 0.0 && share / n >= 0.2)
}

fn opp_creatures(g: &Game, q: PlayerId) -> usize {
    g.player(q).perms.iter().filter(|&&m| !g.perm(m).phased && g.is_creature(m)).count()
}

fn counters_likely(g: &Game, p: PlayerId) -> bool {
    1.0 - g.opps(p).map(|q| 1.0 - crate::ai::brain::prob_holding(g, q, |c| g.db.get(c).tag(Tag::Ctr))).product::<f64>()
        >= 0.4
}

/// free-spell decks at the table (Force, Pacts, Misstep, Swan Song): what Void Mirror and Bauble hit
fn free_spell_table(g: &Game, p: PlayerId) -> bool {
    g.opps(p).any(|q| {
        g.player(q)
            .deck_names
            .iter()
            .filter(|&&c| {
                let d = g.db.get(c);
                d.tag(Tag::Free) || d.tag(Tag::Pact) || d.tag(Tag::Fon) || d.tag(Tag::Misstep)
            })
            .count()
            >= 4
    })
}

fn free_plays_in_hand(g: &Game, p: PlayerId) -> bool {
    FREE_PLAYS.iter().any(|n| in_hand(g, p, n)) || in_hand(g, p, "Sol Ring")
}

/// cast priority of `c` (0-100 scale like `deck_prio`; 0 or less keeps it in hand); None = no opinion
pub fn prio(g: &Game, p: PlayerId, c: CardId) -> Option<f64> {
    let n = nm(g, c);
    let pl = g.player(p);
    if is_dack(g, p, c) {
        if protected(g, p) {
            return Some(94.0);
        }
        if pre_cast(g, p).is_some() {
            return Some(0.0); // Silence / Chant first, then Dack
        }
        let prot_left =
            (pl.hand.iter().filter(|&&x| matches!(nm(g, x), "Angel's Grace" | "Reprieve" | "Bilbo's Gambit")).count()
                as f64)
                .min(2.0);
        let risk = table_risk(g, p) * 0.8f64.powf(prot_left);
        return Some(if risk <= dack_bar(g, p) { 92.0 } else { 0.0 });
    }
    match n {
        // protection before the cast: while Dack is waiting, with the mana for both
        "Silence" | "Orim's Chant" => {
            if dack_in_zone(g, p) && !protected(g, p) && pre_cast(g, p).is_some_and(|x| nm(g, x) == n) {
                Some(96.0)
            } else {
                Some(0.0)
            }
        }
        "Angel's Grace" | "Abeyance" => Some(0.0), // held (Grace is cast by the lethal-damage check)
        _ if FAST.contains(&n) => Some(90.0),
        // taxes that hit our own cheap plays come after the fast mana and the protection
        "Trinisphere" => {
            if free_plays_in_hand(g, p) || FAST.iter().any(|f| in_hand(g, p, f)) || pre_cast(g, p).is_some() {
                Some(0.0)
            } else if dack_in_zone(g, p) && !in_hand(g, p, "Silence") && !in_hand(g, p, "Orim's Chant") {
                Some(66.0)
            } else {
                Some(48.0)
            }
        }
        "Vexing Bauble" | "Void Mirror" => {
            if free_plays_in_hand(g, p) {
                Some(0.0)
            } else if free_spell_table(g, p) {
                Some(64.0)
            } else {
                Some(28.0)
            }
        }
        "Chalice of the Void" => {
            let net = chalice_net(g, p);
            Some(if net >= 3 {
                if g.round <= 4 { 80.0 } else { 56.0 }
            } else if net >= 1 && g.round >= 4 {
                30.0
            } else {
                0.0
            })
        }
        "Cursed Totem" => Some(if creature_table(g, p) { 74.0 } else { 22.0 }),
        "Defense Grid" => Some(if counters_likely(g, p) { 72.0 } else { 34.0 }),
        "Deafening Silence" => {
            // one noncreature spell each: not on a turn that holds the protection
            if dack_in_zone(g, p) && (in_hand(g, p, "Silence") || in_hand(g, p, "Orim's Chant")) {
                Some(0.0)
            } else {
                Some(66.0)
            }
        }
        "Smothering Tithe" => Some(if pl.turns <= 4 { 70.0 } else { 52.0 }),
        // the banishers are removal: held for a real threat (they are also the flip's pool)
        "Fiend Hunter" | "All-Fates Stalker" => {
            let threat = g.opps(p).any(|q| archenemy_score(g, q) >= 3.0 || opp_creatures(g, q) >= 3);
            Some(if threat || g.round >= 6 { 68.0 } else { 0.0 })
        }
        // resets: only against a board that is clearly ahead (donated creatures included)
        "Vanquish the Horde" | "Sunscour" => {
            let theirs: usize = g.opps(p).map(|q| opp_creatures(g, q)).sum();
            let mine = opp_creatures(g, p);
            Some(if theirs >= 5 && mine <= 1 { 60.0 } else { 0.0 })
        }
        _ => None,
    }
}

/// tutor wish list (Enlightened Tutor: artifacts and enchantments), best first
pub fn wish(g: &Game, p: PlayerId) -> Option<Vec<CardId>> {
    let mana = total_mana(g, p, false);
    let mut w: Vec<&str> = vec![];
    if mana < dack_cost(g, p) {
        w.extend(["Sol Ring", "Mana Vault", "Grim Monolith", "Chrome Mox"]);
    }
    if dack_in_zone(g, p) && !in_hand(g, p, "Silence") && !in_hand(g, p, "Orim's Chant") {
        w.push("Deafening Silence"); // the closest to protection the tutor can reach
    }
    if chalice_net(g, p) >= 3 {
        w.push("Chalice of the Void");
    }
    if counters_likely(g, p) {
        w.push("Defense Grid");
    }
    if creature_table(g, p) {
        w.push("Cursed Totem");
    }
    w.extend([
        "Deafening Silence",
        "Defense Grid",
        "Chalice of the Void",
        "Smothering Tithe",
        "Trinisphere",
        "Static Prison",
        "Cursed Totem",
        "Void Mirror",
        "Sol Ring",
        "Mana Vault",
    ]);
    let ids: Vec<CardId> = w.iter().filter(|n| !in_hand(g, p, n) && !out(g, p, n)).filter_map(|n| g.db.id(n)).collect();
    Some(ids)
}

/// a mana source that makes white (or can, for the deck's artifact mana)
fn white_source(g: &Game, c: CardId) -> bool {
    let n = nm(g, c);
    WHITE_LANDS.contains(&n)
        || matches!(
            n,
            "Chrome Mox"
                | "Mox Diamond"
                | "Arcane Signet"
                | "Fellwar Stone"
                | "Lotus Petal"
                | "Pearl Medallion"
                | "Mox Opal"
                | "Ancient Tomb"
                | "City of Traitors"
        ) && !g.db.get(c).creature
}

/// Mulligan: keep a hand that makes about 6 mana with white by turn 3-4 and has something to do. Two lands plus fast
/// mana, or Ancient Tomb / City of Traitors with a mox, keep; no white source, all-land and all-free-mana hands ship.
pub fn keep(g: &Game, p: PlayerId) -> Option<bool> {
    let hand = &g.player(p).hand;
    // mana by turn 2 from this hand (Mox Diamond needs a second land, Chrome Mox a spare card, Sol Ring a land)
    let m = crate::impls::t6_keep::mana(g, p);
    let white = hand.iter().any(|&c| white_source(g, c));
    if m.lands == 0 || m.lands >= 6 || m.spells == 0 || (!white && m.eff < 6.0) {
        return Some(false);
    }
    // a 1-lander lives on its rocks: land + at least 2.5 more mana from the hand, and something to cast with it
    if m.lands == 1 {
        return Some(white && m.eff >= m.need_one_lander() && m.spells >= 2);
    }
    // three mana, or two with cheap plays to make with them (protection, removal, a lock piece)
    let cheap =
        hand.iter().filter(|&&c| !g.db.get(c).land && g.db.get(c).cmc <= 3 && !FAST.contains(&nm(g, c))).count();
    Some(m.eff >= 3.0 || (m.eff >= 2.0 && cheap >= 3))
}

/// extra value of passing with mana up: the first turns are for developing; a held protection spell with Dack
/// waiting but not yet affordable keeps the mana
pub fn hold_up(g: &Game, p: PlayerId) -> Option<f64> {
    if g.player(p).turns <= 3 {
        return Some(-2.0);
    }
    None
}

/// Dack's first flip has resolved, the pieces are donated and the line is in the table: go (no reason to wait).
pub fn go_now(_g: &Game, _p: PlayerId, combo: &str) -> Option<bool> {
    combo.starts_with("Dack Fayden: the first flip").then_some(true)
}
