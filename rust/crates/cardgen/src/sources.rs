//! Building the card set (Python's `cards/sources.py` + `cards/pool_cards.py` + the tag edits the card modules ran):
//!
//!  1. the hand-tagged cards (data/carddb.txt) win;
//!  2. every other card is built from its Scryfall record: Oracle text compiled into abilities (dsl_parse), falling
//!     back to the regex tagger (autotag) when nothing compiles;
//!  3. data/card_overrides.toml replaces a card's tags / types / cost / abilities (except your decks' hand-tagged
//!     cards), then adds the keyword data and tag edits.

use crate::autotag::{AutoTags, autotag};
use crate::cd::Cd;
use crate::data::{CardDbLine, Entry};
use crate::dsl::{build_cd, hints};
use crate::json::{J, Obj};
use crate::re::{rmatch, search};
use crate::scryfall::Cache;
use std::collections::{BTreeMap, BTreeSet};

/// `_tagdict`: "pow=3 fly" -> {pow: "3", fly: true}
pub fn tagdict(s: &str) -> Obj {
    let mut o = Obj::new();
    for x in s.split_whitespace() {
        match x.split_once('=') {
            Some((k, v)) => o.set(k, v),
            None => o.set(x, true),
        }
    }
    o
}

const COLOR_WORDS: [(&str, char); 5] = [("white", 'W'), ("blue", 'U'), ("black", 'B'), ("red", 'R'), ("green", 'G')];

/// the type line's subtypes, lower case
fn subtypes_of(tl: &str) -> BTreeSet<String> {
    match tl.split('—').nth(1) {
        Some(sub) if tl.contains('—') => sub.split_whitespace().map(|w| w.to_lowercase()).collect(),
        _ => BTreeSet::new(),
    }
}

/// a record's first face (the whole record for a split card or a card with one face)
fn first_face(rec: &J) -> &J {
    match rec.get("card_faces").and_then(|f| f.as_arr()).filter(|f| !f.is_empty()) {
        Some(f) if rec.s("layout") != Some("split") => &f[0],
        _ => rec,
    }
}

/// keywords the engine reads from card data: Scryfall's keyword list, protection colours, ward cost
fn keyword_data(cd: &mut Cd, rec: &J, oracle: &str) {
    cd.kws = rec.strs("keywords").iter().map(|k| k.to_lowercase()).collect();
    let face = first_face(rec);
    let tl = face.s("type_line").filter(|s| !s.is_empty()).or_else(|| rec.s("type_line")).unwrap_or("");
    cd.subtypes = subtypes_of(tl);
    let low = oracle.to_lowercase();
    let own: Vec<&str> =
        low.split('\n').filter(|l| rmatch(r"^(?:[a-z ]+, )*protection from", l.trim()).is_some()).collect();
    let own = own.join(" ");
    cd.protfrom = COLOR_WORDS
        .iter()
        .filter(|(w, _)| search(&format!(r"protection from (?:[a-z]+ and from )?{w}"), &own).is_some())
        .map(|(_, c)| *c)
        .collect();
    cd.ward = search(r"\bward \{(\d+)\}", &low).map(|m| m.gs(1).parse().unwrap()).unwrap_or(0);
}

/// a card from its Scryfall record
pub fn from_record(name: &str, rec: &J) -> Cd {
    let r: AutoTags = autotag(rec);
    let face = first_face(rec);
    let oracle: String = match face.get("oracle_text") {
        Some(J::Str(s)) => s.clone(),
        Some(_) => String::new(),
        None => rec.s("oracle_text").unwrap_or("").to_string(),
    };
    let loyalty: Option<String> = face
        .get("loyalty")
        .filter(|v| v.truthy())
        .or_else(|| rec.get("loyalty"))
        .and_then(|v| if v.truthy() { Some(v.py_str()) } else { None });
    let mut cd = build_cd(name, &r.types, &r.cost, &tagdict(&r.tags), &oracle, loyalty.as_deref());
    if cd.dsl.is_none() {
        // nothing compiled: use the regex tagger's tags instead
        cd = Cd::new(name, &r.types, &r.cost, &r.tags);
        cd.source = "scryfall".into();
        cd.unparsed = r.unparsed.clone();
        cd.unparsed.extend(r.notes.iter().map(|x| format!("note: {x}")));
        cd.start_loyalty = loyalty;
    }
    cd.game_changer = Some(r.game_changer);
    cd.identity = Some(rec.strs("color_identity").concat());
    keyword_data(&mut cd, rec, &oracle);
    cd
}

/// hand-tagged cards keep their tags but get their printed creature types (and changeling) from the card data, for
/// the cards that care about creature types (Kindred Discovery, Door of Destinies, the Banners)
fn printed_types(db: &mut BTreeMap<String, Cd>, cache: &Cache) {
    for (n, cd) in db.iter_mut() {
        if cd.source != "manual" || !cd.subtypes.is_empty() || cd.types_filled {
            continue;
        }
        cd.types_filled = true;
        let Some(rec) = cache.get(n) else { continue };
        let face = first_face(rec);
        let tl = face.s("type_line").filter(|s| !s.is_empty()).or_else(|| rec.s("type_line")).unwrap_or("");
        if !tl.contains('—') {
            continue;
        }
        cd.subtypes = subtypes_of(tl);
        if rec.strs("keywords").contains(&"Changeling") {
            cd.kws.insert("changeling".into());
        }
    }
}

/// rebuild a card from an override entry (Python's `pool_cards.apply`, one card)
fn apply_override(base: &Cd, spec: &Entry) -> Cd {
    let types = spec.types.clone().filter(|s| !s.is_empty()).unwrap_or_else(|| base.types.clone());
    let cost = spec.cost.clone().filter(|s| !s.is_empty()).unwrap_or_else(|| base.cost_string());
    let mut tags = spec.tags.clone().unwrap_or_else(|| base.tag_string());
    let dsl: Option<Vec<Obj>> = match &spec.dsl {
        Some(d) => Some(d.clone()),
        None => base.dsl.clone(),
    };
    if let Some(d) = &spec.dsl {
        if !d.is_empty() {
            let extra = hints(d, &types);
            let have: Vec<String> = tags.split_whitespace().map(|s| s.to_string()).collect();
            let mut parts = vec![tags.clone()];
            for (k, v) in extra.iter() {
                if !have.iter().any(|h| h == k) {
                    parts.push(if *v == J::Bool(true) { k.to_string() } else { format!("{k}={}", v.py_str()) });
                }
            }
            tags = parts.join(" ");
        }
    }
    let mut cd = Cd::new(&base.name, &types, &cost, &tags);
    cd.dsl = dsl.filter(|d| !d.is_empty());
    cd.unparsed = if spec.dsl.is_some() || spec.tags.is_some() { vec![] } else { base.unparsed.clone() };
    cd.identity = base.identity.clone();
    cd.kws = base.kws.clone();
    cd.protfrom = base.protfrom.clone();
    cd.ward = base.ward;
    cd.game_changer = base.game_changer;
    cd.start_loyalty = base.start_loyalty.clone();
    cd.subtypes = base.subtypes.clone();
    if let Some(k) = &spec.kws {
        cd.kws = k.iter().cloned().collect();
    }
    if let Some(w) = spec.ward {
        cd.ward = w;
    }
    cd.source = "pool".into();
    cd.pool_override = true;
    cd
}

/// What to build: the lists the cards come from.
pub struct Wanted {
    /// every card of your decks
    pub main: BTreeSet<String>,
    /// every card of the decks the simulator plays (yours and the pool's): these also get the tag edits
    pub played: BTreeSet<String>,
    /// every other card to build (the commanders, the whole card cache)
    pub extra: BTreeSet<String>,
}

pub struct Built {
    pub cards: BTreeMap<String, Cd>,
    /// names with no Scryfall record
    pub missing: Vec<String>,
}

/// Build every card: the hand-tagged ones, then the rest from Scryfall records, then the overrides.
pub fn build(carddb: &[CardDbLine], overrides: &[(String, Entry)], cache: &Cache, want: &Wanted) -> Built {
    let mut db: BTreeMap<String, Cd> = BTreeMap::new();
    for l in carddb {
        db.insert(l.name.clone(), Cd::new(&l.name, &l.types, &l.cost, &l.tags));
    }
    let hand_tagged: BTreeSet<String> = db.keys().cloned().collect();
    let mut names: BTreeSet<&String> = BTreeSet::new();
    names.extend(&want.played);
    names.extend(&want.extra);
    names.extend(&want.main);
    let mut missing = vec![];
    for n in names {
        if db.contains_key(n) {
            continue;
        }
        match cache.get(n) {
            Some(rec) => {
                db.insert(n.clone(), from_record(n, rec));
            }
            None => missing.push(n.clone()),
        }
    }
    printed_types(&mut db, cache);
    // overrides (pool_cards.apply): not on your decks' hand-tagged cards
    for (name, spec) in overrides {
        if !spec.is_override() {
            continue;
        }
        let Some(base) = db.get(name) else { continue };
        if want.main.contains(name) && base.source == "manual" {
            continue;
        }
        if base.pool_override {
            continue;
        }
        let cd = apply_override(base, spec);
        db.insert(name.clone(), cd);
    }
    // tag edits and keyword data of the cards the simulator plays (pool_cards.POST)
    for (name, e) in overrides {
        let Some(cd) = db.get_mut(name) else { continue };
        if e.status.as_deref() == Some("Full") {
            cd.unparsed.clear(); // reviewed as Full: the card code covers what the text parser could not read (display only)
        }
        if hand_tagged.contains(name) || want.played.contains(name) {
            let in_main = want.main.contains(name);
            if !in_main {
                for k in &e.pool_remove_tags {
                    cd.remove_tag(k);
                }
            }
            for k in &e.add_tags {
                cd.set_tag(k);
            }
            if !in_main {
                for k in &e.pool_add_tags {
                    cd.set_tag(k);
                }
            }
        }
        if cd.source == "manual" {
            if let Some(k) = &e.kws {
                cd.kws = k.iter().cloned().collect();
            }
            if let Some(w) = e.ward {
                cd.ward = w;
            }
            if let Some(l) = &e.loyalty {
                cd.start_loyalty = Some(l.clone());
            }
        }
    }
    Built { cards: db, missing }
}
