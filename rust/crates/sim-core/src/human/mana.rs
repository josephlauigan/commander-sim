//! Python's play/mana.py. Ported by the practice-core task (not yet).

use crate::flow::Res;
use crate::ids::PlayerId;
use crate::state::Game;

/// `choices::pay_tax`'s body. PORT(core): milestone A stub.
pub fn pay_tax(_g: &mut Game, _p: PlayerId, _n: u32, _what: &str) -> Res<bool> {
    unimplemented!("PORT(core): mana::pay_tax")
}
