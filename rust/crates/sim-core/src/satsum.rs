//! Saturating sums for i32 power, damage and life totals. Release builds check overflow, and a Thrakkus-style doubling
//! can leave a creature's power near `i32::MAX`, so a plain `.sum()` of powers (or a `* fiery`) panics and the game is
//! dropped as a simulator error. Past `i32::MAX` the number only has to stay lethal, so the sum sticks there.

pub trait SatSum {
    /// the sum of the items, stopping at `i32::MAX` / `i32::MIN` instead of overflowing
    fn ssum(self) -> i32;
}

impl<I: Iterator<Item = i32>> SatSum for I {
    fn ssum(self) -> i32 {
        self.fold(0i32, |a, b| a.saturating_add(b))
    }
}
