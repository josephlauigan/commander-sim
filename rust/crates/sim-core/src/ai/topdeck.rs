//! Ordering the top of a library (Python's `cards/impl/topdeck.py`, the AI half): how much a player wants to draw
//! each card next, and scry / surveil. The cards that use it (cantrips, Sensei's Divining Top, Scroll Rack, Sylvan
//! Library) are card code: PORT(M5) and phase 6.
//!
//! desire(g, p, c) is how much p wants to draw c next:
//!   - a missing combo piece or wish-list card beats everything,
//!   - a land when p is short on lands, a spell when it is flooded,
//!   - otherwise the card's worth to p's AI.
//!
//! Yuriko decks (config top_pref "mv") want the highest mana value on top while Yuriko is available.
//! The top card is the last in `library`.

use super::plans;
use crate::engine::tutors::card_worth;
use crate::flow::Res;
use crate::ids::{CardId, PlayerId};
use crate::state::Game;

/// topdeck.mv_mode: Yuriko decks stack high mana values on top once Yuriko can connect
pub fn mv_mode(g: &Game, p: PlayerId) -> bool {
    let pl = g.player(p);
    if plans::config(pl.key).top_pref != Some("mv") {
        return false;
    }
    pl.cmd_in_zone || pl.perms.iter().any(|&m| g.perm(m).is_cmd)
}

/// topdeck.land_need: 2 when short on lands, 1 when a few more would do, 0 when there are enough
pub fn land_need(g: &Game, p: PlayerId) -> u32 {
    let pl = g.player(p);
    let lands = pl.lands.len() + pl.hand.iter().filter(|&&x| g.db.get(x).land).count();
    if lands < 4 {
        2
    } else if lands < 6 {
        1
    } else {
        0
    }
}

/// topdeck.desire: how much p wants to draw c next
pub fn desire(g: &Game, p: PlayerId, c: CardId) -> f64 {
    let d = g.db.get(c);
    if mv_mode(g, p) {
        return (d.cmc * 10) as f64 + if d.land && land_need(g, p) == 2 { 5.0 } else { 0.0 };
    }
    let wish = plans::wish_list(g, p);
    if let Some(i) = wish.iter().position(|&w| w == c) {
        return 200.0 - i as f64;
    }
    if d.land {
        return match land_need(g, p) {
            2 => 70.0,
            1 => 30.0,
            _ => 5.0,
        };
    }
    card_worth(g, p, c, false)
}

/// topdeck.KEEP: scry keeps cards at least this desirable on top
pub const KEEP: f64 = 35.0;

/// topdeck.arrange: put cards on top of the library, the most desirable on top
pub fn arrange(g: &mut Game, p: PlayerId, cards: &[CardId]) {
    let mut keyed: Vec<(f64, CardId)> = cards.iter().map(|&c| (desire(g, p, c), c)).collect();
    keyed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)); // stable, as Python's sorted
    g.player_mut(p).library.extend(keyed.into_iter().map(|x| x.1));
}

/// topdeck.look: take the top n cards off the library (the top first)
pub fn look(g: &mut Game, p: PlayerId, n: usize) -> Vec<CardId> {
    let lib = &mut g.player_mut(p).library;
    let k = n.min(lib.len());
    (0..k).map(|_| lib.pop().unwrap()).collect()
}

/// topdeck.scry: scry n (surveil false) or surveil n (surveil true): the good cards stay on top in the best order,
/// the rest go to the bottom (scry) or the graveyard (surveil)
pub fn scry(g: &mut Game, p: PlayerId, n: i32, surveil: bool) -> Res {
    if n <= 0 {
        return Ok(());
    }
    let st = g.turn_stamp();
    g.player_mut(p).scry_turn = Some(st); // Desperate Futurescribe: you've scried or surveilled this turn
    // HUMAN(phase 9): a person orders the cards (hc.scry)
    let top = look(g, p, n as usize);
    let (keep, rest): (Vec<CardId>, Vec<CardId>) = top.into_iter().partition(|&c| desire(g, p, c) >= KEEP);
    let pl = g.player_mut(p);
    if surveil {
        pl.gy.extend(rest);
    } else {
        pl.library.splice(0..0, rest);
    }
    arrange(g, p, &keep);
    Ok(())
}
