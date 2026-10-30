//! The mana pool (rule 106.4): the mana a player has made and not spent yet, as mana objects. Each object is an
//! amount of one kind of mana (a colour, colourless, or "any colour" for the mana the engine lets you choose as you
//! spend it), with an optional restriction on what it may pay for (106.6) and how long it stays (rule 500.4: the pool
//! empties as each step and phase ends, unless an effect keeps the mana).
//!
//! Objects that differ only in amount are one entry: mana of the same kind, restriction and keep is the same mana. The
//! entries stay sorted by (restriction, kind, keep), so the payment planner sees the floating mana in a fixed order
//! (the order the old `Floating` fields were read in: red, any, blue, colourless, green, black, then restricted mana).
//! An empty pool is an empty `Vec`: copying a game with empty pools allocates nothing.
//!
//! Design: `audit/phase3/mana-design.md`.

use crate::cards::Colors;

/// What colour a mana object is. `Any` is mana of any of the player's colours, chosen as it is spent (the engine's
/// rituals and the ability language's "add one mana of any colour").
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Kind {
    R,
    Any,
    U,
    C,
    G,
    B,
    W,
}

impl Kind {
    /// the colours this mana may pay as, for a player whose colour identity is `ident`
    pub fn cols(self, ident: Colors) -> Colors {
        match self {
            Kind::Any => ident,
            Kind::C => Colors::NONE,
            k => Colors::from_letters(k.letter()),
        }
    }

    /// the mana symbol: "W" ... "C", "A" for any colour
    pub fn letter(self) -> &'static str {
        match self {
            Kind::W => "W",
            Kind::U => "U",
            Kind::B => "B",
            Kind::R => "R",
            Kind::G => "G",
            Kind::C => "C",
            Kind::Any => "A",
        }
    }

    /// the kind for a mana symbol ('A' or anything else: any colour)
    pub fn of_letter(c: char) -> Kind {
        match c {
            'W' => Kind::W,
            'U' => Kind::U,
            'B' => Kind::B,
            'R' => Kind::R,
            'G' => Kind::G,
            'C' => Kind::C,
            _ => Kind::Any,
        }
    }

    /// The kind that keeps what a source that can make `cols` made beyond what a payment used: one colour floats as
    /// that colour (white as any: the old pool had no white floating mana the planner read), none as {C}, several as
    /// any.
    pub fn of_leftover(cols: Colors) -> Kind {
        match cols.count() {
            0 => Kind::C,
            1 if cols.has('U') => Kind::U,
            1 if cols.has('B') => Kind::B,
            1 if cols.has('R') => Kind::R,
            1 if cols.has('G') => Kind::G,
            _ => Kind::Any,
        }
    }
}

/// What a mana object may be spent on (rule 106.6). `None`: anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Restr {
    #[default]
    None,
    /// can't be spent to cast nonartifact spells: artifact spells, abilities and other costs (Karn, Legacy Reforged;
    /// Powerstone tokens; The Mightstone and Weakstone)
    NotNonartifact,
}

/// How long a mana object stays in the pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Keep {
    /// until the step or phase ends (rule 500.4)
    #[default]
    Step,
    /// until the turn ends ("until end of turn, you don't lose this mana as steps and phases end": Karn, Legacy
    /// Reforged; Birgi; Savage Ventmaw ...)
    Turn,
}

/// Everything about a mana object except its amount: the entries of a pool are keyed by it, and a payment names the
/// entry it takes from (`mana::Source::Pool`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key {
    // field order is the sort order: restriction, kind, keep
    pub restr: Restr,
    pub kind: Kind,
    pub keep: Keep,
}

impl Key {
    /// unrestricted mana of `kind` that empties as the step ends
    pub const fn plain(kind: Kind) -> Key {
        Key { restr: Restr::None, kind, keep: Keep::Step }
    }

    /// Karn, Legacy Reforged's colourless: not for nonartifact spells, kept all turn
    pub const ART: Key = Key { restr: Restr::NotNonartifact, kind: Kind::C, keep: Keep::Turn };
}

/// One entry of a pool: `amt` mana of one key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Mana {
    pub key: Key,
    pub amt: u32,
}

/// A player's mana pool.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct Pool {
    v: Vec<Mana>,
}

impl Pool {
    /// add `n` mana of `key`
    pub fn add_key(&mut self, key: Key, n: u32) {
        if n == 0 {
            return;
        }
        match self.v.binary_search_by(|m| m.key.cmp(&key)) {
            Ok(i) => self.v[i].amt = self.v[i].amt.saturating_add(n),
            Err(i) => self.v.insert(i, Mana { key, amt: n }),
        }
    }

    /// add `n` unrestricted mana of `kind` (it empties as the step ends)
    pub fn add(&mut self, kind: Kind, n: u32) {
        self.add_key(Key::plain(kind), n);
    }

    /// add `n` of Karn, Legacy Reforged's colourless (`Key::ART`)
    pub fn add_art(&mut self, n: u32) {
        self.add_key(Key::ART, n);
    }

    /// how much mana of `key` there is
    pub fn amount(&self, key: Key) -> u32 {
        self.v.binary_search_by(|m| m.key.cmp(&key)).map_or(0, |i| self.v[i].amt)
    }

    /// how much unrestricted mana of `kind` there is, however long it stays
    pub fn get(&self, kind: Kind) -> u32 {
        self.v.iter().filter(|m| m.key.restr == Restr::None && m.key.kind == kind).map(|m| m.amt).sum()
    }

    /// the unrestricted mana but white (what the old pool's fields `u b r g c any` held)
    pub fn plain_but_white(&self) -> u32 {
        self.v.iter().filter(|m| m.key.restr == Restr::None && m.key.kind != Kind::W).map(|m| m.amt).sum()
    }

    /// Karn, Legacy Reforged's colourless in the pool
    pub fn art(&self) -> u32 {
        self.amount(Key::ART)
    }

    /// take up to `n` mana of `key` out of the pool (spent, or lost); what is left of it stays
    pub fn take(&mut self, key: Key, n: u32) {
        if let Ok(i) = self.v.binary_search_by(|m| m.key.cmp(&key)) {
            self.v[i].amt = self.v[i].amt.saturating_sub(n);
            if self.v[i].amt == 0 {
                self.v.remove(i);
            }
        }
    }

    /// take up to `n` unrestricted mana of `kind`, the mana that empties soonest first
    pub fn take_kind(&mut self, kind: Kind, mut n: u32) {
        for keep in [Keep::Step, Keep::Turn] {
            let key = Key { restr: Restr::None, kind, keep };
            let k = self.amount(key).min(n);
            self.take(key, k);
            n -= k;
        }
    }

    /// set the unrestricted mana of `kind` to `n` (a test, or a line that gives back what it borrowed)
    pub fn set(&mut self, kind: Kind, n: u32) {
        self.v.retain(|m| !(m.key.restr == Restr::None && m.key.kind == kind));
        self.add(kind, n);
    }

    /// set Karn's colourless to `n`
    pub fn set_art(&mut self, n: u32) {
        self.take(Key::ART, u32::MAX);
        self.add_art(n);
    }

    /// the pool empties (a new turn; Teferi's Protection)
    pub fn clear(&mut self) {
        self.v.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.v.is_empty()
    }

    /// all the mana in the pool
    pub fn total(&self) -> u32 {
        self.v.iter().map(|m| m.amt).sum()
    }

    /// the entries, in their fixed order
    pub fn iter(&self) -> impl Iterator<Item = &Mana> {
        self.v.iter()
    }
}
