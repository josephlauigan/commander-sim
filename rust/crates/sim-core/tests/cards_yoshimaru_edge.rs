//! Edge cases of the Yoshimaru / Tymna cards found in review (each expectation follows the Oracle text): the
//! acceptance set of the fix round for impls/yoshimaru_cards.rs.

mod play_common;
use play_common::*;
use sim_core::cards::Types;
use sim_core::engine::{cast, combat, turn, zones};
use sim_core::hooks::Opt;
use sim_core::state::{CopyExc, Ctx, DataKey, Step, Val};

const Y: &str = "Yoshimaru, Ever Faithful";
const W: &str = "Witch Enchanter // Witch-Blessed Meadow";

fn tbl() -> Game {
    table(&["heliod-mono-white-stax", "seph", "sauron"])
}
fn person(g: &mut Game, pick: impl Fn(&Request) -> Option<usize> + Send + 'static) -> std::sync::Arc<ScriptHuman> {
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" {
            return Answer::Action(json!({"do":"pass"}));
        }
        pick(r).map(Answer::Pick).unwrap_or(Answer::Decline)
    }));
    g.humans.add(P0, h.clone());
    h
}
fn land(g: &mut Game, name: &str) -> PermId {
    let c = take(g, P0, name);
    turn::play_land_card(g, P0, c, "plays").unwrap();
    *g.player(P0).lands.last().unwrap()
}
fn opts(g: &mut Game, s: PermId) -> Vec<Opt> {
    let f = g.registry.get(g.perm(s).card()).unwrap().options.unwrap();
    f(g, s, P0, Some(false)).unwrap()
}
fn run(g: &mut Game, o: &Opt) -> bool {
    sim_core::ai::act::perform(g, P0, o.act.as_ref().unwrap()).unwrap()
}
fn seriema(g: &mut Game) -> PermId {
    g.player_mut(P0).library.clear();
    perm(g, P0, "The Seriema")
}
fn charge(g: &Game, s: PermId) -> i64 {
    g.perm(s).data.int(DataKey::Charge)
}
fn station(g: &mut Game, s: PermId, power: i32) {
    let t = token(g, P0, power);
    g.perm_mut(t).sick = true;
    let o = opts(g, s).remove(0);
    assert!(run(g, &o));
}
fn cast_free(g: &mut Game, p: PlayerId, name: &str, zone: sim_core::sym::Sym) {
    let c = card(g, name);
    if zone == "hand" {
        hand(g, p, &[name]);
    } else if zone == "gy" {
        g.player_mut(p).gy.push(c);
    } else {
        g.player_mut(p).exile.push(c);
    }
    assert!(cast::cast_card(g, p, c, zone, Ctx::default()).unwrap());
}

#[test]
fn yoshimaru_phase_in_does_not_retroactively_count_lands() {
    let mut g = tbl();
    let y = perm(&mut g, P0, Y);
    g.perm_mut(y).phased = true;
    land(&mut g, "Minas Tirith");
    g.perm_mut(y).phased = false;
    land(&mut g, "Plains");
    assert_eq!(g.perm(y).plus, 0, "the legendary land entered while Yoshimaru was phased out");
}

#[test]
fn yoshimaru_sees_sun_titan_return_a_legendary_land() {
    let mut g = tbl();
    let y = perm(&mut g, P0, Y);
    let c = card(&g, "Minas Tirith");
    g.player_mut(P0).gy = vec![c];
    perm(&mut g, P0, "Sun Titan");
    // Sun Titan's AI branch puts the land onto the battlefield without landfall; the state-based check that follows
    // every resolution in a game (not run by this harness) sees it enter
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert!(g.player(P0).lands.iter().any(|&l| g.perm(l).card() == c));
    assert_eq!(g.perm(y).plus, 1, "a land returned by Sun Titan entered");
}

#[test]
fn yoshimaru_panharmonicon_doubles_legendary_creature_land_entry() {
    let mut g = tbl();
    let y = perm(&mut g, P0, Y);
    perm(&mut g, P0, "Panharmonicon");
    let c = card(&g, "Minas Tirith");
    turn::enter_token_land_copy(
        &mut g,
        P0,
        c,
        CopyExc { add_types: Types::CREATURE, pt: Some((3, 3)), ..Default::default() },
    )
    .unwrap();
    assert_eq!(g.perm(y).plus, 2, "a legendary land creature enters: Panharmonicon applies");
}

#[test]
fn yoshimaru_hushbringer_stops_legendary_creature_land_entry() {
    let mut g = tbl();
    let y = perm(&mut g, P0, Y);
    perm(&mut g, P1, "Hushbringer");
    let c = card(&g, "Minas Tirith");
    turn::enter_token_land_copy(
        &mut g,
        P0,
        c,
        CopyExc { add_types: Types::CREATURE, pt: Some((3, 3)), ..Default::default() },
    )
    .unwrap();
    assert_eq!(g.perm(y).plus, 0, "creatures entering must not trigger Yoshimaru");
}

#[test]
fn yoshimaru_blink_resets_land_marker() {
    let mut g = tbl();
    let y = perm(&mut g, P0, Y);
    land(&mut g, "Minas Tirith");
    let y2 = sim_core::impls::t2::blink(&mut g, P0, y).unwrap().unwrap();
    assert_eq!(g.perm(y2).plus, 0);
    land(&mut g, "Plains");
    assert_eq!(g.perm(y2).plus, 0);
    land(&mut g, "Eiganjo, Seat of the Empire");
    assert_eq!(g.perm(y2).plus, 1);
}

#[test]
fn yoshimaru_legendary_token_and_nonlegendary_copy() {
    let mut g = tbl();
    let y = perm(&mut g, P0, Y);
    let c = card(&g, "Tymna the Weaver");
    zones::enter_token_copy(&mut g, P0, c).unwrap();
    assert_eq!(g.perm(y).plus, 1);
    zones::enter_token_copy_with(&mut g, P0, c, Some(CopyExc { not_legendary: true, ..Default::default() })).unwrap();
    assert_eq!(g.perm(y).plus, 1);
}

#[test]
fn yoshimaru_panharmonicon_and_hushbringer_nonland() {
    let mut g = tbl();
    let y = perm(&mut g, P0, Y);
    perm(&mut g, P0, "Panharmonicon");
    perm(&mut g, P0, "Tymna the Weaver");
    assert_eq!(g.perm(y).plus, 2);
    perm(&mut g, P1, "Hushbringer");
    perm(&mut g, P0, "Kambal, Consul of Allocation");
    assert_eq!(g.perm(y).plus, 2);
    perm(&mut g, P0, "The One Ring");
    assert_eq!(g.perm(y).plus, 4);
}

fn minas_ready(g: &mut Game) -> PermId {
    let l = land(g, "Minas Tirith");
    g.perm_mut(l).tapped = false;
    g.player_mut(P0).pool.add('W', 2);
    let st = g.turn_stamp();
    g.player_mut(P0).atk_count = (Some(st), 2);
    let (a, b) = (token(g, P0, 2), token(g, P0, 2));
    let pl = g.player_mut(P0);
    pl.attacking = Some(st);
    pl.attackers = vec![a, b];
    l
}

#[test]
fn minas_person_can_draw_with_one_card_left() {
    let mut g = tbl();
    person(&mut g, |_| None);
    let l = minas_ready(&mut g);
    let c = card(&g, "Plains");
    g.player_mut(P0).library = vec![c];
    let o = sim_core::human::alt::land_alts(&mut g, P0, l).unwrap();
    assert!(o.iter().any(|(s, _)| s.contains("draw")), "legal draw with one card left was hidden");
}

#[test]
fn minas_two_distinct_attackers_in_extra_combats_count() {
    let mut g = tbl();
    person(&mut g, |_| None);
    let l = land(&mut g, "Minas Tirith");
    g.perm_mut(l).tapped = false;
    g.player_mut(P0).pool.add('W', 2);
    let a = token(&mut g, P0, 2);
    let b = token(&mut g, P0, 2);
    combat::attack_triggers(&mut g, P0, &[a], P1).unwrap();
    combat::attack_triggers(&mut g, P0, &[b], P1).unwrap();
    let o = sim_core::human::alt::land_alts(&mut g, P0, l).unwrap();
    assert!(o.iter().any(|(s, _)| s.contains("draw")), "attacked with two different creatures this turn");
}

#[test]
fn minas_draw_after_attackers_leave_and_in_combat() {
    let mut g = tbl();
    person(&mut g, |_| None);
    let l = land(&mut g, "Minas Tirith");
    g.perm_mut(l).tapped = false;
    g.player_mut(P0).pool.add('W', 2);
    let a = token(&mut g, P0, 2);
    let b = token(&mut g, P0, 2);
    combat::attack_triggers(&mut g, P0, &[a, b], P1).unwrap();
    zones::leave(&mut g, a).unwrap();
    zones::leave(&mut g, b).unwrap();
    g.step = Step::Combat;
    let o = sim_core::human::alt::land_alts(&mut g, P0, l).unwrap();
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, &o[0].1));
    assert_eq!(g.player(P0).pool.total(), 0);
}

#[test]
fn station_still_legal_after_seven_counters() {
    let mut g = tbl();
    let s = seriema(&mut g);
    station(&mut g, s, 7);
    token(&mut g, P0, 2);
    person(&mut g, |r| r.choices.iter().position(|s| s.contains("power 2")));
    assert_eq!(opts(&mut g, s).len(), 1, "Station has no maximum-counter activation restriction");
}

#[test]
fn seriema_stops_being_creature_below_seven() {
    let mut g = tbl();
    let s = seriema(&mut g);
    station(&mut g, s, 7);
    assert!(g.is_creature(s));
    // State after an effect removes one charge counter.
    g.perm_mut(s).data.set(DataKey::Charge, Val::Int(6));
    assert!(!g.is_creature(s), "the 7+ type effect is conditional, not permanent");
}

#[test]
fn seriema_becomes_creature_when_other_effect_puts_seventh_counter() {
    let mut g = tbl();
    let s = seriema(&mut g);
    station(&mut g, s, 6);
    sim_core::impls::common::proliferate(&mut g, P0, 1).unwrap();
    assert_eq!(charge(&g, s), 7);
    assert!(g.is_creature(s), "any source of charge counters can reach the 7+ threshold");
}

#[test]
fn station_zero_power_sick_creature_is_legal_and_choices_exact() {
    let mut g = tbl();
    let s = seriema(&mut g);
    let zero = token(&mut g, P0, 1);
    g.perm_mut(zero).pow = 0;
    g.perm_mut(zero).tgh = 1;
    g.perm_mut(zero).sick = true;
    let tapped = token(&mut g, P0, 3);
    g.perm_mut(tapped).tapped = true;
    token(&mut g, P1, 4);
    let phased = token(&mut g, P0, 5);
    g.perm_mut(phased).phased = true;
    let h = person(&mut g, |r| r.choices.iter().position(|s| s.contains("power 0")));
    let o = opts(&mut g, s).remove(0);
    assert!(run(&mut g, &o));
    assert!(g.perm(zero).tapped);
    assert_eq!(charge(&g, s), 0);
    let r = h.asked().into_iter().find(|r| r.kind == "choose").unwrap();
    assert_eq!(r.choices.len(), 2, "zero-power creature and cancel only: {:?}", r.choices);
}

#[test]
fn station_doubling_season_and_solemnity() {
    for (effect, expected) in [("Doubling Season", 8), ("Solemnity", 0)] {
        let mut g = tbl();
        let s = seriema(&mut g);
        perm(&mut g, P0, effect);
        station(&mut g, s, 4);
        assert_eq!(charge(&g, s), expected, "{effect}");
        assert_eq!(g.is_creature(s), expected >= 7);
    }
}

#[test]
fn seriema_person_can_find_nothing_and_cannot_find_nonlegend() {
    let mut g = tbl();
    let cs = ["Tymna the Weaver", "Mother of Runes", "The One Ring"].map(|n| card(&g, n));
    g.player_mut(P0).library = cs.to_vec();
    let h = person(&mut g, |_| None);
    perm(&mut g, P0, "The Seriema");
    assert!(g.player(P0).hand.is_empty());
    assert_eq!(g.player(P0).library.len(), 3);
    let r = h.asked().into_iter().find(|r| r.choices.iter().any(|s| s.contains("Tymna"))).unwrap();
    assert!(!r.choices.iter().any(|s| s.contains("Mother of Runes") || s.contains("The One Ring")));
}

#[test]
fn kambal_exile_graveyard_and_copies() {
    let mut g = tbl();
    perm(&mut g, P0, "Kambal, Consul of Allocation");
    cast_free(&mut g, P1, "Sol Ring", "exile");
    assert_eq!((g.player(P0).life, g.player(P1).life), (42, 38));
    cast_free(&mut g, P1, "Mind Stone", "gy");
    assert_eq!((g.player(P0).life, g.player(P1).life), (44, 36));
    let c = card(&g, "Sol Ring");
    cast::copy_spell(&mut g, P1, c, None, false).unwrap();
    assert_eq!((g.player(P0).life, g.player(P1).life), (44, 36));
}

#[test]
fn kambal_multiple_controllers_and_spell_types() {
    let mut g = tbl();
    perm(&mut g, P0, "Kambal, Consul of Allocation");
    perm(&mut g, P2, "Kambal, Consul of Allocation");
    cast_free(&mut g, P1, "Mother of Runes", "hand");
    assert_eq!((g.player(P0).life, g.player(P1).life, g.player(P2).life), (40, 40, 40));
    cast_free(&mut g, P1, "Sol Ring", "hand");
    assert_eq!((g.player(P0).life, g.player(P1).life, g.player(P2).life), (42, 36, 42));
}

fn play_meadow(g: &mut Game) -> PermId {
    let c = hand(g, P0, &[W])[0];
    let o = sim_core::human::alt::hand_alts(g, P0, c)
        .unwrap()
        .into_iter()
        .find(|(s, _)| s.contains("as a land"))
        .unwrap()
        .1;
    assert!(run(g, &o));
    *g.player(P0).lands.last().unwrap()
}

#[test]
fn meadow_land_face_is_not_a_creature() {
    let mut g = tbl();
    person(&mut g, |_| Some(1));
    let l = play_meadow(&mut g);
    assert!(!g.is_creature(l), "back face is Land, not Creature");
    assert!(sim_core::engine::chars::card_types(&g, l).has(Types::LAND));
}

#[test]
fn meadow_below_three_life_no_prompt_and_land_drop_used() {
    let mut g = tbl();
    let h = person(&mut g, |_| Some(0));
    g.player_mut(P0).life = 2;
    let l = play_meadow(&mut g);
    assert!(g.perm(l).tapped);
    assert_eq!(g.player(P0).life, 2);
    assert_eq!(g.player(P0).lands_played, 1);
    assert!(!h.asked().iter().any(|r| r.prompt.contains("pay 3 life")));
}

#[test]
fn meadow_person_can_choose_spelunking_after_declining_payment() {
    let mut g = tbl();
    perm(&mut g, P0, "Spelunking");
    person(&mut g, |r| {
        if r.prompt.contains("pay 3 life") {
            Some(1)
        } else {
            r.choices.iter().position(|s| s.contains("untapped") || s.contains("Spelunking"))
        }
    });
    let l = play_meadow(&mut g);
    assert!(!g.perm(l).tapped, "the person must be able to order Spelunking last and enter untapped without paying");
}

#[test]
fn witch_target_menu_excludes_own_and_protected_permanents() {
    let mut g = tbl();
    perm(&mut g, P0, "Mind Stone");
    perm(&mut g, P1, "Mother of Runes");
    let ring = perm(&mut g, P1, "Sol Ring");
    let rhystic = perm(&mut g, P1, "Rhystic Study");
    g.perm_mut(ring).eot_kw.push("hexproof".into());
    let h = person(&mut g, |r| r.choices.iter().position(|s| s.contains("Rhystic")));
    perm(&mut g, P0, W);
    assert!(!g.perm(rhystic).on_bf && g.perm(ring).on_bf);
    let r = h.asked().into_iter().find(|r| r.kind == "target").unwrap();
    assert!(
        !r.choices.iter().any(|s| s.contains("Sol Ring") || s.contains("Mind Stone") || s.contains("Mother of Runes")),
        "{:?}",
        r.choices
    );
}

#[test]
fn station_uses_power_on_resolution_after_pump() {
    let mut g = tbl();
    let s = seriema(&mut g);
    let t = token(&mut g, P0, 6);
    g.perm_mut(t).name = "Station volunteer".into();
    g.perm_mut(t).sick = true;
    hand(&mut g, P1, &["Karametra's Blessing"]);
    pool(&mut g, P1, &[('W', 1)]);
    let mut used = false;
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" {
            if !used {
                used = true;
                Answer::Action(json!({"do":"cast", "card":0}))
            } else {
                Answer::Action(json!({"do":"pass"}))
            }
        } else {
            Answer::Pick(r.choices.iter().position(|s| s.contains("Station volunteer")).unwrap_or(0))
        }
    }));
    g.humans.add(P1, h.clone());
    let o = opts(&mut g, s).remove(0);
    let result = sim_core::ai::act::perform(&mut g, P0, o.act.as_ref().unwrap());
    assert!(
        matches!(result, Ok(true)),
        "{result:?}; asked: {:?}; told: {:?}",
        h.asked().iter().map(|r| (&r.kind, &r.prompt, &r.choices)).collect::<Vec<_>>(),
        h.told()
    );
    assert!(h.told().is_empty(), "{:?}", h.told());
    assert_eq!(sim_core::engine::values::epow(&g, t), 8, "pump resolved before station");
    assert_eq!(charge(&g, s), 8, "station uses current power, not power when tapped");
}

#[test]
fn seriema_search_survives_source_destruction() {
    let mut g = tbl();
    let c = card(&g, "Tymna the Weaver");
    g.player_mut(P0).library = vec![c];
    hand(&mut g, P1, &["Disenchant"]);
    pool(&mut g, P1, &[('W', 2)]);
    let h = seat(&mut g, P1, vec![act(json!({"do":"cast", "card":0})), by_text("The Seriema"), pass()]);
    let s = perm(&mut g, P0, "The Seriema");
    assert!(h.told().is_empty(), "{:?}", h.told());
    assert!(!g.perm(s).on_bf, "Disenchant resolved");
    assert!(g.player(P0).hand.contains(&c), "an ETB trigger exists independently of its source");
}

#[test]
fn seriema_opposition_agent_controller_chooses_search() {
    let mut g = tbl();
    perm(&mut g, P1, "Opposition Agent");
    let cs = ["Tymna the Weaver", "Kambal, Consul of Allocation"].map(|n| card(&g, n));
    g.player_mut(P0).library = cs.to_vec();
    let mine = person(&mut g, |r| r.choices.iter().position(|s| s.contains("Tymna")));
    let theirs = std::sync::Arc::new(ScriptHuman::from_fn(|r| {
        if r.kind == "priority" {
            Answer::Action(json!({"do":"pass"}))
        } else {
            Answer::Pick(r.choices.iter().position(|s| s.contains("Kambal")).unwrap())
        }
    }));
    g.humans.add(P1, theirs.clone());
    perm(&mut g, P0, "The Seriema");
    assert!(!mine.asked().iter().any(|r| r.kind == "search"), "Agent's controller controls the searching player");
    assert!(theirs.asked().iter().any(|r| r.kind == "search"));
    assert!(g.player(P1).agent_ids.contains(&cs[1]));
}

#[test]
fn kambal_countered_spell_still_triggers() {
    let mut g = tbl();
    perm(&mut g, P0, "Kambal, Consul of Allocation");
    hand(&mut g, P0, &["Counterspell"]);
    pool(&mut g, P0, &[('U', 2)]);
    let mut used = false;
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" && r.prompt.contains("casts Sol Ring") && !used {
            used = true;
            Answer::Action(json!({"do":"cast", "card":0}))
        } else if r.kind == "priority" {
            Answer::Action(json!({"do":"pass"}))
        } else {
            Answer::Pick(0)
        }
    }));
    g.humans.add(P0, h.clone());
    let c = hand(&mut g, P1, &["Sol Ring"])[0];
    assert!(
        !cast::cast_card(&mut g, P1, c, "hand", Ctx::default()).unwrap(),
        "Sol Ring countered: {:?}",
        h.asked().iter().map(|r| &r.prompt).collect::<Vec<_>>()
    );
    assert_eq!((g.player(P0).life, g.player(P1).life), (42, 38));
}

#[test]
fn kambal_split_second_and_flash_noncreature() {
    for name in ["Angel's Grace", "Dictate of Erebos"] {
        let mut g = tbl();
        perm(&mut g, P0, "Kambal, Consul of Allocation");
        cast_free(&mut g, P1, name, "hand");
        assert_eq!((g.player(P0).life, g.player(P1).life), (42, 38), "{name}");
    }
}

#[test]
fn yoshimaru_partner_command_zone_casts() {
    let mut g = tbl();
    let y = card(&g, Y);
    let tymna = card(&g, "Tymna the Weaver");
    g.player_mut(P0).cmd = y;
    g.player_mut(P0).set_partner(Some(tymna));
    assert!(cast::cast_card(&mut g, P0, y, "cmd", Ctx::default()).unwrap());
    let yp = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).cd == Some(y)).unwrap();
    assert!(cast::cast_card(&mut g, P0, tymna, "cmd", Ctx::default()).unwrap());
    assert_eq!(g.perm(yp).plus, 1);
    assert!(!g.player(P0).cmd_in_zone && !g.player(P0).cmd2_in_zone);
}

#[test]
fn meadow_exactly_three_life_offers_payment() {
    let mut g = tbl();
    let h = person(&mut g, |_| Some(0));
    g.player_mut(P0).life = 3;
    let _ = play_meadow(&mut g);
    assert!(h.asked().iter().any(|r| r.prompt.contains("pay 3 life")));
    assert_eq!(g.player(P0).life, 0);
    assert!(!g.player(P0).alive, "paying to zero loses at the next state-based check");
}

#[test]
fn station_person_menu_excludes_combat_timing() {
    let mut g = tbl();
    let s = seriema(&mut g);
    token(&mut g, P0, 3);
    person(&mut g, |_| None);
    g.step = Step::Combat;
    let ways = sim_core::human::human::abilities_of(&mut g, P0, s).unwrap();
    assert!(!ways.iter().any(|(s, _)| s.contains("station")), "station only as a sorcery");
}

#[test]
fn yoshimaru_fetched_legendary_land_counts_once() {
    let mut g = tbl();
    let y = perm(&mut g, P0, Y);
    let c = card(&g, "Minas Tirith");
    g.player_mut(P0).library = vec![c];
    assert_eq!(sim_core::impls::karn_engines::fetch_land(&mut g, P0, false, false).unwrap(), Some(c));
    assert_eq!(g.perm(y).plus, 1);
    zones::landfall(&mut g, P0).unwrap();
    assert_eq!(g.perm(y).plus, 1);
}

#[test]
fn yoshimaru_land_played_from_exile_counts_once() {
    let mut g = tbl();
    let y = perm(&mut g, P0, Y);
    let c = card(&g, "Minas Tirith");
    g.player_mut(P0).library = vec![c];
    sim_core::engine::tutors::jeskas_will(&mut g, P0).unwrap();
    assert!(g.player(P0).lands.iter().any(|&l| g.perm(l).card() == c));
    assert_eq!(g.perm(y).plus, 1);
}

#[test]
fn yoshimaru_oracle_of_mul_daya_top_land_counts_once() {
    let mut g = tbl();
    let y = perm(&mut g, P0, Y);
    perm(&mut g, P0, "Oracle of Mul Daya");
    let c = card(&g, "Minas Tirith");
    let filler = card(&g, "Plains");
    g.player_mut(P0).library = vec![filler; 20];
    g.player_mut(P0).library.push(c);
    turn::more_lands(&mut g, P0).unwrap();
    assert!(g.player(P0).lands.iter().any(|&l| g.perm(l).card() == c));
    assert_eq!(g.perm(y).plus, 1);
}

#[test]
fn meadow_cast_creature_face_through_full_database_name() {
    let mut g = tbl();
    let ring = perm(&mut g, P1, "Sol Ring");
    cast_free(&mut g, P0, W, "hand");
    assert!(!g.perm(ring).on_bf);
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).name == W && g.is_creature(m)));
    assert_eq!(g.player(P0).lands_played, 0);
}

#[test]
fn seriema_opposition_agent_ai_takes_legal_search_card() {
    let mut g = tbl();
    perm(&mut g, P1, "Opposition Agent");
    let c = card(&g, "Tymna the Weaver");
    g.player_mut(P0).library = vec![c];
    perm(&mut g, P0, "The Seriema");
    assert!(g.player(P0).library.is_empty() && g.player(P0).hand.is_empty());
    assert!(g.player(P1).agent_ids.contains(&c));
}

#[test]
fn kambal_gain_still_happens_when_opponent_loses_game() {
    let mut g = tbl();
    perm(&mut g, P0, "Kambal, Consul of Allocation");
    g.player_mut(P1).life = 1;
    let c = hand(&mut g, P1, &["Sol Ring"])[0];
    let _ = cast::cast_card(&mut g, P1, c, "hand", Ctx::default()).unwrap();
    assert!(!g.player(P1).alive);
    assert_eq!(g.player(P0).life, 42);
}

#[test]
fn minas_earlier_turn_and_legendary_noncreature_do_not_qualify() {
    let mut g = tbl();
    perm(&mut g, P0, "The One Ring");
    let l = land(&mut g, "Minas Tirith");
    assert!(g.perm(l).tapped);
    person(&mut g, |_| None);
    g.perm_mut(l).tapped = false;
    let st = g.turn_stamp();
    g.player_mut(P0).atk_count = (Some(st), 2);
    g.player_mut(P0).pool.add('W', 2);
    g.active = Some(P1);
    assert!(sim_core::human::alt::land_alts(&mut g, P0, l).unwrap().is_empty());
}

#[test]
fn seriema_indestructible_exact_scope() {
    let mut g = tbl();
    let s = seriema(&mut g);
    let own = perm(&mut g, P0, Y);
    let other = perm(&mut g, P1, "Tymna the Weaver");
    let normal = token(&mut g, P0, 2);
    g.perm_mut(own).tapped = true;
    g.perm_mut(other).tapped = true;
    g.perm_mut(normal).tapped = true;
    station(&mut g, s, 7);
    g.perm_mut(s).tapped = true;
    assert!(sim_core::engine::values::indestructible(&g, own));
    for m in [s, other, normal] {
        assert!(!sim_core::engine::values::indestructible(&g, m));
    }
    g.perm_mut(own).tapped = false;
    assert!(!sim_core::engine::values::indestructible(&g, own));
}

#[test]
fn station_uses_last_known_power_when_tapped_creature_leaves() {
    let mut g = tbl();
    let s = seriema(&mut g);
    let t = token(&mut g, P0, 6);
    g.perm_mut(t).name = "Station volunteer".into();
    g.perm_mut(t).sick = true;
    hand(&mut g, P1, &["Swords to Plowshares"]);
    pool(&mut g, P1, &[('W', 1)]);
    let h = seat(&mut g, P1, vec![act(json!({"do":"cast", "card":0})), by_text("Station volunteer"), pass()]);
    let o = opts(&mut g, s).remove(0);
    assert!(run(&mut g, &o));
    assert!(h.told().is_empty(), "{:?}", h.told());
    assert!(!g.perm(t).on_bf);
    assert_eq!(charge(&g, s), 6);
}

#[test]
fn yoshimaru_ancient_greenwarden_doubles_legendary_land_trigger() {
    let mut g = tbl();
    let y = perm(&mut g, P0, Y);
    perm(&mut g, P0, "Ancient Greenwarden");
    land(&mut g, "Minas Tirith");
    assert_eq!(g.perm(y).plus, 2, "a land entering caused Yoshimaru to trigger; Greenwarden doubles it");
}

#[test]
fn yoshimaru_land_trigger_waits_for_current_resolution() {
    let mut g = tbl();
    let y = perm(&mut g, P0, Y);
    let c = card(&g, "Minas Tirith");
    g.player_mut(P0).library = vec![c];
    // Fetching the land is part of another spell/ability's resolution.
    g.resolving += 1;
    sim_core::impls::karn_engines::fetch_land(&mut g, P0, false, false).unwrap();
    assert_eq!(g.perm(y).plus, 0, "the land-enter trigger cannot resolve inside the search ability");
    g.resolving -= 1;
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert_eq!(g.perm(y).plus, 1);
}

#[test]
fn meadow_land_drop_does_not_destroy_an_opponents_artifact() {
    let mut g = tbl();
    let ring = perm(&mut g, P1, "Sol Ring");
    person(&mut g, |_| Some(1));
    let l = play_meadow(&mut g);
    assert!(
        g.perm(ring).on_bf,
        "the land face has no enters trigger: Witch Enchanter's destroy is the creature face's"
    );
    assert!(g.perm(l).on_bf);
}

#[test]
fn yoshimaru_land_trigger_is_doubled_by_delney() {
    let mut g = tbl();
    perm(&mut g, P0, "Delney, Streetwise Lookout");
    let y = perm(&mut g, P0, Y);
    assert_eq!(g.perm(y).plus, 0);
    land(&mut g, "Minas Tirith");
    assert_eq!(g.perm(y).plus, 2, "a trigger of a creature with power 2 or less triggers an additional time");
}

#[test]
fn yoshimaru_hushbringer_and_panharmonicon_do_not_touch_a_plain_legendary_land() {
    let mut g = tbl();
    let y = perm(&mut g, P0, Y);
    perm(&mut g, P0, "Panharmonicon");
    perm(&mut g, P1, "Hushbringer");
    land(&mut g, "Minas Tirith");
    assert_eq!(g.perm(y).plus, 1, "a land that is not a creature: one trigger, not silenced, not doubled");
}

#[test]
fn yoshimaru_counts_a_legendary_land_under_blood_moon() {
    let mut g = tbl();
    let y = perm(&mut g, P0, Y);
    perm(&mut g, P1, "Blood Moon");
    land(&mut g, "Minas Tirith");
    assert_eq!(g.perm(y).plus, 1, "Blood Moon takes the land's abilities, not its legendary supertype");
}

#[test]
fn yoshimaru_counts_each_legendary_land_once_across_two_land_drops() {
    let mut g = tbl();
    let y = perm(&mut g, P0, Y);
    land(&mut g, "Minas Tirith");
    land(&mut g, "Plains");
    land(&mut g, "Eiganjo, Seat of the Empire");
    assert_eq!(g.perm(y).plus, 2);
}

#[test]
fn meadow_land_face_is_colourless_and_has_no_creature_types() {
    let mut g = tbl();
    person(&mut g, |_| Some(1));
    let l = play_meadow(&mut g);
    let types = sim_core::engine::chars::card_types(&g, l);
    assert!(types.has(Types::LAND) && !types.has(Types::CREATURE));
    assert!(sim_core::engine::chars::subtypes(&g, l).is_empty(), "no Human, no Warlock");
    assert_eq!(sim_core::engine::chars::colors(&g, l), sim_core::cards::Colors::NONE);
    assert!(g.player(P0).lands.contains(&l) && !g.player(P0).perms.contains(&l));
}

#[test]
fn meadow_played_by_the_ai_is_only_a_land_and_destroys_nothing() {
    let mut g = tbl();
    let ring = perm(&mut g, P1, "Sol Ring");
    g.player_mut(P0).hand.clear();
    let c = hand(&mut g, P0, &[W])[0];
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    let o = f(&mut g, c, P0, Some(false)).unwrap().remove(0);
    assert!(run(&mut g, &o));
    let l = *g.player(P0).lands.last().unwrap();
    assert!(!g.is_creature(l));
    assert!(g.perm(ring).on_bf, "the land face has no enters trigger");
}

#[test]
fn meadow_with_spelunking_enters_untapped_and_pays_no_life_for_the_ai_either() {
    let mut g = tbl();
    perm(&mut g, P0, "Spelunking");
    g.player_mut(P0).hand.clear();
    let c = hand(&mut g, P0, &[W])[0];
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    let o = f(&mut g, c, P0, Some(false)).unwrap().remove(0);
    assert!(run(&mut g, &o));
    let l = *g.player(P0).lands.last().unwrap();
    assert!(!g.perm(l).tapped);
    assert_eq!(g.player(P0).life, 40, "no life paid: Spelunking makes it untapped anyway");
}

#[test]
fn seriema_is_a_5_5_flier_at_seven_counters_and_a_plain_artifact_below() {
    let mut g = tbl();
    let s = seriema(&mut g);
    station(&mut g, s, 6);
    assert!(!g.is_creature(s) && !sim_core::engine::chars::has_keyword(&g, s, "flying"));
    station(&mut g, s, 1);
    assert_eq!(charge(&g, s), 7);
    assert!(g.is_creature(s));
    assert!(sim_core::engine::chars::has_keyword(&g, s, "flying"));
    assert_eq!((sim_core::engine::values::epow(&g, s), sim_core::engine::values::etgh(&g, s)), (5, 5));
    g.perm_mut(s).data.set(DataKey::Charge, Val::Int(3));
    assert!(!g.is_creature(s) && !sim_core::engine::chars::has_keyword(&g, s, "flying"));
}

#[test]
fn seriema_ai_does_not_station_once_it_is_a_creature_but_a_person_may() {
    let mut g = tbl();
    let s = seriema(&mut g);
    station(&mut g, s, 7);
    let t = token(&mut g, P0, 2);
    g.perm_mut(t).sick = true;
    assert!(opts(&mut g, s).is_empty(), "the AI has nothing to gain");
    person(&mut g, |r| r.choices.iter().position(|s| s.contains("power 2")));
    let o = opts(&mut g, s).remove(0);
    assert!(run(&mut g, &o));
    assert_eq!(charge(&g, s), 9);
}
