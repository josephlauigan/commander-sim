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
//! - Mox Jasper: 1 when the hand can cast a Dragon creature by turn 2 without it (`dragon_cast`), else 0;
//! - Lion's Eye Diamond, rituals and spirit guides: 0 (they only fuel a spell).

use crate::cards::Colors;
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
pub fn is_piece(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    !d.land
        && (d.tag(Tag::Rock)
            || d.tag(Tag::Dork)
            || d.tag(Tag::Fastmana)
            || d.tag(Tag::Lr)
            || d.tag(Tag::Chromemox)
            || matches!(d.name.as_ref(), "Lotus Petal" | "Chrome Mox" | "Mox Diamond"))
}

/// what one mana piece of the hand makes by turn 2 from this hand (the table in the module doc), and whether it makes it
/// without a land. `lands`, `spells` and `starts` are the hand's: its lands, its cards that are neither lands nor mana
/// pieces, and whether a Lotus Petal can pay for the first rock.
fn piece_value(g: &Game, p: PlayerId, c: CardId, lands: usize, spells: usize, starts: bool) -> (f64, bool) {
    let d = g.db.get(c);
    match nm(g, c) {
        "Mox Diamond" => (if lands >= 2 { 0.5 } else { 0.0 }, false),
        "Chrome Mox" => (if spells >= 2 { 1.0 } else { 0.0 }, false),
        "Lotus Petal" => (0.7, true),
        // needs a legendary creature or planeswalker: the Rograkh decks cast their {0} commander first
        "Mox Amber" => (if g.player(p).key.starts_with("rograkh") { 1.0 } else { 0.5 }, false),
        "Mox Opal" => (0.0, false), // metalcraft: three artifacts
        // only while you control a Dragon: it makes mana from the turn after the hand can cast one, so it counts (like a
        // two-mana rock cast on turn 2) when that is turn 2 or earlier
        "Mox Jasper" => (if dragon_cast(g, p, lands).is_some_and(|x| x.0 <= 2) { 1.0 } else { 0.0 }, false),
        "Sol Ring" => (if lands >= 1 || starts { 2.0 } else { 0.0 }, false),
        _ if d.cmc == 0 => (1.0, true),
        _ if d.cmc == 1 => (if lands >= 1 { if d.tag(Tag::Dork) { 0.8 } else { 1.0 } } else { 0.0 }, false),
        _ if d.cmc == 2 => (if lands >= 2 { if d.tag(Tag::Dork) { 0.8 } else { 1.0 } } else { 0.0 }, false),
        _ => (0.0, false),
    }
}

/// The first turn (1 to 10) on which this hand can cast a Dragon creature (a changeling counts) without Mox Jasper, and
/// which Dragon it is (the cheapest). `cap` is how many lands the hand may play, one a turn. Mana is stepped the way
/// the hint's `Curve` steps it: Ancient Tomb and City of Traitors give two, a zero-cost piece gives its mana the turn it
/// is cast, any other piece from the next turn, and Lotus Petal is one mana, once (kept for the Dragon or spent on the
/// first rock, whichever is earlier). Printed costs only: colours and cost reductions are ignored.
pub fn dragon_cast(g: &Game, p: PlayerId, cap: usize) -> Option<(usize, CardId)> {
    let hand = &g.player(p).hand;
    let lands = hand.iter().filter(|&&c| g.db.get(c).land).count();
    let all: Vec<CardId> = hand.iter().copied().filter(|&c| is_piece(g, c)).collect();
    let spells = hand.len() - lands - all.len();
    let starts = all.iter().any(|&c| g.db.get(c).cmc == 0 && nm(g, c) == "Lotus Petal");
    let doubles =
        hand.iter().filter(|&&c| g.db.get(c).land && matches!(nm(g, c), "Ancient Tomb" | "City of Traitors")).count();
    let dragon = hand
        .iter()
        .copied()
        .filter(|&c| g.db.get(c).creature && crate::impls::t45_urdragon::dragon_card(g, c))
        .min_by_key(|&c| g.db.get(c).cmc)?;
    let need = g.db.get(dragon).cmc as f64;
    let petal = all.iter().any(|&c| nm(g, c) == "Lotus Petal");
    // (cost, mana made) of the other pieces, cheapest first; Jasper is left out so nothing loops
    let mut pieces: Vec<(f64, f64)> = all
        .iter()
        .filter(|&&c| !matches!(nm(g, c), "Mox Jasper" | "Lotus Petal"))
        .map(|&c| (g.db.get(c).cmc as f64, piece_value(g, p, c, lands, spells, starts).0))
        .filter(|x| x.1 > 0.0)
        .collect();
    pieces.sort_by(|a, b| a.0.total_cmp(&b.0).then(b.1.total_cmp(&a.1)));
    let run = |petal_early: bool| -> Option<usize> {
        let mut left = pieces.clone();
        let mut online: Vec<(usize, f64)> = vec![]; // (first turn it makes mana, mana)
        let mut petal_left = petal;
        for t in 1..=10usize {
            let n = t.min(cap);
            let mut base = n as f64 + doubles.min(n) as f64;
            // zero-cost pieces first: they cost nothing and make their mana at once
            left.retain(|&(cost, v)| {
                if cost == 0.0 {
                    base += v;
                    false
                } else {
                    true
                }
            });
            base += online.iter().filter(|x| x.0 <= t).map(|x| x.1).sum::<f64>();
            if need <= base + petal_left as u8 as f64 + 1e-9 {
                return Some(t);
            }
            let mut rem = base + (petal_early && petal_left) as u8 as f64;
            let before = rem;
            let mut i = 0;
            while i < left.len() {
                if left[i].0 <= rem + 1e-9 {
                    rem -= left[i].0;
                    online.push((t + 1, left[i].1));
                    left.remove(i);
                } else {
                    i += 1;
                }
            }
            if petal_early && petal_left && before - rem > base + 1e-9 {
                petal_left = false;
            }
        }
        None
    };
    let t = match (run(false), if petal { run(true) } else { None }) {
        (Some(a), Some(b)) => a.min(b),
        (a, b) => a.or(b)?,
    };
    Some((t, dragon))
}

/// the hand's mana pieces with what each makes by turn 2 (0.0: nothing, from this hand), for explaining a hand
pub fn piece_values(g: &Game, p: PlayerId) -> Vec<(CardId, f64)> {
    let hand = &g.player(p).hand;
    let lands = hand.iter().filter(|&&c| g.db.get(c).land).count();
    let pieces: Vec<CardId> = hand.iter().copied().filter(|&c| is_piece(g, c)).collect();
    let spells = hand.len() - lands - pieces.len();
    let starts = pieces.iter().any(|&c| g.db.get(c).cmc == 0 && nm(g, c) == "Lotus Petal");
    pieces.iter().map(|&c| (c, piece_value(g, p, c, lands, spells, starts).0)).collect()
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
        let before = eff;
        let (v, no_land) = piece_value(g, p, c, lands, spells, starts);
        eff += v;
        free += no_land as usize;
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

// ------------------------------------------------------------------ the colours a hand can make
/// the colours a land card taps for (`Tag::C`: letters, `A` the player's whole identity, `C` none)
pub fn tag_cols(g: &Game, p: PlayerId, c: CardId) -> Colors {
    match g.db.get(c).tags.str(Tag::C) {
        Some("A") => g.player(p).ident,
        Some(s) => Colors::from_letters(s),
        None => Colors::NONE,
    }
}

/// the colours of mana the card makes once it is in play: a rock (`rock` = `amount:colours`), a dork (`dork` =
/// colours), Elvish Spirit Guide ({G} from the hand); Chrome Mox is none
pub fn piece_cols(g: &Game, p: PlayerId, c: CardId) -> Colors {
    let d = g.db.get(c);
    let ident = g.player(p).ident;
    let letters = |s: &str| if s == "A" { ident } else { Colors::from_letters(s) };
    if let Some(r) = d.tags.str(Tag::Rock) {
        return letters(r.rsplit(':').next().unwrap_or(""));
    }
    if let Some(r) = d.tags.str(Tag::Dork) {
        return letters(r);
    }
    match &*d.name {
        "Elvish Spirit Guide" => Colors::from_letters("G"),
        // Chrome Mox exiles the card it imprints, so it cannot make the colour of the card that needs it: no source
        "Chrome Mox" => Colors::NONE,
        _ => Colors::NONE,
    }
}

pub fn covers(have: Colors, need: Colors) -> bool {
    "WUBRG".chars().all(|ch| !need.has(ch) || have.has(ch))
}

/// the colours the hand can make: its lands (a fetch land: the colours of the lands left in the library), then the rocks
/// and dorks it can cast with what it has so far (their colours and a Mox's own cost, repeated until nothing is added)
pub fn hand_sources(g: &Game, p: PlayerId) -> Colors {
    let hand = &g.player(p).hand;
    let lib_cols = || {
        g.player(p)
            .library
            .iter()
            .filter(|&&c| g.db.get(c).land && !g.db.get(c).tag(Tag::F))
            .fold(Colors::NONE, |a, &c| a.union(tag_cols(g, p, c)))
    };
    let mut have = Colors::NONE;
    let mut n_lands = 0;
    for &c in hand.iter().filter(|&&c| g.db.get(c).land) {
        n_lands += 1;
        have = have.union(if g.db.get(c).tag(Tag::F) { lib_cols() } else { tag_cols(g, p, c) });
    }
    let pieces: Vec<CardId> = hand.iter().copied().filter(|&c| !g.db.get(c).land).collect();
    let mut free = 0; // zero-cost pieces make the mana for a generic cost
    for _ in 0..4 {
        let mut changed = false;
        for &c in &pieces {
            let d = g.db.get(c);
            let cols = piece_cols(g, p, c);
            if cols.is_empty() || (covers(have, cols)) {
                continue;
            }
            if covers(have, d.colors) && (d.generic as usize) <= n_lands + free {
                have = have.union(cols);
                if d.cmc == 0 {
                    free += 1;
                }
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    have
}

fn in_scope(g: &Game, c: CardId, scope: Option<&[String]>) -> bool {
    !g.db.get(c).land && scope.is_none_or(|s| s.iter().any(|n| **n == *g.db.get(c).name))
}

/// colours the hand's spells (those named in `scope`, else every nonland card) need and no source in the hand makes
pub fn colour_gaps(g: &Game, p: PlayerId, scope: Option<&[String]>) -> usize {
    let have = hand_sources(g, p);
    let need = g
        .player(p)
        .hand
        .iter()
        .filter(|&&c| in_scope(g, c, scope))
        .fold(Colors::NONE, |a, &c| a.union(g.db.get(c).colors));
    "WUBRG".chars().filter(|&ch| need.has(ch) && !have.has(ch)).count()
}

/// how many of the hand's spells (those named in `scope`, else every nonland card) have a source of each colour they need
pub fn colour_castable(g: &Game, p: PlayerId, scope: Option<&[String]>) -> usize {
    let have = hand_sources(g, p);
    g.player(p).hand.iter().filter(|&&c| in_scope(g, c, scope) && covers(have, g.db.get(c).colors)).count()
}
