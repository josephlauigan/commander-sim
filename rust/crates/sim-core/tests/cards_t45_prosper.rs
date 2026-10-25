//! The Tier 4 prosper-rakdos-exile-treasure cards (impls/t45_prosper.rs and the compiled patterns under
//! `// --- Tiers 4/5 (prosper)`) on hand-built positions. Each expectation follows the card's Oracle text.

use sim_core::engine::cast;
use sim_core::engine::hooks::fire_trigger;
use sim_core::engine::life::lose_life;
use sim_core::engine::zones;
use sim_core::hooks::{Action, Call, Event, Opt, Sacrificed};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::t45_prosper::{COMBOS, cast_from_exile, mark_exile_cast, uba_draw};
use sim_core::state::{Ctx, DataKey, Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const PROSPER: &str = "prosper-rakdos-exile-treasure";

fn tbl() -> Game {
    table(&[PROSPER, "seph", "sauron"])
}

fn imp(g: &Game, name: &str) -> sim_core::hooks::CardImpl {
    *g.registry.get(card(g, name)).unwrap_or_else(|| panic!("{name} has no card code"))
}

fn run(g: &mut Game, o: Opt, p: PlayerId) -> bool {
    match o.act.unwrap() {
        Action::Plan { f, arg } => f(g, p, arg).unwrap(),
        Action::Ability { src, f, arg } => f(g, src, p, arg).unwrap(),
        other => panic!("unexpected action {other:?}"),
    }
}

fn resolve(g: &mut Game, p: PlayerId, name: &str) {
    let c = take(g, p, name);
    cast::resolve(g, p, c, &Ctx::default(), "hand").unwrap();
}

/// put these cards on top of p's library, the last one on top
fn stack_top(g: &mut Game, p: PlayerId, names: &[&str]) -> Vec<CardId> {
    let cs: Vec<CardId> = names.iter().map(|n| take(g, p, n)).collect();
    g.player_mut(p).library.extend(&cs);
    cs
}

fn tokens_of(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf && g.perm(m).token).collect()
}

fn sac_treasure(g: &mut Game, p: PlayerId) {
    g.player_mut(p).sac_treasure();
    fire_trigger(g, Event::Sacrifice, Call::Sacrifice { p, what: Sacrificed::Token("Treasure") }).unwrap();
}

// ------------------------------------------------------------------ Delayed Blast Fireball
#[test]
fn fireball_deals_two_from_hand_and_five_when_foretold() {
    let mut g = tbl();
    let small = perm(&mut g, P1, "Dark Confidant"); // a 2/1
    let big = perm(&mut g, P2, "Sheoldred, the Apocalypse"); // a 4/5
    let mine = token(&mut g, P0, 1);
    let c = hand(&mut g, P0, &["Delayed Blast Fireball"])[0];
    let i = imp(&g, "Delayed Blast Fireball");
    (i.self_cast.unwrap())(&mut g, P0, c).unwrap();
    g.player_mut(P0).hand.retain(|&x| x != c);
    cast::resolve(&mut g, P0, c, &Ctx::default(), "hand").unwrap();
    assert_eq!(g.player(P1).life, 38, "2 to each opponent");
    assert!(!g.perm(small).on_bf && g.perm(big).on_bf && g.perm(mine).on_bf, "2 to their creatures only");
    // foretold: {2} now, castable for 3 more from a later turn, and then 5 damage
    let mut g = tbl();
    let big = perm(&mut g, P2, "Sheoldred, the Apocalypse");
    lands(&mut g, P0, "Mountain", 2, false);
    let c = hand(&mut g, P0, &["Delayed Blast Fireball"])[0];
    let f = i.hand_options.unwrap();
    let o = f(&mut g, c, P0, Some(true)).unwrap().pop().expect("foretell");
    assert!(run(&mut g, o, P0));
    assert!(cast_from_exile(&g, P0, c), "foretold: in exile");
    assert_eq!((i.self_cost.unwrap())(&g, P0, c), 99, "not on the turn it was foretold");
    g.player_mut(P0).turns += 1;
    assert_eq!((i.self_cost.unwrap())(&g, P0, c), 3, "{{4}}{{R}}{{R}} from exile");
    (i.self_cast.unwrap())(&mut g, P0, c).unwrap();
    g.player_mut(P0).hand.retain(|&x| x != c);
    cast::resolve(&mut g, P0, c, &Ctx::default(), "hand").unwrap();
    assert_eq!(g.player(P1).life, 35, "5 to each opponent");
    assert!(!g.perm(big).on_bf, "5 kills a 4/5");
}

// ------------------------------------------------------------------ Passionate Archaeologist
#[test]
fn archaeologist_burns_for_each_spell_cast_from_exile() {
    let mut g = tbl();
    let a = perm(&mut g, P0, "Passionate Archaeologist");
    let pr = perm(&mut g, P0, "Prosper, Tome-Bound");
    let f = imp(&g, "Passionate Archaeologist").cast.unwrap();
    let c = take(&mut g, P0, "Chaos Warp"); // mana value 3
    f(&mut g, a, P0, c).unwrap();
    let total = |g: &Game| g.player(P1).life + g.player(P2).life;
    assert_eq!(total(&g), 80, "not from exile, and Prosper is not a commander yet");
    g.perm_mut(pr).is_cmd = true;
    f(&mut g, a, P0, c).unwrap();
    assert_eq!(total(&g), 80, "cast from hand: nothing");
    g.player_mut(P0).impulse.push(c);
    f(&mut g, a, P0, c).unwrap();
    assert_eq!(total(&g), 77, "3 damage to one opponent");
}

// ------------------------------------------------------------------ Uba Mask
#[test]
fn uba_mask_exiles_draws_playable_on_your_own_turn_only() {
    let mut g = tbl();
    assert!(!uba_draw(&mut g, P0).unwrap(), "no Mask: a normal draw");
    perm(&mut g, P2, "Uba Mask");
    let top = *g.player(P0).library.last().unwrap();
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert!(g.player(P0).hand.contains(&top) && g.player(P0).impulse.contains(&top), "playable this turn");
    let top1 = *g.player(P1).library.last().unwrap();
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert!(g.player(P1).exile.contains(&top1) && !g.player(P1).hand.contains(&top1), "not their turn: exiled");
}

// ------------------------------------------------------------------ Pathways
#[test]
fn a_pathway_taps_for_the_face_it_was_played_as() {
    let mut g = tbl();
    lands(&mut g, P0, "Swamp", 2, false);
    hand(&mut g, P0, &["Goldspan Dragon"]); // {3}{R}{R}
    let l = lands(&mut g, P0, "Blightstep Pathway // Searstep Pathway", 1, false)[0];
    (imp(&g, "Blightstep Pathway // Searstep Pathway").land_etb.unwrap())(&mut g, P0, l).unwrap();
    let cols = sim_core::engine::mana::land_colors_now(&g, P0, l);
    assert!(cols.has('R') && !cols.has('B'), "red needed: Searstep");
    // the colours the hand needs change: the face does not
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Toxic Deluge"]);
    let cols = sim_core::engine::mana::land_colors_now(&g, P0, l);
    assert!(cols.has('R') && !cols.has('B'));
}

// ------------------------------------------------------------------ Bonehoard Dracosaur
#[test]
fn dracosaur_exiles_two_for_the_turn_with_a_dinosaur_and_a_treasure() {
    let mut g = tbl();
    let d = perm(&mut g, P0, "Bonehoard Dracosaur");
    let cs = stack_top(&mut g, P0, &["Mountain", "Sol Ring"]);
    let t0 = g.player(P0).treasures;
    (imp(&g, "Bonehoard Dracosaur").upkeep.unwrap())(&mut g, d, P0).unwrap();
    for c in cs {
        assert!(g.player(P0).hand.contains(&c) && g.player(P0).impulse.contains(&c), "playable this turn");
    }
    assert_eq!(g.player(P0).treasures, t0 + 1, "a nonland card: a Treasure");
    let toks = tokens_of(&g, P0);
    assert_eq!(toks.len(), 1, "a land: a Dinosaur");
    assert_eq!((g.perm(toks[0]).pow, g.perm(toks[0]).tgh), (3, 1));
    assert!(g.db.get(card(&g, "Bonehoard Dracosaur")).abilities.is_empty(), "the compiled draw is off");
}

// ------------------------------------------------------------------ Crime Novelist, Disciple, Master, Nightblade
#[test]
fn crime_novelist_grows_and_adds_red_when_you_sacrifice_an_artifact() {
    let mut g = tbl();
    let n = perm(&mut g, P0, "Crime Novelist");
    g.player_mut(P0).treasures = 1;
    sac_treasure(&mut g, P0);
    assert_eq!((g.perm(n).plus, g.player(P0).floating.r), (1, 1));
    g.player_mut(P1).treasures = 1;
    sac_treasure(&mut g, P1);
    assert_eq!(g.perm(n).plus, 1, "not an opponent's sacrifice");
}

#[test]
fn disciple_drains_for_any_artifact_to_the_graveyard() {
    let mut g = tbl();
    perm(&mut g, P0, "Disciple of the Vault");
    let total = |g: &Game| g.player(P1).life + g.player(P2).life;
    g.player_mut(P1).treasures = 1;
    sac_treasure(&mut g, P1);
    assert_eq!(total(&g), 79, "an opponent's Treasure");
    let r = perm(&mut g, P2, "Sol Ring");
    zones::die(&mut g, r, "destroy").unwrap();
    assert_eq!(total(&g), 78, "an artifact destroyed");
    let t = token(&mut g, P1, 2);
    zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!(total(&g), 78, "not a creature token");
}

#[test]
fn marionette_master_fabricates_and_drains_for_its_power() {
    let mut g = tbl();
    g.player_mut(P0).treasures = 3;
    let m = perm(&mut g, P0, "Marionette Master");
    assert_eq!(g.perm(m).plus, 3, "artifacts to sacrifice: the counters");
    let total = |g: &Game| g.player(P1).life + g.player(P2).life;
    sac_treasure(&mut g, P0);
    assert_eq!(total(&g), 76, "a 1/3 with three counters: an opponent loses 4");
    let mut g = tbl();
    perm(&mut g, P0, "Marionette Master");
    assert_eq!(tokens_of(&g, P0).len(), 3, "nothing to sacrifice: three Servos");
    let s = tokens_of(&g, P0)[0];
    zones::die(&mut g, s, "sac").unwrap();
    assert_eq!(total(&g), 79, "a Servo is an artifact: 1 life (its power)");
}

#[test]
fn nightblade_drains_when_a_token_leaves() {
    let mut g = tbl();
    perm(&mut g, P0, "Nadier's Nightblade");
    let t = token(&mut g, P0, 1);
    zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!((g.player(P1).life, g.player(P2).life, g.player(P0).life), (39, 39, 41));
    g.player_mut(P0).treasures = 1;
    sac_treasure(&mut g, P0);
    assert_eq!(g.player(P1).life, 38, "a Treasure is a token");
    let o = token(&mut g, P1, 1);
    zones::die(&mut g, o, "destroy").unwrap();
    assert_eq!(g.player(P1).life, 38, "not an opponent's token");
}

// ------------------------------------------------------------------ Ob Nixilis, Captive Kingpin
#[test]
fn ob_nixilis_triggers_once_per_event_of_exactly_one_life() {
    let mut g = tbl();
    let ob = perm(&mut g, P0, "Ob Nixilis, Captive Kingpin");
    let lib = g.player(P0).library.len();
    for q in [P1, P2] {
        lose_life(&mut g, q, 1, Some(P0), "drain", None).unwrap();
    }
    assert_eq!(g.perm(ob).plus, 1, "each opponent losing 1 to one effect: one trigger");
    assert_eq!(g.player(P0).library.len(), lib - 1);
    let c = *g.player(P0).hand.last().unwrap();
    assert!(g.player(P0).impulse_long.iter().any(|x| x.0 == c), "playable until the next end step");
    g.tick().unwrap();
    lose_life(&mut g, P1, 2, Some(P0), "drain", None).unwrap();
    assert_eq!(g.perm(ob).plus, 1, "2 life: no trigger");
    lose_life(&mut g, P1, 1, Some(P0), "drain", None).unwrap();
    assert_eq!(g.perm(ob).plus, 2, "a new event");
    lose_life(&mut g, P0, 1, Some(P1), "drain", None).unwrap();
    assert_eq!(g.perm(ob).plus, 2, "not you");
}

// ------------------------------------------------------------------ Rain of Riches, Wild-Magic Sorcerer
#[test]
fn rain_of_riches_cascades_the_first_treasure_paid_spell() {
    let mut g = tbl();
    let r = perm(&mut g, P0, "Rain of Riches");
    let c = take(&mut g, P0, "Goldspan Dragon");
    stack_top(&mut g, P0, &["Sol Ring", "Mountain"]);
    let f = imp(&g, "Rain of Riches").cast.unwrap();
    g.cast_treasure = Some((c, 0));
    f(&mut g, r, P0, c).unwrap();
    assert!(g.player(P0).perms.iter().all(|&m| g.perm(m).name != "Sol Ring"), "no Treasure mana: no cascade");
    g.cast_treasure = Some((c, 1));
    f(&mut g, r, P0, c).unwrap();
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).on_bf && &*g.perm(m).name == "Sol Ring"), "cascade");
    let ring = card(&g, "Sol Ring");
    assert!(cast_from_exile(&g, P0, ring), "a cascaded card is cast from exile (Prosper's Treasure)");
    stack_top(&mut g, P0, &["Arcane Signet"]);
    f(&mut g, r, P0, c).unwrap();
    assert!(g.player(P0).perms.iter().all(|&m| g.perm(m).name != "Arcane Signet"), "the first one only");
}

#[test]
fn wild_magic_sorcerer_cascades_the_first_spell_cast_from_exile() {
    let mut g = tbl();
    let s = perm(&mut g, P0, "Wild-Magic Sorcerer");
    let c = take(&mut g, P0, "Goldspan Dragon");
    stack_top(&mut g, P0, &["Sol Ring"]);
    let f = imp(&g, "Wild-Magic Sorcerer").cast.unwrap();
    f(&mut g, s, P0, c).unwrap();
    assert!(g.player(P0).perms.iter().all(|&m| g.perm(m).name != "Sol Ring"), "cast from hand: no cascade");
    g.player_mut(P0).impulse.push(c);
    f(&mut g, s, P0, c).unwrap();
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).on_bf && &*g.perm(m).name == "Sol Ring"), "cascade");
}

// ------------------------------------------------------------------ Weftstalker Ardent
#[test]
fn weftstalker_pings_for_entering_artifacts_and_warps() {
    let mut g = tbl();
    perm(&mut g, P0, "Weftstalker Ardent");
    zones::add_treasure(&mut g, P0, 2).unwrap();
    assert_eq!(g.player(P1).life, 38, "two Treasures: two pings");
    perm(&mut g, P0, "Sol Ring");
    assert_eq!(g.player(P2).life, 37, "an artifact enters");
    perm(&mut g, P1, "Sol Ring");
    assert_eq!(g.player(P2).life, 37, "not an opponent's");
    // warp {R}: exiled at the end step, then castable from exile later
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 1, false);
    perm(&mut g, P0, "Prosper, Tome-Bound");
    let c = hand(&mut g, P0, &["Weftstalker Ardent"])[0];
    let o = (imp(&g, "Weftstalker Ardent").hand_options.unwrap())(&mut g, c, P0, Some(false)).unwrap().pop();
    assert!(run(&mut g, o.expect("warp"), P0));
    let m = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).cd == Some(c)).expect("on the battlefield");
    assert!(g.perm(m).data.truthy(DataKey::Warp));
    (imp(&g, "Weftstalker Ardent").end_step.unwrap())(&mut g, m, P0).unwrap();
    assert!(!g.perm(m).on_bf && g.player(P0).hand.contains(&c) && cast_from_exile(&g, P0, c), "in exile, castable");
}

// ------------------------------------------------------------------ Hex Magic, Ignite the Future
#[test]
fn hex_magic_exiles_the_hand_playable_and_draws_as_many() {
    let mut g = tbl();
    let held = hand(&mut g, P0, &["Sol Ring", "Mountain", "Terminate"]);
    let c = take(&mut g, P0, "Hex Magic");
    let lib = g.player(P0).library.len();
    cast::resolve(&mut g, P0, c, &Ctx::default(), "hand").unwrap();
    assert_eq!(g.player(P0).hand.len(), 6, "three exiled (held, marked) and three drawn");
    assert_eq!(g.player(P0).library.len(), lib - 3);
    let until = g.player(P0).turns + 1;
    for c in held {
        assert!(g.player(P0).impulse_long.contains(&(c, until)), "playable until the end of your next turn");
    }
}

#[test]
fn ignite_the_future_exiles_three_until_next_turn_free_when_flashed_back() {
    let mut g = tbl();
    let cs = stack_top(&mut g, P0, &["Mountain", "Arcane Signet", "Sol Ring"]);
    resolve(&mut g, P0, "Ignite the Future");
    let until = g.player(P0).turns + 1;
    for &c in &cs {
        assert!(g.player(P0).impulse_long.contains(&(c, until)));
    }
    // flashed back: the nonland cards are cast free
    let mut g = tbl();
    stack_top(&mut g, P0, &["Mountain", "Arcane Signet", "Sol Ring"]);
    let c = take(&mut g, P0, "Ignite the Future");
    g.cur_cast = Some((c, Ctx::default(), "gy"));
    (imp(&g, "Ignite the Future").self_cast.unwrap())(&mut g, P0, c).unwrap();
    g.cur_cast = None;
    cast::resolve(&mut g, P0, c, &Ctx::default(), "gy").unwrap();
    let names: Vec<&str> = g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf).map(|&m| &*g.perm(m).name).collect();
    assert!(names.contains(&"Sol Ring") && names.contains(&"Arcane Signet"), "{names:?}");
}

// ------------------------------------------------------------------ Etali, Primal Storm
#[test]
fn etali_casts_the_nonland_cards_it_exiles_free() {
    let mut g = tbl();
    let e = perm(&mut g, P0, "Etali, Primal Storm");
    stack_top(&mut g, P0, &["Sol Ring"]);
    stack_top(&mut g, P1, &["Mountain"]);
    let theirs = stack_top(&mut g, P2, &["Arcane Signet"])[0];
    g.step = Step::Combat;
    (imp(&g, "Etali, Primal Storm").attack.unwrap())(&mut g, e, P0, &[e], P1).unwrap();
    let mine: Vec<&str> = g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf).map(|&m| &*g.perm(m).name).collect();
    assert!(mine.contains(&"Sol Ring") && mine.contains(&"Arcane Signet"), "{mine:?}");
    assert!(g.player(P1).exile.contains(&card(&g, "Mountain")), "a land stays in exile");
    assert!(cast_from_exile(&g, P0, theirs));
}

// ------------------------------------------------------------------ Rakdos Charm
#[test]
fn rakdos_charm_has_its_creature_backlash_mode() {
    let mut g = tbl();
    for _ in 0..3 {
        token(&mut g, P1, 1);
    }
    g.player_mut(P1).life = 3;
    resolve(&mut g, P0, "Rakdos Charm");
    assert!(g.player(P1).life <= 0, "each creature deals 1 damage to its controller");
}

// ------------------------------------------------------------------ Prosper's Treasure, the combo
#[test]
fn prosper_makes_a_treasure_for_a_cascaded_card() {
    let mut g = tbl();
    let pr = perm(&mut g, P0, "Prosper, Tome-Bound");
    let c = take(&mut g, P0, "Sol Ring");
    let f = imp(&g, "Prosper, Tome-Bound").cast.unwrap();
    let t0 = g.player(P0).treasures;
    f(&mut g, pr, P0, c).unwrap();
    assert_eq!(g.player(P0).treasures, t0);
    mark_exile_cast(&mut g, P0, c);
    f(&mut g, pr, P0, c).unwrap();
    assert_eq!(g.player(P0).treasures, t0 + 1);
}

#[test]
fn citadel_top_reservoir_combo_needs_all_three_on_your_turn() {
    let mut g = tbl();
    let ready = |g: &Game| (COMBOS[0].ready)(g, P0).0;
    perm(&mut g, P0, "Bolas's Citadel");
    perm(&mut g, P0, "Aetherflux Reservoir");
    assert!(!ready(&g), "no Top");
    let t = perm(&mut g, P0, "Sensei's Divining Top");
    assert!(ready(&g));
    g.perm_mut(t).tapped = true;
    assert!(!ready(&g), "a tapped Top can't draw");
    g.perm_mut(t).tapped = false;
    g.active = Some(P1);
    assert!(!ready(&g), "not on an opponent's turn");
    g.active = Some(P0);
    g.player_mut(P0).life = 2;
    assert!(!ready(&g), "too little life to start");
}

#[test]
fn pathways_and_exotic_orchards_on_both_sides_read_their_colours_without_looping() {
    // Pathways put onto the battlefield without being played (no face chosen) enter front face up; each Orchard
    // reads the other player's Pathway, which once worked its face out again through this Orchard, without end
    let mut g = table(&["prosper-rakdos-exile-treasure", "yuriko-dimir-ninjas"]);
    let mine = lands(&mut g, P0, "Blightstep Pathway // Searstep Pathway", 1, false)[0];
    let my_orchard = lands(&mut g, P0, "Exotic Orchard", 1, false)[0];
    let theirs = lands(&mut g, P1, "Clearwater Pathway // Murkwater Pathway", 1, false)[0];
    lands(&mut g, P1, "Exotic Orchard", 1, false);
    let cols = |g: &Game, p, l| sim_core::engine::mana::land_colors_now(g, p, l);
    assert!(cols(&g, P0, mine).has('B') && !cols(&g, P0, mine).has('R'), "Blightstep's front face");
    assert!(cols(&g, P1, theirs).has('U') && !cols(&g, P1, theirs).has('B'), "Clearwater's front face");
    let _ = cols(&g, P0, my_orchard); // the call that looped
}
