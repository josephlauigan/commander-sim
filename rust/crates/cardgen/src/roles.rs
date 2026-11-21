//! Deterministic deck-building roles, deliberately separate from executable engine tags.
//! Rules read normalized Oracle lines, never card names or deck membership. Validation uses independent labels.
use crate::data::Project;
use crate::knowledge::{self, Card, Index};
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
    pub certain: bool,
    pub id: &'static str,
}
pub const RULES: &[Rule] = &[
    Rule {
        role: "finisher",
        pattern: r"you win the game|loses the game",
        negative: r"can't win|can't lose",
        certain: false,
        id: "alternate-win",
    },
    Rule {
        role: "ramp",
        pattern: concat!(
            r"\badd\b.*(?:mana|\{)|costs? (?:\{[^}]+\}|[0-9]+).*less|pay .*life rather than pay|",
            r"additional lands? on",
        ),
        negative: r"^this spell costs|^~ costs|^this ability costs",
        certain: false,
        id: "mana",
    },
    Rule {
        role: "ramp",
        pattern: r"search your library.*(?:land|plains|island|swamp|mountain|forest).*battlefield",
        negative: r"",
        certain: false,
        id: "land-ramp",
    },
    Rule {
        role: "fixing",
        pattern: r"\badd\b|search your library|basic landcycling|mana of any",
        negative: r"",
        certain: false,
        id: "fixing",
    },
    Rule { role: "tutor", pattern: r"search your library", negative: r"", certain: true, id: "library-search" },
    Rule {
        role: "counter",
        pattern: concat!(
            r"\bcounter (?:target|that|each|all) (?:[^.]*?)(?:spell|ability|abilities)\b|",
            r"exile (?:any number of )?target spells?",
        ),
        negative: r"",
        certain: true,
        id: "counter-spell",
    },
    Rule {
        role: "wipe",
        pattern: concat!(
            r"(?:destroy|exile|return|sacrifice) (?:all|each)|\b(?:all|each|",
            r"creatures)\b.*get -[1-9x][0-9]*/-[1-9x]|deals? .*damage (?:[^.]*? )?to (?:each|all)|",
            r"overload",
        ),
        negative: r"",
        certain: false,
        id: "mass-removal",
    },
    Rule {
        role: "removal",
        pattern: concat!(
            r"(?:destroy|exile|return) (?:up to [a-z]+ )?target|",
            r"target.*(?:gets? -[0-9x]+/-[1-9x][0-9]*|sacrifices? (?:a|an|one)|fights)|",
            r"deals? .*damage to (?:any target|target (?:creature|artifact|planeswalker))|",
            r"gain control of|owner of target .*shuffles|enchanted .*?(?:can't attack|",
            r"doesn't untap)|at the beginning.*?\btap (?:it|enchanted creature)",
        ),
        negative: r"until end of turn.*gain control|gain control.*until end of turn",
        certain: false,
        id: "single-removal",
    },
    Rule {
        role: "blink",
        pattern: concat!(
            r"exile .*?(?:you control|~|this creature|this permanent).*?return|",
            r"exile .*?return (?:it|that card|those cards|them).*?battlefield",
        ),
        negative: r"graveyard",
        certain: false,
        id: "exile-return",
    },
    Rule {
        role: "protection",
        pattern: concat!(
            r"(?:gain|gains|have|has).*?(?:hexproof|shroud|indestructible|protection from)|",
            r"phases? out|prevent all (?:combat )?damage|you have protection",
        ),
        negative: r"",
        certain: false,
        id: "protect",
    },
    Rule {
        role: "draw",
        pattern: concat!(
            r"\bdraw\b|(?:play|cast|plot) .*?(?:top of your library|exiled)|(?:put|",
            r"return).*?(?:into|to) your hand|graveyard.*?(?:into your hand|has escape)",
        ),
        negative: r"",
        certain: false,
        id: "card-advantage",
    },
    Rule {
        role: "selection",
        pattern: concat!(
            r"\b(?:scry|surveil) (?:[2-9]|[1-9][0-9]+|x)\b|draw .*?discard|discard .*?draw|",
            r"look at the top|from among them into your hand|reveal the top",
        ),
        negative: r"",
        certain: false,
        id: "filter",
    },
    Rule {
        role: "discard",
        pattern: r"(?:opponents?|target player|that player|each player|players|they) discards?\b",
        negative: r"",
        certain: false,
        id: "opponent-discard",
    },
    Rule {
        role: "gy-hate",
        pattern: r"exile .*?graveyard|exiles .*?graveyard|graveyard.*?(?:can't|instead)|can't .*?graveyard",
        negative: r"",
        certain: false,
        id: "graveyard-hate",
    },
    Rule {
        role: "recursion",
        pattern: concat!(
            r"(?:return|put).*?(?:graveyard.*?(?:hand|library)|(?:hand|library).*?graveyard)|",
            r"(?:cast|play).*?(?:from|in) your graveyard|your graveyard.*?(?:cast|play|",
            r"gains flashback|has escape|into your hand)",
        ),
        negative: r"",
        certain: false,
        id: "graveyard-reuse",
    },
    Rule {
        role: "reanimate",
        pattern: concat!(
            r"(?:return|put).*?(?:graveyard|creature card).*?battlefield|",
            r"return from your graveyard to the battlefield|",
            r"graveyard.*?creature.*?put.*?battlefield",
        ),
        negative: r"",
        certain: false,
        id: "graveyard-battlefield",
    },
    Rule {
        role: "self-mill",
        pattern: r"\bmill\b|(?:put|putting).*?(?:into|in) your graveyard|^discard a card:",
        negative: r"",
        certain: false,
        id: "fill-graveyard",
    },
    Rule {
        role: "copy",
        pattern: r"\bcopy (?:target|that|it|the|up to)|(?:as|becomes?|that's|that is|are) (?:a )?cop(?:y|ies) of",
        negative: r"cast a copy of its spell",
        certain: false,
        id: "copy",
    },
    Rule {
        role: "tokens",
        pattern: r"\bcreate\b.*?tokens?|\b(?:investigate|amass|populate|incubate)\b",
        negative: r"",
        certain: false,
        id: "create-token",
    },
    Rule {
        role: "anthem",
        pattern: concat!(
            r"(?:creatures|creature tokens|[a-z]+s) you control.*?(?:get|have|gain)|",
            r"\bother .*?(?:get|have)|creatures .*?you control.*?(?:get|have|gain)",
        ),
        negative: r"",
        certain: false,
        id: "team-pump",
    },
    Rule {
        role: "sac-outlet",
        pattern: r"sacrifice (?:a|an|another|any number|[a-z]+ or another) .*?:",
        negative: r"sacrifice (?:a|an) (?:treasure|clue|food|blood|map|powerstone)\b",
        certain: false,
        id: "sacrifice-cost",
    },
    Rule {
        role: "drain",
        pattern: concat!(
            r"(?:opponents?|that player|each player|target player|they).*?loses? .*?life|",
            r"deals? .*?damage to (?:each opponent|that player|target player|each player|you)",
        ),
        negative: r"",
        certain: false,
        id: "life-loss",
    },
    Rule {
        role: "lifegain",
        pattern: r"\b(?:gain|gains|gained) .*?life|\blifelink\b",
        negative: r"can't gain|can't.*?lose life",
        certain: false,
        id: "gain-life",
    },
    Rule {
        role: "counters",
        pattern: concat!(
            r"\b(?:put|double|move)\b.*?counters?|\bproliferate\b|^backup [1-9]|^bolster [1-9]|",
            r"^support [1-9]|^adapt [1-9]",
        ),
        negative: r"(?:ki|suspect|void|finality|time|lore|age|stun|oil|charge|quest|loyalty|level) counters?",
        certain: false,
        id: "counters",
    },
    Rule {
        role: "extra-phase",
        pattern: r"(?:extra|additional) (?:turn|combat|beginning phase)|additional combat phase",
        negative: r"(?:can't|during|skip).*?extra|end the turn",
        certain: false,
        id: "extra-phase",
    },
    Rule {
        role: "evasion",
        pattern: concat!(
            r"(?:gains?|has|have).*?(?:flying|menace|fear|shadow|double strike)|(?:creature|",
            r"creatures).*?can't be blocked",
        ),
        negative: r"",
        certain: false,
        id: "grant-evasion",
    },
    Rule {
        role: "stax",
        pattern: concat!(
            r"(?:opponents?|players|creatures|spells).*?(?:can't|cost).*?(?:cast|draw|attack|pay|",
            r"life)|opponents.*?cost .*more|unless that player pays",
        ),
        negative: r"",
        certain: false,
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
}
impl Classifier {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            rules: RULES.iter().map(|r| Ok((regex(r.pattern)?, regex(r.negative)?))).collect::<Result<_, String>>()?,
            ignore: IGNORE.iter().map(|p| regex(p)).collect::<Result<_, _>>()?,
        })
    }

    pub fn classify(&self, card: &Card) -> Result<Assignment, String> {
        let lines = card.lines();
        let mut found: BTreeMap<&str, (usize, Evidence)> = BTreeMap::new();
        let mut unknown = false;
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
                if (!r.negative.is_empty() && hit(negative, &s)?) || !accept(r, card, &s, typ, intrinsic_fixing) {
                    continue;
                }
                let priority = prominence(r.role, &s, line, typ);
                let e =
                    Evidence { source: "rule".into(), certain: r.certain, rule: r.id.into(), face, line: Some(line) };
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
        ranked.truncate(4);
        let roles: Vec<String> = ranked.iter().map(|(r, _)| r.to_string()).collect();
        let needs_llm = unknown || (!roles.is_empty() && roles.iter().all(|r| role_src(r) != "rules"));
        Ok(Assignment {
            primary: roles.first().cloned().unwrap_or_default(),
            roles,
            evidence: ranked.into_iter().map(|(r, (_, e))| (r.into(), e)).collect(),
            needs_llm,
        })
    }
}

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
            "equipped creature",
            "enchanted creature",
            "creatures you control",
            "permanents you control",
            "both creatures",
            "other creatures",
            "you have protection",
            "another target",
        ],
    );
    match r.role {
        "ramp" => {
            if r.id == "land-ramp" {
                return !land && !has(s, &["its controller", "their library", "that player"]);
            }
            if s.contains("less") {
                return !has(
                    s,
                    &["this spell costs", "~ costs", "this ability costs", "equip abilities", "ability costs"],
                );
            }
            if land {
                return s.contains("add")
                    && (s.contains("sacrifice a creature")
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
                        &["any color", "any one color", "any combination of colors", "different colors", "each color"],
                    ) || ['w', 'u', 'b', 'r', 'g']
                        .iter()
                        .filter(|ch| s.split("add").nth(1).unwrap_or("").contains(&format!("{{{ch}}}")))
                        .count()
                        >= 2)
        }
        "tutor" => {
            let q = s.split("search your library").nth(1).unwrap_or("").split([',', '.']).next().unwrap_or("");
            !has(q, &["land card", "land cards", "plains", "island", "swamp", "mountain", "forest"])
                || q.contains("nonland")
        }
        "counter" => true,
        "wipe" => {
            !has(
                s,
                &[
                    "exile all cards",
                    "exile all graveyards",
                    "exile each graveyard",
                    "return all cards",
                    "sacrifice all",
                    "exile all spells",
                ],
            ) && (s.contains("overload") || !s.contains("damage") || !has(s, &["1 damage", "combat damage"]))
                && !(s.contains("exile") && s.contains("return") && s.contains("you control"))
        }
        "removal" => {
            let s = s.split(". this ability costs").next().unwrap_or(s);
            if s.contains("exile") && has(s, &["graveyard", "exile this", "exile ~"]) && !s.contains("exile target") {
                return false;
            }
            if has(s, &["graveyard", "your hand", "target spell", "target spells", "target card"]) {
                return false;
            }
            if s.contains("you control")
                && !has(s, &["don't control", "doesn't control", "fights", "damage to", "each opponent"])
            {
                return false;
            }
            if s.contains("return") && s.contains("battlefield") {
                return false;
            }
            if s.contains("damage") && s.contains("combat damage") {
                return false;
            }
            !s.contains("gain control of this") && !s.contains("gain control of ~")
        }
        "blink" => !s.contains("owner's hand") && !s.contains("your hand"),
        "protection" => {
            (grant || s.contains("phase") || s.contains("prevent all"))
                && !(s.contains("attacking") && s.contains("unless"))
        }
        "draw" => {
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
                    ],
                ) {
                    return false;
                }
                if has(
                    s,
                    &[
                        "discard a card, then draw a card",
                        "draw a card, then discard a card",
                        "draw two cards, then discard two",
                    ],
                ) {
                    return false;
                }
                if has(s, &["draw a card", "draws a card"])
                    && !has(s, &["for each", "equal to"])
                    && (spell || self_sac)
                    && !c.text.to_lowercase().contains("choose the same mode more than once")
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
            has(
                s,
                &[
                    "top of your library",
                    "cast the exiled",
                    "play the exiled",
                    "cast cards exiled",
                    "play cards exiled",
                ],
            ) && has(s, &["may play", "may cast", "may plot"])
                || !spell
                    && !s.contains("search your library")
                    && !s.contains("graveyard")
                    && s.contains("into your hand")
                || s.contains("pile into your hand")
        }
        "selection" => {
            !(s.contains("look at the top") && !has(s, &["put", "shuffle", "discard", "reveal"]))
                && !s.contains("repeat this process")
                && !s.contains("search your library")
                && !has(s, &["draw two cards, then discard a card", "draw three cards, then discard a card"])
                && (!s.contains("reveal the top") || s.contains("into your hand"))
        }
        "discard" => !(s.contains("draw") && has(s, &["each player", "players discard"])),
        "gy-hate" => {
            !s.contains("from anywhere")
                && !has(
                    s,
                    &[
                        "exile it from your graveyard",
                        "exile ~ from your graveyard",
                        "exile this card",
                        "exile a card from your graveyard",
                        "exile cards from your graveyard",
                    ],
                )
        }
        "recursion" => {
            !s.contains("this card from your graveyard")
                && !s.contains("~ from your graveyard")
                && !s.contains("return this")
                && !s.contains("return ~")
                && !s.contains("from anywhere")
        }
        "reanimate" => {
            !has(s, &["return this", "return ~", "exile", "from anywhere"])
                && (s.contains("creature") || s.contains("permanent") || s.contains("if it's a creature"))
        }
        "self-mill" => {
            if s.starts_with("discard a card:") {
                return true;
            }
            if s.contains("mill") {
                return !has(s, &["target player mills", "opponent mills", "opponents mill", "each player mills"]);
            }
            !has(
                s,
                &[
                    "opponent separates",
                    "an opponent separates",
                    "opponent chooses",
                    "from anywhere",
                    "dies",
                    "sacrifice",
                ],
            ) && (s.contains("search your library") || s.contains("your library") || s.contains("from among"))
        }
        "copy" => !s.contains("a copy of ~") && !s.contains("copy of this"),
        "tokens" => {
            !has(s, &["its controller creates", "that player creates", "opponent creates"])
                && (!s.contains("token") || has(s, &["creature", "treasure", "clue", "copy", "investigate", "amass"]))
        }
        "anthem" => {
            // Haste alone is a timing aid. Protection-only grants and self buffs are not team combat roles.
            let team =
                s.contains("s you control") || s.split_whitespace().any(|w| w == "other") && s.contains("creatures");
            team && has(
                s,
                &[
                    "get +",
                    "get an additional",
                    "double strike",
                    "flying",
                    "menace",
                    "trample",
                    "vigilance",
                    "first strike",
                    "deathtouch",
                ],
            ) && !s.contains("whenever a")
        }
        "sac-outlet" => activation && !self_sac,
        "drain" => {
            !has(s, &["combat damage", "damage to any target", "damage to target creature", "damage to you"])
                && (repeat || s.contains("each opponent"))
        }
        "lifegain" => {
            if s.contains("lifelink") {
                return !spell
                    && (!s.contains("until end of turn") || activation)
                    && (grant || !has(s, &["gain", "has", "have"]) && s.split(", ").any(|w| w == "lifelink"));
            }
            if !has(
                s,
                &[
                    "you gain",
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
            !has(s, &["enters with", "enters the battlefield with", "put ~'s counters", "put this creature's counters"])
                && (has(s, &["+1/+1", "-1/-1", "proliferate", "backup", "bolster", "support", "adapt"]) || repeat)
        }
        "extra-phase" | "finisher" => true,
        "evasion" => grant && !s.contains("can't attack"),
        "stax" => {
            !(s.contains("cost") && s.contains("less"))
                && !s.contains("counter target")
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
    for c in &index.cards {
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
}
fn options(args: &[String]) -> Result<Options, String> {
    let mut o = Options::default();
    let mut i = 0;
    while i < args.len() {
        let a = args[i].as_str();
        i += 1;
        match a {
            "--rules" | "--validate" | "--stats" | "--show" => {
                if !o.mode.is_empty() {
                    return Err("roles: choose exactly one of --rules, --validate, --stats, --show NAME".into());
                }
                o.mode = a.into();
                if a == "--show" {
                    let v = args.get(i).filter(|s| !s.starts_with("--")).ok_or("--show needs a name")?;
                    o.name = Some(v.clone());
                    i += 1;
                }
            }
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
    if o.mode.is_empty() || o.gold.is_some() && o.mode != "--validate" || o.json && o.mode != "--validate" {
        return Err("roles: choose --rules, --show NAME, --stats, or --validate [--gold PATH] [--json]".into());
    }
    Ok(o)
}

/// Text and success are separate so a failing validation still prints its entire report before exiting nonzero.
pub fn command(project: &Project, args: &[String]) -> Result<(String, bool), String> {
    let o = options(args)?;
    let dir = knowledge::knowledge_dir(project, o.dir.as_deref())?;
    let mut index = Index::load(&dir.join("oracle.json"))?;
    let taxonomy = Taxonomy::parse(&read(&project.data("knowledge/roles.toml"))?)?;
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
    pub needs_llm: usize,
    pub no_role: usize,
    pub per_role: BTreeMap<String, usize>,
    pub routing: BTreeMap<String, (usize, usize)>,
}
pub fn stats(index: &Index, map: &RoleMap) -> Stats {
    let mut s = Stats { per_role: IDS.iter().map(|r| (r.to_string(), 0)).collect(), ..Stats::default() };
    for c in &index.cards {
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
        let a = &rules[&index.cards[i as usize].raw.oracle_id];
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
