//! Tests for the cloud-ai-fix2 task: one section per fix.

use sim_core::ids::PlayerId;
use sim_core::settings::AiMode;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const CLOUD: &str = "Cloud, Midgar Mercenary";

fn game() -> Game {
    let mut g = table(&["cloud", "seph"]);
    g.player_mut(P0).hand.clear();
    set_ai(&mut g, AiMode::Lookahead);
    g
}

fn set_library(g: &mut Game, p: PlayerId, names: &[&str]) {
    let cs: Vec<_> = names.iter().map(|n| card(g, n)).collect();
    g.player_mut(p).library = cs;
}

const LIB: [&str; 7] = [
    "Sword of Fire and Ice",
    "Sword of Hearth and Home",
    "Sword of Feast and Famine",
    "Champion's Helm",
    "Colossus Hammer",
    "Sword of Kaldra",
    "Wrecking Ball Arm",
];

fn fetched(g: &Game) -> Vec<String> {
    g.player(P0).hand.iter().map(|&c| g.db.get(c).name.to_string()).collect()
}

// ---- fix 1: engine-aware tutor
#[test]
fn tutor_takes_a_big_piece_with_puresteel_metalcraft() {
    let mut g = game();
    set_library(&mut g, P0, &LIB);
    lands(&mut g, P0, "Plains", 2, false);
    for n in ["Sol Ring", "Mind Stone"] {
        perm(&mut g, P0, n);
    }
    perm(&mut g, P0, "Puresteel Paladin");
    perm(&mut g, P0, CLOUD);
    let got = fetched(&g);
    assert_eq!(got.len(), 1, "{got:?}");
    assert!(["Sword of Kaldra", "Wrecking Ball Arm", "Colossus Hammer"].contains(&got[0].as_str()), "{got:?}");
}

#[test]
fn tutor_takes_a_cheap_sword_without_an_engine() {
    let mut g = game();
    set_library(&mut g, P0, &LIB);
    lands(&mut g, P0, "Plains", 2, false);
    for n in ["Sol Ring", "Mind Stone"] {
        perm(&mut g, P0, n);
    }
    perm(&mut g, P0, CLOUD);
    assert_eq!(fetched(&g), vec!["Sword of Fire and Ice".to_string()]);
}

#[test]
fn tutor_takes_a_big_piece_with_ardenn_out() {
    let mut g = game();
    set_library(&mut g, P0, &LIB);
    lands(&mut g, P0, "Plains", 3, false);
    perm(&mut g, P0, "Ardenn, Intrepid Archaeologist");
    perm(&mut g, P0, CLOUD);
    let got = fetched(&g);
    assert_eq!(got.len(), 1, "{got:?}");
    assert!(["Sword of Kaldra", "Wrecking Ball Arm", "Colossus Hammer"].contains(&got[0].as_str()), "{got:?}");
}

const P1: PlayerId = PlayerId(1);

// ---- fix 5: no ping-pong
#[test]
fn worn_equipment_moves_only_to_a_clearly_better_holder() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 4, false);
    let cl = perm(&mut g, P0, CLOUD);
    g.perm_mut(cl).noatk = true; // out of the picture as a holder, but on the battlefield: moves are wanted
    let kd = perm(&mut g, P0, "Kor Duelist");
    let bg = perm(&mut g, P0, "Bronze Guardian");
    let mask = perm(&mut g, P0, "Mask of Memory");
    g.perm_mut(mask).attached = Some(kd);
    let offered = |g: &Game| sim_core::dsl::equip_options(g, P0).iter().any(|o| o.label == "equip Mask of Memory");
    assert!(!offered(&g), "equal holders: no move");
    g.perm_mut(bg).plus += 8;
    assert!(offered(&g), "a much bigger holder: move");
}

// ---- fix 7: wipes and edicts are logged
fn log_of(g: &mut Game, f: impl FnOnce(&mut Game)) -> Vec<String> {
    g.log = Some(vec![]);
    f(g);
    g.log.take().unwrap_or_default()
}

#[test]
fn wipes_and_edicts_name_what_dies() {
    let mut g = game();
    let cl = perm(&mut g, P0, CLOUD);
    let kd = perm(&mut g, P0, "Kor Duelist");
    let _ = (cl, kd);
    let l = log_of(&mut g, |g| {
        sim_core::engine::removal::apply_wipe(g, P1, "destroy".into(), None, &Default::default()).unwrap();
    });
    assert!(l.iter().any(|x| x.contains(CLOUD) && x.contains("removed by the wipe")), "{l:?}");
    assert!(l.iter().any(|x| x.contains("Kor Duelist") && x.contains("removed by the wipe")), "{l:?}");
    // an edict
    let mut g = game();
    perm(&mut g, P0, "Kor Duelist");
    let l = log_of(&mut g, |g| sim_core::engine::zones::edict(g, P0, false).unwrap());
    assert!(l.iter().any(|x| x.contains("sacrifices Kor Duelist (edict)")), "{l:?}");
}
