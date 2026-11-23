//! The hand-edited data files in `data/`: `carddb.txt`, `card_overrides.toml`, `my_decks.toml`, `formats.toml`,
//! `python_hooks.txt`.

use crate::json::{J, Obj};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// the repository: where `data/` and `decklists/` live
#[derive(Debug, Clone)]
pub struct Project {
    pub root: PathBuf,
}

impl Project {
    pub fn new(root: impl Into<PathBuf>) -> Project {
        Project { root: root.into() }
    }

    /// the repository containing the current directory (the nearest parent with data/carddb.txt)
    pub fn find() -> Result<Project, String> {
        let mut d = std::env::current_dir().map_err(|e| e.to_string())?;
        loop {
            if d.join("data").join("carddb.txt").exists() {
                return Ok(Project { root: d });
            }
            if !d.pop() {
                return Err(
                    "not inside the repository (no data/carddb.txt above the current directory); use --root".into()
                );
            }
        }
    }

    pub fn data(&self, file: &str) -> PathBuf {
        self.root.join("data").join(file)
    }

    pub fn decklists(&self) -> PathBuf {
        self.root.join("decklists")
    }
}

pub fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

/// one `name|types|cost|tags` line of carddb.txt
#[derive(Debug, Clone)]
pub struct CardDbLine {
    pub name: String,
    pub types: String,
    pub cost: String,
    pub tags: String,
}

pub fn load_carddb(p: &Project) -> Result<Vec<CardDbLine>, String> {
    let path = p.data("carddb.txt");
    let mut out = vec![];
    for (i, line) in read(&path)?.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let f: Vec<&str> = line.split('|').map(|x| x.trim()).collect();
        if f.len() != 4 {
            return Err(format!("{}:{}: expected name | types | cost | tags", path.display(), i + 1));
        }
        out.push(CardDbLine { name: f[0].into(), types: f[1].into(), cost: f[2].into(), tags: f[3].into() });
    }
    Ok(out)
}

/// one card's table in card_overrides.toml
#[derive(Debug, Clone, Default)]
pub struct Entry {
    pub types: Option<String>,
    pub cost: Option<String>,
    pub tags: Option<String>,
    /// Some(vec![]) is `dsl = []`
    pub dsl: Option<Vec<Obj>>,
    pub kws: Option<Vec<String>>,
    pub ward: Option<i64>,
    pub loyalty: Option<String>,
    pub add_tags: Vec<String>,
    pub pool_add_tags: Vec<String>,
    pub pool_remove_tags: Vec<String>,
    pub status: Option<String>,
    pub note: Option<String>,
    /// `paths_ok = "why"`: the card runs through two paths (compiled abilities and card code, say) on purpose, for
    /// different parts of the card; `cardgen audit --paths` reports it as a declared split, not a double. Read only by
    /// the audit.
    pub paths_ok: Option<String>,
}

impl Entry {
    /// does it override the card's own data (the card is rebuilt from it)?
    pub fn is_override(&self) -> bool {
        self.tags.is_some() || self.dsl.is_some() || self.types.is_some() || self.cost.is_some()
    }
}

fn to_j(v: &toml::Value) -> J {
    match v {
        toml::Value::String(s) => J::Str(s.clone()),
        toml::Value::Integer(i) => J::Int(*i),
        toml::Value::Float(f) => J::Float(*f),
        toml::Value::Boolean(b) => J::Bool(*b),
        toml::Value::Datetime(d) => J::Str(d.to_string()),
        toml::Value::Array(a) => J::Arr(a.iter().map(to_j).collect()),
        toml::Value::Table(t) => J::Obj(Obj(t.iter().map(|(k, v)| (k.clone(), to_j(v))).collect())),
    }
}

fn str_list(v: &toml::Value, what: &str) -> Result<Vec<String>, String> {
    v.as_array()
        .ok_or(format!("{what}: expected a list"))?
        .iter()
        .map(|x| x.as_str().map(|s| s.to_string()).ok_or(format!("{what}: expected strings")))
        .collect()
}

pub fn load_overrides(p: &Project) -> Result<Vec<(String, Entry)>, String> {
    let path = p.data("card_overrides.toml");
    let t: toml::Table = read(&path)?.parse().map_err(|e| format!("{}: {e}", path.display()))?;
    let mut out = vec![];
    for (name, v) in &t {
        let tab = v.as_table().ok_or(format!("{name}: expected a table"))?;
        let mut e = Entry::default();
        for (k, v) in tab {
            let ctx = format!("{} [{name}] {k}", path.display());
            let s = || v.as_str().map(|x| x.to_string()).ok_or(format!("{ctx}: expected a string"));
            match k.as_str() {
                "types" => e.types = Some(s()?),
                "cost" => e.cost = Some(s()?),
                "tags" => e.tags = Some(s()?),
                "loyalty" => e.loyalty = Some(s()?),
                "status" => e.status = Some(s()?),
                "note" => e.note = Some(s()?),
                "paths_ok" => e.paths_ok = Some(s()?),
                "ward" => e.ward = Some(v.as_integer().ok_or(format!("{ctx}: expected a number"))?),
                "kws" => e.kws = Some(str_list(v, &ctx)?),
                "add_tags" => e.add_tags = str_list(v, &ctx)?,
                "pool_add_tags" => e.pool_add_tags = str_list(v, &ctx)?,
                "pool_remove_tags" => e.pool_remove_tags = str_list(v, &ctx)?,
                "dsl" => {
                    let J::Arr(items) = to_j(v) else { return Err(format!("{ctx}: expected a list of abilities")) };
                    e.dsl = Some(
                        items
                            .into_iter()
                            .map(|x| match x {
                                J::Obj(o) => Ok(o),
                                _ => Err(format!("{ctx}: an ability must be a table")),
                            })
                            .collect::<Result<_, _>>()?,
                    );
                }
                other => return Err(format!("{ctx}: unknown key {other:?}")),
            }
        }
        out.push((name.clone(), e));
    }
    Ok(out)
}

/// the format a deck plays when `my_decks.toml` names none
pub const DEFAULT_FORMAT: &str = "commander";

/// how a format decides which cards are banned
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bans {
    /// Scryfall's `legalities.commander`: `banned` and every value but `legal` are refused
    Scryfall,
    /// the format's own `banned` list replaces Commander's ban list
    List,
}

impl Bans {
    pub fn as_str(self) -> &'static str {
        match self {
            Bans::Scryfall => "scryfall",
            Bans::List => "list",
        }
    }
}

/// a game format in data/formats.toml
#[derive(Debug, Clone, PartialEq)]
pub struct FormatDef {
    pub key: String,
    /// what messages call it ("Cali Commander")
    pub name: String,
    /// 2 or 4: the table shapes the pod builders support
    pub seats: i64,
    pub life: i64,
    /// false: the starting player skips its first draw step
    pub first_player_draws: bool,
    pub bans: Bans,
    /// the ban list when `bans` is `List`, one name per entry (a front-face name)
    pub banned: Vec<String>,
}

impl FormatDef {
    /// Commander as the engine has always played it (what `[commander]` in formats.toml must say)
    pub fn commander() -> FormatDef {
        FormatDef {
            key: DEFAULT_FORMAT.into(),
            name: "Commander".into(),
            seats: 4,
            life: 40,
            first_player_draws: true,
            bans: Bans::Scryfall,
            banned: vec![],
        }
    }
}

/// a card name's front face, lower case: how ban lists and the Scryfall cache compare names
pub fn front_face(name: &str) -> String {
    name.split(" // ").next().unwrap_or("").trim().to_lowercase()
}

const FORMAT_KEYS: [&str; 6] = ["name", "seats", "life", "first_player_draws", "bans", "banned"];

/// the formats of formats.toml, in file order (`src` is the text, `path` only names it in errors)
pub fn parse_formats(src: &str, path: &Path) -> Result<Vec<FormatDef>, String> {
    let t: toml::Table = src.parse().map_err(|e| format!("{}: {e}", path.display()))?;
    let mut out: Vec<FormatDef> = vec![];
    for (key, v) in &t {
        let ctx = format!("{} [{key}]", path.display());
        let tbl = v.as_table().ok_or(format!("{ctx}: a format must be a table"))?;
        if let Some(k) = tbl.keys().find(|k| !FORMAT_KEYS.contains(&k.as_str())) {
            return Err(format!("{ctx}: unknown key {k:?} (one of {})", FORMAT_KEYS.join(", ")));
        }
        let get = |f: &str| tbl.get(f).ok_or(format!("{ctx}: missing {f}"));
        let name = get("name")?.as_str().filter(|s| !s.trim().is_empty()).ok_or(format!("{ctx}: name must be text"))?;
        let int = |f: &str| get(f)?.as_integer().ok_or(format!("{ctx}: {f} must be a whole number"));
        let (seats, life) = (int("seats")?, int("life")?);
        if seats != 2 && seats != 4 {
            return Err(format!("{ctx}: seats must be 2 or 4 (the pod builders' table shapes), not {seats}"));
        }
        if life < 1 {
            return Err(format!("{ctx}: life must be at least 1, not {life}"));
        }
        let first_player_draws =
            get("first_player_draws")?.as_bool().ok_or(format!("{ctx}: first_player_draws must be true or false"))?;
        let bans = match get("bans")?.as_str() {
            Some("scryfall") => Bans::Scryfall,
            Some("list") => Bans::List,
            _ => return Err(format!("{ctx}: bans must be \"scryfall\" or \"list\"")),
        };
        let mut banned: Vec<String> = vec![];
        if let Some(b) = tbl.get("banned") {
            for n in b.as_array().ok_or(format!("{ctx}: banned must be a list of names"))? {
                let n = n.as_str().ok_or(format!("{ctx}: banned must be a list of names"))?;
                if n.trim().is_empty() {
                    return Err(format!("{ctx}: a banned name is empty"));
                }
                if n.contains("//") {
                    return Err(format!("{ctx}: banned name {n:?} has `//`: list the front face only"));
                }
                if banned.iter().any(|o| front_face(o) == front_face(n)) {
                    return Err(format!("{ctx}: banned name {n:?} is listed twice"));
                }
                banned.push(n.trim().to_string());
            }
        }
        match bans {
            Bans::List if banned.is_empty() => {
                return Err(format!("{ctx}: bans = \"list\" needs a non-empty banned list"));
            }
            Bans::Scryfall if !banned.is_empty() => {
                return Err(format!("{ctx}: banned only applies to bans = \"list\" (this format uses Scryfall's)"));
            }
            _ => {}
        }
        out.push(FormatDef {
            key: key.clone(),
            name: name.trim().to_string(),
            seats,
            life,
            first_player_draws,
            bans,
            banned,
        });
    }
    // [commander] stays what the engine plays: editing the data cannot change every 4-player game
    match out.iter().find(|f| f.key == DEFAULT_FORMAT) {
        None => return Err(format!("{}: no [{DEFAULT_FORMAT}] format", path.display())),
        Some(c) => {
            let rules = sim_core::format::Rules::COMMANDER;
            let same = c.seats == 4
                && c.life == rules.life as i64
                && c.first_player_draws == rules.first_player_draws
                && c.bans == Bans::Scryfall;
            if !same {
                return Err(format!(
                    "{} [{DEFAULT_FORMAT}]: must stay 4 seats, {} life, first player draws, bans = \"scryfall\" \
                     (the engine's Commander)",
                    path.display(),
                    rules.life
                ));
            }
        }
    }
    Ok(out)
}

pub fn load_formats(p: &Project) -> Result<Vec<FormatDef>, String> {
    let path = p.data("formats.toml");
    parse_formats(&read(&path)?, &path)
}

/// a deck of yours in my_decks.toml
#[derive(Debug, Clone)]
pub struct MyDeck {
    pub key: String,
    /// the deck file under decklists/ ("JD/ghave-abzan-aristocrats.md")
    pub file: String,
    /// the name logs use
    pub name: String,
    pub commander: String,
    /// a second commander with plain Partner (decks with two commanders only)
    pub commander2: Option<String>,
    pub ident: String,
    /// the opponent tiers it is simulated against ("t1".."t5"), at least one
    pub tiers: Vec<String>,
    /// the key of its format in formats.toml ("commander" when my_decks.toml names none)
    pub format: String,
}

const MY_DECK_KEYS: [&str; 7] = ["file", "name", "commander", "commander2", "ident", "tiers", "format"];

pub fn load_my_decks(p: &Project) -> Result<Vec<MyDeck>, String> {
    let path = p.data("my_decks.toml");
    let t: toml::Table = read(&path)?.parse().map_err(|e| format!("{}: {e}", path.display()))?;
    let formats = load_formats(p)?;
    let mut out = vec![];
    for (key, v) in &t {
        // a misspelt field (`fromat`) would otherwise be ignored and the deck would play Commander
        let tbl = v.as_table().ok_or(format!("{} [{key}]: a deck must be a table", path.display()))?;
        if let Some(k) = tbl.keys().find(|k| !MY_DECK_KEYS.contains(&k.as_str())) {
            return Err(format!("{} [{key}]: unknown key {k:?} (one of {})", path.display(), MY_DECK_KEYS.join(", ")));
        }
        let get = |f: &str| -> Result<String, String> {
            v.get(f)
                .and_then(|x| x.as_str())
                .map(|s| s.to_string())
                .ok_or(format!("{} [{key}]: missing {f}", path.display()))
        };
        let format = match v.get("format") {
            None => DEFAULT_FORMAT.to_string(),
            Some(f) => {
                f.as_str().map(str::to_string).ok_or(format!("{} [{key}]: format must be text", path.display()))?
            }
        };
        if !formats.iter().any(|f| f.key == format) {
            return Err(format!(
                "{} [{key}]: unknown format {format:?} (one of {}; data/formats.toml)",
                path.display(),
                formats.iter().map(|f| f.key.as_str()).collect::<Vec<_>>().join(", ")
            ));
        }
        out.push(MyDeck {
            key: key.clone(),
            file: get("file")?,
            name: get("name")?,
            commander: get("commander")?,
            commander2: v.get("commander2").and_then(|x| x.as_str()).map(|s| s.to_string()),
            ident: get("ident")?,
            tiers: get_tiers(v, key, &path)?,
            format,
        });
    }
    Ok(out)
}

/// a card's Python hooks, from the frozen record
#[derive(Debug, Clone, Default)]
pub struct Hooks {
    pub events: Vec<String>,
    pub prio: bool,
}

pub fn load_hooks(p: &Project) -> Result<BTreeMap<String, Hooks>, String> {
    let mut out = BTreeMap::new();
    for line in read(&p.data("python_hooks.txt"))?.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let f: Vec<&str> = line.split('|').collect();
        if f.len() != 3 {
            return Err(format!("python_hooks.txt: bad line {line:?}"));
        }
        out.insert(
            f[0].to_string(),
            Hooks { events: f[1].split_whitespace().map(|s| s.to_string()).collect(), prio: f[2] == "prio" },
        );
    }
    Ok(out)
}

/// the tiers a deck of yours is simulated against
pub const TIERS: [&str; 7] = ["t0", "t1", "t2", "t3", "t4", "t5", "t6"];

fn get_tiers(v: &toml::Value, key: &str, path: &std::path::Path) -> Result<Vec<String>, String> {
    let arr = v
        .get("tiers")
        .and_then(|x| x.as_array())
        .ok_or(format!("{} [{key}]: missing tiers (a list such as [\"t1\", \"t2\"])", path.display()))?;
    let mut out: Vec<String> = vec![];
    for t in arr {
        let t = t.as_str().ok_or(format!("{} [{key}]: tiers must be strings", path.display()))?;
        if !TIERS.contains(&t) {
            return Err(format!("{} [{key}]: unknown tier {t:?} (one of {})", path.display(), TIERS.join(", ")));
        }
        if out.iter().any(|o| o == t) {
            return Err(format!("{} [{key}]: tier {t} listed twice", path.display()));
        }
        out.push(t.to_string());
    }
    if out.is_empty() {
        return Err(format!("{} [{key}]: tiers is empty", path.display()));
    }
    Ok(out)
}

#[cfg(test)]
mod tier_tests {
    use super::*;

    fn tiers(src: &str) -> Result<Vec<String>, String> {
        let t: toml::Table = src.parse().unwrap();
        get_tiers(&t["d"], "d", std::path::Path::new("my_decks.toml"))
    }

    #[test]
    fn tiers_are_required_known_and_unique() {
        assert_eq!(tiers("[d]\ntiers = [\"t1\", \"t3\"]").unwrap(), ["t1", "t3"]);
        assert!(tiers("[d]\nname = \"x\"").unwrap_err().contains("missing tiers"));
        assert!(tiers("[d]\ntiers = []").unwrap_err().contains("empty"));
        assert!(tiers("[d]\ntiers = [\"t7\"]").unwrap_err().contains("unknown tier"));
        assert!(tiers("[d]\ntiers = [\"t1\", \"t1\"]").unwrap_err().contains("twice"));
        assert!(tiers("[d]\ntiers = [1]").unwrap_err().contains("strings"));
    }
}

#[cfg(test)]
mod format_tests {
    use super::*;

    const COMMANDER: &str =
        "[commander]\nname = \"Commander\"\nseats = 4\nlife = 40\nfirst_player_draws = true\nbans = \"scryfall\"\n";

    fn parse(extra: &str) -> Result<Vec<FormatDef>, String> {
        parse_formats(&format!("{COMMANDER}{extra}"), Path::new("formats.toml"))
    }

    fn list(fields: &str) -> String {
        format!("[x]\nname = \"X\"\nseats = 2\nlife = 30\nfirst_player_draws = false\nbans = \"list\"\n{fields}")
    }

    #[test]
    fn the_shipped_formats_load_and_commander_is_what_the_engine_plays() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let f = load_formats(&Project::new(root)).unwrap();
        assert_eq!(f.iter().map(|f| f.key.as_str()).collect::<Vec<_>>(), ["commander", "cali"]);
        assert_eq!(f[0], FormatDef::commander());
        assert_eq!(
            (f[1].name.as_str(), f[1].seats, f[1].life, f[1].first_player_draws),
            ("Cali Commander", 2, 30, false)
        );
        assert_eq!(f[1].banned.len(), 18);
    }

    #[test]
    fn a_list_format_needs_a_list_and_a_scryfall_format_has_none() {
        let ok = parse(&list("banned = [\"A\", \"B // C\"]\n").replace("B // C", "B")).unwrap();
        assert_eq!(ok[1].banned, ["A", "B"]);
        assert!(parse(&list("")).unwrap_err().contains("non-empty banned list"));
        assert!(parse(&list("banned = []\n")).unwrap_err().contains("non-empty banned list"));
        assert!(
            parse(&list("banned = [\"A\", \"a\"]\n")).unwrap_err().contains("listed twice"),
            "duplicates compare by lower-case front face"
        );
        assert!(parse(&list("banned = [\"A // B\"]\n")).unwrap_err().contains("front face only"));
        assert!(parse(&list("banned = [\"\"]\n")).unwrap_err().contains("empty"));
        assert!(parse(&list("banned = [1]\n")).unwrap_err().contains("list of names"));
        let scry = list("banned = [\"A\"]\n").replace("\"list\"", "\"scryfall\"");
        assert!(parse(&scry).unwrap_err().contains("only applies to bans = \"list\""));
    }

    #[test]
    fn the_numbers_and_names_are_checked() {
        let with = |from: &str, to: &str| parse(&list("banned = [\"A\"]\n").replace(from, to));
        assert!(with("seats = 2", "seats = 3").unwrap_err().contains("seats must be 2 or 4"));
        assert!(with("seats = 2", "seats = 1").unwrap_err().contains("seats must be 2 or 4"));
        assert!(with("seats = 2", "seats = 4").is_ok());
        assert!(with("life = 30", "life = 0").unwrap_err().contains("life must be at least 1"));
        assert!(with("life = 30", "life = 1").is_ok());
        assert!(with("bans = \"list\"", "bans = \"all\"").unwrap_err().contains("bans must be"));
        assert!(with("bans = \"list\"\n", "").unwrap_err().contains("missing bans"));
        assert!(with("seats = 2", "seat = 2").unwrap_err().contains("unknown key \"seat\""));
        assert!(
            with("first_player_draws = false", "first_player_draws = \"no\"").unwrap_err().contains("true or false")
        );
        assert!(with("name = \"X\"", "name = \" \"").unwrap_err().contains("name must be text"));
    }

    #[test]
    fn commander_cannot_be_edited_away_from_what_the_engine_plays() {
        for (from, to) in [
            ("seats = 4", "seats = 2"),
            ("life = 40", "life = 30"),
            ("first_player_draws = true", "first_player_draws = false"),
        ] {
            let src = COMMANDER.replace(from, to);
            let e = parse_formats(&src, Path::new("formats.toml")).unwrap_err();
            assert!(e.contains("must stay 4 seats, 40 life"), "{from}: {e}");
        }
        let e = parse_formats(
            "[cali]\nname = \"C\"\nseats = 2\nlife = 30\nfirst_player_draws = false\nbans = \"scryfall\"\n",
            Path::new("f"),
        )
        .unwrap_err();
        assert!(e.contains("no [commander] format"), "{e}");
    }
}
