//! Triggered abilities, the stack, priority and counterspells (engine.py: queue_triggers ... counter_side_effects).
//!
//! Triggers wait in `g.trig_queue` until whatever is resolving finishes, then go on the stack in APNAP order. A
//! card's hook is probed first (run in probe mode until its `trigger_window`, which stops it with `Stop::Probe`) to
//! find out whether it really triggers, then pushed, then run for real top first: its `trigger_window` gives every
//! player priority on it and says whether it was countered.

use crate::ai::hand_key; // --- phase4-hand-off
use crate::ai;
use crate::engine::altcost;
use crate::engine::cast::discard_worst;
use crate::engine::cast::{castable, on_cast};
use crate::engine::hooks::run_hook;
use crate::engine::life::{gain, lose_life};
use crate::engine::mana::{can_pay, pay};
use crate::engine::values::{colors_of, commander_out, has, has_type, pval};
use crate::engine::zones::{Enter, Tokens, add_treasure, draw, enter, make_tokens};
use crate::flow::{Probe, Res, Stop};
use crate::hooks::{TrigAct, Trigger};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game, StackItem, StackKind};
use crate::sym::Sym;
use crate::tag::Tag;

// ------------------------------------------------------------------ the trigger queue
/// triggered abilities that just triggered: on the stack now, or once the spell resolving has finished
pub fn queue_triggers(g: &mut Game, mut entries: Vec<Trigger>) -> Res {
    if entries.is_empty() {
        return Ok(());
    }
    for t in &mut entries {
        t.cast_etb = g.last_cast_etb;
    }
    // Cloud, Midgar Mercenary: a trigger of his, or of an Equipment attached to him, triggers an additional time
    let extra: Vec<Trigger> =
        entries.iter().filter(|t| t.src.is_some_and(|s| crate::impls::cloud::doubles(g, s))).cloned().collect();
    entries.extend(extra);
    // --- Tiers 4/5 (heliod-zur): Delney: a trigger of a creature with power 2 or less triggers an additional time
    let delney: Vec<Trigger> = entries
        .iter()
        .flat_map(|t| {
            let k = t.src.map_or(0, |s| crate::impls::t45_heliod_zur::delney_extra(g, s));
            std::iter::repeat_n(t.clone(), k)
        })
        .collect();
    entries.extend(delney);
    // --- Tiers 4/5 (urdragon): Roaming Throne: a trigger of another creature of the chosen type triggers again
    let throne: Vec<Trigger> = entries
        .iter()
        .flat_map(|t| {
            let k = t.src.map_or(0, |s| crate::impls::t45_urdragon::throne_extra(g, s));
            std::iter::repeat_n(t.clone(), k)
        })
        .collect();
    entries.extend(throne);
    g.trig_queue.extend(entries);
    if g.resolving == 0 { flush_triggers(g) } else { Ok(()) }
}

pub fn flush_triggers(g: &mut Game) -> Res {
    while !g.trig_queue.is_empty() && !g.over {
        let batch = std::mem::take(&mut g.trig_queue);
        resolve_batch(g, batch)?;
    }
    Ok(())
}

/// active player's triggers first (they go on the stack first, so resolve last), then in turn order
pub fn apnap(g: &Game, mut ts: Vec<Trigger>) -> Vec<Trigger> {
    let a = g.active.or_else(|| ts.first().map(|t| t.controller));
    let order: Vec<PlayerId> = match a {
        Some(a) => std::iter::once(a).chain(g.after(a)).collect(),
        None => g.players.iter().map(|q| q.id).collect(),
    };
    ts.sort_by_key(|t| order.iter().position(|&q| q == t.controller).unwrap_or(99)); // stable
    ts
}

/// carry out a trigger's effect
fn call(g: &mut Game, t: &Trigger) -> Res {
    let prev = std::mem::replace(&mut g.last_cast_etb, t.cast_etb);
    let r = match &t.act {
        TrigAct::Hook { event, call } => run_hook(g, t.src.expect("a hook trigger has a source"), *event, call),
        TrigAct::EtbOnce { p, m } => crate::engine::zones::etb_once(g, *p, *m),
        TrigAct::Dsl { p, src, card, idx, ctx } => crate::dsl::run_trigger(g, *p, *src, *card, *idx, ctx),
    };
    g.last_cast_etb = prev;
    r
}

fn next_id(g: &mut Game) -> u32 {
    g.stack_pushes += 1;
    g.stack_pushes
}

fn on_stack(g: &Game, id: u32) -> bool {
    g.stack.iter().any(|it| it.id == id)
}

fn remove_item(g: &mut Game, id: u32) {
    g.stack.retain(|it| it.id != id);
}

fn item(g: &Game, id: u32) -> Option<&StackItem> {
    g.stack.iter().find(|it| it.id == id)
}

fn src_name(g: &Game, src: Option<PermId>) -> String {
    src.map_or_else(|| "A token".into(), |m| g.perm(m).name.to_string())
}

fn resolve_batch(g: &mut Game, batch: Vec<Trigger>) -> Res {
    let batch = apnap(g, batch);
    // --- Tiers 1/2 (brago-meren): a Strionic Resonator copies triggers too
    let copier = |g: &Game, q: PlayerId| camera_ready(g, q) || crate::impls::t12_brago_meren::resonator_ready(g, q);
    if !abilities_answered(g, None) && !batch.iter().any(|t| copier(g, t.controller)) {
        // nobody could respond: as before the stack, in order
        for t in &batch {
            if g.over {
                break;
            }
            call(g, t)?;
        }
        return Ok(());
    }
    let mut real = vec![];
    for mut t in batch {
        // which hooks really trigger (guards only, no effects yet)
        if t.known {
            real.push(t);
            continue;
        }
        let prev_ce = std::mem::replace(&mut g.last_cast_etb, t.cast_etb);
        let prev = std::mem::replace(&mut g.trig_probe, true);
        let r = call(g, &t);
        g.trig_probe = prev;
        g.last_cast_etb = prev_ce;
        match r {
            Err(Stop::Probe(pr)) => {
                t.controller = pr.controller;
                t.name = Some(crate::sym::intern(&pr.name));
                t.imp = Some(pr.imp);
                real.push(t);
            }
            Err(e) => return Err(e),
            Ok(()) => {} // its guard failed: it doesn't trigger
        }
    }
    let real = crate::human::human::order_triggers(g, real)?; // a person orders their own simultaneous triggers
    let mut items = vec![];
    for t in real {
        let id = next_id(g);
        let card = t.src.and_then(|m| g.perm(m).cd);
        let name = format!("{}: {}", src_name(g, t.src), t.name.unwrap_or("trigger"));
        // --- card-models-2: a compiled trigger with a single target is aimed as it goes on the stack
        let (target, dsl_ab) = match &t.act {
            TrigAct::Dsl { p, src, card, idx, ctx } if ctx.target.is_none() => {
                (crate::dsl::aim(g, *p, *card, *idx, Some(*src)), Some((*card, *idx)))
            }
            TrigAct::Dsl { card, idx, .. } => (None, Some((*card, *idx))),
            _ => (None, None),
        };
        g.stack.push(StackItem {
            id,
            controller: t.controller,
            card,
            ctx: Ctx { source: t.src, trigger: Some(Box::new(t.clone())), target, dsl_ab, ..Ctx::default() },
            zone: "trigger",
            imp: t.imp.unwrap_or(3.0),
            aff: vec![],
            generic: false,
            kind: StackKind::Trigger,
            name,
            passed: vec![],
            countered: false,
            countered_by: None,
        });
        items.push((t, id));
    }
    for (t, id) in items.into_iter().rev() {
        // the top of the stack resolves first
        if g.over {
            break;
        }
        if !on_stack(g, id) {
            continue;
        }
        if t.known {
            // the engine's and the ability language's: the window is here
            let first = g.active.unwrap_or(t.controller);
            priority(g, id, first)?;
            let copies = take_copies(g, id);
            let countered = item(g, id).is_some_and(|it| it.countered);
            let aimed = item(g, id).and_then(|it| it.ctx.target);
            remove_item(g, id);
            for _ in 0..copies {
                run_copy(g, &t)?; // Camera copies resolve before the original (choosing targets anew)
            }
            if !countered {
                // --- card-models-2: the target it was aimed at as it went on the stack (or was changed to)
                match (&t.act, aimed) {
                    (TrigAct::Dsl { p, src, card, idx, ctx }, Some(a)) => {
                        if g.perm(a).on_bf {
                            let ctx = Box::new(crate::dsl::DslCtx { target: Some(a), ..(**ctx).clone() });
                            let act = TrigAct::Dsl { p: *p, src: *src, card: *card, idx: *idx, ctx };
                            call(g, &Trigger { act, ..t.clone() })?;
                        } // its only target is gone: it does nothing
                    }
                    _ => call(g, &t)?,
                }
            }
            continue;
        }
        let prev = g.trig_current.replace(id);
        let flow_prev = g.player(t.controller).flow_src; // --- ai-flow: the source in force before it
        let r = call(g, &t); // its trigger_window takes the item: priority, then it resolves
        crate::t6stats::flow_source(g, t.controller, flow_prev);
        g.trig_current = prev;
        remove_item(g, id); // (if its guard failed this time, it no longer triggers)
        r?;
    }
    Ok(())
}

/// A triggered ability's commit point (after its conditions, before its effect). While the engine probes pending
/// hooks it stops with `Stop::Probe`; while resolving a stack item for it, players get priority on it first; called
/// from engine code outside a batch, it gets its own round. True if it resolves. Python's `trigger_window`.
pub fn trigger_window(g: &mut Game, p: PlayerId, src: Option<PermId>, name: &str, imp: Option<f64>) -> Res<bool> {
    if g.trig_probe {
        g.trig_probe = false;
        return Err(Stop::Probe(Probe { controller: p, src, name: name.into(), imp: imp.unwrap_or(3.0) }));
    }
    let flow = src.and_then(|m| g.perm(m).cd); // --- ai-flow: the trigger's draws are its source's
    crate::t6stats::flow_source(g, p, flow);
    if let Some(cur) = g.trig_current.take() {
        if on_stack(g, cur) {
            let first = g.active.unwrap_or(p);
            priority(g, cur, first)?;
            if let Some(t) = item(g, cur).and_then(|it| it.ctx.trigger.clone()) {
                for _ in 0..take_copies(g, cur) {
                    run_copy(g, &t)?; // each copy resolves before the original
                }
            }
            let countered = item(g, cur).is_some_and(|it| it.countered);
            remove_item(g, cur);
            return Ok(!countered);
        }
        return Ok(true);
    }
    if g.over || !abilities_answered(g, Some(p)) {
        return Ok(true);
    }
    let card = src.and_then(|m| g.perm(m).cd);
    let full = if src.is_some() { format!("{}: {name}", src_name(g, src)) } else { name.to_string() };
    let it = new_item(
        g,
        p,
        card,
        Ctx { source: src, ..Ctx::default() },
        "trigger",
        imp.unwrap_or(3.0),
        StackKind::Trigger,
        full,
    );
    stack_window(g, p, it)
}

/// `trigger_window` for a "you may" trigger. The window works as before (the table may respond, the trigger may be
/// countered); then, if it resolves and `p` is a person, they are asked yes/no and `false` comes back on no. For the
/// AI it is exactly `trigger_window` (the card code's own value test decides). The default question is
/// "<card>: <name>?"; use `may_window_ask` for a better one. The question comes at resolution, which is when Oracle
/// has you choose; each copy of a copied trigger asks again.
pub fn may_window(g: &mut Game, p: PlayerId, src: Option<PermId>, name: &str, imp: Option<f64>) -> Res<bool> {
    let prompt = format!("{}: {name}?", src_name(g, src));
    may_window_ask(g, p, src, name, imp, &prompt)
}

/// `may_window` with the question spelled out (start it with the card's name)
pub fn may_window_ask(
    g: &mut Game,
    p: PlayerId,
    src: Option<PermId>,
    name: &str,
    imp: Option<f64>,
    prompt: &str,
) -> Res<bool> {
    let ok = trigger_window(g, p, src, name, imp)?;
    if !ok || !crate::human::is_human(g, p) {
        return Ok(ok);
    }
    crate::human::choices::yes_no(g, p, prompt)
}

/// trigger_window for a trigger whose source is a card, not a permanent (a land's enter trigger)
pub fn trigger_window_card(g: &mut Game, p: PlayerId, cd: CardId, name: &str) -> Res<bool> {
    if g.trig_probe {
        g.trig_probe = false;
        return Err(Stop::Probe(Probe { controller: p, src: None, name: name.into(), imp: 3.0 }));
    }
    if g.trig_current.is_some() || g.over || !abilities_answered(g, Some(p)) {
        let r = trigger_window(g, p, None, name, None);
        crate::t6stats::flow_source(g, p, Some(cd)); // --- ai-flow: the card is the source (trigger_window has none)
        return r;
    }
    crate::t6stats::flow_source(g, p, Some(cd)); // --- ai-flow
    let full = format!("{}: {name}", g.db.get(cd).name);
    let it = new_item(g, p, Some(cd), Ctx::default(), "trigger", 3.0, StackKind::Trigger, full);
    stack_window(g, p, it)
}

/// `may_window` for a trigger whose source is a card, not a permanent (Jocasta returning from the graveyard)
pub fn may_window_card(g: &mut Game, p: PlayerId, cd: CardId, name: &str, prompt: &str) -> Res<bool> {
    let ok = trigger_window_card(g, p, cd, name)?;
    if !ok || !crate::human::is_human(g, p) {
        return Ok(ok);
    }
    crate::human::choices::yes_no(g, p, prompt)
}

#[allow(clippy::too_many_arguments)]
fn new_item(
    g: &mut Game,
    p: PlayerId,
    card: Option<CardId>,
    ctx: Ctx,
    zone: Sym,
    imp: f64,
    kind: StackKind,
    name: String,
) -> StackItem {
    let id = g.stack_pushes + 1; // stack_window counts the push
    StackItem {
        id,
        controller: p,
        card,
        ctx,
        zone,
        imp,
        aff: vec![],
        generic: false,
        kind,
        name,
        passed: vec![],
        countered: false,
        countered_by: None,
    }
}

/// A spell item for card c with a fresh id, as cast_card would make it (tests and card code that put a spell on the
/// stack by hand). Python's `StackItem(p, c, ctx, 'hand', imp, {})`.
pub fn spell_item(g: &mut Game, p: PlayerId, c: CardId, ctx: Ctx, imp: f64) -> StackItem {
    g.stack_pushes += 1;
    StackItem {
        id: g.stack_pushes,
        controller: p,
        card: Some(c),
        ctx,
        zone: "hand",
        imp,
        aff: vec![],
        generic: true,
        kind: StackKind::Spell,
        name: g.db.get(c).name.to_string(),
        passed: vec![],
        countered: false,
        countered_by: None,
    }
}

/// cards the AI uses to counter abilities
pub const ABILITY_ANSWERS: [&str; 2] = ["Tishana's Tidebinder", "Azorius Guildmage"];

/// flash cards the AI casts from its hand to counter an ability (Ertai Resurrected, Louisoix's Sacrifice), or to change
/// its target (--- card-models-2: Redirect Lightning)
pub const ABILITY_ANSWERS_HAND: [&str; 3] = ["Ertai Resurrected", "Louisoix's Sacrifice", "Redirect Lightning"];

/// Could anyone answer an ability of p's (a person at the table, or an opponent with an ability counter)? When nobody
/// could, an ability resolves without a round of priority, which keeps simulations fast.
pub fn abilities_answered(g: &Game, p: Option<PlayerId>) -> bool {
    if g.humans.any() {
        return true;
    }
    // p's own copier: Peter Parker's Camera can copy its trigger
    if p.is_some_and(|p| camera_ready(g, p) || crate::impls::t12_brago_meren::resonator_ready(g, p)) {
        return true;
    }
    g.players.iter().filter(|q| Some(q.id) != p && q.alive).any(|q| {
        q.hand.iter().any(|&c| {
            let n = &*g.db.get(c).name;
            ABILITY_ANSWERS.contains(&n) || ABILITY_ANSWERS_HAND.contains(&n)
        }) || q
            .perms
            .iter()
            .any(|&m| !g.perm(m).phased && g.perm(m).cd.is_some_and(|c| ABILITY_ANSWERS.contains(&&*g.db.get(c).name)))
    })
}

/// --- card-models-2: the AI copies its own activated ability with the Camera when the ability is worth this much
/// (`dsl::ability_worth`: a tutor is 3)
pub const CAMERA_WORTH: f64 = 3.0;

/// p has an untapped Peter Parker's Camera with a film counter left
pub fn camera_ready(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| {
        let x = g.perm(m);
        x.on_bf
            && !x.phased
            && !x.tapped
            && !x.neutered
            && &*x.name == "Peter Parker's Camera"
            && x.data.int(crate::state::DataKey::Counters) > 0
    })
}

/// the number of copies of trigger item `id` that resolved (they are used up)
fn take_copies(g: &mut Game, id: u32) -> usize {
    let n = g.trig_copies.iter().filter(|&&x| x == id).count();
    g.trig_copies.retain(|&x| x != id);
    n
}

/// a copy of a triggered ability resolves: its hook runs again (no new stack window: it already had one)
fn run_copy(g: &mut Game, t: &Trigger) -> Res {
    let prev = g.trig_current.replace(0); // id 0 is never on the stack: trigger_window just says "resolved"
    let r = call(g, t);
    g.trig_current = prev;
    r
}

/// --- card-models-2: can stack item `it` be copied: a triggered ability the engine can run again (its `Trigger`), or
/// an activated ability of the ability language (`ctx.dsl_ab`, put there by `ability_window_ex`, whose caller resolves
/// the copies)
pub fn copyable(it: &StackItem) -> bool {
    match it.kind {
        StackKind::Trigger => it.ctx.trigger.is_some(),
        StackKind::Ability => it.ctx.dsl_ab.is_some(),
        StackKind::Spell => false,
    }
}

/// q copies the triggered or activated ability of theirs on top of the stack (item `id`, `copyable`): the copy goes
/// on the stack above it, players may respond (counter it), and if it resolves the effect runs once more before the
/// original resolves.
/// True if the copy resolved. The copy cannot be copied again.
pub fn copy_trigger(g: &mut Game, q: PlayerId, id: u32) -> Res<bool> {
    let Some(orig) = item(g, id).cloned() else { return Ok(false) };
    if !copyable(&orig) {
        return Ok(false);
    }
    let mut cp = orig;
    cp.id = g.stack_pushes + 1; // stack_window counts the push
    cp.name = format!("copy of {}", cp.name);
    cp.ctx.trigger = None;
    cp.ctx.dsl_ab = None; // the copy cannot be copied again
    cp.ctx.target = None; // its targets are chosen anew as it resolves
    cp.passed = vec![];
    cp.countered = false;
    cp.controller = q;
    if stack_window(g, q, cp)? {
        g.trig_copies.push(id);
        return Ok(true);
    }
    Ok(false)
}

// ------------------------------------------------------------------ priority
/// p paid the cost of an activated ability of src: it goes on the stack and players get priority. True if it
/// resolves (the caller then applies its effect, even if src is gone by then), false if it was countered.
pub fn ability_window(
    g: &mut Game,
    p: PlayerId,
    src: Option<PermId>,
    name: &str,
    imp: Option<f64>,
    target: Option<PermId>,
) -> Res<bool> {
    let flow = src.and_then(|m| g.perm(m).cd); // --- ai-flow: the ability's draws are its source's
    crate::t6stats::flow_source(g, p, flow);
    // Tier 6: an activation that works on a piece of a modeled line is a win step the table may answer
    if let Some(m) = src
        && crate::impls::combos::normal_play_window(g, p, m)?
    {
        return Ok(false);
    }
    if g.over || !abilities_answered(g, Some(p)) {
        return Ok(true);
    }
    let imp = imp.unwrap_or_else(|| 3.0 + src.filter(|&m| g.perm(m).on_bf).map_or(0.0, |m| 0.5 * pval(g, m)));
    let card = src.and_then(|m| g.perm(m).cd);
    let full = format!("{}: {name}", src_name(g, src));
    let it =
        new_item(g, p, card, Ctx { source: src, target, ..Ctx::default() }, "ability", imp, StackKind::Ability, full);
    stack_window(g, p, it)
}

/// --- card-models-2: how an ability put on the stack by `ability_window_ex` left it
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AbilityRes {
    /// it was not countered: the caller applies its effect
    pub resolves: bool,
    /// its single target as it resolves (changed on the stack by Redirect Lightning), as given if it had none
    pub target: Option<PermId>,
    /// copies of it that resolved before it (Peter Parker's Camera): the caller applies the effect once for each
    /// first, choosing new targets
    pub copies: usize,
}

/// --- card-models-2: `ability_window` for an ability the caller can resolve again and re-aim: `ctx` carries its
/// single `target` and `dsl_ab` (the ability language's abilities: `dsl::activate`, `dsl::use_loyalty`). While it is
/// on the stack its target may be changed and it may be copied; the result says how it left.
pub fn ability_window_ex(
    g: &mut Game,
    p: PlayerId,
    src: Option<PermId>,
    name: &str,
    imp: Option<f64>,
    ctx: Ctx,
) -> Res<AbilityRes> {
    let target = ctx.target;
    let flow = src.and_then(|m| g.perm(m).cd);
    crate::t6stats::flow_source(g, p, flow);
    if let Some(m) = src
        && crate::impls::combos::normal_play_window(g, p, m)?
    {
        return Ok(AbilityRes { resolves: false, target, copies: 0 });
    }
    if g.over || !abilities_answered(g, Some(p)) {
        return Ok(AbilityRes { resolves: true, target, copies: 0 });
    }
    let imp = imp.unwrap_or_else(|| 3.0 + src.filter(|&m| g.perm(m).on_bf).map_or(0.0, |m| 0.5 * pval(g, m)));
    let card = src.and_then(|m| g.perm(m).cd);
    let full = format!("{}: {name}", src_name(g, src));
    let ctx = Ctx { source: src, ..ctx };
    let it = new_item(g, p, card, ctx, "ability", imp, StackKind::Ability, full);
    let out = stack_window_item(g, p, it)?;
    let copies = take_copies(g, out.id);
    Ok(AbilityRes { resolves: !out.countered, target: out.ctx.target, copies })
}

/// ability_window for an ability whose source is a card, not a permanent (a land's ability, a card in hand or the
/// graveyard): Python's `ability_window(g, p, L.cd, ...)`
pub fn ability_window_card(
    g: &mut Game,
    p: PlayerId,
    cd: CardId,
    name: &str,
    imp: Option<f64>,
    target: Option<PermId>,
) -> Res<bool> {
    crate::t6stats::flow_source(g, p, Some(cd)); // --- ai-flow
    if g.over || !abilities_answered(g, Some(p)) {
        return Ok(true);
    }
    let full = format!("{}: {name}", g.db.get(cd).name);
    let ctx = Ctx { target, ..Ctx::default() };
    let it = new_item(g, p, Some(cd), ctx, "ability", imp.unwrap_or(3.0), StackKind::Ability, full);
    stack_window(g, p, it)
}

/// pay equip {n} for equipment e onto m; it attaches if the ability resolves and both are still there
pub fn equip_to(g: &mut Game, p: PlayerId, e: PermId, m: PermId, n: u32) -> Res<bool> {
    pay(g, p, n, "", false)?;
    if crate::impls::t6_copies::image_targeted(g, m)? && !g.perm(m).on_bf {
        return Ok(true); // --- card-models-2: a Phantasmal Image targeted by the equip
    }
    let name = format!("equip to {}", g.perm(m).name);
    if ability_window(g, p, Some(e), &name, None, Some(m))? && g.perm(m).on_bf && g.perm(e).on_bf {
        g.perm_mut(e).attached = Some(m);
    }
    Ok(true)
}

/// Item goes on the stack and every player gets priority, p first then in turn order. A response goes on the stack
/// above it, gets its own round, and resolves when everyone passes on it; then the active player gets priority
/// again. True once everyone passes with the item on top (the caller resolves it), false if it was countered.
pub fn stack_window(g: &mut Game, p: PlayerId, item: StackItem) -> Res<bool> {
    let id = item.id;
    g.stack_pushes = g.stack_pushes.max(id);
    g.stack.push(item);
    let r = (|| {
        if !g.trig_queue.is_empty() && g.resolving == 0 {
            flush_triggers(g)?; // its cast triggers resolve first
        }
        priority(g, id, p)
    })();
    let countered = item_countered(g, id);
    remove_item(g, id);
    r?;
    Ok(!countered)
}

fn item_countered(g: &Game, id: u32) -> bool {
    item(g, id).is_some_and(|it| it.countered)
}

fn priority(g: &mut Game, id: u32, first: PlayerId) -> Res {
    let (mut start, mut rounds) = (first, 0);
    while !g.over && on_stack(g, id) && !item_countered(g, id) {
        let order: Vec<PlayerId> =
            std::iter::once(start).chain(g.after(start)).filter(|&q| g.player(q).alive).collect();
        let (mut passes, mut k, mut acted) = (0, 0, false);
        while passes < order.len() && !g.over {
            let q = order[k % order.len()];
            if take_priority(g, q, id)? {
                acted = true; // a response resolved (or was countered) above the item
                break;
            }
            passes += 1;
            k += 1;
        }
        if !acted {
            return Ok(()); // everyone passed in succession: the item resolves
        }
        rounds += 1;
        if rounds > 12 {
            return Ok(()); // a runaway exchange of responses
        }
        start = g.active.filter(|&a| g.player(a).alive).unwrap_or(first);
    }
    Ok(())
}

/// q has priority with the item on top of the stack; true if q put something on the stack (it has resolved by now)
fn take_priority(g: &mut Game, q: PlayerId, _id: u32) -> Res<bool> {
    if !g.player(q).alive
        || g.over
        || crate::engine::values::silenced(g, q)
        || (g.locks_used && crate::engine::cast::split_second_up(g))
        || g.plan_for.is_some_and(|x| x != q) // --- ai-planner: a planner's copy: only its player acts
    {
        return Ok(false);
    }
    let n = g.stack_pushes;
    if crate::human::is_human(g, q) {
        crate::human::human::stack_priority(g, q)?;
    } else {
        ai_respond(g, q)?;
    }
    Ok(g.stack_pushes != n)
}

/// a spell cast by its own card code (not cast_card): it goes on the stack with a round of priority. True if it
/// resolves (the caller carries out its effect), false if it was countered.
pub fn counter_window(g: &mut Game, p: PlayerId, c: CardId, imp: f64, aff: Vec<(PlayerId, f64)>) -> Res<bool> {
    if g.db.get(c).tag(Tag::Unc) || crate::engine::hooks::any_uncounterable(g, p, c) || mistrised(g, c) {
        return Ok(true);
    }
    let name = g.db.get(c).name.to_string();
    let mut it = new_item(g, p, Some(c), Ctx::default(), "hand", imp, StackKind::Spell, name);
    it.aff = aff;
    let id = it.id;
    // remember who countered it (Python's LAST_COUNTER)
    let ok = stack_window_keep(g, p, it)?;
    if !ok.0 {
        g.last_counter = ok.1;
    }
    let _ = id;
    Ok(ok.0)
}

/// stack_window, also returning the card that countered the item
pub fn stack_window_keep(g: &mut Game, p: PlayerId, new: StackItem) -> Res<(bool, Option<CardId>)> {
    let it = stack_window_item(g, p, new)?;
    Ok((!it.countered, it.countered_by))
}

/// stack_window, returning the item as it left the stack: its controller and ctx may have changed (a stolen spell,
/// a new target), and `cast_card` resolves the spell with them
pub fn stack_window_item(g: &mut Game, p: PlayerId, new: StackItem) -> Res<StackItem> {
    let id = new.id;
    g.stack_pushes = g.stack_pushes.max(id);
    g.stack.push(new);
    let r = (|| {
        if !g.trig_queue.is_empty() && g.resolving == 0 {
            flush_triggers(g)?;
        }
        priority(g, id, p)
    })();
    let it = item(g, id).cloned();
    remove_item(g, id);
    r?;
    Ok(it.expect("the item is still on the stack until stack_window takes it off"))
}

pub fn mistrised(g: &Game, c: CardId) -> bool {
    g.unc_cast == Some((c, g.turn_stamp()))
}

pub fn counterable(g: &Game, it: &StackItem) -> bool {
    let Some(c) = it.card else { return true };
    !(g.db.get(c).tag(Tag::Unc) || crate::engine::hooks::any_uncounterable(g, it.controller, c) || mistrised(g, c))
}

/// The AI with priority: counter the spell on top of the stack (or a counterspell aimed at its own spell), or pass.
/// Python's `ai_respond`.
pub fn ai_respond(g: &mut Game, q: PlayerId) -> Res {
    let Some(top) = g.stack.last() else { return Ok(()) };
    if top.controller == q
        && top.kind == StackKind::Trigger
        && (top.imp >= 6.0 || crate::impls::t12_brago_meren::resonator_wants(g, q, top))
        && top.ctx.trigger.is_some()
        && !top.passed.contains(&q)
    {
        // q's own important trigger: copy it (Peter Parker's Camera; --- Tiers 1/2 (brago-meren): Strionic Resonator)
        let top_id = top.id;
        g.stack.last_mut().unwrap().passed.push(q);
        crate::cardcode::copy_ability(g, q, top_id)?;
        return Ok(());
    }
    if top.controller == q
        && top.kind == StackKind::Ability
        && !top.passed.contains(&q)
        && let Some((cd, idx)) = top.ctx.dsl_ab
        && camera_ready(g, q)
        && crate::dsl::ability_worth(g, q, cd, idx) >= CAMERA_WORTH
    {
        // --- card-models-2: q's own valuable activated ability (a tutor ...): the Camera copies it
        let top_id = top.id;
        g.stack.last_mut().unwrap().passed.push(q);
        crate::impls::t6_copies::camera_copy(g, q, top_id)?;
        return Ok(());
    }
    if top.controller == q || top.passed.contains(&q) {
        return Ok(());
    }
    let top_id = top.id;
    g.stack.last_mut().unwrap().passed.push(q);
    let top = g.stack.last().unwrap().clone();
    if top.kind != StackKind::Spell {
        // an ability (or trigger): Azorius Guildmage, Tishana's Tidebinder
        // --- card-models-2: an ability aimed at q's permanent: Redirect Lightning turns it on its controller's own
        if top.ctx.target.is_some_and(|t| g.perm(t).owner == q)
            && crate::impls::t6_redirects::redirect_ability(g, q, top_id)?
        {
            return Ok(());
        }
        if top.imp >= 6.0 {
            crate::cardcode::answer_ability(g, q, top_id)?;
        }
        return Ok(());
    }
    let hand = g.player(q).hand.clone();
    if top.generic
        && hand.iter().any(|&x| crate::impls::t6_counters::STACK_RESPONSES.contains(&&*g.db.get(x).name))
        && crate::impls::t6_counters::stack_response(g, q, top_id)?
    {
        return Ok(()); // Misdirection, Commandeer, Sudden Substitution
    }
    // --- card-models-2: a counterspell aimed at q's important spell: Redirect Lightning turns it on another spell
    if top.ctx.counter.is_some() && crate::impls::t6_redirects::redirect_counter(g, q, top_id)? {
        return Ok(());
    }
    if !hand.iter().any(|&x| g.db.get(x).tag(Tag::Ctr) || &*g.db.get(x).name == "Venser, Shaper Savant") {
        return Ok(()); // nothing to respond with
    }
    if !counterable(g, &top) {
        return Ok(());
    }
    let c = top.card.unwrap();
    if let Some(target) = top.ctx.counter {
        // a counterspell: answer it only to save your own spell
        let Some(t) = item(g, target) else { return Ok(()) };
        if t.controller != q || t.imp < 6.0 {
            return Ok(());
        }
        if let Some(ctr) = pick_counter(g, q, c) {
            cast_counter_spell(g, q, ctr, top_id)?;
        }
        return Ok(());
    }
    let mut val = top.value_to(q);
    if val <= 0.0 && top.aff.is_empty() {
        return Ok(());
    }
    let nc = hand.iter().filter(|&&x| g.db.get(x).tag(Tag::Ctr)).count();
    if hand_key(g, q) == "veyran" && has(g, q, Tag::Veyran) {
        val += 1.5; // every counter is also a doubled magecraft trigger
    }
    let key = g.player(q).key;
    let mut thr = match g.settings.counter_threshold(key) {
        Some(t) => t as f64,
        None => ai::counter_threshold(g, q, g.settings.cthresh_default as f64),
    };
    if nc == 0 {
        return Ok(());
    }
    let pk = pick_counter(g, q, c);
    if let Some(pk) = pk {
        val *= soft_counter_factor(g, q, top.controller, pk);
    }
    if let Some(pk) = pk {
        // Force of Will pitched (no mana for it): two cards for one
        let (cg, cp) = counter_cost(g, pk, Some(c), Some(q));
        if g.db.get(pk).tag(Tag::Free) && !can_pay(g, q, cg, &cp, false) {
            thr += 1.5;
        } else if altcost::use_alt(g, q, pk, Some(c), cg, &cp) {
            thr += 1.5 * altcost::lost(g, q, pk, Some(c)); // cards given up for the alternative cost
        }
    }
    // Tier 6: the table rules decide (archenemy, win steps), not the look-ahead
    if crate::impls::t6_plans::is_t6(g, q) {
        use ai::t6_table::Verdict;
        match ai::t6_table::counter_verdict(g, q, &top, val, thr, nc) {
            Verdict::Counter => {}
            Verdict::Pass => return Ok(()),
            Verdict::Roll(v, t) => {
                if g.rng.random() > ai::wants_counter(v, t, nc) {
                    return Ok(());
                }
            }
        }
    } else {
        // the look-ahead decides counters to the active player's spell in their main phase
        match ai::search_counter(g, q, top.controller, c, top.generic) {
            Some(false) => return Ok(()),
            Some(true) => {}
            None => {
                if g.rng.random() > ai::wants_counter(val, thr, nc) {
                    return Ok(());
                }
            }
        }
    }
    if top.imp >= 6.0 && crate::cardcode::hullbreaker_counter(g, q, c)? {
        g.bounced_spell = true;
        if let Some(it) = g.stack.iter_mut().find(|it| it.id == top_id) {
            it.countered = true;
        }
        return Ok(());
    }
    if let Some(ctr) = pick_counter(g, q, c) {
        cast_counter_spell(g, q, ctr, top_id)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ counterspells
/// can counterspell ctr counter spell c (its `ctr` tag's restriction)
pub fn counter_ok(g: &Game, ctr: CardId, c: CardId) -> bool {
    let d = g.db.get(c);
    match g.db.get(ctr).tags.str(Tag::Ctr) {
        Some("any") => true,
        Some("nc") => !d.creature,
        Some("ise") => d.instant || d.sorcery || (d.types.has(crate::cards::Types::ENCHANTMENT) && !d.creature),
        Some("is") => d.instant || d.sorcery, // Muddle the Mixture
        Some("mv4") => d.cmc >= 4,
        Some("cre") => d.creature,
        Some("mv1") => d.cmc == 1,                                // Mental Misstep
        Some("cre2") => d.creature && (d.pow <= 2 || d.tgh <= 2), // Stern Scolding
        Some("blue") => d.pips.contains('U'),                     // Pyroblast / Red Elemental Blast
        Some("crleg") => d.creature || d.tag(Tag::Leg),           // Icy Reception: creature or legendary spell
        // Disrupting Shoal counters a spell only if its mana value is X. The AI always chooses X equal to the spell's
        // mana value (`counter_cost`; the pitch needs a blue card of that value, `t6_counters::shoal_pitch`).
        Some("mvx") => true,
        _ => false,
    }
}

/// (generic, pips) a counterspell costs against this spell (Brush Off: {1}{U} against an instant or sorcery;
/// Wizard's Retort: {1} less while q controls a Wizard)
pub fn counter_cost(g: &Game, ctr: CardId, spell: Option<CardId>, q: Option<PlayerId>) -> (u32, String) {
    let d = g.db.get(ctr);
    if d.tag(Tag::Brushoff) && spell.is_some_and(|s| g.db.get(s).instant || g.db.get(s).sorcery) {
        return (1, "U".into());
    }
    if &*d.name == "Disrupting Shoal"
        && let Some(s) = spell
    {
        return (g.db.get(s).cmc, d.pips.to_string()); // {X}{U}{U}, X = the spell's mana value
    }
    if &*d.name == "Louisoix's Sacrifice" {
        return (d.generic + 2, d.pips.to_string()); // the additional cost: pay {2} (or sacrifice a legendary creature)
    }
    if &*d.name == "Phantom Interference" {
        return (1, "U".into()); // spree: the counter mode costs {1} more (the Spirit mode: bulk_blue::spree_extra)
    }
    if &*d.name == "Wizard's Retort"
        && let Some(q) = q
        && g.player(q).perms.iter().any(|&m| g.is_creature(m) && !g.perm(m).phased && has_type(g, m, "wizard"))
    {
        return (d.generic.saturating_sub(1), d.pips.to_string());
    }
    (d.generic, d.pips.to_string())
}

fn blue_in_hand(g: &Game, q: PlayerId, except: CardId) -> Vec<CardId> {
    g.player(q).hand.iter().copied().filter(|&x| x != except && g.db.get(x).pips.contains('U')).collect()
}

/// Pact of Negation is only free if {3}{U}{U} (for each pact still owed) can be paid at the next upkeep. Tier 0-5: five
/// lands, as always. Tier 6: also two lands that make blue, and five lands per pact (a lost game to a missed upkeep
/// was the single biggest leak of the cEDH decks that run it).
fn pact_affordable(g: &Game, q: PlayerId) -> bool {
    let lands = g.player(q).lands.len();
    if !crate::impls::t6_plans::is_t6(g, q) {
        return lands >= 5;
    }
    let blue = g.player(q).lands.iter().filter(|&&l| crate::engine::mana::land_colors_now(g, q, l).has('U')).count();
    lands >= 5 * (1 + g.player(q).pacts as usize) && blue >= 2
}

/// the counterspell q would use against c, if any
pub fn pick_counter(g: &Game, q: PlayerId, c: CardId) -> Option<CardId> {
    let mut best: Option<CardId> = None;
    let mut loose = false; // best is a pitch (Force of Will) or alternative-cost pick: any counter paid in mana beats it
    let cheaper = |best: Option<CardId>, ctr: CardId| best.is_none_or(|b| g.db.get(ctr).cmc < g.db.get(b).cmc);
    for &ctr in &g.player(q).hand {
        let d = g.db.get(ctr);
        let t = &d.tags;
        if !crate::ai::rules::followup_ok(g, q, ctr) {
            continue; // --- phase4-rules: a Pact whose upkeep (with every debt owed) can't be paid
        }
        if &*d.name == "Venser, Shaper Savant"
            && g.settings.counter_threshold(g.player(q).key).is_none()
            && !t.has(Tag::Ctr)
        {
            let (cg, cp) = counter_cost(g, ctr, Some(c), Some(q));
            if can_pay(g, q, cg, &cp, false) && castable(g, q, ctr, "hand") && cheaper(best, ctr) {
                best = Some(ctr);
            }
            continue;
        }
        if !t.has(Tag::Ctr) || !counter_ok(g, ctr, c) {
            continue;
        }
        if g.cast_checks() && !castable(g, q, ctr, "hand") {
            continue;
        }
        // the free modes pay the cost increases (Defense Grid) in mana: `altcost::tax`, 0 when nothing taxes spells
        let taxed_ok = altcost::tax_payable(g, q, ctr);
        if t.has(Tag::Fierce) && commander_out(g, q) && taxed_ok {
            return Some(ctr); // Fierce Guardianship: free with your commander out
        }
        if t.has(Tag::Pact) && pact_affordable(g, q) && taxed_ok {
            return Some(ctr); // Pact of Negation: free now, {3}{U}{U} next upkeep
        }
        if t.has(Tag::Pact) && crate::impls::t6_plans::is_t6(g, q) {
            continue; // Tier 6: a Pact whose upkeep can't be paid is not cast (the {0} cost fell through to `can_pay` below)
        }
        if t.has(Tag::Misstep) && g.player(q).life > 10 && taxed_ok {
            return Some(ctr); // Mental Misstep: 2 life
        }
        if t.has(Tag::Fon) && g.active != Some(q) && !blue_in_hand(g, q, ctr).is_empty() && taxed_ok {
            return Some(ctr); // Force of Negation: free on others' turns
        }
        let (cg, cp) = counter_cost(g, ctr, Some(c), Some(q));
        if t.has(Tag::Free) {
            if ((!blue_in_hand(g, q, ctr).is_empty() && taxed_ok) || can_pay(g, q, cg, &cp, false)) && best.is_none() {
                best = Some(ctr);
                loose = true;
            }
            continue;
        }
        if altcost::use_alt(g, q, ctr, Some(c), cg, &cp) {
            // an alternative cost (Daze, Flare of Denial, Disrupting Shoal, Mindbreak Trap): taken at once when it gives
            // up nothing, else (the mana cost can't be paid) like a Force of Will pitch
            if altcost::lost(g, q, ctr, Some(c)) <= 0.0 {
                return Some(ctr);
            }
            if best.is_none() {
                best = Some(ctr);
                loose = true;
            }
            continue;
        }
        if can_pay(g, q, cg, &cp, false) && (cheaper(best, ctr) || loose) {
            best = Some(ctr);
            loose = false;
        }
    }
    best
}

/// counterspell c costs q no mana right now: Force of Will-style pitch counters, or Fierce Guardianship (and kin)
/// while q controls its commander
pub fn free_counter(g: &Game, q: PlayerId, c: CardId) -> bool {
    let t = &g.db.get(c).tags;
    t.has(Tag::Free)
        || (t.has(Tag::Fierce) && commander_out(g, q))
        || (altcost::slot(g, c).is_some()
            && altcost::tax(g, q, c) == 0
            && altcost::lost(g, q, c, None) <= 0.0
            && altcost::available(g, q, c, None)) // Mindbreak Trap's free mode
}

/// pay counterspell ctr's cost (or its alternative cost) against spell; true if paid
pub fn pay_counter(g: &mut Game, q: PlayerId, ctr: CardId, spell: Option<CardId>) -> Res<bool> {
    if g.free_counter {
        return Ok(true); // practice mode: paid from the person's pool
    }
    let t = g.db.get(ctr).tags.clone();
    // a cast without paying the mana cost still pays the cost increases (Defense Grid): `altcost::tax`
    let tax_free_cast = (t.has(Tag::Fierce) && commander_out(g, q))
        || t.has(Tag::Pact)
        || (t.has(Tag::Misstep) && g.player(q).life > 10)
        || (t.has(Tag::Fon) && g.active != Some(q) && !blue_in_hand(g, q, ctr).is_empty());
    if tax_free_cast && !altcost::pay_tax(g, q, ctr)? {
        return Ok(false);
    }
    if t.has(Tag::Fierce) && commander_out(g, q) {
        return Ok(true); // cast without paying its mana cost
    }
    if t.has(Tag::Pact) {
        g.player_mut(q).pacts += 1; // pay {3}{U}{U} at the next upkeep or lose
        return Ok(true);
    }
    if t.has(Tag::Misstep) && g.player(q).life > 10 {
        lose_life(g, q, 2, Some(q), "other", None)?;
        return Ok(true);
    }
    let pitch = |g: &mut Game, blues: Vec<CardId>| -> Res {
        // the least useful blue card (a person picks: play/cards.pick_blue)
        let sauron = hand_key(g, q) == "sauron";
        let mut x = crate::engine::zones::min_by(&blues, |c| {
            let w = crate::engine::tutors::card_worth(g, q, c, false);
            if sauron { w + crate::ai::decks::sauron_pitch_keep(g, q, c) } else { w }
        })
        .unwrap();
        if crate::human::is_human(g, q) {
            let what = g.db.get(ctr).name.to_string();
            x = crate::human::cards::pick_blue(g, q, &blues, &what)?;
        }
        let pl = g.player_mut(q);
        let i = pl.hand.iter().position(|&c| c == x).unwrap();
        pl.hand.remove(i);
        pl.exile.push(x);
        Ok(())
    };
    if t.has(Tag::Fon) && g.active != Some(q) && !blue_in_hand(g, q, ctr).is_empty() {
        let b = blue_in_hand(g, q, ctr);
        pitch(g, b)?;
        return Ok(true);
    }
    let (cg, cp) = counter_cost(g, ctr, spell, Some(q));
    if t.has(Tag::Free) && !can_pay(g, q, cg, &cp, false) {
        let blues = blue_in_hand(g, q, ctr);
        if blues.is_empty() || !altcost::pay_tax(g, q, ctr)? {
            return Ok(false);
        }
        pitch(g, blues)?;
        lose_life(g, q, 1, Some(q), "other", None)?;
        return Ok(true);
    }
    if altcost::use_alt(g, q, ctr, spell, cg, &cp) {
        return altcost::pay_alt(g, q, ctr, spell); // Daze, Flare of Denial, Disrupting Shoal, Mindbreak Trap
    }
    pay(g, q, cg, &cp, false)
}

/// q casts counterspell ctr at stack item `target`. It goes on the stack (players may respond, the target's caster
/// can counter it back), and on resolution counters the target if it's still there. True if it was cast.
pub fn cast_counter_spell(g: &mut Game, q: PlayerId, ctr: CardId, target: u32) -> Res<bool> {
    let Some(t) = item(g, target).cloned() else { return Ok(false) };
    if !g.player(q).hand.contains(&ctr) {
        return Ok(false);
    }
    if crate::cardcode::veil_response(g, t.controller, q, ctr)? {
        return Ok(false);
    }
    if !pay_counter(g, q, ctr, t.card)? {
        return Ok(false);
    }
    let spree = crate::impls::bulk_blue::spree_extra(g, q, ctr)?; // Phantom Interference: the Spirit mode too
    let pl = g.player_mut(q);
    let i = pl.hand.iter().position(|&c| c == ctr).unwrap();
    pl.hand.remove(i);
    pl.spells_this_turn += 1;
    pl.stat("counters_cast", 1);
    pl.cast_names.insert(ctr);
    crate::glog!(
        g,
        "    {} casts {} at {}",
        g.player(q).name,
        g.db.get(ctr).name,
        t.card.map_or("an ability".into(), |c| g.db.get(c).name.to_string())
    );
    on_cast(g, q, ctr)?;
    let name = g.db.get(ctr).name.to_string();
    let it = new_item(
        g,
        q,
        Some(ctr),
        Ctx { counter: Some(target), ..Ctx::default() },
        "hand",
        t.imp.max(6.0),
        StackKind::Spell,
        name,
    );
    let out = stack_window_item(g, q, it)?; // --- card-models-2: its target may have been changed (Redirect Lightning)
    if !out.countered {
        if spree {
            crate::impls::bulk_blue::spirit_token(g, q)?;
        }
        resolve_counter(g, q, ctr, out.ctx.counter.unwrap_or(target))?;
        if g.db.get(ctr).creature {
            // Mystic Snake / Venser: the counter is a creature
            g.skip_etb = &*g.db.get(ctr).name == "Venser, Shaper Savant";
            let r = enter(g, q, ctr, Enter { was_cast: true, ..Enter::default() });
            g.skip_etb = false;
            r?;
        } else {
            g.player_mut(q).gy.push(ctr);
        }
    } else {
        g.player_mut(q).gy.push(ctr);
    }
    Ok(true)
}

/// counterspell ctr resolves: counter the target if it's still on the stack (unless its controller pays for a soft
/// counter)
pub fn resolve_counter(g: &mut Game, q: PlayerId, ctr: CardId, target: u32) -> Res {
    let Some(t) = item(g, target).cloned() else { return Ok(()) }; // gone: the counterspell does nothing
    if t.countered {
        return Ok(());
    }
    let (p, c) = (t.controller, t.card.unwrap());
    let d = g.db.get(ctr);
    let mut soft = d.tags.int(Tag::Soft).unwrap_or(0);
    if &*d.name == "Flusterstorm" {
        // storm: a copy per spell cast before it this turn
        soft = g.players.iter().filter(|x| x.alive).map(|x| crate::engine::cast::casts_this_turn(g, x.id)).sum::<i32>()
            - 1;
    }
    if soft > 0 && can_pay(g, p, soft as u32, "", false) {
        // Spell Pierce / Mystic Confluence: pay and it resolves (a person may decline)
        let ctr_name = g.db.get(ctr).name.to_string();
        let paid = if crate::human::is_human(g, p) {
            crate::human::choices::pay_tax(g, p, soft as u32, &ctr_name)?
        } else {
            pay(g, p, soft as u32, "", false)?;
            true
        };
        if paid {
            crate::glog!(g, "    {} pays {}", g.player(p).name, soft);
            return Ok(());
        }
    }
    crate::glog!(g, "    {} counters {} with {}", g.player(q).name, g.db.get(c).name, g.db.get(ctr).name);
    counter_side_effects(g, q, p, ctr)?;
    if &*g.db.get(ctr).name == "Mana Drain" {
        g.player_mut(q).drain_mana += g.db.get(c).cmc;
    }
    g.player_mut(p).stat("spells_countered", 1);
    g.last_counter = Some(ctr);
    crate::t6stats::counter_hit(g, q, ctr, p);
    if let Some(inner) = t.ctx.counter
        && g.stack.iter().any(|x| x.id == inner && x.controller == q)
    {
        g.player_mut(q).stat("counterwar_won", 1); // q countered the counterspell aimed at q's spell
    }
    let desert = &*g.db.get(ctr).name == "Desertion" && {
        let cd = g.db.get(c);
        !cd.land && (cd.creature || cd.types.has(crate::cards::Types::ARTIFACT))
    };
    if let Some(it) = g.stack.iter_mut().find(|x| x.id == target) {
        it.countered = true;
        it.countered_by = Some(ctr);
        if desert {
            it.ctx.desert_to = Some(q); // Desertion: the countered artifact or creature is q's
        }
    }
    if &*g.db.get(ctr).name == "Mindbreak Trap" {
        crate::impls::t6_counters::mindbreak_exile(g, q)?; // "exile any number of target spells": all the others
    }
    if &*g.db.get(ctr).name == "Tibalt's Trickery" {
        crate::impls::t6_counters::tibalt_after(g, q, p, c)?; // its controller mills, then casts a card free
    }
    Ok(())
}

/// The look-ahead: finish a copied game's stack at once (no more responses), top first, then the triggers waiting
/// to go on it. Python's `settle_stack`.
pub fn settle_stack(g: &mut Game) -> Res {
    while !g.over {
        let Some(it) = g.stack.pop() else { break };
        let p = it.controller;
        if it.countered {
            if it.kind != StackKind::Spell {
                continue;
            }
            let c = it.card.unwrap();
            let o = it.ctx.owner.unwrap_or(p); // a card cast from another player's deck goes back to theirs
            if g.player(p).is_commander(c) {
                g.player_mut(p).set_cmd_zone(c, true);
            } else if it.zone == "gy" || it.ctx.exile_after {
                g.player_mut(o).exile.push(c);
            } else if !g.db.get(c).land {
                g.player_mut(o).gy.push(c);
            }
            continue;
        }
        if it.kind == StackKind::Trigger
            && let Some(t) = &it.ctx.trigger
        {
            call(g, t)?;
            crate::engine::life::check_state(g)?;
            continue;
        }
        if it.kind != StackKind::Spell {
            continue; // an ability: its effect lives with the code that activated it
        }
        let c = it.card.unwrap();
        if let Some(tgt) = it.ctx.counter {
            if let Some(x) = g.stack.iter_mut().find(|x| x.id == tgt) {
                x.countered = true;
                x.countered_by = Some(c);
            }
            g.player_mut(it.ctx.owner.unwrap_or(p)).gy.push(c);
            continue;
        }
        if g.db.get(c).perm && !it.generic {
            enter(g, p, c, Enter { was_cast: true, ..Enter::default() })?;
            continue;
        }
        crate::engine::cast::resolve(g, p, c, &it.ctx, it.zone)?;
        crate::engine::life::check_state(g)?;
    }
    if !g.trig_queue.is_empty() && !g.over {
        flush_triggers(g)?;
    }
    Ok(())
}

/// q countered p's spell with ctr: the counterspell's side effects
pub fn counter_side_effects(g: &mut Game, q: PlayerId, p: PlayerId, ctr: CardId) -> Res {
    let d = g.db.get(ctr);
    let t = d.tags.clone();
    let name = d.name.to_string();
    if t.has(Tag::Offer) {
        add_treasure(g, p, 2)?; // An Offer You Can't Refuse
    }
    if t.has(Tag::Denial) {
        // Arcane Denial (at the next upkeep)
        g.player_mut(p).delayed_draws += 2;
        g.player_mut(q).delayed_draws += 1;
    }
    if t.has(Tag::Undermine) {
        lose_life(g, p, 3, Some(q), "triggers", Some(false))?; // life loss, not damage
    }
    if t.has(Tag::Swan) {
        make_tokens(g, p, Tokens { fly: true, ..Tokens::new(1, 2) })?; // Swan Song gives the caster a Bird
    }
    if name == "Absorb" {
        gain(g, q, 3)?;
    }
    if name == "Amazing Acrobatics" {
        crate::impls::bulk_blue::acrobatics_tap(g, q)?; // its other mode: tap one or two creatures
    }
    if name == "Dismiss" {
        draw(g, q, 1, false)?; // Dismiss: counter target spell. Draw a card.
    }
    if name == "Statute of Denial"
        && g.player(q).perms.iter().any(|&m| g.is_creature(m) && !g.perm(m).phased && colors_of(g, m).has('U'))
    {
        draw(g, q, 1, false)?; // a blue creature: draw a card, then discard a card
        if crate::human::is_human(g, q) {
            crate::human::choices::discard(g, q, 1, "Discard a card")?;
        } else {
            discard_worst(g, q, 1)?;
        }
    }
    Ok(())
}

/// a counterspell cast and resolved at once, outside the stack (combo interruption, the look-ahead's copies)
pub fn cast_counter(g: &mut Game, q: PlayerId, ctr: CardId, spell: Option<CardId>) -> Res<bool> {
    if !pay_counter(g, q, ctr, spell)? {
        return Ok(false);
    }
    let pl = g.player_mut(q);
    let i = pl.hand.iter().position(|&c| c == ctr).unwrap();
    pl.hand.remove(i);
    if g.db.get(ctr).creature {
        // Mystic Snake / Venser: the counter is a creature
        g.skip_etb = &*g.db.get(ctr).name == "Venser, Shaper Savant";
        let r = enter(g, q, ctr, Enter { was_cast: true, ..Enter::default() });
        g.skip_etb = false;
        r?;
    } else {
        g.player_mut(q).gy.push(ctr);
    }
    g.player_mut(q).spells_this_turn += 1;
    on_cast(g, q, ctr)?;
    let pl = g.player_mut(q);
    pl.stat("counters_cast", 1);
    pl.cast_names.insert(ctr);
    Ok(true)
}

/// Priority in a step of the turn, to each player in turn order from the active player. The AI acts here when it's
/// attacked (instant removal on an attacker); its other instant-speed plays keep their own timing. Python's
/// `step_priority`.
pub fn step_priority(g: &mut Game, step: &str, defender: Option<PlayerId>, attackers: &[PermId]) -> Res {
    let Some(a) = g.active else { return Ok(()) };
    if g.over {
        return Ok(());
    }
    let order: Vec<PlayerId> = std::iter::once(a).chain(g.after(a)).collect();
    for q in order {
        if !g.player(q).alive || g.over {
            continue;
        }
        if crate::human::is_human(g, q) {
            crate::human::human::step_priority(g, q, step, defender, attackers)?;
        } else if step == "attackers" && Some(q) == defender {
            ai::attack_response(g, q, a, attackers)?;
        } else if step == "combat" && q != a && hand_key(g, q) == "galadriel" {
            crate::cardcode::galadriel_precombat(g, q)?; // tap the attacker it fears (Errant Doomsayers, Whipcorder)
        }
        if step == "combat" && q != a && !g.hooks.is_empty() && !crate::human::is_human(g, q) {
            crate::cardcode::opposition_precombat(g, q)?; // Opposition: tap the attackers down
        }
    }
    Ok(())
}

/// Kefka's Mana Leak (a soft counter): into open mana the opponent just pays, so the counter is worth only the chance
/// they cannot. 1.0 for everything else (other decks keep the old behaviour).
pub fn soft_counter_factor(g: &Game, q: PlayerId, caster: PlayerId, ctr: CardId) -> f64 {
    let soft = g.db.get(ctr).tags.int(Tag::Soft).unwrap_or(0);
    if hand_key(g, q) == "kefka" && soft > 0 && crate::engine::mana::total_mana(g, caster, false) >= soft as u32 {
        0.3
    } else {
        1.0
    }
}
