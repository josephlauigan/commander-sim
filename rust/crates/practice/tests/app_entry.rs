//! The iPad app's entry points (`practice_start` and friends, via `start_app`) and the first-launch data copy, on
//! temporary folders: the bundle is built from the repo's data, as `ios-shell/build-rust.sh` does.
mod common;
use common::*;
use practice::{bundle::sync_data, set_lan_app, start_app, stop_app};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

static ONE_AT_A_TIME: Mutex<()> = Mutex::new(());

fn lock() -> MutexGuard<'static, ()> {
    ONE_AT_A_TIME.lock().unwrap_or_else(|e| e.into_inner())
}

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("practice-app-{}-{name}", std::process::id()));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

/// a bundle like the app's Resources: data/ (with a stamp), decklists/, static/ (symlinks to the repo's files)
fn bundle(name: &str, stamp: &str) -> PathBuf {
    let b = temp(name);
    let (root, data) = (repo(), b.join("data"));
    fs::create_dir_all(data.join("printings")).unwrap();
    for f in ["cards.json", "decks.json", "my_decks.toml", "scryfall_cache.json"] {
        std::os::unix::fs::symlink(root.join("data").join(f), data.join(f)).unwrap();
    }
    for e in fs::read_dir(root.join("data/printings")).unwrap().flatten() {
        std::os::unix::fs::symlink(e.path(), data.join("printings").join(e.file_name())).unwrap();
    }
    std::os::unix::fs::symlink(root.join("decklists"), b.join("decklists")).unwrap();
    std::os::unix::fs::symlink(root.join("rust/crates/practice/static"), b.join("static")).unwrap();
    fs::write(data.join("bundle.json"), json!({"commit": stamp}).to_string()).unwrap();
    b
}

fn json_of(p: &Path) -> Value {
    serde_json::from_slice(&fs::read(p).unwrap()).unwrap()
}

#[test]
fn test_first_launch_copies_then_the_same_stamp_does_not() {
    let (b, app) = (bundle("b1", "one"), temp("a1"));
    assert!(sync_data(&b, &app).unwrap());
    for f in ["cards.json", "decks.json", "my_decks.toml", "scryfall_cache.json", "bundle.json", "printings/sauron.txt"]
    {
        assert!(app.join("data").join(f).exists(), "{f}");
    }
    assert!(app.join("decklists/pool/pool-results.md").exists());
    fs::write(app.join("data/cards.json"), "mine").unwrap();
    assert!(!sync_data(&b, &app).unwrap());
    assert_eq!(fs::read_to_string(app.join("data/cards.json")).unwrap(), "mine"); // not copied again
}

#[test]
fn test_a_new_stamp_replaces_files_keeps_images_and_merges_the_index_and_never_touches_saves() {
    let (b, app) = (bundle("b2", "one"), temp("a2"));
    sync_data(&b, &app).unwrap();
    // images downloaded since, an index entry for one of them, a saved game
    let images = app.join("data/images");
    fs::create_dir_all(&images).unwrap();
    fs::write(images.join("downloaded.jpg"), "d").unwrap();
    fs::write(images.join("index.json"), r#"{"old":"downloaded.jpg"}"#).unwrap();
    fs::create_dir_all(app.join("saves")).unwrap();
    fs::write(app.join("saves/autosave.json"), "{}").unwrap();
    fs::write(app.join("data/cards.json"), "stale").unwrap();
    fs::write(app.join("data/printings/sauron.txt"), "stale").unwrap();
    // the new build brings an image and an index entry (and one the iPad already has)
    fs::create_dir_all(b.join("data/images")).unwrap();
    fs::write(b.join("data/images/bundled.jpg"), "b").unwrap();
    fs::write(b.join("data/images/downloaded.jpg"), "from the bundle").unwrap();
    fs::write(b.join("data/images/index.json"), r#"{"new":"bundled.jpg"}"#).unwrap();
    fs::write(b.join("data/bundle.json"), r#"{"commit":"two"}"#).unwrap();
    assert!(sync_data(&b, &app).unwrap());
    assert_ne!(fs::read_to_string(app.join("data/cards.json")).unwrap(), "stale");
    assert_ne!(fs::read_to_string(app.join("data/printings/sauron.txt")).unwrap(), "stale");
    assert_eq!(fs::read_to_string(images.join("downloaded.jpg")).unwrap(), "d"); // images are kept
    assert_eq!(fs::read_to_string(images.join("bundled.jpg")).unwrap(), "b");
    assert_eq!(json_of(&images.join("index.json")), json!({"old": "downloaded.jpg", "new": "bundled.jpg"}));
    assert_eq!(fs::read_to_string(app.join("saves/autosave.json")).unwrap(), "{}");
    assert_eq!(json_of(&app.join("data/bundle.json"))["commit"], "two");
}

#[test]
fn test_the_started_app_serves_the_page_and_the_catalog() {
    let _g = lock();
    unsafe { std::env::set_var("PRACTICE_NO_FETCH", "1") };
    let (b, app) = (bundle("b3", "one"), temp("a3"));
    let port = start_app(&b, &app, 0).unwrap();
    assert_ne!(port, 0);
    let mut c = Client::new(&format!("127.0.0.1:{port}"));
    let page = c.get("/");
    assert_eq!(page.status, 200);
    assert_eq!(page.body, fs::read(repo().join("rust/crates/practice/static/index.html")).unwrap());
    let o = c.get("/api/options").json();
    assert_eq!((o["app"].clone(), o["lan"].clone()), (json!(true), json!(false)));
    let cat = o.clone(); // the options carry the catalog
    assert_eq!(cat["decks"].as_array().unwrap().len(), 14); // JD's 12, Godo (Tier 6) and Tymna/Thrasios
    assert!(cat["decks"][0]["name"].as_str().is_some_and(|n| !n.is_empty()));
    assert!(stop_app());
    assert!(!stop_app());
}

#[test]
fn test_an_autosaved_game_is_there_to_continue_after_the_app_is_closed_and_reopened() {
    let _g = lock();
    unsafe { std::env::set_var("PRACTICE_NO_FETCH", "1") };
    let (b, app) = (bundle("b4", "one"), temp("a4"));
    let addr = |p: u16| format!("127.0.0.1:{p}");
    let mut c = Client::new(&addr(start_app(&b, &app, 0).unwrap()));
    assert!(c.get("/api/saves").json()["autosave"].is_null());
    assert_eq!(c.new_game("sauron", "t2", 4).status, 200);
    let mull = wait_for(&mut c, 0, is_request);
    c.answer(mull["id"].as_u64().unwrap(), json!("keep"));
    wait_for(&mut c, mull["id"].as_u64().unwrap(), is_request);
    stop_app(); // iOS closes the app
    assert!(app.join("saves/autosave.json").exists());
    let mut c = Client::new(&addr(start_app(&b, &app, 0).unwrap())); // reopened
    let auto = c.get("/api/saves").json()["autosave"].clone();
    assert_eq!((auto["deck"].as_str(), auto["seed"].as_u64()), (Some("sauron"), Some(4)));
    assert_eq!(c.post("/api/load", json!({"name": "autosave.json", "images": false})).status, 200);
    wait_for(&mut c, 0, is_request); // Continue: back at a decision
    c.quit();
    stop_app();
}

#[test]
fn test_set_lan_restarts_the_server_on_the_network_and_back() {
    let _g = lock();
    unsafe { std::env::set_var("PRACTICE_NO_FETCH", "1") };
    assert!(set_lan_app(true).is_err()); // nothing running
    let (b, app) = (bundle("b5", "one"), temp("a5"));
    start_app(&b, &app, 0).unwrap();
    let p = set_lan_app(true).unwrap();
    let mut c = Client::new(&format!("127.0.0.1:{p}"));
    assert_eq!(c.get("/api/options").json()["lan"], true);
    assert_eq!(set_lan_app(true).unwrap(), p); // already so
    let p = set_lan_app(false).unwrap();
    let mut c = Client::new(&format!("127.0.0.1:{p}"));
    assert_eq!(c.get("/api/options").json()["lan"], false);
    stop_app();
}
