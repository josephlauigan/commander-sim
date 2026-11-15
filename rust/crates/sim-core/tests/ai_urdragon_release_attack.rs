//! The Ur-Dragon seat's Twinflame lethal release commits the turn's first combat: after the cast, the AI attacks with
//! everything at the opponent the engine check proved, and not at whoever the defender heuristic would pick.
//! The table: seat 0 (urdragon) with Twinflame in hand, three Mountains and an untapped Ancient Silver Dragon (8/8
//! flier), so the cast makes one hasty copy and 16 flying damage. Seat 1 (seph) is at 16 life: the check proves it.
//! Seat 2 (aragorn) is at 20 life: the heuristic's kill bonus reaches it too (16 is at least 80% of 20), and a huge
//! grudge makes it the defender the heuristic prefers. The look-ahead is off (the test game's default).
mod play_common;
use play_common::*;
use sim_core::ai::act::perform;
use sim_core::engine::{combat, turn};
use sim_core::state::Step;

const GARTH: &str = "Garth One-Eye";
const SPARK: &str = "Spark Double";
const SEASON: &str = "Doubling Season";
const TWIN: &str = "Twinflame";
const BIG: &str = "Ancient Silver Dragon";
const Q: PlayerId = P1; // the opponent the check proves
const OTHER: PlayerId = P2; // the opponent the heuristic prefers

/// seat 0 = urdragon with the Garth line alive (the three pieces in the library), Twinflame in hand, three Mountains
/// and the big flier; seats 1 and 2 at 16 life, seat 2 the heuristic's favourite
fn base(seed: u64) -> Game {
    let mut g = table_with(&["urdragon", "seph", "aragorn"], seed, 40);
    for p in &mut g.players {
        p.hand.clear();
        p.gy.clear();
    }
    for name in [GARTH, SPARK, SEASON] {
        let c = take(&mut g, P0, name);
        g.player_mut(P0).library.push(c);
    }
    hand(&mut g, P0, &[TWIN]);
    lands(&mut g, P0, "Mountain", 3, false);
    perm(&mut g, P0, BIG);
    g.player_mut(Q).life = 16;
    g.player_mut(OTHER).life = 16;
    g.player_mut(P0).grudge.insert("aragorn", 1000.0);
    g
}

/// the Twinflame option the AI is shown in its first main phase
fn twin_option(g: &mut Game) -> sim_core::hooks::Opt {
    let opts = sim_core::ai::brain::main_options(g, P0, false).unwrap();
    opts.into_iter().find(|o| o.label.starts_with("Twinflame (")).expect("the release option")
}

/// the real cast of the offered option
fn cast_release(g: &mut Game) {
    let o = twin_option(g);
    assert!(perform(g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(g.player(P0).gy.contains(&card(g, TWIN)), "Twinflame resolved");
}

fn dead(g: &Game, q: PlayerId) -> bool {
    !g.player(q).alive || g.player(q).life <= 0
}

/// (a) After a real release cast the combat attacks the proven opponent with everything: the one the heuristic would
/// pick (a grudge) must stay alive, and the proven one must die.
#[test]
fn release_cast_then_combat_attacks_the_proven_opponent_and_kills_it() {
    for seed in 1..=6 {
        let mut g = base(seed);
        cast_release(&mut g);
        turn::end_step_pools(&mut g);
        g.step = Step::Combat;
        combat::combat(&mut g, P0).unwrap();
        assert!(dead(&g, Q), "seed {seed}: the proven opponent survived the combat (life {})", g.player(Q).life);
        assert!(!dead(&g, OTHER), "seed {seed}: the other opponent was hit instead");
    }
}

/// (b) `ai::choose_attack` hands the first combat the proven plan right after the cast.
#[test]
fn choose_attack_returns_all_at_the_proven_opponent_after_a_release_cast() {
    let mut g = base(1);
    cast_release(&mut g);
    assert_eq!(sim_core::ai::choose_attack(&mut g, P0).unwrap(), Some((Q, "all")));
}

/// (c) End to end: a whole turn of the AI from the first main phase. The release is cast and the proven opponent is
/// dead before the turn is over.
#[test]
fn a_whole_turn_with_the_release_kills_the_proven_opponent() {
    for seed in 1..=6 {
        let mut g = base(seed);
        turn::continue_turn(&mut g, P0, Step::Main1).unwrap();
        assert!(g.player(P0).gy.contains(&card(&g, TWIN)), "seed {seed}: Twinflame was cast");
        assert!(dead(&g, Q), "seed {seed}: the proven opponent is alive at the end of the turn");
    }
}

// ------------------------------------------------------------------ guards (coverage: they pass on the unfixed code)
// Each guard test is proved not vacuous by a mutation of the code it guards (see the ledger).

fn turns(g: &Game, p: PlayerId) -> u32 {
    g.player(p).turns
}

fn copies(g: &Game) -> usize {
    g.player(P0).perms.iter().filter(|&&m| g.perm(m).token && g.perm(m).name == BIG).count()
}

/// the cast sets the plan, keyed to the seat, its turn count and the proven opponent
#[test]
fn the_cast_sets_the_plan_keyed_to_seat_turn_and_opponent() {
    let mut g = base(1);
    assert_eq!(g.release_attack, None);
    cast_release(&mut g);
    assert_eq!(copies(&g), 1);
    assert_eq!(g.release_attack, Some((P0, turns(&g, P0), Q)));
}

/// a combat that asks for no plan: the plan ends with the combat (`continue_turn` clears it) and nobody is attacked
fn no_attack_ends_the_plan(case: &str, stop: impl FnOnce(&mut Game)) {
    let mut g = base(1);
    cast_release(&mut g);
    assert!(g.release_attack.is_some(), "{case}: the cast set the plan");
    stop(&mut g);
    turn::continue_turn(&mut g, P0, Step::Combat).unwrap();
    assert_eq!(g.release_attack, None, "{case}: the plan outlived the combat");
    assert!(!dead(&g, Q) && !dead(&g, OTHER), "{case}: nobody was attacked");
}

#[test]
fn a_combat_with_no_attackers_clears_the_plan() {
    no_attack_ends_the_plan("tapped", |g| {
        for m in g.player(P0).perms.clone() {
            g.perm_mut(m).tapped = true;
        }
    });
}

#[test]
fn glacial_chasm_clears_the_plan() {
    no_attack_ends_the_plan("chasm", |g| {
        lands(g, P0, "Glacial Chasm", 1, false);
    });
}

#[test]
fn a_kicked_orims_chant_clears_the_plan() {
    no_attack_ends_the_plan("chant", |g| {
        let st = g.turn_stamp();
        g.player_mut(P1).flag_turn.insert("stax_no_attack", st);
    });
}

/// an extra turn in the same round has the same round and seat but a new turn count: the old plan is not used
#[test]
fn an_extra_turn_in_the_same_round_does_not_inherit_the_plan() {
    let mut g = base(1);
    cast_release(&mut g);
    g.player_mut(P0).turns += 1;
    assert_eq!(sim_core::ai::choose_attack(&mut g, P0).unwrap(), None);
    assert_eq!(g.release_attack, None);
}

/// another seat of the same deck never takes a plan that is not its own (it could kill the same opponent)
#[test]
fn a_second_urdragon_seat_does_not_take_the_first_seats_plan() {
    let mut g = base(1);
    g.player_mut(P2).key = "urdragon";
    cast_release(&mut g);
    for _ in 0..2 {
        perm(&mut g, P2, BIG); // seat 2 could kill seat 1 itself
    }
    assert_eq!(sim_core::ai::choose_attack(&mut g, P2).unwrap(), None, "seat 2 took seat 0's plan");
}

/// a seat of another deck never takes a plan, even a plan with its own seat and turn count on a board that kills
#[test]
fn a_seat_of_another_deck_does_not_take_a_plan() {
    let mut g = base(1);
    for _ in 0..2 {
        perm(&mut g, P1, BIG);
    }
    for m in g.player(P0).perms.clone() {
        g.perm_mut(m).tapped = true; // no blocker in the air for seat 1's fliers
    }
    g.player_mut(P0).life = 16;
    g.release_attack = Some((P1, turns(&g, P1), P0));
    assert_eq!(sim_core::ai::choose_attack(&mut g, P1).unwrap(), None, "seph took a plan");
}

/// a cast that an opponent counters makes no copies and commits nothing
#[test]
fn a_countered_cast_commits_nothing() {
    let mut seen = 0;
    for seed in 1..=60 {
        let mut g = base(seed);
        hand(&mut g, P1, &["Counterspell", "Counterspell", "Mana Drain"]);
        lands(&mut g, P1, "Island", 4, false);
        let o = twin_option(&mut g);
        assert!(perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
        if copies(&g) == 0 {
            seen += 1;
            assert_eq!(g.release_attack, None, "seed {seed}: a countered cast set the plan");
        }
    }
    assert!(seen > 0, "no seed had the cast countered");
}

/// a release that the cast alone completes (Terror of the Peaks' triggers kill, an untapped flier would block the
/// attack) is not an attack proof: nothing is committed
#[test]
fn a_kill_by_the_cast_alone_commits_nothing() {
    let mut g = base(1);
    perm(&mut g, P0, "Terror of the Peaks");
    perm(&mut g, P1, "Shivan Dragon");
    g.player_mut(Q).life = 5;
    g.player_mut(OTHER).life = 40;
    cast_release(&mut g);
    assert!(dead(&g, Q), "Terror's trigger killed the opponent");
    assert_eq!(g.release_attack, None);
}

/// a cast inside a look-ahead copy (it plays out the option the real game built) never sets the plan; the real game's
/// cast of the same option does
#[test]
fn a_cast_inside_a_look_ahead_copy_sets_no_plan() {
    let mut g = base(1);
    let o = twin_option(&mut g);
    let mut copy = g.clone();
    copy.in_search = true;
    assert!(perform(&mut copy, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(copy.player(P0).gy.contains(&card(&copy, TWIN)));
    assert_eq!(copy.release_attack, None, "a look-ahead copy set the plan");
    assert!(perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(g.release_attack.is_some());
}

/// a person's seat (the AI's option performed for it) sets no plan: the person declares the attack
#[test]
fn a_human_seat_sets_no_plan() {
    let mut g = base(1);
    let o = twin_option(&mut g);
    seat(&mut g, P0, vec![]);
    pool(&mut g, P0, &[('R', 2)]); // a person pays from the pool
    assert!(sim_core::human::is_human(&g, P0));
    assert!(perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(g.player(P0).gy.contains(&card(&g, TWIN)));
    assert_eq!(g.release_attack, None);
}

/// the option's proven-opponent bits do nothing for another deck's Twinflame (the same option performed for another seat)
#[test]
fn another_decks_twinflame_sets_no_plan() {
    let mut g = base(1);
    let o = twin_option(&mut g);
    g.player_mut(P0).key = "aragorn";
    assert!(perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(g.player(P0).gy.contains(&card(&g, TWIN)));
    assert_eq!(g.release_attack, None);
}

/// the opponent died before the combat: the plan is dropped and used up
#[test]
fn a_dead_opponent_drops_the_plan() {
    let mut g = base(1);
    cast_release(&mut g);
    g.player_mut(Q).alive = false;
    g.player_mut(Q).life = 0;
    assert_eq!(sim_core::ai::choose_attack(&mut g, P0).unwrap(), None);
    assert_eq!(g.release_attack, None);
}

/// the opponent gained life before the combat: the estimate on the real board no longer finds the kill, the plan is
/// dropped and the combat chooses as it always did
#[test]
fn life_gained_before_the_combat_drops_the_plan() {
    let mut g = base(1);
    cast_release(&mut g);
    g.player_mut(Q).life = 17;
    assert_eq!(sim_core::ai::choose_attack(&mut g, P0).unwrap(), None);
    assert_eq!(g.release_attack, None);
}

/// the copies left before the combat: 8 flying power against 16 life, the plan is dropped
#[test]
fn copies_removed_before_the_combat_drop_the_plan() {
    let mut g = base(1);
    cast_release(&mut g);
    for m in g.player(P0).perms.clone() {
        if g.perm(m).token {
            g.perm_mut(m).tapped = true; // a copy that can't attack counts for nothing
        }
    }
    assert_eq!(sim_core::ai::choose_attack(&mut g, P0).unwrap(), None);
    assert_eq!(g.release_attack, None);
}

/// the control for the three tests above: nothing changed, the plan is the proven one, and it is used once
#[test]
fn an_unchanged_board_keeps_the_plan_and_uses_it_once() {
    let mut g = base(1);
    cast_release(&mut g);
    assert_eq!(sim_core::ai::choose_attack(&mut g, P0).unwrap(), Some((Q, "all")));
    assert_eq!(g.release_attack, None);
    assert_eq!(sim_core::ai::choose_attack(&mut g, P0).unwrap(), None);
}

/// a copy made after the real cast (the look-ahead's own, or an opponent's response look-ahead) inherits the plan, so
/// it models the committed attack; using it there does not use up the real game's plan
#[test]
fn a_copy_made_after_the_cast_inherits_the_plan_and_the_real_game_keeps_its_own() {
    let mut g = base(1);
    cast_release(&mut g);
    let mut copy = sim_core::ai::search::clone(&mut g);
    assert_eq!(copy.release_attack, g.release_attack);
    assert_eq!(sim_core::ai::choose_attack(&mut copy, P0).unwrap(), Some((Q, "all")));
    assert_eq!(g.release_attack, Some((P0, turns(&g, P0), Q)));
}
