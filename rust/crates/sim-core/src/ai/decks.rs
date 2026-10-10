//! Your decks' AI pieces from Python's `ais.py`: cast priorities (Sauron's in full; the others in phase 6), tutor
//! targets, Disruptor Flute's name, the wipe evaluation and mode choice, and the answers to removal and wipes.

use super::brain::{Situation, removal_risk};
use super::{is_main, plans, pool};
use crate::cards::Types;
use crate::engine::cast::on_cast;
use crate::engine::combat::blocked;
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::removal::legal_targets;
use crate::engine::stack::{counter_window, equip_to};
use crate::engine::tutors::card_worth;
use crate::engine::values::{army_of, epow, equipped, etgh, find, has, pval, silenced, stopped, threat};
use crate::engine::zones::{discard_cards, draw, max_by, searchable};
use crate::flow::Res;
use crate::hooks::{Action, Opt};
use crate::ids::{CardId, PermId, PlayerId};
use crate::pysum::PySum;
use crate::state::Game;
use crate::sym::Sym;
use crate::tag::Tag;

// ------------------------------------------------------------------ small helpers
fn in_hand_tag(g: &Game, p: PlayerId, t: Tag) -> bool {
    g.player(p).hand.iter().any(|&c| g.db.get(c).tag(t))
}

/// an Equipment not attached to one of p's creatures on the battlefield (Python: `e.attached is None or
/// e.attached not in p.perms`)
pub fn loose(g: &Game, e: PermId, p: PlayerId) -> bool {
    g.perm(e).attached.is_none_or(|a| !g.perm(a).on_bf || g.perm(a).owner != p)
}

/// ais.bomb_on_bf
pub fn bomb_on_bf(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| {
        let x = g.perm(m);
        !x.phased && x.cd.is_some_and(|c| g.db.get(c).bomb >= 6)
    })
}

/// a compiled ability: a static of this kind
fn has_static(g: &Game, c: CardId, kind: &str) -> bool {
    g.db.get(c).abilities.iter().any(|a| a.static_kind() == Some(kind))
}

// ------------------------------------------------------------------ cast priorities
/// ais.deck_prio: the deck's cast priority for a card (0-90)
pub fn deck_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    match g.player(p).key {
        "sauron" => sauron_prio(g, p, c),
        "seph" => super::seph::seph_prio(g, p, c),
        "veyran" => super::veyran::veyran_prio(g, p, c),
        // PORT(phase 6): galadriel_prio, yshtola_prio, alela_prio, jodah_prio. Until then your other decks cast by
        // the outside decks' tag priority.
        _ => pool::generic_prio(g, p, c),
    }
}

/// ais.sauron_prio
pub fn sauron_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let pl = g.player(p);
    let d = g.db.get(c);
    let t = &d.tags;
    if c == pl.cmd {
        return 85;
    }
    if t.has(Tag::Storm) || t.has(Tag::Led) {
        return 0; // Brain Freeze, Grapeshot, Lion's Eye Diamond: held for the Breach line
    }
    if t.has(Tag::Rock) {
        return if pl.turns <= 5 { 80 } else { 40 };
    }
    if t.has(Tag::Rhystic) {
        return plans::rhystic_prio(g, p, c);
    }
    if t.has(Tag::Remora) {
        return if pl.turns <= 4 { 66 } else { 0 }; // only early, while upkeep is cheap
    }
    if t.has(Tag::Mauhur) {
        return 63;
    }
    if t.has(Tag::Bowmasters) {
        return 62;
    }
    if t.has(Tag::Sword) {
        return 60;
    }
    match &*d.name {
        "Andúril, Flame of the West" => return 57, // another Sword-like piece for the Army
        "Palantír of Orthanc" => return if pl.turns <= 8 { 54 } else { 40 }, // card or damage every end step
        "Lord of the Nazgûl" => {
            // a Wraith per instant or sorcery still to come
            let n = pl.hand.iter().filter(|&&x| g.db.get(x).instant || g.db.get(x).sorcery).count();
            return if n >= 2 { 58 } else { 50 };
        }
        _ => {}
    }
    if compiled_sword(g, c) {
        return 57; // compiled Swords (Fire and Ice, Hearth and Home)
    }
    if t.has(Tag::Assault) {
        return if has(g, p, Tag::Sword) || in_hand_tag(g, p, Tag::Sword) { 58 } else { 30 };
    }
    if t.has(Tag::Cloak) {
        return 57;
    }
    if t.has(Tag::Prolif) {
        return 55;
    }
    if t.has(Tag::Skate) {
        return if army_of(g, p).is_some_and(|a| g.perm(a).plus >= 6) { 56 } else { 0 };
    }
    for (k, v) in [(Tag::Kaervek, 54), (Tag::Witchking, 52), (Tag::SheoA, 60), (Tag::Kefka, 55), (Tag::Eng, 50)] {
        if t.has(k) {
            return v;
        }
    }
    if t.has(Tag::Archivist) {
        return if has(g, p, Tag::Bowmasters) { 50 } else { 20 };
    }
    for (k, v) in [(Tag::Callring, 55), (Tag::Kindred, 50)] {
        if t.has(k) {
            return v;
        }
    }
    if t.has(Tag::Recon) {
        return 46;
    }
    if t.has(Tag::Vraska) {
        return 52;
    }
    if t.has(Tag::Ralzarek) {
        return 42;
    }
    if t.has(Tag::Helm) {
        return if army_of(g, p).is_some() { 44 } else { 20 };
    }
    if t.has(Tag::Warmachine) {
        return 45;
    }
    if t.str(Tag::Prot) == Some("boots") {
        // Lightning Greaves: for Sauron himself (never the Army)
        return if has(g, p, Tag::Sauron) || pl.cmd_in_zone && total_mana(g, p, false) >= 7 { 50 } else { 25 };
    }
    if t.has(Tag::Tut) {
        return 60;
    }
    if &*d.name == "Gamble" {
        return gamble_prio(g, p, c); // hand-written tutor: by the chance of keeping its card
    }
    if matches!(&*d.name, "Wheel of Fortune" | "Windfall" | "Reforge the Soul") {
        // hand-written wheels: their own timing (hand nearly empty)
        return g.registry.get(c).and_then(|i| i.prio).map_or(0, |f| f(g, p, c));
    }
    if &*d.name == "The One Ring" {
        return plans::one_ring_prio(g, p, c);
    }
    if let Some(v) = gc_prio_sauron(g, p, c) {
        return v;
    }
    if t.has(Tag::Draw) && (d.instant || d.sorcery) {
        return 40;
    }
    for (k, v) in [(Tag::Flail, 56), (Tag::Erebos, 48), (Tag::Ozolith, 45), (Tag::Animist, 44)] {
        if t.has(k) {
            return v;
        }
    }
    if t.has(Tag::Unearth) {
        return if pl.gy.iter().any(|&x| g.db.get(x).creature && g.db.get(x).cmc <= 3) { 35 } else { 0 };
    }
    if t.has(Tag::Pwdiscard) {
        return 38;
    }
    if d.creature {
        return 42;
    }
    if d.perm && !t.has(Tag::Prot) {
        return 10;
    }
    0
}

/// a compiled Sword: an equip cost, and a trigger when the equipped creature deals combat damage
fn compiled_sword(g: &Game, c: CardId) -> bool {
    use crate::dsl::model::Ability;
    let d = g.db.get(c);
    has_static(g, c, "equip_cost")
        && d.abilities.iter().any(|a| {
            matches!(a, Ability::Triggered { source, event, .. }
                if source.as_deref() == Some("equipped") && event == "combat_damage")
        })
}

/// ais.gc_prio_sauron: Sauron's priorities for Game Changer candidates (None: not one of them)
fn gc_prio_sauron(g: &Game, p: PlayerId, c: CardId) -> Option<i32> {
    let t = &g.db.get(c).tags;
    Some(if t.has(Tag::Sphinx) {
        sphinx_prio(g, p, c)
    } else if t.has(Tag::Necro) {
        necro_prio(g, p, c)
    } else if t.has(Tag::Citadel) {
        plans::citadel_prio(g, p, c)
    } else if t.has(Tag::Tergrid) {
        55
    } else if t.has(Tag::Agent) {
        plans::agent_prio(g, p, c)
    } else if t.has(Tag::Braids) {
        plans::braids_prio(g, p, c)
    } else if t.has(Tag::Seal) {
        58
    } else if t.has(Tag::Adnaus) {
        plans::adnaus_prio(g, p, c)
    } else if t.has(Tag::Narset) {
        50
    } else if t.has(Tag::Breach) {
        let pl = g.player(p);
        if pl.hand.iter().chain(pl.gy.iter()).any(|&x| g.db.get(x).tag(Tag::Storm)) {
            0 // held for the Breach line (breach_options)
        } else {
            let k = breach_candidates(g, p, false).len();
            if k >= 2 && total_mana(g, p, false) >= 5 { 56 } else { 0 }
        }
    } else if t.has(Tag::Gifts) {
        50
    } else if t.has(Tag::Intuition) {
        48
    } else if t.has(Tag::Jeska) {
        plans::jeska_prio(g, p, c)
    } else {
        return None;
    })
}

/// t4.sphinx_prio: Consecrated Sphinx by the cards it will draw over the next three rounds, discounted with a hand
/// full of spells, and when you're low on life against a big board
pub fn sphinx_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let per_round = 2 * opps.len();
    let lib = g.player(p).library.len();
    let rounds = if per_round > 0 { 3.0f64.min(lib.saturating_sub(12) as f64 / per_round as f64) } else { 0.0 };
    let cards = per_round as f64 * rounds;
    let mana = total_mana(g, p, false).max(1) as f64;
    let backlog: f64 =
        g.player(p).hand.iter().filter(|&&x| x != c && !g.db.get(x).land).map(|&x| g.db.get(x).cmc as f64).psum()
            / mana;
    let usef = if backlog <= 1.0 {
        1.0
    } else if backlog <= 2.0 {
        0.6
    } else {
        0.35
    };
    let mut v = 30.0 + 1.6 * cards * usef;
    let big: i32 = opps
        .iter()
        .flat_map(|&q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && !g.perm(m).phased)
        .map(|m| epow(g, m))
        .sum();
    if big >= 12 && g.player(p).life <= 15 {
        v -= 15.0;
    }
    v.clamp(15.0, 75.0) as i32
}

/// ais.necro_cards: how many cards Necropotence would buy at this end step
pub fn necro_cards(g: &Game, p: PlayerId, c: Option<CardId>) -> i32 {
    let pl = g.player(p);
    let room = 8 - pl.hand.iter().filter(|&&x| Some(x) != c).count() as i32;
    0.max((pl.life - super::necro_floor(g, p)).min(room).min(pl.library.len() as i32))
}

/// ais.necro_prio: Necropotence skips your draw step, so it's worth it only when the life above the floor buys at
/// least two cards a turn
pub fn necro_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let n = necro_cards(g, p, Some(c));
    if n < 2 { 0 } else { 80.min(45 + 5 * n) }
}

/// fixes.gamble_prio: Gamble by the chance its random discard misses the card it found
pub fn gamble_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let rest: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&x| x != c).collect();
    if rest.is_empty() {
        return 0;
    }
    let keep = rest.len() as f64 / (rest.len() + 1) as f64;
    let risk = rest.iter().filter(|&&x| !g.db.get(x).land).map(|&x| card_worth(g, p, x, false)).psum()
        / rest.len().max(1) as f64
        / 10.0;
    (60.0 * keep - 8.0 * risk).max(0.0) as i32
}

/// ais.breach_candidates: nonland cards in your graveyard castable with Underworld Breach's escape
pub fn breach_candidates(g: &Game, p: PlayerId, need_mana: bool) -> Vec<(f64, CardId)> {
    let pl = g.player(p);
    if pl.gy.len() < 4 {
        return vec![];
    }
    let mut out = vec![];
    for &c in &pl.gy {
        let d = g.db.get(c);
        let t = &d.tags;
        if d.land || t.has(Tag::Ctr) || t.has(Tag::Wipe) || t.has(Tag::Breach) {
            continue;
        }
        const SPECIALX: [Tag; 7] = [Tag::Rean, Tag::Fill, Tag::Yawg, Tag::Avarice, Tag::Mastery, Tag::Crackle, Tag::X];
        if SPECIALX.iter().any(|&k| t.has(k)) && !d.has_dsl() {
            continue;
        }
        if d.sorcery && g.active != Some(p) {
            continue;
        }
        if need_mana {
            let (cg, cp) = cost_of(g, p, c);
            if !can_pay(g, p, cg, &cp, false) {
                continue;
            }
        }
        let mut v = card_worth(g, p, c, false);
        if let Some(rk) = t.str(Tag::Rem) {
            let tg = legal_targets(g, p, rk, t.str(Tag::Tgt).unwrap_or("c"), t.has(Tag::Mv4), Some(c));
            if tg.is_empty() {
                continue;
            }
            let best = tg.iter().map(|&m| pval(g, m)).fold(f64::NEG_INFINITY, f64::max);
            v = v.max(10.0 * (best - 2.0));
        }
        if v > 15.0 {
            out.push((v, c));
        }
    }
    out
}

// ------------------------------------------------------------------ Sauron's plays
/// brain.special_options for Sauron: the Breach line, equipping the Army, Champion's Helm, Jace's Archivist
pub fn sauron_options(g: &mut Game, p: PlayerId, _s: &Situation, post: bool) -> Res<Vec<Opt>> {
    let mut o = crate::cardcode::breach_options(g, p, post)?;
    if let Some(a) = army_of(g, p)
        && !equipped(g, a, Tag::Cloak)
        && can_pay(g, p, 2, "", false)
    {
        for tag in [Tag::Sword, Tag::Cloak] {
            if !equipped(g, a, tag) && find(g, p, tag).iter().any(|&e| loose(g, e, p)) {
                o.push(Opt {
                    utility: 6.5,
                    label: "equip the Army".into(),
                    act: Some(Action::Plan { f: sauron_equip, arg: 0 }),
                });
                break;
            }
        }
    }
    let ht = if !find(g, p, Tag::Helm).is_empty() && can_pay(g, p, 1, "", false) { helm_target(g, p) } else { None };
    if let Some(ht) = ht {
        let leg = crate::cardcode::is_legendary(g, ht);
        let u = 3.0 + if leg { 4.0 * removal_risk(g, p) + 0.2 * pval(g, ht) } else { 0.0 };
        o.push(Opt {
            utility: u,
            label: format!("equip Champion's Helm to {}", g.perm(ht).name),
            act: Some(Action::Plan { f: helm_equip, arg: ht.0 as i64 }),
        });
    }
    if find(g, p, Tag::Archivist).iter().any(|&m| !g.perm(m).tapped && !g.perm(m).sick)
        && archivist_worth(g, p)
        && can_pay(g, p, 0, "U", false)
    {
        o.push(Opt {
            utility: 5.0,
            label: "Jace's Archivist wheel".into(),
            act: Some(Action::Plan { f: sauron_archivist, arg: 0 }),
        });
    }
    Ok(o)
}

/// ais.sauron_equip: the Army's Equipment in order: a Sword, Conqueror's Flail, Animist, then Whispersilk Cloak last
pub fn sauron_equip(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    let Some(a) = army_of(g, p) else { return Ok(false) };
    if g.perm(a).phased {
        return Ok(false);
    }
    for tag in [Tag::Sword, Tag::Flail, Tag::Animist, Tag::Cloak] {
        if equipped(g, a, tag) {
            continue;
        }
        let mut eq = vec![];
        for e in find(g, p, tag) {
            if g.perm(e).attached != Some(a) {
                let n = g.perm(e).name;
                if !blocked(g, p, n) {
                    eq.push(e);
                }
            }
        }
        if eq.is_empty() {
            continue;
        }
        if tag == Tag::Cloak {
            if [Tag::Sword, Tag::Flail, Tag::Animist].iter().any(|&x| !find(g, p, x).is_empty() && !equipped(g, a, x)) {
                continue;
            }
            let other = g.player(p).perms.iter().any(|&e| {
                let x = g.perm(e);
                x.cd.is_some_and(|c| has_static(g, c, "equip_cost")) && loose(g, e, p) && !stopped(g, x.name)
            });
            if other {
                continue; // e.g. Sword of Hearth and Home goes on first
            }
        }
        if equipped(g, a, Tag::Cloak) {
            return Ok(false);
        }
        if can_pay(g, p, 2, "", false) {
            return equip_to(g, p, eq[0], a, 2);
        }
    }
    Ok(false)
}

/// ais.helm_target: where Champion's Helm should go: the most valuable legendary creature (the Army once it is the
/// Ring-bearer), else the Army; None if it is already there
pub fn helm_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let legends: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && !g.perm(m).phased && crate::cardcode::is_legendary(g, m))
        .collect();
    let best = max_by(&legends, |m| pval(g, m)).or_else(|| army_of(g, p))?;
    (!equipped(g, best, Tag::Helm)).then_some(best)
}

/// ais.helm_equip
pub fn helm_equip(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let m = PermId(arg as u32);
    let mut helm = None;
    for e in find(g, p, Tag::Helm) {
        let n = g.perm(e).name;
        if !blocked(g, p, n) {
            helm = Some(e);
            break;
        }
    }
    let Some(h) = helm else { return Ok(false) };
    if !g.perm(m).on_bf || g.perm(m).owner != p || !can_pay(g, p, 1, "", false) {
        return Ok(false);
    }
    crate::glog!(g, "  {} equips Champion's Helm to {}", g.player(p).name, g.perm(m).name);
    equip_to(g, p, h, m, 1)?;
    Ok(true)
}

/// mine.archivist_worth: the wheel: with Orcish Bowmasters out (every opponent draw pings), or when your hand is
/// much smaller
pub fn archivist_worth(g: &Game, p: PlayerId) -> bool {
    let most = g.players.iter().filter(|q| q.alive).map(|q| q.hand.len()).max().unwrap_or(0);
    has(g, p, Tag::Bowmasters) || (g.player(p).hand.len() <= 1 && most >= 4)
}

/// ais.sauron_archivist: Jace's Archivist: everyone discards their hand and draws the largest hand size
pub fn sauron_archivist(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    if g.player(p).arch_t == Some(g.player(p).turns) {
        return Ok(false);
    }
    let mut ar = None;
    for m in find(g, p, Tag::Archivist) {
        let x = g.perm(m);
        if !x.sick && !x.tapped {
            let n = x.name;
            if !blocked(g, p, n) {
                ar = Some(m);
                break;
            }
        }
    }
    let Some(a) = ar else { return Ok(false) };
    if !archivist_worth(g, p) || !can_pay(g, p, 0, "U", false) {
        return Ok(false);
    }
    pay(g, p, 0, "U", false)?;
    g.perm_mut(a).tapped = true;
    let t = g.player(p).turns;
    g.player_mut(p).arch_t = Some(t);
    let n = g.players.iter().filter(|q| q.alive).map(|q| q.hand.len()).max().unwrap_or(0) as u32;
    let alive: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
    for &q in &alive {
        let hand = g.player(q).hand.clone();
        discard_cards(g, q, &hand)?;
    }
    for &q in &alive {
        if g.player(q).alive {
            draw(g, q, n, false)?;
        }
    }
    crate::engine::life::check_state(g)?;
    Ok(true)
}

/// ais.monolith_untap_options: Grim Monolith {4}: untap, with mana that would otherwise go unused
pub fn monolith_untap_options(g: &mut Game, p: PlayerId) -> Vec<Opt> {
    let mut out = vec![];
    for m in find(g, p, Tag::Grim) {
        if !g.perm(m).tapped {
            continue;
        }
        let n = g.perm(m).name;
        if blocked(g, p, n) || !can_pay(g, p, 4, "", false) {
            continue;
        }
        out.push(Opt {
            utility: 3.0,
            label: "untap Grim Monolith".into(),
            act: Some(Action::Plan { f: untap_monolith, arg: m.0 as i64 }),
        });
    }
    out
}

fn untap_monolith(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let m = PermId(arg as u32);
    if !g.perm(m).tapped || !g.perm(m).on_bf || g.perm(m).owner != p || !can_pay(g, p, 4, "", false) {
        return Ok(false);
    }
    pay(g, p, 4, "", false)?;
    g.perm_mut(m).tapped = false;
    crate::glog!(g, "  {} untaps Grim Monolith", g.player(p).name);
    Ok(true)
}

// ------------------------------------------------------------------ Disruptor Flute
const ACTIVATED_ENGINES: [&str; 5] = [
    "Erebos, God of the Dead",
    "Vraska, Betrayal's Sting",
    "Ral Zarek, Guest Lecturer",
    "Nicol Bolas, Dragon-God",
    "Rhys the Redeemed",
];

/// ais.flute_pick: the card name that hurts opponents most: commanders with activated engines, combo pieces,
/// Equipment, sacrifice outlets, planeswalkers ... (public information only: commanders and the battlefield)
pub fn flute_pick(g: &Game, p: PlayerId) -> Option<(CardId, f64)> {
    use crate::dsl::model::Ability;
    let taken: Vec<CardId> = g.flutes.iter().filter(|(f, _)| g.perm(*f).on_bf).map(|x| x.1).collect();
    let mut cands: Vec<(CardId, f64)> = vec![];
    let mut add = |c: CardId, v: f64| {
        if taken.contains(&c) {
            return;
        }
        match cands.iter_mut().find(|x| x.0 == c) {
            Some(x) => {
                if v > x.1 {
                    x.1 = v;
                }
            }
            None => {
                if v > 0.0 {
                    cands.push((c, v));
                }
            }
        }
    };
    for q in g.opps(p) {
        let ql = g.player(q);
        if ql.key == "najeela" {
            add(ql.cmd, 9.0 + threat(g, p, q) * 0.05); // WUBRG extra combats + recast tax
        } else {
            add(ql.cmd, if ql.cmd_in_zone { 4.0 } else { 1.5 } + if ql.tax >= 2 { 1.0 } else { 0.0 });
        }
        for &m in &ql.perms {
            let x = g.perm(m);
            let Some(cd) = x.cd else { continue };
            if x.phased {
                continue;
            }
            let d = g.db.get(cd);
            let t = &d.tags;
            let mut v: f64 = 0.0;
            if t.has(Tag::Assault) {
                v = 4.0 + if has(g, q, Tag::Sword) { 7.0 } else { 0.0 };
            }
            if t.has(Tag::Sword) {
                v = 4.0 + if has(g, q, Tag::Assault) { 7.0 } else { 0.0 };
            }
            if t.has(Tag::Cloak) || t.has(Tag::Flail) || t.has(Tag::Animist) {
                v = v.max(3.0);
            }
            if t.str(Tag::Prot) == Some("boots") {
                v = v.max(2.5);
            }
            if t.has(Tag::Sac) {
                v = v.max(if ql.key == "seph" { 3.5 } else { 1.0 });
            }
            if t.has(Tag::Clamp)
                || t.has(Tag::Archivist)
                || t.has(Tag::Rhys)
                || t.has(Tag::Lidless)
                || t.str(Tag::Fill) == Some("tortured")
            {
                v = v.max(4.0);
            }
            if t.has(Tag::Aether) {
                v = v.max(7.0);
            }
            if d.types.has(Types::PLANESWALKER) || ACTIVATED_ENGINES.contains(&&*d.name) {
                v = v.max(4.0);
            }
            if d.abilities.iter().any(|a| {
                matches!(a, Ability::Activated { .. } | Ability::Loyalty { .. })
                    || a.static_kind() == Some("equip_cost")
            }) {
                v = v.max(3.0);
            }
            add(cd, v);
        }
        for &l in &ql.lands {
            let d = g.db.get(g.land(l).cd);
            if d.tag(Tag::Passage) && army_of(g, q).is_some() {
                add(d.id, 3.0);
            }
            if d.tag(Tag::Desert) && g.player(p).key == "seph" {
                add(d.id, 5.0);
            }
        }
    }
    max_by(&cands, |x| x.1)
}

// ------------------------------------------------------------------ tutors
/// ais.TUTOR_OK (plus 'ae' from _tutor_pick_named): what each kind of tutor may find
pub fn tutor_ok(g: &Game, kind: &str, c: CardId) -> bool {
    let d = g.db.get(c);
    match kind {
        "is" => d.instant || d.sorcery,
        "art" => d.types.has(Types::ARTIFACT),
        "perm" => d.perm,
        "ubr" => !d.land && d.pips.chars().any(|ch| "UBR".contains(ch)),
        "cre2" => d.creature && d.pow <= 2,
        "cre" => d.creature,
        "ench" => d.types.has(Types::ENCHANTMENT),
        _ => true,
    }
}

/// the kinds TUTOR_OK knows (any other kind finds anything)
fn tutor_ok_known(kind: &str) -> &str {
    match kind {
        "is" | "art" | "perm" | "ubr" | "cre2" | "cre" | "ench" => kind,
        _ => "any",
    }
}

/// ais._tutor_pick_named: your deck's named wish list for a tutor of this kind
pub fn tutor_pick_named(g: &Game, p: PlayerId, kind: &str) -> Option<CardId> {
    let ok = |c: CardId| match kind {
        "ae" => {
            let t = g.db.get(c).types;
            t.has(Types::ARTIFACT) || t.has(Types::ENCHANTMENT)
        }
        "any" => true,
        k => tutor_ok(g, k, c),
    };
    let okn: Vec<CardId> = searchable(g, p).into_iter().filter(|&c| ok(c)).collect();
    let first = |ns: &[&str]| -> Option<CardId> { ns.iter().filter_map(|n| g.db.id(n)).find(|c| okn.contains(c)) };
    let pl = g.player(p);
    let held = |t: Tag| has(g, p, t) || in_hand_tag(g, p, t);
    match pl.key {
        // PORT(phase 6): Y'shtola's and Jodah's wish lists
        "seph" => super::seph::seph_tutor_target(g, p).and_then(|n| g.db.id(n)).filter(|c| okn.contains(c)),
        "veyran" => {
            if kind == "art" {
                return first(&["Aetherflux Reservoir", "Sol Ring", "Fellwar Stone"]);
            }
            if kind == "cre2" {
                let kitten = held(Tag::Vkitten);
                let fire = held(Tag::Vfire);
                if kitten && !fire {
                    return first(&["Blazing Firesinger // Seething Song"]);
                }
                if fire && !kitten {
                    return first(&["Displacer Kitten"]);
                }
                return first(&[
                    "Displacer Kitten",
                    "Blazing Firesinger // Seething Song",
                    "Guttersnipe",
                    "Archmage Emeritus",
                ]);
            }
            if !in_hand_tag(g, p, Tag::Ctr) {
                return first(&["Counterspell", "Force of Will", "Mystic Confluence", "Stock Up"]);
            }
            first(&["Stock Up", "Flow State", "Quick Study", "Crackle with Power"])
        }
        "sauron" => {
            let sw = held(Tag::Sword);
            let asl = held(Tag::Assault);
            let mut order: Vec<&str> = vec![];
            if sw && !asl {
                order.push("Aggravated Assault");
            }
            if asl && !sw {
                order.push("Sword of Feast and Famine");
            }
            if sw && asl {
                order.push("Whispersilk Cloak");
            }
            let br = held(Tag::Breach);
            let st = pl.hand.iter().chain(pl.gy.iter()).any(|&c| g.db.get(c).tag(Tag::Storm));
            const LABMEN: [&str; 2] = ["Laboratory Maniac", "Jace, Wielder of Mysteries"];
            let tail = [
                "Rhystic Study",
                "Sword of Feast and Famine",
                "Aggravated Assault",
                "Deepglow Skate",
                "Counterspell",
                "Whispersilk Cloak",
            ];
            let breach_first = ["Grapeshot", "Lion's Eye Diamond", LABMEN[0], LABMEN[1]]
                .iter()
                .any(|n| g.db.id(n).is_some_and(|c| pl.deck_names.contains(&c)));
            if breach_first {
                // Breach-first builds
                let mut line: Vec<&str> = vec![];
                if !br {
                    line.push("Underworld Breach");
                }
                if !st {
                    line.extend(["Brain Freeze", "Grapeshot"]);
                }
                const RITUALS: [&str; 5] =
                    ["Dark Ritual", "Cabal Ritual", "Lotus Petal", "Lion's Eye Diamond", "Jeska's Will"];
                let rit = pl.hand.iter().chain(pl.gy.iter()).any(|&c| RITUALS.contains(&&*g.db.get(c).name));
                let lab = pl.perms.iter().any(|&m| g.perm(m).cd.is_some_and(|c| LABMEN.contains(&&*g.db.get(c).name)))
                    || pl.hand.iter().any(|&c| LABMEN.contains(&&*g.db.get(c).name));
                if br && st && !lab {
                    line.extend(LABMEN); // the self-mill finish
                }
                if br && st && !rit {
                    line.extend(["Lion's Eye Diamond", "Jeska's Will", "Dark Ritual", "Cabal Ritual", "Lotus Petal"]);
                }
                line.extend(order);
                line.extend(tail);
                return first(&line);
            }
            if br && !st {
                let at = if !(sw || asl) { 0 } else { order.len() };
                order.insert(at, "Brain Freeze");
            }
            if st && !br {
                let at = if !(sw || asl) { 0 } else { order.len() };
                order.insert(at, "Underworld Breach");
            }
            order.extend(tail); // artifact tutors with the Sword already found: evasion, not a rock
            first(&order)
        }
        _ => None,
    }
}

/// ais.tutor_pick: deck-specific wish lists first; otherwise the highest-priority legal card
pub fn tutor_pick(g: &Game, p: PlayerId, kind: &str) -> Option<CardId> {
    if let Some(c) = tutor_pick_named(g, p, kind) {
        return Some(c);
    }
    let okk = tutor_ok_known(kind);
    let main = is_main(g.player(p).key);
    if !main && let Some(c) = pool::tutor_pick(g, p, |c| tutor_ok(g, okk, c)) {
        return Some(c); // outside deck: its wish list
    }
    let prio = |c: CardId| -> f64 {
        let v = deck_prio(g, p, c) as f64;
        if main || v != 0.0 {
            v
        } else if g.db.get(c).has_dsl() {
            crate::dsl::card_value(g, p, c) * 10.0
        } else {
            0.0
        }
    };
    let cands: Vec<CardId> =
        searchable(g, p).into_iter().filter(|&c| tutor_ok(g, okk, c) && !g.db.get(c).land).collect();
    if cands.is_empty() {
        return None;
    }
    let mut best: Option<(CardId, (f64, f64, f64))> = None;
    let late = g.player(p).turns > 4; // past the early turns: impact, not cast priority (no Sol Ring on turn 10)
    for c in cands {
        let d = g.db.get(c);
        let key = if late { (tutor_value(g, p, c), d.cmc as f64, 0.0) } else { (prio(c), d.bomb as f64, d.cmc as f64) };
        if best.as_ref().is_none_or(|b| key > b.1) {
            best = Some((c, key));
        }
    }
    best.map(|b| b.0)
}

const MANA_TAGS: [Tag; 6] = [Tag::Rock, Tag::Dork, Tag::Lr, Tag::Fastmana, Tag::Moxd, Tag::Chromemox];

/// ais.tutor_value: what fetching c is worth now: its card worth, mana sources discounted after turn 4, a bomb by
/// its bomb rating
pub fn tutor_value(g: &Game, p: PlayerId, c: CardId) -> f64 {
    let d = g.db.get(c);
    let mut v = card_worth(g, p, c, false);
    if g.player(p).turns > 4 && MANA_TAGS.iter().any(|&k| d.tag(k)) {
        v *= 0.3;
    }
    if d.bomb != 0 {
        v = v.max(10.0 * d.bomb as f64);
    }
    v
}

// ------------------------------------------------------------------ reanimation access
/// ais.rean_options: p's ways to reanimate: (card, how, generic, pips)
pub fn rean_options(g: &Game, p: PlayerId) -> Vec<(CardId, Sym, u32, String)> {
    let pl = g.player(p);
    let mut out = vec![];
    for &c in &pl.hand {
        let d = g.db.get(c);
        if d.tag(Tag::Rean) {
            out.push((c, "hand", d.generic, d.pips.to_string()));
        }
    }
    for &c in &pl.gy {
        let d = g.db.get(c);
        let Some(r) = d.tags.str(Tag::Rean) else { continue };
        match r {
            "rites" => out.push((c, "fb", 3, "W".into())),
            "dread" => {
                let fod = pl
                    .perms
                    .iter()
                    .filter(|&&m| {
                        let x = g.perm(m);
                        g.is_creature(m) && (x.token || x.cd.is_none_or(|cd| g.db.get(cd).bomb == 0))
                    })
                    .count();
                if fod >= 3 {
                    out.push((c, "fbsac", 0, String::new()));
                }
            }
            _ => {
                if pl.yawg {
                    out.push((c, "yawg", d.generic, d.pips.to_string()));
                }
            }
        }
    }
    out
}

/// ais.has_rean_access
pub fn has_rean_access(g: &Game, p: PlayerId) -> bool {
    !rean_options(g, p).is_empty()
}

/// ais.rean_targets: (value, card, whose graveyard), best first
pub fn rean_targets(g: &Game, p: PlayerId, kind: &str) -> Vec<(f64, CardId, PlayerId)> {
    let mut res = vec![];
    let need = crate::cardcode::loop_need(g, p);
    for &c in &g.player(p).gy {
        let d = g.db.get(c);
        let leg = d.tag(Tag::Leg);
        if d.creature && need.contains(&c) && !(kind == "persist" && leg) {
            res.push((9.5, c, p)); // the last piece of a loop
            continue;
        }
        if d.creature && super::seph_bval(g, p, c) >= 4.0 {
            if kind == "persist" && leg {
                continue;
            }
            res.push((super::seph_bval(g, p, c), c, p));
        }
    }
    if matches!(kind, "reanimate" | "animate" | "necro") {
        for q in g.opps(p) {
            for &c in &g.player(q).gy {
                let d = g.db.get(c);
                if d.creature && d.pow >= 5 {
                    res.push((d.pow as f64 - 0.5, c, q));
                }
            }
        }
    }
    res.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    res
}

/// ais.gy_worth: what card c in q's graveyard is worth to q (what Farewell's graveyard mode takes from them)
pub fn gy_worth(g: &Game, q: PlayerId, c: CardId) -> f64 {
    const GY_ENGINES: [&str; 3] = ["Meren of Clan Nel Toth", "Muldrotha, the Gravetide", "Sheoldred, Whispering One"];
    let d = g.db.get(c);
    let mut v = 0.25 * card_worth(g, q, c, true);
    if d.creature
        && (has_rean_access(g, q)
            || g.player(q).perms.iter().any(|&m| {
                let x = g.perm(m);
                !x.phased && x.cd.is_some_and(|cd| GY_ENGINES.contains(&&*g.db.get(cd).name))
            }))
    {
        let b = if d.bomb != 0 { d.bomb as f64 } else { 1.0 + 0.3 * d.pow as f64 };
        v = v.max(0.6 * b);
    }
    v
}

// ------------------------------------------------------------------ wipes
/// which permanents a wipe of this kind by `caster` hits (ais.wipe_hit: the one place each wipe's rule lives)
pub struct WipeRule {
    kind: Sym,
    caster: PlayerId,
    modes: Vec<Sym>,
    biggest: Option<PermId>,
    x: i32,
    victim: Option<PlayerId>,
}

impl WipeRule {
    pub fn new(g: &Game, caster: PlayerId, kind: &str) -> WipeRule {
        let kind = crate::sym::intern(kind);
        let mut r = WipeRule { kind, caster, modes: vec![], biggest: None, x: 0, victim: None };
        match kind {
            "farewell" | "austere2" => r.modes = wipe_modes(g, caster, kind),
            "rebuke" => r.victim = wipe_eval(g, caster, kind).2,
            "nib" => {
                let cr: Vec<PermId> = g.player(caster).perms.iter().copied().filter(|&m| g.is_creature(m)).collect();
                if let Some(b) = max_by(&cr, |m| epow(g, m) as f64) {
                    r.biggest = Some(b);
                    r.x = epow(g, b);
                }
            }
            _ => {}
        }
        r
    }

    /// does it hit permanent m, controlled by q?
    pub fn hits(&self, g: &Game, m: PermId, q: PlayerId) -> bool {
        let x = g.perm(m);
        let cd = x.cd.map(|c| g.db.get(c));
        match self.kind {
            "farewell" | "austere2" => mode_hit(g, &self.modes, m),
            "vandal" => q != self.caster && cd.is_some_and(|d| d.types.has(Types::ARTIFACT)),
            "rift" => q != self.caster,
            "rebuke" => Some(q) == self.victim,
            k => {
                if !g.is_creature(m) || Some(m) == self.biggest {
                    return false;
                }
                match k {
                    "dmg13" => etgh(g, m) <= 13,
                    "austere" => cd.is_some_and(|d| d.cmc >= 4),
                    "nib" => self.biggest.is_some() && etgh(g, m) <= self.x,
                    _ => true,
                }
            }
        }
    }
}

/// Farewell's and Austere Command's modes: does the wipe hit m?
fn mode_hit(g: &Game, modes: &[Sym], m: PermId) -> bool {
    let cd = g.perm(m).cd.map(|c| g.db.get(c));
    let ty = |t: Types| cd.map_or(t == Types::CREATURE, |d| d.types.has(t));
    let cre = g.is_creature(m);
    (modes.contains(&"art") && ty(Types::ARTIFACT))
        || (modes.contains(&"ench") && ty(Types::ENCHANTMENT))
        || (modes.contains(&"cre") && cre)
        || (modes.contains(&"le3") && cre && cd.is_none_or(|d| d.cmc <= 3))
        || (modes.contains(&"ge4") && cre && cd.is_some_and(|d| d.cmc >= 4))
}

/// ais.wipe_loss: what q loses to caster's wipe of this kind, by the wipe's own rule: bounced cards count half (they
/// can be recast), and a reanimator's bombs count less unless exiled
pub fn wipe_loss(g: &Game, q: PlayerId, kind: &str, caster: PlayerId) -> f64 {
    let rule = WipeRule::new(g, caster, kind);
    let rean = g.player(q).key == "seph" && has_rean_access(g, q);
    let mut loss = 0.0;
    for &m in &g.player(q).perms {
        let x = g.perm(m);
        if x.phased || !rule.hits(g, m, q) {
            continue;
        }
        let mut v = pval(g, m);
        if matches!(kind, "rift" | "evac" | "rebuke") && !x.token {
            v *= 0.5;
        }
        if rean && x.cd.is_some_and(|c| g.db.get(c).bomb != 0) && !matches!(kind, "exile" | "rift" | "evac" | "rebuke")
        {
            v *= 0.4;
        }
        loss += v;
    }
    loss
}

/// ais.wipe_eval: (what the opponents lose, what p loses, the one player a one-player wipe hits)
pub fn wipe_eval(g: &Game, p: PlayerId, kind: &str) -> (f64, f64, Option<PlayerId>) {
    let alive: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
    if matches!(kind, "farewell" | "austere2") {
        let modes = wipe_modes(g, p, kind);
        let (mut ol, mut ml) = (0.0, 0.0);
        for &q in &alive {
            for &m in &g.player(q).perms {
                if !g.perm(m).phased && mode_hit(g, &modes, m) {
                    if q == p {
                        ml += pval(g, m);
                    } else {
                        ol += pval(g, m);
                    }
                }
            }
        }
        return (ol, ml, None);
    }
    if kind == "vandal" {
        let ol = g
            .opps(p)
            .flat_map(|q| g.player(q).perms.iter().copied())
            .filter(|&m| g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT)))
            .map(|m| pval(g, m))
            .sum();
        return (ol, 0.0, None);
    }
    if matches!(kind, "rift" | "rebuke") {
        let bounced = |m: PermId| pval(g, m) * if g.perm(m).token || kind != "rift" { 1.0 } else { 0.5 };
        let vals: Vec<(PlayerId, f64)> = g
            .opps(p)
            .map(|q| (q, g.player(q).perms.iter().filter(|&&m| !g.perm(m).phased).map(|&m| bounced(m)).sum()))
            .collect();
        if vals.is_empty() {
            return (0.0, 0.0, None);
        }
        if kind == "rift" {
            return (vals.iter().map(|x| x.1).sum(), 0.0, None);
        }
        let v = max_by(&vals, |x| x.1).unwrap();
        return (v.1, 0.0, Some(v.0));
    }
    let mut biggest = None;
    let mut x = 0;
    if kind == "nib" {
        let cr: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m)).collect();
        let Some(b) = max_by(&cr, |m| epow(g, m) as f64) else { return (0.0, 0.0, None) };
        biggest = Some(b);
        x = epow(g, b);
    }
    let rean = g.player(p).key == "seph" && has_rean_access(g, p);
    let (mut ol, mut ml) = (0.0, 0.0);
    for &q in &alive {
        for &m in &g.player(q).perms {
            let pm = g.perm(m);
            if !g.is_creature(m) || pm.phased || Some(m) == biggest {
                continue;
            }
            let cmc = pm.cd.map(|c| g.db.get(c).cmc);
            let hit = match kind {
                "dmg13" => etgh(g, m) <= 13,
                "austere" | "austere2" => cmc.is_some_and(|v| v >= 4),
                "nib" => etgh(g, m) <= x,
                _ => true, // destroy, minus, exile, evac
            };
            if !hit {
                continue;
            }
            let mut v = pval(g, m);
            if kind == "evac" && !pm.token {
                v *= 0.5; // bounced cards can be recast
            }
            if q == p {
                if rean && pm.cd.is_some_and(|c| g.db.get(c).bomb != 0) && kind != "exile" {
                    v *= 0.4;
                }
                ml += v;
            } else {
                ol += v;
            }
        }
    }
    (ol, ml, None)
}

/// ais.wipe_cost: Cyclonic Rift overloaded, Vandalblast overloaded, or the card's cost
pub fn wipe_cost(g: &Game, p: PlayerId, c: CardId) -> (u32, String) {
    match g.db.get(c).tags.str(Tag::Wipe) {
        Some("rift") => (6, "U".into()),
        Some("vandal") => (4, "R".into()),
        _ => cost_of(g, p, c),
    }
}

/// ais.wipe_modes: Farewell's modes (any number) or Austere Command's (exactly two), by net value
pub fn wipe_modes(g: &Game, p: PlayerId, kind: &str) -> Vec<Sym> {
    // HUMAN(phase 9): a person picks the modes
    let val = |pred: &dyn Fn(PermId) -> bool| -> f64 {
        let mut v = 0.0;
        for q in g.players.iter().filter(|q| q.alive) {
            let s = if q.id == p { -1.2 } else { 1.0 };
            for &m in &q.perms {
                if !g.perm(m).phased && pred(m) {
                    v += s * pval(g, m);
                }
            }
        }
        v
    };
    let ty = |m: PermId, t: Types| g.perm(m).cd.map_or(t == Types::CREATURE, |c| g.db.get(c).types.has(t));
    let cmc = |m: PermId| g.perm(m).cd.map(|c| g.db.get(c).cmc);
    let mut opts: Vec<(Sym, f64)> =
        vec![("art", val(&|m| ty(m, Types::ARTIFACT))), ("ench", val(&|m| ty(m, Types::ENCHANTMENT)))];
    if kind == "farewell" {
        opts.push(("cre", val(&|m| g.is_creature(m))));
        // exile all graveyards: theirs (recursion) against yours (reanimation)
        let mut gyv = 0.0;
        for q in g.players.iter().filter(|q| q.alive) {
            let s = if q.id == p { -1.2 } else { 1.0 };
            gyv += s * q.gy.iter().map(|&c| gy_worth(g, q.id, c)).psum();
        }
        opts.push(("gy", gyv));
        let ch: Vec<Sym> = opts.iter().filter(|x| x.1 > 0.0).map(|x| x.0).collect();
        return if ch.is_empty() { vec!["cre"] } else { ch };
    }
    opts.push(("le3", val(&|m| g.is_creature(m) && cmc(m).is_none_or(|v| v <= 3))));
    opts.push(("ge4", val(&|m| g.is_creature(m) && cmc(m).is_some_and(|v| v >= 4))));
    opts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    opts.into_iter().take(2).map(|x| x.0).collect()
}

// ------------------------------------------------------------------ answers to removal and wipes
/// ais.pay_card: cast c in response (protection, a board-saving spell): pay, then it goes on the stack where
/// others may counter it. True if it resolves (the caller carries out its effect).
pub fn pay_card(g: &mut Game, p: PlayerId, c: CardId, kicked: u32) -> Res<bool> {
    let d = g.db.get(c);
    let pips = format!("{}{}", d.pips, if kicked > 0 { "W" } else { "" });
    let gn = d.generic + kicked;
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, gn, &pips, false) {
        return Ok(false);
    }
    remove_from_hand(g, p, c);
    pay(g, p, gn, &pips, false)?;
    let pl = g.player_mut(p);
    pl.gy.push(c);
    pl.spells_this_turn += 1;
    pl.cast_names.insert(c);
    crate::glog!(g, "  {} casts {}", g.player(p).name, g.db.get(c).name);
    on_cast(g, p, c)?;
    counter_window(g, p, c, 6.0, vec![])
}

pub fn remove_from_hand(g: &mut Game, p: PlayerId, c: CardId) {
    let hand = &mut g.player_mut(p).hand;
    if let Some(i) = hand.iter().position(|&x| x == c) {
        hand.remove(i);
    }
}

/// ais.protect_response: targeted removal of kind `kind` is aimed at owner's permanent m: respond if it's worth it
pub fn protect_response(
    g: &mut Game,
    owner: PlayerId,
    m: PermId,
    kind: Sym,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    if g.humans.get(owner).is_some() {
        return Ok(false); // practice mode: the person protects in their own window
    }
    let key = g.player(owner).key;
    if !is_main(key) {
        if silenced(g, owner) {
            return Ok(false);
        }
        return pool::protect(g, owner, m, kind, actor, spell);
    }
    let v = pval(g, m);
    if silenced(g, owner) {
        // Conqueror's Flail: only non-spell responses (Sephiroth's free sacrifice outlets)
        if key == "seph" {
            return super::seph::protect_silenced(g, owner, m, kind, v);
        }
        return Ok(false);
    }
    match key {
        "seph" => super::seph::protect(g, owner, m, kind, v),
        "veyran" => super::veyran::protect(g, owner, m, kind, actor, spell),
        "sauron" => {
            if g.perm(m).army || v >= 5.0 {
                let sl =
                    g.player(owner).hand.iter().copied().find(|&c| g.db.get(c).tags.str(Tag::Prot) == Some("phase"));
                if let Some(c) = sl
                    && can_pay(g, owner, 0, "U", false)
                {
                    if pay_card(g, owner, c, 0)? {
                        g.perm_mut(m).phased = true;
                        return Ok(true);
                    }
                    return Ok(false);
                }
                let nw =
                    g.player(owner).hand.iter().copied().find(|&c| g.db.get(c).tags.str(Tag::Prot) == Some("notw"));
                if let Some(c) = nw {
                    // Not of This World: free with a 7-power creature (the Army or Sauron)
                    let free =
                        g.player(owner).perms.iter().any(|&x| g.is_creature(x) && !g.perm(x).phased && epow(g, x) >= 7);
                    if free || can_pay(g, owner, 7, "", false) {
                        if !free {
                            pay(g, owner, 7, "", false)?;
                        }
                        remove_from_hand(g, owner, c);
                        let pl = g.player_mut(owner);
                        pl.gy.push(c);
                        pl.spells_this_turn += 1;
                        pl.cast_names.insert(c);
                        crate::glog!(g, "  {} casts {}", g.player(owner).name, g.db.get(c).name);
                        on_cast(g, owner, c)?;
                        return counter_window(g, owner, c, 6.0, vec![]);
                    }
                }
            }
            Ok(false)
        }
        // PORT(phase 6): Galadriel, Y'shtola, Jodah
        _ => Ok(false),
    }
}

/// ais.wipe_response: q's answer to a board wipe about to hit it: 'all' (everything saved), 'indes', or none
pub fn wipe_response(g: &mut Game, q: PlayerId, kind: Sym, caster: PlayerId) -> Res<Option<Sym>> {
    if q == caster || silenced(g, q) || g.humans.get(q).is_some() {
        return Ok(None);
    }
    let key = g.player(q).key;
    if !is_main(key) {
        return pool::wipe_response(g, q, kind, caster);
    }
    // PORT(phase 6): Galadriel's, Y'shtola's and Jodah's answers
    if matches!(key, "galadriel" | "yshtola" | "jodah") {
        return Ok(None);
    }
    let loss = wipe_loss(g, q, kind, caster);
    if loss < 6.0 {
        return Ok(None);
    }
    if key == "seph" {
        return super::seph::wipe_response(g, q, kind, loss);
    }
    if key == "sauron"
        && let Some(a) = army_of(g, q)
    {
        let sl = g.player(q).hand.iter().copied().find(|&c| g.db.get(c).tags.str(Tag::Prot) == Some("phase"));
        if let Some(c) = sl
            && can_pay(g, q, 0, "U", false)
            && pay_card(g, q, c, 0)?
        {
            g.perm_mut(a).phased = true;
        }
    }
    Ok(None)
}

/// ais.sauron_grounds_response: Sauron exiles all graveyards (Scavenger Grounds) in response to seph's reanimation
/// spell worth `value`. True if it did.
pub fn sauron_grounds_response(g: &mut Game, seph: PlayerId, value: f64) -> Res<bool> {
    for q in g.opps(seph).collect::<Vec<_>>() {
        if g.player(q).key != "sauron" || value < 6.0 || g.humans.get(q).is_some() {
            continue;
        }
        let gl = g.player(q).lands.iter().copied().find(|&l| {
            let d = g.db.get(g.land(l).cd);
            d.tag(Tag::Desert) && !g.land(l).tapped && !stopped(g, &d.name)
        });
        let Some(l) = gl else { continue };
        g.land_mut(l).tapped = true;
        if !can_pay(g, q, 2, "", false) || g.rng.random() > 0.9 {
            g.land_mut(l).tapped = false;
            continue;
        }
        pay(g, q, 2, "", false)?;
        let cd = g.land(l).cd;
        crate::engine::turn::remove_land(g, q, l);
        g.player_mut(q).gy.push(cd);
        g.player_mut(q).stat("grounds_used", 1);
        if !crate::engine::stack::ability_window(g, q, None, "exile all graveyards", Some(9.0), None)? {
            return Ok(false);
        }
        for pl in g.players.iter_mut().filter(|x| x.alive) {
            let gy = std::mem::take(&mut pl.gy);
            pl.exile.extend(gy);
        }
        g.player_mut(seph).stat("grounds_hit", 1);
        return Ok(true);
    }
    Ok(false)
}
