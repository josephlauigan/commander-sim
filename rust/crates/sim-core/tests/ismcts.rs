//! phase4-search: ISMCTS on the high-stakes decisions of the generic-policy decks (`audit/phase4/search-design.md`).

use sim_core::ai::{brain, ismcts, search};
use sim_core::ids::PlayerId;
use sim_core::settings::AiMode;
use sim_core::state::{Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

/// Krenko (a generic-policy deck) in main 1 against one live opponent at `life` with three 2/2s; Krenko has four
/// 1/1 tokens, six Forests and three Mountains untapped and `cards` in hand
fn krenko_vs_one(life: i32, cards: &[&str]) -> Game {
    let mut g = table(&[
        "krenko-mono-red-goblins",
        "chulane-bant-value-combo",
        "yuriko-dimir-ninjas",
        "edgar-markov-mardu-vampires",
    ]);
    set_ai(&mut g, AiMode::Lookahead);
    for i in [1, 2] {
        g.players[i].alive = false;
    }
    g.players[3].life = life;
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, cards);
    lands(&mut g, P0, "Forest", 6, false);
    lands(&mut g, P0, "Mountain", 3, false);
    for _ in 0..4 {
        token(&mut g, P0, 1);
    }
    for _ in 0..3 {
        token(&mut g, PlayerId(3), 2);
    }
    g.player_mut(P0).turns = 5;
    g.round = 5;
    g.active = Some(P0);
    g.step = Step::Main1;
    g
}

/// the hoof position: Craterhoof Behemoth wins this turn (five creatures, +5/+5, trample, haste: 34 power against
/// 15 life and three 2/2s), and the policy rates Hellrider and Siege-Gang Commander above it
fn hoof() -> (Game, Vec<sim_core::hooks::Opt>) {
    let mut g = krenko_vs_one(15, &["Craterhoof Behemoth", "Hellrider", "Siege-Gang Commander"]);
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    (g, opts)
}

fn knobs(f: impl FnOnce(&mut ismcts::Knobs)) {
    let mut k = ismcts::knobs();
    f(&mut k);
    ismcts::set_knobs(Some(k));
    ismcts::force(Some(true));
}

fn label(opts: &[sim_core::hooks::Opt], i: Option<usize>) -> String {
    opts[i.expect("a pick")].label.clone()
}

fn top(opts: &[sim_core::hooks::Opt]) -> usize {
    (0..opts.len()).fold(0, |b, i| if opts[i].utility > opts[b].utility { i } else { b })
}

/// The tree finds the win this turn that the policy's top play misses
#[test]
fn the_tree_finds_a_forced_win_the_policy_misses() {
    let (mut g, opts) = hoof();
    assert_ne!(opts[top(&opts)].label, "Craterhoof Behemoth", "the policy's top play isn't the win");
    knobs(|k| k.gate = false);
    let i = search::choose(&mut g, P0, false, &opts);
    assert_eq!(label(&opts, i), "Craterhoof Behemoth");
    let scores = search::LAST_SCORES.with(|l| l.borrow().clone());
    assert!(scores.iter().any(|(l, s)| l == "Craterhoof Behemoth" && *s >= 99.0), "{scores:?}");
}

/// Same seed, same choice: the decision's copies come from its seed, so a repeat makes the same pick on the same
/// playout values
#[test]
fn same_seed_same_choice() {
    knobs(|k| k.gate = false);
    let (g, opts) = hoof();
    let run = || {
        let mut g2 = g.clone();
        let i = search::choose(&mut g2, P0, false, &opts);
        (i, g2.search_work, search::LAST_SCORES.with(|l| l.borrow().clone()))
    };
    let a = run();
    assert!(a.1 > 0);
    assert_eq!(a, run());
}

/// The budget is respected: a decision stops once its playouts have taken the budget's steps (it can go over by the
/// last playout only, and a playout is capped at `playout_work`)
#[test]
fn the_budget_is_respected() {
    let (g, opts) = hoof();
    let mut used = vec![];
    for budget in [300, 1_000, 1_000_000] {
        knobs(|k| {
            k.gate = false;
            k.budget = budget;
        });
        let mut g2 = g.clone();
        search::choose(&mut g2, P0, false, &opts);
        assert!(g2.search_work <= budget + g.settings.search.playout_work, "{} over {budget}", g2.search_work);
        used.push(g2.search_work);
    }
    assert!(used[0] < used[1] && used[1] <= used[2], "{used:?}");
}

/// Early stopping fires on a dominant candidate: once every candidate has its first playouts, a candidate clearly
/// ahead of every other (the margin holds against each, on 6 shared copies against the top play) ends the search
/// long before the per-candidate cap
#[test]
fn early_stopping_fires_on_a_dominant_candidate() {
    knobs(|k| {
        k.budget = u64::MAX;
        k.min_visits = 2;
        k.max_visits = 24;
    });
    let mut g = table(&tier("t4")[..4]);
    let keys: Vec<(String, usize)> = (0..5).map(|k| (format!("c{k}"), 0)).collect();
    let noise = |j: u64| [3.0, -2.0, 1.0, -4.0, 2.0][j as usize % 5];
    // candidate 2 is 40 points ahead of the rest on every copy
    let s = ismcts::search_root(&mut g, P0, keys.clone(), Some(0), 2.36, 24, |_, k, j| {
        noise(j) + if k == 2 { 50.0 } else { 10.0 }
    });
    let n: usize = s.vals.iter().map(|v| v.len()).sum();
    assert_eq!(ismcts::pick(&s.vals, Some(0), 2.36, 6), 2);
    assert!(n <= 5 * 6, "{n} playouts");
    // no candidate ahead: the hopeless stop (no gain over the top play) ends it early too, and the top play stays
    let s = ismcts::search_root(&mut g, P0, keys, Some(0), 2.36, 24, |_, k, j| noise(j + k as u64));
    let m: usize = s.vals.iter().map(|v| v.len()).sum();
    assert!(m < 5 * 24, "{m} playouts");
    assert_eq!(ismcts::pick(&s.vals, Some(0), 2.36, 6), 0);
}

/// With little at stake the policy's top play is made without a search, and counted as a policy decision
#[test]
fn a_low_stakes_decision_plays_the_policy() {
    knobs(|k| {
        k.gate = true;
        k.close = 0.0;
    });
    let mut g = krenko_vs_one(40, &["Hellrider", "Siege-Gang Commander"]);
    g.players[3].perms.clear(); // no blockers, nothing to remove
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    let s = ismcts::stakes(&g, P0, &opts);
    assert!(!ismcts::high_stakes_main(&s, &ismcts::knobs()), "{s:?}");
    let w = g.search_work;
    let i = search::choose(&mut g, P0, false, &opts);
    assert_eq!(i, Some(top(&opts)));
    assert_eq!(g.search_work, w, "no playouts");
    assert_eq!(g.player(P0).search_cost.policy[0], 1);
}
