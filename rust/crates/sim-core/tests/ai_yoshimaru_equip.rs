//! The yoshimaru seat's Equipment carrier choice (`tymna_cards::equip_holder_value`, read by `dsl::equip_options`), its
//! holder stat (`tymna_cards::note_equip_holder`) and the Court of Ardenvale AI pick (`legends::court`).
//! Hand-built boards; no games are played.

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
const BOOTS: &str = "Swiftfoot Boots";
const BALLISTA: &str = "Walking Ballista";
const TYMNA: &str = "Tymna the Weaver";
const YOSHI: &str = "Yoshimaru, Ever Faithful";
const DELNEY: &str = "Delney, Streetwise Lookout";
const CLOUD: &str = "Cloud, Midgar Mercenary";

/// P0 is the yoshimaru seat, with no Equipment left in its library (an entering Stoneforge or Cloud would fetch one);
/// P1 and P2 each have a 2-power blocker, so a ground creature of ours does not connect
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

/// Walking Ballista with two +1/+1 counters (it enters as a 0/0 and would die with X = 0)
fn ballista(g: &mut Game) -> PermId {
    let c = take(g, P0, BALLISTA);
    let m = enter(g, P0, c, Enter { sick: false, x: 2, plus: 2, ..Enter::default() }).unwrap();
    assert!(g.perm(m).on_bf, "Walking Ballista with counters survives");
    m
}

/// the creature the generic equip option would put `e` on
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

#[test]
fn hearth_and_home_never_chooses_ballista_while_another_holder_exists() {
    for sword in [SOHH] {
        let mut g = tbl();
        lands(&mut g, P0, "Plains", 6, false);
        let b = ballista(&mut g);
        let ground = token(&mut g, P0, 1);
        let e = perm(&mut g, P0, sword);
        assert_eq!(target_of(&g, e), Some(ground), "{sword}: the other creature, not Ballista");
        assert!(tc::equip_holder_value(&g, P0, e, b) < -100.0, "{sword}: Ballista is vetoed");
    }
}

#[test]
fn the_veto_beats_even_the_penalised_holders() {
    // Ballista against Yoshimaru (penalised) as the only candidates: Yoshimaru takes it
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 6, false);
    let b = ballista(&mut g);
    let y = perm(&mut g, P0, YOSHI);
    let e = perm(&mut g, P0, SOHH);
    assert_eq!(target_of(&g, e), Some(y));
    assert!(tc::equip_holder_value(&g, P0, e, y) > tc::equip_holder_value(&g, P0, e, b));
}

#[test]
fn ballista_alone_gets_no_worthwhile_equip_option_for_hearth_and_home() {
    for sword in [SOHH] {
        let mut g = tbl();
        lands(&mut g, P0, "Plains", 6, false);
        ballista(&mut g);
        let e = perm(&mut g, P0, sword);
        let u = utility_of(&g, e).expect("an option is built");
        assert!(u < 0.0, "{sword}: utility {u} is below doing nothing");
    }
}

#[test]
fn ballista_still_takes_equipment_without_protection_from_white() {
    // Sting (+1/+1, haste, untap) has no protection from white: Ballista is an ordinary holder for it, as for a Sword
    for gear in [SOFI, STING] {
        let mut g = tbl();
        lands(&mut g, P0, "Plains", 6, false);
        let b = ballista(&mut g);
        let e = perm(&mut g, P0, gear);
        assert_eq!(target_of(&g, e), Some(b), "{gear}");
        assert!(tc::equip_holder_value(&g, P0, e, b) > -100.0, "{gear}");
        assert!(utility_of(&g, e).unwrap() > 0.0, "{gear}");
    }
}

#[test]
fn tymna_is_not_a_holder_for_a_sword_but_is_for_swiftfoot_boots() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 6, false);
    let t = perm(&mut g, P0, TYMNA);
    let ground = token(&mut g, P0, 2);
    let sword = perm(&mut g, P0, SOFI);
    let boots = perm(&mut g, P0, BOOTS);
    assert_eq!(target_of(&g, sword), Some(ground), "the Sword goes on the other creature");
    // Boots: a four-point bonus on Tymna (the guide's carrier piece for her), the six-point penalty on a Sword
    let (vb, vs) = (tc::equip_holder_value(&g, P0, boots, t), tc::equip_holder_value(&g, P0, sword, t));
    assert!((vb - vs - 10.0).abs() < 1e-9, "Boots {vb}, Sword {vs}");
    // and with the Boots she beats Yoshimaru, who is penalised for any Equipment
    let y = perm(&mut g, P0, YOSHI);
    assert!(tc::equip_holder_value(&g, P0, boots, t) > tc::equip_holder_value(&g, P0, boots, y));
}

#[test]
fn a_flier_beats_a_bigger_ground_creature() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 6, false);
    let ground = token(&mut g, P0, 3);
    let flier = perm(&mut g, P0, "Serra Angel");
    let e = perm(&mut g, P0, SOFI);
    assert_eq!(target_of(&g, e), Some(flier));
    assert!(tc::equip_holder_value(&g, P0, e, flier) > tc::equip_holder_value(&g, P0, e, ground));
}

#[test]
fn yoshimaru_is_not_chosen_when_a_flier_is_there() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 6, false);
    let y = perm(&mut g, P0, YOSHI);
    let flier = perm(&mut g, P0, "Serra Angel");
    let e = perm(&mut g, P0, SOFI);
    assert_eq!(target_of(&g, e), Some(flier));
    assert!(tc::equip_holder_value(&g, P0, e, y) < tc::equip_holder_value(&g, P0, e, flier));
}

#[test]
fn delney_penalises_power_that_takes_a_creature_over_two() {
    let mut g = tbl();
    let (two, one, big, cloud) =
        (token(&mut g, P0, 2), token(&mut g, P0, 1), token(&mut g, P0, 3), perm(&mut g, P0, CLOUD));
    let sword = perm(&mut g, P0, SOFI); // +2/+2
    let sting = perm(&mut g, P0, STING); // +1/+1
    let probe = |g: &Game| {
        [
            tc::equip_holder_value(g, P0, sword, two),
            tc::equip_holder_value(g, P0, sword, big),
            tc::equip_holder_value(g, P0, sword, cloud),
            tc::equip_holder_value(g, P0, sting, one),
            tc::equip_holder_value(g, P0, sting, two),
        ]
    };
    assert_eq!(epow(&g, cloud), 2, "the test relies on Cloud being a power-2 creature");
    let before = probe(&g);
    perm(&mut g, P0, DELNEY);
    let after = probe(&g);
    assert!((after[0] - before[0] + 4.0).abs() < 1e-9, "a Sword on power 2 loses Delney's doubling");
    assert_eq!(after[1], before[1], "power 3 is already past the limit");
    assert_eq!(after[2], before[2], "Cloud is the exception (the guide's best carrier)");
    assert_eq!(after[3], before[3], "Sting on a power-1 creature leaves power 2");
    assert!((after[4] - before[4] + 4.0).abs() < 1e-9, "Sting on power 2 takes it to 3");
}

#[test]
fn the_other_seats_equip_values_are_unchanged() {
    // cloud and an unrelated deck: zero, as before; the kraum seat: its own formula, no veto and no yoshimaru term
    for key in ["cloud", "seph"] {
        let mut g = table(&[key, "sauron", "sauron"]);
        g.active = Some(P0);
        let b = {
            let c = take(&mut g, P0, "Mother of Runes");
            enter(&mut g, P0, c, Enter { sick: false, ..Enter::default() }).unwrap()
        };
        let e = perm(&mut g, P0, STING);
        let t = token(&mut g, P0, 2);
        for m in [b, t] {
            assert_eq!(tc::equip_holder_value(&g, P0, e, m), 0.0, "{key}");
            assert_eq!(tc::equip_utility_value(&g, P0, e, m), 0.0, "{key}");
        }
    }
    let mut g = table(&["thrasios", "cloud", "seph"]);
    g.active = Some(P0);
    let e = perm(&mut g, P0, STING);
    let t = token(&mut g, P0, 2);
    let v = tc::equip_holder_value(&g, P0, e, t);
    assert!(v > -100.0 && tc::equip_utility_value(&g, P0, e, t) >= -3.0);
}

#[test]
fn the_holder_stat_counts_who_wears_the_equipment_for_the_yoshimaru_seat_only() {
    let mut g = tbl();
    let (t, flier, ground, b) =
        (perm(&mut g, P0, TYMNA), perm(&mut g, P0, "Serra Angel"), token(&mut g, P0, 2), ballista(&mut g));
    g.perm_mut(t).is_cmd = true;
    let e = perm(&mut g, P0, SOFI);
    let stat = |g: &Game, k: &str| g.player(P0).stats.get(k).copied().unwrap_or(0);
    for (m, key) in [(t, "ty_eq_commander"), (flier, "ty_eq_flier"), (ground, "ty_eq_other"), (b, "ty_eq_ballista")] {
        g.perm_mut(e).attached = Some(m);
        tc::note_equip_holder(&mut g, P0, e, m);
        assert_eq!(stat(&g, key), 1, "{key}");
    }
    assert_eq!(stat(&g, "ty_eq_pw_ballista"), 0, "Sword of Fire and Ice gives no protection from white");
    let sting = perm(&mut g, P0, STING);
    g.perm_mut(sting).attached = Some(b);
    tc::note_equip_holder(&mut g, P0, sting, b);
    assert_eq!(stat(&g, "ty_eq_pw_ballista"), 0, "Sting has no protection from white");
    let s = perm(&mut g, P0, SOHH);
    g.perm_mut(s).attached = Some(b);
    tc::note_equip_holder(&mut g, P0, s, b);
    assert_eq!(stat(&g, "ty_eq_pw_ballista"), 1);
    // an equip that did not attach counts nothing; another seat's equip counts nothing
    let n = stat(&g, "ty_eq_other");
    g.perm_mut(e).attached = None;
    tc::note_equip_holder(&mut g, P0, e, ground);
    assert_eq!(stat(&g, "ty_eq_other"), n);
    let mut h = table(&["seph", "sauron"]);
    let (e, m) = (perm(&mut h, P0, SOFI), token(&mut h, P0, 2));
    h.perm_mut(e).attached = Some(m);
    tc::note_equip_holder(&mut h, P0, e, m);
    assert!(h.player(P0).stats.keys().all(|k| !k.starts_with("ty_eq_")));
}

// ------------------------------------------------------------------ Court of Ardenvale never returns Walking Ballista
fn gy_add(g: &mut Game, names: &[&str]) {
    for n in names {
        let c = take(g, P0, n);
        g.player_mut(P0).gy.push(c);
    }
}

fn court_table() -> Game {
    let mut g = tbl();
    perm(&mut g, P0, "Court of Ardenvale");
    assert_eq!(g.monarch, Some(P0));
    g
}

fn on(g: &Game, name: &str) -> bool {
    g.player(P0).perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).name == name)
}

fn in_gy(g: &Game, name: &str) -> bool {
    g.player(P0).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn upkeep(g: &mut Game) {
    sim_core::engine::hooks::fire_trigger(g, Event::Upkeep, Call::Player { p: P0 }).unwrap();
}

#[test]
fn court_as_the_monarch_returns_the_normal_permanent_and_leaves_ballista() {
    let mut g = court_table();
    gy_add(&mut g, &[BALLISTA, "Mind Stone"]);
    upkeep(&mut g);
    assert!(on(&g, "Mind Stone"), "the normal permanent returns");
    assert!(!on(&g, BALLISTA));
    assert!(in_gy(&g, BALLISTA), "Ballista stays in the graveyard");
}

#[test]
fn court_prefers_a_land_to_a_ballista_and_returns_nothing_when_only_ballista_qualifies() {
    let mut g = court_table();
    gy_add(&mut g, &[BALLISTA, "Plains"]);
    upkeep(&mut g);
    assert!(in_gy(&g, BALLISTA) && !in_gy(&g, "Plains"), "the land returns");
    let mut g = court_table();
    gy_add(&mut g, &[BALLISTA]);
    upkeep(&mut g);
    assert!(in_gy(&g, BALLISTA) && !on(&g, BALLISTA));
}

#[test]
fn court_without_the_crown_still_returns_ballista_to_hand() {
    let mut g = court_table();
    g.monarch = Some(P1);
    gy_add(&mut g, &[BALLISTA]);
    upkeep(&mut g);
    assert!(g.player(P0).hand.iter().any(|&c| &*g.db.get(c).name == BALLISTA));
    assert!(!on(&g, BALLISTA));
}

// ------------------------------------------------------------------ the free attach paths in the list (Kor Outfitter, Ardenn)
fn ballista_counters(g: &mut Game, p: PlayerId, n: i32) -> PermId {
    let c = take(g, p, BALLISTA);
    enter(g, p, c, Enter { sick: false, x: n, plus: n, ..Enter::default() }).unwrap()
}

fn combat_start(g: &mut Game) {
    sim_core::engine::hooks::fire_trigger(g, Event::CombatStart, Call::Player { p: P0 }).unwrap();
}

#[test]
fn ardenn_keeps_hearth_and_home_off_ballista_and_moves_exactly_the_kept_pieces() {
    let mut g = tbl();
    let b = ballista_counters(&mut g, P0, 4); // the biggest creature: Cloud's holder_score alone would pick it
    let ground = token(&mut g, P0, 1);
    let (sohh, sofi) = (perm(&mut g, P0, SOHH), perm(&mut g, P0, SOFI));
    let ardenn = perm(&mut g, P0, "Ardenn, Intrepid Archaeologist");
    combat_start(&mut g);
    // a creature that can wear both beats Ballista (which can wear only the Sword of Fire and Ice)
    assert_eq!(g.perm(sohh).attached, Some(ardenn));
    assert_eq!(g.perm(sofi).attached, Some(ardenn));
    assert_eq!((g.perm(b).attached, g.perm(ground).attached), (None, None));
}

#[test]
fn a_sword_is_not_put_on_tymna_by_ardenn_or_kor_outfitter_and_loose_boots_wait_for_her() {
    // loose Boots (first in the list) and Sword of Feast and Famine; Tymna is a legal target, a flier is there. The old
    // choice (best holder, first loose piece) put the Boots on the flier and left the Sword loose.
    for outfitter in [false, true] {
        let mut g = tbl();
        let t = perm(&mut g, P0, TYMNA);
        let flier = perm(&mut g, P0, "Serra Angel");
        let (boots, sword) = (perm(&mut g, P0, BOOTS), perm(&mut g, P0, "Sword of Feast and Famine"));
        if outfitter {
            perm(&mut g, P0, "Kor Outfitter");
        } else {
            perm(&mut g, P0, "Ardenn, Intrepid Archaeologist");
            combat_start(&mut g);
        }
        assert_eq!(g.perm(sword).attached, Some(flier), "outfitter={outfitter}: the Sword goes on the flier");
        assert_eq!(g.perm(boots).attached, None, "outfitter={outfitter}: the Boots wait for Tymna's equip");
        assert_ne!(g.perm(sword).attached, Some(t));
    }
}

#[test]
fn loose_boots_do_move_to_the_flier_when_tymna_is_not_a_target() {
    // Tymna gone
    let mut g = tbl();
    let flier = perm(&mut g, P0, "Serra Angel");
    let boots = perm(&mut g, P0, BOOTS);
    perm(&mut g, P0, "Ardenn, Intrepid Archaeologist");
    combat_start(&mut g);
    assert_eq!(g.perm(boots).attached, Some(flier));
    // Tymna wearing Sword of Hearth and Home: protection from white, so Ardenn (white) can't target her
    let mut g = tbl();
    let t = perm(&mut g, P0, TYMNA);
    let sohh = perm(&mut g, P0, SOHH);
    g.perm_mut(sohh).attached = Some(t);
    let flier = perm(&mut g, P0, "Serra Angel");
    let boots = perm(&mut g, P0, BOOTS);
    perm(&mut g, P0, "Ardenn, Intrepid Archaeologist");
    combat_start(&mut g);
    assert_eq!(g.perm(boots).attached, Some(flier));
}

#[test]
fn boots_worn_by_tymna_stay_on_her_when_ardenn_could_move_them_to_a_flier() {
    let mut g = tbl();
    let t = perm(&mut g, P0, TYMNA);
    perm(&mut g, P0, "Serra Angel");
    let boots = perm(&mut g, P0, BOOTS);
    g.perm_mut(boots).attached = Some(t);
    perm(&mut g, P0, "Ardenn, Intrepid Archaeologist");
    combat_start(&mut g);
    assert_eq!(g.perm(boots).attached, Some(t));
}

#[test]
fn mithril_coat_is_not_penalised_on_tymna_but_a_sword_is() {
    let mut g = tbl();
    let t = perm(&mut g, P0, TYMNA);
    let (coat, sword, buster) =
        (perm(&mut g, P0, "Mithril Coat"), perm(&mut g, P0, SOFI), perm(&mut g, P0, "Buster Sword"));
    g.perm_mut(coat).attached = None;
    let v = |e| tc::equip_holder_value(&g, P0, e, t);
    assert!((v(coat) - v(sword) - 6.0).abs() < 1e-9, "Coat {} vs Sword {}", v(coat), v(sword));
    assert!((v(coat) - v(buster) - 6.0).abs() < 1e-9);
}

#[test]
fn another_seat_keeps_the_old_free_attach_choice() {
    // the cloud seat: Kor Outfitter puts Hearth and Home on the best `holder_score` creature, Ballista included
    let mut g = table(&["cloud", "seph", "sauron"]);
    g.active = Some(P0);
    let b = ballista_counters(&mut g, P0, 4);
    token(&mut g, P0, 1);
    let e = perm(&mut g, P0, SOHH);
    perm(&mut g, P0, "Kor Outfitter");
    assert_eq!(g.perm(e).attached, Some(b));
}
