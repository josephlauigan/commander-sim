// Written by an independent reviewer (the yoshimaru-adv adversarial pass), adopted unchanged except where noted.
//! Adversarial tests for branch yoshimaru-ai: the yoshimaru seat's Equipment carrier choice
//! (`tymna_cards::equip_holder_value` / `equip_utility_value`), the holder stat (`note_equip_holder`, called from
//! `dsl::equip`) and the Court of Ardenvale AI pick. Hand-built boards; no games are played.

use sim_core::engine::values::epow;
use sim_core::engine::zones::{Enter, enter};
use sim_core::hooks::{Action, Call, Event};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::tymna_cards as tc;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

const STING: &str = "Sting, the Glinting Dagger";
const SOHH: &str = "Sword of Hearth and Home";
const SOFI: &str = "Sword of Fire and Ice";
const SOFF: &str = "Sword of Feast and Famine";
const BUSTER: &str = "Buster Sword";
const COAT: &str = "Mithril Coat";
const BOOTS: &str = "Swiftfoot Boots";
const BALLISTA: &str = "Walking Ballista";
const YOSHI: &str = "Yoshimaru, Ever Faithful";
const DELNEY: &str = "Delney, Streetwise Lookout";

fn tbl() -> Game {
    let mut g = table(&["yoshimaru", "seph", "sauron"]);
    let db = g.db.clone();
    g.player_mut(P0).library.retain(|&c| !db.get(c).has_subtype("equipment"));
    g.active = Some(P0);
    for q in [P1, P2] {
        token(&mut g, q, 2);
    }
    g
}

fn ballista_with(g: &mut Game, counters: i32) -> PermId {
    let c = take(g, P0, BALLISTA);
    let m = enter(g, P0, c, Enter { sick: false, x: counters, plus: counters, ..Enter::default() }).unwrap();
    assert!(g.perm(m).on_bf);
    m
}

fn target_of(g: &Game, e: PermId) -> Option<PermId> {
    sim_core::dsl::equip_options(g, P0).into_iter().find_map(|o| match o.act {
        Some(Action::Equip { src, target, .. }) if src == e => Some(target),
        _ => None,
    })
}

fn utility_of(g: &Game, e: PermId) -> Option<f64> {
    sim_core::dsl::equip_options(g, P0)
        .into_iter()
        .find(|o| matches!(o.act, Some(Action::Equip { src, .. }) if src == e))
        .map(|o| o.utility)
}

// ------------------------------------------------------------------ other seats unchanged

/// every deck key (yours, the pools, the retired test seats) except yoshimaru and kraum (which has its own formula):
/// `equip_holder_value` and `equip_utility_value` are exactly 0.0 on states where the yoshimaru arm would be non-zero
/// (Ballista with Sword of Hearth and Home, a flier, Yoshimaru-like commander, Delney out)
#[test]
fn every_other_deck_key_gets_exactly_zero_from_both_hooks() {
    let mut bad = vec![];
    for d in decks() {
        if d.key == "yoshimaru" || d.key == "kraum" {
            continue;
        }
        let mut g = table(&[d.key.as_str(), "sauron", "seph"]);
        g.active = Some(P0);
        let b = {
            let c = take(&mut g, P0, BALLISTA);
            enter(&mut g, P0, c, Enter { sick: false, x: 2, plus: 2, ..Enter::default() }).unwrap()
        };
        let fl = perm(&mut g, P0, "Serra Angel");
        let t = token(&mut g, P0, 2);
        perm(&mut g, P0, DELNEY);
        for gear in [SOHH, SOFI, STING, BOOTS, BUSTER] {
            let e = perm(&mut g, P0, gear);
            for m in [b, fl, t] {
                let (v, u) = (tc::equip_holder_value(&g, P0, e, m), tc::equip_utility_value(&g, P0, e, m));
                if v != 0.0 || u != 0.0 {
                    bad.push(format!("{}: {gear} on {}: value {v} utility {u}", d.key, g.perm(m).name));
                }
            }
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

/// the yoshimaru seat and another seat in one game: the same board state scores differently only for the yoshimaru seat
#[test]
fn a_cloud_seat_in_the_same_game_as_a_yoshimaru_seat_is_unaffected() {
    let mut g = table(&["yoshimaru", "cloud", "seph"]);
    g.active = Some(P0);
    let b0 = {
        let c = take(&mut g, P0, BALLISTA);
        enter(&mut g, P0, c, Enter { sick: false, x: 2, plus: 2, ..Enter::default() }).unwrap()
    };
    let b1 = {
        let c = take(&mut g, P1, BALLISTA);
        enter(&mut g, P1, c, Enter { sick: false, x: 2, plus: 2, ..Enter::default() }).unwrap()
    };
    let e0 = perm(&mut g, P0, SOHH);
    let e1 = perm(&mut g, P1, SOHH);
    assert!(tc::equip_holder_value(&g, P0, e0, b0) < -100.0);
    assert!(tc::equip_utility_value(&g, P0, e0, b0) < 0.0);
    assert_eq!(tc::equip_holder_value(&g, P1, e1, b1), 0.0);
    assert_eq!(tc::equip_utility_value(&g, P1, e1, b1), 0.0);
    // the other seat's stat is untouched when its equip resolves
    g.perm_mut(e1).attached = Some(b1);
    tc::note_equip_holder(&mut g, P1, e1, b1);
    assert!(g.player(P1).stats.keys().all(|k| !k.starts_with("ty_eq_")));
}

// ------------------------------------------------------------------ note_equip_holder robustness

#[test]
fn note_equip_holder_does_not_panic_when_the_holder_died() {
    let mut g = tbl();
    let m = token(&mut g, P0, 2);
    let e = perm(&mut g, P0, SOFI);
    g.perm_mut(e).attached = Some(m);
    sim_core::engine::zones::leave(&mut g, m).unwrap();
    tc::note_equip_holder(&mut g, P0, e, m); // must not panic whatever it counts
}

#[test]
fn note_equip_holder_does_not_panic_when_the_equipment_left() {
    let mut g = tbl();
    let m = token(&mut g, P0, 2);
    let e = perm(&mut g, P0, SOFI);
    g.perm_mut(e).attached = Some(m);
    sim_core::engine::zones::leave(&mut g, e).unwrap();
    tc::note_equip_holder(&mut g, P0, e, m);
}

/// the real `dsl::equip` path: the stat moves for the yoshimaru seat only and exactly once per resolved equip
#[test]
fn the_real_equip_action_counts_once() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 6, false);
    let m = token(&mut g, P0, 2);
    let e = perm(&mut g, P0, SOFI);
    assert!(sim_core::dsl::equip(&mut g, P0, e, m, 2).unwrap());
    assert_eq!(g.perm(e).attached, Some(m));
    let n: i64 = g.player(P0).stats.iter().filter(|(k, _)| k.starts_with("ty_eq_")).map(|(_, v)| *v).sum();
    assert_eq!(n, 1, "one resolved equip is one count");
    // the same holder again (already attached): the equip ability does nothing new, but is it counted again?
    let before = n;
    let _ = sim_core::dsl::equip(&mut g, P0, e, m, 2).unwrap();
    let n2: i64 = g.player(P0).stats.iter().filter(|(k, _)| k.starts_with("ty_eq_")).map(|(_, v)| *v).sum();
    println!("re-equip of the same holder: count {before} -> {n2}");
}

#[test]
fn the_real_equip_action_for_another_seat_counts_nothing() {
    let mut g = table(&["seph", "yoshimaru", "sauron"]);
    g.active = Some(P0);
    lands(&mut g, P0, "Plains", 6, false);
    let m = token(&mut g, P0, 2);
    let e = perm(&mut g, P0, SOFI);
    assert!(sim_core::dsl::equip(&mut g, P0, e, m, 2).unwrap());
    assert!(g.player(P0).stats.keys().all(|k| !k.starts_with("ty_eq_")));
    assert!(g.player(P1).stats.keys().all(|k| !k.starts_with("ty_eq_")));
}

// ------------------------------------------------------------------ scoring edges

#[test]
#[ignore = "arm C: weight removed, see program-decisions 2026-11-23"]
fn yoshimaru_who_flies_scores_flier_plus_connect_minus_four() {
    let mut g = tbl();
    for q in [P1, P2] {
        // remove the blockers so he connects
        let ps: Vec<PermId> = g.player(q).perms.clone();
        for m in ps {
            sim_core::engine::zones::leave(&mut g, m).unwrap();
        }
    }
    let y = perm(&mut g, P0, YOSHI);
    let e = perm(&mut g, P0, STING);
    let base = tc::equip_holder_value(&g, P0, e, y);
    g.perm_mut(y).fly = true;
    let fly = tc::equip_holder_value(&g, P0, e, y);
    assert!((fly - base - 2.0).abs() < 1e-9, "a flying Yoshimaru gains exactly the flier bonus: {base} -> {fly}");
    assert!((base - (1.5 - 4.0)).abs() < 1e-9, "unblocked +1.5, Yoshimaru -4: got {base}");
}

#[test]
fn an_opponents_delney_does_not_penalise_us() {
    let mut g = tbl();
    let two = token(&mut g, P0, 2);
    let sword = perm(&mut g, P0, SOFI);
    let before = tc::equip_holder_value(&g, P0, sword, two);
    perm(&mut g, P1, DELNEY);
    assert_eq!(tc::equip_holder_value(&g, P0, sword, two), before);
}

#[test]
#[ignore = "arm C: weight removed, see program-decisions 2026-11-23"]
fn a_phased_out_or_dead_delney_does_not_penalise() {
    let mut g = tbl();
    let two = token(&mut g, P0, 2);
    let sword = perm(&mut g, P0, SOFI);
    let before = tc::equip_holder_value(&g, P0, sword, two);
    let d = perm(&mut g, P0, DELNEY);
    assert!(tc::equip_holder_value(&g, P0, sword, two) < before, "Delney out: the penalty applies");
    g.perm_mut(d).phased = true;
    assert_eq!(tc::equip_holder_value(&g, P0, sword, two), before, "phased out");
    g.perm_mut(d).phased = false;
    sim_core::engine::zones::leave(&mut g, d).unwrap();
    assert_eq!(tc::equip_holder_value(&g, P0, sword, two), before, "dead");
}

/// Delney herself wears the Sword: her own doubling and unblockability stop applying to her as soon as she is over
/// power 2, exactly like any other power-2 creature. The code skips the penalty for `d == m`.
#[test]
fn delney_herself_as_the_holder_is_penalised_like_any_power_two_creature() {
    let mut g = tbl();
    let d = perm(&mut g, P0, DELNEY);
    let other = token(&mut g, P0, 2);
    let sword = perm(&mut g, P0, SOFI);
    println!("Delney power {} / other power {}", epow(&g, d), epow(&g, other));
    assert!(epow(&g, d) <= 2, "this test needs Delney at power 2 or less");
    // all else equal (neither flies, both blocked by the 2-power blockers): the two holders must score alike
    let (vd, vo) = (tc::equip_holder_value(&g, P0, sword, d), tc::equip_holder_value(&g, P0, sword, other));
    assert_eq!(vd, vo, "Delney as holder {vd}, a power-2 token {vo}");
}

#[test]
fn equipment_without_a_power_bonus_never_triggers_the_delney_rule() {
    let mut g = tbl();
    let two = token(&mut g, P0, 2);
    let probes: Vec<PermId> = [BOOTS, COAT].iter().map(|n| perm(&mut g, P0, n)).collect();
    let before: Vec<f64> = probes.iter().map(|&e| tc::equip_holder_value(&g, P0, e, two)).collect();
    perm(&mut g, P0, DELNEY);
    let after: Vec<f64> = probes.iter().map(|&e| tc::equip_holder_value(&g, P0, e, two)).collect();
    assert_eq!(before, after);
}

#[test]
#[ignore = "arm C: weight removed, see program-decisions 2026-11-23"]
fn buster_sword_and_the_other_swords_are_all_penalised_on_a_power_two_creature_under_delney() {
    let mut g = tbl();
    let two = token(&mut g, P0, 2);
    let gear: Vec<(&str, PermId)> =
        [BUSTER, SOFI, SOFF, SOHH, STING].iter().map(|n| (*n, perm(&mut g, P0, n))).collect();
    let before: Vec<f64> = gear.iter().map(|&(_, e)| tc::equip_holder_value(&g, P0, e, two)).collect();
    perm(&mut g, P0, DELNEY);
    for (i, &(n, e)) in gear.iter().enumerate() {
        let after = tc::equip_holder_value(&g, P0, e, two);
        println!("{n}: {} -> {after}", before[i]);
        assert!((after - before[i] + 4.0).abs() < 1e-9, "{n}: before {} after {after}", before[i]);
    }
}

/// a worn Equipment is already inside the creature's power: scoring the current holder must not count its bonus twice
/// (a base-power-0 holder with a +2 Sword is at power 2; the code adds the bonus again and docks the Delney rule)
#[test]
fn a_worn_equipment_is_not_counted_twice_for_its_current_holder() {
    let mut g = tbl();
    let spec =
        sim_core::engine::zones::Tokens { sick: false, tgh: Some(3), ..sim_core::engine::zones::Tokens::new(1, 0) };
    let zero = sim_core::engine::zones::make_tokens(&mut g, P0, spec).unwrap()[0];
    let sword = perm(&mut g, P0, SOFI);
    g.perm_mut(sword).attached = Some(zero);
    assert_eq!(epow(&g, zero), 2, "the Sword is on: power 2");
    let before = tc::equip_holder_value(&g, P0, sword, zero);
    perm(&mut g, P0, DELNEY);
    let after = tc::equip_holder_value(&g, P0, sword, zero);
    assert_eq!(before, after, "the worn Sword stays at power 2; Delney changes nothing for it ({before} -> {after})");
}

/// `Perm.owner` is the CONTROLLER in this engine (`orig` is the owner), so `yoshimaru_holder_value`'s
/// `g.perm(e).owner` is the seat; a stolen Equipment (orig = another seat) scores like our own. Expected to hold.
#[test]
fn equipment_we_control_but_do_not_own_is_scored_against_our_board() {
    let mut g = tbl();
    let two = token(&mut g, P0, 2);
    perm(&mut g, P0, DELNEY);
    let sword = perm(&mut g, P0, SOFI);
    let ours = tc::equip_holder_value(&g, P0, sword, two);
    g.perm_mut(sword).orig = P1; // owned by P1, controlled by P0
    let stolen = tc::equip_holder_value(&g, P0, sword, two);
    assert_eq!(ours, stolen, "ours {ours}, stolen {stolen}");
}

#[test]
fn the_veto_is_by_data_not_by_counters_or_count() {
    for counters in [1, 2, 6] {
        let mut g = tbl();
        let b = ballista_with(&mut g, counters);
        let e = perm(&mut g, P0, SOHH);
        assert!(tc::equip_holder_value(&g, P0, e, b) < -100.0, "{counters} counters");
    }
    // two Ballistas and a ground creature: the creature wins
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 6, false);
    let (b1, b2) = (ballista_with(&mut g, 2), ballista_with(&mut g, 2));
    let ground = token(&mut g, P0, 1);
    let e = perm(&mut g, P0, SOHH);
    assert_eq!(target_of(&g, e), Some(ground));
    // two Ballistas alone: the option's utility is negative, never positive
    sim_core::engine::zones::leave(&mut g, ground).unwrap();
    let u = utility_of(&g, e).expect("an option exists");
    assert!(u < 0.0, "two Ballistas only: utility {u}");
    let _ = (b1, b2);
}

/// Ballista is the only creature: the vetoed Sword's option is negative, but a Sword that is fine on Ballista keeps a
/// positive option beside it (the veto must not make the AI skip every equip)
#[test]
fn a_vetoed_sword_does_not_suppress_the_other_equipments_options() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 8, false);
    let b = ballista_with(&mut g, 2);
    let hh = perm(&mut g, P0, SOHH);
    let fi = perm(&mut g, P0, SOFI);
    let st = perm(&mut g, P0, STING);
    assert!(utility_of(&g, hh).unwrap() < 0.0);
    assert!(utility_of(&g, fi).unwrap() > 0.0);
    assert!(utility_of(&g, st).unwrap() > 0.0);
    assert_eq!(target_of(&g, fi), Some(b));
}

/// a Sword already on Ballista (put there by a free attach): can the AI move it to a better holder?
#[test]
#[ignore = "ticket T-S3-1: a worn Equipment is never moved (no [equip] moves); needs an [equip] section"]
fn latent_hearth_and_home_already_on_ballista_is_never_moved_off() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 8, false);
    let b = ballista_with(&mut g, 2);
    let ground = token(&mut g, P0, 2);
    let e = perm(&mut g, P0, SOHH);
    g.perm_mut(e).attached = Some(b);
    let t = target_of(&g, e);
    assert_eq!(t, Some(ground), "the equip option should move the Sword off Ballista (got {t:?})");
}

/// Kor Outfitter's ETB attach used Cloud's holder_score, not equip_holder_value: it put Hearth and Home on Ballista.
/// The yoshimaru seat now scores the (Equipment, creature) pair (`cloud::yoshimaru_attach`): not on Ballista when
/// another creature exists.
#[test]
fn kor_outfitter_free_attach_does_not_put_hearth_and_home_on_ballista() {
    let mut g = tbl();
    let b = ballista_with(&mut g, 3);
    let ground = token(&mut g, P0, 1);
    let e = perm(&mut g, P0, SOHH);
    perm(&mut g, P0, "Kor Outfitter");
    assert_ne!(g.perm(e).attached, Some(b), "Hearth and Home must not end up on Walking Ballista");
    // Kor Outfitter (2/2) is itself a creature on the board, so the Sword goes on it or on the token, not on Ballista
    let on = g.perm(e).attached.expect("it is attached to another creature");
    assert!(on == ground || g.perm(on).name == "Kor Outfitter", "attached to {}", g.perm(on).name);
}

// ------------------------------------------------------------------ Court of Ardenvale

fn court_table() -> Game {
    let mut g = tbl();
    perm(&mut g, P0, "Court of Ardenvale");
    assert_eq!(g.monarch, Some(P0));
    g
}
fn gy_add(g: &mut Game, names: &[&str]) {
    for n in names {
        let c = take(g, P0, n);
        g.player_mut(P0).gy.push(c);
    }
}
fn on(g: &Game, name: &str) -> bool {
    g.player(P0).perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).name == name)
}
fn in_gy(g: &Game, name: &str) -> bool {
    g.player(P0).gy.iter().any(|&c| &*g.db.get(c).name == name)
}
fn in_hand(g: &Game, name: &str) -> bool {
    g.player(P0).hand.iter().any(|&c| &*g.db.get(c).name == name)
}
fn upkeep(g: &mut Game) {
    sim_core::engine::hooks::fire_trigger(g, Event::Upkeep, Call::Player { p: P0 }).unwrap();
}

#[test]
fn court_monarch_with_land_and_ballista_takes_the_land_only() {
    let mut g = court_table();
    gy_add(&mut g, &[BALLISTA, "Plains"]);
    upkeep(&mut g);
    assert!(in_gy(&g, BALLISTA) && !in_gy(&g, "Plains"));
}

#[test]
fn court_monarch_with_ballista_and_a_normal_three_drop_takes_the_three_drop() {
    let mut g = court_table();
    gy_add(&mut g, &[BALLISTA, "Flickerwisp"]);
    upkeep(&mut g);
    assert!(on(&g, "Flickerwisp"), "the normal 3-drop returns");
    assert!(in_gy(&g, BALLISTA));
}

#[test]
fn court_monarch_never_returns_another_x_cost_card() {
    let mut g = court_table();
    if g.db.id("Hangarback Walker").is_none() {
        eprintln!("no Hangarback Walker in the database; skipped");
        return;
    }
    gy_add(&mut g, &["Hangarback Walker"]);
    upkeep(&mut g);
    assert!(in_gy(&g, "Hangarback Walker"), "an X-cost creature would enter with X = 0 and die");
}

#[test]
fn court_monarch_with_two_x_cards_and_a_land_returns_the_land() {
    let mut g = court_table();
    gy_add(&mut g, &[BALLISTA, BALLISTA, "Plains"]);
    upkeep(&mut g);
    assert!(!in_gy(&g, "Plains") && g.player(P0).gy.len() == 2);
}

/// not the monarch: the pick is the old one (nonland by worth, Ballista allowed), to hand
#[test]
fn court_without_the_crown_picks_exactly_as_the_old_code_did() {
    // with Ballista as the only card, to hand (author's test); with a competing 3-drop the same pick as the monarch's
    // pick WITHOUT Ballista in the pool means worth ordering is unchanged for the non-X cards
    let mut a = court_table();
    a.monarch = Some(P1);
    gy_add(&mut a, &[BALLISTA, "Flickerwisp", "Mind Stone"]);
    upkeep(&mut a);
    let mut b = court_table();
    b.monarch = Some(P1);
    gy_add(&mut b, &["Flickerwisp", "Mind Stone"]);
    upkeep(&mut b);
    let pick = |g: &Game| ["Flickerwisp", "Mind Stone", BALLISTA].into_iter().find(|n| in_hand(g, n));
    println!("with Ballista in the yard: {:?}; without: {:?}", pick(&a), pick(&b));
    assert!(pick(&a).is_some() && pick(&b).is_some());
    // exactly one card returned in each
    assert_eq!(a.player(P0).hand.len(), b.player(P0).hand.len());
}

/// only X-cost and a nonlegal-for-monarch situation: the trigger consumed no card and left the graveyard intact
#[test]
fn court_returning_nothing_leaves_the_graveyard_and_hand_untouched() {
    let mut g = court_table();
    gy_add(&mut g, &[BALLISTA]);
    let (h, y) = (g.player(P0).hand.len(), g.player(P0).gy.len());
    upkeep(&mut g);
    assert_eq!((g.player(P0).hand.len(), g.player(P0).gy.len()), (h, y));
}
