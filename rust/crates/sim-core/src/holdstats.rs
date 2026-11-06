//! --- t5-hold: what the mana an AI player leaves up at the end of its turn does before its next turn. Trace-only counters
//! in the player's `stats` map (`hd|...`), made while the game is played: nothing here reads the random generator or
//! changes a choice (the decision functions it calls only read the game), and no decision reads the counters, so every
//! game is played as before (`tests/t5_hold.rs` replays a seeded game with them off). Look-ahead copies and people's
//! seats keep none.
//!
//! At the end of a player's second main phase (`snapshot`) the turn is put in one group:
//! - `ctr`, `prot`, `rem`, `adn`, `oth`: `brain::hold_value` names a card to keep mana up for, by what it is (a counterspell,
//!   protection, instant removal, Ad Nauseam, anything else);
//! - `idle`: nothing is held, 2 or more mana is unused and a card other than a held one can be cast.
//!
//! The group is followed to the holder's next turn start (`turn_start`) and ends as one outcome: `used` (a card of the
//! same kind hit something: a counterspell countered a spell, removal removed a permanent, protection saved one),
//! `castnh` (one was cast, nothing was hit), `else` (mana was spent on something else) or `unused` (none was spent).
//! A game that ends first leaves the group without an outcome.
//!
//! Key format: `hd|<b>|<name>` and `hd|<b>|<group>|<name>`; `<b>` is `all` (every own turn) or `mid` (own turns 5-10).
//! - `hd|b|turns`: snapshots taken; `u2any`: 2+ mana unused and any castable nonland card in hand; `u2c|held`, `u2c|idle`:
//!   2+ unused and a castable card that is not a held kind, with or without a card held
//! - `hd|b|g|n`, `mana` (unused mana, summed), `hv` (the hold's value times 100, summed)
//! - `hd|b|g|used`, `castnh`, `else`, `unused`: the outcomes; `spent` (mana spent in the window), `cast` (cards of the
//!   same kind cast), `oinst` (other instants or flash spells cast)
//! - `hd|b|g|hit|<class>`: what the same-kind card hit: `combo` (a combo piece, or anything cast during the owner's
//!   combo attempt), `cmdr` (a commander), `creature`, `engine` (another permanent), `removal`, `spell` (any other
//!   instant or sorcery), `ability`

use crate::ai::brain;
use crate::engine::mana::{can_pay, cost_of};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Game, StackKind};
use crate::sym::intern;
use crate::tag::Tag;

/// the snapshot being followed
#[derive(Debug, Clone, PartialEq)]
pub struct Watch {
    grp: &'static str,
    mid: bool,
    spent: u32,
    cast: u32,
    hit: u32,
    oinst: u32,
}

fn on(g: &Game, p: PlayerId) -> bool {
    g.settings.hold_trace && !g.in_search && !crate::human::is_human(g, p)
}

fn add(g: &mut Game, p: PlayerId, mid: bool, name: &str, n: i64) {
    if n == 0 {
        return;
    }
    for b in ["all", "mid"].into_iter().take(1 + mid as usize) {
        g.player_mut(p).stat(intern(&format!("hd|{b}|{name}")), n);
    }
}

/// what a card is, for the group it is held under and the cards that count as the same kind
fn kind_of(g: &Game, c: CardId) -> &'static str {
    let d = g.db.get(c);
    if d.tag(Tag::Ctr) {
        "ctr"
    } else if d.tag(Tag::Adnaus) {
        "adn"
    } else if d.tags.str(Tag::Prot).is_some() {
        "prot"
    } else if d.tags.str(Tag::Rem).is_some() && (d.instant || d.tag(Tag::Flash)) {
        "rem"
    } else {
        "oth"
    }
}

fn can_pay_now(g: &Game, p: PlayerId, c: CardId) -> bool {
    let (cg, cp) = cost_of(g, p, c);
    can_pay(g, p, cg, &cp, false)
}

fn fast(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    d.instant || d.tag(Tag::Flash)
}

/// what a spell (or ability) of `owner` is, to say what an answer hit
fn class_of_card(g: &Game, owner: PlayerId, c: CardId) -> &'static str {
    let d = g.db.get(c);
    if crate::cardcode::is_combo_piece(g, c) || g.player(owner).combo_tried {
        "combo"
    } else if g.player(owner).is_commander(c) {
        "cmdr"
    } else if d.tags.str(Tag::Rem).is_some() || d.tag(Tag::Wipe) {
        "removal"
    } else if d.creature {
        "creature"
    } else if d.instant || d.sorcery {
        "spell"
    } else {
        "engine"
    }
}

fn class_of_perm(g: &Game, m: PermId) -> &'static str {
    let x = g.perm(m);
    match x.cd {
        Some(c) if crate::cardcode::is_combo_piece(g, c) => "combo",
        _ if x.is_cmd => "cmdr",
        _ if g.is_creature(m) => "creature",
        _ => "engine",
    }
}

/// the end of p's second main phase: put the turn in a group and start following it
pub fn snapshot(g: &mut Game, p: PlayerId) {
    if !on(g, p) {
        return;
    }
    let t = g.player(p).turns;
    let mid = (5..=10).contains(&t);
    let s = brain::Situation::new(g, p);
    let (hc, hv) = brain::hold_value(g, p, &s);
    let hand: Vec<CardId> = g.player(p).hand.clone();
    let castable = |c: CardId, strict: bool| {
        let d = g.db.get(c);
        !d.land
            && can_pay_now(g, p, c)
            && (!strict
                || (Some(c) != hc
                    && !(d.tag(Tag::Ctr) && !d.tag(Tag::Free))
                    && d.tags.str(Tag::Prot).is_none()
                    && !(d.tags.str(Tag::Rem).is_some() && fast(g, c))))
    };
    let any = hand.iter().any(|&c| castable(c, false));
    let other = hand.iter().any(|&c| castable(c, true));
    add(g, p, mid, "turns", 1);
    if s.mana >= 2 && any {
        add(g, p, mid, "u2any", 1);
    }
    if s.mana >= 2 && other {
        add(g, p, mid, if hc.is_some() { "u2c|held" } else { "u2c|idle" }, 1);
    }
    let grp = match hc {
        Some(c) => kind_of(g, c),
        None if s.mana >= 2 && other => "idle",
        None => {
            g.player_mut(p).hw = None;
            return;
        }
    };
    add(g, p, mid, &format!("{grp}|n"), 1);
    add(g, p, mid, &format!("{grp}|mana"), s.mana as i64);
    add(g, p, mid, &format!("{grp}|hv"), (hv * 100.0).round() as i64);
    g.player_mut(p).hw = Some(Watch { grp, mid, spent: 0, cast: 0, hit: 0, oinst: 0 });
}

/// the start of p's own turn: the outcome of the snapshot of the turn before
pub fn turn_start(g: &mut Game, p: PlayerId) {
    let Some(w) = g.player_mut(p).hw.take() else { return };
    let out = if w.hit > 0 {
        "used"
    } else if w.cast > 0 {
        "castnh"
    } else if w.spent > 0 {
        "else"
    } else {
        "unused"
    };
    let gp = w.grp;
    add(g, p, w.mid, &format!("{gp}|{out}"), 1);
    add(g, p, w.mid, &format!("{gp}|spent"), w.spent as i64);
    add(g, p, w.mid, &format!("{gp}|cast"), w.cast as i64);
    add(g, p, w.mid, &format!("{gp}|oinst"), w.oinst as i64);
}

/// p cast c
pub fn cast(g: &mut Game, p: PlayerId, c: CardId) {
    if g.player(p).hw.is_none() || g.in_search {
        return;
    }
    let (grp, inst) = (g.player(p).hw.as_ref().unwrap().grp, fast(g, c));
    let same = grp != "idle" && if grp == "oth" { inst && kind_of(g, c) == "oth" } else { kind_of(g, c) == grp };
    if let Some(w) = g.player_mut(p).hw.as_mut() {
        if same {
            w.cast += 1;
        } else if inst {
            w.oinst += 1;
        }
    }
}

/// p spent n mana
pub fn spent(g: &mut Game, p: PlayerId, n: u32) {
    if let Some(w) = g.player_mut(p).hw.as_mut() {
        w.spent += n;
    }
}

fn hit(g: &mut Game, p: PlayerId, class: &str) {
    let Some(w) = g.player(p).hw.as_ref().filter(|_| !g.in_search) else { return };
    let (gp, mid) = (w.grp, w.mid);
    g.player_mut(p).hw.as_mut().unwrap().hit += 1;
    add(g, p, mid, &format!("{gp}|hit|{class}"), 1);
}

/// q's counterspell countered the topmost spell or ability of p still on the stack
pub fn counter_hit(g: &mut Game, q: PlayerId, p: PlayerId) {
    if !g.player(q).hw.as_ref().is_some_and(|w| w.grp == "ctr") {
        return;
    }
    let class = match g.stack.iter().rev().find(|x| x.controller == p && !x.countered && x.kind != StackKind::Trigger) {
        Some(it) => it.card.map_or("ability", |c| class_of_card(g, p, c)),
        None => "ability",
    };
    hit(g, q, class);
}

/// actor removed m with a spell
pub fn removal_hit(g: &mut Game, actor: PlayerId, m: PermId, spell: Option<CardId>) {
    if spell.is_some() && g.player(actor).hw.as_ref().is_some_and(|w| w.grp == "rem" && w.cast > 0) {
        let class = class_of_perm(g, m);
        hit(g, actor, class);
    }
}

/// owner saved m from removal
pub fn protected(g: &mut Game, owner: PlayerId, m: PermId) {
    if g.player(owner).hw.as_ref().is_some_and(|w| w.grp == "prot") {
        let class = class_of_perm(g, m);
        hit(g, owner, class);
    }
}
