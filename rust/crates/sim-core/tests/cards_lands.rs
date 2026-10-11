//! Card code from lands.py, combos.py and topdeck.py on hand-built positions: the land abilities the AI uses (creature
//! lands, Kor Haven, War Room, Oran-Rief, Vault of the Archangel, the sacrifice-to-draw lands, Emeria, Mortuary Mire,
//! Mosswort Bridge), a combo's piece threat and progress, Python's test_game_changers.Combos (Thassa's Oracle +
//! Tainted Pact), and Sylvan Library.

use sim_core::ai::act::perform;
use sim_core::cardcode;
use sim_core::engine::{mana, zones};
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PlayerId};
use sim_core::impls::{combos, lands as il};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn labels(o: &[Opt]) -> Vec<&str> {
    o.iter().map(|x| x.label.as_str()).collect()
}

fn go(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn land_named(g: &Game, p: PlayerId, name: &str) -> Option<sim_core::ids::LandId> {
    g.player(p).lands.iter().copied().find(|&l| &*g.db.get(g.land(l).cd).name == name)
}

// ------------------------------------------------------------------ creature lands
#[test]
fn mutavault_animates_into_an_open_attack_and_reverts_next_turn() {
    let mut g = table(&["lathril-golgari-elves", "veyran"]);
    lands(&mut g, P0, "Mutavault", 1, false);
    lands(&mut g, P0, "Forest", 2, false);
    let o = cardcode::land_options(&mut g, P0, Some(false)).unwrap();
    assert_eq!(labels(&o), ["animate Mutavault"]);
    assert!((o[0].utility - 0.7).abs() < 1e-9); // 0.8 + 0.35 * 2, one spare mana: -0.8
    assert!(cardcode::land_options(&mut g, P0, Some(true)).unwrap().is_empty()); // only before combat
    assert!(go(&mut g, P0, &o[0]));
    assert!(land_named(&g, P0, "Mutavault").is_none());
    let m = *g.player(P0).perms.last().unwrap();
    assert_eq!((&*g.perm(m).name, g.perm(m).pow, g.perm(m).tgh), ("Mutavault", 2, 2));
    assert!(g.is_creature(m) && !g.perm(m).sick);
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.land(l).tapped).count(), 1); // {1} from a Forest
    il::revert_animated(&mut g).unwrap();
    assert!(!g.perm(m).on_bf);
    assert!(land_named(&g, P0, "Mutavault").is_some());
    assert!(g.animated.is_empty());
}

#[test]
fn an_animated_land_that_died_goes_to_the_graveyard() {
    let mut g = table(&["lathril-golgari-elves", "veyran"]);
    let l = lands(&mut g, P0, "Mutavault", 1, false)[0];
    let m = il::animate(&mut g, P0, l, 2, 2, &[], false);
    zones::die(&mut g, m, "destroy").unwrap();
    il::revert_animated(&mut g).unwrap();
    assert!(g.player(P0).lands.is_empty());
    assert!(g.player(P0).gy.contains(&card(&g, "Mutavault")));
}

#[test]
fn a_creature_land_stays_home_when_a_bigger_blocker_waits() {
    let mut g = table(&["lathril-golgari-elves", "veyran"]);
    lands(&mut g, P0, "Mutavault", 1, false);
    lands(&mut g, P0, "Forest", 3, false);
    token(&mut g, P1, 3);
    assert!(cardcode::land_options(&mut g, P0, Some(false)).unwrap().is_empty());
}

// ------------------------------------------------------------------ Kor Haven
#[test]
fn kor_haven_stops_the_biggest_unblocked_attacker() {
    let mut g = table(&["light-paws-aura-voltron", "veyran"]);
    lands(&mut g, P0, "Kor Haven", 1, false);
    lands(&mut g, P0, "Plains", 2, false);
    let big = token(&mut g, P1, 5);
    let small = token(&mut g, P1, 2);
    let mut atk = vec![small, big];
    let mut assign = vec![];
    cardcode::defend_hooks(&mut g, P1, &mut atk, P0, &mut assign).unwrap();
    assert_eq!(atk, [small]);
    assert!(g.player(P0).lands.iter().all(|&l| g.land(l).tapped)); // the land and {1}{W}
}

#[test]
fn kor_haven_waits_for_four_power() {
    let mut g = table(&["light-paws-aura-voltron", "veyran"]);
    lands(&mut g, P0, "Kor Haven", 1, false);
    lands(&mut g, P0, "Plains", 2, false);
    let a = token(&mut g, P1, 3);
    let mut atk = vec![a];
    cardcode::defend_hooks(&mut g, P1, &mut atk, P0, &mut vec![]).unwrap();
    assert_eq!(atk, [a]);
    assert!(g.player(P0).lands.iter().all(|&l| !g.land(l).tapped));
}

// ------------------------------------------------------------------ value lands
#[test]
fn war_room_draws_at_the_end_of_a_turn_for_life() {
    let mut g = table(&["light-paws-aura-voltron", "veyran"]);
    lands(&mut g, P0, "War Room", 1, false);
    lands(&mut g, P0, "Plains", 3, false);
    assert!(cardcode::land_options(&mut g, P0, Some(false)).unwrap().is_empty()); // not in a main phase
    let o = cardcode::land_options(&mut g, P0, None).unwrap();
    assert_eq!(labels(&o), ["War Room"]);
    assert!(go(&mut g, P0, &o[0]));
    assert_eq!(g.player(P0).hand.len(), 1);
    assert_eq!(g.player(P0).life, 39); // one colour in Light-Paws's identity
    assert!(g.player(P0).lands.iter().all(|&l| g.land(l).tapped));
    g.player_mut(P0).life = 12;
    for l in g.player(P0).lands.clone() {
        g.land_mut(l).tapped = false;
    }
    assert!(cardcode::land_options(&mut g, P0, None).unwrap().is_empty()); // life too low
}

#[test]
fn oran_rief_counters_new_green_creatures_after_combat() {
    let mut g = table(&["tatyova-simic-landfall", "veyran"]);
    lands(&mut g, P0, "Oran-Rief, the Vastwood", 1, false);
    let a = perm(&mut g, P0, "Llanowar Elves");
    assert!(cardcode::land_options(&mut g, P0, Some(true)).unwrap().is_empty()); // not sick
    g.perm_mut(a).sick = true;
    assert!(cardcode::land_options(&mut g, P0, Some(true)).unwrap().is_empty()); // one is not enough
    let b = perm(&mut g, P0, "Llanowar Elves");
    g.perm_mut(b).sick = true;
    assert!(cardcode::land_options(&mut g, P0, Some(false)).unwrap().is_empty()); // the post-combat main only
    let o = cardcode::land_options(&mut g, P0, Some(true)).unwrap();
    assert_eq!(labels(&o), ["Oran-Rief"]);
    assert!(go(&mut g, P0, &o[0]));
    assert_eq!((g.perm(a).plus, g.perm(b).plus), (1, 1));
}

#[test]
fn vault_of_the_archangel_arms_a_wide_attack() {
    let mut g = table(&["teysa-orzhov-aristocrats", "veyran"]);
    lands(&mut g, P0, "Vault of the Archangel", 1, false);
    lands(&mut g, P0, "Plains", 2, false);
    lands(&mut g, P0, "Swamp", 2, false);
    let ts: Vec<_> = (0..2).map(|_| token(&mut g, P0, 1)).collect();
    assert!(cardcode::land_options(&mut g, P0, Some(false)).unwrap().is_empty()); // two attackers
    let t3 = token(&mut g, P0, 1);
    let o = cardcode::land_options(&mut g, P0, Some(false)).unwrap();
    assert_eq!(labels(&o), ["Vault of the Archangel"]);
    assert!((o[0].utility - 0.9).abs() < 1e-9);
    assert!(go(&mut g, P0, &o[0]));
    for m in ts.into_iter().chain([t3]) {
        assert!(g.perm(m).eot_kw.contains(&"deathtouch") && g.perm(m).eot_kw.contains(&"lifelink"));
    }
}

#[test]
fn silent_clearing_cycles_when_flooded_and_its_mana_costs_life() {
    let mut g = table(&["teysa-orzhov-aristocrats", "veyran"]);
    lands(&mut g, P0, "Silent Clearing", 1, false);
    lands(&mut g, P0, "Plains", 5, false);
    assert!(cardcode::land_options(&mut g, P0, Some(false)).unwrap().is_empty()); // six lands
    assert_eq!(labels(&cardcode::land_options(&mut g, P0, None).unwrap()), ["Silent Clearing draw"]); // five at end of turn
    lands(&mut g, P0, "Plains", 1, false);
    let o = cardcode::land_options(&mut g, P0, Some(false)).unwrap();
    assert_eq!(labels(&o), ["Silent Clearing draw"]);
    assert!((o[0].utility - 0.6).abs() < 1e-9);
    assert!(go(&mut g, P0, &o[0]));
    assert!(land_named(&g, P0, "Silent Clearing").is_none());
    assert!(g.player(P0).gy.contains(&card(&g, "Silent Clearing")));
    assert_eq!(g.player(P0).hand.len(), 1);
    assert_eq!(g.player(P0).life, 40); // paid with a Plains

    let mut g = table(&["teysa-orzhov-aristocrats", "veyran"]);
    lands(&mut g, P0, "Silent Clearing", 1, false);
    assert!(mana::pay(&mut g, P0, 0, "W", false).unwrap());
    assert_eq!(g.player(P0).life, 39);
}

#[test]
fn emeria_returns_the_best_creature_with_seven_plains() {
    let mut g = table(&["light-paws-aura-voltron", "veyran"]);
    lands(&mut g, P0, "Emeria, the Sky Ruin", 1, false);
    lands(&mut g, P0, "Plains", 6, false);
    let c = take(&mut g, P0, "Llanowar Elves");
    g.player_mut(P0).gy.push(c);
    il::land_upkeep(&mut g, P0).unwrap();
    assert!(g.player(P0).perms.is_empty()); // six Plains
    lands(&mut g, P0, "Plains", 1, false);
    cardcode::turn_start(&mut g, P0).unwrap();
    assert!(g.player(P0).gy.is_empty());
    assert_eq!(g.player(P0).perms.iter().filter(|&&m| g.perm(m).cd == Some(c)).count(), 1);
}

#[test]
fn mortuary_mire_puts_the_best_creature_on_top() {
    let mut g = table(&["teysa-orzhov-aristocrats", "veyran"]);
    let elves = take(&mut g, P0, "Llanowar Elves");
    let titan = take(&mut g, P0, "Grave Titan");
    let bolt = take(&mut g, P0, "Lightning Bolt");
    g.player_mut(P0).gy.extend([elves, titan, bolt]);
    let mire = take(&mut g, P0, "Mortuary Mire");
    let l = g.add_land(P0, mire, true);
    cardcode::land_etb(&mut g, P0, l).unwrap();
    assert_eq!(g.player(P0).library.last(), Some(&titan));
    assert_eq!(g.player(P0).gy, [elves, bolt]);
}

#[test]
fn mosswort_bridge_hides_a_spell_and_casts_it_with_ten_power() {
    let mut g = table(&["tatyova-simic-landfall", "veyran"]);
    let names = ["Island", "Llanowar Elves", "Forest", "Grave Titan", "Island"]; // the last on top
    let top: Vec<CardId> = names.iter().map(|n| take(&mut g, P0, n)).collect();
    g.player_mut(P0).library.extend(&top);
    let bridge = take(&mut g, P0, "Mosswort Bridge");
    let n = g.player(P0).library.len();
    let l = g.add_land(P0, bridge, true);
    cardcode::land_etb(&mut g, P0, l).unwrap();
    // the top four looked at; the best spell (Grave Titan: a creature, by mana value) hidden, the rest at the bottom
    assert_eq!(g.player(P0).library.len(), n - 1);
    assert!(!g.player(P0).library.contains(&card(&g, "Grave Titan")));
    assert_eq!(g.player(P0).library.last(), Some(&top[0])); // the fifth card is the new top
    g.land_mut(l).tapped = false;
    lands(&mut g, P0, "Forest", 1, false);
    assert!(cardcode::land_options(&mut g, P0, Some(false)).unwrap().is_empty()); // no power on board
    token(&mut g, P0, 10);
    let o = cardcode::land_options(&mut g, P0, Some(false)).unwrap();
    assert_eq!(labels(&o), ["Mosswort Bridge (Grave Titan)"]);
    assert!(cardcode::land_options(&mut g, P0, None).unwrap().is_empty()); // not at the end of a turn
}

// ------------------------------------------------------------------ combos
#[test]
fn a_combo_piece_is_worth_more_as_its_partner_arrives() {
    let mut g = table(&["yawgmoth-mono-black-aristocrats", "veyran"]);
    g.player_mut(P0).cmd_in_zone = false;
    let bond = perm(&mut g, P0, "Sanguine Bond");
    assert_eq!(cardcode::piece_threat(&g, bond), 2.5); // the other piece one step away
    assert!((cardcode::combo_progress(&g, P0) - 0.25).abs() < 1e-9); // half of Sanguine Bond + Exquisite Blood
    let blood = card(&g, "Exquisite Blood");
    assert_eq!(cardcode::missing_pieces(&g, P0), [blood]);
    assert_eq!(cardcode::combo_imp(&g, P0, blood), 9.0); // it completes the combo
    assert!(cardcode::is_combo_piece(&g, blood));
    assert!(cardcode::combo_pieces(&g).contains(&blood));
    hand(&mut g, P0, &["Exquisite Blood"]);
    assert!((cardcode::combo_progress(&g, P0) - 1.0).abs() < 1e-9);
    assert!(cardcode::combo_ready(&g, P0));
    let b2 = perm(&mut g, P0, "Exquisite Blood");
    assert_eq!(cardcode::piece_threat(&g, b2), 7.0);
    assert_eq!(cardcode::piece_threat(&g, bond), 7.0);
    let t = token(&mut g, P0, 1);
    assert_eq!(cardcode::piece_threat(&g, t), 0.0);
    // a deck without a modeled combo
    assert_eq!(cardcode::combo_progress(&g, P1), 0.0);
}

#[test]
fn an_outside_deck_goes_for_a_ready_combo() {
    let mut g = table(&["yawgmoth-mono-black-aristocrats", "veyran"]);
    lands(&mut g, P0, "Swamp", 5, false);
    perm(&mut g, P0, "Sanguine Bond");
    hand(&mut g, P0, &["Exquisite Blood"]);
    assert!(cardcode::combo_options(&mut g, P0, 0.5, None).unwrap().is_empty()); // not at the end of a turn
    let o = cardcode::combo_options(&mut g, P0, 0.5, Some(false)).unwrap();
    assert_eq!(labels(&o), ["combo: Sanguine Bond + Exquisite Blood"]);
    assert!((o[0].utility - 11.0).abs() < 1e-9); // a spell to cast: 14 - 6 * risk
    let mut g2 = table(&["veyran", "seph"]); // your decks don't run the modeled combos
    assert!(cardcode::combo_options(&mut g2, P0, 0.0, Some(false)).unwrap().is_empty());
}

fn oracle_table(library: &[&str]) -> (Game, usize) {
    let mut g = table(&["veyran", "seph"]);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Swamp", 2, false);
    hand(&mut g, P0, &["Thassa's Oracle", "Tainted Pact"]);
    let lib: Vec<CardId> = library.iter().map(|n| card(&g, n)).collect();
    g.player_mut(P0).library = lib;
    (g, combos::combo_index("Oracle").unwrap())
}

#[test]
fn oracle_and_tainted_pact_win_off_two_islands_and_two_swamps() {
    // Pact used to go first and pay its {1} with an Island, leaving no {U}{U} for the Oracle
    let (mut g, i) = oracle_table(&["Sol Ring", "Counterspell", "Brainstorm", "Ponder"]);
    assert!(combos::ready(&g, P0, i));
    combos::attempt(&mut g, P0, i as i64).unwrap();
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn tainted_pact_needs_a_library_without_duplicate_names() {
    let (g, i) = oracle_table(&["Island", "Island", "Sol Ring"]);
    assert!(!combos::ready(&g, P0, i));
}

#[test]
fn the_oracle_waits_with_a_full_library() {
    let g = table(&["urza-mono-blue-artifacts", "veyran"]);
    let c = card(&g, "Thassa's Oracle");
    assert_eq!((g.registry.get(c).unwrap().prio.unwrap())(&g, P0, c), 0);
}

// ------------------------------------------------------------------ Sylvan Library
fn sylvan_table(life: i32) -> (Game, [CardId; 3]) {
    let mut g = table(&["lathril-golgari-elves", "veyran"]);
    perm(&mut g, P0, "Sylvan Library");
    g.player_mut(P0).life = life;
    let cs = [take(&mut g, P0, "Swamp"), take(&mut g, P0, "Forest"), take(&mut g, P0, "Llanowar Elves")];
    g.player_mut(P0).library.extend(cs); // the Elves on top
    (g, cs)
}

#[test]
fn sylvan_library_keeps_wanted_cards_for_four_life() {
    let (mut g, [swamp, forest, elves]) = sylvan_table(40);
    zones::draw(&mut g, P0, 1, true).unwrap();
    let mut h = g.player(P0).hand.clone();
    h.sort();
    let mut want = vec![swamp, forest, elves];
    want.sort();
    assert_eq!(h, want); // lands are wanted with none in play: both kept
    assert_eq!(g.player(P0).life, 32);
}

#[test]
fn sylvan_library_puts_cards_back_when_life_is_short() {
    let (mut g, [swamp, forest, elves]) = sylvan_table(20);
    zones::draw(&mut g, P0, 1, true).unwrap();
    assert_eq!(g.player(P0).hand, [elves]);
    assert_eq!(g.player(P0).life, 20);
    let lib = &g.player(P0).library;
    let top2 = &lib[lib.len() - 2..];
    assert!(top2.contains(&swamp) && top2.contains(&forest));
    zones::draw(&mut g, P0, 1, false).unwrap(); // once per turn
    assert_eq!(g.player(P0).hand.len(), 2);
}

/// a drawn second copy of a card already in hand is put back too (Python picked the drawn cards by identity and kept
/// it for free: fixed in Rust)
#[test]
fn sylvan_library_puts_back_a_copy_of_a_card_in_hand() {
    let (mut g, [swamp, forest, elves]) = sylvan_table(20);
    g.player_mut(P0).hand.push(forest);
    zones::draw(&mut g, P0, 1, true).unwrap();
    let mut h = g.player(P0).hand.clone();
    h.sort();
    let mut want = vec![forest, elves];
    want.sort();
    assert_eq!(h, want); // the Forest that was in hand and the Elves drawn
    let lib = &g.player(P0).library;
    let top2 = &lib[lib.len() - 2..];
    assert!(top2.contains(&swamp) && top2.contains(&forest));
}

/// Scroll Rack taps when used (Python never tapped it: fixed in Rust): a flooded hand swaps two lands for the two
/// spells on top
#[test]
fn scroll_rack_taps_when_used() {
    let mut g = table(&["lathril-golgari-elves", "veyran"]);
    let rk = perm(&mut g, P0, "Scroll Rack");
    lands(&mut g, P0, "Forest", 8, false);
    g.player_mut(P0).hand = vec![take(&mut g, P0, "Forest"), take(&mut g, P0, "Swamp")];
    let top = [take(&mut g, P0, "Elvish Archdruid"), take(&mut g, P0, "Elvish Warmaster")];
    g.player_mut(P0).library.extend(top);
    let f = g.registry.get(g.perm(rk).cd.unwrap()).and_then(|i| i.upkeep).unwrap();
    f(&mut g, rk, P0).unwrap();
    assert!(top.iter().all(|c| g.player(P0).hand.contains(c))); // it was used
    assert!(g.perm(rk).tapped);
}
