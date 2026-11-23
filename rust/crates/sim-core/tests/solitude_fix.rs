//! Solitude and The Wandering Emperor's flash, on hand-built positions: an evoke or a flash cast counts as a cast
//! (spells this turn, the `spells_cast` stat, `cast_names`); a creature whose ward the AI cannot pay is no target of
//! the evoke or of the enters trigger; the yoshimaru seat does not pitch a piece of its lines when another white card
//! can pay the evoke. No games are played.

use sim_core::ai::act::perform;
use sim_core::engine::cast::castable;
use sim_core::engine::tutors::card_worth;
use sim_core::engine::values::pval;
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::Game;
use sim_core::sym::intern;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

const YOSHI: &str = "yoshimaru";
const BRAGO: &str = "brago-azorius-blink-control";
const SOL: &str = "Solitude";
const MIIRYM: &str = "Miirym, Sentinel Wyrm";
const SHELOB: &str = "Shelob, Child of Ungoliant"; // ward {2}, worth more than Grave Titan
const BIG: &str = "Grave Titan";
const HELIOD: &str = "Heliod, Sun-Crowned";
const COURT: &str = "Court of Ardenvale";

/// P0 is the `key` seat with Solitude in hand; the opponent holds no cards (nothing counters or answers)
fn tbl(key: &str) -> (Game, CardId) {
    let mut g = table(&[key, "seph"]);
    g.active = Some(P0);
    g.player_mut(P1).hand.clear();
    let s = hand(&mut g, P0, &[SOL])[0];
    (g, s)
}

fn evoke_opts(g: &mut Game, c: CardId) -> Vec<Opt> {
    let f = g.registry.get(c).and_then(|i| i.hand_options).expect("Solitude has hand options");
    f(g, c, P0, None).unwrap()
}

/// the AI's evoke, if it offers one: performed
fn ai_evokes(g: &mut Game, c: CardId) -> bool {
    let opts = evoke_opts(g, c);
    let Some(o) = opts.iter().find(|o| o.label.contains("evoke Solitude")) else { return false };
    perform(g, P0, o.act.as_ref().unwrap()).unwrap()
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn spells_cast(g: &Game, p: PlayerId) -> i64 {
    g.player(p).stats.get(&intern("spells_cast")).copied().unwrap_or(0)
}

fn tapped_lands(g: &Game, p: PlayerId) -> usize {
    g.player(p).lands.iter().filter(|&&l| g.perm(l).tapped).count()
}

/// a creature with ward {2} as the best target, `BIG` (no ward) as the lesser one; the position asserts the order
fn two_targets(g: &mut Game) -> (PermId, PermId) {
    let m = perm(g, P1, SHELOB);
    let b = perm(g, P1, BIG);
    assert!(pval(g, b) >= 6.0, "the other creature is worth an evoke: {}", pval(g, b));
    assert!(pval(g, m) > pval(g, b), "the warded creature ranks first: {} vs {}", pval(g, m), pval(g, b));
    (m, b)
}

// ------------------------------------------------------------------ (b) ward: the evoke
#[test]
fn tapped_out_the_evoke_aims_past_a_warded_creature() {
    for key in [YOSHI, BRAGO] {
        let (mut g, s) = tbl(key);
        let (m, b) = two_targets(&mut g);
        hand(&mut g, P0, &["Swords to Plowshares"]);
        assert!(ai_evokes(&mut g, s), "{key}: the evoke is offered");
        assert!(!g.perm(b).on_bf, "{key}: the unwarded creature is exiled");
        assert!(g.perm(m).on_bf, "{key}: the warded creature is left alone");
    }
}

#[test]
fn tapped_out_with_only_the_warded_creature_no_evoke_is_offered() {
    let (mut g, s) = tbl(YOSHI);
    let m = perm(&mut g, P1, MIIRYM);
    hand(&mut g, P0, &["Swords to Plowshares"]);
    assert!(pval(&g, m) >= 6.0);
    assert!(
        evoke_opts(&mut g, s).is_empty(),
        "{:?}",
        evoke_opts(&mut g, s).iter().map(|o| &o.label).collect::<Vec<_>>()
    );
    assert!(g.player(P0).hand.contains(&s), "nothing was spent");
}

#[test]
fn with_ward_money_open_the_warded_creature_is_still_a_target() {
    let (mut g, s) = tbl(YOSHI);
    let m = perm(&mut g, P1, MIIRYM);
    lands(&mut g, P0, "Plains", 2, false);
    hand(&mut g, P0, &["Swords to Plowshares"]);
    assert!(ai_evokes(&mut g, s));
    assert!(!g.perm(m).on_bf, "the AI paid ward {{2}} and the exile happened");
    assert_eq!(tapped_lands(&g, P0), 2, "the ward was paid from the two lands");
}

#[test]
fn sheltered_by_ghosts_gives_a_ward_the_evoke_respects() {
    let (mut g, s) = tbl(YOSHI);
    let b = perm(&mut g, P1, BIG);
    let a = perm(&mut g, P1, "Sheltered by Ghosts");
    g.perm_mut(a).attached = Some(b);
    if !g.auras.contains(&a) {
        g.auras.push(a);
    }
    assert_eq!(sim_core::cardcode::aura_ward(&g, b), 2);
    hand(&mut g, P0, &["Swords to Plowshares"]);
    assert!(evoke_opts(&mut g, s).is_empty(), "tapped out: ward {{2}} from the Aura cannot be paid");
    lands(&mut g, P0, "Plains", 2, false);
    assert!(ai_evokes(&mut g, s), "two lands open: it can");
    assert!(!g.perm(b).on_bf);
}

// ------------------------------------------------------------------ (b) ward: the enters trigger
#[test]
fn the_enters_trigger_skips_a_creature_whose_ward_it_cannot_pay() {
    for key in [YOSHI, BRAGO] {
        // hard cast or blink: the trigger picks its target as it resolves
        let mut g = table(&[key, "seph"]);
        let (m, b) = two_targets(&mut g);
        perm(&mut g, P0, SOL);
        assert!(!g.perm(b).on_bf && g.perm(m).on_bf, "{key}: tapped out, the unwarded creature goes");
    }
}

#[test]
fn the_enters_trigger_exiles_nothing_when_only_warded_creatures_remain() {
    for key in [YOSHI, BRAGO] {
        let mut g = table(&[key, "seph"]);
        let m = perm(&mut g, P1, MIIRYM);
        let life = g.player(P1).life;
        let sol = perm(&mut g, P0, SOL);
        assert!(g.perm(m).on_bf, "{key}: ward {{2}} cannot be paid");
        assert_eq!(g.player(P1).life, life);
        // a blink: it leaves and enters again, still nothing to take
        let c = g.perm(sol).card();
        sim_core::engine::zones::leave(&mut g, sol).unwrap();
        sim_core::engine::zones::enter(&mut g, P0, c, Default::default()).unwrap();
        assert!(g.perm(m).on_bf, "{key}: the blink exiles nothing either");
    }
}

#[test]
fn the_enters_trigger_pays_ward_when_it_can() {
    let mut g = table(&[YOSHI, "seph"]);
    let m = perm(&mut g, P1, MIIRYM);
    lands(&mut g, P0, "Plains", 2, false);
    perm(&mut g, P0, SOL);
    assert!(!g.perm(m).on_bf, "ward {{2}} paid, Miirym exiled");
    assert_eq!(tapped_lands(&g, P0), 2);
}

// ------------------------------------------------------------------ (c) the pitch
/// which white card the evoke exiled
fn pitched(g: &Game, names: &[&str]) -> Vec<String> {
    names.iter().filter(|n| g.player(P0).exile.contains(&card(g, n))).map(|n| n.to_string()).collect()
}

#[test]
fn the_yoshimaru_seat_pitches_the_card_that_is_not_a_line_piece() {
    let (mut g, s) = tbl(YOSHI);
    perm(&mut g, P1, BIG);
    hand(&mut g, P0, &[HELIOD, COURT]);
    assert!(sim_core::impls::t6_pieces::is_line_piece(&g, P0, HELIOD));
    let (h, c) = (card(&g, HELIOD), card(&g, COURT));
    assert!(
        card_worth(&g, P0, h, false) < card_worth(&g, P0, c, false),
        "the old rule would pitch Heliod ({} vs {})",
        card_worth(&g, P0, h, false),
        card_worth(&g, P0, c, false)
    );
    assert!(ai_evokes(&mut g, s));
    assert_eq!(pitched(&g, &[HELIOD, COURT]), vec![COURT.to_string()]);
}

#[test]
fn the_yoshimaru_seat_with_only_a_line_piece_still_evokes_by_pitching_it() {
    let (mut g, s) = tbl(YOSHI);
    let b = perm(&mut g, P1, BIG);
    hand(&mut g, P0, &[HELIOD]);
    assert!(ai_evokes(&mut g, s));
    assert_eq!(pitched(&g, &[HELIOD, COURT]), vec![HELIOD.to_string()]);
    assert!(!g.perm(b).on_bf);
}

#[test]
fn another_seat_keeps_the_old_pitch_rule() {
    let (mut g, s) = tbl(BRAGO);
    perm(&mut g, P1, BIG);
    hand(&mut g, P0, &[HELIOD, COURT]);
    let (h, c) = (card(&g, HELIOD), card(&g, COURT));
    let (wh, wc) = (card_worth(&g, P0, h, false), card_worth(&g, P0, c, false));
    assert!(ai_evokes(&mut g, s));
    let expected = if wh < wc { HELIOD } else { COURT };
    assert_ne!(wh, wc, "a tie would not show the rule");
    assert_eq!(pitched(&g, &[HELIOD, COURT]), vec![expected.to_string()], "worths {wh} and {wc}");
}

// ------------------------------------------------------------------ (a) the evoke is a cast
#[test]
fn an_evoke_counts_as_a_cast() {
    let (mut g, s) = tbl(YOSHI);
    perm(&mut g, P1, BIG);
    hand(&mut g, P0, &["Swords to Plowshares"]);
    let (spells, stat) = (g.player(P0).spells_this_turn, spells_cast(&g, P0));
    assert!(ai_evokes(&mut g, s));
    assert_eq!(g.player(P0).spells_this_turn, spells + 1);
    assert_eq!(spells_cast(&g, P0), stat + 1);
    assert!(g.player(P0).cast_names.contains(&s));
}

#[test]
fn an_evoke_the_opponent_counters_still_counts_as_a_cast() {
    let mut countered = 0;
    for seed in 1..30u64 {
        let mut g = table_with(&[YOSHI, BRAGO], seed, 40);
        g.active = Some(P0);
        g.player_mut(P1).hand.clear();
        let s = hand(&mut g, P0, &[SOL])[0];
        let big = perm(&mut g, P1, BIG);
        hand(&mut g, P0, &["Swords to Plowshares"]);
        lands(&mut g, P1, "Island", 4, false);
        hand(&mut g, P1, &["Counterspell", "Counterspell"]);
        let (spells, stat) = (g.player(P0).spells_this_turn, spells_cast(&g, P0));
        assert!(ai_evokes(&mut g, s), "seed {seed}");
        if g.perm(big).on_bf {
            countered += 1;
            assert!(!on_bf(&g, P0, SOL), "seed {seed}: countered, so Solitude did not enter");
        }
        assert_eq!(g.player(P0).spells_this_turn, spells + 1, "seed {seed}");
        assert_eq!(spells_cast(&g, P0), stat + 1, "seed {seed}");
        assert!(g.player(P0).cast_names.contains(&s), "seed {seed}");
    }
    assert!(countered > 0, "no seed had the opponent counter the evoke");
}

/// a regression guard: the lock already read `turn_casts` (set by `on_cast`), so this passes on the old code too; it
/// shows that `record_cast` leaves the lock and `casts_this_turn` unchanged
#[test]
fn rule_of_law_locks_the_next_spell_after_an_evoke() {
    let (mut g, s) = tbl(YOSHI);
    perm(&mut g, P1, BIG);
    perm(&mut g, P1, "Rule of Law");
    let swords = hand(&mut g, P0, &["Swords to Plowshares"])[0];
    lands(&mut g, P0, "Plains", 2, false);
    assert!(castable(&g, P0, swords, "hand"), "before the evoke the spell is allowed");
    assert!(ai_evokes(&mut g, s));
    assert!(!castable(&g, P0, swords, "hand"), "one spell each turn: the evoke was it");
    assert_eq!(sim_core::engine::cast::casts_this_turn(&g, P0), 1);
}

/// Solitude and a white card in hand but no opposing creature: no evoke is offered, and asking records nothing
#[test]
fn a_seat_that_evokes_nothing_records_nothing() {
    let (mut g, s) = tbl(YOSHI);
    hand(&mut g, P0, &["Swords to Plowshares"]);
    assert!(evoke_opts(&mut g, s).is_empty(), "no target: no evoke");
    assert!(!ai_evokes(&mut g, s));
    assert_eq!(g.player(P0).spells_this_turn, 0);
    assert_eq!(spells_cast(&g, P0), 0);
    assert!(g.player(P0).cast_names.is_empty());
    assert!(g.player(P0).hand.contains(&s) && g.player(P0).exile.is_empty(), "nothing was spent");
}

// ------------------------------------------------------------------ (a) the Emperor's flash
#[test]
fn the_emperors_flash_counts_as_a_cast() {
    let mut g = table(&["sauron", "atraxa-infect"]);
    let (p, d) = (P0, P1);
    let big = perm(&mut g, p, "Grave Titan");
    g.perm_mut(big).tapped = true;
    let mut atk = vec![big];
    let e = hand(&mut g, d, &["The Wandering Emperor"])[0];
    lands(&mut g, d, "Plains", 4, false);
    g.player_mut(p).hand.clear();
    let (spells, stat) = (g.player(d).spells_this_turn, spells_cast(&g, d));
    let mut assign = vec![];
    sim_core::cardcode::defend_hooks(&mut g, p, &mut atk, d, &mut assign).unwrap();
    assert!(!g.perm(big).on_bf, "it flashed in and exiled the attacker");
    assert_eq!(g.player(d).spells_this_turn, spells + 1);
    assert_eq!(spells_cast(&g, d), stat + 1);
    assert!(g.player(d).cast_names.contains(&e));
}
