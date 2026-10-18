//! Tier 6 batch `counters`: free and alternative-cost counterspells, redirects and the cards that answer a spell on the
//! stack (rust/agents/t6-batches.md). Cards only: no engine file is touched, so what needs the engine (answering a spell
//! with a card that is not tagged `ctr`, alternative costs, split second, "can't cast" locks) is written here as a
//! plain `pub fn` the engine can call later, and listed in rust/agents/engine-needs-counters.md.
//!
//! - Dizzy Spell: -3/-0 (resolve, and as a defender: `hand_defend`, like Icy Reception) and its Transmute.
//! - Into the Flood Maw: bounce, with the Fish gift when the target is not a creature.
//! - Abeyance: draw a card; the target player gets a turn flag (`abeyance_flag`) the engine reads.
//! - The alternative costs (`alt_cost_ok` / `pay_alt_cost`): Daze, Flare of Denial, Disrupting Shoal, Misdirection,
//!   Snapback, Commandeer, Mindbreak Trap.
//! - Effects on a spell on the stack, by stack item id: `misdirect`, `commandeer`, `substitute`, `bilbo_bounce`,
//!   `mindbreak_exile`, and Tibalt's Trickery's side effect (`tibalt_after`).

use crate::cards::{CardDb, Colors};
use crate::engine::cast::{cast_card, castable, casts_this_turn, on_cast, spell_targets};
use crate::engine::mana::{can_pay, cost_of, pay};
use crate::engine::removal::apply_removal;
use crate::engine::stack::counter_window;
use crate::engine::tutors::card_worth;
use crate::engine::values::{epow, pval, untargetable};
use crate::engine::zones::{Tokens, die, draw, make_tokens, max_by, min_by, sac_worth};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry};
use crate::ids::{CardId, LandId, PermId, PlayerId};
use crate::state::{Ctx, Game, StackKind};
use crate::sym::Sym;

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

fn remove_card(v: &mut Vec<CardId>, c: CardId) {
    if let Some(i) = v.iter().position(|&x| x == c) {
        v.remove(i);
    }
}

fn plan(utility: f64, label: String, f: crate::hooks::PlanFn, arg: i64) -> Opt {
    Opt { utility, label, act: Some(Action::Plan { f, arg }) }
}

// ================================================================== alternative costs
/// a blue card (a blue mana symbol in its cost, as `engine::stack::blue_in_hand` counts it)
fn is_blue(g: &Game, c: CardId) -> bool {
    g.db.get(c).pips.contains('U')
}

/// the blue cards in p's hand other than `card` itself (a pitch cost exiles "a blue card from your hand")
pub fn blue_pitch(g: &Game, p: PlayerId, card: CardId) -> Vec<CardId> {
    g.player(p).hand.iter().copied().filter(|&x| x != card && is_blue(g, x)).collect()
}

/// the Islands p controls (Daze)
pub fn islands(g: &Game, p: PlayerId) -> Vec<LandId> {
    g.player(p)
        .lands
        .iter()
        .copied()
        .filter(|&l| {
            let d = g.db.get(g.land(l).cd);
            &*d.name == "Island" || d.has_subtype("island")
        })
        .collect()
}

/// the nontoken blue creatures p controls (Flare of Denial's sacrifice)
pub fn flare_fodder(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            x.on_bf && !x.phased && !x.token && g.is_creature(m) && x.cd.is_some_and(|c| is_blue(g, c))
        })
        .collect()
}

/// Mindbreak Trap: an opponent of p cast three or more spells this turn
pub fn mindbreak_free(g: &Game, p: PlayerId) -> bool {
    g.opps(p).any(|q| casts_this_turn(g, q) >= 3)
}

/// Disrupting Shoal: the blue card in p's hand (other than the Shoal) whose mana value is `x`, the least useful one
pub fn shoal_pitch(g: &Game, p: PlayerId, shoal: CardId, x: u32) -> Option<CardId> {
    let cs: Vec<CardId> = blue_pitch(g, p, shoal).into_iter().filter(|&c| g.db.get(c).cmc == x).collect();
    min_by(&cs, |c| card_worth(g, p, c, false))
}

/// Disrupting Shoal resolves: the spell is countered only if its mana value is X (X is the pitched card's value, or
/// what was paid when it is cast for {X}{U}{U})
pub fn shoal_counters(g: &Game, spell: CardId, x: u32) -> bool {
    g.db.get(spell).cmc == x
}

/// can p cast `card` for its alternative cost now? `spell` is the spell it answers (Disrupting Shoal needs its mana
/// value; the others ignore it). The card's own mana cost is not looked at.
pub fn alt_cost_ok(g: &Game, p: PlayerId, card: CardId, spell: Option<CardId>) -> bool {
    match cname(g, card) {
        "Daze" => !islands(g, p).is_empty(),
        "Flare of Denial" => !flare_fodder(g, p).is_empty(),
        "Misdirection" | "Snapback" => !blue_pitch(g, p, card).is_empty(),
        "Commandeer" => blue_pitch(g, p, card).len() >= 2,
        "Disrupting Shoal" => spell.is_some_and(|s| shoal_pitch(g, p, card, g.db.get(s).cmc).is_some()),
        "Mindbreak Trap" => mindbreak_free(g, p),
        _ => false,
    }
}

fn exile_from_hand(g: &mut Game, p: PlayerId, c: CardId) {
    let pl = g.player_mut(p);
    remove_card(&mut pl.hand, c);
    pl.exile.push(c);
}

/// pay `card`'s alternative cost (see `alt_cost_ok`): the AI gives up the least useful card. False (nothing paid) if
/// it cannot be paid.
pub fn pay_alt_cost(g: &mut Game, p: PlayerId, card: CardId, spell: Option<CardId>) -> Res<bool> {
    if !alt_cost_ok(g, p, card, spell) {
        return Ok(false);
    }
    match cname(g, card) {
        "Daze" => {
            // return an Island you control to its owner's hand: an untapped one is kept for mana if there is a choice
            let is = islands(g, p);
            let l = min_by(&is, |l| if g.land(l).tapped { 0.0 } else { 1.0 }).unwrap();
            let cd = g.land(l).cd;
            crate::engine::turn::remove_land(g, p, l);
            g.player_mut(p).hand.push(cd);
        }
        "Flare of Denial" => {
            let m = min_by(&flare_fodder(g, p), |m| sac_worth(g, m)).unwrap();
            crate::glog!(g, "    {} sacrifices {}", pname(g, p), g.perm(m).name);
            die(g, m, "sac")?;
        }
        "Misdirection" | "Snapback" => {
            let b = blue_pitch(g, p, card);
            let x = min_by(&b, |c| card_worth(g, p, c, false)).unwrap();
            exile_from_hand(g, p, x);
        }
        "Commandeer" => {
            let mut b = blue_pitch(g, p, card);
            for _ in 0..2 {
                let x = min_by(&b, |c| card_worth(g, p, c, false)).unwrap();
                remove_card(&mut b, x);
                exile_from_hand(g, p, x);
            }
        }
        "Disrupting Shoal" => {
            let x = g.db.get(spell.unwrap()).cmc;
            let c = shoal_pitch(g, p, card, x).unwrap();
            exile_from_hand(g, p, c);
        }
        _ => {} // Mindbreak Trap: {0}
    }
    Ok(true)
}

/// cards' worth of value the alternative cost gives up (the AI's price, 1.5 threshold points per card; see
/// `engine::altcost`): a pitched card is a card, Daze's Island is replayed (a land drop), Mindbreak Trap's free mode is free
pub fn alt_lost(g: &Game, _p: PlayerId, card: CardId, _spell: Option<CardId>) -> f64 {
    match cname(g, card) {
        "Daze" => 0.4,
        "Commandeer" => 2.0,
        "Mindbreak Trap" => 0.0,
        _ => 1.0,
    }
}

/// the alternative-cost slot of the cards above (read through `engine::altcost`)
pub const ALT_COST: crate::hooks::AltCost =
    crate::hooks::AltCost { ok: alt_cost_ok, pay: pay_alt_cost, lost: alt_lost };

// ================================================================== effects on a spell on the stack
fn stack_index(g: &Game, id: u32) -> Option<usize> {
    g.stack.iter().position(|x| x.id == id)
}

/// Misdirection: change the target of the spell (stack item `id`) if it has a single target. A spell aimed at one of
/// q's permanents or at q goes to a permanent of the spell's own controller (the best one it may target), or to its
/// controller. True if the target changed.
pub fn misdirect(g: &mut Game, q: PlayerId, id: u32) -> Res<bool> {
    let Some(i) = stack_index(g, id) else { return Ok(false) };
    let it = g.stack[i].clone();
    if it.kind != StackKind::Spell || it.ctx.target.is_some() == it.ctx.face.is_some() {
        return Ok(false); // no target, or two (a spell with a single target has exactly one)
    }
    let caster = it.controller;
    let c = it.card.unwrap();
    if let Some(t) = it.ctx.target {
        if g.perm(t).owner != q {
            return Ok(false); // not aimed at us: leave it
        }
        let d = g.db.get(c);
        let creature_only = d.tags.str(crate::tag::Tag::Tgt) == Some("c")
            || d.tags.str(crate::tag::Tag::Rem).is_some_and(|r| r.starts_with("dmg"));
        let alt: Vec<PermId> = g
            .player(caster)
            .perms
            .iter()
            .copied()
            .filter(|&x| x != t && !g.perm(x).phased && !untargetable(g, x) && (!creature_only || g.is_creature(x)))
            .collect();
        let Some(n) = max_by(&alt, |x| pval(g, x)) else { return Ok(false) };
        crate::glog!(g, "    {} redirects {} to {}", pname(g, q), cname(g, c), g.perm(n).name);
        g.stack[i].ctx.target = Some(n);
        return Ok(true);
    }
    let f = it.ctx.face.unwrap();
    if f != q || !g.player(caster).alive {
        return Ok(false);
    }
    crate::glog!(g, "    {} redirects {} to {}", pname(g, q), cname(g, c), pname(g, caster));
    g.stack[i].ctx.face = Some(caster);
    Ok(true)
}

/// Commandeer: gain control of the noncreature spell (stack item `id`); its permanent enters, or its effect happens,
/// under q's control. The card still returns to its owner's zones (`ctx.owner`). True if control changed.
pub fn commandeer(g: &mut Game, q: PlayerId, id: u32) -> Res<bool> {
    let Some(i) = stack_index(g, id) else { return Ok(false) };
    let it = &g.stack[i];
    let Some(c) = it.card.filter(|_| it.kind == StackKind::Spell && it.controller != q) else { return Ok(false) };
    if g.db.get(c).creature {
        return Ok(false);
    }
    let old = it.controller;
    crate::glog!(g, "    {} gains control of {}", pname(g, q), cname(g, c));
    let it = &mut g.stack[i];
    it.ctx.owner.get_or_insert(old);
    it.controller = q;
    // "you may choose new targets": the best the new controller has for it
    let fresh = spell_targets(g, q, c, Some(&g.stack[i].ctx.clone()));
    let it = &mut g.stack[i];
    it.ctx.target = fresh.target;
    it.ctx.face = fresh.face;
    Ok(true)
}

/// Sudden Substitution: exchange control of the noncreature spell (stack item `id`) and creature m; the spell's old
/// controller then chooses new targets for it. True if both changed hands.
pub fn substitute(g: &mut Game, id: u32, m: PermId) -> Res<bool> {
    let Some(i) = stack_index(g, id) else { return Ok(false) };
    let it = &g.stack[i];
    let Some(c) = it.card.filter(|_| it.kind == StackKind::Spell) else { return Ok(false) };
    if g.db.get(c).creature || !g.perm(m).on_bf || !g.is_creature(m) || g.perm(m).owner == it.controller {
        return Ok(false);
    }
    let (sp, cp) = (it.controller, g.perm(m).owner);
    crate::glog!(g, "    {} exchanges {} and {}", pname(g, sp), cname(g, c), g.perm(m).name);
    crate::impls::marchesa::steal(g, sp, m, false);
    let it = &mut g.stack[i];
    it.ctx.owner.get_or_insert(sp);
    it.controller = cp;
    let fresh = spell_targets(g, cp, c, Some(&g.stack[i].ctx.clone()));
    let it = &mut g.stack[i];
    it.ctx.target = fresh.target;
    it.ctx.face = fresh.face;
    Ok(true)
}

/// Bilbo's Gambit: return the spell (stack item `id`) to its owner's hand. With the gift promised an opponent gets a
/// Treasure first and nobody can cast spells this turn (`lock_flag`). True if the spell was returned.
pub fn bilbo_bounce(g: &mut Game, q: PlayerId, id: u32, gift: bool) -> Res<bool> {
    let Some(i) = stack_index(g, id) else { return Ok(false) };
    let it = &g.stack[i];
    let Some(c) = it.card.filter(|_| it.kind == StackKind::Spell && !it.countered) else { return Ok(false) };
    let owner = it.ctx.owner.unwrap_or(it.controller);
    if gift {
        // the gift: a Treasure for the opponent least able to use it
        if let Some(o) = min_by(&g.opps(q).collect::<Vec<_>>(), |o| g.player(o).perms.len() as f64) {
            crate::engine::zones::add_treasure(g, o, 1)?;
        }
        set_flag(g, q, "bilbo_lock");
    }
    crate::glog!(g, "    {} returns {} to its owner's hand", pname(g, q), cname(g, c));
    let it = &mut g.stack[i];
    it.countered = true;
    it.countered_by = None;
    g.player_mut(owner).hand.push(c);
    Ok(true)
}

/// the turn flag Bilbo's Gambit's lock sets (on the caster; it means "players can't cast spells this turn")
pub fn lock_flag(g: &Game, p: PlayerId) -> bool {
    flag_now(g, p, "bilbo_lock")
}

/// Mindbreak Trap: exile any number of target spells; the AI exiles every spell on the stack that is not its own.
/// Returns how many it exiled.
pub fn mindbreak_exile(g: &mut Game, q: PlayerId) -> Res<usize> {
    let ids: Vec<u32> = g
        .stack
        .iter()
        .filter(|x| {
            x.kind == StackKind::Spell
                && x.controller != q
                && !x.countered
                && x.card.is_some()
                && crate::engine::stack::counterable(g, x)
        })
        .map(|x| x.id)
        .collect();
    for &id in &ids {
        let i = stack_index(g, id).unwrap();
        let c = g.stack[i].card.unwrap();
        crate::glog!(g, "    {} exiles {} with Mindbreak Trap", pname(g, q), cname(g, c));
        g.stack[i].countered = true;
        g.stack[i].ctx.exile_after = true;
        g.stack[i].countered_by = g.db.id("Mindbreak Trap");
    }
    Ok(ids.len())
}

fn set_flag(g: &mut Game, p: PlayerId, key: Sym) {
    let stamp = g.turn_stamp();
    g.player_mut(p).flag_turn.insert(key, stamp);
}

fn flag_now(g: &Game, p: PlayerId, key: Sym) -> bool {
    g.player(p).flag_turn.get(key) == Some(&g.turn_stamp())
}

// ================================================================== Tibalt's Trickery
/// Tibalt's Trickery's side effect, after it countered `spell` of p's (q cast it): choose 1, 2 or 3 at random; p mills
/// that many, then exiles cards from the top until a nonland card with a different name than the spell, and may cast
/// it free (the AI does unless it is a counterspell it would rather keep); the exiled cards go to the bottom in a
/// random order. Returns the card p cast, if any.
pub fn tibalt_after(g: &mut Game, q: PlayerId, p: PlayerId, spell: CardId) -> Res<Option<CardId>> {
    let n = *g.rng.choice(&[1usize, 2, 3]).unwrap();
    crate::engine::zones::mill(g, p, n)?;
    let mut seen = vec![];
    let mut hit = None;
    while let Some(x) = g.player_mut(p).library.pop() {
        let d = g.db.get(x);
        if !d.land && d.name != g.db.get(spell).name {
            hit = Some(x);
            break;
        }
        seen.push(x);
    }
    let mut cast = None;
    if let Some(h) = hit {
        let want = !g.db.get(h).tag(crate::tag::Tag::Ctr);
        if want && castable(g, p, h, "lib") {
            crate::glog!(g, "    Tibalt's Trickery ({}): {} casts {} free", pname(g, q), pname(g, p), cname(g, h));
            g.last_x = 0;
            let ctx = spell_targets(g, p, h, None);
            cast_card(g, p, h, "lib", ctx)?;
            cast = Some(h);
        } else {
            seen.push(h);
        }
    }
    g.rng.shuffle(&mut seen);
    let lib = &mut g.player_mut(p).library;
    let rest = std::mem::replace(lib, seen);
    lib.extend(rest);
    Ok(cast)
}

// ================================================================== Dizzy Spell
/// Dizzy Spell: target creature gets -3/-0 until end of turn
fn dizzy_resolve(g: &mut Game, _p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf && g.is_creature(t)) {
        super::common::eot(g, t, -3, 0);
    }
    Ok("gy")
}

/// Dizzy Spell as the defender (like Icy Reception): -3/-0 on the attacker whose damage matters most
fn dizzy_defend(
    g: &mut Game,
    c: CardId,
    d: PlayerId,
    _p: PlayerId,
    atk: &mut Vec<PermId>,
    assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    if !g.player(d).hand.contains(&c) || !castable(g, d, c, "hand") {
        return Ok(());
    }
    let (gn, pips) = cost_of(g, d, c);
    if !can_pay(g, d, gn, &pips, false) {
        return Ok(());
    }
    let open: Vec<PermId> = atk
        .iter()
        .copied()
        .filter(|&a| g.perm(a).on_bf && g.perm(a).owner != d && !assign.iter().any(|x| x.0 == a))
        .collect();
    let dmg: i32 = open.iter().map(|&a| epow(g, a).max(0)).sum();
    let Some(big) = max_by(&open, |a| epow(g, a) as f64) else { return Ok(()) };
    let saved = epow(g, big).clamp(0, 3);
    let life = g.player(d).life;
    let worth = (dmg >= life && dmg - saved < life) || (saved >= 3 && dmg * 3 >= life);
    if !worth {
        return Ok(());
    }
    pay(g, d, gn, &pips, false)?;
    remove_card(&mut g.player_mut(d).hand, c);
    crate::glog!(g, "  {} casts Dizzy Spell: {} gets -3/-0", pname(g, d), g.perm(big).name);
    on_cast(g, d, c)?;
    if !counter_window(g, d, c, 4.0, vec![])? {
        g.player_mut(d).gy.push(c);
        return Ok(());
    }
    if g.perm(big).on_bf {
        super::common::eot(g, big, -3, 0);
    }
    g.player_mut(d).gy.push(c);
    Ok(())
}

/// Transmute {1}{U}{U} (sorcery speed, from the hand): discard Dizzy Spell, search for a card with mana value 1
fn dizzy_transmute_opts(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "UU", false) {
        return Ok(vec![]);
    }
    let has_target = g.player(p).library.iter().any(|&x| g.db.get(x).cmc == g.db.get(c).cmc && !g.db.get(x).land);
    if !has_target {
        return Ok(vec![]);
    }
    Ok(vec![plan(2.5, "Dizzy Spell: transmute".into(), dizzy_transmute_go, c.0 as i64)])
}

fn dizzy_transmute_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 1, "UU", false) {
        return Ok(false);
    }
    pay(g, p, 1, "UU", false)?;
    let pl = g.player_mut(p);
    remove_card(&mut pl.hand, c);
    pl.gy.push(c);
    let mv = g.db.get(c).cmc;
    crate::glog!(g, "  {} transmutes Dizzy Spell", pname(g, p));
    let pred = move |g: &Game, x: CardId| g.db.get(x).cmc == mv;
    super::t1::tutor_named(g, p, &pred, 1, "hand")?;
    Ok(true)
}

// ================================================================== Into the Flood Maw
/// Into the Flood Maw: return target creature an opponent controls to its owner's hand; if the gift was promised
/// (the AI promises it when the target is not a creature), instead return target nonland permanent an opponent
/// controls, and an opponent first creates a tapped 1/1 blue Fish.
fn maw_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf && g.perm(t).owner != p) else { return Ok("gy") };
    let gift = !g.is_creature(t);
    if gift {
        // the opponent who gets the Fish: not the target's controller if another opponent has the choice
        let owner = g.perm(t).owner;
        let others: Vec<PlayerId> = g.opps(p).filter(|&o| o != owner).collect();
        let pool = if others.is_empty() { vec![owner] } else { others };
        if let Some(o) = min_by(&pool, |o| g.player(o).life as f64) {
            let spec = Tokens { color: Some(Colors::from_letters("U")), types: vec!["fish"], ..Tokens::new(1, 1) };
            for f in make_tokens(g, o, spec)? {
                g.perm_mut(f).tapped = true;
            }
            crate::glog!(g, "    {} gifts {} a tapped Fish", pname(g, p), pname(g, o));
        }
    }
    if g.perm(t).on_bf {
        apply_removal(g, Some(p), t, "bounce", Some(c))?;
    }
    Ok("gy")
}

// ================================================================== Abeyance
/// the turn flag Abeyance sets on its target: that player can't cast instant or sorcery spells or activate abilities
/// that aren't mana abilities this turn (the engine reads it)
pub fn abeyance_flag(g: &Game, p: PlayerId) -> bool {
    flag_now(g, p, "abeyance")
}

/// Abeyance: target player (the opponent with the most cards in hand) can't cast instants or sorceries or activate
/// non-mana abilities this turn; draw a card
fn abeyance_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let target = ctx.face.filter(|f| opps.contains(f)).or_else(|| max_by(&opps, |o| g.player(o).hand.len() as f64));
    if let Some(t) = target {
        set_flag(g, t, "abeyance");
        crate::glog!(g, "    {} can't cast instants or sorceries this turn (Abeyance)", pname(g, t));
    }
    draw(g, p, 1, false)?;
    Ok("gy")
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Dizzy Spell")?;
    c.resolve = Some(dizzy_resolve);
    c.hand_defend = Some(dizzy_defend);
    c.hand_options = Some(dizzy_transmute_opts);
    r.card(db, "Into the Flood Maw")?.resolve = Some(maw_resolve);
    r.card(db, "Abeyance")?.resolve = Some(abeyance_resolve);
    for name in ["Daze", "Flare of Denial", "Disrupting Shoal", "Misdirection", "Snapback", "Commandeer", "Mindbreak Trap"] {
        r.card(db, name)?.alt_cost = Some(ALT_COST);
    }
    Ok(())
}
