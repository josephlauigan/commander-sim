//! Python's `cards/impl/mine.py`: card code of your decks. Ported so far (M5): Sauron's cards (the Ring, Sauron, the
//! Necromancer, Kefka, Urabrask, the Avengers, Vraska, Barad-dûr, Bloodsoaked Insight, Kaervek, the Underworld Breach
//! line with Brain Freeze and Grapeshot), Kindred Discovery and Old Fat Spider's chapter II, which the engine reaches
//! through `cardcode.rs`. Also ais.py's `breach_gc_options` (Breach escapes, Panoptic Mirror, Lion's Eye Diamond,
//! Bolas's Citadel), which every deck reaches.

use super::lands::{can_pay_without, pay_without};
use super::partials::at_once;
use crate::ai::brain::{Situation, card_utility};
use crate::ai::decks::{breach_candidates, wipe_eval};
use crate::cards::{CardDb, Types};
use crate::engine::cast::{
    cast_card, cast_spell_copy, castable, casts_this_turn, discard_worst, is_casts, magecraft, on_cast, parse_cost,
};
use crate::engine::combat::blocked;
use crate::engine::hooks::fire_trigger;
use crate::engine::life::{check_state, lose_life};
use crate::engine::mana::{Source, can_pay, cost_of, mana_units, pay, total_mana};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::{
    abilities_answered, ability_window, ability_window_card, counter_window, equip_to, stack_window, trigger_window,
};
use crate::engine::tutors::card_worth;
use crate::engine::values::{
    card_name, card_tag, commander_out, epow, equipped, etgh, find, has, has_type, indestructible, melira, pval,
    stopped, threat, untargetable,
};
use crate::engine::zones::{
    Enter, LABMEN, Tokens, Zone, amass, die, discard_cards, discard_index, draw, enter, enter_token_copy, landfall,
    leave, make_tokens, max_by, mill, min_by, to_zone_card,
};
use crate::flow::Res;
use crate::hooks::{Action, Call, Event, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::pysum::PySum;
use crate::state::{Ctx, DataKey, Game, StackItem, StackKind, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;
use indexmap::IndexMap;

// ------------------------------------------------------------------ helpers
/// the first of xs with the highest key (Python's `max(xs, key=..)` for keys compared in order)
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

/// Python's `ability_window(g, p, src.cd, ...)`: an activated ability whose source is a card (a land, or a
/// planeswalker's loyalty ability named by its card): {imp} 3 unless given, no source permanent
fn card_ability_window(
    g: &mut Game,
    p: PlayerId,
    cd: CardId,
    name: &str,
    imp: Option<f64>,
    target: Option<PermId>,
) -> Res<bool> {
    if g.over || !abilities_answered(g, Some(p)) {
        return Ok(true);
    }
    let it = StackItem {
        id: g.stack_pushes + 1,
        controller: p,
        card: Some(cd),
        ctx: Ctx { target, ..Ctx::default() },
        zone: "ability",
        imp: imp.unwrap_or(3.0),
        aff: vec![],
        generic: false,
        kind: StackKind::Ability,
        name: format!("{}: {name}", g.db.get(cd).name),
        passed: vec![],
        countered: false,
        countered_by: None,
    };
    stack_window(g, p, it)
}

fn is_creature_card(g: &Game, c: CardId) -> bool {
    g.db.get(c).creature
}

/// remove the first copy of c from a card list
fn remove_card(xs: &mut Vec<CardId>, c: CardId) -> bool {
    match xs.iter().position(|&x| x == c) {
        Some(i) => {
            xs.remove(i);
            true
        }
        None => false,
    }
}

// ======================================================== the Ring (emblem)
/// mine.ring_bearer: p's Ring-bearer, while it is on p's battlefield and not phased out
pub fn ring_bearer(g: &Game, p: PlayerId) -> Option<PermId> {
    let m = g.player(p).ring_bearer?;
    let x = g.perm(m);
    (x.on_bf && x.owner == p && !x.phased).then_some(m)
}

/// mine.ring_level
pub fn ring_level(g: &Game, p: PlayerId) -> u32 {
    g.player(p).ring_level
}

/// mine.is_legendary: a legendary card, or (the Ring) your Ring-bearer
pub fn is_legendary(g: &Game, m: PermId) -> bool {
    if card_tag(g, m, Tag::Leg) {
        return true;
    }
    let o = g.perm(m).owner;
    ring_level(g, o) >= 1 && ring_bearer(g, o) == Some(m)
}

/// mine.ring_tempt: the Ring tempts you: the emblem gains its next ability, you choose a Ring-bearer (the Army first:
/// it is the threat, and legendary it turns Champion's Helm on), then the 'tempts you' / 'choose a Ring-bearer'
/// triggers. HUMAN(phase 9): a person chooses the Ring-bearer and answers Call of the Ring and Sauron.
pub fn ring_tempt(g: &mut Game, p: PlayerId) -> Res {
    let lvl = (ring_level(g, p) + 1).min(4);
    g.player_mut(p).ring_level = lvl;
    let cr: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).phased).collect();
    if !cr.is_empty() {
        let b = first_max(&cr, |m| (g.perm(m).army, epow(g, m) as f64 + 2.0 * etgh(g, m) as f64 / 5.0)).unwrap();
        g.player_mut(p).ring_bearer = Some(b);
        crate::glog!(g, "    The Ring tempts {} (level {lvl}): Ring-bearer {}", g.player(p).name, g.perm(b).name);
        for _ in find(g, p, Tag::Callring) {
            // Call of the Ring: you may pay 2 life to draw a card (not from an empty library)
            if g.player(p).life > 10 && !g.player(p).library.is_empty() {
                lose_life(g, p, 2, Some(p), "other", None)?;
                draw(g, p, 1, false)?;
            }
        }
    }
    if !find(g, p, Tag::Sauron).is_empty() && g.player(p).hand.len() <= 3 && g.player(p).library.len() >= 4 {
        // Sauron, the Dark Lord: you may discard your hand, then draw four (not when that would draw from an empty
        // library)
        let hand = g.player(p).hand.clone();
        discard_cards(g, p, &hand)?;
        draw(g, p, 4, false)?;
        crate::glog!(g, "    {} discards the hand and draws four (Sauron)", g.player(p).name);
    }
    Ok(())
}

/// m is p's Ring-bearer, the Ring is at level 2 and p's library is empty: attacking would loot from an empty library
/// and lose the game, so m stays home
pub fn ring_loot_decks(g: &Game, p: PlayerId, m: PermId) -> bool {
    ring_level(g, p) >= 2 && ring_bearer(g, p) == Some(m) && g.player(p).library.is_empty()
}

/// mine.ring_attack: level 2: whenever your Ring-bearer attacks, draw a card, then discard a card
pub fn ring_attack(g: &mut Game, p: PlayerId, atk: &[PermId]) -> Res {
    if let Some(b) = ring_bearer(g, p)
        && atk.contains(&b)
        && ring_level(g, p) >= 2
    {
        draw(g, p, 1, false)?;
        discard_worst(g, p, 1)?;
    }
    Ok(())
}

/// mine.ring_unblockable: level 1: your Ring-bearer can't be blocked by creatures with greater power
pub fn ring_unblockable(g: &Game, b: PermId, a: PermId) -> bool {
    let o = g.perm(a).owner;
    ring_level(g, o) >= 1 && ring_bearer(g, o) == Some(a) && epow(g, b) > epow(g, a)
}

/// mine.ring_blocked: level 3: a creature blocking your Ring-bearer is sacrificed at end of combat
pub fn ring_blocked(g: &Game, p: PlayerId, a: PermId, _b: PermId) -> bool {
    ring_level(g, p) >= 3 && ring_bearer(g, p) == Some(a)
}

/// mine.ring_damage: level 4: your Ring-bearer's combat damage to a player makes each opponent lose 3 life
pub fn ring_damage(g: &mut Game, p: PlayerId, a: PermId, _d: PlayerId) -> Res {
    if ring_level(g, p) >= 4 && ring_bearer(g, p) == Some(a) {
        for q in g.opps(p).collect::<Vec<_>>() {
            lose_life(g, q, 3, Some(p), "drain", None)?;
        }
    }
    Ok(())
}

// ======================================================== Sauron, the Necromancer
/// mine.necromancer_attack: menace; attacking exiles your best graveyard creature for a tapped attacking token copy
/// (a 3/3 black Wraith with menace, its abilities kept), exiled at end step unless Sauron is your Ring-bearer
pub fn necromancer_attack(g: &mut Game, p: PlayerId, src: PermId) -> Res<Vec<PermId>> {
    let cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| is_creature_card(g, c)).collect();
    if cs.is_empty() {
        return Ok(vec![]);
    }
    let c = first_max(&cs, |c| card_worth(g, p, c, true) + g.db.get(c).pow as f64).unwrap();
    let pl = g.player_mut(p);
    remove_card(&mut pl.gy, c);
    pl.exile.push(c);
    let Some(t) = enter_token_copy(g, p, c)? else { return Ok(vec![]) };
    let name = intern(&format!("Wraith ({})", g.db.get(c).name));
    let temp = ring_bearer(g, p) != Some(src);
    let x = g.perm_mut(t);
    (x.pow, x.tgh, x.plus, x.tapped, x.sick) = (3, 3, 0, true, false);
    x.name = name;
    if !x.eot_kw.contains(&"menace") {
        x.eot_kw.push("menace");
    }
    x.temp = temp;
    Ok(vec![t])
}

// ======================================================== Kindred Discovery
/// mine.orcish: an Army, Goblin or Orc
fn orcish(g: &Game, m: PermId) -> bool {
    g.perm(m).army || (g.perm(m).cd.is_some() && (has_type(g, m, "orc") || has_type(g, m, "goblin")))
}

/// mine.kindred_match: is m of the type Kindred Discovery named (Orc: Orcs and Orc Armies)
fn kindred_match(g: &Game, src: PermId, m: PermId) -> bool {
    let t = match g.perm(src).data.get(DataKey::Ctype) {
        Some(Val::Str(s)) => *s,
        _ => "orc",
    };
    if t == "orc" { orcish(g, m) } else { has_type(g, m, t) }
}

/// mine.kindred_enter: a creature of the named type entered without going through `enter` (an Orc Army)
pub fn kindred_enter(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    for src in find(g, p, Tag::Kindred) {
        if kindred_match(g, src, m) {
            draw(g, p, 1, false)?;
        }
    }
    Ok(())
}

/// names a creature type as it enters (Orc for Sauron: Orc Armies count): draws whenever a creature of that type you
/// control enters
fn kindred_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m != src
        && g.perm(m).owner == o
        && g.is_creature(m)
        && kindred_match(g, src, m)
        && trigger_window(g, o, Some(src), "draw a card", None)?
    {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

/// ... or attacks
fn kindred_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if p == g.perm(src).owner {
        let n = atk.iter().filter(|&&m| kindred_match(g, src, m)).count();
        if n > 0 && trigger_window(g, p, Some(src), &format!("draw {n}"), None)? {
            draw(g, p, n as u32, false)?;
        }
    }
    Ok(vec![])
}

// ======================================================== attack triggers: Iron Man, War Machine
/// mine.modified: counters, Equipment or an Aura on it
fn modified(g: &Game, m: PermId) -> bool {
    let o = g.perm(m).owner;
    g.perm(m).plus != 0
        || g.player(o).perms.iter().any(|&e| g.perm(e).attached == Some(m))
        || g.auras.iter().any(|&a| {
            let x = g.perm(a);
            x.attached == Some(m) && x.on_bf && !x.phased // common.auras_on
        })
}

/// flying; a +1/+1 counter on the Army per card you draw; attacking gives your other attacking modified creatures
/// flying
fn ironman(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == p
        && atk.contains(&src)
        && atk.iter().any(|&m| m != src && modified(g, m))
        && trigger_window(g, p, Some(src), "modified attackers gain flying", None)?
    {
        for &m in atk {
            if m != src && modified(g, m) && !g.perm(m).eot_kw.contains(&"flying") {
                g.perm_mut(m).eot_kw.push("flying");
            }
        }
    }
    Ok(vec![])
}

/// flying; attacking gives your attacking modified creatures double strike
fn warmachine(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner == p
        && atk.contains(&src)
        && atk.iter().any(|&m| modified(g, m))
        && trigger_window(g, p, Some(src), "modified attackers gain double strike", None)?
    {
        for &m in atk {
            if modified(g, m) && !g.perm(m).eot_kw.contains(&"double strike") {
                g.perm_mut(m).eot_kw.push("double strike");
            }
        }
    }
    Ok(vec![])
}

// ======================================================== Kaervek the Merciless
/// mine.kaervek: an opponent casts a spell: damage equal to its mana value to any target (the best creature it kills,
/// else the lowest life total). HUMAN(phase 9): a person picks the target.
pub fn kaervek(g: &mut Game, k: PlayerId, _caster: PlayerId, c: CardId) -> Res {
    let n = g.db.get(c).cmc as i32;
    if n <= 0 {
        return Ok(());
    }
    let opps: Vec<PlayerId> = g.opps(k).collect();
    let cr: Vec<PermId> = opps
        .iter()
        .flat_map(|&q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && !untargetable(g, m) && etgh(g, m) <= n)
        .collect();
    let best = max_by(&cr, |m| pval(g, m));
    if let Some(b) = best
        && pval(g, b) >= 4.0
    {
        apply_removal(g, Some(k), b, &format!("dmg{n}"), None)?;
    } else if let Some(q) = min_by(&opps, |q| (g.player(q).life - n) as f64) {
        lose_life(g, q, n, Some(k), "triggers", None)?;
    }
    Ok(())
}

// ======================================================== Vision, Synthezoid Avenger
/// flying; each spell cast outside its caster's turn: phases out if it threatens Vision, else a +1/+1 counter
fn vision(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = g.perm(src).owner;
    if g.active == Some(caster) || g.perm(src).phased || !g.perm(src).on_bf {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "phase out, or a +1/+1 counter", None)? {
        return Ok(());
    }
    if !g.perm(src).on_bf {
        return Ok(());
    }
    let target = match &g.cur_cast {
        Some((cc, ctx, _)) if *cc == c => ctx.target,
        _ => None,
    };
    if caster != o && (target == Some(src) || g.db.get(c).tag(Tag::Wipe)) {
        g.perm_mut(src).phased = true;
    } else {
        g.perm_mut(src).plus += 1;
    }
    Ok(())
}

// ======================================================== Scarlet Witch
/// flying; combat damage to a player exiles the top two, then casts the best Hero or noncreature card exiled with her
/// for free. HUMAN(phase 9): a person chooses the card (or none).
fn witch(g: &mut Game, src: Src, p: PlayerId, a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if a != src {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "exile the top two, cast one free", None)? {
        return Ok(());
    }
    let n = g.player(p).library.len().min(2);
    let mut top = vec![];
    for _ in 0..n {
        top.push(g.player_mut(p).library.pop().unwrap());
    }
    let mut held: Vec<CardId> = match g.perm(src).data.get(DataKey::Witch) {
        Some(Val::List(v)) => v.iter().filter_map(|x| if let Val::Card(c) = x { Some(*c) } else { None }).collect(),
        _ => vec![],
    };
    held.extend(&top);
    g.player_mut(p).exile.extend(&top);
    let ok: Vec<CardId> = held
        .iter()
        .copied()
        .filter(|&c| {
            let d = g.db.get(c);
            g.player(p).exile.contains(&c) && !d.land && (!d.creature || d.has_subtype("hero"))
        })
        .collect();
    let save = |g: &mut Game, held: &[CardId]| {
        g.perm_mut(src).data.set(DataKey::Witch, Val::List(held.iter().map(|&c| Val::Card(c)).collect()));
    };
    if ok.is_empty() {
        save(g, &held);
        return Ok(());
    }
    let c = max_by(&ok, |c| card_worth(g, p, c, false)).unwrap();
    remove_card(&mut g.player_mut(p).exile, c);
    remove_card(&mut held, c);
    save(g, &held);
    crate::glog!(g, "    Scarlet Witch: {} casts {} free", g.player(p).name, g.db.get(c).name);
    let ctx = crate::engine::cast::spell_targets(g, p, c, None);
    cast_card(g, p, c, "lib", ctx)?;
    Ok(())
}

// ======================================================== Vraska, Betrayal's Sting
/// mine._pw_once: no loyalty ability used this turn
fn pw_once(g: &Game, src: PermId) -> bool {
    !matches!(g.perm(src).data.get(DataKey::Act), Some(Val::Stamp(s)) if *s == g.turn_stamp())
}

/// mine._pw_use: pay the loyalty cost; at zero loyalty it goes to the graveyard
fn pw_use(g: &mut Game, src: PermId, cost: i32) -> Res<bool> {
    let st = g.turn_stamp();
    let x = g.perm_mut(src);
    x.data.set(DataKey::Act, Val::Stamp(st));
    let l = x.loyalty.unwrap_or(0) + cost;
    x.loyalty = Some(l);
    if l <= 0 {
        leave(g, src)?;
        to_zone_card(g, src, Zone::Gy);
        return Ok(false);
    }
    Ok(true)
}

/// loyalty 6: 0 draws, costs 1 life and proliferates (opponents' poison included); -2 turns the best opposing creature
/// into a Treasure; -9 sets a player to nine poison
fn vraska(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if p != x.owner || post.is_none() || x.phased || x.loyalty.is_none() || !pw_once(g, src) {
        return Ok(vec![]);
    }
    let loyalty = x.loyalty.unwrap();
    let mut out = vec![Opt {
        utility: if g.player(p).life > 10 { 2.0 } else { 0.5 },
        label: "Vraska 0 (draw, proliferate)".into(),
        act: Some(Action::Ability { src, f: vraska_zero, arg: 0 }),
    }];
    let cr: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && !untargetable(g, m))
        .collect();
    if !cr.is_empty() && loyalty >= 2 {
        let t = max_by(&cr, |m| pval(g, m)).unwrap();
        out.push(Opt {
            utility: pval(g, t) - 2.0,
            label: format!("Vraska -2 -> {}", g.perm(t).name),
            act: Some(Action::Ability { src, f: vraska_minus2, arg: t.0 as i64 }),
        });
    }
    if loyalty >= 9 {
        let opps: Vec<PlayerId> = g.opps(p).collect();
        if let Some(q) = max_by(&opps, |o| threat(g, p, o)) {
            out.push(Opt {
                utility: 12.0,
                label: format!("Vraska -9 -> {}", g.player(q).name),
                act: Some(Action::Ability { src, f: vraska_minus9, arg: q.0 as i64 }),
            });
        }
    }
    Ok(out)
}

fn vraska_zero(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !pw_once(g, src) {
        return Ok(false);
    }
    pw_use(g, src, 0)?;
    if ability_window(g, p, Some(src), "0", None, None)? {
        draw(g, p, 1, false)?;
        lose_life(g, p, 1, Some(p), "other", None)?;
        proliferate_all(g, p)?;
    }
    Ok(true)
}

fn vraska_minus2(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    if !pw_once(g, src) || !g.perm(t).on_bf {
        return Ok(false);
    }
    let cd = g.perm(src).cd.unwrap();
    pw_use(g, src, -2)?;
    crate::glog!(g, "  {} uses Vraska -2: {} becomes a Treasure", g.player(p).name, g.perm(t).name);
    if card_ability_window(g, p, cd, "-2", None, Some(t))? && g.perm(t).on_bf {
        // the card itself goes nowhere (Python: leave without to_zone_card)
        let q = g.perm(t).owner;
        leave(g, t)?;
        g.player_mut(q).treasures += 1;
    }
    Ok(true)
}

fn vraska_minus9(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let q = PlayerId(arg as u8);
    if !pw_once(g, src) {
        return Ok(false);
    }
    let cd = g.perm(src).cd.unwrap();
    pw_use(g, src, -9)?;
    if !card_ability_window(g, p, cd, "-9", Some(10.0), None)? {
        return Ok(true);
    }
    if !melira(g, q) {
        let pl = g.player_mut(q);
        pl.poison = pl.poison.max(9);
    }
    crate::glog!(g, "  {} uses Vraska -9: {} has nine poison counters", g.player(p).name, g.player(q).name);
    Ok(true)
}

/// mine.proliferate_all: proliferate: your Army and other creatures' +1/+1 counters, your loyalty, opponents' poison
/// and -1/-1 counters
pub fn proliferate_all(g: &mut Game, p: PlayerId) -> Res {
    for m in g.player(p).perms.clone() {
        if g.perm(m).phased {
            continue;
        }
        if g.perm(m).plus > 0 {
            crate::cardcode::add_counters(g, m, 1);
        }
        let walker = g.perm(m).cd.is_some_and(|c| g.db.get(c).types.has(Types::PLANESWALKER));
        if walker && let Some(l) = g.perm(m).loyalty {
            g.perm_mut(m).loyalty = Some(l + 1);
        }
    }
    for q in g.opps(p).collect::<Vec<_>>() {
        if g.player(q).poison > 0 && !melira(g, q) {
            g.player_mut(q).poison += 1;
        }
        for m in g.player(q).perms.clone() {
            if g.is_creature(m) && g.perm(m).plus < 0 {
                g.perm_mut(m).plus -= 1;
                if etgh(g, m) <= 0 {
                    die(g, m, "sba")?;
                }
            }
        }
    }
    check_state(g)
}

// ======================================================== Barad-dûr
/// enters tapped without a legendary creature; {T}: {B}; {X}{X}{B}, {T}: amass Orcs X after a creature died
fn baraddur(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.land(l).tapped || post.is_none() || g.died_turn != Some(g.turn_stamp()) {
        return Ok(vec![]);
    }
    let mut x = 0u32;
    while can_pay_without(g, p, l, 2 * (x + 1), "B") {
        x += 1;
    }
    if x < 1 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.0 + x as f64,
        label: format!("Barad-dûr (amass {x})"),
        act: Some(Action::Plan { f: baraddur_go, arg: ((x as i64) << 32) | l.0 as i64 }),
    }])
}

fn baraddur_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (l, x) = (LandId(arg as u32), (arg >> 32) as u32);
    if g.land(l).tapped || !pay_without(g, p, l, 2 * x, "B")? {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    crate::glog!(g, "  {} uses Barad-dûr: amass Orcs {x}", g.player(p).name);
    let cd = g.land(l).cd;
    if card_ability_window(g, p, cd, &format!("amass Orcs {x}"), None, None)? {
        amass(g, p, x as i32)?;
    }
    Ok(true)
}

// ======================================================== Bloodsoaked Insight // Sanguine Morass
/// mine.insight_cost: {5}, {1} less per life opponents lost this turn
fn insight_cost(g: &Game, p: PlayerId) -> u32 {
    let st = g.turn_stamp();
    let lost: i32 = g.opps(p).filter_map(|q| g.player(q).lost_turn.filter(|x| x.0 == st).map(|x| x.1)).sum();
    (5 - lost).max(0) as u32
}

/// cast the front face ({5}{B/R}{B/R}, {1} less per life opponents lost this turn): the top three of an opponent's
/// library, playable until the end of your next turn; otherwise it is a land (Sanguine Morass)
fn insight(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || !g.player(p).hand.contains(&c) || g.active != Some(p) {
        return Ok(vec![]);
    }
    let gn = insight_cost(g, p);
    if gn > 2 || !can_pay(g, p, gn, "B", false) || !can_pay(g, p, gn + 1, "B", false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 3.5 - gn as f64,
        label: "Bloodsoaked Insight".into(),
        act: Some(Action::Plan { f: insight_go, arg: ((gn as i64) << 32) | c.0 as i64 }),
    }])
}

fn insight_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, gn) = (CardId(arg as u16), (arg >> 32) as u32);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, gn + 1, "B", false) {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    pay(g, p, gn + 1, "B", false)?;
    g.player_mut(p).spells_this_turn += 1;
    on_cast(g, p, c)?;
    let opps: Vec<PlayerId> = g.opps(p).collect();
    if let Some(q) = max_by(&opps, |o| g.player(o).library.len() as f64) {
        let n = g.player(q).library.len().min(3);
        let mut top = vec![];
        for _ in 0..n {
            top.push(g.player_mut(q).library.pop().unwrap());
        }
        let pl = g.player_mut(p);
        pl.hand.extend(&top);
        let until = pl.turns + 1;
        pl.impulse_long.extend(top.iter().map(|&x| (x, until)));
        if g.log.is_some() {
            let names: Vec<&str> = top.iter().map(|&x| &*g.db.get(x).name).collect();
            crate::glog!(
                g,
                "  {} casts Bloodsoaked Insight: {} from {}",
                g.player(p).name,
                names.join(", "),
                g.player(q).name
            );
        }
    }
    g.player_mut(p).gy.push(c);
    Ok(true)
}

// ======================================================== Urabrask, Heretic Praetor
/// your upkeep: top card exiled, playable this turn; opponents: the next draw each upkeep is exiled instead and
/// playable that turn (so it is not a draw); unplayed cards stay exiled
fn urabrask(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = g.perm(src).owner;
    if p == o {
        if !g.player(p).library.is_empty()
            && trigger_window(g, p, Some(src), "exile the top card, playable this turn", None)?
            && let Some(c) = g.player_mut(p).library.pop()
        {
            let pl = g.player_mut(p);
            pl.hand.push(c);
            pl.impulse.push(c);
            pl.seen.insert(c);
            crate::glog!(g, "    Urabrask exiles {} (playable this turn)", g.db.get(c).name);
        }
    } else if g.player(p).alive
        && trigger_window(g, o, Some(src), &format!("{}'s next draw is exiled", g.player(p).name), None)?
    {
        g.player_mut(p).urabrask = Some(g.turn_stamp());
    }
    Ok(())
}

// ======================================================== Kefka, Court Mage // Kefka, Ruler of Ruin
/// mine._kefka_wheel: each player discards a card, then you draw a card for each card type among the discarded cards.
/// HUMAN(phase 9): a person picks their discard.
fn kefka_wheel(g: &mut Game, p: PlayerId) -> Res {
    let mut discarded = vec![];
    let alive: Vec<PlayerId> = g.players.iter().filter(|q| q.alive).map(|q| q.id).collect();
    for q in alive {
        if g.player(q).hand.is_empty() {
            continue;
        }
        let hand = g.player(q).hand.clone();
        let c = min_by(&hand, |x| card_worth(g, q, x, false)).unwrap(); // their least useful card
        discard_cards(g, q, &[c])?;
        discarded.push(c);
    }
    let mut bits = 0u8;
    for &c in &discarded {
        let t = g.db.get(c).types;
        for (i, k) in [
            Types::LAND,
            Types::CREATURE,
            Types::INSTANT,
            Types::SORCERY,
            Types::ARTIFACT,
            Types::ENCHANTMENT,
            Types::PLANESWALKER,
        ]
        .iter()
        .enumerate()
        {
            if t.has(*k) {
                bits |= 1 << i;
            }
        }
    }
    let n = bits.count_ones();
    if n > 0 {
        draw(g, p, n, false)?;
    }
    if g.log.is_some() {
        let names: Vec<&str> = discarded.iter().map(|&x| &*g.db.get(x).name).collect();
        let names = if names.is_empty() { "nothing".to_string() } else { names.join(", ") };
        crate::glog!(g, "    Kefka: {names} discarded, {} draws {n}", g.player(p).name);
    }
    Ok(())
}

/// enters or attacks: each player discards their least useful card, you draw one per card type discarded
fn kefka_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && trigger_window(g, o, Some(src), "each opponent discards", None)? {
        kefka_wheel(g, p)?;
    }
    Ok(())
}

fn kefka_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if atk.contains(&src)
        && !g.perm(src).data.truthy(DataKey::Ruin)
        && trigger_window(g, p, Some(src), "each opponent discards", None)?
    {
        kefka_wheel(g, p)?;
    }
    Ok(vec![])
}

/// {8} at sorcery speed: each opponent sacrifices their cheapest permanent (a token, else a land when they have lands
/// to spare), then it transforms: 5/7 flying
fn kefka_ruin(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != g.perm(src).owner
        || post.is_none()
        || g.active != Some(p)
        || g.perm(src).data.truthy(DataKey::Ruin)
        || !can_pay(g, p, 8, "", false)
    {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 3.0 + 0.5 * g.opps(p).count() as f64,
        label: "Kefka: {8}, transform".into(),
        act: Some(Action::Ability { src, f: kefka_ruin_go, arg: 0 }),
    }])
}

fn kefka_ruin_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !g.perm(src).on_bf || g.perm(src).owner != p || !can_pay(g, p, 8, "", false) {
        return Ok(false);
    }
    pay(g, p, 8, "", false)?;
    if !ability_window(g, p, Some(src), "{8}: each opponent sacrifices a permanent", Some(7.0), None)? {
        return Ok(true);
    }
    for q in g.opps(p).collect::<Vec<_>>() {
        let toks: Vec<PermId> =
            g.player(q).perms.iter().copied().filter(|&m| g.perm(m).token && !g.perm(m).phased).collect();
        if !toks.is_empty() {
            let m = min_by(&toks, |m| pval(g, m)).unwrap(); // their choice: the cheapest thing
            die(g, m, "sac")?;
        } else if g.player(q).lands.len() > 4 {
            sac_last_land(g, q);
        } else if !g.player(q).perms.is_empty() {
            let perms = g.player(q).perms.clone();
            let m = min_by(&perms, |m| pval(g, m)).unwrap();
            die(g, m, "sac")?;
        } else if !g.player(q).lands.is_empty() {
            sac_last_land(g, q);
        }
    }
    let x = g.perm_mut(src);
    x.data.set(DataKey::Ruin, Val::Bool(true));
    (x.pow, x.tgh, x.fly) = (5, 7, true);
    crate::glog!(
        g,
        "  {} activates Kefka ({{8}}): each opponent sacrifices a permanent; Kefka transforms",
        g.player(p).name
    );
    Ok(true)
}

/// q's last land to the graveyard (Python: `L = q.lands.pop(); q.gy.append(L.cd)`)
fn sac_last_land(g: &mut Game, q: PlayerId) {
    let l = *g.player(q).lands.last().unwrap();
    crate::engine::turn::remove_land(g, q, l);
    let cd = g.land(l).cd;
    g.player_mut(q).gy.push(cd);
}

/// Kefka, Ruler of Ruin: whenever an opponent loses life during your turn, you draw that many cards (not optional,
/// even if it decks you)
fn kefka_ruin_draw(g: &mut Game, src: Src, q: PlayerId, n: i32) -> Res {
    let p = g.perm(src).owner;
    if g.perm(src).data.truthy(DataKey::Ruin)
        && q != p
        && g.active == Some(p)
        && n > 0
        && g.player(p).alive
        && trigger_window(g, p, Some(src), &format!("draw {n}"), None)?
    {
        draw(g, p, n as u32, false)?;
    }
    Ok(())
}

// ======================================================== Old Fat Spider Can't See Me: chapter II
/// mine.damage_prevented: chapter II: all damage a creature would deal is prevented while the Saga remains
pub fn damage_prevented(g: &Game, m: PermId) -> bool {
    g.players.iter().any(|q| {
        q.perms
            .iter()
            .any(|&x| card_tag(g, x, Tag::Spider) && g.perm(x).data.get(DataKey::Prevent) == Some(&Val::Perm(m)))
    })
}

/// mine.spider_chapter2: the biggest opposing creature's damage is prevented while the Saga remains
pub fn spider_chapter2(g: &mut Game, p: PlayerId, saga: PermId) -> Res {
    let cr: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && !g.perm(m).phased && !untargetable(g, m))
        .collect();
    if let Some(t) = max_by(&cr, |m| epow(g, m) as f64) {
        g.perm_mut(saga).data.set(DataKey::Prevent, Val::Perm(t));
        crate::glog!(g, "    Old Fat Spider: all damage {} would deal is prevented", g.perm(t).name);
    }
    Ok(())
}

// ======================================================== Sauron: Underworld Breach + Brain Freeze / Grapeshot
// Storm: a copy per spell cast before it this turn (every player's). With Breach out, each one is recast from the
// graveyard for its mana cost plus three other graveyard cards; Brain Freeze on yourself supplies those cards.
const BRAIN_FREEZE: &str = "Brain Freeze";
const GRAPESHOT: &str = "Grapeshot";
const STORM_PIECES: [&str; 2] = [BRAIN_FREEZE, GRAPESHOT];
/// {generic} besides {B}, mana made, with threshold
const RITUALS: [(&str, u32, u32, u32); 2] = [("Dark Ritual", 0, 3, 3), ("Cabal Ritual", 1, 3, 5)];
/// {0}: sacrifice it for one mana of any colour
const PETAL: &str = "Lotus Petal";
/// {0}: discard your hand, sacrifice it: three of one colour
const LED: &str = "Lion's Eye Diamond";
/// {R} whenever you cast a spell: with Petal, a Brain Freeze
const BIRGI: &str = "Birgi, God of Storytelling // Harnfel, Horn of Bounty";
/// {2}{R}: {R} for each card in an opponent's hand
const JESKA: &str = "Jeska's Will";
const LINE_CARDS: [&str; 7] = [BRAIN_FREEZE, GRAPESHOT, "Dark Ritual", "Cabal Ritual", PETAL, LED, JESKA];

/// mine.storm_count: spells cast this turn before the one resolving now
pub fn storm_count(g: &Game) -> i32 {
    (g.players.iter().map(|q| casts_this_turn(g, q.id)).sum::<i32>() - 1).max(0)
}

/// mine._opp_by: the living opponent (not life-locked) with the lowest key
fn opp_by(g: &Game, p: PlayerId, key: impl Fn(PlayerId) -> f64) -> Option<PlayerId> {
    let opps: Vec<PlayerId> = g.opps(p).filter(|&q| !g.player(q).life_locked).collect();
    min_by(&opps, key)
}

/// storm (spells cast before it this turn, by anyone); each copy mills a player 3. Held for the Underworld Breach
/// line, where it mills you for fuel, then the table. The line passes its targets in ctx.mill (one per copy).
fn brain_freeze(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let n = ctx.storm.unwrap_or_else(|| storm_count(g)) + 1;
    for i in 0..n.max(0) as usize {
        let q = match ctx.mill.get(i) {
            Some(&q) => Some(q),
            None => opp_by(g, p, |x| g.player(x).library.len() as f64),
        };
        if let Some(q) = q
            && g.player(q).alive
        {
            mill(g, q, 3)?;
        }
    }
    crate::glog!(g, "    Brain Freeze: {n} cop{}, 3 cards each", if n == 1 { "y" } else { "ies" });
    Ok("gy")
}

/// storm; each copy deals 1 damage to any target (lowest-life opponent first, so the copies kill someone). Held for
/// the Breach line
fn grapeshot(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let n = ctx.storm.unwrap_or_else(|| storm_count(g)) + 1;
    let mut left = n;
    while left > 0 {
        let Some(q) = opp_by(g, p, |x| g.player(x).life as f64) else { break };
        let k = left.min(g.player(q).life.max(1));
        lose_life(g, q, k, Some(p), "burn", None)?;
        left -= k;
        check_state(g)?;
    }
    crate::glog!(g, "    Grapeshot: {n} damage");
    Ok("gy")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Wh {
    Hand,
    Gy,
}

/// mine._line_state: the numbers the Breach line decides on. Ritual mana floats as any colour in the engine, but a
/// real ritual makes only {B}: it pays generic costs and other rituals, never Brain Freeze's {U} or Grapeshot's {R}
struct LineState {
    land: u32,
    float: u32,
    u: u32,
    b: u32,
    r: u32,
    fuel: usize,
    gy: usize,
    storm: i32,
    /// where each of LINE_CARDS is (hand first), None when it's in the graveyard and graveyard casts are locked
    wh: [Option<Wh>; 7],
    breach: bool,
    breach_hand: bool,
    led_bf: bool,
    labman: bool,
    /// Laboratory Maniac / Jace / Birgi in hand: (card, generic, pips)
    setup_hand: Option<(CardId, u32, String)>,
    /// draw spells in hand or graveyard: (card, zone, generic, pip)
    draws: Vec<(CardId, Wh, u32, String)>,
    /// opponents' library sizes and life, in seat order
    lib: Vec<(PlayerId, usize)>,
    life: Vec<(PlayerId, i32)>,
    mylib: usize,
    opp_hand: usize,
}

impl LineState {
    fn w(&self, name: &str) -> Option<Wh> {
        self.wh[LINE_CARDS.iter().position(|&n| n == name).unwrap()]
    }

    /// st[col]: the real sources of a colour
    fn col(&self, col: &str) -> u32 {
        match col {
            "U" => self.u,
            "B" => self.b,
            "R" => self.r,
            _ => 0,
        }
    }

    fn life_of(&self, q: PlayerId) -> i32 {
        self.life.iter().find(|x| x.0 == q).map_or(0, |x| x.1)
    }
}

fn where_card(g: &Game, p: PlayerId, c: Option<CardId>) -> Option<Wh> {
    let c = c?;
    if g.player(p).hand.contains(&c) {
        Some(Wh::Hand)
    } else if g.player(p).gy.contains(&c) {
        Some(Wh::Gy)
    } else {
        None
    }
}

fn is_line_card(g: &Game, c: CardId) -> bool {
    LINE_CARDS.contains(&&*g.db.get(c).name)
}

fn perm_named(g: &Game, p: PlayerId, name: &str, live: bool) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| card_name(g, m) == Some(name) && !(live && g.perm(m).phased))
}

fn line_state(g: &Game, p: PlayerId) -> LineState {
    let real: Vec<_> = mana_units(g, p, false).into_iter().filter(|u| u.src != Source::FloatAny).collect();
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let bf = g.db.id(BRAIN_FREEZE).unwrap();
    let gy_ok = g.hooks.is_empty() || castable(g, p, bf, "gy"); // a lock on casting from graveyards stops escapes
    let pl = g.player(p);
    let colour = |c: char| real.iter().filter(|u| u.cols.has(c)).map(|u| u.amt).sum::<u32>();
    let mut wh = [None; 7];
    for (i, n) in LINE_CARDS.iter().enumerate() {
        let w = where_card(g, p, g.db.id(n));
        wh[i] = if gy_ok || w != Some(Wh::Gy) { w } else { None };
    }
    let setup_hand = if g.line_no_setup {
        None
    } else {
        pl.hand
            .iter()
            .find(|&&c| {
                let d = g.db.get(c);
                (LABMEN.contains(&&*d.name) || &*d.name == BIRGI)
                    && d.pips.chars().count() == 1
                    && !pl.perms.iter().any(|&m| g.perm(m).cd.is_some_and(|x| g.db.get(x).name == d.name))
            })
            .map(|&c| (c, g.db.get(c).generic, g.db.get(c).pips.to_string()))
    };
    let mut draws = vec![];
    for (z, cs) in [(Wh::Hand, &pl.hand), (Wh::Gy, &pl.gy)] {
        for &c in cs {
            let d = g.db.get(c);
            if d.tags.int(Tag::Draw).is_some()
                && (d.instant || d.sorcery)
                && d.pips.chars().count() == 1
                && !is_line_card(g, c)
                && (z == Wh::Hand || gy_ok)
            {
                draws.push((c, z, d.generic, d.pips.to_string()));
            }
        }
    }
    LineState {
        land: real.iter().map(|u| u.amt).sum(),
        float: pl.floating.any,
        u: colour('U'),
        b: colour('B'),
        r: colour('R'),
        fuel: pl.gy.iter().filter(|&&c| !is_line_card(g, c)).count(),
        gy: pl.gy.len(),
        storm: g.players.iter().map(|q| casts_this_turn(g, q.id)).sum(),
        wh,
        breach: has(g, p, Tag::Breach),
        breach_hand: pl.hand.iter().any(|&c| g.db.get(c).tag(Tag::Breach)),
        led_bf: perm_named(g, p, LED, true).is_some(),
        labman: pl.perms.iter().any(|&m| !g.perm(m).phased && card_name(g, m).is_some_and(|n| LABMEN.contains(&n))),
        setup_hand,
        draws,
        lib: opps.iter().map(|&q| (q, g.player(q).library.len())).collect(),
        life: opps.iter().map(|&q| (q, g.player(q).life)).collect(),
        mylib: pl.library.len(),
        opp_hand: if g.line_no_setup { 0 } else { opps.iter().map(|&q| g.player(q).hand.len()).max().unwrap_or(0) },
    }
}

/// mine._can: can the line pay {gn} plus one {col}? ({B} can come from ritual mana)
fn can(st: &LineState, gn: u32, col: &str) -> bool {
    let ok = if col == "B" { st.float > 0 || st.b > 0 } else { st.col(col) > 0 && st.land > 0 };
    ok && st.float + st.land >= gn + 1
}

fn ready(st: &LineState, name: &str) -> bool {
    let w = st.w(name);
    w == Some(Wh::Hand) || (w == Some(Wh::Gy) && st.fuel >= 3)
}

fn ceil3(n: i64) -> i64 {
    (n + 2).div_euclid(3)
}

/// mine._mill_plan: who each Brain Freeze copy mills (st: after paying for it). The table if the copies finish it.
/// Otherwise yourself first, enough fuel to pay for the next Brain Freeze (Lotus Petal escapes for {U}, rituals for
/// generic), escape it, and recast each ritual twice more for storm; then the opponent closest to an empty library.
/// If nothing can pay the next Brain Freeze's {U}, this is the last one: every copy goes to the table
fn mill_plan(p: PlayerId, st: &LineState, copies: i64) -> Vec<PlayerId> {
    if st.labman {
        return vec![p; copies.max(0) as usize]; // Laboratory Maniac / Jace: mill yourself out, then draw
    }
    let mut lib: Vec<(PlayerId, i64)> = st.lib.iter().map(|&(q, n)| (q, n as i64)).collect();
    let need: i64 = lib.iter().filter(|&&(q, n)| n > 0 && st.life_of(q) > 0).map(|&(_, n)| ceil3(n)).sum();
    let need_u: i64 = if st.u > 0 && st.land > 0 { 0 } else { 1 };
    let need_gen = (2 - st.land as i64 - st.float as i64 - need_u).max(0);
    let petal = st.w(PETAL).is_some();
    let led = st.w(LED).is_some() || st.led_bf;
    let rit = RITUALS.iter().filter(|r| st.w(r.0).is_some()).count() as i64;
    let jeska = if st.w(JESKA).is_some() { 3 } else { 0 }; // fuel for a Jeska's Will escape
    let last = need_u != 0 && !petal && !led;
    let keep = if led {
        // one Lion's Eye Diamond escape pays the whole next Brain Freeze; plus three spare: milled line cards aren't
        // fuel
        6 + if need_u != 0 || need_gen != 0 { 3 } else { 0 } + jeska
    } else {
        let mut esc = if petal { need_u + if rit > 0 { 0 } else { need_gen } } else { 0 };
        esc += if rit > 0 { (need_gen + 1).div_euclid(2) } else { 0 };
        6 + 3 * esc + 6 * rit + jeska // three spare: milled line cards aren't fuel
    };
    let mine = if copies >= need || last {
        0
    } else {
        copies.min(ceil3((keep - st.fuel as i64).max(0))).min(st.mylib as i64 / 3)
    };
    let mut plan = vec![p; mine.max(0) as usize];
    for _ in 0..(copies - mine).max(0) {
        let live: Vec<PlayerId> = lib.iter().filter(|&&(q, n)| n > 0 && st.life_of(q) > 0).map(|&(q, _)| q).collect();
        if live.is_empty() {
            // every opponent is milled out: the spare copy mills one of them (nothing), never yourself. (Python sent it
            // at you, and Sauron decked himself on his own draws before the table drew from empty libraries.)
            plan.push(st.lib.first().map_or(p, |x| x.0));
            continue;
        }
        let q = min_by(&live, |x| lib.iter().find(|y| y.0 == x).unwrap().1 as f64).unwrap();
        lib.iter_mut().find(|y| y.0 == q).unwrap().1 -= 3;
        plan.push(q);
    }
    plan
}

/// mine._draw_ready: a draw spell the line can cast (from hand) or escape (from the graveyard) now
fn draw_ready(st: &LineState) -> Option<(CardId, Wh, u32, String)> {
    st.draws.iter().find(|(_, z, gn, pip)| (*z == Wh::Hand || st.fuel >= 4) && can(st, *gn, pip)).cloned() // (it isn't its own fuel)
}

/// the Breach line's next cast
#[derive(Debug, Clone, Copy, PartialEq)]
enum LineAct {
    Setup,
    Breach,
    Draw,
    Card(&'static str),
}

/// mine._line_step: the next cast of the Breach line, or None. Fuel (graveyard cards) goes first to what the next
/// Brain Freeze needs: its own escape, and a Lotus Petal escape when blue mana runs out; spare fuel becomes ritual
/// escapes, which add storm (and pay generic costs)
fn line_step(st: &LineState) -> Option<LineAct> {
    if let Some((_, gn, pips)) = &st.setup_hand
        && can(st, *gn, pips)
    {
        return Some(LineAct::Setup); // Laboratory Maniac / Birgi first, from hand
    }
    if !st.breach {
        return (st.breach_hand && can(st, 1, "R")).then_some(LineAct::Breach);
    }
    let alive: Vec<PlayerId> = st.lib.iter().filter(|&&(q, n)| n > 0 && st.life_of(q) > 0).map(|&(q, _)| q).collect();
    if alive.is_empty() {
        return None;
    }
    let (hand, gy) = (Some(Wh::Hand), Some(Wh::Gy));
    if st.labman && st.mylib == 0 {
        // library empty: draw a card to win
        if draw_ready(st).is_some() {
            return Some(LineAct::Draw);
        }
        // short of its mana: Petal escapes pay for it
        let res = if st.draws.iter().any(|d| d.1 == Wh::Gy) { 4 } else { 0 };
        if !st.draws.is_empty() && (st.w(PETAL) == hand || (st.w(PETAL) == gy && st.fuel >= 3 + res)) {
            return Some(LineAct::Card(PETAL));
        }
        return None;
    }
    let bf_res = if st.w(BRAIN_FREEZE) == gy { 3 } else { 0 };
    let petal_ready = st.w(PETAL) == hand || (st.w(PETAL) == gy && st.fuel >= 3 + bf_res);
    let petal_res = if st.w(PETAL).is_some() && st.u <= 1 { 3 } else { 0 }; // the next Brain Freeze will want a Petal
    let led_ready = |res: usize| st.led_bf || st.w(LED) == hand || (st.w(LED) == gy && st.fuel >= 3 + res);
    let ritual = |res: usize| {
        for &(n, gn, made, thr) in &RITUALS {
            let got = if st.gy >= 7 { thr } else { made };
            if (st.w(n) == hand || (st.w(n) == gy && st.fuel >= 3 + res)) && got > gn + 1 && can(st, gn, "B") {
                return Some(LineAct::Card(n));
            }
        }
        None
    };
    let copies = st.storm + 1;
    let gs = st.w(GRAPESHOT).is_some() && ready(st, GRAPESHOT) && can(st, 1, "R");
    if gs && copies >= alive.iter().map(|&q| st.life_of(q)).min().unwrap() {
        return Some(LineAct::Card(GRAPESHOT));
    }
    let gs_or_none = if gs { Some(LineAct::Card(GRAPESHOT)) } else { None };
    let bf = st.w(BRAIN_FREEZE).is_some() && ready(st, BRAIN_FREEZE);
    if st.u == 0 || st.land == 0 {
        // no blue left: a Diamond or a Petal makes it
        if led_ready(bf_res) && bf {
            return Some(LineAct::Card(LED));
        }
        if petal_ready && bf {
            return Some(LineAct::Card(PETAL));
        }
        return gs_or_none;
    }
    if st.land + st.float < 2 {
        // blue, but short of the generic {1}
        if led_ready(bf_res) && bf {
            return Some(LineAct::Card(LED));
        }
        if let Some(r) = ritual(bf_res + petal_res) {
            return Some(r);
        }
        if petal_ready && bf {
            return Some(LineAct::Card(PETAL));
        }
        return gs_or_none;
    }
    if st.opp_hand as i64 - 3 >= 2
        && st.w(JESKA).is_some()
        && (st.w(JESKA) == hand || st.fuel >= 3 + bf_res + petal_res)
        && can(st, 2, "R")
    {
        return Some(LineAct::Card(JESKA)); // Jeska's Will: a big hand is a big mana burst
    }
    if let Some(r) = ritual(bf_res + petal_res) {
        return Some(r); // spare fuel first becomes storm
    }
    if st.w(LED).is_some() && led_ready(bf_res + 3) {
        return Some(LineAct::Card(LED)); // (a Diamond escape: storm and three more mana)
    }
    if bf {
        return Some(LineAct::Card(BRAIN_FREEZE));
    }
    gs_or_none
}

/// mine._pay_line: pay for a spell of the line: the pip from a real source ({B} may use ritual mana), generic from
/// ritual mana first
fn pay_line(g: &mut Game, p: PlayerId, gn: u32, col: &str) -> Res<bool> {
    if col == "B" && g.player(p).floating.any > 0 {
        g.player_mut(p).floating.any -= 1;
    } else if !pay(g, p, 0, col, false)? {
        return Ok(false);
    }
    let fl = &mut g.player_mut(p).floating;
    let k = fl.any.min(gn);
    fl.any -= k;
    Ok(gn - k == 0 || pay(g, p, gn - k, "", false)?)
}

/// escape: exile three other graveyard cards (the least useful, line cards and `skip` kept); false if there aren't
/// three
fn escape_fuel(g: &mut Game, p: PlayerId, skip: Option<CardId>) -> bool {
    let mut fuel: Vec<(f64, CardId)> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&x| !is_line_card(g, x) && Some(x) != skip)
        .map(|x| (card_worth(g, p, x, true), x))
        .collect();
    if fuel.len() < 3 {
        return false;
    }
    fuel.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let pl = g.player_mut(p);
    for &(_, x) in &fuel[..3] {
        remove_card(&mut pl.gy, x);
        pl.exile.push(x);
    }
    pl.stat("breach_escapes", 1);
    true
}

fn zone_sym(w: Wh) -> Sym {
    if w == Wh::Gy { "escape" } else { "hand" }
}

/// mine._dry_run: would the line finish every opponent if nobody interacts? Played for real on a copy of the game
/// with the opponents' hands removed (so it neither cheats nor fears their counters); cached for the position. Inside
/// a look-ahead playout (itself a copy) it reuses the real game's answer for this turn (the copy carries it).
fn dry_run(g: &mut Game, p: PlayerId) -> Res<bool> {
    let stamp = g.turn_stamp();
    if g.in_search {
        return Ok(matches!(&g.breach_dry, Some((st, _, ok)) if *st == stamp && *ok));
    }
    let pl = g.player(p);
    let mut key = vec![
        total_mana(g, p, false),
        pl.floating.any,
        pl.gy.len() as u32,
        pl.hand.len() as u32,
        pl.library.len() as u32,
        g.players.iter().map(|q| casts_this_turn(g, q.id)).sum::<i32>() as u32,
    ];
    key.extend(g.players.iter().map(|q| q.library.len() as u32));
    if let Some((st, k, ok)) = &g.breach_dry
        && *st == stamp
        && *k == key
    {
        return Ok(*ok);
    }
    let island = g.db.id("Island").unwrap();
    let (mut ok, mut no_setup) = (false, false);
    for ns in [false, true] {
        // with Lab Man / Birgi / Jeska's Will, then the plain line
        no_setup = ns;
        let mut g2 = crate::ai::search::clone(g);
        for q in g2.opps(p).collect::<Vec<_>>() {
            let n = g2.player(q).hand.len(); // no answers in hand, but the same number of cards (Jeska's Will)
            g2.player_mut(q).hand = vec![island; n];
        }
        g2.line_no_setup = ns;
        breach_line(&mut g2, p, true)?;
        ok = g2.winner == Some(p) || g2.opps(p).all(|q| !g2.player(q).alive || g2.player(q).library.is_empty());
        if ok {
            break;
        }
    }
    g.line_no_setup = ok && no_setup; // the real line follows the plan that worked
    g.breach_dry = Some((stamp, key, ok));
    Ok(ok)
}

/// mine.breach_line: Sauron's Breach line. Dry run (execute false): does it finish every opponent (burned to 0 or
/// milled out) if nobody interacts? Execute: cast it for real, one spell at a time (each can be countered).
pub fn breach_line(g: &mut Game, p: PlayerId, execute: bool) -> Res<bool> {
    if g.player(p).key != "sauron" || g.opps(p).next().is_none() {
        return Ok(false);
    }
    let st = line_state(g, p);
    if !st.breach && !st.breach_hand {
        return Ok(false);
    }
    if !STORM_PIECES.iter().any(|n| st.w(n).is_some()) {
        return Ok(false);
    }
    if !execute {
        // Rust only (a fix): a line with nothing left to do (every opponent's library already empty, or no first cast
        // it can make) isn't offered. Python's dry run counted empty libraries as the line finishing the table, so the
        // AI kept choosing the 15.0 "Underworld Breach line" option, which did nothing, until the main-phase action
        // cap. The first step is looked at with the setup casts allowed (the line's widest choice).
        let saved = g.line_no_setup;
        g.line_no_setup = false;
        let st0 = line_state(g, p);
        g.line_no_setup = saved;
        let target = st0.lib.iter().any(|&(q, n)| n > 0 && st0.life_of(q) > 0); // (the line's own test of who's left)
        if !target || line_step(&st0).is_none() {
            return Ok(false);
        }
        return dry_run(g, p);
    }
    let turns = g.player(p).turns;
    let pl = g.player_mut(p);
    pl.stat("breach_line", 1);
    pl.milestone.entry("combo").or_insert(turns);
    crate::glog!(g, "  {} goes for the Underworld Breach line", g.player(p).name);
    for _ in 0..300 {
        if g.over || !g.player(p).alive {
            break;
        }
        let st = line_state(g, p);
        let Some(act) = line_step(&st) else { break };
        match act {
            LineAct::Breach => {
                let c = *g.player(p).hand.iter().find(|&&x| g.db.get(x).tag(Tag::Breach)).unwrap();
                if !pay_line(g, p, 1, "R")? {
                    break;
                }
                if !cast_card(g, p, c, "hand", Ctx::default())? || !has(g, p, Tag::Breach) {
                    crate::glog!(g, "    ...Underworld Breach is stopped");
                    break;
                }
            }
            LineAct::Card(LED) => {
                if !line_led(g, p, &st)? {
                    break;
                }
            }
            LineAct::Card(JESKA) => {
                let zone = st.w(JESKA).unwrap();
                let c = g.db.id(JESKA).unwrap();
                if zone == Wh::Gy && !escape_fuel(g, p, None) {
                    break;
                }
                if !pay_line(g, p, 2, "R")? {
                    break;
                }
                g.jeska_mana = true; // in the line it's always the mana mode
                let r = cast_card(g, p, c, zone_sym(zone), Ctx::default());
                g.jeska_mana = false;
                r?;
            }
            LineAct::Setup => {
                let (c, gn, pips) = st.setup_hand.clone().unwrap();
                if !pay_line(g, p, gn, &pips)? {
                    break;
                }
                cast_card(g, p, c, "hand", Ctx::default())?;
                if !g
                    .player(p)
                    .perms
                    .iter()
                    .any(|&m| g.perm(m).cd.is_some_and(|x| g.db.get(x).name == g.db.get(c).name))
                {
                    break; // countered
                }
            }
            LineAct::Draw => {
                // Laboratory Maniac / Jace: the draw that wins
                let (c, zone, gn, pip) = draw_ready(&st).unwrap();
                if zone == Wh::Gy && !escape_fuel(g, p, Some(c)) {
                    break;
                }
                if !pay_line(g, p, gn, &pip)? {
                    break;
                }
                cast_card(g, p, c, zone_sym(zone), Ctx::default())?;
            }
            LineAct::Card(act) => {
                let zone = st.w(act).unwrap();
                let c = g.db.id(act).unwrap();
                let mut ctx = Ctx { storm: Some(st.storm), ..Ctx::default() };
                let pay_for: Option<(u32, &str)> = match RITUALS.iter().find(|r| r.0 == act) {
                    Some(r) => Some((r.1, "B")),
                    None if act == PETAL => None,
                    None => Some((1, if act == BRAIN_FREEZE { "U" } else { "R" })),
                };
                if zone == Wh::Gy && !escape_fuel(g, p, None) {
                    break;
                }
                if let Some((gn, col)) = pay_for
                    && !pay_line(g, p, gn, col)?
                {
                    break;
                }
                if act == BRAIN_FREEZE {
                    ctx.mill = mill_plan(p, &line_state(g, p), st.storm as i64 + 1); // as paid for
                }
                cast_card(g, p, c, zone_sym(zone), ctx)?;
                if act == PETAL
                    && let Some(m) = perm_named(g, p, PETAL, false)
                {
                    // sacrifice it at once, so it can be escaped again
                    leave(g, m)?;
                    to_zone_card(g, m, Zone::Gy);
                    let fl = &mut g.player_mut(p).floating;
                    if st.labman && st.mylib == 0 {
                        fl.any += 1; // any colour, for the winning draw
                    } else if st.u == 0 {
                        fl.u += 1;
                    } else if st.r == 0 {
                        fl.r += 1;
                    } else {
                        fl.any += 1;
                    }
                }
            }
        }
    }
    check_state(g)?;
    Ok(true)
}

/// mine._line_led: Lion's Eye Diamond in the Breach line: cast it (from hand, or escaped from the graveyard) unless
/// it's already out, then discard your hand and sacrifice it for three mana: blue for Brain Freeze, red when only
/// Grapeshot is left
fn line_led(g: &mut Game, p: PlayerId, st: &LineState) -> Res<bool> {
    if !st.led_bf {
        let zone = st.w(LED).unwrap();
        let c = g.db.id(LED).unwrap();
        if zone == Wh::Gy && !escape_fuel(g, p, None) {
            return Ok(false);
        }
        cast_card(g, p, c, zone_sym(zone), Ctx::default())?;
    }
    let Some(m) = perm_named(g, p, LED, true) else { return Ok(false) }; // countered
    let hand = g.player(p).hand.clone();
    discard_cards(g, p, &hand)?;
    leave(g, m)?;
    to_zone_card(g, m, Zone::Gy);
    let col = if st.r == 0 && st.w(GRAPESHOT).is_some() && st.w(BRAIN_FREEZE).is_none() { 'R' } else { 'U' };
    let pl = g.player_mut(p);
    if col == 'U' {
        pl.floating.u += 3;
    } else {
        pl.floating.r += 3;
    }
    pl.stat("led_used", 1);
    crate::glog!(g, "    Lion's Eye Diamond: {} discards the hand for {}", g.player(p).name, col.to_string().repeat(3));
    Ok(true)
}

/// mine.breach_options: the AI's main-phase option: go for the Breach line when the dry run finishes the table
pub fn breach_options(g: &mut Game, p: PlayerId, _post: bool) -> Res<Vec<Opt>> {
    if g.active != Some(p) || !breach_line(g, p, false)? {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 15.0,
        label: "Underworld Breach line".into(),
        act: Some(Action::Plan { f: breach_go, arg: 0 }),
    }])
}

fn breach_go(g: &mut Game, p: PlayerId, _arg: i64) -> Res<bool> {
    breach_line(g, p, true)
}

// ======================================================== ais.py: Breach escapes, Panoptic Mirror, LED, Citadel
const SPECIALX: [Tag; 6] = [Tag::Rean, Tag::Fill, Tag::Yawg, Tag::Avarice, Tag::Mastery, Tag::Crackle];

/// the best legal target of a removal card (its `rem` tag), if any
fn best_target(g: &Game, p: PlayerId, c: CardId) -> Option<PermId> {
    let t = &g.db.get(c).tags;
    let rk = t.str(Tag::Rem)?;
    let tg = legal_targets(g, p, rk, t.str(Tag::Tgt).unwrap_or("c"), t.has(Tag::Mv4), Some(c));
    max_by(&tg, |m| pval(g, m))
}

/// ais.breach_escape: cast c from the graveyard with Underworld Breach (its mana cost, exile three others)
fn breach_escape(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let pl = g.player(p);
    if !pl.gy.contains(&c) || pl.gy.len() < 4 || !has(g, p, Tag::Breach) {
        return Ok(false);
    }
    if !g.hooks.is_empty() && !castable(g, p, c, "gy") {
        return Ok(false); // Drannith Magistrate, Rule of Law ...
    }
    let (cg, cp) = cost_of(g, p, c);
    if !can_pay(g, p, cg, &cp, false) {
        return Ok(false);
    }
    pay(g, p, cg, &cp, false)?;
    let mut others: Vec<(f64, CardId)> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&x| x != c)
        .map(|x| (card_worth(g, p, x, true) + 0.01 * card_worth(g, p, x, false), x))
        .collect();
    others.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    let pl = g.player_mut(p);
    for &(_, x) in others.iter().take(3) {
        remove_card(&mut pl.gy, x);
        pl.exile.push(x);
    }
    let mut ctx = Ctx::default();
    if g.db.get(c).tag(Tag::Rem)
        && let Some(t) = best_target(g, p, c)
    {
        ctx.target = Some(t);
    }
    crate::glog!(g, "  {} escapes {} with Underworld Breach", g.player(p).name, g.db.get(c).name);
    g.player_mut(p).stat("breach_escapes", 1);
    cast_card(g, p, c, "escape", ctx)?;
    Ok(true)
}

/// ais.mirror_candidates: instants/sorceries in hand worth imprinting on Panoptic Mirror (copied free every upkeep)
pub fn mirror_candidates(g: &Game, p: PlayerId) -> Vec<(f64, CardId)> {
    let mut out = vec![];
    for &c in &g.player(p).hand {
        let d = g.db.get(c);
        let t = &d.tags;
        if !(d.instant || d.sorcery) || t.has(Tag::Ctr) || t.has(Tag::X) || t.has(Tag::Tokx) {
            continue;
        }
        if SPECIALX.iter().any(|&k| t.has(k)) || t.has(Tag::Fbgrant) {
            continue;
        }
        let mut v = card_worth(g, p, c, false);
        if t.has(Tag::Rem) {
            v = v.max(40.0);
        }
        if t.has(Tag::Wipe) {
            v = v.max(30.0);
        }
        if v >= 25.0 {
            out.push((v, c));
        }
    }
    out
}

/// ais.mirror_imprint_options
fn mirror_imprint_options(g: &mut Game, p: PlayerId) -> Vec<Opt> {
    let mut out = vec![];
    for m in find(g, p, Tag::Panoptic) {
        let name = g.perm(m).name;
        if g.perm(m).tapped || blocked(g, p, name) {
            continue;
        }
        for (v, c) in mirror_candidates(g, p) {
            if !can_pay(g, p, g.db.get(c).cmc, "", false) {
                continue;
            }
            out.push(Opt {
                utility: 1.5 + v / 12.0,
                label: format!("Panoptic Mirror imprints {}", g.db.get(c).name),
                act: Some(Action::Plan { f: mirror_imprint, arg: ((c.0 as i64) << 32) | m.0 as i64 }),
            });
        }
    }
    out
}

fn mirror_imprint(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (m, c) = (PermId(arg as u32), CardId((arg >> 32) as u16));
    let cmc = g.db.get(c).cmc;
    if g.perm(m).tapped
        || !g.perm(m).on_bf
        || g.perm(m).owner != p
        || !g.player(p).hand.contains(&c)
        || !can_pay(g, p, cmc, "", false)
    {
        return Ok(false);
    }
    pay(g, p, cmc, "", false)?;
    g.perm_mut(m).tapped = true;
    let pl = g.player_mut(p);
    remove_card(&mut pl.hand, c);
    pl.exile.push(c);
    g.imprint.push((m, c));
    crate::glog!(g, "  {} imprints {} on Panoptic Mirror", g.player(p).name, g.db.get(c).name);
    g.player_mut(p).stat("mirror_imprints", 1);
    Ok(true)
}

/// ais.mirror_upkeep: at the beginning of your upkeep, you may copy a card exiled with Panoptic Mirror and cast the
/// copy free
pub fn mirror_upkeep(g: &mut Game, p: PlayerId) -> Res {
    for m in find(g, p, Tag::Panoptic) {
        let cs: Vec<CardId> =
            g.imprint.iter().filter(|x| x.0 == m && g.player(p).exile.contains(&x.1)).map(|x| x.1).collect();
        if cs.is_empty() || g.perm(m).phased {
            continue;
        }
        let (mut best, mut bv, mut bctx) = (None, 0.0, Ctx::default());
        for c in cs {
            let mut ctx = Ctx::default();
            let mut v = card_worth(g, p, c, false);
            let t = g.db.get(c).tags.clone();
            if t.has(Tag::Rem) {
                let Some(tg) = best_target(g, p, c) else { continue };
                ctx.target = Some(tg);
                v = 10.0 * pval(g, tg);
                if pval(g, tg) < 2.0 {
                    continue;
                }
            }
            if let Some(kind) = t.str(Tag::Wipe) {
                let (ol, ml, victim) = wipe_eval(g, p, kind);
                if ol - 1.2 * ml < 6.0 {
                    continue;
                }
                v = 10.0 * (ol - 1.2 * ml);
                ctx.victim = victim;
            }
            if v > bv {
                (best, bv, bctx) = (Some(c), v, ctx);
            }
        }
        if let Some(c) = best {
            g.player_mut(p).stat("mirror_copies", 1);
            cast_spell_copy(g, p, c, Some(&bctx))?;
            if g.over {
                return Ok(());
            }
        }
    }
    Ok(())
}

/// ais.led_options: Lion's Eye Diamond: discard your hand, sacrifice: three mana of one colour. Only worth it with an
/// empty hand and something to spend the mana on (commander, flashback, escape).
fn led_options(g: &mut Game, p: PlayerId) -> Vec<Opt> {
    let mut leds = vec![];
    for m in find(g, p, Tag::Led) {
        let name = g.perm(m).name;
        if !blocked(g, p, name) {
            leds.push(m);
        }
    }
    if leds.is_empty() || g.player(p).hand.iter().any(|&c| !g.db.get(c).land) {
        return vec![];
    }
    let tot = total_mana(g, p, false);
    let avail = tot + 3;
    let mut uses: Vec<f64> = vec![];
    let pl = g.player(p);
    if pl.cmd_in_zone {
        let (cg, cp) = cost_of(g, p, pl.cmd);
        let need = cg + cp.chars().count() as u32;
        if need > tot && need <= avail {
            uses.push(8.0);
        }
    }
    for &c in &pl.gy {
        if let Some(fb) = g.db.get(c).tags.str(Tag::Fb) {
            let (fg, fp) = parse_cost(fb);
            let need = fg + fp.chars().count() as u32;
            if need > tot && need <= avail {
                uses.push(3.0);
            }
        }
    }
    if has(g, p, Tag::Breach) && pl.gy.len() >= 4 {
        uses.push(4.0);
    }
    if uses.is_empty() {
        return vec![];
    }
    let blue = pl.cmd_in_zone && g.db.get(pl.cmd).pips.contains('U') && tot < 3;
    let best = uses.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    vec![Opt {
        utility: best - 1.0,
        label: "Lion's Eye Diamond".into(),
        act: Some(Action::Plan { f: led_go, arg: ((blue as i64) << 32) | leds[0].0 as i64 }),
    }]
}

fn led_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (m, blue) = (PermId(arg as u32), (arg >> 32) != 0);
    if !g.perm(m).on_bf || g.perm(m).owner != p || g.player(p).hand.iter().any(|&c| !g.db.get(c).land) {
        return Ok(false);
    }
    let hand = g.player(p).hand.clone();
    discard_cards(g, p, &hand)?;
    leave(g, m)?;
    let cd = g.perm(m).cd.unwrap();
    let pl = g.player_mut(p);
    pl.gy.push(cd);
    if blue {
        pl.floating.u += 3;
    } else {
        pl.floating.r += 3;
    }
    crate::glog!(g, "  {} cracks Lion's Eye Diamond for {}", g.player(p).name, if blue { "UUU" } else { "RRR" });
    g.player_mut(p).stat("led_used", 1);
    Ok(true)
}

/// ais.citadel_options: Bolas's Citadel: play lands and cast spells from the top of your library, paying life equal
/// to mana value; {T}, sacrifice ten nonland permanents: each opponent loses 10 life
fn citadel_options(g: &mut Game, p: PlayerId) -> Vec<Opt> {
    let mut out = vec![];
    let cits: Vec<PermId> = find(g, p, Tag::Citadel).into_iter().filter(|&m| !g.perm(m).phased).collect();
    if cits.is_empty() || g.player(p).library.is_empty() {
        return out;
    }
    let top = *g.player(p).library.last().unwrap();
    let d = g.db.get(top);
    let t = &d.tags;
    let floor = crate::ai::plans::citadel_floor(g, p);
    if d.land {
        if g.player(p).land_turn != g.player(p).turns as i32 {
            out.push(Opt {
                utility: 3.0,
                label: format!("Citadel: play {}", d.name),
                act: Some(Action::Plan { f: citadel_land, arg: top.0 as i64 }),
            });
        }
    } else if !g.hooks.is_empty() && !castable(g, p, top, "lib") {
        // Drannith Magistrate, Rule of Law ...
    } else if g.player(p).life - d.cmc as i32 >= floor
        && !t.has(Tag::Ctr)
        && !t.has(Tag::X)
        && !t.has(Tag::Tokx)
        && !(SPECIALX.iter().any(|&k| t.has(k)) && !d.has_dsl())
    {
        let mut u = card_utility(g, p, &Situation::new(g, p), top);
        if t.has(Tag::Rem) {
            u = best_target(g, p, top).map(|m| pval(g, m) - 3.0);
        }
        if let Some(u) = u {
            out.push(Opt {
                utility: u + 0.6,
                label: format!("Citadel: cast {}", d.name),
                act: Some(Action::Plan { f: citadel_cast, arg: top.0 as i64 }),
            });
        }
    }
    let fodder = g.player(p).perms.iter().filter(|&&m| !g.perm(m).phased).count();
    let cit = cits[0];
    if !g.perm(cit).tapped && fodder >= 10 {
        let name = g.perm(cit).name;
        if !blocked(g, p, name) {
            let kills = g.opps(p).filter(|&q| g.player(q).life <= 10).count();
            out.push(Opt {
                utility: 6.0 * kills as f64 + if kills > 0 { 1.0 } else { -2.0 },
                label: "Citadel: sacrifice ten".into(),
                act: Some(Action::Plan { f: citadel_boom, arg: cit.0 as i64 }),
            });
        }
    }
    out
}

fn citadel_land(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let top = CardId(arg as u16);
    if g.player(p).library.last() != Some(&top) {
        return Ok(false);
    }
    g.player_mut(p).library.pop();
    let tapped = crate::engine::turn::land_enters_tapped(g, p, top);
    g.add_land(p, top, tapped);
    let turns = g.player(p).turns as i32;
    g.player_mut(p).land_turn = turns;
    crate::glog!(g, "  {} plays {} from the top (Citadel)", g.player(p).name, g.db.get(top).name);
    landfall(g, p)?;
    Ok(true)
}

fn citadel_cast(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let top = CardId(arg as u16);
    let cmc = g.db.get(top).cmc as i32;
    if g.player(p).library.last() != Some(&top) || g.player(p).life - cmc < crate::ai::plans::citadel_floor(g, p) {
        return Ok(false);
    }
    let mut ctx = Ctx::default();
    let t = g.db.get(top).tags.clone();
    if t.has(Tag::Rem) {
        let Some(m) = best_target(g, p, top) else { return Ok(false) };
        ctx.target = Some(m);
    }
    if let Some(kind) = t.str(Tag::Wipe) {
        ctx.victim = wipe_eval(g, p, kind).2;
    }
    g.player_mut(p).library.pop();
    lose_life(g, p, cmc, Some(p), "other", None)?;
    g.player_mut(p).stat("citadel_casts", 1);
    crate::glog!(g, "  {} casts {} from the top for {cmc} life (Citadel)", g.player(p).name, g.db.get(top).name);
    cast_card(g, p, top, "lib", ctx)?;
    Ok(true)
}

fn citadel_boom(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let cit = PermId(arg as u32);
    let mut fod: Vec<(f64, PermId)> =
        g.player(p).perms.iter().copied().filter(|&m| !g.perm(m).phased).map(|m| (pval(g, m), m)).collect();
    fod.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    fod.truncate(10);
    if fod.len() < 10 || !g.perm(cit).on_bf || g.perm(cit).owner != p {
        return Ok(false);
    }
    g.perm_mut(cit).tapped = true;
    for (_, m) in fod {
        die(g, m, "sac")?;
    }
    for q in g.opps(p).collect::<Vec<_>>() {
        lose_life(g, q, 10, Some(p), "other", None)?;
    }
    crate::glog!(g, "  {} sacrifices ten permanents to Bolas's Citadel", g.player(p).name);
    check_state(g)?;
    Ok(true)
}

/// ais.breach_gc_options: Breach escapes, Mirror imprints and LED as AI options (and Bolas's Citadel)
pub fn breach_gc_options(g: &mut Game, p: PlayerId) -> Res<Vec<Opt>> {
    let mut out = vec![];
    if has(g, p, Tag::Breach) {
        for (v, c) in breach_candidates(g, p, true) {
            out.push(Opt {
                utility: v / 10.0 - 0.5,
                label: format!("escape {}", g.db.get(c).name),
                act: Some(Action::Plan { f: breach_escape, arg: c.0 as i64 }),
            });
        }
    }
    out.extend(mirror_imprint_options(g, p));
    out.extend(led_options(g, p));
    out.extend(citadel_options(g, p));
    Ok(out)
}

// ======================================================== Sephiroth
// ------------------------------------------------------------------ Displacer Kitten
/// mine.ETB_VALUE: what entering again is worth, by the card's tags (in Python's dict order)
pub(crate) const ETB_VALUE: [(Tag, f64); 9] = [
    (Tag::Atraxa, 8.0),
    (Tag::Archon, 6.0),
    (Tag::Rsd, 4.0),
    (Tag::Titan, 3.0),
    (Tag::Witness, 2.5),
    (Tag::Wall, 2.0),
    (Tag::Wurm, 4.0),
    (Tag::Bowmasters, 2.5),
    (Tag::Skate, 3.0),
];

/// a permanent of p's on the battlefield (Python's `m in p.perms`)
fn on_bf_of(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.perm(m).on_bf && g.perm(m).owner == p
}

/// mine.etb_value: what re-entering the battlefield is worth for permanent m (its enters-the-battlefield effects)
pub fn etb_value(g: &Game, p: PlayerId, m: PermId) -> f64 {
    let x = g.perm(m);
    let Some(cd) = x.cd else { return 0.0 };
    let d = g.db.get(cd);
    let t = &d.tags;
    let mut v = ETB_VALUE.iter().filter(|(k, _)| t.has(*k)).map(|x| x.1).psum();
    if t.has(Tag::Draw) && d.perm {
        v += 1.5 * t.int_or_0(Tag::Draw) as f64;
    }
    if g.registry.get(cd).is_some_and(|i| i.etb.is_some()) {
        v = v.max(2.0);
    }
    if t.has(Tag::Bahamut) {
        v = bahamut_restart(g, p, m); // Summon: Bahamut starts over at chapter I
    }
    if let (Some(start), Some(l)) = (d.start_loyalty, x.loyalty) {
        v = v.max(0.6 * (start - l) as f64);
    }
    v -= 0.5 * x.plus.max(0) as f64; // counters are lost
    if !g.auras.is_empty() && g.is_creature(m) {
        v -= 2.0 * crate::impls::common::auras_on(g, m).len() as f64;
    }
    v
}

/// each noncreature spell you cast flickers your permanent with the best enters-the-battlefield value (Atraxa, Grand
/// Unifier, Eternal Witness ...); stolen permanents are left alone. HUMAN(phase 9): a person picks (up to one).
fn kitten(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let p = g.perm(src).owner;
    let d = g.db.get(c);
    if caster != p || d.creature || d.land || !on_bf_of(g, p, src) || g.perm(src).phased {
        return Ok(());
    }
    let cands = |g: &Game| -> Vec<PermId> {
        g.player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| {
                let x = g.perm(m);
                m != src && !x.token && !x.phased && x.cd.is_some() && x.orig == p
            })
            .collect()
    };
    let cs = cands(g);
    if cs.is_empty() {
        return Ok(());
    }
    let best = first_max(&cs, |m| etb_value(g, p, m)).unwrap();
    if etb_value(g, p, best) < 2.0 {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "flicker a permanent", None)? {
        return Ok(());
    }
    let cs: Vec<PermId> = cs.into_iter().filter(|&m| on_bf_of(g, p, m)).collect();
    let Some(m) = first_max(&cs, |m| etb_value(g, p, m)) else { return Ok(()) };
    if etb_value(g, p, m) < 2.0 {
        return Ok(());
    }
    let (cd, was_cmd) = (g.perm(m).cd.unwrap(), g.perm(m).is_cmd);
    crate::glog!(g, "    Displacer Kitten flickers {}", g.perm(m).name);
    leave(g, m)?;
    let n = enter(g, p, cd, Enter { orig: Some(p), ..Enter::default() })?;
    g.perm_mut(n).is_cmd = was_cmd;
    if !g.hooks.is_empty() && g.is_creature(m) {
        fire_trigger(g, Event::ExiledFromBf, Call::Leaves { m })?; // (Soulherder)
    }
    Ok(())
}

// ------------------------------------------------------------------ Nim Deathmantle
/// Marchesa's returns due at the next end step include card cd (Python's `g.marchesa_due`, set by marchesa.py's card
/// code, impls/marchesa.rs)
fn marchesa_returns(g: &Game, cd: CardId) -> bool {
    g.marchesa_due.iter().any(|&(_, c, _)| c == cd)
}

/// whenever a nontoken creature is put into your graveyard from the battlefield, you may pay {4}: return it and
/// attach this Equipment to it (when it's worth it). HUMAN(phase 9): a person decides to pay.
fn nim_return(g: &mut Game, src: Src, m: PermId) -> Res {
    let p = g.perm(src).owner;
    let x = g.perm(m);
    let Some(cd) = x.cd else { return Ok(()) };
    if !on_bf_of(g, p, src) || g.perm(src).phased || x.token || x.orig != p || !g.player(p).gy.contains(&cd) {
        return Ok(());
    }
    if cd == g.player(p).cmd {
        return Ok(());
    }
    if !can_pay(g, p, 4, "", false) || (pval(g, m) < 3.0 && etb_value(g, p, m) < 2.5) || marchesa_returns(g, cd) {
        return Ok(());
    }
    let name = g.perm(m).name;
    if !trigger_window(g, p, Some(src), &format!("pay {{4}}: return {name}"), None)? {
        return Ok(());
    }
    if cd == g.player(p).cmd || !can_pay(g, p, 4, "", false) {
        return Ok(());
    }
    if marchesa_returns(g, cd) {
        return Ok(()); // Marchesa returns it free
    }
    if pval(g, m) < 3.0 && etb_value(g, p, m) < 2.5 {
        return Ok(());
    }
    pay(g, p, 4, "", false)?;
    if !g.player(p).gy.contains(&cd) {
        // left the graveyard while paying (a Treasure's sacrifice set off a sweep): no return
        crate::glog!(g, "    {} pays 4, but {name} has left the graveyard", g.player(p).name);
        return Ok(());
    }
    remove_card(&mut g.player_mut(p).gy, cd);
    let n = enter(g, p, cd, Enter { orig: Some(p), ..Enter::default() })?;
    g.perm_mut(src).attached = Some(n);
    crate::glog!(g, "    {} pays 4: Nim Deathmantle returns {}", g.player(p).name, g.perm(n).name);
    Ok(())
}

/// equip {4}: onto your best creature (a bomb first)
fn nim_equip(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != g.perm(src).owner || post.is_none() || !can_pay(g, p, 4, "", false) {
        return Ok(vec![]);
    }
    if let Some(a) = g.perm(src).attached
        && on_bf_of(g, p, a)
        && !g.perm(a).phased
    {
        return Ok(vec![]);
    }
    let mikaeus = has(g, p, Tag::Mikaeus);
    let cre: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            g.is_creature(m) && !x.phased && x.cd.is_some() && !(card_name(g, m) == Some("Triskelion") && mikaeus) // +2/+2 would stop its loop needing no outlet
        })
        .collect();
    let Some(t) = first_max(&cre, |m| pval(g, m)) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: 1.0 + 0.2 * pval(g, t),
        label: "equip Nim Deathmantle".into(),
        act: Some(Action::Ability { src, f: nim_equip_go, arg: t.0 as i64 }),
    }])
}

fn nim_equip_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    if !can_pay(g, p, 4, "", false) || !on_bf_of(g, p, t) {
        return Ok(false);
    }
    crate::glog!(g, "  {} equips Nim Deathmantle to {}", g.player(p).name, g.perm(t).name);
    equip_to(g, p, src, t, 4)
}

// ------------------------------------------------------------------ Triskelion
/// enters with three +1/+1 counters
fn trisk_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.perm_mut(src).plus += 3;
    }
    Ok(())
}

/// remove a +1/+1 counter: 1 damage to any target (lethal players, X/1 creatures, or its own last counters when Nim
/// Deathmantle can bring it back)
fn trisk_ping(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let plus = g.perm(src).plus;
    if p != g.perm(src).owner || plus <= 0 || post.is_none() {
        return Ok(vec![]);
    }
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let lethal = opps.iter().copied().find(|&q| g.player(q).life <= plus);
    let tg = opps
        .iter()
        .flat_map(|&q| g.player(q).perms.iter().copied())
        .find(|&m| g.is_creature(m) && etgh(g, m) <= 1 && pval(g, m) >= 2.5 && !untargetable(g, m));
    if lethal.is_none() && tg.is_none() {
        return Ok(vec![]);
    }
    let arg = (lethal.map_or(0, |q| q.0 as i64 + 1) << 40) | tg.map_or(0, |m| m.0 as i64 + 1);
    Ok(vec![Opt {
        utility: if lethal.is_some() { 8.0 } else { 2.0 },
        label: "Triskelion ping".into(),
        act: Some(Action::Ability { src, f: trisk_ping_go, arg }),
    }])
}

fn trisk_ping_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let lethal = match arg >> 40 {
        0 => None,
        q => Some(PlayerId((q - 1) as u8)),
    };
    let tg = match arg & 0xff_ffff_ffff {
        0 => None,
        m => Some(PermId((m - 1) as u32)),
    };
    if g.perm(src).plus <= 0 || !on_bf_of(g, p, src) {
        return Ok(false);
    }
    g.perm_mut(src).plus -= 1;
    let imp = if lethal.is_some() { 8.0 } else { 3.0 };
    if !ability_window(g, p, Some(src), "1 damage", Some(imp), None)? {
        return Ok(true);
    }
    if let Some(q) = lethal.filter(|&q| g.player(q).alive) {
        lose_life(g, q, 1, Some(p), "triggers", None)?;
    } else if let Some(m) = tg.filter(|&m| g.perm(m).on_bf) {
        apply_removal(g, Some(p), m, "dmg1", None)?;
    }
    if etgh(g, src) <= 0 {
        die(g, src, "sba")?;
    }
    Ok(true)
}

// ------------------------------------------------------------------ Strip Mine
/// the key lands Strip Mine goes for first
const STRIP_FIRST: [&str; 6] = [
    "Gaea's Cradle",
    "Cabal Coffers",
    "Serra's Sanctum",
    "Tolarian Academy",
    "Itlimoc, Cradle of the Sun",
    "Nykthos, Shrine to Nyx",
];

/// {T}, sacrifice: destroy target land (an opponent's key land: Cradle, Coffers, Urborg, Tomb ...)
fn strip(g: &mut Game, l: LandId, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if g.land(l).tapped {
        return Ok(vec![]);
    }
    let mut tg: Vec<(PlayerId, LandId)> = vec![];
    for q in g.opps(p) {
        for &x in &g.player(q).lands {
            if crate::impls::lands::KEY_LANDS.contains(&&*g.db.get(g.land(x).cd).name) {
                tg.push((q, x));
            }
        }
    }
    let Some((_, x)) = first_max(&tg, |(q, x)| (STRIP_FIRST.contains(&&*g.db.get(g.land(x).cd).name), threat(g, p, q)))
    else {
        return Ok(vec![]);
    };
    Ok(vec![Opt {
        utility: 3.0,
        label: format!("Strip Mine -> {}", g.db.get(g.land(x).cd).name),
        act: Some(Action::Plan { f: strip_go, arg: ((x.0 as i64) << 32) | l.0 as i64 }),
    }])
}

fn strip_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (l, x) = (LandId(arg as u32), LandId((arg >> 32) as u32));
    let q = g.land(x).owner;
    if !g.player(p).lands.contains(&l) || !g.player(q).lands.contains(&x) {
        return Ok(false);
    }
    crate::engine::turn::remove_land(g, p, l);
    let cd = g.land(l).cd;
    g.player_mut(p).gy.push(cd);
    let xn = g.db.get(g.land(x).cd).name.to_string();
    crate::glog!(g, "  {} sacrifices Strip Mine: destroy {xn} ({})", g.player(p).name, g.player(q).name);
    if ability_window_card(g, p, cd, &format!("destroy {xn}"), None, None)? && g.player(q).lands.contains(&x) {
        crate::impls::fixes::destroy_land(g, q, x)?;
    }
    Ok(true)
}

// ------------------------------------------------------------------ Necromancy: the flash mode
/// mine.card_etb_value: enters-the-battlefield value of a creature card (for a flashed-in Necromancy, which keeps
/// nothing else)
pub fn card_etb_value(g: &Game, c: CardId) -> f64 {
    let t = &g.db.get(c).tags;
    let mut v = ETB_VALUE.iter().filter(|(k, _)| t.has(*k)).map(|x| x.1).psum();
    if t.has(Tag::Draw) {
        v += 1.5 * t.int_or_0(Tag::Draw) as f64;
    }
    v
}

/// reanimates at sorcery speed (the aura is abstracted: the creature stays); flashed in at the end of an opponent's
/// turn for an enters-the-battlefield creature (Atraxa, Grand Unifier), which is sacrificed at cleanup and goes back
/// to the graveyard
fn necro_flash(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some()
        || g.player(p).key != "seph"
        || g.active == Some(p)
        || !g.player(p).hand.contains(&c)
        || !can_pay(g, p, 2, "B", false)
    {
        return Ok(vec![]);
    }
    let mut tg: Vec<(f64, CardId, PlayerId)> = vec![];
    for q in g.players.iter().filter(|q| q.alive) {
        for &x in &q.gy {
            if g.db.get(x).creature {
                tg.push((card_etb_value(g, x), x, q.id));
            }
        }
    }
    tg.retain(|t| t.0 >= 4.0);
    let Some((val, cd, src)) = first_max(&tg, |t| t.0) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: 1.5 + 0.5 * val,
        label: format!("Necromancy (flash) -> {}", g.db.get(cd).name),
        act: Some(Action::Plan { f: necro_flash_go, arg: c.0 as i64 | (cd.0 as i64) << 16 | (src.0 as i64) << 32 }),
    }])
}

fn necro_flash_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, cd, src) = (CardId(arg as u16), CardId((arg >> 16) as u16), PlayerId((arg >> 32) as u8));
    if !g.player(p).hand.contains(&c) || !g.player(src).gy.contains(&cd) || !can_pay(g, p, 2, "B", false) {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    pay(g, p, 2, "B", false)?;
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.cast_names.insert(c);
    on_cast(g, p, c)?;
    crate::glog!(g, "  {} flashes in Necromancy for {}", g.player(p).name, g.db.get(cd).name);
    if !counter_window(g, p, c, 6.0, vec![])? || (!g.hooks.is_empty() && crate::cardcode::gy_response(g, p, 6.0, src)?)
    {
        g.player_mut(p).gy.push(c);
        return Ok(true);
    }
    let n0 = g.player(p).perms.len();
    crate::ai::seph_rean_resolve(g, p, c, &Ctx { rean_target: Some(cd), rean_src: Some(src), ..Ctx::default() })?;
    let new: Vec<PermId> = g.player(p).perms.iter().skip(n0).copied().filter(|&m| g.perm(m).cd == Some(cd)).collect();
    for m in new {
        die(g, m, "sac")?; // cast when a sorcery couldn't be: sacrificed at cleanup
    }
    g.player_mut(p).gy.push(c);
    check_state(g)?;
    Ok(true)
}

// ------------------------------------------------------------------ Summon: Bahamut (a Saga creature)
pub const BAHAMUT: &str = "Summon: Bahamut";

fn bahamut_ch(n: i64) -> &'static str {
    match n {
        1 => "chapter I: destroy a nonland permanent",
        2 => "chapter II: destroy a nonland permanent",
        3 => "chapter III: draw two cards",
        _ => "chapter IV: Mega Flare",
    }
}

/// mine.bahamut_target: chapters I and II: the most threatening opposing nonland permanent it can destroy
pub fn bahamut_target(g: &Game, p: PlayerId) -> Option<PermId> {
    let tg = legal_targets(g, p, "destroy", "nl", false, None);
    first_max(&tg, |m| (pval(g, m), threat(g, p, g.perm(m).owner)))
}

/// mine.mega_flare: chapter IV's damage to each opponent: the total mana value of the other permanents p controls
pub fn mega_flare(g: &Game, p: PlayerId, src: PermId) -> i32 {
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| m != src && !g.perm(m).phased)
        .filter_map(|&m| g.perm(m).cd)
        .map(|c| g.db.get(c).cmc as i32)
        .sum()
}

/// mine.flare_strong: Mega Flare would kill an opponent, or deal 10 or more to each
pub fn flare_strong(g: &Game, p: PlayerId, m: PermId) -> bool {
    let x = mega_flare(g, p, m);
    x >= 10 || g.opps(p).any(|q| g.player(q).life <= x)
}

fn lore(g: &Game, m: PermId) -> Option<i64> {
    g.perm(m).data.get(DataKey::Lore).map(Val::int)
}

/// mine.bahamut_restart: blinking it: a new object at chapter I (another destroy now, II and III again) that isn't
/// sacrificed after IV. Not while a strong Mega Flare is next
pub fn bahamut_restart(g: &Game, p: PlayerId, m: PermId) -> f64 {
    let lo = lore(g, m).unwrap_or(1);
    if lo >= 3 && flare_strong(g, p, m) {
        return 0.0;
    }
    let t = bahamut_target(g, p);
    1.0 + t.map_or(0.0, |t| 5.0f64.min(pval(g, t))) + (lo - 1) as f64
}

/// mine._bahamut_chapter. HUMAN(phase 9): a person picks the chapter I/II target (up to one).
fn bahamut_chapter(g: &mut Game, p: PlayerId, src: PermId, n: i64) -> Res {
    g.player_mut(p).stat("bahamut_chapters", 1);
    if n <= 2 {
        if let Some(t) = bahamut_target(g, p) {
            apply_removal(g, Some(p), t, "destroy", None)?;
        }
    } else if n == 3 {
        draw(g, p, 2, false)?;
    } else {
        let x = mega_flare(g, p, src);
        crate::glog!(g, "    Mega Flare: {x} damage to each opponent");
        let alive: Vec<PlayerId> = g.opps(p).collect();
        for &q in &alive {
            lose_life(g, q, x, Some(p), "triggers", None)?;
        }
        let pl = g.player_mut(p);
        pl.stat("bahamut_flare", 1);
        pl.stat("bahamut_flare_dmg", x as i64);
        check_state(g)?;
        let kills = alive.iter().filter(|&&q| !g.player(q).alive).count() as i64;
        g.player_mut(p).stat("bahamut_flare_kills", kills);
    }
    Ok(())
}

/// it enters with a lore counter (AS_ENTERS), so chapter I triggers: cast, reanimated or blinked alike
fn bahamut_enters(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    g.player_mut(o).stat("bahamut_enters", 1);
    if !trigger_window(g, o, Some(src), bahamut_ch(1), Some(5.0))? {
        return Ok(());
    }
    bahamut_chapter(g, o, src, 1)
}

/// after your draw step: a lore counter
fn bahamut_main1(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == g.perm(src).owner && lore(g, src).is_some() {
        bahamut_lore(g, p, src)?;
    }
    Ok(())
}

/// mine.bahamut_lore: a lore counter (after your draw step, or proliferated): that chapter triggers; sacrificed once
/// IV has resolved
pub fn bahamut_lore(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    let n = lore(g, src).unwrap_or(0) + 1;
    g.perm_mut(src).data.set(DataKey::Lore, Val::Int(n));
    if n <= 4 && trigger_window(g, p, Some(src), bahamut_ch(n), Some(if n == 4 { 7.0 } else { 5.0 }))? {
        bahamut_chapter(g, p, src, n)?;
    }
    if n >= 4 && on_bf_of(g, p, src) {
        die(g, src, "sac")?;
    }
    Ok(())
}

/// mine.bahamut_wants_lore (CI.SAGA's first function): proliferate adds a lore counter toward chapter II or III, or
/// to IV when Mega Flare is strong
pub fn bahamut_wants_lore(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.perm(m).owner == p && (lore(g, m).unwrap_or(0) < 3 || flare_strong(g, p, m))
}

/// mine._bahamut_proliferated (CI.SAGA's second function): proliferate added a lore counter
pub fn bahamut_proliferated(g: &mut Game, p: PlayerId, m: PermId) -> Res {
    g.player_mut(p).stat("bahamut_proliferated", 1);
    bahamut_lore(g, p, m)
}

/// CI.AS_ENTERS: a lore counter as it enters
fn bahamut_as_enters(g: &mut Game, _p: PlayerId, m: PermId) -> Res {
    g.perm_mut(m).data.set(DataKey::Lore, Val::Int(1));
    Ok(())
}

/// CI.SAGA: a Saga creature with card code (Summon: Bahamut): does it want a proliferated lore counter? None: m is
/// no such Saga (or has no lore counter)
pub fn saga_wants_lore(g: &Game, p: PlayerId, m: PermId) -> Option<bool> {
    if card_name(g, m) != Some(BAHAMUT) || lore(g, m).is_none() {
        return None;
    }
    Some(bahamut_wants_lore(g, p, m))
}

// ======================================================== Veyran
/// mine.spell_copy_value: what one more copy of spell c is worth (Ral's -2, Return the Favor)
pub fn spell_copy_value(g: &Game, p: PlayerId, c: CardId) -> f64 {
    let t = &g.db.get(c).tags;
    let mut v = 1.5 * t.int_or_0(Tag::Draw) as f64
        + if t.str(Tag::Tut).is_some_and(|s| !s.is_empty()) { 2.0 } else { 0.0 }
        + if t.has(Tag::Burn) { 4.0 } else { 0.0 };
    if let Some(rk) = t.str(Tag::Rem) {
        let tg = legal_targets(g, p, rk, t.str(Tag::Tgt).unwrap_or("c"), t.has(Tag::Mv4), Some(c));
        v += tg.iter().filter(|&&m| g.perm(m).owner != p).map(|&m| pval(g, m)).fold(0.0, f64::max);
    }
    if t.has(Tag::Mastery) {
        v += 5.0;
    }
    if t.has(Tag::Jeska) {
        v += 4.0;
    }
    const PAY: [Tag; 4] = [Tag::Ping, Tag::Spelltok, Tag::Spelldraw, Tag::Kiln];
    let n =
        g.player(p).perms.iter().filter(|&&m| g.perm(m).cd.is_some_and(|x| PAY.iter().any(|&k| g.db.get(x).tag(k))));
    v + 1.0 * n.count() as f64
}

// ------------------------------------------------------------------ Muldrotha, the Gravetide
/// mine.MULD_TYPES (but the land): the permanent types Muldrotha lets you cast from the graveyard, in order
const MULD_TYPES: [(&str, Types); 4] =
    [("C", Types::CREATURE), ("A", Types::ARTIFACT), ("E", Types::ENCHANTMENT), ("P", Types::PLANESWALKER)];

/// mine.muldrotha_on: during each of your turns, a land and a permanent spell of each permanent type from your
/// graveyard
pub fn muldrotha_on(g: &Game, p: PlayerId) -> bool {
    g.active == Some(p)
        && g.player(p).perms.iter().any(|&m| {
            let x = g.perm(m);
            card_name(g, m) == Some("Muldrotha, the Gravetide") && !x.phased && !x.neutered
        })
}

/// mine.muld_used: the permanent types used this turn
pub fn muld_used(g: &Game, p: PlayerId) -> Vec<Sym> {
    match &g.player(p).muld_used {
        Some((st, v)) if *st == g.turn_stamp() => v.clone(),
        _ => vec![],
    }
}

/// mine.muld_types: the permanent types of graveyard card c that Muldrotha still lets p play this turn
pub fn muld_types(g: &Game, p: PlayerId, c: CardId) -> Vec<Sym> {
    if !muldrotha_on(g, p) || !g.player(p).gy.contains(&c) {
        return vec![];
    }
    let used = muld_used(g, p);
    let d = g.db.get(c);
    if d.land {
        return if used.contains(&"L") { vec![] } else { vec!["L"] };
    }
    if !d.perm {
        return vec![];
    }
    MULD_TYPES.iter().filter(|(t, ty)| d.types.has(*ty) && !used.contains(t)).map(|x| x.0).collect()
}

/// mine.muld_mark: type t was played this turn
pub fn muld_mark(g: &mut Game, p: PlayerId, t: Sym) {
    let st = g.turn_stamp();
    let pl = g.player_mut(p);
    if !matches!(&pl.muld_used, Some((s, _)) if *s == st) {
        pl.muld_used = Some((st, vec![]));
    }
    let v = &mut pl.muld_used.as_mut().unwrap().1;
    if !v.contains(&t) {
        v.push(t);
    }
}

/// the AI: cast the best permanent card from the graveyard whose type is still unused this turn
fn muldrotha(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != g.perm(src).owner || post.is_none() || !muldrotha_on(g, p) {
        return Ok(vec![]);
    }
    let mut out = vec![];
    for c in g.player(p).gy.clone() {
        let ts = muld_types(g, p, c);
        if ts.is_empty() || g.db.get(c).land || !castable(g, p, c, "gy") {
            continue;
        }
        let (gn, pips) = cost_of(g, p, c);
        if !can_pay(g, p, gn, &pips, false) {
            continue;
        }
        let v = crate::ai::decks::deck_prio(g, p, c) as f64 / 10.0 + 0.3 * card_worth(g, p, c, true) / 10.0;
        if v <= 0.5 {
            continue;
        }
        let ti = MULD_TYPES.iter().position(|x| x.0 == ts[0]).unwrap();
        out.push(Opt {
            utility: v,
            label: format!("{} from the graveyard (Muldrotha)", g.db.get(c).name),
            act: Some(Action::Plan { f: muldrotha_go, arg: c.0 as i64 | (ti as i64) << 32 }),
        });
    }
    Ok(out)
}

fn muldrotha_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, t) = (CardId(arg as u16), MULD_TYPES[(arg >> 32) as usize].0);
    let (gn, pips) = cost_of(g, p, c);
    if !g.player(p).gy.contains(&c) || !muld_types(g, p, c).contains(&t) || !can_pay(g, p, gn, &pips, false) {
        return Ok(false);
    }
    pay(g, p, gn, &pips, false)?;
    muld_mark(g, p, t);
    crate::glog!(g, "  {} casts {} from the graveyard (Muldrotha)", g.player(p).name, g.db.get(c).name);
    cast_card(g, p, c, "mgy", Ctx::default())?;
    Ok(true)
}

// ------------------------------------------------------------------ Thousand-Year Storm
/// mine._storm_n: copies: one per instant/sorcery cast before this one this turn; Veyran doubles the trigger
fn storm_n(g: &Game, p: PlayerId) -> i32 {
    (is_casts(g, p) - 1).max(0) * if has(g, p, Tag::Veyran) { 2 } else { 1 }
}

/// the spell being cast now's choices, if it is c (X is copied: Crackle with Power)
fn cast_ctx(g: &Game, c: CardId) -> Option<Ctx> {
    match &g.cur_cast {
        Some((cc, ctx, _)) if *cc == c => Some(ctx.clone()),
        _ => None,
    }
}

/// each instant/sorcery you cast (or cast as a copy) is copied once per instant/sorcery cast before it this turn,
/// twice with Veyran; copies pick new targets and keep X
fn storm(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let p = g.perm(src).owner;
    let d = g.db.get(c);
    if caster != p || !(d.instant || d.sorcery) || g.perm(src).phased || !on_bf_of(g, p, src) {
        return Ok(());
    }
    let n = storm_n(g, p);
    if n == 0 {
        return Ok(());
    }
    let ctx = cast_ctx(g, c);
    let name = g.db.get(c).name.to_string();
    if !trigger_window(g, p, Some(src), &format!("copy {name} {n} time(s)"), Some(6.0))? {
        return Ok(());
    }
    crate::glog!(g, "    Thousand-Year Storm copies {name} {n} time(s)");
    for _ in 0..n {
        crate::engine::cast::copy_spell(g, p, c, ctx.as_ref(), false)?;
        if g.over {
            return Ok(());
        }
    }
    Ok(())
}

/// a cast copy of a back-face spell (prepared, Lightning Bolt) is an instant/sorcery cast too
fn storm_copycast(g: &mut Game, src: Src, p: PlayerId, effect: u8) -> Res {
    if p != g.perm(src).owner || g.perm(src).phased || storm_n(g, p) == 0 {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "copy the spell", Some(6.0))? {
        return Ok(());
    }
    for _ in 0..storm_n(g, p) {
        magecraft(g, p, None, true)?;
        prepared_effect(g, p, effect)?;
        if g.over {
            return Ok(());
        }
    }
    Ok(())
}

// ------------------------------------------------------------------ Alania, Divergent Storm
/// mine.alania_triggers: Alania triggers once, +1 with Veyran (a cast instant or sorcery), +1 more with Harmonic
/// Prodigy (a Wizard)
fn alania_triggers(g: &Game, p: PlayerId) -> u32 {
    1 + has(g, p, Tag::Veyran) as u32 + has(g, p, Tag::Prodigy) as u32
}

/// mine.alania_pick: the AI: which opponent draws the card for a copy of c, or None to decline. A copied counterspell
/// does nothing here, so it isn't worth a card; otherwise the least threatening opponent gets it
fn alania_pick(g: &Game, p: PlayerId, c: CardId, opps: &[PlayerId]) -> Option<PlayerId> {
    if g.db.get(c).tag(Tag::Ctr) {
        return None;
    }
    min_by(opps, |q| threat(g, p, q))
}

/// engine.casts_this_turn with a predicate: p's spells this turn that match
fn casts_matching(g: &Game, p: PlayerId, pred: impl Fn(CardId) -> bool) -> usize {
    match &g.player(p).turn_casts {
        Some((st, cs)) if *st == g.turn_stamp() => cs.iter().filter(|&&c| pred(c)).count(),
        _ => 0,
    }
}

/// your first instant and your first sorcery each turn (any turn): for each trigger (Veyran and Harmonic Prodigy add
/// one each) you may have the least threatening opponent draw a card to copy the spell (not a counterspell); copies
/// pick new targets and keep X. The 'first this turn' check counts spells cast before Alania was out too. The Otter
/// clause is not modeled (no Otters in your decks). HUMAN(phase 9): a person picks the opponent or declines.
fn alania(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let p = g.perm(src).owner;
    let d = g.db.get(c);
    if caster != p || !(d.instant || d.sorcery) || g.perm(src).phased || !on_bf_of(g, p, src) {
        return Ok(());
    }
    let instant = d.instant;
    if casts_matching(g, p, |x| if instant { g.db.get(x).instant } else { g.db.get(x).sorcery }) != 1 {
        return Ok(());
    }
    let ctx = cast_ctx(g, c);
    let name = g.db.get(c).name.to_string();
    for _ in 0..alania_triggers(g, p) {
        let opps: Vec<PlayerId> = g.opps(p).collect();
        if opps.is_empty() || g.over {
            return Ok(());
        }
        if alania_pick(g, p, c, &opps).is_none() {
            return Ok(());
        }
        if !trigger_window(g, p, Some(src), &format!("copy {name}"), None)? {
            continue;
        }
        let opps: Vec<PlayerId> = g.opps(p).collect();
        if opps.is_empty() {
            return Ok(());
        }
        let Some(q) = alania_pick(g, p, c, &opps) else { return Ok(()) }; // the same answer for each trigger
        draw(g, q, 1, false)?;
        crate::glog!(g, "    Alania: {} draws a card, {} copies {name}", g.player(q).name, g.player(p).name);
        crate::engine::cast::copy_spell(g, p, c, ctx.as_ref(), false)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Ral, Storm Conduit: loyalty abilities
/// magecraft ping on each cast or copied instant/sorcery (doubled by Veyran); +2 scry 1; -2 copies your next
/// instant/sorcery (used with a spell worth copying in hand)
fn ral(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if p != x.owner || post.is_none() || x.phased || x.loyalty.is_none() || !pw_once(g, src) {
        return Ok(vec![]);
    }
    let loyalty = x.loyalty.unwrap();
    let best = g
        .player(p)
        .hand
        .iter()
        .filter(|&&c| {
            let d = g.db.get(c);
            let (gn, pips) = cost_of(g, p, c);
            (d.instant || d.sorcery) && can_pay(g, p, gn, &pips, false)
        })
        .map(|&c| spell_copy_value(g, p, c))
        .fold(0.0, f64::max);
    let mut out = vec![Opt {
        utility: 1.0,
        label: "Ral, Storm Conduit +2 (scry 1)".into(),
        act: Some(Action::Ability { src, f: ral_go, arg: 0 }),
    }];
    if loyalty >= 2 && best >= 3.0 {
        out.push(Opt {
            utility: 1.5 + 0.5 * best,
            label: "Ral, Storm Conduit -2 (copy)".into(),
            act: Some(Action::Ability { src, f: ral_go, arg: 1 }),
        });
    }
    Ok(out)
}

fn ral_go(g: &mut Game, src: PermId, p: PlayerId, minus: i64) -> Res<bool> {
    if !on_bf_of(g, p, src) || !pw_once(g, src) {
        return Ok(false);
    }
    let st = g.turn_stamp();
    g.perm_mut(src).data.set(DataKey::Act, Val::Stamp(st));
    let l = g.perm(src).loyalty.unwrap_or(0);
    if minus != 0 {
        g.perm_mut(src).loyalty = Some(l - 2);
        crate::glog!(g, "  {} uses Ral, Storm Conduit -2: the next instant or sorcery is copied", g.player(p).name);
        if l - 2 <= 0 {
            leave(g, src)?;
            to_zone_card(g, src, Zone::Gy);
        }
        let cd = g.perm(src).cd.unwrap();
        if ability_window_card(g, p, cd, "-2", None, None)? {
            g.player_mut(p).ral_copy = Some(g.turn_stamp());
        }
    } else {
        g.perm_mut(src).loyalty = Some(l + 2);
        if ability_window(g, p, Some(src), "+2", None, None)? {
            crate::cardcode::scry(g, p, 1, false)?;
        }
    }
    Ok(true)
}

// ------------------------------------------------------------------ prepared back faces: cast a copy, paying its cost
/// mine.PREPARED: (card, generic, pips, an instant, the back face)
const PREPARED: [(&str, u32, &str, bool, &str); 4] = [
    ("Blazing Firesinger // Seething Song", 2, "R", true, "Seething Song"),
    ("Emeritus of Ideation // Ancestral Recall", 0, "U", true, "Ancestral Recall"),
    ("Sanar, Unfinished Genius // Wild Idea", 3, "UR", false, "Wild Idea"),
    ("Emeritus of Conflict // Lightning Bolt", 0, "R", true, "Lightning Bolt"),
];
/// mine.PAYOFF_TAGS
const PAYOFF_TAGS: [Tag; 7] =
    [Tag::Ping, Tag::Spelltok, Tag::Spelldraw, Tag::Kiln, Tag::Dragoncaller, Tag::Mystic, Tag::Aether];

/// mine._prepared_effect, by the back face's index in PREPARED (Python's closures): Seething Song makes five red,
/// Ancestral Recall draws three, Wild Idea finds an instant or sorcery, Lightning Bolt burns
pub fn prepared_effect(g: &mut Game, p: PlayerId, effect: u8) -> Res {
    match effect {
        0 => {
            g.player_mut(p).floating.r += 5;
            Ok(())
        }
        1 => draw(g, p, 3, false),
        2 => crate::engine::tutors::tutor(g, p, "is"),
        _ => crate::engine::cast::bolt_something(g, p, 3),
    }
}

/// mine._prepared_value
fn prepared_value(g: &Game, p: PlayerId, i: usize) -> f64 {
    let base = 1.2
        * g.player(p)
            .perms
            .iter()
            .filter(|&&m| g.perm(m).cd.is_some_and(|c| PAYOFF_TAGS.iter().any(|&k| g.db.get(c).tag(k))))
            .count() as f64;
    match i {
        0 => {
            // Seething Song: {2}{R} for five red, worth it to reach a spell
            let tot = total_mana(g, p, false);
            let reach = g.player(p).hand.iter().any(|&c| {
                let (gn, pips) = cost_of(g, p, c);
                let need = gn + pips.chars().count() as u32;
                !g.db.get(c).land && tot < need && need <= tot + 2
            });
            base + if reach { 3.0 } else { 0.0 } - 0.5
        }
        1 => base + 5.0,
        2 => base + 3.0,
        _ => {
            let opps: Vec<PlayerId> = g.opps(p).collect();
            if opps.iter().any(|&q| g.player(q).life <= 3) {
                return 10.0;
            }
            let best = opps
                .iter()
                .flat_map(|&q| g.player(q).perms.iter().copied())
                .filter(|&m| g.is_creature(m) && etgh(g, m) <= 3 && !untargetable(g, m))
                .map(|m| pval(g, m))
                .fold(0.0, f64::max);
            base + 0.8 * best
        }
    }
}

fn prepared_index(g: &Game, src: PermId) -> Option<usize> {
    let n = card_name(g, src)?;
    PREPARED.iter().position(|x| x.0 == n)
}

/// mine._prepared_opt: cast a copy of the prepared back face, paying its cost
fn prepared_opt(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != g.perm(src).owner || g.perm(src).phased || !g.perm(src).data.truthy(DataKey::Prepared) {
        return Ok(vec![]);
    }
    let Some(i) = prepared_index(g, src) else { return Ok(vec![]) };
    let (_, gn, pips, instant, spell) = PREPARED[i];
    if post.is_none() && !instant && !has(g, p, Tag::Gandalf) {
        return Ok(vec![]); // Wild Idea is a sorcery
    }
    if !can_pay(g, p, gn, pips, false) {
        return Ok(vec![]);
    }
    let v = prepared_value(g, p, i);
    if v < 1.0 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: v,
        label: format!("{spell} (prepared copy)"),
        act: Some(Action::Ability { src, f: prepared_go, arg: i as i64 }),
    }])
}

fn prepared_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (_, gn, pips, instant, spell) = PREPARED[arg as usize];
    if !g.perm(src).data.truthy(DataKey::Prepared) || !can_pay(g, p, gn, pips, false) {
        return Ok(false);
    }
    pay(g, p, gn, pips, false)?;
    g.perm_mut(src).data.set(DataKey::Prepared, Val::Bool(false));
    cast_copy(g, p, Some(arg as u8), Some(spell), instant, 4.0)?;
    Ok(true)
}

/// engine.cast_copy: 'You may cast a copy of ...' (a back-face instant/sorcery): a real cast (magecraft, opponents'
/// cast triggers, Thousand-Year Storm), no card moves. Named, it goes on the stack, where it can be countered.
/// `effect` is the copy's effect (prepared_effect's number).
pub fn cast_copy(g: &mut Game, p: PlayerId, effect: Option<u8>, name: Option<&str>, _instant: bool, imp: f64) -> Res {
    if let Some(name) = name {
        crate::glog!(g, "  {} casts a copy of {name}", g.player(p).name);
        let c = g.db.id(name).expect("the back face is in the card data or made by the engine (cards::engine_made)");
        if !counter_window(g, p, c, imp, vec![])? {
            let pl = g.player_mut(p);
            pl.spells_this_turn += 1;
            pl.stat("spells_cast", 1);
            return Ok(());
        }
    }
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    crate::engine::cast::count_is_cast(g, p);
    magecraft(g, p, None, false)?;
    if !g.hooks.is_empty()
        && let Some(e) = effect
    {
        fire_trigger(g, Event::Copycast, Call::Copycast { p, effect: e })?; // Thousand-Year Storm
    }
    for q in g.opps(p).collect::<Vec<_>>() {
        if has(g, q, Tag::Sauron) {
            amass(g, q, 1)?;
        }
        if has(g, q, Tag::Rhystic) && crate::cardcode::rhystic_unpaid(g, p)? {
            draw(g, q, 1, false)?;
        }
    }
    if let Some(e) = effect {
        prepared_effect(g, p, e)?;
    }
    check_state(g)
}

/// whenever it attacks: exile eight cards from your graveyard to prepare it again. HUMAN(phase 9): a person picks
/// the eight (or declines).
fn ideation_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner != p
        || !atk.contains(&src)
        || g.perm(src).data.truthy(DataKey::Prepared)
        || g.player(p).gy.len() < 8
    {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "exile eight cards to prepare it", None)? {
        return Ok(vec![]);
    }
    if g.player(p).gy.len() < 8 {
        return Ok(vec![]);
    }
    let mut ex: Vec<((bool, f64), CardId)> = g
        .player(p)
        .gy
        .iter()
        .map(|&c| ((g.db.get(c).instant || g.db.get(c).sorcery, card_worth(g, p, c, true)), c))
        .collect();
    ex.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let pl = g.player_mut(p);
    for &(_, c) in ex.iter().take(8) {
        remove_card(&mut pl.gy, c);
        pl.exile.push(c);
    }
    g.perm_mut(src).data.set(DataKey::Prepared, Val::Bool(true));
    crate::glog!(g, "    {} exiles eight cards: Emeritus of Ideation is prepared", g.player(p).name);
    Ok(vec![])
}

// ------------------------------------------------------------------ card selection
/// mine._top: the top n cards, taken off the library (the top first)
fn top(g: &mut Game, p: PlayerId, n: usize) -> Vec<CardId> {
    let k = n.min(g.player(p).library.len());
    (0..k).map(|_| g.player_mut(p).library.pop().unwrap()).collect()
}

/// the cards best first by topdeck.desire (Python's stable `sorted(.., key=lambda x: -desire(x))`)
fn by_desire(g: &Game, p: PlayerId, cs: Vec<CardId>) -> Vec<CardId> {
    let mut k: Vec<(f64, CardId)> = cs.into_iter().map(|c| (-crate::ai::topdeck::desire(g, p, c), c)).collect();
    k.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    k.into_iter().map(|x| x.1).collect()
}

/// top three: the best to hand, the next exiled and playable this turn, the last to the bottom. HUMAN(phase 9).
fn iteration(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let t = top(g, p, 3);
    let t = by_desire(g, p, t);
    if t.is_empty() {
        return Ok("gy");
    }
    let pl = g.player_mut(p);
    pl.hand.push(t[0]);
    if t.len() >= 2 {
        pl.hand.push(t[1]);
        pl.impulse.push(t[1]); // exiled, may play it this turn
    }
    if t.len() >= 3 {
        pl.library.insert(0, t[2]);
    }
    Ok("gy")
}

/// take k of the top n to hand, the rest to the bottom
fn look_and_take(g: &mut Game, p: PlayerId, n: usize, k: usize) {
    let t = top(g, p, n);
    let t = by_desire(g, p, t);
    let k = k.min(t.len());
    let pl = g.player_mut(p);
    pl.hand.extend(&t[..k]);
    pl.library.splice(0..0, t[k..].iter().copied());
}

/// top three: one to hand (two with an instant and a sorcery in the graveyard), the rest to the bottom.
/// HUMAN(phase 9).
fn flow(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let gy = &g.player(p).gy;
    let k = if gy.iter().any(|&x| g.db.get(x).instant) && gy.iter().any(|&x| g.db.get(x).sorcery) { 2 } else { 1 };
    look_and_take(g, p, 3, k);
    Ok("gy")
}

/// top five: the best two to hand, the rest to the bottom. HUMAN(phase 9).
fn stock(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    look_and_take(g, p, 5, 2);
    Ok("gy")
}

/// draws three when any graveyard has twenty or more cards, else one
fn visions(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let n = if g.players.iter().any(|q| q.gy.len() >= 20) { 3 } else { 1 };
    draw(g, p, n, false)?;
    Ok("gy")
}

/// bounces a nonland permanent, then surveil 1
fn betrayal(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = ctx.target
        && g.perm(t).on_bf
    {
        apply_removal(g, Some(p), t, "bounce", Some(c))?;
    }
    crate::cardcode::scry(g, p, 1, true)?;
    Ok("gy")
}

/// the best of its three modes: ping one or two X/1s (or a player at 1), bounce an opposing commander or token, or
/// surveil 2 and draw. HUMAN(phase 9).
fn charm(g: &mut Game, p: PlayerId, c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let mut x1: Vec<(f64, PermId)> = opps
        .iter()
        .flat_map(|&q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m) && etgh(g, m) <= 1 && pval(g, m) >= 2.0 && !untargetable(g, m))
        .map(|m| (-pval(g, m), m))
        .collect();
    x1.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let x1: Vec<PermId> = x1.into_iter().take(2).map(|x| x.1).collect();
    let faces: Vec<PlayerId> = opps.iter().copied().filter(|&q| g.player(q).life <= 1).collect();
    let ping_v = x1.iter().map(|&m| pval(g, m)).psum() + 10.0 * faces.len() as f64;
    let bn: Vec<PermId> = opps
        .iter()
        .flat_map(|&q| g.player(q).perms.iter().copied())
        .filter(|&m| !untargetable(g, m) && !g.perm(m).phased && (g.perm(m).is_cmd || g.perm(m).token))
        .collect();
    let bv = |m: PermId| pval(g, m) * if g.perm(m).is_cmd { 1.5 } else { 1.0 };
    let b = first_max(&bn, bv);
    let bounce_v = b.map_or(0.0, bv);
    if ping_v >= 4.0f64.max(bounce_v) {
        for &q in faces.iter().take(2) {
            lose_life(g, q, 1, Some(p), "burn", None)?;
        }
        for &m in x1.iter().take(2 - faces.len().min(2)) {
            if g.perm(m).on_bf {
                apply_removal(g, Some(p), m, "dmg1", Some(c))?;
            }
        }
    } else if bounce_v >= 6.0 {
        apply_removal(g, Some(p), b.unwrap(), "bounce", Some(c))?;
    } else {
        crate::cardcode::scry(g, p, 2, true)?;
        draw(g, p, 1, false)?;
    }
    Ok("gy")
}

// ------------------------------------------------------------------ Reenact the Crime
/// mine.gy_this_turn: (card, owner) for every card put into a graveyard this turn (after each turn's start snapshot)
pub fn gy_this_turn(g: &Game) -> Vec<(CardId, PlayerId)> {
    let mut out = vec![];
    for q in &g.players {
        let mut left = q.gy_start.clone();
        for &x in &q.gy {
            if let Some(i) = left.iter().position(|&y| y == x) {
                left.swap_remove(i);
            } else {
                out.push((x, q.id));
            }
        }
    }
    out
}

/// mine.reenact_target: the best nonland card put into a graveyard this turn
pub fn reenact_target(g: &Game, p: PlayerId, exclude: Option<CardId>) -> Option<(CardId, PlayerId)> {
    let cands: Vec<(CardId, PlayerId)> = gy_this_turn(g)
        .into_iter()
        .filter(|&(x, _)| !g.db.get(x).land && Some(x) != exclude && !g.db.get(x).tag(Tag::Crackle))
        .collect();
    first_max(&cands, |(x, _)| card_worth(g, p, x, false))
}

/// exiles the best nonland card put into a graveyard this turn and casts a copy for free (cast only when there is
/// one)
fn reenact(g: &mut Game, p: PlayerId, c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let Some((x, q)) = reenact_target(g, p, Some(c)) else { return Ok("gy") }; // no legal target: it does nothing
    let pl = g.player_mut(q);
    remove_card(&mut pl.gy, x);
    pl.exile.push(x);
    crate::glog!(g, "    Reenact the Crime copies {}", g.db.get(x).name);
    crate::engine::cast::copy_spell(g, p, x, None, true)?;
    Ok("gy")
}

// ------------------------------------------------------------------ Return the Favor (spree)
/// +{1}: copy target instant or sorcery spell: your own big spell
fn rtf_copy(g: &mut Game, c: CardId, p: PlayerId, spell: CardId) -> Res {
    let sd = g.db.get(spell);
    if g.in_rtf || !g.player(p).hand.contains(&c) || !(sd.instant || sd.sorcery) || spell == c {
        return Ok(());
    }
    if spell_copy_value(g, p, spell) < 5.0 || !can_pay(g, p, 1, "RR", false) {
        return Ok(());
    }
    g.in_rtf = true;
    let r = rtf_copy_go(g, c, p, spell);
    g.in_rtf = false;
    r
}

fn rtf_copy_go(g: &mut Game, c: CardId, p: PlayerId, spell: CardId) -> Res {
    remove_card(&mut g.player_mut(p).hand, c);
    pay(g, p, 1, "RR", false)?;
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.cast_names.insert(c);
    on_cast(g, p, c)?;
    crate::glog!(g, "  {} casts Return the Favor: copy {}", g.player(p).name, g.db.get(spell).name);
    if counter_window(g, p, c, 5.0, vec![])? {
        let ctx = cast_ctx(g, spell);
        crate::engine::cast::copy_spell(g, p, spell, ctx.as_ref(), false)?;
    }
    g.player_mut(p).gy.push(c);
    Ok(())
}

/// mine.rtf_redirect: +{1}: change the target of a removal spell aimed at your permanent to one of the caster's
pub fn rtf_redirect(
    g: &mut Game,
    owner: PlayerId,
    m: PermId,
    kind: Sym,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    let (Some(spell), Some(actor)) = (spell, actor) else { return Ok(false) };
    if actor == owner || matches!(kind, "edict" | "wipe") {
        return Ok(false);
    }
    let Some(c) = g.db.id("Return the Favor").filter(|c| g.player(owner).hand.contains(c)) else { return Ok(false) };
    if !can_pay(g, owner, 1, "RR", false) || pval(g, m) < 4.0 {
        return Ok(false);
    }
    let alt: Vec<PermId> = g
        .player(actor)
        .perms
        .iter()
        .copied()
        .filter(|&x| !g.perm(x).phased && !untargetable(g, x) && (g.is_creature(x) || kind != "dmg"))
        .collect();
    let Some(t) = first_max(&alt, |x| pval(g, x)) else { return Ok(false) };
    remove_card(&mut g.player_mut(owner).hand, c);
    pay(g, owner, 1, "RR", false)?;
    let pl = g.player_mut(owner);
    pl.spells_this_turn += 1;
    pl.cast_names.insert(c);
    on_cast(g, owner, c)?;
    g.player_mut(owner).gy.push(c);
    crate::glog!(
        g,
        "    {} casts Return the Favor: {} now targets {}",
        g.player(owner).name,
        g.db.get(spell).name,
        g.perm(t).name
    );
    apply_removal(g, Some(actor), t, kind, Some(spell))?;
    Ok(true)
}

/// mine._cast_response: cast c from hand as a response (paid unless free): its cast triggers (magecraft, Veyran)
/// happen
fn cast_response(g: &mut Game, owner: PlayerId, c: CardId, gn: u32, pips: &str) -> Res<bool> {
    if !g.player(owner).hand.contains(&c) {
        return Ok(false);
    }
    if gn > 0 || !pips.is_empty() {
        if !can_pay(g, owner, gn, pips, false) {
            return Ok(false);
        }
        pay(g, owner, gn, pips, false)?;
    }
    let pl = g.player_mut(owner);
    remove_card(&mut pl.hand, c);
    pl.spells_this_turn += 1;
    pl.cast_names.insert(c);
    pl.stat("spells_cast", 1);
    on_cast(g, owner, c)?;
    g.player_mut(owner).gy.push(c);
    Ok(true)
}

/// mine.veyran_protect: removal aimed at Veyran (or another key permanent): Deflecting Swat (free with your commander
/// out) sends a targeted spell at one of the caster's permanents; Dive Down (hexproof, +0/+3) and Slip Out the Back
/// (phases out) save a creature from a targeted spell
pub fn veyran_protect(
    g: &mut Game,
    owner: PlayerId,
    m: PermId,
    kind: Sym,
    actor: Option<PlayerId>,
    spell: Option<CardId>,
) -> Res<bool> {
    if pval(g, m) < 4.0 && !g.perm(m).is_cmd {
        return Ok(false);
    }
    let targeted = spell.is_some() && actor.is_some_and(|a| a != owner) && !matches!(kind, "edict" | "wipe");
    if !targeted {
        return Ok(false);
    }
    let (actor, spell) = (actor.unwrap(), spell.unwrap());
    if let Some(sw) = g.db.id("Deflecting Swat").filter(|c| g.player(owner).hand.contains(c)) {
        let free = commander_out(g, owner);
        let alt: Vec<PermId> = g
            .player(actor)
            .perms
            .iter()
            .copied()
            .filter(|&x| !g.perm(x).phased && !untargetable(g, x) && (g.is_creature(x) || kind != "dmg"))
            .collect();
        if !alt.is_empty() && cast_response(g, owner, sw, if free { 0 } else { 2 }, if free { "" } else { "R" })? {
            let t = first_max(&alt, |x| pval(g, x)).unwrap();
            crate::glog!(
                g,
                "    {} casts Deflecting Swat: {} now targets {}",
                g.player(owner).name,
                g.db.get(spell).name,
                g.perm(t).name
            );
            apply_removal(g, Some(actor), t, kind, Some(spell))?;
            return Ok(true);
        }
    }
    if !g.is_creature(m) {
        return Ok(false);
    }
    for name in ["Slip Out the Back", "Dive Down", "Shore Up"] {
        let Some(c) = g.db.id(name).filter(|c| g.player(owner).hand.contains(c)) else { continue };
        if !cast_response(g, owner, c, 0, "U")? {
            continue;
        }
        if name == "Slip Out the Back" {
            g.perm_mut(m).phased = true;
            crate::glog!(g, "    {} casts Slip Out the Back: {} phases out", g.player(owner).name, g.perm(m).name);
        } else {
            let x = g.perm_mut(m);
            if !x.eot_kw.contains(&"hexproof") {
                x.eot_kw.push("hexproof");
            }
            let (a, b) = x.eot_pt;
            x.eot_pt = if name == "Dive Down" { (a, b + 3) } else { (a + 1, b + 1) };
            if name == "Shore Up" {
                x.tapped = false;
            }
            crate::glog!(g, "    {} casts {name}: {} gains hexproof", g.player(owner).name, g.perm(m).name);
        }
        return Ok(true);
    }
    Ok(false)
}

// ------------------------------------------------------------------ Veyran's utility lands (end of an opponent's turn)
/// {1}{U}{R}, {T}: draw a card, then discard a card (with mana left over at the end of an opponent's turn)
fn lighthouse(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || g.land(l).tapped || g.active == Some(p) || !can_pay_without(g, p, l, 1, "UR") {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.2,
        label: "Desolate Lighthouse loot".into(),
        act: Some(Action::Plan { f: lighthouse_go, arg: l.0 as i64 }),
    }])
}

fn lighthouse_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId(arg as u32);
    if !g.player(p).lands.contains(&l) || g.land(l).tapped || !pay_without(g, p, l, 1, "UR")? {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    crate::glog!(g, "  {} loots with Desolate Lighthouse", g.player(p).name);
    let cd = g.land(l).cd;
    if ability_window_card(g, p, cd, "draw, then discard", None, None)? {
        draw(g, p, 1, false)?;
        discard_worst(g, p, 1)?;
    }
    Ok(true)
}

/// {2}{U}{R}, {T}: surveil 1 (with mana left over at the end of an opponent's turn)
fn summit(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || g.land(l).tapped || g.active == Some(p) || !can_pay_without(g, p, l, 2, "UR") {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 0.6,
        label: "Spectacle Summit surveil".into(),
        act: Some(Action::Plan { f: summit_go, arg: l.0 as i64 }),
    }])
}

fn summit_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId(arg as u32);
    if !g.player(p).lands.contains(&l) || g.land(l).tapped || !pay_without(g, p, l, 2, "UR")? {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    let cd = g.land(l).cd;
    if ability_window_card(g, p, cd, "surveil 1", None, None)? {
        crate::cardcode::scry(g, p, 1, true)?;
    }
    Ok(true)
}

// ======================================================== the rest of the Sauron section (cards in no list now)
/// mine.helm_hexproof: Champion's Helm gives hexproof while the creature is legendary
pub fn helm_hexproof(g: &Game, m: PermId) -> bool {
    equipped(g, m, Tag::Helm) && is_legendary(g, m)
}

/// names Orc: each other Orc (and every new Orc Army) enters with an additional +1/+1 counter
fn mimic(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src && g.perm(m).owner == g.perm(src).owner && g.is_creature(m) && orcish(g, m) {
        crate::cardcode::add_counters(g, m, 1);
    }
    Ok(())
}

/// draws for each creature of yours that deals combat damage to a player
fn recon(g: &mut Game, src: Src, p: PlayerId, _a: PermId, _d: PlayerId, _dmg: i32) -> Res {
    if p == g.perm(src).owner && trigger_window(g, p, Some(src), "draw a card", None)? {
        draw(g, p, 1, false)?;
    }
    Ok(())
}

/// cycling {2} when few creatures would connect
fn recon_cycle(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "", false) {
        return Ok(vec![]);
    }
    if g.player(p).perms.iter().filter(|&&m| g.is_creature(m)).count() >= 2 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.0,
        label: "cycle Reconnaissance Mission".into(),
        act: Some(Action::Plan { f: recon_cycle_go, arg: c.0 as i64 }),
    }])
}

fn recon_cycle_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    pay(g, p, 2, "", false)?;
    g.player_mut(p).gy.push(c);
    draw(g, p, 1, false)?;
    Ok(true)
}

/// loyalty 3: +1 surveil 2; -1 each opponent discards; -2 returns a creature (mana value 3 or less); -7 an opponent
/// skips 0-5 turns (five coins)
fn ralz(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if p != x.owner || post.is_none() || x.phased || x.loyalty.is_none() || !pw_once(g, src) {
        return Ok(vec![]);
    }
    let loyalty = x.loyalty.unwrap();
    let mut out = vec![Opt {
        utility: 1.0,
        label: "Ral Zarek +1 (surveil 2)".into(),
        act: Some(Action::Ability { src, f: ralz_plus1, arg: 0 }),
    }];
    let hands = g.opps(p).filter(|&q| !g.player(q).hand.is_empty()).count();
    if hands > 0 && loyalty >= 2 {
        out.push(Opt {
            utility: 0.8 + 0.5 * hands as f64,
            label: "Ral Zarek -1 (each opponent discards)".into(),
            act: Some(Action::Ability { src, f: ralz_minus1, arg: 0 }),
        });
    }
    let rc: Vec<CardId> =
        g.player(p).gy.iter().copied().filter(|&c| g.db.get(c).creature && g.db.get(c).cmc <= 3).collect();
    if !rc.is_empty() && loyalty >= 2 {
        let c = first_max(&rc, |c| card_worth(g, p, c, true)).unwrap();
        out.push(Opt {
            utility: 0.8 * card_worth(g, p, c, true) / 2.0,
            label: format!("Ral Zarek -2 -> {}", g.db.get(c).name),
            act: Some(Action::Ability { src, f: ralz_minus2, arg: c.0 as i64 }),
        });
    }
    if loyalty >= 7 && g.opps(p).next().is_some() {
        let opps: Vec<PlayerId> = g.opps(p).collect();
        let q = first_max(&opps, |o| threat(g, p, o)).unwrap();
        out.push(Opt {
            utility: 8.0,
            label: format!("Ral Zarek -7 -> {}", g.player(q).name),
            act: Some(Action::Ability { src, f: ralz_minus7, arg: q.0 as i64 }),
        });
    }
    Ok(out)
}

fn ralz_plus1(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !pw_once(g, src) {
        return Ok(false);
    }
    pw_use(g, src, 1)?;
    if ability_window(g, p, Some(src), "+1", None, None)? {
        crate::cardcode::scry(g, p, 2, true)?;
    }
    Ok(true)
}

fn ralz_minus1(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !pw_once(g, src) {
        return Ok(false);
    }
    let cd = g.perm(src).cd.unwrap();
    pw_use(g, src, -1)?;
    if ability_window_card(g, p, cd, "-1", None, None)? {
        for q in g.opps(p).collect::<Vec<_>>() {
            let n = g.player(q).hand.len();
            if n > 0 {
                let i = g.rng.below(n as u64) as usize;
                discard_index(g, q, i)?;
            }
        }
    }
    Ok(true)
}

fn ralz_minus2(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !pw_once(g, src) || !g.player(p).gy.contains(&c) {
        return Ok(false);
    }
    let cd = g.perm(src).cd.unwrap();
    pw_use(g, src, -2)?;
    if ability_window_card(g, p, cd, "-2", None, None)? && g.player(p).gy.contains(&c) {
        remove_card(&mut g.player_mut(p).gy, c);
        enter(g, p, c, Enter::default())?;
    }
    Ok(true)
}

fn ralz_minus7(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let q = PlayerId(arg as u8);
    if !pw_once(g, src) {
        return Ok(false);
    }
    let cd = g.perm(src).cd.unwrap();
    pw_use(g, src, -7)?;
    if !ability_window_card(g, p, cd, "-7", Some(10.0), None)? {
        return Ok(true);
    }
    let n = (0..5).filter(|_| g.rng.random() < 0.5).count() as u32;
    g.player_mut(q).skip_turns += n;
    crate::glog!(g, "  {} uses Ral Zarek -7: {} skips {n} turn(s)", g.player(p).name, g.player(q).name);
    Ok(true)
}

// ------------------------------------------------------------------ lands: Plaza of Heroes, Unclaimed Territory
/// mine.plaza_colors: {C}; any colour for legendary spells; colours among your legendary permanents
fn plaza_colors(g: &Game, p: PlayerId, _l: LandId) -> crate::cards::Colors {
    if g.pay_for.is_some_and(|c| g.db.get(c).tag(Tag::Leg)) {
        return g.player(p).ident; // a legendary spell: any colour
    }
    let mut cols = crate::cards::Colors::NONE;
    for &m in &g.player(p).perms {
        if let Some(cd) = g.perm(m).cd
            && is_legendary(g, m)
        {
            let pips: String = g.db.get(cd).pips.chars().filter(|ch| "WUBRG".contains(*ch)).collect();
            cols = cols.union(crate::cards::Colors::from_letters(&pips));
        }
    }
    cols
}

/// mine.territory_type: the creature type Unclaimed Territory names as it enters: Sauron's Orc, else the one most of
/// p's creature cards share. Python works it out once and keeps it (on the player, and on the land as `ctype`); the
/// land's colours are asked without a way to keep it here, so it is worked out each time from the same cards.
pub fn territory_type(g: &Game, p: PlayerId) -> Sym {
    if g.player(p).key == "sauron" {
        return "orc";
    }
    let pl = g.player(p);
    let mut n: IndexMap<&str, usize> = IndexMap::new();
    let cards = pl.library.iter().chain(&pl.hand).chain(&pl.gy).chain(&pl.exile).copied();
    let on_bf = pl.perms.iter().filter_map(|&m| g.perm(m).cd);
    for c in cards.chain(on_bf).chain(std::iter::once(pl.cmd)) {
        let d = g.db.get(c);
        if d.creature {
            for x in d.subtypes.iter() {
                *n.entry(&**x).or_insert(0) += 1;
            }
        }
    }
    match n.iter().min_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0))) {
        Some((t, _)) => intern(t),
        None => "human",
    }
}

/// mine.territory_colors: {C}, or any colour for a creature spell of the named type (a changeling is every type)
fn territory_colors(g: &Game, p: PlayerId, l: LandId) -> crate::cards::Colors {
    let Some(pf) = g.pay_for.filter(|&c| g.db.get(c).creature) else { return crate::cards::Colors::NONE };
    let t = match g.land(l).data.get(DataKey::Ctype) {
        Some(Val::Str(s)) => *s,
        _ => territory_type(g, p),
    };
    let d = g.db.get(pf);
    if d.has_subtype(t) || d.has_kw("changeling") { g.player(p).ident } else { crate::cards::Colors::NONE }
}

// ------------------------------------------------------------------ Andúril, Flame of the West
/// the equipped creature attacks: two tapped 1/1 white Spirits with flying, attacking if it's legendary (the Ring
/// makes your Ring-bearer legendary, so the Orc Army's Spirits attack once the Ring has tempted you)
fn anduril(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    let Some(host) = g.perm(src).attached else { return Ok(vec![]) };
    if g.perm(src).owner != p || !atk.contains(&host) || !on_bf_of(g, p, host) {
        return Ok(vec![]);
    }
    let leg = is_legendary(g, host);
    let name = format!("two 1/1 flying Spirits{}", if leg { ", attacking" } else { "" });
    if !trigger_window(g, p, Some(src), &name, None)? {
        return Ok(vec![]);
    }
    let spec = Tokens {
        tgh: Some(1),
        fly: true,
        color: Some(crate::cards::Colors::from_letters("W")),
        types: vec!["spirit"],
        attacking: true,
        sick: !leg,
        ..Tokens::new(2, 1)
    };
    let toks = make_tokens(g, p, spec)?;
    Ok(if leg { toks } else { vec![] }) // tapped either way; only a legendary creature's Spirits attack
}

// ------------------------------------------------------------------ Lord of the Nazgûl
/// flying; a 3/3 menace Wraith for each instant or sorcery you cast; nine or more Wraiths are 9/9 until end of turn
/// (protection from Ring-bearers is not modeled)
fn nazgul(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let o = g.perm(src).owner;
    let d = g.db.get(c);
    if caster != o || !(d.instant || d.sorcery) {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "a 3/3 Wraith with menace", None)? {
        return Ok(());
    }
    let mut data = crate::state::PermData::default();
    data.set(DataKey::Kws, Val::List(vec![Val::Str("menace")]));
    let spec = Tokens {
        tgh: Some(3),
        color: Some(crate::cards::Colors::from_letters("B")),
        types: vec!["wraith"],
        data,
        ..Tokens::new(1, 3)
    };
    make_tokens(g, o, spec)?;
    let wraiths: Vec<PermId> = g
        .player(o)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && !g.perm(m).phased && has_type(g, m, "wraith"))
        .collect();
    if wraiths.len() >= 9 {
        for &m in &wraiths {
            let x = g.perm_mut(m);
            let (a, b) = x.eot_pt;
            x.eot_pt = (a + 9 - x.pow, b + 9 - x.tgh); // base 9/9 (counters and other bonuses still apply)
        }
        g.dsl_on = true;
        crate::glog!(g, "    Lord of the Nazgûl: {} Wraiths are 9/9 this turn", wraiths.len());
    }
    Ok(())
}

// ------------------------------------------------------------------ Palantír of Orthanc
/// mine._palantir_target: the opponent to put the choice to: the one closest to dying. HUMAN(phase 9).
fn palantir_target(g: &Game, o: PlayerId) -> Option<PlayerId> {
    let opps: Vec<PlayerId> = g.opps(o).collect();
    min_by(&opps, |q| g.player(q).life as f64)
}

/// mine._palantir_lets_draw: the targeted opponent's choice: true = you draw a card, false = you mill x and they lose
/// the total mana value. The AI takes the loss while it's small and hands over the card once the expected loss would
/// really hurt. HUMAN(phase 9).
fn palantir_lets_draw(g: &Game, q: PlayerId, o: PlayerId, x: i64) -> bool {
    let lib = &g.player(o).library;
    let avg = if lib.is_empty() {
        0.0
    } else {
        lib.iter().map(|&c| g.db.get(c).cmc as i64).sum::<i64>() as f64 / lib.len() as f64
    };
    let expect = avg * x.min(lib.len() as i64) as f64;
    let life = g.player(q).life as f64;
    expect >= life || expect > 6.0f64.max(life / 4.0)
}

/// end step: an influence counter and scry 2, then the targeted opponent lets you draw a card or takes life loss equal
/// to the mana values of the cards you mill (one per counter)
fn palantir(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let o = g.perm(src).owner;
    if p != o {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "influence counter, scry 2, then an opponent chooses", Some(3.0))?
        || !on_bf_of(g, o, src)
    {
        return Ok(());
    }
    let x = g.perm(src).data.int(DataKey::Influence) + 1;
    g.perm_mut(src).data.set(DataKey::Influence, Val::Int(x));
    crate::cardcode::scry(g, o, 2, false)?;
    let Some(q) = palantir_target(g, o) else { return Ok(()) };
    if !g.player(q).alive {
        return Ok(());
    }
    if palantir_lets_draw(g, q, o, x) {
        crate::glog!(g, "    Palantír: {} lets {} draw a card", g.player(q).name, g.player(o).name);
        draw(g, o, 1, false)?;
    } else {
        let lib = &g.player(o).library;
        let k = (x as usize).min(lib.len());
        let loss: i32 = lib[lib.len() - k..].iter().map(|&c| g.db.get(c).cmc as i32).sum();
        mill(g, o, x as usize)?;
        crate::glog!(g, "    Palantír: {} mills {k}; {} loses {loss}", g.player(o).name, g.player(q).name);
        if loss != 0 {
            lose_life(g, q, loss, Some(o), "triggers", None)?;
        }
        check_state(g)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Twinflame
/// mine.MAGECRAFT
const MAGECRAFT: [Tag; 6] = [Tag::Ping, Tag::Mystic, Tag::Spelldraw, Tag::Spelltok, Tag::Kiln, Tag::Dragoncaller];

/// mine.twinflame_value: what a hasty token copy of creature m (until the end step) is worth this turn; spells: the
/// instants and sorceries still castable after Twinflame (each triggers a magecraft copy again)
fn twinflame_value(g: &Game, p: PlayerId, m: PermId, post: bool, spells: Option<usize>) -> f64 {
    let x = g.perm(m);
    let Some(cd) = x.cd else { return 0.0 };
    if !g.is_creature(m) || x.phased || is_legendary(g, m) || x.owner != p {
        return 0.0;
    }
    let t = &g.db.get(cd).tags;
    let mut v = (etb_value(g, p, m) + 0.5 * x.plus.max(0) as f64).max(0.0); // the copy enters fresh
    if t.has(Tag::Rem) && t.has(Tag::Etb) {
        v += 3.0; // Venser: another bounce
    }
    if MAGECRAFT.iter().any(|&k| t.has(k)) {
        let spells = spells.unwrap_or_else(|| {
            g.player(p)
                .hand
                .iter()
                .filter(|&&c| {
                    let d = g.db.get(c);
                    (d.instant || d.sorcery) && &*d.name != "Twinflame"
                })
                .count()
        });
        let per = if t.has(Tag::Ping) { (t.int_or_0(Tag::Ping) as usize * g.opps(p).count()) as f64 } else { 1.5 };
        v += 0.5 * per * spells as f64 * if has(g, p, Tag::Veyran) { 2.0 } else { 1.0 };
    }
    if !post && !t.has(Tag::Noatk) && epow(g, m) > 0 {
        v += 0.25 * epow(g, m) as f64; // haste: attacks this turn
    }
    v
}

/// the best number of Twinflame targets: (their total value, the targets)
fn twinflame_best(g: &Game, p: PlayerId, c: CardId, post: bool) -> Option<(f64, Vec<PermId>)> {
    let mana = total_mana(g, p, false) as i64;
    let mut rest: Vec<i64> = g
        .player(p)
        .hand
        .iter()
        .filter(|&&x| x != c && (g.db.get(x).instant || g.db.get(x).sorcery))
        .map(|&x| g.db.get(x).cmc as i64)
        .collect();
    rest.sort();
    let mut best: Option<(f64, Vec<PermId>)> = None;
    for n in 1..6usize {
        // n targets: {1}{R} + {2}{R} per extra one
        let cost = 2 + 3 * (n as i64 - 1);
        if !can_pay(g, p, 1 + 2 * (n as u32 - 1), &"R".repeat(n), false) {
            break;
        }
        let (mut left, mut spells) = (mana - cost, 0usize);
        for &x in &rest {
            // the cheapest spells still castable afterwards
            if x <= left {
                left -= x;
                spells += 1;
            }
        }
        let mut ranked: Vec<(f64, PermId)> =
            g.player(p).perms.iter().map(|&m| (twinflame_value(g, p, m, post, Some(spells)), m)).collect();
        ranked.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        ranked.truncate(n);
        if ranked.len() < n || ranked[n - 1].0 < 1.5 {
            break;
        }
        let total = ranked.iter().map(|x| x.0).psum();
        if best.as_ref().is_none_or(|b| total > b.0) {
            best = Some((total, ranked.into_iter().map(|x| x.1).collect()));
        }
    }
    best
}

/// Strive: {1}{R}, plus {2}{R} for each target beyond the first. Hasty token copies of any number of your creatures,
/// exiled at the next end step. The AI copies the creatures worth copying that it can pay for, in its main phases.
fn twinflame(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let Some(post) = post else { return Ok(vec![]) };
    if g.active != Some(p) || !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "R", false) {
        return Ok(vec![]);
    }
    let Some((total, tg)) = twinflame_best(g, p, c, post) else { return Ok(vec![]) };
    let k = tg.len() - 1;
    Ok(vec![Opt {
        utility: 1.0 + 0.6 * total,
        label: format!("Twinflame ({} target{})", tg.len(), if k > 0 { "s" } else { "" }),
        act: Some(Action::Plan { f: twinflame_go, arg: c.0 as i64 | (post as i64) << 32 }),
    }])
}

fn twinflame_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, post) = (CardId(arg as u16), (arg >> 32) != 0);
    // the targets as the option chose them (Python's closure keeps them; nothing has happened since)
    let Some((_, tg)) = twinflame_best(g, p, c, post) else { return Ok(false) };
    let targets: Vec<PermId> = tg.into_iter().filter(|&m| on_bf_of(g, p, m) && !g.perm(m).phased).collect();
    if !g.player(p).hand.contains(&c) || targets.is_empty() {
        return Ok(false);
    }
    let extra = targets.len() as u32 - 1;
    let pips = "R".repeat(extra as usize + 1);
    if !can_pay(g, p, 1 + 2 * extra, &pips, false) {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    pay(g, p, 1 + 2 * extra, &pips, false)?;
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    if g.log.is_some() {
        let names: Vec<&str> = targets.iter().map(|&m| g.perm(m).name).collect();
        crate::glog!(g, "  {} casts Twinflame copying {}", g.player(p).name, names.join(", "));
    }
    on_cast(g, p, c)?;
    if g.over || !g.player(p).alive {
        return Ok(true);
    }
    if !counter_window(g, p, c, 3.0, vec![])? {
        g.player_mut(p).gy.push(c);
        return Ok(true);
    }
    for m in targets {
        if !on_bf_of(g, p, m) {
            continue;
        }
        let cd = g.perm(m).cd.unwrap();
        if let Some(cp) = enter_token_copy(g, p, cd)? {
            g.perm_mut(cp).sick = false; // haste; exiled at the end step
            g.perm_mut(cp).temp = true;
        }
    }
    g.player_mut(p).gy.push(c);
    check_state(g)?;
    Ok(true)
}

// ------------------------------------------------------------------ Galvanic Iteration, Fiery Emancipation
/// copies your next instant or sorcery this turn (a copy: magecraft fires again); flashback {1}{U}{R}
fn galvanic(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    g.player_mut(p).galvanic = Some(g.turn_stamp());
    crate::glog!(g, "    Galvanic Iteration: {}'s next instant or sorcery this turn is copied", g.player(p).name);
    Ok("gy") // (Python: 'exile' with ctx['flashback'], which nothing sets; flashback casts are exiled anyway)
}

/// damage your sources deal to opponents and their permanents is tripled (pings, burn, combat): the engine reads it
/// by name; this hook only puts it among the hooked permanents, as Python's does
fn emancipation_live(_g: &mut Game, _src: Src, _p: PlayerId, _m: PermId) -> Res {
    Ok(())
}

// ------------------------------------------------------------------ Docent of Perfection // Final Iteration
pub const DOCENT: &str = "Docent of Perfection // Final Iteration";

fn wizards(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).phased && has_type(g, m, "wizard")).count()
}

fn docent_in(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.selfpt = true; // the engine reads its Wizards' size from here on
    }
    Ok(())
}

/// mine._final: transformed: once its controller has had three Wizards (its trigger's 'then if you control three or
/// more Wizards, transform'; kept once true). Python marks it whenever the Wizards' size or flying is read; reads
/// can't write here, so a read also counts three Wizards now, and the state check (docent_latch) keeps it.
fn is_final(g: &Game, src: PermId) -> bool {
    let x = g.perm(src);
    x.data.truthy(DataKey::Final) || (x.on_bf && wizards(g, x.owner) >= 3)
}

/// the state check that keeps Final Iteration transformed once it has had three Wizards
fn docent_latch(g: &mut Game, src: Src) -> Res {
    if !g.perm(src).data.truthy(DataKey::Final) && is_final(g, src) {
        g.perm_mut(src).data.set(DataKey::Final, Val::Bool(true));
        crate::glog!(g, "    Docent of Perfection transforms into Final Iteration");
    }
    Ok(())
}

/// mine._final_iteration (common.CREATURE_PT): Wizards you control get +2/+1 per transformed Docent
fn final_iteration(g: &Game, m: PermId) -> (i32, i32) {
    let p = g.perm(m).owner;
    if !g.is_creature(m) || !has_type(g, m, "wizard") {
        return (0, 0);
    }
    let k = g
        .player(p)
        .perms
        .iter()
        .filter(|&&x| card_name(g, x) == Some(DOCENT) && !g.perm(x).phased && is_final(g, x))
        .count() as i32;
    (2 * k, k)
}

/// a 1/1 Human Wizard per instant or sorcery you cast; with three Wizards it transforms into Final Iteration: Wizards
/// you control get +2/+1 and have flying
fn final_fly(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "flying"
        && g.perm(m).owner == g.perm(src).owner
        && g.is_creature(m)
        && has_type(g, m, "wizard")
        && is_final(g, src)
}

// ------------------------------------------------------------------ Manamorphose
/// add two mana in any combination of colours, draw a card. HUMAN(phase 9): a person's pool gets it.
fn manamorphose(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    g.player_mut(p).floating.any += 2;
    draw(g, p, 1, false)?;
    Ok("gy")
}

// ======================================================== Sephiroth's loops
// Each loop is run as its end result, after one window for opponents to answer a key piece (ais.combo_interrupted).
/// mine.OUTLETS: free sacrifice outlets
pub const OUTLETS: [&str; 3] = ["Viscera Seer", "Ashnod's Altar", "Altar of Dementia"];
/// mine.DRAINS
const DRAINS: [&str; 2] = ["Blood Artist", "Zulaport Cutthroat"];
const MIKAEUS: &str = "Mikaeus, the Unhallowed";
const MELIRA: &str = "Melira, Sylvok Outcast";

/// a loop: its name, piece groups (one card of each), and whether it kills alone (else it needs a payoff)
pub struct Loop {
    pub name: &'static str,
    pub groups: &'static [&'static [&'static str]],
    pub kills: bool,
}

/// mine.LOOPS
pub const LOOPS: [Loop; 4] = [
    // outlet: trisk_needs_outlet
    Loop { name: "Mikaeus + Triskelion", groups: &[&[MIKAEUS], &["Triskelion"]], kills: true },
    Loop { name: "Mikaeus + Kitchen Finks", groups: &[&[MIKAEUS], &["Kitchen Finks"], &OUTLETS], kills: false },
    Loop { name: "Melira + Kitchen Finks", groups: &[&[MELIRA], &["Kitchen Finks"], &OUTLETS], kills: false },
    Loop {
        name: "Nim Deathmantle + Ashnod's Altar + Grave Titan",
        groups: &[&["Nim Deathmantle"], &["Ashnod's Altar"], &["Grave Titan"]],
        kills: false,
    },
];

/// mine.LOOP_CARDS: every piece of a loop
fn loop_card(name: &str) -> bool {
    LOOPS.iter().any(|l| l.groups.iter().any(|grp| grp.contains(&name)))
}

/// mine.trisk_needs_outlet: Mikaeus + Triskelion needs no outlet: returned by undying with four counters, Triskelion
/// pings itself twice and opponents twice, and dies as a 2/2 with 2 damage. Only a counterless toughness of 4 or more
/// (Elesh Norn, Grand Cenobite, or Nim Deathmantle on it) makes it ping itself four times; then it takes a free
/// sacrifice outlet
fn trisk_needs_outlet(g: &Game, p: PlayerId, names: &[&str]) -> bool {
    let t = g.player(p).perms.iter().copied().find(|&m| card_name(g, m) == Some("Triskelion") && !g.perm(m).phased);
    match t {
        Some(t) => etgh(g, t) - g.perm(t).plus >= 4,
        None => names.contains(&"Elesh Norn, Grand Cenobite"),
    }
}

/// mine.loop_groups: a loop's piece groups as they stand: Mikaeus + Triskelion adds an outlet only when it needs one
fn loop_groups(g: &Game, p: PlayerId, l: &Loop, names: &[&str]) -> Vec<&'static [&'static str]> {
    let mut v = l.groups.to_vec();
    if l.name == LOOPS[0].name && trisk_needs_outlet(g, p, names) {
        v.push(&OUTLETS);
    }
    v
}

/// mine._names_bf: the names of p's permanents whose abilities work
fn names_bf(g: &Game, p: PlayerId) -> Vec<&str> {
    let mut out: Vec<&str> = vec![];
    for &m in &g.player(p).perms {
        let x = g.perm(m);
        let Some(n) = card_name(g, m) else { continue };
        if !x.phased && !x.neutered && !stopped(g, n) && !out.contains(&n) {
            out.push(n);
        }
    }
    out
}

/// mine.loop_kills: does this loop win with what is on the battlefield (names)?
fn loop_kills(name: &str, names: &[&str]) -> bool {
    if LOOPS.iter().find(|l| l.name == name).is_some_and(|l| l.kills) {
        return true;
    }
    if DRAINS.iter().any(|d| names.contains(d)) || names.contains(&"Altar of Dementia") {
        return true; // drain, or mill everyone
    }
    name.starts_with("Nim") && names.contains(&"Triskelion") // infinite mana: Deathmantle keeps returning Triskelion
}

/// mine.loops_blocked: graveyard hate or Hushbringer: nothing that dies comes back, or its triggers don't happen
fn loops_blocked(g: &Game, p: PlayerId) -> bool {
    (!g.hooks.is_empty() && crate::engine::hooks::total_player(g, Event::NoGraveyard, p) != 0)
        || crate::engine::zones::hushed(g)
}

/// mine.seph_loops: the loops p can run now: (loop index, key permanents, kills)
pub fn seph_loops(g: &Game, p: PlayerId) -> Vec<(usize, Vec<PermId>, bool)> {
    if g.player(p).key != "seph" || loops_blocked(g, p) {
        return vec![];
    }
    let names = names_bf(g, p);
    let mut out = vec![];
    for (i, l) in LOOPS.iter().enumerate() {
        let groups = loop_groups(g, p, l, &names);
        if !groups.iter().all(|grp| grp.iter().any(|n| names.contains(n))) {
            continue;
        }
        let mut keys = vec![];
        for (gi, grp) in groups.iter().enumerate() {
            // a piece is key only if nothing else fills its role
            let have: Vec<PermId> = g
                .player(p)
                .perms
                .iter()
                .copied()
                .filter(|&m| card_name(g, m).is_some_and(|n| grp.contains(&n)) && !g.perm(m).phased)
                .collect();
            let both = names.contains(&MIKAEUS) && names.contains(&MELIRA);
            if have.len() == 1 && !(gi == 0 && l.name.ends_with("Kitchen Finks") && both) {
                keys.push(have[0]);
            }
        }
        out.push((i, keys, loop_kills(l.name, &names)));
    }
    out
}

/// mine.run_loop
pub fn run_loop(g: &mut Game, p: PlayerId, i: usize, keys: &[PermId], kills: bool) -> Res<bool> {
    let name = LOOPS[i].name;
    let turns = g.player(p).turns;
    let pl = g.player_mut(p);
    pl.stat("combo_attempt", 1);
    pl.milestone.entry("combo").or_insert(turns);
    pl.loop_turn = Some(turns);
    crate::glog!(g, "  {} goes for the {name} loop", g.player(p).name);
    if crate::ai::combo_interrupted(g, p, "seph", keys)? {
        g.player_mut(p).stat("combo_stopped", 1);
        crate::glog!(g, "    ...the loop is stopped");
        return Ok(true);
    }
    if has_shards(g, p) && i != 0 {
        for q in g.opps(p).collect::<Vec<_>>() {
            // Aura Shards: every creature entering destroys one
            for m in g.player(q).perms.clone() {
                let ae = g.perm(m).cd.is_some_and(|c| {
                    let t = g.db.get(c).types;
                    t.has(Types::ARTIFACT) || t.has(Types::ENCHANTMENT)
                });
                if ae && !indestructible(g, m) && !untargetable(g, m) {
                    die(g, m, "destroy")?;
                }
            }
        }
        crate::glog!(g, "    Aura Shards clears the opponents' artifacts and enchantments");
    }
    if kills {
        let names = names_bf(g, p);
        let by_mill = i != 0 && !DRAINS.iter().any(|d| names.contains(d)) && !names.contains(&"Triskelion");
        crate::ai::win(g, p, "combo", Some(!by_mill))?;
        return Ok(true);
    }
    if name.contains("Finks") {
        crate::engine::life::gain(g, p, 1000)?;
        crate::glog!(g, "    {} gains 1000 life (as good as infinite)", g.player(p).name);
    } else {
        // infinite colourless mana, a Zombie kept each loop
        let spec = Tokens { color: Some(crate::cards::Colors::from_letters("B")), ..Tokens::new(20, 2) };
        make_tokens(g, p, spec)?;
        g.player_mut(p).floating.c += 20;
        crate::glog!(g, "    {} keeps 20 Zombies and floats 20 colourless mana", g.player(p).name);
    }
    Ok(true)
}

fn has_shards(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| card_tag(g, m, Tag::Shards) && !g.perm(m).phased)
}

/// mine.loop_options: the AI's main-phase options for Sephiroth's loops
pub fn loop_options(g: &Game, p: PlayerId) -> Vec<Opt> {
    if g.player(p).loop_turn == Some(g.player(p).turns) {
        return vec![];
    }
    let mut o = vec![];
    for (i, _, kills) in seph_loops(g, p) {
        let name = LOOPS[i].name;
        if !kills && name.contains("Finks") && g.player(p).life >= 500 {
            continue; // infinite life already
        }
        let u = if kills {
            14.0 - 6.0 * crate::ai::brain::removal_risk(g, p)
        } else if name.contains("Finks") {
            4.0
        } else {
            3.0
        };
        o.push(Opt {
            utility: u,
            label: format!("loop: {name}{}", if kills { "" } else { " (no payoff)" }),
            act: Some(Action::Plan { f: loop_go, arg: i as i64 }),
        });
    }
    o
}

fn loop_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    // the loop as the option listed it (nothing has happened since)
    let Some((i, keys, kills)) = seph_loops(g, p).into_iter().find(|x| x.0 == arg as usize) else { return Ok(false) };
    run_loop(g, p, i, &keys, kills)
}

/// mine.loop_need: card names that would complete a loop (with what is on the battlefield plus `extra` names),
/// killing loops first
pub fn loop_need<'a>(g: &'a Game, p: PlayerId, extra: &[&'a str]) -> Vec<&'static str> {
    if loops_blocked(g, p) {
        return vec![];
    }
    let mut names = names_bf(g, p);
    for &n in extra {
        if !names.contains(&n) {
            names.push(n);
        }
    }
    let mut want: Vec<&'static str> = vec![];
    for kills_first in [true, false] {
        for l in LOOPS.iter().filter(|l| l.kills == kills_first) {
            let groups = loop_groups(g, p, l, &names);
            let missing: Vec<&[&str]> =
                groups.iter().copied().filter(|grp| !grp.iter().any(|n| names.contains(n))).collect();
            if missing.len() == 1 && (loop_kills(l.name, &names) || l.name.ends_with("Finks")) {
                for &n in missing[0] {
                    if !want.contains(&n) {
                        want.push(n);
                    }
                }
            }
        }
    }
    want
}

/// mine.loop_prio: Sephiroth's cast priority for a loop piece in hand: it completes a loop / it is one of two pieces
/// down
pub fn loop_prio(g: &Game, p: PlayerId, c: CardId) -> Option<i32> {
    let n = &*g.db.get(c).name;
    if !loop_card(n) || loops_blocked(g, p) {
        return None;
    }
    if loop_need(g, p, &[]).contains(&n) {
        return Some(85);
    }
    let names = names_bf(g, p);
    for l in &LOOPS {
        let groups = loop_groups(g, p, l, &names);
        if groups.iter().any(|grp| grp.contains(&n))
            && groups.iter().filter(|grp| grp.iter().any(|x| names.contains(x))).count() >= 1
        {
            return Some(55);
        }
    }
    None
}

// ======================================================== Ephemerate and the flicker package: your decks' AI
/// t2.blink_value: how much re-entering is worth for p's permanent m. (A copy of t2.py's, which impls/t2.rs ports:
/// use that one once it's merged.)
fn t2_blink_value(g: &Game, _p: PlayerId, m: PermId) -> f64 {
    let x = g.perm(m);
    let Some(cd) = x.cd else { return 0.0 };
    if x.token || x.is_cmd {
        return 0.0;
    }
    let mut v = crate::dsl::etb_value(g, cd) as f64;
    if g.registry.get(cd).is_some_and(|i| i.etb.is_some()) {
        v += 3.0;
    }
    const ORINGS: [&str; 5] =
        ["Oblivion Ring", "Banishing Light", "Detention Sphere", "Cast Out", "Journey to Nowhere"];
    if ORINGS.contains(&&*g.db.get(cd).name) {
        v = 0.0;
    }
    v
}

/// t2.blink: exile m and return it under its owner's control (it enters again, untapped, summoning sick); blink
/// chains stop at three deep. (A copy of t2.py's, which impls/t2.rs ports: use that one once it's merged.)
pub fn t2_blink(g: &mut Game, _p: PlayerId, m: PermId) -> Res<Option<PermId>> {
    let x = g.perm(m);
    if x.token || x.cd.is_none() || !x.on_bf || g.blink_depth >= 3 {
        return Ok(None);
    }
    g.blink_depth += 1;
    let r = (|| {
        let x = g.perm(m);
        let (cd, owner, cmd) = (x.cd.unwrap(), x.orig, x.is_cmd);
        leave(g, m)?;
        let n = enter(g, owner, cd, Enter { orig: Some(owner), ..Enter::default() })?;
        g.perm_mut(n).is_cmd = cmd;
        if !g.hooks.is_empty() {
            fire_trigger(g, Event::ExiledFromBf, Call::Leaves { m })?;
        }
        crate::glog!(g, "    {} is blinked", g.db.get(cd).name);
        Ok(Some(n))
    })();
    g.blink_depth -= 1;
    r
}

/// mine.blink_worth: what blinking p's creature m is worth: its enter effect (hand-tagged or compiled)
pub fn blink_worth(g: &Game, p: PlayerId, m: PermId) -> f64 {
    if card_tag(g, m, Tag::Bahamut) {
        return etb_value(g, p, m); // its restart, not a flat enter hook
    }
    let (a, b) = (t2_blink_value(g, p, m), etb_value(g, p, m));
    if b > a { b } else { a }
}

/// mine.flicker_worth: Soulherder, Conjurer's Closet, Teleportation Circle, Flickering Hound, Restoration Angel:
/// blink_worth, less the +1/+1 counters lost (etb_value's count) and the Equipment that falls off (Nim Deathmantle);
/// never a token or a creature that would return to another owner
pub fn flicker_worth(g: &Game, p: PlayerId, m: PermId) -> f64 {
    let x = g.perm(m);
    if x.token || x.cd.is_none() || x.orig != p {
        return 0.0;
    }
    let mut v = blink_worth(g, p, m);
    if x.plus > 0 {
        v = v.min(etb_value(g, p, m));
    }
    v - 1.5 * g.player(p).perms.iter().filter(|&&e| g.perm(e).attached == Some(m)).count() as f64
}

/// mine.resto_cast: flash in Restoration Angel from hand: its enters trigger blinks your non-Angel creature m (a
/// removal spell aimed at it fizzles). True if m got away
pub fn resto_cast(g: &mut Game, p: PlayerId, m: PermId, why: &str) -> Res<bool> {
    let Some(c) = g.db.id("Restoration Angel").filter(|c| g.player(p).hand.contains(c)) else { return Ok(false) };
    let d = g.db.get(c);
    if !on_bf_of(g, p, m)
        || g.perm(m).token
        || has_type(g, m, "angel")
        || !castable(g, p, c, "hand")
        || !can_pay(g, p, d.generic, &d.pips, false)
        || !crate::ai::decks::pay_card(g, p, c, 0)?
    {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).gy, c);
    g.resto_target = Some(m);
    let r = enter(g, p, c, Enter::default());
    g.resto_target = None;
    r?;
    g.player_mut(p).stat(intern(&format!("resto_{why}")), 1);
    Ok(!on_bf_of(g, p, m))
}

/// mine.ephemerate_cast: cast Ephemerate from hand on your creature m: exile it and return it (it enters again,
/// untapped and new, so a removal spell aimed at it fizzles); the card is exiled and cast again for free at your next
/// upkeep (rebound). True if m got away (false if Ephemerate couldn't be cast or was countered)
pub fn ephemerate_cast(g: &mut Game, p: PlayerId, m: PermId, why: &str) -> Res<bool> {
    let Some(c) = g.db.id("Ephemerate").filter(|c| g.player(p).hand.contains(c)) else { return Ok(false) };
    if !on_bf_of(g, p, m) || g.perm(m).token || !castable(g, p, c, "hand") || !can_pay(g, p, 0, "W", false) {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    pay(g, p, 0, "W", false)?;
    let pl = g.player_mut(p);
    pl.spells_this_turn += 1;
    pl.stat("spells_cast", 1);
    pl.cast_names.insert(c);
    on_cast(g, p, c)?;
    crate::glog!(g, "  {} casts Ephemerate on {} ({why})", g.player(p).name, g.perm(m).name);
    if g.over || !counter_window(g, p, c, if why == "protection" { 6.0 } else { 3.0 }, vec![])? {
        g.player_mut(p).gy.push(c);
        return Ok(false);
    }
    let pl = g.player_mut(p);
    pl.exile.push(c);
    pl.rebound.push(c);
    pl.stat(intern(&format!("ephemerate_{why}")), 1);
    if on_bf_of(g, p, m) {
        t2_blink(g, p, m)?;
    }
    Ok(true)
}

/// instant {W}: blinks your creature, in response to targeted removal (the spell fizzles) or for its enter effect;
/// rebound: cast again free at your next upkeep on the best enter-effect creature. Your decks: blink the creature with
/// the best enters-the-battlefield effect (Archon of Cruelty, Grave Titan); kept in hand while it is the only
/// protection for a bomb
fn ephemerate_value(g: &mut Game, c: CardId, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if !crate::ai::is_main(g.player(p).key) || !g.player(p).hand.contains(&c) || !can_pay(g, p, 0, "W", false) {
        return Ok(vec![]);
    }
    let cands: Vec<(f64, PermId)> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && !g.perm(m).phased && !g.perm(m).token)
        .map(|m| (blink_worth(g, p, m), m))
        .filter(|x| x.0 >= 3.0)
        .collect();
    let Some((bv, m)) = first_max(&cands, |x| x.0) else { return Ok(vec![]) };
    let keep =
        if g.player(p).perms.iter().any(|&x| g.is_creature(x) && g.perm(x).cd.is_some_and(|cd| g.db.get(cd).bomb >= 6))
        {
            2.5
        } else {
            0.0
        };
    Ok(vec![Opt {
        utility: bv - 2.0 - keep, // two blinks with rebound
        label: format!("Ephemerate (blink {})", g.perm(m).name),
        act: Some(Action::Plan { f: ephemerate_go, arg: m.0 as i64 }),
    }])
}

fn ephemerate_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    ephemerate_cast(g, p, PermId(arg as u32), "value")
}

// ------------------------------------------------------------------ registry
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Barad-dûr")?.land_options = Some(baraddur);
    r.card(db, "Bloodsoaked Insight // Sanguine Morass")?.hand_options = Some(insight);
    r.card(db, BRAIN_FREEZE)?.resolve = Some(brain_freeze);
    r.card(db, GRAPESHOT)?.resolve = Some(grapeshot);
    r.card(db, "Iron Man, Armored Avenger")?.attack = Some(ironman);
    r.card(db, "War Machine, Avenging Arsenal")?.attack = Some(warmachine);
    let k = r.card(db, "Kefka, Court Mage // Kefka, Ruler of Ruin")?;
    k.attack = Some(kefka_attack);
    k.etb = Some(kefka_etb);
    k.lose_life = Some(kefka_ruin_draw);
    k.options = Some(kefka_ruin);
    r.card(db, "Scarlet Witch, Chaotic Avenger")?.combat_damage = Some(witch);
    r.card(db, "Urabrask, Heretic Praetor")?.upkeep = Some(urabrask);
    r.card(db, "Vision, Synthezoid Avenger")?.cast = Some(vision);
    r.card(db, "Vraska, Betrayal's Sting")?.options = Some(vraska);
    let k = r.card(db, "Kindred Discovery")?;
    k.etb = Some(kindred_etb);
    k.attack = Some(kindred_attack);
    // Sephiroth
    r.card(db, "Displacer Kitten")?.cast = Some(kitten);
    let k = r.card(db, "Nim Deathmantle")?;
    k.creature_to_gy = Some(nim_return);
    k.options = Some(nim_equip);
    let k = r.card(db, "Triskelion")?;
    k.etb = Some(trisk_etb);
    at_once(k, Event::Etb, true); // (no trigger window: Python runs it at once)
    k.options = Some(trisk_ping);
    r.card(db, "Strip Mine")?.land_options = Some(strip);
    r.card(db, "Necromancy")?.hand_options = Some(necro_flash);
    let k = r.card(db, BAHAMUT)?;
    k.etb = Some(bahamut_enters);
    k.main1 = Some(bahamut_main1);
    at_once(k, Event::Main1, true); // main1 isn't a triggered event in Python: its hooks run at once
    k.as_enters = Some(bahamut_as_enters);
    k.saga = Some((bahamut_wants_lore, bahamut_proliferated)); // CI.SAGA: Karn's Bastion and proliferate
    r.card(db, "Ephemerate")?.hand_options = Some(ephemerate_value);
    // Veyran
    r.card(db, "Muldrotha, the Gravetide")?.options = Some(muldrotha);
    let k = r.card(db, "Thousand-Year Storm")?;
    k.cast = Some(storm);
    k.copycast = Some(storm_copycast);
    r.card(db, "Alania, Divergent Storm")?.cast = Some(alania);
    r.card(db, "Ral, Storm Conduit")?.options = Some(ral);
    for (name, ..) in PREPARED {
        r.card(db, name)?.options = Some(prepared_opt);
    }
    r.card(db, "Emeritus of Ideation // Ancestral Recall")?.attack = Some(ideation_attack);
    r.card(db, "Expressive Iteration")?.resolve = Some(iteration);
    r.card(db, "Flow State")?.resolve = Some(flow);
    r.card(db, "Stock Up")?.resolve = Some(stock);
    r.card(db, "Visions of Beyond")?.resolve = Some(visions);
    r.card(db, "Banishing Betrayal")?.resolve = Some(betrayal);
    r.card(db, "Prismari Charm")?.resolve = Some(charm);
    r.card(db, "Reenact the Crime")?.resolve = Some(reenact);
    r.card(db, "Return the Favor")?.hand_cast = Some(rtf_copy);
    r.card(db, "Desolate Lighthouse")?.land_options = Some(lighthouse);
    r.card(db, "Spectacle Summit")?.land_options = Some(summit);
    // the rest of the Sauron section, and the shared cards
    let k = r.card(db, "Metallic Mimic")?;
    k.etb = Some(mimic);
    at_once(k, Event::Etb, true);
    let k = r.card(db, "Reconnaissance Mission")?;
    k.combat_damage = Some(recon);
    k.hand_options = Some(recon_cycle);
    r.card(db, "Ral Zarek, Guest Lecturer")?.options = Some(ralz);
    r.card(db, "Plaza of Heroes")?.land_cols = Some(plaza_colors);
    r.card(db, "Unclaimed Territory")?.land_cols = Some(territory_colors);
    r.card(db, "Andúril, Flame of the West")?.attack = Some(anduril);
    r.card(db, "Lord of the Nazgûl")?.cast = Some(nazgul);
    r.card(db, "Palantír of Orthanc")?.end_step = Some(palantir);
    r.card(db, "Twinflame")?.hand_options = Some(twinflame);
    r.card(db, "Galvanic Iteration")?.resolve = Some(galvanic);
    let k = r.card(db, "Fiery Emancipation")?;
    k.etb = Some(emancipation_live);
    at_once(k, Event::Etb, true);
    let k = r.card(db, DOCENT)?;
    k.etb = Some(docent_in);
    at_once(k, Event::Etb, true);
    k.grant_kw = Some(final_fly);
    k.sba = Some(docent_latch); // (Rust only: keeps it transformed; see is_final)
    r.creature_pt.push(final_iteration);
    r.card(db, "Manamorphose")?.resolve = Some(manamorphose);
    Ok(())
}
