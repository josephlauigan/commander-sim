//! Practice mode (model-gaps B3): a person activates Clue, Food and Blood tokens. The engine keeps these tokens as counts
//! on the player (`clues`, `foods`, `bloods`), not as permanents, so they have no entry in a permanent's ability menu:
//! the action is `{"do": "use", "token": "Clue" | "Food" | "Blood"}` (the page lists them; the text client types
//! `use clue`). The AI's own handling (`ai::act::clue`, `common::eat_food`, `sheoldred::blood_rummage`) is untouched.
//!
//! - Clue: {2}, Sacrifice this artifact: Draw a card.
//! - Food: {2}, {T}, Sacrifice this artifact: You gain 3 life.
//! - Blood: {1}, {T}, Discard a card, Sacrifice this artifact: Draw a card.
//!
//! The cost is paid from the pool (Forensic Gadgeteer takes {1} off an artifact's activation, never below {1}); the
//! sacrifice is part of the cost, so Sacrifice triggers fire before the ability is on the stack; then `ability_run`
//! puts the ability on the stack (the table may respond; Rings of Brighthearth and Peter Parker's Camera may copy it, and
//! a copy pays nothing: it draws or gains again). {T}: a Food the Cabbage Merchant has tapped for mana is not available.

use super::{choices, mana};
use crate::engine::life::gain;
use crate::engine::stack::ability_run;
use crate::engine::zones;
use crate::flow::Res;
use crate::hooks::{Call, Event, Sacrificed};
use crate::ids::PlayerId;
use crate::state::Game;

/// the token kinds a person can activate, in the order the page lists them
pub const KINDS: [&str; 3] = ["Clue", "Food", "Blood"];

/// how many tokens of this kind p has
pub fn count(g: &Game, p: PlayerId, kind: &str) -> u32 {
    let pl = g.player(p);
    match kind {
        "Clue" => pl.clues,
        "Food" => pl.foods,
        "Blood" => pl.bloods,
        _ => 0,
    }
}

/// the Food tokens p could tap now: all of them, less those the Cabbage Merchant tapped for mana
fn foods_untapped(g: &Game, p: PlayerId) -> u32 {
    let tapped = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| g.perm(m).on_bf && &*g.perm(m).name == "The Cabbage Merchant")
        .map(|&m| g.player(p).foods.saturating_sub(crate::impls::t6_green2::foods_untapped(g, m)))
        .max()
        .unwrap_or(0);
    g.player(p).foods.saturating_sub(tapped)
}

/// the generic mana the activation costs now (Forensic Gadgeteer)
fn cost(g: &Game, p: PlayerId, kind: &str) -> u32 {
    let base = if kind == "Blood" { 1 } else { 2 };
    crate::impls::t45_urza_yawgmoth::artifact_cost(g, p, base, 0)
}

/// Why p cannot activate a token of this kind now (None: they can)
pub fn problem(g: &Game, p: PlayerId, kind: &str) -> Option<String> {
    if count(g, p, kind) == 0 {
        return Some(format!("You have no {kind} token."));
    }
    if kind == "Food" && foods_untapped(g, p) == 0 {
        return Some("All your Food tokens are tapped.".into());
    }
    if kind == "Blood" && g.player(p).hand.is_empty() {
        return Some("You have no card in hand to discard to the Blood token.".into());
    }
    mana::cost_problem(g, p, cost(g, p, kind), "").map(|why| format!("Can't activate a {kind} token. {why}"))
}

/// the person activates one of their tokens of `kind`. None, or why not (nothing changes then)
pub fn use_token(g: &mut Game, p: PlayerId, kind: &str) -> Res<Option<String>> {
    if !KINDS.contains(&kind) {
        return Ok(Some(format!("{kind} is not a token you can activate.")));
    }
    if let Some(why) = problem(g, p, kind) {
        return Ok(Some(why));
    }
    mana::pay_from_pool(g, p, cost(g, p, kind), "");
    let me = g.player(p).name.to_string();
    if kind == "Blood" {
        let hand = g.player(p).hand.clone();
        let d = choices::pick_cards(g, p, &hand, 1, "Blood token: discard which card?", "choose")?[0];
        zones::discard_cards(g, p, &[d])?;
        crate::glog!(g, "  {me} discards {} to a Blood token", g.db.get(d).name);
    }
    // sacrificing it is a cost
    {
        let pl = g.player_mut(p);
        match kind {
            "Clue" => pl.clues -= 1,
            "Food" => pl.foods -= 1,
            _ => pl.bloods -= 1,
        }
    }
    crate::glog!(g, "  {me} sacrifices a {kind} token");
    if !g.hooks.is_empty() {
        let what = Sacrificed::Token(match kind {
            "Clue" => "Clue",
            "Food" => "Food",
            _ => "Blood",
        });
        crate::engine::hooks::fire_trigger(g, Event::Sacrifice, Call::Sacrifice { p, what })?;
    }
    let (name, food) = match kind {
        "Food" => ("Food: you gain 3 life", true),
        "Clue" => ("Clue: draw a card", false),
        _ => ("Blood: draw a card", false),
    };
    ability_run(
        g,
        p,
        None,
        None,
        name,
        None,
        None,
        |g| {
            if food { gain(g, p, 3) } else { zones::draw(g, p, 1, false) }
        },
    )?;
    crate::engine::life::check_state(g)?;
    Ok(None)
}
