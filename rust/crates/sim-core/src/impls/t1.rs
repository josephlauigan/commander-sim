//! Python's `cards/impl/t1.py`: the Tier 1 pool decks (Isshin, Lathril, Light-Paws, Tatyova, Teysa).

use crate::cards::{CardDb, Colors};
use crate::engine::life::{gain, lose_life};
use crate::engine::stack::trigger_window;
use crate::engine::zones::{Tokens, make_tokens};
use crate::flow::Res;
use crate::hooks::{Registry, Src};
use crate::ids::{PermId, PlayerId};
use crate::state::Game;
use crate::sym::{Sym, intern};

// ------------------------------------------------------------------ helpers
/// the once-per-turn key of src's first attack (Python's `f'first_attack_{id(src)}'`)
fn first_attack_key(src: PermId) -> Sym {
    intern(&format!("first_attack_{}", src.0))
}

/// t1.first_attack: mark src's first attack this turn
fn first_attack(g: &mut Game, src: Src) -> bool {
    let p = g.perm(src).owner;
    crate::engine::values::once_per_turn(g, p, first_attack_key(src))
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
            g.perm_mut(m).eot_kw.push("first strike");
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

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Isshin, Two Heavens as One")?.trigger_copies = Some(isshin);
    r.card(db, "Hellrider")?.attack = Some(hellrider);
    r.card(db, "Brutal Hordechief")?.attack = Some(hordechief);
    r.card(db, "Aurelia, the Warleader")?.attack = Some(aurelia);
    r.card(db, "Karlach, Fury of Avernus")?.attack = Some(karlach);
    r.card(db, "Scourge of the Throne")?.attack = Some(scourge);
    r.card(db, "Brimaz, King of Oreskos")?.attack = Some(brimaz);
    r.card(db, "Hanweir Garrison")?.attack = Some(garrison);
    Ok(())
}
