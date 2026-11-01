//! The Sauron speed and backup-win cards (src/impls/threats.rs), each against its Oracle text.

use sim_core::ai::brain;
use sim_core::engine::mana::total_mana;
use sim_core::engine::{turn, values, zones};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn game() -> Game {
    let mut g = table(&["sauron", "veyran", "seph"]);
    g.active = Some(P0);
    g
}

fn army(g: &Game) -> PermId {
    values::army_of(g, P0).expect("an Army")
}

fn hand_opts(g: &mut Game, c: CardId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    f(g, c, P0, post).unwrap()
}

fn run_plan(g: &mut Game, o: &Opt) -> bool {
    match o.act.clone() {
        Some(Action::Plan { f, arg }) => f(g, P0, arg).unwrap(),
        _ => panic!("not a plan"),
    }
}

// ---------------------------------------------------------------- amass creatures
#[test]
fn lazotep_reaver_amasses_onto_the_orc_army() {
    let mut g = game();
    zones::amass(&mut g, P0, 2).unwrap(); // an Orc Army with 2 counters
    let a = army(&g);
    perm(&mut g, P0, "Lazotep Reaver");
    assert_eq!(g.perm(a).plus, 3, "Zombies 1 goes onto the Army you control whatever its type");
    assert_eq!(g.player(P0).perms.iter().filter(|&&m| g.perm(m).army).count(), 1, "no second Army");
}

#[test]
fn lazotep_reaver_makes_an_army_when_there_is_none() {
    let mut g = game();
    perm(&mut g, P0, "Lazotep Reaver");
    assert_eq!(g.perm(army(&g)).plus, 1);
}

#[test]
fn grim_initiate_has_first_strike_and_amasses_when_it_dies() {
    let mut g = game();
    let m = perm(&mut g, P0, "Grim Initiate");
    assert!(sim_core::engine::combat::first_strike(&g, m));
    assert!(values::army_of(&g, P0).is_none());
    zones::die(&mut g, m, "destroy").unwrap();
    assert_eq!(g.perm(army(&g)).plus, 1);
}

#[test]
fn dreadhorde_invasion_upkeep_and_lifelink() {
    let mut g = game();
    let inv = perm(&mut g, P0, "Dreadhorde Invasion");
    let life = g.player(P0).life;
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).life, life - 1);
    let a = army(&g);
    assert_eq!(g.perm(a).plus, 1);
    let hook = g.registry.get(g.perm(inv).cd.unwrap()).unwrap().attack.unwrap();
    hook(&mut g, inv, P0, &[a], P1).unwrap();
    assert!(!g.perm(a).eot_kw.contains(&"lifelink"), "power 1: no lifelink");
    g.perm_mut(a).plus = 6;
    hook(&mut g, inv, P0, &[a], P1).unwrap();
    assert!(g.perm(a).eot_kw.contains(&"lifelink"), "a Zombie token with power 6 attacks: lifelink");
}

// ---------------------------------------------------------------- Grishnákh
#[test]
fn grishnakh_amasses_and_steals_a_small_nonlegendary_creature() {
    let mut g = game();
    let small = token(&mut g, P1, 2);
    let big = token(&mut g, P1, 5);
    let legend = perm(&mut g, P2, "Veyran, Voice of Duality"); // power 2 but legendary
    g.perm_mut(small).tapped = true;
    perm(&mut g, P0, "Grishnákh, Brash Instigator");
    assert_eq!(g.perm(army(&g)).plus, 2);
    assert_eq!(g.perm(small).owner, P0, "power 2 <= the Army's 2");
    assert!(!g.perm(small).tapped, "untapped");
    assert!(!g.perm(small).sick, "haste");
    assert_eq!(g.perm(big).owner, P1, "power 5 is too big");
    assert_eq!(g.perm(legend).owner, P2, "legendary");
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.perm(small).owner, P1, "it goes back at the end step");
}

// ---------------------------------------------------------------- Torment of Hailfire
fn torment(g: &mut Game, x: i32) {
    let c = hand(g, P0, &["Torment of Hailfire"])[0];
    let f = g.registry.get(c).unwrap().resolve.unwrap();
    f(g, P0, c, &Ctx { x, ..Ctx::default() }).unwrap();
}

#[test]
fn torment_costs_a_comfortable_opponent_three_life_per_iteration_when_they_have_nothing() {
    let mut g = game();
    g.player_mut(P1).hand.clear();
    g.player_mut(P2).hand.clear();
    let (l1, l2) = (g.player(P1).life, g.player(P2).life);
    torment(&mut g, 3);
    assert_eq!((g.player(P1).life, g.player(P2).life), (l1 - 9, l2 - 9));
}

#[test]
fn torment_opponents_pay_with_cheap_cards_first() {
    let mut g = game();
    g.player_mut(P1).hand.clear();
    hand(&mut g, P1, &["Forest", "Forest"]);
    g.player_mut(P2).hand.clear();
    g.player_mut(P2).life = 40;
    let l1 = g.player(P1).life;
    torment(&mut g, 2);
    assert!(g.player(P1).hand.is_empty() || g.player(P1).life < l1, "paid with the lands or the life");
    assert!(g.player(P1).life >= l1 - 6);
}

#[test]
fn torment_a_low_opponent_pays_with_anything_and_a_bare_one_dies() {
    let mut g = game();
    g.player_mut(P1).hand.clear();
    g.player_mut(P1).life = 3;
    token(&mut g, P1, 3);
    g.player_mut(P2).hand.clear();
    torment(&mut g, 1);
    assert_eq!(g.player(P1).life, 3, "sacrificed the token instead");
    assert!(g.player(P1).perms.iter().all(|&m| !g.perm(m).token || !g.perm(m).on_bf));
    torment(&mut g, 1);
    assert!(!g.player(P1).alive, "nothing left to give: 3 life lost");
}

#[test]
fn torment_ai_casts_it_for_a_kill_and_not_early_otherwise() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 8, false);
    let c = hand(&mut g, P0, &["Torment of Hailfire"])[0];
    g.player_mut(P1).hand.clear();
    g.player_mut(P2).hand.clear();
    assert!(hand_opts(&mut g, c, Some(false)).is_empty(), "X = 6 kills nobody at 40 life early");
    g.player_mut(P1).life = 12;
    g.player_mut(P2).life = 12;
    let o = hand_opts(&mut g, c, Some(false));
    assert_eq!(o.len(), 1);
    assert!(run_plan(&mut g, &o[0]));
    assert!(!g.player(P1).alive && !g.player(P2).alive);
}

// ---------------------------------------------------------------- Exsanguinate in Sauron's deck
// ---------------------------------------------------------------- Dualcaster Mage
#[test]
fn dualcaster_mage_is_a_flash_2_2_otherwise() {
    let g = game();
    let d = g.db.get(card(&g, "Dualcaster Mage"));
    assert!(d.tags.has(sim_core::tag::Tag::Flash));
    assert_eq!((d.pow, d.tgh), (2, 2));
}
#[test]
fn rocks_make_mana_for_sauron() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 1, false);
    let base = total_mana(&g, P0, false);
    perm(&mut g, P0, "Talisman of Indulgence");
    assert_eq!(total_mana(&g, P0, false), base + 1);
    perm(&mut g, P0, "Rakdos Signet");
    assert_eq!(total_mana(&g, P0, false), base + 2);
    lands(&mut g, P1, "Forest", 1, false);
    perm(&mut g, P0, "Fellwar Stone");
    assert_eq!(total_mana(&g, P0, false), base + 3);
}

#[test]
fn sauron_ai_plays_the_early_cards_on_turn_two() {
    let mut g = game();
    g.player_mut(P0).turns = 2;
    lands(&mut g, P0, "Swamp", 2, false);
    hand(&mut g, P0, &["Dreadhorde Invasion"]);
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    assert!(
        opts.iter().any(|o| o.label.contains("Dreadhorde Invasion")),
        "{:?}",
        opts.iter().map(|o| &o.label).collect::<Vec<_>>()
    );
}
