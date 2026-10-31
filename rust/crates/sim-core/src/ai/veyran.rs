//! Veyran's AI (`veyran_prio`, `payoff`, `engine_payoff`, `veyran_boots`, `aether_check`, and the 'veyran' parts of
//! `ai/brain.rs`'s special options and `protect_response`). The Displacer Kitten combo is gone (Kitten is not in the deck).
//! Veyran's tutor wish list is in decks::tutor_pick_named.

use super::brain::{Situation, removal_risk};
use super::decks::flute_pick;
use super::plans;
use crate::engine::cast::cast_card;
use crate::engine::combat::blocked;
use crate::engine::life::{check_state, lose_life};
use crate::engine::mana::{can_pay, pay, total_mana};
use crate::engine::stack::{ability_window, equip_to};
use crate::engine::tutors::card_worth;
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
    if t.has(Tag::Vfire) {
        return firesinger_prio(g, p, c);
    }
    for (k, v) in [(Tag::Recruit, 73), (Tag::Kiln, 70), (Tag::Birgi, 69)] {
        if t.has(k) {
            return v;
        }
    }
    if t.str(Tag::Prot) == Some("boots") {
        return greaves_prio(g, p);
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
    if t.has(Tag::Led) {
        return 20;
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
        return tutor_is_prio(g, p, c);
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
    if let Some(v) = engine_creature_prio(g, p, c) {
        return v;
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
    if &*d.name == "Triple Triad" {
        return triple_triad_prio(g, p);
    }
    if &*d.name == "Fiery Emancipation" {
        let any =
            pl.perms.iter().any(|&m| g.perm(m).cd.is_some_and(|cd| g.db.get(cd).tag(Tag::Ping) || g.is_creature(m)));
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
    if t.has(Tag::Breach) {
        // Underworld Breach (10/19 test): the spent instants and sorceries come back with escape, each recast
        // retriggering Veyran's magecraft doubling; once two are worth recasting and there is mana for the turn
        let k = super::decks::breach_candidates(g, p, false).len();
        return if k >= 2 && total_mana(g, p, false) >= 5 { 56 } else { 0 };
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
/// brain.special_options for Veyran: an Aetherflux shot, equipping Boots, Mizzix's Mastery
pub fn options(g: &mut Game, p: PlayerId, _s: &Situation, _post: bool) -> Res<Vec<Opt>> {
    let mut o = vec![];
    let plan = |u: f64, label: String, f: crate::hooks::PlanFn, arg: i64| Opt {
        utility: u,
        label,
        act: Some(Action::Plan { f, arg }),
    };
    let pl = g.player(p);
    if has(g, p, Tag::Aether) && pl.life >= 51 {
        o.push(plan(11.0, "Aetherflux shot".into(), aether_go, 0));
    }
    let loose_boots = find(g, p, Tag::Prot).iter().any(|&e| {
        g.perm(e).cd.is_some_and(|c| g.db.get(c).tags.str(Tag::Prot) == Some("boots"))
            && g.perm(e).attached.is_none_or(|a| !g.perm(a).on_bf || g.perm(a).owner != p)
    });
    let key_creature =
        pl.perms.iter().any(|&m| g.is_creature(m) && g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Veyran)));
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
    let cost = if crate::engine::values::card_name(g, e) == Some("Lightning Greaves") { 0 } else { 1 }; // Swiftfoot Boots: {1}
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
    let rank = |m: PermId| -> f64 {
        let t = &g.db.get(g.perm(m).cd.unwrap()).tags;
        if t.has(Tag::Veyran) { 9.0 } else { pval(g, m) }
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

// ------------------------------------------------------------------ audit fixes (ai-audit cv)
fn spells_in_hand(g: &Game, p: PlayerId, except: Option<CardId>) -> usize {
    g.player(p).hand.iter().filter(|&&x| Some(x) != except && (g.db.get(x).instant || g.db.get(x).sorcery)).count()
}

/// Blazing Firesinger // Seething Song is a 2/3 body and a mid-chain ritual (it only paired with the Kitten loop, which
/// the deck no longer has): below the engines, a little higher with a payoff out and spells to cast
fn firesinger_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    if engine_payoff(g, p) && spells_in_hand(g, p, Some(c)) >= 2 { 56 } else { 48 }
}

/// Lightning Greaves comes before (or with) Veyran: the engine is removed 1.8 times a game, and shroud + haste
/// cost {2} to cast and {0} to equip. After the commander is gone for good (not waiting, not out), 50.
fn greaves_prio(g: &Game, p: PlayerId) -> i32 {
    let pl = g.player(p);
    let waiting = pl.cmd_in_zone || pl.hand.contains(&pl.cmd);
    if waiting || !find(g, p, Tag::Veyran).is_empty() { 75 } else { 50 }
}

/// Triple Triad: six mana, then each upkeep free spells that each trigger Veyran: only with an engine out
fn triple_triad_prio(g: &Game, p: PlayerId) -> i32 {
    if engine_payoff(g, p) && total_mana(g, p, false) >= 6 { 58 } else { 44 }
}

/// the engine creatures that fell to the generic creature 40: a trigger multiplier or a draw-to-ping converter is worth
/// more with spells in hand and a payoff out
fn engine_creature_prio(g: &Game, p: PlayerId, c: CardId) -> Option<i32> {
    let name = &*g.db.get(c).name;
    let base = match name {
        "Harmonic Prodigy" => 52, // doubles Wizard / Shaman triggers
        "Alania, Divergent Storm" => 50,
        "Docent of Perfection // Final Iteration" => 50,
        "Niv-Mizzet, Parun" => 50,
        "Eris, Roar of the Storm" => 46,
        "Flux Channeler" => 47, // a cheap Wizard that proliferates
        _ => return None,
    };
    let s = spells_in_hand(g, p, None).min(4) as i32;
    let payoff = if engine_payoff(g, p) { 4 } else { 0 };
    Some((base + 2 * s + payoff).min(66))
}

/// Mystical Tutor / Solve the Equation: the instant or sorcery the board needs, by state (a wipe when the table
/// out-boards Veyran; Crackle with Power with payoffs and mana; a counter only with mana to hold it), else a cantrip
pub fn is_wish(g: &Game, p: PlayerId) -> Vec<&'static str> {
    let mana = total_mana(g, p, false) as i32 + 1;
    let own_cre = g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && g.perm(m).on_bf).count() as i32;
    let opp_cre: i32 = g
        .opps(p)
        .map(|q| g.player(q).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).count() as i32)
        .sum();
    let mut out: Vec<&'static str> = vec![];
    if plans::opp_power(g, p) >= 12 || (opp_cre >= 4 && opp_cre >= own_cre + 3) {
        // Blasphemous Act costs {1}{R} less per creature on the battlefield (8 at most)
        if mana + opp_cre + own_cre >= 8 {
            out.push("Blasphemous Act");
        }
        if mana >= 5 {
            out.push("Splatter Technique");
        }
        if mana >= 6 {
            out.push("River's Rebuke");
        }
    }
    if payoff(g, p) && mana >= 7 {
        out.push("Crackle with Power");
    }
    let counter_held = g.player(p).hand.iter().any(|&c| g.db.get(c).tag(Tag::Ctr));
    if !counter_held && mana >= 4 && g.player(p).hand.len() >= 3 {
        out.extend(["Counterspell", "Force of Will"]);
    }
    out.extend(["Stock Up", "Flow State", "Quick Study", "Mystic Confluence", "Crackle with Power"]);
    out
}

/// is the first pick a real target (not just a draw spell)? Solve the Equation ({2}{U}) needs one
fn tutor_is_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    if g.db.get(c).cmc <= 1 {
        return 30; // Mystical Tutor: cheap, leaves the card on top
    }
    let real = ["Blasphemous Act", "Splatter Technique", "River's Rebuke", "Crackle with Power"];
    let pick = super::decks::tutor_pick_named(g, p, "is").map(|x| g.db.get(x).name.to_string());
    if pick.is_some_and(|n| real.contains(&n.as_str())) {
        40
    } else if total_mana(g, p, false) >= 6 && g.player(p).hand.len() <= 3 {
        24
    } else {
        0
    }
}

/// Imperial Recruiter (power 2 or less) by what the board lacks: a pinger when none is out, then a token maker,
/// Storm-Kiln Artist for mana, Archmage Emeritus for cards
pub fn recruit_wish(g: &Game, p: PlayerId) -> Vec<&'static str> {
    let held = |t: Tag| has(g, p, t) || g.player(p).hand.iter().any(|&c| g.db.get(c).tag(t));
    let low_opp = g.opps(p).any(|q| g.player(q).life <= 15);
    let mut out: Vec<&'static str> = vec![];
    let ping = ["Guttersnipe", "Kessig Flamebreather"];
    let toks = ["Young Pyromancer", "Talrand, Sky Summoner", "Third Path Iconoclast"];
    let rest = ["Storm-Kiln Artist", "Archmage Emeritus"];
    if !held(Tag::Ping) || low_opp {
        out.extend(ping);
        out.extend(toks);
    } else if !held(Tag::Spelltok) {
        out.extend(toks);
        out.extend(ping);
    } else {
        out.extend(rest);
        out.extend(ping);
        out.extend(toks);
        return out;
    }
    out.extend(rest);
    out
}
