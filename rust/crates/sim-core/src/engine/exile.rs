//! The exile zone (rule 406) as objects: each card in a player's exile is an object with its own id and links (what
//! exiled it and until when, whether it is face down). A card that leaves exile and is exiled again is a new object
//! with a new id (rule 400.7), so an effect that remembers "the card exiled with ~" remembers the object, not the
//! card name: a card with the same `CardId` exiled later by something else isn't it.
//!
//! The zone reads like the `Vec<CardId>` it replaced (`Deref<Target = [CardId]>`: `contains`, `iter`, `len`,
//! indexing), so code that only looks at which cards are in exile is unchanged. Every write goes through `push`,
//! `extend`, `remove`, `take` ... which keep the objects in step with the cards. An empty zone is two empty `Vec`s:
//! copying a game with empty exiles allocates nothing.
//!
//! Design: `audit/phase3/exile-design.md`.

use crate::ids::{CardId, PermId};
use std::ops::Deref;

/// An exiled object's id, unique within its owner's exile zone for the whole game (never reused). A card's
/// (owner, `ExileId`) names it across the game.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExileId(pub u32);

/// One card in exile: its id and its links.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExileObj {
    pub id: ExileId,
    /// exiled "until ~ leaves the battlefield" by this permanent (the O-Ring family, `common::oring_exile`)
    pub until: Option<PermId>,
}

/// A player's exile: the cards, oldest first, and their objects (same order, same length).
#[derive(Debug, Clone, Default)]
pub struct ExileZone {
    cards: Vec<CardId>,
    objs: Vec<ExileObj>,
    next: u32,
}

impl Deref for ExileZone {
    type Target = [CardId];
    fn deref(&self) -> &[CardId] {
        &self.cards
    }
}

impl PartialEq for ExileZone {
    /// two exiles are equal when they hold the same cards with the same links (ids are a numbering, not state)
    fn eq(&self, o: &Self) -> bool {
        self.cards == o.cards && self.objs.iter().zip(&o.objs).all(|(a, b)| a.until == b.until)
    }
}

impl PartialEq<Vec<CardId>> for ExileZone {
    fn eq(&self, o: &Vec<CardId>) -> bool {
        self.cards == *o
    }
}

impl PartialEq<[CardId]> for ExileZone {
    fn eq(&self, o: &[CardId]) -> bool {
        self.cards == o
    }
}

impl<const N: usize> PartialEq<[CardId; N]> for ExileZone {
    fn eq(&self, o: &[CardId; N]) -> bool {
        self.cards == o
    }
}

impl Extend<CardId> for ExileZone {
    fn extend<I: IntoIterator<Item = CardId>>(&mut self, it: I) {
        for c in it {
            self.push(c);
        }
    }
}

impl<'a> Extend<&'a CardId> for ExileZone {
    fn extend<I: IntoIterator<Item = &'a CardId>>(&mut self, it: I) {
        for &c in it {
            self.push(c);
        }
    }
}

impl ExileZone {
    fn fresh(&mut self) -> ExileId {
        let id = ExileId(self.next);
        self.next += 1;
        id
    }

    /// exile a card: a new object (rule 400.7) with no links
    pub fn push(&mut self, c: CardId) {
        self.add(c, None);
    }

    /// exile a card until `src` leaves the battlefield
    pub fn add(&mut self, c: CardId, until: Option<PermId>) -> ExileId {
        let id = self.fresh();
        self.cards.push(c);
        self.objs.push(ExileObj { id, until });
        id
    }

    /// the card at `i` leaves exile (its object is gone)
    pub fn remove(&mut self, i: usize) -> CardId {
        self.objs.remove(i);
        self.cards.remove(i)
    }

    /// the newest card leaves exile
    pub fn pop(&mut self) -> Option<CardId> {
        self.objs.pop();
        self.cards.pop()
    }

    /// the oldest object of card `c` leaves exile; false if `c` isn't in exile
    pub fn take(&mut self, c: CardId) -> bool {
        match self.cards.iter().position(|&x| x == c) {
            Some(i) => {
                self.remove(i);
                true
            }
            None => false,
        }
    }

    /// keep the cards `f` is true of
    pub fn retain(&mut self, mut f: impl FnMut(&CardId) -> bool) {
        let mut i = 0;
        while i < self.cards.len() {
            if f(&self.cards[i]) {
                i += 1;
            } else {
                self.remove(i);
            }
        }
    }

    /// every card leaves exile
    pub fn clear(&mut self) {
        self.cards.clear();
        self.objs.clear();
    }

    /// the cards, oldest first
    pub fn cards(&self) -> &[CardId] {
        &self.cards
    }

    /// the objects, oldest first (the same order as `cards`)
    pub fn objs(&self) -> &[ExileObj] {
        &self.objs
    }

    /// the cards with their objects, oldest first
    pub fn entries(&self) -> impl Iterator<Item = (CardId, &ExileObj)> {
        self.cards.iter().copied().zip(&self.objs)
    }

    /// where the object `id` is, if it is still in exile
    pub fn find(&self, id: ExileId) -> Option<usize> {
        self.objs.iter().position(|o| o.id == id)
    }

    /// the object at `i`
    pub fn obj(&self, i: usize) -> &ExileObj {
        &self.objs[i]
    }

    /// the object at `i`, to change its links
    pub fn obj_mut(&mut self, i: usize) -> &mut ExileObj {
        &mut self.objs[i]
    }

    /// the newest object of card `c`, if it is in exile
    pub fn newest(&self, c: CardId) -> Option<ExileId> {
        self.cards.iter().rposition(|&x| x == c).map(|i| self.objs[i].id)
    }
}

impl From<Vec<CardId>> for ExileZone {
    fn from(v: Vec<CardId>) -> Self {
        let mut z = ExileZone::default();
        z.extend(v);
        z
    }
}

impl<'a> IntoIterator for &'a ExileZone {
    type Item = &'a CardId;
    type IntoIter = std::slice::Iter<'a, CardId>;
    fn into_iter(self) -> Self::IntoIter {
        self.cards.iter()
    }
}
