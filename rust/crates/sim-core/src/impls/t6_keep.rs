//! What an opening hand's mana really is, for the Tier 6 `keep` slots (`t6_plan_*::keep`).
//!
//! The deck plans used to count every rock as "a source" the same as a land. That keeps 1-landers whose "fast mana" is
//! a Mox Diamond (needs a second land to discard), a Chrome Mox (needs a spare card to imprint), a Mana Vault or Sol
//! Ring (need a land to cast) and sits on 1-3 lands for many rounds. Here each piece counts for the mana it makes by
//! turn 2 from THIS hand, without drawing anything:
//!
//! - a land: 1; a zero-cost Mox or Lotus Petal: 1 (a Petal once, so 0.7); Sol Ring: 2 once a land casts it;
//! - Mana Vault, a one-mana rock or dork: 1 once a land casts it (a dork is summoning sick: 0.8);
//! - a two-mana rock (Talisman, Signet): 1 with two lands, else 0;
//! - Mox Diamond: needs a second land to discard, and then only brings the mana a turn earlier: 0.5 with two lands;
//! - Chrome Mox: 1 when two other non-land, non-mana cards can imprint one of them (a spare), else 0;
//! - Lion's Eye Diamond, rituals and spirit guides: 0 (they only fuel a spell).

use crate::ids::{CardId, PlayerId};
use crate::state::Game;
use crate::tag::Tag;

pub struct Mana {
    pub lands: usize,
    /// lands plus every mana piece at the value above
    pub eff: f64,
    /// the pieces that make mana without a land (zero-cost Moxen, Lotus Petal)
    pub free: usize,
    /// cards that are neither lands nor mana pieces
    pub spells: usize,
    /// how many mana pieces count for something
    pub n_mana: usize,
    /// which draw this is: 0 the first seven, 1 the second, 2 and 3 the hands that bottom cards
    pub tries: i64,
}

impl Mana {
    /// the mana a 1-lander needs from its hand: later (smaller) hands are kept more easily
    pub fn need_one_lander(&self) -> f64 {
        match self.tries {
            0 => 3.0,
            1 => 2.5,
            _ => 2.0,
        }
    }

    /// after the first mulligan a hand with 2-4 lands is kept, whatever it holds (the next one is smaller: a hand
    /// needs lands, not perfection); after two, a 1-lander with 1.0 more mana and two spells is kept too
    pub fn settle(&self) -> bool {
        (self.tries >= 1 && (2..=4).contains(&self.lands))
            || (self.tries >= 2 && self.lands == 1 && self.eff >= 2.0 && self.spells >= 2)
    }
}

fn nm(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

/// is this a mana piece of the hand (a rock or dork or ritual-like source) rather than a spell?
fn is_piece(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    !d.land
        && (d.tag(Tag::Rock)
            || d.tag(Tag::Dork)
            || d.tag(Tag::Fastmana)
            || d.tag(Tag::Lr)
            || d.tag(Tag::Chromemox)
            || matches!(d.name.as_ref(), "Lotus Petal" | "Chrome Mox" | "Mox Diamond"))
}

pub fn mana(g: &Game, p: PlayerId) -> Mana {
    let hand = &g.player(p).hand;
    let lands = hand.iter().filter(|&&c| g.db.get(c).land).count();
    let pieces: Vec<CardId> = hand.iter().copied().filter(|&c| is_piece(g, c)).collect();
    let spells = hand.len() - lands - pieces.len();
    let mut eff = lands as f64;
    let mut free = 0;
    // a zero-cost piece (Lotus Petal) pays for a Sol Ring or a 1-mana rock without any land
    let starts = pieces.iter().any(|&c| g.db.get(c).cmc == 0 && nm(g, c) == "Lotus Petal");
    let mut n_mana = 0;
    for &c in hand.iter().filter(|&&c| g.db.get(c).land) {
        if matches!(nm(g, c), "Ancient Tomb" | "City of Traitors") {
            eff += 1.0; // two mana from one land
        }
    }
    for &c in &pieces {
        let d = g.db.get(c);
        let n = nm(g, c);
        let before = eff;
        eff += match n {
            "Mox Diamond" => {
                if lands >= 2 {
                    0.5
                } else {
                    0.0
                }
            }
            "Chrome Mox" => {
                if spells >= 2 {
                    1.0
                } else {
                    0.0
                }
            }
            "Lotus Petal" => {
                free += 1;
                0.7
            }
            // needs a legendary creature or planeswalker: the Rograkh decks cast their {0} commander first
            "Mox Amber" => {
                if g.player(p).key.starts_with("rograkh") {
                    1.0
                } else {
                    0.5
                }
            }
            "Mox Opal" => 0.0, // metalcraft: three artifacts
            "Sol Ring" => {
                if lands >= 1 || starts {
                    2.0
                } else {
                    0.0
                }
            }
            _ if d.cmc == 0 => {
                free += 1;
                1.0
            }
            _ if d.cmc == 1 => {
                if lands >= 1 {
                    if d.tag(Tag::Dork) { 0.8 } else { 1.0 }
                } else {
                    0.0
                }
            }
            _ if d.cmc == 2 => {
                if lands >= 2 {
                    if d.tag(Tag::Dork) { 0.8 } else { 1.0 }
                } else {
                    0.0
                }
            }
            _ => 0.0,
        };
        n_mana += (eff > before) as usize;
    }
    Mana { lands, eff, free, spells, n_mana, tries: g.player(p).stats.get("mull_try").copied().unwrap_or(0) }
}

/// Tier 6's card to bottom after a mulligan (London: draw seven, bottom one per mulligan past the free one). The shared
/// rule bottoms the most expensive spell, which can be the combo piece or tutor the hand was kept for. Here: prefer a
/// card whose loss leaves a hand the deck's own keep rule still takes; among those, bottom the card worth least to the
/// plan: not a tutor target (the deck's wish list, best first), not a mana piece, a land only past the third, the
/// deck's lowest play priority, the most expensive last.
pub fn bottom_pick(g: &Game, p: PlayerId) -> Option<CardId> {
    let hand = g.player(p).hand.clone();
    if hand.is_empty() {
        return None;
    }
    let wish = crate::ai::plans::wish_raw(g, p);
    let lands = hand.iter().filter(|&&c| g.db.get(c).land).count();
    let value = |c: CardId| {
        let d = g.db.get(c);
        if d.land {
            return if lands > 3 { -1.0 } else { 30.0 };
        }
        let wished = wish.iter().position(|&x| x == c).map_or(0.0, |i| 50.0 - i.min(20) as f64);
        let piece = if is_piece(g, c) { 20.0 } else { 0.0 };
        let prio = super::t6_plans::prio(g, p, c).unwrap_or(0.0) / 10.0;
        wished + piece + prio - d.cmc as f64 * 0.1
    };
    let still_keeps = |c: CardId| {
        let mut h = g.clone();
        let pl = h.player_mut(p);
        if let Some(i) = pl.hand.iter().position(|&x| x == c) {
            pl.hand.remove(i);
        }
        super::t6_plans::keep(&h, p).unwrap_or(true)
    };
    let ok: Vec<CardId> = hand.iter().copied().filter(|&c| still_keeps(c)).collect();
    let pool = if ok.is_empty() { &hand } else { &ok };
    pool.iter().copied().min_by(|&a, &b| value(a).total_cmp(&value(b)))
}
