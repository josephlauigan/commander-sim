//! practice-sheoldred: the Mikaeus, the Unhallowed loops of a person's Sheoldred deck, by hand and as table
//! shortcuts, and Niv-Mizzet's standing ping target (Kefka).
//!
//! - Marked damage for a person's damage (the engine kills a creature only when one hit reaches its toughness, so
//!   Triskelion's pings at itself never killed it): kept in `DataKey::Marked` as [turn stamp, damage].
//! - Yawgmoth, Thran Physician's sacrifice ability, with the person choosing the fodder and the target.
//! - "Start the loop" for Mikaeus + Triskelion (unlimited damage, the person picks what it hits) and for Mikaeus +
//!   Yawgmoth + two undying creatures (a drain payoff turns each trip into life loss; every trip draws a card, so the
//!   library is the limit). Both play the loop out trip by trip with the engine's own death, undying, drain and draw
//!   code, after one window for the opponents to answer a key piece.
//! - Niv-Mizzet, the Firemind / Parun: "send every ping to X this turn" so a draw loop is not one prompt per card.
//!
//! Inert for the AI: everything here runs only for a seat played by a person.

use super::abilities::{PermAbility, Why, ability};
use super::cards::{creatures, pick_perm};
use super::{Target, choices, legal};
use crate::engine::values::etgh;
use crate::engine::{life, stack, zones};
use crate::flow::Res;
use crate::ids::{PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
use crate::tag::Tag;

fn pname(g: &Game, p: PlayerId) -> String {
    g.player(p).name.to_string()
}

fn card_name(g: &Game, m: PermId) -> String {
    match g.perm(m).cd {
        Some(c) => g.db.get(c).name.to_string(),
        None => g.perm(m).name.to_string(),
    }
}

fn on_bf(g: &Game, p: PlayerId, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased && g.player(p).perms.contains(&m)
}

fn named(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        x.on_bf && !x.phased && !x.neutered && x.cd.is_some_and(|c| &*g.db.get(c).name == name)
    })
}

// ------------------------------------------------------------------ marked damage
/// the damage a person has dealt to creature m this turn
pub fn marked(g: &Game, m: PermId) -> i32 {
    match g.perm(m).data.get(DataKey::Marked) {
        Some(Val::List(v)) if v.len() == 2 && v[0] == Val::Stamp(g.turn_stamp()) => v[1].int() as i32,
        _ => 0,
    }
}

/// mark n more damage on creature m (it wears off with the turn: the stamp no longer matches)
pub fn mark(g: &mut Game, m: PermId, n: i32) {
    let total = marked(g, m) + n;
    let st = g.turn_stamp();
    g.perm_mut(m).data.set(DataKey::Marked, Val::List(vec![Val::Stamp(st), Val::Int(total as i64)]));
}

/// n more damage from a person would kill creature m (it already has damage marked on it)
pub fn lethal_with(g: &Game, m: PermId, n: i32) -> bool {
    etgh(g, m) <= n + marked(g, m)
}

/// creature m dies if its toughness has dropped to its marked damage (or to 0), as the state-based actions would
pub fn die_if_lethal(g: &mut Game, m: PermId) -> Res {
    if !g.perm(m).on_bf || !g.is_creature(m) {
        return Ok(());
    }
    let t = etgh(g, m);
    if t <= 0 {
        zones::die(g, m, "sba")?;
    } else if marked(g, m) >= t {
        crate::glog!(g, "    {} has lethal damage marked on it and dies", g.perm(m).name);
        zones::die(g, m, "destroy")?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Yawgmoth, Thran Physician
/// Yawgmoth's abilities for the person: pay 1 life, sacrifice another creature: a -1/-1 counter on up to one target
/// creature (any, your own too: it takes off the +1/+1 counter undying left, so the creature can die and return
/// again) and draw a card. Instant speed. (Protection from Humans and the proliferate ability are not modelled.)
pub fn yawgmoth() -> Vec<PermAbility> {
    vec![ability(
        "pay 1 life, sacrifice another creature: a -1/-1 counter on up to one target creature, draw a card",
        yawg_sacrifice,
    )]
}

fn yawg_sacrifice(g: &mut Game, p: PlayerId, m: PermId) -> Res<Why> {
    if g.player(p).life < 1 {
        return Ok(Some("You can't pay 1 life.".into()));
    }
    let fodder: Vec<PermId> =
        creatures(g, p, false).into_iter().filter(|&x| x != m && g.player(p).perms.contains(&x)).collect();
    if fodder.is_empty() {
        return Ok(Some("You have no other creature to sacrifice.".into()));
    }
    let Some(x) = pick_perm(g, p, "choose", "Yawgmoth: sacrifice which other creature?", &fodder, Some("cancel"))?
    else {
        return Ok(None);
    };
    let tg: Vec<PermId> = creatures(g, p, false).into_iter().filter(|&t| t != x).collect();
    let target = if tg.is_empty() {
        None
    } else {
        pick_perm(
            g,
            p,
            "target",
            &format!("Yawgmoth: put a -1/-1 counter on which creature? ({} is sacrificed)", g.perm(x).name),
            &tg,
            Some("no target (just draw a card)"),
        )?
    };
    crate::glog!(g, "  {} pays 1 life and sacrifices {} to Yawgmoth", pname(g, p), g.perm(x).name);
    life::lose_life(g, p, 1, Some(p), "other", None)?;
    zones::die(g, x, "sac")?; // (an undying trigger resolves first: the creature is back before the counter lands)
    if g.over {
        return Ok(None);
    }
    let what = if target.is_some() { "a -1/-1 counter, draw a card" } else { "draw a card" };
    stack::ability_run_w(g, p, Some(m), None, what, None, target, 0.0, |g, is_copy| {
        let mut target = target;
        if is_copy && target.is_some() {
            // "you may choose a new target for the copy"
            let now: Vec<PermId> = tg.iter().copied().filter(|&t| g.perm(t).on_bf && g.is_creature(t)).collect();
            let prompt = "The copy of Yawgmoth's ability: put a -1/-1 counter on which creature?";
            if !now.is_empty()
                && let Some(t) = pick_perm(g, p, "target", prompt, &now, Some("keep the target"))?
            {
                target = Some(t);
            }
        }
        if let Some(t) = target.filter(|&t| g.perm(t).on_bf && g.is_creature(t)) {
            if zones::minus_counter(g, t, 1) {
                crate::glog!(g, "    a -1/-1 counter on {}", g.perm(t).name);
            }
            if g.perm(t).owner != p {
                crate::impls::bites::crime(g, p)?; // an opposing target (Forsaken Miner)
            }
            die_if_lethal(g, t)?;
        }
        zones::draw(g, p, 1, false)
    })?;
    life::check_state(g)?;
    Ok(None)
}

// ------------------------------------------------------------------ the loops as table shortcuts
/// "start the loop" abilities for Yawgmoth / Mikaeus when Mikaeus + Yawgmoth + two undying creatures are out
pub fn yawg_loop_abilities(g: &Game, p: PlayerId, m: PermId) -> Vec<PermAbility> {
    let nm = card_name(g, m);
    if nm != YAWG && nm != MIKAEUS {
        return vec![];
    }
    let Some(yawg) = named(g, p, YAWG) else { return vec![] };
    if named(g, p, MIKAEUS).is_none() || crate::impls::mine::loops_blocked(g, p) || fodder_pool(g, p, yawg).len() < 2 {
        return vec![];
    }
    let label = format!(
        "start the loop: Mikaeus + Yawgmoth + two undying creatures{}",
        if crate::impls::combos::loop_payoff(g, p).is_some() { "" } else { " (no payoff on the battlefield yet)" }
    );
    vec![ability(label, |g: &mut Game, p: PlayerId, _m: PermId| yawg_loop(g, p))]
}

const YAWG: &str = "Yawgmoth, Thran Physician";
const MIKAEUS: &str = "Mikaeus, the Unhallowed";
const TRISK: &str = "Triskelion";

/// creatures that come back from a sacrifice: undying from Mikaeus (non-Humans) or their own
fn fodder_pool(g: &Game, p: PlayerId, yawg: PermId) -> Vec<PermId> {
    let mik = named(g, p, MIKAEUS).is_some();
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            let Some(cd) = x.cd else { return false };
            let d = g.db.get(cd);
            m != yawg
                && x.on_bf
                && !x.phased
                && !x.token
                && x.orig == p
                && d.creature
                && !g.player(p).is_commander(cd)
                && !d.tag(Tag::Mikaeus)
                && ((mik && !d.tag(Tag::Human)) || d.has_kw("undying"))
        })
        .collect()
}

/// the trip count the person wants
fn ask_trips(g: &Game, p: PlayerId) -> Res<Option<usize>> {
    let labels =
        vec![format!("as many times as it takes to kill every opponent"), "five times".to_string(), "once".to_string()];
    Ok(choices::choose(g, p, "choose", "Start the loop: how many times around?", &labels, Some("cancel"), None)?
        .map(|k| [usize::MAX, 5, 1][k]))
}

/// Mikaeus + Yawgmoth + two undying creatures: sacrifice one (it returns with a +1/+1 counter) while the -1/-1 counter
/// takes the other's off, again and again. Each trip costs 1 life, kills a creature (drain payoffs, Sephiroth) and
/// draws a card, so the loop ends when the table is dead, or when the library is down to 1 card or life to 1.
fn yawg_loop(g: &mut Game, p: PlayerId) -> Res<Why> {
    let Some(yawg) = named(g, p, YAWG) else { return Ok(Some("Yawgmoth is not on the battlefield.".into())) };
    let pool = fodder_pool(g, p, yawg);
    if named(g, p, MIKAEUS).is_none() || pool.len() < 2 {
        return Ok(Some("The loop needs Mikaeus, Yawgmoth and two creatures that come back from a sacrifice.".into()));
    }
    let (mut a, mut b) = (pool[0], pool[1]);
    if pool.len() > 2 {
        let Some(first) =
            pick_perm(g, p, "choose", "The loop: the first of two creatures to use?", &pool, Some("cancel"))?
        else {
            return Ok(None);
        };
        let rest: Vec<PermId> = pool.iter().copied().filter(|&m| m != first).collect();
        let Some(second) = pick_perm(g, p, "choose", "The loop: the second creature?", &rest, Some("cancel"))? else {
            return Ok(None);
        };
        (a, b) = (first, second);
    }
    if g.perm(a).plus > 0 && g.perm(b).plus > 0 {
        return Ok(Some(
            "Both creatures have a +1/+1 counter: undying would not bring either back. Sacrifice one by hand first \
             (a third creature, or a -1/-1 counter on it)."
                .into(),
        ));
    }
    let Some(trips) = ask_trips(g, p)? else { return Ok(None) };
    crate::glog!(g, "  {} goes for the Yawgmoth loop ({} and {})", pname(g, p), g.perm(a).name, g.perm(b).name);
    if crate::ai::combo_interrupted(g, p, "seph", &[yawg, a, b])? {
        crate::glog!(g, "    ...the loop is stopped");
        return Ok(None);
    }
    let (start_lib, start_life) = (g.player(p).library.len(), g.player(p).life);
    let mut done = 0;
    let mut why = String::new();
    while done < trips && !g.over {
        if !on_bf(g, p, yawg) || named(g, p, MIKAEUS).is_none() || !on_bf(g, p, a) || !on_bf(g, p, b) {
            why = "a piece of the loop left the battlefield".into();
            break;
        }
        if g.opps(p).all(|q| !g.player(q).alive) {
            break;
        }
        // x is sacrificed (it has no +1/+1 counter), y gives up its counter
        let (x, y) = if g.perm(a).plus <= 0 {
            (a, b)
        } else if g.perm(b).plus <= 0 {
            (b, a)
        } else {
            (a, b)
        };
        if g.perm(x).plus > 0 {
            why = "both creatures have a +1/+1 counter".into();
            break;
        }
        if g.player(p).life <= 1 {
            why = "your life is down to 1".into();
            break;
        }
        if g.player(p).library.len() <= 1 {
            why = "your library is down to 1 card".into();
            break;
        }
        let (cd, y_counter) = (g.perm(x).cd, g.perm(y).plus > 0);
        life::lose_life(g, p, 1, Some(p), "other", None)?;
        zones::die(g, x, "sac")?; // undying brings it back with a +1/+1 counter, before the counter below lands
        let back = g.player(p).perms.iter().rev().copied().find(|&m| {
            let z = g.perm(m);
            z.on_bf && z.cd == cd && z.plus > 0 && m != y
        });
        let Some(nx) = back else {
            why = format!("{} did not come back", g.perm(x).name);
            break;
        };
        if y_counter && !zones::minus_counter(g, y, 1) {
            why = "the -1/-1 counter could not be put on a creature".into();
            break;
        }
        zones::draw(g, p, 1, false)?;
        life::check_state(g)?;
        done += 1;
        (a, b) = (nx, y);
    }
    crate::glog!(
        g,
        "    the loop went round {done} time(s): {} lost {} life net, library {} -> {}{}",
        pname(g, p),
        start_life - g.player(p).life,
        start_lib,
        g.player(p).library.len(),
        if why.is_empty() { String::new() } else { format!(" (stopped: {why})") }
    );
    Ok(None)
}

// ------------------------------------------------------------------ Mikaeus + Triskelion
/// Mikaeus + Triskelion (the combo catalogue's loop 0): Triskelion comes back from undying with four counters; take
/// them all off (the costs), two of the four pings kill it (a 2/2 under Mikaeus) and the others go anywhere. The
/// person says what to hit: each opponent's life to 0, or a creature. Plays trip by trip with the engine's own
/// death and undying code (drain payoffs trigger on every trip).
pub fn trisk_loop(g: &mut Game, p: PlayerId, keys: &[PermId]) -> Res<Why> {
    if named(g, p, TRISK).is_none() || named(g, p, MIKAEUS).is_none() {
        return Ok(Some("The loop needs Mikaeus and Triskelion on the battlefield.".into()));
    }
    crate::glog!(g, "  {} goes for the Mikaeus + Triskelion loop", pname(g, p));
    if crate::ai::combo_interrupted(g, p, "seph", keys)? {
        crate::glog!(g, "    ...the loop is stopped");
        return Ok(None);
    }
    loop {
        if g.over || !g.player(p).alive {
            return Ok(None);
        }
        let opps: Vec<PlayerId> = g.opps(p).filter(|&q| g.player(q).alive).collect();
        let cre: Vec<PermId> = creatures(g, p, true);
        let mut labels = vec![];
        if opps.len() > 1 {
            labels.push("every opponent: their life to 0".to_string());
        }
        for &q in &opps {
            labels.push(format!("{}: their life to 0 ({} life)", pname(g, q), g.player(q).life));
        }
        for &m in &cre {
            labels.push(format!("destroy {}", legal::describe_target(g, p, Target::Perm(m))));
        }
        if labels.is_empty() {
            return Ok(None);
        }
        let Some(k) = choices::choose(
            g,
            p,
            "target",
            "Mikaeus + Triskelion: Triskelion deals as much damage as you like. What does it hit?",
            &labels,
            Some("stop"),
            None,
        )?
        else {
            return Ok(None);
        };
        let shift = usize::from(opps.len() > 1);
        if shift == 1 && k == 0 {
            for &q in &opps {
                trisk_kill_player(g, p, q)?;
            }
        } else if k - shift < opps.len() {
            trisk_kill_player(g, p, opps[k - shift])?;
        } else {
            let m = cre[k - shift - opps.len()];
            if g.perm(m).on_bf {
                crate::glog!(g, "    Triskelion's pings destroy {}", g.perm(m).name);
                super::cards::damage(g, p, Target::Perm(m), 1000, "Triskelion", "triggers")?;
            }
        }
    }
}

/// trips of the Mikaeus + Triskelion loop until player q is dead (or the loop breaks)
fn trisk_kill_player(g: &mut Game, p: PlayerId, q: PlayerId) -> Res {
    let (mut trips, start) = (0, g.player(q).life);
    let mut why = "";
    while g.player(q).alive && !g.over {
        let (Some(t), Some(_)) = (named(g, p, TRISK), named(g, p, MIKAEUS)) else {
            why = "Mikaeus or Triskelion left the battlefield";
            break;
        };
        // all the counters come off as costs; what Triskelion needs to die decides how many pings are free
        let counters = g.perm(t).plus;
        g.perm_mut(t).plus = 0;
        let to_die = etgh(g, t).max(1);
        let free = counters - to_die;
        if free <= 0 {
            g.perm_mut(t).plus = counters;
            why = "Triskelion does not have a counter to spare";
            break;
        }
        let before = g.player(q).life;
        life::lose_life(g, q, free, Some(p), "triggers", None)?;
        zones::die(g, t, "destroy")?; // the rest of the pings hit Triskelion; undying brings it back
        life::check_state(g)?;
        trips += 1;
        if g.player(q).alive && g.player(q).life >= before {
            why = "the damage is being prevented";
            break;
        }
        if trips > 5000 {
            why = "too many trips";
            break;
        }
    }
    crate::glog!(
        g,
        "    Triskelion goes round {trips} time(s): {} {} -> {}{}",
        pname(g, q),
        start,
        g.player(q).life,
        if why.is_empty() { String::new() } else { format!(" (stopped: {why})") }
    );
    Ok(())
}

// ------------------------------------------------------------------ Niv-Mizzet's standing ping target
fn is_niv(g: &Game, m: PermId) -> bool {
    card_name(g, m).starts_with("Niv-Mizzet")
}

/// where Niv-Mizzet m's pings go this turn by the person's standing choice
pub fn standing_ping(g: &Game, m: PermId) -> Option<PlayerId> {
    match g.perm(m).data.get(DataKey::Ping) {
        Some(Val::List(v)) if v.len() == 2 && v[0] == Val::Stamp(g.turn_stamp()) => match v[1] {
            Val::Player(q) if g.player(q).alive && legal::player_targetable(g, q) => Some(q),
            _ => None,
        },
        _ => None,
    }
}

pub fn set_standing_ping(g: &mut Game, m: PermId, q: Option<PlayerId>) {
    let st = g.turn_stamp();
    match q {
        Some(q) => g.perm_mut(m).data.set(DataKey::Ping, Val::List(vec![Val::Stamp(st), Val::Player(q)])),
        None => {
            g.perm_mut(m).data.remove(DataKey::Ping);
        }
    }
}

/// a group of simultaneous triggers that are all pings of Niv-Mizzets with a standing target: their order can't matter
pub fn niv_pings_standing(g: &Game, ts: &[crate::hooks::Trigger]) -> bool {
    !ts.is_empty() && ts.iter().all(|t| t.src.is_some_and(|m| is_niv(g, m) && standing_ping(g, m).is_some()))
}

/// Niv-Mizzet's ability for the person: choose where every ping goes this turn
pub fn niv_abilities(g: &Game, m: PermId) -> Vec<PermAbility> {
    if !is_niv(g, m) {
        return vec![];
    }
    vec![ability(
        "choose where all of its pings go this turn (or go back to asking each time)",
        |g: &mut Game, p: PlayerId, m: PermId| {
            let opps: Vec<PlayerId> = g.opps(p).filter(|&q| legal::player_targetable(g, q)).collect();
            let mut labels: Vec<String> =
                opps.iter().map(|&q| format!("{} ({} life)", pname(g, q), g.player(q).life)).collect();
            labels.push("ask me each time".into());
            let Some(k) = choices::choose(
                g,
                p,
                "target",
                "Niv-Mizzet: where do its pings go this turn?",
                &labels,
                Some("cancel"),
                None,
            )?
            else {
                return Ok(None);
            };
            set_standing_ping(g, m, opps.get(k).copied());
            match opps.get(k) {
                Some(&q) => {
                    crate::glog!(g, "  {} sends every Niv-Mizzet ping to {} this turn", pname(g, p), pname(g, q))
                }
                None => crate::glog!(g, "  {} will choose each Niv-Mizzet ping", pname(g, p)),
            }
            Ok(None)
        },
    )]
}
