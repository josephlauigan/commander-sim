//! Goldfish games (`commander-sim --goldfish`): the deck under test plays alone against three inert seats. An inert
//! seat has 40 life and a library of 99 Wastes (it draws one card a turn, so the library outlasts 20 rounds); its
//! commander is a vanilla creature it never casts. Its turns pass in the normal order (untap, upkeep, draw, combat
//! and end steps happen, so round counting and "each upkeep / each draw" triggers of the deck are real), but it never
//! plays a land, casts, activates, attacks, blocks or responds: it keeps its seven cards (no mulligan), skips its main
//! phases, combat and the end-of-turn window, and has no mana and no spells for any priority window.
//!
//! Inert seats are recognised by their deck key (`goldfish-2`, `goldfish-3`, `goldfish-4`), as the rest of the AI
//! keys on the seat key. Nothing here runs in a game without an inert seat.

use crate::cards::{CardDb, Colors};
use crate::engine::turn::Seat;
use crate::ids::{CardId, PlayerId};
use crate::state::Game;
use crate::sym::intern;
use crate::tag::Tag;

/// the deck-key prefix of an inert seat
pub const PREFIX: &str = "goldfish";

/// the vanilla creatures an inert seat may have as its (never cast) commander: the first one in data/cards.json
pub const COMMANDERS: [&str; 3] = ["Goblin Roughrider", "Armored Warhorse", "Amrou Seekers"];

/// the inert seat's library: basic lands that make no colour
pub const LAND: &str = "Wastes";

/// cards in an inert seat's deck, its commander included
pub const SIZE: usize = 100;

/// cast locks that count as protection on the kill turn: Silence and Orim's Chant cast that turn, Grand Abolisher
/// on the battlefield
pub const LOCKS: [&str; 3] = ["Silence", "Orim's Chant", "Grand Abolisher"];

pub fn is_inert_key(key: &str) -> bool {
    key.starts_with(PREFIX)
}

/// is p an inert goldfish seat?
pub fn is_inert(g: &Game, p: PlayerId) -> bool {
    is_inert_key(g.player(p).key)
}

/// is this a goldfish game (any inert seat)?
pub fn is_goldfish(g: &Game) -> bool {
    g.players.iter().any(|q| is_inert_key(q.key))
}

/// inert seat number `n` (2, 3 or 4): key `goldfish-n`, name `Goldfish n`
pub fn inert_seat(db: &CardDb, n: usize) -> Result<Seat, String> {
    let commander = COMMANDERS
        .iter()
        .find_map(|c| db.id(c))
        .ok_or_else(|| format!("none of {COMMANDERS:?} is in data/cards.json: the goldfish seats need one"))?;
    let land = db.id(LAND).ok_or_else(|| format!("{LAND} is not in data/cards.json"))?;
    let cards: std::sync::Arc<[CardId]> =
        std::iter::once(commander).chain(std::iter::repeat_n(land, SIZE - 1)).collect();
    Ok(Seat {
        key: intern(&format!("{PREFIX}-{n}")),
        name: intern(&format!("Goldfish {n}")),
        commander,
        commander2: None,
        ident: Colors::NONE,
        cards,
    })
}

/// the three inert seats that follow the deck under test (seat 1)
pub fn inert_seats(db: &CardDb) -> Result<Vec<Seat>, String> {
    (2..=4).map(|n| inert_seat(db, n)).collect()
}

/// a card that counts as protection when held on the kill turn: a counterspell or protection spell by its tags
/// (`ctr`, `prot`), or one of the cast locks
pub fn protection_card(db: &CardDb, c: CardId) -> bool {
    let d = db.get(c);
    d.tag(Tag::Ctr) || d.tags.has(Tag::Prot) || LOCKS.contains(&&*d.name)
}

/// p holds protection now: a protection card in hand, Silence or Orim's Chant cast this turn, or Grand Abolisher on
/// the battlefield
pub fn protected(g: &Game, p: PlayerId) -> bool {
    let pl = g.player(p);
    if pl.hand.iter().any(|&c| protection_card(&g.db, c)) {
        return true;
    }
    let st = g.turn_stamp();
    let cast_now = match &pl.turn_casts {
        Some((s, cs)) if *s == st => cs.iter().any(|&c| matches!(&*g.db.get(c).name, "Silence" | "Orim's Chant")),
        _ => false,
    };
    cast_now
        || pl.perms.iter().any(|&m| {
            let x = g.perm(m);
            x.on_bf && !x.phased && &*x.name == "Grand Abolisher"
        })
}

/// stats only, goldfish games only: p goes for a win now (a combo attempt). `gf|prot` is 1 if p is protected at
/// this moment (the last attempt counts), `gf|go_turn` its own turn
pub fn mark_go(g: &mut Game, p: PlayerId) {
    if !is_goldfish(g) || is_inert(g, p) {
        return;
    }
    crate::gf_audit::went(g, p); // --- ai-audit
    let prot = protected(g, p) as i64;
    let t = g.player(p).turns as i64;
    let pl = g.player_mut(p);
    pl.stats.insert(intern("gf|prot"), prot);
    pl.stats.insert(intern("gf|go_turn"), t);
}
