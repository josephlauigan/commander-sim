//! Tier 6 piece care: which cards are pieces of the deck's modeled lines, and which library cards would complete one.
//! Used by the card code of tutors and free casts (Gamble, Beseech the Mirror, Chrome Mox, Wishclaw Talisman,
//! Invasion of Ikoria) so that a Tier 6 deck does not bin or misplay the pieces of its own lines.

use crate::ids::{CardId, PlayerId};
use crate::state::Game;

use super::combos;

/// the piece groups of p's modeled lines: the lines whose every piece group has a card in p's deck (or among its
/// commanders), quiet lines included, without the quiet marker
fn line_groups(g: &Game, p: PlayerId) -> Vec<Vec<&'static [&'static str]>> {
    let pl = g.player(p);
    let mut deck: std::collections::HashSet<&str> = pl.deck_names.iter().map(|&c| &*g.db.get(c).name).collect();
    deck.extend(pl.commanders().map(|c| &*g.db.get(c).name));
    combos::all()
        .iter()
        .map(|cmb| cmb.groups.iter().copied().filter(|grp| *grp != combos::QUIET).collect::<Vec<_>>())
        .filter(|v| !v.is_empty() && v.iter().all(|grp| grp.iter().any(|n| deck.contains(n))))
        .collect()
}

/// is this card name in a piece group of one of p's lines?
pub fn is_line_piece(g: &Game, p: PlayerId, name: &str) -> bool {
    line_groups(g, p).iter().any(|grps| grps.iter().any(|grp| grp.contains(&name)))
}

/// the names p holds toward a line: hand, battlefield (not phased out), commander in the command zone
pub fn held_names(g: &Game, p: PlayerId) -> Vec<String> {
    let pl = g.player(p);
    let mut v: Vec<String> = pl.hand.iter().map(|&c| g.db.get(c).name.to_string()).collect();
    v.extend(
        pl.perms
            .iter()
            .filter(|&&m| !g.perm(m).phased)
            .filter_map(|&m| g.perm(m).cd)
            .map(|c| g.db.get(c).name.to_string()),
    );
    v.extend(pl.lands.iter().filter(|&&l| g.land(l).on_bf).map(|&l| g.db.get(g.land(l).cd).name.to_string()));
    v.extend(pl.cmds_in_zone().map(|c| g.db.get(c).name.to_string()));
    v
}

/// would a card named `name` complete a line of p's: every other group of some line holds a card of `held`?
pub fn completes_line(g: &Game, p: PlayerId, name: &str, held: &[String]) -> bool {
    line_groups(g, p).iter().any(|grps| {
        grps.iter().any(|grp| grp.contains(&name))
            && grps.iter().filter(|grp| !grp.contains(&name)).all(|grp| grp.iter().any(|n| held.iter().any(|h| h == n)))
    })
}

/// the library cards that would complete a line given what p holds now
pub fn completers_in_library(g: &Game, p: PlayerId) -> Vec<CardId> {
    let held = held_names(g, p);
    g.player(p)
        .library
        .iter()
        .copied()
        .filter(|&c| !g.db.get(c).land && completes_line(g, p, &g.db.get(c).name, &held))
        .collect()
}
