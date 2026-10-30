//! Practice mode: the smaller choices are the person's. Mulligans, discards (effects and hand size), sacrifices to
//! edicts, tutors (only what the tutor may find), basic-land searches, scry and surveil (Python's
//! tests/test_play_choices.py).

mod play_common;
use play_common::*;
use sim_core::ai::topdeck;
use sim_core::cards::Types;
use sim_core::dsl::{self, DslCtx, model::Ability};
use sim_core::engine::{cast, turn, tutors, zones};
use sim_core::human::choices;
use sim_core::rng::Rng;

#[test]
fn discard_is_yours() {
    let mut g = table(&["sauron", "veyran"]);
    let cs = hand(&mut g, P0, &["Swamp", "Counterspell", "Sol Ring"]);
    seat(&mut g, P0, vec![pick(2)]); // the third card listed: Sol Ring
    cast::discard_worst(&mut g, P0, 1).unwrap();
    assert!(g.player(P0).gy.contains(&cs[2]));
    assert_eq!(g.player(P0).hand, vec![cs[0], cs[1]]);
}

#[test]
fn discard_to_hand_size() {
    let mut g = table(&["sauron", "veyran"]);
    let mut names = vec!["Swamp"; 4];
    names.extend(vec!["Island"; 5]);
    hand(&mut g, P0, &names);
    let h = seat(&mut g, P0, vec![pick(0), pick(0)]);
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), 7);
    assert_eq!(h.asked().len(), 2);
}

#[test]
fn edict() {
    let mut g = table(&["sauron", "veyran"]);
    let a = perm(&mut g, P0, "Orcish Bowmasters");
    let b = perm(&mut g, P0, "Grave Titan");
    let h =
        ScriptHuman::from_fn(|r| Answer::Pick(r.choices.iter().position(|x| x.starts_with("Grave Titan")).unwrap()));
    g.humans.add(P0, std::sync::Arc::new(h));
    zones::edict(&mut g, P0, false).unwrap();
    assert!(g.player(P0).perms.contains(&a));
    assert!(!g.player(P0).perms.contains(&b));
}

#[test]
fn tutor_finds_only_its_kind() {
    let mut g = table(&["sauron", "veyran"]);
    let h = seat(&mut g, P0, vec![pick(0)]);
    tutors::tutor(&mut g, P0, "art").unwrap();
    let asked = h.asked();
    let ch = &asked[0].choices;
    for x in &ch[..ch.len() - 1] {
        let name = x.split(" {").next().unwrap();
        let c = card(&g, name);
        assert!(g.db.get(c).types.has(Types::ARTIFACT), "{name}");
    }
    assert_eq!(g.player(P0).hand.len(), 1);
    let c = g.player(P0).hand[0];
    assert!(g.db.get(c).types.has(Types::ARTIFACT));
}

#[test]
fn finding_nothing() {
    let mut g = table(&["sauron", "veyran"]);
    let n = g.player(P0).library.len();
    seat(&mut g, P0, vec![cancel()]);
    tutors::tutor(&mut g, P0, "any").unwrap();
    assert_eq!((g.player(P0).hand.len(), g.player(P0).library.len()), (0, n));
}

#[test]
fn basic_land_search() {
    let mut g = table(&["sauron", "veyran"]);
    let h = seat(&mut g, P0, vec![pick(0)]);
    zones::land_ramp(&mut g, P0, 1, true).unwrap();
    let asked = h.asked();
    assert_eq!(asked[0].choices[..asked[0].choices.len() - 1], ["Island", "Mountain", "Swamp"]);
    let l = *g.player(P0).lands.last().unwrap();
    assert_eq!(&*g.db.get(g.land(l).card()).name, "Island");
    assert!(g.land(l).tapped);
}

#[test]
fn scry() {
    let mut g = table(&["sauron", "veyran"]);
    let lib = &g.player(P0).library;
    let (top, second) = (lib[lib.len() - 1], lib[lib.len() - 2]);
    seat(&mut g, P0, vec![pick(1), pick(0)]); // first card to the bottom, second stays
    topdeck::scry(&mut g, P0, 2, false).unwrap();
    let lib = &g.player(P0).library;
    assert_eq!(lib[0], top);
    assert_eq!(lib[lib.len() - 1], second);
}

#[test]
fn mulligan() {
    let mut g = table(&["sauron", "veyran"]);
    // table() leaves hands empty: the library holds the whole deck
    let h = seat(&mut g, P0, vec![ans(Answer::Pick(1)), ans(Answer::Pick(1)), ans(Answer::Pick(0)), pick(0)]);
    choices::mulligan(&mut g, P0, &mut Rng::from_u64(1)).unwrap();
    assert_eq!(g.player(P0).hand.len(), 6); // two mulligans, the first free: one card to the bottom
    let kinds: Vec<String> = h.asked().iter().map(|r| r.kind.to_string()).collect();
    assert_eq!(kinds, ["mulligan", "mulligan", "mulligan", "choose"]);
    assert_eq!(g.player(P0).library.len() + g.player(P0).hand.len(), 99);
}

#[test]
fn choose_modes() {
    let mut g = table(&["sauron", "veyran"]);
    hand(&mut g, P1, &["Island", "Mountain"]);
    let kc = card(&g, "Kolaghan's Command");
    let modal =
        g.db.get(kc)
            .abilities
            .iter()
            .find_map(|a| match a {
                Ability::Spell { effects } => effects.iter().find(|e| e.kind == "modal").cloned(),
                _ => None,
            })
            .expect("a modal effect");
    let h = seat(&mut g, P0, vec![pick(1), pick(0)]); // 'a player discards', then (of the rest) the first
    dsl::execute(&mut g, P0, &[modal], None, &mut DslCtx::default(), Some(kc), 0).unwrap();
    let asked = h.asked();
    assert_eq!(asked.len(), 2);
    assert_eq!(asked[1].choices.len(), 3); // a mode can't be chosen twice
    assert_eq!(g.player(P1).hand.len(), 1);
}

#[test]
fn yes_no() {
    let mut g = table(&["sauron", "veyran"]);
    seat(&mut g, P0, vec![pick(1)]);
    assert!(!choices::yes_no(&g, P0, "Draw?").unwrap());
}
