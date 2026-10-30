//! Phase 3 (one arena): a land is a permanent, so the ability language runs its compiled abilities, one path per
//! effect. A temple scries once and a gain land gains once (their compiled enter triggers; the engine's land_etb_fx is
//! gone); a surveil land surveils (it never did); Seraph Sanctuary's Angel trigger runs; a fetch land cracks once (the
//! f tag) and its compiled fetch is not offered; Boseiju channels from the hand, not the battlefield; Castle Ardenvale
//! makes a Soldier, paid by other lands; Skemfar Elderhall is sacrificed as a land; a manland animates and attacks.
//! Seat 0 plays the sauron deck, seat 1 the seph deck.

use sim_core::ai::act::perform;
use sim_core::cardcode;
use sim_core::engine::{combat, turn, zones};
use sim_core::hooks::Opt;
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["sauron", "seph"])
}

/// a land card enters under p's control the way a land drop does (its enters effects, then landfall)
fn land_enters(g: &mut Game, p: PlayerId, name: &str) -> PermId {
    let c = take(g, p, name);
    let l = g.add_land_entering(p, c, true);
    zones::landfall(g, p).unwrap();
    l
}

fn named(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).lands.iter().copied().find(|&l| &*g.db.get(g.perm(l).card()).name == name)
}

fn ability(g: &mut Game, p: PlayerId, label: &str) -> Option<Opt> {
    sim_core::dsl::ability_options(g, p, true).unwrap().into_iter().find(|o| o.label == label)
}

#[test]
fn a_temple_scries_once_as_it_enters() {
    // with ten lands out, an extra land is not wanted on top: one scry 1 sends the Island under, the Swamp stays on top
    // (a second scry would send it under too)
    let mut g = game();
    lands(&mut g, P0, "Swamp", 10, false);
    let (island, swamp) = (take(&mut g, P0, "Island"), take(&mut g, P0, "Swamp"));
    g.player_mut(P0).library.extend([swamp, island]); // the top is last
    assert!(g.player(P0).scry_turn.is_none());
    land_enters(&mut g, P0, "Temple of Malady");
    assert!(g.player(P0).scry_turn.is_some(), "the compiled scry 1 ran");
    let lib = &g.player(P0).library;
    assert_eq!((lib[0], *lib.last().unwrap()), (island, swamp), "scried once");
}

#[test]
fn a_surveil_land_surveils_as_it_enters() {
    for name in ["Commercial District", "Hidden Grotto", "Undercity Sewers", "Underground Mortuary"] {
        let mut g = game();
        assert!(g.player(P0).scry_turn.is_none());
        land_enters(&mut g, P0, name);
        assert!(g.player(P0).scry_turn.is_some(), "{name} surveils 1");
    }
}

#[test]
fn seraph_sanctuary_gains_as_it_enters_and_as_an_angel_enters() {
    let mut g = game();
    let life = g.player(P0).life;
    land_enters(&mut g, P0, "Seraph Sanctuary");
    assert_eq!(g.player(P0).life, life + 1, "its own enter trigger");
    perm(&mut g, P0, "Serra Angel");
    assert_eq!(g.player(P0).life, life + 2, "an Angel you control enters");
}

#[test]
fn a_fetch_land_cracks_once_and_its_compiled_fetch_is_not_offered() {
    let mut g = game();
    let delta = card(&g, "Polluted Delta");
    take(&mut g, P0, "Polluted Delta");
    let (life, lib) = (g.player(P0).life, g.player(P0).library.len());
    turn::play_land_card(&mut g, P0, delta, "plays").unwrap();
    let pl = g.player(P0);
    assert_eq!(pl.gy.iter().filter(|&&c| c == delta).count(), 1, "sacrificed once");
    assert_eq!(pl.lands.len(), 1, "one basic fetched");
    assert_eq!(pl.library.len(), lib - 1);
    assert_eq!(pl.life, life - 1, "1 life, once");
    assert!(ability(&mut g, P0, "Polluted Delta ability").is_none());
}

#[test]
fn boseiju_channels_from_the_hand_and_not_from_the_battlefield() {
    let mut g = game();
    lands(&mut g, P0, "Forest", 2, false);
    let tithe = perm(&mut g, P1, "Smothering Tithe");
    let c = take(&mut g, P0, "Boseiju, Who Endures");
    g.player_mut(P0).hand.push(c);
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    let o = f(&mut g, c, P0, Some(false)).unwrap();
    assert_eq!(o.iter().map(|x| x.label.as_str()).collect::<Vec<_>>(), ["channel Boseiju"]);
    assert!(perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert!(!g.perm(tithe).on_bf, "the Tithe is destroyed");
    assert!(g.player(P0).gy.contains(&c));
    // on the battlefield, its compiled channel ability is never offered
    g.player_mut(P0).gy.retain(|&x| x != c);
    g.add_land(P0, c, false);
    perm(&mut g, P1, "Rhystic Study");
    assert!(ability(&mut g, P0, "Boseiju, Who Endures ability").is_none());
}

#[test]
fn castle_ardenvale_makes_a_soldier_paid_by_other_lands() {
    let mut g = game();
    let castle = g.add_land(P0, card(&g, "Castle Ardenvale"), false);
    let plains = lands(&mut g, P0, "Plains", 4, false);
    let before = g.player(P0).perms.len();
    let o = ability(&mut g, P0, "Castle Ardenvale ability").expect("offered");
    assert!(perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P0).perms.len(), before + 1, "a 1/1 token");
    let m = *g.player(P0).perms.last().unwrap();
    assert_eq!((g.perm(m).pow, g.perm(m).tgh, g.perm(m).token), (1, 1, true));
    assert!(g.perm(castle).tapped, "{{T}}");
    assert!(plains.iter().all(|&l| g.perm(l).tapped), "{{2}}{{W}}{{W}} from the four Plains, not the Castle");
    assert!(ability(&mut g, P0, "Castle Ardenvale ability").is_none(), "tapped: not again");
}

#[test]
fn skemfar_elderhall_is_sacrificed_as_a_land() {
    let mut g = game();
    g.active = Some(P0);
    let hall = g.add_land(P0, card(&g, "Skemfar Elderhall"), false);
    lands(&mut g, P0, "Swamp", 2, false);
    lands(&mut g, P0, "Forest", 3, false);
    let o = ability(&mut g, P0, "Skemfar Elderhall ability").expect("offered at sorcery speed");
    assert!(perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(named(&g, P0, "Skemfar Elderhall").is_none() && !g.perm(hall).on_bf, "it left the battlefield");
    assert!(g.player(P0).gy.contains(&card(&g, "Skemfar Elderhall")), "into the graveyard");
    assert_eq!(g.player(P0).perms.len(), 2, "two Elf Warriors");
}

#[test]
fn a_manland_animates_and_attacks() {
    let mut g = game();
    g.active = Some(P0);
    lands(&mut g, P0, "Mutavault", 1, false);
    lands(&mut g, P0, "Swamp", 2, false);
    let o = cardcode::land_options(&mut g, P0, Some(false)).unwrap();
    assert_eq!(o.iter().map(|x| x.label.as_str()).collect::<Vec<_>>(), ["animate Mutavault"]);
    assert!(perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    let life = g.player(P1).life;
    combat::combat(&mut g, P0).unwrap();
    assert_eq!(g.player(P1).life, life - 2, "the 2/2 Mutavault connects");
}
