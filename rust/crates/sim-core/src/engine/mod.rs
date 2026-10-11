//! The rules (Python's `engine.py`), one file per section of the Python:
//!
//! | file | engine.py section |
//! |---|---|
//! | values.rs | helpers, values / threat |
//! | life.rs | lose_life ... eliminate |
//! | mana.rs | mana, costs |
//! | zones.rs | zones: draw, tokens, enter, leave, die, lands |
//! | cast.rs | casting, on_cast, resolve |
//! | stack.rs | triggered abilities, the stack, priority, counterspells |
//! | removal.rs | removal, wipes |
//! | tutors.rs | tutors and the Game Changer mechanics |
//! | hooks.rs | calling card code (cardimpl.py's dispatch) |
//! | combat.rs | ais.py: combat |
//! | turn.rs | ais.py: lands, upkeep, end step, the turn, mulligans, a game |

pub mod cast;
pub mod combat;
pub mod hooks;
pub mod life;
pub mod mana;
pub mod removal;
pub mod stack;
pub mod turn;
pub mod tutors;
pub mod values;
pub mod zones;

/// a line naming a card only p may see (one tutored to hand or to the top of the library): the log keeps `msg`; a
/// practice table shows the other players `public` (recorded in `g.secret_lines` for the session's event log).
/// Python's `log_secret`.
pub fn log_secret(g: &mut crate::state::Game, p: crate::ids::PlayerId, public: &str, msg: &str) {
    if g.log.is_none() {
        return;
    }
    let round = g.round;
    let at = g.log.as_ref().map_or(0, Vec::len);
    if g.humans.any() {
        g.secret_lines.push((at, p, format!("R{round:<2} {public}")));
    }
    if let Some(log) = g.log.as_mut() {
        log.push(format!("R{round:<2} {msg}"));
    }
}

/// A play-by-play line, when the game is traced (`--trace`). The arguments are only formatted then.
#[macro_export]
macro_rules! glog {
    ($g:expr, $($arg:tt)*) => {
        if $g.log.is_some() || $g.link.sink.is_some() {
            let line = format!("R{:<2} {}", $g.round, format!($($arg)*));
            if let Some(sink) = $g.link.sink.clone() {
                sink.line(&$g, &line, None);
            }
            if let Some(log) = $g.log.as_mut() {
                log.push(line);
            }
        }
    };
}
