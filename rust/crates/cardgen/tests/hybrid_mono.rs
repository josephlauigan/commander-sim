//! Monocoloured hybrid symbols ({2/W}) in the export: the colour stays a pip, `hybrid` lists "W2", the mana value is 2 for it.
//! Real data only (the Scryfall cache; no network).

use cardgen::autotag::conv_cost;
use cardgen::cardinfo::{hybrid, hybrid_of_cost, mana_value};
use cardgen::cd::Cd;
use cardgen::scryfall::Cache;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn cd(name: &str, mc: &str) -> Cd {
    Cd::new(name, "AC", &conv_cost(mc), "")
}

#[test]
fn conv_cost_keeps_each_two_hybrid_colour_as_a_pip() {
    let mc = "{2/W}{2/U}{2/B}{2/R}{2/G}";
    assert_eq!(conv_cost(mc), "WUBRG");
    let k = cd("Karn, Gilded Guardian", mc);
    assert_eq!((k.generic, k.pips.as_str()), (0, "WUBRG"));
    let h = hybrid_of_cost(mc);
    assert_eq!(h, ["W2", "U2", "B2", "R2", "G2"]);
    assert_eq!(mana_value(&k, &h), 10, "each {{2/X}} has mana value 2");
}

#[test]
fn other_costs_are_unchanged() {
    assert_eq!(conv_cost("{2}{W}{W}"), "2WW");
    assert_eq!(conv_cost("{2}{G/W}{B/P}"), "2GB", "two-colour hybrid and Phyrexian take the first colour, as before");
    assert_eq!(hybrid_of_cost("{2}{G/W}{B/P}{2/B}"), ["GW", "B2"]);
    let c = cd("Kitchen Finks", "{1}{G/W}{G/W}");
    assert_eq!(mana_value(&c, &hybrid_of_cost("{1}{G/W}{G/W}")), 3);
}

#[test]
fn the_four_real_cards() {
    let cache = Cache::load(&root().join("data/scryfall_cache.json")).unwrap();
    for (name, mc, hy, cmc) in [
        ("Karn, Gilded Guardian", "{2/W}{2/U}{2/B}{2/R}{2/G}", vec!["W2", "U2", "B2", "R2", "G2"], 10),
        ("The Reaper, King No More", "{2/B}{2/R}{2/G}", vec!["B2", "R2", "G2"], 6),
        ("Spectral Procession", "{2/W}{2/W}{2/W}", vec!["W2", "W2", "W2"], 6),
    ] {
        let c = cd(name, mc);
        let h = hybrid(&cache, &c);
        assert_eq!(h, hy, "{name}");
        assert_eq!(mana_value(&c, &h), cmc, "{name}");
    }
}

#[test]
fn a_card_whose_cost_an_override_states_lists_no_two_hybrid() {
    // Wildgrowth Archaic: the override says {G}{G} (its Scryfall cost is {2/G}{2/G}); the override is kept as it is
    let cache = Cache::load(&root().join("data/scryfall_cache.json")).unwrap();
    let mut c = Cd::new("Wildgrowth Archaic", "C", "GG", "");
    assert_eq!(hybrid(&cache, &c), ["G2", "G2"], "a card built from Scryfall lists them");
    // an override that does not state a cost (a `dsl = []` block, tags, types) keeps the symbols: Karn, Gilded Guardian's override block is one
    c.pool_override = true;
    assert_eq!(hybrid(&cache, &c), ["G2", "G2"], "an override that states no cost does not hide the {{2/G}} symbols");
    c.cost_override = true;
    assert!(hybrid(&cache, &c).is_empty());
    assert_eq!(mana_value(&c, &[]), 2);
}
