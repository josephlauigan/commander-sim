//! `cardgen map`: the whole-catalog summary (layer 1 of documents/card-knowledge.md), about 3k tokens and under 32 KB.
//!
//! Counts of Commander-legal cards per colour identity and per role by colour, role coverage and review status, the
//! biggest creature types, subtypes and keywords, the Game Changers and the banned cards, a table of JD's decks, the
//! combos in JD's decks and the commands to go further. Deterministic: no timestamps, only the bulk file's
//! `updated_at`. It works with whichever role table is present (rules-only or merged) and never needs the LLM.

use crate::data::Project;
use crate::digest;
use crate::facts::Facts;
use crate::knowledge::{self, Card, Deck, Index, Legal};
use crate::roles::{self, IDS};
use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

/// the hard size limit of `map.md`, in bytes
pub const MAX_BYTES: usize = 32 * 1024;
/// rows of the biggest-creature-types, biggest-subtypes and keyword lists
pub const TOP_SUBTYPES: usize = 40;
pub const TOP_CREATURE_TYPES: usize = 15;
pub const TOP_KEYWORDS: usize = 15;
/// the most combos listed per deck
const MAX_COMBOS: usize = 10;

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

/// What one `data/spellbook/` file says about a deck: its key, identity and fetch date, the combos in the list (as
/// card names), and the counts of the combos a commander change or one more card would give.
struct Spellbook {
    deck: String,
    fetched: String,
    included: Vec<Vec<String>>,
    by_changing_commanders: usize,
    almost: usize,
}

/// every file of `data/spellbook/`, read and checked (a file that does not parse is an error naming it)
fn spellbook(dir: &Path) -> Result<Vec<Spellbook>, String> {
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
        let count = |k: &str| v.get(k).and_then(|a| a.as_array()).map_or(0, Vec::len);
        let s = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("-").to_string();
        let stem = f.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        let included = v
            .get("included")
            .and_then(|a| a.as_array())
            .into_iter()
            .flatten()
            .map(|combo| {
                combo
                    .get("cards")
                    .and_then(|c| c.as_array())
                    .into_iter()
                    .flatten()
                    .filter_map(|c| c.get("name").and_then(|n| n.as_str()).map(String::from))
                    .collect()
            })
            .collect();
        rows.push(Spellbook {
            deck: Some(s("deck")).filter(|d| d != "-").unwrap_or(stem),
            fetched: s("fetched"),
            included,
            by_changing_commanders: count("included_by_changing_commanders"),
            almost: count("almost_included"),
        });
    }
    Ok(rows)
}

/// A deck of `data/decks.json` `mine` with its opponent tiers.
pub struct MineDeck {
    pub deck: Deck,
    pub tiers: Vec<String>,
}

/// JD's own decks (`mine`) of a `decks.json`
pub fn load_mine(path: &Path) -> Result<Vec<MineDeck>, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let v: serde_json::Value = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut out = vec![];
    for d in v.get("mine").and_then(|m| m.as_array()).into_iter().flatten() {
        let deck: Deck = serde_json::from_value(d.clone()).map_err(|e| format!("{}: {e}", path.display()))?;
        let tiers = d
            .get("tiers")
            .and_then(|t| t.as_array())
            .into_iter()
            .flatten()
            .filter_map(|t| t.as_str().map(String::from))
            .collect();
        out.push(MineDeck { deck, tiers });
    }
    Ok(out)
}

/// the subtypes after the `—` of one face's type line (`Time Lord` is one subtype)
fn face_subtypes(face: &str) -> Vec<String> {
    let Some((_, subs)) = face.split_once('—') else { return vec![] };
    let words: Vec<&str> = subs.split_whitespace().collect();
    let (mut out, mut i) = (vec![], 0);
    while i < words.len() {
        let multi = knowledge::MULTI_WORD_SUBTYPES.iter().find(|m| {
            let parts: Vec<&str> = m.split(' ').collect();
            words[i..].starts_with(&parts)
        });
        match multi {
            Some(m) => {
                out.push(m.to_string());
                i += m.split(' ').count();
            }
            None => {
                out.push(words[i].to_string());
                i += 1;
            }
        }
    }
    out
}

/// The creature types of the cards and how many cards have each: the subtypes of every creature-bearing face (a
/// face with `Creature` or `Kindred` in its type), each counted once per card, leaving out the subtypes that only
/// non-creature faces have (`Equipment`, `Aura`, `Adventure`, the land types of a `Land Creature`).
fn creature_types<'a>(cards: impl Iterator<Item = &'a Card> + Clone) -> Vec<(String, usize)> {
    let faces_of = |c: &Card| -> Vec<String> {
        let mut faces: Vec<String> = c.raw.type_line.split(" // ").map(String::from).collect();
        if faces.len() == 1 {
            let from: Vec<String> =
                c.raw.card_faces.iter().flatten().map(|f| f.type_line.clone()).filter(|t| !t.is_empty()).collect();
            if from.len() > 1 {
                faces = from;
            }
        }
        faces
    };
    let creature_face = |f: &str| {
        let head = f.split('—').next().unwrap_or("").to_lowercase();
        head.split_whitespace().any(|w| ["creature", "kindred", "tribal"].contains(&w))
    };
    let mut other: std::collections::BTreeSet<String> = Default::default();
    for c in cards.clone() {
        for f in faces_of(c).iter().filter(|f| !creature_face(f)) {
            other.extend(face_subtypes(f));
        }
    }
    let mut m: BTreeMap<String, usize> = BTreeMap::new();
    for c in cards {
        let mut own: std::collections::BTreeSet<String> = Default::default();
        for f in faces_of(c).iter().filter(|f| creature_face(f)) {
            own.extend(face_subtypes(f).into_iter().filter(|s| !other.contains(s)));
        }
        for s in own {
            *m.entry(s).or_default() += 1;
        }
    }
    m.into_iter().collect()
}

/// Builds the map text. `spellbook_dir` is `data/spellbook/` (the combos section says so when it is absent); JD's
/// decks come from the `decks.json` beside it.
pub fn build(index: &Index, table: &digest::Table, spellbook_dir: Option<&Path>) -> Result<String, String> {
    let decks_file = spellbook_dir.and_then(Path::parent).map(|p| p.join("decks.json")).filter(|p| p.is_file());
    let decks = decks_file.map(|p| load_mine(&p)).transpose()?.unwrap_or_default();
    build_with(index, table, spellbook_dir, &decks)
}

/// [`build`] with JD's decks given.
pub fn build_with(
    index: &Index,
    table: &digest::Table,
    spellbook_dir: Option<&Path>,
    decks: &[MineDeck],
) -> Result<String, String> {
    build_with_facts(index, table, spellbook_dir, decks, None)
}

/// [`build_with`] with the phase 4 facts (engine model status, findings): adds the engine coverage section and the
/// model columns of JD's decks table. Without them the map has neither.
pub fn build_with_facts(
    index: &Index,
    table: &digest::Table,
    spellbook_dir: Option<&Path>,
    decks: &[MineDeck],
    facts: Option<&Facts>,
) -> Result<String, String> {
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
        "- {} cards were decided by the rules alone, or have no role and nothing to ask (no flag in the digest).",
        num(rules_alone)
    );
    let _ = writeln!(
        w,
        "- {} cards need the LLM (`needs_llm`): {} have an LLM answer merged, {} are unreviewed (the digest \
         flag `?`: their roles may be incomplete or wrong).",
        num(routed),
        num(answered),
        num(unreviewed)
    );

    // engine model coverage
    if let Some(f) = facts {
        let mut by: BTreeMap<&str, usize> = BTreeMap::new();
        let mut none = 0;
        for (i, _) in &cover {
            match f.status_of(*i) {
                Some((s, _)) => {
                    let known = crate::audit::STATUSES.iter().find(|k| **k == s).copied().unwrap_or("other");
                    *by.entry(known).or_default() += 1;
                }
                None => none += 1,
            }
        }
        let _ = writeln!(w, "\n## Engine model coverage (what the simulator models)\n");
        if !f.engine.present {
            let _ = writeln!(w, "No engine data in this project (data/carddb.txt): no card has a status.");
        } else if let Some(why) = &f.engine.unavailable {
            let _ = writeln!(w, "Engine status unavailable: {why}; run cardgen export.");
        } else {
            let n = |k: &str| num(by.get(k).copied().unwrap_or(0));
            let _ = writeln!(w, "| status | cards |\n|---|---|");
            for k in crate::audit::STATUSES {
                let _ = writeln!(w, "| {k} | {} |", n(k));
            }
            if by.contains_key("other") {
                let _ = writeln!(w, "| other | {} |", n("other"));
            }
            let _ = writeln!(w, "| no record | {} |", num(none));
            let _ = writeln!(
                w,
                "\nFrom `cardgen audit` over the cards that have a record in data/cards.json (digest flag `m:`, `find \
                 --json` keys `status` and `engine`). The status is computed from the current data files, which is \
                 what the next `cardgen export` will give; data/cards.json decides which cards have a record. A card \
                 with no record has an unknown status (digest `m:N`, `find --status no-record`): treat it as below \
                 Full. Full is hand-verified; Full-auto is compiled from Oracle text, not hand-verified. \
                 `find --status full,full-auto` = cards the sim models with no known gap (Full-auto is compiled, not \
                 hand-verified); every proposal still needs the three swap checks and a --swap run, and a card below \
                 Full (Full-auto, Approximate, Partial, Unmodeled) or with no record goes to JD first."
            );
        }
    }

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
    let creature = creature_types(index.cards.iter().filter(|c| covered(c)));
    let subs = counts(&covered);
    let _ = writeln!(w, "\n## Creature types ({} in all; the {TOP_CREATURE_TYPES} biggest, cards)\n", creature.len());
    let _ = writeln!(w, "{}", counted(&top(creature, TOP_CREATURE_TYPES)));
    let _ = writeln!(
        w,
        "\n## Subtypes of every kind ({} in all, creature types and the rest; the {TOP_SUBTYPES} biggest, cards)\n",
        subs.len()
    );
    let _ = writeln!(w, "{}", counted(&top(subs, TOP_SUBTYPES)));
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

    // JD's decks
    let _ = writeln!(w, "\n## JD's decks (data/decks.json)\n");
    if decks.is_empty() {
        let _ = writeln!(w, "No decks found (data/decks.json `mine`).");
    } else {
        let _ = writeln!(
            w,
            "| key | commander | identity | tiers | list cards not Commander-legal | cards with each role |{}\n\
             |---|---|---|---|---|---|{}",
            if facts.is_some() { " below Full (f/A/P/U) | f A P U | no record | FINDINGS mentions |" } else { "" },
            if facts.is_some() { "---|---|---|---|" } else { "" }
        );
        for d in decks {
            let own = digest::deck_cards(index, &d.deck);
            let illegal: Vec<&str> = own
                .iter()
                .map(|&i| &index.cards[i as usize])
                .filter(|c| c.legal != Legal::Legal)
                .map(|c| c.raw.name.as_str())
                .collect();
            let commander = match &d.deck.commander2 {
                Some(c2) => format!("{} + {c2}", d.deck.commander),
                None => d.deck.commander.clone(),
            };
            let roles_cell =
                digest::role_counts(index, &own).iter().map(|(r, n)| format!("{r} {n}")).collect::<Vec<_>>().join(", ");
            let model_cells = match facts {
                None => String::new(),
                Some(f) => {
                    let count =
                        |status: &str| own.iter().filter(|&&i| f.status_of(i).is_some_and(|s| s.0 == status)).count();
                    let mentions: usize = own.iter().map(|&i| f.findings_of(i).len()).sum();
                    if f.engine.present && f.engine.unavailable.is_none() {
                        let (fa, a, p, u) =
                            (count("Full-auto"), count("Approximate"), count("Partial"), count("Unmodeled"));
                        let none = own.iter().filter(|&&i| f.status_of(i).is_none()).count();
                        format!(" {} | f{fa} A{a} P{p} U{u} | {none} | {mentions} |", fa + a + p + u)
                    } else {
                        format!(" - | - | - | {mentions} |")
                    }
                }
            };
            let _ = writeln!(
                w,
                "| {} | {commander} | {} | {} | {} | {roles_cell} |{model_cells}",
                d.deck.key,
                d.deck.ident,
                d.tiers.join(" "),
                if illegal.is_empty() {
                    "0".to_string()
                } else {
                    format!("{} ({})", illegal.len(), illegal.join(", "))
                },
            );
        }
    }

    // combos
    let _ = writeln!(w, "\n## Combos in JD's decks (Commander Spellbook, data/spellbook/KEY.json)\n");
    match spellbook_dir.map(spellbook) {
        Some(Ok(rows)) if rows.is_empty() => {
            let _ = writeln!(w, "No deck files in data/spellbook/.");
        }
        Some(Ok(rows)) => {
            let mut any = false;
            for d in decks {
                let Some(r) = rows.iter().find(|r| r.deck == d.deck.key) else { continue };
                any = true;
                let list: Vec<String> = r.included.iter().take(MAX_COMBOS).map(|c| c.join(" + ")).collect();
                let more = if r.included.len() > MAX_COMBOS { "; ..." } else { "" };
                let mut parts = vec![format!(
                    "{} in the list{}",
                    r.included.len(),
                    if list.is_empty() { String::new() } else { format!(": {}{more}", list.join("; ")) }
                )];
                if r.by_changing_commanders > 0 {
                    parts.push(format!("{} more with another commander", r.by_changing_commanders));
                }
                parts.push(format!("{} one card away (see the file)", r.almost));
                let _ = writeln!(w, "- {} (fetched {}): {}.", d.deck.key, r.fetched, parts.join(". "));
            }
            if !any {
                let _ = writeln!(w, "No spellbook file for the decks above.");
            }
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

const HOW_TO_USE: [&str; 6] = [
    "Layers: this map (1), a deck digest (2), one card in full (3), the sim (4).",
    "- Search: `cardgen find --ident WU --cmc '<=2' --type instant --text 'counter target' --role counter --limit 20` \
     (`--deck KEY` for a deck's colours without its cards, `--count`, `--json`).",
    "- Digest of a deck's colours, one line per card: `cardgen digest --deck KEY --stdout` (or read \
     data/knowledge/digests/KEY.txt); every card that has a role: \
     `cardgen digest --deck KEY --role removal --limit 300 --offset 0 --stdout`; every candidate with no role: \
     `cardgen digest --deck KEY --view roleless --stdout`; the new ones: `--view new`; any identity: \
     `cardgen digest --ident WUB --stdout`.",
    "- One card in full: `cardgen find --name \"Sol Ring\" --full` (add `--legal any` for a banned or not-legal card).",
    "- For a swap: take the roles of the card to cut (section A of the deck's digest), read the best of that role in \
     its `## B role X` block, list the whole role with `--role X` (primary-X cards first, then the rest) and the \
     roleless candidates with `--view roleless`, then read the finalists with `find --name N --full`; a line marked \
     `BAN` or `NL` is not Commander-legal. Absent from the digest is not the same as nonexistent.",
    "- The digest `?` flag marks a card whose roles are not reviewed yet (routed to the LLM, no answer): its roles may \
     be incomplete or wrong. Roles say what a card does, not whether it is good in a deck: the sim decides (see \
     AGENTS.md).",
];

pub const USAGE: &str = "usage: cardgen map [--out PATH] [--stdout] [--data-dir DIR]
  (the map is always written to its file; --stdout also prints it)";

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
    let decks_path = project.data("decks.json");
    let decks = if decks_path.is_file() { load_mine(&decks_path)? } else { vec![] };
    let facts = Facts::load(project, &index, true, true)?;
    let text = build_with_facts(&index, &table, spell.is_dir().then_some(spell.as_path()), &decks, Some(&facts))?;
    let path = out_path.unwrap_or_else(|| digest::out_dir(project, data_dir.as_deref()).join("map.md"));
    digest::write_file(&path, &text)?;
    Ok((Output { text, path }, stdout))
}
