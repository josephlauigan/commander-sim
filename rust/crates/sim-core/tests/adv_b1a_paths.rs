// written by an independent reviewer
//! Adversarial tests for B1a, attack 4: `when` semantics. (The reviewer's other passing tests, which describe rows
//! `hold_problems` now refuses or which only print scans, and the four tests of removal-mode holds, are not kept.)

use sim_core::ai::brain::{Situation, card_utility, main_options};
use sim_core::ai::plan_data;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;
use std::panic::{AssertUnwindSafe, catch_unwind};

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const FLAWLESS: &str = "Flawless Maneuver";
const YOSHI: &str = "Yoshimaru, Ever Faithful";

struct HoldPlan;
impl HoldPlan {
    fn with(key: &str, text: &str) -> HoldPlan {
        let l = plan_data::load(key, text).unwrap_or_else(|e| panic!("{e}"));
        plan_data::set_hold_plan_for_test(Some(Box::leak(Box::new(l))));
        HoldPlan
    }
}
impl Drop for HoldPlan {
    fn drop(&mut self) {
        plan_data::set_hold_plan_for_test(None);
    }
}

fn row(key: &str, card: &str, when: &str) -> String {
    let w = if when.is_empty() { String::new() } else { format!("when = [{when}]\n") };
    format!("deck = \"{key}\"\n[[hold]]\ncard = {card:?}\nprotects = [\"commander\"]\n{w}")
}

fn colours(g: &mut Game, p: PlayerId, each: usize) {
    for l in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        lands(g, p, l, each, false);
    }
}

/// seat `key` in its first main phase, hands empty, three of each basic untapped, its commander out
fn seat0(key: &str) -> (Game, PermId) {
    let mut g = table(&[key, "seph"]);
    g.active = Some(P0);
    for q in &mut g.players {
        q.hand.clear();
    }
    colours(&mut g, P0, 3);
    let cmd = deck(key).commander.clone();
    let y = perm(&mut g, P0, &cmd);
    g.perm_mut(y).is_cmd = true;
    perm(&mut g, P1, "Grave Titan");
    (g, y)
}

fn utility(g: &Game, c: CardId) -> Option<f64> {
    card_utility(g, P0, &Situation::new(g, P0), c)
}

#[test]
fn when_reads_the_commander_tax_of_the_card_in_the_command_zone() {
    // `card-tax` is the one atom that reads `Cx.card`; hold_problems refuses a commander row, but it loads
    let mut g = table(&["yoshimaru", "seph"]);
    g.active = Some(P0);
    for q in &mut g.players {
        q.hand.clear();
    }
    colours(&mut g, P0, 3);
    let cmd = g.player(P0).cmd;
    let _plan = HoldPlan::with("yoshimaru", &row("yoshimaru", YOSHI, "\"card-tax>=2\""));
    assert!(!plan_data::held_back(&g, P0, cmd), "tax 0: not held");
    g.player_mut(P0).tax = 2;
    assert!(plan_data::held_back(&g, P0, cmd), "tax 2: held");
}

#[test]
fn hand_max_mv_leaves_out_the_card_itself_and_duplicates_are_both_held() {
    let (mut g, _) = seat0("yoshimaru");
    let cs = hand(&mut g, P0, &[FLAWLESS, FLAWLESS]);
    let _plan = HoldPlan::with("yoshimaru", &row("yoshimaru", FLAWLESS, "\"hand-max-mv>=3\""));
    // Flawless Maneuver is mana value 3: with two copies each sees the other's 3
    assert!(cs.iter().all(|&c| plan_data::held_back(&g, P0, c)));
    g.player_mut(P0).hand.pop();
    assert!(!plan_data::held_back(&g, P0, cs[0]), "alone: this copy is left out, nothing else in hand");
}

#[test]
fn several_rows_for_one_card_are_alternatives() {
    let (mut g, _) = seat0("yoshimaru");
    let c = hand(&mut g, P0, &[FLAWLESS])[0];
    let text = format!(
        "deck = \"yoshimaru\"\n[[hold]]\ncard = \"{FLAWLESS}\"\nwhen = [\"lands>=99\"]\n[[hold]]\ncard = \"{FLAWLESS}\"\nwhen = [\"lands>=1\"]\n[[hold]]\ncard = \"{FLAWLESS}\"\nwhen = [\"lands>=98\"]\n"
    );
    let _plan = HoldPlan::with("yoshimaru", &text);
    assert!(plan_data::held_back(&g, P0, c));
    assert_eq!(utility(&g, c), None);
}

/// every atom a `when` may use, evaluated in `held_back`, `card_utility` and `main_options`: none panics or recurses
#[test]
fn every_atom_is_evaluable_in_a_hold_when() {
    let atoms = [
        "commander-out",
        "commander-zone",
        "ninjutsu-window",
        "drain-payoff",
        "to-top",
        "line-ready",
        "commander-indestructible",
        "not commander-out",
        "out:Grave Titan",
        "opp-out:Grave Titan",
        "have:Flawless Maneuver",
        "in-play:Plains",
        "land:Plains",
        "lib:Sol Ring",
        "(lands>=1 | creatures>=1) & turn>=1",
        "hand-gaps>=0",
        "hand-castable>=0",
        "cmd-power>=0",
        "cmd-blockers>=0",
        "cmd-killers>=0",
        "atk-power>=0",
        "def-life>=0",
        "opp-removal-up>=0",
        "card-tax>=0",
        "rocks>=0",
        "hand-size>=0",
        "opp-stop-max>=0",
        "opp-stoppers>=0",
        "opp-open-max>=0",
        "archenemy-max>=0",
        "opp-cards-up>=0",
        "opp-second-spell>=0",
        "hand-blue>=0",
        "counters>=0",
        "mana-perms>=0",
        "creatures>=0",
        "lands>=0",
        "tax>=0",
        "turn>=0",
        "opp-power>=0",
        "life>=0",
        "round>=0",
        "hand-max-mv>=0",
        "hand-lands>=0",
        "hand-b-lands>=0",
        "nonland>=0",
        "tutors>=0",
        "fast>=0",
        "mana>=0",
        "eff>=0",
        "makers>=0",
        "opp-blue-up>=0",
        "mana>=lands",
        "life>=one-lander",
    ];
    let mut bad = vec![];
    for a in atoms {
        let (mut g, _) = seat0("yoshimaru");
        let c = hand(&mut g, P0, &[FLAWLESS, "Sol Ring", "Swords to Plowshares"])[0];
        let text = row("yoshimaru", FLAWLESS, &format!("{a:?}"));
        let Ok(l) = plan_data::load("yoshimaru", &text) else {
            println!("refused at load: {a}");
            continue;
        };
        plan_data::set_hold_plan_for_test(Some(Box::leak(Box::new(l))));
        let r = catch_unwind(AssertUnwindSafe(|| {
            let h = plan_data::held_back(&g, P0, c);
            let u = utility(&g, c);
            let o = main_options(&mut g, P0, false).is_ok();
            (h, u.is_none(), o)
        }));
        plan_data::set_hold_plan_for_test(None);
        match r {
            Ok((h, none, _)) => {
                assert_eq!(h, none, "{a}: held_back and card_utility disagree");
            }
            Err(_) => bad.push(a),
        }
    }
    assert!(bad.is_empty(), "atoms that panic in a hold `when`: {bad:?}");
}
