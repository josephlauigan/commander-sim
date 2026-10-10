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
//!   Petition (spell mastery adds {B}{B}{B}), Plea for Guidance, Take Up the Shield, Champion's Helm for the AI.
//! - Zur the Enchanter in the 99: its attack trigger (zur.py) fetches with this deck's values (`fetch_value`).
//! - The AI: cast priorities (a spell that triggers Y'shtola is worth more), X and drain timing, tutor targets,
//!   protection and wipe responses (Take Up the Shield here; Clever Concealment, Rootborn Defenses and Restoration
//!   Angel in zur.py).
//!
//! The person's choices (practice mode) are `HUMAN(phase 9)`: the AI's choice is used.
//!
//! MERGE(zur): the zur.py functions this module calls (untargetable_by_you, the lock Auras' LOCKS / lock_host /
//! lock_worth, zur_protect, zur_wipe_response) are ported in `zur_shim` at the end of this file, because zur.py is
//! ported in another worktree. Once `impls/zur.rs` is merged, point `use zur_shim as zur;` at `crate::impls::zur`
//! (adapting `is_lock` / `lock_kind` to its LOCKS table) and delete `zur_shim`.

use crate::ai::decks::{helm_equip, helm_target, necro_prio, pay_card, sphinx_prio};
use crate::ai::plans::{citadel_prio, one_ring_prio, wish_list};
use crate::ai::pool::generic_prio;
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::cast_card;
use crate::engine::life::{check_state, gain, lose_life, prevents_damage};
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::{ability_window, ability_window_card, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library, tutor};
use crate::engine::values::{card_name, epow, etgh, protected_from, pval, threat, untargetable};
use crate::engine::zones::{Enter, die, discard_cards, draw, enter, max_by, min_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::common::{AURA, AuraSpec, aura, aura_spec, auras_on};
use crate::pysum::PySum;
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::Sym;
use crate::tag::Tag;
use zur_shim as zur;

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
    // HUMAN(phase 9): a person picks the creature (common.human_aura_host)
    if let Some(y) = ysh_perm(g, p)
        && !zur::untargetable_by_you(g, y)
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
    g.player(p).library.len() > 3
}

/// whenever the enchanted creature deals combat damage to an opponent, you draw
fn curiosity(g: &mut Game, src: Src, _p: PlayerId, a: PermId, d: PlayerId, dmg: i32) -> Res {
    let o = g.perm(src).owner;
    if Some(a) != g.perm(src).attached || !g.opps(o).any(|q| q == d) || dmg <= 0 || !may_draw(g, o) {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "draw a card", None)? {
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
            if trigger_window(g, o, Some(a), "draw a card", None)? {
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
fn x_drain_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
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

/// yshtola._x_option: the drain X spell as a main-phase play (your turn, your deck's AI)
fn x_option(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !hand_has(g, p, c) || g.player(p).key != "yshtola" {
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
    // PORT(phase 9): log_secret (a practice table shows the others "exiles a card face down")
    crate::glog!(g, "    {} exiles {} with {why}", pname(g, p), g.db.get(c).name);
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
pub fn steal_pick(g: &Game, p: PlayerId, top: &[CardId]) -> CardId {
    // HUMAN(phase 9): the person picks (hc.choose)
    max_by(top, |c| stolen_worth(g, p, c)).unwrap()
}

/// yshtola.exile_for: a land is just exiled; anything else p may cast
pub fn exile_for(g: &mut Game, p: PlayerId, pick: CardId, owner: PlayerId, why: &str) {
    if g.db.get(pick).land {
        g.player_mut(owner).exile.push(pick);
        // PORT(phase 9): log_secret
        crate::glog!(g, "    {} exiles {} with {why}", pname(g, p), g.db.get(pick).name);
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
    // HUMAN(phase 9): the person picks the opponent
    let q = max_by(&opps, |q| threat(g, o, q)).unwrap();
    let n = 4.min(g.player(q).library.len());
    let top: Vec<CardId> = (0..n).map(|_| g.player_mut(q).library.pop().unwrap()).collect();
    let pick = steal_pick(g, o, &top);
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
    let pick = steal_pick(g, p, &top);
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
    // HUMAN(phase 9): the person's target, asked once the trigger is committed
    let Some(t) = taker_target(g, o, Some(src)) else { return Ok(()) };
    let name = format!("exile {}", g.perm(t).name);
    if !trigger_window(g, o, Some(src), &name, Some(5.0))? {
        return Ok(());
    }
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
    remove_card(&mut g.player_mut(owner).exile, cd);
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
    // HUMAN(phase 9): the person picks the three cards
    let picks = cap_picks(g, p, q).0;
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
    tutor(g, p, "any")?;
    let n = g.player(p).gy.iter().filter(|&&x| g.db.get(x).instant || g.db.get(x).sorcery).count();
    if n >= 2 {
        // HUMAN(phase 9): a person's mana pool gets the {B}{B}{B}
        g.player_mut(p).floating.b += 3;
        crate::glog!(g, "    spell mastery: Dark Petition adds {{B}}{{B}}{{B}}");
    }
    Ok("gy")
}

/// search for up to two enchantment cards (the AI: its two best for the board)
fn plea(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    // HUMAN(phase 9): the person searches (hc.search)
    tutor(g, p, "ench")?;
    tutor(g, p, "ench")?;
    Ok("gy")
}

/// yshtola.tutor_pick: the card a tutor finds: Sanguine Bond to start the drain engine, a finisher once it kills,
/// Necropotence early, a lock or pillowfort piece when under pressure. `okn`: the cards this tutor may find.
pub fn tutor_pick(g: &Game, p: PlayerId, kind: &str, okn: &[CardId]) -> Option<CardId> {
    let first = |ns: &[&str]| -> Option<CardId> { ns.iter().filter_map(|n| g.db.id(n)).find(|c| okn.contains(c)) };
    let pl = g.player(p);
    let have: Vec<&str> = pl.perms.iter().filter_map(|&m| g.perm(m).cd).map(|c| g.db.get(c).name.as_ref()).collect();
    let hand: Vec<&str> = pl.hand.iter().map(|&c| g.db.get(c).name.as_ref()).collect();
    let held = |n: &str| have.contains(&n) || hand.contains(&n);
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let mana = pl.lands.len() as i32
        + pl.perms.iter().filter(|&&m| g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Rock))).count() as i32;
    let mut order: Vec<&str> = vec![];
    if kind == "any" {
        let low: i32 = opps.iter().map(|&q| g.player(q).life).sum();
        if mana >= 7
            && !opps.is_empty()
            && (low <= 2 * (mana - 2) * opps.len() as i32
                || opps.iter().map(|&q| g.player(q).life).min().unwrap() <= mana - 2)
        {
            order.extend(["Debt to the Deathless", "Exsanguinate"]);
        }
        if ysh_perm(g, p).is_none() && !pl.cmd_in_zone && pl.tax >= 4 {
            order.push("Lightning Greaves");
        }
    }
    if !held("Sanguine Bond") {
        order.push("Sanguine Bond");
    }
    if pl.turns <= 6 && pl.life >= 25 && !held("Necropotence") {
        order.push("Necropotence");
    }
    if under_attack(g, p) && !have.iter().any(|n| matches!(*n, "Propaganda" | "Windborn Muse")) {
        order.push("Propaganda");
    }
    if !held("Marauding Blight-Priest") && kind == "any" {
        order.push("Marauding Blight-Priest");
    }
    if ysh_perm(g, p).is_some() && !held("Curiosity") {
        order.push("Curiosity");
    }
    order.extend(["Ill-Gotten Inheritance", "Prison Sentence", "Arrest", "Propaganda"]);
    if kind == "any" {
        order.extend(["Exsanguinate", "Debt to the Deathless"]);
    }
    first(&order)
}

/// yshtola.under_attack: the table's creatures hit hard enough to matter (total power at 12 or more)
pub fn under_attack(g: &Game, p: PlayerId) -> bool {
    g.opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && !g.perm(m).phased)
        .map(|m| epow(g, m))
        .sum::<i32>()
        >= 12
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
    crate::cardcode::add_counters(g, m, 1);
    add_eot_kw(g, m, "indestructible");
    add_eot_kw(g, m, "lifelink");
    crate::glog!(g, "    Take Up the Shield: {} gets a +1/+1 counter, lifelink and indestructible", g.perm(m).name);
}

// ======================================================== Champion's Helm (the AI equips it on Y'shtola)
fn helm(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != g.perm(src).owner
        || g.player(p).key != "yshtola"
        || human(g, p)
        || post.is_none()
        || !can_pay(g, p, 1, "", false)
    {
        return Ok(vec![]);
    }
    let t = if g.player(p).perms.iter().any(|&m| g.is_creature(m)) { helm_target(g, p) } else { None };
    let Some(t) = t else { return Ok(vec![]) };
    if g.perm(src).attached == Some(t) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 2.5 + 0.3 * pval(g, t),
        label: format!("equip Champion's Helm to {}", g.perm(t).name),
        act: Some(Action::Plan { f: helm_equip, arg: t.0 as i64 }),
    }])
}

// ======================================================== Zur in the 99: what to fetch
/// yshtola.fetch_value: an enchantment with mana value 3 or less from Zur's attack, for this deck (zur.fetch_value
/// calls it: CI.yshtola_fetch_value)
pub fn fetch_value(g: &Game, p: PlayerId, c: CardId, _zur: PermId) -> f64 {
    let d = g.db.get(c);
    let n: &str = &d.name;
    let pl = g.player(p);
    let have = pl.perms.iter().any(|&m| !g.perm(m).phased && card_name(g, m) == Some(n));
    if have && aura_spec(g, c).is_none() && !zur::is_lock(n) {
        return 0.0;
    }
    let turn = pl.turns as f64;
    match n {
        "Necropotence" => {
            if pl.life >= 20 {
                8.5 - 0.3 * turn
            } else {
                1.0
            }
        }
        "Mystic Remora" => 0f64.max(5.0 - 1.2 * (turn - 1.0)),
        "Propaganda" => {
            let n_cr = g
                .opps(p)
                .flat_map(|q| g.player(q).perms.iter().copied())
                .filter(|&m| g.is_creature(m) && !g.perm(m).phased)
                .count();
            2.0 + if under_attack(g, p) { 5.0 } else { 0.0 } + 0.15 * n_cr as f64
        }
        "Curiosity" => {
            let y = ysh_perm(g, p);
            let on_y = y.is_some_and(|y| auras_on(g, y).iter().any(|&a| card_name(g, a) == Some(n)));
            if y.is_some() && !on_y { 3.0 + 1.5 * g.opps(p).count() as f64 } else { 1.5 }
        }
        _ if zur::is_lock(n) => {
            let kind = zur::lock_kind(n);
            match zur::lock_host(g, p, n, false, &[]) {
                Some(t) => zur::lock_worth(g, p, t, kind),
                None => 0.0,
            }
        }
        _ => 0.5,
    }
}

// ======================================================== the AI: priorities
/// yshtola.trigger_bonus: what Y'shtola's cast trigger adds to casting c now (2 to each opponent, 2 life, and what
/// that sets off)
pub fn trigger_bonus(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let d = g.db.get(c);
    let Some(y) = ysh_perm(g, p) else { return 0 };
    if d.creature || d.land || d.cmc < 3 {
        return 0;
    }
    let n_opps = g.opps(p).count() as f64;
    let per = 2 + drain_extra(g, p);
    let cur = auras_on(g, y).iter().filter(|&&a| name_is(g, a, "Curiosity")).count() as f64;
    let mut v = 0.5 * n_opps * per as f64 + if bond_on(g, p) { 1.0 } else { 0.0 } + 1.5 * cur * n_opps;
    if g.opps(p).any(|q| g.player(q).life <= per) {
        v += 6.0;
    }
    20.min((4.0 * v) as i32)
}

/// yshtola.yshtola_prio: the deck's cast priority (0-90). A float only for a card taken from an opponent (its
/// ability-language value), so `ai::decks::deck_prio_f` keeps it as Python does.
pub fn yshtola_prio(g: &Game, p: PlayerId, c: CardId) -> f64 {
    let pl = g.player(p);
    let d = g.db.get(c);
    let (t, n) = (&d.tags, &*d.name);
    let ysh = ysh_perm(g, p);
    if pl.stolen.contains_key(&c) {
        // a card taken from an opponent
        let v = generic_prio(g, p, c) as f64;
        if v == 0.0 && d.has_dsl() {
            return crate::dsl::card_value(g, p, c) * 10.0;
        }
        return v;
    }
    if c == pl.cmd {
        return if pl.turns >= 2 { 90.0 } else { 60.0 };
    }
    let b = trigger_bonus(g, p, c);
    let v: i32 = (|| {
        if t.has(Tag::Rock) {
            return if pl.turns <= 5 { 82 } else { 30 + b };
        }
        if t.has(Tag::Citadel) {
            // Bolas's Citadel: with life above the threat floor
            let v = citadel_prio(g, p, c);
            return if v != 0 { v + b } else { 0 };
        }
        if n == "The One Ring" {
            return one_ring_prio(g, p, c) + b; // draw engine, a turn of protection, and it triggers her
        }
        if t.has(Tag::Necro) {
            // Necropotence: from the cards it will buy
            let v = necro_prio(g, p, c);
            return if v != 0 { v + b } else { 0 };
        }
        if t.has(Tag::Remora) {
            return if pl.turns <= 4 { 66 } else { 8 };
        }
        if n == "Esper Sentinel" {
            return if pl.turns <= 4 { 62 } else { 30 };
        }
        if t.has(Tag::Xdrain) || t.has(Tag::Tokx) {
            return 0; // X spells: their own options
        }
        match n {
            "Sanguine Bond" => return 76 + b,
            "Marauding Blight-Priest" => return 62,
            "Ill-Gotten Inheritance" => return 54 + b,
            "Polluted Bonds" => return (if pl.turns <= 8 { 56 } else { 38 }) + b, // opponents' land drops to come
            "Propaganda" | "Windborn Muse" => {
                if pl.perms.iter().any(|&m| matches!(card_name(g, m), Some("Propaganda" | "Windborn Muse"))) {
                    return 30 + b;
                }
                return 72.min(44 + if under_attack(g, p) { 18 } else { 0 }) + b;
            }
            _ => {}
        }
        if zur::is_lock(n) {
            let kind = zur::lock_kind(n);
            return match zur::lock_host(g, p, n, true, &[]) {
                Some(tg) => 80.min((42.0 + 7.0 * zur::lock_worth(g, p, tg, kind)) as i32) + b,
                None => 0,
            };
        }
        match n {
            "Curiosity" => {
                return if ysh.is_some_and(|y| !zur::untargetable_by_you(g, y)) { 50 } else { 18 };
            }
            "Bribery" => return 56 + b,
            "Hostage Taker" => {
                let tg = if !human(g, p) { taker_target(g, p, None) } else { None };
                return match tg {
                    Some(tg) => 45 + 30.min((6.0 * pval(g, tg)) as i32),
                    None => 25,
                };
            }
            "Massacre Wurm" => {
                let k = g
                    .opps(p)
                    .flat_map(|q| g.player(q).perms.iter().copied())
                    .filter(|&m| g.is_creature(m) && !g.perm(m).phased && etgh(g, m) <= 2)
                    .count() as i32;
                return 80.min(46 + 6 * k);
            }
            "Gonti, Lord of Luxury" => return 50,
            "Thief of Sanity" => return 52,
            "Urborg Syphon-Mage" => return 46,
            "Zur the Enchanter" => return 54,
            "Champion's Helm" => return (if ysh.is_some() { 46 } else { 22 }) + b,
            _ => {}
        }
        if t.str(Tag::Prot) == Some("boots") {
            return if ysh.is_some() { 58 } else { 25 };
        }
        match n {
            "Bastion Protector" => return if ysh.is_some() { 50 } else { 30 },
            "Notion Thief" => return 50,
            _ => {}
        }
        if t.has(Tag::Tithe) {
            // Smothering Tithe: from the Treasures it will make
            return crate::cardcode::tithe_prio(g, p, c) + b;
        }
        if t.has(Tag::Sphinx) {
            // Consecrated Sphinx: from what it will draw
            return sphinx_prio(g, p, c);
        }
        match n {
            "Skyclave Apparition" => {
                let tg = legal_targets(g, p, "exile", "nl", true, Some(c));
                if tg.is_empty() {
                    return 25;
                }
                let best = tg.iter().map(|&m| pval(g, m)).fold(f64::NEG_INFINITY, f64::max);
                return 45 + 30.min((6.0 * best) as i32);
            }
            "Restoration Angel" => return 25, // held for flash
            "The Eternal Wanderer" => return 58 + b,
            "Prayer of Binding" | "Static Net" | "Memory Trap" => {
                // exile an opponent's best nonland permanent
                let best = g
                    .opps(p)
                    .flat_map(|q| g.player(q).perms.iter().copied())
                    .filter(|&m| !g.perm(m).phased && !untargetable(g, m))
                    .map(|m| pval(g, m))
                    .fold(0.0, f64::max);
                return if best >= 3.0 { 75.min((40.0 + 6.0 * best) as i32) + b } else { 0 };
            }
            "Jester's Cap" => return 34 + b,
            _ => {}
        }
        if t.str(Tag::Tut).is_some_and(|s| !s.is_empty()) {
            return 52 + b;
        }
        if n == "Triplicate Spirits" {
            return 46 + b;
        }
        if t.has(Tag::Draw) && (d.instant || d.sorcery) {
            return 46 + b;
        }
        if d.creature {
            return 40;
        }
        0
    })();
    v as f64
}

// ------------------------------------------------------------------ the rigid AI's extra plays (ais.yshtola_main)
// The rigid AI (ais.MAIN) isn't ported: the Rust plays the heuristic AI (brain), which reaches the X spells through
// `hand_options` and the abilities through `options`. These two are ported for completeness.

/// yshtola.yshtola_x_spell: a drain X spell worth casting now
pub fn yshtola_x_spell(g: &mut Game, p: PlayerId, _post: bool) -> Res<bool> {
    let opts: Vec<(f64, CardId)> = g
        .player(p)
        .hand
        .iter()
        .copied()
        .filter(|&c| g.db.get(c).tag(Tag::Xdrain))
        .filter_map(|c| x_drain_option(g, p, c).map(|o| (o.0, c)))
        .collect();
    let Some((u, c)) = max_by(&opts, |o| o.0) else { return Ok(false) };
    Ok(u >= 4.0 && x_drain_go(g, p, c.0 as i64)?)
}

/// yshtola.yshtola_abilities: the best activated ability with a clear use (Syphon-Mage, Ill-Gotten Inheritance,
/// Jester's Cap, the Helm)
pub fn yshtola_abilities(g: &mut Game, p: PlayerId, post: Option<bool>) -> Res<bool> {
    let mut opts: Vec<Opt> = vec![];
    for (src, imp) in crate::engine::hooks::hooked(g, crate::hooks::Event::Options) {
        if g.perm(src).owner == p {
            opts.extend((imp.options.unwrap())(g, src, p, post)?);
        }
    }
    opts.retain(|o| o.utility >= 2.5);
    let mut best: Option<&Opt> = None;
    for o in &opts {
        if best.is_none_or(|b| o.utility > b.utility) {
            best = Some(o);
        }
    }
    match best.and_then(|o| o.act.clone()) {
        Some(a) => crate::ai::act::perform(g, p, &a),
        None => Ok(false),
    }
}

// ------------------------------------------------------------------ protection and wipes
/// yshtola.yshtola_protect: removal at Y'shtola or another key creature: Take Up the Shield against destroy and
/// damage, then the answers in zur.py (Clever Concealment, Rootborn Defenses, Restoration Angel)
pub fn yshtola_protect(
    g: &mut Game,
    owner: PlayerId,
    m: PermId,
    kind: Sym,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    if !g.is_creature(m) || pval(g, m) < 4.0 {
        return Ok(false);
    }
    if kind == "destroy" || kind.starts_with("dmg") {
        for c in g.player(owner).hand.clone() {
            let d = g.db.get(c);
            if d.tags.str(Tag::Prot) == Some("shield")
                && can_pay(g, owner, d.generic, &d.pips, false)
                && !zur::untargetable_by_you(g, m)
            {
                if !pay_card(g, owner, c, 0)? {
                    return Ok(false);
                }
                crate::glog!(g, "    {} casts Take Up the Shield on {}", pname(g, owner), g.perm(m).name);
                shield(g, m);
                return Ok(true);
            }
        }
    }
    zur::zur_protect(g, owner, m, kind, actor, spell)
}

/// yshtola.yshtola_wipe_response (zur.zur_wipe_response)
pub fn yshtola_wipe_response(g: &mut Game, q: PlayerId, kind: Sym) -> Res<Option<Sym>> {
    zur::zur_wipe_response(g, q, kind)
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

// ======================================================== zur.py's pieces this deck uses
/// MERGE(zur): a port of the zur.py functions Y'shtola's code calls, until `impls/zur.rs` (ported in another
/// worktree) is merged; then `use crate::impls::zur as zur;` replaces it. Each function is zur.py's of the same name.
mod zur_shim {
    use super::*;
    use crate::engine::zones::{Tokens, make_tokens};

    /// zur.LOCKS / LOCK_KINDS: the lock Auras and their kind
    const LOCKS: [(&str, &str); 5] = [
        ("Arrest", "arrest"),
        ("Prison Sentence", "arrest"),
        ("Luminous Bonds", "pacify"),
        ("Bound in Silence", "pacify"),
        ("Encrust", "encrust"),
    ];

    /// zur.LOCK_KINDS: the locks each kind puts on what it enchants
    fn kind_locks(kind: &str) -> &'static [&'static str] {
        match kind {
            "arrest" => &["pacify", "noact"],
            "pacify" => &["pacify"],
            "encrust" => &["frozen", "noact"],
            "kasmina" => &["neuter"],
            _ => &[],
        }
    }

    /// `name in zur.LOCKS`
    pub fn is_lock(name: &str) -> bool {
        LOCKS.iter().any(|x| x.0 == name)
    }

    /// the kind of lock Aura `name` (Python: `next(k for k, v in LOCK_KINDS.items() if frozenset(v) == LOCKS[name])`)
    pub fn lock_kind(name: &str) -> &'static str {
        LOCKS.iter().find(|x| x.0 == name).map(|x| x.1).unwrap_or("")
    }

    fn locks_of(g: &Game, a: PermId) -> &'static [&'static str] {
        card_name(g, a).map_or(&[], |n| kind_locks(lock_kind(n)))
    }

    /// zur.locked: is permanent m locked down by an Aura (kind: pacify, noact, frozen, neuter)?
    pub fn locked(g: &Game, m: PermId, kind: &str) -> bool {
        g.auras.iter().any(|&a| {
            let x = g.perm(a);
            x.attached == Some(m) && x.cd.is_some() && locks_of(g, a).contains(&kind) && !x.phased && x.on_bf
        })
    }

    /// zur.lock_factor: how much of m's value is left under the Auras locking it
    pub fn lock_factor(g: &Game, m: PermId) -> f64 {
        let mut f: f64 = 1.0;
        for &a in &g.auras {
            let x = g.perm(a);
            if x.attached != Some(m) || x.cd.is_none() || x.phased || !x.on_bf {
                continue;
            }
            let ks = locks_of(g, a);
            if ks.is_empty() {
                continue;
            }
            if ks.contains(&"neuter") {
                f = f.min(0.15);
            } else if ks.contains(&"pacify") {
                f = f.min(if ks.contains(&"noact") { 0.3 } else { 0.4 });
            } else if ks.contains(&"frozen") {
                f = f.min(0.35);
            }
        }
        f
    }

    /// zur.lock_host: the best opposing permanent for lock Aura `name` (targeted: cast, so hexproof and ward stop
    /// it), or None
    pub fn lock_host(g: &Game, p: PlayerId, name: &str, targeted: bool, exclude: &[PermId]) -> Option<PermId> {
        let kind = lock_kind(name);
        let ok_art = name == "Encrust";
        let col = Colors::from_letters(if name != "Encrust" && name != "Kasmina's Transmutation" { "W" } else { "U" });
        let (mut best, mut bv) = (None, 0.0);
        for q in g.opps(p) {
            for &m in &g.player(q).perms {
                if g.perm(m).phased || exclude.contains(&m) {
                    continue;
                }
                let art = g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT));
                if !(g.is_creature(m) || (ok_art && art)) {
                    continue;
                }
                if protected_from(g, m, col) || (targeted && untargetable(g, m)) {
                    continue;
                }
                if kind_locks(kind).iter().any(|k| locked(g, m, k)) {
                    continue;
                }
                let v = lock_worth(g, p, m, kind);
                if v > bv {
                    (best, bv) = (Some(m), v);
                }
            }
        }
        if bv >= 2.0 { best } else { None }
    }

    /// zur.lock_worth: what locking m this way takes away from its controller
    pub fn lock_worth(g: &Game, _p: PlayerId, m: PermId, kind: &str) -> f64 {
        let v = pval(g, m) * lock_factor(g, m);
        let acts = activated(g, m);
        let cr = g.is_creature(m);
        let pw = 4.min(epow(g, m)) as f64;
        match kind {
            "pacify" => {
                if !cr {
                    return 0.0;
                }
                v * (0.45 + 0.1 * pw) * if acts { 0.5 } else { 1.0 }
            }
            "arrest" => v * (0.55 + 0.08 * pw + if acts { 0.4 } else { 0.0 }),
            "encrust" => {
                if cr || acts {
                    v * (0.5 + if acts { 0.5 } else { 0.0 })
                } else {
                    0.0
                }
            }
            "kasmina" => {
                if !cr {
                    return 0.0;
                }
                let rich = g
                    .perm(m)
                    .cd
                    .is_some_and(|c| g.registry.get(c).is_some_and(|i| i.live()) || g.db.get(c).has_dsl() || acts);
                v * if rich { 0.9 } else { 0.6 }
            }
            _ => v,
        }
    }

    /// zur.activated: does m have activated abilities worth stopping (mana abilities included)?
    pub fn activated(g: &Game, m: PermId) -> bool {
        use crate::dsl::model::Ability;
        let Some(c) = g.perm(m).cd else { return false };
        let d = g.db.get(c);
        if d.tag(Tag::Rock) || d.tag(Tag::Dork) || d.tag(Tag::Clamp) {
            return true;
        }
        if g.registry.get(c).is_some_and(|i| i.options.is_some()) {
            return true;
        }
        if matches!(&*d.name, "Lightning Greaves" | "Swiftfoot Boots" | "Whispersilk Cloak") {
            return true;
        }
        d.abilities.iter().any(|a| matches!(a, Ability::Activated { .. } | Ability::Loyalty { .. }))
    }

    /// zur.untargetable_by_you: shroud (Lightning Greaves) stops even your own spells and abilities
    pub fn untargetable_by_you(g: &Game, m: PermId) -> bool {
        let o = g.perm(m).owner;
        g.player(o).perms.iter().any(|&e| {
            let x = g.perm(e);
            x.attached == Some(m) && card_name(g, e) == Some("Lightning Greaves") && x.on_bf && x.owner == o
        }) || crate::dsl::has_kw(g, m, "shroud")
    }

    /// zur.populate: a copy of p's best creature token
    pub fn populate(g: &mut Game, p: PlayerId) -> Res {
        let toks: Vec<PermId> = g
            .player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| g.perm(m).token && g.is_creature(m) && !g.perm(m).phased)
            .collect();
        let mut best: Option<(PermId, (i32, bool))> = None;
        for &m in &toks {
            let k = (epow(g, m), g.perm(m).fly);
            if best.is_none_or(|b| k > b.1) {
                best = Some((m, k));
            }
        }
        let Some((t, _)) = best else { return Ok(()) };
        let x = g.perm(t).clone();
        let spec = Tokens {
            tgh: Some(x.tgh),
            fly: x.fly,
            color: Some(x.colors),
            types: x.ttypes.clone(),
            ..Tokens::new(1, x.pow)
        };
        for n in make_tokens(g, p, spec)? {
            let y = g.perm_mut(n);
            (y.lifelink, y.dt) = (x.lifelink, x.dt);
            if !x.data.is_empty() {
                y.data = x.data.clone();
            }
        }
        Ok(())
    }

    /// zur.rootborn: Rootborn Defenses resolves: populate, then indestructible until end of turn
    pub fn rootborn(g: &mut Game, p: PlayerId) -> Res {
        populate(g, p)?;
        for m in g.player(p).perms.clone() {
            if g.is_creature(m) {
                add_eot_kw(g, m, "indestructible");
            }
        }
        crate::glog!(g, "  {} casts Rootborn Defenses: creatures gain indestructible", pname(g, p));
        Ok(())
    }

    /// zur.zur_protect: removal at a key creature: phase it out with its Auras (Clever Concealment), make it
    /// indestructible (Rootborn Defenses), or blink it with Restoration Angel (not a creature with Auras on it, which
    /// would lose them)
    pub fn zur_protect(
        g: &mut Game,
        owner: PlayerId,
        m: PermId,
        kind: Sym,
        _actor: Option<PlayerId>,
        _spell: Option<CardId>,
    ) -> Res<bool> {
        if pval(g, m) < 4.0 || !g.is_creature(m) {
            return Ok(false);
        }
        let auras = auras_on(g, m);
        for c in g.player(owner).hand.clone() {
            let d = g.db.get(c);
            let tp = d.tags.str(Tag::Prot);
            if tp == Some("phase") && can_pay(g, owner, d.generic, &d.pips, d.tag(Tag::Convoke)) {
                // Clever Concealment
                if !pay_card(g, owner, c, 0)? {
                    return Ok(false);
                }
                g.perm_mut(m).phased = true;
                for &a in &auras {
                    g.perm_mut(a).phased = true;
                }
                crate::glog!(g, "    {} casts {}: {} phases out", pname(g, owner), g.db.get(c).name, g.perm(m).name);
                return Ok(true);
            }
            if tp == Some("indes")
                && (kind == "destroy" || kind.starts_with("dmg"))
                && can_pay(g, owner, d.generic, &d.pips, false)
            {
                // Rootborn Defenses
                if !pay_card(g, owner, c, 0)? {
                    return Ok(false);
                }
                rootborn(g, owner)?;
                return Ok(true);
            }
        }
        if !auras.is_empty() || (!matches!(kind, "destroy" | "exile" | "bounce" | "tuck") && !kind.starts_with("dmg")) {
            return Ok(false);
        }
        for c in g.player(owner).hand.clone() {
            let d = g.db.get(c);
            if &*d.name == "Restoration Angel" && can_pay(g, owner, d.generic, &d.pips, false) {
                if !pay_card(g, owner, c, 0)? {
                    return Ok(false);
                }
                remove_card(&mut g.player_mut(owner).gy, c);
                // Python sets g.resto_target = m, and the Angel's enters trigger (t2.py) blinks it: the zur.rs and
                // t2.rs ports carry that; here the Angel enters
                enter(g, owner, c, Enter::default())?;
                let nm = g.perm(m).cd.map_or(g.perm(m).name.to_string(), |c| g.db.get(c).name.to_string());
                crate::glog!(g, "    {} casts {}: blinks {nm}", pname(g, owner), g.db.get(c).name);
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// zur.zur_wipe_response: a wipe: phase out your creatures and the Auras on them (Clever Concealment), or make
    /// them indestructible (Rootborn Defenses)
    pub fn zur_wipe_response(g: &mut Game, q: PlayerId, kind: Sym) -> Res<Option<Sym>> {
        let all = matches!(kind, "rift" | "rebuke");
        let loss = g.player(q).perms.iter().filter(|&&m| g.is_creature(m) || all).map(|&m| pval(g, m)).psum();
        if loss < 6.0 {
            return Ok(None);
        }
        for c in g.player(q).hand.clone() {
            let d = g.db.get(c);
            if d.tags.str(Tag::Prot) == Some("phase") && can_pay(g, q, d.generic, &d.pips, d.tag(Tag::Convoke)) {
                if !pay_card(g, q, c, 0)? {
                    return Ok(None);
                }
                let mine: Vec<PermId> = g.player(q).perms.iter().copied().filter(|&m| g.is_creature(m)).collect();
                for m in g.player(q).perms.clone() {
                    if g.is_creature(m) || g.perm(m).attached.is_some_and(|a| mine.contains(&a)) {
                        g.perm_mut(m).phased = true;
                    }
                }
                crate::glog!(g, "    {} casts {}: their creatures phase out", pname(g, q), g.db.get(c).name);
                return Ok(Some("all"));
            }
        }
        if matches!(kind, "destroy" | "dmg13" | "austere" | "nib") {
            for c in g.player(q).hand.clone() {
                let d = g.db.get(c);
                if d.tags.str(Tag::Prot) == Some("indes") && can_pay(g, q, d.generic, &d.pips, false) {
                    if !pay_card(g, q, c, 0)? {
                        return Ok(None);
                    }
                    rootborn(g, q)?;
                    return Ok(Some("indes"));
                }
            }
        }
        Ok(None)
    }
}
