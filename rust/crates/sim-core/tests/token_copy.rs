//! `zones::token_copy_of`: "create a token that's a copy of target creature", for a card-backed permanent and for a
//! token with no card record alike (CR 707.2-707.3, 707.9b, 708.2). The expectation is the rules text:
//!   - a copy has the copiable values (a Thopter token's name, colour, types, flying; a Spider's reach), not the state of
//!     the source (counters, tapped, attachments, until-end-of-turn effects, marks);
//!   - copy exceptions the source was made with carry over (a Helm of the Host token is not legendary, a Cursed Mirror copy
//!     has haste), except the Mirror's own end-of-turn revert;
//!   - a face-down creature copies as a 2/2 with no name, colour or abilities;
//!   - a token doubler doubles the copy, `marks` land on every token, TOKEN_CAP ends it;
//!   - at top level the state-based actions come before the triggers (the legend rule first), nested neither.

mod play_common;
use play_common::*;
use sim_core::cards::Colors;
use sim_core::engine::life::check_state;
use sim_core::engine::repl;
use sim_core::engine::values::{colors_of, epow, etgh};
use sim_core::engine::zones::{self, TOKEN_CAP, Tokens};
use sim_core::state::{CopyExc, DataKey, Step, Val};

const KARN: &str = "Karn, Gilded Guardian";
const BAT: &str = "Vampire Nighthawk";
const MIKAEUS: &str = "Mikaeus, the Unhallowed"; // a legendary creature with no enters trigger

/// P0's turn, main phase one, an empty hand
fn tbl() -> Game {
    let mut g = table(&["sheoldred", "krenko-mono-red-goblins"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g.player_mut(P0).hand.clear();
    g
}

fn mark() -> [(DataKey, Val); 1] {
    [(DataKey::CopySac, Val::Player(P0))]
}

/// a 1/1 blue flying Thopter artifact creature token, as Breya makes them
fn thopter(g: &mut Game) -> PermId {
    let spec = Tokens {
        fly: true,
        sick: false,
        color: Some(Colors::from_letters("U")),
        types: vec!["thopter", "artifact"],
        ..Tokens::new(1, 1)
    };
    zones::make_tokens(g, P0, spec).unwrap()[0]
}

fn tokens_of(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).token).count()
}

#[test]
fn a_copy_of_a_thopter_is_a_blue_flying_artifact_thopter_token_marked_for_sacrifice() {
    let mut g = tbl();
    let karn = perm(&mut g, P0, KARN);
    let src = thopter(&mut g);
    let made = zones::token_copy_of(&mut g, P0, src, &mark()).unwrap();
    assert_eq!(made.len(), 1);
    let t = made[0];
    assert_ne!(t, src);
    assert!(g.perm(t).token && g.perm(t).cd.is_none() && g.perm(t).on_bf);
    assert_eq!((epow(&g, t), etgh(&g, t)), (1, 1));
    assert!(g.perm(t).fly);
    assert_eq!(colors_of(&g, t), Colors::from_letters("U"));
    assert_eq!(g.perm(t).ttypes, vec!["thopter", "artifact"]);
    assert_eq!(g.perm(t).name, g.perm(src).name);
    assert!(g.perm(t).sick, "the token enters summoning sick");
    assert_eq!(g.perm(t).data.get(DataKey::CopySac), Some(&Val::Player(P0)));
    // Karn counts the copy's colour: take the source away and the copy is the only other artifact
    zones::die(&mut g, src, "destroy").unwrap();
    assert_eq!(sim_core::impls::karn_gilded::colours_among_other_artifacts(&g, P0, karn), 1, "blue, from the copy");
}

#[test]
fn a_copy_of_a_spider_keeps_reach() {
    let mut g = tbl();
    let mut spec =
        Tokens { sick: false, color: Some(Colors::from_letters("G")), types: vec!["spider"], ..Tokens::new(1, 2) };
    spec.data.set(DataKey::Kws, Val::List(vec![Val::Str("reach")]));
    let src = zones::make_tokens(&mut g, P0, spec).unwrap()[0];
    assert!(sim_core::dsl::has_kw(&g, src, "reach"));
    let t = zones::token_copy_of(&mut g, P0, src, &mark()).unwrap()[0];
    assert!(sim_core::dsl::has_kw(&g, t, "reach"), "the keyword is a copiable value");
    assert_eq!((epow(&g, t), etgh(&g, t)), (2, 2));
    assert_eq!(g.perm(t).ttypes, vec!["spider"]);
}

#[test]
fn counters_tapped_attachments_effects_and_marks_of_the_source_are_not_copied() {
    let mut g = tbl();
    let src = thopter(&mut g);
    let gear = perm(&mut g, P0, "Lightning Greaves");
    {
        let x = g.perm_mut(src);
        x.plus = 2;
        x.tapped = true;
        x.eot_pt = (3, 3);
        x.eot_kw.push("trample");
        x.data.set(DataKey::CopySac, Val::Player(P1)); // the source is itself marked, by someone else
        x.data.set(DataKey::Frozen, Val::Bool(true));
    }
    g.perm_mut(gear).attached = Some(src);
    let t = zones::token_copy_of(&mut g, P0, src, &mark()).unwrap()[0];
    let y = g.perm(t);
    assert_eq!(y.plus, 0, "no +1/+1 counters");
    assert!(!y.tapped, "not tapped");
    assert_eq!(y.eot_pt, (0, 0), "no until-end-of-turn pump");
    assert!(y.eot_kw.is_empty(), "no until-end-of-turn keywords");
    assert!(!y.data.truthy(DataKey::Frozen), "no other state");
    assert_eq!(y.data.get(DataKey::CopySac), Some(&Val::Player(P0)), "only its own mark");
    assert!(g.player(P0).perms.iter().all(|&e| g.perm(e).attached != Some(t)), "nothing is attached to the copy");
    assert_eq!((epow(&g, t), etgh(&g, t)), (1, 1));
    assert_eq!(g.perm(src).data.get(DataKey::CopySac), Some(&Val::Player(P1)), "the source is never touched");
    assert_eq!(g.perm(src).plus, 2);
}

#[test]
fn a_copy_of_a_helm_of_the_host_token_is_not_legendary() {
    // the exception is a copiable value (707.9b): the copy of the nonlegendary token is nonlegendary too
    let mut g = tbl();
    let c = card(&g, MIKAEUS);
    let exc = CopyExc { not_legendary: true, ..Default::default() };
    let src = zones::enter_token_copy_with(&mut g, P0, c, Some(exc)).unwrap().unwrap();
    assert!(!sim_core::cardcode::is_legendary(&g, src));
    let t = zones::token_copy_of(&mut g, P0, src, &mark()).unwrap()[0];
    assert!(!sim_core::cardcode::is_legendary(&g, t), "still nonlegendary");
    assert!(g.perm(src).on_bf && g.perm(t).on_bf, "no legend rule for two nonlegendary tokens");
}

#[test]
fn a_copy_of_a_cursed_mirror_copy_keeps_haste_and_does_not_revert() {
    let mut g = tbl();
    let c = card(&g, "Llanowar Elves");
    let mirror = card(&g, "Cursed Mirror");
    let exc = CopyExc { haste: true, revert_to: Some(mirror), ..Default::default() };
    let src = zones::enter_token_copy_with(&mut g, P0, c, Some(exc)).unwrap().unwrap();
    assert_eq!(g.perm(src).exc.revert_to, Some(mirror));
    let t = zones::token_copy_of(&mut g, P0, src, &mark()).unwrap()[0];
    assert!(g.perm(t).exc.haste, "'except it has haste' is a copiable value");
    assert_eq!(g.perm(t).exc.revert_to, None, "the Mirror's own end-of-turn effect is not");
    assert!(!g.perm(t).sick, "it entered with haste (the exception clears summoning sickness as it enters)");
}

#[test]
fn a_copy_of_a_face_down_creature_is_a_plain_2_2() {
    let mut g = tbl();
    let src = perm(&mut g, P0, "Llanowar Elves");
    g.perm_mut(src).data.set(DataKey::Facedown, Val::List(vec![Val::Int(2), Val::Int(2)]));
    let made = zones::token_copy_of(&mut g, P0, src, &mark()).unwrap();
    assert_eq!(made.len(), 1);
    let t = made[0];
    assert!(g.perm(t).token && g.perm(t).cd.is_none(), "a token with no card behind it");
    assert!(g.is_creature(t));
    assert_eq!((epow(&g, t), etgh(&g, t)), (2, 2));
    assert_eq!(colors_of(&g, t), Colors::NONE, "colourless");
    assert_ne!(&*g.perm(t).name, "Llanowar Elves", "no name");
    assert!(g.perm(t).ttypes.is_empty(), "no creature types");
    assert!(!g.perm(t).data.truthy(DataKey::Facedown), "the copy is not face down itself");
    assert_eq!(g.perm(t).data.get(DataKey::CopySac), Some(&Val::Player(P0)));
}

#[test]
fn a_token_doubler_doubles_the_copy_and_every_token_is_marked() {
    let mut g = tbl();
    g.dsl_on = true;
    let bat = perm(&mut g, P0, BAT);
    let src = thopter(&mut g); // made before the doubler
    perm(&mut g, P0, "Parallel Lives");
    assert_eq!(repl::token_mult(&g, P0), 2);
    let a = zones::token_copy_of(&mut g, P0, bat, &mark()).unwrap();
    assert_eq!(a.len(), 2, "a card-backed copy makes two tokens");
    let b = zones::token_copy_of(&mut g, P0, src, &mark()).unwrap();
    assert_eq!(b.len(), 2, "a card-less copy makes two too");
    for &t in a.iter().chain(&b) {
        assert!(g.perm(t).token && g.perm(t).on_bf);
        assert_eq!(g.perm(t).data.get(DataKey::CopySac), Some(&Val::Player(P0)), "{}", g.perm(t).name);
    }
}

#[test]
fn at_the_token_limit_nothing_is_made() {
    let mut g = tbl();
    let bat = perm(&mut g, P0, BAT);
    let src = thopter(&mut g);
    let room = TOKEN_CAP - tokens_of(&g, P0);
    zones::make_tokens(&mut g, P0, Tokens::new(room as u32, 1)).unwrap();
    assert_eq!(tokens_of(&g, P0), TOKEN_CAP);
    assert!(zones::token_copy_of(&mut g, P0, bat, &mark()).unwrap().is_empty(), "a card-backed copy");
    assert!(zones::token_copy_of(&mut g, P0, src, &mark()).unwrap().is_empty(), "a card-less copy");
    assert_eq!(tokens_of(&g, P0), TOKEN_CAP);
}

#[test]
fn at_top_level_the_legend_rule_runs_before_the_trigger_and_the_trigger_still_resolves() {
    // the AI keeps the real Karn over the marked copy (even tapped); the copy's trigger draws one card per colour among the
    // other artifacts, the real Karn's five included, although the copy is gone by then
    let mut g = tbl();
    let karn = perm(&mut g, P0, KARN);
    g.perm_mut(karn).tapped = true;
    let before = g.player(P0).hand.len();
    let made = zones::token_copy_of(&mut g, P0, karn, &mark()).unwrap();
    assert_eq!(made.len(), 1);
    assert!(!g.perm(made[0]).on_bf, "the copy left to the legend rule");
    assert!(g.perm(karn).on_bf, "the real Karn stayed");
    assert!(g.trig_queue.is_empty(), "the trigger was flushed");
    assert_eq!(g.player(P0).hand.len(), before + 5, "the trigger resolved without its source");
}

#[test]
fn nested_in_another_resolution_it_neither_checks_state_nor_flushes() {
    let mut g = tbl();
    let karn = perm(&mut g, P0, KARN);
    g.resolving = 1; // inside an outer resolution
    let made = zones::token_copy_of(&mut g, P0, karn, &mark()).unwrap();
    assert!(g.perm(made[0]).on_bf && g.perm(karn).on_bf, "the legend rule is the outer resolution's to run");
    assert!(!g.trig_queue.is_empty(), "so are the queued triggers");
    g.resolving = 0;
    check_state(&mut g).unwrap();
    assert_eq!(perms_named(&g, P0, KARN).len(), 1);
}

#[test]
fn a_marked_copy_of_a_legendary_creature_never_beats_the_card_in_the_legend_rule() {
    // without the mark the AI keeps the untapped token over the tapped card; with it, the card
    let mut g = tbl();
    let real = perm(&mut g, P0, MIKAEUS);
    g.perm_mut(real).tapped = true;
    let c = card(&g, MIKAEUS);
    let unmarked = zones::enter_token_copy(&mut g, P0, c).unwrap().unwrap();
    check_state(&mut g).unwrap();
    assert!(
        g.perm(unmarked).on_bf && !g.perm(real).on_bf,
        "an unmarked untapped token beats a tapped card (token {}, card {})",
        g.perm(unmarked).on_bf,
        g.perm(real).on_bf
    );
    let mut g = tbl();
    let real = perm(&mut g, P0, MIKAEUS);
    g.perm_mut(real).tapped = true;
    let t = zones::token_copy_of(&mut g, P0, real, &mark()).unwrap();
    assert!(g.perm(real).on_bf && !g.perm(t[0]).on_bf, "a marked copy loses");
}
