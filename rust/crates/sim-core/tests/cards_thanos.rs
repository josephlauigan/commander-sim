//! JD's `thanos` deck: Thanos, the Mad Titan's power-up (impls/t6_thanos.rs) and the three spells that are free while
//! you control a commander (Fierce Guardianship, Deflecting Swat, Deadly Rollick): the commander must be on the
//! battlefield, not in the command zone.

use sim_core::ai::brain;
use sim_core::engine::mana::can_pay;
use sim_core::engine::removal;
use sim_core::engine::stack::free_counter;
use sim_core::engine::values::commander_out;
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::t2;
use sim_core::impls::t6_thanos::{self, THANOS};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    table(&["thanos", "kraum-tymna-blue-breach-midrange", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"])
}

/// Thanos on the battlefield as P0's commander; `fresh`: he entered this turn
fn thanos(g: &mut Game, fresh: bool) -> PermId {
    let t = perm(g, P0, THANOS);
    g.perm_mut(t).is_cmd = true;
    g.player_mut(P0).cmd_in_zone = false;
    if !fresh {
        g.turn_enter = g.perm(t).born; // he was here before this turn began
    }
    assert_eq!(t6_thanos::entered_this_turn(g, t), fresh);
    t
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

#[test]
fn mana_crypt_flips_only_on_its_controllers_upkeep_after_the_window() {
    let mut g = tbl();
    let m = perm(&mut g, P0, "Mana Crypt");
    let f = g.registry.get(card(&g, "Mana Crypt")).unwrap().upkeep.unwrap();
    let before = g.rng.clone();
    f(&mut g, m, P1).unwrap();
    assert_eq!(g.rng, before, "no flip on another player's upkeep");
    g.trig_probe = true;
    assert!(matches!(f(&mut g, m, P0), Err(sim_core::flow::Stop::Probe(_))));
    assert_eq!(g.rng, before, "probing cannot flip the coin");
    g.player_mut(P0).life = 10000;
    let mut replay = g.clone();
    let mut losses = 0;
    for _ in 0..200 {
        let life = g.player(P0).life;
        f(&mut g, m, P0).unwrap();
        let damage = life - g.player(P0).life;
        assert!(damage == 0 || damage == 3);
        losses += usize::from(damage == 3);
        f(&mut replay, m, P0).unwrap();
        assert_eq!(g.player(P0).life, replay.player(P0).life);
        assert_eq!(g.rng, replay.rng);
    }
    assert!((60..140).contains(&losses), "fixed-seed losses: {losses}");
    lands(&mut g, P0, "Glacial Chasm", 1, false);
    let life = g.player(P0).life;
    for _ in 0..30 {
        f(&mut g, m, P0).unwrap();
    }
    assert_eq!(g.player(P0).life, life, "Glacial Chasm prevents Crypt's damage from your own source");
}

fn sneak_options(g: &mut Game, m: PermId, post: bool) -> Vec<sim_core::hooks::Opt> {
    let f = g.registry.get(card(g, "Sneak Attack")).unwrap().options.unwrap();
    f(g, m, P0, Some(post)).unwrap()
}

fn sneak_hand(g: &mut Game, name: &str) -> PermId {
    let c = hand(g, P0, &[name])[0];
    sim_core::impls::mine::sneak_put(g, P0, c).unwrap().unwrap()
}

#[test]
fn sneak_ai_uses_red_before_combat_for_uncastable_creatures() {
    let mut g = tbl();
    let m = perm(&mut g, P0, "Sneak Attack");
    hand(&mut g, P0, &["Blightsteel Colossus", "Walking Ballista", "Hangarback Walker", "Llanowar Elves"]);
    assert!(sneak_options(&mut g, m, false).is_empty());
    lands(&mut g, P0, "Mountain", 1, false);
    lands(&mut g, P0, "Forest", 1, false);
    let opts = sneak_options(&mut g, m, false);
    assert_eq!(opts.len(), 1);
    assert!(opts[0].label.contains("Blightsteel"));
    assert!(sneak_options(&mut g, m, true).is_empty());
    g.player_mut(P0).combat_no = 1;
    assert!(sneak_options(&mut g, m, false).is_empty());
    g.player_mut(P0).combat_no = 0;
    let sim_core::hooks::Action::Ability { src, f, arg } = opts.into_iter().next().unwrap().act.unwrap() else {
        panic!()
    };
    assert!(f(&mut g, src, P0, arg).unwrap());
    assert!(!can_pay(&g, P0, 0, "R", false), "activation paid red");
    let b = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).name == "Blightsteel Colossus").unwrap();
    assert!(sim_core::engine::combat::has_haste(&g, b));
    let (lib, gy) = (g.player(P0).library.len(), g.player(P0).gy.len());
    sim_core::engine::zones::die(&mut g, m, "destroy").unwrap();
    let gy = gy + 1;
    sim_core::engine::turn::end_step(&mut g, P0).unwrap();
    assert!(!g.perm(b).on_bf);
    assert_eq!(g.player(P0).library.len(), lib + 1, "Blightsteel shuffles back");
    assert_eq!(g.player(P0).gy.len(), gy);
}

#[test]
fn sneak_fires_enter_triggers_and_next_end_step_is_every_players() {
    let mut g = tbl();
    let victim = perm(&mut g, P1, "Sol Ring");
    let golem = sneak_hand(&mut g, "Meteor Golem");
    assert!(!live(&g, victim), "Golem's enter trigger fires");
    let lands_before = g.player(P0).lands.len();
    let titan = sneak_hand(&mut g, "Primeval Titan");
    assert_eq!(g.player(P0).lands.len(), lands_before + 2);
    g.active = Some(P1);
    sim_core::engine::turn::end_step(&mut g, P1).unwrap();
    assert!(!live(&g, golem) && !live(&g, titan));
    assert_eq!(g.player(P0).lands.len(), lands_before + 2, "fetched lands remain");
    let late = sneak_hand(&mut g, "Llanowar Elves");
    assert!(live(&g, late), "put in after the beginning of the end step");
    sim_core::engine::turn::end_step(&mut g, P0).unwrap();
    assert!(!live(&g, late));
}

#[test]
fn sneak_ai_rechecks_the_hand_when_the_activation_resolves() {
    for remaining in [false, true] {
        let mut g = tbl();
        let src = perm(&mut g, P0, "Sneak Attack");
        lands(&mut g, P0, "Mountain", 1, false);
        let chosen = hand(&mut g, P0, &["Blightsteel Colossus"])[0];
        if remaining {
            hand(&mut g, P0, &["Llanowar Elves"]);
        }
        let opt = sneak_options(&mut g, src, false).into_iter().find(|o| o.label.contains("Blightsteel")).unwrap();
        g.player_mut(P0).hand.retain(|&c| c != chosen);
        let sim_core::hooks::Action::Ability { src, f, arg } = opt.act.unwrap() else { panic!() };
        assert!(f(&mut g, src, P0, arg).unwrap());
        assert!(!g.player(P0).perms.iter().any(|&m| g.perm(m).name == "Blightsteel Colossus"));
        assert_eq!(g.player(P0).perms.iter().any(|&m| g.perm(m).name == "Llanowar Elves"), remaining);
    }
}

#[test]
fn sneak_stolen_phased_blinked_and_commander_creatures() {
    use sim_core::engine::{
        control::{Duration, gain_control},
        turn, zones,
    };
    let mut g = tbl();
    let stolen = sneak_hand(&mut g, "Llanowar Elves");
    gain_control(&mut g, P1, stolen, Duration::Permanent);
    let phased = sneak_hand(&mut g, "Birds of Paradise");
    g.perm_mut(phased).phased = true;
    let old = sneak_hand(&mut g, "Noble Hierarch");
    let c = g.perm(old).cd.unwrap();
    zones::leave(&mut g, old).unwrap();
    let new = zones::enter(&mut g, P0, c, zones::Enter::default()).unwrap();
    let cmd = g.player(P0).cmd;
    g.player_mut(P0).hand.push(cmd);
    g.player_mut(P0).cmd_in_zone = false;
    let commander = sim_core::impls::mine::sneak_put(&mut g, P0, cmd).unwrap().unwrap();
    g.perm_mut(commander).is_cmd = true;
    turn::end_step(&mut g, P0).unwrap();
    assert!(live(&g, stolen) && live(&g, phased) && live(&g, new));
    assert!(!live(&g, commander) && g.player(P0).cmd_in_zone);
    gain_control(&mut g, P0, stolen, Duration::Permanent);
    g.perm_mut(phased).phased = false;
    g.perm_mut(stolen).eot_kw.clear();
    assert!(sim_core::engine::combat::has_haste(&g, stolen), "haste has no duration");
    assert!(!sim_core::engine::combat::has_haste(&g, new), "blink is a new object");
    turn::end_step(&mut g, P1).unwrap();
    assert!(live(&g, stolen) && live(&g, phased), "the delayed trigger was used up");
}

#[test]
fn even_destroys_mana_value_two_and_zero_and_spares_thanos() {
    let mut g = tbl();
    let t = thanos(&mut g, false);
    let two = perm(&mut g, P1, "Grand Abolisher"); // 2
    let zero = token(&mut g, P1, 1); // a token: 0, even
    let one = perm(&mut g, P1, "Birds of Paradise"); // 1
    let three = perm(&mut g, P1, "Opposition Agent"); // 3
    let mine = perm(&mut g, P0, "Noble Hierarch"); // 1
    t6_thanos::resolve(&mut g, P0, t, true).unwrap();
    assert!(!live(&g, two) && !live(&g, zero), "even: 2 and 0 are destroyed");
    assert!(live(&g, one) && live(&g, three) && live(&g, mine), "odd ones survive");
    assert!(live(&g, t), "Thanos (mana value 3, but 'each other creature') survives");
    assert_eq!(g.perm(t).plus, 2, "two +1/+1 counters");
}

#[test]
fn odd_destroys_mana_value_one_and_three() {
    let mut g = tbl();
    let t = thanos(&mut g, false);
    let two = perm(&mut g, P1, "Grand Abolisher");
    let zero = token(&mut g, P1, 1);
    let one = perm(&mut g, P1, "Birds of Paradise");
    let three = perm(&mut g, P1, "Opposition Agent");
    t6_thanos::resolve(&mut g, P0, t, false).unwrap();
    assert!(!live(&g, one) && !live(&g, three));
    assert!(live(&g, two) && live(&g, zero) && live(&g, t));
    assert_eq!(g.perm(t).plus, 2);
}

#[test]
fn the_parity_is_chosen_from_the_board_not_fixed() {
    // the opponents' creatures are odd, ours even: odd
    let mut g = tbl();
    let t = thanos(&mut g, false);
    perm(&mut g, P1, "Opposition Agent");
    perm(&mut g, P1, "Spellseeker"); // 3
    perm(&mut g, P0, "Orcish Bowmasters"); // 2
    assert!(!t6_thanos::best_parity(&g, P0, t).0, "odd");
    // the opponents' creatures are even, ours odd: even
    let mut g = tbl();
    let t = thanos(&mut g, false);
    perm(&mut g, P1, "Grand Abolisher");
    perm(&mut g, P1, "Collector Ouphe");
    perm(&mut g, P0, "Birds of Paradise");
    perm(&mut g, P0, "Noble Hierarch");
    assert!(t6_thanos::best_parity(&g, P0, t).0, "even");
}

#[test]
fn the_cost_is_cug_the_turn_he_entered_and_c_needs_colourless_mana() {
    let mut g = tbl();
    let t = thanos(&mut g, true);
    assert_eq!(t6_thanos::cost(&g, t), (0, "CUG"));
    lands(&mut g, P0, "Tropical Island", 2, false);
    assert!(!can_pay(&g, P0, 0, "CUG", false), "no colourless source: {{C}} isn't generic");
    lands(&mut g, P0, "Ancient Tomb", 1, false);
    assert!(can_pay(&g, P0, 0, "CUG", false));
    // the next turn: the full cost
    let mut g = tbl();
    let t = thanos(&mut g, false);
    assert_eq!(t6_thanos::cost(&g, t), (0, "CWUBRG"));
    lands(&mut g, P0, "City of Brass", 6, false);
    assert!(!can_pay(&g, P0, 0, "CWUBRG", false), "City of Brass makes only coloured mana");
}

#[test]
fn the_power_up_is_once_per_thanos_object_and_a_blink_resets_it() {
    let mut g = tbl();
    let t = thanos(&mut g, true);
    lands(&mut g, P0, "Ancient Tomb", 2, false);
    lands(&mut g, P0, "Tropical Island", 4, false);
    let ab = perm(&mut g, P1, "Grand Abolisher");
    assert!(!t6_thanos::used(&g, t));
    assert!(t6_thanos::thanos_go(&mut g, t, P0, 0).unwrap());
    assert!(t6_thanos::used(&g, t));
    assert!(!live(&g, ab));
    assert_eq!(g.perm(t).plus, 2);
    assert!(!t6_thanos::thanos_go(&mut g, t, P0, 0).unwrap(), "a second activation is refused");
    assert_eq!(g.perm(t).plus, 2);
    // blinked, he is a new object (rule 113.7): no counters, entered this turn, and the power-up is his again
    let n = t2::blink(&mut g, P0, t).unwrap().expect("Thanos returns");
    assert!(!t6_thanos::used(&g, n), "the returned Thanos has not powered up");
    assert_eq!(t6_thanos::cost(&g, n), (0, "CUG"));
    assert_eq!(g.perm(n).plus, 0);
    lands(&mut g, P0, "Ancient Tomb", 1, false);
    lands(&mut g, P0, "Tropical Island", 2, false);
    assert!(t6_thanos::thanos_go(&mut g, n, P0, 0).unwrap(), "the blinked Thanos powers up again");
    assert!(t6_thanos::used(&g, n));
    assert!(!t6_thanos::thanos_go(&mut g, n, P0, 0).unwrap(), "but only once");
}

// ======================================================== free while you control a commander
#[test]
fn fierce_guardianship_is_free_only_with_the_commander_on_the_battlefield() {
    let mut g = tbl();
    let fg = card(&g, "Fierce Guardianship");
    assert!(!commander_out(&g, P0) && !free_counter(&g, P0, fg), "Thanos in the command zone: not free");
    let t = perm(&mut g, P0, THANOS);
    assert!(!free_counter(&g, P0, fg), "a Thanos that isn't the commander doesn't count");
    g.perm_mut(t).is_cmd = true;
    assert!(commander_out(&g, P0) && free_counter(&g, P0, fg));
}

#[test]
fn deadly_rollick_is_offered_free_with_the_commander_out() {
    let mut g = tbl();
    hand(&mut g, P0, &["Deadly Rollick"]);
    perm(&mut g, P1, "Hellkite Tyrant");
    let t = perm(&mut g, P0, THANOS); // no mana at all
    let rollick = |g: &Game| {
        brain::removal_options(g, P0, &brain::Situation::new(g, P0)).iter().any(|o| o.label.contains("Deadly Rollick"))
    };
    assert!(!rollick(&g), "not the commander: Rollick costs {{3}}{{B}}");
    g.perm_mut(t).is_cmd = true;
    assert!(rollick(&g), "with the commander out it is free");
}

#[test]
fn deflecting_swat_is_free_with_the_commander_out() {
    let mut g = tbl();
    let t = thanos(&mut g, false); // no mana at all
    let theirs = perm(&mut g, P1, "Grave Titan");
    hand(&mut g, P0, &["Deflecting Swat"]);
    let stp = card(&g, "Swords to Plowshares");
    removal::apply_removal(&mut g, Some(P1), t, "exile", Some(stp)).unwrap();
    assert!(live(&g, t), "Thanos survives");
    assert!(!live(&g, theirs), "the Swords hit the caster's own creature");
    assert!(g.player(P0).gy.contains(&card(&g, "Deflecting Swat")));
}
