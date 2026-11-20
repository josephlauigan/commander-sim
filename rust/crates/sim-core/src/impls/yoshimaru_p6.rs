//! Cards of the Yoshimaru / Tymna blink deck whose Oracle text needs card code beyond the compiled patterns (the P6 set):
//! Sunpearl Kirin. (Haliya, Guided by Light is in `t45_heliod_zur.rs` and The Meathook Massacre in
//! `t45_urza_yawgmoth.rs`, where those cards were already modelled.) Each card's text is in the doc comment above its
//! code.

use crate::cards::CardDb;
use crate::cards::Colors;
use crate::engine::stack::trigger_window;
use crate::engine::values::{pval, untargetable_by_its_controller};
use crate::engine::zones::{bounce, draw, leave};
use crate::flow::Res;
use crate::hooks::{Registry, Src};
use crate::human::{Target, choices, legal};
use crate::ids::{PermId, PlayerId};
use crate::state::Game;

const KIRIN: &str = "Sunpearl Kirin";

/// what Sunpearl Kirin can return: a permanent, or one of the Treasure, Food and Clue tokens the engine keeps as
/// counters on the player (a Treasure is a nonland permanent too)
#[derive(Clone, Copy, PartialEq)]
enum Back {
    Perm(PermId),
    Token(&'static str),
}

fn live(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased
}

/// another nonland permanent p controls (a creature that is also a land, by a card or by a token, is a land)
fn nonland(g: &Game, p: PlayerId, m: PermId) -> bool {
    live(g, m)
        && g.perm(m).owner == p
        && !g.player(p).lands.contains(&m)
        && g.perm(m).cd.is_none_or(|c| !g.db.get(c).land)
        && !untargetable_by_its_controller(g, m, Colors::from_letters("W"))
}

fn candidates(g: &Game, p: PlayerId, src: PermId) -> Vec<Back> {
    let mut v: Vec<Back> =
        g.player(p).perms.iter().copied().filter(|&m| m != src && nonland(g, p, m)).map(Back::Perm).collect();
    let pl = g.player(p);
    for (kind, n) in [("Treasure", pl.treasures), ("Food", pl.foods), ("Clue", pl.clues)] {
        if n > 0 {
            v.push(Back::Token(kind));
        }
    }
    v
}

fn still_there(g: &Game, p: PlayerId, src: PermId, b: Back) -> bool {
    match b {
        Back::Perm(m) => m != src && nonland(g, p, m),
        Back::Token(k) => {
            let pl = g.player(p);
            match k {
                "Treasure" => pl.treasures > 0,
                "Food" => pl.foods > 0,
                _ => pl.clues > 0,
            }
        }
    }
}

// ================================================================== Sunpearl Kirin
/// **Sunpearl Kirin** ({1}{W} 2/1 Kirin, flash, flying): "When this creature enters, return up to one other target
/// nonland permanent you control to its owner's hand. If it was a token, draw a card." The compiled text had no
/// target and drew a card unconditionally; the card's tags carry no draw (data/card_overrides.toml) and this code does
/// the whole trigger. A permanent goes back to its owner's hand (a commander as the engine's bounce does); a token
/// ceases to exist and its controller draws a card; a Treasure, Food or Clue token (counters on the player in the
/// engine) is returned the same way. The trigger does not depend on Kirin still being on the battlefield, and the
/// permanent must be a legal target for it (`values::untargetable_by_its_controller`: shroud, Lightning Greaves,
/// Whispersilk Cloak, protection from white; hexproof does not stop its controller). The target is chosen as the trigger
/// resolves, after the table's window (the engine
/// probes a trigger up to its first window, so a question before it would be asked twice; the same order every card with
/// a chosen target uses): it does nothing if there is nothing to return. A person chooses among every candidate, or none
/// ("up to one"). The AI returns the permanent whose enter effect is worth replaying most (`mine::flicker_worth`, at
/// least 2, the Displacer Kitten threshold; never a commander, a stolen permanent or one carrying counters it would
/// lose), or a token worth less than the card it draws (`pval` under 1: a 1/1 or 2/2 token); it never returns a
/// Treasure, Food or Clue.
fn kirin_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || g.perm(src).owner != p {
        return Ok(()); // the trigger does not depend on Kirin still being there
    }
    let human = crate::human::is_human(g, p);
    let mut ai_pick = None;
    if human {
        if candidates(g, p, src).is_empty() {
            return Ok(());
        }
    } else {
        ai_pick = ai_choice(g, p, src);
        if ai_pick.is_none() {
            return Ok(());
        }
    }
    if !trigger_window(
        g,
        p,
        Some(src),
        "return up to one other nonland permanent you control to its owner's hand",
        None,
    )? {
        return Ok(());
    }
    let cands: Vec<Back> = candidates(g, p, src);
    let pick = if human {
        if cands.is_empty() {
            return Ok(());
        }
        let labels: Vec<String> = cands
            .iter()
            .map(|b| match *b {
                Back::Perm(m) => legal::describe_target(g, p, Target::Perm(m)),
                Back::Token(k) => format!("a {k} token"),
            })
            .collect();
        let prompt = "Sunpearl Kirin: return which other nonland permanent you control to its owner's hand?";
        choices::choose(g, p, "target", prompt, &labels, Some("none"), None)?.map(|i| cands[i])
    } else {
        ai_pick.filter(|b| still_there(g, p, src, *b))
    };
    let Some(b) = pick.filter(|b| still_there(g, p, src, *b)) else { return Ok(()) };
    let token = match b {
        Back::Perm(t) => {
            let token = g.perm(t).token;
            crate::glog!(g, "    Sunpearl Kirin returns {} to its owner's hand", g.perm(t).name);
            if token {
                leave(g, t)?
            } else {
                bounce(g, t)?
            }
            if g.perm(t).on_bf {
                return Ok(()); // it did not go: not returned, so no card
            }
            token
        }
        Back::Token(k) => {
            let pl = g.player_mut(p);
            match k {
                "Treasure" => pl.sac_treasure(),
                "Food" => pl.foods -= 1,
                _ => pl.clues -= 1,
            }
            crate::glog!(g, "    Sunpearl Kirin returns a {k} token to its owner's hand: it ceases to exist");
            true
        }
    };
    if token {
        draw(g, p, 1, false)?;
    }
    Ok(())
}

/// the AI's choice (see `kirin_etb`): the highest score, the replay value of a permanent (at least 2) or what a token's
/// card is worth over the token (1.5 less its worth, when positive and more than half a card)
fn ai_choice(g: &Game, p: PlayerId, src: PermId) -> Option<Back> {
    let mut best: Option<(Back, f64)> = None;
    for m in g.player(p).perms.iter().copied().filter(|&m| m != src && nonland(g, p, m)) {
        let x = g.perm(m);
        if x.is_cmd {
            continue;
        }
        let score = if x.token {
            let v = 1.5 - pval(g, m);
            if v > 0.5 { v } else { continue }
        } else {
            let v = super::mine::flicker_worth(g, p, m);
            if v >= 2.0 { v } else { continue }
        };
        if best.is_none_or(|b| score > b.1) {
            best = Some((Back::Perm(m), score));
        }
    }
    best.map(|b| b.0)
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, KIRIN)?.etb = Some(kirin_etb);
    Ok(())
}
