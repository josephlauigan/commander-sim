//! Source last-known copies and the shared Prodigy path.
use sim_core::engine::{removal, stack, zones};
use sim_core::ids::PlayerId;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const A: PlayerId = PlayerId(0);
const B: PlayerId = PlayerId(1);
const DELNEY: &str = "Delney, Streetwise Lookout";
fn tbl() -> Game {
    table(&["heliod-mono-white-stax", "seph", "cloud"])
}
fn tokens(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).token).count()
}
fn wipe(g: &mut Game) {
    removal::apply_wipe(g, A, "destroy", None, &Ctx::default()).unwrap();
}

// The source is already off the battlefield when its Leaves hook is queued.
#[test]
fn departed_skyclave_delney_makes_two_three_three_illusions() {
    let mut g = tbl();
    perm(&mut g, A, DELNEY);
    let target = perm(&mut g, B, "Phyrexian Arena");
    let s = perm(&mut g, A, "Skyclave Apparition");
    assert!(!g.perm(target).on_bf);
    let before = g.player(B).perms.len();
    zones::die(&mut g, s, "destroy").unwrap();
    let made = &g.player(B).perms[before..];
    assert_eq!(made.len(), 2);
    assert!(made.iter().all(|&m| g.perm(m).token && g.perm(m).pow == 3 && g.perm(m).tgh == 3));
}

// Compiled self-dies, alone and in either wipe order.
#[test]
fn solemn_delney_dying_alone_draws_two() {
    let mut g = tbl();
    perm(&mut g, A, DELNEY);
    let s = perm(&mut g, A, "Solemn Simulacrum");
    let before = g.player(A).hand.len();
    zones::die(&mut g, s, "destroy").unwrap();
    assert_eq!(g.player(A).hand.len() - before, 2);
}
#[test]
fn solemn_delney_wipe_draws_two_in_either_order() {
    for delney_first in [true, false] {
        let mut g = tbl();
        for n in if delney_first { [DELNEY, "Solemn Simulacrum"] } else { ["Solemn Simulacrum", DELNEY] } {
            perm(&mut g, A, n);
        }
        let before = g.player(A).hand.len();
        wipe(&mut g);
        assert_eq!(g.player(A).hand.len() - before, 2, "Delney first: {delney_first}");
    }
}

fn solemn_surviving_delney_wipe() -> (Game, sim_core::ids::PermId) {
    let mut g = tbl();
    let d = perm(&mut g, A, DELNEY);
    let s = perm(&mut g, A, "Solemn Simulacrum");
    g.perm_mut(s).eot_kw.push("indestructible");
    wipe(&mut g);
    assert!(!g.perm(d).on_bf);
    assert!(g.perm(s).on_bf);
    g.perm_mut(s).eot_kw.clear();
    (g, s)
}

#[test]
fn solemn_surviving_delney_wipe_draws_once_when_destroyed_alone() {
    let (mut g, s) = solemn_surviving_delney_wipe();
    let before = g.player(A).hand.len();
    zones::die(&mut g, s, "destroy").unwrap();
    assert!(!g.perm(s).on_bf);
    assert_eq!(g.player(A).hand.len() - before, 1);
}

#[test]
fn solemn_surviving_delney_wipe_draws_once_in_a_later_wipe() {
    let (mut g, s) = solemn_surviving_delney_wipe();
    let before = g.player(A).hand.len();
    wipe(&mut g);
    assert!(!g.perm(s).on_bf);
    assert_eq!(g.player(A).hand.len() - before, 1);
}

// A surviving listener uses the pre-wipe count even after Delney has left.
#[test]
fn surviving_elas_looks_back_for_delney_first_or_last() {
    for delney_first in [true, false] {
        let mut g = tbl();
        let e = perm(&mut g, A, "Elas il-Kor, Sadistic Pilgrim");
        g.perm_mut(e).eot_kw.push("indestructible");
        if delney_first {
            perm(&mut g, A, DELNEY);
        }
        token(&mut g, A, 1);
        if !delney_first {
            perm(&mut g, A, DELNEY);
        }
        let before = g.player(B).life;
        wipe(&mut g);
        assert!(g.perm(e).on_bf);
        assert_eq!(before - g.player(B).life, 4, "Delney first: {delney_first}");
        // A stale batch record must not double a later, unrelated death.
        let t = token(&mut g, A, 1);
        let before = g.player(B).life;
        zones::die(&mut g, t, "destroy").unwrap();
        assert_eq!(before - g.player(B).life, 1);
    }
}

// A departed Vondam retains Delney's extra; Teysa's copies add to Delney's.
#[test]
fn departed_vondam_left_with_keeps_delney() {
    let mut g = tbl();
    let v = perm(&mut g, A, "Syr Vondam, Sunstar Exemplar");
    perm(&mut g, A, DELNEY);
    let before = g.player(A).life;
    wipe(&mut g);
    assert!(!g.perm(v).on_bf);
    assert_eq!(g.player(A).life - before, 2);
}
#[test]
fn departed_carrier_thrall_teysa_and_delney_make_three_scions() {
    let mut g = tbl();
    perm(&mut g, A, "Teysa Karlov");
    perm(&mut g, A, DELNEY);
    let c = perm(&mut g, A, "Carrier Thrall");
    let before = tokens(&g, A);
    zones::die(&mut g, c, "destroy").unwrap();
    assert_eq!(tokens(&g, A) - before, 3);
}

// Controls: no source doubler writes no record, and a blinked permanent carries no previous record.
#[test]
fn wipe_without_source_doublers_writes_no_lki_extra() {
    let mut g = tbl();
    perm(&mut g, A, "Solemn Simulacrum");
    wipe(&mut g);
    assert!(g.perms.iter().all(|m| !format!("{:?}", m.data).contains("LkiExtra")));
}
#[test]
fn blinked_solemn_after_delney_leaves_draws_once() {
    let mut g = tbl();
    let d = perm(&mut g, A, DELNEY);
    let s = perm(&mut g, A, "Solemn Simulacrum");
    zones::die(&mut g, d, "destroy").unwrap();
    sim_core::impls::t2::blink(&mut g, A, s).unwrap();
    let back = *g.player(A).perms.iter().find(|&&m| g.perm(m).name == "Solemn Simulacrum").unwrap();
    assert_ne!(back, s);
    let before = g.player(A).hand.len();
    zones::die(&mut g, back, "destroy").unwrap();
    assert_eq!(g.player(A).hand.len() - before, 1);
    assert_eq!(stack::extra_for(&g, back), 0);
}

// A real queued Wizard hook gains the same extras as compiled triggers.
#[test]
fn prodigy_spellseeker_queues_two_searches() {
    let mut g = tbl();
    perm(&mut g, A, "Harmonic Prodigy");
    g.player_mut(A).library = ["Counterspell", "Swords to Plowshares", "Brainstorm"].map(|n| card(&g, n)).to_vec();
    let before = g.player(A).hand.len();
    perm(&mut g, A, "Spellseeker");
    assert_eq!(g.player(A).hand.len() - before, 2);
}

// Census payload checks: repeated instances must preserve the actual Oracle effects.
#[test]
fn doubled_thundermaw_damage_accumulates_on_surviving_fliers() {
    for toughness in [2, 4] {
        let mut g = tbl();
        perm(&mut g, A, "Panharmonicon");
        let t = token(&mut g, B, toughness);
        g.perm_mut(t).fly = true;
        perm(&mut g, A, "Thundermaw Hellkite");
        assert_eq!(g.perm(t).on_bf, toughness > 2);
        if toughness > 2 {
            assert!(g.perm(t).tapped);
            assert_eq!(sim_core::human::combos::marked(&g, t), 2);
        }
    }
}
#[test]
fn doubled_massacre_wurm_shrink_accumulates_and_ignores_indestructible() {
    let mut g = tbl();
    perm(&mut g, A, "Panharmonicon");
    let t = token(&mut g, B, 4);
    g.perm_mut(t).eot_kw.push("indestructible");
    perm(&mut g, A, "Massacre Wurm");
    assert!(!g.perm(t).on_bf);
}
#[test]
fn doubled_bonny_creates_a_second_beau_before_the_legend_rule() {
    let mut g = tbl();
    lands(&mut g, A, "Forest", 5, false);
    perm(&mut g, A, "Panharmonicon");
    perm(&mut g, A, "Impact Tremors");
    let before = g.player(B).life;
    perm(&mut g, A, "Bonny Pall, Clearcutter");
    assert_eq!(before - g.player(B).life, 6, "Bonny and both Beaus enter, each with two Tremors instances");
    assert_eq!(g.player(A).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).name == "Beau").count(), 1);
}
#[test]
fn doubled_aura_exile_links_return_both_cards() {
    use sim_core::engine::cast;
    for aura in ["Chains of Custody", "Sheltered by Ghosts"] {
        let mut g = tbl();
        perm(&mut g, A, "Starfield Vocalist");
        let host = perm(&mut g, A, "Serra Angel");
        let a = perm(&mut g, B, "Myr Battlesphere");
        let b = perm(&mut g, B, "Steel Hellkite");
        let c = hand(&mut g, A, &[aura])[0];
        lands(&mut g, A, "Plains", 5, false);
        cast::cast_card(&mut g, A, c, "hand", Ctx { target: Some(host), ..Default::default() }).unwrap();
        assert!(!g.perm(a).on_bf && !g.perm(b).on_bf, "{aura}");
        let m = *g.player(A).perms.iter().find(|&&m| g.perm(m).cd == Some(c)).unwrap();
        zones::die(&mut g, m, "destroy").unwrap();
        for n in ["Myr Battlesphere", "Steel Hellkite"] {
            assert!(g.player(B).perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).name == n), "{aura}: {n}");
        }
    }
}

#[test]
fn welcoming_and_tocasia_hear_creature_tokens_once_per_turn() {
    for listener in ["Welcoming Vampire", "Tocasia's Welcome"] {
        for copier in [false, true] {
            let mut g = tbl();
            if copier {
                perm(&mut g, A, "Panharmonicon");
            }
            let s = perm(&mut g, A, listener);
            let before = g.player(A).hand.len();
            g.resolving += 1;
            token(&mut g, A, 1);
            assert_eq!(g.trig_queue.iter().filter(|t| t.src == Some(s)).count(), if copier { 2 } else { 1 });
            g.resolving -= 1;
            stack::flush_triggers(&mut g).unwrap();
            assert_eq!(g.player(A).hand.len() - before, 1, "{listener}");
            token(&mut g, A, 1);
            assert_eq!(g.player(A).hand.len() - before, 1, "{listener}: a later batch in the same turn");
        }
    }
}
