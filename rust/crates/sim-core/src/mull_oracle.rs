//! --- c4-mulligan-oracle: the setup side of `commander-sim --mulligan-oracle`. Trace only: nothing here runs in an
//! ordinary game, and `engine::turn::mulligan` / `setup_game` are untouched (these are separate functions that
//! follow the same steps).
//!
//! - `opening_tries`: the deck's first seven (try 0) and, when the AI's keep rule mulligans it, the second seven
//!   (try 1), dealt from the same streams `setup_game` uses, with the rule's decision on each;
//! - `setup_playout`: a goldfish game from one of two arms. KEEP starts with a given hand and the rest of the library
//!   reshuffled; MULL puts the hand back and follows `mulligan` from the next try (`mulligan_from`).

use crate::cards::CardDb;
use crate::engine::turn::{Seat, bottom_pick, keeps_hand};
use crate::ids::{CardId, PlayerId};
use crate::rng::Rng;
use crate::settings::Settings;
use crate::state::Game;
use std::sync::Arc;

/// the own-turn cap of an oracle playout; a game with no kill by then scores `NO_KILL`
pub const ROUNDS: u32 = 10;

/// the score of a playout with no kill by turn `ROUNDS`
pub const NO_KILL: u32 = ROUNDS + 1;

/// one dealt seven and the AI's decision on it
#[derive(Debug, Clone, PartialEq)]
pub struct TryHand {
    /// 0 the first seven, 1 the second (the free mulligan)
    pub tries: i64,
    pub hand: Vec<CardId>,
    /// the rule keeps this hand
    pub keeps: bool,
    pub lands: usize,
    /// a Tier 6 deck: the hand's effective mana as `t6_keep` scores it
    pub eff: Option<f64>,
}

const P0: PlayerId = PlayerId(0);

/// shuffle the library and take seven cards: `mulligan`'s `draw7`
fn draw7(g: &mut Game, p: PlayerId, rng: &mut Rng) {
    let pl = g.player_mut(p);
    let hand = std::mem::take(&mut pl.hand);
    pl.library.extend(hand);
    rng.shuffle(&mut pl.library);
    for _ in 0..7 {
        let c = pl.library.pop().unwrap();
        pl.hand.push(c);
    }
}

/// `mulligan`'s `ok`: a Tier 6 deck is told which draw this is, then the keep rule
fn rule_keeps(g: &mut Game, p: PlayerId, tries: i64) -> bool {
    if g.player(p).t6 {
        g.player_mut(p).stats.insert("mull_try", tries);
    }
    keeps_hand(g, p)
}

fn new_game(
    db: &Arc<CardDb>,
    registry: &Arc<crate::hooks::Registry>,
    settings: &Arc<Settings>,
    seats: &[Seat],
    play: &str,
) -> Game {
    let players = seats.iter().enumerate().map(|(i, s)| s.player(PlayerId(i as u8))).collect();
    Game::new(db.clone(), registry.clone(), settings.clone(), players, Rng::named(play))
}

/// the deck's (seat 0) hands as the real game deals them for `seed` (stream `lib:{seed}:{key}`): the first seven, and
/// the second when the rule mulligans the first
pub fn opening_tries(
    db: &Arc<CardDb>,
    registry: &Arc<crate::hooks::Registry>,
    settings: &Arc<Settings>,
    seed: u64,
    seats: &[Seat],
) -> Vec<TryHand> {
    let mut g = new_game(db, registry, settings, seats, &format!("play:{seed}"));
    let mut r = Rng::named(&format!("lib:{seed}:{}", g.player(P0).key));
    let mut lib = std::mem::take(&mut g.player_mut(P0).library);
    r.shuffle(&mut lib);
    g.player_mut(P0).library = lib;
    let mut out = vec![];
    for tries in 0..2 {
        draw7(&mut g, P0, &mut r);
        let keeps = rule_keeps(&mut g, P0, tries);
        let hand = g.player(P0).hand.clone();
        let lands = hand.iter().filter(|&&c| g.db.get(c).land).count();
        let eff = g.player(P0).t6.then(|| crate::impls::t6_keep::mana(&g, P0).eff);
        out.push(TryHand { tries, hand, keeps, lands, eff });
        if keeps {
            break;
        }
    }
    out
}

/// `engine::turn::mulligan` from draw `start` on (0 = all of it): the rule-driven keeps and the bottoming, with the
/// same random draws, so `mulligan_from(.., 0)` is `mulligan`
pub fn mulligan_from(g: &mut Game, p: PlayerId, rng: &mut Rng, start: i64) {
    for t in start..=3 {
        draw7(g, p, rng);
        let keep = rule_keeps(g, p, t);
        match t {
            0 => {
                if keep {
                    return;
                }
            }
            1 => {
                if keep {
                    g.player_mut(p).stats.insert("mull_n", 1);
                    return;
                }
            }
            _ => {
                if keep || t == 3 {
                    let bottom = t - 1;
                    for _ in 0..bottom {
                        let c = bottom_pick(g, p).unwrap();
                        let pl = g.player_mut(p);
                        let i = pl.hand.iter().position(|&x| x == c).unwrap();
                        pl.hand.remove(i);
                        pl.library.insert(0, c);
                    }
                    g.player_mut(p).stats.insert("mulls", bottom + 1);
                    g.player_mut(p).stats.insert("mull_n", bottom + 1);
                    return;
                }
            }
        }
    }
}

/// what the deck does with the hand it was dealt
pub enum Arm<'a> {
    /// keep this hand (dealt as draw `tries`: 0 or 1) and play on
    Keep { hand: &'a [CardId], tries: i64 },
    /// put the hand back and mulligan from draw `from_try` on, by the rule
    Mull { from_try: i64 },
}

/// A goldfish game of `seats` (the deck first) set up for an arm, before turn one. Every random stream is named by
/// `name`: play decisions use `play:{name}`, each seat's shuffle `lib:{name}:{key}`, so both arms of one playout share
/// their play stream. The inert seats keep their seven, as `setup_game` has them.
pub fn setup_playout(
    db: &Arc<CardDb>,
    registry: &Arc<crate::hooks::Registry>,
    settings: &Arc<Settings>,
    seats: &[Seat],
    name: &str,
    diag_seed: u64,
    arm: &Arm,
) -> Game {
    crate::ai::evdiag::set_seed(diag_seed);
    crate::ai::learned::game_start();
    let mut g = new_game(db, registry, settings, seats, &format!("play:{name}"));
    for i in 0..g.players.len() {
        let p = PlayerId(i as u8);
        let mut r = Rng::named(&format!("lib:{name}:{}", g.player(p).key));
        let mut lib = std::mem::take(&mut g.player_mut(p).library);
        if crate::goldfish::is_inert(&g, p) {
            r.shuffle(&mut lib);
            let pl = g.player_mut(p);
            pl.library = lib;
            for _ in 0..7 {
                if let Some(c) = pl.library.pop() {
                    pl.hand.push(c);
                }
            }
            continue;
        }
        match arm {
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
        crate::impls::card_models::caverns_opening(&mut g, p);
        let hand = g.player(p).hand.clone();
        g.player_mut(p).seen.extend(hand);
    }
    crate::gf_audit::start(&mut g);
    g
}

/// the deck's score for a finished playout: its own kill turn, or `NO_KILL`
pub fn score(g: &Game) -> u32 {
    let won = g.winner == Some(P0) && g.wintype.is_none_or(|w| &*w != "timeout");
    if won { g.player(P0).turns.min(NO_KILL) } else { NO_KILL }
}

// ------------------------------------------------------------------ the playouts and their reading, shared with practice
/// one goldfish playout of an arm to turn `ROUNDS`: the deck's score (its kill turn, `NO_KILL` for none)
pub fn play_arm(
    db: &Arc<CardDb>,
    registry: &Arc<crate::hooks::Registry>,
    settings: &Arc<Settings>,
    seats: &[Seat],
    name: &str,
    diag_seed: u64,
    arm: &Arm,
) -> u32 {
    let mut g = setup_playout(db, registry, settings, seats, name, diag_seed, arm);
    crate::engine::turn::run_rounds(&mut g, ROUNDS);
    score(&g)
}

/// The paired playouts `j` in `js` of one dealt hand: for each, the KEEP score and the MULLIGAN score from the same
/// streams (`mo:{seed}:{tries}:{j}`). `threads` > 1 plays the playouts on that many threads (the result is the same in
/// `js` order). None: a playout panicked.
#[allow(clippy::too_many_arguments)]
pub fn paired_scores(
    db: &Arc<CardDb>,
    registry: &Arc<crate::hooks::Registry>,
    settings: &Arc<Settings>,
    seats: &[Seat],
    hand: &[CardId],
    tries: i64,
    seed: u64,
    js: std::ops::Range<u64>,
    threads: usize,
) -> Option<(Vec<u32>, Vec<u32>)> {
    let one = |j: u64| {
        let name = format!("mo:{seed}:{tries}:{j}");
        let diag = seed.wrapping_mul(1_000_000).wrapping_add(tries as u64 * 1000 + j);
        let k = play_arm(db, registry, settings, seats, &name, diag, &Arm::Keep { hand, tries });
        let m = play_arm(db, registry, settings, seats, &name, diag, &Arm::Mull { from_try: tries + 1 });
        (k, m)
    };
    let all: Vec<u64> = js.collect();
    let threads = threads.clamp(1, all.len().max(1));
    if threads == 1 {
        let r: Vec<(u32, u32)> = all.iter().map(|&j| one(j)).collect();
        return Some(r.into_iter().unzip());
    }
    let chunks: Vec<&[u64]> = all.chunks(all.len().div_ceil(threads)).collect();
    let parts: Vec<Option<Vec<(u32, u32)>>> = std::thread::scope(|sc| {
        let hs: Vec<_> = chunks
            .iter()
            .map(|c| {
                let one = &one;
                std::thread::Builder::new()
                    .stack_size(256 << 20)
                    .spawn_scoped(sc, move || c.iter().map(|&j| one(j)).collect::<Vec<_>>())
                    .expect("spawn a playout thread")
            })
            .collect();
        hs.into_iter().map(|h| h.join().ok()).collect()
    });
    let mut out = vec![];
    for p in parts {
        out.extend(p?);
    }
    Some(out.into_iter().unzip())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Keep,
    Mull,
    Unclear,
}

/// the oracle's reading of two paired lists of scores: (keep mean, mull mean, mean difference mull - keep, its
/// standard error, verdict). Fewer than two playouts give no standard error: unclear.
pub fn judge(keep: &[u32], mull: &[u32]) -> (f64, f64, f64, f64, Verdict) {
    let m = keep.len().min(mull.len());
    let mean = |v: &[u32]| v.iter().map(|&x| x as f64).sum::<f64>() / v.len().max(1) as f64;
    let (km, mm) = (mean(&keep[..m]), mean(&mull[..m]));
    let d: Vec<f64> = (0..m).map(|j| mull[j] as f64 - keep[j] as f64).collect();
    let dm = d.iter().sum::<f64>() / m.max(1) as f64;
    let se = if m < 2 {
        f64::INFINITY
    } else {
        let var = d.iter().map(|x| (x - dm).powi(2)).sum::<f64>() / (m - 1) as f64;
        (var / m as f64).sqrt()
    };
    let verdict = if dm < -2.0 * se {
        Verdict::Mull
    } else if dm > 2.0 * se {
        Verdict::Keep
    } else {
        Verdict::Unclear
    };
    (km, mm, dm, se, verdict)
}
