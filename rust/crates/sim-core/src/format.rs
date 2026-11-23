//! The rules a format changes. `Settings.rules` carries them; the engine and the cards read them (a player's
//! `start_life`, the starting player's first draw), so a format is data, not a branch on a name.

/// the format-dependent rules a game is played under
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rules {
    /// every player's starting life total (Commander 40)
    pub life: i32,
    /// whether the starting player (seat 0) draws on its first turn. False skips that draw step (CR 103.8a).
    pub first_player_draws: bool,
}

impl Rules {
    /// Commander as this simulator has always played it. `first_player_draws: true` is deliberately the old 4-seat
    /// rule even for a 2-seat Commander game (CR 103.8a would skip it): do not "fix" it, every Commander trace and
    /// number depends on it.
    pub const COMMANDER: Rules = Rules { life: 40, first_player_draws: true };
}

impl Default for Rules {
    fn default() -> Rules {
        Rules::COMMANDER
    }
}
