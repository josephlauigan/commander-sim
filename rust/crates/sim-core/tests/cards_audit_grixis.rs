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
fn sauron_tutor_fetches_the_ozolith_once_an_army_is_growing() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = sauron_table();
    zones::amass(&mut g, P0, 4).unwrap();
    assert_eq!(decks::tutor_pick(&g, P0, "any"), Some(card(&g, "The Ozolith")));
    perm(&mut g, P0, "The Ozolith");
    assert_ne!(decks::tutor_pick(&g, P0, "any"), Some(card(&g, "The Ozolith")));
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
fn sauron_tutor_gets_evasion_for_a_huge_army_then_the_combat_free_finisher() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    // Aggravated Assault left the list on 10/30: with evasion out and 8 lands, the tutor goes to Torment of Hailfire
    let mut g = sauron_table();
    perm(&mut g, P0, "The Ozolith");
    perm(&mut g, P0, "Not of This World");
    zones::amass(&mut g, P0, 9).unwrap();
    let pk = decks::tutor_pick(&g, P0, "any").unwrap();
    assert_eq!(&*g.db.get(pk).name, "Whispersilk Cloak");
    perm(&mut g, P0, "Whispersilk Cloak");
    let pk = decks::tutor_pick(&g, P0, "any").unwrap();
    assert_eq!(&*g.db.get(pk).name, "Torment of Hailfire");
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
fn sauron_does_not_sweep_away_its_own_army() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = sauron_table();
    perm(&mut g, P0, "Sauron, the Dark Lord");
    zones::amass(&mut g, P0, 3).unwrap(); // Sauron (7) is the source, the 3-power Army would die
    let q = PlayerId(1);
    perm(&mut g, q, "Llanowar Elves");
    let (ol, ml, _) = decks::wipe_eval(&g, P0, "nib");
    assert!(ml > ol, "ol {ol} ml {ml}: one elf is not worth the Army");
    perm(&mut g, P0, "The Ozolith");
    let (_, ml2, _) = decks::wipe_eval(&g, P0, "nib");
    assert!(ml2 < ml, "with The Ozolith out the counters survive");
}

#[test]
fn sauron_force_of_will_pitches_a_draw_spell_not_the_engine_creature() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = table(&["sauron", "veyran"]);
    let h: Vec<_> = g.player_mut(P0).hand.drain(..).collect();
    g.player_mut(P0).library.extend(h);
    let cs = hand(&mut g, P0, &["Force of Will", "Talrand, Sky Summoner", "Fact or Fiction", "Jace's Archivist"]);
    assert!(sim_core::engine::stack::pay_counter(&mut g, P0, cs[0], None).unwrap());
    let ex = &g.player(P0).exile;
    assert_eq!(ex.len(), 1);
    assert_eq!(&*g.db.get(ex[0]).name, "Fact or Fiction");
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

#[test]
fn sauron_gives_an_opponent_cards_with_cost_of_brilliance_when_bowmasters_is_out() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    use sim_core::ai::brain;
    let mut g = sauron_table();
    let h: Vec<_> = g.player_mut(P0).hand.drain(..).collect();
    g.player_mut(P0).library.extend(h);
    g.active = Some(P0);
    zones::amass(&mut g, P0, 2).unwrap();
    hand(&mut g, P0, &["Cost of Brilliance"]);
    let plain = brain::main_options(&mut g, P0, false).unwrap();
    assert!(plain.iter().any(|o| o.label.starts_with("Cost of Brilliance (draw two")));
    perm(&mut g, P0, "Orcish Bowmasters");
    let with = brain::main_options(&mut g, P0, false).unwrap();
    let o = with.iter().find(|o| o.label.contains("Bowmasters pings")).expect("opponent draws");
    assert!(o.utility > 4.0);
}

#[test]
fn sauron_wheels_with_the_archivist_for_kaervek_or_sauron_when_short_of_cards() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = sauron_table();
    g.player_mut(P0).hand.clear();
    assert!(!decks::archivist_worth(&g, P0));
    perm(&mut g, P0, "Kaervek the Merciless");
    assert!(decks::archivist_worth(&g, P0));
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

#[test]
fn kefka_tutors_fetch_the_crawler_for_a_flipped_kefka_else_the_breach_line() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    use sim_core::state::{DataKey, Val};
    let mut g = kefka_hand(&[]);
    assert_eq!(&*g.db.get(decks::tutor_pick(&g, P0, "any").unwrap()).name, "Underworld Breach");
    let c = g.player(P0).cmd;
    let m = zones::enter(&mut g, P0, c, zones::Enter { sick: false, ..Default::default() }).unwrap();
    g.player_mut(P0).cmd_in_zone = false;
    g.perm_mut(m).is_cmd = true;
    g.perm_mut(m).data.set(DataKey::Ruin, Val::Bool(true));
    let nm = |g: &sim_core::state::Game| g.db.get(decks::tutor_pick(g, P0, "any").unwrap()).name.to_string();
    assert_eq!(nm(&g), "Psychosis Crawler");
    hand(&mut g, P0, &["Psychosis Crawler"]);
    assert_eq!(nm(&g), "Underworld Breach", "the Crawler is the win: back to the Breach line");
}

#[test]
fn kefka_counts_mana_leak_into_open_mana_as_a_weak_counter() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = table(&["kefka", "veyran"]);
    let leak = card(&g, "Mana Leak");
    let denial = card(&g, "Arcane Denial");
    let p1 = PlayerId(1);
    assert_eq!(sim_core::engine::stack::soft_counter_factor(&g, P0, p1, leak), 1.0, "no mana: they cannot pay");
    lands(&mut g, p1, "Island", 4, false);
    assert!(sim_core::engine::stack::soft_counter_factor(&g, P0, p1, leak) < 0.5);
    assert_eq!(sim_core::engine::stack::soft_counter_factor(&g, P0, p1, denial), 1.0, "a hard counter is unchanged");
}

#[test]
fn kefka_suspends_profane_tutor_on_an_empty_turn() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    use sim_core::ai::brain;
    let mut g = table(&["kefka", "veyran"]);
    let h: Vec<_> = g.player_mut(P0).hand.drain(..).collect();
    g.player_mut(P0).library.extend(h);
    lands(&mut g, P0, "Swamp", 2, false);
    g.active = Some(P0);
    hand(&mut g, P0, &["Profane Tutor", "Massacre Wurm"]);
    let u = |g: &mut sim_core::state::Game| {
        brain::main_options(g, P0, false).unwrap().iter().find(|o| o.label == "suspend Profane Tutor").unwrap().utility
    };
    let empty = u(&mut g);
    hand(&mut g, P0, &["Lotus Petal"]);
    let busy = u(&mut g);
    assert!(empty > busy, "{empty} vs {busy}");
}
