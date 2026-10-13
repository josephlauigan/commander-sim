//! A card played from another player's deck stays theirs (CR 400.3: it goes to its owner's graveyard): Dauthi
//! Voidwalker, Fallen Shinobi, Gix, Tinybones and a spell countered by Jin-Gitaxias. Before these fixes the card went
//! to the graveyard of the player who cast it (found in a search playout of the 2026-10-12 run: Jodah cast Yuriko's
//! Treasure Cruise with her Dauthi Voidwalker, and its delve priced itself until the stack overflowed). Also the
//! plan milestones of the titania and sheoldred decks.

use sim_core::ai::act::perform;
use sim_core::engine::{cast, zones};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let cd = g.perm(src).cd.unwrap();
    let f = g.registry.get(cd).and_then(|i| i.options).expect("the card has options");
    f(g, src, p, post).unwrap()
}

fn option_with(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>, word: &str) -> Action {
    options(g, src, p, post).into_iter().find(|o| o.label.contains(word)).and_then(|o| o.act).expect("the option")
}

fn in_gy(g: &Game, p: PlayerId, c: CardId) -> usize {
    g.player(p).gy.iter().filter(|&&x| x == c).count()
}

#[test]
fn dauthi_voidwalker_casts_an_opponents_card_and_it_goes_back_to_their_graveyard() {
    let mut g = table(&["yuriko-dimir-ninjas", "galadriel", "sheoldred"]);
    let dauthi = perm(&mut g, P0, "Dauthi Voidwalker");
    let cruise = card(&g, "Treasure Cruise");
    g.player_mut(P1).gy.push(cruise);
    let sweep = g.registry.get(card(&g, "Dauthi Voidwalker")).unwrap().sba.unwrap();
    sweep(&mut g, dauthi).unwrap(); // exiled with a void counter
    assert!(g.player(P1).exile.contains(&cruise));
    let a = option_with(&mut g, dauthi, P0, Some(false), "Treasure Cruise");
    assert!(perform(&mut g, P0, &a).unwrap());
    assert_eq!(in_gy(&g, P1, cruise), 1, "Treasure Cruise is the second player's card");
    assert_eq!(in_gy(&g, P0, cruise), 0);
}

#[test]
fn fallen_shinobi_casts_the_damaged_players_spells_into_their_graveyard() {
    let mut g = table(&["yuriko-dimir-ninjas", "galadriel", "sheoldred"]);
    let shinobi = perm(&mut g, P0, "Fallen Shinobi");
    let (bs, po) = (card(&g, "Brainstorm"), card(&g, "Ponder"));
    let mut lib = g.player(P1).library.clone();
    lib.retain(|&c| c != bs && c != po);
    lib.extend([bs, po]); // the top two (the library's top is its end)
    g.player_mut(P1).library = lib;
    let f = g.registry.get(card(&g, "Fallen Shinobi")).unwrap().combat_damage.unwrap();
    f(&mut g, shinobi, P0, shinobi, P1, 5).unwrap();
    assert_eq!((in_gy(&g, P1, bs), in_gy(&g, P1, po)), (1, 1));
    assert_eq!((in_gy(&g, P0, bs), in_gy(&g, P0, po)), (0, 0));
}

#[test]
fn gix_casts_an_opponents_library_cards_into_their_graveyard() {
    let mut g = table(&["sheoldred", "galadriel", "yuriko-dimir-ninjas"]);
    let gix = perm(&mut g, P0, "Gix, Yawgmoth Praetor");
    lands(&mut g, P0, "Swamp", 7, false);
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Swamp", "Swamp"]);
    let (bs, po) = (card(&g, "Brainstorm"), card(&g, "Ponder"));
    g.player_mut(P1).library = vec![card(&g, "Sol Ring"), card(&g, "Lightning Bolt"), bs, po];
    g.player_mut(P2).library.truncate(2); // Gix picks the longest library: the second player's
    let a = option_with(&mut g, gix, P0, Some(false), "Gix");
    assert!(perform(&mut g, P0, &a).unwrap());
    assert_eq!((in_gy(&g, P1, bs), in_gy(&g, P1, po)), (1, 1));
    assert_eq!((in_gy(&g, P0, bs), in_gy(&g, P0, po)), (0, 0));
}

#[test]
fn tinybones_takes_one_permanent_from_one_graveyard_and_it_stays_its_owners() {
    let mut g = table(&["tergrid-mono-black-disruption", "sauron", "galadriel"]);
    let t = perm(&mut g, P0, "Tinybones, Trinket Thief");
    let (ring, bolt) = (card(&g, "Sol Ring"), card(&g, "Lightning Bolt"));
    g.player_mut(P1).gy.extend([ring, bolt]);
    g.player_mut(P2).gy.push(ring);
    g.player_mut(P1).discarded_turn = Some(g.turn_stamp());
    let f = g.registry.get(card(&g, "Tinybones, Trinket Thief")).unwrap().end_step.unwrap();
    f(&mut g, t, P0).unwrap();
    let rings: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).cd == Some(ring)).collect();
    assert_eq!(rings.len(), 1);
    let owner = g.perm(rings[0]).orig;
    assert!(owner == P1 || owner == P2, "Sol Ring stays its owner's");
    assert_eq!(in_gy(&g, P1, ring) + in_gy(&g, P2, ring), 1, "only one Sol Ring leaves a graveyard");
    assert_eq!(in_gy(&g, P1, bolt), 1, "an instant is not played, so it stays");
}

#[test]
fn a_spell_jin_gitaxias_counters_goes_to_its_owners_graveyard() {
    let mut g = table(&["yuriko-dimir-ninjas", "galadriel", "veyran"]);
    perm(&mut g, P2, "Jin-Gitaxias, Progress Tyrant");
    let bs = card(&g, "Brainstorm");
    g.player_mut(P0).hand.push(bs);
    let resolved = cast::cast_card(&mut g, P0, bs, "hand", Ctx { owner: Some(P1), ..Ctx::default() }).unwrap();
    assert!(!resolved, "Jin-Gitaxias counters the first one");
    assert_eq!((in_gy(&g, P1, bs), in_gy(&g, P0, bs)), (1, 0));
}

#[test]
fn titania_and_sheoldred_mark_their_plans() {
    let mut g = table(&["titania", "sheoldred", "galadriel"]);
    let t = perm(&mut g, P0, "Titania, Protector of Argoth");
    let f = g.registry.get(card(&g, "Titania, Protector of Argoth")).unwrap().land_gy.unwrap();
    let forest = card(&g, "Forest");
    f(&mut g, t, P0, forest).unwrap();
    assert!(g.player(P0).milestone.get("titania").is_some(), "the first 5/3 Elemental");

    perm(&mut g, P1, "Sheoldred, the Apocalypse");
    assert!(g.player(P1).milestone.get("sheoldred").is_none());
    zones::draw(&mut g, P2, 1, false).unwrap();
    assert!(g.player(P1).milestone.get("sheoldred").is_some(), "an opponent drew and lost 2");
}
