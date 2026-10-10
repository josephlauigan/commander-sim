//! Interned strings: token names, subtypes, deck keys, stat names. An interned string is a `&'static str`, so game
//! state that holds one copies a pointer when the look-ahead copies a game, instead of allocating. The set of such
//! strings is small and fixed in practice (names come from card code and the decks), so leaking them is fine.

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

pub type Sym = &'static str;

pub fn intern(s: &str) -> Sym {
    // each thread keeps its own cache of the strings it has seen, so games played on many threads at once (the
    // commander-sim program) don't all queue on the global lock; the global set still decides the one pointer per
    // string
    thread_local! {
        static LOCAL: std::cell::RefCell<HashSet<Sym>> = std::cell::RefCell::new(HashSet::new());
    }
    if let Some(x) = LOCAL.with(|l| l.borrow().get(s).copied()) {
        return x;
    }
    static SET: OnceLock<Mutex<HashSet<Sym>>> = OnceLock::new();
    let x = {
        let mut set = SET.get_or_init(|| Mutex::new(HashSet::new())).lock().unwrap();
        match set.get(s) {
            Some(&x) => x,
            None => {
                let x: Sym = Box::leak(s.to_owned().into_boxed_str());
                set.insert(x);
                x
            }
        }
    };
    LOCAL.with(|l| l.borrow_mut().insert(x));
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
