//! The karn deck's cards that were below Full (model-gaps-a1): each fixed card on a hand-built position, with its
//! Oracle text as the expectation.

mod play_common;
use play_common::*;
use sim_core::ai::{act, brain};
use sim_core::engine::{combat, turn, zones};
use sim_core::hooks::Opt;
use sim_core::human::human::human_main;
use sim_core::impls::{fixes, karn, karn_cards, t6_godo, t6_stax};
use sim_core::state::{DataKey, Val};

fn tbl() -> Game {
    table(&["karn", "kefka", "sauron"])
}

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.contains(&card(g, name))
}

// ======================================================== Darksteel Citadel
#[test]
fn darksteel_citadel_is_indestructible_and_a_plain_land_is_not() {
    let mut g = tbl();
    let cit = lands(&mut g, P0, "Darksteel Citadel", 1, false)[0];
    let plain = lands(&mut g, P0, "Wastes", 1, false)[0];
    fixes::destroy_land(&mut g, P0, cit).unwrap();
    assert!(g.player(P0).lands.contains(&cit), "Indestructible: it survives destroy");
    fixes::destroy_land(&mut g, P0, plain).unwrap();
    assert!(!g.player(P0).lands.contains(&plain), "Wastes is destroyed");
    assert!(in_gy(&g, P0, "Wastes"));
}

// ======================================================== Blightsteel Colossus
#[test]
fn blightsteel_milled_or_discarded_is_shuffled_into_the_library() {
    let mut g = tbl();
    let c = take(&mut g, P0, "Blightsteel Colossus");
    g.player_mut(P0).library.push(c); // on top
    let n = g.player(P0).library.len();
    zones::mill(&mut g, P0, 1).unwrap();
    assert!(!g.player(P0).gy.contains(&c), "not the graveyard");
    assert!(g.player(P0).library.contains(&c), "back in the library");
    assert_eq!(g.player(P0).library.len(), n, "a card milled, the same card shuffled in");
    // discarded
    let c = take(&mut g, P0, "Blightsteel Colossus");
    g.player_mut(P0).hand.push(c);
    zones::discard_cards(&mut g, P0, &[c]).unwrap();
    assert!(!g.player(P0).gy.contains(&c));
    assert!(!g.player(P0).hand.contains(&c));
    assert!(g.player(P0).library.contains(&c));
}

#[test]
fn blightsteel_infect_damage_to_a_blocker_is_minus_one_counters() {
    let mut g = tbl();
    let b = perm(&mut g, P0, "Blightsteel Colossus");
    g.perm_mut(b).plus = -8; // 3/3 for the test
    let titan = perm(&mut g, P1, "Grave Titan");
    let mut tot = 0;
    combat::resolve_blocks(&mut g, P0, vec![b], P1, vec![(b, titan)], &mut tot).unwrap();
    assert!(g.perm(b).on_bf, "indestructible: it survives the Titan's six");
    assert!(g.perm(titan).on_bf, "three infect damage does not kill a 6/6");
    assert_eq!(g.perm(titan).plus, -3, "three -1/-1 counters, not damage");
}

// ======================================================== Cityscape Leveler
#[test]
fn an_unearthed_leveler_is_exiled_instead_of_dying_or_being_bounced() {
    let mut g = tbl();
    let m = perm(&mut g, P0, "Cityscape Leveler");
    g.perm_mut(m).data.set(DataKey::Unearth, Val::Bool(true));
    zones::die(&mut g, m, "destroy").unwrap();
    let c = card(&g, "Cityscape Leveler");
    assert!(g.player(P0).exile.contains(&c), "exiled instead");
    assert!(!g.player(P0).gy.contains(&c));
    // a bounce
    let m2 = perm(&mut g, P0, "Cityscape Leveler");
    g.perm_mut(m2).data.set(DataKey::Unearth, Val::Bool(true));
    zones::bounce(&mut g, m2).unwrap();
    assert!(!g.player(P0).hand.contains(&c), "not the hand");
    // a hard-cast Leveler goes to the graveyard as usual
    let m3 = perm(&mut g, P0, "Cityscape Leveler");
    zones::die(&mut g, m3, "destroy").unwrap();
    assert!(g.player(P0).gy.contains(&c));
}

// ======================================================== Ugin, the Ineffable
fn ugin_plus(g: &mut Game) -> (PermId, sim_core::ids::CardId, PermId) {
    let ugin = perm(g, P0, "Ugin, the Ineffable");
    let top = *g.player(P0).library.last().unwrap();
    let os = options(g, ugin, P0, Some(false));
    let plus = os.iter().find(|o| o.label.contains("+1")).unwrap().clone();
    assert!(run(g, P0, &plus));
    let spirit = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token).unwrap();
    (ugin, top, spirit)
}

#[test]
fn ugins_card_returns_when_the_spirit_is_exiled_even_after_ugin_is_gone() {
    let mut g = tbl();
    let (ugin, top, spirit) = ugin_plus(&mut g);
    assert!(g.player(P0).exile.contains(&top));
    // Ugin leaves first: the card stays in exile ("when that token leaves the battlefield")
    zones::die(&mut g, ugin, "destroy").unwrap();
    assert!(g.player(P0).exile.contains(&top), "still waiting for the Spirit");
    assert!(!g.player(P0).hand.contains(&top));
    // the Spirit is exiled (not dying): the card goes to hand
    zones::exile_perm(&mut g, spirit).unwrap();
    assert!(g.player(P0).hand.contains(&top));
    assert!(!g.player(P0).exile.contains(&top));
}

#[test]
fn ugins_card_returns_when_the_spirit_is_bounced() {
    let mut g = tbl();
    let (_ugin, top, spirit) = ugin_plus(&mut g);
    zones::bounce(&mut g, spirit).unwrap();
    assert!(g.player(P0).hand.contains(&top));
}

// ======================================================== Liberator, Urza's Battlethopter
#[test]
fn liberator_gives_flash_to_artifact_and_colourless_spells_and_the_ai_wipes_with_all_is_dust_at_end_of_turn() {
    let mut g = tbl();
    let aid = card(&g, "All Is Dust");
    assert!(!karn_cards::liberator_flash(&g, P0, aid), "no Liberator yet");
    perm(&mut g, P0, "Liberator, Urza's Battlethopter");
    assert!(karn_cards::liberator_flash(&g, P0, aid), "a colourless sorcery");
    assert!(karn_cards::liberator_flash(&g, P0, card(&g, "Sol Ring")), "an artifact");
    assert!(!karn_cards::liberator_flash(&g, P0, card(&g, "Grave Titan")), "a coloured spell");
    // All Is Dust is a sorcery: at the end of the turn before ours, only Liberator lets the AI cast it
    hand(&mut g, P0, &["All Is Dust"]);
    lands(&mut g, P0, "Wastes", 7, false);
    perm(&mut g, P1, "Grave Titan");
    perm(&mut g, P2, "Lyra Dawnbringer");
    let s = brain::Situation::new(&g, P0);
    let with = brain::wipe_options(&g, P0, &s, true);
    assert!(with.iter().any(|o| o.label == "All Is Dust"), "{:?}", with.iter().map(|o| &o.label).collect::<Vec<_>>());
    let mut g2 = tbl();
    hand(&mut g2, P0, &["All Is Dust"]);
    lands(&mut g2, P0, "Wastes", 7, false);
    perm(&mut g2, P1, "Grave Titan");
    perm(&mut g2, P2, "Lyra Dawnbringer");
    let s2 = brain::Situation::new(&g2, P0);
    assert!(brain::wipe_options(&g2, P0, &s2, true).is_empty(), "no flash without Liberator");
}

// ======================================================== Liquimetal Torque
#[test]
fn the_ai_makes_ugin_an_artifact_before_combat_so_karn_grows() {
    let mut g = tbl();
    let k = perm(&mut g, P0, "Karn, Legacy Reforged");
    g.perm_mut(k).sick = false;
    let ugin = perm(&mut g, P0, "Ugin, the Spirit Dragon");
    let t = perm(&mut g, P0, "Liquimetal Torque");
    let before = sim_core::engine::values::epow(&g, k);
    let os = options(&mut g, t, P0, Some(false));
    let o = os
        .iter()
        .find(|o| o.label.contains("Ugin"))
        .unwrap_or_else(|| panic!("{:?}", os.iter().map(|o| &o.label).collect::<Vec<_>>()));
    assert!(o.utility > -3.0, "worth more than stopping");
    assert!(run(&mut g, P0, o));
    assert!(sim_core::impls::t5::is_artifact(&g, ugin));
    assert!(g.perm(t).tapped);
    assert!(sim_core::engine::values::epow(&g, k) >= before + 2, "Karn is as big as Ugin's mana value now");
    assert_eq!(karn::max_mv(&g, P0), 8);
    // with no Karn the AI keeps tapping it for mana
    let mut g = tbl();
    perm(&mut g, P0, "Ugin, the Spirit Dragon");
    let t = perm(&mut g, P0, "Liquimetal Torque");
    let os = options(&mut g, t, P0, Some(false));
    assert!(os.iter().all(|o| o.utility < -3.0));
}

// ======================================================== Mirrorworks
#[test]
fn mirrorworks_copies_an_artifact_land_played_as_the_land_drop() {
    let mut g = tbl();
    perm(&mut g, P0, "Mirrorworks");
    lands(&mut g, P0, "Wastes", 2, false);
    let c = take(&mut g, P0, "Darksteel Citadel");
    turn::play_land_card(&mut g, P0, c, "plays").unwrap();
    let cits: Vec<PermId> = g.player(P0).lands.iter().copied().filter(|&l| g.perm(l).cd == Some(c)).collect();
    assert_eq!(cits.len(), 2, "the land and its token copy");
    assert_eq!(cits.iter().filter(|&&l| g.perm(l).token).count(), 1);
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.perm(l).tapped).count(), 2, "{{2}} was paid");
    // the token copy is indestructible too, and leaves no card in the graveyard
    let tok = cits.into_iter().find(|&l| g.perm(l).token).unwrap();
    fixes::destroy_land(&mut g, P0, tok).unwrap();
    assert!(g.player(P0).lands.contains(&tok));
    // without the {2} the land comes alone
    let mut g = tbl();
    perm(&mut g, P0, "Mirrorworks");
    let c = take(&mut g, P0, "Darksteel Citadel");
    turn::play_land_card(&mut g, P0, c, "plays").unwrap();
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.perm(l).cd == Some(c)).count(), 1);
}

// ======================================================== Mishra's Factory and Foundry
fn animated_worker(g: &mut Game, p: PlayerId, name: &str) -> PermId {
    let l = lands(g, p, name, 1, false)[0];
    let m = sim_core::impls::lands::animate(g, p, l, 2, 2, &[], false);
    let x = g.perm_mut(m);
    x.ttypes.push("artifact");
    x.ttypes.push("assembly-worker");
    m
}

/// a 2/3 attacker for P1
fn two_three(g: &mut Game) -> PermId {
    let a = token(g, P1, 2);
    g.perm_mut(a).tgh = 3;
    a
}

#[test]
fn the_ai_taps_a_blocking_factory_to_survive_and_kill() {
    let mut g = tbl();
    let a = two_three(&mut g);
    let f = animated_worker(&mut g, P0, "Mishra's Factory");
    assert!(sim_core::engine::values::etgh(&g, f) == 2);
    karn::combat_pumps(&mut g, P1, &[a], P0, &[(a, f)]).unwrap();
    assert!(g.perm(f).tapped, "{{T}}");
    assert_eq!(g.perm(f).eot_pt, (1, 1), "+1/+1 on itself");
    let mut tot = 0;
    combat::resolve_blocks(&mut g, P1, vec![a], P0, vec![(a, f)], &mut tot).unwrap();
    assert!(g.perm(f).on_bf, "3/3 survives 2 damage");
    assert!(!g.perm(a).on_bf, "and kills the 2/3");
}

#[test]
fn an_untapped_factory_land_pumps_another_worker_that_is_blocked() {
    let mut g = tbl();
    let a = two_three(&mut g);
    let f = animated_worker(&mut g, P0, "Mishra's Factory");
    g.perm_mut(f).tapped = true; // it attacked
    let land = lands(&mut g, P0, "Mishra's Factory", 1, false)[0];
    karn::combat_pumps(&mut g, P1, &[], P0, &[]).unwrap();
    assert_eq!(g.perm(f).eot_pt, (0, 0), "no fight, nothing pumped");
    // P0 attacks, P1's 2/3 blocks the Factory
    karn::combat_pumps(&mut g, P0, &[f], P1, &[(f, a)]).unwrap();
    assert!(g.perm(land).tapped, "the Factory land tapped for the pump");
    assert_eq!(g.perm(f).eot_pt, (1, 1));
}

#[test]
fn a_foundry_pumps_an_attacking_worker_for_one_mana_and_only_an_attacking_one() {
    let mut g = tbl();
    let a = two_three(&mut g);
    let f = animated_worker(&mut g, P0, "Mishra's Factory");
    g.perm_mut(f).tapped = true; // attacking
    let foundry = lands(&mut g, P0, "Mishra's Foundry", 1, false)[0];
    lands(&mut g, P0, "Wastes", 1, false);
    // as a blocker the Factory is not "attacking": the Foundry does nothing
    karn::combat_pumps(&mut g, P1, &[a], P0, &[(a, f)]).unwrap();
    assert!(!g.perm(foundry).tapped);
    // attacking: +2/+2
    karn::combat_pumps(&mut g, P0, &[f], P1, &[(f, a)]).unwrap();
    assert!(g.perm(foundry).tapped);
    assert_eq!(g.perm(f).eot_pt, (2, 2));
    assert!(
        g.player(P0).lands.iter().any(|&l| g.perm(l).tapped && g.perm(l).cd == Some(card(&g, "Wastes"))),
        "{{1}} paid"
    );
}

// ======================================================== Steel Hellkite
#[test]
fn the_ai_pumps_an_unblocked_hellkite_when_that_is_lethal() {
    let mut g = tbl();
    let h = perm(&mut g, P0, "Steel Hellkite");
    lands(&mut g, P0, "Wastes", 6, false);
    g.player_mut(P1).life = 8;
    let power = sim_core::engine::values::epow(&g, h);
    assert_eq!(power, 5);
    karn_cards::hellkite_pump_lethal(&mut g, P0, &[h], P1, &[], &[]).unwrap();
    assert_eq!(sim_core::engine::values::epow(&g, h), 8, "three pumps of {{2}}: exactly lethal");
    // not lethal even pumped: nothing is spent
    let mut g = tbl();
    let h = perm(&mut g, P0, "Steel Hellkite");
    lands(&mut g, P0, "Wastes", 4, false);
    g.player_mut(P1).life = 20;
    karn_cards::hellkite_pump_lethal(&mut g, P0, &[h], P1, &[], &[]).unwrap();
    assert_eq!(sim_core::engine::values::epow(&g, h), 5);
    // blocked: no pump
    let mut g = tbl();
    let h = perm(&mut g, P0, "Steel Hellkite");
    let b = token(&mut g, P1, 1);
    lands(&mut g, P0, "Wastes", 6, false);
    g.player_mut(P1).life = 8;
    karn_cards::hellkite_pump_lethal(&mut g, P0, &[h], P1, &[(h, b)], &[]).unwrap();
    assert_eq!(sim_core::engine::values::epow(&g, h), 5);
}

// ======================================================== Mystic Forge
#[test]
fn the_ai_casts_all_is_dust_from_the_top_of_the_library_with_a_mystic_forge() {
    let mut g = tbl();
    perm(&mut g, P0, "Mystic Forge");
    lands(&mut g, P0, "Wastes", 7, false);
    let aid = take(&mut g, P0, "All Is Dust");
    g.player_mut(P0).library.push(aid);
    let titan = perm(&mut g, P1, "Grave Titan");
    perm(&mut g, P2, "Lyra Dawnbringer");
    perm(&mut g, P0, "Wurmcoil Engine");
    let n = g.player(P0).library.len();
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    let o = opts
        .iter()
        .find(|o| o.label == "All Is Dust")
        .unwrap_or_else(|| panic!("{:?}", opts.iter().map(|o| (&o.label, o.utility)).collect::<Vec<_>>()));
    assert!(run(&mut g, P0, o));
    assert!(!g.perm(titan).on_bf, "the coloured permanents were sacrificed");
    assert_eq!(g.player(P0).library.len(), n - 1, "cast from the top");
    assert!(in_gy(&g, P0, "All Is Dust"));
    // without a Forge the same top card is not offered
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 7, false);
    let aid = take(&mut g, P0, "All Is Dust");
    g.player_mut(P0).library.push(aid);
    perm(&mut g, P1, "Grave Titan");
    perm(&mut g, P2, "Lyra Dawnbringer");
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    assert!(opts.iter().all(|o| o.label != "All Is Dust"));
}

// ======================================================== Pithing Needle
fn name_for_needle(g: &mut Game, name: &str) {
    let n = perm(g, P1, "Pithing Needle");
    let c = card(g, name);
    g.perm_mut(n).data.set(DataKey::Imprint, Val::Card(c));
}

#[test]
fn a_person_cannot_activate_a_land_or_unearth_the_named_card_but_can_tap_it_for_mana() {
    let mut g = table(&["karn", "cloud"]);
    name_for_needle(&mut g, "Mishra's Factory");
    let l = lands(&mut g, P0, "Mishra's Factory", 1, false)[0];
    g.player_mut(P0).land_turn = -1;
    pool(&mut g, P0, &[('C', 1)]);
    let i = g.player(P0).lands.iter().position(|&x| x == l).unwrap();
    let h = seat(&mut g, P0, vec![act(json!({"do": "use", "land": i})), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(perms_named(&g, P0, "Mishra's Factory").is_empty(), "{:?}", h.told());
    assert_eq!(pool_total(&g, P0), 1, "nothing paid");
    // its mana ability is not stopped: the AI's own mana code taps it
    assert!(sim_core::engine::mana::total_mana(&g, P0, false) >= 1);
}

#[test]
fn a_person_cannot_use_a_permanents_ability_named_by_a_needle() {
    let mut g = table(&["karn", "cloud"]);
    name_for_needle(&mut g, "Grim Monolith");
    let m = perm(&mut g, P0, "Grim Monolith");
    g.perm_mut(m).tapped = true;
    pool(&mut g, P0, &[('C', 4)]);
    let i = g.player(P0).perms.iter().position(|&x| x == m).unwrap();
    seat(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(g.perm(m).tapped, "its untap ability can't be activated");
    assert_eq!(pool_total(&g, P0), 4);
}

#[test]
fn the_ai_does_not_unearth_a_leveler_a_needle_names() {
    let mut g = tbl();
    let c = card(&g, "Cityscape Leveler");
    g.player_mut(P0).gy.push(c);
    lands(&mut g, P0, "Wastes", 8, false);
    g.active = Some(P0);
    let f = g.registry.get(c).unwrap().gy_options.unwrap();
    assert_eq!(f(&mut g, c, P0, Some(false)).unwrap().len(), 1, "unearth is offered");
    name_for_needle(&mut g, "Cityscape Leveler");
    assert!(t6_stax::needle_stops(&g, "Cityscape Leveler"));
    assert!(f(&mut g, c, P0, Some(false)).unwrap().is_empty(), "stopped by the Needle");
}

// ======================================================== Tezzeret, Cruel Captain
#[test]
fn tezzerets_minus_seven_gives_an_emblem_that_makes_a_robot_with_three_counters() {
    let mut g = tbl();
    let tez = perm(&mut g, P0, "Tezzeret, Cruel Captain");
    let stone = perm(&mut g, P0, "Mind Stone");
    g.perm_mut(tez).loyalty = Some(7); // (after the Stone: an artifact entering adds a loyalty counter)
    let os = options(&mut g, tez, P0, Some(false));
    let ult = os
        .iter()
        .find(|o| o.label.contains("-7"))
        .unwrap_or_else(|| panic!("{:?}", os.iter().map(|o| &o.label).collect::<Vec<_>>()));
    assert!(run(&mut g, P0, ult));
    assert_eq!(g.perm(tez).loyalty, Some(0));
    assert!(g.player(P0).emblems.contains(&t6_godo::TEZ_EMBLEM));
    // the beginning of combat: a noncreature artifact becomes a 0/0 Robot with three +1/+1 counters
    assert!(!g.is_creature(stone));
    t6_godo::tez_emblem(&mut g, P0).unwrap();
    assert!(g.is_creature(stone), "an artifact creature");
    assert!(g.perm(stone).ttypes.contains(&"robot"));
    assert_eq!((sim_core::engine::values::epow(&g, stone), sim_core::engine::values::etgh(&g, stone)), (3, 3));
    // an artifact creature takes the counters as it is
    t6_godo::tez_emblem(&mut g, P0).unwrap();
    assert_eq!(sim_core::engine::values::epow(&g, stone), 6, "the Robot is the artifact creature to pump now");
}

#[test]
fn tezzerets_emblem_is_used_at_the_beginning_of_combat() {
    let mut g = tbl();
    g.player_mut(P0).emblems.push(t6_godo::TEZ_EMBLEM);
    let w = perm(&mut g, P0, "Wurmcoil Engine");
    let before = sim_core::engine::values::epow(&g, w);
    g.active = Some(P0);
    combat::combat(&mut g, P0).unwrap();
    assert!(sim_core::engine::values::epow(&g, w) >= before + 3);
}
