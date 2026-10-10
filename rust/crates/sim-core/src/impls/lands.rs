//! Python's `cards/impl/lands.py`: utility lands for the opponent pools: creature lands, channel lands,
//! sacrifice-to-draw lands, land destruction, graveyard lands and the rest. Lands are not permanents in `perms`, so
//! their abilities are offered through the `land_options` slot (brain.hook_options, through `land_options` here)
//! and upkeep through `land_upkeep` (cardcode::turn_start).
//!
//! Creature lands become a creature (a token standing in for the land) until end of turn: the land leaves the
//! player's lands while animated and comes back at the start of the next turn; if the creature died, the land goes
//! to the graveyard.

use crate::cards::{CardDb, Colors, Types};
use crate::engine::hooks::fire_trigger;
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, pay, total_mana};
use crate::engine::removal::apply_removal;
use crate::engine::stack::ability_window_card;
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{epow, etgh, pval, threat, untargetable};
use crate::engine::zones::{Enter, Tokens, draw, enter, land_ramp, make_tokens, max_by, mill, searchable};
use crate::flow::Res;
use crate::hooks::{Action, Assign, Call, Event, LandOptionsFn, Opt, Registry};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
use crate::sym::intern;
use crate::tag::Tag;

pub const BASICS: [&str; 6] = ["Plains", "Island", "Swamp", "Mountain", "Forest", "Wastes"];

// ------------------------------------------------------------------ helpers
/// Python's `max(xs, key=...)` with a tuple key: the first item with the highest key
fn first_max<T: Copy, K: PartialOrd>(xs: &[T], key: impl Fn(T) -> K) -> Option<T> {
    let mut best: Option<(T, K)> = None;
    for &x in xs {
        let k = key(x);
        if best.as_ref().is_none_or(|b| k > b.1) {
            best = Some((x, k));
        }
    }
    best.map(|b| b.0)
}

fn name_of(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn land_name(g: &Game, l: LandId) -> &str {
    name_of(g, g.land(l).cd)
}

fn has_land(g: &Game, p: PlayerId, l: LandId) -> bool {
    g.player(p).lands.contains(&l)
}

fn opt(utility: f64, label: String, f: crate::hooks::PlanFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Plan { f, arg }) }
}

/// two ids in one option argument (a land and the card, permanent or land it acts on)
fn pack(a: u32, b: u32) -> i64 {
    ((b as i64) << 32) | a as i64
}

fn unpack(arg: i64) -> (u32, u32) {
    ((arg & 0xffff_ffff) as u32, (arg >> 32) as u32)
}

fn remove_first(v: &mut Vec<CardId>, c: CardId) -> bool {
    match v.iter().position(|&x| x == c) {
        Some(i) => {
            v.remove(i);
            true
        }
        None => false,
    }
}

/// lands.land_options: the activated abilities of p's lands (each land's `land_options` slot)
pub fn land_options(g: &mut Game, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut o = vec![];
    for l in g.player(p).lands.clone() {
        if let Some(f) = g.registry.get(g.land(l).cd).and_then(|i| i.land_options) {
            o.extend(f(g, l, p, post)?);
        }
    }
    Ok(o)
}

/// lands.land_upkeep: p's lands' upkeep abilities (Emeria, the Sky Ruin)
pub fn land_upkeep(g: &mut Game, p: PlayerId) -> Res {
    for l in g.player(p).lands.clone() {
        if let Some(f) = g.registry.get(g.land(l).cd).and_then(|i| i.land_upkeep) {
            f(g, l, p)?;
        }
    }
    Ok(())
}

/// lands.pay_without: pay a land's activation cost with other sources (the land itself stays untapped)
pub fn pay_without(g: &mut Game, p: PlayerId, l: LandId, generic: u32, pips: &str) -> Res<bool> {
    let Some(i) = g.player(p).lands.iter().position(|&x| x == l) else { return Ok(false) };
    g.player_mut(p).lands.remove(i);
    let r = if can_pay(g, p, generic, pips, false) { pay(g, p, generic, pips, false).map(|_| true) } else { Ok(false) };
    let lands = &mut g.player_mut(p).lands;
    lands.insert(i.min(lands.len()), l);
    r
}

/// lands.can_pay_without: could p pay this with sources other than the land?
pub fn can_pay_without(g: &mut Game, p: PlayerId, l: LandId, generic: u32, pips: &str) -> bool {
    let Some(i) = g.player(p).lands.iter().position(|&x| x == l) else { return false };
    g.player_mut(p).lands.remove(i);
    let ok = can_pay(g, p, generic, pips, false);
    g.player_mut(p).lands.insert(i, l);
    ok
}

/// lands.sac_land: p sacrifices land l (it goes to the graveyard)
pub fn sac_land(g: &mut Game, p: PlayerId, l: LandId) -> Res {
    let cd = g.land(l).cd;
    if has_land(g, p, l) {
        crate::engine::turn::remove_land(g, p, l);
        g.player_mut(p).gy.push(cd);
    }
    if !g.hooks.is_empty() {
        fire_trigger(g, Event::LandGy, Call::Cards { p, cards: vec![cd] })?;
    }
    Ok(())
}

/// fixes.destroy_land (a copy: fixes.rs is ported separately; the same few lines)
fn destroy_land(g: &mut Game, q: PlayerId, l: LandId) -> Res {
    if !has_land(g, q, l) {
        return Ok(());
    }
    let cd = g.land(l).cd;
    crate::engine::turn::remove_land(g, q, l);
    g.player_mut(q).gy.push(cd);
    crate::glog!(g, "    {} ({}) is destroyed", name_of(g, cd), g.player(q).name);
    if !g.hooks.is_empty() {
        fire_trigger(g, Event::LandGy, Call::Cards { p: q, cards: vec![cd] })?;
    }
    Ok(())
}

/// common.best_opp_nonland (a copy: common.rs is ported separately): the most valuable opposing permanent passing
/// `pred`
fn best_opp_nonland(g: &Game, p: PlayerId, pred: impl Fn(PermId) -> bool) -> Option<PermId> {
    let cs: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| !g.perm(m).phased && !untargetable(g, m) && pred(m))
        .collect();
    max_by(&cs, |m| pval(g, m))
}

// ------------------------------------------------------------------ creature lands
/// lands.revert_animated: at the start of a turn, animated lands stop being creatures (or went to the graveyard if
/// the creature died)
pub fn revert_animated(g: &mut Game) -> Res {
    if g.animated.is_empty() {
        return Ok(());
    }
    for (p, m, l) in std::mem::take(&mut g.animated) {
        if g.perm(m).on_bf && g.perm(m).owner == p {
            g.player_mut(p).perms.retain(|&x| x != m);
            g.perm_mut(m).on_bf = false;
            g.land_mut(l).tapped = g.perm(m).tapped;
            let plus = g.perm(m).plus;
            if plus != 0 && land_name(g, l) == "Raging Ravine" {
                l_counters_set(g, l, plus);
            }
            g.land_mut(l).on_bf = true;
            g.player_mut(p).lands.push(l);
        } else if g.perm(m).owner == p && g.player(p).alive {
            let cd = g.land(l).cd;
            g.player_mut(p).gy.push(cd);
        }
    }
    Ok(())
}

/// lands.L_counters: +1/+1 counters kept on a creature land (Raging Ravine) between animations
pub fn l_counters(g: &Game, l: LandId) -> i32 {
    g.land(l).data.int(DataKey::Counters) as i32
}

fn l_counters_set(g: &mut Game, l: LandId, n: i32) {
    g.land_mut(l).data.set(DataKey::Counters, Val::Int(n as i64));
}

/// lands.animate: land l becomes a pw/tg creature until the start of the next turn (a token standing in for it).
/// Python marks it summoning sick only when it is `p.last_land_cd`, an attribute nothing sets, so an animated land is
/// never sick (the land just played doesn't offer the ability anyway).
pub fn animate(g: &mut Game, p: PlayerId, l: LandId, pw: i32, tg: i32, kws: &[&'static str], fly: bool) -> PermId {
    crate::engine::turn::remove_land(g, p, l);
    let name = intern(land_name(g, l));
    let tapped = g.land(l).tapped;
    let plus = l_counters(g, l);
    let m = g.new_perm(p, None, name, pw, tg);
    let x = g.perm_mut(m);
    x.fly = fly;
    x.tapped = tapped;
    x.sick = false;
    x.dt = kws.contains(&"deathtouch");
    x.lifelink = kws.contains(&"lifelink");
    x.ttypes = vec!["land"];
    x.plus = plus;
    x.data.set(DataKey::Land, Val::Land(l));
    for &k in kws {
        if !x.eot_kw.contains(&k) {
            x.eot_kw.push(k);
        }
    }
    x.on_bf = true;
    g.player_mut(p).perms.push(m);
    g.animated.push((p, m, l));
    crate::glog!(g, "  {} animates {}", g.player(p).name, name);
    m
}

/// lands.open_attack: is there an opponent this creature could hit (no untapped blocker that stops it)?
pub fn open_attack(g: &Game, p: PlayerId, power: i32, evasive: bool) -> bool {
    for q in g.opps(p) {
        let blockers: Vec<PermId> = g
            .player(q)
            .perms
            .iter()
            .copied()
            .filter(|&b| g.is_creature(b) && !g.perm(b).tapped && !g.perm(b).phased)
            .collect();
        if evasive || blockers.is_empty() || blockers.iter().map(|&b| etgh(g, b)).max().unwrap_or(0) < power {
            return true;
        }
    }
    false
}

/// a creature land: {gen}{pips}: pw/tg with these keywords until end of turn
struct Manland {
    name: &'static str,
    generic: u32,
    pips: &'static str,
    pw: i32,
    tg: i32,
    kws: &'static [&'static str],
    fly: bool,
}

const MANLANDS: [Manland; 5] = [
    // {1}: 2/2 with all creature types until end of turn (attacks when open)
    Manland { name: "Mutavault", generic: 1, pips: "", pw: 2, tg: 2, kws: &[], fly: false },
    Manland { name: "Hissing Quagmire", generic: 1, pips: "BG", pw: 2, tg: 2, kws: &["deathtouch"], fly: false },
    Manland { name: "Needle Spires", generic: 2, pips: "RW", pw: 2, tg: 1, kws: &["double strike"], fly: false },
    // {2}{R}{G}: 3/3 until end of turn; a +1/+1 counter each time it attacks (kept on the land: CI.keyword_attack)
    Manland { name: "Raging Ravine", generic: 2, pips: "RG", pw: 3, tg: 3, kws: &[], fly: false },
    // {3}{B}: 3/3 menace until end of turn; exiles a card from the defender's graveyard when it attacks
    // (CI.keyword_attack)
    Manland { name: "Hive of the Eye Tyrant", generic: 3, pips: "B", pw: 3, tg: 3, kws: &["menace"], fly: false },
];

fn manland_of(g: &Game, l: LandId) -> &'static Manland {
    let n = land_name(g, l);
    MANLANDS.iter().find(|x| x.name == n).expect("a creature land")
}

/// lands.manland: animated to attack when it has an open attack
fn manland_options(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let ml = manland_of(g, l);
    if post != Some(false) || g.land(l).tapped || !can_pay_without(g, p, l, ml.generic, ml.pips) {
        return Ok(vec![]);
    }
    let pl = g.player(p);
    if pl.land_turn == pl.turns as i32 && pl.lands.last() == Some(&l) {
        return Ok(vec![]); // just played: it would be summoning sick
    }
    let spare = total_mana(g, p, false) as i64 - (ml.generic as i64 + ml.pips.len() as i64) - 1;
    let evasive = ml.fly || ml.kws.contains(&"menace");
    let power = ml.pw + l_counters(g, l);
    if !open_attack(g, p, power, evasive) {
        return Ok(vec![]);
    }
    let u = 0.8 + 0.35 * power as f64 + if spare >= 2 { 0.5 } else { -0.8 };
    Ok(vec![opt(u, format!("animate {}", ml.name), manland_go, l.0 as i64)])
}

fn manland_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId(arg as u32);
    let ml = manland_of(g, l);
    if !has_land(g, p, l) || g.land(l).tapped || !pay_without(g, p, l, ml.generic, ml.pips)? {
        return Ok(false);
    }
    if !ability_window_card(g, p, g.land(l).cd, "becomes a creature", None, None)? {
        return Ok(true);
    }
    animate(g, p, l, ml.pw, ml.tg, ml.kws, ml.fly);
    Ok(true)
}

// ------------------------------------------------------------------ sacrifice-to-draw lands and pain lands
/// lands._pain: mana costs 1 life
fn pain(g: &mut Game, p: PlayerId, _l: LandId, _n: u32) -> Res {
    lose_life(g, p, 1, Some(p), "other", None)
}

/// lands.cycler_land: {1}, {T}, sacrifice: draw a card (used when flooded); mana costs 1 life
fn cycler_options(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let cost = 1;
    if g.land(l).tapped || !can_pay_without(g, p, l, cost, "") {
        return Ok(vec![]);
    }
    let pl = g.player(p);
    let lands = pl.lands.len() + pl.hand.iter().filter(|&&c| g.db.get(c).land).count();
    if lands < 7 && !(post.is_none() && lands >= 5) {
        return Ok(vec![]);
    }
    let u = if post.is_none() { 1.2 } else { 0.6 };
    Ok(vec![opt(u, format!("{} draw", land_name(g, l)), cycler_go, l.0 as i64)])
}

fn cycler_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId(arg as u32);
    if !has_land(g, p, l) || g.land(l).tapped || !pay_without(g, p, l, 1, "")? {
        return Ok(false);
    }
    sac_land(g, p, l)?;
    crate::glog!(g, "  {} sacrifices {}: draws a card", g.player(p).name, land_name(g, l));
    if ability_window_card(g, p, g.land(l).cd, "draw a card", None, None)? {
        draw(g, p, 1, false)?;
    }
    Ok(true)
}

// ------------------------------------------------------------------ channel lands (from hand)
const BOSEIJU: &str = "Boseiju, Who Endures";

fn boseiju_pred(g: &Game, m: PermId) -> bool {
    g.perm(m).cd.is_some_and(|c| {
        let t = g.db.get(c).types;
        !g.is_creature(m) && (t.has(Types::ARTIFACT) || t.has(Types::ENCHANTMENT))
    })
}

/// channel {1}{G}: destroy an opponent's artifact, enchantment or nonbasic land (they may search for a basic); used
/// on a valuable artifact or enchantment
fn boseiju(g: &mut Game, _c: CardId, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if !can_pay(g, p, 1, "G", false) {
        return Ok(vec![]);
    }
    let Some(t) = best_opp_nonland(g, p, |m| boseiju_pred(g, m)) else { return Ok(vec![]) };
    let v = pval(g, t);
    if v < 4.0 {
        return Ok(vec![]);
    }
    Ok(vec![opt(v - 2.0, "channel Boseiju".into(), boseiju_go, t.0 as i64)])
}

fn boseiju_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    let Some(c) = g.db.id(BOSEIJU) else { return Ok(false) };
    if !g.player(p).hand.contains(&c) || !g.perm(t).on_bf || !can_pay(g, p, 1, "G", false) {
        return Ok(false);
    }
    remove_first(&mut g.player_mut(p).hand, c);
    pay(g, p, 1, "G", false)?;
    g.player_mut(p).gy.push(c);
    crate::glog!(g, "  {} channels Boseiju", g.player(p).name);
    apply_removal(g, Some(p), t, "destroy", None)?;
    Ok(true)
}

const SOKENZAN: &str = "Sokenzan, Crucible of Defiance";

/// channel {3}{R} (less per legendary creature): two hasty 1/1 Spirits, when lands are plentiful
fn sokenzan(g: &mut Game, _c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let pl = g.player(p);
    let leg = pl
        .perms
        .iter()
        .filter(|&&m| g.is_creature(m) && g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Leg)))
        .count() as u32;
    let cost = 3u32.saturating_sub(leg);
    let lands = pl.lands.len() + pl.hand.iter().filter(|&&x| g.db.get(x).land).count();
    if post != Some(false) || !can_pay(g, p, cost, "R", false) || lands < 6 {
        return Ok(vec![]);
    }
    Ok(vec![opt(1.5, "channel Sokenzan".into(), sokenzan_go, cost as i64)])
}

fn sokenzan_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let cost = arg as u32;
    let Some(c) = g.db.id(SOKENZAN) else { return Ok(false) };
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, cost, "R", false) {
        return Ok(false);
    }
    remove_first(&mut g.player_mut(p).hand, c);
    pay(g, p, cost, "R", false)?;
    g.player_mut(p).gy.push(c);
    make_tokens(g, p, Tokens { sick: false, types: vec!["spirit"], ..Tokens::new(2, 1) })?;
    crate::glog!(g, "  {} channels Sokenzan", g.player(p).name);
    Ok(true)
}

// ------------------------------------------------------------------ land destruction lands
pub const KEY_LANDS: [&str; 16] = [
    "Gaea's Cradle",
    "Cabal Coffers",
    "Urborg, Tomb of Yawgmoth",
    "Ancient Tomb",
    "Urza's Saga",
    "Nykthos, Shrine to Nyx",
    "Serra's Sanctum",
    "Itlimoc, Cradle of the Sun",
    "Tolarian Academy",
    "Dark Depths",
    "Field of the Dead",
    "Mishra's Workshop",
    "Glacial Chasm",
    "Maze of Ith",
    "Kor Haven",
    "Gavony Township",
];

/// lands.best_key_land: the key nonbasic land (KEY_LANDS, or one making two or more mana) of the most threatening
/// opponent
pub fn best_key_land(g: &Game, p: PlayerId) -> Option<(PlayerId, LandId)> {
    let cands: Vec<(PlayerId, LandId)> = g
        .opps(p)
        .flat_map(|q| g.player(q).lands.iter().map(move |&l| (q, l)))
        .filter(|&(_, l)| !BASICS.contains(&land_name(g, l)))
        .collect();
    let key: Vec<(PlayerId, LandId)> = cands
        .into_iter()
        .filter(|&(_, l)| {
            KEY_LANDS.contains(&land_name(g, l)) || g.db.get(g.land(l).cd).tags.int(Tag::Amt).unwrap_or(1) >= 2
        })
        .collect();
    max_by(&key, |x| threat(g, p, x.0))
}

/// Ghost Quarter: T, sacrifice: destroys a key opposing land (its controller fetches a basic). Tectonic Edge: {1},
/// T, sacrifice: destroys a key nonbasic land of an opponent with four or more lands.
fn ld_land_options(g: &mut Game, l: LandId, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    let name = land_name(g, l);
    let cost = if name == "Tectonic Edge" { 1 } else { 0 };
    if g.land(l).tapped || !can_pay_without(g, p, l, cost, "") {
        return Ok(vec![]);
    }
    let Some((q, t)) = best_key_land(g, p) else { return Ok(vec![]) };
    if land_name(g, l) == "Tectonic Edge" && g.player(q).lands.len() < 4 {
        return Ok(vec![]);
    }
    let label = format!("{} -> {}", land_name(g, l), land_name(g, t));
    Ok(vec![opt(2.5, label, ld_land_go, pack(l.0, t.0))])
}

fn ld_land_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (l, t) = unpack(arg);
    let (l, t) = (LandId(l), LandId(t));
    let ghost = land_name(g, l) == "Ghost Quarter";
    let cost = if ghost { 0 } else { 1 };
    if !has_land(g, p, l) || g.land(l).tapped || !pay_without(g, p, l, cost, "")? {
        return Ok(false);
    }
    let q = g.land(t).owner;
    if !has_land(g, q, t) {
        return Ok(false);
    }
    sac_land(g, p, l)?;
    let what = format!("destroy {}", land_name(g, t));
    if !ability_window_card(g, p, g.land(l).cd, &what, None, None)? || !has_land(g, q, t) {
        return Ok(true);
    }
    destroy_land(g, q, t)?;
    if ghost {
        land_ramp(g, q, 1, true)?;
    }
    Ok(true)
}

// ------------------------------------------------------------------ mana and value lands
/// enters tapped without another land early; {1}{B}{B}, {T}: draw a card, then lose life equal to the cards in
/// hand (at the end of an opponent's turn, when life allows)
fn locthwain(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || g.land(l).tapped || !can_pay_without(g, p, l, 1, "BB") {
        return Ok(vec![]);
    }
    if g.player(p).life - (g.player(p).hand.len() as i32 + 1) < 15 {
        return Ok(vec![]);
    }
    Ok(vec![opt(1.5, "Castle Locthwain".into(), locthwain_go, l.0 as i64)])
}

fn locthwain_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId(arg as u32);
    if !has_land(g, p, l) || g.land(l).tapped || !pay_without(g, p, l, 1, "BB")? {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    crate::glog!(g, "  {} uses Castle Locthwain", g.player(p).name);
    if ability_window_card(g, p, g.land(l).cd, "draw a card", None, None)? {
        draw(g, p, 1, false)?;
        let n = g.player(p).hand.len() as i32;
        lose_life(g, p, n, Some(p), "other", None)?;
    }
    Ok(true)
}

/// creatures you control can attack: untapped, not summoning sick, able to attack
fn ready_attackers(g: &Game, p: PlayerId) -> usize {
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| {
            let x = g.perm(m);
            g.is_creature(m) && !x.sick && !x.tapped && !x.noatk
        })
        .count()
}

/// {1}{R}{R},{T}: creatures you control get +1/+0 until end of turn (before an attack with four or more)
fn embereth(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || g.land(l).tapped || !can_pay_without(g, p, l, 1, "RR") {
        return Ok(vec![]);
    }
    let ready = ready_attackers(g, p);
    if ready < 4 {
        return Ok(vec![]);
    }
    Ok(vec![opt(0.5 + 0.35 * ready as f64, "Castle Embereth".into(), embereth_go, l.0 as i64)])
}

fn embereth_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId(arg as u32);
    if !has_land(g, p, l) || g.land(l).tapped || !pay_without(g, p, l, 1, "RR")? {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    crate::glog!(g, "  {} uses Castle Embereth", g.player(p).name);
    if ability_window_card(g, p, g.land(l).cd, "creatures get +1/+0", None, None)? {
        for m in g.player(p).perms.clone() {
            if g.is_creature(m) {
                g.perm_mut(m).eot_pt.0 += 1;
            }
        }
    }
    Ok(true)
}

/// {3},{T}, pay life equal to the colors in your commander's identity: draw a card (at the end of an opponent's
/// turn)
fn war_room(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let n = g.player(p).ident.count() as i32;
    if post.is_some() || g.land(l).tapped || !can_pay_without(g, p, l, 3, "") || g.player(p).life - n < 12 {
        return Ok(vec![]);
    }
    Ok(vec![opt(1.3, "War Room".into(), war_room_go, l.0 as i64)])
}

fn war_room_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId(arg as u32);
    let n = g.player(p).ident.count() as i32;
    if !has_land(g, p, l) || g.land(l).tapped || !pay_without(g, p, l, 3, "")? {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    lose_life(g, p, n, Some(p), "other", None)?;
    crate::glog!(g, "  {} uses War Room", g.player(p).name);
    if ability_window_card(g, p, g.land(l).cd, "draw a card", None, None)? {
        draw(g, p, 1, false)?;
    }
    Ok(true)
}

/// Karn's Bastion (Full): {4},{T}: proliferate (at end of turn, when it helps; in your main phase for a lore counter
/// on Summon: Bahamut, a Saga whose CI.SAGA rule wants one)
fn bastion(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let saga: i32 = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| {
            let x = g.perm(m);
            x.data.get(DataKey::Lore).is_some()
                && !x.phased
                && x.cd.and_then(|c| g.registry.get(c)).and_then(|i| i.saga).is_some_and(|s| (s.0)(g, p, m))
        })
        .count() as i32
        * 2;
    if (post.is_some() && saga == 0) || g.land(l).tapped || !can_pay_without(g, p, l, 4, "") {
        return Ok(vec![]);
    }
    let mine =
        g.player(p).perms.iter().filter(|&&m| g.perm(m).plus > 0 || g.perm(m).loyalty.is_some_and(|x| x != 0)).count();
    let theirs = g.opps(p).flat_map(|q| g.player(q).perms.iter()).filter(|&&m| g.perm(m).plus > 0).count();
    let worth = mine as i32 - theirs as i32 + saga;
    if worth < 2 {
        return Ok(vec![]);
    }
    let u = 0.8 + 0.3 * worth as f64 + if saga != 0 { 2.0 } else { 0.0 }; // a chapter: a destroy, two cards
    Ok(vec![opt(u, "Karn's Bastion".into(), bastion_go, l.0 as i64)])
}

fn bastion_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId(arg as u32);
    if !has_land(g, p, l) || g.land(l).tapped || !pay_without(g, p, l, 4, "")? {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    g.player_mut(p).stat("proliferate_bastion", 1);
    if ability_window_card(g, p, g.land(l).cd, "proliferate", None, None)? {
        super::common::proliferate(g, p, 1)?;
    }
    Ok(true)
}

/// {1}{U},{T}: the best artifact from the graveyard on top of the library
fn academy_ruins(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || g.land(l).tapped || !can_pay_without(g, p, l, 1, "U") {
        return Ok(vec![]);
    }
    let arts: Vec<CardId> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&c| g.db.get(c).types.has(Types::ARTIFACT) && !g.db.get(c).land)
        .collect();
    let Some(c) = first_max(&arts, |c| (card_worth(g, p, c, false), g.db.get(c).cmc)) else { return Ok(vec![]) };
    if card_worth(g, p, c, false) < 40.0 {
        return Ok(vec![]);
    }
    Ok(vec![opt(1.5, "Academy Ruins".into(), ruins_go, pack(l.0, c.0 as u32))])
}

fn ruins_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (l, c) = unpack(arg);
    let (l, c) = (LandId(l), CardId(c as u16));
    if !has_land(g, p, l) || g.land(l).tapped || !g.player(p).gy.contains(&c) || !pay_without(g, p, l, 1, "U")? {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    crate::glog!(g, "  {} puts {} on top with Academy Ruins", g.player(p).name, name_of(g, c));
    let what = format!("{} on top", name_of(g, c));
    if ability_window_card(g, p, g.land(l).cd, &what, None, None)? && remove_first(&mut g.player_mut(p).gy, c) {
        g.player_mut(p).library.push(c);
    }
    Ok(true)
}

/// {1}{W},{T}: prevents the combat damage of the biggest unblocked attacker (4+ power): it leaves the attack
fn kor_haven(g: &mut Game, l: LandId, d: PlayerId, p: PlayerId, atk: &mut Vec<PermId>, assign: &mut Assign) -> Res {
    if g.land(l).tapped || !can_pay_without(g, d, l, 1, "W") {
        return Ok(());
    }
    let unb: Vec<PermId> = atk
        .iter()
        .copied()
        .filter(|&a| !assign.iter().any(|x| x.0 == a) && g.perm(a).on_bf && g.perm(a).owner == p)
        .collect();
    let Some(a) = max_by(&unb, |m| epow(g, m) as f64) else { return Ok(()) };
    if epow(g, a) < 4 || !pay_without(g, d, l, 1, "W")? {
        return Ok(());
    }
    g.land_mut(l).tapped = true;
    if let Some(i) = atk.iter().position(|&x| x == a) {
        atk.remove(i);
    }
    crate::glog!(g, "    {} uses Kor Haven on {}", g.player(d).name, g.perm(a).name);
    Ok(())
}

/// {2}{W}{B},{T}: deathtouch and lifelink for the team before an attack with three or more
fn vault(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || g.land(l).tapped || !can_pay_without(g, p, l, 2, "WB") {
        return Ok(vec![]);
    }
    let ready = ready_attackers(g, p);
    if ready < 3 {
        return Ok(vec![]);
    }
    Ok(vec![opt(0.3 * ready as f64, "Vault of the Archangel".into(), vault_go, l.0 as i64)])
}

fn vault_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId(arg as u32);
    if !has_land(g, p, l) || g.land(l).tapped || !pay_without(g, p, l, 2, "WB")? {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    crate::glog!(g, "  {} uses Vault of the Archangel", g.player(p).name);
    if ability_window_card(g, p, g.land(l).cd, "deathtouch and lifelink", None, None)? {
        for m in g.player(p).perms.clone() {
            if g.is_creature(m) {
                let kw = &mut g.perm_mut(m).eot_kw;
                for k in ["deathtouch", "lifelink"] {
                    if !kw.contains(&k) {
                        kw.push(k);
                    }
                }
            }
        }
    }
    Ok(true)
}

/// taps for {G}; {T}: a 1/1 attacker gets +1/+2 when the mana is not needed
fn pendelhaven(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || g.land(l).tapped {
        return Ok(vec![]);
    }
    let one = g.player(p).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        g.is_creature(m) && epow(g, m) == 1 && etgh(g, m) == 1 && !x.sick && !x.tapped
    });
    let Some(m) = one else { return Ok(vec![]) };
    if total_mana(g, p, false) >= 6 {
        return Ok(vec![]);
    }
    Ok(vec![opt(0.4, "Pendelhaven".into(), pendelhaven_go, pack(l.0, m.0))])
}

fn pendelhaven_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (l, m) = unpack(arg);
    let (l, m) = (LandId(l), PermId(m));
    if !has_land(g, p, l) || g.land(l).tapped {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    if ability_window_card(g, p, g.land(l).cd, "+1/+2", None, Some(m))? && g.perm(m).on_bf && g.perm(m).owner == p {
        let x = g.perm_mut(m);
        x.eot_pt.0 += 1;
        x.eot_pt.1 += 2;
    }
    Ok(true)
}

/// green creatures of p's that entered this turn (summoning sick)
fn oran_new(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && g.perm(m).sick && g.perm(m).cd.is_some_and(|c| g.db.get(c).pips.contains('G')))
        .collect()
}

/// enters tapped; {T}: +1/+1 counters on green creatures that entered this turn (end of your main phase)
fn oran_rief(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(true) || g.land(l).tapped {
        return Ok(vec![]);
    }
    let new = oran_new(g, p);
    if new.len() < 2 {
        return Ok(vec![]);
    }
    Ok(vec![opt(0.4 * new.len() as f64, "Oran-Rief".into(), oran_go, l.0 as i64)])
}

fn oran_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId(arg as u32);
    let new = oran_new(g, p); // the creatures the option saw (nothing happened in between)
    if !has_land(g, p, l) || g.land(l).tapped {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    if ability_window_card(g, p, g.land(l).cd, "+1/+1 counters", None, None)? {
        for m in new {
            if g.perm(m).on_bf && g.perm(m).owner == p {
                g.perm_mut(m).plus += 1;
            }
        }
    }
    Ok(true)
}

/// Yavimaya Hollow: {G},{T}: regenerates a valuable creature that would be destroyed
fn hollow(g: &mut Game, l: LandId, p: PlayerId, m: PermId) -> Res<bool> {
    if g.land(l).tapped || pval(g, m) < 4.0 || !can_pay_without(g, p, l, 0, "G") {
        return Ok(false);
    }
    if !pay_without(g, p, l, 0, "G")? {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    g.perm_mut(m).tapped = true;
    crate::glog!(g, "    {} regenerates {} with Yavimaya Hollow", g.player(p).name, g.perm(m).name);
    Ok(true)
}

/// lands.try_regenerate: a land that regenerates (Yavimaya Hollow, the one land with a 'regenerate' hook) saves m
/// from destruction
pub fn try_regenerate(g: &mut Game, m: PermId) -> Res<bool> {
    let p = g.perm(m).owner;
    for l in g.player(p).lands.clone() {
        if land_name(g, l) == "Yavimaya Hollow" && hollow(g, l, p, m)? {
            return Ok(true);
        }
    }
    Ok(false)
}

// ------------------------------------------------------------------ enter-the-battlefield lands
/// enters tapped; the best creature card from the graveyard on top of the library
fn mortuary(g: &mut Game, p: PlayerId, _l: LandId) -> Res {
    let cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    if let Some(c) = first_max(&cs, |c| (g.db.get(c).bomb, card_worth(g, p, c, false))) {
        remove_first(&mut g.player_mut(p).gy, c);
        g.player_mut(p).library.push(c);
        crate::glog!(g, "    Mortuary Mire puts {} on top", name_of(g, c));
    }
    Ok(())
}

fn is_island(g: &Game, l: LandId) -> bool {
    let d = g.db.get(g.land(l).cd);
    &*d.name == "Island" || d.has_subtype("island")
}

/// enters untapped with three other Islands, then puts the best instant or sorcery from the graveyard on top
fn sanctuary(g: &mut Game, p: PlayerId, l: LandId) -> Res {
    let islands = g.player(p).lands.iter().filter(|&&x| x != l && is_island(g, x)).count();
    if islands >= 3 {
        g.land_mut(l).tapped = false;
        let cs: Vec<CardId> =
            g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).instant || g.db.get(c).sorcery).collect();
        if let Some(c) = max_by(&cs, |c| card_worth(g, p, c, false)) {
            remove_first(&mut g.player_mut(p).gy, c);
            g.player_mut(p).library.push(c);
            crate::glog!(g, "    Mystic Sanctuary puts {} on top", name_of(g, c));
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ Vivid lands: two charge counters for any colour
const VIVID: [(&str, char); 5] =
    [("Vivid Creek", 'U'), ("Vivid Grove", 'G'), ("Vivid Marsh", 'B'), ("Vivid Meadow", 'W'), ("Vivid Crag", 'R')];

fn vivid_color(g: &Game, l: LandId) -> char {
    let n = land_name(g, l);
    VIVID.iter().find(|x| x.0 == n).map_or('C', |x| x.1)
}

/// enters with two (however it entered)
fn vivid_charge(g: &Game, l: LandId) -> i64 {
    g.land(l).data.get(DataKey::Charge).map_or(2, Val::int)
}

/// {T}: its own colour, or (with a charge counter left) any colour in p's identity
fn vivid_cols(g: &Game, p: PlayerId, l: LandId) -> Colors {
    let own = Colors::from_letters(&vivid_color(g, l).to_string());
    if vivid_charge(g, l) != 0 { own.union(g.player(p).ident) } else { own }
}

/// a colour other than its own used a charge counter
fn vivid_tap(g: &mut Game, _p: PlayerId, l: LandId, _n: u32) -> Res {
    let own = vivid_color(g, l);
    if "WUBRG".chars().any(|x| x != own && g.tap_cols.has(x)) {
        let n = (vivid_charge(g, l) - 1).max(0);
        g.land_mut(l).data.set(DataKey::Charge, Val::Int(n));
    }
    Ok(())
}

// ------------------------------------------------------------------ hideaway: Mosswort Bridge
/// hideaway 4: keeps the best spell (a creature by mana value, else by worth)
fn mosswort(g: &mut Game, p: PlayerId, l: LandId) -> Res {
    let n = 4.min(g.player(p).library.len());
    let mut top: Vec<CardId> = (0..n).map(|_| g.player_mut(p).library.pop().unwrap()).collect();
    if top.is_empty() {
        return Ok(());
    }
    let c = first_max(&top, |c| {
        let d = g.db.get(c);
        (!d.land, if d.creature { d.cmc as f64 } else { card_worth(g, p, c, false) / 20.0 })
    })
    .unwrap();
    remove_first(&mut top, c);
    g.player_mut(p).library.splice(0..0, top);
    g.land_mut(l).data.set(DataKey::Hidden, Val::Card(c));
    crate::glog!(g, "    Mosswort Bridge hides a card");
    Ok(())
}

fn hidden(g: &Game, l: LandId) -> Option<CardId> {
    match g.land(l).data.get(DataKey::Hidden) {
        Some(Val::Card(c)) => Some(*c),
        _ => None,
    }
}

/// {G},{T}: casts the hidden card free with 10 power on board
fn mosswort_play(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let Some(c) = hidden(g, l) else { return Ok(vec![]) };
    if post.is_none() || g.land(l).tapped || !can_pay_without(g, p, l, 0, "G") {
        return Ok(vec![]);
    }
    let power: i32 = g.player(p).perms.iter().filter(|&&m| g.is_creature(m)).map(|&m| epow(g, m)).sum();
    if power < 10 {
        return Ok(vec![]);
    }
    let label = format!("Mosswort Bridge ({})", name_of(g, c));
    Ok(vec![opt(1.0 + g.db.get(c).cmc as f64 / 3.0, label, mosswort_go, l.0 as i64)])
}

fn mosswort_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId(arg as u32);
    let Some(c) = hidden(g, l) else { return Ok(false) };
    if !has_land(g, p, l) || g.land(l).tapped || !pay_without(g, p, l, 0, "G")? {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    g.land_mut(l).data.set(DataKey::Hidden, Val::None);
    let what = format!("play {}", name_of(g, c));
    if !ability_window_card(g, p, g.land(l).cd, &what, None, None)? {
        return Ok(true);
    }
    if g.db.get(c).land {
        play_land_card_free(g, p, c);
    } else {
        crate::engine::cast::cast_card(g, p, c, "lib", crate::state::Ctx::default())?;
    }
    Ok(true)
}

/// lands.play_land_card_free: land card c onto p's battlefield, tapped
pub fn play_land_card_free(g: &mut Game, p: PlayerId, c: CardId) {
    g.add_land(p, c, true);
    crate::glog!(g, "  {} puts {} onto the battlefield", g.player(p).name, name_of(g, c));
}

// ------------------------------------------------------------------ Emeria, the Sky Ruin
/// enters tapped; with seven Plains, each upkeep returns the best creature card from the graveyard
fn emeria(g: &mut Game, _l: LandId, p: PlayerId) -> Res {
    let plains = g
        .player(p)
        .lands
        .iter()
        .filter(|&&x| {
            let d = g.db.get(g.land(x).cd);
            &*d.name == "Plains" || d.has_subtype("plains")
        })
        .count();
    if plains < 7 {
        return Ok(());
    }
    let cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    if let Some(c) = first_max(&cs, |c| (g.db.get(c).bomb, card_worth(g, p, c, false))) {
        remove_first(&mut g.player_mut(p).gy, c);
        enter(g, p, c, Enter::default())?;
        crate::glog!(g, "    Emeria returns {}", name_of(g, c));
    }
    Ok(())
}

// ------------------------------------------------------------------ Tolaria West: transmute
/// enters tapped; transmute {1}{U}{U}: discard it to tutor a card with mana value 0 (a combo artifact or a key land)
fn tolaria(g: &mut Game, _c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || !can_pay(g, p, 1, "UU", false) {
        return Ok(vec![]);
    }
    let wish = crate::ai::plans::wish_list(g, p);
    let zero: Vec<CardId> = searchable(g, p).into_iter().filter(|&x| g.db.get(x).cmc == 0).collect();
    if zero.is_empty() {
        return Ok(vec![]);
    }
    let t = match zero.iter().copied().find(|x| wish.contains(x)) {
        Some(t) => t,
        None => first_max(&zero, |x| (KEY_LANDS.contains(&name_of(g, x)), card_worth(g, p, x, false))).unwrap(),
    };
    let in_wish = wish.contains(&t);
    if !in_wish && !(g.db.get(t).land && KEY_LANDS.contains(&name_of(g, t))) && card_worth(g, p, t, false) < 45.0 {
        return Ok(vec![]);
    }
    let label = format!("transmute Tolaria West ({})", name_of(g, t));
    Ok(vec![opt(if in_wish { 2.5 } else { 1.5 }, label, tolaria_go, t.0 as i64)])
}

fn tolaria_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let t = CardId(arg as u16);
    let Some(c) = g.db.id("Tolaria West") else { return Ok(false) };
    if !g.player(p).hand.contains(&c) || !g.player(p).library.contains(&t) || !can_pay(g, p, 1, "UU", false) {
        return Ok(false);
    }
    remove_first(&mut g.player_mut(p).hand, c);
    pay(g, p, 1, "UU", false)?;
    g.player_mut(p).gy.push(c);
    remove_first(&mut g.player_mut(p).library, t);
    g.player_mut(p).hand.push(t);
    shuffle_library(g, p);
    crate::glog!(g, "  {} transmutes Tolaria West for {}", g.player(p).name, name_of(g, t));
    Ok(true)
}

// ------------------------------------------------------------------ Dakmor Salvage: dredge 2
/// lands.dakmor_dredge: enters tapped; dredge 2 instead of drawing when out of lands in hand
pub fn dakmor_dredge(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(c) = g.player(p).gy.iter().copied().find(|&x| name_of(g, x) == "Dakmor Salvage") else {
        return Ok(false);
    };
    let pl = g.player(p);
    if pl.library.len() < 25 || pl.hand.iter().filter(|&&x| g.db.get(x).land).count() >= 1 {
        return Ok(false);
    }
    mill(g, p, 2)?;
    remove_first(&mut g.player_mut(p).gy, c);
    g.player_mut(p).hand.push(c);
    crate::glog!(g, "  {} dredges Dakmor Salvage", g.player(p).name);
    Ok(true)
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let land_opts: [(&str, LandOptionsFn); 18] = [
        ("Mutavault", manland_options),
        ("Hissing Quagmire", manland_options),
        ("Needle Spires", manland_options),
        ("Raging Ravine", manland_options),
        ("Hive of the Eye Tyrant", manland_options),
        ("Horizon Canopy", cycler_options),
        ("Silent Clearing", cycler_options),
        ("Sunbaked Canyon", cycler_options),
        ("Waterlogged Grove", cycler_options),
        ("Ghost Quarter", ld_land_options),
        ("Tectonic Edge", ld_land_options),
        ("Castle Locthwain", locthwain),
        ("Castle Embereth", embereth),
        ("War Room", war_room),
        ("Karn's Bastion", bastion),
        ("Academy Ruins", academy_ruins),
        ("Vault of the Archangel", vault),
        ("Pendelhaven", pendelhaven),
    ];
    for (name, f) in land_opts {
        r.card(db, name)?.land_options = Some(f);
    }
    r.card(db, "Oran-Rief, the Vastwood")?.land_options = Some(oran_rief);
    r.card(db, "Mosswort Bridge")?.land_options = Some(mosswort_play);
    for name in ["Horizon Canopy", "Silent Clearing", "Sunbaked Canyon", "Waterlogged Grove"] {
        r.card(db, name)?.on_tap_land = Some(pain);
    }
    r.card(db, BOSEIJU)?.hand_options = Some(boseiju);
    r.card(db, SOKENZAN)?.hand_options = Some(sokenzan);
    r.card(db, "Tolaria West")?.hand_options = Some(tolaria);
    r.card(db, "Kor Haven")?.land_defend = Some(kor_haven);
    r.card(db, "Mortuary Mire")?.land_etb = Some(mortuary);
    r.card(db, "Mystic Sanctuary")?.land_etb = Some(sanctuary);
    r.card(db, "Mosswort Bridge")?.land_etb = Some(mosswort);
    r.card(db, "Emeria, the Sky Ruin")?.land_upkeep = Some(emeria);
    for (name, _) in VIVID {
        if db.id(name).is_none() {
            continue; // Python defines all five; the export has only those in a decklist
        }
        let imp = r.card(db, name)?;
        imp.land_cols = Some(vivid_cols);
        imp.on_tap_land = Some(vivid_tap);
    }
    Ok(())
}
