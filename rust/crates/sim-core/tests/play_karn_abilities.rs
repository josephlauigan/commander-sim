//! Practice mode: the person uses each activated ability of the Karn deck (JD, 10/21), paying from the pool and making
//! the choices the card asks for, and it does what the card says.

mod play_common;
use play_common::*;
use sim_core::engine::zones::{self, Enter, Tokens};
use sim_core::human::human::human_main;
use sim_core::human::{Target, legal};
use sim_core::ids::{LandId, PermId};
use sim_core::state::{DataKey, Val};

/// an answer that picks the choice describing x
fn pick_target(g: &Game, p: PlayerId, x: Target) -> F {
    let label = legal::describe_target(g, p, x);
    Box::new(move |r| {
        Answer::Pick(
            r.choices.iter().position(|c| *c == label).unwrap_or_else(|| panic!("{label:?} not in {:?}", r.choices)),
        )
    })
}

/// use permanent m's ability whose label contains `word`, then these answers
fn use_perm(g: &mut Game, m: PermId, word: &'static str, mut rest: Vec<F>) -> std::sync::Arc<ScriptHuman> {
    let i = g.player(P0).perms.iter().position(|&x| x == m).unwrap();
    let mut a = vec![act(json!({"do": "use", "perm": i})), by_text(word)];
    a.append(&mut rest);
    a.push(pass());
    let h = seat(g, P0, a);
    human_main(g, P0, false).unwrap();
    h
}

fn use_land(g: &mut Game, l: LandId, word: &'static str, mut rest: Vec<F>) -> std::sync::Arc<ScriptHuman> {
    let i = g.player(P0).lands.iter().position(|&x| x == l).unwrap();
    let mut a = vec![act(json!({"do": "use", "land": i})), by_text(word)];
    a.append(&mut rest);
    a.push(pass());
    let h = seat(g, P0, a);
    human_main(g, P0, false).unwrap();
    h
}

fn art_token(g: &mut Game, p: PlayerId, pow: i32) -> PermId {
    let mut spec = Tokens { sick: false, ..Tokens::new(1, pow) };
    spec.tgh = Some(pow.max(1));
    spec.types = vec!["artifact"];
    zones::make_tokens(g, p, spec).unwrap()[0]
}

fn on_bf(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

fn karn_table() -> Game {
    table(&["karn", "veyran"])
}

// ------------------------------------------------------------------ artifacts and creatures
#[test]
fn grim_monolith_untaps_for_four() {
    let mut g = karn_table();
    let m = perm(&mut g, P0, "Grim Monolith");
    g.perm_mut(m).tapped = true;
    pool(&mut g, P0, &[('C', 4)]);
    use_perm(&mut g, m, "untap", vec![]);
    assert!(!g.perm(m).tapped);
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn liquimetal_torque_makes_a_permanent_an_artifact() {
    let mut g = karn_table();
    let t = perm(&mut g, P0, "Liquimetal Torque");
    let gs = perm(&mut g, P1, "Guttersnipe");
    assert!(!sim_core::impls::t5::is_artifact(&g, gs));
    let pt = pick_target(&g, P0, Target::Perm(gs));
    use_perm(&mut g, t, "becomes an artifact", vec![pt]);
    assert!(g.perm(t).tapped);
    assert!(sim_core::impls::t5::is_artifact(&g, gs));
}

#[test]
fn mazemind_tome_scry_and_draw() {
    let mut g = karn_table();
    let t = perm(&mut g, P0, "Mazemind Tome");
    use_perm(&mut g, t, "scry", vec![pick(0)]);
    assert_eq!(g.perm(t).data.int(DataKey::Counters), 1);
    assert!(g.perm(t).tapped);
    g.perm_mut(t).tapped = false;
    pool(&mut g, P0, &[('C', 2)]);
    let n = g.player(P0).hand.len();
    use_perm(&mut g, t, "draw", vec![]);
    assert_eq!(g.player(P0).hand.len(), n + 1);
    assert_eq!(g.perm(t).data.int(DataKey::Counters), 2);
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn mishras_bauble_looks_at_the_top_card_and_draws_next_upkeep() {
    let mut g = karn_table();
    let b = perm(&mut g, P0, "Mishra's Bauble");
    let pt = pick_target(&g, P0, Target::Player(P1));
    let h = use_perm(&mut g, b, "top card", vec![pt, pick(0)]);
    assert!(!on_bf(&g, b));
    assert_eq!(g.player(P0).delayed_draws, 1);
    let top = g.player(P1).library.last().copied().unwrap();
    let want = g.db.get(top).name.to_string();
    assert!(h.asked().iter().any(|r| r.prompt.contains(&want)), "the top card {want} is shown");
}

#[test]
fn urzas_bauble_looks_at_a_card_in_hand() {
    let mut g = karn_table();
    let b = perm(&mut g, P0, "Urza's Bauble");
    hand(&mut g, P1, &["Terminate"]);
    g.player_mut(P1).hand.truncate(1);
    let pt = pick_target(&g, P0, Target::Player(P1));
    let h = use_perm(&mut g, b, "random card", vec![pt, pick(0)]);
    assert!(!on_bf(&g, b));
    assert_eq!(g.player(P0).delayed_draws, 1);
    assert!(h.asked().iter().any(|r| r.prompt.contains("Terminate")));
}

#[test]
fn nevinyrrals_disk_destroys_artifacts_creatures_and_enchantments() {
    let mut g = karn_table();
    let d = perm(&mut g, P0, "Nevinyrral's Disk");
    g.perm_mut(d).tapped = false;
    let ring = perm(&mut g, P0, "Sol Ring");
    let gs = perm(&mut g, P1, "Guttersnipe");
    pool(&mut g, P0, &[('C', 1)]);
    use_perm(&mut g, d, "destroy all", vec![]);
    assert!(!on_bf(&g, ring) && !on_bf(&g, gs) && !on_bf(&g, d));
}

#[test]
fn oblivion_stone_fate_counter_then_wipe() {
    let mut g = karn_table();
    let s = perm(&mut g, P0, "Oblivion Stone");
    let ring = perm(&mut g, P0, "Sol Ring");
    let gs = perm(&mut g, P1, "Guttersnipe");
    pool(&mut g, P0, &[('C', 4)]);
    let pt = pick_target(&g, P0, Target::Perm(ring));
    use_perm(&mut g, s, "fate counter", vec![pt]);
    assert_eq!(g.perm(ring).data.int(DataKey::Fate), 1);
    assert!(g.perm(s).tapped && on_bf(&g, gs));
    g.perm_mut(s).tapped = false;
    pool(&mut g, P0, &[('C', 5)]);
    use_perm(&mut g, s, "destroy each", vec![]);
    assert!(on_bf(&g, ring), "the marked Sol Ring survives");
    assert_eq!(g.perm(ring).data.int(DataKey::Fate), 0, "the counters are removed");
    assert!(!on_bf(&g, gs) && !on_bf(&g, s));
}

#[test]
fn ratchet_bomb_counts_up_and_blows_at_its_counters() {
    let mut g = karn_table();
    let b = perm(&mut g, P0, "Ratchet Bomb");
    let stone = perm(&mut g, P1, "Mind Stone"); // mana value 2
    let gs = perm(&mut g, P1, "Guttersnipe"); // mana value 3
    use_perm(&mut g, b, "charge counter", vec![]);
    assert_eq!(g.perm(b).data.int(DataKey::Charge), 1);
    g.perm_mut(b).tapped = false;
    g.perm_mut(b).data.set(DataKey::Charge, Val::Int(2));
    use_perm(&mut g, b, "mana value 2", vec![]);
    assert!(!on_bf(&g, b) && !on_bf(&g, stone));
    assert!(on_bf(&g, gs));
}

#[test]
fn soul_guide_lantern_exiles_graveyards_or_draws() {
    let mut g = karn_table();
    let l = perm(&mut g, P0, "Soul-Guide Lantern");
    let c = card(&g, "Terminate");
    g.player_mut(P1).gy.push(c);
    use_perm(&mut g, l, "exile each opponent", vec![]);
    assert!(g.player(P1).gy.is_empty() && g.player(P1).exile.contains(&c));
    assert!(!on_bf(&g, l));
    let l2 = perm(&mut g, P0, "Soul-Guide Lantern");
    pool(&mut g, P0, &[('C', 1)]);
    let n = g.player(P0).hand.len();
    use_perm(&mut g, l2, "draw a card", vec![]);
    assert_eq!(g.player(P0).hand.len(), n + 1);
    assert!(!on_bf(&g, l2));
}

#[test]
fn steel_hellkite_pumps_and_destroys_by_mana_value() {
    let mut g = karn_table();
    let k = perm(&mut g, P0, "Steel Hellkite");
    pool(&mut g, P0, &[('C', 2)]);
    use_perm(&mut g, k, "+1/+0", vec![]);
    assert_eq!(g.perm(k).eot_pt.0, 1);
    // it hasn't hit anyone yet: the X ability says so
    let h = use_perm(&mut g, k, "destroy each", vec![]);
    assert!(h.told().iter().any(|t| t.1.contains("combat damage")), "{:?}", h.told());
    let stone = perm(&mut g, P1, "Mind Stone"); // mana value 2
    let gs = perm(&mut g, P1, "Guttersnipe"); // mana value 3
    let st = g.turn_stamp();
    g.perm_mut(k).data.set(DataKey::Hit, Val::List(vec![Val::Stamp(st), Val::Player(P1)]));
    pool(&mut g, P0, &[('C', 2)]);
    use_perm(&mut g, k, "destroy each", vec![by_text("X = 2")]);
    assert!(!on_bf(&g, stone) && on_bf(&g, gs));
    let h = use_perm(&mut g, k, "destroy each", vec![]);
    assert!(h.told().iter().any(|t| t.1.contains("once each turn")), "{:?}", h.told());
}

#[test]
fn steel_overseer_grows_artifact_creatures() {
    let mut g = karn_table();
    let o = perm(&mut g, P0, "Steel Overseer");
    let a = art_token(&mut g, P0, 1);
    let plain = token(&mut g, P0, 1);
    use_perm(&mut g, o, "+1/+1 counter", vec![]);
    assert_eq!(g.perm(a).plus, 1);
    assert_eq!(g.perm(plain).plus, 0);
    assert!(g.perm(o).tapped);
}

#[test]
fn unstable_obelisk_destroys_the_permanent_you_pick() {
    let mut g = karn_table();
    let ob = perm(&mut g, P0, "Unstable Obelisk");
    let gs = perm(&mut g, P1, "Guttersnipe");
    let stone = perm(&mut g, P1, "Mind Stone");
    pool(&mut g, P0, &[('C', 7)]);
    let pt = pick_target(&g, P0, Target::Perm(stone)); // not the best target: the person chooses
    use_perm(&mut g, ob, "destroy target", vec![pt]);
    assert!(!on_bf(&g, ob) && !on_bf(&g, stone) && on_bf(&g, gs));
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn kuldotha_forgemaster_sacrifices_three_and_fetches() {
    let mut g = karn_table();
    let f = perm(&mut g, P0, "Kuldotha Forgemaster");
    let arts: Vec<PermId> = ["Sol Ring", "Mana Vault", "Thought Vessel"].iter().map(|n| perm(&mut g, P0, n)).collect();
    let w = card(&g, "Wurmcoil Engine");
    g.player_mut(P0).library.push(w);
    let h = use_perm(&mut g, f, "sacrifice three", vec![pick(1), pick(1), pick(1), by_text("Wurmcoil")]);
    assert!(h.asked().len() >= 5, "which three, then the search");
    assert!(on_bf(&g, f) && g.perm(f).tapped);
    assert_eq!(arts.iter().filter(|&&a| on_bf(&g, a)).count(), 0, "three artifacts sacrificed");
    assert!(has_perm(&g, P0, "Wurmcoil Engine"));
}

#[test]
fn walking_ballista_grows_and_pings() {
    let mut g = karn_table();
    let c = take(&mut g, P0, "Walking Ballista");
    let b = zones::enter(&mut g, P0, c, Enter { sick: false, plus: 2, ..Default::default() }).unwrap();
    assert_eq!(g.perm(b).plus, 2);
    pool(&mut g, P0, &[('C', 4)]);
    use_perm(&mut g, b, "{4}", vec![]);
    assert_eq!(g.perm(b).plus, 3);
    let before = g.player(P1).life;
    let pt = pick_target(&g, P0, Target::Player(P1));
    use_perm(&mut g, b, "remove a +1/+1", vec![pt]);
    assert_eq!((g.perm(b).plus, g.player(P1).life), (2, before - 1));
    // a creature it kills
    let x = token(&mut g, P1, 1);
    let pt = pick_target(&g, P0, Target::Perm(x));
    use_perm(&mut g, b, "remove a +1/+1", vec![pt]);
    assert!(!on_bf(&g, x), "a 1/1 dies to the ping");
    assert_eq!(g.perm(b).plus, 1);
}

#[test]
fn walking_ballista_dies_when_the_last_counter_goes() {
    let mut g = karn_table();
    let c = take(&mut g, P0, "Walking Ballista");
    let b = zones::enter(&mut g, P0, c, Enter { sick: false, plus: 1, ..Default::default() }).unwrap();
    let pt = pick_target(&g, P0, Target::Player(P1));
    use_perm(&mut g, b, "remove a +1/+1", vec![pt]);
    assert!(!on_bf(&g, b));
    assert_eq!(g.player(P1).life, 39);
}

#[test]
fn batterskull_returns_to_hand_for_three() {
    let mut g = karn_table();
    let s = perm(&mut g, P0, "Batterskull");
    pool(&mut g, P0, &[('C', 3)]);
    use_perm(&mut g, s, "return Batterskull", vec![]);
    assert!(!on_bf(&g, s));
    assert!(g.player(P0).hand.contains(&card(&g, "Batterskull")));
}

#[test]
fn endless_atlas_needs_three_same_named_lands() {
    let mut g = karn_table();
    let a = perm(&mut g, P0, "Endless Atlas");
    lands(&mut g, P0, "Wastes", 2, false);
    pool(&mut g, P0, &[('C', 2)]);
    let h = use_perm(&mut g, a, "draw", vec![]);
    assert!(h.told().iter().any(|t| t.1.contains("three or more lands")), "{:?}", h.told());
    assert!(!g.perm(a).tapped);
    lands(&mut g, P0, "Wastes", 1, false);
    let n = g.player(P0).hand.len();
    use_perm(&mut g, a, "draw", vec![]);
    assert_eq!(g.player(P0).hand.len(), n + 1);
    assert!(g.perm(a).tapped);
}

// ------------------------------------------------------------------ lands
#[test]
fn bonders_enclave_needs_a_power_four_creature() {
    let mut g = karn_table();
    let l = lands(&mut g, P0, "Bonders' Enclave", 1, false)[0];
    pool(&mut g, P0, &[('C', 3)]);
    let h = use_land(&mut g, l, "draw", vec![]);
    assert!(h.told().iter().any(|t| t.1.contains("power 4")), "{:?}", h.told());
    token(&mut g, P0, 4);
    let n = g.player(P0).hand.len();
    use_land(&mut g, l, "draw", vec![]);
    assert_eq!(g.player(P0).hand.len(), n + 1);
    assert!(g.land(l).tapped);
}

#[test]
fn buried_ruin_returns_the_artifact_you_pick() {
    let mut g = karn_table();
    let l = lands(&mut g, P0, "Buried Ruin", 1, false)[0];
    let (a, b) = (card(&g, "Sol Ring"), card(&g, "Mana Vault"));
    g.player_mut(P0).gy.extend([a, b]);
    pool(&mut g, P0, &[('C', 2)]);
    use_land(&mut g, l, "artifact card", vec![by_text("Mana Vault")]);
    assert!(g.player(P0).hand.contains(&b) && g.player(P0).gy.contains(&a));
    assert!(g.player(P0).lands.is_empty(), "sacrificed");
    assert!(g.player(P0).gy.contains(&card(&g, "Buried Ruin")));
}

#[test]
fn inventors_fair_searches_with_three_artifacts() {
    let mut g = karn_table();
    let l = lands(&mut g, P0, "Inventors' Fair", 1, false)[0];
    pool(&mut g, P0, &[('C', 4)]);
    let h = use_land(&mut g, l, "search", vec![]);
    assert!(h.told().iter().any(|t| t.1.contains("fewer than three artifacts")), "{:?}", h.told());
    for a in ["Sol Ring", "Mana Vault", "Thought Vessel"] {
        perm(&mut g, P0, a);
    }
    let w = card(&g, "Wurmcoil Engine");
    g.player_mut(P0).library.push(w);
    use_land(&mut g, l, "search", vec![by_text("Wurmcoil")]);
    assert!(g.player(P0).hand.contains(&w));
    assert!(!g.player(P0).lands.contains(&l), "sacrificed");
}

#[test]
fn war_room_draws_for_three() {
    let mut g = karn_table();
    let l = lands(&mut g, P0, "War Room", 1, false)[0];
    pool(&mut g, P0, &[('C', 3)]);
    g.player_mut(P0).ident = sim_core::cards::Colors::from_letters("WU");
    let n = g.player(P0).hand.len();
    use_land(&mut g, l, "draw", vec![]);
    assert_eq!(g.player(P0).hand.len(), n + 1);
    assert_eq!(g.player(P0).life, 38, "two colours: 2 life");
    assert!(g.land(l).tapped);
}

#[test]
fn treasure_vault_makes_x_treasures() {
    let mut g = karn_table();
    let l = lands(&mut g, P0, "Treasure Vault", 1, false)[0];
    pool(&mut g, P0, &[('C', 5)]);
    use_land(&mut g, l, "Treasure", vec![by_text("X = 2")]);
    assert_eq!(g.player(P0).treasures, 2);
    assert_eq!(pool_total(&g, P0), 1, "{{X}}{{X}} with X = 2 costs 4");
    assert!(!g.player(P0).lands.contains(&l));
}

#[test]
fn urzas_factory_makes_an_assembly_worker() {
    let mut g = karn_table();
    let l = lands(&mut g, P0, "Urza's Factory", 1, false)[0];
    pool(&mut g, P0, &[('C', 7)]);
    use_land(&mut g, l, "Assembly-Worker", vec![]);
    let w = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token).expect("a token");
    assert!(g.is_creature(w) && g.perm(w).ttypes.contains(&"assembly-worker"));
    assert!(sim_core::impls::karn_cards::is_art(&g, w));
    assert!(g.land(l).tapped && pool_total(&g, P0) == 0);
}

#[test]
fn mishras_factory_animates_attacks_and_pumps() {
    let mut g = karn_table();
    let l = lands(&mut g, P0, "Mishra's Factory", 1, false)[0];
    g.player_mut(P0).land_turn = -1; // not played this turn
    pool(&mut g, P0, &[('C', 1)]);
    use_land(&mut g, l, "becomes", vec![]);
    let m = perms_named(&g, P0, "Mishra's Factory")[0];
    assert!(g.is_creature(m) && g.perm(m).ttypes.contains(&"assembly-worker") && !g.perm(m).sick);
    assert_eq!(pool_total(&g, P0), 0);
    // its {T} ability, now that it is a creature: +1/+1 on an Assembly-Worker (itself)
    let pt = pick_target(&g, P0, Target::Perm(m));
    use_perm(&mut g, m, "+1/+1", vec![pt]);
    assert_eq!(g.perm(m).eot_pt, (1, 1));
    assert!(g.perm(m).tapped);
}

#[test]
fn mishras_factory_land_pumps_another_worker_and_a_new_land_is_sick() {
    let mut g = karn_table();
    let l = lands(&mut g, P0, "Mishra's Factory", 1, false)[0];
    // the land came under your control this turn: animated, it is summoning sick
    g.player_mut(P0).land_turn = g.player(P0).turns as i32;
    pool(&mut g, P0, &[('C', 1)]);
    use_land(&mut g, l, "becomes", vec![]);
    let m = perms_named(&g, P0, "Mishra's Factory")[0];
    assert!(g.perm(m).sick);
    // the other Factory, still a land, pumps it
    let l2 = lands(&mut g, P0, "Mishra's Factory", 1, false)[0];
    let pt = pick_target(&g, P0, Target::Perm(m));
    use_land(&mut g, l2, "+1/+1", vec![pt]);
    assert_eq!(g.perm(m).eot_pt, (1, 1));
    assert!(g.land(l2).tapped);
}

#[test]
fn mishras_foundry_pumps_an_attacking_worker() {
    let mut g = karn_table();
    let f = lands(&mut g, P0, "Mishra's Foundry", 1, false)[0];
    let l2 = lands(&mut g, P0, "Urza's Factory", 1, false)[0];
    pool(&mut g, P0, &[('C', 8)]);
    use_land(&mut g, l2, "Assembly-Worker", vec![]);
    let w = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token).unwrap();
    let h = use_land(&mut g, f, "+2/+2", vec![]);
    assert!(h.told().iter().any(|t| t.1.contains("attacking")), "not attacking yet: {:?}", h.told());
    g.in_combat.push(w);
    let pt = pick_target(&g, P0, Target::Perm(w));
    use_land(&mut g, f, "+2/+2", vec![pt]);
    assert_eq!(g.perm(w).eot_pt, (2, 2));
    assert_eq!(pool_total(&g, P0), 0, "{{1}} paid");
    // and the Foundry animates for {2}
    let f2 = lands(&mut g, P0, "Mishra's Foundry", 1, false)[0];
    g.player_mut(P0).land_turn = -1;
    pool(&mut g, P0, &[('C', 2)]);
    use_land(&mut g, f2, "becomes", vec![]);
    assert!(g.is_creature(perms_named(&g, P0, "Mishra's Foundry")[0]));
}

#[test]
fn mystic_forge_exiles_the_top_card_for_a_life() {
    let mut g = karn_table();
    let f = perm(&mut g, P0, "Mystic Forge");
    let top = *g.player(P0).library.last().unwrap();
    let life = g.player(P0).life;
    use_perm(&mut g, f, "exile the top card", vec![]);
    assert!(g.player(P0).exile.contains(&top), "the top card is exiled");
    assert_eq!(g.player(P0).life, life - 1);
    assert!(g.perm(f).tapped);
}
