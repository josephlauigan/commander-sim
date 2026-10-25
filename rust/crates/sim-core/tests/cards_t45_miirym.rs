//! The Tier 5 miirym-temur-dragons cards (impls/t45_miirym.rs and the hooks marked `// --- Tiers 4/5 (miirym)`) on
//! hand-built positions. Each expectation follows the card's Oracle text.

use sim_core::engine::mana::cost_of;
use sim_core::engine::values::{epow, etgh, has_type};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::t45_miirym as m;
use sim_core::state::{Ctx, Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    let mut g = table(&[
        "miirym-temur-dragons",
        "urza-mono-blue-artifacts",
        "kinnan-simic-mana-combo",
        "yawgmoth-mono-black-aristocrats",
    ]);
    g.player_mut(P0).hand.clear();
    g
}

fn run(g: &mut Game, o: Opt, p: PlayerId) -> bool {
    match o.act.unwrap() {
        Action::Plan { f, arg } => f(g, p, arg).unwrap(),
        Action::Ability { src, f, arg } => f(g, src, p, arg).unwrap(),
        other => panic!("unexpected action {other:?}"),
    }
}

fn imp(g: &Game, name: &str) -> sim_core::hooks::CardImpl {
    *g.registry.get(card(g, name)).unwrap()
}

fn on_all(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&x| g.perm(x).on_bf && &*g.perm(x).name == name).collect()
}

fn opp_life(g: &Game) -> i32 {
    g.opps(P0).map(|q| g.player(q).life).sum()
}

// ------------------------------------------------------------------ Astral Dragon
#[test]
fn astral_dragon_makes_two_dragon_copies_of_a_noncreature_permanent() {
    let mut g = tbl();
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P0, "Astral Dragon");
    let rings = on_all(&g, P0, "Sol Ring");
    assert_eq!(rings.len(), 3, "the Sol Ring and two token copies");
    let toks: Vec<PermId> = rings.into_iter().filter(|&x| g.perm(x).token).collect();
    assert_eq!(toks.len(), 2);
    for t in toks {
        assert!(g.is_creature(t), "a creature in addition to its other types");
        assert!(has_type(&g, t, "dragon"));
        assert_eq!((epow(&g, t), etgh(&g, t)), (3, 3));
        assert!(g.perm(t).fly);
    }
}

// ------------------------------------------------------------------ Dragonborn Champion
#[test]
fn dragonborn_champion_draws_on_five_or_more_damage_to_a_player() {
    let mut g = tbl();
    perm(&mut g, P0, "Dragonborn Champion");
    let n = g.player(P0).hand.len();
    sim_core::engine::life::lose_life(&mut g, P1, 4, Some(P0), "combat", None).unwrap();
    assert_eq!(g.player(P0).hand.len(), n, "4 damage: no card");
    sim_core::engine::life::lose_life(&mut g, P1, 5, Some(P0), "triggers", None).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1, "5 damage: a card");
    sim_core::engine::life::lose_life(&mut g, P1, 7, Some(P0), "other", Some(false)).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1, "life loss isn't damage");
    sim_core::engine::life::lose_life(&mut g, P0, 9, Some(P1), "combat", None).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1, "an opponent's source");
}

// ------------------------------------------------------------------ Earthquake Dragon
#[test]
fn earthquake_dragon_costs_less_by_your_dragons_mana_value_and_comes_back() {
    let mut g = tbl();
    let eq = card(&g, "Earthquake Dragon");
    assert_eq!(cost_of(&g, P0, eq).0, 14);
    perm(&mut g, P0, "Goldspan Dragon"); // mana value 5
    perm(&mut g, P0, "Birds of Paradise"); // not a Dragon
    assert_eq!(cost_of(&g, P0, eq).0, 9);
    // {2}{G}, sacrifice a land: from the graveyard to hand
    take(&mut g, P0, "Earthquake Dragon");
    g.player_mut(P0).gy.push(eq);
    lands(&mut g, P0, "Forest", 10, false);
    let f = imp(&g, "Earthquake Dragon").gy_options.unwrap();
    g.player_mut(P0).lands.truncate(9);
    assert!(f(&mut g, eq, P0, None).unwrap().is_empty(), "too dear to cast next turn: {{9}}{{G}} on eight lands");
    lands(&mut g, P0, "Forest", 1, false);
    assert!(f(&mut g, eq, P0, Some(false)).unwrap().is_empty(), "at the end-of-turn window");
    let o = f(&mut g, eq, P0, None).unwrap().pop().expect("back to hand");
    assert!(run(&mut g, o, P0));
    assert!(g.player(P0).hand.contains(&eq) && !g.player(P0).gy.contains(&eq));
    assert_eq!(g.player(P0).lands.len(), 9, "a land sacrificed");
}

// ------------------------------------------------------------------ Korlessa
#[test]
fn korlessa_casts_dragon_spells_from_the_top_of_the_library() {
    let mut g = tbl();
    g.active = Some(P0);
    g.step = Step::Main1;
    let k = perm(&mut g, P0, "Korlessa, Scale Singer");
    lands(&mut g, P0, "Mountain", 6, false);
    let gold = take(&mut g, P0, "Goldspan Dragon");
    let sol = take(&mut g, P0, "Sol Ring");
    let f = imp(&g, "Korlessa, Scale Singer").options.unwrap();
    g.player_mut(P0).library.push(sol);
    assert!(f(&mut g, k, P0, Some(false)).unwrap().is_empty(), "only Dragon spells");
    g.player_mut(P0).library.pop();
    g.player_mut(P0).library.push(gold);
    let o = f(&mut g, k, P0, Some(false)).unwrap().pop().expect("the Dragon on top");
    assert!(run(&mut g, o, P0));
    assert_eq!(on_all(&g, P0, "Goldspan Dragon").len(), 1);
    assert!(!g.player(P0).library.contains(&gold));
}

// ------------------------------------------------------------------ Thrakkus the Butcher
#[test]
fn thrakkus_doubles_the_power_of_each_dragon_when_it_attacks() {
    let mut g = tbl();
    g.step = Step::Combat;
    let t = perm(&mut g, P0, "Thrakkus the Butcher");
    let gold = perm(&mut g, P0, "Goldspan Dragon");
    let bird = perm(&mut g, P0, "Birds of Paradise");
    let f = imp(&g, "Thrakkus the Butcher").attack.unwrap();
    f(&mut g, t, P0, &[gold], P1).unwrap();
    assert_eq!(epow(&g, gold), 4, "Thrakkus itself must attack");
    f(&mut g, t, P0, &[t, gold], P1).unwrap();
    assert_eq!((epow(&g, t), epow(&g, gold), epow(&g, bird)), (6, 8, 0));
}

// ------------------------------------------------------------------ Wrathful Red Dragon
#[test]
fn wrathful_red_dragon_redirects_damage_dealt_to_your_dragons() {
    let mut g = tbl();
    perm(&mut g, P0, "Wrathful Red Dragon");
    let gold = perm(&mut g, P0, "Goldspan Dragon");
    let bird = perm(&mut g, P0, "Birds of Paradise");
    let before = opp_life(&g);
    sim_core::impls::t0_giada::damaged(&mut g, gold, 3).unwrap();
    assert_eq!(opp_life(&g), before - 3, "3 damage dealt to the Dragon: 3 to an opponent");
    sim_core::impls::t0_giada::damaged(&mut g, bird, 1).unwrap();
    assert_eq!(opp_life(&g), before - 3, "the Birds aren't a Dragon");
}

#[test]
fn wrathful_red_dragon_and_blasphemous_act_deal_13_per_dragon() {
    let mut g = tbl();
    perm(&mut g, P0, "Wrathful Red Dragon");
    perm(&mut g, P0, "Goldspan Dragon");
    perm(&mut g, P0, "Thrakkus the Butcher");
    perm(&mut g, P0, "Birds of Paradise");
    let before = opp_life(&g);
    sim_core::engine::removal::apply_wipe(&mut g, P1, "dmg13", None, &Ctx::default()).unwrap();
    assert!(on_all(&g, P0, "Wrathful Red Dragon").is_empty(), "the wipe kills it too");
    assert_eq!(before - opp_life(&g), 3 * 13, "three Dragons dealt damage at once: 13 each");
}

#[test]
fn the_wrathful_blasphemous_act_combo_is_ready_when_it_kills_the_table() {
    let mut g = tbl();
    perm(&mut g, P0, "Wrathful Red Dragon");
    perm(&mut g, P0, "Goldspan Dragon");
    perm(&mut g, P0, "Thrakkus the Butcher");
    lands(&mut g, P0, "Mountain", 9, false);
    hand(&mut g, P0, &["Blasphemous Act"]);
    let cmb = sim_core::impls::combos::all()
        .iter()
        .find(|c| c.name == "Wrathful Red Dragon + Blasphemous Act")
        .expect("in the catalogue");
    for q in [PlayerId(1), PlayerId(2), PlayerId(3)] {
        g.player_mut(q).life = 13;
    }
    assert!((cmb.ready)(&g, P0).0, "three Dragons, three chunks of 13: everyone at 13 dies");
    g.player_mut(PlayerId(3)).life = 14;
    assert!(!(cmb.ready)(&g, P0).0, "a fourth chunk is needed");
    assert_eq!(m::chunks_kill(&mut [40, 40], 6, 13), 1);
}
