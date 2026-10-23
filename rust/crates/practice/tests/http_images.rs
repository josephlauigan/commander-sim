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

/// The lookup and download counts once the server is quiet. `server()` opens the setup screen (`/api/options`), and
/// with a fresh image folder that starts a background download of the commanders' pictures; its lookups must not be
/// counted against the game a test then starts. Quiet = no change for a second.
fn settled() -> (usize, usize) {
    let read = || (fake::LOOKUPS.load(Ordering::Relaxed), fake::DOWNLOADS.load(Ordering::Relaxed));
    let (mut last, mut still) = (read(), 0);
    while still < 10 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        let now = read();
        (last, still) = if now == last { (last, still + 1) } else { (now, 0) };
    }
    last
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
    // your deck's own printings (data/printings/sauron.txt): every copy by set and collector number
    let own = img["printed"]["sauron"].as_object().expect("Sauron's printings");
    let swamps = own["Swamp"].as_array().expect("Swamp copies");
    assert_eq!(swamps.len(), 6);
    assert!(swamps[0][0].as_str().unwrap().starts_with("print-2ed-"), "{swamps:?}");
    let f = own["Sol Ring"][0][0].as_str().unwrap();
    assert_eq!(c.get(&format!("/images/{f}")).status, 200);
    c.quit();
}

#[test]
fn test_cached_the_second_time() {
    let s = rust_srv!();
    let mut c = s.client();
    images_event(&mut c, true);
    c.quit();
    let (lookups, downloads) = settled();
    let (_, img) = images_event(&mut c, true);
    assert!(!img["images"].as_object().unwrap().is_empty());
    assert_eq!((fake::LOOKUPS.load(Ordering::Relaxed), fake::DOWNLOADS.load(Ordering::Relaxed)), (lookups, downloads));
    c.quit();
}

#[test]
fn test_lookups_are_in_batches_of_75() {
    let s = rust_srv!();
    let mut c = s.client();
    let asked = || {
        let o = Ordering::Relaxed;
        (fake::LOOKUPS.load(o), fake::NAMES_ASKED.load(o), fake::PRINTS_ASKED.load(o))
    };
    settled();
    let (l0, n0, p0) = asked();
    images_event(&mut c, true);
    let (l, n, p) = asked();
    let (l, n, p) = (l - l0, n - n0, p - p0);
    // names and printings are asked 75 to a request (separately); the token lookup is one more request
    if n + p > 0 {
        let batches = n.div_ceil(75) + p.div_ceil(75);
        assert!(l >= batches, "{l} requests for {n} names and {p} printings");
        assert!(l <= batches + 1, "{l} requests for {n} names and {p} printings");
    }
    c.quit();
}

#[test]
fn test_a_game_without_images_asks_nobody() {
    let s = rust_srv!();
    let mut c = s.client();
    let before = settled();
    let (_, img) = images_event(&mut c, false);
    assert!(img["images"].is_object());
    assert_eq!((fake::LOOKUPS.load(Ordering::Relaxed), fake::DOWNLOADS.load(Ordering::Relaxed)), before);
    c.quit();
}
