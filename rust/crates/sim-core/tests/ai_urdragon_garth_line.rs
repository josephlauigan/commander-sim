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
        let cs = offered(&g, &hand_cards);
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
