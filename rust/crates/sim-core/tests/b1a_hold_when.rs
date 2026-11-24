//! B1a (Stage 4): "a held card is never a proactive cast". A plan's `[[hold]]` row takes an optional `when`; a card
//! with a DSL and a hold row whose `when` holds is not offered by `brain::card_utility` (so not by `main_options`), and
//! is still cast in response (`pool::protect`). No plan file has such a row today: every deck plays as before.
//!
//! The plan files are compiled in, so a test cannot put a row in one: `plan_data::set_hold_plan_for_test` makes
//! `held_back` read a plan this test loads from text (`hold_plan`).

use sim_core::ai::brain::{Situation, card_utility, main_options};
use sim_core::ai::{plan_data, pool};
use sim_core::ids::{CardId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const FLAWLESS: &str = "Flawless Maneuver";
const YOSHI: &str = "Yoshimaru, Ever Faithful";

/// puts the test plan in for this thread, and takes it out again
struct HoldPlan;

impl HoldPlan {
    fn with(text: &str) -> HoldPlan {
        let l = plan_data::load("yoshimaru", text).unwrap_or_else(|e| panic!("{e}"));
        plan_data::set_hold_plan_for_test(Some(Box::leak(Box::new(l))));
        HoldPlan
    }
}

impl Drop for HoldPlan {
    fn drop(&mut self) {
        plan_data::set_hold_plan_for_test(None);
    }
}

/// a plan whose only content is a hold row for Flawless Maneuver, with this `when` (a TOML array body)
fn hold_text(when: &str) -> String {
    let w = if when.is_empty() { String::new() } else { format!("when = [{when}]\n") };
    format!("deck = \"yoshimaru\"\n[[hold]]\ncard = \"{FLAWLESS}\"\nprotects = [\"commander\"]\n{w}")
}

/// Yoshimaru's seat in its first main phase with an empty stack: Yoshimaru out, three Plains up, Flawless Maneuver
/// the only card in hand
fn position() -> (Game, CardId) {
    let mut g = table(&["yoshimaru", "seph"]);
    g.active = Some(P0);
    for q in &mut g.players {
        q.hand.clear();
    }
    lands(&mut g, P0, "Plains", 3, false);
    let y = perm(&mut g, P0, YOSHI);
    g.perm_mut(y).is_cmd = true; // the commander: worth protecting (`pool::worth_protecting`)
    let c = hand(&mut g, P0, &[FLAWLESS])[0];
    (g, c)
}

fn offered(g: &mut Game) -> bool {
    main_options(g, P0, false).unwrap().iter().any(|o| o.label == FLAWLESS)
}

fn utility(g: &Game, c: CardId) -> Option<f64> {
    card_utility(g, P0, &Situation::new(g, P0), c)
}

// ------------------------------------------------------------------ 1: a held card is not a proactive cast
#[test]
fn a_held_card_is_not_offered_in_a_main_phase_and_the_same_card_without_the_row_is() {
    let (mut g, c) = position();
    assert!(g.db.get(c).has_dsl(), "Flawless Maneuver has a DSL: the row matters for it");
    // the control: no row (the position is the same)
    assert!(utility(&g, c).is_some(), "without a hold row the DSL value offers it");
    assert!(offered(&mut g), "without a hold row it is offered in the main phase");
    // the row, `when` empty (always)
    let _plan = HoldPlan::with(&hold_text(""));
    assert!(plan_data::held_back(&g, P0, c));
    assert_eq!(utility(&g, c), None, "a held card has no utility");
    assert!(!offered(&mut g), "a held card is not offered in the main phase");
}

#[test]
fn a_hold_row_for_another_card_changes_nothing() {
    let (mut g, c) = position();
    let before = utility(&g, c);
    let _plan =
        HoldPlan::with("deck = \"yoshimaru\"\n[[hold]]\ncard = \"Teferi's Protection\"\nprotects = [\"commander\"]\n");
    assert!(!plan_data::held_back(&g, P0, c));
    assert_eq!(utility(&g, c), before);
    assert!(offered(&mut g));
}

// ------------------------------------------------------------------ 2: still cast in response
#[test]
fn a_held_card_is_still_cast_in_response() {
    let _plan = HoldPlan::with(&hold_text(""));
    let (mut g, c) = position();
    let y = *g.player(P0).perms.iter().find(|&&m| &*g.perm(m).name == YOSHI).unwrap();
    assert!(plan_data::held_back(&g, P0, c) && utility(&g, c).is_none());
    // a destroy spell is aimed at Yoshimaru: the protect path casts Flawless Maneuver from hand
    let saved = pool::protect(&mut g, P0, y, "destroy", Some(P1), None).unwrap();
    assert!(saved, "the held card still protects");
    assert!(!g.player(P0).hand.contains(&c), "Flawless Maneuver was cast");
}

#[test]
fn the_response_is_the_same_with_and_without_the_row() {
    let run = || {
        let (mut g, c) = position();
        let y = *g.player(P0).perms.iter().find(|&&m| &*g.perm(m).name == YOSHI).unwrap();
        let saved = pool::protect(&mut g, P0, y, "destroy", Some(P1), None).unwrap();
        let ind = sim_core::engine::values::indestructible(&g, y);
        (saved, g.player(P0).hand.contains(&c), ind, g.player(P0).lands.iter().filter(|&&l| g.perm(l).tapped).count())
    };
    let without = run();
    assert!(without.0 && !without.1, "the control: with no row Flawless Maneuver saves Yoshimaru: {without:?}");
    let _plan = HoldPlan::with(&hold_text(""));
    assert_eq!(run(), without);
}

// ------------------------------------------------------------------ 3: `when`
#[test]
fn when_decides_whether_the_card_is_held() {
    let (mut g, c) = position();
    let plain = utility(&g, c);
    assert!(plain.is_some());
    {
        // one creature out: false
        let _plan = HoldPlan::with(&hold_text("\"creatures>=6\""));
        assert!(!plan_data::held_back(&g, P0, c));
        assert_eq!(utility(&g, c), plain, "a false `when`: offered exactly as before");
        assert!(offered(&mut g));
    }
    {
        let _plan = HoldPlan::with(&hold_text("\"creatures>=1\""));
        assert!(plan_data::held_back(&g, P0, c));
        assert_eq!(utility(&g, c), None, "a true `when`: held");
        assert!(!offered(&mut g));
    }
    {
        // alternatives: any string that holds (the shape of every other `when`)
        let _plan = HoldPlan::with(&hold_text("\"creatures>=6\", \"creatures>=1 & lands>=3\""));
        assert!(plan_data::held_back(&g, P0, c));
    }
    // the condition is read as the position changes: the creature leaves, the hold ends
    let _plan = HoldPlan::with(&hold_text("\"creatures>=1\""));
    assert!(plan_data::held_back(&g, P0, c));
    let y = *g.player(P0).perms.iter().find(|&&m| &*g.perm(m).name == YOSHI).unwrap();
    g.player_mut(P0).perms.retain(|&m| m != y);
    assert!(!plan_data::held_back(&g, P0, c));
}

// ------------------------------------------------------------------ 4: invalid rows
#[test]
fn bad_hold_rows_are_refused_with_their_reason() {
    let bad = |t: &str| plan_data::load("x", t).err().unwrap_or_else(|| panic!("accepted: {t}"));
    let row = |extra: &str| format!("deck = \"x\"\n[[hold]]\ncard = \"{FLAWLESS}\"\n{extra}");
    assert!(plan_data::load("x", &row("when = [\"creatures>=1\"]")).is_ok());
    let e = bad(&row("when = [\"moon > 3\"]"));
    assert!(e.contains("hold rule 1") && e.contains("unknown"), "{e}");
    assert!(bad(&row("when = [\"(creatures>=1\"]")).contains("hold rule 1"));
    assert!(bad(&row("whenn = [\"creatures>=1\"]")).contains("unknown field"));
    assert!(bad(&row("release = true")).contains("unknown field"));
    assert!(bad(&row("when = \"creatures>=1\"")).contains("x.toml"), "`when` must be a list");
}

#[test]
fn a_hold_row_must_name_a_protector_or_a_counter() {
    // `load` has no card database; `hold_problems` checks the rows against it
    let db = db();
    let check = |card: &str| {
        let l = plan_data::load("x", &format!("deck = \"x\"\n[[hold]]\ncard = \"{card}\"\n")).unwrap();
        plan_data::hold_problems(&l, &db)
    };
    for ok in [FLAWLESS, "Teferi's Protection", "Eerie Interlude", "Counterspell", "Mana Drain"] {
        // (tags and the protector list: Flawless Maneuver and Eerie Interlude carry no tag, they are in the list)
        let p = check(ok);
        assert!(p.is_empty(), "{ok}: {p:?}");
    }
    let p = check("Sol Ring");
    assert!(p.len() == 1 && p[0].contains("not a protector or a counter"), "{p:?}");
    let p = check("No Such Card");
    assert!(p.len() == 1 && p[0].contains("no such card"), "{p:?}");
}

// ------------------------------------------------------------------ 5: no existing plan is affected
/// every plan file's hold rows today (plan, card): the ones this test pins
const ROWS: [(&str, &str); 5] = [
    ("aurelia-boros-extra-combats", "Teferi's Protection"),
    ("kyodai", "Teferi's Protection"),
    ("urdragon", "Teferi's Protection"),
    ("yoshimaru", "Eerie Interlude"),
    ("yoshimaru", "Teferi's Protection"),
];

#[test]
fn no_plan_file_has_a_row_the_new_check_can_touch() {
    let db = db();
    let mut rows: Vec<(&str, String)> = vec![];
    for (key, l) in plan_data::all() {
        assert_eq!(l.plan.hold.len(), l.hold_when.len(), "{key}");
        for h in &l.plan.hold {
            assert!(h.when.is_empty(), "{key}: {:?} has a `when`", h.card);
            let d = db.by_name(&h.card).unwrap_or_else(|| panic!("{key}: {:?}", h.card));
            // the check in card_utility is `has_dsl() && held_back`: a card without a DSL never reaches `held_back`
            assert!(!d.has_dsl(), "{key}: {:?} has a DSL, so its row would now hold it", h.card);
            rows.push((key, h.card.clone()));
        }
        assert_eq!(plan_data::hold_problems(l, &db), Vec::<String>::new(), "{key}");
    }
    rows.sort();
    let want: Vec<(&str, String)> = ROWS.iter().map(|r| (r.0, r.1.to_string())).collect();
    assert_eq!(rows, want, "a plan file gained or lost a hold row: check it against the new check, then update ROWS");
}

#[test]
fn every_card_of_every_plan_decks_lists_passes_the_new_check_untouched() {
    let mut checked = 0;
    for (key, l) in plan_data::all() {
        let Some(deck) = decks().iter().find(|d| d.key == *key) else { continue };
        let g = table(&[key, "seph"]);
        let names = deck.cards.iter().chain(std::iter::once(&deck.commander)).chain(deck.commander2.iter());
        for n in names {
            let Some(c) = g.db.id(n) else { continue };
            // the new branch of card_utility is never taken: no card of the list has a DSL and a row
            let d = g.db.get(c);
            assert!(!(d.has_dsl() && plan_data::held_back(&g, P0, c)), "{key}: {n}");
            // and a row's card is the one the plan names
            let named = l.plan.hold.iter().any(|h| h.card == *n);
            assert_eq!(plan_data::held_back(&g, P0, c), named && plan_data::active(&g, P0).is_some(), "{key}: {n}");
            checked += 1;
        }
    }
    assert!(checked > 1000, "{checked} cards checked");
}

#[test]
fn the_real_hold_rows_leave_card_utility_alone() {
    // yoshimaru's own rows (Teferi's Protection, Eerie Interlude): the utility with the row equals the utility with the
    // hook plan that has no row at all, for every card in its hand, in the same position
    let (g, _) = {
        let (mut g, c) = position();
        hand(&mut g, P0, &["Teferi's Protection", "Eerie Interlude", "Swords to Plowshares", "Sol Ring"]);
        (g, c)
    };
    let hand_now = g.player(P0).hand.clone();
    let with_rows: Vec<Option<f64>> = hand_now.iter().map(|&c| utility(&g, c)).collect();
    let _plan = HoldPlan::with("deck = \"yoshimaru\"\n");
    let without: Vec<Option<f64>> = hand_now.iter().map(|&c| utility(&g, c)).collect();
    assert_eq!(with_rows, without);
    for (&c, u) in hand_now.iter().zip(&with_rows) {
        if matches!(&*g.db.get(c).name, "Teferi's Protection" | "Eerie Interlude") {
            assert_eq!(*u, None, "{}: a held-for-response card has no proactive value today either", g.db.get(c).name);
        }
    }
}
