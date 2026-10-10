//! Python's `cards/impl/t3.py`: Tier 3 pool decks' cards (the ones the Tier 1 and Sauron decks play so far: Chord of
//! Calling, Elderfang Disciple, Flux Channeler, Inexorable Tide; the rest come with phase 6).

use super::partials::{at_once, first_max, remove_card};
use crate::cards::CardDb;
use crate::engine::mana::total_mana;
use crate::engine::stack::trigger_window;
use crate::engine::tutors::{card_worth, shuffle_library};

use crate::engine::zones::{Enter, discard_cards, enter, min_by, searchable};
use crate::flow::Res;
use crate::hooks::{Event, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::Sym;

/// t3._put_creature: a creature card from the library onto the battlefield: the deck's wish list first, Craterhoof
/// with a wide board, else the most useful
pub fn put_creature(
    g: &mut Game,
    p: PlayerId,
    pred: impl Fn(&Game, CardId) -> bool,
    prefer_hoof: bool,
) -> Res<Option<CardId>> {
    let cs: Vec<CardId> = searchable(g, p).into_iter().filter(|&c| g.db.get(c).creature && pred(g, c)).collect();
    if cs.is_empty() {
        return Ok(None);
    }
    let wish = crate::ai::plans::wish_list(g, p);
    let c = match cs.iter().copied().find(|c| wish.contains(c)) {
        Some(c) => c,
        None => {
            let hoof = cs.iter().copied().find(|&c| &*g.db.get(c).name == "Craterhoof Behemoth");
            let n = g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).count();
            match hoof {
                Some(h) if prefer_hoof && n >= 5 => h,
                _ => first_max(&cs, |c| (card_worth(g, p, c, false), g.db.get(c).cmc)).unwrap(),
            }
        }
    };
    remove_card(&mut g.player_mut(p).library, c);
    shuffle_library(g, p);
    g.player_mut(p).stat("tutored", 1);
    enter(g, p, c, Enter::default())?;
    Ok(Some(c))
}

// ------------------------------------------------------------------ Chord of Calling (t3._x_tutor, from fixes.py)
/// t3._x_tutor's cast priority for Chord of Calling ({X}{G}{G}{G}): with six mana or more
fn chord_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if total_mana(g, p, false) >= 3 + 3 { 60 } else { 0 }
}

/// convoke; X = spare mana and creatures; a creature with mana value X or less onto the battlefield
fn chord(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let x = ctx.x;
    put_creature(g, p, |g, y| (g.db.get(y).cmc as i32) <= x, true)?;
    Ok("gy")
}

// ------------------------------------------------------------------ Elderfang Disciple
/// t3.discard_worst_for: an opponent chooses what to discard: their least useful card
pub fn discard_worst_for(g: &mut Game, q: PlayerId, n: u32) -> Res {
    for _ in 0..n {
        let hand = g.player(q).hand.clone();
        if let Some(c) = min_by(&hand, |c| card_worth(g, q, c, false)) {
            discard_cards(g, q, &[c])?;
        }
    }
    Ok(())
}

/// each opponent discards a card on entry
fn elderfang(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && trigger_window(g, o, Some(src), "each opponent discards 1", None)? {
        for q in g.opps(o).collect::<Vec<_>>() {
            discard_worst_for(g, q, 1)?;
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ proliferate: Flux Channeler, Inexorable Tide

/// noncreature spell: proliferate
fn flux(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if caster == g.perm(src).owner
        && !g.db.get(c).creature
        && trigger_window(g, caster, Some(src), "proliferate", None)?
    {
        crate::impls::common::proliferate(g, caster, 1)?;
    }
    Ok(())
}

/// every spell: proliferate
fn tide(g: &mut Game, src: Src, caster: PlayerId, _c: CardId) -> Res {
    if caster == g.perm(src).owner && trigger_window(g, caster, Some(src), "proliferate", None)? {
        crate::impls::common::proliferate(g, caster, 1)?;
    }
    Ok(())
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Chord of Calling")?;
    c.resolve = Some(chord);
    c.prio = Some(chord_prio);
    let c = r.card(db, "Elderfang Disciple")?;
    c.etb = Some(elderfang);
    at_once(c, Event::Etb, false);
    let c = r.card(db, "Flux Channeler")?;
    c.cast = Some(flux);
    at_once(c, Event::Cast, false);
    let c = r.card(db, "Inexorable Tide")?;
    c.cast = Some(tide);
    at_once(c, Event::Cast, false);
    Ok(())
}
