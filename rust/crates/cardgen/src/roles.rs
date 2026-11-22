//! Deterministic deck-building roles, deliberately separate from executable engine tags.
//! Rules read normalized Oracle lines and printed stats, never card names or deck membership.
use crate::data::Project;
use crate::knowledge::{self, Card, Index, Legal};
use fancy_regex::{Regex, RegexBuilder};
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
/// Source category and unconditional routing share one policy. Removal additionally routes when it is
/// the only domain present or its evidence joins clauses; the rules-only hints below still need review.
pub const ROLE_POLICY: &[(&str, &str, bool)] = &[
    ("selection", "rules", true),
    ("recursion", "rules", true),
    ("gy-hate", "rules", true),
    ("finisher", "llm", true),
    ("stax", "llm", true),
    ("drain", "rules+llm", true),
    ("copy", "rules+llm", true),
    ("evasion", "rules+llm", true),
    ("protection", "rules+llm", true),
    ("removal", "rules+llm", false),
];
pub fn role_src(id: &str) -> &'static str {
    ROLE_POLICY.iter().find(|(role, _, _)| *role == id).map_or("rules", |(_, source, _)| *source)
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
    #[serde(default)]
    pub route: Vec<RouteReason>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RouteReason {
    UnknownLine,
    NoRole,
    RejectedHit,
    SameLine,
    RiskyRole,
    Overflow,
    RegexError,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoleMeta {
    pub taxonomy: String,
    pub rules_fnv: String,
    pub oracle_fnv: String,
    pub count: usize,
    /// FNV of every byte after the metadata object; lets a metadata-only reader detect a damaged table.
    pub table_fnv: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RoleMap {
    pub meta: RoleMeta,
    pub cards: BTreeMap<String, Assignment>,
}
impl std::ops::Deref for RoleMap {
    type Target = BTreeMap<String, Assignment>;
    fn deref(&self) -> &Self::Target {
        &self.cards
    }
}
impl std::ops::DerefMut for RoleMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.cards
    }
}

pub fn fingerprint(bytes: &[u8]) -> String {
    format!("{:016x}", crate::bulk::fnv1a64(bytes))
}
pub fn rules_hash(overrides: &[u8]) -> String {
    // Include line extraction/normalization ownership as well as runtime override content. Comments and blank
    // lines in the overrides file carry no content, so they do not change the hash.
    let mut bytes = include_bytes!("roles.rs").to_vec();
    bytes.extend_from_slice(include_bytes!("knowledge.rs"));
    for line in
        String::from_utf8_lossy(overrides).lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        bytes.extend_from_slice(line.as_bytes());
        bytes.push(b'\n');
    }
    fingerprint(&bytes)
}

/// Canonical bulk rendering gives the same FNV as sync.lock, independent of input order/JSON whitespace.
/// Rebuild key order too, since JSON consumers may serialize objects in a different order.
pub fn oracle_fingerprint(path: &Path) -> Result<String, String> {
    use crate::json::{self, J, Obj};
    let doc = json::parse(&read(path)?)?;
    let cards = doc.get("cards").and_then(J::as_arr).ok_or("oracle.json: missing cards")?;
    let records = cards
        .iter()
        .map(|card| {
            let slim = crate::bulk::slim_bulk(card);
            let mut ordered = Obj::new();
            for key in [
                "name",
                "mana_cost",
                "cmc",
                "type_line",
                "oracle_text",
                "power",
                "toughness",
                "loyalty",
                "colors",
                "color_identity",
                "keywords",
                "produced_mana",
                "layout",
                "game_changer",
                "legalities",
                "card_faces",
                "oracle_id",
                "edhrec_rank",
                "usd",
                "released_at",
                "set",
                "rarity",
                "reserved",
                "all_parts_meld",
                "meld_parts",
            ] {
                if let Some(value) = slim.get(key).or_else(|| card.get(key)) {
                    ordered.set(key, value.clone());
                }
            }
            (card.s_or("name").to_lowercase(), J::Obj(ordered))
        })
        .collect();
    let meta = crate::bulk::Meta {
        bulk_updated_at: String::new(),
        download_uri: String::new(),
        synced: String::new(),
        count: cards.len(),
    };
    Ok(format!("{:016x}", crate::bulk::render(&meta, records).1))
}

impl RoleMeta {
    pub fn check(&self, index: &Index) -> Result<(), String> {
        if self.taxonomy != index.taxonomy_fnv
            || self.rules_fnv != index.rules_fnv
            || self.oracle_fnv != index.oracle_fnv
            || self.count != index.cards.iter().filter(|c| is_deck_card(c)).count()
        {
            return Err(
                "roles.json: stale taxonomy, rules, Oracle fingerprint or count; run `cardgen roles --rules`".into()
            );
        }
        Ok(())
    }
}
impl RoleMap {
    pub fn check_meta(&self, index: &Index) -> Result<(), String> {
        self.meta.check(index)?;
        if self.meta.count != self.len() {
            return Err("roles.json: stale count; run `cardgen roles --rules`".into());
        }
        Ok(())
    }
}

/// Parse the leading metadata object emitted by bytes(); returns it with the offset where the card table begins.
fn parse_meta(bytes: &[u8]) -> Result<(RoleMeta, usize), String> {
    let (mut depth, mut quoted, mut escaped) = (0usize, false, false);
    for (at, &byte) in bytes.iter().enumerate() {
        if quoted {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                quoted = false;
            }
            continue;
        }
        match byte {
            b'"' => quoted = true,
            b'{' => depth += 1,
            b'}' if depth == 2 => {
                let mut prefix = bytes[..=at].to_vec();
                prefix.push(b'}');
                #[derive(Deserialize)]
                #[serde(deny_unknown_fields)]
                struct Header {
                    meta: RoleMeta,
                }
                return serde_json::from_slice::<Header>(&prefix)
                    .map(|header| (header.meta, at + 1))
                    .map_err(|e| format!("roles.json metadata: {e}; run `cardgen roles --rules`"));
            }
            b'}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    Err("roles.json: missing leading metadata; run `cardgen roles --rules`".into())
}

/// Read only the leading metadata object, bounded independently of the card table.
pub fn load_meta(path: &Path) -> Result<RoleMeta, String> {
    use std::io::Read;
    let file =
        std::fs::File::open(path).map_err(|e| format!("{}: {e}; run `cardgen roles --rules`", path.display()))?;
    let mut prefix = Vec::new();
    std::io::BufReader::new(file)
        .take(65536)
        .read_to_end(&mut prefix)
        .map_err(|e| format!("roles.json: {e}; run `cardgen roles --rules`"))?;
    parse_meta(&prefix).map(|(meta, _)| meta)
}

/// Staleness and damage check for optional consumers: the metadata is compared with the catalog and the card table
/// bytes with `meta.table_fnv`, so truncation or edits are noticed without parsing the table.
pub fn check_file(path: &Path, index: &Index) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{}: {e}; run `cardgen roles --rules`", path.display()))?;
    let (meta, end) = parse_meta(&bytes)?;
    meta.check(index)?;
    if fingerprint(&bytes[end..]) != meta.table_fnv {
        return Err("roles.json: damaged card table; run `cardgen roles --rules`".into());
    }
    Ok(())
}

// Shared grammatical vocabularies. Quantities are not an ad-hoc subset of number words.
const NUMBER: &str = concat!(
    r"(?:a|an|one|two|three|four|five|six|seven|eight|nine|ten|eleven|twelve|thirteen|fourteen|fifteen|",
    r"sixteen|seventeen|eighteen|nineteen|twenty|thirty|forty|fifty|sixty|seventy|eighty|ninety|",
    r"hundred|thousand|million|[0-9]+|x|that many|that much|cards? equal to[^.]*)",
);
const SUBJECT: &str = concat!(
    r"(?:each other player|each opponent|target opponent|an opponent|opponents?|target player|",
    r"defending player|that opponent|that player|its controller|their controller|each player|players|they|",
    r"(?:target|that) creature.s controller)",
);
const TEAM: &str = concat!(
    r"(?:(?:each (?:other )?[^.,:]+|[^.,:]+s|other [a-z]+) you control|other creatures|creatures with [a-z]+|",
    r"all [a-z]+s)",
);
// Compile each fixed semantic expression once, without a shared mutable cache.
macro_rules! grammar {
    ($pattern:expr, $text:expr $(,)?) => {{
        static RE: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        RE.get_or_init(|| regex($pattern).expect("fixed grammar")).is_match($text).unwrap_or(false)
    }};
}
fn vocabulary(pattern: &str) -> String {
    pattern.replace("SUBJECT", SUBJECT).replace("NUMBER", NUMBER).replace("TEAM", TEAM)
}

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
            r"(?:spell|spells)[^.]*(?:has|have) affinity|create[^.]*treasure token",
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
            r"(?:destroy|exile|return) (?:another |up to NUMBER (?:other )?|NUMBER (?:other )?)?target|",
            r"return (?:~|this creature) and another target|",
            r"(?:destroy|exile) (?:it|that (?:creature|permanent))|put (?:up to NUMBER )?target[^.]*owner.s library|",
            r"SUBJECT (?:sacrifices|exiles) NUMBER|target creature.s controller sacrifices it|",
            r"you control enchanted (?:creature|permanent)|",
            r"put (?:a|one) -1/-1 counter on (?:up to one )?target|",
            r"target.*(?:gets? -[0-9x]+/-[1-9x][0-9]*|sacrifices? (?:a|an|one)|fights)|",
            concat!(
                r"deals? [^.]*damage (?:equal to [^.]*? )?(?:to (?:any target|a creature an opponent co",
                r"ntrols|target (?!player)[^.]*?(?:creature|artifact|planeswalker)|",
            ),
            r"each of up to [a-z]+ target creatures)|divided)|",
            r"gain control of|exchange control of target|owner of target .*shuffles|",
            r"enchanted .*?(?:can't attack|doesn't untap|loses all|sacrifice this creature)|",
            r"target permanent.*doesn't untap|tap[^.]*target[^.]*\. (?:it|that [a-z]+) doesn't untap|",
            r"chosen creatures fight|\bannihilator [1-9]|",
            r"\btap (?:it|enchanted creature)|it gets -[0-9x]+/-[1-9x]",
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
            r"\bdraws?\b|you become the monarch|(?:play|cast|plot) .*?(?:top of your library|exiled)|(?:put|",
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
            r"\b(?:scry|surveil) (?:[2-9]|[1-9][0-9]+|x)\b|draws? .*?discards?|discards? .*?draws?|",
            r"look at the top|from among them into your hand|reveal the top|",
            r"discards any number of cards, then draws|\b(?:connives?|recruit|learn)\b|create[^.]*blood token",
        ),
        negative: r"",
        id: "filter",
    },
    Rule {
        role: "discard",
        pattern: concat!(
            r"SUBJECT(?: or [^.]*?controller)? discards?\b|",
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
            r"\b(?:cast|play)\b[^.,]*?from your graveyard|",
            r"your graveyard[^.]*?(?:\bcast\b|\bplay\b|gains flashback|has escape|into your hand)|\bsoulshift\b|",
            r"in your graveyard.*?otherwise, put it into your hand|",
            r"(?:this card|~) is in your graveyard.*?return (?:this card|~) to your hand|",
            r"choose target [^.]*card in your graveyard.*?return it to your hand|",
            r"(?:return|put)[^.]*graveyard[^.]*battlefield|graveyard[^.]*shuffle [^.]*into your library|",
            r"graveyard[^.]*put it into your library|permanent card in your graveyard.*?battlefield|",
            r"return all [^.]*cards in graveyards[^.]*hands|",
            r"exile[^.]*card from your graveyard.*?you may cast that card",
        ),
        negative: r"",
        id: "graveyard-reuse",
    },
    Rule {
        role: "reanimate",
        pattern: concat!(
            r"(?:return|put).*?(?:graveyard|creature card).*?battlefield|",
            r"(?:creature|permanent) card in your graveyard.*?put it onto the battlefield|",
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
            r"TEAM (?:gets?|have|has|gains?)|",
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
            r"SUBJECT[^.]*?loses? (?!the game)[^.]*?life|",
            r"deals? [^.]*?damage to (?:SUBJECT|you|the chosen player)|",
            r"deals damage equal [^.]*to the chosen player",
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
            r"\b(?:put|double|move|distribute)\b.*?counters?|\bproliferate\b|\bbackup [1-9]|\bbolster [1-9]|",
            r"\bsupport [1-9]|\badapt [1-9]|\bawaken [1-9]|\b(?:connives?|dethrone|mentor|evolve|training)\b|",
            r"\bamass\b|create[^.]*map token",
        ),
        negative: r"",
        id: "counters",
    },
    Rule {
        role: "extra-phase",
        pattern: r"(?:extra|additional) (?:turn|combat|beginning phase)",
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
            r"(?:SUBJECT|creatures|spells)[^.,]*?(?:can't|cost)[^.,]*?(?:cast|draw|attack|pay|",
            r"life|untap)|opponents[^.]*?cost .*more|unless (?:that player|they) pay|",
            r"creatures your opponents control enter tapped|enchanted creature[^.]*can't attack you|",
            r"if an opponent[^.]*would put a land[^.]*then sacrifices a land|controller can't cast spells|",
            r"players can't search libraries|(?:nonbasic lands|lands|creatures) ",
            r"(?:enter tapped|don't untap|can't untap|are mountains)|",
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
    RegexBuilder::new(&format!("(?i){}", vocabulary(p)))
        .backtrack_limit(10_000_000)
        .build()
        .map_err(|e| format!("role regex {p}: {e}"))
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
    // Basic land types supply this intrinsic mana synopsis; it is evidence, unlike keyword reminders.
    if let Some(synopsis) = line.strip_prefix('(').and_then(|s| s.strip_suffix(')')) {
        let face = card.lines().into_iter().find(|(_, text)| *text == line).and_then(|(face, _)| face);
        let typ = face
            .and_then(|i| card.raw.card_faces.as_ref()?.get(i))
            .map_or(card.raw.type_line.as_str(), |face| face.type_line.as_str());
        if synopsis.starts_with("{T}: Add ")
            && typ.contains("Land")
            && ["Plains", "Island", "Swamp", "Mountain", "Forest"].iter().filter(|t| typ.contains(**t)).count() >= 2
        {
            return synopsis.to_string();
        }
    }
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
            // A verb-named spell can begin an imperative sentence with its name's spelling.
            let sentence_start = out[..i].trim_end().ends_with('.') || out[..i].trim().is_empty();
            let imperative = ["target", "all", "each", "up to"].iter().any(|w| out[j..].trim_start().starts_with(w));
            if sentence_start && imperative && !name.contains(' ') {
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

// Well above the catalog maximum (645 bytes); bound malformed/future input before backtracking.
const MAX_LINE_BYTES: usize = 2048;

/// A colon is an activation separator only outside a quoted granted/token ability.
fn activation_cost(s: &str) -> Option<(&str, &str)> {
    let colon = s.find(':')?;
    if s.find('"').is_some_and(|quote| quote < colon) {
        return None;
    }
    Some((&s[..colon], &s[colon + 1..]))
}
fn effect_text(s: &str) -> &str {
    let s = activation_cost(s).map_or(s, |(_, effect)| effect).trim();
    let trigger = s.find("whenever ").or_else(|| s.find("when ")).or_else(|| s.find("at the beginning "));
    if let Some(trigger) = trigger.filter(|i| {
        let prefix = s[..*i].trim();
        (prefix.is_empty() || prefix.ends_with('—')) && !prefix.contains('"')
    }) {
        if let Some(comma) = s[trigger..].find(", ") {
            return &s[trigger + comma + 2..];
        }
    }
    s
}

/// Named INCIDENTAL conventions, only for a rejected role (not an arbitrary unparsed effect).
/// scry-one; cantrip/loot; self-sacrifice-cost; ward/self-keyword; enters-with-counter;
/// one-turn-lifelink; opponent-compensation; alternate-role (land search/mana/zone); cost-only;
/// self-only-buff; non-role artifact token; player/bookkeeping-counter; self-downside.
fn incidental(role: &str, _card: &Card, full: &str, typ: &str, accepted: &BTreeSet<&str>) -> Option<&'static str> {
    let effect = effect_text(full);
    let spell = has(typ, &["Instant", "Sorcery"]);
    let cost = activation_cost(full).map_or("", |(cost, _)| cost);
    let self_cost = has(cost, &["sacrifice ~", "sacrifice this", "exile ~", "exile this", "discard this"]);
    match role {
        "finisher" if full.split(", ").any(|keyword| keyword.trim() == "infect") => {
            Some("printed infect below bomb stats")
        }
        "drain"
            if has(
                effect,
                &["combat damage", "damage to any target", "damage to target creature", "damage to you"],
            ) =>
        {
            Some("combat/creature/self damage")
        }
        "drain" if spell && !has(effect, &["each opponent", "each player"]) => Some("single-player one-shot burn"),
        "drain" if !has(effect, &["lose", "damage"]) => Some("life-loss condition"),
        "evasion" | "protection" | "anthem"
            if ["this creature", "~", "it "].iter().any(|subject| effect.starts_with(subject)) =>
        {
            Some("self-only keyword/buff")
        }
        "anthem"
            if effect.split(". ").all(|sentence| {
                grammar!(
                    concat!(
                        r"^(?:(?:up to NUMBER )?target creatures?(?: you control)?|(?:enchanted|equipped) creature|",
                        r"that creature|(?:all|each) creatures?) |^(?:other )?creatures you control (?:have|gain) ",
                        r"haste(?: until end of turn)?\.?$|^(?:you|each opponent|target (?:player|opponent)) "
                    ),
                    sentence,
                )
            }) =>
        {
            Some("single-subject, haste-only, symmetric or non-creature effect")
        }
        "ramp" | "fixing" if typ.contains("Land") && single_mana_ability(full) => Some("land tapping for one mana"),
        "draw" if (has(typ, &["Aura", "Equipment"]) || !spell) && grammar!(SELF_DEATH_DRAW, full) => {
            Some("self-replacing death draw")
        }
        "draw" if grammar!(r"^search your library for (?:a|up to [a-z]+) basic land cards?", effect) => {
            Some("land search to hand")
        }
        "removal" if grammar!(r"^(?:when|whenever) this aura enters, tap enchanted creature\.?$", full) => {
            Some("bare tapping")
        }
        "removal"
            if grammar!(
                concat!(
                    r"^if (?:~|this [a-z]+) would be put into (?:a|your|its owner's) graveyard from anywhere, ",
                    r"exile it instead\.?$"
                ),
                full
            ) =>
        {
            Some("self graveyard replacement")
        }
        "recursion"
            if grammar!(
                concat!(
                    r"^when this (?:aura|equipment) is put into a graveyard from the battlefield, return it ",
                    r"to its owner's hand\.?$"
                ),
                full
            ) =>
        {
            Some("self-returning aura")
        }
        "stax" if accepted.contains("ramp") && grammar!(r"cost (?:\{[^}]+\})+ less to cast", effect) => {
            Some("cost reducer")
        }
        "gy-hate" if grammar!(r"^(?:as an additional cost to cast this spell, exile |escape—[^,]*, exile )", full) => {
            Some("graveyard exile as a cost")
        }
        "lifegain" if full.contains("token") && full.contains("lifelink") => Some("token's printed lifelink"),
        "lifegain" if full.contains("token") && full.contains("\"when this") => Some("token death life rider"),
        "recursion" | "reanimate"
            if !has(
                effect,
                &[
                    "from your graveyard",
                    "in your graveyard",
                    "from a graveyard",
                    "from their graveyard",
                    "from the graveyard",
                    "card in",
                    "cards in",
                    "soulshift",
                    "shuffle your graveyard",
                ],
            ) && !has(effect, &["return", "put", "cast", "play"]) =>
        {
            Some("graveyard event without reuse")
        }
        "counters"
            if !activation_cost(full).is_some()
                && !has(full, &["whenever", "at the beginning"])
                && has(effect, &["on this creature", "on ~"])
                && !has(effect, &["on target", "on each", "instead"]) =>
        {
            Some("one-time self counter")
        }
        "removal" | "wipe"
            if has(effect, &["graveyard", "your hand"])
                && accepted.iter().any(|r| ["recursion", "reanimate", "gy-hate"].contains(r))
                && !has(effect, &["sacrifices all creatures", "destroy target", "damage to target"]) =>
        {
            Some("non-battlefield zone")
        }
        "removal" if grammar!(r"gain control[^.]*until end of turn", effect) => Some("temporary steal"),
        "removal"
            if has(effect, &["return", "exile"])
                && effect.contains("battlefield")
                && accepted.iter().any(|r| ["recursion", "reanimate", "blink"].contains(r)) =>
        {
            Some("return/blink zone")
        }
        "recursion"
            if battlefield_object(effect).is_some_and(|object| {
                object.contains("creature")
                    && !object.contains("permanent card")
                    && !has(object, &["artifact", "enchantment"])
            }) =>
        {
            Some("creature battlefield return")
        }
        "reanimate" if !effect.contains("battlefield") || has(effect, &["return this", "return ~"]) => {
            Some("non-reanimation zone/self return")
        }
        "draw" if full.contains("can't draw") || full.contains("skip") && full.contains("draw step") => {
            Some("draw restriction/downside")
        }
        "lifegain" if full.contains("can't gain life") => Some("life restriction"),
        "lifegain" if spell && has(effect, &["damage", "destroy", "exile", "counter", "put", "draw"]) => {
            Some("spell life rider")
        }
        "lifegain" if creates(full, "food") => Some("one-shot Food"),
        "ramp" if creates(full, "treasure") => Some("single Treasure rider"),
        "ramp" if full.contains("this spell costs") || has(full, &["equip abilities", "this ability costs"]) => {
            Some("self/equip-only discount")
        }
        "selection" if full.contains("look at the top card") && !has(effect, &["put", "shuffle"]) => {
            Some("inspection without filtering")
        }
        "fixing" if grammar!(r"^\{t\}: add \{[wubrgc]\}\.$", full) => Some("one-colour tap mana"),
        "fixing" if accepted.contains("ramp") && grammar!(r"^add (\{[wubrgc]\})(?:\1)*\.$", effect) => {
            Some("accepted ramp with one repeated mana symbol")
        }
        "fixing" if full.contains("search your library") => Some("non-fixing library search"),
        "tutor" if has(full, &["land card", "plains", "island", "swamp", "mountain", "forest"]) => {
            Some("land-only search")
        }
        "ramp" if typ.contains("Land") && grammar!(r"^\{t\}: add \{[wubrgc]\}\.$", full) => Some("one-mana land tap"),
        "ramp" if mana_filter(full) && accepted.contains("fixing") => Some("net-zero mana filter"),
        "draw"
            if (full.contains("search your library") || full.contains("graveyard"))
                && accepted.iter().any(|r| ["tutor", "recursion", "reanimate", "self-mill"].contains(r)) =>
        {
            Some("zone/search role")
        }
        "draw"
            if loot(full) && accepted.contains("selection") || (spell || self_cost) && full.contains("draw a card") =>
        {
            Some("loot/cantrip")
        }
        "draw" if full.contains("would draw") => Some("replacement condition"),
        "draw" if !effect.contains("draw") && full.contains("draw") => Some("trigger condition"),
        "draw" if creates(full, "clue") || full.contains("investigate") => Some("single Clue"),
        "selection" if accepted.contains("draw") || full.contains("draw") && !loot(full) => Some("net draw"),
        "selection" if has(full, &["scry 1", "surveil 1"]) => Some("scry-one"),
        "self-mill"
            if has(full, &["sacrifice", "dies", "from anywhere", "discard your hand"])
                || full.contains("discard") && !full.starts_with("discard a card:") =>
        {
            Some("cost/zone change")
        }
        "sac-outlet" if self_cost => Some("self-sacrifice cost"),
        "counters" if !effect.contains("counter") && !effect.contains("proliferate") => Some("cost-only counter"),
        "counters"
            if has(full, &["enters with", "enters the battlefield with", " with a ", " with an "])
                || !has(effect, &["+1/+1", "-1/-1", "target", "on each", "on another"]) =>
        {
            Some("enters/player/bookkeeping counter")
        }
        "lifegain" if full.contains("lifelink") && spell || self_cost => Some("one-turn/self-sacrifice life"),
        "lifegain" if full.contains("until end of turn") && activation_cost(full).is_none() => Some("one-turn life"),
        "lifegain" if has(full, &["its controller gains", "that player gains"]) => Some("opponent compensation"),
        "tokens" if has(full, &["its controller creates", "that player creates", "they create"]) => {
            Some("opponent compensation")
        }
        "tokens" if ["food", "blood", "map", "junk", "powerstone"].iter().any(|k| creates(full, k)) => {
            Some("non-role artifact token")
        }
        "removal" | "wipe"
            if has(full, &["sacrifice this", "sacrifice ~", "return this", "return ~"])
                && !has(effect, &["destroy target", "exile target", "all creatures"]) =>
        {
            Some("self cost/downside")
        }
        "anthem" | "evasion" | "protection"
            if full.starts_with("this creature")
                || full.starts_with('~')
                || full.starts_with("target creature gets +") =>
        {
            Some("self/single pump")
        }
        "anthem"
            if has(full, &["indestructible", "hexproof", "ward", "lifelink"])
                && !has(full, &["get +", "gets +", "flying", "trample"]) =>
        {
            Some("protection/life-only team")
        }
        _ => None,
    }
}

/// Equal draws/discards are filtering, including third-person and conditional two-sentence loot.
fn loot(s: &str) -> bool {
    let s = s.replace("draws ", "draw ").replace("discards ", "discard ");
    if grammar!(r"draws? (?:a card for each|NUMBER cards? (?:for each|equal to))", &s) {
        return false;
    }
    if grammar!(r"(?:discard|shuffle)[^.]*cards?[^.]*draw that many", &s) {
        return true;
    }
    if s.contains("discards any number") || s.contains("discard any number of cards, then draw that many") {
        return true;
    }
    static PAIR: std::sync::OnceLock<(Regex, Regex)> = std::sync::OnceLock::new();
    let (draw, discard) = PAIR
        .get_or_init(|| (regex(r"\bdraws? (NUMBER) cards?").unwrap(), regex(r"\bdiscards? (NUMBER) cards?").unwrap()));
    let amount = |re: &Regex| {
        re.captures(&s).ok().flatten().and_then(|captures| {
            captures.get(1).map(|m| match m.as_str() {
                "a" | "an" | "one" | "1" => "one".to_string(),
                other => other.to_string(),
            })
        })
    };
    amount(draw).zip(amount(discard)).is_some_and(|(draw, discard)| draw == discard)
}

const SELF_DEATH_DRAW: &str = concat!(
    r"(?:when|if) (?:enchanted creature|equipped creature|this [a-z]+|~) ",
    r"(?:dies|is put into a graveyard from the battlefield)[^.]*draw (?:a|one) card",
);

/// A land ability that adds exactly one mana, whatever the options: a filter or painland tap, not net ramp.
fn single_mana_ability(s: &str) -> bool {
    activation_cost(s).is_some_and(|(_, effect)| {
        grammar!(
            concat!(
                r"^add (?:one mana of (?:any (?:one )?color|the chosen color|any type)|\{[wubrgc]\}",
                r"(?:(?:, | or |, or )",
                r"\{[wubrgc]\})*(?: or one mana of the chosen color)?)\.(?: this land (?:deals [0-9]+ damage to you|",
                r"doesn't untap during your next untap step)\.)?$"
            ),
            effect.trim(),
        )
    })
}

fn pure_keyword_line(s: &str) -> bool {
    s.split(", ").all(|keyword| {
        grammar!(
            concat!(
                r"^(?:flying|first strike|double strike|deathtouch|defender|flash|haste|hexproof|indest",
                r"ructible|lifelink|menace|reach|shroud|trample|vigilance|fear|shadow|infect|wither|hor",
                r"semanship|(?:plains|island|swamp|mountain|forest)walk|(?:ward|equip|crew|saddle|kicke",
                r"r|multikicker|flashback|cycling|echo|morph|megamorph|disguise|foretell|plot|escape|ev",
                r"oke|buyback|bestow|disturb|mutate|fortify|suspend|madness|aftermath|embalm|eternalize",
                r"|unearth|reinforce|transmute|level up) (?:\{[^}]+\}|[0-9]+|—pay [0-9]+ life)+|enchant",
                r" [a-z ]+|protection from [a-z ]+|(?:renown|modular|graft|fading|vanishing|toxic) [0-9",
                r"]+|convoke|delve|rebound|soulbond|daybound|nightbound|sunburst|[a-z]*cycling (?:\{[^}]+\})+|",
                r"cumulative upkeep (?:\{[^}]+\})+|affinity for [a-z ]+|suspend [0-9]+—(?:\{[^}]+\})+)\.?$",
            ),
            keyword,
        )
    })
}

/// A line made only of keywords, one-mana taps or static boilerplate that carries no deck-building function.
/// Each sentence must match a positive pattern; a missing word never excuses it.
fn plain_line(s: &str, typ: &str) -> bool {
    let spell = has(typ, &["Instant", "Sorcery"]);
    s.split(". ").all(|sentence| {
        let sentence = sentence.trim_end_matches('.');
        pure_keyword_line(sentence)
            || grammar!(r"^\{t\}: add \{[wubrgc]\}$", sentence)
            || grammar!(
                concat!(
                    r"^(?:(?:enchanted|equipped) creature gets [+-][0-9x]+/[+-][0-9x]+|(?:this|~) (?:creature |land |",
                    r"artifact )?enters tapped|as (?:this [a-z]+|~) enters, choose a colou?r|partner|companion — .*|",
                    r"(?:scry|surveil) 1|(?:this creature|~|this spell) (?:can't (?:block|be countered|be blocked",
                    r"(?: by [^.]*)?|attack(?: unless [^.]*)?)|attacks each (?:combat|turn) if able|can block ",
                    r"(?:only [^.]*|an additional creature each combat))|(?:\{[^}]+\}(?:, )?)+: (?:this creature",
                    r"|~) (?:gains [a-z ]+ until end of turn|can't be blocked this turn))$",
                ),
                sentence,
            )
            || spell && sentence == "draw a card"
    })
}

/// Mana amounts are additive symbols or explicit quantities, never inferred from missing words.
fn mana_amount(s: &str) -> Option<u32> {
    if grammar!(r"^(?:\{[0-9wubrgc]+\})+$", s.trim()) {
        return s
            .trim()
            .split('{')
            .skip(1)
            .try_fold(0u32, |n, part| n.checked_add(part.trim_end_matches('}').parse::<u32>().unwrap_or(1)));
    }
    if !grammar!(
        concat!(
            r"^(?:one|two|three|[0-9]+) mana(?: of (?:any (?:one )?color|any type|the chosen color)",
            r"| in any combination of colors)?$"
        ),
        s,
    ) {
        return None;
    }
    let amount = s.split_whitespace().next()?;
    match amount {
        "one" => Some(1),
        "two" => Some(2),
        "three" => Some(3),
        _ => amount.parse().ok(),
    }
}
fn mana_filter(s: &str) -> bool {
    let Some((cost, effect)) = activation_cost(s) else { return false };
    let mana_cost = cost.split(',').next().unwrap_or("").trim();
    let Some(added) = effect.trim().strip_prefix("add ") else { return false };
    let added = added.trim_end_matches('.').split(". ").next().unwrap_or("");
    mana_amount(mana_cost).zip(mana_amount(added)).is_some_and(|(cost, added)| cost > 0 && cost == added)
}
fn draws_for_you(s: &str) -> bool {
    // A third-person subject persists through coordinated verbs ("that player exiles ..., then draws").
    s.split(". ").any(|sentence| {
        if !grammar!(r"\bdraws? (?:NUMBER|an additional|cards|that)\b", sentence) {
            return false;
        }
        if grammar!(r"you (?:may )?draw\b|(?:each player|target player)(?! other than you) (?:may )?draws?\b", sentence)
            || grammar!(r"(?:each player|target player)(?! other than you)[^.]*\bdraws?\b", sentence)
        {
            return true;
        }
        grammar!(r"(?:^|[,;:] |then |and )(?:• )?(?:you (?:may )?)?draw\b", sentence)
            && !grammar!(r"SUBJECT[^.]*\bdraws?\b", sentence)
    })
}
fn keyword_grant(s: &str) -> bool {
    grammar!(
        concat!(
            r"(?:gains?|has|have) (?:(?!\b(?:for each|with)\b).)*?(?:flying|menace|fear|shadow|hors",
            r"emanship|double strike|intimidate|hexproof|shroud|indestructible|protection from|ward",
            r")",
        ),
        s,
    ) || s.contains("can't be blocked")
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
        let mut route = BTreeSet::new();
        let mut substantive = false;
        let mut modal_repeat = false;
        let mut modal_face = None;
        for (line, &(face, original)) in lines.iter().enumerate() {
            let f = face.and_then(|i| card.raw.card_faces.as_ref()?.get(i));
            let typ = f.map_or(card.raw.type_line.as_str(), |f| &f.type_line);
            let land = typ.contains("Land");
            // MDFC land backs do not add functions; prepare bodies' own abilities do.
            if card.raw.layout == "modal_dfc" && face.is_some_and(|i| i > 0) && land {
                continue;
            }
            if original.len() > MAX_LINE_BYTES {
                route.insert(RouteReason::RegexError);
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
            substantive |= !s.is_empty() && !plain_line(&s, typ);
            // Basic land types confer mana abilities even when their only Oracle line is parenthesized reminder text.
            let basic_colors =
                ["Plains", "Island", "Swamp", "Mountain", "Forest"].iter().filter(|t| typ.contains(**t)).count();
            let intrinsic_fixing = land && basic_colors >= 2 && original.starts_with("({T}: Add ");
            let mut accepted = BTreeSet::new();
            let mut matched_any = intrinsic_fixing;
            let mut rejected_roles = BTreeSet::new();
            let mut regex_error = false;
            let mut safe_hit = |pattern: &Regex, text: &str| match hit(pattern, text) {
                Ok(yes) => yes,
                Err(_) => {
                    regex_error = true;
                    false
                }
            };
            for (n, r) in RULES.iter().enumerate() {
                let (positive, negative) = &self.rules[n];
                let matched = (r.id == "fixing" && intrinsic_fixing) || safe_hit(positive, &s);
                if !matched {
                    continue;
                }
                matched_any = true;
                // Effects are judged independently of costs, trigger subjects and unrelated sentences.
                // Linked-zone abilities still need the rest of the line to resolve "it/that card".
                let per_sentence = ["removal", "wipe", "counters", "anthem", "protection"].contains(&r.role);
                let effect =
                    if ["sac-outlet", "self-mill", "recursion", "reanimate", "lifegain", "stax"].contains(&r.role) {
                        contextual.as_str()
                    } else {
                        effect_text(&contextual)
                    };
                let candidates: Vec<&str> = if per_sentence
                    && !(r.role == "removal" && effect.contains("tap ") && effect.contains("doesn't untap"))
                {
                    effect.split(". ").collect()
                } else {
                    vec![effect]
                };
                let yes = candidates.iter().any(|part| {
                    let part = if ["sac-outlet", "self-mill", "recursion", "reanimate", "lifegain", "stax"]
                        .contains(&r.role)
                    {
                        *part
                    } else {
                        effect_text(part)
                    };
                    ((r.id == "fixing" && intrinsic_fixing)
                        || r.role == "selection" && loot(&contextual)
                        || safe_hit(positive, part))
                        && (r.negative.is_empty() || !safe_hit(negative, part))
                        && accept(r, card, part, typ, intrinsic_fixing, &contextual)
                });
                if !yes {
                    rejected_roles.insert(r.role);
                    continue;
                }
                accepted.insert(r.role);
                let priority = prominence(r.role, &s, line, typ);
                let mut certain = false;
                for (role, template) in &self.certain {
                    if *role == r.role {
                        certain |= safe_hit(template, &s);
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
            let ignored = s.split(". ").all(|sentence| self.ignore.iter().any(|p| safe_hit(p, sentence)));
            if regex_error {
                route.insert(RouteReason::RegexError);
            }
            if rejected_roles.iter().any(|role| incidental(role, card, &contextual, typ, &accepted).is_none()) {
                route.insert(RouteReason::RejectedHit);
            }
            if !ignored && !matched_any {
                route.insert(RouteReason::UnknownLine);
            }
            if accepted.len() > 1
                && !accepted.iter().all(|r| ["ramp", "fixing", "sac-outlet"].contains(r))
                && accepted != BTreeSet::from(["tokens", "draw"])
                && !(accepted == BTreeSet::from(["tokens", "ramp"]) && creates(&s, "treasure"))
            {
                route.insert(RouteReason::SameLine);
            }
        }
        let mut ranked: Vec<_> = found.into_iter().collect();
        ranked.sort_by_key(|(role, (p, _))| (*p, IDS.iter().position(|r| r == role).unwrap()));
        if ranked.len() > 4 {
            route.insert(RouteReason::Overflow);
        }
        if ranked.len() > 4 && ranked.iter().any(|(r, _)| *r == "copy") {
            let copy_line = ranked.iter().find(|(r, _)| *r == "copy").unwrap().1.1.line;
            ranked.retain(|(r, (_, e))| *r != "tokens" || e.line != copy_line);
        }
        ranked.truncate(4);
        let roles: Vec<String> = ranked.iter().map(|(r, _)| r.to_string()).collect();
        if roles.is_empty() && substantive
            || !roles.iter().any(|r| r == "fixing")
                && card.raw.produced_mana.as_ref().is_some_and(|mana| {
                    mana.iter().filter(|m| ["W", "U", "B", "R", "G"].contains(&m.as_str())).count() >= 2
                })
        {
            route.insert(RouteReason::NoRole);
        }
        let ambiguous = ranked.iter().any(|(r, (_, e))| {
            let s = normalize(card, lines[e.line.unwrap()].1).to_lowercase();
            ROLE_POLICY.iter().any(|(role, _, always)| role == r && *always)
                || *r == "removal" && has(&s, &[",", ". ", " and ", " then ", " if "])
        });
        if ambiguous || !roles.is_empty() && roles.iter().all(|r| role_src(r) != "rules") {
            route.insert(RouteReason::RiskyRole);
        }
        let needs_llm = !route.is_empty();
        Ok(Assignment {
            primary: roles.first().cloned().unwrap_or_default(),
            roles,
            evidence: ranked.into_iter().map(|(r, (_, e))| (r.into(), e)).collect(),
            needs_llm,
            route: route.into_iter().collect(),
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

fn accept(r: &Rule, c: &Card, s: &str, typ: &str, intrinsic: bool, full: &str) -> bool {
    let land = typ.contains("Land");
    let spell = has(typ, &["Instant", "Sorcery"]);
    let activation = activation_cost(full).is_some();
    let repeat = activation || has(full, &["whenever", "at the beginning", "each upkeep", "each end step"]);
    let self_sac = has(
        activation_cost(full).map_or("", |(cost, _)| cost),
        &["sacrifice ~", "sacrifice this", "exile ~", "exile this", "discard this", "discard ~"],
    );
    let grant = grammar!(r"TEAM (?:have|has|gains?|gets?)", s)
        || has(
            s,
            &[
                "target creature",
                "target commander",
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
            if mana_filter(full) {
                return false;
            }
            if creates(s, "treasure") {
                return repeat || !grammar!(r"create (?:a|one|1) treasure token\b", s);
            }
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
                    && (full.contains("sacrifice") && !self_sac
                        || s.contains("{c}{c}")
                        || s.contains("two mana")
                        || s.contains("three mana"));
            }
            !has(s, &["its controller", "that player adds", "opponent adds"])
        }
        "fixing" => {
            if intrinsic || mana_filter(full) {
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
                            "chosen color",
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
            !q.contains("card named ~")
                && !q.contains("cards named ~")
                && !has(q, &["land card", "land cards", "plains", "island", "swamp", "mountain", "forest"])
                || q.contains("nonland") && !q.contains("named ~")
        }
        "counter" => !has(full, &["whenever this creature becomes the target", "whenever ~ becomes the target"]),
        "wipe" => {
            if full.contains("leaves the battlefield") && has(full, &["when ~", "when this"])
                || has(s, &["library", "graveyard", "hand"])
                    && !has(
                        s,
                        &[
                            "return all creatures",
                            "return all nonland",
                            "return each nonland",
                            "return all permanents",
                            "sacrifices all creatures",
                        ],
                    )
            {
                return false;
            }
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
            let s = s.split(", where x is ").next().unwrap_or(s);
            if s.contains("exile")
                && has(s, &["you own", "you control"])
                && has(full, &["return", "put both"])
                && full.contains("battlefield")
            {
                return false;
            }
            if has(full, &["reveals their hand", "reveal their hand"])
                && s.contains("exile")
                && !grammar!(
                    r"(?:destroy|exile) target (?:artifact|enchantment|creature|permanent|planeswalker|land)",
                    s,
                )
            {
                return false;
            }
            if has(s, &["tap it", "tap enchanted creature"]) && !full.contains("at the beginning") {
                return false;
            }
            if s.trim_start().starts_with("search your library") || s.contains("you control enchanted") {
                return s.contains("you control enchanted");
            }
            if has(s, &["from their hand", "from your hand"]) && s.contains("exile") {
                return false;
            }
            if s.contains("exile") && has(s, &["graveyard", "exile this", "exile ~"]) && !s.contains("exile target") {
                return false;
            }
            if s.contains("unless") && has(s, &["pay", "pays"]) && s.contains("sacrific") {
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
                        "deals damage",
                        "each opponent",
                        "-1/-1 counter",
                        "gain control of",
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
            (grant && keyword_grant(s)
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
            if s.contains("you become the monarch") {
                return true;
            }
            if !s.contains("draw") && (s.contains("investigate") || creates(s, "clue") || creates(s, "junk")) {
                return repeat && !self_sac && !spell;
            }
            if s.trim().starts_with("cascade") || s.contains("mill a card") && s.contains("may play that card") {
                return true;
            }
            if s.contains("draw") {
                if !(draws_for_you(s) || grammar!(r"each player.s[^.]*that player draws?", full))
                    || (has(typ, &["Aura", "Equipment"]) || !spell) && grammar!(SELF_DEATH_DRAW, full)
                {
                    return false;
                }
                if has(s, &["whenever an opponent draws", "whenever you draw"])
                    && !has(s, &["you may draw", "you draw", ": draw", ", draw"])
                {
                    return false;
                }
                if full.contains("would draw") && !grammar!(r"draws? NUMBER cards? instead", s) {
                    return false;
                }
                if !grammar!(r"\bdraws? (?:NUMBER|an additional|cards|that)\b", s) {
                    return false;
                }
                if loot(full) && !s.contains("draw an additional") {
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
            if has(s, &["look at the top", "reveal the top"])
                && s.contains("from among them")
                && !has(s, &["cast the exiled", "play the exiled"])
            {
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
            if has(s, &["cast the exiled", "play the exiled"]) {
                return false;
            }
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
            if loot(full) && !has(full, &["counter target", "destroy target", "exile target"]) {
                return true;
            }
            // Looting is selection only when it does not also net cards. Optional discard after drawing is incidental.
            if s.contains("draw") && s.contains("discard") {
                if has(s, &["counter target", "destroy target", "exile target"]) {
                    return false;
                }
                return loot(full);
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
                    || s.contains("from a graveyard")
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
            if full.starts_with("discard a card:") {
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
        "copy" => s.contains("create") || !s.contains("a copy of ~") && !s.contains("copy of this"),
        "tokens" => {
            if (has(s, &["its controller creates", "that player creates", "opponent creates", "they create"])
                || grammar!(r"(?:target|that) creature.s controller[^.]*creates?", s))
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
            if grammar!(r"up to NUMBER target creatures", s) {
                return false;
            }
            if s.contains("+1/+1 counter on each creature") {
                return has(s, &["each creature you control", "each creature target player controls"]);
            }
            let subject = s
                .split([',', '.', ':'])
                .find(|part| {
                    grammar!(r"TEAM", part) && has(part, &[" get ", " gets ", " have ", " has ", " gain ", " gains "])
                })
                .unwrap_or("");
            // Only the team itself may receive the buff; a cost or condition mentioning your team is insufficient.
            let buff = [" get ", " gets "]
                .iter()
                .filter_map(|v| subject.find(v))
                .min()
                .or_else(|| [" have ", " has ", " gain ", " gains "].iter().filter_map(|v| subject.rfind(v)).max());
            let Some(buff) = buff else {
                return false;
            };
            let team = &subject[..buff];
            let payoff = &subject[buff..];
            (grammar!(r"TEAM", team) || team.trim().starts_with("all "))
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
            if s.contains("\"when this") || s.contains("\"whenever this") {
                return false;
            }
            if s.contains("double") && s.contains("life total") {
                return true;
            }
            if s.contains("lifelink") {
                return !spell
                    && (!s.contains("until end of turn") || activation)
                    && (grant || !has(s, &["gain", "has", "have"]) && s.split(", ").any(|w| w == "lifelink"));
            }
            if repeat && has(s, &["target player gains", "target opponent gains"]) {
                return true;
            }
            if !has(
                full,
                &[
                    "you gain",
                    "you may gain",
                    "its controller gains",
                    "you've gained",
                    "you gained",
                    "whenever you gain",
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
            let effect = s;
            // Zone-change annotations are bookkeeping, independent of the counter's name.
            if has(effect, &["with a ", "with an "]) && effect.contains("counter on it") {
                return false;
            }
            if !has(
                effect,
                &[
                    "+1/+1",
                    "-1/-1",
                    "target creature",
                    "target artifact",
                    "target permanent",
                    "on each",
                    "on another",
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
                && !(spell && full.contains("if it's a creature") && has(full, &["hexproof", "indestructible"]))
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
        "evasion" => (grant || s.starts_with("all ")) && keyword_grant(s) && !s.contains("can't attack"),
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
    if index.sync_fnv.as_ref().is_some_and(|hash| *hash != index.oracle_fnv) {
        return Err(
            "Oracle fingerprint differs from sync.lock; run `cardgen bulk-sync`, then `cardgen roles --rules`".into()
        );
    }
    let classifier = Classifier::new()?;
    let mut result = RoleMap {
        meta: RoleMeta {
            taxonomy: index.taxonomy_fnv.clone(),
            rules_fnv: rules_hash(overrides.as_bytes()),
            oracle_fnv: index.oracle_fnv.clone(),
            count: index.cards.iter().filter(|c| is_deck_card(c)).count(),
            table_fnv: String::new(), // filled in by bytes()
        },
        cards: BTreeMap::new(),
    };
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
                route: vec![],
            },
        );
    }
    Ok(result)
}

pub fn load(path: &Path) -> Result<RoleMap, String> {
    let f = std::fs::File::open(path).map_err(|e| format!("{}: {e}; run `cardgen roles --rules`", path.display()))?;
    serde_json::from_reader(std::io::BufReader::new(f))
        .map_err(|e| format!("{}: {e}; run `cardgen roles --rules`", path.display()))
}
pub fn bytes(map: &RoleMap) -> Result<Vec<u8>, String> {
    // The hash covers the bytes after the metadata object, so it is written over a fixed-width placeholder.
    let mut marked = map.clone();
    marked.meta.table_fnv = "0".repeat(16);
    let mut out = serde_json::to_vec_pretty(&marked).map_err(|e| e.to_string())?;
    out.push(b'\n');
    let (_, end) = parse_meta(&out)?;
    let hash = fingerprint(&out[end..]);
    let at = out[..end].windows(16).rposition(|w| w == b"0000000000000000").ok_or("meta placeholder")?;
    out[at..at + 16].copy_from_slice(hash.as_bytes());
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
            "--rules" | "--validate" | "--stats" | "--show" | "--audit" | "--lint" => {
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
        let _ = writeln!(out, "  route: {:?}", a.route);
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

/// Rare alternatives are frozen here with semantic reasons; new sparse alternatives fail lint.
const EXCEPTIONS: &[(&str, &str)] = &[
    (
        r"target creature.s controller sacrifices it",
        concat!(
            "C12 requires a controlled object's edict to remain removal when compensation tokens are rejected; ",
            "ordinary player edicts require a quantity and cannot resolve this targeted pronoun."
        ),
    ),
    (
        r"double all damage that sources you control",
        "Frozen whole-team damage multiplier; narrow finisher hint always routed.",
    ),
    (
        r"each creature[^.]*loses all abilities[^.]*1/1",
        "Frozen mass base-stat replacement; distinct from targeted shrink.",
    ),
    (r"return (?:~|this creature) and another target", "Frozen paired self/opponent bounce; separate object ordering."),
    (r"chosen creatures fight", "Frozen non-targeted fight wording; needs removal review."),
    (
        r"exile[^.]*creature you own.*?put both cards onto the battlefield",
        "Frozen two-card exile-return wording; blink destination is explicit.",
    ),
    (
        r"put an indestructible counter on each",
        "Frozen durable team protection; counters also requires same-line review.",
    ),
    (r"you have protection", "Frozen player protection grant rather than a self creature keyword."),
    (
        r"controller reveals their hand, then discards",
        "Frozen reveal/discard subject elision; direct opposing discard.",
    ),
    (
        r"choose target [^.]*card in your graveyard.*?return it to your hand",
        "Frozen target chosen before pronoun return; recursion always routed.",
    ),
    (r"graveyard[^.]*put it into your library", "Frozen pronoun library recursion; source must be checked."),
    (r"return all [^.]*cards in graveyards[^.]*hands", "Frozen symmetric graveyard-to-hand recursion; not a wipe."),
    (
        r"(?:creature|permanent) card in your graveyard.*?put it onto the battlefield",
        concat!(
            "C4 requires the chosen card's type and source across sentences to distinguish recursion ",
            "from reanimation; pronoun-only destinations cannot establish either."
        ),
    ),
    (r"return from your graveyard to the battlefield", "Frozen destination-first reanimation object ordering."),
    (
        r"opponent sacrifices a nontoken permanent[^.]*put that card onto the battlefield",
        "Frozen sacrifice-to-battlefield reuse; non-targeted object reference.",
    ),
    (r"deals damage equal [^.]*to the chosen player", "Frozen chosen-player damage; drain always routed."),
    (
        r"all [a-z]+s have (?:fear|flying|menace|shadow)",
        "Frozen universal tribal keyword grant; evasion always routed.",
    ),
    (
        r"if an opponent[^.]*would put a land[^.]*then sacrifices a land",
        "Frozen land replacement restriction; stax always routed.",
    ),
    (r"controller can't cast spells", "Frozen controller restriction; stax always routed."),
    (
        r"shuffle their library, that player puts a card from their hand",
        "Frozen shuffle-to-hand replacement restriction; stax always routed.",
    ),
    (r"each noncreature artifact loses all abilities", "Frozen static mass ability loss; stax always routed."),
];

fn alternatives(pattern: &str) -> Vec<&str> {
    let (mut depth, mut class, mut escaped, mut start) = (0usize, false, false, 0);
    let mut out = vec![];
    for (i, ch) in pattern.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '[' => class = true,
            ']' => class = false,
            '(' if !class => depth += 1,
            ')' if !class => depth -= 1,
            '|' if !class && depth == 0 => {
                out.push(&pattern[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&pattern[start..]);
    out
}
pub fn lint(index: &Index) -> Result<(String, bool), String> {
    let mut out =
        String::from("RULE ALTERNATIVES matching fewer than 3 catalog cards\nrule\tcount\talternative\treason\n");
    let mut passed = true;
    let lines: Vec<Vec<String>> = index
        .cards
        .iter()
        .filter(|c| is_deck_card(c))
        .map(|c| c.lines().iter().map(|(_, line)| normalize(c, line)).collect())
        .collect();
    for rule in RULES {
        for alternative in alternatives(rule.pattern) {
            let re = regex(alternative)?;
            let count = lines
                .iter()
                .filter(|lines| {
                    lines.iter().any(|line| line.len() <= MAX_LINE_BYTES && re.is_match(line).unwrap_or(false))
                })
                .count();
            if count >= 3 {
                continue;
            }
            let reason = EXCEPTIONS.iter().find(|(p, _)| *p == alternative).map(|(_, reason)| *reason);
            passed &= reason.is_some();
            let _ = writeln!(out, "{}\t{count}\t{alternative}\t{}", rule.id, reason.unwrap_or("UNDOCUMENTED"));
        }
    }
    let _ = writeln!(out, "{}", if passed { "PASS" } else { "FAIL: undocumented sparse alternatives" });
    Ok((out, passed))
}

/// Text and success are separate so a failing validation still prints its entire report before exiting nonzero.
pub fn command(project: &Project, args: &[String]) -> Result<(String, bool), String> {
    let o = options(args)?;
    let dir = knowledge::knowledge_dir(project, o.dir.as_deref())?;
    let mut index = Index::load(&dir.join("oracle.json"))?;
    let taxonomy = Taxonomy::parse(&read(&project.data("knowledge/roles.toml"))?)?;
    if o.mode == "--lint" {
        return lint(&index);
    }
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
        // The rules stage is scored without hand overrides: a manual role set can never improve rule validation.
        let rules = generate(&index, "")?;
        let report = validate(project, &index, &rules, &read(&gold)?, &taxonomy)?;
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
        let _ = writeln!(out, "route: {:?}", a.route);
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
    pub route_causes: BTreeMap<String, usize>,
    pub legal_route_causes: BTreeMap<String, usize>,
    pub legal_route_exclusive: BTreeMap<String, usize>,
    pub legal_route_only: BTreeMap<String, usize>,
}
pub fn stats(index: &Index, map: &RoleMap) -> Stats {
    let causes = [
        RouteReason::UnknownLine,
        RouteReason::NoRole,
        RouteReason::RejectedHit,
        RouteReason::SameLine,
        RouteReason::RiskyRole,
        RouteReason::Overflow,
        RouteReason::RegexError,
    ];
    let counts = || causes.iter().map(|cause| (format!("{cause:?}"), 0)).collect();
    let mut s = Stats {
        route_causes: counts(),
        legal_route_causes: counts(),
        legal_route_exclusive: counts(),
        legal_route_only: counts(),
        per_role: IDS.iter().map(|r| (r.to_string(), 0)).collect(),
        ..Stats::default()
    };
    for c in &index.cards {
        if !is_deck_card(c) {
            s.skipped += 1;
            continue;
        }
        let a = &map[&c.raw.oracle_id];
        for cause in &a.route {
            *s.route_causes.entry(format!("{cause:?}")).or_default() += 1;
            if c.legal == Legal::Legal {
                *s.legal_route_causes.entry(format!("{cause:?}")).or_default() += 1;
            }
        }
        if c.legal == Legal::Legal {
            if let Some(first) = a.route.first() {
                *s.legal_route_exclusive.entry(format!("{first:?}")).or_default() += 1;
                if a.route.len() == 1 {
                    *s.legal_route_only.entry(format!("{first:?}")).or_default() += 1;
                }
            }
        }
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
        for (cause, count) in &self.legal_route_causes {
            let _ = writeln!(
                s,
                "  cause {cause}: {count} legal (overlapping), {} partition, {} sole cause",
                self.legal_route_exclusive.get(cause).unwrap_or(&0),
                self.legal_route_only.get(cause).unwrap_or(&0)
            );
        }
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
        "tut" | "seal" | "top" | "gifts" | "intuition" | "threedreams" => &["tutor"],
        "rock" | "dork" | "lr" | "lrt" | "treas" | "fastmana" | "chromemox" | "moxd" | "led" | "grim" | "tomb"
        | "birgi" | "rishkar" | "jeska" => &["ramp"],
        "draw" | "fbdraw" | "drawcre" | "eng" | "eotdraw" | "enddraw" | "spelldraw" | "remora" | "rhystic"
        | "sphinx" | "clamp" | "adnaus" | "necro" | "onering" | "callring" | "citadel" => &["draw"],
        "rean" | "unearth" | "necromancer" | "suntitan" | "reenact" => &["reanimate"],
        "fill" if value.as_str() == Some("dispute") => &[],
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
        "fill" => "fill=dispute is an additional-cost sacrifice (convention 2); other fill values map to self-mill",
        "xtutor" => "X-spell engine flag, not a library search; unmapped",
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
    pub gold_micro: Score,
    pub primary_correct: usize,
    pub primary_total: usize,
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
    rules: &RoleMap,
    gold: &str,
    taxonomy: &Taxonomy,
) -> Result<Validation, String> {
    let mut v = Validation {
        passed: true,
        hand: BTreeMap::new(),
        hand_micro: Score::default(),
        gold_all: BTreeMap::new(),
        gold_certain: BTreeMap::new(),
        gold_micro: Score::default(),
        primary_correct: 0,
        primary_total: 0,
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
        routing: stats(index, rules),
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
            || g.roles.iter().collect::<BTreeSet<_>>().len() != g.roles.len()
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
        v.primary_total += 1;
        v.primary_correct += usize::from(a.primary == g.primary);
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
        v.gold_micro.add(&v.gold_all[&role.id]);
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
        row(&mut out, "overall micro", &self.gold_micro);
        let _ = writeln!(
            out,
            "PRIMARY ACCURACY: {}/{} = {:.4}",
            self.primary_correct,
            self.primary_total,
            ratio(self.primary_correct, self.primary_total)
        );
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
