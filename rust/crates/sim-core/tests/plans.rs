//! Phase 4 deck plans (`data/plans/<deck>.toml`, `ai/plan_data.rs`): every plan file loads, names only cards of its
//! deck (`data/decks.json`, or a retired deck's import list), names catalogue combos the deck can assemble, and
//! matches the Commander Spellbook variants it cites. A deck list change that strands a plan line fails here.
//!
//! Combo data by Commander Spellbook, https://commanderspellbook.com.

use serde_json::Value;
use sim_core::ai::plan_data;
use sim_core::testkit::data_dir;
use std::collections::BTreeSet;
use std::fs;

/// the deck's cards (commanders included): decks.json, or for a retired deck its import list
fn deck_cards(key: &str, retired: bool) -> BTreeSet<String> {
    if retired {
        let p = data_dir().join(format!("../decklists/pool/retired/{key}.md"));
        let text = fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e}", p.display()));
        let list = text.split("## Import list").nth(1).unwrap_or_else(|| panic!("{key}: no import list"));
        return list
            .lines()
            .filter_map(|l| l.trim().split_once(' ').filter(|(n, _)| n.parse::<u32>().is_ok()).map(|x| x.1))
            .map(|n| n.trim().to_string())
            .collect();
    }
    let v: Value = serde_json::from_str(&fs::read_to_string(data_dir().join("decks.json")).unwrap()).unwrap();
    let d = v["pool"]
        .as_array()
        .unwrap()
        .iter()
        .chain(v["mine"].as_array().into_iter().flatten())
        .find(|d| d["key"] == key)
        .unwrap_or_else(|| panic!("plan file for {key:?}, which is not in data/decks.json (retired = true?)"));
    let mut s: BTreeSet<String> = d["cards"].as_array().unwrap().iter().map(|c| c.as_str().unwrap().into()).collect();
    match &d["commander"] {
        Value::String(c) => {
            s.insert(c.clone());
        }
        Value::Array(cs) => s.extend(cs.iter().map(|c| c.as_str().unwrap().to_string())),
        _ => {}
    }
    s
}

/// a Spellbook variant's card names, from the deck's Spellbook file (any list)
fn spellbook_cards(key: &str, id: &str) -> Option<BTreeSet<String>> {
    let v: Value =
        serde_json::from_str(&fs::read_to_string(data_dir().join(format!("spellbook/{key}.json"))).ok()?).ok()?;
    v.as_object()?
        .values()
        .filter_map(|x| x.as_array())
        .flatten()
        .find(|x| x["id"] == id)
        .map(|x| x["cards"].as_array().unwrap().iter().map(|c| c["name"].as_str().unwrap().to_string()).collect())
}

#[test]
fn every_plan_file_fits_its_deck() {
    let all = plan_data::all();
    assert!(!all.is_empty(), "no plan files embedded");
    let mut errs = vec![];
    for (key, l) in all.iter() {
        let deck = deck_cards(key, l.plan.retired);
        for n in plan_data::named_cards(l) {
            if !deck.contains(&n) {
                errs.push(format!("{key}: {n:?} is not in the deck"));
            }
        }
        for (x, (groups, _)) in l.plan.line.iter().zip(&l.lines) {
            for grp in groups {
                if !grp.iter().any(|n| deck.contains(n)) {
                    errs.push(format!("{key}: line {:?}: no card of {grp:?} is in the deck", x.name));
                }
            }
            if let Some(id) = &x.spellbook {
                let want: BTreeSet<String> = x.pieces.iter().flatten().cloned().collect();
                match spellbook_cards(key, id) {
                    None => errs.push(format!("{key}: line {:?}: no Spellbook variant {id}", x.name)),
                    Some(s) if !s.is_subset(&want) => {
                        errs.push(format!("{key}: line {:?}: Spellbook {id} is {s:?}", x.name))
                    }
                    _ => {}
                }
            }
        }
    }
    assert!(errs.is_empty(), "{}", errs.join("\n"));
}

#[test]
fn a_bad_plan_file_is_refused_with_its_reason() {
    let bad = |t: &str| plan_data::load("x", t).err().unwrap();
    assert!(bad("deck = \"y\"").contains("named for"));
    assert!(bad("deck = \"x\"\nfoo = 1").contains("unknown field"));
    assert!(bad("deck = \"x\"\n[[line]]\nname = \"a\"\ncombo = \"no such combo\"").contains("no catalogue combo"));
    assert!(bad("deck = \"x\"\n[[prio]]\nvalue = 3").contains("selects no card"));
    assert!(bad("deck = \"x\"\n[[prio]]\ncommander = true\nwhen = [\"moon > 3\"]\nvalue = 3").contains("unknown"));
    assert!(
        plan_data::load("x", "deck = \"x\"\n[[prio]]\ncards = [\"A\"]\nwhen = [\"creatures>=6 & lands<3\"]\nvalue = 1")
            .is_ok()
    );
}

/// `cargo test --release --test plans -- --ignored --nocapture catalogue_lines` (PLAN_DECKS=key,...): the catalogue
/// combos each deck can assemble, the `[[line]] combo = ...` candidates for its plan file
#[test]
#[ignore]
fn catalogue_lines_report() {
    let keys = std::env::var("PLAN_DECKS").unwrap_or_default();
    for key in keys.split(',').filter(|k| !k.is_empty()) {
        let deck = deck_cards(key, false);
        println!("{key}:");
        for c in sim_core::impls::combos::all() {
            if c.groups.iter().all(|g| g.iter().any(|n| deck.contains(*n))) {
                println!("  {:?}", c.name);
            }
        }
    }
}

// ------------------------------------------------------------------ phase4-cedh: the schema extensions
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::{card, hand, perm, table};

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const YU: &str = "yuriko-dimir-ninjas";
const YA: &str = "yawgmoth-mono-black-aristocrats";
const KI: &str = "kinnan-bonder-mana-combo";

fn nm(g: &Game, c: sim_core::ids::CardId) -> String {
    g.db.get(c).name.to_string()
}

/// Yuriko: Scroll Rack counts as a put-back only on the battlefield (`bf:`), a tutor that puts its card on top is a
/// put-back (`?to-top`), and Rhystic Study goes ahead of the line while it is two or more steps away (`engines_ahead`)
#[test]
fn battlefield_only_groups_conditions_and_engines_ahead() {
    let l = plan_data::get(YU).unwrap();
    let mut g = table(&[YU, KI]);
    g.player_mut(P0).hand.clear();
    let first = |g: &Game| nm(g, plan_data::wish_for(l, g, P0)[0]);
    assert_eq!(first(&g), "Rhystic Study", "two steps away: the engine ahead first");
    hand(&mut g, P0, &["Scroll Rack"]);
    assert_eq!(first(&g), "Rhystic Study", "Scroll Rack in hand puts nothing back");
    g.player_mut(P0).hand.clear();
    perm(&mut g, P0, "Scroll Rack");
    assert_eq!(first(&g), "Dig Through Time", "Scroll Rack out: the big card next");
    let mut g = table(&[YU, KI]);
    g.player_mut(P0).hand.clear();
    g.player_mut(P0).to_top = true;
    assert_eq!(first(&g), "Dig Through Time", "a tutor to the top is the put-back");
}

/// Yawgmoth: the undying line needs two undying creatures (`counts`), so one undying creature out leaves it as far as
/// the Mikaeus line (Mikaeus first, in file order), and two bring the payoff first; `bf:<set>` reads the count
#[test]
fn n_cards_of_a_group_and_battlefield_sets() {
    let l = plan_data::get(YA).unwrap();
    let mut g = table(&[YA, KI]);
    g.player_mut(P0).hand.clear();
    perm(&mut g, P0, "Yawgmoth, Thran Physician");
    perm(&mut g, P0, "Geralf's Messenger");
    assert_eq!(nm(&g, plan_data::wish_for(l, &g, P0)[0]), "Mikaeus, the Unhallowed");
    let bg = card(&g, "Butcher Ghoul");
    assert_eq!(plan_data::prio_for(l, &g, P0, bg), Some(78.0), "one undying out: the second at 78");
    perm(&mut g, P0, "Butcher Ghoul");
    let w = plan_data::wish_for(l, &g, P0);
    let payoffs = l.sets[1].clone();
    assert!(payoffs.contains(&nm(&g, w[0])), "the loop is out: a payoff first, got {}", nm(&g, w[0]));
    assert_eq!(plan_data::prio_for(l, &g, P0, card(&g, "Blood Artist")), Some(80.0));
    assert_eq!(plan_data::prio_for(l, &g, P0, card(&g, "Nether Traitor")), None, "the loop is there");
}

/// a card's value from the rest of the hand: `hand-max-mv` leaves the rated card out
#[test]
fn hand_conditions_read_the_rest_of_the_hand() {
    let t = "deck = \"x\"\n[[prio]]\ncards = [\"Brainstorm\"]\nwhen = [\"hand-max-mv>=5\"]\nvalue = 72\nelse = 45";
    let l = plan_data::load("x", t).unwrap();
    let mut g = table(&[YU, KI]);
    g.player_mut(P0).hand.clear();
    let bs = hand(&mut g, P0, &["Brainstorm", "Ponder"])[0];
    assert_eq!(plan_data::prio_for(&l, &g, P0, bs), Some(45.0));
    hand(&mut g, P0, &["Dig Through Time"]);
    assert_eq!(plan_data::prio_for(&l, &g, P0, bs), Some(72.0));
}

/// the five Tier 6 slots of a deck's hand plan code
struct HandSlots {
    prio: fn(&Game, PlayerId, sim_core::ids::CardId) -> Option<f64>,
    wish: fn(&Game, PlayerId) -> Option<Vec<sim_core::ids::CardId>>,
    keep: fn(&Game, PlayerId) -> Option<bool>,
    hold_up: fn(&Game, PlayerId) -> Option<f64>,
    go_now: fn(&Game, PlayerId, &str) -> Option<bool>,
}

/// a Tier 6 plan file says what its hand plan code says: keep, the priority of every card in hand, the wish list,
/// hold-up and every catalogue combo's go-now, on 600 dealt states (seven cards in hand, 0-5 more on the battlefield,
/// turns and rounds 1-7, the first to third draw)
fn plan_matches_hand(key: &str, h: HandSlots) {
    let l = plan_data::get(key).unwrap();
    let mut g = table(&[key, KI]);
    let mut s: u64 = 12345;
    let (mut kept, mut n) = (0, 0);
    for i in 0..600u64 {
        let mut g2 = g.clone();
        let pl = g2.player_mut(P0);
        for k in (1..pl.library.len()).rev() {
            s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            pl.library.swap(k, (s >> 33) as usize % (k + 1));
        }
        pl.hand.clear();
        for _ in 0..7 {
            let c = pl.library.pop().unwrap();
            pl.hand.push(c);
        }
        pl.stats.insert("mull_try", (i % 3) as i64);
        pl.turns = 1 + (i % 7) as u32;
        g2.round = 1 + (i % 7) as u32;
        let keep = (h.keep)(&g2, P0);
        assert_eq!(plan_data::t6::keep(l, &g2, P0), keep, "{key} keep {:?}", g2.player(P0).hand);
        kept += usize::from(keep == Some(true));
        n += 1;
        for _ in 0..i % 6 {
            let c = *g2.player(P0).library.last().unwrap();
            let name = nm(&g2, c);
            perm(&mut g2, P0, &name);
        }
        for &c in &g2.player(P0).hand.clone() {
            assert_eq!(plan_data::t6::prio(l, &g2, P0, c), (h.prio)(&g2, P0, c), "{key} prio {}", nm(&g2, c));
        }
        // a hand list may name cards outside the library (the tutors skip them): compare what a tutor can find
        let lib = g2.player(P0).library.clone();
        let hw = (h.wish)(&g2, P0).map(|v| v.into_iter().filter(|c| lib.contains(c)).collect::<Vec<_>>());
        // and may repeat a card (a tutor and `bottom_pick` read its first place)
        let hw =
            hw.map(|v| v.iter().enumerate().filter(|(i, c)| !v[..*i].contains(c)).map(|x| *x.1).collect::<Vec<_>>());
        let hw = hw.filter(|v| !v.is_empty());
        assert_eq!(plan_data::t6::wish(l, &g2, P0), hw, "{key} wish, state {i}");
        assert_eq!(plan_data::t6::hold_up(l, &g2, P0), (h.hold_up)(&g2, P0), "{key} hold_up, state {i}");
        for cmb in sim_core::impls::combos::all() {
            let x = plan_data::t6::go_now(l, &g2, P0, cmb.name);
            assert_eq!(x, (h.go_now)(&g2, P0, cmb.name), "{key} go_now {}", cmb.name);
        }
    }
    g.round = 1;
    assert!(kept > 50 && kept < n, "{key}: kept {kept} of {n}: the rule both keeps and ships");
}

#[test]
fn the_scion_plan_file_matches_its_hand_code() {
    use sim_core::impls::t6_plan_scion as h;
    plan_matches_hand(
        "scion",
        HandSlots { prio: h::prio, wish: h::wish, keep: h::keep, hold_up: h::hold_up, go_now: h::go_now },
    );
}

/// hold-up and go-now rules: the first rule that holds decides; a go-now rule without `go` leaves it to the table
#[test]
fn hold_up_and_go_now_rules() {
    let t = r#"deck = "x"
[sets]
free = ["Force of Will"]
[[hold_up]]
when = ["turn<=2"]
value = -2.0
[[hold_up]]
when = ["hand:free>=1"]
value = 1.0
[[go_now]]
combos = ["Kinnan + Basalt Monolith"]
when = ["round>=6"]
[[go_now]]
combos = ["Kinnan + Basalt Monolith"]
when = ["opp-out:Sol Ring"]
go = false
"#;
    let l = plan_data::load("x", t).unwrap();
    let mut g = table(&[KI, "dack-fayden-white-stax"]);
    g.player_mut(P0).hand.clear();
    g.player_mut(P0).turns = 1;
    assert_eq!(plan_data::hold_up_for(&l, &g, P0), Some(-2.0));
    g.player_mut(P0).turns = 4;
    assert_eq!(plan_data::hold_up_for(&l, &g, P0), None);
    hand(&mut g, P0, &["Force of Will"]);
    assert_eq!(plan_data::hold_up_for(&l, &g, P0), Some(1.0));
    let k = "Kinnan + Basalt Monolith";
    assert_eq!(plan_data::go_now_for(&l, &g, P0, k), None);
    perm(&mut g, P1, "Sol Ring");
    assert_eq!(plan_data::go_now_for(&l, &g, P0, k), Some(false));
    assert_eq!(plan_data::go_now_for(&l, &g, P0, "Kinnan + Freed from the Real / Pemmin's Aura"), None);
    g.round = 6;
    assert_eq!(plan_data::go_now_for(&l, &g, P0, k), None, "late: the shared rule");
}

#[test]
fn the_extensions_are_validated() {
    let bad = |t: &str| plan_data::load("x", t).err().unwrap();
    let line = "deck = \"x\"\n[[line]]\nname = \"a\"\npieces = [[\"A\"], [\"B\"]]\n";
    assert!(bad(&format!("{line}counts = [1]")).contains("1 counts for 2 groups"));
    assert!(bad(&format!("{line}counts = [1, 0]")).contains("a count of 0"));
    assert!(bad("deck = \"x\"\n[[line]]\nname = \"a\"\npieces = [[\"?to-top\"]]").contains("no card"));
    assert!(bad("deck = \"x\"\n[[line]]\nname = \"a\"\npieces = [[\"A\", \"?moon\"]]").contains("unknown condition"));
    assert!(bad("deck = \"x\"\n[[hold_up]]\nwhen = [\"hand:free>=1\"]\nvalue = 1.0").contains("no set \"free\""));
    assert!(bad("deck = \"x\"\n[[go_now]]\ncombos = [\"No such\"]").contains("no catalogue combo matches"));
    assert!(bad("deck = \"x\"\n[[wish]]\ncards = []\nscore = 1.0").contains("no cards"));
    assert!(bad("deck = \"x\"\n[mulligan]\n[[mulligan.rule]]\nwhen = []\nkeep = true").contains("no when"));
    assert!(
        plan_data::load("x", "deck = \"x\"\n[[prio]]\ncommander = true\nwhen = [\"mana<one-lander\"]\nvalue = 1")
            .is_ok()
    );
}

#[test]
fn conditions_nest_and_sum() {
    let t = "deck = \"x\"\n[sets]\na = [\"Brainstorm\"]\nb = [\"Ponder\"]\n[[prio]]\ncards = [\"Brainstorm\"]\n\
             when = [\"hand:a+hand:b>=2 & (turn<=1 | turn>=9)\"]\nvalue = 72\nelse = 45";
    let l = plan_data::load("x", t).unwrap();
    let mut g = table(&[YU, KI]);
    g.player_mut(P0).hand.clear();
    let bs = hand(&mut g, P0, &["Brainstorm"])[0];
    g.player_mut(P0).turns = 1;
    assert_eq!(plan_data::prio_for(&l, &g, P0, bs), Some(45.0), "one card of a+b");
    hand(&mut g, P0, &["Ponder"]);
    assert_eq!(plan_data::prio_for(&l, &g, P0, bs), Some(72.0));
    g.player_mut(P0).turns = 5;
    assert_eq!(plan_data::prio_for(&l, &g, P0, bs), Some(45.0), "neither turn");
    assert!(
        plan_data::load("x", "deck = \"x\"\n[[prio]]\ncommander = true\nwhen = [\"(turn<=1\"]\nvalue = 1").is_err()
    );
}

