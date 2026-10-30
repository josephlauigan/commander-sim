//! Tier 6 "green1" batch (impls/t6_green1.rs): each card on a hand-built position with its Oracle text as the
//! expectation.

use sim_core::ai::act;
use sim_core::hooks::Opt;
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn tbl() -> Game {
    table(&["rograkh-thrasios-creature-combo", "godo", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"])
}

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn untapped(g: &Game, p: PlayerId) -> u32 {
    sim_core::engine::mana::total_mana(g, p, false)
}

// ======================================================== Oboro Breezecaller
#[test]
fn oboro_returns_a_land_and_untaps_cradle() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 2, false);
    let cr = lands(&mut g, P0, "Gaea's Cradle", 1, true)[0];
    let spare = lands(&mut g, P0, "Forest", 1, true)[0];
    for _ in 0..6 {
        token(&mut g, P0, 1);
    }
    let o = perm(&mut g, P0, "Oboro Breezecaller");
    hand(&mut g, P0, &["Sol Ring"]);
    let opts = options(&mut g, o, P0, Some(false));
    assert_eq!(opts.len(), 1);
    let before = untapped(&g, P0);
    let hand_before = g.player(P0).hand.len();
    assert!(run(&mut g, P0, &opts[0]));
    assert!(!g.perm(cr).tapped, "Cradle untapped");
    assert!(!g.perm(spare).on_bf, "the tapped spare Forest was returned");
    assert_eq!(g.player(P0).hand.len(), hand_before + 1, "to its owner's hand");
    // {2} paid from the untapped Forests, Cradle (7 creatures with Oboro) now untapped
    assert_eq!(untapped(&g, P0), before - 2 + 7);
}

#[test]
fn oboro_not_offered_when_the_untap_does_not_pay() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 3, false);
    lands(&mut g, P0, "Forest", 1, true);
    let o = perm(&mut g, P0, "Oboro Breezecaller");
    hand(&mut g, P0, &["Sol Ring"]);
    assert!(options(&mut g, o, P0, Some(false)).is_empty(), "a Forest is not worth {{2}} and a land");
}

#[test]
fn oboro_costs_less_with_training_grounds_but_never_under_one() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 1, false);
    let cr = lands(&mut g, P0, "Gaea's Cradle", 1, true)[0];
    lands(&mut g, P0, "Forest", 1, true);
    for _ in 0..3 {
        token(&mut g, P0, 1);
    }
    let o = perm(&mut g, P0, "Oboro Breezecaller");
    perm(&mut g, P0, "Training Grounds");
    hand(&mut g, P0, &["Sol Ring"]);
    assert_eq!(sim_core::impls::t6_green1::creature_ability_cost(&g, P0, 2), 1);
    let opts = options(&mut g, o, P0, Some(false));
    assert_eq!(opts.len(), 1, "only {{1}} needed: the one untapped Forest pays");
    let before = untapped(&g, P0);
    assert!(run(&mut g, P0, &opts[0]));
    assert!(!g.perm(cr).tapped);
    assert_eq!(untapped(&g, P0), before - 1 + 4);
}

// ======================================================== Badgermole Cub
#[test]
fn cub_adds_an_additional_g_when_a_creature_is_tapped_for_mana() {
    let mut g = tbl();
    perm(&mut g, P0, "Badgermole Cub");
    perm(&mut g, P0, "Birds of Paradise");
    assert_eq!(g.player(P0).floating.g, 0);
    assert!(sim_core::engine::mana::pay(&mut g, P0, 1, "", false).unwrap(), "Birds pays {{1}}");
    assert_eq!(g.player(P0).floating.g, 1, "the additional {{G}}");
}

#[test]
fn cub_ignores_lands_and_two_cubs_add_two() {
    let mut g = tbl();
    perm(&mut g, P0, "Badgermole Cub");
    lands(&mut g, P0, "Forest", 1, false);
    assert!(sim_core::engine::mana::pay(&mut g, P0, 1, "", false).unwrap());
    assert_eq!(g.player(P0).floating.g, 0, "a land is not a creature");
    let mut g = tbl();
    perm(&mut g, P0, "Badgermole Cub");
    perm(&mut g, P0, "Badgermole Cub");
    perm(&mut g, P0, "Birds of Paradise");
    assert!(sim_core::engine::mana::pay(&mut g, P0, 1, "", false).unwrap());
    assert_eq!(g.player(P0).floating.g, 2, "each Cub adds one");
}

// ======================================================== Biomancer's Familiar
#[test]
fn familiar_takes_two_off_creature_abilities_but_never_below_one() {
    use sim_core::impls::t6_green1::creature_ability_cost as cost;
    let mut g = tbl();
    assert_eq!(cost(&g, P0, 4), 4);
    perm(&mut g, P0, "Biomancer's Familiar");
    assert_eq!(cost(&g, P0, 4), 2, "{{4}} becomes {{2}}");
    assert_eq!(cost(&g, P0, 2), 1, "{{2}} becomes {{1}}, not {{0}}");
    assert_eq!(cost(&g, P0, 1), 1, "a {{1}} ability stays {{1}}");
    assert_eq!(cost(&g, P0, 0), 0, "a free ability stays free");
    assert_eq!(cost(&g, PlayerId(1), 4), 4, "only its controller's creatures");
}

#[test]
fn familiar_lets_oboro_run_on_one_mana() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 1, false);
    lands(&mut g, P0, "Gaea's Cradle", 1, true);
    lands(&mut g, P0, "Forest", 1, true);
    for _ in 0..3 {
        token(&mut g, P0, 1);
    }
    let o = perm(&mut g, P0, "Oboro Breezecaller");
    perm(&mut g, P0, "Biomancer's Familiar");
    hand(&mut g, P0, &["Sol Ring"]);
    assert_eq!(options(&mut g, o, P0, Some(false)).len(), 1);
}

// ======================================================== Training Grounds
#[test]
fn training_grounds_stacks_with_the_familiar_and_floors_at_one() {
    use sim_core::impls::t6_green1::creature_ability_cost as cost;
    let mut g = tbl();
    perm(&mut g, P0, "Training Grounds");
    assert_eq!(cost(&g, P0, 4), 2);
    perm(&mut g, P0, "Biomancer's Familiar");
    assert_eq!(cost(&g, P0, 4), 1, "{{2}} less twice would be {{0}}: floored at {{1}}");
    assert_eq!(cost(&g, P0, 6), 2);
    assert_eq!(cost(&g, PlayerId(2), 6), 6, "an opponent's creatures are not discounted");
}

// ======================================================== Earthcraft
#[test]
fn earthcraft_taps_a_creature_to_untap_a_basic_land() {
    let mut g = tbl();
    let e = perm(&mut g, P0, "Earthcraft");
    let f = lands(&mut g, P0, "Forest", 1, true)[0];
    let t = token(&mut g, P0, 1);
    hand(&mut g, P0, &["Sol Ring"]);
    assert!(options(&mut g, e, P0, Some(false)).is_empty(), "before combat an idle-able creature could attack");
    let o = options(&mut g, e, P0, Some(true));
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, P0, &o[0]));
    assert!(g.perm(t).tapped, "the creature is tapped as the cost");
    assert!(!g.perm(f).tapped, "the Forest is untapped");
    assert!(options(&mut g, e, P0, Some(true)).is_empty(), "no tapped basic left, no untapped creature left");
}

#[test]
fn earthcraft_only_untaps_basic_lands_and_needs_an_untapped_creature() {
    let mut g = tbl();
    let e = perm(&mut g, P0, "Earthcraft");
    lands(&mut g, P0, "Gaea's Cradle", 1, true);
    token(&mut g, P0, 1);
    hand(&mut g, P0, &["Sol Ring"]);
    assert!(options(&mut g, e, P0, Some(true)).is_empty(), "Cradle is not a basic land");
    lands(&mut g, P0, "Forest", 1, true);
    let t2 = token(&mut g, P0, 1);
    g.perm_mut(t2).tapped = true;
    assert_eq!(options(&mut g, e, P0, Some(true)).len(), 1, "the untapped token is enough");
}

// ======================================================== Candelabra of Tawnos / Magus of the Candelabra
#[test]
fn candelabra_untaps_x_lands_for_x() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 3, false);
    let cr = lands(&mut g, P0, "Gaea's Cradle", 1, true)[0];
    let fo = lands(&mut g, P0, "Forest", 1, true)[0];
    for _ in 0..5 {
        token(&mut g, P0, 1);
    }
    let c = perm(&mut g, P0, "Candelabra of Tawnos");
    hand(&mut g, P0, &["Sol Ring"]);
    let o = options(&mut g, c, P0, Some(false));
    assert_eq!(o.len(), 1);
    let before = untapped(&g, P0);
    assert!(run(&mut g, P0, &o[0]));
    assert!(g.perm(c).tapped, "{{T}} is a cost");
    assert!(!g.perm(cr).tapped);
    assert!(g.perm(fo).tapped, "X=1: only Cradle (a Forest for {{1}} nets under 2)");
    assert_eq!(untapped(&g, P0), before - 1 + 5);
    assert!(options(&mut g, c, P0, Some(false)).is_empty(), "tapped now");
}

#[test]
fn candelabra_not_used_when_it_does_not_pay() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 3, false);
    lands(&mut g, P0, "Forest", 2, true);
    let c = perm(&mut g, P0, "Candelabra of Tawnos");
    hand(&mut g, P0, &["Sol Ring"]);
    assert!(options(&mut g, c, P0, Some(false)).is_empty());
}

#[test]
fn magus_is_summoning_sick_and_gets_the_discount() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 2, false);
    let cr = lands(&mut g, P0, "Gaea's Cradle", 1, true)[0];
    let fs = lands(&mut g, P0, "Forest", 2, true);
    for _ in 0..5 {
        token(&mut g, P0, 1);
    }
    hand(&mut g, P0, &["Sol Ring"]);
    let m = perm(&mut g, P0, "Magus of the Candelabra");
    g.perm_mut(m).sick = true;
    assert!(options(&mut g, m, P0, Some(false)).is_empty(), "summoning sick");
    g.perm_mut(m).sick = false;
    perm(&mut g, P0, "Training Grounds");
    // Cradle (6 creatures) + 2 Forests = 9 for X=3, cost max(1, 3-2) = 1
    let o = options(&mut g, m, P0, Some(false));
    assert_eq!(o.len(), 1);
    let before = untapped(&g, P0);
    assert!(run(&mut g, P0, &o[0]));
    assert!(!g.perm(cr).tapped && fs.iter().all(|&l| !g.perm(l).tapped));
    assert_eq!(untapped(&g, P0), before - 1 + 6 + 2);
}

// ======================================================== Formidable Speaker
#[test]
fn speaker_discards_to_tutor_a_creature() {
    let mut g = tbl();
    hand(&mut g, P0, &["Forest", "Forest", "Forest", "Sol Ring"]);
    let lib_cre = g.player(P0).library.iter().filter(|&&c| g.db.get(c).creature).count();
    assert!(lib_cre > 0);
    let h = g.player(P0).hand.len();
    let gy = g.player(P0).gy.len();
    perm(&mut g, P0, "Formidable Speaker");
    assert_eq!(g.player(P0).hand.len(), h, "one discarded, one creature found");
    assert_eq!(g.player(P0).gy.len(), gy + 1);
    assert!(g.player(P0).hand.iter().any(|&c| g.db.get(c).creature));
}

#[test]
fn speaker_untaps_cradle_for_one() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 2, false);
    let cr = lands(&mut g, P0, "Gaea's Cradle", 1, true)[0];
    for _ in 0..5 {
        token(&mut g, P0, 1);
    }
    let s = perm(&mut g, P0, "Formidable Speaker");
    hand(&mut g, P0, &["Sol Ring"]);
    g.perm_mut(s).sick = true;
    assert!(options(&mut g, s, P0, Some(false)).is_empty(), "summoning sick");
    g.perm_mut(s).sick = false;
    let o = options(&mut g, s, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, P0, &o[0]));
    assert!(!g.perm(cr).tapped && g.perm(s).tapped);
}

// ======================================================== Vibrance
fn hand_opts(g: &mut Game, p: PlayerId, name: &str) -> Vec<Opt> {
    let c = card(g, name);
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    f(g, c, p, Some(false)).unwrap()
}

#[test]
fn vibrance_evoked_for_rr_deals_3_and_is_sacrificed() {
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 2, false);
    hand(&mut g, P0, &["Vibrance"]);
    assert!(hand_opts(&mut g, P0, "Vibrance").is_empty(), "no worthwhile target for 3 damage yet");
    g.player_mut(PlayerId(1)).life = 3;
    let o = hand_opts(&mut g, P0, "Vibrance");
    assert_eq!(o.len(), 1, "RR only: no {{G}}{{G}} mode");
    assert!(run(&mut g, P0, &o[0]));
    assert_eq!(g.player(PlayerId(1)).life, 0, "3 damage");
    let v = card(&g, "Vibrance");
    assert!(g.player(P0).gy.contains(&v), "sacrificed (evoke)");
    assert!(!g.player(P0).hand.contains(&v));
}

#[test]
fn vibrance_evoked_for_gg_finds_cradle_and_gains_2() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 2, false);
    hand(&mut g, P0, &["Vibrance"]);
    let cr = card(&g, "Gaea's Cradle");
    if !g.player(P0).library.contains(&cr) {
        g.player_mut(P0).library.push(cr);
    }
    let o = hand_opts(&mut g, P0, "Vibrance");
    assert_eq!(o.len(), 1);
    let life = g.player(P0).life;
    let opp = g.player(PlayerId(1)).life;
    assert!(run(&mut g, P0, &o[0]));
    assert!(g.player(P0).hand.contains(&cr), "searched Gaea's Cradle");
    assert_eq!(g.player(P0).life, life + 2);
    assert_eq!(g.player(PlayerId(1)).life, opp, "no damage in the {{G}}{{G}} mode");
    assert!(g.player(P0).hand.is_empty() || !g.player(P0).hand.contains(&card(&g, "Vibrance")));
}

// ======================================================== Springheart Nantuko
#[test]
fn nantuko_makes_an_insect_on_each_landfall_of_its_controller() {
    let mut g = tbl();
    perm(&mut g, P0, "Springheart Nantuko");
    let creatures = |g: &Game, p: PlayerId| g.player(p).perms.iter().filter(|&&m| g.is_creature(m)).count();
    let n = creatures(&g, P0);
    sim_core::engine::zones::landfall(&mut g, P0).unwrap();
    assert_eq!(creatures(&g, P0), n + 1, "a 1/1 Insect");
    let ins = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token).unwrap();
    assert_eq!(sim_core::engine::values::epow(&g, ins), 1);
    sim_core::engine::zones::landfall(&mut g, PlayerId(1)).unwrap();
    assert_eq!(creatures(&g, P0), n + 1, "an opponent's land does nothing");
}
