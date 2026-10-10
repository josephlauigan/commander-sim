//! Python's `cards/impl/partials.py`: the abilities the audit listed as missing, implemented from their rules text
//! (the Tier 1 and Sauron cards so far; the other decks' cards come with phase 6).

use crate::cardcode;
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{cast_card, castable, on_cast};
use crate::engine::hooks::fire_trigger;
use crate::engine::life::gain;
use crate::engine::mana::{Source, Unit, can_pay, cost_of, pay, total_mana};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{abilities_answered, ability_window, counter_window, stack_window, trigger_window};
use crate::engine::tutors::card_worth;
use crate::engine::values::{epow, etgh, once_per_turn, pval, untargetable};
use crate::engine::zones::{
    Enter, Tokens, die, draw, enter, enter_token_copy, exile_perm, leave, make_tokens, max_by, min_by,
};
use crate::flow::Res;
use crate::hooks::{Action, CardImpl, Event, Opt, Registry, Sacrificed, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, StackItem, StackKind, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

// ------------------------------------------------------------------ helpers shared by the card modules
/// set a triggered-event slot's "runs at once" bit: Python runs a hook at once when its source has no
/// `trigger_window` (`engine.converted`). A later module replacing an earlier one's hook sets the bit either way.
pub(crate) fn at_once(c: &mut CardImpl, e: Event, yes: bool) {
    let bit = 1u128 << (e as u32);
    if yes {
        c.at_once |= bit;
    } else {
        c.at_once &= !bit;
    }
}

/// common.best_opp_creature: the most valuable targetable creature of p's opponents that passes `pred` (a copy until
/// common.rs has it)
pub(crate) fn best_opp_creature(g: &Game, p: PlayerId, pred: impl Fn(&Game, PermId) -> bool) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && !g.perm(m).phased && !untargetable(g, m) && pred(g, m))
        .collect();
    max_by(&cs, |m| pval(g, m))
}

/// common.best_opp_nonland: the most valuable targetable permanent of p's opponents that passes `pred`
pub(crate) fn best_opp_nonland(g: &Game, p: PlayerId, pred: impl Fn(&Game, PermId) -> bool) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| !g.perm(m).phased && !untargetable(g, m) && pred(g, m))
        .collect();
    max_by(&cs, |m| pval(g, m))
}

/// common.auras_on: the Auras on m that the card code tracks (a copy until common.rs has it)
pub(crate) fn auras_on(g: &Game, m: PermId) -> Vec<PermId> {
    g.auras
        .iter()
        .copied()
        .filter(|&a| {
            let x = g.perm(a);
            x.attached == Some(m) && x.on_bf && !x.phased
        })
        .collect()
}

/// the first of xs with the greatest key (Python's `max(xs, key=..)` with a tuple key)
pub(crate) fn first_max<T: Copy, K: PartialOrd>(xs: &[T], key: impl Fn(T) -> K) -> Option<T> {
    let mut best: Option<(T, K)> = None;
    for &x in xs {
        let k = key(x);
        if best.as_ref().is_none_or(|b| k > b.1) {
            best = Some((x, k));
        }
    }
    best.map(|b| b.0)
}

/// the first of xs with the least key (Python's `min(xs, key=..)` with a tuple key)
pub(crate) fn first_min<T: Copy, K: PartialOrd>(xs: &[T], key: impl Fn(T) -> K) -> Option<T> {
    let mut best: Option<(T, K)> = None;
    for &x in xs {
        let k = key(x);
        if best.as_ref().is_none_or(|b| k < b.1) {
            best = Some((x, k));
        }
    }
    best.map(|b| b.0)
}

/// a permanent's card name ("" for a token)
pub(crate) fn name_of(g: &Game, m: PermId) -> &str {
    g.perm(m).cd.map_or("", |c| &g.db.get(c).name)
}

/// m is a card of type t (Python's `m.cd is not None and t in m.cd.types`)
pub(crate) fn of_type(g: &Game, m: PermId, t: Types) -> bool {
    g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(t))
}

/// m is on p's battlefield (Python's `m in p.perms`)
pub(crate) fn on(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.perm(m).on_bf && g.perm(m).owner == p
}

/// a keyword until end of turn (Python's `g.eot_kw.setdefault(id(m), set()).add(kw)`)
pub(crate) fn eot_kw(g: &mut Game, m: PermId, kw: Sym) {
    let x = g.perm_mut(m);
    if !x.eot_kw.contains(&kw) {
        x.eot_kw.push(kw);
    }
}

/// remove card c from a list (Python's `list.remove`: the first copy)
pub(crate) fn remove_card(v: &mut Vec<CardId>, c: CardId) -> bool {
    match v.iter().position(|&x| x == c) {
        Some(i) => {
            v.remove(i);
            true
        }
        None => false,
    }
}

/// two ids in an option's argument
pub(crate) fn pack(a: u32, b: u32) -> i64 {
    a as i64 | (b as i64) << 32
}

pub(crate) fn unpack(x: i64) -> (u32, u32) {
    ((x & 0xffff_ffff) as u32, (x >> 32) as u32)
}

/// engine.ability_window for an ability of a card (from the graveyard, or a permanent's card after it was
/// sacrificed as the cost): Python passes the card as `src`
pub(crate) fn ability_window_card(g: &mut Game, p: PlayerId, cd: CardId, name: &str) -> Res<bool> {
    if g.over || !abilities_answered(g, Some(p)) {
        return Ok(true);
    }
    let it = StackItem {
        id: g.stack_pushes + 1,
        controller: p,
        card: Some(cd),
        ctx: Ctx::default(),
        zone: "ability",
        imp: 3.0,
        aff: vec![],
        generic: false,
        kind: StackKind::Ability,
        name: format!("{}: {name}", g.db.get(cd).name),
        passed: vec![],
        countered: false,
        countered_by: None,
    };
    stack_window(g, p, it)
}

fn turn_now(g: &Game) -> Val {
    Val::Stamp(g.turn_stamp())
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

/// the card's own cost, paid (Python's `pay(g, p, *cost_of(p, c))`)
fn pay_cost(g: &mut Game, p: PlayerId, c: CardId) -> Res<bool> {
    let (gn, pips) = cost_of(g, p, c);
    pay(g, p, gn, &pips, false)
}

fn can_pay_cost(g: &Game, p: PlayerId, c: CardId) -> bool {
    let (gn, pips) = cost_of(g, p, c);
    can_pay(g, p, gn, &pips, false)
}

/// partials.tajic_protects: Tajic prevents noncombat damage to its controller's other creatures
pub fn tajic_protects(g: &Game, m: PermId) -> bool {
    let o = g.perm(m).owner;
    g.player(o).perms.iter().any(|&x| x != m && !g.perm(x).phased && name_of(g, x) == "Tajic, Legion's Edge")
}

// ======================================================== Brutal Hordechief: you choose how they block
/// {3}{R/W}{R/W}: opponents block as you choose (their creatures block where they die without killing)
fn hordechief_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p
        || post != Some(false)
        || !can_pay(g, p, 3, "RR", false) && !can_pay(g, p, 3, "WW", false) && !can_pay(g, p, 3, "RW", false)
    {
        return Ok(vec![]);
    }
    let ready = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| {
            let x = g.perm(m);
            g.is_creature(m) && !x.sick && !x.tapped && !x.noatk
        })
        .count();
    let theirs = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && !g.perm(m).tapped)
        .count();
    if ready < 3 || theirs == 0 || g.perm(src).data.get(DataKey::Lure) == Some(&turn_now(g)) {
        return Ok(vec![]);
    }
    // the hybrid pips are chosen as the option is offered
    let pips = HORDECHIEF_PIPS.iter().position(|x| can_pay(g, p, 3, x, false)).unwrap();
    Ok(vec![Opt {
        utility: 0.5 * theirs as f64,
        label: "Brutal Hordechief (choose blocks)".into(),
        act: Some(Action::Ability { src, f: hordechief_go, arg: pips as i64 }),
    }])
}

const HORDECHIEF_PIPS: [&str; 3] = ["RR", "RW", "WW"];

fn hordechief_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let pips = HORDECHIEF_PIPS[arg as usize];
    if !can_pay(g, p, 3, pips, false) {
        return Ok(false);
    }
    pay(g, p, 3, pips, false)?;
    if !ability_window(g, p, Some(src), "opponents block as you choose", None, None)? {
        return Ok(true);
    }
    let now = turn_now(g);
    g.perm_mut(src).data.set(DataKey::Lure, now);
    crate::glog!(g, "  {} activates Brutal Hordechief: opponents block as {} chooses", pname(g, p), pname(g, p));
    Ok(true)
}

/// after the activation, every untapped creature of the defender blocks, assigned where it dies without killing
fn hordechief_blocks(
    g: &mut Game,
    src: Src,
    p: PlayerId,
    atk: &[PermId],
    d: PlayerId,
    assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    if g.perm(src).owner != p || g.perm(src).data.get(DataKey::Lure) != Some(&turn_now(g)) {
        return Ok(());
    }
    assign.clear();
    let bs: Vec<PermId> = g
        .player(d)
        .perms
        .iter()
        .copied()
        .filter(|&b| g.is_creature(b) && !g.perm(b).tapped && !g.perm(b).phased)
        .collect();
    for b in bs {
        let good: Vec<PermId> = atk
            .iter()
            .copied()
            .filter(|&a| on(g, p, a) && epow(g, a) >= etgh(g, b) && epow(g, b) < etgh(g, a))
            .collect();
        if let Some(a) = max_by(&good, |a| epow(g, a) as f64)
            && !assign.iter().any(|x| x.0 == a)
        {
            assign.push((a, b));
        }
    }
    Ok(())
}

// ======================================================== Chaos Warp: the revealed permanent comes in
/// the target is shuffled away; a revealed permanent card enters for its owner
fn chaos_warp(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) else { return Ok("gy") };
    let q = g.perm(t).owner;
    apply_removal(g, Some(p), t, "tuck", Some(c))?;
    if let Some(&top) = g.player(q).library.last() {
        crate::glog!(g, "    {} reveals {}", pname(g, q), g.db.get(top).name);
        let d = g.db.get(top);
        if d.perm && !d.land {
            g.player_mut(q).library.pop();
            enter(g, q, top, Enter::default())?;
        } else if d.land {
            g.player_mut(q).library.pop();
            g.add_land(q, top, false);
        }
    }
    Ok("gy")
}

// ======================================================== Druid Class level 3
/// level 3 ({3}{G}): a land becomes a creature with power and toughness equal to your lands
fn druid3_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let level = g.perm(src).data.get(DataKey::Level).map_or(1, Val::int);
    if g.perm(src).owner != p || post != Some(false) || level != 2 || !can_pay(g, p, 4, "G", false) {
        return Ok(vec![]);
    }
    let n = g.player(p).lands.len();
    if n < 7 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.0 + n as f64 / 4.0,
        label: "Druid Class level 3".into(),
        act: Some(Action::Ability { src, f: druid3_go, arg: 0 }),
    }])
}

fn druid3_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !can_pay(g, p, 4, "G", false) || g.player(p).lands.is_empty() {
        return Ok(false);
    }
    pay(g, p, 4, "G", false)?;
    if !ability_window(g, p, Some(src), "level 3", None, None)? || !on(g, p, src) || g.player(p).lands.is_empty() {
        return Ok(true);
    }
    g.perm_mut(src).data.set(DataKey::Level, Val::Int(3));
    let lands = g.player(p).lands.clone();
    let l = first_min(&lands, |l| {
        let d = g.db.get(g.land(l).cd);
        (d.tags.str(Tag::C).map_or(0, |s| s.chars().count()), !matches!(&*d.name, "Forest" | "Island"))
    })
    .unwrap();
    crate::engine::turn::remove_land(g, p, l);
    let n = g.player(p).lands.len() as i32 + 1;
    let name = intern(&g.db.get(g.land(l).cd).name);
    let m = g.new_perm(p, None, name, n, n);
    let x = g.perm_mut(m);
    x.sick = false;
    x.ttypes = vec!["land"];
    x.data.set(DataKey::Land, Val::Land(l));
    x.data.set(DataKey::DruidClass, Val::Bool(true));
    x.on_bf = true;
    g.player_mut(p).perms.push(m);
    crate::glog!(g, "  {} levels Druid Class to 3: {} becomes a {}/{} creature", pname(g, p), name, n, n);
    Ok(true)
}

/// the level-3 land creature's size follows your lands
fn druid_size(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner != p {
        return Ok(());
    }
    let n = g.player(p).lands.len() as i32 + 1;
    for m in g.player(p).perms.clone() {
        if g.perm(m).data.truthy(DataKey::DruidClass) {
            let x = g.perm_mut(m);
            x.pow = n;
            x.tgh = n;
        }
    }
    Ok(())
}

// ======================================================== Eidolon of Countless Battles: bestow
/// partials.eidolon_x: creatures and Auras p controls, and bestowed Eidolons
pub fn eidolon_x(g: &Game, p: PlayerId) -> i32 {
    let ps = &g.player(p).perms;
    let cr = ps.iter().filter(|&&x| g.is_creature(x) && !g.perm(x).phased).count();
    let auras = ps
        .iter()
        .filter(|&&x| !g.perm(x).phased && g.perm(x).cd.is_some_and(|c| g.db.get(c).has_subtype("aura")))
        .count();
    let bestowed: usize = ps.iter().map(|&x| bestowed(g, x).len()).sum();
    (cr + auras + bestowed) as i32
}

fn bestowed(g: &Game, m: PermId) -> Vec<CardId> {
    match g.perm(m).data.get(DataKey::Bestow) {
        Some(Val::List(v)) => v.iter().filter_map(|x| if let Val::Card(c) = x { Some(*c) } else { None }).collect(),
        _ => vec![],
    }
}

/// common.own_host: the creature an Aura should go on (the commander in a voltron deck, else the best creature)
fn own_host(g: &Game, p: PlayerId) -> Option<PermId> {
    let cands: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect();
    if cands.is_empty() {
        return None;
    }
    if crate::ai::plans::config(g.player(p).key).aura_host == Some("commander")
        && let Some(&m) = cands.iter().find(|&&m| g.perm(m).is_cmd)
    {
        return Some(m);
    }
    max_by(&cands, |m| {
        let x = g.perm(m);
        x.is_cmd as i32 as f64 * 2.0 + pval(g, m) + if x.fly { 3.0 } else { 0.0 }
    })
}

/// bestow {2}{W}{W}: an Aura on the best creature (the commander in Light-Paws), +X/+X for creatures and Auras
fn bestow_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || !can_pay(g, p, 2, "WW", false) {
        return Ok(vec![]);
    }
    let Some(host) = own_host(g, p) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: 3.0 + 0.5 * eidolon_x(g, p) as f64,
        label: "bestow Eidolon".into(),
        act: Some(Action::Plan { f: bestow_go, arg: pack(c.0 as u32, host.0) }),
    }])
}

fn bestow_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, host) = unpack(arg);
    let (c, host) = (CardId(c as u16), PermId(host));
    if !g.player(p).hand.contains(&c) || !on(g, p, host) || !can_pay(g, p, 2, "WW", false) {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    pay(g, p, 2, "WW", false)?;
    crate::glog!(g, "  {} bestows Eidolon of Countless Battles on {}", pname(g, p), g.perm(host).name);
    on_cast(g, p, c)?;
    if !counter_window(g, p, c, 4.0, vec![])? {
        g.player_mut(p).gy.push(c);
        return Ok(true);
    }
    let data = &mut g.perm_mut(host).data;
    match data.get_mut(DataKey::Bestow) {
        Some(Val::List(v)) => v.push(Val::Card(c)),
        _ => data.set(DataKey::Bestow, Val::List(vec![Val::Card(c)])),
    }
    g.selfpt = true;
    Ok(true)
}

/// partials.bestow_bonus: a bestowed Eidolon's +X/+X on its host
pub fn bestow_bonus(g: &Game, m: PermId) -> i32 {
    let n = bestowed(g, m).len() as i32;
    if n == 0 { 0 } else { n * eidolon_x(g, g.perm(m).owner) }
}

/// partials.coat_bonus: Coat of Arms (+1/+1 per other creature sharing a type). PORT(phase 6): Coat of Arms' code
/// (its enters hook sets `g.coat`); until then no game has it, and the bonus is 0.
pub fn coat_bonus(_g: &Game, _m: PermId) -> i32 {
    0
}

/// partials.lineage_bonus: Lord of Lineage (other Vampires +2/+2). PORT(phase 6): Bloodline Keeper's code (its enters
/// hook sets `g.lineage`); until then the bonus is 0.
pub fn lineage_bonus(_g: &Game, _m: PermId) -> i32 {
    0
}

/// partials.bestow_fall: the host left the battlefield: a bestowed Eidolon becomes a creature
pub fn bestow_fall(g: &mut Game, m: PermId) -> Res {
    let o = g.perm(m).owner;
    for c in bestowed(g, m) {
        if g.player(o).alive {
            enter(g, o, c, Enter::default())?;
        }
    }
    g.perm_mut(m).data.set(DataKey::Bestow, Val::List(vec![]));
    Ok(())
}

// ======================================================== Elvish Spirit Guide; special mana units
/// partials.hand_mana: mana from cards in hand (Elvish Spirit Guide: exiled for {G})
pub fn hand_mana(g: &Game, p: PlayerId) -> Vec<Unit> {
    g.player(p)
        .hand
        .iter()
        .filter(|&&c| &*g.db.get(c).name == "Elvish Spirit Guide")
        .map(|&c| Unit { src: Source::HandCard(c), cols: Colors::from_letters("G"), amt: 1 })
        .collect()
}

/// partials.special_unit_paid: a special mana unit was used: pay its cost (a card from hand is exiled, a Scion or
/// Spawn sacrificed). Quirion Ranger's and Wirewood Symbiote's units (phase 6) aren't made yet.
pub fn special_unit_paid(g: &mut Game, p: PlayerId, u: Unit) -> Res {
    match u.src {
        Source::HandCard(c) => {
            if remove_card(&mut g.player_mut(p).hand, c) {
                g.player_mut(p).exile.push(c);
            }
        }
        Source::Scion(m) => {
            if on(g, p, m) {
                leave(g, m)?;
                if !g.hooks.is_empty() {
                    fire_trigger(g, Event::Sacrifice, crate::hooks::Call::Sacrifice { p, what: Sacrificed::Perm(m) })?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

// ======================================================== Golgari Charm (regeneration)
/// partials.regen_wipe: a destroy wipe is coming: Golgari Charm regenerates p's creatures. PORT(phase 6): Golgari
/// Charm's modes (its resolve hook and `ctx['mode']`); no Tier 1 or Sauron deck plays it, so until then nothing
/// answers a wipe this way.
pub fn regen_wipe(_g: &mut Game, _p: PlayerId) -> Res<bool> {
    Ok(false)
}

// ======================================================== Gryff's Boon: recursion
/// {3}{W}: returns from the graveyard onto a creature
fn gryff_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || !can_pay(g, p, 3, "W", false) || !g.player(p).perms.iter().any(|&m| g.is_creature(m)) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.8,
        label: "Gryff's Boon (graveyard)".into(),
        act: Some(Action::Plan { f: gryff_go, arg: c.0 as i64 }),
    }])
}

fn gryff_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).gy.contains(&c) || !can_pay(g, p, 3, "W", false) {
        return Ok(false);
    }
    pay(g, p, 3, "W", false)?;
    crate::glog!(g, "  {} returns Gryff's Boon from the graveyard", pname(g, p));
    if ability_window_card(g, p, c, "return to the battlefield")? && remove_card(&mut g.player_mut(p).gy, c) {
        enter(g, p, c, Enter::default())?;
    }
    Ok(true)
}

// ======================================================== Heartless Act
/// destroys a creature with no counters, or removes up to three counters (and kills it if that is lethal)
fn heartless(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) else { return Ok("gy") };
    if g.perm(t).plus > 0 {
        let x = g.perm_mut(t);
        x.plus = (x.plus - 3).max(0);
        crate::glog!(g, "    Heartless Act removes counters from {}", g.perm(t).name);
        if etgh(g, t) <= 0 {
            die(g, t, "sba")?;
        }
    } else {
        apply_removal(g, Some(p), t, "destroy", Some(c))?;
    }
    Ok("gy")
}

// ======================================================== Hero of Iroas: heroic
/// heroic: a +1/+1 counter when an Aura of yours enters attached to it
fn hero_heroic(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    let aura = g.perm(m).cd.is_some_and(|c| g.db.get(c).has_subtype("aura"));
    if aura
        && g.perm(m).owner == o
        && g.perm(m).attached == Some(src)
        && trigger_window(g, o, Some(src), "a +1/+1 counter", Some(2.0))?
    {
        g.perm_mut(src).plus += 1;
    }
    Ok(())
}

// ======================================================== Hunter's Insight
/// partials.insight_draw: the creature Hunter's Insight was cast on deals combat damage to a player: draw that many.
/// PORT(phase 6): Hunter's Insight (no Tier 1 or Sauron deck plays it) sets `p.insight`, which has no Rust field yet,
/// so this draws nothing until then.
pub fn insight_draw(_g: &mut Game, _p: PlayerId, _a: PermId, _dmg: i32) -> Res {
    Ok(())
}

// ======================================================== Jagged-Scar Archers
/// {T}: damage equal to its power to a flier
fn archers_options(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || g.perm(src).tapped || g.perm(src).sick {
        return Ok(vec![]);
    }
    let n = epow(g, src);
    let t = best_opp_creature(g, p, |g, m| (g.perm(m).fly || crate::dsl::has_kw(g, m, "flying")) && etgh(g, m) <= n);
    let Some(t) = t.filter(|&t| pval(g, t) >= 2.5) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: pval(g, t) - 1.0,
        label: format!("Jagged-Scar Archers -> {}", g.perm(t).name),
        act: Some(Action::Ability { src, f: archers_go, arg: pack(t.0, n.max(0) as u32) }),
    }])
}

fn archers_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (t, n) = unpack(arg);
    let t = PermId(t);
    if g.perm(src).tapped || !g.perm(t).on_bf {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    let label = format!("{n} damage to {}", g.perm(t).name);
    if ability_window(g, p, Some(src), &label, None, Some(t))? && g.perm(t).on_bf {
        apply_removal(g, Some(p), t, &format!("dmg{n}"), None)?;
    }
    Ok(true)
}

// ======================================================== Jaxis, the Troublemaker
/// blitz {1}{R}: haste, draw when it dies, sacrificed at end of turn
fn jaxis_blitz(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || !can_pay(g, p, 1, "R", false) || !castable(g, p, c, "hand") {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 2.2,
        label: "blitz Jaxis".into(),
        act: Some(Action::Plan { f: jaxis_blitz_go, arg: c.0 as i64 }),
    }])
}

fn jaxis_blitz_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "R", false) {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    pay(g, p, 1, "R", false)?;
    crate::glog!(g, "  {} casts Jaxis for its blitz cost", pname(g, p));
    on_cast(g, p, c)?;
    if !counter_window(g, p, c, 3.0, vec![])? {
        g.player_mut(p).gy.push(c);
        return Ok(true);
    }
    let m = enter(g, p, c, Enter { was_cast: true, ..Enter::default() })?;
    let x = g.perm_mut(m);
    x.sick = false;
    x.data.set(DataKey::Blitz, Val::Bool(true));
    Ok(true)
}

/// blitzed: draw a card when it dies
fn jaxis_draw(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    let o = g.perm(m).owner;
    if g.perm(m).data.truthy(DataKey::Blitz) && trigger_window(g, o, Some(m), "draw a card", None)? {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

/// the end step: the blitzed Jaxis is sacrificed; its token copies leave, each drawing a card
fn jaxis_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner != p {
        return Ok(());
    }
    let copies = |g: &Game| -> Vec<PermId> {
        let o = g.perm(src).owner;
        g.player(o).perms.iter().copied().filter(|&m| g.perm(m).data.truthy(DataKey::JaxisCopy)).collect()
    };
    let blitz = g.perm(src).data.truthy(DataKey::Blitz) && on(g, p, src);
    if !blitz && copies(g).is_empty() {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "sacrifice at end of turn", Some(2.0))? {
        return Ok(());
    }
    if blitz && on(g, p, src) {
        die(g, src, "sac")?;
    }
    for m in copies(g) {
        let o = g.perm(src).owner;
        draw(g, o, 1, false)?;
        leave(g, m)?;
    }
    Ok(())
}

/// partials.etb_val: what entering again is worth for m's card
fn etb_val(g: &Game, m: PermId) -> f64 {
    let Some(c) = g.perm(m).cd else { return 0.0 };
    let d = g.db.get(c);
    if d.has_dsl() {
        crate::dsl::etb_value(g, c) as f64
    } else if d.tag(Tag::Etb) {
        2.0
    } else {
        0.0
    }
}

/// {R}, {T}, discard a card: a hasty token copy of another creature (draw when it dies, sacrificed at end step)
fn jaxis_copy(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if x.owner != p || post != Some(false) || x.tapped || x.sick || !can_pay(g, p, 0, "R", false) {
        return Ok(vec![]);
    }
    let junk = jaxis_junk(g, p);
    let cs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && m != src && g.perm(m).cd.is_some_and(|c| !g.db.get(c).tag(Tag::Leg)))
        .collect();
    if junk.is_empty() || cs.is_empty() {
        return Ok(vec![]);
    }
    let t = max_by(&cs, |m| etb_val(g, m) + epow(g, m) as f64 / 2.0).unwrap();
    Ok(vec![Opt {
        utility: 1.5 + etb_val(g, t) / 2.0,
        label: format!("Jaxis copies {}", g.perm(t).name),
        act: Some(Action::Ability { src, f: jaxis_copy_go, arg: t.0 as i64 }),
    }])
}

fn jaxis_junk(g: &Game, p: PlayerId) -> Vec<CardId> {
    g.player(p).hand.iter().copied().filter(|&c| card_worth(g, p, c, false) < 30.0).collect()
}

fn jaxis_copy_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    let junk = jaxis_junk(g, p);
    if g.perm(src).tapped || !on(g, p, t) || !can_pay(g, p, 0, "R", false) || junk.is_empty() {
        return Ok(false);
    }
    pay(g, p, 0, "R", false)?;
    g.perm_mut(src).tapped = true;
    let x = min_by(&junk, |c| card_worth(g, p, c, false)).unwrap();
    remove_card(&mut g.player_mut(p).hand, x);
    g.player_mut(p).gy.push(x);
    let label = format!("copy {}", g.perm(t).name);
    if !ability_window(g, p, Some(src), &label, None, Some(t))? || !on(g, p, t) {
        return Ok(true);
    }
    let cd = g.perm(t).cd.unwrap();
    if let Some(tok) = enter_token_copy(g, p, cd)? {
        let y = g.perm_mut(tok);
        y.sick = false;
        y.data.set(DataKey::JaxisCopy, Val::Bool(true));
    }
    Ok(true)
}

// ======================================================== Kaya's Wrath
/// destroys all creatures; gain life for each of yours destroyed
fn kayas_wrath(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mine: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect();
    for q in 0..g.players.len() {
        for m in g.players[q].perms.clone() {
            if g.is_creature(m) && !g.perm(m).phased {
                die(g, m, "destroy")?;
            }
        }
    }
    let n = mine.iter().filter(|&&m| !on(g, p, m)).count() as i32;
    gain(g, p, n)?;
    Ok("gy")
}

// ======================================================== Mardu Charm
/// 4 damage to a creature (removal), or two first-strike Warriors at end of turn (the discard mode: Python's
/// `ctx['mode'] == 'discard'`, which nothing sets)
fn mardu_charm(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) {
        apply_removal(g, Some(p), t, "dmg4", Some(c))?;
        return Ok("gy");
    }
    let spec =
        Tokens { warrior: true, color: Some(Colors::from_letters("W")), types: vec!["warrior"], ..Tokens::new(2, 1) };
    for m in make_tokens(g, p, spec)? {
        eot_kw(g, m, "first strike");
    }
    Ok("gy")
}

fn mardu_eot(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || !can_pay(g, p, 0, "RWB", false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.2,
        label: "Mardu Charm (two Warriors)".into(),
        act: Some(Action::Plan { f: mardu_go, arg: c.0 as i64 }),
    }])
}

fn mardu_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 0, "RWB", false) {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    pay(g, p, 0, "RWB", false)?;
    cast_card(g, p, c, "lib", Ctx::default())?;
    Ok(true)
}

// ======================================================== Pia Nalaar
/// a 1/1 flying Thopter on entry
fn pia_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && trigger_window(g, o, Some(src), "create a 1/1 Thopter", None)? {
        let spec = Tokens { fly: true, types: vec!["thopter", "artifact"], ..Tokens::new(1, 1) };
        make_tokens(g, o, spec)?;
    }
    Ok(())
}

fn pia_arts(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            (x.token && x.ttypes.contains(&"artifact")) || (of_type(g, m, Types::ARTIFACT) && pval(g, m) < 2.0)
        })
        .collect()
}

/// {1}, sacrifice an artifact: target creature can't block (the biggest blocker, before a big attack)
fn pia_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || post != Some(false) || !can_pay(g, p, 1, "", false) {
        return Ok(vec![]);
    }
    if pia_arts(g, p).is_empty() && g.player(p).treasures < 1 {
        return Ok(vec![]);
    }
    let ready: i32 = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| g.is_creature(m) && !g.perm(m).sick && !g.perm(m).tapped)
        .map(|&m| epow(g, m))
        .sum();
    let b = best_opp_creature(g, p, |g, m| !g.perm(m).tapped && epow(g, m) >= 3);
    let Some(b) = b.filter(|_| ready >= 6) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: 1.0,
        label: "Pia Nalaar (can't block)".into(),
        act: Some(Action::Ability { src, f: pia_go, arg: b.0 as i64 }),
    }])
}

fn pia_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let b = PermId(arg as u32);
    if !can_pay(g, p, 1, "", false) || !g.perm(b).on_bf {
        return Ok(false);
    }
    let arts = pia_arts(g, p);
    pay(g, p, 1, "", false)?;
    if let Some(&a) = arts.first().filter(|&&a| on(g, p, a)) {
        die(g, a, "sac")?;
    } else if g.player(p).treasures > 0 {
        g.player_mut(p).treasures -= 1;
    } else {
        return Ok(false);
    }
    let label = format!("{} can't block", g.perm(b).name);
    if !ability_window(g, p, Some(src), &label, None, Some(b))? {
        return Ok(true);
    }
    eot_kw(g, b, "cant_block");
    crate::glog!(g, "  {} uses Pia Nalaar: {} can't block", pname(g, p), g.perm(b).name);
    Ok(true)
}

// ======================================================== Rabble Rousing: hideaway 5
fn rabble_hide(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if g.player(o).library.is_empty() || !trigger_window(g, o, Some(src), "hideaway 5", None)? {
        return Ok(());
    }
    let mut top = vec![];
    for _ in 0..g.player(o).library.len().min(5) {
        top.push(g.player_mut(o).library.pop().unwrap());
    }
    if top.is_empty() {
        return Ok(());
    }
    let c = first_max(&top, |c| {
        let d = g.db.get(c);
        (!d.land, d.cmc, card_worth(g, o, c, false))
    })
    .unwrap();
    remove_card(&mut top, c);
    g.rng.shuffle(&mut top);
    let lib = &mut g.player_mut(o).library;
    lib.splice(0..0, top);
    g.perm_mut(src).data.set(DataKey::Hidden, Val::Card(c));
    Ok(())
}

/// with ten creatures, the hidden card is played for free
fn rabble_play(g: &mut Game, src: Src, p: PlayerId, _atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    let hidden = |g: &Game| matches!(g.perm(src).data.get(DataKey::Hidden), Some(Val::Card(_)));
    if g.perm(src).owner != p || !hidden(g) {
        return Ok(vec![]);
    }
    if g.player(p).perms.iter().filter(|&&m| g.is_creature(m)).count() >= 10 {
        if !trigger_window(g, p, Some(src), "play the hidden card", Some(5.0))? || !hidden(g) {
            return Ok(vec![]);
        }
        let Some(Val::Card(c)) = g.perm_mut(src).data.remove(DataKey::Hidden) else { return Ok(vec![]) };
        crate::glog!(g, "    Rabble Rousing plays the hidden {}", g.db.get(c).name);
        if g.db.get(c).land {
            g.add_land(p, c, false);
        } else {
            cast_card(g, p, c, "lib", Ctx::default())?;
        }
    }
    Ok(vec![])
}

// ======================================================== Rapid Hybridization
/// destroys a creature; its controller gets a 3/3 Frog Lizard
fn hybrid(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) else { return Ok("gy") };
    let q = g.perm(t).owner;
    apply_removal(g, Some(p), t, "destroy", Some(c))?;
    if !on(g, q, t) {
        let spec =
            Tokens { color: Some(Colors::from_letters("G")), types: vec!["frog", "lizard"], ..Tokens::new(1, 3) };
        make_tokens(g, q, spec)?;
    }
    Ok("gy")
}

// ======================================================== Reforge the Soul: miracle
/// partials.miracle_options: Reforge the Soul for its miracle cost when it was this turn's first card drawn (the
/// coordinator wires it into `cardcode::pool_card_options`)
pub fn miracle_options(g: &mut Game, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    let Some((st, c)) = g.player(p).miracle else { return Ok(vec![]) };
    if st != g.turn_stamp()
        || !g.player(p).hand.contains(&c)
        || &*g.db.get(c).name != "Reforge the Soul"
        || !can_pay(g, p, 1, "R", false)
    {
        return Ok(vec![]);
    }
    if g.player(p).hand.len() > 4 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 3.0,
        label: "Reforge the Soul (miracle)".into(),
        act: Some(Action::Plan { f: miracle_go, arg: c.0 as i64 }),
    }])
}

fn miracle_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "R", false) {
        return Ok(false);
    }
    pay(g, p, 1, "R", false)?;
    g.player_mut(p).miracle = None;
    crate::glog!(g, "  {} casts Reforge the Soul for its miracle cost", pname(g, p));
    cast_card(g, p, c, "hand", Ctx::default())?;
    Ok(true)
}

// ======================================================== Retreat to Coralhelm
/// landfall: untaps a tapped mana creature, else scry 1
fn coralhelm(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner != p {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "untap a creature or scry 1", Some(2.0))? {
        return Ok(());
    }
    let tapped: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && g.perm(m).tapped && g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Dork)))
        .collect();
    if let Some(m) = max_by(&tapped, |m| cardcode::dyn_mana_perm(g, p, m).unwrap_or(1) as f64) {
        g.perm_mut(m).tapped = false;
    } else {
        cardcode::scry(g, p, 1, false)?;
    }
    Ok(())
}

// ======================================================== Sentinel's Eyes: escape
/// escape {W} and two other graveyard cards
fn eyes_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || !can_pay(g, p, 0, "W", false) || !g.player(p).perms.iter().any(|&m| g.is_creature(m)) {
        return Ok(vec![]);
    }
    if g.player(p).gy.iter().filter(|&&x| x != c).count() < 2 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.5,
        label: "escape Sentinel's Eyes".into(),
        act: Some(Action::Plan { f: eyes_go, arg: c.0 as i64 }),
    }])
}

/// exile the n least useful other cards from p's graveyard (escape's cost)
pub(crate) fn exile_fodder(g: &mut Game, p: PlayerId, c: CardId, n: usize) {
    let mut fod: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&x| x != c).collect();
    let worth: Vec<f64> = fod.iter().map(|&x| card_worth(g, p, x, true)).collect();
    let mut idx: Vec<usize> = (0..fod.len()).collect();
    idx.sort_by(|&a, &b| worth[a].partial_cmp(&worth[b]).unwrap_or(std::cmp::Ordering::Equal)); // stable
    fod = idx.into_iter().map(|i| fod[i]).collect();
    for x in fod.into_iter().take(n) {
        let pl = g.player_mut(p);
        remove_card(&mut pl.gy, x);
        pl.exile.push(x);
    }
}

fn eyes_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).gy.contains(&c) || !can_pay(g, p, 0, "W", false) {
        return Ok(false);
    }
    pay(g, p, 0, "W", false)?;
    exile_fodder(g, p, c, 2);
    remove_card(&mut g.player_mut(p).gy, c);
    crate::glog!(g, "  {} escapes Sentinel's Eyes", pname(g, p));
    on_cast(g, p, c)?;
    if counter_window(g, p, c, 3.0, vec![])? {
        enter(g, p, c, Enter::default())?;
    } else {
        g.player_mut(p).exile.push(c);
    }
    Ok(true)
}

// ======================================================== Sigarda's Aid
/// Equipment entering under your control attaches to your best creature
fn aid_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    let equip = g.perm(m).cd.is_some_and(|c| g.db.get(c).has_subtype("equipment"));
    if g.perm(m).owner != o || !equip || m == src {
        return Ok(());
    }
    let creatures = |g: &Game| -> Vec<PermId> {
        g.player(o).perms.iter().copied().filter(|&x| g.is_creature(x) && !g.perm(x).phased).collect()
    };
    if creatures(g).is_empty() {
        return Ok(());
    }
    let label = format!("attach {}", g.perm(m).name);
    if !trigger_window(g, o, Some(src), &label, Some(2.0))? || !g.perm(m).on_bf {
        return Ok(());
    }
    let cr = creatures(g);
    if let Some(x) = first_max(&cr, |x| (g.perm(x).is_cmd, pval(g, x))) {
        g.perm_mut(m).attached = Some(x);
    }
    Ok(())
}

/// partials.aid_active: Sigarda's Aid lets p cast Auras at instant speed
pub fn aid_active(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| !g.perm(m).phased && name_of(g, m) == "Sigarda's Aid")
}

// ======================================================== Starfield Mystic
/// a +1/+1 counter when one of your enchantments goes to the graveyard
fn mystic_grow(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let o = g.perm(src).owner;
    if m != src
        && g.perm(m).owner == o
        && of_type(g, m, Types::ENCHANTMENT)
        && trigger_window(g, o, Some(src), "a +1/+1 counter", Some(2.0))?
    {
        g.perm_mut(src).plus += 1;
    }
    Ok(())
}

// ======================================================== Sunfall
/// exiles all creatures; an Incubator with that many counters
fn sunfall(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mut n = 0;
    for q in 0..g.players.len() {
        for m in g.players[q].perms.clone() {
            if g.is_creature(m) && !g.perm(m).phased {
                exile_perm(g, m)?;
                n += 1;
            }
        }
    }
    if n > 0 {
        g.player_mut(p).incubator += n;
        crate::glog!(g, "    {} incubates {}", pname(g, p), n);
    }
    Ok("gy")
}

/// partials.incubator_options: {2}: the Incubator transforms into an X/X Phyrexian artifact creature (the
/// coordinator wires it into `cardcode::pool_card_options`)
pub fn incubator_options(g: &mut Game, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    let n = g.player(p).incubator;
    if n == 0 || !can_pay(g, p, 2, "", false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.0 + n as f64 / 2.0,
        label: "transform Incubator".into(),
        act: Some(Action::Plan { f: incubate_go, arg: 0 }),
    }])
}

fn incubate_go(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    let n = g.player(p).incubator;
    if !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    g.player_mut(p).incubator = 0;
    make_tokens(g, p, Tokens { types: vec!["phyrexian", "artifact"], ..Tokens::new(1, n) })?;
    crate::glog!(g, "  {} transforms the Incubator ({}/{})", pname(g, p), n, n);
    Ok(true)
}

// ======================================================== Tajic, Legion's Edge
/// {R}{W}: first strike when it attacks (with the mana to spare)
fn tajic_fs(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == p && atk.contains(&src) && can_pay(g, p, 0, "RW", false) && total_mana(g, p, false) >= 4 {
        pay(g, p, 0, "RW", false)?;
        eot_kw(g, src, "first strike");
    }
    Ok(vec![])
}

// ======================================================== Tishana's Tidebinder
/// partials.tidebinder_response: an opponent's creature with a strong enters trigger enters: an outside deck holding
/// Tidebinder counters the trigger (the creature loses its abilities while Tidebinder stays)
pub fn tidebinder_response(g: &mut Game, p: PlayerId, m: PermId) -> Res<bool> {
    let v = if g.perm(m).cd.is_some() { etb_val(g, m) } else { 0.0 };
    if v < 3.0 && !g.perm(m).cd.is_some_and(|c| g.db.get(c).bomb >= 7) {
        return Ok(false);
    }
    for q in g.after(p).collect::<Vec<_>>() {
        if g.humans.get(q).is_some() {
            continue; // HUMAN(phase 9): a person casts their own Tidebinder
        }
        if q == p || !g.player(q).alive || crate::ai::is_main(g.player(q).key) {
            continue;
        }
        let Some(c) = g.player(q).hand.iter().copied().find(|&x| &*g.db.get(x).name == "Tishana's Tidebinder") else {
            continue;
        };
        if !castable(g, q, c, "hand") || !can_pay_cost(g, q, c) {
            continue;
        }
        pay_cost(g, q, c)?;
        remove_card(&mut g.player_mut(q).hand, c);
        crate::glog!(
            g,
            "    {} flashes in Tishana's Tidebinder: {}'s trigger is countered",
            pname(g, q),
            g.perm(m).name
        );
        on_cast(g, q, c)?;
        if !counter_window(g, q, c, 4.0, vec![])? {
            g.player_mut(q).gy.push(c);
            return Ok(false);
        }
        let tb = enter(g, q, c, Enter { was_cast: true, ..Enter::default() })?;
        g.perm_mut(tb).data.set(DataKey::Bound, Val::Perm(m));
        g.perm_mut(m).neutered = true;
        return Ok(true);
    }
    Ok(false)
}

/// partials.answer_ability: the AI q with priority and an opponent's important ability (or trigger) on top of the
/// stack: counter it with Azorius Guildmage ({2}{U}) or by flashing in Tishana's Tidebinder (its source then loses its
/// abilities). `item` is the stack item's id.
pub fn answer_ability(g: &mut Game, q: PlayerId, item: u32) -> Res {
    let Some(it) = g.stack.iter().find(|x| x.id == item).cloned() else { return Ok(()) };
    let gm = g.player(q).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        !x.phased && !x.neutered && name_of(g, m) == "Azorius Guildmage"
    });
    if let Some(gm) = gm
        && it.kind == StackKind::Ability
        && can_pay(g, q, 2, "U", false)
    {
        pay(g, q, 2, "U", false)?;
        crate::glog!(g, "  {} activates Azorius Guildmage: counters {}", pname(g, q), it.name);
        if ability_window(g, q, Some(gm), "counter target activated ability", Some(it.imp), None)?
            && let Some(x) = g.stack.iter_mut().find(|x| x.id == item)
        {
            x.countered = true;
        }
        return Ok(());
    }
    let Some(c) = g.player(q).hand.iter().copied().find(|&x| &*g.db.get(x).name == "Tishana's Tidebinder") else {
        return Ok(());
    };
    if g.humans.get(q).is_some() || !castable(g, q, c, "hand") || !can_pay_cost(g, q, c) {
        return Ok(());
    }
    pay_cost(g, q, c)?;
    remove_card(&mut g.player_mut(q).hand, c);
    crate::glog!(g, "    {} flashes in Tishana's Tidebinder: {} is countered", pname(g, q), it.name);
    on_cast(g, q, c)?;
    if !counter_window(g, q, c, 4.0, vec![])? {
        g.player_mut(q).gy.push(c);
        return Ok(());
    }
    let tb = enter(g, q, c, Enter { was_cast: true, ..Enter::default() })?;
    if let Some(x) = g.stack.iter_mut().find(|x| x.id == item) {
        x.countered = true;
        if let Some(src) = it.ctx.source
            && g.perm(src).on_bf
            && g.perm(src).cd.is_some_and(|cd| {
                let t = g.db.get(cd).types;
                g.is_creature(src) || t.has(Types::ARTIFACT) || t.has(Types::PLANESWALKER)
            })
        {
            g.perm_mut(tb).data.set(DataKey::Bound, Val::Perm(src));
            g.perm_mut(src).neutered = true;
        }
    }
    Ok(())
}

/// the bound permanent gets its abilities back when Tidebinder leaves. HUMAN(phase 9): a person's Tidebinder
/// counters an ability on the stack as it enters (its enters hook does nothing for the AI).
fn tide_free(g: &mut Game, _src: Src, m: PermId) -> Res {
    if let Some(&Val::Perm(b)) = g.perm(m).data.get(DataKey::Bound) {
        g.perm_mut(b).neutered = false;
    }
    Ok(())
}

// ======================================================== Tyvar the Bellicose: counters on mana creatures
/// creatures that tap for mana get that many +1/+1 counters (once each turn)
fn tyvar(g: &mut Game, src: Src, p: PlayerId, m: PermId, amt: u32) -> Res {
    if g.perm(src).owner == p && g.is_creature(m) && once_per_turn(g, p, intern(&format!("tyvar{}", m.0))) {
        g.perm_mut(m).plus += amt as i32;
    }
    Ok(())
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Brutal Hordechief")?;
    c.options = Some(hordechief_options);
    c.blocks = Some(hordechief_blocks);
    r.card(db, "Chaos Warp")?.resolve = Some(chaos_warp);
    let c = r.card(db, "Druid Class")?;
    c.options = Some(druid3_options);
    c.upkeep = Some(druid_size);
    at_once(c, Event::Upkeep, true);
    r.card(db, "Eidolon of Countless Battles")?.hand_options = Some(bestow_options);
    r.card(db, "Gryff's Boon")?.gy_options = Some(gryff_options);
    r.card(db, "Heartless Act")?.resolve = Some(heartless);
    let c = r.card(db, "Hero of Iroas")?;
    c.etb = Some(hero_heroic);
    at_once(c, Event::Etb, false);
    r.card(db, "Jagged-Scar Archers")?.options = Some(archers_options);
    let c = r.card(db, "Jaxis, the Troublemaker")?;
    c.hand_options = Some(jaxis_blitz);
    c.self_dies = Some(jaxis_draw);
    at_once(c, Event::SelfDies, false);
    c.end_step = Some(jaxis_end);
    at_once(c, Event::EndStep, false);
    c.options = Some(jaxis_copy);
    r.card(db, "Kaya's Wrath")?.resolve = Some(kayas_wrath);
    let c = r.card(db, "Mardu Charm")?;
    c.resolve = Some(mardu_charm);
    c.hand_options = Some(mardu_eot);
    let c = r.card(db, "Pia Nalaar")?;
    c.etb = Some(pia_etb);
    at_once(c, Event::Etb, false);
    c.options = Some(pia_options);
    let c = r.card(db, "Rabble Rousing")?;
    c.etb = Some(rabble_hide);
    at_once(c, Event::Etb, false);
    c.attack = Some(rabble_play);
    at_once(c, Event::Attack, false);
    r.card(db, "Rapid Hybridization")?.resolve = Some(hybrid);
    let c = r.card(db, "Retreat to Coralhelm")?;
    c.landfall = Some(coralhelm);
    at_once(c, Event::Landfall, false);
    r.card(db, "Sentinel's Eyes")?.gy_options = Some(eyes_options);
    let c = r.card(db, "Sigarda's Aid")?;
    c.etb = Some(aid_etb);
    at_once(c, Event::Etb, false);
    let c = r.card(db, "Starfield Mystic")?;
    c.dies = Some(mystic_grow);
    at_once(c, Event::Dies, false);
    r.card(db, "Sunfall")?.resolve = Some(sunfall);
    let c = r.card(db, "Tajic, Legion's Edge")?;
    c.attack = Some(tajic_fs);
    at_once(c, Event::Attack, true);
    let c = r.card(db, "Tishana's Tidebinder")?;
    c.leaves = Some(tide_free);
    at_once(c, Event::Leaves, true);
    r.card(db, "Tyvar the Bellicose")?.mana_tapped = Some(tyvar);
    Ok(())
}
