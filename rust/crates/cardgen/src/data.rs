//! The hand-edited data files in `data/`: `carddb.txt`, `card_overrides.toml`, `my_decks.toml`, `python_hooks.txt`.

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
                return Err("not inside the repository (no data/carddb.txt above the current directory); use --root".into());
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

/// a deck of yours in my_decks.toml
#[derive(Debug, Clone)]
pub struct MyDeck {
    pub key: String,
    /// the deck file under decklists/ ("JD/sephiroth-phyrexian-reanimator.md")
    pub file: String,
    /// the name logs use
    pub name: String,
    pub commander: String,
    pub ident: String,
}

pub fn load_my_decks(p: &Project) -> Result<Vec<MyDeck>, String> {
    let path = p.data("my_decks.toml");
    let t: toml::Table = read(&path)?.parse().map_err(|e| format!("{}: {e}", path.display()))?;
    let mut out = vec![];
    for (key, v) in &t {
        let get = |f: &str| -> Result<String, String> {
            v.get(f).and_then(|x| x.as_str()).map(|s| s.to_string()).ok_or(format!("{} [{key}]: missing {f}", path.display()))
        };
        out.push(MyDeck { key: key.clone(), file: get("file")?, name: get("name")?, commander: get("commander")?, ident: get("ident")? });
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
        out.insert(f[0].to_string(), Hooks { events: f[1].split_whitespace().map(|s| s.to_string()).collect(), prio: f[2] == "prio" });
    }
    Ok(out)
}
