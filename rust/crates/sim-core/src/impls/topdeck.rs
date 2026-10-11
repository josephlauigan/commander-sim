//! Python's `cards/impl/topdeck.py`, the cards: cantrips (Brainstorm, Ponder, Preordain, Opt, Consider, Serum
//! Visions), Sensei's Divining Top, Scroll Rack and Sylvan Library. The AI half (desire, scry, arrange, look) is
//! `ai::topdeck`. The top card is the last in `library`.

use crate::ai::topdeck::{KEEP, arrange, desire, look, mv_mode, scry};
use crate::cards::CardDb;
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::tutors::shuffle_library;
use crate::engine::values::once_per_turn;
use crate::engine::zones::{draw, leave, max_by, min_by};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, PrioFn, Registry, ResolveFn, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

fn remove_first(v: &mut Vec<CardId>, c: CardId) {
    if let Some(i) = v.iter().position(|&x| x == c) {
        v.remove(i);
    }
}

/// a stable sort by an f64 key (Python's `sorted(xs, key=...)`)
fn sorted_by(g: &Game, xs: &[CardId], key: impl Fn(&Game, CardId) -> f64) -> Vec<CardId> {
    let mut keyed: Vec<(f64, CardId)> = xs.iter().map(|&c| (key(g, c), c)).collect();
    keyed.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    keyed.into_iter().map(|x| x.1).collect()
}

/// topdeck.put_back: put n cards from hand on top (Brainstorm, Scroll Rack): the least wanted, or in Yuriko mode the
/// biggest (`exclude`: a card not to put back, the spell itself)
pub fn put_back(g: &mut Game, p: PlayerId, n: usize, exclude: Option<CardId>) {
    for _ in 0..n {
        let rest: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&x| Some(x) != exclude).collect();
        if rest.is_empty() {
            return;
        }
        let x = if mv_mode(g, p) {
            // (cmc >= 6, cmc) as one number: the biggest
            max_by(&rest, |c| {
                let mv = g.db.get(c).cmc as f64;
                if mv >= 6.0 { 1e6 + mv } else { mv }
            })
        } else {
            min_by(&rest, |c| desire(g, p, c))
        }
        .unwrap();
        let pl = g.player_mut(p);
        remove_first(&mut pl.hand, x);
        pl.library.push(x);
    }
    if mv_mode(g, p) && n > 1 {
        // the biggest last so it is revealed first
        let lib = &mut g.player_mut(p).library;
        let k = n.min(lib.len());
        let mut top = lib.split_off(lib.len() - k);
        let db = g.db.clone();
        top.sort_by_key(|&c| db.get(c).cmc);
        g.player_mut(p).library.extend(top);
    }
}

// ------------------------------------------------------------------ cantrips
fn cantrip_prio(n: i32) -> impl Fn(&Game, PlayerId) -> i32 {
    move |g, p| if g.player(p).library.len() > 10 { n } else { 0 }
}

fn prio_40(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    cantrip_prio(40)(g, p)
}

fn prio_42(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    cantrip_prio(42)(g, p)
}

fn prio_38(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    cantrip_prio(38)(g, p)
}

/// topdeck._brainstorm: draw three, then put back the two least wanted cards (Yuriko: the two biggest, for the
/// reveal). Jace's Brainstorm ability (t3) calls it with no card.
pub fn brainstorm_effect(g: &mut Game, p: PlayerId, c: Option<CardId>) -> Res {
    draw(g, p, 3, false)?;
    put_back(g, p, 2, c);
    Ok(())
}

fn brainstorm(g: &mut Game, p: PlayerId, c: CardId, _ctx: &Ctx) -> Res<Sym> {
    brainstorm_effect(g, p, Some(c))?;
    Ok("gy")
}

/// look at the top three: keep them in the best order, or shuffle if none is wanted; then draw
fn ponder(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let top = look(g, p, 3);
    let best = top.iter().map(|&x| desire(g, p, x)).fold(f64::NEG_INFINITY, f64::max);
    if !top.is_empty() && best < KEEP {
        g.player_mut(p).library.extend(top);
        shuffle_library(g, p);
    } else {
        arrange(g, p, &top);
    }
    draw(g, p, 1, false)?;
    Ok("gy")
}

/// scry 2, then draw
fn preordain(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    scry(g, p, 2, false)?;
    draw(g, p, 1, false)?;
    Ok("gy")
}

/// scry 1, then draw
fn opt(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    scry(g, p, 1, false)?;
    draw(g, p, 1, false)?;
    Ok("gy")
}

/// surveil 1, then draw
fn consider(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    scry(g, p, 1, true)?;
    draw(g, p, 1, false)?;
    Ok("gy")
}

/// draw, then scry 2
fn visions(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    draw(g, p, 1, false)?;
    scry(g, p, 2, false)?;
    Ok("gy")
}

// ------------------------------------------------------------------ permanents
/// Sensei's Divining Top: {1}: look at the top three and rearrange them (each upkeep, before the draw)
fn top_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != g.perm(src).owner || g.perm(src).tapped || !can_pay(g, p, 1, "", false) {
        return Ok(());
    }
    pay(g, p, 1, "", false)?;
    let top = look(g, p, 3);
    arrange(g, p, &top);
    Ok(())
}

/// Sensei's Divining Top: {T}: draw a card and put Top on top of the library: used when the top card is wanted now (a
/// combo piece, or with Bolas's Citadel, where Top can be cast again for 1 life)
fn top_draw(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    let Some(&c) = g.player(p).library.last() else { return Ok(vec![]) };
    if p != g.perm(src).owner || g.perm(src).tapped {
        return Ok(vec![]);
    }
    let citadel = g.player(p).perms.iter().any(|&m| g.perm(m).cd.is_some_and(|x| g.db.get(x).tag(Tag::Citadel)));
    if !citadel && desire(g, p, c) < 150.0 {
        return Ok(vec![]);
    }
    let u = if citadel { 4.0 } else { 3.0 };
    Ok(vec![Opt {
        utility: u,
        label: "Sensei's Divining Top draw".into(),
        act: Some(Action::Ability { src, f: top_go, arg: 0 }),
    }])
}

fn top_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let on = |g: &Game| g.perm(src).on_bf && g.perm(src).owner == p;
    if !on(g) || g.perm(src).tapped {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if !ability_window(g, p, Some(src), "draw, then Top goes on top", None, None)? {
        return Ok(true);
    }
    draw(g, p, 1, false)?;
    if on(g) {
        leave(g, src)?;
        if let Some(cd) = g.perm(src).cd {
            g.player_mut(p).library.push(cd);
        }
    }
    crate::glog!(g, "  {} draws with Sensei's Divining Top (Top goes on top)", g.player(p).name);
    Ok(true)
}

/// Scroll Rack: {1}, {T}: exile any number of cards from hand face down, put that many from the top into hand, then
/// put the exiled cards on top in any order: swaps the least wanted cards (Yuriko: puts the biggest on top). Each
/// upkeep, tapping it (Python never tapped it, so it was free to use again: fixed in Rust).
fn rack(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let pl = g.player(p);
    if p != g.perm(src).owner
        || g.perm(src).tapped
        || !can_pay(g, p, 1, "", false)
        || pl.hand.is_empty()
        || pl.library.is_empty()
    {
        return Ok(());
    }
    if mv_mode(g, p) {
        let big: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).cmc >= 5).collect();
        let big = sorted_by(g, &big, |g, c| g.db.get(c).cmc as f64);
        if big.is_empty() {
            return Ok(());
        }
        pay(g, p, 1, "", false)?;
        g.perm_mut(src).tapped = true;
        let top = look(g, p, big.len());
        let pl = g.player_mut(p);
        for &c in &big {
            remove_first(&mut pl.hand, c);
        }
        pl.hand.extend(top);
        pl.library.extend(big.iter().copied());
        crate::glog!(g, "  {} uses Scroll Rack: {} on top", g.player(p).name, g.db.get(*big.last().unwrap()).name);
        return Ok(());
    }
    let hand = g.player(p).hand.clone();
    let worst: Vec<CardId> = sorted_by(g, &hand, |g, c| desire(g, p, c)).into_iter().take(2).collect();
    let lib = &g.player(p).library;
    let top: Vec<CardId> = lib[lib.len().saturating_sub(2)..].to_vec();
    let top_min = top.iter().map(|&c| desire(g, p, c)).fold(f64::INFINITY, f64::min);
    let worst_max = worst.iter().map(|&c| desire(g, p, c)).fold(f64::NEG_INFINITY, f64::max);
    if top.is_empty() || top_min <= worst_max {
        return Ok(());
    }
    pay(g, p, 1, "", false)?;
    g.perm_mut(src).tapped = true;
    let top = look(g, p, worst.len());
    let pl = g.player_mut(p);
    for &c in &worst {
        remove_first(&mut pl.hand, c);
    }
    pl.hand.extend(top);
    pl.library.extend(worst);
    Ok(())
}

/// the once-per-turn key of a Sylvan Library (Python's `f'sylvan{id(src)}'`)
fn sylvan_key(src: PermId) -> Sym {
    intern(&format!("sylvan{}", src.0))
}

/// Sylvan Library: draws two extra each draw step; keeps a strong card for 4 life when above 24, puts the rest back
/// in the best order
fn library(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let k = sylvan_key(src);
    if p != g.perm(src).owner || g.active != Some(p) || g.player(p).flag_turn.get(k) == Some(&g.turn_stamp()) {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "draw two more cards", None)? || !once_per_turn(g, p, k) {
        return Ok(());
    }
    let before = g.player(p).hand.len();
    draw(g, p, 2, false)?;
    // the cards just drawn are the end of the hand (Python picked them by identity, and cards are shared per name, so
    // a drawn second copy of a card already in hand — a basic land — was kept for free; fixed in Rust)
    let new: Vec<CardId> = g.player(p).hand.get(before..).unwrap_or(&[]).to_vec();
    let order = sorted_by(g, &new, |g, c| -desire(g, p, c));
    for c in order {
        if g.player(p).life >= 24 && desire(g, p, c) >= 55.0 {
            lose_life(g, p, 4, Some(p), "other", None)?;
            crate::glog!(g, "    {} pays 4 life for a Sylvan Library card", g.player(p).name);
        } else {
            let pl = g.player_mut(p);
            remove_first(&mut pl.hand, c);
            pl.library.push(c);
        }
    }
    Ok(())
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let spells: [(&str, ResolveFn, PrioFn); 6] = [
        ("Brainstorm", brainstorm, prio_40),
        ("Ponder", ponder, prio_42),
        ("Preordain", preordain, prio_42),
        ("Opt", opt, prio_38),
        ("Consider", consider, prio_38),
        ("Serum Visions", visions, prio_40),
    ];
    for (name, f, prio) in spells {
        if db.id(name).is_none() {
            continue; // not in the export (Serum Visions: in no decklist)
        }
        let imp = r.card(db, name)?;
        imp.resolve = Some(f);
        imp.prio = Some(prio);
    }
    let top = r.card(db, "Sensei's Divining Top")?;
    top.upkeep = Some(top_upkeep);
    top.options = Some(top_draw);
    *top = top.at_once(Event::Upkeep); // no trigger window: it runs as the upkeep begins
    let rk = r.card(db, "Scroll Rack")?;
    rk.upkeep = Some(rack);
    *rk = rk.at_once(Event::Upkeep);
    r.card(db, "Sylvan Library")?.draw = Some(library);
    Ok(())
}
