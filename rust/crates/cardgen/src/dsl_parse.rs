//! Compiling Oracle text into ability data (Python's `cards/dsl_parse.py`).
//!
//! A card is a list of abilities; every ability is plain JSON, built here as ordered objects (`Obj`) so the output
//! has the keys in the order the Python dicts had:
//!
//! ```text
//! {"type": "spell",     "effects": [...]}                                   instants / sorceries
//! {"type": "triggered", "event": "etb", "source": "self", "effects": [...]}  "When ~ enters, ..."
//! {"type": "activated", "cost": {"mana": "2B", "tap": true, "sac": "self"}, "effects": [...]}
//! {"type": "loyalty",   "loyalty": -2, "effects": [...]}                      planeswalkers
//! {"type": "static",    "static": "anthem", "filter": {...}, "pow": 1, "tgh": 1}
//! {"type": "replacement", "replace": "tokens", "multiplier": 2}
//! ```
//!
//! Effects are small building blocks (`{"do": "draw", "n": 2, "who": "you"}`, `{"do": "damage", ...}` ...); the
//! interpreter in sim-core reads them. Selectors are `{"sel": "target" | "all" | "self", "filter": {...}}`; numbers
//! are ints or names ("X", "creatures_you_control", "cards_in_hand", ...).

use crate::json::{J, Obj};
use crate::py;
use crate::re::{M, findall, fullmatch, is, rmatch, search, split, sub};
use crate::{jo, obj};
use fancy_regex::Regex;
use std::cell::RefCell;
use std::sync::LazyLock;

type Fx = Vec<Obj>;

const KW: [&str; 13] = [
    "flying",
    "trample",
    "haste",
    "vigilance",
    "lifelink",
    "deathtouch",
    "hexproof",
    "indestructible",
    "menace",
    "reach",
    "first strike",
    "double strike",
    "shroud",
];

thread_local! {
    /// the full line being parsed (some clauses need context from neighbours); like Python's module global it
    /// keeps its value between cards
    static CTX_LINE: RefCell<String> = const { RefCell::new(String::new()) };
}

fn set_ctx(l: &str) {
    CTX_LINE.with(|c| *c.borrow_mut() = l.to_string());
}

fn ctx_line() -> String {
    CTX_LINE.with(|c| c.borrow().clone())
}

/// a number word as ints, "X" for x / that many (Python's `n_`)
pub fn n_(s: &str) -> J {
    let s = s.trim().to_lowercase();
    if py::isdigit(&s) {
        return J::Int(s.parse().unwrap());
    }
    match s.as_str() {
        "a" | "an" | "one" => J::Int(1),
        "two" | "twice" => J::Int(2),
        "three" => J::Int(3),
        "four" => J::Int(4),
        "five" => J::Int(5),
        "six" => J::Int(6),
        "seven" => J::Int(7),
        "eight" => J::Int(8),
        "nine" => J::Int(9),
        "ten" => J::Int(10),
        "x" | "that many" => J::Str("X".into()),
        _ => J::Int(1),
    }
}

fn is_numw(s: &str) -> bool {
    matches!(
        s,
        "a" | "an"
            | "one"
            | "two"
            | "three"
            | "four"
            | "five"
            | "six"
            | "seven"
            | "eight"
            | "nine"
            | "ten"
            | "x"
            | "that many"
            | "twice"
    )
}

/// an int from a regex group that matched `\d+`
fn int(s: &str) -> i64 {
    s.parse().unwrap()
}

/// "{2}{U}" -> "2U" (X, tap, untap, energy dropped; hybrid takes the first colour)
pub fn mana_str(s: &str) -> String {
    let (mut gen_, mut pips) = (0i64, String::new());
    for sym in findall(r"\{([^}]+)\}", s) {
        let sym = sym.to_uppercase();
        if py::isdigit(&sym) {
            gen_ += int(&sym);
        } else if matches!(sym.as_str(), "X" | "T" | "Q" | "E") {
            continue;
        } else if sym == "C" {
            gen_ += 1;
        } else {
            let c = py::first(&sym, "/");
            if !c.is_empty() && "WUBRG".contains(c) {
                pips.push_str(c);
            }
        }
    }
    format!("{}{}", if gen_ != 0 { gen_.to_string() } else { String::new() }, pips)
}

// ------------------------------------------------------------------ filters and selectors
const TYPE_WORDS: [(&str, &str); 17] = [
    ("nonland permanent", "nonland"),
    ("artifact, creature, or planeswalker", "acp"),
    ("artifact or creature", "ac"),
    ("creature or planeswalker", "cp"),
    ("creature or enchantment", "ce"),
    ("artifact or enchantment", "ae"),
    ("nonartifact creature", "nonartifact_creature"),
    ("noncreature", "noncreature"),
    ("creature", "creature"),
    ("planeswalker", "planeswalker"),
    ("artifact", "artifact"),
    ("enchantment", "enchantment"),
    ("land", "land"),
    ("permanent", "permanent"),
    ("spell", "spell"),
    ("player", "player"),
    ("opponent", "opponent"),
];

pub fn filt(phrase: &str) -> Obj {
    let p = phrase.to_lowercase();
    let mut f = Obj::new();
    for (w, t) in TYPE_WORDS {
        if p.contains(w) {
            f.set("type", t);
            break;
        }
    }
    if p.contains("you don't control")
        || p.contains("an opponent controls")
        || p.contains("your opponents control")
        || p.contains("opponents control")
    {
        f.set("controller", "opp");
    } else if p.contains("you control") || p.contains("you own") || p.split_whitespace().next() == Some("your") {
        f.set("controller", "you");
    }
    if p.contains("another") || p.split_whitespace().any(|w| w == "other") {
        f.set("other", true);
    }
    if let Some(m) = search(r"mana value (\d+) or less", &p) {
        f.set("max_mv", int(m.gs(1)));
    }
    if let Some(m) = search(r"mana value (\d+) or greater", &p) {
        f.set("min_mv", int(m.gs(1)));
    }
    if let Some(m) = search(r"power (\d+) or less", &p) {
        f.set("max_pow", int(m.gs(1)));
    }
    if let Some(m) = search(r"power (\d+) or greater", &p) {
        f.set("min_pow", int(m.gs(1)));
    }
    if p.contains("nontoken") {
        f.set("nontoken", true);
    }
    if p.contains("nonlegendary") {
        f.set("nonlegendary", true);
    }
    if p.contains("legendary") && !p.contains("nonlegendary") {
        f.set("legendary", true);
    }
    if p.contains("tapped") {
        f.set("tapped", true);
    }
    if p.contains("with flying") {
        f.set("kw", "flying");
    }
    // --- Tiers 4/5 (urdragon): "all Dragon creatures" / "all non-Dragon creatures" (Crux of Fate)
    if search(r"\bnon-dragon\b", &p).is_some() {
        f.set("subtype", "non-dragon");
    } else if search(r"\bdragons?\b", &p).is_some() && !f.contains("subtype") {
        f.set("subtype", "dragon");
    }
    if search(r"(instant or sorcery|instant|sorcery)", &p).is_some() && !f.contains("type") {
        f.set("type", "instant_or_sorcery");
    }
    f
}

pub fn selector(phrase: &str) -> Obj {
    let p = phrase.trim().to_lowercase();
    if matches!(p.as_str(), "~" | "it" | "this creature" | "this permanent") {
        return obj! {"sel" => "self"};
    }
    if p.starts_with("each ") || p.starts_with("all ") {
        return obj! {"sel" => "all", "filter" => filt(&p)};
    }
    if p.contains("target") {
        return obj! {"sel" => "target", "filter" => filt(&p), "upto" => p.contains("up to")};
    }
    obj! {"sel" => "all", "filter" => filt(&p)}
}

pub fn who(phrase: &str) -> &'static str {
    let p = if phrase.is_empty() { "you".to_string() } else { phrase.trim().to_lowercase() };
    for (k, v) in [
        ("each other player", "each_opponent"),
        ("each opponent", "each_opponent"),
        ("target opponent", "target_opponent"),
        ("target player", "target_player"),
        ("each player", "each_player"),
        ("its controller", "target_controller"),
        ("that player", "event_player"),
        ("defending player", "defending_player"),
        ("you", "you"),
    ] {
        if p.starts_with(k) {
            return v;
        }
    }
    "you"
}

// ------------------------------------------------------------------ effect clauses
pub fn num_expr(s: &str) -> J {
    let s = s.trim().to_lowercase();
    if is_numw(&s) || py::isdigit(&s) {
        return n_(&s);
    }
    // counting phrases the interpreter can name (anything else falls through to a bare "X", which the audit flags)
    // --- Tiers 1/2 (isshin): "the number of attacking creatures" (Mishra, Claimed by Gix), "the number of other
    // attacking creatures" (Commissar Severina Raine): counted in an attack trigger
    if let Some(am) = rmatch(r"^(?:the number of )?(other )?attacking creatures(?: you control)?$", &s) {
        return if am.g(1).is_some() { "other_attacking_creatures" } else { "attacking_creatures" }.into();
    }
    // --- Tiers 1/2 (brago-meren): "each creature card in your graveyard" (Wight of the Reliquary)
    if rmatch(r"^(?:the number of )?creature cards? in your graveyard$", &s).is_some() {
        return "gy_creatures".into();
    }
    if s.contains("creature") && s.contains("died this turn") {
        return "creatures_died_this_turn".into();
    }
    if s.contains("destroyed this way") {
        return "destroyed_this_way".into();
    }
    if s.contains("instant") && s.contains("sorcery") && s.contains("graveyard") && s.contains("your") {
        return "gy_instants_sorceries".into();
    }
    if s.contains("colors among") && s.contains("you control") {
        return "colors_among_permanents".into();
    }
    if s.contains("sacrificed") && s.contains("mana value") {
        return "sacrificed_mv".into();
    }
    // --- Tiers 1/2 (tatyova-teysa): "the number of lands you control" (Beanstalk Giant), "the number of creature cards
    // in your graveyard" (Hallowed Spiritkeeper)
    if rmatch(r"^(?:the number of )?lands you control$", &s).is_some() {
        return "lands_you_control".into();
    }
    if rmatch(r"^(?:the number of )?creature cards in your graveyard$", &s).is_some() {
        return "gy_creatures".into();
    }
    if s.contains("creature") && s.contains("you control") {
        return "creatures_you_control".into();
    }
    if let Some(tm) = rmatch(
        r"^(?:the number of )?(elves|goblins|zombies|humans|ninjas|soldiers|warriors|knights|vampires|spirits|faeries|equipment) (you control|on the battlefield)$",
        &s,
    ) {
        return format!("{}{}", if tm.gs(2) == "you control" { "type_you:" } else { "type_all:" }, singular(tm.gs(1)))
            .into();
    }
    // --- Tier 3 (marwyn): "each Elf you control", "each Elf on the battlefield", "each attacking Elf you control",
    // "each +1/+1 counter on ~"
    if let Some(tm) = rmatch(
        r"^(?:the number of )?(attacking )?(elf|elves|goblin|zombie|human|ninja|soldier|warrior|knight|vampire|spirit|faerie|sliver|merfolk|dragon|angel|demon|dinosaur) (?:creatures )?(you control|on the battlefield)$",
        &s,
    ) {
        let t = singular(tm.gs(2));
        if tm.g(1).is_some() {
            return format!("attacking_type:{t}").into();
        }
        return format!("{}{t}", if tm.gs(3) == "you control" { "type_you:" } else { "type_all:" }).into();
    }
    // --- Tiers 1/2 (lathril): "Elves you control plus the number of Elf cards in your graveyard" (Abomination of
    // Llanowar)
    if let Some(tm) = rmatch(
        r"^(?:the number of )?(elves|goblins|zombies|humans|vampires|merfolk|dragons|slivers) you control plus the number of (?:elf|goblin|zombie|human|vampire|merfolk|dragon|sliver) cards in your graveyard$",
        &s,
    ) {
        return format!("type_you_gy:{}", singular(tm.gs(1))).into();
    }
    // --- Tiers 1/2 (windgrace): "the number of lands you control" (Lumra, Bellow of the Woods), "each land you
    // control"
    if rmatch(r"^(?:the number of )?lands? you control$", &s).is_some() {
        return "lands_you_control".into();
    }
    if rmatch(r"^\+1/\+1 counter on (?:~|it|this creature)$", &s).is_some() {
        return "counters_on_self".into();
    }
    if s.contains("card") && s.contains("hand") {
        return "cards_in_hand".into();
    }
    if s.contains("devotion to black") {
        return "devotion_B".into();
    }
    if s.contains("opponent") {
        return "opponents".into();
    }
    if s.contains("its power") {
        return "target_power".into();
    }
    if s.contains("its toughness") {
        return "target_toughness".into();
    }
    if s.contains("its mana value") {
        return "target_mv".into();
    }
    "X".into()
}

type RuleFn = fn(&M) -> Option<Fx>;

fn anchored(pat: &str) -> &'static Regex {
    crate::re::rx(&format!(r"\A(?:{pat})"))
}

fn eff(pairs: Obj) -> Fx {
    vec![pairs]
}

fn rules() -> &'static Vec<(&'static Regex, RuleFn)> {
    static R: LazyLock<Vec<(&'static Regex, RuleFn)>> =
        LazyLock::new(|| EFFECT_RULES.iter().map(|(p, f)| (anchored(p), *f)).collect());
    &R
}

fn matches_rule(rx: &Regex, s: &str) -> Option<M> {
    crate::re::captures_re(rx, s)
}

static EFFECT_RULES: &[(&str, RuleFn)] = &[
    // draw
    (r"^(?:you )?draws? (a|an|one|two|three|four|five|six|seven|x|\d+) (?:additional )?cards?$", |m| {
        Some(eff(obj! {"do" => "draw", "n" => n_(m.gs(1)), "who" => "you"}))
    }),
    (
        r"^(target player|each player|each opponent|target opponent|its controller|that player) draws? (a|one|two|three|x|\d+) cards?$",
        |m| Some(eff(obj! {"do" => "draw", "n" => n_(m.gs(2)), "who" => who(m.gs(1))})),
    ),
    (r"^(?:you )?draw a card for each (.+)$", |m| {
        Some(eff(obj! {"do" => "draw", "n" => num_expr(m.gs(1)), "who" => "you"}))
    }),
    (
        r"^(?:you )?draw (a|one|two|three|\d+) cards?(?:,)? (?:and|then) (?:you )?(?:lose|discard) (a|one|two|three|\d+) (life|cards?)$",
        |m| {
            let mut e = eff(obj! {"do" => "draw", "n" => n_(m.gs(1)), "who" => "you"});
            e.push(if m.gs(3) == "life" {
                obj! {"do" => "lose_life", "n" => n_(m.gs(2)), "who" => "you"}
            } else {
                obj! {"do" => "discard", "n" => n_(m.gs(2)), "who" => "you"}
            });
            Some(e)
        },
    ),
    // damage
    (
        r"^(?:~ )?deals? (\d+|x) damage to (any target|any other target|target creature or planeswalker|target creature|target player or planeswalker|target player|target opponent|each opponent|each creature your opponents control|each other creature|each creature|each player|each creature and each player)$",
        |m| {
            let n = n_(m.gs(1));
            let t = m.gs(2);
            let to = if matches!(t, "any target" | "any other target" | "target player or planeswalker") {
                obj! {"sel" => "any_target"}
            } else if matches!(t, "target player" | "target opponent" | "each opponent" | "each player") {
                obj! {"sel" => "player", "who" => who(t)}
            } else if t == "each creature and each player" {
                return Some(vec![
                    obj! {"do" => "damage", "n" => n.clone(), "to" => obj!{"sel" => "all", "filter" => obj!{"type" => "creature"}}},
                    obj! {"do" => "damage", "n" => n, "to" => obj!{"sel" => "player", "who" => "each_player"}},
                ]);
            } else {
                selector(t)
            };
            Some(eff(obj! {"do" => "damage", "n" => n, "to" => to}))
        },
    ),
    // destroy / exile
    (
        r"^(destroy|exile) (target|all|each|up to one target|another target|up to one other target|up to (?:one|two|three) target) ([^.]+?)(?:\. (?:they|it) can't be regenerated)?$",
        |m| {
            let what = selector(&format!("{} {}", m.gs(2), m.gs(3)));
            if m.gs(3).contains("card") && m.gs(3).contains("graveyard") {
                return Some(eff(
                    obj! {"do" => "exile_graveyard", "who" => if m.gs(3).contains("target") { "target_player" } else { "each_player" }},
                ));
            }
            if m.gs(2) == "target" && m.gs(3) == "player's graveyard" {
                // --- Tiers 4/5 (prosper): "exile target player's graveyard" (Rakdos Charm) is one graveyard
                return Some(eff(obj! {"do" => "exile_graveyard", "who" => "target_player"}));
            }
            if m.gs(3).contains("graveyard") {
                return Some(eff(obj! {"do" => "exile_graveyard", "who" => "each_player"}));
            }
            Some(eff(obj! {"do" => m.gs(1), "what" => what}))
        },
    ),
    (r"^return (target|all|each|up to one target|another target) ([^.]+?) to (?:its|their) owners?'s? hands?$", |m| {
        let ph = m.gs(2);
        if ph.contains("card") && ph.contains("graveyard") {
            return Some(eff(obj! {"do" => "regrow", "filter" => filt(py::first(ph, "card")), "from" => "yours"}));
        }
        Some(eff(obj! {"do" => "bounce", "what" => selector(&format!("{} {}", m.gs(1), ph))}))
    }),
    (
        r"^(?:return|put) (?:target |up to one target |another target )?([^.]*?)cards? from (your|a|an opponent's|target player's) graveyard (?:to|onto) the battlefield(?: under your control)?(?: tapped)?(?: with (?:a|two|three) (?:additional )?\+1/\+1 counters? on it)?$",
        |m| {
            let g1 = if m.gs(1).is_empty() { "creature" } else { m.gs(1) };
            let mut o =
                obj! {"do" => "reanimate", "filter" => filt(g1), "from" => if m.gs(2) == "your" { "yours" } else { "any" }};
            // --- Tiers 1/2 (kaalia-sythis): "return all enchantment cards from your graveyard to the battlefield"
            // (Resurgent Belief, Splendid Reclamation, World Shaper): every one of them, tapped if it says so
            if g1.starts_with("all ") && m.gs(2) == "your" {
                o.0.push(("all".to_string(), J::from(true)));
                if m.all().contains("battlefield tapped") {
                    o.0.push(("tapped".to_string(), J::from(true)));
                }
            }
            Some(eff(o))
        },
    ),
    (
        r"^return (?:target |up to one target |up to two target )?([^.]*?)cards? from your graveyard to your hand$",
        |m| {
            let g1 = if m.gs(1).is_empty() { "card" } else { m.gs(1) };
            Some(eff(obj! {"do" => "regrow", "filter" => filt(g1), "from" => "yours"}))
        },
    ),
    // search
    (
        r"^search your library for (an?|up to (?:one|two|three|x))\s?([^,]*?)\s?cards?(?: with [^,]*)?(?:,| and)? ?(?:reveal (?:it|them|those cards)(?:,| and)? ?)?(?:put (?:it|that card|them|those cards|one of them|one) (into your hand|onto the battlefield(?: tapped)?|into your graveyard|on top of your library)|then shuffle and put that card on top)?(.*)$",
        |m| {
            let what = m.gs(2);
            let rest = m.gs(4);
            let dest = match m.g(3) {
                Some(d) => d.to_string(),
                None => {
                    if format!("{}{}", m.all(), ctx_line()).contains("on top") {
                        "on top of your library".to_string()
                    } else {
                        "into your hand".to_string()
                    }
                }
            };
            let to = if dest.contains("hand") {
                "hand"
            } else if dest.contains("battlefield") {
                "battlefield"
            } else if dest.contains("graveyard") {
                "graveyard"
            } else {
                "top"
            };
            let n = if m.gs(1).contains("two") { 2 } else { 1 };
            let filter = if !what.is_empty() && what != "a" { filt(what) } else { Obj::new() };
            let mut e = obj! {"do" => "search", "filter" => filter, "to" => to, "n" => n, "tapped" => dest.contains("tapped") || rest.contains("tapped")};
            if what.contains("basic land") || search(r"\b(forest|plains|island|swamp|mountain|land)\b", what).is_some()
            {
                e.set("filter", obj! {"type" => "land", "basic" => what.contains("basic")});
            }
            let mut out = vec![];
            if rest.contains("the other into your hand") {
                e.set("n", 1);
                out.push(e);
                out.push(obj! {"do" => "search", "filter" => obj!{"type" => "land", "basic" => true}, "to" => "hand", "n" => 1});
            } else {
                out.push(e);
            }
            Some(out)
        },
    ),
    // tokens
    (
        r"^create (a|an|one|two|three|four|five|x|that many|\d+) (\d+|x)/(\d+|x) ([^.]*?)creature tokens?(?: with ([a-z, ]+?))?(?: that'?s? tapped and attacking[^.]*| named [^.]*)?(?:\. they gain [a-z]+ until end of turn)?$",
        |m| {
            let both = format!("{} {}", m.gs(4), m.gs(5));
            let kws: Vec<J> = KW.iter().filter(|k| both.contains(*k)).map(|k| J::from(*k)).collect();
            let types: Vec<J> = m
                .gs(4)
                .split_whitespace()
                .filter(|w| {
                    !matches!(
                        *w,
                        "white"
                            | "blue"
                            | "black"
                            | "red"
                            | "green"
                            | "colorless"
                            | "and"
                            | "artifact"
                            | "enchantment"
                            | "legendary"
                            | "snow"
                    )
                })
                .map(J::from)
                .collect();
            let mut e = obj! {
                "do" => "token", "n" => n_(m.gs(1)),
                "pow" => if m.gs(2) == "x" { 1 } else { int(m.gs(2)) },
                "tgh" => if m.gs(3) == "x" { 1 } else { int(m.gs(3)) },
                "keywords" => kws, "attacking" => m.all().contains("attacking"), "warrior" => m.gs(4).contains("warrior"),
                "types" => types
            };
            // --- Tiers 1/2 (tatyova-teysa): the token's colours as printed ("1/1 white Spirit"; "colorless": none),
            // for the cards that care (Teysa, Orzhov Scion's white and black creatures)
            let words: Vec<&str> = m.gs(4).split_whitespace().collect();
            let letters: String = [("white", 'W'), ("blue", 'U'), ("black", 'B'), ("red", 'R'), ("green", 'G')]
                .iter()
                .filter(|(w, _)| words.contains(w))
                .map(|(_, c)| *c)
                .collect();
            if !letters.is_empty() || words.contains(&"colorless") {
                e.set("color", letters);
            }
            Some(eff(e))
        },
    ),
    (r"^create (a|an|one|two|three|x|that many|\d+) (treasure|clue|food|blood) tokens?$", |m| {
        let k = m.gs(2);
        if k == "treasure" || k == "clue" {
            return Some(eff(obj! {"do" => k, "n" => n_(m.gs(1))}));
        }
        Some(eff(obj! {"do" => "noop"}))
    }),
    // life
    (r"^(?:you )?gain (\d+|x|that much) life$", |m| {
        Some(eff(obj! {"do" => "gain_life", "n" => n_(m.gs(1)), "who" => "you"}))
    }),
    (
        r"^(each opponent|target opponent|target player|each player|you|that player|its controller) (?:loses|lose) (\d+|x) life(?: and you gain (?:\d+|x|that much) life)?$",
        |m| {
            let mut e = eff(obj! {"do" => "lose_life", "n" => n_(m.gs(2)), "who" => who(m.gs(1))});
            if m.all().contains("you gain") {
                // --- Tiers 4/5 (urza-yawgmoth): "you gain 1 life" is 1 in all (Ayara, Cauldron of Essence, Shimmercreep);
                // only "that much" is per opponent
                let per = m.gs(1) == "each opponent" && m.all().contains("that much");
                e.push(obj! {"do" => "gain_life", "n" => n_(m.gs(2)), "who" => "you", "per_opponent" => per});
            }
            Some(e)
        },
    ),
    (
        r"^(?:its controller|that player|target player|you) gains? life equal to (?:its|that creature's) (power|toughness)$",
        |m| {
            // --- Tiers 1/2 (lathril): "whenever another Elf you control enters, you gain life equal to its toughness"
            // (Wolverine Riders): the creature that entered, and you gain it
            let l = ctx_line();
            if m.all().starts_with("you ") && l.starts_with("whenever ") && l.contains(" enters, ") {
                return Some(eff(obj! {"do" => "gain_life", "n" => format!("event_{}", m.gs(1)), "who" => "you"}));
            }
            Some(eff(obj! {"do" => "gain_life", "n" => format!("target_{}", m.gs(1)), "who" => "target_controller"}))
        },
    ),
    // counters, pump, grant
    (
        r"^put (a|an|one|two|three|x|\d+) \+1/\+1 counters? on (each creature you control|each other creature you control|target creature|~|target creature you control|each of up to two target creatures|it)$",
        |m| Some(eff(obj! {"do" => "counters", "n" => n_(m.gs(1)), "what" => selector(m.gs(2))})),
    ),
    (
        r"^(creatures you control|other creatures you control|target creature|~|target creature you control|each creature you control|up to one target creature) gets? \+(\d+|x)/\+(\d+|x)(?: and gains? ([a-z, ]+?))? until end of turn$",
        |m| {
            let mut e =
                eff(obj! {"do" => "pump", "pow" => n_(m.gs(2)), "tgh" => n_(m.gs(3)), "what" => selector(m.gs(1))});
            if let Some(g4) = m.g(4) {
                for k in KW {
                    if g4.contains(k) {
                        e.push(obj! {"do" => "grant", "keyword" => k, "what" => selector(m.gs(1))});
                    }
                }
            }
            Some(e)
        },
    ),
    (
        r"^(creatures you control|target creature|target creature you control|~|each creature you control|permanents you control|other permanents you control) gains? ([a-z, ]+?) until end of turn$",
        |m| {
            let v: Fx = KW
                .iter()
                .filter(|k| m.gs(2).contains(*k))
                .map(|k| obj! {"do" => "grant", "keyword" => *k, "what" => selector(m.gs(1))})
                .collect();
            if v.is_empty() { None } else { Some(v) }
        },
    ),
    // counter
    (
        r"^counter target ([a-z, ]*?)\s?spell(?: with mana value (\d+) or greater| that targets [^.]*)?(?: unless its controller pays \{(\d+)\})?$",
        |m| {
            let g1 = m.gs(1);
            let mut f = filt(&format!("{g1} spell"));
            let t = if g1.contains("noncreature") {
                "noncreature"
            } else if g1.trim() == "creature" {
                "creature"
            } else if g1.contains("instant") || g1.contains("sorcery") {
                "instant_or_sorcery"
            } else {
                "any"
            };
            f.set("type", t);
            if let Some(g2) = m.g(2) {
                f.set("min_mv", int(g2));
            }
            let mut e = obj! {"do" => "counter_spell", "filter" => f};
            if let Some(g3) = m.g(3) {
                e.set("unless", int(g3));
            }
            Some(eff(e))
        },
    ),
    (
        r"^(each opponent|target opponent|target player|each player|you|that player) (discards?|mills?) (a|one|two|three|x|\d+) cards?$",
        |m| {
            Some(eff(
                obj! {"do" => if m.gs(2).contains("discard") { "discard" } else { "mill" }, "n" => n_(m.gs(3)), "who" => who(m.gs(1))},
            ))
        },
    ),
    (r"^(?:you )?(discard|mill) (a|one|two|three|\d+) cards?$", |m| {
        Some(eff(obj! {"do" => m.gs(1), "n" => n_(m.gs(2)), "who" => "you"}))
    }),
    (
        r"^(each other player|each opponent|target opponent|target player|each player|defending player) sacrifices? (a|an|two) ([a-z ,]+?)(?: of their choice| with the (?:least|greatest) power[^.]*)?$",
        |m| Some(eff(obj! {"do" => "sacrifice", "who" => who(m.gs(1)), "filter" => filt(m.gs(3)), "n" => n_(m.gs(2))})),
    ),
    // poison counters (Prologue to Phyresis, Infectious Inquiry, Ichor Rats, Ajani's emblem)
    (
        r"^(each opponent|each other player|each player|target opponent|target player) gets? (a|an|one|two|three|\d+) poison counters?$",
        |m| Some(eff(obj! {"do" => "poison", "who" => who(m.gs(1)), "n" => n_(m.gs(2))})),
    ),
    (r"^proliferate (twice|three times)$", |m| {
        Some(eff(obj! {"do" => "proliferate", "n" => if m.gs(1) == "twice" { 2 } else { 3 }}))
    }),
    (
        r"^(?:scry|surveil) (\d+|x)$|^investigate$|^proliferate$|^you may pay \{\d+\}$|^shuffle$|^then shuffle$|^shuffle and put that card on top$|^reveal (?:it|them)$",
        |m| {
            let t = m.all();
            if t == "investigate" {
                return Some(eff(obj! {"do" => "clue", "n" => 1}));
            }
            if t == "proliferate" {
                return Some(eff(obj! {"do" => "proliferate"}));
            }
            if (t.starts_with("scry") || t.starts_with("surveil")) && m.g(1).is_some() && m.g(1) != Some("x") {
                return Some(eff(obj! {"do" => py::first(t, " "), "n" => int(m.gs(1))}));
            }
            Some(eff(obj! {"do" => "noop"}))
        },
    ),
    (r"^amass (?:orcs|zombies) (\d+|x)$", |m| Some(eff(obj! {"do" => "amass", "n" => n_(m.gs(1))}))),
    // --- Tiers 4/5 (winota): "the Ring tempts you" (Boromir, Fiery Inscription, Gollum, Witch-king)
    (r"^the ring tempts you$", |_| Some(eff(obj! {"do" => "ring_tempt"}))),
    (
        r"^untap (all creatures you control|target creature|all lands you control|~|all attacking creatures|up to (?:two|three|four|five) target lands|target permanent)$",
        |m| Some(eff(obj! {"do" => "untap", "what" => selector(m.gs(1))})),
    ),
    (
        r"^add ((?:\{[wubrgc]\})+|one mana of any color|two mana of any one color|three mana of any one color|x mana of any one color)$",
        |m| {
            let s = m.gs(1);
            let n = if s.contains('{') {
                s.matches('{').count() as i64
            } else {
                return Some(eff(obj! {"do" => "add_mana", "n" => n_(py::first(s, " "))}));
            };
            Some(eff(obj! {"do" => "add_mana", "n" => n}))
        },
    ),
    (
        r"^(any number of target nonland permanents you control|target creature you control|target permanent you control|target creature|permanents you control) phases? out$",
        |m| Some(eff(obj! {"do" => "phase_out", "what" => selector(m.gs(1))})),
    ),
    (
        r"^exile (target creature you control|another target creature you control|up to one target creature you control|target nonland permanent you control)(?:,)? then return (?:it|that card) to the battlefield under (?:its owner's|your) control$",
        |m| Some(eff(obj! {"do" => "blink", "what" => selector(m.gs(1))})),
    ),
    (
        r"^blink (up to one target creature you (?:control|own)|target creature you (?:control|own)|another target creature you (?:control|own))$",
        |m| Some(eff(obj! {"do" => "blink", "what" => selector(&m.gs(1).replace("you own", "you control"))})),
    ),
    (
        r"^after this (?:main )?phase, there is an additional combat phase(?: followed by an additional main phase)?$|^there is an additional combat phase after this phase$",
        |_| Some(eff(obj! {"do" => "extra_combat"})),
    ),
    (r"^the owner of target ([a-z ]+) shuffles it into their library$", |m| {
        Some(eff(obj! {"do" => "tuck", "what" => obj!{"sel" => "target", "filter" => filt(m.gs(1))}}))
    }),
    (r"^tap (target creature|up to one target creature|target permanent|all creatures your opponents control)$", |m| {
        Some(eff(obj! {"do" => "tap", "what" => selector(m.gs(1))}))
    }),
    // --- Tiers 1/2 (windgrace): "tap ~" (Seasoned Hallowblade: "Tap ~. It gains indestructible ..."); Soul of
    // Windgrace's "~ gains ... Tap it." is read in the activated-ability branch of `compile_card`
    (r"^tap ~$", |_| Some(eff(obj! {"do" => "tap", "what" => obj!{"sel" => "self"}}))),
    (
        r"^(?:its controller|that player) may search their library for a basic land card, put it onto the battlefield tapped$",
        |_| Some(eff(obj! {"do" => "opp_ramp"})),
    ),
    (r"^(?:its controller|that player) creates? a (\d+)/(\d+) [a-z ]+ creature token$", |m| {
        Some(eff(obj! {"do" => "opp_token", "pow" => int(m.gs(1))}))
    }),
    (r"^(?:you )?lose (\d+|x) life$", |m| Some(eff(obj! {"do" => "lose_life", "n" => n_(m.gs(1)), "who" => "you"}))),
    (r"^all creatures get -(\d+|x)/-(\d+|x) until end of turn$", |m| {
        Some(eff(obj! {
            "do" => "pump",
            "pow" => if m.gs(1) == "x" { -9 } else { -int(m.gs(1)) },
            "tgh" => if m.gs(2) == "x" { -9 } else { -int(m.gs(2)) },
            "what" => obj!{"sel" => "all", "filter" => obj!{"type" => "creature"}}
        }))
    }),
    (r"^creatures your opponents control get -(\d+)/-(\d+) until end of turn$", |m| {
        Some(eff(obj! {"do" => "pump", "pow" => -int(m.gs(1)), "tgh" => -int(m.gs(2)),
        "what" => obj!{"sel" => "all", "filter" => obj!{"type" => "creature", "controller" => "opp"}}}))
    }),
    (
        r"^(?:its controller|that player) may search their library for a basic land card, put it onto the battlefield(?: tapped)?$",
        |_| Some(eff(obj! {"do" => "opp_ramp"})),
    ),
    (r"^(?:its controller|that player) creates? (a|two|three) treasure tokens?$", |m| {
        Some(eff(obj! {"do" => "opp_treasure", "n" => n_(m.gs(1))}))
    }),
    (r"^(?:its controller|that player) (?:may )?draws? (?:up to )?(a|one|two|three) cards?$", |m| {
        Some(eff(obj! {"do" => "draw", "n" => n_(m.gs(1)), "who" => "target_controller"}))
    }),
    (r"^(?:you )?gain life equal to the life lost this way$", |_| {
        Some(eff(obj! {"do" => "gain_life", "n" => "last_lost", "who" => "you"}))
    }),
    (r"^(?:you )?gain (\d+|x) life for each (.+)$", |m| {
        Some(eff(obj! {"do" => "gain_life", "n" => num_expr(m.gs(2)), "who" => "you", "mult" => n_(m.gs(1))}))
    }),
    (
        r"^(?:you )?lose life equal to (?:its|that permanent's|that creature's|the sacrificed creature's) (mana value|power)$",
        |m| {
            Some(eff(
                obj! {"do" => "lose_life", "n" => if m.gs(1).contains("mana") { "target_mv" } else { "target_power" }, "who" => "you"},
            ))
        },
    ),
    // --- Tiers 4/5 (krenko)
    // "~ deals damage equal to the number of Goblins that entered the battlefield under your control this turn to any
    // target" (Hobgoblin Bandit Lord): counted by the interpreter (`type_entered_you:goblin`)
    (
        r"^~ deals damage equal to the number of (goblins|elves|zombies|humans|soldiers|warriors|knights) that entered (?:the battlefield )?under your control this turn to (any target|each opponent)$",
        |m| {
            let n = format!("type_entered_you:{}", singular(m.gs(1)));
            let to = if m.gs(2) == "any target" {
                obj! {"sel" => "any_target"}
            } else {
                obj! {"sel" => "player", "who" => "each_opponent"}
            };
            Some(eff(obj! {"do" => "damage", "n" => n, "to" => to}))
        },
    ),
    // "~ deals damage to target player or planeswalker equal to the number of Goblins you control" (Goblin War
    // Strike), "~ deals damage to any target equal to the number of creatures you control" (Massive Raid)
    (
        r"^~ deals damage to (any target|target player or planeswalker|target player|target opponent|each opponent|target creature) equal to (?:the number of )?(.+)$",
        |m| {
            let n = num_expr(m.gs(2));
            if n == J::from("X") {
                return None;
            }
            let t = m.gs(1);
            let to = if t == "any target" {
                obj! {"sel" => "any_target"}
            } else if t == "target creature" {
                selector(t)
            } else {
                obj! {"sel" => "player", "who" => who(t)}
            };
            Some(eff(obj! {"do" => "damage", "n" => n, "to" => to}))
        },
    ),
    // "add {R} for each Goblin on the battlefield" (Brightstone Ritual): a count of creatures only (not Rousing
    // Refrain's "each card in target opponent's hand")
    (r"^add \{([wubrgc])\} for each (.+)$", |m| {
        let n = num_expr(m.gs(2));
        if !n.as_str().is_some_and(|s| s.starts_with("type_") || s == "creatures_you_control") {
            return None;
        }
        Some(eff(obj! {"do" => "add_mana", "n" => n}))
    }),
    (
        r"^~ deals damage equal to (?:that spell's mana value|its power|the number of [^.]+) to (any target|each opponent|target creature|target opponent)$",
        |m| {
            let a = m.all();
            let n = if a.contains("spell") {
                "event_spell_mv"
            } else if a.contains("its power") {
                "source_power"
            } else {
                "X"
            };
            let g1 = m.gs(1);
            let to = if g1 == "any target" {
                obj! {"sel" => "any_target"}
            } else if g1.contains("opponent") {
                obj! {"sel" => "player", "who" => who(g1)}
            } else {
                selector(g1)
            };
            Some(eff(obj! {"do" => "damage", "n" => n, "to" => to}))
        },
    ),
    (
        r"^(?:until end of turn, )?creatures you control have base power and toughness x/x(?: and gain all creature types)?(?: until end of turn)?$",
        |_| Some(eff(obj! {"do" => "set_pt", "n" => "X"})),
    ),
    (
        r"^double the number of each kind of counter on (?:any number of target permanents|each permanent you control|target permanent)$",
        |_| Some(eff(obj! {"do" => "double_counters"})),
    ),
    (r"^(?:they|those creatures|it) gains? ([a-z, ]+?) until end of turn$", |m| {
        let v: Fx = KW
            .iter()
            .filter(|k| m.gs(1).contains(*k))
            .map(|k| obj! {"do" => "grant", "keyword" => *k, "what" => obj!{"sel" => "all", "filter" => obj!{"type" => "creature", "controller" => "you"}}})
            .collect();
        if v.is_empty() { None } else { Some(v) }
    }),
    (r"^untap all attacking creatures$", |_| {
        Some(eff(
            obj! {"do" => "untap", "what" => obj!{"sel" => "all", "filter" => obj!{"type" => "creature", "controller" => "you"}}},
        ))
    }),
    (
        r"^(each player|each opponent|target player) discards? (?:their|his or her) hand(?:,)? (?:then|and) draws? (?:seven|that many|\w+) cards?$",
        |m| Some(eff(obj! {"do" => "wheel", "who" => who(m.gs(1))})),
    ),
    (r"^if a creature is destroyed this way, you gain life equal to its toughness$", |_| {
        Some(eff(obj! {"do" => "gain_life", "n" => "target_toughness", "who" => "you"}))
    }),
    (r"^exile the top (card|two cards|three cards) of your library$", |m| {
        let n = match m.gs(1) {
            "card" => 1,
            "two cards" => 2,
            _ => 3,
        };
        Some(eff(obj! {"do" => "draw", "n" => n, "who" => "you"}))
    }),
    (
        r"^(?:you may |until (?:the )?end of (?:your next )?turn, you may )?(?:cast|play) (?:that card|those cards|it|them|the exiled cards?)(?: this turn| until the end of your next turn)?$",
        |_| Some(eff(obj! {"do" => "noop"})),
    ),
    (r"^(?:you may )?put (?:a|up to one) land card from your hand onto the battlefield(?: tapped)?$", |_| {
        Some(eff(obj! {"do" => "put_land"}))
    }),
    (r"^if you don't, ~ deals (\d+) damage to each opponent$", |_| Some(eff(obj! {"do" => "noop"}))),
    // --- Tier 3 (aurelia)
    // "~ deals 3 damage to each of your opponents" (Aurelia, the Law Above): each opponent
    (r"^(?:~ )?deals? (\d+|x) damage to each of your opponents$", |m| {
        Some(eff(obj! {"do" => "damage", "n" => n_(m.gs(1)), "to" => obj!{"sel" => "player", "who" => "each_opponent"}}))
    }),
    // --- Tier 3 (marwyn): "put a +1/+1 counter on target creature for each Elf you control" (Immaculate Magistrate)
    (
        r"^put (?:a|an|one) \+1/\+1 counter on (target creature|~|target creature you control|each creature you control) for each (.+)$",
        |m| Some(eff(obj! {"do" => "counters", "n" => num_expr(m.gs(2)), "what" => selector(m.gs(1))})),
    ),
    // --- Tiers 1/2 (isshin): "that creature gets +2/+2 until end of turn" (Fervent Charge; "it" in a "whenever a
    // creature you control attacks" trigger is rewritten to "that creature"): the creature the trigger is about
    (r"^that creature gets \+(\d+)/\+(\d+) until end of turn$", |m| {
        Some(eff(obj! {"do" => "pump", "pow" => n_(m.gs(1)), "tgh" => n_(m.gs(2)), "what" => obj!{"sel" => "event"}}))
    }),
    // "that creature deals 1 damage to each opponent" (Ingris Stingerquill): each opponent takes it
    (r"^that creature deals (\d+) damage to each opponent$", |m| {
        Some(eff(obj! {"do" => "damage", "n" => n_(m.gs(1)), "to" => obj!{"sel" => "player", "who" => "each_opponent"}}))
    }),
    // --- Tiers 4/5 (prosper): "each creature deals 1 damage to its controller" (Rakdos Charm)
    (r"^each creature deals 1 damage to its controller$", |_| Some(eff(obj! {"do" => "creature_backlash"}))),
    // --- Tiers 4/5 (chulane-kinnan): "untap up to five lands" (Peregrine Drake, Frantic Search): n of your tapped lands
    (r"^untap up to (two|three|four|five|six|seven) lands$", |m| {
        Some(eff(obj! {"do" => "untap", "n" => n_(m.gs(1)), "what" => selector("all lands you control")}))
    }),
    // --- Tiers 4/5 (heliod-zur): "you may have that player lose 1 life" (Suture Priest; "you may" is split off first)
    (r"^have (that player|target player|target opponent|each opponent) lose (\d+|x) life$", |m| {
        Some(eff(obj! {"do" => "lose_life", "n" => n_(m.gs(2)), "who" => who(m.gs(1))}))
    }),
    // --- Tiers 4/5 (edgar)
    // "put a +1/+1 counter on each Vampire you control" (Edgar Markov, Cordial Vampire, Indulgent Aristocrat), "each
    // other Vampire you control", "each attacking creature you control" (Drana, Liberator of Malakir)
    (
        r"^put (a|an|one|two|three|\d+) \+1/\+1 counters? on each (other )?(attacking )?([a-z]+) (?:creature )?you control$",
        |m| {
            let t = singular(m.gs(4));
            let mut f = obj! {"type" => "creature", "controller" => "you"};
            if m.g(2).is_some() {
                f.set("other", true);
            }
            if t != "creature" {
                f.set("subtype", t.as_str());
            }
            if m.g(3).is_some() {
                f.set("attacking", true);
            }
            Some(eff(obj! {"do" => "counters", "n" => n_(m.gs(1)), "what" => obj!{"sel" => "all", "filter" => f}}))
        },
    ),
    // "each opponent loses life equal to the number of Vampires you control" (Malakir Bloodwitch): a count the
    // interpreter names, else nothing
    (r"^(each opponent|target opponent|target player) loses life equal to (?:the number of )?(.+)$", |m| {
        let n = num_expr(m.gs(2));
        if n == J::from("X") {
            return None;
        }
        Some(eff(obj! {"do" => "lose_life", "n" => n, "who" => who(m.gs(1))}))
    }),
    // --- Tiers 1/2 (lathril): "each opponent blights 1" (High Perfect Morcant): each puts N -1/-1 counters on a creature
    // it controls
    (r"^(each opponent|target opponent|each player|you) blights? (one|two|three|\d+)$", |m| {
        Some(eff(obj! {"do" => "blight", "n" => n_(m.gs(2)), "who" => who(m.gs(1))}))
    }),
    // --- Tiers 1/2 (tatyova-teysa)
    // "you may play up to three additional lands this turn" (Summer Bloom), "you may play an additional land this
    // turn" (Urban Evolution): that many more land drops this turn
    (r"^(?:you may )?play (a|an|one|two|three|up to two|up to three) additional lands? this turn$", |m| {
        let k = m.gs(1).trim_start_matches("up to ");
        Some(eff(obj! {"do" => "extra_land", "n" => n_(k)}))
    }),
    // "sacrifice a land" as a spell's own effect (Roiling Regrowth, Entish Restoration): your least useful land
    (r"^sacrifice (a|an|one) land$", |_| {
        Some(eff(obj! {"do" => "sacrifice", "who" => "you", "filter" => obj!{"type" => "land"}, "n" => 1}))
    }),
    // "double the number of +1/+1 counters on ~" (Mossborn Hydra): as many more as it has
    (r"^double the number of \+1/\+1 counters on (?:~|this creature|it)$", |_| {
        Some(eff(obj! {"do" => "counters", "n" => "counters_on_self", "what" => obj!{"sel" => "self"}}))
    }),
    // "put a land card from your hand or graveyard onto the battlefield" (Bonny Pall, Clearcutter)
    (r"^(?:you may )?put (?:a|up to one) land card from your hand or graveyard onto the battlefield$", |_| {
        Some(eff(obj! {"do" => "put_land", "from" => "hand_or_gy"}))
    }),
];

/// how a clause depends on a "you may" earlier in the text
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Dep {
    /// independent of any "you may"
    None,
    /// the "you may" clause itself: the person is asked
    May,
    /// a ", then ..." continuation of a "you may" sentence: skipped when the person says no (the AI runs it as ever)
    Cont,
    /// the sentence after it, "If you do, ..." / "When you do, ...": runs only for a person who said yes
    IfYouDo,
}

/// the clauses of a sentence with their [`Dep`]: the leading "you may " is stripped; ", then ..." continuations of a
/// "you may" clause are `Cont`; a following sentence starting "if you do, " / "when you do, " is `IfYouDo` (prefix
/// stripped); the group ends at ". " or "; "
pub fn split_clauses_may(sentence: &str) -> Vec<(String, Dep, &'static str)> {
    let s = py::strip(py::strip(sentence, " \t\n\r").trim(), ".");
    let s = s.trim();
    let mut out = vec![];
    let mut group = false; // the previous sentence was a "you may" one
    for part in split(r"(?:\. |; )", s) {
        let mut part = part.trim().to_string();
        let mut if_do = "";
        for pre in ["if you do, ", "when you do, "] {
            if group && part.starts_with(pre) {
                part = part[pre.len()..].to_string();
                if_do = pre;
                break;
            }
        }
        let mut may = false;
        for p in split(r",? then ", &part) {
            let mut p = py::strip(p.trim(), ",").trim().to_string();
            let mut dep = if !if_do.is_empty() {
                Dep::IfYouDo
            } else if may {
                Dep::Cont
            } else {
                Dep::None
            };
            if if_do.is_empty() && p.starts_with("you may ") {
                p = p[8..].to_string();
                may = true;
                dep = Dep::May;
            }
            if p.starts_with("then ") {
                p = p[5..].to_string();
            }
            if !p.is_empty() {
                out.push((p, dep, if_do));
            }
        }
        group = may;
    }
    out
}

pub fn split_clauses(sentence: &str) -> Vec<String> {
    split_clauses_may(sentence).into_iter().map(|(c, _, _)| c).collect()
}

const SUBJ: &str = r"^(each other player|target opponent|each opponent|target player|each player|that player|its controller|defending player) ";

/// returns (effects, unparsed clauses)
pub fn parse_effects(text: &str) -> (Fx, Vec<String>) {
    let mut effs: Fx = vec![];
    let mut bad: Vec<String> = vec![];
    let mut text = sub(r" at the beginning of the next turn's upkeep", "", text);
    text = sub(
        r"exile ((?:up to one |another )?target creature you (?:control|own)),? then return (?:it|that card) to the battlefield under (?:its owner's|your) control",
        r"blink \1",
        &text,
    );
    text = sub(
        r"exile ((?:up to one |another )?target creature you (?:control|own)),? then search your library for (an?|up to one) (basic land|land) card\.? put both cards onto the battlefield(?: tapped)?(?: under your control)?",
        r"blink \1. search your library for a \3 card, put it onto the battlefield",
        &text,
    );
    let mut xexpr: Option<J> = None;
    if let Some(xm) = search(r",? where x is (.+?)(?=\.|$)", &text) {
        xexpr = Some(num_expr(xm.gs(1)));
        text = format!("{}{}", &text[..xm.start()], &text[xm.end()..]);
    }
    // "target opponent sacrifices a creature, discards a card, and loses 3 life" -> one subject, several verbs
    let mut sents: Vec<String> = text.split(". ").map(|s| s.to_string()).collect();
    for sen in sents.iter_mut() {
        if let Some(sm) = rmatch(&format!("{SUBJ}(.+)$"), sen) {
            if sm.gs(2).contains(", ") {
                let verbs: Vec<String> = split(r",? and |, ", sm.gs(2))
                    .into_iter()
                    // --- Tier 3 (korvold): "each opponent discards a card, then mills a card" (Vicious Rumors),
                    // "..., then sacrifices a land of their choice" (Smallpox): the last verb keeps the subject (only
                    // these verbs: "then draws" / "then you ..." keep their old reading)
                    .map(|v| {
                        let v = v.trim();
                        match v.strip_prefix("then ") {
                            Some(r) if is(r"^(discards?|mills?|sacrifices?|loses?) ", r) => r.to_string(),
                            _ => v.to_string(),
                        }
                    })
                    .filter(|v| !v.is_empty())
                    .collect();
                *sen = verbs.iter().map(|v| format!("{} {}", sm.gs(1), v)).collect::<Vec<_>>().join(". ");
            }
        }
    }
    text = sents.join(". ");
    let mut per_opp = false;
    if text.starts_with("for each opponent, ") {
        text = text["for each opponent, ".len()..].to_string();
        per_opp = true;
    }
    for (cl, dep, pre) in split_clauses_may(&text) {
        let mut got: Option<Fx> = None;
        for (rx, f) in rules() {
            if let Some(m) = matches_rule(rx, &cl) {
                let r = f(&m);
                if r.as_ref().is_some_and(|v| !v.is_empty()) {
                    got = r;
                    break;
                }
            }
        }
        if got.is_none() && cl.contains(" and ") {
            let (a, b) = cl.split_once(" and ").unwrap();
            let (ea, ba) = parse_effects(a);
            let (eb, bb) = parse_effects(b);
            if !ea.is_empty() && !eb.is_empty() && ba.is_empty() && bb.is_empty() {
                let mut v = ea;
                v.extend(eb);
                got = Some(v);
            }
        }
        // --- Tiers 1/2 (tatyova-teysa): "you may sacrifice a land" (Tenured Tethermage) is a choice the AI would not
        // weigh (it ignores the "may"): left to card code, as before the plain "sacrifice a land" compiled
        let own_sac = |e: &Obj| {
            e.get("do") == Some(&J::Str("sacrifice".into())) && e.get("who") == Some(&J::Str("you".into()))
        };
        if dep == Dep::May && got.as_ref().is_some_and(|v| v.iter().any(own_sac)) {
            bad.push(format!("{pre}{cl}"));
            continue;
        }
        if let Some(got) = got {
            for mut e in got {
                if e.get("do") != Some(&J::Str("noop".into())) {
                    match dep {
                        Dep::May => e.set("optional", true),
                        Dep::Cont => e.set("cont", true),
                        Dep::IfYouDo | Dep::None => {}
                    }
                }
                if let Some(x) = &xexpr {
                    for k in ["n", "pow", "tgh"] {
                        if e.get(k) == Some(&J::Str("X".into())) {
                            e.set(k, x.clone());
                        }
                    }
                }
                if per_opp && e.get("do") == Some(&J::Str("token".into())) {
                    e.set("n", "opponents");
                }
                if e.get("do") == Some(&J::Str("noop".into())) {
                    continue;
                }
                if dep == Dep::IfYouDo {
                    // nested in the "you may" effect it depends on (invisible to everything that reads the flat list:
                    // the AI never sees it); with no such effect the clause stays unparsed, as before
                    let head = effs.iter_mut().rev().find(|h| h.get("optional") == Some(&J::Bool(true)));
                    match head {
                        Some(h) => {
                            let mut v = match h.get("if_you_do") {
                                Some(J::Arr(v)) => v.clone(),
                                _ => vec![],
                            };
                            v.push(J::Obj(e));
                            h.set("if_you_do", J::Arr(v));
                        }
                        None => bad.push(format!("{pre}{cl}")),
                    }
                } else {
                    effs.push(e);
                }
            }
        } else if rmatch(r"^(it|they) (?:can't be regenerated|gains? haste until end of turn)$", &cl).is_some() {
        } else {
            bad.push(format!("{pre}{cl}"));
        }
    }
    (effs, bad)
}

// ------------------------------------------------------------------ triggers, costs, statics
type TrigFn = fn(&M, &str) -> Fx;

fn cast_spell(m: &M, group: usize) -> String {
    let g = m.g(group).filter(|s| !s.is_empty()).unwrap_or("any");
    g.replace(' ', "_")
}

static TRIGGERS: &[(&str, TrigFn)] = &[
    (r"^~ enters(?: the battlefield)?(?: or attacks)?$", |_, t| {
        let mut v = vec![obj! {"event" => "etb", "source" => "self"}];
        if t.contains("attacks") {
            v.push(obj! {"event" => "attack", "source" => "self"});
        }
        v
    }),
    (r"^~ attacks$", |_, _| vec![obj! {"event" => "attack", "source" => "self"}]),
    (r"^~ dies$", |_, _| vec![obj! {"event" => "dies", "source" => "self"}]),
    (r"^~ or another creature (you control )?dies$", |m, _| {
        vec![obj! {"event" => "dies", "source" => if m.g(1).is_some() { "you_creature" } else { "any_creature" }}]
    }),
    (r"^(a|another|one or more other|one or more) (nontoken )?creatures? you control (?:enters?|enter)$", |m, _| {
        vec![
            obj! {"event" => "etb", "source" => "you_creature", "other" => m.gs(1).contains("other") || m.gs(1).contains("another")},
        ]
    }),
    (r"^(a|another) creature (enters)$", |m, _| {
        vec![obj! {"event" => "etb", "source" => "any_creature", "other" => m.gs(1) == "another"}]
    }),
    (r"^a creature an opponent controls enters$|^a creature enters under an opponent's control$", |_, _| {
        vec![obj! {"event" => "etb", "source" => "opp_creature"}]
    }),
    (r"^a land (?:you control )?enters(?: under your control)?$", |_, _| vec![obj! {"event" => "landfall"}]),
    (r"^(a|another) creature (you control )?dies$", |m, _| {
        vec![
            obj! {"event" => "dies", "source" => if m.g(2).is_some() { "you_creature" } else { "any_creature" }, "other" => m.gs(1) == "another"},
        ]
    }),
    (r"^a creature an opponent controls dies$|^a creature your opponents control dies$", |_, _| {
        vec![obj! {"event" => "dies", "source" => "opp_creature"}]
    }),
    (
        r"^you cast (?:or copy )?(a|an|your first|your second) ?(instant or sorcery|noncreature|creature|artifact|enchantment|legendary|historic|multicolored|)? ?spell(?: each turn)?$",
        |m, _| {
            vec![obj! {"event" => "cast", "who" => "you", "spell" => cast_spell(m, 2),
            "nth" => if m.gs(1).contains("second") { J::Int(2) } else { J::Null }}]
        },
    ),
    (
        r"^(an opponent|a player|another player) casts (a|an) ?(noncreature|creature|instant or sorcery|)? ?spell$",
        |m, _| {
            vec![
                obj! {"event" => "cast", "who" => if m.gs(1).contains("opponent") || m.gs(1).contains("another") { "opponent" } else { "any" },
                "spell" => cast_spell(m, 3)},
            ]
        },
    ),
    (r"^you draw a card$|^you draw your second card each turn$", |_, _| vec![obj! {"event" => "draw", "who" => "you"}]),
    (r"^an opponent draws a card$", |_, _| vec![obj! {"event" => "draw", "who" => "opponent"}]),
    (r"^the beginning of your upkeep$", |_, _| vec![obj! {"event" => "upkeep", "who" => "you"}]),
    (r"^the beginning of each opponent's upkeep$", |_, _| vec![obj! {"event" => "upkeep", "who" => "opponent"}]),
    (r"^the beginning of each upkeep$", |_, _| vec![obj! {"event" => "upkeep", "who" => "any"}]),
    (r"^the beginning of your end step$", |_, _| vec![obj! {"event" => "end_step", "who" => "you"}]),
    (r"^the beginning of each end step$", |_, _| vec![obj! {"event" => "end_step", "who" => "any"}]),
    (
        r"^the beginning of combat on your turn$|^the beginning of your precombat main phase$|^the beginning of your draw step$",
        |_, _| vec![obj! {"event" => "upkeep", "who" => "you"}],
    ),
    (r"^you attack(?: with one or more creatures)?$|^one or more creatures you control attack$", |_, _| {
        vec![obj! {"event" => "attack", "source" => "you_any"}]
    }),
    (r"^a creature you control attacks$|^(?:a|another) (?:nontoken )?creature you control attacks$", |_, _| {
        vec![obj! {"event" => "attack", "source" => "you_each"}]
    }),
    (r"^~ deals combat damage to a player$|^~ deals combat damage to a player or planeswalker$", |_, _| {
        vec![obj! {"event" => "combat_damage", "source" => "self"}]
    }),
    (r"^(?:a|one or more) creatures? you control deals? combat damage to a player$", |_, _| {
        vec![obj! {"event" => "combat_damage", "source" => "you_creature"}]
    }),
    (r"^equipped creature deals combat damage to a player$", |_, _| {
        vec![obj! {"event" => "combat_damage", "source" => "equipped"}]
    }),
    (r"^equipped creature attacks$", |_, _| vec![obj! {"event" => "attack", "source" => "equipped"}]),
    (r"^equipped creature dies$", |_, _| vec![obj! {"event" => "dies", "source" => "equipped"}]),
    (r"^you gain life$", |_, _| vec![obj! {"event" => "gain_life", "who" => "you"}]),
    (r"^you proliferate$", |_, _| vec![obj! {"event" => "proliferate", "who" => "you"}]), // Ezuri, Voidwing Hybrid
    (r"^~ becomes tapped$", |_, _| vec![obj! {"event" => "attack", "source" => "self"}]),
    (r"^you cast your (third|second) spell each turn$", |m, _| {
        vec![
            obj! {"event" => "cast", "who" => "you", "spell" => "any", "nth" => if m.gs(1) == "third" { 3 } else { 2 }},
        ]
    }),
    (r"^you cast a spell$", |_, _| vec![obj! {"event" => "cast", "who" => "you", "spell" => "any"}]),
    (r"^a creature you control with power (\d+) or greater enters$", |m, _| {
        vec![obj! {"event" => "etb", "source" => "you_creature", "min_pow" => int(m.gs(1))}]
    }),
    (r"^a warrior attacks$|^a (\w+) you control attacks$", |m, _| {
        vec![obj! {"event" => "attack", "source" => "you_each", "subtype" => m.g(1).unwrap_or("warrior")}]
    }),
    // (Tier 3 aurelia: the count is kept, `min_atk`; it was dropped)
    (r"^you attack with (two|three) or more creatures$", |m, _| {
        vec![obj! {"event" => "attack", "source" => "you_any", "min_atk" => n_(m.gs(1))}]
    }),
    // --- Tier 3 (aurelia)
    // battalion: "~ and at least two other creatures attack" (Firemane Avenger, Boros Elite)
    (r"^~ and at least (two|three) other creatures attack$", |m, _| {
        vec![obj! {"event" => "attack", "source" => "self", "min_atk" => n_(m.gs(1)).as_i64().unwrap_or(2) + 1}]
    }),
    // --- Tiers 4/5 (krenko): "one or more Goblins you control attack" (General Kreat): once per attack that has one
    (r"^one or more (goblins|elves|zombies|humans|soldiers|warriors|knights|dragons|vampires) you control attack$", |m, _| {
        vec![obj! {"event" => "attack", "source" => "you_any", "subtype" => singular(m.gs(1))}]
    }),
    // "a player attacks with N or more creatures" (Aurelia, the Law Above): anyone's attack, counted whole
    (r"^a player attacks with (two|three|four|five|six) or more creatures$", |m, _| {
        vec![obj! {"event" => "attack", "source" => "any_player", "min_atk" => n_(m.gs(1))}]
    }),
    // --- Tiers 4/5 (heliod-zur): "another creature you control enters or dies" (Daxos, Blessed by the Sun)
    (r"^(a|another) creature you control enters or dies$", |m, _| {
        let other = m.gs(1) == "another";
        vec![
            obj! {"event" => "etb", "source" => "you_creature", "other" => other},
            obj! {"event" => "dies", "source" => "you_creature", "other" => other},
        ]
    }),
    // --- Tiers 4/5 (urza-yawgmoth): "~ or another black creature you control enters" (Ayara, First of Locthwain): itself
    // and the creatures of that colour entering under your control
    (r"^~ or another (white|blue|black|red|green) creature you control enters$", |m, _| {
        let col = match m.gs(1) {
            "white" => "W",
            "blue" => "U",
            "black" => "B",
            "red" => "R",
            _ => "G",
        };
        vec![obj! {"event" => "etb", "source" => "you_creature", "color" => col}]
    }),
    // --- Tiers 1/2 (lathril): "an Elf you control dies" (Elderfang Venom), "another Elf you control enters" (Wolverine
    // Riders), "~ or another Elf you control enters" (High Perfect Morcant): your creatures of that type
    (
        r"^(~ or another|a|an|another) (elf|goblin|zombie|human|vampire|merfolk|dragon|angel|sliver|knight|soldier|wizard|cleric|rogue|ninja|faerie|spirit|dinosaur|squirrel|rat|cat|dog|dwarf) you control (dies|enters)$",
        |m, _| {
            let ev = if m.gs(3) == "dies" { "dies" } else { "etb" };
            vec![obj! {"event" => ev, "source" => "you_creature", "other" => m.gs(1) == "another", "subtype" => m.gs(2)}]
        },
    ),
];

pub fn parse_trigger(clause: &str) -> Option<Fx> {
    static T: LazyLock<Vec<(&'static Regex, TrigFn)>> =
        LazyLock::new(|| TRIGGERS.iter().map(|(p, f)| (anchored(p), *f)).collect());
    let c = sub(r", if [^,]*$", "", clause.trim());
    for (rx, f) in T.iter() {
        if let Some(m) = matches_rule(rx, &c) {
            return Some(f(&m, &c));
        }
    }
    None
}

pub fn parse_cost(s: &str) -> Option<Obj> {
    let mut cost = Obj::new();
    for x in s.split(',').map(|x| x.trim()) {
        if fullmatch(r"(\{[^}]+\})+", x).is_some() {
            let ms = mana_str(x);
            if x.contains("{t}") {
                cost.set("tap", true);
            }
            if !ms.is_empty() {
                cost.set("mana", ms);
            }
            if x.contains("{e}") {
                cost.set("energy", true);
            }
        } else if x == "{t}" {
            cost.set("tap", true);
        } else if x.starts_with("sacrifice ~") || x.starts_with("sacrifice this") {
            cost.set("sac", "self");
        } else if rmatch(r"^sacrifice (two|three|four|five|x|\d+) ", x).is_some() {
            // --- Tiers 1/2 (tatyova-teysa): a cost of several permanents (Teysa, Orzhov Scion's "sacrifice three
            // white creatures") is not one sacrifice: leave the line to card code
            return None;
        } else if x.starts_with("sacrifice") {
            let f = filt(x);
            cost.set("sac", f.get("type").cloned().unwrap_or("permanent".into()));
        } else if x.starts_with("pay") && x.contains("life") {
            let m = search(r"pay (\w+) life", x)?; // variable life cost (War Room): leave the line unmodeled
            cost.set("life", n_(m.gs(1)));
        } else if x.starts_with("discard") {
            cost.set("discard", n_(x.split_whitespace().nth(1)?));
            // --- Tiers 1/2 (windgrace): "discard a land card" / "discard a creature card" (Soul of Windgrace, Fauna
            // Shaman): only a card of that type pays
            if let Some(m) = search(r"^discard (?:a|an|one|two|\d+) (land|creature|artifact|enchantment) cards?$", x) {
                cost.set("discard_type", m.gs(1));
            }
        } else if x.starts_with("exile ~ from your graveyard") || x.starts_with("remove") {
            cost.set("other", x);
        } else if x.starts_with("tap an untapped") || x.starts_with("tap ") {
            cost.set("tap_other", x);
        } else {
            return None;
        }
    }
    Some(cost)
}

fn kw_statics(text: Option<&str>, mut mk: impl FnMut(&str) -> Obj) -> Fx {
    match text {
        Some(t) => KW.iter().filter(|k| t.contains(*k)).map(|k| mk(k)).collect(),
        None => vec![],
    }
}

type StaticFn = fn(&M) -> Option<Fx>;

static STATICS: &[(&str, StaticFn)] = &[
    // --- Tiers 4/5 (krenko): "other Goblins get +1/+1 and have mountainwalk" (Goblin King): every player's Goblins
    // (no "you control"); a landwalk the keyword list lacks is card code (impls/t45_krenko.rs)
    (
        r"^other (elves|goblins|zombies|humans|soldiers|warriors|knights|merfolk|vampires|spirits|faeries) get \+(\d+)/\+(\d+)(?: and have ([a-z, ]+))?$",
        |m| {
            let sub_ = singular(m.gs(1));
            let mut v = vec![obj! {"type" => "static", "static" => "anthem",
            "filter" => obj!{"type" => "creature", "other" => true, "subtype" => sub_.clone()},
            "pow" => int(m.gs(2)), "tgh" => int(m.gs(3))}];
            v.extend(kw_statics(m.g(4), |k| {
                obj! {"type" => "static", "static" => "keyword", "keyword" => k,
                "filter" => obj!{"type" => "creature", "other" => true, "subtype" => sub_.clone()}}
            }));
            Some(v)
        },
    ),
    // --- Tier 3 (marwyn): "other Elf creatures you control get +1/+1 for each +1/+1 counter on ~" (Joraga Warcaller)
    (r"^(other )?([a-z ]*?)creatures you control get \+(\d+)/\+(\d+) for each (.+)$", |m| {
        let per = num_expr(m.gs(5));
        if per == J::from("X") {
            return None;
        }
        let mut filter = obj! {"type" => "creature", "controller" => "you", "other" => m.g(1).is_some()};
        if !m.gs(2).trim().is_empty() {
            filter.set("subtype", m.gs(2).trim());
        }
        Some(eff(obj! {"type" => "static", "static" => "anthem", "filter" => filter, "pow" => int(m.gs(3)),
        "tgh" => int(m.gs(4)), "per" => per}))
    }),
    // --- Tier 3 (marwyn): "green spells you cast cost {1} less to cast" (the Medallions)
    (r"^(white|blue|black|red|green) spells you cast cost \{(\d+)\} less to cast$", |m| {
        Some(eff(obj! {"type" => "static", "static" => "cost", "who" => "you", "spell" => m.gs(1),
        "amount" => -int(m.gs(2))}))
    }),
    (r"^(other )?([a-z ]*?)creatures you control get \+(\d+)/\+(\d+)(?: and have ([a-z, ]+))?$", |m| {
        let mut filter = obj! {"type" => "creature", "controller" => "you", "other" => m.g(1).is_some()};
        if !m.gs(2).trim().is_empty() {
            filter.set("subtype", m.gs(2).trim());
        }
        let mut v = vec![
            obj! {"type" => "static", "static" => "anthem", "filter" => filter, "pow" => int(m.gs(3)), "tgh" => int(m.gs(4))},
        ];
        v.extend(kw_statics(m.g(5), |k| {
            obj! {"type" => "static", "static" => "keyword", "keyword" => k, "filter" => obj!{"type" => "creature", "controller" => "you"}}
        }));
        Some(v)
    }),
    (
        r"^(other )?(elves|goblins|zombies|humans|ninjas|soldiers|warriors|knights|angels|demons|dragons|merfolk|vampires|spirits|faeries) you control get \+(\d+)/\+(\d+)(?: and have ([a-z, ]+))?$",
        |m| {
            let sub_ = singular(m.gs(2));
            let mut v = vec![obj! {"type" => "static", "static" => "anthem",
            "filter" => obj!{"type" => "creature", "controller" => "you", "other" => m.g(1).is_some(), "subtype" => sub_.clone()},
            "pow" => int(m.gs(3)), "tgh" => int(m.gs(4))}];
            v.extend(kw_statics(m.g(5), |k| {
                obj! {"type" => "static", "static" => "keyword", "keyword" => k,
                "filter" => obj!{"type" => "creature", "controller" => "you", "subtype" => sub_.clone()}}
            }));
            Some(v)
        },
    ),
    (
        r"^other (elf|goblin|zombie|human|ninja|soldier|warrior|knight|angel|demon|dragon|vampire|spirit|faerie) creatures get \+(\d+)/\+(\d+)(?: and have [a-z ]+)?$",
        |m| {
            Some(eff(obj! {"type" => "static", "static" => "anthem",
            "filter" => obj!{"type" => "creature", "other" => true, "subtype" => m.gs(1)}, "pow" => int(m.gs(2)), "tgh" => int(m.gs(3))}))
        },
    ),
    (r"^creatures your opponents control get -(\d+)/-(\d+)$", |m| {
        Some(eff(
            obj! {"type" => "static", "static" => "anthem", "filter" => obj!{"type" => "creature", "controller" => "opp"},
            "pow" => -int(m.gs(1)), "tgh" => -int(m.gs(2))},
        ))
    }),
    (r"^(other )?creatures you control have ([a-z, ]+)$", |m| {
        let v = kw_statics(m.g(2), |k| {
            obj! {"type" => "static", "static" => "keyword", "keyword" => k,
            "filter" => obj!{"type" => "creature", "controller" => "you", "other" => m.g(1).is_some()}}
        });
        if v.is_empty() { None } else { Some(v) }
    }),
    (r"^equipped creature gets \+(\d+)/\+(\d+)(?: and has ([a-z, ]+))?$", |m| {
        let mut v =
            vec![obj! {"type" => "static", "static" => "equip_bonus", "pow" => int(m.gs(1)), "tgh" => int(m.gs(2))}];
        v.extend(kw_statics(m.g(3), |k| obj! {"type" => "static", "static" => "equip_keyword", "keyword" => k}));
        if let Some(g3) = m.g(3) {
            if g3.contains("protection from") {
                v.push(obj! {"type" => "static", "static" => "equip_protection", "colors" => prot_colors(g3)});
            }
        }
        Some(v)
    }),
    (r"^equipped creature has ([a-z, ]+)$", |m| {
        let v = kw_statics(m.g(1), |k| obj! {"type" => "static", "static" => "equip_keyword", "keyword" => k});
        if v.is_empty() { None } else { Some(v) }
    }),
    (
        r"^(instant and sorcery|instant|sorcery|creature|noncreature|artifact|enchantment|legendary|)? ?spells you cast cost \{(\d+)\} less to cast$",
        |m| {
            Some(eff(obj! {"type" => "static", "static" => "cost", "who" => "you",
            "spell" => m.g(1).filter(|s| !s.is_empty()).unwrap_or("any").replace(' ', "_").replace("instant_and_sorcery", "instant_or_sorcery"),
            "amount" => -int(m.gs(2))}))
        },
    ),
    (r"^(noncreature )?spells your opponents cast cost \{(\d+)\} more to cast$", |m| {
        Some(eff(obj! {"type" => "static", "static" => "cost", "who" => "opponents",
        "spell" => if m.g(1).is_some() { "noncreature" } else { "any" }, "amount" => int(m.gs(2))}))
    }),
    (r"^your opponents can't gain life$", |_| {
        Some(eff(obj! {"type" => "static", "static" => "no_lifegain", "who" => "opponents"}))
    }),
    (r"^you have no maximum hand size$", |_| Some(eff(obj! {"type" => "static", "static" => "no_max_hand"}))),
    (r"^~ can't be blocked$", |_| Some(eff(obj! {"type" => "static", "static" => "unblockable"}))),
    (r"^~ can't block$", |_| Some(eff(obj! {"type" => "static", "static" => "cant_block"}))),
    (
        r"^if an effect would create one or more tokens under your control, it creates twice that many of those tokens instead$",
        |_| Some(eff(obj! {"type" => "replacement", "replace" => "tokens", "multiplier" => 2})),
    ),
    (
        r"^if one or more \+1/\+1 counters would be put on (?:a creature|a permanent|an artifact or creature) you control, twice that many [^.]*instead$",
        |_| Some(eff(obj! {"type" => "replacement", "replace" => "counters", "multiplier" => 2})),
    ),
    (r"^(?:~|this creature) gets \+(\d+)/\+(\d+) for each (.+)$", |m| {
        Some(eff(
            obj! {"type" => "static", "static" => "self_scaling", "pow" => int(m.gs(1)), "tgh" => int(m.gs(2)), "per" => num_expr(m.gs(3))},
        ))
    }),
    (
        r"^~'s power is equal to the number of (.+)$|^~'s power and toughness are each equal to the number of (.+)$",
        |m| {
            let g = m.g(1).or(m.g(2)).unwrap_or("");
            // "power and toughness are each equal" sets both; "power is equal" (Haughty Djinn) leaves the toughness alone
            let mut e = obj! {"type" => "static", "static" => "self_scaling", "pow" => 1, "per" => num_expr(g)};
            if m.g(2).is_some() {
                e.set("base0", true);
            }
            Some(eff(e))
        },
    ),
    // --- Tier 3 (aurelia)
    // "attacking creatures you control have double strike" (Blade Historian): only while they attack
    (r"^attacking creatures you control have ([a-z, ]+)$", |m| {
        let v = kw_statics(m.g(1), |k| {
            obj! {"type" => "static", "static" => "keyword", "keyword" => k,
            "filter" => obj!{"type" => "creature", "controller" => "you", "attacking" => true}}
        });
        if v.is_empty() { None } else { Some(v) }
    }),
    // "other permanents you control have indestructible" (Avacyn, Angel of Hope): noncreature permanents too
    (r"^(other )?permanents you control have ([a-z, ]+)$", |m| {
        let v = kw_statics(m.g(2), |k| {
            obj! {"type" => "static", "static" => "keyword", "keyword" => k,
            "filter" => obj!{"type" => "permanent", "controller" => "you", "other" => m.g(1).is_some()}}
        });
        if v.is_empty() { None } else { Some(v) }
    }),
    // --- Tiers 4/5 (heliod-zur)
    // "white creature spells you cast cost {1} less to cast" (Oketra's Monument)
    (r"^(white|blue|black|red|green) creature spells you cast cost \{(\d+)\} less to cast$", |m| {
        Some(eff(obj! {"type" => "static", "static" => "cost", "who" => "you", "spell" => format!("{}_creature", m.gs(1)),
        "amount" => -int(m.gs(2))}))
    }),
    // "if you would gain life, you gain that much life plus 1 instead" (Cleric Class, Angel of Vitality's wording)
    (r"^if you would gain life, you gain that much life plus (\d+) instead$", |m| {
        Some(eff(obj! {"type" => "static", "static" => "lifegain_plus", "amount" => int(m.gs(1))}))
    }),
    // "if you would gain life, you gain twice that much life instead" (The Wind Crystal, Boon Reflection)
    (r"^if you would gain life, you gain twice that much life instead$", |_| {
        Some(eff(obj! {"type" => "replacement", "replace" => "lifegain", "multiplier" => 2}))
    }),
    // "creatures your opponents control enter tapped" (Authority of the Consuls)
    (r"^creatures your opponents control enter tapped$", |_| {
        Some(eff(obj! {"type" => "static", "static" => "enters_tapped", "who" => "opponents",
        "filter" => obj!{"type" => "creature", "controller" => "opp"}}))
    }),
    // --- Tiers 4/5 (urdragon)
    // "Dragon spells you cast cost {1} less to cast" (Dragonlord's Servant, Dragonspeaker Shaman, Sarkhan, Soul
    // Aflame): a spell with that creature type (changelings too)
    (
        r"^(dragon|goblin|elf|zombie|human|wizard|knight|vampire|merfolk|sliver|spirit) spells you cast cost \{(\d+)\} less to cast$",
        |m| {
            Some(eff(obj! {"type" => "static", "static" => "cost", "who" => "you",
            "spell" => format!("sub_{}", m.gs(1)), "amount" => -int(m.gs(2))}))
        },
    ),
    // "Dragons you control have indestructible" (Call the Spirit Dragons)
    (
        r"^(other )?(elves|goblins|zombies|humans|ninjas|soldiers|warriors|knights|angels|demons|dragons|merfolk|vampires|spirits|faeries) you control have ([a-z, ]+)$",
        |m| {
            let sub_ = singular(m.gs(2));
            let v = kw_statics(m.g(3), |k| {
                obj! {"type" => "static", "static" => "keyword", "keyword" => k,
                "filter" => obj!{"type" => "creature", "controller" => "you", "other" => m.g(1).is_some(),
                "subtype" => sub_.clone()}}
            });
            if v.is_empty() { None } else { Some(v) }
        },
    ),
    // --- Tiers 1/2 (lathril): "attacking Elves you control have deathtouch" (Elderfang Venom): only while they attack
    (
        r"^attacking (elves|goblins|zombies|humans|ninjas|soldiers|warriors|knights|dragons|merfolk|vampires|spirits|faeries) you control have ([a-z, ]+)$",
        |m| {
            let sub_ = singular(m.gs(1));
            let v = kw_statics(m.g(2), |k| {
                obj! {"type" => "static", "static" => "keyword", "keyword" => k,
                "filter" => obj!{"type" => "creature", "controller" => "you", "attacking" => true, "subtype" => sub_.clone()}}
            });
            if v.is_empty() { None } else { Some(v) }
        },
    ),
    // --- Tiers 1/2 (lightpaws)
    // "Aura spells you cast cost {1} less to cast" (Transcendent Envoy); "artifact and enchantment spells you cast
    // cost {1} less to cast" (Starnheim Courser): one discount on a spell of either type
    (r"^(aura|artifact and enchantment) spells you cast cost \{(\d+)\} less to cast$", |m| {
        let spell = if m.gs(1) == "aura" { "aura" } else { "artifact_or_enchantment" };
        Some(eff(obj! {"type" => "static", "static" => "cost", "who" => "you", "spell" => spell,
        "amount" => -int(m.gs(2))}))
    }),
];

pub fn prot_colors(text: &str) -> String {
    let after = text.split_once("protection from").map(|x| x.1).unwrap_or("");
    [("white", 'W'), ("blue", 'U'), ("black", 'B'), ("red", 'R'), ("green", 'G')]
        .iter()
        .filter(|(w, _)| after.contains(w))
        .map(|(_, c)| *c)
        .collect()
}

pub fn singular(w: &str) -> String {
    match w {
        "elves" => "elf".into(),
        "dwarves" => "dwarf".into(),
        "merfolk" => "merfolk".into(),
        "faeries" => "faerie".into(),
        _ => w.strip_suffix('s').unwrap_or(w).to_string(),
    }
}

pub fn strip_name(text: &str, name: &str) -> String {
    let mut t = sub(r"\([^)]*\)", "", text);
    let front = py::first(name, " // ");
    for nm in [front, py::first(front, ",")] {
        if py::len(nm) > 3 {
            t = t.replace(nm, "~");
        }
    }
    // --- Tiers 4/5 (edgar): a legend's first name in its own trigger ("Whenever Edgar attacks": Edgar Markov)
    let first = py::first(front, " ");
    if py::len(first) > 3 && first != front && !front.contains(',') {
        let pat = format!(r"\b(When|Whenever) {} (attacks|dies|deals)\b", fancy_regex::escape(first));
        t = crate::re::sub_fn(&pat, &t, |m| format!("{} ~ {}", m.gs(1), m.gs(2)));
    }
    sub(r"(?i)\bthis (creature|spell|artifact|enchantment|land|permanent|card|planeswalker|equipment|aura)\b", "~", &t)
}

const KW_LINE: &str = r"^(ward|equip|cycling|infect|kicker|flashback|convoke|overload|foretell|spectacle|prowess|protection|landcycling|basic landcycling|devour|undying|persist|flash|rebound|dredge|partner|crew|cumulative upkeep|affinity|delve|improvise|storm|cascade|buyback|madness|evoke|escape|disturb|channel|companion|hideaway|toxic|mentor|exalted|intimidate|defender|changeling|split second|enchant)\b";

fn arr(v: Fx) -> J {
    J::Arr(v.into_iter().map(J::Obj).collect())
}

fn modes_of(lines: &[String], i: &mut usize, mut bad: Option<&mut Vec<String>>) -> Vec<J> {
    let mut modes = vec![];
    while *i < lines.len() && lines[*i].trim_start().starts_with('•') {
        let (e, b) = parse_effects(py::rstrip(&py::lstrip(&lines[*i], "• ").to_lowercase(), "."));
        if !e.is_empty() {
            modes.push(arr(e));
        }
        if let Some(bad) = bad.as_deref_mut() {
            bad.extend(b.iter().map(|x| format!("• {x}")));
        }
        *i += 1;
    }
    modes
}

/// Oracle text -> (abilities, unparsed lines). `types`: engine letters (C, I, S, E, A, P, L).
pub fn compile_card(name: &str, oracle: &str, types: &str, _loyalty: Option<&str>) -> (Fx, Vec<String>) {
    let spell = types.contains('I') || types.contains('S');
    let text = strip_name(oracle, name);
    let mut abil: Fx = vec![];
    let mut bad: Vec<String> = vec![];
    let lines: Vec<String> = text.split('\n').map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect();
    let mut i = 0;
    static ST: LazyLock<Vec<(&'static Regex, StaticFn)>> =
        LazyLock::new(|| STATICS.iter().map(|(p, f)| (anchored(p), *f)).collect());
    while i < lines.len() {
        let raw = lines[i].clone();
        let l0 = raw.to_lowercase();
        let l0 = py::rstrip(&l0, ".");
        i += 1;
        let l_owned = sub(r"^[a-z ]+ — (?=(when|whenever|at|choose|\{))", "", l0); // ability words
        let l: &str = &l_owned;
        // keyword lines and keyword abilities
        if let Some(m) = rmatch(r"^equip \{(\d+)\}$", l) {
            abil.push(obj! {"type" => "static", "static" => "equip_cost", "mana" => int(m.gs(1))});
            continue;
        }
        // --- Tiers 1/2 (tatyova-teysa): "afterlife N" (Ministrant of Obligation, Orzhov Enforcer): when it dies, N 1/1
        // white and black Spirit tokens with flying
        if let Some(m) = rmatch(r"^afterlife (\d+)(?: \(.*\))?$", l) {
            abil.push(obj! {"type" => "triggered", "event" => "dies", "source" => "self", "effects" => vec![jo!{
                "do" => "token", "n" => int(m.gs(1)), "pow" => 1, "tgh" => 1, "keywords" => vec![J::from("flying")],
                "attacking" => false, "warrior" => false, "types" => vec![J::from("spirit")], "color" => "WB"}]});
            continue;
        }
        // --- Tiers 1/2 (lathril): "vigilance; menace" (Abomination of Llanowar) is a keyword line too
        let words: Vec<&str> = l.split([',', ';']).map(|w| w.trim()).collect();
        if words.iter().all(|w| KW.contains(w) || rmatch(KW_LINE, w).is_some()) {
            // toxic N: the amount is kept (combat gives that many poison counters); the keyword alone says no number
            for w in &words {
                if let Some(m) = rmatch(r"^toxic (\d+)", w) {
                    abil.push(obj! {"type" => "static", "static" => "toxic", "amount" => int(m.gs(1))});
                }
            }
            continue;
        }
        if rmatch(r"^\{t\}(?:, [^:]*)?: add ", l).is_some() || rmatch(r"^~ deals 1 damage to you$", l).is_some() {
            continue; // mana abilities -> mana system
        }
        if types.contains('L')
            && rmatch(r"^(\{[^}]+\}, )?\{t\}, sacrifice ~: search your library for a basic land", l).is_some()
        {
            continue; // fetch lands -> land tags
        }
        if rmatch(r"^~ can't be countered$", l).is_some() {
            abil.push(obj! {"type" => "static", "static" => "uncounterable"});
            continue;
        }
        if rmatch(r"^you may pay 1 life and exile a blue card from your hand rather than pay", l).is_some() {
            abil.push(obj! {"type" => "static", "static" => "free_counter"});
            continue;
        }
        if rmatch(r"^you may cast sorcery spells as though they had flash$", l).is_some() {
            abil.push(obj! {"type" => "static", "static" => "sorcery_flash"});
            continue;
        }
        if rmatch(r"^(~ enters prepared|as ~ enters, choose)", l).is_some() {
            abil.push(obj! {"type" => "static", "static" => "note", "text" => l});
            continue;
        }
        if l.starts_with("as an additional cost to cast") {
            abil.push(obj! {"type" => "additional_cost", "text" => l});
            continue;
        }
        if rmatch(r"^~ costs \{\d+\} less to cast for each", l).is_some()
            || rmatch(r"^~ enters (tapped|with)", l).is_some()
        {
            abil.push(obj! {"type" => "static", "static" => "note", "text" => l});
            continue;
        }
        // modal spells / abilities
        if let Some(m) = rmatch(
            r"^choose (one|two|three|one or both|one or more|any number)(?: that hasn't been chosen)?(?:\. you may choose the same mode more than once)?\.? ?—?$",
            l,
        ) {
            let modes = modes_of(&lines, &mut i, Some(&mut bad));
            let k = match m.gs(1) {
                "one" => 1,
                "two" => 2,
                "three" => 3,
                _ => 2,
            };
            if !modes.is_empty() {
                abil.push(obj! {"type" => if spell { "spell" } else { "triggered_modal" },
                "effects" => vec![jo!{"do" => "modal", "choose" => k, "modes" => modes}]});
            }
            continue;
        }
        // loyalty abilities
        if types.contains('P') {
            if let Some(m) = rmatch(r"^([+−-]?\d+|0): (.+)$", l) {
                let (e, b) = parse_effects(m.gs(2));
                if !e.is_empty() {
                    abil.push(
                        obj! {"type" => "loyalty", "loyalty" => int(&m.gs(1).replace('−', "-")), "effects" => arr(e)},
                    );
                }
                bad.extend(b);
                continue;
            }
        }
        // triggered abilities
        let tm = rmatch(r"^(when|whenever|at) (.+?), (.+)$", l);
        if let Some(m) = &tm {
            if rmatch(r"^choose (one|two) —$", m.gs(3)).is_some() {
                let trig = parse_trigger(m.gs(2));
                let modes = modes_of(&lines, &mut i, None);
                match trig {
                    Some(trig) if !modes.is_empty() => {
                        for t in trig {
                            let mut a = obj! {"type" => "triggered", "effects" => vec![jo!{"do" => "modal", "choose" => 1, "modes" => modes.clone()}]};
                            a.update(&t);
                            abil.push(a);
                        }
                    }
                    _ => bad.push(raw),
                }
                continue;
            }
        }
        if let Some(m) = &tm {
            set_ctx(l);
            let trig = parse_trigger(m.gs(2));
            // --- Tiers 1/2 (isshin): "whenever a creature you control attacks, it gets +2/+2" (Fervent Charge): "it"
            // is the attacking creature, not this permanent
            let each = trig.as_ref().is_some_and(|t| t.iter().any(|x| x.get("source") == Some(&J::from("you_each"))));
            let body = match m.gs(3).strip_prefix("it gets ") {
                Some(r) if each => format!("that creature gets {r}"),
                _ => m.gs(3).to_string(),
            };
            let (e, b) = parse_effects(&body);
            match trig {
                Some(trig) if !e.is_empty() => {
                    for t in trig {
                        let mut a = obj! {"type" => "triggered", "effects" => arr(e.clone())};
                        a.update(&t);
                        abil.push(a);
                    }
                    if !b.is_empty() {
                        bad.push(format!("{raw}   [partly: {}]", b.join("; ")));
                    }
                }
                _ => bad.push(raw),
            }
            continue;
        }
        // activated abilities
        if let Some(m) = rmatch(r#"^([^:"]+): (.+)$"#, l) {
            let g1 = m.gs(1);
            if g1.contains('{') || g1.contains("sacrifice") || g1.contains("pay") || g1.contains("discard") {
                let cost = parse_cost(g1);
                let body = sub(r"\.? ?activate (only|this ability only)[^.]*", "", m.gs(2));
                let (mut e, mut b) = parse_effects(&body);
                // --- Tiers 1/2 (windgrace): "~ gains ... Tap it." (Soul of Windgrace): "it" is ~
                if body.starts_with("~ ") && b.iter().any(|x| x == "tap it") {
                    b.retain(|x| x != "tap it");
                    e.push(eff(obj! {"do" => "tap", "what" => obj!{"sel" => "self"}}).remove(0));
                }
                if let Some(c) = &cost {
                    if !e.is_empty()
                        && e.iter().all(|x| x.get("do") == Some(&J::Str("add_mana".into())))
                        && c.keys().all(|k| k == "tap" || k == "mana")
                    {
                        continue; // mana abilities are handled by the mana system
                    }
                }
                match cost {
                    Some(c) if !e.is_empty() => {
                        abil.push(obj! {"type" => "activated", "cost" => c, "effects" => arr(e),
                        "sorcery" => m.gs(2).contains("as a sorcery"), "once" => m.gs(2).contains("once each turn")});
                        if !b.is_empty() {
                            bad.push(format!("{raw}   [partly: {}]", b.join("; ")));
                        }
                    }
                    _ => bad.push(raw),
                }
                continue;
            }
        }
        // --- Tier 3 (korvold): an Equipment's bonus and keywords with a granted combat-damage trigger in quotes:
        // 'equipped creature gets +1/+1 and has trample and "whenever this creature deals combat damage to a player
        // or planeswalker, create that many treasure tokens."' (The Reaver Cleaver). The bonus compiles as the plain
        // line; the quoted ability is the equipped creature's combat-damage trigger ("that many": the damage dealt).
        if let Some(m) = rmatch(
            r#"^(equipped creature (?:gets|has) [^"]+?) and "whenever (?:~|this creature) deals combat damage to a player(?: or planeswalker)?, ([^"]+?)\.?"$"#,
            l,
        ) {
            let mut stat: Option<Fx> = None;
            for (rx, f) in ST.iter() {
                if let Some(mm) = matches_rule(rx, m.gs(1)) {
                    stat = f(&mm).filter(|v| !v.is_empty());
                    if stat.is_some() {
                        break;
                    }
                }
            }
            let (e, b) = parse_effects(m.gs(2));
            if let Some(st) = stat
                && !e.is_empty()
                && b.is_empty()
            {
                abil.extend(st);
                abil.push(obj! {"type" => "triggered", "effects" => arr(e), "event" => "combat_damage",
                "source" => "equipped"});
                continue;
            }
        }
        // statics
        let mut got: Option<Fx> = None;
        for (rx, f) in ST.iter() {
            if let Some(mm) = matches_rule(rx, l) {
                let r = f(&mm);
                if r.as_ref().is_some_and(|v| !v.is_empty()) {
                    got = r;
                    break;
                }
            }
        }
        if let Some(g) = got {
            abil.extend(g);
            continue;
        }
        // spell text
        if spell {
            set_ctx(l);
            let (e, b) = parse_effects(l);
            if !e.is_empty() {
                abil.push(obj! {"type" => "spell", "effects" => arr(e.clone())});
            }
            if !b.is_empty() {
                bad.push(if e.is_empty() { raw } else { format!("{raw}   [partly: {}]", b.join("; ")) });
            }
            continue;
        }
        if is(r"\benters tapped\b|\{t\}: add|pay 2 life", l) {
            continue;
        }
        if types.contains('L') {
            continue; // other land text: utility abilities not modeled
        }
        bad.push(raw);
    }
    // merge several spell lines into one spell ability
    let sp: Vec<&Obj> = abil.iter().filter(|a| a.get("type") == Some(&J::Str("spell".into()))).collect();
    if sp.len() > 1 {
        let mut effects: Vec<J> = vec![];
        for a in &sp {
            if let Some(J::Arr(es)) = a.get("effects") {
                effects.extend(es.iter().cloned());
            }
        }
        let merged = obj! {"type" => "spell", "effects" => effects};
        abil.retain(|a| a.get("type") != Some(&J::Str("spell".into())));
        abil.push(merged);
    }
    (abil, bad)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_pattern_compiles() {
        assert!(!rules().is_empty());
        let _ = parse_trigger("x");
        let _ = compile_card("X", "~ enters the battlefield", "C", None);
    }

    #[test]
    fn lightning_bolt() {
        let (a, bad) = compile_card("Lightning Bolt", "Lightning Bolt deals 3 damage to any target.", "I", None);
        assert!(bad.is_empty());
        assert_eq!(
            crate::json::dumps(&arr(a), None, false),
            r#"[{"type": "spell", "effects": [{"do": "damage", "n": 3, "to": {"sel": "any_target"}}]}]"#
        );
    }
}
