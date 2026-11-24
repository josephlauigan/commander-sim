//! Dockside Extortionist ({1}{R}, Creature - Goblin Pirate 1/2): "When this creature enters, create X Treasure tokens,
//! where X is the number of artifacts and enchantments your opponents control." Banned in Commander, legal in Cali
//! Commander by the format's own list (data/formats.toml). The expected results come from that text and the
//! comprehensive rules, not from what the engine did before. Card code: impls/dockside.rs.

mod play_common;
use play_common::*;
use sim_core::ai::act::perform;
use sim_core::ai::brain::{Situation, card_utility, main_options};
use sim_core::engine::mana::{self, Source, total_mana};
use sim_core::engine::zones::{self, Tokens};
use sim_core::export::{RawDeck, load_decks};
use sim_core::format::Format;
use sim_core::impls::dockside::opposing_artifacts_and_enchantments as x_of;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

const DOCKSIDE: &str = "Dockside Extortionist";
const P3: PlayerId = PlayerId(3);

/// P0 plays the arzakon deck (WUBRG: a Treasure pays any colour), P1..P3 are opponents with empty boards
fn tbl() -> Game {
    table(&["arzakon", "yshtola", "karn", "seph"])
}

fn treasures(g: &Game, p: PlayerId) -> u32 {
    g.player(p).treasures
}

fn artifact_token(g: &mut Game, p: PlayerId) -> PermId {
    let spec = Tokens { types: vec!["construct", "artifact"], sick: false, ..Tokens::new(1, 2) };
    zones::make_tokens(g, p, spec).unwrap()[0]
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

// ------------------------------------------------------------------ what X counts
#[test]
fn x_counts_an_opponents_artifacts() {
    let mut g = tbl();
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Mana Vault");
    perm(&mut g, P0, DOCKSIDE);
    assert_eq!(treasures(&g, P0), 2, "two artifacts: two Treasures");
    assert_eq!(treasures(&g, P1), 0, "the opponent gets none");
}

#[test]
fn x_counts_an_opponents_enchantments() {
    let mut g = tbl();
    perm(&mut g, P1, "Rhystic Study");
    perm(&mut g, P1, "Mystic Remora");
    perm(&mut g, P1, "Phyrexian Arena");
    perm(&mut g, P0, DOCKSIDE);
    assert_eq!(treasures(&g, P0), 3);
}

#[test]
fn x_counts_artifacts_and_enchantments_together() {
    let mut g = tbl();
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Rhystic Study");
    perm(&mut g, P0, DOCKSIDE);
    assert_eq!(treasures(&g, P0), 2);
}

#[test]
fn an_artifact_creature_counts_once() {
    let mut g = tbl();
    let b = perm(&mut g, P1, "Academy Manufactor");
    assert!(g.is_creature(b));
    perm(&mut g, P1, "Llanowar Elves"); // a creature that is neither
    perm(&mut g, P0, DOCKSIDE);
    assert_eq!(treasures(&g, P0), 1, "the Manufactor once; the Elves not at all");
}

#[test]
fn an_enchantment_creature_counts_once() {
    let mut g = tbl();
    let c = perm(&mut g, P1, "Courser of Kruphix");
    assert!(g.is_creature(c));
    perm(&mut g, P0, DOCKSIDE);
    assert_eq!(treasures(&g, P0), 1);
}

#[test]
fn an_artifact_land_counts() {
    let mut g = tbl();
    lands(&mut g, P1, "Darksteel Citadel", 1, false);
    lands(&mut g, P1, "Treasure Vault", 1, true); // tapped makes no difference
    lands(&mut g, P1, "Mountain", 2, false); // plain lands do not
    perm(&mut g, P0, DOCKSIDE);
    assert_eq!(treasures(&g, P0), 2);
}

#[test]
fn artifact_tokens_count_treasure_clue_food_blood_and_a_construct() {
    let mut g = tbl();
    zones::add_treasure(&mut g, P1, 2).unwrap();
    zones::make_artifact_tokens(&mut g, P1, "Clue", 1).unwrap();
    zones::make_artifact_tokens(&mut g, P1, "Food", 1).unwrap();
    g.player_mut(P1).bloods = 1;
    artifact_token(&mut g, P1);
    assert_eq!(x_of(&g, P0), 6);
    perm(&mut g, P0, DOCKSIDE);
    assert_eq!(treasures(&g, P0), 6);
    assert_eq!(treasures(&g, P1), 2, "the opponent's Treasures are still there");
}

#[test]
fn own_permanents_and_tokens_are_not_counted() {
    let mut g = tbl();
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P0, "Rhystic Study");
    perm(&mut g, P0, "Academy Manufactor");
    zones::add_treasure(&mut g, P0, 3).unwrap();
    artifact_token(&mut g, P0);
    lands(&mut g, P0, "Darksteel Citadel", 1, false);
    perm(&mut g, P1, "Mana Vault");
    perm(&mut g, P0, DOCKSIDE);
    assert_eq!(treasures(&g, P0), 3 + 1, "its own three Treasures stay; one new for the opponent's Mana Vault");
}

#[test]
fn x_zero_makes_no_tokens_and_is_not_an_error() {
    let mut g = tbl();
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P1, "Llanowar Elves");
    lands(&mut g, P1, "Mountain", 3, false);
    let d = perm(&mut g, P0, DOCKSIDE);
    assert!(g.perm(d).on_bf, "Dockside is on the battlefield");
    assert_eq!(treasures(&g, P0), 0);
    assert!(!g.over);
}

#[test]
fn several_opponents_add_together() {
    let mut g = tbl();
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P2, "Mana Vault");
    perm(&mut g, P2, "Rhystic Study");
    perm(&mut g, P3, "Mystic Remora");
    assert_eq!(x_of(&g, P0), 4);
    perm(&mut g, P0, DOCKSIDE);
    assert_eq!(treasures(&g, P0), 4);
}

#[test]
fn a_dead_opponent_is_not_counted() {
    let mut g = tbl();
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P2, "Mana Vault");
    g.player_mut(P2).alive = false;
    perm(&mut g, P0, DOCKSIDE);
    assert_eq!(treasures(&g, P0), 1);
}

#[test]
fn a_phased_out_permanent_is_not_counted() {
    let mut g = tbl();
    let r = perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Mana Vault");
    g.perm_mut(r).phased = true;
    perm(&mut g, P0, DOCKSIDE);
    assert_eq!(treasures(&g, P0), 1);
}

#[test]
fn the_count_follows_the_controller_not_the_seat() {
    // P1 controls Dockside: its opponents are P0, P2 and P3, so P0's Sol Ring counts and P1's own Mana Vault does not
    let mut g = tbl();
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P1, "Mana Vault");
    perm(&mut g, P1, DOCKSIDE);
    assert_eq!(treasures(&g, P1), 1);
    assert_eq!(treasures(&g, P0), 0);
}

// ------------------------------------------------------------------ the trigger uses the stack
/// a person at `p` that passes priority (or lets `f` answer first); a person gets priority on their own trigger only
/// with autopass "all"
fn person(g: &mut Game, p: PlayerId, f: impl FnMut(&Request) -> Option<Answer> + Send + 'static) -> Arc<ScriptHuman> {
    let mut f = f;
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if let Some(a) = f(r) {
            return a;
        }
        if &*r.kind == "priority" { Answer::Action(json!({"do": "pass"})) } else { Answer::Decline }
    }));
    g.humans.add(p, h.clone());
    h.set_autopass("all");
    h
}

#[test]
fn it_counts_at_resolution_a_response_that_destroys_an_artifact_lowers_x() {
    let mut g = tbl();
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Mana Vault");
    let disenchant = hand(&mut g, P0, &["Disenchant"])[0];
    pool(&mut g, P0, &[('W', 2)]);
    let cast_it = Arc::new(AtomicBool::new(false));
    let flag = cast_it.clone();
    let h = person(&mut g, P0, move |r| {
        if &*r.kind == "priority" && !flag.swap(true, Ordering::SeqCst) {
            return Some(Answer::Action(json!({"do": "cast", "card": 0})));
        }
        if &*r.kind == "target" {
            return Some(Answer::Pick(r.choices.iter().position(|c| c.contains("Mana Vault")).unwrap_or(0)));
        }
        None
    });
    perm(&mut g, P0, DOCKSIDE);
    assert!(cast_it.load(Ordering::SeqCst), "the person responded to the trigger");
    assert!(g.player(P0).gy.contains(&disenchant), "Disenchant resolved first");
    assert_eq!(x_of(&g, P0), 1);
    assert_eq!(treasures(&g, P0), 1, "X was counted as the trigger resolved: one artifact was left");
    drop(h);
}

#[test]
fn it_counts_at_resolution_a_response_that_adds_an_artifact_raises_x() {
    let mut g = tbl();
    perm(&mut g, P1, "Mana Vault");
    hand(&mut g, P1, &["Deduce"]);
    pool(&mut g, P1, &[('U', 2)]);
    let cast_it = Arc::new(AtomicBool::new(false));
    let flag = cast_it.clone();
    // P1 is a person who answers the trigger with an instant that makes an artifact (Deduce: a Clue)
    g.active = Some(P0);
    let _h = person(&mut g, P1, move |r| {
        if &*r.kind == "priority" && !flag.swap(true, Ordering::SeqCst) {
            return Some(Answer::Action(json!({"do": "cast", "card": 0})));
        }
        None
    });
    perm(&mut g, P0, DOCKSIDE);
    assert!(cast_it.load(Ordering::SeqCst), "P1 responded to the trigger");
    assert_eq!(g.player(P1).clues, 1, "Deduce resolved before the trigger");
    assert_eq!(treasures(&g, P0), 2, "Mana Vault and the Clue made in response");
}

#[test]
fn elesh_norn_blanks_the_trigger() {
    let mut g = tbl();
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Elesh Norn, Mother of Machines");
    perm(&mut g, P0, DOCKSIDE);
    assert_eq!(treasures(&g, P0), 0, "an opponent's Elesh Norn: the enter trigger does not trigger");
}

#[test]
fn a_doubler_makes_a_second_trigger_that_counts_again() {
    let mut g = tbl();
    perm(&mut g, P0, "Panharmonicon");
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Mana Vault");
    perm(&mut g, P0, DOCKSIDE);
    assert_eq!(treasures(&g, P0), 4, "two triggers, each counting two");
}

#[test]
fn token_doublers_apply_to_the_treasures() {
    let mut g = tbl();
    perm(&mut g, P0, "Academy Manufactor");
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P0, DOCKSIDE);
    // Academy Manufactor: a Treasure also makes a Food and a Clue
    assert_eq!(treasures(&g, P0), 1);
    assert_eq!((g.player(P0).foods, g.player(P0).clues), (1, 1));
}

// ------------------------------------------------------------------ the Treasures are real artifacts
#[test]
fn the_treasures_tap_for_any_colour_and_make_mox_opals_metalcraft() {
    let mut g = tbl();
    perm(&mut g, P0, "Mox Opal");
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P1, "Mana Vault");
    assert_eq!(total_mana(&g, P0, false), 2, "Sol Ring only: two artifacts, no metalcraft");
    perm(&mut g, P0, DOCKSIDE);
    assert_eq!(treasures(&g, P0), 1);
    assert_eq!(total_mana(&g, P0, false), 2 + 1 + 1, "Sol Ring, the Treasure and Mox Opal (three artifacts now)");
    let units = mana::mana_units(&g, P0, false);
    let t = units.iter().find(|u| u.src == Source::Treasure).expect("a Treasure unit");
    assert!(['W', 'U', 'B', 'R', 'G'].iter().all(|&c| t.cols.has(c)), "any one colour");
}

#[test]
fn the_treasures_are_artifacts_for_other_artifact_counts() {
    // Karn's and Cloud's counts (the engine's one count of a player's artifacts) include them
    let mut g = tbl();
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Mana Vault");
    perm(&mut g, P1, "Rhystic Study");
    let before = sim_core::impls::karn::artifacts(&g, P0);
    perm(&mut g, P0, DOCKSIDE);
    assert_eq!(sim_core::impls::karn::artifacts(&g, P0), before + 3);
}

// ------------------------------------------------------------------ practice mode: a person casts it
#[test]
fn a_person_who_casts_it_gets_the_treasures_and_is_never_asked() {
    let mut g = tbl();
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P2, "Rhystic Study");
    hand(&mut g, P0, &[DOCKSIDE]);
    pool(&mut g, P0, &[('R', 2)]);
    let cast_it = Arc::new(AtomicBool::new(false));
    let flag = cast_it.clone();
    let asked: Arc<std::sync::Mutex<Vec<String>>> = Arc::default();
    let seen = asked.clone();
    let h = person(&mut g, P0, move |r| {
        seen.lock().unwrap().push(format!("{}: {}", r.kind, r.prompt));
        if &*r.kind == "priority" && !flag.swap(true, Ordering::SeqCst) {
            return Some(Answer::Action(json!({"do": "cast", "card": 0})));
        }
        None
    });
    g.active = Some(P0);
    g.step = sim_core::state::Step::Main1;
    sim_core::human::human::human_main(&mut g, P0, false).unwrap();
    assert!(cast_it.load(Ordering::SeqCst), "the person was offered the cast");
    assert_eq!(perms_named(&g, P0, DOCKSIDE).len(), 1);
    assert_eq!(treasures(&g, P0), 2);
    let asked = asked.lock().unwrap();
    assert!(
        !asked.iter().any(|q| q.contains("Dockside") && !q.starts_with("priority")),
        "no question about the trigger: {asked:?}"
    );
    drop(h);
}

// ------------------------------------------------------------------ the AI
#[test]
fn the_ai_holds_it_while_no_opponent_has_an_artifact_or_an_enchantment() {
    let mut g = tbl();
    g.active = Some(P0);
    g.player_mut(P0).turns = 3;
    lands(&mut g, P0, "Mountain", 2, false);
    hand(&mut g, P0, &[DOCKSIDE]);
    perm(&mut g, P1, "Llanowar Elves");
    let c = card(&g, DOCKSIDE);
    let s = Situation::new(&g, P0);
    assert_eq!(card_utility(&g, P0, &s, c), None, "nothing to count: wait for a later turn");
    assert!(main_options(&mut g, P0, false).unwrap().iter().all(|o| !o.label.contains("Dockside")));
}

#[test]
fn the_ai_casts_it_when_an_opponent_has_several_artifacts_and_gets_its_treasures() {
    let mut g = tbl();
    g.active = Some(P0);
    g.player_mut(P0).turns = 3;
    lands(&mut g, P0, "Mountain", 2, false);
    hand(&mut g, P0, &[DOCKSIDE]);
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Mana Vault");
    perm(&mut g, P2, "Rhystic Study");
    let c = card(&g, DOCKSIDE);
    let s = Situation::new(&g, P0);
    let u = card_utility(&g, P0, &s, c).expect("three artifacts and enchantments: worth casting");
    assert!(u > 5.0, "a priority of 62 reads as about 6, got {u}");
    let opts = main_options(&mut g, P0, false).unwrap();
    let o = opts.iter().find(|o| o.label.contains("Dockside")).unwrap_or_else(|| panic!("not offered: {opts:?}"));
    assert!(perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert_eq!(perms_named(&g, P0, DOCKSIDE).len(), 1);
    assert_eq!(treasures(&g, P0), 3);
}

#[test]
fn the_ai_wants_it_more_with_more_to_count() {
    let mut g = tbl();
    g.active = Some(P0);
    g.player_mut(P0).turns = 3;
    hand(&mut g, P0, &[DOCKSIDE]);
    let c = card(&g, DOCKSIDE);
    let prio = |g: &Game| sim_core::ai::decks::deck_prio_f(g, P0, c);
    assert_eq!(prio(&g), 0.0);
    perm(&mut g, P1, "Sol Ring");
    let one = prio(&g);
    assert!(one > 0.0);
    perm(&mut g, P2, "Mana Vault");
    let two = prio(&g);
    assert!(two > one);
}

// ------------------------------------------------------------------ the format
fn raw_deck(format: Option<&str>, cards: &[&str]) -> RawDeck {
    RawDeck {
        tier: None,
        key: "d".into(),
        name: "d".into(),
        commander: "X".into(),
        commander2: None,
        ident: None,
        tiers: vec![],
        format: format.map(String::from),
        cards: cards.iter().map(|c| c.to_string()).collect(),
    }
}

#[test]
fn cali_admits_it_and_a_commander_table_refuses_a_cali_deck_holding_it() {
    // Scryfall bans it in Commander ...
    let cache: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root().join("data/scryfall_cache.json")).unwrap()).unwrap();
    let rec = &cache["dockside extortionist"];
    assert!(!rec.is_null(), "Scryfall record of the card");
    assert_eq!(rec["legalities"]["commander"], "banned");
    // ... Cali's own list does not name it, so a Cali table seats a Commander deck that holds it
    let f = load_decks(&root().join("data/decks.json")).unwrap();
    let mut cali = f.formats["cali"].clone();
    cali.key = "cali".into();
    assert!(!cali.banned.iter().any(|b| b.eq_ignore_ascii_case(DOCKSIDE)), "not on the Cali list");
    let mut cmd = Format::commander();
    cmd.key = "commander".into();
    assert!(cali.admits(&raw_deck(None, &["Sol Ring", DOCKSIDE]), &cmd).is_ok(), "Cali admits it");
    // a Cali deck holding it sits at its own table, and a Commander table (4 seats, Scryfall bans) does not seat it
    let own = raw_deck(Some("cali"), &[DOCKSIDE]);
    assert!(cali.admits(&own, &cali).is_ok());
    assert!(cmd.admits(&own, &cali).is_err(), "a Commander table refuses a deck of the Cali format");
}
