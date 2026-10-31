//! Tags from a Scryfall card record (Python's `cards/autotag.py`), for cards nobody hand-tagged.
//!
//! Exact: types, mana cost, power/toughness, legendary, keywords, mana abilities of lands, rocks and dorks.
//! Parsed: common templated Oracle effects (draw, tutor, removal, sweepers, counterspells, reanimation, tokens,
//! spell-cast triggers, drains, anthems, protection, flashback, cost reductions). Every Oracle line it doesn't
//! understand comes back in `unparsed`.

use crate::json::{J, Obj};
use crate::py;
use crate::re::{findall, is, rmatch, search, starts, sub};

const NUM: &[(&str, i64)] = &[
    ("a", 1),
    ("an", 1),
    ("one", 1),
    ("two", 2),
    ("three", 3),
    ("four", 4),
    ("five", 5),
    ("six", 6),
    ("seven", 7),
    ("eight", 8),
    ("x", 3),
    ("that many", 2),
];
const CARD_TYPES: &[(&str, Option<char>)] = &[
    ("Land", Some('L')),
    ("Creature", Some('C')),
    ("Artifact", Some('A')),
    ("Enchantment", Some('E')),
    ("Instant", Some('I')),
    ("Sorcery", Some('S')),
    ("Planeswalker", Some('P')),
    ("Battle", None),
];
pub const KEYWORDS: &[(&str, &str)] = &[
    ("flying", "fly"),
    ("deathtouch", "dt"),
    ("vigilance", "vig"),
    ("lifelink", "lifelink"),
    ("haste", "haste"),
    ("trample", "trample"),
    ("flash", "flash"),
    ("reach", "reach"),
    ("convoke", "convoke"),
];
const IGNORABLE_KEYWORDS: &[&str] = &[
    "menace",
    "first strike",
    "double strike",
    "hexproof",
    "indestructible",
    "prowess",
    "ward",
    "defender",
    "shroud",
    "intimidate",
    "changeling",
    "cycling",
    "kicker",
    "equip",
    "protection",
    "infect", // combat: poison to players, -1/-1 counters to creatures (sim-core engine::combat)
    "toxic",  // the amount is compiled (dsl_parse: static "toxic")
];
const IGNORABLE: &[&str] = &[
    r"\bscry\b",
    r"\bsurveil\b",
    r"you gain \d+ life",
    r"enters tapped",
    r"mill ",
    r"investigate",
    r"^equip \{",
    r"crew \d",
    r"this spell can't be countered",
    r"as .* enters, choose",
    r"you may pay \{\d\}\.?$",
];

fn kw(x: &str) -> Option<&'static str> {
    KEYWORDS.iter().find(|(k, _)| *k == x).map(|(_, v)| *v)
}

/// Python's `num`: a number word ("two", "x" counts 3)
pub fn num(s: &str) -> i64 {
    let s = s.trim().to_lowercase();
    if py::isdigit(&s) {
        return s.parse().unwrap();
    }
    NUM.iter().find(|(k, _)| *k == s).map(|(_, v)| *v).unwrap_or(1)
}

/// '{2}{U}{U/P}{X}' -> '2UU' (X dropped, hybrid / Phyrexian take the first colour); '-' for no cost
pub fn conv_cost(mc: &str) -> String {
    if mc.is_empty() {
        return "-".into();
    }
    let (mut gen_, mut pips) = (0i64, String::new());
    for sym in findall(r"\{([^}]+)\}", py::first(mc, " // ")) {
        if py::isdigit(&sym) {
            gen_ += sym.parse::<i64>().unwrap();
        } else if sym == "X" || sym == "Y" {
            continue;
        } else if sym == "C" {
            gen_ += 1;
        } else {
            let c = py::first(&sym, "/");
            if "WUBRG".contains(c) {
                pips.push_str(c);
            } else if py::isdigit(c) {
                gen_ += c.parse::<i64>().unwrap();
            }
        }
    }
    if gen_ != 0 || !pips.is_empty() {
        format!("{}{}", if gen_ != 0 { gen_.to_string() } else { String::new() }, pips)
    } else {
        "0".into()
    }
}

/// a card's front face, with the record's own fields filling what the face lacks; the back faces' Oracle text
pub fn front(rec: &J) -> (J, Vec<Option<String>>) {
    let faces = rec.get("card_faces").and_then(|f| f.as_arr()).filter(|f| !f.is_empty());
    if let Some(f) = faces {
        if rec.s("layout") != Some("split") {
            let mut face = f[0].as_obj().cloned().unwrap_or_default();
            for k in ["mana_cost", "type_line", "oracle_text", "power", "toughness"] {
                let has = face.get(k).map(|v| v.truthy()).unwrap_or(false);
                if !has {
                    if let Some(v) = rec.get(k).filter(|v| v.truthy()) {
                        face.set(k, v.clone());
                    }
                }
            }
            let back = f[1..]
                .iter()
                .map(|x| match x.get("oracle_text") {
                    None => Some(String::new()),
                    Some(J::Str(s)) => Some(s.clone()),
                    Some(_) => None,
                })
                .collect();
            return (J::Obj(face), back);
        }
    }
    (rec.clone(), vec![])
}

fn target_kind(phrase: &str) -> Option<&'static str> {
    let p = phrase.to_lowercase();
    Some(if p.contains("nonland permanent") {
        "nl"
    } else if p.contains("artifact, creature, or planeswalker") || p.contains("artifact or creature") {
        "cap"
    } else if p.contains("creature, enchantment, or planeswalker") {
        "cep" // --- card-models-3: Get Lost
    } else if p.contains("creature or planeswalker") {
        "cp"
    } else if p.contains("creature or enchantment") {
        "ce"
    } else if p.contains("nonartifact creature") {
        "cna"
    } else if p.contains("creature") {
        "c"
    } else if p.contains("permanent") {
        "p"
    } else if p.contains("artifact") {
        "a"
    } else if p.contains("enchantment") || p.contains("planeswalker") {
        "nl"
    } else {
        return None;
    })
}

/// What autotag makes of a record.
#[derive(Debug, Clone)]
pub struct AutoTags {
    pub types: String,
    pub cost: String,
    /// the tag string ("pow=3 fly leg")
    pub tags: String,
    pub unparsed: Vec<String>,
    pub notes: Vec<String>,
    pub game_changer: bool,
}

/// `k` or `k=v` per tag, space separated (Python's tag string)
pub fn tagstr(tags: &Obj) -> String {
    tags.iter()
        .map(|(k, v)| if *v == J::Bool(true) { k.to_string() } else { format!("{k}={}", v.py_str()) })
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn autotag(rec: &J) -> AutoTags {
    let (face, back) = front(rec);
    let name = py::first(rec.s_or("name"), " // ").to_string();
    let tl = face.s("type_line").or_else(|| if face.get("type_line").is_none() { rec.s("type_line") } else { None });
    let tl = tl.unwrap_or("").to_string();
    let main_types = py::first(&tl, "—");
    let mut types: String =
        CARD_TYPES.iter().filter(|(t, l)| main_types.contains(t) && l.is_some()).map(|(_, l)| l.unwrap()).collect();
    if types.is_empty() {
        types = "S".into();
    }
    if types.contains('C') && types.contains('A') {
        types = "AC".into();
    }
    let mc = face.s("mana_cost").filter(|s| !s.is_empty()).or_else(|| rec.s("mana_cost")).unwrap_or("");
    let cost = conv_cost(mc);
    let mut tags = Obj::new();
    let mut unparsed: Vec<String> = vec![];
    let mut notes: Vec<String> = vec![];
    let creature = types.contains('C');
    let land = types.contains('L');
    let spell = types.contains('I') || types.contains('S');
    if tl.contains("Legendary") {
        tags.set("leg", true);
    }
    let subl = if tl.contains('—') { py::part(&tl, "—", 1).unwrap_or("").to_string() } else { String::new() };
    for (w, t) in [("Human", "human"), ("Warrior", "warrior"), ("Shaman", "shaman"), ("Wizard", "wizard")] {
        if subl.contains(w) {
            tags.set(t, true);
        }
    }
    if creature {
        let digit = |k: &str| face.s(k).filter(|s| py::isdigit(s)).map(|s| s.parse::<i64>().unwrap());
        let has = |k: &str| face.get(k).is_some();
        let pw = if has("power") { digit("power").unwrap_or(1) } else { 1 };
        let th = if has("toughness") { digit("toughness").unwrap_or(pw.max(1)) } else { 1 };
        tags.set("pow", pw);
        if th != pw.max(1) {
            tags.set("tgh", th);
        }
        if pw >= 5 {
            tags.set("bomb", pw.min(9));
        }
    }
    for k in rec.strs("keywords") {
        if let Some(t) = kw(&k.to_lowercase()) {
            tags.set(t, true);
        }
    }
    let mut text = face.s("oracle_text").or_else(|| rec.s("oracle_text")).unwrap_or("").to_string();
    if face.get("oracle_text").is_some() && face.s("oracle_text").is_none() {
        text = String::new(); // a face with oracle_text: null
    }
    text = sub(r"\([^)]*\)", "", &text);
    let short = py::first(&name, ",").to_string();
    for nm in [&name, &short] {
        if py::len(nm) > 3 {
            text = text.replace(nm.as_str(), "~");
        }
    }
    for w in ["this creature", "this spell", "this artifact", "this enchantment", "this land"] {
        text = text.replace(w, "~");
    }
    let mut mana_c: Option<String> = None;
    macro_rules! add {
        ($k:expr) => {
            if !tags.contains($k) {
                tags.set($k, true);
            }
        };
        ($k:expr, $v:expr) => {
            if !tags.contains($k) {
                tags.set($k, J::from($v));
            }
        };
    }
    let low = text.to_lowercase();
    let mut whole_hit: Vec<&str> = vec![];
    if low.contains("destroy all artifacts") && low.contains("destroy all enchantments") && low.contains("mana value") {
        tags.set("wipe", "austere2");
        whole_hit.push("destroy all");
    }
    if low.contains("exile all artifacts") && low.contains("exile all creatures") {
        tags.set("wipe", "farewell");
        whole_hit.push("exile all");
    }
    let lines: Vec<String> = text.split('\n').map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
    for raw in lines {
        let lraw = raw.to_lowercase();
        let l = py::rstrip(&lraw, ".");
        let mut hit = whole_hit.iter().any(|w| py::lstrip(l, "• ").starts_with(w)) || l.starts_with("choose ");
        let parts: Vec<&str> = l.split(',').map(|x| x.trim()).collect();
        if parts
            .iter()
            .all(|x| kw(x).is_some() || IGNORABLE_KEYWORDS.iter().any(|k| *x == *k || x.starts_with(&format!("{k} "))))
        {
            for x in parts {
                if let Some(k) = kw(x) {
                    add!(k);
                }
            }
            continue;
        }
        let etb = starts(r"(when|whenever) ~ enters", l);
        let activated = starts(r#"^[^"]*?\{[^}]*\}[^"]*?:"#, l) && !spell;
        let eff = spell || etb;
        let upkeep =
            l.starts_with("at the beginning of your upkeep") || l.starts_with("at the beginning of your draw step");
        let castrig = starts(
            r"(magecraft — )?whenever you cast (or copy )?(an instant or sorcery|a noncreature|an? (instant|sorcery)) spell",
            l,
        );
        let attack = starts(r"whenever ~ attacks", l);
        // --- mana abilities
        if is(r#"creatures you control have "\{t\}: add"#, l) {
            add!("rite");
            hit = true;
        }
        if let Some(m) = search(r"\{t\}(?:, [^:]*)?: add ([^.]*)", l) {
            if !hit {
                let add_s = m.gs(1);
                let cols: String = if (add_s.contains("any color") || add_s.contains("any type"))
                    && !l.contains("spend this mana only")
                    && !l.contains("as long as")
                {
                    "A".into()
                } else if add_s.contains("any color") || l.contains("as long as") {
                    "C".into()
                } else {
                    let c: String = "wubrg"
                        .chars()
                        .filter(|c| add_s.contains(&format!("{{{c}}}")))
                        .collect::<String>()
                        .to_uppercase();
                    if c.is_empty() { "C".into() } else { c }
                };
                let amt = if !add_s.contains(" or ") { findall(r"\{[wubrgc]\}", add_s).len() as i64 } else { 1 };
                if land {
                    let prev = mana_c.clone().unwrap_or_default();
                    mana_c = Some(if prev == "A" || cols == "A" {
                        "A".into()
                    } else {
                        let s: String = "WUBRG".chars().filter(|c| prev.contains(*c) || cols.contains(*c)).collect();
                        if s.is_empty() { "C".into() } else { s }
                    });
                    if amt >= 2 {
                        add!("amt", amt);
                    }
                } else if creature {
                    add!("dork", cols.as_str());
                    add!("noatk");
                } else if !spell {
                    add!("rock", format!("{}:{}", amt.max(1), cols));
                }
                hit = true;
            }
        }
        // --- draw
        let mut m = search(r"(?:^|[,.:—] |then |and |you )draw (a|one|two|three|four|five|x|\d+) cards?", l);
        if m.is_some() && is(r"whenever (you|an opponent|a player|equipped)[^,]* draws?|opponent draws|player draws", l)
        {
            m = None;
        }
        if let Some(m) = m {
            if !castrig {
                if upkeep {
                    add!("eng", 1);
                    hit = true;
                } else if eff {
                    add!("draw", num(m.gs(1)));
                    hit = true;
                } else if activated && !land && !py::first(l, ":").contains("sacrifice") {
                    add!("eng", 1);
                    hit = true;
                }
            }
        }
        if search(r"draw (a|one|two|three) additional cards?", l).is_some() && upkeep {
            add!("eng", 1);
            hit = true;
        }
        if let Some(m) = search(
            r"put (one|two|three) of them into your hand|you may play (those|that) cards?|draw a card for each",
            l,
        ) {
            if eff && !tags.contains("draw") {
                let two = matches!(m.g(1), Some("two") | Some("three")) || m.g(2) == Some("those");
                add!("draw", if two { 2 } else { 1 });
                hit = true;
            }
        }
        // --- tutors / ramp
        let mut m = search(r"search your library for (an?|up to (?:one|two|three))\s(.*?)\s?cards?\b", l);
        if let Some(mm) = &m {
            if land && is(r"sacrifice ~", l) && is(r"land|forest|plains|island|swamp|mountain", mm.gs(2)) {
                tags.set("f", true);
                mana_c = Some("A".into());
                hit = true;
                m = None;
            }
        }
        if m.is_some() && l.contains("the other into your hand") {
            add!("lr", 1);
            add!("lrt");
            add!("lh", 1);
            hit = true;
            m = None;
        }
        if let Some(m) = m {
            let what = m.gs(2);
            if ["land", "forest", "plains", "island", "swamp", "mountain"].iter().any(|w| what.contains(w)) {
                let n = if m.gs(1).contains("up to two") { 2 } else { 1 };
                if l.contains("onto the battlefield") {
                    add!("lr", n);
                    if l.contains("tapped") {
                        add!("lrt");
                    }
                    if l.contains("into your hand") {
                        add!("lh", 1);
                    }
                } else {
                    add!("lh", 1);
                }
            } else if l.contains("into your graveyard") {
                add!("fill", "entomb");
            } else {
                let kind = if what.contains("instant or sorcery") {
                    "is"
                } else if what.contains("artifact") {
                    "art"
                } else if what.contains("enchantment") {
                    "ench"
                } else if what.contains("creature") {
                    "cre"
                } else {
                    "any"
                };
                add!("tut", kind);
            }
            hit = true;
        }
        // --- removal
        if let Some(m) = search(
            r"(destroy|exile) (up to one )?(another )?target ([^.]*?)(\.|$| and | with | you don't| an opponent controls| your opponents control)",
            l,
        ) {
            if eff && !l.contains("from a graveyard") && !m.gs(4).contains("card") && !m.gs(4).contains("you control") {
                if let Some(k) = target_kind(m.gs(4)) {
                    add!("rem", if m.gs(1) == "destroy" { "destroy" } else { "exile" });
                    add!("tgt", k);
                    if etb && !spell {
                        add!("etb");
                    }
                    hit = true;
                }
            }
        }
        if let Some(m) = search(
            r"return (up to one )?target (spell or permanent|nonland permanent|creature|permanent|artifact or creature)[^.]*? to (its|their) owner's hand",
            l,
        ) {
            if eff
                && !text.to_lowercase().contains("overload")
                && !m.all().replace("you don't control", "").contains("you control")
            {
                add!("rem", "bounce");
                add!("tgt", target_kind(m.gs(2)).unwrap_or("nl"));
                if etb && !spell {
                    add!("etb");
                }
                hit = true;
            }
        }
        if let Some(m) = search(
            r"deals (\d+|x) damage to (any target|target creature or planeswalker|target creature|any other target)",
            l,
        ) {
            if !castrig && eff {
                let d = if m.gs(1) == "x" { 3 } else { m.gs(1).parse::<i64>().unwrap() };
                add!("rem", format!("dmg{d}"));
                add!("tgt", "c");
                if m.gs(2).contains("any") {
                    add!("face");
                }
                if etb && !spell {
                    add!("etb");
                }
                hit = true;
            }
        }
        if is(r"(destroy|exile) target artifact", l)
            && tags.contains("rem")
            && !tags.contains("alsoart")
            && tags.get("tgt") == Some(&J::from("c"))
        {
            add!("alsoart");
            hit = true;
        }
        if is(r"owner of target permanent shuffles it into their library", l) && eff {
            add!("rem", "tuck");
            add!("tgt", "p");
            hit = true;
        }
        if is(r"return target nonland permanent you don't control to its owner's hand", l)
            && text.to_lowercase().contains("overload")
        {
            add!("wipe", "rift");
            add!("rem", "bounce");
            add!("tgt", "nl");
            hit = true;
        }
        if l.starts_with("overload") {
            hit = true;
        }
        if is(r"destroy target artifact you don't control", l) && low.contains("overload") {
            add!("wipe", "vandal");
            add!("rem", "destroy");
            add!("tgt", "a");
            hit = true;
        }
        if is(r"deals damage equal to its power to each other creature", l) {
            add!("wipe", "nib");
            hit = true;
        }
        if is(r"metalcraft .* exile that creature", l) {
            add!("rem", "exile");
            add!("tgt", "c");
            add!("needart3");
            hit = true;
        }
        if is(r"^tap target creature", l) {
            hit = true;
        }
        // --- sweepers
        if is(r"destroy all creatures", l) {
            add!("wipe", "destroy");
            hit = true;
        } else if is(r"exile all creatures", l) {
            add!("wipe", "exile");
            hit = true;
        } else if is(r"all creatures get -(\d+|x)/-(\d+|x)", l) {
            add!("wipe", "minus");
            hit = true;
        } else if is(r"return all creatures to their owners' hands", l) {
            add!("wipe", "evac");
            hit = true;
        } else if is(
            r"return all nonland permanents (you don't control|target player controls|your opponents control)",
            l,
        ) {
            add!("wipe", if l.contains("target player") { "rebuke" } else { "rift" });
            hit = true;
        }
        if let Some(m) = search(r"deals (\d+) damage to each creature", l) {
            if m.gs(1).parse::<i64>().unwrap_or(0) >= 6 {
                add!("wipe", "dmg13");
            } else {
                notes.push(format!("small damage sweep ({}) not modeled", m.gs(1)));
            }
            hit = true;
        }
        if is(r"costs \{1\} less to cast for each creature on the battlefield", l) {
            add!("perCreature");
            hit = true;
        }
        // --- counterspells
        let mut m = search(r"counter target ([a-z, ]*?)\s?(spell or ability|spell|ability)", l);
        if m.is_some() && l.contains("that targets a permanent you control") {
            add!("prot", "notw");
            hit = true;
            m = None;
        }
        if let Some(m) = m {
            let what = m.gs(1);
            if m.gs(2) == "ability" || what.contains("ability") {
                notes.push("ability counter not modeled".into());
            } else {
                let scope = if what.contains("noncreature") {
                    "nc"
                } else if what.trim() == "creature" {
                    "cre"
                } else if l.contains("mana value 4") {
                    "mv4"
                } else if !what.is_empty() {
                    "ise"
                } else {
                    "any"
                };
                add!("ctr", scope);
                if let Some(s) = search(r"unless its controller pays \{(\d+)\}", l) {
                    add!("soft", s.gs(1).parse::<i64>().unwrap());
                }
            }
            hit = true;
        }
        // --- reanimation
        if let Some(m) = search(
            r"(return|put) target (?:nonlegendary )?creature card from (a|your) graveyard (to|onto) the battlefield",
            l,
        ) {
            // ('Enchantment' in types) never holds: types are letters
            if spell || tl.to_lowercase().contains("aura") {
                let mut kind = if m.gs(2) == "a" { "animate" } else { "evil" };
                if l.contains("lose life equal to its mana value") {
                    kind = "reanimate";
                }
                add!("rean", kind);
                hit = true;
            }
        }
        if is(r"enchant creature card in a graveyard", l) {
            add!("rean", "animate");
            hit = true;
        }
        if is(r"return enchanted creature card to the battlefield", l) {
            hit = true;
        }
        // --- tokens
        if let Some(m) =
            search(r"create (a|an|one|two|three|four|five|x|that many|\d+) (\d+|x)/(\d+|x) ([^.]*?)creature tokens?", l)
        {
            let n = num(m.gs(1));
            let p = if m.gs(2) == "x" { 1 } else { m.gs(2).parse::<i64>().unwrap() };
            let fly = l.contains("flying");
            let dt = l.contains("deathtouch");
            if m.gs(1) == "x" && spell {
                add!("tokx");
            } else if castrig {
                add!("spelltok", p);
                if fly {
                    add!("spelltokfly");
                }
            } else if upkeep {
                add!("tokup", n);
            } else if attack {
                add!("tokatk", n);
            } else if spell {
                add!("mktok", format!("{n}:{p}:{}", fly as i64));
            } else {
                add!("tok", n);
                add!("tokp", p);
                if fly {
                    add!("tokfly");
                }
                if dt {
                    add!("tokdt");
                }
            }
            hit = true;
        }
        if let Some(m) = search(r"create (a|two|three|x) treasure tokens?", l) {
            if !castrig {
                add!("treas", num(m.gs(1)));
                hit = true;
            }
        }
        // --- spell-cast triggers
        if castrig {
            if let Some(m) =
                search(r"deals? (\d+) damage to (each opponent|target opponent|any target|each of up to)", l)
            {
                add!("ping", m.gs(1).parse::<i64>().unwrap());
                if !m.gs(2).contains("each opponent") {
                    add!("ral");
                }
            }
            if is(r"draw a card", l) {
                add!("spelldraw");
            }
            if is(r"create a treasure", l) {
                add!("kiln");
            }
            if is(r"create (a|an) (\d+)/(\d+)", l) && !tags.contains("spelltok") {
                let mm = search(r"create (a|an) (\d+)/(\d+)", l).unwrap();
                add!("spelltok", mm.gs(2).parse::<i64>().unwrap());
            }
            if creature && ["ping", "spelldraw", "spelltok", "kiln"].iter().any(|k| tags.contains(k)) {
                add!("noatk");
            }
            hit = true;
        }
        // --- drains, death triggers
        if let Some(m) = search(r"each opponent loses (\d+|x) life", l) {
            if etb || spell {
                add!("drainetb", num(m.gs(1)));
                hit = true;
            }
        }
        if is(r"whenever (~ or )?another creature (you control )?dies, each opponent loses 1 life", l) {
            add!("drain");
            hit = true;
        }
        if is(r"whenever (~ or )?another creature dies, target (player|opponent) loses 1 life", l) {
            add!("bartist");
            hit = true;
        }
        if is(r"each opponent sacrifices a creature", l) && (etb || spell) {
            add!("edictetb");
            hit = true;
        }
        // --- anthems and pumps
        if let Some(m) = search(r"(other )?creatures you control get \+(\d+)/\+(\d+)(?! until end of turn)", l) {
            let n = m.gs(2).parse::<i64>().unwrap();
            if !l.contains("until end of turn") {
                add!("anth", n);
            } else {
                add!("pumpall", n);
            }
            hit = true;
        }
        // --- protection
        if is(r"gain hexproof and indestructible until end of turn", l) {
            add!("prot", "hi");
            hit = true;
        } else if is(r"creatures you control gain indestructible until end of turn", l) {
            add!("prot", "indes");
            hit = true;
        } else if is(r"phases? out", l) && spell {
            add!("prot", "phase");
            hit = true;
        }
        if is(r"equipped creature has (hexproof|shroud)", l) {
            add!("prot", "boots");
            hit = true;
        }
        if is(r"^sacrifice (a|another) creature:", l) {
            add!("sac");
            hit = true;
        }
        if let Some(m) = search(r"flashback (\{[^ ]+\})", l) {
            add!("fb", conv_cost(&m.gs(1).to_uppercase()));
            hit = true;
        }
        if !hit && IGNORABLE.iter().any(|p| is(p, l)) {
            hit = true;
        }
        if !hit && rmatch(r"(equipped|enchanted) creature gets \+\d/\+\d", l).is_some() {
            hit = true;
        }
        if !hit {
            unparsed.push(raw.clone());
        }
    }
    if land {
        let basic: String = [("Plains", 'W'), ("Island", 'U'), ("Swamp", 'B'), ("Mountain", 'R'), ("Forest", 'G')]
            .iter()
            .filter(|(t, _)| subl.contains(t))
            .map(|(_, c)| *c)
            .collect();
        let produced = rec.strs("produced_mana");
        let pm: String = "WUBRG".chars().filter(|c| produced.contains(&c.to_string().as_str())).collect();
        let mut cols = mana_c.clone().unwrap_or_default();
        if cols != "A" {
            let all = format!("{cols}{basic}{pm}");
            let s: String = "WUBRG".chars().filter(|c| all.contains(*c)).collect();
            cols = if !s.is_empty() {
                s
            } else if cols.is_empty() || cols == "C" {
                "C".into()
            } else {
                cols
            };
        }
        tags.set("c", if cols == "A" || py::len(&cols) >= 5 { "A".to_string() } else { cols });
        let tl_low = text.to_lowercase();
        if tl_low.contains("enters tapped unless") || tl_low.contains("unless you control") {
            tags.set("ck", true);
        } else if tl_low.contains("enters tapped") {
            tags.set("t", true);
        }
        if is(r"sacrifice ~: search your library for a basic land", &tl_low) {
            tags.set("f", true);
            tags.set("c", "A");
        }
        unparsed.retain(|u| !is(r"enters tapped|\{t\}: add|pay 2 life", &u.to_lowercase()));
    }
    for b in back.into_iter().flatten() {
        if !b.is_empty() {
            unparsed.push(format!("[back face] {}", b.replace('\n', " / ")));
        }
    }
    AutoTags {
        types,
        cost,
        tags: tagstr(&tags),
        unparsed,
        notes,
        game_changer: rec.get("game_changer").map(|v| v.truthy()).unwrap_or(false),
    }
}
