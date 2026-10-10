//! Veyran's AI (Python's `ais.py`: `veyran_prio`, `payoff`, `engine_payoff`, `veyran_try_combo`, `veyran_fair`,
//! `veyran_boots`, `aether_check`, and the 'veyran' parts of `ai/brain.py`'s special options and `protect_response`).
//! Veyran's tutor wish list is in decks::tutor_pick_named.

use super::brain::{Situation, counter_risk, removal_risk};
use super::decks::{breach_candidates, flute_pick};
use super::plans;
use crate::engine::cast::cast_card;
use crate::engine::combat::blocked;
use crate::engine::life::{check_state, lose_life};
use crate::engine::mana::{can_pay, pay, total_mana};
use crate::engine::stack::{ability_window, equip_to};
use crate::engine::tutors::{card_worth, tutor};
use crate::engine::values::{find, has, pval, shielded, threat, untargetable};
use crate::flow::Res;
use crate::hooks::{Action, Opt};
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::mine;
use crate::state::{Ctx, Game};
use crate::sym::Sym;
use crate::tag::Tag;

// ------------------------------------------------------------------ cast priorities
/// ais.veyran_prio
pub fn veyran_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let pl = g.player(p);
    let d = g.db.get(c);
    let t = &d.tags;
    if c == pl.cmd {
        return 70;
    }
    if t.has(Tag::Rock) || t.has(Tag::Fastmana) {
        return if pl.turns <= 5 { 80 } else { 40 };
    }
    if t.has(Tag::Remora) {
        return if pl.turns <= 4 { 66 } else { 0 }; // Mystic Remora: only early, while upkeep is cheap
    }
    if t.has(Tag::Vkitten) || t.has(Tag::Vfire) {
        return 74;
    }
    for (k, v) in [(Tag::Recruit, 73), (Tag::Kiln, 70), (Tag::Birgi, 69)] {
        if t.has(k) {
            return v;
        }
    }
    if t.str(Tag::Prot) == Some("boots") {
        return 50;
    }
    for (k, v) in [(Tag::Ping, 72), (Tag::Spelltok, 71), (Tag::Aether, 66), (Tag::Dragoncaller, 60)] {
        if t.has(k) {
            return v;
        }
    }
    if t.has(Tag::Spelldraw) || t.has(Tag::Mystic) {
        return 58;
    }
    if &*d.name == "The One Ring" {
        return plans::one_ring_prio(g, p, c);
    }
    if t.has(Tag::Rhystic) {
        return plans::rhystic_prio(g, p, c);
    }
    if t.has(Tag::Sphinx) {
        return super::decks::sphinx_prio(g, p, c);
    }
    if t.has(Tag::Narset) {
        return 52;
    }
    if t.has(Tag::Chromemox) {
        // needs a coloured nonland card to spare
        let spare = pl
            .hand
            .iter()
            .filter(|&&x| {
                let xd = g.db.get(x);
                !xd.land
                    && !xd.types.has(crate::cards::Types::ARTIFACT)
                    && crate::cards::Colors::from_letters(&xd.pips).intersects(pl.ident)
            })
            .count();
        return if spare >= 2 {
            if pl.turns <= 5 { 80 } else { 30 }
        } else {
            0
        };
    }
    if t.has(Tag::Moxd) {
        // needs a land card to discard (keep one for the land drop)
        let n = pl.hand.iter().filter(|&&x| g.db.get(x).land).count();
        return if n >= 2 || (n >= 1 && pl.land_turn == pl.turns as i32) {
            if pl.turns <= 5 { 80 } else { 30 }
        } else {
            0
        };
    }
    if t.has(Tag::Led) {
        return 20;
    }
    if t.has(Tag::Breach) {
        let k = breach_candidates(g, p, false).len();
        return if k >= 2 && total_mana(g, p, false) >= 5 { 56 } else { 0 };
    }
    if t.has(Tag::Panoptic) {
        return if mine::mirror_candidates(g, p).is_empty() { 12 } else { 44 };
    }
    if t.has(Tag::Gifts) {
        return 50;
    }
    if t.has(Tag::Intuition) {
        return 48;
    }
    if t.has(Tag::Jeska) {
        return plans::jeska_prio(g, p, c);
    }
    let tut = t.str(Tag::Tut);
    if tut == Some("any") {
        return 45;
    }
    if t.has(Tag::Thor) {
        return 50;
    }
    if tut == Some("art") {
        return 55;
    }
    if tut == Some("is") {
        return 30;
    }
    if t.has(Tag::Draw) && (d.instant || d.sorcery) {
        return 48;
    }
    if t.has(Tag::Draw) {
        return 42;
    }
    if t.has(Tag::Spider) {
        return 46;
    }
    if t.has(Tag::Flute) {
        return if flute_pick(g, p).is_some_and(|x| x.1 >= 5.0) { 45 } else { 0 };
    }
    if t.has(Tag::Fbgrant) {
        let any = pl.gy.iter().any(|&x| (g.db.get(x).instant || g.db.get(x).sorcery) && g.db.get(x).tag(Tag::Draw));
        return if any { 38 } else { 0 };
    }
    if d.creature {
        return 40;
    }
    if t.has(Tag::Burn) && g.opps(p).any(|q| g.player(q).life <= 15) {
        return 35;
    }
    if t.has(Tag::Tys) {
        return 57; // Thousand-Year Storm
    }
    if t.has(Tag::Stormburn) {
        // Grapeshot: the end of a long turn
        let st = mine::storm_count(g);
        return if st >= 5 || g.opps(p).any(|q| g.player(q).life <= st + 1) { 62 } else { 0 };
    }
    if matches!(&*d.name, "Propaganda" | "Ghostly Prison" | "Crawlspace") {
        // attack tax: sooner when the table hits hard
        return 52 + 18.min(plans::opp_power(g, p));
    }
    if &*d.name == "Fiery Emancipation" {
        let any = pl.perms.iter().any(|&m| {
            g.perm(m).cd.is_some_and(|cd| g.db.get(cd).tag(Tag::Ping) || g.is_creature(m))
        });
        return if any { 64 } else { 40 };
    }
    if &*d.name == "Galvanic Iteration" {
        // with another spell to copy this turn
        let rest = total_mana(g, p, false) as i64 - 2;
        let any = pl.hand.iter().any(|&x| {
            let xd = g.db.get(x);
            x != c
                && (xd.instant || xd.sorcery)
                && xd.cmc as i64 <= rest
                && (xd.tag(Tag::Draw) || xd.tag(Tag::Burn) || xd.tag(Tag::Rem))
        });
        return if any { 47 } else { 0 };
    }
    if t.has(Tag::Reenact) {
        // Reenact the Crime: only with a target
        let tg = mine::reenact_target(g, p, Some(c));
        return if tg.is_some_and(|(x, _)| card_worth(g, p, x, false) >= 4.0) { 50 } else { 0 };
    }
    0
}

/// ais.payoff: payoffs that turn the infinite loop into a win
pub fn payoff(g: &Game, p: PlayerId) -> bool {
    has(g, p, Tag::Ping) || has(g, p, Tag::Aether) || has(g, p, Tag::Dragoncaller)
}

/// ais.engine_payoff: anything that gets value from each spell (for the 'engine online' milestone)
pub fn engine_payoff(g: &Game, p: PlayerId) -> bool {
    payoff(g, p)
        || has(g, p, Tag::Mystic)
        || has(g, p, Tag::Spelltok)
        || has(g, p, Tag::Spelldraw)
        || has(g, p, Tag::Kiln)
}

// ------------------------------------------------------------------ the main-phase options
/// brain.special_options for Veyran: the Kitten + Firesinger combo, an Aetherflux shot, Inventors' Fair for
/// Aetherflux, equipping Boots, Mizzix's Mastery
pub fn options(g: &mut Game, p: PlayerId, s: &Situation, _post: bool) -> Res<Vec<Opt>> {
    let mut o = vec![];
    let plan = |u: f64, label: String, f: crate::hooks::PlanFn, arg: i64| Opt {
        utility: u,
        label,
        act: Some(Action::Plan { f, arg }),
    };
    let pl = g.player(p);
    if has(g, p, Tag::Vkitten) && has(g, p, Tag::Vfire) && payoff(g, p) && !pl.combo_tried && can_pay(g, p, 2, "R", false)
    {
        let risk = 1.0 - (1.0 - counter_risk(g, p)) * (1.0 - removal_risk(g, p));
        let u = 12.0 - 8.0 * risk + if s.danger > 0.6 { 3.0 } else { 0.0 };
        o.push(plan(u, "go for the combo".into(), combo_go, 0));
    }
    if has(g, p, Tag::Aether) && pl.life >= 51 {
        o.push(plan(11.0, "Aetherflux shot".into(), aether_go, 0));
    }
    let fair = pl.lands.iter().any(|&l| g.db.get(g.land(l).cd).tag(Tag::Fair) && !g.land(l).tapped);
    let arts = pl
        .perms
        .iter()
        .filter(|&&m| g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(crate::cards::Types::ARTIFACT)))
        .count();
    if fair
        && !has(g, p, Tag::Aether)
        && !pl.hand.iter().any(|&c| g.db.get(c).tag(Tag::Aether))
        && pl.library.iter().any(|&c| g.db.get(c).tag(Tag::Aether))
        && arts >= 3
        && total_mana(g, p, false) >= 5
    {
        o.push(plan(6.5, "Inventors' Fair for Aetherflux".into(), fair_go, 0));
    }
    let loose_boots = find(g, p, Tag::Prot).iter().any(|&e| {
        g.perm(e).cd.is_some_and(|c| g.db.get(c).tags.str(Tag::Prot) == Some("boots"))
            && g.perm(e).attached.is_none_or(|a| !g.perm(a).on_bf || g.perm(a).owner != p)
    });
    let key_creature = pl.perms.iter().any(|&m| {
        g.is_creature(m)
            && g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Veyran) || g.db.get(c).tag(Tag::Vkitten))
    });
    if can_pay(g, p, 1, "", false) && loose_boots && key_creature {
        o.push(plan(3.0 + 5.0 * removal_risk(g, p), "equip Boots".into(), boots_go, 0));
    }
    let ms = pl.hand.iter().copied().find(|&c| g.db.get(c).tag(Tag::Mastery));
    let k = pl
        .gy
        .iter()
        .filter(|&&c| {
            let d = g.db.get(c);
            (d.instant || d.sorcery) && !d.tag(Tag::Ctr)
        })
        .count();
    if let Some(m) = ms
        && k >= 4
        && can_pay(g, p, 5, "RRR", false)
    {
        let payoff_n = pl
            .perms
            .iter()
            .filter(|&&x| {
                g.perm(x).cd.is_some_and(|c| {
                    let t = &g.db.get(c).tags;
                    t.has(Tag::Ping) || t.has(Tag::Dragoncaller) || t.has(Tag::Aether) || t.has(Tag::Mystic)
                })
            })
            .count();
        let u = 3.0 + 0.5 * k as f64 + 1.5 * payoff_n as f64;
        o.push(plan(u, format!("Mizzix's Mastery overload ({k} spells)"), mastery_go, m.0 as i64));
    }
    if let Some(m) = ms
        && k >= 1
        && can_pay(g, p, 3, "R", false)
    {
        // one target: the best instant/sorcery copied free
        let best = pl
            .gy
            .iter()
            .filter(|&&x| {
                let d = g.db.get(x);
                (d.instant || d.sorcery) && !d.tag(Tag::Ctr)
            })
            .map(|&x| card_worth(g, p, x, true))
            .fold(0.0, f64::max);
        if best >= 4.0 {
            o.push(plan(1.0 + 0.4 * best, "Mizzix's Mastery (one spell)".into(), mastery1_go, m.0 as i64));
        }
    }
    Ok(o)
}

fn mastery_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 5, "RRR", false) {
        return Ok(false);
    }
    pay(g, p, 5, "RRR", false)?;
    cast_card(g, p, c, "hand", Ctx { overload: true, ..Ctx::default() })?;
    Ok(true)
}

fn mastery1_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 3, "R", false) {
        return Ok(false);
    }
    pay(g, p, 3, "R", false)?;
    cast_card(g, p, c, "hand", Ctx::default())?;
    Ok(true)
}

fn combo_go(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    veyran_try_combo(g, p)
}

/// ais.veyran_try_combo: Displacer Kitten + Blazing Firesinger: an infinite spell loop that a payoff turns into a win
pub fn veyran_try_combo(g: &mut Game, p: PlayerId) -> Res<bool> {
    if g.player(p).combo_tried {
        return Ok(false);
    }
    let k: Vec<PermId> = find(g, p, Tag::Vkitten).into_iter().filter(|&m| !g.perm(m).neutered).collect();
    let f = find(g, p, Tag::Vfire);
    if k.is_empty() || f.is_empty() || !payoff(g, p) || !can_pay(g, p, 2, "R", false) {
        return Ok(false);
    }
    g.player_mut(p).combo_tried = true;
    pay(g, p, 2, "R", false)?;
    let t = g.player(p).turns;
    g.player_mut(p).milestone.entry("combo").or_insert(t);
    if g.goldfish {
        return Ok(true);
    }
    g.player_mut(p).stat("combo_attempt", 1);
    crate::glog!(g, "  Veyran starts the Kitten + Firesinger loop");
    let keys: Vec<PermId> = k.into_iter().chain(f).collect();
    if super::combo_interrupted(g, p, "veyran", &keys)? {
        g.player_mut(p).stat("combo_stopped", 1);
        crate::glog!(g, "    ...the loop is stopped");
        return Ok(false);
    }
    super::win(g, p, "combo", None)?;
    Ok(true)
}

fn fair_go(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    veyran_fair(g, p)
}

/// ais.veyran_fair: Inventors' Fair: {4},{T}, sacrifice: search for an artifact (needs 3+ artifacts). Goes for
/// Aetherflux.
pub fn veyran_fair(g: &mut Game, p: PlayerId) -> Res<bool> {
    let mut fairs = vec![];
    for l in g.player(p).lands.clone() {
        let d = g.db.get(g.land(l).cd);
        if d.tag(Tag::Fair) && !g.land(l).tapped {
            let n = d.name.to_string();
            if !blocked(g, p, &n) {
                fairs.push(l);
            }
        }
    }
    if fairs.is_empty() || has(g, p, Tag::Aether) || g.player(p).hand.iter().any(|&c| g.db.get(c).tag(Tag::Aether)) {
        return Ok(false);
    }
    if !g.player(p).library.iter().any(|&c| g.db.get(c).tag(Tag::Aether)) {
        return Ok(false);
    }
    let arts = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(crate::cards::Types::ARTIFACT)))
        .count();
    if arts < 3 {
        return Ok(false);
    }
    let l = fairs[0];
    g.land_mut(l).tapped = true;
    if !can_pay(g, p, 4, "", false) {
        g.land_mut(l).tapped = false;
        return Ok(false);
    }
    pay(g, p, 4, "", false)?;
    crate::engine::turn::remove_land(g, p, l);
    let cd = g.land(l).cd;
    g.player_mut(p).gy.push(cd);
    crate::glog!(g, "  Veyran sacrifices Inventors' Fair");
    tutor(g, p, "art")?;
    Ok(true)
}

fn boots_go(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    veyran_boots(g, p)
}

/// ais.veyran_boots: Swiftfoot Boots / Lightning Greaves: Kitten during a combo setup, else Veyran, else the best
/// engine
pub fn veyran_boots(g: &mut Game, p: PlayerId) -> Res<bool> {
    let mut eq = vec![];
    for e in find(g, p, Tag::Prot) {
        let x = g.perm(e);
        let boots = x.cd.is_some_and(|c| g.db.get(c).tags.str(Tag::Prot) == Some("boots"));
        let loose = x.attached.is_none_or(|a| !g.perm(a).on_bf || g.perm(a).owner != p);
        if boots && loose {
            let n = x.name;
            if !blocked(g, p, n) {
                eq.push(e);
            }
        }
    }
    let Some(&e) = eq.first() else { return Ok(false) };
    let cost = super::seph::boots_cost(g, e);
    if !can_pay(g, p, cost, "", false) {
        return Ok(false);
    }
    let cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && g.perm(m).cd.is_some() && !untargetable(g, m))
        .collect();
    if cands.is_empty() {
        return Ok(false);
    }
    let fire = has(g, p, Tag::Vfire);
    let rank = |m: PermId| -> f64 {
        let t = &g.db.get(g.perm(m).cd.unwrap()).tags;
        if t.has(Tag::Vkitten) && fire {
            10.0
        } else if t.has(Tag::Veyran) {
            9.0
        } else if t.has(Tag::Vkitten) || t.has(Tag::Vfire) {
            7.0
        } else {
            pval(g, m)
        }
    };
    let t = crate::engine::zones::max_by(&cands, rank).unwrap();
    equip_to(g, p, e, t, cost)
}

fn aether_go(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    aether_check(g, p)
}

/// ais.aether_check: Aetherflux Reservoir: pay 50 life, 50 damage to the most threatening opponent
pub fn aether_check(g: &mut Game, p: PlayerId) -> Res<bool> {
    if !has(g, p, Tag::Aether) || g.player(p).life < 51 || blocked(g, p, "Aetherflux Reservoir") {
        return Ok(false);
    }
    let opps: Vec<PlayerId> = g.opps(p).filter(|&q| !shielded(g, q)).collect(); // the damage would be prevented
    if opps.is_empty() {
        return Ok(false);
    }
    lose_life(g, p, 50, Some(p), "other", None)?;
    let src = find(g, p, Tag::Aether).first().copied();
    let ok = match src {
        Some(m) => ability_window(g, p, Some(m), "50 damage", Some(9.0), None)?,
        None => {
            let cd = g.db.id("Aetherflux Reservoir").unwrap();
            crate::engine::stack::ability_window_card(g, p, cd, "50 damage", Some(9.0), None)?
        }
    };
    if !ok {
        return Ok(true);
    }
    let tgt = crate::engine::zones::max_by(&opps, |o| threat(g, p, o)).unwrap();
    let t = g.player(p).turns;
    let pl = g.player_mut(p);
    pl.stat("aether_shots", 1);
    pl.milestone.entry("aether").or_insert(t);
    crate::glog!(g, "  Veyran fires Aetherflux Reservoir at {}", g.player(tgt).name);
    lose_life(g, tgt, 50, Some(p), "aether", None)?;
    check_state(g)?;
    Ok(true)
}

// ------------------------------------------------------------------ protection
/// ais.protect_response for Veyran: Return the Favor turns a removal spell onto one of the caster's permanents;
/// Deflecting Swat, Slip Out the Back, Dive Down (impls::mine)
pub fn protect(
    g: &mut Game,
    owner: PlayerId,
    m: PermId,
    kind: Sym,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    if mine::rtf_redirect(g, owner, m, kind, actor, spell)? {
        return Ok(true);
    }
    mine::veyran_protect(g, owner, m, kind, actor, spell)
}
