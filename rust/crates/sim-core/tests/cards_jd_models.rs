//! The Partial and Unmodeled cards of JD's decks, modelled in the card-models PRs (10-30): each test checks the
//! card's Oracle behaviour, and the AI's use of it where that matters. JD's decks sit only at their own tiers' tables.

use sim_core::engine::{cast, hooks, values};
use sim_core::hooks::{Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    sim_core::ai::act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn fire(g: &mut Game, e: Event, p: PlayerId) {
    hooks::fire_trigger(g, e, Call::Player { p }).unwrap();
}

#[allow(dead_code)]
fn cast_from_hand(g: &mut Game, p: PlayerId, name: &str) -> bool {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap()
}

fn pt(g: &Game, m: PermId) -> (i32, i32) {
    (values::epow(g, m), values::etgh(g, m))
}

#[allow(dead_code)]
fn ids(g: &Game, names: &[&str]) -> Vec<CardId> {
    names.iter().map(|n| card(g, n)).collect()
}

// ======================================================== Y'shtola: Venat, Heart of Hydaelyn // Hydaelyn
const VENAT: &str = "Venat, Heart of Hydaelyn // Hydaelyn, the Mothercrystal";

/// Y'shtola at a t5 table
fn yshtola_table() -> Game {
    table(&["yshtola", "zur-esper-enchantment-control", "miirym-temur-dragons"])
}

#[test]
fn venat_has_its_card_code() {
    let g = yshtola_table();
    let c = card(&g, VENAT);
    let imp = g.registry.get(c).expect("Venat's card code is registered under the deck's spelling");
    assert!(imp.cast.is_some() && imp.options.is_some() && imp.grant_kw.is_some() && imp.crew.is_some());
    assert!(!g.db.get(c).has_dsl(), "the compiled draw trigger would draw a second card");
}

#[test]
fn venat_draws_on_the_first_legendary_spell_each_turn() {
    let mut g = yshtola_table();
    g.active = Some(P0);
    perm(&mut g, P0, VENAT);
    let cs = hand(&mut g, P0, &["Sol Ring", "Gonti, Lord of Luxury", "Grand Arbiter Augustin IV"]);
    let draws = |g: &Game| g.player(P0).stats.get("jr_venat_draws").copied().unwrap_or(0);
    let lib = g.player(P0).library.len();
    cast::cast_card(&mut g, P0, cs[0], "hand", Ctx::default()).unwrap(); // not legendary: nothing
    assert_eq!((draws(&g), g.player(P0).library.len()), (0, lib));
    cast::cast_card(&mut g, P0, cs[1], "hand", Ctx::default()).unwrap();
    assert_eq!((draws(&g), g.player(P0).library.len()), (1, lib - 1));
    cast::cast_card(&mut g, P0, cs[2], "hand", Ctx::default()).unwrap(); // only once each turn
    assert_eq!((draws(&g), g.player(P0).library.len()), (1, lib - 1));
}

#[test]
fn heros_sundering_exiles_the_best_opposing_permanent_and_transforms() {
    let mut g = yshtola_table();
    g.active = Some(P0);
    let v = perm(&mut g, P0, VENAT);
    g.perm_mut(v).sick = false;
    lands(&mut g, P0, "Plains", 7, false);
    let big = perm(&mut g, P1, "Consecrated Sphinx");
    perm(&mut g, P2, "Sol Ring");
    let o = options(&mut g, v, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert!(o[0].label.contains("Consecrated Sphinx"), "{}", o[0].label);
    assert!(run(&mut g, P0, &o[0]));
    assert!(!g.perm(big).on_bf);
    assert!(g.perm(v).tapped);
    assert_eq!(pt(&g, v), (4, 4));
    assert!(values::indestructible(&g, v));
    assert!(options(&mut g, v, P0, Some(false)).is_empty(), "Hydaelyn has no Sundering");
}

#[test]
fn hydaelyn_blesses_a_legend_at_combat_and_draws() {
    let mut g = yshtola_table();
    g.active = Some(P0);
    let v = perm(&mut g, P0, VENAT);
    g.perm_mut(v).sick = false;
    lands(&mut g, P0, "Plains", 7, false);
    perm(&mut g, P1, "Consecrated Sphinx");
    let o = options(&mut g, v, P0, Some(false));
    run(&mut g, P0, &o[0]);
    let gonti = perm(&mut g, P0, "Gonti, Lord of Luxury");
    let n = g.player(P0).hand.len();
    fire(&mut g, Event::Crew, P0);
    assert_eq!(g.perm(gonti).plus, 1);
    assert!(values::indestructible(&g, gonti));
    assert_eq!(g.player(P0).hand.len(), n + 1);
    fire(&mut g, Event::Crew, P0); // once each turn
    assert_eq!(g.perm(gonti).plus, 1);
}

// ======================================================== Sephiroth: Glimpse the Unthinkable
/// Sephiroth at a t3 table
fn seph_table() -> Game {
    table(&["seph", "korvold-jund-sacrifice", "tergrid-mono-black-disruption"])
}

#[test]
fn glimpse_fills_the_graveyard_when_the_plan_wants_it() {
    let mut g = seph_table();
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    let glimpse = hand(&mut g, P0, &["Glimpse the Unthinkable", "Animate Dead"])[0];
    lands(&mut g, P0, "Watery Grave", 2, false);
    let (lib, gy) = (g.player(P0).library.len(), g.player(P0).gy.len());
    assert!(sim_core::ai::seph::seph_fill(&mut g, P0).unwrap());
    let pl = g.player(P0);
    assert!(!pl.hand.contains(&glimpse));
    assert_eq!(pl.library.len(), lib - 10);
    assert_eq!(pl.gy.len(), gy + 11, "ten milled cards and Glimpse itself");
}

#[test]
fn glimpse_waits_without_a_way_to_reanimate() {
    let mut g = seph_table();
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Glimpse the Unthinkable"]);
    lands(&mut g, P0, "Watery Grave", 2, false);
    let lib = g.player(P0).library.len();
    let _ = sim_core::ai::seph::seph_fill(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).library.len(), lib);
}

// ======================================================== Kefka: Price of Progress
/// Kefka at a t3 table
fn kefka_table() -> Game {
    table(&["kefka", "korvold-jund-sacrifice", "atraxa-infect"])
}

#[test]
fn price_of_progress_hits_each_player_for_twice_their_nonbasics() {
    let mut g = kefka_table();
    lands(&mut g, P0, "Steam Vents", 1, false);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P1, "Overgrown Tomb", 3, false);
    lands(&mut g, P2, "Forest", 4, false);
    let life: Vec<i32> = [P0, P1, P2].iter().map(|&q| g.player(q).life).collect();
    let c = hand(&mut g, P0, &["Price of Progress"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    let after: Vec<i32> = [P0, P1, P2].iter().map(|&q| g.player(q).life).collect();
    assert_eq!(after, vec![life[0] - 2, life[1] - 6, life[2]]);
}

#[test]
fn price_of_progress_is_worth_casting_only_when_the_opponents_pay_more() {
    let mut g = kefka_table();
    let c = card(&g, "Price of Progress");
    lands(&mut g, P1, "Overgrown Tomb", 4, false);
    lands(&mut g, P2, "Breeding Pool", 3, false);
    let good = sim_core::dsl::card_value(&g, P0, c);
    lands(&mut g, P0, "Steam Vents", 6, false);
    let bad = sim_core::dsl::card_value(&g, P0, c);
    assert!(good > 3.0 && bad < good, "{good} {bad}");
}

// ======================================================== Sauron: Baleful Mastery
/// Sauron at a t3 table
fn sauron_table() -> Game {
    table(&["sauron", "korvold-jund-sacrifice", "aurelia-boros-extra-combats"])
}

fn hands(g: &Game) -> Vec<usize> {
    [P0, P1, P2].iter().map(|&q| g.player(q).hand.len()).collect()
}

#[test]
fn baleful_mastery_short_of_mana_pays_one_b_and_an_opponent_draws() {
    let mut g = sauron_table();
    g.active = Some(P0);
    let c = hand(&mut g, P0, &["Baleful Mastery"])[0];
    lands(&mut g, P0, "Swamp", 2, false);
    let t = token(&mut g, P1, 6); // (not Consecrated Sphinx: it would draw two on the opponent's draw)
    let before = hands(&g);
    assert!(sim_core::ai::brain::cast_removal(&mut g, P0, c, &[t], 0, None).unwrap());
    assert!(!g.perm(t).on_bf);
    let after = hands(&g);
    assert_eq!(after[0], before[0] - 1);
    assert_eq!(after[1] + after[2], before[1] + before[2] + 1, "one opponent draws a card");
}

#[test]
fn baleful_mastery_with_the_mana_pays_full_and_nobody_draws() {
    let mut g = sauron_table();
    g.active = Some(P0);
    let c = hand(&mut g, P0, &["Baleful Mastery"])[0];
    lands(&mut g, P0, "Swamp", 4, false);
    let t = perm(&mut g, P1, "Consecrated Sphinx");
    let before = hands(&g);
    assert!(sim_core::ai::brain::cast_removal(&mut g, P0, c, &[t], 0, None).unwrap());
    assert!(!g.perm(t).on_bf);
    let after = hands(&g);
    assert_eq!((after[1], after[2]), (before[1], before[2]));
    assert!(g.player(P0).lands.iter().all(|&m| g.perm(m).tapped), "{{3}}{{B}} paid");
}

// ======================================================== Thanos: Orim's Chant
/// Thanos at a t6 table
fn thanos_table() -> Game {
    table(&["thanos", "kraum-tymna-blue-breach-midrange", "sisay-legends-toolbox"])
}

fn chant(g: &mut Game, kicks: u32, turn: PlayerId) {
    g.active = Some(turn);
    let c = hand(g, P0, &["Orim's Chant"])[0];
    cast::cast_card(g, P0, c, "hand", Ctx { face: Some(P1), kicks, ..Ctx::default() }).unwrap();
}

#[test]
fn orims_chant_locks_the_target_player_out_of_spells() {
    let mut g = thanos_table();
    chant(&mut g, 0, P0);
    assert!(cast::spell_locked(&g, P1, card(&g, "Sol Ring")));
    assert!(!cast::spell_locked(&g, P2, card(&g, "Sol Ring")));
    assert!(!sim_core::impls::t6_stax::attack_lock(&g));
}

#[test]
fn kicked_orims_chant_stops_every_attack_this_turn() {
    let mut g = thanos_table();
    chant(&mut g, 1, P1); // cast in P1's turn, before its combat
    assert!(sim_core::impls::t6_stax::attack_lock(&g));
    token(&mut g, P1, 9);
    let life: Vec<i32> = [P0, P2].iter().map(|&q| g.player(q).life).collect();
    sim_core::engine::combat::combat(&mut g, P1).unwrap();
    assert_eq!(life, [P0, P2].iter().map(|&q| g.player(q).life).collect::<Vec<_>>());
}

// ======================================================== Karn: Treasure Vault ({X} activation cost)
/// Karn at a t4 table
fn karn_table() -> Game {
    table(&["karn", "chulane-bant-value-combo", "yuriko-dimir-ninjas"])
}

fn treasures(g: &Game, p: PlayerId) -> u32 {
    g.player(p).treasures
}

fn vault_options(g: &mut Game, vault: PermId, sorcery_ok: bool) -> Vec<Opt> {
    let o = sim_core::dsl::ability_options(g, P0, sorcery_ok).unwrap();
    o.into_iter().filter(|o| matches!(o.act, Some(sim_core::hooks::Action::Dsl { src, .. }) if src == vault)).collect()
}

#[test]
fn treasure_vault_pays_x_twice_for_x_treasures() {
    let mut g = karn_table();
    g.active = Some(P0);
    let v = lands(&mut g, P0, "Treasure Vault", 1, false)[0];
    lands(&mut g, P0, "Wastes", 5, false);
    let o = vault_options(&mut g, v, true);
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, P0, &o[0]));
    assert_eq!(treasures(&g, P0), 2, "five other mana pays X = 2");
    assert!(!g.perm(v).on_bf, "sacrificed");
    assert_eq!(g.player(P0).lands.iter().filter(|&&m| !g.perm(m).tapped).count(), 1);
}

#[test]
fn treasure_vault_waits_for_the_end_of_the_turn_before_its_own() {
    let mut g = karn_table();
    g.active = Some(P1);
    let v = lands(&mut g, P0, "Treasure Vault", 1, false)[0];
    lands(&mut g, P0, "Wastes", 6, false);
    let main = vault_options(&mut g, v, true);
    let eot = vault_options(&mut g, v, false);
    assert!(main[0].utility < 0.0, "in a main phase three Treasures don't pay for six mana and a land");
    assert!(eot[0].utility > 0.0, "with the mana going unused they do");
}

#[test]
fn treasure_vault_needs_two_mana_for_x_one() {
    let mut g = karn_table();
    let v = lands(&mut g, P0, "Treasure Vault", 1, false)[0];
    lands(&mut g, P0, "Wastes", 1, false);
    assert!(vault_options(&mut g, v, false).is_empty());
}

// ======================================================== The Ur-Dragon: Dragonlord Ojutai
/// The Ur-Dragon at a t5 table
fn urdragon_table() -> Game {
    table(&["urdragon", "kinnan-simic-mana-combo", "urza-mono-blue-artifacts"])
}

#[test]
fn ojutai_has_hexproof_only_while_untapped() {
    let mut g = urdragon_table();
    let o = perm(&mut g, P0, "Dragonlord Ojutai");
    assert!(values::untargetable(&g, o));
    g.perm_mut(o).tapped = true;
    assert!(!values::untargetable(&g, o));
}

#[test]
fn ojutai_digs_three_on_combat_damage() {
    let mut g = urdragon_table();
    g.active = Some(P0);
    let o = perm(&mut g, P0, "Dragonlord Ojutai");
    let top = ids(&g, &["Island", "Sol Ring", "Forest"]);
    for &c in &top {
        let lib = &mut g.player_mut(P0).library;
        lib.retain(|&x| x != c);
        lib.push(c); // the last is the top
    }
    let (hand, lib) = (g.player(P0).hand.len(), g.player(P0).library.len());
    g.perm_mut(o).tapped = true;
    sim_core::engine::combat::resolve_combat(&mut g, P0, &[o], P1, &[o]).unwrap();
    let pl = g.player(P0);
    assert_eq!(pl.hand.len(), hand + 1);
    assert_eq!(pl.library.len(), lib - 1);
    assert!(pl.hand.contains(&top[1]), "Sol Ring is the one worth most");
    assert!(pl.library[..2].contains(&top[0]) && pl.library[..2].contains(&top[2]), "the rest on the bottom");
}

// ======================================================== Thanos: Gemstone Caverns
#[test]
fn gemstone_caverns_begins_on_the_battlefield_when_not_starting() {
    let mut g = table(&["kraum-tymna-blue-breach-midrange", "thanos", "sisay-legends-toolbox"]);
    let c = hand(&mut g, P1, &["Gemstone Caverns", "Sol Ring", "Island"])[0];
    let (hand, exiled) = (g.player(P1).hand.len(), g.player(P1).exile.cards().len());
    sim_core::impls::card_models::caverns_opening(&mut g, P1);
    let pl = g.player(P1);
    assert!(!pl.hand.contains(&c));
    assert_eq!((pl.hand.len(), pl.exile.cards().len()), (hand - 2, exiled + 1), "a card exiled from the hand");
    let l = *pl.lands.iter().find(|&&m| g.perm(m).cd == Some(c)).expect("on the battlefield");
    assert_eq!(g.perm(l).data.int(sim_core::state::DataKey::Counters), 1, "a luck counter: any colour");
}

#[test]
fn gemstone_caverns_stays_in_the_starting_players_hand() {
    let mut g = table(&["thanos", "kraum-tymna-blue-breach-midrange", "sisay-legends-toolbox"]);
    let c = hand(&mut g, P0, &["Gemstone Caverns"])[0];
    sim_core::impls::card_models::caverns_opening(&mut g, P0);
    assert!(g.player(P0).hand.contains(&c));
}

// ======================================================== Cloud: Buster Sword
/// Cloud at a t1 table
fn cloud_table() -> Game {
    table(&["cloud", "isshin-mardu-attack-triggers", "teysa-orzhov-aristocrats"])
}

#[test]
fn buster_sword_draws_then_casts_a_spell_up_to_the_damage_free() {
    let mut g = cloud_table();
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    let a = token(&mut g, P0, 2);
    let s = perm(&mut g, P0, "Buster Sword");
    g.perm_mut(s).attached = Some(a);
    let cs = hand(&mut g, P0, &["Sol Ring", "Swords to Plowshares"]);
    let big = token(&mut g, P0, 9); // (a second attacker without the Sword: no trigger)
    let lib = g.player(P0).library.len();
    let dmg = values::epow(&g, a);
    assert_eq!(dmg, 5, "+3/+2");
    sim_core::engine::combat::resolve_combat(&mut g, P0, &[a, big], P1, &[a, big]).unwrap();
    let pl = g.player(P0);
    assert_eq!(pl.library.len(), lib - 1, "one card drawn");
    assert!(!pl.hand.contains(&cs[0]), "Sol Ring cast free");
    assert!(pl.perms.iter().any(|&m| g.perm(m).cd == Some(cs[0])));
}
