//! The Yoshimaru / Tymna deck, part 7a: the person paths and models that were missing.
//!
//! # The Wandering Emperor
//! {2}{W}{W}, flash, loyalty 4. "As long as The Wandering Emperor entered this turn, you may activate her loyalty
//! abilities any time you could cast an instant" (`human/cards.rs` `loyalty`: a person's timing check).
//! - +1: put a +1/+1 counter on up to one target creature (any player's); it gains first strike until end of turn
//!   (`emp_plus`, shared by the person's walker in `human/walkers.rs`).
//! - -1: a 2/2 white Samurai creature token with vigilance (`emp_samurai`).
//! - -2: exile target tapped creature (any player's, your own included); you gain 2 life (`human/walkers.rs`).

use crate::engine::repl;
use crate::engine::values::{colors_of, protected_from};
use crate::engine::zones::{Tokens, make_tokens};
use crate::flow::Res;
use crate::ids::{PermId, PlayerId};
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
