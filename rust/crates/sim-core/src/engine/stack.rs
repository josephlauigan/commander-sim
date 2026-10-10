//! Triggered abilities, the stack, priority and counterspells (engine.py: queue_triggers ... counter_side_effects).
//!
//! Triggers wait in `g.trig_queue` until whatever is resolving finishes, then go on the stack in APNAP order. A
//! card's hook is probed first (run in probe mode until its `trigger_window`, which stops it with `Stop::Probe`) to
//! find out whether it really triggers, then pushed, then run for real top first: its `trigger_window` gives every
//! player priority on it and says whether it was countered.

use crate::ai;
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
fn apnap(g: &Game, mut ts: Vec<Trigger>) -> Vec<Trigger> {
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
    if !abilities_answered(g, None) {
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
    // HUMAN(phase 9): the person orders their own simultaneous triggers (_order_triggers)
    let mut items = vec![];
    for t in real {
        let id = next_id(g);
        let card = t.src.and_then(|m| g.perm(m).cd);
        let name = format!("{}: {}", src_name(g, t.src), t.name.unwrap_or("trigger"));
        g.stack.push(StackItem {
            id,
            controller: t.controller,
            card,
            ctx: Ctx { source: t.src, trigger: Some(Box::new(t.clone())), ..Ctx::default() },
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
            let countered = item(g, id).is_some_and(|it| it.countered);
            remove_item(g, id);
            if !countered {
                call(g, &t)?;
            }
            continue;
        }
        let prev = g.trig_current.replace(id);
        let r = call(g, &t); // its trigger_window takes the item: priority, then it resolves
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
    if let Some(cur) = g.trig_current.take() {
        if on_stack(g, cur) {
            let first = g.active.unwrap_or(p);
            priority(g, cur, first)?;
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

/// trigger_window for a trigger whose source is a card, not a permanent (a land's enter trigger)
pub fn trigger_window_card(g: &mut Game, p: PlayerId, cd: CardId, name: &str) -> Res<bool> {
    if g.trig_probe {
        g.trig_probe = false;
        return Err(Stop::Probe(Probe { controller: p, src: None, name: name.into(), imp: 3.0 }));
    }
    if g.trig_current.is_some() || g.over || !abilities_answered(g, Some(p)) {
        return trigger_window(g, p, None, name, None);
    }
    let full = format!("{}: {name}", g.db.get(cd).name);
    let it = new_item(g, p, Some(cd), Ctx::default(), "trigger", 3.0, StackKind::Trigger, full);
    stack_window(g, p, it)
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

/// cards the AI uses to counter abilities
pub const ABILITY_ANSWERS: [&str; 2] = ["Tishana's Tidebinder", "Azorius Guildmage"];

/// Could anyone answer an ability of p's (a person at the table, or an opponent with an ability counter)? When nobody
/// could, an ability resolves without a round of priority, which keeps simulations fast.
pub fn abilities_answered(g: &Game, p: Option<PlayerId>) -> bool {
    if g.humans.any() {
        return true;
    }
    g.players.iter().filter(|q| Some(q.id) != p && q.alive).any(|q| {
        q.hand.iter().any(|&c| ABILITY_ANSWERS.contains(&&*g.db.get(c).name))
            || q.perms.iter().any(|&m| {
                !g.perm(m).phased && g.perm(m).cd.is_some_and(|c| ABILITY_ANSWERS.contains(&&*g.db.get(c).name))
            })
    })
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

/// pay equip {n} for equipment e onto m; it attaches if the ability resolves and both are still there
pub fn equip_to(g: &mut Game, p: PlayerId, e: PermId, m: PermId, n: u32) -> Res<bool> {
    pay(g, p, n, "", false)?;
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
    if !g.player(q).alive || g.over || crate::engine::values::silenced(g, q) {
        return Ok(false);
    }
    let n = g.stack_pushes;
    // HUMAN(phase 9): play/human.stack_priority for a person's seat
    ai_respond(g, q)?;
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
    let it = it.expect("the item is still on the stack until stack_window takes it off");
    Ok((!it.countered, it.countered_by))
}

pub fn mistrised(g: &Game, c: CardId) -> bool {
    g.unc_cast == Some((c, g.turn_stamp()))
}

pub fn counterable(g: &Game, it: &StackItem) -> bool {
    let Some(c) = it.card else { return true };
    !(g.db.get(c).tag(Tag::Unc) || crate::engine::hooks::any_uncounterable(g, it.controller, c) || mistrised(g, c))
}

/// The AI with priority: counter the spell on top of the stack (or a counterspell aimed at its own spell), or pass.
/// Python's `ai_respond` (the adaptive AI's branch; the look-ahead's choice comes in M4).
pub fn ai_respond(g: &mut Game, q: PlayerId) -> Res {
    let Some(top) = g.stack.last() else { return Ok(()) };
    if top.controller == q || top.passed.contains(&q) {
        return Ok(());
    }
    let top_id = top.id;
    g.stack.last_mut().unwrap().passed.push(q);
    let top = g.stack.last().unwrap().clone();
    if top.kind != StackKind::Spell {
        // an ability (or trigger): Azorius Guildmage, Tishana's Tidebinder
        if top.imp >= 6.0 {
            crate::cardcode::answer_ability(g, q, top_id)?;
        }
        return Ok(());
    }
    let hand = g.player(q).hand.clone();
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
    if g.player(q).key == "veyran" && has(g, q, Tag::Veyran) {
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
        // Force of Will pitched (no mana for it): two cards for one
        let (cg, cp) = counter_cost(g, pk, Some(c), Some(q));
        if g.db.get(pk).tag(Tag::Free) && !can_pay(g, q, cg, &cp, false) {
            thr += 1.5;
        }
    }
    // PORT(M4): the look-ahead decides counters in the active player's main phase (search.choose_counter)
    if g.rng.random() > ai::wants_counter(val, thr, nc) {
        return Ok(());
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

/// the counterspell q would use against c, if any
pub fn pick_counter(g: &Game, q: PlayerId, c: CardId) -> Option<CardId> {
    let mut best: Option<CardId> = None;
    let cheaper = |best: Option<CardId>, ctr: CardId| best.is_none_or(|b| g.db.get(ctr).cmc < g.db.get(b).cmc);
    for &ctr in &g.player(q).hand {
        let d = g.db.get(ctr);
        let t = &d.tags;
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
        if !g.hooks.is_empty() && !castable(g, q, ctr, "hand") {
            continue;
        }
        if t.has(Tag::Fierce) && commander_out(g, q) {
            return Some(ctr); // Fierce Guardianship: free with your commander out
        }
        if t.has(Tag::Pact) && g.player(q).lands.len() >= 5 {
            return Some(ctr); // Pact of Negation: free now, {3}{U}{U} next upkeep
        }
        if t.has(Tag::Misstep) && g.player(q).life > 10 {
            return Some(ctr); // Mental Misstep: 2 life
        }
        if t.has(Tag::Fon) && g.active != Some(q) && !blue_in_hand(g, q, ctr).is_empty() {
            return Some(ctr); // Force of Negation: free on others' turns
        }
        let (cg, cp) = counter_cost(g, ctr, Some(c), Some(q));
        if t.has(Tag::Free) {
            if (!blue_in_hand(g, q, ctr).is_empty() || can_pay(g, q, cg, &cp, false)) && best.is_none() {
                best = Some(ctr);
            }
            continue;
        }
        if can_pay(g, q, cg, &cp, false) && (cheaper(best, ctr) || best.is_some_and(|b| g.db.get(b).tag(Tag::Free))) {
            best = Some(ctr);
        }
    }
    best
}

/// counterspell c costs q no mana right now: Force of Will-style pitch counters, or Fierce Guardianship (and kin)
/// while q controls its commander
pub fn free_counter(g: &Game, q: PlayerId, c: CardId) -> bool {
    let t = &g.db.get(c).tags;
    t.has(Tag::Free) || (t.has(Tag::Fierce) && commander_out(g, q))
}

/// pay counterspell ctr's cost (or its alternative cost) against spell; true if paid
pub fn pay_counter(g: &mut Game, q: PlayerId, ctr: CardId, spell: Option<CardId>) -> Res<bool> {
    if g.free_counter {
        return Ok(true); // practice mode: paid from the person's pool
    }
    let t = g.db.get(ctr).tags.clone();
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
    let pitch = |g: &mut Game, blues: Vec<CardId>| {
        // the least useful blue card (HUMAN(phase 9): the person picks, play/cards.pick_blue)
        let x = crate::engine::zones::min_by(&blues, |c| crate::engine::tutors::card_worth(g, q, c, false)).unwrap();
        let pl = g.player_mut(q);
        let i = pl.hand.iter().position(|&c| c == x).unwrap();
        pl.hand.remove(i);
        pl.exile.push(x);
    };
    if t.has(Tag::Fon) && g.active != Some(q) && !blue_in_hand(g, q, ctr).is_empty() {
        let b = blue_in_hand(g, q, ctr);
        pitch(g, b);
        return Ok(true);
    }
    let (cg, cp) = counter_cost(g, ctr, spell, Some(q));
    if t.has(Tag::Free) && !can_pay(g, q, cg, &cp, false) {
        let blues = blue_in_hand(g, q, ctr);
        if blues.is_empty() {
            return Ok(false);
        }
        pitch(g, blues);
        lose_life(g, q, 1, Some(q), "other", None)?;
        return Ok(true);
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
    if stack_window(g, q, it)? {
        resolve_counter(g, q, ctr, target)?;
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
        // Spell Pierce / Mystic Confluence: pay and it resolves (HUMAN(phase 9): the person may decline, hc.pay_tax)
        pay(g, p, soft as u32, "", false)?;
        crate::glog!(g, "    {} pays {}", g.player(p).name, soft);
        return Ok(());
    }
    crate::glog!(g, "    {} counters {} with {}", g.player(q).name, g.db.get(c).name, g.db.get(ctr).name);
    counter_side_effects(g, q, p, ctr)?;
    if &*g.db.get(ctr).name == "Mana Drain" {
        g.player_mut(q).drain_mana += g.db.get(c).cmc;
    }
    if g.player(p).key == "seph" {
        g.player_mut(p).stat("seph_spell_countered", 1);
    }
    g.player_mut(p).stat("spells_countered", 1);
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
            if c == g.player(p).cmd {
                g.player_mut(p).cmd_in_zone = true;
            } else if it.zone == "gy" || it.ctx.exile_after {
                g.player_mut(p).exile.push(c);
            } else if !g.db.get(c).land {
                g.player_mut(p).gy.push(c);
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
            g.player_mut(p).gy.push(c);
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
    if name == "Statute of Denial"
        && g.player(q).perms.iter().any(|&m| g.is_creature(m) && !g.perm(m).phased && colors_of(g, m).has('U'))
    {
        draw(g, q, 1, false)?; // a blue creature: draw a card, then discard a card
        discard_worst(g, q, 1)?; // HUMAN(phase 9): hc.discard
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
