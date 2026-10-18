//! Tier 6 combo shortcuts for JD's Godo, Bandit Warlord deck (audit/t6/godo-bandit-warlord/SPEC.md, win lines 1, 2, 3,
//! 5 and 6), as `combos::Combo`s with declared steps (`combos::Step`): each step has the answer window that fits it,
//! and an answered step stops the combo (Godo stays out; the AI may try another line later).
//!
//! The Godo lines share their shape: cast Godo (a spell: counter window), his enter trigger fetches an Equipment
//! (the real engine runs it, with the real ability-answer path, `Engine` step), the Helm gets attached (by Hammer of
//! Nazahn's trigger, a Brass Squire / Magnetic Theft / {5} equip step), then Helm's beginning-of-combat trigger (a
//! `Trigger` step) and the permanents Godo and Helm (a `Permanent` step: instant-speed removal or bounce). Once the
//! last answerable step passes, the loop (a hasty token Godo each combat, each first attack adding a combat) is not
//! played out: the player wins, as for every other combo. Speed: the real loop would run hundreds of combats.
//!
//! Lines: (a) Hammer out / Brass Squire / Magnetic Theft / {5} equip; (b) Peter Parker's Camera copies Godo's enter
//! trigger (the copy fetches Hammer, the original Helm); (c) Hammer first, then a Godo copy whose enter trigger
//! fetches Helm: a copy spell (Twinflame, Molten Duplication, Heat Shimmer, Electroduplicate), Cursed Mirror or
//! Sculpting Steel as the Mirror, or Jaxis; (d) Dualcaster Mage + a copy spell (infinite hasty bodies, no commander
//! needed). Left out: Goblin Welder + Goblin Engineer (line 4), a rebuild with Godo already out (line 5's second
//! half), Basalt Monolith + a copied Kinnan (line 8).

use super::combos::{Combo, Ready, Step, StepKind, done, no_keys, no_src};
use super::t6_godo::{godo_fetch, godo_wants};
use crate::engine::cast::{castable, on_cast};
use crate::engine::life::check_state;
use crate::engine::mana::{can_pay, cost_of, pay};
use crate::engine::stack::{camera_ready, counter_window};
use crate::engine::zones::{Enter, enter, enter_token_copy};
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};

const GODO: &str = "Godo, Bandit Warlord";
const HELM: &str = "Helm of the Host";
const HAMMER: &str = "Hammer of Nazahn";
const SQUIRE: &str = "Brass Squire";
const THEFT: &str = "Magnetic Theft";
const CAMERA: &str = "Peter Parker's Camera";
const DUALCASTER: &str = "Dualcaster Mage";
const MIRROR: &str = "Cursed Mirror";
const STEEL: &str = "Sculpting Steel";
const JAXIS: &str = "Jaxis, the Troublemaker";
/// the copy spells that make a token copy of a creature
const TOKEN_COPIERS: &[&str] = &["Twinflame", "Molten Duplication", "Heat Shimmer", "Electroduplicate"];

// ------------------------------------------------------------------ helpers
fn nm(g: &Game, m: PermId) -> &str {
    &g.perm(m).name
}

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && nm(g, m) == name)
}

/// the last permanent of this name (the newest: a copy that just entered)
fn bf_last(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().rev().copied().find(|&m| live(g, m) && nm(g, m) == name)
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> Option<CardId> {
    g.player(p).hand.iter().copied().find(|&c| cname(g, c) == name)
}

fn in_lib(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).library.iter().any(|&c| cname(g, c) == name)
}

/// Godo's card: in the command zone, or in hand (Command Beacon)
fn godo_card(g: &Game, p: PlayerId) -> Option<CardId> {
    let pl = g.player(p);
    pl.cmds_in_zone().chain(pl.hand.iter().copied()).find(|&c| cname(g, c) == GODO)
}

type Cost = (u32, String);

fn cost(g: &Game, p: PlayerId, c: CardId) -> Cost {
    cost_of(g, p, c)
}

/// can p pay all of these at once?
fn affordable(g: &Game, p: PlayerId, parts: &[Cost]) -> bool {
    let gn: u32 = parts.iter().map(|x| x.0).sum();
    let pips: String = parts.iter().map(|x| x.1.as_str()).collect();
    can_pay(g, p, gn, &pips, false)
}

fn extra(gn: u32, pips: &str) -> Cost {
    (gn, pips.to_string())
}

fn ok_ready() -> Ready {
    (true, vec![], vec![])
}

fn no() -> Ready {
    (false, vec![], vec![])
}

/// everything every Godo line needs: Godo castable, not out yet, Helm still in the library; the cost of Godo
fn godo_base(g: &Game, p: PlayerId) -> Option<Cost> {
    let c = godo_card(g, p)?;
    let zone = if g.player(p).hand.contains(&c) { "hand" } else { "cmd" };
    if bf(g, p, GODO).is_some() || bf(g, p, HELM).is_some() || !in_lib(g, p, HELM) || !castable(g, p, c, zone) {
        return None;
    }
    Some(cost(g, p, c))
}

fn squire_ready(g: &Game, p: PlayerId) -> Option<PermId> {
    bf(g, p, SQUIRE).filter(|&m| !g.perm(m).tapped && !g.perm(m).sick)
}

fn token_copier(g: &Game, p: PlayerId) -> Option<CardId> {
    // the cheapest copy spell in hand
    g.player(p)
        .hand
        .iter()
        .copied()
        .filter(|&c| TOKEN_COPIERS.contains(&cname(g, c)))
        .min_by_key(|&c| cost(g, p, c).0 + cost(g, p, c).1.len() as u32)
}

fn my_creature(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && g.is_creature(m))
}

// ------------------------------------------------------------------ ready checks
/// line 2, Hammer of Nazahn out: Godo's fetch is Helm, Hammer attaches it at once
fn ready_hammer(g: &Game, p: PlayerId) -> Ready {
    let Some(godo) = godo_base(g, p) else { return no() };
    if bf(g, p, HAMMER).is_none() || godo_wants(g, p) != Some(HELM) || !affordable(g, p, &[godo]) {
        return no();
    }
    ok_ready()
}

/// line 2, an untapped Brass Squire: {T}: attach the Helm to Godo
fn ready_squire(g: &Game, p: PlayerId) -> Ready {
    let Some(godo) = godo_base(g, p) else { return no() };
    if bf(g, p, HAMMER).is_some() || squire_ready(g, p).is_none() || godo_wants(g, p) != Some(HELM) {
        return no();
    }
    if !affordable(g, p, &[godo]) { no() } else { ok_ready() }
}

/// line 2, Magnetic Theft ({R}) attaches the Helm
fn ready_theft(g: &Game, p: PlayerId) -> Ready {
    let Some(godo) = godo_base(g, p) else { return no() };
    let Some(theft) = in_hand(g, p, THEFT) else { return no() };
    if bf(g, p, HAMMER).is_some() || squire_ready(g, p).is_some() || godo_wants(g, p) != Some(HELM) {
        return no();
    }
    if !affordable(g, p, &[godo, cost(g, p, theft)]) { no() } else { ok_ready() }
}

/// line 1, nothing cheaper: pay {5} to equip
fn ready_equip(g: &Game, p: PlayerId) -> Ready {
    let Some(godo) = godo_base(g, p) else { return no() };
    if bf(g, p, HAMMER).is_some()
        || squire_ready(g, p).is_some()
        || in_hand(g, p, THEFT).is_some()
        || godo_wants(g, p) != Some(HELM)
    {
        return no();
    }
    if !affordable(g, p, &[godo, extra(5, "")]) { no() } else { ok_ready() }
}

/// hammer-first lines: Hammer and Helm in the library, Hammer not out, Godo fetches Hammer first
fn hammer_first(g: &Game, p: PlayerId) -> Option<Cost> {
    let godo = godo_base(g, p)?;
    if bf(g, p, HAMMER).is_some() || !in_lib(g, p, HAMMER) || godo_wants(g, p) != Some(HAMMER) {
        return None;
    }
    Some(godo)
}

/// line 3, Peter Parker's Camera: {2}, {T} copies the enter trigger
fn ready_camera(g: &Game, p: PlayerId) -> Ready {
    let Some(godo) = hammer_first(g, p) else { return no() };
    if !camera_ready(g, p) || !affordable(g, p, &[godo, extra(2, "")]) {
        return no();
    }
    ok_ready()
}

/// line 3, Hammer first, then a copy spell on Godo
fn ready_copy_spell(g: &Game, p: PlayerId) -> Ready {
    let Some(godo) = hammer_first(g, p) else { return no() };
    let Some(c) = token_copier(g, p) else { return no() };
    if !affordable(g, p, &[godo, cost(g, p, c)]) { no() } else { ok_ready() }
}

/// the Mirror line's card: Cursed Mirror, or Sculpting Steel with a Cursed Mirror on the battlefield to copy
fn mirror_card(g: &Game, p: PlayerId) -> Option<CardId> {
    in_hand(g, p, MIRROR).or_else(|| in_hand(g, p, STEEL).filter(|_| bf(g, p, MIRROR).is_some()))
}

fn ready_mirror(g: &Game, p: PlayerId) -> Ready {
    let Some(godo) = hammer_first(g, p) else { return no() };
    let Some(c) = mirror_card(g, p) else { return no() };
    if !affordable(g, p, &[godo, cost(g, p, c)]) { no() } else { ok_ready() }
}

/// Jaxis: on the battlefield and untapped (not summoning sick), or blitzed from hand ({1}{R}, haste); {R}, {T}
fn jaxis_plan(g: &Game, p: PlayerId) -> Option<Cost> {
    if bf(g, p, JAXIS).is_some_and(|m| !g.perm(m).tapped && !g.perm(m).sick) {
        return Some(extra(0, "R"));
    }
    in_hand(g, p, JAXIS).map(|_| extra(1, "RR"))
}

fn ready_jaxis(g: &Game, p: PlayerId) -> Ready {
    let Some(godo) = hammer_first(g, p) else { return no() };
    let Some(j) = jaxis_plan(g, p) else { return no() };
    if !affordable(g, p, &[godo, j]) { no() } else { ok_ready() }
}

/// line 6: Dualcaster Mage and a copy spell, and a creature to target
fn ready_dualcaster(g: &Game, p: PlayerId) -> Ready {
    let (Some(d), Some(s)) = (in_hand(g, p, DUALCASTER), token_copier(g, p)) else { return no() };
    if my_creature(g, p).is_none() || !affordable(g, p, &[cost(g, p, d), cost(g, p, s)]) {
        return no();
    }
    ok_ready()
}

// ------------------------------------------------------------------ steps
fn cast_godo(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(c) = godo_card(g, p) else { return Ok(false) };
    crate::ai::brain::do_cast(g, p, c, None)?;
    Ok(bf(g, p, GODO).is_some())
}

/// Godo's enter trigger (the engine ran it with its own windows): the Equipment the line needs is out
fn helm_fetched(g: &mut Game, p: PlayerId) -> Res<bool> {
    Ok(bf(g, p, GODO).is_some() && bf(g, p, HELM).is_some())
}

/// Camera line: the copy fetched Hammer, the original Helm
fn both_fetched(g: &mut Game, p: PlayerId) -> Res<bool> {
    Ok(bf(g, p, GODO).is_some() && bf(g, p, HELM).is_some() && bf(g, p, HAMMER).is_some())
}

/// hammer-first lines: the first fetch is Hammer
fn hammer_fetched(g: &mut Game, p: PlayerId) -> Res<bool> {
    Ok(bf(g, p, GODO).is_some() && bf(g, p, HAMMER).is_some())
}

/// the Helm is worn by a Godo
fn helm_on_godo(g: &Game, p: PlayerId) -> bool {
    bf(g, p, HELM).is_some_and(|h| g.perm(h).attached.is_some_and(|t| live(g, t) && nm(g, t) == GODO))
}

fn worn(g: &mut Game, p: PlayerId) -> Res<bool> {
    Ok(helm_on_godo(g, p))
}

fn helm_src(g: &Game, p: PlayerId) -> Option<PermId> {
    bf(g, p, HELM)
}

fn squire_src(g: &Game, p: PlayerId) -> Option<PermId> {
    bf(g, p, SQUIRE)
}

fn attach_helm(g: &mut Game, p: PlayerId) -> bool {
    match (bf(g, p, HELM), bf_last(g, p, GODO)) {
        (Some(h), Some(t)) => super::cloud::attach(g, h, t),
        _ => false,
    }
}

fn squire_attach(g: &mut Game, p: PlayerId) -> Res<bool> {
    if let Some(s) = bf(g, p, SQUIRE) {
        g.perm_mut(s).tapped = true;
    }
    Ok(attach_helm(g, p))
}

fn equip_helm(g: &mut Game, p: PlayerId) -> Res<bool> {
    Ok(attach_helm(g, p))
}

fn cast_theft(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(c) = in_hand(g, p, THEFT) else { return Ok(false) };
    crate::ai::brain::do_cast(g, p, c, None)?;
    Ok(helm_on_godo(g, p))
}

fn godo_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    [bf(g, p, GODO), bf(g, p, HELM)].into_iter().flatten().collect()
}

/// pay for, cast and (if `resolve`) resolve a copy spell aimed at `target`: the counter window is the normal one;
/// the token copy is hasty and goes at the end step. Not resolved: the spell stays "on the stack" (Dualcaster copies it)
/// and ends in the graveyard.
fn cast_copy_spell(g: &mut Game, p: PlayerId, c: CardId, target: PermId, resolve: bool) -> Res<bool> {
    let (gn, pips) = cost(g, p, c);
    if !g.player(p).hand.contains(&c) || !castable(g, p, c, "hand") || !can_pay(g, p, gn, &pips, false) {
        return Ok(false);
    }
    pay(g, p, gn, &pips, false)?;
    let hand = &mut g.player_mut(p).hand;
    if let Some(i) = hand.iter().position(|&x| x == c) {
        hand.remove(i);
    }
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    crate::glog!(g, "  {} casts {} -> {}", g.player(p).name, cname(g, c), nm(g, target));
    on_cast(g, p, c)?;
    if g.over || !counter_window(g, p, c, 6.0, vec![])? {
        g.player_mut(p).gy.push(c);
        return Ok(false);
    }
    if resolve && live(g, target) {
        let cd = g.perm(target).cd.unwrap();
        if let Some(t) = enter_token_copy(g, p, cd)?
            && g.perm(t).on_bf
        {
            let x = g.perm_mut(t);
            x.sick = false; // haste
            if cname(g, c) == "Heat Shimmer" || cname(g, c) == "Twinflame" {
                g.perm_mut(t).temp = true; // exiled at the end step
            } else {
                g.perm_mut(t).data.set(DataKey::EotSac, Val::Bool(true));
            }
        }
    }
    g.player_mut(p).gy.push(c);
    check_state(g)?;
    Ok(true)
}

/// (c) a copy spell on Godo: the token's own enter trigger runs for real (fetches Helm; Hammer attaches it)
fn cast_copier_on_godo(g: &mut Game, p: PlayerId) -> Res<bool> {
    let (Some(c), Some(godo)) = (token_copier(g, p), bf(g, p, GODO)) else { return Ok(false) };
    if !cast_copy_spell(g, p, c, godo, true)? {
        return Ok(false);
    }
    Ok(bf(g, p, HELM).is_some())
}

/// (c) Cursed Mirror (or Sculpting Steel copying it) enters as a copy of Godo
fn cast_mirror(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(c) = mirror_card(g, p) else { return Ok(false) };
    let (gn, pips) = cost(g, p, c);
    if !castable(g, p, c, "hand") || !can_pay(g, p, gn, &pips, false) {
        return Ok(false);
    }
    pay(g, p, gn, &pips, false)?;
    let hand = &mut g.player_mut(p).hand;
    if let Some(i) = hand.iter().position(|&x| x == c) {
        hand.remove(i);
    }
    g.player_mut(p).spells_this_turn += 1;
    g.player_mut(p).cast_names.insert(c);
    crate::glog!(g, "  {} casts {} as a copy of Godo", g.player(p).name, cname(g, c));
    on_cast(g, p, c)?;
    if g.over || !counter_window(g, p, c, 6.0, vec![])? {
        g.player_mut(p).gy.push(c);
        return Ok(false);
    }
    enter(g, p, c, Enter { was_cast: true, ..Enter::default() })?;
    Ok(bf(g, p, GODO).is_some())
}

/// the Mirror (or the Steel that is a Mirror): the source of the copy's enter trigger
fn mirror_src(g: &Game, p: PlayerId) -> Option<PermId> {
    bf_last(g, p, MIRROR).or_else(|| bf_last(g, p, STEEL))
}

/// the Godo copy's enter trigger resolves: fetch Helm (Hammer attaches it)
fn copy_fetch(g: &mut Game, p: PlayerId) -> Res<bool> {
    godo_fetch(g, p)?;
    Ok(bf(g, p, HELM).is_some())
}

/// Jaxis onto the battlefield (blitz from hand), or already there
fn jaxis_in(g: &mut Game, p: PlayerId) -> Res<bool> {
    if bf(g, p, JAXIS).is_some() {
        return Ok(true);
    }
    let Some(c) = in_hand(g, p, JAXIS) else { return Ok(false) };
    super::partials::jaxis_blitz_go(g, p, c.0 as i64)?;
    Ok(bf(g, p, JAXIS).is_some())
}

fn jaxis_src(g: &Game, p: PlayerId) -> Option<PermId> {
    bf(g, p, JAXIS)
}

/// Jaxis: {R}, {T}: a hasty token copy of Godo (its enter trigger runs for real)
fn jaxis_copy(g: &mut Game, p: PlayerId) -> Res<bool> {
    let (Some(j), Some(godo)) = (bf(g, p, JAXIS), bf(g, p, GODO)) else { return Ok(false) };
    g.perm_mut(j).tapped = true;
    let cd = g.perm(godo).cd.unwrap();
    if let Some(t) = enter_token_copy(g, p, cd)?
        && g.perm(t).on_bf
    {
        g.perm_mut(t).sick = false;
        g.perm_mut(t).temp = true;
    }
    Ok(bf(g, p, HELM).is_some())
}

// (d) Dualcaster Mage
fn cast_original_copy_spell(g: &mut Game, p: PlayerId) -> Res<bool> {
    let (Some(c), Some(t)) = (token_copier(g, p), my_creature(g, p)) else { return Ok(false) };
    cast_copy_spell(g, p, c, t, false)
}

fn cast_dualcaster(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(c) = in_hand(g, p, DUALCASTER) else { return Ok(false) };
    let (gn, pips) = cost(g, p, c);
    if !castable(g, p, c, "hand") || !can_pay(g, p, gn, &pips, false) {
        return Ok(false);
    }
    pay(g, p, gn, &pips, false)?;
    crate::glog!(g, "  {} flashes in Dualcaster Mage over its own copy spell", g.player(p).name);
    let ok = crate::engine::cast::cast_card(g, p, c, "hand", crate::state::Ctx::default())?;
    Ok(ok && bf(g, p, DUALCASTER).is_some())
}

fn dualcaster_src(g: &Game, p: PlayerId) -> Option<PermId> {
    bf(g, p, DUALCASTER)
}

// ------------------------------------------------------------------ the step tables
const fn step(label: &'static str, kind: StepKind, run: fn(&mut Game, PlayerId) -> Res<bool>) -> Step {
    Step { label, kind, src: no_src, pay: (0, ""), run, keys: no_keys }
}

const S_CAST: Step = step("cast Godo", StepKind::Spell, cast_godo);
const S_FETCH_HELM: Step = step("Godo's enter trigger fetches Helm of the Host", StepKind::Engine, helm_fetched);
const S_FETCH_BOTH: Step =
    step("the copied enter trigger fetches Hammer, the original Helm", StepKind::Engine, both_fetched);
const S_FETCH_HAMMER: Step = step("Godo's enter trigger fetches Hammer of Nazahn", StepKind::Engine, hammer_fetched);
const S_WORN: Step = step("Hammer's trigger attaches Helm to Godo", StepKind::Engine, worn);
const S_SQUIRE: Step = Step {
    label: "Brass Squire attaches Helm to Godo",
    kind: StepKind::Ability,
    src: squire_src,
    pay: (0, ""),
    run: squire_attach,
    keys: no_keys,
};
const S_THEFT: Step = step("Magnetic Theft attaches Helm to Godo", StepKind::Spell, cast_theft);
const S_EQUIP: Step = Step {
    label: "equip Helm of the Host to Godo",
    kind: StepKind::Ability,
    src: helm_src,
    pay: (5, ""),
    run: equip_helm,
    keys: no_keys,
};
const S_COPY_SPELL: Step = step("copy spell on Godo", StepKind::Spell, cast_copier_on_godo);
const S_MIRROR: Step = step("Cursed Mirror enters as a copy of Godo", StepKind::Spell, cast_mirror);
const S_MIRROR_TRIGGER: Step = Step {
    label: "the Godo copy's enter trigger fetches Helm of the Host",
    kind: StepKind::Trigger,
    src: mirror_src,
    pay: (0, ""),
    run: copy_fetch,
    keys: no_keys,
};
const S_JAXIS_IN: Step = step("Jaxis enters (blitz)", StepKind::Spell, jaxis_in);
const S_JAXIS_COPY: Step = Step {
    label: "Jaxis copies Godo",
    kind: StepKind::Ability,
    src: jaxis_src,
    pay: (0, "R"),
    run: jaxis_copy,
    keys: no_keys,
};
const S_COMBAT: Step = Step {
    label: "Helm of the Host makes a token copy of Godo at the beginning of combat",
    kind: StepKind::Trigger,
    src: helm_src,
    pay: (0, ""),
    run: done,
    keys: no_keys,
};
const S_LOOP: Step = Step {
    label: "Godo and Helm of the Host stay: each token's first attack adds a combat",
    kind: StepKind::Permanent,
    src: no_src,
    pay: (0, ""),
    run: done,
    keys: godo_keys,
};

const fn combo(
    name: &'static str,
    groups: &'static [&'static [&'static str]],
    ready: fn(&Game, PlayerId) -> Ready,
    steps: &'static [Step],
) -> Combo {
    Combo { name, groups, ready, mana: (0, ""), finish: None, xcost: &[], steps }
}

/// the Godo combos (chained after `combos::BASE` by `combos::all`)
pub static COMBOS: [Combo; 9] = [
    combo(
        "Godo + Helm of the Host (Hammer of Nazahn out)",
        &[&[GODO], &[HELM], &[HAMMER]],
        ready_hammer,
        &[S_CAST, S_FETCH_HELM, S_WORN, S_COMBAT, S_LOOP],
    ),
    combo(
        "Godo + Helm of the Host (Brass Squire attaches)",
        &[&[GODO], &[HELM], &[SQUIRE]],
        ready_squire,
        &[S_CAST, S_FETCH_HELM, S_SQUIRE, S_COMBAT, S_LOOP],
    ),
    combo(
        "Godo + Helm of the Host (Magnetic Theft attaches)",
        &[&[GODO], &[HELM], &[THEFT]],
        ready_theft,
        &[S_CAST, S_FETCH_HELM, S_THEFT, S_COMBAT, S_LOOP],
    ),
    combo(
        "Godo + Helm of the Host (equip {5})",
        &[&[GODO], &[HELM]],
        ready_equip,
        &[S_CAST, S_FETCH_HELM, S_EQUIP, S_COMBAT, S_LOOP],
    ),
    combo(
        "Godo + Peter Parker's Camera copies the enter trigger",
        &[&[GODO], &[HELM], &[HAMMER], &[CAMERA]],
        ready_camera,
        &[S_CAST, S_FETCH_BOTH, S_WORN, S_COMBAT, S_LOOP],
    ),
    combo(
        "Godo + Hammer first, then a copy spell on Godo",
        &[&[GODO], &[HELM], &[HAMMER], TOKEN_COPIERS],
        ready_copy_spell,
        &[S_CAST, S_FETCH_HAMMER, S_COPY_SPELL, S_WORN, S_COMBAT, S_LOOP],
    ),
    combo(
        "Godo + Hammer first, then Cursed Mirror / Sculpting Steel as Godo",
        &[&[GODO], &[HELM], &[HAMMER], &[MIRROR, STEEL]],
        ready_mirror,
        &[S_CAST, S_FETCH_HAMMER, S_MIRROR, S_MIRROR_TRIGGER, S_WORN, S_COMBAT, S_LOOP],
    ),
    combo(
        "Godo + Hammer first, then Jaxis copies Godo",
        &[&[GODO], &[HELM], &[HAMMER], &[JAXIS]],
        ready_jaxis,
        &[S_CAST, S_FETCH_HAMMER, S_JAXIS_IN, S_JAXIS_COPY, S_WORN, S_COMBAT, S_LOOP],
    ),
    combo(
        "Dualcaster Mage + a copy spell",
        &[&[DUALCASTER], TOKEN_COPIERS],
        ready_dualcaster,
        &[
            step("cast the copy spell and hold priority", StepKind::Spell, cast_original_copy_spell),
            step("flash in Dualcaster Mage", StepKind::Spell, cast_dualcaster),
            Step {
                label: "Dualcaster Mage's enter trigger copies the spell (again and again)",
                kind: StepKind::Trigger,
                src: dualcaster_src,
                pay: (0, ""),
                run: done,
                keys: no_keys,
            },
        ],
    ),
];
