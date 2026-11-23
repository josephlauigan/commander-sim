//! Hybrid mana: a card's `pips` hold the first colour of each two-colour hybrid symbol ({G/W} is "G") and its
//! `hybrid` field names the pairs ("GW"). `cost_of` hands the cost to `resolve`, which keeps the first colours when
//! they can be paid and otherwise tries the other colour of each symbol, so Leyline of the Guildpact
//! ({G/W}{G/U}{B/G}{R/G}) is paid by GGGG, WUBR or any mix.

use crate::engine::mana::can_pay_spell;
use crate::ids::{CardId, PlayerId};
use crate::state::Game;
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    /// the colours the hybrid symbols of a card were last priced at ("RG" for Vibrance paid {R}{G}): a card that
    /// cares which colours were spent on it (Vibrance) reads them when it enters
    static PRICED: RefCell<HashMap<CardId, String>> = RefCell::new(HashMap::new());
}

/// the hybrid symbols of a card as (first colour, other colour)
fn pairs(hybrid: &str) -> Vec<(char, char)> {
    let cs: Vec<char> = hybrid.chars().collect();
    cs.chunks_exact(2).map(|w| (w[0], w[1])).collect()
}

/// Settle the hybrid symbols of card c's cost `(gn, pips)` (the cost `cost_of` has built): the printed first colours
/// when `can_pay_spell` accepts them (nothing changes), else the first other choice it accepts (fewest symbols
/// switched first); when none can be paid the cost is left as it was
/// (and not recorded: `priced` is for a cost that was paid).
pub fn resolve(g: &Game, p: PlayerId, c: CardId, gn: u32, pips: &mut String) {
    let d = g.db.get(c);
    if d.hybrid.is_empty() {
        return;
    }
    // the symbols still in the cost (a pip another effect took out is not hybrid any more)
    let mut rest = pips.clone();
    let mut hy: Vec<(char, char)> = vec![];
    for (a, b) in pairs(&d.hybrid) {
        if let Some(i) = rest.find(a) {
            rest.remove(i);
            hy.push((a, b));
        }
    }
    if hy.is_empty() {
        return;
    }
    let mut masks: Vec<u32> = (0..1u32 << hy.len()).collect();
    masks.sort_by_key(|m| m.count_ones()); // stable: the default first
    let mut tried: Vec<Vec<char>> = vec![];
    for m in masks {
        let mut chosen: Vec<char> =
            hy.iter().enumerate().map(|(i, &(a, b))| if m >> i & 1 == 1 { b } else { a }).collect();
        let mut key = chosen.clone();
        key.sort();
        if tried.contains(&key) {
            continue; // the same colours in another order
        }
        tried.push(key);
        let cand: String = if m == 0 { pips.clone() } else { rest.chars().chain(chosen.iter().copied()).collect() };
        if can_pay_spell(g, p, c, gn, &cand, false) {
            record(c, &mut chosen);
            *pips = cand;
            return;
        }
    }
}

fn record(c: CardId, chosen: &mut [char]) {
    chosen.sort();
    PRICED.with(|m| m.borrow_mut().insert(c, chosen.iter().collect()));
}

/// the colours the hybrid symbols of c were last priced at, sorted ("GG", "GR"); None when it never was
pub fn priced(c: CardId) -> Option<String> {
    PRICED.with(|m| m.borrow().get(&c).cloned())
}
