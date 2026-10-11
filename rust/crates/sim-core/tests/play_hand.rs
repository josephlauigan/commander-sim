//! Practice mode: casts from hand that the card code keeps for the AI (play/cards.py HAND), copying spells with
//! Return the Favor, and new targets for copies (Python's tests/test_play_hand.py). The AI never casts these for the
//! human seat.
//!
//! (They need the real `human::cards`.)

mod play_common;
use play_common::*;
use sim_core::engine::zones::{Tokens, make_tokens};
use sim_core::human::human::respond;

fn count_tokens(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).token).count()
}

#[test]
fn twinflame_two_targets() {
    let mut g = table(&["veyran", "seph"]);
    hand(&mut g, P0, &["Twinflame"]);
    perm(&mut g, P0, "Guttersnipe");
    perm(&mut g, P0, "Kessig Flamebreather");
    pool(&mut g, P0, &[('R', 2), ('C', 3)]);
    main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), by_text("Guttersnipe"), by_text("Kessig")]);
    let toks: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token).collect();
    let mut names: Vec<String> = toks.iter().map(|&m| g.perm(m).name.to_string()).collect();
    names.sort();
    assert_eq!(names, ["Guttersnipe", "Kessig Flamebreather"]);
    assert!(toks.iter().all(|&m| !g.perm(m).sick));
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn twinflame_extra_target_needs_mana() {
    let mut g = table(&["veyran", "seph"]);
    let c = hand(&mut g, P0, &["Twinflame"])[0];
    perm(&mut g, P0, "Guttersnipe");
    perm(&mut g, P0, "Kessig Flamebreather");
    pool(&mut g, P0, &[('R', 1), ('C', 1)]);
    let h = main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), by_text("Guttersnipe"), by_text("Kessig")]);
    assert!(told_last(&h).contains("Can't cast Twinflame on 2"), "{}", told_last(&h));
    assert!(g.player(P0).hand.contains(&c));
}

#[test]
fn twinflame_copies_a_token() {
    let mut g = table(&["veyran", "seph"]);
    hand(&mut g, P0, &["Twinflame"]);
    let mut spec = Tokens::new(1, 4);
    spec.fly = true;
    spec.sick = false;
    make_tokens(&mut g, P0, spec).unwrap();
    pool(&mut g, P0, &[('R', 1), ('C', 1)]);
    main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), pick(0)]);
    let n = g.player(P0).perms.iter().filter(|&&m| g.perm(m).token && g.perm(m).pow == 4).count();
    assert_eq!(n, 2);
}

#[test]
fn ephemerate_and_rebound() {
    let mut g = table(&["seph", "veyran"]);
    hand(&mut g, P0, &["Ephemerate"]);
    let m = perm(&mut g, P0, "Grave Titan");
    pool(&mut g, P0, &[('W', 1)]);
    main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), by_text("Grave Titan")]);
    assert!(!g.player(P0).perms.contains(&m)); // it left and came back as a new object
    assert_eq!(count_tokens(&g, P0), 4); // Grave Titan's Zombies: two more
    let c = card(&g, "Ephemerate");
    assert!(g.player(P0).exile.contains(&c));
    seat(&mut g, P0, vec![cancel()]);
    let f = g.registry.get(c).and_then(|i| i.rebound).expect("rebound hook");
    f(&mut g, P0, c).unwrap();
    assert!(g.player(P0).exile.contains(&c)); // not cast: stays in exile
}

#[test]
fn cycling_at_instant_speed() {
    let mut g = table(&["seph", "veyran"]);
    g.active = Some(P1);
    hand(&mut g, P0, &["Unearth"]);
    pool(&mut g, P0, &[('C', 2)]);
    let n = g.player(P0).hand.len();
    seat(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), by_text("cycling"), pass()]);
    respond(&mut g, P0, "End of turn", None, None).unwrap();
    assert_eq!(g.player(P0).hand.len(), n);
    let c = card(&g, "Unearth");
    assert!(g.player(P0).gy.contains(&c));
}

#[test]
fn unearth_your_pick() {
    let mut g = table(&["seph", "veyran"]);
    hand(&mut g, P0, &["Unearth"]);
    let (a, b) = (card(&g, "Carrion Feeder"), card(&g, "Burglar Rat"));
    g.player_mut(P0).gy.extend([a, b]);
    pool(&mut g, P0, &[('B', 1)]);
    main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), by_text("cast it"), by_text("Burglar Rat")]);
    assert!(has_perm(&g, P0, "Burglar Rat"));
}

#[test]
fn sokenzan_channel() {
    let mut g = table(&["veyran", "seph"]);
    hand(&mut g, P0, &["Sokenzan, Crucible of Defiance"]);
    pool(&mut g, P0, &[('R', 1), ('C', 3)]);
    main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0}))]);
    let n = g.player(P0).perms.iter().filter(|&&m| g.perm(m).token && !g.perm(m).sick).count();
    assert_eq!(n, 2);
}

#[test]
fn necromancy_at_instant_speed() {
    let mut g = table(&["seph", "veyran"]);
    g.active = Some(P1);
    hand(&mut g, P0, &["Necromancy"]);
    let t = card(&g, "Grave Titan");
    g.player_mut(P0).gy.push(t);
    pool(&mut g, P0, &[('B', 1), ('C', 2)]);
    seat(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), by_text("Grave Titan"), pass()]);
    respond(&mut g, P0, "End of turn", None, None).unwrap();
    assert!(!has_perm(&g, P0, "Grave Titan")); // sacrificed: cast as though it had flash
    assert_eq!(count_tokens(&g, P0), 2); // its enter trigger happened
}

#[test]
fn return_the_favor_needs_a_spell() {
    let mut g = table(&["veyran", "seph"]);
    hand(&mut g, P0, &["Return the Favor"]);
    pool(&mut g, P0, &[('R', 3)]);
    let h = main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0}))]);
    assert!(told_last(&h).contains("in response"), "{}", told_last(&h));
}

#[test]
fn return_the_favor_copies_your_bolt() {
    let mut g = table(&["veyran", "seph"]);
    hand(&mut g, P0, &["Lightning Bolt", "Return the Favor"]);
    pool(&mut g, P0, &[('R', 4)]);
    main(
        &mut g,
        P0,
        vec![
            act(json!({"do": "cast", "card": 0})),
            by_text("Sephiroth"), // Bolt at Sephiroth
            act(json!({"do": "cast", "card": 0})),
            by_text("copy"),
            by_text("keep"), // then, holding priority: copy it
            pass(),
        ],
    );
    assert_eq!(g.player(P1).life, 34);
}

#[test]
fn the_ai_does_not_cast_return_the_favor_for_you() {
    let mut g = table(&["veyran", "seph"]);
    hand(&mut g, P0, &["Lightning Bolt", "Return the Favor"]);
    pool(&mut g, P0, &[('R', 4)]);
    main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), by_text("Sephiroth"), pass()]);
    assert_eq!(g.player(P1).life, 37);
    let c = card(&g, "Return the Favor");
    assert!(g.player(P0).hand.contains(&c));
}
