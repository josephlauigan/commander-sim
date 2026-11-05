//! model-gaps-a3: token lands leave no card behind, Astral Dragon's land and Aura copies, ward on every targeting,
//! Mystic Forge's top-of-library counterspell, Peter Parker's Camera and Marvin with hand-written abilities.

mod play_common;
use play_common::*;
use sim_core::engine::turn::remove_land;
use sim_core::engine::zones::Zone;
use sim_core::impls::{fixes, lands as land_fx, rules};
use sim_core::control::Answer;

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

// ======================================================== item 2: Astral Dragon copies a land or an Aura
/// a permanent put onto the battlefield without its enters trigger (the test calls it)
fn quiet_perm(g: &mut Game, p: PlayerId, name: &str) -> PermId {
    g.skip_etb = true;
    let m = perm(g, p, name);
    g.skip_etb = false;
    m
}

fn astral_etb(g: &mut Game, p: PlayerId, src: PermId) {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().etb.unwrap();
    f(g, src, p, src).unwrap();
}

/// a person's answers: pick the choice with this word, otherwise say yes (the trigger window)
fn pick_word(word: &'static str) -> F {
    Box::new(move |r| match r.choices.iter().position(|x| x.contains(word)) {
        Some(i) => Answer::Pick(i),
        None => Answer::Yes,
    })
}

fn astral_land_copies(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).lands.iter().copied().filter(|&l| g.perm(l).token && &*g.db.get(g.perm(l).card()).name == name).collect()
}

#[test]
fn astral_dragon_copies_a_land_that_is_also_a_dragon_creature() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 2, false);
    let vault = lands(&mut g, P0, "Treasure Vault", 1, false)[0];
    let dragon = quiet_perm(&mut g, P0, "Astral Dragon");
    seat(&mut g, P0, vec![pick_word("Treasure Vault"), pick_word("Treasure Vault"), pick_word("Treasure Vault")]);
    let (nl, np) = (g.player(P0).lands.len(), g.player(P0).perms.len());
    astral_etb(&mut g, P0, dragon);
    let toks = astral_land_copies(&g, P0, "Treasure Vault");
    assert_eq!(toks.len(), 2, "two token copies of the land, in the lands");
    assert_eq!(g.player(P0).lands.len(), nl + 2);
    assert_eq!(g.player(P0).perms.len(), np + 2, "listed as creatures too");
    for &t in &toks {
        assert!(g.is_creature(t) && sim_core::engine::values::has_type(&g, t, "dragon"));
        assert_eq!((sim_core::engine::values::epow(&g, t), sim_core::engine::values::etgh(&g, t)), (3, 3));
        assert!(g.perm(t).fly && g.perm(t).sick, "flying, summoning sick");
    }
    assert!(g.player(P0).lands.contains(&vault) && !g.is_creature(vault));
    // summoning sick: no mana from them yet; next turn they tap for {C}
    let before = sim_core::engine::mana::total_mana(&g, P0, false);
    for &t in &toks {
        g.perm_mut(t).sick = false;
    }
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), before + 2, "each taps for {{C}} once it is not sick");
    // a creature: it attacks (the opponent has no blocker)
    g.humans = Default::default();
    let life = g.player(P1).life + g.player(P2).life;
    sim_core::engine::combat::combat(&mut g, P0).unwrap();
    assert!(g.player(P1).life + g.player(P2).life < life, "the Dragon lands attack");
    assert!(toks.iter().any(|&t| g.perm(t).tapped), "attacking taps them");
    // it dies as a creature: no card, no land left
    let gy = g.player(P0).gy.len();
    sim_core::engine::zones::die(&mut g, toks[0], "dmg").unwrap();
    assert!(!g.player(P0).lands.contains(&toks[0]) && !g.player(P0).perms.contains(&toks[0]));
    assert_eq!(g.player(P0).gy.len(), gy, "a token leaves no card");
    // and as a land it can be destroyed (Strip Mine, Wasteland)
    fixes::destroy_land(&mut g, P0, toks[1]).unwrap();
    assert!(!g.perm(toks[1]).on_bf && !g.player(P0).perms.contains(&toks[1]));
    assert_eq!(g.player(P0).gy.len(), gy);
}

#[test]
fn astral_dragon_copying_an_aura_makes_aura_creatures_that_go_to_the_graveyard() {
    let mut g = tbl();
    let aura = card(&g, "Arrest");
    let a0 = quiet_perm(&mut g, P0, "Arrest"); // (the Aura to copy: what it enchants does not matter here)
    let dragon = quiet_perm(&mut g, P0, "Astral Dragon");
    seat(&mut g, P0, (0..12).map(|_| pick_word("Arrest")).collect());
    let toks = g.player(P0).perms.len();
    let gy0 = g.player(P0).gy.iter().filter(|&&c| &*g.db.get(c).name == "Arrest").count();
    astral_etb(&mut g, P0, dragon);
    assert_eq!(g.player(P0).perms.len(), toks, "both Aura creature tokens are gone (303.4d)");
    let entered = g.entered.iter().filter(|e| g.perm(e.1).token && g.perm(e.1).cd == Some(aura)).count();
    assert_eq!(entered, 2, "both Aura creature tokens did enter (their enters triggers saw them)");
    let _ = a0;
    assert_eq!(
        g.player(P0).gy.iter().filter(|&&c| &*g.db.get(c).name == "Arrest").count(),
        gy0,
        "tokens cease to exist"
    );
}

#[test]
fn the_ai_still_copies_a_nonland_permanent() {
    let mut g = tbl();
    lands(&mut g, P0, "Treasure Vault", 1, false);
    perm(&mut g, P0, "Sol Ring");
    let dragon = quiet_perm(&mut g, P0, "Astral Dragon");
    astral_etb(&mut g, P0, dragon);
    assert!(astral_land_copies(&g, P0, "Treasure Vault").is_empty());
    let sol = card(&g, "Sol Ring");
    assert_eq!(g.player(P0).perms.iter().filter(|&&m| g.perm(m).token && g.perm(m).cd == Some(sol)).count(), 2);
}

