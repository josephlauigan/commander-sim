//! Practice mode, opponent effects (2026-11-04): where an AI opponent's card makes the person sacrifice, discard,
//! pick or pay, the person decides (found by `practice_opponent_probe`). Each test puts the person at P0 with a scripted
//! answer, lets P1's card do its thing and checks both that the person was asked and that their answer is what happened.

mod play_common;
use play_common::*;
use sim_core::state::Ctx;
use std::sync::Arc;

/// a person who answers with `pick` (None: the first choice, or yes); at a priority, pass
fn person(pick: impl Fn(&Request) -> Option<usize> + Send + Sync + 'static) -> Arc<ScriptHuman> {
    Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        match pick(r) {
            Some(k) => Answer::Pick(k),
            None if r.choices.is_empty() => Answer::Yes,
            None => Answer::Pick(0),
        }
    }))
}

/// the index of the first choice containing `word`
fn with(r: &Request, word: &str) -> Option<usize> {
    r.choices.iter().position(|c| c.contains(word))
}

fn asked(h: &ScriptHuman, text: &str) -> bool {
    h.asked().iter().any(|r| r.prompt.contains(text))
}

/// the table: the person (P0) and an AI opponent (P1) with plenty of mana
fn setup(h: &Arc<ScriptHuman>) -> Game {
    let mut g = table(&["kefka", "seph"]);
    g.humans.add(P0, h.clone());
    lands(&mut g, P1, "Swamp", 8, false);
    g
}

/// P1 casts the spell (the AI's seat; an instant or sorcery gets its targets as the engine picks them)
fn cast_p1(g: &mut Game, name: &str) {
    let c = take(g, P1, name);
    g.player_mut(P1).hand.push(c);
    let ctx = if g.db.get(c).instant || g.db.get(c).sorcery {
        sim_core::engine::cast::spell_targets(g, P1, c, None)
    } else {
        Ctx::default()
    };
    sim_core::engine::cast::cast_card(g, P1, c, "hand".into(), ctx).unwrap();
    sim_core::engine::stack::flush_triggers(g).unwrap();
}

fn gy_has(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn hand_has(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

// ------------------------------------------------------------------ DSL "sacrifice"
#[test]
fn dsl_sacrifice_the_person_picks_the_creature() {
    // Liliana's Triumph: "each opponent sacrifices a creature of their choice"
    let h = person(|r| if r.prompt.contains("sacrifice") { with(r, "Birds of Paradise") } else { None });
    let mut g = setup(&h);
    perm(&mut g, P0, "Armored Warhorse");
    perm(&mut g, P0, "Birds of Paradise");
    perm(&mut g, P0, "Llanowar Elves");
    cast_p1(&mut g, "Liliana's Triumph");
    assert!(asked(&h, "sacrifice"), "the person was asked");
    assert!(gy_has(&g, P0, "Birds of Paradise"), "the creature the person picked died");
    assert!(has_perm(&g, P0, "Armored Warhorse") && has_perm(&g, P0, "Llanowar Elves"), "and no other");
}

#[test]
fn dsl_sacrifice_mire_in_misery_offers_enchantments_too() {
    let h = person(|r| if r.prompt.contains("sacrifice") { with(r, "Parallel Lives") } else { None });
    let mut g = setup(&h);
    perm(&mut g, P0, "Armored Warhorse");
    perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Parallel Lives");
    cast_p1(&mut g, "Mire in Misery");
    assert!(asked(&h, "sacrifice"));
    assert!(!has_perm(&g, P0, "Parallel Lives"), "the picked enchantment went");
    assert!(has_perm(&g, P0, "Armored Warhorse") && has_perm(&g, P0, "Llanowar Elves"));
}

#[test]
fn flare_of_malice_offers_only_the_greatest_mana_value() {
    // Roughrider (MV 3) and Elves (MV 1): Oracle says the greatest mana value only; the person picks among ties
    let h = person(|r| if r.prompt.contains("Flare of Malice") { with(r, "Roughrider") } else { None });
    let mut g = setup(&h);
    perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Birds of Paradise");
    perm(&mut g, P0, "Goblin Roughrider");
    perm(&mut g, P0, "Kitchen Finks"); // MV 3, tied with the Roughrider
    cast_p1(&mut g, "Flare of Malice");
    let r = h
        .asked()
        .into_iter()
        .find(|r| &*r.kind != "priority" && r.prompt.contains("Flare of Malice"))
        .expect("the person was asked");
    assert_eq!(r.choices.len(), 2, "only the two greatest-MV creatures: {:?}", r.choices);
    assert!(r.choices.iter().all(|c| c.contains("Roughrider") || c.contains("Finks")));
    assert!(gy_has(&g, P0, "Goblin Roughrider"), "the person picked the Roughrider");
    assert!(has_perm(&g, P0, "Kitchen Finks"));
    assert!(has_perm(&g, P0, "Llanowar Elves") && has_perm(&g, P0, "Birds of Paradise"));
}

// ------------------------------------------------------------------ DSL "discard"
#[test]
fn dsl_discard_the_person_picks_the_card() {
    // Burglar Rat's sibling Nezumi Informant: "each opponent discards a card" (the DSL arm)
    let h = person(|r| if r.prompt.contains("iscard") { with(r, "Sol Ring") } else { None });
    let mut g = setup(&h);
    hand(&mut g, P0, &["Lightning Bolt", "Sol Ring", "Island"]);
    cast_p1(&mut g, "Nezumi Informant");
    assert!(asked(&h, "iscard"), "the person was asked");
    assert!(gy_has(&g, P0, "Sol Ring"));
    assert!(hand_has(&g, P0, "Lightning Bolt") && hand_has(&g, P0, "Island"));
}
