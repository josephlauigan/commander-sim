//! Tier 6 "mana" batch: the mana rocks, rituals, dorks and mana-adjacent artifacts of the Tier 6 decks that the
//! compiled Oracle text and the engine's tags don't cover (the audit verdicts are in data/card_overrides.toml; each
//! card's doc comment says how it is modeled and what is left out; what needs the shared engine is in
//! rust/agents/engine-needs-mana.md).

use super::partials::at_once;
use crate::cards::CardDb;
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::trigger_window;
use crate::engine::zones::{die, draw};
use crate::flow::Res;
use crate::hooks::{Action, Event, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::Sym;

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn ability(utility: f64, label: &str, src: PermId, f: crate::hooks::AbilityFn, arg: i64) -> Opt {
    Opt { utility, label: label.into(), act: Some(Action::Ability { src, f, arg }) }
}

// ================================================================== Chromatic Star
/// **Chromatic Star** ({1} artifact): {1}, {T}, sacrifice: one mana of any colour (a mana ability: floating mana of
/// any colour of the player's identity); when it is put into a graveyard from the battlefield, draw a card (any way
/// it dies: sacrificed or destroyed). The AI cracks it for the card in the second main phase or the end-of-turn
/// window when it has a spare {1}. The data override drops the old `rock` tag: Star is not a free mana rock.
fn star_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post == Some(false) || !live(g, src) || owner(g, src) != p || g.perm(src).tapped || !can_pay(g, p, 1, "", false)
    {
        return Ok(vec![]);
    }
    Ok(vec![ability(1.0, "crack Chromatic Star", src, star_go, 0)])
}

fn star_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).tapped {
        return Ok(false);
    }
    g.perm_mut(src).tapped = true;
    if !pay(g, p, 1, "", false)? {
        g.perm_mut(src).tapped = false;
        return Ok(false);
    }
    crate::glog!(g, "  {} cracks Chromatic Star", g.player(p).name);
    die(g, src, "sac")?;
    g.player_mut(p).floating.any += 1;
    Ok(true)
}

/// when it is put into a graveyard from the battlefield: draw a card
fn star_dies(g: &mut Game, _src: Src, m: PermId, _cause: Sym) -> Res {
    let o = owner(g, m);
    if trigger_window(g, o, Some(m), "draw a card", None)? {
        draw(g, o, 1, false)?;
    }
    Ok(())
}

// ================================================================== Desperate Ritual
/// **Desperate Ritual** ({1}{R} Arcane instant): add {R}{R}{R} (floating red mana). Splice onto Arcane is not
/// modeled: no other Arcane spell is played alongside it in any deck (Godo is the only deck that plays it). Like
/// Cabal Ritual, the AI casts it only through a deck plan (priority 0).
fn desperate_ritual(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    g.player_mut(p).floating.r += 3;
    crate::glog!(g, "    {} adds RRR (Desperate Ritual)", g.player(p).name);
    Ok("gy")
}

fn zero_prio(_g: &Game, _p: PlayerId, _c: CardId) -> i32 {
    0
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, "Chromatic Star")?;
    c.options = Some(star_options);
    c.self_dies = Some(star_dies);
    at_once(c, Event::SelfDies, false);
    let c = r.card(db, "Desperate Ritual")?;
    c.resolve = Some(desperate_ritual);
    c.prio = Some(zero_prio);
    Ok(())
}
