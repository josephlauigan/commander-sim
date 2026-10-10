//! The compile step (Python's `cards/dsl.py`, the card-building part): AI hint tags from abilities, and a card built
//! from Oracle text.

use crate::autotag::tagstr;
use crate::cd::Cd;
use crate::dsl_parse::compile_card;
use crate::json::{J, Obj};

/// the tags a hand-written or autotagged card keeps when abilities are compiled
pub const SAFE_TAGS: [&str; 29] = [
    "pow", "tgh", "fly", "dt", "vig", "lifelink", "haste", "trample", "reach", "flash", "convoke", "leg", "human", "warrior",
    "shaman", "wizard", "c", "t", "ck", "f", "amt", "rock", "dork", "rite", "bomb", "fb", "perCreature", "noatk", "spectacle",
];

fn s<'a>(o: &'a Obj, k: &str) -> Option<&'a str> {
    o.get(k).and_then(|v| v.as_str())
}

fn int_or(v: Option<&J>, default: i64) -> i64 {
    match v {
        Some(J::Int(i)) => *i,
        _ => default,
    }
}

/// AI hint tags derived from abilities (what the card is *for*); execution stays in the interpreter
pub fn hints(abilities: &[Obj], types: &str) -> Obj {
    let mut h = Obj::new();
    let spell = types.contains('I') || types.contains('S');
    for a in abilities {
        match s(a, "static") {
            Some("uncounterable") => h.set("unc", true),
            Some("free_counter") => h.set("free", true),
            Some("sorcery_flash") => h.set("gandalf", true),
            _ => {}
        }
        if s(a, "type") != Some("spell") {
            continue;
        }
        let mut effs: Vec<Obj> = match a.get("effects") {
            Some(J::Arr(v)) => v.iter().filter_map(|x| x.as_obj().cloned()).collect(),
            _ => vec![],
        };
        for e in effs.clone() {
            if s(&e, "do") == Some("modal") {
                if let Some(J::Arr(modes)) = e.get("modes") {
                    for ms in modes {
                        if let J::Arr(v) = ms {
                            effs.extend(v.iter().filter_map(|x| x.as_obj().cloned()));
                        }
                    }
                }
            }
        }
        for e in &effs {
            let d = s(e, "do");
            // `e.get('what') or e.get('to') or {}`
            let sel: Obj = [e.get("what"), e.get("to")]
                .into_iter()
                .flatten()
                .find(|v| v.truthy())
                .and_then(|v| v.as_obj().cloned())
                .unwrap_or_default();
            let sel_kind = s(&sel, "sel");
            let filter: Obj = sel.get("filter").and_then(|v| v.as_obj().cloned()).unwrap_or_default();
            if matches!(d, Some("destroy" | "exile" | "bounce" | "tuck" | "damage"))
                && matches!(sel_kind, Some("target" | "any_target"))
                && !h.contains("rem")
            {
                let d = d.unwrap();
                if d != "damage" {
                    h.set("rem", d);
                } else {
                    let n = match e.get("n") {
                        Some(J::Int(i)) => *i,
                        _ => 3,
                    };
                    h.set("rem", format!("dmg{n}"));
                }
                let t = s(&filter, "type");
                let tgt = match t {
                    Some("creature") => "c",
                    Some("cp") => "cp",
                    Some("acp") | Some("ac") => "cap",
                    Some("ce") => "ce",
                    Some("nonland") => "nl",
                    Some("permanent") => "p",
                    Some("artifact") => "a",
                    Some("nonartifact_creature") => "cna",
                    _ => {
                        if d == "damage" {
                            "c"
                        } else {
                            "nl"
                        }
                    }
                };
                h.set("tgt", tgt);
                if sel_kind == Some("any_target") {
                    h.set("face", true);
                }
            }
            if matches!(d, Some("destroy" | "exile" | "bounce"))
                && sel_kind == Some("all")
                && matches!(s(&filter, "type"), Some("creature" | "nonland" | "permanent"))
            {
                h.set("wipe", match d.unwrap() {
                    "destroy" => "destroy",
                    "exile" => "exile",
                    _ => "evac",
                });
            }
            if d == Some("pump") && sel_kind == Some("all") && int_or(e.get("tgh"), 0) <= -4 {
                h.set("wipe", "minus");
            }
            if d == Some("counter_spell") {
                let f: Obj = e.get("filter").and_then(|v| v.as_obj().cloned()).unwrap_or_default();
                let ft = s(&f, "type").unwrap_or("any");
                let mut c = match ft {
                    "noncreature" => "nc",
                    "creature" => "cre",
                    "instant_or_sorcery" => "ise",
                    _ => "any",
                };
                if int_or(f.get("min_mv"), 0) >= 4 {
                    c = "mv4";
                }
                h.set("ctr", c);
                if e.get("unless").is_some_and(|v| v.truthy()) {
                    h.set("soft", e.get("unless").cloned().unwrap());
                }
            }
            if d == Some("draw") && spell {
                let add = match e.get("n") {
                    Some(J::Int(i)) => *i,
                    _ => 2,
                };
                let cur = int_or(h.get("draw"), 0);
                h.set("draw", cur + add);
            }
            if d == Some("search") && matches!(s(e, "to"), Some("hand" | "top")) {
                let f: Obj = e.get("filter").and_then(|v| v.as_obj().cloned()).unwrap_or_default();
                let t = s(&f, "type");
                if t != Some("land") {
                    h.set("tut", match t {
                        Some("creature") => "cre",
                        Some("instant_or_sorcery") => "is",
                        Some("artifact") => "art",
                        Some("enchantment") => "ench",
                        _ => "any",
                    });
                }
            }
            if d == Some("reanimate") {
                h.set("rean", if s(e, "from") == Some("any") { "animate" } else { "evil" });
            }
            if d == Some("token") && e.get("n") == Some(&J::Str("X".into())) {
                h.set("tokx", true);
            }
        }
    }
    h
}

/// CD for a card driven by the interpreter
pub fn build_cd(name: &str, rec_types: &str, rec_cost: &str, autotags: &Obj, oracle: &str, loyalty: Option<&str>) -> Cd {
    let (abilities, unparsed) = compile_card(name, oracle, rec_types, loyalty);
    let mut base = Obj::new();
    for (k, v) in autotags.iter() {
        if SAFE_TAGS.contains(&k) {
            base.set(k, v.clone());
        }
    }
    base.update(&hints(&abilities, rec_types));
    let mut cd = Cd::new(name, rec_types, rec_cost, &tagstr(&base));
    cd.dsl = if abilities.is_empty() { None } else { Some(abilities) };
    cd.unparsed = unparsed;
    cd.start_loyalty = loyalty.map(|s| s.to_string());
    cd.source = "scryfall+dsl".into();
    cd
}
