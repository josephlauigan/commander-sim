//! Kyodai, Soul of Kamigawa, the `thanos` deck's commander. Oracle text (data/scryfall_cache.json):
//!
//! - {3}{W} Legendary Creature - Dragon Spirit 3/3. "Flash. Flying. When Kyodai enters, another target permanent gains
//!   indestructible for as long as you control Kyodai. {W}{U}{B}{R}{G}: Kyodai gets +5/+5 until end of turn."
//!
//! Flash, flying and the pump are compiled from the card data (`data/cards.json`). The enter trigger is card code:
//!
//! - **A triggered ability that uses the stack** (a hook not marked at once), so an opponent's Elesh Norn blanks it,
//!   Panharmonicon and the other doublers repeat it (each instance picks its own target and holds its own grant), and
//!   the table may respond. It does not trigger when there is no other permanent Kyodai's controller may target. The
//!   target is chosen as it resolves (the engine's convention for hook triggers): the person is asked, the AI picks.
//! - **The grant** is a static `grant_kw` hook on Kyodai (`partials::tekuthal_kw`'s shape): the targets and the
//!   controller as the trigger resolved sit in Kyodai's data (`DataKey::Kyodai`), and the hook says "indestructible"
//!   for them while Kyodai is on the battlefield under that controller. A blink or a recast is a new object with new
//!   data, so the old grant ends and the new trigger makes its own. If Kyodai left before the trigger resolved, its
//!   duration had already ended and nothing happens (CR 611.2b).
//! - **Targets**: the permanent must be on the battlefield and not phased out, not shrouded (Lightning Greaves too),
//!   not hexproof when it is an opponent's, and not protected from white (Kyodai's colours, so from every colour
//!   under a Leyline of the Guildpact). An opponent's ward is paid as it resolves (`ward::pay_ward`); unpaid, nothing
//!   happens.
//! - **The AI's pick**: one of its own permanents, never an opponent's: a card the plan names (its engines and the
//!   cards of its lines) first, then the permanent whose card the AI wants most, then one of its own lands. A second
//!   instance (a doubled trigger) takes a permanent that has no grant yet when there is one.

use crate::cards::CardDb;
use crate::engine::stack::trigger_window;
use crate::engine::tutors::card_worth;
use crate::engine::values::{colors_of, protected_from, pval};
use crate::flow::Res;
use crate::hooks::{Registry, Src};
use crate::ids::{PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
use crate::sym::Sym;

pub const KYODAI: &str = "Kyodai, Soul of Kamigawa";

/// the other permanents `o` may target with Kyodai's trigger (on the battlefield, not phased out, no shroud, no
/// hexproof on an opponent's, no protection from Kyodai's colours)
fn candidates(g: &Game, src: PermId, o: PlayerId) -> Vec<PermId> {
    let cols = colors_of(g, src);
    let mut out: Vec<PermId> = vec![];
    for q in g.players.iter().filter(|q| q.alive) {
        for &m in q.perms.iter().chain(q.lands.iter()) {
            if m != src
                && !out.contains(&m)
                && super::vondam_elspeth::may_target(g, o, m)
                && !protected_from(g, m, cols)
            {
                out.push(m);
            }
        }
    }
    out
}

/// the permanents Kyodai's grants hold now (the first item of the list is the controller they were made under)
fn granted(g: &Game, src: PermId) -> Vec<PermId> {
    match g.perm(src).data.get(DataKey::Kyodai) {
        Some(Val::List(v)) => v
            .iter()
            .filter_map(|x| match x {
                Val::Perm(m) => Some(*m),
                _ => None,
            })
            .collect(),
        _ => vec![],
    }
}

/// the names of the cards the plan p plays from builds on: its engines and the cards of its lines
fn plan_names(g: &Game, p: PlayerId) -> Vec<&'static str> {
    let Some(l) = crate::ai::plan_data::active(g, p) else { return vec![] };
    let mut v: Vec<&'static str> = l.plan.engines.iter().chain(&l.plan.engines_ahead).map(String::as_str).collect();
    v.extend(l.lines.iter().flat_map(|(gs, _)| gs.iter().flatten()).map(String::as_str));
    v
}

/// the AI's pick among its own permanents (see the module doc)
fn ai_pick(g: &Game, o: PlayerId, src: PermId, cs: &[PermId]) -> Option<PermId> {
    let have = granted(g, src);
    let mine: Vec<PermId> = cs.iter().copied().filter(|&m| g.perm(m).owner == o).collect();
    let fresh: Vec<PermId> = mine.iter().copied().filter(|m| !have.contains(m)).collect();
    let pool = if fresh.is_empty() { &mine } else { &fresh };
    let is_land = |m: PermId| g.player(o).lands.contains(&m);
    let value = |m: PermId| g.perm(m).cd.map_or(pval(g, m), |c| card_worth(g, o, c, false));
    let named = plan_names(g, o);
    let nonland: Vec<PermId> = pool.iter().copied().filter(|&m| !is_land(m)).collect();
    let planned: Vec<PermId> = nonland.iter().copied().filter(|&m| named.contains(&&*g.perm(m).name)).collect();
    let best = |xs: &[PermId]| crate::engine::zones::max_by(xs, value);
    best(&planned).or_else(|| best(&nonland)).or_else(|| best(pool))
}

/// "When Kyodai enters, another target permanent gains indestructible for as long as you control Kyodai."
fn kyodai_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = crate::engine::hooks::trigger_controller(g, src);
    if candidates(g, src, o).is_empty() {
        return Ok(()); // no target: the ability is not put on the stack
    }
    if !trigger_window(g, o, Some(src), "another target permanent gains indestructible", Some(5.0))? {
        return Ok(());
    }
    // the duration "for as long as you control Kyodai" ended before it began: he left (or changed hands) in response
    if !g.perm(src).on_bf || g.perm(src).phased || g.perm(src).owner != o {
        return Ok(());
    }
    let cs = candidates(g, src, o);
    if cs.is_empty() {
        return Ok(()); // every target became illegal
    }
    let to = if crate::human::is_human(g, o) {
        crate::human::gaps_t2::pick_target(g, o, &cs, "Kyodai: which permanent gains indestructible?", None)?
    } else {
        ai_pick(g, o, src, &cs)
    };
    let Some(t) = to else { return Ok(()) };
    if g.perm(t).owner != o && !crate::engine::ward::pay_ward(g, o, t)? {
        return Ok(()); // ward countered the ability
    }
    let mut list = match g.perm(src).data.get(DataKey::Kyodai) {
        Some(Val::List(v)) if v.first() == Some(&Val::Player(o)) => v.clone(),
        _ => vec![Val::Player(o)],
    };
    list.push(Val::Perm(t));
    g.perm_mut(src).data.set(DataKey::Kyodai, Val::List(list));
    crate::glog!(
        g,
        "  Kyodai: {} gains indestructible for as long as {} controls him",
        &*g.perm(t).name,
        &*g.player(o).name
    );
    Ok(())
}

/// the grant: the permanents Kyodai made indestructible, while he is on the battlefield under the player who
/// controlled him as the trigger resolved (`hooked` skips him when he is phased out or has lost his abilities)
fn kyodai_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "indestructible"
        && matches!(g.perm(src).data.get(DataKey::Kyodai),
            Some(Val::List(v)) if v.first() == Some(&Val::Player(g.perm(src).owner)) && v.contains(&Val::Perm(m)))
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, KYODAI)?;
    c.etb = Some(kyodai_etb);
    c.grant_kw = Some(kyodai_kw);
    Ok(())
}
