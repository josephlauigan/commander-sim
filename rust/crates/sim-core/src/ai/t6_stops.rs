//! The switch `AI_T6STOPS` (Tier 6 fix 2, 2026-11-02): fixes to Tier 6 combo attempts that stopped without any card
//! stopping them (`audit/t6-stops/trace-2026-11-02.md`). Unset, empty or anything but `0`: on (the default).
//! `AI_T6STOPS=0`: today's play, byte for byte. Read like `AI_SEARCH_B` / `AI_RULES`.
//!
//! What it changes (each only for a Tier 6 player going for a combo):
//! - a line whose cast pieces cannot be cast now (a lock, a second spell under Rule of Law) is not "ready";
//! - "Ad Nauseam into a win" is ready only when Ad Nauseam is likely to find the pieces it lacks;
//! (A counter war that spends the mana of the line's later steps is left: see the trace write-up.)
//!
//! The stop labels name the step or cast that failed instead of "?"; they change no play, but read this too, so that
//! `AI_T6STOPS=0` reports stay byte-identical.

use std::cell::Cell;
use std::sync::OnceLock;

thread_local! {
    static FORCE: Cell<Option<bool>> = const { Cell::new(None) };
}

/// for tests: the fixes on or off on this thread, whatever `AI_T6STOPS` says (None: back to it)
pub fn force(on: Option<bool>) {
    FORCE.with(|f| f.set(on));
}

/// `AI_T6STOPS` read as a switch: `0` off, anything else on
pub fn parse(v: Option<&str>) -> bool {
    !matches!(v.map(str::trim), Some("0"))
}

/// are the fixes on?
pub fn on() -> bool {
    if let Some(f) = FORCE.with(|f| f.get()) {
        return f;
    }
    static M: OnceLock<bool> = OnceLock::new();
    *M.get_or_init(|| parse(std::env::var("AI_T6STOPS").ok().as_deref()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_switch_reads_like_the_others() {
        assert!(parse(None));
        assert!(parse(Some("")));
        assert!(parse(Some("1")));
        assert!(!parse(Some("0")));
        assert!(!parse(Some(" 0 ")));
    }
}
