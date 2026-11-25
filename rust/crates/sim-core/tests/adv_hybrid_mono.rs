//! Adversarial checks for 5af7fd67 (monocoloured hybrid {2/X}: Karn, Gilded Guardian; The Reaper, King No More; Spectral
//! Procession; Transguild Courier is all colors). Written by a reviewer who is not the author. Source and data are unchanged.
//!
//! These are regression tests for the {2/X} hybrid cost and the Courier's colours.
//!
//! The differential test at the end compares `cost_of` and the card definitions of every card with a golden dump taken
//! on the base commit (a32bfd2a): `ADV_DUMP=<file>` writes the dump, `ADV_GOLDEN=<file>` compares against one.

mod play_common;
use play_common::*;
use sim_core::engine::mana::{can_pay_spell, cost_of, cost_of_with, cost_of_x};
use sim_core::engine::{cast, chars};
use sim_core::ids::CardId;
use sim_core::testkit::{set_commander, table};

const KARN: &str = "Karn, Gilded Guardian";
const REAPER: &str = "The Reaper, King No More";
const PROC: &str = "Spectral Procession";
const COURIER: &str = "Transguild Courier";

fn tbl() -> Game {
    let mut g = table(&["sheoldred", "seph"]);
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    g
}

fn sorted(s: &str) -> String {
    let mut v: Vec<char> = s.chars().collect();
    v.sort();
    v.into_iter().collect()
}

fn basic(col: char) -> &'static str {
    match col {
        'W' => "Plains",
        'U' => "Island",
        'B' => "Swamp",
        'R' => "Mountain",
        'G' => "Forest",
        _ => "Wastes",
    }
}

/// untapped lands: ('W', 2) is two Plains; 'C' is Wastes
fn src(g: &mut Game, spec: &[(char, usize)]) {
    for &(col, n) in spec {
        lands(g, P0, basic(col), n, false);
    }
}

fn price(g: &Game, c: CardId) -> (u32, String) {
    let (gn, pips) = cost_of(g, P0, c);
    (gn, sorted(&pips))
}

fn untapped(g: &Game) -> usize {
    g.player(P0).lands.iter().filter(|&&l| !g.perm(l).tapped).count()
}

fn ai_cast(g: &mut Game, c: CardId) -> bool {
    sim_core::ai::brain::do_cast(g, P0, c, None).unwrap()
}

// =============================================================== 1. real AI cast attempts
#[test]
fn ai_casts_karn_from_hand_with_wub_and_four_colourless() {
    let mut g = tbl();
    src(&mut g, &[('W', 1), ('U', 1), ('B', 1), ('C', 4)]);
    let c = hand(&mut g, P0, &[KARN])[0];
    assert_eq!(price(&g, c), (4, "BUW".to_string()));
    assert!(ai_cast(&mut g, c), "seven lands pay {{2}}{{2}}{{W}}{{U}}{{B}}");
    assert!(has_perm(&g, P0, KARN), "Karn is on the battlefield");
    assert_eq!(untapped(&g), 0, "all seven lands were spent (and not one more mana)");
}

#[test]
fn ai_does_not_cast_karn_with_six_lands_and_taps_nothing() {
    let mut g = tbl();
    src(&mut g, &[('W', 1), ('U', 1), ('B', 1), ('C', 3)]);
    let c = hand(&mut g, P0, &[KARN])[0];
    assert!(!ai_cast(&mut g, c));
    assert!(!has_perm(&g, P0, KARN));
    assert!(g.player(P0).hand.contains(&c), "still in hand");
    assert_eq!(untapped(&g), 6, "nothing tapped by a refused cast");
}

#[test]
fn ai_casts_karn_with_five_colours_and_spares_the_extra_lands() {
    let mut g = tbl();
    src(&mut g, &[('W', 1), ('U', 1), ('B', 1), ('R', 1), ('G', 1), ('C', 2)]);
    let c = hand(&mut g, P0, &[KARN])[0];
    assert_eq!(price(&g, c), (0, sorted("WUBRG")));
    assert!(ai_cast(&mut g, c));
    assert_eq!(untapped(&g), 2, "five mana, not seven");
}

#[test]
fn ai_cast_of_the_commander_from_the_command_zone_twice_pays_tax_after_the_two() {
    let mut g = tbl();
    set_commander(&mut g, P0, KARN);
    let c = card(&g, KARN);
    src(&mut g, &[('W', 1), ('U', 1), ('B', 1), ('C', 4)]);
    assert_eq!(price(&g, c), (4, "BUW".to_string()), "first cast: {{2}} for red and green");
    assert!(ai_cast(&mut g, c));
    assert!(has_perm(&g, P0, KARN));
    assert_eq!(g.player(P0).tax_of(c), 2, "the tax is recorded after the cast");
    // the commander dies and goes back to the command zone; untap, add two lands
    let karns = perms_named(&g, P0, KARN);
    for &m in &karns {
        g.perm_mut(m).on_bf = false;
    }
    g.player_mut(P0).perms.retain(|m| !karns.contains(m));
    g.player_mut(P0).set_cmd_zone(c, true);
    for l in g.player(P0).lands.clone() {
        g.perm_mut(l).tapped = false;
    }
    src(&mut g, &[('C', 2)]);
    assert_eq!(price(&g, c), (6, "BUW".to_string()), "{{2}}{{2}} chosen first, tax {{2}} after: nine mana");
    assert!(ai_cast(&mut g, c), "nine lands");
    assert_eq!(g.player(P0).tax_of(c), 4);
    assert_eq!(untapped(&g), 0);
}

// =============================================================== 2. cost semantics
#[test]
fn the_reaper_prices_each_two_separately() {
    let mut g = tbl();
    let c = hand(&mut g, P0, &[REAPER])[0];
    assert_eq!(price(&g, c), (6, String::new()), "no sources: three {{2}}");
    src(&mut g, &[('B', 1)]);
    assert_eq!(price(&g, c), (4, "B".to_string()));
    src(&mut g, &[('R', 1)]);
    assert_eq!(price(&g, c), (2, "BR".to_string()));
    src(&mut g, &[('G', 1)]);
    assert_eq!(price(&g, c), (0, "BGR".to_string()));
    assert!(can_pay_spell(&g, P0, c, 0, "BGR", false));
}

#[test]
fn the_reaper_is_cast_by_the_ai_with_two_black_and_four_colourless() {
    let mut g = tbl();
    src(&mut g, &[('B', 2), ('C', 3)]);
    let c = hand(&mut g, P0, &[REAPER])[0];
    // {2}{2} for red and green + {B} = 5 mana; five lands
    assert_eq!(price(&g, c), (4, "B".to_string()));
    assert!(ai_cast(&mut g, c));
    assert!(has_perm(&g, P0, REAPER));
}

#[test]
fn three_twos_with_only_w_and_u() {
    let mut g = tbl();
    src(&mut g, &[('W', 1), ('U', 1), ('C', 6)]);
    let c = hand(&mut g, P0, &[KARN])[0];
    assert_eq!(price(&g, c), (6, "UW".to_string()), "{{2}} for B, R and G");
    assert!(can_pay_spell(&g, P0, c, 6, "WU", false));
}

#[test]
fn a_land_that_makes_any_colour_pays_a_pip_not_a_two() {
    let mut g = tbl();
    g.player_mut(P0).ident = sim_core::cards::Colors::from_letters("WUBRG"); // Karn's identity (the table's seat is mono-B)
    src(&mut g, &[('C', 4)]);
    lands(&mut g, P0, "Mana Confluence", 3, false);
    let c = hand(&mut g, P0, &[KARN])[0];
    let (gn, pips) = cost_of(&g, P0, c);
    assert!(can_pay_spell(&g, P0, c, gn, &pips, false), "seven lands, three of them any colour: {gn} {pips}");
    assert_eq!((gn, pips.len()), (4, 3), "{{2}}{{2}} and three coloured pips: seven mana");
}

#[test]
fn two_untapped_confluences_and_a_tapped_land_do_not_double_count() {
    let mut g = tbl();
    let ls = lands(&mut g, P0, "Mana Confluence", 5, false);
    g.perm_mut(ls[0]).tapped = true;
    g.perm_mut(ls[1]).tapped = true;
    let c = hand(&mut g, P0, &[KARN])[0];
    let (gn, pips) = cost_of(&g, P0, c);
    // three untapped lands: nothing is payable; the price is the fallback and must say so
    assert!(!can_pay_spell(&g, P0, c, gn, &pips, false), "{gn} {pips}");
}

#[test]
fn medallions_make_a_two_cheaper_than_a_pip() {
    // Rule 601.2b: the player chooses {2} or the colour for each symbol before the cost is determined. With three generic
    // reducers out, choosing {2} for one colour costs {2} - {3} = nothing, so Karn costs four coloured mana, not five.
    let mut g = tbl();
    for m in ["Jet Medallion", "Sapphire Medallion", "Ruby Medallion"] {
        perm(&mut g, P0, m);
    }
    src(&mut g, &[('W', 1), ('U', 1), ('B', 1), ('R', 1), ('G', 1)]);
    let c = hand(&mut g, P0, &[KARN])[0];
    let (gn, pips) = cost_of(&g, P0, c);
    let total = gn + pips.len() as u32;
    assert!(total <= 4, "paid {total} ({gn} {pips}) where choosing one {{2}} pays 4");
}

#[test]
fn cost_increasers_apply_to_a_chosen_two() {
    // Sphere of Resistance: every spell costs {1} more
    let mut g = tbl();
    perm(&mut g, P0, "Sphere of Resistance");
    src(&mut g, &[('W', 1), ('U', 1), ('B', 1), ('C', 5)]);
    let c = hand(&mut g, P0, &[KARN])[0];
    assert_eq!(price(&g, c), (5, "BUW".to_string()), "{{2}}{{2}}+{{1}}");
    assert!(can_pay_spell(&g, P0, c, 5, "WUB", false));
    // eight lands, not seven: the Sphere makes it eight
    let mut g = tbl();
    perm(&mut g, P0, "Sphere of Resistance");
    src(&mut g, &[('W', 1), ('U', 1), ('B', 1), ('C', 4)]);
    let c = hand(&mut g, P0, &[KARN])[0];
    let (gn, pips) = cost_of(&g, P0, c);
    assert!(!can_pay_spell(&g, P0, c, gn, &pips, false), "seven lands do not pay eight");
}

#[test]
fn thalia_taxes_the_noncreature_procession_but_not_karn() {
    let mut g = tbl();
    perm(&mut g, P1, "Thalia, Guardian of Thraben");
    src(&mut g, &[('W', 3)]);
    let proc = hand(&mut g, P0, &[PROC])[0];
    let karn = hand(&mut g, P0, &[KARN])[0];
    assert_eq!(price(&g, proc), (1, "WWW".to_string()), "noncreature spell: {{1}} more");
    src(&mut g, &[('U', 1), ('B', 1), ('C', 4)]);
    assert_eq!(price(&g, karn), (4, "BUW".to_string()), "Karn is a creature: Thalia does not touch it");
}

#[test]
fn trinisphere_floor_counts_the_two_as_two_symbols() {
    let mut g = tbl();
    perm(&mut g, P1, "Trinisphere");
    src(&mut g, &[('W', 1), ('B', 5)]);
    let proc = hand(&mut g, P0, &[PROC])[0];
    // {2}{2}+{W}: five mana; Trinisphere's minimum of three is already met
    assert_eq!(price(&g, proc), (4, "W".to_string()));
    assert!(can_pay_spell(&g, P0, proc, 4, "W", false));
}

#[test]
fn spectral_procession_with_a_second_white_and_swamps() {
    let mut g = tbl();
    src(&mut g, &[('W', 2), ('B', 3)]);
    let c = hand(&mut g, P0, &[PROC])[0];
    assert_eq!(price(&g, c), (2, "WW".to_string()), "two white, one {{2}}: four mana");
}

#[test]
fn x_added_to_a_card_with_twos() {
    let mut g = tbl();
    src(&mut g, &[('W', 1), ('U', 1), ('B', 1), ('R', 1), ('G', 1), ('C', 3)]);
    let c = hand(&mut g, P0, &[KARN])[0];
    let (gn, pips) = cost_of_x(&g, P0, c, 3);
    assert_eq!((gn, sorted(&pips)), (3, sorted("WUBRG")), "X = 3 added once to the generic part");
    assert_eq!(cost_of_x(&g, P0, c, -4), cost_of_x(&g, P0, c, 0));
    assert_eq!(cost_of_x(&g, P0, c, i32::MAX).0, i32::MAX as u32, "no overflow with the {{2}} choice");
    let (gn, _) = cost_of_x(&g, P0, c, i32::MAX);
    assert_eq!(gn, i32::MAX as u32);
}

#[test]
fn an_extra_pip_in_the_cost_is_not_mistaken_for_the_two_hybrid_colour() {
    // a strive-style extra {G}: Karn's {2/G} and the added {G} are different symbols
    let mut g = tbl();
    src(&mut g, &[('W', 1), ('U', 1), ('B', 1), ('R', 1), ('G', 1), ('C', 3)]);
    let c = hand(&mut g, P0, &[KARN])[0];
    let (gn, pips) = cost_of_with(&g, P0, c, 0, "WUBRGG");
    assert!(can_pay_spell(&g, P0, c, gn, &pips, false), "{gn} {pips}");
    // one green source only: the {2/G} is paid with {2}, the extra {G} with the Forest
    assert_eq!((gn, sorted(&pips)), (2, sorted("WUBRG")), "{{2}} for one G, a real {{G}} for the other");
}

#[test]
fn a_cost_with_the_two_colour_pips_removed_by_the_caller() {
    // `pips` that lack a letter (a pip another effect took out) leave that {2/X} out of the choices
    let mut g = tbl();
    src(&mut g, &[('W', 1)]);
    let c = hand(&mut g, P0, &[REAPER])[0];
    let (gn, pips) = cost_of_with(&g, P0, c, 0, "BR");
    assert_eq!(
        (gn, sorted(&pips)),
        (4, String::new()),
        "only {{2/B}} and {{2/R}} are left; no source for either: {{2}}{{2}}"
    );
}

// =============================================================== 3. the human seat
#[test]
fn the_person_pays_from_a_pool_with_wub_and_four_colourless() {
    let mut g = tbl();
    seat(&mut g, P0, vec![]);
    let c = hand(&mut g, P0, &[KARN])[0];
    pool(&mut g, P0, &[('W', 1), ('U', 1), ('B', 1), ('C', 4)]);
    let h = main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0}))]);
    assert!(!g.player(P0).hand.contains(&c), "{}", told_last(&h));
    assert!(has_perm(&g, P0, KARN));
    assert_eq!(pool_total(&g, P0), 0, "the whole pool paid it");
}

#[test]
fn the_person_with_six_mana_is_refused_and_keeps_the_pool() {
    let mut g = tbl();
    let c = hand(&mut g, P0, &[KARN])[0];
    pool(&mut g, P0, &[('W', 1), ('U', 1), ('B', 1), ('C', 3)]);
    let h = main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0}))]);
    assert!(g.player(P0).hand.contains(&c), "{}", told_last(&h));
    assert_eq!(pool_total(&g, P0), 6, "nothing was spent");
}

#[test]
fn the_person_with_five_colours_in_the_pool_pays_five() {
    let mut g = tbl();
    let c = hand(&mut g, P0, &[KARN])[0];
    pool(&mut g, P0, &[('W', 1), ('U', 1), ('B', 1), ('R', 1), ('G', 1), ('C', 2)]);
    let h = main(&mut g, P0, vec![act(json!({"do": "cast", "card": 0}))]);
    assert!(!g.player(P0).hand.contains(&c), "{}", told_last(&h));
    assert_eq!(pool_total(&g, P0), 2, "five mana, two left");
}

#[test]
fn the_persons_displayed_cost_with_an_empty_pool_follows_the_untapped_lands() {
    let mut g = tbl();
    seat(&mut g, P0, vec![]);
    let c = hand(&mut g, P0, &[KARN])[0];
    src(&mut g, &[('W', 1), ('U', 1), ('B', 1), ('C', 4)]);
    let (gn, pips) = sim_core::human::legal::base_cost(&g, P0, c);
    assert_eq!((gn, sorted(&pips)), (4, "BUW".to_string()), "the displayed cost: {{2}}{{2}}{{W}}{{U}}{{B}}");
}

#[test]
fn the_person_casts_the_commander_from_the_command_zone_with_a_pool() {
    let mut g = tbl();
    set_commander(&mut g, P0, KARN);
    pool(&mut g, P0, &[('W', 1), ('U', 1), ('B', 1), ('C', 4)]);
    let h = main(&mut g, P0, vec![act(json!({"do": "cast", "zone": "cmd"}))]);
    assert!(has_perm(&g, P0, KARN), "{}", told_last(&h));
    assert_eq!(g.player(P0).tax_of(card(&g, KARN)), 2);
}

// =============================================================== 4. colours
#[test]
fn courier_is_five_colours_in_every_zone() {
    let mut g = tbl();
    let id = card(&g, COURIER);
    let all = |g: &Game| ['W', 'U', 'B', 'R', 'G'].iter().all(|&ch| g.db.get(id).colors.has(ch));
    // hand
    let c = hand(&mut g, P0, &[COURIER])[0];
    assert!(all(&g));
    // graveyard, exile, library
    g.player_mut(P0).hand.retain(|&x| x != c);
    g.player_mut(P0).gy.push(c);
    assert!(all(&g));
    g.player_mut(P0).gy.clear();
    g.player_mut(P0).exile.push(c);
    assert!(all(&g));
    g.player_mut(P0).exile.clear();
    // battlefield through the characteristics pass and as a spell on the stack (the colours the stack reads)
    let m = perm(&mut g, P0, COURIER);
    assert_eq!(chars::colors(&g, m).count(), 5);
}

#[test]
fn a_blue_hate_card_sees_a_courier_spell_as_blue() {
    // the Pyroblast / Red Elemental Blast / Force of Will check on a Courier
    let g = tbl();
    let c = card(&g, COURIER);
    assert!(sim_core::engine::stack::blue_card(&g, c));
}

#[test]
fn leyline_and_courier_together_are_still_five_colours() {
    let mut g = tbl();
    perm(&mut g, P0, "Leyline of the Guildpact");
    let m = perm(&mut g, P0, COURIER);
    assert_eq!(chars::colors(&g, m).count(), 5);
}

#[test]
fn metamorph_copy_of_courier_is_five_colours() {
    use sim_core::state::Ctx;
    let mut g = tbl();
    let original = perm(&mut g, P0, COURIER);
    let meta = hand(&mut g, P0, &["Phyrexian Metamorph"])[0];
    cast::cast_card(&mut g, P0, meta, "hand", Ctx::default()).unwrap();
    let copies: Vec<_> = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).on_bf && m != original && &*g.perm(m).name == COURIER)
        .collect();
    assert_eq!(copies.len(), 1, "Phyrexian Metamorph copied the Courier");
    assert_eq!(chars::colors(&g, copies[0]).count(), 5);
}

#[test]
fn goblin_cratermaker_does_not_see_the_courier_as_colourless() {
    // Cratermaker: "{1}, sacrifice: destroy target colorless nonland permanent". The engine's option finds those by
    // `pips.is_empty()` (impls/partials.rs): the Courier has no pips and is every colour.
    let options = |g: &mut Game| {
        let m = perms_named(g, P0, "Goblin Cratermaker")[0];
        let c = g.perm(m).cd.unwrap();
        g.registry.get(c).unwrap().options.unwrap()(g, m, P0, Some(false)).unwrap()
    };
    // control: a really colourless permanent is offered (so the check can see anything at all)
    let mut seen = vec![];
    for name in [
        "Sol Ring",
        "Wurmcoil Engine",
        "Myr Battlesphere",
        "Walking Ballista",
        "Mystic Forge",
        "Grim Monolith",
        "Hangarback Walker",
        "Solemn Simulacrum",
        "Mana Vault",
        "Chromatic Lantern",
        "Kozilek, Butcher of Truth",
        "Ulamog, the Ceaseless Hunger",
    ] {
        let mut g = tbl();
        if g.db.id(name).is_none() {
            continue;
        }
        perm(&mut g, P0, "Goblin Cratermaker");
        src(&mut g, &[('R', 2)]);
        let t = perm(&mut g, P1, name);
        eprintln!("pval({name}) = {}", sim_core::engine::values::pval(&g, t));
        let labels = options(&mut g).iter().map(|o| o.label.clone()).collect::<Vec<_>>();
        eprintln!("control {name}: {labels:?}");
        if !labels.is_empty() {
            seen.push(name);
        }
    }
    assert!(!seen.is_empty(), "no colourless control is ever offered: the Courier check below would prove nothing");

    let mut g = tbl();
    perm(&mut g, P0, "Goblin Cratermaker");
    src(&mut g, &[('R', 2)]);
    let cour = perm(&mut g, P1, COURIER);
    g.perm_mut(cour).plus = 1; // a 4/4: worth the Cratermaker's attention (the option needs a value of 3)
    assert_eq!(chars::colors(&g, cour).count(), 5);
    let pv = sim_core::engine::values::pval(&g, cour);
    assert!(pv >= 3.0, "pval(Courier) = {pv}: the check below would prove nothing");
    let labels: Vec<String> = options(&mut g).iter().map(|o| o.label.clone()).collect();
    assert!(
        !labels.iter().any(|l| l.contains(COURIER)),
        "Cratermaker offers to destroy the five-colour Courier as a colorless permanent: {labels:?}"
    );
}

#[test]
fn karns_own_devotion_and_colours() {
    let mut g = tbl();
    let m = perm(&mut g, P0, KARN);
    assert_eq!(chars::colors(&g, m).count(), 5);
    let d = g.db.get(card(&g, KARN));
    assert_eq!(d.devotion_to("WUBRG"), 5);
    assert_eq!(d.devotion_to("B"), 1);
    let r = g.db.get(card(&g, REAPER));
    assert_eq!(r.devotion_to("BR"), 2);
    assert_eq!(r.devotion_to("W"), 0);
    let s = g.db.get(card(&g, PROC));
    assert_eq!(s.devotion_to("W"), 3);
    assert_eq!(sim_core::impls::rules2::devotion(&g, P0, 'U'), 1, "Karn on the battlefield counts once for blue");
}

// =============================================================== 5. data
#[test]
fn every_two_hybrid_has_its_colour_in_pips_and_the_mana_value_adds_up() {
    let g = tbl();
    for d in g.db.iter() {
        let twos = d.hybrid.chars().collect::<Vec<_>>();
        let twos: Vec<char> = twos.chunks_exact(2).filter(|w| w[1] == '2').map(|w| w[0]).collect();
        let mut rest = d.pips.to_string();
        for l in &twos {
            let i = rest.find(*l).unwrap_or_else(|| panic!("{}: {{2/{l}}} has no pip", d.name));
            rest.remove(i);
        }
        let printed_pairs = d.hybrid.len() / 2;
        assert_eq!(d.hybrid.len() % 2, 0, "{}: hybrid is whole pairs", d.name);
        assert_eq!(d.cmc as usize, d.generic as usize + d.pips.chars().count() + twos.len(), "{}", d.name);
        let _ = printed_pairs;
    }
}

#[test]
fn the_cache_holds_exactly_four_cards_with_a_two_hybrid_symbol() {
    let path = sim_core::testkit::data_dir().join("scryfall_cache.json");
    let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let mut names = vec![];
    for (_, r) in v.as_object().unwrap() {
        let mut costs = vec![r.get("mana_cost").and_then(|x| x.as_str()).unwrap_or("").to_string()];
        if let Some(fs) = r.get("card_faces").and_then(|f| f.as_array()) {
            costs.extend(fs.iter().map(|f| f.get("mana_cost").and_then(|x| x.as_str()).unwrap_or("").to_string()));
        }
        if costs.iter().any(|c| ["{2/W}", "{2/U}", "{2/B}", "{2/R}", "{2/G}"].iter().any(|s| c.contains(s))) {
            names.push(r.get("name").and_then(|x| x.as_str()).unwrap_or("?").to_string());
        }
    }
    names.sort();
    assert_eq!(
        names,
        ["Karn, Gilded Guardian", "Spectral Procession", "The Reaper, King No More", "Wildgrowth Archaic"]
    );
    let g = tbl();
    let w = g.db.get(card(&g, "Wildgrowth Archaic"));
    assert_eq!((w.generic, &*w.pips, w.cmc, &*w.hybrid), (0, "GG", 2, ""), "its override states {{G}}{{G}}");
}

#[test]
fn spectral_procession_changed_only_where_it_should() {
    let g = tbl();
    let d = g.db.get(card(&g, PROC));
    assert_eq!((d.generic, &*d.pips, d.cmc), (0, "WWW", 6));
    assert_eq!(d.colors.count(), 1);
    assert!(d.colors.has('W'), "it was colourless in the engine before: now white");
    assert!(d.sorcery && !d.creature);
}

// =============================================================== 6. speed
#[test]
fn pricing_an_unaffordable_karn_stays_cheap() {
    let mut g = tbl();
    src(&mut g, &[('W', 1), ('U', 1)]);
    for m in ["Jet Medallion", "Sapphire Medallion", "Sphere of Resistance"] {
        perm(&mut g, P0, m);
    }
    let c = hand(&mut g, P0, &[KARN])[0];
    let t = std::time::Instant::now();
    let n = 2000;
    for _ in 0..n {
        std::hint::black_box(cost_of(&g, P0, c));
    }
    let per = t.elapsed() / n;
    eprintln!("cost_of(Karn), unaffordable, three cost hooks: {per:?} per call (32 masks)");
    let t = std::time::Instant::now();
    let sol = hand(&mut g, P0, &["Kitchen Finks"])[0];
    for _ in 0..n {
        std::hint::black_box(cost_of(&g, P0, sol));
    }
    eprintln!("cost_of(Kitchen Finks): {:?} per call", t.elapsed() / n);
    assert!(per < std::time::Duration::from_millis(2), "{per:?} per call");
}

// =============================================================== 7. differential against the base commit
fn dump() -> String {
    use std::fmt::Write;
    let mut out = String::new();
    let boards: [Option<&str>; 6] = [
        None,
        Some("Jet Medallion"),
        Some("Sphere of Resistance"),
        Some("Trinisphere"),
        Some("K'rrik, Son of Yawgmoth"),
        Some("Defense Grid"),
    ];
    let lands: [(&str, Vec<(char, usize)>); 2] =
        [("none", vec![]), ("rich", vec![('W', 3), ('U', 3), ('B', 3), ('R', 3), ('G', 3), ('C', 3)])];
    for (bi, board) in boards.iter().enumerate() {
        for (lname, spec) in &lands {
            let mut g = tbl();
            if let Some(b) = board {
                perm(&mut g, P0, b);
            }
            src(&mut g, spec);
            for d in g.db.iter() {
                let (gn, pips) = cost_of(&g, P0, d.id);
                let _ = writeln!(out, "C|{}|{bi}|{lname}|{gn}|{}", d.name, sorted(&pips));
            }
        }
    }
    let g = tbl();
    for d in g.db.iter() {
        let cols: String = ['W', 'U', 'B', 'R', 'G'].iter().filter(|&&c| d.colors.has(c)).collect();
        let _ = writeln!(
            out,
            "S|{}|{}|{}|{}|{}|{}|{}|{}",
            d.name,
            d.cmc,
            d.generic,
            d.pips,
            d.hybrid,
            cols,
            d.devotion_to("WUBRG"),
            d.phyrexian
        );
    }
    out
}

#[test]
fn differential_against_the_base_commit() {
    if let Ok(path) = std::env::var("ADV_DUMP") {
        std::fs::write(path, dump()).unwrap();
        return;
    }
    let Ok(path) = std::env::var("ADV_GOLDEN") else {
        eprintln!("ADV_GOLDEN not set: differential skipped");
        return;
    };
    let golden = std::fs::read_to_string(path).unwrap();
    let now = dump();
    let key = |l: &str| {
        let f: Vec<&str> = l.split('|').collect();
        match f[0] {
            "C" => format!("C|{}|{}|{}", f[1], f[2], f[3]),
            _ => format!("S|{}", f[1]),
        }
    };
    let gold: std::collections::HashMap<String, &str> = golden.lines().map(|l| (key(l), l)).collect();
    let allowed = [PROC, COURIER];
    let mut diffs = vec![];
    let mut compared = 0;
    for l in now.lines() {
        let Some(old) = gold.get(&key(l)) else { continue };
        compared += 1;
        if *old != l && !allowed.iter().any(|a| l.split('|').nth(1) == Some(a)) {
            diffs.push(format!("- {old}\n+ {l}"));
        }
    }
    assert!(compared > 20_000, "compared only {compared} lines");
    assert!(
        diffs.is_empty(),
        "{} lines differ from the base commit outside {allowed:?}:\n{}",
        diffs.len(),
        diffs.join("\n")
    );
    // and the allowed ones differ exactly as intended
    for l in now.lines().filter(|l| l.contains("|Spectral Procession|") || l.contains("|Transguild Courier|")) {
        eprintln!("changed: {} -> {l}", gold.get(&key(l)).copied().unwrap_or("(no such card in the base)"));
    }
}
