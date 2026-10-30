//! Why casual games ran long (rust/agents/casual-speed.md): the fixes, each on a hand-built position.

use sim_core::engine::turn::play_land;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn land_names(g: &Game, p: PlayerId) -> Vec<String> {
    g.player(p).lands.iter().map(|&l| g.db.get(g.perm(l).card()).name.to_string()).collect()
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

/// Selesnya Sanctuary: "When this land enters, return a land you control to its owner's hand." With no other land
/// it returns itself, so a first land drop of a bounceland is lost (and was lost every turn: the AI chose it again
/// over the Forest in hand, 0 lands on turn 5 in 13 of 100 Tier 0 games). The Forest goes first.
#[test]
fn bounceland_waits_for_another_land() {
    let mut g = table(&["anikthea-enduring-enchantments", "giada-calling-all-angels"]);
    hand(&mut g, P0, &["Selesnya Sanctuary", "Forest"]);
    play_land(&mut g, P0).unwrap();
    assert_eq!(land_names(&g, P0), ["Forest"]);
    assert!(in_hand(&g, P0, "Selesnya Sanctuary"));
}

/// with another land out the bounceland is played as before: it returns that land
#[test]
fn bounceland_after_a_land() {
    let mut g = table(&["anikthea-enduring-enchantments", "giada-calling-all-angels"]);
    lands(&mut g, P0, "Plains", 1, false);
    hand(&mut g, P0, &["Selesnya Sanctuary"]);
    play_land(&mut g, P0).unwrap();
    assert_eq!(land_names(&g, P0), ["Selesnya Sanctuary"]);
    assert!(in_hand(&g, P0, "Plains"));
}

/// Lotus Field: "When Lotus Field enters, sacrifice two untapped lands." With no other land it sacrifices itself:
/// the Swamp goes first
#[test]
fn lotus_field_waits_for_two_lands() {
    let mut g = table(&["lord-windgrace-jund-lands", "giada-calling-all-angels"]);
    hand(&mut g, P0, &["Lotus Field", "Swamp"]);
    play_land(&mut g, P0).unwrap();
    assert_eq!(land_names(&g, P0), ["Swamp"]);
    assert!(in_hand(&g, P0, "Lotus Field"));
}

const P1: PlayerId = PlayerId(1);

/// An outside deck's attack filter holds back attackers that would die to a good block (a blocker that kills one and
/// survives). It counted every creature that could block as a good blocker, so three 2/2s stayed home from one 3/3
/// and two 1/1s. The 3/3 stops one of them: the attack goes ahead.
#[test]
fn attack_filter_counts_good_blockers_only() {
    let mut g = table(&["zurgo-mardu-surge", "captain-america-avengers-assemble"]);
    let atk: Vec<_> = (0..3).map(|_| token(&mut g, P0, 2)).collect();
    token(&mut g, P1, 3);
    token(&mut g, P1, 1);
    token(&mut g, P1, 1);
    let out = sim_core::ai::pool::attack_filter(&mut g, P0, atk.clone(), P1);
    assert_eq!(out, atk);
}

/// with as many good blockers as risky attackers, they stay home as before
#[test]
fn attack_filter_holds_into_good_blockers() {
    let mut g = table(&["zurgo-mardu-surge", "captain-america-avengers-assemble"]);
    let atk: Vec<_> = (0..2).map(|_| token(&mut g, P0, 2)).collect();
    token(&mut g, P1, 3);
    token(&mut g, P1, 3);
    let out = sim_core::ai::pool::attack_filter(&mut g, P0, atk, P1);
    assert!(out.is_empty());
}

/// Giada, Font of Hope (2/2 flying, vigilance; {T}: add {W} for Angels) is tagged as a mana creature that stays
/// home, so she never attacked. With vigilance attacking doesn't tap her: she attacks an open board.
#[test]
fn giada_attacks_with_vigilance() {
    let mut g = table(&["giada-calling-all-angels", "captain-america-avengers-assemble"]);
    let giada = perm(&mut g, P0, "Giada, Font of Hope");
    assert!(g.perm(giada).noatk && g.perm(giada).vig);
    sim_core::engine::combat::combat(&mut g, P0).unwrap();
    assert_eq!(g.player(P1).life, 38);
    assert!(!g.perm(giada).tapped);
}

/// The look-ahead's attack plan for an outside deck: "none" (every attacker home) stands only when it beats the best
/// attack by `HOLD_MARGIN` (a rollout average); a near tie goes to the attack
#[test]
fn attack_plan_near_tie_attacks() {
    use sim_core::ai::search::{HOLD_MARGIN, attack_on_near_tie};
    let cands = [(P1, "filtered"), (P1, "all"), (P1, "none")];
    let r = 6.0;
    // none ahead by half a point: attack, with the better of the two attack plans
    assert_eq!(attack_on_near_tie(&cands, &[2.0 * r, 2.4 * r, 2.9 * r], r, 2), 1);
    // none clearly ahead: stays home
    assert_eq!(attack_on_near_tie(&cands, &[2.0 * r, 2.4 * r, (2.4 + HOLD_MARGIN + 0.1) * r], r, 2), 2);
    // an attack plan chosen: unchanged
    assert_eq!(attack_on_near_tie(&cands, &[3.0 * r, 2.4 * r, 2.9 * r], r, 0), 0);
}
