//! Python's `cards/impl/marchesa.py`: card rules first written for the Marchesa deck (removed 2026-10-07), kept for
//! the cards other lists still run: Marchesa, the Black Rose (Jodah's 99), Coalition Relic, Notion Thief's flash,
//! Hellkite Tyrant's alternate win, Accursed Marauder, and the shared helpers other modules use (best_steal and steal,
//! card_etb_value). Deep Analysis, Last Gasp and Tezzeret's Gambit are tags only (data/cards.json).

use super::partials::{at_once, remove_card};
use crate::cards::{CardDb, Types};
use crate::engine::cast::cast_card;
use crate::engine::hooks::total_player;
use crate::engine::life::check_state;
use crate::engine::mana::{can_pay, pay};
use crate::engine::removal::legal_targets;
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::values::{has, pval};
use crate::engine::zones::{Enter, die, enter, max_by, min_by, sac_worth};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::pysum::PySum;
use crate::state::{Ctx, DataKey, Game, Val};
use crate::tag::Tag;

pub const MARCHESA: &str = "Marchesa, the Black Rose";

/// marchesa.marchesa_out: p controls Marchesa
fn marchesa_out(g: &Game, p: PlayerId) -> bool {
    has(g, p, Tag::Marchesa)
}

// ================================================================== Marchesa, the Black Rose
/// dethrone, and other creatures you control have dethrone; a creature you control with a +1/+1 counter that dies (a
/// stolen one too, and those dying in the same wipe as Marchesa) returns under your control at the beginning of the
/// next end step; she returns herself from the command zone the same way. The engine starts checking deaths for her
/// trigger once she has entered.
fn marchesa_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.marchesa_on = true;
    }
    Ok(())
}

/// marchesa.marchesa_dies: creature m that p controlled died: Marchesa's trigger when p controls her, when she is the
/// creature, or when she died in the same event (a wipe: her ability looks back in time)
pub fn marchesa_dies(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    let x = g.perm(m);
    let Some(mcd) = x.cd else { return Ok(()) };
    let is_m = g.db.get(mcd).tag(Tag::Marchesa);
    let batch = g.cur_batch;
    if is_m && batch.is_some() {
        g.player_mut(p).marchesa_batch = batch;
    }
    let x = g.perm(m);
    if x.plus <= 0 {
        return Ok(());
    }
    if !(is_m || marchesa_out(g, p) || (batch.is_some() && g.player(p).marchesa_batch == batch)) {
        return Ok(());
    }
    let cd = x.phys.unwrap_or(mcd);
    let owner = x.orig;
    // (Python also sets `owner.cmd_pending` for a commander here; nothing reads it)
    g.marchesa_due.push((p, cd, owner));
    crate::glog!(g, "    Marchesa: {} returns at the beginning of the next end step", g.db.get(cd).name);
    Ok(())
}

/// marchesa.marchesa_return: at the beginning of the end step, the cards Marchesa's triggers are waiting on come back
/// (if still there)
pub fn marchesa_return(g: &mut Game) -> Res {
    let due = std::mem::take(&mut g.marchesa_due);
    for (p, cd, owner) in due {
        if g.over || !g.player(p).alive {
            continue;
        }
        let is_cmd = cd == g.player(owner).cmd;
        if is_cmd {
            if !g.player(owner).cmd_in_zone {
                continue; // she went to the graveyard, not the command zone
            }
            g.player_mut(owner).cmd_in_zone = false;
        } else if !remove_card(&mut g.player_mut(owner).gy, cd) {
            continue; // exiled, reanimated or returned by something else meanwhile
        }
        let m = enter(g, p, cd, Enter { orig: Some(owner), ..Enter::default() })?;
        if is_cmd && owner == p {
            g.perm_mut(m).is_cmd = true;
        }
        g.player_mut(p).stat("marchesa_returns", 1);
        crate::glog!(g, "  Marchesa returns {} to the battlefield ({})", g.db.get(cd).name, g.player(p).name);
    }
    check_state(g)
}

/// marchesa.returns: m (a creature with a +1/+1 counter) comes back if it dies now
fn returns(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    let p = x.owner;
    if x.token || x.plus <= 0 || !(marchesa_out(g, p) || x.cd.is_some_and(|c| g.db.get(c).tag(Tag::Marchesa))) {
        return false;
    }
    !(!g.hooks.is_empty() && total_player(g, Event::NoGraveyard, p) != 0)
}

/// marchesa.marchesa_sac_worth: a creature Marchesa will return costs little to sacrifice, and re-buys what it does
/// when it enters
pub fn marchesa_sac_worth(g: &Game, m: PermId, v: f64) -> f64 {
    if !returns(g, m) {
        return v;
    }
    let x = g.perm(m);
    let auras = if g.auras.is_empty() { 0.0 } else { 0.5 * super::common::auras_on(g, m).len() as f64 };
    0.15 * v + 0.3 * x.plus as f64 + auras - card_etb_value(g, x.owner, x.phys.or(x.cd))
}

// ------------------------------------------------------------------ what re-entering is worth (sacrifice decisions)
/// mine.ETB_VALUE (Sephiroth's table: Displacer Kitten's blink values), in Python's dict order
const ETB_VALUE: [(Tag, f64); 9] = [
    (Tag::Atraxa, 8.0),
    (Tag::Archon, 6.0),
    (Tag::Rsd, 4.0),
    (Tag::Titan, 3.0),
    (Tag::Witness, 2.5),
    (Tag::Wall, 2.0),
    (Tag::Wurm, 4.0),
    (Tag::Bowmasters, 2.5),
    (Tag::Skate, 3.0),
];

/// marchesa.card_etb_value: what creature card c does for p when it enters the battlefield, on the board as it is now
pub fn card_etb_value(g: &Game, p: PlayerId, c: Option<CardId>) -> f64 {
    let Some(c) = c else { return 0.0 };
    let d = g.db.get(c);
    if !d.creature {
        return 0.0;
    }
    let opps: Vec<PlayerId> = g.opps(p).collect();
    match &*d.name {
        "Accursed Marauder" => {
            let n = opps
                .iter()
                .filter(|&&q| {
                    g.player(q).perms.iter().any(|&m| g.is_creature(m) && !g.perm(m).token && !g.perm(m).phased)
                })
                .count();
            return 1.2 * n as f64;
        }
        "Burglar Rat" => return 0.8 * opps.iter().filter(|&&q| !g.player(q).hand.is_empty()).count() as f64,
        "Deepglow Skate" => {
            let n: i32 = g.player(p).perms.iter().map(|&m| g.perm(m).plus).filter(|&x| x > 0).sum();
            return 0.5 * n as f64;
        }
        "Triskelion" => return 3.0,
        _ => {}
    }
    let mut v = ETB_VALUE.iter().filter(|(k, _)| d.tag(*k)).map(|x| x.1).psum();
    if d.tag(Tag::Draw) {
        v += 1.5 * d.tags.int(Tag::Draw).unwrap_or(0) as f64;
    }
    if g.registry.get(c).is_some_and(|i| i.live() && i.etb.is_some()) {
        v = v.max(1.5);
    }
    use crate::dsl::model::Ability;
    if d.abilities.iter().any(|a| matches!(a, Ability::Triggered { event, .. } if event == "etb")) {
        v = v.max(1.5);
    }
    v
}

// ================================================================== gaining control (Ray of Command, Abduction)
/// marchesa.best_steal: the most valuable creature p's steal spell can target
pub fn best_steal(g: &Game, p: PlayerId, spell: Option<CardId>) -> Option<PermId> {
    let tg: Vec<PermId> =
        legal_targets(g, p, "steal", "c", false, spell).into_iter().filter(|&m| !g.perm(m).phased).collect();
    max_by(&tg, |m| pval(g, m))
}

/// marchesa.steal: p gains control of m (until end of turn: untapped and hasty, back at the end step)
pub fn steal(g: &mut Game, p: PlayerId, m: PermId, until_eot: bool) {
    let q = g.perm(m).owner;
    g.player_mut(q).perms.retain(|&x| x != m);
    let x = g.perm_mut(m);
    x.owner = p;
    x.attached = None;
    g.player_mut(p).perms.push(m);
    g.bf_ver += 1;
    if until_eot {
        let x = g.perm_mut(m);
        x.tapped = false;
        x.sick = false;
        g.player_mut(p).borrowed.push(m);
    } else {
        g.perm_mut(m).sick = true;
    }
    crate::glog!(g, "    {} gains control of {} ({})", g.player(p).name, g.perm(m).name, g.player(q).name);
}

// ================================================================== Coalition Relic
/// at your end step, an untapped Relic taps for a charge counter unless held-up instants need the mana
/// HUMAN(phase 9): a person's Relic is left to them (marchesa._you)
fn relic_charge(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let x = g.perm(src);
    if p != x.owner || x.tapped || x.phased || g.humans.get(p).is_some() {
        return Ok(());
    }
    let held: Vec<CardId> = g
        .player(p)
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            let d = g.db.get(c);
            d.instant && (d.tag(Tag::Ctr) || d.tag(Tag::Rem)) && can_pay(g, p, d.generic, &d.pips, false)
        })
        .collect();
    g.perm_mut(src).tapped = true;
    if !held.is_empty()
        && !held.iter().any(|&c| {
            let d = g.db.get(c);
            can_pay(g, p, d.generic, &d.pips, false)
        })
    {
        g.perm_mut(src).tapped = false; // it would cost the held-up instant: keep it untapped
        return Ok(());
    }
    if ability_window(g, p, Some(src), "a charge counter", None, None)? {
        let n = g.perm(src).data.int(DataKey::Charge);
        g.perm_mut(src).data.set(DataKey::Charge, Val::Int(n + 1));
    }
    Ok(())
}

/// at the beginning of your precombat main phase: one mana of any colour per charge counter
fn relic_release(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let k = g.perm(src).data.int(DataKey::Charge);
    if p == g.perm(src).owner && k != 0 && trigger_window(g, p, Some(src), &format!("{k} mana of any colour"), None)? {
        let k = g.perm(src).data.int(DataKey::Charge);
        g.perm_mut(src).data.set(DataKey::Charge, Val::Int(0));
        g.player_mut(p).floating.any += k.max(0) as u32;
    }
    Ok(())
}

// ================================================================== Notion Thief: flash at end of turn
fn thief_flash(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || g.active == Some(p) || !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "UB", false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 4.0,
        label: "Notion Thief (flash)".into(),
        act: Some(Action::Plan { f: thief_go, arg: c.0 as i64 }),
    }])
}

/// marchesa.IC_cast_perm(g, p, c, 2, 'UB')
fn thief_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "UB", false) {
        return Ok(false);
    }
    pay(g, p, 2, "UB", false)?;
    cast_card(g, p, c, "hand", Ctx::default())?;
    Ok(true)
}

// ================================================================== Hellkite Tyrant: twenty artifacts
/// your upkeep with twenty or more artifacts (Treasures and Clues count): you win
fn tyrant(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != g.perm(src).owner || g.perm(src).phased {
        return Ok(());
    }
    let pl = g.player(p);
    let arts = pl.perms.iter().filter(|&&m| g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT)));
    let n = arts.count() as u32 + pl.treasures + pl.clues + pl.foods;
    if n >= 20 && trigger_window(g, p, Some(src), "win the game", Some(10.0))? {
        crate::glog!(g, "  {} controls {n} artifacts: Hellkite Tyrant wins the game", g.player(p).name);
        crate::ai::win(g, p, "alt", None)?;
    }
    Ok(())
}

// ================================================================== Accursed Marauder: each player's choice
/// enters: each player sacrifices a nontoken creature of their choice (the one cheapest to lose: Marchesa's returns)
/// HUMAN(phase 9): a person chooses their creature
fn marauder(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if !trigger_window(g, o, Some(src), "each player sacrifices a creature", Some(5.0))? {
        return Ok(());
    }
    for i in 0..g.players.len() {
        let q = PlayerId(i as u8);
        if !g.player(q).alive {
            continue;
        }
        let cr: Vec<PermId> = g
            .player(q)
            .perms
            .iter()
            .copied()
            .filter(|&x| g.is_creature(x) && !g.perm(x).phased && !g.perm(x).token)
            .collect();
        if let Some(x) = min_by(&cr, |x| sac_worth(g, x)) {
            die(g, x, "sac")?;
        }
    }
    Ok(())
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, MARCHESA)?;
    c.etb = Some(marchesa_etb);
    at_once(c, Event::Etb, true);
    let c = r.card(db, "Coalition Relic")?;
    c.end_step = Some(relic_charge);
    at_once(c, Event::EndStep, true);
    c.upkeep = Some(relic_release);
    r.card(db, "Notion Thief")?.hand_options = Some(thief_flash);
    r.card(db, "Hellkite Tyrant")?.upkeep = Some(tyrant);
    r.card(db, "Accursed Marauder")?.etb = Some(marauder);
    Ok(())
}
