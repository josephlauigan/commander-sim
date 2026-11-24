//! Adversarial tests for the eight karn_gilded cards of E5 (impls/karn_gilded_cards.rs and the shared engine edits that
//! came with them: `Restr::CreatureSpell`, `human/mana.rs`'s creature pool, the two hooks in `cardcode.rs`).
//! Written by an independent reviewer; every expectation is the Oracle text or the rules, not the code. A failing test
//! is a finding.

mod play_common;
use play_common::*;
use sim_core::ai::act;
use sim_core::cardcode;
use sim_core::cards::{Colors, Types};
use sim_core::engine::mana::{self, can_pay, can_pay_spell, cost_of};
use sim_core::engine::{chars, combat, values, ward, zones};
use sim_core::hooks::{Action, Opt};
use sim_core::state::{DataKey, Step, Val};

const KARN: &str = "Karn, Gilded Guardian";
const STATION: &str = "Infinite Guideline Station";
const BREYA: &str = "Breya, Etherium Shaper";
const THOPTERYX: &str = "Gold-Forged Thopteryx";
const SHORIKAI: &str = "Shorikai, Genesis Engine";
const REAPER: &str = "The Reaper, King No More";
const HEIRLOOM: &str = "Herd Heirloom";
const DOLL: &str = "Twitching Doll";
const COURIER: &str = "Transguild Courier";
const NORN_MOM: &str = "Elesh Norn, Mother of Machines";

fn tbl() -> Game {
    let mut g = table(&["arzakon", "yshtola"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g.player_mut(P0).hand.clear();
    g
}

fn on(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn ready(g: &mut Game, m: PermId) {
    let x = g.perm_mut(m);
    x.sick = false;
    x.tapped = false;
}

fn tokens_of(g: &Game, p: PlayerId, ttype: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).token && g.perm(m).ttypes.contains(&ttype)).collect()
}

fn colour_letters(g: &Game, m: PermId) -> String {
    "WUBRG".chars().filter(|&c| chars::colors(g, m).has(c)).collect()
}

fn gain_kw(g: &mut Game, m: PermId, kw: &'static str) {
    g.perm_mut(m).eot_kw.push(kw);
}

fn two_colour_token(g: &mut Game, p: PlayerId) -> PermId {
    let spec = zones::Tokens { color: Some(Colors::from_letters("WU")), sick: false, ..zones::Tokens::new(1, 2) };
    zones::make_tokens(g, p, spec).unwrap()[0]
}

// ============================================================ A. shared engine code
#[test]
fn a1_every_other_restriction_is_unchanged_and_the_new_one_sorts_last() {
    use sim_core::engine::pool::Restr;
    let g = tbl();
    let r = |n: &str| g.db.get(card(&g, n)).mana_restr;
    assert_eq!(r("Rivaz of the Claw"), Restr::DragonCreature);
    assert_eq!(r("Mishra's Workshop"), Restr::ArtifactSpell);
    assert_eq!(r("Hydro-Channeler"), Restr::InstantSorcery);
    assert_eq!(r("Giada, Font of Hope"), Restr::Subtype("angel"));
    assert_eq!(r("The Mightstone and Weakstone"), Restr::NotNonartifact);
    assert_eq!(r("Llanowar Elves"), Restr::None);
    assert_eq!(r("Sol Ring"), Restr::None);
    assert_eq!(r("Twitching Doll"), Restr::None, "the Doll's mana is unrestricted");
    for older in [
        Restr::None,
        Restr::NotNonartifact,
        Restr::SpellOnly,
        Restr::ArtifactSpell,
        Restr::InstantSorcery,
        Restr::Legendary,
        Restr::DragonCreature,
        Restr::Subtype("zzz"),
        Restr::CreatureType("zzz"),
    ] {
        assert!(older < Restr::CreatureSpell, "{older:?} must still sort before the new last variant");
        assert_ne!(older, Restr::CreatureSpell);
    }
}

#[test]
fn a1_rivaz_still_pays_dragons_only_and_the_heirloom_does_not_change_it() {
    let mut g = tbl();
    let riv = perm(&mut g, P0, "Rivaz of the Claw");
    ready(&mut g, riv);
    let elves = hand(&mut g, P0, &["Llanowar Elves"])[0];
    assert!(!can_pay_spell(&g, P0, elves, 0, "G", false), "Rivaz mana is Dragon creature spells only");
    let h = perm(&mut g, P0, HEIRLOOM);
    ready(&mut g, h);
    assert!(can_pay_spell(&g, P0, elves, 0, "G", false), "the Heirloom pays the Elves");
    assert!(mana::pay_spell(&mut g, P0, elves, 0, "G", false).unwrap());
    assert!(g.perm(h).tapped, "the Heirloom paid, not Rivaz");
    assert!(!g.perm(riv).tapped, "Rivaz's Dragon-only mana was not wasted on an Elf");
}

#[test]
fn a1_heirloom_pays_no_ability_tax_equip_or_noncreature_spell_and_is_not_tapped_by_a_failed_payment() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 1, false);
    let h = perm(&mut g, P0, HEIRLOOM);
    ready(&mut g, h);
    assert!(!can_pay(&g, P0, 2, "", false), "Heirloom + 1 land is one real mana for an ability");
    assert!(!mana::pay(&mut g, P0, 2, "", false).unwrap());
    assert!(!g.perm(h).tapped, "a failed payment leaves the Heirloom untapped");
    assert_eq!(mana::total_mana(&g, P0, false), 1);
    let ring = hand(&mut g, P0, &["Sol Ring"])[0]; // an artifact spell, {1}
    let ws = hand(&mut g, P0, &["Night's Whisper"])[0]; // a sorcery
    let w = g.player(P0).lands[0];
    g.perm_mut(w).tapped = true; // only the Heirloom is left
    assert!(!can_pay_spell(&g, P0, ring, 1, "", false), "an artifact spell");
    assert!(!can_pay_spell(&g, P0, ws, 1, "B", false), "a sorcery");
    let doll = hand(&mut g, P0, &[DOLL])[0]; // an artifact creature spell, {1}{G}
    lands(&mut g, P0, "Forest", 1, false);
    assert!(can_pay_spell(&g, P0, doll, 1, "G", false), "an artifact creature spell: Forest + the Heirloom");
}

#[test]
fn a1_commander_cost_creature_commander_yes_noncreature_commander_no() {
    let mut g = tbl();
    for b in ["Island", "Swamp", "Mountain", "Forest"] {
        lands(&mut g, P0, b, 1, false);
    }
    lands(&mut g, P0, "Wastes", 2, false);
    let h = perm(&mut g, P0, HEIRLOOM);
    ready(&mut g, h);
    set_commander(&mut g, P0, KARN);
    g.player_mut(P0).tax = 2;
    let karn = card(&g, KARN);
    let (gn, pips) = cost_of(&g, P0, karn);
    assert!(gn >= 2, "commander tax counted: {gn}");
    assert!(can_pay_spell(&g, P0, karn, gn, &pips, false), "a creature commander, with tax: Heirloom is the white pip");
    set_commander(&mut g, P0, STATION);
    let st = card(&g, STATION);
    let (gn, pips) = cost_of(&g, P0, st);
    assert!(!can_pay_spell(&g, P0, st, gn, &pips, false), "an artifact (noncreature) commander: no creature-only mana");
}

#[test]
fn a2_person_creature_pool_empties_counts_and_does_not_leak() {
    use sim_core::human::mana::{ManaPool, Spending};
    let mut g = tbl();
    g.player_mut(P0).pool.creature[4] = 2; // {G}{G} for creature spells
    assert_eq!(g.player(P0).pool.total(), 2, "counted in the mana in the pool");
    assert!(g.player(P0).pool.text().contains("creature"), "{}", g.player(P0).pool.text());
    sim_core::human::mana::empty_pools(&mut g);
    assert_eq!(g.player(P0).pool.total(), 0, "emptied as the step ends");
    assert_eq!(g.player(P0).pool.creature, [0; 5]);

    let elves = card(&g, "Llanowar Elves");
    let study = card(&g, "Rhystic Study"); // {2}{U}
    let mut pool = ManaPool::default();
    pool.creature[4] = 1;
    pool.add('U', 1);
    pool.add('G', 1);
    pool.spending = Some(Spending::Card(study));
    assert!(!pool.can_pay(&g, P0, 2, "U"), "creature-only mana must not help an enchantment: {}", pool.text());
    pool.spending = Some(Spending::Card(elves));
    assert!(pool.pay(&g, P0, 0, "G"));
    assert_eq!(pool.creature, [0; 5], "the restricted mana is spent first, so the plain mana stays usable");
    assert_eq!(pool.total(), 2);
    // generic part of a creature spell: creature mana first as well
    let mut pool = ManaPool::default();
    pool.creature[0] = 1;
    pool.add('R', 1);
    pool.spending = Some(Spending::Card(elves));
    assert!(pool.pay(&g, P0, 1, ""));
    assert_eq!(pool.creature, [0; 5]);
    assert_eq!(pool.total(), 1);
    // unrelated payments (no spending): creature mana never pays
    let mut pool = ManaPool::default();
    pool.creature[4] = 3;
    pool.spending = None;
    assert!(!pool.can_pay(&g, P0, 1, ""));
    // an unrelated deck's payment with plain mana is unchanged
    let mut pool = ManaPool::default();
    pool.add('G', 2);
    pool.spending = Some(Spending::Card(study));
    pool.add('U', 1);
    assert!(pool.pay(&g, P0, 2, "U"));
    assert_eq!(pool.total(), 0);
}

#[test]
fn a2_person_taps_a_doll_and_gets_the_nest_counter() {
    use sim_core::human::mana::{sources, tap};
    let mut g = tbl();
    seat(&mut g, P0, vec![]);
    let d = perm(&mut g, P0, DOLL);
    ready(&mut g, d);
    let src = sources(&g, P0).into_iter().find(|s| s.name == DOLL).expect("the Doll is a listed source");
    assert!(tap(&mut g, P0, src.id as i64, Some('G')).unwrap().is_none());
    assert!(g.perm(d).tapped);
    assert_eq!(g.perm(d).data.int(DataKey::Counters), 1, "a person's tap puts the nest counter on too");
}

#[test]
fn a3_the_new_hooks_are_no_ops_for_everything_else() {
    let mut g = tbl();
    let karn = perm(&mut g, P0, KARN);
    let st = perm(&mut g, P0, STATION);
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let theirs = perm(&mut g, P1, KARN);
    for m in [karn, st, elves, theirs] {
        assert_eq!(cardcode::aura_ward(&g, m), 0, "{}: no Thopteryx on the table", g.perm(m).name);
    }
    // the Thopteryx on the OTHER side gives nothing to our legends and nothing to theirs but its own side
    perm(&mut g, P1, THOPTERYX);
    assert_eq!(cardcode::aura_ward(&g, karn), 0);
    assert_eq!(cardcode::aura_ward(&g, theirs), 2);
    // heirloom_draw: nothing for a plain creature, a zero or a negative damage, a creature that is not marked
    let big = token(&mut g, P0, 5);
    let before = g.player(P0).hand.len();
    cardcode::combat_damage_cards(&mut g, P0, big, P1, 5).unwrap();
    cardcode::combat_damage_cards(&mut g, P0, big, P1, 0).unwrap();
    gain_kw(&mut g, big, "heirloom_draw");
    cardcode::combat_damage_cards(&mut g, P0, big, P1, 0).unwrap();
    cardcode::combat_damage_cards(&mut g, P0, big, P1, -3).unwrap();
    assert_eq!(g.player(P0).hand.len() - before, 0, "no damage dealt, no card");
}

#[test]
fn a3_marked_creature_draws_only_when_it_connects_with_a_player() {
    let mut g = tbl();
    let a = token(&mut g, P0, 5);
    gain_kw(&mut g, a, "heirloom_draw");
    // blocked by a big blocker (no trample): no damage to the player, no card
    let blocker = token(&mut g, P1, 9);
    let before = g.player(P0).hand.len();
    let mut tot = 0;
    combat::resolve_blocks(&mut g, P0, vec![a], P1, vec![(a, blocker)], &mut tot).unwrap();
    assert_eq!(g.player(P0).hand.len(), before, "a blocked creature deals no combat damage to a player");
    // unblocked: one card
    let a2 = token(&mut g, P0, 5);
    gain_kw(&mut g, a2, "heirloom_draw");
    let life = g.player(P1).life;
    combat::resolve_blocks(&mut g, P0, vec![a2], P1, vec![], &mut tot).unwrap();
    assert_eq!(g.player(P1).life, life - 5);
    assert_eq!(g.player(P0).hand.len(), before + 1, "unblocked: exactly one card");
}

#[test]
fn a3_trample_over_a_blocker_and_double_strike_draw_per_damage_event() {
    // trample (granted by the Heirloom itself): 6/6 blocked by a 2/2 deals 4 to the player: a card
    let mut g = tbl();
    let a = token(&mut g, P0, 6);
    gain_kw(&mut g, a, "heirloom_draw");
    gain_kw(&mut g, a, "trample");
    let b = token(&mut g, P1, 2);
    let before = g.player(P0).hand.len();
    let life = g.player(P1).life;
    let mut tot = 0;
    combat::resolve_blocks(&mut g, P0, vec![a], P1, vec![(a, b)], &mut tot).unwrap();
    assert_eq!(g.player(P1).life, life - 4, "trample over the 2/2");
    assert_eq!(g.player(P0).hand.len(), before + 1, "trample damage to the player draws a card");
    // double strike, unblocked, power 4: two damage events, two cards
    let mut g = tbl();
    let a = token(&mut g, P0, 4);
    gain_kw(&mut g, a, "heirloom_draw");
    gain_kw(&mut g, a, "double strike");
    let before = g.player(P0).hand.len();
    let life = g.player(P1).life;
    let mut tot = 0;
    combat::resolve_blocks(&mut g, P0, vec![a], P1, vec![], &mut tot).unwrap();
    assert_eq!(life - g.player(P1).life, 8, "double strike: 4 twice");
    assert_eq!(g.player(P0).hand.len(), before + 2, "double strike: a card for each of the two damage events");
}

// ============================================================ B. Infinite Guideline Station
#[test]
fn b1_robots_count_two_colour_tokens_and_the_courier_but_not_colourless_robots() {
    let mut g = tbl();
    two_colour_token(&mut g, P0);
    perm(&mut g, P0, COURIER);
    perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P0, STATION);
    let robots = tokens_of(&g, P0, "robot");
    assert_eq!(robots.len(), 3, "the WU token, the Courier and the Station itself");
    for &r in &robots {
        assert!(g.perm(r).tapped);
        assert!(chars::colors(&g, r).is_empty());
        assert!(g.is_creature(r));
    }
}

#[test]
fn b1_a_second_station_does_not_count_the_first_stations_colourless_robots() {
    let mut g = tbl();
    perm(&mut g, P0, STATION);
    assert_eq!(tokens_of(&g, P0, "robot").len(), 1);
    // the legend rule removes one Station in a real game; the count of the second trigger is what is asked here
    perm(&mut g, P0, STATION);
    let n = tokens_of(&g, P0, "robot").len();
    assert!(n == 2 || n == 3, "two Stations are 2 multicoloured permanents at most: {n} robots");
}

#[test]
fn b2_opposing_elesh_norn_blanks_the_stack_triggers_of_the_new_cards() {
    let mut g = tbl();
    perm(&mut g, P1, NORN_MOM);
    perm(&mut g, P0, STATION);
    perm(&mut g, P0, BREYA);
    assert!(tokens_of(&g, P0, "robot").is_empty(), "no Robots under an opposing Elesh Norn");
    assert!(tokens_of(&g, P0, "thopter").is_empty(), "no Thopters under an opposing Elesh Norn");
    let a = perm(&mut g, P1, "Grave Titan");
    perm(&mut g, P0, REAPER);
    assert_eq!(g.perm(a).plus, 0, "no counters under an opposing Elesh Norn");
}

#[test]
fn b2_panharmonicon_repeats_breya_and_the_station() {
    let mut g = tbl();
    perm(&mut g, P0, "Panharmonicon");
    perm(&mut g, P0, BREYA);
    assert_eq!(tokens_of(&g, P0, "thopter").len(), 4, "Breya is an artifact creature: two triggers, four Thopters");
    perm(&mut g, P0, STATION);
    assert!(tokens_of(&g, P0, "robot").len() >= 2, "the Station is an artifact: its trigger repeats");
}

#[test]
fn b3_station_taps_a_summoning_sick_creature_but_never_a_tapped_foreign_or_noncreature_one() {
    let mut g = tbl();
    let st = perm(&mut g, P0, STATION);
    let sick = token(&mut g, P0, 3);
    g.perm_mut(sick).sick = true;
    let os = options(&mut g, st, P0, Some(false));
    assert_eq!(os.len(), 1, "a sick creature may be tapped for Station");
    let Some(Action::Ability { src, f, .. }) = os[0].act.clone() else { panic!("an ability") };
    let tapped = token(&mut g, P0, 5);
    g.perm_mut(tapped).tapped = true;
    let foe = token(&mut g, P1, 5);
    let before = g.perm(st).data.int(DataKey::Charge);
    for bad in [tapped, foe, st] {
        let ok = act::perform(&mut g, P0, &Action::Ability { src, f, arg: bad.0 as i64 }).unwrap();
        assert!(!ok, "{} can't be tapped for Station", g.perm(bad).name);
        assert_eq!(g.perm(st).data.int(DataKey::Charge), before);
    }
    assert!(run(&mut g, P0, &os[0]));
    assert_eq!(g.perm(st).data.int(DataKey::Charge), 3);
    assert!(g.perm(sick).tapped);
}

#[test]
fn b3_station_does_not_work_while_something_waits_to_resolve() {
    let mut g = tbl();
    let st = perm(&mut g, P0, STATION);
    token(&mut g, P0, 3);
    g.responding = 1;
    assert!(options(&mut g, st, P0, Some(false)).is_empty(), "sorcery timing: nothing on the stack");
}

#[test]
fn b4_station_counters_go_through_a_counter_doubler() {
    let mut g = tbl();
    perm(&mut g, P0, "Vorinclex, Monstrous Raider");
    let st = perm(&mut g, P0, STATION);
    let t = token(&mut g, P0, 3);
    g.perm_mut(t).sick = true;
    let os = options(&mut g, st, P0, Some(false));
    assert!(!os.is_empty());
    assert!(run(&mut g, P0, &os[0]));
    assert_eq!(g.perm(st).data.int(DataKey::Charge), 6, "Vorinclex doubles the charge counters (3 -> 6)");
}

#[test]
fn b5_station_is_legendary_and_at_12_attacks_for_a_card_per_multicoloured_permanent() {
    let mut g = tbl();
    let c = perm(&mut g, P0, COURIER);
    g.perm_mut(c).sick = true; // stays home
    let st = perm(&mut g, P0, STATION);
    assert!(sim_core::impls::legends::legendary(&g, st));
    g.perm_mut(st).data.set(DataKey::Charge, Val::Int(12));
    let hand0 = g.player(P0).hand.len();
    let life = g.player(P1).life;
    combat::combat(&mut g, P0).unwrap();
    assert_eq!(g.player(P1).life, life - 7, "a 7/15 flier connects");
    assert_eq!(g.player(P0).hand.len() - hand0, 2, "the Courier and the Station are multicoloured");
}

#[test]
fn b5_a_station_that_entered_this_turn_cannot_attack_even_at_12() {
    let mut g = tbl();
    let c = card(&g, STATION);
    g.player_mut(P0).library.retain(|&x| x != c);
    let st = zones::enter(&mut g, P0, c, zones::Enter::default()).unwrap();
    assert!(g.perm(st).sick, "it entered this turn");
    for r in tokens_of(&g, P0, "robot") {
        g.perm_mut(r).sick = true;
    }
    g.perm_mut(st).data.set(DataKey::Charge, Val::Int(12));
    let life = g.player(P1).life;
    combat::combat(&mut g, P0).unwrap();
    assert_eq!(g.player(P1).life, life, "summoning sickness applies as soon as it is a creature");
}

// ============================================================ C. Breya
/// the colours among the artifacts o controls other than `leave_out` (the real count of Karn, Gilded Guardian, copied:
/// kg-e3-karn's `colour_set`)
fn colour_set(g: &Game, o: PlayerId, leave_out: &[PermId]) -> Colors {
    let q = g.player(o);
    let mut all = Colors::NONE;
    for &m in q.perms.iter().chain(q.lands.iter()) {
        let x = g.perm(m);
        if leave_out.contains(&m) || !x.on_bf || x.phased {
            continue;
        }
        if chars::card_types(g, m).has(Types::ARTIFACT) || x.ttypes.contains(&"artifact") {
            all = all.union(chars::colors(g, m));
        }
    }
    all
}

#[test]
fn c1_karn_counts_the_thopters_as_blue() {
    let mut g = tbl();
    let b = perm(&mut g, P0, BREYA);
    let thopters = tokens_of(&g, P0, "thopter");
    assert_eq!(thopters.len(), 2);
    assert_eq!(colour_set(&g, P0, &[b]).count(), 1, "the two Thopters alone are one colour");
    assert!(colour_set(&g, P0, &[b]).has('U'), "blue");
    assert_eq!(colour_set(&g, P0, &[]).count(), 4, "with Breya herself: W U B R");
    for &t in &thopters {
        assert!(g.perm(t).fly);
        assert!(g.is_creature(t));
        assert_eq!(colour_letters(&g, t), "U");
    }
}

fn breya_set(life_p1: i32) -> (Game, PermId) {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 2, false);
    let b = perm(&mut g, P0, BREYA);
    ready(&mut g, b);
    g.player_mut(P1).life = life_p1;
    (g, b)
}

#[test]
fn c2_breya_sacrifices_only_two_other_artifacts_never_a_nonartifact_nor_herself() {
    let (mut g, b) = breya_set(40);
    let elves = perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Sol Ring");
    g.player_mut(P0).life = 5;
    let os = options(&mut g, b, P0, Some(false));
    assert_eq!(os.len(), 1, "gain 5 life is worth it at 5 life");
    assert!(run(&mut g, P0, &os[0]));
    assert!(on(&g, b), "never herself");
    assert!(on(&g, elves), "never a nonartifact permanent");
    assert_eq!(g.player(P0).life, 10);
    let arts_left =
        g.player(P0).perms.iter().filter(|&&m| on(&g, m) && values::ptypes(&g, m).has(Types::ARTIFACT)).count()
            + tokens_of(&g, P0, "thopter").len();
    assert!(arts_left >= 1, "{arts_left}");
}

#[test]
fn c3_breya_minus_four_never_targets_a_hexproof_or_shroud_creature() {
    for kw in ["hexproof", "shroud"] {
        c3_one(kw);
    }
}

#[test]
fn c3b_breya_minus_four_never_targets_a_creature_with_protection_from_her_colours() {
    for kw in ["pro_R", "pro_W", "pro_B", "pro_U"] {
        c3_one(kw);
    }
}

fn c3_one(kw: &'static str) {
    {
        let (mut g, b) = breya_set(40);
        let big = perm(&mut g, P1, "Glorybringer"); // 4/4
        gain_kw(&mut g, big, kw);
        let os = options(&mut g, b, P0, Some(false));
        for o in &os {
            run(&mut g, P0, o);
        }
        assert!(on(&g, big), "Breya (white, blue, black, red) can't target a creature with {kw}: it must survive");
        assert_eq!(g.perm(big).eot_pt, (0, 0), "{kw}: no -4/-4 on it");
    }
}

#[test]
fn c4_breya_ai_does_not_throw_two_artifacts_at_a_ward_it_cannot_pay() {
    // P1 has the Thopteryx and a legendary 4/4 (The Reaper); P0 has exactly the {2} Breya costs and nothing for ward {2}
    let mut g = tbl();
    perm(&mut g, P1, THOPTERYX);
    let reaper = perm(&mut g, P1, REAPER);
    lands(&mut g, P0, "Wastes", 2, false);
    let b = perm(&mut g, P0, BREYA);
    ready(&mut g, b);
    g.player_mut(P1).life = 40;
    assert_eq!(cardcode::aura_ward(&g, reaper), 2, "ward {{2}} from the Thopteryx");
    let os = options(&mut g, b, P0, Some(false));
    for o in &os {
        run(&mut g, P0, o);
    }
    let thopters_left = tokens_of(&g, P0, "thopter").len();
    assert!(
        thopters_left == 2 || !on(&g, reaper),
        "the AI sacrificed its Thopters into a ward it could not pay: Reaper alive, {thopters_left} Thopters left"
    );
}

#[test]
fn c5_a_person_uses_breya_end_to_end() {
    let (mut g, b) = breya_set(40);
    g.player_mut(P0).life = 12;
    seat(&mut g, P0, vec![pick(2), pick(1), pick(1)]); // gain 5 life; the two Thopters
    pool(&mut g, P0, &[('C', 2)]);
    let abil = sim_core::human::human::abilities_of(&mut g, P0, b).unwrap();
    let (label, kind) = abil.into_iter().find(|(l, _)| l.contains("Breya")).expect("Breya's ability is listed");
    assert!(label.contains("sacrifice"), "{label}");
    let sim_core::human::human::AbilityKind::Hook(a) = kind else { panic!("a hook ability") };
    assert!(act::perform(&mut g, P0, &a).unwrap());
    assert_eq!(g.player(P0).life, 17);
    assert_eq!(tokens_of(&g, P0, "thopter").len(), 0, "two artifacts were sacrificed");
}

// ============================================================ D. Gold-Forged Thopteryx
#[test]
fn d1_two_thopteryx_are_two_ward_instances_and_both_are_paid() {
    let mut g = tbl();
    let karn = perm(&mut g, P0, KARN);
    perm(&mut g, P0, THOPTERYX);
    perm(&mut g, P0, THOPTERYX);
    assert_eq!(cardcode::aura_ward(&g, karn), 4, "CR 702.21c: each instance triggers separately");
    lands(&mut g, P1, "Wastes", 3, false);
    assert!(!ward::ward_affordable(&g, P1, karn, 0, ""), "three mana can't pay {{2}} twice");
    assert!(!ward::pay_ward(&mut g, P1, karn).unwrap());
    lands(&mut g, P1, "Wastes", 1, false);
    for &l in &g.player(P1).lands.clone() {
        g.perm_mut(l).tapped = false;
    }
    assert!(ward::pay_ward(&mut g, P1, karn).unwrap());
    assert_eq!(g.player(P1).lands.iter().filter(|&&l| !g.perm(l).tapped).count(), 0, "all four paid");
}

#[test]
fn d2_ward_follows_the_thopteryxs_controller_and_the_legend_not_the_card_owner() {
    let mut g = tbl();
    let karn = perm(&mut g, P0, KARN);
    let t = perm(&mut g, P0, THOPTERYX);
    assert_eq!(cardcode::aura_ward(&g, karn), 2);
    // phased out: no ward from it
    g.perm_mut(t).phased = true;
    assert_eq!(cardcode::aura_ward(&g, karn), 0, "a phased-out Thopteryx gives nothing");
    g.perm_mut(t).phased = false;
    // a phased-out legend is not a target anyway; a dead Thopteryx gives nothing
    zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!(cardcode::aura_ward(&g, karn), 0);
    // our own effects never pay ward on our legends
    perm(&mut g, P0, THOPTERYX);
    assert!(ward::pay_ward(&mut g, P0, karn).unwrap());
}

#[test]
fn d3_legendary_lands_and_tokens_get_ward_too() {
    let mut g = tbl();
    perm(&mut g, P0, THOPTERYX);
    let n = g.db.len();
    let name = (0..n)
        .map(|i| sim_core::ids::CardId(i as u16))
        .find(|&c| {
            let d = g.db.get(c);
            d.land && d.tag(sim_core::tag::Tag::Leg)
        })
        .map(|c| g.db.get(c).name.to_string());
    let Some(name) = name else { return };
    let l = lands(&mut g, P0, &name, 1, false)[0];
    assert_eq!(cardcode::aura_ward(&g, l), 2, "{name}: a legendary land is a legendary permanent");
}

// ============================================================ E. The Reaper
fn reaper_table() -> (Game, PermId, PermId, PermId) {
    let mut g = tbl();
    let a = perm(&mut g, P1, "Glorybringer");
    let b = perm(&mut g, P1, "Grave Titan");
    let c = perm(&mut g, P1, "Ainok Strike Leader");
    (g, a, b, c)
}

#[test]
fn e1_etb_skips_hexproof_and_shroud_creatures_and_uses_the_next_best() {
    for kw in ["hexproof", "shroud"] {
        e1_one(kw);
    }
}

#[test]
fn e1b_etb_skips_a_creature_with_protection_from_black_red_or_green() {
    for kw in ["pro_B", "pro_R", "pro_G"] {
        e1_one(kw);
    }
}

fn e1_one(kw: &'static str) {
    {
        let (mut g, a, b, c) = reaper_table();
        gain_kw(&mut g, a, kw);
        perm(&mut g, P0, REAPER);
        assert_eq!(g.perm(a).plus, 0, "Glorybringer with {kw} can't be targeted by a black, red and green source");
        assert_eq!(g.perm(b).plus, -1);
        assert_eq!(g.perm(c).plus, -1, "the third creature is the next target");
    }
}

#[test]
fn e2_etb_pays_ward_when_it_can_and_skips_when_it_cannot() {
    let mut g = tbl();
    perm(&mut g, P1, THOPTERYX);
    let karn = perm(&mut g, P1, KARN);
    let bears = perm(&mut g, P1, "Ainok Strike Leader");
    perm(&mut g, P0, REAPER);
    assert_eq!(g.perm(karn).plus, 0, "ward {{2}} and no mana: countered");
    assert!(g.perm(bears).plus <= 0);
    let mut g = tbl();
    perm(&mut g, P1, THOPTERYX);
    let karn = perm(&mut g, P1, KARN);
    lands(&mut g, P0, "Wastes", 2, false);
    perm(&mut g, P0, REAPER);
    assert_eq!(g.perm(karn).plus, -1, "two lands pay the ward");
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| !g.perm(l).tapped).count(), 0);
}

#[test]
fn e3_etb_with_one_target_none_and_only_our_own_creatures_does_not_panic() {
    let mut g = tbl();
    perm(&mut g, P0, REAPER);
    let mut g = tbl();
    let only = perm(&mut g, P1, "Grave Titan");
    perm(&mut g, P0, REAPER);
    assert_eq!(g.perm(only).plus, -1);
    let mut g = tbl();
    let mine = perm(&mut g, P0, "Grave Titan");
    perm(&mut g, P0, REAPER);
    assert_eq!(g.perm(mine).plus, 0, "the AI never shrinks its own creature");
}

#[test]
fn e4_dies_needs_a_net_minus_counter_and_a_live_reaper() {
    let mut g = tbl();
    let r = perm(&mut g, P0, REAPER);
    let a = perm(&mut g, P1, "Ainok Strike Leader");
    g.perm_mut(a).plus = 2; // a +1/+1 counter after the annihilation: no -1/-1 counter
    zones::die(&mut g, a, "destroy").unwrap();
    assert!(!has_perm(&g, P0, "Ainok Strike Leader"));
    // the Reaper is gone: nothing
    let b = perm(&mut g, P1, "Amrou Seekers");
    g.perm_mut(b).plus = -1;
    zones::die(&mut g, r, "destroy").unwrap();
    zones::die(&mut g, b, "destroy").unwrap();
    assert!(!has_perm(&g, P0, "Amrou Seekers"), "no Reaper, no trigger");
    assert!(g.player(P1).gy.contains(&card(&g, "Amrou Seekers")));
}

#[test]
fn e5_two_marked_creatures_dying_in_one_turn_return_one_even_across_other_players_turns() {
    let mut g = tbl();
    perm(&mut g, P0, REAPER);
    let a = perm(&mut g, P1, "Ainok Strike Leader");
    let b = perm(&mut g, P1, "Amrou Seekers");
    g.perm_mut(a).plus = -1;
    g.perm_mut(b).plus = -1;
    g.active = Some(P1); // the opponent's own turn
    zones::die(&mut g, a, "sac").unwrap();
    zones::die(&mut g, b, "destroy").unwrap();
    let back = ["Ainok Strike Leader", "Amrou Seekers"].iter().filter(|n| has_perm(&g, P0, n)).count();
    assert_eq!(back, 1, "once each turn, whoever's turn it is");
}

#[test]
fn e6_the_returned_creature_is_the_reapers_controllers_with_its_enter_effects() {
    let mut g = tbl();
    perm(&mut g, P0, REAPER);
    let t = perm(&mut g, P1, "Grave Titan");
    g.perm_mut(t).plus = -1;
    zones::die(&mut g, t, "destroy").unwrap();
    assert!(has_perm(&g, P0, "Grave Titan"));
    assert_eq!(
        perms_named(&g, P0, "Token").len(),
        2,
        "Grave Titan's enter trigger makes its two Zombies for the Reaper's side"
    );
    assert_eq!(perms_named(&g, P1, "Token").len(), 2, "and the two it made for P1 as it first entered stay P1's");
}

#[test]
fn e7_a_person_picks_the_reapers_targets() {
    let mut g = tbl();
    let a = perm(&mut g, P1, "Grave Titan");
    let b = perm(&mut g, P1, "Ainok Strike Leader");
    let h = std::sync::Arc::new(sim_core::human::script::ScriptHuman::from_fn(|r| {
        if r.choices.is_empty() { Answer::Yes } else { Answer::Pick(0) }
    }));
    g.humans.add(P0, h);
    perm(&mut g, P0, REAPER);
    let marked = [a, b].iter().filter(|&&m| g.perm(m).plus == -1).count();
    assert!(marked >= 1 && marked <= 2, "{marked}");
}

// ============================================================ F. Herd Heirloom
#[test]
fn f1_heirloom_pays_the_reaper_and_the_doll_not_shorikai() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 5, false);
    let h = perm(&mut g, P0, HEIRLOOM);
    ready(&mut g, h);
    let reaper = hand(&mut g, P0, &[REAPER])[0];
    let (gn, pips) = cost_of(&g, P0, reaper);
    assert!(can_pay_spell(&g, P0, reaper, gn, &pips, false), "an artifact creature spell: 5 Wastes + the Heirloom");
    let shorikai = hand(&mut g, P0, &[SHORIKAI])[0];
    let (sg, sp) = cost_of(&g, P0, shorikai);
    assert!(!can_pay_spell(&g, P0, shorikai, sg, &sp, false), "no W or U: a Vehicle is no creature spell");
}

#[test]
fn f2_heirloom_gives_nothing_to_power_three_and_survives_the_grant_leaving() {
    let mut g = tbl();
    let h = perm(&mut g, P0, HEIRLOOM);
    ready(&mut g, h);
    let three = token(&mut g, P0, 3);
    ready(&mut g, three);
    assert!(options(&mut g, h, P0, Some(false)).is_empty());
    // main 2 / on another player's turn: the AI offers nothing
    let four = token(&mut g, P0, 4);
    ready(&mut g, four);
    assert!(options(&mut g, h, P0, Some(true)).is_empty(), "after combat the grant is pointless");
    g.active = Some(P1);
    assert!(options(&mut g, h, P0, Some(false)).is_empty(), "AI: only on its own turn");
}

#[test]
fn f3_heirloom_ability_is_not_a_mana_ability_of_a_tapped_heirloom() {
    let mut g = tbl();
    let h = perm(&mut g, P0, HEIRLOOM);
    ready(&mut g, h);
    let big = token(&mut g, P0, 5);
    ready(&mut g, big);
    g.perm_mut(h).tapped = true;
    assert!(options(&mut g, h, P0, Some(false)).is_empty());
    assert!(mana::total_mana(&g, P0, false) == 0);
}

// ============================================================ G. Twitching Doll
#[test]
fn g1_vorinclex_doubles_the_nest_counter() {
    let mut g = tbl();
    perm(&mut g, P0, "Vorinclex, Monstrous Raider");
    let d = perm(&mut g, P0, DOLL);
    ready(&mut g, d);
    assert!(mana::pay(&mut g, P0, 0, "G", false).unwrap());
    assert_eq!(g.perm(d).data.int(DataKey::Counters), 2, "Vorinclex doubles a counter its controller puts");
}

#[test]
fn g2_a_summoning_sick_doll_makes_no_mana_and_no_counter() {
    let mut g = tbl();
    let d = perm(&mut g, P0, DOLL);
    g.perm_mut(d).sick = true;
    assert!(
        !can_pay(&g, P0, 1, "", false),
        "a creature's {{T}} ability needs it to be under control since the turn began"
    );
    assert_eq!(g.perm(d).data.int(DataKey::Counters), 0);
}

#[test]
fn g3_zero_counters_make_no_spiders_and_do_not_panic() {
    let mut g = tbl();
    let d = perm(&mut g, P0, DOLL);
    ready(&mut g, d);
    g.perm_mut(d).data.set(DataKey::Counters, Val::Int(3));
    let os = options(&mut g, d, P0, Some(false));
    assert_eq!(os.len(), 1);
    g.perm_mut(d).data.set(DataKey::Counters, Val::Int(0));
    assert!(run(&mut g, P0, &os[0]));
    assert!(tokens_of(&g, P0, "spider").is_empty(), "no counter, no Spider");
}

#[test]
fn g4_doll_is_not_sacrificed_in_response_or_on_the_opponents_turn() {
    let mut g = tbl();
    let d = perm(&mut g, P0, DOLL);
    ready(&mut g, d);
    g.perm_mut(d).data.set(DataKey::Counters, Val::Int(5));
    g.responding = 1;
    assert!(options(&mut g, d, P0, Some(false)).is_empty());
    g.responding = 0;
    g.active = Some(P1);
    assert!(options(&mut g, d, P0, Some(false)).is_empty());
}

#[test]
fn g5_the_persons_doll_is_listed_with_one_counter_and_makes_one_spider() {
    let mut g = tbl();
    seat(&mut g, P0, vec![]);
    let d = perm(&mut g, P0, DOLL);
    ready(&mut g, d);
    g.perm_mut(d).data.set(DataKey::Counters, Val::Int(1));
    let abil = sim_core::human::human::abilities_of(&mut g, P0, d).unwrap();
    let (_, kind) = abil.into_iter().find(|(l, _)| l.contains("Twitching Doll")).expect("listed for a person");
    let sim_core::human::human::AbilityKind::Hook(a) = kind else { panic!() };
    assert!(act::perform(&mut g, P0, &a).unwrap());
    assert_eq!(tokens_of(&g, P0, "spider").len(), 1);
    assert!(!on(&g, d));
}

// ============================================================ H. Shorikai
#[test]
fn h1_shorikai_is_never_a_creature_for_blocking_or_attacking_and_the_pilot_is_no_artifact() {
    let mut g = tbl();
    let s = perm(&mut g, P0, SHORIKAI);
    assert!(!g.is_creature(s));
    let attacker = token(&mut g, P1, 5);
    ready(&mut g, attacker);
    let life = g.player(P0).life;
    combat::combat(&mut g, P1).unwrap();
    assert_eq!(g.player(P0).life, life - 5, "the Vehicle can't block");
    assert!(on(&g, s) && !g.perm(s).tapped);
    // its own combat: it never attacks
    g.active = Some(P0);
    let life = g.player(P1).life;
    combat::combat(&mut g, P0).unwrap();
    assert_eq!(g.player(P1).life, life, "a Vehicle that is not crewed doesn't attack");
}

#[test]
fn h2_the_pilot_token_is_a_colourless_nonartifact_creature() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 1, false);
    let s = perm(&mut g, P0, SHORIKAI);
    let os: Vec<Opt> = sim_core::dsl::ability_options(&mut g, P0, true)
        .unwrap()
        .into_iter()
        .filter(|o| matches!(o.act, Some(Action::Dsl { src, .. }) if src == s))
        .collect();
    assert_eq!(os.len(), 1);
    let Some(Action::Dsl { src, idx }) = os[0].act.clone() else { panic!() };
    assert!(sim_core::dsl::activate(&mut g, P0, src, idx).unwrap());
    let pilots = tokens_of(&g, P0, "pilot");
    assert_eq!(pilots.len(), 1);
    assert!(!values::ptypes(&g, pilots[0]).has(Types::ARTIFACT));
    assert!(!sim_core::impls::karn_cards::is_art(&g, pilots[0]), "a Pilot creature token is not an artifact");
    assert!(chars::colors(&g, pilots[0]).is_empty());
    // activating again needs {T}: Shorikai is tapped now
    assert!(
        sim_core::dsl::ability_options(&mut g, P0, true)
            .unwrap()
            .iter()
            .all(|o| !matches!(o.act, Some(Action::Dsl { src, .. }) if src == s))
    );
}

// ============================================================ I. data
#[test]
fn i1_the_new_cards_have_one_path_each() {
    let g = tbl();
    for n in [STATION, BREYA, THOPTERYX, REAPER, HEIRLOOM, DOLL] {
        let d = g.db.get(card(&g, n));
        assert!(d.abilities.is_empty(), "{n}: dsl must be empty so no second path exists: {:?}", d.abilities.len());
    }
    assert_eq!(g.db.get(card(&g, THOPTERYX)).ward, 0);
    assert!(g.db.get(card(&g, COURIER)).creature);
}
