//! `cardgen bulk-sync` on a 44-card fixture in Scryfall's bulk shape (every layout, a banned card, an un-card, null
//! prices and edhrec rank, a foil-only price, a duplicate oracle_id). Nothing here uses the network: the sync runs on
//! `--from-file`, or against a `Remote` that copies the fixture.

use cardgen::bulk::{self, Entry, Meta, Options, Remote};
use cardgen::json::{self, J};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

const SYNCED: &str = "2026-11-21T00:00:00Z";

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/bulk_fixture.json")
}

fn fixture_cards() -> Vec<J> {
    match json::parse(&std::fs::read_to_string(fixture()).unwrap()).unwrap() {
        J::Arr(a) => a,
        _ => panic!("fixture is not an array"),
    }
}

/// a temporary directory, removed on drop
struct Tmp(PathBuf);

impl Tmp {
    fn new() -> Tmp {
        static N: AtomicUsize = AtomicUsize::new(0);
        let p = std::env::temp_dir().join(format!(
            "bulk-sync-test-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&p).unwrap();
        Tmp(p)
    }
    fn join(&self, f: &str) -> PathBuf {
        self.0.join(f)
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn opts(dir: &Path, from_file: Option<PathBuf>, min_count: usize, synced: &str) -> Options {
    Options { dir: dir.to_path_buf(), from_file, min_count, synced: synced.to_string() }
}

/// a sync of the fixture with `--from-file` (no remote is consulted)
fn sync_fixture(dir: &Path, synced: &str) -> bulk::Outcome {
    bulk::run(&opts(dir, Some(fixture()), 1, synced), &mut Panics).unwrap()
}

struct Panics;

impl Remote for Panics {
    fn latest(&mut self) -> Result<Entry, String> {
        panic!("--from-file must not ask the remote")
    }
    fn download(&mut self, _: &str, _: &Path) -> Result<(), String> {
        panic!("--from-file must not download")
    }
}

struct Mock {
    updated_at: String,
    src: PathBuf,
    downloads: usize,
    fail_download: bool,
}

impl Mock {
    fn new(updated_at: &str) -> Mock {
        Mock { updated_at: updated_at.into(), src: fixture(), downloads: 0, fail_download: false }
    }
}

impl Remote for Mock {
    fn latest(&mut self) -> Result<Entry, String> {
        Ok(Entry { updated_at: self.updated_at.clone(), download_uri: "https://example.invalid/oracle.json".into() })
    }
    fn download(&mut self, _: &str, dest: &Path) -> Result<(), String> {
        self.downloads += 1;
        if self.fail_download {
            std::fs::write(dest, b"[{\"name\": \"half").unwrap();
            return Err("connection reset".into());
        }
        std::fs::copy(&self.src, dest).map(|_| ()).map_err(|e| e.to_string())
    }
}

fn read_oracle(dir: &Path) -> J {
    json::parse(&std::fs::read_to_string(dir.join("oracle.json")).unwrap()).unwrap()
}

fn oracle_cards(dir: &Path) -> Vec<J> {
    read_oracle(dir).get("cards").unwrap().as_arr().unwrap().clone()
}

fn find<'a>(cards: &'a [J], name: &str) -> &'a J {
    cards.iter().find(|c| c.s("name") == Some(name)).unwrap_or_else(|| panic!("{name} not in oracle.json"))
}

fn no_tmp_files(dir: &Path) {
    for e in std::fs::read_dir(dir).unwrap() {
        let n = e.unwrap().file_name().to_string_lossy().to_string();
        assert!(!n.ends_with(".tmp"), "left a temp file: {n}");
    }
}

// ------------------------------------------------------------------ slim_bulk

/// the 15 cache fields are `slim`'s, in the same order and with the same values; faces differ only by `colors`
#[test]
fn slim_bulk_agrees_with_slim_on_the_cache_fields() {
    let cards = fixture_cards();
    assert_eq!(cards.len(), 44);
    for c in &cards {
        let (s, b) = (cardgen::scryfall::slim(c), bulk::slim_bulk(c));
        let (s, b) = (s.as_obj().unwrap(), b.as_obj().unwrap());
        let name = c.s_or("name");
        assert_eq!(b.keys().take(s.len()).collect::<Vec<_>>(), s.keys().collect::<Vec<_>>(), "key order, {name}");
        for (k, v) in s.iter() {
            let got = b.get(k).unwrap();
            if k == "card_faces" {
                let mut faces = got.clone();
                for f in match &mut faces {
                    J::Arr(a) => a.iter_mut(),
                    _ => panic!("card_faces is not an array"),
                } {
                    let o = f.as_obj_mut().unwrap();
                    o.0.retain(|(k, _)| k != "colors");
                }
                assert_eq!(&faces, v, "card_faces, {name}");
            } else {
                assert_eq!(got, v, "{k}, {name}");
            }
        }
    }
}

#[test]
fn slim_bulk_extras() {
    let cards = fixture_cards();
    let slim = |n: &str| bulk::slim_bulk(cards.iter().find(|c| c.s("name") == Some(n)).unwrap());
    let sol = slim("Sol Ring");
    assert_eq!(sol.s("oracle_id"), Some("11111111-1111-4111-8111-111111111111"));
    assert_eq!(sol.get("edhrec_rank"), Some(&J::Int(1)));
    assert_eq!(sol.s("usd"), Some("1.50"));
    assert_eq!(
        (sol.s("set"), sol.s("rarity"), sol.s("released_at")),
        (Some("xyz"), Some("uncommon"), Some("2020-01-01"))
    );
    assert_eq!(sol.get("reserved"), Some(&J::Bool(false)));
    assert!(sol.get("all_parts_meld").is_none());
    // fields the engine does not use stay out
    for k in ["image_uris", "prices", "multiverse_ids", "uri", "digital", "set_name"] {
        assert!(sol.get(k).is_none(), "{k}");
    }
    // null edhrec_rank and null prices: absent, not null
    let forest = slim("Forest");
    assert!(forest.get("edhrec_rank").is_none());
    let lotus = slim("Black Lotus");
    assert!(lotus.get("edhrec_rank").is_none() && lotus.get("usd").is_none());
    assert_eq!(lotus.get("reserved"), Some(&J::Bool(true)));
    // usd_foil only when usd is null
    assert_eq!(slim("Mystery Foil Only").s("usd"), Some("1.23"));
    // only the meld result is flagged
    assert_eq!(slim("Brisela, Voice of Nightmares").get("all_parts_meld"), Some(&J::Bool(true)));
    assert!(slim("Bruna, the Fading Light").get("all_parts_meld").is_none());
    assert!(slim("Gisela, the Broken Blade").get("all_parts_meld").is_none());
    // faces gain their colours
    let delver = slim("Delver of Secrets // Insectile Aberration");
    let faces = delver.get("card_faces").unwrap().as_arr().unwrap();
    assert_eq!(faces.len(), 2);
    assert_eq!(faces[0].get("colors"), Some(&json::parse(r#"["U"]"#).unwrap()));
    assert_eq!(faces[1].s("name"), Some("Insectile Aberration"));
    // a colourless face keeps its empty colour list
    let mdfc = slim("Shatterskull Smashing // Shatterskull, the Hammer Pass");
    assert_eq!(mdfc.get("card_faces").unwrap().as_arr().unwrap()[1].get("colors"), Some(&J::Arr(vec![])));
}

// ------------------------------------------------------------------ reading: drops, layouts, warnings

#[test]
fn drop_list_dedupe_and_layout_counts() {
    let s = bulk::read_bulk(&fixture()).unwrap();
    assert_eq!((s.read, s.kept(), s.dropped()), (44, 34, 10));
    assert_eq!((s.dropped_layout, s.dropped_digital, s.dropped_duplicate), (8, 1, 1));
    let n = |l: &str| (s.layouts[l].kept, s.layouts[l].dropped);
    assert_eq!(n("normal"), (18, 2)); // 20 records: a duplicate oracle_id and a digital-only card are dropped
    for l in ["transform", "modal_dfc", "split", "adventure"] {
        assert_eq!(n(l), (2, 0), "{l}");
    }
    for (l, kept) in [("flip", 1), ("meld", 3), ("saga", 1), ("class", 1), ("leveler", 1), ("foobar_layout", 1)] {
        assert_eq!(n(l), (kept, 0), "{l}");
    }
    assert_eq!(n("token"), (0, 2));
    for l in ["double_faced_token", "emblem", "art_series", "scheme", "planar", "vanguard"] {
        assert_eq!(n(l), (0, 1), "{l}");
    }
    let names: Vec<&str> = s.cards.iter().map(|(_, c)| c.s_or("name")).collect();
    assert!(names.contains(&"Sol Ring") && !names.contains(&"Duplicate Oracle Copy"), "first oracle_id wins");
    assert!(!names.contains(&"A-Mother of Runes") && !names.contains(&"Goblin"));
    // banned, not-legal and un-cards stay, with their legality
    let legal = |n: &str| {
        let c = &s.cards.iter().find(|(_, c)| c.s("name") == Some(n)).unwrap().1;
        c.get("legalities").unwrap().s("commander").unwrap().to_string()
    };
    assert_eq!(legal("Primeval Titan"), "banned");
    assert_eq!(legal("Black Lotus"), "banned");
    assert_eq!(legal("Blast from the Past"), "not_legal");
    assert_eq!(legal("Sol Ring"), "legal");
    assert_eq!(s.unknown_layouts(), ["foobar_layout"]);
}

#[test]
fn unknown_layout_warns_and_keeps_the_card() {
    let t = Tmp::new();
    let out = sync_fixture(&t.0, SYNCED);
    assert_eq!(out.warnings.len(), 1, "{:?}", out.warnings);
    assert!(out.warnings[0].contains("foobar_layout"));
    assert!(out.text.contains("foobar_layout"), "the summary counts it");
    let cards = oracle_cards(&t.0);
    assert_eq!(find(&cards, "Mystery Layout Card").s("layout"), Some("foobar_layout"));
    // nothing in the output was dropped by a layout the drop list does not name
    assert!(cards.iter().all(|c| !bulk::DROP_LAYOUTS.contains(&c.s_or("layout"))));
    assert_eq!(cards.iter().filter(|c| c.s("layout") == Some("token")).count(), 0);
}

#[test]
fn summary_lists_counts_per_layout() {
    let t = Tmp::new();
    let out = sync_fixture(&t.0, SYNCED);
    assert!(out.text.contains("bulk updated_at: from-file"));
    for (l, k, d) in [("normal", 18, 2), ("token", 0, 2), ("meld", 3, 0)] {
        let line = format!("{l:<22}{k:>8}{d:>9}");
        assert!(out.text.contains(&line), "missing {line:?} in\n{}", out.text);
    }
    assert!(
        out.text.contains("read 44, kept 34, dropped 10 (layout 8, digital 1, duplicate oracle_id 1)"),
        "{}",
        out.text
    );
}

// ------------------------------------------------------------------ writing

#[test]
fn output_is_sorted_complete_and_deterministic() {
    let (a, b, c) = (Tmp::new(), Tmp::new(), Tmp::new());
    sync_fixture(&a.0, SYNCED);
    sync_fixture(&b.0, SYNCED);
    sync_fixture(&c.0, "2030-01-01T00:00:00Z");
    let (ta, tb, tc) = (read(&a, "oracle.json"), read(&b, "oracle.json"), read(&c, "oracle.json"));
    assert_eq!(ta, tb, "same input, same bytes");
    assert_eq!(read(&a, "sync.lock"), read(&c, "sync.lock"), "the lock does not carry the sync time");
    // only meta.synced differs
    assert_eq!(ta.replace(SYNCED, "X"), tc.replace("2030-01-01T00:00:00Z", "X"));
    assert_ne!(ta, tc);
    let v = read_oracle(&a.0);
    let meta = v.get("meta").unwrap();
    assert_eq!(meta.get("schema"), Some(&J::Int(1)));
    assert_eq!(meta.get("count"), Some(&J::Int(34)));
    assert_eq!(meta.s("synced"), Some(SYNCED));
    let cards = v.get("cards").unwrap().as_arr().unwrap();
    assert_eq!(cards.len(), 34);
    let names: Vec<String> = cards.iter().map(|c| c.s_or("name").to_lowercase()).collect();
    let mut sorted = names.clone();
    sorted.sort();
    assert_eq!(names, sorted, "sorted by lower-case name");
    assert_eq!(names.first().map(String::as_str), Some("akki lavarunner // tok-tok, volcano born"));
    // the unicode name is written as UTF-8
    assert!(ta.contains("Lim-D\u{fb}l's Vault"));
    // meta can be read back from the first line
    let m = bulk::read_meta(&a.join("oracle.json")).unwrap();
    assert_eq!(
        m,
        Meta {
            bulk_updated_at: "from-file".into(),
            download_uri: format!("file:{}", fixture().display()),
            synced: SYNCED.into(),
            count: 34
        }
    );
    no_tmp_files(&a.0);
}

fn read(t: &Tmp, f: &str) -> String {
    std::fs::read_to_string(t.join(f)).unwrap()
}

#[test]
fn sync_lock_records_the_bulk_file_and_the_hash() {
    let t = Tmp::new();
    sync_fixture(&t.0, SYNCED);
    let lock = read(&t, "sync.lock");
    let kv: Vec<(&str, &str)> = lock.lines().map(|l| l.split_once('=').unwrap()).collect();
    assert_eq!(kv.iter().map(|k| k.0).collect::<Vec<_>>(), ["bulk_updated_at", "count", "fnv1a64"]);
    assert_eq!(kv[0].1, "from-file");
    assert_eq!(kv[1].1, "34");
    let want = format!("{:016x}", bulk::hash_of_file(&t.join("oracle.json")).unwrap());
    assert_eq!(kv[2].1, want);
    assert_eq!(want.len(), 16);
    // a changed card changes the hash
    let text = read(&t, "oracle.json").replace("Sol Ring", "Sol Rinh");
    std::fs::write(t.join("changed.json"), text).unwrap();
    assert_ne!(format!("{:016x}", bulk::hash_of_file(&t.join("changed.json")).unwrap()), want);
}

#[test]
fn fnv1a64_known_values() {
    assert_eq!(bulk::fnv1a64(b""), 0xcbf2_9ce4_8422_2325);
    assert_eq!(bulk::fnv1a64(b"a"), 0xaf63_dc4c_8601_ec8c);
    assert_eq!(bulk::fnv1a64(b"foobar"), 0x8594_4171_f739_67e8);
}

// ------------------------------------------------------------------ input formats

fn jsonl_of_fixture() -> String {
    fixture_cards().iter().map(|c| json::dumps(c, None, false) + "\n").collect()
}

fn gzip(bytes: &[u8]) -> Vec<u8> {
    use std::io::Write;
    let mut e = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    e.write_all(bytes).unwrap();
    e.finish().unwrap()
}

/// JSON Lines and gzip, alone or together, give the same oracle.json as the JSON array
#[test]
fn json_lines_and_gzip_read_back_the_same_as_the_array() {
    let src = Tmp::new();
    let array = std::fs::read(fixture()).unwrap();
    let lines = jsonl_of_fixture();
    std::fs::write(src.join("lines.jsonl"), &lines).unwrap();
    std::fs::write(src.join("lines.jsonl.gz"), gzip(lines.as_bytes())).unwrap();
    std::fs::write(src.join("array.json.gz"), gzip(&array)).unwrap();
    // the name does not matter, the bytes do
    std::fs::write(src.join("renamed.json"), gzip(lines.as_bytes())).unwrap();
    // JSON Lines with blank lines and CRLF
    std::fs::write(src.join("crlf.jsonl"), lines.replace('\n', "\r\n\r\n")).unwrap();
    let want = Tmp::new();
    sync_fixture(&want.0, SYNCED);
    let want = read(&want, "oracle.json");
    for name in ["lines.jsonl", "lines.jsonl.gz", "array.json.gz", "renamed.json", "crlf.jsonl"] {
        let t = Tmp::new();
        let o = opts(&t.0, Some(src.join(name)), 1, SYNCED);
        let out = bulk::run(&o, &mut Panics).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert!(out.text.contains("read 44, kept 34, dropped 10"), "{name}: {}", out.text);
        // only the recorded source differs
        let got = read(&t, "oracle.json").replace(&src.join(name).display().to_string(), "SRC");
        let want = want.replace(&fixture().display().to_string(), "SRC");
        assert_eq!(got, want, "{name}");
    }
}

#[test]
fn corrupt_gzip_and_json_lines_fail_and_leave_oracle_json_alone() {
    let t = Tmp::new();
    sync_fixture(&t.0, SYNCED);
    let old = std::fs::read(t.join("oracle.json")).unwrap();
    let bad = Tmp::new();
    let lines = jsonl_of_fixture();
    let gz = gzip(lines.as_bytes());
    for (name, body) in [
        ("cut.gz", gz[..gz.len() / 2].to_vec()),
        ("bitrot.gz", [&gz[..2], b"not gzip at all"].concat()),
        ("cut.jsonl", lines.as_bytes()[..lines.len() / 2].to_vec()),
        ("nonname.jsonl", b"{\"object\": \"error\"}\n".to_vec()),
        ("number.jsonl", b"42\n".to_vec()),
    ] {
        std::fs::write(bad.join(name), body).unwrap();
        let err = bulk::run(&opts(&t.0, Some(bad.join(name)), 1, SYNCED), &mut Panics).unwrap_err();
        assert!(err.contains(name), "{name}: {err}");
        assert_untouched(&t, &old);
    }
}

// ------------------------------------------------------------------ the remote path

#[test]
fn up_to_date_decision() {
    assert!(bulk::is_up_to_date(Some("2026-11-20T21:00:00.000+00:00"), "2026-11-20T21:00:00.000+00:00"));
    assert!(!bulk::is_up_to_date(Some("2026-11-19T21:00:00.000+00:00"), "2026-11-20T21:00:00.000+00:00"));
    assert!(!bulk::is_up_to_date(None, "2026-11-20T21:00:00.000+00:00"));
}

#[test]
fn second_run_with_the_same_updated_at_downloads_nothing() {
    let t = Tmp::new();
    let o = opts(&t.0, None, 1, SYNCED);
    let mut remote = Mock::new("2026-11-20T21:00:00.000+00:00");
    let first = bulk::run(&o, &mut remote).unwrap();
    assert!(!first.up_to_date);
    assert!(first.text.contains("bulk updated_at: 2026-11-20T21:00:00.000+00:00"));
    assert_eq!(remote.downloads, 1);
    let before = (read(&t, "oracle.json"), read(&t, "sync.lock"));
    assert_eq!(bulk::read_meta(&t.join("oracle.json")).unwrap().bulk_updated_at, "2026-11-20T21:00:00.000+00:00");
    let second = bulk::run(&opts(&t.0, None, 1, "2031-01-01T00:00:00Z"), &mut remote).unwrap();
    assert!(second.up_to_date && second.text.starts_with("up to date"), "{}", second.text);
    assert_eq!(remote.downloads, 1, "no second download");
    assert_eq!((read(&t, "oracle.json"), read(&t, "sync.lock")), before, "nothing rewritten");
    assert!(!t.join("oracle.prev.json").exists());
    no_tmp_files(&t.0);
}

#[test]
fn a_newer_bulk_file_keeps_the_previous_sync() {
    let t = Tmp::new();
    let mut remote = Mock::new("2026-11-20T21:00:00.000+00:00");
    bulk::run(&opts(&t.0, None, 1, SYNCED), &mut remote).unwrap();
    let old = read(&t, "oracle.json");
    remote.updated_at = "2026-11-27T21:00:00.000+00:00".into();
    let out = bulk::run(&opts(&t.0, None, 1, "2026-11-28T00:00:00Z"), &mut remote).unwrap();
    assert!(!out.up_to_date);
    assert_eq!(remote.downloads, 2);
    assert_eq!(read(&t, "oracle.prev.json"), old, "the previous sync is kept");
    assert_eq!(bulk::read_meta(&t.join("oracle.json")).unwrap().bulk_updated_at, "2026-11-27T21:00:00.000+00:00");
    assert!(read(&t, "sync.lock").contains("bulk_updated_at=2026-11-27T21:00:00.000+00:00"));
    no_tmp_files(&t.0);
}

#[test]
fn a_failed_download_leaves_the_existing_file_untouched() {
    let t = Tmp::new();
    let mut remote = Mock::new("2026-11-20T21:00:00.000+00:00");
    bulk::run(&opts(&t.0, None, 1, SYNCED), &mut remote).unwrap();
    let (old, lock) = (read(&t, "oracle.json"), read(&t, "sync.lock"));
    remote.updated_at = "2026-11-27T21:00:00.000+00:00".into();
    remote.fail_download = true;
    let err = bulk::run(&opts(&t.0, None, 1, SYNCED), &mut remote).unwrap_err();
    assert!(err.contains("connection reset"), "{err}");
    assert_eq!((read(&t, "oracle.json"), read(&t, "sync.lock")), (old, lock));
    assert!(!t.join("oracle.prev.json").exists());
    no_tmp_files(&t.0);
}

// ------------------------------------------------------------------ failures leave oracle.json alone

fn assert_untouched(dir: &Tmp, old: &[u8]) {
    assert_eq!(std::fs::read(dir.join("oracle.json")).unwrap(), old, "oracle.json changed");
    assert!(!dir.join("oracle.prev.json").exists());
    no_tmp_files(&dir.0);
}

#[test]
fn corrupt_input_fails_and_leaves_oracle_json_byte_identical() {
    let t = Tmp::new();
    sync_fixture(&t.0, SYNCED);
    let old = std::fs::read(t.join("oracle.json")).unwrap();
    let text = std::fs::read_to_string(fixture()).unwrap();
    let bad = Tmp::new();
    for (name, body) in [
        ("truncated.json", text[..text.len() / 2].to_string()),
        ("garbage.json", "this is not json".to_string()),
        ("object.json", "{\"object\": \"error\", \"status\": 404}".to_string()),
        ("trailing.json", format!("{text} trailing")),
        ("empty.json", String::new()),
    ] {
        std::fs::write(bad.join(name), body).unwrap();
        let err = bulk::run(&opts(&t.0, Some(bad.join(name)), 1, SYNCED), &mut Panics).unwrap_err();
        assert!(err.contains(name), "{name}: {err}");
        assert_untouched(&t, &old);
    }
    // a missing file too
    assert!(bulk::run(&opts(&t.0, Some(bad.join("nope.json")), 1, SYNCED), &mut Panics).is_err());
    assert_untouched(&t, &old);
}

#[test]
fn too_few_cards_fails_and_leaves_oracle_json_byte_identical() {
    let t = Tmp::new();
    sync_fixture(&t.0, SYNCED);
    let old = std::fs::read(t.join("oracle.json")).unwrap();
    // the default floor is 25,000 cards: the 34-card fixture does not reach it
    let err = bulk::run(&opts(&t.0, Some(fixture()), bulk::MIN_COUNT, SYNCED), &mut Panics).unwrap_err();
    assert!(err.contains("only 34 cards kept") && err.contains("25000"), "{err}");
    assert_untouched(&t, &old);
    // the floor is on kept cards, not cards read
    assert!(bulk::run(&opts(&t.0, Some(fixture()), 35, SYNCED), &mut Panics).is_err());
    assert_untouched(&t, &old);
    assert_eq!(bulk::MIN_COUNT, 25_000);
}

// ------------------------------------------------------------------ the command

fn cardgen(args: &[&str]) -> std::process::Output {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
    Command::new(env!("CARGO_BIN_EXE_cardgen")).arg("--root").arg(root).args(args).output().unwrap()
}

#[test]
fn command_exits_non_zero_on_a_corrupt_file_and_zero_on_the_fixture() {
    let t = Tmp::new();
    let dir = t.join("knowledge");
    let (d, f) = (dir.to_str().unwrap(), fixture());
    let f = f.to_str().unwrap();
    let ok = cardgen(&["bulk-sync", "--from-file", f, "--data-dir", d, "--min-count", "1"]);
    assert!(ok.status.success(), "{}", String::from_utf8_lossy(&ok.stderr));
    let stdout = String::from_utf8_lossy(&ok.stdout);
    assert!(stdout.contains("read 44, kept 34, dropped 10"), "{stdout}");
    assert!(String::from_utf8_lossy(&ok.stderr).contains("foobar_layout"), "the unknown layout is warned about");
    let old = std::fs::read(dir.join("oracle.json")).unwrap();

    std::fs::write(t.join("corrupt.json"), "[{\"name\": \"Sol Ring\", ").unwrap();
    let bad = cardgen(&[
        "bulk-sync",
        "--from-file",
        t.join("corrupt.json").to_str().unwrap(),
        "--data-dir",
        d,
        "--min-count",
        "1",
    ]);
    assert!(!bad.status.success());
    assert!(String::from_utf8_lossy(&bad.stderr).contains("corrupt.json"));
    assert_eq!(std::fs::read(dir.join("oracle.json")).unwrap(), old);
    // the default floor rejects the fixture
    assert!(!cardgen(&["bulk-sync", "--from-file", f, "--data-dir", d]).status.success());
    assert_eq!(std::fs::read(dir.join("oracle.json")).unwrap(), old);
    // bad usage
    assert!(!cardgen(&["bulk-sync", "--min-count", "many", "--data-dir", d]).status.success());
    assert!(!cardgen(&["bulk-sync", "stray", "--data-dir", d]).status.success());
}
