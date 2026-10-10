//! Python's `cards/impl/galadriel.py`: the Galadriel deck's cards (Elvish Archdruid so far, which Lathril's Elves
//! play too; the deck's AI and the rest come with phase 6).

use super::partials::at_once;
use crate::cards::CardDb;
use crate::engine::values::has_type;
use crate::flow::Res;
use crate::hooks::{Event, Registry, Src};
use crate::ids::{PermId, PlayerId};
use crate::state::Game;

/// galadriel._elves: the Elf creatures p controls
fn elves(g: &Game, p: PlayerId) -> u32 {
    g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased && has_type(g, m, "elf")).count() as u32
}

/// {T}: {G} for each Elf you control (replaces t1's count, which counted every Elf permanent)
fn archdruid_mana(g: &Game, p: PlayerId, _m: PermId) -> u32 {
    elves(g, p)
}

/// galadriel's enters hook does nothing: it makes the card's code live (Python's `CI.live`)
fn archdruid_live(_g: &mut Game, _src: Src, _p: PlayerId, _m: PermId) -> Res {
    Ok(())
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Elvish Archdruid")?;
    c.etb = Some(archdruid_live);
    at_once(c, Event::Etb, true);
    c.dyn_mana_perm = Some(archdruid_mana);
    Ok(())
}
