//! Making an AI play (Python calls the option's closure). Each `Action` is data, so the look-ahead can find the same
//! play again in a copy of the game and make it there.

use super::brain;
use crate::engine::cast::cast_card;
use crate::engine::mana::{can_pay, pay};
use crate::engine::zones::draw;
use crate::flow::Res;
use crate::hooks::{Action, Call, Event, Sacrificed};
use crate::ids::{CardId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::intern;
use crate::tag::Tag;

/// make play `a` for p: true if it did anything
pub fn perform(g: &mut Game, p: PlayerId, a: &Action) -> Res<bool> {
    match a {
        Action::Cast { card, zone } => brain::do_cast(g, p, *card, *zone),
        Action::Removal { card, targets, kick, kind } => brain::cast_removal(g, p, *card, targets, *kick, *kind),
        Action::Wipe { card, victim } => brain::cast_wipe(g, p, *card, *victim),
        Action::Face { card, q, kick } => face(g, p, *card, *q, *kick),
        Action::Crackle { card, x } => crackle(g, p, *card, *x),
        Action::Clue => clue(g, p),
        Action::Plan { f, arg } => f(g, p, *arg),
        Action::Ability { src, f, arg } => f(g, *src, p, *arg),
        Action::Dsl { src, idx } => crate::dsl::activate(g, p, *src, *idx),
        Action::Loyalty { src, idx } => crate::dsl::use_loyalty(g, p, *src, *idx),
        Action::Equip { src, target, n } => crate::dsl::equip(g, p, *src, *target, *n),
    }
}

/// burn at q's face (kicked: 2 more damage)
fn face(g: &mut Game, p: PlayerId, c: CardId, q: crate::ids::PlayerId, kick: u32) -> Res<bool> {
    let d = g.db.get(c);
    let (gn, pips) = (d.generic + kick, d.pips.to_string());
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, gn, &pips, false) {
        return Ok(false);
    }
    let mut ctx = Ctx { face: Some(q), ..Ctx::default() };
    if kick > 0 {
        let r = d.tags.str(Tag::Rem).unwrap_or("dmg0");
        ctx.rem_kind = Some(intern(&format!("dmg{}", r[3..].parse::<i32>().unwrap_or(0) + 2)));
    }
    pay(g, p, gn, &pips, false)?;
    cast_card(g, p, c, "hand", ctx)?;
    Ok(true)
}

/// Crackle with Power for X
fn crackle(g: &mut Game, p: PlayerId, c: CardId, x: u32) -> Res<bool> {
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 3 * x, "RR", false) {
        return Ok(false);
    }
    pay(g, p, 3 * x, "RR", false)?;
    cast_card(g, p, c, "hand", Ctx { x: x as i32, ..Ctx::default() })?;
    Ok(true)
}

/// {2}, sacrifice a Clue: draw a card
fn clue(g: &mut Game, p: PlayerId) -> Res<bool> {
    if g.player(p).clues == 0 || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    g.player_mut(p).clues -= 1;
    draw(g, p, 1, false)?;
    if !g.hooks.is_empty() {
        crate::engine::hooks::fire_trigger(
            g,
            Event::Sacrifice,
            Call::Sacrifice { p, what: Sacrificed::Token("Clue") },
        )?;
    }
    Ok(true)
}
