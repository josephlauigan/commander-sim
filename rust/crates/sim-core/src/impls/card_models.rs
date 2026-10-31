//! Card code for the Partial cards of JD's decks that the ability language can't express (card-models PRs, 10-30).
//! Each function says which Oracle sentence it plays; `tests/cards_jd_models.rs` checks them.

use crate::cards::CardDb;
use crate::engine::mana::{can_pay, pay};
use crate::engine::values::threat;
use crate::engine::zones::{draw, min_by};
use crate::flow::Res;
use crate::hooks::{AltCost, Registry};
use crate::ids::{CardId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::{Sym, intern};

// ------------------------------------------------------------------ Baleful Mastery (Sauron)
const BALEFUL: &str = "Baleful Mastery";

/// "You may pay {1}{B} rather than pay this spell's mana cost."
fn baleful_ok(g: &Game, p: PlayerId, _c: CardId, _spell: Option<CardId>) -> bool {
    can_pay(g, p, 1, "B", false)
}

/// pays {1}{B} and remembers it for the resolution ("if the {1}{B} cost was paid, an opponent draws a card")
fn baleful_pay(g: &mut Game, p: PlayerId, _c: CardId, _spell: Option<CardId>) -> Res<bool> {
    if !pay(g, p, 1, "B", false)? {
        return Ok(false);
    }
    let st = g.turn_stamp();
    g.player_mut(p).flag_turn.insert(intern("baleful_alt"), st);
    Ok(true)
}

/// an opponent's card: about half a card of ours. The AI takes the cheap cost only when {3}{B} can't be paid
/// (altcost::use_alt)
const BALEFUL_ALT: AltCost = AltCost { ok: baleful_ok, pay: baleful_pay, lost: |_, _, _, _| 0.5 };

/// "If the {1}{B} cost was paid, an opponent draws a card. Exile target creature or planeswalker." The opponent who
/// draws is the one the caster fears least; then the compiled exile resolves
fn baleful_resolve(g: &mut Game, p: PlayerId, c: CardId, ctx: &Ctx) -> Res<Sym> {
    let st = g.turn_stamp();
    if g.player_mut(p).flag_turn.shift_remove("baleful_alt") == Some(st) {
        let opps: Vec<PlayerId> = g.opps(p).collect();
        if let Some(q) = min_by(&opps, |q| threat(g, p, q)) {
            crate::glog!(
                g,
                "  {} paid {{1}}{{B}} for Baleful Mastery: {} draws a card",
                g.player(p).name,
                g.player(q).name
            );
            draw(g, q, 1, false)?;
        }
    }
    crate::dsl::resolve_spell(g, p, c, ctx)?;
    Ok("gy")
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    if db.id(BALEFUL).is_some() {
        let c = r.card(db, BALEFUL)?;
        c.alt_cost = Some(BALEFUL_ALT);
        c.resolve = Some(baleful_resolve);
    }
    Ok(())
}
