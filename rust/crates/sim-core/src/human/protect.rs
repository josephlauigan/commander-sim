//! Practice mode: protection spells the AI plays through its own code (`ai::pool`, `zones::cast_teferis_protection`,
//! `mine::veyran_protect_swat`) and that did nothing when a person cast them (found by `tests/practice_counter_probe.rs`,
//! 2026-11-04):
//! - **Not of This World** ({7}: counter target spell or ability that targets a permanent you control; it costs {7}
//!   less if it targets a spell that targets a creature you control with power 7 or greater): cast as a counterspell
//!   at the stack item the person picks.
//! - **Deflecting Swat** ({2}{R}, free while you control a commander: you may choose new targets for target spell or
//!   ability): the person picks the spell and its new target; the spell then resolves at it.
//! - **Teferi's Protection**: resolves with the engine's own effect (`zones::teferis_protection`).

use super::{Target, choices, legal, mana};
use crate::engine::{self, stack};
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;

pub const NOTW: &str = "Not of This World";
pub const SWAT: &str = "Deflecting Swat";

fn cname(g: &Game, c: CardId) -> String {
    g.db.get(c).name.to_string()
}

/// q controls permanent m (on the battlefield)
fn mine(g: &Game, q: PlayerId, m: PermId) -> bool {
    g.player(q).perms.contains(&m) || g.player(q).lands.contains(&m)
}

/// the stack items with a permanent target: (item id, its card, the target)
fn targeted(g: &Game) -> Vec<(u32, CardId, PermId)> {
    g.stack
        .iter()
        .rev()
        .filter(|it| !it.countered)
        .filter_map(|it| Some((it.id, it.card?, it.ctx.target?)))
        .filter(|&(_, _, t)| g.perm(t).on_bf)
        .collect()
}

fn pick_item(g: &Game, q: PlayerId, what: &str, items: &[(u32, CardId, PermId)]) -> Res<Option<usize>> {
    if items.len() == 1 {
        return Ok(Some(0));
    }
    let labels: Vec<String> =
        items.iter().map(|&(_, c, t)| format!("{} (targeting {})", cname(g, c), g.perm(t).name)).collect();
    choices::choose(g, q, "target", &format!("{what}: which spell or ability?"), &labels, Some("cancel"), None)
}

/// Not of This World, cast by q in response: None, or why not
pub fn not_of_this_world(g: &mut Game, q: PlayerId, c: CardId) -> Res<Option<String>> {
    let items: Vec<(u32, CardId, PermId)> = targeted(g).into_iter().filter(|&(_, _, t)| mine(g, q, t)).collect();
    if items.is_empty() {
        return Ok(Some(format!(
            "{NOTW} counters a spell or ability that targets a permanent you control: there is none."
        )));
    }
    let Some(k) = pick_item(g, q, NOTW, &items)? else { return Ok(Some("Cancelled.".into())) };
    let (id, _, t) = items[k];
    // {7} less against a spell (not an ability) aimed at a creature of yours with power 7 or more
    let is_spell = g.stack.iter().any(|x| x.id == id && x.kind == crate::state::StackKind::Spell);
    let cost = if is_spell && g.is_creature(t) && engine::values::epow(g, t) >= 7 { 0 } else { 7 };
    if let Some(why) = mana::cost_problem(g, q, cost, "") {
        return Ok(Some(format!("Can't cast {NOTW}. {why}")));
    }
    mana::pay_from_pool(g, q, cost, "");
    g.free_counter = true; // paid
    let r = stack::cast_counter_spell(g, q, c, id);
    g.free_counter = false;
    if !r? {
        return Ok(Some(format!("{NOTW} couldn't be cast.")));
    }
    Ok(None)
}

/// Deflecting Swat, cast by q in response: None, or why not
pub fn deflecting_swat(g: &mut Game, q: PlayerId, c: CardId) -> Res<Option<String>> {
    let items = targeted(g);
    if items.is_empty() {
        return Ok(Some(format!("{SWAT} changes the target of a spell or ability on the stack: there is none.")));
    }
    let Some(k) = pick_item(g, q, SWAT, &items)? else { return Ok(Some("Cancelled.".into())) };
    let (id, spell, old) = items[k];
    let caster = g.stack.iter().find(|x| x.id == id).map(|x| x.controller).unwrap_or(q);
    // the new target: one the spell could have targeted (as its caster cast it), other than the old one
    let news: Vec<PermId> = legal::spell_targets(g, caster, spell)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|t| if let Target::Perm(m) = t { Some(m) } else { None })
        .filter(|&m| m != old && g.perm(m).on_bf)
        .collect();
    if news.is_empty() {
        return Ok(Some(format!("{} has no other legal target.", cname(g, spell))));
    }
    let free = engine::values::commander_out(g, q);
    let (gn, pips) = if free { (engine::altcost::tax(g, q, c), "") } else { (2, "R") };
    if let Some(why) = mana::cost_problem(g, q, gn, pips) {
        return Ok(Some(format!("Can't cast {SWAT}. {why}")));
    }
    let labels: Vec<String> = news.iter().map(|&m| legal::describe_target(g, q, Target::Perm(m))).collect();
    let prompt = format!("{SWAT}: choose a new target for {}", cname(g, spell));
    let Some(j) = choices::choose(g, q, "target", &prompt, &labels, Some("cancel"), None)? else {
        return Ok(Some("Cancelled.".into()));
    };
    let new = news[j];
    mana::pay_from_pool(g, q, gn, pips);
    let pl = g.player_mut(q);
    let i = pl.hand.iter().position(|&x| x == c).unwrap();
    pl.hand.remove(i);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    engine::cast::on_cast(g, q, c)?;
    crate::glog!(g, "  {} casts {SWAT} at {}", g.player(q).name, cname(g, spell));
    let resolved = stack::counter_window(g, q, c, 4.0, vec![])?;
    g.player_mut(q).gy.push(c);
    if resolved && let Some(it) = g.stack.iter_mut().find(|x| x.id == id && !x.countered) {
        it.ctx.target = Some(new);
        crate::glog!(g, "    {} now targets {}", cname(g, spell), g.perm(new).name);
    }
    Ok(None)
}
