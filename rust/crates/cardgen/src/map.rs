//! `cardgen map`: the whole-catalog summary (layer 1 of documents/card-knowledge.md), about 8k tokens.
//!
//! Counts of Commander-legal cards per colour identity and per role by colour, role coverage and review status,
//! the biggest creature types, subtypes and keywords, the Game Changers and the banned cards, the combos known per
//! deck and the commands to go further. Deterministic: no timestamps, only the bulk file's `updated_at`. It works with
//! whichever role table is present (rules-only or merged) and never needs the LLM.

use crate::data::Project;
use crate::digest;
use crate::knowledge::{self, Card, Index, Legal, T_CREATURE};
use crate::roles::{self, IDS};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// the hard size limit of `map.md`, in bytes
pub const MAX_BYTES: usize = 32 * 1024;
/// rows of the biggest-creature-types, biggest-subtypes and keyword lists
pub const TOP_TYPES: usize = 40;
pub const TOP_KEYWORDS: usize = 60;

fn num(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// A card the digest and `find` cover: Commander-legal, a deck card (not a Sticker or Attraction), not a meld result.
fn covered(c: &Card) -> bool {
    c.legal == Legal::Legal && roles::is_deck_card(c) && !c.is_meld()
}

/// the colour column of a card: W U B R G for a mono-colour card, C for colourless, M for two or more colours
fn colour_col(ident: u8) -> usize {
    match ident.count_ones() {
        0 => 5,
        1 => ident.trailing_zeros() as usize,
        _ => 6,
    }
}

/// the biggest entries first, ties by name
fn top(mut v: Vec<(String, usize)>, n: usize) -> Vec<(String, usize)> {
    v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    v.truncate(n);
    v
}

fn counted(v: &[(String, usize)]) -> String {
    v.iter().map(|(n, c)| format!("{n} {c}")).collect::<Vec<_>>().join(", ")
}

/// per deck file of `data/spellbook/`: (deck, identity, fetched, included, included by changing commanders, almost)
fn spellbook(dir: &Path) -> Result<Vec<[String; 6]>, String> {
    let mut files: Vec<PathBuf> = vec![];
    for e in std::fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let p = e.map_err(|e| e.to_string())?.path();
        if p.extension().is_some_and(|x| x == "json") {
            files.push(p);
        }
    }
    files.sort();
    let mut rows = vec![];
    for f in files {
        let text = std::fs::read_to_string(&f).map_err(|e| format!("{}: {e}", f.display()))?;
        let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", f.display()))?;
        let count = |k: &str| v.get(k).and_then(|a| a.as_array()).map_or(0, Vec::len).to_string();
        let s = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("-").to_string();
        let stem = f.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let deck = Some(s("deck")).filter(|d| d != "-").unwrap_or(stem);
        rows.push([
            deck,
            s("identity"),
            s("fetched"),
            count("included"),
            count("included_by_changing_commanders"),
            count("almost_included"),
        ]);
    }
    Ok(rows)
}

/// Builds the map text. `spellbook_dir` is `data/spellbook/` (the combos section says so when it is absent).
pub fn build(index: &Index, table: &digest::Table, spellbook_dir: Option<&Path>) -> Result<String, String> {
    if index.roles(0).is_none() {
        return Err("the catalog has no role table attached; run `cardgen roles --rules`".into());
    }
    let merged = table.merged;
    let mut out = String::new();
    let w = &mut out;
    let all = index.cards.len();
    let by_legal = |l: Legal| index.cards.iter().filter(|c| c.legal == l).count();
    let legal = by_legal(Legal::Legal);
    let cover: Vec<(u32, &Card)> =
        index.cards.iter().enumerate().filter(|(_, c)| covered(c)).map(|(i, c)| (i as u32, c)).collect();
    let meld = index.cards.iter().filter(|c| c.legal == Legal::Legal && c.is_meld()).count();
    let supplemental = index.cards.iter().filter(|c| c.legal == Legal::Legal && !roles::is_deck_card(c)).count();
    let gcs = index.cards.iter().filter(|c| c.raw.game_changer).count();

    let _ = writeln!(w, "# Card catalog map\n");
    let _ = writeln!(w, "Source: Scryfall Oracle Cards bulk file, updated_at {}.", index.bulk_updated_at);
    let _ = writeln!(
        w,
        "{} cards: {} Commander-legal, {} banned, {} not legal{}; {} Game Changers.",
        num(all),
        num(legal),
        num(by_legal(Legal::Banned)),
        num(by_legal(Legal::NotLegal)),
        match by_legal(Legal::Restricted) + by_legal(Legal::Unknown) {
            0 => String::new(),
            n => format!(", {} other", num(n)),
        },
        gcs
    );
    let _ = writeln!(
        w,
        "`find` and `digest` cover {} legal cards: {meld} meld results (`find --meld` shows them) and {supplemental} \
         Stickers/Attractions are left out. Every count below is over those {}.",
        num(cover.len()),
        num(cover.len())
    );
    let _ = writeln!(w, "Role table: {}.\n", table.kind());

    // colour identities
    let mut idents: BTreeMap<u8, usize> = BTreeMap::new();
    for (_, c) in &cover {
        *idents.entry(c.ident).or_default() += 1;
    }
    let mut rows: Vec<(u8, usize)> = idents.into_iter().collect();
    rows.sort_by_key(|(bits, _)| (bits.count_ones(), *bits));
    let _ = writeln!(
        w,
        "## Colour identities ({} that occur; a card fits a deck whose identity contains its own)\n",
        rows.len()
    );
    let _ = writeln!(w, "| identity | cards |\n|---|---|");
    for (bits, n) in &rows {
        let label = if *bits == 0 { "C".to_string() } else { knowledge::ident_string(*bits) };
        let _ = writeln!(w, "| {label} | {} |", num(*n));
    }

    // roles by colour
    let _ = writeln!(
        w,
        "\n## Roles by colour (cards that have the role, any position; M = two or more colours; \
         primary = cards whose first role it is)\n"
    );
    let _ =
        writeln!(w, "| role | W | U | B | R | G | C | M | all | primary |\n|---|---|---|---|---|---|---|---|---|---|");
    let mut grid: BTreeMap<&str, [usize; 9]> = BTreeMap::new();
    let mut none = [0usize; 9];
    for (i, c) in &cover {
        let col = colour_col(c.ident);
        let a = index.roles(*i).expect("roles are attached");
        if a.roles.is_empty() {
            none[col] += 1;
            none[7] += 1;
        }
        for r in &a.roles {
            let row = grid.entry(r.as_str()).or_default();
            row[col] += 1;
            row[7] += 1;
            if *r == a.primary {
                row[8] += 1;
            }
        }
    }
    for r in IDS {
        let row = grid.get(r).copied().unwrap_or_default();
        let _ = writeln!(w, "| {r} | {} |", row.iter().map(|n| n.to_string()).collect::<Vec<_>>().join(" | "));
    }
    let _ =
        writeln!(w, "| (no role) | {} | - |", none[..8].iter().map(|n| n.to_string()).collect::<Vec<_>>().join(" | "));

    // coverage and review status
    let (mut with_role, mut rules_alone, mut routed, mut answered, mut unreviewed) = (0, 0, 0, 0, 0);
    for (i, _) in &cover {
        let a = index.roles(*i).expect("roles are attached");
        with_role += usize::from(!a.roles.is_empty());
        match digest::review(a, merged) {
            digest::Review::Llm => {
                routed += 1;
                answered += 1;
            }
            digest::Review::Uncertain => {
                routed += 1;
                unreviewed += 1;
            }
            _ => rules_alone += 1,
        }
    }
    let _ = writeln!(w, "\n## Role coverage and review status\n");
    let _ = writeln!(
        w,
        "- {} of {} cards have at least one role; {} have none.",
        num(with_role),
        num(cover.len()),
        num(cover.len() - with_role)
    );
    let _ = writeln!(
        w,
        "- {} cards were decided by the rules alone (digest flag `r:rules`, or `r:none` for no role).",
        num(rules_alone)
    );
    let _ = writeln!(
        w,
        "- {} cards need the LLM (`needs_llm`): {} have an LLM answer merged (`r:llm`), {} are unreviewed \
         (`r:uncertain`: their roles may be incomplete or wrong).",
        num(routed),
        num(answered),
        num(unreviewed)
    );

    // types and keywords
    let counts = |keep: &dyn Fn(&Card) -> bool| -> Vec<(String, usize)> {
        let mut m: BTreeMap<String, usize> = BTreeMap::new();
        for c in index.cards.iter().filter(|c| keep(c)) {
            for w in c.subtypes() {
                *m.entry(w.to_string()).or_default() += 1;
            }
        }
        m.into_iter().collect()
    };
    let creature = counts(&|c| covered(c) && c.types & T_CREATURE != 0);
    let subs = counts(&covered);
    let _ = writeln!(w, "\n## Creature types ({} in all; the {TOP_TYPES} biggest, cards)\n", creature.len());
    let _ = writeln!(w, "{}", counted(&top(creature, TOP_TYPES)));
    let _ = writeln!(w, "\n## Subtypes of every kind ({} in all; the {TOP_TYPES} biggest, cards)\n", subs.len());
    let _ = writeln!(w, "{}", counted(&top(subs, TOP_TYPES)));
    let mut kw: BTreeMap<&str, usize> = BTreeMap::new();
    for (_, c) in &cover {
        let mut seen: Vec<&str> = vec![];
        for k in &c.raw.keywords {
            if !seen.contains(&k.as_str()) {
                seen.push(k);
                *kw.entry(k).or_default() += 1;
            }
        }
    }
    let _ = writeln!(w, "\n## Keywords ({} in all; the {TOP_KEYWORDS} biggest, cards)\n", kw.len());
    let kw_rows = top(kw.iter().map(|(k, n)| (k.to_string(), *n)).collect(), TOP_KEYWORDS);
    let _ = writeln!(w, "{}", counted(&kw_rows));

    // lists
    let names = |pred: &dyn Fn(&Card) -> bool| {
        let mut v: Vec<&str> = index.cards.iter().filter(|c| pred(c)).map(|c| c.raw.name.as_str()).collect();
        v.sort();
        v.dedup();
        v.join("; ")
    };
    let _ = writeln!(w, "\n## Game Changers ({gcs})\n");
    let _ = writeln!(w, "{}", names(&|c| c.raw.game_changer));
    let _ = writeln!(w, "\n## Banned in Commander ({})\n", by_legal(Legal::Banned));
    let _ = writeln!(w, "{}", names(&|c| c.legal == Legal::Banned));

    // combos
    let _ = writeln!(w, "\n## Combos known per deck (Commander Spellbook, data/spellbook/)\n");
    match spellbook_dir.map(spellbook) {
        Some(Ok(rows)) if !rows.is_empty() => {
            let _ = writeln!(
                w,
                "| deck | identity | fetched | in the list | with another commander | one card away |\n\
                 |---|---|---|---|---|---|"
            );
            for r in &rows {
                let _ = writeln!(w, "| {} |", r.join(" | "));
            }
        }
        Some(Ok(_)) => {
            let _ = writeln!(w, "No deck files in data/spellbook/.");
        }
        Some(Err(e)) => return Err(e),
        None => {
            let _ = writeln!(w, "No data/spellbook/ directory.");
        }
    }

    // how to use
    let _ = writeln!(w, "\n## How to use\n");
    for line in HOW_TO_USE {
        let _ = writeln!(w, "{line}");
    }
    if out.len() > MAX_BYTES {
        return Err(format!("map is {} bytes, over its {MAX_BYTES} limit", out.len()));
    }
    Ok(out)
}

const HOW_TO_USE: [&str; 5] = [
    "Layers: this map (1), a deck digest (2), one card in full (3), the sim (4).",
    "- Search: `cardgen find --ident WU --cmc '<=2' --type instant --text 'counter target' --role counter --limit 20` \
     (`--deck KEY` for a deck's colours without its cards, `--count`, `--json`).",
    "- Digest of a deck's colours, one line per card: `cardgen digest --deck KEY --stdout` (or read \
     data/knowledge/digests/KEY.txt); one role in full: \
     `cardgen digest --deck KEY --role removal --limit 300 --offset 0 --stdout`; \
     any identity: `cardgen digest --ident WUB --stdout`.",
    "- One card in full: `cardgen find --name \"Sol Ring\" --full` (add `--legal any` for a banned or not-legal card).",
    "- The digest `r:` flag says how a card's roles were decided: `r:rules`, `r:llm`, `r:uncertain` (not reviewed \
     yet), \
     `r:none`. Roles say what a card does, not whether it is good in a deck: the sim decides (see AGENTS.md).",
];

pub const USAGE: &str = "usage: cardgen map [--out PATH] [--stdout] [--data-dir DIR]";

/// What the command did.
#[derive(Debug)]
pub struct Output {
    pub text: String,
    pub path: PathBuf,
}

impl Output {
    pub fn summary(&self) -> String {
        format!("map: {} bytes, ~{} tokens -> {}", self.text.len(), digest::tokens(&self.text), self.path.display())
    }
}

/// `cardgen map`: writes the map (default `<knowledge>/map.md`) and returns it. Read-only except for that file.
/// Returns whether `--stdout` was given with the output; arguments are `--out PATH`, `--stdout`, `--data-dir DIR`.
pub fn command(project: &Project, args: &[String]) -> Result<(Output, bool), String> {
    let (mut out_path, mut data_dir, mut stdout) = (None, None, false);
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--stdout" => stdout = true,
            f @ ("--out" | "--data-dir") => {
                i += 1;
                let Some(v) = args.get(i).filter(|v| !v.starts_with("--")) else {
                    return Err(format!("{f} needs a value\n{USAGE}"));
                };
                if f == "--out" { out_path = Some(PathBuf::from(v)) } else { data_dir = Some(PathBuf::from(v)) }
            }
            other => return Err(format!("unknown argument '{other}'\n{USAGE}")),
        }
        i += 1;
    }
    let (index, table) = digest::open(project, data_dir.as_deref())?;
    let spell = project.data("spellbook");
    let text = build(&index, &table, spell.is_dir().then_some(spell.as_path()))?;
    let path = out_path.unwrap_or_else(|| digest::out_dir(project, data_dir.as_deref()).join("map.md"));
    digest::write_file(&path, &text)?;
    Ok((Output { text, path }, stdout))
}
