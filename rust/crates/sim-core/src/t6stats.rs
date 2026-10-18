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
