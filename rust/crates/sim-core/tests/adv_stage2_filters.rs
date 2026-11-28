//! Adversarial coverage of the stage2 card matcher, including every compiled zone-filter effect.
mod play_common;
use play_common::*;
use sim_core::cards::{CardDb, Types};
use sim_core::dsl::model::{Effect, Filter};
use sim_core::dsl::{self, DslCtx, card_matches, card_matches_except};
use sim_core::engine::zones;
use sim_core::export::load_cards;
use sim_core::ids::CardId;
use sim_core::tag::Tag;
use std::sync::Arc;

fn filter(v: Value) -> Filter {
    serde_json::from_value(v).unwrap()
}

/// The exact origin/main predicate: type/basic, mana value, maximum power and nonlegendary only.
fn old_matches(g: &Game, c: CardId, f: &Filter) -> bool {
    let d = g.db.get(c);
    let ty = |t| d.types.has(t);
    let kind = match f.kind.as_deref() {
        None | Some("any") => true,
        Some("permanent") => d.perm || d.land,
        Some("creature") => d.creature,
        Some("land") => d.land && (f.basic != Some(true) || zones::BASIC_LANDS.contains(&&*d.name)),
        Some("artifact") => ty(Types::ARTIFACT),
        Some("equipment") => d.has_subtype("equipment"),
        Some("enchantment") => ty(Types::ENCHANTMENT),
        Some("instant_or_sorcery") => d.instant || d.sorcery,
        Some("noncreature") => !d.creature && !d.land,
        Some("planeswalker") => ty(Types::PLANESWALKER),
        Some("nonland") | Some("spell") => !d.land,
        Some("cp") => d.creature || ty(Types::PLANESWALKER),
        Some("ac") => d.creature || ty(Types::ARTIFACT),
        Some("acp") => d.creature || ty(Types::ARTIFACT) || ty(Types::PLANESWALKER),
        Some("ce") => d.creature || ty(Types::ENCHANTMENT),
        Some("ae") => ty(Types::ARTIFACT) || ty(Types::ENCHANTMENT),
        Some("nonartifact_creature") => d.creature && !ty(Types::ARTIFACT),
        Some(_) => true,
    };
    kind && !f.max_mv.is_some_and(|v| d.cmc > v)
        && !f.min_mv.is_some_and(|v| d.cmc < v)
        && !f.max_pow.is_some_and(|v| d.pow > v)
        && !(f.nonlegendary == Some(true) && d.tag(Tag::Leg))
}

fn zone_effects<'a>(v: &'a Value, path: String, out: &mut Vec<(String, &'a Value)>) {
    match v {
        Value::Object(o) => {
            if matches!(v["do"].as_str(), Some("search" | "regrow" | "reanimate")) {
                out.push((path.clone(), v));
            }
            for (k, v) in o {
                zone_effects(v, format!("{path}/{k}"), out);
            }
        }
        Value::Array(a) => {
            for (i, v) in a.iter().enumerate() {
                zone_effects(v, format!("{path}/{i}"), out);
            }
        }
        _ => {}
    }
}

#[test]
fn inventory_every_compiled_zone_filter_against_every_card() {
    let g = table(&["seph", "cloud"]);
    let data: Value = serde_json::from_str(&std::fs::read_to_string(data_dir().join("cards.json")).unwrap()).unwrap();
    let mut count = 0;
    let mut changed = vec![];
    for c in data["cards"].as_array().unwrap() {
        let name = c["name"].as_str().unwrap();
        let mut effects = vec![];
        zone_effects(&c["dsl"], "dsl".into(), &mut effects);
        for (path, e) in effects {
            count += 1;
            let f: Filter = serde_json::from_value(e.get("filter").cloned().unwrap_or(json!({}))).unwrap();
            assert_ne!(f.tapped, Some(true), "{name} {path}: battlefield state in a card filter");
            assert_ne!(f.attacking, Some(true), "{name} {path}: battlefield state in a card filter");
            let mut deltas = 0;
            for d in g.db.iter() {
                let old = old_matches(&g, d.id, &f);
                let new = card_matches_except(&g, d.id, Some(&f), g.db.id(name));
                if old != new {
                    deltas += 1;
                    let expected = match name {
                        "Maelstrom of the Spirit Dragon" => d.has_subtype("dragon") || d.has_kw("changeling"),
                        "Sarkhan Unbroken" => d.creature && (d.has_subtype("dragon") || d.has_kw("changeling")),
                        "Morcant's Loyalist" => (d.has_subtype("elf") || d.has_kw("changeling")) && &*d.name != name,
                        _ => panic!("unexpected changed filter: {name} {path}, candidate {}", d.name),
                    };
                    assert_eq!(new, expected, "{name} {path}: candidate {}", d.name);
                }
            }
            if deltas > 0 {
                changed.push(name);
            }
            eprintln!("{name} | {path} | {} | old/new delta candidates: {deltas}", e["filter"]);
        }
    }
    // 103 before the Trostani deck: Kodama's Reach's two compiled searches (a basic land tapped onto the battlefield, a
    // basic land to hand) went when data/carddb.txt gave it Cultivate's land-ramp tags (lr=1 lrt lh=1), which the engine runs
    // as it does Cultivate's; no other card lost an effect (the six dsl = [] overrides of the Trostani deck: Trostani,
    // Branching Evolution, Conclave Mentor, Treebeard, The Doctor and Elder Gargaroth, had no zone effect to lose)
    assert_eq!(count, 101);
    assert_eq!(changed, ["Maelstrom of the Spirit Dragon", "Morcant's Loyalist", "Sarkhan Unbroken"]);
    eprintln!("Compared {count} effects against all {} card definitions", g.db.len());
}

#[test]
fn changeling_does_not_grant_land_or_artifact_or_enchantment_subtypes() {
    let g = table(&["seph", "cloud"]);
    let c = card(&g, "Changeling Outcast");
    assert!(card_matches(&g, c, Some(&filter(json!({"subtype":"dragon"})))));
    let wrong: Vec<_> = ["forest", "equipment", "aura", "basic"]
        .into_iter()
        .filter(|st| card_matches(&g, c, Some(&filter(json!({"subtype": st})))))
        .collect();
    assert!(wrong.is_empty(), "changeling incorrectly confers these non-creature types: {wrong:?}");
}

#[test]
fn noncreature_changeling_is_a_dragon_card_for_maelstrom() {
    let mut g = table(&["seph", "cloud"]);
    let mut raw = load_cards(&data_dir().join("cards.json")).unwrap();
    // Nameless Inversion is a Kindred Instant — Shapeshifter with changeling. The export has no Kindred code;
    // its instant, subtype and keyword fields suffice for the predicate under test. Keep existing IDs intact.
    let mut r = raw.iter().find(|c| c.name == "Lightning Bolt").unwrap().clone();
    r.name = "Nameless Inversion (adversarial fixture)".into();
    r.generic = 1;
    r.pips = "B".into();
    r.kws = vec!["changeling".into()];
    r.subtypes = vec!["shapeshifter".into()];
    r.dsl = None;
    r.derived = r.compute_derived();
    raw.push(r);
    g.db = Arc::new(CardDb::from_raw(&raw).unwrap());
    let c = card(&g, "Nameless Inversion (adversarial fixture)");
    assert!(!g.db.get(c).creature);
    let maelstrom = card(&g, "Maelstrom of the Spirit Dragon");
    let sim_core::dsl::model::Ability::Activated { effects, .. } = &g.db.get(maelstrom).abilities[0] else {
        panic!("Maelstrom has a compiled activated search");
    };
    assert!(card_matches(&g, c, effects[0].filter.as_ref()), "a noncreature changeling is every creature type too");
}

#[test]
fn negated_subtypes_unset_kind_multicolour_and_keyword_alias_controls() {
    let g = table(&["seph", "cloud"]);
    let changeling = card(&g, "Changeling Outcast");
    assert!(card_matches(&g, changeling, Some(&filter(json!({"subtype":"time lord"})))));
    assert!(!card_matches(&g, changeling, Some(&filter(json!({"subtype":"non-dragon"})))));
    for (name, f, expected) in [
        ("Forest", json!({"subtype":"forest"}), true),
        ("Sol Ring", json!({"color":""}), true),
        ("Sol Ring", json!({"color":"C"}), false),
        ("Dragonlord Ojutai", json!({"color":"U"}), true),
        ("Dragonlord Ojutai", json!({"color":"RG"}), false),
        ("Goblin Guide", json!({"kw":"haste"}), true),
        ("Goblin Guide", json!({"keyword":"haste"}), true),
        ("Goblin Guide", json!({"keyword":"Haste"}), false),
        ("Goblin Guide", json!({"min_pow":2}), true),
        ("Goblin Guide", json!({"min_pow":3}), false),
        ("Sol Ring", json!({"nontoken":true}), true),
        ("Sol Ring", json!({"tapped":true}), false),
        ("Sol Ring", json!({"attacking":true}), false),
    ] {
        assert_eq!(card_matches(&g, card(&g, name), Some(&filter(f.clone()))), expected, "{name} {f}");
    }
}

#[test]
#[ignore = "ticket: card colours come from the mana cost only"]
fn colour_filters_respect_colour_indicators_on_cards_without_coloured_mana_costs() {
    let g = table(&["seph", "cloud"]);
    let failures: Vec<_> = [("Dryad Arbor", "G"), ("Pact of Negation", "U")]
        .into_iter()
        .filter(|(name, colour)| !card_matches(&g, card(&g, name), Some(&filter(json!({"color":colour})))))
        .collect();
    assert!(failures.is_empty(), "printed colour indicators were ignored: {failures:?}");
}

#[test]
#[ignore = "ticket: other excludes by CardId, not by name"]
fn loyalist_can_return_a_different_card_with_the_same_name() {
    let mut g = table(&["seph", "cloud"]);
    let src = perm(&mut g, P0, "Morcant's Loyalist");
    let c = card(&g, "Morcant's Loyalist");
    g.player_mut(P0).gy = vec![c]; // a different physical card, already in the graveyard
    zones::die(&mut g, src, "sac").unwrap();
    assert_eq!(g.player(P0).hand, [c], "another same-named Elf is legal; only the card that died is excluded");
    assert_eq!(g.player(P0).gy, [c]);
}

#[test]
#[ignore = "ticket: Maskwood Nexus is not read for cards in zones"]
fn maskwood_makes_a_library_creature_a_dragon_for_sarkhan() {
    let mut g = table(&["seph", "cloud"]);
    perm(&mut g, P0, "Maskwood Nexus");
    let s = perm(&mut g, P0, "Sarkhan Unbroken");
    g.perm_mut(s).loyalty = Some(9);
    let c = card(&g, "Goblin Guide");
    g.player_mut(P0).library = vec![c];
    assert!(dsl::use_loyalty(&mut g, P0, s, 2).unwrap());
    assert!(
        has_perm(&g, P0, "Goblin Guide"),
        "Maskwood makes creature cards outside the battlefield every creature type"
    );
}

#[test]
#[ignore = "ticket: Maskwood Nexus is not read for cards in zones"]
fn maskwood_makes_a_graveyard_creature_an_elf_for_loyalist() {
    let mut g = table(&["seph", "cloud"]);
    perm(&mut g, P0, "Maskwood Nexus");
    let src = perm(&mut g, P0, "Morcant's Loyalist");
    let c = card(&g, "Goblin Guide");
    g.player_mut(P0).gy = vec![c];
    zones::die(&mut g, src, "sac").unwrap();
    assert_eq!(g.player(P0).hand, [c], "Maskwood makes the Goblin card an Elf card in the graveyard");
}

#[test]
fn other_with_token_and_spell_sources_does_not_exclude_unrelated_cards() {
    for token_source in [true, false] {
        let mut g = table(&["seph", "cloud"]);
        let elf = card(&g, "Llanowar Elves");
        let source_card = card(&g, "Regrowth");
        g.player_mut(P0).gy = vec![elf];
        let src = token_source.then(|| token(&mut g, P0, 2));
        let e: Effect = serde_json::from_value(json!({"do":"regrow","filter":{"other":true,"subtype":"elf"}})).unwrap();
        dsl::execute(&mut g, P0, &[e], src, &mut DslCtx::default(), (!token_source).then_some(source_card), 0).unwrap();
        assert_eq!(g.player(P0).hand, [elf]);
    }
}
