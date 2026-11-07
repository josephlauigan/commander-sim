//! Ringsight against its Oracle text: "The Ring tempts you. Search your library for a card that shares a color with a
//! legendary creature you control, reveal it, put it into your hand, then shuffle." (`tut=legcol` in data/carddb.txt;
//! the colours resolve per caster in `ai::decks::tut_kind`. It used to be a fixed blue/black/red search, from Sauron.)

use sim_core::ai::decks::{tut_kind, tutor_ok};
use sim_core::engine::cast;
use sim_core::ids::PlayerId;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

/// P0's library is exactly these cards
fn library(g: &mut Game, names: &[&str]) {
    let lib = names.iter().map(|n| card(g, n)).collect();
    g.player_mut(P0).library = lib;
}

fn cast_ringsight(g: &mut Game) {
    let c = take(g, P0, "Ringsight");
    cast::resolve(g, P0, c, &Ctx::default(), "hand").unwrap();
}

#[test]
fn with_yshtola_out_it_finds_white_blue_or_black_cards_only() {
    let mut g = table(&["yshtola", "cloud"]);
    perm(&mut g, P0, "Y'shtola, Night's Blessed");
    assert_eq!(tut_kind(&g, P0, "legcol"), "col:WUB");
    let k = tut_kind(&g, P0, "legcol");
    assert!(tutor_ok(&g, &k, card(&g, "Swords to Plowshares")), "white shares a colour with Y'shtola");
    assert!(tutor_ok(&g, &k, card(&g, "Counterspell")), "blue");
    assert!(!tutor_ok(&g, &k, card(&g, "Lightning Bolt")), "red does not");
    assert!(!tutor_ok(&g, &k, card(&g, "Sol Ring")), "a colourless card has no colour to share");
    assert!(!tutor_ok(&g, &k, card(&g, "Plains")), "lands aren't searched for");
}

#[test]
fn it_takes_the_white_card_a_red_card_cannot_be_found() {
    let mut g = table(&["yshtola", "cloud"]);
    perm(&mut g, P0, "Y'shtola, Night's Blessed");
    library(&mut g, &["Lightning Bolt", "Swords to Plowshares"]);
    cast_ringsight(&mut g);
    let hand = &g.player(P0).hand;
    assert!(hand.contains(&card(&g, "Swords to Plowshares")), "the white card is found");
    assert!(!hand.contains(&card(&g, "Lightning Bolt")), "the red card is not");
    assert!(g.player(P0).library.contains(&card(&g, "Lightning Bolt")));
}

#[test]
fn the_ring_bearer_counts_as_legendary() {
    // no legendary creature: the Ring tempts first, its Ring-bearer becomes legendary, and its colour counts
    let mut g = table(&["yshtola", "cloud"]);
    perm(&mut g, P0, "Restoration Angel");
    library(&mut g, &["Lightning Bolt", "Swords to Plowshares"]);
    cast_ringsight(&mut g);
    assert!(g.player(P0).hand.contains(&card(&g, "Swords to Plowshares")), "white, the Ring-bearer's colour");
}

#[test]
fn a_token_ring_bearer_counts_by_its_token_colour() {
    // a token has no card: its colour is the token's own (this panicked before the fix)
    let mut g = table(&["yshtola", "cloud"]);
    let t = token(&mut g, P0, 1);
    g.player_mut(P0).ring_bearer = Some(t);
    g.player_mut(P0).ring_level = 1;
    let k = tut_kind(&g, P0, "legcol"); // must not panic
    assert!(k.starts_with("col:"));
    library(&mut g, &["Lightning Bolt", "Swords to Plowshares"]);
    cast_ringsight(&mut g);
}

#[test]
fn with_no_creature_it_finds_nothing() {
    let mut g = table(&["yshtola", "cloud"]);
    library(&mut g, &["Lightning Bolt", "Swords to Plowshares"]);
    let before = g.player(P0).hand.len();
    cast_ringsight(&mut g);
    assert_eq!(g.player(P0).hand.len(), before, "no legendary creature, so no colour to share");
    assert_eq!(g.player(P0).library.len(), 2);
}
