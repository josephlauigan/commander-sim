//! The Yoshimaru list's Syr Vondam, Sunstar Exemplar and Elspeth, Knight-Errant.
//!
//! # Syr Vondam, Sunstar Exemplar
//! {W}{B}, 2/2, vigilance, menace.
//!
//! - "Whenever another creature you control dies or is put into exile, put a +1/+1 counter on Syr Vondam and you gain
//!   1 life": the `dies` and `exiled_from_bf` hooks. Both look back in time (CR 603.10a): the creature counts as it last
//!   existed on the battlefield (a token too, and whoever controlled it when it left). A creature that leaves in the
//!   same wipe as Syr Vondam still triggers him: `left_with` finds him (he left in the same batch) and queues his
//!   trigger for the creatures that leave after him; the counter then has nowhere to go and the life is gained.
//! - "When Syr Vondam dies or is put into exile while its power is 4 or greater, destroy up to one target nonland
//!   permanent": the `self_dies` and `self_exiled` hooks. The power is the last known one, taken as he leaves
//!   (`leaving`, called by `zones::leave` before anything about him changes: an Equipment still attached counts), and is
//!   not checked again as the ability resolves. The target is chosen as it resolves (the AI: the best opposing
//!   permanent that dies to it, or none; a person: any nonland permanent they may target, or none).
//!
//! # Elspeth, Knight-Errant
//! {2}{W}{W}, loyalty 4. A card-code walker (`dsl = []`): the AI's list is `ELSPETH_AB` here, a person's is in
//! `human/walkers.rs` (both call the effects below).
//! - +1: a 1/1 white Soldier creature token.
//! - +1: target creature gets +3/+3 and gains flying until end of turn. The AI aims it at its own creature that can
//!   attack now (the hardest hitter), else its best creature; a person picks any creature Elspeth may target (hexproof,
//!   shroud and protection from white respected), before the loyalty is paid. `pump_legal` is checked again as the
//!   ability resolves, and a person pays any ward then (`ward::pay_ward`).
//! - -8: the emblem `elspeth_ke` (`Player::emblems`): `chars::has_keyword` gives indestructible to the artifacts,
//!   creatures, enchantments and lands its owner controls (not planeswalkers; sacrifice, toughness 0 and -X/-X still
//!   kill).

use crate::cards::{CardDb, Colors, Types};
use crate::engine::chars::card_types;
use crate::engine::life::gain;
use crate::engine::removal::apply_removal;
use crate::engine::repl;
use crate::engine::stack::{queue_triggers, trigger_window};
use crate::engine::values::{colors_of, epow, indestructible, protected_from, pval, untargetable};
use crate::engine::zones::{Tokens, make_tokens, max_by};
use crate::flow::Res;
use crate::hooks::{Call, Event, Registry, Src, TrigAct, Trigger};
use crate::human::cards::pick_perm;
use crate::ids::{PermId, PlayerId};
use crate::impls::common::{WalkerAb, walker};
use crate::state::{DataKey, Game, Val};
use crate::sym::Sym;

pub const VONDAM: &str = "Syr Vondam, Sunstar Exemplar";
/// the emblem Elspeth, Knight-Errant's -8 gives (`Player::emblems`)
pub const EMBLEM: &str = "elspeth_ke";

// ------------------------------------------------------------------ last known information
/// `zones::leave`, before the permanent is taken off the battlefield (or off an Equipment): Syr Vondam's power as he
/// leaves, and the wipe (`Game::cur_batch`, 0 outside one) he leaves in. Kept on the permanent that left, under
/// `DataKey::LastKnown`. Inside a wipe the power is the one from before anything left (CR 603.10a: the wipe's permanents
/// leave at the same time): the first leave of the batch records it for every Syr Vondam on the battlefield, so an anthem
/// the wipe took first no longer counts as gone.
pub fn leaving(g: &mut Game, m: PermId) {
    let Some(b) = g.cur_batch else {
        if g.perm(m).name == VONDAM {
            let pow = epow(g, m);
            record(g, m, pow, 0);
        }
        return;
    };
    let mut vs: Vec<PermId> = g
        .players
        .iter()
        .flat_map(|q| q.perms.iter().copied())
        .filter(|&v| g.perm(v).name == VONDAM && g.perm(v).on_bf && !g.perm(v).phased)
        .collect();
    if g.perm(m).name == VONDAM && !vs.contains(&m) {
        vs.push(m); // he leaves phased out
    }
    for v in vs {
        if last_known(g, v).is_none_or(|(_, vb)| vb != b) {
            let pow = epow(g, v);
            record(g, v, pow, b);
        }
    }
}

fn record(g: &mut Game, m: PermId, pow: i32, batch: u32) {
    g.perm_mut(m).data.set(DataKey::LastKnown, Val::List(vec![Val::Int(pow as i64), Val::Int(batch as i64)]));
}

/// (the power, the wipe) recorded as the permanent left, if it did
fn last_known(g: &Game, m: PermId) -> Option<(i32, u32)> {
    match g.perm(m).data.get(DataKey::LastKnown) {
        Some(Val::List(v)) if v.len() == 2 => Some((v[0].int() as i32, v[1].int() as u32)),
        _ => None,
    }
}

// ------------------------------------------------------------------ another creature you control
/// the creature m left the battlefield under `o`'s control: does Syr Vondam (src, controlled by o) see it?
fn sees(g: &Game, src: PermId, m: PermId) -> bool {
    m != src && g.perm(m).owner == g.perm(src).owner && g.is_creature(m)
}

fn another(g: &mut Game, src: Src, m: PermId) -> Res {
    if !sees(g, src, m) {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if !trigger_window(g, o, Some(src), "a +1/+1 counter and 1 life", Some(2.0))? {
        return Ok(());
    }
    if g.perm(src).on_bf && !g.perm(src).phased {
        repl::put_plus(g, src, 1); // (not when he left in the same wipe: the counter has nowhere to go)
    }
    gain(g, o, 1)
}

fn another_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    another(g, src, m)
}

fn another_exiled(g: &mut Game, src: Src, m: PermId) -> Res {
    another(g, src, m)
}

/// `die_triggers` (dying) and `exiled_from_bf` (exiled): the creature m left during a wipe; each Syr Vondam of its
/// controller that left earlier in the same wipe triggers for it too (they left at the same time)
pub fn left_with(g: &mut Game, m: PermId, dying: Option<Sym>) -> Res {
    let Some(b) = g.cur_batch else { return Ok(()) };
    let mut ts = vec![];
    for i in 0..g.perms.len() {
        let v = PermId(i as u32);
        let x = g.perm(v);
        if x.on_bf
            || x.neutered // he had lost his abilities as he left
            || x.name != VONDAM
            || v == m
            || last_known(g, v).is_none_or(|(_, vb)| vb != b)
            || !sees(g, v, m)
        {
            continue;
        }
        let (event, call) = match dying {
            Some(cause) => (Event::Dies, Call::Dies { m, cause }),
            None => (Event::ExiledFromBf, Call::Leaves { m }),
        };
        ts.push(Trigger {
            controller: x.owner,
            src: Some(v),
            act: TrigAct::Hook { event, call },
            name: None,
            known: false,
            imp: None,
            cast_etb: false,
        });
    }
    queue_triggers(g, ts)
}

// ------------------------------------------------------------------ Syr Vondam himself
/// what the destroy can hit: a nonland permanent on the battlefield that `o` may target with Syr Vondam's colours
fn targets(g: &Game, o: PlayerId, src: PermId) -> Vec<PermId> {
    let cols = colors_of(g, src);
    let mut out = vec![];
    for q in g.players.iter().filter(|q| q.alive) {
        for &t in &q.perms {
            let x = g.perm(t);
            if x.on_bf
                && !x.phased
                && !card_types(g, t).has(Types::LAND)
                && (q.id == o || !untargetable(g, t))
                && !protected_from(g, t, cols)
            {
                out.push(t);
            }
        }
    }
    out
}

/// the AI's pick: the opposing permanent most worth destroying that dies to it, if any is worth it
fn ai_target(g: &Game, o: PlayerId, src: PermId) -> Option<PermId> {
    let cs: Vec<PermId> =
        targets(g, o, src).into_iter().filter(|&t| g.perm(t).owner != o && !indestructible(g, t)).collect();
    max_by(&cs, |t| pval(g, t)).filter(|&t| pval(g, t) >= 2.0)
}

/// Syr Vondam (m) left the battlefield (died or was exiled) with power 4 or more
fn destroy(g: &mut Game, m: PermId) -> Res {
    if g.perm(m).neutered || last_known(g, m).is_none_or(|(pow, _)| pow < 4) {
        return Ok(());
    }
    let o = g.perm(m).owner;
    if !trigger_window(g, o, Some(m), "destroy up to one target nonland permanent", Some(5.0))? {
        return Ok(());
    }
    let t = if crate::human::is_human(g, o) {
        let cs = targets(g, o, m);
        if cs.is_empty() {
            return Ok(());
        }
        pick_perm(g, o, "target", "Syr Vondam: destroy which nonland permanent?", &cs, Some("no target"))?
    } else {
        ai_target(g, o, m)
    };
    if let Some(t) = t {
        apply_removal(g, Some(o), t, "destroy", None)?;
    }
    Ok(())
}

fn self_dies(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    destroy(g, m)
}

fn self_exiled(g: &mut Game, _src: Src, m: PermId) -> Res {
    destroy(g, m)
}

// ------------------------------------------------------------------ Elspeth, Knight-Errant
pub const ELSPETH: &str = "Elspeth, Knight-Errant";

/// +1: a 1/1 white Soldier creature token
fn soldier_val(_g: &Game, _p: PlayerId, _src: PermId) -> Option<f64> {
    Some(3.0)
}

pub fn soldier(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let spec = Tokens { color: Some(Colors::from_letters("W")), types: vec!["soldier"], ..Tokens::new(1, 1) };
    make_tokens(g, p, spec).map(|_| ())
}

/// can Elspeth (src, p's) give `t` +3/+3 and flying: a creature on the battlefield, not phased out, no shroud, no
/// protection from Elspeth's colours, and not an opponent's creature with hexproof. Checked as the ability is chosen and
/// again as it resolves.
pub fn pump_legal(g: &Game, p: PlayerId, src: PermId, t: PermId) -> bool {
    let x = g.perm(t);
    x.on_bf
        && !x.phased
        && g.is_creature(t)
        && !crate::dsl::has_kw(g, t, "shroud")
        && (x.owner == p || !untargetable(g, t))
        && !protected_from(g, t, colors_of(g, src))
}

/// +1: the creature the AI pumps: one of its own that can attack now (the one that hits hardest), else its best creature
fn pump_target(g: &Game, p: PlayerId, src: PermId) -> Option<PermId> {
    let mine: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| pump_legal(g, p, src, m)).collect();
    let ready: Vec<PermId> = mine
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            !x.tapped && !x.noatk && (!x.sick || crate::dsl::has_kw(g, m, "haste"))
        })
        .collect();
    let pool = if ready.is_empty() { &mine } else { &ready };
    max_by(pool, |m| epow(g, m) as f64 + 0.1 * pval(g, m))
}

fn pump_val(g: &Game, p: PlayerId, src: PermId) -> Option<f64> {
    let t = pump_target(g, p, src)?;
    let x = g.perm(t);
    let attacks = !x.tapped && !x.noatk && (!x.sick || crate::dsl::has_kw(g, t, "haste"));
    Some(if attacks { 4.0 } else { 1.5 })
}

fn pump_eff(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    if let Some(t) = pump_target(g, p, src) {
        pump(g, t);
    }
    Ok(())
}

/// the creature gets +3/+3 and gains flying until end of turn
pub fn pump(g: &mut Game, t: PermId) {
    let x = g.perm_mut(t);
    x.eot_pt = (x.eot_pt.0 + 3, x.eot_pt.1 + 3);
    crate::impls::partials::eot_kw(g, t, "flying");
}

/// -8: you get an emblem with "Artifacts, creatures, enchantments, and lands you control have indestructible"
fn emblem_val(_g: &Game, _p: PlayerId, _src: PermId) -> Option<f64> {
    Some(9.0)
}

pub fn emblem(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let e = &mut g.player_mut(p).emblems;
    if !e.contains(&EMBLEM) {
        e.push(EMBLEM);
    }
    crate::glog!(
        g,
        "    {} gets an emblem: their artifacts, creatures, enchantments and lands have indestructible",
        g.player(p).name
    );
    Ok(())
}

static ELSPETH_AB: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "create a 1/1 white Soldier creature token", val: soldier_val, eff: soldier },
    WalkerAb {
        delta: 1,
        label: "target creature gets +3/+3 and gains flying until end of turn",
        val: pump_val,
        eff: pump_eff,
    },
    WalkerAb {
        delta: -8,
        label: "you get an emblem: artifacts, creatures, enchantments and lands you control have indestructible",
        val: emblem_val,
        eff: emblem,
    },
];

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, VONDAM)?;
    c.dies = Some(another_dies);
    c.exiled_from_bf = Some(another_exiled);
    c.self_dies = Some(self_dies);
    c.self_exiled = Some(self_exiled);
    walker(r, db, ELSPETH, &ELSPETH_AB)
}
