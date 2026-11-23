//! Practice mode: the person's own choices for Gifts Ungiven (the target opponent) and Mindbreak Trap (any number of
//! target spells), checked against the Oracle text and against the AI's play of the same card.
//!
//! Gifts Ungiven: "Search your library for up to four cards with different names and reveal them. Target opponent
//! chooses two of those cards. Put the chosen cards into your graveyard and the rest into your hand. Then shuffle."
//! (a pile of four: two in the hand, two in the graveyard; tests/pile_tutor_split.rs covers smaller piles). With one
//! opponent there is nothing to choose.

mod parity_common;
mod play_common;
use play_common::*;
use sim_core::engine::tutors;
use std::sync::{Arc, Mutex};

const P3: PlayerId = PlayerId(3);

/// everything the person was asked, as "kind: prompt [choices]"
type Asked = Arc<Mutex<Vec<(String, String, Vec<String>)>>>;

/// seat 0 as a person who answers the target-opponent question with the choice starting with `who` (None: the
/// first) and everything else with the first choice
fn gifts_person(g: &mut Game, who: Option<&'static str>) -> Asked {
    let log: Asked = Arc::new(Mutex::new(vec![]));
    let l2 = log.clone();
    g.humans.add(
        P0,
        Arc::new(ScriptHuman::from_fn(move |r: &Request| {
            l2.lock().unwrap().push((r.kind.to_string(), r.prompt.clone(), r.choices.clone()));
            if r.prompt.contains("target opponent") {
                let at = who.and_then(|w| r.choices.iter().position(|c| c.starts_with(w)));
                return Answer::Pick(at.unwrap_or(0));
            }
            Answer::Pick(0)
        })),
    );
    log
}

fn target_questions(a: &Asked) -> Vec<Vec<String>> {
    a.lock().unwrap().iter().filter(|x| x.1.contains("target opponent")).map(|x| x.2.clone()).collect()
}

/// the line of the log that names who split the pile
fn bins_line(g: &Game) -> String {
    g.log.as_ref().unwrap().iter().find(|l| l.contains(" bins ")).cloned().expect("a 'bins' line in the log")
}

fn four_seat_game() -> Game {
    let mut g = table(&["seph", "cloud", "kefka", "thanos"]);
    g.log = Some(vec![]);
    g
}

/// Gifts Ungiven's "target opponent": every living opponent of the caster, and nobody else
#[test]
fn gifts_asks_the_person_which_opponent_splits_the_pile() {
    let mut g = four_seat_game();
    let asked = gifts_person(&mut g, None);
    tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
    // the Oracle text lists "target opponent": the three other seats, each shown as a player with their life
    let want: Vec<String> = [P1, P2, P3].iter().map(|&q| format!("{} (player, 40 life)", g.player(q).name)).collect();
    assert_eq!(target_questions(&asked), vec![want], "one question, with exactly the three opponents");
}

#[test]
fn gifts_the_opponent_the_person_names_is_the_one_who_splits() {
    for who in [P1, P2, P3] {
        let mut g = four_seat_game();
        let name = g.player(who).name.to_string();
        let leaked: &'static str = Box::leak(name.clone().into_boxed_str());
        let _ = gifts_person(&mut g, Some(leaked));
        tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
        assert!(bins_line(&g).contains(&format!("; {name} bins ")), "{name} splits: {}", bins_line(&g));
    }
}

#[test]
fn gifts_a_dead_opponent_is_not_offered() {
    let mut g = four_seat_game();
    g.player_mut(P2).alive = false;
    let asked = gifts_person(&mut g, None);
    tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
    let want: Vec<String> = [P1, P3].iter().map(|&q| format!("{} (player, 40 life)", g.player(q).name)).collect();
    assert_eq!(target_questions(&asked), vec![want], "two living opponents only");
}

#[test]
fn gifts_an_opponent_who_cannot_be_targeted_is_not_offered() {
    // The One Ring: protection from everything, so no spell can target its controller
    let mut g = four_seat_game();
    g.player_mut(P3).ring_prot = true;
    let asked = gifts_person(&mut g, None);
    tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
    let want: Vec<String> = [P1, P2].iter().map(|&q| format!("{} (player, 40 life)", g.player(q).name)).collect();
    assert_eq!(target_questions(&asked), vec![want], "the protected opponent is not a legal target");
}

#[test]
fn gifts_with_one_opponent_asks_nothing_about_the_target() {
    let mut g = table(&["seph", "cloud"]);
    g.log = Some(vec![]);
    let asked = gifts_person(&mut g, None);
    tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
    assert!(target_questions(&asked).is_empty(), "forced: {:?}", asked.lock().unwrap());
    assert!(bins_line(&g).contains(&format!("; {} bins ", g.player(P1).name)));
}

#[test]
fn gifts_with_one_living_opponent_left_asks_nothing_about_the_target() {
    let mut g = four_seat_game();
    g.player_mut(P1).alive = false;
    g.player_mut(P3).alive = false;
    let asked = gifts_person(&mut g, None);
    tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
    assert!(target_questions(&asked).is_empty(), "forced: {:?}", asked.lock().unwrap());
    assert!(bins_line(&g).contains(&format!("; {} bins ", g.player(P2).name)));
}

#[test]
fn gifts_with_two_living_opponents_one_untargetable_asks_nothing_about_the_target() {
    // The One Ring on one of two living opponents: the other is the only legal target, so nothing is asked
    let mut g = table(&["seph", "cloud", "kefka"]);
    g.log = Some(vec![]);
    g.player_mut(P2).ring_prot = true;
    let asked = gifts_person(&mut g, None);
    tutors::pile_tutor(&mut g, P0, 4, 2).unwrap();
    assert!(target_questions(&asked).is_empty(), "forced: {:?}", asked.lock().unwrap());
    assert!(bins_line(&g).contains(&format!("; {} bins ", g.player(P1).name)), "{}", bins_line(&g));
}

// ================================================================== Mindbreak Trap
// "Exile any number of target spells." A person chooses which spells on the stack, any of them, their own included.

use sim_core::ids::CardId;
use sim_core::state::{Ctx, StackItem, StackKind};

/// a spell of p's on the stack (cast: out of the hand)
fn on_stack(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = hand(g, p, &[name])[0];
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
        name: name.to_string(),
        passed: vec![],
        countered: false,
        countered_by: None,
    });
    c
}

/// seat 0 holds Mindbreak Trap and four blue mana; the stack, bottom to top: P0's Opt, P2's Brainstorm, P1's Lightning
/// Bolt
fn trap_table() -> Game {
    let mut g = table(&["thanos", "seph", "kefka"]);
    g.active = Some(P1);
    hand(&mut g, P0, &["Mindbreak Trap"]);
    pool(&mut g, P0, &[('U', 4)]);
    on_stack(&mut g, P0, "Opt");
    on_stack(&mut g, P2, "Brainstorm");
    on_stack(&mut g, P1, "Lightning Bolt");
    g
}

/// seat 0 as a person who answers the Trap's questions with these spells, in order, then says they are done
fn trap_person(g: &mut Game, names: &[&'static str]) -> Asked {
    let log: Asked = Arc::new(Mutex::new(vec![]));
    let (l2, mut left) = (log.clone(), names.to_vec().into_iter());
    g.humans.add(
        P0,
        Arc::new(ScriptHuman::from_fn(move |r: &Request| {
            l2.lock().unwrap().push((r.kind.to_string(), r.prompt.clone(), r.choices.clone()));
            match left.next() {
                Some(n) => Answer::Pick(
                    r.choices.iter().position(|c| c.starts_with(n)).unwrap_or_else(|| panic!("{n} in {:?}", r.choices)),
                ),
                None => Answer::Decline,
            }
        })),
    );
    log
}

fn trap_questions(a: &Asked) -> Vec<Vec<String>> {
    a.lock().unwrap().iter().filter(|x| x.1.starts_with("Mindbreak Trap")).map(|x| x.2.clone()).collect()
}

/// (countered, exiled by the Trap's own rule) of the spell of this name on the stack
fn spell_state(g: &Game, name: &str) -> (bool, bool) {
    let it = g.stack.iter().find(|x| x.name == name).unwrap_or_else(|| panic!("{name} is on the stack"));
    (it.countered, it.ctx.exile_after)
}

fn cast_trap(g: &mut Game) -> Option<String> {
    let trap = card(g, "Mindbreak Trap");
    sim_core::human::human::cast_counterspell(g, P0, trap).unwrap()
}

#[test]
fn trap_the_person_is_offered_every_spell_on_the_stack_and_then_the_rest() {
    let mut g = trap_table();
    let asked = trap_person(&mut g, &["Brainstorm", "Opt"]);
    let (n0, n1, n2) = (g.player(P0).name.to_string(), g.player(P1).name.to_string(), g.player(P2).name.to_string());
    assert_eq!(cast_trap(&mut g), None);
    // Oracle: any number of target spells: all three are legal targets (their own too), the top one first. After
    // each pick, the spells not yet chosen, and a way to stop
    let first = vec![
        format!("Lightning Bolt ({n1})"),
        format!("Brainstorm ({n2})"),
        format!("Opt ({n0})"),
        "cancel".to_string(),
    ];
    let second = vec![format!("Lightning Bolt ({n1})"), format!("Opt ({n0})"), "done".to_string()];
    let third = vec![format!("Lightning Bolt ({n1})"), "done".to_string()];
    assert_eq!(trap_questions(&asked), vec![first, second, third]);
}

#[test]
fn trap_the_person_exiles_only_the_spells_they_chose() {
    let mut g = trap_table();
    let _ = trap_person(&mut g, &["Brainstorm"]); // then done
    assert_eq!(cast_trap(&mut g), None);
    assert!(spell_state(&g, "Brainstorm").0, "the chosen spell");
    assert!(!spell_state(&g, "Lightning Bolt").0, "an opposing spell they left alone");
    assert!(!spell_state(&g, "Opt").0, "their own spell");
}

#[test]
fn trap_the_person_can_exile_their_own_spell_and_one_other() {
    let mut g = trap_table();
    let _ = trap_person(&mut g, &["Opt", "Lightning Bolt"]);
    assert_eq!(cast_trap(&mut g), None);
    assert_eq!(spell_state(&g, "Opt"), (true, false), "their own spell is the first target");
    assert_eq!(spell_state(&g, "Lightning Bolt"), (true, true), "exiled");
    assert!(!spell_state(&g, "Brainstorm").0, "not chosen");
}

#[test]
fn trap_the_person_can_exile_everything_including_their_own() {
    let mut g = trap_table();
    let asked = trap_person(&mut g, &["Lightning Bolt", "Opt", "Brainstorm"]);
    assert_eq!(cast_trap(&mut g), None);
    for s in ["Lightning Bolt", "Brainstorm", "Opt"] {
        assert!(spell_state(&g, s).0, "{s}");
    }
    assert_eq!(trap_questions(&asked).len(), 3, "nothing left to choose after the third: no fourth question");
}

#[test]
fn trap_cancelling_the_first_question_casts_nothing() {
    let mut g = trap_table();
    let _ = trap_person(&mut g, &[]);
    assert_eq!(cast_trap(&mut g).as_deref(), Some("Cancelled."));
    assert!(g.stack.iter().all(|x| !x.countered), "nothing exiled");
    assert!(g.player(P0).hand.iter().any(|&c| &*g.db.get(c).name == "Mindbreak Trap"), "still in hand");
    assert_eq!(pool_total(&g, P0), 4, "nothing paid");
}

#[test]
fn trap_with_one_opposing_spell_has_nothing_to_choose() {
    let mut g = table(&["thanos", "seph"]);
    g.active = Some(P1);
    hand(&mut g, P0, &["Mindbreak Trap"]);
    pool(&mut g, P0, &[('U', 4)]);
    on_stack(&mut g, P1, "Lightning Bolt");
    let asked = trap_person(&mut g, &[]);
    assert_eq!(cast_trap(&mut g), None);
    assert!(trap_questions(&asked).is_empty(), "forced: {:?}", asked.lock().unwrap());
    assert!(spell_state(&g, "Lightning Bolt").0);
}

#[test]
fn trap_the_ai_still_exiles_every_spell_that_is_not_its_own() {
    // the AI at seat 0 (no person): the top spell is the target, and every other opposing spell goes with it
    let mut g = trap_table();
    let trap = card(&g, "Mindbreak Trap");
    let top = g.stack.last().unwrap().id;
    // free mode (an opponent cast three spells this turn): the AI has no lands to pay with
    let (st, cs) = (g.turn_stamp(), ["Lightning Bolt", "Brainstorm", "Opt"].map(|n| card(&g, n)).to_vec());
    g.player_mut(P1).turn_casts = Some((st, cs));
    assert!(sim_core::engine::stack::cast_counter_spell(&mut g, P0, trap, top).unwrap());
    assert!(spell_state(&g, "Lightning Bolt").0 && spell_state(&g, "Brainstorm").0);
    assert!(!spell_state(&g, "Opt").0, "its own spell is not exiled");
}

// ------------------------------------------------------------------ only spells are targets
/// an ability or a trigger on the stack, from a card of p's hand (the card is its source)
fn item_on_stack(g: &mut Game, p: PlayerId, name: &str, kind: StackKind) {
    let c = hand(g, p, &[name])[0];
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
        kind,
        name: format!("{name} ability"),
        passed: vec![],
        countered: false,
        countered_by: None,
    });
}

/// the stack, bottom to top: P0's Opt, P0's ability, P2's Brainstorm, P1's trigger, P1's Lightning Bolt, already
/// countered. Oracle: "target spells": only Brainstorm and Opt are legal
fn trap_table_with_non_spells() -> Game {
    let mut g = table(&["thanos", "seph", "kefka"]);
    g.active = Some(P1);
    hand(&mut g, P0, &["Mindbreak Trap"]);
    pool(&mut g, P0, &[('U', 4)]);
    on_stack(&mut g, P0, "Opt");
    item_on_stack(&mut g, P0, "Sol Ring", StackKind::Ability);
    on_stack(&mut g, P2, "Brainstorm");
    item_on_stack(&mut g, P1, "Mana Crypt", StackKind::Trigger);
    on_stack(&mut g, P1, "Lightning Bolt");
    g.stack.last_mut().unwrap().countered = true;
    g
}

#[test]
fn trap_the_person_is_offered_only_spells_that_are_not_countered() {
    let mut g = trap_table_with_non_spells();
    let asked = trap_person(&mut g, &["Brainstorm", "Opt"]);
    let (n0, n2) = (g.player(P0).name.to_string(), g.player(P2).name.to_string());
    assert_eq!(cast_trap(&mut g), None);
    let first = vec![format!("Brainstorm ({n2})"), format!("Opt ({n0})"), "cancel".to_string()];
    let second = vec![format!("Opt ({n0})"), "done".to_string()];
    assert_eq!(trap_questions(&asked), vec![first, second]);
    assert_eq!(spell_state(&g, "Brainstorm").0, true);
    assert_eq!(spell_state(&g, "Opt"), (true, true), "exiled");
}

#[test]
fn trap_the_person_with_one_legal_spell_among_other_items_has_nothing_to_choose() {
    // only P1's Lightning Bolt is a spell that can be targeted: the other items are an ability, a trigger and a
    // countered spell
    let mut g = table(&["thanos", "seph"]);
    g.active = Some(P1);
    hand(&mut g, P0, &["Mindbreak Trap"]);
    pool(&mut g, P0, &[('U', 4)]);
    item_on_stack(&mut g, P0, "Sol Ring", StackKind::Ability);
    on_stack(&mut g, P1, "Brainstorm");
    g.stack.last_mut().unwrap().countered = true;
    item_on_stack(&mut g, P1, "Mana Crypt", StackKind::Trigger);
    on_stack(&mut g, P1, "Lightning Bolt");
    let asked = trap_person(&mut g, &[]);
    assert_eq!(cast_trap(&mut g), None);
    assert!(trap_questions(&asked).is_empty(), "forced: {:?}", asked.lock().unwrap());
    assert!(spell_state(&g, "Lightning Bolt").0);
}

// ------------------------------------------------------------------ the first target is gone by resolution
/// CR 608.2b: a spell with several targets still resolves if any of them is legal. The person's Trap aims at Lightning
/// Bolt first, then Brainstorm; in response its controller casts their own Trap on the Bolt, so the Bolt is exiled
/// before the first Trap resolves, and the first Trap still exiles Brainstorm
#[test]
fn trap_the_chosen_further_targets_are_exiled_when_the_first_target_left_in_response() {
    let mut g = table(&["thanos", "seph", "kefka"]);
    g.active = Some(P1);
    hand(&mut g, P0, &["Mindbreak Trap"]);
    pool(&mut g, P0, &[('U', 4)]);
    hand(&mut g, P1, &["Mindbreak Trap"]);
    pool(&mut g, P1, &[('U', 4)]);
    on_stack(&mut g, P0, "Opt");
    on_stack(&mut g, P2, "Brainstorm");
    on_stack(&mut g, P1, "Lightning Bolt");
    let _ = trap_person_only_trap(&mut g, P0, &["Lightning Bolt", "Brainstorm"]);
    // P1 responds once: their own Trap, aimed at their Lightning Bolt only
    let mut cast = false;
    let mut script = vec!["Lightning Bolt"].into_iter();
    g.humans.add(
        P1,
        Arc::new(ScriptHuman::from_fn(move |r: &Request| {
            if r.prompt.starts_with("Mindbreak Trap") {
                return match script.next() {
                    Some(n) => Answer::Pick(r.choices.iter().position(|c| c.starts_with(n)).unwrap()),
                    None => Answer::Decline,
                };
            }
            if !cast && r.kind == "priority" {
                cast = true;
                return Answer::Action(json!({"do": "cast", "card": 0}));
            }
            Answer::Action(json!({"do": "pass"}))
        })),
    );
    assert_eq!(cast_trap(&mut g), None);
    assert!(spell_state(&g, "Lightning Bolt").0, "the first target was exiled in response");
    assert_eq!(spell_state(&g, "Brainstorm"), (true, true), "a chosen further target is still exiled");
    assert!(!spell_state(&g, "Opt").0, "not chosen");
}

/// seat p as a person who answers only the Trap's own questions, with these spells in order and then done
fn trap_person_only_trap(g: &mut Game, p: PlayerId, names: &[&'static str]) -> Asked {
    let log: Asked = Arc::new(Mutex::new(vec![]));
    let (l2, mut left) = (log.clone(), names.to_vec().into_iter());
    g.humans.add(
        p,
        Arc::new(ScriptHuman::from_fn(move |r: &Request| {
            l2.lock().unwrap().push((r.kind.to_string(), r.prompt.clone(), r.choices.clone()));
            if !r.prompt.starts_with("Mindbreak Trap") {
                return Answer::Action(json!({"do": "pass"}));
            }
            match left.next() {
                Some(n) => Answer::Pick(
                    r.choices.iter().position(|c| c.starts_with(n)).unwrap_or_else(|| panic!("{n} in {:?}", r.choices)),
                ),
                None => Answer::Decline,
            }
        })),
    );
    log
}

#[test]
fn trap_the_first_question_says_exile_not_counter() {
    let mut g = trap_table();
    let asked = trap_person(&mut g, &["Brainstorm"]);
    assert_eq!(cast_trap(&mut g), None);
    let prompts: Vec<String> =
        asked.lock().unwrap().iter().map(|x| x.1.clone()).filter(|p| p.starts_with("Mindbreak Trap")).collect();
    assert_eq!(prompts[0], "Mindbreak Trap: exile which spell?");
    assert!(prompts[1].starts_with("Mindbreak Trap: exile another spell?"), "{prompts:?}");
}
