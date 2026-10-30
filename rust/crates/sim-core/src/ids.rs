//! Handles. Nothing in a game points at anything else directly: cards, players and permanents are named by these
//! small numbers, so a whole `Game` is plain data and copying it (the look-ahead) is copying a few arrays.

/// A card definition in the `CardDb`. Like Python's shared `CD` objects: every copy of a card in every zone and every
/// game is the same `CardId` (two Swamps in a library are the same id twice).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CardId(pub u16);

/// A seat, by its index in turn order (`Game::players`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlayerId(pub u8);

/// A permanent or token on (or once on) the battlefield, land or not (Python's `Perm` and `Land`), by its index in
/// `Game::perms`. Ids are never reused within a game, as Python keeps every permanent alive until the game ends
/// (`g.alive_objs`) so that tables keyed by a permanent can't be inherited by a new one. The index also plays the
/// part of Python's creation number (`hid`). Lands and nonland permanents share the numbering (Phase 3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PermId(pub u32);

/// Phase 3 (one arena), during the move: a land is a `Perm`, named by a `PermId`.
pub type LandId = PermId;

/// Phase 3, during the move: `LandId(n)` as it was written before lands joined the arena.
#[allow(non_snake_case)]
pub fn LandId(n: u32) -> PermId {
    PermId(n)
}

impl PlayerId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl PermId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl CardId {
    pub fn index(self) -> usize {
        self.0 as usize
    }
}
