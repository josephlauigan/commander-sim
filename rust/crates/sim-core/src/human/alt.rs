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
use crate::hooks::{Action, Opt, PlanFn};
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
    if super::specs::spec(&g.db.get(c).name).is_some() {
        return Ok(vec![]); // its own cast asks the person (specs.rs); the hook is only the AI's plan for that cast
    }
    if !timing_ok(g, p, c) || crate::engine::values::silenced(g, p) {
        return Ok(vec![]);
    }
    let mut out: Vec<(String, Opt)> = f(g, c, p, Some(false))?
        .into_iter()
        .filter(|o| o.act.is_some() && !plain_cast(o, c))
        .map(|o| (named(g, c, &o.label), o))
        .collect();
    if out.is_empty() {
        out.extend(forced_hand(g, c));
    }
    Ok(out)
}

/// The hook options for a card in p's graveyard. `normal`: the person can already cast it the usual way (so its
/// plain cast is left out).
pub fn gy_alts(g: &mut Game, p: PlayerId, c: CardId) -> Res<Vec<(String, Opt)>> {
    let Some(f) = g.registry.get(c).and_then(|i| i.gy_options) else { return Ok(vec![]) };
    if !timing_ok(g, p, c) || crate::engine::values::silenced(g, p) {
        return Ok(vec![]);
    }
    let mut out: Vec<(String, Opt)> = f(g, c, p, Some(false))?
        .into_iter()
        .filter(|o| o.act.is_some() && !plain_cast(o, c))
        .map(|o| (named(g, c, &o.label), o))
        .collect();
    if out.is_empty() {
        out.extend(forced_gy(g, c));
    }
    Ok(out)
}

/// The hook options of land l.
pub fn land_alts(g: &mut Game, p: PlayerId, l: LandId) -> Res<Vec<(String, Opt)>> {
    if crate::engine::mana::moon_land(g, l) || g.land(l).tapped {
        return Ok(vec![]);
    }
    let Some(f) = g.registry.get(g.land(l).cd).and_then(|i| i.land_options) else { return Ok(vec![]) };
    let mut out: Vec<(String, Opt)> =
        f(g, l, p, Some(false))?.into_iter().filter(|o| o.act.is_some()).map(|o| (o.label.clone(), o)).collect();
    if out.is_empty() {
        out.extend(forced_land(g, l));
    }
    Ok(out)
}

/// Does the card have a hook the person's menus should look at? (a cheap test for the page)
pub fn hand_hook(g: &Game, c: CardId) -> bool {
    super::specs::spec(&g.db.get(c).name).is_none() && g.registry.get(c).is_some_and(|i| i.hand_options.is_some())
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

// ------------------------------------------------------------------ plays the AI's hooks hold back
/// Some hooks only offer a play when the AI's own judgement likes it (a target worth it, life to spare ...). The
/// person decides for themselves, so for these cards the play's own code (`*_go`, which checks that it can be paid and
/// done) is offered whenever the hook offers nothing.
fn forced(label: &str, f: PlanFn, arg: i64) -> (String, Opt) {
    (label.to_string(), Opt { utility: 0.0, label: label.to_string(), act: Some(Action::Plan { f, arg }) })
}

fn forced_hand(g: &Game, c: CardId) -> Option<(String, Opt)> {
    use crate::impls::{bulk_white, kefka, sheoldred};
    let n = c.0 as i64;
    Some(match &*g.db.get(c).name {
        "Exalted Angel" | "Forum Familiar" | "Unyielding Gatekeeper" | "Exit Specialist" => {
            forced("cast it face down for {3} (morph / disguise)", bulk_white::cast_down, n)
        }
        "Susurian Voidborn" => {
            forced("warp {B}: cast it for {B}; exiled at the end of the turn", kefka::voidborn_warp, n)
        }
        "Perigee Beckoner" => {
            forced("warp {1}{B}: cast it for {1}{B}; exiled at the end of the turn", sheoldred::warp_go, n)
        }
        _ => return None,
    })
}

fn forced_gy(g: &Game, c: CardId) -> Option<(String, Opt)> {
    use crate::impls::{kefka, t3};
    let n = c.0 as i64;
    Some(match &*g.db.get(c).name {
        "Demonic Embrace" => {
            forced("cast it from your graveyard (pay 3 life, discard a card)", super::cards::embrace_person, n)
        }
        "Scarlet Spider, Kaine" => {
            forced("mayhem {B}: cast it from your graveyard (discarded this turn)", kefka::mayhem_go, n)
        }
        "Reassembling Skeleton" => forced("{1}{B}: return it to the battlefield tapped", t3::skeleton_go, n),
        _ => return None,
    })
}

fn forced_land(g: &Game, l: LandId) -> Option<(String, Opt)> {
    use crate::impls::{lands, legends, sheoldred, t6_lands};
    let n = l.0 as i64;
    Some(match &*g.db.get(g.land(l).cd).name {
        "Command Beacon" => forced("{T}, sacrifice: put your commander into your hand", legends::beacon_go, n),
        "Karn's Bastion" => forced("{4}, {T}: proliferate", lands::bastion_go, n),
        "Geier Reach Sanitarium" => {
            forced("{2}, {T}: each player draws a card, then discards a card", sheoldred::geier_go, n)
        }
        "Mikokoro, Center of the Sea" => forced("{2}, {T}: each player draws a card", sheoldred::mikokoro_go, n),
        "Abstergo Entertainment" => forced(
            "{3}, {T}, exile it: return a historic card from your graveyard to your hand, exile all graveyards",
            t6_lands::abstergo_go,
            n,
        ),
        _ => return None,
    })
}
