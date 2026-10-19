//! The titania deck's cards (impls/titania.rs) on hand-built positions. Seat 0 plays the titania deck, seat 1 the seph deck.

use sim_core::ai::brain;
use sim_core::engine::turn::crack_fetch;
use sim_core::engine::values::{epow, etgh};
use sim_core::engine::{cast, combat, zones};
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::titania;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["titania", "seph"])
}

fn options(g: &mut Game, p: PlayerId, post: bool) -> Vec<Opt> {
    brain::main_options(g, p, post).unwrap()
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn elementals(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).on_bf && g.perm(m).token && epow(g, m) == 5 && etgh(g, m) == 3)
        .collect()
}

fn cast_spell(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
    c
}

fn gy_to_top(g: &mut Game, p: PlayerId, names: &[&str]) {
    for n in names {
        let c = take(g, p, n);
        g.player_mut(p).library.push(c);
    }
}

// ------------------------------------------------------------------ Titania herself
#[test]
fn titania_makes_a_5_3_for_each_fetch_land_sacrificed() {
    for fetch in ["Evolving Wilds", "Terramorphic Expanse", "Fabled Passage"] {
        let mut g = game();
        perm(&mut g, P0, "Titania, Protector of Argoth");
        let l = lands(&mut g, P0, fetch, 1, false)[0];
        crack_fetch(&mut g, P0, l).unwrap();
        assert!(in_gy(&g, P0, fetch), "{fetch} is in the graveyard");
        assert_eq!(elementals(&g, P0).len(), 1, "{fetch}: one 5/3 Elemental");
    }
}

#[test]
fn titania_makes_a_5_3_when_a_land_dies_any_other_way() {
    let mut g = game();
    perm(&mut g, P0, "Titania, Protector of Argoth");
    let l = lands(&mut g, P0, "Forest", 2, false);
    sim_core::impls::lands::sac_land(&mut g, P0, l[0]).unwrap();
    assert_eq!(elementals(&g, P0).len(), 1, "a sacrificed Forest");
    sim_core::impls::fixes::destroy_land(&mut g, P0, l[1]).unwrap();
    assert_eq!(elementals(&g, P0).len(), 2, "a destroyed Forest");
    // an opponent's land does not count
    let theirs = lands(&mut g, P1, "Swamp", 1, false)[0];
    sim_core::impls::lands::sac_land(&mut g, P1, theirs).unwrap();
    assert_eq!(elementals(&g, P0).len(), 2);
}

#[test]
fn titania_enters_and_returns_the_fetch_land_then_cracks_it() {
    let mut g = game();
    for n in ["Forest", "Evolving Wilds"] {
        let c = take(&mut g, P0, n);
        g.player_mut(P0).gy.push(c);
    }
    let t = hand(&mut g, P0, &["Titania, Protector of Argoth"])[0];
    zones::enter(&mut g, P0, t, zones::Enter { sick: false, ..Default::default() }).unwrap();
    // the fetch land came back (not the Forest) and was cracked at once: a 5/3 and a basic
    assert!(in_gy(&g, P0, "Evolving Wilds"), "the fetch land was cracked again");
    assert!(in_gy(&g, P0, "Forest"), "the Forest stays in the graveyard");
    assert_eq!(elementals(&g, P0).len(), 1);
    assert_eq!(g.player(P0).lands.len(), 1, "the basic it found");
}

#[test]
fn titania_returns_a_basic_untapped_when_there_is_no_fetch_land() {
    let mut g = game();
    let c = take(&mut g, P0, "Forest");
    g.player_mut(P0).gy.push(c);
    let t = hand(&mut g, P0, &["Titania, Protector of Argoth"])[0];
    zones::enter(&mut g, P0, t, zones::Enter { sick: false, ..Default::default() }).unwrap();
    assert_eq!(g.player(P0).lands.len(), 1);
    assert!(!g.land(g.player(P0).lands[0]).tapped, "the land returns untapped");
}

#[test]
fn a_fetch_land_with_nothing_to_find_is_still_sacrificed() {
    let mut g = game();
    perm(&mut g, P0, "Titania, Protector of Argoth");
    let db = g.db.clone();
    g.player_mut(P0).library.retain(|&c| !db.get(c).land);
    let l = lands(&mut g, P0, "Terramorphic Expanse", 1, false)[0];
    crack_fetch(&mut g, P0, l).unwrap();
    assert!(in_gy(&g, P0, "Terramorphic Expanse"));
    assert_eq!(elementals(&g, P0).len(), 1);
}

// ------------------------------------------------------------------ fetch lands held back
#[test]
fn a_fetch_land_is_held_while_titania_can_be_cast_next_turn_and_cracked_when_she_arrives() {
    let mut g = game();
    g.player_mut(P0).turns = 5;
    lands(&mut g, P0, "Forest", 5, false);
    let fetch = hand(&mut g, P0, &["Evolving Wilds"])[0];
    assert!(titania::hold_fetch(&g, P0), "five mana and Titania in the command zone: hold");
    assert!(titania::land_adjust(&g, P0, fetch) > 0.0);
    // with only two lands there is no Titania in sight: crack for mana
    let mut g2 = game();
    lands(&mut g2, P0, "Forest", 2, false);
    assert!(!titania::hold_fetch(&g2, P0));
}

#[test]
fn held_fetch_lands_are_cracked_once_titania_is_out() {
    let mut g = game();
    lands(&mut g, P0, "Forest", 5, false);
    let l = lands(&mut g, P0, "Evolving Wilds", 1, false)[0];
    let _ = l;
    let t = hand(&mut g, P0, &["Titania, Protector of Argoth"])[0];
    zones::enter(&mut g, P0, t, zones::Enter { sick: false, ..Default::default() }).unwrap();
    // her enter trigger cracks the fetch land that was waiting
    assert!(in_gy(&g, P0, "Evolving Wilds"));
    assert_eq!(elementals(&g, P0).len(), 1);
}

#[test]
fn the_crack_option_is_offered_with_titania_out() {
    let mut g = game();
    perm(&mut g, P0, "Titania, Protector of Argoth");
    lands(&mut g, P0, "Evolving Wilds", 1, false);
    let opts = options(&mut g, P0, false);
    assert!(
        opts.iter().any(|o| o.label.starts_with("crack the fetch lands")),
        "{:?}",
        opts.iter().map(|o| &o.label).collect::<Vec<_>>()
    );
}

#[test]
fn garruks_uprising_is_cast_before_the_fetch_lands_are_cracked() {
    let mut g = game();
    perm(&mut g, P0, "Titania, Protector of Argoth");
    lands(&mut g, P0, "Forest", 3, false);
    lands(&mut g, P0, "Evolving Wilds", 1, false);
    hand(&mut g, P0, &["Garruk's Uprising"]);
    let o = options(&mut g, P0, false);
    let crack = o.iter().find(|o| o.label.starts_with("crack the fetch lands")).unwrap();
    sim_core::ai::act::perform(&mut g, P0, crack.act.as_ref().unwrap()).unwrap();
    assert!(on_bf(&g, P0, "Garruk's Uprising").is_some(), "Uprising first");
    assert_eq!(elementals(&g, P0).len(), 1);
    // the 5/3 entered with Uprising out: a card drawn for it (Titania herself, a 5-power Elemental)
    assert!(g.player(P0).hand.len() >= 2, "Uprising's own draw and the Elemental's");
}

#[test]
fn satyr_wayfinder_leaves_the_fetch_land_in_the_graveyard() {
    let mut g = game();
    gy_to_top(&mut g, P0, &["Evolving Wilds", "Forest"]);
    let w = hand(&mut g, P0, &["Satyr Wayfinder"])[0];
    zones::enter(&mut g, P0, w, zones::Enter { sick: false, ..Default::default() }).unwrap();
    assert!(in_hand(&g, P0, "Forest"), "the basic goes to hand");
    assert!(in_gy(&g, P0, "Evolving Wilds"), "the fetch land is left for Titania");
}

// ------------------------------------------------------------------ the fillers
#[test]
fn something_worth_saving_mills_four_takes_a_permanent_and_gains_one() {
    let mut g = game();
    gy_to_top(&mut g, P0, &["Evolving Wilds", "Cultivate", "Hornet Queen", "Rampant Growth"]);
    let life = g.player(P0).life;
    hand(&mut g, P0, &["Something Worth Saving"]);
    let n = g.player(P0).library.len();
    let c = g.player(P0).hand[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert_eq!(g.player(P0).library.len(), n - 4);
    assert!(in_hand(&g, P0, "Hornet Queen"), "the best permanent card");
    assert!(in_gy(&g, P0, "Evolving Wilds") && in_gy(&g, P0, "Cultivate"));
    assert_eq!(g.player(P0).life, life + 1);
}

#[test]
fn scout_the_city_look_around_mills_three_takes_a_permanent_and_gains_three() {
    let mut g = game();
    gy_to_top(&mut g, P0, &["Evolving Wilds", "Cultivate", "Ghalta the Unstoppable"]);
    let life = g.player(P0).life;
    cast_spell(&mut g, P0, "Scout the City");
    assert!(in_hand(&g, P0, "Ghalta the Unstoppable"));
    assert_eq!(g.player(P0).life, life + 3);
}

#[test]
fn scout_the_city_bring_down_destroys_a_flier() {
    let mut g = game();
    let f = perm(&mut g, P1, "Serra Angel");
    assert!(g.perm(f).on_bf);
    let life = g.player(P0).life;
    let c = hand(&mut g, P0, &["Scout the City"])[0];
    let n = g.player(P0).library.len();
    let target = on_bf(&g, P1, "Serra Angel");
    cast::cast_card(&mut g, P0, c, "hand", Ctx { target, ..Ctx::default() }).unwrap();
    assert!(on_bf(&g, P1, "Serra Angel").is_none(), "destroyed");
    assert_eq!(g.player(P0).library.len(), n, "nothing milled");
    assert_eq!(g.player(P0).life, life);
}

#[test]
fn malevolent_rumble_reveals_four_takes_a_permanent_and_makes_a_spawn() {
    let mut g = game();
    gy_to_top(&mut g, P0, &["Evolving Wilds", "Cultivate", "Hornet Queen", "Rampant Growth"]);
    cast_spell(&mut g, P0, "Malevolent Rumble");
    assert!(in_hand(&g, P0, "Hornet Queen"));
    assert!(in_gy(&g, P0, "Rampant Growth") && in_gy(&g, P0, "Evolving Wilds"), "the rest to the graveyard");
    let spawn = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .find(|&m| g.perm(m).on_bf && g.perm(m).token && g.perm(m).ttypes.contains(&"spawn"))
        .expect("an Eldrazi Spawn");
    assert_eq!((epow(&g, spawn), etgh(&g, spawn)), (0, 1));
    // it can be sacrificed for mana: with no lands, the spawn pays for a one-mana spell
    assert!(sim_core::engine::mana::total_mana(&g, P0, false) >= 1);
}

#[test]
fn follow_the_lumarets_takes_one_card_or_two_with_infusion() {
    let mut g = game();
    gy_to_top(&mut g, P0, &["Cultivate", "Forest", "Hornet Queen", "Ghalta the Unstoppable"]);
    cast_spell(&mut g, P0, "Follow the Lumarets");
    let taken = g.player(P0).hand.len();
    assert_eq!(taken, 1, "no life gained: one card");
    let mut g = game();
    gy_to_top(&mut g, P0, &["Cultivate", "Forest", "Hornet Queen", "Ghalta the Unstoppable"]);
    sim_core::engine::life::gain(&mut g, P0, 2).unwrap();
    cast_spell(&mut g, P0, "Follow the Lumarets");
    assert_eq!(g.player(P0).hand.len(), 2, "infusion: two cards");
}

// ------------------------------------------------------------------ Glorious Decay
#[test]
fn glorious_decay_destroys_an_artifact_damages_a_flier_or_exiles_and_draws() {
    let mut g = game();
    let a = perm(&mut g, P1, "Sol Ring");
    let c = hand(&mut g, P0, &["Glorious Decay"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { target: Some(a), ..Ctx::default() }).unwrap();
    assert!(on_bf(&g, P1, "Sol Ring").is_none());
    // 4 damage kills a 2/3 flier but a 5-toughness flier is not a target
    let mut g = game();
    perm(&mut g, P1, "Baleful Strix");
    let c = hand(&mut g, P0, &["Glorious Decay"])[0];
    let t = on_bf(&g, P1, "Baleful Strix");
    cast::cast_card(&mut g, P0, c, "hand", Ctx { target: t, ..Ctx::default() }).unwrap();
    assert!(on_bf(&g, P1, "Baleful Strix").is_none());
    // no target: exile a card from the graveyard and draw
    let mut g = game();
    let x = take(&mut g, P1, "Hornet Queen");
    g.player_mut(P1).gy.push(x);
    let h = g.player(P0).hand.len();
    cast_spell(&mut g, P0, "Glorious Decay");
    assert!(!in_gy(&g, P1, "Hornet Queen"), "exiled");
    assert!(g.player(P1).exile.contains(&x));
    assert_eq!(g.player(P0).hand.len(), h + 1, "drew a card");
}

// ------------------------------------------------------------------ Annoyed Altisaur
#[test]
fn annoyed_altisaur_cascades_into_a_cheaper_spell() {
    let mut g = game();
    lands(&mut g, P0, "Forest", 3, false);
    // the top of the library: a land (exiled), then Rampant Growth (cast free), then the rest
    gy_to_top(&mut g, P0, &["Cultivate", "Rampant Growth", "Forest"]);
    let c = hand(&mut g, P0, &["Annoyed Altisaur"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert!(in_gy(&g, P0, "Rampant Growth"), "Rampant Growth was cast free");
    assert!(in_hand(&g, P0, "Cultivate") || !in_gy(&g, P0, "Cultivate"), "Cultivate was not touched");
}

// ------------------------------------------------------------------ Arachnogenesis
#[test]
fn arachnogenesis_makes_a_spider_per_attacker_and_stops_non_spider_damage() {
    let mut g = game();
    g.active = Some(P1);
    lands(&mut g, P0, "Forest", 3, false);
    hand(&mut g, P0, &["Arachnogenesis"]);
    let a = token(&mut g, P1, 10);
    let b = token(&mut g, P1, 10);
    g.in_combat = vec![a, b];
    let life = g.player(P0).life;
    combat::resolve_combat(&mut g, P1, &[a, b], P0, &[]).unwrap();
    assert_eq!(g.player(P0).life, life, "no damage got through");
    let spiders =
        g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).ttypes.contains(&"spider")).count();
    assert_eq!(spiders, 2, "one Spider per attacker");
    assert!(in_gy(&g, P0, "Arachnogenesis"));
}

#[test]
fn arachnogenesis_is_held_for_a_small_attack() {
    let mut g = game();
    g.active = Some(P1);
    lands(&mut g, P0, "Forest", 3, false);
    hand(&mut g, P0, &["Arachnogenesis"]);
    let a = token(&mut g, P1, 2);
    g.in_combat = vec![a];
    combat::resolve_combat(&mut g, P1, &[a], P0, &[]).unwrap();
    assert!(in_hand(&g, P0, "Arachnogenesis"), "2 damage of 40 life is not worth a card");
    assert_eq!(g.player(P0).life, 38);
}

#[test]
fn spiders_are_the_only_creatures_that_still_deal_combat_damage() {
    let mut g = game();
    let a = token(&mut g, P1, 5);
    let s = token(&mut g, P0, 1);
    g.perm_mut(s).ttypes.push("spider");
    g.fog_spiders = Some(g.turn_stamp());
    assert!(sim_core::cardcode::damage_prevented(&g, a));
    assert!(!sim_core::cardcode::damage_prevented(&g, s));
}

// ------------------------------------------------------------------ Infestation Expert
#[test]
fn infestation_expert_makes_it_day_and_an_insect_and_turns_into_the_werewolf_at_night() {
    let mut g = game();
    let e = perm(&mut g, P0, "Infestation Expert");
    assert_eq!(g.day_night, 1, "the first daybound permanent makes it day");
    assert_eq!((epow(&g, e), etgh(&g, e)), (3, 4));
    let insects = |g: &Game| {
        g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).ttypes.contains(&"insect")).count()
    };
    assert_eq!(insects(&g), 1);
    // P0's turn passes with no spell cast: night when the next turn begins
    titania::day_night_tick(&mut g, P0).unwrap();
    g.player_mut(P0).spells_this_turn = 0;
    titania::day_night_tick(&mut g, P1).unwrap();
    assert_eq!(g.day_night, 2);
    assert_eq!((epow(&g, e), etgh(&g, e)), (4, 5), "Infested Werewolf");
    // night: attacking makes two Insects
    sim_core::engine::hooks::fire_trigger(
        &mut g,
        sim_core::hooks::Event::Attack,
        sim_core::hooks::Call::Attack { p: P0, atk: vec![e], d: P1 },
    )
    .unwrap();
    assert_eq!(insects(&g), 3, "two more at night");
}

#[test]
fn it_becomes_day_again_after_a_turn_with_two_spells() {
    let mut g = game();
    perm(&mut g, P0, "Infestation Expert");
    titania::day_night_tick(&mut g, P0).unwrap();
    g.player_mut(P0).spells_this_turn = 0;
    titania::day_night_tick(&mut g, P1).unwrap();
    assert_eq!(g.day_night, 2);
    g.player_mut(P1).spells_this_turn = 1;
    titania::day_night_tick(&mut g, P0).unwrap();
    assert_eq!(g.day_night, 2, "one spell keeps the night");
    g.player_mut(P0).spells_this_turn = 2;
    titania::day_night_tick(&mut g, P1).unwrap();
    assert_eq!(g.day_night, 1, "two spells: day");
}

// ------------------------------------------------------------------ protecting Titania
#[test]
fn royal_treatment_answers_removal_on_titania() {
    let mut g = game();
    let t = perm(&mut g, P0, "Titania, Protector of Argoth");
    lands(&mut g, P0, "Forest", 2, false);
    hand(&mut g, P0, &["Royal Treatment", "Blossoming Defense"]);
    let saved = titania::titania_protect(&mut g, P0, t, "destroy", Some(P1), None).unwrap();
    assert!(saved);
    assert!(in_gy(&g, P0, "Royal Treatment"), "Royal Treatment first");
    assert!(sim_core::dsl::has_kw(&g, t, "hexproof"));
    // an edict can't be answered
    assert!(!titania::titania_protect(&mut g, P0, t, "edict", Some(P1), None).unwrap());
}

#[test]
fn blossoming_defense_is_the_second_answer_and_the_fetch_lands_are_cracked_when_nothing_saves_her() {
    let mut g = game();
    let t = perm(&mut g, P0, "Titania, Protector of Argoth");
    lands(&mut g, P0, "Forest", 1, false);
    hand(&mut g, P0, &["Blossoming Defense"]);
    assert!(titania::titania_protect(&mut g, P0, t, "destroy", Some(P1), None).unwrap());
    assert_eq!((epow(&g, t), etgh(&g, t)), (7, 5), "+2/+2");
    // no answer left: the held fetch land is cracked in response
    lands(&mut g, P0, "Evolving Wilds", 1, false);
    assert!(!titania::titania_protect(&mut g, P0, t, "destroy", Some(P1), None).unwrap());
    assert!(in_gy(&g, P0, "Evolving Wilds"));
    assert_eq!(elementals(&g, P0).len(), 1);
}

// ------------------------------------------------------------------ the finishers
#[test]
fn the_finisher_waits_until_the_attack_is_lethal_through_the_blockers() {
    let mut g = table_with(&["titania", "seph"], 1, 40);
    let c = hand(&mut g, P0, &["Overwhelming Stampede"])[0];
    token(&mut g, P0, 5);
    token(&mut g, P0, 5);
    assert_eq!(titania::titania_prio(&g, P0, c), 0, "not lethal: 2 x (5+5) = 20 of 40");
    for _ in 0..3 {
        token(&mut g, P0, 5);
    }
    assert_eq!(titania::titania_prio(&g, P0, c), 92, "5 x 10 = 50 of 40");
    // blockers soak up toughness: four 6-toughness walls
    for _ in 0..4 {
        let spec = zones::Tokens { tgh: Some(6), sick: false, ..zones::Tokens::new(1, 0) };
        zones::make_tokens(&mut g, P1, spec).unwrap();
    }
    assert_eq!(titania::titania_prio(&g, P0, c), 0, "50 - 24 = 26 gets through");
}

#[test]
fn titania_is_cast_first_and_the_fillers_come_before_her_with_no_fetch_land_in_the_graveyard() {
    let mut g = game();
    g.player_mut(P0).turns = 5;
    let cmd = g.player(P0).cmd;
    assert!(titania::titania_prio(&g, P0, cmd) >= 90);
    let rumble = hand(&mut g, P0, &["Malevolent Rumble"])[0];
    assert!(titania::titania_prio(&g, P0, rumble) >= 80, "a filler while no fetch land waits");
    let c = take(&mut g, P0, "Evolving Wilds");
    g.player_mut(P0).gy.push(c);
    assert!(titania::titania_prio(&g, P0, rumble) < 80, "a fetch land is waiting: no hurry");
}

// ------------------------------------------------------------------ Primeval Titan
/// Primeval Titan: "Trample. Whenever Primeval Titan enters or attacks, you may search your library for up to two land
/// cards, put them onto the battlefield tapped, then shuffle." In the titania deck the fetch lands go first (each is a
/// 5/3 under Titania once cracked), and a Titan-fetched fetch land stays put until it untaps ({T} to sacrifice).
#[test]
fn primeval_titan_enters_and_attacks_for_two_lands_each_fetch_lands_first() {
    let mut g = game();
    let titania = perm(&mut g, P0, "Titania, Protector of Argoth");
    let _ = titania;
    let t = hand(&mut g, P0, &["Primeval Titan"])[0];
    assert!(g.db.get(t).tags.has(sim_core::tag::Tag::Trample));
    let before = g.player(P0).lands.len();
    zones::enter(&mut g, P0, t, zones::Enter { sick: false, ..Default::default() }).unwrap();
    let lands: Vec<_> = g.player(P0).lands.clone();
    assert_eq!(lands.len(), before + 2, "two lands");
    let names: Vec<String> = lands.iter().map(|&l| g.db.get(g.land(l).cd).name.to_string()).collect();
    assert!(lands.iter().all(|&l| g.land(l).tapped), "all enter tapped: {names:?}");
    let fetches = ["Evolving Wilds", "Terramorphic Expanse", "Fabled Passage"];
    assert!(names.iter().all(|n| fetches.contains(&n.as_str())), "fetch lands first: {names:?}");
    // tapped, they cannot be cracked yet: no Elemental, and the AI offers no crack
    assert_eq!(elementals(&g, P0).len(), 0);
    assert!(!options(&mut g, P0, false).iter().any(|o| o.label.contains("crack the fetch")));
    // they untap next turn: now the crack is offered and makes two 5/3s
    for l in lands {
        g.land_mut(l).tapped = false;
    }
    g.active = Some(P0);
    let o = options(&mut g, P0, false);
    let crack = o.iter().find(|o| o.label.contains("crack the fetch")).expect("crack offered");
    sim_core::ai::act::perform(&mut g, P0, crack.act.as_ref().unwrap()).unwrap();
    assert_eq!(elementals(&g, P0).len(), 2);
    // attacking: two more lands (the last fetch land, then a basic), counted as landfall (a Forest here)
    let titan = on_bf(&g, P0, "Primeval Titan").unwrap();
    let lib_before = g.player(P0).library.len();
    combat::attack_triggers(&mut g, P0, &[titan], P1).unwrap();
    assert_eq!(g.player(P0).library.len(), lib_before - 2, "two more lands left the library");
}

#[test]
fn primeval_titan_takes_basics_when_no_better_land_is_left_and_stops_at_an_empty_library() {
    let mut g = game();
    let f = card(&g, "Forest");
    g.player_mut(P0).library = vec![f];
    let t = hand(&mut g, P0, &["Primeval Titan"])[0];
    zones::enter(&mut g, P0, t, zones::Enter { sick: false, ..Default::default() }).unwrap();
    assert_eq!(g.player(P0).lands.len(), 1, "up to two: only one land to find");
    assert!(g.player(P0).library.is_empty());
}

#[test]
fn the_ai_casts_primeval_titan() {
    let mut g = game();
    g.active = Some(P0);
    hand(&mut g, P0, &["Primeval Titan"]);
    lands(&mut g, P0, "Forest", 6, false);
    let o = options(&mut g, P0, false);
    assert!(o.iter().any(|o| o.label.contains("Primeval Titan")), "castable: {:?}", o.iter().map(|o| &o.label).collect::<Vec<_>>());
}
