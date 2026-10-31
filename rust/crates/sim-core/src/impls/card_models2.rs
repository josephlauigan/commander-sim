//! Card models, part 2 (10-31): card code for the pool's Partial and Unmodeled cards that the ability language
//! can't express. Each function says which Oracle sentence it plays; `tests/cards_jd_models2.rs` checks them.

use super::partials::at_once;
use crate::cards::CardDb;
use crate::engine::stack::{trigger_window, trigger_window_card};
use crate::flow::Res;
use crate::hooks::{Event, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

// ------------------------------------------------------------------ Skithiryx, the Blight Dragon (atraxa-infect)
const SKITHIRYX: &str = "Skithiryx, the Blight Dragon";

/// "{B}{B}: Regenerate Skithiryx." (the AI always pays when it can, as for the other regenerators)
fn skithiryx_regen(g: &mut Game, m: PermId) -> Res<bool> {
    super::legends::regen(g, m, 0, "BB")
}

// ------------------------------------------------------------------ Dreamtide Whale (atraxa-infect)
const WHALE: &str = "Dreamtide Whale";

/// "Vanishing 2": it enters with two time counters (`t6_copies::vanishing_upkeep` removes them)
fn whale_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        let x = g.perm_mut(src);
        x.exc.vanishing = 2;
        x.data.set(DataKey::Vanish, Val::Int(2));
    }
    Ok(())
}

/// "Whenever a player casts their second spell each turn, proliferate."
fn whale_cast(g: &mut Game, src: Src, caster: PlayerId, _c: CardId) -> Res {
    if !live(g, src) || crate::engine::cast::casts_this_turn(g, caster) != 2 {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if trigger_window(g, o, Some(src), "proliferate", None)? {
        super::common::proliferate(g, o, 1)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Voidwing Hybrid (atraxa-infect)
const VOIDWING: &str = "Voidwing Hybrid";

/// "When you proliferate, return Voidwing Hybrid from your graveyard to your hand." Called by `common::proliferate`
/// after p proliferated.
pub fn voidwing_return(g: &mut Game, p: PlayerId) -> Res {
    let Some(v) = g.db.id(VOIDWING) else { return Ok(()) };
    while g.player(p).gy.contains(&v) && !g.over {
        if !trigger_window_card(g, p, v, "return it to your hand")? {
            break;
        }
        let gy = &mut g.player_mut(p).gy;
        let Some(i) = gy.iter().position(|&c| c == v) else { break };
        gy.remove(i);
        g.player_mut(p).hand.push(v);
        crate::glog!(g, "    {} returns {VOIDWING} to hand", g.player(p).name);
    }
    Ok(())
}

// ------------------------------------------------------------------ Norn's Decree (atraxa-infect)
const DECREE: &str = "Norn's Decree";

/// "Whenever one or more creatures an opponent controls deal combat damage to you, that opponent gets a poison
/// counter." Once per combat damage step and opponent: the first of that opponent's creatures to connect this turn
/// (a double strike's second step is the same combat).
fn decree_damage(g: &mut Game, src: Src, p: PlayerId, _a: PermId, d: PlayerId, _dmg: i32) -> Res {
    let o = g.perm(src).owner;
    if !live(g, src) || d != o || p == o {
        return Ok(());
    }
    let key = crate::sym::intern(&format!("norns_decree_{}", p.0));
    let st = g.turn_stamp();
    if g.player(o).flag_turn.get(key) == Some(&st) {
        return Ok(());
    }
    g.player_mut(o).flag_turn.insert(key, st);
    if trigger_window(g, o, Some(src), "that opponent gets a poison counter", None)? {
        crate::engine::repl::give_poison(g, o, p, 1, crate::engine::repl::Path::POISON);
        crate::engine::life::check_state(g)?;
    }
    Ok(())
}

/// "Whenever a player attacks, if one or more players being attacked are poisoned, the attacking player draws a card."
fn decree_attack(g: &mut Game, src: Src, p: PlayerId, _atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    if live(g, src) && g.player(d).poison > 0 && trigger_window(g, g.perm(src).owner, Some(src), "draw", None)? {
        crate::engine::zones::draw(g, p, 1, false)?;
    }
    Ok(vec![])
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, SKITHIRYX)?.self_regen = Some(skithiryx_regen);
    let c = r.card(db, WHALE)?;
    c.etb = Some(whale_etb);
    at_once(c, Event::Etb, true);
    c.cast = Some(whale_cast);
    at_once(c, Event::Cast, false);
    let c = r.card(db, DECREE)?;
    c.combat_damage = Some(decree_damage);
    at_once(c, Event::CombatDamage, false);
    c.attack = Some(decree_attack);
    at_once(c, Event::Attack, false);
    Ok(())
}
