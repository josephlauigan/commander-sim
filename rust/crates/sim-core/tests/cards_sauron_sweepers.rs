//! The one-sided board-control cards for Sauron's sweeper tests (impls/sweepers.rs): Crippling Fear, Widespread
//! Brutality, Chandra's Ignition, Inferno Titan, Ral and the Implicit Maze and Massacre Wurm. Seat 0 plays the sauron
//! deck, seat 1 the veyran deck. Each expectation comes from the card's Oracle text.

use sim_core::ai::{brain, decks};
use sim_core::engine::{cast, values, zones};
use sim_core::hooks::Opt;
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["sauron", "veyran"])
}

/// a creature token with these types, not summoning sick
fn body(g: &mut Game, p: PlayerId, pow: i32, tgh: i32, types: &[&'static str]) -> PermId {
    let spec = zones::Tokens { sick: false, tgh: Some(tgh), types: types.to_vec(), ..zones::Tokens::new(1, pow) };
    zones::make_tokens(g, p, spec).unwrap()[0]
}

fn army(g: &mut Game, p: PlayerId, counters: i32) -> PermId {
    zones::amass(g, p, counters).unwrap();
    values::army_of(g, p).expect("an Army")
}

fn cast_spell(g: &mut Game, p: PlayerId, name: &str) {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
}

fn options(g: &mut Game, p: PlayerId) -> Vec<Opt> {
    brain::main_options(g, p, false).unwrap()
}

fn alive(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

// ======================================================== Crippling Fear
#[test]
fn crippling_fear_spares_the_orc_army_and_kills_the_rest_with_toughness_three_or_less() {
    let mut g = game();
    let a = army(&mut g, P0, 2); // a 2/2 Orc Army: it would die to -3/-3 unless Orc is named
    let mine = body(&mut g, P0, 2, 2, &["wraith"]);
    let small = body(&mut g, P1, 2, 2, &["human"]);
    let three = body(&mut g, P1, 1, 3, &["elf"]);
    let big = body(&mut g, P1, 4, 4, &["giant"]);
    assert!(!decks::WipeRule::new(&g, P0, "fear").hits(&g, a, P0), "Orc is named");
    cast_spell(&mut g, P0, "Crippling Fear");
    assert!(alive(&g, a), "the Orc Army is of the chosen type");
    assert!(!alive(&g, mine), "-3/-3 kills the caster's own Wraith");
    assert!(!alive(&g, small));
    assert!(!alive(&g, three), "toughness 3 dies to -3/-3");
    assert!(alive(&g, big), "toughness 4 survives -3/-3");
}

#[test]
fn crippling_fear_names_the_type_that_saves_its_own_creatures() {
    let mut g = game();
    let marshal = perm(&mut g, P0, "Mogg War Marshal"); // a Goblin Warrior, with its Goblin token
    let h1 = body(&mut g, P1, 2, 2, &["human"]);
    let h2 = body(&mut g, P1, 2, 2, &["human"]);
    let ty = sim_core::impls::sweepers::fear_type(&g, P0);
    assert!(ty == "goblin" || ty == "warrior", "named {ty}");
    cast_spell(&mut g, P0, "Crippling Fear");
    assert!(alive(&g, marshal));
    assert!(!alive(&g, h1) && !alive(&g, h2));
}

#[test]
fn crippling_fear_is_offered_only_when_it_kills_more_of_theirs() {
    let mut g = game();
    army(&mut g, P0, 3);
    lands(&mut g, P0, "Swamp", 4, false);
    hand(&mut g, P0, &["Crippling Fear"]);
    assert!(!options(&mut g, P0).iter().any(|o| o.label == "Crippling Fear"), "nothing of theirs to kill");
    for _ in 0..3 {
        body(&mut g, P1, 5, 3, &["human"]);
    }
    let opts = options(&mut g, P0);
    let o = opts.iter().find(|o| o.label == "Crippling Fear").expect("offered against three 5/3s");
    assert!(o.utility > 0.0, "utility {}", o.utility);
}

// ======================================================== Widespread Brutality
#[test]
fn widespread_brutality_amasses_two_then_hits_each_non_army_creature() {
    let mut g = game();
    let a = army(&mut g, P0, 3); // 3/3, 5/5 after the amass
    let mine = body(&mut g, P0, 2, 2, &["wraith"]);
    let opp_army = army(&mut g, P1, 1);
    let small = body(&mut g, P1, 2, 2, &["human"]);
    let five = body(&mut g, P1, 1, 5, &["giant"]);
    let six = body(&mut g, P1, 1, 6, &["giant"]);
    cast_spell(&mut g, P0, "Widespread Brutality");
    assert_eq!(values::epow(&g, a), 5, "amass 2");
    assert!(alive(&g, a), "the Army is not hit");
    assert!(alive(&g, opp_army), "no Army is hit, an opponent's included");
    assert!(!alive(&g, mine), "the caster's own other creatures are hit");
    assert!(!alive(&g, small));
    assert!(!alive(&g, five), "5 damage kills toughness 5");
    assert!(alive(&g, six));
}

#[test]
fn widespread_brutality_makes_an_army_when_there_is_none() {
    let mut g = game();
    let small = body(&mut g, P1, 2, 2, &["human"]);
    cast_spell(&mut g, P0, "Widespread Brutality");
    let a = values::army_of(&g, P0).expect("a new Army");
    assert_eq!(values::epow(&g, a), 2);
    assert!(!alive(&g, small), "the 2/2 Army deals 2 damage");
}

#[test]
fn widespread_brutality_is_held_when_it_kills_only_its_own() {
    let mut g = game();
    army(&mut g, P0, 3);
    for _ in 0..3 {
        body(&mut g, P0, 2, 2, &["wraith"]);
    }
    body(&mut g, P1, 1, 9, &["giant"]);
    lands(&mut g, P0, "Swamp", 4, false);
    hand(&mut g, P0, &["Widespread Brutality"]);
    let (ol, ml, _) = decks::wipe_eval(&g, P0, "brutal");
    assert!(ol <= 0.0 && ml > 0.0, "ol {ol} ml {ml}");
    assert!(!options(&mut g, P0).iter().any(|o| o.label == "Widespread Brutality"));
}

// ======================================================== Chandra's Ignition
#[test]
fn chandras_ignition_uses_the_strongest_creature_and_hits_each_opponent() {
    let mut g = game();
    let a = army(&mut g, P0, 5); // the strongest creature: 5 damage
    let mine = body(&mut g, P0, 1, 1, &["wraith"]);
    let three = body(&mut g, P1, 3, 3, &["human"]);
    let six = body(&mut g, P1, 1, 6, &["giant"]);
    let life = g.player(P1).life;
    cast_spell(&mut g, P0, "Chandra's Ignition");
    assert!(alive(&g, a), "the source is not an \"other\" creature");
    assert!(!alive(&g, mine), "each other creature, the caster's own included");
    assert!(!alive(&g, three));
    assert!(alive(&g, six));
    assert_eq!(g.player(P1).life, life - 5, "and each opponent");
}

#[test]
fn chandras_ignition_is_cast_for_a_lethal_hit_on_a_player_with_no_board() {
    let mut g = game();
    army(&mut g, P0, 5);
    lands(&mut g, P0, "Mountain", 5, false);
    hand(&mut g, P0, &["Chandra's Ignition"]);
    assert!(!options(&mut g, P0).iter().any(|o| o.label == "Chandra's Ignition"), "5 damage to 40 life, no board");
    g.player_mut(P1).life = 5;
    let opts = options(&mut g, P0);
    let o = opts.iter().find(|o| o.label == "Chandra's Ignition").expect("lethal damage");
    assert!(o.utility > 0.0);
    cast_spell(&mut g, P0, "Chandra's Ignition");
    assert!(!g.player(P1).alive, "5 damage kills the opponent at 5");
}

#[test]
fn chandras_ignition_is_held_when_it_costs_more_than_it_kills() {
    let mut g = game();
    army(&mut g, P0, 5);
    for _ in 0..3 {
        body(&mut g, P0, 2, 2, &["wraith"]);
    }
    body(&mut g, P1, 1, 1, &["human"]);
    lands(&mut g, P0, "Mountain", 5, false);
    hand(&mut g, P0, &["Chandra's Ignition"]);
    let (ol, ml, _) = decks::wipe_eval(&g, P0, "ignite");
    assert!(ml > ol, "ol {ol} ml {ml}");
    let opts = options(&mut g, P0);
    assert!(opts.iter().find(|o| o.label == "Chandra's Ignition").is_none_or(|o| o.utility < 0.0));
}

// ======================================================== Inferno Titan
#[test]
fn inferno_titan_kills_the_best_small_creatures_when_it_enters() {
    let mut g = game();
    let a = body(&mut g, P1, 3, 1, &["human"]);
    let b = body(&mut g, P1, 3, 2, &["human"]);
    let c = body(&mut g, P1, 1, 4, &["giant"]);
    let life = g.player(P1).life;
    perm(&mut g, P0, "Inferno Titan");
    assert!(!alive(&g, a) && !alive(&g, b), "1 + 2 damage kills both");
    assert!(alive(&g, c));
    assert_eq!(g.player(P1).life, life);
}

#[test]
fn inferno_titan_goes_at_the_face_when_lethal_and_when_nothing_is_worth_killing() {
    let mut g = game();
    g.player_mut(P1).life = 3;
    body(&mut g, P1, 1, 1, &["human"]);
    perm(&mut g, P0, "Inferno Titan");
    assert!(!g.player(P1).alive, "3 damage to a player at 3");
    let mut g = game();
    let big = body(&mut g, P1, 1, 6, &["giant"]);
    let life = g.player(P1).life;
    perm(&mut g, P0, "Inferno Titan");
    assert!(alive(&g, big));
    assert_eq!(g.player(P1).life, life - 3, "nothing it can kill: all 3 to the weakest opponent");
}

#[test]
fn inferno_titan_deals_its_damage_again_when_it_attacks() {
    let mut g = game();
    let t = perm(&mut g, P0, "Inferno Titan");
    let x = body(&mut g, P1, 3, 2, &["human"]);
    let f = g.registry.get(g.perm(t).cd.unwrap()).unwrap().attack.unwrap();
    f(&mut g, t, P0, &[t], P1).unwrap();
    assert!(!alive(&g, x));
}

#[test]
fn inferno_titan_hits_a_planeswalker() {
    let mut g = game();
    let w = perm(&mut g, P1, "Chandra, Torch of Defiance");
    let loyalty = g.perm(w).loyalty.unwrap();
    perm(&mut g, P0, "Inferno Titan");
    let after = if alive(&g, w) { g.perm(w).loyalty.unwrap() } else { 0 };
    assert!(after < loyalty, "loyalty {loyalty} -> {after}");
}

// ======================================================== Ral and the Implicit Maze
#[test]
fn ral_chapter_one_spares_the_casters_side_and_hits_each_opposing_creature_and_planeswalker() {
    let mut g = game();
    let a = army(&mut g, P0, 2);
    let mine = body(&mut g, P0, 1, 1, &["wraith"]);
    let small = body(&mut g, P1, 2, 2, &["human"]);
    let big = body(&mut g, P1, 1, 5, &["giant"]);
    let w = perm(&mut g, P1, "Chandra, Torch of Defiance");
    let loyalty = g.perm(w).loyalty.unwrap();
    perm(&mut g, P0, "Ral and the Implicit Maze");
    assert!(alive(&g, a) && alive(&g, mine), "nothing of the caster's is hit");
    assert!(!alive(&g, small));
    assert!(alive(&g, big));
    assert_eq!(g.perm(w).loyalty, Some(loyalty - 2));
}

#[test]
fn ral_runs_all_three_chapters() {
    let mut g = game();
    hand(&mut g, P0, &["Mountain"]);
    let saga = perm(&mut g, P0, "Ral and the Implicit Maze");
    let before = g.player(P0).hand.len();
    main1(&mut g, saga, P0);
    assert_eq!(g.player(P0).hand.len(), before + 1, "discard one, exile two playable");
    assert_eq!(g.player(P0).impulse_long.len(), 2);
    main1(&mut g, saga, P0);
    let weird =
        g.player(P0).perms.iter().copied().find(|&m| g.perm(m).data.truthy(DataKey::WeirdToken)).expect("Weird");
    assert!(!alive(&g, saga), "sacrificed after III");
    on_cast(&mut g, P0, "Lightning Bolt");
    assert_eq!(g.perm(weird).plus, 1, "a +1/+1 counter per noncreature spell");
}

fn main1(g: &mut Game, src: PermId, p: PlayerId) {
    let f = g.registry.get(g.perm(src).cd.unwrap()).unwrap().main1.unwrap();
    f(g, src, p).unwrap();
}

fn on_cast(g: &mut Game, p: PlayerId, name: &str) {
    let c = card(g, name);
    cast::on_cast(g, p, c).unwrap();
}

// ======================================================== Massacre Wurm
#[test]
fn massacre_wurm_only_shrinks_the_opponents_creatures() {
    let mut g = game();
    let a = army(&mut g, P0, 1); // a 1/1: it would die to -2/-2 if the Wurm hit its own side
    let mine = body(&mut g, P0, 2, 2, &["wraith"]);
    let small = body(&mut g, P1, 2, 2, &["human"]);
    let three = body(&mut g, P1, 3, 3, &["human"]);
    let life = g.player(P1).life;
    perm(&mut g, P0, "Massacre Wurm");
    assert!(alive(&g, a) && alive(&g, mine), "creatures your opponents control only");
    assert!(!alive(&g, small));
    assert!(alive(&g, three));
    assert_eq!(g.player(P1).life, life - 2, "an opposing creature died: that player loses 2 life");
}
