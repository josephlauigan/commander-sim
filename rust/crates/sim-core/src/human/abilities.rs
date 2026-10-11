//! Python's play/abilities.py. Ported by the practice-cards task. The stand-ins below let `human.rs` compile.

use crate::flow::Res;
use crate::ids::{LandId, PermId, PlayerId};
use crate::state::Game;

/// an activated ability of a permanent: None, or the reason it can't be done
pub type PermFn = fn(&mut Game, PlayerId, PermId) -> Res<Option<String>>;
/// an activated ability of a land (beyond tapping for mana)
pub type LandFn = fn(&mut Game, PlayerId, LandId) -> Res<Option<String>>;

/// `abilities.permanent_abilities(g, p, m)`: [(label, fn)]. PORT(practice-cards): stub.
pub fn permanent_abilities(_g: &Game, _p: PlayerId, _m: PermId) -> Vec<(String, PermFn)> {
    vec![]
}

/// `abilities.land_abilities(g, p, L)`: [(label, fn)]. PORT(practice-cards): stub.
pub fn land_abilities(_g: &Game, _p: PlayerId, _l: LandId) -> Vec<(String, LandFn)> {
    vec![]
}
