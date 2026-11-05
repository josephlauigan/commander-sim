//! Practice mode (2026-11-05, model-gaps B1): activated abilities of the owned decks' cards that a person could not use
//! at all, found when the trigger probe was given the preconditions it lacked (`tests/practice_trigger_probe.rs`).
//! Neither is used by the AI (Maelstrom of the Spirit Dragon's search has no AI code; Loran's draw is an option hook
//! with the AI's own opponent choice), so both are person-only entries: the AI's paths are unchanged.
//!
//! `abilities` is tried by `cards::more_abilities` before `gaps_c`; `land_abilities` is appended by
//! `abilities::land_abilities`.

use super::abilities::{LandAbility, PermAbility, Why, ability};
use super::{choices, gaps_t2, mana};
use crate::engine::stack::{ability_window, ability_window_card};
use crate::engine::{self, zones};
use crate::flow::Res;
use crate::ids::{PermId, PlayerId};
use crate::state::Game;

/// the abilities of p's permanent m (card name `nm`), or None when this file doesn't list the card
pub fn abilities(nm: &str) -> Option<Vec<PermAbility>> {
    Some(match nm {
        "Loran of the Third Path" => vec![ability("{T}: you and target opponent each draw a card", loran)],
        _ => return None,
    })
}

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

// ------------------------------------------------------------------ Loran of the Third Path
/// {T}: you and target opponent each draw a card (the opponent is the person's choice)
fn loran(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    let x = g.perm(m);
    if x.tapped {
        return Ok(Some("Loran of the Third Path is tapped.".into()));
    }
    if x.sick {
        return Ok(Some(
            "Loran of the Third Path has summoning sickness (it came under your control this turn).".into(),
        ));
    }
    let Some(q) = gaps_t2::pick_opponent(g, p, "Loran of the Third Path")? else {
        return Ok(Some("There is no opponent to target.".into()));
    };
    g.perm_mut(m).tapped = true;
    let qn = g.player(q).name.to_string();
    crate::glog!(
        g,
        "  {} activates Loran of the Third Path: {} and {} each draw a card",
        g.player(p).name,
        g.player(p).name,
        qn
    );
    if ability_window(g, p, Some(m), &format!("you and {qn} each draw a card"), None, None)? {
        zones::draw(g, p, 1, false)?;
        if g.player(q).alive {
            zones::draw(g, q, 1, false)?;
        }
    }
    Ok(None)
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
