//! The mana pool (Phase 3, PR 4: `engine/pool.rs`, `audit/phase3/mana-design.md`): mana objects with a restriction
//! and a keep, emptied as steps and phases end (rule 500.4); restricted sources' leftover mana keeps its restriction;
//! Cavern of Souls' "can't be countered". Seat 0 plays the karn deck, seat 1 the seph deck.

use sim_core::engine::mana::{can_pay, pay};
use sim_core::engine::pool::{Keep, Key, Kind, Pool, Restr};
use sim_core::ids::PlayerId;
use sim_core::state::{DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["karn", "seph"])
}

#[test]
fn a_pool_keeps_one_entry_per_kind_restriction_and_keep() {
    let mut p = Pool::default();
    p.add(Kind::G, 2);
    p.add(Kind::R, 1);
    p.add(Kind::G, 1);
    p.add_key(Key { restr: Restr::None, kind: Kind::R, keep: Keep::Turn }, 2);
    p.add_art(3);
    assert_eq!(p.iter().count(), 4, "{{G}} x3, {{R}}, kept {{R}} x2, Karn's");
    assert_eq!((p.get(Kind::G), p.get(Kind::R), p.art(), p.total()), (3, 3, 3, 9));
    // the planner reads red first, then green, then restricted mana
    let kinds: Vec<Kind> = p.iter().map(|m| m.key.kind).collect();
    assert_eq!(kinds, vec![Kind::R, Kind::R, Kind::G, Kind::C]);
    p.take_kind(Kind::R, 2); // the mana that empties soonest first
    assert_eq!(p.amount(Key::plain(Kind::R)), 0);
    assert_eq!(p.amount(Key { restr: Restr::None, kind: Kind::R, keep: Keep::Turn }), 1);
}

#[test]
fn the_pool_empties_as_a_step_ends_except_mana_kept_all_turn() {
    let mut p = Pool::default();
    p.add(Kind::B, 3);
    p.add(Kind::W, 1);
    p.add_art(2); // Karn, Legacy Reforged: "you don't lose this mana as steps and phases end"
    p.add_key(Key { restr: Restr::None, kind: Kind::R, keep: Keep::Turn }, 1); // Birgi
    p.end_step();
    assert_eq!((p.get(Kind::B), p.get(Kind::W), p.art(), p.get(Kind::R)), (0, 0, 2, 1));
    p.clear(); // the turn ends
    assert!(p.is_empty());
}

#[test]
fn birgis_mana_is_kept_until_the_end_of_the_turn() {
    let mut g = table(&["sauron", "seph"]);
    perm(&mut g, P0, "Birgi, God of Storytelling // Harnfel, Horn of Bounty");
    let c = hand(&mut g, P0, &["Dark Ritual"])[0];
    assert!(
        sim_core::engine::cast::cast_card(&mut g, P0, c, sim_core::sym::intern("hand"), Default::default()).unwrap()
    );
    let kept = Key { restr: Restr::None, kind: Kind::R, keep: Keep::Turn };
    assert!(g.player(P0).mana.amount(kept) >= 1, "{:?}", g.player(P0).mana);
    g.player_mut(P0).mana.end_step();
    assert!(g.player(P0).mana.amount(kept) >= 1);
}

#[test]
fn mishras_workshops_leftover_still_pays_only_for_artifact_spells() {
    let mut g = game();
    lands(&mut g, P0, "Mishra's Workshop", 1, false);
    let sol = card(&g, "Sol Ring");
    g.pay_for = Some(sol);
    assert!(pay(&mut g, P0, 1, "", false).unwrap());
    let left = Key { restr: Restr::ArtifactSpell, kind: Kind::C, keep: Keep::Step };
    assert_eq!(g.player(P0).mana.amount(left), 2, "{{C}}{{C}} floats, for artifact spells");
    assert!(can_pay(&g, P0, 2, "", false), "another artifact spell");
    g.pay_for = Some(card(&g, "Counterspell"));
    assert!(!can_pay(&g, P0, 1, "", false), "not a nonartifact spell");
    g.pay_for = None;
    assert!(!can_pay(&g, P0, 1, "", false), "not an ability");
}

#[test]
fn the_mightstones_leftover_cant_pay_for_a_nonartifact_spell() {
    let mut g = game();
    let m = perm(&mut g, P0, "The Mightstone and Weakstone");
    g.perm_mut(m).sick = false;
    g.pay_for = None; // an ability
    assert!(pay(&mut g, P0, 1, "", false).unwrap());
    let left = Key { restr: Restr::NotNonartifact, kind: Kind::C, keep: Keep::Step };
    assert_eq!(g.player(P0).mana.amount(left), 1);
    g.pay_for = Some(card(&g, "Counterspell"));
    assert!(!can_pay(&g, P0, 1, "", false));
    g.pay_for = Some(card(&g, "Sol Ring"));
    assert!(can_pay(&g, P0, 1, "", false));
}

#[test]
fn white_floating_mana_is_spent() {
    let mut g = game();
    g.player_mut(P0).ident = sim_core::cards::Colors::from_letters("W");
    g.player_mut(P0).mana.add(Kind::W, 1); // Photon, Mighty Marvel's white
    assert!(can_pay(&g, P0, 0, "W", false));
    assert!(pay(&mut g, P0, 0, "W", false).unwrap());
    assert_eq!(g.player(P0).mana.get(Kind::W), 0);
}

#[test]
fn cavern_of_souls_coloured_mana_makes_the_creature_spell_uncounterable() {
    let mut g = table(&["seph", "karn"]);
    let l = lands(&mut g, P0, "Cavern of Souls", 1, false)[0];
    g.perm_mut(l).data.set(DataKey::Ctype, Val::Str("human"));
    let human =
        g.db.iter()
            .find(|d| {
                d.creature && d.has_subtype("human") && d.generic == 0 && d.pips.len() == 1 && !d.has_kw("changeling")
            })
            .map(|d| d.id);
    let Some(c) = human else { panic!("no one-mana Human creature in the card data") };
    let pip = g.db.get(c).pips.to_string();
    g.player_mut(P0).ident = sim_core::cards::Colors::from_letters(&pip);
    g.pay_for = Some(c);
    assert!(pay(&mut g, P0, 0, &pip, false).unwrap());
    g.pay_for = None;
    assert!(sim_core::engine::stack::mistrised(&g, c), "that spell can't be countered");
    // its {C} is just colourless mana
    let mut g = table(&["seph", "karn"]);
    let l = lands(&mut g, P0, "Cavern of Souls", 1, false)[0];
    g.perm_mut(l).data.set(DataKey::Ctype, Val::Str("human"));
    let sol = card(&g, "Sol Ring");
    g.pay_for = Some(sol);
    assert!(pay(&mut g, P0, 1, "", false).unwrap());
    assert!(!sim_core::engine::stack::mistrised(&g, sol));
}

#[test]
fn maelstrom_of_the_spirit_dragon_makes_coloured_mana_only_for_dragons() {
    let mut g = table(&["seph", "karn"]);
    g.player_mut(P0).ident = sim_core::cards::Colors::from_letters("G");
    lands(&mut g, P0, "Maelstrom of the Spirit Dragon", 1, false);
    g.pay_for = Some(card(&g, "Sol Ring"));
    assert!(can_pay(&g, P0, 1, "", false), "{{C}} for anything");
    assert!(!can_pay(&g, P0, 0, "G", false), "no colour for a non-Dragon");
    let dragon = g.db.iter().find(|d| d.has_subtype("dragon")).map(|d| d.id).expect("a Dragon");
    g.pay_for = Some(dragon);
    assert!(can_pay(&g, P0, 0, "G", false), "any colour for a Dragon spell");
    let _ = P1;
}
