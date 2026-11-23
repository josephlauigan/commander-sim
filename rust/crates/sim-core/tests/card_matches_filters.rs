//! `dsl::card_matches` (cards in a library, hand or graveyard: search, regrow, reanimate) reads every field of the
//! filter, and the compiled cards that carry those fields work from the Oracle text.

mod play_common;
use play_common::*;
use serde_json::from_str;
use sim_core::dsl::model::Filter;
use sim_core::dsl::{card_matches, card_matches_except, use_loyalty};
use sim_core::engine::zones;

fn filter(json: &str) -> Filter {
    from_str(json).unwrap()
}

fn game() -> Game {
    table(&["seph", "cloud"])
}

/// the cards of `names` that satisfy the filter
fn pass<'a>(g: &Game, names: &[&'a str], f: &Filter) -> Vec<&'a str> {
    names.iter().copied().filter(|n| card_matches(g, card(g, n), Some(f))).collect()
}

const CARDS: [&str; 5] = ["Llanowar Elves", "Goblin Guide", "Wrathful Red Dragon", "Dragonlord Ojutai", "Sol Ring"];

#[test]
fn legendary_and_nonlegendary_filter_cards() {
    let g = game();
    assert_eq!(pass(&g, &CARDS, &filter(r#"{"type":"creature","legendary":true}"#)), ["Dragonlord Ojutai"]);
    let non = pass(&g, &CARDS, &filter(r#"{"type":"creature","nonlegendary":true}"#));
    assert_eq!(non, ["Llanowar Elves", "Goblin Guide", "Wrathful Red Dragon"]);
}

#[test]
fn subtype_filters_cards_and_negation() {
    let g = game();
    assert_eq!(pass(&g, &CARDS, &filter(r#"{"subtype":"dragon"}"#)), ["Wrathful Red Dragon", "Dragonlord Ojutai"]);
    assert_eq!(pass(&g, &CARDS, &filter(r#"{"subtype":"goblin"}"#)), ["Goblin Guide"]);
    assert_eq!(
        pass(&g, &CARDS, &filter(r#"{"type":"creature","subtype":"non-dragon"}"#)),
        ["Llanowar Elves", "Goblin Guide"]
    );
}

#[test]
fn power_keyword_and_colour_filter_cards() {
    let g = game();
    assert_eq!(
        pass(&g, &CARDS, &filter(r#"{"type":"creature","min_pow":2}"#)),
        ["Goblin Guide", "Wrathful Red Dragon", "Dragonlord Ojutai"]
    );
    assert_eq!(pass(&g, &CARDS, &filter(r#"{"kw":"flying"}"#)), ["Wrathful Red Dragon", "Dragonlord Ojutai"]);
    assert_eq!(pass(&g, &CARDS, &filter(r#"{"keyword":"haste"}"#)), ["Goblin Guide"]);
    assert_eq!(pass(&g, &CARDS, &filter(r#"{"type":"creature","color":"G"}"#)), ["Llanowar Elves"]);
    assert_eq!(
        pass(&g, &CARDS, &filter(r#"{"type":"creature","color":"RG"}"#)),
        ["Llanowar Elves", "Goblin Guide", "Wrathful Red Dragon"]
    );
}

#[test]
fn battlefield_only_fields_match_no_card_and_other_excludes_the_source() {
    let g = game();
    assert!(pass(&g, &CARDS, &filter(r#"{"tapped":true}"#)).is_empty(), "a card in a zone is not tapped");
    assert!(pass(&g, &CARDS, &filter(r#"{"attacking":true}"#)).is_empty(), "or attacking");
    assert_eq!(pass(&g, &CARDS, &filter(r#"{"nontoken":true,"type":"artifact"}"#)), ["Sol Ring"]);
    let (f, elf) = (filter(r#"{"other":true,"subtype":"elf"}"#), card(&g, "Llanowar Elves"));
    assert!(card_matches(&g, elf, Some(&f)), "without a source, another card is any card");
    assert!(!card_matches_except(&g, elf, Some(&f), Some(elf)), "the source is not another card");
}

/// Sarkhan Unbroken -8: "Search your library for a Dragon card, put it onto the battlefield, then shuffle."
fn sarkhan_minus_eight(lib: &[&str]) -> Vec<String> {
    let mut g = game();
    let s = perm(&mut g, P0, "Sarkhan Unbroken");
    g.perm_mut(s).loyalty = Some(9);
    g.player_mut(P0).library = lib.iter().map(|n| card(&g, n)).collect();
    assert!(use_loyalty(&mut g, P0, s, 2).unwrap());
    g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf).map(|&m| g.perm(m).name.to_string()).collect()
}

#[test]
fn sarkhan_unbroken_minus_eight_finds_a_dragon_only() {
    let bf = sarkhan_minus_eight(&["Goblin Guide", "Llanowar Elves", "Sol Ring"]);
    assert!(!bf.iter().any(|n| n == "Goblin Guide" || n == "Llanowar Elves" || n == "Sol Ring"), "no Dragon: {bf:?}");
    let bf = sarkhan_minus_eight(&["Goblin Guide", "Wrathful Red Dragon", "Llanowar Elves"]);
    assert!(bf.iter().any(|n| n == "Wrathful Red Dragon"), "the Dragon came in: {bf:?}");
    assert!(!bf.iter().any(|n| n == "Goblin Guide" || n == "Llanowar Elves"), "and nothing else did: {bf:?}");
}

/// Morcant's Loyalist: "When Morcant's Loyalist dies, return another target Elf card from your graveyard to your hand."
#[test]
fn morcants_loyalist_returns_another_elf_only() {
    let mut g = game();
    let loyalist = perm(&mut g, P0, "Morcant's Loyalist");
    let (elf, goblin, dragon, warrior) =
        ["Llanowar Elves", "Goblin Guide", "Wrathful Red Dragon", "Alesha, Who Laughs at Fate"]
            .map(|n| card(&g, n))
            .into();
    g.player_mut(P0).gy.extend([goblin, dragon, elf, warrior]); // (a Warrior is not an Elf)
    zones::die(&mut g, loyalist, "destroy").unwrap();
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert!(g.player(P0).hand.contains(&elf), "the Elf came back");
    assert!(!g.player(P0).hand.contains(&goblin) && !g.player(P0).hand.contains(&dragon), "no other card did");
    assert!(!g.player(P0).hand.contains(&warrior), "a Warrior is not an Elf");
    let lc = card(&g, "Morcant's Loyalist");
    assert!(g.player(P0).gy.contains(&lc), "the Loyalist stays in the graveyard: it is not another card");
    assert!(!g.player(P0).hand.contains(&lc));
}

#[test]
fn morcants_loyalist_with_nothing_else_to_return_returns_nothing() {
    let mut g = game();
    let loyalist = perm(&mut g, P0, "Morcant's Loyalist");
    let goblin = card(&g, "Goblin Guide");
    g.player_mut(P0).gy.push(goblin);
    let hand_before = g.player(P0).hand.len();
    zones::die(&mut g, loyalist, "destroy").unwrap();
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert_eq!(g.player(P0).hand.len(), hand_before, "a Goblin and the Loyalist itself are not returnable");
}
