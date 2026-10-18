//! Tier 6 batch `green2`: the green and small utility cards of the Rograkh & Thrasios and Kinnan decks
//! (rust/agents/t6-batches.md). Cards only: no engine file is touched; what needs the engine is listed in
//! rust/agents/engine-needs-green2.md.
//!
//! - Chatterstorm: the Squirrel token is the compiled spell; storm is `t0_stella::storm_copies` on the cast.
//! - Cloud of Faeries: ETB untaps two lands; cycling {2} is a hand option.
//! - Noxious Revival: resolve puts the best of the caster's own graveyard cards on top of its library.

use crate::cards::CardDb;
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::trigger_window;
use crate::engine::tutors::card_worth;
use crate::engine::zones::{discard_cards, draw, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::{Sym, intern};

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn remove_card(v: &mut Vec<CardId>, c: CardId) {
    if let Some(i) = v.iter().position(|&x| x == c) {
        v.remove(i);
    }
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

// ================================================================== Chatterstorm
/// Storm: when you cast it, copy it for each spell cast before it this turn
fn chatterstorm_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    crate::impls::t0_stella::storm_copies(g, p, c)
}

// ================================================================== Cloud of Faeries
/// When this creature enters, untap up to two lands: the AI untaps two tapped lands of its own (it always takes the
/// full two when it has them).
fn cloud_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m != src || !trigger_window(g, o, Some(src), "untap up to two lands", Some(3.0))? {
        return Ok(());
    }
    let tapped: Vec<_> = g.player(o).lands.iter().copied().filter(|&l| g.land(l).tapped).take(2).collect();
    for l in &tapped {
        g.land_mut(*l).tapped = false;
    }
    crate::glog!(g, "    {} untaps {} land(s) (Cloud of Faeries)", pname(g, o), tapped.len());
    Ok(())
}

/// Cycling {2}: the AI cycles it in an opponent's end step or after combat when it has the two mana to spare.
fn cloud_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !g.player(p).hand.contains(&c) || !(post.is_none() && g.active != Some(p) || post == Some(true)) {
        return Ok(vec![]);
    }
    if !can_pay(g, p, 2, "", false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 0.9,
        label: "cycle Cloud of Faeries".into(),
        act: Some(Action::Plan { f: cloud_cycle, arg: c.0 as i64 }),
    }])
}

fn cloud_cycle(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    crate::glog!(g, "  {} cycles Cloud of Faeries", pname(g, p));
    discard_cards(g, p, &[c])?;
    draw(g, p, 1, false)?;
    Ok(true)
}

// ================================================================== Noxious Revival
/// the nonland card in one of p's own graveyard worth putting back on top: the best by `card_worth`, a creature
/// counting a little more and mana value breaking ties (Oboro, Eternal Witness); None if there is none
pub fn revival_pick(g: &Game, p: PlayerId) -> Option<CardId> {
    let gy: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| !g.db.get(c).land).collect();
    max_by(&gy, |c| {
        let d = g.db.get(c);
        card_worth(g, p, c, true) + if d.creature { 5.0 } else { 0.0 } + 0.1 * d.cmc as f64
    })
}

/// {G/P}, instant: put target card from a graveyard on top of its owner's library. The AI aims it at its own best
/// card (the engine's Phyrexian mana lets it pay 2 life); with nothing worth it, it fizzles to the graveyard.
fn revival_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = revival_pick(g, p) {
        let pl = g.player_mut(p);
        remove_card(&mut pl.gy, t);
        pl.library.push(t);
        crate::glog!(g, "    {} puts {} on top of its library (Noxious Revival)", pname(g, p), cname(g, t));
    }
    Ok(intern("gy"))
}

/// cast only to set up a draw: when the graveyard holds a card worth drawing
fn revival_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if revival_pick(g, p).is_some() { 30 } else { 0 }
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Chatterstorm")?.self_cast = Some(chatterstorm_cast);
    let c = r.card(db, "Cloud of Faeries")?;
    c.etb = Some(cloud_etb);
    c.hand_options = Some(cloud_options);
    let c = r.card(db, "Noxious Revival")?;
    c.resolve = Some(revival_resolve);
    c.prio = Some(revival_prio);
    Ok(())
}
