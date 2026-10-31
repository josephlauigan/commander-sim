//! phase4-search-b: blocks, protection and wipe responses as searched decisions (`audit/phase4/search-design.md`,
//! "Part (b)").

use sim_core::ai::{self, ismcts};
use sim_core::ids::{PermId, PlayerId};
use sim_core::settings::AiMode;
use sim_core::state::{Ctx, Game, SearchCost, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P3: PlayerId = PlayerId(3);

/// Krenko (P0) against Edgar Markov (P3, a generic-policy deck) alone; both on the look-ahead with ISMCTS and part
/// (b)'s searches forced on; P0's turn, `step`
fn duel(step: Step) -> Game {
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
    for p in [P0, P3] {
        g.player_mut(p).hand.clear();
    }
    lands(&mut g, P0, "Mountain", 4, false);
    lands(&mut g, P3, "Swamp", 4, false);
    g.player_mut(P0).turns = 5;
    g.player_mut(P3).turns = 4;
    g.round = 5;
    g.active = Some(P0);
    g.step = step;
    ismcts::force(Some(true));
    ismcts::force_b(Some(true));
    g
}

/// a vanilla token of P/T (pow, tgh)
fn body(g: &mut Game, p: PlayerId, pow: i32, tgh: i32) -> PermId {
    let t = token(g, p, pow.max(1));
    g.perm_mut(t).pow = pow;
    g.perm_mut(t).tgh = tgh;
    t
}

/// five 1/1 attackers at P3 on 5 life, who has one 0/4: the heuristic never chumps a 1-power attacker (and a 0/4
/// neither kills nor trades), so its blocks let lethal through
fn swarm() -> (Game, Vec<PermId>, Vec<(PermId, PermId)>) {
    let mut g = duel(Step::Combat);
    g.player_mut(P3).life = 5;
    body(&mut g, P3, 0, 4);
    let atk: Vec<PermId> = (0..5).map(|_| body(&mut g, P0, 1, 1)).collect();
    for &a in &atk {
        g.perm_mut(a).tapped = true;
    }
    let mut rng = sim_core::rng::Rng::from_u64(1);
    let heur = sim_core::engine::combat::ai_blocks(&g, P0, &atk, P3, &[], &mut rng);
    (g, atk, heur)
}

/// The search finds the chump that prevents lethal, which the heuristic misses
#[test]
fn a_chump_block_that_prevents_lethal() {
    let (mut g, atk, heur) = swarm();
    assert!(heur.is_empty(), "the heuristic blocks nothing: {heur:?}");
    let plan = ismcts::choose_blocks(&mut g, P0, &atk, P3, &[], &heur).expect("searched");
    assert_eq!(plan.len(), 1, "{plan:?}");
    assert!(ismcts::unblocked_damage(&g, &atk, &plan) < g.player(P3).life);
    assert_eq!(g.player(P3).search_cost.n[SearchCost::BLOCK], 1);
}

/// Same seed, same choice; and the budget is respected
#[test]
fn blocks_same_seed_same_choice_within_budget() {
    let (g, atk, heur) = swarm();
    let run = |budget: u64| {
        let mut k = ismcts::knobs();
        k.budget = budget;
        ismcts::set_knobs(Some(k));
        let mut g2 = g.clone();
        let plan = ismcts::choose_blocks(&mut g2, P0, &atk, P3, &[], &heur);
        ismcts::set_knobs(None);
        (plan, g2.search_work)
    };
    let a = run(ismcts::BUDGET);
    assert!(a.1 > 0);
    assert_eq!(a, run(ismcts::BUDGET));
    let small = run(200);
    assert!(small.1 <= 200 + g.settings.search.playout_work, "{} steps", small.1);
}

/// A small attack (below half the defender's life) is left to the heuristic
#[test]
fn a_small_attack_is_not_searched() {
    let (mut g, atk, heur) = swarm();
    g.player_mut(P3).life = 40;
    assert!(ismcts::choose_blocks(&mut g, P0, &atk, P3, &[], &heur).is_none());
    assert_eq!(g.player(P3).search_cost.policy[SearchCost::BLOCK], 1);
}

/// P3 holds Heroic Intervention with mana for it, and has `body` on the battlefield beside a 1/1; P0 is at 8 life
/// with no creatures. Returns (game, P3's big creature, its 1/1).
fn protection(pow: i32) -> (Game, PermId, PermId) {
    let mut g = duel(Step::Main1);
    lands(&mut g, P3, "Forest", 2, false);
    hand(&mut g, P3, &["Heroic Intervention"]);
    g.player_mut(P0).life = 8;
    let big = body(&mut g, P3, pow, pow);
    let small = body(&mut g, P3, 1, 1);
    (g, big, small)
}

fn has_intervention(g: &Game) -> bool {
    g.player(P3).hand.iter().any(|&c| &*g.db.get(c).name == "Heroic Intervention")
}

/// Protection is used on the permanent whose loss swings the game (a 10/10 that kills P0 next turn)
#[test]
fn protection_saves_the_game_winning_permanent() {
    let (mut g, big, _) = protection(10);
    assert!(ai::protect_response(&mut g, P3, big, "destroy", Some(P0), None).unwrap());
    assert!(g.perm(big).on_bf);
    assert!(!has_intervention(&g));
    assert_eq!(g.player(P3).search_cost.n[SearchCost::PROTECT], 1);
}

/// The protection spell is held when the removal is aimed at a low-value target (a 1/1)
#[test]
fn protection_is_held_for_a_low_value_target() {
    let (mut g, _, small) = protection(10);
    assert!(!ai::protect_response(&mut g, P3, small, "destroy", Some(P0), None).unwrap());
    assert!(has_intervention(&g));
    assert_eq!(g.player(P3).search_cost.n[SearchCost::PROTECT], 1);
}

/// Same seed, same choice for protection
#[test]
fn protection_same_seed_same_choice() {
    let (g, big, _) = protection(10);
    let run = || {
        let mut g2 = g.clone();
        let r = ai::protect_response(&mut g2, P3, big, "destroy", Some(P0), None).unwrap();
        (r, g2.search_work, has_intervention(&g2))
    };
    let a = run();
    assert!(a.1 > 0);
    assert_eq!(a, run());
}

/// A wipe response that saves the key permanent: P0 casts a destroy-all wipe while P3's 10/10 would kill P0 next
/// turn; P3 answers with Teferi's Protection
#[test]
fn a_wipe_response_saves_the_key_permanent() {
    let mut g = duel(Step::Main1);
    lands(&mut g, P3, "Plains", 3, false);
    hand(&mut g, P3, &["Teferi's Protection"]);
    g.player_mut(P0).life = 8;
    let big = perm(&mut g, P3, "Craterhoof Behemoth");
    g.perm_mut(big).pow = 10;
    g.perm_mut(big).tgh = 10;
    perm(&mut g, P3, "Hellrider");
    perm(&mut g, P3, "Siege-Gang Commander");
    sim_core::engine::removal::apply_wipe(&mut g, P0, "destroy", None, &Ctx::default()).unwrap();
    let c = g.player(P3).search_cost;
    assert!(g.perm(big).on_bf, "the 10/10 survives (phased out): {c:?} {:?}", g.player(P3).hand);
    assert_eq!(g.player(P3).search_cost.n[SearchCost::WIPE], 1);
}
