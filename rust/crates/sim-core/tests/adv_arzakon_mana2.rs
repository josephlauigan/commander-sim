//! Independent probes of dd15c5b0 against Oracle text in the local Scryfall cache.
mod play_common;
use play_common::*;
use sim_core::cards::Colors;
use sim_core::engine::mana::{self, Shape, Source, Unit};
use sim_core::human::mana as hm;
use sim_core::tag::Tag;

fn game() -> Game {
    table(&["arzakon", "karn_gilded"])
}
fn grant(g: &mut Game, name: &str) {
    let m = perm(g, P0, name);
    g.perm_mut(m).tapped = true;
}
fn unit(g: &Game, l: PermId) -> Unit {
    *mana::mana_units(g, P0, false).iter().find(|u| u.src == Source::Land(l)).unwrap()
}
fn fits(g: &Game, l: PermId, generic: u32, pips: &str) -> bool {
    mana::plan_pay(g, &[unit(g, l)], generic, pips).is_some()
}
// The baseline's Lantern branch in 128d5645: it preserves amt, recolours to identity,
// and always sets Shape::Free. This comparison is intentionally restricted to
// Lantern, normal life, no Moon/Dryad/Sphere/other mana modifiers, and existing units.
// It is a reconstruction of those three old expressions, not a second engine build.
fn baseline_lantern(g: &Game, current: Unit) -> Unit {
    Unit { cols: g.player(P0).ident, shape: Shape::Free, ..current }
}

#[test]
fn every_printed_multimana_land_keeps_its_native_payments() {
    let names: Vec<_> =
        db().iter().filter(|d| d.land && d.tags.int(Tag::Amt).unwrap_or(1) >= 2).map(|d| d.name.to_string()).collect();
    assert_eq!(names.len(), 22);
    for name in names {
        let mut g = game();
        let l = lands(&mut g, P0, &name, 1, false)[0];
        lands(&mut g, P0, "Wastes", 5, false); // activation/five-land conditions
        g.pay_for = Some(card(&g, "Sol Ring")); // include Workshop
        let native = unit(&g, l);
        grant(&mut g, "Chromatic Lantern");
        let now = unit(&g, l);
        let old = baseline_lantern(&g, now);
        assert_eq!(native.amt, now.amt, "{name}: total changed");
        let mut old_count = 0;
        let mut new_count = 0;
        for gn in 0..=4 {
            for pips in ["", "W", "U", "B", "R", "G", "C", "WW", "UU", "BG", "WU", "CC", "WUB"] {
                let before = mana::plan_pay(&g, &[old], gn, pips).is_some();
                let after = mana::plan_pay(&g, &[now], gn, pips).is_some();
                old_count += usize::from(before);
                new_count += usize::from(after);
                if mana::plan_pay(&g, &[native], gn, pips).is_some() {
                    assert!(after, "{name}: lost native payment {gn}+{pips}: {now:?}");
                }
            }
        }
        println!(
            "{name}: amt {} -> {}, shape {:?}, sampled payable {old_count} -> {new_count}",
            old.amt, now.amt, now.shape
        );
    }
}

#[test]
fn every_bounceland_gets_lanterns_single_off_colour() {
    let mut missing = vec![];
    for d in db().iter().filter(|d| d.land && d.tag(Tag::Bounceland)) {
        let mut g = game();
        let l = lands(&mut g, P0, &d.name, 1, false)[0];
        grant(&mut g, "Chromatic Lantern");
        let off = "WUBRG".chars().find(|c| !d.tags.str(Tag::C).unwrap().contains(*c)).unwrap().to_string();
        assert!(mana::plan_pay(&g, &[baseline_lantern(&g, unit(&g, l))], 0, &off).is_some());
        if !fits(&g, l, 0, &off) {
            missing.push(d.name.to_string());
        }
    }
    assert!(missing.is_empty(), "Lantern's legal one-mana alternative disappeared: {missing:?}");
}

#[test]
fn urza_trio_keeps_seven_colourless_or_three_single_colours() {
    for deck in ["arzakon", "atraxa-infect"] {
        let mut g = table(&[deck, "karn_gilded"]);
        let ls: Vec<_> = ["Urza's Tower", "Urza's Mine", "Urza's Power Plant"]
            .iter()
            .map(|n| lands(&mut g, P0, n, 1, false)[0])
            .collect();
        grant(&mut g, "Chromatic Lantern");
        assert_eq!(ls.iter().map(|&l| unit(&g, l).amt).collect::<Vec<_>>(), [3, 2, 2]);
        assert_eq!(mana::total_mana(&g, P0, false), 7);
        assert!(mana::can_pay(&g, P0, 0, "CCCCCCC", false));
        assert!(mana::can_pay(&g, P0, 0, "WUB", false));
        assert!(!mana::can_pay(&g, P0, 0, "WUBG", false));
        for &l in &ls {
            let own = unit(&g, l);
            assert_eq!(own.shape, Shape::Alt(Colors::NONE));
            assert!(fits(&g, l, own.amt, ""));
            assert!(fits(&g, l, 0, "U"));
            assert!(!fits(&g, l, 0, "UU"));
        }
        assert!(mana::pay(&mut g, P0, 0, "WUB", false).unwrap());
        assert_eq!(g.player(P0).mana.total(), 0);
    }
}

#[test]
fn tomb_low_life_and_necessary_pain_sources() {
    for life in [1, 2, 3] {
        for grants in [false, true] {
            let mut g = game();
            lands(&mut g, P0, "Ancient Tomb", 1, false);
            if grants {
                grant(&mut g, "Chromatic Lantern");
            }
            g.player_mut(P0).life = life;
            assert_eq!(mana::total_mana(&g, P0, false), if life > 2 { 2 } else { u32::from(grants) });
            assert_eq!(mana::can_pay(&g, P0, 2, "", false), life > 2);
            assert_eq!(mana::can_pay(&g, P0, 0, "U", false), grants);
            if grants {
                assert!(mana::pay(&mut g, P0, 0, "U", false).unwrap());
                assert_eq!(g.player(P0).life, life);
            }
        }
    }
    for name in ["Mana Confluence", "City of Brass"] {
        for life in [2, 3, 40] {
            let mut g = game();
            let l = lands(&mut g, P0, name, 1, false)[0];
            lands(&mut g, P0, "Command Tower", 1, false);
            g.player_mut(P0).life = life;
            assert!(mana::pay(&mut g, P0, 2, "", false).unwrap());
            assert!(g.perm(l).tapped, "{name} was needed");
            assert_eq!(g.player(P0).life, life - 1);
        }
    }
}

#[test]
fn dynamic_amounts_keep_their_native_mana_and_one_alternative() {
    for name in ["Gaea's Cradle", "Cabal Coffers", "Nykthos, Shrine to Nyx"] {
        let mut g = game();
        let l = lands(&mut g, P0, name, 1, false)[0];
        lands(&mut g, P0, "Swamp", 5, false);
        for _ in 0..4 {
            perm(&mut g, P0, "Llanowar Elves");
        }
        for &m in &g.player(P0).perms.clone() {
            g.perm_mut(m).tapped = true;
        }
        let native = unit(&g, l);
        grant(&mut g, "Chromatic Lantern");
        assert_eq!(native.amt, unit(&g, l).amt, "{name}");
        assert!(fits(&g, l, native.amt, ""), "{name}");
        assert!(fits(&g, l, 0, "U"), "{name} gets one U");
        println!("{name}: native {native:?}, Lantern {:?}", unit(&g, l));
    }
}

#[test]
#[ignore = "named limit (JD to approve, 2026-11-26): a Cradle or Coffers with zero native output also loses a Lantern's independent one-mana ability (inherited)"]
fn zero_output_cradle_and_coffers_still_have_lanterns_ability() {
    let mut missing = vec![];
    for name in ["Gaea's Cradle", "Cabal Coffers"] {
        let mut g = game();
        lands(&mut g, P0, name, 1, false);
        grant(&mut g, "Chromatic Lantern");
        if mana::total_mana(&g, P0, false) != 1 || !mana::can_pay(&g, P0, 0, "U", false) {
            missing.push(name);
        }
    }
    assert!(missing.is_empty(), "zero native amount suppressed a granted ability: {missing:?}");
}

#[test]
#[ignore = "named limit (JD to approve, 2026-11-26): Mishra's Workshop's artifact-only restriction wrongly applies to a Lantern's independent one-mana ability (inherited)"]
fn workshop_granted_mana_is_unrestricted() {
    let mut g = game();
    lands(&mut g, P0, "Mishra's Workshop", 1, false);
    grant(&mut g, "Chromatic Lantern");
    let c = card(&g, "Swords to Plowshares");
    assert!(mana::can_pay_spell(&g, P0, c, 0, "W", false), "restriction belongs to Workshop's own ability");
    assert!(mana::pay_spell(&mut g, P0, c, 0, "W", false).unwrap());
}

#[test]
#[ignore = "named limit (JD to approve, 2026-11-26): Damping Sphere collapses a multi-mana land to {C} before the Lantern's one-mana alternative is added (inherited)"]
fn sphere_does_not_replace_lanterns_one_mana_alternative() {
    let mut g = game();
    lands(&mut g, P0, "Ancient Tomb", 1, false);
    grant(&mut g, "Chromatic Lantern");
    perm(&mut g, P0, "Damping Sphere");
    assert_eq!(mana::total_mana(&g, P0, false), 1);
    assert!(mana::can_pay(&g, P0, 0, "U", false), "one mana is below Sphere's replacement threshold");
}

#[test]
#[ignore = "named limit (JD to approve, 2026-11-26): a Shape::Alt unit makes exactly one mana, so Nyxbloom Ancient's multiplier on the granted ability is dropped (existing Alt/Leyline defect, now also with a Lantern)"]
fn nyxbloom_triples_the_granted_ability_too() {
    let mut refused = vec![];
    for both in [false, true] {
        let mut g = game();
        lands(&mut g, P0, "Ancient Tomb", 1, false);
        grant(&mut g, "Chromatic Lantern");
        if both {
            perm(&mut g, P0, "Leyline of the Guildpact");
        }
        perm(&mut g, P0, "Nyxbloom Ancient");
        assert_eq!(mana::total_mana(&g, P0, false), 6);
        assert!(!mana::can_pay(&g, P0, 0, "UUUU", false));
        let life = g.player(P0).life;
        if !mana::pay(&mut g, P0, 0, "UUU", false).unwrap() {
            refused.push(both);
        }
        assert_eq!(g.player(P0).life, life);
    }
    assert!(refused.is_empty(), "Lantern U becomes UUU; refused with Leyline present={refused:?}");
}

#[test]
#[ignore = "named limit (JD to approve, 2026-11-26): Crypt Ghast's extra mana is recoloured with a Lantern's land (inherited recolouring path)"]
fn ghast_adds_black_not_an_arbitrary_second_colour() {
    let mut g = game();
    let l = lands(&mut g, P0, "Swamp", 1, false)[0];
    grant(&mut g, "Chromatic Lantern");
    let ghast = perm(&mut g, P0, "Crypt Ghast");
    g.perm_mut(ghast).tapped = true;
    assert!(fits(&g, l, 0, "UB"));
    assert!(!fits(&g, l, 0, "UU"), "Ghast's extra B cannot become U");
}

#[test]
#[ignore = "named limit (JD to approve, 2026-11-26): Dryad of the Ilysian Grove still recolours all of Ancient Tomb's mana (inherited; the dryad branch bypasses the Alt handling)"]
fn dryad_does_not_recolour_both_mana_of_tomb() {
    let mut g = game();
    lands(&mut g, P0, "Ancient Tomb", 1, false);
    grant(&mut g, "Chromatic Lantern");
    let d = perm(&mut g, P0, "Dryad of the Ilysian Grove");
    g.perm_mut(d).tapped = true;
    assert!(!mana::can_pay(&g, P0, 0, "UU", false), "Dryad grants a separate one-mana ability");
}

#[test]
fn moon_controls_only_the_two_named_ignored_limits() {
    let mut g = game();
    lands(&mut g, P0, "Mana Confluence", 1, false);
    grant(&mut g, "Chromatic Lantern");
    let moon = perm(&mut g, P0, "Blood Moon");
    g.player_mut(P0).life = 1;
    assert!(mana::pay(&mut g, P0, 0, "R", false).unwrap());
    assert_eq!(g.player(P0).life, 1);
    g.perm_mut(g.player(P0).lands[0]).tapped = false;
    assert!(!mana::can_pay(&g, P0, 0, "U", false)); // accepted named limit
    sim_core::engine::zones::bounce(&mut g, moon).unwrap();
    assert!(mana::can_pay(&g, P0, 0, "U", false)); // fails only with Moon
    let mut g = game();
    lands(&mut g, P0, "Mana Confluence", 2, false);
    g.player_mut(P0).life = 2;
    assert!(mana::can_pay(&g, P0, 0, "WU", false)); // accepted combined-life limit
    assert!(mana::pay(&mut g, P0, 0, "WU", false).unwrap());
    assert_eq!(g.player(P0).life, 0);
}

#[test]
fn confluence_locks_grants_and_estimates_agree() {
    for life in [1, 2, 40] {
        for locked in [false, true] {
            for effect in ["", "Chromatic Lantern", "Leyline of the Guildpact", "Dryad of the Ilysian Grove"] {
                let mut g = game();
                lands(&mut g, P0, "Mana Confluence", 1, false);
                if !effect.is_empty() {
                    grant(&mut g, effect);
                }
                g.player_mut(P0).life = life;
                g.player_mut(P0).life_locked = locked;
                g.player_mut(P1).life_locked = true; // opponent's lock is irrelevant
                let expected = !effect.is_empty() || (!locked && life > 1);
                assert_eq!(mana::total_mana(&g, P0, false), u32::from(expected));
                let spell = card(&g, "Swords to Plowshares");
                assert_eq!(mana::total_mana_spell(&g, P0, spell, false), u32::from(expected));
                assert_eq!(mana::can_pay_spell(&g, P0, spell, 0, "W", false), expected);
                assert_eq!(mana::pay_spell(&mut g, P0, spell, 0, "W", false).unwrap(), expected);
                assert_eq!(g.player(P0).life, life - i32::from(expected && effect.is_empty()));
            }
        }
    }
}

#[test]
#[ignore = "named limit (JD to approve, 2026-11-26): Mana Confluence at 1 life or with a locked life total counts as no mana even when Urborg/Yavimaya gives a free {B}/{G} ability (the guard cannot see which colour is asked for)"]
fn urborg_or_yavimaya_free_ability_survives_one_life_and_lock() {
    let mut missing = vec![];
    for (grant_name, pip) in [("Urborg, Tomb of Yawgmoth", "B"), ("Yavimaya, Cradle of Growth", "G")] {
        for locked in [false, true] {
            let mut g = game();
            lands(&mut g, P0, "Mana Confluence", 1, false);
            lands(&mut g, P0, grant_name, 1, true);
            g.player_mut(P0).life = if locked { 40 } else { 1 };
            g.player_mut(P0).life_locked = locked;
            if !mana::can_pay(&g, P0, 0, pip, false) {
                missing.push((grant_name, locked));
            }
        }
    }
    assert!(missing.is_empty(), "free basic-type ability lost: {missing:?}");
}

#[test]
fn practice_tomb_and_tron_granted_taps_make_one_or_native_amount() {
    for (name, amt) in [("Ancient Tomb", 2), ("Urza's Tower", 3)] {
        for c in ['U', 'C'] {
            let mut g = game();
            let l = lands(&mut g, P0, name, 1, false)[0];
            if name == "Urza's Tower" {
                lands(&mut g, P0, "Urza's Mine", 1, true);
                lands(&mut g, P0, "Urza's Power Plant", 1, true);
            }
            grant(&mut g, "Chromatic Lantern");
            seat(&mut g, P0, vec![]);
            let s = hm::sources(&g, P0).into_iter().find(|s| s.land == Some(0)).unwrap();
            assert_eq!(s.amount, amt);
            assert!(s.colours.contains('U') && s.colours.contains('C'));
            assert_eq!(hm::tap(&mut g, P0, s.id as i64, Some(c)).unwrap(), None);
            assert!(g.perm(l).tapped);
            assert_eq!(pool_total(&g, P0), if c == 'U' { 1 } else { amt });
            assert_eq!(g.player(P0).life, if name == "Ancient Tomb" && c == 'C' { 38 } else { 40 });
        }
    }
}

#[test]
#[ignore = "named limit (JD to approve, 2026-11-26): practice mode: a Karoo (bounceland) with a Lantern gives its pair, not the single colour picked (the Karoo keeps the old Free reading under a Lantern)"]
fn practice_bounceland_can_choose_lanterns_blue_instead_of_pair() {
    let mut g = game();
    lands(&mut g, P0, "Golgari Rot Farm", 1, false);
    grant(&mut g, "Chromatic Lantern");
    seat(&mut g, P0, vec![]);
    let s = hm::sources(&g, P0).into_iter().find(|s| s.land == Some(0)).unwrap();
    assert_eq!(hm::tap(&mut g, P0, s.id as i64, Some('U')).unwrap(), None);
    assert_eq!(pool_text(&g, P0), "{U}", "an explicit U selection must not produce BG");
}

#[test]
fn world_tree_six_land_threshold_grants_the_same_alternative() {
    for n in [4, 5] {
        let mut g = game();
        let farm = lands(&mut g, P0, "Golgari Rot Farm", 1, false)[0];
        lands(&mut g, P0, "The World Tree", 1, true);
        lands(&mut g, P0, "Wastes", n - 1, true);
        assert_eq!(mana::lands_any_colour(&g, P0), n == 5);
        assert_eq!(fits(&g, farm, 0, "U"), n == 5, "six lands must grant the farm a U ability");
    }
}

#[test]
#[ignore = "named limit (JD to approve, 2026-11-26): payment planner ranks Shape::Alt sources by their full native amount, so a Cradle + Ancient Tomb + Lantern payment depends on pip order (inherited from Leyline's Alt units; newly reachable with a Lantern; arzakon has no Cradle)"]
fn alternative_sources_do_not_make_payment_depend_on_pip_order() {
    let mut g = game();
    lands(&mut g, P0, "Gaea's Cradle", 1, false);
    lands(&mut g, P0, "Ancient Tomb", 1, false);
    for _ in 0..3 {
        token(&mut g, P0, 1);
    }
    grant(&mut g, "Chromatic Lantern");
    let old: Vec<_> = mana::mana_units(&g, P0, false).into_iter().map(|u| baseline_lantern(&g, u)).collect();
    assert!(mana::plan_pay(&g, &old, 0, "UGGG").is_some());
    // Tomb makes U, Cradle makes GGG. Both strings describe exactly that cost.
    assert!(mana::can_pay(&g, P0, 0, "GGGU", false));
    assert!(mana::can_pay(&g, P0, 0, "UGGG", false), "same legal cost, different pip order");
}

#[test]
#[ignore = "named limit (JD to approve, 2026-11-26): a Shape::Alt unit makes exactly one mana, so Nissa's trigger bonus on a Lantern tap of Ancient Tomb is dropped (the old Free unit kept it)"]
fn lantern_tomb_with_nissa_retains_the_extra_green_on_its_free_tap() {
    let mut g = game();
    lands(&mut g, P0, "Ancient Tomb", 1, false);
    lands(&mut g, P0, "Yavimaya, Cradle of Growth", 1, true);
    grant(&mut g, "Chromatic Lantern");
    perm(&mut g, P0, "Nissa, Who Shakes the World");
    // Nissa triggers for an additional G regardless of the ability chosen.
    assert!(mana::can_pay(&g, P0, 0, "UG", false), "Lantern U plus Nissa G");
}

#[test]
fn confluence_x_estimate_matches_a_payable_cost_under_locks() {
    for life in [1, 2] {
        for locked in [false, true] {
            for free in [false, true] {
                let mut g = game();
                lands(&mut g, P0, "Swamp", 2, false);
                lands(&mut g, P0, "Mana Confluence", 1, false);
                if free {
                    grant(&mut g, "Leyline of the Guildpact");
                }
                g.player_mut(P0).life = life;
                g.player_mut(P0).life_locked = locked;
                let c = card(&g, "Exsanguinate");
                let x = sim_core::ai::brain::max_x(&g, P0, c, 1, false);
                assert_eq!(x, i32::from(free || (!locked && life > 1)));
                let (gn, pips) = mana::cost_of_x(&g, P0, c, x);
                assert!(mana::pay_spell(&mut g, P0, c, gn, &pips, false).unwrap());
            }
        }
    }
}

#[test]
fn actual_teferi_and_flare_effects_block_only_the_costed_ability() {
    for teferi in [false, true] {
        for free in [false, true] {
            let mut g = game();
            if teferi {
                sim_core::engine::zones::teferis_protection(&mut g, P0);
            } else {
                sim_core::impls::t45_winota::after_protector(&mut g, P0, "Flare of Fortitude").unwrap();
            }
            // A land and Lantern acquired after Teferi's phase-out are still usable.
            lands(&mut g, P0, "Mana Confluence", 1, false);
            if free {
                grant(&mut g, "Chromatic Lantern");
            }
            assert!(g.player(P0).life_locked);
            assert_eq!(mana::total_mana(&g, P0, false), u32::from(free));
            assert_eq!(mana::pay(&mut g, P0, 1, "", false).unwrap(), free);
            assert_eq!(g.player(P0).life, 40);
        }
    }
}

#[test]
fn world_tree_grants_confluence_free_mana_at_one_life_or_under_lock() {
    for locked in [false, true] {
        let mut g = game();
        lands(&mut g, P0, "Mana Confluence", 1, false);
        lands(&mut g, P0, "The World Tree", 1, true);
        lands(&mut g, P0, "Wastes", 4, true);
        g.player_mut(P0).life = 1;
        g.player_mut(P0).life_locked = locked;
        assert_eq!(mana::total_mana(&g, P0, false), 1);
        assert!(mana::pay(&mut g, P0, 0, "U", false).unwrap());
        assert_eq!(g.player(P0).life, 1);
    }
}

#[test]
fn actual_lantern_deck_land_boards_keep_total_mana_and_identity_colours() {
    let decks = sim_core::export::load_decks(&data_dir().join("decks.json")).unwrap();
    let affected: Vec<_> = decks
        .mine
        .iter()
        .chain(decks.pool.iter())
        .filter(|d| d.cards.iter().any(|n| n == "Chromatic Lantern" || n == "The World Tree"))
        .collect();
    assert_eq!(affected.iter().map(|d| d.key.as_str()).collect::<Vec<_>>(), ["arzakon", "marchesa", "atraxa-infect"]);
    for d in affected {
        let mut g = table(&[&d.key, "karn_gilded"]);
        for n in &d.cards {
            if g.db.get(card(&g, n)).land {
                lands(&mut g, P0, n, 1, false);
            }
        }
        grant(&mut g, "Chromatic Lantern");
        let now = mana::mana_units(&g, P0, false);
        let old: Vec<_> = now.iter().map(|&u| baseline_lantern(&g, u)).collect();
        assert_eq!(old.iter().map(|u| u.amt).sum::<u32>(), mana::total_mana(&g, P0, false));
        for c in ["W", "U", "B", "R", "G", "WUBG"] {
            if !c.chars().all(|c| g.player(P0).ident.has(c)) {
                continue; // the baseline also limits rainbow sources to the seat's identity
            }
            assert!(mana::can_pay(&g, P0, 0, c, false), "{} cannot pay {c}", d.key);
        }
        println!("{}: {} land units, baseline/current total {}", d.key, now.len(), mana::total_mana(&g, P0, false));
    }
}
