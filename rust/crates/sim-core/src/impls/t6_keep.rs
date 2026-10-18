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

    /// after two mulligans a hand with 2-4 lands is kept, whatever it holds (a 5-card hand needs lands, not perfection)
    pub fn settle(&self) -> bool {
        self.tries >= 2 && ((2..=4).contains(&self.lands) || (self.lands == 1 && self.eff >= 2.0 && self.spells >= 2))
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
            "Mox Opal" | "Mox Amber" => 0.0, // metalcraft / a legend: not on turn 1 from this hand
            "Sol Ring" => {
                if lands >= 1 {
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
