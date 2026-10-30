//! Counters the Tier 6 report reads (`commander-sim --calibrate within --pool t6`). They are plain additions to a
//! player's `stats` map, made while the game is played: nothing here reads the random generator or changes a choice,
//! so Tier 0-5 games (whose players have `t6 == false`) are untouched and play identically.
//!
//! Key formats (fields separated by `|`, the player's own stats):
//! - `ix|counter|<counter card>|<victim deck key>`: a counterspell of this player resolved on that player's spell or ability
//! - `ix|removal|<spell or "ability">|<victim deck key>`: this player removed a permanent of that player
//! - `ix|wipe|<card>`: a board wipe this player cast resolved
//! - `ix|stax|<card>`: a stax piece this player cast
//! - `go|<combo>`: this player went for the combo (once per attempt)
//! - `win|<combo>`: the game ended on that combo's finish
//! - `stop|<combo>|<card>`: the line was stopped by that card (`?` when the framework could not say which)
//! - `mull_n`: mulligans taken (0-3)

use crate::ids::{CardId, PlayerId};
use crate::state::Game;
use crate::sym::intern;

/// the deck keys of the Tier 6 pool, Godo (retired 2026-10-25, a test seat), JD's Thanos, and Scion (retired
/// 2026-10-30, a test seat)
pub const T6_KEYS: [&str; 9] = [
    "dack-fayden-white-stax",
    "kinnan-bonder-mana-combo",
    "kraum-tymna-blue-breach-midrange",
    "rograkh-silas-turbo-breach",
    "rograkh-thrasios-creature-combo",
    "sisay-legends-toolbox",
    "godo",
    "thanos", // --- thanos: JD's Thanos, the Mad Titan turbo deck (Scion's seat, 2026-10-30)
    "scion",  // --- scion: JD's Scion of the Ur-Dragon deck (retired 2026-10-30, a test seat of its card code)
];

/// is this deck key a Tier 6 deck (pool or Godo)? The one place the AI and the counters ask.
pub fn is_t6_key(key: &str) -> bool {
    T6_KEYS.contains(&key)
}

/// is p a Tier 6 player?
pub fn is_t6(g: &Game, p: PlayerId) -> bool {
    g.player(p).t6
}

/// stax pieces and Silence effects the report counts when they are cast
const STAX: [&str; 22] = [
    "Trinisphere",
    "Chalice of the Void",
    "Damping Sphere",
    "Deafening Silence",
    "Cursed Totem",
    "Pithing Needle",
    "Void Mirror",
    "Vexing Bauble",
    "Abeyance",
    "Silence",
    "Orim's Chant",
    "Mandate of Peace",
    "The Tabernacle at Pendrell Vale",
    "Blind Obedience",
    "Collector Ouphe",
    "Null Rod",
    "Stony Silence",
    "Rule of Law",
    "Grafdigger's Cage",
    "Opposition Agent",
    "Drannith Magistrate",
    "Defense Grid",
];

fn bump(g: &mut Game, p: PlayerId, key: String) {
    g.player_mut(p).stat(intern(&key), 1);
}

fn card(g: &Game, c: CardId) -> String {
    g.db.get(c).name.to_string()
}

/// q's counterspell `ctr` countered p's spell or ability
pub fn counter_hit(g: &mut Game, q: PlayerId, ctr: CardId, p: PlayerId) {
    if is_t6(g, q) {
        let k = format!("ix|counter|{}|{}", card(g, ctr), g.player(p).key);
        bump(g, q, k);
    }
}

/// `actor` removed a permanent of `owner` (with this spell, or an ability)
pub fn removal_hit(g: &mut Game, actor: PlayerId, spell: Option<CardId>, owner: PlayerId) {
    if actor != owner && is_t6(g, actor) {
        let by = spell.map_or("ability".to_string(), |c| card(g, c));
        let k = format!("ix|removal|{by}|{}", g.player(owner).key);
        bump(g, actor, k);
    }
}

/// p cast a card: count the stax pieces
pub fn cast(g: &mut Game, p: PlayerId, c: CardId) {
    mana_cast(g, p, c);
    flow_cast(g, p); // --- ai-flow
    if is_t6(g, p) && STAX.contains(&&*g.db.get(c).name) {
        let k = format!("ix|stax|{}", card(g, c));
        bump(g, p, k);
    }
}

/// a board wipe cast by p resolved
pub fn wipe(g: &mut Game, p: PlayerId, c: Option<CardId>) {
    if is_t6(g, p) {
        let k = format!("ix|wipe|{}", c.map_or("ability".to_string(), |c| card(g, c)));
        bump(g, p, k);
    }
}

pub fn combo_go(g: &mut Game, p: PlayerId, combo: &str) {
    speed_mark(g, p, "ready"); // a line that is gone for was ready (also when the measure missed it)
    speed_mark(g, p, "reach");
    speed_mark(g, p, "go");
    crate::goldfish::mark_go(g, p); // --- scion: protection held as the deck goes for the win (goldfish games)
    if is_t6(g, p) {
        bump(g, p, format!("go|{combo}"));
    }
}

pub fn combo_win(g: &mut Game, p: PlayerId, combo: &str) {
    if is_t6(g, p) {
        bump(g, p, format!("win|{combo}"));
    }
}

/// the line `combo` of p was stopped by `by` (a card name, "?" when not known)
pub fn combo_stop(g: &mut Game, p: PlayerId, combo: &str, by: &str) {
    if is_t6(g, p) {
        bump(g, p, format!("stop|{combo}|{by}"));
    }
}

/// the card that last countered something (set by `resolve_counter` and the Tier 6 counters), for `combo_stop`
pub fn last_counter_name(g: &Game) -> String {
    g.last_counter.map_or("?".to_string(), |c| card(g, c))
}

// ---------------------------------------------------------------------------------------------------- speed
// Measurement for the Tier 6 speed project (stats only; nothing here decides or draws a random number).
// - `sp|ready`, `sp|reach`, `sp|go`: the round (`g.round`) a line of the deck was first ready / one tutor away / gone
//   for in this game (absent: never). Set once per game.
// - `spx|<label>`: decisions taken while a line was ready and the deck had not gone yet, by the chosen option
// - `sp|rdy_now`: 1 while a line is ready at the current decision (transient)

/// set `sp|<what>` to the current round, the first time
pub fn speed_mark(g: &mut Game, p: PlayerId, what: &str) {
    if is_t6(g, p) {
        let r = g.round as i64;
        g.player_mut(p).stats.entry(intern(&format!("sp|{what}"))).or_insert(r);
    }
}

pub fn speed_flag(g: &mut Game, p: PlayerId, ready_now: bool) {
    g.player_mut(p).stats.insert(intern("sp|rdy_now"), ready_now as i64);
}

/// `sp|away_now`: 1 while a line is one tutor away (and none is ready) at the current decision (transient)
pub fn away_flag(g: &mut Game, p: PlayerId, away_now: bool) {
    g.player_mut(p).stats.insert(intern("sp|away_now"), away_now as i64);
}

/// p chose `label` at a main-phase decision: count it under `spx|` when a line was ready and the deck has not gone
/// yet, and under `spa|` when a line was one tutor away and none was ready
pub fn speed_decided(g: &mut Game, p: PlayerId, label: &str) {
    if !g.player(p).t6 {
        return;
    }
    let pl = g.player(p);
    let now = |k: &str| pl.stats.get(&intern(k)).copied().unwrap_or(0) != 0;
    let (ready, away) = (now("sp|rdy_now"), now("sp|away_now"));
    if !(ready || away) || pl.stats.contains_key(&intern("sp|go")) {
        return;
    }
    // the card or play, without target and search tags: "Cast X -> Y [search]" -> "X"
    let l = label.split(" -> ").next().unwrap_or(label).trim_end_matches(" [search]").trim_end_matches(" [must]");
    let l = l.strip_prefix("combo: ").map_or(l.to_string(), |c| format!("combo: {c}"));
    bump(g, p, format!("{}|{l}", if ready { "spx" } else { "spa" }));
    if !ready && l.starts_with("stop") {
        // passed while one tutor away: which tutors it held (`spah|<card>`, one per tutor in hand)
        use crate::tag::Tag;
        let held: Vec<String> = g
            .player(p)
            .hand
            .iter()
            .filter(|&&c| g.db.get(c).tags.has(Tag::Tut) || g.db.get(c).tags.has(Tag::Xtutor))
            .map(|&c| card(g, c))
            .collect();
        for t in held {
            bump(g, p, format!("spah|{t}"));
        }
    }
}

// ---------------------------------------------------------------------------------------------------- mana (speed project)
// - `mn|T<n>`: mana available after the land drop on the player's turn n (1-5)
// - `mn|perm|T<n>`: mana permanents (non-land: rocks, dorks, moxen) in play then
// - `mn|hand|T<n>`: mana cards in hand then (rocks, dorks, moxen, rituals, spirit guides); `mn|handc|T<n>` of those, the
//   ones with a cost within the mana available
// - `mn|cast|T<n>`: mana cards cast on turn n (n = 1-3)
// - `mn|n|T<n>`: 1 when the player reached turn n (the divisor)

// --- ai-audit: pub (gf_audit reads it)
pub const RITUALS: [&str; 14] = [
    "Dark Ritual",
    "Cabal Ritual",
    "Rite of Flame",
    "Desperate Ritual",
    "Pyretic Ritual",
    "Seething Song",
    "Elvish Spirit Guide",
    "Simian Spirit Guide",
    "Jeweled Lotus",
    "Lion's Eye Diamond",
    "Lotus Petal",
    "Culling the Weak",
    "Sol Ring",
    "Mana Geode",
];

/// a mana source card that is not a land: rock, dork, mox, fast mana, ritual
pub fn is_mana_card(g: &Game, c: CardId) -> bool {
    use crate::tag::Tag;
    let d = g.db.get(c);
    !d.land
        && (d.tag(Tag::Rock)
            || d.tag(Tag::Dork)
            || d.tag(Tag::Fastmana)
            || d.tag(Tag::Moxd)
            || d.tag(Tag::Chromemox)
            || d.tag(Tag::Lr)
            || RITUALS.contains(&&*d.name))
}

/// stats-only snapshot at the start of p's main phase (after the land drop)
pub fn mana_snapshot(g: &mut Game, p: PlayerId) {
    if !is_t6(g, p) {
        return;
    }
    let t = g.player(p).turns;
    let mana = crate::engine::mana::total_mana(g, p, false) as i64;
    let perms = g.player(p).perms.iter().filter(|&&m| g.perm(m).cd.is_some_and(|c| is_mana_card(g, c))).count() as i64;
    let hand: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&c| is_mana_card(g, c)).collect();
    let cheap = hand.iter().filter(|&&c| (g.db.get(c).cmc as i64) <= mana).count() as i64;
    let pl = g.player_mut(p);
    pl.stat(intern(&format!("mn|T{t}")), mana);
    pl.stat(intern(&format!("mn|perm|T{t}")), perms);
    pl.stat(intern(&format!("mn|hand|T{t}")), hand.len() as i64);
    pl.stat(intern(&format!("mn|handc|T{t}")), cheap);
    pl.stat(intern(&format!("mn|n|T{t}")), 1);
}

/// p cast a mana card on its own turn 1-3
fn mana_cast(g: &mut Game, p: PlayerId, c: CardId) {
    if is_t6(g, p) && g.active == Some(p) && g.player(p).turns <= 3 && is_mana_card(g, c) {
        let t = g.player(p).turns;
        bump(g, p, format!("mn|cast|T{t}"));
    }
}

/// stats-only: at the end of p's second main phase on turns 1-3, the mana cards still in hand that p could pay for
/// and that would make mana if cast now (`mn|held|T<n>|<card>`; rituals and one-shot sources under `mn|heldr|`), so
/// the report can say which fast mana is held. For each held permanent, `mn|why|<card>|<reason>`: `rule` when the
/// AI's own utility refused it (a priority of 0, a plan rule), `lost` when it was offered and another play (or the
/// stop) was chosen.
pub fn mana_held(g: &mut Game, p: PlayerId) {
    if !is_t6(g, p) || g.player(p).turns > 3 {
        return;
    }
    let t = g.player(p).turns;
    let hand: Vec<CardId> = g.player(p).hand.clone();
    let s = crate::ai::brain::Situation::new(g, p);
    for c in hand {
        if !is_mana_card(g, c) || !makes_mana_now(g, p, c) {
            continue;
        }
        let (cg, cp) = crate::engine::mana::cost_of(g, p, c);
        g.pay_for = Some(c);
        let ok = crate::engine::mana::can_pay(g, p, cg, &cp, false);
        g.pay_for = None;
        if !ok {
            continue;
        }
        let d = g.db.get(c);
        // one-shot sources: rituals, Lotus Petal, Lion's Eye Diamond (held for the turn they pay for)
        let one_shot = d.instant || d.sorcery || matches!(d.name.as_ref(), "Lotus Petal" | "Lion's Eye Diamond");
        bump(g, p, format!("mn|held{}|T{t}|{}", if one_shot { "r" } else { "" }, card(g, c)));
        if !one_shot {
            let why = if crate::ai::brain::card_utility(g, p, &s, c).is_some() { "lost" } else { "rule" };
            bump(g, p, format!("mn|why|{}|{why}", card(g, c)));
        }
    }
}

/// would this mana card make mana if p cast it now (stats only, by Oracle text)? Mox Diamond needs a land card to
/// discard, Chrome Mox a nonartifact, nonland card of p's colours to imprint, Mox Amber a legendary creature or
/// planeswalker, Mox Opal two other artifacts; a Spirit Guide's mana is exiling it, not casting it
fn makes_mana_now(g: &Game, p: PlayerId, c: CardId) -> bool {
    use crate::cards::Types;
    let pl = g.player(p);
    let d = g.db.get(c);
    let bf = || pl.perms.iter().filter(|&&m| !g.perm(m).phased).filter_map(|&m| g.perm(m).cd).map(|x| g.db.get(x));
    match d.name.as_ref() {
        "Mox Diamond" => pl.hand.iter().any(|&x| g.db.get(x).land),
        "Chrome Mox" => pl.hand.iter().any(|&x| {
            let xd = g.db.get(x);
            x != c && !xd.land && !xd.types.has(Types::ARTIFACT) && xd.colors.intersects(pl.ident)
        }),
        "Mox Amber" => bf().any(|x| (x.creature && x.tag(crate::tag::Tag::Leg)) || x.types.has(Types::PLANESWALKER)),
        "Mox Opal" => bf().filter(|x| x.types.has(Types::ARTIFACT)).count() >= 2,
        "Elvish Spirit Guide" | "Simian Spirit Guide" => false,
        _ => true,
    }
}

/// stats only (`mn|eotw|...`): the mana a Tier 6 player has untapped when the end-of-turn window opens (the end of
/// the turn before its own), and the {4} dig activations that mana could pay for when Thrasios is in play.
/// Keys: `mn|eotw|n` windows, `mn|eotw|mana` mana summed, `mn|eotw|thr` windows with Thrasios on the battlefield,
/// `mn|eotw|can` floor(mana / 4) summed over those, `mn|eotw|cradle` windows with Gaea's Cradle in play.
pub fn eot_window(g: &mut Game, p: PlayerId) {
    if !is_t6(g, p) {
        return;
    }
    let mana = crate::engine::mana::total_mana(g, p, false) as i64;
    let thr = crate::ai::plans::on_bf(g, p, "Thrasios, Triton Hero").is_some();
    bump(g, p, "mn|eotw|n".to_string());
    g.player_mut(p).stat(intern("mn|eotw|mana"), mana);
    if thr {
        bump(g, p, "mn|eotw|thr".to_string());
        g.player_mut(p).stat(intern("mn|eotw|can"), mana / 4);
    }
}

// ---------------------------------------------------------------------------------------------------- flow
// --- ai-flow: card and mana flow of a Tier 6 player, per own turn 1-8 (stats only: nothing here decides or draws a
// random number). Keys, T<t> the own turn:
// - `fl|n|T<t>`: the turn began (the divisor); `fl|ne|T<t>`: it reached its end step (the divisor of the end-of-turn
//   snapshots); `fl|hand0`: the kept hand's size
// - `fl|d|<class>|T<t>`: cards drawn, by source (`step`, `cantrip`, `engine`, `wheel`, `cmdr`, `other`), counted on
//   the own turn the card can first be used (this turn on its own turn, else its next); Necropotence's and Ad
//   Nauseam's cards and Thrasios's land-or-hand card count as their source's draws; `fl|dc|<card>`: the same by card
//   (turns 1-8 together); `fl|dnew|T<t>`: those that were new to the player (not seen before)
// - `fl|seen|T<t>`: distinct cards seen by the end of own turn t (the kept hand, which setup marks seen, then every
//   card a draw, tutor or engine put into the hand); `fl|seenD|T<t>`: those new since the end of the turn before (so tutored cards are
//   `fl|tut|T<t>`, `seenD` less the turn's new draws); `fl|_seen`: the last snapshot (transient)
// - `fl|cast|T<t>` / `fl|casto|T<t>`: spells cast on own turn t / on the other turns that follow it
// - `fl|mana|T<t>` / `fl|hand|T<t>`: mana available and cards in hand at the start of the main phase (after the
//   land drop); `fl|out|<card>`: own turns 2-8 that began with that draw engine out; `fl|tax|<card>|trig|paid`: its
//   tax triggers and the paid ones (`flow_tax`); `fl|spent|T<t>` /
//   `fl|spento|T<t>`: mana spent on own turn t / on the other turns that follow it
// - `fl|life|<card>|T<t>`: life paid for cards (Necropotence, Ad Nauseam, Sylvan Library, Tymna)
// - `fl|land|T<t>`: a land was played; `fl|nold|T<t>`: none was; `fl|noldh|T<t>`: none was with a land in hand

/// the card-draw engines the flow report names (the brief's list)
pub const FLOW_ENGINES: [&str; 7] = [
    "Rhystic Study",
    "Mystic Remora",
    "Esper Sentinel",
    "Sylvan Library",
    "Necropotence",
    "The One Ring",
    "Ad Nauseam",
];

/// the wheels: each player (or the caster) draws a new hand
pub const FLOW_WHEELS: [&str; 8] = [
    "Timetwister",
    "Windfall",
    "Wheel of Fortune",
    "Echo of Eons",
    "Reforge the Soul",
    "Wheel of Misfortune",
    "Winds of Change",
    "Magus of the Wheel",
];

const FLOW_TURNS: u32 = 8;

fn flow_on(g: &Game, p: PlayerId) -> bool {
    !g.in_search && g.player(p).t6
}

/// the own turn on which p can first use what it gets now (this turn on its own turn, else its next)
fn flow_use_turn(g: &Game, p: PlayerId) -> u32 {
    (g.player(p).turns + u32::from(g.active != Some(p))).max(1)
}

fn flow_add(g: &mut Game, p: PlayerId, key: String, n: i64) {
    g.player_mut(p).stat(intern(&key), n);
}

/// the source whose effect is resolving for p (a spell, a trigger or an activated ability of p's): draws are
/// credited to it. Set by the stack's windows and by `cast::resolve`.
pub fn flow_source(g: &mut Game, p: PlayerId, c: Option<CardId>) {
    if g.player(p).t6 {
        g.player_mut(p).flow_src = c;
    }
}

/// the class of a draw source for p
fn flow_class(g: &Game, p: PlayerId, c: Option<CardId>) -> &'static str {
    let Some(c) = c else { return "other" };
    let d = g.db.get(c);
    let n = &*d.name;
    if FLOW_ENGINES.contains(&n) {
        "engine"
    } else if FLOW_WHEELS.contains(&n) {
        "wheel"
    } else if g.player(p).is_commander(c) || n.starts_with("Dack Fayden") {
        "cmdr"
    } else if d.instant || d.sorcery {
        "cantrip"
    } else {
        "other"
    }
}

/// p drew card c (`step`: the draw step's own draw; `new`: c was not seen before)
pub fn flow_draw(g: &mut Game, p: PlayerId, step: bool, new: bool) {
    if !flow_on(g, p) {
        return;
    }
    let src = g.player(p).flow_src;
    let cls = if step { "step" } else { flow_class(g, p, src) };
    flow_cards(g, p, cls, if step { None } else { src }, 1, new as i64);
}

/// p got n cards from the library into its hand from source `src` of class `cls` (`fresh` of them new)
fn flow_cards(g: &mut Game, p: PlayerId, cls: &str, src: Option<CardId>, n: i64, fresh: i64) {
    let t = flow_use_turn(g, p);
    if t > FLOW_TURNS || n == 0 {
        return;
    }
    flow_add(g, p, format!("fl|d|{cls}|T{t}"), n);
    flow_add(g, p, format!("fl|dnew|T{t}"), fresh);
    if let Some(c) = src {
        let k = format!("fl|dc|{}", g.db.get(c).name);
        flow_add(g, p, k, n);
    }
}

/// an engine that is not a draw put n new cards into p's hand (Necropotence, Ad Nauseam, Thrasios)
pub fn flow_engine_cards(g: &mut Game, p: PlayerId, name: &str, n: i64) {
    if !flow_on(g, p) {
        return;
    }
    let src = g.db.id(name);
    let cls = match src {
        Some(c) => flow_class(g, p, Some(c)),
        None => "other",
    };
    flow_cards(g, p, cls, src, n, n);
}

/// a card p drew from `name` went back (Sylvan Library's cards not paid for): it is not a draw
pub fn flow_undraw(g: &mut Game, p: PlayerId, name: &str, n: i64) {
    if !flow_on(g, p) || n == 0 {
        return;
    }
    let t = flow_use_turn(g, p);
    if t > FLOW_TURNS {
        return;
    }
    flow_add(g, p, format!("fl|d|engine|T{t}"), -n);
    flow_add(g, p, format!("fl|dnew|T{t}"), -n);
    flow_add(g, p, format!("fl|dc|{name}"), -n);
}

/// p paid n life for card `name`
pub fn flow_life(g: &mut Game, p: PlayerId, name: &str, n: i64) {
    if !flow_on(g, p) || n <= 0 {
        return;
    }
    let t = g.player(p).turns.max(1);
    if t <= FLOW_TURNS {
        flow_add(g, p, format!("fl|life|{name}|T{t}"), n);
    }
}

/// own turn t, the cycle it starts (that turn, and the other players' turns until p's next)
fn flow_cycle(g: &Game, p: PlayerId) -> Option<(u32, bool)> {
    let t = g.player(p).turns;
    (1..=FLOW_TURNS).contains(&t).then_some((t, g.active == Some(p)))
}

fn flow_cast(g: &mut Game, p: PlayerId) {
    if !flow_on(g, p) {
        return;
    }
    if let Some((t, own)) = flow_cycle(g, p) {
        flow_add(g, p, format!("fl|{}|T{t}", if own { "cast" } else { "casto" }), 1);
    }
}

/// p's spell c was countered: `fl|ctrd|T<t>` (own turn t's cycle), and `fl|ctrdc|<card>` for a draw engine
pub fn flow_countered(g: &mut Game, p: PlayerId, c: CardId) {
    if !flow_on(g, p) {
        return;
    }
    if let Some((t, _)) = flow_cycle(g, p) {
        flow_add(g, p, format!("fl|ctrd|T{t}"), 1);
        let n = g.db.get(c).name.to_string();
        if FLOW_ENGINES.contains(&n.as_str()) || FLOW_OUT.contains(&n.as_str()) {
            flow_add(g, p, format!("fl|ctrdc|{n}"), 1);
        }
    }
}

/// p spent n mana (a mana unit used to pay)
pub fn flow_spent(g: &mut Game, p: PlayerId, n: u32) {
    if n == 0 || !flow_on(g, p) {
        return;
    }
    if let Some((t, own)) = flow_cycle(g, p) {
        flow_add(g, p, format!("fl|{}|T{t}", if own { "spent" } else { "spento" }), n as i64);
    }
}

/// the start of p's own turn (after `turns` counts it)
pub fn flow_turn_start(g: &mut Game, p: PlayerId) {
    if !flow_on(g, p) {
        return;
    }
    let t = g.player(p).turns;
    if t == 1 {
        // the kept hand is already seen (setup_game): the baseline of the per-turn new cards
        let (h, s) = (g.player(p).hand.len() as i64, g.player(p).seen.len() as i64);
        g.player_mut(p).stats.insert(intern("fl|hand0"), h);
        g.player_mut(p).stats.insert(intern("fl|_seen"), s);
    }
    if t <= FLOW_TURNS {
        flow_add(g, p, format!("fl|n|T{t}"), 1);
    }
    // the draw engines that were out through the turns before this one (turns 2-8: the draws they made then count
    // toward this turn): `fl|out|<card>`, so the report can give draws per turn cycle out
    if (2..=FLOW_TURNS).contains(&t) {
        let out: Vec<String> = g
            .player(p)
            .perms
            .iter()
            .filter(|&&m| g.perm(m).on_bf && !g.perm(m).phased)
            .filter_map(|&m| g.perm(m).cd)
            .map(|c| g.db.get(c).name.to_string())
            .filter(|n| FLOW_ENGINES.contains(&n.as_str()) || FLOW_OUT.contains(&n.as_str()))
            .collect();
        for n in out {
            flow_add(g, p, format!("fl|out|{n}"), 1);
        }
    }
}

/// the other draw permanents whose turns out the report counts
const FLOW_OUT: [&str; 5] =
    ["Kraum, Ludevic's Opus", "Tymna the Weaver", "Thrasios, Triton Hero", "Faerie Mastermind", "Consecrated Sphinx"];

/// a tax trigger of p's engine `name` (Rhystic Study, Mystic Remora, Esper Sentinel) resolved against an opponent's
/// spell: `fl|tax|<card>|trig` and, when the caster paid, `fl|tax|<card>|paid` (turns 1-8 of p)
pub fn flow_tax(g: &mut Game, p: PlayerId, name: &str, paid: bool) {
    if !flow_on(g, p) || flow_use_turn(g, p) > FLOW_TURNS {
        return;
    }
    flow_add(g, p, format!("fl|tax|{name}|trig"), 1);
    if paid {
        flow_add(g, p, format!("fl|tax|{name}|paid"), 1);
    }
}

/// the start of p's main phase, after the land drop: the mana available
pub fn flow_main(g: &mut Game, p: PlayerId) {
    if !flow_on(g, p) {
        return;
    }
    let t = g.player(p).turns;
    if t <= FLOW_TURNS {
        let mana = crate::engine::mana::total_mana(g, p, false) as i64;
        let hand = g.player(p).hand.len() as i64;
        flow_add(g, p, format!("fl|mana|T{t}"), mana);
        flow_add(g, p, format!("fl|hand|T{t}"), hand);
        g.player_mut(p).stats.insert(intern("fl|_mana"), mana); // the main phase's mana (the turn-4 fate)
    }
}

/// the end of p's own turn: the cards seen so far and the land drop
pub fn flow_turn_end(g: &mut Game, p: PlayerId) {
    if !flow_on(g, p) {
        return;
    }
    let t = g.player(p).turns;
    if !(1..=FLOW_TURNS).contains(&t) {
        return;
    }
    let pl = g.player(p);
    let seen = pl.seen.len() as i64;
    let last = pl.stats.get(&intern("fl|_seen")).copied().unwrap_or(0);
    let played = pl.land_turn == t as i32;
    let land_in_hand = pl.hand.iter().any(|&c| g.db.get(c).land);
    g.player_mut(p).stats.insert(intern("fl|_seen"), seen);
    flow_add(g, p, format!("fl|ne|T{t}"), 1);
    flow_add(g, p, format!("fl|seen|T{t}"), seen);
    flow_add(g, p, format!("fl|seenD|T{t}"), seen - last);
    // the turn's new draws are all counted by now (later ones count toward the next turn): the rest came by tutors
    let dnew = g.player(p).stats.get(&intern(&format!("fl|dnew|T{t}"))).copied().unwrap_or(0);
    flow_add(g, p, format!("fl|tut|T{t}"), seen - last - dnew);
    // the engine funnel by the end of this turn: seen, cast, out (`fl|ef|<card>|seen|cast|out|T<t>`); the
    // commanders count as seen from the start, and a wheel is never out
    let funnel: Vec<(String, bool, bool, bool)> = FLOW_ENGINES
        .iter()
        .chain(FLOW_OUT.iter())
        .chain(FLOW_WHEELS.iter())
        .filter_map(|&n| {
            let c = g.db.id(n)?;
            let pl = g.player(p);
            if pl.is_commander(c) {
                let out = pl.perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).cd == Some(c));
                return Some((n.to_string(), true, pl.cast_names.contains(&c), out));
            }
            if !pl.deck_names.contains(&c) {
                return None;
            }
            let out = pl.perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).cd == Some(c));
            Some((n.to_string(), pl.seen.contains(&c), pl.cast_names.contains(&c), out))
        })
        .collect();
    // where a seen engine that was never cast is at the end of turn 4 (`fl|ez|<card>|<zone>`): still in hand (and
    // whether p had the mana for it then), or gone (graveyard: discarded; exile: pitched or imprinted; library)
    if t == 4 {
        let fates: Vec<String> = funnel
            .iter()
            .filter(|f| f.1 && !f.2)
            .filter_map(|f| {
                let c = g.db.id(&f.0)?;
                let pl = g.player(p);
                let z = if pl.hand.contains(&c) {
                    let mana = pl.stats.get(&intern("fl|_mana")).copied().unwrap_or(0);
                    if (g.db.get(c).cmc as i64) <= mana { "hand_payable" } else { "hand" }
                } else if pl.gy.contains(&c) {
                    "gy"
                } else if pl.exile.contains(&c) {
                    "exile"
                } else if pl.library.contains(&c) {
                    "lib"
                } else {
                    "other"
                };
                Some(format!("fl|ez|{}|{z}", f.0))
            })
            .collect();
        for k in fates {
            flow_add(g, p, k, 1);
        }
    }
    for (n, s, c, o) in funnel {
        for (what, yes) in [("seen", s), ("cast", c), ("out", o)] {
            if yes {
                flow_add(g, p, format!("fl|ef|{n}|{what}|T{t}"), 1);
            }
        }
    }
    if played {
        flow_add(g, p, format!("fl|land|T{t}"), 1);
    } else {
        flow_add(g, p, format!("fl|nold|T{t}"), 1);
        if land_in_hand {
            flow_add(g, p, format!("fl|noldh|T{t}"), 1);
        }
    }
}
