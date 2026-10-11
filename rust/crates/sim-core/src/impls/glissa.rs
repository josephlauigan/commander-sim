//! The glissa candidate deck (decklists/JD/candidates/glissa.md): Golgari fight club. Its card code and AI plan.
//! The mechanics built here, once each:
//! - **bite** (`pick`, `deal`, `bite_resolve`): "target creature you control deals damage equal to its power to target
//!   creature or planeswalker you don't control". The biting creature is `Ctx::host`, the victim `Ctx::target`.
//!   Deathtouch makes any bite lethal. Bite Down, Hard-Hitting Question, Compel Brutality, Diplomatic Relations,
//!   Burrog Barrage and Knockout Maneuver are bites; Chelonian Tackle is a fight (the victim hits back).
//!   The engine tracks no marked damage, so a bite that does not kill does nothing.
//! - **repartee** (`repartee`): "whenever you cast an instant or sorcery that targets a creature" (the card's `cast` hook
//!   reads `g.cur_cast`; a bite targets your own creature, so it always counts).
//! - **crime** (`crime`): targeting an opponent, their permanents or their graveyard cards (Hardbristle Bandit).
//! - **infusion** (`gained_life`): "if you gained life this turn" (`Player::gained_turn`).
//!
//! Several cards here are shared with the jenova candidate deck: they are keyed by card name, not by deck.
//! The AI plan: bite and fight with the best deathtouch creature when one is out (any bite kills), otherwise the biggest
//! creature, at the biggest threat; ramp first, then threats (the generic priorities); Glissa is protected by
//! Champion's Helm and Darksteel Plate (the generic equip plays). Tests: tests/cards_glissa.rs.

use super::common::{AURA, AuraSpec, aura};
use super::partials::{eot_kw, remove_card};
use crate::cards::CardDb;
use crate::engine::cast::{castable, is_casts};
use crate::engine::life::{gain, lose_life};
use crate::engine::mana::{can_pay, cost_of};
use crate::engine::removal::{apply_removal, legal_targets};
use crate::engine::stack::trigger_window;
use crate::engine::tutors::shuffle_library;
use crate::engine::values::{epow, etgh, once_per_turn, pval, untargetable};
use crate::engine::zones::{
    Enter, Zone, die, discard_cards, draw, enter, leave, make_artifact_tokens, max_by, min_by, to_zone_card,
};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::{Sym, intern};

/// glissa's cast priority: the generic priorities (the hand-written spells hide themselves through their `prio` slot
/// and are played by their `hand_options`)
pub fn glissa_prio(g: &Game, p: PlayerId, c: CardId) -> i32 {
    crate::ai::pool::generic_prio(g, p, c)
}

// ------------------------------------------------------------------ helpers
fn name_of(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn human(g: &Game, p: PlayerId) -> bool {
    crate::human::is_human(g, p)
}

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    x.on_bf && !x.phased
}

fn live_creature(g: &Game, m: PermId) -> bool {
    live(g, m) && g.is_creature(m)
}

fn deathtouch(g: &Game, m: PermId) -> bool {
    g.perm(m).dt || crate::engine::combat::kw(g, m, "deathtouch")
}

fn is_walker(g: &Game, m: PermId) -> bool {
    g.perm(m).loyalty.is_some()
}

/// infusion: p gained life this turn
pub fn gained_life(g: &Game, p: PlayerId) -> bool {
    g.player(p).gained_turn.is_some_and(|x| x.0 == g.turn_stamp() && x.1 > 0)
}

fn affordable(g: &Game, p: PlayerId, c: CardId) -> bool {
    let (cg, cp) = cost_of(g, p, c);
    can_pay(g, p, cg, &cp, false) && castable(g, p, c, "hand")
}

fn zero_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

/// a pump until end of turn
fn pump(g: &mut Game, m: PermId, dp: i32, dt: i32) {
    let x = g.perm_mut(m);
    x.eot_pt.0 += dp;
    x.eot_pt.1 += dt;
}

fn plus_counter(g: &mut Game, m: PermId, n: i32) {
    crate::cardcode::add_counters(g, m, n);
}

/// my cards in the graveyard that are creatures
fn gy_creatures(g: &Game, p: PlayerId) -> usize {
    g.player(p).gy.iter().filter(|&&c| g.db.get(c).creature).count()
}

// ------------------------------------------------------------------ crime (Hardbristle Bandit)
/// p committed a crime: each Hardbristle Bandit of p's untaps (once each turn)
pub fn crime(g: &mut Game, p: PlayerId) -> Res {
    for m in g.player(p).perms.clone() {
        if live(g, m) && g.perm(m).cd.is_some_and(|c| name_of(g, c) == "Hardbristle Bandit") {
            bandit_untap(g, m)?;
        }
    }
    Ok(())
}

fn bandit_untap(g: &mut Game, m: PermId) -> Res {
    let p = owner(g, m);
    if once_per_turn(g, p, intern(&format!("crime_{}", m.0)))
        && trigger_window(g, p, Some(m), "a crime: untap Hardbristle Bandit", Some(2.0))?
        && live(g, m)
    {
        g.perm_mut(m).tapped = false;
    }
    Ok(())
}

/// does the spell being cast target an opponent, an opponent's permanent or a card in their graveyard?
fn spell_is_crime(g: &Game, p: PlayerId, ctx: &Ctx) -> bool {
    ctx.target.is_some_and(|t| owner(g, t) != p) || ctx.face.is_some_and(|q| q != p)
}

/// Hardbristle Bandit: whenever you commit a crime (a spell that targets an opponent or their things), untap it
fn bandit_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if caster != owner(g, src) || !live(g, src) {
        return Ok(());
    }
    let Some((cc, ctx, _)) = g.cur_cast.clone() else { return Ok(()) };
    if cc == c && spell_is_crime(g, caster, &ctx) {
        bandit_untap(g, src)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ repartee
/// the spell being cast is an instant or sorcery of p's that targets a creature
fn repartee(g: &Game, src: Src, caster: PlayerId, c: CardId) -> bool {
    let d = g.db.get(c);
    if caster != owner(g, src) || !live(g, src) || !(d.instant || d.sorcery) {
        return false;
    }
    let Some((cc, ctx, _)) = &g.cur_cast else { return false };
    *cc == c && [ctx.target, ctx.host].into_iter().flatten().any(|t| live_creature(g, t))
}

/// Lecturing Scornmage: repartee, a +1/+1 counter
fn scornmage_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if repartee(g, src, caster, c) && trigger_window(g, caster, Some(src), "repartee: a +1/+1 counter", None)? {
        plus_counter(g, src, 1);
    }
    Ok(())
}

/// Melancholic Poet: repartee, each opponent loses 1 life and you gain 1
fn poet_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if repartee(g, src, caster, c) && trigger_window(g, caster, Some(src), "repartee: drain 1", None)? {
        for o in g.opps(caster).collect::<Vec<_>>() {
            lose_life(g, o, 1, Some(caster), "drain", None)?;
        }
        gain(g, caster, 1)?;
    }
    Ok(())
}

/// Forum Necroscribe: repartee, return the best creature card of your graveyard to the battlefield. (Ward -- discard a
/// card: `rules::removal_taxes`.)
fn necroscribe_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    if !repartee(g, src, caster, c) {
        return Ok(());
    }
    let cands: Vec<CardId> = g.player(caster).gy.iter().copied().filter(|&x| g.db.get(x).creature).collect();
    let Some(best) = max_by(&cands, |x| {
        let d = g.db.get(x);
        d.cmc as f64 + d.bomb as f64 + if d.tags.has(crate::tag::Tag::Dt) { 2.0 } else { 0.0 }
    }) else {
        return Ok(());
    };
    if trigger_window(g, caster, Some(src), "repartee: return a creature card from the graveyard", Some(6.0))?
        && remove_card(&mut g.player_mut(caster).gy, best)
    {
        enter(g, caster, best, Enter::default())?;
    }
    Ok(())
}

// ------------------------------------------------------------------ bite and fight
#[derive(Clone, Copy)]
struct Pick {
    src: PermId,
    tgt: PermId,
    v: f64,
}

/// the damage `src` deals as the spell resolves (`bonus`: power the spell gives it first)
fn bite_power(g: &Game, src: PermId, bonus: i32, walker_src: bool) -> i32 {
    if walker_src { g.perm(src).loyalty.unwrap_or(0) } else { (epow(g, src) + bonus).max(0) }
}

fn kills(g: &Game, src: PermId, amount: i32, t: PermId, walker_src: bool) -> bool {
    if amount <= 0 {
        return false;
    }
    if is_walker(g, t) {
        return amount >= g.perm(t).loyalty.unwrap_or(0);
    }
    (deathtouch(g, src) && !walker_src) || amount >= etgh(g, t)
}

/// the best (creature, victim) for the bite spell c: the victim is the biggest threat that some creature kills; the
/// biter is a deathtouch creature if there is one, else the most powerful one. `bonus`: power the spell gives the biter
/// first; `fight`: the victim hits back (a victim with deathtouch is left alone); `walkers`: planeswalkers are legal
/// victims; `walker_src`: a planeswalker of mine may be the source (Compel Brutality's second mode).
fn pick(g: &Game, p: PlayerId, c: CardId, bonus: i32, fight: bool, walkers: bool, walker_src: bool) -> Option<Pick> {
    let tgts = legal_targets(g, p, "destroy", if walkers { "cp" } else { "c" }, false, Some(c));
    let srcs: Vec<PermId> = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            if walker_src { live(g, m) && is_walker(g, m) } else { live_creature(g, m) && !untargetable_by_me(g, m) }
        })
        .collect();
    let mut best: Option<Pick> = None;
    for &t in &tgts {
        if fight && deathtouch(g, t) && epow(g, t) > 0 {
            continue;
        }
        let ok: Vec<PermId> =
            srcs.iter().copied().filter(|&s| kills(g, s, bite_power(g, s, bonus, walker_src), t, walker_src)).collect();
        let s = ok.iter().copied().find(|&s| deathtouch(g, s)).or_else(|| {
            max_by(&ok, |s| (bite_power(g, s, bonus, walker_src), pval(g, s)).0 as f64 * 100.0 - pval(g, s))
        });
        let Some(s) = s else { continue };
        let v = pval(g, t);
        if best.is_none_or(|b| v > b.v) {
            best = Some(Pick { src: s, tgt: t, v });
        }
    }
    best
}

/// hexproof or shroud on my own creature does not stop me targeting it; this is only a hook for a future rule
fn untargetable_by_me(_g: &Game, _m: PermId) -> bool {
    false
}

/// `src` deals `amount` damage to t (a creature or planeswalker)
fn deal(
    g: &mut Game,
    p: PlayerId,
    src: PermId,
    t: PermId,
    amount: i32,
    spell: Option<CardId>,
    walker_src: bool,
) -> Res {
    if !live(g, t) || amount <= 0 {
        return Ok(());
    }
    if is_walker(g, t) {
        let l = g.perm(t).loyalty.unwrap_or(0) - amount;
        g.perm_mut(t).loyalty = Some(l);
        crate::glog!(g, "    {} deals {} to {} (loyalty {})", g.perm(src).name, amount, g.perm(t).name, l);
        if l <= 0 {
            let o = owner(g, t);
            leave(g, t)?;
            to_zone_card(g, t, Zone::Gy);
            let nm = g.perm(t).name;
            *g.player_mut(o).lost_names.entry(nm).or_insert(0) += 1;
        }
        return Ok(());
    }
    if kills(g, src, amount, t, walker_src) {
        let n = if deathtouch(g, src) && !walker_src { 100_000 } else { amount };
        apply_removal(g, Some(p), t, &format!("dmg{n}"), spell)?;
    }
    Ok(())
}

/// Bite Down, Hard-Hitting Question, Compel Brutality, Diplomatic Relations, Burrog Barrage, Knockout Maneuver and
/// Chelonian Tackle as they resolve: the spell's own effect on the biter, then the damage
fn bite_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let name = name_of(g, c).to_string();
    let walker_src = ctx.mode == Some("walker");
    let (bonus, fight, walkers) = match name.as_str() {
        "Diplomatic Relations" => (1, false, false),
        "Burrog Barrage" => (i32::from(is_casts(g, p) >= 2), false, false),
        "Knockout Maneuver" => (1, false, false),
        "Chelonian Tackle" => (0, true, false),
        "Compel Brutality" => (0, false, true),
        _ => (0, false, true), // Bite Down, Hard-Hitting Question
    };
    // a person chooses both targets; the AI (or a spell cast without them) picks now
    let (mut src, mut tgt) = (ctx.host, ctx.target);
    if src.is_none_or(|s| !live(g, s) || owner(g, s) != p) || (tgt.is_none() && name != "Burrog Barrage" && !fight) {
        if let Some(b) = pick(g, p, c, bonus, fight, walkers, walker_src) {
            (src, tgt) = (Some(b.src), Some(b.tgt));
        } else if src.is_none() {
            src =
                max_by(&g.player(p).perms.iter().copied().filter(|&m| live_creature(g, m)).collect::<Vec<_>>(), |m| {
                    epow(g, m) as f64
                });
        }
    }
    let Some(s) = src.filter(|&s| live(g, s) && owner(g, s) == p) else { return Ok("gy") };
    // the spell's effect on the creature
    match name.as_str() {
        "Diplomatic Relations" => {
            pump(g, s, 1, 0);
            eot_kw(g, s, "vigilance");
        }
        "Burrog Barrage" if is_casts(g, p) >= 2 => pump(g, s, 1, 0),
        "Knockout Maneuver" => plus_counter(g, s, 1),
        "Chelonian Tackle" => pump(g, s, 0, 10),
        _ => {}
    }
    let Some(t) = tgt.filter(|&t| live(g, t)) else { return Ok("gy") };
    let amount = bite_power(g, s, 0, walker_src);
    if fight {
        let back = epow(g, t);
        let kill_back = back > 0 && (deathtouch(g, t) || back >= etgh(g, s));
        deal(g, p, s, t, amount, Some(c), false)?;
        if kill_back && live(g, s) {
            die(g, s, "destroy")?;
        }
    } else {
        deal(g, p, s, t, amount, Some(c), walker_src)?;
    }
    Ok("gy")
}

fn bite_plan(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let Some((bonus, fight, walkers)) = bite_kind(g, p, c) else { return Ok(false) };
    let name = name_of(g, c).to_string();
    let (b, mode) = match pick(g, p, c, bonus, fight, walkers, false) {
        Some(b) => (b, None),
        None if name == "Compel Brutality" => match pick(g, p, c, 0, false, true, true) {
            Some(b) => (b, Some(intern("walker"))),
            None => return Ok(false),
        },
        None => return Ok(false),
    };
    crate::ai::brain::do_cast_with(
        g,
        p,
        c,
        None,
        Ctx { target: Some(b.tgt), host: Some(b.src), mode, ..Ctx::default() },
    )
}

/// (power the spell gives the biter, is it a fight, may it hit planeswalkers) for a bite card
fn bite_kind(g: &Game, p: PlayerId, c: CardId) -> Option<(i32, bool, bool)> {
    Some(match name_of(g, c) {
        "Bite Down" | "Hard-Hitting Question" | "Compel Brutality" => (0, false, true),
        "Diplomatic Relations" => (1, false, false),
        "Burrog Barrage" => (i32::from(is_casts(g, p) >= 1), false, false),
        "Knockout Maneuver" => (1, false, false),
        "Chelonian Tackle" => (0, true, false),
        _ => return None,
    })
}

/// the main-phase play of a bite card: at the biggest threat one of my creatures can kill
fn bite_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if human(g, p) || post.is_none() || g.active != Some(p) || !affordable(g, p, c) {
        return Ok(vec![]);
    }
    let Some((bonus, fight, walkers)) = bite_kind(g, p, c) else { return Ok(vec![]) };
    let mut found = pick(g, p, c, bonus, fight, walkers, false);
    if found.is_none() && name_of(g, c) == "Compel Brutality" {
        found = pick(g, p, c, 0, false, true, true);
    }
    let Some(b) = found else { return Ok(vec![]) };
    if b.v < 1.2 {
        return Ok(vec![]); // a spell is not spent on a trivial creature
    }
    let d = g.db.get(c);
    let mut u = b.v - 3.5;
    if d.instant && post == Some(false) {
        u -= 0.3; // an instant: slightly better held, but a bite before combat clears a blocker
    }
    if d.name.as_ref() == "Knockout Maneuver" {
        u += 0.4; // the counter stays
    }
    Ok(vec![Opt {
        utility: u,
        label: format!("{} -> {}", d.name, g.perm(b.tgt).name),
        act: Some(Action::Plan { f: bite_plan, arg: c.0 as i64 }),
    }])
}

// ------------------------------------------------------------------ Cost of Brilliance
/// Cost of Brilliance: the AI draws two (and loses 2 life) itself and puts the counter on its best creature
fn cost_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let q = ctx.face.filter(|&q| g.player(q).alive).unwrap_or(p);
    draw(g, q, 2, false)?;
    lose_life(g, q, 2, Some(p), "other", None)?;
    let host = ctx.host.filter(|&m| live_creature(g, m)).or_else(|| best_creature(g, p));
    if let Some(m) = host {
        plus_counter(g, m, 1);
    }
    Ok("gy")
}

fn best_creature(g: &Game, p: PlayerId) -> Option<PermId> {
    let cs: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| live_creature(g, m)).collect();
    max_by(&cs, |m| pval(g, m))
}

fn cost_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if human(g, p) || post.is_none() || g.active != Some(p) || !affordable(g, p, c) {
        return Ok(vec![]);
    }
    let pl = g.player(p);
    if pl.life <= 6 || pl.library.len() < 6 {
        return Ok(vec![]);
    }
    let mut u = 3.2 + if pl.hand.len() <= 2 { 1.2 } else { 0.0 } - if pl.life < 12 { 1.0 } else { 0.0 };
    if pl.hand.len() >= 6 {
        u -= 1.5;
    }
    let host = best_creature(g, p);
    u += if host.is_some() { 0.3 } else { 0.0 };
    Ok(vec![Opt {
        utility: u,
        label: "Cost of Brilliance (draw two)".into(),
        act: Some(Action::Plan { f: cost_plan, arg: c.0 as i64 }),
    }])
}

fn cost_plan(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    let host = best_creature(g, p);
    crate::ai::brain::do_cast_with(g, p, c, None, Ctx { face: Some(p), host, ..Ctx::default() })
}

// ------------------------------------------------------------------ Foolish Fate
/// Foolish Fate: destroy target creature; infusion: if you gained life this turn, its controller loses 3 life
fn fate_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let Some(t) = ctx.target.filter(|&t| live(g, t)) else { return Ok("gy") };
    let ctrl = owner(g, t);
    let infused = gained_life(g, p);
    apply_removal(g, Some(p), t, "destroy", Some(c))?;
    if infused {
        lose_life(g, ctrl, 3, Some(p), "other", None)?;
    }
    Ok("gy")
}

// ------------------------------------------------------------------ Witherbloom Charm
const CHARM_DESTROY: &str = "destroy";
const CHARM_DRAW: &str = "draw";
const CHARM_LIFE: &str = "life";

/// the permanent p would sacrifice to Witherbloom Charm's second mode, cheapest first: a Food, a Clue, a worthless
/// creature (a token), then an extra Treasure
fn charm_fodder(g: &Game, p: PlayerId) -> Option<&'static str> {
    let pl = g.player(p);
    if pl.foods > 0 {
        return Some("Food");
    }
    if pl.clues > 0 {
        return Some("Clue");
    }
    let tok = pl.perms.iter().any(|&m| live(g, m) && g.perm(m).token && pval(g, m) <= 1.5 && !g.perm(m).army);
    if tok {
        return Some("token");
    }
    (pl.treasures > 1).then_some("Treasure")
}

fn charm_sacrifice(g: &mut Game, p: PlayerId) -> Res<bool> {
    match charm_fodder(g, p) {
        Some("Food") => crate::impls::common::sac_food(g, p, 1),
        Some("Clue") => {
            g.player_mut(p).clues -= 1;
            Ok(true)
        }
        Some("Treasure") => {
            g.player_mut(p).treasures -= 1;
            Ok(true)
        }
        Some(_) => {
            let toks: Vec<PermId> = g
                .player(p)
                .perms
                .iter()
                .copied()
                .filter(|&m| live(g, m) && g.perm(m).token && pval(g, m) <= 1.5 && !g.perm(m).army)
                .collect();
            let Some(m) = min_by(&toks, |m| pval(g, m)) else { return Ok(false) };
            die(g, m, "sac")?;
            Ok(true)
        }
        None => Ok(false),
    }
}

/// opposing nonland permanents with mana value 2 or less that the charm may destroy (a token has mana value 0)
fn charm_targets(g: &Game, p: PlayerId, c: CardId) -> Vec<PermId> {
    legal_targets(g, p, "destroy", "nl", false, Some(c))
        .into_iter()
        .filter(|&m| g.perm(m).cd.is_none_or(|cd| g.db.get(cd).cmc <= 2))
        .collect()
}

fn charm_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let mode = ctx.mode.unwrap_or(CHARM_LIFE);
    if mode == CHARM_DESTROY {
        if let Some(t) = ctx.target.filter(|&t| live(g, t)) {
            apply_removal(g, Some(p), t, "destroy", Some(c))?;
        }
    } else if mode == CHARM_DRAW {
        // "you may sacrifice a permanent. If you do, draw two cards"
        if charm_sacrifice(g, p)? {
            draw(g, p, 2, false)?;
        }
    } else {
        gain(g, p, 5)?;
    }
    Ok("gy")
}

fn charm_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if human(g, p) || post.is_none() || g.active != Some(p) || !affordable(g, p, c) {
        return Ok(vec![]);
    }
    let mut out = vec![];
    let tg = charm_targets(g, p, c);
    if let Some(best) = max_by(&tg, |m| pval(g, m)) {
        out.push(Opt {
            utility: pval(g, best) - 3.5,
            label: format!("Witherbloom Charm (destroy {})", g.perm(best).name),
            act: Some(Action::Plan { f: charm_plan, arg: c.0 as i64 | 1 << 16 }),
        });
    }
    if charm_fodder(g, p).is_some() && g.player(p).library.len() > 6 {
        let hand = g.player(p).hand.len() as f64;
        out.push(Opt {
            utility: 3.2 - 0.5 * (hand - 2.0).max(0.0),
            label: "Witherbloom Charm (sacrifice, draw two)".into(),
            act: Some(Action::Plan { f: charm_plan, arg: c.0 as i64 | 2 << 16 }),
        });
    }
    let pl = g.player(p);
    let infusion_cards = pl.hand.iter().any(|&x| matches!(name_of(g, x), "Foolish Fate")) as i32
        + pl.perms.iter().any(|&m| g.perm(m).cd.is_some_and(|x| name_of(g, x) == "Tenured Concocter")) as i32;
    if pl.life <= 15 {
        out.push(Opt {
            utility: 1.2 + 0.8 * infusion_cards as f64 + if pl.life <= 8 { 1.5 } else { 0.0 },
            label: "Witherbloom Charm (gain 5)".into(),
            act: Some(Action::Plan { f: charm_plan, arg: c.0 as i64 | 3 << 16 }),
        });
    }
    Ok(out)
}

fn charm_plan(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId((arg & 0xffff) as u16);
    let ctx = match arg >> 16 {
        1 => {
            let tg = charm_targets(g, p, c);
            let Some(t) = max_by(&tg, |m| pval(g, m)) else { return Ok(false) };
            Ctx { target: Some(t), mode: Some(CHARM_DESTROY), ..Ctx::default() }
        }
        2 => Ctx { mode: Some(CHARM_DRAW), ..Ctx::default() },
        _ => Ctx { mode: Some(CHARM_LIFE), ..Ctx::default() },
    };
    crate::ai::brain::do_cast_with(g, p, c, None, ctx)
}

// ------------------------------------------------------------------ Culling Ritual
/// nonland permanents with mana value 2 or less, as (permanent) and the count of Treasure, Food and Clue tokens
fn culling_victims(g: &Game, q: PlayerId) -> Vec<PermId> {
    g.player(q)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            x.on_bf && !x.phased && x.loyalty.is_none_or(|_| true) && x.cd.is_none_or(|cd| g.db.get(cd).cmc <= 2)
        })
        .collect()
}

fn artifact_tokens(g: &Game, q: PlayerId) -> u32 {
    let pl = g.player(q);
    pl.treasures + pl.foods + pl.clues
}

/// Culling Ritual: destroy each nonland permanent with mana value 2 or less (tokens, Treasure, Food and Clues
/// included), then add {B} or {G} for each one destroyed: black for the first half, green for the rest
fn culling_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mut n = 0u32;
    for q in g.players.iter().filter(|x| x.alive).map(|x| x.id).collect::<Vec<_>>() {
        for m in culling_victims(g, q) {
            if live(g, m) {
                die(g, m, "destroy")?;
                if !g.perm(m).on_bf {
                    n += 1;
                }
            }
        }
        let pl = g.player_mut(q);
        n += pl.treasures + pl.foods + pl.clues;
        (pl.treasures, pl.foods, pl.clues) = (0, 0, 0);
    }
    let (b, gr) = mana_split(g, p, n);
    let f = &mut g.player_mut(p).floating;
    f.b += b;
    f.g += gr;
    Ok("gy")
}

/// split n mana between B and G by what the hand still needs
fn mana_split(g: &Game, p: PlayerId, n: u32) -> (u32, u32) {
    let (mut nb, mut ng) = (0u32, 0u32);
    for &c in &g.player(p).hand {
        let d = g.db.get(c);
        nb += d.pips.chars().filter(|&ch| ch == 'B').count() as u32;
        ng += d.pips.chars().filter(|&ch| ch == 'G').count() as u32;
    }
    let b = if nb + ng == 0 { n / 2 } else { (n * nb).div_ceil(nb + ng).min(n) };
    (b, n - b)
}

fn culling_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if human(g, p) || post.is_none() || g.active != Some(p) || !affordable(g, p, c) {
        return Ok(vec![]);
    }
    let mut ol = 0.0;
    let mut ml = 0.0;
    let mut n = 0u32;
    for q in g.players.iter().filter(|x| x.alive).map(|x| x.id).collect::<Vec<_>>() {
        let v: f64 = culling_victims(g, q).iter().map(|&m| pval(g, m).max(0.5)).sum();
        let toks = artifact_tokens(g, q) as f64 * 0.8;
        n += culling_victims(g, q).len() as u32 + artifact_tokens(g, q);
        if q == p {
            ml += v + toks;
        } else {
            ol += v + toks;
        }
    }
    let net = ol - ml + 0.5 * n as f64;
    if ol < 3.0 || net < 3.0 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: net / 2.2 - 2.5 + 1.0,
        label: format!("Culling Ritual ({n} permanents)"),
        act: Some(Action::Plan { f: culling_plan, arg: c.0 as i64 }),
    }])
}

fn culling_plan(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    crate::ai::brain::do_cast_with(g, p, CardId(arg as u16), None, Ctx::default())
}

// ------------------------------------------------------------------ Track Down
/// Track Down: scry 3, then reveal the top card; a creature or land card is drawn
fn track_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    crate::cardcode::scry(g, p, 3, false)?;
    if let Some(&top) = g.player(p).library.last() {
        let d = g.db.get(top);
        if d.creature || d.land {
            draw(g, p, 1, false)?;
        }
    }
    Ok("gy")
}

fn track_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    if g.player(p).library.len() < 8 { 0 } else { 32 }
}

// ------------------------------------------------------------------ Canyon Crawler
/// Canyon Crawler: a Food token as it enters
fn crawler_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src && trigger_window(g, owner(g, src), Some(src), "a Food token", None)? {
        make_artifact_tokens(g, owner(g, src), "Food", 1)?;
    }
    let _ = p;
    Ok(())
}

/// swampcycling {2}: when the hand needs lands, discard it and fetch a Swamp
fn crawler_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if human(g, p) || !can_pay(g, p, 2, "", false) || post == Some(true) {
        return Ok(vec![]);
    }
    let pl = g.player(p);
    let lands = pl.lands.len() + pl.hand.iter().filter(|&&x| g.db.get(x).land).count();
    if !pl.library.iter().any(|&x| name_of(g, x) == "Swamp") || lands >= 7 {
        return Ok(vec![]);
    }
    let hand_lands = pl.hand.iter().filter(|&&x| g.db.get(x).land).count();
    let u = if lands <= 4 && hand_lands == 0 {
        4.5
    } else if lands <= 5 && hand_lands == 0 {
        2.5
    } else {
        return Ok(vec![]);
    };
    Ok(vec![Opt {
        utility: u,
        label: "Canyon Crawler (swampcycling)".into(),
        act: Some(Action::Plan { f: crawler_cycle, arg: c.0 as i64 }),
    }])
}

fn crawler_cycle(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    crate::engine::mana::pay(g, p, 2, "", false)?;
    crate::glog!(g, "  {} swampcycles Canyon Crawler", g.player(p).name);
    discard_cards(g, p, &[c])?;
    if let Some(s) = g.player(p).library.iter().rev().copied().find(|&x| name_of(g, x) == "Swamp") {
        remove_card(&mut g.player_mut(p).library, s);
        g.player_mut(p).hand.push(s);
    }
    shuffle_library(g, p);
    Ok(true)
}

// ------------------------------------------------------------------ cost reductions
/// Lashwhip Predator costs {2} less if your opponents control three or more creatures
fn lashwhip_cost(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    let n: usize = g.opps(p).map(|q| g.player(q).perms.iter().filter(|&&m| live_creature(g, m)).count()).sum();
    if n >= 3 { -2 } else { 0 }
}

/// Writhing Necromass costs {1} less for each creature card in your graveyard
fn necromass_cost(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    -(gy_creatures(g, p) as i32)
}

// ------------------------------------------------------------------ Mabel, Vault Plunderer, Seymour, Bugenhagen
/// Mabel, Bitter Recluse: remove up to three counters from another target creature or planeswalker: +1/+1 counters
/// off the best opposing creature, -1/-1 counters off one of mine, loyalty off an opposing planeswalker
fn mabel_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let p = owner(g, src);
    let mut best: Option<(f64, PermId)> = None;
    let consider = |v: f64, t: PermId, best: &mut Option<(f64, PermId)>| {
        if v >= 1.0 && best.is_none_or(|b| v > b.0) {
            *best = Some((v, t));
        }
    };
    for q in g.players.iter().filter(|x| x.alive).map(|x| x.id).collect::<Vec<_>>() {
        for &t in &g.player(q).perms {
            if t == src || !live(g, t) || (q != p && untargetable(g, t)) {
                continue;
            }
            if q != p && is_walker(g, t) {
                consider(g.perm(t).loyalty.unwrap_or(0).min(3) as f64, t, &mut best);
            } else if q != p && g.is_creature(t) && g.perm(t).plus > 0 {
                consider(g.perm(t).plus.min(3) as f64 * (1.0 + pval(g, t) / 8.0), t, &mut best);
            } else if q == p && g.is_creature(t) && g.perm(t).plus < 0 {
                consider((-g.perm(t).plus).min(3) as f64, t, &mut best);
            }
        }
    }
    let Some((_, t)) = best else { return Ok(()) };
    if !trigger_window(g, p, Some(src), "remove up to three counters from another target", Some(3.0))? || !live(g, t) {
        return Ok(());
    }
    if owner(g, t) != p {
        crime(g, p)?;
    }
    if is_walker(g, t) {
        let l = g.perm(t).loyalty.unwrap_or(0) - 3;
        g.perm_mut(t).loyalty = Some(l);
        if l <= 0 {
            let o = owner(g, t);
            leave(g, t)?;
            to_zone_card(g, t, Zone::Gy);
            let nm = g.perm(t).name;
            *g.player_mut(o).lost_names.entry(nm).or_insert(0) += 1;
        }
    } else {
        let x = g.perm_mut(t);
        x.plus = if x.plus > 0 { x.plus - x.plus.min(3) } else { x.plus + (-x.plus).min(3) };
    }
    Ok(())
}

/// Vault Plunderer: target player draws a card and loses 1 life (you, unless you are nearly dead)
fn plunderer_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let p = owner(g, src);
    let opps: Vec<PlayerId> = g.opps(p).collect();
    let target = if g.player(p).life > 3 && g.player(p).library.len() > 2 {
        p
    } else if let Some(q) = min_by(&opps, |q| g.player(q).life as f64) {
        q
    } else {
        return Ok(());
    };
    if trigger_window(g, p, Some(src), "target player draws a card and loses 1 life", Some(3.0))? {
        if target != p {
            crime(g, p)?;
        }
        draw(g, target, 1, false)?;
        lose_life(g, target, 1, Some(p), "other", None)?;
    }
    Ok(())
}

/// Seymour Flux: at the beginning of your upkeep, you may pay 1 life: draw a card and a +1/+1 counter on it
fn seymour_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || g.player(p).life <= 8 || g.player(p).library.len() < 3 {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "pay 1 life: draw a card and a +1/+1 counter", Some(4.0))? && live(g, src) {
        lose_life(g, p, 1, Some(p), "other", None)?;
        draw(g, p, 1, false)?;
        plus_counter(g, src, 1);
    }
    Ok(())
}

/// Bugenhagen, Wise Elder: at the beginning of your upkeep, if you control a creature with power 7 or greater, draw
fn bugenhagen_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) {
        return Ok(());
    }
    let big = g.player(p).perms.iter().any(|&m| live_creature(g, m) && epow(g, m) >= 7);
    if big && trigger_window(g, p, Some(src), "draw a card (a creature with power 7 or greater)", Some(4.0))? {
        draw(g, p, 1, false)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Armored Scrapgorger
fn oil(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.int(DataKey::Oil)
}

/// Armored Scrapgorger: whenever it becomes tapped (for its mana), exile target card from a graveyard and put an oil
/// counter on it. The target: the best creature card of an opponent's graveyard (a crime), else any card of theirs, else
/// my own least useful card.
fn scrapgorger_tap(g: &mut Game, p: PlayerId, m: PermId, _n: u32) -> Res {
    let mut best: Option<(f64, PlayerId, CardId)> = None;
    for q in g.players.iter().filter(|x| x.alive).map(|x| x.id).collect::<Vec<_>>() {
        for &c in &g.player(q).gy {
            let d = g.db.get(c);
            let v = if q != p {
                10.0 + d.cmc as f64 + if d.creature { 20.0 } else { 0.0 }
            } else if d.creature {
                -20.0 + d.cmc as f64 * -1.0
            } else {
                -(d.cmc as f64) - if d.land { 0.0 } else { 1.0 }
            };
            if best.is_none_or(|b| v > b.0) {
                best = Some((v, q, c));
            }
        }
    }
    let Some((_, q, c)) = best else { return Ok(()) };
    remove_card(&mut g.player_mut(q).gy, c);
    g.player_mut(q).exile.push(c);
    let n = oil(g, m) + 1;
    g.perm_mut(m).data.set(DataKey::Oil, Val::Int(n));
    crate::glog!(g, "    Armored Scrapgorger exiles {} (oil {})", g.db.get(c).name, n);
    if q != p {
        crime(g, p)?;
    }
    Ok(())
}

fn scrapgorger_pt(g: &Game, _p: PlayerId, m: PermId) -> (i32, i32) {
    if oil(g, m) >= 3 { (3, 0) } else { (0, 0) }
}

fn selfpt_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m == src {
        g.selfpt = true; // the engine reads this creature's size from its own rule from here on
    }
    Ok(())
}

// ------------------------------------------------------------------ Tenured Concocter
fn concocter_pt(g: &Game, p: PlayerId, _m: PermId) -> (i32, i32) {
    if gained_life(g, p) { (2, 0) } else { (0, 0) }
}

/// m becomes the target of a spell or ability a player other than its controller controls (`engine::removal::apply_removal`)
pub fn becomes_target(g: &mut Game, m: PermId, _by: PlayerId) -> Res {
    if g.perm(m).cd.is_none_or(|c| name_of(g, c) != "Tenured Concocter") || !live(g, m) {
        return Ok(());
    }
    let o = owner(g, m);
    if g.player(o).library.len() > 2 && trigger_window(g, o, Some(m), "you may draw a card", Some(3.0))? {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

// ------------------------------------------------------------------ Paladin of Predation
/// Paladin of Predation: toxic 6 (six poison counters with the combat damage). It can't be blocked by creatures with
/// power 2 or less: `rules::evasion_blocked`.
fn paladin_damage(g: &mut Game, src: Src, _p: PlayerId, a: PermId, d: PlayerId, dmg: i32) -> Res {
    if a == src && dmg > 0 && !crate::engine::values::melira(g, d) {
        g.player_mut(d).poison += 6;
        crate::glog!(g, "    {} gets 6 poison counters (toxic)", g.player(d).name);
    }
    Ok(())
}

// ------------------------------------------------------------------ Hunter's Axe
/// Hunter's Axe: whenever the equipped creature attacks, it gains trample or deathtouch until end of turn (deathtouch
/// when it lacks it: any blocker dies; else trample)
fn axe_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    let Some(host) = g.perm(src).attached else { return Ok(vec![]) };
    if owner(g, src) != p || !atk.contains(&host) || !live_creature(g, host) {
        return Ok(vec![]);
    }
    if trigger_window(g, p, Some(src), "the equipped creature gains trample or deathtouch", None)? {
        eot_kw(g, host, if deathtouch(g, host) { "trample" } else { "deathtouch" });
    }
    Ok(vec![])
}

// ------------------------------------------------------------------ Darksteel Plate
/// the AI plan: Glissa wears Darksteel Plate (indestructible, so her first strike and deathtouch never cost her the
/// game): equip it to her ahead of the generic equip plays. (Champion's Helm: `yshtola::helm`, which also serves this deck.)
fn plate_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if p != owner(g, src)
        || g.player(p).key != "glissa"
        || human(g, p)
        || post.is_none()
        || !can_pay(g, p, 2, "", false)
    {
        return Ok(vec![]);
    }
    let cmd = g.player(p).perms.iter().copied().find(|&m| live_creature(g, m) && g.perm(m).is_cmd);
    let Some(t) = cmd.filter(|&t| g.perm(src).attached != Some(t)) else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: 4.5,
        label: "equip Darksteel Plate to Glissa".into(),
        act: Some(Action::Equip { src, target: t, n: 2 }),
    }])
}

// ------------------------------------------------------------------ Ferocity of the Hunt
/// Ferocity of the Hunt: when the enchanted creature dies, its card returns to the battlefield tapped (the mark is read
/// by `zones::die_rest`)
fn ferocity_dies(g: &mut Game, _p: PlayerId, _aura: PermId, host: PermId) -> Res {
    if g.perm(host).cd.is_some() {
        g.perm_mut(host).data.set(DataKey::Ferocity, Val::Bool(true));
    }
    Ok(())
}

static FEROCITY: AuraSpec = AuraSpec { pow: 1, kws: &["deathtouch"], host_dies: Some(ferocity_dies), ..AURA };

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    // the bites, the fight, Cost of Brilliance, Witherbloom Charm, Culling Ritual: cast by their own options
    for name in [
        "Bite Down",
        "Hard-Hitting Question",
        "Compel Brutality",
        "Diplomatic Relations",
        "Burrog Barrage",
        "Knockout Maneuver",
        "Chelonian Tackle",
    ] {
        let c = r.card(db, name)?;
        c.resolve = Some(bite_resolve);
        c.hand_options = Some(bite_options);
        c.prio = Some(zero_prio);
    }
    let c = r.card(db, "Cost of Brilliance")?;
    c.resolve = Some(cost_resolve);
    c.hand_options = Some(cost_options);
    c.prio = Some(zero_prio);
    let c = r.card(db, "Witherbloom Charm")?;
    c.resolve = Some(charm_resolve);
    c.hand_options = Some(charm_options);
    c.prio = Some(zero_prio);
    let c = r.card(db, "Culling Ritual")?;
    c.resolve = Some(culling_resolve);
    c.hand_options = Some(culling_options);
    c.prio = Some(zero_prio);
    let c = r.card(db, "Track Down")?;
    c.resolve = Some(track_resolve);
    c.prio = Some(track_prio);
    r.card(db, "Foolish Fate")?.resolve = Some(fate_resolve);
    // creatures
    let c = r.card(db, "Canyon Crawler")?;
    c.etb = Some(crawler_etb);
    c.hand_options = Some(crawler_options);
    r.card(db, "Lashwhip Predator")?.self_cost = Some(lashwhip_cost);
    r.card(db, "Writhing Necromass")?.self_cost = Some(necromass_cost);
    r.card(db, "Lecturing Scornmage")?.cast = Some(scornmage_cast);
    r.card(db, "Melancholic Poet")?.cast = Some(poet_cast);
    r.card(db, "Forum Necroscribe")?.cast = Some(necroscribe_cast);
    r.card(db, "Hardbristle Bandit")?.cast = Some(bandit_cast);
    r.card(db, "Mabel, Bitter Recluse")?.etb = Some(mabel_etb);
    r.card(db, "Vault Plunderer")?.etb = Some(plunderer_etb);
    r.card(db, "Seymour Flux")?.upkeep = Some(seymour_upkeep);
    r.card(db, "Bugenhagen, Wise Elder")?.upkeep = Some(bugenhagen_upkeep);
    let c = r.card(db, "Armored Scrapgorger")?;
    c.on_tap_perm = Some(scrapgorger_tap);
    c.self_pt = Some(scrapgorger_pt);
    c.etb = Some(selfpt_etb);
    let c = r.card(db, "Tenured Concocter")?;
    c.self_pt = Some(concocter_pt);
    c.etb = Some(selfpt_etb);
    r.card(db, "Paladin of Predation")?.combat_damage = Some(paladin_damage);
    r.card(db, "Hunter's Axe")?.attack = Some(axe_attack);
    r.card(db, "Darksteel Plate")?.options = Some(plate_options);
    aura(r, db, "Ferocity of the Hunt", &FEROCITY)?;
    Ok(())
}
