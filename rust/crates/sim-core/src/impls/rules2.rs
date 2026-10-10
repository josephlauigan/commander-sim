//! Python's `cards/impl/rules2.py`: the last approximated clauses of pool cards (forced attacks, crew, blocking
//! restrictions and taxes, ward from Auras, graveyard triggers, and the remaining activated abilities). The Tier 1 and
//! Sauron cards so far; the other decks' cards come with phase 6.

use super::partials::{ability_window_card, at_once, auras_on, exile_fodder, first_min, name_of, on, remove_card};
use crate::cards::{CardDb, Colors};
use crate::engine::cast::{castable, on_cast};
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, cost_of, pay};
use crate::engine::stack::{ability_window, counter_window, trigger_window};
use crate::engine::tutors::{card_worth, shuffle_library};
use crate::engine::values::{has_type, pval};
use crate::engine::zones::{Enter, Tokens, die, draw, enter, land_ramp, make_tokens, max_by, searchable};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
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
/// turn in `p.glimpse`; the Rust field is a flag, which phase 6's Glimpse of Nature sets for the turn.)
pub fn glimpse_draw(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    if g.db.get(c).creature && g.player(p).glimpse {
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
/// the flipped land: {2}{W}, {T}: a 1/1 lifelink Vampire (at end of turn). Python reads `p.adanto`, which nothing sets,
/// so the option never shows.
fn adanto(_g: &mut Game, _l: LandId, _p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    Ok(vec![])
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

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    for name in ["Goblin Rabblemaster", "Legion Warboss"] {
        let c = r.card(db, name)?;
        c.upkeep = Some(goblin_maker);
        at_once(c, Event::Upkeep, false);
    }
    r.card(db, "Legion's Landing // Adanto, the First Fort")?.land_options = Some(adanto);
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
    Ok(())
}
