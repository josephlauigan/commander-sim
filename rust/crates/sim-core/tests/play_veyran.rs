//! Practice mode, Veyran's spells: the choices the card code makes for the AI are yours as they resolve (Crackle
//! with Power's X and targets, Mizzix's Mastery, Flashback, Jeska's Will, Expressive Iteration, Flow State, Stock Up,
//! Prismari Charm) and copies of your spells take new targets; Alania, Divergent Storm's copies (the AI's and yours)
//! (Python's tests/test_play_veyran.py).

mod play_common;
use play_common::*;
use sim_core::engine::cast::{cast_card, on_cast};
use sim_core::state::Ctx;

fn gy_push(g: &mut Game, p: PlayerId, name: &str) -> sim_core::ids::CardId {
    let c = card(g, name);
    g.player_mut(p).gy.push(c);
    c
}

fn cast_card_ctx(g: &mut Game, p: PlayerId, name: &str, ctx: Ctx) {
    let c = hand(g, p, &[name])[0];
    cast_card(g, p, c, "hand", ctx).unwrap();
}

fn cast_cast() -> F {
    act(json!({"do": "cast", "card": 0}))
}

// ------------------------------------------------------------------ Spells
#[test]
fn crackle_with_power_x_two() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    hand(&mut g, P0, &["Crackle with Power"]);
    pool(&mut g, P0, &[('R', 2), ('C', 6)]);
    main(&mut g, P0, vec![cast_cast(), by_text("X = 2"), by_text("Sephiroth"), by_text("Sauron")]);
    assert_eq!((g.player(P1).life, g.player(P2).life), (30, 30));
}

#[test]
fn crackle_one_target_is_enough() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    hand(&mut g, P0, &["Crackle with Power"]);
    pool(&mut g, P0, &[('R', 2), ('C', 6)]);
    main(&mut g, P0, vec![cast_cast(), by_text("X = 2"), by_text("Sephiroth"), by_text("no more")]);
    assert_eq!((g.player(P1).life, g.player(P2).life), (30, 40));
}

#[test]
fn mizzixs_mastery_your_target() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = table(&["veyran", "seph"]);
    hand(&mut g, P0, &["Mizzix's Mastery"]);
    let qs = gy_push(&mut g, P0, "Quick Study");
    let bolt = gy_push(&mut g, P0, "Lightning Bolt");
    pool(&mut g, P0, &[('R', 1), ('C', 3)]);
    let n = g.player(P0).hand.len();
    main(&mut g, P0, vec![cast_cast(), by_text("one target"), by_text("Quick Study")]);
    assert_eq!(g.player(P0).hand.len(), n - 1 + 2);
    assert!(g.player(P0).exile.contains(&qs));
    assert!(g.player(P0).gy.contains(&bolt));
}

#[test]
fn flashback_grants_flashback() {
    let mut g = table(&["veyran", "seph"]);
    hand(&mut g, P0, &["Flashback"]);
    let qs = gy_push(&mut g, P0, "Quick Study");
    pool(&mut g, P0, &[('R', 1), ('U', 1), ('C', 2)]);
    let n = g.player(P0).hand.len();
    main(&mut g, P0, vec![cast_cast(), pick(0), act(json!({"do": "cast", "zone": "gy", "card": 0}))]);
    assert_eq!(g.player(P0).hand.len(), n - 1 + 2);
    assert!(g.player(P0).exile.contains(&qs));
}

#[test]
fn jeskas_will_mana() {
    let mut g = table(&["veyran", "seph"]);
    hand(&mut g, P0, &["Jeska's Will"]);
    hand(&mut g, P1, &["Ephemerate", "Necromancy", "Entomb"]);
    pool(&mut g, P0, &[('R', 1), ('C', 2)]);
    main(&mut g, P0, vec![cast_cast(), by_text("add {R}"), pick(0)]);
    assert_eq!(pool_text(&g, P0), "{R}{R}{R}"); // three red
}

#[test]
fn jeskas_will_exile_does_not_play_a_land_for_you() {
    let mut g = table(&["veyran", "seph"]);
    hand(&mut g, P0, &["Jeska's Will"]);
    pool(&mut g, P0, &[('R', 1), ('C', 2)]);
    let isl = card(&g, "Island");
    g.player_mut(P0).library.extend([isl; 3]);
    main(&mut g, P0, vec![cast_cast(), by_text("exile")]);
    assert_eq!(g.player(P0).lands.len(), 0);
    assert_eq!(g.player(P0).hand.iter().filter(|&&c| c == isl).count(), 3);
}

#[test]
fn expressive_iteration() {
    let mut g = table(&["veyran", "seph"]);
    hand(&mut g, P0, &["Expressive Iteration"]);
    pool(&mut g, P0, &[('U', 1), ('R', 1)]);
    let lib = g.player(P0).library.clone();
    let top3 = lib[lib.len() - 3..].to_vec();
    main(&mut g, P0, vec![cast_cast(), pick(0), pick(0)]);
    assert!(g.player(P0).hand.contains(&top3[2]));
    assert!(g.player(P0).impulse.contains(&top3[1]));
    assert_eq!(g.player(P0).library[0], top3[0]);
}

#[test]
fn stock_up() {
    let mut g = table(&["veyran", "seph"]);
    hand(&mut g, P0, &["Stock Up"]);
    pool(&mut g, P0, &[('U', 1), ('C', 2)]);
    let n = g.player(P0).hand.len();
    main(&mut g, P0, vec![cast_cast(), pick(0), pick(0)]);
    assert_eq!(g.player(P0).hand.len(), n - 1 + 2);
}

#[test]
fn prismari_charm_two_pings() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    hand(&mut g, P0, &["Prismari Charm"]);
    pool(&mut g, P0, &[('U', 1), ('R', 1)]);
    main(&mut g, P0, vec![cast_cast(), by_text("1 damage"), by_text("Sephiroth"), by_text("Sauron")]);
    assert_eq!((g.player(P1).life, g.player(P2).life), (39, 39));
}

#[test]
fn thousand_year_storm_copies_take_new_targets() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    perm(&mut g, P0, "Thousand-Year Storm");
    hand(&mut g, P0, &["Quick Study", "Lightning Bolt"]);
    pool(&mut g, P0, &[('U', 1), ('C', 2), ('R', 1)]);
    main(&mut g, P0, vec![cast_cast(), cast_cast(), by_text("Sephiroth"), by_text("Sauron")]);
    assert_eq!((g.player(P1).life, g.player(P2).life), (37, 37));
}

// ------------------------------------------------------------------ Alania
// Alania, Divergent Storm: your first instant and first sorcery each turn, for an opponent's card per copy
fn bolt(g: &mut Game, v: PlayerId, target: PlayerId) {
    cast_card_ctx(g, v, "Lightning Bolt", Ctx { face: Some(target), ..Default::default() });
}

fn opp_cards(g: &Game) -> usize {
    g.player(P1).hand.len() + g.player(P2).hand.len()
}

#[test]
fn the_first_instant_is_copied_once() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    perm(&mut g, P0, "Alania, Divergent Storm");
    let before = opp_cards(&g);
    bolt(&mut g, P0, P1);
    assert_eq!(opp_cards(&g), before + 1); // one opponent drew for the copy
    assert_eq!(g.player(P1).life + g.player(P2).life, 80 - 6); // the Bolt and its copy
    cast_card_ctx(&mut g, P0, "Burst Lightning", Ctx { face: Some(P1), ..Default::default() }); // not the first instant
    assert_eq!(opp_cards(&g), before + 1);
}

#[test]
fn veyran_and_harmonic_prodigy_each_add_a_copy() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    for n in ["Alania, Divergent Storm", "Veyran, Voice of Duality", "Harmonic Prodigy"] {
        perm(&mut g, P0, n);
    }
    let before = opp_cards(&g);
    bolt(&mut g, P0, P1);
    assert_eq!(opp_cards(&g), before + 3);
}

#[test]
fn the_first_sorcery_too_and_not_a_counterspell() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    perm(&mut g, P0, "Alania, Divergent Storm");
    let before = opp_cards(&g);
    cast_card_ctx(&mut g, P0, "Stock Up", Ctx::default());
    assert_eq!(opp_cards(&g), before + 1);
    let c = hand(&mut g, P0, &["Counterspell"])[0];
    g.log = Some(vec![]);
    on_cast(&mut g, P0, c).unwrap(); // the first instant: a counterspell
    assert!(!g.log.as_ref().unwrap().iter().any(|x| x.contains("Alania")));
}

#[test]
fn a_spell_before_alania_counts() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    bolt(&mut g, P0, P1);
    perm(&mut g, P0, "Alania, Divergent Storm");
    let before = opp_cards(&g);
    cast_card_ctx(&mut g, P0, "Burst Lightning", Ctx { face: Some(P1), ..Default::default() });
    assert_eq!(opp_cards(&g), before);
}

#[test]
fn you_choose_who_draws_or_no_copy() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    for n in ["Alania, Divergent Storm", "Veyran, Voice of Duality"] {
        perm(&mut g, P0, n);
    }
    hand(&mut g, P0, &["Lightning Bolt"]);
    pool(&mut g, P0, &[('R', 1)]);
    let (ns, no) = (g.player(P1).hand.len(), g.player(P2).hand.len());
    main(
        &mut g,
        P0,
        vec![
            cast_cast(),
            by_text("Sephiroth"), // the Bolt's target
            by_text("Sauron"),
            by_text("Sephiroth"), // first trigger: Sauron draws; the copy hits Sephiroth
            cancel(),             // second trigger: no copy
        ],
    );
    assert_eq!((g.player(P1).hand.len(), g.player(P2).hand.len()), (ns, no + 1));
    assert_eq!(g.player(P1).life, 40 - 6);
}
