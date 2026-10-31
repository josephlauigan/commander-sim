//! Engine feature `dacktest` (plan row 19): a resolved Dack Fayden's first flip, decided from the 35-pile table
//! (impls/t6_combos_t6.rs, "Dack: the first flip"; audit/t6/dack-fayden-white-stax/SPEC.md "First-flip trace").
//! Seat 0 plays the Dack deck, seats 1-3 are the table.

use sim_core::ids::PlayerId;
use sim_core::impls::combos;
use sim_core::impls::t6_combos_t6::{AB, FE, FH, PR, SO, ST, WF, bits, dack_line, pile_number, pile_wins};
use sim_core::impls::t6_dack::SEVEN;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const DACK: &str = "dack-fayden-white-stax";
const DACK_CARD: &str = "Dack Fayden, Helping Hand";
const FLIP: &str = "Dack Fayden: the first flip";

fn code(c: char) -> usize {
    match c {
        'A' => AB,
        'S' => ST,
        'H' => FH,
        'O' => SO,
        'W' => WF,
        'P' => PR,
        'F' => FE,
        _ => panic!("{c}"),
    }
}

// ============================================================ 1. the table against the traces' summaries
/// (pile number, its three creatures, the cards left in the library for which the pile wins without a spell, and with
/// Cloudshift / Ephemerate). Letters: A Abdel, S Stalker, H Fiend Hunter, O Solar, W Werefox, P Preston, F Felidar.
/// "*" = whatever is left. Each row is the summary table of its flips-*.md file.
const TRACES: [(u32, &str, &str, &str); 35] = [
    // flips-a-preston-felidar.md: all five WIN-NOW
    (1, "PFA", "*", "*"),
    (2, "PFS", "*", "*"),
    (3, "PFH", "*", "*"),
    (4, "PFO", "*", "*"),
    (5, "PFW", "*", "*"),
    // flips-b1-felidar.md: 3/4 (win unless Preston is left), 10 and 14 1/4 (only Solar left; 2/4 with a spell:
    // Solar or Abdel left), 12 2/4 (Abdel or Solar left; no change with a spell)
    (6, "FAS", "HOW", "HOW"),
    (7, "FAH", "SOW", "SOW"),
    (8, "FAO", "SHW", "SHW"),
    (9, "FAW", "SHO", "SHO"),
    (10, "FSH", "O", "OA"),
    (11, "FSO", "AHW", "AHW"),
    (12, "FSW", "AO", "AO"),
    (13, "FHO", "ASW", "ASW"),
    (14, "FHW", "O", "OA"),
    (15, "FOW", "ASH", "ASH"),
    // flips-b2a-preston.md: 16, 17, 19, 20 WIN-NOW; 18 NO-WIN (spell: wins when Felidar is not the one left, "not fully traced")
    (16, "PAS", "*", "*"),
    (17, "PAH", "*", "*"),
    (18, "PAO", "", "SHW"),
    (19, "PAW", "*", "*"),
    (20, "PSH", "*", "*"),
    // flips-b2b-preston.md: 22, 24 WIN-NOW; 23 WIN+1 (any left); 21 and 25 NO-WIN (spell: Abdel or Fiend Hunter left)
    (21, "PSO", "", "AH"),
    (22, "PSW", "*", "*"),
    (23, "PHO", "", "*"),
    (24, "PHW", "*", "*"),
    (25, "POW", "", "AH"),
    // flips-c-neither.md: 26-31 3/4 (win unless Preston is left), 32-35 2/4 (Felidar or Abdel left); a spell wins all
    (26, "ASH", "FOW", "*"),
    (27, "ASO", "FHW", "*"),
    (28, "ASW", "FOH", "*"),
    (29, "AHO", "FSW", "*"),
    (30, "AHW", "FOS", "*"),
    (31, "AOW", "FSH", "*"),
    (32, "SHO", "FA", "*"),
    (33, "SHW", "FA", "*"),
    (34, "SOW", "FA", "*"),
    (35, "HOW", "FA", "*"),
];

fn pile_bits(s: &str) -> u32 {
    let v: Vec<usize> = s.chars().map(code).collect();
    bits(v[0], v[1], v[2])
}

/// the four creatures not in the pile
fn rest(pile: u32) -> Vec<usize> {
    (0..7).filter(|i| pile & (1 << i) == 0).collect()
}

fn expect(pile: u32, spec: &str) -> Vec<usize> {
    if spec == "*" { rest(pile) } else { spec.chars().map(code).collect() }
}

#[test]
fn every_one_of_the_35_piles_gets_the_outcome_in_its_trace() {
    let (mut plain, mut with_spell) = (0, 0);
    for &(n, members, no_spell, spell) in &TRACES {
        let pile = pile_bits(members);
        assert_eq!(pile_number(pile), Some(n), "pile {n} {members}: numbering");
        let (want_plain, want_spell) = (expect(pile, no_spell), expect(pile, spell));
        for x in rest(pile) {
            let w = want_plain.contains(&x);
            assert_eq!(pile_wins(n, Some(x), false), w, "pile {n} {members}, {} left, no spell", SEVEN[x]);
            assert_eq!(dack_line(pile, Some(x), false).is_some(), w, "pile {n} line without a spell");
            let ws = want_spell.contains(&x);
            assert_eq!(pile_wins(n, Some(x), true) || w, ws, "pile {n} {members}, {} left, with a spell", SEVEN[x]);
            assert_eq!(dack_line(pile, Some(x), true).map(|l| l.spell), ws.then_some(!w), "pile {n} line with a spell");
            plain += usize::from(w);
            with_spell += usize::from(ws);
        }
    }
    // 23.75 of 35 piles = 95 of 140 (pile, left) cases; the trace's group totals: 5 + 6.25 + 6 + 6.5
    assert_eq!(plain, 95, "23.75 of 35 piles win without a spell");
    assert!(with_spell > plain, "a flicker spell adds wins");
    let group = |lo: u32, hi: u32| {
        TRACES
            .iter()
            .filter(|t| (lo..=hi).contains(&t.0))
            .map(|t| rest(pile_bits(t.1)).iter().filter(|&&x| pile_wins(t.0, Some(x), false)).count())
            .sum::<usize>()
    };
    assert_eq!((group(1, 5), group(6, 15), group(16, 25), group(26, 35)), (20, 25, 24, 26));
}

#[test]
fn the_35_piles_are_all_the_three_creature_sets_of_the_seven() {
    let mut seen = std::collections::BTreeSet::new();
    for a in 0..7 {
        for b in a + 1..7 {
            for c in b + 1..7 {
                let n = pile_number(bits(a, b, c)).expect("numbered");
                assert!((1..=35).contains(&n));
                assert!(seen.insert(n), "pile {n} twice");
            }
        }
    }
    assert_eq!(seen.len(), 35);
}

// ============================================================ 2. end-to-end games
fn four(seed: u64) -> Game {
    let mut g = table_with(&[DACK, "seph", "sheoldred", "yshtola"], seed, 40);
    g.active = Some(P0);
    lands(&mut g, P0, "Plains", 2, false);
    g
}

/// the library: `top_first` creatures (index codes) on top, in that order, then the rest of the seven, then lands
fn stack(g: &mut Game, top_first: &[usize]) {
    let mut order: Vec<usize> = top_first.to_vec();
    order.extend((0..7).filter(|i| !top_first.contains(i)));
    let mut lib = vec![card(g, "Plains"); 3];
    lib.extend(order.iter().rev().map(|&i| card(g, SEVEN[i])));
    g.player_mut(P0).library = lib;
}

/// Dack resolves (his enters trigger flips the pile); then the combo, if it is ready
fn dack_then_combo(g: &mut Game) -> bool {
    perm(g, P0, DACK_CARD);
    let i = combos::combo_index(FLIP).expect("the combo");
    if g.winner.is_some() || !combos::ready(g, P0, i) {
        return false;
    }
    combos::attempt(g, P0, i as i64).unwrap();
    true
}

#[test]
fn a_class_a_pile_wins_the_turn_unopposed() {
    for seed in 0..4 {
        let mut g = four(seed);
        stack(&mut g, &[PR, FE, AB]);
        dack_then_combo(&mut g);
        assert_eq!(g.winner, Some(P0), "seed {seed}: Preston + Felidar + Abdel");
    }
}

#[test]
fn the_combo_sees_the_flip_and_the_card_left_behind() {
    let mut g = four(1);
    stack(&mut g, &[ST, FH, SO]); // pile 32 (Stalker, Fiend Hunter, Solar)
    perm(&mut g, P0, DACK_CARD);
    let i = combos::combo_index(FLIP).unwrap();
    // 32 wins without a spell only when Felidar or Abdel is the one left: the shuffled library decides
    let lib: Vec<_> = g
        .player(P0)
        .library
        .iter()
        .rev()
        .filter(|&&c| g.db.get(c).creature)
        .map(|&c| SEVEN.iter().position(|s| **s == *g.db.get(c).name).unwrap())
        .collect();
    assert_eq!(lib.len(), 4);
    let left = lib[3];
    assert_eq!(combos::ready(&g, P0, i), pile_wins(32, Some(left), false), "ready follows the table ({})", SEVEN[left]);
}

fn swords_stops(pile: &[usize], key_name: &str) -> (usize, usize) {
    let (mut stopped, mut won) = (0, 0);
    for seed in 0..10u64 {
        let mut g = four(seed);
        stack(&mut g, pile);
        for q in [1u8, 2, 3] {
            let q = PlayerId(q);
            lands(&mut g, q, "Plains", 1, false);
            hand(&mut g, q, &["Swords to Plowshares"]);
        }
        if !dack_then_combo(&mut g) {
            continue;
        }
        if g.winner == Some(P0) {
            won += 1;
        } else {
            stopped += 1;
            let gone =
                !g.players.iter().flat_map(|pl| pl.perms.iter()).any(|&m| {
                    g.perm(m).on_bf && !g.perm(m).token && g.perm(m).orig == P0 && &*g.perm(m).name == key_name
                });
            assert!(gone, "seed {seed}: stopped, so {key_name} was removed");
        }
    }
    (stopped, won)
}

#[test]
fn swords_to_plowshares_on_preston_stops_a_class_a_pile() {
    let (stopped, won) = swords_stops(&[PR, FE, AB], "Preston, the Vanisher");
    assert!(stopped >= 7, "Swords on Preston stops the line ({stopped} stopped, {won} won of 10)");
}

#[test]
fn ertai_counters_dacks_enters_trigger_so_nothing_flips() {
    let mut countered = 0;
    for seed in 0..16u64 {
        let mut g = four(seed);
        stack(&mut g, &[PR, FE, AB]);
        lands(&mut g, P1, "Island", 2, false);
        lands(&mut g, P1, "Swamp", 2, false);
        hand(&mut g, P1, &["Ertai Resurrected"]);
        g.log = Some(vec![]);
        let ran = dack_then_combo(&mut g);
        let log = g.log.clone().unwrap_or_default().join("\n");
        if log.contains("is countered") && !log.contains("Dack reveals") {
            countered += 1;
            assert!(!ran && g.winner.is_none(), "seed {seed}: no flip, no combo");
        }
    }
    assert!(countered >= 1, "Ertai counters the trigger sometimes ({countered}/16)");
}

// ============================================================ 3. Monte Carlo over random libraries
#[test]
fn fresh_dack_resolutions_win_the_same_turn_about_68_percent_of_the_time() {
    let mut state: u64 = 1017;
    let n = 2000;
    let (mut wins, mut by_combo, mut ready_no_win) = (0, 0, 0);
    let mut by_class = [(0u32, 0u32); 4];
    for t in 0..n {
        let mut order: Vec<usize> = (0..7).collect();
        for k in (1..7).rev() {
            state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            order.swap(k, ((state >> 33) % (k as u64 + 1)) as usize);
        }
        let mut g = four(t as u64);
        stack(&mut g, &order);
        let pile = bits(order[0], order[1], order[2]);
        let pn = pile_number(pile).unwrap();
        let class = match pn {
            1..=5 => 0,
            6..=15 => 1,
            16..=25 => 2,
            _ => 3,
        };
        let early = {
            perm(&mut g, P0, DACK_CARD);
            g.winner.is_some()
        };
        let i = combos::combo_index(FLIP).unwrap();
        let mut won = early;
        if !early && combos::ready(&g, P0, i) {
            combos::attempt(&mut g, P0, i as i64).unwrap();
            won = g.winner == Some(P0);
            by_combo += usize::from(won);
            ready_no_win += usize::from(!won);
        }
        by_class[class].0 += u32::from(won);
        by_class[class].1 += 1;
        wins += usize::from(won);
    }
    let rate = wins as f64 / n as f64;
    eprintln!(
        "Dack first flip, {n} fresh resolutions: {wins} same-turn wins = {:.1}% ({by_combo} by the shortcut, {} before it, {ready_no_win} ready but not won); per pile group (a, b1, b2, c) wins and flips: {:?}",
        rate * 100.0,
        wins - by_combo,
        by_class
    );
    assert!((0.58..=0.78).contains(&rate), "same-turn win rate {rate:.3}, the trace says ~0.68");
}

// ============================================================ 4. a flicker spell in hand
#[test]
fn a_flicker_spell_wins_pile_23_that_loses_without_one() {
    for spell in ["Cloudshift", "Ephemerate"] {
        let mut g = four(3);
        stack(&mut g, &[PR, FH, SO]); // pile 23: Preston + Fiend Hunter + Solar
        assert!(!dack_then_combo(&mut g), "no spell: pile 23 is not a win");
        assert_eq!(g.winner, None);

        let mut g = four(3);
        stack(&mut g, &[PR, FH, SO]);
        hand(&mut g, P0, &[spell]);
        assert!(dack_then_combo(&mut g), "{spell}: the combo is ready");
        assert_eq!(g.winner, Some(P0), "{spell}");
        assert!(g.player(P0).gy.contains(&card(&g, spell)), "the spell was cast");
    }
}

// ============================================================ 5. the traces cover a full library only (dack-check, 2026-10-31)
/// the library: `top_first` creatures on top, then the rest of the seven but `missing`, then lands; `missing` goes to
/// the graveyard
fn stack_without(g: &mut Game, top_first: &[usize], missing: usize) {
    stack(g, top_first);
    let gone = card(g, SEVEN[missing]);
    g.player_mut(P0).library.retain(|&c| c != gone);
    g.player_mut(P0).gy.push(gone);
}

/// Every trace (flips-*.md) assumes the four creatures not in the pile are all still in the library: the second flip
/// takes them. With one of them gone the "wins whatever the second flip is" piles have no traced line (pile 19,
/// Preston + Abdel + Werefox, loops Felidar through Preston with a Solar token: no Felidar in the library, no loop),
/// so the shortcut must not call the turn won.
#[test]
fn a_pile_is_not_a_win_when_a_piece_its_trace_flips_next_is_not_in_the_library() {
    for seed in 0..4 {
        // pile 19 with all four others in the library: the table's win
        let mut g = four(seed);
        stack(&mut g, &[PR, AB, WF]);
        assert!(dack_then_combo(&mut g), "seed {seed}: full library, the shortcut is ready");
        assert_eq!(g.winner, Some(P0), "seed {seed}: pile 19 wins with the four others to flip");

        // the same pile with Felidar Guardian in the graveyard: no traced line, no shortcut win
        for missing in [FE, SO] {
            let mut g = four(seed);
            stack_without(&mut g, &[PR, AB, WF], missing);
            assert!(!dack_then_combo(&mut g), "seed {seed}: {} not in the library, no shortcut", SEVEN[missing]);
            assert_eq!(g.winner, None, "seed {seed}: {} gone", SEVEN[missing]);
        }
    }
}
