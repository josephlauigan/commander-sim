//! land-audit2: nonbasic lands that made more mana than their Oracle text allows (filter lands, {1},{T} lands, Spire of
//! Industry, Exotic Orchard, Cavern of Souls, Hall of the Bandit Lord, Spectator Seating, Sulfur Falls, Fabled Passage).

use sim_core::engine::mana::{self, Source, land_colors_now, mana_units};
use sim_core::ids::{LandId, PlayerId};
use sim_core::state::Game;
use sim_core::tag::Tag;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn amt(g: &Game, l: LandId) -> u32 {
    mana_units(g, P0, false).iter().find(|u| u.src == Source::Land(l)).unwrap().amt
}

fn cols(g: &Game, l: LandId) -> String {
    let c = land_colors_now(g, P0, l);
    "WUBRG".chars().filter(|&ch| c.has(ch)).collect()
}

#[test]
fn every_filter_land_needs_an_untapped_land_that_makes_one_of_its_colours() {
    let mut g = table(&["meren-golgari-recursion", "veyran"]);
    let mire = lands(&mut g, P0, "Twilight Mire", 1, false)[0];
    assert_eq!(cols(&g, mire), "", "alone: colourless only");
    lands(&mut g, P0, "Plains", 1, false);
    assert_eq!(cols(&g, mire), "", "a Plains makes neither {{B}} nor {{G}}");
    let swamp = lands(&mut g, P0, "Swamp", 1, true)[0];
    assert_eq!(cols(&g, mire), "", "a tapped Swamp cannot pay");
    g.land_mut(swamp).tapped = false;
    assert_eq!(cols(&g, mire), "BG", "an untapped Swamp pays the {{B/G}}");
}

#[test]
fn two_filter_lands_alone_make_nothing_but_a_fed_one_feeds_the_next() {
    let mut g = table(&["aurelia-boros-extra-combats", "veyran"]);
    let prairie = lands(&mut g, P0, "Rugged Prairie", 1, false)[0];
    let gate = lands(&mut g, P0, "Mystic Gate", 1, false)[0];
    assert_eq!(cols(&g, prairie), "", "two filters and no real source");
    assert_eq!(cols(&g, gate), "");
    lands(&mut g, P0, "Island", 1, false);
    assert_eq!(cols(&g, gate), "WU", "the Island pays the Gate");
    assert_eq!(cols(&g, prairie), "WR", "the fed Gate makes {{W}} to pay the Prairie");
    // Fetid Heath and Cascade Bluffs keep working
    let heath = lands(&mut g, P0, "Fetid Heath", 1, false)[0];
    assert_eq!(cols(&g, heath), "WB", "the Gate's {{W}} pays the Heath");
}

#[test]
fn sungrass_prairie_and_ferrous_lake_need_another_untapped_land_and_net_one_mana() {
    let mut g = table(&["sythis-selesnya-enchantress", "veyran"]);
    let sp = lands(&mut g, P0, "Sungrass Prairie", 1, false)[0];
    assert_eq!(amt(&g, sp), 0, "nothing to pay the {{1}}");
    let f = lands(&mut g, P0, "Forest", 1, true)[0];
    assert_eq!(amt(&g, sp), 0, "a tapped Forest cannot pay");
    g.land_mut(f).tapped = false;
    assert_eq!(amt(&g, sp), 1);
    let mut g = table(&["thrasios", "veyran"]);
    let fl = lands(&mut g, P0, "Ferrous Lake", 1, false)[0];
    assert_eq!(amt(&g, fl), 0);
    lands(&mut g, P0, "Island", 1, false);
    assert_eq!(amt(&g, fl), 1);
}

#[test]
fn spire_of_industry_needs_an_artifact_and_costs_a_life() {
    let mut g = table(&["yshtola", "veyran"]);
    let spire = lands(&mut g, P0, "Spire of Industry", 1, false)[0];
    assert_eq!(cols(&g, spire), "", "no artifact: colourless only");
    perm(&mut g, P0, "Arcane Signet");
    assert!(land_colors_now(&g, P0, spire).count() > 1, "with an artifact, any colour of the identity");
    assert!(g.db.get(g.land(spire).cd).tag(Tag::Pain), "coloured mana costs 1 life (the pain tag)");
}

#[test]
fn exotic_orchard_makes_only_colours_an_opponent_land_could_make() {
    let mut g = table(&["omnath", "veyran"]);
    let orchard = lands(&mut g, P0, "Exotic Orchard", 1, false)[0];
    assert_eq!(cols(&g, orchard), "", "the opponents control no lands");
    lands(&mut g, P1, "Forest", 1, false);
    assert_eq!(cols(&g, orchard), "G");
    lands(&mut g, P1, "Exotic Orchard", 1, false);
    assert_eq!(cols(&g, orchard), "G", "an opponent's Orchard adds nothing");
}

#[test]
fn hall_of_the_bandit_lord_costs_three_life_per_mana() {
    let mut g = table(&["krenko-mono-red-goblins", "veyran"]);
    let hall = lands(&mut g, P0, "Hall of the Bandit Lord", 1, false)[0];
    assert_eq!(amt(&g, hall), 1);
    assert!(mana::pay(&mut g, P0, 1, "", false).unwrap());
    assert_eq!(g.player(P0).life, 37, "paid 3 life");
    g.land_mut(hall).tapped = false;
    g.player_mut(P0).life = 3;
    assert_eq!(amt(&g, hall), 0, "not at 3 life");
}

#[test]
fn cavern_of_souls_makes_coloured_mana_for_creature_spells_of_one_type_only() {
    let mut g = table(&["heliod-mono-white-stax", "veyran"]);
    let cavern = lands(&mut g, P0, "Cavern of Souls", 1, false)[0];
    assert_eq!(cols(&g, cavern), "", "no spell: colourless");
    let t = sim_core::impls::mine::territory_type(&g, P0);
    let lib = g.player(P0).library.clone();
    let same = lib
        .iter()
        .copied()
        .find(|&c| g.db.get(c).creature && g.db.get(c).has_subtype(t))
        .expect("a creature of the type");
    let other = lib
        .iter()
        .copied()
        .find(|&c| g.db.get(c).creature && !g.db.get(c).has_subtype(t) && !g.db.get(c).has_kw("changeling"))
        .expect("another creature");
    let spell = lib.iter().copied().find(|&c| !g.db.get(c).creature && !g.db.get(c).land).unwrap();
    g.pay_for = Some(same);
    assert_eq!(cols(&g, cavern), "W", "a creature of the chosen type");
    g.pay_for = Some(other);
    assert_eq!(cols(&g, cavern), "", "another creature type");
    g.pay_for = Some(spell);
    assert_eq!(cols(&g, cavern), "", "a noncreature spell");
}

#[test]
fn spectator_seating_makes_red_or_white_and_sulfur_falls_deals_no_damage() {
    let mut g = table(&["thrasios", "veyran"]);
    let ss = lands(&mut g, P0, "Spectator Seating", 1, false)[0];
    assert_eq!(cols(&g, ss), "WR");
    assert!(!g.db.get(g.land(ss).cd).tag(Tag::Pain));
    let sf = lands(&mut g, P0, "Sulfur Falls", 1, false)[0];
    assert!(!g.db.get(g.land(sf).cd).tag(Tag::Pain), "a check land, not a painland");
}

#[test]
fn hidden_grotto_taps_for_colourless() {
    let mut g = table(&["alela", "veyran"]);
    let hg = lands(&mut g, P0, "Hidden Grotto", 1, false)[0];
    lands(&mut g, P0, "Plains", 1, false);
    assert_eq!(cols(&g, hg), "", "the {{1}},{{T}} any-colour mode nets no mana and is left out");
    assert_eq!(amt(&g, hg), 1);
}

#[test]
fn fabled_passage_untaps_the_land_with_three_other_lands() {
    for (others, tapped) in [(2usize, true), (3, false)] {
        let mut g = table(&["omnath", "veyran"]);
        lands(&mut g, P0, "Forest", others, false);
        let before = g.player(P0).lands.len();
        let fp = lands(&mut g, P0, "Fabled Passage", 1, false)[0];
        sim_core::engine::turn::crack_fetch(&mut g, P0, fp).unwrap();
        assert_eq!(g.player(P0).lands.len(), before + 1, "the Passage is replaced by a basic");
        let new = *g.player(P0).lands.last().unwrap();
        assert_eq!(g.land(new).tapped, tapped, "{others} other lands");
    }
}

#[test]
fn orchards_and_filter_lands_on_both_sides_do_not_recurse() {
    let mut g = table(&["galadriel", "meren-golgari-recursion"]);
    let o0 = lands(&mut g, P0, "Exotic Orchard", 1, false)[0];
    lands(&mut g, P0, "Mystic Gate", 1, false);
    lands(&mut g, P1, "Exotic Orchard", 1, false);
    lands(&mut g, P1, "Twilight Mire", 1, false);
    let _ = cols(&g, o0);
    let _ = mana_units(&g, P0, false);
    let _ = mana_units(&g, P1, false);
}

#[test]
fn dryad_arbor_cannot_tap_the_turn_it_enters() {
    let mut g = table(&["marwyn-mono-green-elves", "veyran"]);
    let c = take(&mut g, P0, "Dryad Arbor");
    assert!(sim_core::engine::turn::land_enters_tapped(&g, P0, c), "summoning sick: no mana this turn");
}
