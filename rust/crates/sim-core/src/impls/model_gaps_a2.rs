//! Model gaps, batch A2 (rust/agents/model-gaps-a2.md): cards of the thanos, kefka, urdragon and sauron decks that the
//! audit rated below Full. The audit verdicts are in data/card_overrides.toml.
//!
//! - **Lotho, Corrupt Shirriff**: "Whenever a player casts their second spell each turn, you lose 1 life and create a
//!   Treasure token." Its regex `treas` tag made a Treasure when Lotho entered (not on the card); the override drops the
//!   tag and this cast hook does what the Oracle text says, for every player's second spell.

use crate::cards::CardDb;
use crate::engine::cast::casts_this_turn;
use crate::engine::life::lose_life;
use crate::engine::stack::trigger_window;
use crate::engine::zones::add_treasure;
use crate::flow::Res;
use crate::hooks::{Registry, Src};
use crate::ids::{CardId, PlayerId};
use crate::state::Game;

pub const LOTHO: &str = "Lotho, Corrupt Shirriff";

/// Lotho: a player casts their second spell this turn (any player's, yours too): its controller loses 1 life and
/// creates a Treasure
fn lotho_cast(g: &mut Game, src: Src, caster: PlayerId, _c: CardId) -> Res {
    let o = g.perm(src).owner;
    if !g.perm(src).on_bf || g.perm(src).phased || !g.player(o).alive || casts_this_turn(g, caster) != 2 {
        return Ok(());
    }
    if trigger_window(g, o, Some(src), "lose 1 life, create a Treasure", None)? {
        lose_life(g, o, 1, Some(o), "life", None)?;
        add_treasure(g, o, 1)?;
    }
    Ok(())
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    if db.id(LOTHO).is_some() {
        r.card(db, LOTHO)?.cast = Some(lotho_cast);
    }
    Ok(())
}
