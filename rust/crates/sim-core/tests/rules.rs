//! Core rules on hand-built positions: Python's tests/test_rules.py (the classes M2 covers: Mana, StateChecks,
//! Counterspells, Removal). Each expectation comes from the game rules or the card's Oracle text.

use sim_core::engine::{life, mana, removal, stack, zones};
use sim_core::ids::PlayerId;
use sim_core::state::Ctx;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

// ------------------------------------------------------------------ Mana
#[test]
fn colours_must_match() {
    let mut g = table(&["veyran", "seph"]);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Mountain", 1, false);
    assert!(mana::can_pay(&g, P0, 1, "UR", false));
    assert!(!mana::can_pay(&g, P0, 0, "RR", false));
    assert!(!mana::can_pay(&g, P0, 0, "UUU", false));
    assert!(!mana::can_pay(&g, P0, 3, "U", false)); // four mana from three lands
}

#[test]
fn paying_taps_the_sources() {
    let mut g = table(&["veyran", "seph"]);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Mountain", 1, false);
    assert!(mana::pay(&mut g, P0, 0, "UU", false).unwrap());
    let tapped: Vec<bool> = g.player(P0).lands.iter().map(|&l| g.land(l).tapped).collect();
    assert_eq!(tapped, [true, true, false]);
    assert!(!mana::pay(&mut g, P0, 0, "U", false).unwrap()); // nothing blue left
}

#[test]
fn sol_ring_makes_two() {
    let mut g = table(&["veyran", "seph"]);
    perm(&mut g, P0, "Sol Ring");
    assert_eq!(mana::total_mana(&g, P0, false), 2);
}

#[test]
fn treasure_is_spent_last() {
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Swamp", 1, false);
    zones::add_treasure(&mut g, P0, 1).unwrap();
    mana::pay(&mut g, P0, 1, "", false).unwrap();
    assert_eq!(g.player(P0).treasures, 1);
    assert!(g.land(g.player(P0).lands[0]).tapped);
}

#[test]
fn talisman_hurts_only_for_coloured_mana() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Talisman of Hierarchy");
    mana::pay(&mut g, P0, 1, "", false).unwrap();
    assert_eq!(g.player(P0).life, 40);
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Talisman of Hierarchy");
    mana::pay(&mut g, P0, 0, "B", false).unwrap();
    assert_eq!(g.player(P0).life, 39);
}

#[test]
fn commander_tax() {
    let mut g = table(&["seph", "veyran"]);
    let cmd = g.player(P0).cmd;
    assert_eq!(mana::cost_of(&g, P0, cmd), (3, "GWUB".to_string())); // Atraxa, Grand Unifier
    g.player_mut(P0).tax = 4;
    assert_eq!(mana::cost_of(&g, P0, cmd), (7, "GWUB".to_string()));
}

// ------------------------------------------------------------------ StateChecks
#[test]
fn zero_life_eliminates() {
    let mut g = table(&["seph", "veyran", "sauron"]);
    g.player_mut(P1).life = 0;
    life::check_state(&mut g).unwrap();
    assert!(!g.player(P1).alive);
    assert!(!g.over);
}

#[test]
fn twenty_one_commander_damage_eliminates() {
    let mut g = table(&["seph", "veyran"]);
    g.player_mut(P1).cmd_dmg[0] = 21;
    life::check_state(&mut g).unwrap();
    assert!(!g.player(P1).alive && g.over);
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn twenty_commander_damage_does_not() {
    let mut g = table(&["seph", "veyran"]);
    g.player_mut(P1).cmd_dmg[0] = 20;
    life::check_state(&mut g).unwrap();
    assert!(g.player(P1).alive);
}

#[test]
fn drawing_from_an_empty_library_loses() {
    let mut g = table(&["seph", "veyran"]);
    g.player_mut(P1).library.clear();
    zones::draw(&mut g, P1, 1, false).unwrap();
    life::check_state(&mut g).unwrap();
    assert!(!g.player(P1).alive);
}

#[test]
fn ten_poison_counters_lose() {
    let mut g = table(&["seph", "veyran"]);
    g.player_mut(P1).poison = 10;
    life::check_state(&mut g).unwrap();
    assert!(!g.player(P1).alive);
}

// ------------------------------------------------------------------ Counterspells
#[test]
fn what_each_counter_can_hit() {
    let g = table(&["seph", "veyran"]);
    let c = |n| card(&g, n);
    let (titan, tutor, ring) = (c("Grave Titan"), c("Demonic Tutor"), c("Sol Ring"));
    assert!(stack::counter_ok(&g, c("Counterspell"), titan));
    assert!(!stack::counter_ok(&g, c("Spell Pierce"), titan)); // noncreature only
    assert!(stack::counter_ok(&g, c("Spell Pierce"), tutor));
    assert!(stack::counter_ok(&g, c("Disdainful Stroke"), titan)); // mana value 4 or more
    assert!(!stack::counter_ok(&g, c("Disdainful Stroke"), ring));
    assert!(!stack::counter_ok(&g, c("Dovin's Veto"), titan)); // noncreature only
    assert!(stack::counter_ok(&g, c("Swan Song"), c("Aura Shards"))); // enchantment
    assert!(!stack::counter_ok(&g, c("Swan Song"), titan));
}

#[test]
fn a_counter_needs_its_mana() {
    let mut g = table(&["veyran", "seph"]);
    hand(&mut g, P0, &["Counterspell"]);
    let ls = lands(&mut g, P0, "Island", 2, false);
    let titan = card(&g, "Grave Titan");
    assert_eq!(stack::pick_counter(&g, P0, titan), Some(card(&g, "Counterspell")));
    g.land_mut(ls[0]).tapped = true;
    assert_eq!(stack::pick_counter(&g, P0, titan), None);
}

#[test]
fn force_of_will_exiles_a_blue_card_and_one_life() {
    let mut g = table(&["veyran", "seph"]);
    let cs = hand(&mut g, P0, &["Force of Will", "Think Twice"]);
    let titan = card(&g, "Grave Titan");
    assert_eq!(stack::pick_counter(&g, P0, titan), Some(cs[0]));
    assert!(stack::cast_counter(&mut g, P0, cs[0], None).unwrap());
    assert_eq!(g.player(P0).life, 39);
    assert_eq!(g.player(P0).exile, [cs[1]]);
    assert!(g.player(P0).hand.is_empty());
}

// ------------------------------------------------------------------ Removal
#[test]
fn swords_exiles_and_its_controller_gains_its_power() {
    let mut g = table(&["seph", "sauron"]);
    let t = perm(&mut g, P1, "Grave Titan"); // 6/6
    let swords = card(&g, "Swords to Plowshares");
    removal::apply_removal(&mut g, Some(P0), t, "exile", Some(swords)).unwrap();
    assert!(!g.player(P1).perms.contains(&t));
    assert_eq!(g.player(P1).exile, [card(&g, "Grave Titan")]);
    assert_eq!(g.player(P1).life, 46);
}

#[test]
fn path_gives_a_tapped_land() {
    let mut g = table(&["seph", "sauron"]);
    let t = perm(&mut g, P1, "Hellkite Tyrant");
    let path = card(&g, "Path to Exile");
    removal::apply_removal(&mut g, Some(P0), t, "exile", Some(path)).unwrap();
    let tapped: Vec<bool> = g.player(P1).lands.iter().map(|&l| g.land(l).tapped).collect();
    assert_eq!(tapped, [true]);
}

#[test]
fn bolt_kills_three_toughness_only() {
    let mut g = table(&["veyran", "sauron"]);
    let small = perm(&mut g, P1, "Jace's Archivist"); // 2/3
    let big = perm(&mut g, P1, "Hellkite Tyrant"); // 6/5
    let bolt = card(&g, "Lightning Bolt");
    assert_eq!(removal::legal_targets(&g, P0, "dmg3", "c", false, Some(bolt)), [small]);
    removal::apply_removal(&mut g, Some(P0), small, "dmg3", Some(bolt)).unwrap();
    assert!(!g.player(P1).perms.contains(&small) && g.player(P1).perms.contains(&big));
    assert_eq!(g.player(P1).gy, [card(&g, "Jace's Archivist")]);
}

#[test]
fn a_removed_token_leaves_no_card() {
    let mut g = table(&["veyran", "sauron"]);
    let t = token(&mut g, P1, 2);
    let pongify = card(&g, "Pongify");
    removal::apply_removal(&mut g, Some(P0), t, "destroy", Some(pongify)).unwrap();
    assert!(!g.player(P1).perms.contains(&t));
    assert!(g.player(P1).gy.is_empty());
}

#[test]
fn toxic_deluge_kills_creatures_only() {
    let mut g = table(&["seph", "sauron"]);
    perm(&mut g, P1, "Grave Titan");
    perm(&mut g, P0, "Blood Artist");
    perm(&mut g, P1, "Sol Ring");
    removal::apply_wipe(&mut g, P0, "minus", None, &Ctx::default()).unwrap();
    let names: Vec<&str> = g.players.iter().flat_map(|q| q.perms.iter()).map(|&m| g.perm(m).name).collect();
    assert_eq!(names, ["Sol Ring"]);
}

// ------------------------------------------------------------------ TeferisProtection
// "Until your next turn, your life total can't change and you gain protection from everything. All permanents you
// control phase out. Exile Teferi's Protection."
#[test]
fn teferis_protection_effects() {
    let mut g = table(&["seph", "sauron"]);
    let (s, r) = (P0, P1);
    lands(&mut g, s, "Plains", 3, false);
    perm(&mut g, s, "Grave Titan");
    hand(&mut g, s, &["Teferi's Protection"]);
    assert!(zones::cast_teferis_protection(&mut g, s, 8.0).unwrap());
    assert!(g.player(s).perms.iter().all(|&m| g.perm(m).phased));
    assert!(g.player(s).lands.iter().all(|&l| g.land(l).tapped)); // its lands phased out too: no mana
    assert_eq!(g.player(s).exile, [card(&g, "Teferi's Protection")]);
    life::lose_life(&mut g, s, 10, Some(r), "other", None).unwrap();
    life::gain(&mut g, s, 5).unwrap();
    assert_eq!(g.player(s).life, 40); // life can't change
    assert!(life::prevents_damage(&g, s, Some(r))); // protection from everything
    g.active = Some(s);
    sim_core::engine::turn::step_start(&mut g, s).unwrap(); // until your next turn
    assert!(!g.player(s).perms.iter().any(|&m| g.perm(m).phased));
    assert!(!(g.player(s).life_locked || g.player(s).ring_prot));
}

#[test]
fn teferis_protection_survives_a_damage_combo_but_not_an_alternate_win() {
    let mut g = table(&["sauron", "seph", "veyran"]);
    let (r, s, v) = (P0, P1, P2);
    lands(&mut g, s, "Plains", 3, false);
    hand(&mut g, s, &["Teferi's Protection"]);
    sim_core::ai::win(&mut g, r, "combo", None).unwrap();
    assert!(g.player(s).alive && !g.player(v).alive && !g.over);
    let mut g = table(&["sauron", "seph", "veyran"]);
    lands(&mut g, s, "Plains", 3, false);
    hand(&mut g, s, &["Teferi's Protection"]);
    sim_core::ai::win(&mut g, r, "combo", Some(false)).unwrap(); // Thassa's Oracle, mill
    assert!(!g.player(s).alive && g.over);
}

#[test]
fn teferis_protection_needs_its_mana() {
    let mut g = table(&["sauron", "seph"]);
    lands(&mut g, P1, "Plains", 2, false);
    hand(&mut g, P1, &["Teferi's Protection"]);
    assert!(!zones::cast_teferis_protection(&mut g, P1, 8.0).unwrap());
}

// ------------------------------------------------------------------ test_game_changers.Mana
#[test]
fn ancient_tomb_is_not_tapped_when_it_would_kill_you() {
    let mut g = table(&["veyran", "seph"]);
    lands(&mut g, P0, "Ancient Tomb", 1, false);
    assert_eq!(mana::total_mana(&g, P0, false), 2);
    g.player_mut(P0).life = 2;
    assert_eq!(mana::total_mana(&g, P0, false), 0);
}

#[test]
fn mox_diamond_put_onto_the_battlefield_still_needs_a_land() {
    let mut g = table(&["kinnan-simic-mana-combo", "veyran"]);
    let mox = card(&g, "Mox Diamond");
    zones::enter(&mut g, P0, mox, zones::Enter::default()).unwrap(); // Urza's Saga's chapter III, no land in hand
    assert!(!g.player(P0).perms.iter().any(|&m| g.perm(m).name == "Mox Diamond"));
    assert!(g.player(P0).gy.contains(&mox));
}

// ------------------------------------------------------------------ test_land_entry.LandEntersTriggers
#[test]
fn a_gain_land_gains_life_and_each_land_only_once() {
    let mut g = table(&["yshtola", "veyran"]);
    let life0 = g.player(P0).life;
    let barrens = card(&g, "Scoured Barrens");
    g.add_land(P0, barrens, true); // PORT(M3): through ais.play_land_card
    life::check_state(&mut g).unwrap();
    zones::landfall(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).life, life0 + 1);
}

#[test]
fn a_pact_is_affordable_from_everything_untapped() {
    use sim_core::engine::cast::pact_affordable;
    let mut g = table(&["veyran", "seph"]);
    let ls = lands(&mut g, P0, "Island", 5, true); // all tapped now
    assert!(pact_affordable(&g, P0, "3UU")); // Pact of Negation: everything untaps at the upkeep
    assert!(g.land(ls[0]).tapped); // the real game is untouched
    g.player_mut(P0).pact_debts.push((1, String::new()));
    assert!(!pact_affordable(&g, P0, "3UU")); // a debt already owed
}

// ------------------------------------------------------------------ Mulligan
#[test]
fn kept_hands() {
    for seed in 0..40 {
        let mut g = table(&["seph", "veyran"]);
        let mut r = sim_core::rng::Rng::from_u64(seed);
        sim_core::engine::turn::mulligan(&mut g, P0, &mut r);
        let pl = g.player(P0);
        assert_eq!(pl.hand.len() + pl.library.len(), 99); // the commander is in the command zone
        assert!((5..=7).contains(&pl.hand.len()));
        if pl.hand.len() == 7 && !pl.stats.contains_key("mulls") {
            let lands = pl.hand.iter().filter(|&&c| g.db.get(c).land).count();
            assert!((2..=5).contains(&lands));
        }
    }
}

// ------------------------------------------------------------------ Combat
use sim_core::engine::combat;

#[test]
fn an_unblocked_commander_deals_commander_damage() {
    let mut g = table(&["sauron", "veyran"]);
    let w = perm(&mut g, P0, "Witch-king, Bringer of Ruin"); // 5/3 flying
    g.perm_mut(w).is_cmd = true;
    combat::resolve_combat(&mut g, P0, &[w], P1, &[]).unwrap();
    assert_eq!(g.player(P1).life, 35);
    assert_eq!(g.player(P1).cmd_dmg[0], 5);
}

#[test]
fn flyers_need_flying_or_reach_to_block() {
    let mut g = table(&["sauron", "veyran"]);
    let blocker = perm(&mut g, P1, "Murmuring Mystic");
    let flyer = perm(&mut g, P0, "Witch-king, Bringer of Ruin");
    let ground = perm(&mut g, P0, "Grave Titan");
    assert!(!combat::can_block(&g, blocker, flyer));
    assert!(combat::can_block(&g, blocker, ground));
}

#[test]
fn a_deathtouch_blocker_kills_a_big_attacker() {
    let mut g = table(&["sauron", "seph"]);
    let k = perm(&mut g, P0, "Kaervek the Merciless"); // 5/4
    let imp = perm(&mut g, P1, "Stinkweed Imp"); // 1/2 flying, deathtouch
    combat::resolve_combat(&mut g, P0, &[k], P1, &[]).unwrap();
    assert!(!g.perm(k).on_bf && !g.perm(imp).on_bf);
    assert_eq!(g.player(P1).life, 40);
}

#[test]
fn lifelink() {
    let mut g = table(&["seph", "veyran"]);
    let a = perm(&mut g, P0, "Atraxa, Grand Unifier"); // 7/7 lifelink
    g.player_mut(P0).life = 30;
    combat::resolve_combat(&mut g, P0, &[a], P1, &[]).unwrap();
    assert_eq!((g.player(P0).life, g.player(P1).life), (37, 33));
}
