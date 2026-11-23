//! Adversarial review of 45df040c. Tests intentionally retain assertions exposing real bugs.
mod play_common;

use sim_core::ai::brain::{self, Situation};
use sim_core::ai::{act, pool};
use sim_core::engine::pool::Kind;
use sim_core::engine::{cast, mana};
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game};
use sim_core::testkit::*;

const P: PlayerId = PlayerId(0);
const Q: PlayerId = PlayerId(1);

fn game() -> Game {
    play_common::table(&["yshtola", "kefka", "sauron"])
}

// Matching Medallions for each colour; Jet is also tested on nonblack spells (no discount).
fn setup(g: &mut Game, medallion: &str, mode: usize) {
    for _ in 0..match mode {
        1 | 4 | 7 => 1,
        2 => 2,
        _ => 0,
    } {
        perm(g, P, medallion);
    }
    if matches!(mode, 3 | 4) {
        perm(g, Q, "Sphere of Resistance");
    }
    if mode == 5 {
        perm(g, Q, "Trinisphere");
    }
    if mode >= 6 {
        perm(g, P, "K'rrik, Son of Yawgmoth");
    }
}

fn opts(g: &mut Game, c: CardId, post: Option<bool>) -> Vec<Opt> {
    g.registry.get(c).unwrap().hand_options.unwrap()(g, c, P, post).unwrap()
}

fn run(g: &mut Game, o: &Opt) -> bool {
    act::perform(g, P, o.act.as_ref().unwrap()).unwrap()
}

fn numbers(label: &str) -> Vec<i32> {
    label.split(|c: char| !c.is_ascii_digit()).filter(|s| !s.is_empty()).map(|s| s.parse().unwrap()).collect()
}

fn tapped(g: &Game) -> u32 {
    g.player(P).lands.iter().filter(|&&m| g.perm(m).tapped).count() as u32
}

fn basics(g: &mut Game, pips: &str, n: usize) {
    let cols: Vec<_> = if pips.is_empty() { vec!['C'] } else { pips.chars().collect() };
    for i in 0..n {
        let name = match cols[i % cols.len()] {
            'W' => "Plains",
            'U' => "Island",
            'B' => "Swamp",
            'R' => "Mountain",
            'G' => "Forest",
            _ => "Wastes",
        };
        lands(g, P, name, 1, false);
    }
}

// Brute-force the documented raw-increment contract, independently of max_x's upper bound.
fn brute(g: &Game, c: CardId, per: i32) -> i32 {
    (0..=64)
        .rev()
        .find(|&x| {
            let (gn, pips) = mana::cost_of_x(g, P, c, per * x);
            mana::can_pay_spell(g, P, c, gn, &pips, false)
        })
        .unwrap_or(0)
}

#[test]
fn every_fixed_cost_search_matches_exhaustive_small_pools() {
    let mut failures = vec![];
    for (name, med, per) in [
        ("Exsanguinate", "Jet Medallion", 1),
        ("Debt to the Deathless", "Jet Medallion", 1),
        ("Torment of Hailfire", "Jet Medallion", 1),
        ("Secure the Wastes", "Pearl Medallion", 1),
        ("Mind Twist", "Jet Medallion", 1),
        ("The Meathook Massacre", "Jet Medallion", 1),
        ("Crackle with Power", "Ruby Medallion", 3),
        ("Curse of the Swine", "Sapphire Medallion", 1),
        ("Epic Experiment", "Sapphire Medallion", 1),
        ("Finale of Promise", "Ruby Medallion", 1),
        ("Finale of Revelation", "Sapphire Medallion", 1),
        ("Shatterskull Smashing", "Ruby Medallion", 1),
        ("West Coast Expansion", "Sapphire Medallion", 1),
        ("Astral Cornucopia", "Etherium Sculptor", 3),
        ("Comet Storm", "Ruby Medallion", 1),
        ("Wild Hypothesis", "Emerald Medallion", 1),
        ("Wan Shi Tong, Librarian", "Sapphire Medallion", 1),
        ("Braingeyser", "Sapphire Medallion", 1),
        ("Heliod's Intervention", "Pearl Medallion", 1),
        ("Hydroid Krasis", "Sapphire Medallion", 1),
        ("Primal Might", "Emerald Medallion", 1),
        ("White Sun's Twilight", "Pearl Medallion", 1),
        ("Stargaze", "Jet Medallion", 1),
        ("Disintegrate", "Ruby Medallion", 1),
        ("Everflowing Chalice", "Etherium Sculptor", 2),
        ("Nature's Rhythm", "Emerald Medallion", 1),
        ("Mockingbird", "Sapphire Medallion", 1),
        ("Agadeem's Awakening // Agadeem, the Undercrypt", "Jet Medallion", 1),
    ] {
        for n in 0..=10 {
            for mode in 0..8 {
                let mut g = game();
                setup(&mut g, med, mode);
                let c = card(&g, name);
                let pips = g.db.get(c).pips.to_string();
                basics(&mut g, &pips, n);
                let expected = brute(&g, c, per);
                if mode == 0 {
                    let fixed = g.db.get(c).generic as i32 + g.db.get(c).pips.len() as i32;
                    assert_eq!(expected, (n as i32 - fixed).max(0) / per, "old all-spare-mana rule: {name}/{n}");
                }
                let got = brain::max_x(&g, P, c, per as u32, false);
                if expected != got {
                    failures.push(format!("{name}, lands={n}, mode={mode}: expected X={expected}, got {got}"));
                }
            }
        }
    }
    // BUG: max_x samples its linear bound at total mana, where reducers can still be clamped away.
    assert!(failures.is_empty(), "{} mismatches:\n{}", failures.len(), failures.join("\n"));
}

fn hand_matrix(name: &str, med: &str, post: Option<bool>, cap: i32) {
    let mut exercised = 0;
    for n in 1..=10 {
        for mode in 0..8 {
            let mut g = game();
            setup(&mut g, med, mode);
            g.player_mut(P).turns = 8;
            let filler = card(&g, "Swamp");
            g.player_mut(P).library = vec![filler; 100]; // Epic Experiment must not introduce unrelated free spells.
            let c = hand(&mut g, P, &[name])[0];
            let pips = g.db.get(c).pips.to_string();
            basics(&mut g, &pips, n);
            let available = brute(&g, c, 1);
            for o in opts(&mut g, c, post) {
                let x = numbers(&o.label)[0];
                assert_eq!(x, available.min(cap), "{name}/{n}/{mode}: {}", o.label);
                let (gn, paid_pips) = mana::cost_of_x(&g, P, c, x);
                let spent = gn + paid_pips.len() as u32;
                let life = mana::phyrexian_life(&g, P, c, &paid_pips);
                let life_effect = match name {
                    "Exsanguinate" => 2 * x,
                    "Debt to the Deathless" => 4 * x,
                    "Stargaze" => -x,
                    "White Sun's Twilight" => x,
                    _ => 0,
                };
                assert!(run(&mut g, &o), "offered an unpayable option: {name}/{n}/{mode}");
                let untapped = if name == "Finale of Revelation" && x >= 10 { 5 } else { 0 };
                assert_eq!(tapped(&g), spent - untapped, "wrong payment: {name}/{n}/{mode}, X={x}");
                assert_eq!(g.player(P).life, 40 - life + life_effect, "life: {name}/{n}/{mode}");
                exercised += 1;
                break; // these cards expose one X plan; do not execute a second stale plan
            }
        }
    }
    assert!(exercised >= 8, "each scenario must execute at least one real callback: {name}");
}

#[test]
fn exsanguinate_payment_matrix() {
    hand_matrix("Exsanguinate", "Jet Medallion", Some(false), 64);
}
#[test]
fn debt_payment_matrix() {
    hand_matrix("Debt to the Deathless", "Jet Medallion", Some(false), 64);
}
#[test]
fn torment_payment_matrix() {
    hand_matrix("Torment of Hailfire", "Jet Medallion", Some(false), 64);
}
#[test]
fn west_coast_payment_matrix() {
    hand_matrix("West Coast Expansion", "Sapphire Medallion", Some(false), 64);
}
#[test]
fn hypothesis_payment_matrix() {
    hand_matrix("Wild Hypothesis", "Emerald Medallion", Some(false), 64);
}
#[test]
fn wan_payment_matrix() {
    hand_matrix("Wan Shi Tong, Librarian", "Sapphire Medallion", None, 64);
}
#[test]
fn revelation_payment_matrix() {
    hand_matrix("Finale of Revelation", "Sapphire Medallion", Some(true), 64);
}
#[test]
fn experiment_payment_matrix() {
    hand_matrix("Epic Experiment", "Sapphire Medallion", Some(false), 7);
}
#[test]
fn stargaze_payment_matrix() {
    hand_matrix("Stargaze", "Jet Medallion", Some(false), 5);
}
#[test]
fn twilight_payment_matrix() {
    hand_matrix("White Sun's Twilight", "Pearl Medallion", Some(false), 4);
}

#[test]
fn tokx_and_unplanned_xtutor_use_all_spare_mana_and_charge_life_once() {
    let mut failures = vec![];
    for name in ["Secure the Wastes", "Mind Twist"] {
        for n in 0..=10 {
            for mode in 0..8 {
                let mut g = game();
                let black = name == "Mind Twist";
                setup(&mut g, if black { "Jet Medallion" } else { "Pearl Medallion" }, mode);
                hand(&mut g, Q, &["Island"; 20]);
                let c = hand(&mut g, P, &[name])[0];
                basics(&mut g, if black { "B" } else { "W" }, n);
                let x = brute(&g, c, 1);
                let (gn, pips) = mana::cost_of_x(&g, P, c, x);
                let payable = mana::can_pay_spell(&g, P, c, gn, &pips, false);
                let life = mana::phyrexian_life(&g, P, c, &pips);
                let before = g.player(P).perms.len();
                assert_eq!(brain::do_cast(&mut g, P, c, None).unwrap(), payable, "{name}/{n}/{mode}");
                assert_eq!(tapped(&g), if payable { gn + pips.len() as u32 } else { 0 });
                assert_eq!(g.player(P).life, 40 - if payable { life } else { 0 });
                if payable {
                    let actual = if black { 20 - g.player(Q).hand.len() } else { g.player(P).perms.len() - before };
                    if actual != x as usize {
                        failures.push(format!("{name}/{n}/{mode}: expected X={x}, got {actual}"));
                    }
                    if mode == 0 {
                        assert_eq!(x, (n as i32 - 1).max(0));
                    }
                }
            }
        }
    }
    // BUG: with zero lands, Jet and K'rrik, the search misses a payable X=1 Mind Twist.
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn planned_xtutor_pays_exactly_the_requested_x_or_changes_nothing() {
    for n in 0..=10 {
        for mode in 0..8 {
            for x in 0..=10 {
                let mut g = game();
                setup(&mut g, "Jet Medallion", mode);
                hand(&mut g, Q, &["Island"; 20]);
                let c = hand(&mut g, P, &["Mind Twist"])[0];
                basics(&mut g, "B", n);
                g.player_mut(P).plan_x = Some(x as u32);
                let (gn, pips) = mana::cost_of_x(&g, P, c, x);
                let payable = mana::can_pay_spell(&g, P, c, gn, &pips, false);
                let life = mana::phyrexian_life(&g, P, c, &pips);
                assert_eq!(brain::do_cast(&mut g, P, c, None).unwrap(), payable, "{n}/{mode}/{x}");
                assert_eq!(tapped(&g), if payable { gn + pips.len() as u32 } else { 0 });
                assert_eq!(g.player(P).life, 40 - if payable { life } else { 0 });
                assert_eq!(20 - g.player(Q).hand.len(), if payable { x as usize } else { 0 });
                assert_eq!(g.player(P).hand.contains(&c), !payable);
            }
        }
    }
}

#[test]
fn cornucopia_and_chalice_multikicker_pay_the_symbol_count_once() {
    let mut failures = vec![];
    for (name, per, key, cap) in
        [("Astral Cornucopia", 3, DataKey::Charge, 64), ("Everflowing Chalice", 2, DataKey::Kicks, 3)]
    {
        for n in 0..=10 {
            for mode in 0..8 {
                let mut g = game();
                setup(&mut g, "Etherium Sculptor", mode);
                basics(&mut g, "", n);
                let c = hand(&mut g, P, &[name])[0];
                let x = brute(&g, c, per).min(cap);
                let oo = opts(&mut g, c, Some(false));
                if oo.len() != usize::from(x > 0) {
                    failures.push(format!("{name}/{n}/{mode}: expected X={x}, got {} options", oo.len()));
                    continue;
                }
                if x == 0 {
                    continue;
                }
                let (gn, pips) = mana::cost_of_x(&g, P, c, per * x);
                assert!(run(&mut g, &oo[0]));
                assert_eq!(tapped(&g), gn + pips.len() as u32, "{name}/{n}/{mode}");
                let m = g.player(P).perms.iter().copied().find(|&m| g.perm(m).cd == Some(c)).unwrap();
                assert_eq!(g.perm(m).data.int(key), x as i64);
            }
        }
    }
    // BUG: two Sculptors make one kick free, but max_x returns zero with an empty mana pool.
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn graveyard_yawgmoths_will_cast_pays_krrik_life() {
    for med in [false, true] {
        let mut g = game();
        perm(&mut g, P, "K'rrik, Son of Yawgmoth");
        if med {
            perm(&mut g, P, "Jet Medallion");
        }
        basics(&mut g, "B", 3);
        hand(&mut g, Q, &["Island"; 20]);
        let c = card(&g, "Mind Twist");
        g.player_mut(P).gy.push(c);
        g.player_mut(P).yawg = true;
        assert!(brain::do_cast(&mut g, P, c, Some("yawg")).unwrap());
        assert_eq!(tapped(&g), 3);
        assert_eq!(20 - g.player(Q).hand.len(), if med { 4 } else { 3 });
        // BUG: do_cast_with drops K'rrik's B from yawg costs but charges life only for hand/cmd zones.
        assert_eq!(g.player(P).life, 38, "K'rrik applies to spells cast from every zone");
    }
}

#[test]
fn practice_braingeyser_copy_keeps_the_chosen_x_and_exact_payment() {
    for n in 0..=10 {
        for x in 1..=10 {
            let mut g = game();
            play_common::seat(&mut g, P, vec![]);
            play_common::pool(&mut g, P, &[('U', n as u32)]);
            let c = card(&g, "Braingeyser");
            let before = g.player(Q).hand.len();
            let ok = sim_core::impls::garth::cast_copy_at(&mut g, P, c, None, Some(Q), x).unwrap();
            assert_eq!(ok, n >= x as usize + 2);
            assert_eq!(play_common::pool_total(&g, P), n as u32 - if ok { x + 2 } else { 0 });
            assert_eq!(g.player(Q).hand.len() - before, if ok { x as usize } else { 0 });
            assert!(!g.player(P).gy.contains(&c));
        }
    }
}

#[test]
fn stack_copy_preserves_x_without_paying_again_or_counting_as_a_cast() {
    let mut g = game();
    basics(&mut g, "W", 4);
    let c = hand(&mut g, P, &["Secure the Wastes"])[0];
    assert!(brain::do_cast(&mut g, P, c, None).unwrap());
    let casts = g.player(P).spells_this_turn;
    cast::copy_spell(&mut g, P, c, Some(&Ctx { x: 3, ..Ctx::default() }), false).unwrap();
    assert_eq!(g.player(P).perms.len(), 6);
    assert_eq!(g.player(P).spells_this_turn, casts);
    assert_eq!(tapped(&g), 4);
}

#[test]
fn wrong_colours_do_not_pay_for_x_spells() {
    for name in ["Secure the Wastes", "Mind Twist", "The Meathook Massacre"] {
        let mut g = game();
        basics(&mut g, "R", 10);
        let c = hand(&mut g, P, &[name])[0];
        assert!(!brain::do_cast(&mut g, P, c, None).unwrap());
        assert_eq!(tapped(&g), 0);
        assert!(g.player(P).hand.contains(&c));
    }
}

#[test]
fn huge_pool_search_respects_the_engines_bounded_mana_units() {
    let mut g = game();
    g.player_mut(P).mana.add(Kind::C, 1_000_000_000);
    let c = card(&g, "Astral Cornucopia");
    // units() deliberately caps a floating entry at 1000, regardless of the billions it contains.
    assert_eq!(mana::total_mana_spell(&g, P, c, false), 1000);
    assert_eq!(brain::max_x(&g, P, c, 3, false), 333);
    assert!(brain::pay_x(&mut g, P, c, 333, 3, false).unwrap());
    assert_eq!(g.player(P).mana.total(), 1_000_000_000 - 999);
}

#[test]
fn huge_chosen_x_payment_is_rejected_without_overflow_or_spending() {
    let mut g = game();
    g.player_mut(P).mana.add(Kind::C, 10);
    let c = card(&g, "Astral Cornucopia");
    // BUG: can_pay_x/pay_x multiply caller-supplied X by per in i32 instead of checking the product.
    assert!(!brain::can_pay_x(&g, P, c, i32::MAX, 3, false));
    assert!(!brain::pay_x(&mut g, P, c, i32::MAX, 3, false).unwrap());
    assert_eq!(g.player(P).mana.total(), 10);
}

#[test]
fn huge_chosen_x_pay_helper_rejects_without_overflow() {
    let mut g = game();
    g.player_mut(P).mana.add(Kind::C, 10);
    let c = card(&g, "Astral Cornucopia");
    // BUG: pay_x independently has the same unchecked i32 multiplication as can_pay_x.
    assert!(!brain::pay_x(&mut g, P, c, i32::MAX, 3, false).unwrap());
    assert_eq!(g.player(P).mana.total(), 10);
}

#[test]
fn meathook_cast_hook_charges_the_selected_x_and_life_once() {
    let mut failures = vec![];
    for n in 0..=10 {
        for mode in 0..8 {
            let mut g = game();
            setup(&mut g, "Jet Medallion", mode);
            basics(&mut g, "B", n);
            for t in 1..=12 {
                token(&mut g, Q, t);
            }
            let c = hand(&mut g, P, &["The Meathook Massacre"])[0];
            let x = brute(&g, c, 1);
            let (gn, pips) = mana::cost_of_x(&g, P, c, x);
            let life = mana::phyrexian_life(&g, P, c, &pips);
            let payable = mana::can_pay_spell(&g, P, c, gn, &pips, false);
            assert_eq!(brain::do_cast(&mut g, P, c, None).unwrap(), payable);
            if payable {
                let m = g.player(P).perms.iter().copied().find(|&m| g.perm(m).cd == Some(c)).unwrap();
                if g.perm(m).data.int(DataKey::CastX) != x as i64 {
                    failures.push(format!("{n}/{mode}: expected X={x}, got {}", g.perm(m).data.int(DataKey::CastX)));
                    continue;
                }
                assert_eq!(tapped(&g), gn + pips.len() as u32);
                assert_eq!(g.player(P).life, 40 - life + x);
            }
        }
    }
    // BUG: Jet plus K'rrik makes X=1 payable with no mana, but the cast hook receives zero.
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn crackle_three_symbols_and_comet_multikicker_payment_matrix() {
    for name in ["Crackle with Power", "Comet Storm"] {
        for n in 1..=10 {
            for mode in 0..8 {
                let mut g = game();
                setup(&mut g, "Ruby Medallion", mode);
                basics(&mut g, "R", n);
                g.player_mut(P).turns = 8;
                g.player_mut(Q).life = 1;
                let c = hand(&mut g, P, &[name])[0];
                let oo = if name == "Crackle with Power" {
                    let s = Situation::new(&g, P);
                    brain::extra_options(&mut g, P, &s, false, true)
                } else {
                    opts(&mut g, c, Some(false))
                };
                for o in oo.iter().filter(|o| o.label.starts_with(name)) {
                    let ns = numbers(&o.label);
                    let raw = if name == "Crackle with Power" { 3 * ns[0] } else { ns[0] + ns[1] };
                    let (gn, pips) = mana::cost_of_x(&g, P, c, raw);
                    assert!(run(&mut g, o), "{name}/{n}/{mode}");
                    assert_eq!(tapped(&g), gn + pips.len() as u32, "{name}/{n}/{mode}");
                    assert!(!g.player(Q).alive);
                    break;
                }
            }
        }
    }
}

#[test]
fn target_limited_swine_and_intervention_pay_only_the_chosen_targets() {
    for (name, med, pips) in
        [("Curse of the Swine", "Sapphire Medallion", "UU"), ("Heliod's Intervention", "Pearl Medallion", "WW")]
    {
        for n in 1..=10 {
            for mode in 0..8 {
                let mut g = game();
                setup(&mut g, med, mode);
                basics(&mut g, pips, n);
                if name == "Curse of the Swine" {
                    for _ in 0..4 {
                        perm(&mut g, Q, "Shivan Dragon");
                    }
                } else {
                    for _ in 0..4 {
                        perm(&mut g, Q, "Basalt Monolith");
                    }
                }
                let c = hand(&mut g, P, &[name])[0];
                let targets = 4 + i32::from(name == "Heliod's Intervention" && matches!(mode, 3..=5));
                let expected = brute(&g, c, 1).min(targets);
                let oo = opts(&mut g, c, Some(false));
                if expected >= 2 {
                    assert!(!oo.is_empty(), "{name}/{n}/{mode}");
                }
                for o in oo.iter().take(1) {
                    assert_eq!(numbers(&o.label)[0], expected);
                    let (gn, pips) = mana::cost_of_x(&g, P, c, expected);
                    assert!(run(&mut g, o));
                    assert_eq!(tapped(&g), gn + pips.len() as u32, "{name}/{n}/{mode}");
                }
            }
        }
    }
}

#[test]
fn selected_x_for_smashing_disintegrate_mockingbird_and_agadeem_is_paid_once() {
    for (name, med, pips, want) in [
        ("Shatterskull Smashing", "Ruby Medallion", "RR", 5),
        ("Disintegrate", "Ruby Medallion", "R", 5),
        ("Mockingbird", "Sapphire Medallion", "U", 5),
        ("Agadeem's Awakening // Agadeem, the Undercrypt", "Jet Medallion", "BBB", 6),
    ] {
        for n in 1..=10 {
            for mode in 0..8 {
                let mut g = game();
                setup(&mut g, med, mode);
                basics(&mut g, pips, n);
                let c = hand(&mut g, P, &[name])[0];
                if name.starts_with("Agadeem") {
                    let t = card(&g, "Grave Titan");
                    g.player_mut(P).gy.push(t);
                } else if name == "Mockingbird" {
                    perm(&mut g, Q, "Grave Titan");
                } else {
                    perm(&mut g, Q, "Shivan Dragon");
                }
                // Mockingbird copies a creature whose mana value is at most the mana spent: its X is the smallest one whose
                // whole cost (generic and pips) reaches the mana value 6 of the Grave Titan, whatever reducer is out
                let want = if name == "Mockingbird" {
                    (0..=20)
                        .find(|&x| {
                            let (gn, pp) = mana::cost_of_x(&g, P, c, x);
                            gn + pp.chars().count() as u32 >= 6
                        })
                        .unwrap()
                } else {
                    want
                };
                let (gn, paid_pips) = mana::cost_of_x(&g, P, c, want);
                let payable = mana::can_pay_spell(&g, P, c, gn, &paid_pips, false);
                let life = mana::phyrexian_life(&g, P, c, &paid_pips);
                let oo = opts(&mut g, c, Some(false));
                let o = oo.iter().find(|o| o.label.contains("X") || o.label.starts_with("Disintegrate"));
                assert_eq!(
                    o.is_some(),
                    payable,
                    "{name}/{n}/{mode}: {:?}",
                    oo.iter().map(|o| &o.label).collect::<Vec<_>>()
                );
                if let Some(o) = o {
                    assert!(run(&mut g, o), "{name}/{n}/{mode}");
                    assert_eq!(tapped(&g), gn + paid_pips.len() as u32, "{name}/{n}/{mode}");
                    assert_eq!(g.player(P).life, 40 - life);
                }
            }
        }
    }
}

#[test]
fn krasis_and_primal_might_pool_paths_use_all_spare_mana() {
    for (name, med, pips) in
        [("Hydroid Krasis", "Sapphire Medallion", "GU"), ("Primal Might", "Emerald Medallion", "G")]
    {
        for n in 1..=10 {
            for mode in 0..8 {
                let mut g = game();
                setup(&mut g, med, mode);
                basics(&mut g, pips, n);
                if name == "Primal Might" {
                    token(&mut g, P, 1);
                    perm(&mut g, Q, "Shivan Dragon");
                }
                let c = hand(&mut g, P, &[name])[0];
                let x = brute(&g, c, 1);
                let s = Situation::new(&g, P);
                let oo = pool::spell_options(&g, P, &s, Some(false));
                let o = oo.iter().find(|o| o.label.starts_with(name));
                if x >= 4 {
                    assert!(o.is_some(), "{name}/{n}/{mode}");
                }
                if let Some(o) = o {
                    let (gn, pips) = mana::cost_of_x(&g, P, c, x);
                    assert!(run(&mut g, o));
                    assert_eq!(tapped(&g), gn + pips.len() as u32);
                    if name == "Hydroid Krasis" {
                        // The separate body test isolates its pre-existing 0/0 entry problem; still check X here.
                        assert_eq!(g.player(P).hand.len(), (x / 2) as usize);
                        assert_eq!(g.player(P).life, 40 + x / 2);
                    }
                }
            }
        }
    }
}

#[test]
fn finale_of_promise_pays_its_selected_four_not_every_spare_mana() {
    for n in 1..=10 {
        for mode in 0..8 {
            let mut g = game();
            setup(&mut g, "Ruby Medallion", mode);
            basics(&mut g, "R", n);
            let c = hand(&mut g, P, &["Finale of Promise"])[0];
            let target = card(&g, "Fact or Fiction");
            g.player_mut(P).gy.push(target);
            let (gn, pips) = mana::cost_of_x(&g, P, c, 4);
            let payable = mana::can_pay_spell(&g, P, c, gn, &pips, false);
            let oo = opts(&mut g, c, Some(false));
            assert_eq!(!oo.is_empty(), payable, "{n}/{mode}");
            if payable {
                assert_eq!(numbers(&oo[0].label)[0], 4);
                assert!(run(&mut g, &oo[0]));
                assert_eq!(tapped(&g), gn + pips.len() as u32);
            }
        }
    }
}

#[test]
fn harmonize_keeps_four_green_pips_under_trinisphere() {
    let mut g = game();
    basics(&mut g, "G", 4);
    perm(&mut g, Q, "Trinisphere");
    token(&mut g, P, 6);
    let c = card(&g, "Nature's Rhythm");
    g.player_mut(P).gy.push(c);
    let target = card(&g, "Consecrated Sphinx");
    g.player_mut(P).library = vec![target];
    let f = g.registry.get(c).unwrap().gy_options.unwrap();
    let oo = f(&mut g, c, P, Some(false)).unwrap();
    // BUG: harmonize uses Trinisphere's floor calculated with printed GG, then substitutes GGGG: costs five, not four.
    assert_eq!(oo.len(), 1, "{{X}}{{G}}{{G}}{{G}}{{G}}, X=6 reduced by six power: four mana already meets Trinisphere");
    assert!(run(&mut g, &oo[0]));
    assert_eq!(tapped(&g), 4);
    assert!(g.player(P).perms.iter().any(|&m| g.perm(m).cd == Some(target)));
}

#[test]
fn harmonize_normal_reducers_taxes_and_wrong_colours() {
    for n in 0..=10 {
        for mode in 0..8 {
            if mode == 5 {
                continue;
            } // the Trinisphere failure is isolated above
            let mut g = game();
            setup(&mut g, "Emerald Medallion", mode);
            basics(&mut g, "G", n);
            token(&mut g, P, 6);
            let c = card(&g, "Nature's Rhythm");
            g.player_mut(P).gy.push(c);
            let oo = g.registry.get(c).unwrap().gy_options.unwrap()(&mut g, c, P, Some(false)).unwrap();
            let payable =
                (0..=64).any(|x| mana::can_pay_spell(&g, P, c, mana::cost_of_x(&g, P, c, x).0, "GGGG", false));
            assert_eq!(!oo.is_empty(), payable, "{n}/{mode}");
            if payable {
                assert!(run(&mut g, &oo[0]));
                assert_eq!(tapped(&g), n as u32);
            }
        }
    }
}

#[test]
fn flashback_keeps_its_existing_fixed_cost_instead_of_using_printed_cost() {
    for n in 0..=10 {
        for reducer in [false, true] {
            let mut g = game();
            if reducer {
                perm(&mut g, P, "Sapphire Medallion");
            }
            basics(&mut g, "U", n);
            let c = card(&g, "Think Twice");
            g.player_mut(P).gy.push(c);
            let before = g.player(P).hand.len();
            let ok = brain::do_cast(&mut g, P, c, Some("gy")).unwrap();
            // Deliberately checks the unchanged parse_cost(Fb) path, including its existing lack of reducers.
            assert_eq!(ok, n >= 3);
            assert_eq!(tapped(&g), if ok { 3 } else { 0 });
            assert_eq!(g.player(P).hand.len() - before, usize::from(ok));
        }
    }
}

#[test]
fn damping_sphere_prices_x_after_prior_casts() {
    let mut g = game();
    perm(&mut g, Q, "Damping Sphere");
    perm(&mut g, P, "Pearl Medallion");
    let prior = card(&g, "Opt");
    g.player_mut(P).turn_casts = Some((g.turn_stamp(), vec![prior; 2]));
    basics(&mut g, "W", 5);
    let c = hand(&mut g, P, &["Secure the Wastes"])[0];
    let before = g.player(P).perms.len();
    assert!(brain::do_cast(&mut g, P, c, None).unwrap());
    assert_eq!(g.player(P).perms.len() - before, 3);
    assert_eq!(tapped(&g), 5);
}

#[test]
fn hangarback_taxes_must_not_become_extra_counters() {
    let mut g = game();
    basics(&mut g, "", 6);
    perm(&mut g, Q, "Sphere of Resistance");
    perm(&mut g, Q, "Sphere of Resistance");
    let c = hand(&mut g, P, &["Hangarback Walker"])[0];
    assert!(brain::do_cast(&mut g, P, c, None).unwrap());
    let m = g.player(P).perms.iter().copied().find(|&m| g.perm(m).cd == Some(c)).unwrap();
    assert_eq!(tapped(&g), 6); // X=2 costs 2X plus the two taxes
    // BUG: Hangarback derives X from mana_spent/2, so two taxes add an unearned +1/+1 counter.
    assert_eq!(g.perm(m).plus, 2);
}

#[test]
fn chalice_taxes_must_not_become_extra_charge_counters() {
    let mut g = game();
    basics(&mut g, "", 4);
    perm(&mut g, Q, "Sphere of Resistance");
    perm(&mut g, Q, "Sphere of Resistance");
    let one = card(&g, "Lightning Bolt");
    g.player_mut(Q).library = vec![one; 20];
    let c = hand(&mut g, P, &["Chalice of the Void"])[0];
    let x = sim_core::impls::t6_stax::chalice_x(&g, P).unwrap();
    assert_eq!(x, 1);
    assert!(brain::do_cast(&mut g, P, c, None).unwrap());
    let m = g.player(P).perms.iter().copied().find(|&m| g.perm(m).cd == Some(c)).unwrap();
    assert_eq!(tapped(&g), 4);
    // BUG: Chalice also derives X from total mana spent, incorrectly counting the taxes as X.
    assert_eq!(g.perm(m).data.int(DataKey::Charge), x as i64);
}

#[test]
fn twinflame_must_pay_the_spell_tax_in_addition_to_strive() {
    let mut g = game();
    basics(&mut g, "R", 2);
    perm(&mut g, Q, "Sphere of Resistance");
    perm(&mut g, P, "Grave Titan");
    let c = hand(&mut g, P, &["Twinflame"])[0];
    let oo = opts(&mut g, c, Some(false));
    if let Some(o) = oo.first() {
        assert!(run(&mut g, o));
        assert_eq!(tapped(&g), 2, "the executor really casts through the tax for only two mana");
    }
    // BUG: Twinflame ignores cost_of and offers/casts for printed {1}{R} through Sphere's {1} tax.
    assert!(
        oo.is_empty(),
        "two mana cannot pay {{1}}{{R}} plus {{1}}: {:?}",
        oo.iter().map(|o| &o.label).collect::<Vec<_>>()
    );
}

#[test]
fn unchanged_combo_accounting_is_conservative_with_an_artifact_reducer() {
    use sim_core::impls::combos;
    let mut g = game();
    perm(&mut g, P, "Etherium Sculptor");
    let c = hand(&mut g, P, &["Walking Ballista"])[0];
    let cmb = combos::all().iter().find(|c| c.name == "Heliod, Sun-Crowned + Walking Ballista").unwrap();
    let (gn, pips) = combos::mana_need(&g, P, cmb, &[c]);
    let exact = mana::cost_of_x(&g, P, c, 2).0 + cmb.mana.0;
    assert_eq!(gn, exact + 1, "one extra mana reserved: conservative, not an underpayment");
    assert_eq!(pips, "W");
}

#[test]
fn hydroid_krasis_body_enters_with_its_paid_x_counters() {
    let mut g = game();
    basics(&mut g, "GU", 6);
    let c = hand(&mut g, P, &["Hydroid Krasis"])[0];
    let s = Situation::new(&g, P);
    let oo = pool::spell_options(&g, P, &s, Some(false));
    let o = oo.iter().find(|o| o.label == "Hydroid Krasis X=4").unwrap();
    assert!(run(&mut g, o));
    assert_eq!(tapped(&g), 6);
    // BUG: krasis calls enter on the bare 0/0, then puts counters on the already-dead permanent.
    let body = g.player(P).perms.iter().copied().find(|&m| g.perm(m).cd == Some(c));
    assert!(body.is_some(), "X=4 must resolve as a living 4/4, not go straight to the graveyard");
    assert_eq!(g.perm(body.unwrap()).plus, 4);
}

#[test]
fn free_positive_x_is_not_lost_when_reducers_exceed_the_pool() {
    let mut g = game();
    basics(&mut g, "W", 1);
    for _ in 0..3 {
        perm(&mut g, P, "Pearl Medallion");
    }
    let c = hand(&mut g, P, &["Secure the Wastes"])[0];
    let before = g.player(P).perms.len();
    assert!(brain::do_cast(&mut g, P, c, None).unwrap());
    assert_eq!(tapped(&g), 1);
    // BUG: the linear bound samples X=1 inside the reduction floor and chooses X=2 rather than payable X=3.
    assert_eq!(g.player(P).perms.len() - before, 3);
}

#[test]
fn planner_reservation_overestimates_reduced_x_by_one_but_never_overspends() {
    use sim_core::ai::planner;
    use sim_core::hooks::Action;
    let mut g = game();
    basics(&mut g, "G", 4);
    perm(&mut g, P, "Emerald Medallion");
    let cards = hand(&mut g, P, &["Green Sun's Zenith", "Llanowar Elves"]);
    let option = Opt {
        utility: 1.0,
        label: "cast Llanowar Elves".into(),
        act: Some(Action::Cast { card: cards[1], zone: None }),
    };
    let kept = planner::reserve(&mut g, P, cards[0], 3, vec![option.clone()]);
    assert!(kept.is_empty(), "document the conservative cost_of(0)+X reservation");
    // The rejected sequence is actually payable: one mana for Elves, three for X=3 Zenith less one.
    assert!(run(&mut g, &option));
    g.player_mut(P).plan_x = Some(3);
    assert!(brain::do_cast(&mut g, P, cards[0], None).unwrap());
    assert_eq!(tapped(&g), 4);
}

#[test]
fn unfixed_artifact_self_costs_are_conservative_with_reducers_and_ignore_krrik() {
    for name in ["Hangarback Walker", "Chalice of the Void"] {
        for krrik in [false, true] {
            let mut g = game();
            basics(&mut g, "", 6);
            if krrik {
                perm(&mut g, P, "K'rrik, Son of Yawgmoth");
            }
            perm(&mut g, P, "Etherium Sculptor");
            let one = card(&g, "Lightning Bolt");
            g.player_mut(Q).library = vec![one; 20];
            let c = hand(&mut g, P, &[name])[0];
            let x = if name == "Hangarback Walker" { 2 } else { 1 };
            let (gn, pips) = mana::cost_of(&g, P, c);
            assert_eq!(gn, 2 * x, "self_cost adds 2X after the artifact reducer was floored away");
            assert!(pips.is_empty());
            assert!(brain::do_cast(&mut g, P, c, None).unwrap());
            assert_eq!(tapped(&g), 2 * x, "one more than the legal 2X-1 cost (conservative)");
            assert_eq!(g.player(P).life, 40, "neither artifact has a black pip for K'rrik");
        }
    }
}

#[test]
fn twinflame_strive_with_a_reducer_pays_one_extra_mana_conservatively() {
    let mut g = game();
    basics(&mut g, "R", 5);
    perm(&mut g, P, "Ruby Medallion");
    for _ in 0..2 {
        perm(&mut g, P, "Grave Titan");
    }
    let c = hand(&mut g, P, &["Twinflame"])[0];
    let oo = opts(&mut g, c, Some(false));
    assert_eq!(oo.len(), 1);
    assert_eq!(oo[0].label, "Twinflame (2 targets)");
    assert!(run(&mut g, &oo[0]));
    assert_eq!(tapped(&g), 5, "printed {{3}}{{R}}{{R}}, one more than the reduced four-mana cost");
}

#[test]
fn garths_ai_braingeyser_prices_its_cap_and_reducers_once() {
    for n in 1..=10 {
        for mode in 0..8 {
            let mut g = game();
            setup(&mut g, "Sapphire Medallion", mode);
            basics(&mut g, "U", n);
            let c = card(&g, "Braingeyser");
            let x = brute(&g, c, 1).min(8);
            let garth = perm(&mut g, P, "Garth One-Eye");
            let f = g.registry.get(card(&g, "Garth One-Eye")).unwrap().options.unwrap();
            let oo = f(&mut g, garth, P, Some(false)).unwrap();
            let o = oo.iter().find(|o| o.label.starts_with("Garth One-Eye: Braingeyser"));
            assert_eq!(o.is_some(), x >= 3, "{n}/{mode}");
            if let Some(o) = o {
                let (gn, pips) = mana::cost_of_x(&g, P, c, x);
                assert!(run(&mut g, o));
                assert_eq!(tapped(&g), gn + pips.len() as u32);
                assert_eq!(g.player(P).hand.len(), x as usize);
            }
        }
    }
}

#[test]
fn thousand_year_storm_copies_the_chosen_x_without_more_mana_or_casts() {
    let mut g = game();
    perm(&mut g, P, "Thousand-Year Storm");
    perm(&mut g, P, "Pearl Medallion");
    let st = g.turn_stamp();
    g.player_mut(P).is_cast_n = Some((st, 2));
    g.player_mut(P).spells_this_turn = 2;
    basics(&mut g, "W", 4);
    let c = hand(&mut g, P, &["Secure the Wastes"])[0];
    let before = g.player(P).perms.len();
    assert!(brain::do_cast(&mut g, P, c, None).unwrap());
    assert_eq!(g.player(P).perms.len() - before, 12, "X=4, original and two copies");
    assert_eq!(tapped(&g), 4);
    assert_eq!(g.player(P).spells_this_turn, 3, "copies are not cast again");
}

#[test]
fn jet_medallion_does_not_reduce_nonblack_x_spells() {
    for (name, pips) in [("Secure the Wastes", "W"), ("Braingeyser", "UU"), ("Astral Cornucopia", "")] {
        let mut g = game();
        basics(&mut g, pips, 6);
        let c = card(&g, name);
        let before = mana::cost_of_x(&g, P, c, 6);
        for _ in 0..2 {
            perm(&mut g, P, "Jet Medallion");
        }
        assert_eq!(mana::cost_of_x(&g, P, c, 6), before);
    }
}
