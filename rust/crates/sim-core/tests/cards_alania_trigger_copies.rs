use sim_core::engine::cast;
use sim_core::ids::PlayerId;
use sim_core::state::Ctx;
use sim_core::testkit::*;

fn check_copies(copier: Option<&str>, expected: usize) {
    let (p, q, r) = (PlayerId(0), PlayerId(1), PlayerId(2));
    let mut g = table(&["kefka", "seph", "sauron"]);
    perm(&mut g, p, "Alania, Divergent Storm");
    if let Some(name) = copier {
        perm(&mut g, p, name);
    }
    let before = g.player(q).hand.len() + g.player(r).hand.len();
    let c = hand(&mut g, p, &["Lightning Bolt"])[0];
    cast::cast_card(&mut g, p, c, "hand", Ctx { face: Some(q), ..Ctx::default() }).unwrap();
    assert_eq!(g.player(q).hand.len() + g.player(r).hand.len(), before + expected);
    assert_eq!(g.player(q).life + g.player(r).life, 80 - 3 * (1 + expected as i32));
}

#[test]
fn alania_alone_copies_once() {
    check_copies(None, 1);
}

#[test]
fn alania_with_veyran_copies_twice() {
    check_copies(Some("Veyran, Voice of Duality"), 2);
}

#[test]
fn alania_with_prodigy_copies_twice() {
    check_copies(Some("Harmonic Prodigy"), 2);
}
