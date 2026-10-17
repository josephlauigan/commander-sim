//! A person playing the Partner deck `kraum` (Tymna the Weaver + Kraum, Ludevic's Opus): the setup list shows both
//! commanders, the view gives both zones and taxes, and the commanders are cast from the command zone with
//! `{"do": "cast", "zone": "cmd", "cmd": i}`.

#[path = "session_common/mod.rs"]
mod common;
use common::*;
use practice::session::Session;
use serde_json::{Value, json};
use std::cell::Cell;
use std::time::Duration;

const LIMIT: Duration = Duration::from_secs(120);

#[test]
fn the_catalog_lists_kraum_with_both_commanders() {
    let data = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");
    let cat = practice::catalog::catalog(&data).unwrap();
    let keys: Vec<&str> = cat["decks"].as_array().unwrap().iter().map(|d| d["key"].as_str().unwrap()).collect();
    assert!(keys.contains(&"kraum"), "{keys:?}");
    let k = cat["decks"].as_array().unwrap().iter().find(|d| d["key"] == "kraum").unwrap();
    assert_eq!(k["commanders"], json!(["Tymna the Weaver", "Kraum, Ludevic's Opus"]));
    assert_eq!(
        (k["commander"].as_str(), k["commander2"].as_str()),
        (Some("Tymna the Weaver"), Some("Kraum, Ludevic's Opus"))
    );
    assert_eq!(k["ident"], "WUBR");
    assert_eq!(k["name"], "Tymna the Weaver and Kraum"); // the deck file's two commanders
    // one commander elsewhere: no second one
    let s = cat["decks"].as_array().unwrap().iter().find(|d| d["key"] == "sauron").unwrap();
    assert_eq!(s["commander2"], Value::Null);
    assert_eq!(s["commanders"].as_array().unwrap().len(), 1);
    // both commanders get their images fetched
    let names = practice::catalog::commanders(&cat);
    assert!(names.contains(&"Tymna the Weaver".to_string()) && names.contains(&"Kraum, Ludevic's Opus".to_string()));
}

#[test]
fn the_deck_file_line_with_two_commanders() {
    assert_eq!(
        practice::catalog::display_name("x\n**Commanders (2).** Tymna the Weaver, Kraum, Ludevic's Opus\n").unwrap(),
        "Tymna the Weaver and Kraum"
    );
}

fn me(req: &Value) -> Option<&Value> {
    req["data"]["view"]["players"].as_array()?.iter().find(|p| p["you"] == true)
}

/// the person's turns: play a land, tap everything, then cast a commander the mana in the pool pays for.
/// One game of that policy with this seed: the casts of the person's commanders from the log, and the taxes at the end
fn play(seed: u64) -> (Vec<String>, u64, u64) {
    let mut o = opts("adaptive");
    o.deck = "kraum".into();
    o.tier = "t3".into();
    o.seed = Some(seed);
    o.max_rounds = 14;
    o.compare = false;
    let s = Session::new(o).unwrap();
    s.start();
    let (round, landed, tried) = (Cell::new(0u64), Cell::new(false), Cell::new(false));
    let seen_first_view = Cell::new(false);
    let policy = |req: &Value, _n: usize| -> Option<Value> {
        if req["kind"] != "priority" {
            return None;
        }
        let m = me(req)?;
        if !seen_first_view.get() {
            // the view has both commanders, both in the zone, no tax yet
            assert_eq!(m["commander"], "Tymna the Weaver");
            assert_eq!(m["commander2"], "Kraum, Ludevic's Opus");
            assert!(m["commander_in_zone"] == true && m["commander2_in_zone"] == true);
            assert_eq!((m["tax"].as_u64(), m["tax2"].as_u64()), (Some(0), Some(0)));
            seen_first_view.set(true);
        }
        let r = req["data"]["view"]["round"].as_u64().unwrap();
        if r != round.get() {
            round.set(r);
            landed.set(false);
            tried.set(false);
        }
        let in_zone = |k: &str| m[k] == true;
        if !landed.get()
            && let Some(i) = m["hand_land"].as_array().unwrap().iter().position(|l| l == true)
        {
            landed.set(true);
            return Some(json!({"do": "land", "card": i}));
        }
        if (in_zone("commander_in_zone") || in_zone("commander2_in_zone"))
            && let Some(src) = m["mana_sources"].as_array().and_then(|a| a.first())
        {
            return Some(json!({"do": "tap", "source": src["id"]}));
        }
        let pool = m["mana_pool"].as_str().unwrap_or("").chars().count() as u64;
        if !tried.get() && pool > 0 {
            let (t1, t2) = (3 + m["tax"].as_u64().unwrap(), 5 + m["tax2"].as_u64().unwrap());
            let has = |cs: &str| cs.chars().all(|c| m["mana_pool"].as_str().unwrap_or("").contains(c));
            if in_zone("commander2_in_zone") && pool >= t2 && has("UR") {
                tried.set(true);
                return Some(json!({"do": "cast", "zone": "cmd", "cmd": 1}));
            }
            if in_zone("commander_in_zone") && pool >= t1 && has("WB") {
                tried.set(true);
                return Some(json!({"do": "cast", "zone": "cmd", "cmd": 0}));
            }
        }
        Some(json!({"do": "pass"}))
    };
    let (_, evs) = drive(&s, 0, LIMIT, &policy);
    // the last table the person saw shows the taxes
    let last = over(&evs)["view"]["players"].as_array().unwrap().iter().find(|p| p["you"] == true).unwrap().clone();
    let cast: Vec<String> = evs
        .iter()
        .filter_map(|e| e["text"].as_str())
        .filter(|t| t.contains("Tymna/Kraum casts Tymna the Weaver") || t.contains("Tymna/Kraum casts Kraum"))
        .map(String::from)
        .collect();
    s.close(Duration::from_secs(10));
    (cast, last["tax"].as_u64().unwrap(), last["tax2"].as_u64().unwrap())
}

#[test]
fn a_person_on_kraum_casts_either_commander_from_the_command_zone_and_pays_its_own_tax() {
    let (mut tymna_recast, mut kraum_seen) = (false, false);
    for seed in 1..=24 {
        let (cast, tax, tax2) = play(seed);
        let of = |n: &str| cast.iter().filter(|t| t.contains(&format!("Tymna/Kraum casts {n}"))).count() as u64;
        // every cast raised that commander's own tax, and only its own
        assert_eq!(tax, 2 * of("Tymna the Weaver"), "seed {seed}: {cast:?}");
        assert_eq!(tax2, 2 * of("Kraum"), "seed {seed}: {cast:?}");
        tymna_recast |= of("Tymna the Weaver") >= 2;
        kraum_seen |= of("Kraum") >= 1;
    }
    assert!(tymna_recast, "no game recast Tymna");
    assert!(kraum_seen, "no game cast Kraum");
}

#[test]
fn the_list_of_playable_decks_is_every_deck_of_decks_json() {
    let data = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");
    let d: Value = serde_json::from_str(&std::fs::read_to_string(data.join("decks.json")).unwrap()).unwrap();
    let mut mine: Vec<&str> = d["mine"].as_array().unwrap().iter().map(|x| x["key"].as_str().unwrap()).collect();
    mine.sort();
    let mut listed = practice::session::MY_DECKS.to_vec();
    listed.sort();
    assert_eq!(listed, mine);
    let mut cat = practice::catalog::MY_DECKS.to_vec();
    cat.sort();
    assert_eq!(cat, mine);
}
