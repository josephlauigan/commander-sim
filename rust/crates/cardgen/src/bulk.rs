//! `cardgen bulk-sync`: Scryfall's "Oracle Cards" bulk file (one record per Oracle card) slimmed into the generated,
//! gitignored `data/knowledge/oracle.json`, plus `oracle.prev.json` (the previous sync) and the tracked `sync.lock`.
//! See `documents/card-knowledge.md`.
//!
//! The bulk file is about 160 MB (25 MB as the gzip JSON Lines file Scryfall serves), so it is streamed to a file (the
//! cache's `Client` has a 30 s total timeout and ureq's `into_string` stops at 10 MB) and parsed one card at a time:
//! only the slim records stay in memory.
//! Any failure leaves the existing `oracle.json` byte-identical. Tests never touch the network: `--from-file` is the
//! offline path, and [`Remote`] is the seam the tests replace.

use crate::jo;
use crate::json::{self, J};
use crate::scryfall::slim;
use serde::de::{DeserializeSeed, SeqAccess, Visitor};
use std::collections::{BTreeMap, HashSet};
use std::fmt;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::time::Duration;

const API: &str = "https://api.scryfall.com";
const USER_AGENT: &str = "commander-pod-sim/1.0 (personal deck testing)";

pub const SCHEMA: i64 = 1;
/// a real sync keeps about 30,000 cards; fewer than this means a truncated or wrong file
pub const MIN_COUNT: usize = 25_000;
/// the layouts dropped at sync (`front_card` is memorabilia: type "Card", all not_legal, and it repeats real names);
/// the only other drop is by type line, see `JUNK_TYPES`
pub const DROP_LAYOUTS: [&str; 8] =
    ["token", "double_faced_token", "emblem", "art_series", "scheme", "planar", "vanguard", "front_card"];
/// a card whose type line (or a face's) starts with one of these words is not a card: Role and other tokens that
/// have a card layout, helper cards (Experience, Poison Counter), Dungeons
const JUNK_TYPES: [&str; 3] = ["Token", "Card", "Dungeon"];
/// layouts Scryfall documents; anything else is kept, with a warning
const KNOWN_LAYOUTS: [&str; 26] = [
    "normal",
    "split",
    "flip",
    "transform",
    "modal_dfc",
    "meld",
    "leveler",
    "saga",
    "adventure",
    "class",
    "case",
    "mutate",
    "prototype",
    "battle",
    "augment",
    "host",
    "reversible_card",
    "prepare",
    "token",
    "double_faced_token",
    "emblem",
    "art_series",
    "scheme",
    "planar",
    "vanguard",
    "front_card",
];

/// FNV-1a, 64 bit
pub fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for &b in bytes {
        h ^= b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    h
}

/// The bulk record: `slim` (the 15 cache fields, byte-compatible) plus what the knowledge layer needs. `set` and
/// `released_at` describe Scryfall's chosen printing, not the first printing.
pub fn slim_bulk(card: &J) -> J {
    let J::Obj(mut out) = slim(card) else { unreachable!("slim returns an object") };
    // faces gain their colours (slim keeps the other 7 keys)
    let src_faces = card.get("card_faces").and_then(|f| f.as_arr());
    if let (Some(J::Arr(faces)), Some(src)) = (out.get_mut("card_faces"), src_faces) {
        for (f, s) in faces.iter_mut().zip(src) {
            if let (J::Obj(fo), Some(c)) = (f, s.field("colors")) {
                fo.set("colors", c.clone());
            }
        }
    }
    if let Some(id) = oracle_id(card) {
        out.set("oracle_id", id);
    }
    if let Some(J::Int(r)) = card.get("edhrec_rank") {
        out.set("edhrec_rank", *r);
    }
    let price = |k: &str| card.get("prices").and_then(|p| p.s(k)).map(str::to_string);
    if let Some(usd) = price("usd").or_else(|| price("usd_foil")) {
        out.set("usd", usd);
    }
    for k in ["released_at", "set", "rarity"] {
        if let Some(v) = card.field(k) {
            out.set(k, v.clone());
        }
    }
    if let Some(v @ J::Bool(_)) = card.get("reserved") {
        out.set("reserved", v.clone());
    }
    if is_meld_result(card) {
        out.set("all_parts_meld", true);
        // the names of the cards that meld into it (the result's own `all_parts` entries of component meld_part)
        let parts = card.get("all_parts").and_then(|p| p.as_arr()).into_iter().flatten();
        let names: Vec<J> =
            parts.filter(|p| p.s("component") == Some("meld_part")).filter_map(|p| p.s("name")).map(J::from).collect();
        out.set("meld_parts", names);
    }
    J::Obj(out)
}

fn oracle_id(card: &J) -> Option<&str> {
    card.s("oracle_id").or_else(|| {
        card.get("card_faces").and_then(|f| f.as_arr()).and_then(|f| f.first()).and_then(|f| f.s("oracle_id"))
    })
}

/// the card is the result of a meld: its own entry in `all_parts` is a `meld_result`. Matched by id or by name: the
/// ids in `all_parts` are printing ids, and the bulk file keeps one arbitrary printing per Oracle card, so an id
/// mismatch says nothing (Ragnarok, Divine Deliverance is card 01c5bafe..., its own entry says a951cc35...).
fn is_meld_result(card: &J) -> bool {
    let (name, id) = (card.s("name"), card.s("id"));
    card.get("all_parts").and_then(|p| p.as_arr()).is_some_and(|parts| {
        parts.iter().any(|p| {
            p.s("component") == Some("meld_result")
                && (id.is_some() && p.s("id") == id || name.is_some() && p.s("name") == name)
        })
    })
}

/// the first word of the card's type line or of any face's type line is a `JUNK_TYPES` word
fn junk_type(card: &J) -> bool {
    let faces = card.get("card_faces").and_then(|f| f.as_arr()).into_iter().flatten();
    std::iter::once(card)
        .chain(faces)
        .any(|c| c.s("type_line").and_then(|t| t.split_whitespace().next()).is_some_and(|w| JUNK_TYPES.contains(&w)))
}

// ------------------------------------------------------------------ reading the bulk file

/// kept and dropped counts of one layout
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct LayoutCount {
    pub kept: usize,
    pub dropped: usize,
}

#[derive(Debug, Default)]
pub struct Slimmed {
    /// (sort key, slim record), unsorted
    pub cards: Vec<(String, J)>,
    pub layouts: BTreeMap<String, LayoutCount>,
    pub read: usize,
    pub dropped_layout: usize,
    pub dropped_type: usize,
    pub dropped_duplicate: usize,
    seen: HashSet<String>,
}

impl Slimmed {
    fn add(&mut self, card: J) {
        self.read += 1;
        let layout = card.s("layout").unwrap_or("(missing)").to_string();
        let drop_layout = DROP_LAYOUTS.contains(&layout.as_str());
        // `digital` is NOT a drop reason: the one printing the Oracle Cards file keeps for an old staple is often a
        // digital Masters set. Commander legality decides playability.
        let drop_type = !drop_layout && junk_type(&card);
        let dup = !drop_layout && !drop_type && oracle_id(&card).is_some_and(|id| self.seen.contains(id));
        let count = self.layouts.entry(layout).or_default();
        if drop_layout {
            count.dropped += 1;
            self.dropped_layout += 1;
        } else if drop_type {
            count.dropped += 1;
            self.dropped_type += 1;
        } else if dup {
            count.dropped += 1;
            self.dropped_duplicate += 1;
        } else {
            count.kept += 1;
            if let Some(id) = oracle_id(&card) {
                self.seen.insert(id.to_string());
            }
            let rec = slim_bulk(&card);
            let name = rec.s_or("name").to_lowercase();
            self.cards.push((name, rec));
        }
    }

    pub fn kept(&self) -> usize {
        self.cards.len()
    }

    pub fn dropped(&self) -> usize {
        self.read - self.cards.len()
    }

    /// layouts that are neither documented nor dropped (the cards were kept)
    pub fn unknown_layouts(&self) -> Vec<&str> {
        self.layouts.keys().map(String::as_str).filter(|l| !KNOWN_LAYOUTS.contains(l)).collect()
    }
}

struct CardsSeed<'a>(&'a mut Slimmed);

impl<'de> DeserializeSeed<'de> for CardsSeed<'_> {
    type Value = ();
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> Result<(), D::Error> {
        d.deserialize_seq(self)
    }
}

impl<'de> Visitor<'de> for CardsSeed<'_> {
    type Value = ();
    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("an array of Scryfall cards")
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<(), A::Error> {
        // one card at a time: the full record is dropped as soon as its slim record is made
        while let Some(card) = seq.next_element::<J>()? {
            check_card(&card).map_err(serde::de::Error::custom)?;
            self.0.add(card);
        }
        Ok(())
    }
}

/// a bulk record must at least be an object with a name; anything else (an error object, a stray value) is not a card
fn check_card(card: &J) -> Result<(), &'static str> {
    if card.s("name").is_some() { Ok(()) } else { Err("a record without a name") }
}

/// the first byte that is not whitespace (left unread), None at the end of the input
fn first_byte<R: BufRead>(r: &mut R) -> std::io::Result<Option<u8>> {
    loop {
        let buf = r.fill_buf()?;
        if buf.is_empty() {
            return Ok(None);
        }
        match buf.iter().position(|b| !b.is_ascii_whitespace()) {
            Some(i) => {
                let b = buf[i];
                r.consume(i);
                return Ok(Some(b));
            }
            None => {
                let n = buf.len();
                r.consume(n);
            }
        }
    }
}

/// Stream-parse a bulk file into slim records: a JSON array of cards, or JSON Lines (what Scryfall serves now), plain
/// or gzip-compressed (told by the gzip magic bytes, not the file name).
pub fn read_bulk(path: &Path) -> Result<Slimmed, String> {
    let bad = |e: &dyn fmt::Display| format!("{}: not a Scryfall bulk file: {e}", path.display());
    let f = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut raw = BufReader::with_capacity(1 << 20, f);
    let gz = raw.fill_buf().map_err(|e| bad(&e))?.starts_with(&[0x1f, 0x8b]);
    if gz {
        parse_cards(BufReader::with_capacity(1 << 20, flate2::read::MultiGzDecoder::new(raw)), &bad)
    } else {
        parse_cards(raw, &bad)
    }
}

fn parse_cards<R: BufRead>(mut r: R, bad: &dyn Fn(&dyn fmt::Display) -> String) -> Result<Slimmed, String> {
    let mut out = Slimmed::default();
    match first_byte(&mut r).map_err(|e| bad(&e))? {
        None => return Err(bad(&"it is empty")),
        Some(b'[') => {
            let mut de = serde_json::Deserializer::from_reader(r);
            CardsSeed(&mut out).deserialize(&mut de).map_err(|e| bad(&e))?;
            de.end().map_err(|e| bad(&e))?;
        }
        Some(_) => {
            for card in serde_json::Deserializer::from_reader(r).into_iter::<J>() {
                let card = card.map_err(|e| bad(&e))?;
                check_card(&card).map_err(|e| bad(&e))?;
                out.add(card);
            }
        }
    }
    Ok(out)
}

// ------------------------------------------------------------------ the output files

#[derive(Debug, Clone, PartialEq)]
pub struct Meta {
    pub bulk_updated_at: String,
    pub download_uri: String,
    pub synced: String,
    pub count: usize,
}

const CARDS_TAIL: &str = ", \"cards\": [";

/// `oracle.json`: line 1 is `{"meta": {...}, "cards": [`, then one card per line, sorted by lower-case name
/// (ties by oracle_id, name, record text); the same input gives the same bytes apart from `meta.synced`. Returns the
/// file text and the FNV-1a-64 of its `cards` array text (`[` .. `]`).
pub fn render(meta: &Meta, mut cards: Vec<(String, J)>) -> (String, u64) {
    // ties: oracle_id, the name as written, then the record text, so the order never depends on the input order
    cards.sort_by(|a, b| {
        (&a.0, a.1.s_or("oracle_id"), a.1.s_or("name"))
            .cmp(&(&b.0, b.1.s_or("oracle_id"), b.1.s_or("name")))
            .then_with(|| json::dumps(&a.1, None, false).cmp(&json::dumps(&b.1, None, false)))
    });
    let meta_obj = jo! {
        "schema" => SCHEMA,
        "bulk_updated_at" => meta.bulk_updated_at.as_str(),
        "download_uri" => meta.download_uri.as_str(),
        "synced" => meta.synced.as_str(),
        "count" => meta.count,
    };
    let mut array = String::from("[\n");
    for (i, (_, c)) in cards.iter().enumerate() {
        if i > 0 {
            array.push_str(",\n");
        }
        array.push_str(&json::dumps(c, None, false));
    }
    array.push_str("\n]");
    let hash = fnv1a64(array.as_bytes());
    (format!("{{\"meta\": {}{CARDS_TAIL}{}}}\n", json::dumps(&meta_obj, None, false), &array[1..]), hash)
}

/// the meta of an existing `oracle.json`, read from its first line only; None if missing or unreadable
pub fn read_meta(path: &Path) -> Option<Meta> {
    use std::io::BufRead;
    let mut line = String::new();
    BufReader::new(std::fs::File::open(path).ok()?).read_line(&mut line).ok()?;
    let head = line.trim_end().strip_suffix(CARDS_TAIL)?;
    let v = json::parse(&format!("{head}}}")).ok()?;
    let m = v.get("meta")?;
    if m.get("schema")?.as_i64()? != SCHEMA {
        return None;
    }
    Some(Meta {
        bulk_updated_at: m.s("bulk_updated_at")?.to_string(),
        download_uri: m.s("download_uri")?.to_string(),
        synced: m.s("synced")?.to_string(),
        count: m.get("count")?.as_i64()? as usize,
    })
}

/// the FNV-1a-64 hash `sync.lock` records, recomputed from an existing `oracle.json`
pub fn hash_of_file(path: &Path) -> Result<u64, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let start = text.find(CARDS_TAIL).ok_or("not an oracle.json")? + CARDS_TAIL.len() - 1;
    let end = text.trim_end().strip_suffix('}').ok_or("not an oracle.json")?.len();
    Ok(fnv1a64(&text.as_bytes()[start..end]))
}

/// what `sync.lock` records
#[derive(Debug, Clone, PartialEq)]
pub struct Lock {
    pub bulk_updated_at: String,
    pub count: usize,
    pub fnv1a64: String,
}

/// an existing `sync.lock`; None if missing or not three well-formed `key=value` lines
pub fn read_lock(path: &Path) -> Option<Lock> {
    let text = std::fs::read_to_string(path).ok()?;
    let (mut schema, mut updated, mut count, mut hash) = (None, None, None, None);
    for line in text.lines() {
        let (k, v) = line.split_once('=')?;
        match k {
            "schema" => schema = v.parse::<i64>().ok(),
            "bulk_updated_at" => updated = Some(v.to_string()),
            "count" => count = v.parse().ok(),
            "fnv1a64" => hash = Some(v.to_string()),
            _ => return None,
        }
    }
    // a lock without the current schema line is invalid (an older or newer cardgen wrote it): re-sync
    (schema? == SCHEMA).then_some(Lock { bulk_updated_at: updated?, count: count?, fnv1a64: hash? })
}

fn lock_text(meta: &Meta, hash: u64) -> String {
    format!("schema={SCHEMA}\nbulk_updated_at={}\ncount={}\nfnv1a64={hash:016x}\n", meta.bulk_updated_at, meta.count)
}

// ------------------------------------------------------------------ the sync

/// the bulk file is current when its `updated_at` is the one the existing `oracle.json` was built from
pub fn is_up_to_date(current: Option<&str>, remote_updated_at: &str) -> bool {
    current == Some(remote_updated_at)
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub updated_at: String,
    pub download_uri: String,
}

/// where the bulk file comes from; the tests replace it so they never use the network
pub trait Remote {
    fn latest(&mut self) -> Result<Entry, String>;
    fn download(&mut self, uri: &str, dest: &Path) -> Result<(), String>;
}

/// Scryfall's public endpoints
pub struct Http;

impl Http {
    fn get(agent: &ureq::Agent, url: &str) -> Result<ureq::Response, String> {
        agent.get(url).set("User-Agent", USER_AGENT).set("Accept", "application/json").call().map_err(|e| e.to_string())
    }
}

impl Remote for Http {
    fn latest(&mut self) -> Result<Entry, String> {
        let agent = ureq::AgentBuilder::new().timeout(Duration::from_secs(30)).build();
        let body = Self::get(&agent, &format!("{API}/bulk-data"))?.into_string().map_err(|e| e.to_string())?;
        let v = json::parse(&body)?;
        let e = v
            .get("data")
            .and_then(|d| d.as_arr())
            .and_then(|d| d.iter().find(|e| e.s("type") == Some("oracle_cards")))
            .ok_or("Scryfall's bulk-data list has no oracle_cards entry")?;
        // Scryfall serves a gzip file of JSON Lines (`jsonl_download_uri`); `download_uri` was a JSON array
        match (e.s("updated_at"), e.s("jsonl_download_uri").or_else(|| e.s("download_uri"))) {
            (Some(u), Some(d)) => Ok(Entry { updated_at: u.to_string(), download_uri: d.to_string() }),
            _ => Err("the oracle_cards entry has no updated_at or jsonl_download_uri".into()),
        }
    }

    /// streamed to `dest` as served (a gzip file of about 25 MB; `read_bulk` decompresses it): no total timeout, 60 s
    /// without a byte is a failure
    fn download(&mut self, uri: &str, dest: &Path) -> Result<(), String> {
        if !uri.starts_with("https://") {
            return Err(format!("refusing to download {uri}: only https:// is allowed"));
        }
        let agent = ureq::AgentBuilder::new()
            .timeout_connect(Duration::from_secs(30))
            .timeout_read(Duration::from_secs(60))
            .build();
        let resp = Self::get(&agent, uri)?;
        let mut out = std::fs::File::create(dest).map_err(|e| format!("{}: {e}", dest.display()))?;
        std::io::copy(&mut resp.into_reader(), &mut out).map_err(|e| format!("download failed: {e}"))?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Options {
    /// the knowledge directory (`data/knowledge`)
    pub dir: PathBuf,
    /// parse this local bulk file instead of downloading (no network, no up-to-date check)
    pub from_file: Option<PathBuf>,
    /// refuse to write fewer cards than this
    pub min_count: usize,
    /// the time to record in `meta.synced` (injected so tests are deterministic)
    pub synced: String,
}

#[derive(Debug)]
pub struct Outcome {
    /// what to print
    pub text: String,
    pub warnings: Vec<String>,
    pub up_to_date: bool,
}

/// `oracle.json` is current when its meta, `sync.lock` and the remote agree on `updated_at`, the lock's count is the
/// meta's, and the lock's hash is the one recomputed from the file; anything else (missing, corrupt, stale) re-syncs
fn current_count(dir: &Path, remote_updated_at: &str) -> Option<usize> {
    let oracle = dir.join("oracle.json");
    let meta = read_meta(&oracle)?;
    let lock = read_lock(&dir.join("sync.lock"))?;
    let hash = format!("{:016x}", hash_of_file(&oracle).ok()?);
    let agree = lock.bulk_updated_at == meta.bulk_updated_at && lock.count == meta.count && lock.fnv1a64 == hash;
    (agree && is_up_to_date(Some(&meta.bulk_updated_at), remote_updated_at)).then_some(meta.count)
}

/// a name for this invocation's temp files (process id and a counter, so two runs never share one)
fn temp_token() -> String {
    static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    format!("{}-{}", std::process::id(), N.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
}

pub fn run(opts: &Options, remote: &mut dyn Remote) -> Result<Outcome, String> {
    std::fs::create_dir_all(&opts.dir).map_err(|e| format!("{}: {e}", opts.dir.display()))?;
    let tok = temp_token();
    let oracle = opts.dir.join("oracle.json");
    let download = opts.dir.join(format!("oracle.{tok}.download.tmp"));
    let tmps = Temps {
        fresh: opts.dir.join(format!("oracle.{tok}.new.tmp")),
        lock: opts.dir.join(format!("sync.{tok}.lock.tmp")),
    };
    let (updated_at, uri, source, downloaded) = match &opts.from_file {
        Some(p) => ("from-file".to_string(), format!("file:{}", p.display()), p.clone(), false),
        None => {
            let e = remote.latest()?;
            if let Some(n) = current_count(&opts.dir, &e.updated_at) {
                return Ok(Outcome {
                    text: format!("up to date (bulk updated_at {}, {n} cards)\n", e.updated_at),
                    warnings: vec![],
                    up_to_date: true,
                });
            }
            remote.download(&e.download_uri, &download).inspect_err(|_| {
                let _ = std::fs::remove_file(&download);
            })?;
            (e.updated_at, e.download_uri, download.clone(), true)
        }
    };
    let res = sync_file(opts, &source, updated_at, uri, &oracle, &tmps);
    // only files this invocation made: never the --from-file source
    if downloaded {
        let _ = std::fs::remove_file(&download);
    }
    let _ = std::fs::remove_file(&tmps.fresh);
    let _ = std::fs::remove_file(&tmps.lock);
    res
}

/// the staging files of one sync
struct Temps {
    fresh: PathBuf,
    lock: PathBuf,
}

fn sync_file(
    opts: &Options,
    source: &Path,
    updated_at: String,
    uri: String,
    oracle: &Path,
    tmps: &Temps,
) -> Result<Outcome, String> {
    let lock = opts.dir.join("sync.lock");
    let s = read_bulk(source)?;
    if s.kept() < opts.min_count {
        return Err(format!(
            "only {} cards kept of {} read (minimum {}); the file is truncated or not the Oracle Cards bulk data; \
             {} left untouched",
            s.kept(),
            s.read,
            opts.min_count,
            oracle.display()
        ));
    }
    let warnings: Vec<String> = s
        .unknown_layouts()
        .iter()
        .map(|l| {
            let n = s.layouts[*l].kept;
            format!(
                "warning: unknown layout \"{l}\" ({n} cards kept); add it to KNOWN_LAYOUTS in bulk.rs if it is fine"
            )
        })
        .collect();
    for w in &warnings {
        eprintln!("{w}");
    }
    let mut text = format!("bulk updated_at: {updated_at}\n");
    text.push_str(&format!("{:<22}{:>8}{:>9}\n", "layout", "kept", "dropped"));
    for (l, c) in &s.layouts {
        text.push_str(&format!("{l:<22}{:>8}{:>9}\n", c.kept, c.dropped));
    }
    text.push_str(&format!(
        "read {}, kept {}, dropped {} (layout {}, type {}, duplicate oracle_id {})\n",
        s.read,
        s.kept(),
        s.dropped(),
        s.dropped_layout,
        s.dropped_type,
        s.dropped_duplicate
    ));
    let meta = Meta { bulk_updated_at: updated_at, download_uri: uri, synced: opts.synced.clone(), count: s.kept() };
    let (file_text, hash) = render(&meta, s.cards);
    // a read-only lock is the user's; refuse before anything is touched (a rename would replace it anyway)
    if std::fs::metadata(&lock).is_ok_and(|m| m.permissions().readonly()) {
        return Err(format!("{}: is read-only; {} left untouched", lock.display(), oracle.display()));
    }
    // stage both files first, so a failure here changes nothing
    std::fs::write(&tmps.fresh, file_text)
        .map_err(|e| format!("staging oracle.new.tmp ({}): {e}", tmps.fresh.display()))?;
    std::fs::write(&tmps.lock, lock_text(&meta, hash)).map_err(|e| format!("{}: {e}", tmps.lock.display()))?;
    let had_old = oracle.exists();
    let prev = opts.dir.join("oracle.prev.json");
    if had_old {
        // keep the previous sync; oracle.json itself is never absent (a hard link, then an atomic rename over it)
        let _ = std::fs::remove_file(&prev);
        if std::fs::hard_link(oracle, &prev).is_err() {
            std::fs::copy(oracle, &prev).map_err(|e| format!("{}: {e}", prev.display()))?;
        }
    }
    std::fs::rename(&tmps.fresh, oracle).map_err(|e| format!("{}: {e}", oracle.display()))?;
    // the lock is renamed into place (a rename replaces a symlink, it never writes through it); if that fails the
    // new oracle is rolled back, so oracle.json and sync.lock always describe the same sync
    if let Err(e) = std::fs::rename(&tmps.lock, &lock) {
        let undone = if had_old { std::fs::rename(&prev, oracle) } else { std::fs::remove_file(oracle) };
        return Err(rollback_message(&lock, &e, oracle, undone));
    }
    text.push_str(&format!("wrote {} ({} cards), {}\n", oracle.display(), meta.count, lock.display()));
    Ok(Outcome { text, warnings, up_to_date: false })
}

/// the error for a failed lock rename, saying truthfully whether the oracle was put back
fn rollback_message(lock: &Path, err: &std::io::Error, oracle: &Path, undone: std::io::Result<()>) -> String {
    match undone {
        Ok(()) => format!("{}: {err}; {} restored", lock.display(), oracle.display()),
        Err(e) => format!(
            "{}: {err}; {} NOT restored: {e}; oracle.prev.json holds the previous sync",
            lock.display(),
            oracle.display()
        ),
    }
}

/// the time for `meta.synced`
pub fn now_utc() -> String {
    chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollback_message_says_whether_the_oracle_was_restored() {
        let (lock, oracle) = (Path::new("d/sync.lock"), Path::new("d/oracle.json"));
        let e = std::io::Error::other("is a directory");
        let ok = rollback_message(lock, &e, oracle, Ok(()));
        assert!(ok.contains("d/oracle.json restored") && !ok.contains("NOT"), "{ok}");
        let failed = rollback_message(lock, &e, oracle, Err(std::io::Error::other("gone")));
        assert!(failed.contains("d/oracle.json NOT restored: gone") && failed.contains("oracle.prev.json"), "{failed}");
        assert!(!failed.contains(" restored;") && failed.matches("restored").count() == 1, "{failed}");
    }
}
