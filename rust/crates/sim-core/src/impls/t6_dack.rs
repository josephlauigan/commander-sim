//! Tier 6 batch `dack`: Dack Fayden's white stax pile (rust/agents/t6-batches.md, audit/t6/dack-fayden-white-stax).
//! Cards only: no engine file is touched. What needs the engine is listed in rust/agents/engine-needs-dack.md.
//!
//!
//! The banishers' enter hooks read "you" from the enter call's `p` (the player the creature entered under), not from
//! the permanent's current controller: Dack puts creatures onto the battlefield under its controller and then
//! donates them, so by the time the queued enter trigger runs the permanent belongs to an opponent but the trigger is
//! still its original controller's (CR 603.3a).

use crate::cards::CardDb;
use crate::engine::stack::trigger_window;
use crate::flow::Res;
use crate::hooks::{Registry, Src};
use crate::ids::{PermId, PlayerId};
use crate::impls::common::{oring_exile, oring_return};
use crate::state::Game;


// ================================================================== Fiend Hunter
/// Full: "you may exile another target creature"; the exiled card returns under its owner's control when Fiend
/// Hunter leaves (a leaves trigger). The AI takes the best opposing creature worth taking (pval >= 1).
fn hunter_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, p, Some(src), "exile another target creature", Some(5.0))? && g.perm(src).on_bf {
        oring_exile(g, src, p, |g, x| x != src && g.is_creature(x), false, true)?;
    }
    Ok(())
}

fn hunter_leaves(g: &mut Game, src: Src, _m: PermId) -> Res {
    oring_return(g, src)
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Fiend Hunter")?;
    c.etb = Some(hunter_etb);
    c.leaves = Some(hunter_leaves);
    Ok(())
}
