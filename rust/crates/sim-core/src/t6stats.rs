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

/// the deck keys of the Tier 6 pool and JD's Godo deck
pub const T6_KEYS: [&str; 7] = [
    "dack-fayden-white-stax",
    "kinnan-bonder-mana-combo",
    "kraum-tymna-blue-breach-midrange",
    "rograkh-silas-turbo-breach",
    "rograkh-thrasios-creature-combo",
    "sisay-legends-toolbox",
    "godo",
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

/// p chose `label` at a main-phase decision: count it when a line was ready and the deck has not gone yet
pub fn speed_decided(g: &mut Game, p: PlayerId, label: &str) {
    if !g.player(p).t6 {
        return;
    }
    let pl = g.player(p);
    if pl.stats.get(&intern("sp|rdy_now")).copied().unwrap_or(0) == 0 || pl.stats.contains_key(&intern("sp|go")) {
        return;
    }
    // the card or play, without target and search tags: "Cast X -> Y [search]" -> "X"
    let l = label.split(" -> ").next().unwrap_or(label).trim_end_matches(" [search]").trim_end_matches(" [must]");
    let l = l.strip_prefix("combo: ").map_or(l.to_string(), |c| format!("combo: {c}"));
    bump(g, p, format!("spx|{l}"));
}

// ---------------------------------------------------------------------------------------------------- mana (speed project)
// - `mn|T<n>`: mana available after the land drop on the player's turn n (1-5)
// - `mn|perm|T<n>`: mana permanents (non-land: rocks, dorks, moxen) in play then
// - `mn|hand|T<n>`: mana cards in hand then (rocks, dorks, moxen, rituals, spirit guides); `mn|handc|T<n>` of those, the
//   ones with a cost within the mana available
// - `mn|cast|T<n>`: mana cards cast on turn n (n = 1-3)
// - `mn|n|T<n>`: 1 when the player reached turn n (the divisor)

const RITUALS: [&str; 14] = [
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
/// (`mn|held|T<n>|<card>`), so the report can say which fast mana is held
pub fn mana_held(g: &mut Game, p: PlayerId) {
    if !is_t6(g, p) || g.player(p).turns > 3 {
        return;
    }
    let t = g.player(p).turns;
    let hand: Vec<CardId> = g.player(p).hand.clone();
    for c in hand {
        if !is_mana_card(g, c) {
            continue;
        }
        let (cg, cp) = crate::engine::mana::cost_of(g, p, c);
        g.pay_for = Some(c);
        let ok = crate::engine::mana::can_pay(g, p, cg, &cp, false);
        g.pay_for = None;
        let d = g.db.get(c);
        let ritual = d.instant || d.sorcery;
        let k = format!("mn|held{}|T{t}|{}", if ritual { "r" } else { "" }, card(g, c));
        if ok {
            bump(g, p, k);
        }
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
