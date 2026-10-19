//! "Play it another way" for a person: the alternative costs and the graveyard and land abilities that card code keeps
//! in the `hand_options`, `gy_options` and `land_options` hooks, which only the AI's `brain` read. The person gets the
//! hook's own options as extra ways to use the card, labelled with the hook's label. Each runs the AI's play
//! (`ai::act::perform`), so what it asks of the person is whatever that play asks (the AI's choices inside it), except
//! for the cards with their own person path in `cards.rs` / `abilities.rs`.
//!
//! The AI's affordability test (`can_pay`) already reads the person's mana pool, so the options appear once the person
//! has tapped the mana. Nothing here runs for an AI seat.

use super::{choices, legal};
use crate::flow::Res;
use crate::hooks::{Action, Opt};
use crate::ids::{CardId, LandId, PlayerId};
use crate::state::Game;

pub type Why = Option<String>;

/// is this option just the plain cast of card c (which the person already has)?
fn plain_cast(o: &Opt, c: CardId) -> bool {
    matches!(o.act, Some(Action::Cast { card, .. }) if card == c)
}

/// a label that starts with the card's name
fn named(g: &Game, c: CardId, label: &str) -> String {
    let n = &*g.db.get(c).name;
    let short = n.split(" // ").next().unwrap_or(n);
    if label.contains(short) { label.to_string() } else { format!("{short}: {label}") }
}

fn timing_ok(g: &Game, p: PlayerId, c: CardId) -> bool {
    legal::instant_speed(g, p, c) || legal::sorcery_timing(g, p).is_none()
}

/// The hook options for a card in p's hand, as (label, option).
pub fn hand_alts(g: &mut Game, p: PlayerId, c: CardId) -> Res<Vec<(String, Opt)>> {
    let Some(f) = g.registry.get(c).and_then(|i| i.hand_options) else { return Ok(vec![]) };
    if !timing_ok(g, p, c) || crate::engine::values::silenced(g, p) {
        return Ok(vec![]);
    }
    Ok(f(g, c, p, Some(false))?
        .into_iter()
        .filter(|o| o.act.is_some() && !plain_cast(o, c))
        .map(|o| (named(g, c, &o.label), o))
        .collect())
}

/// The hook options for a card in p's graveyard. `normal`: the person can already cast it the usual way (so its
/// plain cast is left out).
pub fn gy_alts(g: &mut Game, p: PlayerId, c: CardId) -> Res<Vec<(String, Opt)>> {
    let Some(f) = g.registry.get(c).and_then(|i| i.gy_options) else { return Ok(vec![]) };
    if !timing_ok(g, p, c) || crate::engine::values::silenced(g, p) {
        return Ok(vec![]);
    }
    Ok(f(g, c, p, Some(false))?
        .into_iter()
        .filter(|o| o.act.is_some() && !plain_cast(o, c))
        .map(|o| (named(g, c, &o.label), o))
        .collect())
}

/// The hook options of land l.
pub fn land_alts(g: &mut Game, p: PlayerId, l: LandId) -> Res<Vec<(String, Opt)>> {
    if crate::engine::mana::moon_land(g, l) || g.land(l).tapped {
        return Ok(vec![]);
    }
    let Some(f) = g.registry.get(g.land(l).cd).and_then(|i| i.land_options) else { return Ok(vec![]) };
    Ok(f(g, l, p, Some(false))?.into_iter().filter(|o| o.act.is_some()).map(|o| (o.label.clone(), o)).collect())
}

/// Does the card have a hook the person's menus should look at? (a cheap test for the page)
pub fn hand_hook(g: &Game, c: CardId) -> bool {
    g.registry.get(c).is_some_and(|i| i.hand_options.is_some())
}
pub fn gy_hook(g: &Game, c: CardId) -> bool {
    g.registry.get(c).is_some_and(|i| i.gy_options.is_some())
}
pub fn land_hook(g: &Game, l: LandId) -> bool {
    g.registry.get(g.land(l).cd).is_some_and(|i| i.land_options.is_some())
}

/// Run one hook option for the person. Ok(None) when it was done (or the person backed out); else why not.
pub fn run(g: &mut Game, p: PlayerId, c: CardId, o: &Opt) -> Res<Why> {
    let Some(a) = &o.act else { return Ok(None) };
    let did = crate::ai::act::perform(g, p, a)?;
    crate::engine::life::check_state(g)?;
    if !did {
        let n = &*g.db.get(c).name;
        return Ok(Some(format!("Couldn't do that with {n}: check the mana in your pool and what it needs.")));
    }
    Ok(None)
}

/// The person picks one of `ways` (label, option) and it is run; a single way is run at once.
pub fn pick_and_run(g: &mut Game, p: PlayerId, c: CardId, ways: Vec<(String, Opt)>) -> Res<Why> {
    let k = if ways.len() > 1 {
        let labels: Vec<String> = ways.iter().map(|w| w.0.clone()).collect();
        let n = g.db.get(c).name.to_string();
        let Some(k) = choices::choose(g, p, "choose", &format!("{n}: which?"), &labels, Some("cancel"), None)? else {
            return Ok(None);
        };
        k
    } else {
        0
    };
    run(g, p, c, &ways[k].1)
}
