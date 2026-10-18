//! A 0 from one of JD's own priority functions is deliberate for an instant and for removal, wipes, counters and
//! protection (held for responses): the generic ability value no longer replaces it (ai::brain::card_utility).
//! Prints, per deck, the cards this changes and the 0-priority cards that still get the ability value
//! (`--nocapture`).
use sim_core::ai::brain::{self, Situation};
use sim_core::tag::Tag;
use sim_core::ids::PlayerId;
use sim_core::testkit::*;

const JD: [&str; 10] =
    ["seph", "veyran", "sauron", "yshtola", "kefka", "titania", "sheoldred", "cloud", "kraum", "sephfs"];

#[test]
fn a_held_zero_from_a_jd_deck_is_never_replaced_by_the_generic_value() {
    for key in JD {
        let mut g = table(&[key, "sauron", "seph"]);
        let (mut held, mut kept) = (std::collections::BTreeSet::new(), std::collections::BTreeSet::new());
        for turn in [1u32, 8] {
            g.player_mut(PlayerId(0)).turns = turn;
            let s = Situation::new(&g, PlayerId(0));
            let names: Vec<_> = g.player(PlayerId(0)).deck_names.to_vec();
            for c in names {
                let d = g.db.get(c);
                if d.land || !d.has_dsl() || g.registry.get(c).is_some_and(|i| i.prio.is_some()) {
                    continue;
                }
                if sim_core::ai::decks::deck_prio_f(&g, PlayerId(0), c) > 0.0 {
                    continue;
                }
                let ritual = d.name.as_ref() == "Dark Ritual";
                let held_kind = !ritual && (d.instant || d.tag(Tag::Rem) || d.tag(Tag::Wipe) || d.tag(Tag::Ctr) || d.tag(Tag::Prot));
                if held_kind {
                    held.insert(d.name.to_string());
                    assert_eq!(brain::card_utility(&g, PlayerId(0), &s, c), None, "{key}: {}", d.name);
                } else {
                    kept.insert(d.name.to_string());
                }
            }
        }
        println!("HELD {key}: {}", held.into_iter().collect::<Vec<_>>().join("; "));
        println!("KEPT {key}: {}", kept.into_iter().collect::<Vec<_>>().join("; "));
    }
}

#[test]
fn the_answers_are_not_cast_proactively() {
    for (key, name) in [("sheoldred", "Masterful Flourish"), ("sheoldred", "Battle-Rage Blessing"), ("titania", "Blossoming Defense")] {
        let mut g = table(&[key, "sauron", "seph"]);
        let c = take(&mut g, PlayerId(0), name);
        g.player_mut(PlayerId(0)).hand.push(c);
        let s = Situation::new(&g, PlayerId(0));
        assert_eq!(brain::card_utility(&g, PlayerId(0), &s, c), None, "{key}: {name}");
    }
}
