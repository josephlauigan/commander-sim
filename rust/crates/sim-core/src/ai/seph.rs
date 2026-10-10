//! Sephiroth's AI (Python's `ais.py`, the `seph_*` functions, and the 'seph' parts of `ai/brain.py`, `protect_response`,
//! `wipe_response`, `end_step` and `_step_start`): cast priorities, the reanimation plan (fill the graveyard, tutor,
//! reanimate, hardcast), the loops (card code in impls::mine), protection and answers to wipes.

use super::brain::{Situation, removal_risk};
use super::decks::{bomb_on_bf, has_rean_access, pay_card, rean_options, rean_targets, remove_from_hand};
use super::{note_bomb, plans};
use crate::engine::cast::{cast_card, on_cast};
use crate::engine::combat::blocked;
use crate::engine::life::{check_state, lose_life};
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::stack::{counter_window, equip_to};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{card_name, epow, etgh, find, has, pval, stopped, untargetable};
use crate::engine::zones::{
    KEYSPELL, agent_for, agent_take, cast_teferis_protection, die, discard_cards, draw, land_ramp, max_by, mill, min_by,
};
use crate::flow::Res;
use crate::hooks::{Action, Opt};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::mine;
use crate::state::{Ctx, Game};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

fn hand_has(g: &Game, p: PlayerId, pred: impl Fn(CardId) -> bool) -> bool {
    g.player(p).hand.iter().any(|&c| pred(c))
}

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

// ------------------------------------------------------------------ values
/// ais.seph_bval: what a creature card is worth as a reanimation target
pub fn seph_bval(g: &Game, p: PlayerId, c: CardId) -> f64 {
    let d = g.db.get(c);
    let t = &d.tags;
    let mut v = if d.bomb != 0 {
        d.bomb as f64
    } else if d.creature {
        d.pow as f64
    } else {
        0.0
    };
    if t.has(Tag::Normgc) && g.opps(p).any(|q| g.player(q).key == "najeela") {
        v += 2.0;
    }
    if t.has(Tag::Mother) && g.opps(p).any(|q| g.player(q).key == "sauron") {
        v += 1.0;
    }
    if t.has(Tag::Bahamut) {
        // Mega Flare's reach
        let mv: u32 = g.player(p).perms.iter().filter_map(|&m| g.perm(m).cd).map(|c| g.db.get(c).cmc).sum();
        v += 1.5f64.min(mv as f64 / 15.0);
    }
    v
}

/// ais.own_bomb_in_gy
pub fn own_bomb_in_gy(g: &Game, p: PlayerId) -> bool {
    g.player(p).gy.iter().any(|&c| g.db.get(c).creature && seph_bval(g, p, c) >= 6.0)
}

/// ais.free_sac: a free sacrifice outlet on p's battlefield (Disruptor Flute stops one)
pub fn free_sac(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| {
        let x = g.perm(m);
        x.cd.is_some_and(|c| g.db.get(c).tag(Tag::Sac)) && !x.phased && !stopped(g, x.name)
    })
}

/// ais.seph_sac: sacrifice m to a free outlet; with Altar of Dementia out, mill yourself for its power (bombs for the
/// reanimation spells) while the library can spare it
pub fn seph_sac(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    let pw = epow(g, m);
    die(g, m, "sac")?;
    let altar = g.player(p).perms.iter().any(|&x| card_name(g, x) == Some("Altar of Dementia") && !g.perm(x).phased);
    if altar && pw > 0 && g.player(p).library.len() as i32 >= pw + 15 {
        mill(g, p, pw as usize)?;
        crate::glog!(g, "    Altar of Dementia: {} mills {pw}", g.player(p).name);
    }
    Ok(())
}

// ------------------------------------------------------------------ cast priorities
/// ais.FLICKERS
const FLICKERS: [&str; 5] =
    ["Soulherder", "Conjurer's Closet", "Teleportation Circle", "Flickering Hound", "Restoration Angel"];

/// ais.seph_prio
pub fn seph_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let pl = g.player(p);
    let d = g.db.get(c);
    let t = &d.tags;
    if let Some(lp) = mine::loop_prio(g, p, c) {
        return lp; // a piece of one of the loops
    }
    if t.has(Tag::Shards) {
        return plans::shards_prio(g, p, c);
    }
    if t.has(Tag::Gifts) {
        // Gifts Ungiven: an instant, held for the end of a turn
        if g.active == Some(p) {
            return 0;
        }
        return if seph_tutor_target(g, p).is_some() { 55 } else { 30 };
    }
    if &*d.name == "The One Ring" {
        return plans::one_ring_prio(g, p, c);
    }
    if t.has(Tag::Sphinx) {
        return super::decks::sphinx_prio(g, p, c); // Consecrated Sphinx: hard-cast from what it will draw
    }
    if c == pl.cmd {
        return 0;
    }
    if FLICKERS.contains(&&*d.name) {
        return flicker_prio(g, p, c);
    }
    if t.has(Tag::Rock) || t.has(Tag::Dork) || t.has(Tag::Lr) {
        return if pl.turns <= 5 { 80 } else { 30 };
    }
    if t.has(Tag::Tithe) {
        return crate::cardcode::tithe_prio(g, p, c); // Smothering Tithe: from the Treasures it will make
    }
    if t.has(Tag::Necro) {
        return super::decks::necro_prio(g, p, c); // Necropotence
    }
    if t.has(Tag::Citadel) {
        return plans::citadel_prio(g, p, c); // Bolas's Citadel: life above the threat floor
    }
    match t.str(Tag::Fill) {
        Some("stitcher") => return 75,
        Some("tortured") => return 65,
        Some("wayfinder") => return 45,
        _ => {}
    }
    if t.has(Tag::Eng) {
        return 60;
    }
    if t.has(Tag::Rite) {
        return 45;
    }
    if t.has(Tag::Clamp) {
        return 50;
    }
    if t.str(Tag::Prot) == Some("boots") {
        return if bomb_on_bf(g, p) { 70 } else { 25 };
    }
    if t.has(Tag::Nim) {
        // Nim Deathmantle: recursion
        return if bomb_on_bf(g, p) || own_bomb_in_gy(g, p) { 58 } else { 35 };
    }
    if t.has(Tag::Sac) {
        return 50;
    }
    if t.has(Tag::Bartist) || t.has(Tag::Drain) {
        return 45;
    }
    if t.has(Tag::Clone) {
        let bomb = g
            .players
            .iter()
            .flat_map(|q| q.perms.iter())
            .any(|&x| g.is_creature(x) && g.perm(x).cd.is_some_and(|cd| g.db.get(cd).bomb != 0));
        return if bomb { 45 } else { 0 };
    }
    let key = |x: CardId| KEYSPELL.iter().any(|&k| g.db.get(x).tag(k));
    if t.has(Tag::Witness) {
        return if pl.gy.iter().any(|&x| key(x)) { 50 } else { 20 };
    }
    if t.has(Tag::Wall) {
        let any = pl.gy.iter().any(|&x| (g.db.get(x).instant || g.db.get(x).sorcery) && key(x));
        return if any { 45 } else { 8 };
    }
    if t.has(Tag::Draw) && d.sorcery && !t.has(Tag::Fill) {
        return 35;
    }
    // ('tide': Tishana's Tidebinder, no longer in the list and no card has the tag)
    if d.creature && d.bomb == 0 {
        return 30;
    }
    0
}

/// ais.flicker_prio: the flicker engines: sooner with a creature worth flickering out. Restoration Angel waits for the
/// end of an opponent's turn, and is held to protect a bomb when it has nothing worth blinking
fn flicker_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let resto = &*g.db.get(c).name == "Restoration Angel";
    let best = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            g.is_creature(m) && !g.perm(m).phased && !(resto && crate::engine::values::has_type(g, m, "angel"))
        })
        .map(|m| mine::flicker_worth(g, p, m)) // (t2.flicker_worth is Sephiroth's own for 'seph')
        .fold(0.0, f64::max);
    if !resto {
        return if best >= 3.0 { 50 } else { 30 };
    }
    if g.active == Some(p) {
        return 0;
    }
    if best >= 3.0 {
        30 + (6.0 * best.min(5.0)) as i32
    } else if bomb_on_bf(g, p) {
        0
    } else {
        25
    }
}

// ------------------------------------------------------------------ tutors
/// ais.seph_tutor_target: the card Sephiroth's tutors fetch (by name)
pub fn seph_tutor_target(g: &Game, p: PlayerId) -> Option<&'static str> {
    let pl = g.player(p);
    let in_lib = |n: &str| g.db.id(n).is_some_and(|c| pl.library.contains(&c));
    let hand: Vec<&str> = pl.hand.iter().map(|&c| &*g.db.get(c).name).collect();
    if let Some(n) = mine::loop_need(g, p, &hand).into_iter().find(|n| in_lib(n)) {
        return Some(n); // the last piece of a loop
    }
    let rean = has_rean_access(g, p);
    let tgt = own_bomb_in_gy(g, p);
    let first = |names: &[&'static str]| names.iter().copied().find(|n| in_lib(n));
    if !bomb_on_bf(g, p) {
        if rean && !tgt {
            return first(&["Entomb", "Buried Alive", "Unmarked Grave"]);
        }
        if tgt && !rean {
            let mut order: Vec<&'static str> = if pl.life > 22 { vec!["Reanimate"] } else { vec![] };
            order.extend([
                "Animate Dead",
                "Necromancy",
                "Dread Return",
                "Persist",
                "Unburial Rites",
                "Evil Reawakened",
            ]);
            return first(&order);
        }
        if !rean && !tgt {
            return first(&["Entomb", "Buried Alive", "Animate Dead"]);
        }
    }
    first(&[
        "Heroic Intervention",
        "Counterspell",
        "Swan Song",
        "Swiftfoot Boots",
        "Dovin's Veto",
        "Phyrexian Arena",
        "Animate Dead",
    ])
}

// ------------------------------------------------------------------ the main-phase options
fn payable_in_hand(g: &Game, p: PlayerId, pred: impl Fn(CardId) -> bool) -> bool {
    g.player(p).hand.iter().any(|&c| pred(c) && can_pay(g, p, g.db.get(c).generic, &g.db.get(c).pips, false))
}

const FILLERS: [&str; 4] = ["Entomb", "Buried Alive", "Unmarked Grave", "Grisly Salvage"];

/// brain.special_options for Sephiroth: the loops, reanimation, Yawgmoth's Will, Tortured Existence, filling the
/// graveyard, tutors, Insatiable Avarice, hardcasting a bomb, Trophy on Scavenger Grounds, Skullclamp, Deadly Dispute,
/// equipping Boots
pub fn options(g: &mut Game, p: PlayerId, s: &Situation, _post: bool) -> Res<Vec<Opt>> {
    let mut o = mine::loop_options(g, p); // Sephiroth's loops
    let gy_bomb = own_bomb_in_gy(g, p);
    let rean = has_rean_access(g, p);
    let pl = g.player(p);
    let plan = |u: f64, label: &str, f: crate::hooks::PlanFn| Opt {
        utility: u,
        label: label.into(),
        act: Some(Action::Plan { f, arg: 0 }),
    };
    // reanimation: best payable (spell, target) pair
    let mut best_t: f64 = 0.0;
    for (c, _zone, cg, cp) in rean_options(g, p) {
        let kind = g.db.get(c).tags.str(Tag::Rean).unwrap_or("");
        let mut tg = rean_targets(g, p, kind);
        if kind == "reanimate" {
            tg.retain(|x| pl.life - g.db.get(x.1).cmc as i32 >= 12);
        }
        if !tg.is_empty() && can_pay(g, p, cg, &cp, false) {
            best_t = best_t.max(tg[0].0);
        }
    }
    if best_t != 0.0 {
        let u = best_t * (1.0 - 0.45 * s.ctr_risk) + if s.danger > 0.5 { 1.0 } else { 0.0 };
        o.push(plan(u, "reanimate", reanimate_go));
    }
    let ys = pl.hand.iter().any(|&c| g.db.get(c).tag(Tag::Yawg));
    let reans_gy: Vec<CardId> = pl.gy.iter().copied().filter(|&c| g.db.get(c).tag(Tag::Rean)).collect();
    if ys && !reans_gy.is_empty() && gy_bomb && !hand_has(g, p, |c| g.db.get(c).tag(Tag::Rean)) {
        let ch = min_by(&reans_gy, |c| g.db.get(c).cmc as f64).unwrap();
        let d = g.db.get(ch);
        if can_pay(g, p, 2 + d.generic, &format!("B{}", d.pips), false) {
            o.push(plan(7.5, "Yawgmoth's Will line", yawg_go));
        }
    }
    let te =
        pl.perms.iter().any(|&m| g.perm(m).cd.is_some_and(|c| g.db.get(c).tags.str(Tag::Fill) == Some("tortured")));
    if te
        && !gy_bomb
        && rean
        && pl.te_used != Some(pl.turns)
        && hand_has(g, p, |c| g.db.get(c).creature && g.db.get(c).bomb >= 6)
        && can_pay(g, p, 0, "B", false)
    {
        o.push(plan(6.0, "Tortured Existence discard", tortured_go));
    }
    let tut_in_hand = hand_has(g, p, |c| g.db.get(c).tags.str(Tag::Tut).is_some_and(|s| !s.is_empty()));
    if !gy_bomb
        && (rean || tut_in_hand)
        && pl.library.iter().any(|&c| g.db.get(c).creature && g.db.get(c).bomb >= 5)
        && payable_in_hand(g, p, |c| FILLERS.contains(&&*g.db.get(c).name))
    {
        o.push(plan(if rean { 6.3 } else { 3.0 }, "fill graveyard", fill_go));
    }
    if payable_in_hand(g, p, |c| g.db.get(c).tags.str(Tag::Tut) == Some("any"))
        && !(bomb_on_bf(g, p) && pl.turns < 6)
        && seph_tutor_target(g, p).is_some()
    {
        let missing = !bomb_on_bf(g, p) && (rean != gy_bomb);
        o.push(plan(if missing { 6.2 } else { 3.0 }, "tutor", tutor_go));
    }
    if hand_has(g, p, |c| g.db.get(c).tag(Tag::Avarice)) && can_pay(g, p, 2, "B", false) {
        o.push(plan(5.0, "Insatiable Avarice", avarice_go));
    }
    let mut hc: f64 = 0.0;
    if pl.cmd_in_zone {
        let (cg, cp) = cost_of(g, p, pl.cmd);
        if can_pay(g, p, cg, &cp, false) {
            hc = 8.0;
        }
    }
    for &c in &pl.hand {
        let d = g.db.get(c);
        if d.creature && d.bomb >= 4 && can_pay(g, p, d.generic, &d.pips, false) {
            hc = hc.max(seph_bval(g, p, c) - 1.0);
        }
    }
    if hc != 0.0 {
        o.push(plan(hc * (1.0 - 0.35 * s.ctr_risk), "hardcast bomb", hardcast_go));
    }
    let deserts = s.opps.iter().any(|&q| g.player(q).lands.iter().any(|&l| g.db.get(g.land(l).cd).tag(Tag::Desert)));
    if deserts
        && (rean || gy_bomb)
        && payable_in_hand(g, p, |c| {
            let t = &g.db.get(c).tags;
            t.str(Tag::Tgt) == Some("p") && t.str(Tag::Rem) == Some("destroy")
        })
    {
        o.push(plan(7.5, "Trophy the Scavenger Grounds", trophy_go));
    }
    let clamp_fod = pl.perms.iter().any(|&m| {
        let x = g.perm(m);
        g.is_creature(m)
            && etgh(g, m) == 1
            && (x.token
                || x.cd.is_some_and(|c| matches!(g.db.get(c).tags.str(Tag::Fill), Some("stitcher" | "wayfinder"))))
    });
    if has(g, p, Tag::Clamp) && can_pay(g, p, 1, "", false) && pl.clamp_n < 2 && clamp_fod {
        o.push(plan(4.0, "Skullclamp", clamp_go));
    }
    let dispute_fod = pl.perms.iter().any(|&m| {
        let x = g.perm(m);
        g.is_creature(m) && (x.token || x.cd.is_some_and(|c| g.db.get(c).tags.str(Tag::Fill) == Some("stitcher")))
    });
    if payable_in_hand(g, p, |c| g.db.get(c).tags.str(Tag::Fill) == Some("dispute"))
        && (pl.treasures > 0 || dispute_fod)
    {
        o.push(plan(3.5, "Deadly Dispute", dispute_go));
    }
    let loose_boots = find(g, p, Tag::Prot).iter().any(|&e| {
        g.perm(e).cd.is_some_and(|c| g.db.get(c).tags.str(Tag::Prot) == Some("boots")) && g.perm(e).attached.is_none()
    });
    let bomb_out = pl
        .perms
        .iter()
        .any(|&m| g.is_creature(m) && g.perm(m).cd.is_some_and(|c| g.db.get(c).bomb >= 6) && !untargetable(g, m));
    if can_pay(g, p, 1, "", false) && loose_boots && bomb_out {
        o.push(plan(3.0 + 4.0 * removal_risk(g, p), "equip Boots", boots_go));
    }
    Ok(o)
}

// ------------------------------------------------------------------ the plays
fn reanimate_go(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    seph_reanimate(g, p)
}

/// ais.seph_reanimate: the cheapest reanimation spell with a target, cast at the best target
pub fn seph_reanimate(g: &mut Game, p: PlayerId) -> Res<bool> {
    let mut opts = rean_options(g, p);
    if opts.is_empty() {
        return Ok(false);
    }
    opts.sort_by_key(|o| o.2 + o.3.chars().count() as u32);
    for (c, zone, cg, cp) in opts {
        let kind = g.db.get(c).tags.str(Tag::Rean).unwrap_or("").to_string();
        let mut tg = rean_targets(g, p, &kind);
        if kind == "reanimate" {
            let life = g.player(p).life;
            tg.retain(|x| life - g.db.get(x.1).cmc as i32 >= 12);
        }
        let Some(&(val, cd, src)) = tg.first() else { continue };
        if !can_pay(g, p, cg, &cp, false) {
            continue;
        }
        pay(g, p, cg, &cp, false)?;
        if zone == "hand" {
            remove_from_hand(g, p, c);
        } else {
            let gy = &mut g.player_mut(p).gy;
            if let Some(i) = gy.iter().position(|&x| x == c) {
                gy.remove(i);
            }
        }
        if zone == "fbsac" {
            let mut fod: Vec<(f64, PermId)> = g
                .player(p)
                .perms
                .iter()
                .copied()
                .filter(|&m| {
                    let x = g.perm(m);
                    g.is_creature(m) && (x.token || x.cd.is_none_or(|cd| g.db.get(cd).bomb == 0))
                })
                .map(|m| (pval(g, m), m))
                .collect();
            fod.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
            for (_, m) in fod.into_iter().take(3) {
                die(g, m, "sac")?;
            }
        }
        g.player_mut(p).spells_this_turn += 1;
        on_cast(g, p, c)?;
        if g.over || !g.player(p).alive {
            return Ok(true);
        }
        let pl = g.player_mut(p);
        pl.stat("rean_cast", 1);
        pl.stat("spells_cast", 1);
        pl.cast_names.insert(c);
        if g.log.is_some() {
            let from = if src != p { format!(" from {}'s graveyard", g.player(src).name) } else { String::new() };
            crate::glog!(g, "  Sephiroth casts {} targeting {}{from}", g.db.get(c).name, g.db.get(cd).name);
        }
        let to_gy = zone == "hand";
        let dest = |g: &mut Game| {
            let pl = g.player_mut(p);
            if to_gy { pl.gy.push(c) } else { pl.exile.push(c) }
        };
        if !counter_window(g, p, c, val, vec![])? {
            dest(g);
            return Ok(true);
        }
        if super::decks::sauron_grounds_response(g, p, val)?
            || (!g.hooks.is_empty() && crate::cardcode::gy_response(g, p, val, src)?)
        {
            dest(g);
            return Ok(true);
        }
        super::seph_rean_resolve(g, p, c, &Ctx { rean_target: Some(cd), rean_src: Some(src), ..Ctx::default() })?;
        dest(g);
        check_state(g)?;
        return Ok(true);
    }
    Ok(false)
}

fn fill_go(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    seph_fill(g, p)
}

/// ais.seph_fill: Entomb, Buried Alive, Unmarked Grave or Grisly Salvage, with a bomb to bin and a way to use it
pub fn seph_fill(g: &mut Game, p: PlayerId) -> Res<bool> {
    if own_bomb_in_gy(g, p) {
        return Ok(false);
    }
    let tut = hand_has(g, p, |c| g.db.get(c).tags.str(Tag::Tut).is_some_and(|s| !s.is_empty()));
    if !(has_rean_access(g, p) || tut) {
        return Ok(false);
    }
    if !g.player(p).library.iter().any(|&c| g.db.get(c).creature && g.db.get(c).bomb >= 5) {
        return Ok(false);
    }
    for name in FILLERS {
        for c in g.player(p).hand.clone() {
            let d = g.db.get(c);
            if &*d.name == name && can_pay(g, p, d.generic, &d.pips, false) {
                let (gn, pips) = (d.generic, d.pips.to_string());
                pay(g, p, gn, &pips, false)?;
                cast_card(g, p, c, "hand", Ctx::default())?;
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn tortured_go(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    seph_tortured(g, p)
}

/// ais.seph_tortured: Tortured Existence bins a bomb for the reanimation spells and returns the best creature that
/// isn't a reanimation target, once per turn
pub fn seph_tortured(g: &mut Game, p: PlayerId) -> Res<bool> {
    let te = g
        .player(p)
        .perms
        .iter()
        .any(|&m| g.perm(m).cd.is_some_and(|c| g.db.get(c).tags.str(Tag::Fill) == Some("tortured")));
    if !te || blocked(g, p, "Tortured Existence") {
        return Ok(false);
    }
    if own_bomb_in_gy(g, p) || g.player(p).te_used == Some(g.player(p).turns) {
        return Ok(false);
    }
    let bombs: Vec<CardId> =
        g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).creature && g.db.get(c).bomb >= 6).collect();
    if bombs.is_empty() || !can_pay(g, p, 0, "B", false) || !has_rean_access(g, p) {
        return Ok(false);
    }
    pay(g, p, 0, "B", false)?;
    let b = first_max(&bombs, |c| seph_bval(g, p, c)).unwrap();
    discard_cards(g, p, &[b])?;
    let t = g.player(p).turns;
    g.player_mut(p).te_used = Some(t);
    let small: Vec<CardId> =
        g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature && g.db.get(c).bomb == 0 && c != b).collect();
    if let Some(x) = first_max(&small, |c| card_worth(g, p, c, false)) {
        let pl = g.player_mut(p);
        if let Some(i) = pl.gy.iter().position(|&y| y == x) {
            pl.gy.remove(i);
        }
        pl.hand.push(x);
    }
    Ok(true)
}

fn tutor_go(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    seph_tutor(g, p)
}

/// ais.seph_tutor: the cheapest tutor that finds anything (Diabolic Intent sacrifices a spare creature)
pub fn seph_tutor(g: &mut Game, p: PlayerId) -> Res<bool> {
    let mut tuts: Vec<CardId> =
        g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).tags.str(Tag::Tut) == Some("any")).collect();
    if tuts.is_empty() {
        return Ok(false);
    }
    if bomb_on_bf(g, p) && g.player(p).turns < 6 {
        return Ok(false);
    }
    if seph_tutor_target(g, p).is_none() {
        return Ok(false);
    }
    tuts.sort_by_key(|&c| g.db.get(c).cmc);
    for c in tuts {
        let d = g.db.get(c);
        let (gn, pips) = (d.generic, d.pips.to_string());
        if !can_pay(g, p, gn, &pips, false) {
            continue;
        }
        if d.tag(Tag::Needsac) {
            let cmd = g.player(p).cmd;
            let fod: Vec<PermId> = g
                .player(p)
                .perms
                .iter()
                .copied()
                .filter(|&m| {
                    let x = g.perm(m);
                    g.is_creature(m) && (x.token || x.cd.is_some_and(|cd| g.db.get(cd).bomb == 0 && cd != cmd))
                })
                .collect();
            if fod.is_empty() {
                continue;
            }
            pay(g, p, gn, &pips, false)?;
            let f = min_by(&fod, |x| pval(g, x)).unwrap();
            die(g, f, "sac")?;
        } else {
            pay(g, p, gn, &pips, false)?;
        }
        cast_card(g, p, c, "hand", Ctx::default())?;
        return Ok(true);
    }
    Ok(false)
}

fn avarice_go(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    seph_avarice(g, p)
}

/// ais.seph_avarice: Insatiable Avarice (spree): +{2} tutor to top, +{B}{B} target player draws 3 and loses 3. Both
/// modes (5 mana) = tutor then draw it with two extra cards.
pub fn seph_avarice(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(c) = g.player(p).hand.iter().copied().find(|&c| g.db.get(c).tag(Tag::Avarice)) else { return Ok(false) };
    let mut want = seph_tutor_target(g, p);
    if bomb_on_bf(g, p) && total_mana(g, p, false) < 7 {
        want = None;
    }
    let life = g.player(p).life;
    let mode = if want.is_some() && life > 12 && can_pay(g, p, 2, "BBB", false) {
        "both"
    } else if want.is_some() && can_pay(g, p, 2, "B", false) {
        "tutor"
    } else if want.is_none() && life > 15 && g.player(p).turns >= 4 && can_pay(g, p, 0, "BBB", false) {
        "draw"
    } else {
        return Ok(false);
    };
    let (gn, pips) = match mode {
        "both" => (2, "BBB"),
        "tutor" => (2, "B"),
        _ => (0, "BBB"),
    };
    pay(g, p, gn, pips, false)?;
    remove_from_hand(g, p, c);
    g.player_mut(p).spells_this_turn += 1;
    on_cast(g, p, c)?;
    if g.over || !g.player(p).alive {
        return Ok(true);
    }
    if mode != "draw" {
        let hit = want.and_then(|w| g.db.id(w)).filter(|h| g.player(p).library.contains(h));
        if let Some(h) = hit {
            let lib = &mut g.player_mut(p).library;
            let i = lib.iter().position(|&x| x == h).unwrap();
            lib.remove(i);
            if let Some(a) = agent_for(g, p) {
                agent_take(g, a, p, h);
                shuffle_library(g, p);
            } else {
                shuffle_library(g, p);
                let pl = g.player_mut(p);
                pl.library.push(h);
                pl.stat("tutored", 1);
            }
        }
    }
    if mode != "tutor" {
        draw(g, p, 3, false)?;
        lose_life(g, p, 3, Some(p), "other", None)?;
    }
    let pl = g.player_mut(p);
    pl.gy.push(c);
    pl.stat(intern(&format!("avarice_{mode}")), 1);
    check_state(g)?;
    Ok(true)
}

fn hardcast_go(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    seph_hardcast(g, p)
}

/// ais.seph_hardcast: the commander from the command zone, else the best bomb in hand
pub fn seph_hardcast(g: &mut Game, p: PlayerId) -> Res<bool> {
    if g.player(p).cmd_in_zone {
        let cmd = g.player(p).cmd;
        let (cg, cp) = cost_of(g, p, cmd);
        if can_pay(g, p, cg, &cp, false) {
            pay(g, p, cg, &cp, false)?;
            if cast_card(g, p, cmd, "cmd", Ctx::default())? {
                note_bomb(g, p, cmd, false);
            }
            return Ok(true);
        }
    }
    let mut bombs: Vec<(f64, CardId)> = g
        .player(p)
        .hand
        .iter()
        .copied()
        .filter(|&c| g.db.get(c).creature && g.db.get(c).bomb >= 4)
        .map(|c| (-seph_bval(g, p, c), c))
        .collect();
    bombs.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    for (_, c) in bombs {
        let d = g.db.get(c);
        let (gn, pips) = (d.generic, d.pips.to_string());
        if can_pay(g, p, gn, &pips, false) {
            pay(g, p, gn, &pips, false)?;
            if cast_card(g, p, c, "hand", Ctx::default())? {
                note_bomb(g, p, c, false);
            }
            return Ok(true);
        }
    }
    Ok(false)
}

fn yawg_go(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    seph_yawg(g, p)
}

/// ais.seph_yawg: Yawgmoth's Will to recast a reanimation spell from the graveyard at a bomb there
pub fn seph_yawg(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(y) = g.player(p).hand.iter().copied().find(|&c| g.db.get(c).tag(Tag::Yawg)) else { return Ok(false) };
    let reans: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).tag(Tag::Rean)).collect();
    if reans.is_empty() || !own_bomb_in_gy(g, p) {
        return Ok(false);
    }
    if hand_has(g, p, |c| g.db.get(c).tag(Tag::Rean)) {
        return Ok(false);
    }
    let ch = min_by(&reans, |c| g.db.get(c).cmc as f64).unwrap();
    let d = g.db.get(ch);
    if !can_pay(g, p, 2 + d.generic, &format!("B{}", d.pips), false) {
        return Ok(false);
    }
    pay(g, p, 2, "B", false)?;
    cast_card(g, p, y, "hand", Ctx::default())?;
    Ok(true)
}

fn trophy_go(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    seph_trophy_grounds(g, p)
}

/// ais.seph_trophy_grounds: destroy an opponent's Scavenger Grounds (a Desert) before reanimating
pub fn seph_trophy_grounds(g: &mut Game, p: PlayerId) -> Res<bool> {
    if !(own_bomb_in_gy(g, p) || has_rean_access(g, p)) {
        return Ok(false);
    }
    for q in g.opps(p).collect::<Vec<_>>() {
        let Some(l) = g.player(q).lands.iter().copied().find(|&l| g.db.get(g.land(l).cd).tag(Tag::Desert)) else {
            continue;
        };
        for c in g.player(p).hand.clone() {
            let d = g.db.get(c);
            let t = &d.tags;
            if t.str(Tag::Tgt) == Some("p")
                && t.str(Tag::Rem) == Some("destroy")
                && can_pay(g, p, d.generic, &d.pips, false)
            {
                if !pay_card(g, p, c, 0)? {
                    return Ok(true);
                }
                if g.player(q).lands.contains(&l) {
                    crate::engine::turn::remove_land(g, q, l);
                    let cd = g.land(l).cd;
                    g.player_mut(q).gy.push(cd);
                    land_ramp(g, q, 1, true)?;
                }
                g.player_mut(p).stat("trophy_grounds", 1);
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn clamp_go(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    seph_clamp(g, p)
}

/// ais.seph_clamp: Skullclamp on a 1-toughness body (a token, Stitcher's Supplier, a late mana dork): draw two, twice a
/// turn at most
pub fn seph_clamp(g: &mut Game, p: PlayerId) -> Res<bool> {
    if !has(g, p, Tag::Clamp) || blocked(g, p, "Skullclamp") {
        return Ok(false);
    }
    let t = g.player(p).turns;
    if g.player(p).clamp_t != Some(t) {
        let pl = g.player_mut(p);
        pl.clamp_t = Some(t);
        pl.clamp_n = 0;
    }
    if g.player(p).clamp_n >= 2 {
        return Ok(false);
    }
    let fod = g.player(p).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        g.is_creature(m)
            && !x.phased
            && etgh(g, m) == 1
            && (x.token
                || x.cd.is_some_and(|c| {
                    let tg = &g.db.get(c).tags;
                    matches!(tg.str(Tag::Fill), Some("stitcher" | "wayfinder")) || (t >= 7 && tg.has(Tag::Dork))
                }))
    });
    let Some(f) = fod else { return Ok(false) };
    if !can_pay(g, p, 1, "", false) {
        return Ok(false);
    }
    pay(g, p, 1, "", false)?;
    g.player_mut(p).clamp_n += 1;
    die(g, f, "sac")?;
    draw(g, p, 2, false)?;
    Ok(true)
}

fn dispute_go(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    seph_dispute(g, p)
}

/// ais.seph_dispute: Deadly Dispute, sacrificing a token, Stitcher's Supplier or a Treasure
pub fn seph_dispute(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(c) = g.player(p).hand.iter().copied().find(|&c| g.db.get(c).tags.str(Tag::Fill) == Some("dispute")) else {
        return Ok(false);
    };
    let fod = g.player(p).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        g.is_creature(m)
            && !x.phased
            && (x.token
                || x.cd.is_some_and(|cd| {
                    let d = g.db.get(cd);
                    d.bomb == 0 && d.tags.str(Tag::Fill) == Some("stitcher")
                }))
    });
    if fod.is_none() && g.player(p).treasures == 0 {
        return Ok(false);
    }
    let d = g.db.get(c);
    let (gn, pips) = (d.generic, d.pips.to_string());
    if !can_pay(g, p, gn, &pips, false) {
        return Ok(false);
    }
    pay(g, p, gn, &pips, false)?;
    if let Some(f) = fod {
        die(g, f, "sac")?;
    } else if g.player(p).treasures > 0 {
        g.player_mut(p).treasures -= 1;
    }
    cast_card(g, p, c, "hand", Ctx::default())?;
    Ok(true)
}

/// ais.boots_cost: Lightning Greaves equips for {0}, Swiftfoot Boots for {1}
pub fn boots_cost(g: &Game, e: PermId) -> u32 {
    if card_name(g, e) == Some("Lightning Greaves") { 0 } else { 1 }
}

fn boots_go(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    seph_boots(g, p)
}

/// ais.seph_boots: an unattached Lightning Greaves or Swiftfoot Boots onto the best bomb
pub fn seph_boots(g: &mut Game, p: PlayerId) -> Res<bool> {
    for e in find(g, p, Tag::Prot) {
        let is_boots = g.perm(e).cd.is_some_and(|c| g.db.get(c).tags.str(Tag::Prot) == Some("boots"));
        if !is_boots || g.perm(e).attached.is_some() {
            continue;
        }
        let n = g.perm(e).name;
        if blocked(g, p, n) {
            continue;
        }
        let bombs: Vec<PermId> = g
            .player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| g.is_creature(m) && g.perm(m).cd.is_some_and(|c| g.db.get(c).bomb >= 6) && !untargetable(g, m))
            .collect();
        let cost = boots_cost(g, e);
        if !bombs.is_empty() && can_pay(g, p, cost, "", false) {
            let t = max_by(&bombs, |x| pval(g, x)).unwrap();
            return equip_to(g, p, e, t, cost);
        }
    }
    Ok(false)
}

// ------------------------------------------------------------------ the draw step and the end step
/// ais.seph_dredge: Stinkweed Imp dredges 5 instead of the draw when a reanimation spell is available and no bomb is
/// out or in the graveyard
pub fn seph_dredge(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(imp) = g.player(p).gy.iter().copied().find(|&c| g.db.get(c).tag(Tag::Dredge)) else { return Ok(false) };
    if g.player(p).library.len() < 6 {
        return Ok(false);
    }
    if own_bomb_in_gy(g, p) || bomb_on_bf(g, p) || !has_rean_access(g, p) {
        return Ok(false);
    }
    let pl = g.player_mut(p);
    let i = pl.gy.iter().position(|&x| x == imp).unwrap();
    pl.gy.remove(i);
    pl.hand.push(imp);
    mill(g, p, 5)?;
    g.player_mut(p).stat("dredged", 1);
    Ok(true)
}

/// ais.end_step's Sephiroth milestones (reports): the turn a bomb was first out, and each turn whether interaction
/// was held (int_held) and its mana kept up (int_up). Python keeps the last two as {turn: bool} dicts; here each is a
/// milestone named for its turn, 'int_up:5', set to 1 or 0.
pub fn end_milestones(g: &mut Game, p: PlayerId) {
    let t = g.player(p).turns;
    if bomb_on_bf(g, p) {
        g.player_mut(p).milestone.entry("bomb_by").or_insert(t);
    }
    let held: Vec<CardId> = g
        .player(p)
        .hand
        .iter()
        .copied()
        .filter(|&c| g.db.get(c).tag(Tag::Ctr) || g.db.get(c).tags.str(Tag::Prot) == Some("hi"))
        .collect();
    let up = held.iter().any(|&c| can_pay(g, p, g.db.get(c).generic, &g.db.get(c).pips, false));
    let pl = g.player_mut(p);
    pl.milestone.insert(intern(&format!("int_up:{t}")), up as u32);
    pl.milestone.insert(intern(&format!("int_held:{t}")), !held.is_empty() as u32);
}

// ------------------------------------------------------------------ protection and answers to wipes
/// ais.protect_response for Sephiroth under Conqueror's Flail (no spells): a free sacrifice outlet still saves a bomb
/// from exile, bounce or tuck for the reanimation spells
pub fn protect_silenced(g: &mut Game, owner: PlayerId, m: PermId, kind: Sym, v: f64) -> Res<bool> {
    if matches!(kind, "exile" | "bounce" | "tuck") && g.is_creature(m) && free_sac(g, owner) && v >= 6.0 {
        g.player_mut(owner).stat("sac_saves", 1);
        seph_sac(g, owner, m)?;
        return Ok(true);
    }
    Ok(false)
}

/// ais.protect_response for Sephiroth: Ephemerate or Restoration Angel blink a threatened creature; for a bomb,
/// Heroic Intervention, Galadriel's Dismissal (phasing), or a free sacrifice outlet against exile, bounce or tuck
pub fn protect(g: &mut Game, owner: PlayerId, m: PermId, kind: Sym, v: f64) -> Res<bool> {
    let creature = g.is_creature(m);
    if v >= 5.0
        && creature
        && !g.perm(m).token
        && !matches!(kind, "edict" | "wipe")
        && mine::ephemerate_cast(g, owner, m, "protection")?
    {
        return Ok(true);
    }
    if v >= 5.0
        && creature
        && (matches!(kind, "destroy" | "exile" | "bounce" | "tuck") || kind.starts_with("dmg"))
        && mine::resto_cast(g, owner, m, "protection")?
    {
        return Ok(true);
    }
    if v >= 6.0 && creature {
        let hi = g.player(owner).hand.iter().copied().find(|&c| g.db.get(c).tags.str(Tag::Prot) == Some("hi"));
        if let Some(c) = hi
            && kind != "edict"
            && can_pay(g, owner, 1, "G", false)
        {
            if pay_card(g, owner, c, 0)? {
                g.player_mut(owner).stat("hi_used", 1);
                return Ok(true);
            }
            return Ok(false);
        }
        let gd = g.db.id("Galadriel's Dismissal").filter(|c| g.player(owner).hand.contains(c));
        if let Some(c) = gd
            && can_pay(g, owner, 0, "W", false)
            && pay_card(g, owner, c, 0)?
        {
            g.perm_mut(m).phased = true; // phases out: safe from anything
            g.player_mut(owner).stat("phase_saves", 1);
            return Ok(true);
        }
        if matches!(kind, "exile" | "bounce" | "tuck") && free_sac(g, owner) && !g.perm(m).token {
            g.player_mut(owner).stat("sac_saves", 1);
            seph_sac(g, owner, m)?;
            return Ok(true);
        }
    }
    Ok(false)
}

/// ais.wipe_response for Sephiroth (the wipe costs it `loss`, 6 or more): Heroic Intervention against destruction,
/// Galadriel's Dismissal kicked (every creature phases out), Teferi's Protection against a big loss, or sacrificing
/// the bombs to an exile wipe (they come back from the graveyard)
pub fn wipe_response(g: &mut Game, q: PlayerId, kind: Sym, loss: f64) -> Res<Option<Sym>> {
    if matches!(kind, "destroy" | "dmg13" | "austere" | "nib") {
        let hi = g.player(q).hand.iter().copied().find(|&c| g.db.get(c).tags.str(Tag::Prot) == Some("hi"));
        if let Some(c) = hi
            && can_pay(g, q, 1, "G", false)
            && pay_card(g, q, c, 0)?
        {
            g.player_mut(q).stat("hi_used", 1);
            return Ok(Some("indes"));
        }
    }
    let gd = g.db.id("Galadriel's Dismissal").filter(|c| g.player(q).hand.contains(c));
    if let Some(c) = gd
        && can_pay(g, q, 2, "WW", false)
        && pay_card(g, q, c, 2)?
    {
        // kicked: every creature you control
        for m in g.player(q).perms.clone() {
            if g.is_creature(m) {
                g.perm_mut(m).phased = true;
            }
        }
        g.player_mut(q).stat("phase_saves", 1);
        return Ok(Some("all"));
    }
    if loss >= 10.0 && cast_teferis_protection(g, q, 8.0)? {
        // everything phases out (a real wipe of a real board)
        g.player_mut(q).stat("phase_saves", 1);
        return Ok(Some("all"));
    }
    if matches!(kind, "exile" | "rift" | "rebuke") && free_sac(g, q) {
        let cmd = g.player(q).cmd;
        for m in g.player(q).perms.clone() {
            let x = g.perm(m);
            if g.is_creature(m) && !x.token && x.cd.is_some_and(|c| g.db.get(c).bomb != 0 && c != cmd) {
                g.player_mut(q).stat("sac_saves", 1);
                seph_sac(g, q, m)?;
            }
        }
    }
    Ok(None)
}
