//! The Zada deck's supporting cards (impls/zada_cards.rs) and the prepared and intimidate mechanics, on hand-built
//! positions. Seat 0 plays the zada deck, seat 1 the seph deck.

use sim_core::ai::{act, brain};
use sim_core::engine::{cast, combat, hooks, removal, zones};
use sim_core::hooks::{Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["zada", "seph"])
}

fn options(g: &mut Game, p: PlayerId, post: bool) -> Vec<Opt> {
    brain::main_options(g, p, post).unwrap()
}

fn take_option(g: &mut Game, p: PlayerId, label: &str) -> bool {
    let opts = options(g, p, false);
    let o = opts
        .iter()
        .find(|o| o.label == label)
        .unwrap_or_else(|| panic!("no option {label:?} in {:?}", opts.iter().map(|o| &o.label).collect::<Vec<_>>()));
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn attack(g: &mut Game, src: PermId, p: PlayerId, atk: &[PermId], d: PlayerId) {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().attack.unwrap();
    f(g, src, p, atk, d).unwrap();
}

fn cast_at(g: &mut Game, p: PlayerId, name: &str, target: Option<PermId>, face: Option<PlayerId>) {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx { target, face, ..Ctx::default() }).unwrap();
}

fn names(g: &Game, p: PlayerId) -> Vec<String> {
    // a token is named by its subtypes
    g.player(p)
        .perms
        .iter()
        .map(|&m| {
            let x = g.perm(m);
            if x.token {
                x.ttypes.iter().map(|t| t.to_string()).collect::<Vec<_>>().join(" ")
            } else {
                x.name.to_string()
            }
        })
        .collect()
}

fn in_zone(g: &Game, zone: &[CardId], name: &str) -> bool {
    zone.iter().any(|&c| &*g.db.get(c).name == name)
}

// ------------------------------------------------------------------ creatures
#[test]
fn goblin_arsonist_pings_when_it_dies() {
    let mut g = game();
    let m = perm(&mut g, P0, "Goblin Arsonist");
    zones::die(&mut g, m, "destroy".into()).unwrap();
    assert_eq!(g.player(P1).life, 39);
}

#[test]
fn bomber_corps_battalion_needs_two_other_attackers() {
    let mut g = game();
    let b = perm(&mut g, P0, "Bomber Corps");
    let (t1, t2) = (token(&mut g, P0, 1), token(&mut g, P0, 1));
    attack(&mut g, b, P0, &[b, t1], P1);
    assert_eq!(g.player(P1).life, 40);
    attack(&mut g, b, P0, &[b, t1, t2], P1);
    assert_eq!(g.player(P1).life, 39);
}

#[test]
fn glitterfang_returns_to_hand_at_the_end_step() {
    let mut g = game();
    let f = perm(&mut g, P0, "Glitterfang");
    hooks::fire_trigger(&mut g, Event::EndStep, Call::Player { p: P0 }).unwrap();
    assert!(!g.perm(f).on_bf);
    assert!(in_zone(&g, &g.player(P0).hand, "Glitterfang"));
}

#[test]
fn goblin_rimerunner_stops_the_best_blocker() {
    let mut g = game();
    let r = perm(&mut g, P0, "Goblin Rimerunner");
    let a = token(&mut g, P0, 2);
    let small = token(&mut g, P1, 1);
    let big = perm(&mut g, P1, "Grave Titan");
    attack(&mut g, r, P0, &[a], P1);
    assert!(g.perm(r).tapped);
    assert!(g.perm(big).eot_kw.contains(&"cant_block"));
    assert!(!g.perm(small).eot_kw.contains(&"cant_block"));
    assert!(!combat::can_block(&g, big, a));
}

#[test]
fn skilled_battlecarver_has_first_strike_on_its_controllers_turn() {
    let mut g = game();
    let m = perm(&mut g, P0, "Skilled Battlecarver");
    assert!(sim_core::dsl::has_kw(&g, m, "first strike"));
    g.active = Some(P1);
    assert!(!sim_core::dsl::has_kw(&g, m, "first strike"));
}

#[test]
fn rundvelt_hordemaster_exiles_for_each_goblin_death() {
    let mut g = game();
    let h = perm(&mut g, P0, "Rundvelt Hordemaster");
    let gob = perm(&mut g, P0, "Goblin Arsonist");
    // a Goblin creature card on top: castable until the end of the next turn
    let top = take(&mut g, P0, "Mogg War Marshal");
    g.player_mut(P0).library.push(top);
    zones::die(&mut g, gob, "destroy".into()).unwrap();
    assert!(in_zone(&g, &g.player(P0).hand, "Mogg War Marshal"));
    assert!(g.player(P0).impulse_long.iter().any(|x| x.0 == top));
    // a card that isn't a Goblin creature stays exiled
    let top = take(&mut g, P0, "Arcane Signet");
    g.player_mut(P0).library.push(top);
    zones::die(&mut g, h, "destroy".into()).unwrap();
    assert!(in_zone(&g, &g.player(P0).exile, "Arcane Signet"));
    assert!(!in_zone(&g, &g.player(P0).hand, "Arcane Signet"));
}

#[test]
fn pia_makes_a_thopter_and_pumps_an_attacker_by_the_artifacts() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 6, false);
    let pia = perm(&mut g, P0, "Pia, Determined Rebuilder");
    assert!(names(&g, P0).iter().any(|n| n.contains("thopter")));
    g.player_mut(P0).treasures = 2;
    let thopter = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).fly).unwrap();
    attack(&mut g, pia, P0, &[thopter], P1);
    assert_eq!(g.perm(thopter).eot_pt, (3, 0)); // the Thopter and two Treasures
}

#[test]
fn krenkos_enforcer_has_intimidate() {
    let mut g = game();
    let e = perm(&mut g, P0, "Krenko's Enforcer");
    let red = perm(&mut g, P1, "Mogg War Marshal");
    let black = perm(&mut g, P1, "Grave Titan");
    let art = perm(&mut g, P1, "Reconstructed Thopter");
    assert!(combat::can_block(&g, red, e));
    assert!(!combat::can_block(&g, black, e));
    assert!(combat::can_block(&g, art, e));
}

#[test]
fn strife_scholar_has_ward_pay_two_life() {
    let mut g = game();
    let s = perm(&mut g, P0, "Strife Scholar");
    let c = card(&g, "Lightning Strike");
    removal::apply_removal(&mut g, Some(P1), s, "dmg3", Some(c)).unwrap();
    assert_eq!(g.player(P1).life, 38);
    assert!(!g.perm(s).on_bf);
}

// ------------------------------------------------------------------ prepared
#[test]
fn glasswright_prepared_copy_makes_a_treasure_once() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 1, false);
    let m = perm(&mut g, P0, "Goblin Glasswright");
    assert!(g.perm(m).data.truthy(DataKey::Prepared));
    assert!(take_option(&mut g, P0, "Craft with Pride (prepared copy)"));
    assert_eq!(g.player(P0).treasures, 1);
    assert!(!g.perm(m).data.truthy(DataKey::Prepared));
    assert_eq!(g.player(P0).spells_this_turn, 1);
}

#[test]
fn strife_scholar_prepared_copy_makes_two_spirits() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 6, false);
    let m = perm(&mut g, P0, "Strife Scholar");
    assert!(g.perm(m).data.truthy(DataKey::Prepared));
    assert!(take_option(&mut g, P0, "Awaken the Ages (prepared copy)"));
    assert_eq!(names(&g, P0).iter().filter(|n| n.contains("spirit")).count(), 2);
    assert!(!g.perm(m).data.truthy(DataKey::Prepared));
}

#[test]
fn firesinger_prepared_copy_adds_five_red() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 3, false);
    let m = perm(&mut g, P0, "Blazing Firesinger");
    assert!(g.perm(m).data.truthy(DataKey::Prepared));
    // the AI casts it when the mana reaches a spell (a 6-drop in hand)
    hand(&mut g, P0, &["Spawn-Gang Commander"]);
    assert!(take_option(&mut g, P0, "Seething Song (prepared copy)"));
    assert_eq!(g.player(P0).floating.r, 5);
    assert!(!g.perm(m).data.truthy(DataKey::Prepared));
}

// ------------------------------------------------------------------ the burn
#[test]
fn wrath_of_the_bloodmane_is_cheaper_with_a_legendary_creature() {
    let mut g = game();
    let c = card(&g, "Wrath of the Bloodmane");
    assert_eq!(sim_core::engine::mana::cost_of(&g, P0, c).0, 2);
    perm(&mut g, P0, "Pia, Determined Rebuilder");
    assert_eq!(sim_core::engine::mana::cost_of(&g, P0, c).0, 1);
    let t = perm(&mut g, P1, "Mogg War Marshal");
    cast_at(&mut g, P0, "Wrath of the Bloodmane", Some(t), None);
    assert!(!g.perm(t).on_bf);
}

#[test]
fn suplex_exiles_what_it_kills_or_an_artifact() {
    let mut g = game();
    let t = perm(&mut g, P1, "Mogg War Marshal");
    cast_at(&mut g, P0, "Suplex", Some(t), None);
    assert!(in_zone(&g, &g.player(P1).exile, "Mogg War Marshal"));
    assert!(!in_zone(&g, &g.player(P1).gy, "Mogg War Marshal"));
    let a = perm(&mut g, P1, "Sol Ring");
    cast_at(&mut g, P0, "Suplex", Some(a), None);
    assert!(in_zone(&g, &g.player(P1).exile, "Sol Ring"));
}

#[test]
fn scorching_dragonfire_exiles_instead_of_dying() {
    let mut g = game();
    let t = perm(&mut g, P1, "Mogg War Marshal");
    cast_at(&mut g, P0, "Scorching Dragonfire", Some(t), None);
    assert!(in_zone(&g, &g.player(P1).exile, "Mogg War Marshal"));
}

#[test]
fn heated_argument_exiles_a_graveyard_card_for_two_damage() {
    let mut g = game();
    let gy = take(&mut g, P0, "Arcane Signet");
    g.player_mut(P0).gy.push(gy);
    let t = perm(&mut g, P1, "Mogg War Marshal");
    cast_at(&mut g, P0, "Heated Argument", Some(t), None);
    assert!(!g.perm(t).on_bf);
    assert_eq!(g.player(P1).life, 38);
    assert!(in_zone(&g, &g.player(P0).exile, "Arcane Signet"));
}

#[test]
fn heated_argument_without_a_graveyard_card_deals_no_extra_damage() {
    let mut g = game();
    let t = perm(&mut g, P1, "Mogg War Marshal");
    cast_at(&mut g, P0, "Heated Argument", Some(t), None);
    assert_eq!(g.player(P1).life, 40);
}

#[test]
fn violent_echoes_empowers_jace_by_the_excess() {
    let mut g = game();
    let t = perm(&mut g, P1, "Mogg War Marshal"); // 1/1: 5 excess
    cast_at(&mut g, P0, "Violent Echoes", Some(t), None);
    assert!(!g.perm(t).on_bf);
    let j = sim_core::impls::alela::jace_token(&g, P0).expect("a Jace token");
    assert_eq!(g.perm(j).loyalty, Some(5));
}

#[test]
fn orcish_cannonade_hits_you_for_three_and_draws() {
    let mut g = game();
    let before = g.player(P0).hand.len();
    cast_at(&mut g, P0, "Orcish Cannonade", None, Some(P1));
    assert_eq!(g.player(P1).life, 38);
    assert_eq!(g.player(P0).life, 37);
    assert_eq!(g.player(P0).hand.len(), before + 1);
}

#[test]
fn reiterating_bolt_deals_three() {
    let mut g = game();
    let t = token(&mut g, P1, 3);
    cast_at(&mut g, P0, "Reiterating Bolt", Some(t), None);
    assert!(!g.perm(t).on_bf);
}
