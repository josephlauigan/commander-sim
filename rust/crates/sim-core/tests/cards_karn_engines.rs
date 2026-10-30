//! Karn deck engines (impls/karn_engines.rs): Karn, Scion of Urza; Mazemind Tome; Mirrorworks; Mystic Forge; Urza's
//! Blueprints; Urza's Tome. Each on a hand-built position with its Oracle text as the expectation.

use sim_core::hooks::{Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    let mut g = table(&["karn", "seph"]);
    g.active = Some(P0);
    g
}

fn opts(g: &mut Game, src: PermId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, P0, post).unwrap()
}

fn go(g: &mut Game, o: &Opt) -> bool {
    sim_core::ai::act::perform(g, P0, o.act.as_ref().unwrap()).unwrap()
}

fn on(g: &Game, p: PlayerId, name: &str) -> usize {
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| g.perm(m).on_bf && g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == name))
        .count()
}

/// put these cards on top of p's library, the last name on top
fn stack_top(g: &mut Game, p: PlayerId, names: &[&str]) {
    for n in names {
        let c = take(g, p, n);
        g.player_mut(p).library.push(c);
    }
}

fn name(g: &Game, c: CardId) -> String {
    g.db.get(c).name.to_string()
}

// ======================================================== Karn, Scion of Urza
#[test]
fn karn_enters_with_five_loyalty() {
    let mut g = tbl();
    let k = perm(&mut g, P0, "Karn, Scion of Urza");
    assert_eq!(g.perm(k).loyalty, Some(5));
}

#[test]
fn karn_plus_gives_one_card_and_exiles_the_other_with_a_silver_counter() {
    let mut g = tbl();
    let k = perm(&mut g, P0, "Karn, Scion of Urza");
    stack_top(&mut g, P0, &["Wastes", "Wurmcoil Engine"]); // Wurmcoil on top
    let hand0 = g.player(P0).hand.len();
    let o = opts(&mut g, k, Some(false));
    let plus = o.iter().find(|o| o.label.contains("+1")).expect("+1");
    assert!(go(&mut g, plus));
    assert_eq!(g.perm(k).loyalty, Some(6));
    assert_eq!(g.player(P0).hand.len(), hand0 + 1, "one card to the hand");
    assert_eq!(g.player(P0).exile.silver().len(), 1, "the other has a silver counter");
    let silver = g.player(P0).exile.silver()[0];
    assert!(g.player(P0).exile.contains(&silver));
    let got = *g.player(P0).hand.last().unwrap();
    assert_ne!(got, silver);
    let both = [name(&g, got), name(&g, silver)];
    assert!(both.contains(&"Wastes".to_string()) && both.contains(&"Wurmcoil Engine".to_string()), "{both:?}");
    assert_eq!(name(&g, got), "Wastes", "the opponent hands over the card you want less");
}

#[test]
fn karn_minus_one_returns_the_silver_card_and_minus_two_makes_a_construct() {
    let mut g = tbl();
    let k = perm(&mut g, P0, "Karn, Scion of Urza");
    stack_top(&mut g, P0, &["Wastes", "Wurmcoil Engine"]);
    let plus = opts(&mut g, k, Some(false)).into_iter().find(|o| o.label.contains("+1")).unwrap();
    go(&mut g, &plus);
    g.perm_mut(k).loyalty_used = None;
    let m1 = opts(&mut g, k, Some(false)).into_iter().find(|o| o.label.contains("-1")).expect("-1 with a silver card");
    let hand0 = g.player(P0).hand.len();
    assert!(go(&mut g, &m1));
    assert_eq!(g.player(P0).hand.len(), hand0 + 1);
    assert_eq!(name(&g, *g.player(P0).hand.last().unwrap()), "Wurmcoil Engine");
    assert!(g.player(P0).exile.silver().is_empty());
    assert!(opts(&mut g, k, Some(false)).iter().all(|o| !o.label.contains("-1")), "nothing left to return");

    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P0, "Mind Stone");
    g.perm_mut(k).loyalty_used = None;
    g.perm_mut(k).loyalty = Some(5);
    let m2 = opts(&mut g, k, Some(false)).into_iter().find(|o| o.label.contains("-2")).expect("-2");
    assert!(go(&mut g, &m2));
    assert_eq!(g.perm(k).loyalty, Some(3));
    let con =
        g.player(P0).perms.iter().copied().find(|&m| g.perm(m).data.truthy(DataKey::Construct)).expect("a Construct");
    assert!(g.perm(con).token);
    assert_eq!(
        (sim_core::engine::values::epow(&g, con), sim_core::engine::values::etgh(&g, con)),
        (3, 3),
        "Sol Ring, Mind Stone and itself"
    );
}

// ======================================================== Mazemind Tome
#[test]
fn mazemind_tome_draws_for_two_and_is_exiled_at_four_pages_for_four_life() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 4, false);
    let t = perm(&mut g, P0, "Mazemind Tome");
    let life = g.player(P0).life;
    let hand0 = g.player(P0).hand.len();
    for i in 1..=4 {
        g.perm_mut(t).tapped = false;
        let d = opts(&mut g, t, Some(false)).into_iter().find(|o| o.label.contains("draw")).expect("draw option");
        assert!(go(&mut g, &d), "use {i}");
        if i < 4 {
            assert!(g.perm(t).on_bf);
            assert_eq!(g.perm(t).data.int(DataKey::Counters), i);
        }
        for l in g.player(P0).lands.clone() {
            g.perm_mut(l).tapped = false;
        }
    }
    assert_eq!(g.player(P0).hand.len(), hand0 + 4);
    assert!(!g.perm(t).on_bf, "exiled at the fourth page counter");
    assert!(g.player(P0).exile.iter().any(|&c| name(&g, c) == "Mazemind Tome"));
    assert_eq!(g.player(P0).life, life + 4);
}

#[test]
fn mazemind_tome_scrys_when_it_cannot_pay() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Mazemind Tome");
    let o = opts(&mut g, t, Some(false));
    let s = o.iter().find(|o| o.label.contains("scry")).expect("scry without mana");
    assert!(go(&mut g, s));
    assert!(g.perm(t).tapped);
    assert_eq!(g.perm(t).data.int(DataKey::Counters), 1);
}

// ======================================================== Urza's Tome
#[test]
fn urzas_tome_draws_and_exiles_a_historic_card_instead_of_discarding() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 3, false);
    let t = perm(&mut g, P0, "Urza's Tome");
    let sr = take(&mut g, P0, "Sol Ring");
    g.player_mut(P0).gy.push(sr);
    let hand0 = g.player(P0).hand.len();
    let o = opts(&mut g, t, Some(false));
    assert!(go(&mut g, &o[0]));
    assert_eq!(g.player(P0).hand.len(), hand0 + 1, "drew and did not discard");
    assert!(g.player(P0).exile.contains(&sr), "Sol Ring (an artifact) is exiled from the graveyard");
    assert!(!g.player(P0).gy.contains(&sr));
}

#[test]
fn urzas_tome_discards_without_a_historic_card() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 3, false);
    let t = perm(&mut g, P0, "Urza's Tome");
    let w = take(&mut g, P0, "Wastes");
    g.player_mut(P0).gy.push(w); // a basic land is not historic
    let hand0 = g.player(P0).hand.len();
    let gy0 = g.player(P0).gy.len();
    let o = opts(&mut g, t, Some(false));
    assert!(go(&mut g, &o[0]));
    assert_eq!(g.player(P0).hand.len(), hand0, "drew one, discarded one");
    assert_eq!(g.player(P0).gy.len(), gy0 + 1);
    assert!(g.player(P0).exile.is_empty());
}

#[test]
fn urzas_tome_needs_three_mana() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 2, false);
    let t = perm(&mut g, P0, "Urza's Tome");
    assert!(opts(&mut g, t, Some(false)).is_empty());
}

// ======================================================== Mirrorworks
#[test]
fn mirrorworks_copies_a_nontoken_artifact_for_two() {
    let mut g = tbl();
    perm(&mut g, P0, "Mirrorworks");
    lands(&mut g, P0, "Wastes", 2, false);
    perm(&mut g, P0, "Sol Ring");
    assert_eq!(on(&g, P0, "Sol Ring"), 2, "the card and its token copy");
    let toks = g.player(P0).perms.iter().filter(|&&m| g.perm(m).token && g.perm(m).on_bf).count();
    assert_eq!(toks, 1, "the copy does not copy itself");
    assert!(g.player(P0).lands.iter().all(|&l| g.perm(l).tapped), "{{2}} was paid");
}

#[test]
fn mirrorworks_does_nothing_without_the_mana_or_for_an_opponents_artifact() {
    let mut g = tbl();
    perm(&mut g, P0, "Mirrorworks");
    perm(&mut g, P0, "Mind Stone");
    assert_eq!(on(&g, P0, "Mind Stone"), 1, "no mana for the copy");
    lands(&mut g, P0, "Wastes", 2, false);
    perm(&mut g, P1, "Sol Ring");
    assert_eq!(on(&g, P1, "Sol Ring"), 1);
    assert_eq!(on(&g, P0, "Sol Ring"), 0);
}

#[test]
fn mirrorworks_skips_a_legend() {
    let mut g = tbl();
    perm(&mut g, P0, "Mirrorworks");
    lands(&mut g, P0, "Wastes", 2, false);
    perm(&mut g, P0, "Karn, Legacy Reforged");
    assert_eq!(on(&g, P0, "Karn, Legacy Reforged"), 1);
}

// ======================================================== Mystic Forge
#[test]
fn mystic_forge_casts_an_artifact_from_the_top_of_the_library() {
    let mut g = tbl();
    let f = perm(&mut g, P0, "Mystic Forge");
    lands(&mut g, P0, "Wastes", 2, false);
    stack_top(&mut g, P0, &["Sol Ring"]);
    let o = opts(&mut g, f, Some(false));
    let cast = o.iter().find(|o| o.label.contains("cast Sol Ring")).expect("cast option");
    let lib = g.player(P0).library.len();
    assert!(go(&mut g, cast));
    assert_eq!(on(&g, P0, "Sol Ring"), 1);
    assert_eq!(g.player(P0).library.len(), lib - 1);
}

#[test]
fn mystic_forge_cannot_cast_a_coloured_card_or_without_mana() {
    let mut g = tbl();
    let f = perm(&mut g, P0, "Mystic Forge");
    stack_top(&mut g, P0, &["Sol Ring"]);
    assert!(opts(&mut g, f, Some(false)).iter().all(|o| !o.label.contains("cast")), "no mana");
    lands(&mut g, P0, "Wastes", 1, false);
    stack_top(&mut g, P0, &["Wastes"]);
    assert!(opts(&mut g, f, Some(false)).iter().all(|o| !o.label.contains("cast")), "a land on top is not cast");
}

#[test]
fn mystic_forge_exiles_a_land_for_a_life_when_it_has_plenty() {
    let mut g = tbl();
    let f = perm(&mut g, P0, "Mystic Forge");
    lands(&mut g, P0, "Wastes", 5, false);
    stack_top(&mut g, P0, &["Wastes"]);
    let life = g.player(P0).life;
    let o = opts(&mut g, f, Some(false));
    let ex = o.iter().find(|o| o.label.contains("exile")).expect("exile the land");
    assert!(go(&mut g, ex));
    assert_eq!(g.player(P0).life, life - 1);
    assert!(g.perm(f).tapped);
    assert!(g.player(P0).exile.iter().any(|&c| name(&g, c) == "Wastes"));
}

#[test]
fn mystic_forge_does_not_draw_a_card_any_more() {
    let mut g = tbl();
    let f = perm(&mut g, P0, "Mystic Forge");
    let c = g.perm(f).cd.unwrap();
    assert!(!g.db.get(c).has_dsl(), "the old compiled text drew a card");
}

// ======================================================== Urza's Blueprints
fn upkeep(g: &mut Game) {
    sim_core::engine::hooks::fire_trigger(g, Event::Upkeep, sim_core::hooks::Call::Player { p: P0 }).unwrap();
}

#[test]
fn blueprints_echo_is_paid_with_spare_mana() {
    let mut g = tbl();
    let b = perm(&mut g, P0, "Urza's Blueprints");
    lands(&mut g, P0, "Wastes", 9, false);
    upkeep(&mut g);
    assert!(g.perm(b).on_bf, "echo paid");
    assert!(g.player(P0).lands.iter().filter(|&&l| g.perm(l).tapped).count() >= 6);
    upkeep(&mut g);
    assert!(g.perm(b).on_bf, "echo is paid only once");
}

#[test]
fn blueprints_are_sacrificed_when_echo_is_not_paid() {
    let mut g = tbl();
    let b = perm(&mut g, P0, "Urza's Blueprints");
    lands(&mut g, P0, "Wastes", 3, false);
    upkeep(&mut g);
    assert!(!g.perm(b).on_bf);
    assert!(g.player(P0).gy.iter().any(|&c| name(&g, c) == "Urza's Blueprints"));
}

#[test]
fn blueprints_draws_on_tap() {
    let mut g = tbl();
    let b = perm(&mut g, P0, "Urza's Blueprints");
    let c = g.perm(b).cd.unwrap();
    assert!(g.db.get(c).has_dsl(), "the compiled {{T}}: draw");
}
