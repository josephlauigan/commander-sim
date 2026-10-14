//! The Tymna partner decks (`thrasios`: Tymna the Weaver + Thrasios, Triton Hero; `kraum`: Tymna + Kraum, Ludevic's
//! Opus). The seat has two commanders (`Player::cmd` and `Player::cmd2`); the engine casts both from the command
//! zone with the generic commander priority. The decks' own AI plans are written later: for now both use the
//! default priority.

use crate::ids::{CardId, PlayerId};
use crate::state::Game;

/// ai/decks.rs `deck_prio` for the thrasios seat (the default until the plan is written)
pub fn thrasios_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    crate::ai::pool::generic_prio(g, p, c)
}

/// ai/decks.rs `deck_prio` for the kraum seat (the default until the plan is written)
pub fn kraum_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    crate::ai::pool::generic_prio(g, p, c)
}

// ------------------------------------------------------------------ measurement
/// `ty_tt_<name>` / `ty_tn_<name>`: the first turn (of the owner's own turn count) a partner-deck commander or Tymna came
/// down, summed over games, and the number of games it did (the analysis report prints the mean).
fn landed(g: &mut Game, src: crate::hooks::Src, _p: PlayerId, m: crate::ids::PermId) -> crate::flow::Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    let name = crate::engine::values::card_name(g, src).unwrap_or("?").split(',').next().unwrap_or("?").replace(' ', "_");
    let turns = g.player(o).turns as i64;
    let pl = g.player_mut(o);
    let key = crate::sym::intern(&format!("ty_tn_{name}"));
    if pl.stats.contains_key(&key) {
        pl.stat(crate::sym::intern(&format!("ty_re_{name}")), 1); // cast again after it died
        return Ok(());
    }
    pl.stat(crate::sym::intern(&format!("ty_tt_{name}")), turns);
    pl.stat(key, 1);
    Ok(())
}

pub fn register(r: &mut crate::hooks::Registry, db: &crate::cards::CardDb) -> Result<(), String> {
    for name in ["Tymna the Weaver", "Thrasios, Triton Hero", "Kraum, Ludevic's Opus"] {
        if db.id(name).is_some() {
            r.card(db, name)?.etb = Some(landed);
        }
    }
    Ok(())
}
