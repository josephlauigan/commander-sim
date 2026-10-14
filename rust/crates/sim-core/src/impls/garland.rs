//! The Garland deck's plan (`garland`: Dimir, Garland, Royal Kidnapper). Garland himself (enter: an opponent becomes
//! the monarch; whenever an opponent becomes the monarch, steal their creature for as long as they hold the crown;
//! +2/+2 and no sacrifice for creatures you control but don't own) is in `legends.rs`; this module holds what the deck
//! decides around him.
//!
//! - **Never hit the monarch.** Combat damage to the monarch makes the attacker's controller the monarch, which ends
//!   every steal. `attack_targets` removes the monarch from the players the deck attacks (unless the hit is lethal or
//!   nobody else is left).
//! - **Re-crown.** Each entry of Garland crowns a player, so the deck wants him back: `recrown_value` says what
//!   entering again is worth (a better target elsewhere than the creature held now, the crown lost, a steal that has
//!   run a full round). Rescue, Matterbending Mage, Specimen Freighter, All Aboard, Whoosh! and Unsummon read it
//!   (through `best_own_etb`, `blink_value` and `unsummon_options`); the token copies (Relm's Sketching, Echocasting
//!   Symposium) copy him when it is high.
//! - Stats for `--analyze`: `gl_*` counters (steals, rounds held, crown changes, entries, damage by stolen creatures).

use crate::engine::mana::{can_pay, cost_of};
use crate::engine::values::{epow, pval, untargetable};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;
use crate::sym::intern;

pub const KEY: &str = "garland";
pub const GARLAND: &str = "Garland, Royal Kidnapper";

fn stat(g: &mut Game, p: PlayerId, name: &str, n: i64) {
    g.player_mut(p).stat(intern(name), n);
}

/// is permanent m Garland (the commander or a token copy of him)?
pub fn is_garland(g: &Game, m: PermId) -> bool {
    g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == GARLAND)
}

/// p's Garland on the battlefield (the real card first)
pub fn garland_of(g: &Game, p: PlayerId) -> Option<PermId> {
    let mut best = None;
    for &m in &g.player(p).perms {
        let x = g.perm(m);
        if x.on_bf && !x.phased && is_garland(g, m) && x.owner == p {
            if !x.token {
                return Some(m);
            }
            best = Some(m);
        }
    }
    best
}

/// what stealing from opponent q is worth: the value of the creature Garland's trigger would take
pub fn steal_value(g: &Game, p: PlayerId, q: PlayerId) -> f64 {
    crate::impls::legends::garland_pick(g, p, q).map_or(0.0, |t| pval(g, t))
}

/// the value of the creatures p holds through Garland now
fn held_value(g: &Game, p: PlayerId) -> f64 {
    g.garland_list.iter().filter(|x| x.0 == p && g.perm(x.1).on_bf).map(|x| pval(g, x.1)).sum()
}

/// what entering the battlefield again is worth to p's Garland (0: nothing). Crowning the monarch again does nothing,
/// so the target is another opponent: his best creature against what the steals held now are worth (they end when
/// the crown moves), plus the crown itself when we hold it, plus a full round with one holder (the extra card).
pub fn recrown_value(g: &Game, p: PlayerId) -> f64 {
    if g.player(p).key != KEY {
        return 0.0;
    }
    let mon = g.monarch;
    let cand = g
        .opps(p)
        .filter(|&q| g.player(q).alive && Some(q) != mon)
        .map(|q| steal_value(g, p, q))
        .fold(0.0, f64::max);
    if cand <= 0.0 {
        return 0.0;
    }
    let held = held_value(g, p);
    let mut v = cand - held - 1.5;
    if mon == Some(p) {
        v += 3.0; // the crown is ours: every steal is over
    }
    if g.garland_list.iter().any(|x| x.0 == p && g.round > x.3) {
        v += 1.0; // a full round with the same holder
    }
    if held == 0.0 && mon.is_some() && mon != Some(p) {
        v += 1.0; // the crown is out there and we take nothing for it
    }
    v.max(0.0)
}

// ------------------------------------------------------------------ attacks
/// the opponents p attacks: never the monarch (combat damage to it takes the crown and ends the steals), unless that
/// hit is lethal or nobody else is left
pub fn attack_targets(g: &Game, p: PlayerId, opps: Vec<PlayerId>) -> Vec<PlayerId> {
    let Some(mon) = g.monarch.filter(|&m| m != p && opps.contains(&m)) else { return opps };
    if opps.len() <= 1 || g.garland_list.iter().all(|x| x.0 != p) && garland_of(g, p).is_none() {
        return opps;
    }
    let power: i32 = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| {
            let x = g.perm(m);
            g.is_creature(m) && !x.tapped && !x.noatk && !x.sick
        })
        .map(|&m| epow(g, m))
        .sum();
    if power >= g.player(mon).life {
        return opps;
    }
    opps.into_iter().filter(|&q| q != mon).collect()
}

// ------------------------------------------------------------------ casting
/// the deck's cast priority: the bounce and copy cards are held back until entering Garland again is worth it
pub fn garland_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let base = crate::ai::pool::generic_prio(g, p, c);
    let d = g.db.get(c);
    let recrown = matches!(
        &*d.name,
        "Rescue, Pepper Potts"
            | "Matterbending Mage"
            | "Specimen Freighter"
            | "Relm's Sketching"
            | "Echocasting Symposium"
            | "Whoosh!"
            | "Unsummon"
            | "Skycoach Conductor // All Aboard"
    );
    if !recrown || garland_of(g, p).is_none() {
        return base;
    }
    let v = recrown_value(g, p);
    match &*d.name {
        // the creatures are bodies in their own right: a floor of the generic order, more when Garland re-enters
        "Rescue, Pepper Potts" | "Matterbending Mage" | "Specimen Freighter" | "Skycoach Conductor // All Aboard" => {
            if v >= 2.0 { base.max(70) } else { base }
        }
        // spells that exist for the re-entry
        _ => {
            if v >= 2.0 { 66 } else { 0 }
        }
    }
}

/// Unsummon (and the like) aimed at our own Garland at the end of an opponent's turn, to cast him again
pub fn unsummon_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.player(p).key != KEY || post.is_some() || g.active == Some(p) {
        return Ok(vec![]);
    }
    let Some(m) = garland_of(g, p) else { return Ok(vec![]) };
    let v = recrown_value(g, p);
    let (gn, pips) = cost_of(g, p, c);
    // the mana for Unsummon and then Garland (4) must be there on our turn: untapped lands now are for Unsummon only
    if v < 2.5 || !can_pay(g, p, gn, &pips, false) || g.perm(m).token {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: v,
        label: format!("{} (bounce Garland)", g.db.get(c).name),
        act: Some(Action::Plan { f: unsummon_go, arg: c.0 as i64 }),
    }])
}

fn unsummon_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let Some(m) = garland_of(g, p) else { return Ok(false) };
    if !g.player(p).hand.contains(&c) || untargetable(g, m) && g.perm(m).owner != p {
        return Ok(false);
    }
    let ctx = crate::state::Ctx { target: Some(m), ..crate::state::Ctx::default() };
    crate::engine::cast::cast_card(g, p, c, "hand", ctx)
}

/// (bookkeeping) Garland entered under p: cast from the hand or the command zone, or a token copy
pub fn entered(g: &mut Game, p: PlayerId, m: PermId) {
    if g.player(p).key != KEY {
        return;
    }
    stat(g, p, "gl_entries", 1);
    if g.perm(m).token {
        stat(g, p, "gl_copies", 1);
    } else {
        stat(g, p, "gl_casts", 1);
    }
}

/// (bookkeeping) the monarch changed hands
pub fn crown_moved(g: &mut Game, new: PlayerId) {
    for i in 0..g.players.len() {
        let id = g.players[i].id;
        if g.players[i].key == KEY && g.players[i].alive {
            stat(g, id, "gl_crown_changes", 1);
            if new == id {
                stat(g, id, "gl_we_monarch", 1);
            }
        }
    }
}

pub fn register(r: &mut Registry, db: &crate::cards::CardDb) -> Result<(), String> {
    r.card(db, "Unsummon")?.hand_options = Some(unsummon_options);
    Ok(())
}

/// m is a Garland under the Garland deck's seat (other decks that happen to run the card keep their old behaviour)
pub fn is_my_garland(g: &Game, m: PermId) -> bool {
    is_garland(g, m) && g.player(g.perm(m).owner).key == KEY
}
