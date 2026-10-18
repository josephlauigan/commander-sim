//! Tier 6 lands batch (impls/t6_lands.rs) on hand-built positions. Each expectation follows the card's Oracle text.

use sim_core::engine::{mana, turn};
use sim_core::ids::PlayerId;
use sim_core::state::{DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn pod() -> Game {
    table(&["lathril-golgari-elves", "veyran"])
}

fn play_land(g: &mut Game, p: PlayerId, name: &str) {
    let c = take(g, p, name);
    turn::play_land_card(g, p, c, "plays").unwrap();
}

fn names(g: &Game, p: PlayerId) -> Vec<String> {
    g.player(p).lands.iter().map(|&l| g.db.get(g.land(l).cd).name.to_string()).collect()
}

// ------------------------------------------------------------------ City of Brass
#[test]
fn city_of_brass_deals_one_damage_each_time_it_is_tapped() {
    let mut g = pod();
    lands(&mut g, P0, "City of Brass", 1, false);
    let life = g.player(P0).life;
    assert!(mana::pay(&mut g, P0, 1, "", false).unwrap());
    assert_eq!(g.player(P0).life, life - 1, "1 damage when it becomes tapped");
    assert!(g.player(P0).lands.iter().all(|&l| g.land(l).tapped));
}

// ------------------------------------------------------------------ City of Traitors
#[test]
fn city_of_traitors_is_sacrificed_when_you_play_another_land() {
    let mut g = pod();
    lands(&mut g, P0, "City of Traitors", 1, false);
    assert_eq!(mana::total_mana(&g, P0, false), 2, "{{T}}: {{C}}{{C}}");
    play_land(&mut g, P0, "Forest");
    assert_eq!(names(&g, P0), ["Forest"]);
    assert!(g.player(P0).gy.contains(&card(&g, "City of Traitors")));
    // the City itself being played does not sacrifice it, another land afterwards does; the opponent's lands do not
    play_land(&mut g, P0, "City of Traitors");
    assert_eq!(names(&g, P0), ["Forest", "City of Traitors"]);
    play_land(&mut g, P1, "Island");
    assert_eq!(names(&g, P0), ["Forest", "City of Traitors"]);
}

// ------------------------------------------------------------------ Gemstone Caverns
#[test]
fn gemstone_caverns_makes_colourless_unless_it_has_a_luck_counter() {
    let mut g = pod();
    let l = lands(&mut g, P0, "Gemstone Caverns", 1, false)[0];
    let u = mana::mana_units(&g, P0, false);
    assert_eq!(u.len(), 1);
    assert!(u[0].cols.is_empty(), "{{T}}: Add {{C}}");
    g.land_mut(l).data.set(DataKey::Counters, Val::Int(1)); // a luck counter
    let u = mana::mana_units(&g, P0, false);
    assert_eq!(u[0].cols, g.player(P0).ident, "with a luck counter: one mana of any colour");
    assert_eq!(u[0].amt, 1);
}

// ------------------------------------------------------------------ Abstergo Entertainment
#[test]
fn abstergo_taps_for_colourless_and_filters_one_mana_into_any_colour() {
    let mut g = pod();
    lands(&mut g, P0, "Abstergo Entertainment", 1, false);
    let u = mana::mana_units(&g, P0, false);
    assert!(u.len() == 1 && u[0].cols.is_empty(), "alone: {{T}}: Add {{C}} only (nothing pays the {{1}})");
    lands(&mut g, P0, "Forest", 1, false);
    let u = mana::mana_units(&g, P0, false);
    assert_eq!(u.len(), 2);
    assert_eq!(u.iter().map(|x| x.amt).sum::<u32>(), 2, "one mana per land");
    assert!(u.iter().any(|x| x.cols == g.player(P0).ident), "{{1}},{{T}}: one mana of any colour");
}

#[test]
fn abstergo_exiles_itself_returns_a_historic_card_and_exiles_all_graveyards() {
    let mut g = pod();
    let a = lands(&mut g, P0, "Abstergo Entertainment", 1, false)[0];
    lands(&mut g, P0, "Forest", 3, false);
    let sol = take(&mut g, P0, "Sol Ring");
    let bear = take(&mut g, P0, "Llanowar Elves");
    g.player_mut(P0).gy.extend([sol, bear]);
    let junk = take(&mut g, P1, "Llanowar Elves");
    g.player_mut(P1).gy.push(junk);
    let o = sim_core::cardcode::land_options(&mut g, P0, None).unwrap();
    assert_eq!(o.iter().map(|x| x.label.as_str()).collect::<Vec<_>>(), ["Abstergo Entertainment"]);
    assert!(sim_core::ai::act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert!(!g.player(P0).lands.contains(&a), "exiled as a cost");
    assert!(g.player(P0).exile.contains(&card(&g, "Abstergo Entertainment")));
    assert!(g.player(P0).hand.contains(&sol), "the historic card returns");
    assert!(g.player(P0).gy.is_empty() && g.player(P1).gy.is_empty(), "all graveyards exiled");
    assert!(g.player(P0).exile.contains(&bear) && g.player(P1).exile.contains(&junk));
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.land(l).tapped).count(), 3, "{{3}} paid");
}

// ------------------------------------------------------------------ Eiganjo, Seat of the Empire
#[test]
fn eiganjo_channel_deals_four_to_an_attacker_and_costs_one_less_per_legendary_creature() {
    let mut g = pod();
    let e = hand(&mut g, P0, &["Eiganjo, Seat of the Empire"])[0];
    lands(&mut g, P0, "Plains", 3, false);
    let a = perm(&mut g, P1, "Craterhoof Behemoth"); // 5/5: 4 damage doesn't kill it
    let b = perm(&mut g, P1, "Llanowar Elves"); // too small to be worth it
    let big = perm(&mut g, P1, "Kaalia of the Vast"); // 2 toughness, pval 7
    let mut atk = vec![a, b, big];
    let mut assign = vec![];
    for &m in &atk {
        g.perm_mut(m).tapped = true;
    }
    sim_core::cardcode::defend_hooks(&mut g, P1, &mut atk, P0, &mut assign).unwrap();
    assert!(g.perm(a).on_bf && g.perm(b).on_bf, "too big / too small to be worth 4 damage");
    assert!(!g.perm(big).on_bf && !atk.contains(&big), "Kaalia is killed and leaves the attack");
    assert!(!g.player(P0).hand.contains(&e) && g.player(P0).gy.contains(&e), "discarded as the cost");
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.land(l).tapped).count(), 3, "{{2}}{{W}}");
}

#[test]
fn eiganjo_costs_one_less_for_each_legendary_creature() {
    let mut g = pod();
    assert_eq!(sim_core::impls::t6_lands::eiganjo_cost(&g, P0), 2);
    perm(&mut g, P0, "Isshin, Two Heavens as One");
    assert_eq!(sim_core::impls::t6_lands::eiganjo_cost(&g, P0), 1);
}

#[test]
fn eiganjo_channel_also_hits_a_blocker_of_its_own_attack() {
    let mut g = pod();
    let e = hand(&mut g, P0, &["Eiganjo, Seat of the Empire"])[0];
    lands(&mut g, P0, "Plains", 3, false);
    let atk = perm(&mut g, P0, "Craterhoof Behemoth");
    let blk = perm(&mut g, P1, "Kaalia of the Vast");
    let mut assign = vec![(atk, blk)];
    let mut a = vec![atk];
    sim_core::cardcode::defend_hooks(&mut g, P0, &mut a, P1, &mut assign).unwrap();
    assert!(!g.perm(blk).on_bf && assign.is_empty(), "the blocker takes 4 damage and is removed from the blocks");
    assert!(g.player(P0).gy.contains(&e));
}

// ------------------------------------------------------------------ Deserted Temple
#[test]
fn deserted_temple_untaps_a_land_that_makes_enough_mana_to_pay_for_it() {
    let mut g = pod();
    let t = lands(&mut g, P0, "Deserted Temple", 1, false)[0];
    lands(&mut g, P0, "Forest", 1, false);
    let c = lands(&mut g, P0, "City of Traitors", 1, true)[0]; // {C}{C}, tapped
    let o = sim_core::cardcode::land_options(&mut g, P0, None).unwrap();
    assert!(o.is_empty(), "untapping the City nets 2 - 1 - 1 = 0 mana: not worth it");
    lands(&mut g, P0, "Forest", 1, false);
    let _ = (t, c);
    // a land worth 3: City of Traitors is only 2, so use a tapped Gaea's Cradle with three creatures
    let cradle = lands(&mut g, P0, "Gaea's Cradle", 1, true)[0];
    for _ in 0..3 {
        token(&mut g, P0, 1);
    }
    let o = sim_core::cardcode::land_options(&mut g, P0, None).unwrap();
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].label, "Deserted Temple untaps Gaea's Cradle");
    assert!(sim_core::ai::act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert!(!g.land(cradle).tapped, "the Cradle is untapped");
    assert!(g.land(t).tapped, "{{T}}");
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.land(l).tapped).count(), 3, "Temple + the {{1}} + the City");
}

// ------------------------------------------------------------------ Minamo, School at Water's Edge
#[test]
fn minamo_untaps_a_legendary_land_for_u_and_its_own_tap() {
    let mut g = pod();
    let m = lands(&mut g, P0, "Minamo, School at Water's Edge", 1, false)[0];
    lands(&mut g, P0, "Island", 2, false);
    let cradle = lands(&mut g, P0, "Gaea's Cradle", 1, true)[0];
    for _ in 0..3 {
        token(&mut g, P0, 1);
    }
    let o = sim_core::cardcode::land_options(&mut g, P0, None).unwrap();
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].label, "Minamo untaps Gaea's Cradle");
    assert!(sim_core::ai::act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert!(!g.land(cradle).tapped && g.land(m).tapped);
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.land(l).tapped).count(), 2, "Minamo and the Island that paid {{U}}");
}

#[test]
fn minamo_does_nothing_when_the_untap_does_not_pay_for_itself() {
    let mut g = pod();
    lands(&mut g, P0, "Minamo, School at Water's Edge", 1, false);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Gaea's Cradle", 1, true); // no creatures: it makes nothing
    assert!(sim_core::cardcode::land_options(&mut g, P0, None).unwrap().is_empty());
}

// ------------------------------------------------------------------ Emergence Zone
#[test]
fn emergence_zone_sacrifices_itself_and_marks_the_turn_for_flash() {
    use sim_core::impls::t6_lands as t6;
    let mut g = pod();
    let z = lands(&mut g, P0, "Emergence Zone", 1, false)[0];
    lands(&mut g, P0, "Forest", 1, false);
    assert!(t6::emergence_go(&mut g, P0, z.0 as i64).unwrap());
    assert!(!g.player(P0).lands.contains(&z), "sacrificed");
    assert!(g.player(P0).gy.contains(&card(&g, "Emergence Zone")));
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.land(l).tapped).count(), 1, "the {{1}} came from the Forest");
    let flag = sim_core::sym::intern(t6::FLASH_FLAG);
    assert_eq!(g.player(P0).flag_turn.get(&flag), Some(&g.turn_stamp()), "flash this turn");
}

// ------------------------------------------------------------------ Forbidden Orchard
#[test]
fn forbidden_orchard_gives_an_opponent_a_spirit_each_time_it_is_tapped_for_mana() {
    let mut g = pod();
    lands(&mut g, P0, "Forbidden Orchard", 1, false);
    let before = g.player(P1).perms.len();
    assert!(mana::pay(&mut g, P0, 1, "", false).unwrap());
    let toks: Vec<_> = g.player(P1).perms.iter().copied().filter(|&m| g.perm(m).token).collect();
    assert_eq!(g.player(P1).perms.len(), before + 1);
    let s = toks[0];
    assert_eq!((g.perm(s).pow, g.perm(s).tgh), (1, 1));
    assert!(g.perm(s).colors.is_empty(), "colorless");
    assert!(g.perm(s).ttypes.contains(&sim_core::sym::intern("spirit")));
    assert!(g.player(P0).perms.iter().all(|&m| !g.perm(m).token), "the tapper gets nothing");
}

// ------------------------------------------------------------------ Petrified Hamlet
#[test]
fn petrified_hamlet_names_the_opponents_utility_land() {
    use sim_core::impls::t6_lands as t6;
    let mut g = pod();
    lands(&mut g, P1, "Forest", 2, false);
    lands(&mut g, P1, "Mutavault", 1, false);
    play_land(&mut g, P0, "Petrified Hamlet");
    assert_eq!(t6::hamlet_names(&g), [card(&g, "Mutavault")]);
}

#[test]
fn petrified_hamlet_names_nothing_without_a_land_with_abilities_and_still_sacrifices_a_city() {
    use sim_core::impls::t6_lands as t6;
    let mut g = pod();
    lands(&mut g, P1, "Forest", 2, false);
    lands(&mut g, P0, "City of Traitors", 1, false);
    play_land(&mut g, P0, "Petrified Hamlet");
    assert!(t6::hamlet_names(&g).is_empty());
    assert_eq!(names(&g, P0), ["Petrified Hamlet"], "playing the Hamlet sacrifices the City");
}
