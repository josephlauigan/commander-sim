//! Alternative costs for spells cast from hand (Daze, Force of Will-style pitch, "exile a red card"), and the cost
//! increases that still apply to them (Defense Grid, Trinisphere, Thalia).
//!
//! A card with an alternative cost has an `alt_cost` slot (hooks.rs `AltCost`): `ok` says whether it can be paid now,
//! `pay` pays it, `lost` says how many cards' worth of value it gives up (the AI's price). The engine asks through
//! this module, so a card needs no special case in the casting code.
//!
//! The AI's rule, as for Force of Will: pay the alternative cost when the mana cost can't be paid, or when it costs
//! nothing of value (`lost == 0`, Mindbreak Trap's free mode). Each card lost raises the bar for casting: counters
//! raise the counter threshold by 1.5 per card (`stack::ai_respond`), removal and wipes lower their utility by 1.5.

use crate::engine::cast::castable;
use crate::engine::mana::{can_pay, pay};
use crate::flow::Res;
use crate::hooks::AltCost;
use crate::ids::{CardId, PlayerId};
use crate::state::Game;

/// the card's alternative-cost slot, if it has one
pub fn slot(g: &Game, c: CardId) -> Option<AltCost> {
    g.registry.get(c).and_then(|i| i.alt_cost)
}

/// The mana a spell cast without paying its mana cost still costs: cost increases (Defense Grid's {3} off the caster's
/// turn, Thalia, Trinisphere's floor of three), less any reductions, never below 0.
pub fn tax(g: &Game, p: PlayerId, c: CardId) -> u32 {
    let mut t = crate::dsl::cost_delta(g, p, c) + crate::impls::t0_stella::kaza_delta(g, p, c);
    let mut floor = 0;
    if !g.hooks.is_empty() {
        t += crate::engine::hooks::total_cost(g, p, c);
        floor = crate::engine::hooks::max_min_cost(g, p, c);
    }
    t.max(floor).max(0) as u32
}

/// can p pay the tax of casting c for free (always true when nothing taxes it)
pub fn tax_payable(g: &Game, p: PlayerId, c: CardId) -> bool {
    let t = tax(g, p, c);
    t == 0 || can_pay(g, p, t, "", false)
}

/// pay the tax of a free cast; false (nothing paid) if p can't
pub fn pay_tax(g: &mut Game, p: PlayerId, c: CardId) -> Res<bool> {
    let t = tax(g, p, c);
    if t == 0 {
        return Ok(true);
    }
    pay(g, p, t, "", false)
}

/// can p cast `c` from hand for its alternative cost now (the cost itself, the card's locks, and the tax)? `spell` is
/// the spell it answers, if any (Disrupting Shoal needs its mana value).
pub fn available(g: &Game, p: PlayerId, c: CardId, spell: Option<CardId>) -> bool {
    let Some(a) = slot(g, c) else { return false };
    g.player(p).hand.contains(&c)
        && (a.ok)(g, p, c, spell)
        && (!g.cast_checks() || castable(g, p, c, "hand"))
        && tax_payable(g, p, c)
}

/// cards' worth of value the alternative cost gives up (0 without a slot)
pub fn lost(g: &Game, p: PlayerId, c: CardId, spell: Option<CardId>) -> f64 {
    slot(g, c).map_or(0.0, |a| (a.lost)(g, p, c, spell))
}

/// should the AI pay `c`'s alternative cost, given that its mana cost is `(generic, pips)`? Yes when it can and the mana
/// cost can't be paid, or the alternative costs nothing of value.
pub fn use_alt(g: &Game, p: PlayerId, c: CardId, spell: Option<CardId>, generic: u32, pips: &str) -> bool {
    available(g, p, c, spell) && (!can_pay(g, p, generic, pips, false) || lost(g, p, c, spell) <= 0.0)
}

/// pay `c`'s alternative cost and its tax. False, nothing paid, if it can't be paid.
pub fn pay_alt(g: &mut Game, p: PlayerId, c: CardId, spell: Option<CardId>) -> Res<bool> {
    if !available(g, p, c, spell) {
        return Ok(false);
    }
    let a = slot(g, c).unwrap();
    // the mana the tax and the alternative cost spend adds up as the mana spent on the spell
    g.mana_spent = 0;
    g.mana_colored = false;
    g.spent_keep = true;
    let r = (|| -> Res<bool> {
        if !pay_tax(g, p, c)? {
            return Ok(false);
        }
        (a.pay)(g, p, c, spell)
    })();
    g.spent_keep = false;
    r
}
