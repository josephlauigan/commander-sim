//! The yawgmoth candidate deck (decklists/JD/candidates/yawgmoth.md): mono-black aristocrats under Yawgmoth, Thran
//! Physician. Its card code and AI plan. Every card below is Full in `cardgen audit --mine` (the verdicts are in
//! data/card_overrides.toml); tests are tests/cards_yawgmoth.rs.
//!
//! - **Yawgmoth, Thran Physician**: pay 1 life, sacrifice another creature: a -1/-1 counter on up to one target
//!   creature and draw a card; {B}{B}, discard a card: proliferate (`common::proliferate`: +1/+1 counters, loyalty,
//!   lore counters of your Sagas, and the -1/-1 counters on opposing creatures, which kill the ones at 0 toughness).
//!   Protection from Humans is `rules2::prot_vs` (no damage from Humans) and `rules2::prot_unblockable` (Humans can't
//!   block it). Not modeled: a Human's *ability* can't target it (the engine doesn't know an ability's source
//!   creature types when it picks targets; no deck in the pools has a Human that targets creatures with abilities in
//!   a way that matters). The same card is the opponent pool's Yawgmoth deck; for that deck (`key` other than
//!   `yawgmoth`) the old t5 code runs unchanged.
//! - **Sagas**: Braids's Frightful Return (read ahead: the chapter it starts on is chosen as it enters, `braids_pick`;
//!   lore counters come after the draw step, from Proliferate too).
//! - **Returning creatures**: Presumed Dead and Perigee Beckoner mark a creature with a keyword for the turn
//!   (`ret_dies_*` in `eot_kw`, cleared at end of turn); `zones::die_rest` calls `returns_when_dies`, which puts the card
//!   back (tapped, or suspected). Suspect is `DataKey::Suspect`: `dsl::has_kw` gives menace and "cant_block".
//! - **Reanimation**: Rewrite Regrets (with its Empower Jace 2), It Doesn't Add Up, Phyrexian Delver, Cheerful
//!   Osteomancer's Raise Dead copy, Soul Enervation's drain (a state check comparing the creature cards in the
//!   graveyard with the last look).
//! - **Transformers**: Bloodsworn Squire (Bloodsworn Knight) and Catapult Fodder (Catapult Captain) stay one permanent
//!   with `DataKey::Flipped` set; `self_pt` gives the back face's size.
//! - **The AI plan**: `yawgmoth_prio` (cast order), `yawgmoth_options` (Presumed Dead, warp), and Yawgmoth's own
//!   `options`/`defend`: fodder first, life kept above 10, the -1/-1 counters on what they kill or shrink the most.

use crate::ai::pool::generic_prio;
use crate::cards::{CardDb, Colors};
use crate::engine::cast::{cast_card, discard_worst};
use crate::engine::life::{gain, lose_life};
use crate::engine::mana::{can_pay, cost_of, pay};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::values::{epow, etgh, pval, threat, untargetable};
use crate::engine::zones::{
    Enter, Tokens, die, discard_cards, draw, enter, make_tokens, max_by, min_by, minus_counter, returns_left,
};
use crate::flow::Res;
use crate::hooks::{Action, Assign, Event, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Perm, Val};
use crate::sym::{Sym, intern};

pub const YAWGMOTH: &str = "Yawgmoth, Thran Physician";
const BRAIDS: &str = "Braids's Frightful Return";
const ENERVATION: &str = "Soul Enervation";
const SQUIRE: &str = "Bloodsworn Squire";
const FODDER: &str = "Catapult Fodder";
const OSTEOMANCER: &str = "Cheerful Osteomancer";
const ZOLA: &str = "Arnim Zola, Bio-Fanatic";
const BECKONER: &str = "Perigee Beckoner";
const DELVER: &str = "Phyrexian Delver";

// ------------------------------------------------------------------ helpers
fn name_of(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

/// m is on the battlefield under p's control
fn on(g: &Game, p: PlayerId, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && x.owner == p && !x.phased
}

fn my_creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect()
}

fn opp_creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && !g.perm(m).phased)
        .collect()
}

/// creature cards in p's graveyard
fn gy_creatures(g: &Game, p: PlayerId) -> Vec<CardId> {
    g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature).collect()
}

fn flipped(g: &Game, m: PermId) -> bool {
    g.perm(m).data.truthy(DataKey::Flipped)
}

fn is_dead_weight(g: &Game, m: PermId) -> bool {
    // an effectively dead permanent (0 toughness cards are handled by the state check)
    etgh(g, m) <= 0
}

/// what a creature card in the graveyard is worth to bring back: the table's bomb value or its size, its mana value,
/// and the value of its own enter / death triggers
fn cv(g: &Game, c: CardId) -> f64 {
    let d = g.db.get(c);
    let mut v = if d.bomb != 0 { d.bomb as f64 } else { 1.0 + 0.5 * d.pow as f64 };
    v += 0.15 * d.cmc as f64;
    v += match &*d.name {
        DELVER => 2.5,
        "Dread Drone" => 1.8,
        "Blood Artist" => 1.2,
        "Accursed Marauder" => 1.0,
        BECKONER => 0.8,
        "Phyrexian Rager" | "Burglar Rat" | "Undercity Dire Rat" | "Death Cultist" | "Festering Goblin" => 0.7,
        "Carrion Feeder" | "Sangrophage" => 0.4,
        _ => 0.0,
    };
    v
}

/// the best creature card of p's graveyard that `pred` accepts
fn best_gy(g: &Game, p: PlayerId, pred: impl Fn(&Game, CardId) -> bool) -> Option<CardId> {
    let cs: Vec<CardId> = gy_creatures(g, p).into_iter().filter(|&c| pred(g, c)).collect();
    max_by(&cs, |c| cv(g, c))
}

/// a graveyard card enters the battlefield under p's control. Returns the permanent.
fn reanimate(g: &mut Game, p: PlayerId, c: CardId) -> Res<Option<PermId>> {
    let Some(i) = g.player(p).gy.iter().position(|&x| x == c) else { return Ok(None) };
    g.player_mut(p).gy.remove(i);
    g.player_mut(p).stat("rean_resolved", 1);
    let m = enter(g, p, c, Enter::default())?;
    crate::glog!(g, "    {} returns to the battlefield", name_of(g, c));
    Ok(Some(m))
}

/// suspect m: menace, and it can't block (until it leaves the battlefield)
fn suspect(g: &mut Game, m: PermId) {
    g.perm_mut(m).data.set(DataKey::Suspect, Val::Bool(true));
    crate::glog!(g, "    {} is suspected", g.perm(m).name);
}

/// -n/-n until end of turn on m (a lethal one goes through the removal code)
fn shrink(g: &mut Game, p: PlayerId, m: PermId, n: i32) -> Res {
    if !g.perm(m).on_bf {
        return Ok(());
    }
    if etgh(g, m) <= n {
        return apply_removal(g, Some(p), m, &format!("shrink{n}"), None);
    }
    let x = g.perm_mut(m);
    x.eot_pt = (x.eot_pt.0 - n, x.eot_pt.1 - n);
    Ok(())
}

/// what sacrificing m costs p: its worth, less what its death itself pays (tokens are nearly free; a Spawn also
/// makes mana, so it costs a little more)
fn sac_cost(g: &Game, p: PlayerId, m: PermId) -> f64 {
    let x = g.perm(m);
    let mut c = if x.token {
        if x.ttypes.contains(&"spawn") { 0.45 } else { 0.25 + 0.25 * epow(g, m).max(0) as f64 }
    } else {
        pval(g, m)
    };
    if let Some(cd) = x.cd {
        let d = g.db.get(cd);
        if d.kws.iter().any(|k| &**k == "undying") && x.plus <= 0 {
            c = c.min(0.3);
        }
        if matches!(&*d.name, "Festering Goblin" | "Blood Artist" | "Dread Drone" | "Phyrexian Rager") {
            c -= 0.5; // pays on the way out
        }
    }
    // the death triggers on the board (Blood Artist ...)
    (c - 0.5 * crate::impls::common::death_value(g, p, Some(m))).max(0.0)
}

fn can_sac(g: &Game, src: PermId, m: PermId) -> bool {
    m != src && g.is_creature(m) && !g.perm(m).is_cmd && !g.perm(m).phased && !crate::engine::values::no_sac(g, m)
}

// ------------------------------------------------------------------ Yawgmoth, Thran Physician
/// the creature Yawgmoth's counter should go on: (target, kills it, value). Toughness 1 creatures first (the highest
/// worth), then the one the counter shrinks the most.
fn yawg_target(g: &Game, p: PlayerId) -> Option<(PermId, bool, f64)> {
    let mut best: Option<(PermId, bool, f64)> = None;
    for m in opp_creatures(g, p) {
        if untargetable(g, m) {
            continue;
        }
        let t = etgh(g, m);
        let kills = t <= 1;
        let v = if kills {
            2.0 + pval(g, m)
        } else {
            // the counter stays: worth more on a threat with small toughness
            0.8 * pval(g, m) / t.max(1) as f64
        };
        if (kills || v >= 0.6) && best.is_none_or(|b| v > b.2) {
            best = Some((m, kills, v));
        }
    }
    best
}

/// Yawgmoth's ability, resolved: the counter on the target, the draw
fn yawg_resolve(g: &mut Game, src: PermId, p: PlayerId) -> Res {
    if let Some((x, _, _)) = yawg_target(g, p).filter(|_| g.perm(src).on_bf || true) {
        if minus_counter(g, x, 1) {
            crate::glog!(g, "    -1/-1 counter on {}", g.perm(x).name);
            if g.perm(x).on_bf && is_dead_weight(g, x) {
                die(g, x, "sba")?;
            }
        }
    }
    draw(g, p, 1, false)
}

/// the activation: pay 1 life and sacrifice m, then (once the ability has had its window) the counter and the draw
fn yawg_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let m = PermId(arg as u32);
    if !on(g, p, src) || !on(g, p, m) || !can_sac(g, src, m) || g.player(p).life <= 1 {
        return Ok(false);
    }
    crate::glog!(g, "  {} sacrifices {} to Yawgmoth", pname(g, p), g.perm(m).name);
    lose_life(g, p, 1, Some(p), "other", None)?;
    die(g, m, "sac")?;
    if !g.player(p).alive {
        return Ok(true);
    }
    if !ability_window(g, p, Some(src), "a -1/-1 counter, draw a card", Some(3.0), None)? {
        return Ok(true);
    }
    yawg_resolve(g, src, p)?;
    Ok(true)
}

/// Yawgmoth's options in your main phases: the cheapest fodder when the card (and the counter) is worth more than
/// the creature and the life; proliferate when it kills something or grows a threat
pub fn yawg_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.player(p).key != "yawgmoth" {
        return crate::impls::t5::yawg(g, src, p, post); // the opponent pool's Yawgmoth deck
    }
    let mut out = vec![];
    if g.perm(src).owner != p || g.perm(src).phased {
        return Ok(out);
    }
    let life = g.player(p).life;
    let tgt = yawg_target(g, p);
    let kill = tgt.is_some_and(|t| t.1);
    let floor = if kill && tgt.is_some_and(|t| t.2 >= 4.0) { 6 } else { 10 };
    if life > floor && !g.player(p).library.is_empty() {
        let fod: Vec<PermId> = my_creatures(g, p).into_iter().filter(|&m| can_sac(g, src, m)).collect();
        // the draw, the counter, and what the death pays
        let mut gain_ = 1.4 + if g.player(p).hand.len() <= 2 { 0.7 } else { 0.0 };
        if let Some((_, _, v)) = tgt {
            gain_ += v;
        }
        let busy = |m: PermId| {
            // a creature that can still attack this turn is worth more before combat
            let x = g.perm(m);
            if post == Some(false) && !x.sick && !x.tapped && epow(g, m) >= 2 { 0.8 } else { 0.0 }
        };
        if let Some(m) = min_by(&fod, |m| sac_cost(g, p, m) + busy(m)) {
            let u = gain_ - 0.8 * (sac_cost(g, p, m) + busy(m));
            if u > 0.5 && (post.is_some() || sac_cost(g, p, m) < 0.6) {
                out.push(Opt {
                    utility: u,
                    label: format!("Yawgmoth: sacrifice {}", g.perm(m).name),
                    act: Some(Action::Ability { src, f: yawg_go, arg: m.0 as i64 }),
                });
            }
        }
    }
    let v = proliferate_value(g, p);
    if post == Some(true) || post == Some(false) {
        let hand = g.player(p).hand.len();
        if v >= 2.0 && hand >= 1 && can_pay(g, p, 0, "BB", false) && (hand >= 2 || v >= 3.0) {
            out.push(Opt {
                utility: v - 0.5,
                label: "Yawgmoth: proliferate".into(),
                act: Some(Action::Ability { src, f: yawg_prol, arg: 0 }),
            });
        }
    }
    Ok(out)
}

/// what one proliferate is worth to p: opposing creatures it kills or shrinks, +1/+1 counters it grows, planeswalkers
/// and Sagas that gain
fn proliferate_value(g: &Game, p: PlayerId) -> f64 {
    let mut v = 0.0;
    for m in opp_creatures(g, p) {
        if g.perm(m).plus < 0 {
            if etgh(g, m) <= 1 {
                v += 2.5 + pval(g, m);
            } else {
                v += 0.4 + 0.2 * pval(g, m);
            }
        }
    }
    for &m in &g.player(p).perms {
        let x = g.perm(m);
        if x.phased || x.army {
            continue;
        }
        if x.plus > 0 && g.is_creature(m) {
            v += 0.5 + if x.is_cmd { 0.5 } else { 0.0 };
        }
        if x.loyalty.is_some_and(|l| l > 0) {
            v += 1.0;
        }
        if x.data.get(DataKey::Lore).is_some() && braids_wants(g, p, m) {
            v += 1.0;
        }
    }
    v
}

fn yawg_prol(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !on(g, p, src) || !can_pay(g, p, 0, "BB", false) || g.player(p).hand.is_empty() {
        return Ok(false);
    }
    pay(g, p, 0, "BB", false)?;
    discard_worst(g, p, 1)?;
    if ability_window(g, p, Some(src), "proliferate", Some(3.0), None)? {
        crate::impls::common::proliferate(g, p, 1)?;
    }
    Ok(true)
}

/// after blockers, when attacked: fodder that is dying anyway (a blocker that dies, a token) buys a card and a
/// -1/-1 counter on an attacker (it kills an X/1; otherwise it goes on the creature the sacrificed blocker faced)
fn yawg_defend_hook(g: &mut Game, src: Src, d: PlayerId, p: PlayerId, atk: &[PermId], assign: &mut Assign) -> Res {
    if g.player(d).key != "yawgmoth" {
        return crate::impls::t5::yawg_defend(g, src, d, p, atk, assign);
    }
    if g.perm(src).owner != d || !g.perm(src).on_bf || g.perm(src).phased {
        return Ok(());
    }
    for _ in 0..4 {
        if g.player(d).life <= 10 || g.player(d).library.is_empty() {
            return Ok(());
        }
        // fodder: a blocker that dies in the block without taking its attacker along, or cheap fodder
        let mut best: Option<(PermId, Option<PermId>, f64)> = None;
        for m in my_creatures(g, d) {
            if !can_sac(g, src, m) {
                continue;
            }
            let faced = assign.iter().find(|x| x.1 == m).map(|x| x.0).filter(|&a| on(g, p, a));
            let dying = faced.is_some_and(|a| epow(g, a) >= etgh(g, m) || g.perm(a).dt) && {
                let a = faced.unwrap();
                epow(g, m) < etgh(g, a)
            };
            let c = if dying { 0.1 } else { sac_cost(g, d, m) };
            if c < 0.7 && best.is_none_or(|b| c < b.2) {
                best = Some((m, faced, c));
            }
        }
        let Some((m, faced, _)) = best else { return Ok(()) };
        // the attacker the counter goes on
        let tg: Vec<PermId> =
            atk.iter().copied().filter(|&a| on(g, p, a) && g.is_creature(a) && !untargetable(g, a)).collect();
        let kill = max_by(&tg.iter().copied().filter(|&a| etgh(g, a) <= 1).collect::<Vec<_>>(), |a| pval(g, a));
        let tgt = kill.or(faced.filter(|&a| !untargetable(g, a))).or_else(|| max_by(&tg, |a| pval(g, a)));
        crate::glog!(g, "    {} sacrifices {} to Yawgmoth", pname(g, d), g.perm(m).name);
        lose_life(g, d, 1, Some(d), "other", None)?;
        die(g, m, "sac")?;
        assign.retain(|x| x.1 != m);
        if let Some(x) = tgt
            && on(g, p, x)
            && minus_counter(g, x, 1)
        {
            crate::glog!(g, "    -1/-1 counter on {}", g.perm(x).name);
            if is_dead_weight(g, x) {
                die(g, x, "sba")?;
            }
        }
        draw(g, d, 1, false)?;
        if g.over || !g.player(d).alive {
            return Ok(());
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ returning creatures (die_rest) and suspect
/// `zones::die_rest`: a creature that died with Presumed Dead's or Perigee Beckoner's grant on it returns to the
/// battlefield under its owner's control (tapped, or suspected). `x` is the permanent as it died. True if it returned.
pub fn returns_when_dies(g: &mut Game, x: &Perm, _cause: Sym) -> Res<bool> {
    let tapped = x.eot_kw.contains(&"ret_dies_tapped");
    let susp = x.eot_kw.contains(&"ret_dies_suspect");
    let Some(cd) = x.cd else { return Ok(false) };
    if (!tapped && !susp) || x.token || x.is_cmd || !returns_left(g, cd) {
        return Ok(false);
    }
    if !g.hooks.is_empty() && crate::engine::hooks::total_player(g, Event::NoGraveyard, x.orig) != 0 {
        return Ok(false);
    }
    let n = enter(g, x.orig, cd, Enter::default())?;
    crate::glog!(g, "    {} returns to the battlefield", name_of(g, cd));
    if tapped {
        g.perm_mut(n).tapped = true;
    }
    if susp {
        suspect(g, n);
    }
    Ok(true)
}

fn grant_eot(g: &mut Game, m: PermId, kind: &str) {
    let k = intern(kind);
    let x = g.perm_mut(m);
    if !x.eot_kw.contains(&k) {
        x.eot_kw.push(k);
    }
}

// ------------------------------------------------------------------ Braids's Frightful Return
fn lore(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.get(DataKey::Lore).map_or(0, Val::int)
}

/// the cheapest creature p would sacrifice to chapter I
fn braids_fodder(g: &Game, p: PlayerId, src: Option<PermId>) -> Option<PermId> {
    let fod: Vec<PermId> = my_creatures(g, p)
        .into_iter()
        .filter(|&m| Some(m) != src && !g.perm(m).is_cmd && !crate::engine::values::no_sac(g, m))
        .collect();
    min_by(&fod, |m| sac_cost(g, p, m)).filter(|&m| sac_cost(g, p, m) < 1.3)
}

/// read ahead: the chapter to start on. Each chapter's worth now (I: a discard for a cheap creature; II: the best
/// creature card in the graveyard; III: a permanent or 2 life and a card), the later ones discounted.
fn braids_pick(g: &Game, p: PlayerId) -> i64 {
    let opp_hand = g.opps(p).any(|q| !g.player(q).hand.is_empty());
    let v1 = if opp_hand && braids_fodder(g, p, None).is_some() { 1.2 } else { 0.0 };
    let v2 = best_gy(g, p, |_, _| true).map_or(0.0, |c| 0.6 * cv(g, c).min(5.0));
    let v3 = 1.8;
    let tot = |n: i64| -> f64 {
        [v1, v2, v3]
            .iter()
            .enumerate()
            .filter(|(i, _)| *i as i64 + 1 >= n)
            .map(|(i, v)| v * if i as i64 + 1 == n { 1.0 } else { 0.55 })
            .sum()
    };
    (1..=3).max_by(|&a, &b| tot(a).total_cmp(&tot(b)).then(b.cmp(&a))).unwrap_or(3)
}

fn braids_as_enters(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    let n = braids_pick(g, p);
    g.perm_mut(m).data.set(DataKey::Lore, Val::Int(n));
    crate::glog!(g, "    {} reads ahead to chapter {n}", g.perm(m).name);
    Ok(())
}

/// the chapter's effect
fn braids_chapter(g: &mut Game, p: PlayerId, src: PermId, n: i64) -> Res {
    match n {
        1 => {
            if let Some(m) = braids_fodder(g, p, None)
                && g.opps(p).any(|q| !g.player(q).hand.is_empty())
                && trigger_window(g, p, Some(src), "sacrifice a creature: each opponent discards a card", Some(3.0))?
            {
                crate::glog!(g, "    {} sacrifices {}", pname(g, p), g.perm(m).name);
                die(g, m, "sac")?;
                for q in g.opps(p).collect::<Vec<_>>() {
                    discard_worst(g, q, 1)?;
                }
            }
        }
        2 => {
            if let Some(c) = best_gy(g, p, |_, _| true)
                && trigger_window(g, p, Some(src), "return a creature card to your hand", Some(3.0))?
                && let Some(i) = g.player(p).gy.iter().position(|&x| x == c)
            {
                g.player_mut(p).gy.remove(i);
                g.player_mut(p).hand.push(c);
                crate::glog!(g, "    {} returns to {}'s hand", name_of(g, c), pname(g, p));
            }
        }
        _ => {
            let opps: Vec<PlayerId> = g.opps(p).collect();
            let Some(q) = max_by(&opps, |q| threat(g, p, q)) else { return Ok(()) };
            if !trigger_window(g, p, Some(src), "an opponent sacrifices a permanent or loses 2 life", Some(3.0))? {
                return Ok(());
            }
            // the opponent's choice: a permanent they can spare, or the 2 life
            let cands: Vec<PermId> = g
                .player(q)
                .perms
                .iter()
                .copied()
                .filter(|&m| !g.perm(m).token && !g.perm(m).phased && g.perm(m).cd.is_some_and(|c| !g.db.get(c).land))
                .collect();
            let spare = min_by(&cands, |m| pval(g, m)).filter(|&m| pval(g, m) < 2.2 || g.player(q).life <= 4);
            if let Some(m) = spare {
                crate::glog!(g, "    {} sacrifices {}", pname(g, q), g.perm(m).name);
                die(g, m, "sac")?;
            } else {
                lose_life(g, q, 2, Some(p), "other", None)?;
                draw(g, p, 1, false)?;
            }
        }
    }
    Ok(())
}

/// the starting chapter triggers as it enters
fn braids_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    let n = lore(g, src);
    braids_chapter(g, o, src, n)?;
    if n >= 3 && g.perm(src).on_bf {
        die(g, src, "sac")?; // sacrificed after III
    }
    Ok(())
}

/// after your draw step: a lore counter
fn braids_main1(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner == p && g.perm(src).data.get(DataKey::Lore).is_some() {
        braids_lore(g, p, src)?;
    }
    Ok(())
}

/// a lore counter (after the draw step, or proliferated): that chapter triggers; after III it is sacrificed
fn braids_lore(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    let n = lore(g, src) + 1;
    g.perm_mut(src).data.set(DataKey::Lore, Val::Int(n));
    if n <= 3 {
        braids_chapter(g, p, src, n)?;
    }
    if n >= 3 && g.perm(src).on_bf {
        die(g, src, "sac")?;
    }
    Ok(())
}

/// does a proliferated lore counter help (chapter II or III next)
fn braids_wants(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.perm(m).owner == p && lore(g, m) < 3
}

// ------------------------------------------------------------------ Soul Enervation
/// -4/-4 on the opposing creature it kills (the most valuable), else the one it shrinks most
fn enervation_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    snapshot(g, src);
    let tg: Vec<PermId> = opp_creatures(g, o).into_iter().filter(|&x| !untargetable(g, x)).collect();
    let kill: Vec<PermId> = tg.iter().copied().filter(|&x| etgh(g, x) <= 4).collect();
    let pick = max_by(&kill, |x| pval(g, x)).or_else(|| max_by(&tg, |x| pval(g, x)));
    if let Some(x) = pick
        && trigger_window(g, o, Some(src), "-4/-4 until end of turn", Some(4.0))?
    {
        shrink(g, o, x, 4)?;
    }
    Ok(())
}

/// the creature cards in p's graveyard as of the last look
fn snapshot(g: &mut Game, src: PermId) {
    let o = g.perm(src).owner;
    let cur: Vec<Val> = gy_creatures(g, o).into_iter().map(|c| Val::Int(c.0 as i64)).collect();
    g.perm_mut(src).data.set(DataKey::Held, Val::List(cur));
}

/// whenever one or more creature cards leave your graveyard, each opponent loses 1 life and you gain 1 life. Seen as a
/// state check: a creature card of the last look that is no longer in the graveyard left it (reanimated, returned to
/// hand, cast, exiled).
fn enervation_sba(g: &mut Game, src: Src) -> Res {
    let x = g.perm(src);
    if !x.on_bf || x.phased || x.neutered {
        return Ok(());
    }
    let o = x.owner;
    let Some(Val::List(prev)) = x.data.get(DataKey::Held).cloned() else {
        snapshot(g, src);
        return Ok(());
    };
    let left = prev.iter().any(|v| {
        let c = CardId(v.int() as u16);
        !g.player(o).gy.contains(&c)
    });
    snapshot(g, src);
    if left && g.player(o).alive {
        crate::glog!(g, "    Soul Enervation: each opponent loses 1 life, {} gains 1", pname(g, o));
        for q in g.opps(o).collect::<Vec<_>>() {
            lose_life(g, q, 1, Some(o), "drain", None)?;
        }
        gain(g, o, 1)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ the reanimation spells
/// the best graveyard creature card that costs no more than `life` - 6 to bring back with a life loss of its mana value
fn rean_pick(g: &Game, p: PlayerId, max_mv: u32, pay_life: bool) -> Option<CardId> {
    let life = g.player(p).life;
    best_gy(g, p, |g, c| {
        let d = g.db.get(c);
        d.cmc <= max_mv && (!pay_life || life - d.cmc as i32 >= 6)
    })
}

/// Rewrite Regrets: return target creature or planeswalker card with mana value 6 or less from your graveyard to the
/// battlefield. Empower Jace 2. (A planeswalker card in the graveyard comes back only when no creature is there.)
fn rewrite_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let pick = rean_pick(g, p, 6, false).or_else(|| {
        let ws: Vec<CardId> = g
            .player(p)
            .gy
            .iter()
            .copied()
            .filter(|&c| g.db.get(c).types.has(crate::cards::Types::PLANESWALKER) && g.db.get(c).cmc <= 6)
            .collect();
        max_by(&ws, |c| g.db.get(c).cmc as f64)
    });
    if let Some(c) = pick {
        reanimate(g, p, c)?;
    }
    crate::impls::alela::empower(g, p, 2);
    Ok("gy")
}

/// It Doesn't Add Up: return target creature card from your graveyard to the battlefield. Suspect it.
fn addup_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    if let Some(c) = rean_pick(g, p, 99, false)
        && let Some(m) = reanimate(g, p, c)?
        && g.perm(m).on_bf
    {
        suspect(g, m);
    }
    Ok("gy")
}

/// Phyrexian Delver: when it enters, return target creature card from your graveyard to the battlefield; you lose
/// life equal to its mana value
fn delver_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    let Some(c) = rean_pick(g, o, 99, true) else { return Ok(()) };
    if !trigger_window(g, o, Some(src), "return a creature card from your graveyard", Some(5.0))? {
        return Ok(());
    }
    if reanimate(g, o, c)?.is_some() {
        let mv = g.db.get(c).cmc as i32;
        lose_life(g, o, mv, Some(o), "other", None)?;
    }
    Ok(())
}

/// Send in the Pest: each opponent discards a card (their choice); you create a 1/1 black and green Pest (it gains
/// you 1 life whenever it attacks: `cardcode::keyword_attack`)
fn pest_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    for q in g.opps(p).collect::<Vec<_>>() {
        discard_worst(g, q, 1)?;
    }
    let spec = Tokens { color: Some(Colors::from_letters("BG")), types: vec![intern("pest")], ..Tokens::new(1, 1) };
    make_tokens(g, p, spec)?;
    Ok("gy")
}

/// Presumed Dead: until end of turn, target creature gets +2/+0 and gains "When this creature dies, return it to the
/// battlefield under its owner's control and suspect it."
fn presumed_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let t = ctx.target.filter(|&t| g.perm(t).on_bf && g.is_creature(t)).or_else(|| {
        let cs = my_creatures(g, p);
        max_by(&cs, |m| pval(g, m))
    });
    if let Some(t) = t {
        let x = g.perm_mut(t);
        x.eot_pt.0 += 2;
        grant_eot(g, t, "ret_dies_suspect");
    }
    Ok("gy")
}

fn presumed_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId((arg & 0xffff) as u16);
    let t = PermId((arg >> 16) as u32);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "B", false) || !on(g, p, t) {
        return Ok(false);
    }
    pay(g, p, 1, "B", false)?;
    cast_card(g, p, c, "hand", Ctx { target: Some(t), ..Ctx::default() })?;
    Ok(true)
}

// ------------------------------------------------------------------ the creatures
/// Festering Goblin: when it dies, target creature gets -1/-1 until end of turn (an opposing one it kills, else the
/// most valuable)
fn goblin_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    let tg: Vec<PermId> = opp_creatures(g, o).into_iter().filter(|&x| !untargetable(g, x)).collect();
    let kill: Vec<PermId> = tg.iter().copied().filter(|&x| etgh(g, x) <= 1).collect();
    let Some(x) = max_by(&kill, |x| pval(g, x)).or_else(|| max_by(&tg, |x| pval(g, x))) else { return Ok(()) };
    if trigger_window(g, o, Some(src), "-1/-1 until end of turn", Some(3.0))? {
        shrink(g, o, x, 1)?;
    }
    Ok(())
}

/// Perigee Beckoner: when it enters, until end of turn another target creature you control gets +2/+0 and gains "When
/// this creature dies, return it to the battlefield tapped under its owner's control."
fn beckoner_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    let cs: Vec<PermId> = my_creatures(g, o).into_iter().filter(|&x| x != src).collect();
    let Some(t) = max_by(&cs, |x| pval(g, x) + if g.perm(x).sick { 0.0 } else { 1.0 }) else { return Ok(()) };
    if trigger_window(g, o, Some(src), "+2/+0 and returns when it dies", Some(3.0))? && g.perm(t).on_bf {
        g.perm_mut(t).eot_pt.0 += 2;
        grant_eot(g, t, "ret_dies_tapped");
    }
    Ok(())
}

/// the warped Beckoner is exiled at the beginning of the next end step; its card can be cast from exile later
fn beckoner_end(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    if !g.perm(src).on_bf || !g.perm(src).data.truthy(DataKey::Warp) {
        return Ok(());
    }
    let o = g.perm(src).owner;
    let c = g.perm(src).cd.unwrap();
    g.perm_mut(src).data.remove(DataKey::Warp);
    crate::glog!(g, "    {} (warp) is exiled", name_of(g, c));
    crate::engine::zones::exile_perm(g, src)?;
    let st = g.turn_stamp();
    g.player_mut(o).flag_turn.insert(intern("beckoner_warped"), st);
    if !g.player(o).exile.contains(&c) {
        g.player_mut(o).exile.push(c);
    }
    Ok(())
}

fn warp_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "B", false) {
        return Ok(false);
    }
    pay(g, p, 1, "B", false)?;
    cast_card(g, p, c, "hand", Ctx::default())?;
    let found = g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && g.perm(m).cd == Some(c));
    if let Some(m) = found {
        g.perm_mut(m).data.set(DataKey::Warp, Val::Bool(true));
    }
    Ok(true)
}

fn exile_cast_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let (cg, cp) = cost_of(g, p, c);
    let Some(i) = g.player(p).exile.iter().position(|&x| x == c) else { return Ok(false) };
    if !can_pay(g, p, cg, &cp, false) {
        return Ok(false);
    }
    pay(g, p, cg, &cp, false)?;
    g.player_mut(p).exile.remove(i);
    cast_card(g, p, c, "exile", Ctx::default())?;
    Ok(true)
}

/// Arnim Zola, Bio-Fanatic: {3}, {T}: create a tapped 2/1 black Villain creature token with menace. Activate only if
/// there are two or more creature cards in your graveyard.
fn zola_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if x.owner != p
        || x.phased
        || x.tapped
        || (x.sick && !crate::dsl::has_kw(g, src, "haste"))
        || gy_creatures(g, p).len() < 2
        || !can_pay(g, p, 3, "", false)
    {
        return Ok(vec![]);
    }
    // spare mana only: the hand's spells come first in your own main phases
    let busy = g.active == Some(p) && post.is_some() && g.player(p).hand.iter().any(|&c| !g.db.get(c).land);
    Ok(vec![Opt {
        utility: if busy { 0.9 } else { 2.0 },
        label: "Arnim Zola: a Villain token".into(),
        act: Some(Action::Ability { src, f: zola_go, arg: 0 }),
    }])
}

fn zola_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !on(g, p, src) || g.perm(src).tapped || gy_creatures(g, p).len() < 2 || !can_pay(g, p, 3, "", false) {
        return Ok(false);
    }
    pay(g, p, 3, "", false)?;
    g.perm_mut(src).tapped = true;
    if !ability_window(g, p, Some(src), "a tapped 2/1 Villain with menace", Some(3.0), None)? {
        return Ok(true);
    }
    let mut spec = Tokens {
        color: Some(Colors::from_letters("B")),
        types: vec![intern("villain")],
        tgh: Some(1),
        ..Tokens::new(1, 2)
    };
    spec.data.set(DataKey::Kws, Val::List(vec![Val::Str(intern("menace"))]));
    for t in make_tokens(g, p, spec)? {
        g.perm_mut(t).tapped = true;
    }
    Ok(true)
}

/// Bloodsworn Squire: {1}{B}, discard a card: it gains indestructible until end of turn. Tap it. Then if there are
/// four or more creature cards in your graveyard, transform it (Bloodsworn Knight: power and toughness each equal to
/// the creature cards in your graveyard; the same ability).
fn squire_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if x.owner != p || x.phased || g.player(p).hand.is_empty() || !can_pay(g, p, 1, "B", false) {
        return Ok(vec![]);
    }
    let n = gy_creatures(g, p).len();
    let hand_cr = g.player(p).hand.iter().any(|&c| g.db.get(c).creature);
    // to transform it: four creature cards (the discard may be the fourth); in the second main phase, as it taps
    if flipped(g, src) || !(n >= 4 || (n >= 3 && hand_cr)) || post != Some(true) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 2.4,
        label: "Bloodsworn Squire: transform".into(),
        act: Some(Action::Ability { src, f: squire_go, arg: 0 }),
    }])
}

fn squire_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !on(g, p, src) || g.player(p).hand.is_empty() || !can_pay(g, p, 1, "B", false) {
        return Ok(false);
    }
    pay(g, p, 1, "B", false)?;
    // discard a creature card when there is a spare one (it counts toward the four)
    let hand = g.player(p).hand.clone();
    let cr: Vec<CardId> = hand.iter().copied().filter(|&c| g.db.get(c).creature).collect();
    let pick = min_by(&cr, |c| cv(g, c));
    if let Some(c) = pick.filter(|_| gy_creatures(g, p).len() == 3) {
        discard_cards(g, p, &[c])?;
    } else {
        discard_worst(g, p, 1)?;
    }
    if !ability_window(g, p, Some(src), "indestructible, tap it, maybe transform", Some(3.0), None)? {
        return Ok(true);
    }
    if !g.perm(src).on_bf {
        return Ok(true);
    }
    grant_eot(g, src, "indestructible");
    g.perm_mut(src).tapped = true;
    if gy_creatures(g, p).len() >= 4 && !flipped(g, src) {
        g.perm_mut(src).data.set(DataKey::Flipped, Val::Bool(true));
        g.selfpt = true;
        crate::glog!(g, "    Bloodsworn Squire transforms into Bloodsworn Knight");
    }
    Ok(true)
}

/// Bloodsworn Knight is as big as the creature cards in your graveyard (the Squire is 3/3)
fn squire_pt(g: &Game, p: PlayerId, m: PermId) -> (i32, i32) {
    if !flipped(g, m) {
        return (0, 0);
    }
    let k = gy_creatures(g, p).len() as i32 - 3;
    (k, k)
}

/// Catapult Fodder: at the beginning of combat on your turn, if you control three or more creatures that each have
/// toughness greater than their power, transform it (Catapult Captain, 2/6)
fn fodder_combat(g: &mut Game, src: Src, p: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == p && !g.perm(src).phased && !flipped(g, src) {
        let n = my_creatures(g, p).into_iter().filter(|&m| etgh(g, m) > epow(g, m)).count();
        if n >= 3 {
            g.perm_mut(src).data.set(DataKey::Flipped, Val::Bool(true));
            g.selfpt = true;
            crate::glog!(g, "    Catapult Fodder transforms into Catapult Captain");
        }
    }
    Ok(vec![])
}

fn fodder_pt(g: &Game, _p: PlayerId, m: PermId) -> (i32, i32) {
    if flipped(g, m) { (1, 1) } else { (0, 0) }
}

/// Catapult Captain: {2}{B}, {T}, sacrifice another creature: target opponent loses life equal to the sacrificed
/// creature's toughness
fn captain_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if x.owner != p
        || x.phased
        || !flipped(g, src)
        || x.tapped
        || (x.sick && !crate::dsl::has_kw(g, src, "haste"))
        || !can_pay(g, p, 2, "B", false)
    {
        return Ok(vec![]);
    }
    let fod: Vec<PermId> = my_creatures(g, p).into_iter().filter(|&m| can_sac(g, src, m)).collect();
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let Some(q) = min_by(&opps, |q| g.player(q).life as f64) else { return Ok(vec![]) };
    // the creature whose toughness is worth the most for what it costs
    let Some(m) = max_by(&fod, |m| etgh(g, m) as f64 - 1.2 * sac_cost(g, p, m)) else { return Ok(vec![]) };
    let t = etgh(g, m).max(0);
    let lethal = g.player(q).life <= t;
    let u = 0.7 * t as f64 - 1.2 * sac_cost(g, p, m);
    if !lethal && (u < 0.8 || post.is_none()) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: if lethal { 9.0 } else { u },
        label: format!("Catapult Captain: sacrifice {} ({} life)", g.perm(m).name, t),
        act: Some(Action::Ability { src, f: captain_go, arg: m.0 as i64 }),
    }])
}

fn captain_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let m = PermId(arg as u32);
    if !on(g, p, src) || !on(g, p, m) || !can_sac(g, src, m) || g.perm(src).tapped || !pay(g, p, 2, "B", false)? {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    let t = etgh(g, m).max(0);
    die(g, m, "sac")?;
    let opps: Vec<PlayerId> = g.opps(p).collect();
    if let Some(q) = min_by(&opps, |q| g.player(q).life as f64)
        && ability_window(g, p, Some(src), "target opponent loses life", Some(3.0), None)?
    {
        lose_life(g, q, t, Some(p), "drain", None)?;
    }
    Ok(true)
}

/// Cheerful Osteomancer enters prepared; while it is prepared you may cast a copy of Raise Dead ({B} sorcery: return
/// target creature card from your graveyard to your hand), which unprepares it
fn osteomancer_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.perm_mut(m).data.set(DataKey::Prepared, Val::Bool(true));
    }
    Ok(())
}

fn osteomancer_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if x.owner != p
        || x.phased
        || !x.data.truthy(DataKey::Prepared)
        || post.is_none()
        || g.active != Some(p)
        || !can_pay(g, p, 0, "B", false)
    {
        return Ok(vec![]);
    }
    let Some(c) = best_gy(g, p, |_, _| true) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: 1.2 + 0.4 * cv(g, c).min(5.0),
        label: format!("Raise Dead (prepared) -> {}", name_of(g, c)),
        act: Some(Action::Ability { src, f: raise_go, arg: 0 }),
    }])
}

fn raise_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !g.perm(src).data.truthy(DataKey::Prepared) || !can_pay(g, p, 0, "B", false) {
        return Ok(false);
    }
    pay(g, p, 0, "B", false)?;
    g.perm_mut(src).data.set(DataKey::Prepared, Val::Bool(false));
    crate::impls::mine::cast_copy_with(g, p, None, None, false, 2.0, &mut |g| {
        if let Some(c) = best_gy(g, p, |_, _| true)
            && let Some(i) = g.player(p).gy.iter().position(|&x| x == c)
        {
            g.player_mut(p).gy.remove(i);
            g.player_mut(p).hand.push(c);
            crate::glog!(g, "    {} returns to {}'s hand", name_of(g, c), pname(g, p));
        }
        Ok(())
    })?;
    Ok(true)
}

// ------------------------------------------------------------------ the plan
/// the plays yawgmoth adds from the hand and exile: Presumed Dead on an attacker, warp, and the Beckoner from exile
pub fn yawgmoth_options(g: &mut Game, p: PlayerId, post: bool) -> Res<Vec<Opt>> {
    let mut out = vec![];
    if g.active != Some(p) {
        return Ok(out);
    }
    for c in g.player(p).hand.clone() {
        match name_of(g, c) {
            "Presumed Dead" if !post && can_pay(g, p, 1, "B", false) => {
                // before combat, on an attacker that is worth bringing back
                let cs: Vec<PermId> = my_creatures(g, p)
                    .into_iter()
                    .filter(|&m| !g.perm(m).sick && !g.perm(m).tapped && !g.perm(m).noatk && pval(g, m) >= 2.0)
                    .collect();
                if let Some(m) = max_by(&cs, |m| pval(g, m)) {
                    out.push(Opt {
                        utility: 1.8,
                        label: format!("Presumed Dead -> {}", g.perm(m).name),
                        act: Some(Action::Plan { f: presumed_go, arg: (c.0 as i64) | ((m.0 as i64) << 16) }),
                    });
                }
            }
            BECKONER if post == false && can_pay(g, p, 1, "B", false) => {
                // warp when the full cost can't be paid this turn and there is a creature to buff
                let (cg, cp) = cost_of(g, p, c);
                if !can_pay(g, p, cg, &cp, false) && !my_creatures(g, p).is_empty() {
                    out.push(Opt {
                        utility: 2.2,
                        label: "Perigee Beckoner (warp)".into(),
                        act: Some(Action::Plan { f: warp_go, arg: c.0 as i64 }),
                    });
                }
            }
            _ => {}
        }
    }
    let st = g.turn_stamp();
    let later = g.player(p).flag_turn.get(&intern("beckoner_warped")).is_none_or(|s| *s != st);
    if later && let Some(c) = g.player(p).exile.iter().copied().find(|&c| name_of(g, c) == BECKONER) {
        let (cg, cp) = cost_of(g, p, c);
        if can_pay(g, p, cg, &cp, false) {
            out.push(Opt {
                utility: 4.5,
                label: "Perigee Beckoner (from exile)".into(),
                act: Some(Action::Plan { f: exile_cast_go, arg: c.0 as i64 }),
            });
        }
    }
    Ok(out)
}

/// yawgmoth's cast priority: Yawgmoth first; the recursion spells and the Delver only with a worthwhile card in the
/// graveyard; the fodder makers, drains and Sagas next; the rest as the generic priorities have it
pub fn yawgmoth_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let d = g.db.get(c);
    let pl = g.player(p);
    if c == pl.cmd {
        return 88 - 3 * pl.tax as i32;
    }
    let life = pl.life;
    let best = |max_mv: u32, pay_life: bool| rean_pick(g, p, max_mv, pay_life).map_or(0.0, |c| cv(g, c));
    let yawg_out = pl.perms.iter().any(|&m| g.perm(m).cd == Some(pl.cmd) && !g.perm(m).phased);
    match &*d.name {
        "Rewrite Regrets" => {
            if best(6, false) >= 2.5 {
                68
            } else {
                0
            }
        }
        "It Doesn't Add Up" => {
            if best(99, false) >= 2.5 {
                64
            } else {
                0
            }
        }
        DELVER => {
            if best(99, true) >= 2.5 && life > 14 {
                74
            } else {
                36
            }
        }
        "Presumed Dead" => 0, // `yawgmoth_options` casts it
        ENERVATION => {
            let kill = opp_creatures(g, p).iter().any(|&m| etgh(g, m) <= 4 && pval(g, m) >= 2.0);
            if kill { 66 } else { 34 }
        }
        BRAIDS => {
            let v = best_gy(g, p, |_, _| true).is_some() || braids_fodder(g, p, None).is_some();
            if v { 58 } else { 30 }
        }
        "Send in the Pest" => 56,
        "Dread Drone" => 60,
        ZOLA => 57,
        BECKONER => 52,
        FODDER | SQUIRE => 50,
        OSTEOMANCER => {
            if gy_creatures(g, p).is_empty() {
                48
            } else {
                62
            }
        }
        "Blood Artist" => {
            if yawg_out || my_creatures(g, p).len() >= 2 {
                66
            } else {
                54
            }
        }
        _ => generic_prio(g, p, c),
    }
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, YAWGMOTH)?;
    c.options = Some(yawg_options);
    c.defend = Some(yawg_defend_hook);
    let c = r.card(db, BRAIDS)?;
    c.as_enters = Some(braids_as_enters);
    c.etb = Some(braids_etb);
    c.main1 = Some(braids_main1);
    c.saga = Some((braids_wants, braids_lore));
    let c = r.card(db, ENERVATION)?;
    c.etb = Some(enervation_etb);
    c.sba = Some(enervation_sba);
    r.card(db, "Rewrite Regrets")?.resolve = Some(rewrite_resolve);
    r.card(db, "It Doesn't Add Up")?.resolve = Some(addup_resolve);
    r.card(db, "Send in the Pest")?.resolve = Some(pest_resolve);
    r.card(db, "Presumed Dead")?.resolve = Some(presumed_resolve);
    r.card(db, DELVER)?.etb = Some(delver_etb);
    let c = r.card(db, "Festering Goblin")?;
    c.self_dies = Some(goblin_dies);
    let c = r.card(db, BECKONER)?;
    c.etb = Some(beckoner_etb);
    c.end_step = Some(beckoner_end);
    r.card(db, ZOLA)?.options = Some(zola_options);
    let c = r.card(db, SQUIRE)?;
    c.options = Some(squire_options);
    c.self_pt = Some(squire_pt);
    let c = r.card(db, FODDER)?;
    c.combat_start = Some(fodder_combat);
    c.options = Some(captain_options);
    c.self_pt = Some(fodder_pt);
    let c = r.card(db, OSTEOMANCER)?;
    c.etb = Some(osteomancer_etb);
    c.options = Some(osteomancer_options);
    Ok(())
}
