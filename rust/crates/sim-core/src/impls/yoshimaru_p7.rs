//! The Yoshimaru / Tymna deck, part 7a: the person paths and models that were missing.
//!
//! # The Wandering Emperor
//! {2}{W}{W}, flash, loyalty 4. "As long as The Wandering Emperor entered this turn, you may activate her loyalty
//! abilities any time you could cast an instant" (`human/cards.rs` `loyalty`: a person's timing check).
//! - +1: put a +1/+1 counter on up to one target creature (any player's); it gains first strike until end of turn
//!   (`emp_plus`, shared by the person's walker in `human/walkers.rs`).
//! - -1: a 2/2 white Samurai creature token with vigilance (`emp_samurai`).
//! - -2: exile target tapped creature (any player's, your own included); you gain 2 life (`human/walkers.rs`).
//!
//! # Eiganjo, Seat of the Empire
//! Channel {2}{W}, discard it: 4 damage to target attacking or blocking creature; {1} less for each legendary creature
//! you control. The AI's channel is `t6_lands.rs` (`hand_defend` / `hand_blocks`); a person's is `human/cards.rs`
//! (`eiganjo`: any player's attacking or blocking creature, at instant speed, Pithing Needle respected). This file
//! lists the creatures it may aim at (`combatants`) and when the person's declare-blockers stop is worth it
//! (`eiganjo_ready`).

use crate::cards::Colors;
use crate::engine::repl;
use crate::engine::values::{colors_of, protected_from};
use crate::engine::zones::{Tokens, make_tokens};
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;

const EMPEROR: &str = "The Wandering Emperor";

/// may p's The Wandering Emperor (`src`) target creature t (white: protection from white stops it)
pub fn emp_legal(g: &Game, p: PlayerId, src: PermId, t: PermId) -> bool {
    g.is_creature(t) && super::vondam_elspeth::may_target(g, p, t) && !protected_from(g, t, colors_of(g, src))
}

/// +1: a +1/+1 counter on t, put by p (Hardened Scales and Doubling Season look at the one who puts it), and first
/// strike until end of turn
pub fn emp_plus(g: &mut Game, p: PlayerId, t: PermId) -> Res {
    if !g.perm(t).on_bf {
        return Ok(());
    }
    repl::put_counters(g, p, t, 1);
    super::partials::eot_kw(g, t, "first strike");
    crate::glog!(g, "    {EMPEROR}: a +1/+1 counter on {} and first strike until end of turn", g.perm(t).name);
    Ok(())
}

/// -1: a 2/2 white Samurai creature token with vigilance
pub fn emp_samurai(g: &mut Game, p: PlayerId) -> Res<Vec<PermId>> {
    let spec = Tokens {
        tgh: Some(2),
        color: Some(crate::cards::Colors::from_letters("W")),
        types: vec!["samurai"],
        ..Tokens::new(1, 2)
    };
    let toks = make_tokens(g, p, spec)?;
    for &t in &toks {
        g.perm_mut(t).vig = true;
    }
    Ok(toks)
}

// ------------------------------------------------------------------ Eiganjo, Seat of the Empire
/// the attacking and blocking creatures an ability of p's (from a source of colours `source`) may target: on the
/// battlefield, hexproof and shroud respected, protection from the source's colours (from colorless: a colorless
/// source) too
pub fn combatants(g: &Game, p: PlayerId, source: Colors) -> Vec<PermId> {
    let mut fighting = g.in_combat.clone();
    for &x in &g.blocking {
        if !fighting.contains(&x) {
            fighting.push(x);
        }
    }
    fighting
        .into_iter()
        .filter(|&x| {
            g.perm(x).on_bf
                && g.player(g.perm(x).owner).perms.contains(&x)
                && g.is_creature(x)
                && super::vondam_elspeth::may_target(g, p, x)
                && !protected_from(g, x, source)
        })
        .collect()
}

/// card c in q's hand is Eiganjo and q could channel it now (`have` is the mana q has): a reason to stop at the declare
/// blockers step, as an instant would be
pub fn eiganjo_ready(g: &Game, q: PlayerId, c: CardId, have: u32) -> bool {
    let name = &*g.db.get(c).name;
    name == "Eiganjo, Seat of the Empire"
        && !super::t6_stax::needle_stops(g, name)
        && have > super::t6_lands::eiganjo_cost(g, q)
        && !combatants(g, q, Colors::NONE).is_empty()
}
