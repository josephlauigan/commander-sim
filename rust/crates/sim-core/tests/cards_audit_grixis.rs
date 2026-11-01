//! AI audit, phase 2: Sauron and Kefka (audit/ai-audit/sauron.md, kefka.md).

use sim_core::ai::decks;
use sim_core::engine::zones;
use sim_core::ids::PlayerId;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn sauron_table() -> sim_core::state::Game {
    let mut g = table(&["sauron", "veyran"]);
    let h: Vec<_> = g.player_mut(P0).hand.drain(..).collect();
    g.player_mut(P0).library.extend(h);
    lands(&mut g, P0, "Island", 3, false);
    lands(&mut g, P0, "Swamp", 3, false);
    lands(&mut g, P0, "Mountain", 2, false);
    g.player_mut(P0).turns = 8;
    g
}

#[test]
fn sauron_tutor_never_takes_the_old_list_or_a_lone_assault() {
    let mut g = sauron_table();
    perm(&mut g, P0, "The Ozolith");
    zones::amass(&mut g, P0, 2).unwrap();
    let pick = decks::tutor_pick(&g, P0, "any").unwrap();
    assert_ne!(&*g.db.get(pick).name, "Aggravated Assault");
}

#[test]
fn sauron_tutor_ringsight_wants_the_archivist_with_bowmasters_out() {
    let mut g = sauron_table();
    perm(&mut g, P0, "The Ozolith");
    perm(&mut g, P0, "Orcish Bowmasters");
    zones::amass(&mut g, P0, 2).unwrap();
    assert_eq!(decks::tutor_pick(&g, P0, "ubr"), Some(card(&g, "Jace's Archivist")));
}

#[test]
fn sauron_tutor_fetches_sol_ring_when_short_of_mana() {
    let mut g = table(&["sauron", "veyran"]);
    lands(&mut g, P0, "Island", 2, false);
    g.player_mut(P0).turns = 3;
    zones::amass(&mut g, P0, 1).unwrap();
    assert_eq!(decks::tutor_pick(&g, P0, "any"), Some(card(&g, "Sol Ring")));
}

#[test]
fn sauron_holds_mana_up_for_two_counters_more_than_for_one() {
    use sim_core::ai::brain;
    let mut g = sauron_table();
    let h: Vec<_> = g.player_mut(P0).hand.drain(..).collect();
    g.player_mut(P0).library.extend(h);
    hand(&mut g, P0, &["Counterspell"]);
    let s = brain::Situation::new(&g, P0);
    let (c1, v1) = brain::hold_value(&g, P0, &s);
    assert!(c1.is_some());
    assert!(v1 >= 1.5, "one counter is worth holding mana for: {v1}");
    hand(&mut g, P0, &["Arcane Denial"]);
    let (_, v2) = brain::hold_value(&g, P0, &s);
    assert!(v2 > v1);
}

fn kefka_hand(names: &[&str]) -> sim_core::state::Game {
    let mut g = table(&["kefka", "veyran"]);
    let h: Vec<_> = g.player_mut(P0).hand.drain(..).collect();
    g.player_mut(P0).library.extend(h);
    hand(&mut g, P0, names);
    g
}

fn kefka_discards(g: &sim_core::state::Game) -> String {
    let h = g.player(P0).hand.clone();
    let c = sim_core::impls::kefka::discard_choice(g, P0, &h, 0).unwrap();
    g.db.get(c).name.to_string()
}

#[test]
fn kefka_keeps_breach_and_will_over_a_land_while_a_storm_card_is_left() {
    let g = kefka_hand(&["Underworld Breach", "Yawgmoth's Will", "Island"]);
    assert_eq!(kefka_discards(&g), "Island");
}

#[test]
fn kefka_keeps_protection_over_a_land() {
    let g = kefka_hand(&["Deflecting Swat", "Slip Out the Back", "Island"]);
    assert_eq!(kefka_discards(&g), "Island");
}

