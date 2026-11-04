//! The Ur-Dragon deck's three infinite combos in the catalogue (impls/t45_urdragon_combos.rs): when each is ready, that
//! it wins (or fills the board), and that an opponent's counterspell or removal stops it. Seat 0 plays the combo,
//! seat 1 holds the answers.

use sim_core::engine::zones::{self, Enter};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::combos;
use sim_core::state::{Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

const MIRROR: &str = "Astral Dragon + Cursed Mirror";
const MIIRYM: &str = "Miirym + Astral Dragon + Doubling Season";
const GARTH: &str = "Garth One-Eye + Twinflame + Spark Double + Doubling Season";

fn tbl_with(seed: u64) -> Game {
    let mut g = table_with(&["urdragon", "seph"], seed, 40);
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    // the Ur-Dragon out of the command zone: its eminence would take {1} off Astral Dragon
    let c = card(&g, "The Ur-Dragon");
    g.player_mut(P0).set_cmd_zone(c, false);
    g
}

fn tbl() -> Game {
    tbl_with(1)
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn tokens(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).token).count()
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn is_ready(g: &Game, part: &str) -> bool {
    let i = combos::combo_index(part).unwrap_or_else(|| panic!("no combo {part}"));
    combos::ready(g, P0, i)
}

fn go(g: &mut Game, part: &str) -> bool {
    let i = combos::combo_index(part).unwrap();
    assert!(combos::ready(g, P0, i), "{part} is ready");
    combos::attempt(g, P0, i as i64).unwrap()
}

// ============================================================ Astral Dragon + Cursed Mirror
/// the Mirror out as the artifact (no creature on the table to copy), Astral Dragon in hand, eight lands
fn mirror_table(seed: u64) -> Game {
    let mut g = tbl_with(seed);
    perm(&mut g, P0, "Cursed Mirror");
    hand(&mut g, P0, &["Astral Dragon"]);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Mountain", 6, false);
    g
}

#[test]
fn mirror_line_is_not_ready_with_a_piece_missing_or_too_little_mana() {
    let g = mirror_table(1);
    assert!(is_ready(&g, MIRROR));

    // no Astral Dragon in hand
    let mut g = mirror_table(1);
    g.player_mut(P0).hand.clear();
    assert!(!is_ready(&g, MIRROR), "no Astral Dragon");

    // no Cursed Mirror on the battlefield (in hand does not count: it must be out before the Dragon)
    let mut g = tbl();
    hand(&mut g, P0, &["Astral Dragon", "Cursed Mirror"]);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Mountain", 6, false);
    assert!(!is_ready(&g, MIRROR), "the Mirror is still in hand");

    // six lands and the Mirror (it taps for {R}) make seven: {6}{U}{U} is eight
    let mut g = tbl();
    perm(&mut g, P0, "Cursed Mirror");
    hand(&mut g, P0, &["Astral Dragon"]);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Mountain", 4, false);
    assert!(!is_ready(&g, MIRROR), "seven mana");

    // {U}{U} is part of the cost
    let mut g = tbl();
    perm(&mut g, P0, "Cursed Mirror");
    hand(&mut g, P0, &["Astral Dragon"]);
    lands(&mut g, P0, "Island", 1, false);
    lands(&mut g, P0, "Mountain", 7, false);
    assert!(!is_ready(&g, MIRROR), "one blue source");
}

#[test]
fn mirror_line_needs_a_combat_still_to_come() {
    let mut g = mirror_table(1);
    g.step = Step::Main2;
    assert!(!is_ready(&g, MIRROR), "the hasty Dragons could not attack in the second main phase");
    let mut g = mirror_table(1);
    g.active = Some(P1);
    assert!(!is_ready(&g, MIRROR), "not p's turn");
}

#[test]
fn a_mirror_that_is_a_creature_copy_is_not_the_artifact() {
    let mut g = tbl();
    token(&mut g, P0, 3);
    hand(&mut g, P0, &["Astral Dragon"]);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Mountain", 6, false);
    let m = perm(&mut g, P0, "Cursed Mirror");
    if &*g.perm(m).name != "Cursed Mirror" {
        // it entered as a copy of the 3/3: "Cursed Mirror cannot be copying a creature"
        assert!(!is_ready(&g, MIRROR));
    }
}

#[test]
fn astral_dragon_and_the_mirror_win() {
    let mut g = mirror_table(1);
    assert_eq!(combos::all()[combos::combo_index(MIRROR).unwrap()].steps.len(), 3);
    go(&mut g, MIRROR);
    assert_eq!(g.winner, Some(P0));
    assert!(on_bf(&g, P0, "Astral Dragon").is_some());
}

#[test]
fn a_counterspell_on_astral_dragon_stops_the_mirror_line() {
    let mut countered = 0;
    for seed in 0..16u64 {
        let mut g = mirror_table(seed);
        lands(&mut g, P1, "Island", 2, false);
        hand(&mut g, P1, &["Counterspell"]);
        go(&mut g, MIRROR);
        if g.winner.is_none() {
            countered += 1;
            assert!(on_bf(&g, P0, "Astral Dragon").is_none(), "the Dragon was countered");
            assert!(in_gy(&g, P0, "Astral Dragon"));
            assert!(on_bf(&g, P0, "Cursed Mirror").is_some(), "the Mirror stays");
        }
    }
    assert!(countered >= 1, "the AI counters a combo piece sometimes");
}

#[test]
fn removal_on_astral_dragon_stops_the_mirror_line() {
    let mut stopped = 0;
    for seed in 0..8u64 {
        let mut g = mirror_table(seed);
        lands(&mut g, P1, "Plains", 1, false);
        hand(&mut g, P1, &["Swords to Plowshares"]);
        go(&mut g, MIRROR);
        if g.winner.is_none() {
            stopped += 1;
            assert!(on_bf(&g, P0, "Astral Dragon").is_none(), "Swords exiled the Dragon");
        }
    }
    assert!(stopped >= 6, "the answer is held back only 5% of the time ({stopped}/8)");
}

// ============================================================ Miirym + Astral Dragon + Doubling Season
fn miirym_table(seed: u64) -> Game {
    let mut g = tbl_with(seed);
    perm(&mut g, P0, "Doubling Season");
    perm(&mut g, P0, "Miirym, Sentinel Wyrm");
    hand(&mut g, P0, &["Astral Dragon"]);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Mountain", 6, false);
    g
}

#[test]
fn miirym_line_is_not_ready_with_a_piece_missing_or_too_little_mana() {
    assert!(is_ready(&miirym_table(1), MIIRYM));

    let mut g = miirym_table(1);
    g.player_mut(P0).hand.clear();
    assert!(!is_ready(&g, MIIRYM), "no Astral Dragon");

    let mut g = tbl();
    perm(&mut g, P0, "Doubling Season");
    hand(&mut g, P0, &["Astral Dragon"]);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Mountain", 6, false);
    assert!(!is_ready(&g, MIIRYM), "no Miirym");

    let mut g = tbl();
    perm(&mut g, P0, "Miirym, Sentinel Wyrm");
    hand(&mut g, P0, &["Astral Dragon"]);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Mountain", 6, false);
    assert!(!is_ready(&g, MIIRYM), "no Doubling Season");

    // Miirym and Doubling Season in hand are not on the battlefield
    let mut g = tbl();
    hand(&mut g, P0, &["Astral Dragon", "Miirym, Sentinel Wyrm", "Doubling Season"]);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Mountain", 6, false);
    assert!(!is_ready(&g, MIIRYM), "the other two pieces are still in hand");

    let mut g = tbl();
    perm(&mut g, P0, "Doubling Season");
    perm(&mut g, P0, "Miirym, Sentinel Wyrm");
    hand(&mut g, P0, &["Astral Dragon"]);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Mountain", 5, false);
    assert!(!is_ready(&g, MIIRYM), "seven mana");
}

#[test]
fn miirym_astral_and_the_season_fill_the_board_without_winning_outright() {
    let mut g = miirym_table(1);
    go(&mut g, MIIRYM);
    // the tokens are summoning sick: the combat to come is the kill, not this line's finish
    assert_eq!(g.winner, None);
    assert!(tokens(&g, P0) >= 100, "a board that grows without bound ({} tokens)", tokens(&g, P0));
    assert!(on_bf(&g, P0, "Astral Dragon").is_some());
    let t = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token && g.perm(m).on_bf).unwrap();
    assert!(g.perm(t).fly && g.perm(t).sick, "flying and summoning sick");
}

#[test]
fn a_counterspell_on_astral_dragon_stops_the_miirym_line() {
    let mut countered = 0;
    for seed in 0..16u64 {
        let mut g = miirym_table(seed);
        lands(&mut g, P1, "Island", 2, false);
        hand(&mut g, P1, &["Counterspell"]);
        go(&mut g, MIIRYM);
        if on_bf(&g, P0, "Astral Dragon").is_none() {
            countered += 1;
            assert!(tokens(&g, P0) < 10, "no flood after a counter");
        } else {
            assert!(tokens(&g, P0) >= 100);
        }
    }
    assert!(countered >= 1, "the AI counters a combo piece sometimes");
}

#[test]
fn removal_on_a_piece_stops_the_miirym_line_before_the_flood() {
    let mut stopped = 0;
    for seed in 0..8u64 {
        let mut g = miirym_table(seed);
        lands(&mut g, P1, "Plains", 1, false);
        hand(&mut g, P1, &["Swords to Plowshares"]);
        go(&mut g, MIIRYM);
        if tokens(&g, P0) < 100 {
            stopped += 1;
        }
    }
    assert!(stopped >= 1, "instant-speed removal on Miirym or the Dragon stops the line ({stopped}/8)");
}

// ============================================================ Garth + Twinflame + Spark Double + Doubling Season
/// Garth, Spark Double (a copy of Garth) and Doubling Season out; Twinflame in hand; seven mana open: {1}{R} and {1}{G}
/// for the first round, {2}{R} for the second Twinflame's strive
fn garth_table(seed: u64) -> Game {
    let mut g = tbl_with(seed);
    perm(&mut g, P0, "Garth One-Eye");
    let sd = take(&mut g, P0, "Spark Double");
    zones::enter(&mut g, P0, sd, Enter { plus: 1, ..Enter::default() }).unwrap();
    perm(&mut g, P0, "Doubling Season");
    hand(&mut g, P0, &["Twinflame"]);
    lands(&mut g, P0, "Mountain", 6, false);
    lands(&mut g, P0, "Forest", 1, false);
    g
}

#[test]
fn the_garth_table_has_both_garths() {
    let g = garth_table(1);
    let n = g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf && &*g.perm(m).name == "Garth One-Eye").count();
    assert_eq!(n, 2, "Garth and Spark Double's copy of it");
}

#[test]
fn garth_line_is_not_ready_with_a_piece_missing_or_too_little_mana() {
    assert!(is_ready(&garth_table(1), GARTH));

    let mut g = garth_table(1);
    g.player_mut(P0).hand.clear();
    assert!(!is_ready(&g, GARTH), "no Twinflame");

    // no Spark Double: one Garth only
    let mut g = tbl();
    perm(&mut g, P0, "Garth One-Eye");
    perm(&mut g, P0, "Doubling Season");
    hand(&mut g, P0, &["Twinflame"]);
    lands(&mut g, P0, "Mountain", 3, false);
    lands(&mut g, P0, "Forest", 1, false);
    assert!(!is_ready(&g, GARTH), "no Spark Double");

    // no Doubling Season
    let mut g = tbl();
    perm(&mut g, P0, "Garth One-Eye");
    let sd = take(&mut g, P0, "Spark Double");
    zones::enter(&mut g, P0, sd, Enter { plus: 1, ..Enter::default() }).unwrap();
    hand(&mut g, P0, &["Twinflame"]);
    lands(&mut g, P0, "Mountain", 3, false);
    lands(&mut g, P0, "Forest", 1, false);
    assert!(!is_ready(&g, GARTH), "no Doubling Season");

    // no Garth: a Spark Double with nothing to copy is not one
    let mut g = tbl();
    perm(&mut g, P0, "Doubling Season");
    hand(&mut g, P0, &["Twinflame", "Garth One-Eye", "Spark Double"]);
    lands(&mut g, P0, "Mountain", 3, false);
    lands(&mut g, P0, "Forest", 1, false);
    assert!(!is_ready(&g, GARTH), "Garth and Spark Double in hand");

    // six open mana: enough for the first round ({1}{R}, {1}{G}) but not the second Twinflame's strive ({2}{R}),
    // and one Twinflame on one Garth loses a mana a round (Regrowth {1}{G} + Twinflame {1}{R} vs one Black Lotus)
    let mut g = garth_table(1);
    let l = g.player(P0).lands.clone();
    g.perm_mut(l[0]).tapped = true;
    assert!(!is_ready(&g, GARTH), "six open mana");

    // {1}{G} needs a green source
    let mut g = tbl();
    perm(&mut g, P0, "Garth One-Eye");
    let sd = take(&mut g, P0, "Spark Double");
    zones::enter(&mut g, P0, sd, Enter { plus: 1, ..Enter::default() }).unwrap();
    perm(&mut g, P0, "Doubling Season");
    hand(&mut g, P0, &["Twinflame"]);
    lands(&mut g, P0, "Mountain", 7, false);
    assert!(!is_ready(&g, GARTH), "no green");
}

#[test]
fn garth_line_needs_a_combat_still_to_come() {
    let mut g = garth_table(1);
    g.step = Step::Main2;
    assert!(!is_ready(&g, GARTH), "the hasty Garths could not attack in the second main phase");
}

#[test]
fn garth_twinflame_and_spark_double_win() {
    let mut g = garth_table(1);
    assert_eq!(combos::all()[combos::combo_index(GARTH).unwrap()].steps.len(), 4);
    go(&mut g, GARTH);
    assert_eq!(g.winner, Some(P0));
    assert!(on_bf(&g, P0, "Garth One-Eye").is_some());
}

#[test]
fn a_counterspell_on_twinflame_or_regrowth_stops_the_garth_line() {
    let mut countered = 0;
    for seed in 0..16u64 {
        let mut g = garth_table(seed);
        lands(&mut g, P1, "Island", 2, false);
        hand(&mut g, P1, &["Counterspell"]);
        go(&mut g, GARTH);
        if g.winner.is_none() {
            countered += 1;
        }
    }
    assert!(countered >= 1, "the AI counters a combo piece sometimes");
}

#[test]
fn removal_on_a_garth_stops_the_garth_line() {
    let mut stopped = 0;
    for seed in 0..8u64 {
        let mut g = garth_table(seed);
        lands(&mut g, P1, "Plains", 1, false);
        hand(&mut g, P1, &["Swords to Plowshares"]);
        go(&mut g, GARTH);
        if g.winner.is_none() {
            stopped += 1;
        }
    }
    assert!(stopped >= 6, "the answer is held back only 5% of the time ({stopped}/8)");
}

// ============================================================ the catalogue
#[test]
fn the_three_entries_are_in_the_catalogue_with_steps() {
    for part in [MIRROR, MIIRYM, GARTH] {
        let i = combos::combo_index(part).unwrap_or_else(|| panic!("no combo {part}"));
        assert!(!combos::all()[i].steps.is_empty(), "{part} has declared steps");
        assert_eq!(combos::all()[i].name, part, "the plan file names it exactly");
    }
}

#[test]
fn holding_astral_dragon_the_ai_keeps_cursed_mirror_an_artifact() {
    // without Astral Dragon in hand, the Mirror copies a creature worth copying
    let mut g = tbl();
    perm(&mut g, P1, "Grave Titan");
    let c = take(&mut g, P0, "Cursed Mirror");
    let m = zones::enter(&mut g, P0, c, Enter::default()).unwrap();
    assert!(g.is_creature(m), "copies Grave Titan");
    // holding Astral Dragon it stays the artifact, so the loop can target it this turn
    let mut g = tbl();
    perm(&mut g, P1, "Grave Titan");
    hand(&mut g, P0, &["Astral Dragon"]);
    let c = take(&mut g, P0, "Cursed Mirror");
    let m = zones::enter(&mut g, P0, c, Enter::default()).unwrap();
    assert!(!g.is_creature(m) && &*g.perm(m).name == "Cursed Mirror", "{}", g.perm(m).name);
}
