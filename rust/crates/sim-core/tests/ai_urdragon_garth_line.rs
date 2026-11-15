//! The Ur-Dragon deck's card-code fixes, guarded to the `urdragon` seat: The Ur-Dragon's free drop never puts Tiamat
//! onto the battlefield (its "if you cast it" search would be lost), Twinflame is held while the Garth line is alive,
//! and Spark Double copies Garth when the rest of the line is near. Every other seat plays as before.
mod play_common;
use play_common::*;
use sim_core::ai::brain::main_options;
use sim_core::engine::zones;
use sim_core::ids::CardId;
use sim_core::impls::t45_urdragon::ur_pick;
use sim_core::rng::Rng;
use sim_core::state::Step;
use std::sync::Arc;

// ------------------------------------------------------------------ the free drop: never Tiamat
fn permanent_cards(g: &Game) -> Vec<CardId> {
    deck("urdragon")
        .cards
        .iter()
        .map(|n| card(g, n))
        .filter(|&c| {
            let d = g.db.get(c);
            d.land || (d.perm && !d.has_subtype("aura"))
        })
        .collect()
}

/// The cards `ur_attack` offers `ur_pick`: lands and non-Aura permanents of the hand.
fn offered(g: &Game, hand: &[CardId]) -> Vec<CardId> {
    hand.iter()
        .copied()
        .filter(|&c| g.db.get(c).land || (g.db.get(c).perm && !g.db.get(c).has_subtype("aura")))
        .collect()
}

#[test]
fn ur_pick_never_returns_tiamat_over_500_seeded_states() {
    let mut rng = Rng::from_u64(20261115);
    let mut seen = 0;
    for i in 0..500 {
        let mut g = table(&["urdragon", "seph"]);
        let tiamat = card(&g, "Tiamat");
        let pool: Vec<CardId> = permanent_cards(&g).into_iter().filter(|&c| c != tiamat).collect();
        let mut hand_cards = vec![tiamat];
        for _ in 0..rng.below(8) {
            hand_cards.push(*rng.choice(&pool).unwrap());
        }
        for _ in 0..rng.below(9) {
            lands(&mut g, P0, "Plains", 1, false);
        }
        // the sampled cards are the seat's real hand, and no longer in its library
        g.player_mut(P0).hand = hand_cards.clone();
        g.player_mut(P0).library.retain(|c| !hand_cards.contains(c));
        assert!(g.player(P0).hand.contains(&tiamat));
        let cs = offered(&g, &g.player(P0).hand.clone());
        let without: Vec<CardId> = cs.iter().copied().filter(|&c| c != tiamat).collect();
        let pick = ur_pick(&g, P0, &cs);
        assert_ne!(pick, Some(tiamat), "state {i}: hand {:?} offered Tiamat", names(&g, &cs));
        assert_eq!(pick, ur_pick(&g, P0, &without), "state {i}: Tiamat changed the pick from {:?}", names(&g, &cs));
        seen += 1;
    }
    assert_eq!(seen, 500);
}

fn names(g: &Game, cs: &[CardId]) -> Vec<String> {
    cs.iter().map(|&c| g.db.get(c).name.to_string()).collect()
}

#[test]
fn ur_pick_with_only_tiamat_picks_nothing_and_with_a_land_picks_the_land() {
    let g = table(&["urdragon", "seph"]);
    let tiamat = card(&g, "Tiamat");
    assert_eq!(ur_pick(&g, P0, &[tiamat]), None);
    let land = card(&g, "Plains");
    assert_eq!(ur_pick(&g, P0, &[tiamat, land]), Some(land));
}

#[test]
fn ur_pick_for_another_seat_still_picks_the_biggest_permanent_tiamat() {
    // guard off: Tiamat is the highest `body_worth` of Tiamat and a one-drop, no land
    let g = table(&["aragorn", "seph"]);
    let (tiamat, elves) = (card(&g, "Tiamat"), card(&g, "Llanowar Elves"));
    assert_eq!(ur_pick(&g, P0, &[tiamat, elves]), Some(tiamat));
}

// ------------------------------------------------------------------ Twinflame held for the Garth line
const GARTH: &str = "Garth One-Eye";
const SPARK: &str = "Spark Double";
const SEASON: &str = "Doubling Season";
const TWIN: &str = "Twinflame";

#[derive(Clone, Copy, Debug, PartialEq)]
enum Z {
    Library,
    Hand,
    Gy,
    Exile,
    Bf,
    Gone,
}

/// out of every zone of p's, then into `z` (the battlefield through `perm`, so its enter effects happen)
fn put(g: &mut Game, p: PlayerId, name: &str, z: Z) {
    let c = card(g, name);
    let pl = g.player_mut(p);
    pl.library.retain(|&x| x != c);
    pl.hand.retain(|&x| x != c);
    pl.gy.retain(|&x| x != c);
    pl.exile.retain(|&x| x != c);
    match z {
        Z::Library => pl.library.push(c),
        Z::Hand => pl.hand.push(c),
        Z::Gy => pl.gy.push(c),
        Z::Exile => pl.exile.push(c),
        Z::Gone => {}
        Z::Bf => {
            perm(g, p, name);
        }
    }
}

/// seat 0 (the key) in its first main phase with Twinflame in hand, the three pieces in the library, an opponent at
/// 40 life with nothing out. Mana and creatures are added by the row.
fn base(key: &str) -> Game {
    let mut g = table(&[key, "seph"]);
    for q in &mut g.players {
        q.hand.clear();
        q.gy.clear();
    }
    for n in [GARTH, SPARK, SEASON] {
        put(&mut g, P0, n, Z::Library);
    }
    put(&mut g, P0, TWIN, Z::Hand);
    g
}

/// The fair target: an untapped, not-sick 8/8 flier (Ancient Silver Dragon). Twinflame's AI only offers a target worth
/// 1.5 or more, and a 4/4 flier with haste (Glorybringer) is worth 1.0, so the creature must be bigger.
const BIG: &str = "Ancient Silver Dragon";
const BIG_POW: i32 = 8;

/// three Mountains and the fair target
fn glory(g: &mut Game) {
    lands(g, P0, "Mountain", 3, false);
    perm(g, P0, BIG);
}

/// the line's pieces on the battlefield (Garth, then Spark Double as a copy of it, then Doubling Season) with seven
/// lands: RR and G
fn ready_board(g: &mut Game, full: bool) {
    put(g, P0, GARTH, Z::Bf);
    put(g, P0, SPARK, Z::Bf);
    put(g, P0, SEASON, Z::Bf);
    let spark = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).phys == Some(card(g, SPARK))).unwrap();
    assert_eq!(&*g.perm(spark).name, GARTH, "Spark Double entered as a copy of Garth");
    let mana: &[(&str, usize)] =
        if full { &[("Mountain", 4), ("Forest", 2), ("Island", 1)] } else { &[("Mountain", 3), ("Forest", 1)] };
    for &(n, k) in mana {
        lands(g, P0, n, k, false);
    }
    perm(g, P0, BIG);
}

/// the options the AI is shown in its main phase that cast Twinflame
fn twin_opts(g: &mut Game, post: bool) -> Vec<String> {
    main_options(g, P0, post)
        .unwrap()
        .into_iter()
        .filter(|o| o.label.starts_with("Twinflame"))
        .map(|o| o.label)
        .collect()
}

fn fair(g: &mut Game, post: bool) {
    let o = twin_opts(g, post);
    assert!(!o.is_empty(), "a fair Twinflame option was expected, none offered");
}

fn held(g: &mut Game, post: bool) {
    let o = twin_opts(g, post);
    assert!(o.is_empty(), "Twinflame should be held, but {o:?} was offered");
}

fn opp_life(g: &mut Game, n: i32) {
    g.player_mut(P1).life = n;
}

fn main2(g: &mut Game) {
    g.step = Step::Main2;
    // a creature with an enter effect, so a second-main-phase Twinflame is worth casting
    perm(g, P0, "Mulldrifter");
}

// row builders: `key` is the seat (urdragon, or another deck for the control)
fn row01(key: &str) -> Game {
    let mut g = base(key);
    glory(&mut g);
    g
}
fn row02(key: &str) -> Game {
    let mut g = base(key);
    ready_board(&mut g, true);
    g
}
fn row03(key: &str) -> Game {
    let mut g = base(key);
    ready_board(&mut g, true);
    main2(&mut g);
    g
}
fn row04(key: &str) -> Game {
    let mut g = base(key);
    ready_board(&mut g, false);
    g
}
fn row07(key: &str) -> Game {
    let mut g = base(key);
    lands(&mut g, P0, "Mountain", 3, false);
    perm(&mut g, P0, BIG);
    put(&mut g, P0, SPARK, Z::Bf); // a copy of the big flier
    put(&mut g, P0, GARTH, Z::Bf);
    g
}
fn row11(key: &str) -> Game {
    let mut g = base(key);
    put(&mut g, P0, GARTH, Z::Hand);
    put(&mut g, P0, SPARK, Z::Hand);
    glory(&mut g);
    g
}
fn row12(key: &str) -> Game {
    let mut g = base(key);
    put(&mut g, P0, GARTH, Z::Bf);
    put(&mut g, P0, SPARK, Z::Bf);
    glory(&mut g);
    g
}
/// lethal: the flier 8 + its copy 8 = 16 against 16 life
fn row13(key: &str) -> Game {
    let mut g = row01(key);
    opp_life(&mut g, 2 * BIG_POW);
    g
}
fn row17(key: &str) -> Game {
    let mut g = base(key);
    put(&mut g, P0, SEASON, Z::Bf);
    glory(&mut g);
    opp_life(&mut g, 3 * BIG_POW); // the flier 8 + two copies 16
    g
}

#[test]
fn twinflame_row_01_all_pieces_in_the_library_is_held() {
    let mut g = row01("urdragon");
    held(&mut g, false);
    let mut c = row01("aragorn");
    fair(&mut c, false); // the same board for another seat offers it
}

#[test]
fn twinflame_row_02_ready_line_offers_no_fair_option_but_the_combo() {
    let mut g = row02("urdragon");
    held(&mut g, false);
    let tw = card(&g, TWIN);
    let lines = sim_core::impls::combos::ready_lines(&g, P0);
    assert!(lines.iter().any(|(_, cs)| cs.contains(&tw)), "the Garth line is ready and uses Twinflame: {lines:?}");
    // the combo option is what makes holding the fair option safe: it must really be offered
    let offers: Vec<String> = main_options(&mut g, P0, false).unwrap().into_iter().map(|o| o.label).collect();
    assert!(
        offers.iter().any(|l| l.starts_with("combo:") && l.contains("Garth One-Eye")),
        "the Garth combo option is offered: {offers:?}"
    );
    let mut c = row02("aragorn");
    fair(&mut c, false);
}

#[test]
fn twinflame_row_03_ready_board_in_the_second_main_phase_is_held() {
    let mut g = row03("urdragon");
    held(&mut g, true);
    let mut c = row03("aragorn");
    fair(&mut c, true);
}

#[test]
fn twinflame_row_04_ready_board_with_four_mana_is_held() {
    let mut g = row04("urdragon");
    held(&mut g, false);
    let mut c = row04("aragorn");
    fair(&mut c, false);
}

#[test]
fn twinflame_rows_05_to_10_a_dead_piece_releases_it() {
    let dead: [(&str, Z); 6] = [
        (GARTH, Z::Gy),     // 5
        (GARTH, Z::Exile),  // 6
        (SPARK, Z::Gy),     // 8
        (SEASON, Z::Gy),    // 9
        (SEASON, Z::Exile), // 10
        (GARTH, Z::Gone),   // (a piece in no zone)
    ];
    for (n, z) in dead {
        let mut g = base("urdragon");
        put(&mut g, P0, n, z);
        glory(&mut g);
        let o = twin_opts(&mut g, false);
        assert!(!o.is_empty(), "{n} in {z:?}: the line is dead, a fair Twinflame was expected");
    }
}

#[test]
fn twinflame_row_07_spark_double_copying_something_else_is_dead() {
    let mut g = row07("urdragon");
    let spark = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).phys == Some(card(&g, SPARK))).unwrap();
    assert_eq!(&*g.perm(spark).name, BIG, "Spark Double is a copy of the big flier, not of Garth");
    fair(&mut g, false);
}

#[test]
fn twinflame_row_11_garth_and_spark_in_hand_is_alive_and_held() {
    let mut g = row11("urdragon");
    held(&mut g, false);
    let mut c = row11("aragorn");
    fair(&mut c, false);
}

#[test]
fn twinflame_row_12_garth_and_spark_out_season_in_library_is_held() {
    let mut g = row12("urdragon");
    held(&mut g, false);
    let mut c = row12("aragorn");
    fair(&mut c, false);
}

#[test]
fn twinflame_row_13_lethal_on_one_opponent_releases_the_hold() {
    let mut g = row13("urdragon");
    fair(&mut g, false);
}

#[test]
fn twinflame_row_14_lethal_numbers_in_the_second_main_phase_are_held() {
    let mut g = row13("urdragon");
    main2(&mut g);
    held(&mut g, true);
    let mut c = row13("aragorn");
    main2(&mut c);
    fair(&mut c, true);
}

#[test]
fn twinflame_row_15_lethal_numbers_but_an_untapped_flier_blocks_are_held() {
    let mut g = row13("urdragon");
    perm(&mut g, P1, "Shivan Dragon");
    held(&mut g, false);
    let mut c = row13("aragorn");
    perm(&mut c, P1, "Shivan Dragon");
    fair(&mut c, false);
}

#[test]
fn twinflame_row_16_one_short_of_lethal_is_held() {
    let mut g = row01("urdragon");
    opp_life(&mut g, 2 * BIG_POW + 1);
    held(&mut g, false);
    let mut c = row01("aragorn");
    opp_life(&mut c, 2 * BIG_POW + 1);
    fair(&mut c, false);
}

#[test]
fn twinflame_row_17_doubling_season_out_doubles_the_copies_for_lethal() {
    let mut g = row17("urdragon");
    fair(&mut g, false);
    let mut c = row17("urdragon");
    opp_life(&mut c, 3 * BIG_POW + 1);
    held(&mut c, false);
}

#[test]
fn twinflame_row_18_a_table_that_can_kill_p_releases_the_hold() {
    let mut g = row01("urdragon");
    g.player_mut(P0).life = 4;
    perm(&mut g, P1, "Shivan Dragon");
    assert!(sim_core::ai::brain::Situation::new(&g, P0).danger >= 1.0);
    fair(&mut g, false);
}

#[test]
fn twinflame_row_19_a_much_better_fair_target_does_not_release_the_hold() {
    let build = |key: &str| {
        let mut g = row01(key);
        for n in ["Mulldrifter", "Shriekmaw", "Solemn Simulacrum", "Ravenous Chupacabra"] {
            perm(&mut g, P0, n);
        }
        g
    };
    let mut g = build("urdragon");
    held(&mut g, false);
    let mut c = build("aragorn");
    fair(&mut c, false);
}

#[test]
fn twinflame_row_20_ready_overrides_lethal() {
    let mut g = row02("urdragon");
    opp_life(&mut g, 2 * BIG_POW);
    held(&mut g, false);
    // twins: at that life total the attack is lethal, so the fair option is offered when the line is dead, and when
    // it is alive but not ready (one land fewer: no RR and G); only readiness holds it back
    let mut dead = row02("urdragon");
    let season = dead.player(P0).perms.iter().copied().find(|&m| dead.perm(m).name == SEASON).unwrap();
    zones::leave(&mut dead, season).unwrap();
    put(&mut dead, P0, SEASON, Z::Gone); // Doubling Season is gone for good: the line is dead
    opp_life(&mut dead, 2 * BIG_POW);
    fair(&mut dead, false);
    let mut unready = row04("urdragon");
    opp_life(&mut unready, 2 * BIG_POW);
    assert!(!sim_core::impls::t45_urdragon_combos::ready_garth(&unready, P0).0);
    fair(&mut unready, false);
}

#[test]
fn twinflame_row_21_other_seats_are_unchanged() {
    for key in ["aragorn", "yshtola", "thanos"] {
        let mut g = row01(key);
        fair(&mut g, false);
    }
}

#[test]
fn twinflame_row_22_a_test_that_placed_only_twinflame_is_dead() {
    let mut g = base("urdragon");
    for n in [GARTH, SPARK, SEASON] {
        put(&mut g, P0, n, Z::Gone);
    }
    glory(&mut g);
    fair(&mut g, false);
}

#[test]
fn twinflame_row_23_a_stolen_garth_is_dead() {
    let mut g = base("urdragon");
    put(&mut g, P0, GARTH, Z::Gone);
    perm(&mut g, P1, GARTH);
    glory(&mut g);
    fair(&mut g, false);
}

// ------------------------------------------------------------------ Spark Double copies Garth when the line is near
const TERROR: &str = "Terror of the Peaks";

/// Spark Double enters for `key` with a plain Garth and Terror of the Peaks (the higher-`pval` alternative) on the
/// battlefield; Doubling Season and Twinflame in the given zones, `hand` extra cards in hand. Returns the game and
/// the name of the creature Spark Double became.
fn spark_case(key: &str, season: Z, twin: Z, hand_extra: &[&str]) -> (Game, String) {
    let mut g = table(&[key, "seph"]);
    for q in &mut g.players {
        q.hand.clear();
        q.gy.clear();
    }
    put(&mut g, P0, GARTH, Z::Bf);
    put(&mut g, P0, TERROR, Z::Bf);
    put(&mut g, P0, SEASON, season);
    put(&mut g, P0, TWIN, twin);
    for n in hand_extra {
        put(&mut g, P0, n, Z::Hand);
    }
    let name = enter_spark(&mut g);
    (g, name)
}

fn enter_spark(g: &mut Game) -> String {
    perm(g, P0, SPARK);
    spark_name(g)
}

fn spark_name(g: &Game) -> String {
    let c = card(g, SPARK);
    let m = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).phys == Some(c)).expect("Spark Double entered");
    g.perm(m).name.to_string()
}

fn garths_on_bf(g: &Game) -> usize {
    g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf && &*g.perm(m).name == GARTH).count()
}

#[test]
fn spark_s01_season_out_twinflame_in_hand_copies_garth() {
    let (g, name) = spark_case("urdragon", Z::Bf, Z::Hand, &[]);
    assert_eq!(name, GARTH);
    assert_eq!(garths_on_bf(&g), 2, "the legend rule removed nothing: both Garths stay");
    let copy = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).phys == Some(card(&g, SPARK))).unwrap();
    assert!(!sim_core::cardcode::is_legendary(&g, copy), "Spark Double's copy of a legend isn't legendary");
}

#[test]
fn spark_s02_season_and_twinflame_in_hand_copies_garth() {
    assert_eq!(spark_case("urdragon", Z::Hand, Z::Hand, &[]).1, GARTH);
}

#[test]
fn spark_s03_season_out_twinflame_in_library_with_demonic_tutor_copies_garth() {
    assert_eq!(spark_case("urdragon", Z::Bf, Z::Library, &["Demonic Tutor"]).1, GARTH);
}

#[test]
fn spark_s04_season_in_library_twinflame_in_hand_with_vampiric_tutor_copies_garth() {
    assert_eq!(spark_case("urdragon", Z::Library, Z::Hand, &["Vampiric Tutor"]).1, GARTH);
}

#[test]
fn spark_s05_twinflame_in_library_without_a_tutor_copies_the_best_creature() {
    assert_eq!(spark_case("urdragon", Z::Bf, Z::Library, &[]).1, TERROR);
    // Worldly Tutor finds only creatures
    assert_eq!(spark_case("urdragon", Z::Bf, Z::Library, &["Worldly Tutor"]).1, TERROR);
}

#[test]
fn spark_s06_both_pieces_in_the_library_with_a_tutor_copies_the_best_creature() {
    assert_eq!(spark_case("urdragon", Z::Library, Z::Library, &["Demonic Tutor"]).1, TERROR);
}

#[test]
fn spark_s07_season_in_the_graveyard_copies_the_best_creature() {
    assert_eq!(spark_case("urdragon", Z::Gy, Z::Hand, &[]).1, TERROR);
}

#[test]
fn spark_s08_twinflame_in_the_graveyard_copies_the_best_creature() {
    assert_eq!(spark_case("urdragon", Z::Bf, Z::Gy, &[]).1, TERROR);
}

#[test]
fn spark_s09_garth_in_hand_is_not_a_candidate() {
    let mut g = table(&["urdragon", "seph"]);
    g.player_mut(P0).hand.clear();
    put(&mut g, P0, GARTH, Z::Hand);
    put(&mut g, P0, TERROR, Z::Bf);
    put(&mut g, P0, SEASON, Z::Bf);
    put(&mut g, P0, TWIN, Z::Hand);
    assert_eq!(enter_spark(&mut g), TERROR);
}

#[test]
fn spark_s10_garth_phased_out_is_not_a_candidate() {
    let mut g = table(&["urdragon", "seph"]);
    g.player_mut(P0).hand.clear();
    let garth = perm(&mut g, P0, GARTH);
    put(&mut g, P0, TERROR, Z::Bf);
    put(&mut g, P0, SEASON, Z::Bf);
    put(&mut g, P0, TWIN, Z::Hand);
    g.perm_mut(garth).phased = true;
    assert_eq!(enter_spark(&mut g), TERROR);
}

#[test]
fn spark_s11_other_seats_keep_the_best_creature() {
    for key in ["aragorn", "atraxa-infect"] {
        assert_eq!(spark_case(key, Z::Bf, Z::Hand, &[]).1, TERROR, "{key}");
    }
}

#[test]
fn spark_s12_a_person_in_the_urdragon_seat_is_asked() {
    let mut g = table(&["urdragon", "seph"]);
    g.player_mut(P0).hand.clear();
    put(&mut g, P0, GARTH, Z::Bf);
    put(&mut g, P0, TERROR, Z::Bf);
    put(&mut g, P0, SEASON, Z::Bf);
    put(&mut g, P0, TWIN, Z::Hand);
    let h = Arc::new(ScriptHuman::from_fn(|r| {
        match r.choices.iter().position(|c| c.contains("Terror")).filter(|_| r.prompt.contains("Spark Double: enter")) {
            Some(i) => Answer::Pick(i),
            None => Answer::Decline, // Terror's own enter trigger and the rest
        }
    }));
    g.humans.add(P0, h.clone());
    let name = enter_spark(&mut g);
    assert!(h.asked().iter().any(|r| r.prompt.contains("Spark Double: enter as a copy")), "the person is asked");
    assert_eq!(name, TERROR, "the person's pick stands");
}

#[test]
fn spark_s13_the_free_drop_copies_garth_too() {
    let mut g = table(&["urdragon", "seph"]);
    g.player_mut(P0).hand.clear();
    put(&mut g, P0, GARTH, Z::Bf);
    put(&mut g, P0, TERROR, Z::Bf);
    put(&mut g, P0, SEASON, Z::Bf);
    put(&mut g, P0, TWIN, Z::Hand);
    let c = take(&mut g, P0, SPARK);
    zones::enter(&mut g, P0, c, zones::Enter::default()).unwrap();
    assert_eq!(spark_name(&g), GARTH);
}

#[test]
fn spark_s14_a_token_garth_is_not_a_candidate_the_card_is() {
    let mut g = table(&["urdragon", "seph"]);
    g.player_mut(P0).hand.clear();
    let garth = perm(&mut g, P0, GARTH);
    put(&mut g, P0, TERROR, Z::Bf);
    put(&mut g, P0, SEASON, Z::Bf);
    put(&mut g, P0, TWIN, Z::Hand);
    let cd = g.perm(garth).cd.unwrap();
    let tok = zones::enter_token_copy(&mut g, P0, cd).unwrap().unwrap();
    assert!(g.perm(tok).token);
    assert_eq!(enter_spark(&mut g), GARTH);
    assert_eq!(garths_on_bf(&g), 3, "card Garth, the token and Spark Double's copy");
}

#[test]
fn twinflame_doubling_season_under_another_players_control_is_dead() {
    let mut g = base("urdragon");
    put(&mut g, P0, SEASON, Z::Gone);
    perm(&mut g, P1, SEASON);
    glory(&mut g);
    fair(&mut g, false);
}

// ------------------------------------------------------------------ round 2: what the lethal release really counts
/// the live tokens of seat 0 with the fair target's name
fn big_tokens(g: &Game) -> usize {
    g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).token && g.perm(m).name == BIG).count()
}

/// cast the offered fair Twinflame ("Twinflame (N targets)") for seat 0
fn cast_fair(g: &mut Game) {
    let opts = main_options(g, P0, false).unwrap();
    let o = opts.iter().find(|o| o.label.starts_with("Twinflame (")).expect("a fair Twinflame option");
    assert!(sim_core::ai::act::perform(g, P0, o.act.as_ref().unwrap()).unwrap());
}

fn tokens_of_p0(g: &Game) -> usize {
    g.player(P0).perms.iter().filter(|&&m| g.perm(m).token).count()
}

// R1: Doubling Season doubles the copies the fair resolution makes
#[test]
fn twinflame_r1_fair_resolution_makes_the_doubled_copies() {
    let mut g = row17("urdragon");
    cast_fair(&mut g);
    assert_eq!(big_tokens(&g), 2, "Doubling Season: two hasty copies of the one target");
}

#[test]
fn twinflame_r1_another_seat_gets_the_doubling_and_nothing_else_changes() {
    // Twinflame is only in the urdragon deck, so another seat reaches the resolver only in a test; its result is the
    // rules' result: doubled under Doubling Season, a single copy without it
    let mut g = row17("aragorn");
    cast_fair(&mut g);
    assert_eq!(big_tokens(&g), 2);
    let mut one = row01("aragorn");
    cast_fair(&mut one);
    assert_eq!(big_tokens(&one), 1);
}

#[test]
fn twinflame_r1_the_copies_stop_at_the_token_cap() {
    // 249 tokens already, then Doubling Season: only one of the two copies fits
    let mut g = row01("aragorn");
    zones::make_tokens(&mut g, P0, zones::Tokens { tgh: Some(1), ..zones::Tokens::new(249, 0) }).unwrap();
    perm(&mut g, P0, SEASON);
    assert_eq!(tokens_of_p0(&g), 249);
    cast_fair(&mut g);
    assert_eq!(tokens_of_p0(&g), zones::TOKEN_CAP);
    assert_eq!(big_tokens(&g), 1);
}

// R2: The One Ring's protection prevents the combat damage and Terror's trigger
#[test]
fn twinflame_r2_a_protected_opponent_is_not_lethal() {
    let mut g = row13("urdragon");
    g.player_mut(P1).ring_prot = true;
    assert!(sim_core::engine::life::prevents_damage(&g, P1, Some(P0)));
    held(&mut g, false);
    g.player_mut(P1).ring_prot = false;
    fair(&mut g, false); // the same board without the protection is lethal
}

// R3: Terror's trigger targets; a hexproof player can't be hit by it, combat damage still counts
#[test]
fn twinflame_r3_terror_damage_is_not_counted_against_a_hexproof_player() {
    let mut g = row01("urdragon");
    perm(&mut g, P0, TERROR);
    let shalai = perm(&mut g, P1, "Shalai, Voice of Plenty");
    g.perm_mut(shalai).tapped = true; // it can't block
    assert!(sim_core::engine::values::player_hexproof(&g, P1));
    // the fliers 8 + 5 = 13, the copy 5 (Terror is the target) = 18 in combat; the trigger's 5 would make 23
    opp_life(&mut g, 23);
    held(&mut g, false);
    opp_life(&mut g, 18);
    fair(&mut g, false); // combat damage alone kills
}

// R4: protection from red: Twinflame can't target the creature
fn red_protected_big(g: &mut Game) {
    let m = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).name == BIG).unwrap();
    g.perm_mut(m).eot_kw.push("pro_R");
    assert!(sim_core::engine::values::protected_from(g, m, sim_core::cards::Colors::from_letters("R")));
}

#[test]
fn twinflame_r4_a_protected_target_supplies_no_copy_to_the_lethal_estimate() {
    let mut g = row13("urdragon");
    red_protected_big(&mut g);
    held(&mut g, false);
}

#[test]
fn twinflame_r4_the_fair_option_skips_a_target_with_protection_from_red() {
    let mut g = row01("aragorn");
    red_protected_big(&mut g);
    assert!(twin_opts(&mut g, false).iter().all(|l| !l.starts_with("Twinflame (")), "the only target is protected");
}

// R5: the copies the estimate counts must fit under the token cap
#[test]
fn twinflame_r5_no_copy_at_the_token_cap_means_no_lethal_release() {
    let mut g = row13("urdragon");
    zones::make_tokens(&mut g, P0, zones::Tokens { tgh: Some(1), ..zones::Tokens::new(250, 0) }).unwrap();
    assert_eq!(tokens_of_p0(&g), 250);
    held(&mut g, false);
}

// Q1: Terror's trigger is not combat damage: every copy's power counts, flying or not
#[test]
fn twinflame_q1_terror_damage_counts_a_ground_copy() {
    let mut g = base("urdragon");
    lands(&mut g, P0, "Mountain", 3, false);
    perm(&mut g, P0, "Massacre Wurm");
    perm(&mut g, P0, TERROR);
    // Massacre Wurm is the target: its copy's 6 from Terror's trigger, then the ready Terror's 5 in combat
    opp_life(&mut g, 12);
    held(&mut g, false);
    opp_life(&mut g, 11);
    fair(&mut g, false);
}

#[test]
fn twinflame_q1_terror_damage_ignores_a_flying_blocker() {
    let mut g = base("urdragon");
    lands(&mut g, P0, "Mountain", 3, false);
    perm(&mut g, P0, "Massacre Wurm");
    perm(&mut g, P0, TERROR);
    perm(&mut g, P1, "Glorybringer"); // an untapped flier: Terror's 5 is blocked, the trigger's 6 is not
    opp_life(&mut g, 6);
    fair(&mut g, false);
    opp_life(&mut g, 7);
    held(&mut g, false);
}

// Q2 (coverage): a token-only Garth leaves the line dead by the alive table, yet `ready_garth` accepts it. Readiness
// wins by design: the combo option is the only Twinflame play, so the fair option is not offered.
#[test]
fn twinflame_q2_a_token_only_garth_is_dead_and_ready_and_readiness_wins() {
    use sim_core::impls::t45_urdragon_combos::{garth_line_alive, ready_garth};
    let mut g = row02("urdragon");
    let original = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .find(|&m| g.perm(m).name == GARTH && g.perm(m).phys.is_none() && !g.perm(m).token)
        .unwrap();
    let cd = card(&g, GARTH);
    zones::enter_token_copy(&mut g, P0, cd).unwrap();
    zones::leave(&mut g, original).unwrap();
    assert!(!garth_line_alive(&g, P0));
    assert!(ready_garth(&g, P0).0);
    held(&mut g, false);
}

// Q3 (coverage): the hold is zone-only. A permanent that locks the line (an opponent's Rule of Law) does not release
// it; that is the current policy, not a claim that it is right.
#[test]
fn twinflame_q3_a_locked_line_still_holds_under_the_zone_only_policy() {
    let mut g = row01("urdragon");
    perm(&mut g, P1, "Rule of Law");
    held(&mut g, false);
}

// ------------------------------------------------------------------ round 3: what the lethal release can really count
/// seat 0 as in row 13 plus a ready Terror of the Peaks (a 5/4 flier). Twinflame copies Terror (its enter effect makes it
/// the best target): 8 + 5 = 13 ready fliers, the copy's 5 in the air, and the original Terror's trigger of 5 for the
/// copy: 23 if every point reached the opponent.
const TERROR_POW: i32 = 5;
fn row13_terror() -> Game {
    let mut g = row13("urdragon");
    perm(&mut g, P0, TERROR);
    g
}

/// a ground creature the engine's "damage to any target" would remove: toughness within the trigger's 5, `pval` of 3 or more
fn ground_removal_target(g: &mut Game) {
    let m = perm(g, P1, "Port Razer"); // a vanilla 4/4: 1 + 4 * 0.5 = 3.0
    assert!(sim_core::engine::values::etgh(g, m) <= TERROR_POW, "within one trigger's damage");
    assert!(sim_core::engine::values::pval(g, m) >= 3.0, "worth removing: {}", sim_core::engine::values::pval(g, m));
    assert!(!g.perm(m).fly);
}

// Terror's trigger goes where `best_target_any` sends it: the opponent whose life it kills, else a creature it would
// remove, else the most threatening opponent's face
#[test]
fn twinflame_c1_terror_damage_goes_to_the_creature_the_engine_removes() {
    let mut g = row13_terror();
    ground_removal_target(&mut g);
    opp_life(&mut g, 23); // combat 18, the trigger would make 23, but it removes the 4/4
    held(&mut g, false);
}

#[test]
fn twinflame_c1_terror_damage_reaches_the_opponent_when_nothing_is_worth_removing() {
    let mut g = row13_terror();
    opp_life(&mut g, 23);
    fair(&mut g, false); // no creature to remove: the face takes it (13 + 5 combat + 5 trigger)
    let mut h = row13_terror();
    opp_life(&mut h, 24);
    held(&mut h, false);
}

#[test]
fn twinflame_c1_one_trigger_that_kills_the_opponent_alone_counts_even_with_a_creature_to_remove() {
    // an untapped flier blocks every flier, so only Terror's trigger of 5 can kill, and it does at 5 life
    let mut g = row13_terror();
    ground_removal_target(&mut g);
    perm(&mut g, P1, "Shivan Dragon");
    opp_life(&mut g, TERROR_POW);
    fair(&mut g, false);
}

#[test]
fn twinflame_c1_terror_damage_is_aimed_at_the_most_threatening_opponent() {
    // three seats: the engine sends the trigger to the opponent with the highest threat, not to the one under test
    let mut g = table(&["urdragon", "seph", "aragorn"]);
    for q in &mut g.players {
        q.hand.clear();
        q.gy.clear();
    }
    for n in [GARTH, SPARK, SEASON] {
        put(&mut g, P0, n, Z::Library);
    }
    put(&mut g, P0, TWIN, Z::Hand);
    glory(&mut g);
    perm(&mut g, P0, TERROR);
    // seat 2 has no creature to remove and a huge life total: the highest threat
    g.player_mut(PlayerId(2)).life = 999;
    assert!(sim_core::engine::values::threat(&g, P0, PlayerId(2)) > sim_core::engine::values::threat(&g, P0, P1));
    opp_life(&mut g, 23);
    held(&mut g, false); // seat 1 at 23: 18 in combat, the trigger goes elsewhere
}

// ------------------------------------------------------------------ round 3: attack restrictions
#[test]
fn twinflame_c2_a_propaganda_the_mana_cannot_pay_holds_the_line() {
    let mut g = row13("urdragon");
    perm(&mut g, P1, "Propaganda"); // {2} per attacker; 3 mana, 2 for Twinflame: nothing attacks
    assert_eq!(sim_core::engine::combat::attack_restrictions(&g, P0, P1).0, 2);
    held(&mut g, false);
}

#[test]
fn twinflame_c2_a_propaganda_holds_even_when_the_mana_could_pay() {
    // 6 mana: 2 for Twinflame and {2} each for the flier and its copy would pay. The estimate no longer models taxes (the
    // AI keeps no mana back for one, and the leftover mana may be restricted or wasted), so a taxed opponent is held.
    let mut g = row13("urdragon");
    lands(&mut g, P0, "Mountain", 3, false);
    perm(&mut g, P1, "Propaganda");
    held(&mut g, false);
    let mut h = row13("urdragon");
    lands(&mut h, P0, "Mountain", 2, false); // 5 mana: one attacker is paid for
    perm(&mut h, P1, "Propaganda");
    held(&mut h, false);
}

#[test]
fn twinflame_c2_an_attack_cap_holds_the_line_even_for_one_attacker() {
    // an attack cap in play: the estimate counts no combat damage at all (a cap can also turn on only after the cast)
    let mut g = row13("urdragon");
    perm(&mut g, P1, "Silent Arbiter"); // one attacker per combat
    assert_eq!(sim_core::engine::combat::attack_restrictions(&g, P0, P1).1, Some(1));
    held(&mut g, false);
    opp_life(&mut g, BIG_POW);
    held(&mut g, false); // one attacker would be enough at 8 life: held on purpose
}

#[test]
fn twinflame_c2_a_pacified_flier_is_not_counted() {
    let mut g = row13("urdragon");
    let dragon = perm(&mut g, P0, "Shivan Dragon"); // a ready 5/5 flier
    g.skip_etb = true; // the Aura without its enters effect; the test attaches it
    let arrest = perm(&mut g, P1, "Arrest");
    g.skip_etb = false;
    assert!(sim_core::impls::zur::lock_attach(&mut g, arrest, dragon).unwrap());
    assert!(sim_core::cardcode::locked(&g, dragon, "pacify"));
    opp_life(&mut g, 2 * BIG_POW + 5); // the flier 8 + its copy 8 + the pacified 5
    held(&mut g, false);
    opp_life(&mut g, 2 * BIG_POW);
    fair(&mut g, false);
}

// ------------------------------------------------------------------ round 4: the release holds unless nothing unmodelled is live
use sim_core::ai::act::perform;
use sim_core::engine::{combat, turn, values};
use sim_core::hooks::Opt;

/// seat 0 = urdragon with the line alive, Twinflame in hand, three Mountains, an untapped Ancient Silver Dragon; the
/// other seats are `seph`, `aragorn`, `cloud`
fn adv_base(seats: usize) -> Game {
    let keys = ["urdragon", "seph", "aragorn", "cloud"];
    let mut g = table(&keys[..seats]);
    for p in &mut g.players {
        p.hand.clear();
        p.gy.clear();
    }
    for name in [GARTH, SPARK, SEASON] {
        let c = take(&mut g, P0, name);
        g.player_mut(P0).library.push(c);
    }
    hand(&mut g, P0, &[TWIN]);
    lands(&mut g, P0, "Mountain", 3, false);
    perm(&mut g, P0, BIG);
    g
}

/// what Twinflame's hand hook offers right now (empty = held)
fn adv_hook(g: &mut Game) -> Vec<Opt> {
    let c = card(g, TWIN);
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    f(g, c, P0, Some(false)).unwrap()
}

fn adv_cast_offered(g: &mut Game) {
    let opts = main_options(g, P0, false).unwrap();
    let o = opts.iter().find(|o| o.label.starts_with("Twinflame (")).expect("fair option");
    assert!(perform(g, P0, o.act.as_ref().unwrap()).unwrap());
}

/// only to bypass the hold in the probe: exile Garth
fn adv_kill_line(g: &mut Game) {
    let c = card(g, GARTH);
    g.player_mut(P0).library.retain(|&x| x != c);
    g.player_mut(P0).exile.push(c);
}

fn adv_dead(g: &Game, q: PlayerId) -> bool {
    !g.player(q).alive || g.player(q).life <= 0
}

/// (the estimate releases, an all-in attack after the real cast kills somebody, the engine's own combat kills somebody).
/// If the all-in attack cannot kill, nothing can: a release without it is a wrong release.
fn adv_probe(g: &Game) -> (bool, bool, bool) {
    let mut e = g.clone();
    let rel = !adv_hook(&mut e).is_empty();
    let mut a = g.clone();
    adv_kill_line(&mut a);
    adv_cast_offered(&mut a);
    if a.players.iter().any(|o| o.id != P0 && adv_dead(&a, o.id)) {
        return (rel, true, true);
    }
    turn::end_step_pools(&mut a); // floating mana empties before combat
    let mut allin = false;
    for q in a.opps(P0).collect::<Vec<_>>() {
        let mut b = a.clone();
        let atk: Vec<PermId> = b
            .player(P0)
            .perms
            .clone()
            .into_iter()
            .filter(|&m| {
                let x = b.perm(m);
                b.is_creature(m)
                    && !x.tapped
                    && !x.phased
                    && (!x.sick || combat::has_haste(&b, m))
                    && (!x.noatk || values::epow(&b, m) >= 3)
                    && values::epow(&b, m) > 0
                    && !sim_core::cardcode::locked(&b, m, "pacify")
            })
            .collect();
        let atk = if values::chasm(&b, P0) { vec![] } else { atk };
        let paid = combat::attack_limits(&mut b, P0, q, atk).unwrap();
        if paid.is_empty() {
            continue;
        }
        combat::resolve_combat(&mut b, P0, &paid, q, &[]).unwrap();
        if b.players.iter().any(|o| o.id != P0 && adv_dead(&b, o.id)) {
            allin = true;
        }
    }
    let mut c = a.clone();
    combat::combat(&mut c, P0).unwrap();
    (rel, allin, c.players.iter().any(|o| o.id != P0 && adv_dead(&c, o.id)))
}

/// Twinflame is offered although no opponent can die even if everything attacks the best way
fn adv_wrong(g: &Game) -> bool {
    let (rel, allin, _) = adv_probe(g);
    rel && !allin
}

fn adv_terror_token(g: &mut Game) {
    let c = card(g, TERROR);
    zones::enter_token_copy(g, P0, c).unwrap().unwrap();
}

// sanity: the probe state itself, one point either side of lethal
#[test]
fn twinflame_d0_sanity_16_releases_and_kills_17_holds() {
    let mut g = adv_base(2);
    g.player_mut(P1).life = 16;
    assert_eq!(adv_probe(&g), (true, true, true));
    g.player_mut(P1).life = 17;
    assert!(!adv_probe(&g).0);
}

// F1: attack restrictions. The estimate counts no combat damage to an opponent whose attacks are taxed, and none while
// any attack cap is out.
#[test]
fn twinflame_d1_angelic_arbiter_turns_on_with_the_cast_so_it_holds() {
    let mut g = adv_base(2);
    let a = perm(&mut g, P1, "Angelic Arbiter");
    g.perm_mut(a).tapped = true;
    g.perm_mut(a).sick = false;
    g.player_mut(P1).life = 16;
    assert!(!adv_wrong(&g), "Twinflame offered, but the cast itself bans the attack");
}

#[test]
fn twinflame_d1_sandwurm_convergence_bars_the_fliers_so_it_holds() {
    let mut g = adv_base(2);
    perm(&mut g, P1, "Sandwurm Convergence");
    g.player_mut(P1).life = 16;
    assert!(!adv_wrong(&g), "Twinflame offered, but no flier can attack");
}

#[test]
fn twinflame_d1_spell_only_mana_does_not_pay_an_attack_tax() {
    let mut g = adv_base(2);
    perm(&mut g, P0, "Hydro-Channeler");
    let c = card(&g, "Hydro-Channeler");
    for _ in 0..2 {
        let m = zones::enter_token_copy(&mut g, P0, c).unwrap().unwrap();
        g.perm_mut(m).sick = false;
    }
    perm(&mut g, P1, "Propaganda");
    g.player_mut(P1).life = 16;
    assert!(!adv_wrong(&g), "Twinflame offered, but the restricted mana cannot pay the tax");
}

#[test]
fn twinflame_d1_mana_that_empties_before_combat_does_not_pay_an_attack_tax() {
    // Mountain + Sol Ring: Twinflame pays R and the generic from Sol Ring, whose second mana empties at the end of main 1
    let mut g = table(&["urdragon", "seph"]);
    for p in &mut g.players {
        p.hand.clear();
        p.gy.clear();
    }
    for name in [GARTH, SPARK, SEASON] {
        let c = take(&mut g, P0, name);
        g.player_mut(P0).library.push(c);
    }
    hand(&mut g, P0, &[TWIN]);
    lands(&mut g, P0, "Mountain", 1, false);
    perm(&mut g, P0, BIG);
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P1, "Baird, Steward of Argive"); // {1} per attacker
    g.player_mut(P1).life = 8;
    assert!(!adv_wrong(&g), "Twinflame offered, but no mana is left for the tax at combat");
}

#[test]
fn twinflame_d1_a_pumped_noatk_creature_takes_the_only_cap_slot() {
    let mut g = adv_base(2);
    let dork = perm(&mut g, P0, "Paradise Druid");
    g.perm_mut(dork).plus = 8;
    perm(&mut g, P1, "Silent Arbiter");
    g.player_mut(P1).life = 8;
    assert!(!adv_wrong(&g), "Twinflame offered, but the cap slot goes to the ground creature");
}

// F1 controls: a tax on another seat only leaves the untaxed seat's combat counted; and a table with nothing in the way
#[test]
fn twinflame_d1_control_a_tax_on_another_seat_still_releases_for_the_untaxed_one() {
    let mut g = adv_base(3);
    perm(&mut g, P2, "Propaganda");
    g.player_mut(P1).life = 16;
    assert_eq!(adv_probe(&g), (true, true, true));
}

// F2: a target the estimate may count must take damage like any other: every reducing gate the engine applies in
// `lose_life` and the combat damage step
#[test]
fn twinflame_d2_gisela_halves_damage_to_its_controller_so_it_holds() {
    let mut g = adv_base(2);
    let m = perm(&mut g, P1, "Gisela, Blade of Goldnight");
    g.perm_mut(m).tapped = true;
    g.player_mut(P1).life = 16;
    assert!(!adv_wrong(&g), "Twinflame offered, but Gisela halves the damage");
}

#[test]
fn twinflame_d2_angels_grace_already_in_force_holds() {
    let mut g = adv_base(2);
    g.player_mut(P1).life = 16;
    let (k, st) = (sim_core::sym::intern("stax_grace"), g.turn_stamp());
    g.player_mut(P1).flag_turn.insert(k, st);
    assert!(!adv_wrong(&g), "Twinflame offered, but damage cannot take the opponent below 1");
}

#[test]
fn twinflame_d2_a_fog_this_turn_holds() {
    let mut g = adv_base(2);
    g.fog = Some(g.turn_stamp());
    g.player_mut(P1).life = 16;
    assert!(!adv_wrong(&g), "Twinflame offered, but combat damage is prevented");
}

#[test]
fn twinflame_d2_a_prevented_attacker_is_not_counted() {
    let mut g = adv_base(2);
    let silver = perms_named(&g, P0, BIG)[0];
    let m = perm(&mut g, P1, "Old Fat Spider Can't See Me");
    g.perm_mut(m).data.set(sim_core::state::DataKey::Prevent, sim_core::state::Val::Perm(silver));
    g.player_mut(P1).life = 16;
    assert!(!adv_wrong(&g), "Twinflame offered, but the 8-power flier's damage is prevented");
}

#[test]
fn twinflame_d2_phyrexian_unlife_means_zero_life_is_not_a_kill() {
    // `adv_wrong` would count life <= 0 as dead, so this asserts on the hook itself
    let mut g = adv_base(2);
    perm(&mut g, P1, "Phyrexian Unlife");
    g.player_mut(P1).life = 16;
    assert!(adv_hook(&mut g).is_empty(), "Twinflame offered, but an opponent at 0 life does not lose");
}

#[test]
fn twinflame_d2_control_without_unlife_it_releases_at_16() {
    let mut g = adv_base(2);
    g.player_mut(P1).life = 16;
    assert!(!adv_hook(&mut g).is_empty());
}

// F3: Terror of the Peaks' triggers are replayed one at a time with the engine's aim (`t2::best_target_any`)
#[test]
fn twinflame_d3_the_first_opponent_in_reach_is_protected_so_the_trigger_kills_nobody() {
    // no combat (Glacial Chasm). One trigger of 5: the engine aims at the first opponent with 5 or less life, the
    // protected one at 1, not at the one at 5
    let mut g = adv_base(3);
    perm(&mut g, P0, TERROR);
    lands(&mut g, P0, "Glacial Chasm", 1, false);
    g.player_mut(P1).life = 1;
    g.player_mut(P1).ring_prot = true;
    g.player_mut(PlayerId(2)).life = 5;
    assert!(!adv_wrong(&g), "Twinflame offered, but the trigger is aimed at the protected opponent");
}

#[test]
fn twinflame_d3_the_aim_changes_after_the_first_trigger() {
    // two Terrors, two triggers of 5; 28/27: the first drops the leader below the other, so the second goes to the other
    let mut g = adv_base(3);
    perm(&mut g, P0, TERROR);
    adv_terror_token(&mut g);
    g.player_mut(P1).life = 28;
    g.player_mut(PlayerId(2)).life = 27;
    assert!(!adv_wrong(&g), "Twinflame offered, but the second trigger changes aim");
}

#[test]
fn twinflame_d3_control_flip_23_22_kills() {
    let mut g = adv_base(3);
    perm(&mut g, P0, TERROR);
    adv_terror_token(&mut g);
    g.player_mut(P1).life = 23;
    g.player_mut(PlayerId(2)).life = 22;
    assert_eq!(adv_probe(&g), (true, true, true));
}

#[test]
fn twinflame_d3_control_flip_28_29_holds() {
    let mut g = adv_base(3);
    perm(&mut g, P0, TERROR);
    adv_terror_token(&mut g);
    g.player_mut(P1).life = 28;
    g.player_mut(PlayerId(2)).life = 29;
    assert!(!adv_probe(&g).0);
}

// Recorded limit (not a defect to fix here): when Twinflame copies a Terror under Doubling Season, the second token's
// entry also triggers the first token, which the estimate leaves out. The error only delays the release.
#[test]
fn twinflame_d4_limit_later_copies_trigger_earlier_terror_tokens_is_a_hold() {
    let mut g = adv_base(2);
    perm(&mut g, P0, TERROR);
    perm(&mut g, P0, SEASON);
    g.player_mut(P1).life = 38;
    let (rel, allin, _) = adv_probe(&g);
    assert!(allin, "the engine kills here");
    assert!(!rel, "today's estimate holds (limit: the first token's trigger for the second copy is not counted)");
}

// F4: copies that enter tapped cannot attack, so their power is not combat damage. Blind Obedience makes every
// opponent's creatures enter tapped; Authority of the Consuls does the same through a compiled static.
#[test]
fn twinflame_e1_blind_obedience_on_the_opponent_makes_the_copies_tapped_so_it_holds() {
    let mut g = adv_base(2);
    perm(&mut g, P1, "Blind Obedience");
    g.player_mut(P1).life = 16;
    assert!(!adv_wrong(&g), "Twinflame offered, but the copy enters tapped and only the 8-power flier attacks");
    assert!(adv_hook(&mut g).is_empty(), "the estimate must not count the tapped copy's power");
}

#[test]
fn twinflame_e1_authority_of_the_consuls_on_the_opponent_makes_the_copies_tapped_so_it_holds() {
    let mut g = adv_base(2);
    perm(&mut g, P1, "Authority of the Consuls");
    g.player_mut(P1).life = 16;
    assert!(!adv_wrong(&g), "Twinflame offered, but the copy enters tapped and only the 8-power flier attacks");
    assert!(adv_hook(&mut g).is_empty(), "the estimate must not count the tapped copy's power");
}

#[test]
fn twinflame_e1_control_blind_obedience_under_the_casting_seat_releases_at_16() {
    let mut g = adv_base(2);
    perm(&mut g, P0, "Blind Obedience");
    g.player_mut(P1).life = 16;
    assert!(!adv_hook(&mut g).is_empty());
    assert!(!adv_wrong(&g));
}

#[test]
fn twinflame_e1_control_authority_of_the_consuls_under_the_casting_seat_releases_at_16() {
    let mut g = adv_base(2);
    perm(&mut g, P0, "Authority of the Consuls");
    g.player_mut(P1).life = 16;
    assert!(!adv_hook(&mut g).is_empty());
    assert!(!adv_wrong(&g));
}

#[test]
fn twinflame_e1_control_a_tapped_copy_still_triggers_terror_so_the_release_stands() {
    // the copy of the tapped Terror enters tapped and its entry triggers the original Terror for 5, enough at 5 life
    let mut g = adv_base(2);
    let silver = perms_named(&g, P0, BIG)[0];
    g.perm_mut(silver).tapped = true;
    let t = perm(&mut g, P0, TERROR);
    g.perm_mut(t).tapped = true;
    perm(&mut g, P1, "Blind Obedience");
    g.player_mut(P1).life = 5;
    assert_eq!(adv_probe(&g), (true, true, true));
}

// ------------------------------------------------------------------ round 6: visible effects the estimate did not read
/// `adv_probe`, after printing what the real cast leaves behind
fn adv_result(g: &Game) -> (bool, bool, bool) {
    let mut a = g.clone();
    adv_kill_line(&mut a);
    adv_cast_offered(&mut a);
    println!(
        "after cast: opponents {:?}; own creatures {:?}",
        a.opps(P0).map(|q| (q, a.player(q).life, a.player(q).poison)).collect::<Vec<_>>(),
        a.player(P0)
            .perms
            .iter()
            .filter(|&&m| a.is_creature(m))
            .map(|&m| (a.perm(m).name, a.perm(m).tapped))
            .collect::<Vec<_>>()
    );
    let r = adv_probe(g);
    println!("release={} allin_kills={} ai_kills={}", r.0, r.1, r.2);
    r
}

// F5: an opponent's entry trigger gains life while the line plays out: the copy enters, the opponent gains, and the
// attack that was exactly lethal is one short
#[test]
fn authority_gain_invalidates_original_flier_lethal() {
    let mut g = adv_base(2);
    perm(&mut g, P1, "Authority of the Consuls");
    g.player_mut(P1).life = 8;
    assert_eq!(adv_result(&g), (false, false, false));
}

#[test]
fn soul_warden_gain_invalidates_exact_combat_lethal() {
    let mut g = adv_base(2);
    perm(&mut g, P1, "Soul Warden");
    g.player_mut(P1).life = 16;
    assert_eq!(adv_result(&g), (false, false, false));
}

// F6: Terror's triggers the engine does not run
#[test]
fn hushbringer_suppresses_terror_triggers() {
    let mut g = adv_base(2);
    let big = g.player(P0).perms[0];
    g.perm_mut(big).tapped = true;
    let terror = perm(&mut g, P0, TERROR);
    g.perm_mut(terror).tapped = true;
    perm(&mut g, P1, "Hushbringer");
    perm(&mut g, P1, "Blind Obedience");
    g.player_mut(P1).life = 5;
    // Hushbringer blocks any flying copy, and silences its entry trigger.
    assert_eq!(adv_result(&g), (false, false, false));
}

#[test]
fn neutered_terror_has_no_entry_trigger() {
    let mut g = adv_base(2);
    let big = g.player(P0).perms[0];
    g.perm_mut(big).tapped = true;
    let terror = perm(&mut g, P0, TERROR);
    g.perm_mut(terror).neutered = true;
    g.perm_mut(terror).tapped = true;
    perm(&mut g, P1, "Blind Obedience");
    g.player_mut(P1).life = 5;
    assert_eq!(adv_result(&g), (false, false, false));
}

// F7: damage an attacker deals that is not life loss
#[test]
fn szadek_combat_mills_instead_of_life_loss() {
    let mut g = adv_base(2);
    let big = g.player(P0).perms[0];
    g.perm_mut(big).tapped = true;
    perm(&mut g, P0, "Szadek, Lord of Secrets");
    g.player_mut(P1).life = 13;
    assert_eq!(adv_result(&g), (false, false, false));
}

// F8: robustness: no living opponent and a Terror
#[test]
fn no_living_opponents_does_not_panic() {
    let mut g = adv_base(2);
    perm(&mut g, P0, TERROR);
    g.player_mut(P1).alive = false;
    assert!(adv_hook(&mut g).is_empty());
}

// F9: visible effects of the opponent that act during the line and the estimate did not follow
#[test]
fn spore_frog_fogs_the_attack() {
    let mut g = adv_base(2);
    perm(&mut g, P1, "Spore Frog");
    g.player_mut(P1).life = 16;
    assert_eq!(adv_result(&g), (false, false, false));
}

#[test]
fn sunscorch_cast_gain_invalidates_exact_lethal() {
    let mut g = adv_base(2);
    let r = perm(&mut g, P1, "Sunscorch Regent");
    g.perm_mut(r).tapped = true;
    g.player_mut(P1).life = 16;
    assert_eq!(adv_result(&g), (false, false, false));
}

#[test]
fn hammer_loses_flying_attacker_is_blockable() {
    let mut g = adv_base(2);
    let big = g.player(P0).perms[0];
    let hammer = perm(&mut g, P0, "Colossus Hammer");
    g.perm_mut(hammer).attached = Some(big);
    perm(&mut g, P1, "Armored Warhorse"); // a ready ground blocker
    g.player_mut(P1).life = 20;
    assert_eq!(adv_result(&g), (false, false, false));
}
