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
const CONFIG: [(&str, DeckConfig); 2] = [
    (
        "gaa-azorius-stax-control",
        DeckConfig { prio_fn: Some(gaa_prio), wish: Some(gaa_wish), style: style(1.0, 0.35, 0.9), ..NONE },
    ),
    (
        "heliod-mono-white-stax",
        DeckConfig { prio_fn: Some(heliod_prio), wish: Some(heliod_wish), style: style(1.0, 0.5, 0.8), ..NONE },
    ),
];

fn hand_config(key: &str) -> Option<&'static DeckConfig> {
    CONFIG.iter().find(|x| x.0 == key).map(|x| &x.1)
}

/// --- phase4-plans: the configuration each deck with a plan file (`data/plans/<key>.toml`) plays from when the plan
/// is on (`plan_data::plan_on`: env AI_PLAN_DATA, else the file's `default`): its static knobs, its derived wish list
/// and priority rules, and the hand wish / priority functions where the file has none yet
fn effective() -> &'static std::collections::HashMap<&'static str, DeckConfig> {
    static E: std::sync::LazyLock<std::collections::HashMap<&'static str, DeckConfig>> =
        std::sync::LazyLock::new(|| {
            super::plan_data::all()
                .iter()
                .filter(|(k, _)| {
                    super::plan_data::plan_on(k, hand_config(k).is_some() || crate::impls::t6_plans::has_hand(k))
                })
                .map(|(k, l)| {
                    let mut c = l.cfg;
                    if let Some(h) = hand_config(k) {
                        c.wish = c.wish.or(h.wish);
                        c.prio_fn = c.prio_fn.or(h.prio_fn);
                    }
                    if crate::impls::t6_plans::is_t6_key(k) {
                        // --- phase4-cedh: a Tier 6 plan is read through the five plan slots (`t6_plans.rs`); the
                        // shared fallbacks (closest combo's pieces, generic priority) stay as they were
                        c.wish = None;
                        c.prio_fn = None;
                    }
                    (*k, c)
                })
                .collect()
        });
    &E
}

/// --- phase4-plans: the deck plays from its plan file
pub fn plays_plan(key: &str) -> bool {
    effective().contains_key(key)
}

/// the deck's configuration (an empty one for a deck without an entry): from its plan file when that is on, else
/// its hand entry
pub fn config(key: &str) -> &'static DeckConfig {
    if let Some(c) = effective().get(key) {
        return c;
    }
    hand_config(key).unwrap_or(&NONE)
}

/// what this deck's tutors want: its configured list, else the pieces of its closest combo
pub fn wish_raw(g: &Game, p: PlayerId) -> Vec<CardId> {
    if let Some(w) = crate::impls::t6_plans::wish(g, p) {
        return w;
    }
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

/// combos.DRAIN_PAYOFFS
pub const DRAIN_PAYOFFS: [&str; 13] = [
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
    // --- Tiers 4/5 (urza-yawgmoth): each opponent loses 1 as your creatures die
    "The Meathook Massacre",
    "Marionette Apprentice",
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

pub fn opp_power(g: &Game, p: PlayerId) -> i32 {
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

/// Ad Nauseam's threat floor: the largest opposing board plus half of the others' (more than one opponent can swing
/// at you before your next turn), plus 6, and never under 10
pub fn adnaus_floor(g: &Game, p: PlayerId) -> i32 {
    let mut boards: Vec<i32> = g
        .opps(p)
        .map(|q| {
            g.player(q).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).map(|&m| epow(g, m)).sum()
        })
        .collect();
    boards.sort_unstable_by(|a, b| b.cmp(a));
    let rest: i32 = boards.iter().skip(1).sum();
    10.max(boards.first().copied().unwrap_or(0) + rest / 2 + 6)
}

/// the life Ad Nauseam costs per card: the average mana value of the library (lands are free)
pub fn adnaus_life_per_card(g: &Game, p: PlayerId) -> f64 {
    let lib = &g.player(p).library;
    if lib.is_empty() {
        return 2.5;
    }
    let total: f64 = lib.iter().map(|&c| g.db.get(c).cmc as f64).sum();
    (total / lib.len() as f64).max(0.5)
}

/// the hand Ad Nauseam should stop at: seven (the cleanup discards the rest) plus what next turn's mana casts,
/// about one card per three mana (this turn's lands and a land drop)
pub fn adnaus_hand_cap(g: &Game, p: PlayerId) -> usize {
    7 + (g.player(p).lands.len() + 1) / 3
}

/// Ad Nauseam: what it's worth, from the cards the life above the threat floor buys; worth it for four or more.
/// When to cast it (the end of the turn before yours, with room in the hand) is `adnaus_cast_now`.
pub fn adnaus_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let cards = (g.player(p).life - adnaus_floor(g, p) - 7) as f64 / adnaus_life_per_card(g, p);
    if cards < 4.0 { 0 } else { 80.0f64.min(40.0 + 4.0 * cards) as i32 }
}

/// Ad Nauseam now: at the end of the turn before yours (your mana untaps for the cards it finds), with room for four
/// cards under the hand cap (the cast itself leaves the hand)
pub fn adnaus_cast_now(g: &Game, p: PlayerId) -> bool {
    let room = (adnaus_hand_cap(g, p) + 1).saturating_sub(g.player(p).hand.len());
    // the sheoldred deck taps out on its own turn, so it never holds 5 mana for the opposing end step; its own second
    // main phase (everything better already cast) is when the mana is idle
    if g.player(p).key == "sheoldred" && g.active == Some(p) && g.step == crate::state::Step::Main2 && room >= 4 {
        return true;
    }
    g.active != Some(p) && room >= 4
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
    let mut k = g.opps(p).filter(|&q| g.player(q).any_cmd_in_zone()).count() as i32;
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
