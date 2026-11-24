//! The format-dependent rules (`Settings.rules`, format.rs): every player's starting life total and whether the
//! starting player draws on its first turn (CR 103.8a). Cali is 30 life and no first draw; Commander is 40 and the old
//! rule (the starting player draws), which every existing test and trace relies on.

mod play_common;
use play_common::*;
use sim_core::engine::hooks::fire_trigger;
use sim_core::engine::values::{epow, etgh, threat};
use sim_core::engine::{combat, turn};
use sim_core::format::Rules;
use sim_core::hooks::{Action, Call, Event};
use sim_core::settings::AiMode;
use sim_core::state::DataKey;

const CALI: Rules = Rules { life: 30, first_player_draws: false };
const THEORIST: &str = "The Theorist, Jace Beleren";

fn cali(keys: &[&str]) -> Game {
    table_in(keys, 1, CALI)
}

fn commander(keys: &[&str]) -> Game {
    table_in(keys, 1, Rules::COMMANDER)
}

/// a table whose players hold known nonlands (the land drop can't hide a draw) and whose seat 0 has not had a turn
fn bare(rules: Rules, n: usize) -> Game {
    let mut g = table_in(&["galadriel", "cloud", "seph", "sauron"][..n], 1, rules);
    let c = card(&g, "Counterspell");
    for q in &mut g.players {
        q.library = vec![c; 20];
        q.hand.clear();
        q.turns = 0;
    }
    g.round = 1;
    g
}

// ------------------------------------------------------------------ starting life
#[test]
fn every_seat_starts_at_the_rules_life_total() {
    let g = cali(&["galadriel", "cloud", "seph"]);
    assert!(g.players.iter().all(|p| p.life == 30 && p.start_life == 30));
    let g = commander(&["galadriel", "cloud", "seph", "sauron"]);
    assert!(g.players.iter().all(|p| p.life == 40 && p.start_life == 40));
}

#[test]
fn a_whole_setup_game_starts_every_seat_at_the_rules_life_total() {
    for (rules, life) in [(CALI, 30), (Rules::COMMANDER, 40)] {
        let g = play_in(&["arzakon", "seph"], 3, false, AiMode::Adaptive, rules);
        assert!(g.players.iter().all(|p| p.start_life == life), "start_life {life}");
    }
}

#[test]
fn commander_rules_are_the_default_and_the_old_table_helpers_stay_commander() {
    assert_eq!(Rules::default(), Rules::COMMANDER);
    assert_eq!((Rules::COMMANDER.life, Rules::COMMANDER.first_player_draws), (40, true));
    let g = table(&["galadriel", "cloud"]);
    assert!(g.players.iter().all(|p| p.life == 40 && p.start_life == 40));
    let g = table_with(&["galadriel", "cloud"], 1, 23);
    assert!(g.players.iter().all(|p| p.life == 23 && p.start_life == 40), "table_with sets only the life total");
}

// ------------------------------------------------------------------ the starting player's first draw
#[test]
fn cali_seat_zero_skips_the_draw_of_its_first_turn_and_commander_does_not() {
    let mut g = bare(CALI, 2);
    turn::step_start(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), 0, "no draw on the play");
    let mut g = bare(Rules::COMMANDER, 2);
    turn::step_start(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), 1, "a Commander game draws as it always did");
}

#[test]
fn cali_seat_one_draws_on_its_first_turn() {
    let mut g = bare(CALI, 2);
    turn::step_start(&mut g, P1).unwrap();
    assert_eq!(g.player(P1).hand.len(), 1);
}

#[test]
fn cali_seat_zero_draws_on_its_second_turn() {
    let mut g = bare(CALI, 2);
    turn::step_start(&mut g, P0).unwrap();
    g.round = 2;
    turn::step_start(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), 1, "the second turn draws");
}

#[test]
fn cali_extra_turn_in_round_one_draws() {
    let mut g = bare(CALI, 2);
    turn::step_start(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), 0);
    turn::step_start(&mut g, P0).unwrap(); // an extra turn: still round 1, the second turn of the seat
    assert_eq!(g.player(P0).hand.len(), 1, "only the first turn skips the draw");
}

#[test]
fn the_skipped_step_logs_a_line_only_when_it_skips() {
    let skip_line = |g: &Game| g.log.as_ref().unwrap().iter().any(|l| l.contains("skips the draw step"));
    let mut g = bare(CALI, 2);
    g.log = Some(vec![]);
    turn::step_start(&mut g, P0).unwrap();
    assert!(skip_line(&g));
    let mut g = bare(Rules::COMMANDER, 2);
    g.log = Some(vec![]);
    turn::step_start(&mut g, P0).unwrap();
    assert!(!skip_line(&g), "Commander logs nothing new");
}

#[test]
fn mana_vault_does_not_ping_on_the_skipped_step() {
    // Mana Vault stays tapped through the untap step, so at the draw step it deals 1 damage in Commander
    for (rules, life) in [(CALI, 30), (Rules::COMMANDER, 39)] {
        let mut g = bare(rules, 2);
        let v = perm(&mut g, P0, "Mana Vault");
        g.perm_mut(v).tapped = true;
        turn::step_start(&mut g, P0).unwrap();
        assert_eq!(g.player(P0).life, life, "{rules:?}");
    }
}

#[test]
fn howling_mine_adds_nothing_on_the_skipped_step() {
    let mut g = bare(CALI, 2);
    perm(&mut g, P0, "Howling Mine");
    turn::step_start(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), 0);
    let mut g = bare(Rules::COMMANDER, 2);
    perm(&mut g, P0, "Howling Mine");
    turn::step_start(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), 2, "the draw and the Mine's extra card");
}

#[test]
fn the_theorist_does_not_draw_on_the_skipped_step() {
    let mut g = bare(CALI, 2);
    perm(&mut g, P1, THEORIST);
    turn::step_start(&mut g, P0).unwrap();
    assert_eq!(g.player(P1).hand.len(), 0);
    let mut g = bare(Rules::COMMANDER, 2);
    perm(&mut g, P1, THEORIST);
    turn::step_start(&mut g, P0).unwrap();
    assert_eq!(g.player(P1).hand.len(), 1, "an opponent's draw step still triggers it in Commander");
}

#[test]
fn a_person_on_the_play_in_cali_gets_no_draw_step_request() {
    let dredges = |rules: Rules| {
        let mut g = bare(rules, 2);
        let imp = take(&mut g, P0, "Stinkweed Imp"); // dredge 5: the person is asked in the draw step
        g.player_mut(P0).gy.push(imp);
        let h = std::sync::Arc::new(sim_core::human::script::ScriptHuman::from_fn(|r| {
            if r.prompt.contains("dredge") { Answer::Pick(1) } else { Answer::Decline }
        }));
        g.humans.add(P0, h.clone());
        turn::step_start(&mut g, P0).unwrap();
        (h.asked().iter().filter(|r| r.prompt.contains("Draw step")).count(), g.player(P0).hand.len())
    };
    assert_eq!(dredges(CALI), (0, 0), "no dredge offer and no card on the skipped step");
    assert_eq!(dredges(Rules::COMMANDER), (1, 1), "Commander still offers the dredge, then draws");
}

// ------------------------------------------------------------------ cards that read the starting life total
#[test]
fn cecil_flips_at_half_the_starting_life_total() {
    let hit = |rules: Rules, life: i32| {
        let mut g = table_in(&["galadriel", "cloud", "seph"], 1, rules);
        let c = perm(&mut g, P0, "Cecil, Dark Knight");
        g.player_mut(P0).life = life;
        combat::resolve_combat(&mut g, P0, &[c], P1, &[c]).unwrap();
        g.perm(c).data.truthy(DataKey::Flipped)
    };
    assert!(!hit(CALI, 22), "20 is more than 15");
    assert!(hit(CALI, 17), "15 is half of 30");
    assert!(hit(Rules::COMMANDER, 22), "20 is half of 40");
}

#[test]
fn cosmos_elixir_draws_above_the_starting_life_total_and_gains_at_or_below_it() {
    let end = |rules: Rules, life: i32| {
        let mut g = table_in(&["heliod-mono-white-stax", "seph", "sauron"], 1, rules);
        perm(&mut g, P0, "Cosmos Elixir");
        g.player_mut(P0).life = life;
        let hand0 = g.player(P0).hand.len();
        fire_trigger(&mut g, Event::EndStep, Call::Player { p: P0 }).unwrap();
        (g.player(P0).life, g.player(P0).hand.len() - hand0)
    };
    assert_eq!(end(CALI, 30), (32, 0), "at 30: gain 2");
    assert_eq!(end(CALI, 31), (31, 1), "above 30: draw");
    assert_eq!(end(Rules::COMMANDER, 35), (37, 0), "35 is not above 40");
    assert_eq!(end(Rules::COMMANDER, 41), (41, 1));
}

#[test]
fn ajani_zero_needs_fifteen_above_the_starting_life_total() {
    let zero = |rules: Rules, life: i32| {
        let mut g = table_in(&["heliod-mono-white-stax", "seph", "sauron"], 1, rules);
        let a = perm(&mut g, P0, "Ajani, Strength of the Pride");
        perm(&mut g, P1, "Sun Titan"); // something worth exiling: the AI only offers the 0 when it pays
        g.player_mut(P0).life = life;
        let f = g.registry.get(g.perm(a).cd.unwrap()).unwrap().options.unwrap();
        f(&mut g, a, P0, Some(false)).unwrap().iter().any(|o| o.label.ends_with("+0") && o.act.is_some())
    };
    assert!(!zero(CALI, 44) && zero(CALI, 45), "45 in Cali");
    assert!(!zero(Rules::COMMANDER, 54) && zero(Rules::COMMANDER, 55), "55 in Commander");
}

#[test]
fn righteous_valkyrie_pumps_at_seven_above_the_starting_life_total() {
    let pumped = |rules: Rules, life: i32| {
        let mut g = table_in(&["giada-calling-all-angels", "cloud", "seph"], 1, rules);
        perm(&mut g, P0, "Righteous Valkyrie");
        let w = perm(&mut g, P0, "Metallic Mimic");
        let base = (epow(&g, w), etgh(&g, w));
        g.player_mut(P0).life = life;
        (epow(&g, w), etgh(&g, w)) != base
    };
    assert!(!pumped(CALI, 36) && pumped(CALI, 37), "37 in Cali");
    assert!(!pumped(Rules::COMMANDER, 46) && pumped(Rules::COMMANDER, 47), "47 in Commander");
}

#[test]
fn speaker_of_the_heavens_needs_seven_above_the_starting_life_total() {
    let allowed = |rules: Rules, life: i32| {
        let mut g = table_in(&["giada-calling-all-angels", "cloud", "seph"], 1, rules);
        g.active = Some(P0);
        let s = perm(&mut g, P0, "Speaker of the Heavens");
        g.perm_mut(s).sick = false;
        let hook = *g.registry.get(card(&g, "Speaker of the Heavens")).unwrap();
        g.player_mut(P0).life = life;
        let opts = (hook.options.unwrap())(&mut g, s, P0, None).unwrap();
        opts.into_iter().any(|o| matches!(o.act, Some(Action::Ability { .. })))
    };
    assert!(!allowed(CALI, 36) && allowed(CALI, 37), "37 in Cali");
    assert!(!allowed(Rules::COMMANDER, 46) && allowed(Rules::COMMANDER, 47), "47 in Commander");
}

#[test]
fn shadow_of_mortality_costs_less_per_life_below_the_starting_life_total() {
    let less = |rules: Rules, life: i32| {
        let mut g = table_in(&["yuriko-dimir-ninjas", "seph", "cloud"], 1, rules);
        let c = card(&g, "Shadow of Mortality");
        let f = g.registry.get(c).unwrap().self_cost.unwrap();
        g.player_mut(P0).life = life;
        f(&g, P0, c)
    };
    assert_eq!(less(CALI, 30), 0);
    assert_eq!(less(CALI, 18), -12, "30 - 18");
    assert_eq!(less(Rules::COMMANDER, 28), -12, "40 - 28, as before");
}

#[test]
fn the_threat_baseline_is_the_starting_life_total() {
    let g = cali(&["galadriel", "cloud"]);
    assert_eq!(threat(&g, P0, P1), 0.0, "an empty board at 30 is the baseline");
    let mut g = cali(&["galadriel", "cloud"]);
    g.player_mut(P1).life = 40;
    assert!((threat(&g, P0, P1) - 0.5).abs() < 1e-9);
    let g = commander(&["galadriel", "cloud"]);
    assert_eq!(threat(&g, P0, P1), 0.0);
}

// ------------------------------------------------------------------ whole games
#[test]
fn two_seat_games_finish_under_cali_rules() {
    let t5 = tier("t5");
    assert!(!t5.is_empty());
    for (i, seed) in [1u64, 2, 3].into_iter().enumerate() {
        let opp = t5[i % t5.len()];
        let g = play_in(&["arzakon", opp], seed, false, AiMode::Adaptive, CALI);
        assert_eq!(g.players.len(), 2);
        assert!(g.over || g.wintype == Some("timeout"), "seed {seed} vs {opp}: the game ended or timed out");
        assert!(g.winner.is_some() || g.players.iter().all(|p| !p.alive), "seed {seed} vs {opp}");
    }
}
