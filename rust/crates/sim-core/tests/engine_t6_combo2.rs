//! Engine feature `combo2`: the Dack, Sisay and Rograkh & Thrasios combos (impls/t6_combos_t6.rs; the specs' "Combo
//! tests": Dack 3 and 4, Sisay 1/2/3/7/8, Rograkh A, B, E and F). Seat 0 plays the combo deck; seat 1 holds the answers.

use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::combos;
use sim_core::state::Game;
use sim_core::sym::intern;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl(key: &str) -> Game {
    let mut g = table(&[key, "seph"]);
    g.active = Some(P0);
    g
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn go(g: &mut Game, part: &str) {
    g.log = Some(vec![]);
    let i = combos::combo_index(part).unwrap_or_else(|| panic!("no combo {part}"));
    assert!(combos::ready(g, P0, i), "{part} is ready");
    combos::attempt(g, P0, i as i64).unwrap();
    if std::env::var("COMBO_LOG").is_ok() {
        eprintln!("{}", g.log.clone().unwrap_or_default().join("\n"));
    }
}

fn ready(g: &Game, part: &str) -> bool {
    combos::ready(g, P0, combos::combo_index(part).unwrap_or_else(|| panic!("no combo {part}")))
}

/// the card in p's library (moved there from the hand or made up), so that a tutor-like step finds it
fn to_library(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = card(g, name);
    g.player_mut(p).hand.retain(|&x| x != c);
    if !g.player(p).library.contains(&c) {
        g.player_mut(p).library.push(c);
    }
    c
}

/// over a few seeds, with a Counterspell for the opponent: whether the AI counters is a roll; when it did, nobody won
/// (the combo stopped at the spell). Returns how many runs were countered.
fn countered_runs(key: &str, part: &str, setup: fn(&mut Game)) -> usize {
    let mut countered = 0;
    for seed in 0..16u64 {
        let mut g = table_with(&[key, "seph"], seed, 40);
        g.active = Some(P0);
        setup(&mut g);
        lands(&mut g, P1, "Island", 2, false);
        hand(&mut g, P1, &["Counterspell"]);
        go(&mut g, part);
        if g.player(P1).gy.contains(&card(&g, "Counterspell")) {
            countered += 1;
            assert_eq!(g.winner, None, "{part}: countered, so the combo stopped");
        }
    }
    countered
}

// ============================================================ Dack: Preston + Felidar Guardian + Radiant Solar
const DACK: &str = "dack-fayden-white-stax";
const DACK_COMBO: &str = "Felidar Guardian + Radiant Solar (flicker";

/// the three pieces, entered in an order that starts no loop (Felidar first: nothing to flicker; Preston last)
fn dack_board(g: &mut Game) {
    perm(g, P0, "Felidar Guardian");
    perm(g, P0, "Radiant Solar");
    perm(g, P0, "Preston, the Vanisher");
    lands(g, P0, "Plains", 2, false);
}

#[test]
fn spec_3_a_flicker_spell_on_preston_felidar_and_solar_wins() {
    for spell in ["Cloudshift", "Ephemerate"] {
        let mut g = tbl(DACK);
        dack_board(&mut g);
        assert!(!ready(&g, DACK_COMBO), "no flicker spell yet");
        hand(&mut g, P0, &[spell]);
        go(&mut g, DACK_COMBO);
        assert_eq!(g.winner, Some(P0), "{spell}");
        assert!(g.player(P0).gy.contains(&card(&g, spell)));
    }
}

#[test]
fn a_missing_piece_means_no_combo() {
    let mut g = tbl(DACK);
    perm(&mut g, P0, "Felidar Guardian");
    perm(&mut g, P0, "Preston, the Vanisher");
    lands(&mut g, P0, "Plains", 2, false);
    hand(&mut g, P0, &["Cloudshift"]);
    assert!(!ready(&g, DACK_COMBO), "Radiant Solar is missing");
    perm(&mut g, P0, "Radiant Solar");
    assert!(ready(&g, DACK_COMBO));
}

#[test]
fn spec_4_swords_on_preston_stops_the_loop() {
    let mut stopped = 0;
    for seed in 0..8u64 {
        let mut g = table_with(&[DACK, "seph"], seed, 40);
        g.active = Some(P0);
        dack_board(&mut g);
        hand(&mut g, P0, &["Cloudshift"]);
        lands(&mut g, P1, "Plains", 1, false);
        hand(&mut g, P1, &["Swords to Plowshares"]);
        go(&mut g, DACK_COMBO);
        if g.winner.is_none() {
            stopped += 1;
            assert!(
                on_bf(&g, P0, "Preston, the Vanisher").is_none()
                    || on_bf(&g, P0, "Felidar Guardian").is_none()
                    || on_bf(&g, P0, "Radiant Solar").is_none(),
                "a key piece is gone"
            );
        }
    }
    assert!(stopped >= 6, "instant-speed removal on a piece stops the combo ({stopped}/8)");
}

#[test]
fn a_counterspell_on_the_flicker_spell_stops_the_combo() {
    let n = countered_runs(DACK, DACK_COMBO, |g| {
        dack_board(g);
        hand(g, P0, &["Cloudshift"]);
    });
    assert!(n >= 1, "the AI counters the flicker spell sometimes");
}

#[test]
fn the_loop_that_hit_the_engines_cap_is_a_combo_without_a_spell() {
    let mut g = tbl(DACK);
    dack_board(&mut g);
    assert!(!ready(&g, "(loop running)"), "the cap was not hit");
    let stamp = (g.round as f64) * 8.0 + 0.0;
    g.player_mut(P0).grudge.insert(intern("preston_cap_round"), stamp);
    g.player_mut(P0).grudge.insert(intern("preston_cap_n"), 31.0);
    go(&mut g, "(loop running)");
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn the_preston_cap_still_stops_the_real_loop() {
    // the real engine loop (no combo): Preston + an uncast Felidar copies until the cap, no stack overflow
    let mut g = tbl(DACK);
    perm(&mut g, P0, "Felidar Guardian");
    perm(&mut g, P0, "Radiant Solar");
    perm(&mut g, P0, "Preston, the Vanisher");
    let fel = on_bf(&g, P0, "Felidar Guardian").unwrap();
    let cd = g.perm(fel).cd.unwrap();
    sim_core::engine::zones::enter_token_copy(&mut g, P0, cd).unwrap();
    let n = g.player(P0).grudge.get(&intern("preston_cap_n")).copied().unwrap_or(0.0);
    assert!(n >= 1.0, "Preston copied at least once");
}

// ============================================================ Sisay
const SISAY: &str = "sisay-legends-toolbox";

fn doom(g: &mut Game) {
    lands(g, P0, "Mount Doom", 1, false);
}

fn ddr_board(g: &mut Game) {
    perm(g, P0, "Ioreth of the Healing House");
    perm(g, P0, "Marvin, Murderous Mimic");
    perm(g, P0, "Selvala, Heart of the Wilds");
    doom(g);
}

#[test]
fn spec_3_marvin_ioreth_and_selvala_win_with_mount_doom() {
    let mut g = tbl(SISAY);
    ddr_board(&mut g);
    go(&mut g, "(DDR)");
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn ddr_needs_pieces_free_of_summoning_sickness_or_a_haste_source() {
    let mut g = tbl(SISAY);
    ddr_board(&mut g);
    let marvin = on_bf(&g, P0, "Marvin, Murderous Mimic").unwrap();
    g.perm_mut(marvin).sick = true;
    assert!(!ready(&g, "(DDR)"), "Marvin is summoning sick");
    perm(&mut g, P0, "Shang-Chi, Master of Kung Fu");
    assert!(ready(&g, "(DDR)"), "Shang-Chi lets abilities ignore summoning sickness");
}

#[test]
fn spec_8_ddr_cast_the_same_turn_with_shang_chi_and_vitality() {
    let mut g = tbl(SISAY);
    perm(&mut g, P0, "Shang-Chi, Master of Kung Fu");
    perm(&mut g, P0, "Enduring Vitality");
    doom(&mut g);
    hand(&mut g, P0, &["Ioreth of the Healing House", "Marvin, Murderous Mimic"]);
    assert!(!ready(&g, "(DDR)"), "5 mana is missing");
    lands(&mut g, P0, "Island", 5, false);
    // Shang-Chi is the legend that taps (Vitality makes his mana unrestricted)
    go(&mut g, "(DDR)");
    assert_eq!(g.winner, Some(P0));
    assert!(
        on_bf(&g, P0, "Ioreth of the Healing House").is_some() && on_bf(&g, P0, "Marvin, Murderous Mimic").is_some()
    );
}

#[test]
fn sisay_fetches_ioreth_and_marvin_without_tapping_ioreth_for_the_second_fetch() {
    // seed 500012, game 12: Relic of Legends taps a legendary creature for mana, and paying for
    // the Marvin fetch tapped the Ioreth just fetched; the line then stopped as "no longer ready"
    let mut g = tbl(SISAY);
    perm(&mut g, P0, "Sisay, Weatherlight Captain");
    perm(&mut g, P0, "Tyvar, Jubilant Brawler");
    perm(&mut g, P0, "Derevi, Empyrial Tactician");
    perm(&mut g, P0, "Relic of Legends");
    doom(&mut g);
    lands(&mut g, P0, "City of Brass", 8, false);
    to_library(&mut g, P0, "Ioreth of the Healing House");
    to_library(&mut g, P0, "Marvin, Murderous Mimic");
    go(&mut g, "(DDR)");
    assert_eq!(g.winner, Some(P0), "both fetched, Ioreth untapped: the loop runs");
}

#[test]
fn a_kept_piece_is_no_mana_source() {
    let mut g = tbl(SISAY);
    perm(&mut g, P0, "Birds of Paradise");
    let birds = on_bf(&g, P0, "Birds of Paradise").unwrap();
    assert!(sim_core::engine::mana::can_pay(&g, P0, 0, "G", false), "Birds taps for {{G}}");
    g.keep_untapped = vec![birds];
    assert!(!sim_core::engine::mana::can_pay(&g, P0, 0, "G", false), "kept untapped for a combo step");
}

#[test]
fn sisay_fetches_mount_doom_as_a_land_and_the_ddr_line_wins() {
    // seed 500012, game 12: a fetched Mount Doom entered as a permanent, not a land, and the line stopped
    let mut g = tbl(SISAY);
    ddr_board(&mut g);
    let doom = g.player(P0).lands.iter().copied().find(|&l| &*g.db.get(g.perm(l).card()).name == "Mount Doom").unwrap();
    sim_core::engine::turn::remove_land(&mut g, P0, doom);
    perm(&mut g, P0, "Sisay, Weatherlight Captain");
    perm(&mut g, P0, "Tyvar, Jubilant Brawler");
    perm(&mut g, P0, "Derevi, Empyrial Tactician");
    lands(&mut g, P0, "City of Brass", 5, false);
    to_library(&mut g, P0, "Mount Doom");
    go(&mut g, "(DDR)");
    assert_eq!(g.winner, Some(P0), "Mount Doom fetched onto the battlefield as a land");
}

#[test]
fn ddr_waits_when_a_fetched_ioreth_would_enter_tapped() {
    // seed 500036, game 36: Blind Obedience; the line fetched Ioreth tapped and stopped
    let mut g = tbl(SISAY);
    perm(&mut g, P0, "Sisay, Weatherlight Captain");
    perm(&mut g, P0, "Tyvar, Jubilant Brawler");
    perm(&mut g, P0, "Derevi, Empyrial Tactician");
    perm(&mut g, P0, "Marvin, Murderous Mimic");
    perm(&mut g, P0, "Selvala, Heart of the Wilds");
    doom(&mut g);
    lands(&mut g, P0, "City of Brass", 5, false);
    to_library(&mut g, P0, "Ioreth of the Healing House");
    assert!(ready(&g, "(DDR)"), "Sisay fetches Ioreth");
    perm(&mut g, P1, "Blind Obedience");
    assert!(!ready(&g, "(DDR)"), "Ioreth would enter tapped");
}

#[test]
fn ddr_without_a_legend_that_taps_for_mana_is_no_combo() {
    let mut g = tbl(SISAY);
    perm(&mut g, P0, "Ioreth of the Healing House");
    perm(&mut g, P0, "Marvin, Murderous Mimic");
    perm(&mut g, P0, "Shang-Chi, Master of Kung Fu"); // his mana only pays creature abilities
    doom(&mut g);
    assert!(!ready(&g, "(DDR)"));
}

#[test]
fn removal_on_ioreth_stops_ddr() {
    let mut stopped = 0;
    for seed in 0..8u64 {
        let mut g = table_with(&[SISAY, "seph"], seed, 40);
        g.active = Some(P0);
        ddr_board(&mut g);
        lands(&mut g, P1, "Plains", 1, false);
        hand(&mut g, P1, &["Swords to Plowshares"]);
        go(&mut g, "(DDR)");
        if g.winner.is_none() {
            stopped += 1;
        }
    }
    assert!(stopped >= 6, "instant-speed removal on a piece stops the loop ({stopped}/8)");
}

#[test]
fn a_counterspell_on_a_cast_piece_stops_ddr() {
    let n = countered_runs(SISAY, "(DDR)", |g| {
        perm(g, P0, "Marvin, Murderous Mimic");
        perm(g, P0, "Selvala, Heart of the Wilds");
        perm(g, P0, "Shang-Chi, Master of Kung Fu");
        doom(g);
        hand(g, P0, &["Ioreth of the Healing House"]);
        lands(g, P0, "Island", 3, false);
    });
    assert!(n >= 1, "the AI counters Ioreth sometimes");
}

fn unicorn_board(g: &mut Game) {
    perm(g, P0, "Derevi, Empyrial Tactician");
    perm(g, P0, "Emiel the Blessed");
    token(g, P0, 1);
    token(g, P0, 1);
    lands(g, P0, "Gaea's Cradle", 1, false);
    doom(g);
}

#[test]
fn spec_1_and_7_derevi_emiel_and_cradle_win_with_mount_doom() {
    let mut g = tbl(SISAY);
    unicorn_board(&mut g);
    go(&mut g, "Flash Unicorn");
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn flash_unicorn_needs_four_creatures_for_cradle() {
    let mut g = tbl(SISAY);
    perm(&mut g, P0, "Derevi, Empyrial Tactician");
    perm(&mut g, P0, "Emiel the Blessed");
    token(&mut g, P0, 1);
    lands(&mut g, P0, "Gaea's Cradle", 1, false);
    doom(&mut g);
    assert!(!ready(&g, "Flash Unicorn"), "3 G a tap pays the {{3}} blink with nothing over");
    token(&mut g, P0, 1);
    assert!(ready(&g, "Flash Unicorn"));
}

#[test]
fn flash_unicorn_casts_derevi_and_emiel_from_hand() {
    let mut g = tbl(SISAY);
    token(&mut g, P0, 1);
    token(&mut g, P0, 1);
    lands(&mut g, P0, "Gaea's Cradle", 1, false);
    doom(&mut g);
    hand(&mut g, P0, &["Derevi, Empyrial Tactician", "Emiel the Blessed"]);
    lands(&mut g, P0, "Plains", 4, false);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Forest", 4, false);
    assert!(ready(&g, "Flash Unicorn"));
    go(&mut g, "Flash Unicorn");
    assert_eq!(g.winner, Some(P0));
    assert!(on_bf(&g, P0, "Derevi, Empyrial Tactician").is_some() && on_bf(&g, P0, "Emiel the Blessed").is_some());
}

#[test]
fn ertai_counters_the_derevi_trigger_of_the_unicorn_loop() {
    let mut g = tbl(SISAY);
    unicorn_board(&mut g);
    lands(&mut g, P1, "Island", 2, false);
    lands(&mut g, P1, "Swamp", 2, false);
    hand(&mut g, P1, &["Ertai Resurrected"]);
    go(&mut g, "Flash Unicorn");
    assert_eq!(g.winner, None, "the ability answer path stopped the loop");
    assert!(on_bf(&g, P1, "Ertai Resurrected").is_some());
}

#[test]
fn the_finisher_must_be_there_mount_doom_missing_means_no_combo() {
    let mut g = tbl(SISAY);
    perm(&mut g, P0, "Derevi, Empyrial Tactician");
    perm(&mut g, P0, "Emiel the Blessed");
    token(&mut g, P0, 1);
    token(&mut g, P0, 1);
    lands(&mut g, P0, "Gaea's Cradle", 1, false);
    assert!(!ready(&g, "Flash Unicorn"));
    hand(&mut g, P0, &["Mount Doom"]);
    assert!(ready(&g, "Flash Unicorn"), "Mount Doom in hand with the land drop open");
}

// ============================================================ Rograkh & Thrasios
const ROG: &str = "rograkh-thrasios-creature-combo";
const PLAN_A: &str = "Talon Gates of Madara";
const PLAN_B: &str = "Eternal Witness + Snap";

/// Rograkh, Thrasios, Birds, a token, Oboro and a sixth creature, Cradle, Island, Gates in hand, Finale in the library
fn plan_a_board(g: &mut Game) {
    perm(g, P0, "Rograkh, Son of Rohgahh");
    perm(g, P0, "Thrasios, Triton Hero");
    perm(g, P0, "Birds of Paradise");
    token(g, P0, 1);
    token(g, P0, 1);
    perm(g, P0, "Oboro Breezecaller");
    lands(g, P0, "Gaea's Cradle", 1, false);
    lands(g, P0, "Island", 1, false);
    lands(g, P0, "Forest", 5, false);
    hand(g, P0, &["Talon Gates of Madara"]);
    to_library(g, P0, "Finale of Devastation");
}

#[test]
fn rograkh_plan_a_oboro_cradle_and_gates_win_through_thrasios_and_finale() {
    let mut g = tbl(ROG);
    plan_a_board(&mut g);
    // opposing creatures phase out with the loop
    let blocker = token(&mut g, P1, 3);
    go(&mut g, PLAN_A);
    assert_eq!(g.winner, Some(P0));
    assert!(g.perm(blocker).phased, "Talon Gates phased out the table's creatures");
    assert!(g.player(P0).gy.contains(&card(&g, "Finale of Devastation")), "Thrasios found Finale and it was cast");
    assert!(on_bf(&g, P0, "Talon Gates of Madara").is_none(), "Gates is a land, not a permanent card on perms");
}

#[test]
fn rograkh_plan_a_needs_six_creatures() {
    let mut g = tbl(ROG);
    plan_a_board(&mut g);
    let toks: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token).collect();
    sim_core::engine::zones::die(&mut g, toks[0], "destroy").unwrap();
    assert!(!ready(&g, PLAN_A), "five creatures make Cradle break even at best");
}

#[test]
fn rograkh_plan_a_casts_oboro_from_hand_and_a_counterspell_stops_it() {
    let mut g = tbl(ROG);
    plan_a_board(&mut g);
    let oboro = on_bf(&g, P0, "Oboro Breezecaller").unwrap();
    sim_core::engine::zones::die(&mut g, oboro, "destroy").unwrap();
    let ob = card(&g, "Oboro Breezecaller");
    g.player_mut(P0).gy.retain(|&c| c != ob);
    hand(&mut g, P0, &["Oboro Breezecaller"]);
    assert!(ready(&g, PLAN_A), "Oboro is cast first: six creatures after it");
    lands(&mut g, P1, "Island", 2, false);
    hand(&mut g, P1, &["Counterspell"]);
    go(&mut g, PLAN_A);
    assert_eq!(g.winner, None, "Oboro Breezecaller was countered");
}

#[test]
fn removal_on_oboro_stops_plan_a() {
    let mut stopped = 0;
    for seed in 0..8u64 {
        let mut g = table_with(&[ROG, "seph"], seed, 40);
        g.active = Some(P0);
        plan_a_board(&mut g);
        lands(&mut g, P1, "Plains", 1, false);
        hand(&mut g, P1, &["Swords to Plowshares"]);
        go(&mut g, PLAN_A);
        if g.winner.is_none() {
            stopped += 1;
        }
    }
    assert!(stopped >= 6, "removal on Oboro or a creature breaks the loop ({stopped}/8)");
}

#[test]
fn rograkh_plan_a_finishes_with_faerie_mastermind_and_green_suns_zenith() {
    let mut g = tbl(ROG);
    plan_a_board(&mut g);
    for n in ["Finale of Devastation", "Faerie Mastermind", "Green Sun's Zenith"] {
        let c = card(&g, n);
        g.player_mut(P0).library.retain(|&x| x != c);
        g.player_mut(P0).hand.retain(|&x| x != c);
    }
    assert!(!ready(&g, PLAN_A), "no finisher in reach");
    to_library(&mut g, P0, "Faerie Mastermind");
    to_library(&mut g, P0, "Green Sun's Zenith");
    go(&mut g, PLAN_A);
    assert_eq!(g.winner, Some(P0));
    assert!(on_bf(&g, P0, "Faerie Mastermind").is_some(), "Mastermind was cast and its draw ability used");
}

#[test]
fn a_counterspell_on_finale_stops_the_finish() {
    let n = countered_runs(ROG, PLAN_A, plan_a_board);
    assert!(n >= 1, "the AI counters Finale sometimes");
}

fn plan_b_board(g: &mut Game) {
    perm(g, P0, "Rograkh, Son of Rohgahh");
    perm(g, P0, "Thrasios, Triton Hero");
    perm(g, P0, "Birds of Paradise");
    token(g, P0, 1);
    token(g, P0, 1);
    lands(g, P0, "Gaea's Cradle", 1, false);
    lands(g, P0, "Island", 2, false);
    lands(g, P0, "Forest", 3, false);
    hand(g, P0, &["Eternal Witness", "Snap"]);
    to_library(g, P0, "Finale of Devastation");
}

#[test]
fn spec_b_eternal_witness_snap_and_cradle_win() {
    let mut g = tbl(ROG);
    plan_b_board(&mut g);
    go(&mut g, PLAN_B);
    assert_eq!(g.winner, Some(P0));
    assert!(g.player(P0).gy.contains(&card(&g, "Snap")) || g.player(P0).hand.contains(&card(&g, "Snap")));
}

#[test]
fn witness_and_snap_need_cradle_at_five_and_the_pieces() {
    let mut g = tbl(ROG);
    plan_b_board(&mut g);
    let t = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token).unwrap();
    sim_core::engine::zones::die(&mut g, t, "destroy").unwrap();
    assert!(!ready(&g, PLAN_B), "four creatures: Cradle is too small");
    token(&mut g, P0, 1);
    assert!(ready(&g, PLAN_B));
    let snap = card(&g, "Snap");
    g.player_mut(P0).hand.retain(|&c| c != snap);
    assert!(!ready(&g, PLAN_B), "Snap is missing");
}

#[test]
fn a_counterspell_on_witness_or_snap_stops_the_loop() {
    let n = countered_runs(ROG, PLAN_B, plan_b_board);
    assert!(n >= 1, "the AI counters a spell of the loop sometimes");
}

// --- t6 rograkh-thrasios: the line B starts and the Thrasios finish from the command zone
/// Witness already on the battlefield (cast earlier for value) and Snap in hand: Snap it first, recast it, the loop runs.
/// The Witness out does not count for Cradle (the loop bounces it), so five other creatures are needed.
#[test]
fn line_b_starts_with_witness_on_the_battlefield_and_snap_in_hand() {
    let mut g = tbl(ROG);
    plan_b_board(&mut g);
    let w = card(&g, "Eternal Witness");
    g.player_mut(P0).hand.retain(|&c| c != w);
    perm(&mut g, P0, "Eternal Witness");
    assert!(ready(&g, PLAN_B), "Witness out, Snap in hand, five other creatures");
    go(&mut g, PLAN_B);
    assert_eq!(g.winner, Some(P0));

    let mut g = tbl(ROG);
    plan_b_board(&mut g);
    let w = card(&g, "Eternal Witness");
    g.player_mut(P0).hand.retain(|&c| c != w);
    perm(&mut g, P0, "Eternal Witness");
    let t = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token).unwrap();
    sim_core::engine::zones::die(&mut g, t, "destroy").unwrap();
    assert!(!ready(&g, PLAN_B), "four creatures besides Witness: Cradle is too small once Witness is bounced");
}

/// Witness in hand and Snap in the graveyard: Witness's enter trigger returns Snap, then the loop runs
#[test]
fn line_b_starts_with_snap_in_the_graveyard() {
    let mut g = tbl(ROG);
    plan_b_board(&mut g);
    let s = card(&g, "Snap");
    g.player_mut(P0).hand.retain(|&c| c != s);
    g.player_mut(P0).gy.push(s);
    hand(&mut g, P1, &[]);
    assert!(ready(&g, PLAN_B), "Witness in hand returns Snap");
    go(&mut g, PLAN_B);
    assert_eq!(g.winner, Some(P0));
    let mut g = tbl(ROG);
    plan_b_board(&mut g);
    let s = card(&g, "Snap");
    g.player_mut(P0).hand.retain(|&c| c != s);
    assert!(!ready(&g, PLAN_B), "Snap neither in hand nor in the graveyard");
}

/// Thrasios in the command zone: on p's main phase the unbounded mana casts it (whatever the tax) and it draws the
/// library for the finisher; off p's turn it can't be cast, so the line is not ready
#[test]
fn thrasios_in_the_command_zone_is_cast_by_the_loop_on_the_main_phase() {
    let mut g = tbl(ROG);
    plan_a_board(&mut g);
    let th = on_bf(&g, P0, "Thrasios, Triton Hero").unwrap();
    sim_core::engine::zones::die(&mut g, th, "destroy").unwrap();
    let thc = card(&g, "Thrasios, Triton Hero");
    g.player_mut(P0).gy.retain(|&c| c != thc);
    g.player_mut(P0).set_cmd_zone(thc, true);
    g.player_mut(P0).add_tax(thc);
    g.player_mut(P0).add_tax(thc);
    token(&mut g, P0, 1); // Thrasios was one of the six creatures
    assert!(g.player(P0).cmds_in_zone().any(|c| c == thc));
    assert!(ready(&g, PLAN_A), "Thrasios in the command zone, own main phase");
    g.active = Some(P1);
    assert!(!ready(&g, PLAN_A), "not p's turn: Thrasios can't be cast");
    g.active = Some(P0);
    go(&mut g, PLAN_A);
    assert_eq!(g.winner, Some(P0));
    assert!(on_bf(&g, P0, "Thrasios, Triton Hero").is_some(), "Thrasios was cast from the command zone");
}

/// Finale of Devastation in hand finishes without Thrasios (the unbounded mana pays X of 10 or more)
#[test]
fn finale_in_hand_needs_no_thrasios() {
    let mut g = tbl(ROG);
    plan_a_board(&mut g);
    let th = on_bf(&g, P0, "Thrasios, Triton Hero").unwrap();
    sim_core::engine::zones::die(&mut g, th, "destroy").unwrap();
    let thc = card(&g, "Thrasios, Triton Hero");
    g.player_mut(P0).gy.retain(|&c| c != thc);
    g.player_mut(P0).set_cmd_zone(thc, false);
    g.player_mut(P0).exile.push(thc);
    token(&mut g, P0, 1);
    assert!(!ready(&g, PLAN_A), "no Thrasios and Finale in the library: no finisher");
    let f = card(&g, "Finale of Devastation");
    g.player_mut(P0).library.retain(|&c| c != f);
    g.player_mut(P0).hand.push(f);
    assert!(ready(&g, PLAN_A), "Finale in hand");
    go(&mut g, PLAN_A);
    assert_eq!(g.winner, Some(P0));
}

/// Training Grounds makes Oboro's ability {1}: Cradle at 5 already nets mana with Talon Gates
#[test]
fn plan_a_with_training_grounds_needs_five_creatures() {
    let mut g = tbl(ROG);
    plan_a_board(&mut g);
    let t = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token).unwrap();
    sim_core::engine::zones::die(&mut g, t, "destroy").unwrap();
    assert!(!ready(&g, PLAN_A), "five creatures, Oboro at {{2}}");
    perm(&mut g, P0, "Training Grounds");
    assert!(ready(&g, PLAN_A), "five creatures, Oboro at {{1}}");
    go(&mut g, PLAN_A);
    assert_eq!(g.winner, Some(P0));
}

/// the speed measurement sees a quiet line one tutor away: Snap in hand, Eternal Witness in the library,
/// Finale of Devastation in hand (a tutor the line itself does not use)
#[test]
fn a_quiet_line_one_tutor_away_is_measured() {
    let mut g = tbl(ROG);
    plan_b_board(&mut g);
    to_library(&mut g, P0, "Eternal Witness");
    hand(&mut g, P0, &["Finale of Devastation"]);
    assert!(!ready(&g, PLAN_B));
    g.player_mut(P0).t6 = true;
    combos::combo_options(&mut g, P0, 0.0, Some(true)).unwrap();
    let st = &g.player(P0).stats;
    let keys: Vec<String> = st.keys().map(|k| k.to_string()).collect();
    assert!(st.contains_key(&intern("sp|reach")), "one tutor away was marked: {keys:?}");
    assert!(!st.contains_key(&intern("sp|ready")));
}

/// Green Sun's Zenith for X = 3 puts Eternal Witness in (SPEC scenario 19's chain): line B with Witness in the library
#[test]
fn green_suns_zenith_puts_witness_in_for_line_b() {
    let mut g = tbl(ROG);
    plan_b_board(&mut g);
    to_library(&mut g, P0, "Eternal Witness");
    assert!(!ready(&g, PLAN_B), "no Witness and no tutor");
    let gsz = hand(&mut g, P0, &["Green Sun's Zenith"])[0];
    lands(&mut g, P0, "Forest", 2, false);
    assert!(ready(&g, PLAN_B), "GSZ (4) and Snap (2) are paid first");
    go(&mut g, PLAN_B);
    assert_eq!(g.winner, Some(P0));
    assert!(!g.player(P0).gy.contains(&gsz), "Green Sun's Zenith shuffles itself back");
}

/// Chord of Calling for X = 2 puts Oboro in; Green Sun's Zenith can't (Oboro is blue)
#[test]
fn chord_of_calling_puts_oboro_in_for_plan_a_and_gsz_does_not() {
    let mut g = tbl(ROG);
    plan_a_board(&mut g);
    let oboro = on_bf(&g, P0, "Oboro Breezecaller").unwrap();
    sim_core::engine::zones::die(&mut g, oboro, "destroy").unwrap();
    let ob = card(&g, "Oboro Breezecaller");
    g.player_mut(P0).gy.retain(|&c| c != ob);
    to_library(&mut g, P0, "Oboro Breezecaller");
    lands(&mut g, P0, "Forest", 5, false);
    hand(&mut g, P0, &["Green Sun's Zenith"]);
    assert!(!ready(&g, PLAN_A), "Green Sun's Zenith finds green creatures only");
    hand(&mut g, P0, &["Chord of Calling"]);
    assert!(ready(&g, PLAN_A), "Chord of Calling X = 2 finds Oboro");
    go(&mut g, PLAN_A);
    assert_eq!(g.winner, Some(P0));
    assert!(on_bf(&g, P0, "Oboro Breezecaller").is_some());
}
