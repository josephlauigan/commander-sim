//! The three generic AI fixes of 2026-11-02 (`ai/table_fixes.rs`), each on the situation of the pool trace that showed
//! the mistake, and the old good choices kept. `AI_SACSTEAL` / `AI_CMDRISK` / `AI_WIPEVALUE` are forced on or off per
//! test (a thread-local, like the other AI switches).

use sim_core::ai::brain::{self, Situation};
use sim_core::ai::table_fixes as tf;
use sim_core::engine::{cast, values, zones};
use sim_core::ids::PlayerId;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const TERGRID_CARD: &str = "Tergrid, God of Fright // Tergrid's Lantern";

fn on_bf(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

// ================================================================== fix 1: sacrifices with Tergrid out
/// P0 (Korvold) has a Birds of Paradise and a big token; P1 (Tergrid) has Tergrid, or not
fn edict_table(tergrid: bool) -> (Game, sim_core::ids::PermId, sim_core::ids::PermId) {
    let mut g = table(&["korvold-jund-sacrifice", "tergrid-mono-black-disruption"]);
    for p in g.players.iter_mut() {
        p.hand.clear();
    }
    let fanatic = perm(&mut g, P0, "Birds of Paradise");
    let tok = token(&mut g, P0, 6);
    if tergrid {
        perm(&mut g, P1, TERGRID_CARD);
    }
    (g, fanatic, tok)
}

#[test]
fn an_edict_gives_the_token_not_a_card_to_tergrid() {
    tf::force_sacsteal(Some(true));
    let (mut g, fanatic, tok) = edict_table(true);
    // by value alone Birds is the cheaper creature to lose; Tergrid would take it
    assert!(values::pval(&g, fanatic) < values::pval(&g, tok));
    zones::edict(&mut g, P0, false).unwrap();
    assert!(g.perm(fanatic).on_bf, "the nontoken creature stays");
    assert!(!g.perm(tok).on_bf, "the token goes");
    assert!(g.player(P1).perms.iter().all(|&m| &*g.perm(m).name != "Birds of Paradise"), "Tergrid took nothing");
    tf::force_sacsteal(None);
}

#[test]
fn an_edict_is_as_before_with_the_switch_off_or_without_tergrid() {
    tf::force_sacsteal(Some(false));
    let (mut g, fanatic, tok) = edict_table(true);
    zones::edict(&mut g, P0, false).unwrap();
    assert!(
        !g.perm(fanatic).on_bf && g.perm(tok).on_bf,
        "off: the cheaper creature by value goes (and Tergrid takes it)"
    );
    tf::force_sacsteal(Some(true));
    let (mut g, fanatic, tok) = edict_table(false);
    zones::edict(&mut g, P0, false).unwrap();
    assert!(!g.perm(fanatic).on_bf && g.perm(tok).on_bf, "no Tergrid: the cheaper creature by value goes");
    tf::force_sacsteal(None);
}

/// Torment of Hailfire (X = 1) from P1 at P0, who is at 40 life with a Birds of Paradise and nothing in hand
fn torment_at(tergrid_seat: bool) -> Game {
    let mut g = table(&["korvold-jund-sacrifice", "tergrid-mono-black-disruption"]);
    for p in g.players.iter_mut() {
        p.hand.clear();
        p.cmd_in_zone = false;
    }
    g.players[0].life = 40;
    perm(&mut g, P0, "Birds of Paradise");
    if tergrid_seat {
        perm(&mut g, P1, TERGRID_CARD);
    }
    let t = take(&mut g, P1, "Torment of Hailfire");
    cast::resolve(&mut g, P1, t, &Ctx { x: 1, ..Ctx::default() }, "hand").unwrap();
    g
}

#[test]
fn torment_of_hailfire_is_paid_in_life_not_in_cards_for_tergrid() {
    tf::force_sacsteal(Some(true));
    let g = torment_at(true);
    assert!(on_bf(&g, P0, "Birds of Paradise"), "nothing handed to Tergrid");
    assert_eq!(g.player(P0).life, 37);
    // the old good choice: with no Tergrid Birds is the cheaper way out
    let g = torment_at(false);
    assert!(!on_bf(&g, P0, "Birds of Paradise"));
    assert_eq!(g.player(P0).life, 40);
    tf::force_sacsteal(Some(false));
    let g = torment_at(true);
    assert!(!on_bf(&g, P0, "Birds of Paradise"), "off: as before");
    tf::force_sacsteal(None);
}

#[test]
fn korvolds_sacrifice_takes_the_treasure_first_and_a_land_over_a_card_with_tergrid_out() {
    tf::force_sacsteal(Some(true));
    let mut g = table(&["korvold-jund-sacrifice", "tergrid-mono-black-disruption"]);
    perm(&mut g, P1, TERGRID_CARD);
    lands(&mut g, P0, "Forest", 5, false);
    perm(&mut g, P0, "Birds of Paradise");
    g.player_mut(P0).treasures = 1;
    let got = sim_core::impls::t3::sac_worst_permanent(&mut g, P0, None).unwrap();
    assert_eq!(got, Some(sim_core::hooks::Sacrificed::Token("Treasure")), "a Treasure is no card for Tergrid");
    // no token left: five lands, a land goes as before (about one point to her; Birds would cost three)
    let got = sim_core::impls::t3::sac_worst_permanent(&mut g, P0, None).unwrap();
    assert!(matches!(got, Some(sim_core::hooks::Sacrificed::Card(_))));
    assert!(on_bf(&g, P0, "Birds of Paradise"));
    tf::force_sacsteal(None);
}

#[test]
fn the_steal_cost_is_zero_for_tokens_and_without_tergrid() {
    tf::force_sacsteal(Some(true));
    let (g, fanatic, tok) = edict_table(true);
    assert_eq!(tf::steal_cost(&g, P0, tok), 0.0);
    assert!(tf::steal_cost(&g, P0, fanatic) >= values::pval(&g, fanatic));
    assert_eq!(tf::steal_cost(&g, P1, fanatic), 0.0, "Tergrid's own opponents' view only");
    let (g, fanatic, _) = edict_table(false);
    assert_eq!(tf::steal_cost(&g, P0, fanatic), 0.0);
    tf::force_sacsteal(None);
}

// ================================================================== fix 2: recasting a commander into removal
/// Brago (P0) at tax `tax` with Hero of Bladehold in hand and 9 lands; two removal decks with mana up
fn brago_table(tax: u32, hero: bool) -> Game {
    let mut g = table(&["brago-azorius-blink-control", "meren-golgari-recursion", "kaalia-mardu-creature-cheat"]);
    for p in g.players.iter_mut() {
        p.hand.clear();
    }
    lands(&mut g, P0, "Plains", 9, false);
    lands(&mut g, P1, "Swamp", 4, false);
    lands(&mut g, P2, "Mountain", 4, false);
    // a hand each, so that the public decklists can hold removal
    hand(&mut g, P1, &["Putrefy", "Beast Within", "Swamp"]);
    hand(&mut g, P2, &["Lightning Bolt", "Chaos Warp", "Mountain"]);
    if hero {
        hand(&mut g, P0, &["Hero of Bladehold"]);
    }
    let pl = g.player_mut(P0);
    pl.turns = 8;
    pl.cmd_in_zone = true;
    pl.tax = tax;
    g
}

fn cmd_utility(g: &Game) -> f64 {
    let s = Situation::new(g, P0);
    let c = g.player(P0).cmd;
    brain::card_utility(g, P0, &s, c).expect("the commander is on offer")
}

#[test]
fn a_high_tax_commander_waits_for_the_other_threats_when_removal_is_up() {
    let g = brago_table(6, true);
    tf::force_cmdrisk(Some(false));
    let old = cmd_utility(&g);
    tf::force_cmdrisk(Some(true));
    let new = cmd_utility(&g);
    assert!(new < old - 0.4, "recast at tax 6 into two removal decks: {new} against {old}");
    let hero = card(&g, "Hero of Bladehold");
    let s = Situation::new(&g, P0);
    let hero_u = brain::card_utility(&g, P0, &s, hero).unwrap();
    // the commander is still on offer: a modest nudge, not a freeze
    assert!(new > 0.0 && new > old - tf_cap() - 0.1, "{new} {old}");
    assert!(hero_u > 0.0);
    tf::force_cmdrisk(None);
}

fn tf_cap() -> f64 {
    2.8
}

#[test]
fn the_commander_is_cast_as_before_first_cast_alone_or_when_the_table_is_killing_us() {
    tf::force_cmdrisk(Some(true));
    // first cast (tax 0): unchanged
    let g = brago_table(0, true);
    let on = cmd_utility(&g);
    tf::force_cmdrisk(Some(false));
    assert_eq!(on, cmd_utility(&g));
    // nothing else to cast: unchanged
    tf::force_cmdrisk(Some(true));
    let g = brago_table(6, false);
    let on = cmd_utility(&g);
    tf::force_cmdrisk(Some(false));
    assert_eq!(on, cmd_utility(&g));
    // the table is about to kill us: the commander saves the game
    tf::force_cmdrisk(Some(true));
    let mut g = brago_table(6, true);
    for _ in 0..6 {
        token(&mut g, P1, 6);
    }
    g.player_mut(P0).life = 8;
    let s = Situation::new(&g, P0);
    assert!(s.danger >= 0.8);
    let on = cmd_utility(&g);
    tf::force_cmdrisk(Some(false));
    assert_eq!(on, cmd_utility(&g));
    tf::force_cmdrisk(None);
}

#[test]
fn no_removal_in_the_decks_no_penalty() {
    // two decks with no removal at all: the risk is zero
    let mut g = table(&["brago-azorius-blink-control", "sythis-selesnya-enchantress"]);
    for p in g.players.iter_mut() {
        p.hand.clear();
    }
    lands(&mut g, P0, "Plains", 9, false);
    hand(&mut g, P0, &["Hero of Bladehold"]);
    let pl = g.player_mut(P0);
    pl.turns = 8;
    pl.cmd_in_zone = true;
    pl.tax = 4;
    tf::force_cmdrisk(Some(true));
    let risk = tf::cmd_removal_risk(&g, P0);
    assert!(risk < 0.35, "{risk}");
    tf::force_cmdrisk(None);
}

// ================================================================== fix 3: wipes that cost the caster as much
/// P0 casts Blasphemous Act with `mine` creatures of its own against `theirs` for each opponent
fn act_table(mine: usize, theirs: usize) -> Game {
    let mut g = table(&["isshin-mardu-attack-triggers", "meren-golgari-recursion", "kaalia-mardu-creature-cheat"]);
    for p in g.players.iter_mut() {
        p.hand.clear();
        p.cmd_in_zone = false;
    }
    lands(&mut g, P0, "Mountain", 9, false);
    hand(&mut g, P0, &["Blasphemous Act"]);
    for _ in 0..mine {
        token(&mut g, P0, 3);
    }
    for q in [P1, P2] {
        for _ in 0..theirs {
            token(&mut g, q, 3);
        }
    }
    g
}

fn wipes(g: &Game) -> usize {
    let s = Situation::new(g, P0);
    brain::wipe_options(g, P0, &s, false).len()
}

#[test]
fn a_wipe_that_costs_the_caster_as_much_as_each_opponent_is_held() {
    // Windgrace's Blasphemous Act in t2-2 round 7: two creatures each, the caster's the same as theirs
    let g = act_table(2, 2);
    tf::force_wipevalue(Some(false));
    assert_eq!(wipes(&g), 1);
    tf::force_wipevalue(Some(true));
    assert_eq!(wipes(&g), 0);
    tf::force_wipevalue(None);
}

#[test]
fn a_wipe_that_hurts_the_table_more_is_still_cast() {
    let g = act_table(1, 4);
    tf::force_wipevalue(Some(true));
    assert_eq!(wipes(&g), 1);
    // one side is far ahead: the leader goes
    let mut g = act_table(2, 2);
    for _ in 0..4 {
        token(&mut g, P1, 3);
    }
    assert_eq!(wipes(&g), 1);
    // the caster has nothing to lose
    let g = act_table(0, 2);
    assert_eq!(wipes(&g), 1);
    tf::force_wipevalue(None);
}

#[test]
fn a_symmetric_wipe_is_cast_when_the_caster_is_about_to_lose() {
    let mut g = act_table(2, 2);
    for _ in 0..4 {
        token(&mut g, P1, 6);
    }
    g.player_mut(P0).life = 6;
    let s = Situation::new(&g, P0);
    assert!(s.danger >= 1.0, "{}", s.danger);
    tf::force_wipevalue(Some(true));
    assert_eq!(wipes(&g), 1);
    tf::force_wipevalue(None);
}
