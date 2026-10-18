//! Tier 6 batch `legends2`: Sisay's legends and interaction (Magda, The Queen of Dale, Tataru Taru, Teferi, Who Slows
//! the Sunset, Spider-Woman, Ertai Resurrected, Legolas's Quick Reflexes, Colossal Skyturtle, Wan Shi Tong). Cards only:
//! no engine file is touched; what needs the engine is listed in rust/agents/engine-needs-legends2.md.

use super::partials::{at_once, first_max, on};
use crate::cards::{CardDb, Types};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::card_name;
use crate::engine::zones::{Enter, add_treasure, agent_for, agent_take, enter, searchable};
use crate::flow::Res;
use crate::hooks::{Action, Call, Event, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn ability(utility: f64, label: &str, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label: label.into(), act: Some(Action::Ability { src, f, arg }) }
}

fn remove_card(v: &mut Vec<CardId>, c: CardId) {
    if let Some(i) = v.iter().position(|&x| x == c) {
        v.remove(i);
    }
}

// ================================================================== Magda, Brazen Outlaw
const MAGDA: &str = "Magda, Brazen Outlaw";

/// a Dwarf creature permanent (a changeling counts)
fn is_dwarf(g: &Game, m: PermId) -> bool {
    g.is_creature(m)
        && g.perm(m).cd.is_some_and(|c| {
            let d = g.db.get(c);
            d.has_subtype("dwarf") || d.has_kw("changeling")
        })
}

/// the Magdas p controls (not phased out), other than `except`
fn magdas(g: &Game, p: PlayerId, except: Option<PermId>) -> usize {
    g.player(p).perms.iter().filter(|&&y| Some(y) != except && live(g, y) && card_name(g, y) == Some(MAGDA)).count()
}

/// other Dwarves you control get +1/+0 (one per Magda), read from `creature_pt` while `g.selfpt` is on
fn magda_pt(g: &Game, m: PermId) -> (i32, i32) {
    if !is_dwarf(g, m) {
        return (0, 0);
    }
    (magdas(g, owner(g, m), Some(m)) as i32, 0)
}

fn magda_as(g: &mut Game, _p: PlayerId, _m: PermId) -> Res {
    g.selfpt = true;
    Ok(())
}

/// whenever a Dwarf you control becomes tapped for mana (the mana-tapped event): a Treasure
fn magda_mana(g: &mut Game, src: Src, p: PlayerId, m: PermId, _n: u32) -> Res {
    if live(g, src) && owner(g, src) == p && owner(g, m) == p && is_dwarf(g, m) {
        add_treasure(g, p, 1)?;
        crate::glog!(g, "    {} makes a Treasure ({} tapped)", g.perm(src).name, g.perm(m).name);
    }
    Ok(())
}

/// whenever a Dwarf you control becomes tapped by attacking: a Treasure each (one trigger window for all of them)
fn magda_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if !live(g, src) || owner(g, src) != p {
        return Ok(vec![]);
    }
    let k = atk.iter().filter(|&&m| owner(g, m) == p && g.perm(m).tapped && is_dwarf(g, m)).count();
    if k > 0 && trigger_window(g, p, Some(src), &format!("create {k} Treasure"), None)? {
        add_treasure(g, p, k as i32)?;
    }
    Ok(vec![])
}

/// the artifact or Dragon card Magda fetches: the best by worth
fn magda_pick(g: &Game, p: PlayerId) -> Option<CardId> {
    let ok: Vec<CardId> = searchable(g, p)
        .into_iter()
        .filter(|&c| {
            let d = g.db.get(c);
            d.types.has(Types::ARTIFACT) || d.has_subtype("dragon")
        })
        .collect();
    first_max(&ok, |c| card_worth(g, p, c, false))
}

/// Sacrifice five Treasures: search for an artifact or Dragon card, put it onto the battlefield, shuffle
fn magda_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !live(g, src) || owner(g, src) != p || g.player(p).treasures < 5 {
        return Ok(vec![]);
    }
    let Some(t) = magda_pick(g, p) else { return Ok(vec![]) };
    let u = 8.0f64.min(2.0 + 0.5 * g.db.get(t).cmc as f64);
    Ok(vec![ability(u, &format!("Magda: {}", g.db.get(t).name), src, magda_go, t.0 as i64)])
}

fn magda_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = CardId(arg as u16);
    if !on(g, p, src) || g.player(p).treasures < 5 || !g.player(p).library.contains(&t) {
        return Ok(false);
    }
    for _ in 0..5 {
        g.player_mut(p).treasures -= 1;
        if !g.hooks.is_empty() {
            crate::engine::hooks::fire_trigger(
                g,
                Event::Sacrifice,
                Call::Sacrifice { p, what: crate::hooks::Sacrificed::Token("Treasure") },
            )?;
        }
    }
    crate::glog!(g, "  {} sacrifices five Treasures: Magda searches for {}", g.player(p).name, g.db.get(t).name);
    if !ability_window(g, p, Some(src), &format!("search for {}", g.db.get(t).name), None, None)?
        || !g.player(p).library.contains(&t)
    {
        return Ok(true);
    }
    remove_card(&mut g.player_mut(p).library, t);
    shuffle_library(g, p);
    match agent_for(g, p) {
        Some(a) => agent_take(g, a, p, t),
        None => {
            enter(g, p, t, Enter::default())?;
        }
    }
    Ok(true)
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.creature_pt.push(magda_pt);
    let c = r.card(db, MAGDA)?;
    c.as_enters = Some(magda_as);
    c.mana_tapped = Some(magda_mana);
    c.attack = Some(magda_attack);
    at_once(c, Event::Attack, false);
    c.options = Some(magda_options);
    Ok(())
}
