//! The Anikthea precon's cards (impls/t0_anikthea.rs): each card on a hand-built position, with its Oracle text as the
//! expectation.

use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    table(&["anikthea-enduring-enchantments", "galadriel", "veyran"])
}

fn on(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

/// the beginning of p's precombat main phase for one permanent (a lore counter after the draw step)
fn main1(g: &mut Game, m: PermId, p: PlayerId) {
    let c = g.perm(m).cd.unwrap();
    let f = g.registry.get(c).unwrap().main1.unwrap();
    f(g, m, p).unwrap();
}

fn names(g: &Game, p: PlayerId) -> Vec<String> {
    g.player(p).perms.iter().map(|&m| g.perm(m).name.to_string()).collect()
}

fn count_tokens(g: &Game, p: PlayerId, ty: &str) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).token && g.perm(m).ttypes.contains(&ty)).count()
}

fn put_gy(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = take(g, p, name);
    g.player_mut(p).gy.push(c);
    c
}

// ======================================================== Sagas
#[test]
fn eldest_reborn_chapters_one_to_three() {
    let mut g = tbl();
    let small = token(&mut g, P1, 1);
    let big = perm(&mut g, P1, "Grave Titan");
    let before = g.player(P1).perms.len();
    hand(&mut g, P1, &["Sol Ring", "Mind Stone"]);
    let dead = put_gy(&mut g, P1, "Kraum, Ludevic's Opus");
    let s = perm(&mut g, P0, "The Eldest Reborn");
    assert_eq!(g.player(P1).perms.len(), before - 1, "chapter I: the opponent sacrificed one creature");
    assert!(on(&g, big) && !on(&g, small), "the weakest creature was the one sacrificed");
    let hand_before = g.player(P1).hand.len();
    main1(&mut g, s, P0);
    assert_eq!(g.player(P1).hand.len(), hand_before - 1, "chapter II: a discard");
    assert!(on(&g, s));
    main1(&mut g, s, P0);
    assert!(!on(&g, s), "sacrificed after chapter III");
    assert!(!g.player(P1).gy.contains(&dead), "chapter III took a creature card from a graveyard");
    assert!(names(&g, P0).iter().any(|n| n.starts_with("Kraum") || n == "Grave Titan"), "under P0's control: {:?}", names(&g, P0));
}

#[test]
fn helvault_exiles_until_it_leaves_then_avacyn() {
    let mut g = tbl();
    let big = perm(&mut g, P1, "Grave Titan");
    let s = perm(&mut g, P0, "Battle at the Helvault");
    assert!(!on(&g, big), "chapter I exiled the best permanent");
    main1(&mut g, s, P0);
    main1(&mut g, s, P0);
    assert!(!on(&g, s), "sacrificed after chapter III");
    assert_eq!(count_tokens(&g, P0, "angel"), 1, "Avacyn");
    let av = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token).unwrap();
    assert_eq!((g.perm(av).pow, g.perm(av).tgh), (8, 8));
    assert!(g.perm(av).vig && g.perm(av).fly);
    assert!(g.perm(av).data.truthy(DataKey::Indestr));
    assert!(
        g.player(P1).perms.iter().any(|&m| g.perm(m).cd == Some(card(&g, "Grave Titan"))),
        "the exiled Titan came back when the Saga was sacrificed"
    );
}

#[test]
fn bretagard_copies_each_token_name_once() {
    let mut g = tbl();
    let s = perm(&mut g, P0, "Battle for Bretagard");
    assert_eq!(count_tokens(&g, P0, "human"), 1);
    main1(&mut g, s, P0);
    assert_eq!(count_tokens(&g, P0, "elf"), 1);
    main1(&mut g, s, P0);
    assert_eq!(count_tokens(&g, P0, "human"), 2, "a copy of the Human Warrior");
    assert_eq!(count_tokens(&g, P0, "elf"), 2, "a copy of the Elf Warrior");
    assert!(!on(&g, s));
}

#[test]
fn bretagard_copies_a_treasure_kind_too() {
    let mut g = tbl();
    g.player_mut(P0).treasures = 3;
    let s = perm(&mut g, P0, "Battle for Bretagard");
    main1(&mut g, s, P0);
    main1(&mut g, s, P0);
    assert_eq!(g.player(P0).treasures, 4, "one more Treasure, not one per Treasure");
}

#[test]
fn love_song_reads_ahead_to_chapter_two() {
    let mut g = tbl();
    let n = g.player(P0).hand.len();
    let s = perm(&mut g, P0, "Love Song of Night and Day");
    assert_eq!(count_tokens(&g, P0, "bird"), 1, "chapter II first: a Bird");
    assert_eq!(g.player(P0).hand.len(), n, "chapter I skipped");
    main1(&mut g, s, P0);
    assert!(!on(&g, s));
    let bird = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token).unwrap();
    assert_eq!(g.perm(bird).plus, 1, "chapter III: a +1/+1 counter");
}

#[test]
fn binding_the_old_gods_destroys_fetches_and_gives_deathtouch() {
    let mut g = tbl();
    let big = perm(&mut g, P1, "Grave Titan");
    let mine = token(&mut g, P0, 2);
    let s = perm(&mut g, P0, "Binding the Old Gods");
    assert!(!on(&g, big), "chapter I destroyed the Titan");
    let lands_before = g.player(P0).lands.len();
    main1(&mut g, s, P0);
    assert_eq!(g.player(P0).lands.len(), lands_before + 1, "chapter II: a Forest");
    main1(&mut g, s, P0);
    assert!(g.perm(mine).eot_kw.contains(&"deathtouch"), "chapter III");
    assert!(!on(&g, s));
}

#[test]
fn mending_of_dominaria_returns_lands_and_shuffles() {
    let mut g = tbl();
    let sol = take(&mut g, P0, "Sol Ring");
    g.player_mut(P0).library = vec![sol; 20];
    let s = perm(&mut g, P0, "The Mending of Dominaria");
    put_gy(&mut g, P0, "Forest");
    put_gy(&mut g, P0, "Plains");
    put_gy(&mut g, P0, "Sol Ring");
    let lands_before = g.player(P0).lands.len();
    main1(&mut g, s, P0);
    main1(&mut g, s, P0);
    assert_eq!(g.player(P0).lands.len(), lands_before + 2, "both lands returned");
    let saga = card(&g, "The Mending of Dominaria");
    assert_eq!(g.player(P0).gy, vec![saga], "the graveyard was shuffled into the library; the sacrificed Saga came after");
    assert!(!on(&g, s));
}

#[test]
fn binding_of_the_titans_mills_exiles_and_returns() {
    let mut g = tbl();
    let lib = g.player(P1).library.len();
    let s = perm(&mut g, P0, "The Binding of the Titans");
    assert_eq!(g.player(P1).library.len(), lib - 3, "chapter I: each player mills three");
    let c = put_gy(&mut g, P1, "Grave Titan");
    let life = g.player(P0).life;
    main1(&mut g, s, P0);
    assert!(g.player(P1).exile.contains(&c), "chapter II exiled the creature card");
    assert!(g.player(P0).life > life, "and gained life for it");
    let mine = put_gy(&mut g, P0, "Kraum, Ludevic's Opus");
    main1(&mut g, s, P0);
    assert!(g.player(P0).hand.contains(&mine), "chapter III");
}
