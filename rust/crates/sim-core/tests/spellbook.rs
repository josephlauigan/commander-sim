//! The modelled combo lines (`impls::combos::all()` and Sephiroth's `impls::mine::LOOPS`) against Commander Spellbook's combos for each deck
//! (`data/spellbook/<deck>.json`, fetched by `scripts/spellbook-fetch.sh`).
//!
//! Combo data by Commander Spellbook, https://commanderspellbook.com.
//!
//! A Spellbook combo is matched by its card set against each catalogue line's piece groups, one piece per group. A
//! Spellbook template ("a sacrifice outlet", "a persist creature") is a slot, not a card: a line's group may fill it
//! with any card of the group the deck plays.
//! - `Exact`: the line's pieces are the combo's cards (and the combo has no template);
//! - `Smaller`: the line needs only some of the combo's cards (it fires whenever the combo is assembled);
//! - `Template`: the line's pieces are combo cards plus, for the combo's templates, deck cards of the line's groups;
//! - `Bigger`: every card of the combo is a piece of the line, but the line also needs more (another group, which the
//!   deck fills: often a payoff Spellbook lists as a separate combo);
//! - `None`: no line.
//!
//! The guard (`included_combos_of_catalogue_pieces_are_modelled`) fails only for an "included" combo whose cards are
//! all catalogue pieces, that is legal in Commander and that no line covers at all (`None`), unless it is in
//! `KNOWN_GAPS` (the gaps listed in audit/phase2/spellbook-2026-10-29.md for JD; modelling them is a later PR).
//! `cargo test --release --test spellbook -- --ignored --nocapture spellbook_report` prints the whole comparison.

use serde_json::Value;
use sim_core::impls::combos::{all, quiet};
use sim_core::impls::mine::LOOPS;
use sim_core::testkit::data_dir;
use std::collections::{BTreeMap, HashSet};
use std::fs;

/// included combos whose cards are all catalogue pieces but that no line plays, in decks that hold them
/// (deck key, Spellbook variant id): the audit's list, not to model in this PR
const KNOWN_GAPS: &[(&str, &str)] = &[
    ("kefka", "1368-1414-4856"),                // Underworld Breach + Lotus Petal + Brain Freeze
    ("scion", "1368-1414-4856"),                // Underworld Breach + Lotus Petal + Brain Freeze
    ("krenko-mono-red-goblins", "38-659-2645"), // Skirk Prospector + Krenko, Mob Boss + Staff of Domination
    ("krenko-mono-red-goblins", "38-659-2816"), // Skirk Prospector + Krenko, Mob Boss + Umbral Mantle
    ("krenko-mono-red-goblins", "618-2178-5147"), // Kiki-Jiki, Mirror Breaker + Thornbite Staff + Goblin Bombardment
    ("krenko-mono-red-goblins", "38-618-2178"), // Kiki-Jiki, Mirror Breaker + Thornbite Staff + Skirk Prospector
    ("krenko-mono-red-goblins", "618-2034-2178"), // Kiki-Jiki, Mirror Breaker + Thornbite Staff + Ashnod's Altar
    ("kinnan-simic-mana-combo", "3430-4131"),   // Basalt Monolith + Nyxbloom Ancient
    ("urza-mono-blue-artifacts", "513-2364--47"), // Hullbreaker Horror + Mana Vault
    ("yawgmoth-mono-black-aristocrats", "3240-4050-4314"), // Nether Traitor + Phyrexian Altar + Pawn of Ulamog
    ("yawgmoth-mono-black-aristocrats", "2393-3240-4050"), // Nether Traitor + Phyrexian Altar + Sifter of Skulls
    ("kraum-tymna-blue-breach-midrange", "1368-1414-4856"), // Underworld Breach + Lotus Petal + Brain Freeze
    ("rograkh-silas-turbo-breach", "1368-1414-4856"), // Underworld Breach + Lotus Petal + Brain Freeze
    ("sisay-legends-toolbox", "2364-2608-4499"), // Derevi, Empyrial Tactician + Emiel the Blessed + Mana Vault
    ("sisay-legends-toolbox", "527-2608-4499"), // Derevi, Empyrial Tactician + Emiel the Blessed + Selvala, Heart of the Wilds
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cover {
    Exact,
    Smaller,
    Template,
    Bigger,
    None,
}

struct Variant {
    id: String,
    cards: Vec<String>,
    results: Vec<String>,
    requires: Vec<String>,
    prerequisites: String,
    legal: bool,
    missing: Vec<String>,
}

struct DeckCombos {
    key: String,
    tier: String,
    names: HashSet<String>,
    included: Vec<Variant>,
    almost: Vec<Variant>,
}

fn strs(v: &Value) -> Vec<String> {
    v.as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(str::to_string)).collect()).unwrap_or_default()
}

fn variant(v: &Value) -> Variant {
    Variant {
        id: v["id"].as_str().unwrap().to_string(),
        cards: v["cards"].as_array().unwrap().iter().map(|c| c["name"].as_str().unwrap().to_string()).collect(),
        results: strs(&v["results"]),
        requires: strs(&v["requires"]),
        prerequisites: v["prerequisites"].as_str().unwrap_or("").to_string(),
        legal: v["legal"].as_bool().unwrap_or(true),
        missing: strs(&v["missing"]),
    }
}

/// every deck of decks.json with its Spellbook file
fn decks() -> Vec<DeckCombos> {
    let decks: Value = serde_json::from_str(&fs::read_to_string(data_dir().join("decks.json")).unwrap()).unwrap();
    let mut out = vec![];
    for (side, d) in decks["mine"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| ("mine", d))
        .chain(decks["pool"].as_array().unwrap().iter().map(|d| ("pool", d)))
    {
        let key = d["key"].as_str().unwrap().to_string();
        let path = data_dir().join("spellbook").join(format!("{key}.json"));
        let sb: Value = serde_json::from_str(
            &fs::read_to_string(&path)
                .unwrap_or_else(|_| panic!("{} is missing: run scripts/spellbook-fetch.sh {key}", path.display())),
        )
        .unwrap();
        let mut names: HashSet<String> = strs(&d["cards"]).into_iter().collect();
        names.insert(d["commander"].as_str().unwrap().to_string());
        if let Some(c2) = d["commander2"].as_str() {
            names.insert(c2.to_string());
        }
        let tier = if side == "mine" {
            format!("JD ({})", strs(&d["tiers"]).join(", "))
        } else {
            d["tier"].as_str().unwrap().to_string()
        };
        out.push(DeckCombos {
            key,
            tier,
            names,
            included: sb["included"].as_array().unwrap().iter().map(variant).collect(),
            almost: sb["almost_included"].as_array().unwrap().iter().map(variant).collect(),
        });
    }
    out
}

/// a modelled line: the combo catalogue's (`combos::all()`, a quiet line's marker group dropped) and Sephiroth's
/// loops (`mine::LOOPS`, JD's deck)
pub struct Line {
    pub name: &'static str,
    pub groups: &'static [&'static [&'static str]],
}

fn lines() -> &'static [Line] {
    static L: std::sync::LazyLock<Vec<Line>> = std::sync::LazyLock::new(|| {
        let combos =
            all().iter().map(|c| Line { name: c.name, groups: if quiet(c) { &c.groups[1..] } else { c.groups } });
        let loops = LOOPS.iter().map(|l| Line { name: l.name, groups: l.groups });
        combos.chain(loops).filter(|l| !l.groups.is_empty()).collect()
    });
    &L
}

fn groups(c: &Line) -> &'static [&'static [&'static str]] {
    c.groups
}

/// can each group take a distinct name of `pool`, or one of `slots` template slots when the deck has a card of the
/// group? (`used`: names taken)
fn assign(gs: &[&[&str]], pool: &[&str], used: &mut Vec<usize>, slots: usize, deck: &HashSet<String>) -> bool {
    let Some((g, rest)) = gs.split_first() else { return true };
    for (i, n) in pool.iter().enumerate() {
        if !used.contains(&i) && g.contains(n) {
            used.push(i);
            if assign(rest, pool, used, slots, deck) {
                return true;
            }
            used.pop();
        }
    }
    slots > 0 && g.iter().any(|n| deck.contains(*n)) && assign(rest, pool, used, slots - 1, deck)
}

/// can each card take a distinct group, and the groups left each a card of the deck?
fn embed(cards: &[&str], gs: &[&[&str]], taken: &mut Vec<usize>, deck: &HashSet<String>) -> bool {
    let Some((c, rest)) = cards.split_first() else {
        return gs.iter().enumerate().all(|(i, g)| taken.contains(&i) || g.iter().any(|n| deck.contains(*n)));
    };
    for (i, g) in gs.iter().enumerate() {
        if !taken.contains(&i) && g.contains(c) {
            taken.push(i);
            if embed(rest, gs, taken, deck) {
                return true;
            }
            taken.pop();
        }
    }
    false
}

/// how a catalogue line covers a Spellbook combo's cards, in a deck
pub fn cover(c: &Line, cards: &[&str], templates: usize, deck: &HashSet<String>) -> Cover {
    let gs = groups(c);
    if gs.is_empty() {
        return Cover::None;
    }
    if gs.len() <= cards.len() && assign(gs, cards, &mut vec![], 0, deck) {
        return if gs.len() == cards.len() && templates == 0 { Cover::Exact } else { Cover::Smaller };
    }
    if templates > 0 && gs.len() <= cards.len() + templates && assign(gs, cards, &mut vec![], templates, deck) {
        return Cover::Template;
    }
    if gs.len() > cards.len() && embed(cards, gs, &mut vec![], deck) {
        return Cover::Bigger;
    }
    Cover::None
}

/// the best line for a combo: (cover, the line's name)
fn best(v: &Variant, deck: &HashSet<String>) -> (Cover, Option<&'static str>) {
    let cards = &v.cards;
    let cs: Vec<&str> = cards.iter().map(String::as_str).collect();
    lines()
        .iter()
        .map(|c| (cover(c, &cs, v.requires.len(), deck), Some(c.name)))
        .min_by_key(|x| x.0)
        .unwrap_or((Cover::None, None))
}

fn piece_names() -> HashSet<&'static str> {
    lines().iter().flat_map(|c| groups(c).iter().flat_map(|g| g.iter().copied())).collect()
}

/// the catalogue lines whose every group is in the deck
fn deck_lines(deck: &HashSet<String>) -> Vec<&'static Line> {
    lines()
        .iter()
        .filter(|c| !groups(c).is_empty() && groups(c).iter().all(|g| g.iter().any(|n| deck.contains(*n))))
        .collect()
}

/// does the line match some included Spellbook combo of the deck (any cover but None)?
fn known_by_spellbook(c: &Line, d: &DeckCombos) -> Option<(Cover, String)> {
    d.included
        .iter()
        .map(|v| {
            let cs: Vec<&str> = v.cards.iter().map(String::as_str).collect();
            (cover(c, &cs, v.requires.len(), &d.names), v.id.clone())
        })
        .filter(|x| x.0 != Cover::None)
        .min_by_key(|x| x.0)
}

#[test]
fn spellbook_files_match_the_decks() {
    for d in decks() {
        for v in &d.included {
            for n in &v.cards {
                assert!(d.names.contains(n), "{}: Spellbook's included {} uses {n:?}, not in the deck", d.key, v.id);
            }
        }
        // one card missing, or none when what is missing is a template's card ("a persist creature"): two would be a
        // name Spellbook spells differently from the deck list
        for v in &d.almost {
            assert!(
                v.missing.len() == 1 || (v.missing.is_empty() && !v.requires.is_empty()),
                "{}: almost included {} misses {:?}",
                d.key,
                v.id,
                v.missing
            );
        }
    }
}

#[test]
fn included_combos_of_catalogue_pieces_are_modelled() {
    let pieces = piece_names();
    let mut gaps = vec![];
    let mut uncovered: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for d in decks() {
        for v in d.included.iter().filter(|v| v.legal) {
            let (c, _) = best(v, &d.names);
            if c == Cover::None {
                uncovered.entry(d.key.clone()).or_default().push(format!("{} [{}]", v.id, v.cards.join(" + ")));
                if v.cards.iter().all(|n| pieces.contains(n.as_str()))
                    && !KNOWN_GAPS.contains(&(d.key.as_str(), v.id.as_str()))
                {
                    gaps.push(format!("({:?}, {:?}), // {}", d.key, v.id, v.cards.join(" + ")));
                }
            }
        }
    }
    for (k, v) in &uncovered {
        println!("{k}: {} included combos with no line: {}", v.len(), v.join("; "));
    }
    assert!(
        gaps.is_empty(),
        "included Spellbook combos whose cards are all catalogue pieces, with no line (model them, or list them in \
         KNOWN_GAPS and the audit):\n{}",
        gaps.join("\n")
    );
}

/// does a result end the game, or nearly (the lines that matter most)?
fn wins(results: &[String]) -> bool {
    results.iter().any(|r| {
        let r = r.to_lowercase();
        r.starts_with("win the game")
            || r.contains("each opponent loses the game")
            || r.contains("infinite damage")
            || r.contains("infinite lifeloss")
            || r.contains("infinite combat damage")
            || r.contains("infinite combat phases")
            || r.contains("infinite mill")
            || r.contains("near-infinite damage")
            || r.contains("near-infinite lifeloss")
    })
}

/// included combos the sim plays through card code rather than a catalogue line (checked by hand in the audit):
/// (Spellbook variant id, where)
const CARD_CODE: &[(&str, &str)] = &[
    ("2384-6574", "kefka.rs: the Psychosis Crawler loop (`loop_safe`)"),
    ("4971-6574", "kefka.rs: the Niv-Mizzet loop (`niv_safe`)"),
    ("1561-2498", "sheoldred.rs: Peer into the Abyss at an opponent (`peer_options`)"),
    ("1561-2720", "sheoldred.rs: Peer into the Abyss at an opponent (`peer_options`, `draw_toll`)"),
    ("1800-2429", "common.rs: Revel in Riches wins at ten Treasures; Old Gnawbone makes them (t3.rs)"),
    ("865-2034-2429", "common.rs: Revel in Riches wins at ten Treasures; Mahadi makes them (t4.rs)"),
    ("865-2292-2429", "common.rs: Revel in Riches wins at ten Treasures; Mahadi makes them (t4.rs)"),
    ("865-2429-4050", "common.rs: Revel in Riches wins at ten Treasures; Mahadi makes them (t4.rs)"),
];

fn card_code(id: &str) -> Option<&'static str> {
    CARD_CODE.iter().find(|x| x.0 == id).map(|x| x.1)
}

fn row(v: &Variant, c: Cover, line: Option<&str>) -> String {
    let mut pre = v.requires.clone();
    if !v.prerequisites.is_empty() {
        pre.push(v.prerequisites.replace('\n', " "));
    }
    format!(
        "| {} | {} | {} | {} | {} | {} | {} |",
        v.id,
        v.cards.len(),
        v.cards.join(" + "),
        if wins(&v.results) { "**win**" } else { "" },
        v.results.join("; "),
        pre.join("; ").replace('|', "/"),
        match c {
            Cover::None => card_code(&v.id).map_or("no".to_string(), |w| format!("card code: {w}")),
            c => format!("{c:?}: {}", line.unwrap_or("")),
        }
    )
}

/// the whole comparison, as markdown, for the audit (SPELLBOOK_REPORT=<path>, else stdout)
#[test]
#[ignore]
fn spellbook_report() {
    let mut s = String::new();
    let pieces = piece_names();
    let mut ds = decks();
    ds.sort_by(|a, b| a.tier.cmp(&b.tier).then(a.key.cmp(&b.key)));
    let mut summary = vec![];
    let mut used_lines: HashSet<&str> = HashSet::new();
    for d in &ds {
        let incl: Vec<&Variant> = d.included.iter().filter(|v| v.legal).collect();
        let mut rows = vec![];
        let (mut covered, mut win_gaps) = (0, 0);
        for v in &incl {
            let (c, line) = best(v, &d.names);
            if c != Cover::None || card_code(&v.id).is_some() {
                covered += 1;
            } else if wins(&v.results) && v.cards.len() <= 3 {
                win_gaps += 1;
            }
            rows.push((c, row(v, c, line)));
        }
        let lines = deck_lines(&d.names);
        summary.push(format!(
            "| {} | {} | {} | {} | {} | {} | {} |",
            d.tier,
            d.key,
            incl.len(),
            covered,
            win_gaps,
            lines.len(),
            d.almost.iter().filter(|v| v.legal).count()
        ));
        s += &format!("\n### {} ({})\n\n", d.key, d.tier);
        if incl.is_empty() {
            s += "Spellbook finds no combo in this list.\n";
        } else {
            s += "| Spellbook id | cards | combo | wins | results | prerequisites | sim line |\n|---|---|---|---|---|---|---|\n";
            rows.sort_by(|a, b| a.0.cmp(&b.0).reverse());
            for (_, r) in &rows {
                s += r;
                s += "\n";
            }
        }
        if !lines.is_empty() {
            s += "\nCatalogue lines the deck holds:\n\n";
            for c in &lines {
                used_lines.insert(c.name);
                match known_by_spellbook(c, d) {
                    Some((cv, id)) => s += &format!("- {}: Spellbook {id} ({cv:?})\n", c.name),
                    None => s += &format!("- {}: **not in Spellbook's list for this deck**\n", c.name),
                }
            }
        }
        // the almost-included combos one card short whose cards are otherwise all catalogue pieces are noise: show
        // the winning two- and three-card ones only
        let near: Vec<&Variant> =
            d.almost.iter().filter(|v| v.legal && v.cards.len() <= 3 && wins(&v.results)).collect();
        if !near.is_empty() {
            s += &format!("\nAlmost included (one card missing), winning, 2-3 cards: {}\n\n", near.len());
            for v in near.iter().take(12) {
                s += &format!("- {} ({}): missing {}\n", v.cards.join(" + "), v.id, v.missing.join(", "));
            }
            if near.len() > 12 {
                s += &format!("- ... and {} more\n", near.len() - 12);
            }
        }
    }
    let mut out = String::from(
        "| tier | deck | included (legal) | sim covers (line or card code) | uncovered 2-3 card wins | catalogue lines held | almost included |\n|---|---|---|---|---|---|---|\n",
    );
    out += &summary.join("\n");
    out += "\n";
    out += &s;
    out += "\n### Catalogue lines no deck holds\n\n";
    for c in lines() {
        if !used_lines.contains(c.name) {
            out += &format!("- {} ({} groups)\n", c.name, groups(c).len());
        }
    }
    out += &format!("\n{} catalogue lines, {} piece names.\n", lines().len(), pieces.len());
    match std::env::var("SPELLBOOK_REPORT") {
        Ok(p) => fs::write(p, out).unwrap(),
        Err(_) => println!("{out}"),
    }
}
