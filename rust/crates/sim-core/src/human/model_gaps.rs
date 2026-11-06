//! Practice mode (2026-11-05, model-gaps B1): activated abilities of the owned decks' cards that a person could not use
//! at all, found when the trigger probe was given the preconditions it lacked (`tests/practice_trigger_probe.rs`).
//! Maelstrom of the Spirit Dragon's search has no AI code, so it is a person-only entry: the AI's paths are unchanged.
//!
//! (Loran of the Third Path's person path, also written in B1, is `marvin_hooks::loran`, which model-gaps A4 added for the
//! owner and for Marvin; the B1 copy was removed when the branches were merged.) `land_abilities` is appended by
//! `abilities::land_abilities`.

use super::abilities::{LandAbility, Why, ability};
use super::{choices, mana};
use crate::engine::stack::ability_window_card;
use crate::engine::{self, zones};
use crate::flow::Res;
use crate::ids::{PermId, PlayerId};
use crate::state::Game;

/// the abilities of p's land l that this file lists
pub fn land_abilities(g: &Game, l: PermId) -> Vec<LandAbility> {
    match &*g.db.get(g.perm(l).card()).name {
        "Maelstrom of the Spirit Dragon" if !engine::mana::moon_land(g, l) => vec![ability(
            "{4}, {T}, sacrifice it: search your library for a Dragon card, reveal it, put it into your hand",
            maelstrom,
        )],
        _ => vec![],
    }
}

// ------------------------------------------------------------------ Maelstrom of the Spirit Dragon
/// {4}, {T}, sacrifice this land: search your library for a Dragon card, reveal it, put it into your hand, then shuffle
fn maelstrom(g: &mut Game, p: PlayerId, l: PermId) -> Res<Why> {
    if g.perm(l).tapped {
        return Ok(Some("Maelstrom of the Spirit Dragon is tapped.".into()));
    }
    if engine::mana::moon_land(g, l) {
        return Ok(Some("Maelstrom of the Spirit Dragon has lost its abilities.".into()));
    }
    if let Some(why) = mana::cost_problem(g, p, 4, "") {
        return Ok(Some(format!("Can't activate Maelstrom of the Spirit Dragon. {why}")));
    }
    mana::pay_from_pool(g, p, 4, "");
    g.perm_mut(l).tapped = true;
    let cd = g.perm(l).card();
    crate::glog!(g, "  {} sacrifices Maelstrom of the Spirit Dragon", g.player(p).name);
    crate::impls::lands::sac_land(g, p, l)?; // the sacrifice is a cost: it leaves before the ability can be answered
    if !ability_window_card(g, p, cd, "search for a Dragon card", None, None)? {
        return Ok(None);
    }
    let found = choices::search(
        g,
        p,
        &|g: &Game, c| {
            let d = g.db.get(c);
            d.has_subtype("dragon") || d.has_kw("changeling")
        },
        1,
        "Maelstrom of the Spirit Dragon: search for a Dragon card",
        true,
    )?;
    let Some(&c) = found.first() else {
        crate::glog!(g, "    {} finds nothing", g.player(p).name);
        return Ok(None);
    };
    if let Some(a) = zones::agent_for(g, p) {
        zones::agent_take(g, a, p, c); // Opposition Agent
        return Ok(None);
    }
    let pl = g.player_mut(p);
    pl.hand.push(c);
    pl.seen.insert(c);
    crate::glog!(g, "    {} reveals {} and puts it into their hand", g.player(p).name, g.db.get(c).name);
    Ok(None)
}
