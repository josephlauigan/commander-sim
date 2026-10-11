//! Practice mode: the human seat's main phase (Python's tests/test_play_turn.py). Lands, casting from hand and the
//! command zone, the rules check's reasons. (The whole games played through the human seat by a bot are in
//! practice/tests/play_bot_games.rs: a game needs the practice crate's seating.)

mod play_common;
use play_common::*;
use sim_core::human::{
    human::{apply, human_main},
    legal, mana,
};
use sim_core::state::Step;

fn tap_all(g: &mut Game, p: PlayerId) {
    while let Some(s) = mana::sources(g, p).first() {
        let id = s.id as i64;
        mana::tap(g, p, id, None).unwrap();
    }
}

#[test]
fn one_land_per_turn() {
    let mut g = table(&["sauron", "veyran"]);
    let cs = hand(&mut g, P0, &["Swamp", "Island"]);
    assert_eq!(legal::check_land(&g, P0, cs[0]), None);
    assert_eq!(apply(&mut g, P0, &json!({"do": "land", "card": 0})).unwrap(), None);
    assert_eq!(legal::check_land(&g, P0, cs[1]).as_deref(), Some("You've already played a land this turn."));
}

#[test]
fn lands_and_sorceries_only_in_your_main_phase() {
    let mut g = table(&["sauron", "veyran"]);
    let c = hand(&mut g, P0, &["Swamp"])[0];
    let sheo = hand(&mut g, P0, &["Sheoldred, the Apocalypse"])[0];
    g.step = Step::Combat;
    assert_eq!(legal::check_land(&g, P0, c).as_deref(), Some("You can only play a land in one of your main phases."));
    g.active = Some(P1);
    g.step = Step::Main1;
    assert_eq!(legal::check_cast(&mut g, P0, sheo, "hand").as_deref(), Some("It isn't your turn."));
}

#[test]
fn instants_any_time_you_have_priority() {
    let mut g = table(&["sauron", "veyran"]);
    let cs = hand(&mut g, P0, &["Infernal Grasp"])[0];
    lands(&mut g, P0, "Swamp", 2, false);
    perm(&mut g, P1, "Guttersnipe");
    g.active = Some(P1);
    tap_all(&mut g, P0);
    assert_eq!(legal::check_cast(&mut g, P0, cs, "hand"), None);
}

#[test]
fn the_reason_when_mana_is_short() {
    let mut g = table(&["sauron", "veyran"]);
    let sheo = hand(&mut g, P0, &["Sheoldred, the Apocalypse"])[0];
    lands(&mut g, P0, "Swamp", 2, false);
    tap_all(&mut g, P0);
    assert_eq!(
        legal::check_cast(&mut g, P0, sheo, "hand").as_deref(),
        Some("Can't cast Sheoldred, the Apocalypse. It costs {2}{B}{B}; your mana pool has {B}{B}. Missing {2}.")
    );
}

#[test]
fn commander_tax() {
    let mut g = table(&["sauron", "veyran"]);
    g.player_mut(P0).tax = 2;
    for n in ["Island", "Swamp", "Mountain"] {
        lands(&mut g, P0, n, 2, false);
    }
    tap_all(&mut g, P0);
    let cmd = g.player(P0).cmd;
    let why = legal::check_cast(&mut g, P0, cmd, "cmd").unwrap();
    assert!(why.contains("Missing {2}"), "{why}"); // {3}{U}{B}{R} + 2 tax from 6 mana
    lands(&mut g, P0, "Swamp", 2, false);
    tap_all(&mut g, P0);
    assert_eq!(legal::check_cast(&mut g, P0, cmd, "cmd"), None);
}

#[test]
fn tap_then_cast() {
    let mut g = table(&["sauron", "veyran"]);
    hand(&mut g, P0, &["Orcish Bowmasters"]);
    lands(&mut g, P0, "Swamp", 2, false);
    let tap = || act(json!({"do": "tap", "source": 0}));
    // Bowmasters' 1 damage: at the opponent
    let target: F = Box::new(|r| {
        Answer::Pick(r.choices.iter().position(|x| x.contains("player") && !x.contains("you")).expect("a player"))
    });
    let h = seat(&mut g, P0, vec![tap(), tap(), act(json!({"do": "cast", "card": 0})), target, pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(has_perm(&g, P0, "Orcish Bowmasters"));
    assert_eq!(pool_total(&g, P0), 0);
    assert!(h.told().is_empty(), "{:?}", h.told());
}

#[test]
fn an_illegal_move_is_explained_and_you_keep_priority() {
    let mut g = table(&["sauron", "veyran"]);
    let sheo = hand(&mut g, P0, &["Sheoldred, the Apocalypse"])[0];
    let h = seat(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert_eq!(h.asked().len(), 2);
    assert_eq!(h.told()[0].0, "invalid");
    assert!(h.told()[0].1.contains("Missing {2}{B}{B}"), "{}", h.told()[0].1);
    assert!(g.player(P0).hand.contains(&sheo));
}

#[test]
fn casting_the_commander_raises_the_tax() {
    let mut g = table(&["sauron", "veyran"]);
    for n in ["Island", "Swamp", "Mountain", "Island", "Swamp", "Mountain"] {
        lands(&mut g, P0, n, 1, false);
    }
    let mut a: Vec<F> = (0..6).map(|_| act(json!({"do": "tap", "source": 0}))).collect();
    a.push(act(json!({"do": "cast", "zone": "cmd"})));
    a.push(pass());
    seat(&mut g, P0, a);
    human_main(&mut g, P0, false).unwrap();
    let cmd = g.player(P0).cmd;
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).cd == Some(cmd)));
    assert_eq!(g.player(P0).tax, 2);
    assert!(!g.player(P0).cmd_in_zone);
}
