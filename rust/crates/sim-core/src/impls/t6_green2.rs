//! Tier 6 batch `green2`: the green and small utility cards of the Rograkh & Thrasios and Kinnan decks
//! (rust/agents/t6-batches.md). Cards only: no engine file is touched; what needs the engine is listed in
//! rust/agents/engine-needs-green2.md.
//!
//! - Chatterstorm: the Squirrel token is the compiled spell; storm is `t0_stella::storm_copies` on the cast.

use crate::cards::CardDb;
use crate::flow::Res;
use crate::hooks::Registry;
use crate::ids::{CardId, PlayerId};

// ================================================================== Chatterstorm
/// Storm: when you cast it, copy it for each spell cast before it this turn
fn chatterstorm_cast(g: &mut crate::state::Game, p: PlayerId, c: CardId) -> Res {
    crate::impls::t0_stella::storm_copies(g, p, c)
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Chatterstorm")?.self_cast = Some(chatterstorm_cast);
    Ok(())
}
