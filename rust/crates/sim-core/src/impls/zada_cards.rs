//! The Zada deck's supporting cards (deck key `zada`; Zada himself, the Zada spells and the spell payoffs are
//! `zada.rs`'s): the small creatures, the burn, the Sagas and a few artifacts. The mechanics built here are
//! - prepared (`mine.rs`'s PREPARED table: Blazing Firesinger, Goblin Glasswright and Strife Scholar),
//! - intimidate (`rules::evasion_blocked`) and
//! - plot (Irascible Wolverine: `plot_*` below).
//!
//! Cards whose text is fully covered by their tags (Orcish Cannonade, Reiterating Bolt ...) have no code here, only
//! an entry in data/card_overrides.toml; the tests in tests/cards_zada_cards.rs cover them too.

use crate::cards::{CardDb, Colors};
use crate::engine::cast::bolt_something;
use crate::engine::life::lose_life;
use crate::engine::removal::apply_removal;
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::values::{epow, etgh, has_type, pval, untargetable};
use crate::engine::zones::{Tokens, bounce, make_tokens, max_by, min_by};
use crate::flow::Res;
use crate::hooks::{Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game};
use crate::sym::Sym;
use crate::tag::Tag;

// ------------------------------------------------------------------ helpers
fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

/// 1 damage (or n) to any target: a creature worth killing, else the weakest opponent
fn ping(g: &mut Game, p: PlayerId, n: i32) -> Res {
    bolt_something(g, p, n)
}

/// p's artifacts: the permanents, plus the Treasure, Food and Clue tokens (kept as counts)
pub fn artifacts(g: &Game, p: PlayerId) -> i32 {
    let perms = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| {
            let x = g.perm(m);
            !x.phased
                && (x.cd.is_some_and(|c| g.db.get(c).types.has(crate::cards::Types::ARTIFACT))
                    || x.ttypes.contains(&"artifact")
                    || x.data.truthy(DataKey::Artifact))
        })
        .count();
    let pl = g.player(p);
    (perms as u32 + pl.treasures + pl.foods + pl.clues) as i32
}

fn legendary_creature(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| {
        let x = g.perm(m);
        !x.phased && g.is_creature(m) && x.cd.is_some_and(|c| g.db.get(c).tag(Tag::Leg))
    })
}

/// the top card of p's library exiled; None if the library is empty
fn exile_top(g: &mut Game, p: PlayerId) -> Option<CardId> {
    let c = g.player_mut(p).library.pop()?;
    g.player_mut(p).seen.insert(c);
    Some(c)
}

/// a card exiled "you may play it until the end of your next turn"
fn playable_until_next_turn(g: &mut Game, p: PlayerId, c: CardId) {
    let pl = g.player_mut(p);
    pl.hand.push(c);
    let until = pl.turns + 1;
    pl.impulse_long.push((c, until));
}

// ------------------------------------------------------------------ Goblin Arsonist, Bomber Corps
/// Goblin Arsonist: when it dies, you may have it deal 1 damage to any target
fn arsonist_dies(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, m);
    if trigger_window(g, o, Some(m), "1 damage to any target", None)? {
        ping(g, o, 1)?;
    }
    Ok(())
}

/// Bomber Corps, battalion: it and at least two other creatures attack: 1 damage to any target
fn bomber_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) == p
        && atk.contains(&src)
        && atk.len() >= 3
        && g.perm(src).on_bf
        && trigger_window(g, p, Some(src), "battalion: 1 damage to any target", None)?
    {
        ping(g, p, 1)?;
    }
    Ok(vec![])
}

// ------------------------------------------------------------------ Glitterfang, Goblin Rimerunner, Battlecarver
/// Glitterfang: at the beginning of the end step, return it to its owner's hand (haste is in its keywords)
fn glitterfang_end(g: &mut Game, src: Src, _p: PlayerId) -> Res {
    if g.perm(src).on_bf && trigger_window(g, owner(g, src), Some(src), "return to hand", None)? && g.perm(src).on_bf {
        bounce(g, src)?;
    }
    Ok(())
}

/// Goblin Rimerunner: {T}: target creature can't block this turn. The AI uses it as it attacks, on the best
/// untapped creature that could block. ({S}: haste needs snow mana, which no card of the deck makes.)
fn rimerunner_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], d: PlayerId) -> Res<Vec<PermId>> {
    let x = g.perm(src);
    if x.owner != p || !x.on_bf || x.tapped || x.phased || atk.contains(&src) || x.neutered {
        return Ok(vec![]);
    }
    if x.sick && !crate::dsl::has_kw(g, src, "haste") {
        return Ok(vec![]);
    }
    let blockers: Vec<PermId> = g
        .player(d)
        .perms
        .iter()
        .copied()
        .filter(|&b| {
            g.is_creature(b)
                && !g.perm(b).tapped
                && !g.perm(b).phased
                && !untargetable(g, b)
                && atk.iter().any(|&a| crate::engine::combat::can_block(g, b, a))
        })
        .collect();
    let Some(b) = max_by(&blockers, |b| pval(g, b)) else { return Ok(vec![]) };
    g.perm_mut(src).tapped = true;
    if ability_window(g, p, Some(src), "target creature can't block this turn", Some(2.0), Some(b))? && g.perm(b).on_bf
    {
        g.dsl_on = true; // the engine reads "can't block" only while the ability language is on
        let k = &mut g.perm_mut(b).eot_kw;
        if !k.contains(&"cant_block") {
            k.push("cant_block");
        }
        crate::glog!(g, "    {} can't block this turn", g.perm(b).name);
    }
    Ok(vec![])
}

/// Skilled Battlecarver: first strike during its controller's turn
fn battlecarver_kw(g: &Game, src: Src, m: PermId, kw: Sym) -> bool {
    m == src && kw == "first strike" && g.active == Some(owner(g, src))
}

// ------------------------------------------------------------------ Rundvelt Hordemaster
/// the top card of the library exiled; a Goblin creature card may be cast until the end of your next turn
fn hordemaster_exile(g: &mut Game, src: Src, dead: PermId) -> Res {
    let o = owner(g, dead);
    if g.player(o).library.is_empty() || !trigger_window(g, o, Some(src), "exile the top card", None)? {
        return Ok(());
    }
    let Some(c) = exile_top(g, o) else { return Ok(()) };
    let d = g.db.get(c);
    let goblin = d.creature && d.subtypes.iter().any(|s| &**s == "goblin");
    crate::glog!(g, "    Rundvelt Hordemaster exiles {}{}", d.name, if goblin { " (castable)" } else { "" });
    if goblin {
        playable_until_next_turn(g, o, c);
    } else {
        g.player_mut(o).exile.push(c);
    }
    Ok(())
}

/// another Goblin you control dies (the Hordemaster itself is `hordemaster_self_dies`)
fn hordemaster_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    if m != src && owner(g, m) == owner(g, src) && has_type(g, m, "goblin") {
        hordemaster_exile(g, src, m)?;
    }
    Ok(())
}

fn hordemaster_self_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    hordemaster_exile(g, src, m)
}

// ------------------------------------------------------------------ Pia, Determined Rebuilder
/// When Pia enters, a 1/1 colourless Thopter artifact creature token with flying
fn pia_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, p, Some(src), "create a 1/1 Thopter", None)? {
        let spec = Tokens {
            fly: true,
            color: Some(Colors::from_letters("")),
            types: vec!["thopter", "artifact"],
            ..Tokens::new(1, 1)
        };
        make_tokens(g, p, spec)?;
    }
    Ok(())
}

/// {5}{R}: target creature gets +X/+0 until end of turn, X = artifacts you control. The AI pays it as it attacks,
/// for the attacker that gains the most (an evasive one first), when X is at least 2.
fn pia_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if owner(g, src) != p || !g.perm(src).on_bf || g.perm(src).neutered || g.perm(src).phased {
        return Ok(vec![]);
    }
    let x = artifacts(g, p);
    if x < 2 || !crate::engine::mana::can_pay(g, p, 5, "R", false) {
        return Ok(vec![]);
    }
    let Some(t) = max_by(atk, |a| {
        let m = g.perm(a);
        (if m.fly { 3.0 } else { 0.0 }) + (if m.dt { 1.0 } else { 0.0 }) + epow(g, a) as f64 * 0.1
    }) else {
        return Ok(vec![]);
    };
    crate::engine::mana::pay(g, p, 5, "R", false)?;
    if ability_window(g, p, Some(src), "+X/+0 (X = artifacts)", Some(3.0), Some(t))? && g.perm(t).on_bf {
        crate::impls::common::eot(g, t, x, 0);
        crate::glog!(g, "    Pia gives {} +{x}/+0", g.perm(t).name);
    }
    Ok(vec![])
}

// ------------------------------------------------------------------ the burn
/// Wrath of the Bloodmane costs {1} less with a legendary creature
fn wrath_cost(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if legendary_creature(g, p) { -1 } else { 0 }
}

/// Suplex: 3 damage to a creature (exiled instead if it would die), or exile an artifact. The engine's removal logic
/// picks the target (creatures first; `alsoart` lets it pick an artifact when no creature is worth the burn).
fn suplex(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = ctx.target
        && g.perm(t).on_bf
    {
        if g.is_creature(t) {
            apply_removal(g, Some(p), t, "dmg3", Some(c))?;
        } else {
            apply_removal(g, Some(p), t, "exile", Some(c))?;
        }
    }
    Ok("gy")
}

/// Heated Argument: 6 damage to a creature; you may exile a card from your graveyard, and if you do, 2 damage to
/// that creature's controller. The AI exiles the least valuable card (and always does when it has one).
fn heated_argument(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target else { return burn_face(g, p, c, ctx, 6) };
    let victim = owner(g, t);
    apply_removal(g, Some(p), t, "dmg6", Some(c))?;
    let gy: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&x| x != c).collect();
    if let Some(x) = min_by(&gy, |x| crate::engine::tutors::card_worth(g, p, x, true)) {
        let pl = g.player_mut(p);
        let i = pl.gy.iter().position(|&y| y == x).unwrap();
        pl.gy.remove(i);
        pl.exile.push(x);
        crate::glog!(
            g,
            "    Heated Argument exiles {} from the graveyard: 2 damage to {}",
            g.db.get(x).name,
            g.player(victim).name
        );
        lose_life(g, victim, 2, Some(p), "burn", None)?;
    }
    Ok("gy")
}

fn burn_face(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx, n: i32) -> Res<Sym> {
    if let Some(q) = ctx.face {
        lose_life(g, q, n, Some(p), "burn", None)?;
    }
    Ok("gy")
}

/// Violent Echoes: 6 damage to a creature or planeswalker; excess damage empowers Jace by that much
fn violent_echoes(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target.filter(|&t| g.perm(t).on_bf) else { return Ok("gy") };
    let toughness = if g.is_creature(t) { etgh(g, t) } else { g.perm(t).loyalty.unwrap_or(0) };
    let excess = 6 - toughness;
    apply_removal(g, Some(p), t, "dmg6", Some(c))?;
    if excess > 0 && !g.perm(t).on_bf {
        crate::impls::alela::empower(g, p, excess);
    }
    Ok("gy")
}

// ------------------------------------------------------------------ registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Goblin Arsonist")?.self_dies = Some(arsonist_dies);
    r.card(db, "Bomber Corps")?.attack = Some(bomber_attack);
    r.card(db, "Glitterfang")?.end_step = Some(glitterfang_end);
    r.card(db, "Goblin Rimerunner")?.attack = Some(rimerunner_attack);
    r.card(db, "Skilled Battlecarver")?.grant_kw = Some(battlecarver_kw);
    let c = r.card(db, "Rundvelt Hordemaster")?;
    c.dies = Some(hordemaster_dies);
    c.self_dies = Some(hordemaster_self_dies);
    let c = r.card(db, "Pia, Determined Rebuilder")?;
    c.etb = Some(pia_etb);
    c.attack = Some(pia_attack);
    r.card(db, "Wrath of the Bloodmane")?.self_cost = Some(wrath_cost);
    r.card(db, "Suplex")?.resolve = Some(suplex);
    r.card(db, "Heated Argument")?.resolve = Some(heated_argument);
    r.card(db, "Violent Echoes")?.resolve = Some(violent_echoes);
    Ok(())
}
