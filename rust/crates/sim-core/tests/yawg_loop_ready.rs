use sim_core::ids::PlayerId;
use sim_core::impls::combos;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const YAWG: &str = "Yawgmoth, Thran Physician";
const MIKAEUS: &str = "Mikaeus, the Unhallowed";

fn ready(g: &Game) -> bool {
    let line = combos::all().iter().find(|c| c.name == "Yawgmoth + undying loop + a death payoff").unwrap();
    (line.ready)(g, P0).0
}

fn mikaeus_board() -> Game {
    let mut g = table(&["sheoldred", "seph"]);
    perm(&mut g, P0, YAWG);
    perm(&mut g, P0, MIKAEUS);
    perm(&mut g, P0, "Gray Merchant of Asphodel");
    perm(&mut g, PlayerId(1), "Viscera Seer"); // preserves the payoff-less board-control line
    g
}

#[test]
fn mikaeus_and_one_other_non_human_cannot_loop() {
    let g = mikaeus_board();
    assert!(!ready(&g), "Mikaeus does not give himself undying; Gray Merchant alone stalls");
}

#[test]
fn mikaeus_and_two_other_non_humans_can_loop() {
    let mut g = mikaeus_board();
    perm(&mut g, P0, "Blood Artist");
    assert!(ready(&g));
}

#[test]
fn a_zombie_token_cannot_be_the_second_undying_creature() {
    let mut g = mikaeus_board();
    let zombie = token(&mut g, P0, 2);
    g.perm_mut(zombie).ttypes.push("zombie".into());
    assert!(!ready(&g), "a token cannot return after it dies");
}

#[test]
fn humans_do_not_gain_undying_from_mikaeus() {
    let mut g = mikaeus_board();
    perm(&mut g, P0, "Zulaport Cutthroat");
    assert!(!ready(&g));
}

#[test]
fn two_intrinsic_undying_creatures_can_loop_without_mikaeus() {
    let mut g = table(&["sheoldred", "seph"]);
    perm(&mut g, P0, YAWG);
    perm(&mut g, P0, "Geralf's Messenger"); // also the payoff
    perm(&mut g, P0, "Butcher Ghoul");
    assert!(ready(&g));
}

#[test]
fn an_intrinsic_undying_token_does_not_count() {
    let mut g = table(&["sheoldred", "seph"]);
    perm(&mut g, P0, YAWG);
    perm(&mut g, P0, "Geralf's Messenger");
    let ghoul = perm(&mut g, P0, "Butcher Ghoul");
    g.perm_mut(ghoul).token = true;
    assert!(!ready(&g));
}
