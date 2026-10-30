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
/// needed) for a Spellbook combo the catalogue doesn't have or a non-combo plan.
///
/// --- phase4-cedh: a group entry `"bf:<card>"` counts that card as held only on the battlefield (Scroll Rack in hand
/// puts nothing back); an entry `"?<condition>"` holds the group while the condition holds (`?to-top`: the tutor puts
/// its card on top); `counts` says how many cards of each group the line needs (Yawgmoth's two undying creatures)
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
    /// --- phase4-cedh: cards needed of each group (default 1 each; one entry per group)
    #[serde(default)]
    pub counts: Vec<usize>,
    /// the groups are steps: tutors want only the first missing one (a payoff after its engine)
    #[serde(default)]
    pub ordered: bool,
    /// the line is live only when this holds (see `Cond`); not live, it counts one step further away
    #[serde(default)]
    pub when: Vec<String>,
}

/// a cast-priority rule: the first rule matching a card decides (`value` when `when` holds, else `else`, else the next
/// rule); no rule: the generic priority. Values are on the 0-100 scale (whole numbers for the pool decks; a Tier 6
/// deck's plan slot reads them as they are)
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
    pub value: f64,
    #[serde(default, rename = "else")]
    pub otherwise: Option<f64>,
    /// value + min(mana value, plus_mv)
    #[serde(default)]
    pub plus_mv: Option<i32>,
    /// --- phase4-cedh: value + per_tax x the card's commander tax (Kinnan: 92 - 3 per cast)
    #[serde(default)]
    pub per_tax: Option<f64>,
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

/// --- phase4-cedh: one row of a mulligan rule table: the first row whose `when` holds decides
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MullRule {
    pub when: Vec<String>,
    pub keep: bool,
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
    /// --- phase4-cedh: count the hand's mana as the Tier 6 rule does (`t6_keep::mana`: what each rock, Mox and dork
    /// makes by turn 2 from this hand) and keep a settled hand (2-4 lands after a mulligan) first
    #[serde(default)]
    pub fast_mana: bool,
    /// one-shot mana (rituals, spirit guides): counted into `fast` and `mana`
    #[serde(default)]
    pub rituals: Vec<String>,
    /// the rule table (replaces `lands` / `pieces` when given): the first row that holds decides
    #[serde(default)]
    pub rule: Vec<MullRule>,
    /// no row holds: keep (default true)
    #[serde(default)]
    pub otherwise: Option<bool>,
}

/// --- phase4-cedh: a scored tutor target (a Tier 6 wish list): each library card takes the score of the first rule
/// selecting it whose `when` holds; best score first, ties in library order
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WishRule {
    pub cards: Vec<String>,
    #[serde(default)]
    pub when: Vec<String>,
    pub score: f64,
    /// a card that brings a started line closer is worth more: + plus_line / the line's groups still missing
    #[serde(default)]
    pub plus_line: Option<f64>,
    /// the card is a target only above this score (default 0)
    #[serde(default)]
    pub min_score: Option<f64>,
}

/// --- phase4-cedh: the extra utility of passing with mana up in the main phase (Tier 6 `hold_up`): the first rule
/// whose `when` holds gives its value (none given: no opinion); no rule holds: no opinion
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HoldUp {
    #[serde(default)]
    pub when: Vec<String>,
    /// absent: no opinion (and the search stops)
    #[serde(default)]
    pub value: Option<f64>,
}

/// --- phase4-cedh: a go-now gate (Tier 6 `go_now`) for the catalogue combos whose name starts with one of `combos`:
/// the first rule matching the combo whose `when` holds decides: `go = false` blocks the attempt, `true` forces it,
/// no `go` leaves it to the shared table check (and stops the search)
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoNow {
    pub combos: Vec<String>,
    #[serde(default)]
    pub when: Vec<String>,
    #[serde(default)]
    pub go: Option<bool>,
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
    /// --- phase4-cedh: card engines tutors want ahead of every line two or more steps away (Yuriko's Rhystic Study)
    #[serde(default)]
    pub engines_ahead: Vec<String>,
    /// --- phase4-cedh: named card sets the conditions count in hand (`hand:<set>`)
    #[serde(default)]
    pub sets: IndexMap<String, Vec<String>>,
    #[serde(default)]
    pub prio: Vec<PrioRule>,
    /// --- phase4-cedh: scored tutor targets, ahead of the lines' pieces
    #[serde(default)]
    pub wish: Vec<WishRule>,
    #[serde(default)]
    pub hold: Vec<Hold>,
    /// --- phase4-cedh: Tier 6 hold-up utility rules
    #[serde(default)]
    pub hold_up: Vec<HoldUp>,
    /// --- phase4-cedh: Tier 6 go-now gates
    #[serde(default)]
    pub go_now: Vec<GoNow>,
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

/// a quantity a condition compares
#[derive(Debug, Clone, Copy, PartialEq)]
enum Q {
    Creatures,
    Lands,
    Tax,
    Turn,
    OppPower,
    Life,
    /// --- phase4-cedh: the game round
    Round,
    /// the biggest mana value among the other cards in hand (the card being rated left out)
    HandMaxMv,
    /// the hand's lands (mulligan rules)
    HandLands,
    /// the hand's nonland cards
    Nonland,
    /// the hand's tutors (tag `tut`)
    Tutors,
    /// the hand's mana pieces that count (`t6_keep::mana`'s `n_mana`) plus its rituals
    Fast,
    /// the hand's mana by turn 2 (`t6_keep::mana`'s `eff`) plus its rituals
    Mana,
    /// `t6_keep::mana`'s `eff` alone
    Eff,
    /// `t6_keep::mana`'s `n_mana` alone
    Makers,
    /// opponents with two or more open blue mana and a card in hand
    OppBlueUp,
    /// --- phase4-cedh: the table: the highest chance an opponent stops a spell (`t6_table::stop_chance`)
    OppStopMax,
    /// opponents whose stop chance is 0.5 or more
    OppStoppers,
    /// the most open mana an opponent has
    OppOpenMax,
    /// the highest archenemy score at the table (`t6_table::archenemy_score`)
    ArchenemyMax,
    /// opponents with a card in hand and open mana
    OppCardsUp,
    /// opponents who can cast a second spell (one cast this turn and a card, or two cards and two mana)
    OppSecondSpell,
    /// the hand's blue cards
    HandBlue,
    /// the hand's counterspells (tag `ctr`)
    Counters,
    /// p's nonland permanents of mana value 3 or less that make mana (tags rock, dork, fastmana, lr; Lotus Petal,
    /// Mana Vault, Mox Amber, Mox Opal)
    ManaPerms,
    /// cards of this plan set (`[sets]`, by index) in hand
    Hand(usize),
    /// p's permanents of this plan set on the battlefield
    Bf(usize),
}

const NUMS: [(&str, Q); 25] = [
    ("opp-stop-max", Q::OppStopMax),
    ("opp-stoppers", Q::OppStoppers),
    ("opp-open-max", Q::OppOpenMax),
    ("archenemy-max", Q::ArchenemyMax),
    ("opp-cards-up", Q::OppCardsUp),
    ("opp-second-spell", Q::OppSecondSpell),
    ("hand-blue", Q::HandBlue),
    ("counters", Q::Counters),
    ("mana-perms", Q::ManaPerms),
    ("creatures", Q::Creatures),
    ("lands", Q::Lands),
    ("tax", Q::Tax),
    ("turn", Q::Turn),
    ("opp-power", Q::OppPower),
    ("life", Q::Life),
    ("round", Q::Round),
    ("hand-max-mv", Q::HandMaxMv),
    ("hand-lands", Q::HandLands),
    ("nonland", Q::Nonland),
    ("tutors", Q::Tutors),
    ("fast", Q::Fast),
    ("mana", Q::Mana),
    ("eff", Q::Eff),
    ("makers", Q::Makers),
    ("opp-blue-up", Q::OppBlueUp),
];

/// the right-hand side of a comparison: a number, or `one-lander` (the mana a one-land hand needs, by mulligan)
#[derive(Debug, Clone, Copy, PartialEq)]
enum Rhs {
    N(f64),
    OneLander,
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
    /// --- phase4-cedh: the tutor being chosen puts its card on top of the library
    ToTop,
    /// p's permanent of this name on the battlefield
    Out(String),
    /// --- phase4-cedh: an opponent's permanent of this name on the battlefield
    OppOut(String),
    /// --- phase4-cedh: p holds this card on the battlefield or in hand
    Have(String),
    /// --- phase4-cedh: a line of p's catalogue combos is ready to go (`combos::line_progress`)
    LineReady,
    /// a sum of quantities compared with a number
    Num(Vec<Q>, Cmp, Rhs),
    Not(Box<Atom>),
    /// --- phase4-cedh: a parenthesised condition: any of the alternatives, each all of its atoms
    Any(Vec<Vec<Atom>>),
}

/// a condition: any of the alternatives, each `a & b & ...`; `|` and parentheses nest
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Cond(Vec<Vec<Atom>>);

/// split at `ch` outside parentheses
fn split_top(s: &str, ch: char) -> Vec<&str> {
    let (mut depth, mut start, mut out) = (0i32, 0, vec![]);
    for (i, c) in s.char_indices() {
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ if c == ch && depth == 0 => {
                out.push(&s[start..i]);
                start = i + c.len_utf8();
            }
            _ => {}
        }
    }
    out.push(&s[start..]);
    out
}

/// `a & b | c & (d | e)`: alternatives of conjunctions
fn alts(s: &str, sets: &[&str]) -> Result<Vec<Vec<Atom>>, String> {
    split_top(s, '|').into_iter().map(|a| split_top(a, '&').into_iter().map(|x| atom(x, sets)).collect()).collect()
}

fn atom(s: &str, sets: &[&str]) -> Result<Atom, String> {
    let s = s.trim();
    if let Some(r) = s.strip_prefix("not ") {
        return Ok(Atom::Not(Box::new(atom(r, sets)?)));
    }
    if let Some(r) = s.strip_prefix('(').and_then(|r| r.strip_suffix(')')) {
        return Ok(Atom::Any(alts(r, sets)?));
    }
    if s.contains('(') || s.contains(')') {
        return Err(format!("unbalanced parentheses in {s:?}"));
    }
    for (pre, f) in [("out:", Atom::Out as fn(String) -> Atom), ("opp-out:", Atom::OppOut), ("have:", Atom::Have)] {
        if let Some(r) = s.strip_prefix(pre) {
            return Ok(f(r.trim().to_string()));
        }
    }
    match s {
        "commander-out" => return Ok(Atom::CmdOut),
        "commander-zone" => return Ok(Atom::CmdZone),
        "ninjutsu-window" => return Ok(Atom::NinjutsuWindow),
        "drain-payoff" => return Ok(Atom::DrainPayoff),
        "to-top" => return Ok(Atom::ToTop),
        "line-ready" => return Ok(Atom::LineReady),
        _ => {}
    }
    for (op, c) in [(">=", Cmp::Ge), ("<=", Cmp::Le), (">", Cmp::Gt), ("<", Cmp::Lt), ("=", Cmp::Eq)] {
        if let Some((k, v)) = s.split_once(op) {
            let set_ix =
                |set: &str| sets.iter().position(|x| *x == set).ok_or_else(|| format!("no set {set:?} in {s:?}"));
            let mut qs = vec![];
            for k in k.split('+').map(str::trim) {
                qs.push(if let Some(set) = k.strip_prefix("hand:") {
                    Q::Hand(set_ix(set)?)
                } else if let Some(set) = k.strip_prefix("bf:") {
                    Q::Bf(set_ix(set)?)
                } else {
                    NUMS.iter().find(|n| n.0 == k).ok_or_else(|| format!("unknown quantity {k:?} in {s:?}"))?.1
                });
            }
            let v = match v.trim() {
                "one-lander" => Rhs::OneLander,
                x => Rhs::N(x.parse().map_err(|_| format!("bad number in {s:?}"))?),
            };
            return Ok(Atom::Num(qs, c, v));
        }
    }
    Err(format!("unknown condition {s:?}"))
}

/// what a condition is evaluated for: the plan (its sets and rituals) and the card being rated, if any
#[derive(Clone, Copy, Default)]
pub struct Cx<'a> {
    pub plan: Option<&'a Loaded>,
    pub card: Option<CardId>,
}

impl Cond {
    /// parse without plan sets (`hand:<set>` refused)
    pub fn parse(alts: &[String]) -> Result<Cond, String> {
        Cond::parse_with(alts, &[])
    }

    pub fn parse_with(list: &[String], sets: &[&str]) -> Result<Cond, String> {
        let mut out = vec![];
        for a in list {
            out.extend(alts(a, sets)?);
        }
        Ok(Cond(out))
    }

    /// no alternatives: always true
    pub fn holds(&self, g: &Game, p: PlayerId) -> bool {
        self.holds_cx(g, p, Cx::default())
    }

    pub fn holds_cx(&self, g: &Game, p: PlayerId, cx: Cx) -> bool {
        self.0.is_empty() || self.0.iter().any(|all| all.iter().all(|a| eval(g, p, a, cx)))
    }

    fn names(&self) -> impl Iterator<Item = &str> {
        fn walk<'a>(a: &'a Atom, out: &mut Vec<&'a str>) {
            match a {
                Atom::Out(n) | Atom::Have(n) => out.push(n),
                Atom::Not(b) => walk(b, out),
                Atom::Any(alts) => alts.iter().flatten().for_each(|x| walk(x, out)),
                _ => {}
            }
        }
        let mut out = vec![];
        self.0.iter().flatten().for_each(|a| walk(a, &mut out));
        out.into_iter()
    }
}

fn creatures(g: &Game, p: PlayerId) -> i32 {
    g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).count() as i32
}

fn hand_count(g: &Game, p: PlayerId, names: &[String]) -> usize {
    g.player(p).hand.iter().filter(|&&c| names.iter().any(|n| *n == *g.db.get(c).name)).count()
}

fn rituals(g: &Game, p: PlayerId, cx: Cx) -> f64 {
    let r = cx.plan.and_then(|l| l.plan.mulligan.as_ref()).map_or(&[][..], |m| &m.rituals[..]);
    hand_count(g, p, r) as f64
}

fn quantity(g: &Game, p: PlayerId, q: Q, cx: Cx) -> f64 {
    let pl = g.player(p);
    let hand = &pl.hand;
    let mana = || crate::impls::t6_keep::mana(g, p);
    match q {
        Q::Creatures => creatures(g, p) as f64,
        Q::Lands => pl.lands.len() as f64,
        Q::Tax => pl.tax as f64,
        Q::Turn => pl.turns as f64,
        Q::OppPower => super::plans::opp_power(g, p) as f64,
        Q::Life => pl.life as f64,
        Q::Round => g.round as f64,
        Q::HandMaxMv => {
            let me = cx.card.and_then(|c| hand.iter().position(|&x| x == c)); // this copy only
            hand.iter().enumerate().filter(|(i, _)| Some(*i) != me).map(|(_, &x)| g.db.get(x).cmc).max().unwrap_or(0)
                as f64
        }
        Q::HandLands => hand.iter().filter(|&&c| g.db.get(c).land).count() as f64,
        Q::Nonland => hand.iter().filter(|&&c| !g.db.get(c).land).count() as f64,
        Q::Tutors => hand.iter().filter(|&&c| g.db.get(c).tag(Tag::Tut)).count() as f64,
        Q::Fast => mana().n_mana as f64 + rituals(g, p, cx),
        Q::Mana => mana().eff + rituals(g, p, cx),
        Q::Eff => mana().eff,
        Q::Makers => mana().n_mana as f64,
        Q::OppBlueUp => {
            g.opps(p).filter(|&q| super::brain::open_mana(g, q, Some('U')) >= 2 && !g.player(q).hand.is_empty()).count()
                as f64
        }
        Q::OppStopMax => g.opps(p).map(|q| super::t6_table::stop_chance(g, q)).fold(0.0, f64::max),
        Q::OppStoppers => g.opps(p).filter(|&q| super::t6_table::stop_chance(g, q) >= 0.5).count() as f64,
        Q::OppOpenMax => g.opps(p).map(|q| super::brain::open_mana(g, q, None)).max().unwrap_or(0) as f64,
        Q::ArchenemyMax => g.opps(p).map(|q| super::t6_table::archenemy_score(g, q)).fold(0.0, f64::max),
        Q::OppCardsUp => {
            g.opps(p).filter(|&q| !g.player(q).hand.is_empty() && super::brain::open_mana(g, q, None) >= 1).count()
                as f64
        }
        Q::OppSecondSpell => g
            .opps(p)
            .filter(|&q| {
                let (cards, mana) = (g.player(q).hand.len(), super::brain::open_mana(g, q, None));
                (crate::engine::cast::casts_this_turn(g, q) >= 1 && cards >= 1) || (cards >= 2 && mana >= 2)
            })
            .count() as f64,
        Q::HandBlue => hand.iter().filter(|&&c| g.db.get(c).colors.has('U')).count() as f64,
        Q::Counters => hand.iter().filter(|&&c| g.db.get(c).tag(Tag::Ctr)).count() as f64,
        Q::ManaPerms => pl
            .perms
            .iter()
            .filter(|&&m| {
                g.perm(m).cd.is_some_and(|c| {
                    let d = g.db.get(c);
                    !d.land
                        && d.cmc <= 3
                        && (d.tag(Tag::Rock)
                            || d.tag(Tag::Dork)
                            || d.tag(Tag::Fastmana)
                            || d.tag(Tag::Lr)
                            || matches!(&*d.name, "Lotus Petal" | "Mana Vault" | "Mox Amber" | "Mox Opal"))
                })
            })
            .count() as f64,
        Q::Hand(i) => cx.plan.map_or(0, |l| hand_count(g, p, &l.sets[i])) as f64,
        Q::Bf(i) => cx.plan.map_or(0, |l| {
            let set = &l.sets[i];
            pl.perms
                .iter()
                .filter(|&&m| !g.perm(m).phased && g.perm(m).cd.is_some_and(|c| set.iter().any(|n| *n == *name(g, c))))
                .count()
        }) as f64,
    }
}

fn eval(g: &Game, p: PlayerId, a: &Atom, cx: Cx) -> bool {
    let pl = g.player(p);
    match a {
        Atom::CmdOut => pl.perms.iter().any(|&m| g.perm(m).is_cmd),
        Atom::CmdZone => pl.cmd_in_zone,
        Atom::NinjutsuWindow => super::pool::combat_reserve(g, p).is_some(),
        Atom::DrainPayoff => super::plans::drain_payoff(g, p).is_some(),
        Atom::ToTop => pl.to_top,
        Atom::LineReady => crate::impls::combos::line_progress(g, p).1,
        Atom::Any(alts) => alts.iter().any(|all| all.iter().all(|a| eval(g, p, a, cx))),
        Atom::Out(n) => super::plans::on_bf(g, p, n).is_some(),
        Atom::OppOut(n) => g.opps(p).any(|q| super::plans::on_bf(g, q, n).is_some()),
        Atom::Have(n) => super::plans::on_bf(g, p, n).is_some() || pl.hand.iter().any(|&c| *g.db.get(c).name == **n),
        Atom::Not(b) => !eval(g, p, b, cx),
        Atom::Num(q, c, v) => {
            let x: f64 = q.iter().map(|q| quantity(g, p, *q, cx)).sum();
            let v = match v {
                Rhs::N(v) => *v,
                Rhs::OneLander => crate::impls::t6_keep::mana(g, p).need_one_lander(),
            };
            match c {
                Cmp::Ge => x >= v,
                Cmp::Le => x <= v,
                Cmp::Gt => x > v,
                Cmp::Lt => x < v,
                Cmp::Eq => x == v,
            }
        }
    }
}

// ------------------------------------------------------------------ the loaded plans
/// --- phase4-cedh: a line's group as the policy reads it: its cards (each with "held only on the battlefield"), the
/// conditions that hold it, and how many cards of it the line needs
#[derive(Debug, Clone, Default)]
pub struct Group {
    pub cards: Vec<(String, bool)>,
    pub conds: Vec<Cond>,
    pub need: usize,
}

/// a plan with its conditions parsed and its lines' groups resolved
pub struct Loaded {
    pub plan: Plan,
    /// per line: its groups' card names (a catalogue combo's, or its pieces) and its condition
    pub lines: Vec<(Vec<Vec<String>>, Cond)>,
    /// per line: its groups as the policy reads them (same order as `lines`)
    pub groups: Vec<Vec<Group>>,
    pub prio_when: Vec<Cond>,
    pub wish_when: Vec<Cond>,
    pub hold_up_when: Vec<Cond>,
    pub go_now_when: Vec<Cond>,
    pub mull_when: Vec<Cond>,
    /// the `[sets]`, by index (`Q::Hand`)
    pub sets: Vec<Vec<String>>,
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
    let set_names: Vec<&str> = plan.sets.keys().map(|k| k.as_str()).collect();
    let cond = |w: &[String], what: String| Cond::parse_with(w, &set_names).map_err(|e| err(format!("{what}: {e}")));
    let mut lines = vec![];
    let mut all_groups = vec![];
    for l in &plan.line {
        let raw: Vec<Vec<String>> = match (&l.combo, l.pieces.is_empty()) {
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
        if !l.counts.is_empty() && l.counts.len() != raw.len() {
            return Err(err(format!("line {:?}: {} counts for {} groups", l.name, l.counts.len(), raw.len())));
        }
        if l.counts.contains(&0) {
            return Err(err(format!("line {:?}: a count of 0", l.name)));
        }
        let mut names = vec![];
        let mut groups = vec![];
        for (k, g) in raw.iter().enumerate() {
            let mut grp = Group { need: l.counts.get(k).copied().unwrap_or(1), ..Group::default() };
            for e in g {
                if let Some(c) = e.strip_prefix('?') {
                    grp.conds.push(cond(&[c.to_string()], format!("line {:?}", l.name))?);
                } else if let Some(n) = e.strip_prefix("bf:") {
                    grp.cards.push((n.trim().to_string(), true));
                } else {
                    grp.cards.push((e.clone(), false));
                }
            }
            if grp.cards.is_empty() {
                return Err(err(format!("line {:?}: a group with no card", l.name)));
            }
            names.push(grp.cards.iter().map(|x| x.0.clone()).collect());
            groups.push(grp);
        }
        lines.push((names, cond(&l.when, format!("line {:?}", l.name))?));
        all_groups.push(groups);
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
        prio_when.push(cond(&r.when, format!("prio rule {}", i + 1))?);
    }
    let mut wish_when = vec![];
    for (i, r) in plan.wish.iter().enumerate() {
        if r.cards.is_empty() {
            return Err(err(format!("wish rule {}: no cards", i + 1)));
        }
        wish_when.push(cond(&r.when, format!("wish rule {}", i + 1))?);
    }
    let hold_up_when: Vec<Cond> = plan
        .hold_up
        .iter()
        .enumerate()
        .map(|(i, r)| cond(&r.when, format!("hold_up rule {}", i + 1)))
        .collect::<Result<_, _>>()?;
    let mut go_now_when = vec![];
    for (i, r) in plan.go_now.iter().enumerate() {
        if r.combos.is_empty() {
            return Err(err(format!("go_now rule {}: no combos", i + 1)));
        }
        if let Some(c) =
            r.combos.iter().find(|c| !crate::impls::combos::all().iter().any(|x| x.name.starts_with(&***c)))
        {
            return Err(err(format!("go_now rule {}: no catalogue combo starts with {c:?}", i + 1)));
        }
        go_now_when.push(cond(&r.when, format!("go_now rule {}", i + 1))?);
    }
    let mut mull_when = vec![];
    for (i, r) in plan.mulligan.iter().flat_map(|m| &m.rule).enumerate() {
        if r.when.is_empty() {
            return Err(err(format!("mulligan rule {}: no when", i + 1)));
        }
        mull_when.push(cond(&r.when, format!("mulligan rule {}", i + 1))?);
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
    let has_wish = !plan.line.is_empty() || !plan.engines.is_empty() || !plan.wish.is_empty();
    let cfg = super::plans::DeckConfig {
        style: plan.style.map(|s| Style { temp: s.temp, aggression: s.aggression, caution: s.caution }),
        key_cards: Box::leak(key_cards.into_boxed_slice()),
        wish: has_wish.then_some(derived_wish as super::plans::WishFn),
        prio_fn: (!plan.prio.is_empty()).then_some(rule_prio as super::plans::PlanPrioFn),
        cmd_prio: plan.commander.prio,
        cmd_turn: plan.commander.turn,
        aura_prio: plan.knobs.aura_prio,
        aura_host: plan.knobs.aura_host.as_deref().map(leak),
        top_pref: plan.knobs.top_pref.as_deref().map(leak),
        hooked_prio: plan.knobs.hooked_prio,
    };
    let sets = plan.sets.values().cloned().collect();
    Ok(Loaded {
        plan,
        lines,
        groups: all_groups,
        prio_when,
        wish_when,
        hold_up_when,
        go_now_when,
        mull_when,
        sets,
        cfg,
    })
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
pub fn active(g: &Game, p: PlayerId) -> Option<&'static Loaded> {
    let key = g.player(p).key;
    get(key).filter(|_| super::plans::plays_plan(key))
}

// ------------------------------------------------------------------ what the generic policy reads
fn name(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

/// how many cards of the group p holds: on the battlefield, and (unless a card counts only there) in hand, as a land
/// in play, as a commander in the command zone, or on top of its library where p put it itself; a condition of the
/// group that holds counts as the whole group
fn group_held(g: &Game, p: PlayerId, grp: &Group, top: &[&str], l: &Loaded) -> usize {
    if grp.conds.iter().any(|c| c.holds_cx(g, p, Cx { plan: Some(l), card: None })) {
        return grp.need;
    }
    let pl = g.player(p);
    grp.cards
        .iter()
        .map(|(n, bf_only)| {
            let bf = pl.perms.iter().filter(|&&m| !g.perm(m).phased && g.perm(m).cd.is_some_and(|c| name(g, c) == n));
            let mut k = bf.count();
            if !bf_only {
                k += pl.hand.iter().filter(|&&c| name(g, c) == n).count()
                    + pl.lands.iter().filter(|&&x| name(g, g.perm(x).card()) == n).count()
                    + pl.cmds_in_zone().filter(|&c| name(g, c) == n).count()
                    + top.iter().filter(|&&x| x == n).count();
            }
            k
        })
        .sum()
}

fn in_deck(g: &Game, p: PlayerId, n: &str) -> bool {
    let pl = g.player(p);
    pl.deck_names.iter().any(|&c| name(g, c) == n) || pl.commanders().any(|c| name(g, c) == n)
}

/// the lines' wanted library cards, closest line first: (distance, file order, cards). Distance = the cards the line
/// still needs (one per group, or the group's count), plus one when its condition fails
fn line_wants(l: &Loaded, g: &Game, p: PlayerId) -> Vec<(usize, usize, Vec<CardId>)> {
    let top: Vec<&str> = crate::engine::tutors::known_top(g, p).into_iter().map(|c| name(g, c)).collect();
    let lib = &g.player(p).library;
    let mut ranked: Vec<(usize, usize, Vec<CardId>)> = vec![];
    for (i, ((names, cond), groups)) in l.lines.iter().zip(&l.groups).enumerate() {
        if !names.iter().all(|grp| grp.iter().any(|n| in_deck(g, p, n))) {
            continue; // a group the list no longer plays (the plan test catches it first)
        }
        let miss: Vec<(&Vec<String>, usize)> = names
            .iter()
            .zip(groups)
            .map(|(n, grp)| (n, grp.need.saturating_sub(group_held(g, p, grp, &top, l))))
            .filter(|x| x.1 > 0)
            .collect();
        if miss.is_empty() {
            continue;
        }
        let dist = miss.iter().map(|x| x.1).sum::<usize>() + usize::from(!cond.holds(g, p));
        let wanted = if l.plan.line[i].ordered { &miss[..1] } else { &miss[..] };
        let want: Vec<CardId> = wanted
            .iter()
            .flat_map(|grp| grp.0.iter())
            .filter_map(|n| lib.iter().copied().find(|&c| name(g, c) == n))
            .collect();
        if !want.is_empty() {
            ranked.push((dist, i, want));
        }
    }
    ranked.sort_by_key(|x| (x.0, x.1));
    ranked
}

/// the bonus a wish rule's `plus_line` gives a card: `plus_line` / the groups still missing of the closest line holding
/// the card that is started (a group held) and not complete; groups are held as `group_held` counts them, without the
/// library top
fn line_bonus(l: &Loaded, g: &Game, p: PlayerId, n: &str, k: f64) -> f64 {
    let mut best = 0.0f64;
    for groups in &l.groups {
        if !groups.iter().any(|grp| grp.cards.iter().any(|x| x.0 == n)) {
            continue;
        }
        let held: Vec<usize> = groups.iter().map(|grp| group_held(g, p, grp, &[], l)).collect();
        let miss = groups.iter().zip(&held).filter(|(grp, h)| **h < grp.need).count();
        if miss > 0 && held.iter().any(|h| *h > 0) {
            best = best.max(k / miss as f64);
        }
    }
    best
}

/// the wish rules' library cards, best score first (ties in library order); a card's score is the first rule
/// selecting it whose `when` holds, plus its line bonus, and counts above `min_score` (default 0)
fn rule_wants(l: &Loaded, g: &Game, p: PlayerId) -> Vec<CardId> {
    let mut v: Vec<(f64, CardId)> = vec![];
    for &c in &g.player(p).library {
        let n = name(g, c);
        let r = l
            .plan
            .wish
            .iter()
            .zip(&l.wish_when)
            .find(|(r, w)| r.cards.iter().any(|x| x == n) && w.holds_cx(g, p, Cx { plan: Some(l), card: Some(c) }))
            .map(|x| x.0);
        let Some(r) = r else { continue };
        let mut s = r.score;
        if s > 0.0
            && let Some(k) = r.plus_line
        {
            s += line_bonus(l, g, p, n, k);
        }
        if s > r.min_score.unwrap_or(0.0) {
            v.push((s, c));
        }
    }
    v.sort_by(|a, b| b.0.total_cmp(&a.0));
    v.into_iter().map(|x| x.1).collect()
}

/// the derived wish list of a plan, without the mana step: the wish rules' cards, then the lines (the engines ahead
/// before the first line two or more steps away), then the engines
pub fn wish_for(l: &Loaded, g: &Game, p: PlayerId) -> Vec<CardId> {
    let pl = g.player(p);
    let lib = &pl.library;
    let mut out: Vec<CardId> = rule_wants(l, g, p);
    let have = |c: CardId| pl.hand.contains(&c) || pl.perms.iter().any(|&m| g.perm(m).cd == Some(c));
    let push_engines = |out: &mut Vec<CardId>, names: &[String]| {
        for n in names {
            if let Some(c) = lib.iter().copied().find(|&c| name(g, c) == n)
                && !have(c)
                && !out.contains(&c)
            {
                out.push(c);
            }
        }
    };
    let mut ahead = false;
    for (dist, _, want) in line_wants(l, g, p) {
        if dist >= 2 && !ahead {
            push_engines(&mut out, &l.plan.engines_ahead);
            ahead = true;
        }
        for c in want {
            if !out.contains(&c) {
                out.push(c);
            }
        }
    }
    if !ahead {
        push_engines(&mut out, &l.plan.engines_ahead);
    }
    push_engines(&mut out, &l.plan.engines);
    out
}

/// The derived tutor priority (replaces a hand wish list): the mana a line complete but for its mana needs; then the
/// wish rules' cards; then, line by line from the closest (cards missing, one more when its condition fails; ties in
/// file order), the library cards of its missing groups (an ordered line: of its first missing group only), the
/// engines ahead before the first line two or more steps away; then the engines.
pub fn derived_wish(g: &Game, p: PlayerId) -> Vec<CardId> {
    let Some(l) = active(g, p) else { return crate::cardcode::missing_pieces(g, p) };
    let mana = crate::impls::combos::mana_for_lines(g, p);
    if !mana.is_empty() {
        return mana;
    }
    wish_for(l, g, p)
}

/// the plan's cast-priority rules for card c (see `PrioRule`), on the 0-100 scale
pub fn prio_for(l: &Loaded, g: &Game, p: PlayerId, c: CardId) -> Option<f64> {
    let pl = g.player(p);
    let d = g.db.get(c);
    let cx = Cx { plan: Some(l), card: Some(c) };
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
            let on_bf = |grp: &Group| group_held(g, p, &Group { conds: vec![], ..bf_only(grp) }, &[], l) >= grp.need;
            let Some(k) = l.groups[i].iter().position(|grp| !on_bf(grp)) else { continue };
            if !l.groups[i][k].cards.iter().any(|x| *x.0 == *d.name) {
                continue;
            }
        }
        if when.holds_cx(g, p, cx) {
            let mut v = r.value + r.plus_mv.map_or(0, |k| (d.cmc as i32).min(k)) as f64;
            if let Some(t) = r.per_tax {
                v += t * pl.tax_of(c) as f64;
            }
            return Some(v);
        }
        if let Some(v) = r.otherwise {
            return Some(v);
        }
    }
    None
}

/// a group counted on the battlefield only
fn bf_only(grp: &Group) -> Group {
    Group { cards: grp.cards.iter().map(|x| (x.0.clone(), true)).collect(), conds: vec![], need: grp.need }
}

/// the plan's cast-priority rules (replaces a hand priority function): see `PrioRule`
pub fn rule_prio(g: &Game, p: PlayerId, c: CardId) -> Option<i32> {
    prio_for(active(g, p)?, g, p, c).map(|v| v as i32)
}

/// a hold of p's plan protects this permanent's card (`pool::worth_protecting`)
pub fn protects(g: &Game, p: PlayerId, card: &str) -> bool {
    active(g, p).is_some_and(|l| l.plan.hold.iter().any(|h| h.protects.iter().any(|x| x == card)))
}

/// the plan's keep rule for p's current hand (None: no mulligan section)
pub fn keep_for(l: &Loaded, g: &Game, p: PlayerId) -> Option<bool> {
    let m = l.plan.mulligan.as_ref()?;
    if m.fast_mana && crate::impls::t6_keep::mana(g, p).settle() {
        return Some(true);
    }
    if !m.rule.is_empty() {
        let cx = Cx { plan: Some(l), card: None };
        let row = m.rule.iter().zip(&l.mull_when).find(|(_, w)| w.holds_cx(g, p, cx));
        return Some(row.map_or(m.otherwise.unwrap_or(true), |r| r.0.keep));
    }
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

/// the plan's keep rule for p's current hand (None: no mulligan section, the generic rule decides)
pub fn keeps_hand(g: &Game, p: PlayerId) -> Option<bool> {
    keep_for(active(g, p)?, g, p)
}

/// --- phase4-cedh: the hold-up utility of the plan (first rule that holds), None: no opinion
pub fn hold_up_for(l: &Loaded, g: &Game, p: PlayerId) -> Option<f64> {
    let cx = Cx { plan: Some(l), card: None };
    l.plan.hold_up.iter().zip(&l.hold_up_when).find(|(_, w)| w.holds_cx(g, p, cx)).and_then(|x| x.0.value)
}

/// --- phase4-cedh: the plan's go-now gate for this catalogue combo, None: the shared table check decides
pub fn go_now_for(l: &Loaded, g: &Game, p: PlayerId, combo: &str) -> Option<bool> {
    let cx = Cx { plan: Some(l), card: None };
    l.plan
        .go_now
        .iter()
        .zip(&l.go_now_when)
        .find(|(r, w)| r.combos.iter().any(|c| combo.starts_with(&**c)) && w.holds_cx(g, p, cx))
        .and_then(|x| x.0.go)
}

/// --- phase4-cedh: a Tier 6 deck playing from its plan file: the five plan slots (`t6_plans.rs`) read from it
pub mod t6 {
    use super::*;

    pub fn prio(l: &Loaded, g: &Game, p: PlayerId, c: CardId) -> Option<f64> {
        prio_for(l, g, p, c)
    }

    /// the wish rules and the lines (no mana step); nothing wanted: no opinion (the closest combo's missing pieces)
    pub fn wish(l: &Loaded, g: &Game, p: PlayerId) -> Option<Vec<CardId>> {
        Some(wish_for(l, g, p)).filter(|v| !v.is_empty())
    }

    pub fn keep(l: &Loaded, g: &Game, p: PlayerId) -> Option<bool> {
        keep_for(l, g, p)
    }

    pub fn hold_up(l: &Loaded, g: &Game, p: PlayerId) -> Option<f64> {
        hold_up_for(l, g, p)
    }

    pub fn go_now(l: &Loaded, g: &Game, p: PlayerId, combo: &str) -> Option<bool> {
        go_now_for(l, g, p, combo)
    }
}

/// every card name a plan file names (for the plan test: each must be in the deck)
pub fn named_cards(l: &Loaded) -> Vec<String> {
    let p = &l.plan;
    let mut v: Vec<String> = p.key_cards.keys().cloned().collect();
    for (i, x) in p.line.iter().enumerate() {
        if x.combo.is_none() {
            v.extend(l.lines[i].0.iter().flatten().cloned());
        }
        v.extend(l.lines[i].1.names().map(String::from));
        for grp in &l.groups[i] {
            v.extend(grp.conds.iter().flat_map(|c| c.names()).map(String::from));
        }
    }
    v.extend(p.engines.iter().cloned());
    v.extend(p.engines_ahead.iter().cloned());
    v.extend(p.sets.values().flatten().cloned());
    for (r, w) in p.prio.iter().zip(&l.prio_when) {
        v.extend(r.cards.iter().cloned());
        v.extend(w.names().map(String::from));
    }
    for (r, w) in p.wish.iter().zip(&l.wish_when) {
        v.extend(r.cards.iter().cloned());
        v.extend(w.names().map(String::from));
    }
    for w in l.hold_up_when.iter().chain(&l.go_now_when).chain(&l.mull_when) {
        v.extend(w.names().map(String::from));
    }
    if let Some(m) = &p.mulligan {
        v.extend(m.rituals.iter().cloned());
    }
    for h in &p.hold {
        v.push(h.card.clone());
        v.extend(h.protects.iter().filter(|x| *x != "commander").cloned());
    }
    v
}
