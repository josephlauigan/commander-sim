//! Practice mode (2026-11-04, gaps found by the practice probes), Agent C's cards: the Alela, Sheoldred, Thrasios,
//! Y'shtola and Galadriel cards whose activated ability the person could not use, or whose choice the AI made for them.
//! Each test asks the person and checks that the answer is what happens.

mod play_common;
use play_common::*;
use sim_core::engine::combat::combat;
use sim_core::human::combat::attack_candidates;
use sim_core::human::human::{AbilityKind, abilities_of, apply};
use sim_core::state::{DataKey, Step, Val};
use std::sync::Arc;

/// a person who answers `f` first and otherwise passes priority and declines (a "cancel" at the end of a list)
fn person(g: &mut Game, f: impl FnMut(&Request) -> Option<Answer> + Send + 'static) -> Arc<ScriptHuman> {
    let mut f = f;
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        f(r).unwrap_or_else(|| {
            if &*r.kind == "priority" { Answer::Action(json!({"do": "pass"})) } else { Answer::Decline }
        })
    }));
    g.humans.add(P0, h.clone());
    h
}

/// pick the first choice containing `word`
fn text(r: &Request, word: &str) -> Option<Answer> {
    Some(Answer::Pick(
        r.choices.iter().position(|c| c.contains(word)).unwrap_or_else(|| panic!("no {word:?} in {:?}", r.choices)),
    ))
}

/// the prompts the person was asked, "kind: prompt"
fn asked(h: &ScriptHuman) -> Vec<String> {
    h.asked().iter().filter(|r| &*r.kind != "priority").map(|r| format!("{}: {}", r.kind, r.prompt)).collect()
}

fn asked_has(h: &ScriptHuman, what: &str) -> bool {
    asked(h).iter().any(|l| l.contains(what))
}

/// run the ability of m whose label contains `word`; the reason it can't be done, if any
fn use_ability(g: &mut Game, p: PlayerId, m: PermId, word: &str) -> Option<String> {
    let list = abilities_of(g, p, m).unwrap();
    let Some((_, kind)) = list.into_iter().find(|(l, _)| l.contains(word)) else {
        panic!("no ability with {word:?} on {}", g.perm(m).name)
    };
    let AbilityKind::Extra(act) = kind else { panic!("not an extra ability") };
    act(g, p, m).unwrap()
}

fn on_table(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    perms_named(g, p, name).into_iter().find(|&m| g.perm(m).on_bf)
}

/// the card goes from the library to the person's hand and is cast with `{"do": "cast"}`
fn cast_from_hand(g: &mut Game, name: &str) {
    let i = g.player(P0).hand.iter().position(|&c| &*g.db.get(c).name == name).unwrap();
    apply(g, P0, &json!({"do": "cast", "card": i})).unwrap();
}

// ================================================================== activated abilities
#[test]
fn angelic_page_pumps_the_creature_you_pick() {
    let mut g = table(&["alela", "seph"]);
    let page = perm(&mut g, P0, "Angelic Page");
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let elder = perm(&mut g, P0, "Sakura-Tribe Elder");
    // no combat: nothing to target
    let why = use_ability(&mut g, P0, page, "+1/+1");
    assert!(why.is_some_and(|w| w.contains("attacking or blocking")));
    g.in_combat = vec![elves, elder];
    let h = person(&mut g, |r| r.prompt.contains("Angelic Page").then(|| text(r, "Sakura")).flatten());
    assert_eq!(use_ability(&mut g, P0, page, "+1/+1"), None);
    assert!(asked_has(&h, "which attacking or blocking creature"));
    assert_eq!(g.perm(elder).eot_pt, (1, 1));
    assert_eq!(g.perm(elves).eot_pt, (0, 0));
    assert!(g.perm(page).tapped, "{{T}} is paid");
    assert!(use_ability(&mut g, P0, page, "+1/+1").is_some_and(|w| w.contains("tapped")));
}

#[test]
fn cabal_patriarch_sacrifices_what_you_pick() {
    let mut g = table(&["alela", "seph"]);
    let pat = perm(&mut g, P0, "Cabal Patriarch");
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let birds = perm(&mut g, P1, "Birds of Paradise");
    pool(&mut g, P0, &[('B', 1), ('C', 2)]);
    let h = person(&mut g, |r| {
        if r.prompt.contains("-2/-2 on which") {
            text(r, "Birds")
        } else if r.prompt.contains("sacrifice which") {
            text(r, "Llanowar")
        } else {
            None
        }
    });
    assert_eq!(use_ability(&mut g, P0, pat, "sacrifice a creature"), None);
    assert!(asked_has(&h, "sacrifice which creature"));
    assert!(!g.perm(birds).on_bf, "-2/-2 kills the 0/1");
    assert!(!g.perm(elves).on_bf, "the creature the person picked is sacrificed");
    assert!(g.perm(pat).on_bf);
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn cabal_patriarch_can_exile_a_creature_card_instead() {
    let mut g = table(&["alela", "seph"]);
    let pat = perm(&mut g, P0, "Cabal Patriarch");
    let birds = perm(&mut g, P1, "Birds of Paradise");
    let titan = card(&g, "Grave Titan");
    let elves = card(&g, "Llanowar Elves");
    g.player_mut(P0).gy.extend([titan, elves]);
    pool(&mut g, P0, &[('B', 1), ('C', 2)]);
    person(&mut g, |r| {
        if r.prompt.contains("exile which") {
            text(r, "Llanowar")
        } else if r.prompt.contains("-2/-2 on which") {
            text(r, "Birds")
        } else {
            None
        }
    });
    assert_eq!(use_ability(&mut g, P0, pat, "exile a creature card"), None);
    assert!(!g.perm(birds).on_bf);
    assert!(g.player(P0).exile.contains(&elves) && g.player(P0).gy.contains(&titan));
}

#[test]
fn credit_voucher_shuffles_the_cards_you_pick_and_draws_that_many() {
    let mut g = table(&["alela", "seph"]);
    let v = perm(&mut g, P0, "Credit Voucher");
    let cs = hand(&mut g, P0, &["Plains", "Island", "Swamp"]);
    pool(&mut g, P0, &[('C', 2)]);
    let lib = g.player(P0).library.len();
    let n = std::sync::atomic::AtomicUsize::new(0);
    let h = person(&mut g, move |r| {
        if r.prompt.contains("shuffle which card") {
            // the Plains, then the Island, then done
            return Some(if n.fetch_add(1, std::sync::atomic::Ordering::SeqCst) < 2 {
                Answer::Pick(0)
            } else {
                Answer::Decline
            });
        }
        None
    });
    assert_eq!(use_ability(&mut g, P0, v, "shuffle any number"), None);
    assert_eq!(asked(&h).iter().filter(|l| l.contains("shuffle which card")).count(), 3);
    assert!(!g.perm(v).on_bf, "sacrificed");
    assert_eq!(g.player(P0).hand.len(), 3, "the Swamp and two cards drawn");
    assert!(g.player(P0).hand.contains(&cs[2]));
    assert_eq!(g.player(P0).library.len(), lib, "two in, two drawn");
}

#[test]
fn crystal_chimes_returns_every_enchantment_card() {
    let mut g = table(&["alela", "seph"]);
    let c = perm(&mut g, P0, "Crystal Chimes");
    let (a, b) = (card(&g, "Propaganda"), card(&g, "Doubling Season"));
    g.player_mut(P0).gy.extend([a, b]);
    pool(&mut g, P0, &[('C', 3)]);
    person(&mut g, |_| None);
    assert_eq!(use_ability(&mut g, P0, c, "enchantment cards"), None);
    assert!(g.player(P0).hand.contains(&a) && g.player(P0).hand.contains(&b), "even one the AI would leave");
    assert!(!g.perm(c).on_bf);
}

#[test]
fn emry_casts_the_artifact_card_you_pick() {
    let mut g = table(&["alela", "seph"]);
    let e = perm(&mut g, P0, "Emry, Lurker of the Loch");
    let (ring, sig) = (card(&g, "Sol Ring"), card(&g, "Arcane Signet"));
    g.player_mut(P0).gy.extend([ring, sig]);
    pool(&mut g, P0, &[('C', 2)]);
    let h = person(&mut g, |r| r.prompt.contains("Emry").then(|| text(r, "Arcane Signet")).flatten());
    assert_eq!(use_ability(&mut g, P0, e, "artifact card"), None);
    assert!(asked_has(&h, "cast which artifact card"));
    assert!(on_table(&g, P0, "Arcane Signet").is_some());
    assert!(g.player(P0).gy.contains(&ring), "the other stays");
    assert!(g.perm(e).tapped);
}

#[test]
fn mirage_mirror_becomes_the_copy_you_pick_until_end_of_turn() {
    let mut g = table(&["alela", "seph"]);
    let mm = perm(&mut g, P0, "Mirage Mirror");
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Doubling Season");
    pool(&mut g, P0, &[('C', 2)]);
    let h = person(&mut g, |r| r.prompt.contains("Mirage Mirror").then(|| text(r, "Doubling")).flatten());
    assert_eq!(use_ability(&mut g, P0, mm, "becomes a copy"), None);
    assert!(asked_has(&h, "become a copy of which"));
    assert_eq!(g.perm(mm).name, "Doubling Season");
    sim_core::impls::alela::mirror_revert(&mut g).unwrap();
    assert_eq!(g.perm(mm).name, "Mirage Mirror", "it ends with the turn");
}

#[test]
fn wispdrinker_vampire_gives_the_small_creatures_deathtouch_and_lifelink() {
    let mut g = table(&["alela", "seph"]);
    let w = perm(&mut g, P0, "Wispdrinker Vampire");
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let titan = perm(&mut g, P0, "Grave Titan");
    pool(&mut g, P0, &[('W', 1), ('B', 1), ('C', 5)]);
    person(&mut g, |_| None);
    assert_eq!(use_ability(&mut g, P0, w, "deathtouch and lifelink"), None);
    assert!(g.perm(elves).eot_kw.contains(&"deathtouch") && g.perm(elves).eot_kw.contains(&"lifelink"));
    assert!(g.perm(titan).eot_kw.is_empty(), "power 6 is too big");
}

#[test]
fn gix_plays_what_you_choose_from_the_top_of_their_library() {
    let mut g = table(&["sheoldred", "seph"]);
    let gix = perm(&mut g, P0, "Gix, Yawgmoth Praetor");
    hand(&mut g, P0, &["Plains", "Island"]);
    let (forest, ring) = (card(&g, "Forest"), card(&g, "Sol Ring"));
    g.player_mut(P1).library.extend([forest, ring]); // the Sol Ring is on top
    pool(&mut g, P0, &[('B', 3), ('C', 4)]);
    let h = person(&mut g, |r| {
        if r.prompt.contains("X = ?") {
            text(r, "X = 2")
        } else if r.prompt.contains("discard which") {
            Some(Answer::Pick(0))
        } else if r.prompt.contains("cast Sol Ring") {
            Some(Answer::Pick(0))
        } else if r.prompt.contains("play Forest") {
            Some(Answer::Pick(1)) // no
        } else {
            None
        }
    });
    assert_eq!(use_ability(&mut g, P0, gix, "discard X cards"), None);
    assert!(asked_has(&h, "cast Sol Ring"), "asked: {:?}", asked(&h));
    assert!(on_table(&g, P0, "Sol Ring").is_some(), "cast for free");
    assert!(g.player(P0).hand.is_empty() && g.player(P0).gy.len() == 2, "X = 2 cards discarded");
    assert!(g.player(P1).exile.contains(&forest), "the card the person doesn't play stays exiled");
    assert!(g.player(P0).lands.is_empty());
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn gix_plays_a_land_with_the_land_drop() {
    let mut g = table(&["sheoldred", "seph"]);
    let gix = perm(&mut g, P0, "Gix, Yawgmoth Praetor");
    hand(&mut g, P0, &["Plains"]);
    let forest = card(&g, "Forest");
    g.player_mut(P1).library.push(forest);
    pool(&mut g, P0, &[('B', 3), ('C', 4)]);
    person(&mut g, |r| {
        if r.prompt.contains("X = ?") {
            text(r, "X = 1")
        } else if r.prompt.contains("play Forest") {
            Some(Answer::Pick(0))
        } else {
            Some(Answer::Pick(0)).filter(|_| r.prompt.contains("discard which"))
        }
    });
    assert_eq!(use_ability(&mut g, P0, gix, "discard X cards"), None);
    assert_eq!(g.player(P0).lands.len(), 1);
    assert_eq!(g.player(P0).lands_played, 1, "it was the land drop");
}

#[test]
fn dauthi_voidwalker_plays_the_exiled_card_you_pick() {
    let mut g = table(&["thrasios", "seph"]);
    let d = perm(&mut g, P0, "Dauthi Voidwalker");
    let (ring, sig) = (card(&g, "Sol Ring"), card(&g, "Arcane Signet"));
    g.player_mut(P1).exile.extend([ring, sig]);
    let void = Val::List([ring, sig].iter().map(|&c| Val::List(vec![Val::Card(c), Val::Player(P1)])).collect());
    g.perm_mut(d).data.set(DataKey::Void, void);
    let h = person(&mut g, |r| r.prompt.contains("Dauthi").then(|| text(r, "Arcane Signet")).flatten());
    assert_eq!(use_ability(&mut g, P0, d, "void counter"), None);
    assert!(asked_has(&h, "play which exiled card"));
    assert!(on_table(&g, P0, "Arcane Signet").is_some());
    assert!(on_table(&g, P0, "Sol Ring").is_none());
    assert!(!g.perm(d).on_bf, "sacrificed");
}

#[test]
fn resplendent_angel_pumps_whenever_you_have_the_mana() {
    let mut g = table(&["thrasios", "seph"]);
    let a = perm(&mut g, P0, "Resplendent Angel");
    assert!(use_ability(&mut g, P0, a, "+2/+2").is_some(), "no mana in the pool");
    pool(&mut g, P0, &[('W', 6)]);
    person(&mut g, |_| None);
    assert_eq!(use_ability(&mut g, P0, a, "+2/+2"), None);
    assert_eq!(g.perm(a).eot_pt, (2, 2));
    assert!(g.perm(a).eot_kw.contains(&"lifelink"));
}

#[test]
fn venat_exiles_the_permanent_you_pick_and_transforms() {
    let mut g = table(&["yshtola", "seph"]);
    let v = perm(&mut g, P0, "Venat, Heart of Hydaelyn // Hydaelyn, the Mothercrystal");
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Doubling Season");
    pool(&mut g, P0, &[('C', 7)]);
    let h = person(&mut g, |r| r.prompt.contains("Hero's Sundering").then(|| text(r, "Doubling")).flatten());
    g.active = Some(P1);
    assert!(use_ability(&mut g, P0, v, "Hero's Sundering").is_some_and(|w| w.contains("turn")), "sorcery speed");
    g.active = Some(P0);
    g.step = Step::Main1;
    assert_eq!(use_ability(&mut g, P0, v, "Hero's Sundering"), None);
    assert!(asked_has(&h, "exile which nonland permanent"));
    assert!(on_table(&g, P1, "Doubling Season").is_none() && on_table(&g, P1, "Sol Ring").is_some());
    assert!(g.perm(v).data.truthy(DataKey::Hydaelyn));
    assert!(g.perm(v).tapped);
}

// ================================================================== choices the AI made
#[test]
fn airlift_chaplain_offers_the_milled_plains() {
    let mut g = table(&["alela", "seph"]);
    let (plains, ring, forest) = (card(&g, "Plains"), card(&g, "Sol Ring"), card(&g, "Forest"));
    g.player_mut(P0).library.extend([forest, ring, plains]); // the Plains is on top
    let h = person(&mut g, |r| r.prompt.contains("Airlift Chaplain").then(|| text(r, "Plains")).flatten());
    let c = perm(&mut g, P0, "Airlift Chaplain");
    assert!(asked_has(&h, "put a milled card into your hand"));
    assert!(g.player(P0).hand.contains(&plains));
    assert_eq!(g.perm(c).plus, 0, "no counter when you take a card");
}

#[test]
fn airlift_chaplain_gets_the_counter_when_you_decline() {
    let mut g = table(&["alela", "seph"]);
    let (plains, ring, forest) = (card(&g, "Plains"), card(&g, "Sol Ring"), card(&g, "Forest"));
    g.player_mut(P0).library.extend([forest, ring, plains]);
    person(&mut g, |_| None);
    let c = perm(&mut g, P0, "Airlift Chaplain");
    assert!(!g.player(P0).hand.contains(&plains));
    assert_eq!(g.perm(c).plus, 1);
}

#[test]
fn pileated_provisioner_puts_the_counter_where_you_say() {
    let mut g = table(&["alela", "seph"]);
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let elder = perm(&mut g, P0, "Sakura-Tribe Elder");
    let h = person(&mut g, |r| r.prompt.contains("Pileated").then(|| text(r, "Sakura")).flatten());
    perm(&mut g, P0, "Pileated Provisioner");
    assert!(asked_has(&h, "creature without flying"));
    assert_eq!(g.perm(elder).plus, 1);
    assert_eq!(g.perm(elves).plus, 0);
}

#[test]
fn plan_for_all_outcomes_lets_you_choose_the_permanent_and_the_end_of_the_library() {
    let mut g = table(&["alela", "seph"]);
    let ring = perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P1, "Doubling Season");
    let ring_card = g.perm(ring).card();
    let h = person(&mut g, |r| {
        if r.prompt.contains("which other nonland") {
            text(r, "Sol Ring")
        } else if r.prompt.contains("which end") {
            text(r, "bottom")
        } else {
            None
        }
    });
    perm(&mut g, P0, "Plan for All Outcomes");
    assert!(asked_has(&h, "which end of your library"));
    assert!(!g.perm(ring).on_bf);
    assert_eq!(g.player(P0).library[0], ring_card, "on the bottom");
}

#[test]
fn plan_for_all_outcomes_on_an_opponents_permanent_puts_it_on_top() {
    let mut g = table(&["alela", "seph"]);
    let ds = perm(&mut g, P1, "Doubling Season");
    let card_id = g.perm(ds).card();
    let h = person(&mut g, |r| r.prompt.contains("which other nonland").then(|| text(r, "Doubling")).flatten());
    perm(&mut g, P0, "Plan for All Outcomes");
    assert!(!asked_has(&h, "which end"), "their owner chooses, not the person");
    assert!(!g.perm(ds).on_bf);
    assert_eq!(*g.player(P1).library.last().unwrap(), card_id);
}

#[test]
fn rivers_rebuke_asks_whose_permanents_to_return() {
    let mut g = table(&["galadriel", "seph"]);
    hand(&mut g, P0, &["River's Rebuke"]);
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Llanowar Elves");
    lands(&mut g, P1, "Forest", 2, false);
    perm(&mut g, P0, "Arcane Signet");
    pool(&mut g, P0, &[('U', 2), ('C', 4)]);
    let h = person(&mut g, |r| {
        if r.prompt.contains("nonland permanents of which player") {
            r.choices.iter().position(|c| c.contains("(player,") && !c.contains("(you)")).map(Answer::Pick)
        } else {
            None
        }
    });
    cast_from_hand(&mut g, "River's Rebuke");
    assert!(asked_has(&h, "of which player"));
    assert!(g.player(P1).perms.is_empty(), "their nonland permanents are bounced");
    assert_eq!(g.player(P1).lands.len(), 2, "lands stay");
    assert_eq!(g.player(P1).hand.len(), 2);
    assert!(on_table(&g, P0, "Arcane Signet").is_some(), "yours stay");
}

#[test]
fn perigee_beckoner_gives_its_grant_to_the_creature_you_pick() {
    let mut g = table(&["sheoldred", "seph"]);
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let elder = perm(&mut g, P0, "Sakura-Tribe Elder");
    let h = person(&mut g, |r| r.prompt.contains("Perigee Beckoner").then(|| text(r, "Llanowar")).flatten());
    perm(&mut g, P0, "Perigee Beckoner");
    assert!(asked_has(&h, "Perigee Beckoner: +2/+0"));
    assert_eq!(g.perm(elves).eot_pt.0, 2);
    assert_eq!(g.perm(elder).eot_pt.0, 0);
}

#[test]
fn phyrexian_delver_returns_the_creature_card_you_pick() {
    let mut g = table(&["sheoldred", "seph"]);
    let (titan, elves) = (card(&g, "Grave Titan"), card(&g, "Llanowar Elves"));
    g.player_mut(P0).gy.extend([titan, elves]);
    let h = person(&mut g, |r| r.prompt.contains("Phyrexian Delver").then(|| text(r, "Llanowar")).flatten());
    perm(&mut g, P0, "Phyrexian Delver");
    assert!(asked_has(&h, "Phyrexian Delver: return which creature card"));
    assert!(on_table(&g, P0, "Llanowar Elves").is_some());
    assert!(g.player(P0).gy.contains(&titan));
    assert_eq!(g.player(P0).life, 40 - 1, "you lose life equal to its mana value");
}

#[test]
fn raise_dead_returns_the_creature_card_you_pick() {
    let mut g = table(&["sheoldred", "seph"]);
    hand(&mut g, P0, &["Raise Dead"]);
    let (titan, elves) = (card(&g, "Grave Titan"), card(&g, "Llanowar Elves"));
    g.player_mut(P0).gy.extend([titan, elves]);
    pool(&mut g, P0, &[('B', 1)]);
    let h = person(&mut g, |r| r.prompt.contains("Raise Dead").then(|| text(r, "Llanowar")).flatten());
    cast_from_hand(&mut g, "Raise Dead");
    assert!(asked_has(&h, "Raise Dead: return which card"));
    assert!(g.player(P0).hand.contains(&elves));
    assert!(g.player(P0).gy.contains(&titan));
}

#[test]
fn natural_order_asks_for_the_sacrifice_and_the_search() {
    let mut g = table(&["thrasios", "seph"]);
    hand(&mut g, P0, &["Natural Order"]);
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let birds = perm(&mut g, P0, "Birds of Paradise");
    let hoof = card(&g, "Craterhoof Behemoth");
    g.player_mut(P0).library.push(hoof);
    pool(&mut g, P0, &[('G', 2), ('C', 2)]);
    let h = person(&mut g, |r| {
        if r.prompt.contains("sacrifice which") {
            text(r, "Birds")
        } else if &*r.kind == "search" {
            text(r, "Craterhoof")
        } else {
            None
        }
    });
    cast_from_hand(&mut g, "Natural Order");
    assert!(asked_has(&h, "Natural Order: sacrifice which permanent"), "asked: {:?}", asked(&h));
    assert!(!g.perm(birds).on_bf && g.perm(elves).on_bf, "the person picked the Birds");
    assert!(on_table(&g, P0, "Craterhoof Behemoth").is_some(), "the person picked the creature to find");
}

#[test]
fn the_wasp_gives_hexproof_to_the_hero_you_pick() {
    let mut g = table(&["thrasios", "seph"]);
    let ant = perm(&mut g, P0, "Ant-Man, Elusive Avenger");
    let h = person(&mut g, |r| r.prompt.contains("The Wasp").then(|| text(r, "Ant-Man")).flatten());
    let wasp = perm(&mut g, P0, "The Wasp, Winsome Avenger");
    assert!(asked_has(&h, "which Hero gains hexproof"));
    assert!(g.perm(ant).eot_kw.contains(&"hexproof"));
    assert!(!g.perm(wasp).eot_kw.contains(&"hexproof"));
}

#[test]
fn the_wasp_taps_the_creature_you_pick_when_it_attacks() {
    let mut g = table(&["thrasios", "seph"]);
    let wasp = perm(&mut g, P0, "The Wasp, Winsome Avenger");
    let elves = perm(&mut g, P1, "Llanowar Elves");
    let birds = perm(&mut g, P1, "Birds of Paradise");
    let i = attack_candidates(&g, P0).iter().position(|&m| m == wasp).unwrap();
    let h = person(&mut g, move |r| {
        if &*r.kind == "attack" {
            Some(Answer::Picks(vec![i]))
        } else if r.prompt.contains("The Wasp: tap which") {
            text(r, "Llanowar")
        } else {
            None
        }
    });
    combat(&mut g, P0).unwrap();
    assert!(asked_has(&h, "The Wasp: tap which creature"), "asked: {:?}", asked(&h));
    assert!(g.perm(elves).tapped && !g.perm(birds).tapped);
}

#[test]
fn deep_analysis_asks_which_player_draws() {
    let mut g = table(&["yshtola", "seph"]);
    hand(&mut g, P0, &["Deep Analysis"]);
    pool(&mut g, P0, &[('U', 1), ('C', 3)]);
    let h = person(&mut g, |r| {
        if r.prompt.contains("which player draws") {
            r.choices.iter().position(|c| c.contains("(player,") && !c.contains("(you)")).map(Answer::Pick)
        } else {
            None
        }
    });
    let (mine, theirs) = (g.player(P0).hand.len(), g.player(P1).hand.len());
    cast_from_hand(&mut g, "Deep Analysis");
    assert!(asked_has(&h, "Deep Analysis: which player draws"));
    assert_eq!(g.player(P1).hand.len(), theirs + 2, "the opponent draws two");
    assert_eq!(g.player(P0).hand.len(), mine - 1, "the caster draws nothing");
}
