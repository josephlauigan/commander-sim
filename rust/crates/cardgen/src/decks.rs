//! Deck lists: your decks (`decklists/<owner>/<deck>.md`, named in data/my_decks.toml) and the outside decks
//! (`decklists/pool/<tier folder>/<deck>.md`, the five power tiers). A deck file's cards are its `## Import list`
//! block ("1 Sol Ring" lines); a pool deck's commander is its `**Commander:**` line.

use crate::data::{self, Bans, FormatDef, MyDeck, Project, front_face, read};
use crate::json::J;
use crate::py;
use crate::re::{rmatch, search};
use crate::scryfall::Cache;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// the cards of a deck file (each listed name repeated by its count)
pub fn load(path: &Path) -> Result<Vec<String>, String> {
    let txt = read(path)?;
    let bad = || format!("{}: no '## Import list' block", path.display());
    let after = txt.split("## Import list").nth(1).ok_or_else(bad)?;
    let block = after.split("```").nth(1).ok_or_else(bad)?;
    let mut out = vec![];
    for line in block.trim().lines() {
        let (n, name) = line.split_once(' ').ok_or(format!("{}: bad line {line:?}", path.display()))?;
        let n: usize = n.parse().map_err(|_| format!("{}: bad count in {line:?}", path.display()))?;
        for _ in 0..n {
            out.push(name.trim().to_string());
        }
    }
    Ok(out)
}

/// where one of your deck files is: $SIM_DECKS (flat, or with the owner folders), then decklists/
pub fn deck_path(p: &Project, d: &MyDeck) -> PathBuf {
    let file = Path::new(&d.file);
    let base = file.file_name().map(PathBuf::from).unwrap_or_default();
    if let Ok(env) = std::env::var("SIM_DECKS") {
        if !env.is_empty() {
            for c in [Path::new(&env).join(&base), Path::new(&env).join(file)] {
                if c.exists() {
                    return c;
                }
            }
        }
    }
    p.decklists().join(file)
}

#[derive(Debug, Clone)]
pub struct PoolDeck {
    pub path: PathBuf,
    pub tier: String,
    pub key: String,
    pub commander: String,
    /// a second commander (a partner pair: `- **Commander:** A / B` in a pool file, or your two-commander decks)
    pub commander2: Option<String>,
    pub cards: Vec<String>,
    /// the file's first line without its '#'
    pub title: String,
    /// the Game Changer count the file's header claims
    pub gc_claimed: Option<usize>,
    /// the key of the format it is checked under: your decks' own (data/my_decks.toml), `commander` for a pool deck
    pub format: String,
}

/// all pool decks, sorted by path (tier, then file name)
pub fn load_pool(p: &Project) -> Result<Vec<PoolDeck>, String> {
    let mut paths = vec![];
    let dir = p.decklists().join("pool");
    for tier in std::fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let tier = tier.map_err(|e| e.to_string())?.path();
        let fname = tier.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
        if !tier.is_dir() || !fname.starts_with('t') {
            continue;
        }
        for f in std::fs::read_dir(&tier).map_err(|e| e.to_string())? {
            let f = f.map_err(|e| e.to_string())?.path();
            if f.extension().is_some_and(|e| e == "md") {
                paths.push(f);
            }
        }
    }
    paths.sort_by_key(|a| a.to_string_lossy().to_string());
    paths.iter().map(|f| load_pool_deck(f)).collect()
}

pub fn load_pool_deck(path: &Path) -> Result<PoolDeck, String> {
    let folder = path.parent().and_then(|d| d.file_name()).map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let tier = folder.split('-').next().unwrap_or("").to_string();
    let key = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    let txt = read(path)?;
    let line = search(r"(?m)^- \*\*Commander:\*\*\s*(.+?)\s*$", &txt)
        .map(|m| m.gs(1).to_string())
        .ok_or(format!("{}: no **Commander:** line", path.display()))?;
    // `A / B` is a partner pair (a double-faced card's `A // B` has no spaced single slash)
    let (commander, commander2) = match line.split_once(" / ") {
        Some((a, b)) => (a.trim().to_string(), Some(b.trim().to_string())),
        None => (line, None),
    };
    let title = py::strip(txt.lines().next().unwrap_or(""), "# ").to_string();
    let gc_claimed = search(r"\*\*Game Changers \((\d+)\)", &txt).and_then(|m| m.gs(1).parse().ok());
    Ok(PoolDeck {
        path: path.to_path_buf(),
        tier,
        key,
        commander,
        commander2,
        cards: load(path)?,
        title,
        gc_claimed,
        format: data::DEFAULT_FORMAT.into(),
    })
}

/// the name logs use for a pool deck: its commander's first name part
pub fn short_name(commander: &str) -> String {
    commander.split(" // ").next().unwrap_or("").split(',').next().unwrap_or("").to_string()
}

/// the name a pool deck prints: the short name, and for a partner pair both first names ("Rograkh/Silas")
pub fn display_name(commander: &str, commander2: Option<&str>) -> String {
    let first = |c: &str| short_name(c).split_whitespace().next().unwrap_or("").to_string();
    match commander2 {
        Some(c2) => format!("{}/{}", first(commander), first(c2)),
        None => short_name(commander),
    }
}

/// `n Card Name` -> (n, name)
pub fn parse_line(line: &str) -> Option<(usize, String)> {
    let m = rmatch(r"^(\d+)\s+(.+?)\s*$", line.trim())?;
    Some((m.gs(1).parse().ok()?, m.gs(2).to_string()))
}

pub const BASICS: [&str; 11] = [
    "Plains",
    "Island",
    "Swamp",
    "Mountain",
    "Forest",
    "Wastes",
    "Snow-Covered Plains",
    "Snow-Covered Island",
    "Snow-Covered Swamp",
    "Snow-Covered Mountain",
    "Snow-Covered Forest",
];

/// Game Changer rules per tier (decklists/pool/README.md): (min, max); None = unrestricted
pub fn gc_rules(tier: &str) -> Option<(usize, usize)> {
    match tier {
        "t0" | "t1" => Some((0, 0)),
        "t2" => Some((1, 2)),
        "t3" => Some((3, 3)),
        _ => None,
    }
}

/// Checks that need no card data: size, singleton, commander present.
pub fn structural_problems(cards: &[String], commanders: &[&str], size: usize) -> Vec<String> {
    let mut probs = vec![];
    if cards.len() != size {
        probs.push(format!("{} cards, expected {size}", cards.len()));
    }
    if commanders.iter().all(|c| c.is_empty()) {
        probs.push("no **Commander:** line".into());
    }
    for commander in commanders.iter().filter(|c| !c.is_empty()) {
        if !cards.iter().any(|c| c == commander) {
            probs.push(format!("commander {} is not in the import list", py_repr(commander)));
        }
    }
    let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    for c in cards {
        *count.entry(c).or_default() += 1;
    }
    let dups: Vec<&str> = count.iter().filter(|(n, k)| **k > 1 && !BASICS.contains(n)).map(|(n, _)| *n).collect();
    if !dups.is_empty() {
        probs.push(format!("not singleton: {}", dups.join(", ")));
    }
    probs
}

/// Python's `repr()` of a card name (single quotes unless the name has one and no double quote)
pub fn py_repr(s: &str) -> String {
    if s.contains('\'') && !s.contains('"') {
        format!("\"{s}\"")
    } else {
        format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'"))
    }
}

/// Did Scryfall return the card we asked for (not a fuzzy near-miss)?
pub fn resolved_name_ok(requested: &str, rec: &J) -> bool {
    let key = requested.trim().to_lowercase();
    let mut names = vec![rec.s_or("name").to_lowercase()];
    if let Some(f) = rec.get("card_faces").and_then(|f| f.as_arr()) {
        names.extend(f.iter().map(|x| x.s_or("name").to_lowercase()));
    }
    names.contains(&key)
}

/// why a card is not legal in a format
#[derive(Debug, Clone, PartialEq)]
pub enum Illegal {
    /// on the format's ban list (Scryfall's `banned` for Commander, `banned` in formats.toml for a list format)
    Banned,
    /// outside the format's card pool, with the reason ("Scryfall not_legal", "Scryfall restricted", "no legality data")
    NotLegal(String),
}

/// What keeps a card out of a format (an empty list: it is legal).
/// - `bans = "scryfall"` (Commander): `legalities.commander` `banned`, or any other value but `legal`, refuses it. A
///   record with no legality field passes (the cache's real records always have one).
/// - `bans = "list"` (Cali Commander): the card must be a Commander-playable card (`legal` or `banned` in Scryfall's
///   Commander field: a card Scryfall calls `banned` is playable there, just not in Commander; `not_legal` and
///   `restricted` are refused, and so is a record with no legality field), and then no face of it may be on the
///   format's list. Names compare by front face, lower case, for the requested name, the record's name and each of
///   its faces (a list may name the face that is the card's identity: "Erayo's Essence" is Erayo's back face).
/// `rec` is None for a name with no Scryfall record: a list format still reports it when the name is on the list.
pub fn legality_problems(fmt: &FormatDef, requested: &str, rec: Option<&J>) -> Vec<Illegal> {
    let mut out = vec![];
    let scry = rec.and_then(|r| r.get("legalities")).and_then(|l| l.get("commander")).and_then(|c| c.as_str());
    match fmt.bans {
        Bans::Scryfall => match scry {
            Some("banned") => out.push(Illegal::Banned),
            Some("legal") | None => {}
            Some(other) => out.push(Illegal::NotLegal(format!("Scryfall {other}"))),
        },
        Bans::List => {
            if let Some(r) = rec {
                match scry {
                    Some("legal") | Some("banned") => {}
                    Some(other) => out.push(Illegal::NotLegal(format!("Scryfall {other}"))),
                    None => out.push(Illegal::NotLegal("no legality data".into())),
                }
                let mut names = vec![front_face(requested), front_face(r.s_or("name"))];
                if let Some(f) = r.get("card_faces").and_then(|f| f.as_arr()) {
                    names.extend(f.iter().map(|x| front_face(x.s_or("name"))));
                }
                if fmt.banned.iter().any(|b| names.contains(&front_face(b))) {
                    out.push(Illegal::Banned);
                }
            } else if fmt.banned.iter().any(|b| front_face(b) == front_face(requested)) {
                out.push(Illegal::Banned);
            }
        }
    }
    out
}

/// what `card_problems` found out besides the problems
pub struct DeckInfo {
    pub gcs: Vec<String>,
    pub identity: String,
}

/// Checks that need Scryfall data. `recs`: requested name -> record (Python's slim records).
pub fn card_problems(
    fmt: &FormatDef,
    cards: &[String],
    commanders: &[&str],
    tier: Option<&str>,
    gc_claimed: Option<usize>,
    recs: &dyn Fn(&str) -> Option<J>,
) -> (Vec<String>, DeckInfo) {
    let mut probs = vec![];
    let mut names: Vec<&String> = cards.iter().collect();
    names.sort();
    names.dedup();
    let rec_of: Vec<(&String, Option<J>)> = names.iter().map(|n| (*n, recs(n))).collect();
    let missing: Vec<&str> = rec_of.iter().filter(|(_, r)| r.is_none()).map(|(n, _)| n.as_str()).collect();
    if !missing.is_empty() {
        probs.push(format!("not found on Scryfall: {}", missing.join(", ")));
    }
    let fuzzy: Vec<String> = rec_of
        .iter()
        .filter_map(|(n, r)| {
            r.as_ref().filter(|r| !resolved_name_ok(n, r)).map(|r| format!("{n} -> {}", r.s_or("name")))
        })
        .collect();
    if !fuzzy.is_empty() {
        probs.push(format!("resolved only by fuzzy match (check spelling): {}", fuzzy.join(", ")));
    }
    let cmds: Vec<(&str, Option<J>)> = commanders.iter().filter(|c| !c.is_empty()).map(|c| (*c, recs(c))).collect();
    let oracle_of = |c: &J| {
        let mut text = c.s("oracle_text").unwrap_or("").to_string();
        text.push(' ');
        text.push_str(
            &c.get("card_faces")
                .and_then(|f| f.as_arr())
                .map(|f| f.iter().map(|x| x.s("oracle_text").unwrap_or("").to_string()).collect::<Vec<_>>().join(" "))
                .unwrap_or_default(),
        );
        text
    };
    for (commander, cmd) in &cmds {
        if let Some(c) = cmd {
            let tl = c.s("type_line").unwrap_or("");
            let text = oracle_of(c);
            if !(tl.contains("Legendary") && tl.contains("Creature")) && !text.contains("can be your commander") {
                probs.push(format!("commander {commander} is not a legendary creature"));
            }
            // two commanders: each needs plain Partner (not "Partner with", not Friends forever)
            if cmds.len() == 2 {
                let plain = c.s("oracle_text").unwrap_or("").lines().any(|l| {
                    let l = l.trim();
                    l == "Partner" || l.starts_with("Partner (You can have two commanders")
                });
                if !plain {
                    probs.push(format!("commander {commander} does not have plain Partner"));
                }
            }
        }
    }
    if cmds.len() > 2 {
        probs.push(format!("{} commanders (at most 2)", cmds.len()));
    }
    // the deck's colour identity: the union of its commanders'
    let ident: Option<Vec<String>> = if !cmds.is_empty() && cmds.iter().all(|(_, c)| c.is_some()) {
        let mut u: Vec<String> = vec![];
        for (_, c) in &cmds {
            for s in c.as_ref().unwrap().strs("color_identity") {
                if !u.iter().any(|x| x == s) {
                    u.push(s.to_string());
                }
            }
        }
        Some(u)
    } else {
        None
    };
    let identity = match &ident {
        Some(i) => {
            let mut v: Vec<&str> = i.iter().map(|s| s.as_str()).collect();
            v.sort();
            let id = v.concat();
            let off: Vec<&str> = rec_of
                .iter()
                .filter(|(_, r)| {
                    r.as_ref().is_some_and(|r| !r.strs("color_identity").iter().all(|c| i.iter().any(|x| x == c)))
                })
                .map(|(n, _)| n.as_str())
                .collect();
            if !off.is_empty() {
                probs.push(format!(
                    "outside colour identity {}: {}",
                    if id.is_empty() { "C" } else { &id },
                    off.join(", ")
                ));
            }
            id
        }
        None => "?".to_string(),
    };
    let verdicts: Vec<(&str, Vec<Illegal>)> =
        rec_of.iter().map(|(n, r)| (n.as_str(), legality_problems(fmt, n, r.as_ref()))).collect();
    let banned: Vec<&str> = verdicts.iter().filter(|(_, v)| v.contains(&Illegal::Banned)).map(|(n, _)| *n).collect();
    if !banned.is_empty() {
        probs.push(format!("banned in {}: {}", fmt.name, banned.join(", ")));
    }
    let not_legal: Vec<String> = verdicts
        .iter()
        .flat_map(|(n, v)| {
            v.iter().filter_map(move |i| match i {
                Illegal::NotLegal(why) => Some(format!("{n} ({why})")),
                Illegal::Banned => None,
            })
        })
        .collect();
    if !not_legal.is_empty() {
        probs.push(format!("not legal in {}: {}", fmt.name, not_legal.join(", ")));
    }
    let gcs: Vec<String> = rec_of
        .iter()
        .filter(|(_, r)| r.as_ref().is_some_and(|r| r.get("game_changer").is_some_and(|g| g.truthy())))
        .map(|(n, _)| n.to_string())
        .collect();
    if let Some((lo, hi)) = tier.and_then(gc_rules) {
        if !(lo <= gcs.len() && gcs.len() <= hi) {
            probs.push(format!("{} Game Changers, tier {} allows {lo}-{hi}", gcs.len(), tier.unwrap()));
        }
    }
    if let Some(c) = gc_claimed {
        if c != gcs.len() {
            probs.push(format!("file says {c} Game Changers, Scryfall says {}", gcs.len()));
        }
    }
    (probs, DeckInfo { gcs, identity })
}

/// the format with this key; formats.toml always has `commander` and `load_my_decks` refuses an unknown key
pub fn format_of<'a>(formats: &'a [FormatDef], key: &str) -> Result<&'a FormatDef, String> {
    formats.iter().find(|f| f.key == key).ok_or(format!("no format {key:?} in data/formats.toml"))
}

/// `cardgen validate`: every pool deck's problems (Python's `pools --validate`), under Commander, then each of your
/// decks (size, singleton, commanders, colours, bans) under its own format
pub fn validate_all(p: &Project, cache: &Cache) -> Result<Vec<(PoolDeck, Vec<String>, DeckInfo)>, String> {
    let formats = data::load_formats(p)?;
    let commander = format_of(&formats, data::DEFAULT_FORMAT)?;
    let mut out = vec![];
    for d in load_pool(p)? {
        let cmds: Vec<&str> = std::iter::once(d.commander.as_str()).chain(d.commander2.as_deref()).collect();
        let mut probs = structural_problems(&d.cards, &cmds, 100);
        let (p2, info) =
            card_problems(commander, &d.cards, &cmds, Some(&d.tier), d.gc_claimed, &|n| cache.get(n).cloned());
        probs.extend(p2);
        out.push((d, probs, info));
    }
    // your decks: 100 cards, the commanders (two need plain Partner; the union of their colours), each deck's format
    for m in data::load_my_decks(p)? {
        let path = deck_path(p, &m);
        let d = PoolDeck {
            path: path.clone(),
            tier: "mine".into(),
            key: m.key.clone(),
            commander: m.commander.clone(),
            commander2: m.commander2.clone(),
            cards: load(&path)?,
            title: String::new(),
            gc_claimed: None,
            format: m.format.clone(),
        };
        let cmds: Vec<&str> = std::iter::once(d.commander.as_str()).chain(m.commander2.as_deref()).collect();
        let mut probs = structural_problems(&d.cards, &cmds, 100);
        let (p2, info) =
            card_problems(format_of(&formats, &m.format)?, &d.cards, &cmds, None, None, &|n| cache.get(n).cloned());
        probs.extend(p2);
        out.push((d, probs, info));
    }
    Ok(out)
}

#[cfg(test)]
mod legality_tests {
    use super::*;
    use crate::data::{FormatDef, load_formats};
    use crate::jo;

    fn formats() -> (FormatDef, FormatDef) {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let all = load_formats(&Project::new(root)).unwrap();
        let get = |k: &str| all.iter().find(|f| f.key == k).unwrap().clone();
        (get("commander"), get("cali"))
    }

    /// a fabricated Scryfall record
    fn rec(name: &str, scry: &str) -> J {
        jo! {"name" => name, "color_identity" => J::Arr(vec![]), "legalities" => jo!{"commander" => scry}}
    }

    fn illegal(fmt: &FormatDef, name: &str, scry: &str) -> Vec<Illegal> {
        legality_problems(fmt, name, Some(&rec(name, scry)))
    }

    /// the 18 names: the Cali Commander ban list (decklists/cali_commander/kyodai-artifact-combo.md)
    const CALI_BANNED: [&str; 18] = [
        "Ancestral Recall",
        "Balance",
        "Black Lotus",
        "Channel",
        "Erayo, Soratami Ascendant",
        "Fastbond",
        "Karakas",
        "Mox Emerald",
        "Mox Jet",
        "Mox Pearl",
        "Mox Ruby",
        "Mox Sapphire",
        "Nadu, Winged Wisdom",
        "The Tabernacle at Pendrell Vale",
        "Time Vault",
        "Time Walk",
        "Tolarian Academy",
        "Yawgmoth's Bargain",
    ];

    #[test]
    fn the_cali_ban_list_is_pinned_to_the_18_names() {
        let (_, cali) = formats();
        let mut got = cali.banned.clone();
        got.sort();
        let mut want: Vec<String> = CALI_BANNED.iter().map(|s| s.to_string()).collect();
        want.sort();
        assert_eq!(got, want);
        assert_eq!(cali.bans, Bans::List);
        assert_eq!((cali.seats, cali.life, cali.first_player_draws), (2, 30, false));
    }

    #[test]
    fn commander_rejects_the_commander_banned_three_and_accepts_the_tabernacle() {
        let (cmd, _) = formats();
        for n in ["Mana Crypt", "Primeval Titan", "Tinker", "Black Lotus"] {
            assert_eq!(illegal(&cmd, n, "banned"), [Illegal::Banned], "{n}");
        }
        assert!(illegal(&cmd, "The Tabernacle at Pendrell Vale", "legal").is_empty());
    }

    #[test]
    fn cali_accepts_the_three_and_rejects_all_18() {
        let (_, cali) = formats();
        // Scryfall calls the three banned in Commander: the Cali list replaces that
        for n in ["Mana Crypt", "Primeval Titan", "Tinker"] {
            assert!(illegal(&cali, n, "banned").is_empty(), "{n}");
        }
        assert!(illegal(&cali, "Sol Ring", "legal").is_empty());
        for n in CALI_BANNED {
            // banned on the list whether Scryfall calls it legal (the Tabernacle) or banned (Black Lotus)
            for scry in ["legal", "banned"] {
                assert_eq!(illegal(&cali, n, scry), [Illegal::Banned], "{n} {scry}");
            }
            assert_eq!(illegal(&cali, &n.to_uppercase(), "legal"), [Illegal::Banned], "{n} upper case");
        }
        // the three are Cali-legal, so a deck holding them passes there and fails under Commander
        let (cmd, _) = formats();
        let cards: Vec<String> = ["Mana Crypt", "Primeval Titan", "Tinker"].map(String::from).to_vec();
        let recs = |n: &str| Some(rec(n, "banned"));
        let (p, _) = card_problems(&cali, &cards, &[], None, None, &recs);
        assert!(p.is_empty(), "{p:?}");
        let (p, _) = card_problems(&cmd, &cards, &[], None, None, &recs);
        assert_eq!(p, ["banned in Commander: Mana Crypt, Primeval Titan, Tinker"]);
    }

    #[test]
    fn a_card_scryfall_calls_not_legal_is_refused_in_both_formats() {
        let (cmd, cali) = formats();
        for scry in ["not_legal", "restricted"] {
            assert_eq!(illegal(&cmd, "Some Joke Card", scry), [Illegal::NotLegal(format!("Scryfall {scry}"))]);
            assert_eq!(illegal(&cali, "Some Joke Card", scry), [Illegal::NotLegal(format!("Scryfall {scry}"))]);
        }
        let cards = vec!["Some Joke Card".to_string()];
        let recs = |n: &str| Some(rec(n, "not_legal"));
        for f in [&cmd, &cali] {
            let (p, _) = card_problems(f, &cards, &[], None, None, &recs);
            assert_eq!(p, [format!("not legal in {}: Some Joke Card (Scryfall not_legal)", f.name)]);
        }
        // a record with no legality field: Commander (as before) lets it through, Cali needs the data
        let bare = jo! {"name" => "Bare Card", "color_identity" => J::Arr(vec![])};
        assert!(legality_problems(&cmd, "Bare Card", Some(&bare)).is_empty());
        assert_eq!(legality_problems(&cali, "Bare Card", Some(&bare)), [Illegal::NotLegal("no legality data".into())]);
    }

    #[test]
    fn erayos_flip_card_is_refused_in_cali_by_any_of_its_names() {
        let (_, cali) = formats();
        let erayo = jo! {"name" => "Erayo, Soratami Ascendant // Erayo's Essence", "layout" => "flip",
        "color_identity" => J::Arr(vec![J::from("U")]),
        "legalities" => jo!{"commander" => "banned"},
        "card_faces" => J::Arr(vec![
            jo!{"name" => "Erayo, Soratami Ascendant"}, jo!{"name" => "Erayo's Essence"}])};
        // the list names the front face, the whole card or the back face
        for entry in ["Erayo, Soratami Ascendant", "Erayo's Essence"] {
            let mut fmt = cali.clone();
            fmt.banned = vec![entry.to_string()];
            for asked in ["Erayo, Soratami Ascendant", "Erayo, Soratami Ascendant // Erayo's Essence"] {
                assert_eq!(legality_problems(&fmt, asked, Some(&erayo)), [Illegal::Banned], "{entry} / {asked}");
            }
        }
        // an entry written as the full name is front-faced too
        let mut fmt = cali.clone();
        fmt.banned = vec!["Erayo, Soratami Ascendant // Erayo's Essence".into()];
        assert_eq!(legality_problems(&fmt, "Erayo, Soratami Ascendant", Some(&erayo)), [Illegal::Banned]);
        // the shipped list catches it too
        assert_eq!(legality_problems(&cali, "Erayo, Soratami Ascendant", Some(&erayo)), [Illegal::Banned]);
    }

    #[test]
    fn a_double_faced_card_is_matched_by_its_front_face() {
        let (_, cali) = formats();
        let mut fmt = cali.clone();
        fmt.banned = vec!["Leech Collector".into()];
        let dfc = jo! {"name" => "Leech Collector // Bloodletting", "color_identity" => J::Arr(vec![]),
        "legalities" => jo!{"commander" => "legal"}, "card_faces" => J::Arr(vec![
            jo!{"name" => "Leech Collector"}, jo!{"name" => "Bloodletting"}])};
        assert_eq!(legality_problems(&fmt, "Leech Collector // Bloodletting", Some(&dfc)), [Illegal::Banned]);
        // a card that merely shares a word is not caught
        assert!(illegal(&fmt, "Leech Collector Two", "legal").is_empty());
    }

    #[test]
    fn a_banned_name_with_no_record_is_reported_banned_and_not_found() {
        let (cmd, cali) = formats();
        let cards = vec!["Mox Jet".to_string()];
        let (p, _) = card_problems(&cali, &cards, &[], None, None, &|_| None);
        assert!(p.contains(&"not found on Scryfall: Mox Jet".to_string()), "{p:?}");
        assert!(p.contains(&"banned in Cali Commander: Mox Jet".to_string()), "{p:?}");
        // Commander has no list to check a missing card against
        let (p, _) = card_problems(&cmd, &cards, &[], None, None, &|_| None);
        assert_eq!(p, ["not found on Scryfall: Mox Jet"]);
    }

    #[test]
    fn the_commander_message_is_unchanged() {
        let (cmd, _) = formats();
        let cards = vec!["Black Lotus".to_string(), "Sol Ring".to_string()];
        let recs = |n: &str| Some(rec(n, if n == "Black Lotus" { "banned" } else { "legal" }));
        let (p, _) = card_problems(&cmd, &cards, &[], None, None, &recs);
        assert_eq!(p, ["banned in Commander: Black Lotus"]);
    }
}
