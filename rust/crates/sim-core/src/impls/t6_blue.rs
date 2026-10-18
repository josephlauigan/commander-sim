//! Tier 6 batch `blue` (rust/agents/t6-batches.md): the blue and small utility cards of the Kraum & Tymna, Rograkh &
//! Silas and Kinnan decks. Cards only: no engine file is touched; what needs the engine is listed in
//! rust/agents/engine-needs-blue.md.
//!
//! - Borne Upon a Wind: draw, and the one-turn "cast spells as though they had flash" flag (`t6_lands::FLASH_FLAG`).

use crate::cards::CardDb;
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

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Borne Upon a Wind")?.resolve = Some(borne_resolve);
    Ok(())
}
