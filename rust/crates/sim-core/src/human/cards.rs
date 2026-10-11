//! Python's play/cards.py. Ported by the practice-cards task.

use crate::ids::{CardId, PlayerId};
use crate::state::Game;

/// What card c needs before the person can cast it that `legal` doesn't know (Python's `cards.needs`): the reason
/// it can't be cast, or None. PORT(practice-cards): stub, never blocks.
pub fn needs(_g: &Game, _p: PlayerId, _c: CardId) -> Option<String> {
    None
}

/// Does p's Flashback (the card) give graveyard card c flashback (Python's `cards.granted_flashback`).
/// PORT(practice-cards): stub.
pub fn granted_flashback(_g: &Game, _p: PlayerId, _c: CardId) -> bool {
    false
}

// ---- stand-ins for Python's per-card tables (play/cards.py): PORT(practice-cards) replaces each body ----
use crate::flow::Res;
use crate::ids::PermId;

pub type PermFn = super::abilities::PermFn;

/// `cards.abilities(g, p, m)`: the abilities card code offers for permanent m (None: the card has no card code of
/// its own; Some: by the rules, replacing the AI's hook options). PORT(practice-cards): stub.
pub fn abilities(_g: &Game, _p: PlayerId, _m: PermId) -> Option<Vec<(String, PermFn)>> {
    None
}

/// `cards.copy_card(c)`: Return the Favor, Dualcaster Mage (cast to copy a spell on the stack). PORT(practice-cards).
pub fn copy_card(_g: &Game, _c: CardId) -> bool {
    false
}

/// `cards.copy_in_response(g, p, c, spell, caster)`: None or why not. PORT(practice-cards): stub.
pub fn copy_in_response(
    _g: &mut Game,
    _p: PlayerId,
    _c: CardId,
    _spell: Option<CardId>,
    _caster: PlayerId,
) -> Res<Option<String>> {
    Ok(None)
}

/// `c.name in cards.HAND`: a hand card whose casting card code handles. PORT(practice-cards): stub.
pub fn hand_override(_g: &Game, _c: CardId) -> bool {
    false
}

/// `cards.cast_from_hand(g, p, c)`. PORT(practice-cards): stub.
pub fn cast_from_hand(_g: &mut Game, _p: PlayerId, _c: CardId) -> Res<Option<String>> {
    Ok(None)
}

/// `c.name in cards.GY`: a graveyard card whose casting card code handles. PORT(practice-cards): stub.
pub fn gy_override(_g: &Game, _c: CardId) -> bool {
    false
}

/// `cards.GY[c.name](g, p, c)`. PORT(practice-cards): stub.
pub fn cast_from_gy(_g: &mut Game, _p: PlayerId, _c: CardId) -> Res<Option<String>> {
    Ok(None)
}

/// The result of `cards.CAST_TARGET` (a tagless spell with a creature target, picked with `pick_creature`).
pub enum CastTarget {
    /// the card has no such target
    NotNeeded,
    /// the person cancelled
    Cancelled,
    Picked(PermId),
}

/// PORT(practice-cards): stub.
pub fn cast_target(_g: &Game, _p: PlayerId, _c: CardId) -> Res<CastTarget> {
    Ok(CastTarget::NotNeeded)
}

/// `cards.mastery_target(g, p, c)`: Mizzix's Mastery's one target; false = cancelled. PORT(practice-cards): stub
/// (a person cannot cast Mastery for one target until this is ported).
pub fn mastery_target(_g: &mut Game, _p: PlayerId, _c: CardId) -> Res<bool> {
    Ok(false)
}

/// `cards.pick_blue`: an alternative cost that exiles a blue card from your hand (Force of Will, Force of
/// Negation): which one.
pub fn pick_blue(g: &Game, p: PlayerId, blues: &[CardId], what: &str) -> Res<CardId> {
    if blues.len() == 1 {
        return Ok(blues[0]);
    }
    let labels: Vec<String> = blues.iter().map(|&c| g.db.get(c).name.to_string()).collect();
    let k =
        super::choices::choose_one(g, p, "choose", &format!("{what}: exile which blue card from your hand?"), &labels)?;
    Ok(blues[k])
}
