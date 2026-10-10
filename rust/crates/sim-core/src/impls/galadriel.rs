//! Python's `cards/impl/galadriel.py`: Galadriel, Light of Valinor (your Bant Rebels deck, key `galadriel`).
//!
//! - Galadriel's Alliance trigger: each other creature entering picks a mode not yet chosen this turn ({G}{G}{G}, a
//!   +1/+1 counter on each creature you control, or scry 2 and draw).
//! - The Rebel chain: Ramosian Sergeant, Lieutenant, Captain and Commander, Defiant Vanguard and Lin Sivvi search for
//!   a Rebel permanent card with a mana value cap and put it onto the battlefield; Ramosian Revivalist returns one
//!   from the graveyard. Maskwood Nexus makes every creature card you own a Rebel (and every other type).
//! - Creature types: Kindred Discovery, Door of Destinies, Patchwork Banner, Vanquisher's Banner and Secluded
//!   Courtyard name a type as they enter.
//! - The other cards that need code: Panharmonicon, Abduction, Bribery, Crackdown, Mangara, Tocasia's Welcome, Voice
//!   of Resurgence, Eerie Interlude, Flicker, Planar Genesis, Return to Dust, Unbreakable Formation, Lawbringer,
//!   Lightbringer, Ballista Squad, Errant Doomsayers, Whipcorder, Knight of the Holy Nimbus, Cho-Manno, Amrou
//!   Seekers, Elvish Archdruid, Springleaf Drum, Grand Coliseum (Cho-Manno, Amrou Seekers, the Gliders, Jhovall
//!   Queen, Grand Coliseum and the searchers' `noatk` are tags: nothing to port).
//! - The AI: cast priorities, which Rebel to fetch, protection and wipe responses, answers when attacked, the
//!   tappers before an opponent's combat.
//!
//! Not ported: `galadriel_abilities` (it returns False, and only Python's old-mode main loop calls it; the Rust has no
//! old mode). The person's choices (practice mode) are `HUMAN(phase 9)`. A few functions of modules other agents
//! port (marchesa.best_steal / steal, zur.guildmage_target / populate / rootborn, t2.blink / blink_value) have
//! stand-ins at the end of this file: swap them for those modules' functions once they land.

use super::partials::at_once;
use crate::ai::decks::{pay_card, protect_response};
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::cast_card;
use crate::engine::combat::{can_block, has_haste};
use crate::engine::mana::{can_pay, pay, total_mana};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::{ability_window, ability_window_card, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{
    ALL_TYPES, card_name, colors_of, epow, etgh, has_type, indestructible, maskwood as maskwood_on, no_damage,
    once_per_turn, protected_from, pval, subtypes, untargetable,
};
use crate::engine::zones::{
    Enter, Tokens, Zone, die, draw, enter, landfall, leave, make_tokens, max_by, searchable, to_zone_card,
};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::pysum::PySum;
use crate::state::{Ctx, DataKey, Game, Step, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

// ------------------------------------------------------------------ helpers
/// galadriel.named: p's first permanent of this name, not phased out
pub fn named(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| !g.perm(m).phased && card_name(g, m) == Some(name))
}

/// the first of xs with the highest key (Python's `max(xs, key=..)` for tuple keys)
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

/// the first of xs with the lowest key (Python's `min(xs, key=..)` for tuple keys)
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

/// `m in p.perms`
fn controls(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.perm(m).on_bf && g.perm(m).owner == p
}

/// `m in m.owner.perms`
fn on_bf(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

fn name_of(g: &Game, m: PermId) -> Sym {
    g.perm(m).name
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

/// remove the first copy of c (Python's `list.remove`)
fn remove_first(v: &mut Vec<CardId>, c: CardId) -> bool {
    match v.iter().position(|&x| x == c) {
        Some(i) => {
            v.remove(i);
            true
        }
        None => false,
    }
}

/// `g.eot_kw.setdefault(id(m), set()).add(kw)`
fn add_kw(g: &mut Game, m: PermId, kw: Sym) {
    let k = &mut g.perm_mut(m).eot_kw;
    if !k.contains(&kw) {
        k.push(kw);
    }
}

fn ability(utility: f64, label: String, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Ability { src, f, arg }) }
}

fn in_main(g: &Game) -> bool {
    matches!(g.step, Step::Main1 | Step::Main2)
}

/// the enters hook that does nothing: it makes the card's code live (Python's `CI.live`)
fn live(_g: &mut Game, _src: Src, _p: PlayerId, _m: PermId) -> Res {
    Ok(())
}

// ================================================================== creature types
/// galadriel.card_is: is card c (in p's library, hand or graveyard) of creature type t
pub fn card_is(g: &Game, p: PlayerId, c: CardId, t: &str) -> bool {
    let d = g.db.get(c);
    d.has_subtype(t) || d.has_kw("changeling") || (d.creature && maskwood_on(g, p))
}

/// galadriel.type_counts: the creature types of p's creature cards (hand, library, graveyard, battlefield, commander),
/// in the order first seen. Python counts with a Counter over each card's subtypes, a frozenset whose order is the
/// string hash's (it differs from run to run): here each card's printed order.
fn type_counts(g: &Game, p: PlayerId) -> Vec<(Sym, i32)> {
    let pl = g.player(p);
    let mut zones: Vec<CardId> = pl.hand.iter().chain(pl.library.iter()).chain(pl.gy.iter()).copied().collect();
    zones.extend(pl.perms.iter().filter(|&&m| g.is_creature(m)).filter_map(|&m| g.perm(m).cd));
    zones.push(pl.cmd);
    let mut n: Vec<(Sym, i32)> = vec![];
    for c in zones {
        let d = g.db.get(c);
        if !d.creature {
            continue;
        }
        for t in d.subtypes.iter() {
            let t = intern(t);
            match n.iter_mut().find(|x| x.0 == t) {
                Some(x) => x.1 += 1,
                None => n.push((t, 1)),
            }
        }
    }
    n
}

/// galadriel.choose_type: the creature type a permanent names as it enters: the AI names its deck's main type (Orc
/// for Sauron's Kindred Discovery). HUMAN(phase 9): a person chooses.
pub fn choose_type(g: &Game, p: PlayerId, _what: &str) -> Sym {
    let counts = type_counts(g, p);
    if g.player(p).key == "sauron" {
        return "orc";
    }
    let mut on_bf: Vec<(Sym, i32)> = vec![];
    for &m in &g.player(p).perms {
        if !g.is_creature(m) {
            continue;
        }
        let mut seen: Vec<Sym> = vec![];
        for t in subtypes(g, m) {
            if seen.contains(&t) {
                continue; // Python's subtypes is a set
            }
            seen.push(t);
            match on_bf.iter_mut().find(|x| x.0 == t) {
                Some(x) => x.1 += 1,
                None => on_bf.push((t, 1)),
            }
        }
    }
    let bf = |t: Sym| on_bf.iter().find(|x| x.0 == t).map_or(0, |x| x.1);
    first_max(&counts, |x| x.1 + 2 * bf(x.0)).map_or("human", |x| x.0)
}

/// galadriel.ctype: the creature type a permanent named
pub fn ctype(g: &Game, m: PermId) -> Option<Sym> {
    match g.perm(m).data.get(DataKey::Ctype) {
        Some(Val::Str(s)) => Some(*s),
        _ => None,
    }
}

/// galadriel._name_type (AS_ENTERS of Kindred Discovery, Door of Destinies, Patchwork Banner, Vanquisher's Banner)
fn name_type(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    let name = g.perm(m).cd.map_or("", |c| &*g.db.get(c).name).to_string();
    let t = choose_type(g, p, &name);
    g.perm_mut(m).data.set(DataKey::Ctype, Val::Str(t));
    crate::glog!(g, "    {} names {}", name, title(t));
    g.selfpt = true; // the Banners and the Door change creatures' size
    Ok(())
}

/// Python's `str.title()` for a creature type
fn title(t: &str) -> String {
    let mut out = String::new();
    let mut up = true;
    for ch in t.chars() {
        if up {
            out.extend(ch.to_uppercase());
        } else {
            out.push(ch);
        }
        up = !ch.is_alphabetic();
    }
    out
}

/// galadriel.typed_pt (common.CREATURE_PT): the Banners (+1/+1) and Door of Destinies (+1/+1 per charge counter) for
/// creatures of the named type
fn typed_pt(g: &Game, m: PermId) -> (i32, i32) {
    let mut b = 0;
    for &x in &g.player(g.perm(m).owner).perms {
        let px = g.perm(x);
        let Some(c) = px.cd else { continue };
        let n = &*g.db.get(c).name;
        if px.phased || !matches!(n, "Patchwork Banner" | "Vanquisher's Banner" | "Door of Destinies") {
            continue;
        }
        let Some(t) = ctype(g, x) else { continue };
        if !has_type(g, m, t) {
            continue;
        }
        b += if n == "Door of Destinies" { px.data.int(DataKey::Charge) as i32 } else { 1 };
    }
    (b, b)
}

/// Door of Destinies: names a creature type as it enters; a charge counter for each spell of that type you cast;
/// creatures of that type get +1/+1 per counter
fn door(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let t = ctype(g, src);
    let Some(t) = t else { return Ok(()) };
    if caster != g.perm(src).owner || !card_is(g, caster, c, t) {
        return Ok(());
    }
    if !trigger_window(g, caster, Some(src), "a charge counter", None)? {
        return Ok(());
    }
    let n = g.perm(src).data.int(DataKey::Charge);
    g.perm_mut(src).data.set(DataKey::Charge, Val::Int(n + 1));
    Ok(())
}

/// Vanquisher's Banner: names a creature type as it enters: those creatures get +1/+1, and you draw a card for each
/// creature spell of that type you cast
fn vanquisher(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let Some(t) = ctype(g, src) else { return Ok(()) };
    if caster != g.perm(src).owner || !g.db.get(c).creature || !card_is(g, caster, c, t) {
        return Ok(());
    }
    if trigger_window(g, caster, Some(src), "draw a card", None)? {
        draw(g, caster, 1, false)?;
    }
    Ok(())
}

/// Secluded Courtyard: any colour only for creature spells of the named type (and abilities of those creatures)
fn courtyard_cols(g: &Game, p: PlayerId, l: LandId) -> Colors {
    let t = match g.land(l).data.get(DataKey::Ctype) {
        Some(Val::Str(s)) => Some(*s),
        _ => None,
    };
    if let (Some(t), Some(spell)) = (t, g.pay_for)
        && g.db.get(spell).creature
        && card_is(g, p, spell, t)
    {
        return g.player(p).ident;
    }
    Colors::NONE
}

/// Secluded Courtyard names a creature type as it enters
fn courtyard_etb(g: &mut Game, p: PlayerId, l: LandId) -> Res {
    let t = choose_type(g, p, "Secluded Courtyard");
    g.land_mut(l).data.set(DataKey::Ctype, Val::Str(t));
    crate::glog!(g, "    Secluded Courtyard names {}", title(t));
    Ok(())
}

/// galadriel.shapeshifter (Maskwood Nexus): a 2/2 blue Shapeshifter with changeling (every creature type)
fn shapeshifter(g: &mut Game, p: PlayerId) -> Res {
    let spec = Tokens {
        tgh: Some(2),
        color: Some(Colors::from_letters("U")),
        types: vec!["shapeshifter"],
        ..Tokens::new(1, 2)
    };
    for x in make_tokens(g, p, spec)? {
        let xm = g.perm_mut(x);
        xm.data.set(DataKey::Kws, Val::List(vec![Val::Str("changeling")]));
        for t in ALL_TYPES.iter().copied().chain(["rebel", "shapeshifter"]) {
            if !xm.ttypes.contains(&t) {
                xm.ttypes.push(t);
            }
        }
    }
    Ok(())
}

/// Maskwood Nexus: your creatures, creature spells and creature cards are every creature type; {3}, {T}: a 2/2 blue
/// Shapeshifter with changeling, with spare mana at an opponent's end step
fn maskwood_ai(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if p != x.owner || x.tapped || x.phased || post.is_some() || g.active == Some(p) || !can_pay(g, p, 3, "", false) {
        return Ok(vec![]);
    }
    Ok(vec![ability(1.6 + fetch_bonus(g, p), "Maskwood Nexus: a Shapeshifter".into(), src, maskwood_go, 0)])
}

fn maskwood_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if g.perm(src).tapped || !can_pay(g, p, 3, "", false) {
        return Ok(false);
    }
    pay(g, p, 3, "", false)?;
    g.perm_mut(src).tapped = true;
    crate::glog!(g, "  {} activates Maskwood Nexus", pname(g, p));
    if ability_window(g, p, Some(src), "a 2/2 Shapeshifter", None, None)? {
        shapeshifter(g, p)?;
    }
    Ok(true)
}

// ================================================================== Galadriel, Light of Valinor
const MODES: [Sym; 3] = ["mana", "counters", "draw"];

fn mode_text(m: &str) -> &'static str {
    match m {
        "mana" => "add {G}{G}{G}",
        "counters" => "a +1/+1 counter on each creature you control",
        _ => "scry 2, then draw a card",
    }
}

/// galadriel._chosen: the modes chosen this turn
pub fn chosen(g: &Game, src: PermId) -> Vec<Sym> {
    match g.perm(src).data.get(DataKey::Alliance) {
        Some(Val::List(v)) if v.first() == Some(&Val::Stamp(g.turn_stamp())) => {
            v[1..].iter().filter_map(|x| if let Val::Str(s) = x { Some(*s) } else { None }).collect()
        }
        _ => vec![],
    }
}

/// galadriel.alliance_value: what each mode is worth to p now
pub fn alliance_value(g: &Game, p: PlayerId, mode: &str) -> f64 {
    let cre = g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).count();
    if mode == "counters" {
        let early = g.active == Some(p) && matches!(g.step, Step::Start | Step::Main1);
        return 0.9 * cre as f64 + if early { 0.8 } else { 0.0 };
    }
    if mode == "draw" {
        return 2.6;
    }
    if g.active != Some(p) || !in_main(g) {
        return 0.2;
    }
    let have = total_mana(g, p, false);
    let want = g.player(p).hand.iter().any(|&c| {
        let d = g.db.get(c);
        // 'G' not in c.pips.replace('G', '', 3): at most three green pips
        !d.land && d.cmc > have && d.cmc <= have + 3 && d.pips.chars().filter(|&x| x == 'G').count() <= 3
    });
    if want { 3.2 } else { 0.6 }
}

/// Alliance: whenever another creature you control enters, choose one that hasn't been chosen this turn: {G}{G}{G},
/// a +1/+1 counter on each creature you control, or scry 2 and draw (the AI weighs them). HUMAN(phase 9): a person
/// chooses the mode (and the mana goes to their pool).
fn alliance(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src || g.perm(m).owner != o || !g.is_creature(m) || !controls(g, o, src) || g.perm(src).phased {
        return Ok(());
    }
    if chosen(g, src).len() >= MODES.len() {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "Alliance", None)? {
        return Ok(());
    }
    let done = chosen(g, src);
    let left: Vec<Sym> = MODES.iter().copied().filter(|k| !done.contains(k)).collect();
    if left.is_empty() {
        return Ok(());
    }
    let mode = first_max(&left, |x| alliance_value(g, o, x)).unwrap();
    let mut v = vec![Val::Stamp(g.turn_stamp())];
    v.extend(done.iter().map(|&x| Val::Str(x)));
    v.push(Val::Str(mode));
    g.perm_mut(src).data.set(DataKey::Alliance, Val::List(v));
    let turns = g.player(o).turns;
    g.player_mut(o).milestone.entry("alliance").or_insert(turns);
    g.player_mut(o).stat(intern(&format!("alliance_{mode}")), 1);
    crate::glog!(g, "    Galadriel (Alliance): {}", mode_text(mode));
    match mode {
        "mana" => g.player_mut(o).floating.g += 3,
        "counters" => {
            for x in g.player(o).perms.clone() {
                if g.is_creature(x) && !g.perm(x).phased {
                    crate::cardcode::add_counters(g, x, 1);
                }
            }
        }
        _ => {
            crate::cardcode::scry(g, o, 2, false)?;
            draw(g, o, 1, false)?;
        }
    }
    Ok(())
}

// ================================================================== Panharmonicon
/// a triggered ability of your permanents that an artifact or creature entering sets off triggers an additional
/// time (card code, the ability language, the engine's enters tags, Cathars' Crusade)
fn panharmonicon(g: &Game, src: Src, p: PlayerId, kind: Sym, x: Option<PermId>) -> i32 {
    let s = g.perm(src);
    if kind != "etb" || p != s.owner || s.phased {
        return 0;
    }
    let Some(m) = x else { return 0 };
    let art = g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::ARTIFACT));
    (g.is_creature(m) || art) as i32
}

// ================================================================== the Rebel searchers
/// galadriel.SEARCHERS: (activation cost, mana value cap)
const SEARCHERS: [(&str, u32, u32); 5] = [
    ("Ramosian Sergeant", 3, 2),
    ("Ramosian Lieutenant", 4, 3),
    ("Ramosian Captain", 5, 4),
    ("Defiant Vanguard", 5, 4),
    ("Ramosian Commander", 6, 5),
];
const LIN: &str = "Lin Sivvi, Defiant Hero";

fn searcher(name: &str) -> Option<(u32, u32)> {
    SEARCHERS.iter().find(|x| x.0 == name).map(|x| (x.1, x.2))
}

/// galadriel.rebel_cards: Rebel permanent cards with mana value cap or less
fn rebel_cards(g: &Game, p: PlayerId, cards: &[CardId], cap: u32) -> Vec<CardId> {
    cards
        .iter()
        .copied()
        .filter(|&c| {
            let d = g.db.get(c);
            d.perm && !d.land && d.cmc <= cap && card_is(g, p, c, "rebel")
        })
        .collect()
}

/// galadriel.rebel_value: how much the AI wants Rebel c on the battlefield now
pub fn rebel_value(g: &Game, p: PlayerId, c: CardId) -> f64 {
    let d = g.db.get(c);
    let n = &*d.name;
    let pl = g.player(p);
    let on_bf = |name: &str| pl.perms.iter().any(|&m| card_name(g, m) == Some(name));
    let searchers =
        pl.perms.iter().filter(|&&m| card_name(g, m).is_some_and(|x| searcher(x).is_some() || x == LIN)).count() as f64;
    let opp_col = |ch: char| {
        g.opps(p)
            .flat_map(|q| g.player(q).perms.iter().copied())
            .any(|m| g.is_creature(m) && pval(g, m) >= 3.0 && colors_of(g, m).has(ch))
    };
    if on_bf(n) && d.tag(Tag::Leg) {
        return 0.0;
    }
    if n == LIN {
        return 9.0;
    }
    if searcher(n).is_some() {
        let base = match n {
            "Ramosian Commander" => 7.0,
            "Ramosian Captain" => 6.0,
            "Defiant Vanguard" => 5.5,
            "Ramosian Lieutenant" => 5.0,
            _ => 3.5,
        };
        return base - 1.2 * (searchers - 1.0).max(0.0) - if on_bf(n) { 1.5 } else { 0.0 };
    }
    match n {
        "Lawbringer" => return if opp_col('R') { 6.5 } else { 1.5 },
        "Lightbringer" => return if opp_col('B') { 6.5 } else { 1.5 },
        "Nightwind Glider" => return 3.5 + if opp_col('B') { 1.0 } else { 0.0 },
        "Thermal Glider" => return 3.5 + if opp_col('R') { 1.0 } else { 0.0 },
        "Ramosian Revivalist" => {
            return 3.5 + if rebel_cards(g, p, &pl.gy, 5).is_empty() { 0.0 } else { 2.0 };
        }
        "Jhovall Queen" => 5.0,
        "Ballista Squad" | "Cho-Manno, Revolutionary" | "Mirror Entity" => 4.5,
        "Whipcorder" => 4.0,
        "Knight of the Holy Nimbus" => 3.5,
        "Amrou Seekers" | "Errant Doomsayers" => 3.0,
        _ => card_worth(g, p, c, false) / 12.0,
    }
}

const ENTER_PAYOFFS: [&str; 11] = [
    "Galadriel, Light of Valinor",
    "Cathars' Crusade",
    "Welcoming Vampire",
    "Tocasia's Welcome",
    "Mentor of the Meek",
    "Kindred Discovery",
    "Panharmonicon",
    "Vanquisher's Banner",
    "Patchwork Banner",
    "Door of Destinies",
    "Elvish Archdruid",
];

/// galadriel.fetch_bonus: what a creature entering is worth beyond itself here: each payoff on the battlefield
pub fn fetch_bonus(g: &Game, p: PlayerId) -> f64 {
    0.6 * g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| !g.perm(m).phased && card_name(g, m).is_some_and(|n| ENTER_PAYOFFS.contains(&n)))
        .count() as f64
}

/// galadriel.fetch_u: the AI's utility for putting a card worth v onto the battlefield for free: like casting it (a
/// card more), plus the enters payoffs
fn fetch_u(g: &Game, p: PlayerId, v: f64) -> f64 {
    3.5 + 0.6 * v + fetch_bonus(g, p)
}

/// galadriel.best_rebel: the Rebel the AI would take (from the library, or from `zone`)
fn best_rebel(g: &Game, p: PlayerId, cap: u32, zone: Option<&[CardId]>) -> Option<CardId> {
    let pool = match zone {
        Some(z) => z.to_vec(),
        None => searchable(g, p),
    };
    let cs = rebel_cards(g, p, &pool, cap);
    first_max(&cs, |c| (rebel_value(g, p, c), g.db.get(c).cmc))
}

/// galadriel.put_rebel: search (or, from the graveyard, return) a Rebel permanent card with mana value cap or less onto
/// the battlefield: the AI takes the most useful one. HUMAN(phase 9): a person picks.
fn put_rebel(g: &mut Game, p: PlayerId, src: &str, cap: u32, from_gy: bool) -> Res<Option<PermId>> {
    let pool = if from_gy { g.player(p).gy.clone() } else { searchable(g, p) };
    let cs = rebel_cards(g, p, &pool, cap);
    let c = first_max(&cs, |c| (rebel_value(g, p, c), g.db.get(c).cmc));
    let Some(c) = c else {
        if !from_gy {
            shuffle_library(g, p);
        }
        return Ok(None);
    };
    let pl = g.player_mut(p);
    remove_first(if from_gy { &mut pl.gy } else { &mut pl.library }, c);
    if !from_gy {
        shuffle_library(g, p);
    }
    crate::glog!(g, "    {} puts {} onto the battlefield", src, g.db.get(c).name);
    g.player_mut(p).stat("rebels_fetched", 1);
    Ok(Some(enter(g, p, c, Enter::default())?))
}

/// galadriel.ability_locked: an Aura stops src's activated abilities, or its abilities are off
fn ability_locked(g: &Game, m: PermId) -> bool {
    (!g.auras.is_empty() && crate::cardcode::locked(g, m, "noact")) || g.perm(m).neutered
}

/// galadriel._searcher_ok: src can use its {T} ability now
fn searcher_ok(g: &Game, p: PlayerId, src: PermId) -> bool {
    let x = g.perm(src);
    p == x.owner && x.on_bf && !x.phased && !x.tapped && !x.sick && !ability_locked(g, src)
}

/// galadriel._when_ok: the AI's timing: your main phases, or an opponent's end step (post None)
fn when_ok(g: &Game, p: PlayerId, post: Option<bool>) -> bool {
    !(post.is_none() && g.active == Some(p))
}

/// {cost}, {T}: search for a Rebel permanent card with mana value cap or less and put it onto the battlefield (the AI
/// at an opponent's end step or in its main phases): Ramosian Sergeant, Lieutenant, Captain, Commander, Defiant
/// Vanguard
fn searcher_opts(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let name = name_of(g, src);
    let Some((cost, cap)) = searcher(name) else { return Ok(vec![]) };
    if !searcher_ok(g, p, src) || !when_ok(g, p, post) || !can_pay(g, p, cost, "", false) {
        return Ok(vec![]);
    }
    let Some(c) = best_rebel(g, p, cap, None) else { return Ok(vec![]) };
    let v = rebel_value(g, p, c);
    Ok(vec![ability(fetch_u(g, p, v), format!("{name}: fetch {}", g.db.get(c).name), src, searcher_go, 0)])
}

fn searcher_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let name = name_of(g, src);
    let Some((cost, cap)) = searcher(name) else { return Ok(false) };
    if !searcher_ok(g, p, src) || !can_pay(g, p, cost, "", false) {
        return Ok(false);
    }
    pay(g, p, cost, "", false)?;
    g.perm_mut(src).tapped = true;
    crate::glog!(g, "  {} activates {}", pname(g, p), name);
    if ability_window(g, p, Some(src), &format!("search for a Rebel (mana value {cap} or less)"), None, None)? {
        put_rebel(g, p, name, cap, false)?;
    }
    Ok(true)
}

/// Lin Sivvi, Defiant Hero: {X}, {T}: a Rebel permanent card with mana value X or less onto the battlefield; {3}: a
/// Rebel card from your graveyard to the bottom of your library
fn lin(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let mut out = vec![];
    if searcher_ok(g, p, src) && when_ok(g, p, post) {
        let have = total_mana(g, p, false);
        if let Some(c) = best_rebel(g, p, have, None) {
            let x = g.db.get(c).cmc;
            let v = rebel_value(g, p, c);
            out.push(ability(
                fetch_u(g, p, v),
                format!("Lin Sivvi (X = {x}): fetch {}", g.db.get(c).name),
                src,
                lin_go,
                x as i64,
            ));
        }
    }
    if p == g.perm(src).owner && post == Some(true) && !ability_locked(g, src) && can_pay(g, p, 3, "", false) {
        let gy = rebel_cards(g, p, &g.player(p).gy, 99);
        if !gy.is_empty() && rebel_cards(g, p, &g.player(p).library, 99).is_empty() {
            let c2 = max_by(&gy, |c| rebel_value(g, p, c)).unwrap();
            out.push(ability(
                0.4,
                format!("Lin Sivvi: {} back to the library", g.db.get(c2).name),
                src,
                lin_back,
                c2.0 as i64,
            ));
        }
    }
    Ok(out)
}

fn lin_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let x = arg as u32;
    if !searcher_ok(g, p, src) || !can_pay(g, p, x, "", false) {
        return Ok(false);
    }
    pay(g, p, x, "", false)?;
    g.perm_mut(src).tapped = true;
    crate::glog!(g, "  {} activates Lin Sivvi, X = {x}", pname(g, p));
    if ability_window(g, p, Some(src), &format!("search for a Rebel (mana value {x} or less)"), None, None)? {
        put_rebel(g, p, "Lin Sivvi", x, false)?;
    }
    Ok(true)
}

fn lin_back(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let c2 = CardId(arg as u16);
    if !g.player(p).gy.contains(&c2) || !can_pay(g, p, 3, "", false) {
        return Ok(false);
    }
    pay(g, p, 3, "", false)?;
    let n = g.db.get(c2).name.clone();
    crate::glog!(g, "  {} activates Lin Sivvi: {} to the bottom of the library", pname(g, p), n);
    if ability_window(g, p, Some(src), &format!("put {n} on the bottom of the library"), None, None)?
        && g.player(p).gy.contains(&c2)
    {
        remove_first(&mut g.player_mut(p).gy, c2);
        g.player_mut(p).library.insert(0, c2);
    }
    Ok(true)
}

/// Ramosian Revivalist: {6}, {T}: return a Rebel permanent card with mana value 5 or less from your graveyard to the
/// battlefield
fn revivalist(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !searcher_ok(g, p, src) || !when_ok(g, p, post) || !can_pay(g, p, 6, "", false) {
        return Ok(vec![]);
    }
    let gy = g.player(p).gy.clone();
    let Some(c) = best_rebel(g, p, 5, Some(&gy)) else { return Ok(vec![]) };
    let u = fetch_u(g, p, rebel_value(g, p, c));
    Ok(vec![ability(u, format!("Ramosian Revivalist: return {}", g.db.get(c).name), src, revivalist_go, 0)])
}

fn revivalist_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !searcher_ok(g, p, src) || !can_pay(g, p, 6, "", false) {
        return Ok(false);
    }
    pay(g, p, 6, "", false)?;
    g.perm_mut(src).tapped = true;
    crate::glog!(g, "  {} activates Ramosian Revivalist", pname(g, p));
    if ability_window(g, p, Some(src), "return a Rebel from your graveyard", None, None)? {
        put_rebel(g, p, "Ramosian Revivalist", 5, true)?;
    }
    Ok(true)
}

// ================================================================== creatures with abilities
/// galadriel._opp_creatures
fn opp_creatures(g: &Game, p: PlayerId, pred: impl Fn(PermId) -> bool) -> Vec<PermId> {
    g.opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && !g.perm(m).phased && !untargetable(g, m) && pred(m))
        .collect()
}

/// Lawbringer / Lightbringer: {T}, sacrifice it: exile target red / black creature (the AI on a threat, or an
/// attacker)
fn bringer_opts(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !searcher_ok(g, p, src) || post == Some(false) {
        return Ok(vec![]);
    }
    let colour = if name_of(g, src) == "Lawbringer" { 'R' } else { 'B' };
    let w = Colors::from_letters("W");
    let tg = opp_creatures(g, p, |m| colors_of(g, m).has(colour) && !protected_from(g, m, w));
    let Some(t) = max_by(&tg, |m| pval(g, m)) else { return Ok(vec![]) };
    if pval(g, t) < 3.0 {
        return Ok(vec![]);
    }
    let label = format!("{}: exile {}", name_of(g, src), name_of(g, t));
    Ok(vec![ability(0.8 + 0.5 * pval(g, t), label, src, bringer_go, t.0 as i64)])
}

fn bringer_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    bringer_use(g, p, src, PermId(arg as u32))
}

/// galadriel.bringer_use
pub fn bringer_use(g: &mut Game, p: PlayerId, src: PermId, t: PermId) -> Res<bool> {
    if !controls(g, p, src) || g.perm(src).tapped || !on_bf(g, t) {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    let sname = name_of(g, src);
    crate::glog!(g, "  {} sacrifices {}: exile {}", pname(g, p), sname, name_of(g, t));
    die(g, src, "sac")?;
    let cd = g.perm(src).cd.unwrap();
    if ability_window_card(g, p, cd, &format!("exile {}", name_of(g, t)), Some(6.0), Some(t))?
        && on_bf(g, t)
        && !untargetable(g, t)
    {
        apply_removal(g, Some(p), t, "exile", None)?;
    }
    Ok(true)
}

/// galadriel.tapper_target: an opposing creature this tapper can tap that would block your best attacker
fn tapper_target(g: &Game, p: PlayerId, src: PermId) -> Option<PermId> {
    let tough = if name_of(g, src) == "Errant Doomsayers" { 2 } else { 99 };
    guildmage_target(g, p).filter(|&t| etgh(g, t) <= tough)
}

/// galadriel.TAPPERS: (generic, pips)
fn tapper_cost(name: &str) -> Option<(u32, &'static str)> {
    match name {
        "Errant Doomsayers" => Some((0, "")),
        "Whipcorder" => Some((0, "W")),
        _ => None,
    }
}

/// Errant Doomsayers ({T}: tap target creature with toughness 2 or less) and Whipcorder ({W}, {T}: tap target
/// creature), before your combat: the creature that would block your attacker. Whipcorder's morph is practice mode.
fn tapper_opts(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let Some((gen_, pips)) = tapper_cost(name_of(g, src)) else { return Ok(vec![]) };
    if !searcher_ok(g, p, src) || post != Some(false) || g.active != Some(p) || !can_pay(g, p, gen_, pips, false) {
        return Ok(vec![]);
    }
    let Some(t) = tapper_target(g, p, src) else { return Ok(vec![]) };
    let label = format!("{}: tap {}", name_of(g, src), name_of(g, t));
    Ok(vec![ability(1.0 + 0.5 * pval(g, t), label, src, tapper_go, t.0 as i64)])
}

fn tapper_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let Some((gen_, pips)) = tapper_cost(name_of(g, src)) else { return Ok(false) };
    tapper_use(g, p, src, PermId(arg as u32), gen_, pips)
}

/// galadriel.tapper_use
fn tapper_use(g: &mut Game, p: PlayerId, src: PermId, t: PermId, gen_: u32, pips: &str) -> Res<bool> {
    if !searcher_ok(g, p, src) || !on_bf(g, t) || g.perm(t).tapped || !can_pay(g, p, gen_, pips, false) {
        return Ok(false);
    }
    pay(g, p, gen_, pips, false)?;
    g.perm_mut(src).tapped = true;
    crate::glog!(g, "  {} activates {}: tap {}", pname(g, p), name_of(g, src), name_of(g, t));
    if ability_window(g, p, Some(src), &format!("tap {}", name_of(g, t)), None, Some(t))? && on_bf(g, t) {
        g.perm_mut(t).tapped = true;
    }
    Ok(true)
}

/// galadriel.precombat (CI.galadriel_precombat): q (the AI) at the beginning of another player's combat: tap their
/// best attacker with a tapper
pub fn precombat(g: &mut Game, q: PlayerId) -> Res {
    let Some(a) = g.active else { return Ok(()) };
    let tappers: Vec<PermId> = g
        .player(q)
        .perms
        .iter()
        .copied()
        .filter(|&m| card_name(g, m).is_some_and(|n| tapper_cost(n).is_some()))
        .collect();
    for src in tappers {
        let name = name_of(g, src);
        let (gen_, pips) = tapper_cost(name).unwrap();
        if !searcher_ok(g, q, src) || !can_pay(g, q, gen_, pips, false) {
            continue;
        }
        let tough = if name == "Errant Doomsayers" { 2 } else { 99 };
        let cands: Vec<PermId> = g
            .player(a)
            .perms
            .iter()
            .copied()
            .filter(|&m| {
                let x = g.perm(m);
                g.is_creature(m)
                    && !x.tapped
                    && !x.phased
                    && !untargetable(g, m)
                    && etgh(g, m) <= tough
                    && epow(g, m) >= 3
                    && (!x.sick || has_haste(g, m))
            })
            .collect();
        let Some(t) = max_by(&cands, |m| epow(g, m) as f64) else { continue };
        tapper_use(g, q, src, t, gen_, pips)?;
    }
    Ok(())
}

/// One of the answers galadriel.attack_answers lists (Python's closures)
#[derive(Debug, Clone, Copy)]
pub enum Answer {
    /// Ballista Squad: X damage to an attacker
    Ballista { src: PermId, a: PermId, x: u32 },
    /// Lawbringer / Lightbringer: exile an attacker
    Bringer { src: PermId, a: PermId },
}

/// galadriel.attack_answers (CI.galadriel_attack_answers): d (the AI) is attacked by p: its permanents' answers
/// [(value, answer)]: Ballista Squad, Lawbringer, Lightbringer
pub fn attack_answers(g: &Game, d: PlayerId, p: PlayerId, atk: &[PermId]) -> Vec<(f64, Answer)> {
    let mut out = vec![];
    let w = Colors::from_letters("W");
    for &src in &g.player(d).perms {
        if g.perm(src).cd.is_none() || !searcher_ok(g, d, src) {
            continue;
        }
        let n = name_of(g, src);
        if n == "Ballista Squad" {
            let have = total_mana(g, d, false) as i32 - 1;
            for &a in atk {
                if !controls(g, p, a) || untargetable(g, a) || protected_from(g, a, w) || no_damage(g, a) {
                    continue;
                }
                let x = etgh(g, a);
                if x > have || indestructible(g, a) {
                    continue;
                }
                let v = pval(g, a) + 0.3 * epow(g, a) as f64 - 0.4 * x as f64;
                out.push((v, Answer::Ballista { src, a, x: x.max(0) as u32 }));
            }
        } else if n == "Lawbringer" || n == "Lightbringer" {
            let col = if n == "Lawbringer" { 'R' } else { 'B' };
            for &a in atk {
                if controls(g, p, a) && colors_of(g, a).has(col) && !untargetable(g, a) && !protected_from(g, a, w) {
                    out.push((pval(g, a) + 0.3 * epow(g, a) as f64 - 1.0, Answer::Bringer { src, a }));
                }
            }
        }
    }
    out
}

/// make an answer from attack_answers
pub fn answer(g: &mut Game, d: PlayerId, a: Answer) -> Res<bool> {
    match a {
        Answer::Ballista { src, a, x } => ballista_use(g, d, src, a, x),
        Answer::Bringer { src, a } => bringer_use(g, d, src, a),
    }
}

/// Ballista Squad: {X}{W}, {T}: X damage to target attacking or blocking creature (the AI kills an attacker)
fn ballista_use(g: &mut Game, p: PlayerId, src: PermId, t: PermId, x: u32) -> Res<bool> {
    if !searcher_ok(g, p, src) || !on_bf(g, t) || !can_pay(g, p, x, "W", false) {
        return Ok(false);
    }
    pay(g, p, x, "W", false)?;
    g.perm_mut(src).tapped = true;
    crate::glog!(g, "  {} activates Ballista Squad: {x} damage to {}", pname(g, p), name_of(g, t));
    if ability_window(g, p, Some(src), &format!("{x} damage to {}", name_of(g, t)), Some(5.0), Some(t))?
        && on_bf(g, t)
        && !no_damage(g, t)
        && etgh(g, t) <= x as i32
    {
        apply_removal(g, Some(p), t, &format!("dmg{x}"), None)?;
    }
    Ok(true)
}

/// galadriel.vanguard_blocks (CI.vanguard_blocks): after combat damage, each Defiant Vanguard that blocked destroys
/// itself and the attacker
pub fn vanguard_blocks(g: &mut Game, assign: &[(PermId, PermId)]) -> Res {
    for &(a, b) in assign {
        if card_name(g, b) != Some("Defiant Vanguard") {
            continue;
        }
        let bo = g.perm(b).owner;
        if !trigger_window(g, bo, Some(b), &format!("destroy it and {}", name_of(g, a)), Some(5.0))? {
            continue;
        }
        if on_bf(g, a) {
            die(g, a, "destroy")?;
        }
        if on_bf(g, b) {
            die(g, b, "destroy")?;
        }
    }
    Ok(())
}

/// Knight of the Holy Nimbus (CI.SELF_REGEN): flanking; if it would be destroyed it regenerates, unless an opponent
/// paid {2} this turn (the AI pays when it can). HUMAN(phase 9): a person decides whether to pay.
fn nimbus_regen(g: &mut Game, m: PermId) -> Res<bool> {
    let p = g.perm(m).owner;
    let st = g.turn_stamp();
    if g.perm(m).data.get(DataKey::Noregen) == Some(&Val::Stamp(st)) {
        return Ok(false);
    }
    let order: Vec<PlayerId> = match g.active {
        Some(a) => std::iter::once(a).chain(g.after(a)).collect(),
        None => g.opps(p).collect(),
    };
    for q in order {
        if q == p || !g.player(q).alive {
            continue;
        }
        if can_pay(g, q, 2, "", false) && pval(g, m) >= 1.5 {
            pay(g, q, 2, "", false)?;
            crate::glog!(g, "    {} pays {{2}}: Knight of the Holy Nimbus can't regenerate", pname(g, q));
            g.perm_mut(m).data.set(DataKey::Noregen, Val::Stamp(st));
            return Ok(false);
        }
    }
    g.perm_mut(m).tapped = true;
    add_kw(g, m, "regenerated");
    crate::glog!(g, "    Knight of the Holy Nimbus regenerates");
    Ok(true)
}

// ================================================================== Mangara, Tocasia's Welcome, Voice of Resurgence
/// Mangara, the Diplomat: draws when an opponent attacks you with two or more creatures ...
fn mangara_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    let o = g.perm(src).owner;
    if p == o || d != o || atk.len() < 2 {
        return Ok(vec![]);
    }
    if trigger_window(g, o, Some(src), "draw a card", None)? {
        draw(g, o, 1, false)?;
    }
    Ok(vec![])
}

/// ... and when an opponent casts their second spell each turn
fn mangara_cast(g: &mut Game, src: Src, caster: PlayerId, _c: CardId) -> Res {
    let o = g.perm(src).owner;
    if caster == o || g.player(caster).spells_this_turn != 2 {
        return Ok(());
    }
    if trigger_window(g, o, Some(src), "draw a card", None)? {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

/// Tocasia's Welcome: once each turn, when one or more creatures with mana value 3 or less enter under your control,
/// draw a card
fn tocasia(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    let Some(cd) = g.perm(m).cd else { return Ok(()) };
    if g.perm(m).owner != o || !g.is_creature(m) || g.db.get(cd).cmc > 3 {
        return Ok(());
    }
    let k = intern(&format!("tocasia{}", src.0));
    if g.player(o).flag_turn.get(k) == Some(&g.turn_stamp()) {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "draw a card", None)? {
        return Ok(());
    }
    if once_per_turn(g, o, k) {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

/// galadriel._voice_token: an Elemental whose size is the number of creatures you control
fn voice_token(g: &mut Game, p: PlayerId) -> Res {
    g.selfpt = true;
    let spec =
        Tokens { tgh: Some(1), color: Some(Colors::from_letters("GW")), types: vec!["elemental"], ..Tokens::new(1, 1) };
    for x in make_tokens(g, p, spec)? {
        g.perm_mut(x).data.set(DataKey::Voice, Val::Bool(true));
    }
    crate::glog!(g, "    {} creates an Elemental (its size is the number of creatures they control)", pname(g, p));
    Ok(())
}

/// galadriel.voice_pt (common.TOKEN_PT): the Elemental's power and toughness are the number of creatures you control
fn voice_pt(g: &Game, m: PermId) -> (i32, i32) {
    if !g.perm(m).data.truthy(DataKey::Voice) {
        return (0, 0);
    }
    let o = g.perm(m).owner;
    let n = g.player(o).perms.iter().filter(|&&x| g.is_creature(x) && !g.perm(x).phased).count() as i32;
    (n - 1, n - 1)
}

/// Voice of Resurgence: an Elemental token when an opponent casts a spell during your turn ...
fn voice_cast(g: &mut Game, src: Src, caster: PlayerId, _c: CardId) -> Res {
    let o = g.perm(src).owner;
    if caster == o || g.active != Some(o) {
        return Ok(());
    }
    if trigger_window(g, o, Some(src), "an Elemental token", None)? {
        voice_token(g, o)?;
    }
    Ok(())
}

/// ... and when it dies
fn voice_dies(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    let o = g.perm(m).owner;
    if trigger_window(g, o, Some(m), "an Elemental token", None)? {
        voice_token(g, o)?;
    }
    Ok(())
}

// ================================================================== Crackdown
/// galadriel.crackdown_on (CI.crackdown_on): Crackdown is on the battlefield
pub fn crackdown_on(g: &Game) -> bool {
    g.players
        .iter()
        .filter(|q| q.alive)
        .any(|q| q.perms.iter().any(|&m| !g.perm(m).phased && card_name(g, m) == Some("Crackdown")))
}

/// galadriel.crackdown_holds (CI.crackdown_holds): a nonwhite creature with power 3 or greater doesn't untap
pub fn crackdown_holds(g: &Game, m: PermId) -> bool {
    g.is_creature(m) && !colors_of(g, m).has('W') && epow(g, m) >= 3
}

// ================================================================== Abduction
/// Abduction: enchant creature: you control it; it untaps as the Aura enters; when it dies, it returns to its owner.
/// HUMAN(phase 9): a person picks the creature.
fn abduction(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let t = best_steal(g, p, g.perm(src).cd);
    let Some(t) = t else {
        leave(g, src)?;
        to_zone_card(g, src, Zone::Gy);
        return Ok(());
    };
    if g.perm(t).owner == p {
        g.perm_mut(src).attached = Some(t);
        return abduction_untap(g, p, src, t);
    }
    let cd = g.perm(src).cd;
    let to = g.perm(t).owner;
    if protect_response(g, to, t, "steal", Some(p), cd)? || !on_bf(g, t) {
        leave(g, src)?;
        to_zone_card(g, src, Zone::Gy);
        return Ok(());
    }
    steal(g, p, t, false);
    g.perm_mut(src).attached = Some(t);
    abduction_untap(g, p, src, t)
}

/// galadriel._abduction_untap
fn abduction_untap(g: &mut Game, p: PlayerId, src: PermId, t: PermId) -> Res {
    g.perm_mut(src).data.set(DataKey::Abducted, Val::Perm(t)); // remembered: the creature is detached as it dies
    if trigger_window(g, p, Some(src), &format!("untap {}", name_of(g, t)), None)? && controls(g, p, t) {
        g.perm_mut(t).tapped = false;
    }
    Ok(())
}

/// the enchanted creature died: that card returns to the battlefield under its owner's control
fn abduction_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if g.perm(src).data.get(DataKey::Abducted) != Some(&Val::Perm(m)) {
        return Ok(());
    }
    let owner = g.perm(m).orig;
    let Some(cd) = g.perm(m).cd else { return Ok(()) };
    if g.perm(m).token || !g.player(owner).alive {
        return Ok(());
    }
    let so = g.perm(src).owner;
    let text = format!("{} returns to {}", name_of(g, m), pname(g, owner));
    if !trigger_window(g, so, Some(src), &text, None)? {
        return Ok(());
    }
    if remove_first(&mut g.player_mut(owner).gy, cd) {
        enter(g, owner, cd, Enter::default())?;
        crate::glog!(
            g,
            "    {} returns to the battlefield under {}'s control (Abduction)",
            name_of(g, m),
            pname(g, owner)
        );
    }
    Ok(())
}

/// Abduction leaves: control of the creature returns
fn abduction_leaves(g: &mut Game, src: Src, _m: PermId) -> Res {
    let so = g.perm(src).owner;
    let Some(h) = g.perm(src).attached else { return Ok(()) };
    let orig = g.perm(h).orig;
    if controls(g, so, h) && orig != so && g.player(orig).alive {
        g.player_mut(so).perms.retain(|&x| x != h);
        g.perm_mut(h).owner = orig;
        g.player_mut(orig).perms.push(h);
        g.bf_ver += 1;
    }
    Ok(())
}

/// Abduction falls off when its creature is gone
fn abduction_sba(g: &mut Game, src: Src) -> Res {
    if !on_bf(g, src) {
        return Ok(());
    }
    let gone = g.perm(src).attached.is_none_or(|a| !on_bf(g, a));
    if gone {
        g.perm_mut(src).attached = None;
        leave(g, src)?;
        to_zone_card(g, src, Zone::Gy);
    }
    Ok(())
}

// ================================================================== spells
/// Bribery: the best creature in an opponent's library onto the battlefield under your control. HUMAN(phase 9): a
/// person chooses the opponent and the card.
fn bribery(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    if g.opps(p).next().is_none() {
        return Ok("gy");
    }
    let (q, pick) = bribery_pick(g, p);
    let Some(q) = q else { return Ok("gy") };
    if let Some(x) = pick {
        remove_first(&mut g.player_mut(q).library, x);
        crate::glog!(g, "    Bribery: {} takes {} from {}'s library", pname(g, p), g.db.get(x).name, pname(g, q));
        enter(g, p, x, Enter { orig: Some(q), ..Enter::default() })?;
    }
    shuffle_library(g, q);
    Ok("gy")
}

/// galadriel.bribery_pick: (the opponent, the creature card): bomb rating twice, mana value, and 3 for a flier
pub fn bribery_pick(g: &Game, p: PlayerId) -> (Option<PlayerId>, Option<CardId>) {
    let mut best: (Option<PlayerId>, Option<CardId>, f64) = (None, None, -1.0);
    for q in g.opps(p) {
        for &x in &g.player(q).library {
            let d = g.db.get(x);
            if d.creature {
                let v = (d.bomb * 2) as f64 + d.cmc as f64 + if d.tag(Tag::Fly) { 3.0 } else { 0.0 };
                if v > best.2 {
                    best = (Some(q), Some(x), v);
                }
            }
        }
    }
    (best.0.or_else(|| g.opps(p).next()), best.1)
}

/// galadriel.eerie_return: the cards come back at the beginning of the next end step
fn eerie_return(g: &mut Game, p: PlayerId, cards: &[CardId]) {
    g.eot_returns.extend(cards.iter().map(|&c| (p, c)));
}

/// galadriel.eot_returns (CI.eot_returns): the beginning of the end step: creatures exiled by Eerie Interlude return
/// under their owner's control
pub fn eot_returns(g: &mut Game) -> Res {
    if g.eot_returns.is_empty() {
        return Ok(());
    }
    let rs = std::mem::take(&mut g.eot_returns);
    for (p, cd) in rs {
        if !g.player(p).alive || !g.player(p).exile.contains(&cd) {
            continue;
        }
        remove_first(&mut g.player_mut(p).exile, cd);
        let n = enter(g, p, cd, Enter::default())?;
        g.perm_mut(n).is_cmd = cd == g.player(p).cmd;
        crate::glog!(g, "    {} returns to the battlefield (Eerie Interlude)", g.db.get(cd).name);
    }
    Ok(())
}

/// galadriel.interlude: exile your creatures ms; they return at the beginning of the next end step (tokens are gone
/// for good)
pub fn interlude(g: &mut Game, p: PlayerId, ms: &[PermId]) -> Res {
    let mut back = vec![];
    for &m in ms {
        if !controls(g, p, m) {
            continue;
        }
        let (cd, tok) = (g.perm(m).cd, g.perm(m).token);
        leave(g, m)?;
        if !tok && let Some(cd) = cd {
            g.player_mut(p).exile.push(cd);
            back.push(cd);
        }
    }
    eerie_return(g, p, &back);
    let names: Vec<&str> = back.iter().map(|&c| &*g.db.get(c).name).collect();
    crate::glog!(
        g,
        "    Eerie Interlude exiles {} until the end step",
        if names.is_empty() { "nothing".to_string() } else { names.join(", ") }
    );
    Ok(())
}

/// Eerie Interlude: exiles any of your creatures; they return at the beginning of the next end step (the AI saves its
/// board from a wipe or a creature from removal). HUMAN(phase 9): a person's chosen targets (Python's
/// `ctx['targets']`, which only practice mode sets).
fn eerie(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mine: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect();
    let ms: Vec<PermId> = mine.into_iter().filter(|&m| !g.perm(m).token && etb_worth(g, p, m) > 0.0).collect();
    let ms: Vec<PermId> = ms.into_iter().filter(|&m| controls(g, p, m)).collect();
    interlude(g, p, &ms)?;
    Ok("gy")
}

/// galadriel.etb_worth: what blinking creature m gains: its enters effect, and Galadriel's Alliance
pub fn etb_worth(g: &Game, p: PlayerId, m: PermId) -> f64 {
    let mut v = 0.0;
    let Some(cd) = g.perm(m).cd else { return v };
    v += crate::ai::card_etb_value(g, p, cd);
    if named(g, p, "Galadriel, Light of Valinor").is_some() && &*g.db.get(cd).name != "Galadriel, Light of Valinor" {
        v += 1.5;
    }
    v
}

/// Flicker: exiles a nontoken permanent and returns it at once under its owner's control (the AI blinks its best
/// enters-the-battlefield creature). HUMAN(phase 9): a person picks.
fn flicker(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let t = match ctx.target {
        Some(t) => Some(t),
        None => flicker_target(g, p),
    };
    let Some(t) = t.filter(|&t| on_bf(g, t)) else { return Ok("gy") };
    let Some(cd) = g.perm(t).cd else { return Ok("gy") };
    let owner = g.perm(t).orig;
    leave(g, t)?;
    let n = enter(g, owner, cd, Enter::default())?;
    g.perm_mut(n).is_cmd = cd == g.player(owner).cmd;
    crate::glog!(g, "    Flicker: {} leaves and returns", g.db.get(cd).name);
    Ok("gy")
}

/// galadriel.flicker_target
pub fn flicker_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let mine: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            !x.token && g.is_creature(m) && !x.phased && x.cd.is_some()
        })
        .collect();
    max_by(&mine, |m| etb_worth(g, p, m))
}

/// Planar Genesis: top four: a land onto the battlefield tapped, or else a card into your hand; the rest to the bottom
/// in a random order. HUMAN(phase 9): a person picks.
fn genesis(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let n = 4.min(g.player(p).library.len());
    let mut top = vec![];
    for _ in 0..n {
        top.push(g.player_mut(p).library.pop().unwrap());
    }
    let lands: Vec<CardId> = top.iter().copied().filter(|&x| g.db.get(x).land).collect();
    let (mut land, mut take) = (None, None);
    if !lands.is_empty() && g.player(p).lands.len() < 8 {
        land = Some(lands[0]);
    } else {
        take = max_by(&top, |x| card_worth(g, p, x, false));
    }
    // `x is not land`: the card objects are shared, so every copy of the chosen card is left out (as in the Python)
    let mut rest: Vec<CardId> = top.iter().copied().filter(|&x| Some(x) != land && Some(x) != take).collect();
    g.rng.shuffle(&mut rest);
    for x in rest {
        g.player_mut(p).library.insert(0, x);
    }
    if let Some(l) = land {
        // not the land drop: onto the battlefield tapped
        let lid = g.add_land(p, l, true);
        if let Some(f) = g.registry.get(l).filter(|i| i.live()).and_then(|i| i.land_etb) {
            f(g, p, lid)?;
        }
        crate::glog!(g, "    Planar Genesis: {} puts {} onto the battlefield tapped", pname(g, p), g.db.get(l).name);
        landfall(g, p)?;
    }
    if let Some(t) = take {
        g.player_mut(p).hand.push(t);
        g.player_mut(p).seen.insert(t);
        crate::glog!(g, "    Planar Genesis: {} takes a card", pname(g, p));
    }
    Ok("gy")
}

/// Planar Genesis: at an opponent's end step
fn genesis_eot(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || g.active == Some(p) || !g.player(p).hand.contains(&c) || !can_pay(g, p, 0, "GU", false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 2.2,
        label: "Planar Genesis".into(),
        act: Some(Action::Plan { f: genesis_go, arg: c.0 as i64 }),
    }])
}

fn genesis_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 0, "GU", false) {
        return Ok(false);
    }
    pay(g, p, 0, "GU", false)?;
    remove_first(&mut g.player_mut(p).hand, c);
    g.player_mut(p).spells_this_turn += 1;
    cast_card(g, p, c, "hand", Ctx::default())?;
    Ok(true)
}

/// Return to Dust: exile target artifact or enchantment; cast in your main phase, a second one too. HUMAN(phase 9): a
/// person picks the second.
fn dust(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let t = ctx.target;
    if let Some(t) = t
        && on_bf(g, t)
        && !untargetable(g, t)
    {
        apply_removal(g, Some(p), t, "exile", Some(c))?;
    }
    if g.active != Some(p) || !in_main(g) {
        return Ok("gy");
    }
    let mut cands = vec![];
    for q in g.players.iter().filter(|q| q.alive) {
        for &m in &q.perms {
            let x = g.perm(m);
            let Some(cd) = x.cd else { continue };
            let ty = g.db.get(cd).types;
            if Some(m) != t
                && !x.phased
                && (ty.has(Types::ARTIFACT) || ty.has(Types::ENCHANTMENT))
                && !g.is_creature(m)
                && !(x.owner != p && untargetable(g, m))
            {
                cands.push(m);
            }
        }
    }
    let theirs: Vec<PermId> = cands.into_iter().filter(|&m| g.perm(m).owner != p).collect();
    let t2 = max_by(&theirs, |m| pval(g, m)).filter(|&m| pval(g, m) >= 1.0);
    if let Some(t2) = t2 {
        apply_removal(g, Some(p), t2, "exile", Some(c))?;
    }
    Ok("gy")
}

/// Unbreakable Formation: your creatures gain indestructible; cast in your main phase, also a +1/+1 counter and
/// vigilance
fn formation_spell(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let addendum = g.active == Some(p) && in_main(g);
    formation(g, p, addendum);
    Ok("gy")
}

/// galadriel.formation
pub fn formation(g: &mut Game, p: PlayerId, addendum: bool) {
    for m in g.player(p).perms.clone() {
        if g.is_creature(m) && !g.perm(m).phased {
            add_kw(g, m, "indestructible");
            if addendum {
                crate::cardcode::add_counters(g, m, 1);
                add_kw(g, m, "vigilance");
            }
        }
    }
    crate::glog!(
        g,
        "    {}'s creatures gain indestructible{}",
        pname(g, p),
        if addendum { ", a +1/+1 counter and vigilance" } else { "" }
    );
}

/// galadriel.make_a_stand: your creatures get +1/+0 and indestructible
pub fn make_a_stand(g: &mut Game, p: PlayerId) {
    for m in g.player(p).perms.clone() {
        if g.is_creature(m) && !g.perm(m).phased {
            add_kw(g, m, "indestructible");
        }
        if g.is_creature(m) {
            g.perm_mut(m).eot_pt.0 += 1;
        }
    }
    crate::glog!(g, "    {}'s creatures get +1/+0 and indestructible", pname(g, p));
}

// ================================================================== mana
/// galadriel._elves: the Elf creatures p controls
fn elves(g: &Game, p: PlayerId) -> u32 {
    g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased && has_type(g, m, "elf")).count() as u32
}

/// Elvish Archdruid: other Elves you control get +1/+1; {T}: {G} for each Elf you control (replaces t1's count,
/// which counted every Elf permanent)
fn archdruid_mana(g: &Game, p: PlayerId, _m: PermId) -> u32 {
    elves(g, p)
}

/// galadriel._drum_creatures
fn drum_creatures(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).tapped && !g.perm(m).phased).collect()
}

/// Springleaf Drum: {T}, tap an untapped creature you control: one mana of any colour
fn drum_mana(g: &Game, p: PlayerId, _m: PermId) -> u32 {
    !drum_creatures(g, p).is_empty() as u32
}

/// galadriel._drum_tap: tap an untapped creature you control as part of the cost: the AI taps one that won't attack.
/// HUMAN(phase 9): a person chooses.
fn drum_tap(g: &mut Game, p: PlayerId, _m: PermId, _used: u32) -> Res {
    let cs = drum_creatures(g, p);
    if let Some(x) = first_min(&cs, |x| {
        let y = g.perm(x);
        (!y.sick, !y.noatk, epow(g, x))
    }) {
        g.perm_mut(x).tapped = true;
    }
    Ok(())
}

// ================================================================== the AI: cast priorities
const INDES: [&str; 3] = ["Unbreakable Formation", "Make a Stand", "Rootborn Defenses"];

/// galadriel.galadriel_prio: the deck's cast priority (0-90)
pub fn galadriel_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    let d = g.db.get(c);
    let t = &d.tags;
    let n = &*d.name;
    let pl = g.player(p);
    let gal = named(g, p, "Galadriel, Light of Valinor");
    let cre = pl.perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased).count();
    if c == pl.cmd {
        return if pl.turns >= 3 { 84 } else { 66 };
    }
    if t.has(Tag::Ctr) || INDES.contains(&n) || n == "Eerie Interlude" {
        return 0; // held for their window
    }
    if t.has(Tag::Remora) {
        return if pl.turns <= 4 { 66 } else { 0 }; // Mystic Remora: only early, while upkeep is cheap
    }
    if (t.has(Tag::Rem) || t.has(Tag::Wipe)) && !d.perm {
        return 0; // removal and wipes: through their own decisions
    }
    if t.has(Tag::Rock) || t.has(Tag::Dork) || n == "Springleaf Drum" {
        return if pl.turns <= 5 { 82 } else { 30 };
    }
    if t.has(Tag::Lr) || n == "Farhaven Elf" || n == "Shared Roots" {
        return if pl.turns <= 5 { 78 } else { 25 };
    }
    if n == "Elvish Archdruid" {
        return if pl.turns <= 5 { 76 } else { 45 };
    }
    if n == LIN {
        return 74;
    }
    if searcher(n).is_some() {
        // the engine: first, while none is out
        let out = pl.perms.iter().any(|&m| card_name(g, m).is_some_and(|x| searcher(x).is_some() || x == LIN));
        let base = match n {
            "Ramosian Commander" => 66,
            "Ramosian Captain" => 64,
            "Defiant Vanguard" => 60,
            "Ramosian Lieutenant" => 62,
            _ => 63,
        };
        return base + if out { 0 } else { 10 };
    }
    if n == "Cathars' Crusade" || n == "Panharmonicon" {
        return if cre >= 2 || gal.is_some() { 72 } else { 50 };
    }
    if matches!(
        n,
        "Kindred Discovery"
            | "Vanquisher's Banner"
            | "Tocasia's Welcome"
            | "Beast Whisperer"
            | "Welcoming Vampire"
            | "Mentor of the Meek"
    ) {
        return 70;
    }
    if matches!(n, "Door of Destinies" | "Patchwork Banner" | "Maskwood Nexus") {
        return 58;
    }
    match n {
        "Mangara, the Diplomat" => return 56,
        "Adeline, Resplendent Cathar" => return 73,
        "Recruiter of the Guard" => return 65,
        "Abduction" => {
            return match best_steal(g, p, Some(c)) {
                Some(b) if pval(g, b) >= 3.0 => 80.min((40.0 + 6.0 * pval(g, b)) as i32),
                _ => 0,
            };
        }
        "Bribery" => return 68,
        "Crackdown" => {
            let theirs = g
                .opps(p)
                .flat_map(|q| g.player(q).perms.iter().copied())
                .filter(|&m| g.is_creature(m) && crackdown_holds(g, m))
                .count();
            let mine = pl.perms.iter().filter(|&&m| g.is_creature(m) && crackdown_holds(g, m)).count();
            return if theirs >= mine + 2 { 55 } else { 0 };
        }
        "Flicker" => {
            return match flicker_target(g, p) {
                Some(b) if etb_worth(g, p, b) >= 3.0 => 45,
                _ => 0,
            };
        }
        "Shamanic Revelation" => return if cre >= 4 { 60 } else { 15 },
        "Planar Genesis" => return if pl.turns <= 6 { 52 } else { 44 }, // a land or a card; nothing to wait for
        "Elspeth, Sun's Champion" => return 66,
        "Reya Dawnbringer" => return 50,
        _ => {}
    }
    if t.has(Tag::Tokatk) {
        return 70;
    }
    if d.creature {
        return 48;
    }
    if t.has(Tag::Draw) && (d.instant || d.sorcery) {
        return 46;
    }
    40
}

// ================================================================== the AI: protection and wipes
/// galadriel._cast_protect: cast an indestructible spell in response
fn cast_protect(g: &mut Game, owner: PlayerId, c: CardId) -> Res<bool> {
    match &*g.db.get(c).name {
        "Unbreakable Formation" => {
            if !pay_card(g, owner, c, 0)? {
                return Ok(false);
            }
            formation(g, owner, false);
            Ok(true)
        }
        "Make a Stand" => {
            if !pay_card(g, owner, c, 0)? {
                return Ok(false);
            }
            make_a_stand(g, owner);
            Ok(true)
        }
        "Rootborn Defenses" => {
            if !pay_card(g, owner, c, 0)? {
                return Ok(false);
            }
            rootborn(g, owner)?;
            Ok(true)
        }
        _ => Ok(false),
    }
}

/// galadriel.galadriel_protect (CI.galadriel_protect): removal at a key creature: indestructible (Unbreakable
/// Formation, Make a Stand, Rootborn Defenses) against destroy and damage, or Eerie Interlude (it returns at the end
/// step) against anything
pub fn galadriel_protect(
    g: &mut Game,
    owner: PlayerId,
    m: PermId,
    kind: Sym,
    _actor: Option<PlayerId>,
    _spell: Option<CardId>,
) -> Res<bool> {
    if !g.is_creature(m) || pval(g, m) < 4.0 {
        return Ok(false);
    }
    let destroyish = kind == "destroy" || kind.starts_with("dmg");
    for c in g.player(owner).hand.clone() {
        let d = g.db.get(c);
        if INDES.contains(&&*d.name) && destroyish && can_pay(g, owner, d.generic, &d.pips, false) {
            return cast_protect(g, owner, c);
        }
    }
    for c in g.player(owner).hand.clone() {
        let d = g.db.get(c);
        if &*d.name == "Eerie Interlude" && !g.perm(m).token && can_pay(g, owner, d.generic, &d.pips, false) {
            if !pay_card(g, owner, c, 0)? {
                return Ok(false);
            }
            interlude(g, owner, &[m])?;
            return Ok(true);
        }
    }
    Ok(false)
}

/// galadriel.galadriel_wipe_response (CI.galadriel_wipe_response): indestructible against a destroy or damage wipe,
/// else Eerie Interlude on every nontoken creature
pub fn galadriel_wipe_response(g: &mut Game, q: PlayerId, kind: Sym) -> Res<Option<Sym>> {
    let all = matches!(kind, "rift" | "rebuke");
    let loss = g.player(q).perms.iter().filter(|&&m| g.is_creature(m) || all).map(|&m| pval(g, m)).psum();
    if loss < 6.0 {
        return Ok(None);
    }
    if matches!(kind, "destroy" | "dmg13" | "austere" | "nib") {
        for c in g.player(q).hand.clone() {
            let d = g.db.get(c);
            if INDES.contains(&&*d.name) && can_pay(g, q, d.generic, &d.pips, false) {
                return Ok(if cast_protect(g, q, c)? { Some("indes") } else { None });
            }
        }
    }
    for c in g.player(q).hand.clone() {
        let d = g.db.get(c);
        if &*d.name == "Eerie Interlude" && can_pay(g, q, d.generic, &d.pips, false) {
            if !pay_card(g, q, c, 0)? {
                return Ok(None);
            }
            let ms: Vec<PermId> =
                g.player(q).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).token).collect();
            interlude(g, q, &ms)?;
            return Ok(None);
        }
    }
    Ok(None)
}

// ============================================================= Mirror Entity, Whipcorder's morph, Elspeth's emblem
/// galadriel.mirror: Mirror Entity: until end of turn your creatures have base power and toughness X/X and every
/// creature type (practice mode's activation, play/cards.py: phase 9 calls it)
pub fn mirror(g: &mut Game, p: PlayerId, x: i32) -> Res {
    for m in g.player(p).perms.clone() {
        if !g.is_creature(m) || g.perm(m).phased {
            continue;
        }
        let (pw, tg) = (g.perm(m).pow, g.perm(m).tgh);
        let e = &mut g.perm_mut(m).eot_pt;
        *e = (e.0 + x - pw, e.1 + x - tg);
        add_kw(g, m, "all types");
        if etgh(g, m) <= 0 {
            die(g, m, "sba")?;
        }
    }
    crate::glog!(g, "    {}'s creatures are {x}/{x} until end of turn", pname(g, p));
    Ok(())
}

/// galadriel.enter_face_down: morph: a face-down 2/2 creature with no name and no abilities (Whipcorder); turned face
/// up for its morph cost (practice mode, phase 9)
pub fn enter_face_down(g: &mut Game, p: PlayerId, c: CardId) -> Res<PermId> {
    let m = enter(g, p, c, Enter { was_cast: true, ..Enter::default() })?;
    let x = g.perm_mut(m);
    x.data.set(DataKey::Facedown, Val::List(vec![Val::Int(x.pow as i64), Val::Int(x.tgh as i64)]));
    x.pow = 2;
    x.tgh = 2;
    x.neutered = true;
    crate::glog!(g, "    {} has a face-down 2/2 creature", pname(g, p));
    Ok(m)
}

/// galadriel.turn_face_up (practice mode, phase 9)
pub fn turn_face_up(g: &mut Game, p: PlayerId, m: PermId) {
    let x = g.perm_mut(m);
    let (pw, tg) = match x.data.get(DataKey::Facedown) {
        Some(Val::List(v)) if v.len() == 2 => (v[0].int() as i32, v[1].int() as i32),
        _ => (x.pow, x.tgh),
    };
    x.pow = pw;
    x.tgh = tg;
    x.neutered = false;
    x.data.remove(DataKey::Facedown);
    crate::glog!(g, "  {} turns {} face up", pname(g, p), name_of(g, m));
}

/// galadriel.elspeth_emblem (CI.elspeth_emblem): creatures p controls get +2/+2 and have flying
pub fn elspeth_emblem(g: &mut Game, p: PlayerId) {
    g.player_mut(p).elspeth_emblem = true;
    g.dsl_on = true; // flying is read through the keyword checks
    crate::glog!(g, "    {} gets an emblem: creatures they control get +2/+2 and have flying", pname(g, p));
}

// ================================================================== stand-ins for other modules
// These port functions of modules other agents port in phase 6. Replace them with those modules' functions (same
// names) once they land.

/// stand-in for marchesa.best_steal: the most valuable creature p may gain control of with this spell
pub fn best_steal(g: &Game, p: PlayerId, spell: Option<CardId>) -> Option<PermId> {
    let tg: Vec<PermId> =
        legal_targets(g, p, "steal", "c", false, spell).into_iter().filter(|&m| !g.perm(m).phased).collect();
    max_by(&tg, |m| pval(g, m))
}

/// stand-in for marchesa.steal: p gains control of m (until end of turn: untapped, hasty, given back at the end
/// step)
pub fn steal(g: &mut Game, p: PlayerId, m: PermId, until_eot: bool) {
    let q = g.perm(m).owner;
    g.player_mut(q).perms.retain(|&x| x != m);
    let x = g.perm_mut(m);
    x.owner = p;
    x.attached = None;
    g.player_mut(p).perms.push(m);
    g.bf_ver += 1;
    if until_eot {
        let x = g.perm_mut(m);
        x.tapped = false;
        x.sick = false;
        g.player_mut(p).borrowed.push(m);
    } else {
        g.perm_mut(m).sick = true;
    }
    crate::glog!(g, "    {} gains control of {} ({})", pname(g, p), name_of(g, m), pname(g, q));
}

/// stand-in for zur.guildmage_target: an untapped opposing creature that could block your best attacker (your
/// commander first) and win the fight (also Galadriel's tappers)
pub fn guildmage_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let mine: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            g.is_creature(m) && !x.tapped && !x.phased && (!x.sick || x.is_cmd)
        })
        .collect();
    let a = max_by(&mine, |m| (g.perm(m).is_cmd as i32 * 5 + epow(g, m)) as f64)?;
    let blockers: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&b| {
            let x = g.perm(b);
            g.is_creature(b)
                && !x.tapped
                && !x.phased
                && !untargetable(g, b)
                && can_block(g, b, a)
                && epow(g, b) >= etgh(g, a)
        })
        .collect();
    max_by(&blockers, |b| epow(g, b) as f64)
}

/// stand-in for zur.populate: a copy of your best creature token
fn populate(g: &mut Game, p: PlayerId) -> Res {
    let toks: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).token && g.is_creature(m) && !g.perm(m).phased)
        .collect();
    let Some(t) = first_max(&toks, |m| (epow(g, m), g.perm(m).fly)) else { return Ok(()) };
    let x = g.perm(t).clone();
    let spec = Tokens {
        tgh: Some(x.tgh),
        fly: x.fly,
        color: Some(x.colors),
        types: x.ttypes.clone(),
        ..Tokens::new(1, x.pow)
    };
    for n in make_tokens(g, p, spec)? {
        let y = g.perm_mut(n);
        y.lifelink = x.lifelink;
        y.dt = x.dt;
        if !x.data.is_empty() {
            y.data = x.data.clone();
        }
    }
    Ok(())
}

/// stand-in for zur.rootborn: Rootborn Defenses resolves: populate, then indestructible until end of turn
fn rootborn(g: &mut Game, p: PlayerId) -> Res {
    populate(g, p)?;
    for m in g.player(p).perms.clone() {
        if g.is_creature(m) {
            add_kw(g, m, "indestructible");
        }
    }
    crate::glog!(g, "  {} casts Rootborn Defenses: creatures gain indestructible", pname(g, p));
    Ok(())
}

/// stand-in for t2.blink: exile m and return it under its owner's control (enter effects again, untapped, summoning
/// sick). PORT(phase 6, t2): CI.fire(g, 'exiled_from_bf', m) (Soulherder) once that event has a slot.
pub fn blink(g: &mut Game, _p: PlayerId, m: PermId) -> Res<Option<PermId>> {
    if g.perm(m).token || g.perm(m).cd.is_none() || !on_bf(g, m) {
        return Ok(None);
    }
    if g.blink_depth >= 3 {
        return Ok(None); // blink chains are combos, not loops here
    }
    g.blink_depth += 1;
    let r = (|| {
        let x = g.perm(m);
        let (cd, owner, cmd) = (x.cd.unwrap(), x.orig, x.is_cmd);
        leave(g, m)?;
        let n = enter(g, owner, cd, Enter { orig: Some(owner), ..Enter::default() })?;
        g.perm_mut(n).is_cmd = cmd;
        crate::glog!(g, "    {} is blinked", g.db.get(cd).name);
        Ok(Some(n))
    })();
    g.blink_depth -= 1;
    r
}

/// stand-in for t2.blink_value: how much re-entering is worth for p's permanent m
pub fn blink_value(g: &Game, _p: PlayerId, m: PermId) -> f64 {
    let x = g.perm(m);
    let Some(cd) = x.cd else { return 0.0 };
    if x.token || x.is_cmd {
        return 0.0;
    }
    let mut v = crate::dsl::etb_value(g, cd) as f64;
    if g.registry.get(cd).is_some_and(|i| i.etb.is_some()) {
        v += 3.0;
    }
    if matches!(
        &*g.db.get(cd).name,
        "Oblivion Ring" | "Banishing Light" | "Detention Sphere" | "Cast Out" | "Journey to Nowhere"
    ) {
        v = 0.0;
    }
    v
}

// ================================================================== registry
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    for n in ["Kindred Discovery", "Door of Destinies", "Patchwork Banner", "Vanquisher's Banner"] {
        r.card(db, n)?.as_enters = Some(name_type);
    }
    r.creature_pt.push(typed_pt);
    r.card(db, "Door of Destinies")?.cast = Some(door);
    let c = r.card(db, "Patchwork Banner")?;
    c.etb = Some(live); // registers the card (its +1/+1 lives in typed_pt)
    at_once(c, Event::Etb, true);
    r.card(db, "Vanquisher's Banner")?.cast = Some(vanquisher);
    let c = r.card(db, "Secluded Courtyard")?;
    c.land_cols = Some(courtyard_cols);
    c.land_etb = Some(courtyard_etb);
    c.etb = Some(live);
    at_once(c, Event::Etb, true);
    r.card(db, "Maskwood Nexus")?.options = Some(maskwood_ai);
    r.card(db, "Galadriel, Light of Valinor")?.etb = Some(alliance);
    r.card(db, "Panharmonicon")?.trigger_copies = Some(panharmonicon);
    for (n, _, _) in SEARCHERS {
        r.card(db, n)?.options = Some(searcher_opts);
    }
    r.card(db, LIN)?.options = Some(lin);
    r.card(db, "Ramosian Revivalist")?.options = Some(revivalist);
    for n in ["Lawbringer", "Lightbringer"] {
        r.card(db, n)?.options = Some(bringer_opts);
    }
    for n in ["Errant Doomsayers", "Whipcorder"] {
        r.card(db, n)?.options = Some(tapper_opts);
    }
    r.card(db, "Knight of the Holy Nimbus")?.self_regen = Some(nimbus_regen);
    let c = r.card(db, "Mangara, the Diplomat")?;
    c.attack = Some(mangara_attack);
    c.cast = Some(mangara_cast);
    r.card(db, "Tocasia's Welcome")?.etb = Some(tocasia);
    r.token_pt.push(voice_pt);
    let c = r.card(db, "Voice of Resurgence")?;
    c.cast = Some(voice_cast);
    c.self_dies = Some(voice_dies);
    let c = r.card(db, "Crackdown")?;
    c.etb = Some(live);
    at_once(c, Event::Etb, true);
    let c = r.card(db, "Abduction")?;
    c.etb = Some(abduction); // no trigger_window of its own: at once
    at_once(c, Event::Etb, true);
    c.dies = Some(abduction_dies);
    c.leaves = Some(abduction_leaves);
    at_once(c, Event::Leaves, true);
    c.sba = Some(abduction_sba);
    r.card(db, "Bribery")?.resolve = Some(bribery);
    r.card(db, "Eerie Interlude")?.resolve = Some(eerie);
    r.card(db, "Flicker")?.resolve = Some(flicker);
    let c = r.card(db, "Planar Genesis")?;
    c.resolve = Some(genesis);
    c.hand_options = Some(genesis_eot);
    r.card(db, "Return to Dust")?.resolve = Some(dust);
    r.card(db, "Unbreakable Formation")?.resolve = Some(formation_spell);
    let c = r.card(db, "Elvish Archdruid")?;
    c.etb = Some(live);
    at_once(c, Event::Etb, true);
    c.dyn_mana_perm = Some(archdruid_mana);
    let c = r.card(db, "Springleaf Drum")?;
    c.etb = Some(live);
    at_once(c, Event::Etb, true);
    c.dyn_mana_perm = Some(drum_mana);
    c.on_tap_perm = Some(drum_tap);
    Ok(())
}
