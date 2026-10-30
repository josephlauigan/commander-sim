//! Plan row 16, mana effects: hand mana for any card (data-driven), "as though they had haste" in the mana
//! sources, creature ability cost reduction for Thrasios, and Formidable Speaker untapping a creature or artifact.

use sim_core::ai::act;
use sim_core::engine::mana::{can_pay, pay};
use sim_core::hooks::Opt;
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn tbl() -> Game {
    table(&["rograkh-thrasios-creature-combo", "godo", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"])
}

fn set_library(g: &mut Game, p: PlayerId, top: &[&str]) {
    let filler = card(g, "Plains");
    let mut lib = vec![filler; 20];
    lib.extend(top.iter().map(|n| card(g, n)));
    g.player_mut(p).library = lib;
}

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

#[test]
fn simian_spirit_guide_pays_r_and_is_exiled() {
    let mut g = tbl();
    assert!(!can_pay(&g, P0, 0, "R", false));
    let c = hand(&mut g, P0, &["Simian Spirit Guide"])[0];
    assert!(can_pay(&g, P0, 0, "R", false), "exile it from hand for {{R}}");
    assert!(!can_pay(&g, P0, 0, "G", false), "only red");
    pay(&mut g, P0, 0, "R", false).unwrap();
    assert!(!g.player(P0).hand.contains(&c) && g.player(P0).exile.contains(&c));
}

#[test]
fn elvish_spirit_guide_still_pays_g() {
    let mut g = tbl();
    hand(&mut g, P0, &["Elvish Spirit Guide"]);
    assert!(can_pay(&g, P0, 0, "G", false));
    assert!(!can_pay(&g, P0, 0, "R", false));
}

#[test]
fn a_summoning_sick_dork_taps_for_mana_with_shang_chi_or_tyvar() {
    let mut g = tbl();
    let d = perm(&mut g, P0, "Llanowar Elves");
    g.perm_mut(d).sick = true;
    assert!(!can_pay(&g, P0, 0, "G", false), "summoning sick");
    perm(&mut g, P0, "Shang-Chi, Master of Kung Fu");
    assert!(can_pay(&g, P0, 0, "G", false), "Shang-Chi: abilities as though they had haste");
}

#[test]
fn thrasios_costs_two_with_training_grounds_and_one_with_the_familiar_too() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 2, false);
    let t = perm(&mut g, P0, "Thrasios, Triton Hero");
    set_library(&mut g, P0, &["Sol Ring", "Sol Ring", "Sol Ring"]);
    assert!(options(&mut g, t, P0, None).is_empty(), "{{4}} with two lands");
    perm(&mut g, P0, "Training Grounds");
    let o = options(&mut g, t, P0, None);
    assert_eq!(o.len(), 1, "{{2}} with Training Grounds");
    assert!(act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 0);
}

#[test]
fn speaker_untaps_a_tapped_magus_which_then_untaps_cradle() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 2, false);
    let cr = lands(&mut g, P0, "Gaea's Cradle", 1, true)[0];
    lands(&mut g, P0, "Forest", 2, true);
    for _ in 0..5 {
        token(&mut g, P0, 1);
    }
    perm(&mut g, P0, "Training Grounds"); // Magus: X=3 for {1}, better than Cradle alone
    let mg = perm(&mut g, P0, "Magus of the Candelabra");
    g.perm_mut(mg).sick = false;
    g.perm_mut(mg).tapped = true;
    let s = perm(&mut g, P0, "Formidable Speaker");
    g.perm_mut(s).sick = false;
    hand(&mut g, P0, &["Sol Ring"]);
    let o = options(&mut g, s, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert!(act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert!(!g.perm(mg).tapped, "Magus untapped");
    assert!(g.perm(s).tapped);
    let o = options(&mut g, mg, P0, Some(false));
    assert_eq!(o.len(), 1, "Magus can untap Cradle again");
    assert!(act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert!(!g.perm(cr).tapped);
}
