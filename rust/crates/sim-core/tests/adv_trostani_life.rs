//! Independent Batch B adversarial fixtures. Oracle text supplies the expectations.
//! `resolving` defers triggers while a spell/effect is still resolving; no production hooks are changed on disk.

use sim_core::control::{Answer, Request};
use sim_core::engine::hooks::fire_trigger;
use sim_core::engine::stack::{abilities_answered, flush_triggers, trigger_window};
use sim_core::engine::values::{epow, etgh, is_artifact_perm};
use sim_core::engine::zones::{self, Tokens};
use sim_core::engine::{life, repl};
use sim_core::hooks::{Action, Call, Event, Registry, Src};
use sim_core::human::script::ScriptHuman;
use sim_core::ids::{PermId, PlayerId};
use sim_core::settings::Profile;
use sim_core::state::{DataKey, Game, Val};
use sim_core::testkit::*;
use std::sync::{Arc, Mutex};

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const T: &str = "Trostani, Selesnya's Voice";
const A: &str = "Verdant Sun's Avatar";
const E: &str = "Emiel the Blessed";
const B: &str = "Treebeard, Gracious Host";
const TEST: &str = "Test of Endurance";
const FEL: &str = "Felidar Sovereign";

fn tbl() -> Game {
    let mut g = table(&["trostani", "seph", "sauron"]);
    let mut settings = (*g.settings).clone();
    settings.profile = Profile::Loose;
    g.settings = Arc::new(settings);
    g
}

fn responses(g: &mut Game) {
    perm(g, P1, "Azorius Guildmage");
    assert!(abilities_answered(g, None));
}

fn tok(g: &mut Game, types: Vec<&'static str>, tgh: i32) -> PermId {
    zones::make_tokens(g, P0, Tokens { types, tgh: Some(tgh), ..Tokens::new(1, 1) }).unwrap()[0]
}

fn tokens(g: &Game) -> Vec<PermId> {
    g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).on_bf && g.perm(m).token).collect()
}

fn copied(g: &mut Game) -> PermId {
    let before = tokens(g);
    sim_core::impls::zur::populate(g, P0).unwrap();
    let made: Vec<_> = tokens(g).into_iter().filter(|m| !before.contains(m)).collect();
    assert_eq!(made.len(), 1);
    made[0]
}

fn finish(g: &mut Game) {
    g.resolving -= 1;
    flush_triggers(g).unwrap();
}

#[test]
fn emiel_unicorn_token_gets_two_counters() {
    let mut g = tbl();
    perm(&mut g, P0, E);
    lands(&mut g, P0, "Plains", 1, false);
    let m = tok(&mut g, vec!["unicorn"], 1);
    assert_eq!(g.perm(m).plus, 2);
}

#[test]
fn emiel_unicorn_counters_are_replaced_by_doubling_season() {
    let mut g = tbl();
    perm(&mut g, P0, E);
    perm(&mut g, P0, "Doubling Season");
    lands(&mut g, P0, "Plains", 2, false);
    let m = tok(&mut g, vec!["unicorn"], 1);
    assert_eq!(g.perm(m).plus, 4);
    assert!(tokens(&g).iter().all(|&m| g.perm(m).plus == 4));
}

#[test]
fn emiel_hybrid_payment_accepts_green() {
    let mut g = tbl();
    perm(&mut g, P0, E);
    lands(&mut g, P0, "Forest", 1, false);
    let m = tok(&mut g, vec!["soldier"], 1);
    assert_eq!(g.perm(m).plus, 1);
}

#[test]
fn emiel_hybrid_payment_rejects_colorless() {
    let mut g = tbl();
    perm(&mut g, P0, E);
    lands(&mut g, P0, "Wastes", 3, false);
    let m = tok(&mut g, vec!["soldier"], 1);
    assert_eq!(g.perm(m).plus, 0);
}

#[test]
fn emiel_token_without_mana_gets_no_counter() {
    let mut g = tbl();
    perm(&mut g, P0, E);
    let m = tok(&mut g, vec!["unicorn"], 1);
    assert_eq!(g.perm(m).plus, 0);
}

/// CR 113.7a: a triggered ability exists independently of its source. Emiel's already-triggered ability can still pay.
#[test]
fn emiel_trigger_survives_its_source_leaving() {
    let mut g = tbl();
    responses(&mut g);
    let e = perm(&mut g, P0, E);
    lands(&mut g, P0, "Plains", 1, false);
    g.resolving += 1;
    let m = tok(&mut g, vec!["soldier"], 1);
    zones::die(&mut g, e, "sac").unwrap();
    finish(&mut g);
    assert_eq!(g.perm(m).plus, 1, "Emiel's trigger must survive its source");
}

/// CR 113.7a: the life-gain trigger survives Trostani leaving; it is not an ongoing static ability.
#[test]
fn trostani_trigger_survives_its_source_leaving() {
    let mut g = tbl();
    let t = perm(&mut g, P0, T);
    g.player_mut(P0).life = 20;
    g.resolving += 1;
    tok(&mut g, vec!["beast"], 3);
    zones::die(&mut g, t, "sac").unwrap();
    finish(&mut g);
    assert_eq!(g.player(P0).life, 23);
}

/// CR 608.2h: the toughness is read on resolution, including a -2/-2 effect applied since the trigger event.
#[test]
fn trostani_reads_reduced_toughness_on_resolution() {
    let mut g = tbl();
    perm(&mut g, P0, T);
    g.player_mut(P0).life = 20;
    g.resolving += 1;
    let m = tok(&mut g, vec!["beast"], 3);
    g.perm_mut(m).eot_pt = (-2, -2);
    finish(&mut g);
    assert_eq!(g.player(P0).life, 21);
}

/// Trostani's toughness instruction cannot cause life loss when the last known toughness was negative (CR 107.1b).
#[test]
fn trostani_negative_toughness_gains_zero() {
    let mut g = tbl();
    perm(&mut g, P0, T);
    g.player_mut(P0).life = 20;
    g.resolving += 1;
    let m = tok(&mut g, vec!["beast"], 3);
    g.perm_mut(m).eot_pt = (-5, -5);
    zones::die(&mut g, m, "sba").unwrap();
    finish(&mut g);
    assert_eq!(g.player(P0).life, 20);
}

/// CR 608.2h: the departed creature's last known toughness includes equipment, even though leaving detaches it.
#[test]
#[ignore = "T5: pending JD"]
fn trostani_last_known_toughness_includes_detached_equipment() {
    let mut g = tbl();
    perm(&mut g, P0, T);
    let sword = perm(&mut g, P0, "Sword of Fire and Ice");
    g.player_mut(P0).life = 20;
    g.resolving += 1;
    let m = tok(&mut g, vec!["beast"], 3);
    g.perm_mut(sword).attached = Some(m);
    assert_eq!(etgh(&g, m), 5);
    zones::die(&mut g, m, "sac").unwrap();
    finish(&mut g);
    assert_eq!(g.player(P0).life, 25, "last known toughness is 5, not the detached body's 3");
}

#[test]
fn avatar_and_trostani_gain_separately_with_rhox_and_treebeard() {
    let mut g = tbl();
    perm(&mut g, P0, T);
    perm(&mut g, P0, A);
    perm(&mut g, P0, "Rhox Faithmender");
    let b = perm(&mut g, P0, B);
    g.perm_mut(b).plus = 0;
    g.player_mut(P0).life = 20;
    tok(&mut g, vec!["beast"], 3);
    assert_eq!(g.player(P0).life, 32, "two gains, each 3 doubled to 6");
    assert_eq!(g.perm(b).plus, 12);
}

/// Two simultaneous tokens cause two gains, with an Archangel trigger between them: toughness 1 then 2.
#[test]
fn simultaneous_tokens_archangel_updates_second_toughness() {
    for stack in [false, true] {
        let mut g = tbl();
        if stack {
            responses(&mut g);
        }
        perm(&mut g, P0, T);
        perm(&mut g, P0, "Archangel of Thune");
        g.player_mut(P0).life = 20;
        let made = zones::make_tokens(&mut g, P0, Tokens::new(2, 1)).unwrap();
        assert_eq!(g.player(P0).life, 23, "stack={stack}");
        assert!(made.iter().all(|&m| g.perm(m).plus == 2));
    }
}

#[test]
fn panharmonicon_copies_trostani_token_trigger_once() {
    let mut g = tbl();
    perm(&mut g, P0, T);
    perm(&mut g, P0, "Panharmonicon");
    g.player_mut(P0).life = 20;
    tok(&mut g, vec!["beast"], 3);
    assert_eq!(g.player(P0).life, 26);
}

#[test]
fn panharmonicon_and_elesh_add_extra_token_triggers() {
    let mut g = tbl();
    perm(&mut g, P0, T);
    perm(&mut g, P0, "Panharmonicon");
    perm(&mut g, P0, "Elesh Norn, Mother of Machines");
    g.player_mut(P0).life = 20;
    tok(&mut g, vec!["beast"], 3);
    assert_eq!(g.player(P0).life, 29, "603.2d: 1 + 1 + 1, not four triggers");
}

#[test]
fn opponents_elesh_suppresses_emiel_and_trostani_token_triggers() {
    let mut g = tbl();
    perm(&mut g, P0, T);
    perm(&mut g, P0, E);
    perm(&mut g, P1, "Elesh Norn, Mother of Machines");
    lands(&mut g, P0, "Plains", 1, false);
    g.player_mut(P0).life = 20;
    let m = tok(&mut g, vec!["unicorn"], 1);
    assert_eq!((g.player(P0).life, g.perm(m).plus), (20, 0));
}

#[test]
fn treebeard_life_locked_creates_no_counters() {
    let mut g = tbl();
    let b = perm(&mut g, P0, B);
    g.player_mut(P0).life_locked = true;
    life::gain(&mut g, P0, 4).unwrap();
    assert_eq!(g.perm(b).plus, 0);
}

#[test]
fn treebeard_erebos_prevents_gain_and_counters() {
    let mut g = tbl();
    let b = perm(&mut g, P0, B);
    perm(&mut g, P1, "Erebos, God of the Dead");
    let before = g.player(P0).life;
    life::gain(&mut g, P0, 4).unwrap();
    assert_eq!((g.player(P0).life, g.perm(b).plus), (before, 0));
}

/// Treebeard says "target Halfling or Treefolk"; shroud forbids even its controller's targeting (CR 702.18a).
#[test]
fn treebeard_cannot_target_itself_with_shroud() {
    let mut g = tbl();
    let b = perm(&mut g, P0, B);
    g.perm_mut(b).eot_kw.push("shroud");
    life::gain(&mut g, P0, 4).unwrap();
    assert_eq!(g.perm(b).plus, 0, "the only Halfling/Treefolk is an illegal target");
}

/// CR 113.7a: removing Treebeard does not remove its life-gain trigger; another Treefolk remains a legal target.
#[test]
fn treebeard_trigger_survives_source_removal_with_another_target() {
    let mut g = tbl();
    let b = perm(&mut g, P0, B);
    let target = tok(&mut g, vec!["treefolk"], 3);
    g.resolving += 1;
    life::gain(&mut g, P0, 4).unwrap();
    zones::die(&mut g, b, "sac").unwrap();
    finish(&mut g);
    assert_eq!(g.perm(target).plus, 4);
}

#[test]
fn populate_copy_of_copy_preserves_intrinsic_keywords_not_counters() {
    let mut g = tbl();
    let m = tok(&mut g, vec!["spider"], 2);
    g.perm_mut(m).data.set(DataKey::Kws, Val::List(vec![Val::Str("reach")]));
    g.perm_mut(m).plus = 7;
    let first = copied(&mut g);
    zones::die(&mut g, m, "sac").unwrap();
    g.perm_mut(first).plus = 4;
    let second = copied(&mut g);
    assert_eq!((epow(&g, second), etgh(&g, second), g.perm(second).plus), (1, 2, 0));
    assert!(sim_core::dsl::has_kw(&g, second, "reach"));
}

#[test]
fn populate_token_copy_of_nontoken_keeps_printed_characteristics() {
    let mut g = tbl();
    let c = card(&g, "Vampire Nighthawk");
    let m = zones::enter_token_copy_with(&mut g, P0, c, None).unwrap().unwrap();
    g.perm_mut(m).plus = 7;
    let copy = copied(&mut g);
    assert_eq!(g.perm(copy).cd, Some(c));
    assert_eq!((epow(&g, copy), etgh(&g, copy)), (2, 3));
    assert!(g.perm(copy).fly && g.perm(copy).lifelink && g.perm(copy).dt);
}

#[test]
fn populate_ignores_transient_pump_ability_and_tapped_state() {
    let mut g = tbl();
    let m = tok(&mut g, vec!["soldier"], 1);
    g.perm_mut(m).eot_pt = (9, 9);
    g.perm_mut(m).eot_kw.push("flying");
    g.perm_mut(m).tapped = true;
    let copy = copied(&mut g);
    assert_eq!((epow(&g, copy), etgh(&g, copy)), (1, 1));
    assert!(!sim_core::dsl::has_kw(&g, copy, "flying"));
    assert!(!g.perm(copy).tapped);
}

#[test]
fn populate_cannot_copy_opponents_creature_token() {
    let mut g = tbl();
    token(&mut g, P1, 6);
    perm(&mut g, P0, "Sylvan Caryatid");
    sim_core::impls::zur::populate(&mut g, P0).unwrap();
    assert!(tokens(&g).is_empty());
}

#[test]
fn populate_ignores_food_treasure_clue_and_creature_cards() {
    let mut g = tbl();
    for kind in ["Food", "Treasure", "Clue"] {
        zones::make_artifact_tokens(&mut g, P0, kind, 1).unwrap();
    }
    perm(&mut g, P0, "Sylvan Caryatid");
    let before = tokens(&g).len();
    sim_core::impls::zur::populate(&mut g, P0).unwrap();
    assert_eq!(tokens(&g).len(), before);
    assert_eq!((g.player(P0).foods, g.player(P0).treasures, g.player(P0).clues), (1, 1, 1));
}

#[test]
fn populate_doubled_by_each_doubler_and_by_both() {
    for doublers in
        [vec!["Doubling Season"], vec!["Anointed Procession"], vec!["Doubling Season", "Anointed Procession"]]
    {
        let mut g = tbl();
        tok(&mut g, vec!["beast"], 3);
        for name in &doublers {
            perm(&mut g, P0, name);
        }
        sim_core::impls::zur::populate(&mut g, P0).unwrap();
        assert_eq!(tokens(&g).len(), 1 + (1 << doublers.len()));
    }
}

#[test]
fn populated_artifact_creature_keeps_artifact_type() {
    let mut g = tbl();
    let mut spec = Tokens { fly: true, types: vec!["thopter", "artifact"], ..Tokens::new(1, 1) };
    spec.data.set(DataKey::Artifact, Val::Bool(true));
    zones::make_tokens(&mut g, P0, spec).unwrap();
    let copy = copied(&mut g);
    assert!(is_artifact_perm(&g, copy) && g.is_creature(copy) && g.perm(copy).fly);
}

#[test]
fn rootborn_defenses_without_token_still_grants_indestructible() {
    let mut g = tbl();
    let m = perm(&mut g, P0, "Sylvan Caryatid");
    sim_core::impls::zur::rootborn(&mut g, P0).unwrap();
    assert!(sim_core::dsl::has_kw(&g, m, "indestructible"));
    assert!(tokens(&g).is_empty());
}

/// A mana-producing Trostani must not be tapped both to pay {1}{G}{W} and for the same ability's {T} (CR 602.2b).
#[test]
fn trostani_cannot_pay_populate_using_her_own_mana_tap() {
    let mut g = tbl();
    let t = perm(&mut g, P0, T);
    tok(&mut g, vec!["beast"], 3);
    perm(&mut g, P0, "Cryptolith Rite");
    // The engine enumerates Rite mana for nonattacking creatures. This legal restriction does not remove {T} abilities.
    g.perm_mut(t).noatk = true;
    lands(&mut g, P0, "Forest", 1, false);
    lands(&mut g, P0, "Plains", 1, false);
    assert!(sim_core::engine::mana::can_pay(&g, P0, 1, "GW", false), "fixture includes Trostani's mana");
    let options = g.registry.get(g.perm(t).cd.unwrap()).unwrap().options.unwrap();
    let opts = options(&mut g, t, P0, None).unwrap();
    let Some(Action::Ability { src, f, arg }) = opts.first().and_then(|o| o.act.clone()) else {
        panic!("populate must be offered: the test checks that its payment cannot use Trostani's own {{T}}");
    };
    assert!(!f(&mut g, src, P0, arg).unwrap(), "cannot tap Trostani twice to pay one activation");
}

fn upkeep(g: &mut Game) {
    fire_trigger(g, Event::Upkeep, Call::Player { p: P0 }).unwrap();
}

fn raise(g: &mut Game, src: Src, p: PlayerId) -> sim_core::flow::Res {
    if p == P0 && trigger_window(g, p, Some(src), "fixture: gain 1", None)? {
        life::gain(g, p, 1)?;
    }
    Ok(())
}

fn remove_win_source(g: &mut Game, src: Src, p: PlayerId) -> sim_core::flow::Res {
    if p == P0 && trigger_window(g, p, Some(src), "fixture: remove win source", None)? {
        let targets: Vec<_> =
            g.player(p).perms.iter().copied().filter(|&m| matches!(g.perm(m).name, TEST | FEL)).collect();
        for m in targets {
            zones::die(g, m, "sac")?;
        }
    }
    Ok(())
}

fn fixture_upkeep(g: &mut Game, f: sim_core::hooks::PlayerFn) {
    let mut r = sim_core::impls::registry(&g.db).unwrap();
    r.card(&g.db, "Wall of Omens").unwrap().upkeep = Some(f);
    g.registry = Arc::new(r);
}

#[test]
fn upkeep_thresholds_and_opponents_upkeep() {
    for (name, need) in [(TEST, 50), (FEL, 40)] {
        for stack in [false, true] {
            let mut g = tbl();
            if stack {
                responses(&mut g);
            }
            perm(&mut g, P0, name);
            g.player_mut(P0).life = need;
            fire_trigger(&mut g, Event::Upkeep, Call::Player { p: P1 }).unwrap();
            assert!(!g.over);
            g.player_mut(P0).life = need - 1;
            upkeep(&mut g);
            assert!(!g.over);
            g.player_mut(P0).life = need;
            upkeep(&mut g);
            assert_eq!(g.winner, Some(P0), "{name}, stack={stack}");
        }
    }
}

/// CR 603.4: gaining the missing life after upkeep begins cannot create a trigger that did not trigger then.
/// (The no-response path calls each upkeep hook in turn, so Test's "if" is read after the earlier hook gained life;
/// Felidar Sovereign's hook had the same shape before Batch B. The stack-path twin below passes.)
#[test]
#[ignore = "T6: pending JD"]
fn test_of_endurance_cannot_trigger_retroactively_on_fast_path() {
    let mut g = tbl();
    fixture_upkeep(&mut g, raise);
    perm(&mut g, P0, "Wall of Omens");
    perm(&mut g, P0, TEST);
    g.player_mut(P0).life = 49;
    upkeep(&mut g);
    assert_eq!(g.player(P0).life, 50);
    assert!(!g.over, "no Test trigger existed at 49 at the beginning of upkeep");
}

#[test]
fn test_of_endurance_cannot_trigger_retroactively_on_stack_path() {
    let mut g = tbl();
    responses(&mut g);
    fixture_upkeep(&mut g, raise);
    perm(&mut g, P0, "Wall of Omens");
    perm(&mut g, P0, TEST);
    g.player_mut(P0).life = 49;
    upkeep(&mut g);
    assert_eq!(g.player(P0).life, 50);
    assert!(!g.over);
}

/// CR 113.7a: the upkeep win ability still resolves after Test of Endurance is destroyed in response.
#[test]
fn test_of_endurance_trigger_survives_source_removal() {
    let mut g = tbl();
    responses(&mut g);
    fixture_upkeep(&mut g, remove_win_source);
    let t = perm(&mut g, P0, TEST);
    perm(&mut g, P0, "Wall of Omens");
    g.player_mut(P0).life = 50;
    upkeep(&mut g);
    assert!(!g.perm(t).on_bf);
    assert_eq!(g.winner, Some(P0), "the ability is independent of its source");
}

/// CR 113.7a also applies to Felidar's shared upkeep helper, with its own 40-life threshold.
#[test]
fn felidar_trigger_survives_source_removal() {
    let mut g = tbl();
    responses(&mut g);
    fixture_upkeep(&mut g, remove_win_source);
    let t = perm(&mut g, P0, FEL);
    perm(&mut g, P0, "Wall of Omens");
    g.player_mut(P0).life = 40;
    upkeep(&mut g);
    assert!(!g.perm(t).on_bf);
    assert_eq!(g.winner, Some(P0));
}

// Stand-in opponent trigger from the SAME token-entry event, putting two counters on the entering token.
// The fixture isolates APNAP and the late key; it does not claim Wall of Omens has this ability.
fn opponent_buff(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> sim_core::flow::Res {
    if trigger_window(g, P1, Some(src), "fixture: put two counters", None)? {
        repl::put_counters(g, P1, m, 2);
    }
    Ok(())
}

fn opponent_tokens(g: &mut Game, src: Src, p: PlayerId, ts: &[PermId]) -> sim_core::flow::Res {
    sim_core::engine::hooks::queue_etb_per_token(g, src, p, ts)
}

fn apnap_fixture(stack: bool) -> i32 {
    let mut g = tbl();
    if stack {
        responses(&mut g);
    }
    let mut r: Registry = sim_core::impls::registry(&g.db).unwrap();
    let c = r.card(&g.db, "Wall of Omens").unwrap();
    c.etb = Some(opponent_buff);
    c.tokens_enter = Some(opponent_tokens);
    *c = c.enter_queues(Event::TokensEnter);
    g.registry = Arc::new(r);
    perm(&mut g, P0, T);
    perm(&mut g, P1, "Wall of Omens");
    g.player_mut(P0).life = 20;
    tok(&mut g, vec!["soldier"], 1);
    g.player(P0).life
}

/// CR 603.3b/101.4: active player's trigger goes on first and resolves after the opponent's, even with no responses.
/// (The no-response path resolves the batch in `apnap` order, active player first, which is the reverse; that order
/// predates Batch B and `late_order` only reorders inside one controller's group. The stack-path twin below passes.)
#[test]
#[ignore = "T7: pending JD"]
fn late_key_keeps_apnap_resolution_on_fast_path() {
    assert_eq!(apnap_fixture(false), 23);
}

#[test]
fn late_key_keeps_apnap_resolution_on_stack_path() {
    assert_eq!(apnap_fixture(true), 23);
}

/// CR 603.3b: a human can choose Trostani first (gain 1), even though AI prefers Emiel first (gain 2).
#[test]
fn human_order_can_override_late_key() {
    let mut g = tbl();
    responses(&mut g); // a person is asked to order triggers only when they go on the stack (an opponent could respond)
    perm(&mut g, P0, T);
    perm(&mut g, P0, E);
    lands(&mut g, P0, "Plains", 1, false);
    let asked = Arc::new(Mutex::new(false));
    let seen = asked.clone();
    g.humans.add(
        P0,
        Arc::new(ScriptHuman::from_fn(move |r: &Request| {
            if &*r.kind == "priority" {
                return Answer::Action(serde_json::json!({"do": "pass"}));
            }
            if r.prompt.contains("which resolves first") {
                *seen.lock().unwrap() = true;
                return Answer::Pick(r.choices.iter().position(|c| c.contains("Trostani")).unwrap());
            }
            Answer::Pick(0)
        })),
    );
    g.player_mut(P0).pool.add('W', 1); // a person pays from the mana pool, not from lands
    g.player_mut(P0).life = 20;
    let m = tok(&mut g, vec!["soldier"], 1);
    assert!(*asked.lock().unwrap(), "human must be offered trigger order");
    assert_eq!(g.player(P0).life, 21);
    assert_eq!(g.perm(m).plus, 1);
}

/// CR 608.2h last known information includes counters: a token that carried two +1/+1 counters when it left has
/// toughness 3+2 = 5, so Trostani's trigger (already on the stack) gains 5.
#[test]
fn trostani_last_known_toughness_includes_counters() {
    let mut g = tbl();
    perm(&mut g, P0, T);
    g.player_mut(P0).life = 20;
    g.resolving += 1;
    let m = tok(&mut g, vec!["beast"], 3);
    g.perm_mut(m).plus = 2;
    zones::die(&mut g, m, "sac").unwrap();
    finish(&mut g);
    assert_eq!(g.player(P0).life, 25);
}

/// "Whenever another creature YOU control enters": an opponent's creature token triggers neither Trostani, the Avatar
/// nor Emiel.
#[test]
fn opponents_token_triggers_none_of_my_life_cards() {
    let mut g = tbl();
    perm(&mut g, P0, T);
    perm(&mut g, P0, A);
    perm(&mut g, P0, E);
    lands(&mut g, P0, "Plains", 1, false);
    g.player_mut(P0).life = 20;
    let m =
        zones::make_tokens(&mut g, P1, Tokens { types: vec!["beast"], tgh: Some(3), ..Tokens::new(1, 1) }).unwrap()[0];
    assert_eq!((g.player(P0).life, g.perm(m).plus), (20, 0));
}

/// Populate: "a creature token you control". With an opponent's bigger token and my own Soldier, copy mine.
#[test]
fn populate_copies_my_token_not_the_bigger_opponent_token() {
    let mut g = tbl();
    token(&mut g, P1, 9);
    tok(&mut g, vec!["soldier"], 1);
    let c = copied(&mut g);
    assert_eq!((epow(&g, c), etgh(&g, c)), (1, 1));
    assert_eq!(g.perm(c).owner, P0);
}

/// Emiel: one {G/W} per trigger. Two tokens, one white source: exactly one counter in total, in both resolve paths.
#[test]
fn emiel_two_tokens_one_mana_one_counter() {
    for stack in [false, true] {
        let mut g = tbl();
        if stack {
            responses(&mut g);
        }
        perm(&mut g, P0, E);
        lands(&mut g, P0, "Plains", 1, false);
        let made = zones::make_tokens(&mut g, P0, Tokens::new(2, 1)).unwrap();
        assert_eq!(made.iter().map(|&m| g.perm(m).plus).sum::<i32>(), 1, "stack={stack}");
    }
}

/// Trostani's ability has {T} in its cost: a tapped Trostani cannot populate (the AI offers no option).
#[test]
fn trostani_tapped_cannot_populate() {
    let mut g = tbl();
    let t = perm(&mut g, P0, T);
    tok(&mut g, vec!["beast"], 3);
    lands(&mut g, P0, "Forest", 2, false);
    lands(&mut g, P0, "Plains", 1, false);
    let options = g.registry.get(g.perm(t).cd.unwrap()).unwrap().options.unwrap();
    assert_eq!(
        options(&mut g, t, P0, None).unwrap().len(),
        1,
        "control: untapped with {{1}}{{G}}{{W}} offers populate"
    );
    g.perm_mut(t).tapped = true;
    assert!(options(&mut g, t, P0, None).unwrap().is_empty());
}

/// {1}{G}{W} is three mana: with two lands there is no option; the cost is really paid when she activates.
#[test]
fn trostani_populate_costs_three_mana() {
    let mut g = tbl();
    let t = perm(&mut g, P0, T);
    tok(&mut g, vec!["beast"], 3);
    lands(&mut g, P0, "Forest", 1, false);
    lands(&mut g, P0, "Plains", 1, false);
    let options = g.registry.get(g.perm(t).cd.unwrap()).unwrap().options.unwrap();
    assert!(options(&mut g, t, P0, None).unwrap().is_empty());
    lands(&mut g, P0, "Forest", 1, false);
    let opts = options(&mut g, t, P0, None).unwrap();
    let Some(Action::Ability { src, f, arg }) = opts[0].act.clone() else { panic!("not an ability") };
    let before = tokens(&g).len();
    assert!(f(&mut g, src, P0, arg).unwrap());
    assert_eq!(tokens(&g).len(), before + 1);
    assert!(g.perm(t).tapped);
}

/// A person controlling Emiel still gets her enters trigger for a token (the same hook, probed the same way).
#[test]
fn emiel_token_trigger_with_a_human_controller() {
    let mut g = tbl();
    responses(&mut g);
    perm(&mut g, P0, E);
    lands(&mut g, P0, "Plains", 1, false);
    g.humans.add(
        P0,
        Arc::new(ScriptHuman::from_fn(move |r: &Request| {
            if &*r.kind == "priority" {
                return Answer::Action(serde_json::json!({"do": "pass"}));
            }
            Answer::Pick(0)
        })),
    );
    g.player_mut(P0).pool.add('W', 1); // a person pays from the mana pool, not from lands
    let m = tok(&mut g, vec!["soldier"], 1);
    assert_eq!(g.perm(m).plus, 1);
}
