//! `data/plans/urdragon.toml`: the keep rule and the plan's wiring for JD's Ur-Dragon deck (the guide's Mulligans table).

use sim_core::ai::plan_data;
use sim_core::ids::PlayerId;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);

fn tbl() -> Game {
    table(&["urdragon", "miirym-temur-dragons", "urza-mono-blue-artifacts", "zur-esper-enchantment-control"])
}

fn keeps(cards: &[&str]) -> bool {
    let mut g = tbl();
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, cards);
    plan_data::keeps_hand(&g, P0).expect("urdragon has a plan with a mulligan rule")
}

#[test]
fn a_hand_with_lands_ramp_and_a_dragon_is_a_keep() {
    // the guide's hand 1: Turn 2 Sol Ring and Birds, turn 3 Terror
    assert!(keeps(&[
        "Taiga",
        "Windswept Heath",
        "Birds of Paradise",
        "Sol Ring",
        "Terror of the Peaks",
        "Lightning Bolt",
        "Swords to Plowshares"
    ]));
}

#[test]
fn a_hand_of_only_seven_drops_goes_back_even_with_mana() {
    assert!(!keeps(&[
        "Tundra",
        "Bayou",
        "Taiga",
        "Klauth, Unrivaled Ancient",
        "Old Gnawbone",
        "Tiamat",
        "Utvara Hellkite"
    ]));
    // the guide's hand 2: one colourless land and nothing before turn 5
    assert!(!keeps(&[
        "Ancient Tomb",
        "Mana Vault",
        "Mox Jasper",
        "Old Gnawbone",
        "Klauth, Unrivaled Ancient",
        "Utvara Hellkite",
        "Rivaz of the Claw"
    ]));
}

#[test]
fn lands_outside_two_to_five_and_one_landers_need_fast_mana() {
    assert!(
        !keeps(&["Tundra", "Bayou", "Taiga", "Plateau", "Badlands", "Savannah", "Terror of the Peaks"]),
        "six lands"
    );
    assert!(
        !keeps(&[
            "Terror of the Peaks",
            "Goldspan Dragon",
            "Glorybringer",
            "Thundermaw Hellkite",
            "Rhystic Study",
            "Lightning Bolt",
            "Swords to Plowshares"
        ]),
        "no land"
    );
    assert!(
        !keeps(&[
            "Tundra",
            "Terror of the Peaks",
            "Goldspan Dragon",
            "Glorybringer",
            "Thundermaw Hellkite",
            "Rhystic Study",
            "Lightning Bolt"
        ]),
        "one land, no mana"
    );
    assert!(
        keeps(&[
            "Tundra",
            "Sol Ring",
            "Terror of the Peaks",
            "Goldspan Dragon",
            "Glorybringer",
            "Rhystic Study",
            "Lightning Bolt"
        ]),
        "one land with Sol Ring"
    );
    assert!(
        keeps(&[
            "Bayou",
            "Birds of Paradise",
            "Llanowar Elves",
            "Terror of the Peaks",
            "Goldspan Dragon",
            "Rhystic Study",
            "Lightning Bolt"
        ]),
        "one land with two dorks"
    );
}

#[test]
fn a_hand_with_nothing_to_do_or_too_little_mana_goes_back() {
    assert!(
        !keeps(&["Tundra", "Bayou", "Taiga", "Lightning Bolt", "Swords to Plowshares", "Path to Exile", "Chaos Warp"]),
        "no action card"
    );
    assert!(
        !keeps(&[
            "Tundra",
            "Bayou",
            "Terror of the Peaks",
            "Goldspan Dragon",
            "Glorybringer",
            "Thundermaw Hellkite",
            "Rhystic Study"
        ]),
        "two lands and no mana piece"
    );
    // the guide's hand 3 (a close keep): two duals, two ramp spells, a dork
    assert!(keeps(&[
        "Tundra",
        "Bayou",
        "Elvish Mystic",
        "Nature's Lore",
        "Three Visits",
        "Lathliss, Dragon Queen",
        "Doubling Season"
    ]));
}

#[test]
fn the_plan_is_active_for_the_deck_and_names_the_garth_line() {
    let g = tbl();
    let l = plan_data::active(&g, P0).expect("the plan file is read for urdragon");
    assert!(
        l.plan
            .line
            .iter()
            .any(|x| x.combo.as_deref() == Some("Garth One-Eye + Twinflame + Spark Double + Doubling Season"))
    );
    // Twinflame and the wipes must keep their generic rating: no priority rule may name them
    for name in ["Twinflame", "Crux of Fate", "Austere Command", "Decimate", "Lightning Bolt", "Teferi's Protection"] {
        assert!(!l.plan.prio.iter().any(|r| r.cards.iter().any(|c| c == name)), "{name} has a [[prio]] row");
    }
}

#[test]
fn a_lone_land_that_cannot_cast_the_dorks_goes_back() {
    // Tundra makes {W}{U}; Birds and Llanowar Elves need {G}
    assert!(!keeps(&[
        "Tundra",
        "Birds of Paradise",
        "Llanowar Elves",
        "Terror of the Peaks",
        "Goldspan Dragon",
        "Rhystic Study",
        "Lightning Bolt"
    ]));
    // a lone Ancient Tomb makes no coloured mana
    assert!(!keeps(&[
        "Ancient Tomb",
        "Sol Ring",
        "Terror of the Peaks",
        "Goldspan Dragon",
        "Rhystic Study",
        "Lightning Bolt",
        "Swords to Plowshares"
    ]));
}

#[test]
fn the_guides_other_close_keeps_and_the_ramp_the_rule_counts() {
    // hand 4: tutors and Sol Ring find the rest
    assert!(keeps(&[
        "Command Tower",
        "Sol Ring",
        "Demonic Tutor",
        "Vampiric Tutor",
        "Garth One-Eye",
        "Spark Double",
        "Twinflame"
    ]));
    // two lands plus Three Visits, and a land plus Ancient Tomb, each count as three mana by turn 3
    assert!(keeps(&[
        "Tundra",
        "Bayou",
        "Three Visits",
        "Terror of the Peaks",
        "Goldspan Dragon",
        "Glorybringer",
        "Rhystic Study"
    ]));
    assert!(keeps(&[
        "Ancient Tomb",
        "Bayou",
        "Terror of the Peaks",
        "Goldspan Dragon",
        "Glorybringer",
        "Thundermaw Hellkite",
        "Rhystic Study"
    ]));
    // ramp alone is an action card
    assert!(keeps(&[
        "Tundra",
        "Bayou",
        "Taiga",
        "Sol Ring",
        "Arcane Signet",
        "Birds of Paradise",
        "Swords to Plowshares"
    ]));
    // a cheap Dragon, a three-mana rock or Temur Ascendancy is something to do (all are in the `plays` set)
    for play in ["Rivaz of the Claw", "Scion of Draco", "Cursed Mirror", "Temur Ascendancy"] {
        assert!(
            keeps(&["Tundra", "Bayou", "Taiga", play, "Lightning Bolt", "Swords to Plowshares", "Path to Exile"]),
            "{play} counts as something to do"
        );
    }
}
