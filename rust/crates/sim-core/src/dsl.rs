//! The card ability language's interpreter (Python's `cards/dsl.py`): compiled abilities on cards, run by the engine.
//! Ported in M3; until then each function is a `PORT(M3)` placeholder with the value a card with no compiled
//! abilities would give, so tag-modeled cards play as they do in Python.

use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;

/// Power and toughness changes beyond printed stats and counters: until-end-of-turn pumps, then (PORT(M3)) Auras
/// and the ability language's anthems, equipment bonuses and self-scaling creatures. Python's `dsl.pt`.
pub fn pt(g: &Game, m: PermId) -> (i32, i32) {
    let x = g.perm(m);
    let (dp, dt) = x.eot_pt;
    // PORT(M3): CI.attached_bonus (Auras, Elspeth's emblem) and the anthem / equip_bonus / bma_anthem /
    // self_scaling statics
    (dp, dt)
}

/// Does m have keyword kw (lower case, as Scryfall writes them)? Until-end-of-turn grants, a token's own keywords,
/// printed keywords (unless its abilities are off), then (PORT(M3)) Auras, card grants and the ability language's
/// statics. Python's `dsl.has_kw`.
pub fn has_kw(g: &Game, m: PermId, kw: &str) -> bool {
    let x = g.perm(m);
    if x.eot_kw.contains(&kw) {
        return true;
    }
    match x.cd {
        None => {
            if let Some(crate::state::Val::List(v)) = x.data.get(crate::state::DataKey::Kws)
                && v.iter().any(|k| matches!(k, crate::state::Val::Str(s) if *s == kw))
            {
                return true;
            }
        }
        Some(c) => {
            if !x.neutered && g.db.get(c).has_kw(kw) {
                return true;
            }
        }
    }
    // PORT(M3): Elspeth's emblem (flying), CI.attached_kw, CI.granted_kw, the ability language's keyword statics
    false
}

/// PORT(M3): colours m has protection from through the ability language
pub fn protection(_g: &Game, _m: PermId) -> crate::cards::Colors {
    crate::cards::Colors::NONE
}

/// PORT(M3): cost changes from static abilities (generic mana, + or -)
pub fn cost_delta(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

/// PORT(M3): "players can't gain life" statics
pub fn no_lifegain(_g: &Game, _p: PlayerId) -> bool {
    false
}

/// PORT(M3): token doublers
pub fn token_mult(_g: &Game, _p: PlayerId) -> u32 {
    1
}

/// PORT(M3): the interpreter's estimate of a card's value
pub fn card_value(_g: &Game, _p: PlayerId, _c: CardId) -> f64 {
    0.0
}

/// PORT(M3): an instant or sorcery with compiled abilities resolves
pub fn resolve_spell(_g: &mut Game, _p: PlayerId, _c: CardId, _ctx: &crate::state::Ctx) -> crate::flow::Res {
    Ok(())
}
