//! Practice mode, Sephiroth's graveyard cards (and Deep Analysis): reanimation targets are chosen as the spell is cast,
//! from any graveyard where the card allows; Entomb, Buried Alive and Grisly Salvage let you pick; Deadly Dispute asks
//! what to sacrifice (Python's tests/test_play_seph.py).

mod play_common;
use play_common::*;
use sim_core::engine::{tutors, zones};
use sim_core::human::human::human_main;
use sim_core::human::{choices, legal};

fn float_mana(g: &mut Game, p: PlayerId, m: &[(char, u32)]) {
    pool(g, p, m);
}

fn gy_push(g: &mut Game, p: PlayerId, name: &str) -> sim_core::ids::CardId {
    let c = card(g, name);
    g.player_mut(p).gy.push(c);
    c
}

// ------------------------------------------------------------------ Reanimation
#[test]
fn animate_dead_from_an_opponents_graveyard() {
    let mut g = table(&["seph", "veyran"]);
    hand(&mut g, P0, &["Animate Dead"]);
    gy_push(&mut g, P1, "Guttersnipe");
    gy_push(&mut g, P0, "Grave Titan");
    float_mana(&mut g, P0, &[('C', 1), ('B', 1)]);
    let h = seat(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), by_text("Guttersnipe"), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(has_perm(&g, P0, "Guttersnipe"));
    assert!(h.asked()[1].choices[0].contains("graveyard"));
}

#[test]
fn unburial_rites_only_from_your_graveyard() {
    let mut g = table(&["seph", "veyran"]);
    gy_push(&mut g, P1, "Guttersnipe");
    assert!(choices::rean_candidates(&g, P0, "rites").is_empty());
    let c = hand(&mut g, P0, &["Unburial Rites"])[0];
    assert!(legal::check_cast(&mut g, P0, c, "hand").unwrap().contains("no creature card"));
}

#[test]
fn entomb() {
    let mut g = table(&["seph", "veyran"]);
    seat(&mut g, P0, vec![by_text("Grave Titan")]);
    choices::fill(&mut g, P0, "entomb").unwrap();
    let names: Vec<String> = g.player(P0).gy.iter().map(|&c| g.db.get(c).name.to_string()).collect();
    assert_eq!(names, ["Grave Titan"]);
}

#[test]
fn grisly_salvage() {
    let mut g = table(&["seph", "veyran"]);
    let n = g.player(P0).hand.len();
    seat(&mut g, P0, vec![cancel()]);
    choices::fill(&mut g, P0, "grisly").unwrap();
    assert_eq!((g.player(P0).hand.len(), g.player(P0).gy.len()), (n, 5));
}

// ------------------------------------------------------------------ Dispute
#[test]
fn deadly_dispute_sacrifices_your_pick() {
    let mut g = table(&["seph", "veyran"]);
    hand(&mut g, P0, &["Deadly Dispute"]);
    let a = perm(&mut g, P0, "Sol Ring");
    float_mana(&mut g, P0, &[('C', 1), ('B', 1)]);
    seat(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), pick(0), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(!g.player(P0).perms.contains(&a));
    assert_eq!(g.player(P0).treasures, 1);
}

// ------------------------------------------------------------------ FromTheGraveyard
#[test]
fn flashback_deep_analysis() {
    let mut g = table(&["yshtola", "veyran"]);
    let da = gy_push(&mut g, P0, "Deep Analysis");
    float_mana(&mut g, P0, &[('C', 1), ('U', 1)]);
    let n = g.player(P0).hand.len();
    seat(&mut g, P0, vec![act(json!({"do": "cast", "zone": "gy", "card": 0})), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert_eq!((g.player(P0).hand.len(), g.player(P0).life), (n + 2, 37));
    assert!(g.player(P0).exile.contains(&da));
}

#[test]
fn dread_return_flashback_sacrifices_three() {
    let mut g = table(&["seph", "veyran"]);
    for n in ["Llanowar Elves", "Birds of Paradise", "Viscera Seer"] {
        perm(&mut g, P0, n);
    }
    gy_push(&mut g, P0, "Dread Return");
    gy_push(&mut g, P0, "Grave Titan");
    seat(
        &mut g,
        P0,
        vec![
            act(json!({"do": "cast", "zone": "gy", "card": 0})),
            by_text("Grave Titan"),
            pick(0),
            pick(0),
            pick(0),
            pass(),
        ],
    );
    human_main(&mut g, P0, false).unwrap();
    assert!(has_perm(&g, P0, "Grave Titan"));
    assert!(!["Llanowar Elves", "Birds of Paradise", "Viscera Seer"].iter().any(|n| has_perm(&g, P0, n)));
}

#[test]
fn yawgmoths_will() {
    let mut g = table(&["seph", "veyran"]);
    gy_push(&mut g, P0, "Swamp");
    let entomb = gy_push(&mut g, P0, "Entomb");
    g.player_mut(P0).yawg = true;
    g.player_mut(P0).yawg_gy = g.player(P0).gy.clone();
    seat(&mut g, P0, vec![act(json!({"do": "land", "zone": "gy", "card": 0})), pass()]);
    human_main(&mut g, P0, false).unwrap();
    let l = *g.player(P0).lands.last().unwrap();
    assert_eq!(&*g.db.get(g.perm(l).card()).name, "Swamp");
    float_mana(&mut g, P0, &[('B', 1)]);
    assert_eq!(legal::check_cast_gy(&g, P0, entomb), None); // its normal cost, from the graveyard
}

#[test]
fn not_from_the_graveyard_otherwise() {
    let mut g = table(&["seph", "veyran"]);
    let c = gy_push(&mut g, P0, "Entomb");
    assert!(legal::check_cast_gy(&g, P0, c).unwrap().contains("can't cast Entomb from your graveyard"));
}

// ------------------------------------------------------------------ Loops
#[test]
fn start_the_loop() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Mikaeus, the Unhallowed");
    let trisk = perm(&mut g, P0, "Triskelion");
    let labels: Vec<String> =
        sim_core::human::human::abilities_of(&mut g, P0, trisk).unwrap().into_iter().map(|a| a.0).collect();
    assert!(labels.iter().any(|l| l.starts_with("start the loop: Mikaeus + Triskelion")), "{labels:?}");
    let k = labels.iter().position(|l| l.starts_with("start the loop")).unwrap();
    let i = g.player(P0).perms.iter().position(|&x| x == trisk).unwrap();
    seat(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pick(k)]);
    human_main(&mut g, P0, false).unwrap();
    assert!(g.over);
    assert_eq!(g.winner, Some(P0));
}

// ------------------------------------------------------------------ Triggers
#[test]
fn aura_shards_target_or_none() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Aura Shards");
    let ring = perm(&mut g, P1, "Sol Ring");
    seat(&mut g, P0, vec![cancel()]);
    perm(&mut g, P0, "Llanowar Elves");
    assert!(g.player(P1).perms.contains(&ring));
    seat(&mut g, P0, vec![by_text("Sol Ring")]);
    perm(&mut g, P0, "Birds of Paradise");
    assert!(!g.player(P1).perms.contains(&ring));
}

#[test]
fn archon_targets_your_pick() {
    let mut g = table(&["seph", "veyran", "sauron"]);
    seat(&mut g, P0, vec![by_text("Sauron")]);
    perm(&mut g, P0, "Archon of Cruelty");
    assert_eq!((g.player(P1).life, g.player(P2).life), (40, 37));
}

#[test]
fn consecrated_sphinx_may() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Consecrated Sphinx");
    let n = g.player(P0).hand.len();
    seat(&mut g, P0, vec![pick(1)]);
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!(g.player(P0).hand.len(), n);
    seat(&mut g, P0, vec![pick(0)]);
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 2);
}

#[test]
fn nim_deathmantle_pay_or_not() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Nim Deathmantle");
    lands(&mut g, P0, "Swamp", 4, false);
    let t = perm(&mut g, P0, "Grave Titan");
    seat(&mut g, P0, vec![pick(0)]);
    zones::die(&mut g, t, "destroy").unwrap();
    assert!(has_perm(&g, P0, "Grave Titan"));
}

#[test]
fn tortured_existence() {
    use sim_core::human::abilities::tortured;
    let mut g = table(&["seph", "veyran"]);
    let te = perm(&mut g, P0, "Tortured Existence");
    let elves = hand(&mut g, P0, &["Llanowar Elves"])[0];
    let titan = gy_push(&mut g, P0, "Grave Titan");
    float_mana(&mut g, P0, &[('B', 1)]);
    seat(&mut g, P0, vec![pick(0), pick(0)]);
    assert_eq!(tortured(&mut g, P0, te).unwrap(), None);
    assert!(g.player(P0).hand.contains(&titan));
    assert!(g.player(P0).gy.contains(&elves));
}

#[test]
fn gifts_ungiven_you_pick_they_split() {
    let mut g = table(&["seph", "veyran"]);
    seat(&mut g, P0, vec![pick(0), pick(0), pick(0), pick(0)]);
    let n = g.player(P0).hand.len();
    tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
    assert_eq!((g.player(P0).hand.len(), g.player(P0).gy.len()), (n + 2, 2));
}

// ------------------------------------------------------------------ Atraxa
#[test]
fn you_pick_a_card_of_each_type() {
    let mut g = table(&["seph", "veyran"]);
    let top = [
        "Swamp",
        "Sol Ring",
        "Grave Titan",
        "Animate Dead",
        "Entomb",
        "Reanimate",
        "Forest",
        "Mind Stone",
        "Kitchen Finks",
        "Toxic Deluge",
    ];
    let ids: Vec<_> = top.iter().map(|n| card(&g, n)).collect();
    g.player_mut(P0).library.extend(ids.iter().rev().copied()); // the first of these on top
    let n = g.player(P0).library.len();
    // artifact, creature, enchantment, instant, no land, sorcery
    let h = seat(
        &mut g,
        P0,
        vec![
            by_text("Mind Stone"),
            by_text("Kitchen Finks"),
            by_text("Animate Dead"),
            by_text("Entomb"),
            cancel(),
            by_text("Toxic Deluge"),
        ],
    );
    tutors::atraxa_reveal(&mut g, P0).unwrap();
    let hand_ = &g.player(P0).hand;
    let mut names: Vec<String> = hand_[hand_.len() - 5..].iter().map(|&c| g.db.get(c).name.to_string()).collect();
    names.sort();
    assert_eq!(names, ["Animate Dead", "Entomb", "Kitchen Finks", "Mind Stone", "Toxic Deluge"]);
    assert_eq!(g.player(P0).library.len(), n - 5); // the other five on the bottom
    let asked = h.asked();
    assert!(asked[0].prompt.contains("artifact"));
    assert!(asked[4].prompt.contains("land"));
}
