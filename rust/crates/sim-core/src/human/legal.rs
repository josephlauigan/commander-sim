//! What the person may do by the rules (Python's `play/legal.py`): legality checks with Python's wording, the
//! legal targets of a spell, and how a target is described to the person.

use super::Target;
use crate::engine::values::{epow, etgh};
use crate::ids::PlayerId;
use crate::state::{Game, Perm};
use std::sync::OnceLock;

/// Teferi's Protection and The One Ring give protection from everything.
pub fn player_targetable(g: &Game, q: PlayerId) -> bool {
    let q = g.player(q);
    q.alive && !q.life_locked && !q.ring_prot
}

type Guesser = fn(&Game, &Perm) -> Option<String>;
static GUESSER: OnceLock<Guesser> = OnceLock::new();

/// The practice crate registers Python's `view.guess_token` (it needs the token catalog of `images.py`).
pub fn set_token_guesser(f: Guesser) {
    let _ = GUESSER.set(f);
}

/// A token's name as players say it: 'Orc Army', 'Goblin token', 'Treasure' (Python's `view.token_name`).
pub fn token_name(g: &Game, m: &Perm) -> String {
    if m.army {
        return "Orc Army".into();
    }
    if !m.name.is_empty() && m.name != "Token" {
        return m.name.to_string();
    }
    let mut kinds: Vec<&str> = m.ttypes.iter().copied().filter(|t| !t.is_empty()).collect();
    kinds.sort_unstable();
    if !kinds.is_empty() {
        let words: Vec<String> = kinds.iter().map(|k| title(k)).collect();
        return format!("{} token", words.join(" "));
    }
    match GUESSER.get().and_then(|f| f(g, m)) {
        Some(guess) => format!("{guess} token"),
        None => "Token".into(),
    }
}

/// Python's `str.title()` for a lower-case word: first letter up, the rest down, after every non-letter.
pub fn title(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut up = true;
    for ch in s.chars() {
        if ch.is_alphabetic() {
            if up {
                out.extend(ch.to_uppercase());
            } else {
                out.extend(ch.to_lowercase());
            }
            up = false;
        } else {
            out.push(ch);
            up = true;
        }
    }
    out
}

/// 'Sheoldred, the Apocalypse (Prosper, tapped)' / 'Krenko (player, 31 life)'.
pub fn describe_target(g: &Game, p: PlayerId, x: Target) -> String {
    match x {
        Target::Player(q) => {
            let pl = g.player(q);
            format!("{} (player, {} life){}", pl.name, pl.life, if q == p { " (you)" } else { "" })
        }
        Target::Perm(id) => {
            let m = g.perm(id);
            let name = if m.token { token_name(g, m) } else { m.name.to_string() };
            let mut bits = vec![if m.owner == p { "you".to_string() } else { g.player(m.owner).name.to_string() }];
            if g.is_creature(id) {
                bits.push(format!("{}/{}", epow(g, id), etgh(g, id)));
            }
            if m.tapped {
                bits.push("tapped".into());
            }
            format!("{name} ({})", bits.join(", "))
        }
    }
}

/// What spell c may target, by the rules (not by what the AI would pick): permanents and players, or None if c
/// doesn't target. PORT(core): milestone A stub.
pub fn spell_targets(_g: &Game, _p: PlayerId, _c: crate::ids::CardId) -> Option<Vec<Target>> {
    unimplemented!("PORT(core): legal::spell_targets")
}
