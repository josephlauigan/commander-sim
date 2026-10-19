//! Practice mode: "you may" triggers ask the person (engine::stack::may_window). Yes applies the effect, no skips it,
//! and for the AI the card code's own judgment decides as before.

mod play_common;
use play_common::*;
use sim_core::engine::{turn, values};
use std::sync::Arc;

/// a person who answers questions whose prompt starts with `start` with `answer` (index into the choices), and
/// passes/declines everything else (priority and the like)
fn answers(g: &mut Game, p: PlayerId, start: &'static str, answer: usize) -> Arc<ScriptHuman> {
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if r.prompt.starts_with(start) { Answer::Pick(answer) } else { Answer::Decline }
    }));
    g.humans.add(p, h.clone());
    h
}

fn asked_with(h: &ScriptHuman, start: &str) -> usize {
    h.asked().iter().filter(|r| r.prompt.starts_with(start)).count()
}

#[test]
fn seymour_flux_yes_pays_life_draws_and_grows() {
    let mut g = table(&["sheoldred", "seph"]);
    let s = perm(&mut g, P0, "Seymour Flux");
    let h = answers(&mut g, P0, "Seymour Flux:", 0);
    let (hand0, p0) = (g.player(P0).hand.len(), values::epow(&g, s));
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(asked_with(&h, "Seymour Flux:"), 1, "you were asked");
    assert_eq!(g.player(P0).life, 39);
    assert_eq!(g.player(P0).hand.len(), hand0 + 1);
    assert_eq!(values::epow(&g, s), p0 + 1);
}

#[test]
fn seymour_flux_no_skips_it() {
    let mut g = table(&["sheoldred", "seph"]);
    let s = perm(&mut g, P0, "Seymour Flux");
    let h = answers(&mut g, P0, "Seymour Flux:", 1);
    let (hand0, p0) = (g.player(P0).hand.len(), values::epow(&g, s));
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(asked_with(&h, "Seymour Flux:"), 1);
    assert_eq!((g.player(P0).life, g.player(P0).hand.len(), values::epow(&g, s)), (40, hand0, p0));
}

#[test]
fn seymour_flux_asks_a_person_even_at_low_life_where_the_ai_never_pays() {
    let mut g = table_with(&["sheoldred", "seph"], 1, 6);
    perm(&mut g, P0, "Seymour Flux");
    let h = answers(&mut g, P0, "Seymour Flux:", 0);
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(asked_with(&h, "Seymour Flux:"), 1);
    assert_eq!(g.player(P0).life, 5);
}

#[test]
fn the_ai_is_not_asked_and_keeps_its_own_rule() {
    // no person seated: at 8 life or less Seymour Flux is not paid for (the AI's rule); above it, it is
    let mut g = table_with(&["sheoldred", "seph"], 1, 8);
    perm(&mut g, P0, "Seymour Flux");
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).life, 8, "AI at 8 life does not pay");
    let mut g = table(&["sheoldred", "seph"]);
    perm(&mut g, P0, "Seymour Flux");
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).life, 39, "AI at 40 life pays");
}

#[test]
fn mana_vault_untaps_only_if_you_pay_four() {
    for (answer, tapped_after) in [(0, false), (1, true)] {
        let mut g = table(&["godo", "seph"]);
        let v = perm(&mut g, P0, "Mana Vault");
        g.perm_mut(v).tapped = true;
        lands(&mut g, P0, "Mountain", 5, false);
        let h = answers(&mut g, P0, "Mana Vault", answer);
        turn::upkeep(&mut g, P0).unwrap();
        assert_eq!(asked_with(&h, "Mana Vault"), 1, "you were asked");
        assert_eq!(g.perm(v).tapped, tapped_after);
    }
}

#[test]
fn stadium_tidalmage_loots_only_on_yes() {
    for (answer, drawn) in [(0usize, true), (1, false)] {
        let mut g = table(&["kefka", "seph"]);
        let h = answers(&mut g, P0, "Stadium Tidalmage", answer);
        let lib0 = g.player(P0).library.len();
        perm(&mut g, P0, "Stadium Tidalmage");
        assert!(asked_with(&h, "Stadium Tidalmage") >= 1, "you were asked");
        assert_eq!(g.player(P0).library.len(), lib0 - 1 - usize::from(drawn), "the Tidalmage left the library; yes draws one more");
    }
}

#[test]
fn conjurer_s_closet_blinks_the_creature_you_pick_or_nothing() {
    // blinking a Stadium Tidalmage asks its own loot question again, so the count of its asks tells whether it blinked
    for (answer, blinked) in [(0usize, true), (1, false)] {
        let mut g = table(&["seph", "sheoldred"]);
        perm(&mut g, P0, "Conjurer's Closet");
        perm(&mut g, P0, "Stadium Tidalmage");
        let h = answers(&mut g, P0, "Conjurer's Closet:", answer);
        let before = h.asked().len();
        let closet = on_bf_perm(&g, "Conjurer's Closet");
        sim_core::impls::t2::end_step_flicker(&mut g, closet, P0, false, false).unwrap();
        let qs: Vec<String> = h.asked()[before..].iter().map(|r| r.prompt.clone()).collect();
        assert!(qs.iter().any(|q| q.starts_with("Conjurer's Closet:")), "asked: {qs:?}");
        assert_eq!(qs.iter().any(|q| q.starts_with("Stadium Tidalmage")), blinked, "{qs:?}");
    }
}

fn on_bf_perm(g: &Game, name: &str) -> PermId {
    g.players[0].perms.iter().copied().find(|&m| &*g.perm(m).name == name).unwrap()
}

#[test]
fn rhystic_study_asks_its_owner_before_drawing() {
    for (answer, drew) in [(0usize, true), (1, false)] {
        let mut g = table(&["yshtola", "seph"]);
        perm(&mut g, P0, "Rhystic Study");
        let h = answers(&mut g, P0, "Rhystic Study:", answer);
        let hand0 = g.player(P0).hand.len();
        let c = take(&mut g, P1, "Sol Ring");
        sim_core::engine::cast::on_cast(&mut g, P1, c).unwrap();
        assert_eq!(asked_with(&h, "Rhystic Study:"), 1, "the owner was asked");
        // the AI opponent pays {1} when it can spare it; with no mana it cannot, so the owner draws on yes
        assert_eq!(g.player(P0).hand.len() == hand0 + 1, drew);
    }
}

