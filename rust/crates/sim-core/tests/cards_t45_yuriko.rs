//! Tiers 4/5 card code for `yuriko-dimir-ninjas` (impls/t45_yuriko.rs) on hand-built positions: the new Ninjas'
//! ninjutsu, Kaito, Nashi, Skullsnatcher, Satoru, the evasion cards, the extra turns, the spell // land cards.

use sim_core::cardcode;
use sim_core::engine::{combat, hooks, zones};
use sim_core::hooks::{Action, Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

const YURIKO: &str = "yuriko-dimir-ninjas";

fn named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf && g.perm(m).name == name).collect()
}

fn opts(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.perm(src).cd.unwrap()).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn hand_opts(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    f(g, c, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    match o.act.clone() {
        Some(Action::Ability { src, f, arg }) => f(g, src, p, arg).unwrap(),
        Some(Action::Plan { f, arg }) => f(g, p, arg).unwrap(),
        _ => panic!("not an ability or plan"),
    }
}

fn resolve(g: &mut Game, p: PlayerId, name: &str, ctx: Ctx) -> &'static str {
    let c = card(g, name);
    let f = g.registry.get(c).unwrap().resolve.unwrap();
    f(g, p, c, &ctx).unwrap()
}

fn combat_damage(g: &mut Game, p: PlayerId, a: PermId, d: PlayerId, dmg: i32) {
    hooks::fire_trigger(g, Event::CombatDamage, Call::CombatDamage { p, a, d, dmg }).unwrap();
}

fn yuriko_table() -> Game {
    let mut g = table(&[YURIKO, "seph", "veyran"]);
    g.player_mut(P0).cmd_in_zone = false; // Yuriko elsewhere: only the Ninjas in hand
    g
}

// ------------------------------------------------------------------ ninjutsu
#[test]
fn the_new_ninjas_have_their_ninjutsu_costs() {
    let mut g = yuriko_table();
    hand(
        &mut g,
        P0,
        &["Ninja of the Deep Hours", "Skullsnatcher", "Nashi, Moon Sage's Scion", "Kaito, Bane of Nightmares"],
    );
    let costs = cardcode::ninjutsu_costs(&g, P0);
    assert_eq!(
        costs,
        vec![(1, "U".to_string()), (0, "B".to_string()), (3, "B".to_string()), (1, "UB".to_string())],
        "Deep Hours {{1}}{{U}}, Skullsnatcher {{B}}, Nashi {{3}}{{B}}, Kaito {{1}}{{U}}{{B}}"
    );
}

#[test]
fn skullsnatcher_ninjutsus_in_and_exiles_two_graveyard_cards() {
    let mut g = yuriko_table();
    lands(&mut g, P0, "Swamp", 1, false);
    hand(&mut g, P0, &["Skullsnatcher"]);
    let t = token(&mut g, P0, 1);
    let mut atk = vec![t];
    cardcode::ninjutsu(&mut g, P0, &mut atk, P1, &mut []).unwrap();
    let s = named(&g, P0, "Skullsnatcher");
    assert_eq!(atk, s, "Skullsnatcher attacks in the token's place");
    let gy: Vec<CardId> = ["Sol Ring", "Island", "Swamp"].iter().map(|n| card(&g, n)).collect();
    g.player_mut(P1).gy = gy;
    combat_damage(&mut g, P0, s[0], P1, 2);
    assert_eq!(g.player(P1).gy.len(), 1, "two of the three cards left the graveyard");
    assert_eq!(g.player(P1).exile.len(), 2);
    assert!(g.player(P1).exile.contains(&card(&g, "Sol Ring")), "the most valuable card goes");
}

#[test]
fn sakashimas_student_ninjutsus_in_as_a_copy_that_is_a_ninja() {
    let mut g = yuriko_table();
    lands(&mut g, P0, "Island", 2, false);
    perm(&mut g, P1, "Consecrated Sphinx");
    hand(&mut g, P0, &["Sakashima's Student"]);
    let t = token(&mut g, P0, 1);
    let mut atk = vec![t];
    cardcode::ninjutsu(&mut g, P0, &mut atk, P1, &mut []).unwrap();
    assert_eq!(atk.len(), 1);
    let m = atk[0];
    assert_eq!(&*g.perm(m).name, "Consecrated Sphinx", "the Student entered as a copy");
    assert!(sim_core::engine::values::has_type(&g, m, "ninja"), "a Ninja in addition");
    assert!(g.perm(m).tapped);
}

#[test]
fn sakashimas_student_stays_in_hand_with_nothing_to_copy() {
    let mut g = table(&[YURIKO, "seph"]);
    g.player_mut(P0).cmd_in_zone = false;
    lands(&mut g, P0, "Island", 2, false);
    hand(&mut g, P0, &["Sakashima's Student"]);
    let t = token(&mut g, P0, 1);
    let mut atk = vec![t];
    cardcode::ninjutsu(&mut g, P0, &mut atk, P1, &mut []).unwrap();
    assert_eq!(atk, vec![t], "a 0/0 Student would die: no ninjutsu");
}

// ------------------------------------------------------------------ Kaito, Bane of Nightmares
#[test]
fn kaito_ninjutsus_in_as_a_hexproof_ninja_creature_on_your_turn() {
    let mut g = yuriko_table();
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Swamp", 1, false);
    hand(&mut g, P0, &["Kaito, Bane of Nightmares"]);
    let t = token(&mut g, P0, 1);
    let mut atk = vec![t];
    cardcode::ninjutsu(&mut g, P0, &mut atk, P1, &mut []).unwrap();
    let k = named(&g, P0, "Kaito, Bane of Nightmares");
    assert_eq!(atk, k);
    let k = k[0];
    assert!(g.is_creature(k));
    assert_eq!((sim_core::engine::values::epow(&g, k), sim_core::engine::values::etgh(&g, k)), (3, 4));
    assert!(sim_core::engine::values::has_type(&g, k, "ninja"));
    assert!(sim_core::dsl::has_kw(&g, k, "hexproof"));
    // on an opponent's turn he is only a planeswalker
    g.active = Some(P1);
    sim_core::engine::hooks::fire_sba(&mut g).unwrap();
    assert!(!g.is_creature(k));
    assert!(!sim_core::dsl::has_kw(&g, k, "hexproof"));
}

#[test]
fn kaitos_emblem_pumps_ninjas_and_his_minus_two_stuns() {
    let mut g = yuriko_table();
    let k = perm(&mut g, P0, "Kaito, Bane of Nightmares");
    let ninja = perm(&mut g, P0, "Moon-Circuit Hacker");
    let base = sim_core::engine::values::epow(&g, ninja);
    let o = opts(&mut g, k, P0, Some(false));
    let plus = o.iter().find(|o| o.label.ends_with("+1")).expect("the +1").clone();
    assert!(run(&mut g, P0, &plus));
    assert_eq!(sim_core::engine::values::epow(&g, ninja), base + 1, "Ninjas you control get +1/+1");
    assert_eq!(g.perm(k).loyalty, Some(5));
    // next round: -2 on the best opposing creature
    g.round += 1;
    let big = perm(&mut g, P1, "Consecrated Sphinx");
    let o = opts(&mut g, k, P0, Some(false));
    let minus = o.iter().find(|o| o.label.ends_with("-2")).expect("the -2").clone();
    assert!(run(&mut g, P0, &minus));
    assert!(g.perm(big).tapped);
    assert_eq!(g.perm(big).data.int(sim_core::state::DataKey::Stun), 2);
}

#[test]
fn kaitos_zero_draws_for_each_opponent_who_lost_life() {
    let mut g = yuriko_table();
    let k = perm(&mut g, P0, "Kaito, Bane of Nightmares");
    sim_core::engine::life::lose_life(&mut g, P1, 2, Some(P0), "drain", None).unwrap();
    sim_core::engine::life::lose_life(&mut g, P2, 1, Some(P0), "drain", None).unwrap();
    let h = g.player(P0).hand.len();
    let o = opts(&mut g, k, P0, Some(false));
    let zero = o.iter().find(|o| o.label.ends_with("+0")).expect("the 0").clone();
    assert!(run(&mut g, P0, &zero));
    assert_eq!(g.player(P0).hand.len(), h + 2, "two opponents lost life this turn");
}

// ------------------------------------------------------------------ Nashi, Satoru
#[test]
fn nashi_casts_an_exiled_card_for_life() {
    let mut g = yuriko_table();
    let n = perm(&mut g, P0, "Nashi, Moon Sage's Scion");
    let ring = take(&mut g, P1, "Sol Ring");
    g.player_mut(P1).library.push(ring);
    let isl = take(&mut g, P0, "Island");
    g.player_mut(P0).library.push(isl);
    combat_damage(&mut g, P0, n, P1, 3);
    let rings = named(&g, P0, "Sol Ring");
    assert_eq!(rings.len(), 1, "Sol Ring was cast under Nashi's controller");
    assert_eq!(g.perm(rings[0]).orig, P1, "P1 still owns it");
    assert_eq!(g.player(P0).life, 39, "paid 1 life (its mana value)");
    assert!(g.player(P0).exile.contains(&isl), "the other exiled cards stay in exile");
}

#[test]
fn satoru_draws_when_a_creature_enters_without_being_cast() {
    let mut g = yuriko_table();
    perm(&mut g, P0, "Satoru, the Infiltrator");
    let h = g.player(P0).hand.len();
    let c = take(&mut g, P0, "Moon-Circuit Hacker");
    zones::enter(&mut g, P0, c, zones::Enter::default()).unwrap();
    assert_eq!(g.player(P0).hand.len(), h + 1, "put onto the battlefield: draw");
    let c = take(&mut g, P0, "Faerie Seer");
    zones::enter(&mut g, P0, c, zones::Enter { was_cast: true, ..zones::Enter::default() }).unwrap();
    assert_eq!(g.player(P0).hand.len(), h + 1, "a cast creature draws nothing");
    token(&mut g, P0, 1);
    assert_eq!(g.player(P0).hand.len(), h + 1, "tokens don't count");
}

// ------------------------------------------------------------------ evasion
#[test]
fn cover_of_darkness_gives_the_named_type_fear() {
    let mut g = yuriko_table();
    perm(&mut g, P0, "Cover of Darkness");
    let ninja = perm(&mut g, P0, "Moon-Circuit Hacker");
    assert!(sim_core::dsl::has_kw(&g, ninja, "fear"), "the deck names Ninja");
    let b = perm(&mut g, P1, "Consecrated Sphinx"); // blue: not black, not an artifact
    assert!(!combat::can_block(&g, b, ninja));
}

#[test]
fn shizo_gives_a_legendary_attacker_fear() {
    let mut g = yuriko_table();
    let shizo = lands(&mut g, P0, "Shizo, Death's Storehouse", 1, false)[0];
    lands(&mut g, P0, "Swamp", 1, false);
    let nashi = perm(&mut g, P0, "Nashi, Moon Sage's Scion");
    perm(&mut g, P1, "Consecrated Sphinx");
    let f = g.registry.get(card(&g, "Shizo, Death's Storehouse")).unwrap().land_options.unwrap();
    let o = f(&mut g, shizo, P0, Some(false)).unwrap();
    assert_eq!(o.len(), 1, "{o:?}");
    assert!(run(&mut g, P0, &o[0]));
    assert!(sim_core::dsl::has_kw(&g, nashi, "fear"));
    assert!(g.land(shizo).tapped);
}

#[test]
fn mothdust_changeling_taps_a_sick_creature_to_fly() {
    let mut g = yuriko_table();
    let moth = perm(&mut g, P0, "Mothdust Changeling");
    let sick = take(&mut g, P0, "Faerie Seer");
    let sick = zones::enter(&mut g, P0, sick, zones::Enter::default()).unwrap();
    token(&mut g, P1, 2);
    hooks::fire_trigger(&mut g, Event::Crew, Call::Player { p: P0 }).unwrap();
    assert!(sim_core::dsl::has_kw(&g, moth, "flying"));
    assert!(g.perm(sick).tapped, "the summoning-sick Seer paid for it");
}

#[test]
fn wingcrafter_pairs_and_both_fly() {
    let mut g = yuriko_table();
    let mate = perm(&mut g, P0, "Moon-Circuit Hacker");
    let w = perm(&mut g, P0, "Wingcrafter");
    assert!(sim_core::dsl::has_kw(&g, w, "flying"));
    assert!(sim_core::dsl::has_kw(&g, mate, "flying"));
}

// ------------------------------------------------------------------ spells
#[test]
fn temporal_mastery_takes_an_extra_turn_and_has_miracle() {
    let mut g = yuriko_table();
    assert_eq!(resolve(&mut g, P0, "Temporal Mastery", Ctx::default()), "exile");
    assert_eq!(g.extra_turns, vec![(P0, false)]);
    // miracle: the first card drawn this turn
    let mut g = yuriko_table();
    lands(&mut g, P0, "Island", 2, false);
    let tm = take(&mut g, P0, "Temporal Mastery");
    g.player_mut(P0).library.push(tm);
    zones::draw(&mut g, P0, 1, false).unwrap();
    let o = hand_opts(&mut g, tm, P0, Some(false));
    assert_eq!(o.len(), 1, "the miracle option");
    let o2 = o[0].clone();
    run(&mut g, P0, &o2);
    assert!(!g.player(P0).hand.contains(&tm));
}

#[test]
fn temporal_trespass_delves_and_takes_an_extra_turn() {
    let mut g = yuriko_table();
    let c = card(&g, "Temporal Trespass");
    g.player_mut(P0).gy = ["Island", "Swamp", "Island"].iter().map(|n| card(&g, n)).collect();
    let f = g.registry.get(c).unwrap().self_cost.unwrap();
    assert_eq!(f(&g, P0, c), -3, "three graveyard cards pay {{3}}");
    assert_eq!(resolve(&mut g, P0, "Temporal Trespass", Ctx::default()), "exile");
    assert!(g.player(P0).gy.is_empty());
    assert_eq!(g.extra_turns, vec![(P0, false)]);
}

#[test]
fn scheming_symmetry_tutors_for_both_players() {
    let mut g = yuriko_table();
    let (a, b) = (g.player(P0).library.len(), g.player(P1).library.len() + g.player(P2).library.len());
    resolve(&mut g, P0, "Scheming Symmetry", Ctx::default());
    assert_eq!(g.player(P0).library.len(), a, "the card went back on top");
    assert_eq!(g.player(P1).library.len() + g.player(P2).library.len(), b);
    let tut = |g: &Game, p: PlayerId| g.player(p).stats.get("tutored").copied().unwrap_or(0);
    assert_eq!(tut(&g, P0), 1, "you put a card on top");
    assert_eq!(tut(&g, P1) + tut(&g, P2), 1, "and one opponent did");
}

#[test]
fn shadow_of_mortality_costs_less_the_more_life_you_lost() {
    let mut g = yuriko_table();
    let c = card(&g, "Shadow of Mortality");
    let f = g.registry.get(c).unwrap().self_cost.unwrap();
    assert_eq!(f(&g, P0, c), 0);
    g.player_mut(P0).life = 28;
    assert_eq!(f(&g, P0, c), -12);
}

#[test]
fn agadeems_awakening_returns_one_creature_per_mana_value() {
    let mut g = yuriko_table();
    g.player_mut(P0).gy = ["Ornithopter", "Faerie Seer", "Slither Blade", "Moon-Circuit Hacker", "Consecrated Sphinx"]
        .iter()
        .map(|n| card(&g, n))
        .collect();
    resolve(&mut g, P0, "Agadeem's Awakening // Agadeem, the Undercrypt", Ctx { x: 2, ..Ctx::default() });
    let back: Vec<&str> = g.player(P0).perms.iter().map(|&m| &*g.perm(m).name).collect();
    assert!(back.contains(&"Ornithopter"), "mana value 0: {back:?}");
    assert_eq!(back.len(), 3, "one each of mana value 0, 1 and 2: {back:?}");
    assert!(g.player(P0).gy.iter().any(|&c| &*g.db.get(c).name == "Consecrated Sphinx"), "too big for X = 2");
}

#[test]
fn sea_gate_restoration_draws_hand_plus_one() {
    let mut g = yuriko_table();
    hand(&mut g, P0, &["Island", "Swamp", "Brainstorm"]);
    resolve(&mut g, P0, "Sea Gate Restoration // Sea Gate, Reborn", Ctx::default());
    assert_eq!(g.player(P0).hand.len(), 7);
    assert!(g.player(P0).t0s.nomax);
}

#[test]
fn a_spell_land_is_played_as_a_land_when_short() {
    let mut g = yuriko_table();
    let c = hand(&mut g, P0, &["Sea Gate Restoration // Sea Gate, Reborn"])[0];
    let o = hand_opts(&mut g, c, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, P0, &o[0]));
    let l = *g.player(P0).lands.last().unwrap();
    assert_eq!(g.land(l).card(), c);
    assert!(sim_core::engine::mana::land_colors_now(&g, P0, l).has('U'));
}

#[test]
fn clearwater_pathway_picks_the_missing_colour() {
    let mut g = yuriko_table();
    lands(&mut g, P0, "Island", 2, false);
    let c = hand(&mut g, P0, &["Clearwater Pathway // Murkwater Pathway"])[0];
    g.player_mut(P0).hand.retain(|&x| x != c);
    sim_core::engine::turn::play_land_card(&mut g, P0, c, "plays").unwrap();
    let l = *g.player(P0).lands.last().unwrap();
    let cols = sim_core::engine::mana::land_colors_now(&g, P0, l);
    assert!(cols.has('B') && !cols.has('U'), "two Islands: it is Murkwater Pathway");
}

#[test]
fn hydroelectric_specimen_takes_the_removal_aimed_at_yuriko() {
    let mut g = yuriko_table();
    lands(&mut g, P0, "Island", 3, false);
    let y = perm(&mut g, P0, "Moon-Circuit Hacker");
    g.perm_mut(y).is_cmd = true; // as key as the commander
    hand(&mut g, P0, &["Hydroelectric Specimen // Hydroelectric Laboratory"]);
    let bolt = card(&g, "Swords to Plowshares");
    let id = 4242;
    g.stack.push(sim_core::state::StackItem {
        id,
        card: Some(bolt),
        controller: P1,
        ctx: Ctx { target: Some(y), ..Ctx::default() },
        zone: "hand",
        imp: 5.0,
        aff: vec![],
        generic: true,
        kind: sim_core::state::StackKind::Spell,
        name: "Swords to Plowshares".into(),
        passed: vec![],
        countered: false,
        countered_by: None,
    });
    assert!(sim_core::impls::t45_yuriko::specimen_response(&mut g, P0, id).unwrap());
    let s = named(&g, P0, "Hydroelectric Specimen // Hydroelectric Laboratory");
    assert_eq!(s.len(), 1);
    assert_eq!(g.stack[0].ctx.target, Some(s[0]), "the Swords now targets the Specimen");
}
