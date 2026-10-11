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

// ------------------------------------------------------------------ the Sagas
fn main1(g: &mut Game, src: PermId, p: PlayerId) {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().main1.unwrap();
    f(g, src, p).unwrap();
}

#[test]
fn burn_burn_tree_and_fern_runs_all_four_chapters() {
    let mut g = game();
    let victim = perm(&mut g, P1, "Mogg War Marshal");
    let rock = perm(&mut g, P1, "Sol Ring");
    let saga = perm(&mut g, P0, "Burn, Burn, Tree and Fern");
    assert!(!g.perm(victim).on_bf, "chapter I: 6 damage to a creature");
    assert!(g.perm(rock).on_bf);
    main1(&mut g, saga, P0);
    assert!(!g.perm(rock).on_bf, "chapter II: destroy an artifact");
    main1(&mut g, saga, P0);
    assert_eq!(g.player(P0).floating.r, 1, "chapter III adds {{R}}");
    assert!(g.perm(saga).on_bf);
    main1(&mut g, saga, P0);
    assert_eq!(g.player(P0).floating.r, 2, "chapter IV adds {{R}}");
    assert!(!g.perm(saga).on_bf, "sacrificed after IV");
}

#[test]
fn ral_and_the_implicit_maze_runs_all_three_chapters() {
    let mut g = game();
    let small = perm(&mut g, P1, "Mogg War Marshal");
    let big = zones::make_tokens(&mut g, P1, zones::Tokens { tgh: Some(5), ..zones::Tokens::new(1, 5) }).unwrap()[0];
    hand(&mut g, P0, &["Mountain"]);
    let saga = perm(&mut g, P0, "Ral and the Implicit Maze");
    assert!(!g.perm(small).on_bf, "chapter I: 2 damage to each opposing creature");
    assert!(g.perm(big).on_bf);
    let hand_before = g.player(P0).hand.len();
    main1(&mut g, saga, P0);
    // discarded one, exiled two playable: net one more card in the hand
    assert_eq!(g.player(P0).hand.len(), hand_before + 1);
    assert_eq!(g.player(P0).impulse_long.len(), 2);
    main1(&mut g, saga, P0);
    let weird = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).ttypes.contains(&"weird")).expect("a Weird");
    assert!(!g.perm(saga).on_bf, "sacrificed after III");
    // the Weird grows with each noncreature spell
    lands(&mut g, P0, "Mountain", 2, false);
    let t = perm(&mut g, P1, "Mogg War Marshal");
    cast_at(&mut g, P0, "Wrath of the Bloodmane", Some(t), None);
    assert_eq!(g.perm(weird).plus, 1);
}

// ------------------------------------------------------------------ Command the Stage, Glimpse the Impossible
#[test]
fn command_the_stage_makes_a_cadet_and_grows_other_wizard_tokens() {
    let mut g = game();
    let old = zones::make_tokens(
        &mut g,
        P0,
        zones::Tokens { types: vec!["wizard"], tgh: Some(2), ..zones::Tokens::new(1, 2) },
    )
    .unwrap()[0];
    cast_at(&mut g, P0, "Command the Stage", None, None);
    assert_eq!(g.perm(old).plus, 1);
    let cadets: Vec<PermId> = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).ttypes.contains(&"wizard") && g.perm(m).ttypes.contains(&"soldier"))
        .collect();
    assert_eq!(cadets.len(), 1);
    assert_eq!(g.perm(cadets[0]).plus, 0, "only the other Wizard tokens get a counter");
}

#[test]
fn command_the_stage_returns_after_noncombat_damage_to_an_opponent() {
    let mut g = game();
    let c = card(&g, "Command the Stage");
    g.player_mut(P0).gy.push(c);
    // no damage: it stays
    sim_core::engine::turn::upkeep(&mut g, P0).unwrap();
    assert!(g.player(P0).gy.contains(&c));
    // an opponent is burned, the turn passes, the next upkeep returns it
    sim_core::engine::life::lose_life(&mut g, P1, 2, Some(P0), "burn".into(), None).unwrap();
    sim_core::engine::turn::upkeep(&mut g, P1).unwrap();
    assert!(g.player(P0).hand.contains(&c));
    assert!(!g.player(P0).gy.contains(&c));
    assert_eq!(g.player(P0).gy_left, Some(g.turn_stamp()));
}

#[test]
fn command_the_stage_ignores_combat_damage() {
    let mut g = game();
    let c = card(&g, "Command the Stage");
    g.player_mut(P0).gy.push(c);
    sim_core::engine::life::lose_life(&mut g, P1, 2, Some(P0), "combat".into(), None).unwrap();
    sim_core::engine::turn::upkeep(&mut g, P1).unwrap();
    assert!(g.player(P0).gy.contains(&c));
}

#[test]
fn glimpse_the_impossible_makes_spawn_for_the_unplayed_cards() {
    let mut g = game();
    let lib = g.player(P0).library.len();
    cast_at(&mut g, P0, "Glimpse the Impossible", None, None);
    assert_eq!(g.player(P0).library.len(), lib - 4); // one is the spell
    assert_eq!(g.player(P0).glimpse_exiled.len(), 3);
    let exiled = g.player(P0).glimpse_exiled.clone();
    // one is played (leaves the hand), two stay exiled
    g.player_mut(P0).hand.retain(|&c| c != exiled[0]);
    let gy = g.player(P0).gy.len();
    sim_core::engine::turn::end_step(&mut g, P0).unwrap();
    let spawns = names(&g, P0).iter().filter(|n| n.contains("spawn")).count();
    assert_eq!(spawns, 2);
    assert_eq!(g.player(P0).gy.len(), gy + 2);
    assert!(exiled[1..].iter().all(|c| g.player(P0).gy.contains(c)));
}

// ------------------------------------------------------------------ Living History
#[test]
fn living_history_pumps_an_attacker_after_a_card_left_the_graveyard() {
    let mut g = game();
    let hist = perm(&mut g, P0, "Living History");
    assert_eq!(names(&g, P0).iter().filter(|n| n.contains("spirit")).count(), 1);
    let a = token(&mut g, P0, 2);
    let base = sim_core::engine::values::epow(&g, a);
    attack(&mut g, hist, P0, &[a], P1);
    assert_eq!(sim_core::engine::values::epow(&g, a), base, "nothing left the graveyard yet");
    // Command the Stage comes back from the graveyard
    let c = card(&g, "Command the Stage");
    g.player_mut(P0).gy.push(c);
    sim_core::engine::life::lose_life(&mut g, P1, 1, Some(P0), "burn".into(), None).unwrap();
    sim_core::engine::turn::upkeep(&mut g, P1).unwrap();
    attack(&mut g, hist, P0, &[a], P1);
    assert_eq!(sim_core::engine::values::epow(&g, a), base + 2);
}

#[test]
fn casting_from_the_graveyard_counts_as_leaving_it() {
    let mut g = game();
    let c = card(&g, "Orcish Cannonade");
    g.player_mut(P0).gy.push(c);
    cast::cast_card(&mut g, P0, c, "gy", Ctx { face: Some(P1), ..Ctx::default() }).unwrap();
    assert_eq!(g.player(P0).gy_left, Some(g.turn_stamp()));
}

// ------------------------------------------------------------------ Irascible Wolverine (plot)
#[test]
fn irascible_wolverine_exiles_the_top_card_on_entry() {
    let mut g = game();
    let top = *g.player(P0).library.last().unwrap();
    let hand_before = g.player(P0).hand.len();
    perm(&mut g, P0, "Irascible Wolverine");
    assert_eq!(g.player(P0).hand.len(), hand_before + 1);
    assert!(g.player(P0).hand.contains(&top));
    assert!(g.player(P0).impulse.contains(&top));
}

#[test]
fn irascible_wolverine_plots_then_casts_free_on_a_later_turn() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 3, false);
    g.active = Some(P0);
    let w = hand(&mut g, P0, &["Irascible Wolverine"])[0];
    // plot in the second main phase
    let opts = options(&mut g, P0, true);
    let o = opts.iter().find(|o| o.label == "plot Irascible Wolverine").expect("plot option");
    assert!(act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(g.player(P0).hand.contains(&w), "a plotted card stays in the hand");
    assert!(!sim_core::engine::mana::can_pay(&g, P0, 2, "R", false), "the plot cost was paid");
    // the same turn: no free cast
    assert!(!options(&mut g, P0, false).iter().any(|o| o.label.contains("plotted")));
    // later turn: free cast, and the normal cast is not offered
    g.player_mut(P0).turns += 1;
    for l in g.player(P0).lands.clone() {
        g.land_mut(l).tapped = true;
    }
    assert!(take_option(&mut g, P0, "Irascible Wolverine (plotted, free)"));
    assert!(g.player(P0).perms.iter().any(|&m| &*g.perm(m).name == "Irascible Wolverine"));
    assert!(g.player(P0).impulse_long.iter().all(|x| x.0 != w));
}

// ------------------------------------------------------------------ Lightning, Security Sergeant
#[test]
fn lightning_exiles_a_card_on_combat_damage_and_loses_it_when_it_leaves() {
    let mut g = game();
    let l = perm(&mut g, P0, "Lightning, Security Sergeant");
    let top = *g.player(P0).library.last().unwrap();
    let c = g.perm(l).cd.unwrap();
    let f = g.registry.get(c).unwrap().combat_damage.unwrap();
    f(&mut g, l, P0, l, P1, 2).unwrap();
    assert!(g.player(P0).hand.contains(&top));
    assert!(g.player(P0).impulse_long.contains(&(top, u32::MAX)));
    zones::die(&mut g, l, "destroy".into()).unwrap();
    assert!(!g.player(P0).hand.contains(&top));
    assert!(g.player(P0).exile.contains(&top));
    assert!(g.player(P0).impulse_long.is_empty());
}

// ------------------------------------------------------------------ Spawn-Gang Commander
#[test]
fn spawn_gang_commander_makes_three_spawn_and_throws_damage() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 1, false);
    cast_at(&mut g, P0, "Spawn-Gang Commander", None, None);
    let spawn = |g: &Game| names(g, P0).iter().filter(|n| n.contains("spawn")).count();
    assert_eq!(spawn(&g), 3);
    let cmd = g.player(P0).perms.iter().copied().find(|&m| !g.perm(m).token).unwrap();
    assert!(sim_core::engine::values::colors_of(&g, cmd).is_empty(), "devoid");
    for m in g.player(P0).perms.clone() {
        g.perm_mut(m).sick = false;
    }
    let victim = perm(&mut g, P1, "Mogg War Marshal");
    assert!(take_option(&mut g, P0, "Spawn-Gang Commander: 2 damage"));
    assert!(!g.perm(victim).on_bf || g.player(P1).life < 40);
    assert!(spawn(&g) <= 2, "a Spawn was sacrificed");
}

// ------------------------------------------------------------------ Relic of Legends, Bonders' Enclave
#[test]
fn relic_of_legends_taps_a_legendary_creature_for_mana() {
    let mut g = game();
    perm(&mut g, P0, "Relic of Legends");
    let before = sim_core::engine::mana::total_mana(&g, P0, false);
    assert_eq!(before, 1);
    let pia = perm(&mut g, P0, "Pia, Determined Rebuilder");
    g.perm_mut(pia).sick = true;
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 2);
    g.perm_mut(pia).tapped = true;
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 1);
}

#[test]
fn bonders_enclave_draws_with_a_big_creature() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 3, false);
    let e = g.add_land(P0, card(&g, "Bonders' Enclave"), false);
    let f = g.registry.get(g.land(e).cd).unwrap().land_options.unwrap();
    assert!(f(&mut g, e, P0, None).unwrap().is_empty(), "no big creature");
    token(&mut g, P0, 4);
    let opts = f(&mut g, e, P0, None).unwrap();
    assert_eq!(opts.len(), 1);
    let hand_before = g.player(P0).hand.len();
    assert!(act::perform(&mut g, P0, opts[0].act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P0).hand.len(), hand_before + 1);
    assert!(g.land(e).tapped);
}

// ------------------------------------------------------------------ Generator Servant
#[test]
fn generator_servant_sacrifices_for_two_colourless_and_hastes_a_creature_spell() {
    let mut g = game();
    let s = perm(&mut g, P0, "Generator Servant");
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 2);
    let c = hand(&mut g, P0, &["Goblin Rimerunner"])[0];
    g.pay_for = Some(c);
    sim_core::engine::mana::pay(&mut g, P0, 2, "", false).unwrap();
    g.pay_for = None;
    assert!(!g.perm(s).on_bf, "sacrificed");
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    let m = g.player(P0).perms.iter().copied().find(|&m| &*g.perm(m).name == "Goblin Rimerunner").unwrap();
    assert!(!g.perm(m).sick, "haste from the Servant's mana");
}
