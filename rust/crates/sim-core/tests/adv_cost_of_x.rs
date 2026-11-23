//! Adversarial checks for 83ac6e3d..2d901769. Source and generated data remain unchanged.

use sim_core::engine::{cast, mana, values, zones};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const PACT: &str = "Damnable Pact";
const TEN: &str = "Enduring Tenacity";
const SHEO: &str = "Sheoldred, the Apocalypse";
const PEER: &str = "Peer into the Abyss";
const BRILLIANCE: &str = "Cost of Brilliance";
const SPELLS: [&str; 4] = [PACT, "Painful Lesson", PEER, BRILLIANCE];

fn game(seats: usize) -> Game {
    table(&["sheoldred", "seph", "cloud", "karn"][..seats])
}

fn options(g: &mut Game, c: CardId) -> Vec<Opt> {
    g.registry.get(c).unwrap().hand_options.unwrap()(g, c, P0, Some(false)).unwrap()
}

fn run(g: &mut Game, opt: &Opt) -> bool {
    let Some(Action::Plan { f, arg }) = opt.act else { panic!("expected plan: {opt:?}") };
    f(g, P0, arg).unwrap()
}

fn x_in_label(opt: &Opt) -> i32 {
    let (_, rest) = opt.label.split_once("draws ").or_else(|| opt.label.split_once("draw ")).unwrap();
    rest.split_whitespace().next().unwrap().trim_end_matches(')').parse().unwrap()
}

#[test]
fn x_zero_matches_cost_of_across_the_entire_database_and_boards() {
    for board in [None, Some("Jet Medallion"), Some("Sphere of Resistance"), Some("K'rrik, Son of Yawgmoth")] {
        let mut g = game(3);
        if let Some(name) = board {
            perm(&mut g, P0, name);
        }
        for d in g.db.iter() {
            assert_eq!(mana::cost_of_x(&g, P0, d.id, 0), mana::cost_of(&g, P0, d.id), "{}: {board:?}", d.name);
        }
    }
}

#[test]
fn negative_x_is_zero_including_i32_min_across_the_database() {
    let g = game(3);
    for d in g.db.iter() {
        for x in [-1, i32::MIN] {
            assert_eq!(mana::cost_of_x(&g, P0, d.id, x), mana::cost_of(&g, P0, d.id), "{}: {x}", d.name);
        }
    }
}

#[test]
fn single_x_costs_have_only_zero_or_one_steps_through_all_floors() {
    for reducers in 0..=3 {
        for tax in [false, true] {
            for minimum in [false, true] {
                let mut g = game(3);
                for _ in 0..reducers {
                    perm(&mut g, P0, "Jet Medallion");
                }
                if tax {
                    perm(&mut g, P1, "Sphere of Resistance");
                }
                if minimum {
                    perm(&mut g, P1, "Trinisphere");
                }
                for name in [PACT, "Exsanguinate", "Diabolic Revelation"] {
                    let c = card(&g, name);
                    for x in 0..20 {
                        let a = mana::cost_of_x(&g, P0, c, x);
                        let b = mana::cost_of_x(&g, P0, c, x + 1);
                        assert!(
                            b.0 >= a.0 && b.0 <= a.0 + 1,
                            "{name}: {reducers}/{tax}/{minimum}, X={x}: {a:?}->{b:?}"
                        );
                        assert_eq!(a.1, b.1);
                        if x >= 4 {
                            assert_eq!(b.0, a.0 + 1, "past all floors");
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn non_x_input_is_an_unconditional_generic_increment_documented_by_the_helper() {
    // Finding, rather than a failure: the helper explicitly documents adding x to the printed generic part,
    // without inspecting xcount. Callers must pass zero for a non-X card (and scaled X for multiple symbols).
    let g = game(3);
    for d in g.db.iter().filter(|d| d.xcount == 0 && !d.land) {
        assert_eq!(mana::cost_of_x(&g, P0, d.id, 0), mana::cost_of(&g, P0, d.id));
    }
    let c = card(&g, "Sign in Blood"); // {B}{B}: no printed generic component
    for x in 0..=20 {
        assert_eq!(mana::cost_of_x(&g, P0, c, x), (x as u32, "BB".into()));
    }
}

#[test]
fn large_representable_x_keeps_its_value() {
    let g = game(3);
    for name in [PACT, "Diabolic Revelation"] {
        let c = card(&g, name);
        let x = i32::MAX / 2;
        assert_eq!(mana::cost_of_x(&g, P0, c, x).0, x as u32 + g.db.get(c).generic);
    }
    assert_eq!(mana::cost_of_x(&g, P0, card(&g, PACT), i32::MAX).0, i32::MAX as u32);
}

#[test]
fn maximum_x_plus_printed_generic_does_not_overflow_i32() {
    // BUG: a valid u32 cost (i32::MAX + 3) panics in the initial signed addition.
    let g = game(3);
    assert_eq!(mana::cost_of_x(&g, P0, card(&g, "Diabolic Revelation"), i32::MAX).0, i32::MAX as u32 + 3);
}

#[test]
fn maximum_x_plus_a_tax_does_not_overflow_i32() {
    // BUG: adding Sphere's tax to i32::MAX panics despite the result fitting the returned u32.
    let mut g = game(3);
    perm(&mut g, P1, "Sphere of Resistance");
    assert_eq!(mana::cost_of_x(&g, P0, card(&g, PACT), i32::MAX).0, i32::MAX as u32 + 1);
}

fn pact_position(mana: usize, reducers: usize, tax: bool, minimum: bool, krrik: bool) -> Game {
    let mut g = game(3);
    g.player_mut(P0).life = 100;
    lands(&mut g, P0, "Swamp", mana, false);
    for _ in 0..reducers {
        perm(&mut g, P0, "Jet Medallion");
    }
    if tax {
        perm(&mut g, P1, "Sphere of Resistance");
    }
    if minimum {
        perm(&mut g, P1, "Trinisphere");
    }
    if krrik {
        perm(&mut g, P0, "K'rrik, Son of Yawgmoth");
    }
    hand(&mut g, P0, &[PACT]);
    g
}

#[test]
fn pact_ai_maximum_matches_exhaustive_payability_in_512_positions() {
    let mut failures = vec![];
    for n in 0..=15 {
        for reducers in 0..=3 {
            for tax in [false, true] {
                for minimum in [false, true] {
                    for krrik in [false, true] {
                        let mut g = pact_position(n, reducers, tax, minimum, krrik);
                        let c = card(&g, PACT);
                        // The largest possible X here is 15 lands + 3 reducers; 32 includes an unpaid sentinel.
                        let payable = |g: &Game, x| {
                            let (gn, pips) = mana::cost_of_x(g, P0, c, x);
                            mana::can_pay_spell(g, P0, c, gn, &pips, false)
                        };
                        assert!(!payable(&g, 32));
                        let best = (1..=32).rev().find(|&x| payable(&g, x)).unwrap_or(0);
                        let opts = options(&mut g, c);
                        let expected_self = (best >= 2).then_some(best.min(6));
                        let observed = opts.first().map(x_in_label);
                        let label = format!("mana={n} reducers={reducers} tax={tax} trini={minimum} krrik={krrik}");
                        if observed != expected_self {
                            failures.push(format!(
                                "{label}: brute X={best}, self expected {expected_self:?}, got {observed:?}"
                            ));
                        }
                        // A kill at exactly best exposes X=1 as well as maxima above the self-draw cap of six.
                        // It also avoids confusing a deliberate smaller killing X with the private maximum.
                        perm(&mut g, P0, SHEO);
                        g.player_mut(P1).life = if best > 0 { 3 * best } else { 40 };
                        g.player_mut(PlayerId(2)).life = 1000;
                        let opts = options(&mut g, c);
                        let observed = opts.first().map(x_in_label);
                        if observed != (best > 0).then_some(best) {
                            failures.push(format!("{label}: brute X={best}, kill option {observed:?}"));
                        }
                        for o in &opts {
                            assert!(payable(&g, x_in_label(o)), "over-count: {label}: {}", o.label);
                        }
                    }
                }
            }
        }
    }
    // BUG: the search assumes floors no longer bite at X=total mana; excess reducers invalidate that bound.
    assert!(failures.is_empty(), "{} mismatches:\n{}", failures.len(), failures.join("\n"));
}

#[test]
fn phyrexian_and_hybrid_pips_and_life_payments_survive_x_pricing() {
    let mut g = game(3);
    let dismember = card(&g, "Dismember");
    assert_eq!(mana::cost_of_x(&g, P0, dismember, 0), (1, "".into()));
    assert_eq!(mana::phyrexian_life(&g, P0, dismember, ""), 4);
    lands(&mut g, P0, "Plains", 2, false);
    let guildmage = card(&g, "Azorius Guildmage");
    assert_eq!(mana::cost_of_x(&g, P0, guildmage, 0), (0, "WW".into()));
    let mut g = pact_position(0, 3, false, false, true);
    let c = card(&g, PACT);
    assert_eq!(mana::cost_of_x(&g, P0, c, 3), (0, "".into()));
    assert_eq!(mana::phyrexian_life(&g, P0, c, ""), 4);
    // Force the valid free X=3 plan through the real plan callback, independently of the broken maximum search.
    lands(&mut g, P0, "Swamp", 4, false);
    let o = options(&mut g, c).remove(0);
    let Some(Action::Plan { f, arg }) = o.act else { unreachable!() };
    for m in g.player(P0).lands.clone() {
        g.perm_mut(m).tapped = true;
    }
    let before = g.player(P0).life;
    assert!(f(&mut g, P0, (arg & 0xffff_ffff) | (3_i64 << 32)).unwrap());
    assert_eq!(g.player(P0).life, before - 4 - 3);
    assert_eq!(g.player(P0).hand.len(), 3);
}

#[test]
fn kaza_reduction_consumes_x_and_expires_with_its_turn_stamp() {
    let mut g = game(3);
    let c = card(&g, PACT);
    g.player_mut(P0).t0s.kaza = Some((g.turn_stamp(), 4));
    perm(&mut g, P0, "Jet Medallion");
    for x in 0..=20 {
        assert_eq!(mana::cost_of_x(&g, P0, c, x).0, (x - 5).max(0) as u32);
    }
    g.round += 1;
    assert_eq!(mana::cost_of_x(&g, P0, c, 8).0, 7);
}

#[test]
fn eminence_dsl_hooks_and_self_cost_all_contribute() {
    let mut g = table(&["ur-dragon-five-color-dragons", "seph", "cloud"]);
    let c = card(&g, "Earthquake Dragon"); // {14}{G}; other Dragons' total mana value reduces it.
    let dragon = perm(&mut g, P0, "Goldspan Dragon");
    let mv = g.db.get(g.perm(dragon).cd.unwrap()).cmc as i32;
    perm(&mut g, P0, "Dragonlord's Servant"); // DSL -1
    perm(&mut g, P1, "Sphere of Resistance"); // hook +1
    assert_eq!(sim_core::impls::t45_urdragon::eminence(&g, P0, c), -1);
    assert_eq!(sim_core::cardcode::self_cost(&g, P0, c), -mv);
    for x in 0..=20 {
        assert_eq!(mana::cost_of_x(&g, P0, c, x).0, (14 + x - 1 - 1 + 1 - mv) as u32);
    }
}

#[test]
fn graveyard_and_creature_count_reducers_keep_the_existing_floors() {
    let mut g = game(3);
    let bolt = card(&g, "Lightning Bolt");
    let lesson = card(&g, "Painful Lesson");
    g.player_mut(P0).gy.extend([bolt, bolt, lesson]);
    let eris = card(&g, "Eris, Roar of the Storm");
    let dawning = card(&g, "The Dawning Archaic");
    assert_eq!(mana::cost_of_x(&g, P0, eris, 0), (4, "UR".into()));
    assert_eq!(mana::cost_of_x(&g, P0, dawning, 0), (7, "".into()));
    for owner in [P0, P1, PlayerId(2)] {
        token(&mut g, owner, 1);
    }
    let act = card(&g, "Blasphemous Act");
    assert_eq!(mana::cost_of_x(&g, P0, act, 0), (5, "R".into()));
    g.player_mut(PlayerId(2)).alive = false;
    assert_eq!(mana::cost_of_x(&g, P0, act, 0), (6, "R".into()));
    let stage = card(&g, "Light Up the Stage");
    g.player_mut(P0).hit_turn = g.player(P0).turns as i32;
    assert_eq!(mana::cost_of_x(&g, P0, stage, 0), (0, "R".into()));
}

#[test]
fn colored_reductions_free_dragons_and_agent_mana_are_preserved() {
    let mut g = table(&["ur-dragon-five-color-dragons", "seph", "cloud"]);
    let gold = card(&g, "Goldspan Dragon");
    let morophon = perm(&mut g, P0, "Morophon, the Boundless");
    g.perm_mut(morophon).data.set(DataKey::Ctype, Val::Str("dragon"));
    assert_eq!(mana::cost_of_x(&g, P0, gold, 0), (2, "R".into()));
    perm(&mut g, P0, "Dracogenesis");
    assert_eq!(mana::cost_of_x(&g, P0, gold, 20), (0, "".into()));
    let mut g = game(3);
    perm(&mut g, P0, "Defiler of Dreams");
    let c = card(&g, "Consecrated Sphinx"); // {4}{U}{U}; Defiler removes one U.
    assert_eq!(mana::cost_of_x(&g, P0, c, 0), (4, "U".into()));
    let c = card(&g, "Lightning Bolt");
    g.player_mut(P0).agent_ids.push(c);
    assert_eq!(mana::cost_of_x(&g, P0, c, 0), (1, "".into()));
}

#[test]
fn multiple_x_symbols_require_caller_scaling_under_the_documented_raw_increment_contract() {
    // Finding: unlike human::cast_inner, this helper does not multiply x by the printed xcount.
    let g = game(3);
    let c = card(&g, "Astral Cornucopia"); // {X}{X}{X}
    assert_eq!(g.db.get(c).xcount, 3);
    assert_eq!(mana::cost_of_x(&g, P0, c, 2).0, 2);
    assert_eq!(mana::cost_of_x(&g, P0, c, 3 * 2).0, 6);
}

#[test]
fn minimum_is_not_lost_to_a_later_self_reduction() {
    // BUG: self_cost is subtracted after Trinisphere's floor, allowing a payable cost below three mana.
    let mut g = game(3);
    let c = card(&g, "Shadow of Mortality"); // {13}{B}{B}, reduced by life below 40.
    g.player_mut(P0).life = 20;
    lands(&mut g, P0, "Swamp", 3, false);
    perm(&mut g, P1, "Trinisphere");
    assert_eq!(mana::cost_of_x(&g, P0, c, 0), (1, "BB".into()));
}

#[test]
fn free_casts_flute_and_commander_tax_keep_the_existing_cost_pipeline() {
    let mut g = game(3);
    let c = hand(&mut g, P0, &[PACT])[0];
    g.player_mut(P0).emblems.push("tamiyo");
    assert_eq!(mana::cost_of_x(&g, P0, c, 20), (0, "".into()));
    g.player_mut(P0).emblems.clear();
    let flute = perm(&mut g, P1, "Disruptor Flute");
    g.flutes.clear();
    g.flutes.push((flute, c));
    assert_eq!(mana::cost_of_x(&g, P0, c, 4), (7, "BB".into()));
    let cmd = g.player(P0).cmd;
    g.player_mut(P0).tax = 6;
    assert_eq!(mana::cost_of_x(&g, P0, cmd, 0).0, g.db.get(cmd).generic + 6);
    hand(&mut g, P0, &[SHEO]);
    assert_eq!(mana::cost_of_x(&g, P0, cmd, 0).0, g.db.get(cmd).generic);
}

#[test]
fn dead_shalai_and_veil_targets_fizzle_in_three_and_four_seat_tables() {
    for seats in [3, 4] {
        for name in SPELLS {
            for shield in ["dead", "shalai", "veil"] {
                let mut g = game(seats);
                let target = PlayerId((seats - 1) as u8);
                let host = token(&mut g, P0, 1);
                let c = hand(&mut g, P0, &[name])[0];
                match shield {
                    "dead" => g.player_mut(target).alive = false,
                    "shalai" => {
                        perm(&mut g, target, "Shalai, Voice of Plenty");
                    }
                    _ => g.player_mut(target).veil_turn = Some(g.turn_stamp()),
                }
                let before: Vec<_> = g.players.iter().map(|p| (p.hand.len(), p.life, p.library.len())).collect();
                cast::cast_card(
                    &mut g,
                    P0,
                    c,
                    "hand",
                    Ctx { face: Some(target), host: Some(host), x: 3, ..Ctx::default() },
                )
                .unwrap();
                assert!(!g.over, "three or more seats keep the game running");
                for (i, p) in g.players.iter().enumerate() {
                    assert_eq!(
                        (p.hand.len(), p.life, p.library.len()),
                        (before[i].0 - usize::from(i == 0), before[i].1, before[i].2),
                        "{seats}/{name}/{shield}/seat {i}"
                    );
                }
                assert!(g.player(P0).gy.contains(&c), "{name}");
                assert_eq!(values::epow(&g, host), if name == BRILLIANCE { 2 } else { 1 });
            }
        }
    }
}

// Painful Lesson is absent from the deck's plays list. Reuse the callback Peer actually exposes: both cards use
// exactly lesson_plan and the same packed CardId/target format; no plan data or source is changed.
fn target_plan(g: &mut Game, name: &str, target: PlayerId) -> Opt {
    let provider = if name == "Painful Lesson" { PEER } else { name };
    let c = hand(g, P0, &[provider])[0];
    let mut o = options(g, c).remove(0);
    if name == "Painful Lesson" {
        g.player_mut(P0).hand.retain(|&x| x != c);
        let lesson = hand(g, P0, &[name])[0];
        let Some(Action::Plan { f, .. }) = o.act else { unreachable!() };
        o.act = Some(Action::Plan { f, arg: lesson.0 as i64 | ((target.0 as i64 + 1) << 16) });
    }
    let Some(Action::Plan { arg, .. }) = o.act else { unreachable!() };
    assert_eq!(((arg >> 16) & 0xffff) - 1, target.0 as i64, "{}", o.label);
    o
}

#[test]
fn stale_plans_reject_death_and_new_hexproof_before_any_payment() {
    for seats in [3, 4] {
        for name in SPELLS {
            for shield in ["dead", "shalai", "veil"] {
                let mut g = game(seats);
                perm(&mut g, P0, SHEO);
                lands(&mut g, P0, "Swamp", 8, false);
                g.player_mut(P1).life = 6;
                g.player_mut(P1).library.truncate(10);
                let o = target_plan(&mut g, name, P1);
                match shield {
                    "dead" => g.player_mut(P1).alive = false,
                    "shalai" => {
                        perm(&mut g, P1, "Shalai, Voice of Plenty");
                    }
                    _ => g.player_mut(P1).veil_turn = Some(g.turn_stamp()),
                }
                let before: Vec<_> = g.players.iter().map(|p| (p.hand.clone(), p.life, p.library.clone())).collect();
                assert!(!run(&mut g, &o), "{seats}/{name}/{shield}");
                for (i, p) in g.players.iter().enumerate() {
                    assert_eq!((&p.hand, p.life, &p.library), (&before[i].0, before[i].1, &before[i].2));
                }
                assert!(g.player(P0).lands.iter().all(|&m| !g.perm(m).tapped));
                assert!(!g.over);
            }
        }
    }
}

#[test]
fn self_targets_remain_legal_through_hexproof_for_direct_casts_and_plans() {
    for name in SPELLS {
        for plan in [false, true] {
            let mut g = game(3);
            lands(&mut g, P0, "Swamp", 8, false);
            let c = hand(&mut g, P0, &[name])[0];
            let o = if plan {
                let provider = if name == "Painful Lesson" { PEER } else { name };
                let pc = card(&g, provider);
                if pc != c {
                    g.player_mut(P0).hand.push(pc);
                }
                let mut o = options(&mut g, pc).remove(0);
                if pc != c {
                    g.player_mut(P0).hand.retain(|&x| x != pc);
                    let Some(Action::Plan { f, .. }) = o.act else { unreachable!() };
                    o.act = Some(Action::Plan { f, arg: c.0 as i64 });
                }
                Some(o)
            } else {
                None
            };
            perm(&mut g, P0, "Shalai, Voice of Plenty");
            g.player_mut(P0).veil_turn = Some(g.turn_stamp());
            let lib = g.player(P0).library.len();
            let draws = if name == PEER {
                lib.div_ceil(2)
            } else if name == PACT {
                o.as_ref().map(x_in_label).unwrap_or(3) as usize
            } else {
                2
            };
            if let Some(o) = o {
                assert!(run(&mut g, &o), "{name}");
            } else {
                cast::cast_card(&mut g, P0, c, "hand", Ctx { face: Some(P0), x: 3, ..Ctx::default() }).unwrap();
            }
            assert_eq!(g.player(P0).hand.len(), draws, "{name}, plan={plan}");
            assert_eq!(g.player(P0).life, if name == PEER { 20 } else { 40 - draws as i32 });
            assert!(g.player(P0).gy.contains(&c));
        }
    }
}

#[test]
fn brilliance_does_not_redirect_its_illegal_creature_target() {
    // BUG: the resolver substitutes best_creature for an explicitly named creature that has left the battlefield.
    let mut g = game(3);
    let gone = token(&mut g, P0, 1);
    let survivor = token(&mut g, P0, 2);
    zones::die(&mut g, gone, "destroy").unwrap();
    g.player_mut(P1).alive = false;
    let c = hand(&mut g, P0, &[BRILLIANCE])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { face: Some(P1), host: Some(gone), ..Ctx::default() }).unwrap();
    assert_eq!(values::epow(&g, survivor), 2, "both targets illegal: no replacement target for the counter");
    assert!(g.player(P0).gy.contains(&c));
}

#[test]
fn options_do_not_offer_an_already_hexproof_player_as_the_only_plan() {
    // BUG: Peer/Brilliance option generation ignores hexproof, then the new plan guard rejects that sole option.
    let mut failures = vec![];
    for name in [PEER, BRILLIANCE] {
        let mut g = game(3);
        perm(&mut g, P0, SHEO);
        perm(&mut g, P1, "Shalai, Voice of Plenty");
        g.player_mut(P1).life = 6;
        g.player_mut(P1).library.truncate(10);
        lands(&mut g, P0, "Swamp", 8, false);
        let c = hand(&mut g, P0, &[name])[0];
        for o in options(&mut g, c) {
            if !run(&mut g, &o) {
                failures.push(o.label);
            }
        }
    }
    assert!(failures.is_empty(), "unexecutable options on an unchanged board: {failures:?}");
}

fn tenacities(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf && &*g.perm(m).name == TEN).collect()
}

#[test]
fn physical_tenacity_returns_once_and_the_returned_enchantment_stays_dead() {
    let mut g = game(3);
    let t = perm(&mut g, P0, TEN);
    zones::die(&mut g, t, "destroy").unwrap();
    let back = tenacities(&g, P0);
    assert_eq!(back.len(), 1);
    assert_ne!(back[0], t);
    assert!(!g.is_creature(back[0]));
    assert!(g.perm(back[0]).data.truthy(DataKey::NotCreature));
    zones::die(&mut g, back[0], "destroy").unwrap();
    assert!(tenacities(&g, P0).is_empty());
}

fn check_dying_image(owner: PlayerId) {
    // BUG: a nontoken copy has Tenacity's death trigger; phys must select its physical card, not suppress the trigger.
    // Oracle's "return it" refers to the dying object, so the brief's "nothing returns" expectation is incorrect.
    let mut g = game(3);
    perm(&mut g, PlayerId(2), TEN);
    let image = perm(&mut g, owner, "Phantasmal Image");
    assert_eq!(&*g.perm(image).name, TEN);
    let physical = card(&g, "Phantasmal Image");
    assert_eq!(g.perm(image).phys, Some(physical));
    let real = card(&g, TEN);
    g.player_mut(owner).gy.push(real);
    zones::die(&mut g, image, "destroy").unwrap();
    assert!(g.player(owner).gy.contains(&real), "the unrelated real card stays in the graveyard");
    let back: Vec<_> = g
        .player(owner)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            let p = g.perm(m);
            p.on_bf && (p.phys == Some(physical) || p.cd == Some(physical))
        })
        .collect();
    assert_eq!(back.len(), 1, "owner={owner:?}: the Image card itself must return");
    assert!(!g.is_creature(back[0]));
}

#[test]
fn our_dying_image_returns_itself_and_leaves_the_real_card_in_the_graveyard() {
    // BUG: the phys guard suppresses the physical Image's own mandatory return trigger.
    check_dying_image(P0);
}

#[test]
fn an_opponents_dying_image_returns_itself_and_leaves_the_real_card_in_the_graveyard() {
    // BUG: the same phys guard suppresses an opponent-owned Image's mandatory return trigger.
    check_dying_image(P1);
}

#[test]
fn mirage_mirror_returns_itself_without_reanimating_an_unrelated_tenacity() {
    // BUG: a Mirror that was a creature copy of Tenacity returns its own card, not nothing.
    let mut g = game(3);
    perm(&mut g, P1, TEN);
    lands(&mut g, P0, "Swamp", 2, false);
    let mirror = perm(&mut g, P0, "Mirage Mirror");
    let c = card(&g, "Mirage Mirror");
    let opts = g.registry.get(c).unwrap().options.unwrap()(&mut g, mirror, P0, Some(false)).unwrap();
    let Some(Action::Ability { src, f, arg }) = opts[0].act else { panic!("Mirror ability") };
    assert!(f(&mut g, src, P0, arg).unwrap());
    assert_eq!(&*g.perm(mirror).name, TEN);
    let real = card(&g, TEN);
    g.player_mut(P0).gy.push(real);
    zones::die(&mut g, mirror, "destroy").unwrap();
    assert!(g.player(P0).gy.contains(&real));
    assert!(tenacities(&g, P0).is_empty());
    let mirrors: Vec<_> =
        g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).on_bf && g.perm(m).cd == Some(c)).collect();
    assert_eq!(mirrors.len(), 1, "the physical Mirror must return");
    assert!(!g.is_creature(mirrors[0]));
}

#[test]
fn copying_a_returned_enchantment_does_not_copy_its_type_changing_effect() {
    // CR 707.2: the return ability's type-changing effect is not part of copiable values.
    let mut g = game(3);
    let t = perm(&mut g, P1, TEN);
    zones::die(&mut g, t, "destroy").unwrap();
    let back = tenacities(&g, P1)[0];
    assert!(!g.is_creature(back));
    lands(&mut g, P0, "Swamp", 2, false);
    let mirror = perm(&mut g, P0, "Mirage Mirror");
    // The public ability callback can target the returned enchantment even when the AI would not offer it.
    let c = card(&g, "Mirage Mirror");
    let temp = perm(&mut g, PlayerId(2), TEN);
    let opts = g.registry.get(c).unwrap().options.unwrap()(&mut g, mirror, P0, Some(false)).unwrap();
    let Some(Action::Ability { src, f, .. }) = opts[0].act else { panic!("Mirror ability") };
    zones::exile_perm(&mut g, temp).unwrap();
    assert!(f(&mut g, src, P0, back.0 as i64).unwrap());
    assert!(g.is_creature(mirror), "the return effect is not copiable");
}
