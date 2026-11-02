//! Python's `cards/impl/yshtola.py`: Y'shtola, Night's Blessed (your Esper Drain deck, key `yshtola`).
//!
//! - Y'shtola: whenever you cast a noncreature spell with mana value 3 or greater (X counts), she deals 2 damage to
//!   each opponent and you gain 2 life; at the beginning of each end step, if a player lost 4 or more life this turn,
//!   you draw.
//! - The drain package: Exsanguinate and Debt to the Deathless (X chosen by the AI), Ill-Gotten Inheritance, Urborg
//!   Syphon-Mage, Marauding Blight-Priest, Polluted Bonds (Sanguine Bond is in t4.py).
//! - Cards taken from opponents and cast with mana of any type: Gonti, Lord of Luxury, Hostage Taker, Thief of Sanity.
//!   The card is held in your hand (as Opposition Agent's are), counted out of your hand size, and goes back to its
//!   owner's graveyard; Hostage Taker's card returns to the battlefield if the Taker leaves before you cast it.
//! - The rest that needs code: Curiosity (on Y'shtola it draws for each opponent her trigger hits), Jester's Cap, Dark
//!   Petition (spell mastery adds {B}{B}{B}), Plea for Guidance, Take Up the Shield.
//! - The AI (cast priorities, X and drain timing, tutor targets, protection and wipe responses, Zur's fetch values)
//!   was deleted on 2026-11-01: the deck plays the generic policy and `data/plans/yshtola.toml`
//!   (`audit/phase4/hand-off-design.md`, "Deleted, part 2"). What stays is the cards' own options (X drains,
//!   Syphon-Mage, Ill-Gotten Inheritance, Jester's Cap).
//!
//! The person's choices (practice mode) are the `human_of` branches.

use crate::engine::repl;
use crate::engine::pool::Kind;
use crate::ai::plans::wish_list;
use crate::ai::pool::generic_prio;
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::cast_card;
use crate::engine::life::{check_state, gain, lose_life, prevents_damage};
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{ability_window, ability_window_card, may_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library, tutor};
use crate::engine::values::{card_name, epow, protected_from, pval, threat, untargetable};
use crate::engine::zones::{Enter, die, discard_cards, draw, enter, max_by, min_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::human::{cards, choices, human_of};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::common::{AURA, AuraSpec, aura};
use crate::pysum::PySum;
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::Sym;
use crate::tag::Tag;

pub const YSH: &str = "Y'shtola, Night's Blessed";

// ------------------------------------------------------------------ helpers
fn human(g: &Game, p: PlayerId) -> bool {
    g.humans.get(p).is_some()
}

fn name_is(g: &Game, m: PermId, name: &str) -> bool {
    card_name(g, m) == Some(name)
}

fn hand_has(g: &Game, p: PlayerId, c: CardId) -> bool {
    g.player(p).hand.contains(&c)
}

/// remove the first copy of c from a card list (Python's `list.remove`)
fn remove_card(xs: &mut Vec<CardId>, c: CardId) -> bool {
    match xs.iter().position(|&x| x == c) {
        Some(i) => {
            xs.remove(i);
            true
        }
        None => false,
    }
}

/// `g.eot_kw.setdefault(id(m), set()).add(kw)`: a keyword until end of turn
fn add_eot_kw(g: &mut Game, m: PermId, kw: Sym) {
    let x = &mut g.perm_mut(m).eot_kw;
    if !x.contains(&kw) {
        x.push(kw);
    }
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

/// yshtola.ysh_perm: Y'shtola on p's battlefield (the commander, or a copy under p's control)
pub fn ysh_perm(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        name_is(g, m, YSH) && !x.phased && !x.neutered
    })
}

/// yshtola.lost_this_turn: the life q has lost this turn
pub fn lost_this_turn(g: &Game, q: PlayerId) -> i32 {
    match g.player(q).lost_turn {
        Some((st, n)) if st == g.turn_stamp() => n,
        _ => 0,
    }
}

// ======================================================== Y'shtola
/// yshtola.cast_mv: the mana value of spell c as it's cast: X counts (Exsanguinate, Debt to the Deathless, Secure the
/// Wastes; Crackle with Power's X three times)
pub fn cast_mv(g: &Game, c: CardId) -> i32 {
    let x = match &g.cur_cast {
        Some((cc, ctx, _)) if *cc == c => ctx.x,
        _ => 0,
    };
    let d = g.db.get(c);
    d.cmc as i32 + if d.tag(Tag::Crackle) { 3 * x } else { x }
}

/// whenever you cast a noncreature spell with mana value 3 or greater: 2 damage to each opponent, gain 2 life
fn ysh_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = g.perm(src).owner;
    let d = g.db.get(c);
    if caster != o || d.creature || d.land || cast_mv(g, c) < 3 || g.opps(o).next().is_none() {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "2 damage to each opponent; you gain 2 life", Some(4.0))? {
        return Ok(());
    }
    ysh_drain(g, o, src)
}

/// yshtola.ysh_drain: Y'shtola's trigger resolves (Curiosity on her draws for each opponent it hit)
pub fn ysh_drain(g: &mut Game, o: PlayerId, src: PermId) -> Res {
    let turns = g.player(o).turns;
    g.player_mut(o).milestone.entry("yshtola").or_insert(turns);
    g.player_mut(o).stat("ysh_triggers", 1);
    crate::glog!(g, "    Y'shtola deals 2 damage to each opponent; {} gains 2 life", pname(g, o));
    let mut hit = 0;
    for q in g.opps(o).collect::<Vec<_>>() {
        let before = g.player(q).life;
        lose_life(g, q, 2, Some(o), "triggers", Some(true))?;
        if g.player(q).life < before {
            hit += 1;
        }
    }
    gain(g, o, 2)?;
    if g.perm(src).on_bf && g.perm(src).owner == o && hit > 0 {
        curiosity_draws(g, src, hit)?;
    }
    check_state(g)
}

/// at the beginning of each end step, if a player lost 4 or more life this turn, you draw a card
fn ysh_end(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    if !g.players.iter().any(|q| lost_this_turn(g, q.id) >= 4) {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if !trigger_window(g, o, Some(src), "draw a card (a player lost 4 or more life this turn)", Some(3.0))? {
        return Ok(());
    }
    g.player_mut(o).stat("ysh_draws", 1);
    draw(g, o, 1, false)
}

// ======================================================== Curiosity
/// on Y'shtola (her trigger hits every opponent), else your best evasive creature
fn curiosity_host(g: &mut Game, p: PlayerId, a: PermId) -> Res<Option<PermId>> {
    if human_of(g, p).is_some() {
        return Ok(None); // the person picks (common.human_aura_host)
    }
    if let Some(y) = ysh_perm(g, p)
        && !crate::impls::zur::untargetable_by_you(g, y)
    {
        return Ok(Some(y));
    }
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && !g.perm(m).phased && m != a && !g.perm(m).noatk)
        .collect();
    Ok(max_by(&cands, |m| (if g.perm(m).fly { 3.0 } else { 0.0 }) + epow(g, m) as f64 + pval(g, m) * 0.2))
}

static CURIOSITY: AuraSpec = AuraSpec { target: "either", host_pick: Some(curiosity_host), ..AURA };

/// yshtola.may_draw: draws are skipped with three cards or fewer left in the library
fn may_draw(g: &Game, p: PlayerId) -> bool {
    g.player(p).library.len() > 3 || (crate::human::is_human(g, p) && !g.player(p).library.is_empty()) // a person decides for themselves
}

/// whenever the enchanted creature deals combat damage to an opponent, you draw
fn curiosity(g: &mut Game, src: Src, _p: PlayerId, a: PermId, d: PlayerId, dmg: i32) -> Res {
    let o = g.perm(src).owner;
    if Some(a) != g.perm(src).attached || !g.opps(o).any(|q| q == d) || dmg <= 0 || !may_draw(g, o) {
        return Ok(());
    }
    if !may_window(g, o, Some(src), "draw a card", None)? {
        return Ok(());
    }
    draw(g, o, 1, false)
}

/// yshtola.curiosity_draws: creature (Y'shtola) dealt damage to n opponents outside combat: each Curiosity on it draws n
pub fn curiosity_draws(g: &mut Game, creature: PermId, n: usize) -> Res {
    for a in g.auras.clone() {
        let x = g.perm(a);
        if x.attached != Some(creature) || !name_is(g, a, "Curiosity") || x.phased || !x.on_bf {
            continue;
        }
        let o = x.owner;
        for _ in 0..n {
            if !may_draw(g, o) {
                break;
            }
            if may_window(g, o, Some(a), "draw a card", None)? {
                draw(g, o, 1, false)?;
            }
        }
    }
    Ok(())
}

// ======================================================== the drain package
/// yshtola.drain_extra: life each opponent also loses for each time you gain life (Marauding Blight-Priest)
pub fn drain_extra(g: &Game, p: PlayerId) -> i32 {
    g.player(p).perms.iter().filter(|&&m| name_is(g, m, "Marauding Blight-Priest") && !g.perm(m).phased).count() as i32
}

/// yshtola.bond_on: Sanguine Bond on p's battlefield
pub fn bond_on(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| name_is(g, m, "Sanguine Bond") && !g.perm(m).phased)
}

/// Marauding Blight-Priest: whenever you gain life, each opponent loses 1 life
fn priest(g: &mut Game, src: Src, p: PlayerId, _n: i32) -> Res {
    let o = g.perm(src).owner;
    if p != o || g.opps(o).next().is_none() {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "each opponent loses 1 life", None)? {
        return Ok(());
    }
    for q in g.opps(o).collect::<Vec<_>>() {
        lose_life(g, q, 1, Some(o), "drain", None)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Exsanguinate, Debt to the Deathless
/// yshtola.X_DRAIN: each opponent loses (this) times X
fn x_drain_k(g: &Game, c: CardId) -> i32 {
    match &*g.db.get(c).name {
        "Exsanguinate" => 1,
        "Debt to the Deathless" => 2,
        n => panic!("{n} is not an X drain"),
    }
}

/// yshtola.x_drain: each opponent loses k times X; you gain the total
pub fn x_drain(g: &mut Game, p: PlayerId, c: CardId, x: i32) -> Res {
    let mut lost = 0;
    let k = x_drain_k(g, c);
    for q in g.opps(p).collect::<Vec<_>>() {
        let b = g.player(q).life;
        lose_life(g, q, k * x, Some(p), "drain", None)?;
        lost += 0.max(b - g.player(q).life);
    }
    crate::glog!(
        g,
        "    {} (X = {x}): each opponent loses {} life; {} gains {lost}",
        g.db.get(c).name,
        k * x,
        pname(g, p)
    );
    if lost != 0 {
        gain(g, p, lost)?;
    }
    check_state(g)
}

/// yshtola._x_resolve
fn x_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    x_drain(g, p, c, ctx.x)?;
    Ok("gy")
}

/// yshtola.drain_kills: opponents dead after each loses per_opp and you gain `gained` (Sanguine Bond sends it to one
/// more)
pub fn drain_kills(g: &Game, p: PlayerId, per_opp: i32, gained: i32) -> usize {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let mut dead: Vec<PlayerId> =
        opps.iter().copied().filter(|&q| g.player(q).life <= per_opp && !g.player(q).life_locked).collect();
    if bond_on(g, p) {
        let left: Vec<PlayerId> =
            opps.iter().copied().filter(|q| !dead.contains(q) && !g.player(*q).life_locked).collect();
        if left.iter().any(|&q| g.player(q).life <= per_opp + gained) {
            dead.push(min_by(&left, |q| g.player(q).life as f64).unwrap());
        }
    }
    dead.len()
}

/// yshtola.x_drain_option: the best X for a drain spell in hand, as (utility, label), or None
pub fn x_drain_option(g: &Game, p: PlayerId, c: CardId) -> Option<(f64, String)> {
    let (gn, pips) = cost_of(g, p, c);
    if !can_pay(g, p, gn, &pips, false) {
        return None;
    }
    let x = total_mana(g, p, false) as i32 - gn as i32 - pips.chars().count() as i32;
    if x < 1 {
        return None;
    }
    let k = x_drain_k(g, c);
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let ysh = ysh_perm(g, p).is_some() && g.db.get(c).cmc as i32 + x >= 3;
    let pri = drain_extra(g, p);
    // its gain, and Y'shtola's, each set off the Priest
    let per = k * x + if ysh { 2 } else { 0 } + pri * if ysh { 2 } else { 1 };
    let total: i32 = opps.iter().filter(|&&q| !g.player(q).life_locked).map(|&q| g.player(q).life.min(per)).sum();
    let kills = drain_kills(g, p, per, total);
    let late = g.player(p).turns >= 8 || total_mana(g, p, false) >= 9;
    if kills == 0 && (x < 4 || (!late && k * x * (opps.len() as i32) < 15)) {
        return None;
    }
    let u = 1.5 + 0.12 * total as f64 + 7.0 * kills as f64 + if bond_on(g, p) { 1.0 } else { 0.0 };
    Some((u, format!("{} (X = {x})", g.db.get(c).name)))
}

/// x_drain_option's play: cast the drain spell (arg) for the largest X the mana allows
pub fn x_drain_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !hand_has(g, p, c) {
        return Ok(false);
    }
    let (g2, p2) = cost_of(g, p, c);
    let xx = total_mana(g, p, false) as i32 - g2 as i32 - p2.chars().count() as i32;
    if xx < 1 || !can_pay(g, p, g2 + xx as u32, &p2, false) {
        return Ok(false);
    }
    pay(g, p, g2 + xx as u32, &p2, false)?;
    cast_card(g, p, c, "hand", Ctx { x: xx, ..Ctx::default() })?;
    Ok(true)
}

/// yshtola._x_option: the drain X spell as a main-phase play (your turn, a deck whose plan `plays` it)
fn x_option(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none()
        || g.active != Some(p)
        || !hand_has(g, p, c)
        || !crate::ai::plan_data::plays(g, p, &g.db.get(c).name) // --- plan-files-3: a plan's `plays`
    {
        return Ok(vec![]);
    }
    Ok(x_drain_option(g, p, c)
        .map(|(u, label)| Opt { utility: u, label, act: Some(Action::Plan { f: x_drain_go, arg: c.0 as i64 }) })
        .into_iter()
        .collect())
}

// ------------------------------------------------------------------ Polluted Bonds
/// a land an opponent controls enters: that player loses 2 life and you gain 2 (Sanguine Bond and Marauding
/// Blight-Priest turn the gain into more drain)
fn polluted_bonds(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = g.perm(src).owner;
    if p == o || !g.opps(o).any(|q| q == p) || !g.player(p).alive {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), &format!("{} loses 2 life; you gain 2", pname(g, p)), None)? {
        return Ok(());
    }
    lose_life(g, p, 2, Some(o), "drain", None)?;
    gain(g, o, 2)?;
    check_state(g)
}

// ------------------------------------------------------------------ Ill-Gotten Inheritance
/// your upkeep: 1 damage to each opponent, you gain 1 life
fn inheritance(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = g.perm(src).owner;
    if p != o || g.opps(o).next().is_none() {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "1 damage to each opponent; you gain 1 life", None)? {
        return Ok(());
    }
    for q in g.opps(o).collect::<Vec<_>>() {
        lose_life(g, q, 1, Some(o), "triggers", Some(true))?;
    }
    gain(g, o, 1)?;
    check_state(g)
}

/// yshtola._inheritance_sac: {5}{B}, sacrifice it: 4 damage to target opponent and you gain 4 life (for a kill, or at
/// an opponent's end step with the mana spare)
pub fn inheritance_sac_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != g.perm(src).owner || human(g, p) || !can_pay(g, p, 5, "B", false) || g.opps(p).next().is_none() {
        return Ok(vec![]);
    }
    let (t, kill) = inheritance_target(g, p);
    if !kill && (post.is_some() || g.active == Some(p) || g.player(p).turns < 9) {
        return Ok(vec![]);
    }
    let u = if kill { 9.0 } else { 1.5 + if bond_on(g, p) { 1.0 } else { 0.0 } };
    Ok(vec![Opt {
        utility: u,
        label: format!("Ill-Gotten Inheritance: 4 damage to {}", pname(g, t)),
        act: Some(Action::Ability { src, f: inheritance_go, arg: t.0 as i64 }),
    }])
}

fn inheritance_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PlayerId(arg as u8);
    if !g.perm(src).on_bf || g.perm(src).owner != p || !can_pay(g, p, 5, "B", false) || !g.player(t).alive {
        return Ok(false);
    }
    pay(g, p, 5, "B", false)?;
    inheritance_sac(g, p, src, t)?;
    Ok(true)
}

/// yshtola.inheritance_target: the opponent it kills (the most threatening), else the most threatening
pub fn inheritance_target(g: &Game, p: PlayerId) -> (PlayerId, bool) {
    let per = 4 + drain_extra(g, p);
    let mut opps: Vec<PlayerId> =
        g.opps(p).filter(|&q| !g.player(q).life_locked && !prevents_damage(g, q, Some(p))).collect();
    if opps.is_empty() {
        opps = g.opps(p).collect();
    }
    let dead: Vec<PlayerId> = opps.iter().copied().filter(|&q| g.player(q).life <= per).collect();
    if !dead.is_empty() {
        return (max_by(&dead, |q| threat(g, p, q)).unwrap(), true);
    }
    (max_by(&opps, |q| threat(g, p, q)).unwrap(), false)
}

/// yshtola.inheritance_sac
pub fn inheritance_sac(g: &mut Game, p: PlayerId, src: PermId, t: PlayerId) -> Res {
    crate::glog!(g, "  {} sacrifices Ill-Gotten Inheritance: 4 damage to {}", pname(g, p), pname(g, t));
    die(g, src, "sac")?;
    let cd = g.perm(src).cd.unwrap();
    if !ability_window_card(g, p, cd, &format!("4 damage to {}; gain 4 life", pname(g, t)), Some(4.0), None)? {
        return Ok(());
    }
    if g.player(t).alive {
        lose_life(g, t, 4, Some(p), "triggers", Some(true))?;
    }
    gain(g, p, 4)?;
    check_state(g)
}

// ------------------------------------------------------------------ Urborg Syphon-Mage
/// yshtola._syphon: {2}{B}, {T}, discard a card: each other player loses 2 life; you gain the total
pub fn syphon_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if p != x.owner || human(g, p) || x.tapped || x.sick || !can_pay(g, p, 2, "B", false) {
        return Ok(vec![]);
    }
    let Some(worst) = syphon_discard(g, p) else { return Ok(vec![]) };
    let per = 2 + drain_extra(g, p);
    let total: i32 = g.opps(p).filter(|&q| !g.player(q).life_locked).map(|q| g.player(q).life.min(per)).sum();
    let kills = drain_kills(g, p, per, total);
    let pl = g.player(p);
    let spare =
        g.db.get(worst).land && pl.hand.iter().filter(|&&c| g.db.get(c).land).count() >= 2 && pl.lands.len() >= 5;
    if kills == 0 && !spare && post.is_some() {
        return Ok(vec![]); // otherwise at an opponent's end step only
    }
    if kills == 0 && post.is_none() && g.active == Some(p) {
        return Ok(vec![]);
    }
    let cost = card_worth(g, p, worst, false) / 20.0;
    let u = 1.0 + 0.15 * total as f64 + 8.0 * kills as f64 + if bond_on(g, p) { 1.0 } else { 0.0 } - cost;
    if u <= 0.5 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: u,
        label: format!("Urborg Syphon-Mage (discard {})", g.db.get(worst).name),
        act: Some(Action::Ability { src, f: syphon_go, arg: worst.0 as i64 }),
    }])
}

fn syphon_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let worst = CardId(arg as u16);
    let x = g.perm(src);
    if !x.on_bf || x.owner != p || x.tapped || !hand_has(g, p, worst) || !can_pay(g, p, 2, "B", false) {
        return Ok(false);
    }
    pay(g, p, 2, "B", false)?;
    g.perm_mut(src).tapped = true;
    discard_cards(g, p, &[worst])?;
    syphon(g, p, src)?;
    Ok(true)
}

/// yshtola.syphon_discard: the least valuable card in hand (not one taken from an opponent)
pub fn syphon_discard(g: &Game, p: PlayerId) -> Option<CardId> {
    let pl = g.player(p);
    let cs: Vec<CardId> = pl.hand.iter().copied().filter(|c| !pl.stolen.contains_key(c)).collect();
    min_by(&cs, |c| card_worth(g, p, c, false))
}

/// yshtola.syphon: the ability resolves
pub fn syphon(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    crate::glog!(g, "  {} activates Urborg Syphon-Mage", pname(g, p));
    if !ability_window(g, p, Some(src), "each other player loses 2 life", Some(4.0), None)? {
        return Ok(());
    }
    let mut lost = 0;
    let others: Vec<PlayerId> = g.players.iter().filter(|q| q.id != p && q.alive).map(|q| q.id).collect();
    for q in others {
        let b = g.player(q).life;
        lose_life(g, q, 2, Some(p), "drain", None)?;
        lost += 0.max(b - g.player(q).life);
    }
    if lost != 0 {
        gain(g, p, lost)?;
    }
    check_state(g)
}

// ======================================================== cards taken from opponents
/// yshtola.take_card: p may cast c (owner's card, in exile) with mana of any type: held in p's hand, as Opposition
/// Agent's cards
pub fn take_card(g: &mut Game, p: PlayerId, c: CardId, owner: PlayerId, why: &str) {
    let pl = g.player_mut(p);
    pl.hand.push(c);
    if !pl.agent_ids.contains(&c) {
        pl.agent_ids.push(c);
    }
    pl.stolen.insert(c, owner);
    pl.stat("cards_stolen", 1);
    let (public, line) = (
        format!("    {} exiles a card face down ({why})", pname(g, p)),
        format!("    {} exiles {} with {why}", pname(g, p), g.db.get(c).name),
    );
    g.log_secret(p, &public, &line);
}

/// yshtola.stolen_worth: how much p wants to cast opponent's card c (a land can't be cast)
pub fn stolen_worth(g: &Game, p: PlayerId, c: CardId) -> f64 {
    let d = g.db.get(c);
    if d.land {
        return -1.0;
    }
    let mut v = generic_prio(g, p, c) as f64;
    if v == 0.0 {
        v = if d.has_dsl() { crate::dsl::card_value(g, p, c) * 10.0 } else { 0.0 };
    }
    let t = &d.tags;
    if v == 0.0 {
        v = if t.has(Tag::Ctr) {
            55.0
        } else if t.has(Tag::Rem) || t.has(Tag::Wipe) {
            50.0
        } else {
            30.0
        };
    }
    v + 4.0 * d.bomb as f64 + if d.cmc as usize <= g.player(p).lands.len() + 1 { 6.0 } else { 0.0 }
}

/// yshtola.steal_pick: p looks at cards `top` from owner's library and exiles one: the best to cast
pub fn steal_pick(g: &Game, p: PlayerId, top: &[CardId], why: &str) -> Res<CardId> {
    if human_of(g, p).is_some() {
        let labels: Vec<String> = top
            .iter()
            .map(|&c| {
                let d = g.db.get(c);
                format!("{}{}", d.name, if d.land { " (a land: it can't be cast)" } else { "" })
            })
            .collect();
        let prompt = format!("{why}: exile which card face down? (you may cast it, with mana of any type)");
        return Ok(top[choices::choose_one(g, p, "choose", &prompt, &labels)?]);
    }
    Ok(max_by(top, |c| stolen_worth(g, p, c)).unwrap())
}

/// yshtola.exile_for: a land is just exiled; anything else p may cast
pub fn exile_for(g: &mut Game, p: PlayerId, pick: CardId, owner: PlayerId, why: &str) {
    if g.db.get(pick).land {
        g.player_mut(owner).exile.push(pick);
        let (public, line) = (
            format!("    {} exiles a card face down ({why})", pname(g, p)),
            format!("    {} exiles {} with {why}", pname(g, p), g.db.get(pick).name),
        );
        g.log_secret(p, &public, &line);
    } else {
        take_card(g, p, pick, owner, why);
    }
}

/// look at the top four of target opponent's library, exile one face down (castable), the rest on the bottom
fn gonti(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if !g.opps(o).any(|q| !g.player(q).library.is_empty()) {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "look at the top four of an opponent's library, exile one", Some(4.0))? {
        return Ok(());
    }
    let opps: Vec<PlayerId> = g.opps(o).filter(|&q| !g.player(q).library.is_empty()).collect();
    if opps.is_empty() {
        return Ok(());
    }
    let q = if human_of(g, o).is_some() && opps.len() > 1 {
        cards::pick_player(g, o, "target", "Gonti: look at which opponent's library?", &opps)?
    } else {
        max_by(&opps, |q| threat(g, o, q)).unwrap()
    };
    let n = 4.min(g.player(q).library.len());
    let top: Vec<CardId> = (0..n).map(|_| g.player_mut(q).library.pop().unwrap()).collect();
    let pick = steal_pick(g, o, &top, "Gonti")?;
    // Python keeps `c is not pick`: other copies of the picked card are dropped too (cards are shared objects)
    let mut rest: Vec<CardId> = top.iter().copied().filter(|&c| c != pick).collect();
    g.rng.shuffle(&mut rest);
    for c in rest {
        g.player_mut(q).library.insert(0, c);
    }
    exile_for(g, o, pick, q, "Gonti");
    Ok(())
}

/// combat damage to a player: the top three of their library, one exiled face down (castable), the rest to the
/// graveyard
fn thief(g: &mut Game, src: Src, p: PlayerId, a: PermId, d: PlayerId, dmg: i32) -> Res {
    if a != src || p != g.perm(src).owner || g.player(d).library.is_empty() || dmg <= 0 {
        return Ok(());
    }
    let name = format!("look at the top three of {}'s library, exile one", pname(g, d));
    if !trigger_window(g, p, Some(src), &name, Some(4.0))? {
        return Ok(());
    }
    if g.player(d).library.is_empty() {
        return Ok(());
    }
    let n = 3.min(g.player(d).library.len());
    let top: Vec<CardId> = (0..n).map(|_| g.player_mut(d).library.pop().unwrap()).collect();
    let pick = steal_pick(g, p, &top, "Thief of Sanity")?;
    for &c in &top {
        if c != pick {
            g.player_mut(d).gy.push(c);
        }
    }
    exile_for(g, p, pick, d, "Thief of Sanity");
    Ok(())
}

/// exile another target creature or artifact until Hostage Taker leaves; you may cast it with mana of any type
fn taker(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    let hum = human_of(g, o).is_some();
    let t = if hum { None } else { taker_target(g, o, Some(src)) };
    if t.is_none() && (!hum || taker_cands(g, o, Some(src)).is_empty()) {
        return Ok(());
    }
    let name = match t {
        Some(t) => format!("exile {}", g.perm(t).name),
        None => "exile a creature or artifact".to_string(),
    };
    if !trigger_window(g, o, Some(src), &name, Some(5.0))? {
        return Ok(());
    }
    let t = if hum {
        // the person's target (asked once the trigger is committed)
        let cands = taker_cands(g, o, Some(src));
        if cands.is_empty() {
            return Ok(());
        }
        cards::pick_perm(g, o, "target", "Hostage Taker: exile which other creature or artifact?", &cands, None)?
            .unwrap()
    } else {
        t.unwrap()
    };
    if !g.perm(t).on_bf || g.perm(t).phased {
        return Ok(());
    }
    let x = g.perm(t);
    let (owner, cd, tok) = (x.orig, x.phys.or(x.cd), x.token);
    let prev = g.rem_src.replace(src);
    let r = apply_removal(g, Some(o), t, "exile", g.perm(src).cd);
    g.rem_src = prev;
    r?;
    let Some(cd) = cd else { return Ok(()) };
    if tok || g.perm(t).on_bf || !g.player(owner).exile.contains(&cd) {
        return Ok(()); // a token, a commander, or saved
    }
    g.player_mut(owner).exile.take(cd);
    if !(g.perm(src).on_bf && g.perm(src).owner == o) {
        enter(g, owner, cd, Enter::default())?; // the Taker already left: the card returns at once
        return Ok(());
    }
    g.perm_mut(src).data.set(DataKey::Hostage, Val::List(vec![Val::Card(cd), Val::Player(owner)]));
    take_card(g, o, cd, owner, "Hostage Taker");
    Ok(())
}

/// yshtola.taker_cands: another creature or artifact Hostage Taker can target (blue and black: protection from
/// either stops it)
pub fn taker_cands(g: &Game, o: PlayerId, src: Option<PermId>) -> Vec<PermId> {
    let (u, b) = (Colors::from_letters("U"), Colors::from_letters("B"));
    let mut out = vec![];
    for q in g.players.iter().filter(|q| q.alive) {
        for &x in &q.perms {
            let px = g.perm(x);
            if Some(x) == src || px.phased {
                continue;
            }
            let art = px.cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT));
            if !(g.is_creature(x) || art) {
                continue;
            }
            if (px.owner != o && untargetable(g, x)) || protected_from(g, x, u) || protected_from(g, x, b) {
                continue;
            }
            out.push(x);
        }
    }
    out
}

/// yshtola.taker_target: the AI's target: the best opposing creature or artifact
pub fn taker_target(g: &Game, o: PlayerId, src: Option<PermId>) -> Option<PermId> {
    let opp: Vec<PermId> = taker_cands(g, o, src).into_iter().filter(|&x| g.perm(x).owner != o).collect();
    let best = max_by(&opp, |x| {
        let px = g.perm(x);
        pval(g, x) + if !px.token && px.cd.is_some_and(|c| g.db.get(c).creature) { 1.5 } else { 0.0 }
    })?;
    (pval(g, best) >= 1.5).then_some(best)
}

/// the exiled card returns to the battlefield under its owner's control, if it hasn't been cast
fn taker_leaves(g: &mut Game, _src: Src, m: PermId) -> Res {
    let (cd, owner) = match g.perm(m).data.get(DataKey::Hostage) {
        Some(Val::List(v)) => match (&v[0], &v[1]) {
            (Val::Card(c), Val::Player(q)) => (*c, *q),
            _ => return Ok(()),
        },
        _ => return Ok(()),
    };
    g.perm_mut(m).data.set(DataKey::Hostage, Val::None);
    let o = g.perm(m).owner;
    let pl = g.player(o);
    if pl.hand.contains(&cd) && pl.stolen.get(&cd) == Some(&owner) && g.player(owner).alive {
        let pl = g.player_mut(o);
        remove_card(&mut pl.hand, cd);
        pl.stolen.shift_remove(&cd);
        pl.agent_ids.retain(|&x| x != cd);
        crate::glog!(
            g,
            "    {} returns to the battlefield under {}'s control (Hostage Taker left)",
            g.db.get(cd).name,
            pname(g, owner)
        );
        enter(g, owner, cd, Enter::default())?;
    }
    Ok(())
}

// ======================================================== Jester's Cap
/// yshtola._cap: {2}, {T}, sacrifice: search target player's library for three cards and exile them (main 2, or an
/// opponent's end step: the opponent whose best three cards matter most)
pub fn cap_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != g.perm(src).owner || human(g, p) || g.perm(src).tapped || !can_pay(g, p, 2, "", false) {
        return Ok(vec![]);
    }
    if post == Some(false) || (post.is_none() && g.active == Some(p)) {
        return Ok(vec![]);
    }
    let (q, _, v) = cap_plan(g, p);
    let Some(q) = q else { return Ok(vec![]) };
    if v < 6.0 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.0 + v / 10.0,
        label: format!("Jester's Cap on {}", pname(g, q)),
        act: Some(Action::Ability { src, f: cap_go, arg: q.0 as i64 }),
    }])
}

fn cap_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let q = PlayerId(arg as u8);
    let x = g.perm(src);
    if !x.on_bf || x.owner != p || x.tapped || !can_pay(g, p, 2, "", false) || !g.player(q).alive {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    cap_use(g, p, src, q)?;
    Ok(true)
}

/// yshtola.cap_picks: the three cards of q's library p exiles: combo pieces, wished cards and bombs first
pub fn cap_picks(g: &Game, p: PlayerId, q: PlayerId) -> (Vec<CardId>, f64) {
    let _ = p;
    let wish = wish_list(g, q); // empty for your decks (Python: `if q.key in E.SEATS`)
    let worth = |c: CardId| -> f64 {
        let d = g.db.get(c);
        (if crate::cardcode::is_combo_piece(g, c) { 8.0 } else { 0.0 })
            + (if wish.contains(&c) { 6.0 } else { 0.0 })
            + 2.0 * d.bomb as f64
            + if !d.land { card_worth(g, q, c, false) / 10.0 } else { 0.0 }
    };
    let mut lib: Vec<(f64, CardId)> = g.player(q).library.iter().map(|&c| (-worth(c), c)).collect();
    lib.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal)); // stable, as Python's sorted
    let mut out: Vec<CardId> = vec![]; // one of each name
    for (_, c) in lib {
        if out.contains(&c) {
            continue;
        }
        out.push(c);
        if out.len() == 3 {
            break;
        }
    }
    let v = out.iter().map(|&c| worth(c)).psum();
    (out, v)
}

/// yshtola.cap_plan: (the opponent, the cards, their worth) for the most damaging Cap
pub fn cap_plan(g: &Game, p: PlayerId) -> (Option<PlayerId>, Vec<CardId>, f64) {
    let mut best = (None, vec![], 0.0);
    for q in g.opps(p) {
        if g.player(q).library.is_empty() {
            continue;
        }
        let (picks, mut v) = cap_picks(g, p, q);
        v *= 1.0 + 0.05 * 0f64.max(threat(g, p, q));
        if v > best.2 {
            best = (Some(q), picks, v);
        }
    }
    best
}

/// yshtola.cap_use: tap and sacrifice it: three cards of q's library exiled, then q shuffles
pub fn cap_use(g: &mut Game, p: PlayerId, src: PermId, q: PlayerId) -> Res {
    crate::glog!(g, "  {} activates Jester's Cap on {}", pname(g, p), pname(g, q));
    g.perm_mut(src).tapped = true;
    die(g, src, "sac")?;
    let cd = g.perm(src).cd.unwrap();
    let name = format!("search {}'s library for three cards and exile them", pname(g, q));
    if !ability_window_card(g, p, cd, &name, Some(5.0), None)? {
        return Ok(());
    }
    let picks = if human_of(g, p).is_some() {
        let mut picks: Vec<CardId> = vec![];
        for i in 0..3 {
            let mut pool: Vec<CardId> = vec![];
            for &c in &g.player(q).library {
                if !picks.contains(&c) && !pool.iter().any(|&x| g.db.get(x).name == g.db.get(c).name) {
                    pool.push(c);
                }
            }
            pool.sort_by(|&a, &b| g.db.get(a).name.cmp(&g.db.get(b).name));
            if pool.is_empty() {
                break;
            }
            let labels: Vec<String> = pool.iter().map(|&c| g.db.get(c).name.to_string()).collect();
            let prompt = format!("Jester's Cap: exile which card from {}'s library? ({} of 3)", pname(g, q), i + 1);
            let Some(k) = choices::choose(g, p, "search", &prompt, &labels, Some("stop"), None)? else { break };
            picks.push(pool[k]);
        }
        picks
    } else {
        cap_picks(g, p, q).0
    };
    for &c in &picks {
        let pl = g.player_mut(q);
        if remove_card(&mut pl.library, c) {
            pl.exile.push(c);
        }
    }
    shuffle_library(g, q);
    let names: Vec<&str> = picks.iter().map(|&c| &*g.db.get(c).name).collect();
    crate::glog!(
        g,
        "    Jester's Cap exiles {} from {}'s library",
        if names.is_empty() { "nothing".to_string() } else { names.join(", ") },
        pname(g, q)
    );
    Ok(())
}

// ======================================================== tutors
/// search for any card; with two or more instants and sorceries in your graveyard, add {B}{B}{B}
fn petition(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    tutor(g, p, "any", "Dark Petition")?;
    let n = g.player(p).gy.iter().filter(|&&x| g.db.get(x).instant || g.db.get(x).sorcery).count();
    if n >= 2 {
        if human_of(g, p).is_some() {
            g.player_mut(p).pool.add('B', 3);
        } else {
            g.player_mut(p).mana.add(Kind::B, 3);
        }
        crate::glog!(g, "    spell mastery: Dark Petition adds {{B}}{{B}}{{B}}");
    }
    Ok("gy")
}

/// search for up to two enchantment cards (the AI: its two best for the board)
fn plea(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    if human_of(g, p).is_some() {
        let pred = |g: &Game, c: CardId| crate::ai::decks::tutor_ok(g, "ench", c);
        let got = choices::search(g, p, &pred, 2, "Plea for Guidance: search for up to two enchantment cards", true)?;
        for x in got {
            let pl = g.player_mut(p);
            pl.hand.push(x);
            pl.stat("tutored", 1);
            pl.seen.insert(x);
        }
        return Ok("gy");
    }
    tutor(g, p, "ench", "Plea for Guidance")?;
    tutor(g, p, "ench", "Plea for Guidance")?;
    Ok("gy")
}

// ======================================================== Take Up the Shield
fn shield_resolve(g: &mut Game, _p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = ctx.target
        && g.perm(t).on_bf
    {
        shield(g, t);
    }
    Ok("gy")
}

/// yshtola.shield: a +1/+1 counter, lifelink and indestructible until end of turn
pub fn shield(g: &mut Game, m: PermId) {
    repl::put_plus(g, m, 1);
    add_eot_kw(g, m, "indestructible");
    add_eot_kw(g, m, "lifelink");
    crate::glog!(g, "    Take Up the Shield: {} gets a +1/+1 counter, lifelink and indestructible", g.perm(m).name);
}

// ======================================================== Champion's Helm
/// Champion's Helm's options slot: none. The Y'shtola deck's hand AI equipped it on her; it was deleted on 2026-11-01
/// (`audit/phase4/hand-off-design.md`, "Deleted, part 2") and a plan's `[equip]` extra decides. The slot stays
/// registered: opponents rate a card with an options hook as an engine (`pool::card_threat_value`), so removing it
/// would change how they value the Helm.
fn helm(_g: &mut Game, _src: Src, _p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    Ok(vec![])
}

// ======================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let y = r.card(db, YSH)?;
    y.cast = Some(ysh_cast);
    y.end_step = Some(ysh_end);
    aura(r, db, "Curiosity", &CURIOSITY)?;
    r.card(db, "Curiosity")?.combat_damage = Some(curiosity);
    r.card(db, "Marauding Blight-Priest")?.gain_life = Some(priest);
    for n in ["Exsanguinate", "Debt to the Deathless"] {
        let c = r.card(db, n)?;
        c.resolve = Some(x_resolve);
        c.hand_options = Some(x_option);
    }
    r.card(db, "Polluted Bonds")?.landfall = Some(polluted_bonds);
    let c = r.card(db, "Ill-Gotten Inheritance")?;
    c.upkeep = Some(inheritance);
    c.options = Some(inheritance_sac_options);
    r.card(db, "Urborg Syphon-Mage")?.options = Some(syphon_options);
    r.card(db, "Gonti, Lord of Luxury")?.etb = Some(gonti);
    r.card(db, "Thief of Sanity")?.combat_damage = Some(thief);
    let c = r.card(db, "Hostage Taker")?;
    c.etb = Some(taker);
    c.leaves = Some(taker_leaves);
    *c = c.at_once(crate::hooks::Event::Leaves); // no trigger window: the card returns as the Taker leaves
    r.card(db, "Jester's Cap")?.options = Some(cap_options);
    r.card(db, "Dark Petition")?.resolve = Some(petition);
    r.card(db, "Plea for Guidance")?.resolve = Some(plea);
    r.card(db, "Take Up the Shield")?.resolve = Some(shield_resolve);
    r.card(db, "Champion's Helm")?.options = Some(helm);
    Ok(())
}

