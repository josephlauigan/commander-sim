//! Practice mode, Sauron's cards: the choices the AI used to make for them are yours. Damage triggers (Orcish
//! Bowmasters, Kaervek, Sword of Fire and Ice) ask for a target; the Ring asks for its Ring-bearer, Call of the Ring
//! and Sauron ask whether to use their "may" abilities (Python's tests/test_play_sauron.py).

mod play_common;
use play_common::*;
use sim_core::cardcode as rules;
use sim_core::engine::{combat, zones};
use sim_core::human::combat::attack_candidates;
use sim_core::human::human::{human_main, use_land};
use sim_core::human::{Target, legal};
use sim_core::impls::mine;

/// an answer function that picks the choice describing x, whatever position it is listed in
fn label_of(g: &Game, p: PlayerId, x: Target) -> String {
    legal::describe_target(g, p, x)
}

fn pick_label(label: String) -> F {
    Box::new(move |r| {
        Answer::Pick(
            r.choices.iter().position(|c| *c == label).unwrap_or_else(|| panic!("{label:?} not in {:?}", r.choices)),
        )
    })
}

/// a seat that gives the same kind of answer to every request
fn seat_fn(
    g: &mut Game,
    p: PlayerId,
    f: impl FnMut(&Request) -> Answer + Send + 'static,
) -> std::sync::Arc<ScriptHuman> {
    let h = std::sync::Arc::new(ScriptHuman::from_fn(f));
    g.humans.add(p, h.clone());
    h
}

fn float_mana(g: &mut Game, p: PlayerId, m: &[(char, u32)]) {
    pool(g, p, m);
}

// ------------------------------------------------------------------ Damage
#[test]
fn bowmasters_enters() {
    let mut g = table(&["sauron", "veyran"]);
    let l = label_of(&g, P0, Target::Player(P1));
    let h = seat_fn(&mut g, P0, {
        let mut f = pick_label(l);
        move |r| f(r)
    });
    perm(&mut g, P0, "Orcish Bowmasters");
    assert_eq!(g.player(P1).life, 39);
    assert!(sim_core::engine::values::army_of(&g, P0).is_some());
    assert_eq!(&*h.asked()[0].kind, "target");
}

#[test]
fn bowmasters_on_extra_draws() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Orcish Bowmasters");
    let h = seat_fn(&mut g, P0, |r| {
        // the label carries the life total, which changes with each trigger
        Answer::Pick(r.choices.iter().position(|c| c.starts_with("Veyran (player")).unwrap())
    });
    zones::draw(&mut g, P1, 2, false).unwrap(); // two cards, neither is the draw step's first
    assert_eq!(h.asked().len(), 2);
}

#[test]
fn a_creature_survives_damage_below_its_toughness() {
    let mut g = table(&["sauron", "veyran"]);
    let big = perm(&mut g, P1, "Grave Titan");
    let l = label_of(&g, P0, Target::Perm(big));
    seat(&mut g, P0, vec![pick_label(l)]);
    let c = card(&g, "Counterspell");
    mine::kaervek(&mut g, P0, P1, c).unwrap(); // mana value 2 against a 6/6
    assert!(g.player(P1).perms.contains(&big));
}

#[test]
fn kaervek_kills_what_it_reaches() {
    let mut g = table(&["sauron", "veyran"]);
    let gs = perm(&mut g, P1, "Guttersnipe");
    let l = label_of(&g, P0, Target::Perm(gs));
    seat(&mut g, P0, vec![pick_label(l)]);
    let c = card(&g, "Counterspell");
    mine::kaervek(&mut g, P0, P1, c).unwrap();
    assert!(!g.player(P1).perms.contains(&gs));
}

// ------------------------------------------------------------------ Ring
#[test]
fn tempt() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Call of the Ring");
    perm(&mut g, P0, "Sauron, the Dark Lord");
    let bow = perm(&mut g, P0, "Grave Titan");
    hand(&mut g, P0, &["Island"]);
    let l = label_of(&g, P0, Target::Perm(bow));
    seat_fn(&mut g, P0, move |r| {
        if r.prompt.starts_with("The Ring") {
            Answer::Pick(r.choices.iter().position(|c| *c == l).unwrap())
        } else if r.prompt.starts_with("Call") {
            Answer::Pick(0)
        } else {
            Answer::Pick(1) // Sauron's discard: no
        }
    });
    mine::ring_tempt(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).ring_bearer, Some(bow));
    assert_eq!(g.player(P0).life, 38);
    assert_eq!(g.player(P0).hand.len(), 2); // paid 2, drew 1, kept the hand
}

// ------------------------------------------------------------------ Activations
#[test]
fn jaces_archivist() {
    use sim_core::human::abilities::archivist;
    let mut g = table(&["sauron", "veyran"]);
    let arch = perm(&mut g, P0, "Jace's Archivist");
    g.perm_mut(arch).sick = false;
    hand(&mut g, P0, &["Island", "Swamp"]);
    hand(&mut g, P1, &["Island", "Mountain", "Island"]);
    seat(&mut g, P0, vec![]);
    float_mana(&mut g, P0, &[('U', 1)]);
    assert_eq!(archivist(&mut g, P0, arch).unwrap(), None);
    assert_eq!((g.player(P0).hand.len(), g.player(P1).hand.len()), (3, 3));
    assert!(g.perm(arch).tapped);
}

#[test]
fn archivist_needs_to_untap_first() {
    use sim_core::human::abilities::archivist;
    let mut g = table(&["sauron", "veyran"]);
    let arch = perm(&mut g, P0, "Jace's Archivist");
    g.perm_mut(arch).sick = true;
    seat(&mut g, P0, vec![]);
    float_mana(&mut g, P0, &[('U', 1)]);
    assert!(archivist(&mut g, P0, arch).unwrap().unwrap().contains("summoning sickness"));
}

#[test]
fn aggravated_assault() {
    use sim_core::human::abilities::assault;
    let mut g = table(&["sauron", "veyran"]);
    let aa = perm(&mut g, P0, "Aggravated Assault");
    let t = perm(&mut g, P0, "Grave Titan");
    g.perm_mut(t).tapped = true;
    seat(&mut g, P0, vec![]);
    float_mana(&mut g, P0, &[('C', 3), ('R', 2)]);
    g.step = sim_core::state::Step::Main1;
    assert_eq!(assault(&mut g, P0, aa).unwrap(), None);
    assert!(!g.perm(t).tapped);
    assert_eq!(g.player(P0).extra_combats, 1);
}

#[test]
fn rogues_passage() {
    let mut g = table(&["sauron", "veyran"]);
    lands(&mut g, P0, "Rogue's Passage", 1, false);
    let t = perm(&mut g, P0, "Grave Titan");
    g.perm_mut(t).sick = false;
    let gs = perm(&mut g, P1, "Guttersnipe");
    g.perm_mut(gs).sick = false;
    float_mana(&mut g, P0, &[('C', 4)]);
    let l = label_of(&g, P0, Target::Perm(t));
    seat(&mut g, P0, vec![pick(0), pick_label(l)]); // the ability, then the creature
    let land = g.player(P0).lands[0];
    assert_eq!(use_land(&mut g, P0, land).unwrap(), None);
    assert!(g.land(land).tapped);
    let i = attack_candidates(&g, P0).iter().position(|&x| x == t).unwrap();
    seat(&mut g, P0, vec![picks(&[i])]);
    combat::combat(&mut g, P0).unwrap(); // Guttersnipe can't block it
    assert_eq!(g.player(P1).life, 34);
}

#[test]
fn scavenger_grounds() {
    let mut g = table(&["sauron", "veyran"]);
    lands(&mut g, P0, "Scavenger Grounds", 1, false);
    let titan = card(&g, "Grave Titan");
    g.player_mut(P1).gy.push(titan);
    float_mana(&mut g, P0, &[('C', 2)]);
    seat(&mut g, P0, vec![pick(0)]);
    let land = g.player(P0).lands[0];
    assert_eq!(use_land(&mut g, P0, land).unwrap(), None);
    assert!(g.player(P1).gy.is_empty());
    assert!(g.player(P1).exile.contains(&titan));
}

#[test]
fn cyclonic_rift_overloaded() {
    let mut g = table(&["sauron", "veyran"]);
    hand(&mut g, P0, &["Cyclonic Rift"]);
    perm(&mut g, P1, "Guttersnipe");
    perm(&mut g, P1, "Sol Ring");
    float_mana(&mut g, P0, &[('C', 6), ('U', 1)]);
    seat(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), pick(1), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(g.player(P1).perms.is_empty());
}

// ------------------------------------------------------------------ SpellChoices
#[test]
fn toxic_deluge_x() {
    let mut g = table(&["sauron", "veyran"]);
    hand(&mut g, P0, &["Toxic Deluge"]);
    let small = perm(&mut g, P1, "Guttersnipe");
    let big = perm(&mut g, P1, "Grave Titan");
    float_mana(&mut g, P0, &[('C', 2), ('B', 1)]);
    seat(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), pick(2), pass()]); // X = 2
    human_main(&mut g, P0, false).unwrap();
    assert!(!g.player(P1).perms.contains(&small));
    assert!(g.player(P1).perms.contains(&big));
    assert_eq!(g.player(P0).life, 38);
}

#[test]
fn bitter_triumph_discard_instead_of_life() {
    let mut g = table(&["sauron", "veyran"]);
    hand(&mut g, P0, &["Bitter Triumph", "Island"]);
    let gs = perm(&mut g, P1, "Guttersnipe");
    float_mana(&mut g, P0, &[('C', 1), ('B', 1)]);
    let l = label_of(&g, P0, Target::Perm(gs));
    seat(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), pick_label(l), pick(1), pick(0), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(!g.player(P1).perms.contains(&gs));
    assert_eq!(g.player(P0).life, 40);
    let isl = card(&g, "Island");
    assert!(g.player(P0).gy.contains(&isl));
}

#[test]
fn gearhulk_target_or_none() {
    let mut g = table(&["sauron", "veyran"]);
    let gs = perm(&mut g, P1, "Guttersnipe");
    seat(&mut g, P0, vec![cancel()]);
    perm(&mut g, P0, "Noxious Gearhulk");
    assert!(g.player(P1).perms.contains(&gs));
    let l = label_of(&g, P0, Target::Perm(gs));
    seat(&mut g, P0, vec![pick_label(l)]);
    perm(&mut g, P0, "Noxious Gearhulk");
    assert!(!g.player(P1).perms.contains(&gs));
}

#[test]
fn kefka_discard_is_yours() {
    let mut g = table(&["sauron", "veyran"]);
    let hs = hand(&mut g, P0, &["Island", "Counterspell"]);
    hand(&mut g, P1, &["Mountain"]);
    seat(&mut g, P0, vec![pick(1)]); // discard Counterspell
    mine::kefka_wheel(&mut g, P0).unwrap();
    assert!(g.player(P0).gy.contains(&hs[1]));
    assert!(g.player(P0).hand.contains(&hs[0]));
}

// ------------------------------------------------------------------ Taxes
#[test]
fn rhystic_study_asks() {
    let mut g = table(&["sauron", "veyran"]);
    let l = lands(&mut g, P0, "Island", 1, false)[0];
    seat(&mut g, P0, vec![pick(0)]); // yes: pay {1} (taps the Island)
    assert!(!rules::rhystic_unpaid(&mut g, P0).unwrap());
    assert!(g.land(l).tapped);
    seat(&mut g, P0, vec![pick(1)]); // no
    assert!(rules::rhystic_unpaid(&mut g, P0).unwrap());
}

#[test]
fn cant_pay_isnt_asked() {
    let mut g = table(&["sauron", "veyran"]);
    let h = seat(&mut g, P0, vec![]);
    assert!(rules::tithe_unpaid(&mut g, P0).unwrap());
    assert!(h.asked().is_empty());
}
