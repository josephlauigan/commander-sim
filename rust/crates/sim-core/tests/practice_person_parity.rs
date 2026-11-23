//! A person who answers with the AI's own picks gets the AI's result (`parity_common::person_matches_ai`): Gifts
//! Ungiven, Mindbreak Trap and Relic of Sauron's loot. The same scenario runs once with the AI deciding and once with a
//! scripted person; the state fingerprint and the public action lines must be equal.

mod parity_common;
mod play_common;
use parity_common::*;
use play_common::*;
use sim_core::control::Request;
use sim_core::engine::{stack, tutors};
use sim_core::ids::CardId;
use sim_core::state::{Ctx, StackItem, StackKind};
use std::collections::VecDeque;

// ------------------------------------------------------------------ Gifts Ungiven
fn gifts_game() -> Game {
    table(&["seph", "cloud", "kefka", "kyodai"])
}

/// the person's answers for Gifts: the pile the AI found (four cards that left the library) and the opponent the AI
/// chose (named in its "bins" line), or, with `other_opponent`, the first opponent who is not that one
fn gifts_script(
    before: &Game,
    after: &Game,
    lines: &[String],
    other_opponent: bool,
) -> Box<dyn FnMut(&Request) -> Answer + Send> {
    let mut pile: VecDeque<String> = gained(before, after, P0).into();
    let bins = lines.iter().find(|l| l.contains(" bins ")).expect("a bins line").clone();
    let who = bins.split("; ").nth(1).unwrap().split(" bins ").next().unwrap().to_string();
    Box::new(move |r: &Request| {
        if r.prompt.contains("target opponent") {
            let at = if other_opponent {
                r.choices.iter().position(|c| !c.starts_with(&who))
            } else {
                r.choices.iter().position(|c| c.starts_with(&who))
            };
            return Answer::Pick(at.expect("the opponent is offered"));
        }
        // search: any card of the pile still on offer
        let (i, k) = pile.iter().enumerate().find_map(|(i, n)| choice_of(r, n).map(|k| (i, k))).expect("a pile card");
        pile.remove(i);
        Answer::Pick(k)
    })
}

fn gifts_scenario<'a>(
    adapter: &'a dyn Fn(&Game, &Game, &[String]) -> Box<dyn FnMut(&Request) -> Answer + Send>,
) -> Scenario<'a> {
    Scenario {
        seat: P0,
        build: &gifts_game,
        ai: &|g| tutors::pile_tutor(g, P0, 4, 2).unwrap(),
        person_prep: &|_| {},
        person: &|g| tutors::pile_tutor(g, P0, 4, 2).unwrap(),
        adapter,
    }
}

#[test]
fn gifts_a_person_naming_the_ais_pile_and_opponent_gets_the_ais_result() {
    let p = person_matches_ai(&gifts_scenario(&|b, a, l| gifts_script(b, a, l, false)));
    assert!(
        p.asked.iter().any(|r| r.prompt.contains("target opponent")),
        "the person was asked who splits: {:?}",
        p.asked.iter().map(|r| &r.prompt).collect::<Vec<_>>()
    );
    assert!(p.ai.lines.iter().any(|l| l.contains(" bins ")), "{:?}", p.ai.lines);
}

// ------------------------------------------------------------------ Mindbreak Trap
/// the stack, bottom to top: P0's Opt, P2's Brainstorm, P1's Lightning Bolt; P1 has cast three spells this turn, so the
/// Trap costs {0} for either kind of player
fn trap_game() -> Game {
    let mut g = table(&["kyodai", "seph", "kefka"]);
    g.active = Some(P1);
    hand(&mut g, P0, &["Mindbreak Trap"]);
    for (p, n) in [(P0, "Opt"), (P2, "Brainstorm"), (P1, "Lightning Bolt")] {
        let c = hand(&mut g, p, &[n])[0];
        let i = g.player(p).hand.iter().position(|&x| x == c).unwrap();
        g.player_mut(p).hand.remove(i);
        g.stack_pushes += 1;
        g.stack.push(StackItem {
            id: g.stack_pushes,
            controller: p,
            card: Some(c),
            ctx: Ctx::default(),
            zone: "hand",
            imp: 6.0,
            aff: vec![],
            generic: true,
            kind: StackKind::Spell,
            name: n.to_string(),
            passed: vec![],
            countered: false,
            countered_by: None,
        });
    }
    let (st, cs) = (g.turn_stamp(), ["Lightning Bolt", "Brainstorm", "Opt"].map(|n| card(&g, n)).to_vec());
    g.player_mut(P1).turn_casts = Some((st, cs));
    g
}

#[test]
fn trap_a_person_naming_the_ais_spells_gets_the_ais_result() {
    let s = Scenario {
        seat: P0,
        build: &trap_game,
        // the AI's decision as `ai_respond` ends: the counter it picks for the top spell, cast at it
        ai: &|g| {
            let top = g.stack.last().unwrap();
            let (top_id, spell) = (top.id, top.card.unwrap());
            let ctr = stack::pick_counter(g, P0, spell).expect("the AI casts the Trap");
            assert!(stack::cast_counter_spell(g, P0, ctr, top_id).unwrap());
        },
        person_prep: &|_| {},
        person: &|g| {
            let trap = card(g, "Mindbreak Trap");
            assert_eq!(sim_core::human::human::cast_counterspell(g, P0, trap).unwrap(), None);
        },
        adapter: &|_, after, _| {
            // the first target is the one the Trap countered without the exile mark; the others carry it
            let by_trap = |x: &&StackItem| x.countered && x.countered_by.is_some();
            let first: Vec<String> =
                after.stack.iter().filter(by_trap).filter(|x| !x.ctx.exile_after).map(|x| x.name.clone()).collect();
            let more: Vec<String> =
                after.stack.iter().filter(by_trap).filter(|x| x.ctx.exile_after).map(|x| x.name.clone()).collect();
            let mut names: VecDeque<String> = first.into_iter().chain(more).collect();
            Box::new(move |r: &Request| match names.pop_front() {
                Some(n) => Answer::Pick(choice_of(r, &n).unwrap_or_else(|| panic!("{n} in {:?}", r.choices))),
                None => Answer::Decline,
            })
        },
    };
    let p = person_matches_ai(&s);
    // the AI exiled the opposing spells (Bolt as its target, Brainstorm with it) and left its own Opt
    let fp = p.ai.fingerprint.join("\n");
    assert!(fp.contains("Lightning Bolt of P1 countered=true"), "{fp}");
    assert!(fp.contains("Brainstorm of P2 countered=true"), "{fp}");
    assert!(fp.contains("Opt of P0 countered=false"), "{fp}");
}

// ------------------------------------------------------------------ Relic of Sauron
fn relic_game() -> Game {
    let mut g =
        table(&["urdragon", "miirym-temur-dragons", "urza-mono-blue-artifacts", "zur-esper-enchantment-control"]);
    g.active = Some(P0);
    perm(&mut g, P0, "Relic of Sauron");
    lands(&mut g, P0, "Mountain", 3, false);
    hand(&mut g, P0, &["Sol Ring", "Brainstorm"]);
    g
}

#[test]
fn relic_a_person_discarding_the_ais_card_gets_the_ais_loot() {
    let relic = |g: &mut Game| {
        let m = g.player(P0).perms.iter().copied().find(|&m| &*g.perm(m).name == "Relic of Sauron").unwrap();
        assert!(sim_core::dsl::activate(g, P0, m, 0).unwrap());
    };
    let s = Scenario {
        seat: P0,
        build: &relic_game,
        ai: &relic,
        // a person taps three Mountains for mana first (the AI pays {3} inside its activation), then activates
        person_prep: &|g| {
            while let Some(s) = sim_core::human::mana::sources(g, P0).into_iter().find(|s| s.name == "Mountain") {
                assert_eq!(sim_core::human::mana::tap(g, P0, s.id as i64, None).unwrap(), None);
            }
        },
        person: &relic,
        adapter: &|before, after, _| {
            // the card the AI discarded: the one that is in its graveyard now and was not before
            let name = |g: &Game, c: CardId| g.db.get(c).name.to_string();
            let old: Vec<String> = before.player(P0).gy.iter().map(|&c| name(before, c)).collect();
            let discarded =
                after.player(P0).gy.iter().map(|&c| name(after, c)).find(|n| !old.contains(n)).expect("a discard");
            Box::new(move |r: &Request| {
                Answer::Pick(choice_of(r, &discarded).unwrap_or_else(|| panic!("{discarded} in {:?}", r.choices)))
            })
        },
    };
    let p = person_matches_ai(&s);
    let asks: Vec<&String> = p.asked.iter().map(|r| &r.prompt).collect();
    assert!(asks.iter().any(|a| a.contains("Discard a card")), "asked what to discard: {asks:?}");
}

// ------------------------------------------------------------------ the helper is not vacuous
#[test]
#[should_panic(expected = "action lines")]
fn gifts_a_person_naming_another_opponent_is_noticed() {
    // same pile, but an opponent other than the AI's: only the "bins" line differs
    let _ = person_matches_ai(&gifts_scenario(&|b, a, l| gifts_script(b, a, l, true)));
}

#[test]
#[should_panic(expected = "state fingerprints")]
fn trap_a_person_exiling_fewer_spells_than_the_ai_is_noticed() {
    let s = Scenario {
        seat: P0,
        build: &trap_game,
        ai: &|g| {
            let top = g.stack.last().unwrap();
            let (top_id, spell) = (top.id, top.card.unwrap());
            let ctr = stack::pick_counter(g, P0, spell).unwrap();
            assert!(stack::cast_counter_spell(g, P0, ctr, top_id).unwrap());
        },
        person_prep: &|_| {},
        person: &|g| {
            let trap = card(g, "Mindbreak Trap");
            sim_core::human::human::cast_counterspell(g, P0, trap).unwrap();
        },
        // the person picks the Bolt and then says they are done: Brainstorm stays, the AI exiled it
        adapter: &|_, _, _| {
            let mut first = true;
            Box::new(move |r: &Request| {
                if std::mem::replace(&mut first, false) {
                    return Answer::Pick(choice_of(r, "Lightning Bolt").unwrap());
                }
                Answer::Decline
            })
        },
    };
    let _ = person_matches_ai(&s);
}
