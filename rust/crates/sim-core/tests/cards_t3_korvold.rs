//! The Tier 3 korvold-jund-sacrifice and tergrid-mono-black-disruption cards (impls/t3_korvold.rs and the compiled
//! patterns under `// --- Tier 3 (korvold)`) on hand-built positions. Each expectation follows the card's Oracle text.

use sim_core::dsl::{Fired, fire, has_kw};
use sim_core::engine::cast;
use sim_core::engine::hooks::{fire_trigger, run_hook};
use sim_core::engine::values::{epow, etgh};
use sim_core::engine::zones;
use sim_core::hooks::{Action, Call, Event, Opt, Sacrificed};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const KORVOLD: &str = "korvold-jund-sacrifice";
const TERGRID: &str = "tergrid-mono-black-disruption";

fn korvold() -> Game {
    table(&[KORVOLD, "seph", "sauron"])
}

fn tergrid() -> Game {
    table(&[TERGRID, "seph", "sauron"])
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn creature_count(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.is_creature(m)).count()
}

fn to_gy(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = take(g, p, name);
    g.player_mut(p).gy.push(c);
    c
}

fn run(g: &mut Game, o: Opt, p: PlayerId) -> bool {
    match o.act.unwrap() {
        Action::Plan { f, arg } => f(g, p, arg).unwrap(),
        Action::Ability { src, f, arg } => f(g, src, p, arg).unwrap(),
        other => panic!("unexpected action {other:?}"),
    }
}

fn hand_opts(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).and_then(|i| i.hand_options).expect("hand options");
    f(g, c, p, post).unwrap()
}

fn resolve(g: &mut Game, p: PlayerId, name: &str, ctx: Ctx) {
    let c = take(g, p, name);
    cast::resolve(g, p, c, &ctx, "hand").unwrap();
}

fn my_turn(g: &mut Game, p: PlayerId, step: Step) {
    g.active = Some(p);
    g.step = step;
}

// ------------------------------------------------------------------ Rhythm of the Wild
#[test]
fn rhythm_gives_riot_and_makes_creature_spells_uncounterable() {
    let mut g = korvold();
    let r = perm(&mut g, P0, "Rhythm of the Wild");
    let imp = *g.registry.get(card(&g, "Rhythm of the Wild")).unwrap();
    let f = imp.uncounterable.unwrap();
    assert!(f(&g, r, P0, card(&g, "Juri, Master of the Revue")), "my creature spell");
    assert!(!f(&g, r, P0, card(&g, "Pox")), "not a noncreature spell");
    assert!(!f(&g, r, P1, card(&g, "Juri, Master of the Revue")), "not an opponent's");
    // before combat on my turn: haste
    my_turn(&mut g, P0, Step::Main1);
    let c = take(&mut g, P0, "Mazirek, Kraul Death Priest");
    let m = zones::enter(&mut g, P0, c, zones::Enter::default()).unwrap();
    assert!(sim_core::engine::combat::has_haste(&g, m) && g.perm(m).plus == 0, "riot: haste");
    // after combat: the counter
    g.step = Step::Main2;
    let c = take(&mut g, P0, "Juri, Master of the Revue");
    let j = zones::enter(&mut g, P0, c, zones::Enter::default()).unwrap();
    assert_eq!(g.perm(j).plus, 1, "riot: +1/+1 counter");
    // tokens and the opponents' creatures get nothing
    let t = token(&mut g, P0, 1);
    assert_eq!(g.perm(t).plus, 0);
    let o = perm(&mut g, P1, "Mindslicer");
    assert_eq!(g.perm(o).plus, 0);
}

// ------------------------------------------------------------------ Impulsive Pilferer
#[test]
fn pilferer_encore_makes_a_hasty_attacker_per_opponent_that_leaves_a_treasure() {
    let mut g = korvold();
    lands(&mut g, P0, "Mountain", 4, false);
    let c = to_gy(&mut g, P0, "Impulsive Pilferer");
    my_turn(&mut g, P0, Step::Main1);
    let f = g.registry.get(c).and_then(|i| i.gy_options).unwrap();
    let o = f(&mut g, c, P0, Some(false)).unwrap().pop().expect("encore option");
    assert!(run(&mut g, o, P0));
    assert!(g.player(P0).exile.contains(&c) && !g.player(P0).gy.contains(&c));
    let toks: Vec<PermId> =
        g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).on_bf && g.perm(m).token).collect();
    assert_eq!(toks.len(), 2, "one per opponent");
    for &t in &toks {
        assert!(!g.perm(t).sick && g.perm(t).data.truthy(DataKey::EotSac));
        assert!(g.perm(t).data.get(DataKey::MustAttack).is_some());
    }
    let before = g.player(P0).treasures;
    zones::die(&mut g, toks[0], "sac").unwrap(); // sacrificed at the end step: its dies trigger
    assert_eq!(g.player(P0).treasures, before + 1);
}

// ------------------------------------------------------------------ Juri, Master of the Revue
#[test]
fn juri_grows_on_sacrifices_and_burns_for_its_power_when_it_dies() {
    let mut g = korvold();
    let j = perm(&mut g, P0, "Juri, Master of the Revue");
    let t = token(&mut g, P0, 1);
    zones::die(&mut g, t, "sac").unwrap();
    fire_trigger(&mut g, Event::Sacrifice, Call::Sacrifice { p: P0, what: Sacrificed::Token("Treasure") }).unwrap();
    assert_eq!(g.perm(j).plus, 2, "a creature and a Treasure");
    let o = token(&mut g, P1, 1);
    zones::die(&mut g, o, "sac").unwrap();
    assert_eq!(g.perm(j).plus, 2, "an opponent's sacrifice doesn't count");
    assert_eq!(epow(&g, j), 3);
    let (l1, l2) = (g.player(P1).life, g.player(P2).life);
    zones::die(&mut g, j, "destroy").unwrap();
    let lost = (l1 - g.player(P1).life) + (l2 - g.player(P2).life);
    assert_eq!(lost, 3, "3 damage to a player (no creature worth it)");
}

// ------------------------------------------------------------------ Marionette Apprentice
#[test]
fn marionette_apprentice_fabricates_and_drains_on_creatures_and_artifacts() {
    let mut g = korvold();
    let a = perm(&mut g, P0, "Marionette Apprentice");
    let servo: Vec<PermId> =
        g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).on_bf && g.perm(m).token && m != a).collect();
    assert_eq!(servo.len(), 1, "fabricate 1: a Servo");
    let (l1, l2) = (g.player(P1).life, g.player(P2).life);
    zones::die(&mut g, servo[0], "sac").unwrap();
    assert_eq!((g.player(P1).life, g.player(P2).life), (l1 - 1, l2 - 1));
    fire_trigger(&mut g, Event::Sacrifice, Call::Sacrifice { p: P0, what: Sacrificed::Token("Treasure") }).unwrap();
    assert_eq!((g.player(P1).life, g.player(P2).life), (l1 - 2, l2 - 2), "a Treasure is an artifact");
    let o = token(&mut g, P1, 1);
    zones::die(&mut g, o, "sac").unwrap();
    assert_eq!(g.player(P2).life, l2 - 2, "not an opponent's creature");
}

// ------------------------------------------------------------------ Mazirek, Kraul Death Priest
#[test]
fn mazirek_counters_on_any_players_sacrifice() {
    let mut g = korvold();
    let m = perm(&mut g, P0, "Mazirek, Kraul Death Priest");
    let t = token(&mut g, P0, 1);
    let o = token(&mut g, P1, 1);
    zones::die(&mut g, o, "sac").unwrap();
    assert_eq!((g.perm(m).plus, g.perm(t).plus), (1, 1), "an opponent's sacrifice counts");
    fire_trigger(&mut g, Event::Sacrifice, Call::Sacrifice { p: P2, what: Sacrificed::Token("Treasure") }).unwrap();
    assert_eq!(g.perm(m).plus, 2);
    zones::die(&mut g, o, "destroy").unwrap(); // already gone: nothing
    assert_eq!(g.perm(m).plus, 2);
}

// ------------------------------------------------------------------ Ruthless Technomancer
#[test]
fn technomancer_sacrifices_for_treasures_and_reanimates_for_artifacts() {
    let mut g = korvold();
    let big = token(&mut g, P0, 4);
    let before = g.player(P0).treasures;
    let tm = perm(&mut g, P0, "Ruthless Technomancer");
    assert!(!g.perm(big).on_bf, "the 4/4 token goes");
    assert_eq!(g.player(P0).treasures, before + 4, "Treasures equal to its power");
    // {2}{B}, sacrifice X artifacts: a creature card with power X or less
    to_gy(&mut g, P0, "Smaug, Wicked Worm");
    lands(&mut g, P0, "Swamp", 3, false);
    g.player_mut(P0).treasures = 5;
    my_turn(&mut g, P0, Step::Main1);
    let f = g.registry.get(card(&g, "Ruthless Technomancer")).and_then(|i| i.options).unwrap();
    let o = f(&mut g, tm, P0, Some(false)).unwrap().pop().expect("reanimation option");
    assert!(run(&mut g, o, P0));
    assert!(on_bf(&g, P0, "Smaug, Wicked Worm").is_some());
    assert_eq!(g.player(P0).treasures, 0, "five Treasures for a 5-power creature");
}

// ------------------------------------------------------------------ Smaug, Wicked Worm
#[test]
fn smaug_counts_opposing_artifacts_and_draws_off_treasure_mana() {
    let mut g = korvold();
    perm(&mut g, P1, "Sol Ring");
    g.player_mut(P2).treasures = 2;
    let s = perm(&mut g, P0, "Smaug, Wicked Worm");
    assert_eq!(g.player(P0).treasures, 3);
    assert_eq!(g.player(P0).tapped_treasures, 3, "tapped");
    let c = card(&g, "Pox");
    let (h, l) = (g.player(P0).hand.len(), g.player(P0).life);
    g.cast_treasure = Some((c, 0));
    run_hook(&mut g, s, Event::Cast, &Call::Cast { caster: P0, c }).unwrap();
    assert_eq!(g.player(P0).hand.len(), h, "no Treasure mana: nothing");
    g.cast_treasure = Some((c, 1));
    run_hook(&mut g, s, Event::Cast, &Call::Cast { caster: P0, c }).unwrap();
    assert_eq!((g.player(P0).hand.len(), g.player(P0).life), (h + 1, l - 1));
}

// ------------------------------------------------------------------ The Reaver Cleaver (compiled)
#[test]
fn reaver_cleaver_pumps_tramples_and_makes_treasures_per_damage() {
    let mut g = korvold();
    let m = token(&mut g, P0, 3);
    let e = perm(&mut g, P0, "The Reaver Cleaver");
    g.perm_mut(e).attached = Some(m);
    assert_eq!((epow(&g, m), etgh(&g, m)), (4, 4));
    assert!(has_kw(&g, m, "trample"));
    let before = g.player(P0).treasures;
    fire(&mut g, "combat_damage", Fired { attacker: Some(m), defender: Some(P1), dmg: Some(4), ..Default::default() })
        .unwrap();
    assert_eq!(g.player(P0).treasures, before + 4, "that many Treasures");
}

// ------------------------------------------------------------------ Blasphemous Edict
#[test]
fn blasphemous_edict_takes_every_creature_and_tergrid_keeps_theirs() {
    let mut g = tergrid();
    perm(&mut g, P0, "Tergrid, God of Fright // Tergrid's Lantern");
    perm(&mut g, P1, "Mindslicer");
    for _ in 0..3 {
        token(&mut g, P2, 1);
    }
    resolve(&mut g, P0, "Blasphemous Edict", Ctx::default());
    assert_eq!(creature_count(&g, P1) + creature_count(&g, P2), 0);
    assert!(on_bf(&g, P0, "Mindslicer").is_some(), "Tergrid sees it sacrificed before she goes");
    assert!(on_bf(&g, P0, "Tergrid, God of Fright // Tergrid's Lantern").is_none());
}

#[test]
fn blasphemous_edict_costs_b_with_thirteen_creatures() {
    let mut g = tergrid();
    lands(&mut g, P0, "Swamp", 1, false);
    let c = hand(&mut g, P0, &["Blasphemous Edict"])[0];
    for _ in 0..13 {
        token(&mut g, P1, 2);
    }
    my_turn(&mut g, P0, Step::Main1);
    let o = hand_opts(&mut g, c, P0, Some(false)).pop().expect("edict option for {B}");
    assert!(run(&mut g, o, P0));
    assert_eq!(creature_count(&g, P1), 0);
}

// ------------------------------------------------------------------ Bottomless Pit
#[test]
fn bottomless_pit_discards_at_random_in_each_upkeep() {
    let mut g = tergrid();
    let pit = perm(&mut g, P0, "Bottomless Pit");
    hand(&mut g, P1, &["Swamp", "Swamp"]);
    run_hook(&mut g, pit, Event::Upkeep, &Call::Player { p: P1 }).unwrap();
    assert_eq!(g.player(P1).hand.len(), 1);
}

// ------------------------------------------------------------------ Pox and Smallpox
#[test]
fn pox_takes_a_third_of_everything_rounded_up() {
    let mut g = tergrid();
    for p in [P1, P2] {
        lands(&mut g, p, "Swamp", 4, false);
        hand(&mut g, p, &["Swamp", "Swamp", "Swamp", "Swamp"]);
        token(&mut g, p, 1);
        token(&mut g, p, 1);
    }
    g.player_mut(P1).life = 40;
    resolve(&mut g, P0, "Pox", Ctx::default());
    assert_eq!(g.player(P1).life, 26, "40 - 14");
    assert_eq!(g.player(P1).hand.len(), 2, "4 - 2");
    assert_eq!(creature_count(&g, P1), 1, "2 - 1");
    assert_eq!(g.player(P1).lands.len(), 2, "4 - 2");
}

#[test]
fn smallpox_compiles_its_land_sacrifice() {
    let mut g = tergrid();
    lands(&mut g, P1, "Swamp", 3, false);
    hand(&mut g, P1, &["Swamp"]);
    token(&mut g, P1, 1);
    let l = g.player(P1).life;
    resolve(&mut g, P0, "Smallpox", Ctx::default());
    assert_eq!(g.player(P1).life, l - 1);
    assert_eq!((g.player(P1).hand.len(), creature_count(&g, P1), g.player(P1).lands.len()), (0, 0, 2));
}

// ------------------------------------------------------------------ Vicious Rumors (compiled)
#[test]
fn vicious_rumors_mills_each_opponent() {
    let mut g = tergrid();
    let lib = g.player(P1).library.len();
    resolve(&mut g, P0, "Vicious Rumors", Ctx::default());
    assert_eq!(g.player(P1).library.len(), lib - 1, "then mills a card");
}

// ------------------------------------------------------------------ Flare of Malice
#[test]
fn flare_of_malice_takes_the_highest_mana_value_and_can_be_paid_with_a_creature() {
    let mut g = tergrid();
    perm(&mut g, P1, "Smaug, Wicked Worm");
    perm(&mut g, P1, "Juri, Master of the Revue");
    let c = hand(&mut g, P0, &["Flare of Malice"])[0];
    let fodder = perm(&mut g, P0, "Marionette Apprentice");
    my_turn(&mut g, P0, Step::Main1);
    let (pf, ps) = (
        sim_core::engine::values::pval(&g, fodder),
        sim_core::engine::values::pval(&g, on_bf(&g, P1, "Smaug, Wicked Worm").unwrap()),
    );
    let o =
        hand_opts(&mut g, c, P0, Some(false)).pop().unwrap_or_else(|| panic!("flare option: fodder {pf} smaug {ps}"));
    assert!(run(&mut g, o, P0));
    assert!(!g.perm(fodder).on_bf, "the nontoken black creature paid for it");
    assert!(on_bf(&g, P1, "Smaug, Wicked Worm").is_none() && on_bf(&g, P1, "Juri, Master of the Revue").is_some());
}

// ------------------------------------------------------------------ Liliana, Waker of the Dead
#[test]
fn liliana_plus_minus_and_emblem() {
    let mut g = tergrid();
    let li = perm(&mut g, P0, "Liliana, Waker of the Dead");
    let w = g.registry.get(card(&g, "Liliana, Waker of the Dead")).and_then(|i| i.walker).unwrap();
    hand(&mut g, P1, &["Swamp"]);
    g.player_mut(P2).hand.clear();
    let l2 = g.player(P2).life;
    (w[0].eff)(&mut g, P0, li).unwrap();
    assert!(g.player(P1).hand.is_empty());
    assert_eq!(g.player(P2).life, l2 - 3, "can't discard: loses 3");
    // -3: -X/-X, X the cards in my graveyard
    let m = perm(&mut g, P1, "Mazirek, Kraul Death Priest");
    for _ in 0..4 {
        to_gy(&mut g, P0, "Swamp");
    }
    (w[1].eff)(&mut g, P0, li).unwrap();
    assert!(!g.perm(m).on_bf);
    // -7: the emblem returns a creature card from a graveyard with haste at the beginning of combat
    (w[2].eff)(&mut g, P0, li).unwrap();
    sim_core::impls::t3_korvold::liliana_emblem(&mut g, P0).unwrap();
    let back = on_bf(&g, P0, "Mazirek, Kraul Death Priest").expect("Mazirek comes back under my control");
    assert!(!g.perm(back).sick);
}

// ------------------------------------------------------------------ Malakir Rebirth
#[test]
fn malakir_rebirth_returns_the_creature_tapped_and_plays_as_a_tapped_land() {
    let mut g = tergrid();
    let m = perm(&mut g, P0, "Rankle, Master of Pranks");
    lands(&mut g, P0, "Swamp", 1, false);
    hand(&mut g, P0, &["Malakir Rebirth // Malakir Mire"]);
    let l = g.player(P0).life;
    assert!(sim_core::impls::t3_korvold::rebirth_protect(&mut g, P0, m, "destroy").unwrap());
    assert_eq!(g.player(P0).life, l - 2);
    zones::die(&mut g, m, "destroy").unwrap();
    let r = on_bf(&g, P0, "Rankle, Master of Pranks").expect("returns");
    assert!(g.perm(r).tapped);
    // the land face
    let mut g = tergrid();
    let c = hand(&mut g, P0, &["Malakir Rebirth // Malakir Mire"])[0];
    g.player_mut(P0).hand.retain(|&x| x == c);
    my_turn(&mut g, P0, Step::Main1);
    let o = hand_opts(&mut g, c, P0, Some(false)).pop().expect("land option");
    assert!(run(&mut g, o, P0));
    let land = *g.player(P0).lands.last().unwrap();
    assert!(g.land(land).card() == c && g.land(land).tapped);
}

// ------------------------------------------------------------------ Mindslicer, Rankle, Sangromancer, The Raven Man
#[test]
fn mindslicer_empties_every_hand() {
    let mut g = tergrid();
    let m = perm(&mut g, P0, "Mindslicer");
    hand(&mut g, P1, &["Swamp", "Swamp"]);
    hand(&mut g, P0, &["Swamp"]);
    zones::die(&mut g, m, "destroy").unwrap();
    assert!(g.player(P0).hand.is_empty() && g.player(P1).hand.is_empty());
}

#[test]
fn rankle_picks_its_pranks() {
    let mut g = tergrid();
    let r = perm(&mut g, P0, "Rankle, Master of Pranks");
    token(&mut g, P0, 1);
    let theirs = perm(&mut g, P1, "Juri, Master of the Revue");
    hand(&mut g, P1, &["Swamp"]);
    hand(&mut g, P2, &["Swamp"]);
    g.player_mut(P0).hand.clear();
    run_hook(&mut g, r, Event::CombatDamage, &Call::CombatDamage { p: P0, a: r, d: P1, dmg: 3 }).unwrap();
    assert!(g.player(P1).hand.len() <= 1, "discard mode (my hand is empty)");
    assert!(!g.perm(theirs).on_bf, "sacrifice mode: their creature for my token");
    assert!(g.perm(r).on_bf, "Rankle stays: the token went");
}

#[test]
fn sangromancer_gains_on_opponents_discards() {
    let mut g = tergrid();
    perm(&mut g, P0, "Sangromancer");
    let l = g.player(P0).life;
    hand(&mut g, P1, &["Swamp"]);
    zones::discard_index(&mut g, P1, 0).unwrap();
    assert_eq!(g.player(P0).life, l + 3);
    hand(&mut g, P0, &["Swamp"]);
    let i = g.player(P0).hand.len() - 1;
    zones::discard_index(&mut g, P0, i).unwrap();
    assert_eq!(g.player(P0).life, l + 3, "not my own discard");
}

#[test]
fn raven_man_makes_a_bird_at_the_end_step_after_a_discard() {
    let mut g = tergrid();
    let rm = perm(&mut g, P0, "The Raven Man");
    run_hook(&mut g, rm, Event::EndStep, &Call::Player { p: P1 }).unwrap();
    assert_eq!(creature_count(&g, P0), 1, "nobody discarded: no Bird");
    hand(&mut g, P2, &["Swamp"]);
    zones::discard_index(&mut g, P2, 0).unwrap();
    run_hook(&mut g, rm, Event::EndStep, &Call::Player { p: P1 }).unwrap();
    let bird = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).on_bf && g.perm(m).token).expect("a Bird");
    assert!(g.perm(bird).fly && has_kw(&g, bird, "cant_block"));
}
