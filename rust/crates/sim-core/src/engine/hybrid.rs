//! Hybrid mana: a card's `pips` hold the first colour of each two-colour hybrid symbol ({G/W} is "G") and its
//! `hybrid` field names the pairs ("GW"). `cost_of` hands the cost to `resolve`, which keeps the first colours when
//! they can be paid and otherwise tries the other colour of each symbol, so Leyline of the Guildpact
//! ({G/W}{G/U}{B/G}{R/G}) is paid by GGGG, WUBR or any mix.
//!
//! A monocoloured hybrid symbol ({2/W}: Karn, Gilded Guardian, The Reaper, Spectral Procession) is listed as "W2" in the
//! `hybrid` field and its colour stays in `pips`. `resolve` leaves it alone; `cost_of_with` sends a card that has one to
//! `mono_cost`, because {2} is chosen for it before any reduction or tax (CR 601.2b, 601.2f), so the reducers see it.

use crate::engine::mana::{can_pay_spell, cost_of_base, mana_units};
use crate::ids::{CardId, PlayerId};
use crate::state::Game;

/// the two-colour hybrid symbols of a card as (first colour, other colour); a {2/W} ("W2") is not one
fn pairs(hybrid: &str) -> Vec<(char, char)> {
    let cs: Vec<char> = hybrid.chars().collect();
    cs.chunks_exact(2).filter(|w| w[1] != '2').map(|w| (w[0], w[1])).collect()
}

/// the colours of the {2/X} symbols of a card, in cost order ("W2U2" -> W, U)
fn mono_letters(hybrid: &str) -> Vec<char> {
    let cs: Vec<char> = hybrid.chars().collect();
    cs.chunks_exact(2).filter(|w| w[1] == '2').map(|w| w[0]).collect()
}

/// what card c costs p when it has {2/X} symbols (`cost_of_with` calls this for such a card only, `pips` being the coloured
/// part to price). Each {2/X} is paid with its colour or with {2}, and which is chosen before reductions (601.2b), so the
/// generic cost the pipeline starts from includes a chosen {2} and every reducer, floor and tax applies to it as to any
/// generic mana. Masks are tried fewest {2} first (the printed pips first), each through the whole pipeline, and the first
/// cost `can_pay_spell` accepts is the price (its colours are recorded as `resolve` does). When none can be paid, the
/// price is the one with {2} for exactly the colours the player has no untapped source for, or the printed colours
/// when every colour has one (it is then uncastable for lack of mana, not of a colour).
pub fn mono_cost(g: &Game, p: PlayerId, c: CardId, x: i32, pips: &str) -> (u32, String) {
    let d = g.db.get(c);
    // the {2/X} symbols still in the cost (a pip the caller left out is not one any more)
    let mut rest = pips.to_string();
    let mut ls: Vec<char> = vec![];
    for l in mono_letters(&d.hybrid) {
        if let Some(i) = rest.find(l) {
            rest.remove(i);
            ls.push(l);
        }
    }
    if ls.is_empty() {
        return cost_of_base(g, p, c, x, pips);
    }
    let price = |m: u32| {
        let mut cand = pips.to_string();
        let mut twos = 0i32;
        let mut paid: Vec<char> = vec![];
        for (i, &l) in ls.iter().enumerate() {
            if m >> i & 1 == 1 {
                let at = cand.find(l).expect("a letter of ls is in pips");
                cand.remove(at);
                twos += 2;
            } else {
                paid.push(l);
            }
        }
        let (gn, cp) = cost_of_base(g, p, c, x.max(0).saturating_add(twos), &cand);
        (gn, cp, paid)
    };
    let mut masks: Vec<u32> = (0..1u32 << ls.len()).collect();
    masks.sort_by_key(|m| m.count_ones()); // stable: the printed colours first
    for m in masks {
        let (gn, cp, mut paid) = price(m);
        if can_pay_spell(g, p, c, gn, &cp, false) {
            record(g, c, &mut paid);
            return (gn, cp);
        }
    }
    let have = mana_units(g, p, false).iter().fold(crate::cards::Colors::NONE, |a, u| a.union(u.cols));
    let m = ls.iter().enumerate().filter(|(_, l)| !have.has(**l)).fold(0u32, |m, (i, _)| m | 1 << i);
    let (gn, cp, _) = price(m);
    (gn, cp)
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
            record(g, c, &mut chosen);
            *pips = cand;
            return;
        }
    }
}

/// the colours the hybrid symbols of a card were last priced at ("RG" for Vibrance paid {R}{G}) are kept in the game
/// (`Game::hybrid_paid`): a card that cares which colours were spent on it (Vibrance) reads them when it enters
fn record(g: &Game, c: CardId, chosen: &mut [char]) {
    chosen.sort();
    g.hybrid_paid.borrow_mut().insert(c, chosen.iter().collect());
}

/// the colours the hybrid symbols of c were last priced at in this game, sorted ("GG", "GR"); None when it never was
pub fn priced(g: &Game, c: CardId) -> Option<String> {
    g.hybrid_paid.borrow().get(&c).cloned()
}
