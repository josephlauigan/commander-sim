//! The Fire Crystal ({2}{R}{R} legendary artifact), in the Karn, Gilded Guardian deck. One test for each part of each Oracle
//! line, on a hand-built position; the expectation is the Oracle text, not the code:
//!   - "Red spells you cast cost {1} less to cast." (compiled `static:cost`)
//!   - "Creatures you control have haste." (compiled `static:keyword`)
//!   - "{4}{R}{R}, {T}: Create a token that's a copy of target creature you control. Sacrifice it at the beginning of the
//!     next end step." (a person's ability, `impls::karn_gilded_cards::fire_crystal_go`; the token carries
//!     `DataKey::CopySac` and `turn::end_step` sacrifices it)
//!
//! The AI has no use of the copy ability (no card-code hook): `the_ai_has_no_hook_for_the_crystal` pins that.

mod play_common;
use play_common::*;
use sim_core::engine::mana::cost_of;
use sim_core::engine::values::{epow, etgh};
use sim_core::engine::zones::{self, Enter, Tokens};
use sim_core::engine::{combat, turn};
use sim_core::human::abilities::PermAbility;
use sim_core::human::script::ScriptHuman;
use sim_core::state::{DataKey, Step};
use std::sync::Arc;

const CRYSTAL: &str = "The Fire Crystal";
const KARN: &str = "Karn, Gilded Guardian";
const BAT: &str = "Vampire Nighthawk"; // a nonlegendary 2/3 creature with a card record and no enters trigger

/// P0 and P1: P0's turn, main phase one, an empty hand
fn tbl() -> Game {
    let mut g = table(&["sheoldred", "krenko-mono-red-goblins"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g.player_mut(P0).hand.clear();
    g
}

fn crystal(g: &mut Game, p: PlayerId) -> PermId {
    perm(g, p, CRYSTAL)
}

/// the cost of a card in P0's hand, as the engine would charge it now: (generic, coloured pips)
fn cost(g: &mut Game, name: &str) -> (u32, String) {
    let c = card(g, name);
    cost_of(g, P0, c)
}

/// a creature of p's that is summoning sick (it came this turn)
fn sick_creature(g: &mut Game, p: PlayerId, name: &str) -> PermId {
    let c = take(g, p, name);
    zones::enter(g, p, c, Enter { sick: true, ..Default::default() }).unwrap()
}

fn can_attack(g: &Game, p: PlayerId, m: PermId) -> bool {
    sim_core::human::combat::can_attack(g, p, m)
}

fn activate(g: &mut Game, p: PlayerId, src: PermId) -> Option<String> {
    let a: Vec<PermAbility> = sim_core::human::cards::abilities(g, p, src).expect("the Crystal has a person's ability");
    assert_eq!(a.len(), 1);
    (a[0].act)(g, p, src).unwrap()
}

/// a person at P0's seat: the Crystal's "copy which creature?" is answered with the first choice that contains `target`
/// (the first listed when the word is empty), the legend rule's "which do you keep?" with the choice at index `keep`
/// (the permanents are listed in the order they entered), a target for an enters trigger with an opponent's permanent, and
/// every other request (a window to respond) with a pass
fn person(g: &mut Game, target: &'static str, keep: usize) -> Arc<ScriptHuman> {
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if r.prompt.contains("copy which creature") {
            Answer::Pick(r.choices.iter().position(|c| !target.is_empty() && c.contains(target)).unwrap_or(0))
        } else if r.prompt.contains("legend rule") {
            Answer::Pick(keep)
        } else if r.prompt.contains("choose a target") {
            // an enters trigger of the copy: the first opponent's permanent listed (the opponent is Krenko)
            Answer::Pick(r.choices.iter().position(|c| c.contains("Krenko")).unwrap_or(0))
        } else {
            Answer::Action(json!({"do": "pass"}))
        }
    }));
    g.humans.add(P0, h.clone());
    h
}

/// the Crystal, a person at P0's seat who copies the first creature listed, and `r` red mana and `c` more in the pool
fn ready(g: &mut Game, r: u32, c: u32) -> PermId {
    person(g, "", 0);
    let m = crystal(g, P0);
    pool(g, P0, &[('R', r), ('C', c)]);
    m
}

fn tokens_named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    perms_named(g, p, name).into_iter().filter(|&m| g.perm(m).token).collect()
}

// ======================================================== "Red spells you cast cost {1} less to cast."
#[test]
fn red_spells_cost_one_less_generic_mana() {
    let mut g = tbl();
    assert_eq!(cost(&mut g, "Terror of the Peaks"), (3, "RR".into()), "{{3}}{{R}}{{R}} before");
    assert_eq!(cost(&mut g, "Inferno Titan"), (4, "RR".into()));
    crystal(&mut g, P0);
    assert_eq!(cost(&mut g, "Terror of the Peaks"), (2, "RR".into()), "{{2}}{{R}}{{R}} with the Crystal");
    assert_eq!(cost(&mut g, "Inferno Titan"), (3, "RR".into()), "{{3}}{{R}}{{R}}");
}

#[test]
fn the_reduction_takes_only_generic_mana_never_a_coloured_pip() {
    let mut g = tbl();
    crystal(&mut g, P0);
    assert_eq!(cost(&mut g, "Lightning Bolt"), (0, "R".into()), "{{R}} has no generic part: unchanged");
    assert_eq!(cost(&mut g, "Goblin Chainwhirler"), (0, "RRR".into()), "{{R}}{{R}}{{R}}: unchanged");
}

#[test]
fn spells_of_other_colours_and_colourless_spells_cost_the_same() {
    let names = ["Sol Ring", "Counterspell", "Demonic Tutor", "Thought Vessel", "Swords to Plowshares", "Cultivate"];
    let mut g = tbl();
    let before: Vec<_> = names.iter().map(|n| cost(&mut g, n)).collect();
    crystal(&mut g, P0);
    for (n, b) in names.iter().zip(before) {
        assert_eq!(cost(&mut g, n), b, "{n} is not red");
    }
}

#[test]
fn a_multicolour_red_spell_is_a_red_spell() {
    let mut g = tbl();
    assert_eq!(cost(&mut g, "Kolaghan's Command"), (1, "BR".into()));
    crystal(&mut g, P0);
    assert_eq!(cost(&mut g, "Kolaghan's Command"), (0, "BR".into()), "black and red: red, one generic mana less");
    assert_eq!(cost(&mut g, "Lightning Helix"), (0, "RW".into()), "no generic part: unchanged");
}

#[test]
fn an_opponents_crystal_does_not_reduce_your_spells() {
    let mut g = tbl();
    crystal(&mut g, P1);
    assert_eq!(cost(&mut g, "Terror of the Peaks"), (3, "RR".into()), "'Red spells you cast': its controller's");
}

#[test]
fn two_crystals_reduce_by_two() {
    // a second copy of the card exists only in a test; each copy's text applies
    let mut g = tbl();
    crystal(&mut g, P0);
    crystal(&mut g, P0);
    assert_eq!(cost(&mut g, "Terror of the Peaks"), (1, "RR".into()));
}

#[test]
fn karn_is_red_so_the_reduction_takes_a_generic_two_paid_for_a_missing_colour() {
    // Karn costs five with all five colours (no generic part: nothing to reduce); a missing colour is paid with {2}
    let mut g = tbl();
    for l in ["Plains", "Island", "Swamp", "Forest"] {
        lands(&mut g, P0, l, 1, false);
    }
    let (gn0, _) = cost(&mut g, KARN);
    crystal(&mut g, P0);
    let (gn1, _) = cost(&mut g, KARN);
    assert_eq!(gn0, 2, "red missing: {{2}} paid for the {{2/R}}");
    assert_eq!(gn1, 1, "the Crystal takes one off that generic part");
    lands(&mut g, P0, "Mountain", 1, false);
    let (gn2, pips) = cost(&mut g, KARN);
    assert_eq!((gn2, pips.len()), (0, 5), "all five colours: five coloured mana, nothing generic");
}

// ======================================================== "Creatures you control have haste."
#[test]
fn a_creature_that_entered_before_the_crystal_has_haste() {
    let mut g = tbl();
    let m = sick_creature(&mut g, P0, BAT);
    assert!(!can_attack(&g, P0, m), "summoning sick without it");
    crystal(&mut g, P0);
    assert!(combat::has_haste(&g, m));
    assert!(can_attack(&g, P0, m), "the Crystal gives it haste");
}

#[test]
fn a_creature_that_enters_after_the_crystal_has_haste() {
    let mut g = tbl();
    crystal(&mut g, P0);
    let m = sick_creature(&mut g, P0, BAT);
    assert!(g.perm(m).sick, "it did just come under your control");
    assert!(can_attack(&g, P0, m));
}

#[test]
fn a_token_creature_has_haste() {
    let mut g = tbl();
    crystal(&mut g, P0);
    let t = zones::make_tokens(&mut g, P0, Tokens { sick: true, ..Tokens::new(1, 2) }).unwrap()[0];
    assert!(g.perm(t).token && g.is_creature(t));
    assert!(can_attack(&g, P0, t));
}

#[test]
fn an_opponents_creature_does_not_get_haste_from_your_crystal() {
    let mut g = tbl();
    crystal(&mut g, P0);
    let theirs = sick_creature(&mut g, P1, BAT);
    assert!(!combat::has_haste(&g, theirs), "'creatures you control'");
    assert!(!can_attack(&g, P1, theirs));
}

#[test]
fn haste_ends_when_the_crystal_leaves() {
    let mut g = tbl();
    let c = crystal(&mut g, P0);
    let m = sick_creature(&mut g, P0, BAT);
    assert!(can_attack(&g, P0, m));
    zones::die(&mut g, c, "destroy").unwrap();
    assert!(!combat::has_haste(&g, m));
    assert!(!can_attack(&g, P0, m));
}

// ======================================================== "{4}{R}{R}, {T}: Create a token that's a copy ..."
#[test]
fn the_ability_needs_four_generic_and_two_red() {
    let mut g = tbl();
    let c = ready(&mut g, 1, 5); // one red: not enough
    sick_creature(&mut g, P0, BAT);
    let why = activate(&mut g, P0, c).expect("it can't be paid");
    assert!(why.contains("Can't activate"), "{why}");
    assert!(!g.perm(c).tapped, "nothing was paid");
    assert!(tokens_named(&g, P0, BAT).is_empty());
    let mut g = tbl();
    let c = ready(&mut g, 2, 3); // {R}{R} and only three more: five mana is not six
    sick_creature(&mut g, P0, BAT);
    assert!(activate(&mut g, P0, c).is_some());
    assert!(!g.perm(c).tapped);
    assert!(tokens_named(&g, P0, BAT).is_empty());
}

#[test]
fn the_ability_taps_the_crystal_and_spends_exactly_six_mana() {
    let mut g = tbl();
    let c = ready(&mut g, 3, 5); // eight mana
    sick_creature(&mut g, P0, BAT);
    assert!(activate(&mut g, P0, c).is_none());
    assert!(g.perm(c).tapped, "{{T}} is a cost");
    assert_eq!(pool_total(&g, P0), 2, "{{4}}{{R}}{{R}}: six of the eight mana");
    assert_eq!(tokens_named(&g, P0, BAT).len(), 1);
}

#[test]
fn a_tapped_crystal_cannot_activate() {
    let mut g = tbl();
    let c = ready(&mut g, 2, 4);
    sick_creature(&mut g, P0, BAT);
    g.perm_mut(c).tapped = true;
    assert!(activate(&mut g, P0, c).is_some());
    assert_eq!(pool_total(&g, P0), 6, "no mana spent");
    assert!(tokens_named(&g, P0, BAT).is_empty());
}

#[test]
fn the_crystal_can_be_used_again_only_after_it_untaps() {
    let mut g = tbl();
    let c = ready(&mut g, 4, 8);
    sick_creature(&mut g, P0, BAT);
    assert!(activate(&mut g, P0, c).is_none());
    assert!(activate(&mut g, P0, c).is_some(), "tapped");
    g.perm_mut(c).tapped = false;
    assert!(activate(&mut g, P0, c).is_none());
    assert_eq!(tokens_named(&g, P0, BAT).len(), 2);
}

#[test]
fn it_needs_a_creature_you_control() {
    let mut g = tbl();
    let c = ready(&mut g, 2, 4);
    perm(&mut g, P1, "Grave Titan"); // not yours
    perm(&mut g, P0, "Sol Ring"); // not a creature
    let why = activate(&mut g, P0, c).expect("no target");
    assert!(why.contains("no creature"), "{why}");
    assert!(!g.perm(c).tapped);
    assert_eq!(pool_total(&g, P0), 6);
}

#[test]
fn the_target_list_is_your_creatures_only_and_cancelling_pays_nothing() {
    let mut g = tbl();
    let c = crystal(&mut g, P0);
    pool(&mut g, P0, &[('R', 2), ('C', 4)]);
    sick_creature(&mut g, P0, BAT);
    perm(&mut g, P1, "Grave Titan");
    perm(&mut g, P0, "Sol Ring");
    let h = Arc::new(ScriptHuman::new(vec![Answer::Decline]));
    g.humans.add(P0, h.clone());
    assert!(activate(&mut g, P0, c).is_none(), "cancelling the target is not an error");
    let asked = h.asked();
    assert_eq!(asked.len(), 1);
    assert_eq!(asked[0].choices.len(), 2, "my one creature and 'cancel': {:?}", asked[0].choices);
    assert!(asked[0].choices[0].contains("Nighthawk"), "{:?}", asked[0].choices);
    assert!(!g.perm(c).tapped, "cancelled before the costs: targets first, then the costs");
    assert_eq!(pool_total(&g, P0), 6);
}

#[test]
fn a_creature_with_protection_from_red_is_not_a_target() {
    // the Crystal is a red source: its ability can't target a creature with protection from red
    let mut g = tbl();
    let c = ready(&mut g, 2, 4);
    perm(&mut g, P0, "Kor Firewalker"); // protection from red
    let why = activate(&mut g, P0, c).expect("no legal target");
    assert!(why.contains("no creature"), "{why}");
    assert!(!g.perm(c).tapped);
    let mut g = tbl();
    let c = crystal(&mut g, P0);
    pool(&mut g, P0, &[('R', 2), ('C', 4)]);
    perm(&mut g, P0, "Kor Firewalker");
    sick_creature(&mut g, P0, BAT);
    let h = person(&mut g, "Nighthawk", 0);
    assert!(activate(&mut g, P0, c).is_none());
    let asked = h.asked();
    assert!(asked[0].choices.iter().all(|x| !x.contains("Firewalker")), "{:?}", asked[0].choices);
    assert_eq!(tokens_named(&g, P0, BAT).len(), 1);
}

#[test]
fn the_token_is_a_copy_with_haste_from_the_crystal_itself() {
    let mut g = tbl();
    let c = ready(&mut g, 2, 4);
    let orig = sick_creature(&mut g, P0, BAT);
    assert!(activate(&mut g, P0, c).is_none());
    let toks = tokens_named(&g, P0, BAT);
    assert_eq!(toks.len(), 1, "a token that's a copy of the Nighthawk");
    let t = toks[0];
    assert_ne!(t, orig);
    assert!(g.is_creature(t));
    assert_eq!((epow(&g, t), etgh(&g, t)), (2, 3));
    assert!(g.perm(t).sick, "it did just enter");
    assert!(can_attack(&g, P0, t), "creatures you control have haste: the copy too");
    assert!(g.perm(orig).on_bf, "the original stays");
}

#[test]
fn a_nonlegendary_copy_and_its_original_both_stay_until_the_end_step() {
    let mut g = tbl();
    let c = ready(&mut g, 2, 4);
    let orig = sick_creature(&mut g, P0, BAT);
    assert!(activate(&mut g, P0, c).is_none());
    assert_eq!(perms_named(&g, P0, BAT).len(), 2, "no legend rule for a nonlegendary creature");
    assert!(g.perm(orig).on_bf);
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(perms_named(&g, P0, BAT), vec![orig], "the token is sacrificed at the beginning of the end step");
    assert!(tokens_named(&g, P0, BAT).is_empty());
}

#[test]
fn the_copy_made_on_an_opponents_turn_is_sacrificed_at_that_end_step() {
    // the next end step is the opponent's, whoever's turn it is: made in their main phase, gone at their end step
    let mut g = tbl();
    g.active = Some(P1);
    let c = ready(&mut g, 2, 4);
    sick_creature(&mut g, P0, BAT);
    assert!(activate(&mut g, P0, c).is_none());
    assert_eq!(tokens_named(&g, P0, BAT).len(), 1);
    turn::end_step(&mut g, P1).unwrap();
    assert!(tokens_named(&g, P0, BAT).is_empty(), "sacrificed at the opponent's end step");
    assert_eq!(perms_named(&g, P0, BAT).len(), 1);
}

#[test]
fn a_copy_made_during_an_end_step_waits_for_the_next_end_step() {
    // "the beginning of the next end step": an activation during an end step is too late for that step
    let mut g = tbl();
    g.step = Step::End;
    let c = ready(&mut g, 2, 4);
    sick_creature(&mut g, P0, BAT);
    let due = turn::copy_sacs_due(&g); // the end step begins: nothing is marked yet
    assert!(due.is_empty());
    assert!(activate(&mut g, P0, c).is_none()); // during the step
    let t = tokens_named(&g, P0, BAT)[0];
    assert!(g.perm(t).data.truthy(DataKey::CopySac));
    turn::sacrifice_copies(&mut g, due).unwrap(); // this end step's own sacrifices
    assert!(g.perm(t).on_bf, "not sacrificed at this end step");
    g.active = Some(P1);
    turn::end_step(&mut g, P1).unwrap(); // the next end step to begin is the following turn's
    assert!(!g.perm(t).on_bf, "sacrificed at the next end step");
}

#[test]
fn a_copy_that_is_already_gone_is_not_sacrificed_twice_and_the_original_is_never_marked() {
    let mut g = tbl();
    let c = ready(&mut g, 2, 4);
    let orig = sick_creature(&mut g, P0, BAT);
    assert!(activate(&mut g, P0, c).is_none());
    assert!(!g.perm(orig).data.truthy(DataKey::CopySac));
    turn::end_step(&mut g, P0).unwrap();
    turn::end_step(&mut g, P1).unwrap();
    assert!(g.perm(orig).on_bf, "a later end step does nothing to the original");
}

// ---- Karn: the enters trigger and the legend rule
fn karn_on_the_table(g: &mut Game) -> PermId {
    set_commander(g, P0, KARN);
    let m = perm(g, P0, KARN);
    let pl = g.player_mut(P0);
    pl.cmd_in_zone = false;
    g.perm_mut(m).is_cmd = true;
    m
}

#[test]
fn a_copy_of_karn_draws_for_every_colour_among_the_other_artifacts_then_the_legend_rule_keeps_one() {
    // the original Karn is another artifact of all five colours, so the copy's enters trigger draws five whatever else is out
    let mut g = tbl();
    let karn = karn_on_the_table(&mut g);
    let c = ready(&mut g, 2, 4);
    let hand = g.player(P0).hand.len();
    assert!(activate(&mut g, P0, c).is_none());
    assert_eq!(g.player(P0).hand.len(), hand + 5, "the copy's enters trigger: a card for each of the five colours");
    assert_eq!(perms_named(&g, P0, KARN).len(), 1, "the legend rule: the controller keeps one Karn");
    assert!(g.perm(karn).on_bf, "the first one listed, the real Karn, is kept");
}

#[test]
fn the_person_chooses_which_karn_the_legend_rule_keeps_and_the_commander_can_stay() {
    let mut g = tbl();
    let karn = karn_on_the_table(&mut g);
    let c = crystal(&mut g, P0);
    pool(&mut g, P0, &[('R', 2), ('C', 4)]);
    let h = person(&mut g, "", 0); // keep the first Karn listed: the real one
    assert!(activate(&mut g, P0, c).is_none());
    let legend = h.asked().into_iter().find(|r| r.prompt.contains("legend rule")).expect("asked which Karn to keep");
    assert_eq!(legend.choices.len(), 2, "{:?}", legend.choices);
    assert_eq!(perms_named(&g, P0, KARN).len(), 1);
    assert!(g.perm(karn).on_bf && g.perm(karn).is_cmd, "the real Karn, the commander, stays");
    turn::end_step(&mut g, P0).unwrap();
    assert!(g.perm(karn).on_bf, "nothing left to sacrifice");
}

#[test]
fn keeping_the_token_karn_is_legal_and_it_is_sacrificed_at_the_end_step() {
    // 704.5j: the controller chooses; keeping the token puts the real Karn into the graveyard (the commander
    // replacement then applies); the token still goes at the end step
    let mut g = tbl();
    let karn = karn_on_the_table(&mut g);
    let c = crystal(&mut g, P0);
    pool(&mut g, P0, &[('R', 2), ('C', 4)]);
    let h = person(&mut g, "", 1); // the second one listed: the token, which entered last
    assert!(activate(&mut g, P0, c).is_none());
    let legend = h.asked().into_iter().find(|r| r.prompt.contains("legend rule")).expect("asked which Karn to keep");
    assert_eq!(legend.choices.len(), 2, "{:?}", legend.choices);
    let now = perms_named(&g, P0, KARN);
    assert_eq!(now.len(), 1);
    assert!(g.perm(now[0]).token, "the token was kept");
    assert!(!g.perm(karn).on_bf, "the real Karn left");
    assert!(g.perm(now[0]).data.truthy(DataKey::CopySac));
    turn::end_step(&mut g, P0).unwrap();
    assert!(perms_named(&g, P0, KARN).is_empty(), "the token was sacrificed at the end step");
}

#[test]
fn a_creature_with_an_enters_trigger_repeats_it_through_the_copy() {
    // Skyclave Apparition: "When this creature enters, exile up to one target nonland, nontoken permanent you don't
    // control with mana value 4 or less": the copy's trigger is the card's own, so a second permanent goes
    let mut g = tbl();
    let a = perm(&mut g, P1, "Vampire Nighthawk");
    let b = perm(&mut g, P1, "Vampire Nighthawk");
    let sky = perm(&mut g, P0, "Skyclave Apparition");
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert_eq!([a, b].iter().filter(|&&m| g.perm(m).on_bf).count(), 1, "the Apparition exiled one of them");
    let h = person(&mut g, "", 0);
    let c = crystal(&mut g, P0);
    pool(&mut g, P0, &[('R', 2), ('C', 4)]);
    assert!(activate(&mut g, P0, c).is_none());
    assert_eq!(tokens_named(&g, P0, "Skyclave Apparition").len(), 1);
    let asked: Vec<String> = h.asked().iter().map(|r| format!("{} {:?}", r.prompt, r.choices)).collect();
    assert!(!g.perm(a).on_bf && !g.perm(b).on_bf, "the copy's enters trigger exiled the other: {asked:?}");
    assert!(g.perm(sky).on_bf);
}

// ======================================================== the AI
#[test]
fn the_ai_has_no_hook_for_the_crystal() {
    // the copy ability is a person's ability: no `options` hook, so the AI never activates it, and the card stays a
    // compiled cost change in the AI's cast ranking (`ai::pool`) rather than a hooked card
    let g = tbl();
    let c = card(&g, CRYSTAL);
    assert!(g.registry.get(c).and_then(|i| i.options).is_none(), "no AI option for the copy ability");
    assert!(g.registry.get(c).is_none_or(|i| !i.live()), "no card code runs for it: the compiled statics do the rest");
    let d = g.db.get(c);
    assert!(d.abilities.iter().any(|a| a.static_kind() == Some("cost")));
    assert!(d.abilities.iter().any(|a| a.static_kind() == Some("keyword")));
}
