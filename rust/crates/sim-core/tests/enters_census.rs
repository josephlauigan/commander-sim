//! The census: every card in data/decks.json (mine and pool) is put onto the battlefield by its real path (cast for a
//! permanent, `play_land_card` for a land, `resolve` for a Battle) twice, with and without an opponent's live Elesh Norn,
//! Mother of Machines, from the same fixture (life 50, creatures, instants, lands, an artifact and an enchantment in the
//! graveyards, creatures and artifacts on both sides, utility lands, counters, a populated library), and the whole game
//! state and the RNG are compared. The Oracle text decides what the answer must be:
//!   TRIG   a "When/Whenever ... enters" clause (landfall and hideaway count; "enters tapped", "enters with", "enters
//!          prepared", "As ... enters" and a Saga's lore counter do not): the two runs must differ (the trigger is blanked)
//!   SETUP  everything else: the two runs must be identical
//! Two more runs with the Elesh neutered separate "a body was added" from "the trigger was blanked". A card whose result
//! disagrees with its Oracle class is a bug, or goes into `tests/enters_census_allow.txt`, one row per line:
//! `card | [fixture] or [deferred] reason`. [fixture] means this fixture cannot exercise the card's guard (no later
//! entrant, a target is missing, the extra body changes an AI choice: say which); [deferred] means a named existing model
//! or ticket outside P5b. A row whose card now agrees is fine to leave; a new disagreement must be fixed or reasoned.
//! The opponent-suppression test prints its own-Elesh columns. The doubling test asserts all nine modes:
//! baseline, opponent Elesh live/inert, own Elesh live/inert, own Vocalist live/inert and own Panharmonicon live/inert.
//! It also installs each observer without firing its own entry, then separately casts a 4/4, creates a 1/1 and a 4/4,
//! drops a basic land and creates a Treasure. Its exceptions are in `enters_census_double_allow.txt`;
//! unexplained rows and stale non-fixture rows fail, and numeric over-counts require a nonlinear explanation.
//! `CENSUS_CARD=<name>` runs one card; `ENTERS_CENSUS_REPORT=<file>` writes the full table.

use sim_core::engine::{cast, turn, zones};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game};
use sim_core::testkit::*;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, OnceLock};

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const ELESH: &str = "Elesh Norn, Mother of Machines";

fn tbl() -> Game {
    table(&["heliod-mono-white-stax", "seph", "sauron"])
}
fn raw(g: &mut Game, p: PlayerId, name: &str) -> PermId {
    let c = card(g, name);
    let m = g.new_perm(p, Some(c), "", 0, 0);
    g.perm_mut(m).on_bf = true;
    g.player_mut(p).perms.push(m);
    if g.registry.get(c).is_some_and(|i| i.live()) {
        g.hooks.push(m);
    }
    m
}
fn oracle_text(v: &serde_json::Value) -> String {
    let mut text = v["oracle_text"].as_str().unwrap_or("").to_string();
    if let Some(faces) = v["card_faces"].as_array() {
        for f in faces {
            text.push('\n');
            text.push_str(f["oracle_text"].as_str().unwrap_or(""));
        }
    }
    text
}
fn oracle_trig(text: &str) -> bool {
    let text = text.to_lowercase();
    text.split(['.', '\n']).any(|clause| {
        ["when ", "whenever "].into_iter().any(|lead| {
            clause.match_indices(lead).any(|(i, _)| {
                let condition = clause[i + lead.len()..].split(',').next().unwrap_or("");
                condition.split_whitespace().any(|w| matches!(w.trim_matches(['(', ')', ';', '"']), "enter" | "enters"))
            })
        })
    }) || text.contains("landfall")
        || text.contains("hideaway")
}
fn census_fixture() -> (Game, [PermId; 2]) {
    let mut g = tbl();
    for p in [P0, P1, P2] {
        g.player_mut(p).life = 50;
        for n in ["Mother of Runes", "Serra Angel", "Viscera Seer", "Yoshimaru, Ever Faithful", "Thraben Inspector"] {
            raw(&mut g, p, n);
        }
        for n in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
            lands(&mut g, p, n, 3, n == "Island");
        }
        raw(&mut g, p, "Sol Ring");
        raw(&mut g, p, "Mask of Memory");
        lands(&mut g, p, "Command Tower", 1, false);
        lands(&mut g, p, "War Room", 1, false);
        if p != P0 {
            lands(&mut g, p, "Wastes", 1, false);
        }
        for n in [
            "Grave Titan",
            "Elvish Mystic",
            "Counterspell",
            "Swords to Plowshares",
            "Island",
            "Mask of Memory",
            "Phyrexian Arena",
        ] {
            let c = take(&mut g, p, n);
            g.player_mut(p).gy.push(c);
        }
        hand(
            &mut g,
            p,
            &["Plains", "Forest", "Serra Angel", "Mask of Memory", "Brainstorm", "Captain America, Team Leader"],
        );
        let lib = [
            "Plains",
            "Island",
            "Serra Angel",
            "Elvish Mystic",
            "Counterspell",
            "Mask of Memory",
            "Viscera Seer",
            "Goblin Warchief",
        ]
        .into_iter()
        .cycle()
        .take(72)
        .map(|n| card(&g, n))
        .collect();
        g.player_mut(p).library = lib;
    }
    g.perm_mut(PermId(0)).plus = 1;
    let islands: Vec<_> =
        g.player(P0).lands.iter().copied().filter(|&l| g.perm(l).cd == g.db.id("Island")).take(2).collect();
    for l in islands {
        g.perm_mut(l).tapped = false;
    }
    let sentinels = [P0, P1].map(|p| {
        let m = raw(&mut g, p, ELESH);
        g.perm_mut(m).eot_kw = vec!["shroud", "indestructible"];
        g.perm_mut(m).on_bf = false;
        g.player_mut(p).perms.retain(|&x| x != m);
        g.hooks.retain(|&x| x != m);
        m
    });
    g.hooks.clear();
    (g, sentinels)
}
fn snapshot<const N: usize>(g: &Game, sentinels: [PermId; N], template: &Game) -> String {
    let mut s = g.clone();
    // Normalize only the controlled input slots/membership. All other dynamic Debug state and RNG are compared.
    for m in sentinels {
        s.perms[m.0 as usize] = template.perm(m).clone();
        for p in &mut s.players {
            p.perms.retain(|&x| x != m);
        }
        s.hooks.retain(|&x| x != m);
    }
    // Identical immutable Arcs need not be rendered thousands of times.
    static EMPTY_DB: OnceLock<Arc<sim_core::cards::CardDb>> = OnceLock::new();
    static EMPTY_REG: OnceLock<Arc<sim_core::hooks::Registry>> = OnceLock::new();
    assert!(Arc::ptr_eq(&g.db, &template.db));
    s.db = EMPTY_DB.get_or_init(|| Arc::new(sim_core::cards::CardDb::from_raw(&[]).unwrap())).clone();
    s.registry = EMPTY_REG.get_or_init(|| Arc::new(sim_core::hooks::Registry::default())).clone();
    format!("{s:?}")
}
fn effects<const N: usize>(g: &Game, sentinels: [PermId; N]) -> Vec<i64> {
    let mut out = vec![];
    for p in &g.players {
        out.extend([
            p.life as i64,
            p.hand.len() as i64,
            p.library.len() as i64,
            p.gy.len() as i64,
            p.exile.len() as i64,
            p.energy as i64,
            p.treasures as i64,
        ]);
        let ms: Vec<_> = p.perms.iter().chain(p.lands.iter()).copied().filter(|m| !sentinels.contains(m)).collect();
        out.extend([
            ms.iter().filter(|&&m| g.perm(m).on_bf).count() as i64,
            ms.iter().filter(|&&m| g.perm(m).token && g.perm(m).on_bf).count() as i64,
            ms.iter().map(|&m| g.perm(m).plus as i64).sum(),
            ms.iter().map(|&m| g.perm(m).data.int(DataKey::Counters)).sum(),
            ms.iter().map(|&m| g.perm(m).eot_pt.0 as i64).sum(),
        ]);
    }
    out
}
#[test]
fn every_card_enters_as_its_oracle_text_says_under_an_opponents_elesh() {
    let decks = sim_core::export::load_decks(&data_dir().join("decks.json")).unwrap();
    let names: BTreeSet<String> = decks.mine.iter().chain(&decks.pool).flat_map(|d| d.cards.iter().cloned()).collect();
    let oracle: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(data_dir().join("scryfall_cache.json")).unwrap()).unwrap();
    let allow: BTreeMap<_, _> = include_str!("enters_census_allow.txt")
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|l| l.split_once('|').expect("card | reason"))
        .map(|(n, r)| (n.trim(), r.trim()))
        .collect();
    let (template, sentinels) = census_fixture();
    let mut rows = vec!["card|oracle|path|opponent_diff|own_diff|own_effect_metrics_differ|inert_opponent_diff|own_vs_inert_diff|numeric_double|status|reason".to_string()];
    let (mut trig, mut setup, mut disagreements, mut own_non_double) = (0, 0, vec![], vec![]);
    for name in &names {
        if std::env::var("CENSUS_CARD").ok().is_some_and(|n| n != *name) {
            continue;
        }
        let v = &oracle[&name.to_lowercase()];
        assert!(!v.is_null(), "Oracle cache missing {name}");
        let is_trig = oracle_trig(&oracle_text(v));
        if is_trig {
            trig += 1;
        } else {
            setup += 1;
        }
        let cd = card(&template, name);
        let d = template.db.get(cd);
        let path = if d.land {
            "land"
        } else if v["type_line"].as_str().unwrap_or("").contains("Battle") {
            "battle"
        } else if d.perm {
            "permanent"
        } else {
            "spell-control"
        };
        let (mut states, mut metrics, mut outcomes) = (vec![], vec![], vec![]);
        for mode in 0..5 {
            let mut g = template.clone();
            if mode > 0 {
                let m = sentinels[if mode == 1 || mode == 3 { 1 } else { 0 }];
                let p = g.perm(m).owner;
                g.perm_mut(m).on_bf = true;
                g.perm_mut(m).neutered = mode >= 3;
                g.player_mut(p).perms.push(m);
                g.hooks.push(m);
            }
            g.last_x = 2;
            let target = template.player(P1).perms[1];
            let ctx =
                Ctx { x: 2, target: Some(if d.has_subtype("aura") { PermId(0) } else { target }), ..Ctx::default() };
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if name == "Wan Shi Tong, Librarian" {
                    g.player_mut(P0).hand.push(cd);
                    sim_core::impls::t6_legends2::wan_go(&mut g, P0, cd.0 as i64)
                } else if d.land {
                    turn::play_land_card(&mut g, P0, cd, "census").map(|_| true)
                } else if path == "battle" {
                    cast::resolve(&mut g, P0, cd, &ctx, "hand").map(|_| true)
                } else {
                    g.player_mut(P0).hand.push(cd);
                    cast::cast_card(&mut g, P0, cd, "hand", ctx)
                }
            }));
            outcomes.push(match result {
                Ok(Ok(ok)) => format!("ok:{ok}"),
                Ok(Err(e)) => format!("stop:{e:?}"),
                Err(_) => "PANIC".to_string(),
            });
            metrics.push(effects(&g, sentinels));
            states.push((snapshot(&g, sentinels, &template), g.rng.clone()));
        }
        let different = states[0] != states[1];
        let own = states[0] != states[2];
        let metric_diff = metrics[0] != metrics[2];
        let inert_opponent_diff = states[1] != states[3];
        let own_vs_inert_diff = states[2] != states[4];
        let effect_indices: Vec<_> = (0..metrics[0].len()).filter(|&k| metrics[0][k] != metrics[1][k]).collect();
        let numeric_double = if effect_indices.is_empty() {
            "unexercised"
        } else if effect_indices.iter().all(|&k| metrics[2][k] - metrics[0][k] == metrics[0][k] - metrics[1][k]) {
            "yes"
        } else {
            "no"
        };
        // a TRIG card must differ under a live Elesh and not under the same body with its abilities off (a body that
        // only changes an AI choice is not a blanked trigger)
        let valid =
            different == is_trig && (!is_trig || inert_opponent_diff) && outcomes.iter().all(|r| r == "ok:true");
        let reason = allow.get(name.as_str()).copied().unwrap_or("");
        if !valid {
            let pos = states[0].0.bytes().zip(states[1].0.bytes()).position(|(a, b)| a != b).unwrap_or(0);
            let lo = states[0].0.floor_char_boundary(pos.saturating_sub(50));
            let hi0 = states[0].0.floor_char_boundary((pos + 160).min(states[0].0.len()));
            let hi1 = states[1].0.floor_char_boundary((pos + 160).min(states[1].0.len()));
            rows.push(format!("DIFF|{name}|{}|{}", &states[0].0[lo..hi0], &states[1].0[lo..hi1]));
        }
        if !valid && reason.is_empty() {
            disagreements.push(format!(
                "{name}: {} opp_diff={different}, outcomes={outcomes:?}",
                if is_trig { "TRIG" } else { "SETUP" }
            ));
        }
        if is_trig && !metric_diff {
            own_non_double.push(name.clone());
        }
        rows.push(format!("{name}|{}|{path}|{different}|{own}|{metric_diff}|{inert_opponent_diff}|{own_vs_inert_diff}|{numeric_double}|{}|{reason}", if is_trig {"TRIG"} else {"SETUP"},outcomes.join(",")));
    }
    let report = format!(
        "cards={} TRIG={trig} SETUP={setup} allowlist={} unresolved={} own_no_effect_difference={}\n{}\n\nUNRESOLVED\n{}\n\nOWN_NO_EFFECT_DIFFERENCE\n{}\n",
        trig + setup,
        allow.len(),
        disagreements.len(),
        own_non_double.len(),
        rows.join("\n"),
        disagreements.join("\n"),
        own_non_double.join("\n")
    );
    if let Ok(path) = std::env::var("ENTERS_CENSUS_REPORT") {
        std::fs::write(path, &report).unwrap();
    }
    println!("{}", report.lines().next().unwrap());
    assert!(
        disagreements.is_empty(),
        "{} unresolved disagreements; set ENTERS_CENSUS_REPORT=<file> for the full table, and read the header of tests/enters_census.rs\n{}",
        disagreements.len(),
        disagreements.join("\n")
    );
}

#[test]
fn the_oracle_classifier_reads_the_controls() {
    for t in [
        "When this land enters, sacrifice it.",
        "Hideaway 4 (When this land enters, look at four cards.)",
        "Landfall — Whenever a land you control enters, draw a card.",
        "Whenever one or more creatures enter, you gain 1 life.",
        "When this creature enters with a counter on it, draw a card.",
    ] {
        assert!(oracle_trig(t), "{t}");
    }
    for t in [
        "As this land enters, choose a creature type.",
        "This artifact enters tapped.",
        "This creature enters with two +1/+1 counters on it.",
        "This creature enters prepared.",
        "Whenever you draw a card, create a token that enters tapped.",
        "(As this Saga enters and after your draw step, add a lore counter.)\nI — Create a Goblin.",
    ] {
        assert!(!oracle_trig(t), "{t}");
    }
}

// Own-source entry and observer entry paths share the nine controlled modes.
#[derive(Debug)]
struct DoubleOutcome {
    state: (String, sim_core::rng::Rng),
    metrics: Vec<i64>,
    outcome: String,
}
fn double_fixture() -> (Game, [PermId; 4]) {
    let (mut g, es) = census_fixture();
    let ds = ["Starfield Vocalist", "Panharmonicon"].map(|n| {
        let m = raw(&mut g, P0, n);
        g.perm_mut(m).eot_kw = vec!["shroud", "indestructible"];
        g.perm_mut(m).on_bf = false;
        g.player_mut(P0).perms.retain(|&x| x != m);
        g.hooks.retain(|&x| x != m);
        m
    });
    (g, [es[0], es[1], ds[0], ds[1]])
}
fn double_modes(
    template: &Game,
    sentinels: [PermId; 4],
    name: &str,
    observer: Option<&str>,
    battle: bool,
) -> Vec<DoubleOutcome> {
    let cd = card(template, name);
    (0..9)
        .map(|mode| {
            let mut g = template.clone();
            if mode > 0 {
                let (i, inert) = match mode {
                    1 => (1, false),
                    2 => (0, false),
                    3 => (1, true),
                    4 => (0, true),
                    5 => (2, false),
                    6 => (2, true),
                    7 => (3, false),
                    8 => (3, true),
                    _ => unreachable!(),
                };
                let m = sentinels[i];
                let p = g.perm(m).owner;
                g.perm_mut(m).on_bf = true;
                g.perm_mut(m).neutered = inert;
                g.player_mut(p).perms.push(m);
                g.hooks.push(m);
            }
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if let Some(entrant) = observer {
                    raw(&mut g, P0, name);
                    g.dsl_on |= g.db.get(cd).has_dsl();
                    match entrant {
                        "creature" => {
                            let c = card(&g, "Serra Angel");
                            g.player_mut(P0).hand.push(c);
                            cast::cast_card(&mut g, P0, c, "hand", Ctx::default())
                        }
                        "small-token" => zones::make_tokens(&mut g, P0, zones::Tokens::new(1, 1)).map(|_| true),
                        "big-token" => zones::make_tokens(&mut g, P0, zones::Tokens::new(1, 4)).map(|_| true),
                        "land" => {
                            let c = card(&g, "Forest");
                            turn::play_land_card(&mut g, P0, c, "census").map(|_| true)
                        }
                        "treasure" => zones::add_treasure(&mut g, P0, 1).map(|_| true),
                        _ => unreachable!(),
                    }
                } else {
                    g.last_x = 2;
                    let d = g.db.get(cd);
                    let ctx = Ctx {
                        x: 2,
                        target: Some(if d.has_subtype("aura") { PermId(0) } else { template.player(P1).perms[1] }),
                        ..Ctx::default()
                    };
                    if name == "Wan Shi Tong, Librarian" {
                        g.player_mut(P0).hand.push(cd);
                        sim_core::impls::t6_legends2::wan_go(&mut g, P0, cd.0 as i64)
                    } else if d.land {
                        turn::play_land_card(&mut g, P0, cd, "census").map(|_| true)
                    } else if battle {
                        cast::resolve(&mut g, P0, cd, &ctx, "hand").map(|_| true)
                    } else {
                        g.player_mut(P0).hand.push(cd);
                        cast::cast_card(&mut g, P0, cd, "hand", ctx)
                    }
                }
            }));
            let outcome = match result {
                Ok(Ok(ok)) => format!("ok:{ok}"),
                Ok(Err(e)) => format!("stop:{e:?}"),
                Err(_) => "PANIC".to_string(),
            };
            DoubleOutcome {
                state: (snapshot(&g, sentinels, template), g.rng.clone()),
                metrics: effects(&g, sentinels),
                outcome,
            }
        })
        .collect()
}
fn observer_trig(text: &str) -> bool {
    let text = text.to_lowercase();
    text.split(['.', '\n']).any(|clause| {
        ["when ", "whenever "].into_iter().any(|lead| {
            clause.match_indices(lead).any(|(i, _)| {
                let condition = clause[i + lead.len()..].split(',').next().unwrap_or("");
                condition.split_whitespace().any(|w| matches!(w, "enter" | "enters"))
                    && [
                        "another ",
                        "a creature",
                        "an artifact",
                        "a land",
                        "an enchantment",
                        "one or more",
                        "a permanent",
                        "a nontoken",
                        "an angel",
                        "a dragon",
                        "an equipment",
                        "a commander",
                    ]
                    .iter()
                    .any(|s| condition.contains(s))
            })
        })
    }) || text.contains("landfall")
}
fn pan_type(v: &serde_json::Value) -> bool {
    std::iter::once(v).chain(v["card_faces"].as_array().into_iter().flatten()).any(|f| {
        let t = f["type_line"].as_str().unwrap_or("");
        t.contains("Artifact") || t.contains("Creature")
    })
}

#[test]
fn every_enters_trigger_has_exactly_one_additional_instance_per_doubler() {
    let decks = sim_core::export::load_decks(&data_dir().join("decks.json")).unwrap();
    let names: BTreeSet<String> = decks.mine.iter().chain(&decks.pool).flat_map(|d| d.cards.iter().cloned()).collect();
    let oracle: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(data_dir().join("scryfall_cache.json")).unwrap()).unwrap();
    let old_allow: BTreeMap<_, _> = include_str!("enters_census_allow.txt")
        .lines()
        .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
        .map(|l| l.split_once('|').unwrap())
        .map(|(n, r)| (n.trim(), r.trim()))
        .collect();
    let mut allow = BTreeMap::new();
    for l in
        include_str!("enters_census_double_allow.txt").lines().filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
    {
        let (n, r) = l.split_once('|').expect("card | [category] reason");
        let (n, r) = (n.trim(), r.trim());
        let (cat, reason) = r.split_once(']').expect("[category] reason");
        assert!(
            ["[idempotent", "[exhausted", "[intervening-if", "[once-per-turn", "[nonlinear", "[fixture", "[deferred"]
                .contains(&cat)
                && !reason.trim().is_empty(),
            "invalid allowlist row: {l}"
        );
        assert!(names.contains(n), "unknown allowlist card: {n}");
        assert!(allow.insert(n, r).is_none(), "duplicate allowlist row: {n}");
    }
    let (template, sentinels) = double_fixture();
    let mut unresolved = vec![];
    let mut report = vec![];
    let mut needed = BTreeSet::new();
    for name in &names {
        if std::env::var("CENSUS_CARD").ok().is_some_and(|n| n != *name) {
            continue;
        }
        let v = &oracle[&name.to_lowercase()];
        assert!(!v.is_null(), "Oracle cache missing {name}");
        let text = oracle_text(v);
        let trig = oracle_trig(&text);
        let reason = allow.get(name.as_str()).copied().unwrap_or("");
        let mut cases = vec![None];
        if observer_trig(&text) {
            cases.extend(["creature", "small-token", "big-token", "land", "treasure"].map(Some));
        }
        let mut any_exercised = false;
        for case in cases {
            let runs = double_modes(
                &template,
                sentinels,
                name,
                case,
                v["type_line"].as_str().unwrap_or("").contains("Battle"),
            );
            let exercised = runs[0].state != runs[1].state && runs[1].state != runs[3].state;
            any_exercised |= exercised;
            let mut failures = vec![];
            let mut overs = vec![];
            if runs.iter().any(|r| r.outcome != "ok:true") {
                failures.push(format!("outcomes={:?}", runs.iter().map(|r| &r.outcome).collect::<Vec<_>>()));
            }
            for (d, live, inert) in [("Elesh", 2, 4), ("Vocalist", 5, 6), ("Panharmonicon", 7, 8)] {
                let applicable = d != "Panharmonicon" || case.map_or_else(|| pan_type(v), |c| c != "land");
                let changed = runs[live].state != runs[inert].state;
                if !applicable || !trig {
                    if changed {
                        failures.push(format!("{d} copies SETUP/ineligible entry"));
                    }
                } else if exercised {
                    if !changed {
                        failures.push(format!("{d} no additional effect"));
                    }
                    for k in 0..runs[0].metrics.len() {
                        let one = runs[0].metrics[k] - runs[1].metrics[k];
                        if one == 0 {
                            continue;
                        }
                        let extra = runs[live].metrics[k] - runs[0].metrics[k];
                        if extra != one {
                            failures.push(format!("{d} metric {k}: one={one}, extra={extra}"));
                        }
                        if extra.abs() > one.abs() {
                            overs.push(format!("OVER {d} metric {k}: one={one}, extra={extra}"));
                        }
                    }
                }
            }
            if !failures.is_empty() {
                needed.insert(name.as_str());
                if reason.is_empty() {
                    unresolved.push(format!("{name} | {} | {}", case.unwrap_or("own-entry"), failures.join("; ")));
                }
            }
            if !overs.is_empty() && !reason.starts_with("[nonlinear]") {
                unresolved.push(format!("{name} | {} | {}", case.unwrap_or("own-entry"), overs.join("; ")));
            }
            report.push(format!(
                "{name}|{}|exercised={exercised}|{}|{reason}",
                case.unwrap_or("own-entry"),
                failures.join("; ")
            ));
        }
        if trig && !any_exercised {
            needed.insert(name.as_str());
            let old = old_allow
                .get(name.as_str())
                .copied()
                .unwrap_or("no exercised enters clause in the standard entrant set");
            if reason.is_empty() {
                unresolved.push(format!("{name} | unexercised | {old}"));
            }
        }
    }
    if std::env::var("CENSUS_CARD").is_err() {
        for (&name, &reason) in &allow {
            if !needed.contains(name) && !reason.starts_with("[fixture]") {
                unresolved.push(format!("STALE {name} | {reason}"));
            }
        }
    }
    if let Ok(path) = std::env::var("ENTERS_CENSUS_DOUBLE_REPORT") {
        std::fs::write(path, format!("{}\n\nUNRESOLVED\n{}\n", report.join("\n"), unresolved.join("\n"))).unwrap();
    }
    assert!(unresolved.is_empty(), "{} unexplained doubling rows:\n{}", unresolved.len(), unresolved.join("\n"));
}
