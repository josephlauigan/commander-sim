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
    /// for as long as the gaining player controls this permanent
    WhileControls(PermId),
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
        let land = g.player(old).lands.contains(&m); // (a creature token land: Astral Dragon's copy)
        g.player_mut(old).lands.retain(|&x| x != m);
        g.player_mut(old).borrowed.retain(|&x| x != m);
        let x = g.perm_mut(m);
        x.owner = new;
        x.attached = None;
        g.player_mut(new).perms.push(m);
        if land {
            g.player_mut(new).lands.push(m);
        }
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
        Duration::Permanent | Duration::WhileControls(_) => {
            g.player_mut(new).borrowed.retain(|&x| x != m);
            if old != new {
                g.perm_mut(m).sick = true;
            }
        }
    }
    if let Duration::WhileControls(anchor) = dur {
        g.control_links.push(ControlLink { stolen: m, anchor, holder: new, prev: old });
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
    if !g.control_links.is_empty() {
        end_links(g);
    }
    true
}

/// End linked control effects immediately when the anchor leaves, phases out, or changes controller.
pub fn end_links(g: &mut Game) {
    let mut ended = vec![];
    for link in std::mem::take(&mut g.control_links) {
        let x = g.perm(link.stolen);
        if !x.on_bf || x.owner != link.holder {
            continue;
        }
        if !g.player(x.orig).alive {
            let q = x.owner;
            g.player_mut(q).perms.retain(|&m| m != link.stolen);
            g.player_mut(q).lands.retain(|&m| m != link.stolen);
            g.perm_mut(link.stolen).on_bf = false;
            g.perm_mut(link.stolen).data.remove(DataKey::PhaseInWith);
            g.bf_ver += 1;
        } else if !g.perm(link.anchor).on_bf
            || g.perm(link.anchor).phased
            || g.perm(link.anchor).owner != link.holder
            || !g.player(link.holder).alive
        {
            ended.push(link);
        } else {
            g.control_links.push(link);
        }
    }
    for link in ended {
        if g.perm(link.stolen).owner != link.holder {
            continue;
        }
        if let Some(live) = g.control_links.iter_mut().find(|x| x.stolen == link.stolen && x.holder == link.holder) {
            // The later effect still holds it. Remember where to return it when that effect ends too.
            if live.prev == link.holder {
                live.prev = link.prev;
            }
            continue;
        }
        let orig = g.perm(link.stolen).orig;
        let to = if g.player(link.prev).alive { link.prev } else { orig };
        if g.perm(link.stolen).phased && g.perm(link.stolen).data.get(DataKey::PhaseInWith).is_none() {
            g.perm_mut(link.stolen).data.set(DataKey::PhaseInWith, Val::Player(link.holder));
        }
        gain_control(g, to, link.stolen, Duration::Permanent);
    }
}

/// the end-step return of creatures borrowed until end of turn by p (control goes back to the owner)
pub fn return_borrowed(g: &mut Game, p: PlayerId) {
    for m in std::mem::take(&mut g.player_mut(p).borrowed) {
        let orig = g.perm(m).orig;
        if g.perm(m).on_bf && g.perm(m).owner == p && g.player(orig).alive {
            g.player_mut(p).perms.retain(|&x| x != m);
            let land = g.player(p).lands.contains(&m);
            g.player_mut(p).lands.retain(|&x| x != m);
            g.perm_mut(m).owner = orig;
            g.player_mut(orig).perms.push(m);
            if land {
                g.player_mut(orig).lands.push(m);
            }
            g.bf_ver += 1;
            if g.perm_mut(m).data.remove(DataKey::TapOnReturn).is_some_and(|v| v.truthy()) {
                g.perm_mut(m).tapped = true; // Ray of Command
            }
        }
    }
    if !g.control_links.is_empty() {
        end_links(g);
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

/// A targeted steal: `new` gains control of the opponent's permanent m with a spell or ability that targets it, so a ward
/// on m is paid first (`ward::pay_ward`; unpaid, the effect is countered). False if nothing changed hands.
pub fn steal(g: &mut Game, new: PlayerId, m: PermId, dur: Duration) -> crate::flow::Res<bool> {
    if !crate::engine::ward::pay_ward(g, new, m)? {
        return Ok(false);
    }
    Ok(gain_control(g, new, m, dur))
}
