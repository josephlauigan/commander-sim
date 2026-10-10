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

pub mod cast;
pub mod hooks;
pub mod life;
pub mod mana;
pub mod removal;
pub mod stack;
pub mod tutors;
pub mod values;
pub mod zones;

/// A play-by-play line, when the game is traced (`--trace`). The arguments are only formatted then.
#[macro_export]
macro_rules! glog {
    ($g:expr, $($arg:tt)*) => {
        if $g.log.is_some() {
            let line = format!("R{:<2} {}", $g.round, format!($($arg)*));
            if let Some(log) = $g.log.as_mut() {
                log.push(line);
            }
        }
    };
}
