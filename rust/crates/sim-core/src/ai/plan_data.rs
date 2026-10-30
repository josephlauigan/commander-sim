//! Deck plans as data (Phase 4, `audit/phase4/plans-design.md`): one TOML file per deck, `data/plans/<deck-key>.toml`,
//! embedded at build time (`build.rs`). The generic policy reads it: the static knobs (style, commander priority and
//! earliest turn, aura host, top-of-library preference, key cards), the win lines the tutors derive their wish list
//! from, cast-priority rules, holds and the mulligan rule.
//!
//! The A/B switch: env `AI_PLAN_DATA=1` plays every deck with a plan file from it, `=0` plays every deck that still
//! has hand code (`plans::CONFIG`) with that code; unset, each plan file's `default` decides ("hand" or "plan").

use super::Style;
use crate::ids::{CardId, PlayerId};
use crate::state::Game;
use crate::tag::Tag;
use indexmap::IndexMap;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::LazyLock;

mod files {
    include!(concat!(env!("OUT_DIR"), "/plan_files.rs"));
}

/// which arm plays the deck when `AI_PLAN_DATA` is unset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Arm {
    /// the deck's hand code (`plans::CONFIG`), the plan file supplying nothing
    #[default]
    Hand,
    /// the plan file
    Plan,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StyleT {
    pub temp: f64,
    pub aggression: f64,
    pub caution: f64,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Commander {
    /// cast priority (default 75, 85 for a commander with card code)
    pub prio: Option<i32>,
    /// earliest turn to cast it (default 2)
    pub turn: Option<u32>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Knobs {
    /// where the deck's Auras go: "commander"
    pub aura_host: Option<String>,
    /// an Aura's cast priority (default 55)
    pub aura_prio: Option<i32>,
    /// how the deck stacks the top of its library: "mv" (biggest mana value first, once the commander can connect)
    pub top_pref: Option<String>,
    /// a card-code permanent's default cast priority (default 55)
    pub hooked_prio: Option<i32>,
}

/// a win line: a catalogue combo (`combos::all()` name), or pieces (groups of card names, one card of each group
/// needed) for a Spellbook combo the catalogue doesn't have or a non-combo plan
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Line {
    pub name: String,
    /// "combo" (default), "combat", "drain", "commander-damage", "lock", "value": documentation and reports
    #[serde(default)]
    pub kind: Option<String>,
    /// the catalogue combo this line is (its groups are the pieces)
    #[serde(default)]
    pub combo: Option<String>,
    /// the Commander Spellbook variant id the pieces are (the plan test checks they match the deck's Spellbook file)
    #[serde(default)]
    pub spellbook: Option<String>,
    #[serde(default)]
    pub pieces: Vec<Vec<String>>,
    /// the groups are steps: tutors want only the first missing one (a payoff after its engine)
    #[serde(default)]
    pub ordered: bool,
    /// the line is live only when this holds (see `Cond`); not live, it counts one step further away
    #[serde(default)]
    pub when: Vec<String>,
}

/// a cast-priority rule: the first rule matching a card decides (`value` when `when` holds, else `else`, else the next
/// rule); no rule: the generic priority
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrioRule {
    #[serde(default)]
    pub cards: Vec<String>,
    /// the deck's commander(s)
    #[serde(default)]
    pub commander: bool,
    #[serde(default)]
    pub creature: Option<bool>,
    #[serde(default)]
    pub flying: Option<bool>,
    /// any of these creature types (lowercase, as `CardData::has_subtype` reads them)
    #[serde(default)]
    pub subtypes: Vec<String>,
    #[serde(default)]
    pub min_mv: Option<u32>,
    #[serde(default)]
    pub max_mv: Option<u32>,
    /// only the cards of this line's next step (its first group not on the battlefield, every earlier group on it)
    #[serde(default)]
    pub next_of: Option<String>,
    #[serde(default)]
    pub when: Vec<String>,
    pub value: i32,
    #[serde(default, rename = "else")]
    pub otherwise: Option<i32>,
    /// value + min(mana value, plus_mv)
    #[serde(default)]
    pub plus_mv: Option<i32>,
}

/// a card the deck keeps for protection (it isn't cast proactively), what it protects, and when to keep mana up
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Hold {
    pub card: String,
    /// "commander", or card names: the permanents worth protecting (`pool::worth_protecting`)
    #[serde(default)]
    pub protects: Vec<String>,
    /// keep its mana up once a protected permanent is out (not read yet)
    #[serde(default)]
    pub mana: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mulligan {
    /// keep with this many lands, inclusive (the generic rule: 2 to 5)
    pub lands: Option<[usize; 2]>,
    /// and at least this many cards of the win lines or engines
    #[serde(default)]
    pub pieces: usize,
    /// a source of each of the deck's colours (not read yet)
    #[serde(default)]
    pub fixing: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub deck: String,
    /// a retired pool deck (only a test seat): its cards are checked against decklists/pool/retired/<deck>.md
    #[serde(default)]
    pub retired: bool,
    #[serde(default)]
    pub default: Arm,
    pub style: Option<StyleT>,
    #[serde(default)]
    pub commander: Commander,
    #[serde(default)]
    pub knobs: Knobs,
    /// card name -> cast priority, over every other rule
    #[serde(default)]
    pub key_cards: IndexMap<String, i32>,
    /// win lines, best first (the order breaks ties)
    #[serde(default)]
    pub line: Vec<Line>,
    /// card engines tutors want once no line wants a card
    #[serde(default)]
    pub engines: Vec<String>,
    #[serde(default)]
    pub prio: Vec<PrioRule>,
    #[serde(default)]
    pub hold: Vec<Hold>,
    pub mulligan: Option<Mulligan>,
}

// ------------------------------------------------------------------ conditions
#[derive(Debug, Clone, Copy, PartialEq)]
enum Cmp {
    Ge,
    Le,
    Gt,
    Lt,
    Eq,
}

#[derive(Debug, Clone, PartialEq)]
enum Atom {
    /// a commander of p's on the battlefield
    CmdOut,
    /// a commander of p's in the command zone
    CmdZone,
    /// a combat with a ninjutsu window ahead (`pool::combat_reserve`)
    NinjutsuWindow,
    /// a death-drain payoff out (`plans::drain_payoff`)
    DrainPayoff,
    /// p's permanent of this name on the battlefield
    Out(String),
    Num(&'static str, Cmp, i32),
    Not(Box<Atom>),
}

/// a condition: any of the alternatives, each `a & b & ...`
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Cond(Vec<Vec<Atom>>);

const NUMS: [&str; 6] = ["creatures", "lands", "tax", "turn", "opp-power", "life"];

fn atom(s: &str) -> Result<Atom, String> {
    let s = s.trim();
    if let Some(r) = s.strip_prefix("not ") {
        return Ok(Atom::Not(Box::new(atom(r)?)));
    }
    if let Some(r) = s.strip_prefix("out:") {
        return Ok(Atom::Out(r.trim().to_string()));
    }
    match s {
        "commander-out" => return Ok(Atom::CmdOut),
        "commander-zone" => return Ok(Atom::CmdZone),
        "ninjutsu-window" => return Ok(Atom::NinjutsuWindow),
        "drain-payoff" => return Ok(Atom::DrainPayoff),
        _ => {}
    }
    for (op, c) in [(">=", Cmp::Ge), ("<=", Cmp::Le), (">", Cmp::Gt), ("<", Cmp::Lt), ("=", Cmp::Eq)] {
        if let Some((k, v)) = s.split_once(op) {
            let k = NUMS.iter().find(|n| **n == k.trim()).ok_or_else(|| format!("unknown quantity {k:?} in {s:?}"))?;
            let v = v.trim().parse().map_err(|_| format!("bad number in {s:?}"))?;
            return Ok(Atom::Num(k, c, v));
        }
    }
    Err(format!("unknown condition {s:?}"))
}

impl Cond {
    pub fn parse(alts: &[String]) -> Result<Cond, String> {
        alts.iter().map(|a| a.split('&').map(atom).collect()).collect::<Result<_, _>>().map(Cond)
    }

    /// no alternatives: always true
    pub fn holds(&self, g: &Game, p: PlayerId) -> bool {
        self.0.is_empty() || self.0.iter().any(|all| all.iter().all(|a| eval(g, p, a)))
    }

    fn names(&self) -> impl Iterator<Item = &str> {
        fn walk(a: &Atom) -> Option<&str> {
            match a {
                Atom::Out(n) => Some(n),
                Atom::Not(b) => walk(b),
                _ => None,
            }
        }
        self.0.iter().flatten().filter_map(walk)
    }
}

fn creatures(g: &Game, p: PlayerId) -> i32 {
    g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).count() as i32
}

fn eval(g: &Game, p: PlayerId, a: &Atom) -> bool {
    let pl = g.player(p);
    match a {
        Atom::CmdOut => pl.perms.iter().any(|&m| g.perm(m).is_cmd),
        Atom::CmdZone => pl.cmd_in_zone,
        Atom::NinjutsuWindow => super::pool::combat_reserve(g, p).is_some(),
        Atom::DrainPayoff => super::plans::drain_payoff(g, p).is_some(),
        Atom::Out(n) => super::plans::on_bf(g, p, n).is_some(),
        Atom::Not(b) => !eval(g, p, b),
        Atom::Num(k, c, v) => {
            let x = match *k {
                "creatures" => creatures(g, p),
                "lands" => pl.lands.len() as i32,
                "tax" => pl.tax as i32,
                "turn" => pl.turns as i32,
                "opp-power" => super::plans::opp_power(g, p),
                "life" => pl.life,
                _ => unreachable!(),
            };
            match c {
                Cmp::Ge => x >= *v,
                Cmp::Le => x <= *v,
                Cmp::Gt => x > *v,
                Cmp::Lt => x < *v,
                Cmp::Eq => x == *v,
            }
        }
    }
}

// ------------------------------------------------------------------ the loaded plans
/// a plan with its conditions parsed and its lines' groups resolved
pub struct Loaded {
    pub plan: Plan,
    /// per line: its groups (a catalogue combo's, or its pieces) and its condition
    pub lines: Vec<(Vec<Vec<String>>, Cond)>,
    pub prio_when: Vec<Cond>,
    /// the DeckConfig this plan stands for (hand functions filled in by `plans::config`)
    pub cfg: super::plans::DeckConfig,
}

fn leak(s: &str) -> &'static str {
    Box::leak(s.to_string().into_boxed_str())
}

/// parse and resolve one plan file (`stem`: its file name, which must be the deck key)
pub fn load(stem: &str, text: &str) -> Result<Loaded, String> {
    let plan: Plan = toml::from_str(text).map_err(|e| format!("data/plans/{stem}.toml: {e}"))?;
    let err = |m: String| format!("data/plans/{stem}.toml: {m}");
    if plan.deck != stem {
        return Err(err(format!("deck = {:?}, but the file is named for {stem:?}", plan.deck)));
    }
    let mut lines = vec![];
    for l in &plan.line {
        let groups: Vec<Vec<String>> = match (&l.combo, l.pieces.is_empty()) {
            (Some(c), true) => {
                let cmb = crate::impls::combos::all()
                    .iter()
                    .find(|x| x.name == c)
                    .ok_or_else(|| err(format!("line {:?}: no catalogue combo {c:?}", l.name)))?;
                cmb.groups.iter().map(|g| g.iter().map(|s| s.to_string()).collect()).collect()
            }
            (None, false) => l.pieces.clone(),
            (Some(_), false) => return Err(err(format!("line {:?}: give combo or pieces, not both", l.name))),
            (None, true) => return Err(err(format!("line {:?}: no combo and no pieces", l.name))),
        };
        if groups.iter().any(|g| g.is_empty()) {
            return Err(err(format!("line {:?}: an empty group", l.name)));
        }
        lines.push((groups, Cond::parse(&l.when).map_err(|e| err(format!("line {:?}: {e}", l.name)))?));
    }
    let mut prio_when = vec![];
    for (i, r) in plan.prio.iter().enumerate() {
        let any_sel = !r.cards.is_empty()
            || r.commander
            || r.creature.is_some()
            || r.flying.is_some()
            || !r.subtypes.is_empty()
            || r.min_mv.is_some()
            || r.max_mv.is_some()
            || r.next_of.is_some();
        if !any_sel {
            return Err(err(format!("prio rule {}: it selects no card", i + 1)));
        }
        if let Some(n) = &r.next_of
            && !plan.line.iter().any(|l| &l.name == n)
        {
            return Err(err(format!("prio rule {}: next_of {n:?} is not a line", i + 1)));
        }
        prio_when.push(Cond::parse(&r.when).map_err(|e| err(format!("prio rule {}: {e}", i + 1)))?);
    }
    if let Some(s) = plan.style
        && ![s.temp, s.aggression, s.caution].iter().all(|v| (0.0..=5.0).contains(v))
    {
        return Err(err("style values must be in 0..5".into()));
    }
    if let Some(t) = &plan.knobs.top_pref
        && t != "mv"
    {
        return Err(err(format!("knobs.top_pref {t:?}: only \"mv\"")));
    }
    if let Some(t) = &plan.knobs.aura_host
        && t != "commander"
    {
        return Err(err(format!("knobs.aura_host {t:?}: only \"commander\"")));
    }
    let key_cards: Vec<(&'static str, i32)> = plan.key_cards.iter().map(|(k, v)| (leak(k), *v)).collect();
    let cfg = super::plans::DeckConfig {
        style: plan.style.map(|s| Style { temp: s.temp, aggression: s.aggression, caution: s.caution }),
        key_cards: Box::leak(key_cards.into_boxed_slice()),
        wish: (!plan.line.is_empty() || !plan.engines.is_empty()).then_some(derived_wish as super::plans::WishFn),
        prio_fn: (!plan.prio.is_empty()).then_some(rule_prio as super::plans::PlanPrioFn),
        cmd_prio: plan.commander.prio,
        cmd_turn: plan.commander.turn,
        aura_prio: plan.knobs.aura_prio,
        aura_host: plan.knobs.aura_host.as_deref().map(leak),
        top_pref: plan.knobs.top_pref.as_deref().map(leak),
        hooked_prio: plan.knobs.hooked_prio,
    };
    Ok(Loaded { plan, lines, prio_when, cfg })
}

/// every plan file, by deck key (a bad file panics on first use, naming the file)
pub fn all() -> &'static HashMap<&'static str, Loaded> {
    static ALL: LazyLock<HashMap<&'static str, Loaded>> = LazyLock::new(|| {
        files::FILES.iter().map(|(stem, text)| (*stem, load(stem, text).unwrap_or_else(|e| panic!("{e}")))).collect()
    });
    &ALL
}

pub fn get(key: &str) -> Option<&'static Loaded> {
    all().get(key)
}

/// the A/B switch, env `AI_PLAN_DATA`: "1" every deck from its plan file, "0" every deck with hand code from that,
/// a comma-separated list of deck keys: those decks from their plan files and every other deck with hand code from
/// that (a paired test of one deck); unset or empty: each file's `default`
enum Switch {
    Unset,
    All(bool),
    Only(Vec<String>),
}

fn switch() -> &'static Switch {
    static S: LazyLock<Switch> = LazyLock::new(|| match std::env::var("AI_PLAN_DATA").ok().as_deref() {
        None | Some("") => Switch::Unset,
        Some("1") => Switch::All(true),
        Some("0") => Switch::All(false),
        Some(v) => {
            let keys: Vec<String> = v.split(',').map(|k| k.trim().to_string()).collect();
            if let Some(k) = keys.iter().find(|k| get(k).is_none()) {
                panic!("bad AI_PLAN_DATA: {k:?} has no plan file (0, 1 or deck keys)");
            }
            Switch::Only(keys)
        }
    });
    &S
}

/// the deck plays from its plan file (`has_hand`: it still has hand code to fall back on)
pub fn plan_on(key: &str, has_hand: bool) -> bool {
    let Some(l) = get(key) else { return false };
    if !has_hand {
        return true;
    }
    match switch() {
        Switch::Unset => l.plan.default == Arm::Plan,
        Switch::All(v) => *v,
        Switch::Only(keys) => keys.iter().any(|k| k == key),
    }
}

/// the plan p plays from, if any (`plans::config` decided)
fn active(g: &Game, p: PlayerId) -> Option<&'static Loaded> {
    let key = g.player(p).key;
    get(key).filter(|_| super::plans::plays_plan(key))
}

// ------------------------------------------------------------------ what the generic policy reads
fn name(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

/// p holds a card of the group: on the battlefield, in hand, a land in play, a commander in the command zone, or on
/// top of its library where p put it itself
fn holds_group(g: &Game, p: PlayerId, grp: &[String], top: &[&str]) -> bool {
    let pl = g.player(p);
    grp.iter().any(|n| {
        super::plans::on_bf(g, p, n).is_some()
            || pl.hand.iter().any(|&c| name(g, c) == n)
            || pl.lands.iter().any(|&l| name(g, g.perm(l).card()) == n)
            || pl.cmds_in_zone().any(|c| name(g, c) == n)
            || top.contains(&n.as_str())
    })
}

fn in_deck(g: &Game, p: PlayerId, n: &str) -> bool {
    let pl = g.player(p);
    pl.deck_names.iter().any(|&c| name(g, c) == n) || pl.commanders().any(|c| name(g, c) == n)
}

/// The derived tutor priority (replaces a hand wish list): the mana a line complete but for its mana needs; then, line
/// by line from the closest (fewest groups missing, one more when its condition fails; ties in file order), the
/// library cards of its missing groups (an ordered line: of its first missing group only); then the engines.
pub fn derived_wish(g: &Game, p: PlayerId) -> Vec<CardId> {
    let Some(l) = active(g, p) else { return crate::cardcode::missing_pieces(g, p) };
    let mana = crate::impls::combos::mana_for_lines(g, p);
    if !mana.is_empty() {
        return mana;
    }
    let top: Vec<&str> = crate::engine::tutors::known_top(g, p).into_iter().map(|c| name(g, c)).collect();
    let pl = g.player(p);
    let lib: Vec<CardId> = pl.library.clone();
    let mut ranked: Vec<(usize, usize, Vec<CardId>)> = vec![];
    for (i, (groups, cond)) in l.lines.iter().enumerate() {
        if !groups.iter().all(|grp| grp.iter().any(|n| in_deck(g, p, n))) {
            continue; // a group the list no longer plays (the plan test catches it first)
        }
        let miss: Vec<&Vec<String>> = groups.iter().filter(|grp| !holds_group(g, p, grp, &top)).collect();
        if miss.is_empty() {
            continue;
        }
        let dist = miss.len() + usize::from(!cond.holds(g, p));
        let wanted: &[&Vec<String>] = if l.plan.line[i].ordered { &miss[..1] } else { &miss };
        let want: Vec<CardId> = wanted
            .iter()
            .flat_map(|grp| grp.iter())
            .filter_map(|n| lib.iter().copied().find(|&c| name(g, c) == n))
            .collect();
        if !want.is_empty() {
            ranked.push((dist, i, want));
        }
    }
    ranked.sort_by_key(|x| (x.0, x.1));
    let mut out: Vec<CardId> = vec![];
    for c in ranked.into_iter().flat_map(|x| x.2) {
        if !out.contains(&c) {
            out.push(c);
        }
    }
    let have = |c: CardId| pl.hand.contains(&c) || pl.perms.iter().any(|&m| g.perm(m).cd == Some(c));
    for n in &l.plan.engines {
        if let Some(c) = lib.iter().copied().find(|&c| name(g, c) == n)
            && !have(c)
            && !out.contains(&c)
        {
            out.push(c);
        }
    }
    out
}

fn on_bf_group(g: &Game, p: PlayerId, grp: &[String]) -> bool {
    grp.iter().any(|n| super::plans::on_bf(g, p, n).is_some())
}

/// the plan's cast-priority rules (replaces a hand priority function): see `PrioRule`
pub fn rule_prio(g: &Game, p: PlayerId, c: CardId) -> Option<i32> {
    let l = active(g, p)?;
    let pl = g.player(p);
    let d = g.db.get(c);
    for (r, when) in l.plan.prio.iter().zip(&l.prio_when) {
        let sel = (r.cards.is_empty() || r.cards.iter().any(|n| *n == *d.name))
            && (!r.commander || pl.is_commander(c))
            && r.creature.is_none_or(|v| d.creature == v)
            && r.flying.is_none_or(|v| d.tag(Tag::Fly) == v)
            && (r.subtypes.is_empty() || r.subtypes.iter().any(|t| d.has_subtype(t)))
            && r.min_mv.is_none_or(|v| d.cmc >= v)
            && r.max_mv.is_none_or(|v| d.cmc <= v);
        if !sel {
            continue;
        }
        if let Some(n) = &r.next_of {
            let i = l.plan.line.iter().position(|x| &x.name == n).unwrap();
            let groups = &l.lines[i].0;
            let Some(k) = groups.iter().position(|grp| !on_bf_group(g, p, grp)) else { continue };
            if !groups[k].iter().any(|x| *x == *d.name) {
                continue;
            }
        }
        if when.holds(g, p) {
            return Some(r.value + r.plus_mv.map_or(0, |k| (d.cmc as i32).min(k)));
        }
        if let Some(v) = r.otherwise {
            return Some(v);
        }
    }
    None
}

/// a hold of p's plan protects this permanent's card (`pool::worth_protecting`)
pub fn protects(g: &Game, p: PlayerId, card: &str) -> bool {
    active(g, p).is_some_and(|l| l.plan.hold.iter().any(|h| h.protects.iter().any(|x| x == card)))
}

/// the plan's keep rule for p's current hand (None: no mulligan section, the generic rule decides)
pub fn keeps_hand(g: &Game, p: PlayerId) -> Option<bool> {
    let l = active(g, p)?;
    let m = l.plan.mulligan.as_ref()?;
    let hand = &g.player(p).hand;
    let lands = hand.iter().filter(|&&c| g.db.get(c).land).count();
    let [lo, hi] = m.lands.unwrap_or([2, 5]);
    if !(lo..=hi).contains(&lands) {
        return Some(false);
    }
    let piece = |c: CardId| {
        let n = name(g, c);
        l.lines.iter().any(|(gs, _)| gs.iter().flatten().any(|x| x == n)) || l.plan.engines.iter().any(|x| x == n)
    };
    Some(hand.iter().filter(|&&c| piece(c)).count() >= m.pieces)
}

/// every card name a plan file names (for the plan test: each must be in the deck)
pub fn named_cards(l: &Loaded) -> Vec<String> {
    let p = &l.plan;
    let mut v: Vec<String> = p.key_cards.keys().cloned().collect();
    for (i, x) in p.line.iter().enumerate() {
        if x.combo.is_none() {
            v.extend(x.pieces.iter().flatten().cloned());
        }
        v.extend(l.lines[i].1.names().map(String::from));
    }
    v.extend(p.engines.iter().cloned());
    for (r, w) in p.prio.iter().zip(&l.prio_when) {
        v.extend(r.cards.iter().cloned());
        v.extend(w.names().map(String::from));
    }
    for h in &p.hold {
        v.push(h.card.clone());
        v.extend(h.protects.iter().filter(|x| *x != "commander").cloned());
    }
    v
}
