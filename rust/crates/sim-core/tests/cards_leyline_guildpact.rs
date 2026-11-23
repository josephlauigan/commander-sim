//! Leyline of the Guildpact ({G/W}{G/U}{B/G}{R/G}, Enchantment): "If this card is in your opening hand, you may begin the
//! game with it on the battlefield. Each nonland permanent you control is all colors. Lands you control are every basic
//! land type in addition to their other types." Each test checks the Oracle text, not what the engine did before. The
//! hybrid cost (paying and casting it) is `cards_hybrid_cost.rs`; these tests put Leyline on the battlefield directly.

mod play_common;
use play_common::*;
use sim_core::cards::Colors;
use sim_core::engine::mana::{Source, mana_units};
use sim_core::engine::values::colors_of;
use sim_core::engine::{chars, mana, removal, turn, zones};
use sim_core::impls::card_models::{opening_hand_actions, opening_hand_person};
use sim_core::state::{DataKey, Val};

const WUBRG: &str = "WUBRG";

fn all() -> Colors {
    Colors::from_letters(WUBRG)
}

/// a three-seat table with Leyline on P0's battlefield
fn game() -> Game {
    table(&["thanos", "kraum-tymna-blue-breach-midrange", "sisay-legends-toolbox"])
}

fn with_leyline() -> (Game, PermId) {
    let mut g = game();
    let l = perm(&mut g, P0, "Leyline of the Guildpact");
    (g, l)
}

fn unit_of(g: &Game, p: PlayerId, src: Source) -> sim_core::engine::mana::Unit {
    *mana_units(g, p, false).iter().find(|u| u.src == src).expect("a unit for that source")
}

// ------------------------------------------------------------------ layer 5: colour
#[test]
fn nonland_permanents_and_tokens_all_colours() {
    let (mut g, ley) = with_leyline();
    let angel = perm(&mut g, P0, "Serra Angel");
    let ring = perm(&mut g, P0, "Sol Ring");
    let tok = token(&mut g, P0, 2);
    for m in [ley, angel, ring, tok] {
        assert_eq!(colors_of(&g, m), all(), "{} is every colour", g.perm(m).name);
    }
    assert_eq!(chars::colors(&g, ring), all(), "a colourless artifact too");
}

#[test]
fn lands_stay_colourless() {
    let (mut g, _) = with_leyline();
    let island = lands(&mut g, P0, "Island", 1, false)[0];
    let vault = lands(&mut g, P0, "Mutavault", 1, false)[0];
    assert_eq!(colors_of(&g, island), Colors::NONE);
    // a manland is still a land: it stays colourless
    g.perm_mut(vault).data.set(DataKey::Anim, Val::Bool(true));
    (g.perm_mut(vault).pow, g.perm_mut(vault).tgh) = (2, 2);
    assert!(g.is_creature(vault));
    assert_eq!(colors_of(&g, vault), Colors::NONE, "an animated land has no colour from Leyline");
    // the land face of a permanent that is a land now
    let angel = perm(&mut g, P0, "Serra Angel");
    g.perm_mut(angel).data.set(DataKey::LandFace, Val::Bool(true));
    assert_eq!(colors_of(&g, angel), Colors::NONE, "the land face is a land");
}

#[test]
fn opponents_permanents_unaffected() {
    let (mut g, _) = with_leyline();
    let theirs = perm(&mut g, P1, "Serra Angel");
    let their_island = lands(&mut g, P1, "Island", 1, false)[0];
    let mine = lands(&mut g, P0, "Island", 1, false)[0];
    assert_eq!(colors_of(&g, theirs), Colors::from_letters("W"));
    assert_eq!(chars::land_types(&g, their_island), Colors::from_letters("U"), "an opponent's land keeps its types");
    assert_eq!(chars::land_types(&g, mine), all(), "mine is every basic type");
    assert_eq!(mana::land_colors_now(&g, P1, their_island), Colors::from_letters("U"));
}

#[test]
fn elder_alone_is_5_5_and_taps_wubrg() {
    let (mut g, _) = with_leyline();
    let elder = perm(&mut g, P0, "Faeburrow Elder");
    // Leyline and the Elder are the only permanents, and both are all five colours
    assert_eq!((sim_core::engine::values::epow(&g, elder), sim_core::engine::values::etgh(&g, elder)), (5, 5));
    let u = unit_of(&g, P0, Source::Perm(elder));
    assert_eq!((u.cols, u.amt, u.shape), (all(), 5, mana::Shape::Each), "one mana of each colour");
}

// ------------------------------------------------------------------ layer 4: land types
#[test]
fn basic_and_strip_mine_tap_any_colour() {
    let (mut g, _) = with_leyline();
    let island = lands(&mut g, P0, "Island", 1, false)[0];
    let mine = lands(&mut g, P0, "Strip Mine", 1, false)[0];
    for l in [island, mine] {
        assert_eq!(chars::land_types(&g, l), all(), "{} is every basic type", g.perm(l).name);
        assert_eq!(mana::land_colors_now(&g, P0, l), all(), "{} taps for any colour", g.perm(l).name);
        let u = unit_of(&g, P0, Source::Land(l));
        assert_eq!((u.cols, u.amt), (all(), 1));
    }
    // the Island paying {W}{U}{B}{R}{G} with four other such lands: five lands, five colours
    lands(&mut g, P0, "Island", 4, false);
    assert!(mana::can_pay(&g, P0, 0, "WUBRG", false));
}

#[test]
fn fetch_untapped_taps_any_colour() {
    let (mut g, _) = with_leyline();
    let fetch = lands(&mut g, P0, "Windswept Heath", 1, false)[0];
    assert_eq!(mana::land_colors_now(&g, P0, fetch), all(), "an untapped fetch land has every basic type's ability");
    // and it still has its own ability: a land that is Tapped can't make mana
    g.perm_mut(fetch).tapped = true;
    assert!(mana_units(&g, P0, false).iter().all(|u| u.src != Source::Land(fetch)));
}

#[test]
fn fetch_search_uses_printed_types() {
    let (mut g, _) = with_leyline();
    // the library cards are not permanents: Leyline gives them no types, so a fetch land finds only a printed type
    let (mine, plains) = (card(&g, "Strip Mine"), card(&g, "Plains"));
    assert!(!turn::fetch_legal(&g, mine, Some("GW")), "Strip Mine has no Forest or Plains type in the library");
    assert!(turn::fetch_legal(&g, plains, Some("GW")));
    let fetch = lands(&mut g, P0, "Windswept Heath", 1, false)[0];
    g.player_mut(P0).library = vec![mine, plains];
    turn::crack_fetch(&mut g, P0, fetch).unwrap();
    let names: Vec<String> = g.player(P0).lands.iter().map(|&l| g.perm(l).name.to_string()).collect();
    assert_eq!(names, ["Plains"], "it found the Plains, not Strip Mine");
}

#[test]
fn leyline_leaves_everything_reverts() {
    let (mut g, ley) = with_leyline();
    let angel = perm(&mut g, P0, "Serra Angel");
    let island = lands(&mut g, P0, "Island", 1, false)[0];
    assert_eq!(colors_of(&g, angel), all());
    zones::bounce(&mut g, ley).unwrap();
    assert!(!g.hooks.contains(&ley), "no longer listed");
    assert_eq!(colors_of(&g, angel), Colors::from_letters("W"));
    assert_eq!(chars::land_types(&g, island), Colors::from_letters("U"));
    assert_eq!(mana::land_colors_now(&g, P0, island), Colors::from_letters("U"));
}

#[test]
fn leyline_phased_out_does_nothing() {
    let (mut g, ley) = with_leyline();
    let angel = perm(&mut g, P0, "Serra Angel");
    g.perm_mut(ley).phased = true;
    assert_eq!(colors_of(&g, angel), Colors::from_letters("W"));
    g.perm_mut(ley).phased = false;
    assert_eq!(colors_of(&g, angel), all());
}

#[test]
fn copy_of_leyline_no_double_effect() {
    let (mut g, ley) = with_leyline();
    let angel = perm(&mut g, P0, "Serra Angel");
    let island = lands(&mut g, P0, "Island", 1, false)[0];
    // Copy Enchantment: it enters as a copy of the best enchantment on the battlefield, Leyline
    let copy = perm(&mut g, P0, "Copy Enchantment");
    assert_eq!(g.perm(copy).cd, g.perm(ley).cd, "a copy of Leyline");
    assert!(g.hooks.contains(&copy), "the copy is listed with the card code it copied");
    assert_eq!((colors_of(&g, angel), chars::land_types(&g, island)), (all(), all()), "two Leylines change nothing");
    assert_eq!(colors_of(&g, copy), all());
    // either one alone does it
    zones::bounce(&mut g, ley).unwrap();
    assert_eq!((colors_of(&g, angel), chars::land_types(&g, island)), (all(), all()), "the copy alone");
    zones::bounce(&mut g, copy).unwrap();
    assert_eq!(colors_of(&g, angel), Colors::from_letters("W"), "none left");
}

// ------------------------------------------------------------------ what the colours do
#[test]
fn doom_blade_cannot_target() {
    let (mut g, _) = with_leyline();
    // Terror: "destroy target nonartifact, nonblack creature". The opponent's removal, aimed at P0's board
    let angel = perm(&mut g, P0, "Serra Angel");
    let terror = card(&g, "Terror");
    let before = removal::legal_targets(&g, P1, "destroy", "cna", false, Some(terror));
    assert!(!before.contains(&angel), "Leyline makes the Angel black too: no target for a nonblack spell");
    let mut g2 = game();
    let angel2 = perm(&mut g2, P0, "Serra Angel");
    let terror2 = card(&g2, "Terror");
    assert!(
        removal::legal_targets(&g2, P1, "destroy", "cna", false, Some(terror2)).contains(&angel2),
        "control: without Leyline the Angel is a legal target"
    );
}

#[test]
fn pyroblast_hits_every_permanent() {
    let (mut g, _) = with_leyline();
    let angel = perm(&mut g, P0, "Serra Angel"); // white: not a Pyroblast target by itself
    let pyro = card(&g, "Pyroblast");
    assert!(removal::legal_targets(&g, P1, "destroy", "blue", false, Some(pyro)).contains(&angel), "it is blue too");
    let mut g2 = game();
    let angel2 = perm(&mut g2, P0, "Serra Angel");
    let pyro2 = card(&g2, "Pyroblast");
    assert!(!removal::legal_targets(&g2, P1, "destroy", "blue", false, Some(pyro2)).contains(&angel2));
}

#[test]
fn protection_from_red_stops_ballista_damage() {
    let (mut g, ley) = with_leyline();
    let ballista = perm(&mut g, P0, "Walking Ballista");
    g.perm_mut(ballista).plus = 2;
    let firewalker = perm(&mut g, P1, "Kor Firewalker"); // protection from red
    assert_eq!(colors_of(&g, ballista), all(), "the Ballista is red too");
    assert_eq!(sim_core::engine::life::creature_damage(&g, P0, Some(ballista), firewalker, 1), 0, "prevented");
    zones::bounce(&mut g, ley).unwrap();
    assert_eq!(colors_of(&g, ballista), Colors::NONE);
    assert_eq!(sim_core::engine::life::creature_damage(&g, P0, Some(ballista), firewalker, 1), 1, "control");
}

#[test]
fn back_to_basics_still_applies() {
    let (mut g, _) = with_leyline();
    perm(&mut g, P1, "Back to Basics");
    let island = lands(&mut g, P0, "Island", 1, true)[0];
    let mine = lands(&mut g, P0, "Strip Mine", 1, true)[0];
    // Leyline adds land types, not the basic supertype: Strip Mine is still nonbasic
    assert!(!g.db.get(g.perm(mine).card()).basic);
    let lock = sim_core::impls::t45_urza_yawgmoth::untap_lock(&g, P0);
    assert!(lock.is_some(), "Back to Basics is on the battlefield");
    for l in [island, mine] {
        g.perm_mut(l).tapped = false; // the untap step
    }
    sim_core::impls::t45_urza_yawgmoth::untap_step(&mut g, P0, lock);
    assert!(!g.perm(island).tapped, "the basic land untaps");
    assert!(g.perm(mine).tapped, "the nonbasic land stays tapped");
}

#[test]
fn urborg_adds_to_leyline() {
    let (mut g, _) = with_leyline();
    lands(&mut g, P1, "Urborg, Tomb of Yawgmoth", 1, false);
    let island = lands(&mut g, P0, "Island", 1, false)[0];
    assert_eq!(chars::land_types(&g, island), all(), "Urborg's Swamp is already among them");
    assert_eq!(mana::land_colors_now(&g, P0, island), all());
    // an opponent's land has Urborg's Swamp only
    let theirs = lands(&mut g, P1, "Forest", 1, false)[0];
    assert_eq!(chars::land_types(&g, theirs), Colors::from_letters("GB"));
}

#[test]
fn blood_moon_interaction() {
    // Policy: the engine has no timestamps, so Blood Moon always wins over Leyline for a nonbasic land. By the rules the
    // later of the two would. A basic land is not touched by Blood Moon and keeps Leyline's types.
    let (mut g, _) = with_leyline();
    let mine = lands(&mut g, P0, "Strip Mine", 1, false)[0];
    let island = lands(&mut g, P0, "Island", 1, false)[0];
    perm(&mut g, P1, "Blood Moon");
    assert_eq!(chars::land_types(&g, mine), Colors::from_letters("R"), "Blood Moon wins: a Mountain only");
    assert_eq!(mana::land_colors_now(&g, P0, mine), Colors::from_letters("R"));
    assert_eq!(chars::land_types(&g, island), all(), "a basic land keeps Leyline's types");
    assert_eq!(mana::land_colors_now(&g, P0, island), all());
    let units = mana_units(&g, P0, false);
    assert_eq!(units.iter().find(|u| u.src == Source::Land(mine)).unwrap().cols, Colors::from_letters("R"));
}

// ------------------------------------------------------------------ the opening hand
#[test]
fn opening_hand_ai_begins_with_it() {
    for seat_no in [0usize, 1] {
        let mut g = game();
        let p = PlayerId(seat_no as u8);
        let c = hand(&mut g, p, &["Leyline of the Guildpact", "Island"])[0];
        let n = g.player(p).hand.len();
        opening_hand_actions(&mut g, p);
        assert!(!g.player(p).hand.contains(&c), "out of the hand");
        assert_eq!(g.player(p).hand.len(), n - 1, "only Leyline left it");
        let m = *g.player(p).perms.iter().find(|&&m| g.perm(m).cd == Some(c)).expect("on the battlefield");
        assert!(g.hooks.contains(&m) && !g.perm(m).sick, "listed, in play from the start");
        assert_eq!(g.stack.len(), 0, "it was not cast");
        let angel = perm(&mut g, p, "Serra Angel");
        assert_eq!(colors_of(&g, angel), all(), "and its effect is on");
    }
}

#[test]
fn opening_hand_person_asked_yes_no() {
    // yes: it begins on the battlefield ("yes" is the first choice)
    let mut g = game();
    let c = hand(&mut g, P0, &["Leyline of the Guildpact", "Island"])[0];
    let h = seat(&mut g, P0, vec![pick(0)]);
    opening_hand_person(&mut g, P0).unwrap();
    let asked = h.asked();
    assert_eq!(asked.len(), 1, "one question");
    assert!(asked[0].prompt.contains("Leyline of the Guildpact"), "{}", asked[0].prompt);
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).cd == Some(c)));
    assert!(!g.player(P0).hand.contains(&c));
    // no: it stays in the hand (to be cast later)
    let mut g = game();
    let c = hand(&mut g, P0, &["Leyline of the Guildpact", "Island"])[0];
    let h = seat(&mut g, P0, vec![pick(1)]);
    opening_hand_person(&mut g, P0).unwrap();
    assert_eq!(h.asked().len(), 1);
    assert!(g.player(P0).hand.contains(&c));
    assert!(g.player(P0).perms.iter().all(|&m| g.perm(m).cd != Some(c)));
    // a hand without it: no question; an AI seat: no question and no change
    let mut g = game();
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Island"]);
    let h = seat(&mut g, P0, vec![]);
    opening_hand_person(&mut g, P0).unwrap();
    assert!(h.asked().is_empty());
    let c = hand(&mut g, P1, &["Leyline of the Guildpact"])[0];
    opening_hand_person(&mut g, P1).unwrap();
    assert!(g.player(P1).hand.contains(&c), "an AI seat is handled by opening_hand_actions, not asked");
}

#[test]
fn mulligan_bottomed_leyline_not_put_in() {
    // a Leyline put on the bottom of the library by a mulligan was not in the opening hand: it stays there
    let mut g = game();
    let c = card(&g, "Leyline of the Guildpact");
    g.player_mut(P1).hand.retain(|&x| x != c);
    g.player_mut(P1).library.retain(|&x| x != c);
    g.player_mut(P1).library.insert(0, c);
    opening_hand_actions(&mut g, P1);
    assert_eq!(g.player(P1).library.first(), Some(&c), "still on the bottom");
    assert!(g.player(P1).perms.iter().all(|&m| g.perm(m).cd != Some(c)));
}
