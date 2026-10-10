//! Per-deck AI configuration and plans for the outside decks (Python's `ai/pool_decks.py`, `ai/deck_plans.py`), and
//! the Game Changers' cast priorities computed from the game (`ai/gc_prio.py`).
//!
//! A deck without an entry plays with the default style and the generic tag priority. A plan's priority function
//! returns a 0-90 priority, or None to fall back to the generic one.

use super::Style;
use crate::cards::Types;
use crate::engine::mana::total_mana;
use crate::engine::values::{commander_out, epow, pval};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;

pub type WishFn = fn(&Game, PlayerId) -> Vec<CardId>;
pub type PlanPrioFn = fn(&Game, PlayerId, CardId) -> Option<i32>;

/// pool_decks.CONFIG entry
#[derive(Debug, Clone, Copy, Default)]
pub struct DeckConfig {
    /// play style for the heuristic AI (aggression, caution, temp)
    pub style: Option<Style>,
    /// card name -> cast priority, overriding the generic one
    pub key_cards: &'static [(&'static str, i32)],
    /// tutor wish list (the first card still in the library and not in hand or play is fetched)
    pub wish: Option<WishFn>,
    pub prio_fn: Option<PlanPrioFn>,
    /// commander cast priority (default 75, 85 with card code) and the earliest turn to cast it (default 2)
    pub cmd_prio: Option<i32>,
    pub cmd_turn: Option<u32>,
    pub aura_prio: Option<i32>,
    /// where the deck's Auras go ('commander')
    pub aura_host: Option<&'static str>,
    /// how the deck stacks the top of its library ('mv': biggest mana value first)
    pub top_pref: Option<&'static str>,
    pub hooked_prio: Option<i32>,
}

const NONE: DeckConfig = DeckConfig {
    style: None,
    key_cards: &[],
    wish: None,
    prio_fn: None,
    cmd_prio: None,
    cmd_turn: None,
    aura_prio: None,
    aura_host: None,
    top_pref: None,
    hooked_prio: None,
};

const fn style(temp: f64, aggression: f64, caution: f64) -> Option<Style> {
    Some(Style { temp, aggression, caution })
}

/// pool_decks.CONFIG
const CONFIG: [(&str, DeckConfig); 7] = [
    (
        "yuriko-dimir-ninjas",
        DeckConfig {
            cmd_prio: Some(15),
            cmd_turn: Some(6),
            top_pref: Some("mv"),
            wish: Some(crate::cardcode::yuriko_wish),
            prio_fn: Some(crate::cardcode::yuriko_prio),
            style: style(1.0, 0.8, 0.5),
            ..NONE
        },
    ),
    (
        "chulane-bant-value-combo",
        DeckConfig { prio_fn: Some(chulane_prio), wish: Some(chulane_wish), style: style(1.0, 0.5, 0.7), ..NONE },
    ),
    ("yawgmoth-mono-black-aristocrats", DeckConfig { wish: Some(yawg_wish), prio_fn: Some(yawg_prio), ..NONE }),
    (
        "gaa-azorius-stax-control",
        DeckConfig { prio_fn: Some(gaa_prio), wish: Some(gaa_wish), style: style(1.0, 0.35, 0.9), ..NONE },
    ),
    (
        "heliod-mono-white-stax",
        DeckConfig { prio_fn: Some(heliod_prio), wish: Some(heliod_wish), style: style(1.0, 0.5, 0.8), ..NONE },
    ),
    (
        "kaalia-mardu-creature-cheat",
        DeckConfig {
            prio_fn: Some(crate::cardcode::kaalia_prio),
            cmd_prio: Some(88),
            style: style(1.0, 0.8, 0.4),
            ..NONE
        },
    ),
    (
        "light-paws-aura-voltron",
        DeckConfig {
            aura_host: Some("commander"),
            aura_prio: Some(70),
            cmd_prio: Some(88),
            cmd_turn: Some(1),
            style: style(1.0, 0.8, 0.4),
            ..NONE
        },
    ),
];

/// the deck's configuration (an empty one for a deck without an entry)
pub fn config(key: &str) -> &'static DeckConfig {
    CONFIG.iter().find(|x| x.0 == key).map_or(&NONE, |x| &x.1)
}

/// cards a deck's AI casts only in response, never for their ability-language value (Python's
/// `CI.RESPONSE_ONLY`). PORT(phase 6): Jodah's protection spells.
pub fn response_only(_key: &str, _c: CardId) -> bool {
    false
}

/// what this deck's tutors want: its configured list, else the pieces of its closest combo
pub fn wish_raw(g: &Game, p: PlayerId) -> Vec<CardId> {
    match config(g.player(p).key).wish {
        Some(f) => f(g, p),
        None => crate::cardcode::missing_pieces(g, p),
    }
}

/// pool_ai.wish_list (your decks have none)
pub fn wish_list(g: &Game, p: PlayerId) -> Vec<CardId> {
    if super::is_main(g.player(p).key) {
        return vec![];
    }
    wish_raw(g, p)
}

fn ids(g: &Game, names: &[&str]) -> Vec<CardId> {
    names.iter().filter_map(|n| g.db.id(n)).collect()
}

// ------------------------------------------------------------------ deck_plans helpers
/// deck_plans.on_bf: p's permanent of this name on the battlefield, not phased out
pub fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        !x.phased && x.cd.is_some_and(|c| &*g.db.get(c).name == name)
    })
}

fn board(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).count()
}

/// combos.DRAIN_PAYOFFS
pub const DRAIN_PAYOFFS: [&str; 11] = [
    "Blood Artist",
    "Zulaport Cutthroat",
    "Cruel Celebrant",
    "Bastion of Remembrance",
    "Falkenrath Noble",
    "Mayhem Devil",
    "Syr Konrad, the Grim",
    "Vindictive Vampire",
    "Poison-Tip Archer",
    "Goblin Bombardment",
    "Elas il-Kor, Sadistic Pilgrim",
];

/// combos.drain_payoff: p has a death-drain payoff out
pub fn drain_payoff(g: &Game, p: PlayerId) -> Option<PermId> {
    use crate::tag::Tag;
    DRAIN_PAYOFFS.iter().find_map(|n| on_bf(g, p, n)).or_else(|| {
        g.player(p)
            .perms
            .iter()
            .copied()
            .find(|&m| g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Bartist) || g.db.get(c).tag(Tag::Drain)))
    })
}

// ------------------------------------------------------------------ Chulane, Teller of Tales
const BOUNCERS: [&str; 4] = ["Whitemane Lion", "Shrieking Drake", "Kor Skyfisher", "Man-o'-War"];

/// deck_plans.chulane_prio: with Chulane out every creature spell draws and ramps: creatures first, cheap
/// self-bouncers recast; the finishers once the board or the land count can end the game
fn chulane_prio(g: &Game, p: PlayerId, c: CardId) -> Option<i32> {
    let pl = g.player(p);
    let d = g.db.get(c);
    let chulane = pl.perms.iter().any(|&m| g.perm(m).is_cmd);
    match &*d.name {
        "Craterhoof Behemoth" => return Some(if board(g, p) >= 6 { 80 } else { 20 }),
        "Avenger of Zendikar" => return Some(if pl.lands.len() >= 7 { 78 } else { 45 }),
        _ => {}
    }
    if chulane && d.creature {
        if BOUNCERS.contains(&&*d.name) {
            return Some(76);
        }
        return Some(70 + (d.cmc as i32).min(8));
    }
    if &*d.name == "Aluren" {
        return Some(if chulane || pl.cmd_in_zone { 75 } else { 40 });
    }
    None
}

fn chulane_wish(g: &Game, p: PlayerId) -> Vec<CardId> {
    let miss = crate::cardcode::missing_pieces(g, p);
    if board(g, p) >= 6 {
        let mut v = ids(g, &["Craterhoof Behemoth"]);
        v.extend(miss);
        return v;
    }
    if !miss.is_empty() {
        return miss;
    }
    ids(g, &["Avenger of Zendikar", "Consecrated Sphinx", "Craterhoof Behemoth"])
}

// ------------------------------------------------------------------ Grand Arbiter Augustin IV (stax)
const STAX_EARLY: [&str; 14] = [
    "Sphere of Resistance",
    "Thorn of Amethyst",
    "Thalia, Guardian of Thraben",
    "Rule of Law",
    "Drannith Magistrate",
    "Deafening Silence",
    "Ethersworn Canonist",
    "Archon of Emeria",
    "Glowrider",
    "Vryn Wingmare",
    "Lavinia, Azorius Renegade",
    "Spirit of the Labyrinth",
    "Thalia, Heretic Cathar",
    "Grand Abolisher",
];
const PILLOW: [&str; 2] = ["Ghostly Prison", "Propaganda"];

fn opp_power(g: &Game, p: PlayerId) -> i32 {
    g.opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && !g.perm(m).phased)
        .map(|m| epow(g, m))
        .sum()
}

/// deck_plans.gaa_prio: rocks, then the taxes and spell limits while opponents are still developing; pillowfort as
/// soon as creatures show up; the commander (a tax piece that also discounts your spells) early
fn gaa_prio(g: &Game, p: PlayerId, c: CardId) -> Option<i32> {
    let pl = g.player(p);
    let d = g.db.get(c);
    if c == pl.cmd {
        return Some(if pl.turns >= 3 { 84 } else { 0 });
    }
    if d.tag(crate::tag::Tag::Rock) {
        return Some(if pl.turns <= 5 { 86 } else { 45 });
    }
    if PILLOW.contains(&&*d.name) {
        return Some(60 + ((opp_power(g, p) as f64 * 1.5) as i32).min(25));
    }
    if STAX_EARLY.contains(&&*d.name) {
        return Some(if pl.turns <= 5 { 76 } else { 55 });
    }
    None
}

fn gaa_wish(g: &Game, p: PlayerId) -> Vec<CardId> {
    if opp_power(g, p) >= 10 && !PILLOW.iter().any(|n| on_bf(g, p, n).is_some()) {
        return ids(g, &PILLOW);
    }
    let miss = crate::cardcode::missing_pieces(g, p);
    if miss.is_empty() { ids(g, &PILLOW) } else { miss }
}

// ------------------------------------------------------------------ Heliod, Sun-Crowned (mono-white stax)
/// deck_plans.heliod_prio: rocks and taxes early like GAA; Heliod on turn three; Walking Ballista is kept for the
/// combo; the protection creatures once Heliod or a stax piece needs guarding
fn heliod_prio(g: &Game, p: PlayerId, c: CardId) -> Option<i32> {
    let pl = g.player(p);
    let d = g.db.get(c);
    if c == pl.cmd {
        return Some(if pl.turns >= 3 { 84 } else { 0 });
    }
    if &*d.name == "Walking Ballista" {
        return Some(0);
    }
    if d.tag(crate::tag::Tag::Rock) {
        return Some(if pl.turns <= 5 { 86 } else { 45 });
    }
    if STAX_EARLY.contains(&&*d.name) {
        return Some(if pl.turns <= 6 { 76 } else { 55 });
    }
    if PILLOW.contains(&&*d.name) {
        return Some(60 + ((opp_power(g, p) as f64 * 1.5) as i32).min(25));
    }
    if matches!(&*d.name, "Mother of Runes" | "Giver of Runes") {
        return Some(70);
    }
    None
}

fn heliod_wish(g: &Game, p: PlayerId) -> Vec<CardId> {
    let ballista = g.db.id("Walking Ballista");
    if on_bf(g, p, "Walking Ballista").is_none() && !ballista.is_some_and(|b| g.player(p).hand.contains(&b)) {
        return ids(g, &["Walking Ballista", "Recruiter of the Guard", "Ranger-Captain of Eos"]);
    }
    if opp_power(g, p) >= 10 && !PILLOW.iter().any(|n| on_bf(g, p, n).is_some()) {
        return ids(g, &PILLOW);
    }
    let miss = crate::cardcode::missing_pieces(g, p);
    if !miss.is_empty() {
        return miss;
    }
    ids(g, &["Rule of Law", "Smothering Tithe", "Drannith Magistrate", "Thalia, Guardian of Thraben"])
}

// ------------------------------------------------------------------ Yawgmoth, Thran Physician
const UNDYING: [&str; 4] = ["Geralf's Messenger", "Butcher Ghoul", "Young Wolf", "Nether Traitor"];

fn undying_count(g: &Game, p: PlayerId) -> usize {
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| g.is_creature(m) && g.perm(m).cd.is_some_and(|c| g.db.get(c).has_kw("undying")))
        .count()
}

/// deck_plans.yawg_prio: with Yawgmoth out, the loop comes first: Mikaeus (or undying creatures), then a death
/// payoff once the loop is there; Yawgmoth itself as soon as it can be cast
fn yawg_prio(g: &Game, p: PlayerId, c: CardId) -> Option<i32> {
    let d = g.db.get(c);
    if c == g.player(p).cmd {
        return Some(86);
    }
    on_bf(g, p, "Yawgmoth, Thran Physician")?;
    let und = undying_count(g, p);
    let mik = on_bf(g, p, "Mikaeus, the Unhallowed").is_some();
    let lp = und >= 2 || mik;
    if &*d.name == "Mikaeus, the Unhallowed" && !mik {
        return Some(82);
    }
    if UNDYING.contains(&&*d.name) && !lp {
        return Some(if und == 1 { 78 } else { 72 });
    }
    if lp && DRAIN_PAYOFFS.contains(&&*d.name) && drain_payoff(g, p).is_none() {
        return Some(80);
    }
    None
}

/// deck_plans.yawg_wish: the loop needs Yawgmoth plus two undying creatures (or Mikaeus and fodder), then a drain
/// payoff
fn yawg_wish(g: &Game, p: PlayerId) -> Vec<CardId> {
    let pl = g.player(p);
    let und = undying_count(g, p);
    let mik = on_bf(g, p, "Mikaeus, the Unhallowed").is_some()
        || pl.hand.iter().any(|&c| &*g.db.get(c).name == "Mikaeus, the Unhallowed");
    let mut out = vec![];
    if on_bf(g, p, "Yawgmoth, Thran Physician").is_none() && !pl.cmd_in_zone {
        out.extend(ids(g, &["Yawgmoth, Thran Physician"]));
    }
    if !mik && und < 2 {
        out.extend(ids(g, &["Mikaeus, the Unhallowed"]));
        out.extend(ids(g, &UNDYING));
    }
    if drain_payoff(g, p).is_none() {
        out.extend(ids(g, &DRAIN_PAYOFFS));
    }
    out.extend(crate::cardcode::missing_pieces(g, p));
    out
}

// ------------------------------------------------------------------ gc_prio: Game Changers from the game
/// gc_prio.life_margin: life above what the table could hit you for
pub fn life_margin(g: &Game, p: PlayerId) -> i32 {
    g.player(p).life - super::necro_floor(g, p)
}

/// the printed cost (cost_of's reductions value graveyard cards with card_worth, which can call these back)
fn mv(g: &Game, c: CardId) -> i32 {
    let d = g.db.get(c);
    (d.generic as usize + d.pips.len()) as i32
}

/// gc_prio.unlocks: would casting c (a mana source adding `gain` this turn) let you cast a spell from hand you can't
/// now?
fn unlocks(g: &Game, p: PlayerId, c: CardId, gain: i32) -> bool {
    let avail = total_mana(g, p, false) as i32;
    let after = avail - mv(g, c) + gain;
    g.player(p).hand.iter().any(|&x| x != c && !g.db.get(x).land && avail < mv(g, x) && mv(g, x) <= after)
}

/// Bolas's Citadel: worth it with life to spend above the threat floor
pub fn citadel_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let m = life_margin(g, p);
    if m < 8 { 0 } else { 80.min(40 + 2 * m) }
}

/// the life Citadel keeps when casting off the top
pub fn citadel_floor(g: &Game, p: PlayerId) -> i32 {
    10.max(super::necro_floor(g, p))
}

/// Ad Nauseam: about 2.5 life per card; worth it for four or more cards
pub fn adnaus_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let cards = life_margin(g, p) as f64 / 2.5;
    if cards < 4.0 { 0 } else { 80.0f64.min(40.0 + 4.0 * cards) as i32 }
}

/// The One Ring: a turn of protection matters most when the table could kill you; the burden costs life every upkeep
pub fn one_ring_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let mut v = 55;
    if life_margin(g, p) < 6 {
        v += 15;
    }
    if g.player(p).life <= 10 {
        v -= 20;
    }
    v.clamp(20, 80)
}

/// Rhystic Study: a tax on every opponent's spell, worth more with more opponents and the earlier it lands
pub fn rhystic_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let n = g.opps(p).count() as i32;
    let early = (10 - g.round as i32).max(0);
    80.min(30 + 6 * n + 2 * early)
}

/// Jeska's Will: {R} per card in the biggest opposing hand (net of its own 3) and, with your commander out, three
/// exiled cards to play this turn; the mana counts only if a spell in hand can use it
pub fn jeska_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let most = g.opps(p).map(|q| g.player(q).hand.len() as i32).max().unwrap_or(0);
    let mut v = 25 + if commander_out(g, p) { 12 } else { 0 };
    let gain = most - 3;
    if gain >= 2 && unlocks(g, p, c, most) {
        v += 6 * gain.min(5);
    }
    v.min(80)
}

/// a fast-mana source (Chrome Mox, Mox Diamond, Grim Monolith): first when its mana casts something now
pub fn ramp_prio(g: &Game, p: PlayerId, c: CardId, gain: i32) -> i32 {
    if unlocks(g, p, c, gain) {
        return 82;
    }
    if g.player(p).turns <= 5 { 70 } else { 30 }
}

/// Seedborn Muse: untapping on every other turn pays for instants, abilities and rocks that don't untap
pub fn seedborn_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    use crate::tag::Tag;
    let pl = g.player(p);
    let mut k = pl.hand.iter().filter(|&&x| g.db.get(x).instant || g.db.get(x).tag(Tag::Flash)).count() as i32;
    k += pl.perms.iter().filter(|&&m| g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Nountap))).count() as i32;
    70.min(40 + 6 * k)
}

/// Braids, Cabal Minion: everyone sacrifices each upkeep; good with spare permanents when the opponents have none
pub fn braids_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let spare = |q: PlayerId| {
        g.player(q).perms.iter().filter(|&&m| !g.perm(m).phased && (g.perm(m).token || g.perm(m).undying)).count()
    };
    let mine = spare(p) as i32;
    let theirs = g
        .opps(p)
        .map(|q| {
            g.player(q)
                .perms
                .iter()
                .filter(|&&m| {
                    !g.perm(m).phased
                        && (g.is_creature(m) || g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT)))
                })
                .count() as i32
        })
        .min()
        .unwrap_or(0);
    (30 + 8 * mine.min(4) - 3 * theirs.min(6)).clamp(15, 75)
}

/// Aura Shards: each creature you play destroys an artifact or enchantment
pub fn shards_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let tg: f64 = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| {
            !g.perm(m).phased
                && g.perm(m).cd.is_some_and(|c| {
                    let t = g.db.get(c).types;
                    t.has(Types::ARTIFACT) || t.has(Types::ENCHANTMENT)
                })
        })
        .map(|m| pval(g, m))
        .sum();
    let soon = g.player(p).hand.iter().filter(|&&x| g.db.get(x).creature).count() as f64;
    (30.0 + 3.0 * tg.min(12.0) + 3.0 * soon.min(3.0)).clamp(20.0, 75.0) as i32
}

/// Drannith Magistrate: worth the opponents' commanders still in the command zone and their graveyard engines
pub fn drannith_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    use crate::tag::Tag;
    let mut k = g.opps(p).filter(|&q| g.player(q).cmd_in_zone).count() as i32;
    k += g
        .opps(p)
        .filter(|&q| g.player(q).perms.iter().any(|&m| g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Breach))))
        .count() as i32;
    76.min(40 + 10 * k)
}

/// Opposition Agent: you control their searches; worth more against decks that tutor
pub fn agent_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    use crate::tag::Tag;
    let n = g
        .opps(p)
        .filter(|&q| {
            let ql = g.player(q);
            ql.library.iter().chain(ql.hand.iter()).any(|&x| g.db.get(x).tag(Tag::Tut) || g.db.get(x).tag(Tag::Seal))
        })
        .count() as i32;
    70.min(40 + 8 * n)
}
