//! Practice mode for the Trostani deck: what a person is offered and asked where the AI's code used to decide for them
//! (populate, Sterling Grove, Innkeeper's Talent, Emiel the Blessed, Treebeard, Gracious Host). Each test seats a person at
//! P0 (answering a priority with a pass), and the expected numbers come from the Oracle text. The AI's own path is
//! untouched: the tests in `cards_trostani_*.rs` cover it.

mod play_common;
use play_common::*;
use sim_core::engine::hooks::fire_trigger;
use sim_core::engine::life;
use sim_core::engine::zones::{Tokens, make_tokens};
use sim_core::hooks::{Action, Call, Event};
use sim_core::human::human;
use sim_core::state::Step;
use std::sync::Arc;

const T: &str = "Trostani, Selesnya's Voice";

fn tbl() -> Game {
    let mut g = table(&["trostani", "seph"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g
}

/// seats a person at P0: `pick` answers every question that is not a priority (None: the first choice, or yes)
fn seat_person(g: &mut Game, pick: impl Fn(&Request) -> Option<usize> + Send + Sync + 'static) -> Arc<ScriptHuman> {
    let h = Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        match pick(r) {
            Some(k) => Answer::Pick(k),
            None if r.choices.is_empty() => Answer::Yes,
            None => Answer::Pick(0),
        }
    }));
    g.humans.add(P0, h.clone());
    h
}

/// the index of the first choice containing `word`
fn with(r: &Request, word: &str) -> Option<usize> {
    r.choices.iter().position(|c| c.contains(word))
}

fn asked(h: &ScriptHuman, text: &str) -> usize {
    h.asked().iter().filter(|r| r.prompt.contains(text)).count()
}

/// what the person can activate on m now
fn labels(g: &mut Game, m: PermId) -> Vec<String> {
    human::abilities_of(g, P0, m).unwrap().into_iter().map(|a| a.0).collect()
}

/// the person uses the card-code option whose label contains `word` (the same hook `abilities_of` lists)
fn use_option(g: &mut Game, m: PermId, post: Option<bool>, word: &str) -> bool {
    let f = g.registry.get(g.perm(m).cd.unwrap()).and_then(|i| i.options).expect("the card has options");
    let o = f(g, m, P0, post).unwrap().into_iter().find(|o| o.label.contains(word)).expect("the option is offered");
    match o.act.clone().unwrap() {
        Action::Ability { src, f, arg } => f(g, src, P0, arg).unwrap(),
        other => panic!("unexpected action {other:?}"),
    }
}

fn creature_tokens(g: &Game) -> Vec<PermId> {
    g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).on_bf && g.perm(m).token && g.is_creature(m)).collect()
}

// ------------------------------------------------------------------ Trostani: populate
/// "{1}{G}{W}, {T}: Populate." is an activated ability at instant speed: a person may use it in their own main phases and
/// on an opponent's turn alike (the AI keeps its mana for an opponent's end step)
#[test]
fn a_person_is_offered_populate_in_their_own_main_phase() {
    let mut g = tbl();
    let t = perm(&mut g, P0, T);
    make_tokens(&mut g, P0, Tokens::new(1, 1)).unwrap();
    seat_person(&mut g, |_| None);
    pool(&mut g, P0, &[('W', 1), ('G', 1), ('C', 1)]);
    for (active, step) in [(P0, Step::Main1), (P0, Step::Main2), (P1, Step::Main1)] {
        g.active = Some(active);
        g.step = step;
        assert!(labels(&mut g, t).iter().any(|l| l.contains("populate")), "populate is offered ({active:?}, {step:?})");
    }
}

/// "Populate: create a token that's a copy of a creature token you control": the person chooses which one
#[test]
fn populate_asks_a_person_which_token_to_copy() {
    let mut g = tbl();
    let t = perm(&mut g, P0, T);
    let soldier = make_tokens(&mut g, P0, Tokens::new(1, 1)).unwrap()[0];
    let beast = make_tokens(&mut g, P0, Tokens { types: vec!["beast"], tgh: Some(3), ..Tokens::new(1, 3) }).unwrap()[0];
    let h = seat_person(&mut g, |r| if r.prompt.contains("Populate") { Some(0) } else { None });
    pool(&mut g, P0, &[('W', 1), ('G', 1), ('C', 1)]);
    assert!(use_option(&mut g, t, Some(false), "populate"));
    assert_eq!(asked(&h, "Populate"), 1, "asked once which token to copy");
    let new: Vec<PermId> = creature_tokens(&g).into_iter().filter(|&m| m != soldier && m != beast).collect();
    assert_eq!(new.len(), 1);
    assert_eq!(g.perm(new[0]).pow, 1, "the person's Soldier, not the 3/3 the AI would copy");
    assert!(g.perm(t).tapped, "{{T}}");
}

#[test]
fn populate_with_one_token_asks_a_person_nothing() {
    let mut g = tbl();
    make_tokens(&mut g, P0, Tokens::new(1, 1)).unwrap();
    let h = seat_person(&mut g, |_| None);
    sim_core::impls::zur::populate(&mut g, P0).unwrap();
    assert_eq!(creature_tokens(&g).len(), 2);
    assert_eq!(asked(&h, "Populate"), 0);
}

/// Rootborn Defenses: "Populate. Creatures you control gain indestructible until end of turn": the same choice
#[test]
fn rootborn_defenses_asks_a_person_which_token_to_copy() {
    let mut g = tbl();
    let soldier = make_tokens(&mut g, P0, Tokens::new(1, 1)).unwrap()[0];
    let beast = make_tokens(&mut g, P0, Tokens { types: vec!["beast"], tgh: Some(3), ..Tokens::new(1, 3) }).unwrap()[0];
    let h = seat_person(&mut g, |r| if r.prompt.contains("Populate") { Some(0) } else { None });
    sim_core::impls::zur::rootborn(&mut g, P0).unwrap();
    assert_eq!(asked(&h, "Populate"), 1);
    let new: Vec<PermId> = creature_tokens(&g).into_iter().filter(|&m| m != soldier && m != beast).collect();
    assert_eq!((new.len(), g.perm(new[0]).pow), (1, 1), "a copy of the first token, the Soldier");
}

// ------------------------------------------------------------------ Sterling Grove
/// "{1}, Sacrifice this enchantment: Search your library for an enchantment card, reveal it, then shuffle and put that
/// card on top." Instant speed; the person chooses the card
#[test]
fn a_person_uses_sterling_grove_at_any_time_and_picks_the_enchantment() {
    for want in ["Cathars' Crusade", "Doubling Season", "Branching Evolution"] {
        let mut g = tbl();
        let grove = perm(&mut g, P0, "Sterling Grove");
        let lib: Vec<_> = ["Cathars' Crusade", "Doubling Season", "Branching Evolution", "Forest"]
            .iter()
            .map(|n| card(&g, n))
            .collect();
        g.player_mut(P0).library = lib;
        let h = seat_person(&mut g, move |r| if r.prompt.contains("Sterling Grove") { with(r, want) } else { None });
        pool(&mut g, P0, &[('C', 1)]);
        g.active = Some(P1); // an opponent's turn: instant speed
        assert!(labels(&mut g, grove).iter().any(|l| l.contains("Sterling Grove")), "offered on an opponent's turn");
        g.active = Some(P0);
        g.step = Step::Main1;
        assert!(use_option(&mut g, grove, Some(false), "Sterling Grove"), "offered in the first main phase");
        assert_eq!(asked(&h, "Sterling Grove"), 1);
        assert!(!g.perm(grove).on_bf, "sacrificed");
        let top = *g.player(P0).library.last().unwrap();
        assert_eq!(&*g.db.get(top).name, want, "the person's choice is on top");
    }
}

// ------------------------------------------------------------------ Innkeeper's Talent
/// "{G}: Level 2" ... "{3}{G}: Level 3": a class's level-up is a sorcery-speed activation a person may use with nothing
/// else on the table (the AI waits for a permanent with counters)
#[test]
fn a_person_may_level_up_innkeepers_talent_with_no_counters_in_play() {
    let mut g = tbl();
    let talent = perm(&mut g, P0, "Innkeeper's Talent");
    seat_person(&mut g, |_| None);
    pool(&mut g, P0, &[('G', 1)]);
    assert!(labels(&mut g, talent).iter().any(|l| l.contains("level 2")), "level 2 is offered");
    assert!(use_option(&mut g, talent, Some(false), "level 2"));
    pool(&mut g, P0, &[('G', 1), ('C', 3)]);
    assert!(labels(&mut g, talent).iter().any(|l| l.contains("level 3")), "then level 3 ({{3}}{{G}})");
}

/// "At the beginning of combat on your turn, put a +1/+1 counter on target creature you control": the person chooses
#[test]
fn innkeepers_combat_trigger_asks_a_person_for_the_creature() {
    let mut g = tbl();
    perm(&mut g, P0, "Innkeeper's Talent");
    let a = token(&mut g, P0, 3); // the AI's pick: the most power
    let b = token(&mut g, P0, 1);
    let h = seat_person(&mut g, |r| if r.prompt.contains("Innkeeper") { Some(1) } else { None });
    fire_trigger(&mut g, Event::EachCombat, Call::Player { p: P0 }).unwrap();
    assert_eq!(asked(&h, "Innkeeper"), 1);
    assert_eq!((g.perm(a).plus, g.perm(b).plus), (0, 1), "the counter went on the creature the person picked");
}

// ------------------------------------------------------------------ Emiel the Blessed
/// "Whenever another creature you control enters, you may pay {G/W}. If you do, put a +1/+1 counter on it": a person is
/// asked, and a no keeps the mana
#[test]
fn emiel_asks_a_person_whether_to_pay() {
    for pay in [false, true] {
        let mut g = tbl();
        perm(&mut g, P0, "Emiel the Blessed");
        let h = seat_person(&mut g, move |r| {
            if r.prompt.contains("Emiel") { with(r, if pay { "yes" } else { "no" }) } else { None }
        });
        pool(&mut g, P0, &[('W', 1)]);
        let tok = make_tokens(&mut g, P0, Tokens::new(1, 1)).unwrap()[0];
        assert_eq!(asked(&h, "Emiel the Blessed: pay {G/W}"), 1, "asked once");
        assert_eq!(g.perm(tok).plus, pay as i32, "a counter only on a yes");
        assert_eq!(g.player(P0).pool.total(), if pay { 0 } else { 1 }, "the mana is spent only on a yes");
    }
}

// ------------------------------------------------------------------ Treebeard, Gracious Host
/// "Whenever you gain life, put that many +1/+1 counters on target Halfling or Treefolk": with two, the person picks
#[test]
fn treebeard_asks_a_person_which_halfling_or_treefolk_gets_the_counters() {
    let mut g = tbl();
    let treebeard = perm(&mut g, P0, "Treebeard, Gracious Host");
    let ent = token(&mut g, P0, 2);
    g.perm_mut(ent).ttypes.push("treefolk");
    let h = seat_person(&mut g, |r| if r.prompt.contains("Treebeard") { Some(1) } else { None });
    life::gain(&mut g, P0, 3).unwrap();
    assert_eq!(asked(&h, "Treebeard"), 1);
    assert_eq!((g.perm(treebeard).plus, g.perm(ent).plus), (0, 3), "the Treefolk the person picked, not Treebeard");
}

#[test]
fn treebeard_alone_asks_a_person_nothing_and_takes_the_counters_itself() {
    let mut g = tbl();
    let treebeard = perm(&mut g, P0, "Treebeard, Gracious Host");
    let h = seat_person(&mut g, |_| None);
    life::gain(&mut g, P0, 3).unwrap();
    assert_eq!(asked(&h, "Treebeard"), 0, "one legal target: no choice");
    assert_eq!(g.perm(treebeard).plus, 3);
}

// ------------------------------------------------------------------ Wildwood Scourge on the stack, as a person sees it
/// the Scourge's trigger shows as "Wildwood Scourge: a +1/+1 counter" (the stack adds the source's name itself)
#[test]
fn the_scourge_trigger_is_named_once_on_the_stack() {
    let mut g = tbl();
    perm(&mut g, P1, "Azorius Guildmage"); // someone who can respond: the trigger gets a priority window
    let scourge = {
        let c = take(&mut g, P0, "Wildwood Scourge");
        sim_core::engine::zones::enter(
            &mut g,
            P0,
            c,
            sim_core::engine::zones::Enter { x: 2, was_cast: true, sick: false, ..Default::default() },
        )
        .unwrap()
    };
    let bear = token(&mut g, P0, 2);
    let h = Arc::new(ScriptHuman::from_fn(|_| Answer::Action(json!({"do": "pass"}))));
    h.set_autopass("all");
    g.humans.add(P0, h.clone());
    sim_core::engine::repl::put_counters(&mut g, P0, bear, 1);
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert_eq!(g.perm(scourge).plus, 3);
    let seen: Vec<String> = h.asked_kind("priority").into_iter().map(|r| r.prompt).collect();
    assert!(seen.iter().any(|p| p.contains("Wildwood Scourge: a +1/+1 counter")), "{seen:?}");
    assert!(seen.iter().all(|p| !p.contains("Wildwood Scourge: Wildwood Scourge")), "{seen:?}");
}
