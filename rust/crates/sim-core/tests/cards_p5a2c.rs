//! P5a-2b C4/C5: source last-known copies and the shared Prodigy path.
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

// 32: the source is already off the battlefield when its Leaves hook is queued.
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

// 33/34: compiled self-dies, alone and in either wipe order.
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

// 35: a surviving listener uses the pre-wipe count even after Delney has left.
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

// 36: Vondam has already left when left_with queues his trigger; Teysa adds to Delney.
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

// 37: these controls compile and pass before the new DataKey exists.
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
