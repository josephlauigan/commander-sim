//! Python's `cards/impl/jodah.py`: the Jodah deck's cards (Mother and Giver of Runes so far, which Light-Paws plays
//! too; Jodah's protection AI and the rest come with phase 6).

use crate::cards::CardDb;
use crate::flow::Res;
use crate::hooks::Registry;
use crate::ids::{PermId, PlayerId};
use crate::state::Game;

/// jodah._runes_home: Jodah's AI keeps Mother and Giver of Runes home (untapped, to protect Jodah) instead of
/// attacking with them; other decks attack with them as usual
fn runes_home(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    if g.player(p).key == "jodah" {
        g.perm_mut(m).noatk = true;
    }
    Ok(())
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    for name in ["Mother of Runes", "Giver of Runes"] {
        r.card(db, name)?.as_enters = Some(runes_home);
    }
    Ok(())
}
