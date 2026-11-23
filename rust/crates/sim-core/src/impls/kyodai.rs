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
//!   target is chosen as the trigger goes on the stack (the person is asked, the AI picks), before anyone responds; at
//!   resolution it is rechecked (Kyodai still here under the same controller, the target still a legal choice) and never
//!   re-picked.
//! - **The grant** is a resolved duration effect, remembered rather than recomputed: the targets and the controller as
//!   the trigger resolved sit in Kyodai's data (`DataKey::Kyodai`), each target keeps the Kyodai that granted it
//!   (`DataKey::KyodaiOn`), and `chars::has_keyword` asks `grants` for "indestructible". It holds while Kyodai is on the
//!   battlefield, phased in, under that controller; losing his abilities does not end it (CR 611.2b), but phasing out
//!   (cleared when he phases back in, `turn::phase_in_for`, CR 702.26) or changing hands (cleared in
//!   `control::gain_control`) ends it for good. A blink or a recast is a new object with new data, so the old grant
//!   ends and the new trigger makes its own. If Kyodai left before the trigger resolved, nothing happens.
//!   Limits: a direct `.owner =` write (a few cards write it without `gain_control`) does not clear the list, so a
//!   Kyodai taken that way and returned resumes his grants; Imprisoned in the Moon is modelled as leaving the battlefield,
//!   so it ends them.
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

/// "When Kyodai enters, another target permanent gains indestructible for as long as you control Kyodai." The target is
/// chosen (a person asked) as the trigger goes on the stack, before anyone may respond (Heliod's pattern, `t4::heliod_gain`);
/// at resolution the source must still be here under the same controller and the target still a legal choice, else
/// nothing happens (it is never re-picked).
fn kyodai_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = crate::engine::hooks::trigger_controller(g, src);
    let cs = candidates(g, src, o);
    if cs.is_empty() {
        return Ok(()); // no target: the ability is not put on the stack
    }
    const NAME: &str = "another target permanent gains indestructible";
    if g.trig_probe {
        // only whether the trigger goes on the stack: its target is chosen (and a person asked) once, as it does
        return trigger_window(g, o, Some(src), NAME, Some(5.0)).map(|_| ());
    }
    let to = if crate::human::is_human(g, o) {
        crate::human::gaps_t2::pick_target(g, o, &cs, "Kyodai: which permanent gains indestructible?", None)?
    } else {
        ai_pick(g, o, src, &cs)
    };
    let Some(t) = to else { return Ok(()) };
    if !trigger_window(g, o, Some(src), NAME, Some(5.0))? {
        return Ok(()); // countered
    }
    // the duration "for as long as you control Kyodai" ended before it began (he left, phased out or changed hands in
    // response), or the target is gone or no longer legal (hexproof, protection, phased out): nothing happens
    if !g.perm(src).on_bf || g.perm(src).phased || g.perm(src).owner != o || !candidates(g, src, o).contains(&t) {
        return Ok(());
    }
    if g.perm(t).owner != o && !crate::engine::ward::pay_ward(g, o, t)? {
        return Ok(()); // ward countered the ability
    }
    let mut list = match g.perm(src).data.get(DataKey::Kyodai) {
        Some(Val::List(v)) if v.first() == Some(&Val::Player(o)) => v.clone(),
        _ => vec![Val::Player(o)],
    };
    list.push(Val::Perm(t));
    g.perm_mut(src).data.set(DataKey::Kyodai, Val::List(list));
    let mut on = match g.perm(t).data.get(DataKey::KyodaiOn) {
        Some(Val::List(v)) => v.clone(),
        _ => vec![],
    };
    if !on.contains(&Val::Perm(src)) {
        on.push(Val::Perm(src));
    }
    g.perm_mut(t).data.set(DataKey::KyodaiOn, Val::List(on));
    crate::glog!(
        g,
        "  Kyodai: {} gains indestructible for as long as {} controls him",
        &*g.perm(t).name,
        &*g.player(o).name
    );
    Ok(())
}

/// does a Kyodai's grant hold `m` indestructible now (`chars::has_keyword`)? A resolved duration effect: it lasts while
/// that Kyodai is on the battlefield, phased in, under the player who controlled him as the trigger resolved, and his
/// `DataKey::Kyodai` list still names `m`. Losing his abilities does not end it (so it is read here and not through a
/// hook); changing hands (`control::gain_control`) or phasing in again (`turn::phase_in_for`) clears the list for good.
pub fn grants(g: &Game, m: PermId) -> bool {
    let Some(Val::List(on)) = g.perm(m).data.get(DataKey::KyodaiOn) else { return false };
    on.iter().any(|k| {
        let Val::Perm(s) = k else { return false };
        let x = g.perm(*s);
        x.on_bf
            && !x.phased
            && matches!(x.data.get(DataKey::Kyodai),
                Some(Val::List(v)) if v.first() == Some(&Val::Player(x.owner)) && v.contains(&Val::Perm(m)))
    })
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, KYODAI)?;
    c.etb = Some(kyodai_etb);
    Ok(())
}
