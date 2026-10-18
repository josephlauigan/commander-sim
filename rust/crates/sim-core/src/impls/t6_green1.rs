//! Tier 6, batch `green1`: the creature-engine cards of the Rograkh / Thrasios deck (Oboro Breezecaller, Badgermole
//! Cub, Biomancer's Familiar, Earthcraft, Training Grounds, Candelabra of Tawnos, Magus of the Candelabra,
//! Formidable Speaker, Springheart Nantuko, Vibrance). Cards only: no shared engine code is changed here (what the
//! engine still has to do is in rust/agents/engine-needs-green1.md).
//!
//! Most of them untap lands for the deck's mana engine (Gaea's Cradle taps for one {G} per creature), so the AI uses
//! an untap effect only when the land it untaps makes clearly more mana than the effect costs.

use super::partials::{pack, unpack};
use crate::cards::CardDb;
use crate::engine::hooks as eh;
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::ability_window;
use crate::engine::turn::remove_land;
use crate::engine::zones::min_by;
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{LandId, PermId, PlayerId};
use crate::state::Game;
use crate::tag::Tag;

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased && !g.perm(m).neutered
}

fn name_of(g: &Game, m: PermId) -> &str {
    g.perm(m).cd.map_or("", |c| &g.db.get(c).name)
}

fn ability(utility: f64, label: &str, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label: label.into(), act: Some(Action::Ability { src, f, arg }) }
}

/// the mana land l makes when tapped now (Gaea's Cradle counts creatures; Talon Gates of Madara makes {C})
fn land_yield(g: &Game, p: PlayerId, l: LandId) -> u32 {
    let base = crate::cardcode::dyn_mana_land(g, p, l)
        .map_or_else(|| g.db.get(g.land(l).cd).tags.int(Tag::Amt).unwrap_or(1).max(0) as u32, |n| n);
    (base as i32 + eh::total_land_mana(g, p, l)).max(0) as u32
}

fn land_on(g: &Game, p: PlayerId, l: LandId) -> bool {
    g.land(l).on_bf && g.player(p).lands.contains(&l)
}

/// {2} less per Training Grounds / Biomancer's Familiar p controls (each: "activated abilities of creatures you
/// control cost {2} less to activate; can't reduce the mana in that cost to less than one mana")
pub fn ability_discount(g: &Game, p: PlayerId) -> u32 {
    let n = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| live(g, m) && matches!(name_of(g, m), "Training Grounds" | "Biomancer's Familiar"))
        .count();
    2 * n as u32
}

/// what a creature's activated ability of printed generic cost `cost` costs p now (never below {1}, unless it was
/// free); the discount applies to creature abilities only
pub fn creature_ability_cost(g: &Game, p: PlayerId, cost: u32) -> u32 {
    if cost == 0 { 0 } else { cost.saturating_sub(ability_discount(g, p)).max(1) }
}

// ================================================================== Oboro Breezecaller
/// the best tapped land to untap and the land to return for it (cheapest to lose: a tapped one costs nothing, an
/// untapped one costs the mana it would make); None when the untap does not pay for its cost
fn oboro_plan(g: &Game, p: PlayerId, cost: u32) -> Option<(LandId, LandId)> {
    let lands = g.player(p).lands.clone();
    let tapped: Vec<LandId> = lands.iter().copied().filter(|&l| g.land(l).tapped).collect();
    let t = crate::engine::zones::max_by(&tapped, |l| land_yield(g, p, l) as f64)?;
    let others: Vec<LandId> = lands.iter().copied().filter(|&l| l != t).collect();
    let b = min_by(&others, |l| if g.land(l).tapped { 0.0 } else { land_yield(g, p, l) as f64 })?;
    let lost = if g.land(b).tapped { 0 } else { land_yield(g, p, b) };
    // the gain: the untapped land's mana, less the ability's cost and the mana the returned land would have made
    (land_yield(g, p, t) as i32 - cost as i32 - lost as i32 >= 2).then_some((t, b))
}

/// whether p has anything to spend extra mana on: a spell in hand or Thrasios's mana sink on the battlefield
fn has_sink(g: &Game, p: PlayerId) -> bool {
    g.player(p).hand.iter().any(|&c| !g.db.get(c).land)
        || g.player(p).perms.iter().any(|&m| live(g, m) && name_of(g, m).starts_with("Thrasios"))
}

/// **Oboro Breezecaller** ({1}{U}, 1/1 flying): {2}, return a land you control to its owner's hand: untap target
/// land. The cost is reduced by Training Grounds / Biomancer's Familiar (`creature_ability_cost`). The AI uses it on
/// its tapped land that makes the most mana (Gaea's Cradle) when that pays back the cost, the returned land and at
/// least {2} more, and something is waiting for the mana; it returns a tapped land if it has one. Not offered while
/// the Breezecaller is phased out, neutered or locked.
fn oboro_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !live(g, src) || owner(g, src) != p || !has_sink(g, p) {
        return Ok(vec![]);
    }
    let cost = creature_ability_cost(g, p, 2);
    if !can_pay(g, p, cost, "", false) {
        return Ok(vec![]);
    }
    let Some((t, b)) = oboro_plan(g, p, cost) else { return Ok(vec![]) };
    Ok(vec![ability(2.5, "untap a land (Oboro Breezecaller)", src, oboro_go, pack(t.0, b.0))])
}

fn oboro_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (t, b) = unpack(arg);
    let (t, b) = (LandId(t), LandId(b));
    let cost = creature_ability_cost(g, p, 2);
    if !live(g, src) || owner(g, src) != p || !land_on(g, p, t) || !land_on(g, p, b) || t == b {
        return Ok(false);
    }
    if !can_pay(g, p, cost, "", false) {
        return Ok(false);
    }
    pay(g, p, cost, "", false)?;
    // the cost: return a land you control to its owner's hand
    if !land_on(g, p, b) {
        return Ok(true);
    }
    let cd = g.land(b).cd;
    remove_land(g, p, b);
    g.player_mut(p).hand.push(cd);
    if !ability_window(g, p, Some(src), "untap target land", None, None)? {
        return Ok(true);
    }
    if land_on(g, p, t) {
        g.land_mut(t).tapped = false;
        crate::glog!(
            g,
            "    {} returns {} and untaps {} (Oboro Breezecaller)",
            g.player(p).name,
            g.db.get(cd).name,
            g.db.get(g.land(t).cd).name
        );
    }
    Ok(true)
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Oboro Breezecaller")?.options = Some(oboro_options);
    Ok(())
}
