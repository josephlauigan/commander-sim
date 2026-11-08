//! --- c5-oracle-vs-tier: the setup side of `commander-sim --mulligan-oracle KEYS --vs-tier`. Trace only: nothing here
//! runs in an ordinary game, and `engine::turn::mulligan` / `setup_game` are untouched.
//!
//! - `setup_pod_playout`: a real pod game (the deck and three opponents, in the seat order pool mode draws) set up for
//!   one of the two `mull_oracle::Arm`s. The deck's seat takes the arm; every other seat shuffles and mulligans as
//!   `setup_game` has it. All streams are named by `name` (`play:{name}`, `lib:{name}:{key}`), as in `setup_game`
//!   with `seed = name`, so both arms of one playout share the opponents' libraries, hands and the play stream, and
//!   `Arm::Mull { from_try: 0 }` is `setup_game` itself.
//! - `hand_features`: what an opening hand holds, for the report's splits and the CSV.

use crate::cards::CardDb;
use crate::engine::turn::{Seat, mulligan};
use crate::ids::{CardId, PlayerId};
use crate::mull_oracle::{Arm, mulligan_from};
use crate::rng::Rng;
use crate::settings::Settings;
use crate::state::Game;
use crate::tag::Tag;
use std::sync::Arc;

/// the own-turn cap of a pod game (pool mode's `Engine::play` is played to 20 rounds)
pub const POD_ROUNDS: u32 = 20;

/// what an opening hand holds
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HandFeat {
    pub lands: usize,
    /// lands plus the mana the hand's pieces make by turn 2 (`t6_keep::mana`), for every deck
    pub eff: f64,
    /// fast mana: zero-cost and one-cost rocks, Moxen, Sol Ring, Lotus Petal, Chrome Mox, Mox Diamond, and mana dorks and
    /// two-mana rocks (the `fastmana`, `rock` and `dork` tags up to mana value 2)
    pub fast_mana: usize,
    /// cards that fetch a card (the planner's `tutor_rule`)
    pub tutors: usize,
    /// cards the deck's plan file names in a win line or as an engine (0 for a deck with no plan file)
    pub payoffs: usize,
}

fn one_seat_game(
    db: &Arc<CardDb>,
    registry: &Arc<crate::hooks::Registry>,
    settings: &Arc<Settings>,
    seat: &Seat,
) -> Game {
    Game::new(
        db.clone(),
        registry.clone(),
        settings.clone(),
        vec![seat.player(PlayerId(0))],
        Rng::named("hand-features"),
    )
}

/// the features of `hand` for the deck of `seat`
pub fn hand_features(
    db: &Arc<CardDb>,
    registry: &Arc<crate::hooks::Registry>,
    settings: &Arc<Settings>,
    seat: &Seat,
    hand: &[CardId],
) -> HandFeat {
    let mut g = one_seat_game(db, registry, settings, seat);
    let p = PlayerId(0);
    g.player_mut(p).hand = hand.to_vec();
    let eff = crate::impls::t6_keep::mana(&g, p).eff;
    let (mut lands, mut fast_mana, mut tutors, mut payoffs) = (0, 0, 0, 0);
    let plan = crate::ai::plan_data::active(&g, p);
    for &c in hand {
        let d = g.db.get(c);
        if d.land {
            lands += 1;
            continue;
        }
        let name: &str = &d.name;
        if d.tag(Tag::Fastmana)
            || ((d.tag(Tag::Rock) || d.tag(Tag::Dork)) && d.cmc <= 2)
            || matches!(name, "Lotus Petal" | "Chrome Mox" | "Mox Diamond")
        {
            fast_mana += 1;
        }
        if crate::ai::planner::tutor_rule(&g, c).is_some() {
            tutors += 1;
        }
        if let Some(l) = plan
            && (l.lines.iter().any(|(gs, _)| gs.iter().flatten().any(|x| x == name))
                || l.plan.engines.iter().any(|x| x == name))
        {
            payoffs += 1;
        }
    }
    HandFeat { lands, eff, fast_mana, tutors, payoffs }
}

/// A pod game of `seats` (any order; the deck under test is seat `me`) set up for an arm, before turn one. The seats
/// other than `me` are set up as `setup_game` sets them up.
pub fn setup_pod_playout(
    db: &Arc<CardDb>,
    registry: &Arc<crate::hooks::Registry>,
    settings: &Arc<Settings>,
    seats: &[Seat],
    me: usize,
    name: &str,
    diag_seed: u64,
    arm: &Arm,
) -> Game {
    crate::ai::evdiag::set_seed(diag_seed);
    crate::ai::learned::game_start();
    let players = seats.iter().enumerate().map(|(i, s)| s.player(PlayerId(i as u8))).collect();
    let mut g = Game::new(db.clone(), registry.clone(), settings.clone(), players, Rng::named(&format!("play:{name}")));
    for i in 0..g.players.len() {
        let p = PlayerId(i as u8);
        let mut r = Rng::named(&format!("lib:{name}:{}", g.player(p).key));
        let mut lib = std::mem::take(&mut g.player_mut(p).library);
        if i != me {
            r.shuffle(&mut lib);
            g.player_mut(p).library = lib;
            mulligan(&mut g, p, &mut r);
        } else {
            match arm {
                // as `mull_oracle::setup_playout`'s KEEP arm: the hand is fixed, the rest reshuffled
                Arm::Keep { hand, tries } => {
                    for &c in *hand {
                        if let Some(j) = lib.iter().position(|&x| x == c) {
                            lib.remove(j);
                        }
                    }
                    r.shuffle(&mut lib);
                    let pl = g.player_mut(p);
                    pl.library = lib;
                    pl.hand = hand.to_vec();
                    if pl.t6 {
                        pl.stats.insert("mull_try", *tries);
                    }
                    if *tries >= 1 {
                        pl.stats.insert("mull_n", *tries);
                    }
                }
                Arm::Mull { from_try } => {
                    r.shuffle(&mut lib);
                    g.player_mut(p).library = lib;
                    mulligan_from(&mut g, p, &mut r, *from_try);
                }
            }
        }
        crate::impls::card_models::caverns_opening(&mut g, p);
        let hand = g.player(p).hand.clone();
        g.player_mut(p).seen.extend(hand);
    }
    crate::gf_audit::start(&mut g);
    g
}

/// the deck (seat `me`) won, as pool mode counts a win (`winner == me`, a timeout's strongest player included)
pub fn won(g: &Game, me: usize) -> bool {
    g.winner == Some(PlayerId(me as u8))
}
