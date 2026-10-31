//! What a creature card is worth to the reanimation spells and the graveyard fillers (Entomb, Buried Alive), for every
//! deck. --- ghave: these were the generic parts of Sephiroth's hand AI (`ai/seph.rs`, Python's `ais.py`), kept when
//! that AI was removed on 2026-10-30 (Ghave, Guru of Spores took the seat and plays from its plan file).

use crate::ids::{CardId, PlayerId};
use crate::impls::mine;
use crate::state::Game;
use crate::tag::Tag;

/// The bomb rating a card gets as a reanimation target. Hornet Queen (four deathtouch fliers, 4 bodies) and Muldrotha
/// carry no `bomb=` tag because the pool decks that play them share the tag line; here they are rated as the bombs
/// they are.
pub fn sbomb(g: &Game, c: CardId) -> i32 {
    let d = g.db.get(c);
    if d.bomb != 0 {
        return d.bomb;
    }
    match &*d.name {
        "Hornet Queen" => 7,
        "Muldrotha, the Gravetide" => 6,
        _ => 0,
    }
}

/// A creature worth reanimating, binning or tutoring for (one threshold for every test: Mikaeus, Muldrotha and Hornet
/// Queen count, as the reanimation spells' own target rule already did)
pub const BOMB_TARGET: i32 = 5;
/// A creature that, once on the battlefield, means the deck has its bomb out (Mikaeus alone is only a loop piece)
pub const BOMB_BOARD: i32 = 6;

/// ais.seph_bval: what a creature card is worth as a reanimation target
pub fn bval(g: &Game, p: PlayerId, c: CardId) -> f64 {
    let d = g.db.get(c);
    let t = &d.tags;
    let mut v = if sbomb(g, c) != 0 {
        sbomb(g, c) as f64
    } else if d.creature {
        d.pow as f64
    } else {
        0.0
    };
    if t.has(Tag::Normgc) && g.opps(p).any(|q| g.player(q).key == "najeela") {
        v += 2.0;
    }
    if t.has(Tag::Mother) && g.opps(p).any(|q| g.player(q).key == "sauron") {
        v += 1.0;
    }
    if t.has(Tag::Bahamut) {
        // Mega Flare's reach
        let mv: u32 = g.player(p).perms.iter().filter_map(|&m| g.perm(m).cd).map(|c| g.db.get(c).cmc).sum();
        v += 1.5f64.min(mv as f64 / 15.0);
    }
    v
}

/// library creatures that are the missing piece of a loop (Entomb/Buried Alive bin these first: Finks, Triskelion,
/// Melira). --- ghave: every deck (it was Sephiroth's only); only the ghave list plays the loops' pieces
pub fn fill_pieces(g: &Game, p: PlayerId) -> Vec<CardId> {
    let need = mine::loop_need(g, p, &[]);
    if need.is_empty() {
        return vec![];
    }
    g.player(p)
        .library
        .iter()
        .copied()
        .filter(|&c| g.db.get(c).creature && need.contains(&&*g.db.get(c).name))
        .collect()
}

/// what a graveyard-filling spell may bin: the missing loop pieces, then every bomb
pub fn fill_targets(g: &Game, p: PlayerId) -> Vec<CardId> {
    let need = fill_pieces(g, p);
    g.player(p)
        .library
        .iter()
        .copied()
        .filter(|&c| g.db.get(c).creature && (need.contains(&c) || sbomb(g, c) >= BOMB_TARGET))
        .collect()
}

/// a reanimation target (a bomb) in p's graveyard
pub fn own_bomb_in_gy(g: &Game, p: PlayerId) -> bool {
    g.player(p).gy.iter().any(|&c| g.db.get(c).creature && sbomb(g, c) >= BOMB_TARGET)
}
