//! Python's `cards/impl/partials.py`: the abilities the audit listed as missing, implemented from their rules text
//! (the Tier 1 and Sauron cards for M5, the rest in phase 6: `register_phase6`).

use super::common;
use crate::cardcode;
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{cast_card, castable, casts_this_turn, on_cast};
use crate::engine::hooks::fire_trigger;
use crate::engine::life::{check_state, gain, lose_life};
use crate::engine::mana::{Source, Unit, can_pay, cost_of, pay, total_mana};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{abilities_answered, ability_window, counter_window, stack_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{epow, etgh, has_type, indestructible, once_per_turn, pval, threat, untargetable};
use crate::engine::zones::{
    Enter, Tokens, bounce, die, draw, enter, enter_token_copy, exile_perm, leave, make_tokens, max_by, min_by,
};
use crate::flow::Res;
use crate::hooks::{Action, Call, CardImpl, Event, Opt, Registry, Sacrificed, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::pysum::PySum;
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

/// common.best_opp_creature: the most valuable targetable creature of p's opponents that passes `pred`
pub(crate) fn best_opp_creature(g: &Game, p: PlayerId, pred: impl Fn(&Game, PermId) -> bool) -> Option<PermId> {
    common::best_opp_creature(g, p, |m| pred(g, m))
}

/// common.best_opp_nonland: the most valuable targetable permanent of p's opponents that passes `pred`
pub(crate) fn best_opp_nonland(g: &Game, p: PlayerId, pred: impl Fn(&Game, PermId) -> bool) -> Option<PermId> {
    common::best_opp_nonland(g, p, |m| pred(g, m))
}

/// common.auras_on: the Auras on m that the card code tracks
pub(crate) fn auras_on(g: &Game, m: PermId) -> Vec<PermId> {
    common::auras_on(g, m)
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
/// level 3 ({4}{G}): a land becomes a creature with power and toughness equal to your lands.
///
/// Bug fix (phase 6): Python's hook replaces t1's level-2 option and offers level 3 only from level 2, so Druid
/// Class never left level 1 (no extra land drop, no level 3). Level 2 ({2}{G}, as t1's) is offered here too.
fn druid3_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let level = g.perm(src).data.get(DataKey::Level).map_or(1, Val::int);
    if g.perm(src).owner != p || post != Some(false) {
        return Ok(vec![]);
    }
    if level < 2 {
        if !can_pay(g, p, 2, "G", false) {
            return Ok(vec![]);
        }
        return Ok(vec![opt_ability(2.0, "Druid Class level 2".into(), src, druid2_go, 0)]);
    }
    if level != 2 || !can_pay(g, p, 4, "G", false) {
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

/// bestow {2}{W}{W}: an Aura on the best creature (the commander in Light-Paws), +X/+X for creatures and Auras
fn bestow_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || !can_pay(g, p, 2, "WW", false) {
        return Ok(vec![]);
    }
    let Some(host) = common::own_host(g, p, &common::AURA, None) else { return Ok(vec![]) };
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
/// Spawn sacrificed, Quirion Ranger returns a Forest, Wirewood Symbiote an Elf)
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
        Source::Quirion(src, _dork) => {
            let now = turn_now(g);
            g.perm_mut(src).data.set(DataKey::Used, now);
            let forest = g.player(p).lands.iter().copied().find(|&l| &*g.db.get(g.land(l).cd).name == "Forest");
            if let Some(l) = forest {
                crate::engine::turn::remove_land(g, p, l);
                let cd = g.land(l).cd;
                g.player_mut(p).hand.push(cd);
            }
        }
        Source::Symbiote(src, _dork, elf) => {
            let now = turn_now(g);
            g.perm_mut(src).data.set(DataKey::Used, now);
            if on(g, p, elf) {
                bounce(g, elf)?;
            }
        }
        _ => {}
    }
    Ok(())
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
/// 4 damage to a creature (removal), two first-strike Warriors at end of turn, or a noncreature card discarded
fn mardu_charm(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) {
        apply_removal(g, Some(p), t, "dmg4", Some(c))?;
        return Ok("gy");
    }
    if ctx.mode == Some("discard") {
        let os: Vec<PlayerId> = g.opps(p).collect();
        let q = max_by(&os, |q| threat(g, p, q)).unwrap();
        let cs: Vec<CardId> =
            g.player(q).hand.iter().copied().filter(|&x| !g.db.get(x).creature && !g.db.get(x).land).collect();
        if let Some(x) = max_by(&cs, |x| card_worth(g, q, x, false)) {
            let pl = g.player_mut(q);
            remove_card(&mut pl.hand, x);
            pl.gy.push(x);
            crate::glog!(g, "    {} discards {}", pname(g, q), g.db.get(x).name);
        }
        return Ok("gy");
    }
    let spec =
        Tokens { warrior: true, color: Some(Colors::from_letters("W")), types: vec!["warrior"], ..Tokens::new(2, 1) };
    for m in make_tokens(g, p, spec)? {
        eot_kw(g, m, "first strike");
    }
    Ok("gy")
}

/// Bug fix (phase 6): Python's resolve reads `ctx['mode']` for the discard mode, which nothing sets, so the charm
/// always made Warriors. At end of turn the AI now picks a mode: the Warriors (1.2), or the discard when the most
/// threatening opponent holds enough cards for it to be worth more (0.3 a card: five or more).
fn mardu_eot(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || !can_pay(g, p, 0, "RWB", false) {
        return Ok(vec![]);
    }
    let disc = mardu_discard_value(g, p);
    if disc > 1.2 {
        return Ok(vec![opt_plan(disc, "Mardu Charm (discard)".into(), mardu_go, (c.0 as i64) | (1 << 16))]);
    }
    Ok(vec![opt_plan(1.2, "Mardu Charm (two Warriors)".into(), mardu_go, c.0 as i64)])
}

fn mardu_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId((arg & 0xffff) as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 0, "RWB", false) {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    pay(g, p, 0, "RWB", false)?;
    let mode = if arg >> 16 == 1 { Some("discard") } else { None };
    cast_card(g, p, c, "lib", Ctx { mode, ..Ctx::default() })?;
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

// ======================================================== phase 6: the rest of partials.py
fn opt_ability(utility: f64, label: String, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Ability { src, f, arg }) }
}

fn opt_plan(utility: f64, label: String, f: crate::hooks::PlanFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Plan { f, arg }) }
}

/// cardimpl.count_type: p's permanents (not phased out) of subtype t
fn count_type(g: &Game, p: PlayerId, t: &str) -> usize {
    g.player(p).perms.iter().filter(|&&m| !g.perm(m).phased && has_type(g, m, t)).count()
}

fn creatures_of(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m)).collect()
}

fn opp_creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.opps(p).flat_map(|q| g.player(q).perms.iter().copied()).filter(|&m| g.is_creature(m)).collect()
}

fn data_val(g: &Game, m: PermId, k: DataKey) -> Option<&Val> {
    g.perm(m).data.get(k)
}

/// partials.deal_noncombat: n damage to creature m from a non-combat source (Tajic prevents it for the other
/// creatures of its owner)
pub fn deal_noncombat(g: &mut Game, _src_owner: PlayerId, m: PermId, n: i32) -> Res {
    if !g.perm(m).on_bf {
        return Ok(());
    }
    if tajic_protects(g, m) {
        crate::glog!(g, "    damage to {} is prevented (Tajic)", g.perm(m).name);
        return Ok(());
    }
    if etgh(g, m) <= n && !indestructible(g, m) {
        die(g, m, "destroy")?;
    }
    Ok(())
}

// ======================================================== Archangel Avacyn
/// Full: transforms at the next upkeep after a non-Angel creature of yours dies
fn avacyn_flag(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let o = g.perm(src).owner;
    if m != src
        && g.perm(m).owner == o
        && g.is_creature(m)
        && !has_type(g, m, "angel")
        && !g.perm(src).data.truthy(DataKey::Flipped)
    {
        if !trigger_window(g, o, Some(src), "transform at the next upkeep", Some(2.0))? {
            return Ok(());
        }
        g.perm_mut(src).data.set(DataKey::Flip, Val::Bool(true));
    }
    Ok(())
}

/// 3 damage to each other creature and each opponent; 6/5
fn avacyn_flip(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    let d = &g.perm(src).data;
    if !d.truthy(DataKey::Flip) || d.truthy(DataKey::Flipped) {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if !trigger_window(g, o, Some(src), "transform: 3 damage to each other creature and each opponent", Some(6.0))? {
        g.perm_mut(src).data.set(DataKey::Flip, Val::Bool(false));
        return Ok(());
    }
    if !on(g, o, src) {
        return Ok(());
    }
    let x = g.perm_mut(src);
    x.data.set(DataKey::Flip, Val::Bool(false));
    x.data.set(DataKey::Flipped, Val::Bool(true));
    x.pow = 6;
    x.tgh = 5;
    crate::glog!(g, "    Archangel Avacyn transforms: 3 damage to each other creature and each opponent");
    for q in 0..g.players.len() {
        for m in g.players[q].perms.clone() {
            if m != src && g.is_creature(m) {
                deal_noncombat(g, o, m, 3)?;
            }
        }
    }
    for q in g.opps(o).collect::<Vec<_>>() {
        lose_life(g, q, 3, Some(o), "burn", Some(true))?;
    }
    check_state(g)
}

// ======================================================== Archfiend of Sorrows: unearth
/// unearth {3}{B}{B} when it would wipe small creatures (hasty, exiled at end of turn)
fn archfiend_unearth(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || !can_pay(g, p, 3, "BB", false) {
        return Ok(vec![]);
    }
    let small = opp_creatures(g, p).into_iter().filter(|&m| etgh(g, m) <= 2).map(|m| pval(g, m)).psum();
    if small < 5.0 {
        return Ok(vec![]);
    }
    Ok(vec![opt_plan(small / 3.0, "unearth Archfiend of Sorrows".into(), archfiend_go, c.0 as i64)])
}

fn archfiend_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).gy.contains(&c) || !can_pay(g, p, 3, "BB", false) {
        return Ok(false);
    }
    pay(g, p, 3, "BB", false)?;
    if !ability_window_card(g, p, c, "unearth")? || !g.player(p).gy.contains(&c) {
        return Ok(true);
    }
    remove_card(&mut g.player_mut(p).gy, c);
    let m = enter(g, p, c, Enter::default())?;
    let x = g.perm_mut(m);
    x.sick = false;
    x.data.set(DataKey::Unearth, Val::Bool(true));
    crate::glog!(g, "  {} unearths Archfiend of Sorrows", pname(g, p));
    Ok(true)
}

fn archfiend_exile(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    let o = g.perm(src).owner;
    if g.perm(src).data.truthy(DataKey::Unearth) && on(g, o, src) {
        if !trigger_window(g, o, Some(src), "exile it (unearth)", Some(2.0))? || !on(g, o, src) {
            return Ok(());
        }
        leave(g, src)?;
        let cd = g.perm(src).cd.unwrap();
        let pl = g.player_mut(o);
        remove_card(&mut pl.gy, cd);
        pl.exile.push(cd);
    }
    Ok(())
}

// ======================================================== Aven Mindcensor: searches see the top four
/// Full: opponents searching their library search only the top four cards
fn mindcensor(g: &Game, src: Src, searcher: PlayerId) -> Option<u32> {
    if searcher != g.perm(src).owner { Some(4) } else { None }
}

// ======================================================== Bloodline Keeper // Lord of Lineage
const KEEPER: &str = "Bloodline Keeper // Lord of Lineage";

/// Full: {T}: 2/2 flying Vampire; {B}: transform with five Vampires (Lord of Lineage: other Vampires +2/+2, still
/// makes Vampires)
fn keeper(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || post.is_none() {
        return Ok(vec![]);
    }
    let mut o = vec![];
    if !g.perm(src).tapped && !g.perm(src).sick {
        o.push(opt_ability(2.0, "Bloodline Keeper token".into(), src, keeper_tok, 0));
    }
    if !g.perm(src).data.truthy(DataKey::Lord) && count_type(g, p, "vampire") >= 5 && can_pay(g, p, 0, "B", false) {
        o.push(opt_ability(4.0, "transform Bloodline Keeper".into(), src, keeper_flip, 0));
    }
    Ok(o)
}

fn keeper_tok(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if g.perm(src).tapped {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if ability_window(g, p, Some(src), "a 2/2 flying Vampire", None, None)? {
        let spec =
            Tokens { fly: true, color: Some(Colors::from_letters("B")), types: vec!["vampire"], ..Tokens::new(1, 2) };
        make_tokens(g, p, spec)?;
    }
    Ok(true)
}

fn keeper_flip(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !can_pay(g, p, 0, "B", false) {
        return Ok(false);
    }
    pay(g, p, 0, "B", false)?;
    if !ability_window(g, p, Some(src), "transform", None, None)? || !on(g, p, src) {
        return Ok(true);
    }
    let x = g.perm_mut(src);
    x.data.set(DataKey::Lord, Val::Bool(true));
    x.pow = 5;
    x.tgh = 5;
    crate::glog!(g, "  {} transforms Bloodline Keeper into Lord of Lineage", pname(g, p));
    Ok(true)
}

/// common.SELF_PT['Bloodline Keeper // Lord of Lineage']: (0, 0)
fn keeper_pt(_g: &Game, _p: PlayerId, _m: PermId) -> (i32, i32) {
    (0, 0)
}

fn keeper_on(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.selfpt = true;
        g.lineage = true;
    }
    Ok(())
}

/// partials.lineage_bonus: Lord of Lineage: other Vampires its controller controls get +2/+2
pub fn lineage_bonus(g: &Game, m: PermId) -> i32 {
    if !g.lineage || !g.is_creature(m) {
        return 0;
    }
    let o = g.perm(m).owner;
    let n = g
        .player(o)
        .perms
        .iter()
        .filter(|&&x| x != m && name_of(g, x) == KEEPER && g.perm(x).data.truthy(DataKey::Lord) && !g.perm(x).phased)
        .count() as i32;
    if n != 0 && has_type(g, m, "vampire") { 2 * n } else { 0 }
}

// ======================================================== Chain of Vapor
fn prio_0(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

/// Full: bounces a nonland permanent; its controller may sacrifice a land to copy it back at you
fn chain(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) else { return Ok("gy") };
    let q = g.perm(t).owner;
    apply_removal(g, Some(p), t, "bounce", None)?;
    if q == p || g.player(q).lands.len() < 5 {
        return Ok("gy");
    }
    if let Some(back) = best_opp_nonland(g, q, |_, _| true).filter(|&b| pval(g, b) >= 5.0) {
        let lands = g.player(q).lands.clone();
        let l = first_min(&lands, |l| {
            let x = g.land(l);
            (g.db.get(x.cd).tags.str(Tag::C).map_or(0, |s| s.chars().count()), !x.tapped)
        })
        .unwrap();
        crate::engine::turn::remove_land(g, q, l);
        let cd = g.land(l).cd;
        g.player_mut(q).gy.push(cd);
        crate::glog!(g, "    {} sacrifices {} to copy Chain of Vapor", pname(g, q), g.db.get(cd).name);
        apply_removal(g, Some(q), back, "bounce", None)?;
    }
    Ok("gy")
}

// ======================================================== Coat of Arms
fn coat_on(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.coat = true;
        g.selfpt = true;
    }
    Ok(())
}

/// partials._ctypes: a creature's creature types
fn ctypes(g: &Game, m: PermId) -> Vec<Sym> {
    let x = g.perm(m);
    match x.cd {
        Some(c) => {
            g.db.get(c).subtypes.iter().filter(|t| !matches!(&***t, "aura" | "equipment")).map(|t| intern(t)).collect()
        }
        None => x.ttypes.iter().copied().filter(|t| !matches!(*t, "land" | "artifact")).collect(),
    }
}

/// partials.coat_bonus: Coat of Arms: +1/+1 per other creature sharing a creature type (cached per battlefield
/// version, as Python's `g.coat_cache`)
pub fn coat_bonus(g: &Game, m: PermId) -> i32 {
    if !g.coat || !g.is_creature(m) {
        return 0;
    }
    let ver = g.bf_ver;
    let fresh = matches!(&*g.coat_cache.borrow(), Some((v, _)) if *v == ver);
    if !fresh {
        let coats = g
            .players
            .iter()
            .flat_map(|q| q.perms.iter())
            .filter(|&&x| name_of(g, x) == "Coat of Arms" && !g.perm(x).phased)
            .count() as i32;
        let cs: Vec<PermId> = g
            .players
            .iter()
            .filter(|q| q.alive)
            .flat_map(|q| q.perms.iter().copied())
            .filter(|&x| g.is_creature(x) && !g.perm(x).phased)
            .collect();
        let types: Vec<Vec<Sym>> = cs.iter().map(|&x| ctypes(g, x)).collect();
        let mut bonus = vec![];
        if coats != 0 {
            for (i, &x) in cs.iter().enumerate() {
                let tx = &types[i];
                let n = if tx.is_empty() {
                    0
                } else {
                    coats * (0..cs.len()).filter(|&j| j != i && types[j].iter().any(|t| tx.contains(t))).count() as i32
                };
                bonus.push((x, n));
            }
        }
        *g.coat_cache.borrow_mut() = Some((ver, bonus));
    }
    let cache = g.coat_cache.borrow();
    cache.as_ref().unwrap().1.iter().find(|x| x.0 == m).map_or(0, |x| x.1)
}

// ======================================================== Conduit of Worlds
/// {T}: cast a nonland permanent card from your graveyard if you haven't cast a spell this turn (sorcery)
fn conduit(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || post != Some(false) || g.perm(src).tapped || casts_this_turn(g, p) != 0 {
        return Ok(vec![]);
    }
    let cs: Vec<CardId> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&c| g.db.get(c).perm && !g.db.get(c).land && can_pay_cost(g, p, c))
        .collect();
    let Some(c) = first_max(&cs, |c| (card_worth(g, p, c, false), g.db.get(c).cmc)) else { return Ok(vec![]) };
    let label = format!("Conduit of Worlds ({})", g.db.get(c).name);
    Ok(vec![opt_ability(1.0 + card_worth(g, p, c, false) / 20.0, label, src, conduit_go, c.0 as i64)])
}

fn conduit_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if g.perm(src).tapped || !g.player(p).gy.contains(&c) || !can_pay_cost(g, p, c) {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    let label = format!("cast {} from the graveyard", g.db.get(c).name);
    if !ability_window(g, p, Some(src), &label, None, None)? || !g.player(p).gy.contains(&c) {
        return Ok(true);
    }
    pay_cost(g, p, c)?;
    crate::glog!(g, "  {} casts {} from the graveyard with Conduit of Worlds", pname(g, p), g.db.get(c).name);
    cast_card(g, p, c, "gy", Ctx::default())?;
    g.player_mut(p).conduit_lock = Some(g.turn_stamp());
    Ok(true)
}

fn conduit_lock(g: &Game, src: Src, caster: PlayerId, _c: CardId, _zone: Sym) -> bool {
    !(caster == g.perm(src).owner && g.player(caster).conduit_lock == Some(g.turn_stamp()))
}

// ======================================================== Conspicuous Snoop
/// Full: casts Goblins from the top of the library; gains the activated abilities of a Goblin on top
fn snoop(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || g.player(p).library.is_empty() {
        return Ok(vec![]);
    }
    let top = *g.player(p).library.last().unwrap();
    let d = g.db.get(top);
    let goblin = d.has_subtype("goblin");
    let mut o = vec![];
    if post == Some(false) && goblin && d.creature && can_pay_cost(g, p, top) && castable(g, p, top, "hand") {
        let label = format!("Snoop: cast {}", d.name);
        o.push(opt_ability(2.0 + card_worth(g, p, top, false) / 20.0, label, src, snoop_go, top.0 as i64));
    }
    if goblin && let Some(f) = g.registry.get(top).and_then(|i| i.options) {
        o.extend(f(g, src, p, post)?);
    }
    Ok(o)
}

fn snoop_go(g: &mut Game, _src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let top = CardId(arg as u16);
    if g.player(p).library.last() != Some(&top) || !can_pay_cost(g, p, top) {
        return Ok(false);
    }
    g.player_mut(p).library.pop();
    pay_cost(g, p, top)?;
    crate::glog!(g, "  {} casts {} from the top of the library (Snoop)", pname(g, p), g.db.get(top).name);
    cast_card(g, p, top, "lib", Ctx::default())?;
    Ok(true)
}

// ======================================================== Crypt Ghast: extort
/// partials.extort: whenever you cast a spell, pay {B} (with spare mana): each opponent loses 1, you gain that much
fn extort(g: &mut Game, src: Src, caster: PlayerId, _c: CardId) -> Res {
    if caster != g.perm(src).owner || g.perm(src).phased {
        return Ok(());
    }
    let ok = |g: &Game| total_mana(g, caster, false) >= 3 && can_pay(g, caster, 0, "B", false);
    if ok(g) {
        if !trigger_window(g, caster, Some(src), "extort", None)? {
            return Ok(());
        }
        if !ok(g) {
            return Ok(());
        }
        pay(g, caster, 0, "B", false)?;
        let mut n = 0;
        for q in g.opps(caster).collect::<Vec<_>>() {
            lose_life(g, q, 1, Some(caster), "drain", None)?;
            n += 1;
        }
        gain(g, caster, n)?;
    }
    Ok(())
}

// ======================================================== delve: Dig Through Time, Treasure Cruise; Draco
/// partials.delve_count: the generic mana delve pays (the graveyard cards not worth keeping)
pub fn delve_count(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let fodder = g.player(p).gy.iter().filter(|&&x| card_worth(g, p, x, true) < 20.0 && x != c).count();
    (g.db.get(c).generic as usize).min(fodder) as i32
}

fn delve_cost(g: &Game, p: PlayerId, c: CardId) -> i32 {
    -delve_count(g, p, c)
}

/// Full: delve (exiles the least useful graveyard cards); draw three
fn cruise(g: &mut Game, p: PlayerId, c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let n = delve_count(g, p, c) as usize;
    exile_fodder(g, p, c, n);
    draw(g, p, 3, false)?;
    Ok("gy")
}

fn prio_48(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    48
}

/// Full: delve; look at the top seven, keep the best two, the rest on the bottom
fn dig(g: &mut Game, p: PlayerId, c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let n = delve_count(g, p, c) as usize;
    exile_fodder(g, p, c, n);
    let top = crate::ai::topdeck::look(g, p, 7);
    let w: Vec<f64> = top.iter().map(|&x| -crate::ai::topdeck::desire(g, p, x)).collect();
    let mut idx: Vec<usize> = (0..top.len()).collect();
    idx.sort_by(|&a, &b| w[a].partial_cmp(&w[b]).unwrap_or(std::cmp::Ordering::Equal)); // stable
    let top: Vec<CardId> = idx.into_iter().map(|i| top[i]).collect();
    let pl = g.player_mut(p);
    for &x in top.iter().take(2) {
        pl.hand.push(x);
        pl.seen.insert(x);
    }
    pl.library.splice(0..0, top.iter().skip(2).copied());
    Ok("gy")
}

fn prio_50(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    50
}

/// partials.domain: the basic land types among p's lands
pub fn domain(g: &Game, p: PlayerId) -> i32 {
    const BASIC: [(&str, &str); 5] = [
        ("plains", "Plains"),
        ("island", "Island"),
        ("swamp", "Swamp"),
        ("mountain", "Mountain"),
        ("forest", "Forest"),
    ];
    BASIC
        .iter()
        .filter(|(t, n)| {
            g.player(p).lands.iter().any(|&l| {
                let d = g.db.get(g.land(l).cd);
                &*d.name == *n || d.has_subtype(t)
            })
        })
        .count() as i32
}

fn draco_cost(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    -2 * domain(g, p)
}

/// Full: upkeep: pay {10} minus {2} per basic land type or sacrifice it
fn draco_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != g.perm(src).owner {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "sacrifice Draco unless you pay", None)? || !on(g, p, src) {
        return Ok(());
    }
    let n = (10 - 2 * domain(g, p)).max(0) as u32;
    if can_pay(g, p, n, "", false) {
        pay(g, p, n, "", false)?;
    } else {
        crate::glog!(g, "    {} sacrifices Draco", pname(g, p));
        die(g, src, "sac")?;
    }
    Ok(())
}

/// a Yuriko deck keeps Draco for the reveal
fn draco_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if crate::ai::topdeck::mv_mode(g, p) { 0 } else { 50 }
}

// ======================================================== Enter the Infinite
/// Full: draws the library, puts the best card for next turn on top, no maximum hand size until your next turn
fn eti(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let n = g.player(p).library.len() as u32;
    draw(g, p, n, false)?;
    let hand = g.player(p).hand.clone();
    if let Some(x) = max_by(&hand, |x| card_worth(g, p, x, false)) {
        let pl = g.player_mut(p);
        remove_card(&mut pl.hand, x);
        pl.library.push(x);
    }
    let pl = g.player_mut(p);
    pl.nomax_turn = Some(pl.turns);
    Ok("gy")
}

fn prio_70(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    70
}

// ======================================================== Faerie Mastermind
/// Full: draws when an opponent draws their second card each turn
fn mastermind(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = g.perm(src).owner;
    if p == o || !g.player(p).alive || g.player(p).draw_n != 2 {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "draw a card", None)? {
        return Ok(());
    }
    let key = intern(&format!("mastermind{}{}", src.0, g.player(p).key));
    if once_per_turn(g, o, key) {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

// ======================================================== Gitaxian Probe: Phyrexian mana
/// Full: {U} or 2 life; draw a card (looking at a hand has no effect in the sim)
fn probe(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    if can_pay(g, p, 0, "U", false) && total_mana(g, p, false) >= 3 {
        pay(g, p, 0, "U", false)?;
    } else {
        lose_life(g, p, 2, Some(p), "other", None)?;
    }
    draw(g, p, 1, false)?;
    Ok("gy")
}

fn prio_40(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    40
}

// ======================================================== Gix, Yawgmoth Praetor
fn gix_junk(g: &Game, p: PlayerId) -> Vec<CardId> {
    g.player(p).hand.iter().copied().filter(|&c| card_worth(g, p, c, false) < 35.0).collect()
}

fn gix_target(g: &Game, p: PlayerId) -> Option<PlayerId> {
    first_max(&g.opps(p).collect::<Vec<_>>(), |q| g.player(q).library.len())
}

/// Full: {4}{B}{B}{B}, discard X junk cards: plays the top X of an opponent's library free
fn gix(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || post != Some(false) || !can_pay(g, p, 4, "BBB", false) {
        return Ok(vec![]);
    }
    let junk = gix_junk(g, p);
    if junk.len() < 2 {
        return Ok(vec![]);
    }
    let Some(q) = gix_target(g, p).filter(|&q| g.player(q).library.len() >= junk.len()) else { return Ok(vec![]) };
    let _ = q;
    Ok(vec![opt_ability(1.0 + 0.8 * junk.len() as f64, "Gix (discard X)".into(), src, gix_go, 0)])
}

fn gix_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let junk = gix_junk(g, p);
    let Some(q) = gix_target(g, p) else { return Ok(false) };
    if !can_pay(g, p, 4, "BBB", false) {
        return Ok(false);
    }
    pay(g, p, 4, "BBB", false)?;
    for &c in &junk {
        let pl = g.player_mut(p);
        if remove_card(&mut pl.hand, c) {
            pl.gy.push(c);
        }
    }
    let label = format!("exile the top {} of a library", junk.len());
    if !ability_window(g, p, Some(src), &label, Some(6.0), None)? {
        return Ok(true);
    }
    let mut top = vec![];
    for _ in 0..junk.len().min(g.player(q).library.len()) {
        top.push(g.player_mut(q).library.pop().unwrap());
    }
    crate::glog!(g, "  {} activates Gix: exiles {} cards of {}", pname(g, p), top.len(), pname(g, q));
    for c in top {
        let d = g.db.get(c);
        if d.land && g.player(p).lands.len() < 20 {
            g.add_land(p, c, false);
        } else if !d.land && castable(g, p, c, "hand") {
            cast_card(g, p, c, "lib", Ctx::default())?;
        } else {
            g.player_mut(q).exile.push(c);
        }
    }
    Ok(true)
}

// ======================================================== Goblin Cratermaker
/// Full: {1}, sacrifice: 2 damage to a creature or destroy a colorless nonland permanent
fn cratermaker(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || !can_pay(g, p, 1, "", false) {
        return Ok(vec![]);
    }
    let t1 = best_opp_creature(g, p, |g, m| etgh(g, m) <= 2);
    let t2 = best_opp_nonland(g, p, |g, m| g.perm(m).cd.is_some_and(|c| g.db.get(c).pips.is_empty()));
    let ts: Vec<PermId> = [t1, t2].into_iter().flatten().collect();
    let Some(t) = max_by(&ts, |m| pval(g, m)).filter(|&t| pval(g, t) >= 3.0) else { return Ok(vec![]) };
    let dmg = Some(t) == t1;
    let label = format!("Goblin Cratermaker -> {}", g.perm(t).name);
    Ok(vec![opt_ability(pval(g, t) - 2.0, label, src, cratermaker_go, pack(t.0, dmg as u32))])
}

fn cratermaker_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (t, dmg) = unpack(arg);
    let t = PermId(t);
    if !on(g, p, src) || !g.perm(t).on_bf || !can_pay(g, p, 1, "", false) {
        return Ok(false);
    }
    pay(g, p, 1, "", false)?;
    die(g, src, "sac")?;
    let cd = g.perm(src).cd.unwrap();
    let label = g.perm(t).name.to_string();
    if crate::engine::stack::ability_window_card(g, p, cd, &label, None, Some(t))? && g.perm(t).on_bf {
        apply_removal(g, Some(p), t, if dmg != 0 { "dmg2" } else { "destroy" }, None)?;
    }
    Ok(true)
}

// ======================================================== Golgari Charm
/// Full: modes: -1/-1 to all creatures (against tokens), destroy an enchantment, or regenerate your creatures in
/// response to a destroy wipe
fn golgari_charm(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    match ctx.mode {
        Some("regen") => {
            g.player_mut(p).regen_turn = Some(g.turn_stamp());
            return Ok("gy");
        }
        Some("shrink") => {
            for q in 0..g.players.len() {
                for m in g.players[q].perms.clone() {
                    if g.is_creature(m) {
                        super::common::eot(g, m, -1, -1);
                        if etgh(g, m) <= 0 {
                            die(g, m, "sba")?;
                        }
                    }
                }
            }
            return Ok("gy");
        }
        _ => {}
    }
    if let Some(t) = best_opp_nonland(g, p, |g, m| of_type(g, m, Types::ENCHANTMENT)) {
        apply_removal(g, Some(p), t, "destroy", Some(c))?;
    }
    Ok("gy")
}

fn golgari_opts(g: &mut Game, c: CardId, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if !can_pay(g, p, 0, "BG", false) {
        return Ok(vec![]);
    }
    let mut o = vec![];
    let toks = opp_creatures(g, p).into_iter().filter(|&m| etgh(g, m) <= 1).map(|m| pval(g, m)).psum();
    let mine = creatures_of(g, p).into_iter().filter(|&m| etgh(g, m) <= 1).map(|m| pval(g, m)).psum();
    if toks - mine >= 5.0 {
        o.push(opt_plan(toks - mine - 2.0, "Golgari Charm (-1/-1)".into(), golgari_shrink, c.0 as i64));
    }
    if let Some(t) = best_opp_nonland(g, p, |g, m| of_type(g, m, Types::ENCHANTMENT)).filter(|&t| pval(g, t) >= 4.0) {
        o.push(opt_plan(pval(g, t) - 2.5, "Golgari Charm (enchantment)".into(), golgari_ench, c.0 as i64));
    }
    Ok(o)
}

fn golgari_shrink(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    cast_mode(g, p, CardId(arg as u16), "shrink")
}

fn golgari_ench(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    cast_mode(g, p, CardId(arg as u16), "ench")
}

/// partials._cast_mode: Golgari Charm cast for this mode
fn cast_mode(g: &mut Game, p: PlayerId, c: CardId, mode: Sym) -> Res<bool> {
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 0, "BG", false) {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    pay(g, p, 0, "BG", false)?;
    cast_card(g, p, c, "lib", Ctx { mode: Some(mode), ..Ctx::default() })?;
    if !g.player(p).gy.contains(&c) {
        g.player_mut(p).gy.push(c);
    }
    Ok(true)
}

/// partials.regen_wipe: a destroy wipe is coming: Golgari Charm regenerates p's creatures
pub fn regen_wipe(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(c) = g.player(p).hand.iter().copied().find(|&x| &*g.db.get(x).name == "Golgari Charm") else {
        return Ok(false);
    };
    if !can_pay(g, p, 0, "BG", false) || !castable(g, p, c, "hand") {
        return Ok(false);
    }
    if creatures_of(g, p).into_iter().map(|m| pval(g, m)).psum() < 8.0 {
        return Ok(false);
    }
    cast_mode(g, p, c, "regen")
}

// ======================================================== Grim Hireling: -X/-X for Treasures
/// Full: {B}, sacrifice X Treasures: -X/-X to a creature
fn hireling(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let tr = g.player(p).treasures as i32;
    if g.perm(src).owner != p || post != Some(false) || !can_pay(g, p, 0, "B", false) || tr < 1 {
        return Ok(vec![]);
    }
    let Some(t) = best_opp_creature(g, p, |g, m| etgh(g, m) <= tr).filter(|&t| pval(g, t) >= 4.0) else {
        return Ok(vec![]);
    };
    let x = etgh(g, t);
    let label = format!("Grim Hireling -> {}", g.perm(t).name);
    Ok(vec![opt_ability(pval(g, t) - 1.0 - 0.6 * x as f64, label, src, hireling_go, pack(t.0, x as u32))])
}

fn hireling_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (t, x) = unpack(arg);
    let (t, x) = (PermId(t), x as i32);
    if !g.perm(t).on_bf || (g.player(p).treasures as i32) < x || !can_pay(g, p, 0, "B", false) {
        return Ok(false);
    }
    pay(g, p, 0, "B", false)?;
    let pl = g.player_mut(p);
    pl.treasures = (pl.treasures as i32 - x).max(0) as u32;
    for _ in 0..x {
        if !g.hooks.is_empty() {
            fire_trigger(g, Event::Sacrifice, Call::Sacrifice { p, what: Sacrificed::Token("Treasure") })?;
        }
    }
    crate::glog!(g, "  {} sacrifices {} Treasures: {} gets -{}/-{}", pname(g, p), x, g.perm(t).name, x, x);
    let label = format!("-{x}/-{x} to {}", g.perm(t).name);
    if !ability_window(g, p, Some(src), &label, None, Some(t))? || !g.perm(t).on_bf {
        return Ok(true);
    }
    if !untargetable(g, t) {
        super::common::eot(g, t, -x, -x);
        if etgh(g, t) <= 0 {
            die(g, t, "sba")?;
        }
    }
    Ok(true)
}

// ======================================================== Hope of Ghirapur
fn hope_hit(g: &mut Game, src: Src, _p: PlayerId, a: PermId, d: PlayerId, _dmg: i32) -> Res {
    if a == src {
        let now = turn_now(g);
        g.perm_mut(src).data.set(DataKey::Hit, Val::List(vec![now, Val::Player(d)]));
    }
    Ok(())
}

/// Full: after it connects, sacrificed to stop that player casting noncreature spells until your next turn
fn hope_sac(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || post != Some(true) {
        return Ok(vec![]);
    }
    let Some(Val::List(hit)) = data_val(g, src, DataKey::Hit) else { return Ok(vec![]) };
    let (Val::Stamp(st), Val::Player(d)) = (&hit[0], &hit[1]) else { return Ok(vec![]) };
    let (st, d) = (*st, *d);
    if st != g.turn_stamp() || !g.player(d).alive || g.player(d).hand.len() < 3 {
        return Ok(vec![]);
    }
    let u = 1.0 + 0.2 * g.player(d).hand.len() as f64;
    Ok(vec![opt_ability(u, "Hope of Ghirapur".into(), src, hope_go, d.0 as i64)])
}

fn hope_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let d = PlayerId(arg as u8);
    if !on(g, p, src) {
        return Ok(false);
    }
    die(g, src, "sac")?;
    crate::glog!(g, "  {} sacrifices Hope of Ghirapur: {} can't cast noncreature spells", pname(g, p), pname(g, d));
    let cd = g.perm(src).cd.unwrap();
    let label = format!("{} can't cast noncreature spells", pname(g, d));
    if crate::engine::stack::ability_window_card(g, p, cd, &label, None, None)? {
        let t = g.player(p).turns;
        g.player_mut(d).hope_lock = Some((p, t));
    }
    Ok(true)
}

// ======================================================== Hunter's Insight
/// Full: cast before combat on the creature most likely to connect: its combat damage to a player draws that many
fn insight(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = ctx.host {
        g.player_mut(p).insight = Some((g.turn_stamp(), t));
    }
    Ok("gy")
}

fn insight_opt(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || !can_pay(g, p, 2, "G", false) {
        return Ok(vec![]);
    }
    let good: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            g.is_creature(m) && !x.sick && !x.tapped && !x.noatk
        })
        .filter(|&m| super::lands::open_attack(g, p, epow(g, m), g.perm(m).fly))
        .collect();
    let Some(t) = max_by(&good, |m| epow(g, m) as f64).filter(|&t| epow(g, t) >= 3) else { return Ok(vec![]) };
    let label = format!("Hunter's Insight on {}", g.perm(t).name);
    Ok(vec![opt_plan(1.0 + 0.5 * epow(g, t) as f64, label, insight_go, pack(c.0 as u32, t.0))])
}

fn insight_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, t) = unpack(arg);
    let (c, t) = (CardId(c as u16), PermId(t));
    if !g.player(p).hand.contains(&c) || !on(g, p, t) || !can_pay(g, p, 2, "G", false) {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    pay(g, p, 2, "G", false)?;
    cast_card(g, p, c, "lib", Ctx { host: Some(t), ..Ctx::default() })?;
    if !g.player(p).gy.contains(&c) {
        g.player_mut(p).gy.push(c);
    }
    Ok(true)
}

/// partials.insight_draw: the creature Hunter's Insight was cast on deals combat damage to a player: draw that many
pub fn insight_draw(g: &mut Game, p: PlayerId, a: PermId, dmg: i32) -> Res {
    if let Some((st, m)) = g.player(p).insight
        && st == g.turn_stamp()
        && m == a
        && dmg > 0
    {
        draw(g, p, dmg as u32, false)?;
        crate::glog!(g, "    Hunter's Insight: {} draws {}", pname(g, p), dmg);
    }
    Ok(())
}

// ======================================================== Lim-Dûl's Vault
fn vault_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if crate::ai::plans::wish_list(g, p).is_empty() { 0 } else { 45 }
}

/// Full: digs five at a time for 1 life each until a wanted card turns up, then stacks it on top
fn vault(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let wish = crate::ai::plans::wish_list(g, p);
    for i in 0..6 {
        let top = crate::ai::topdeck::look(g, p, 5);
        if top.is_empty() {
            return Ok("gy");
        }
        if top.iter().any(|x| wish.contains(x)) || i == 5 || g.player(p).life <= 15 || g.player(p).library.len() < 10 {
            shuffle_library(g, p);
            crate::ai::topdeck::arrange(g, p, &top);
            return Ok("gy");
        }
        g.player_mut(p).library.splice(0..0, top);
        lose_life(g, p, 1, Some(p), "other", None)?;
    }
    Ok("gy")
}

// ======================================================== Loran of the Third Path
/// Full: {T}: you and the least threatening opponent each draw (at end of turn)
fn loran(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || post.is_some() || g.perm(src).tapped || g.perm(src).sick {
        return Ok(vec![]);
    }
    let os: Vec<PlayerId> = g.opps(p).collect();
    let Some(q) = min_by(&os, |q| threat(g, p, q)) else { return Ok(vec![]) };
    Ok(vec![opt_ability(0.6, "Loran draw".into(), src, loran_go, q.0 as i64)])
}

fn loran_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let q = PlayerId(arg as u8);
    if g.perm(src).tapped {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if ability_window(g, p, Some(src), "you and an opponent draw", None, None)? {
        draw(g, p, 1, false)?;
        draw(g, q, 1, false)?;
    }
    Ok(true)
}

// ======================================================== Massacre Girl
/// Full: menace; -1/-1 to every other creature on entry, repeated for each creature that dies
fn massacre(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if !trigger_window(g, o, Some(src), "each other creature gets -1/-1, repeated", Some(6.0))? {
        return Ok(());
    }
    crate::glog!(g, "    Massacre Girl: each other creature gets -1/-1, again for every creature that dies");
    let count = |g: &Game| g.players.iter().flat_map(|q| q.perms.iter()).filter(|&&x| g.is_creature(x)).count() as i32;
    let mut shrink = 1;
    while shrink != 0 {
        let before = count(g);
        for q in 0..g.players.len() {
            for x in g.players[q].perms.clone() {
                if x == src || !g.is_creature(x) {
                    continue;
                }
                super::common::eot(g, x, -shrink, -shrink);
            }
        }
        for q in 0..g.players.len() {
            for x in g.players[q].perms.clone() {
                if x != src && g.is_creature(x) && etgh(g, x) <= 0 {
                    die(g, x, "sba")?;
                }
            }
        }
        let after = count(g);
        shrink = before - after;
        if shrink <= 0 || after <= 1 {
            break;
        }
    }
    Ok(())
}

fn massacre_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let theirs = opp_creatures(g, p).into_iter().filter(|&m| etgh(g, m) <= 2).map(|m| pval(g, m)).psum();
    let mine = creatures_of(g, p).into_iter().map(|m| pval(g, m)).psum();
    if theirs >= 8.0 + mine { 60 } else { 0 }
}

// ======================================================== Mishra's Bauble / Urza's Bauble
/// Full: {T}, sacrifice: draw at the beginning of the next upkeep (kept as an artifact for Urza)
fn bauble(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || g.perm(src).tapped || post.is_some() {
        return Ok(vec![]);
    }
    if g.player(p).perms.iter().any(|&m| name_of(g, m) == "Urza, Lord High Artificer") {
        return Ok(vec![]);
    }
    let name = name_of(g, src).to_string();
    Ok(vec![opt_ability(0.8, name, src, bauble_go, 0)])
}

fn bauble_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !on(g, p, src) {
        return Ok(false);
    }
    die(g, src, "sac")?;
    let cd = g.perm(src).cd.unwrap();
    crate::glog!(g, "  {} sacrifices {} (draws at the next upkeep)", pname(g, p), g.db.get(cd).name);
    if crate::engine::stack::ability_window_card(g, p, cd, "draw at the next upkeep", None, None)? {
        g.player_mut(p).delayed_draws += 1;
    }
    Ok(true)
}

// ======================================================== Mogg Fanatic
/// Full: sacrifice: 1 damage to a valuable X/1 or a player at 1 life
fn fanatic(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p {
        return Ok(vec![]);
    }
    let t = best_opp_creature(g, p, |g, m| etgh(g, m) <= 1);
    if let Some(q) = g.opps(p).find(|&q| g.player(q).life <= 1) {
        return Ok(vec![opt_ability(9.0, "Mogg Fanatic (lethal)".into(), src, fanatic_kill, q.0 as i64)]);
    }
    let Some(t) = t.filter(|&t| pval(g, t) >= 3.0) else { return Ok(vec![]) };
    let label = format!("Mogg Fanatic -> {}", g.perm(t).name);
    Ok(vec![opt_ability(pval(g, t) - 1.5, label, src, fanatic_ping, t.0 as i64)])
}

fn fanatic_kill(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let q = PlayerId(arg as u8);
    die(g, src, "sac")?;
    let cd = g.perm(src).cd.unwrap();
    if crate::engine::stack::ability_window_card(g, p, cd, "1 damage", Some(9.0), None)? {
        lose_life(g, q, 1, Some(p), "burn", None)?;
    }
    Ok(true)
}

fn fanatic_ping(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    die(g, src, "sac")?;
    let cd = g.perm(src).cd.unwrap();
    let label = format!("1 damage to {}", g.perm(t).name);
    if crate::engine::stack::ability_window_card(g, p, cd, &label, None, Some(t))? && g.perm(t).on_bf {
        apply_removal(g, Some(p), t, "dmg1", None)?;
    }
    Ok(true)
}

// ======================================================== Multani: return from the graveyard
/// Full: {1}{G}, return two lands: back to hand from the graveyard
fn multani(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || !can_pay(g, p, 1, "G", false) || g.player(p).lands.len() < 7 {
        return Ok(vec![]);
    }
    let pl = g.player(p);
    if pl.land_turn == pl.turns as i32 && !pl.hand.iter().any(|&x| g.db.get(x).land) {
        return Ok(vec![]);
    }
    Ok(vec![opt_plan(1.5, "Multani (graveyard)".into(), multani_go, c.0 as i64)])
}

fn multani_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).gy.contains(&c) || !can_pay(g, p, 1, "G", false) || g.player(p).lands.len() < 2 {
        return Ok(false);
    }
    pay(g, p, 1, "G", false)?;
    let mut lands = g.player(p).lands.clone();
    let key = |g: &Game, l: LandId| {
        let x = g.land(l);
        (!x.tapped, g.db.get(x.cd).tags.str(Tag::C).map_or(0, |s| s.chars().count()))
    };
    lands.sort_by_key(|&l| key(g, l)); // stable, as Python's sorted
    for l in lands.into_iter().take(2) {
        crate::engine::turn::remove_land(g, p, l);
        let cd = g.land(l).cd;
        g.player_mut(p).hand.push(cd);
    }
    if !ability_window_card(g, p, c, "return to hand")? || !g.player(p).gy.contains(&c) {
        return Ok(true);
    }
    let pl = g.player_mut(p);
    remove_card(&mut pl.gy, c);
    pl.hand.push(c);
    crate::glog!(g, "  {} returns Multani to hand", pname(g, p));
    Ok(true)
}

// ======================================================== Muxus: attack pump
/// Full: attacks with +1/+1 per other Goblin
fn muxus_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if p == g.perm(src).owner && atk.contains(&src) {
        if !trigger_window(g, p, Some(src), "+1/+1 for each other Goblin", None)? || !on(g, p, src) {
            return Ok(vec![]);
        }
        let n = g.player(p).perms.iter().filter(|&&m| m != src && g.is_creature(m) && has_type(g, m, "goblin")).count()
            as i32;
        super::common::eot(g, src, n, n);
    }
    Ok(vec![])
}

// ======================================================== Oath of Teferi
fn oath_cands(g: &Game, o: PlayerId, src: PermId) -> Vec<PermId> {
    g.player(o)
        .perms
        .iter()
        .copied()
        .filter(|&x| {
            let y = g.perm(x);
            x != src
                && !y.token
                && y.cd.is_some_and(|c| {
                    let d = g.db.get(c);
                    etb_val(g, x) > 0.0 || matches!((d.start_loyalty, y.loyalty), (Some(s), Some(l)) if l < s)
                })
        })
        .collect()
}

/// Full: blinks a permanent with an ETB or a used planeswalker until the end step (planeswalkers activate twice
/// each turn: common._allowed)
fn oath(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if oath_cands(g, o, src).is_empty()
        || !trigger_window(g, o, Some(src), "exile a permanent you control until the end step", None)?
    {
        return Ok(());
    }
    let cs = oath_cands(g, o, src);
    if let Some(t) = max_by(&cs, |x| {
        let walker = g.perm(x).cd.is_some_and(|c| g.db.get(c).start_loyalty.is_some());
        etb_val(g, x) + if walker { 3.0 } else { 0.0 }
    }) {
        leave(g, t)?;
        let cd = g.perm(t).cd.unwrap();
        g.player_mut(o).oath_return.push(cd);
        crate::glog!(g, "    Oath of Teferi exiles {} until the end step", g.perm(t).name);
    }
    Ok(())
}

fn oath_back(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    let o = g.perm(src).owner;
    if g.player(o).oath_return.is_empty() {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "return the exiled permanent", None)? {
        g.player_mut(o).oath_return.clear();
        return Ok(());
    }
    for cd in g.player(o).oath_return.clone() {
        enter(g, o, cd, Enter::default())?;
    }
    g.player_mut(o).oath_return.clear();
    Ok(())
}

/// partials.walker_uses: Oath of Teferi lets planeswalkers activate twice each turn
pub fn walker_uses(g: &Game, p: PlayerId) -> i64 {
    super::common::allowed(g, p)
}

// ======================================================== Paradise Druid
/// Full: hexproof while untapped; taps for any color
fn pdruid(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "hexproof" && m == src && !g.perm(src).tapped
}

// ======================================================== Pashalik Mons: Goblin tokens
fn mons_fodder(g: &Game, p: PlayerId, src: PermId) -> Option<PermId> {
    let ps = &g.player(p).perms;
    ps.iter().copied().find(|&m| g.is_creature(m) && g.perm(m).token && has_type(g, m, "goblin")).or_else(|| {
        ps.iter().copied().find(|&m| g.is_creature(m) && has_type(g, m, "goblin") && m != src && pval(g, m) < 2.0)
    })
}

/// Full: {3}{R}, sacrifice a Goblin: two Goblin tokens
fn mons(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || !can_pay(g, p, 3, "R", false) {
        return Ok(vec![]);
    }
    let Some(f) = mons_fodder(g, p, src) else { return Ok(vec![]) };
    let u = if post.is_none() { 1.2 } else { 0.8 };
    Ok(vec![opt_ability(u, "Pashalik Mons tokens".into(), src, mons_go, f.0 as i64)])
}

fn mons_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let f = PermId(arg as u32);
    if !can_pay(g, p, 3, "R", false) || !on(g, p, f) {
        return Ok(false);
    }
    pay(g, p, 3, "R", false)?;
    die(g, f, "sac")?;
    if ability_window(g, p, Some(src), "two Goblins", None, None)? {
        let spec = Tokens { color: Some(Colors::from_letters("R")), types: vec!["goblin"], ..Tokens::new(2, 1) };
        make_tokens(g, p, spec)?;
    }
    Ok(true)
}

// ======================================================== Quirion Ranger / Wirewood Symbiote
fn best_dork(g: &Game, p: PlayerId) -> Option<(PermId, u32)> {
    let dorks: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            g.is_creature(m) && x.tapped && x.cd.is_some_and(|c| g.db.get(c).tag(Tag::Dork)) && !x.sick
        })
        .collect();
    let amt = |m: PermId| cardcode::dyn_mana_perm(g, p, m).unwrap_or(1);
    let d = max_by(&dorks, |m| amt(m) as f64)?;
    Some((d, amt(d)))
}

/// Full: returns a Forest (replayed as the land drop) to untap the best mana creature, once a turn
fn quirion(g: &Game, src: Src, p: PlayerId, _u: &[Unit]) -> Vec<Unit> {
    if g.perm(src).owner != p || data_val(g, src, DataKey::Used) == Some(&turn_now(g)) {
        return vec![];
    }
    let pl = g.player(p);
    if !pl.lands.iter().any(|&l| &*g.db.get(g.land(l).cd).name == "Forest") || pl.land_turn == pl.turns as i32 {
        return vec![];
    }
    let Some((d, amt)) = best_dork(g, p) else { return vec![] };
    if amt < 2 {
        return vec![];
    }
    vec![Unit { src: Source::Quirion(src, d), cols: Colors::from_letters("G"), amt }]
}

/// Full: returns a cheap Elf to untap the best mana creature, once a turn
fn symbiote(g: &Game, src: Src, p: PlayerId, _u: &[Unit]) -> Vec<Unit> {
    if g.perm(src).owner != p || data_val(g, src, DataKey::Used) == Some(&turn_now(g)) {
        return vec![];
    }
    let cheap = g.player(p).perms.iter().copied().find(|&m| {
        g.is_creature(m)
            && m != src
            && has_type(g, m, "elf")
            && g.perm(m).cd.is_some_and(|c| g.db.get(c).cmc <= 2 && !g.db.get(c).tag(Tag::Dork))
    });
    let Some(elf) = cheap else { return vec![] };
    let Some((d, amt)) = best_dork(g, p) else { return vec![] };
    if amt < 3 {
        return vec![];
    }
    vec![Unit { src: Source::Symbiote(src, d, elf), cols: Colors::from_letters("G"), amt }]
}

// ======================================================== Ranger-Captain of Eos: silence for a combo turn
/// partials.ranger_silence: before a combo or a big spell: sacrifice Ranger-Captain so opponents can't cast
/// noncreature spells (no AI calls it: the Python's is unused)
pub fn ranger_silence(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(m) =
        g.player(p).perms.iter().copied().find(|&m| name_of(g, m) == "Ranger-Captain of Eos" && !g.perm(m).phased)
    else {
        return Ok(false);
    };
    die(g, m, "sac")?;
    g.silence = Some((g.turn_stamp(), p));
    crate::glog!(
        g,
        "  {} sacrifices Ranger-Captain of Eos: opponents can't cast noncreature spells this turn",
        pname(g, p)
    );
    Ok(true)
}

/// partials.silence_active (engine.castable reads `g.silence` itself)
pub fn silence_active(g: &Game, q: PlayerId) -> bool {
    g.silence.is_some_and(|(st, p)| st == g.turn_stamp() && p != q)
}

// ======================================================== wheels: Wheel of Fortune, Reforge the Soul
/// partials.wheel: each player discards their hand and draws seven
fn wheel(g: &mut Game) -> Res {
    let alive: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
    for q in alive {
        let pl = g.player_mut(q);
        let hand = std::mem::take(&mut pl.hand);
        pl.gy.extend(hand);
        draw(g, q, 7, false)?;
    }
    Ok(())
}

/// partials.wheel_prio: cast when your hand is nearly empty, never wheeling away a combo piece
fn wheel_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let hand = &g.player(p).hand;
    if hand.iter().any(|&x| x != c && cardcode::is_combo_piece(g, x)) {
        return 0; // don't wheel away a combo piece
    }
    let mine = hand.len() as i64 - 1;
    let theirs = g.opps(p).map(|q| g.player(q).hand.len()).max().unwrap_or(0);
    if mine <= 2 && theirs <= 5 { 55 } else { 0 }
}

/// Wheel of Fortune (Full): each player discards their hand and draws seven (cast when your hand is nearly empty);
/// Reforge the Soul (Full): wheel; miracle {1}{R} when it is the first card drawn in a turn
fn wheel_spell(g: &mut Game, _p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    wheel(g)?;
    Ok("gy")
}

// ======================================================== Sai, Master Thopterist
fn sai_thopters(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            x.token && (x.ttypes.contains(&"thopter") || x.ttypes.contains(&"artifact"))
        })
        .collect()
}

/// Full: {1}{U}, sacrifice two artifacts: draw (spare Thopters)
fn sai(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || post.is_some() || !can_pay(g, p, 1, "U", false) {
        return Ok(vec![]);
    }
    let urza = g.player(p).perms.iter().any(|&m| name_of(g, m) == "Urza, Lord High Artificer");
    if sai_thopters(g, p).len() < if urza { 4 } else { 3 } {
        return Ok(vec![]);
    }
    Ok(vec![opt_ability(1.0, "Sai: sacrifice two for a card".into(), src, sai_go, 0)])
}

fn sai_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let thop = sai_thopters(g, p);
    if !can_pay(g, p, 1, "U", false) || thop.len() < 2 {
        return Ok(false);
    }
    pay(g, p, 1, "U", false)?;
    for m in thop.into_iter().take(2) {
        die(g, m, "sac")?;
    }
    if ability_window(g, p, Some(src), "draw a card", None, None)? {
        draw(g, p, 1, false)?;
    }
    Ok(true)
}

// ======================================================== Sakashima's Protege
/// Full: flash; cascade; enters as a copy of the best permanent that entered this turn (the copy is made as a token
/// copy)
fn protege(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if g.last_cast_etb {
        protege_cascade(g, o, src)?; // the spell was cast (entering as a copy: a replacement)
    }
    let st = g.turn_stamp();
    let ent: Vec<PermId> = g
        .entered
        .iter()
        .filter(|x| x.0 == st && x.1 != src && g.perm(x.1).cd.is_some() && g.perm(x.1).on_bf)
        .map(|x| x.1)
        .collect();
    if let Some(t) = max_by(&ent, |x| pval(g, x))
        && pval(g, t) >= 4.0
    {
        crate::glog!(g, "    Sakashima's Protege enters as a copy of {}", g.perm(t).name);
        leave(g, src)?;
        let cd = g.perm(src).cd.unwrap();
        remove_card(&mut g.player_mut(o).gy, cd);
        let tcd = g.perm(t).cd.unwrap();
        enter_token_copy(g, o, tcd)?;
    }
    Ok(())
}

/// cascade: a cast trigger
fn protege_cascade(g: &mut Game, o: PlayerId, src: PermId) -> Res {
    if !trigger_window(g, o, Some(src), "cascade", None)? {
        return Ok(());
    }
    let mut cs = vec![];
    while let Some(c) = g.player_mut(o).library.pop() {
        let d = g.db.get(c);
        if !d.land && d.cmc < 6 {
            crate::glog!(g, "    cascade: {}", d.name);
            cs.push(c);
            cast_card(g, o, c, "lib", Ctx::default())?;
            break;
        }
        cs.push(c);
    }
    let pl = g.player(o);
    let back: Vec<CardId> = cs
        .iter()
        .copied()
        .filter(|c| {
            !pl.hand.contains(c)
                && !pl.gy.contains(c)
                && !pl.perms.iter().any(|&x| g.perm(x).cd == Some(*c))
                && !pl.exile.contains(c)
        })
        .collect();
    g.player_mut(o).library.splice(0..0, back);
    Ok(())
}

// ======================================================== Skyclave Apparition
/// engine.apply_removal for Skyclave Apparition: it remembers what it exiled (`src.data['exiled'] = (m.cd, owner)`)
pub fn apparition_note(g: &mut Game, src: PermId, cd: Option<CardId>, owner: PlayerId) {
    let c = cd.map_or(Val::None, Val::Card);
    g.perm_mut(src).data.set(DataKey::Exiled, Val::List(vec![c, Val::Player(owner)]));
}

/// Full: its owner gets an X/X Illusion when the Apparition leaves
fn skyclave_leave(g: &mut Game, _src: Src, m: PermId) -> Res {
    let Some(Val::List(ex)) = data_val(g, m, DataKey::Exiled).cloned() else { return Ok(()) };
    let (Val::Card(cd), Val::Player(owner)) = (&ex[0], &ex[1]) else { return Ok(()) };
    let (cd, owner) = (*cd, *owner);
    if !g.player(owner).alive {
        return Ok(());
    }
    let o = g.perm(m).owner;
    if !trigger_window(g, o, Some(m), "the exiled card's owner creates an X/X Illusion", None)? {
        return Ok(());
    }
    let n = g.db.get(cd).cmc as i32;
    let spec = Tokens { color: Some(Colors::from_letters("U")), types: vec!["illusion"], ..Tokens::new(1, n) };
    make_tokens(g, owner, spec)?;
    crate::glog!(g, "    {} gets a {}/{} Illusion", pname(g, owner), n, n);
    Ok(())
}

// ======================================================== Snap: untap two lands
/// Full: bounces a creature and untaps two lands
fn snap(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) {
        apply_removal(g, Some(p), t, "bounce", Some(c))?;
    }
    let mut n = 0;
    for l in g.player(p).lands.clone() {
        if g.land(l).tapped && n < 2 {
            g.land_mut(l).tapped = false;
            n += 1;
        }
    }
    Ok("gy")
}

// ======================================================== Starfield of Nyx: enchantments become creatures
/// partials.starfield_update: with five enchantments the other non-Aura enchantments are creatures with power and
/// toughness equal to their mana value
pub fn starfield_update(g: &mut Game, p: PlayerId) {
    let ps = g.player(p).perms.clone();
    let on_ = ps.iter().any(|&m| name_of(g, m) == "Starfield of Nyx" && !g.perm(m).phased)
        && ps.iter().filter(|&&m| of_type(g, m, Types::ENCHANTMENT) && !g.perm(m).phased).count() >= 5;
    for m in ps {
        let Some(c) = g.perm(m).cd else { continue };
        let d = g.db.get(c);
        if !d.types.has(Types::ENCHANTMENT) || d.creature || d.has_subtype("aura") || &*d.name == "Starfield of Nyx" {
            continue;
        }
        let cmc = d.cmc as i32;
        let was = g.perm(m).data.truthy(DataKey::Anim);
        let x = g.perm_mut(m);
        if on_ && !was {
            x.data.set(DataKey::Anim, Val::Bool(true));
            x.pow = cmc;
            x.tgh = cmc;
        } else if !on_ && was {
            x.data.set(DataKey::Anim, Val::Bool(false));
            x.pow = 0;
            x.tgh = 0;
        }
    }
    g.bf_ver += 1;
}

/// Full: returns an enchantment each upkeep (its compiled ability); the animation follows the enchantment count
fn starfield_etb(g: &mut Game, src: Src, _p: PlayerId, _m: PermId) -> Res {
    starfield_update(g, g.perm(src).owner);
    Ok(())
}

fn starfield_upkeep(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    starfield_update(g, g.perm(src).owner);
    Ok(())
}

fn starfield_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if g.perm(m).owner == g.perm(src).owner && of_type(g, m, Types::ENCHANTMENT) {
        starfield_update(g, g.perm(src).owner);
    }
    Ok(())
}

fn starfield_gone(g: &mut Game, _src: Src, m: PermId) -> Res {
    let o = g.perm(m).owner;
    for x in g.player(o).perms.clone() {
        if g.perm(x).data.truthy(DataKey::Anim) {
            let y = g.perm_mut(x);
            y.data.set(DataKey::Anim, Val::Bool(false));
            y.pow = 0;
            y.tgh = 0;
        }
    }
    Ok(())
}

// ======================================================== Tekuthal: indestructible counter
fn tekuthal_pool(g: &Game, p: PlayerId, src: PermId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| m != src && (g.perm(m).plus > 0 || g.perm(m).loyalty.unwrap_or(0) > 3))
        .collect()
}

/// Full: removes three counters for an indestructible counter
fn tekuthal(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || g.perm(src).data.truthy(DataKey::Indestr) || post.is_none() {
        return Ok(vec![]);
    }
    let pool = tekuthal_pool(g, p, src);
    let have: i32 = pool.iter().map(|&m| g.perm(m).plus).sum::<i32>()
        + pool.iter().map(|&m| (g.perm(m).loyalty.unwrap_or(0) - 3).max(0)).sum::<i32>();
    if have < 3 || !can_pay(g, p, 1, "", false) || (!can_pay(g, p, 1, "UU", false) && g.player(p).life < 20) {
        return Ok(vec![]);
    }
    Ok(vec![opt_ability(1.5 + 0.2 * pval(g, src), "Tekuthal indestructible".into(), src, tekuthal_go, 0)])
}

fn tekuthal_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let pool = tekuthal_pool(g, p, src);
    if !can_pay(g, p, 1, "", false) {
        return Ok(false);
    }
    if can_pay(g, p, 1, "UU", false) {
        pay(g, p, 1, "UU", false)?;
    } else {
        pay(g, p, 1, "", false)?;
        lose_life(g, p, 4, Some(p), "other", None)?;
    }
    if !ability_window(g, p, Some(src), "an indestructible counter", None, None)? {
        return Ok(true);
    }
    let mut left = 3;
    let w: Vec<f64> = pool.iter().map(|&m| pval(g, m)).collect();
    let mut idx: Vec<usize> = (0..pool.len()).collect();
    idx.sort_by(|&a, &b| w[a].partial_cmp(&w[b]).unwrap_or(std::cmp::Ordering::Equal)); // stable
    for i in idx {
        let m = pool[i];
        while left > 0 && g.perm(m).plus > 0 {
            g.perm_mut(m).plus -= 1;
            left -= 1;
        }
        while left > 0 && g.perm(m).loyalty.unwrap_or(0) > 3 {
            let x = g.perm_mut(m);
            x.loyalty = x.loyalty.map(|l| l - 1);
            left -= 1;
        }
    }
    g.perm_mut(src).data.set(DataKey::Indestr, Val::Bool(true));
    crate::glog!(g, "  {} puts an indestructible counter on Tekuthal", pname(g, p));
    Ok(true)
}

fn tekuthal_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "indestructible" && m == src && g.perm(src).data.truthy(DataKey::Indestr)
}

// ======================================================== Temur Sabertooth
/// {1}{G}: return another creature to hand (and gain indestructible): rebuys an ETB creature when there is mana to
/// recast it this turn (with Chulane: another draw and land)
fn sabertooth(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || post != Some(false) || !can_pay(g, p, 1, "G", false) {
        return Ok(vec![]);
    }
    let chulane = g.player(p).perms.iter().any(|&m| name_of(g, m) == "Chulane, Teller of Tales");
    let mut cs: Vec<(f64, PermId)> = vec![];
    for m in g.player(p).perms.clone() {
        let x = g.perm(m);
        if !g.is_creature(m) || m == src || x.token || x.cd.is_none() || x.is_cmd {
            continue;
        }
        let v = etb_val(g, m) + if chulane { 2.0 } else { 0.0 };
        if v <= 0.0 {
            continue;
        }
        let (gn, pips) = cost_of(g, p, x.cd.unwrap());
        if !can_pay(g, p, 2 + gn, &format!("G{pips}"), false) {
            continue;
        }
        cs.push((v, m));
    }
    let Some((v, m)) = first_max(&cs, |x| x.0) else { return Ok(vec![]) };
    let label = format!("Temur Sabertooth rebuys {}", g.perm(m).name);
    Ok(vec![opt_ability(1.0 + v, label, src, sabertooth_go, m.0 as i64)])
}

fn sabertooth_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let m = PermId(arg as u32);
    if !on(g, p, m) || !can_pay(g, p, 1, "G", false) {
        return Ok(false);
    }
    pay(g, p, 1, "G", false)?;
    let label = format!("return {} to hand", g.perm(m).name);
    if !ability_window(g, p, Some(src), &label, None, Some(m))? {
        return Ok(true);
    }
    if on(g, p, m) {
        bounce(g, m)?;
    }
    eot_kw(g, src, "indestructible");
    crate::glog!(g, "  {} returns {} with Temur Sabertooth", pname(g, p), g.perm(m).name);
    Ok(true)
}

// ======================================================== The One Ring
/// Full: protection from everything until your next turn when cast
fn ring(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src && g.last_cast_etb {
        let o = g.perm(src).owner;
        if !trigger_window(g, o, Some(src), "protection from everything", Some(5.0))? {
            return Ok(());
        }
        g.player_mut(o).ring_prot = true;
        crate::glog!(g, "    {} gains protection from everything until their next turn", pname(g, o));
    }
    Ok(())
}

fn burden(g: &Game, src: PermId) -> i64 {
    g.perm(src).data.get(DataKey::Burden).map_or(0, Val::int)
}

/// upkeep: lose 1 life per burden counter
fn ring_burden(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == g.perm(src).owner
        && burden(g, src) != 0
        && trigger_window(g, p, Some(src), "lose 1 life per burden counter", None)?
        && burden(g, src) != 0
    {
        let n = burden(g, src) as i32;
        lose_life(g, p, n, Some(p), "other", None)?;
    }
    Ok(())
}

/// {T}: a burden counter, draw that many
fn ring_draw(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || g.perm(src).tapped {
        return Ok(vec![]);
    }
    let b = burden(g, src) as i32;
    if g.player(p).life - (b + 1) * 2 < 12 && b >= 2 {
        return Ok(vec![]);
    }
    Ok(vec![opt_ability(2.0 + b as f64, "The One Ring".into(), src, ring_go, 0)])
}

fn ring_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if g.perm(src).tapped {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if !ability_window(g, p, Some(src), "a burden counter, then draw", None, None)? {
        return Ok(true);
    }
    let b = burden(g, src) + 1;
    g.perm_mut(src).data.set(DataKey::Burden, Val::Int(b));
    draw(g, p, b as u32, false)?;
    Ok(true)
}

/// gc_prio.one_ring_prio
fn ring_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    crate::ai::plans::one_ring_prio(g, p, c)
}

// ======================================================== Tishana's Tidebinder (practice mode's own enters)
/// practice mode: when your Tidebinder enters, you may counter an activated or triggered ability on the stack. The
/// AI's own Tidebinder plays are in answer_ability and tidebinder_response, so for the AI it does nothing.
fn tide_etb_you(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if g.humans.get(o).is_none() {
        return Ok(());
    }
    // HUMAN(phase 9): the person picks an ability on the stack to counter (trigger_window 'counter an ability')
    Ok(())
}

// ======================================================== Voltaic Key
/// Full: {1},{T}: untap an artifact (the best mana rock: net mana), also used in the Monolith loops
fn voltaic_key(g: &Game, src: Src, p: PlayerId, _u: &[Unit]) -> Vec<Unit> {
    if g.perm(src).owner != p || g.perm(src).tapped {
        return vec![];
    }
    let best = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| m != src)
        .filter_map(|&m| {
            let d = g.db.get(g.perm(m).cd?);
            if !d.types.has(Types::ARTIFACT) {
                return None;
            }
            d.tags.str(Tag::Rock).and_then(|r| r.split(':').next()).and_then(|x| x.parse::<u32>().ok())
        })
        .max()
        .unwrap_or(0);
    if best < 2 {
        return vec![];
    }
    vec![Unit { src: Source::Perm(src), cols: Colors::NONE, amt: best - 1 }]
}

// ======================================================== Whirler Rogue
/// taps two artifacts to make an attacker (a Ninja first) unblockable
fn whirler(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    if p != g.perm(src).owner {
        return Ok(vec![]);
    }
    let arts: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            !x.tapped
                && !atk.contains(&m)
                && ((x.token && x.ttypes.contains(&"artifact")) || of_type(g, m, Types::ARTIFACT))
        })
        .collect();
    if arts.len() < 2 {
        return Ok(vec![]);
    }
    if !g.player(d).perms.iter().any(|&b| g.is_creature(b) && !g.perm(b).tapped) {
        return Ok(vec![]);
    }
    let cands: Vec<PermId> =
        atk.iter().copied().filter(|&a| on(g, p, a) && !crate::dsl::has_kw(g, a, "unblockable")).collect();
    // t4._ninja(a) or a.is_cmd, then power
    let Some(a) = first_max(&cands, |a| (has_type(g, a, "ninja") || g.perm(a).is_cmd, epow(g, a))) else {
        return Ok(vec![]);
    };
    for &m in arts.iter().take(2) {
        g.perm_mut(m).tapped = true;
    }
    eot_kw(g, a, "unblockable");
    crate::glog!(g, "    Whirler Rogue: {} can't be blocked", g.perm(a).name);
    Ok(vec![])
}

/// Full: two flying Thopters
fn whirler_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && trigger_window(g, o, Some(src), "create two 1/1 Thopters", None)? {
        let spec = Tokens { fly: true, types: vec!["thopter", "artifact"], ..Tokens::new(2, 1) };
        make_tokens(g, o, spec)?;
    }
    Ok(())
}

// ======================================================== Druid Class level 2 (the fix)
fn druid2_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !can_pay(g, p, 2, "G", false) {
        return Ok(false);
    }
    pay(g, p, 2, "G", false)?;
    if ability_window(g, p, Some(src), "level 2", None, None)? && on(g, p, src) {
        let mut d = crate::state::PermData::default(); // t1: `src.data = {'level': 2}`
        d.set(DataKey::Level, Val::Int(2));
        g.perm_mut(src).data = d;
    }
    Ok(true)
}

// ======================================================== Mardu Charm's mode (the fix)
/// what Mardu Charm's discard mode is worth: the most threatening opponent's hand (its size is public), the card it
/// would take being a noncreature, nonland one (a counterspell, a combo piece, a wipe)
fn mardu_discard_value(g: &Game, p: PlayerId) -> f64 {
    let os: Vec<PlayerId> = g.opps(p).collect();
    let Some(q) = max_by(&os, |q| threat(g, p, q)) else { return 0.0 };
    0.3 * g.player(q).hand.len() as f64
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
    register_phase6(r, db)
}

/// the rest of partials.py (phase 6), in the Python's order
fn register_phase6(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Archangel Avacyn // Avacyn, the Purifier")?;
    c.dies = Some(avacyn_flag);
    at_once(c, Event::Dies, false);
    c.upkeep = Some(avacyn_flip);
    at_once(c, Event::Upkeep, false);
    let c = r.card(db, "Archfiend of Sorrows")?;
    c.gy_options = Some(archfiend_unearth);
    c.end_step = Some(archfiend_exile);
    at_once(c, Event::EndStep, false);
    r.card(db, "Aven Mindcensor")?.search_limit = Some(mindcensor);
    let c = r.card(db, KEEPER)?;
    c.options = Some(keeper);
    c.self_pt = Some(keeper_pt);
    c.etb = Some(keeper_on);
    at_once(c, Event::Etb, true);
    let c = r.card(db, "Chain of Vapor")?;
    c.resolve = Some(chain);
    c.prio = Some(prio_0);
    let c = r.card(db, "Coat of Arms")?;
    c.etb = Some(coat_on);
    at_once(c, Event::Etb, true);
    let c = r.card(db, "Conduit of Worlds")?;
    c.options = Some(conduit);
    c.can_cast = Some(conduit_lock);
    r.card(db, "Conspicuous Snoop")?.options = Some(snoop);
    let c = r.card(db, "Crypt Ghast")?;
    c.cast = Some(extort);
    at_once(c, Event::Cast, false);
    let c = r.card(db, "Treasure Cruise")?;
    c.resolve = Some(cruise);
    c.prio = Some(prio_48);
    c.self_cost = Some(delve_cost);
    let c = r.card(db, "Dig Through Time")?;
    c.resolve = Some(dig);
    c.prio = Some(prio_50);
    c.self_cost = Some(delve_cost);
    let c = r.card(db, "Draco")?;
    c.self_cost = Some(draco_cost);
    c.upkeep = Some(draco_upkeep);
    at_once(c, Event::Upkeep, false);
    c.prio = Some(draco_prio);
    r.card(db, "Elvish Spirit Guide")?.prio = Some(prio_0);
    let c = r.card(db, "Enter the Infinite")?;
    c.resolve = Some(eti);
    c.prio = Some(prio_70);
    let c = r.card(db, "Faerie Mastermind")?;
    c.draw = Some(mastermind);
    at_once(c, Event::Draw, false);
    let c = r.card(db, "Gitaxian Probe")?;
    c.resolve = Some(probe);
    c.prio = Some(prio_40);
    r.card(db, "Gix, Yawgmoth Praetor")?.options = Some(gix);
    r.card(db, "Goblin Cratermaker")?.options = Some(cratermaker);
    let c = r.card(db, "Golgari Charm")?;
    c.resolve = Some(golgari_charm);
    c.hand_options = Some(golgari_opts);
    c.prio = Some(prio_0);
    r.card(db, "Grim Hireling")?.options = Some(hireling);
    let c = r.card(db, "Hope of Ghirapur")?;
    c.combat_damage = Some(hope_hit);
    at_once(c, Event::CombatDamage, true);
    c.options = Some(hope_sac);
    let c = r.card(db, "Hunter's Insight")?;
    c.resolve = Some(insight);
    c.hand_options = Some(insight_opt);
    c.prio = Some(prio_0);
    let c = r.card(db, "Lim-Dûl's Vault")?;
    c.resolve = Some(vault);
    c.prio = Some(vault_prio);
    r.card(db, "Loran of the Third Path")?.options = Some(loran);
    let c = r.card(db, "Massacre Girl")?;
    c.etb = Some(massacre);
    at_once(c, Event::Etb, false);
    c.prio = Some(massacre_prio);
    for name in ["Mishra's Bauble", "Urza's Bauble"] {
        r.card(db, name)?.options = Some(bauble);
    }
    r.card(db, "Mogg Fanatic")?.options = Some(fanatic);
    r.card(db, "Multani, Yavimaya's Avatar")?.gy_options = Some(multani);
    let c = r.card(db, "Muxus, Goblin Grandee")?;
    c.attack = Some(muxus_attack);
    at_once(c, Event::Attack, false);
    let c = r.card(db, "Oath of Teferi")?;
    c.etb = Some(oath);
    at_once(c, Event::Etb, false);
    c.end_step = Some(oath_back);
    at_once(c, Event::EndStep, false);
    r.card(db, "Paradise Druid")?.grant_kw = Some(pdruid);
    r.card(db, "Pashalik Mons")?.options = Some(mons);
    r.card(db, "Quirion Ranger")?.extra_mana = Some(quirion);
    r.card(db, "Wirewood Symbiote")?.extra_mana = Some(symbiote);
    for name in ["Wheel of Fortune", "Reforge the Soul"] {
        let c = r.card(db, name)?;
        c.resolve = Some(wheel_spell);
        c.prio = Some(wheel_prio);
    }
    r.card(db, "Sai, Master Thopterist")?.options = Some(sai);
    let c = r.card(db, "Sakashima's Protege")?;
    c.etb = Some(protege);
    at_once(c, Event::Etb, true);
    let c = r.card(db, "Skyclave Apparition")?;
    c.leaves = Some(skyclave_leave);
    at_once(c, Event::Leaves, false);
    r.card(db, "Snap")?.resolve = Some(snap);
    let c = r.card(db, "Starfield of Nyx")?;
    c.etb = Some(starfield_etb);
    at_once(c, Event::Etb, true);
    c.upkeep = Some(starfield_upkeep);
    at_once(c, Event::Upkeep, true);
    c.dies = Some(starfield_dies);
    at_once(c, Event::Dies, true);
    c.leaves = Some(starfield_gone);
    at_once(c, Event::Leaves, true);
    let c = r.card(db, "Tekuthal, Inquiry Dominus")?;
    c.options = Some(tekuthal);
    c.grant_kw = Some(tekuthal_kw);
    r.card(db, "Temur Sabertooth")?.options = Some(sabertooth);
    let c = r.card(db, "The One Ring")?;
    c.etb = Some(ring);
    at_once(c, Event::Etb, false);
    c.upkeep = Some(ring_burden);
    at_once(c, Event::Upkeep, false);
    c.options = Some(ring_draw);
    c.prio = Some(ring_prio);
    let c = r.card(db, "Tishana's Tidebinder")?;
    c.etb = Some(tide_etb_you);
    at_once(c, Event::Etb, false);
    r.card(db, "Voltaic Key")?.extra_mana = Some(voltaic_key);
    let c = r.card(db, "Whirler Rogue")?;
    c.attack = Some(whirler);
    at_once(c, Event::Attack, true);
    c.etb = Some(whirler_etb);
    at_once(c, Event::Etb, false);
    Ok(())
}
