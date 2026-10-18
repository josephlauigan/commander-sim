//! Tier 6 batch `blue` (rust/agents/t6-batches.md): the blue and small utility cards of the Kraum & Tymna, Rograkh &
//! Silas and Kinnan decks. Cards only: no engine file is touched; what needs the engine is listed in
//! rust/agents/engine-needs-blue.md.
//!
//! - Borne Upon a Wind: draw, and the one-turn "cast spells as though they had flash" flag (`t6_lands::FLASH_FLAG`).
//! - Mnemonic Betrayal: opponents' graveyards are exiled and their nonland cards held in hand (any mana type) for the turn.
//! - Timetwister: every player's hand and graveyard shuffled into their library, then seven cards drawn.

use crate::cards::CardDb;
use crate::engine::tutors::shuffle_library;
use crate::engine::zones::draw;
use crate::flow::Res;
use crate::hooks::Registry;
use crate::ids::{CardId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::Sym;

fn set_turn_flag(g: &mut Game, p: PlayerId, key: Sym) {
    let stamp = g.turn_stamp();
    g.player_mut(p).flag_turn.insert(key, stamp);
}

/// does p hold the one-turn grant "you may cast spells this turn as though they had flash"? (Borne Upon a Wind;
/// shared with Emergence Zone). The engine's cast-timing check and the AI's end-of-turn window should read this.
pub fn flash_turn_grant(g: &Game, p: PlayerId) -> bool {
    g.player(p).flag_turn.get(crate::impls::t6_lands::FLASH_FLAG) == Some(&g.turn_stamp())
}

// ================================================================== Borne Upon a Wind
/// Borne Upon a Wind: you may cast spells this turn as though they had flash; draw a card
fn borne_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    set_turn_flag(g, p, crate::impls::t6_lands::FLASH_FLAG);
    draw(g, p, 1, false)?;
    Ok("gy")
}

// ================================================================== Mnemonic Betrayal
/// Mnemonic Betrayal: exile every opponent's graveyard; p may cast spells from among those cards this turn with mana
/// of any type (the cards are held in p's hand like Opposition Agent's, `yshtola::take_card`, and listed in
/// `impulse` so the end step clears what was not cast); Betrayal itself is exiled.
/// Missing: what remains goes back to its owner's graveyard (the end-step cleanup exiles it with p's own impulse cards).
fn betrayal_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    for q in g.opps(p).collect::<Vec<_>>() {
        let gy = std::mem::take(&mut g.player_mut(q).gy);
        let n = gy.len();
        for c in gy {
            if g.db.get(c).land {
                g.player_mut(q).exile.push(c); // a land can't be cast
            } else {
                crate::impls::yshtola::take_card(g, p, c, q, "Mnemonic Betrayal");
                g.player_mut(p).impulse.push(c);
            }
        }
        crate::glog!(g, "    Mnemonic Betrayal exiles {} card(s) from {}'s graveyard", n, g.player(q).name);
    }
    Ok("exile")
}

// ================================================================== Timetwister
/// Timetwister: each player shuffles their hand and graveyard into their library, then draws seven cards; Timetwister
/// itself goes to the graveyard afterwards (it is not shuffled in). A card held from an opponent (Opposition Agent
/// style) goes into its owner's library.
fn timetwister_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mut order = vec![p];
    order.extend(g.after(p).filter(|&q| g.player(q).alive));
    for &q in &order {
        let hand = std::mem::take(&mut g.player_mut(q).hand);
        let gy = std::mem::take(&mut g.player_mut(q).gy);
        for c in hand.into_iter().chain(gy) {
            let owner = g.player_mut(q).stolen.swap_remove(&c).unwrap_or(q);
            g.player_mut(q).agent_ids.retain(|&x| x != c);
            g.player_mut(q).impulse.retain(|&x| x != c);
            g.player_mut(owner).library.push(c);
        }
    }
    for &q in &order {
        shuffle_library(g, q);
    }
    for &q in &order {
        draw(g, q, 7, false)?;
        if g.over {
            break;
        }
    }
    Ok("gy")
}


// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Timetwister")?.resolve = Some(timetwister_resolve);
    r.card(db, "Borne Upon a Wind")?.resolve = Some(borne_resolve);
    r.card(db, "Mnemonic Betrayal")?.resolve = Some(betrayal_resolve);
    Ok(())
}
