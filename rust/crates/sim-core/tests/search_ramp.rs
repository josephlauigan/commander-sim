//! search-ramp: the look-ahead gives casting early ramp a bonus (`search::ramp_bonus`), so early ramp is cast
//! instead of held for a creature, an Equipment or "stop".

use sim_core::ai::{brain, search};
use sim_core::ids::PlayerId;
use sim_core::settings::AiMode;
use sim_core::state::Game;
use sim_core::testkit::*;
use std::sync::Arc;

const P0: PlayerId = PlayerId(0);

fn game(keys: &[&str], basic: &str) -> Game {
    let mut g = table(keys);
    g.player_mut(P0).hand.clear();
    set_ai(&mut g, AiMode::Lookahead);
    let cs: Vec<_> = (0..30).map(|_| card(&g, basic)).collect();
    g.player_mut(P0).library = cs;
    g
}

fn set_ramp_w(g: &mut Game, w: f64) {
    let mut s = (*g.settings).clone();
    s.search.ramp_w = w;
    g.settings = Arc::new(s);
}

fn bonus(g: &mut Game, label: &str) -> f64 {
    let opts = brain::main_options(g, P0, false).unwrap();
    let o = opts.iter().find(|o| o.label == label).unwrap_or_else(|| panic!("{label} castable"));
    search::ramp_bonus(g, P0, o)
}

fn pick(g: &mut Game) -> Option<String> {
    let opts = brain::main_options(g, P0, false).unwrap();
    search::choose(g, P0, false, &opts).map(|i| opts[i].label.clone())
}

#[test]
fn the_bonus_scales_with_mana_and_turns_left() {
    let mut g = game(&["cloud", "seph"], "Plains");
    set_ramp_w(&mut g, 1.0);
    lands(&mut g, P0, "Plains", 2, false);
    hand(&mut g, P0, &["Mind Stone", "Sol Ring", "Puresteel Paladin"]);
    g.player_mut(P0).turns = 2;
    assert_eq!(bonus(&mut g, "Mind Stone"), 6.0, "one mana, six turns left");
    assert_eq!(bonus(&mut g, "Sol Ring"), 12.0, "two mana");
    assert_eq!(bonus(&mut g, "Puresteel Paladin"), 0.0, "not ramp");
    g.player_mut(P0).turns = 8;
    assert_eq!(bonus(&mut g, "Mind Stone"), 0.0, "late: no bonus");
    g.player_mut(P0).turns = 2;
    set_ramp_w(&mut g, 0.0);
    assert_eq!(bonus(&mut g, "Sol Ring"), 0.0, "ramp_w 0: the old search");
}

#[test]
fn lookahead_casts_the_rock_over_a_two_drop_on_turn_two() {
    let mut g = game(&["cloud", "seph"], "Plains");
    set_ramp_w(&mut g, sim_core::settings::RAMP_W);
    lands(&mut g, P0, "Plains", 2, false);
    perm(&mut g, P0, "Cloud, Midgar Mercenary");
    hand(&mut g, P0, &["Mind Stone", "Puresteel Paladin"]);
    g.player_mut(P0).turns = 2;
    assert_eq!(pick(&mut g).as_deref(), Some("Mind Stone"));
}

#[test]
fn lookahead_casts_land_ramp_rather_than_stop() {
    let mut g = game(&["titania", "seph"], "Forest");
    set_ramp_w(&mut g, sim_core::settings::RAMP_W);
    lands(&mut g, P0, "Forest", 2, false);
    hand(&mut g, P0, &["Rampant Growth"]);
    g.player_mut(P0).turns = 2;
    assert_eq!(pick(&mut g).as_deref(), Some("Rampant Growth"));
}
