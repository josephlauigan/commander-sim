//! Interned strings: token names, subtypes, deck keys, stat names. An interned string is a `&'static str`, so game
//! state that holds one copies a pointer when the look-ahead copies a game, instead of allocating. The set of such
//! strings is small and fixed in practice (names come from card code and the decks), so leaking them is fine.

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

pub type Sym = &'static str;

pub fn intern(s: &str) -> Sym {
    static SET: OnceLock<Mutex<HashSet<Sym>>> = OnceLock::new();
    let mut set = SET.get_or_init(|| Mutex::new(HashSet::new())).lock().unwrap();
    if let Some(&x) = set.get(s) {
        return x;
    }
    let x: Sym = Box::leak(s.to_owned().into_boxed_str());
    set.insert(x);
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_string_is_the_same_pointer() {
        let a = intern(&String::from("Zombie"));
        let b = intern("Zombie");
        assert!(std::ptr::eq(a, b));
        assert_ne!(intern("Goblin"), a);
    }
}
