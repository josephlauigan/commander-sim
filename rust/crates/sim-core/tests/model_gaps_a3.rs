//! model-gaps-a3: token lands leave no card behind, Astral Dragon's land and Aura copies, ward on every targeting,
//! Mystic Forge's top-of-library counterspell, Peter Parker's Camera and Marvin with hand-written abilities.

mod play_common;
use play_common::*;
use sim_core::engine::turn::remove_land;
use sim_core::engine::zones::Zone;
use sim_core::impls::{fixes, lands as land_fx, rules};

fn tbl() -> Game {
    table(&["karn", "kefka", "sauron"])
}

/// a token copy of a land card (a Mirrorworks or Astral Dragon copy), listed in the controller's lands
fn token_land(g: &mut Game, p: PlayerId, name: &str) -> PermId {
    let c = card(g, name);
    let t = g.add_land(p, c, false);
    g.perm_mut(t).token = true;
    t
}

/// how many copies of card `name` sit in p's graveyard, hand, library and exile together
fn copies_elsewhere(g: &Game, p: PlayerId, name: &str) -> usize {
    let c = card(g, name);
    let pl = g.player(p);
    let n = |z: &Vec<sim_core::ids::CardId>| z.iter().filter(|&&x| x == c).count();
    n(&pl.gy) + n(&pl.hand) + n(&pl.library) + pl.exile.iter().filter(|&&x| x == c).count()
}

// ======================================================== item 1: token lands
#[test]
fn a_token_land_destroyed_or_sacrificed_leaves_no_card() {
    let mut g = tbl();
    let l = lands(&mut g, P0, "Treasure Vault", 1, false)[0]; // (a real one, taken from the library)
    let before = copies_elsewhere(&g, P0, "Treasure Vault");
    let t = token_land(&mut g, P0, "Treasure Vault");
    fixes::destroy_land(&mut g, P0, t).unwrap();
    assert!(!g.player(P0).lands.contains(&t), "destroyed");
    assert_eq!(copies_elsewhere(&g, P0, "Treasure Vault"), before, "a token ceases to exist: no card in the graveyard");
    let t = token_land(&mut g, P0, "Treasure Vault");
    land_fx::sac_land(&mut g, P0, t).unwrap();
    assert_eq!(copies_elsewhere(&g, P0, "Treasure Vault"), before);
    // a real Vault does go to the graveyard
    fixes::destroy_land(&mut g, P0, l).unwrap();
    assert_eq!(copies_elsewhere(&g, P0, "Treasure Vault"), before + 1);
}

#[test]
fn a_token_land_bounced_exiled_or_tucked_leaves_no_card() {
    let mut g = tbl();
    let l = lands(&mut g, P0, "Darksteel Citadel", 1, false)[0];
    let before = copies_elsewhere(&g, P0, "Darksteel Citadel");
    for z in [Zone::Hand, Zone::Exile, Zone::Gy, Zone::Lib, Zone::Top] {
        let t = token_land(&mut g, P0, "Darksteel Citadel");
        assert!(!remove_land(&mut g, P0, t, Some(z)), "no card went anywhere");
        assert!(!g.player(P0).lands.contains(&t) && !g.perm(t).on_bf);
    }
    assert_eq!(copies_elsewhere(&g, P0, "Darksteel Citadel"), before);
    assert!(remove_land(&mut g, P0, l, Some(Zone::Hand)));
    assert_eq!(copies_elsewhere(&g, P0, "Darksteel Citadel"), before + 1, "a real land card goes to the hand");
    assert!(g.player(P0).hand.contains(&card(&g, "Darksteel Citadel")));
}

#[test]
fn mass_sacrifice_of_lands_leaves_no_card_for_a_token() {
    let mut g = tbl();
    lands(&mut g, P1, "Wastes", 2, false);
    token_land(&mut g, P1, "Treasure Vault");
    token_land(&mut g, P1, "Darksteel Citadel");
    let before = (copies_elsewhere(&g, P1, "Treasure Vault"), copies_elsewhere(&g, P1, "Darksteel Citadel"));
    let src = perm(&mut g, P0, "Liliana, Dreadhorde General");
    (rules::LDG_ULT.eff)(&mut g, P0, src).unwrap();
    assert_eq!(g.player(P1).lands.len(), 1, "each opponent keeps one land");
    assert_eq!((copies_elsewhere(&g, P1, "Treasure Vault"), copies_elsewhere(&g, P1, "Darksteel Citadel")), before);
}
