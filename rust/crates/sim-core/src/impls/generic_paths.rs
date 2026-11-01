//! --- generic-paths (2026-11-01): plays that only a deck's hand-written AI offered, for any deck holding the card.
//! The hand AI of eleven decks was deleted on 2026-11-01 (`audit/phase4/hand-off-design.md`, "Deleted"); these
//! cards' only play path went with it. The audit and what was done per card: `audit/phase4/generic-paths-2026-11-01.md`.
//!
//! - **Jace's Archivist** ({U}, {T}: each player discards their hand, then draws as many as the most anyone
//!   discarded): the wheel when it pays: a punisher of the opponents' draws or casts is out (Orcish Bowmasters,
//!   Sheoldred, the Apocalypse, Kaervek the Merciless), or p's hand is nearly empty against a full one. It was
//!   Sauron's hand AI (`decks::sauron_options`).
//! - **Yawgmoth's Will**: cast when the graveyard pays for it: a storm card (Brain Freeze, Grapeshot) with the mana
//!   to cast it after the Will, or three cheap spells that fit in the mana left. It was Kefka's hand AI
//!   (`kefka::will_options`, already written for "a deck that isn't built around it").
//! - **Susurian Voidborn**: cast from exile on a later turn after its warp (the warp itself is `kefka::voidborn_hand`).
//!   It was Kefka's hand AI (`kefka::kefka_options`); `brain::hook_options` calls `exile_options`.

use crate::cards::CardDb;
use crate::engine::cast::cast_card;
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::stack::ability_window;
use crate::engine::values::has;
use crate::engine::zones::{discard_cards, draw};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::intern;
use crate::tag::Tag;

const ARCHIVIST: &str = "Jace's Archivist";
const WILL: &str = "Yawgmoth's Will";
const VOIDBORN: &str = "Susurian Voidborn";

fn human(g: &Game, p: PlayerId) -> bool {
    crate::human::is_human(g, p)
}

// ------------------------------------------------------------------ Jace's Archivist
/// is the wheel worth it for p now? An opponents' draw or cast punisher out, or p's hand nearly empty while some
/// hand at the table holds four or more
pub fn archivist_worth(g: &Game, p: PlayerId) -> bool {
    let most = g.players.iter().filter(|q| q.alive).map(|q| q.hand.len()).max().unwrap_or(0);
    let mine = g.player(p).hand.len();
    // Bowmasters pings each extra draw; Sheoldred drains each; refilled opponents cast into Kaervek, and into Sauron
    // (an Orc for each spell)
    has(g, p, Tag::Bowmasters)
        || has(g, p, Tag::SheoA)
        || has(g, p, Tag::Kaervek)
        || (has(g, p, Tag::Sauron) && mine <= 2 && most >= 4)
        || (mine <= 1 && most >= 4)
}

/// the Archivist's {U}, {T}, on p's turn in a main phase
fn archivist_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if x.owner != p || x.tapped || x.sick || x.phased || post.is_none() || g.active != Some(p) || human(g, p) {
        return Ok(vec![]);
    }
    if crate::engine::values::stopped(g, ARCHIVIST)
        || g.player(p).arch_t == Some(g.player(p).turns)
        || !archivist_worth(g, p)
        || !can_pay(g, p, 0, "U", false)
    {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 5.0,
        label: "Jace's Archivist wheel".into(),
        act: Some(Action::Ability { src, f: archivist_go, arg: 0 }),
    }])
}

fn archivist_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let x = g.perm(src);
    if x.owner != p || !x.on_bf || x.tapped || x.sick || !archivist_worth(g, p) || !can_pay(g, p, 0, "U", false) {
        return Ok(false);
    }
    pay(g, p, 0, "U", false)?;
    g.perm_mut(src).tapped = true;
    let t = g.player(p).turns;
    g.player_mut(p).arch_t = Some(t);
    crate::glog!(g, "  {} activates Jace's Archivist", g.player(p).name);
    if !ability_window(g, p, Some(src), "each player discards their hand, then draws", None, None)? {
        return Ok(true);
    }
    let n = g.players.iter().filter(|q| q.alive).map(|q| q.hand.len()).max().unwrap_or(0) as u32;
    let alive: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
    for &q in &alive {
        let hand = g.player(q).hand.clone();
        discard_cards(g, q, &hand)?;
    }
    for &q in &alive {
        if g.player(q).alive && !g.over {
            draw(g, q, n, false)?;
        }
    }
    Ok(true)
}

// ------------------------------------------------------------------ Yawgmoth's Will
/// is p's graveyard worth replaying now? The option's utility: a storm card (Brain Freeze, Grapeshot) with the mana
/// to cast it after the Will, or three cheap castable spells that fit in the mana left; never into an empty yard
pub fn will_value(g: &Game, p: PlayerId) -> Option<f64> {
    let pl = g.player(p);
    if pl.yawg || g.active != Some(p) || !can_pay(g, p, 2, "B", false) {
        return None;
    }
    let left = total_mana(g, p, false).saturating_sub(3);
    let cost = |c: CardId| {
        let (cg, cp) = cost_of(g, p, c);
        cg + cp.chars().count() as u32
    };
    let rituals = pl
        .gy
        .iter()
        .filter(|&&c| matches!(&*g.db.get(c).name, "Dark Ritual" | "Cabal Ritual" | "Lotus Petal" | "Jeska's Will"))
        .count() as u32;
    let storm = pl.gy.iter().any(|&c| matches!(&*g.db.get(c).name, "Brain Freeze" | "Grapeshot"));
    if storm && left + 2 * rituals >= 2 {
        return Some(7.0);
    }
    let mut costs: Vec<u32> = pl
        .gy
        .iter()
        .filter(|&&c| {
            let d = g.db.get(c);
            !d.land && !d.tag(Tag::Rean) && !d.tag(Tag::Fill) && !d.tag(Tag::Yawg)
        })
        .map(|&c| cost(c))
        .collect();
    costs.sort_unstable();
    (costs.len() >= 3 && costs.iter().take(3).sum::<u32>() <= left).then_some(5.0)
}

fn will_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !g.player(p).hand.contains(&c) || human(g, p) {
        return Ok(vec![]);
    }
    Ok(will_value(g, p)
        .map(|u| Opt {
            utility: u,
            label: "Yawgmoth's Will (replay the graveyard)".into(),
            act: Some(Action::Plan { f: will_go, arg: c.0 as i64 }),
        })
        .into_iter()
        .collect())
}

fn will_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let w = CardId(arg as u16);
    if !g.player(p).hand.contains(&w) || will_value(g, p).is_none() {
        return Ok(false);
    }
    pay(g, p, 2, "B", false)?;
    cast_card(g, p, w, "hand", Ctx::default())?;
    Ok(true)
}

// ------------------------------------------------------------------ Susurian Voidborn
/// the warped Voidborn in p's exile (exiled by its warp at an end step: `kefka::voidborn_end`), castable for its
/// mana cost on a later turn, on p's turn in a main phase
pub fn exile_options(g: &mut Game, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || human(g, p) {
        return Ok(vec![]);
    }
    let st = g.turn_stamp();
    let Some(&when) = g.player(p).flag_turn.get(&intern("voidborn_warped")) else { return Ok(vec![]) };
    if when == st {
        return Ok(vec![]);
    }
    let Some(c) = g.player(p).exile.iter().copied().find(|&c| &*g.db.get(c).name == VOIDBORN) else {
        return Ok(vec![]);
    };
    let (cg, cp) = cost_of(g, p, c);
    if !can_pay(g, p, cg, &cp, false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 4.5,
        label: "Susurian Voidborn (from exile)".into(),
        act: Some(Action::Plan { f: voidborn_from_exile, arg: c.0 as i64 }),
    }])
}

fn voidborn_from_exile(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let (cg, cp) = cost_of(g, p, c);
    if !g.player(p).exile.contains(&c) || !can_pay(g, p, cg, &cp, false) {
        return Ok(false);
    }
    pay(g, p, cg, &cp, false)?;
    g.player_mut(p).exile.take(c);
    g.player_mut(p).flag_turn.shift_remove(&intern("voidborn_warped"));
    cast_card(g, p, c, "lib", Ctx::default())?;
    Ok(true)
}

// ================================================================== registry
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, ARCHIVIST)?.options = Some(archivist_options);
    r.card(db, WILL)?.hand_options = Some(will_options);
    Ok(())
}
