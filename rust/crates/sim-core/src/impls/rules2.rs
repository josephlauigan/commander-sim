//! Python's `cards/impl/rules2.py`: the last approximated clauses of pool cards (forced attacks, crew, blocking
//! restrictions and taxes, ward from Auras, graveyard triggers, and the remaining activated abilities).
//!
//! Grist's and Freyalise's first loyalty abilities (`grist_plus`, `freyalise_plus`) replace t3's in Python: t3.rs
//! lists them.

use super::partials::{
    ability_window_card, at_once, auras_on, best_opp_creature, best_opp_nonland, eot_kw, exile_fodder, first_max,
    first_min, name_of, on, pack, remove_card, unpack,
};
use super::t3::{ability, card_is, plan};
use crate::cards::{CardDb, Colors, Types};
use crate::engine::cast::{cast_card, castable, on_cast};
use crate::engine::life::{gain, lose_life};
use crate::engine::mana::{Source, Unit, can_pay, cost_of, pay, total_mana};
use crate::engine::removal::apply_removal;
use crate::engine::stack::{ability_window, counter_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library, tutor};
use crate::engine::values::{epow, etgh, has_type, pval, untargetable};
use crate::engine::zones::{
    Enter, Tokens, die, draw, enter, enter_token_copy, exile_perm, land_ramp, leave, make_artifact_tokens, make_tokens,
    max_by, min_by, searchable,
};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::pysum::PySum;
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::Sym;
use crate::tag::Tag;

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn turn_now(g: &Game) -> Val {
    Val::Stamp(g.turn_stamp())
}

// ================================================================== forced attacks
/// rules2.forced_attackers: Goblin Rabblemaster: other Goblins you control attack each combat if able; Legion Warboss
/// / Rabblemaster tokens made this turn attack if able
pub fn forced_attackers(g: &mut Game, p: PlayerId, xs: Vec<PermId>, cand0: &[PermId]) -> Res<Vec<PermId>> {
    let rabble = g.player(p).perms.iter().any(|&m| !g.perm(m).phased && name_of(g, m) == "Goblin Rabblemaster");
    let now = turn_now(g);
    let mut out = xs;
    for &m in cand0 {
        if out.contains(&m) || g.perm(m).tapped || !g.is_creature(m) {
            continue;
        }
        if (rabble && has_type(g, m, "goblin") && name_of(g, m) != "Goblin Rabblemaster")
            || g.perm(m).data.get(DataKey::MustAttack) == Some(&now)
        {
            out.push(m);
        }
    }
    Ok(out)
}

/// Goblin Rabblemaster / Legion Warboss: a hasty 1/1 Goblin each turn (at your upkeep) that must attack this turn
/// (replaces t1's upkeep hook)
fn goblin_maker(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if g.perm(src).owner == p
        && g.active == Some(p)
        && trigger_window(g, p, Some(src), "create a hasty 1/1 Goblin", None)?
    {
        let spec =
            Tokens { color: Some(Colors::from_letters("R")), types: vec!["goblin"], sick: false, ..Tokens::new(1, 1) };
        let now = turn_now(g);
        for m in make_tokens(g, p, spec)? {
            g.perm_mut(m).data.set(DataKey::MustAttack, now.clone());
        }
    }
    Ok(())
}

// ================================================================== crew
/// rules2.uncrew: vehicles crewed on an earlier turn stop being creatures
pub fn uncrew(g: &mut Game, p: PlayerId) {
    let now = turn_now(g);
    for m in g.player(p).perms.clone() {
        let d = &mut g.perm_mut(m).data;
        if d.truthy(DataKey::Crewed) && d.get(DataKey::Crewed) != Some(&now) {
            d.set(DataKey::Anim, Val::Bool(false));
            d.set(DataKey::Crewed, Val::None);
        }
    }
}

// ================================================================== Shadowspear
/// {1}: opponents' permanents lose hexproof and indestructible until end of turn (before removal or combat)
fn shadowspear(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || post.is_none() || !can_pay(g, p, 1, "", false) || g.spear == Some(g.turn_stamp()) {
        return Ok(vec![]);
    }
    let prot: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| {
            !g.perm(m).phased
                && (crate::dsl::has_kw(g, m, "hexproof") || crate::dsl::has_kw(g, m, "indestructible"))
                && pval(g, m) >= 5.0
        })
        .collect();
    let removal = g.player(p).hand.iter().any(|&c| {
        let d = g.db.get(c);
        d.tags.has(Tag::Rem) && can_pay(g, p, d.generic + 1, &d.pips, false)
    });
    if prot.is_empty() || !removal {
        return Ok(vec![]);
    }
    let best = max_by(&prot, |m| pval(g, m)).unwrap();
    Ok(vec![Opt {
        utility: 1.5 + pval(g, best) / 3.0,
        label: "Shadowspear".into(),
        act: Some(Action::Ability { src, f: spear_go, arg: 0 }),
    }])
}

fn spear_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !can_pay(g, p, 1, "", false) {
        return Ok(false);
    }
    pay(g, p, 1, "", false)?;
    crate::glog!(g, "  {} activates Shadowspear", pname(g, p));
    if ability_window(g, p, Some(src), "opponents lose hexproof and indestructible", None, None)? {
        g.spear = Some(g.turn_stamp());
    }
    Ok(true)
}

// ================================================================== blocking rules
/// rules2.annex_life: Norn's Annex: {W/P} per attacker: pay W, or 2 life
pub fn annex_life(g: &mut Game, p: PlayerId, d: PlayerId, xs: Vec<PermId>) -> Res<Vec<PermId>> {
    if !g.player(d).perms.iter().any(|&m| !g.perm(m).phased && name_of(g, m) == "Norn's Annex") {
        return Ok(xs);
    }
    let mut out = vec![];
    for m in xs {
        if can_pay(g, p, 0, "W", false) {
            pay(g, p, 0, "W", false)?;
            out.push(m);
        } else if g.player(p).life > 12 {
            lose_life(g, p, 2, Some(p), "other", None)?;
            out.push(m);
        }
    }
    Ok(out)
}

/// rules2.aura_ward: ward {2} from Sheltered by Ghosts
pub fn aura_ward(g: &Game, m: PermId) -> u32 {
    if g.auras.is_empty() {
        return 0;
    }
    2 * auras_on(g, m).iter().filter(|&&a| name_of(g, a) == "Sheltered by Ghosts").count() as u32
}

// ================================================================== Glimpse of Nature, Chatterfang
/// rules2.glimpse_draw: Glimpse of Nature: each creature spell you cast this turn draws a card. (Python keeps the
/// turn in `p.glimpse`; here `glimpse` is the flag the engine checks and `glimpse_turn` the turn.)
pub fn glimpse_draw(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    if g.db.get(c).creature && g.player(p).glimpse_turn == Some(g.turn_stamp()) {
        draw(g, p, 1, false)?;
    }
    Ok(())
}

/// rules2.chatterfang_squirrels: Chatterfang makes that many 1/1 Squirrels whenever p makes tokens
pub fn chatterfang_squirrels(g: &mut Game, p: PlayerId, n: i32) -> Res {
    if n > 0 && g.player(p).perms.iter().any(|&m| !g.perm(m).phased && name_of(g, m) == "Chatterfang, Squirrel General")
    {
        g.no_fang = true;
        let spec =
            Tokens { color: Some(Colors::from_letters("G")), types: vec!["squirrel"], ..Tokens::new(n as u32, 1) };
        let r = make_tokens(g, p, spec);
        g.no_fang = false;
        r?;
    }
    Ok(())
}

// ================================================================== Legion's Landing
/// Adanto, the First Fort (Legion's Landing transformed: t1's attack trigger, when you attack with three or more
/// creatures): {2}{W}, {T}: a 1/1 lifelink Vampire (at end of turn).
/// Fix (Rust only): Python registers this on "Legion's Landing // Adanto, the First Fort", which is never a land, and
/// reads `p.adanto`, which nothing sets, so the flipped land never made a token. It is registered on the land Legion's
/// Landing becomes (t1.ADANTO), without the `p.adanto` check.
fn adanto(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_some() || g.land(l).tapped || !super::lands::can_pay_without(g, p, l, 2, "W") {
        return Ok(vec![]);
    }
    Ok(vec![plan(1.2, "Adanto token".into(), adanto_go, l.0 as i64)])
}

fn adanto_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId(arg as u32);
    if !g.player(p).lands.contains(&l) || g.land(l).tapped || !super::lands::pay_without(g, p, l, 2, "W")? {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    let cd = g.land(l).cd;
    if crate::engine::stack::ability_window_card(g, p, cd, "a 1/1 lifelink Vampire", None, None)? {
        let spec = Tokens {
            lifelink: true,
            color: Some(Colors::from_letters("W")),
            types: vec!["vampire"],
            ..Tokens::new(1, 1)
        };
        make_tokens(g, p, spec)?;
    }
    Ok(true)
}

// ================================================================== Hullbreaker Horror: bounce a spell
/// rules2.hullbreaker_counter: q controls Hullbreaker Horror and holds a cheap instant: casting it bounces c (the spell
/// returns to its hand)
pub fn hullbreaker_counter(g: &mut Game, q: PlayerId, c: CardId) -> Res<bool> {
    if !g.player(q).perms.iter().any(|&m| !g.perm(m).phased && name_of(g, m) == "Hullbreaker Horror") {
        return Ok(false);
    }
    let inst: Vec<CardId> = g
        .player(q)
        .hand
        .iter()
        .copied()
        .filter(|&x| {
            let d = g.db.get(x);
            let (gn, pips) = cost_of(g, q, x);
            d.instant && !d.tag(Tag::Ctr) && can_pay(g, q, gn, &pips, false) && castable(g, q, x, "hand")
        })
        .collect();
    let Some(x) = first_min(&inst, |x| (g.db.get(x).cmc, card_worth(g, q, x, false))) else { return Ok(false) };
    let (gn, pips) = cost_of(g, q, x);
    pay(g, q, gn, &pips, false)?;
    remove_card(&mut g.player_mut(q).hand, x);
    g.player_mut(q).gy.push(x);
    on_cast(g, q, x)?;
    crate::glog!(
        g,
        "    {} casts {}: Hullbreaker Horror returns {} to its owner's hand",
        pname(g, q),
        g.db.get(x).name,
        g.db.get(c).name
    );
    Ok(true)
}

// ================================================================== Sakura-Tribe Elder: chump, then sacrifice
/// after it blocks, it's sacrificed for a basic land
fn ste_block(
    g: &mut Game,
    src: Src,
    _p: PlayerId,
    _atk: &[PermId],
    d: PlayerId,
    assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    if g.perm(src).owner == d && assign.iter().any(|x| x.1 == src) && on(g, d, src) && g.humans.get(d).is_none() {
        die(g, src, "sac")?;
        land_ramp(g, d, 1, true)?;
        crate::glog!(g, "    {} sacrifices Sakura-Tribe Elder after blocking", pname(g, d));
    }
    Ok(())
}

/// at the end of an opponent's turn: sacrificed for a basic land
fn ste_eot(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if g.perm(src).owner != p || post.is_some() {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.5,
        label: "sacrifice Sakura-Tribe Elder".into(),
        act: Some(Action::Ability { src, f: ste_go, arg: 0 }),
    }])
}

fn ste_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !on(g, p, src) {
        return Ok(false);
    }
    die(g, src, "sac")?;
    let cd = g.perm(src).cd.unwrap();
    if ability_window_card(g, p, cd, "search for a basic land")? {
        land_ramp(g, p, 1, true)?;
    }
    Ok(true)
}

// ================================================================== Springbloom Druid, Knight of the White Orchid
const BASICS: [&str; 5] = ["Forest", "Island", "Plains", "Swamp", "Mountain"];

/// on entry: sacrifice a land for two basics tapped
fn springbloom(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m != src || g.player(o).lands.len() < 3 {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "sacrifice a land for two basics", None)? || g.player(o).lands.len() < 3 {
        return Ok(());
    }
    let lands = g.player(o).lands.clone();
    let l = first_min(&lands, |l| (!BASICS.contains(&&*g.db.get(g.land(l).cd).name), !g.land(l).tapped)).unwrap();
    crate::engine::turn::remove_land(g, o, l);
    let cd = g.land(l).cd;
    g.player_mut(o).gy.push(cd);
    land_ramp(g, o, 2, true)
}

/// a Plains onto the battlefield if an opponent has more lands
fn kotwo(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    let behind = |g: &Game| {
        let n = g.player(o).lands.len();
        g.opps(o).any(|q| g.player(q).lands.len() > n)
    };
    if behind(g) && trigger_window(g, o, Some(src), "search for a Plains", None)? && behind(g) {
        let cs: Vec<CardId> = searchable(g, o)
            .into_iter()
            .filter(|&x| {
                let d = g.db.get(x);
                d.land && (&*d.name == "Plains" || d.has_subtype("plains"))
            })
            .collect();
        if let Some(&x) = cs.first() {
            remove_card(&mut g.player_mut(o).library, x);
            shuffle_library(g, o);
            g.add_land(o, x, false);
        }
    }
    Ok(())
}

// ================================================================== sacrifice outlets' own effects
/// rules2._seer: Viscera Seer's and Woe Strider's sacrifice: scry 1 (common.SAC_OUTLET's effect for them)
pub fn seer(g: &mut Game, p: PlayerId, _src: PermId, _m: PermId) -> Res {
    crate::cardcode::scry(g, p, 1, false)
}

/// escape {3}{B}{B}, exile four other cards: returns with two +1/+1 counters
fn woe_escape(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || !can_pay(g, p, 3, "BB", false) || g.player(p).gy.iter().filter(|&&x| x != c).count() < 4 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.6,
        label: "escape Woe Strider".into(),
        act: Some(Action::Plan { f: woe_go, arg: c.0 as i64 }),
    }])
}

fn woe_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).gy.contains(&c) || !can_pay(g, p, 3, "BB", false) {
        return Ok(false);
    }
    pay(g, p, 3, "BB", false)?;
    exile_fodder(g, p, c, 4);
    remove_card(&mut g.player_mut(p).gy, c);
    crate::glog!(g, "  {} escapes Woe Strider", pname(g, p));
    on_cast(g, p, c)?;
    if counter_window(g, p, c, 3.0, vec![])? {
        enter(g, p, c, Enter { plus: 2, ..Enter::default() })?;
    } else {
        g.player_mut(p).exile.push(c);
    }
    Ok(true)
}

/// rules2.prot_vs: m has protection from creature `other` (Spirit Mantle / Unquestioned Authority: all creatures;
/// Baneslayer: Demons and Dragons): no damage from it
pub fn prot_vs(g: &Game, m: PermId, other: PermId) -> bool {
    if !g.auras.is_empty()
        && auras_on(g, m).iter().any(|&a| matches!(name_of(g, a), "Spirit Mantle" | "Unquestioned Authority"))
    {
        return true;
    }
    if super::rules::dovin_blocked(g, m) || super::rules::dovin_blocked(g, other) {
        return true; // Dovin: no damage to or from it
    }
    name_of(g, m) == "Baneslayer Angel" && (has_type(g, other, "demon") || has_type(g, other, "dragon"))
}

/// rules2.prot_unblockable: Baneslayer Angel can't be blocked by Demons and Dragons
pub fn prot_unblockable(g: &Game, b: PermId, a: PermId) -> bool {
    name_of(g, a) == "Baneslayer Angel" && (has_type(g, b, "demon") || has_type(g, b, "dragon"))
}

// ================================================================== helpers
/// rules2.ready_attackers: p's creatures that could attack now
fn ready_attackers(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            g.is_creature(m) && !x.tapped && !x.phased && !x.noatk && (!x.sick || crate::dsl::has_kw(g, m, "haste"))
        })
        .collect()
}

/// rules2.R_etb (partials.etb_val): what entering again is worth for m's card
fn etb_val(g: &Game, m: PermId) -> f64 {
    let Some(c) = g.perm(m).cd else { return 0.0 };
    let d = g.db.get(c);
    if d.has_dsl() {
        crate::dsl::etb_value(g, c) as f64
    } else if d.tag(Tag::Etb) {
        2.0
    } else {
        0.0
    }
}

/// rules.casts_isc (set by rules2): instants and sorceries p cast this turn
pub fn casts_isc(g: &Game, p: PlayerId) -> i32 {
    match &g.player(p).turn_casts {
        Some((st, cs)) if *st == g.turn_stamp() => {
            cs.iter().filter(|&&c| g.db.get(c).instant || g.db.get(c).sorcery).count() as i32
        }
        _ => 0,
    }
}

/// rules2.devotion: the coloured pips of `col` among p's permanents
pub fn devotion(g: &Game, p: PlayerId, col: char) -> usize {
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| !g.perm(m).phased)
        .filter_map(|&m| g.perm(m).cd)
        .map(|c| g.db.get(c).pips.chars().filter(|&x| x == col).count())
        .sum()
}

// ================================================================== crew: Esika's Chariot
/// crew 4 (tapped creatures with power 4 or more are kept), before the first combat when the attack is open
fn crew(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != g.perm(src).owner || g.perm(src).tapped {
        return Ok(());
    }
    let mut cands: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && !g.perm(m).tapped && !g.perm(m).phased && m != src)
        .collect();
    cands.sort_by(|&a, &b| {
        (pval(g, a), -epow(g, a)).partial_cmp(&(pval(g, b), -epow(g, b))).unwrap_or(std::cmp::Ordering::Equal)
    });
    let (mut power, mut pick) = (0, vec![]);
    for m in cands {
        if power >= 4 {
            break;
        }
        if g.perm(m).sick || epow(g, m) < 4 || g.perm(m).token {
            pick.push(m);
            power += epow(g, m);
        }
    }
    if power < 4 || !super::lands::open_attack(g, p, 4, false) {
        return Ok(());
    }
    for m in pick {
        g.perm_mut(m).tapped = true;
    }
    let now = turn_now(g);
    let x = g.perm_mut(src);
    x.data.set(DataKey::Anim, Val::Bool(true));
    (x.pow, x.tgh) = (4, 4);
    x.data.set(DataKey::Crewed, now);
    crate::glog!(g, "  {} crews Esika's Chariot", pname(g, p));
    Ok(())
}

/// attacking copies a token
fn chariot_copy(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    let toks = |g: &Game| -> Vec<PermId> {
        g.player(p)
            .perms
            .iter()
            .copied()
            .filter(|&m| g.perm(m).token && g.is_creature(m) && !g.perm(m).phased)
            .collect()
    };
    if atk.contains(&src) && !toks(g).is_empty() && trigger_window(g, p, Some(src), "copy a token", None)? {
        let ts = toks(g);
        if let Some(t) = max_by(&ts, |m| epow(g, m) as f64) {
            let x = g.perm(t);
            let spec = Tokens {
                tgh: Some(x.tgh),
                fly: x.fly,
                types: x.ttypes.clone(),
                color: Some(x.colors),
                ..Tokens::new(1, x.pow)
            };
            make_tokens(g, p, spec)?;
        }
    }
    Ok(vec![])
}

/// two 2/2 Cats on entry
fn chariot_cats(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m == src && trigger_window(g, o, Some(src), "create two 2/2 Cats", None)? {
        let spec =
            Tokens { tgh: Some(2), color: Some(Colors::from_letters("G")), types: vec!["cat"], ..Tokens::new(2, 2) };
        make_tokens(g, o, spec)?;
    }
    Ok(())
}

// ================================================================== flash in combat: Embercleave, The Wandering Emperor
/// rules2's Embercleave cost (engine.SELF_COST): {1} less per creature attacking this turn
pub fn embercleave_cost(g: &Game, p: PlayerId) -> i32 {
    let pl = g.player(p);
    if pl.attacking != Some(g.turn_stamp()) {
        return 0;
    }
    -(pl.perms.iter().filter(|&&m| g.is_creature(m) && g.perm(m).tapped && pl.attackers.contains(&m)).count() as i32)
}

/// flash, costs {1} less per attacking creature, attaches on entry: cast after attackers are declared
fn cleave_flash(g: &mut Game, c: CardId, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res {
    let st = g.turn_stamp();
    let pl = g.player_mut(p);
    pl.attacking = Some(st);
    pl.attackers = atk.to_vec();
    if !g.player(p).hand.contains(&c) || !castable(g, p, c, "hand") {
        return Ok(());
    }
    let (gn, pips) = cost_of(g, p, c);
    if !can_pay(g, p, gn, &pips, false) {
        return Ok(());
    }
    if atk.len() < 2 && atk.iter().map(|&m| epow(g, m)).sum::<i32>() < 5 {
        return Ok(());
    }
    remove_card(&mut g.player_mut(p).hand, c);
    let (gn, pips) = cost_of(g, p, c);
    pay(g, p, gn, &pips, false)?;
    crate::glog!(g, "  {} flashes in Embercleave", pname(g, p));
    on_cast(g, p, c)?;
    if !counter_window(g, p, c, 5.0, vec![])? {
        g.player_mut(p).gy.push(c);
        return Ok(());
    }
    let m = enter(g, p, c, Enter { was_cast: true, ..Enter::default() })?;
    if let Some(host) = max_by(atk, |a| epow(g, a) as f64) {
        g.perm_mut(m).attached = Some(host);
    }
    Ok(())
}

fn no_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

/// flashed in during an opponent's attack: -2 exiles the biggest tapped attacker (taken out of the attack)
fn emperor_flash(
    g: &mut Game,
    c: CardId,
    d: PlayerId,
    p: PlayerId,
    atk: &mut Vec<PermId>,
    _assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    if !g.player(d).hand.contains(&c) || !castable(g, d, c, "hand") {
        return Ok(());
    }
    let (gn, pips) = cost_of(g, d, c);
    if !can_pay(g, d, gn, &pips, false) {
        return Ok(());
    }
    let tapped: Vec<PermId> = atk.iter().copied().filter(|&a| on(g, p, a) && g.perm(a).tapped).collect();
    let Some(t) = max_by(&tapped, |a| pval(g, a)) else { return Ok(()) };
    if pval(g, t) < 4.0 {
        return Ok(());
    }
    pay(g, d, gn, &pips, false)?;
    remove_card(&mut g.player_mut(d).hand, c);
    crate::glog!(g, "  {} flashes in The Wandering Emperor", pname(g, d));
    on_cast(g, d, c)?;
    if !counter_window(g, d, c, 5.0, vec![])? {
        g.player_mut(d).gy.push(c);
        return Ok(());
    }
    let m = enter(g, d, c, Enter { was_cast: true, ..Enter::default() })?;
    let st = crate::state::TurnStamp { round: g.round, active: Some(d) };
    let x = g.perm_mut(m);
    x.loyalty = Some(x.loyalty.unwrap_or(3) - 2);
    x.loyalty_used = Some(st);
    x.data.set(DataKey::LoyaltyN, Val::Int(1));
    if let Some(i) = atk.iter().position(|&a| a == t) {
        atk.remove(i);
    }
    apply_removal(g, Some(d), t, "exile", None)?;
    gain(g, d, 2)
}

// ================================================================== evoke: Solitude
/// evoke by exiling a white card from hand (free, instant speed): exile the best opposing creature
fn solitude_evoke(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let (gn, pips) = cost_of(g, p, c);
    if can_pay(g, p, gn, &pips, false) && post.is_some() {
        return Ok(vec![]);
    }
    let whites = g.player(p).hand.iter().any(|&x| x != c && g.db.get(x).pips.contains('W'));
    let Some(t) = best_opp_creature(g, p, |_, _| true) else { return Ok(vec![]) };
    if !whites || pval(g, t) < 6.0 {
        return Ok(vec![]);
    }
    Ok(vec![plan(pval(g, t) - 4.0, "evoke Solitude".into(), solitude_go, pack(c.0 as u32, t.0))])
}

fn solitude_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (c, t) = unpack(arg);
    let (c, t) = (CardId(c as u16), PermId(t));
    if !g.player(p).hand.contains(&c) || !g.perm(t).on_bf {
        return Ok(false);
    }
    let whites: Vec<CardId> =
        g.player(p).hand.iter().copied().filter(|&x| x != c && g.db.get(x).pips.contains('W')).collect();
    let Some(x) = min_by(&whites, |x| card_worth(g, p, x, false)) else { return Ok(false) };
    remove_card(&mut g.player_mut(p).hand, x);
    g.player_mut(p).exile.push(x);
    remove_card(&mut g.player_mut(p).hand, c);
    crate::glog!(g, "  {} evokes Solitude (exiling {})", pname(g, p), g.db.get(x).name);
    on_cast(g, p, c)?;
    let m = enter(g, p, c, Enter { was_cast: true, ..Enter::default() })?;
    if on(g, p, m) {
        die(g, m, "sac")?;
    }
    Ok(true)
}

// ================================================================== dash: Ragavan
fn ragavan_dash(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) || !can_pay(g, p, 1, "R", false) || !castable(g, p, c, "hand") {
        return Ok(vec![]);
    }
    if !super::lands::open_attack(g, p, 2, false) {
        return Ok(vec![]);
    }
    Ok(vec![plan(2.5, "dash Ragavan".into(), ragavan_go, c.0 as i64)])
}

fn ragavan_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "R", false) {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    pay(g, p, 1, "R", false)?;
    crate::glog!(g, "  {} dashes Ragavan", pname(g, p));
    on_cast(g, p, c)?;
    if !counter_window(g, p, c, 3.0, vec![])? {
        g.player_mut(p).gy.push(c);
        return Ok(true);
    }
    let m = enter(g, p, c, Enter { was_cast: true, ..Enter::default() })?;
    g.perm_mut(m).sick = false;
    g.perm_mut(m).data.set(DataKey::Dash, Val::Bool(true));
    Ok(true)
}

/// combat damage: a Treasure and the defender's top card (its compiled abilities); dash {1}{R}: back to hand at the
/// end step
fn ragavan_back(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p == g.perm(src).owner
        && g.perm(src).data.truthy(DataKey::Dash)
        && on(g, p, src)
        && trigger_window(g, p, Some(src), "return it to hand (dash)", Some(1.0))?
        && on(g, p, src)
    {
        leave(g, src)?;
        let cd = g.perm(src).cd.unwrap();
        g.player_mut(p).hand.push(cd);
    }
    Ok(())
}

// ================================================================== casualty: Ob Nixilis, the Adversary
/// casualty X (a spare creature: a token copy with loyalty X)
fn obnix_casualty(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src || !g.last_cast_etb || g.perm(src).data.truthy(DataKey::Copy) {
        return Ok(());
    }
    let o = g.perm(src).owner;
    let fod: Vec<PermId> = g
        .player(o)
        .perms
        .iter()
        .copied()
        .filter(|&x| {
            g.is_creature(x)
                && !g.perm(x).is_cmd
                && x != src
                && (g.perm(x).token || pval(g, x) < 3.0)
                && epow(g, x) >= 2
        })
        .collect();
    let Some(x) = max_by(&fod, |x| epow(g, x) as f64) else { return Ok(()) };
    let n = epow(g, x);
    die(g, x, "sac")?; // casualty: a cost paid as it's cast
    obnix_copy(g, o, src, n)
}

/// casualty's trigger: copy the spell (a token copy with loyalty X)
fn obnix_copy(g: &mut Game, o: PlayerId, src: PermId, n: i32) -> Res {
    if !trigger_window(g, o, Some(src), &format!("copy it (loyalty {n})"), Some(5.0))? {
        return Ok(());
    }
    let cd = g.perm(src).cd.unwrap();
    if let Some(cp) = enter_token_copy(g, o, cd)? {
        g.perm_mut(cp).loyalty = Some(n);
        g.perm_mut(cp).data.set(DataKey::Copy, Val::Bool(true));
        crate::glog!(g, "    casualty {n}: a token copy of Ob Nixilis with loyalty {n}");
    }
    Ok(())
}

// ================================================================== overload: Winds of Abandon
fn winds_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let opp_value = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| g.is_creature(m))
        .map(|m| pval(g, m))
        .psum();
    if total_mana(g, p, false) >= 6 && opp_value >= 12.0 {
        62
    } else if best_opp_creature(g, p, |_, _| true).is_some() {
        40
    } else {
        0
    }
}

/// exiles an opposing creature (its controller searches a basic), or overloaded for {4}{W}{W} every opposing creature.
/// (Python's `p.winds_overload` is never set: the overload is paid whenever four more mana are there.)
fn winds(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let overload = total_mana(g, p, false) >= 4;
    if overload && can_pay(g, p, 4, "", false) {
        pay(g, p, 4, "", false)?;
        for q in g.opps(p).collect::<Vec<_>>() {
            let mut n = 0;
            for m in g.player(q).perms.clone() {
                if g.is_creature(m) && !g.perm(m).phased {
                    exile_perm(g, m)?;
                    n += 1;
                }
            }
            land_ramp(g, q, n, true)?;
        }
        return Ok("gy");
    }
    if let Some(t) = ctx.target.or_else(|| best_opp_creature(g, p, |_, _| true)) {
        let q = g.perm(t).owner;
        apply_removal(g, Some(p), t, "exile", Some(c))?;
        land_ramp(g, q, 1, true)?;
    }
    Ok("gy")
}

// ================================================================== level up / adapt / mana tokens
fn joraga_level(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.int(DataKey::Level)
}

/// level up {1}{G}: level 1 taps for GG; level 5 every Elf taps for GG
fn joraga(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != g.perm(src).owner || post != Some(false) {
        return Ok(vec![]);
    }
    let lvl = joraga_level(g, src);
    if lvl >= 5 || !can_pay(g, p, 1, "G", false) {
        return Ok(vec![]);
    }
    if lvl >= 1 && total_mana(g, p, false) < 6 {
        return Ok(vec![]);
    }
    let u = if lvl < 1 { 2.5 } else { 0.6 + 0.3 * super::t1::count_type(g, p, "elf", false) as f64 };
    Ok(vec![ability(u, format!("level up Joraga ({})", lvl + 1), src, joraga_go, 0)])
}

fn joraga_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !can_pay(g, p, 1, "G", false) {
        return Ok(false);
    }
    pay(g, p, 1, "G", false)?;
    if !ability_window(g, p, Some(src), "level up", None, None)? || !on(g, p, src) {
        return Ok(true);
    }
    let lvl = joraga_level(g, src) + 1;
    g.perm_mut(src).data.set(DataKey::Level, Val::Int(lvl));
    Ok(true)
}

fn joraga_mana(g: &Game, _p: PlayerId, m: PermId) -> u32 {
    if joraga_level(g, m) >= 1 { 2 } else { 0 }
}

/// level 5: Elves you control have {T}: add {G}{G}
fn joraga_elves(g: &Game, src: Src, p: PlayerId, units: &[Unit]) -> Vec<Unit> {
    if g.perm(src).owner != p || joraga_level(g, src) < 5 {
        return vec![];
    }
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            g.is_creature(m)
                && m != src
                && has_type(g, m, "elf")
                && !x.tapped
                && !x.sick
                && !units.iter().any(|u| u.src == Source::Perm(m))
        })
        .map(|m| Unit { src: Source::Perm(m), cols: Colors::from_letters("G"), amt: 2 })
        .collect()
}

/// adapt 3 for {3}{G}{G}
fn adapt(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != g.perm(src).owner || post.is_none() || g.perm(src).plus > 0 || !can_pay(g, p, 3, "GG", false) {
        return Ok(vec![]);
    }
    Ok(vec![ability(1.5, "adapt Incubation Druid".into(), src, adapt_go, 0)])
}

fn adapt_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if g.perm(src).plus > 0 || !can_pay(g, p, 3, "GG", false) {
        return Ok(false);
    }
    pay(g, p, 3, "GG", false)?;
    crate::glog!(g, "  {} adapts Incubation Druid", pname(g, p));
    if ability_window(g, p, Some(src), "adapt 3", None, None)? && on(g, p, src) && g.perm(src).plus <= 0 {
        g.perm_mut(src).plus += 3;
    }
    Ok(true)
}

/// rules2.mana_token('G'): n 1/1 Elf Druid tokens that tap for {G} (Freyalise's)
fn elf_druids(g: &mut Game, p: PlayerId, n: u32) -> Res {
    let spec = Tokens { color: Some(Colors::from_letters("G")), types: vec!["elf", "druid"], ..Tokens::new(n, 1) };
    for m in make_tokens(g, p, spec)? {
        g.perm_mut(m).data.set(DataKey::Manatok, Val::Str("G"));
    }
    Ok(())
}

/// rules2._freyalise_plus: Freyalise's +2 (it replaces t3's): two Elf Druids that tap for {G}
pub fn freyalise_plus(g: &mut Game, p: PlayerId, _src: PermId) -> Res {
    elf_druids(g, p, 2)
}

// ================================================================== harmonize, bargain, improvise, transmute, cycling
/// harmonize {X}{G}{G}{G}{G} (tapping a creature reduces it by its power): cast from the graveyard, then exile it
fn harmonize(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post != Some(false) {
        return Ok(vec![]);
    }
    let red = g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && !g.perm(m).tapped).map(|&m| epow(g, m)).max();
    let x = total_mana(g, p, false) as i32 + red.unwrap_or(0) - 4;
    if x < 3 || !can_pay(g, p, 0, "GGGG", false) {
        return Ok(vec![]);
    }
    Ok(vec![plan(1.0 + x as f64 / 2.0, "harmonize Nature's Rhythm".into(), harmonize_go, c.0 as i64)])
}

fn harmonize_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).gy.contains(&c) {
        return Ok(false);
    }
    let untapped: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| g.is_creature(m) && !g.perm(m).tapped).collect();
    let tapper = max_by(&untapped, |m| epow(g, m) as f64);
    let r = tapper.map_or(0, |m| epow(g, m));
    if let Some(m) = tapper {
        g.perm_mut(m).tapped = true;
    }
    let xx = total_mana(g, p, false) as i32 - 4 + r;
    let gn = (xx - r).max(0) as u32;
    if !can_pay(g, p, gn, "GGGG", false) {
        return Ok(false);
    }
    remove_card(&mut g.player_mut(p).gy, c);
    pay(g, p, gn, "GGGG", false)?;
    g.player_mut(p).exile.push(c);
    crate::glog!(g, "  {} harmonizes Nature's Rhythm (X={})", pname(g, p), xx);
    on_cast(g, p, c)?;
    if !counter_window(g, p, c, 5.0, vec![])? {
        return Ok(true);
    }
    super::t3::put_creature(g, p, |g, y| g.db.get(y).cmc as i32 <= xx, true)?;
    Ok(true)
}

fn fifty(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    50
}

/// tutor to hand; bargained (a token or spare artifact/enchantment sacrificed), a found card with mana value 4 or less
/// is cast free
fn beseech(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let fod: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            g.perm(m).token
                || ((card_is(g, m, Types::ARTIFACT) || card_is(g, m, Types::ENCHANTMENT)) && pval(g, m) < 2.0)
        })
        .collect();
    let before = g.player(p).hand.clone();
    tutor(g, p, "any")?;
    let new: Vec<CardId> = g.player(p).hand.iter().copied().filter(|x| !before.contains(x)).collect();
    if let (Some(m), Some(&x)) = (min_by(&fod, |m| pval(g, m)), new.first())
        && g.db.get(x).cmc <= 4
        && !g.db.get(x).land
        && castable(g, p, x, "hand")
    {
        if g.perm(m).token {
            leave(g, m)?;
        } else {
            die(g, m, "sac")?;
        }
        remove_card(&mut g.player_mut(p).hand, x);
        crate::glog!(g, "    bargained: {} is cast free", g.db.get(x).name);
        cast_card(g, p, x, "lib", Ctx::default())?;
    }
    Ok("gy")
}

/// t5.IC_COMBO_ART: the artifacts of the Isochron / Power Artifact combos (Whir of Invention finds them first)
const IC_COMBO_ART: [&str; 6] = [
    "Isochron Scepter",
    "Power Artifact",
    "Basalt Monolith",
    "Grim Monolith",
    "Mycosynth Lattice",
    "Rings of Brighthearth",
];

fn whir_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if total_mana(g, p, false) >= 5 { 55 } else { 0 }
}

/// X = spare mana (improvise: untapped artifacts help pay); the best artifact with mana value X or less
fn whir(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let mut x = ctx.x;
    let arts: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let y = g.perm(m);
            !y.tapped
                && ((card_is(g, m, Types::ARTIFACT) && !y.cd.is_some_and(|c| g.db.get(c).tag(Tag::Rock)))
                    || (y.token && y.ttypes.contains(&"artifact")))
        })
        .collect();
    for &m in &arts {
        g.perm_mut(m).tapped = true;
    }
    x += arts.len() as i32;
    let cs: Vec<CardId> = searchable(g, p)
        .into_iter()
        .filter(|&y| g.db.get(y).types.has(Types::ARTIFACT) && g.db.get(y).cmc as i32 <= x)
        .collect();
    if let Some(y) =
        first_max(&cs, |y| (IC_COMBO_ART.contains(&&*g.db.get(y).name), card_worth(g, p, y, false), g.db.get(y).cmc))
    {
        remove_card(&mut g.player_mut(p).library, y);
        shuffle_library(g, p);
        enter(g, p, y, Enter::default())?;
    }
    Ok("gy")
}

fn transmute_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let cheap = g.player(p).perms.iter().any(|&m| card_is(g, m, Types::ARTIFACT) && pval(g, m) < 3.0);
    if cheap { 50 } else { 0 }
}

/// sacrifice an artifact, search an artifact: onto the battlefield if you pay the difference in mana value
fn transmute(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let arts: Vec<PermId> =
        g.player(p).perms.iter().copied().filter(|&m| card_is(g, m, Types::ARTIFACT) && !g.perm(m).phased).collect();
    let Some(s) = min_by(&arts, |m| pval(g, m)) else { return Ok("gy") };
    let mv = g.db.get(g.perm(s).cd.unwrap()).cmc;
    die(g, s, "sac")?;
    let cs: Vec<CardId> = searchable(g, p).into_iter().filter(|&y| g.db.get(y).types.has(Types::ARTIFACT)).collect();
    if cs.is_empty() {
        return Ok("gy");
    }
    let wish = crate::ai::plans::wish_list(g, p);
    let best = first_max(&cs, |y| {
        let d = g.db.get(y);
        (wish.contains(&y) && can_pay(g, p, d.cmc.saturating_sub(mv), "", false), card_worth(g, p, y, false))
    })
    .unwrap();
    remove_card(&mut g.player_mut(p).library, best);
    shuffle_library(g, p);
    let diff = g.db.get(best).cmc.saturating_sub(mv);
    if can_pay(g, p, diff, "", false) {
        pay(g, p, diff, "", false)?;
        enter(g, p, best, Enter::default())?;
    } else {
        g.player_mut(p).gy.push(best);
    }
    Ok("gy")
}

/// cycling {2} when there is nothing worth returning
fn unearth_cycle(g: &mut Game, c: CardId, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    let target = g.player(p).gy.iter().any(|&x| g.db.get(x).creature && g.db.get(x).cmc <= 3);
    if target || !can_pay(g, p, 2, "", false) {
        return Ok(vec![]);
    }
    Ok(vec![plan(0.8, "cycle Unearth".into(), unearth_go, c.0 as i64)])
}

fn unearth_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
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

// ================================================================== Wishclaw Talisman: the opponent uses it back
fn wishes(g: &Game, src: PermId) -> i64 {
    g.perm(src).data.get(DataKey::Wishes).map_or(3, Val::int)
}

/// three wishes: {1},{T} tutor (on your turn), then the most threatening opponent gains control and uses it on theirs
fn wishclaw(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != g.perm(src).owner || g.perm(src).tapped || !can_pay(g, p, 1, "", false) || wishes(g, src) <= 0 {
        return Ok(vec![]);
    }
    if g.active != Some(p) && post.is_some() {
        return Ok(vec![]);
    }
    let u = if crate::ai::plans::wish_list(g, p).is_empty() { 1.5 } else { 3.5 };
    Ok(vec![ability(u, "Wishclaw Talisman".into(), src, wishclaw_go, 0)])
}

fn wishclaw_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if g.perm(src).tapped || !can_pay(g, p, 1, "", false) {
        return Ok(false);
    }
    pay(g, p, 1, "", false)?;
    g.perm_mut(src).tapped = true;
    let w = wishes(g, src) - 1;
    g.perm_mut(src).data.set(DataKey::Wishes, Val::Int(w));
    if !ability_window(g, p, Some(src), "search for a card", Some(7.0), None)? {
        return Ok(true);
    }
    tutor(g, p, "any")?;
    if let Some(q) = super::t3::top_threat(g, p)
        && on(g, p, src)
        && wishes(g, src) > 0
    {
        g.player_mut(p).perms.retain(|&x| x != src);
        g.perm_mut(src).owner = q;
        g.player_mut(q).perms.push(src);
        g.perm_mut(src).tapped = false;
        g.bf_ver += 1;
        crate::glog!(g, "    {} gains control of Wishclaw Talisman", pname(g, q));
    }
    Ok(true)
}

// ================================================================== Aura of Silence, Soul-Guide Lantern
fn silence_target(g: &Game, p: PlayerId) -> Option<PermId> {
    best_opp_nonland(g, p, |g, m| card_is(g, m, Types::ARTIFACT) || card_is(g, m, Types::ENCHANTMENT))
}

/// sacrifice: destroy a valuable artifact or enchantment
fn aura_silence(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if p != g.perm(src).owner {
        return Ok(vec![]);
    }
    let Some(t) = silence_target(g, p) else { return Ok(vec![]) };
    if pval(g, t) < 5.0 {
        return Ok(vec![]);
    }
    let label = format!("sacrifice Aura of Silence -> {}", g.perm(t).name);
    Ok(vec![ability(pval(g, t) - 3.0, label, src, aura_silence_go, t.0 as i64)])
}

fn aura_silence_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    if !on(g, p, src) || !g.perm(t).on_bf {
        return Ok(false);
    }
    die(g, src, "sac")?;
    let cd = g.perm(src).cd.unwrap();
    let name = format!("destroy {}", g.perm(t).name);
    if crate::engine::stack::ability_window_card(g, p, cd, &name, None, Some(t))? && g.perm(t).on_bf {
        apply_removal(g, Some(p), t, "destroy", None)?;
    }
    Ok(true)
}

/// exiles a card from an opponent's graveyard on entry
fn lantern_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m != src || !g.opps(o).any(|q| !g.player(q).gy.is_empty()) {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "exile a card from a graveyard", None)? {
        return Ok(());
    }
    let cands: Vec<(PlayerId, CardId)> = g.opps(o).flat_map(|q| g.player(q).gy.iter().map(move |&x| (q, x))).collect();
    if let Some((q, x)) = first_max(&cands, |(_, x)| {
        let d = g.db.get(x);
        (d.creature, d.bomb, d.cmc)
    }) {
        remove_card(&mut g.player_mut(q).gy, x);
        g.player_mut(q).exile.push(x);
    }
    Ok(())
}

/// sacrificed for a card when no opposing graveyard holds a creature worth exiling
fn lantern_draw(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != g.perm(src).owner || post.is_none() || !can_pay(g, p, 1, "", false) {
        return Ok(vec![]);
    }
    if g.opps(p).flat_map(|q| g.player(q).gy.iter()).any(|&x| g.db.get(x).creature && g.db.get(x).bomb >= 5) {
        return Ok(vec![]);
    }
    if total_mana(g, p, false) < 4 {
        return Ok(vec![]);
    }
    Ok(vec![ability(0.7, "Soul-Guide Lantern draw".into(), src, lantern_go, 0)])
}

fn lantern_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !on(g, p, src) || !can_pay(g, p, 1, "", false) {
        return Ok(false);
    }
    pay(g, p, 1, "", false)?;
    die(g, src, "sac")?;
    let cd = g.perm(src).cd.unwrap();
    if ability_window_card(g, p, cd, "draw a card")? {
        draw(g, p, 1, false)?;
    }
    Ok(true)
}

// ================================================================== blocking rules
/// Silent Arbiter: no more than one creature can block each combat
fn arbiter_block(
    g: &mut Game,
    _src: Src,
    _p: PlayerId,
    _atk: &[PermId],
    _d: PlayerId,
    assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    if assign.len() > 1 {
        let keep = first_max(assign, |(a, _)| pval(g, a)).unwrap();
        assign.clear();
        assign.push(keep);
    }
    Ok(())
}

/// Archangel of Tithes: while it attacks, creatures can't block unless their controller pays {1} each
fn tithes_block(
    g: &mut Game,
    src: Src,
    p: PlayerId,
    atk: &[PermId],
    d: PlayerId,
    assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    if p != g.perm(src).owner || !atk.contains(&src) {
        return Ok(());
    }
    let mut kept = vec![];
    for (a, b) in assign.drain(..).collect::<Vec<_>>() {
        if can_pay(g, d, 1, "", false) {
            pay(g, d, 1, "", false)?;
            kept.push((a, b));
        }
    }
    *assign = kept;
    Ok(())
}

/// Norn's Annex: its {W/P} per attacker is annex_life's (attack_tax 0)
fn annex_tax(_g: &Game, _src: Src, _attacker: PlayerId, _d: PlayerId) -> Option<i32> {
    Some(0)
}

// ================================================================== graveyard triggers: Syr Konrad
/// a creature card milled or discarded deals 1 to each opponent
fn konrad_mill(g: &mut Game, src: Src, _p: PlayerId, cards: &[CardId]) -> Res {
    let n = cards.iter().filter(|&&c| g.db.get(c).creature).count() as i32;
    let o = g.perm(src).owner;
    if n > 0 && trigger_window(g, o, Some(src), &format!("{n} damage to each opponent"), None)? {
        for q in g.opps(o).collect::<Vec<_>>() {
            lose_life(g, q, n, Some(o), "triggers", None)?;
        }
    }
    Ok(())
}

// ================================================================== Glimpse of Nature
fn glimpse_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let cheap = g.player(p).hand.iter().filter(|&&x| g.db.get(x).creature && g.db.get(x).cmc <= 2).count();
    if cheap >= 2 { 55 } else { 0 }
}

/// this turn, each creature spell you cast draws a card
fn glimpse(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let st = g.turn_stamp();
    let pl = g.player_mut(p);
    pl.glimpse = true;
    pl.glimpse_turn = Some(st);
    Ok("gy")
}

// ================================================================== Chatterfang
fn squirrels(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).token && g.perm(m).ttypes.contains(&"squirrel")).collect()
}

/// {B}, sacrifice X Squirrels: target creature gets +X/-X
fn fang(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if p != g.perm(src).owner || !can_pay(g, p, 0, "B", false) {
        return Ok(vec![]);
    }
    let n = squirrels(g, p).len() as i32;
    let Some(t) = best_opp_creature(g, p, |g, m| etgh(g, m) <= n) else { return Ok(vec![]) };
    if pval(g, t) < 4.0 {
        return Ok(vec![]);
    }
    let x = etgh(g, t);
    let label = format!("Chatterfang -> {}", g.perm(t).name);
    Ok(vec![ability(pval(g, t) - 0.5 * x as f64, label, src, fang_go, pack(t.0, x.max(0) as u32))])
}

fn fang_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let (t, x) = unpack(arg);
    let (t, x) = (PermId(t), x as i32);
    if !g.perm(t).on_bf || !can_pay(g, p, 0, "B", false) {
        return Ok(false);
    }
    pay(g, p, 0, "B", false)?;
    for m in squirrels(g, p).into_iter().take(x.max(0) as usize) {
        die(g, m, "sac")?;
    }
    let name = format!("+{x}/-{x} to {}", g.perm(t).name);
    if !ability_window(g, p, Some(src), &name, None, Some(t))? || !g.perm(t).on_bf {
        return Ok(true);
    }
    if !untargetable(g, t) {
        super::t3::eot(g, t, x, -x);
        if etgh(g, t) <= 0 {
            die(g, t, "sba")?;
        }
    }
    Ok(true)
}

// ================================================================== Grist: repeat on Insects
/// rules2._grist_plus: Grist's +1 (it replaces t3's): an Insect and mill one, again while the milled card is an Insect
/// (a loyalty counter each time)
pub fn grist_plus(g: &mut Game, p: PlayerId, src: PermId) -> Res {
    loop {
        let spec = Tokens { color: Some(Colors::from_letters("G")), types: vec!["insect"], ..Tokens::new(1, 1) };
        make_tokens(g, p, spec)?;
        let Some(c) = g.player_mut(p).library.pop() else { return Ok(()) };
        g.player_mut(p).gy.push(c);
        if !g.db.get(c).has_subtype("insect") {
            return Ok(());
        }
        let x = g.perm_mut(src);
        x.loyalty = Some(x.loyalty.unwrap_or(0) + 1);
    }
}

// ================================================================== Detention Sphere, Eldrazi Displacer, Charming Prince
fn dsphere_target(g: &Game, o: PlayerId) -> Option<PermId> {
    best_opp_nonland(g, o, |g, x| !g.perm(x).cd.is_some_and(|c| g.db.get(c).land))
}

/// exiles a nonland permanent and every other one with its name (tokens are gone for good); they return when it leaves
fn dsphere(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if dsphere_target(g, o).is_none() {
        return Ok(());
    }
    if !trigger_window(g, o, Some(src), "exile a nonland permanent and all with its name", Some(5.0))? {
        return Ok(());
    }
    let Some(t) = dsphere_target(g, o) else { return Ok(()) };
    let name = g.perm(t).name;
    let same: Vec<PermId> = g
        .opps(o)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&x| g.perm(x).name == name && !g.perm(x).phased)
        .collect();
    g.perm_mut(src).data.set(DataKey::Held, Val::List(vec![]));
    for x in same {
        if g.perm(x).token {
            leave(g, x)?;
        } else {
            let (cd, q) = (g.perm(x).cd.unwrap(), g.perm(x).owner);
            if let Some(Val::List(h)) = g.perm_mut(src).data.get_mut(DataKey::Held) {
                h.push(Val::List(vec![Val::Card(cd), Val::Player(q)]));
            }
            exile_perm(g, x)?;
        }
    }
    crate::glog!(g, "    Detention Sphere exiles every {name}");
    Ok(())
}

fn dsphere_back(g: &mut Game, _src: Src, m: PermId) -> Res {
    let held = match g.perm(m).data.get(DataKey::Held) {
        Some(Val::List(h)) => h.clone(),
        _ => vec![],
    };
    for v in held {
        if let Val::List(x) = v
            && let [Val::Card(cd), Val::Player(q)] = x.as_slice()
            && g.player(*q).alive
            && remove_card(&mut g.player_mut(*q).exile, *cd)
        {
            enter(g, *q, *cd, Enter::default())?;
        }
    }
    Ok(())
}

/// {2}{C}: blink an opponent's token (it's gone)
fn displacer(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != g.perm(src).owner || post.is_none() || !can_pay(g, p, 3, "", false) {
        return Ok(vec![]);
    }
    let toks: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| g.perm(m).token && g.is_creature(m) && !untargetable(g, m))
        .collect();
    let Some(t) = max_by(&toks, |m| epow(g, m) as f64) else { return Ok(vec![]) };
    if epow(g, t) < 3 {
        return Ok(vec![]);
    }
    let u = 1.0 + 0.4 * epow(g, t) as f64;
    Ok(vec![ability(u, "Eldrazi Displacer on a token".into(), src, displacer_go, t.0 as i64)])
}

fn displacer_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    if !g.perm(t).on_bf || !can_pay(g, p, 3, "", false) {
        return Ok(false);
    }
    pay(g, p, 3, "", false)?;
    let name = format!("blink {}", g.perm(t).name);
    if ability_window(g, p, Some(src), &name, None, Some(t))? && g.perm(t).on_bf {
        leave(g, t)?;
        crate::glog!(g, "  {} blinks {} with Eldrazi Displacer (a token: gone)", pname(g, p), g.perm(t).name);
    }
    Ok(true)
}

/// blinks your best ETB creature until the end step, else 3 life
fn prince(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if !trigger_window(g, o, Some(src), "blink a creature or gain 3 life", None)? {
        return Ok(());
    }
    let cs: Vec<PermId> = g
        .player(o)
        .perms
        .iter()
        .copied()
        .filter(|&x| x != src && g.is_creature(x) && !g.perm(x).token && g.perm(x).cd.is_some() && etb_val(g, x) > 0.0)
        .collect();
    if let Some(t) = max_by(&cs, |x| etb_val(g, x)) {
        leave(g, t)?;
        let cd = g.perm(t).cd.unwrap();
        g.player_mut(o).oath_return.push(cd);
        crate::glog!(g, "    Charming Prince exiles {} until the end step", g.perm(t).name);
    } else {
        gain(g, o, 3)?;
    }
    Ok(())
}

/// the blinked creature returns at the end step (any player's: Python doesn't check whose)
fn prince_back(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    let o = g.perm(src).owner;
    for cd in g.player(o).oath_return.clone() {
        enter(g, o, cd, Enter::default())?;
    }
    g.player_mut(o).oath_return.clear();
    Ok(())
}

// ================================================================== pumps: Battle Cry Goblin, Purphoros, Scourge, Resplendent, Rionya
/// {1}{R}: Goblins +1/+0 before a wide attack
fn bcg_pump(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != g.perm(src).owner || post != Some(false) || !can_pay(g, p, 1, "R", false) {
        return Ok(vec![]);
    }
    let gobs = ready_attackers(g, p).into_iter().filter(|&m| has_type(g, m, "goblin")).count();
    if gobs < 4 || g.player(p).bcg_turn == Some(g.turn_stamp()) {
        return Ok(vec![]);
    }
    Ok(vec![ability(0.3 * gobs as f64, "Battle Cry Goblin pump".into(), src, bcg_go, 0)])
}

fn bcg_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !can_pay(g, p, 1, "R", false) {
        return Ok(false);
    }
    pay(g, p, 1, "R", false)?;
    g.player_mut(p).bcg_turn = Some(g.turn_stamp());
    if !ability_window(g, p, Some(src), "Goblins get +1/+0", None, None)? {
        return Ok(true);
    }
    for m in g.player(p).perms.clone() {
        if g.is_creature(m) && has_type(g, m, "goblin") {
            super::t3::eot(g, m, 1, 0);
        }
    }
    Ok(true)
}

/// rules2.purph_update: Purphoros is a creature only with devotion to red 5 or more
fn purph_update(g: &mut Game, src: PermId) {
    let on_ = devotion(g, g.perm(src).owner, 'R') >= 5;
    g.perm_mut(src).data.set(DataKey::Anim, Val::Bool(on_));
}

/// 2 damage to each opponent per creature entering (the devotion check is bookkeeping, before the window)
fn purph_on(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        let x = g.perm_mut(src);
        (x.pow, x.tgh) = (6, 5);
    }
    purph_update(g, src);
    let o = g.perm(src).owner;
    if g.perm(m).owner == o
        && g.is_creature(m)
        && m != src
        && trigger_window(g, o, Some(src), "2 damage to each opponent", None)?
    {
        for q in g.opps(o).collect::<Vec<_>>() {
            lose_life(g, q, 2, Some(o), "triggers", Some(true))?;
        }
    }
    Ok(())
}

/// {2}{R}: creatures +1/+0 before a wide attack
fn purph_pump(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    purph_update(g, src);
    if p != g.perm(src).owner || post != Some(false) || !can_pay(g, p, 2, "R", false) {
        return Ok(vec![]);
    }
    let ready = ready_attackers(g, p).len();
    if ready < 4 || g.player(p).purph_turn == Some(g.turn_stamp()) {
        return Ok(vec![]);
    }
    Ok(vec![ability(0.3 * ready as f64, "Purphoros pump".into(), src, purph_go, 0)])
}

fn purph_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !can_pay(g, p, 2, "R", false) {
        return Ok(false);
    }
    pay(g, p, 2, "R", false)?;
    g.player_mut(p).purph_turn = Some(g.turn_stamp());
    if !ability_window(g, p, Some(src), "creatures get +1/+0", None, None)? {
        return Ok(true);
    }
    for m in g.player(p).perms.clone() {
        if g.is_creature(m) {
            super::t3::eot(g, m, 1, 0);
        }
    }
    Ok(true)
}

fn purph_ind(_g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    kw == "indestructible" && m == src
}

/// {R}: +1/+0 before an open attack (firebreathing)
fn scourge_fire(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if p != x.owner || post != Some(false) || x.tapped || x.sick || total_mana(g, p, false) < 4 {
        return Ok(vec![]);
    }
    if !super::lands::open_attack(g, p, epow(g, src), true) {
        return Ok(vec![]);
    }
    let n = total_mana(g, p, false) as i32 - 2;
    Ok(vec![ability(0.4 * n as f64, "Scourge of Valkas firebreathing".into(), src, scourge_go, n as i64)])
}

fn scourge_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let n = arg as i32;
    let gn = (n - 1).max(0) as u32;
    if !can_pay(g, p, gn, "R", false) {
        return Ok(false);
    }
    pay(g, p, gn, "R", false)?;
    if ability_window(g, p, Some(src), &format!("+{n}/+0"), None, None)? && on(g, p, src) {
        super::t3::eot(g, src, n, 0);
    }
    Ok(true)
}

/// {3}{W}{W}{W}: +2/+0 and lifelink
fn resplendent_pump(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != g.perm(src).owner || post != Some(false) || g.perm(src).tapped || !can_pay(g, p, 3, "WWW", false) {
        return Ok(vec![]);
    }
    Ok(vec![ability(1.2, "Resplendent Angel pump".into(), src, resplendent_go, 0)])
}

fn resplendent_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !can_pay(g, p, 3, "WWW", false) {
        return Ok(false);
    }
    pay(g, p, 3, "WWW", false)?;
    if ability_window(g, p, Some(src), "+2/+0 and lifelink", None, None)? && on(g, p, src) {
        super::t3::eot(g, src, 2, 0);
        eot_kw(g, src, "lifelink");
    }
    Ok(true)
}

fn rionya_cands(g: &Game, p: PlayerId, src: PermId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.is_creature(m) && m != src && g.perm(m).cd.is_some() && !g.perm(m).phased)
        .collect()
}

/// each combat: 1 + (instants and sorceries cast this turn) hasty token copies of a creature, exiled at end step
fn rionya_x(g: &mut Game, src: Src, p: PlayerId) -> Res<Vec<PermId>> {
    if p != g.perm(src).owner || rionya_cands(g, p, src).is_empty() {
        return Ok(vec![]);
    }
    if !trigger_window(g, p, Some(src), "create hasty token copies of a creature", None)? {
        return Ok(vec![]);
    }
    let cs = rionya_cands(g, p, src);
    let Some(t) = max_by(&cs, |m| epow(g, m) as f64 + etb_val(g, m)) else { return Ok(vec![]) };
    let x = 1 + casts_isc(g, p);
    let cd = g.perm(t).cd.unwrap();
    let mut out = vec![];
    for _ in 0..x {
        if let Some(tok) = enter_token_copy(g, p, cd)? {
            g.perm_mut(tok).sick = false;
            g.perm_mut(tok).data.set(DataKey::Rionya, Val::Bool(true));
            out.push(tok);
        }
    }
    crate::glog!(g, "    Rionya makes {} hasty copies of {}", out.len(), g.perm(t).name);
    Ok(out)
}

fn rionya_exile(g: &mut Game, src: Src, p: PlayerId) -> Res {
    let copies = |g: &Game| -> Vec<PermId> {
        g.player(p).perms.iter().copied().filter(|&m| g.perm(m).data.truthy(DataKey::Rionya)).collect()
    };
    if p == g.perm(src).owner
        && !copies(g).is_empty()
        && trigger_window(g, p, Some(src), "exile the token copies", Some(1.0))?
    {
        for m in copies(g) {
            leave(g, m)?;
        }
    }
    Ok(())
}

// ================================================================== Stoneforge Mystic, Gilded Goose, Legion's Landing, Destiny Spinner
/// {1}{W}, {T}: put an Equipment card from your hand onto the battlefield
fn sfm_put(g: &mut Game, src: Src, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if p != x.owner || x.tapped || x.sick || !can_pay(g, p, 1, "W", false) {
        return Ok(vec![]);
    }
    let eq: Vec<CardId> = g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).has_subtype("equipment")).collect();
    let Some(c) = max_by(&eq, |c| g.db.get(c).cmc as f64) else { return Ok(vec![]) };
    let cmc = g.db.get(c).cmc;
    if cmc <= 2 {
        return Ok(vec![]);
    }
    let label = format!("Stoneforge Mystic ({})", g.db.get(c).name);
    Ok(vec![ability(1.0 + cmc as f64 / 2.0, label, src, sfm_go, c.0 as i64)])
}

fn sfm_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || g.perm(src).tapped || !can_pay(g, p, 1, "W", false) {
        return Ok(false);
    }
    pay(g, p, 1, "W", false)?;
    g.perm_mut(src).tapped = true;
    let name = format!("put {} onto the battlefield", g.db.get(c).name);
    if !ability_window(g, p, Some(src), &name, Some(5.0), None)? || !g.player(p).hand.contains(&c) {
        return Ok(true);
    }
    remove_card(&mut g.player_mut(p).hand, c);
    enter(g, p, c, Enter::default())?;
    crate::glog!(g, "  {} puts {} onto the battlefield with Stoneforge Mystic", pname(g, p), g.db.get(c).name);
    Ok(true)
}

/// {1}{G}, {T}: a Food (at end of turn)
fn goose_food(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != g.perm(src).owner || g.perm(src).tapped || !can_pay(g, p, 1, "G", false) || g.player(p).foods > 0 {
        return Ok(vec![]);
    }
    if post.is_some() {
        return Ok(vec![]);
    }
    Ok(vec![ability(0.9, "Gilded Goose: Food".into(), src, goose_go, 0)])
}

fn goose_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if g.perm(src).tapped || !can_pay(g, p, 1, "G", false) {
        return Ok(false);
    }
    pay(g, p, 1, "G", false)?;
    g.perm_mut(src).tapped = true;
    if ability_window(g, p, Some(src), "a Food", None, None)? {
        make_artifact_tokens(g, p, "Food", 1)?;
    }
    Ok(true)
}

/// {3}{G}: a land becomes an X/X Elemental with trample and haste (X = enchantments you control)
fn spinner(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != g.perm(src).owner || post != Some(false) || !can_pay(g, p, 3, "G", false) {
        return Ok(vec![]);
    }
    let x = spinner_x(g, p);
    if x < 4 || g.player(p).lands.is_empty() {
        return Ok(vec![]);
    }
    if !super::lands::open_attack(g, p, x, false) {
        return Ok(vec![]);
    }
    Ok(vec![ability(0.5 * x as f64, "Destiny Spinner animates a land".into(), src, spinner_go, x as i64)])
}

fn spinner_x(g: &Game, p: PlayerId) -> i32 {
    g.player(p).perms.iter().filter(|&&m| card_is(g, m, Types::ENCHANTMENT) && !g.perm(m).phased).count() as i32
}

fn spinner_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let x = arg as i32;
    if !can_pay(g, p, 3, "G", false) || g.player(p).lands.len() < 2 {
        return Ok(false);
    }
    pay(g, p, 3, "G", false)?;
    if !ability_window(g, p, Some(src), "animate a land", None, None)? || g.player(p).lands.is_empty() {
        return Ok(true);
    }
    let lands = g.player(p).lands.clone();
    let l = lands.iter().copied().find(|&l| g.land(l).tapped).unwrap_or(lands[0]);
    super::lands::animate(g, p, l, x, x, &["trample", "haste"], false);
    Ok(true)
}

// ================================================================== Valakut Exploration, Finale, Farseek, Ichormoon
/// at your end step, each card exiled with it that you didn't play goes to the graveyard: 1 damage each
fn valakut_end(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != g.perm(src).owner {
        return Ok(());
    }
    let held = g.player(p).valakut_cards.clone();
    if held.iter().any(|c| g.player(p).hand.contains(c))
        && !trigger_window(g, p, Some(src), "unplayed cards to the graveyard, 1 damage each", None)?
    {
        return Ok(());
    }
    let held = g.player(p).valakut_cards.clone();
    let mut left = 0;
    for c in held {
        if remove_card(&mut g.player_mut(p).hand, c) {
            g.player_mut(p).gy.push(c);
            left += 1;
        }
    }
    if left > 0 {
        for q in g.opps(p).collect::<Vec<_>>() {
            lose_life(g, q, left, Some(p), "burn", Some(true))?;
        }
    }
    g.player_mut(p).valakut_cards.clear();
    Ok(())
}

/// landfall: exile the top card, playable until end of turn (held in hand)
fn valakut_land(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != g.perm(src).owner || g.player(p).library.is_empty() {
        return Ok(());
    }
    if !trigger_window(g, p, Some(src), "exile the top card", None)? {
        return Ok(());
    }
    let Some(c) = g.player_mut(p).library.pop() else { return Ok(()) };
    let pl = g.player_mut(p);
    pl.hand.push(c);
    pl.seen.insert(c);
    pl.valakut_cards.push(c);
    Ok(())
}

fn finale_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if total_mana(g, p, false) >= 5 { 60 } else { 0 }
}

/// X = spare mana: a creature with mana value X or less from library or graveyard onto the battlefield; X >= 10:
/// creatures +X/+X and haste
fn finale(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let x = ctx.x;
    let gy: Vec<CardId> =
        g.player(p).gy.iter().copied().filter(|&y| g.db.get(y).creature && g.db.get(y).cmc as i32 <= x).collect();
    let got = super::t3::put_creature(g, p, |g, y| g.db.get(y).cmc as i32 <= x, true)?;
    if got.is_none()
        && let Some(y) = first_max(&gy, |y| (card_worth(g, p, y, false), g.db.get(y).cmc))
    {
        remove_card(&mut g.player_mut(p).gy, y);
        enter(g, p, y, Enter::default())?;
    }
    if x >= 10 {
        for m in g.player(p).perms.clone() {
            if g.is_creature(m) {
                super::t3::eot(g, m, x, x);
                g.perm_mut(m).sick = false;
            }
        }
    }
    Ok("gy")
}

fn farseek_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if g.player(p).turns <= 4 { 80 } else { 40 }
}

/// a Plains, Island, Swamp or Mountain card (duals included) onto the battlefield tapped
fn farseek(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    const TYPES: [&str; 4] = ["plains", "island", "swamp", "mountain"];
    let cs: Vec<CardId> = searchable(g, p)
        .into_iter()
        .filter(|&x| {
            let d = g.db.get(x);
            d.land && (TYPES.contains(&&*d.name.to_lowercase()) || TYPES.iter().any(|t| d.has_subtype(t)))
        })
        .collect();
    if let Some(x) = max_by(&cs, |x| g.db.get(x).tags.str(Tag::C).map_or(0, |s| s.len()) as f64) {
        remove_card(&mut g.player_mut(p).library, x);
        shuffle_library(g, p);
        g.add_land(p, x, true);
        crate::glog!(g, "    Farseek finds {}", g.db.get(x).name);
    }
    Ok("gy")
}

/// each noncreature spell you cast proliferates
fn ichor(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let d = g.db.get(c);
    if caster == g.perm(src).owner
        && !d.creature
        && !d.land
        && trigger_window(g, caster, Some(src), "proliferate", None)?
    {
        super::common::proliferate(g, caster, 1)?;
    }
    Ok(())
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    for name in ["Goblin Rabblemaster", "Legion Warboss"] {
        let c = r.card(db, name)?;
        c.upkeep = Some(goblin_maker);
        at_once(c, Event::Upkeep, false);
    }
    r.card(db, super::t1::ADANTO)?.land_options = Some(adanto);
    r.card(db, "Shadowspear")?.options = Some(shadowspear);
    let c = r.card(db, "Sakura-Tribe Elder")?;
    c.blocks = Some(ste_block);
    at_once(c, Event::Blocks, true);
    c.options = Some(ste_eot);
    let c = r.card(db, "Springbloom Druid")?;
    c.etb = Some(springbloom);
    at_once(c, Event::Etb, false);
    let c = r.card(db, "Knight of the White Orchid")?;
    c.etb = Some(kotwo);
    at_once(c, Event::Etb, false);
    r.card(db, "Woe Strider")?.gy_options = Some(woe_escape);
    // crew, flash in combat, evoke, dash, casualty, overload
    let c = r.card(db, "Esika's Chariot")?;
    c.crew = Some(crew);
    at_once(c, Event::Crew, true);
    c.attack = Some(chariot_copy);
    c.etb = Some(chariot_cats);
    let c = r.card(db, "Embercleave")?;
    c.hand_attack = Some(cleave_flash);
    c.prio = Some(no_prio);
    r.card(db, "The Wandering Emperor")?.hand_defend = Some(emperor_flash);
    r.card(db, "Solitude")?.hand_options = Some(solitude_evoke);
    let c = r.card(db, "Ragavan, Nimble Pilferer")?;
    c.hand_options = Some(ragavan_dash);
    c.end_step = Some(ragavan_back);
    let c = r.card(db, "Ob Nixilis, the Adversary")?;
    c.etb = Some(obnix_casualty);
    at_once(c, Event::Etb, true);
    let c = r.card(db, "Winds of Abandon")?;
    c.resolve = Some(winds);
    c.prio = Some(winds_prio);
    // level up, adapt
    let c = r.card(db, "Joraga Treespeaker")?;
    c.options = Some(joraga);
    c.dyn_mana_perm = Some(joraga_mana);
    c.extra_mana = Some(joraga_elves);
    r.card(db, "Incubation Druid")?.options = Some(adapt);
    // harmonize, bargain, improvise, transmute, cycling
    r.card(db, "Nature's Rhythm")?.gy_options = Some(harmonize);
    let c = r.card(db, "Beseech the Mirror")?;
    c.resolve = Some(beseech);
    c.prio = Some(fifty);
    let c = r.card(db, "Whir of Invention")?;
    c.resolve = Some(whir);
    c.prio = Some(whir_prio);
    let c = r.card(db, "Transmute Artifact")?;
    c.resolve = Some(transmute);
    c.prio = Some(transmute_prio);
    r.card(db, "Unearth")?.hand_options = Some(unearth_cycle);
    r.card(db, "Wishclaw Talisman")?.options = Some(wishclaw);
    r.card(db, "Aura of Silence")?.options = Some(aura_silence);
    let c = r.card(db, "Soul-Guide Lantern")?;
    c.etb = Some(lantern_etb);
    c.options = Some(lantern_draw);
    // blocking rules
    let c = r.card(db, "Silent Arbiter")?;
    c.blocks = Some(arbiter_block);
    at_once(c, Event::Blocks, true);
    let c = r.card(db, "Archangel of Tithes")?;
    c.blocks = Some(tithes_block);
    at_once(c, Event::Blocks, true);
    r.card(db, "Norn's Annex")?.attack_tax = Some(annex_tax);
    r.card(db, "Syr Konrad, the Grim")?.cards_to_gy = Some(konrad_mill);
    let c = r.card(db, "Glimpse of Nature")?;
    c.resolve = Some(glimpse);
    c.prio = Some(glimpse_prio);
    r.card(db, "Chatterfang, Squirrel General")?.options = Some(fang);
    // Detention Sphere (replaces common's O-Ring), Eldrazi Displacer, Charming Prince
    let c = r.card(db, "Detention Sphere")?;
    c.etb = Some(dsphere);
    at_once(c, Event::Etb, false);
    c.leaves = Some(dsphere_back);
    at_once(c, Event::Leaves, true);
    r.card(db, "Eldrazi Displacer")?.options = Some(displacer);
    let c = r.card(db, "Charming Prince")?;
    c.etb = Some(prince);
    c.end_step = Some(prince_back);
    at_once(c, Event::EndStep, true);
    // pumps
    r.card(db, "Battle Cry Goblin")?.options = Some(bcg_pump);
    let c = r.card(db, "Purphoros, God of the Forge")?;
    c.etb = Some(purph_on);
    c.options = Some(purph_pump);
    c.grant_kw = Some(purph_ind);
    r.card(db, "Scourge of Valkas")?.options = Some(scourge_fire);
    r.card(db, "Resplendent Angel")?.options = Some(resplendent_pump);
    let c = r.card(db, "Rionya, Fire Dancer")?;
    c.combat_start = Some(rionya_x);
    c.end_step = Some(rionya_exile);
    // Stoneforge Mystic, Gilded Goose, Destiny Spinner
    r.card(db, "Stoneforge Mystic")?.options = Some(sfm_put);
    r.card(db, "Gilded Goose")?.options = Some(goose_food);
    r.card(db, "Destiny Spinner")?.options = Some(spinner);
    // Valakut Exploration, Finale, Farseek, Ichormoon Gauntlet
    let c = r.card(db, "Valakut Exploration")?;
    c.end_step = Some(valakut_end);
    c.landfall = Some(valakut_land);
    let c = r.card(db, "Finale of Devastation")?;
    c.resolve = Some(finale);
    c.prio = Some(finale_prio);
    let c = r.card(db, "Farseek")?;
    c.resolve = Some(farseek);
    c.prio = Some(farseek_prio);
    r.card(db, "Ichormoon Gauntlet")?.cast = Some(ichor);
    Ok(())
}
