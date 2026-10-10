//! Python's `cards/impl/t4.py`: the Tier 4 pool decks (Yuriko, Krenko, Chulane, Prosper, Heliod; Grand Arbiter
//! Augustin IV's control staples are kept for the lists that still run them).
//!
//! The counterspells' and other cards' tags (`card(...)`) are in data/cards.json already. Three of t4.py's hooks are
//! replaced in Python by rules2.py's for the same card and event, so they never run and aren't ported: Rionya, Fire
//! Dancer's combat_start, Valakut Exploration's landfall and end_step, and Purphoros, God of the Forge's etb.

use crate::cards::{CardDb, Colors};
use crate::engine::cast::{cast_card, discard_worst};
use crate::engine::hooks::hooked;
use crate::engine::life::{gain, lose_life};
use crate::engine::mana::{can_pay, cost_of, pay};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::tutors::card_worth;
use crate::engine::values::{epow, etgh, has_type, once_per_turn, pval, threat, untargetable};
use crate::engine::zones::{
    Enter, Tokens, bounce, die, draw, enter, enter_token_copy, leave, make_artifact_tokens, make_tokens, max_by, min_by,
};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Sacrificed, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::common::{AURA, AuraSpec, WalkerAb, best_opp_creature, best_opp_nonland, walker};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

// ------------------------------------------------------------------ helpers
/// `m in p.perms`
fn controls(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.perm(m).on_bf && g.perm(m).owner == p
}

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

/// the card name of a permanent ("" for a token)
fn card_name(g: &Game, m: PermId) -> &str {
    g.perm(m).cd.map_or("", |c| &g.db.get(c).name)
}

fn name(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn pname(g: &Game, p: PlayerId) -> &'static str {
    g.player(p).name
}

fn opps(g: &Game, p: PlayerId) -> Vec<PlayerId> {
    g.opps(p).collect()
}

/// p has a permanent with this card name (phased out ones too, as Python's `m.cd.name == x for m in p.perms`)
fn has_named(g: &Game, p: PlayerId, n: &str) -> bool {
    g.player(p).perms.iter().any(|&m| card_name(g, m) == n)
}

/// t1.fresh: once_per_turn(g, p, key) would pass; sets nothing (trigger guards run twice: probe, then real)
fn fresh(g: &Game, p: PlayerId, key: Sym) -> bool {
    g.player(p).flag_turn.get(key) != Some(&g.turn_stamp())
}

/// cardimpl._eot: +dp/+dt until end of turn
fn eot(g: &mut Game, m: PermId, dp: i32, dt: i32) {
    let x = &mut g.perm_mut(m).eot_pt;
    *x = (x.0 + dp, x.1 + dt);
}

/// remove the first copy of c from a zone (Python's `list.remove`)
fn remove_first(v: &mut Vec<CardId>, c: CardId) {
    if let Some(i) = v.iter().position(|&x| x == c) {
        v.remove(i);
    }
}

/// `c.bomb or c.pow`
fn bomb_or_pow(g: &Game, c: CardId) -> i32 {
    let d = g.db.get(c);
    if d.bomb != 0 { d.bomb } else { d.pow }
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

/// the first item with the lowest key, for keys that are tuples (Python's `min(xs, key=...)`)
fn first_min<T: Copy, K: PartialOrd>(xs: &[T], key: impl Fn(T) -> K) -> Option<T> {
    let mut best: Option<(T, K)> = None;
    for &x in xs {
        let k = key(x);
        if best.as_ref().is_none_or(|b| k < b.1) {
            best = Some((x, k));
        }
    }
    best.map(|b| b.0)
}

/// a stable ascending sort by a key (Python's `sorted(xs, key=...)`)
fn sort_by_key<T: Copy, K: PartialOrd>(xs: &mut [T], key: impl Fn(T) -> K) {
    let mut keyed: Vec<(K, T)> = xs.iter().map(|&x| (key(x), x)).collect();
    keyed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    for (i, (_, x)) in keyed.into_iter().enumerate() {
        xs[i] = x;
    }
}

/// each opponent of p loses n life (a trigger's damage)
fn each_opp_loses(g: &mut Game, p: PlayerId, n: i32) -> Res {
    for q in opps(g, p) {
        lose_life(g, q, n, Some(p), "triggers", None)?;
    }
    Ok(())
}

/// an activated ability of src for the AI's option list
fn ability(utility: f64, label: String, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Ability { src, f, arg }) }
}

/// t2.best_target_any: 'deals n damage to any target': kill the best creature it kills, else the most threatening
/// opponent's face. MERGE: a private copy until t2.rs lands; then call `super::t2::best_target_any`.
fn best_target_any(g: &mut Game, p: PlayerId, n: i32) -> Res {
    let os = opps(g, p);
    if os.is_empty() {
        return Ok(());
    }
    // HUMAN(phase 9): hc.deal_damage (the person picks the target)
    if let Some(&q) = os.iter().find(|&&q| g.player(q).life <= n) {
        return lose_life(g, q, n, Some(p), "triggers", None);
    }
    let tg: Vec<PermId> = os
        .iter()
        .flat_map(|&q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && !untargetable(g, m) && etgh(g, m) <= n)
        .collect();
    let best = max_by(&tg, |m| pval(g, m));
    match best {
        Some(b) if pval(g, b) >= 3.0 => apply_removal(g, Some(p), b, &format!("dmg{n}"), None),
        _ => {
            let q = max_by(&os, |q| threat(g, p, q)).unwrap();
            lose_life(g, q, n, Some(p), "triggers", None)
        }
    }
}

/// t3.sac_worst_permanent: sacrifice the least valuable permanent (Food / Clue / Treasure first). MERGE: a private
/// copy until t3's is ported; then call it.
fn sac_worst_permanent(g: &mut Game, p: PlayerId, exclude: Option<PermId>) -> Res {
    use crate::engine::hooks::fire_trigger;
    use crate::hooks::Call;
    if g.player(p).foods > 0 {
        crate::impls::common::sac_food(g, p, 1)?;
        return Ok(());
    }
    for (kind, n) in [("Clue", g.player(p).clues), ("Treasure", g.player(p).treasures)] {
        if n > 0 {
            let pl = g.player_mut(p);
            if kind == "Clue" {
                pl.clues -= 1;
            } else {
                pl.treasures -= 1;
            }
            if !g.hooks.is_empty() {
                fire_trigger(g, Event::Sacrifice, Call::Sacrifice { p, what: Sacrificed::Token(kind) })?;
            }
            return Ok(());
        }
    }
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| Some(m) != exclude && !g.perm(m).is_cmd && !g.perm(m).phased)
        .collect();
    if let Some(m) = min_by(&cands, |m| pval(g, m)) {
        return die(g, m, "sac");
    }
    let ls = g.player(p).lands.clone();
    if let Some(l) = first_min(&ls, |l| {
        let x = g.land(l);
        (!x.tapped, g.db.get(x.cd).tags.str(Tag::C).map_or(0, |s| s.chars().count()))
    }) {
        let cd = g.land(l).cd;
        g.player_mut(p).lands.retain(|&x| x != l);
        g.land_mut(l).on_bf = false;
        g.player_mut(p).gy.push(cd);
        if !g.hooks.is_empty() {
            fire_trigger(g, Event::Sacrifice, Call::Sacrifice { p, what: Sacrificed::Card(cd) })?;
            fire_trigger(g, Event::LandGy, Call::Cards { p, cards: vec![cd] })?;
        }
    }
    Ok(())
}

/// rules2.devotion: p's devotion to a colour (pips among p's permanents, phased out ones excepted). MERGE: a private
/// copy until rules2's is ported; then call it.
fn devotion(g: &Game, p: PlayerId, col: char) -> usize {
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| !g.perm(m).phased)
        .filter_map(|&m| g.perm(m).cd)
        .map(|c| g.db.get(c).pips.chars().filter(|&x| x == col).count())
        .sum()
}

// ======================================================== Ninjutsu and Yuriko, the Tiger's Shadow
/// t4.NINJUTSU: each Ninja's ninjutsu cost (generic, pips)
const NINJUTSU: [(&str, u32, &str); 12] = [
    ("Yuriko, the Tiger's Shadow", 0, "UB"),
    ("Ink-Eyes, Servant of Oni", 3, "BB"),
    ("Fallen Shinobi", 2, "UB"),
    ("Higure, the Still Wind", 2, "UU"),
    ("Ingenious Infiltrator", 0, "UB"),
    ("Mist-Syndicate Naga", 2, "U"),
    ("Mistblade Shinobi", 0, "U"),
    ("Moon-Circuit Hacker", 0, "U"),
    ("Prosperous Thief", 1, "U"),
    ("Silent-Blade Oni", 4, "UB"),
    ("Silver-Fur Master", 0, "UB"),
    ("Thousand-Faced Shadow", 2, "UU"),
];

fn ninjutsu_entry(n: &str) -> Option<(u32, &'static str)> {
    NINJUTSU.iter().find(|x| x.0 == n).map(|x| (x.1, x.2))
}

/// `name in NINJUTSU`
pub fn has_ninjutsu(n: &str) -> bool {
    ninjutsu_entry(n).is_some()
}

/// t4.ninjutsu_cost: the card's ninjutsu cost ({2}{U}{B} for one Satoru grants), {1} less per Silver-Fur Master
pub fn ninjutsu_cost(g: &Game, p: PlayerId, n: &str) -> (u32, &'static str) {
    let (gen_, pips) = ninjutsu_entry(n).unwrap_or((2, "UB")); // Satoru grants {2}{U}{B}
    let disc =
        g.player(p).perms.iter().filter(|&&m| card_name(g, m) == "Silver-Fur Master" && !g.perm(m).phased).count();
    (gen_.saturating_sub(disc as u32), pips)
}

/// pool_ai.combat_reserve's costs: the ninjutsu costs of the Ninjas in p's hand
pub fn ninjutsu_costs(g: &Game, p: PlayerId) -> Vec<(u32, String)> {
    g.player(p)
        .hand
        .iter()
        .filter(|&&c| has_ninjutsu(name(g, c)))
        .map(|&c| {
            let (n, pips) = ninjutsu_cost(g, p, name(g, c));
            (n, pips.to_string())
        })
        .collect()
}

/// t4.ninjutsu: after blockers: return an unblocked attacker to hand and put a Ninja onto the battlefield tapped and
/// attacking (up to three times)
pub fn ninjutsu(g: &mut Game, p: PlayerId, atk: &mut Vec<PermId>, _d: PlayerId, assign: &[(PermId, PermId)]) -> Res {
    for _ in 0..3 {
        let unbl: Vec<PermId> = atk
            .iter()
            .copied()
            .filter(|&a| !assign.iter().any(|x| x.0 == a) && controls(g, p, a) && !g.perm(a).is_cmd)
            .collect();
        if unbl.is_empty() {
            return Ok(());
        }
        let satoru = g
            .player(p)
            .perms
            .iter()
            .any(|&m| matches!(card_name(g, m), "Satoru Umezawa, Mirror of the Ninja" | "Satoru Umezawa"));
        // (card, is it Yuriko from the command zone)
        let mut cands: Vec<(CardId, bool)> = g
            .player(p)
            .hand
            .iter()
            .filter(|&&c| has_ninjutsu(name(g, c)) || (satoru && g.db.get(c).creature))
            .map(|&c| (c, false))
            .collect();
        let pl = g.player(p);
        if name(g, pl.cmd) == "Yuriko, the Tiger's Shadow" && pl.cmd_in_zone {
            cands.push((pl.cmd, true));
        }
        let mut best: Option<(i32, CardId, bool, u32, &'static str)> = None;
        for (c, yuri) in cands {
            let n = name(g, c);
            let (mut gen_, mut pips) = ninjutsu_cost(g, p, n);
            if yuri {
                (gen_, pips) = (0, "UB");
            }
            if !has_ninjutsu(n) && satoru {
                (gen_, pips) = (2, "UB");
            }
            if !can_pay(g, p, gen_, pips, false) {
                continue;
            }
            let v = bomb_or_pow(g, c) + if yuri { 6 } else { 0 } + if has_ninjutsu(n) { 3 } else { 0 };
            if best.is_none_or(|b| v > b.0) {
                best = Some((v, c, yuri, gen_, pips));
            }
        }
        let Some((_, c, yuri, gen_, pips)) = best else { return Ok(()) };
        let a = first_min(&unbl, |m| (pval(g, m), -epow(g, m))).unwrap();
        if pval(g, a) > (bomb_or_pow(g, c) + 3) as f64 && !yuri {
            return Ok(());
        }
        pay(g, p, gen_, pips, false)?;
        atk.retain(|&x| x != a);
        if g.perm(a).token {
            leave(g, a)?;
        } else {
            bounce(g, a)?;
        }
        if yuri {
            g.player_mut(p).cmd_in_zone = false;
        } else {
            remove_first(&mut g.player_mut(p).hand, c);
        }
        let m = enter(g, p, c, Enter::default())?;
        let x = g.perm_mut(m);
        x.tapped = true;
        x.sick = false;
        if c == g.player(p).cmd {
            g.perm_mut(m).is_cmd = true;
        }
        atk.push(m);
        g.player_mut(p).stat("ninjutsu", 1);
        crate::glog!(g, "    ninjutsu: {} replaces {}", name(g, c), g.perm(a).name);
        if !g.hooks.is_empty() {
            // the attack a ninjutsu trigger can add to (Thousand-Faced Shadow) is passed along
            for (src, imp) in hooked(g, Event::Ninjutsu) {
                (imp.ninjutsu.unwrap())(g, src, p, m, atk)?;
                if g.over {
                    break;
                }
            }
        }
    }
    Ok(())
}

/// every creature card in hand has ninjutsu {2}{U}{B}; ninjutsu digs three deep
fn satoru(g: &mut Game, src: Src, p: PlayerId, _m: PermId, _atk: &mut Vec<PermId>) -> Res {
    if p == owner(g, src) && once_per_turn(g, p, intern(&format!("satoru{}", src.0))) {
        let k = 3.min(g.player(p).library.len());
        let mut top: Vec<CardId> = (0..k).map(|_| g.player_mut(p).library.pop().unwrap()).collect();
        if !top.is_empty() {
            let c = max_by(&top, |c| card_worth(g, p, c, false)).unwrap();
            remove_first(&mut top, c);
            g.player_mut(p).hand.push(c);
        }
        g.player_mut(p).library.splice(0..0, top);
    }
    Ok(())
}

/// t4._ninja: a Ninja (Changeling Outcast too)
pub fn ninja(g: &Game, m: PermId) -> bool {
    has_type(g, m, "ninja")
}

/// commander ninjutsu; each Ninja hit reveals the top card: to hand, each opponent loses its mana value
fn yuriko(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    let o = owner(g, src);
    if owner(g, a) != o || !ninja(g, a) || g.player(p).library.is_empty() {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "reveal the top card, each opponent loses its mana value", Some(5.0))? {
        return Ok(());
    }
    let Some(c) = g.player_mut(p).library.pop() else { return Ok(()) };
    let pl = g.player_mut(p);
    pl.hand.push(c);
    pl.seen.insert(c);
    let cmc = g.db.get(c).cmc as i32;
    if cmc != 0 {
        each_opp_loses(g, p, cmc)?;
    }
    crate::glog!(g, "    Yuriko reveals {}: each opponent loses {}", name(g, c), cmc);
    Ok(())
}

/// t4.YURIKO_BIG: the cards worth stacking on top for Yuriko's reveal
const YURIKO_BIG: [&str; 4] = ["Draco", "Enter the Infinite", "Dig Through Time", "Treasure Cruise"];
/// t4.EVASIVE_ONE: the evasive one-drops that carry Yuriko in on turn two
const EVASIVE_ONE: [&str; 11] = [
    "Ornithopter",
    "Changeling Outcast",
    "Gudul Lurker",
    "Slither Blade",
    "Faerie Seer",
    "Spectral Sailor",
    "Triton Shorestalker",
    "Siren Stormtamer",
    "Hope of Ghirapur",
    "Moon-Circuit Hacker",
    "Ingenious Infiltrator",
];

/// t4.yuriko_wish: tutors: a big card for the reveal only when it ends up on top (Vampiric Tutor, or a way to put it
/// back); otherwise card advantage and Yuriko's enablers
pub fn yuriko_wish(g: &Game, p: PlayerId) -> Vec<CardId> {
    let pl = g.player(p);
    let yuri = pl.perms.iter().any(|&m| g.perm(m).is_cmd) || pl.cmd_in_zone;
    let putback = pl.to_top
        || pl.hand.iter().any(|&c| name(g, c) == "Brainstorm")
        || pl.perms.iter().any(|&m| matches!(card_name(g, m), "Scroll Rack" | "Sensei's Divining Top"));
    let names: &[&str] = if yuri && putback {
        &YURIKO_BIG
    } else {
        &["Rhystic Study", "Scroll Rack", "Sensei's Divining Top", "Mystic Remora"]
    };
    names.iter().filter_map(|n| g.db.id(n)).collect()
}

/// t4.yuriko_prio: evasive one-drops first: they carry Yuriko in on turn two; with a ninjutsu window open, cantrips
/// before combat stack the biggest cards for the reveal
pub fn yuriko_prio(g: &Game, p: PlayerId, c: CardId) -> Option<i32> {
    let d = g.db.get(c);
    let n = &*d.name;
    if matches!(n, "Brainstorm" | "Ponder" | "Preordain") && crate::ai::pool::combat_reserve(g, p).is_some() {
        let hand = &g.player(p).hand;
        let me = hand.iter().position(|&x| x == c); // `x is not c`: this copy only
        let big = hand.iter().enumerate().any(|(i, &x)| Some(i) != me && g.db.get(x).cmc >= 5) || n != "Brainstorm";
        return Some(if big { 72 } else { 45 });
    }
    if EVASIVE_ONE.contains(&n) || (d.creature && d.cmc <= 2 && (d.tag(Tag::Fly) || matches!(n, "Invisible Stalker"))) {
        let have = g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).count();
        return Some(if have < 2 { 80 } else { 58 });
    }
    None
}

/// ninjutsu {0}{U}{B}: a Ninja you control deals combat damage: draw a card
fn infiltrator(g: &mut Game, src: Src, _p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    let o = owner(g, src);
    if owner(g, a) == o && ninja(g, a) && trigger_window(g, o, Some(src), "draw a card", None)? {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

/// ninjutsu {U}: returns the best creature of the damaged player's to hand
fn mistblade(g: &mut Game, src: Src, _p: PlayerId, a: PermId, d: PlayerId, _dmg: i32) -> Res {
    if a != src {
        return Ok(());
    }
    let cs: Vec<PermId> =
        g.player(d).perms.iter().copied().filter(|&m| g.is_creature(m) && !untargetable(g, m)).collect();
    let Some(t) = max_by(&cs, |m| pval(g, m)) else { return Ok(()) };
    let o = owner(g, src);
    if trigger_window(g, o, Some(src), &format!("return {} to hand", g.perm(t).name), Some(5.0))? && controls(g, d, t) {
        bounce(g, t)?;
    }
    Ok(())
}

/// ninjutsu {U}: draws on combat damage
fn hacker(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if a == src && trigger_window(g, p, Some(src), "draw a card", None)? {
        draw(g, p, 1, false)?;
    }
    Ok(())
}

/// ninjutsu {1}{U}: a Treasure once per damaged player each turn, when a Ninja or Rogue of yours connects
fn thief(g: &mut Game, src: Src, p: PlayerId, a: PermId, d: PlayerId, _dmg: i32) -> Res {
    let k = intern(&format!("thief{}_{}", src.0, d.0));
    if owner(g, a) == owner(g, src)
        && (ninja(g, a) || has_type(g, a, "rogue"))
        && fresh(g, p, k)
        && trigger_window(g, p, Some(src), "create a Treasure", None)?
        && once_per_turn(g, p, k)
    {
        make_artifact_tokens(g, p, "Treasure", 1)?;
    }
    Ok(())
}

/// ninjutsu {2}{U}: a token copy of itself on combat damage
fn naga(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if a == src && trigger_window(g, p, Some(src), "create a token copy", None)? {
        let cd = g.perm(src).cd.unwrap();
        enter_token_copy(g, p, cd)?;
    }
    Ok(())
}

/// ninjutsu {3}{B}{B}: reanimates the damaged player's best creature card under your control
fn inkeyes(g: &mut Game, src: Src, p: PlayerId, a: PermId, d: PlayerId, _dmg: i32) -> Res {
    if a != src {
        return Ok(());
    }
    let creatures =
        |g: &Game| -> Vec<CardId> { g.player(d).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect() };
    if creatures(g).is_empty() || !trigger_window(g, p, Some(src), "reanimate a creature card", Some(5.0))? {
        return Ok(());
    }
    let cs = creatures(g);
    let Some(c) = first_max(&cs, |c| bomb_or_pow(g, c)) else { return Ok(()) };
    remove_first(&mut g.player_mut(d).gy, c);
    enter(g, p, c, Enter { orig: Some(d), ..Enter::default() })?;
    Ok(())
}

/// ninjutsu {2}{U}{B}: exiles the top two cards of the damaged player's library and plays them free. (A permanent
/// card stays in their exile as it enters under your control, as in the Python.)
fn shinobi(g: &mut Game, src: Src, p: PlayerId, a: PermId, d: PlayerId, _dmg: i32) -> Res {
    if a != src || g.player(d).library.is_empty() {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "exile the top two cards, play them free", Some(5.0))? {
        return Ok(());
    }
    for _ in 0..2 {
        let Some(c) = g.player_mut(d).library.pop() else { return Ok(()) };
        g.player_mut(d).exile.push(c);
        let def = g.db.get(c);
        if def.land {
            continue;
        }
        if def.perm {
            enter(g, p, c, Enter { orig: Some(d), ..Enter::default() })?;
        } else if def.instant || def.sorcery {
            remove_first(&mut g.player_mut(d).exile, c);
            g.player_mut(p).hand.push(c);
            cast_card(g, p, c, "hand", Ctx::default())?;
        }
    }
    Ok(())
}

/// ninjutsu {4}{U}{B}: casts the best card from the damaged player's hand free
fn oni(g: &mut Game, src: Src, p: PlayerId, a: PermId, d: PlayerId, _dmg: i32) -> Res {
    if a != src {
        return Ok(());
    }
    let cards = |g: &Game| -> Vec<CardId> {
        g.player(d).hand.iter().copied().filter(|&c| !g.db.get(c).land && !g.db.get(c).tag(Tag::Ctr)).collect()
    };
    if cards(g).is_empty() || !trigger_window(g, p, Some(src), "cast a card from that player's hand", Some(5.0))? {
        return Ok(());
    }
    let cs = cards(g);
    let Some(c) = max_by(&cs, |c| card_worth(g, d, c, false)) else { return Ok(()) };
    remove_first(&mut g.player_mut(d).hand, c);
    if g.db.get(c).perm {
        enter(g, p, c, Enter { orig: Some(d), ..Enter::default() })?;
    } else {
        g.player_mut(p).hand.push(c);
        cast_card(g, p, c, "hand", Ctx::default())?;
    }
    Ok(())
}

/// ninjutsu {2}{U}{U}: searches for a Ninja on combat damage
fn higure(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if a == src && trigger_window(g, p, Some(src), "search for a Ninja", None)? {
        crate::impls::t1::tutor_named(g, p, &|g, c| g.db.get(c).has_subtype("ninja"), 1, "hand")?;
    }
    Ok(())
}

/// it entered from hand attacking (ninjutsu): a token copy of another attacking creature, tapped and attacking.
/// Only the ninjutsu itself triggers it, so a token copy (or a Shadow tapped by Thalia, Heretic Cathar) never does.
fn thousand_faced(g: &mut Game, src: Src, _p: PlayerId, m: PermId, atk: &mut Vec<PermId>) -> Res {
    if m != src {
        return Ok(());
    }
    let others: Vec<PermId> =
        atk.iter().copied().filter(|&x| x != src && g.perm(x).cd.is_some() && controls(g, owner(g, x), x)).collect();
    if let Some(b) = max_by(&others, |x| pval(g, x)) {
        let cd = g.perm(b).cd.unwrap();
        if let Some(t) = enter_token_copy(g, owner(g, src), cd)? {
            let x = g.perm_mut(t);
            x.tapped = true;
            x.sick = false;
            atk.push(t);
        }
    }
    Ok(())
}

/// Tetsuko Umezawa: your creatures with power or toughness 1 or less can't be blocked
fn tetsuko(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "unblockable" && owner(g, m) == owner(g, src) && g.is_creature(m) && (epow(g, m) <= 1 || etgh(g, m) <= 1)
}

fn always_2_5(_: &Game, _: PlayerId, _: PermId) -> Option<f64> {
    Some(2.5)
}

fn always_3(_: &Game, _: PlayerId, _: PermId) -> Option<f64> {
    Some(3.0)
}

fn always_2(_: &Game, _: PlayerId, _: PermId) -> Option<f64> {
    Some(2.0)
}

fn always_1(_: &Game, _: PlayerId, _: PermId) -> Option<f64> {
    Some(1.0)
}

fn draw1(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    draw(g, p, 1, false)
}

/// t4._kaito_minus: an unblockable 1/1 Ninja
fn kaito_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let spec = Tokens { color: Some(Colors::from_letters("U")), types: vec!["ninja"], ..Tokens::new(1, 1) };
    for t in make_tokens(g, p, spec)? {
        g.perm_mut(t).data.set(DataKey::Unblockable, Val::Bool(true));
    }
    Ok(())
}

fn always_8(_: &Game, _: PlayerId, _: PermId) -> Option<f64> {
    Some(8.0)
}

fn always_9(_: &Game, _: PlayerId, _: PermId) -> Option<f64> {
    Some(9.0)
}

/// rules.give_emblem: p gets an emblem (its effects: rules.emblem_cast, emblem_combat ...)
fn give_emblem(g: &mut Game, p: PlayerId, kind: Sym) {
    let e = &mut g.player_mut(p).emblems;
    if !e.contains(&kind) {
        e.push(kind);
    }
}

/// rules.ult's -7 for Kaito: the emblem (combat damage puts a creature from your library onto the battlefield)
fn kaito_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    give_emblem(g, p, "kaito");
    Ok(())
}

/// Kaito Shizuki (Approximate: phasing on its first turn is not modeled; the Ninja token is unblockable). The -7 is
/// rules.py's (`rules.ult` appends it to t4's list); the whole list is registered here.
static KAITO: [WalkerAb; 3] = [
    WalkerAb { delta: 1, label: "draw (discard unless attacked)", val: always_2_5, eff: draw1 },
    WalkerAb { delta: -2, label: "unblockable Ninja", val: always_3, eff: kaito_minus },
    WalkerAb { delta: -7, label: "emblem", val: always_8, eff: kaito_ult },
];

// ======================================================== Krenko, Mob Boss (Goblins)
/// t4.goblins: n 1/1 red Goblins
pub fn goblins(g: &mut Game, p: PlayerId, n: u32) -> Res<Vec<PermId>> {
    let spec = Tokens { color: Some(Colors::from_letters("R")), types: vec!["goblin"], ..Tokens::new(n, 1) };
    make_tokens(g, p, spec)
}

/// t4._goblin_haste: Goblin Warchief or Goblin Chieftain gives p's Goblins haste
fn goblin_haste(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| matches!(card_name(g, m), "Goblin Warchief" | "Goblin Chieftain"))
}

/// {T}: a Goblin per Goblin (instant speed). Without a haste enabler the tokens can't attack the turn they're made,
/// so Krenko waits for the end of the turn before yours (the tokens are ready to attack, and sorcery-speed removal
/// and wipes had their chance); with haste, activating in your main phase attacks right away.
fn krenko(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if x.tapped || (x.sick && !crate::dsl::has_kw(g, src, "haste") && !goblin_haste(g, p)) {
        return Ok(vec![]);
    }
    let n = crate::impls::t1::count_type(g, p, "goblin", false) as f64;
    let haste = goblin_haste(g, p) || has_named(g, p, "Legion Loyalist");
    let u = match post {
        None => 3.5 + 0.4 * n, // end of the turn before yours
        Some(false) => {
            if haste {
                3.0 + 0.4 * n
            } else if n < 4.0 {
                0.2
            } else {
                1.0 + 0.2 * n
            }
        }
        Some(true) => 0.2 + 0.1 * n, // after combat: only if nothing better will come
    };
    Ok(vec![ability(u, format!("Krenko: {n} Goblins"), src, krenko_go, 0)])
}

fn krenko_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if g.perm(src).tapped {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if !ability_window(g, p, Some(src), "a Goblin per Goblin", Some(5.0), None)? {
        return Ok(true);
    }
    let n = crate::impls::t1::count_type(g, p, "goblin", false);
    goblins(g, p, n.max(0) as u32)?;
    crate::glog!(g, "  Krenko makes {} Goblins", n);
    Ok(true)
}

/// Goblin Warchief: Goblins you control have haste
fn warchief_haste(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "haste" && owner(g, m) == owner(g, src) && has_type(g, m, "goblin")
}

/// Goblin Chieftain: other Goblins you control have haste
fn chieftain_haste(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "haste" && owner(g, m) == owner(g, src) && has_type(g, m, "goblin") && m != src
}

/// common._reducer for Goblin Warchief: Goblin spells cost {1} less
fn warchief_cost(g: &Game, src: Src, caster: PlayerId, c: CardId) -> i32 {
    if caster == owner(g, src) && g.db.get(c).has_subtype("goblin") { -1 } else { 0 }
}

/// common._reducer for Ruby Medallion: red spells cost {1} less
fn ruby_cost(g: &Game, src: Src, caster: PlayerId, c: CardId) -> i32 {
    if caster == owner(g, src) && g.db.get(c).pips.contains('R') { -1 } else { 0 }
}

/// t4._etb_goblins: n Goblins as it enters
fn etb_goblins(g: &mut Game, src: Src, m: PermId, n: u32) -> Res {
    let o = owner(g, src);
    let text = format!("create {n} Goblin{}", if n > 1 { "s" } else { "" });
    if m == src && trigger_window(g, o, Some(src), &text, None)? {
        goblins(g, o, n)?;
    }
    Ok(())
}

/// Beetleback Chief: two Goblins
fn beetleback(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    etb_goblins(g, src, m, 2)
}

/// Siege-Gang Commander (Approximate: three Goblins; the sacrifice-for-2 ability works as a sacrifice outlet)
fn siege_gang(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    etb_goblins(g, src, m, 3)
}

/// Goblin Instigator: one Goblin
fn instigator(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    etb_goblins(g, src, m, 1)
}

/// Mogg War Marshal (Approximate: Goblin on entry and death; echo not paid)
fn mogg(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if m == src && trigger_window(g, o, Some(src), "create a Goblin", None)? {
        goblins(g, o, 1)?;
    }
    Ok(())
}

fn mogg_dies(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, m);
    if trigger_window(g, o, Some(m), "create a Goblin", None)? {
        goblins(g, o, 1)?;
    }
    Ok(())
}

/// Hordeling Outburst: three Goblins
fn outburst(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    goblins(g, p, 3)?;
    Ok("gy")
}

fn prio_55(_: &Game, _: PlayerId, _: CardId) -> i32 {
    55
}

fn prio_45(_: &Game, _: PlayerId, _: CardId) -> i32 {
    45
}

fn prio_0(_: &Game, _: PlayerId, _: CardId) -> i32 {
    0
}

/// Impact Tremors: 1 damage to each opponent per creature of yours entering
fn tremors(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if owner(g, m) == o
        && g.is_creature(m)
        && m != src
        && trigger_window(g, o, Some(src), "1 damage to each opponent", None)?
    {
        each_opp_loses(g, o, 1)?;
    }
    Ok(())
}

/// Pashalik Mons (Partial: a Goblin dying deals 1; the token-making activation is not used)
fn mons(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, src);
    if owner(g, m) == o
        && (m == src || has_type(g, m, "goblin"))
        && trigger_window(g, o, Some(src), "1 damage to any target", None)?
    {
        best_target_any(g, o, 1)?;
    }
    Ok(())
}

/// Goblin Sharpshooter (Approximate: 1 damage whenever a creature dies; the untap loop is abstracted)
fn sharpshooter(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, src);
    if m != src && trigger_window(g, o, Some(src), "1 damage to any target", None)? {
        best_target_any(g, o, 1)?;
    }
    Ok(())
}

/// Goblin Chainwhirler: 1 damage to each opponent and each creature they control
fn chainwhirler(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if m == src && trigger_window(g, o, Some(src), "1 damage to each opponent and their creatures", Some(5.0))? {
        for q in opps(g, o) {
            lose_life(g, q, 1, Some(o), "triggers", None)?;
            for x in g.player(q).perms.clone() {
                if g.is_creature(x) && etgh(g, x) <= 1 {
                    die(g, x, "destroy")?;
                }
            }
        }
    }
    Ok(())
}

/// Goblin Lackey: combat damage puts a Goblin from hand onto the battlefield
fn lackey(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if a != src {
        return Ok(());
    }
    let gobs = |g: &Game| -> Vec<CardId> {
        g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).has_subtype("goblin") && g.db.get(c).perm).collect()
    };
    if gobs(g).is_empty() || !trigger_window(g, p, Some(src), "put a Goblin onto the battlefield", None)? {
        return Ok(());
    }
    let gs = gobs(g);
    if let Some(c) = max_by(&gs, |c| g.db.get(c).cmc as f64) {
        remove_first(&mut g.player_mut(p).hand, c);
        enter(g, p, c, Enter::default())?;
    }
    Ok(())
}

/// Goblin Recruiter (Approximate: stacks the four best Goblins on top)
fn recruiter(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "stack Goblins on top", None)? {
        return Ok(());
    }
    let mut gs: Vec<CardId> =
        g.player(o).library.iter().copied().filter(|&c| g.db.get(c).has_subtype("goblin")).collect();
    sort_by_key(&mut gs, |c| card_worth(g, o, c, false));
    let gs: Vec<CardId> = gs[gs.len().saturating_sub(4)..].to_vec();
    for &c in &gs {
        remove_first(&mut g.player_mut(o).library, c);
    }
    let mut lib = std::mem::take(&mut g.player_mut(o).library);
    g.rng.shuffle(&mut lib);
    lib.extend(gs);
    g.player_mut(o).library = lib;
    Ok(())
}

/// Goblin Ringleader: the Goblins from the top four to hand
fn ringleader(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = owner(g, src);
    if m == src && trigger_window(g, o, Some(src), "take the Goblins from the top four", None)? {
        crate::impls::t1::look_take(g, o, 4, &|g, c| g.db.get(c).has_subtype("goblin"), "hand", 4)?;
    }
    Ok(())
}

/// Muxus, Goblin Grandee (Partial: Goblins from the top six onto the battlefield; the attack pump is not used)
fn muxus(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "Goblins from the top six onto the battlefield", Some(6.0))? {
        return Ok(());
    }
    let k = 6.min(g.player(o).library.len());
    let top: Vec<CardId> = (0..k).map(|_| g.player_mut(o).library.pop().unwrap()).collect();
    for c in top {
        let d = g.db.get(c);
        if d.creature && d.has_subtype("goblin") && d.cmc <= 5 {
            enter(g, o, c, Enter::default())?;
        } else {
            g.player_mut(o).library.insert(0, c);
        }
    }
    Ok(())
}

/// Goblin Piledriver: +2/+0 per other attacking Goblin
fn piledriver(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if atk.contains(&src) && trigger_window(g, p, Some(src), "gets +2/+0 per other attacking Goblin", None)? {
        let n = atk.iter().filter(|&&m| m != src && has_type(g, m, "goblin")).count() as i32;
        eot(g, src, 2 * n, 0);
    }
    Ok(vec![])
}

/// Battle Cry Goblin (Approximate: pack tactics Goblin; the pump is not used)
fn battle_cry(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if atk.contains(&src)
        && atk.iter().map(|&m| epow(g, m)).sum::<i32>() >= 6
        && trigger_window(g, p, Some(src), "create an attacking Goblin", None)?
    {
        let spec = Tokens {
            color: Some(Colors::from_letters("R")),
            types: vec!["goblin"],
            attacking: true,
            sick: false,
            ..Tokens::new(1, 1)
        };
        return make_tokens(g, p, spec);
    }
    Ok(vec![])
}

/// Krenko, Tin Street Kingpin: a +1/+1 counter, then Goblins equal to its power
fn kingpin(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if atk.contains(&src) && trigger_window(g, p, Some(src), "a +1/+1 counter, then Goblins equal to its power", None)?
    {
        g.perm_mut(src).plus += 1;
        let n = epow(g, src);
        goblins(g, p, n.max(0) as u32)?;
    }
    Ok(vec![])
}

/// Battle Hymn (Approximate: adds R per creature; used only with a big hand)
fn hymn(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let n = g.player(p).perms.iter().filter(|&&m| g.is_creature(m)).count() as u32;
    g.player_mut(p).floating.any += n;
    Ok("gy")
}

/// Thornbite Staff (Approximate: attaches to Krenko; the untap loop is a combo)
fn staff(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        let o = owner(g, src);
        let k = g.player(o).perms.iter().copied().find(|&x| card_name(g, x) == "Krenko, Mob Boss");
        if k.is_some() {
            g.perm_mut(src).attached = k;
        }
    }
    Ok(())
}

/// t4._chandra_plus: exile the top card: cast it, or 2 damage to each opponent
fn chandra_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    let Some(c) = g.player_mut(p).library.pop() else { return Ok(()) };
    let d = g.db.get(c);
    let (gen_, pips) = (d.generic, d.pips.to_string());
    if !d.land && can_pay(g, p, gen_, &pips, false) && !d.tag(Tag::Ctr) {
        g.player_mut(p).hand.push(c);
        pay(g, p, gen_, &pips, false)?;
        cast_card(g, p, c, "hand", Ctx::default())?;
    } else {
        g.player_mut(p).exile.push(c);
        each_opp_loses(g, p, 2)?;
    }
    Ok(())
}

fn chandra_rr(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    g.player_mut(p).floating.r += 2;
    Ok(())
}

fn small_creature(g: &Game, m: PermId) -> bool {
    etgh(g, m) <= 4
}

fn chandra_minus_val(g: &Game, p: PlayerId, _src: PermId) -> Option<f64> {
    let t = best_opp_creature(g, p, |m| small_creature(g, m))?;
    if pval(g, t) >= 4.0 { Some(pval(g, t) - 2.0) } else { None }
}

fn chandra_minus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    // (Python passes None on to apply_removal when nothing is left to hit; that can't happen right after the value)
    match best_opp_creature(g, p, |m| small_creature(g, m)) {
        Some(t) => apply_removal(g, Some(p), t, "dmg4", None),
        None => Ok(()),
    }
}

/// rules.ult's -7 for Chandra: the emblem (5 damage per spell you cast)
fn chandra_ult(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    give_emblem(g, p, "chandra");
    Ok(())
}

/// Chandra, Torch of Defiance: both +1s, -3 for 4 damage, -7 emblem. The -7 is rules.py's (`rules.ult` appends it
/// to t4's list); the whole list is registered here.
static CHANDRA: [WalkerAb; 4] = [
    WalkerAb { delta: 1, label: "exile top: cast it or 2 damage each", val: always_3, eff: chandra_plus },
    WalkerAb { delta: 1, label: "RR", val: always_1, eff: chandra_rr },
    WalkerAb { delta: -3, label: "4 damage to a creature", val: chandra_minus_val, eff: chandra_minus },
    WalkerAb { delta: -7, label: "emblem", val: always_9, eff: chandra_ult },
];

// ======================================================== Chulane, Teller of Tales (and Aluren)
/// creature spells: draw, then a land from hand onto the battlefield (the bounce activation is only part of the
/// Aluren combo)
fn chulane(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if caster != owner(g, src) || !g.db.get(c).creature {
        return Ok(());
    }
    if !trigger_window(g, caster, Some(src), "draw a card, then put a land onto the battlefield", None)? {
        return Ok(());
    }
    draw(g, caster, 1, false)?;
    crate::impls::t1::put_land(g, caster)
}

/// Aluren (Approximate: creature spells with MV 3 or less are free for everyone; the flash part is ignored; the
/// Chulane loop is a combo)
fn aluren(g: &Game, _src: Src, _caster: PlayerId, c: CardId) -> i32 {
    let d = g.db.get(c);
    if d.creature && d.cmc <= 3 { -99 } else { 0 }
}

/// t4._self_bounce: as it enters, return a permanent you control to hand: your best enters-the-battlefield one
/// (`pred_other`), or itself
fn self_bounce(g: &mut Game, src: Src, m: PermId, pred_other: fn(&Game, PermId) -> bool) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    if !trigger_window(g, o, Some(src), "return a permanent you control to hand", None)? {
        return Ok(());
    }
    let mine: Vec<PermId> =
        g.player(o).perms.iter().copied().filter(|&x| x != src && pred_other(g, x) && !g.perm(x).is_cmd).collect();
    let etb_value = |cd: CardId| crate::dsl::etb_value(g, cd) as f64;
    let cands: Vec<PermId> = mine
        .into_iter()
        .filter(|&x| {
            g.perm(x).cd.is_some_and(|cd| g.registry.get(cd).is_some_and(|i| i.etb.is_some()) || etb_value(cd) > 0.0)
        })
        .collect();
    let tgt = max_by(&cands, |x| etb_value(g.perm(x).cd.unwrap())).unwrap_or(src);
    if !controls(g, o, tgt) {
        return Ok(());
    }
    if g.perm(tgt).token { leave(g, tgt) } else { bounce(g, tgt) }
}

fn creature_pred(g: &Game, x: PermId) -> bool {
    g.is_creature(x)
}

fn nonland_card_pred(g: &Game, x: PermId) -> bool {
    g.perm(x).cd.is_some_and(|c| !g.db.get(c).land)
}

/// Shrieking Drake (Approximate: returns your best ETB creature, or itself)
fn shrieking_drake(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    self_bounce(g, src, m, creature_pred)
}

/// Whitemane Lion (Approximate: returns your best ETB creature, or itself)
fn whitemane(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    self_bounce(g, src, m, creature_pred)
}

/// Kor Skyfisher (Approximate: returns your best ETB permanent, or itself)
fn skyfisher(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    self_bounce(g, src, m, nonland_card_pred)
}

/// Consecrated Sphinx (Approximate: draws two per opponent draw, capped by library size)
fn sphinx(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = owner(g, src);
    let k = intern(&format!("sphinx{}{}{}", src.0, g.player(p).key, g.player(p).draw_n));
    if p != o
        && fresh(g, o, k)
        && trigger_window(g, o, Some(src), "you may draw two cards", Some(4.0))?
        && once_per_turn(g, o, k)
    {
        // HUMAN(phase 9): the person chooses whether to draw two (hc.yes_no)
        if g.player(o).library.len() > 12 && g.sphinx_depth < 4 {
            g.sphinx_depth += 1; // two Sphinxes feed each other: stop after a few rounds
            let r = draw(g, o, 2, false);
            g.sphinx_depth -= 1;
            r?;
        }
    }
    Ok(())
}

/// Hullbreaker Horror (Approximate: uncounterable; each of your spells bounces the best opposing nonland permanent)
fn hullbreaker(g: &mut Game, src: Src, caster: PlayerId, _c: CardId) -> Res {
    if caster != owner(g, src) {
        return Ok(());
    }
    let Some(t) = best_opp_nonland(g, caster, |_| true) else { return Ok(()) };
    if pval(g, t) >= 3.0
        && trigger_window(g, caster, Some(src), &format!("return {} to hand", g.perm(t).name), Some(5.0))?
        && g.perm(t).on_bf
    {
        bounce(g, t)?;
    }
    Ok(())
}

// ======================================================== Prosper, Tome-Bound (Treasure, impulse draw)
/// end step: exile the top card, playable until the end of your next turn
fn prosper(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == owner(g, src)
        && !g.player(p).library.is_empty()
        && trigger_window(g, p, Some(src), "exile the top card, play it until next turn", None)?
        && let Some(c) = g.player_mut(p).library.pop()
    {
        let pl = g.player_mut(p);
        pl.hand.push(c);
        pl.seen.insert(c);
        let t = pl.turns + 1;
        pl.impulse_long.push((c, t));
    }
    Ok(())
}

/// a Treasure whenever you play a card from exile (Prosper, Laelia, Valakut, Siege, Reckless Impulse ...)
fn prosper_treasure(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let pl = g.player(caster);
    if caster == owner(g, src)
        && (pl.impulse.contains(&c) || pl.impulse_long.iter().any(|x| x.0 == c))
        && trigger_window(g, caster, Some(src), "create a Treasure", None)?
    {
        make_artifact_tokens(g, caster, "Treasure", 1)?;
    }
    Ok(())
}

/// Reckless Impulse: the top two cards, playable until the end of your next turn
fn reckless(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    for _ in 0..2 {
        if let Some(x) = g.player_mut(p).library.pop() {
            let pl = g.player_mut(p);
            pl.hand.push(x);
            pl.seen.insert(x);
            let t = pl.turns + 1;
            pl.impulse_long.push((x, t));
        }
    }
    Ok("gy")
}

/// the creatures that died this turn (t4's `g.deaths_turn[turn_stamp(g)]`)
fn deaths_now(g: &Game) -> i32 {
    match g.deaths_turn {
        Some((st, n)) if st == g.turn_stamp() => n,
        _ => 0,
    }
}

/// Mahadi, Emporium Master (Approximate: counts deaths while it is on the battlefield): a Treasure per creature that
/// died this turn
fn mahadi(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == owner(g, src)
        && deaths_now(g) != 0
        && trigger_window(g, p, Some(src), "a Treasure per creature that died", None)?
    {
        make_artifact_tokens(g, p, "Treasure", deaths_now(g))?;
    }
    Ok(())
}

fn mahadi_count(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    if g.is_creature(m) {
        let st = g.turn_stamp();
        g.deaths_turn = Some((st, deaths_now(g) + 1));
    }
    Ok(())
}

/// Ragavan, Nimble Pilferer (Approximate: Treasure and a stolen top card on hit; only permanents are cast; dash not
/// used)
fn ragavan(g: &mut Game, src: Src, p: PlayerId, a: PermId, d: PlayerId, _dmg: i32) -> Res {
    if a != src || !trigger_window(g, p, Some(src), "create a Treasure, exile the top card", Some(4.0))? {
        return Ok(());
    }
    make_artifact_tokens(g, p, "Treasure", 1)?;
    if let Some(c) = g.player_mut(d).library.pop() {
        g.player_mut(d).exile.push(c);
        let (gen_, pips) = cost_of(g, p, c);
        if !g.db.get(c).land && can_pay(g, p, gen_, &pips, false) && g.db.get(c).perm {
            pay(g, p, gen_, &pips, false)?;
            remove_first(&mut g.player_mut(d).exile, c);
            enter(g, p, c, Enter { orig: Some(d), ..Enter::default() })?;
        }
    }
    Ok(())
}

/// Xorn (Approximate: an extra Treasure for hooked Treasure makers)
fn xorn(g: &mut Game, src: Src, p: PlayerId, kinds: &[Sym], _n: i32) -> Res {
    if p == owner(g, src) && kinds.contains(&"Treasure") {
        g.player_mut(p).treasures += 1;
    }
    Ok(())
}

/// Reckless Fireweaver (Approximate: artifact tokens made by hooks trigger it; cast artifacts do not)
fn fireweaver(g: &mut Game, src: Src, p: PlayerId, _kinds: &[Sym], n: i32) -> Res {
    if p == owner(g, src) && trigger_window(g, p, Some(src), &format!("{n} damage to each opponent"), None)? {
        each_opp_loses(g, p, n)?;
    }
    Ok(())
}

/// Dark Confidant: reveal the top card at upkeep, lose life equal to its mana value
fn bob(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == owner(g, src)
        && !g.player(p).library.is_empty()
        && trigger_window(g, p, Some(src), "reveal the top card, lose life equal to its mv", None)?
        && let Some(c) = g.player_mut(p).library.pop()
    {
        g.player_mut(p).hand.push(c);
        let n = g.db.get(c).cmc as i32;
        lose_life(g, p, n, Some(p), "other", None)?;
    }
    Ok(())
}

/// Tavern Scoundrel: {1}, {T}, sacrifice another permanent: flip a coin; win: two Treasures
fn scoundrel(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    let pl = g.player(p);
    if x.tapped
        || x.sick
        || post.is_none()
        || !can_pay(g, p, 1, "", false)
        || (pl.treasures == 0 && pl.foods == 0 && pl.clues == 0)
    {
        return Ok(vec![]);
    }
    Ok(vec![ability(1.0, "Tavern Scoundrel flip".into(), src, scoundrel_go, 0)])
}

fn scoundrel_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if g.perm(src).tapped || !can_pay(g, p, 1, "", false) {
        return Ok(false);
    }
    pay(g, p, 1, "", false)?;
    g.perm_mut(src).tapped = true;
    sac_worst_permanent(g, p, Some(src))?;
    if !ability_window(g, p, Some(src), "flip a coin", None, None)? {
        return Ok(true);
    }
    if g.rng.random() < 0.5 {
        make_artifact_tokens(g, p, "Treasure", 2)?;
    }
    Ok(true)
}

/// Sticky Fingers (Approximate: menace and a Treasure on combat damage; the draw on death is ignored)
static STICKY: AuraSpec = AuraSpec { kws: &["menace"], ..AURA };

fn sticky(g: &mut Game, src: Src, _p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    let o = owner(g, src);
    if g.perm(src).attached == Some(a) && trigger_window(g, o, Some(src), "create a Treasure", None)? {
        make_artifact_tokens(g, o, "Treasure", 1)?;
    }
    Ok(())
}

/// t4._obnix_plus: each opponent loses 2 life or discards
fn obnix_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    for q in opps(g, p) {
        let (hand, life) = (!g.player(q).hand.is_empty(), g.player(q).life);
        if hand && life > 12 {
            lose_life(g, q, 2, Some(p), "drain", None)?;
        } else if hand {
            discard_worst(g, q, 1)?;
        } else {
            lose_life(g, q, 2, Some(p), "drain", None)?;
        }
    }
    Ok(())
}

fn draw2(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    draw(g, p, 2, false)
}

/// Ob Nixilis, the Adversary (Approximate: casualty copy not used)
static OBNIX: [WalkerAb; 2] = [
    WalkerAb { delta: 1, label: "each opponent loses 2 or discards", val: always_2_5, eff: obnix_plus },
    WalkerAb { delta: -2, label: "draw", val: always_2, eff: draw2 },
];

// ======================================================== GAA and control staples
/// t4._imprintable: an instant with mana value 2 or less that isn't a counterspell
fn imprintable(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    d.instant && d.cmc <= 2 && !d.tag(Tag::Ctr)
}

/// imprint: Dramatic Reversal if in hand, else the best removal / card-draw instant with mana value 2 or less
fn scepter_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = owner(g, src);
    let cands = |g: &Game| -> Vec<CardId> { g.player(o).hand.iter().copied().filter(|&c| imprintable(g, c)).collect() };
    if cands(g).is_empty() || !trigger_window(g, o, Some(src), "imprint an instant", None)? {
        return Ok(());
    }
    let cs = cands(g);
    let Some(c) = first_max(&cs, |c| {
        (name(g, c) == "Dramatic Reversal", g.db.get(c).tags.has(Tag::Rem), card_worth(g, o, c, false))
    }) else {
        return Ok(());
    };
    let pl = g.player_mut(o);
    remove_first(&mut pl.hand, c);
    pl.exile.push(c);
    g.perm_mut(src).data.set(DataKey::Imprint, Val::Card(c));
    crate::glog!(g, "    Isochron Scepter imprints {}", name(g, c));
    Ok(())
}

/// the imprinted card (`src.data.get('imprint')`)
fn imprinted(g: &Game, src: PermId) -> Option<CardId> {
    match g.perm(src).data.get(DataKey::Imprint) {
        Some(Val::Card(c)) => Some(*c),
        _ => None,
    }
}

/// {2}, {T}: cast a copy of the imprinted card (the Reversal loop itself is in the combo framework)
fn scepter_use(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src) || g.perm(src).tapped || g.perm(src).data.is_empty() || !can_pay(g, p, 2, "", false) {
        return Ok(vec![]);
    }
    let Some(c) = imprinted(g, src) else { return Ok(vec![]) };
    if name(g, c) == "Dramatic Reversal" {
        return Ok(vec![]);
    }
    let (u, target) = match scepter_target(g, p, c) {
        Some(Some(t)) => (2.0 + pval(g, t), Some(t)),
        Some(None) => return Ok(vec![]),
        None => (1.5 + card_worth(g, p, c, false) / 10.0, None),
    };
    let arg = target.map_or(-1, |t| t.0 as i64);
    Ok(vec![ability(u, format!("Isochron Scepter ({})", name(g, c)), src, scepter_go, arg)])
}

/// a removal card's best target worth casting a copy at: None for a card that isn't removal, Some(None) when no
/// target is worth it
fn scepter_target(g: &Game, p: PlayerId, c: CardId) -> Option<Option<PermId>> {
    let t = &g.db.get(c).tags;
    let rem = t.str(Tag::Rem)?;
    let tg = legal_targets(g, p, rem, t.str(Tag::Tgt).unwrap_or("c"), t.has(Tag::Mv4), Some(c));
    let best = max_by(&tg, |m| pval(g, m));
    Some(best.filter(|&b| pval(g, b) >= 3.0))
}

fn scepter_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    if g.perm(src).tapped || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    let Some(c) = imprinted(g, src) else { return Ok(false) };
    pay(g, p, 2, "", false)?;
    g.perm_mut(src).tapped = true;
    let n = name(g, c).to_string();
    if !ability_window(g, p, Some(src), &format!("cast a copy of {n}"), None, None)? {
        return Ok(true);
    }
    crate::glog!(g, "  {} casts a copy of {} with Isochron Scepter", pname(g, p), n);
    let gy_n = g.player(p).gy.len();
    let ctx = Ctx { target: (arg >= 0).then_some(PermId(arg as u32)), ..Ctx::default() };
    cast_card(g, p, c, "lib", ctx)?;
    let pl = g.player_mut(p);
    if pl.gy.len() > gy_n && pl.gy.last() == Some(&c) {
        pl.gy.pop(); // the copy ceases to exist
    } else if pl.exile.last() == Some(&c) {
        pl.exile.pop();
    }
    Ok(true)
}

/// t4._scepter_prio: with Dramatic Reversal in hand; held while the Reversal is still in the library
fn scepter_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let pl = g.player(p);
    if pl.hand.iter().any(|&x| name(g, x) == "Dramatic Reversal") {
        return 62;
    }
    if pl.library.iter().any(|&x| name(g, x) == "Dramatic Reversal") {
        return 0; // wait for the Reversal
    }
    if pl.hand.iter().any(|&x| imprintable(g, x) && g.db.get(x).tags.has(Tag::Rem)) { 30 } else { 0 }
}

/// Sanguine Bond: your life gain drains the most threatening opponent; with Exquisite Blood a combo
fn bond(g: &mut Game, src: Src, p: PlayerId, n: i32) -> Res {
    if p == owner(g, src)
        && g.opps(p).next().is_some()
        && g.bond_depth < 3
        && trigger_window(g, p, Some(src), &format!("an opponent loses {n} life"), Some(5.0))?
    {
        if g.opps(p).next().is_none() {
            return Ok(());
        }
        g.bond_depth += 1;
        let q = bond_target(g, p, n);
        let r = lose_life(g, q, n, Some(p), "drain", None);
        g.bond_depth -= 1;
        r?;
    }
    Ok(())
}

/// t4.bond_target: the opponent who loses the life: one it kills, else the most threatening
pub fn bond_target(g: &Game, p: PlayerId, n: i32) -> PlayerId {
    let os = opps(g, p);
    // HUMAN(phase 9): the person picks the opponent (hc.choose)
    let dead: Vec<PlayerId> =
        os.iter().copied().filter(|&q| g.player(q).life <= n && !g.player(q).life_locked).collect();
    if !dead.is_empty() {
        return max_by(&dead, |q| threat(g, p, q)).unwrap();
    }
    max_by(&os, |q| threat(g, p, q)).unwrap()
}

/// Exquisite Blood: opponents' life loss gains you life; with Sanguine Bond a combo
fn blood(g: &mut Game, src: Src, p: PlayerId, n: i32) -> Res {
    let o = owner(g, src);
    if p != o && g.bond_depth < 3 && trigger_window(g, o, Some(src), &format!("gain {n} life"), Some(4.0))? {
        g.bond_depth += 1;
        let r = gain(g, o, n);
        g.bond_depth -= 1;
        r?;
    }
    Ok(())
}

// ======================================================== Heliod, Sun-Crowned (replaces GAA IV in Tier 4)
/// t4.heliod_update: a creature (5/5) only while its controller's devotion to white is 5 or more
fn heliod_update(g: &mut Game, src: PermId) {
    let anim = devotion(g, owner(g, src), 'W') >= 5;
    let x = g.perm_mut(src);
    x.pow = 5;
    x.tgh = 5;
    x.data.set(DataKey::Anim, Val::Bool(anim));
}

fn heliod_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src || owner(g, m) == owner(g, src) {
        heliod_update(g, src);
    }
    Ok(())
}

fn heliod_sba(g: &mut Game, src: Src) -> Res {
    heliod_update(g, src);
    Ok(())
}

fn heliod_ind(_g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "indestructible" && m == src
}

/// whenever you gain life: a +1/+1 counter on your best creature (a Walking Ballista first: one more ping)
fn heliod_gain(g: &mut Game, src: Src, p: PlayerId, _n: i32) -> Res {
    if p != owner(g, src) {
        return Ok(());
    }
    let cre = |g: &Game| -> Vec<PermId> {
        g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased && m != src).collect()
    };
    if cre(g).is_empty() || !trigger_window(g, p, Some(src), "a +1/+1 counter on a creature", None)? {
        return Ok(());
    }
    let cs = cre(g);
    let t = cs.iter().copied().find(|&m| card_name(g, m) == "Walking Ballista").or_else(|| max_by(&cs, |m| pval(g, m)));
    if let Some(t) = t {
        g.perm_mut(t).plus += 1;
    }
    Ok(())
}

// ======================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    // ninjutsu and Yuriko's Ninjas
    r.card(db, "Satoru Umezawa")?.ninjutsu = Some(satoru);
    r.card(db, "Yuriko, the Tiger's Shadow")?.combat_damage = Some(yuriko);
    r.card(db, "Ingenious Infiltrator")?.combat_damage = Some(infiltrator);
    r.card(db, "Mistblade Shinobi")?.combat_damage = Some(mistblade);
    r.card(db, "Moon-Circuit Hacker")?.combat_damage = Some(hacker);
    r.card(db, "Prosperous Thief")?.combat_damage = Some(thief);
    r.card(db, "Mist-Syndicate Naga")?.combat_damage = Some(naga);
    r.card(db, "Ink-Eyes, Servant of Oni")?.combat_damage = Some(inkeyes);
    r.card(db, "Fallen Shinobi")?.combat_damage = Some(shinobi);
    r.card(db, "Silent-Blade Oni")?.combat_damage = Some(oni);
    r.card(db, "Higure, the Still Wind")?.combat_damage = Some(higure);
    r.card(db, "Thousand-Faced Shadow")?.ninjutsu = Some(thousand_faced);
    r.card(db, "Tetsuko Umezawa, Fugitive")?.grant_kw = Some(tetsuko);
    walker(r, db, "Kaito Shizuki", &KAITO)?;
    // Krenko and the Goblins
    r.card(db, "Krenko, Mob Boss")?.options = Some(krenko);
    let c = r.card(db, "Goblin Warchief")?;
    c.grant_kw = Some(warchief_haste);
    c.cost = Some(warchief_cost);
    r.card(db, "Goblin Chieftain")?.grant_kw = Some(chieftain_haste);
    r.card(db, "Ruby Medallion")?.cost = Some(ruby_cost);
    r.card(db, "Beetleback Chief")?.etb = Some(beetleback);
    r.card(db, "Siege-Gang Commander")?.etb = Some(siege_gang);
    r.card(db, "Goblin Instigator")?.etb = Some(instigator);
    let c = r.card(db, "Mogg War Marshal")?;
    c.etb = Some(mogg);
    c.self_dies = Some(mogg_dies);
    let c = r.card(db, "Hordeling Outburst")?;
    c.resolve = Some(outburst);
    c.prio = Some(prio_55);
    r.card(db, "Impact Tremors")?.etb = Some(tremors);
    // Purphoros, God of the Forge: rules2's etb hook replaces t4's
    // Skirk Prospector: common's SAC_OUTLET table has its entry
    r.card(db, "Pashalik Mons")?.dies = Some(mons);
    r.card(db, "Goblin Sharpshooter")?.dies = Some(sharpshooter);
    r.card(db, "Goblin Chainwhirler")?.etb = Some(chainwhirler);
    r.card(db, "Goblin Lackey")?.combat_damage = Some(lackey);
    r.card(db, "Goblin Recruiter")?.etb = Some(recruiter);
    r.card(db, "Goblin Ringleader")?.etb = Some(ringleader);
    r.card(db, "Muxus, Goblin Grandee")?.etb = Some(muxus);
    r.card(db, "Goblin Piledriver")?.attack = Some(piledriver);
    r.card(db, "Battle Cry Goblin")?.attack = Some(battle_cry);
    r.card(db, "Krenko, Tin Street Kingpin")?.attack = Some(kingpin);
    let c = r.card(db, "Battle Hymn")?;
    c.resolve = Some(hymn);
    c.prio = Some(prio_0);
    // Rionya, Fire Dancer: rules2's combat_start hook replaces t4's
    let c = r.card(db, "Thornbite Staff")?;
    c.etb = Some(staff);
    *c = c.at_once(Event::Etb);
    walker(r, db, "Chandra, Torch of Defiance", &CHANDRA)?;
    // Valakut Exploration: rules2's landfall and end_step hooks replace t4's
    // Chulane, Aluren and the self-bouncers
    r.card(db, "Chulane, Teller of Tales")?.cast = Some(chulane);
    r.card(db, "Aluren")?.cost = Some(aluren);
    r.card(db, "Shrieking Drake")?.etb = Some(shrieking_drake);
    r.card(db, "Whitemane Lion")?.etb = Some(whitemane);
    r.card(db, "Kor Skyfisher")?.etb = Some(skyfisher);
    r.card(db, "Consecrated Sphinx")?.draw = Some(sphinx);
    r.card(db, "Hullbreaker Horror")?.cast = Some(hullbreaker);
    // Prosper and the Treasure makers
    let c = r.card(db, "Prosper, Tome-Bound")?;
    c.end_step = Some(prosper);
    c.cast = Some(prosper_treasure);
    let c = r.card(db, "Reckless Impulse")?;
    c.resolve = Some(reckless);
    c.prio = Some(prio_45);
    let c = r.card(db, "Mahadi, Emporium Master")?;
    c.end_step = Some(mahadi);
    c.dies = Some(mahadi_count);
    *c = c.at_once(Event::Dies);
    r.card(db, "Ragavan, Nimble Pilferer")?.combat_damage = Some(ragavan);
    let c = r.card(db, "Xorn")?;
    c.token_created = Some(xorn);
    *c = c.at_once(Event::TokenCreated);
    r.card(db, "Reckless Fireweaver")?.token_created = Some(fireweaver);
    r.card(db, "Dark Confidant")?.upkeep = Some(bob);
    r.card(db, "Tavern Scoundrel")?.options = Some(scoundrel);
    crate::impls::common::aura(r, db, "Sticky Fingers", &STICKY)?;
    r.card(db, "Sticky Fingers")?.combat_damage = Some(sticky);
    walker(r, db, "Ob Nixilis, the Adversary", &OBNIX)?;
    // GAA's control staples
    let c = r.card(db, "Isochron Scepter")?;
    c.etb = Some(scepter_etb);
    c.options = Some(scepter_use);
    c.prio = Some(scepter_prio);
    r.card(db, "Dramatic Reversal")?.prio = Some(prio_0);
    r.card(db, "Sanguine Bond")?.gain_life = Some(bond);
    r.card(db, "Exquisite Blood")?.lose_life = Some(blood);
    // Heliod, Sun-Crowned
    let c = r.card(db, "Heliod, Sun-Crowned")?;
    c.etb = Some(heliod_etb);
    c.sba = Some(heliod_sba);
    c.grant_kw = Some(heliod_ind);
    c.gain_life = Some(heliod_gain);
    *c = c.at_once(Event::Etb);
    Ok(())
}
