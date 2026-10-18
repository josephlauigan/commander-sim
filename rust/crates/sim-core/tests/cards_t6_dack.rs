//! Tier 6 batch `dack` (impls/t6_dack.rs) on hand-built positions. Seat 0 plays Dack's deck, seat 1 the seph deck.

use sim_core::ids::{PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    let mut g = table(&["dack-fayden-white-stax", "seph"]);
    g.active = Some(P0);
    g
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn in_exile(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).exile.iter().any(|&c| &*g.db.get(c).name == name)
}

// ======================================================== Fiend Hunter
#[test]
fn fiend_hunter_exiles_the_best_creature_and_returns_it_when_it_leaves() {
    let mut g = game();
    let titan = perm(&mut g, P1, "Grave Titan");
    let hunter = perm(&mut g, P0, "Fiend Hunter");
    assert!(!g.perm(titan).on_bf, "Grave Titan is exiled");
    assert!(in_exile(&g, P1, "Grave Titan"));
    sim_core::engine::zones::die(&mut g, hunter, "test").unwrap();
    assert!(on_bf(&g, P1, "Grave Titan").is_some(), "it returns under its owner's control");
    assert!(!in_exile(&g, P1, "Grave Titan"));
}

#[test]
fn fiend_hunter_never_takes_its_own_creature() {
    let mut g = game();
    let mine = perm(&mut g, P0, "Grave Titan");
    let _hunter = perm(&mut g, P0, "Fiend Hunter");
    assert!(g.perm(mine).on_bf);
}

// ======================================================== All-Fates Stalker
#[test]
fn stalker_exiles_a_non_assassin_and_returns_it_when_it_leaves() {
    let mut g = game();
    let titan = perm(&mut g, P1, "Grave Titan");
    let stalker = perm(&mut g, P0, "All-Fates Stalker");
    assert!(!g.perm(titan).on_bf && in_exile(&g, P1, "Grave Titan"));
    sim_core::engine::zones::die(&mut g, stalker, "test").unwrap();
    assert!(on_bf(&g, P1, "Grave Titan").is_some(), "it returns at once, under its owner's control");
}

#[test]
fn stalker_leaves_assassins_alone() {
    let mut g = game();
    let other = perm(&mut g, P1, "All-Fates Stalker");
    let _mine = perm(&mut g, P0, "All-Fates Stalker");
    assert!(g.perm(other).on_bf, "an Assassin is not a legal target");
}

// ======================================================== Touch the Spirit Realm
#[test]
fn touch_exiles_an_opposing_creature_until_it_leaves() {
    let mut g = game();
    let titan = perm(&mut g, P1, "Grave Titan");
    let touch = perm(&mut g, P0, "Touch the Spirit Realm");
    assert!(!g.perm(titan).on_bf && in_exile(&g, P1, "Grave Titan"));
    sim_core::engine::zones::die(&mut g, touch, "test").unwrap();
    assert!(on_bf(&g, P1, "Grave Titan").is_some());
}

#[test]
fn touch_channel_returns_the_target_at_the_end_step_under_its_owner() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 2, false);
    let titan = perm(&mut g, P1, "Grave Titan");
    let c = hand(&mut g, P0, &["Touch the Spirit Realm"])[0];
    assert!(sim_core::impls::t6_dack::touch_channel(&mut g, P0, c, titan).unwrap());
    assert!(!g.perm(titan).on_bf && in_exile(&g, P1, "Grave Titan"));
    assert!(g.player(P0).gy.contains(&c), "the card was discarded as the cost");
    sim_core::impls::galadriel::eot_returns(&mut g).unwrap();
    assert!(on_bf(&g, P1, "Grave Titan").is_some(), "it returns under its owner's control at the end step");
}

#[test]
fn touch_channel_needs_its_mana_and_a_legal_target() {
    let mut g = game();
    let titan = perm(&mut g, P1, "Grave Titan");
    let c = hand(&mut g, P0, &["Touch the Spirit Realm"])[0];
    assert!(!sim_core::impls::t6_dack::touch_channel(&mut g, P0, c, titan).unwrap(), "no mana");
    lands(&mut g, P0, "Plains", 2, false);
    let land_like = token(&mut g, P1, 1);
    sim_core::engine::zones::die(&mut g, land_like, "test").unwrap();
    assert!(!sim_core::impls::t6_dack::touch_channel(&mut g, P0, c, land_like).unwrap(), "target is gone");
}

// ======================================================== Radiant Solar
fn room(g: &Game, p: PlayerId) -> u32 {
    g.player(p).grudge.get(&sim_core::sym::intern("dungeon_room")).copied().unwrap_or(0.0) as u32
}

#[test]
fn solar_ventures_for_itself_and_other_nontoken_creatures_but_not_tokens() {
    let mut g = game();
    let _solar = perm(&mut g, P0, "Radiant Solar");
    assert_eq!(room(&g, P0), 1, "entering itself ventures");
    let _t = token(&mut g, P0, 1);
    assert_eq!(room(&g, P0), 1, "a token doesn't");
    let _x = perm(&mut g, P0, "Grave Titan");
    assert_eq!(room(&g, P0), 2);
    let _y = perm(&mut g, P1, "Grave Titan");
    assert_eq!(room(&g, P0), 2, "an opponent's creature doesn't");
}

#[test]
fn four_ventures_drain_each_opponent_once_and_draw_then_start_over() {
    let mut g = game();
    let (l0, l1) = (g.player(P0).life, g.player(P1).life);
    let hand0 = g.player(P0).hand.len();
    for _ in 0..4 {
        sim_core::impls::t6_dack::venture(&mut g, P0).unwrap();
    }
    assert_eq!(g.player(P1).life, l1 - 1);
    assert_eq!(g.player(P0).life, l0 + 1);
    assert_eq!(g.player(P0).hand.len(), hand0 + 1, "Temple of Dumathoin draws");
    assert_eq!(room(&g, P0), 0, "completed");
    sim_core::impls::t6_dack::venture(&mut g, P0).unwrap();
    assert_eq!(room(&g, P0), 1, "the next venture starts a new dungeon");
}

#[test]
fn solar_discard_ventures_and_gains_three() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 1, false);
    let c = hand(&mut g, P0, &["Radiant Solar"])[0];
    let life = g.player(P0).life;
    assert!(sim_core::impls::t6_dack::solar_discard(&mut g, P0, c).unwrap());
    assert_eq!(room(&g, P0), 1);
    assert_eq!(g.player(P0).life, life + 3);
    assert!(g.player(P0).gy.contains(&c));
    assert!(!sim_core::impls::t6_dack::solar_discard(&mut g, P0, c).unwrap(), "no longer in hand");
}

#[test]
fn a_donated_solar_still_counts_creatures_that_entered_before_the_donation() {
    let mut g = game();
    let solar = perm(&mut g, P0, "Radiant Solar");
    let early = perm(&mut g, P0, "Grave Titan");
    sim_core::impls::marchesa::steal(&mut g, P1, solar, false);
    sim_core::impls::t6_dack::mark_donated(&mut g, solar, P0);
    assert!(sim_core::impls::t6_dack::controlled_when(&g, solar, P0, early));
    let late = perm(&mut g, P0, "Grave Titan");
    assert!(!sim_core::impls::t6_dack::controlled_when(&g, solar, P0, late), "entered after the donation");
}

// ======================================================== Preston, the Vanisher
fn illusions_of(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).on_bf && g.perm(m).token && g.perm(m).ttypes.contains(&"illusion"))
        .collect()
}

#[test]
fn preston_copies_a_creature_that_was_not_cast_as_a_0_1_white_illusion() {
    let mut g = game();
    let _preston = perm(&mut g, P0, "Preston, the Vanisher");
    let titan = perm(&mut g, P0, "Serra Angel");
    let ill = illusions_of(&g, P0);
    assert_eq!(ill.len(), 1, "perm() puts it onto the battlefield without casting it");
    assert!(g.perm(ill[0]).token);
    assert_eq!((g.perm(ill[0]).pow, g.perm(ill[0]).tgh), (0, 1));
    assert!(g.perm(ill[0]).colors.has('W'));
    assert!(g.perm(titan).on_bf);
}

#[test]
fn preston_ignores_tokens_cast_creatures_and_opponents() {
    let mut g = game();
    let _preston = perm(&mut g, P0, "Preston, the Vanisher");
    let _t = token(&mut g, P0, 2);
    let _o = perm(&mut g, P1, "Grave Titan");
    assert!(illusions_of(&g, P0).is_empty());
    // a cast creature
    lands(&mut g, P0, "Plains", 4, false);
    let c = hand(&mut g, P0, &["Serra Angel"])[0];
    sim_core::engine::cast::cast_card(&mut g, P0, c, sim_core::sym::intern("hand"), Default::default()).unwrap();
    assert!(illusions_of(&g, P0).is_empty(), "a cast creature isn't copied");
}

#[test]
fn preston_copy_enters_trigger_fires() {
    let mut g = game();
    let _preston = perm(&mut g, P0, "Preston, the Vanisher");
    let theirs = perm(&mut g, P1, "Grave Titan");
    // Fiend Hunter enters uncast: its Illusion copy also exiles (a second banisher aimed at the same opposing board)
    let _fh = perm(&mut g, P0, "Fiend Hunter");
    assert!(!g.perm(theirs).on_bf, "the original (or its copy) exiled the Titan");
    assert_eq!(illusions_of(&g, P0).len(), 1);
}

#[test]
fn preston_sacrifices_five_illusions_to_exile_a_nonland_permanent() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 2, false);
    let preston = perm(&mut g, P0, "Preston, the Vanisher");
    let theirs = perm(&mut g, P1, "Grave Titan");
    assert!(!sim_core::impls::t6_dack::preston_exile(&mut g, preston, P0, theirs).unwrap(), "no Illusions yet");
    for _ in 0..5 {
        perm(&mut g, P0, "Serra Angel");
    }
    assert_eq!(illusions_of(&g, P0).len(), 5);
    assert!(sim_core::impls::t6_dack::preston_exile(&mut g, preston, P0, theirs).unwrap());
    assert!(illusions_of(&g, P0).is_empty());
    assert!(!g.perm(theirs).on_bf);
}

// ======================================================== Dack Fayden, Helping Hand
fn stack_library(g: &mut Game, p: PlayerId, top_last: &[&str]) {
    // top_last: bottom ... top
    let cs: Vec<_> = top_last.iter().map(|n| card(g, n)).collect();
    g.player_mut(p).library = cs;
}

#[test]
fn dack_flips_one_creature_per_opponent_and_gives_each_away_goaded() {
    let mut g = table(&["dack-fayden-white-stax", "seph", "sheoldred", "yshtola"]);
    g.active = Some(P0);
    // top of the library (the end) first: land, Serra Angel, land, Serra Angel, ...
    stack_library(
        &mut g,
        P0,
        &["Grave Titan", "Serra Angel", "Plains", "Serra Angel", "Plains", "Serra Angel", "Plains"],
    );
    let _dack = perm(&mut g, P0, "Dack Fayden, Helping Hand");
    let mine: Vec<_> = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).on_bf && g.is_creature(m)).collect();
    assert_eq!(mine.len(), 1, "only Dack stays with his controller");
    let mut owners = vec![];
    for q in [PlayerId(1), PlayerId(2), PlayerId(3)] {
        let n = g.player(q).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).orig == P0).count();
        owners.push(n);
    }
    assert_eq!(owners, vec![1, 1, 1], "each opponent gains control of exactly one");
    for q in [PlayerId(1), PlayerId(2), PlayerId(3)] {
        for &m in &g.player(q).perms {
            if g.perm(m).orig == P0 {
                assert!(g.perm(m).data.truthy(sim_core::state::DataKey::Goaded), "goaded");
            }
        }
    }
    // the revealed non-creature cards and the rest of the library are shuffled back: 4 cards remain
    assert_eq!(g.player(P0).library.len(), 4);
}

#[test]
fn dack_with_too_few_creatures_flips_what_remains() {
    let mut g = table(&["dack-fayden-white-stax", "seph", "sheoldred", "yshtola"]);
    g.active = Some(P0);
    stack_library(&mut g, P0, &["Plains", "Serra Angel", "Plains"]);
    let _dack = perm(&mut g, P0, "Dack Fayden, Helping Hand");
    let n: usize = (1..4).map(|i| g.player(PlayerId(i)).perms.iter().filter(|&&m| g.perm(m).orig == P0).count()).sum();
    assert_eq!(n, 1);
}

#[test]
fn dacks_pile_banisher_can_exile_a_creature_that_was_just_donated() {
    let mut g = table(&["dack-fayden-white-stax", "seph", "sheoldred", "yshtola"]);
    g.active = Some(P0);
    // two opponents' worth... three creatures: Fiend Hunter, Serra Angel, Serra Angel (top is the end)
    stack_library(&mut g, P0, &["Serra Angel", "Serra Angel", "Fiend Hunter"]);
    let _dack = perm(&mut g, P0, "Dack Fayden, Helping Hand");
    let exiled: usize = g.players.iter().map(|pl| pl.exile.len()).sum();
    assert_eq!(exiled, 1, "the Hunter (its enters trigger is its first controller's) exiled a donated creature");
}

#[test]
fn dack_pile_solar_and_preston_see_each_other_before_the_donation() {
    let mut g = table(&["dack-fayden-white-stax", "seph", "sheoldred", "yshtola"]);
    g.active = Some(P0);
    stack_library(&mut g, P0, &["Radiant Solar", "Serra Angel", "Preston, the Vanisher"]);
    let _dack = perm(&mut g, P0, "Dack Fayden, Helping Hand");
    let room = g.player(P0).grudge.get(&sim_core::sym::intern("dungeon_room")).copied().unwrap_or(0.0);
    assert!(room >= 2.0, "Solar ventures for itself and the creatures that entered with it (got room {room})");
    let ill = g
        .players
        .iter()
        .flat_map(|pl| pl.perms.iter())
        .filter(|&&m| g.perm(m).token && g.perm(m).ttypes.contains(&"illusion"))
        .count();
    assert!(ill >= 1, "Preston copies the creatures that entered with it");
}

// ======================================================== Gideon of the Trials
#[test]
fn gideon_plus_locks_the_best_opposing_creatures_damage_and_gains_loyalty() {
    let mut g = game();
    let titan = perm(&mut g, P1, "Grave Titan");
    let gideon = perm(&mut g, P0, "Gideon of the Trials");
    assert_eq!(g.perm(gideon).loyalty, Some(3));
    let ab = sim_core::impls::common::walker_options(&mut g, gideon, P0, Some(false)).unwrap();
    let plus = ab.iter().find(|o| o.label.ends_with("+1")).expect("a +1 option");
    if let Some(sim_core::hooks::Action::Ability { src, f, arg }) = &plus.act {
        assert!(f(&mut g, *src, P0, *arg).unwrap());
    } else {
        panic!("not an ability")
    }
    assert_eq!(g.perm(gideon).loyalty, Some(4));
    assert!(sim_core::cardcode::dovin_blocked(&g, titan), "Grave Titan's combat damage is prevented");
}

#[test]
fn gideon_zero_makes_him_a_4_4_indestructible_creature_until_end_of_turn() {
    let mut g = game();
    let gideon = perm(&mut g, P0, "Gideon of the Trials");
    g.perm_mut(gideon).sick = false;
    let opts = sim_core::impls::common::walker_options(&mut g, gideon, P0, Some(false)).unwrap();
    let animate = opts.iter().find(|o| o.label.ends_with("+0")).expect("a 0 option");
    // there are two 0 abilities: pick the animation by running each and checking
    let mut animated = false;
    for o in opts.iter().filter(|o| o.label.ends_with("+0") || o.label.ends_with("0")) {
        let _ = animate;
        if let Some(sim_core::hooks::Action::Ability { src, f, arg }) = &o.act {
            let mut h = g.clone();
            f(&mut h, *src, P0, *arg).unwrap();
            if h.is_creature(gideon) {
                animated = true;
                assert_eq!((h.perm(gideon).pow, h.perm(gideon).tgh), (4, 4));
                assert!(sim_core::engine::values::indestructible(&h, gideon));
            }
        }
    }
    assert!(animated, "one of the 0 abilities animates him");
}

#[test]
fn gideon_stops_being_a_creature_at_end_of_turn_and_the_other_zero_makes_an_emblem() {
    let mut g = game();
    let gideon = perm(&mut g, P0, "Gideon of the Trials");
    g.perm_mut(gideon).sick = false;
    let opts = sim_core::impls::common::walker_options(&mut g, gideon, P0, Some(false)).unwrap();
    let (mut ended, mut emblem) = (false, false);
    for o in opts.iter().filter(|o| o.label.ends_with("0")) {
        if let Some(sim_core::hooks::Action::Ability { src, f, arg }) = &o.act {
            let mut h = g.clone();
            f(&mut h, *src, P0, *arg).unwrap();
            if h.is_creature(gideon) {
                let cd = h.perm(gideon).cd.unwrap();
                let end = h.registry.get(cd).unwrap().end_step.unwrap();
                end(&mut h, gideon, P0).unwrap();
                ended = !h.is_creature(gideon) && !sim_core::engine::values::indestructible(&h, gideon);
            }
            if h.player(P0).emblems.contains(&"gideon_trials") {
                emblem = true;
            }
        }
    }
    assert!(ended, "back to a plain planeswalker");
    assert!(emblem, "the other 0 ability gives the emblem");
}

// ======================================================== Paladin Class
#[test]
fn paladin_class_taxes_opposing_spells_on_its_controllers_turn_only() {
    let mut g = game();
    let class = perm(&mut g, P0, "Paladin Class");
    let cd = g.perm(class).cd.unwrap();
    let tax = g.registry.get(cd).unwrap().cost.unwrap();
    let bolt = card(&g, "Swords to Plowshares");
    assert_eq!(tax(&g, class, P1, bolt), 1);
    assert_eq!(tax(&g, class, P0, bolt), 0, "its own controller's spells aren't taxed");
    g.active = Some(P1);
    assert_eq!(tax(&g, class, P1, bolt), 0, "only during its controller's turn");
}

#[test]
fn paladin_class_levels_one_at_a_time_and_level_three_pumps_an_attacker() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 8, false);
    let class = perm(&mut g, P0, "Paladin Class");
    let a = perm(&mut g, P0, "Serra Angel");
    let b = perm(&mut g, P0, "Serra Angel");
    assert!(!sim_core::impls::t6_dack::paladin_level_up_for_test(&mut g, class, P0, 3), "no skipping level 2");
    assert!(sim_core::impls::t6_dack::paladin_level_up_for_test(&mut g, class, P0, 2));
    assert!(sim_core::impls::t6_dack::paladin_level_up_for_test(&mut g, class, P0, 3));
    let cd = g.perm(class).cd.unwrap();
    let attack = g.registry.get(cd).unwrap().attack.unwrap();
    let before = sim_core::engine::values::epow(&g, a);
    attack(&mut g, class, P0, &[a, b], P1).unwrap();
    let after = (sim_core::engine::values::epow(&g, a), sim_core::engine::values::epow(&g, b));
    assert!(after.0 == before + 1 || after.1 == before + 1, "one attacker gets +1/+1 (one other attacker)");
    assert!(sim_core::engine::combat::double_strike(&g, a) || sim_core::engine::combat::double_strike(&g, b));
}

#[test]
fn preston_with_felidar_guardian_is_capped_instead_of_overflowing() {
    use sim_core::sym::intern;
    let mut g = game();
    let _solar = perm(&mut g, P0, "Radiant Solar");
    let _preston = perm(&mut g, P0, "Preston, the Vanisher");
    let _felidar = perm(&mut g, P0, "Felidar Guardian"); // enters uncast: no stack overflow
    let n = g.player(P0).grudge.get(&intern("preston_cap_n")).copied().unwrap_or(0.0);
    let cap = sim_core::impls::t6_dack::PRESTON_CAP;
    assert!(n >= 1.0 && n <= cap + 1.0, "copies counted and bounded, got {n}");
    let room = g.player(P0).grudge.get(&intern("dungeon_room")).copied().unwrap_or(0.0);
    let done = g.player(P0).grudge.get(&intern("dungeons_completed")).copied().unwrap_or(0.0);
    assert!(room + done > 0.0, "Solar's ventures were counted");
    // at the cap, Preston stops: a new uncast creature gets no copy
    let stamp = g.player(P0).grudge.get(&intern("preston_cap_round")).copied().unwrap();
    g.player_mut(P0).grudge.insert(intern("preston_cap_round"), stamp);
    g.player_mut(P0).grudge.insert(intern("preston_cap_n"), cap);
    let before =
        g.player(P0).perms.iter().filter(|&&m| g.perm(m).token && g.perm(m).ttypes.contains(&"illusion")).count();
    let _x = perm(&mut g, P0, "Serra Angel");
    let after =
        g.player(P0).perms.iter().filter(|&&m| g.perm(m).token && g.perm(m).ttypes.contains(&"illusion")).count();
    assert_eq!(before, after, "capped: no more copies this turn");
}

// ======================================================== control change and goad (engine verbs)
#[test]
fn gain_control_until_end_of_turn_untaps_hastes_and_returns_at_the_end_step() {
    use sim_core::engine::control::{Duration, gain_control, return_borrowed};
    let mut g = game();
    let m = perm(&mut g, P1, "Serra Angel");
    g.perm_mut(m).tapped = true;
    assert!(gain_control(&mut g, P0, m, Duration::EndOfTurn));
    assert_eq!(g.perm(m).owner, P0);
    assert!(!g.perm(m).tapped && !g.perm(m).sick);
    assert!(g.player(P0).perms.contains(&m) && !g.player(P1).perms.contains(&m));
    return_borrowed(&mut g, P0);
    assert_eq!(g.perm(m).owner, P1);
    assert!(g.player(P1).perms.contains(&m) && !g.player(P0).perms.contains(&m));
}

#[test]
fn permanent_gain_control_is_sick_and_never_returns() {
    use sim_core::engine::control::{Duration, gain_control, return_borrowed};
    let mut g = game();
    let m = perm(&mut g, P1, "Serra Angel");
    gain_control(&mut g, P0, m, Duration::EndOfTurn);
    gain_control(&mut g, P0, m, Duration::Permanent); // the longer effect wins
    return_borrowed(&mut g, P0);
    assert_eq!(g.perm(m).owner, P0);
}

#[test]
fn forever_goad_outlasts_the_goaders_turns_but_a_normal_goad_lapses() {
    use sim_core::engine::control::{GoadFor, goad};
    let mut g = game();
    let a = perm(&mut g, P1, "Serra Angel");
    let b = perm(&mut g, P1, "Serra Angel");
    goad(&mut g, P0, a, GoadFor::Forever);
    goad(&mut g, P0, b, GoadFor::NextTurn);
    g.player_mut(P0).turns += 2;
    assert_eq!(sim_core::impls::t0_zurgo::goaded_by(&g, a), Some(P0));
    assert_eq!(sim_core::impls::t0_zurgo::goaded_by(&g, b), None);
}

#[test]
fn dacks_donated_creatures_stay_goaded_after_two_of_his_turns() {
    let mut g = table(&["dack-fayden-white-stax", "seph", "sheoldred", "yshtola"]);
    g.active = Some(P0);
    stack_library(&mut g, P0, &["Serra Angel", "Plains", "Serra Angel", "Plains", "Serra Angel", "Plains"]);
    let _dack = perm(&mut g, P0, "Dack Fayden, Helping Hand");
    g.player_mut(P0).turns += 2;
    let mut n = 0;
    for q in [PlayerId(1), PlayerId(2), PlayerId(3)] {
        for &m in &g.player(q).perms {
            if g.perm(m).orig == P0 {
                assert_eq!(sim_core::impls::t0_zurgo::goaded_by(&g, m), Some(P0));
                n += 1;
            }
        }
    }
    assert_eq!(n, 3);
}
