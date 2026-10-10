//! Python's `cards/impl/mine.py`: card code of your decks. Ported so far (M5): Sauron's cards (the Ring, Sauron, the
//! Necromancer, Kefka, Urabrask, the Avengers, Vraska, Barad-dûr, Bloodsoaked Insight, Kaervek, the Underworld Breach
//! line with Brain Freeze and Grapeshot), Kindred Discovery and Old Fat Spider's chapter II, which the engine reaches
//! through `cardcode.rs`. Also ais.py's `breach_gc_options` (Breach escapes, Panoptic Mirror, Lion's Eye Diamond,
//! Bolas's Citadel), which every deck reaches.

use crate::ai::brain::{Situation, card_utility};
use crate::ai::decks::{breach_candidates, wipe_eval};
use crate::cards::{CardDb, Types};
use crate::engine::cast::{cast_card, cast_spell_copy, castable, casts_this_turn, discard_worst, on_cast, parse_cost};
use crate::engine::combat::blocked;
use crate::engine::life::{check_state, lose_life};
use crate::engine::mana::{Source, can_pay, cost_of, mana_units, pay, total_mana};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::{abilities_answered, ability_window, stack_window, trigger_window};
use crate::engine::tutors::card_worth;
use crate::engine::values::{card_name, card_tag, epow, etgh, find, has, has_type, melira, pval, threat, untargetable};
use crate::engine::zones::{
    LABMEN, Zone, amass, die, discard_cards, draw, enter_token_copy, landfall, leave, max_by, mill, min_by,
    to_zone_card,
};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, StackItem, StackKind, Val};
use crate::sym::{Sym, intern};
use crate::tag::Tag;

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
/// lands.can_pay_without / pay_without: pay a land's activation cost with other sources (the land itself stays
/// untapped). A copy of the lands module's helper (lands.rs is ported separately).
fn without_land<T>(g: &mut Game, p: PlayerId, l: LandId, f: impl FnOnce(&mut Game) -> T) -> Option<T> {
    let i = g.player(p).lands.iter().position(|&x| x == l)?;
    g.player_mut(p).lands.remove(i);
    let r = f(g);
    g.player_mut(p).lands.insert(i, l);
    Some(r)
}

fn can_pay_without(g: &mut Game, p: PlayerId, l: LandId, gn: u32, pips: &str) -> bool {
    without_land(g, p, l, |g| can_pay(g, p, gn, pips, false)).unwrap_or(false)
}

fn pay_without(g: &mut Game, p: PlayerId, l: LandId, gn: u32, pips: &str) -> Res<bool> {
    let r =
        without_land(g, p, l, |g| if can_pay(g, p, gn, pips, false) { pay(g, p, gn, pips, false) } else { Ok(false) });
    match r {
        Some(r) => r,
        None => Ok(false),
    }
}

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
fn storm_count(g: &Game) -> i32 {
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
fn mirror_candidates(g: &Game, p: PlayerId) -> Vec<(f64, CardId)> {
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
    Ok(())
}
