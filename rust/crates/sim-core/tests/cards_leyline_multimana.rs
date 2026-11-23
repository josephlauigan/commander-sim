//! Leyline of the Guildpact makes every land every basic land type, so a land that makes several mana (Ancient Tomb's
//! {C}{C}, Gaea's Cradle) ALSO has the basic types' ability "{T}: Add one mana of any color". Two abilities of one land:
//! a tap uses one of them, never both (`mana::Shape::Alt`). Each test checks the Oracle text, not the old behaviour.

mod play_common;
use play_common::*;
use sim_core::cards::Colors;
use sim_core::engine::mana::{self, Shape, Source};

fn game() -> Game {
    table(&["thanos", "kraum-tymna-blue-breach-midrange", "sisay-legends-toolbox"])
}

fn with_leyline() -> Game {
    let mut g = game();
    perm(&mut g, P0, "Leyline of the Guildpact");
    g
}

fn tomb(g: &mut Game) -> PermId {
    lands(g, P0, "Ancient Tomb", 1, false)[0]
}

fn unit_of(g: &Game, l: PermId) -> mana::Unit {
    *mana::mana_units(g, P0, false).iter().find(|u| u.src == Source::Land(l)).expect("a unit for the land")
}

fn life(g: &Game) -> i32 {
    g.player(P0).life
}

fn wubrg() -> Colors {
    Colors::from_letters("WUBRG")
}

#[test]
fn tomb_under_leyline_is_one_unit_with_two_alternative_abilities() {
    let mut g = with_leyline();
    let t = tomb(&mut g);
    let u = unit_of(&g, t);
    assert_eq!((u.cols, u.amt, u.shape), (wubrg(), 2, Shape::Alt(Colors::NONE)));
    assert_eq!(mana::mana_units(&g, P0, false).len(), 1, "one land, one unit: it cannot be tapped twice");
    assert_eq!(mana::land_colors_now(&g, P0, t), wubrg(), "it can make any colour (Fellwar Stone, colours_have)");
}

#[test]
fn tomb_pays_one_coloured_mana_with_no_damage() {
    for c in "WUBRG".chars() {
        let mut g = with_leyline();
        let t = tomb(&mut g);
        let pips = c.to_string();
        assert!(mana::can_pay(&g, P0, 0, &pips, false), "{{{c}}} from Ancient Tomb under Leyline");
        assert!(mana::pay(&mut g, P0, 0, &pips, false).unwrap());
        assert!(g.perm(t).tapped);
        assert_eq!(life(&g), 40, "Leyline's ability deals no damage");
        assert_eq!(g.player(P0).mana.total(), 0, "its second mana is not made");
    }
}

#[test]
fn tomb_pays_cc_with_two_damage() {
    // {C}{C} as generic mana (the planner may not make a coloured mana of it) ...
    let mut g = with_leyline();
    let t = tomb(&mut g);
    assert!(mana::can_pay(&g, P0, 2, "", false));
    assert!(mana::pay(&mut g, P0, 2, "", false).unwrap());
    assert!(g.perm(t).tapped);
    assert_eq!(life(&g), 38, "its own ability deals 2 damage to you");
    // ... and as two colourless pips (Thanos's power-up)
    let mut g = with_leyline();
    tomb(&mut g);
    assert!(mana::can_pay(&g, P0, 0, "CC", false));
    assert!(mana::pay(&mut g, P0, 0, "CC", false).unwrap());
    assert_eq!(life(&g), 38);
}

#[test]
fn one_generic_mana_uses_leylines_ability_no_damage() {
    let mut g = with_leyline();
    let t = tomb(&mut g);
    assert!(mana::pay(&mut g, P0, 1, "", false).unwrap());
    assert!(g.perm(t).tapped);
    assert_eq!(life(&g), 40, "one mana: Leyline's ability, not the {{C}}{{C}} one");
    assert_eq!(g.player(P0).mana.total(), 0, "and no second mana floats");
}

#[test]
fn own_ability_floats_what_it_did_not_use() {
    // {C} pip from the {C}{C} ability: the other {C} floats (as before Leyline), the damage is taken
    let mut g = with_leyline();
    tomb(&mut g);
    assert!(mana::pay(&mut g, P0, 0, "C", false).unwrap());
    assert_eq!(life(&g), 38);
    assert_eq!(g.player(P0).mana.total(), 1, "the unused {{C}} floats");
}

#[test]
fn five_lands_including_tomb_pay_wubrg_under_leyline() {
    let mut g = with_leyline();
    let t = tomb(&mut g);
    let others = lands(&mut g, P0, "Island", 4, false);
    assert!(mana::can_pay(&g, P0, 0, "WUBRG", false));
    assert!(mana::pay(&mut g, P0, 0, "WUBRG", false).unwrap());
    assert!(g.perm(t).tapped && others.iter().all(|&l| g.perm(l).tapped), "all five lands tapped");
    assert_eq!(life(&g), 40, "the Tomb made one coloured mana: no damage");
    // without Leyline the same five lands cannot (the Tomb makes {C}{C})
    let mut g = game();
    tomb(&mut g);
    lands(&mut g, P0, "Island", 4, false);
    assert!(!mana::can_pay(&g, P0, 0, "WUBRG", false));
}

#[test]
fn never_both_abilities_in_one_tap() {
    let mut g = with_leyline();
    tomb(&mut g);
    // {C}{C} and {W} are 3 mana; one Tomb makes {C}{C} or one coloured mana, never both
    assert!(!mana::can_pay(&g, P0, 0, "CCW", false), "{{C}}{{C}}{{W}} from one Tomb");
    assert!(!mana::can_pay(&g, P0, 2, "W", false), "{{2}}{{W}} from one Tomb");
    assert!(!mana::can_pay(&g, P0, 1, "W", false), "{{1}}{{W}} from one Tomb: two mana, one of them coloured");
    assert!(!mana::can_pay(&g, P0, 0, "WW", false), "two coloured mana from one Tomb");
    assert!(!mana::can_pay(&g, P0, 0, "WU", false));
    assert!(!mana::can_pay(&g, P0, 0, "CW", false));
    assert!(!mana::can_pay(&g, P0, 3, "", false), "three mana");
    // but each alone is fine, and a second land adds what the rules give
    assert!(mana::can_pay(&g, P0, 2, "", false) && mana::can_pay(&g, P0, 0, "W", false));
    lands(&mut g, P0, "Island", 1, false);
    assert!(mana::can_pay(&g, P0, 0, "WU", false), "Tomb makes one, the Island the other");
    assert!(mana::can_pay(&g, P0, 2, "U", false), "Island {{U}}, Tomb {{C}}{{C}}");
    assert!(
        !mana::can_pay(&g, P0, 1, "WU", false),
        "{{1}}{{W}}{{U}}: the Island and one Tomb ability make two mana at most"
    );
}

#[test]
fn tomb_plus_island_costs() {
    // {2}{W} with Tomb + Island: Island {W}, Tomb {C}{C}, 2 damage
    let mut g = with_leyline();
    tomb(&mut g);
    lands(&mut g, P0, "Island", 1, false);
    assert!(mana::pay(&mut g, P0, 2, "W", false).unwrap());
    assert_eq!(life(&g), 38);
    // {W}{U}: no damage at all
    let mut g = with_leyline();
    tomb(&mut g);
    lands(&mut g, P0, "Island", 1, false);
    assert!(mana::pay(&mut g, P0, 0, "WU", false).unwrap());
    assert_eq!(life(&g), 40);
    // two generic: the Island's mana and one of the Tomb's: Leyline's ability, no damage
    let mut g = with_leyline();
    tomb(&mut g);
    lands(&mut g, P0, "Island", 1, false);
    assert!(mana::pay(&mut g, P0, 2, "", false).unwrap());
    assert_eq!(life(&g), 40);
}

#[test]
fn at_two_life_tomb_still_makes_one_coloured_mana() {
    // its own ability would kill you, Leyline's does not
    let mut g = with_leyline();
    let t = tomb(&mut g);
    g.player_mut(P0).life = 2;
    assert!(mana::can_pay(&g, P0, 0, "G", false));
    assert!(!mana::can_pay(&g, P0, 2, "", false), "{{C}}{{C}} is out of reach");
    assert!(!mana::can_pay(&g, P0, 0, "C", false));
    assert!(mana::pay(&mut g, P0, 0, "G", false).unwrap());
    assert!(g.perm(t).tapped);
    assert_eq!(life(&g), 2);
    // without Leyline the Tomb is no source at 2 life
    let mut g = game();
    tomb(&mut g);
    g.player_mut(P0).life = 2;
    assert!(mana::mana_units(&g, P0, false).is_empty());
}

#[test]
fn without_leyline_nothing_changes() {
    let mut g = game();
    let t = tomb(&mut g);
    let u = unit_of(&g, t);
    assert_eq!((u.cols, u.amt, u.shape), (Colors::NONE, 2, Shape::Free));
    assert!(!mana::can_pay(&g, P0, 0, "W", false));
    assert!(mana::pay(&mut g, P0, 1, "", false).unwrap());
    assert_eq!(life(&g), 38, "one generic mana from a Tomb costs the 2 damage");
    assert_eq!(g.player(P0).mana.total(), 1, "and the other {{C}} floats");
    // an opponent's Tomb is not helped by my Leyline
    let mut g = with_leyline();
    let theirs = lands(&mut g, P1, "Ancient Tomb", 1, false)[0];
    let u = *mana::mana_units(&g, P1, false).iter().find(|u| u.src == Source::Land(theirs)).unwrap();
    assert_eq!((u.cols, u.shape), (Colors::NONE, Shape::Free));
    // a Leyline that left: back to {C}{C}
    let mut g = with_leyline();
    let t = tomb(&mut g);
    let ley = perms_named(&g, P0, "Leyline of the Guildpact")[0];
    sim_core::engine::zones::bounce(&mut g, ley).unwrap();
    assert_eq!(unit_of(&g, t).shape, Shape::Free);
}

#[test]
fn gaeas_cradle_gets_the_any_colour_alternative() {
    let mut g = with_leyline();
    let cradle = lands(&mut g, P0, "Gaea's Cradle", 1, false)[0];
    for _ in 0..3 {
        token(&mut g, P0, 1);
    }
    let u = unit_of(&g, cradle);
    assert_eq!((u.cols, u.amt, u.shape), (wubrg(), 3, Shape::Alt(Colors::from_letters("G"))));
    assert!(mana::can_pay(&g, P0, 0, "GGG", false), "its own ability: {{G}} for each creature");
    assert!(mana::can_pay(&g, P0, 0, "W", false), "or one mana of any colour");
    assert!(!mana::can_pay(&g, P0, 0, "WG", false), "not both in one tap");
    assert!(!mana::can_pay(&g, P0, 0, "WW", false));
    assert!(!mana::can_pay(&g, P0, 1, "W", false));
    assert!(mana::can_pay(&g, P0, 1, "GG", false));
    // paying one {G}: the other two {G} float, as for any land
    assert!(mana::pay(&mut g, P0, 0, "G", false).unwrap());
    assert_eq!(g.player(P0).mana.total(), 2);
}

#[test]
fn karoo_and_lotus_are_left_as_they_were() {
    // an "each colour" land has no alternative modelled (see the report): unchanged
    let mut g = with_leyline();
    let karoo = lands(&mut g, P0, "Golgari Rot Farm", 1, false)[0];
    let u = unit_of(&g, karoo);
    assert_eq!((u.cols, u.amt, u.shape), (Colors::from_letters("BG"), 2, Shape::Each));
}

#[test]
fn restricted_workshop_gets_the_alternative_for_artifacts() {
    let mut g = with_leyline();
    let shop = lands(&mut g, P0, "Mishra's Workshop", 1, false)[0];
    g.pay_for = Some(card(&g, "Sol Ring"));
    let u = unit_of(&g, shop);
    assert_eq!((u.cols, u.amt, u.shape), (wubrg(), 3, Shape::Alt(Colors::NONE)));
    g.pay_for = None;
}

#[test]
fn a_person_taps_the_tomb_for_one_coloured_mana_or_two_colourless() {
    use sim_core::human::mana as hm;
    let mut g = with_leyline();
    tomb(&mut g);
    seat(&mut g, P0, vec![]);
    let s = hm::sources(&g, P0);
    assert_eq!(s.len(), 1);
    assert!(s[0].colours.contains('C') && "WUBRG".chars().all(|c| s[0].colours.contains(c)), "{}", s[0].colours);
    assert_eq!(hm::tap(&mut g, P0, s[0].id as i64, Some('U')).unwrap(), None);
    assert_eq!(pool_text(&g, P0), "{U}");
    assert_eq!(life(&g), 40, "Leyline's ability: no damage");
    // the other way
    let mut g = with_leyline();
    tomb(&mut g);
    seat(&mut g, P0, vec![]);
    let s = hm::sources(&g, P0);
    assert_eq!(hm::tap(&mut g, P0, s[0].id as i64, Some('C')).unwrap(), None);
    assert_eq!(pool_total(&g, P0), 2, "{{C}}{{C}}: {}", pool_text(&g, P0));
    assert_eq!(life(&g), 38);
}
