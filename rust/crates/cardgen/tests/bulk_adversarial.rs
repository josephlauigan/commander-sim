//! Offline adversarial review. Known defects intentionally remain failing and unignored.
use cardgen::bulk::{self, Entry, Meta, Options, Remote};
use cardgen::json::{self, J};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Tmp(PathBuf);

impl Tmp {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "bulk-adversarial-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn join(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }

    fn input(&self, bytes: impl AsRef<[u8]>) -> PathBuf {
        let path = self.join("input");
        fs::write(&path, bytes).unwrap();
        path
    }
}

impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/bulk_fixture.json")
}

fn card(name: &str, id: &str) -> J {
    cardgen::jo! {"name" => name, "oracle_id" => id, "layout" => "normal", "oracle_text" => "Draw a card."}
}

fn set(card: &mut J, key: &str, value: impl Into<J>) {
    card.as_obj_mut().unwrap().set(key, value);
}

fn remove(card: &mut J, key: &str) {
    card.as_obj_mut().unwrap().0.retain(|(k, _)| k != key);
}

fn lines(cards: &[J]) -> String {
    cards.iter().map(|c| json::dumps(c, None, false) + "\n").collect()
}

fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
    encoder.write_all(bytes).unwrap();
    encoder.finish().unwrap()
}

fn opts(dir: &Path, source: Option<PathBuf>) -> Options {
    Options { dir: dir.into(), from_file: source, min_count: 1, synced: "2030-01-01T00:00:00Z".into() }
}

#[derive(Default)]
struct Fake {
    bytes: Vec<u8>,
    updated: String,
    downloads: usize,
    fail_latest: bool,
    fail_download: bool,
}

impl Fake {
    fn new() -> Self {
        Self { bytes: lines(&[card("Original", "id-1")]).into_bytes(), updated: "v1".into(), ..Self::default() }
    }
}

impl Remote for Fake {
    fn latest(&mut self) -> Result<Entry, String> {
        if self.fail_latest {
            return Err("injected latest failure".into());
        }
        Ok(Entry { updated_at: self.updated.clone(), download_uri: "offline:fake".into() })
    }

    fn download(&mut self, _: &str, dest: &Path) -> Result<(), String> {
        self.downloads += 1;
        fs::write(dest, &self.bytes).map_err(|e| e.to_string())?;
        if self.fail_download {
            return Err("injected mid-stream failure".into());
        }
        Ok(())
    }
}

struct Offline;

impl Remote for Offline {
    fn latest(&mut self) -> Result<Entry, String> {
        panic!("offline path consulted remote");
    }
    fn download(&mut self, _: &str, _: &Path) -> Result<(), String> {
        panic!("offline path downloaded");
    }
}

fn seed(t: &Tmp) -> (Options, Fake) {
    let options = opts(&t.0, None);
    let mut fake = Fake::new();
    bulk::run(&options, &mut fake).unwrap();
    (options, fake)
}

fn snapshot(t: &Tmp) -> (Vec<u8>, Vec<u8>) {
    (fs::read(t.join("oracle.json")).unwrap(), fs::read(t.join("sync.lock")).unwrap())
}

fn no_temps(t: &Tmp) {
    let leftovers: Vec<_> = fs::read_dir(&t.0)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .filter(|n| n.to_string_lossy().ends_with(".tmp"))
        .collect();
    assert!(leftovers.is_empty(), "temporary files remain: {leftovers:?}");
}

fn lock_matches(t: &Tmp) {
    let hash = bulk::hash_of_file(&t.join("oracle.json")).unwrap();
    let meta = bulk::read_meta(&t.join("oracle.json")).unwrap();
    assert_eq!(
        fs::read_to_string(t.join("sync.lock")).unwrap(),
        format!("bulk_updated_at={}\ncount={}\nfnv1a64={hash:016x}\n", meta.bulk_updated_at, meta.count)
    );
}

#[test]
fn reader_crlf_blank_lines_and_huge_single_line() {
    let t = Tmp::new();
    let mut c = card("Huge", "huge");
    set(&mut c, "oracle_text", "x".repeat(3 * 1024 * 1024));
    let input = t.input(format!(" \r\n\r\n{}\r\n\r\n", json::dumps(&c, None, false)));
    let result = bulk::read_bulk(&input).unwrap();
    assert_eq!(result.kept(), 1);
    assert_eq!(result.cards[0].1, bulk::slim_bulk(&c));
}

#[test]
fn malformed_readers_preserve_oracle_lock_and_remove_temps() {
    let t = Tmp::new();
    let (mut options, _) = seed(&t);
    let before = snapshot(&t);
    let src = Tmp::new();
    let good = lines(&[card("Good", "good")]);
    let cases = vec![
        Vec::new(),
        b" \r\n\t ".to_vec(),
        gzip(b""),
        b"[42, {\"name\":\"Good\"}]".to_vec(),
        b"{\"object\":\"error\",\"status\":503}".to_vec(),
        b"{\"object\":\"list\",\"data\":[{\"name\":\"Good\"}]}".to_vec(),
        b"{\"name\":42}".to_vec(),
        b"{\"name\":null}".to_vec(),
        format!("{good}{{\"name\":\"unfinished").into_bytes(),
        format!("[{}]\n{good}", json::dumps(&card("First", "first"), None, false)).into_bytes(),
        [b"\xef\xbb\xbf".as_slice(), good.as_bytes()].concat(),
        {
            let mut gz = gzip(good.as_bytes());
            gz.truncate(gz.len() - 5);
            gz
        },
    ];
    for (i, bytes) in cases.into_iter().enumerate() {
        options.from_file = Some(src.input(bytes));
        let result = bulk::run(&options, &mut Offline);
        assert!(result.is_err(), "malformed reader case {i}: {result:?}");
        assert_eq!(snapshot(&t), before, "reader case {i}");
        assert!(!t.join("oracle.prev.json").exists());
        no_temps(&t);
    }
}

#[test]
fn defect_gzip_trailing_garbage_is_silently_accepted() {
    let t = Tmp::new();
    let bytes = [gzip(lines(&[card("A", "a")]).as_bytes()), b"corrupt trailing bytes".to_vec()].concat();
    assert!(bulk::read_bulk(&t.input(bytes)).is_err(), "gzip trailing bytes must not be silently discarded");
}

#[test]
fn defect_concatenated_gzip_members_lose_cards() {
    let t = Tmp::new();
    let bytes = [gzip(lines(&[card("A", "a")]).as_bytes()), gzip(lines(&[card("B", "b")]).as_bytes())].concat();
    let result = bulk::read_bulk(&t.input(bytes)).unwrap();
    assert_eq!(result.kept(), 2, "both valid gzip members must be read");
}

#[test]
fn odd_fields_keep_slim_compatibility() {
    let mut cases = vec![card("Odd", "id")];
    for (key, value) in [
        ("card_faces", J::Str("not an array".into())),
        ("card_faces", J::Arr(vec![])),
        ("card_faces", json::parse(r#"[{"oracle_id":"face-id","colors":["U"]}, {}, null, 42]"#).unwrap()),
        ("legalities", J::Null),
        ("prices", J::Str("not an object".into())),
        ("prices", json::parse(r#"{"usd":null,"usd_foil":null}"#).unwrap()),
        ("edhrec_rank", J::Float(1.5)),
        ("edhrec_rank", J::Str("12".into())),
        ("edhrec_rank", J::Null),
        ("reserved", J::Int(1)),
        ("all_parts", J::Str("wrong".into())),
        ("all_parts", json::parse(r#"[null,42,{}, {"component":"meld_result"}]"#).unwrap()),
    ] {
        let mut c = card("Odd", "id");
        set(&mut c, key, value);
        if key == "card_faces" {
            remove(&mut c, "oracle_id");
        }
        cases.push(c);
    }
    for c in cases {
        let expected = cardgen::scryfall::slim(&c);
        let mut actual = bulk::slim_bulk(&c);
        for (key, value) in expected.as_obj().unwrap().iter() {
            if key == "card_faces" {
                for f in match actual.as_obj_mut().unwrap().get_mut(key).unwrap() {
                    J::Arr(f) => f,
                    _ => unreachable!(),
                } {
                    remove(f, "colors");
                }
            }
            assert_eq!(actual.get(key), Some(value), "{key}: {c:?}");
        }
        assert_eq!(
            actual.as_obj().unwrap().keys().take(expected.as_obj().unwrap().len()).collect::<Vec<_>>(),
            expected.as_obj().unwrap().keys().collect::<Vec<_>>()
        );
    }
    let c = json::parse(r#"{"name":"Faces","card_faces":[{"oracle_id":"face-id"}]}"#).unwrap();
    assert_eq!(bulk::slim_bulk(&c).s("oracle_id"), Some("face-id"));
}

#[test]
fn name_bearing_wrong_field_types_do_not_panic() {
    let t = Tmp::new();
    let c = json::parse(r#"{"name":"Odd","card_faces":"bad","legalities":null,"prices":"bad"}"#).unwrap();
    let result = bulk::read_bulk(&t.input(lines(&[c.clone()]))).unwrap();
    assert_eq!(result.cards[0].1, bulk::slim_bulk(&c));
}

fn meld(id: &str, name: &str) -> J {
    let mut c = card(name, "oracle-meld");
    set(&mut c, "id", id);
    set(&mut c, "layout", "meld");
    set(
        &mut c,
        "all_parts",
        vec![cardgen::jo! {
            "component" => "meld_result", "id" => "result-id", "name" => "Result"
        }],
    );
    c
}

#[test]
fn meld_result_and_meld_part_are_distinguished() {
    assert_eq!(bulk::slim_bulk(&meld("result-id", "Result")).get("all_parts_meld"), Some(&J::Bool(true)));
    assert!(bulk::slim_bulk(&meld("part-id", "Part")).get("all_parts_meld").is_none());
}

#[test]
fn defect_meld_result_name_collision_overrides_different_id() {
    assert!(
        bulk::slim_bulk(&meld("different-card-id", "Result")).get("all_parts_meld").is_none(),
        "different concrete Scryfall ids must take precedence over a matching name"
    );
}

fn rendered(cards: Vec<J>) -> String {
    let t = Tmp::new();
    let s = bulk::read_bulk(&t.input(lines(&cards))).unwrap();
    bulk::render(
        &Meta { bulk_updated_at: "v1".into(), download_uri: "offline".into(), synced: "fixed".into(), count: s.kept() },
        s.cards,
    )
    .0
}

#[test]
fn dedupe_first_kept_copy_wins_and_dropped_copy_does_not_poison_id() {
    let t = Tmp::new();
    let mut dropped = card("Token", "same");
    set(&mut dropped, "layout", "token");
    let s = bulk::read_bulk(&t.input(lines(&[dropped, card("Kept", "same"), card("Later", "same")]))).unwrap();
    assert_eq!((s.read, s.kept(), s.dropped_layout, s.dropped_duplicate), (3, 1, 1, 1));
    assert_eq!(s.cards[0].1.s("name"), Some("Kept"));
}

#[test]
fn unicode_names_and_id_ties_are_shuffle_deterministic() {
    let names = ["Lim-Dûl's Vault", "Fire // Ice", "O'Brien", "İ", "i\u{307}", "ß", "Æ", "æ", "Same", "same"];
    let cards: Vec<_> = names.iter().enumerate().map(|(i, n)| card(n, &format!("id-{i}"))).collect();
    let expected = rendered(cards.clone());
    let mut shuffled = cards;
    for _ in 0..shuffled.len() {
        shuffled.rotate_left(1);
        assert_eq!(rendered(shuffled.clone()), expected);
    }
}

#[test]
fn defect_equal_lowercase_names_without_oracle_ids_depend_on_input_order() {
    let mut a = card("Tie", "unused");
    let mut b = card("tie", "unused");
    remove(&mut a, "oracle_id");
    remove(&mut b, "oracle_id");
    assert_eq!(
        rendered(vec![a.clone(), b.clone()]),
        rendered(vec![b, a]),
        "sort needs a final deterministic tie-break"
    );
}

#[test]
fn metadata_and_card_delimiters_round_trip_and_hash_correctly() {
    let t = Tmp::new();
    let strange = "quote \" backslash \\ newline\n }, \"cards\": [";
    let meta = Meta { bulk_updated_at: strange.into(), download_uri: strange.into(), synced: strange.into(), count: 1 };
    let mut c = card("Delimiters", "delimiters");
    set(&mut c, "oracle_text", strange);
    let (text, hash) = bulk::render(&meta, vec![("delimiters".into(), c)]);
    let p = t.input(&text);
    json::parse(&text).unwrap();
    assert_eq!(bulk::read_meta(&p), Some(meta));
    assert_eq!(bulk::hash_of_file(&p).unwrap(), hash);
}

#[test]
fn latest_download_parse_and_floor_failures_are_atomic() {
    for stage in ["latest", "download", "parse", "floor"] {
        let t = Tmp::new();
        let (mut options, mut fake) = seed(&t);
        let before = snapshot(&t);
        fake.updated = "v2".into();
        match stage {
            "latest" => fake.fail_latest = true,
            "download" => {
                fake.bytes = b"{\"name\":\"partial".to_vec();
                fake.fail_download = true;
            }
            "parse" => fake.bytes = b"{\"name\":\"partial".to_vec(),
            "floor" => options.min_count = 2,
            _ => unreachable!(),
        }
        assert!(bulk::run(&options, &mut fake).is_err(), "{stage}");
        assert_eq!(snapshot(&t), before, "{stage}");
        assert!(!t.join("oracle.prev.json").exists(), "{stage}");
        no_temps(&t);
    }
}

#[test]
fn backup_creation_failure_is_atomic() {
    let t = Tmp::new();
    let (options, mut fake) = seed(&t);
    let before = snapshot(&t);
    fs::create_dir(t.join("oracle.prev.json")).unwrap();
    fake.updated = "v2".into();
    let error = bulk::run(&options, &mut fake).unwrap_err();
    assert!(error.contains("oracle.prev.json"), "{error}");
    assert_eq!(snapshot(&t), before);
    no_temps(&t);
}

#[cfg(unix)]
struct Mode(PathBuf, fs::Permissions);

#[cfg(unix)]
impl Mode {
    fn set(path: &Path, mode: u32) -> Self {
        use std::os::unix::fs::PermissionsExt;
        let old = fs::metadata(path).unwrap().permissions();
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).unwrap();
        Self(path.into(), old)
    }
}

#[cfg(unix)]
impl Drop for Mode {
    fn drop(&mut self) {
        fs::set_permissions(&self.0, self.1.clone()).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn unwritable_directory_failure_is_atomic() {
    let t = Tmp::new();
    let (options, mut fake) = seed(&t);
    let before = snapshot(&t);
    let _mode = Mode::set(&t.0, 0o555);
    fake.updated = "v2".into();
    assert!(bulk::run(&options, &mut fake).is_err());
    // Also reach the output-write failure without a download failing first.
    let local = opts(&t.0, Some(fixture()));
    let error = bulk::run(&local, &mut Offline).unwrap_err();
    assert!(error.contains("oracle.new.tmp"), "{error}");
    assert_eq!(snapshot(&t), before);
    no_temps(&t);
}

#[cfg(unix)]
#[test]
fn defect_lock_write_failure_replaces_oracle() {
    let t = Tmp::new();
    let (options, mut fake) = seed(&t);
    let before = snapshot(&t);
    let _mode = Mode::set(&t.join("sync.lock"), 0o444);
    fake.updated = "v2".into();
    fake.bytes = lines(&[card("Replacement", "id-2")]).into_bytes();
    let error = bulk::run(&options, &mut fake).unwrap_err();
    assert!(error.contains("sync.lock"), "{error}");
    no_temps(&t);
    let after = snapshot(&t);
    assert_eq!(after.1, before.1, "lock must remain byte-identical");
    assert!(after.0 == before.0, "run returned {error}, but oracle.json was replaced; lock still describes v1");
}

#[cfg(unix)]
#[test]
fn defect_lock_symlink_can_overwrite_oracle_with_lock_text() {
    use std::os::unix::fs::symlink;
    let t = Tmp::new();
    let (options, mut fake) = seed(&t);
    fs::remove_file(t.join("sync.lock")).unwrap();
    symlink("oracle.json", t.join("sync.lock")).unwrap();
    fake.updated = "v2".into();
    let outcome = bulk::run(&options, &mut fake);
    let oracle = fs::read_to_string(t.join("oracle.json")).unwrap();
    assert!(json::parse(&oracle).is_ok(), "{outcome:?} left oracle.json containing {oracle:?}");
}

#[test]
fn three_syncs_preserve_immediately_previous_oracle_and_lock_matches() {
    let t = Tmp::new();
    let (options, mut fake) = seed(&t);
    lock_matches(&t);
    for version in ["v2", "v3"] {
        let previous = fs::read(t.join("oracle.json")).unwrap();
        fake.updated = version.into();
        fake.bytes = lines(&[card(version, version)]).into_bytes();
        bulk::run(&options, &mut fake).unwrap();
        assert_eq!(fs::read(t.join("oracle.prev.json")).unwrap(), previous);
        lock_matches(&t);
        no_temps(&t);
    }
}

#[test]
fn timestamp_skip_and_changed_timestamp_with_identical_content() {
    let t = Tmp::new();
    let (options, mut fake) = seed(&t);
    let before = snapshot(&t);
    assert!(bulk::run(&options, &mut fake).unwrap().up_to_date);
    assert_eq!(fake.downloads, 1);
    assert_eq!(snapshot(&t), before);
    let hash = bulk::hash_of_file(&t.join("oracle.json")).unwrap();
    fake.updated = "v2".into();
    assert!(!bulk::run(&options, &mut fake).unwrap().up_to_date);
    assert_eq!(fake.downloads, 2);
    assert_eq!(bulk::hash_of_file(&t.join("oracle.json")).unwrap(), hash);
    lock_matches(&t);
}

#[test]
fn corrupt_header_or_other_schema_causes_resync() {
    for corrupt in ["not JSON".to_string(), "".into(), "{\"meta\": {\"schema\": 999}, \"cards\": [\n".into()] {
        let t = Tmp::new();
        let (options, mut fake) = seed(&t);
        fs::write(t.join("oracle.json"), corrupt).unwrap();
        assert!(bulk::read_meta(&t.join("oracle.json")).is_none());
        assert!(!bulk::run(&options, &mut fake).unwrap().up_to_date);
        assert_eq!(fake.downloads, 2);
        lock_matches(&t);
    }
}

#[test]
fn defect_corrupt_cards_with_valid_header_are_treated_as_current() {
    let t = Tmp::new();
    let (options, mut fake) = seed(&t);
    let oracle = fs::read_to_string(t.join("oracle.json")).unwrap();
    fs::write(t.join("oracle.json"), oracle.lines().next().unwrap().to_string() + "\nBROKEN").unwrap();
    let result = bulk::run(&options, &mut fake).unwrap();
    assert!(!result.up_to_date, "invalid cards body was accepted solely from its surviving header");
    assert_eq!(fake.downloads, 2);
    lock_matches(&t);
}

#[test]
fn from_file_replaces_remote_timestamp_then_remote_downloads_again() {
    let t = Tmp::new();
    let (options, mut fake) = seed(&t);
    let source = Tmp::new();
    let local = opts(&t.0, Some(source.input(lines(&[card("Local", "local")]))));
    bulk::run(&local, &mut Offline).unwrap();
    assert_eq!(bulk::read_meta(&t.join("oracle.json")).unwrap().bulk_updated_at, "from-file");
    lock_matches(&t);
    assert!(!bulk::run(&options, &mut fake).unwrap().up_to_date);
    assert_eq!(fake.downloads, 2);
    assert!(bulk::run(&options, &mut fake).unwrap().up_to_date);
    lock_matches(&t);
}

#[test]
fn defect_from_file_deletes_source_named_download_tmp() {
    let t = Tmp::new();
    let source = t.join("oracle.download.tmp");
    let bytes = lines(&[card("Local", "local")]);
    fs::write(&source, &bytes).unwrap();
    bulk::run(&opts(&t.0, Some(source.clone())), &mut Offline).unwrap();
    assert!(source.exists(), "--from-file deleted its caller-owned input file");
    assert_eq!(fs::read_to_string(source).unwrap(), bytes);
}

#[test]
fn defect_up_to_date_leaves_mismatched_lock() {
    let t = Tmp::new();
    let (options, mut fake) = seed(&t);
    fs::write(t.join("sync.lock"), "bulk_updated_at=wrong\ncount=0\nfnv1a64=0000000000000000\n").unwrap();
    bulk::run(&options, &mut fake).unwrap();
    lock_matches(&t);
}

// Channels make the interleaving reproducible without a timing race: run A is paused after its download, run B
// completes in the same directory, then A resumes.
#[test]
fn concurrent_runs_report_only() {
    use std::sync::mpsc;
    struct Paused {
        fake: Fake,
        written: mpsc::Sender<()>,
        resume: mpsc::Receiver<()>,
    }
    impl Remote for Paused {
        fn latest(&mut self) -> Result<Entry, String> {
            self.fake.latest()
        }
        fn download(&mut self, uri: &str, dest: &Path) -> Result<(), String> {
            self.fake.download(uri, dest)?;
            self.written.send(()).unwrap();
            self.resume.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
            Ok(())
        }
    }
    let t = Tmp::new();
    let (written_tx, written_rx) = mpsc::channel();
    let (resume_tx, resume_rx) = mpsc::channel();
    let options = opts(&t.0, None);
    let first_opts = options.clone();
    let first = std::thread::spawn(move || {
        bulk::run(&first_opts, &mut Paused { fake: Fake::new(), written: written_tx, resume: resume_rx })
    });
    written_rx.recv_timeout(std::time::Duration::from_secs(10)).unwrap();
    let mut second_remote = Fake::new();
    second_remote.updated = "v2".into();
    let second = bulk::run(&options, &mut second_remote);
    resume_tx.send(()).unwrap();
    let first = first.join().unwrap();
    // each run has its own temp files: neither fails on the other's, the last rename wins, and the lock matches it
    first.unwrap();
    second.unwrap();
    lock_matches(&t);
    no_temps(&t);
}

fn cli(root: &Path) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_cardgen"));
    c.arg("--root").arg(root).arg("bulk-sync").env_remove("CARDGEN_KNOWLEDGE_DIR");
    c
}

fn failure(output: Output, expected: &str) {
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(expected), "wanted {expected:?}, stderr={stderr}");
    assert!(output.stdout.is_empty());
}

#[test]
fn cli_bad_arguments_and_missing_inputs_write_nothing() {
    let t = Tmp::new();
    let src = fixture();
    let src = src.to_str().unwrap();
    for (args, message) in [
        (vec!["--min-count", "abc", "--from-file", src], "--min-count needs a number"),
        (vec!["--from-file"], "--from-file needs a value"),
        (vec!["--from-file", src, "--mystery"], "usage:"),
        (vec!["--from-file", src, "extra"], "usage:"),
        (vec!["--from-file", "missing-offline-file"], "missing-offline-file"),
    ] {
        let dir = t.join("output");
        failure(cli(&t.0).arg("--data-dir").arg(&dir).args(args).output().unwrap(), message);
        if dir.exists() {
            assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
        }
    }
    let file = t.join("plain-file");
    fs::write(&file, "unchanged").unwrap();
    for dest in [&file, &file.join("uncreatable")] {
        failure(
            cli(&t.0).arg("--from-file").arg(fixture()).arg("--data-dir").arg(dest).output().unwrap(),
            dest.to_str().unwrap(),
        );
        assert_eq!(fs::read_to_string(&file).unwrap(), "unchanged");
    }
}

#[test]
fn cli_environment_empty_set_and_flag_precedence() {
    for (env, flag, expected) in
        [("", false, "data/knowledge"), ("environment", false, "environment"), ("environment", true, "flag")]
    {
        let t = Tmp::new();
        let mut command = cli(&t.0);
        command.arg("--from-file").arg(fixture()).args(["--min-count", "1"]);
        if env.is_empty() {
            command.env("CARDGEN_KNOWLEDGE_DIR", "");
        } else {
            command.env("CARDGEN_KNOWLEDGE_DIR", t.join(env));
        }
        if flag {
            command.arg("--data-dir").arg(t.join("flag"));
        }
        let output = command.output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(t.join(expected).join("oracle.json").exists());
        if flag {
            assert!(!t.join("environment").exists());
        }
        if !env.is_empty() {
            assert!(!t.join("data/knowledge").exists());
        }
    }
}

#[test]
fn scale_40000_cards_offline_under_one_gigabyte() {
    let t = Tmp::new();
    let cards = json::parse(&fs::read_to_string(fixture()).unwrap()).unwrap();
    let base = cards.as_arr().unwrap().iter().find(|c| c.s("name") == Some("Sol Ring")).unwrap();
    let source = t.join("scale.jsonl");
    let mut file = std::io::BufWriter::new(fs::File::create(&source).unwrap());
    for i in 0..40_000 {
        let mut c = base.clone();
        set(&mut c, "name", format!("Synthetic {i:05}"));
        set(&mut c, "oracle_id", format!("00000000-0000-4000-8000-{i:012}"));
        writeln!(file, "{}", json::dumps(&c, None, false)).unwrap();
    }
    file.flush().unwrap();
    let report = t.join("time.txt");
    let mut command = Command::new("/usr/bin/time");
    command
        .args(["-v", "-o"])
        .arg(&report)
        .arg(env!("CARGO_BIN_EXE_cardgen"))
        .arg("--root")
        .arg(&t.0)
        .arg("bulk-sync")
        .arg("--from-file")
        .arg(source)
        .arg("--data-dir")
        .arg(t.join("knowledge"))
        .env_remove("CARDGEN_KNOWLEDGE_DIR")
        .env("LC_ALL", "C");
    let output = command.output().unwrap();
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("read 40000, kept 40000, dropped 0"));
    let report = fs::read_to_string(report).unwrap();
    eprintln!("40,000-card bulk-sync measurement:\n{report}");
    let rss: u64 = report
        .lines()
        .find_map(|l| l.trim().strip_prefix("Maximum resident set size (kbytes): "))
        .unwrap()
        .parse()
        .unwrap();
    assert!(rss < 1_000_000_000 / 1024, "peak RSS {rss} KiB exceeds 1 GB");
    let dir = Tmp(t.join("knowledge"));
    assert_eq!(bulk::read_meta(&dir.join("oracle.json")).unwrap().count, 40_000);
    lock_matches(&dir);
    no_temps(&dir);
}
