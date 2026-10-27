//! The card tags that *execute* an effect, by the engine code that runs them (`cardgen audit --paths`). Every other
//! tag only informs the AI (what a card is for, how much it is worth) or is a number another rule reads (`pow`).
//!
//! The lists name the tags each executor tests on the card itself; tests below scan the executors' source so a tag
//! the code starts to act on can't be left out. A tag can run in more than one executor (`draw` on a spell resolves,
//! on a permanent it is an enters trigger). This module only describes: the engine does not read it.

use crate::cards::CardDef;
use crate::engine::zones::{ETB_SCRYFALL_TAGS, ETB_TRIGGER_TAGS, LOCK_KINDS};
use crate::tag::Tag;

/// The engine code that runs a tag's effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Exec {
    /// `engine/zones.rs` `do_etb` -> `etb_once`: an enters trigger on the stack (a permanent with no compiled ability)
    EnterTrigger,
    /// `engine/zones.rs` `do_etb` -> `etb_static`: as it enters (a permanent with no compiled ability)
    EnterStatic,
    /// `engine/cast.rs` `resolve_inner`: an instant or sorcery resolves (no compiled ability, no registry `resolve`)
    SpellResolve,
    /// `engine/cast.rs` `resolve_inner`: a permanent spell resolves (always)
    PermResolve,
    /// `engine/stack.rs` `counterable` and the counter check: can't be countered (always)
    Cast,
    /// `engine/mana.rs` mana units: what a permanent or land taps for, and what tapping it costs (always)
    Mana,
    /// `engine/mana.rs` `cost_of`: the card's own cost reduction (always)
    SelfCost,
}

impl Exec {
    pub fn name(self) -> &'static str {
        match self {
            Exec::EnterTrigger => "enter",
            Exec::EnterStatic => "enter-static",
            Exec::SpellResolve => "spell",
            Exec::PermResolve => "perm-resolve",
            Exec::Cast => "cast",
            Exec::Mana => "mana",
            Exec::SelfCost => "self-cost",
        }
    }
}

/// `etb_once` tags beyond `ETB_TRIGGER_TAGS` / `ETB_SCRYFALL_TAGS`: `draw` (a permanent's), `rem` with `etb` (not a
/// lock), `fill=stitcher|wayfinder`
pub const ENTER_EXTRA_TAGS: [Tag; 3] = [Tag::Draw, Tag::Rem, Tag::Fill];
/// `etb_static` (and `rem` with `etb` that is a lock Aura)
pub const ENTER_STATIC_TAGS: [Tag; 6] = [Tag::Flute, Tag::Prepare, Tag::Mycoloth, Tag::Endraze, Tag::Spider, Tag::Rem];
/// `resolve_inner`, an instant or sorcery
pub const SPELL_RESOLVE_TAGS: [Tag; 38] = [
    Tag::Draw,
    Tag::Treas,
    Tag::Lr,
    Tag::Lh,
    Tag::Ringtempt,
    Tag::Tut,
    Tag::Gifts,
    Tag::Intuition,
    Tag::Jeska,
    Tag::Explore,
    Tag::Loam,
    Tag::Rishkar,
    Tag::Threedreams,
    Tag::Seal,
    Tag::Adnaus,
    Tag::Lose,
    Tag::Selfdmg,
    Tag::Discard1,
    Tag::Rem,
    Tag::Wipe,
    Tag::Fill,
    Tag::Rean,
    Tag::Burn,
    Tag::Tokx,
    Tag::Clue,
    Tag::Mktok,
    Tag::Drainetb,
    Tag::Edictetb,
    Tag::Pumpall,
    Tag::Crackle,
    Tag::Drawcre,
    Tag::Stampede,
    Tag::Prolif1,
    Tag::Unearth,
    Tag::Fbgrant,
    Tag::Fbnib,
    Tag::Yawg,
    Tag::Mastery,
];
/// `resolve_inner`, a permanent spell
pub const PERM_RESOLVE_TAGS: [Tag; 2] = [Tag::Moxd, Tag::Lr];
/// `engine/stack.rs`: the spell can't be countered
pub const CAST_TAGS: [Tag; 1] = [Tag::Unc];
/// `engine/mana.rs`: mana sources and what tapping them costs
pub const MANA_TAGS: [Tag; 9] = [
    Tag::Rock,
    Tag::Dork,
    Tag::Chromemox,
    Tag::Tomb,
    Tag::Pain,
    Tag::Workshop,
    Tag::Rite,
    Tag::Lantern,
    Tag::Worldtree,
];
/// `engine/mana.rs` `cost_of`: the card's own cost
pub const SELF_COST_TAGS: [Tag; 4] = [Tag::PerCreature, Tag::Eris, Tag::Dawning, Tag::Spectacle];

/// Tags the executors read only as a detail of another tag's effect (`tokfly` on `tok`'s tokens, `lrt` on `lr`'s
/// lands, `amt` on a land's mana): not executors themselves.
pub const EXEC_MODIFIERS: [Tag; 17] = [
    Tag::Etb,
    Tag::Tokdt,
    Tag::Tokfly,
    Tag::Tokp,
    Tag::F,
    Tag::Veyran,
    Tag::Lrt,
    Tag::Top,
    Tag::Fbdraw,
    Tag::Warrior,
    Tag::Toklife,
    Tag::Amt,
    Tag::Pstone,
    Tag::C,
    Tag::Tgt,
    Tag::Face,
    Tag::Mv4,
];

/// Every (tag, executor) pair: the tag-to-executor table.
pub fn table() -> Vec<(Tag, Exec)> {
    let mut v = vec![];
    let mut add = |tags: &[Tag], e: Exec| v.extend(tags.iter().map(|&t| (t, e)));
    add(&ETB_TRIGGER_TAGS, Exec::EnterTrigger);
    add(&ETB_SCRYFALL_TAGS, Exec::EnterTrigger);
    add(&ENTER_EXTRA_TAGS, Exec::EnterTrigger);
    add(&ENTER_STATIC_TAGS, Exec::EnterStatic);
    add(&SPELL_RESOLVE_TAGS, Exec::SpellResolve);
    add(&PERM_RESOLVE_TAGS, Exec::PermResolve);
    add(&CAST_TAGS, Exec::Cast);
    add(&MANA_TAGS, Exec::Mana);
    add(&SELF_COST_TAGS, Exec::SelfCost);
    v.sort();
    v.dedup();
    v
}

/// does any executor run this tag?
pub fn executes(t: Tag) -> bool {
    table().iter().any(|&(x, _)| x == t)
}

/// The executing tags that actually run for this card, with their executor: the enter executors skip a card with
/// compiled abilities (`do_etb`), the spell executor skips one with abilities or a registry `resolve` (`resolve_hook`).
pub fn live(d: &CardDef, resolve_hook: bool) -> Vec<(Tag, Exec)> {
    let t = &d.tags;
    let spell = d.instant || d.sorcery;
    let mut v = vec![];
    if !spell && !d.land && !d.has_dsl() {
        let rem_etb = t.has(Tag::Rem) && t.has(Tag::Etb);
        let lock = t.str(Tag::Rem).is_some_and(|r| LOCK_KINDS.contains(&r));
        for &k in ETB_TRIGGER_TAGS.iter() {
            if t.has(k) {
                v.push((k, Exec::EnterTrigger));
            }
        }
        if matches!(t.str(Tag::Fill), Some("stitcher" | "wayfinder")) {
            v.push((Tag::Fill, Exec::EnterTrigger));
        }
        if t.has(Tag::Draw) {
            v.push((Tag::Draw, Exec::EnterTrigger));
        }
        if rem_etb && !lock {
            v.push((Tag::Rem, Exec::EnterTrigger));
        }
        if d.source_scryfall() {
            for &k in ETB_SCRYFALL_TAGS.iter() {
                if t.has(k) {
                    v.push((k, Exec::EnterTrigger));
                }
            }
        }
        for &k in &ENTER_STATIC_TAGS[..5] {
            if t.has(k) {
                v.push((k, Exec::EnterStatic));
            }
        }
        if rem_etb && lock {
            v.push((Tag::Rem, Exec::EnterStatic));
        }
    }
    if spell && !d.has_dsl() && !resolve_hook {
        v.extend(SPELL_RESOLVE_TAGS.iter().filter(|&&k| t.has(k)).map(|&k| (k, Exec::SpellResolve)));
    }
    if d.perm {
        v.extend(PERM_RESOLVE_TAGS.iter().filter(|&&k| t.has(k)).map(|&k| (k, Exec::PermResolve)));
    }
    v.extend(CAST_TAGS.iter().filter(|&&k| t.has(k)).map(|&k| (k, Exec::Cast)));
    v.extend(MANA_TAGS.iter().filter(|&&k| t.has(k)).map(|&k| (k, Exec::Mana)));
    v.extend(SELF_COST_TAGS.iter().filter(|&&k| t.has(k)).map(|&k| (k, Exec::SelfCost)));
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    /// the `Tag::X` names a function's body reads off the card's own tags (`t.has(Tag::X)`, `t.int(..)`, `t.str(..)`)
    fn own_tags(src: &str, start: &str) -> Vec<String> {
        let i = src.find(start).unwrap_or_else(|| panic!("{start:?} not found"));
        let body = &src[i..];
        let end = body.find("\n}\n").map_or(body.len(), |e| e + 3);
        let body = &body[..end];
        let mut out = vec![];
        for pat in ["t.has(Tag::", "t.int(Tag::", "t.str(Tag::"] {
            for (j, _) in body.match_indices(pat) {
                let rest = &body[j + pat.len()..];
                let name: String = rest.chars().take_while(|c| c.is_alphanumeric()).collect();
                out.push(name);
            }
        }
        out.sort();
        out.dedup();
        out
    }

    fn known(name: &str) -> bool {
        let listed = |t: Tag| format!("{t:?}") == name;
        table().iter().any(|&(t, _)| listed(t)) || EXEC_MODIFIERS.iter().any(|&t| listed(t))
    }

    #[test]
    fn every_tag_an_executor_reads_is_listed() {
        let zones = include_str!("engine/zones.rs");
        let cast = include_str!("engine/cast.rs");
        let mana = include_str!("engine/mana.rs");
        let mut missing = vec![];
        for (src, f) in [
            (zones, "pub fn etb_once("),
            (zones, "pub fn etb_static("),
            (zones, "pub fn etb_triggers("),
            (cast, "fn resolve_inner("),
            (mana, "pub fn cost_of("),
        ] {
            for n in own_tags(src, f) {
                if !known(&n) {
                    missing.push(format!("{f} Tag::{n}"));
                }
            }
        }
        for (j, _) in mana.match_indices("t.has(Tag::").chain(mana.match_indices("t.str(Tag::")) {
            let n: String = mana[j + 11..].chars().take_while(|c| c.is_alphanumeric()).collect();
            if !known(&n) {
                missing.push(format!("mana.rs Tag::{n}"));
            }
        }
        missing.sort();
        missing.dedup();
        assert!(missing.is_empty(), "tags the executors read but exec_tags.rs doesn't list: {missing:?}");
    }

    #[test]
    fn executors_and_modifiers_are_apart() {
        for t in EXEC_MODIFIERS {
            assert!(!executes(t), "{t:?} is both an executor and a modifier");
        }
        assert!(executes(Tag::Unc) && executes(Tag::Titan) && !executes(Tag::Bomb));
    }
}
