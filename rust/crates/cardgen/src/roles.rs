//! Deterministic deck-building roles, deliberately separate from executable engine tags.
//! Rules read normalized Oracle lines and printed stats, never card names or deck membership.
use crate::data::Project;
use crate::knowledge::{self, Card, Index, Legal};
use fancy_regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::path::{Path, PathBuf};

pub const IDS: [&str; 26] = [
    "ramp",
    "fixing",
    "draw",
    "selection",
    "tutor",
    "removal",
    "wipe",
    "counter",
    "protection",
    "stax",
    "discard",
    "gy-hate",
    "recursion",
    "reanimate",
    "self-mill",
    "tokens",
    "anthem",
    "sac-outlet",
    "drain",
    "lifegain",
    "counters",
    "blink",
    "copy",
    "extra-phase",
    "finisher",
    "evasion",
];

pub fn check_role(id: &str) -> Result<(), String> {
    if IDS.contains(&id) { Ok(()) } else { Err(format!("unknown role '{id}'; valid roles: {}", IDS.join(", "))) }
}

#[derive(Debug, Deserialize)]
pub struct Taxonomy {
    pub version: u32,
    pub role: Vec<Role>,
}
#[derive(Debug, Deserialize)]
pub struct Role {
    pub id: String,
    pub definition: String,
    pub src: String,
    pub guard: String,
}
impl Taxonomy {
    pub fn parse(s: &str) -> Result<Self, String> {
        let t: Self = toml::from_str(s).map_err(|e| format!("roles.toml: {e}"))?;
        if t.version != 1 || t.role.len() != IDS.len() {
            return Err("roles.toml: expected version 1 and exactly 26 roles".into());
        }
        let mut seen = BTreeSet::new();
        for r in &t.role {
            check_role(&r.id)?;
            if !seen.insert(&r.id) || r.definition.is_empty() || r.src != role_src(&r.id) {
                return Err(format!("roles.toml: invalid definition, source or repeated role {}", r.id));
            }
            Regex::new(&r.guard).map_err(|e| format!("guard {}: {e}", r.id))?;
        }
        Ok(t)
    }
}
pub fn role_src(id: &str) -> &'static str {
    match id {
        "stax" | "finisher" => "llm",
        "removal" | "protection" | "drain" | "copy" | "evasion" => "rules+llm",
        _ => "rules",
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub source: String,
    pub certain: bool,
    pub rule: String,
    /// Zero-based index into Card::lines(); face is redundant, checked on attach. Overrides may lack evidence.
    pub face: Option<usize>,
    pub line: Option<usize>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Assignment {
    pub roles: Vec<String>,
    pub primary: String,
    pub evidence: BTreeMap<String, Evidence>,
    pub needs_llm: bool,
}
pub type RoleMap = BTreeMap<String, Assignment>;

/// The list is ordered: ties in prominence use this order, then the first Oracle line.
/// Semantic conditions below implement the gold conventions; a regex hit alone is not an assignment.
pub struct Rule {
    pub role: &'static str,
    pub pattern: &'static str,
    pub negative: &'static str,
    pub id: &'static str,
}
pub const RULES: &[Rule] = &[
    Rule {
        role: "finisher",
        pattern: concat!(
            r"you win the game|loses the game|creatures you control[^.]*?get \+[3-9x]/\+[0-9x]|",
            r"double all damage that sources you control|\binfect\b",
        ),
        negative: r"can't win|can't lose",
        id: "alternate-win",
    },
    Rule {
        role: "ramp",
        pattern: concat!(
            r"\badd\b.*(?:mana|\{)|costs? (?:\{[^}]+\}|[0-9]+).*less|pay .*life rather than pay|",
            r"additional lands? on|untap (?:(?:two|three|all) (?:target )?lands|target land)|",
            r"return[^.]*land cards? from your graveyard to the battlefield|",
            r"create[^.]*\b(?:forest|plains|island|swamp|mountain)\b[^.]*land creature token|",
            r"(?:spell|spells)[^.]*(?:has|have) affinity",
        ),
        negative: r"^this spell costs|^~ costs|^this ability costs",
        id: "mana",
    },
    Rule {
        role: "ramp",
        pattern: r"search your library.*(?:land|plains|island|swamp|mountain|forest).*battlefield",
        negative: r"",
        id: "land-ramp",
    },
    Rule {
        role: "fixing",
        pattern: r"\badd\b|search your library|basic landcycling|mana of any|enchanted land is",
        negative: r"",
        id: "fixing",
    },
    Rule { role: "tutor", pattern: r"search your library|\btransmute\b", negative: r"", id: "library-search" },
    Rule {
        role: "counter",
        pattern: concat!(
            r"\bcounter (?:up to (?:two|three) target|target|that|each|all) (?:[^.]*?)(?:spells?|ability|abilities)\b|",
            r"exile (?:any number of )?target spells?",
        ),
        negative: r"",
        id: "counter-spell",
    },
    Rule {
        role: "wipe",
        pattern: concat!(
            r"(?:destroy|exile|return|sacrifice) (?:all|each)|\b(?:all|each)\b[^.]*gets? -[1-9x][0-9]*/-[1-9x]|",
            r"\bcreatures\b[^.]*get -[1-9x][0-9]*/-[1-9x]|deals? [^.]*damage [^.]*\b(?:each|all) [^.]*creature|",
            r"overload|sacrifices all (?:other )?creatures|destroy the rest|",
            r"each creature[^.]*loses all abilities[^.]*1/1",
        ),
        negative: r"",
        id: "mass-removal",
    },
    Rule {
        role: "removal",
        pattern: concat!(
            r"(?:destroy|exile|return) (?:another |up to [a-z]+ |two |three |four )?target|",
            r"return (?:~|this creature) and another target|",
            r"(?:destroy|exile) (?:it|that (?:creature|permanent))|put target[^.]*owner.s library|",
            r"(?:each|target) opponents? (?:sacrifices|exiles) (?:a|an)|you control enchanted (?:creature|permanent)|",
            r"put (?:a|one) -1/-1 counter on (?:up to one )?target|",
            r"target.*(?:gets? -[0-9x]+/-[1-9x][0-9]*|sacrifices? (?:a|an|one)|fights)|",
            r"deals? [^.]*damage (?:to (?:any target|target (?!player)[^.]*?(?:creature|artifact|planeswalker)|",
            r"each of up to [a-z]+ target creatures)|divided)|",
            r"gain control of|exchange control of target|owner of target .*shuffles|",
            r"enchanted .*?(?:can't attack|doesn't untap|loses all|sacrifice this creature)|",
            r"target permanent.*doesn't untap|chosen creatures fight|\bannihilator [1-9]|",
            r"at the beginning.*?\btap (?:it|enchanted creature)|it gets -[0-9x]+/-[1-9x]",
        ),
        negative: r"until end of turn.*gain control|gain control.*until end of turn",
        id: "single-removal",
    },
    Rule {
        role: "blink",
        pattern: concat!(
            r"exile .*?(?:you control|~|this creature|this permanent).*?return|",
            r"exile .*?return (?:it|that card|those cards|them|all other permanent cards).*?battlefield|",
            r"exile[^.]*creature you own.*?put both cards onto the battlefield",
        ),
        negative: r"graveyard",
        id: "exile-return",
    },
    Rule {
        role: "protection",
        pattern: concat!(
            r"(?:gain|gains|have|has).*?(?:hexproof|shroud|indestructible|protection from|ward)|",
            r"put an indestructible counter on each|",
            r"phases? out|prevent (?:all|the next [0-9x]+) (?:combat )?damage|you have protection|",
            r"regenerate (?:target|another|each)|umbra armor",
        ),
        negative: r"",
        id: "protect",
    },
    Rule {
        role: "draw",
        pattern: concat!(
            r"\bdraws?\b|(?:play|cast|plot) .*?(?:top of your library|exiled)|(?:put|",
            r"return).*?(?:into|to) your hand|graveyard.*?(?:into your hand|has escape)|\bcascade\b|",
            r"may (?:play|cast) (?:it|that card|those cards|one of those cards)|",
            r"\binvestigate\b|create[^.]*\b(?:clue|junk) tokens?",
        ),
        negative: r"",
        id: "card-advantage",
    },
    Rule {
        role: "selection",
        pattern: concat!(
            r"\b(?:scry|surveil) (?:[2-9]|[1-9][0-9]+|x)\b|draw .*?discard|discard .*?draw|",
            r"look at the top|from among them into your hand|reveal the top|",
            r"discards any number of cards, then draws|\b(?:connives?|recruit|learn)\b|create[^.]*blood token",
        ),
        negative: r"",
        id: "filter",
    },
    Rule {
        role: "discard",
        pattern: concat!(
            r"(?:opponents?|target player|defending player|that player|controller|each player|players|they) ",
            r"discards?\b|",
            r"controller reveals their hand, then discards",
        ),
        negative: r"",
        id: "opponent-discard",
    },
    Rule {
        role: "gy-hate",
        pattern: concat!(
            r"\bexiles? [^.]*?\b(?:from|in|all|each) ",
            r"(?:a |an |your |their |target |opponent.s )?graveyards?|",
            r"\bexile target player.s graveyard|graveyard[^.]*?(?:can't|instead exile)|can't [^.]*?graveyard",
        ),
        negative: r"",
        id: "graveyard-hate",
    },
    Rule {
        role: "recursion",
        pattern: concat!(
            r"(?:return|put)[^.]*?(?:from|in) (?:your|their|a) graveyard[^.]*?(?:hand|library)|",
            r"shuffle your graveyard into your library|",
            r"\b(?:cast|play)\b[^.]*?(?:from|in) your graveyard|",
            r"your graveyard[^.]*?(?:\bcast\b|\bplay\b|gains flashback|has escape|into your hand)|\bsoulshift\b|",
            r"in your graveyard.*?otherwise, put it into your hand|",
            r"(?:this card|~) is in your graveyard.*?return (?:this card|~) to your hand|",
            r"choose target [^.]*card in your graveyard.*?return it to your hand|",
            r"(?:return|put)[^.]*graveyard[^.]*battlefield|graveyard[^.]*shuffle [^.]*into your library|",
            r"graveyard[^.]*put it into your library|permanent card in your graveyard[^.]*battlefield|",
            r"return all [^.]*cards in graveyards[^.]*hands",
        ),
        negative: r"",
        id: "graveyard-reuse",
    },
    Rule {
        role: "reanimate",
        pattern: concat!(
            r"(?:return|put).*?(?:graveyard|creature card).*?battlefield|",
            r"return from your graveyard to the battlefield|",
            r"graveyard.*?creature.*?put.*?battlefield|",
            r"creature[^.]*dies, return it to the battlefield|",
            r"puts all cards they exiled this way onto the battlefield|",
            r"opponent sacrifices a nontoken permanent[^.]*put that card onto the battlefield",
        ),
        negative: r"",
        id: "graveyard-battlefield",
    },
    Rule {
        role: "self-mill",
        pattern: r"\bmills?\b|(?:put|putting).*?(?:into|in) your graveyard|^discard a card:",
        negative: r"",
        id: "fill-graveyard",
    },
    Rule {
        role: "copy",
        pattern: concat!(
            r"\bcopy (?:target|that|it|the|up to)|(?:as|becomes?|that's|that is|are) (?:a )?cop(?:y|ies) of|",
            r"\bcopies (?:it|that spell)\b|",
            r"^\b(?:casualty|conspire|replicate|storm)\b"
        ),
        negative: r"cast a copy of its spell",
        id: "copy",
    },
    Rule {
        role: "tokens",
        pattern: r"\bcreates?\b.*?tokens?|\b(?:investigate|amass|populate|incubate|mobilize|afterlife|recruit)\b",
        negative: r"",
        id: "create-token",
    },
    Rule {
        role: "anthem",
        pattern: concat!(
            r"(?:creatures|creature tokens|[a-z]+s) you control.*?(?:get|have|gain)|",
            r"\bother .*?(?:get|have)|creatures? .*?you control.*?(?:gets?|have|gain)|",
            r"(?:all [a-z]+s|creatures with [a-z]+) (?:get|have)|each other [^.]*you control.*gets|",
            r"put [^.]*\+1/\+1 counters? on each creature (?:you control|target player controls)",
        ),
        negative: r"",
        id: "team-pump",
    },
    Rule {
        role: "sac-outlet",
        pattern: r"sacrifice (?:a|an|another|two|three|any number|x|[a-z]+ or another) .*?:",
        negative: r"sacrifice (?:a|an|x) (?:treasures?|clues?|foods?|blood|maps?|powerstones?)\b",
        id: "sacrifice-cost",
    },
    Rule {
        role: "drain",
        pattern: concat!(
            r"(?:opponents?|that player|defending player|each player|target player|they)[^.]*?loses? ",
            r"(?!the game)[^.]*?life|deals? [^.]*?damage to (?:each opponent|that opponent|that player|",
            r"target player|each player|you|the chosen player)|deals damage equal [^.]*to the chosen player",
        ),
        negative: r"",
        id: "life-loss",
    },
    Rule {
        role: "lifegain",
        pattern: r"\b(?:gain|gains|gained) .*?life|\blifelink\b|create[^.]*food token|double[^.]*life total",
        negative: r"can't gain|can't.*?lose life",
        id: "gain-life",
    },
    Rule {
        role: "counters",
        pattern: concat!(
            r"\b(?:put|double|move|distribute)\b.*?counters?|\bproliferate\b|^backup [1-9]|^bolster [1-9]|",
            r"^support [1-9]|^adapt [1-9]|^awaken [1-9]|\b(?:connives?|dethrone|mentor|evolve|training)\b|",
            r"\bamass\b|create[^.]*map token",
        ),
        negative: r"",
        id: "counters",
    },
    Rule {
        role: "extra-phase",
        pattern: r"(?:extra|additional) (?:turn|combat|beginning phase)|additional combat phase",
        negative: r"(?:can't|during|skip).*?extra|end the turn",
        id: "extra-phase",
    },
    Rule {
        role: "evasion",
        pattern: concat!(
            r"(?:gains?|has|have).*?(?:flying|menace|fear|shadow|horsemanship|double strike|intimidate)|(?:creature|",
            r"creatures).*?can't be blocked|all [a-z]+s have (?:fear|flying|menace|shadow)",
        ),
        negative: r"",
        id: "grant-evasion",
    },
    Rule {
        role: "stax",
        pattern: concat!(
            r"(?:opponents?|players|creatures|spells)[^.]*?(?:can't|cost)[^.]*?(?:cast|draw|attack|pay|",
            r"life)|opponents[^.]*?cost .*more|unless (?:that player|they) pay|",
            r"creatures your opponents control enter tapped|enchanted creature[^.]*can't attack you|",
            r"if an opponent[^.]*would put a land[^.]*then sacrifices a land|controller can't cast spells|",
            r"players can't search libraries|(?:nonbasic lands|lands|creatures) ",
            r"(?:enter tapped|don't untap|are mountains)|",
            r"shuffle their library, that player puts a card from their hand|",
            r"each noncreature artifact loses all abilities",
        ),
        negative: r"",
        id: "restriction",
    },
];

/// Explicit boilerplate: these explain no deck-building role. Unknown effects must stay visible to routing.
const IGNORE: &[&str] = &[
    r"^$",
    concat!(
        r"^(?:flying|first strike|double strike|deathtouch|defender|flash|haste|hexproof|",
        r"indestructible|lifelink|menace|reach|shroud|trample|vigilance|ward|protection from|",
        r"fear|shadow|infect|wither|toxic|swampwalk|islandwalk|mountainwalk|forestwalk|",
        r"plainswalk|horsemanship)\b",
    ),
    concat!(
        r"^(?:equip|enchant|fortify|crew|saddle|convoke|delve|affinity|kicker|multikicker|",
        r"flashback|aftermath|rebound|suspend|madness|foretell|plot|morph|megamorph|disguise|",
        r"prototype|evoke|escape|buyback|echo|entwine|replicate|storm|strive|cycling|",
        r"basic landcycling|mountaincycling|plainscycling|islandcycling|swampcycling|",
        r"forestcycling|soulbond|bestow|disturb|mutate|partner|choose a background|",
        r"friends forever|companion|prepared|renown|cumulative upkeep|fading|vanishing|",
        r"daybound|nightbound|sunburst|modular|graft)\b",
    ),
    concat!(
        r"^(?:choose (?:one|two|three|a|any)|as an additional cost|",
        r"as long as .*? is on the stack|you may have ~ enter|~ enters|this (?:land|artifact|",
        r"creature|enchantment) enters|as (?:~|this .*?) enters|spend (?:this|only) mana|",
        r"activate only|a deck can have|this spell can't be countered|~ can't be countered)\b",
    ),
    concat!(
        r"^(?:~|this creature|this permanent|enchanted creature|equipped creature) (?:gets?|",
        r"has|is|can|can't|doesn't|must|attacks?|blocks?|enters|costs)\b",
    ),
    r"(?:power and toughness|base power and toughness|power is equal|toughness is equal|legend rule|maximum hand size)",
    r"^\{[^.]*?: add\b",
    r"^the next spell .*can't be countered",
    r"^scry 1\b",
    r"^surveil 1\b",
    r"^(?:threshold|hellbent|delirium|formidable|ferocious) — .*?(?:gets?|has|can|can't)\b",
    r"^(?:at the beginning .*?(?:sacrifice ~|sacrifice this)|when .*? dies, (?:return|exile) (?:it|~))",
];

fn regex(p: &str) -> Result<Regex, String> {
    Regex::new(&format!("(?i){p}")).map_err(|e| format!("role regex {p}: {e}"))
}
fn hit(r: &Regex, s: &str) -> Result<bool, String> {
    r.is_match(s).map_err(|e| format!("role regex {}: {e}", r.as_str()))
}
fn has(s: &str, pieces: &[&str]) -> bool {
    pieces.iter().any(|p| s.contains(p))
}

/// Supplemental sheets/Attractions stay in Oracle, but are not deck-card role assignments.
pub fn is_deck_card(c: &Card) -> bool {
    !c.raw.type_line.split_whitespace().any(|t| matches!(t, "Stickers" | "Attraction"))
}

fn creates(s: &str, kind: &str) -> bool {
    s.split("create").skip(1).any(|object| object.split("token").next().unwrap_or("").contains(kind))
}

/// Inspect the returned object and destination together, not a different return in the same sentence.
fn battlefield_object(s: &str) -> Option<&str> {
    let end = s.find("to the battlefield").or_else(|| s.find("onto the battlefield"))?;
    let before = &s[..end];
    if !before.contains("graveyard") {
        return None;
    }
    let start = before.rfind("return ").or_else(|| before.rfind("put ")).unwrap_or(0);
    // A target chosen earlier supplies the object of a later "return/put it".
    // Some returns name their object after the destination ("Return from ... to the battlefield ... creatures").
    let returned = s[start..].split(". ").next().unwrap_or("");
    let object = if before[start..].contains("card") { &before[start..] } else { returned };
    Some(if has(object, &["card", "land"]) { object } else { before })
}

/// Nested parentheses are stripped, including ward and prepare reminder text. Names are replaced longest first,
/// case-sensitively and at word boundaries (an apostrophe is a boundary, as in a self name's possessive).
pub fn normalize(card: &Card, line: &str) -> String {
    let mut depth = 0usize;
    let mut out = String::new();
    for c in line.chars() {
        match c {
            '(' => depth += 1,
            ')' if depth > 0 => depth -= 1,
            _ if depth == 0 => out.push(c),
            _ => {}
        }
    }
    let mut names = card.self_names();
    names.sort_by_key(|n| std::cmp::Reverse(n.len()));
    for name in names {
        let mut next = String::new();
        let mut end = 0;
        for (i, _) in out.match_indices(&name) {
            let j = i + name.len();
            let word = |c: char| c.is_alphanumeric() || c == '_';
            if out[..i].chars().next_back().is_some_and(word) || out[j..].chars().next().is_some_and(word) {
                continue;
            }
            next.push_str(&out[end..i]);
            next.push('~');
            end = j;
        }
        next.push_str(&out[end..]);
        out = next;
    }
    out.trim().to_string()
}

pub struct Classifier {
    rules: Vec<(Regex, Regex)>,
    ignore: Vec<Regex>,
    certain: Vec<(&'static str, Regex)>,
}
impl Classifier {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            rules: RULES.iter().map(|r| Ok((regex(r.pattern)?, regex(r.negative)?))).collect::<Result<_, String>>()?,
            ignore: IGNORE.iter().map(|p| regex(p)).collect::<Result<_, _>>()?,
            certain: CERTAIN.iter().map(|(role, p)| Ok((*role, regex(p)?))).collect::<Result<_, String>>()?,
        })
    }

    pub fn classify(&self, card: &Card) -> Result<Assignment, String> {
        let lines = card.lines();
        let mut found: BTreeMap<&str, (usize, Evidence)> = BTreeMap::new();
        let mut unknown = false;
        let mut modal_repeat = false;
        let mut modal_face = None;
        for (line, &(face, original)) in lines.iter().enumerate() {
            let f = face.and_then(|i| card.raw.card_faces.as_ref()?.get(i));
            let typ = f.map_or(card.raw.type_line.as_str(), |f| &f.type_line);
            let land = typ.contains("Land");
            // MDFC land backs and prepare bodies are not classified as extra functions.
            if (card.raw.layout == "modal_dfc" && face.is_some_and(|i| i > 0) && land)
                || (card.raw.layout == "prepare" && !has(typ, &["Instant", "Sorcery"]))
            {
                continue;
            }
            let loyalty = f.and_then(|f| f.loyalty.as_deref()).or(card.raw.loyalty.as_deref());
            let text = normalize(card, original);
            let s = text.to_lowercase();
            if !s.starts_with('•') {
                modal_repeat = has(&s, &["at the beginning", "whenever", ":"]) && s.contains("choose one");
                modal_face = face;
            }
            let contextual = if s.starts_with('•') && modal_repeat && face == modal_face {
                format!("whenever a mode is chosen, {s}")
            } else {
                s.clone()
            };
            if let Some(cost) =
                s.strip_prefix(['−', '-']).and_then(|s| s.split_once(':')).and_then(|(c, _)| c.parse::<u32>().ok())
            {
                if loyalty.and_then(|l| l.parse::<u32>().ok()).is_some_and(|l| cost >= l) {
                    continue;
                }
            }
            // Basic land types confer mana abilities even when their only Oracle line is parenthesized reminder text.
            let basic_colors =
                ["Plains", "Island", "Swamp", "Mountain", "Forest"].iter().filter(|t| typ.contains(**t)).count();
            let intrinsic_fixing = land && basic_colors >= 2 && original.starts_with("({T}: Add ");
            let mut recognized = intrinsic_fixing;
            for (n, r) in RULES.iter().enumerate() {
                let (positive, negative) = &self.rules[n];
                let matched = (r.id == "fixing" && intrinsic_fixing) || hit(positive, &s)?;
                if !matched {
                    continue;
                }
                // A pattern remains recognized when a convention rejects its incidental effect.
                recognized = true;
                if (!r.negative.is_empty() && hit(negative, &s)?)
                    || !accept(r, card, &contextual, typ, intrinsic_fixing)
                {
                    continue;
                }
                let priority = prominence(r.role, &s, line, typ);
                let mut certain = false;
                for (role, template) in &self.certain {
                    if *role == r.role {
                        certain |= hit(template, &s)?;
                    }
                }
                let e = Evidence { source: "rule".into(), certain, rule: r.id.into(), face, line: Some(line) };
                match found.get_mut(r.role) {
                    Some((p, old)) => {
                        if e.certain && !old.certain || e.certain == old.certain && priority < *p {
                            *p = priority;
                            *old = e;
                        }
                    }
                    None => {
                        found.insert(r.role, (priority, e));
                    }
                }
            }
            if !recognized {
                let mut ignored = false;
                for p in &self.ignore {
                    ignored |= hit(p, &s)?;
                }
                unknown |= !ignored;
            }
        }
        let mut ranked: Vec<_> = found.into_iter().collect();
        ranked.sort_by_key(|(role, (p, _))| (*p, IDS.iter().position(|r| r == role).unwrap()));
        unknown |= ranked.len() > 4;
        if ranked.len() > 4 && ranked.iter().any(|(r, _)| *r == "copy") {
            let copy_line = ranked.iter().find(|(r, _)| *r == "copy").unwrap().1.1.line;
            ranked.retain(|(r, (_, e))| *r != "tokens" || e.line != copy_line);
        }
        ranked.truncate(4);
        let roles: Vec<String> = ranked.iter().map(|(r, _)| r.to_string()).collect();
        let ambiguous = ranked.iter().any(|(r, (_, e))| {
            let s = normalize(card, lines[e.line.unwrap()].1).to_lowercase();
            matches!(
                *r,
                "selection"
                    | "recursion"
                    | "gy-hate"
                    | "finisher"
                    | "stax"
                    | "drain"
                    | "copy"
                    | "evasion"
                    | "protection"
            ) || *r == "removal" && has(&s, &[",", ". ", " and ", " then ", " if "])
        });
        let needs_llm = unknown || ambiguous || (!roles.is_empty() && roles.iter().all(|r| role_src(r) != "rules"));
        Ok(Assignment {
            primary: roles.first().cloned().unwrap_or_default(),
            roles,
            evidence: ranked.into_iter().map(|(r, (_, e))| (r.into(), e)).collect(),
            needs_llm,
        })
    }
}

/// Certainty requires a complete direct effect template. Conditional, triggered and modal variants remain hints.
const CERTAIN: &[(&str, &str)] = &[
    ("counter", r"^counter target (?:spell|(?:[a-z]+[, ]*(?:or )?)+spell)\.(?: |$)"),
    (
        "tutor",
        concat!(
            r"^search your library for a (?:card|(?:nonland |creature |artifact |enchantment |instant |",
            r"sorcery )+card), (?:reveal it, )?put (?:it|that card) into your hand, then shuffle\.$"
        ),
    ),
    ("ramp", r"^\{t\}: add \{[wubrg]\}\.$"),
    ("draw", r"^(?:you )?draw (?:two|three|four) cards\.$"),
    ("lifegain", r"^lifelink$"),
];

fn prominence(role: &str, s: &str, line: usize, typ: &str) -> usize {
    // Functional costs precede their payoff; printed lifelink is always last. Remaining ties follow Oracle order.
    match role {
        "finisher" => 0,
        "ramp" if s.contains("sacrifice") && s.contains("add") => 10,
        "sac-outlet" => 20,
        "tutor" => 30,
        "protection" if has(typ, &["Instant", "Sorcery"]) => 40,
        "counter" => 50,
        "copy" => 60,
        "lifegain" => usize::MAX,
        // In a paired unblockable/shroud grant, evasion is the leading effect in the Oracle sentence.
        "evasion" if s.contains("can't be blocked and has") => 107 + line * 30,
        _ => 100 + line * 30 + IDS.iter().position(|r| *r == role).unwrap_or(26),
    }
}

fn accept(r: &Rule, c: &Card, s: &str, typ: &str, intrinsic: bool) -> bool {
    let land = typ.contains("Land");
    let spell = has(typ, &["Instant", "Sorcery"]);
    let activation = s.contains(':');
    let repeat = activation || has(s, &["whenever", "at the beginning", "each upkeep", "each end step"]);
    let self_sac = has(
        s.split(':').next().unwrap_or(s),
        &["sacrifice ~", "sacrifice this", "exile ~", "exile this", "discard this", "discard ~"],
    );
    let grant = has(
        s,
        &[
            "target creature",
            "creature you control",
            "equipped creature",
            "enchanted creature",
            "creatures you control",
            "permanents you control",
            "both creatures",
            "other creatures",
            "you have protection",
            "another target",
            "legendary permanent you control",
            "vehicles you control",
            "those creatures",
            "each other ",
            "target permanent",
            "target artifact",
            "target enchantment",
            "target land",
            "target planeswalker",
            "permanent you control",
        ],
    );
    match r.role {
        "finisher" => {
            if s.contains("infect") {
                return c.raw.power.as_deref().and_then(|p| p.parse::<u32>().ok()).is_some_and(|p| p >= 10);
            }
            let trigger = s.split(',').next().unwrap_or(s);
            !(trigger.contains("loses the game") && has(trigger, &["when ", "whenever ", "if "]))
                && !has(s, &["if you win the game", "can't win"])
                && (!s.contains("get +") || s.contains("trample"))
        }
        "ramp" => {
            if r.id == "land-ramp" {
                return !land && !has(s, &["its controller", "their library", "that player"]);
            }
            if has(s, &["untap", "land creature token", "has affinity", "have affinity", "land card"])
                && !s.contains("add")
            {
                return !land;
            }
            if s.contains("less") {
                return !has(
                    s,
                    &["this spell costs", "~ costs", "this ability costs", "equip abilities", "ability costs"],
                );
            }
            if land {
                return s.contains("add")
                    && (s.contains("sacrifice") && !self_sac
                        || s.contains("{c}{c}")
                        || s.contains("two mana")
                        || s.contains("three mana"));
            }
            !has(s, &["its controller", "that player adds", "opponent adds"])
        }
        "fixing" => {
            if intrinsic {
                return true;
            }
            if s.contains("enchanted land is") {
                return ["plains", "island", "swamp", "mountain", "forest"].iter().filter(|t| s.contains(**t)).count()
                    >= 2;
            }
            if s.contains("search your library") {
                let searched = s.split("search your library").nth(1).unwrap().split("card").next().unwrap_or("");
                let types = ["plains", "island", "swamp", "mountain", "forest"]
                    .iter()
                    .filter(|t| searched.contains(**t))
                    .count();
                return searched.contains("land") && !searched.contains("nonland") || types >= 2;
            }
            s.contains("basic landcycling")
                || s.contains("add")
                    && (has(
                        s,
                        &[
                            "any color",
                            "any one color",
                            "any combination of colors",
                            "different colors",
                            "each color",
                            "any type",
                        ],
                    ) || ['w', 'u', 'b', 'r', 'g']
                        .iter()
                        .filter(|ch| s.split("add").nth(1).unwrap_or("").contains(&format!("{{{ch}}}")))
                        .count()
                        >= 2)
        }
        "tutor" => {
            if s.starts_with("transmute") {
                return true;
            }
            let q = s.split("search your library").nth(1).unwrap_or("").split([',', '.']).next().unwrap_or("");
            !has(q, &["land card", "land cards", "plains", "island", "swamp", "mountain", "forest"])
                || q.contains("nonland")
        }
        "counter" => !has(s, &["whenever this creature becomes the target", "whenever ~ becomes the target"]),
        "wipe" => {
            if s.contains("overload") {
                return c.text.to_lowercase().contains("damage")
                    || has(&c.text.to_lowercase(), &["destroy", "exile", "return"]);
            }
            if has(s, &["damage to each opponent", "damage to each player", "return all other permanent cards"])
                || has(
                    s,
                    &[
                        "exile all cards",
                        "exile all graveyards",
                        "exile each graveyard",
                        "return all cards",
                        "exile all spells",
                        "sacrifice all",
                    ],
                )
            {
                return false;
            }
            (!s.contains("damage") || !s.contains("combat damage"))
                && !(s.contains("1 damage") && !spell && !repeat)
                && !s.contains("each of up to")
                && !(s.contains("exile") && s.contains("return") && s.contains("you control"))
        }
        "removal" => {
            let s = s.split(". this ability costs").next().unwrap_or(s);
            let s = s.split_once(':').map_or(s, |(_, effect)| effect);
            // A preceding hand-reveal or combat trigger must not suppress an independent damage effect.
            let s = if let Some(i) = s.rfind("deals ") {
                &s[i..]
            } else if let Some(i) = s.rfind("deal ") {
                &s[i..]
            } else {
                s
            };
            let s = s
                .split(". ")
                .find(|part| {
                    has(
                        part,
                        &[
                            "destroy",
                            "exile",
                            "return",
                            "put target",
                            "gets -",
                            "get -",
                            "damage",
                            "fights",
                            "sacrifices",
                            "exiles",
                            "gain control",
                            "enchanted",
                            "-1/-1 counter",
                            "fight each other",
                            "annihilator",
                            "it gets -",
                        ],
                    )
                })
                .unwrap_or(s);
            if s.trim_start().starts_with("search your library") || s.contains("you control enchanted") {
                return s.contains("you control enchanted");
            }
            if has(s, &["from their hand", "from your hand"]) && s.contains("exile") {
                return false;
            }
            if s.contains("exile") && has(s, &["graveyard", "exile this", "exile ~"]) && !s.contains("exile target") {
                return false;
            }
            if s.contains("annihilator") {
                return true;
            }
            if has(s, &["graveyard", "your hand", "target spell", "target spells", "target card"]) {
                return false;
            }
            if s.contains("you control")
                && !has(
                    s,
                    &[
                        "don't control",
                        "doesn't control",
                        "opponent controls",
                        "fights",
                        "fight each other",
                        "damage to",
                        "each opponent",
                        "-1/-1 counter",
                        "gain control of",
                        "another target",
                        "opponents control",
                    ],
                )
            {
                return false;
            }
            if s.contains("return") && s.contains("battlefield") {
                return false;
            }
            if has(s, &["deals combat damage", "deal combat damage"]) {
                return false;
            }
            !s.contains("gain control of this") && !s.contains("gain control of ~")
        }
        "blink" => !s.contains("owner's hand") && !s.contains("your hand"),
        "protection" => {
            (grant
                || s.contains("phase")
                || has(
                    s,
                    &[
                        "prevent all",
                        "prevent the next",
                        "regenerate target",
                        "regenerate another",
                        "regenerate each",
                        "indestructible counter on each",
                        "umbra armor",
                    ],
                ))
                && !(s.contains("attacking") && s.contains("unless"))
                && !s.contains("loses all other abilities")
        }
        "draw" => {
            if !s.contains("draw") && (s.contains("investigate") || creates(s, "clue") || creates(s, "junk")) {
                return repeat && !self_sac && !spell;
            }
            if s.contains("cascade") || s.contains("mill a card") && s.contains("may play that card") {
                return true;
            }
            if s.contains("draw") {
                if has(s, &["whenever an opponent draws", "whenever you draw"])
                    && !has(s, &["you may draw", "you draw", ": draw", ", draw"])
                {
                    return false;
                }
                if !has(
                    s,
                    &[
                        "draw a",
                        "draw an",
                        "draw two",
                        "draw three",
                        "draw four",
                        "draw seven",
                        "draw x",
                        "draw cards",
                        "draw that",
                        "draw a card",
                        "draws a card",
                        "draws two",
                        "draws three",
                        "draws x",
                        "draws cards",
                        "draws that",
                        "draw an additional",
                    ],
                ) {
                    return false;
                }
                if s.contains("discards any number of cards, then draws that many") {
                    return false;
                }
                if !s.contains("draw an additional")
                    && has(
                        s,
                        &[
                            "discard a card, then draw a card",
                            "draw a card, then discard a card",
                            "draw two cards, then discard two",
                            "discard a card: draw a card",
                        ],
                    )
                {
                    return false;
                }
                if has(s, &["draw a card", "draws a card"])
                    && !has(s, &["for each", "equal to"])
                    && (spell || self_sac)
                    && !c.text.to_lowercase().contains("choose the same mode more than once")
                    && !c.text.to_lowercase().contains("choose one or more")
                {
                    return false;
                }
                return true;
            }
            if s.contains("search your library") {
                return false;
            }
            if s.contains("repeat this process") && s.contains("into your hand") {
                return true;
            }
            if has(s, &["look at the top", "reveal the top"]) && s.contains("from among them") {
                return false;
            }
            has(
                s,
                &[
                    "top of your library",
                    "exile the top",
                    "exile a nonland card",
                    "shuffle your library, then exile",
                    "remains exiled",
                    "cast the exiled",
                    "play the exiled",
                    "cast cards exiled",
                    "play cards exiled",
                    "exile the top card of each player's library",
                ],
            ) && has(s, &["may play", "may cast", "may plot"])
                || !spell
                    && !s.contains("search your library")
                    && !s.contains("graveyard")
                    && !has(s, &["look at the top", "reveal the top"])
                    && s.contains("into your hand")
                || s.contains("pile into your hand")
        }
        "selection" => {
            if creates(s, "blood") {
                return repeat && !self_sac && !spell;
            }
            if has(s, &["connive", "recruit", "learn"]) {
                return true;
            }
            if has(
                s,
                &[
                    "repeat this process",
                    "search your library",
                    "opponent's library",
                    "opponent’s library",
                    "separates",
                    "you win the game",
                    "discard your hand",
                    "draw an additional",
                ],
            ) {
                return false;
            }
            // Looting is selection only when it does not also net cards. Optional discard after drawing is incidental.
            if s.contains("draw") && s.contains("discard") {
                if has(s, &["counter target", "destroy target", "exile target"]) {
                    return false;
                }
                return has(
                    s,
                    &[
                        "draw a card, then discard a card",
                        "discard a card, then draw a card",
                        "discard a card: draw a card",
                        "draw two cards, then discard two cards",
                        "discards any number of cards, then draws that many",
                    ],
                );
            }
            s.contains("scry")
                || s.contains("surveil")
                || has(s, &["into your hand", "from among them into your hand"])
                || s.contains("look at the top") && has(s, &["put", "shuffle"])
        }
        "discard" => !(s.contains("draw") && has(s, &["each player", "players discard"])),
        "gy-hate" => {
            // Paying with your own graveyard is a cost, not disruption. Exile must act on a graveyard card,
            // not put a card from exile into a graveyard or borrow cards before returning them to play.
            if has(s, &["instead exile", "graveyard can't", "graveyards can't", "can't cast", "can't enter"]) {
                return !s.contains("your graveyard");
            }
            !s.contains("your graveyard") && !s.contains("onto the battlefield") && !s.contains("then puts all cards")
        }
        "recursion" => {
            if !has(s, &["hand", "library"]) {
                if let Some(object) = battlefield_object(s) {
                    return s.contains("your graveyard")
                        && has(
                            object,
                            &["artifact", "enchantment", "planeswalker", "land card", "permanent card", "noncreature"],
                        );
                }
            }
            !has(s, &["cast this card", "cast ~", "additional cost"])
                && (s.contains("soulshift")
                    || s.contains("shuffle your graveyard")
                    || s.contains("from your graveyard")
                    || s.contains("in your graveyard")
                    || s.contains("into your graveyard") && s.contains("into your library")
                    || s.contains("is in your graveyard")
                    || s.contains("card in your graveyard")
                    || s.contains("from their graveyard")
                    || s.contains("your graveyard has")
                    || s.contains("cards in graveyards"))
        }
        "reanimate" => {
            if has(s, &["return this", "return ~", "from anywhere"]) {
                return false;
            }
            if let Some(object) = battlefield_object(s) {
                return !object.contains("noncreature")
                    && (object.contains("creature") || object.contains("permanent card"));
            }
            s.contains("creature") && s.contains("dies, return it to the battlefield")
                || s.contains("opponent sacrifices a nontoken permanent") && s.contains("put that card onto")
        }
        "self-mill" => {
            if s.starts_with("discard a card:") {
                return true;
            }
            if s.contains("mill") {
                return !has(s, &["target player mills", "opponent mills", "opponents mill"]);
            }
            !has(s, &["opponent separates", "an opponent separates", "from anywhere", "dies", "sacrifice"])
                && (s.contains("search your library")
                    || s.find("your library").zip(s.find("your graveyard")).is_some_and(|(l, g)| l < g)
                    || s.contains("from among"))
        }
        "copy" => !s.contains("a copy of ~") && !s.contains("copy of this"),
        "tokens" => {
            if has(s, &["its controller creates", "that player creates", "opponent creates", "they create"])
                && !has(s, &["you create", "each player creates", "another target player creates"])
            {
                return false;
            }
            if has(s, &["investigate", "amass", "populate", "incubate", "mobilize", "afterlife", "recruit"]) {
                return true;
            }
            // Inspect the created object, not an unrelated creature in the trigger, target or later sentence.
            let object = s.split("create").nth(1).unwrap_or("").split("token").next().unwrap_or("");
            has(object, &["creature", "treasure", "clue"])
                || s.contains("token that's a copy")
                    && (has(object, &["attacking", "a "])
                        || has(s.split("copy").nth(1).unwrap_or(""), &["creature", "permanent"]))
        }
        "anthem" => {
            if s.contains("+1/+1 counter on each creature") {
                return has(s, &["each creature you control", "each creature target player controls"]);
            }
            let subject = s
                .split([',', '.', ':'])
                .find(|p| {
                    has(
                        p,
                        &[
                            "creatures",
                            "creature you control",
                            "all ",
                            "s you control",
                            "creatures with",
                            "each other ",
                        ],
                    )
                })
                .unwrap_or("");
            // Only the team itself may receive the buff; a cost or condition mentioning your team is insufficient.
            let buff = [" get ", " gets "]
                .iter()
                .filter_map(|v| subject.find(v))
                .min()
                .or_else(|| [" have ", " gain "].iter().filter_map(|v| subject.rfind(v)).max());
            let Some(buff) = buff else {
                return false;
            };
            let team = &subject[..buff];
            let payoff = &subject[buff..];
            (has(team, &["creatures", "creature you control", "s you control", "each other "])
                || team.trim().starts_with("all "))
                && !has(
                    team,
                    &[
                        "target creature ",
                        "this creature",
                        "this token",
                        "~",
                        "equipped creature",
                        "enchanted creature",
                    ],
                )
                && has(
                    payoff,
                    &[
                        "get +",
                        "gets +",
                        "get an additional",
                        "double strike",
                        "flying",
                        "menace",
                        "trample",
                        "vigilance",
                        "first strike",
                        "deathtouch",
                        "fear",
                        "haste",
                    ],
                )
        }
        "sac-outlet" => activation && !self_sac,
        "drain" => {
            !has(
                s,
                &[
                    "combat damage",
                    "damage to any target",
                    "damage to target creature",
                    "damage to you",
                    "whenever an opponent loses",
                    "whenever a player loses",
                ],
            ) && (repeat || s.contains("each opponent"))
        }
        "lifegain" => {
            if creates(s, "food") {
                return repeat && !self_sac && !spell;
            }
            if s.contains("double") && s.contains("life total") {
                return true;
            }
            if s.contains("lifelink") {
                return !spell
                    && (!s.contains("until end of turn") || activation)
                    && (grant || !has(s, &["gain", "has", "have"]) && s.split(", ").any(|w| w == "lifelink"));
            }
            if !has(
                s,
                &[
                    "you gain",
                    "you may gain",
                    "its controller gains",
                    "you've gained",
                    "you gained",
                    "whenever you gain",
                    "whenever you gain",
                    "whenever.*gain",
                ],
            ) {
                return false;
            }
            (repeat && !self_sac)
                || !has(
                    s,
                    &[
                        "destroy",
                        "exile target",
                        "search your library",
                        "look at",
                        "draw a card",
                        "put it into your hand",
                    ],
                ) && (!spell
                    || s.contains("life lost this way")
                    || s.contains("gain x")
                    || s.contains("gain 7")
                    || s.starts_with("you gain"))
        }
        "counters" => {
            if s.contains("proliferate") {
                return true;
            }
            if s.contains("amass") || creates(s, "map") {
                return repeat && !self_sac && !spell;
            }
            let effect = s.rfind("put ").map_or(s, |p| &s[p..]);
            if has(
                effect,
                &[
                    "ki counter",
                    "suspect counter",
                    "void counter",
                    "finality counter",
                    "time counter",
                    "lore counter",
                    "age counter",
                    "stun counter",
                    "oil counter",
                    "quest counter",
                    "loyalty counter",
                    "level counter",
                    "spite counter",
                    "valor counter",
                    "verse counter",
                    "conqueror counter",
                    "poison counter",
                    "energy counter",
                    "experience counter",
                    "rad counter",
                ],
            ) {
                return false;
            }
            if !repeat && has(effect, &["on this creature", "on ~"]) && !has(effect, &["target", "on each", "instead"])
            {
                return false;
            }
            if s.contains("charge counter")
                && !has(s, &["counter on target artifact", "counter on another", "counter on each artifact"])
            {
                return false;
            }
            !has(s, &["enters with", "enters the battlefield with", "put ~'s counters", "put this creature's counters"])
                && !(spell && s.contains("if it's a creature") && has(s, &["hexproof", "indestructible"]))
                && (has(
                    s,
                    &[
                        "+1/+1",
                        "-1/-1",
                        "proliferate",
                        "backup",
                        "bolster",
                        "support",
                        "adapt",
                        "awaken",
                        "connive",
                        "dethrone",
                        "mentor",
                        "evolve",
                        "training",
                        "target creature",
                        "target artifact",
                        "on each",
                        "on another",
                    ],
                ) || repeat)
        }
        "extra-phase" => true,
        "evasion" => (grant || s.starts_with("all ")) && !s.contains("can't attack"),
        "stax" => {
            !(s.contains("cost") && s.contains("less"))
                && !s.contains("this spell")
                && !s.contains("~ can't")
                && !s.contains("this creature can't")
                && !s.contains("you can't")
                && !s.contains("unless you")
        }
        _ => false,
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Override {
    pub name: String,
    pub roles: Vec<String>,
    pub note: String,
}
#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Overrides {
    #[serde(default)]
    card: Vec<Override>,
}
pub fn generate(index: &Index, overrides: &str) -> Result<RoleMap, String> {
    let classifier = Classifier::new()?;
    let mut result = RoleMap::new();
    for c in index.cards.iter().filter(|c| is_deck_card(c)) {
        result.insert(c.raw.oracle_id.clone(), classifier.classify(c)?);
    }
    let overrides: Overrides = toml::from_str(overrides).map_err(|e| format!("roles_overrides.toml: {e}"))?;
    let mut seen = BTreeSet::new();
    for o in overrides.card {
        if o.note.trim().is_empty()
            || o.roles.len() > 4
            || o.roles.iter().collect::<BTreeSet<_>>().len() != o.roles.len()
        {
            return Err(format!("invalid override for {}: need a note and at most four unique roles", o.name));
        }
        for r in &o.roles {
            check_role(r)?;
        }
        let ids = index.lookup(&o.name);
        if ids.len() != 1 {
            return Err(format!("override '{}' resolves to {} cards; use an unambiguous full name", o.name, ids.len()));
        }
        let id = &index.cards[ids[0] as usize].raw.oracle_id;
        if !result.contains_key(id) {
            return Err(format!("override '{}' is not a deck card", o.name));
        }
        if !seen.insert(id) {
            return Err(format!("duplicate override for {}", o.name));
        }
        let evidence = o
            .roles
            .iter()
            .map(|r| {
                (
                    r.clone(),
                    Evidence {
                        source: "override".into(),
                        certain: false,
                        rule: o.note.clone(),
                        face: None,
                        line: None,
                    },
                )
            })
            .collect();
        result.insert(
            id.clone(),
            Assignment {
                primary: o.roles.first().cloned().unwrap_or_default(),
                roles: o.roles,
                evidence,
                needs_llm: false,
            },
        );
    }
    Ok(result)
}

pub fn load(path: &Path) -> Result<RoleMap, String> {
    let f = std::fs::File::open(path).map_err(|e| format!("{}: {e}; run `cardgen roles --rules`", path.display()))?;
    serde_json::from_reader(std::io::BufReader::new(f)).map_err(|e| format!("{}: {e}", path.display()))
}
pub fn bytes(map: &RoleMap) -> Result<Vec<u8>, String> {
    let mut out = serde_json::to_vec_pretty(map).map_err(|e| e.to_string())?;
    out.push(b'\n');
    Ok(out)
}
fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))
}

#[derive(Default)]
struct Options {
    mode: String,
    dir: Option<PathBuf>,
    gold: Option<PathBuf>,
    json: bool,
    name: Option<String>,
    seed: Option<u64>,
    n: Option<usize>,
    rule: Option<String>,
    certain: bool,
}
fn options(args: &[String]) -> Result<Options, String> {
    let mut o = Options::default();
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        i += 1;
        match a {
            "--rules" | "--validate" | "--stats" | "--show" | "--audit" => {
                if !o.mode.is_empty() {
                    return Err(
                        "roles: choose exactly one of --rules, --validate, --stats, --show NAME, --audit".into()
                    );
                }
                o.mode = a.into();
                if a == "--show" {
                    let v = args.get(i).filter(|s| !s.starts_with("--")).ok_or("--show needs a name")?;
                    o.name = Some(v.clone());
                    i += 1;
                }
            }
            "--seed" | "--n" | "--rule" => {
                let v = args.get(i).ok_or_else(|| format!("{a} needs a value"))?;
                match a {
                    "--seed" if o.seed.is_none() => o.seed = Some(v.parse().map_err(|_| "invalid audit seed")?),
                    "--n" if o.n.is_none() => o.n = Some(v.parse().map_err(|_| "invalid audit size")?),
                    "--rule" if o.rule.is_none() && RULES.iter().any(|r| r.id == v) => o.rule = Some(v.clone()),
                    _ => return Err(format!("invalid or repeated {a}")),
                }
                i += 1;
            }
            "--certain" if !o.certain => o.certain = true,
            "--json" if !o.json => o.json = true,
            "--data-dir" | "--gold" => {
                let v = args.get(i).filter(|s| !s.starts_with("--")).ok_or_else(|| format!("{a} needs a path"))?;
                let slot = if a == "--gold" { &mut o.gold } else { &mut o.dir };
                if slot.is_some() {
                    return Err(format!("{a} given twice"));
                }
                *slot = Some(PathBuf::from(v));
                i += 1;
            }
            _ => return Err(format!("unknown roles argument '{a}'")),
        }
    }
    if o.mode.is_empty()
        || o.gold.is_some() && o.mode != "--validate"
        || o.json && o.mode != "--validate"
        || (o.seed.is_some() || o.n.is_some() || o.rule.is_some() || o.certain) && o.mode != "--audit"
    {
        return Err(concat!(
            "roles: choose --rules, --show NAME, --stats, --validate [--gold PATH] [--json], ",
            "or --audit [--seed N] [--n N] [--rule ID] [--certain]"
        )
        .into());
    }
    Ok(o)
}

/// Stable seeded sampling without replacement, independent of catalog order and saved overrides.
/// SplitMix64 mixes a fixed FNV-1a Oracle-id hash; no shared engine RNG or platform hash state.
pub fn audit(
    index: &Index,
    gold: &str,
    seed: u64,
    n: usize,
    rule: Option<&str>,
    certain_only: bool,
) -> Result<String, String> {
    if n == 0 {
        return Err("audit size must be positive".into());
    }
    if rule.is_some_and(|id| !RULES.iter().any(|r| r.id == id)) {
        return Err("unknown audit rule".into());
    }
    let gold: Gold = toml::from_str(gold).map_err(|e| format!("gold: {e}"))?;
    let excluded: BTreeSet<_> = gold.card.iter().map(|c| c.oracle_id.as_str()).collect();
    let key = |id: &str| {
        let h = id.bytes().fold(0xcbf29ce484222325u64, |h, b| (h ^ u64::from(b)).wrapping_mul(0x100000001b3));
        let mut z = (h ^ seed).wrapping_add(0x9e3779b97f4a7c15);
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    };
    let mut cards: Vec<_> = index
        .cards
        .iter()
        .filter(|c| is_deck_card(c) && c.legal == Legal::Legal && !excluded.contains(c.raw.oracle_id.as_str()))
        .collect();
    cards.sort_by_key(|c| (key(&c.raw.oracle_id), &c.raw.oracle_id));
    let classifier = Classifier::new()?;
    let mut out = format!(
        "AUDIT seed={seed} requested={n} rule={} certain_only={certain_only} (legal, gold excluded)\n",
        rule.unwrap_or("all")
    );
    let (mut shown, mut assignments, mut finals, mut final_assignments) = (0, 0, 0, 0);
    for c in cards {
        let a = classifier.classify(c)?;
        if !a.evidence.values().any(|e| rule.is_none_or(|id| e.rule == id) && (!certain_only || e.certain))
            && (rule.is_some() || certain_only)
        {
            continue;
        }
        shown += 1;
        assignments += a.roles.len();
        finals += usize::from(!a.needs_llm);
        if !a.needs_llm {
            final_assignments += a.roles.len();
        }
        let _ = writeln!(out, "\n[{shown}] {} | {} | {}", c.raw.name, c.raw.oracle_id, c.raw.type_line);
        for (face, line) in c.lines() {
            let _ = writeln!(out, "  Oracle face={face:?}: {line}");
        }
        let _ = writeln!(out, "  roles: {} | needs_llm={}", a.roles.join(", "), a.needs_llm);
        for r in &a.roles {
            let e = &a.evidence[r];
            let _ = writeln!(out, "    {r}: {} certain={} face={:?} line={:?}", e.rule, e.certain, e.face, e.line);
        }
        if shown == n {
            break;
        }
    }
    let _ = writeln!(
        out,
        "\nSHOWN {shown} cards, {assignments} assignments; rules-final {finals} cards, {final_assignments} assignments"
    );
    Ok(out)
}

/// Text and success are separate so a failing validation still prints its entire report before exiting nonzero.
pub fn command(project: &Project, args: &[String]) -> Result<(String, bool), String> {
    let o = options(args)?;
    let dir = knowledge::knowledge_dir(project, o.dir.as_deref())?;
    let mut index = Index::load(&dir.join("oracle.json"))?;
    let taxonomy = Taxonomy::parse(&read(&project.data("knowledge/roles.toml"))?)?;
    if o.mode == "--audit" {
        let gold = read(&project.data("knowledge/roles_gold.toml"))?;
        return Ok((audit(&index, &gold, o.seed.unwrap_or(0), o.n.unwrap_or(80), o.rule.as_deref(), o.certain)?, true));
    }
    let path = dir.join("roles.json");
    if o.mode == "--rules" {
        let map = generate(&index, &read(&project.data("knowledge/roles_overrides.toml"))?)?;
        let data = bytes(&map)?;
        let tmp = dir.join(format!("roles.{}.tmp", std::process::id()));
        std::fs::write(&tmp, data).map_err(|e| format!("{}: {e}", tmp.display()))?;
        if let Err(e) = std::fs::rename(&tmp, &path) {
            let _ = std::fs::remove_file(&tmp);
            return Err(format!("{}: {e}", path.display()));
        }
        let s = stats(&index, &map);
        return Ok((format!("wrote {} ({} cards)\n{}", path.display(), map.len(), s.text()), true));
    }
    let map = load(&path)?;
    index.attach_roles(&map)?;
    if o.mode == "--validate" {
        let gold = o.gold.unwrap_or_else(|| project.data("knowledge/roles_gold.toml"));
        let report = validate(project, &index, &map, &read(&gold)?, &taxonomy)?;
        let passed = report.passed;
        return Ok((
            if o.json { format!("{}\n", serde_json::to_string_pretty(&report).unwrap()) } else { report.text() },
            passed,
        ));
    }
    if o.mode == "--stats" {
        return Ok((stats(&index, &map).text(), true));
    }
    let ids = index.lookup(o.name.as_deref().unwrap());
    if ids.is_empty() {
        return Err(format!("no card named '{}'", o.name.unwrap()));
    }
    let mut out = String::new();
    for &i in ids {
        let c = &index.cards[i as usize];
        if !is_deck_card(c) {
            let _ = writeln!(out, "{}: excluded from roles (not a deck card)", c.raw.name);
            continue;
        }
        let a = &map[&c.raw.oracle_id];
        let _ = writeln!(
            out,
            "{}\nroles: {}\nprimary: {}\nneeds_llm: {}",
            c.raw.name,
            a.roles.join(", "),
            a.primary,
            a.needs_llm
        );
        let lines = c.lines();
        for r in &a.roles {
            let e = &a.evidence[r];
            let _ = writeln!(
                out,
                "  {r}: {} {} certain={} face={:?} line={:?}",
                e.source, e.rule, e.certain, e.face, e.line
            );
            if let Some((_, text)) = e.line.and_then(|i| lines.get(i)) {
                let _ = writeln!(out, "    {text}");
            }
        }
    }
    Ok((out, true))
}

#[derive(Default, Serialize)]
pub struct Stats {
    pub total: usize,
    pub skipped: usize,
    pub needs_llm: usize,
    pub no_role: usize,
    pub per_role: BTreeMap<String, usize>,
    pub routing: BTreeMap<String, (usize, usize)>,
}
pub fn stats(index: &Index, map: &RoleMap) -> Stats {
    let mut s = Stats { per_role: IDS.iter().map(|r| (r.to_string(), 0)).collect(), ..Stats::default() };
    for c in &index.cards {
        if !is_deck_card(c) {
            s.skipped += 1;
            continue;
        }
        let a = &map[&c.raw.oracle_id];
        s.total += 1;
        s.needs_llm += usize::from(a.needs_llm);
        s.no_role += usize::from(a.roles.is_empty());
        for r in &a.roles {
            *s.per_role.entry(r.clone()).or_default() += 1;
        }
        let group = s.routing.entry(format!("{} / {}", c.legal.as_str(), c.raw.layout)).or_default();
        group.0 += 1;
        group.1 += usize::from(a.needs_llm);
    }
    s
}
impl Stats {
    pub fn text(&self) -> String {
        let mut s = format!("ROUTING: {} cards, {} needs_llm, {} no role\n", self.total, self.needs_llm, self.no_role);
        let _ = writeln!(s, "SKIPPED: {} Stickers/Attractions (not deck cards)", self.skipped);
        let legal: Vec<_> = self.routing.iter().filter(|(g, _)| g.starts_with("legal / ")).map(|(_, n)| n).collect();
        let legal_total: usize = legal.iter().map(|n| n.0).sum();
        let legal_llm: usize = legal.iter().map(|n| n.1).sum();
        let _ = writeln!(
            s,
            "LEGAL ROUTING: {legal_llm}/{legal_total} = {:.2}% needs_llm",
            100.0 * ratio(legal_llm, legal_total)
        );
        for (r, n) in &self.per_role {
            let _ = writeln!(s, "  {r:<12} {n}");
        }
        for (g, (all, llm)) in &self.routing {
            let _ = writeln!(s, "  {g:<30} {llm:>6} / {all:<6} needs_llm");
        }
        s
    }
}

/// Fixed recall-only translation. A tag is not evidence for classification. Some engine tags encode an
/// approximation or an AI action, so expected pairs can conflict with the deck-building conventions.
/// Every tag in TAGS is reported with this mapping or the explicit unmapped reason below.
pub fn tag_roles(tag: &str, value: &serde_json::Value) -> (&'static [&'static str], &'static str) {
    let roles: &'static [&'static str] = match tag {
        "rem" | "edictetb" | "brushoff" | "primalmight" | "snuff" | "verdict" => &["removal"],
        "wipe" | "deluge" => &["wipe"],
        "ctr" | "ctrexile" | "denial" | "fierce" | "fon" | "lapse" | "misstep" | "offer" | "pact" | "remand"
        | "swan" | "undermine" => &["counter"],
        "tut" | "xtutor" | "seal" | "top" | "gifts" | "intuition" | "threedreams" => &["tutor"],
        "rock" | "dork" | "lr" | "lrt" | "treas" | "fastmana" | "chromemox" | "moxd" | "led" | "grim" | "tomb"
        | "birgi" | "rishkar" | "jeska" => &["ramp"],
        "draw" | "fbdraw" | "drawcre" | "eng" | "eotdraw" | "enddraw" | "spelldraw" | "remora" | "rhystic"
        | "sphinx" | "clamp" | "adnaus" | "necro" | "onering" | "callring" | "citadel" => &["draw"],
        "rean" | "unearth" | "necromancer" | "suntitan" | "reenact" => &["reanimate"],
        "fill" | "dredge" => &["self-mill"],
        "sac" => &["sac-outlet"],
        "bartist" | "drain" | "drainetb" | "wurmdrain" | "xdrain" | "gray" | "sheoA" => &["drain"],
        "prot" if value.as_str() == Some("blink") => &["blink"],
        "prot" | "mother" | "melira" | "helm" => &["protection"],
        "tok" | "mktok" | "tokatk" | "tokbig" | "tokdt" | "tokfly" | "toklife" | "tokp" | "tokpw" | "tokup"
        | "tokx" | "spelltok" | "spelltokfly" | "titan" | "rhys" | "populate" | "tithe" => &["tokens"],
        "anth" | "anthem2" | "anthemnh" | "pumpall" | "crusade" => &["anthem"],
        "lifelink" | "fblife" | "lh" => &["lifegain"],
        "prolif" | "prolif1" | "prolifall" | "pvprolif" | "upkprolif" | "ozolith" => &["counters"],
        "spellloot" => &["selection"],
        "discard1" | "pwdiscard" => &["discard"],
        "clone" | "dualcaster" | "twinflame" | "mirror" | "panoptic" => &["copy"],
        "wisp" | "vkitten" => &["blink"],
        "witness" | "yawg" | "breach" | "loam" | "fbgrant" => &["recursion"],
        "assault" | "warleader" | "ralzarek" => &["extra-phase"],
        "passage" => &["evasion"],
        _ => &[],
    };
    if !roles.is_empty() {
        let reason = if tag == "prot" {
            "prot=blink maps to blink; other prot values map to protection"
        } else {
            "engine effect translated to the deck-building taxonomy"
        };
        return (roles, reason);
    }
    let reason = match tag {
        "vaultping" => "damage to its own controller; not opposing drain",
        "c" | "oneclr" | "f" | "bounceland" | "rland" => {
            "mana/land metadata; does not by itself imply extra mana or multiple colours"
        }
        "fly" | "dt" | "haste" | "reach" | "trample" | "vig" | "fs" | "swampwalk" => {
            "printed self keyword; conventions exclude evasion/protection"
        }
        "bomb" | "pow" | "tgh" | "amt" | "tgt" | "kick" | "lose" | "selfdmg" | "rgain" | "rtok" => {
            "effect magnitude, target, cost or opponent compensation; not a main role"
        }
        "flash" | "free" | "freecmd" | "pactpay" | "phyU" | "fb" | "etb" | "needsac" | "nountap" | "ck" | "pain" => {
            "casting, timing or cost metadata; does not establish a role"
        }
        "human" | "warrior" | "wizard" | "rogue" | "shaman" | "knight" | "leg" | "aura" => {
            "type metadata, not a deck-building role"
        }
        _ => "engine hook/AI flag or narrow modifier; requires Oracle interpretation, no universal role implication",
    };
    (roles, reason)
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct Score {
    pub tp: usize,
    pub fp: usize,
    pub missed: usize,
}
impl Score {
    pub fn precision(&self) -> f64 {
        ratio(self.tp, self.tp + self.fp)
    }
    pub fn recall(&self) -> f64 {
        ratio(self.tp, self.tp + self.missed)
    }
    fn add(&mut self, s: &Self) {
        self.tp += s.tp;
        self.fp += s.fp;
        self.missed += s.missed;
    }
}
fn ratio(a: usize, b: usize) -> f64 {
    if b == 0 { 1.0 } else { a as f64 / b as f64 }
}
#[derive(Serialize)]
pub struct TagTranslation {
    pub roles: Vec<String>,
    pub reason: String,
}
#[derive(Serialize)]
pub struct Validation {
    pub passed: bool,
    pub hand: BTreeMap<String, Score>,
    pub hand_micro: Score,
    pub gold_all: BTreeMap<String, Score>,
    pub gold_certain: BTreeMap<String, Score>,
    pub gold_rules_micro: Score,
    pub gold_other_micro: Score,
    pub certain_micro: Score,
    pub rules_final_micro: Score,
    pub rules_final_cards: usize,
    pub not_roles: Vec<String>,
    pub certain_not_roles: Vec<String>,
    pub failures: Vec<String>,
    pub hand_misses: Vec<String>,
    pub gold_differences: Vec<String>,
    pub tag_mapping: BTreeMap<String, TagTranslation>,
    pub routing: Stats,
}
#[derive(Deserialize)]
struct Gold {
    card: Vec<GoldCard>,
}
#[derive(Deserialize)]
struct GoldCard {
    name: String,
    oracle_id: String,
    roles: Vec<String>,
    primary: String,
    #[serde(default)]
    not_roles: Vec<String>,
}

pub fn validate(
    project: &Project,
    index: &Index,
    _saved: &RoleMap,
    gold: &str,
    taxonomy: &Taxonomy,
) -> Result<Validation, String> {
    // Reclassify without overrides: a manual role set can never improve rule validation scores.
    let rules = generate(index, "")?;
    let mut v = Validation {
        passed: true,
        hand: BTreeMap::new(),
        hand_micro: Score::default(),
        gold_all: BTreeMap::new(),
        gold_certain: BTreeMap::new(),
        gold_rules_micro: Score::default(),
        gold_other_micro: Score::default(),
        certain_micro: Score::default(),
        rules_final_micro: Score::default(),
        rules_final_cards: 0,
        not_roles: vec![],
        certain_not_roles: vec![],
        failures: vec![],
        hand_misses: vec![],
        gold_differences: vec![],
        tag_mapping: BTreeMap::new(),
        routing: stats(index, &rules),
    };
    for (_, tag, _) in sim_core::tag::TAGS {
        let (roles, reason) = tag_roles(tag, &serde_json::Value::Null);
        v.tag_mapping.insert(
            tag.into(),
            TagTranslation { roles: roles.iter().map(|s| s.to_string()).collect(), reason: reason.into() },
        );
    }
    let decks: serde_json::Value =
        serde_json::from_str(&read(&project.data("decks.json"))?).map_err(|e| e.to_string())?;
    let mine = decks["mine"].as_array().ok_or("decks.json: expected mine array")?;
    let mut names = BTreeSet::new();
    for d in mine {
        let deck: knowledge::Deck = serde_json::from_value(d.clone()).map_err(|e| format!("mine deck: {e}"))?;
        let (members, unresolved) = index.deck_members(&deck);
        for n in unresolved {
            v.hand_misses.push(format!("unresolved deck member: {n}"));
        }
        for (i, _) in members.iter().enumerate().filter(|(_, m)| **m) {
            names.insert(i as u32);
        }
    }
    let cards: serde_json::Value =
        serde_json::from_str(&read(&project.data("cards.json"))?).map_err(|e| e.to_string())?;
    let mut pairs = BTreeSet::new();
    for c in cards["cards"].as_array().ok_or("cards.json: expected cards array")? {
        if !matches!(c["source"].as_str(), Some("manual" | "pool")) {
            continue;
        }
        let name = c["name"].as_str().ok_or("engine card missing name")?;
        let tags = c["tags"].as_object().ok_or("engine card missing tags")?;
        for &i in index.lookup(name).iter().filter(|i| names.contains(i)) {
            for (tag, value) in tags {
                for &role in tag_roles(tag, value).0 {
                    pairs.insert((i, role));
                }
            }
        }
    }
    if pairs.is_empty() {
        return Err("hand-tag validation found no mapped pairs".into());
    }
    for (i, role) in pairs {
        let c = &index.cards[i as usize];
        let score = v.hand.entry(role.into()).or_default();
        if rules[&c.raw.oracle_id].roles.iter().any(|r| r == role) {
            score.tp += 1;
        } else {
            score.missed += 1;
            v.hand_misses.push(format!("{} -> {role}", c.raw.name));
        }
    }
    for s in v.hand.values() {
        v.hand_micro.add(s);
    }
    let gold: Gold = toml::from_str(gold).map_err(|e| format!("gold: {e}"))?;
    if gold.card.is_empty() {
        return Err("gold set is empty".into());
    }
    let mut ids = BTreeSet::new();
    for g in gold.card {
        if !ids.insert(g.oracle_id.clone()) {
            return Err(format!("duplicate gold id {}", g.oracle_id));
        }
        for r in g.roles.iter().chain(&g.not_roles) {
            check_role(r)?;
        }
        if g.roles.len() > 4
            || g.roles.first().map_or("", String::as_str) != g.primary
            || g.roles.iter().any(|r| g.not_roles.contains(r))
        {
            return Err(format!("inconsistent gold labels for {}", g.name));
        }
        let i = index
            .by_oracle_id(&g.oracle_id)
            .ok_or_else(|| format!("gold card absent: {} ({})", g.name, g.oracle_id))?;
        if !is_deck_card(&index.cards[i as usize]) {
            return Err(format!("gold card is not a deck card: {}", g.name));
        }
        let a = &rules[&index.cards[i as usize].raw.oracle_id];
        v.rules_final_cards += usize::from(!a.needs_llm);
        for &role in &IDS {
            let expected = g.roles.iter().any(|r| r == role);
            let predicted = a.roles.iter().any(|r| r == role);
            let certain = a.evidence.get(role).is_some_and(|e| e.certain);
            for (map, got) in [(&mut v.gold_all, predicted), (&mut v.gold_certain, certain)] {
                let score = map.entry(role.into()).or_default();
                match (expected, got) {
                    (true, true) => score.tp += 1,
                    (false, true) => score.fp += 1,
                    (true, false) => score.missed += 1,
                    _ => {}
                }
            }
            if !a.needs_llm {
                match (expected, predicted) {
                    (true, true) => v.rules_final_micro.tp += 1,
                    (false, true) => v.rules_final_micro.fp += 1,
                    (true, false) => v.rules_final_micro.missed += 1,
                    _ => {}
                }
            }
            if expected != predicted {
                v.gold_differences.push(format!(
                    "{} -> {role}: {}",
                    g.name,
                    if predicted { "unexpected" } else { "missing" }
                ));
            }
            if predicted && g.not_roles.iter().any(|r| r == role) {
                let pair = format!("{} -> {role}", g.name);
                v.not_roles.push(pair.clone());
                if certain {
                    v.certain_not_roles.push(pair);
                }
            }
        }
    }
    for role in &taxonomy.role {
        let dest = if role.src == "rules" { &mut v.gold_rules_micro } else { &mut v.gold_other_micro };
        dest.add(&v.gold_all[&role.id]);
        v.certain_micro.add(&v.gold_certain[&role.id]);
    }
    if v.hand_micro.recall() < 0.95 {
        v.failures.push("hand micro recall < 0.95".into());
    }
    for (r, s) in &v.hand {
        if s.tp + s.missed >= 10 && s.recall() < 0.85 {
            v.failures.push(format!("hand {r} recall < 0.85"));
        }
    }
    if v.gold_rules_micro.precision() < 0.90 {
        v.failures.push("gold rules precision < 0.90".into());
    }
    if v.gold_rules_micro.recall() < 0.85 {
        v.failures.push("gold rules recall < 0.85".into());
    }
    if v.rules_final_micro.precision() < 0.93 {
        v.failures.push("rules-final precision < 0.93".into());
    }
    if v.rules_final_micro.recall() < 0.85 {
        v.failures.push("rules-final recall < 0.85".into());
    }
    if v.certain_micro.precision() < 0.95 {
        v.failures.push("certain precision < 0.95".into());
    }
    if !v.certain_not_roles.is_empty() {
        v.failures.push("certain not_roles violations".into());
    }
    v.passed = v.failures.is_empty();
    Ok(v)
}
impl Validation {
    pub fn text(&self) -> String {
        let row = |out: &mut String, r: &str, s: &Score| {
            let _ = writeln!(
                out,
                "  {r:<16} {:>4} {:>4} {:>4}   {:.4}   {:.4}",
                s.tp,
                s.fp,
                s.missed,
                s.precision(),
                s.recall()
            );
        };
        let mut out = "HAND-TAG RECALL (tags are not exhaustive; precision is not measured)\n".to_string();
        out.push_str("  role               tp miss   recall\n");
        for (r, s) in self.hand.iter().map(|(r, s)| (r.as_str(), s)).chain([("micro", &self.hand_micro)]) {
            let _ = writeln!(out, "  {r:<16} {:>4} {:>4}   {:.4}", s.tp, s.missed, s.recall());
        }
        for (title, scores) in [("GOLD ALL RULES", &self.gold_all), ("GOLD CERTAIN ONLY", &self.gold_certain)] {
            let _ = writeln!(out, "{title}\n  role               tp   fp miss   precision recall");
            for (r, s) in scores {
                row(&mut out, r, s);
            }
        }
        out.push_str("GOLD MICRO (all rules / all other roles / certain assignments)\n");
        row(&mut out, "rules micro", &self.gold_rules_micro);
        row(&mut out, "other micro", &self.gold_other_micro);
        row(&mut out, "certain micro", &self.certain_micro);
        let _ = writeln!(out, "RULES-FINAL SUBSET ({} cards; all roles)", self.rules_final_cards);
        row(&mut out, "final micro", &self.rules_final_micro);
        let _ = writeln!(
            out,
            "not_roles violations: {} all, {} certain",
            self.not_roles.len(),
            self.certain_not_roles.len()
        );
        for (title, items) in [
            ("Hand misses", &self.hand_misses),
            ("Gold differences", &self.gold_differences),
            ("not_roles", &self.not_roles),
        ] {
            let _ = writeln!(out, "{title}:");
            for s in items {
                let _ = writeln!(out, "  {s}");
            }
        }
        out.push_str(&self.routing.text());
        out.push_str("TAG TRANSLATION (recall only):\n");
        for (tag, t) in &self.tag_mapping {
            let _ = writeln!(
                out,
                "  {tag}: {} — {}",
                if t.roles.is_empty() { "unmapped".into() } else { t.roles.join(",") },
                t.reason
            );
        }
        let _ = writeln!(
            out,
            "{}",
            if self.passed { "PASS".into() } else { format!("FAIL: {}", self.failures.join("; ")) }
        );
        out
    }
}
