//! Ghave, Guru of Spores (JD's deck `ghave`, decklists/JD/ghave-abzan-aristocrats.md; Seph's t3-t4 seat since
//! 2026-10-30). Ghave's two activated abilities are compiled from the Oracle text (the ability language: the
//! "remove a +1/+1 counter from a creature you control" cost is read by `dsl::pay_ability_cost`, and a paid
//! "sacrifice a creature" outlet answers removal and wipes through `common::sac_in_response` / `sac_before_wipe` for
//! a deck with the plan knob `paid_outlets`). This file has the rest:
//!
//! - "Ghave enters with five +1/+1 counters on it" (`enters_with`, read by `zones::enter`: a placement by an effect,
//!   so Doubling Season makes it ten, on any entry: cast, reanimated or blinked);
//! - the deck's loops as combo-catalogue entries (`COMBOS`), each run as its result after the catalogue's answer
//!   windows, as every catalogue combo is:
//!   1-4. Ghave + Ashnod's Altar or Phyrexian Altar + Cathars' Crusade or Doubling Season: unbounded Saprolings,
//!        enters and deaths (and colourless mana with Ashnod's Altar); a death payoff wins (Blood Artist, Zulaport
//!        Cutthroat, Cruel Celebrant, Sephiroth, Fabled SOLDIER), and with Ashnod's Altar's mana so does Triskelion
//!        (Ghave puts the counters on it, it pings them off);
//!   5.   Mikaeus, the Unhallowed + Triskelion (kills alone);
//!   6.   Mikaeus + Kitchen Finks + a free outlet, with a death payoff;
//!   7.   Nim Deathmantle + Ashnod's Altar + Grave Titan, with a death payoff or Triskelion.
//!   8.   Kitchen Finks + Cathars' Crusade + a free outlet (persist's -1/-1 counter meets the Crusade's +1/+1), with a
//!        death payoff or Altar of Dementia (Commander Spellbook 2034-2744--5 and kin).
//!   Lines 5-7 are Sephiroth's old loops (impls/mine.rs `LOOPS`), read through `mine::loops_now` for any deck.
//!
//! Only the ghave list plays these pieces together (no pool deck holds Ghave, Triskelion, Kitchen Finks or Nim
//! Deathmantle), so the catalogue entries change no pool deck's games.

use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::combos::{Combo, Ready, loop_payoff};
use crate::state::Game;

pub const GHAVE: &str = "Ghave, Guru of Spores";
const ALTARS: &[&str] = &["Ashnod's Altar", "Phyrexian Altar"];
const ASHNOD: &[&str] = &["Ashnod's Altar"];
/// a Saproling's enter puts the counter back (Crusade), or one activation makes two (Doubling Season)
const DOUBLERS: &[&str] = &["Cathars' Crusade", "Doubling Season"];
const SEPHIROTH: &str = "Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel";
/// a death payoff that turns the unbounded deaths into a kill
const PAYOFFS: &[&str] = &["Blood Artist", "Zulaport Cutthroat", "Cruel Celebrant", SEPHIROTH];
const TRISKELION: &[&str] = &["Triskelion"];
const MIKAEUS: &[&str] = &["Mikaeus, the Unhallowed"];
const FINKS: &[&str] = &["Kitchen Finks"];
const FREE_OUTLETS: &[&str] = &["Viscera Seer", "Ashnod's Altar", "Altar of Dementia"];
const NIM: &[&str] = &["Nim Deathmantle"];
const TITAN: &[&str] = &["Grave Titan"];
/// the death payoffs, or Altar of Dementia milling the table with the unbounded sacrifices
const FINKS_PAYOFFS: &[&str] =
    &["Blood Artist", "Zulaport Cutthroat", "Cruel Celebrant", SEPHIROTH, "Altar of Dementia"];

const NOT_READY: fn() -> Ready = || (false, vec![], vec![]);

/// the +1/+1 counters card cd enters with whatever put it onto the battlefield (`zones::enter` adds them; the enter
/// step's replacements apply)
pub fn enters_with(g: &Game, cd: CardId) -> i32 {
    if &*g.db.get(cd).name == GHAVE { 5 } else { 0 }
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        x.on_bf
            && !x.phased
            && !x.neutered
            && x.cd.is_some_and(|c| &*g.db.get(c).name == name)
            && !crate::engine::values::stopped(g, &g.db.get(x.card()).name)
    })
}

fn any_bf(g: &Game, p: PlayerId, names: &[&str]) -> Option<PermId> {
    names.iter().find_map(|n| on_bf(g, p, n))
}

/// a death payoff out (the catalogue's drain payoffs, or Sephiroth, Fabled SOLDIER's drain)
fn payoff(g: &Game, p: PlayerId) -> Option<PermId> {
    loop_payoff(g, p).or_else(|| on_bf(g, p, SEPHIROTH))
}

/// Ghave on the battlefield with his abilities working, and a +1/+1 counter on one of p's creatures to start from
fn ghave_ready(g: &Game, p: PlayerId) -> Option<PermId> {
    let gh = on_bf(g, p, GHAVE)?;
    if crate::cardcode::ability_locked(g, gh, p) || super::mine::loops_blocked(g, p) {
        return None;
    }
    let counter = g.player(p).perms.iter().any(|&m| g.is_creature(m) && !g.perm(m).phased && g.perm(m).plus > 0);
    counter.then_some(gh)
}

/// lines 1-4 with a death payoff
fn ghave_drain(g: &Game, p: PlayerId) -> Ready {
    let (Some(gh), Some(altar), Some(dbl), Some(pay)) =
        (ghave_ready(g, p), any_bf(g, p, ALTARS), any_bf(g, p, DOUBLERS), payoff(g, p))
    else {
        return NOT_READY();
    };
    (true, vec![gh, altar, dbl, pay], vec![])
}

/// lines 1 and 3 with Triskelion: the mana pays Ghave's {1} to put each Saproling's death on Triskelion as a counter
fn ghave_trisk(g: &Game, p: PlayerId) -> Ready {
    let (Some(gh), Some(altar), Some(dbl), Some(t)) =
        (ghave_ready(g, p), any_bf(g, p, ASHNOD), any_bf(g, p, DOUBLERS), any_bf(g, p, TRISKELION))
    else {
        return NOT_READY();
    };
    (true, vec![gh, altar, dbl, t], vec![])
}

/// one of Sephiroth's loops (impls/mine.rs `LOOPS[i]`) that kills: alone, or with a death payoff out
fn mine_loop(g: &Game, p: PlayerId, i: usize) -> Ready {
    let Some((_, keys, kills)) = super::mine::loops_now(g, p).into_iter().find(|x| x.0 == i) else {
        return NOT_READY();
    };
    if !(kills || payoff(g, p).is_some()) {
        return NOT_READY();
    }
    (true, keys, vec![])
}

fn mikaeus_trisk(g: &Game, p: PlayerId) -> Ready {
    mine_loop(g, p, 0)
}

fn mikaeus_finks(g: &Game, p: PlayerId) -> Ready {
    mine_loop(g, p, 1)
}

fn nim_titan(g: &Game, p: PlayerId) -> Ready {
    mine_loop(g, p, 3)
}

/// line 8: Kitchen Finks + Cathars' Crusade + a free outlet: Finks returns by persist with a -1/-1 counter, the
/// Crusade's +1/+1 counter cancels it, so persist works every time; a death payoff wins (or Altar of Dementia mills)
fn finks_crusade(g: &Game, p: PlayerId) -> Ready {
    if super::mine::loops_blocked(g, p) {
        return NOT_READY();
    }
    let (Some(f), Some(c), Some(o)) =
        (any_bf(g, p, FINKS), on_bf(g, p, "Cathars' Crusade"), any_bf(g, p, FREE_OUTLETS))
    else {
        return NOT_READY();
    };
    let Some(k) = payoff(g, p).or_else(|| on_bf(g, p, "Altar of Dementia")) else { return NOT_READY() };
    (true, vec![f, c, o, k], vec![])
}

pub static COMBOS: [Combo; 6] = [
    Combo {
        name: "Ghave + an altar + Cathars' Crusade / Doubling Season + a death payoff",
        groups: &[&[GHAVE], ALTARS, DOUBLERS, PAYOFFS],
        ready: ghave_drain,
        mana: (1, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Ghave + Ashnod's Altar + Cathars' Crusade / Doubling Season + Triskelion",
        groups: &[&[GHAVE], ASHNOD, DOUBLERS, TRISKELION],
        ready: ghave_trisk,
        mana: (1, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Mikaeus + Triskelion",
        groups: &[MIKAEUS, TRISKELION],
        ready: mikaeus_trisk,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Mikaeus + Kitchen Finks + a free outlet + a death payoff",
        groups: &[MIKAEUS, FINKS, FREE_OUTLETS, PAYOFFS],
        ready: mikaeus_finks,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Nim Deathmantle + Ashnod's Altar + Grave Titan + a death payoff",
        groups: &[NIM, ASHNOD, TITAN, PAYOFFS],
        ready: nim_titan,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
    Combo {
        name: "Kitchen Finks + Cathars' Crusade + a free outlet + a death payoff",
        groups: &[FINKS, &["Cathars' Crusade"], FREE_OUTLETS, FINKS_PAYOFFS],
        ready: finks_crusade,
        mana: (0, ""),
        finish: None,
        xcost: &[],
        steps: &[],
    },
];
