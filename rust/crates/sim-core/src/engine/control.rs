//! Control change and goad as engine verbs (t6-e-control).
//!
//! `gain_control` moves a permanent between players' `perms` lists with a duration; `goad` marks a creature with the
//! goading player and a duration. Card code calls these (Dack's donation, Loki's Scepter, Volatile Stormdrake's
//! exchange, Sudden Substitution, Zealous Conscripts-style steals via `marchesa::steal`) instead of moving
//! permanents itself. The reading side of goad (`goaded_by`, `goad_attackers`) stays in `impls::t0_zurgo`.

use crate::ids::{PermId, PlayerId};
use crate::state::*;

/// how long a control-changing effect lasts
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Duration {
    /// until end of turn: the creature is untapped and hasty, and goes back at the end step (Threaten effects)
    EndOfTurn,
    /// for the rest of the game (gain control without a duration, a donation): summoning sick under the new controller
    Permanent,
}

/// how long a goad lasts
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GoadFor {
    /// until the goading player's next turn (the printed duration of most goad cards)
    NextTurn,
    /// for the rest of the game (Dack Fayden's creatures)
    Forever,
}

/// `gain_control`: `new` gains control of m. Returns false if m is not on the battlefield. Control that was borrowed
/// until end of turn is forgotten when a permanent change follows (the later effect wins); a creature already under
/// `new`'s control only gets the longer duration. Attachments the creature carries stay attached; an Aura or
/// Equipment itself is unattached when it changes hands.
pub fn gain_control(g: &mut Game, new: PlayerId, m: PermId, dur: Duration) -> bool {
    if !g.perm(m).on_bf {
        return false;
    }
    let old = g.perm(m).owner;
    if old != new {
        g.player_mut(old).perms.retain(|&x| x != m);
        g.player_mut(old).borrowed.retain(|&x| x != m);
        let x = g.perm_mut(m);
        x.owner = new;
        x.attached = None;
        g.player_mut(new).perms.push(m);
        g.bf_ver += 1;
    }
    match dur {
        Duration::EndOfTurn => {
            let x = g.perm_mut(m);
            x.tapped = false;
            x.sick = false;
            if !g.player(new).borrowed.contains(&m) && g.perm(m).orig != new {
                g.player_mut(new).borrowed.push(m);
            }
        }
        Duration::Permanent => {
            g.player_mut(new).borrowed.retain(|&x| x != m);
            if old != new {
                g.perm_mut(m).sick = true;
            }
        }
    }
    if old != new {
        crate::glog!(
            g,
            "    {} gains control of {} ({}){}",
            g.player(new).name,
            g.perm(m).name,
            g.player(old).name,
            if dur == Duration::EndOfTurn { " until end of turn" } else { "" }
        );
    }
    true
}

/// the end-step return of creatures borrowed until end of turn by p (control goes back to the owner)
pub fn return_borrowed(g: &mut Game, p: PlayerId) {
    for m in std::mem::take(&mut g.player_mut(p).borrowed) {
        let orig = g.perm(m).orig;
        if g.perm(m).on_bf && g.perm(m).owner == p && g.player(orig).alive {
            g.player_mut(p).perms.retain(|&x| x != m);
            g.perm_mut(m).owner = orig;
            g.player_mut(orig).perms.push(m);
            g.bf_ver += 1;
            if g.perm_mut(m).data.remove(DataKey::TapOnReturn).is_some_and(|v| v.truthy()) {
                g.perm_mut(m).tapped = true; // Ray of Command
            }
        }
    }
}

/// `goad`: goader goads m. `GoadFor::NextTurn` lapses when the goader's turn count moves on; `Forever` never does
/// (while the goader is alive, as for any goad).
pub fn goad(g: &mut Game, goader: PlayerId, m: PermId, dur: GoadFor) {
    let stamp = match dur {
        GoadFor::NextTurn => g.player(goader).turns as i64,
        GoadFor::Forever => -1,
    };
    g.perm_mut(m).data.set(DataKey::Goaded, Val::List(vec![Val::Player(goader), Val::Int(stamp)]));
}
