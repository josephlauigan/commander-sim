//! fix-combo-colour-pay: ritual mana has the colour the card prints. A goldfish Tymna/Thrasios turn 2 with Watery Grave
//! and Tundra, Dark Ritual, Thassa's Oracle and Demonic Consultation paid {U}{U}{B} with {B}{B}{B}{U}: the engine
//! floated every ritual's mana as "any colour of the player's identity", so Dark Ritual's {B}{B}{B} paid {U}{U}.

use sim_core::engine::mana::{can_pay, can_pay_with_rituals, mana_units};
use sim_core::engine::pool::Kind;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

/// the Thrasios seat against three inert goldfish seats, its hand emptied
fn thrasios_goldfish() -> Game {
    let mut g =
        table(&["thrasios", "kraum-tymna-blue-breach-midrange", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"]);
    for i in 1..4 {
        let q = PlayerId(i);
        let wastes: Vec<_> = (0..60).map(|_| card(&g, "Wastes")).collect();
        let pl = g.player_mut(q);
        pl.key = sim_core::sym::intern(&format!("goldfish-{}", i + 1));
        pl.hand.clear();
        pl.library = wastes;
    }
    g.player_mut(P0).hand.clear();
    let lib: Vec<_> = (0..40).map(|_| card(&g, "Island")).collect();
    g.player_mut(P0).library = lib;
    g
}

fn turn_two(land_names: &[&str], hand_names: &[&str]) -> Game {
    let mut g = thrasios_goldfish();
    g.active = Some(P0);
    g.player_mut(P0).turns = 2;
    for l in land_names {
        lands(&mut g, P0, l, 1, false);
    }
    hand(&mut g, P0, hand_names);
    g
}

const LINE: [&str; 3] = ["Thassa's Oracle", "Demonic Consultation", "Dark Ritual"];

/// The unit test of the report: Watery Grave's {B} pays Dark Ritual, which leaves {B}{B}{B} and Tundra's {U}. That is not
/// {U}{U}{B}.
#[test]
fn black_ritual_mana_and_one_blue_land_do_not_pay_uub() {
    let g = turn_two(&["Watery Grave", "Tundra"], &LINE);
    assert!(!can_pay_with_rituals(&g, P0, 0, "UUB", &[]), "BBB + U is not UUB");
    assert!(can_pay_with_rituals(&g, P0, 0, "UBB", &[]), "BBB + U is UBB (the ritual is counted)");
}

/// with a third land that makes {U} the line is legal: Grave -> Dark Ritual (BBB), Tundra {U}, Sea {U}
#[test]
fn three_lands_and_a_ritual_do_pay_uub() {
    let g = turn_two(&["Watery Grave", "Tundra", "Underground Sea"], &LINE);
    assert!(can_pay_with_rituals(&g, P0, 0, "UUB", &[]));
}

/// the reproduced game: two lands, nothing else, the AI must not go for the Oracle line
#[test]
fn the_ai_does_not_win_on_turn_two_with_two_lands_and_dark_ritual() {
    let mut g = turn_two(&["Watery Grave", "Tundra"], &LINE);
    let opts = sim_core::ai::brain::main_options(&mut g, P0, false).unwrap();
    assert!(
        !opts.iter().any(|o| o.label.starts_with("combo: Thassa's Oracle")),
        "{:?}",
        opts.iter().map(|o| &o.label).collect::<Vec<_>>()
    );
    sim_core::ai::brain::main(&mut g, P0, false).unwrap();
    assert!(!(g.over && g.winner == Some(P0)), "UU + B was paid with BBB + U");
}

/// Dark Ritual cast for real floats {B}{B}{B}: it pays {B}, never {U}
#[test]
fn dark_ritual_floats_black_mana() {
    let mut g = turn_two(&["Watery Grave", "Tundra"], &["Dark Ritual"]);
    let r = card(&g, "Dark Ritual");
    assert!(sim_core::ai::brain::do_cast(&mut g, P0, r, None).unwrap());
    assert_eq!(g.player(P0).mana.get(Kind::B), 3, "{{B}}{{B}}{{B}} floats as black");
    assert_eq!(g.player(P0).mana.get(Kind::Any), 0);
    // BBB floating + Tundra's {U}: {B}{B}{B} and {U} can be paid, {U}{U} cannot
    assert!(can_pay(&g, P0, 0, "BBBU", false));
    assert!(!can_pay(&g, P0, 0, "UU", false));
    assert!(mana_units(&g, P0, false).iter().filter(|u| u.cols.has('U')).map(|u| u.amt).sum::<u32>() == 1);
}

/// The audit: every ritual floats the colour it prints (Cabal Ritual and Battle Hymn used to float any colour; the
/// ritual cards of the ability language, Seething Song among them, do now by their `color`). Cast with lands that pay.
#[test]
fn every_ritual_floats_its_printed_colour() {
    for (name, kind, n, land, count) in [
        ("Cabal Ritual", Kind::B, 3, "Swamp", 2),
        ("Seething Song", Kind::R, 5, "Mountain", 3),
        ("Pyretic Ritual", Kind::R, 3, "Mountain", 2),
        ("Desperate Ritual", Kind::R, 3, "Mountain", 2),
        ("Rite of Flame", Kind::R, 2, "Mountain", 1),
    ] {
        let mut g = thrasios_goldfish();
        g.active = Some(P0);
        g.player_mut(P0).turns = 2;
        lands(&mut g, P0, land, count, false);
        let c = hand(&mut g, P0, &[name])[0];
        assert!(sim_core::ai::brain::do_cast(&mut g, P0, c, None).unwrap(), "{name} is cast");
        let m = &g.player(P0).mana;
        assert_eq!(m.get(Kind::Any), 0, "{name} floats no any-colour mana");
        assert!(m.get(kind) >= n, "{name} floats {n} of its colour");
    }
}

/// Battle Hymn adds {R} for each creature you control: red mana, never any colour
#[test]
fn battle_hymn_floats_red() {
    let mut g = thrasios_goldfish();
    g.active = Some(P0);
    g.player_mut(P0).turns = 2;
    lands(&mut g, P0, "Mountain", 2, false);
    perm(&mut g, P0, "Llanowar Elves");
    let c = hand(&mut g, P0, &["Battle Hymn"])[0];
    assert!(sim_core::ai::brain::do_cast(&mut g, P0, c, None).unwrap());
    assert_eq!(g.player(P0).mana.get(Kind::Any), 0);
    assert!(g.player(P0).mana.get(Kind::R) >= 1);
}

/// the hand-written Cabal Ritual counts as {B}{B}{B} in the planner too (its threshold five included)
#[test]
fn the_planner_sees_the_colours_a_ritual_will_make() {
    let g = turn_two(&["Swamp", "Swamp"], &["Cabal Ritual", "Dark Ritual"]);
    assert!(
        can_pay_with_rituals(&g, P0, 0, "BBBB", &[]),
        "two Swamps cast Cabal Ritual (BBB); Dark Ritual then adds more black"
    );
    assert!(!can_pay_with_rituals(&g, P0, 0, "UU", &[]), "no ritual makes blue mana");
    assert!(!can_pay_with_rituals(&g, P0, 0, "RR", &[]), "no black ritual makes red mana");
}
