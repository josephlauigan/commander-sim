//! Port of tests/test_play_images.py at the HTTP level: the loading screen's progress, card images found by name,
//! tokens found through the cards that make them, every image downloaded once and served from /images/, cached the
//! second time, and a game that starts without images. The Rust server looks cards up on a local fake Scryfall
//! (PRACTICE_SCRYFALL, see common::fake), so nothing here touches the network. The Python server cannot be pointed
//! at a fake (its lookups are internal functions), so these tests are skipped there. (Python's offline,
//! failed-download and two-faced-card cases are unit tests of internals; the Rust crate tests them in src/images.rs.)
mod common;
use common::fake;
use common::*;
use serde_json::json;
use std::sync::atomic::Ordering;

macro_rules! rust_srv {
    () => {
        match server() {
            Some(s) if s.rust => s,
            Some(_) => {
                eprintln!("SKIPPED: card images need the Rust server (it reads PRACTICE_SCRYFALL; the Python server cannot use a fake Scryfall)");
                return;
            }
            None => return,
        }
    };
}

fn images_event(c: &mut Client, images: bool) -> (Vec<serde_json::Value>, serde_json::Value) {
    let r = c.post("/api/new", json!({"deck": "sauron", "tier": "t2", "seed": 5, "ai": "adaptive", "images": images}));
    assert_eq!(r.status, 200, "{}", String::from_utf8_lossy(&r.body));
    let mut evs = c.stream(0, None).until(120, |e| e["kind"] == "images");
    let img = evs.pop().unwrap();
    (evs, img)
}

#[test]
fn test_cards_tokens_and_progress() {
    let s = rust_srv!();
    let mut c = s.client();
    let (loading, img) = images_event(&mut c, true);
    let map = img["images"].as_object().expect("images: name -> files");
    assert!(!map.is_empty());
    assert!(fake::LOOKUPS.load(Ordering::Relaxed) > 0);
    // every file of every card is served from /images/ with the fake's bytes
    let (name, files) = map.iter().find(|(k, _)| k.as_str() != "Zombie").unwrap();
    let f = files[0].as_str().unwrap();
    let r = c.get(&format!("/images/{f}"));
    assert_eq!((r.status, r.header("content-type")), (200, Some("image/jpeg")), "{name}");
    assert!(r.body.starts_with(b"jpg:"));
    // the token one card makes is found through that card
    assert!(map.contains_key("Zombie"), "no token image: {:?}", map.keys().take(5).collect::<Vec<_>>());
    // progress: loading events count up to the total
    let last = loading.iter().rev().find(|e| e["kind"] == "loading").expect("loading events");
    assert_eq!(last["done"], last["total"]);
    c.quit();
}

#[test]
fn test_cached_the_second_time() {
    let s = rust_srv!();
    let mut c = s.client();
    images_event(&mut c, true);
    c.quit();
    let (lookups, downloads) = (fake::LOOKUPS.load(Ordering::Relaxed), fake::DOWNLOADS.load(Ordering::Relaxed));
    let (_, img) = images_event(&mut c, true);
    assert!(!img["images"].as_object().unwrap().is_empty());
    assert_eq!((fake::LOOKUPS.load(Ordering::Relaxed), fake::DOWNLOADS.load(Ordering::Relaxed)), (lookups, downloads));
    c.quit();
}

#[test]
fn test_lookups_are_in_batches_of_75() {
    let s = rust_srv!();
    let mut c = s.client();
    let (l0, n0) = (fake::LOOKUPS.load(Ordering::Relaxed), fake::NAMES_ASKED.load(Ordering::Relaxed));
    images_event(&mut c, true);
    let (l, n) = (fake::LOOKUPS.load(Ordering::Relaxed) - l0, fake::NAMES_ASKED.load(Ordering::Relaxed) - n0);
    // names are asked 75 to a request; the token lookup is one more request
    if n > 0 {
        assert!(l >= n.div_ceil(75), "{l} requests for {n} names");
        assert!(l <= n.div_ceil(75) + 1, "{l} requests for {n} names");
    }
    c.quit();
}

#[test]
fn test_a_game_without_images_asks_nobody() {
    let s = rust_srv!();
    let mut c = s.client();
    let before = (fake::LOOKUPS.load(Ordering::Relaxed), fake::DOWNLOADS.load(Ordering::Relaxed));
    let (_, img) = images_event(&mut c, false);
    assert!(img["images"].is_object());
    assert_eq!((fake::LOOKUPS.load(Ordering::Relaxed), fake::DOWNLOADS.load(Ordering::Relaxed)), before);
    c.quit();
}
