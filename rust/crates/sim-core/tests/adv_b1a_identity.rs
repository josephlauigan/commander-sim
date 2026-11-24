// written by an independent reviewer
//! Adversarial tests for B1a, attacks 1, 2 and 7: byte identity for every real deck and plan, the thread-local test
//! hook, and what wiring `hold_problems` into the plan-file guard would do.

use sim_core::ai::{plan_data, plans};
use sim_core::ids::{CardId, PlayerId};
use sim_core::testkit::*;
use std::collections::BTreeSet;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const FLAWLESS: &str = "Flawless Maneuver";

struct HoldPlan;
impl HoldPlan {
    fn with(key: &str, text: &str) -> HoldPlan {
        let l = plan_data::load(key, text).unwrap_or_else(|e| panic!("{e}"));
        plan_data::set_hold_plan_for_test(Some(Box::leak(Box::new(l))));
        HoldPlan
    }
}
impl Drop for HoldPlan {
    fn drop(&mut self) {
        plan_data::set_hold_plan_for_test(None);
    }
}

fn hold_text(key: &str, card: &str) -> String {
    format!("deck = \"{key}\"\n[[hold]]\ncard = \"{card}\"\nprotects = [\"commander\"]\n")
}

// ------------------------------------------------------------------ (1) byte identity
/// The only change to `card_utility` is `if has_dsl() && held_back(..) { return None }`. So every difference from
/// origin/main is a (seat, card) pair for which that is true. With no hook set: for EVERY deck in decks.json (mine,
/// pool, retired test decks) as seat 0 and as seat 1 (against "seph"), and EVERY card of the card database, check it is
/// never true, and that `held_back` is true exactly for the cards a real hold row names.
#[test]
fn no_deck_seat_and_no_db_card_is_held_back_beyond_the_pinned_rows() {
    let db = db();
    let n = db.len();
    let mut seats = 0;
    let mut named_true = 0;
    for d in decks() {
        let g = table(&[&d.key, "seph"]);
        for p in [P0, P1] {
            seats += 1;
            let key = g.player(p).key;
            let rows: BTreeSet<String> = plan_data::active(&g, p)
                .map(|l| l.plan.hold.iter().map(|h| h.card.clone()).collect())
                .unwrap_or_default();
            for i in 0..n {
                let c = CardId(i as u16);
                let cd = g.db.get(c);
                let held = plan_data::held_back(&g, p, c);
                assert_eq!(held, rows.contains(&*cd.name), "{key}/{p:?}: {}", cd.name);
                if held {
                    named_true += 1;
                    assert!(
                        !cd.has_dsl(),
                        "{key}: {} has a DSL and a real hold row: card_utility now differs",
                        cd.name
                    );
                }
            }
        }
    }
    assert!(seats > 100, "{seats}");
    assert!(named_true > 0, "no real row was ever exercised");
}

/// The same with each real plan file loaded as the hook plan (so every seat reads that file's hold rows, whatever its
/// own plan): no card with a DSL is held, so `card_utility` cannot differ however a seat is matched to a plan.
#[test]
fn every_real_plan_file_as_the_hook_plan_holds_no_card_with_a_dsl() {
    let g = table(&["yoshimaru", "seph"]);
    let n = g.db.len();
    let dir = data_dir().join("plans");
    let mut files = 0;
    for e in std::fs::read_dir(&dir).unwrap().flatten() {
        let path = e.path();
        if path.extension().is_none_or(|x| x != "toml") {
            continue;
        }
        let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
        let text = std::fs::read_to_string(&path).unwrap();
        let _plan = HoldPlan::with(&stem, &text);
        files += 1;
        for i in 0..n {
            let c = CardId(i as u16);
            let cd = g.db.get(c);
            assert!(!(cd.has_dsl() && plan_data::held_back(&g, P0, c)), "{stem}: {} has a DSL and is held", cd.name);
        }
    }
    assert!(files >= 20, "{files}");
}

/// A hold row is only read for a seat whose plan is ACTIVE (`plans::plays_plan`). List every plan file that has a hold
/// row and whether its seat reads it: a row in an inactive plan is dead data in production.
#[test]
fn which_real_hold_rows_are_read_at_all() {
    let mut inactive = vec![];
    for (key, l) in plan_data::all() {
        if l.plan.hold.is_empty() {
            continue;
        }
        let g = table(&[key, "seph"]);
        let on = plan_data::active(&g, P0).is_some();
        println!(
            "hold rows in {key}: {} (plan active: {on}, plays_plan: {})",
            l.plan.hold.len(),
            plans::plays_plan(key)
        );
        if !on {
            inactive.push(*key);
        }
    }
    // yoshimaru is the deck B1a is built for: its plan must be the one its seat reads, or adopting the row is a no-op
    let g = table(&["yoshimaru", "seph"]);
    assert!(plan_data::active(&g, P0).is_some(), "yoshimaru does not play its plan file by default");
    println!("plans with hold rows that are NOT active by default: {inactive:?}");
}

// ------------------------------------------------------------------ (2) the test hook
#[test]
fn the_guard_resets_the_hook_even_when_the_test_panics() {
    let g = table(&["yoshimaru", "seph"]);
    let c = card(&g, FLAWLESS);
    assert!(!plan_data::held_back(&g, P0, c));
    let r = std::panic::catch_unwind(|| {
        let _plan = HoldPlan::with("yoshimaru", &hold_text("yoshimaru", FLAWLESS));
        panic!("a test failing while the hook is set");
    });
    assert!(r.is_err());
    // the same thread (cargo reuses test threads): no stale plan
    assert!(!plan_data::held_back(&g, P0, c), "the hook leaked past a panicking test");
}

#[test]
fn the_hook_is_thread_local_and_a_second_thread_never_sees_it() {
    let _plan = HoldPlan::with("yoshimaru", &hold_text("yoshimaru", FLAWLESS));
    let g = table(&["yoshimaru", "seph"]);
    let c = card(&g, FLAWLESS);
    assert!(plan_data::held_back(&g, P0, c));
    let other = std::thread::scope(|s| {
        s.spawn(|| {
            let g = table(&["yoshimaru", "seph"]);
            plan_data::held_back(&g, P0, card(&g, FLAWLESS))
        })
        .join()
        .unwrap()
    });
    assert!(!other, "another thread saw this thread's hook");
}

/// Documented limitation, not a bug: with the hook set it replaces the seat's own plan for EVERY seat, so a seat with
/// no plan at all (and a seat whose plan is inactive) is held too. Only the hook, never production.
#[test]
fn the_hook_overrides_a_seat_that_has_no_plan() {
    let g = table(&["yoshimaru", "seph"]);
    let c = card(&g, FLAWLESS);
    assert!(plan_data::active(&g, P1).is_none(), "seph plays no plan");
    assert!(!plan_data::held_back(&g, P1, c));
    let _plan = HoldPlan::with("yoshimaru", &hold_text("yoshimaru", FLAWLESS));
    assert!(plan_data::held_back(&g, P1, c), "the hook did not reach the plan-less seat");
}

/// Nothing in any crate's src (other than the hook's own definition) calls the hook.
#[test]
fn no_production_code_sets_the_hook() {
    fn walk(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) {
        for e in std::fs::read_dir(dir).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.file_name().is_some_and(|n| n == "target") {
                    continue;
                }
                walk(&p, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    let crates = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut files = vec![];
    for c in std::fs::read_dir(&crates).unwrap().flatten() {
        let src = c.path().join("src");
        if src.is_dir() {
            walk(&src, &mut files);
        }
    }
    assert!(files.len() > 50);
    let mut hits = vec![];
    for f in files {
        let text = std::fs::read_to_string(&f).unwrap();
        for (i, l) in text.lines().enumerate() {
            if l.contains("set_hold_plan_for_test") || l.contains("TEST_HOLD_PLAN") {
                hits.push(format!("{}:{}", f.display(), i + 1));
            }
        }
    }
    // the definition (3 lines: the static, its doc, the fn) and the read in `held_back`, all in plan_data.rs
    assert!(hits.iter().all(|h| h.contains("plan_data.rs")), "{hits:?}");
}

// ------------------------------------------------------------------ (7) hold_problems and the plan-file guard
/// What `every_plan_file_fits_its_deck` would do with `hold_problems` added: the real plan files give no problem, and
/// the real yoshimaru plan with the row JD would adopt (`[[hold]] card = "Flawless Maneuver"`) loads, names only cards
/// of the deck and has no problem. So wiring it in is safe today.
#[test]
fn wiring_hold_problems_into_the_plan_guard_is_safe_and_the_adoption_row_fits() {
    let db = db();
    for (key, l) in plan_data::all() {
        assert_eq!(plan_data::hold_problems(l, &db), Vec::<String>::new(), "{key}");
    }
    let real = std::fs::read_to_string(data_dir().join("plans/yoshimaru.toml")).unwrap();
    let text = format!("{real}\n[[hold]]\ncard = \"{FLAWLESS}\"\nprotects = [\"commander\"]\n");
    let l = plan_data::load("yoshimaru", &text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(plan_data::hold_problems(&l, &db), Vec::<String>::new());
    let d = deck("yoshimaru");
    let cards: BTreeSet<&str> = d.cards.iter().map(|s| s.as_str()).chain([d.commander.as_str()]).collect();
    assert!(cards.contains(FLAWLESS), "Flawless Maneuver is not in the yoshimaru list");
    for n in plan_data::named_cards(&l) {
        assert!(cards.contains(n.as_str()), "{n:?} is not in the deck");
    }
}

/// `hold_problems` is silent about a row `card_utility` cannot act on (a protector without a DSL: the branch is gated
/// by `has_dsl()`): both the real rows pass it and do nothing. Lists every protector and counter row that is inert.
#[test]
fn hold_problems_accepts_rows_that_do_nothing() {
    let db = db();
    let mut inert = vec![];
    let mut live = vec![];
    for cd in db.iter() {
        let l = plan_data::load("x", &format!("deck = \"x\"\n[[hold]]\ncard = {:?}\n", &*cd.name)).unwrap();
        if plan_data::hold_problems(&l, &db).is_empty() {
            if cd.has_dsl() { live.push(cd.name.to_string()) } else { inert.push(cd.name.to_string()) }
        }
    }
    println!("rows hold_problems accepts that DO something: {} {:?}", live.len(), live);
    println!("rows hold_problems accepts that are INERT in card_utility (no DSL): {} {:?}", inert.len(), inert);
    assert!(inert.iter().any(|n| n == "Teferi's Protection") && inert.iter().any(|n| n == "Eerie Interlude"));
    assert!(live.iter().any(|n| n == FLAWLESS));
}
