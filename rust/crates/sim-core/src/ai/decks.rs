//! Your decks' AI pieces from Python's `ais.py`: cast priorities (Sauron's in full; the others in phase 6), tutor
//! targets, Disruptor Flute's name, the wipe evaluation and mode choice, and the answers to removal and wipes.

use crate::ai::hand_key; // --- phase4-hand-off
use super::{is_main, pool};
use crate::cards::Types;
use crate::engine::cast::on_cast;
use crate::engine::combat::blocked;
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::removal::legal_targets;
use crate::engine::stack::{counter_window, equip_to};
use crate::engine::tutors::card_worth;
use crate::engine::values::{army_of, epow, equipped, etgh, find, has, pval, silenced, threat};
use crate::engine::zones::{max_by, searchable};
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

/// ais.bomb_on_bf
pub fn bomb_on_bf(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| {
        let x = g.perm(m);
        !x.phased && x.cd.is_some_and(|c| super::reanimate::sbomb(g, c) >= super::reanimate::BOMB_BOARD)
    })
}

// ------------------------------------------------------------------ cast priorities
/// ais.deck_prio: the deck's cast priority for a card (0-90)
pub fn deck_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let v = match hand_key(g, p) {
        "yshtola" => crate::impls::yshtola::yshtola_prio(g, p, c) as i32,
        "omnath" => crate::impls::titania::titania_prio(g, p, c),
        "sheoldred" => crate::impls::sheoldred::sheoldred_prio(g, p, c),
        "cloud" => crate::impls::cloud::cloud_prio(g, p, c),
        "kraum" => crate::impls::tymna_partners::kraum_prio(g, p, c),
        "garland" => crate::impls::garland::garland_prio(g, p, c),
        "sephfs" => crate::impls::sephfs::sephfs_prio(g, p, c),
        _ => pool::generic_prio(g, p, c),
    };
    if v == 0 && g.db.get(c).tag(Tag::Top) {
        // a card-to-top tutor (Worldly, Mystical) the deck's own priority doesn't know: the card's own timing
        if let Some(f) = g.registry.get(c).and_then(|i| i.prio) {
            return f(g, p, c);
        }
    }
    v
}

/// ais.deck_prio as Python returns it: a float where a deck's priority isn't whole (Y'shtola's for a card taken from
/// an opponent: its ability-language value)
pub fn deck_prio_f(g: &Game, p: PlayerId, c: CardId) -> f64 {
    if let Some(v) = crate::impls::t6_plans::prio(g, p, c) {
        return v; // a Tier 6 deck's own plan slot
    }
    match hand_key(g, p) {
        "yshtola" => crate::impls::yshtola::yshtola_prio(g, p, c),
        _ => deck_prio(g, p, c) as f64,
    }
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
    thread_local! {
        static IN_GAMBLE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    }
    // Valuing the hand can come back here: Underworld Breach in hand values the graveyard, and a Gamble there asks
    // for this priority again, without end (a stack overflow, sauron vs t4, seeds 500298 and 500841). Inside that
    // loop Gamble is worth nothing.
    if IN_GAMBLE.with(|f| f.replace(true)) {
        return 0;
    }
    let rest: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&x| x != c).collect();
    let v = if rest.is_empty() {
        0
    } else {
        let keep = rest.len() as f64 / (rest.len() + 1) as f64;
        let risk = rest.iter().filter(|&&x| !g.db.get(x).land).map(|&x| card_worth(g, p, x, false)).psum()
            / rest.len().max(1) as f64
            / 10.0;
        (60.0 * keep - 8.0 * risk).max(0.0) as i32
    };
    IN_GAMBLE.with(|f| f.set(false));
    v
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
        // the kraum seat escapes its own wipes (Austere Command ...) when the table is far ahead on the board
        let kraum_wipe = t.has(Tag::Wipe) && !t.has(Tag::Ctr) && pl.key == "kraum" && behind_on_board(g, p);
        if d.land || t.has(Tag::Ctr) || (t.has(Tag::Wipe) && !kraum_wipe) || t.has(Tag::Breach) {
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
        if kraum_wipe {
            v = v.max(40.0);
        }
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

pub(crate) fn untap_monolith(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
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
            if let Some(c2) = ql.cmd2 {
                add(c2, if ql.cmd2_in_zone { 4.0 } else { 1.5 } + if ql.tax2 >= 2 { 1.0 } else { 0.0 });
            }
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
                v = v.max(1.0);
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
            let d = g.db.get(g.perm(l).card());
            if d.tag(Tag::Passage) && army_of(g, q).is_some() {
                add(d.id, 3.0);
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
        "ae" => d.types.has(Types::ARTIFACT) || d.types.has(Types::ENCHANTMENT), // --- ai-fix: B: Enlightened Tutor
        _ => true,
    }
}

/// the kinds TUTOR_OK knows (any other kind finds anything)
fn tutor_ok_known(kind: &str) -> &str {
    match kind {
        "is" | "art" | "perm" | "ubr" | "cre2" | "cre" | "ench" | "ae" => kind, // --- ai-fix: B: "ae"
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
    match hand_key(g, p) {
        "yshtola" => crate::impls::yshtola::tutor_pick(g, p, kind, &okn),
        "cloud" if matches!(kind, "ae" | "art") => crate::impls::cloud::tutor_pick(g, p, &okn),
        "kraum" => crate::impls::tymna_cards::tutor_pick(g, p, kind, &okn),
        "sephfs" => crate::impls::sephfs::tutor_pick(g, p, kind, &okn),
        "omnath" if kind == "cre" => crate::impls::titania::worldly_pick(g, p, &okn),
        _ => None,
    }
}

/// ais.tutor_pick: deck-specific wish lists first; otherwise the highest-priority legal card
pub fn tutor_pick(g: &Game, p: PlayerId, kind: &str) -> Option<CardId> {
    // --- ai-planner: the line planner's choice (`Player::plan_tutor`), when it is in the library and meets the
    // tutor's restriction; otherwise the usual pick
    if let Some(c) = g.player(p).plan_tutor
        && searchable(g, p).contains(&c)
        && tutor_ok(g, tutor_ok_known(kind), c)
    {
        return Some(c);
    }
    if let Some(c) = tutor_pick_named(g, p, kind) {
        return Some(c);
    }
    let okk = tutor_ok_known(kind);
    let main = is_main(hand_key(g, p));
    if !main && let Some(c) = pool::tutor_pick(g, p, |c| tutor_ok(g, okk, c)) {
        return Some(c); // outside deck: its wish list
    }
    let prio = |c: CardId| -> f64 {
        let v = deck_prio_f(g, p, c);
        if main || v != 0.0 {
            v
        } else if g.db.get(c).has_dsl() {
            crate::dsl::card_value(g, p, c) * 10.0
        } else {
            0.0
        }
    };
    let top = crate::engine::tutors::known_top(g, p); // --- ai-fix: B: not the card it already put on top
    let cands: Vec<CardId> = searchable(g, p)
        .into_iter()
        .filter(|&c| tutor_ok(g, okk, c) && !g.db.get(c).land && !top.contains(&c))
        .collect();
    if cands.is_empty() {
        return None;
    }
    let mut best: Option<(CardId, (f64, f64, f64))> = None;
    let late = g.player(p).turns > 4; // past the early turns: impact, not cast priority (no Sol Ring on turn 10)
    for c in cands {
        let d = g.db.get(c);
        let key = if late {
            (tutor_value(g, p, c), d.cmc as f64, 0.0)
        } else {
            (prio(c), super::values::bomb_of(g, p, c), d.cmc as f64) // --- phase4-values: the reader's bomb
        };
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
    if super::values::on(g, p) {
        // --- phase4-values: the derived threat, from a bomb up
        let b = super::values::threat(g, c);
        if b >= super::values::BOMB {
            v = v.max(10.0 * b);
        }
    } else if d.bomb != 0 {
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
                // --- phase4-values: fodder is no bomb by the reader's reading
                let fodder_max = if super::values::on(g, p) { super::values::BOMB } else { f64::MIN_POSITIVE };
                let fod = pl
                    .perms
                    .iter()
                    .filter(|&&m| {
                        let x = g.perm(m);
                        g.is_creature(m)
                            && (x.token || x.cd.is_none_or(|cd| super::values::bomb_of(g, p, cd) < fodder_max))
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
    if g.player(p).key == "sephfs" {
        return crate::impls::sephfs::rean_targets(g, p, kind);
    }
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
        let bomb = if super::values::on(g, q) {
            Some(super::values::threat(g, c)).filter(|&b| b >= super::values::BOMB) // --- phase4-values
        } else {
            Some(d.bomb as f64).filter(|&b| b != 0.0)
        };
        let b = bomb.unwrap_or(1.0 + 0.3 * d.pow as f64);
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
    /// the sweepers of impls/sweepers.rs (Crippling Fear, Widespread Brutality, Chandra's Ignition)
    ext: Option<crate::impls::sweepers::Sweep>,
}

impl WipeRule {
    pub fn new(g: &Game, caster: PlayerId, kind: &str) -> WipeRule {
        let kind = crate::sym::intern(kind);
        let mut r = WipeRule { kind, caster, modes: vec![], biggest: None, x: 0, victim: None, ext: None };
        match kind {
            k if crate::impls::sweepers::is_kind(k) => {
                r.ext = Some(crate::impls::sweepers::Sweep::new(g, caster, k, false))
            }
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
        if let Some(s) = &self.ext {
            return s.hits(g, m);
        }
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
        loss += v;
    }
    loss
}

/// ais.wipe_eval: (what the opponents lose, what p loses, the one player a one-player wipe hits)
pub fn wipe_eval(g: &Game, p: PlayerId, kind: &str) -> (f64, f64, Option<PlayerId>) {
    if crate::impls::sweepers::is_kind(kind) {
        return crate::impls::sweepers::eval(g, p, kind);
    }
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
                "minus" => crate::impls::sheoldred::deluge_cap(g, p).is_none_or(|cap| etgh(g, m) <= cap),
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
    // (a person's modes: human::choices::wipe_modes, called by engine::removal)
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
    let ty = |m: PermId, t: Types| g.perm(m).cd.map_or(t == Types::CREATURE, |_| crate::engine::values::ptypes(g, m).has(t));
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

/// ais.protect_response: targeted removal of kind `kind` is aimed at owner's permanent m: respond if it's worth it; a
/// creature that can't be saved is sacrificed to a free outlet when that gains something
pub fn protect_response(
    g: &mut Game,
    owner: PlayerId,
    m: PermId,
    kind: Sym,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    if crate::impls::tymna_cards::protect(g, owner, m, kind, actor, spell)? {
        return Ok(true); // Razia redirects burn, King Darien saves tokens
    }
    if redirect_response(g, owner, m, kind, actor, spell)? {
        return Ok(true); // Redirect Lightning, Ricochet Trap, Untimely Malfunction, Deflecting Swat
    }
    if protect_main(g, owner, m, kind, actor, spell)? {
        return Ok(true);
    }
    if g.humans.get(owner).is_some()
        || !is_main(hand_key(g, owner))
        || silenced(g, owner)
        || !g.perm(m).on_bf
    {
        return Ok(false); // (the pools sacrifice in pool::protect)
    }
    crate::cardcode::sac_in_response(g, owner, m, kind)
}

fn protect_main(
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
    let key = hand_key(g, owner);
    if !is_main(key) {
        if silenced(g, owner) {
            return Ok(false);
        }
        if key == "sephfs" && crate::impls::sephfs_cards::presumed_protect(g, owner, m, kind)? {
            return Ok(false); // Presumed Dead: the removal goes through, the creature comes back (suspected)
        }
        if crate::impls::t3_korvold::rebirth_protect(g, owner, m, kind)? {
            return Ok(false); // Malakir Rebirth (Tier 3 Tergrid): the removal goes through, the creature comes back tapped
        }
        if crate::impls::t45_chulane_kinnan::protect(g, owner, m, kind, actor, spell)? {
            return Ok(true); // --- Tiers 4/5 (chulane-kinnan): Spellskite redirects, Nezahal exiles itself
        }
        if key == "kraum"
            && crate::impls::bulk_white::white_protect(g, owner, m, kind, actor, spell)?
        {
            return Ok(true); // Shardmage's Rescue, Ultimate Magic: Holy for Tymna and the partners
        }
        if crate::impls::t12_lightpaws::protect(g, owner, m, kind, actor, spell)? {
            return Ok(true); // --- Tiers 1/2 (lightpaws): the Light-Paws deck's flash Auras, Blessings, Flickerform
        }
        return pool::protect(g, owner, m, kind, actor, spell);
    }
    if silenced(g, owner) {
        return Ok(false);
    }
    if key == "cloud" && crate::impls::bulk_white::white_protect(g, owner, m, kind, actor, spell)? {
        return Ok(true); // Patriot, Shardmage's Rescue, Cho-Manno's Blessing, Ultimate Magic: Holy
    }
    match key {
        "yshtola" => crate::impls::yshtola::yshtola_protect(g, owner, m, kind, actor, spell),
        "omnath" => crate::impls::titania::titania_protect(g, owner, m, kind, actor, spell),
        "sheoldred" => crate::impls::sheoldred::sheoldred_protect(g, owner, m, kind, actor, spell),
        "cloud" => crate::impls::cloud::cloud_protect(g, owner, m, kind, actor, spell),
        _ => Ok(false),
    }
}

/// ais.wipe_response: q's answer to a board wipe about to hit it: 'all' (everything saved), 'indes', or none
/// the wipe will kill these anyway: sacrifice them to an outlet first (the free ones; a paid one too for a deck with
/// the `paid_outlets` plan knob), the least worth keeping first
fn sac_doomed(g: &mut Game, q: PlayerId, kind: Sym, caster: PlayerId) -> Res {
    if matches!(kind, "rift" | "evac" | "rebuke" | "vandal") {
        return Ok(());
    }
    let rule = WipeRule::new(g, caster, kind);
    let mut doomed: Vec<PermId> = g
        .player(q)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && !g.perm(m).phased && rule.hits(g, m, q))
        .collect();
    doomed.sort_by(|&a, &b| crate::engine::zones::sac_worth(g, a).total_cmp(&crate::engine::zones::sac_worth(g, b)));
    crate::cardcode::sac_before_wipe(g, q, &doomed)?;
    Ok(())
}

pub fn wipe_response(g: &mut Game, q: PlayerId, kind: Sym, caster: PlayerId) -> Res<Option<Sym>> {
    crate::impls::tymna_cards::darien_wipe(g, q, kind, caster)?; // King Darien: tokens survive a destroy sweeper
    if q == caster || silenced(g, q) || g.humans.get(q).is_some() {
        return Ok(None);
    }
    let key = hand_key(g, q);
    if !is_main(key) {
        if key == "kraum"
            && let Some(r) = crate::impls::bulk_white::white_wipe_response(g, q, kind, caster)?
        {
            return Ok(Some(r)); // Ultimate Magic: Holy
        }
        crate::impls::t12_lightpaws::wipe_response(g, q, kind, caster)?; // --- Tiers 1/2 (lightpaws): Flickerform
        let r = pool::wipe_response(g, q, kind, caster)?;
        if r.is_none() && crate::impls::common::paid_outlets_on(g, q) {
            sac_doomed(g, q, kind, caster)?; // --- ghave: unprotected: Ghave's {1} outlet (and the free ones) first
        }
        return Ok(r);
    }
    if key == "yshtola" {
        return crate::impls::yshtola::yshtola_wipe_response(g, q, kind, caster);
    }
    if key == "cloud"
        && let Some(r) = crate::impls::bulk_white::white_wipe_response(g, q, kind, caster)?
    {
        return Ok(Some(r)); // Ultimate Magic: Holy
    }
    if key == "cloud"
        && let Some(r) = crate::impls::cloud::cloud_wipe_response(g, q, kind, caster)?
    {
        return Ok(Some(r)); // Unbreakable Formation, Rootborn Defenses
    }
    if !g.marchesa_on // (her return counts the creatures that die together)
        && !crate::cardcode::outlets_of(g, q).is_empty()
    {
        sac_doomed(g, q, kind, caster)?;
    }
    let loss = wipe_loss(g, q, kind, caster);
    if loss < 6.0 {
        return Ok(None);
    }
    Ok(None)
}

/// ais.necro_pay: Necropotence, pay 1 life per card (exiled face down, to hand at this end step); keep life at the
/// floor (`necro_floor`: what the table could hit you for) and don't overshoot the hand size
pub fn necro_pay(g: &mut Game, p: PlayerId, floor: Option<i32>) -> Res {
    if blocked(g, p, "Necropotence") {
        return Ok(());
    }
    let floor = floor.unwrap_or_else(|| super::necro_floor(g, p));
    let pl = g.player(p);
    let n = 0.max((pl.life - floor).min(8 - pl.hand.len() as i32).min(pl.library.len() as i32));
    if n == 0 {
        return Ok(());
    }
    crate::engine::life::lose_life(g, p, n, Some(p), "other", None)?;
    for _ in 0..n {
        let pl = g.player_mut(p);
        let c = pl.library.pop().unwrap();
        pl.hand.push(c);
        pl.seen.insert(c);
    }
    g.player_mut(p).stat("necro_cards", n as i64);
    crate::glog!(g, "  {} pays {n} life to Necropotence", g.player(p).name);
    crate::t6stats::flow_life(g, p, "Necropotence", n as i64); // --- ai-flow
    crate::t6stats::flow_engine_cards(g, p, "Necropotence", n as i64);
    Ok(())
}

/// ais.necro_deliver: the beginning of p's end step: the cards Necropotence exiled for this end step (a person's,
/// waiting in `necro_wait`) go to p's hand
pub fn necro_deliver(g: &mut Game, p: PlayerId) -> Res {
    let turns = g.player(p).turns;
    let pl = g.player_mut(p);
    if pl.necro_wait.is_empty() {
        return Ok(());
    }
    let (now, later): (Vec<_>, Vec<_>) = std::mem::take(&mut pl.necro_wait).into_iter().partition(|&(_, r)| r <= turns);
    pl.necro_wait = later;
    if now.is_empty() {
        return Ok(());
    }
    let cards: Vec<CardId> = now.iter().map(|x| x.0).collect();
    pl.hand.extend(cards.iter().copied());
    pl.seen.extend(cards.iter().copied());
    pl.stat("necro_cards", cards.len() as i64);
    let name = g.player(p).name.to_string();
    let names: Vec<&str> = cards.iter().map(|&c| &*g.db.get(c).name).collect();
    let (public, line) = (
        format!("  {name} puts {} card(s) into their hand (Necropotence)", cards.len()),
        format!("  {name} puts {} into their hand (Necropotence)", names.join(", ")),
    );
    g.log_secret(p, &public, &line);
    Ok(())
}

/// ais.braids_sacrifice: Braids, Cabal Minion: at the beginning of each player's upkeep, that player sacrifices an
/// artifact, creature, or land (the cheapest to lose)
pub fn braids_sacrifice(g: &mut Game, p: PlayerId) -> Res {
    enum X {
        Perm(PermId),
        Land(crate::ids::PermId),
    }
    let mut opts: Vec<(f64, X)> = vec![];
    for &m in &g.player(p).perms {
        if g.perm(m).phased {
            continue;
        }
        if g.is_creature(m) {
            opts.push((pval(g, m), X::Perm(m)));
        } else if let Some(c) = g.perm(m).cd
            && g.db.get(c).types.has(Types::ARTIFACT)
        {
            let v = if g.db.get(c).tag(Tag::Rock) { 2.5 } else { pval(g, m).max(1.0) };
            opts.push((v, X::Perm(m)));
        }
    }
    let nl = g.player(p).lands.len();
    for &l in &g.player(p).lands {
        opts.push((if nl >= 6 { 3.0 } else { 6.0 }, X::Land(l)));
    }
    let mut best: Option<usize> = None;
    for (i, o) in opts.iter().enumerate() {
        if best.is_none_or(|b| o.0 < opts[b].0) {
            best = Some(i);
        }
    }
    let Some(b) = best else { return Ok(()) };
    match opts.swap_remove(b).1 {
        X::Land(l) => {
            let cd = g.perm(l).card();
            crate::engine::turn::remove_land(g, p, l);
            g.player_mut(p).gy.push(cd);
            crate::engine::zones::tergrid_steal(g, p, cd, p)?;
            crate::glog!(g, "    Braids: {} sacrifices {}", g.player(p).name, g.db.get(cd).name);
        }
        X::Perm(m) => {
            crate::glog!(g, "    Braids: {} sacrifices {}", g.player(p).name, g.perm(m).name);
            crate::engine::zones::die(g, m, "sac")?;
        }
    }
    g.player_mut(p).stat("braids_sacs", 1);
    Ok(())
}

// ------------------------------------------------------------------ the Underworld Breach line's pieces
/// a card of the Underworld Breach line other than Breach itself: a storm card (Brain Freeze, Grapeshot) or a ritual
fn is_storm_card(g: &Game, c: CardId) -> bool {
    matches!(&*g.db.get(c).name, "Brain Freeze" | "Grapeshot")
}

fn is_ritual_card(g: &Game, c: CardId) -> bool {
    matches!(&*g.db.get(c).name, "Dark Ritual" | "Cabal Ritual" | "Lotus Petal" | "Lion's Eye Diamond" | "Jeska's Will")
}

/// Underworld Breach in hand or on the battlefield
pub fn breach_out(g: &Game, p: PlayerId) -> bool {
    has(g, p, Tag::Breach) || in_hand_tag(g, p, Tag::Breach)
}

/// a storm card in hand or graveyard (the graveyard one is escaped by the Breach)
pub fn storm_held(g: &Game, p: PlayerId) -> bool {
    let pl = g.player(p);
    pl.hand.iter().chain(pl.gy.iter()).any(|&c| is_storm_card(g, c))
}

/// the cheap test for the line: Breach out and a storm card in hand or graveyard
pub fn breach_pieces(g: &Game, p: PlayerId) -> bool {
    breach_out(g, p) && storm_held(g, p)
}

/// worth added to a line card's discard score: Breach, a storm card or a ritual that the other half of the line is
/// waiting for isn't thrown away (a storm card with no Breach in sight goes to the graveyard happily: it is escaped)
pub fn breach_keep_bonus(g: &Game, p: PlayerId, c: CardId) -> f64 {
    let t = &g.db.get(c).tags;
    let keep = if t.has(Tag::Breach) {
        storm_held(g, p)
    } else if is_storm_card(g, c) || t.has(Tag::Led) {
        has(g, p, Tag::Breach) || g.player(p).hand.iter().any(|&x| x != c && g.db.get(x).tag(Tag::Breach))
    } else if is_ritual_card(g, c) {
        breach_pieces(g, p)
    } else {
        false
    };
    if keep { 80.0 } else { 0.0 }
}

/// Any deck holding a redirect card (Redirect Lightning, Ricochet Trap, Untimely Malfunction, Deflecting Swat) turns
/// removal aimed at its key permanent on the caster's own best permanent. Veyran and Kefka run Swat in their own
/// `protect` (after their other options), so they only get the three here.
fn redirect_response(
    g: &mut Game,
    owner: PlayerId,
    m: PermId,
    kind: Sym,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    if spell.is_none() || g.humans.get(owner).is_some() || silenced(g, owner) || !g.perm(m).on_bf {
        return Ok(false);
    }
    let hand = &g.player(owner).hand;
    let holds = |n: &str| g.db.id(n).is_some_and(|c| hand.contains(&c));
    let (cards, swat) = (
        holds("Redirect Lightning") || holds("Ricochet Trap") || holds("Untimely Malfunction"),
        holds("Deflecting Swat") && !matches!(g.player(owner).key, "veyran" | "kefka"),
    );
    if cards && crate::impls::t6_redirects::redirect_removal(g, owner, m, kind, actor, spell)? {
        return Ok(true);
    }
    if swat && crate::impls::mine::deflecting_swat(g, owner, m, kind, actor, spell)? {
        return Ok(true);
    }
    Ok(false)
}

/// the kraum seat's wipe rule (also the Diabolic Tutor one): the strongest opponent's creature power is 6 or more above mine
fn behind_on_board(g: &Game, p: PlayerId) -> bool {
    let pow = |q: PlayerId| -> i32 {
        g.player(q)
            .perms
            .iter()
            .filter(|&&m| g.perm(m).on_bf && !g.perm(m).phased && g.is_creature(m))
            .map(|&m| crate::engine::values::epow(g, m))
            .sum()
    };
    let theirs = g.opps(p).filter(|&q| g.player(q).alive).map(pow).max().unwrap_or(0);
    theirs >= pow(p) + 6
}

