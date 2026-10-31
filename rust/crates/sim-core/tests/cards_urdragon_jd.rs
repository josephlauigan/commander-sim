//! JD's `urdragon` deck: the cards modeled for it (Kindred Dominance in impls/sweepers.rs, Kaldra Compleat's living
//! weapon in impls/karn_engines.rs and its compiled bonus), and the seat's plan file.

use sim_core::engine::removal::apply_wipe;
use sim_core::engine::values::{epow, etgh, indestructible};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    table(&["urdragon", "miirym-temur-dragons", "urza-mono-blue-artifacts", "zur-esper-enchantment-control"])
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

fn dominance(g: &mut Game) {
    let c = card(g, "Kindred Dominance");
    apply_wipe(g, P0, "dominance", Some(c), &Ctx::default()).unwrap();
}

#[test]
fn kindred_dominance_names_the_type_that_keeps_the_casters_creatures() {
    let mut g = tbl();
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord"); // Human Barbarian
    let titan = perm(&mut g, P1, "Grave Titan"); // Zombie Giant
    let tok = token(&mut g, P1, 1);
    dominance(&mut g);
    assert!(live(&g, godo), "Godo's type is named");
    assert!(!live(&g, titan) && !live(&g, tok), "every creature not of that type is destroyed");
}

#[test]
fn kindred_dominance_with_maskwood_nexus_spares_every_creature_of_ours() {
    let mut g = tbl();
    perm(&mut g, P0, "Maskwood Nexus");
    let godo = perm(&mut g, P0, "Godo, Bandit Warlord");
    let celebrant = perm(&mut g, P0, "Combat Celebrant");
    let titan = perm(&mut g, P1, "Grave Titan");
    dominance(&mut g);
    assert!(live(&g, godo) && live(&g, celebrant), "Maskwood Nexus: ours are every type");
    assert!(!live(&g, titan));
}

#[test]
fn kaldra_compleat_comes_with_a_germ_it_makes_a_five_five() {
    let mut g = tbl();
    let k = perm(&mut g, P0, "Kaldra Compleat");
    assert!(indestructible(&g, k), "Kaldra itself is indestructible");
    let germ = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token && g.perm(m).on_bf).expect("a Germ");
    assert_eq!(g.perm(k).attached, Some(germ), "living weapon: attached to the Germ");
    assert_eq!((epow(&g, germ), etgh(&g, germ)), (5, 5));
    for kw in ["first strike", "trample", "indestructible", "haste"] {
        assert!(sim_core::dsl::has_kw(&g, germ, kw), "{kw}");
    }
}

#[test]
fn the_urdragon_seat_plays_from_its_plan_file() {
    assert!(sim_core::ai::plan_data::get("urdragon").is_some());
    assert!(sim_core::ai::plans::plays_plan("urdragon"));
    assert!(!sim_core::ai::is_main("urdragon"), "a plan-file deck, not a hand-written main deck");
    let s = sim_core::ai::base_style("urdragon");
    assert_eq!((s.aggression, s.caution), (0.65, 0.45));
    assert_eq!(sim_core::ai::base_style("thanos"), sim_core::ai::base_style("scion"), "Scion's style, from data");
}
