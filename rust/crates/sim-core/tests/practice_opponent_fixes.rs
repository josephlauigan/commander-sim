//! Practice mode, opponent effects (2026-11-04): where an AI opponent's card makes the person sacrifice, discard,
//! pick or pay, the person decides (found by `practice_opponent_probe`). Each test puts the person at P0 with a scripted
//! answer, lets P1's card do its thing and checks both that the person was asked and that their answer is what happened.

mod play_common;
use play_common::*;
use sim_core::engine::combat::combat;
use sim_core::ids::PermId;
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

// ------------------------------------------------------------------ the individual cards
/// P1 puts a permanent onto the battlefield (its enter triggers run)
fn enter_p1(g: &mut Game, name: &str) -> PermId {
    let m = perm(g, P1, name);
    sim_core::engine::stack::flush_triggers(g).unwrap();
    m
}

fn answer_prompt(word: &'static str, choice: &'static str) -> Arc<ScriptHuman> {
    person(move |r| if r.prompt.contains(word) { with(r, choice) } else { None })
}

/// a person who answers every yes/no with `f(request)`, and passes at a priority
fn yn(f: impl Fn(&Request) -> bool + Send + Sync + 'static) -> Arc<ScriptHuman> {
    person(move |r| with(r, if f(r) { "yes" } else { "no" }))
}

#[test]
fn burglar_rat_and_elderfang_disciple_the_person_discards_what_they_choose() {
    for rat in ["Burglar Rat", "Elderfang Disciple"] {
        let h = answer_prompt("iscard", "Island");
        let mut g = setup(&h);
        hand(&mut g, P0, &["Lightning Bolt", "Sol Ring", "Island"]);
        enter_p1(&mut g, rat);
        assert!(asked(&h, "iscard"), "{rat}: the person was asked");
        assert!(gy_has(&g, P0, "Island"), "{rat}: the picked card was discarded");
        assert!(hand_has(&g, P0, "Lightning Bolt") && hand_has(&g, P0, "Sol Ring"));
    }
}

#[test]
fn chittering_rats_the_person_picks_the_card_to_put_back() {
    let h = answer_prompt("Chittering Rats", "Sol Ring");
    let mut g = setup(&h);
    hand(&mut g, P0, &["Lightning Bolt", "Sol Ring", "Island"]);
    enter_p1(&mut g, "Chittering Rats");
    assert!(asked(&h, "Chittering Rats"), "the person was asked");
    assert_eq!(g.player(P0).library.last().map(|&c| g.db.get(c).name.to_string()), Some("Sol Ring".to_string()));
    assert!(!hand_has(&g, P0, "Sol Ring") && hand_has(&g, P0, "Lightning Bolt") && hand_has(&g, P0, "Island"));
}

#[test]
fn archfiend_of_depravity_the_person_chooses_the_two_creatures_they_keep() {
    let h = person(|r| {
        if r.prompt.contains("Archfiend") {
            if r.prompt.contains("1 of 2") { with(r, "Llanowar") } else { with(r, "Birds") }
        } else {
            None
        }
    });
    let mut g = setup(&h);
    for n in ["Llanowar Elves", "Birds of Paradise", "Goblin Roughrider", "Armored Warhorse"] {
        perm(&mut g, P0, n);
    }
    enter_p1(&mut g, "Archfiend of Depravity");
    g.active = Some(P0);
    sim_core::engine::turn::end_step(&mut g, P0).unwrap();
    assert!(asked(&h, "Archfiend"), "the person was asked");
    assert!(has_perm(&g, P0, "Llanowar Elves") && has_perm(&g, P0, "Birds of Paradise"), "the two chosen stay");
    assert!(!has_perm(&g, P0, "Goblin Roughrider") && !has_perm(&g, P0, "Armored Warhorse"), "the rest are sacrificed");
}

#[test]
fn braids_cabal_minion_the_person_picks_what_to_sacrifice_in_their_upkeep() {
    let h = answer_prompt("Braids", "Sol Ring");
    let mut g = setup(&h);
    perm(&mut g, P0, "Armored Warhorse");
    perm(&mut g, P0, "Sol Ring");
    lands(&mut g, P0, "Forest", 3, false);
    enter_p1(&mut g, "Braids, Cabal Minion");
    g.active = Some(P0);
    sim_core::engine::turn::upkeep(&mut g, P0).unwrap();
    assert!(asked(&h, "Braids"), "the person was asked");
    assert!(!has_perm(&g, P0, "Sol Ring"), "the picked artifact went");
    assert!(has_perm(&g, P0, "Armored Warhorse") && g.player(P0).lands.len() == 3);
}

#[test]
fn braids_cabal_minion_the_person_can_sacrifice_a_land() {
    let h = answer_prompt("Braids", "Mountain");
    let mut g = setup(&h);
    perm(&mut g, P0, "Armored Warhorse");
    lands(&mut g, P0, "Forest", 2, false);
    lands(&mut g, P0, "Mountain", 1, false);
    enter_p1(&mut g, "Braids, Cabal Minion");
    g.active = Some(P0);
    sim_core::engine::turn::upkeep(&mut g, P0).unwrap();
    assert!(has_perm(&g, P0, "Armored Warhorse"));
    assert_eq!(g.player(P0).lands.len(), 2, "one land went");
    assert!(gy_has(&g, P0, "Mountain"), "the Mountain the person picked");
}

#[test]
fn councils_judgment_the_person_casts_their_own_vote() {
    let h = answer_prompt("vote", "Doubling Season");
    let mut g = setup(&h);
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P0, "Doubling Season");
    perm(&mut g, P0, "Mind Stone");
    cast_p1(&mut g, "Council's Judgment");
    assert!(asked(&h, "vote"), "the person was asked");
    assert!(!has_perm(&g, P0, "Doubling Season"), "the permanent the person voted for was exiled (it has a vote)");
}

#[test]
fn liliana_of_the_veil_the_person_picks_the_pile_they_sacrifice() {
    let h = person(|r| if r.prompt.contains("Liliana of the Veil") { Some(1) } else { None });
    let mut g = setup(&h);
    for n in ["Sol Ring", "Arcane Signet", "Mind Stone", "Doubling Season"] {
        perm(&mut g, P0, n);
    }
    let src = perm(&mut g, P1, "Liliana of the Veil");
    (sim_core::impls::rules::LOTV_ULT.eff)(&mut g, P1, src).unwrap();
    let r = h.asked().into_iter().find(|r| r.prompt.contains("Liliana of the Veil")).expect("the person was asked");
    assert_eq!(r.choices.len(), 2, "two piles: {:?}", r.choices);
    let left =
        ["Sol Ring", "Arcane Signet", "Mind Stone", "Doubling Season"].iter().filter(|n| has_perm(&g, P0, n)).count();
    assert_eq!(left, 2, "the second pile (two permanents) was sacrificed, the first stayed");
    assert!(r.choices[0].starts_with("Pile 1") && r.choices[1].starts_with("Pile 2"));
    // the permanents named in pile 1 are the ones still there
    for n in ["Sol Ring", "Arcane Signet", "Mind Stone", "Doubling Season"] {
        assert_eq!(r.choices[0].contains(n), has_perm(&g, P0, n), "{n}: pile 1 stays, pile 2 is sacrificed");
    }
}

#[test]
fn liliana_dreadhorde_general_the_person_keeps_one_permanent_of_each_type() {
    let h = person(|r| {
        if r.prompt.contains("Liliana, Dreadhorde General") {
            with(r, "Mind Stone")
                .or_else(|| with(r, "Birds"))
                .or_else(|| with(r, "Parallel Lives"))
                .or_else(|| with(r, "Forest"))
        } else {
            None
        }
    });
    let mut g = setup(&h);
    for n in ["Sol Ring", "Mind Stone", "Llanowar Elves", "Birds of Paradise", "Parallel Lives", "Doubling Season"] {
        perm(&mut g, P0, n);
    }
    lands(&mut g, P0, "Forest", 2, false);
    lands(&mut g, P0, "Island", 2, false);
    let src = perm(&mut g, P1, "Liliana, Dreadhorde General");
    (sim_core::impls::rules::LDG_ULT.eff)(&mut g, P1, src).unwrap();
    assert!(asked(&h, "Liliana, Dreadhorde General"), "the person was asked");
    assert!(has_perm(&g, P0, "Mind Stone") && !has_perm(&g, P0, "Sol Ring"), "the artifact the person kept");
    assert!(has_perm(&g, P0, "Birds of Paradise") && !has_perm(&g, P0, "Llanowar Elves"), "the creature");
    assert!(has_perm(&g, P0, "Parallel Lives") && !has_perm(&g, P0, "Doubling Season"), "the enchantment");
    assert_eq!(g.player(P0).lands.len(), 1, "one land");
    assert_eq!(&*g.perm(g.player(P0).lands[0]).name, "Forest", "the Forest the person picked");
}

#[test]
fn painful_quandary_the_person_chooses_to_discard_or_lose_life() {
    // discard: says yes, then picks the card
    let h = person(|r| if r.prompt.contains("Painful Quandary: discard a card") { with(r, "Island") } else { None });
    let mut g = setup(&h);
    hand(&mut g, P0, &["Lightning Bolt", "Island", "Sol Ring"]);
    enter_p1(&mut g, "Painful Quandary");
    g.active = Some(P0);
    let life = g.player(P0).life;
    let x = take(&mut g, P0, "Mind Stone");
    g.player_mut(P0).hand.push(x);
    sim_core::engine::cast::cast_card(&mut g, P0, x, "hand".into(), Ctx::default()).unwrap();
    assert!(asked(&h, "Painful Quandary"), "the person was asked");
    assert!(gy_has(&g, P0, "Island"), "the picked card was discarded");
    assert_eq!(g.player(P0).life, life, "and no life was lost");
    // life: says no
    let h = yn(|_| false);
    let mut g = setup(&h);
    hand(&mut g, P0, &["Lightning Bolt", "Island", "Sol Ring"]);
    enter_p1(&mut g, "Painful Quandary");
    g.active = Some(P0);
    let life = g.player(P0).life;
    let x = take(&mut g, P0, "Mind Stone");
    g.player_mut(P0).hand.push(x);
    sim_core::engine::cast::cast_card(&mut g, P0, x, "hand".into(), Ctx::default()).unwrap();
    assert_eq!(g.player(P0).life, life - 5, "the person chose to lose 5 life");
    assert!(hand_has(&g, P0, "Island") && hand_has(&g, P0, "Lightning Bolt"), "and kept every card");
}

#[test]
fn rakdos_patron_of_chaos_the_person_decides_and_picks_the_two_permanents() {
    let h = person(|r| {
        if r.prompt.contains("Rakdos: sacrifice which") {
            if r.prompt.contains("1 of 2") { with(r, "Sol Ring") } else { with(r, "Mind Stone") }
        } else {
            None
        }
    });
    let mut g = setup(&h);
    for n in ["Sol Ring", "Mind Stone", "Arcane Signet", "Doubling Season"] {
        perm(&mut g, P0, n);
    }
    enter_p1(&mut g, "Rakdos, Patron of Chaos");
    g.active = Some(P1);
    let hand_before = g.player(P1).hand.len();
    sim_core::engine::turn::end_step(&mut g, P1).unwrap();
    assert!(asked(&h, "Rakdos"), "the person was asked");
    assert!(!has_perm(&g, P0, "Sol Ring") && !has_perm(&g, P0, "Mind Stone"), "the two picked went");
    assert!(has_perm(&g, P0, "Arcane Signet") && has_perm(&g, P0, "Doubling Season"));
    assert_eq!(g.player(P1).hand.len(), hand_before, "so Rakdos' controller didn't draw");
}

#[test]
fn rakdos_patron_of_chaos_the_person_may_decline_and_the_opponent_draws() {
    let h = yn(|_| false);
    let mut g = setup(&h);
    for n in ["Sol Ring", "Mind Stone", "Arcane Signet"] {
        perm(&mut g, P0, n);
    }
    enter_p1(&mut g, "Rakdos, Patron of Chaos");
    g.active = Some(P1);
    let hand_before = g.player(P1).hand.len();
    sim_core::engine::turn::end_step(&mut g, P1).unwrap();
    assert!(has_perm(&g, P0, "Sol Ring") && has_perm(&g, P0, "Mind Stone") && has_perm(&g, P0, "Arcane Signet"));
    assert_eq!(g.player(P1).hand.len(), hand_before + 2, "the opponent drew two");
}

#[test]
fn single_combat_the_person_chooses_the_creature_they_keep() {
    let h = answer_prompt("Single Combat", "Llanowar");
    let mut g = setup(&h);
    for n in ["Llanowar Elves", "Goblin Roughrider", "Armored Warhorse"] {
        perm(&mut g, P0, n);
    }
    cast_p1(&mut g, "Single Combat");
    assert!(asked(&h, "Single Combat"), "the person was asked");
    assert!(has_perm(&g, P0, "Llanowar Elves"), "the one the person chose stays (the AI would keep the best)");
    assert!(!has_perm(&g, P0, "Goblin Roughrider") && !has_perm(&g, P0, "Armored Warhorse"));
}

#[test]
fn sword_of_feast_and_famine_the_person_chooses_the_discard() {
    let h = answer_prompt("iscard", "Island");
    let mut g = setup(&h);
    hand(&mut g, P0, &["Lightning Bolt", "Island", "Sol Ring"]);
    let sword = perm(&mut g, P1, "Sword of Feast and Famine");
    let a = perm(&mut g, P1, "Armored Warhorse");
    g.perm_mut(sword).attached = Some(a);
    g.perm_mut(a).sick = false;
    g.step = sim_core::state::Step::Combat;
    g.active = Some(P1);
    sim_core::engine::combat::resolve_combat(&mut g, P1, &[a], P0, &[a]).unwrap();
    assert!(asked(&h, "iscard"), "the person was asked");
    assert!(gy_has(&g, P0, "Island"));
    assert!(hand_has(&g, P0, "Lightning Bolt") && hand_has(&g, P0, "Sol Ring"));
}

#[test]
fn archon_of_cruelty_the_person_chooses_the_discard() {
    let h = answer_prompt("iscard", "Island");
    let mut g = setup(&h);
    hand(&mut g, P0, &["Lightning Bolt", "Island", "Sol Ring"]);
    perm(&mut g, P0, "Armored Warhorse");
    sim_core::engine::combat::archon_attack(&mut g, P1, P0).unwrap();
    assert!(gy_has(&g, P0, "Island"));
    assert!(hand_has(&g, P0, "Lightning Bolt") && hand_has(&g, P0, "Sol Ring"));
}

#[test]
fn the_tabernacle_the_person_pays_for_the_creatures_they_choose() {
    // yes for the Warhorse, no for the Elves
    let h = yn(|r| r.prompt.contains("Warhorse"));
    let mut g = setup(&h);
    perm(&mut g, P0, "Armored Warhorse");
    perm(&mut g, P0, "Llanowar Elves");
    lands(&mut g, P0, "Forest", 4, false);
    let c = take(&mut g, P1, "The Tabernacle at Pendrell Vale");
    g.add_land(P1, c, false);
    g.active = Some(P0);
    sim_core::engine::turn::upkeep(&mut g, P0).unwrap();
    assert!(asked(&h, "Tabernacle"), "the person was asked");
    assert!(has_perm(&g, P0, "Armored Warhorse"), "paid for the Warhorse");
    assert!(!has_perm(&g, P0, "Llanowar Elves"), "didn't pay for the Elves");
}

#[test]
fn wandering_archaic_the_person_decides_whether_to_pay_two() {
    for (pay, copied) in [(true, false), (false, true)] {
        let h = yn(move |_| pay);
        let mut g = setup(&h);
        lands(&mut g, P0, "Island", 4, false);
        enter_p1(&mut g, "Wandering Archaic // Explore the Vastlands");
        g.active = Some(P0);
        let x = take(&mut g, P0, "Brainstorm");
        g.player_mut(P0).hand.push(x);
        let tapped = |g: &Game| g.player(P0).lands.iter().filter(|&&l| g.perm(l).tapped).count();
        let before = tapped(&g);
        sim_core::engine::cast::cast_card(&mut g, P0, x, "hand".into(), Ctx::default()).unwrap();
        assert!(asked(&h, "Wandering Archaic"), "the person was asked");
        if pay {
            assert!(tapped(&g) >= before + 2, "the person paid {{2}}");
        } else {
            assert_eq!(tapped(&g), before, "the person didn't pay");
        }
        let _ = copied;
    }
}

// ------------------------------------------------------------------ attack taxes
fn tax_table() -> Game {
    let mut g = table(&["sauron", "cloud"]);
    perm(&mut g, P1, "Ghostly Prison");
    g
}

#[test]
fn attack_tax_the_person_picks_which_creatures_they_can_afford_to_send() {
    // two attackers declared, {2} to pay: one can attack. The person picks the 1/1, not the strongest
    let mut g = tax_table();
    perm(&mut g, P0, "Grave Titan"); // 6/6
    token(&mut g, P0, 1);
    lands(&mut g, P0, "Plains", 2, false);
    let h = seat(&mut g, P0, vec![picks(&[0, 1]), pick(1), pick(0)]);
    combat(&mut g, P0).unwrap();
    assert!(asked(&h, "Which creature attacks"), "the person was asked which creature attacks");
    assert_eq!(g.player(P1).life, 40 - 1, "the 1/1 the person picked attacked, the Titan stayed home");
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| !g.perm(l).tapped).count(), 0, "and {{2}} was paid");
}

#[test]
fn attack_tax_the_person_may_send_fewer_than_they_can_pay_for() {
    let mut g = tax_table();
    perm(&mut g, P0, "Grave Titan");
    token(&mut g, P0, 1);
    token(&mut g, P0, 1);
    lands(&mut g, P0, "Plains", 4, false); // pays for two of three
    // pick the Titan, then "no more attackers" (index 2 of the two left plus the cancel)
    let h = seat(&mut g, P0, vec![picks(&[0, 1, 2]), pick(0), pick(2), pick(0)]);
    combat(&mut g, P0).unwrap();
    assert!(asked(&h, "Which creature attacks"));
    assert_eq!(g.player(P1).life, 40 - 6, "only the Titan attacked");
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| !g.perm(l).tapped).count(), 2, "{{2}} for one attacker");
}

// ------------------------------------------------------------------ Chain of Vapor
fn chain_table(h: &Arc<ScriptHuman>) -> Game {
    let mut g = setup(h);
    perm(&mut g, P0, "Sol Ring"); // the AI's target
    perm(&mut g, P1, "Armored Warhorse");
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Forest", 1, false);
    g
}

#[test]
fn chain_of_vapor_the_person_may_sacrifice_a_land_and_copy_it_at_a_new_target() {
    let h = person(|r| {
        if r.prompt.contains("which land") {
            with(r, "Forest")
        } else if r.prompt.contains("returns which") {
            with(r, "Warhorse")
        } else {
            None // yes to the copy
        }
    });
    let mut g = chain_table(&h);
    cast_p1(&mut g, "Chain of Vapor");
    assert!(hand_has(&g, P0, "Sol Ring"), "the AI's Chain returned the person's Sol Ring");
    assert!(asked(&h, "sacrifice a land to copy"), "the person was offered the copy");
    assert!(asked(&h, "which land") && asked(&h, "returns which"));
    assert!(gy_has(&g, P0, "Forest") && g.player(P0).lands.len() == 2, "the Forest the person picked was sacrificed");
    assert!(!has_perm(&g, P1, "Armored Warhorse"), "the copy returned the new target");
    assert!(g.player(P1).hand.iter().any(|&c| &*g.db.get(c).name == "Armored Warhorse"));
}

#[test]
fn chain_of_vapor_the_person_may_decline() {
    let h = person(|r| if r.prompt.contains("sacrifice a land to copy") { with(r, "no") } else { None });
    let mut g = chain_table(&h);
    cast_p1(&mut g, "Chain of Vapor");
    assert!(asked(&h, "sacrifice a land to copy"));
    assert!(hand_has(&g, P0, "Sol Ring"));
    assert_eq!(g.player(P0).lands.len(), 3, "no land sacrificed");
    assert!(has_perm(&g, P1, "Armored Warhorse"), "and no copy");
}
