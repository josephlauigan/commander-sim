//! Practice mode: when an opponent's card affects the person, they make the choice (the AI's choices for AI seats
//! are unchanged and tested by the sims).

mod play_common;
use play_common::*;
use sim_core::engine::cast;
use sim_core::state::Ctx;

fn resolve(g: &mut Game, p: PlayerId, name: &str, x: i32) {
    let c = take(g, p, name);
    cast::resolve(g, p, c, &Ctx { x, ..Ctx::default() }, "hand".into()).unwrap();
}

#[test]
fn an_edict_lets_you_pick_the_creature() {
    let mut g = table(&["seph", "sephfs"]);
    let weak = token(&mut g, P1, 1);
    let strong = token(&mut g, P1, 6);
    let h = seat(&mut g, P1, vec![by_text("6/")]);
    perm(&mut g, P0, "Plaguecrafter");
    assert!(h.asked().iter().any(|r| r.prompt.starts_with("Plaguecrafter:")), "you were asked");
    assert!(!g.perm(strong).on_bf, "your choice died");
    assert!(g.perm(weak).on_bf, "the AI's pick (the weakest) was not used");
}

#[test]
fn fact_or_fiction_you_cast_it_and_pick_a_pile() {
    let mut g = table(&["seph", "sephfs"]);
    let h = seat(&mut g, P0, vec![pick(1)]);
    let (hand0, gy0) = (g.player(P0).hand.len(), g.player(P0).gy.len());
    resolve(&mut g, P0, "Fact or Fiction", 0);
    let asked = h.asked();
    let r = asked.iter().find(|r| r.prompt.starts_with("Fact or Fiction")).expect("asked");
    assert_eq!(r.choices.len(), 2, "two piles");
    assert!(r.choices[0].starts_with("pile 1: ") && r.choices[1].starts_with("pile 2: "));
    let taken = g.player(P0).hand.len() - hand0;
    assert_eq!(taken + (g.player(P0).gy.len() - gy0 - 1), 5, "five cards split (minus the spell in the graveyard)");
}

#[test]
fn fact_or_fiction_you_split_the_five() {
    let mut g = table(&["seph", "sephfs"]);
    // the AI casts it; you divide the cards: the first card alone, then done
    let h = seat(&mut g, P1, vec![pick(0), cancel()]);
    let (hand0, gy0) = (g.player(P0).hand.len(), g.player(P0).gy.len());
    resolve(&mut g, P0, "Fact or Fiction", 0);
    let asked = h.asked();
    assert!(asked.iter().all(|r| r.prompt.starts_with("Fact or Fiction")), "{asked:?}");
    assert_eq!(asked.len(), 2);
    assert!(asked[0].prompt.contains("pile 1 now: empty"));
    assert!(asked[1].prompt.contains("pile 1 now: "), "the pile so far is shown by name");
    assert_eq!(asked[0].choices.len(), 6, "five cards and done");
    let kept = g.player(P0).hand.len() - hand0;
    assert!(kept == 1 || kept == 4, "the AI took one of your two piles ({kept})");
    assert_eq!(kept + g.player(P0).gy.len() - gy0 - 1, 5);
}

#[test]
fn torment_of_hailfire_you_choose_each_time() {
    let mut g = table(&["sauron", "seph"]);
    hand(&mut g, P1, &["Sol Ring", "Opt"]);
    let (life0, hand0) = (g.player(P1).life, g.player(P1).hand.len());
    let h = seat(&mut g, P1, vec![by_text("lose 3"), by_text("discard"), pick(0)]);
    resolve(&mut g, P0, "Torment of Hailfire", 2);
    assert_eq!(
        h.asked().iter().filter(|r| r.prompt.starts_with("Torment of Hailfire")).count(),
        3,
        "two asks and a discard pick"
    );
    assert_eq!(g.player(P1).life, life0 - 3, "the first round: life");
    assert_eq!(g.player(P1).hand.len(), hand0 - 1, "the second round: a discard");
}

#[test]
fn esper_sentinel_you_decide_whether_to_pay() {
    for pays in [true, false] {
        let mut g = table(&["yshtola", "seph"]);
        perm(&mut g, P0, "Esper Sentinel");
        lands(&mut g, P1, "Island", 3, false);
        let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r: &Request| {
            if r.prompt.contains("has a trigger") {
                Answer::Action(json!({"do": "pass"}))
            } else {
                Answer::Pick(if pays { 0 } else { 1 })
            }
        }));
        g.humans.add(P1, h.clone());
        let hand0 = g.player(P0).hand.len();
        let c = take(&mut g, P1, "Opt");
        g.player_mut(P1).hand.push(c);
        cast::on_cast(&mut g, P1, c).unwrap();
        assert!(h.asked().iter().any(|r| r.prompt.starts_with("Esper Sentinel")), "you were asked");
        assert_eq!(g.player(P0).hand.len(), hand0 + if pays { 0 } else { 1 }, "pays: {pays}");
    }
}
