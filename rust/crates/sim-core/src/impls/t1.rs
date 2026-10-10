//! Python's `cards/impl/t1.py`: the Tier 1 pool decks (Isshin, Lathril, Light-Paws, Tatyova, Teysa).

use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{bolt_something, cast_card, on_cast};
use crate::engine::life::{check_state, gain, lose_life};
use crate::engine::mana::{can_pay, pay, total_mana};
use crate::engine::stack::{ability_window, counter_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{commander_out, epow, etgh, has_type, once_per_turn, pval, subtypes, threat, untargetable};
use crate::engine::zones::{
    Enter, Tokens, add_treasure, agent_for, agent_take, die, discard_index, draw, enter, enter_token_copy,
    land_to_hand, landfall, leave, make_tokens, max_by, mill, searchable,
};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Sacrificed, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::state::{DataKey, Game, PermData, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

// ------------------------------------------------------------------ helpers
/// the once-per-turn key of src's first attack (Python's `f'first_attack_{id(src)}'`)
fn first_attack_key(src: PermId) -> Sym {
    intern(&format!("first_attack_{}", src.0))
}

/// t1.first_attack: mark src's first attack this turn
fn first_attack(g: &mut Game, src: Src) -> bool {
    let p = g.perm(src).owner;
    once_per_turn(g, p, first_attack_key(src))
}

/// t1.fresh: once_per_turn(g, p, key) would pass; sets nothing (trigger guards run twice: probe, then real)
fn fresh(g: &Game, p: PlayerId, key: Sym) -> bool {
    g.player(p).flag_turn.get(key) != Some(&g.turn_stamp())
}

/// t1.untap_all: untap p's creatures (only those in `only`, if given)
fn untap_all(g: &mut Game, p: PlayerId, only: Option<&[PermId]>) {
    for m in g.player(p).perms.clone() {
        if g.is_creature(m) && only.is_none_or(|o| o.contains(&m)) {
            g.perm_mut(m).tapped = false;
        }
    }
}

/// t1.attacking_tokens: n p/t tokens entering tapped and attacking
fn attacking_tokens(g: &mut Game, p: PlayerId, n: u32, pw: i32, color: &str) -> Res<Vec<PermId>> {
    let spec = Tokens { attacking: true, sick: false, color: Some(Colors::from_letters(color)), ..Tokens::new(n, pw) };
    make_tokens(g, p, spec)
}

/// cardimpl._eot: +dp/+dt until end of turn
fn eot(g: &mut Game, m: PermId, dp: i32, dt: i32) {
    let x = &mut g.perm_mut(m).eot_pt;
    *x = (x.0 + dp, x.1 + dt);
}

/// `g.eot_kw.setdefault(id(m), set()).add(kw)`: a keyword until end of turn
fn add_kw(g: &mut Game, m: PermId, kw: Sym) {
    let k = &mut g.perm_mut(m).eot_kw;
    if !k.contains(&kw) {
        k.push(kw);
    }
}

/// `m in p.perms`
fn controls(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.perm(m).on_bf && g.perm(m).owner == p
}

/// the card's subtypes include s (Python's `s in c.subtypes`)
fn card_sub(g: &Game, c: CardId, s: &str) -> bool {
    g.db.get(c).has_subtype(s)
}

/// the first item with the highest key, for keys that are tuples (Python's `max(xs, key=...)`)
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

/// a stable ascending sort by a tuple key (Python's `sorted(xs, key=...)`)
fn sort_by_key<T: Copy, K: PartialOrd>(xs: &mut [T], key: impl Fn(T) -> K) {
    let mut keyed: Vec<(K, T)> = xs.iter().map(|&x| (key(x), x)).collect();
    keyed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    for (i, (_, x)) in keyed.into_iter().enumerate() {
        xs[i] = x;
    }
}

/// remove the first copy of c from a zone (Python's `list.remove`)
fn remove_first(v: &mut Vec<CardId>, c: CardId) {
    if let Some(i) = v.iter().position(|&x| x == c) {
        v.remove(i);
    }
}

/// cardimpl.count_type: permanents of subtype t controlled by p (or by anyone)
pub fn count_type(g: &Game, p: PlayerId, t: &str, everyone: bool) -> i32 {
    let ps: Vec<PlayerId> =
        if everyone { g.players.iter().filter(|q| q.alive).map(|q| q.id).collect() } else { vec![p] };
    ps.iter()
        .flat_map(|&q| g.player(q).perms.iter().copied())
        .filter(|&m| !g.perm(m).phased && has_type(g, m, t))
        .count() as i32
}

/// an activated ability of src for the AI's option list
fn ability(utility: f64, label: &str, src: PermId, f: crate::hooks::AbilityFn) -> Opt {
    Opt { utility, label: label.into(), act: Some(Action::Ability { src, f, arg: 0 }) }
}

// ======================================================== Isshin, Two Heavens as One
/// attack-caused triggers of your permanents fire twice (hand tags, compiled triggers and hooks); ETB of tokens
/// entering attacking is not doubled, matching the card
fn isshin(g: &Game, src: Src, p: PlayerId, kind: Sym, _x: Option<PermId>) -> i32 {
    (kind == "attack" && g.perm(src).owner == p) as i32
}

/// haste; 1 damage to the defending player per attacking creature
fn hellrider(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == p
        && trigger_window(g, p, Some(src), &format!("deal {} damage to the defending player", atk.len()), None)?
    {
        for _ in atk {
            lose_life(g, d, 1, Some(p), "triggers", None)?;
        }
    }
    Ok(vec![])
}

/// drain per attacker (the forced-block activation is in partials)
fn hordechief(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == p && trigger_window(g, p, Some(src), &format!("drain {}", atk.len()), None)? {
        for _ in atk {
            lose_life(g, d, 1, Some(p), "drain", None)?;
            gain(g, p, 1)?;
        }
    }
    Ok(vec![])
}

/// first attack each turn: untap all creatures, one additional combat
fn aurelia(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == p && atk.contains(&src) && fresh(g, p, first_attack_key(src)) {
        let ok = trigger_window(g, p, Some(src), "untap all creatures, additional combat", Some(6.0))?;
        first_attack(g, src);
        if !ok {
            return Ok(vec![]);
        }
        untap_all(g, p, None);
        g.player_mut(p).extra_combats += 1;
        crate::glog!(g, "    Aurelia: untap, additional combat");
    }
    Ok(vec![])
}

/// first combat: attackers untap and gain first strike; additional combat
fn karlach(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == p
        && g.player(p).combat_no == 1
        && trigger_window(g, p, Some(src), "untap attackers, first strike, additional combat", Some(6.0))?
    {
        untap_all(g, p, Some(atk));
        for &m in atk {
            add_kw(g, m, "first strike");
        }
        g.player_mut(p).extra_combats += 1;
    }
    Ok(vec![])
}

/// dethrone; first attack at the highest life total: untap attackers, extra combat
fn scourge(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    let top = g.players.iter().filter(|q| q.alive).map(|q| q.life).max().unwrap_or(0);
    if g.perm(src).owner == p
        && atk.contains(&src)
        && fresh(g, p, first_attack_key(src))
        && g.player(d).alive
        && g.player(d).life >= top
    {
        let ok = trigger_window(g, p, Some(src), "untap attackers, additional combat", Some(6.0))?;
        first_attack(g, src);
        if ok {
            untap_all(g, p, Some(atk));
            g.player_mut(p).extra_combats += 1;
        }
    }
    Ok(vec![])
}

/// attacking Cat token (the blocking token is in rules)
fn brimaz(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == p
        && atk.contains(&src)
        && trigger_window(g, p, Some(src), "create an attacking 1/1 Cat", None)?
    {
        return attacking_tokens(g, p, 1, 1, "W");
    }
    Ok(vec![])
}

/// two attacking Human tokens (meld ignored: its partner is not in the deck)
fn garrison(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == p
        && atk.contains(&src)
        && trigger_window(g, p, Some(src), "create two attacking 1/1 Humans", None)?
    {
        return attacking_tokens(g, p, 2, 1, "R");
    }
    Ok(vec![])
}

/// counter, then X attacking Gnome tokens (X = its +1/+1 counters)
fn anim_pakal(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner != p || !atk.iter().any(|&m| !has_type(g, m, "gnome")) {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "a +1/+1 counter, then X attacking Gnomes", None)? {
        return Ok(vec![]);
    }
    g.perm_mut(src).plus += 1;
    let n = g.perm(src).plus.max(0) as u32;
    let made = attacking_tokens(g, p, n, 1, "")?;
    for &m in &made {
        g.perm_mut(m).ttypes = vec!["gnome"];
    }
    Ok(made)
}

/// sacrifices spare fodder: two attacking tokens, then damage equal to tokens (3+) or draw; mode choice is a fixed
/// rule
fn caesar(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner != p {
        return Ok(vec![]);
    }
    let fod = |g: &Game| -> Vec<PermId> {
        g.player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| g.is_creature(m) && m != src && !atk.contains(&m) && (g.perm(m).token || pval(g, m) < 2.0))
            .collect()
    };
    if fod(g).is_empty() || !trigger_window(g, p, Some(src), "sacrifice a creature: two attacking tokens", None)? {
        return Ok(vec![]);
    }
    let fod = fod(g);
    if fod.is_empty() {
        return Ok(vec![]);
    }
    let victim = crate::engine::zones::min_by(&fod, |m| pval(g, m)).unwrap();
    die(g, victim, "sac")?;
    let made = attacking_tokens(g, p, 2, 1, "RW")?;
    for &m in &made {
        g.perm_mut(m).sick = false;
    }
    let toks = g.player(p).perms.iter().filter(|&&m| g.perm(m).token && g.is_creature(m)).count() as i32;
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let tgt = max_by(&opps, |q| threat(g, p, q));
    match tgt {
        Some(t) if toks >= 3 => lose_life(g, t, toks, Some(p), "triggers", None)?,
        _ => {
            draw(g, p, 1, false)?;
            lose_life(g, p, 1, Some(p), "other", None)?;
        }
    }
    Ok(made)
}

/// a 1/1 Goblin with haste at the beginning of combat (here: upkeep), for Goblin Rabblemaster and Legion Warboss.
/// Not registered: in Python a later module's hook for the same card and event replaces this one (rules2.py's upkeep), so it never runs.
#[allow(dead_code)]
fn goblin_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner == p && trigger_window(g, p, Some(src), "create a 1/1 Goblin", None)? {
        let spec = Tokens { sick: false, color: Some(Colors::from_letters("R")), ..Tokens::new(1, 1) };
        for m in make_tokens(g, p, spec)? {
            g.perm_mut(m).ttypes = vec!["goblin"];
        }
    }
    Ok(())
}

/// hasty Goblin each turn, +1/+0 per other attacking Goblin; the must-attack clause is ignored (the AI attacks when
/// it wants)
fn rabble_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == p
        && atk.contains(&src)
        && trigger_window(g, p, Some(src), "+1/+0 per other attacking Goblin", None)?
    {
        let n = atk.iter().filter(|&&m| m != src && has_type(g, m, "goblin")).count() as i32;
        eot(g, src, n, 0);
    }
    Ok(vec![])
}

/// spends all spare mana on X attacking tokens; kept with 10+ permanents
fn tilonalli(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner != p || !atk.contains(&src) {
        return Ok(vec![]);
    }
    let spare = |g: &Game| total_mana(g, p, false) as i32 - 1;
    let x = spare(g);
    if x < 1 || !can_pay(g, p, x as u32, "R", false) {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "pay X: X attacking tokens", None)? {
        return Ok(vec![]);
    }
    let x = spare(g);
    if x < 1 || !can_pay(g, p, x as u32, "R", false) {
        return Ok(vec![]);
    }
    pay(g, p, x as u32, "R", false)?;
    let made = attacking_tokens(g, p, x as u32, 1, "R")?;
    let city = g.player(p).perms.len() + g.player(p).lands.len() >= 10;
    if !city {
        for &m in &made {
            g.perm_mut(m).temp = true; // exiled at the end step
        }
    }
    Ok(made)
}

/// t1._shares_type: two permanents share a subtype
fn shares_type(g: &Game, a: PermId, b: PermId) -> bool {
    let sb = subtypes(g, b);
    subtypes(g, a).iter().any(|t| sb.contains(t))
}

/// each attacker +1/+0 per other attacker sharing a creature type
fn animosity(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner != p
        || !trigger_window(g, p, Some(src), "attackers get +1/+0 per attacker sharing a type", None)?
    {
        return Ok(vec![]);
    }
    for &m in atk {
        let n = atk.iter().filter(|&&x| x != m && shares_type(g, m, x)).count() as i32;
        if n != 0 {
            eot(g, m, n, 0);
        }
    }
    Ok(vec![])
}

/// ETB: creatures gain flying and +X/+X
fn moonshaker(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, p, Some(src), "creatures gain flying and +X/+X", Some(6.0))? {
        let cr: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&x| g.is_creature(x)).collect();
        let n = cr.len() as i32;
        for x in cr {
            eot(g, x, n, n);
            add_kw(g, x, "flying");
        }
    }
    Ok(())
}

/// other creatures entering get +2/+0 and haste
fn battledriver(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src
        && g.perm(src).owner == p
        && g.is_creature(m)
        && g.perm(m).owner == p
        && trigger_window(g, p, Some(src), &format!("{} gets +2/+0 and haste", g.perm(m).name), None)?
    {
        eot(g, m, 2, 0);
        g.perm_mut(m).sick = false;
    }
    Ok(())
}

/// pays {1} to draw when a small creature enters (if it can, hand not full)
fn mentor_meek(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src || g.perm(src).owner != p || !g.is_creature(m) || g.perm(m).owner != p || epow(g, m) > 2 {
        return Ok(());
    }
    // HUMAN(phase 9): a person pays {1} or not as the trigger resolves (hc.pay_tax)
    if can_pay(g, p, 1, "", false)
        && g.player(p).hand.len() <= 6
        && trigger_window(g, p, Some(src), "pay {1}: draw a card", None)?
        && can_pay(g, p, 1, "", false)
    {
        pay(g, p, 1, "", false)?;
        draw(g, p, 1, false)?;
    }
    Ok(())
}

/// draw once each turn when small creatures enter
fn welcoming(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    let key = intern(&format!("welcoming{}", src.0));
    if m != src
        && g.perm(src).owner == p
        && g.is_creature(m)
        && g.perm(m).owner == p
        && epow(g, m) <= 2
        && fresh(g, p, key)
    {
        let ok = trigger_window(g, p, Some(src), "draw a card", None)?;
        once_per_turn(g, p, key);
        if ok {
            draw(g, p, 1, false)?;
        }
    }
    Ok(())
}

/// attack: exile top card playable this turn (as an impulse draw), +1/+1 counter
fn laelia(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == p
        && atk.contains(&src)
        && !g.player(p).library.is_empty()
        && trigger_window(g, p, Some(src), "exile the top card, +1/+1 counter", None)?
    {
        let pl = g.player_mut(p);
        if let Some(c) = pl.library.pop() {
            pl.hand.push(c);
            pl.impulse.push(c);
            pl.seen.insert(c);
        }
        g.perm_mut(src).plus += 1;
    }
    Ok(vec![])
}

/// random hasty token each upkeep (trample on the 3/1 ignored)
fn merriment(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner != p || !trigger_window(g, p, Some(src), "create a random Outlaw", None)? {
        return Ok(());
    }
    let rw = Some(Colors::from_letters("RW"));
    let k = g.rng.below(3);
    if k == 0 {
        make_tokens(g, p, Tokens { tgh: Some(1), sick: false, color: rw, ..Tokens::new(1, 3) })?;
    } else if k == 1 {
        make_tokens(g, p, Tokens { tgh: Some(1), sick: false, lifelink: true, color: rw, ..Tokens::new(1, 2) })?;
    } else {
        make_tokens(g, p, Tokens { tgh: Some(2), sick: false, color: rw, ..Tokens::new(1, 1) })?;
        bolt_something(g, p, 1)?;
    }
    Ok(())
}

/// each turn: Treasure; plus a card above 15 life and a 3/2 above 20
fn bmc(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner != p || !trigger_window(g, p, Some(src), "choose modes", None)? {
        return Ok(());
    }
    add_treasure(g, p, 1)?;
    lose_life(g, p, 1, Some(p), "other", None)?;
    if g.player(p).life > 15 {
        draw(g, p, 1, false)?;
        lose_life(g, p, 2, Some(p), "other", None)?;
    }
    if g.player(p).life > 20 {
        make_tokens(g, p, Tokens { tgh: Some(2), color: Some(Colors::NONE), ..Tokens::new(1, 3) })?;
        lose_life(g, p, 3, Some(p), "other", None)?;
    }
    Ok(())
}

/// Thopter at the beginning of combat while you control your commander
fn apprentice(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner == p && commander_out(g, p) && trigger_window(g, p, Some(src), "create a 1/1 Thopter", None)? {
        make_tokens(g, p, Tokens { fly: true, sick: false, color: Some(Colors::NONE), ..Tokens::new(1, 1) })?;
    }
    Ok(())
}

/// the land Legion's Landing transforms into (Python's `t1.ADANTO`, a card made on the fly: cards.rs adds it)
pub const ADANTO: &str = "Adanto, the First Fort";

/// lifelink token, flips into a land with 3+ attackers; the land's token ability is not used
fn landing(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == p
        && atk.len() >= 3
        && controls(g, p, src)
        && trigger_window(g, p, Some(src), "transform", None)?
    {
        if !controls(g, p, src) {
            return Ok(vec![]);
        }
        leave(g, src)?;
        let adanto = g.db.id(ADANTO).expect("Adanto, the First Fort: an engine-made card (cards.rs)");
        g.add_land(p, adanto, false);
        crate::glog!(g, "    Legion's Landing transforms into Adanto");
    }
    Ok(vec![])
}

// ======================================================== Lathril, Blade of the Elves (and Elf support shared with Marwyn)
/// t1.ELF_WARRIOR
const ELF_WARRIOR: [Sym; 2] = ["elf", "warrior"];

/// t1.elf_tokens: n 1/1 green Elf Warriors
pub fn elf_tokens(g: &mut Game, p: PlayerId, n: u32) -> Res<Vec<PermId>> {
    make_tokens(
        g,
        p,
        Tokens { color: Some(Colors::from_letters("G")), types: ELF_WARRIOR.to_vec(), ..Tokens::new(n, 1) },
    )
}

/// menace; Elf Warrior tokens equal to combat damage
fn lathril_dmg(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, dmg: i32) -> Res {
    if a == src && trigger_window(g, p, Some(src), &format!("create {dmg} Elf Warriors"), None)? {
        elf_tokens(g, p, dmg.max(0) as u32)?;
    }
    Ok(())
}

/// tap ten untapped Elves: each opponent loses 10 life
fn lathril_drain(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if x.tapped || x.sick || post.is_none() {
        return Ok(vec![]);
    }
    let elves = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| m != src && g.is_creature(m) && !g.perm(m).tapped && !g.perm(m).phased && has_type(g, m, "elf"))
        .count();
    if elves < 10 {
        return Ok(vec![]);
    }
    let lethal = g.opps(p).any(|q| g.player(q).life <= 10);
    Ok(vec![ability(if lethal { 9.0 } else { 6.0 }, "Lathril drain 10", src, lathril_go)])
}

fn lathril_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let mut es: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| m != src && g.is_creature(m) && !g.perm(m).tapped && has_type(g, m, "elf"))
        .collect();
    if es.len() < 10 || g.perm(src).tapped {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    sort_by_key(&mut es, |m| (!g.perm(m).sick, pval(g, m)));
    for &m in es.iter().take(10) {
        g.perm_mut(m).tapped = true;
    }
    if !ability_window(g, p, Some(src), "each opponent loses 10 life", Some(9.0), None)? {
        return Ok(true);
    }
    for q in g.opps(p).collect::<Vec<_>>() {
        lose_life(g, q, 10, Some(p), "drain", None)?;
    }
    gain(g, p, 10)?;
    crate::glog!(g, "  Lathril taps ten Elves: each opponent loses 10");
    check_state(g)?;
    Ok(true)
}

/// Priest of Titania and Wirewood Channeler: G per Elf on the battlefield
fn mana_elves_all(g: &Game, p: PlayerId, _m: PermId) -> u32 {
    count_type(g, p, "elf", true) as u32
}

/// Elvish Archdruid: G per Elf you control. Not registered: galadriel.py's DYN_MANA entry for the card replaces this
/// one in Python.
#[allow(dead_code)]
fn mana_elves_mine(g: &Game, p: PlayerId, _m: PermId) -> u32 {
    count_type(g, p, "elf", false) as u32
}

/// Llanowar Tribe and Canopy Tactician: GGG
fn mana_three(_g: &Game, _p: PlayerId, _m: PermId) -> u32 {
    3
}

/// an untapped Elf that isn't a mana creature (Heritage Druid taps these)
fn heritage_elf(g: &Game, x: PermId) -> bool {
    let y = g.perm(x);
    g.is_creature(x) && !y.tapped && has_type(g, x, "elf") && !y.cd.is_some_and(|c| g.db.get(c).tag(Tag::Dork))
}

/// t1._heritage_amt: GGG per three untapped non-mana Elves (summoning-sick ones included, as on the card)
fn heritage_amt(g: &Game, p: PlayerId, _m: PermId) -> u32 {
    let n = g.player(p).perms.iter().filter(|&&x| heritage_elf(g, x) && !g.perm(x).phased).count();
    if n >= 3 { 3 * (n / 3) as u32 } else { 0 }
}

/// t1._heritage_tap: Heritage Druid itself isn't tapped; three other Elves are per GGG used
fn heritage_tap(g: &mut Game, p: PlayerId, m: PermId, used: u32) -> Res {
    g.perm_mut(m).tapped = false;
    let mut elves: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&x| x != m && heritage_elf(g, x)).collect();
    sort_by_key(&mut elves, |x| (!g.perm(x).sick, pval(g, x)));
    for &x in elves.iter().take(3 * (used as usize).div_ceil(3)) {
        g.perm_mut(x).tapped = true;
    }
    Ok(())
}

/// an Elf Warrior if you control another Elf
fn dwynen(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src
        && g.player(p).perms.iter().any(|&x| x != src && has_type(g, x, "elf"))
        && trigger_window(g, p, Some(src), "create a 1/1 Elf Warrior", None)?
    {
        elf_tokens(g, p, 1)?;
    }
    Ok(())
}

/// Elf Warrior once a turn when Elves enter
fn warmaster(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    let key = intern(&format!("warm{}", src.0));
    if m != src && g.perm(m).owner == o && has_type(g, m, "elf") && fresh(g, o, key) {
        let ok = trigger_window(g, o, Some(src), "create a 1/1 Elf Warrior", None)?;
        once_per_turn(g, o, key);
        if ok {
            elf_tokens(g, o, 1)?;
        }
    }
    Ok(())
}

/// creature, Elf, not summoning sick, untapped
fn ready_elf(g: &Game, m: PermId) -> bool {
    g.is_creature(m) && has_type(g, m, "elf") && !g.perm(m).sick && !g.perm(m).tapped
}

/// pump before combat with a wide board: Elves get +2/+2 and deathtouch
fn warmaster_pump(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || !can_pay(g, p, 5, "GG", false) {
        return Ok(vec![]);
    }
    let elves = g.player(p).perms.iter().filter(|&&m| ready_elf(g, m)).count();
    if elves < 5 {
        return Ok(vec![]);
    }
    Ok(vec![ability(1.0 + 0.5 * elves as f64, "Elvish Warmaster pump", src, warmaster_go)])
}

fn warmaster_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !can_pay(g, p, 5, "GG", false) {
        return Ok(false);
    }
    pay(g, p, 5, "GG", false)?;
    if !ability_window(g, p, Some(src), "Elves get +2/+2 and deathtouch", None, None)? {
        return Ok(true);
    }
    for m in g.player(p).perms.clone() {
        if g.is_creature(m) && has_type(g, m, "elf") {
            eot(g, m, 2, 2);
            add_kw(g, m, "deathtouch");
        }
    }
    Ok(true)
}

/// an Elf Warrior whenever you cast an Elf creature spell
fn lys(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if caster == g.perm(src).owner
        && g.db.get(c).creature
        && card_sub(g, c, "elf")
        && trigger_window(g, caster, Some(src), "create a 1/1 Elf Warrior", None)?
    {
        elf_tokens(g, caster, 1)?;
    }
    Ok(())
}

/// Elf lord; pays G to draw on Elf casts when it can
fn leafcrowned(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if caster == g.perm(src).owner
        && g.db.get(c).creature
        && card_sub(g, c, "elf")
        && can_pay(g, caster, 0, "G", false)
        && g.player(caster).hand.len() < 7
        && trigger_window(g, caster, Some(src), "pay {G}: draw a card", None)?
        && can_pay(g, caster, 0, "G", false)
    {
        pay(g, caster, 0, "G", false)?;
        draw(g, caster, 1, false)?;
    }
    Ok(())
}

/// an Elf Warrior whenever another nontoken Elf of yours dies
fn prowess_fair(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let o = g.perm(src).owner;
    if g.perm(m).owner == o
        && !g.perm(m).token
        && g.is_creature(m)
        && has_type(g, m, "elf")
        && m != src
        && trigger_window(g, o, Some(src), "create a 1/1 Elf Warrior", None)?
    {
        elf_tokens(g, o, 1)?;
    }
    Ok(())
}

/// dies: three Elf Warriors, mill 3, drain 2
fn cullers(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    let p = g.perm(m).owner;
    if g.player(p).alive && trigger_window(g, p, Some(m), "three Elf Warriors, mill 3, drain 2", None)? {
        elf_tokens(g, p, 3)?;
        mill(g, p, 3)?;
        for q in g.opps(p).collect::<Vec<_>>() {
            lose_life(g, q, 2, Some(p), "drain", None)?;
        }
        gain(g, p, 2)?;
    }
    Ok(())
}

/// t1.look_take: look at the top n cards, take up to k matching (best first) to `to` ('hand', 'land_bf' or 'bf'),
/// the rest to the bottom in a random order
pub fn look_take(
    g: &mut Game,
    p: PlayerId,
    n: usize,
    pred: &dyn Fn(&Game, CardId) -> bool,
    to: &str,
    k: usize,
) -> Res<Vec<CardId>> {
    let n = n.min(g.player(p).library.len());
    let mut top: Vec<CardId> = (0..n).map(|_| g.player_mut(p).library.pop().unwrap()).collect();
    let mut hits: Vec<CardId> = top.iter().copied().filter(|&c| pred(g, c)).collect();
    sort_by_key(&mut hits, |c| -card_worth(g, p, c, false));
    hits.truncate(k);
    for &c in &hits {
        remove_first(&mut top, c);
        match to {
            "hand" => {
                let pl = g.player_mut(p);
                pl.hand.push(c);
                pl.seen.insert(c);
            }
            "land_bf" => {
                g.add_land(p, c, true);
                landfall(g, p)?;
            }
            "bf" => {
                enter(g, p, c, Enter::default())?;
            }
            _ => {}
        }
    }
    g.rng.shuffle(&mut top);
    g.player_mut(p).library.splice(0..0, top);
    Ok(hits)
}

/// menace; top five: an Elf/Warrior/Tyvar to hand
fn harald(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, p, Some(src), "look at the top five", None)? {
        let pred = |g: &Game, c: CardId| {
            card_sub(g, c, "elf") || card_sub(g, c, "warrior") || g.db.get(c).name.starts_with("Tyvar")
        };
        look_take(g, p, 5, &pred, "hand", 1)?;
    }
    Ok(())
}

/// reveal the top four, take the Elves
fn messenger(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, p, Some(src), "reveal the top four, take the Elves", None)? {
        look_take(g, p, 4, &|g, c| card_sub(g, c, "elf"), "hand", 4)?;
    }
    Ok(())
}

/// look at the top five, put a land onto the battlefield
fn rejuvenator(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, p, Some(src), "look at the top five, put a land onto the battlefield", None)? {
        look_take(g, p, 5, &|g, c| g.db.get(c).land, "land_bf", 1)?;
    }
    Ok(())
}

/// target opponent loses 1 life per Elf you control
fn shaman_pack(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src
        && g.opps(p).next().is_some()
        && trigger_window(g, p, Some(src), "target opponent loses 1 life per Elf", None)?
        && g.opps(p).next().is_some()
    {
        let opps: Vec<PlayerId> = g.opps(p).collect();
        let q = max_by(&opps, |o| threat(g, p, o)).unwrap();
        let n = count_type(g, p, "elf", false);
        lose_life(g, q, n, Some(p), "drain", None)?;
    }
    Ok(())
}

/// draw a card and lose 1 life whenever another nontoken Elf or Berserker of yours dies
fn skemfar(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let o = g.perm(src).owner;
    if g.perm(m).owner == o
        && m != src
        && !g.perm(m).token
        && (has_type(g, m, "elf") || has_type(g, m, "berserker"))
        && trigger_window(g, o, Some(src), "draw a card, lose 1 life", None)?
    {
        draw(g, o, 1, false)?;
        lose_life(g, o, 1, Some(o), "other", None)?;
    }
    Ok(())
}

/// p's creatures that could attack this turn, other than src
fn timberwatch_targets(g: &Game, src: PermId, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            g.is_creature(m) && !x.tapped && !x.sick && m != src && !x.noatk
        })
        .collect()
}

/// pumps its best attacker before combat
fn timberwatch(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || g.perm(src).tapped || g.perm(src).sick {
        return Ok(vec![]);
    }
    if timberwatch_targets(g, src, p).is_empty() {
        return Ok(vec![]);
    }
    let x = count_type(g, p, "elf", true);
    Ok(vec![ability(1.0 + 0.3 * x as f64, "Timberwatch Elf pump", src, timberwatch_go)])
}

fn timberwatch_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let atk = timberwatch_targets(g, src, p);
    let x = count_type(g, p, "elf", true);
    if g.perm(src).tapped {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    let Some(t) = first_max(&atk, |m| (g.perm(m).fly || g.perm(m).is_cmd, epow(g, m))) else { return Ok(true) };
    let name = format!("{} gets +{x}/+{x}", g.perm(t).name);
    if ability_window(g, p, Some(src), &name, None, Some(t))? && controls(g, p, t) {
        eot(g, t, x, x);
    }
    Ok(true)
}

/// attacking Elves gain deathtouch; the +1/+1 counters on mana creatures are not modeled
fn tyvar(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == p
        && atk.iter().any(|&m| has_type(g, m, "elf"))
        && trigger_window(g, p, Some(src), "attacking Elves gain deathtouch", None)?
    {
        for &m in atk {
            if has_type(g, m, "elf") {
                add_kw(g, m, "deathtouch");
            }
        }
    }
    Ok(vec![])
}

/// overrun before combat with 4+ Elves; regeneration not modeled
/// Not registered: in Python a later module's hook for the same card and event replaces this one (rules.py's options), so it never runs.
#[allow(dead_code)]
fn ezuri(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || !can_pay(g, p, 2, "GGG", false) {
        return Ok(vec![]);
    }
    let elves = g.player(p).perms.iter().filter(|&&m| ready_elf(g, m)).count();
    if elves < 4 {
        return Ok(vec![]);
    }
    Ok(vec![ability(1.0 + 0.8 * elves as f64, "Ezuri overrun", src, ezuri_go)])
}

#[allow(dead_code)]
fn ezuri_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !can_pay(g, p, 2, "GGG", false) {
        return Ok(false);
    }
    pay(g, p, 2, "GGG", false)?;
    if !ability_window(g, p, Some(src), "Elves get +3/+3 and trample", Some(6.0), None)? {
        return Ok(true);
    }
    for m in g.player(p).perms.clone() {
        if g.is_creature(m) && has_type(g, m, "elf") {
            eot(g, m, 3, 3);
        }
    }
    g.player_mut(p).trample = true;
    Ok(true)
}

/// uncounterable; Elves become 5/5 before combat; other green spells still counterable
fn allosaurus(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || !can_pay(g, p, 4, "GG", false) {
        return Ok(vec![]);
    }
    let elves = g.player(p).perms.iter().filter(|&&m| ready_elf(g, m) && epow(g, m) < 5).count();
    if elves < 4 {
        return Ok(vec![]);
    }
    Ok(vec![ability(1.0 + 0.6 * elves as f64, "Allosaurus Shepherd 5/5s", src, allosaurus_go)])
}

fn allosaurus_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !can_pay(g, p, 4, "GG", false) {
        return Ok(false);
    }
    pay(g, p, 4, "GG", false)?;
    if !ability_window(g, p, Some(src), "Elves become 5/5", None, None)? {
        return Ok(true);
    }
    for m in g.player(p).perms.clone() {
        if g.is_creature(m) && has_type(g, m, "elf") {
            let (dp, dt) = ((5 - epow(g, m)).max(0), (5 - etgh(g, m)).max(0));
            eot(g, m, dp, dt);
        }
    }
    Ok(true)
}

/// quest counter per attacker; +5/+5 at seven (the anthem is the ability language's bma_anthem)
fn bma(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == p && trigger_window(g, p, Some(src), &format!("{} quest counter(s)", atk.len()), None)? {
        g.perm_mut(src).plus += atk.len() as i32;
    }
    Ok(vec![])
}

/// a Saproling every upkeep (anyone's: Python doesn't check whose); the city's blessing bonus is applied when the
/// token is made
fn tendershoot(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    let o = g.perm(src).owner;
    if !trigger_window(g, o, Some(src), "create a Saproling", None)? {
        return Ok(());
    }
    let city = g.player(o).perms.len() + g.player(o).lands.len() >= 10;
    let spec = Tokens {
        types: vec!["saproling"],
        color: Some(Colors::from_letters("G")),
        ..Tokens::new(1, if city { 3 } else { 1 })
    };
    make_tokens(g, o, spec)?;
    Ok(())
}

/// draw when a nontoken creature with a new name enters under your control
fn guardian(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    let Some(cd) = g.perm(m).cd else { return Ok(()) };
    if g.perm(m).owner == o
        && g.is_creature(m)
        && !g.perm(m).token
        && !g.player(o).perms.iter().any(|&x| x != m && g.perm(x).cd == Some(cd))
        && !g.player(o).gy.contains(&cd)
        && trigger_window(g, o, Some(src), "draw a card", None)?
    {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

/// random discard each upkeep
fn nath_up(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner != p || g.opps(p).next().is_none() {
        return Ok(());
    }
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let q = first_max(&opps, |o| (!g.player(o).hand.is_empty(), threat(g, p, o))).unwrap();
    if !g.player(q).hand.is_empty()
        && trigger_window(g, p, Some(src), "target opponent discards at random", None)?
        && !g.player(q).hand.is_empty()
    {
        let i = g.rng.below(g.player(q).hand.len() as u64) as usize;
        discard_index(g, q, i)?;
    }
    Ok(())
}

/// an Elf Warrior with deathtouch whenever an opponent discards
fn nath_disc(g: &mut Game, src: Src, q: PlayerId, _c: CardId) -> Res {
    let o = g.perm(src).owner;
    if q != o && trigger_window(g, o, Some(src), "create a 1/1 deathtouch Elf Warrior", None)? {
        let spec = Tokens {
            dt: true,
            color: Some(Colors::from_letters("G")),
            types: ELF_WARRIOR.to_vec(),
            ..Tokens::new(1, 1)
        };
        make_tokens(g, o, spec)?;
    }
    Ok(())
}

/// dies: return an Elf card from the graveyard to hand
fn ritualist(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    let p = g.perm(m).owner;
    let own = g.perm(m).cd;
    let cs = |g: &Game| -> Vec<CardId> {
        g.player(p).gy.iter().copied().filter(|&c| card_sub(g, c, "elf") && Some(c) != own).collect()
    };
    if cs(g).is_empty() || !trigger_window(g, p, Some(m), "return an Elf card to hand", None)? {
        return Ok(());
    }
    let cs = cs(g);
    if let Some(c) = max_by(&cs, |c| card_worth(g, p, c, false)) {
        let pl = g.player_mut(p);
        remove_first(&mut pl.gy, c);
        pl.hand.push(c);
    }
    Ok(())
}

/// Elf to the top; taps for any colour
fn harbinger(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || !trigger_window(g, p, Some(src), "search for an Elf, put it on top", None)? {
        return Ok(());
    }
    let cs: Vec<CardId> = g.player(p).library.iter().copied().filter(|&c| card_sub(g, c, "elf")).collect();
    if let Some(c) = max_by(&cs, |c| card_worth(g, p, c, false)) {
        remove_first(&mut g.player_mut(p).library, c);
        shuffle_library(g, p);
        g.player_mut(p).library.push(c);
    }
    Ok(())
}

// ======================================================== Light-Paws, Emperor's Voice (Aura voltron)
/// t1.my_auras: p's Auras on the battlefield
fn my_auras(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).cd.is_some_and(|c| card_sub(g, c, "aura"))).collect()
}

/// the cards of p's Auras on the battlefield (Python's `{a.cd.name for a in my_auras(p)}`)
fn aura_names(g: &Game, p: PlayerId) -> Vec<CardId> {
    my_auras(g, p).iter().filter_map(|&a| g.perm(a).cd).collect()
}

/// common.auras_on: the Auras attached to m (copied here: common's port owns the Aura table)
fn auras_on(g: &Game, m: PermId) -> Vec<PermId> {
    g.auras.iter().copied().filter(|&a| g.perm(a).attached == Some(m) && g.perm(a).on_bf && !g.perm(a).phased).collect()
}

/// cast Aura -> fetch an Aura with lower or equal MV and a new name, onto Light-Paws
fn lightpaws(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    let Some(mcd) = g.perm(m).cd else { return Ok(()) };
    if m == src || g.perm(m).owner != o || !card_sub(g, mcd, "aura") || !g.last_cast_etb {
        return Ok(());
    }
    let mv = g.db.get(mcd).cmc;
    let have = aura_names(g, o);
    let cands: Vec<CardId> = g
        .player(o)
        .library
        .iter()
        .copied()
        .filter(|&c| {
            card_sub(g, c, "aura")
                && g.db.get(c).cmc <= mv
                && !have.contains(&c)
                && crate::cardcode::aura_known(g, c)
                && crate::cardcode::aura_own_fits(g, o, c, src)
        })
        .collect();
    if cands.is_empty() || !trigger_window(g, o, Some(src), "search for an Aura", None)? {
        return Ok(());
    }
    let have = aura_names(g, o);
    let cands: Vec<CardId> =
        cands.into_iter().filter(|c| g.player(o).library.contains(c) && !have.contains(c)).collect();
    if cands.is_empty() || !controls(g, o, src) {
        return Ok(());
    }
    let c = first_max(&cands, |c| (g.db.get(c).cmc, card_worth(g, o, c, false))).unwrap();
    remove_first(&mut g.player_mut(o).library, c);
    shuffle_library(g, o);
    crate::glog!(g, "    Light-Paws fetches {}", g.db.get(c).name);
    g.attach_to = Some(src); // the Aura's enters hook (common) puts it on Light-Paws
    let r = enter(g, o, c, Enter::default());
    g.attach_to = None;
    r?;
    Ok(())
}

/// {W}: return to hand, recast for W: each cast re-triggers Light-Paws for a one-mana Aura
fn ward_loop(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let lp = g.player(p).perms.iter().any(|&m| g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == LIGHT_PAWS));
    if post.is_none() || !lp || !can_pay(g, p, 0, "WW", false) {
        return Ok(vec![]);
    }
    let have = aura_names(g, p);
    if !g.player(p).library.iter().any(|&c| {
        card_sub(g, c, "aura") && g.db.get(c).cmc <= 1 && !have.contains(&c) && crate::cardcode::aura_known(g, c)
    }) {
        return Ok(vec![]);
    }
    Ok(vec![ability(5.0, "Flickering Ward recast (Light-Paws)", src, ward_go)])
}

/// {W}: return it to its owner's hand, then recast it for {W}. (Python's `leave` took the Ward off the battlefield
/// without putting the card in hand, so it was never recast and the card was lost; fixed in Rust.)
fn ward_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !controls(g, p, src) || !can_pay(g, p, 0, "WW", false) {
        return Ok(false);
    }
    pay(g, p, 0, "W", false)?;
    if !ability_window(g, p, Some(src), "return to hand", None, None)? || !controls(g, p, src) {
        return Ok(true);
    }
    leave(g, src)?;
    crate::engine::zones::to_zone_card(g, src, crate::engine::zones::Zone::Hand);
    let c = g.perm(src).cd.unwrap();
    pay(g, p, 0, "W", false)?;
    if g.player(p).hand.contains(&c) {
        cast_card(g, p, c, "hand", crate::state::Ctx::default())?;
    }
    Ok(true)
}

const LIGHT_PAWS: &str = "Light-Paws, Emperor's Voice";

/// draw whenever you cast an Aura, Equipment or Vehicle spell
fn sram(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if caster == g.perm(src).owner
        && (card_sub(g, c, "aura") || card_sub(g, c, "equipment") || card_sub(g, c, "vehicle"))
        && trigger_window(g, caster, Some(src), "draw a card", None)?
    {
        draw(g, caster, 1, false)?;
    }
    Ok(())
}

/// draw on Aura casts
fn spiritdancer(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if caster == g.perm(src).owner
        && card_sub(g, c, "aura")
        && trigger_window(g, caster, Some(src), "draw a card", None)?
    {
        draw(g, caster, 1, false)?;
    }
    Ok(())
}

/// a creature with its own power/toughness rule entered: turn the rule on (runs at once)
fn selfpt_on(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.selfpt = true;
    }
    Ok(())
}

/// as a creature with its own size rule enters, before the state check sees a 0/0 (Eidolon of Countless Battles):
/// Python's `AS_ENTERS` for Korlash and Sisay. (Python turned Eidolon's rule on only in its enters hook, so Eidolon
/// cast alone died; fixed in Rust.)
fn selfpt_as_enters(g: &mut Game, _p: PlayerId, _m: PermId) -> Res {
    g.selfpt = true;
    Ok(())
}

/// Kor Spiritdancer: +2/+2 per Aura on it
fn spiritdancer_pt(g: &Game, _p: PlayerId, m: PermId) -> (i32, i32) {
    let k = if g.auras.is_empty() { 0 } else { 2 * auras_on(g, m).len() as i32 };
    (k, k)
}

/// Eidolon of Countless Battles: cast as a creature with its +1/+1 per creature and Aura; bestow not modeled
fn eidolon_pt(g: &Game, p: PlayerId, _m: PermId) -> (i32, i32) {
    let pl = g.player(p);
    let k = pl.perms.iter().filter(|&&x| g.is_creature(x)).count() + my_auras(g, p).len();
    (k as i32, k as i32)
}

/// constellation: 2/2 flying lifelink Pegasus
fn archon_sg(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if g.perm(m).owner == o
        && g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::ENCHANTMENT))
        && trigger_window(g, o, Some(src), "create a 2/2 flying Pegasus", None)?
    {
        let spec = Tokens {
            fly: true,
            lifelink: true,
            color: Some(Colors::from_letters("W")),
            types: vec!["pegasus"],
            ..Tokens::new(1, 2)
        };
        make_tokens(g, o, spec)?;
    }
    Ok(())
}

/// search for an Aura
fn pilgrim(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, p, Some(src), "search for an Aura", None)? {
        tutor_named(g, p, &|g, c| card_sub(g, c, "aura"), 1, "hand")?;
    }
    Ok(())
}

/// common.TUTOR_PRED: what the creature tutors can find (tutors follow chains: Recruiter -> Spellseeker -> Reversal).
/// Copied here for tutor_named's chain rank: the table is filled by common.py (_tutor_etb) and t5.py.
fn tutor_pred(g: &Game, c: CardId) -> Option<fn(&Game, CardId) -> bool> {
    match &*g.db.get(c).name {
        "Spellseeker" => Some(|g, x| {
            let d = g.db.get(x);
            (d.instant || d.sorcery) && d.cmc <= 2
        }),
        "Recruiter of the Guard" => Some(|g, x| {
            let d = g.db.get(x);
            d.creature && d.tgh <= 2
        }),
        "Goblin Matron" => Some(|g, x| card_sub(g, x, "goblin")),
        // t5.py's entries
        "Trophy Mage" => Some(|g, x| {
            let d = g.db.get(x);
            d.types.has(Types::ARTIFACT) && d.cmc == 3
        }),
        "Tribute Mage" => Some(|g, x| {
            let d = g.db.get(x);
            d.types.has(Types::ARTIFACT) && d.cmc == 2
        }),
        _ => None,
    }
}

/// how tutor_named ranks a card: (wished and not held, a tutor for a wished card, earlier on the wish list, not
/// held, its worth)
type TutorRank = (bool, bool, i64, bool, f64);

/// t1.tutor_named: search for up to k cards matching pred (different names), to `to` ('hand'), the deck's wish list
/// first, then a tutor that can find a wished card, then the best card not already held
pub fn tutor_named(
    g: &mut Game,
    p: PlayerId,
    pred: &dyn Fn(&Game, CardId) -> bool,
    k: usize,
    to: &str,
) -> Res<Vec<CardId>> {
    // HUMAN(phase 9): practice mode: the person picks (hc.search)
    let pl = g.player(p);
    let mut have: Vec<CardId> = pl.hand.clone();
    have.extend(pl.perms.iter().filter_map(|&m| g.perm(m).cd));
    let wish = crate::ai::plans::wish_list(g, p);
    let lib: Vec<CardId> = pl.library.iter().copied().filter(|c| wish.contains(c) && !have.contains(c)).collect();
    let chain = |c: CardId| tutor_pred(g, c).is_some_and(|f| !have.contains(&c) && lib.iter().any(|&x| f(g, x)));
    let mut names: Vec<CardId> = vec![];
    for c in searchable(g, p) {
        if pred(g, c) && !names.contains(&c) {
            names.push(c);
        }
    }
    let mut keyed: Vec<(TutorRank, CardId)> = names
        .iter()
        .map(|&c| {
            let wi = wish.iter().position(|&x| x == c);
            let held = have.contains(&c);
            ((wi.is_some() && !held, chain(c), wi.map_or(0, |i| -(i as i64)), !held, card_worth(g, p, c, false)), c)
        })
        .collect();
    keyed.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let cands: Vec<CardId> = keyed.into_iter().take(k).map(|x| x.1).collect();
    for &c in &cands {
        remove_first(&mut g.player_mut(p).library, c);
        if let Some(a) = agent_for(g, p) {
            agent_take(g, a, p, c);
            continue;
        }
        let pl = g.player_mut(p);
        if to == "hand" {
            pl.hand.push(c);
            pl.seen.insert(c);
        }
        pl.stat("tutored", 1);
    }
    shuffle_library(g, p);
    Ok(cands)
}

/// some opponent has more lands than p
fn behind_on_lands(g: &Game, p: PlayerId) -> bool {
    let n = g.player(p).lands.len();
    g.opps(p).any(|q| g.player(q).lands.len() > n)
}

/// three basic lands to hand when an opponent has more lands
fn landtax(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner == p
        && behind_on_lands(g, p)
        && trigger_window(g, p, Some(src), "search for three basic lands", None)?
    {
        for _ in 0..3 {
            land_to_hand(g, p)?;
        }
    }
    Ok(())
}

/// {W}, tap: search for a land when an opponent has more lands (and none in hand)
fn wayfarer(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.perm(src).tapped || g.perm(src).sick || !can_pay(g, p, 0, "W", false) {
        return Ok(vec![]);
    }
    if !behind_on_lands(g, p) || g.player(p).hand.iter().any(|&c| g.db.get(c).land) {
        return Ok(vec![]);
    }
    Ok(vec![ability(2.5, "Weathered Wayfarer", src, wayfarer_go)])
}

fn wayfarer_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if g.perm(src).tapped || !can_pay(g, p, 0, "W", false) {
        return Ok(false);
    }
    pay(g, p, 0, "W", false)?;
    g.perm_mut(src).tapped = true;
    if ability_window(g, p, Some(src), "search for a land", None, None)? {
        tutor_named(g, p, &|g, c| g.db.get(c).land, 1, "hand")?;
    }
    Ok(true)
}

// ======================================================== Tatyova, Benthic Druid (lands matter)
/// t1._extra(1): one additional land drop each turn (Exploration, Dryad of the Ilysian Grove, Oracle of Mul Daya,
/// Aesi)
fn extra1(g: &Game, src: Src, p: PlayerId) -> i32 {
    (g.perm(src).owner == p) as i32
}

/// t1._extra(2): two additional land drops each turn (Azusa)
fn extra2(g: &Game, src: Src, p: PlayerId) -> i32 {
    2 * (g.perm(src).owner == p) as i32
}

/// plays lands from the top of the library (Oracle of Mul Daya) / the graveyard (Crucible of Worlds, Ramunap
/// Excavator, Ancient Greenwarden)
fn own_lands_from(g: &Game, src: Src, p: PlayerId) -> i32 {
    (g.perm(src).owner == p) as i32
}

/// landfall triggers twice
fn greenwarden(g: &Game, src: Src, p: PlayerId, kind: Sym, _x: Option<PermId>) -> i32 {
    (kind == "landfall" && g.perm(src).owner == p) as i32
}

/// landfall: you may draw (the AI does with more than 15 cards in library)
fn aesi(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner == p && g.player(p).library.len() > 15 && trigger_window(g, p, Some(src), "draw a card", None)?
    {
        draw(g, p, 1, false)?;
    }
    Ok(())
}

/// whenever an opponent plays a land, put a land from your hand onto the battlefield (runs at once: land_play is not
/// a triggered event in Python)
fn burgeoning(g: &mut Game, src: Src, p: PlayerId, _c: CardId) -> Res {
    let o = g.perm(src).owner;
    if p != o
        && let Some(&x) = g.player(o).hand.iter().find(|&&x| g.db.get(x).land)
    {
        remove_first(&mut g.player_mut(o).hand, x);
        let tapped = crate::engine::turn::land_enters_tapped(g, o, x);
        g.add_land(o, x, tapped);
        landfall(g, o)?;
    }
    Ok(())
}

/// Druid Class's level (1 until it levels up)
fn druid_level(g: &Game, src: PermId) -> i64 {
    g.perm(src).data.get(DataKey::Level).map_or(1, Val::int)
}

/// landfall: gain 1 life
fn druidclass(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner == p && trigger_window(g, p, Some(src), "gain 1 life", None)? {
        gain(g, p, 1)?;
    }
    Ok(())
}

/// level 2: an additional land drop
fn druidclass_extra(g: &Game, src: Src, p: PlayerId) -> i32 {
    (g.perm(src).owner == p && druid_level(g, src) >= 2) as i32
}

/// level 2 for {2}{G}; level 3 (land creature) not modeled
/// Not registered: in Python a later module's hook for the same card and event replaces this one (partials.py's options), so it never runs.
#[allow(dead_code)]
fn druidclass_level(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || druid_level(g, src) >= 2 || !can_pay(g, p, 2, "G", false) {
        return Ok(vec![]);
    }
    Ok(vec![ability(2.0, "Druid Class level 2", src, druidclass_go)])
}

#[allow(dead_code)]
fn druidclass_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !can_pay(g, p, 2, "G", false) {
        return Ok(false);
    }
    pay(g, p, 2, "G", false)?;
    if ability_window(g, p, Some(src), "level 2", None, None)? && controls(g, p, src) {
        let mut d = PermData::default(); // Python replaces the dict: `src.data = {'level': 2}`
        d.set(DataKey::Level, Val::Int(2));
        g.perm_mut(src).data = d;
    }
    Ok(true)
}

/// 0/1 Plant per land
fn avenger(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && trigger_window(g, o, Some(src), "a 0/1 Plant per land", Some(5.0))? {
        let n = g.player(o).lands.len() as u32;
        let spec =
            Tokens { tgh: Some(1), color: Some(Colors::from_letters("G")), types: vec!["plant"], ..Tokens::new(n, 0) };
        make_tokens(g, o, spec)?;
    }
    Ok(())
}

/// a Plant token of p's
fn plant(g: &Game, m: PermId) -> bool {
    g.perm(m).token && has_type(g, m, "plant")
}

/// landfall: +1/+1 counter on each Plant
fn avenger_lf(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner == p
        && g.player(p).perms.iter().any(|&m| plant(g, m))
        && trigger_window(g, p, Some(src), "+1/+1 counter on each Plant", None)?
    {
        for m in g.player(p).perms.clone() {
            if plant(g, m) {
                g.perm_mut(m).plus += 1;
            }
        }
    }
    Ok(())
}

/// landfall: Insect, or a copy of itself with six or more lands (token cap 250)
fn scute(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner != p || !trigger_window(g, p, Some(src), "create a token", None)? {
        return Ok(());
    }
    if g.player(p).lands.len() >= 6 {
        let cd = g.perm(src).cd.unwrap();
        enter_token_copy(g, p, cd)?;
    } else {
        let spec = Tokens { color: Some(Colors::from_letters("G")), types: vec!["insect"], ..Tokens::new(1, 1) };
        make_tokens(g, p, spec)?;
    }
    Ok(())
}

/// landfall: steal the best opposing creature while it stays
fn roil(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner != p {
        return Ok(());
    }
    let cands: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && !untargetable(g, m) && !g.perm(m).is_cmd)
        .collect();
    let Some(m) = max_by(&cands, |m| pval(g, m)) else { return Ok(()) };
    if pval(g, m) < 2.0 || !trigger_window(g, p, Some(src), &format!("gain control of {}", g.perm(m).name), Some(6.0))?
    {
        return Ok(());
    }
    if !g.perm(m).on_bf || g.perm(m).owner == p || !controls(g, p, src) {
        return Ok(());
    }
    let q = g.perm(m).owner;
    let cd = g.perm(src).cd;
    // targeted: can be answered
    if crate::ai::protect_response(g, q, m, "steal", Some(p), cd)? || !g.perm(m).on_bf {
        return Ok(());
    }
    let q = g.perm(m).owner;
    g.player_mut(q).perms.retain(|&x| x != m);
    let x = g.perm_mut(m);
    x.owner = p;
    x.attached = None;
    g.player_mut(p).perms.push(m);
    g.bf_ver += 1;
    let d = &mut g.perm_mut(src).data;
    match d.get_mut(DataKey::Stolen) {
        Some(Val::List(v)) => v.push(Val::Perm(m)),
        _ => d.set(DataKey::Stolen, Val::List(vec![Val::Perm(m)])),
    }
    crate::glog!(g, "    Roil Elemental steals {}", g.perm(m).name);
    Ok(())
}

/// Roil Elemental leaves: what it stole goes back (runs at once: an effect ending)
fn roil_leaves(g: &mut Game, src: Src, _m: PermId) -> Res {
    let stolen: Vec<PermId> = match g.perm(src).data.get(DataKey::Stolen) {
        Some(Val::List(v)) => v.iter().filter_map(|x| if let Val::Perm(m) = x { Some(*m) } else { None }).collect(),
        _ => vec![],
    };
    let o = g.perm(src).owner;
    for m in stolen {
        if controls(g, o, m) {
            g.player_mut(o).perms.retain(|&x| x != m);
            let orig = g.perm(m).orig;
            g.perm_mut(m).owner = orig;
            g.player_mut(orig).perms.push(m);
            g.bf_ver += 1;
        }
    }
    Ok(())
}

/// landfall mana (spendable this turn); second landfall finds an Elf or Elemental (the other revealed cards go to
/// the bottom in the original, here they stay). Python counts landfalls in `p.flag_turn[k] = (stamp, n)`; Rust's
/// flag_turn holds stamps, so the count is two stamps: `nissa{id}` (one landfall this turn) and `nissa{id}_2` (two).
fn nissa_ra(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner != p || !trigger_window(g, p, Some(src), "add one mana", None)? {
        return Ok(());
    }
    g.player_mut(p).floating.any += 1;
    let (k1, k2) = (intern(&format!("nissa{}", src.0)), intern(&format!("nissa{}_2", src.0)));
    let st = g.turn_stamp();
    let pl = g.player_mut(p);
    let second = if pl.flag_turn.get(k1) == Some(&st) {
        let first_two = pl.flag_turn.get(k2) != Some(&st);
        pl.flag_turn.insert(k2, st);
        first_two
    } else {
        pl.flag_turn.insert(k1, st);
        false
    };
    if second {
        // reveal from the top until an Elf or Elemental: it goes to hand, the rest revealed go to the bottom in a
        // random order (Python left them on top; fixed in Rust)
        let lib = &g.player(p).library;
        let found = (0..lib.len()).rev().find(|&i| card_sub(g, lib[i], "elf") || card_sub(g, lib[i], "elemental"));
        if let Some(i) = found {
            let mut above = g.player_mut(p).library.split_off(i + 1);
            let c = g.player_mut(p).library.pop().unwrap();
            g.rng.shuffle(&mut above);
            let pl = g.player_mut(p);
            pl.hand.push(c);
            pl.library.splice(0..0, above);
        }
    }
    Ok(())
}

/// landfall: always takes the Treasure
fn provisioner(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner == p && trigger_window(g, p, Some(src), "create a Treasure", None)? {
        add_treasure(g, p, 1)?;
    }
    Ok(())
}

/// landfall: investigate
fn tracker(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner == p && trigger_window(g, p, Some(src), "investigate", None)? {
        g.player_mut(p).clues += 1;
    }
    Ok(())
}

/// sacrificing a Clue: +1/+1 counter
fn tracker_clue(g: &mut Game, src: Src, p: PlayerId, what: Sacrificed) -> Res {
    if p == g.perm(src).owner
        && what == Sacrificed::Token("Clue")
        && trigger_window(g, p, Some(src), "+1/+1 counter", None)?
    {
        g.perm_mut(src).plus += 1;
    }
    Ok(())
}

/// enters: return a land from the graveyard to the battlefield
fn titania(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src
        && g.player(o).gy.iter().any(|&c| g.db.get(c).land)
        && trigger_window(g, o, Some(src), "return a land from the graveyard", None)?
    {
        let ls: Vec<CardId> = g.player(o).gy.iter().copied().filter(|&c| g.db.get(c).land).collect();
        let cols = |c: CardId| g.db.get(c).tags.str(Tag::C).map_or(0, |s| s.chars().count());
        if let Some(c) = first_max(&ls, cols) {
            remove_first(&mut g.player_mut(o).gy, c);
            g.add_land(o, c, true);
            landfall(g, o)?;
        }
    }
    Ok(())
}

/// a 5/3 Elemental when a land goes to the graveyard (fetch lands, land sacrifices)
fn titania_gy(g: &mut Game, src: Src, p: PlayerId, _c: CardId) -> Res {
    if p == g.perm(src).owner && trigger_window(g, p, Some(src), "create a 5/3 Elemental", None)? {
        let spec = Tokens {
            tgh: Some(3),
            color: Some(Colors::from_letters("G")),
            types: vec!["elemental"],
            ..Tokens::new(1, 5)
        };
        make_tokens(g, p, spec)?;
    }
    Ok(())
}

/// reveal the top card: a land onto the battlefield, anything else to hand
fn coiling(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src
        && !g.player(o).library.is_empty()
        && trigger_window(g, o, Some(src), "reveal the top card", None)?
        && !g.player(o).library.is_empty()
    {
        let c = g.player_mut(o).library.pop().unwrap();
        if g.db.get(c).land {
            g.add_land(o, c, false);
            landfall(g, o)?;
        } else {
            g.player_mut(o).hand.push(c);
        }
    }
    Ok(())
}

/// enters: 3 life, draw, land (then sacrificed, unless it escaped)
fn uro(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let esc = g.uro_escaping;
    let o = g.perm(src).owner;
    if trigger_window(g, o, Some(src), "gain 3 life, draw, put a land", None)? {
        uro_value(g, o)?;
    }
    if esc {
        let mut d = PermData::default(); // Python: `src.data = {'escaped': True}`
        d.set(DataKey::Escaped, Val::Bool(true));
        g.perm_mut(src).data = d;
    } else {
        die(g, src, "sac")?; // the sacrifice trigger, kept as before
    }
    Ok(())
}

/// escaped Uro attacks: it repeats its enter effect
fn uro_atk(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if atk.contains(&src) && trigger_window(g, p, Some(src), "gain 3 life, draw, put a land", None)? {
        uro_value(g, p)?;
    }
    Ok(vec![])
}

/// t1._uro_value: gain 3, draw, put a land from hand onto the battlefield (the ability language's put_land)
fn uro_value(g: &mut Game, p: PlayerId) -> Res {
    gain(g, p, 3)?;
    draw(g, p, 1, false)?;
    put_land(g, p)
}

/// dsl.run(g, p, {'do': 'put_land'}): put a land card from your hand onto the battlefield (the one making the most
/// colours, a fetch land breaking ties)
fn put_land(g: &mut Game, p: PlayerId) -> Res {
    let ls: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).land).collect();
    if let Some(c) = max_by(&ls, |c| {
        let t = &g.db.get(c).tags;
        t.str(Tag::C).map_or(0, |s| s.chars().count()) as f64 * 10.0 + t.has(Tag::F) as i32 as f64
    }) {
        remove_first(&mut g.player_mut(p).hand, c);
        let tapped = crate::engine::turn::land_enters_tapped(g, p, c);
        let l: LandId = g.add_land(p, c, tapped);
        landfall(g, p)?;
        if g.db.get(c).tag(Tag::F) && g.player(p).lands.last() == Some(&l) {
            crate::engine::turn::crack_fetch(g, p, l)?;
        }
    }
    Ok(())
}

/// escape from the graveyard ({G}{G}{U}{U}, exile five other cards) as a 6/6 that repeats its enter effect on attack
/// (escaped Uro is not countered here)
fn uro_escape(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !can_pay(g, p, 0, "GGUU", false) || g.player(p).gy.len() < 6 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 6.0,
        label: "escape Uro".into(),
        act: Some(Action::Plan { f: uro_escape_go, arg: c.0 as i64 }),
    }])
}

fn uro_escape_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).gy.contains(&c) || !can_pay(g, p, 0, "GGUU", false) || g.player(p).gy.len() < 6 {
        return Ok(false);
    }
    remove_first(&mut g.player_mut(p).gy, c);
    pay(g, p, 0, "GGUU", false)?;
    let mut others = g.player(p).gy.clone();
    sort_by_key(&mut others, |x| card_worth(g, p, x, true));
    for &x in others.iter().take(5) {
        let pl = g.player_mut(p);
        remove_first(&mut pl.gy, x);
        pl.exile.push(x);
    }
    let pl = g.player_mut(p);
    pl.cast_names.insert(c);
    pl.spells_this_turn += 1;
    on_cast(g, p, c)?;
    if !counter_window(g, p, c, 6.0, vec![])? {
        g.player_mut(p).gy.push(c);
        return Ok(true);
    }
    g.uro_escaping = true;
    let r = enter(g, p, c, Enter { was_cast: true, ..Enter::default() });
    g.uro_escaping = false;
    r?;
    Ok(true)
}

/// creature spells draw
fn zr(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if caster == g.perm(src).owner && g.db.get(c).creature && trigger_window(g, caster, Some(src), "draw a card", None)?
    {
        draw(g, caster, 1, false)?;
    }
    Ok(())
}

/// lands tap for double
fn zr_mana(g: &Game, src: Src, p: PlayerId, _l: LandId) -> i32 {
    (g.perm(src).owner == p) as i32
}

// ======================================================== Teysa Karlov (aristocrats)
/// death-caused triggers of your permanents fire twice
fn teysa(g: &Game, src: Src, p: PlayerId, kind: Sym, m: Option<PermId>) -> i32 {
    (kind == "dies" && g.perm(src).owner == p && m.is_none_or(|m| g.is_creature(m))) as i32
}

/// tokens have vigilance and lifelink
fn teysa_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    (kw == "vigilance" || kw == "lifelink")
        && g.perm(m).token
        && g.is_creature(m)
        && g.perm(m).owner == g.perm(src).owner
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Isshin, Two Heavens as One")?.trigger_copies = Some(isshin);
    r.card(db, "Hellrider")?.attack = Some(hellrider);
    r.card(db, "Brutal Hordechief")?.attack = Some(hordechief);
    r.card(db, "Aurelia, the Warleader")?.attack = Some(aurelia);
    r.card(db, "Karlach, Fury of Avernus")?.attack = Some(karlach);
    r.card(db, "Scourge of the Throne")?.attack = Some(scourge);
    r.card(db, "Brimaz, King of Oreskos")?.attack = Some(brimaz);
    r.card(db, "Hanweir Garrison")?.attack = Some(garrison);
    r.card(db, "Anim Pakal, Thousandth Moon")?.attack = Some(anim_pakal);
    r.card(db, "Caesar, Legion's Emperor")?.attack = Some(caesar);
    let c = r.card(db, "Goblin Rabblemaster")?;
    c.attack = Some(rabble_attack); // its upkeep Goblin: rules2's hook (goblin_upkeep is overridden in Python)
    // Legion Warboss: rules2's upkeep hook replaces t1's (goblin_upkeep)
    r.card(db, "Tilonalli's Summoner")?.attack = Some(tilonalli);
    r.card(db, "Shared Animosity")?.attack = Some(animosity);
    r.card(db, "Moonshaker Cavalry")?.etb = Some(moonshaker);
    r.card(db, "Ogre Battledriver")?.etb = Some(battledriver);
    r.card(db, "Mentor of the Meek")?.etb = Some(mentor_meek);
    r.card(db, "Welcoming Vampire")?.etb = Some(welcoming);
    r.card(db, "Laelia, the Blade Reforged")?.attack = Some(laelia);
    r.card(db, "Outlaws' Merriment")?.upkeep = Some(merriment);
    r.card(db, "Black Market Connections")?.upkeep = Some(bmc);
    r.card(db, "Loyal Apprentice")?.upkeep = Some(apprentice);
    r.card(db, "Legion's Landing // Adanto, the First Fort")?.attack = Some(landing);
    // Lathril and the Elves
    let c = r.card(db, "Lathril, Blade of the Elves")?;
    c.combat_damage = Some(lathril_dmg);
    c.options = Some(lathril_drain);
    r.card(db, "Priest of Titania")?.dyn_mana_perm = Some(mana_elves_all);
    // Elvish Archdruid: galadriel's DYN_MANA entry replaces t1's (mana_elves_mine)
    r.card(db, "Wirewood Channeler")?.dyn_mana_perm = Some(mana_elves_all);
    r.card(db, "Llanowar Tribe")?.dyn_mana_perm = Some(mana_three);
    r.card(db, "Canopy Tactician")?.dyn_mana_perm = Some(mana_three);
    let c = r.card(db, "Heritage Druid")?;
    c.dyn_mana_perm = Some(heritage_amt);
    c.on_tap_perm = Some(heritage_tap);
    r.card(db, "Dwynen's Elite")?.etb = Some(dwynen);
    let c = r.card(db, "Elvish Warmaster")?;
    c.etb = Some(warmaster);
    c.options = Some(warmaster_pump);
    r.card(db, "Lys Alana Huntmaster")?.cast = Some(lys);
    r.card(db, "Leaf-Crowned Visionary")?.cast = Some(leafcrowned);
    r.card(db, "Prowess of the Fair")?.dies = Some(prowess_fair);
    r.card(db, "Eyeblight Cullers")?.self_dies = Some(cullers);
    r.card(db, "Harald, King of Skemfar")?.etb = Some(harald);
    r.card(db, "Sylvan Messenger")?.etb = Some(messenger);
    r.card(db, "Elvish Rejuvenator")?.etb = Some(rejuvenator);
    r.card(db, "Shaman of the Pack")?.etb = Some(shaman_pack);
    r.card(db, "Skemfar Avenger")?.dies = Some(skemfar);
    r.card(db, "Timberwatch Elf")?.options = Some(timberwatch);
    r.card(db, "Tyvar the Bellicose")?.attack = Some(tyvar);
    // Ezuri, Renegade Leader: rules' options hook replaces t1's (ezuri)
    r.card(db, "Allosaurus Shepherd")?.options = Some(allosaurus);
    r.card(db, "Beastmaster Ascension")?.attack = Some(bma);
    r.card(db, "Tendershoot Dryad")?.upkeep = Some(tendershoot);
    r.card(db, "Guardian Project")?.etb = Some(guardian);
    let c = r.card(db, "Nath of the Gilt-Leaf")?;
    c.upkeep = Some(nath_up);
    c.discard = Some(nath_disc);
    r.card(db, "Elderfang Ritualist")?.self_dies = Some(ritualist);
    r.card(db, "Elvish Harbinger")?.etb = Some(harbinger);
    // Light-Paws and the Auras
    r.card(db, LIGHT_PAWS)?.etb = Some(lightpaws);
    r.card(db, "Flickering Ward")?.options = Some(ward_loop);
    r.card(db, "Sram, Senior Edificer")?.cast = Some(sram);
    let c = r.card(db, "Kor Spiritdancer")?;
    c.cast = Some(spiritdancer);
    c.etb = Some(selfpt_on);
    *c = c.at_once(Event::Etb);
    c.self_pt = Some(spiritdancer_pt);
    let c = r.card(db, "Eidolon of Countless Battles")?;
    c.as_enters = Some(selfpt_as_enters);
    c.etb = Some(selfpt_on);
    *c = c.at_once(Event::Etb);
    c.self_pt = Some(eidolon_pt);
    r.card(db, "Archon of Sun's Grace")?.etb = Some(archon_sg);
    r.card(db, "Heliod's Pilgrim")?.etb = Some(pilgrim);
    r.card(db, "Land Tax")?.upkeep = Some(landtax);
    r.card(db, "Weathered Wayfarer")?.options = Some(wayfarer);
    // Tatyova and the lands
    for name in ["Exploration", "Dryad of the Ilysian Grove", "Oracle of Mul Daya", "Aesi, Tyrant of Gyre Strait"] {
        r.card(db, name)?.extra_lands = Some(extra1);
    }
    r.card(db, "Azusa, Lost but Seeking")?.extra_lands = Some(extra2);
    r.card(db, "Oracle of Mul Daya")?.lands_from_top = Some(own_lands_from);
    for name in ["Crucible of Worlds", "Ramunap Excavator", "Ancient Greenwarden"] {
        r.card(db, name)?.lands_from_gy = Some(own_lands_from);
    }
    r.card(db, "Ancient Greenwarden")?.trigger_copies = Some(greenwarden);
    r.card(db, "Aesi, Tyrant of Gyre Strait")?.landfall = Some(aesi);
    let c = r.card(db, "Burgeoning")?;
    c.land_play = Some(burgeoning);
    *c = c.at_once(Event::LandPlay);
    let c = r.card(db, "Druid Class")?;
    c.landfall = Some(druidclass);
    c.extra_lands = Some(druidclass_extra);
    // its options: partials' hook replaces t1's (druidclass_level)
    let c = r.card(db, "Avenger of Zendikar")?;
    c.etb = Some(avenger);
    c.landfall = Some(avenger_lf);
    r.card(db, "Scute Swarm")?.landfall = Some(scute);
    let c = r.card(db, "Roil Elemental")?;
    c.landfall = Some(roil);
    c.leaves = Some(roil_leaves);
    *c = c.at_once(Event::Leaves);
    r.card(db, "Nissa, Resurgent Animist")?.landfall = Some(nissa_ra);
    r.card(db, "Tireless Provisioner")?.landfall = Some(provisioner);
    let c = r.card(db, "Tireless Tracker")?;
    c.landfall = Some(tracker);
    c.sacrifice = Some(tracker_clue);
    let c = r.card(db, "Titania, Protector of Argoth")?;
    c.etb = Some(titania);
    c.land_gy = Some(titania_gy);
    r.card(db, "Coiling Oracle")?.etb = Some(coiling);
    let c = r.card(db, "Uro, Titan of Nature's Wrath")?;
    c.etb = Some(uro);
    c.attack = Some(uro_atk);
    c.gy_options = Some(uro_escape);
    let c = r.card(db, "Zendikar Resurgent")?;
    c.cast = Some(zr);
    c.land_mana = Some(zr_mana);
    // Teysa
    let c = r.card(db, "Teysa Karlov")?;
    c.trigger_copies = Some(teysa);
    c.grant_kw = Some(teysa_kw);
    Ok(())
}
