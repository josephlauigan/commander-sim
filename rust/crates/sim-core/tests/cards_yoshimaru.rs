//! Cards of the Yoshimaru / Tymna blink deck, modelled before the deck is registered (impls/yoshimaru_cards.rs and the
//! compiled Oracle text): coverage rows, no red/green. Each expectation follows the card's Oracle text.

mod play_common;
use play_common::*;
use sim_core::cards::Types;
use sim_core::engine::values::{epow, indestructible};
use sim_core::engine::{cast, turn, zones};
use sim_core::ids::CardId;
use sim_core::state::Ctx;
use sim_core::state::Step;
use sim_core::state::{DataKey, Val};

fn tbl() -> Game {
    let mut g = table(&["heliod-mono-white-stax", "seph", "sauron"]);
    g.active = Some(P0);
    g
}

/// a person at P0 who answers by what is asked: `pick` gives the index of the choice to take (None: decline); priority
/// is always passed
fn person(g: &mut Game, pick: impl Fn(&Request) -> Option<usize> + Send + 'static) -> std::sync::Arc<ScriptHuman> {
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        match pick(r) {
            Some(i) => Answer::Pick(i),
            None => Answer::Decline,
        }
    }));
    g.humans.add(P0, h.clone());
    h
}

fn choice_with(r: &Request, word: &str) -> Option<usize> {
    r.choices.iter().position(|c| c.contains(word))
}

fn counters(g: &Game, m: PermId) -> i32 {
    g.perm(m).plus
}

fn play_land(g: &mut Game, name: &str) -> PermId {
    let c = take(g, P0, name);
    g.player_mut(P0).hand.push(c);
    let n = g.player(P0).lands.len();
    g.player_mut(P0).hand.retain(|&x| x != c);
    turn::play_land_card(g, P0, c, "plays").unwrap();
    assert_eq!(g.player(P0).lands.len(), n + 1);
    *g.player(P0).lands.last().unwrap()
}

// ------------------------------------------------------------------ Yoshimaru, Ever Faithful
#[test]
fn yoshimaru_grows_for_another_legendary_creature_you_control() {
    let mut g = tbl();
    let y = perm(&mut g, P0, "Yoshimaru, Ever Faithful");
    assert_eq!(counters(&g, y), 0, "his own entry gives nothing");
    perm(&mut g, P0, "Tymna the Weaver");
    assert_eq!(counters(&g, y), 1);
    perm(&mut g, P0, "Kambal, Consul of Allocation");
    assert_eq!(counters(&g, y), 2);
}

#[test]
fn yoshimaru_counts_legendary_artifacts_and_planeswalkers_too() {
    let mut g = tbl();
    let y = perm(&mut g, P0, "Yoshimaru, Ever Faithful");
    perm(&mut g, P0, "The One Ring");
    perm(&mut g, P0, "Sorin, Vengeful Bloodlord");
    assert_eq!(counters(&g, y), 2);
}

#[test]
fn yoshimaru_ignores_a_permanent_that_is_not_legendary() {
    let mut g = tbl();
    let y = perm(&mut g, P0, "Yoshimaru, Ever Faithful");
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P0, "Mother of Runes");
    token(&mut g, P0, 2);
    assert_eq!(counters(&g, y), 0);
}

#[test]
fn yoshimaru_ignores_a_legendary_permanent_an_opponent_controls() {
    let mut g = tbl();
    let y = perm(&mut g, P0, "Yoshimaru, Ever Faithful");
    perm(&mut g, P1, "Tymna the Weaver");
    assert_eq!(counters(&g, y), 0);
}

#[test]
fn yoshimaru_must_be_there_first() {
    let mut g = tbl();
    perm(&mut g, P0, "Tymna the Weaver");
    let y = perm(&mut g, P0, "Yoshimaru, Ever Faithful");
    assert_eq!(counters(&g, y), 0, "Tymna came first");
    perm(&mut g, P0, "The One Ring");
    assert_eq!(counters(&g, y), 1);
}

#[test]
fn yoshimaru_counts_a_legendary_land_you_play_once() {
    let mut g = tbl();
    let y = perm(&mut g, P0, "Yoshimaru, Ever Faithful");
    play_land(&mut g, "Plains");
    assert_eq!(counters(&g, y), 0, "a basic land is not legendary");
    play_land(&mut g, "Minas Tirith");
    assert_eq!(counters(&g, y), 1);
    zones::landfall(&mut g, P0).unwrap(); // landfall again, no new land: nothing more
    assert_eq!(counters(&g, y), 1);
    play_land(&mut g, "Eiganjo, Seat of the Empire");
    assert_eq!(counters(&g, y), 2);
}

#[test]
fn yoshimaru_does_not_count_a_legendary_land_that_was_there_before_him() {
    let mut g = tbl();
    play_land(&mut g, "Minas Tirith");
    let y = perm(&mut g, P0, "Yoshimaru, Ever Faithful");
    play_land(&mut g, "Plains");
    assert_eq!(counters(&g, y), 0);
}

#[test]
fn yoshimaru_ignores_an_opponents_legendary_land() {
    let mut g = tbl();
    let y = perm(&mut g, P0, "Yoshimaru, Ever Faithful");
    let c = take(&mut g, P1, "Minas Tirith");
    g.player_mut(P1).hand.push(c);
    g.player_mut(P1).hand.retain(|&x| x != c);
    turn::play_land_card(&mut g, P1, c, "plays").unwrap();
    assert_eq!(counters(&g, y), 0);
}

#[test]
fn yoshimaru_phased_out_sees_nothing() {
    let mut g = tbl();
    let y = perm(&mut g, P0, "Yoshimaru, Ever Faithful");
    g.perm_mut(y).phased = true;
    perm(&mut g, P0, "Tymna the Weaver");
    play_land(&mut g, "Minas Tirith");
    assert_eq!(counters(&g, y), 0);
}

#[test]
fn a_person_s_yoshimaru_asks_nothing() {
    // no choice in the card: the person is asked only for priority on the trigger
    let mut g = tbl();
    let h = person(&mut g, |_| None);
    let y = perm(&mut g, P0, "Yoshimaru, Ever Faithful");
    perm(&mut g, P0, "Tymna the Weaver");
    assert_eq!(counters(&g, y), 1);
    assert!(
        h.asked().iter().all(|r| r.kind == "priority"),
        "{:?}",
        h.asked().iter().map(|r| &r.kind).collect::<Vec<_>>()
    );
}

// ------------------------------------------------------------------ Kambal, Consul of Allocation
fn cast_from_hand(g: &mut Game, p: PlayerId, name: &str) {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
}

#[test]
fn kambal_drains_an_opponent_who_casts_a_noncreature_spell() {
    let mut g = tbl();
    perm(&mut g, P0, "Kambal, Consul of Allocation");
    let (mine, theirs) = (g.player(P0).life, g.player(P1).life);
    lands(&mut g, P1, "Plains", 3, false);
    cast_from_hand(&mut g, P1, "Sol Ring");
    assert_eq!(g.player(P1).life, theirs - 2);
    assert_eq!(g.player(P0).life, mine + 2);
}

#[test]
fn kambal_drains_whichever_opponent_cast_it() {
    let mut g = tbl();
    perm(&mut g, P0, "Kambal, Consul of Allocation");
    let (mine, p1, p2) = (g.player(P0).life, g.player(P1).life, g.player(P2).life);
    lands(&mut g, P2, "Plains", 3, false);
    cast_from_hand(&mut g, P2, "Sol Ring");
    assert_eq!((g.player(P1).life, g.player(P2).life), (p1, p2 - 2));
    assert_eq!(g.player(P0).life, mine + 2);
}

#[test]
fn kambal_ignores_a_creature_spell() {
    let mut g = tbl();
    perm(&mut g, P0, "Kambal, Consul of Allocation");
    let (mine, theirs) = (g.player(P0).life, g.player(P1).life);
    lands(&mut g, P1, "Plains", 3, false);
    cast_from_hand(&mut g, P1, "Mother of Runes");
    assert_eq!((g.player(P0).life, g.player(P1).life), (mine, theirs));
}

#[test]
fn kambal_ignores_your_own_noncreature_spell() {
    let mut g = tbl();
    perm(&mut g, P0, "Kambal, Consul of Allocation");
    let (mine, theirs) = (g.player(P0).life, g.player(P1).life);
    lands(&mut g, P0, "Plains", 3, false);
    cast_from_hand(&mut g, P0, "Sol Ring");
    assert_eq!((g.player(P0).life, g.player(P1).life), (mine, theirs));
}

#[test]
fn a_person_s_kambal_asks_nothing() {
    let mut g = tbl();
    let h = person(&mut g, |_| None);
    perm(&mut g, P0, "Kambal, Consul of Allocation");
    let mine = g.player(P0).life;
    lands(&mut g, P1, "Plains", 3, false);
    cast_from_hand(&mut g, P1, "Sol Ring");
    assert_eq!(g.player(P0).life, mine + 2);
    assert!(h.asked().iter().all(|r| r.kind == "priority"));
}

// ------------------------------------------------------------------ Minas Tirith
fn minas(g: &mut Game) -> CardId {
    card(g, "Minas Tirith")
}

#[test]
fn minas_tirith_enters_tapped_without_a_legendary_creature() {
    let mut g = tbl();
    let c = minas(&mut g);
    assert!(turn::land_enters_tapped(&g, P0, c));
    perm(&mut g, P0, "Mother of Runes");
    assert!(turn::land_enters_tapped(&g, P0, c), "a creature that is not legendary does not help");
    let l = play_land(&mut g, "Minas Tirith");
    assert!(g.perm(l).tapped);
}

#[test]
fn minas_tirith_enters_untapped_with_a_legendary_creature() {
    let mut g = tbl();
    perm(&mut g, P0, "Tymna the Weaver");
    let l = play_land(&mut g, "Minas Tirith");
    assert!(!g.perm(l).tapped);
    assert!(g.db.get(g.perm(l).card()).types.has(Types::LAND));
}

/// the Ring tempts P0 once and `m` is their Ring-bearer (legendary while it is)
fn ring_bearer_is(g: &mut Game, m: PermId) {
    g.player_mut(P0).ring_level = 1;
    g.player_mut(P0).ring_bearer = Some(m);
}

#[test]
fn minas_tirith_counts_the_ring_bearer_as_legendary() {
    let mut g = tbl();
    let c = minas(&mut g);
    let m = perm(&mut g, P0, "Mother of Runes");
    assert!(turn::land_enters_tapped(&g, P0, c), "not yet the Ring-bearer");
    ring_bearer_is(&mut g, m);
    assert!(!turn::land_enters_tapped(&g, P0, c), "the Ring-bearer is legendary");
    let l = play_land(&mut g, "Minas Tirith");
    assert!(!g.perm(l).tapped);
}

#[test]
fn minas_tirith_ignores_a_phased_out_legendary_creature() {
    let mut g = tbl();
    let c = minas(&mut g);
    let t = perm(&mut g, P0, "Tymna the Weaver");
    assert!(!turn::land_enters_tapped(&g, P0, c));
    g.perm_mut(t).phased = true;
    assert!(turn::land_enters_tapped(&g, P0, c), "a phased-out creature is treated as though it isn't there");
    let l = play_land(&mut g, "Minas Tirith");
    assert!(g.perm(l).tapped);
}

#[test]
fn minas_tirith_counts_a_legendary_token() {
    let mut g = tbl();
    let c = minas(&mut g);
    let t = token(&mut g, P0, 2);
    assert!(turn::land_enters_tapped(&g, P0, c), "a plain token is not legendary");
    g.perm_mut(t).data.set(DataKey::Legendary, Val::Bool(true));
    assert!(!turn::land_enters_tapped(&g, P0, c), "a legendary token is");
}

#[test]
fn minas_tirith_ignores_a_nonlegendary_copy_of_a_legend() {
    let mut g = tbl();
    let c = minas(&mut g);
    let t = perm(&mut g, P0, "Tymna the Weaver");
    g.perm_mut(t).exc.not_legendary = true; // Spark Double and the Ur-Dragon copies: "it isn't legendary"
    assert!(turn::land_enters_tapped(&g, P0, c));
}

#[test]
fn barad_dur_enters_untapped_with_a_nonlegendary_ring_bearer() {
    // the sheoldred list's Barad-dur has the same rule: no printed legend out, but the Ring-bearer is legendary
    let mut g = table(&["sheoldred", "seph"]);
    g.active = Some(P0);
    let c = card(&g, "Barad-dûr");
    let m = perm(&mut g, P0, "Mother of Runes");
    assert!(turn::land_enters_tapped(&g, P0, c));
    ring_bearer_is(&mut g, m);
    assert!(!turn::land_enters_tapped(&g, P0, c));
}

#[test]
fn minas_tirith_counts_a_token_ring_bearer_as_legendary() {
    // the Army from Sauron's tempt is a token: it has no card, but the Ring-bearer is legendary all the same
    let mut g = tbl();
    let c = minas(&mut g);
    let t = token(&mut g, P0, 2);
    g.player_mut(P0).ring_bearer = Some(t); // chosen, but the Ring has not tempted P0 yet (level 0)
    assert!(turn::land_enters_tapped(&g, P0, c), "no Ring, no legend");
    g.player_mut(P0).ring_level = 1;
    assert!(sim_core::impls::legends::legendary(&g, t), "the Ring-bearer is legendary, token or not");
    assert!(!turn::land_enters_tapped(&g, P0, c), "and Minas Tirith sees it");
    let other = token(&mut g, P0, 1);
    assert!(!sim_core::impls::legends::legendary(&g, other), "another token is not");
}

#[test]
fn barad_dur_enters_untapped_with_a_token_ring_bearer() {
    let mut g = table(&["sheoldred", "seph"]);
    g.active = Some(P0);
    let c = card(&g, "Barad-dûr");
    let t = token(&mut g, P0, 2);
    assert!(turn::land_enters_tapped(&g, P0, c));
    ring_bearer_is(&mut g, t);
    assert!(!turn::land_enters_tapped(&g, P0, c));
}

/// a person with Minas Tirith untapped, two attackers this turn and {W}{C} in the pool; "which ability?" takes the first
fn ready_person_minas(g: &mut Game) -> PermId {
    person(g, |r| r.prompt.contains("which ability?").then_some(0));
    let l = play_land(g, "Minas Tirith");
    g.perm_mut(l).tapped = false;
    attacked_with(g, 2);
    pool(g, P0, &[('W', 1), ('C', 1)]);
    l
}

fn no_draw_label(g: &Game, l: PermId) -> bool {
    !sim_core::human::abilities::land_abilities(g, P0, l).iter().any(|a| a.label.contains("draw a card"))
}

#[test]
fn blood_moon_removes_the_person_draw_ability() {
    let mut g = tbl();
    let l = ready_person_minas(&mut g);
    perm(&mut g, P1, "Blood Moon");
    assert!(sim_core::engine::mana::moon_land(&g, l));
    assert!(no_draw_label(&g, l), "a Mountain has no draw ability");
    let before = (g.player(P0).hand.len(), g.player(P0).pool.total());
    let _ = sim_core::human::human::use_land(&mut g, P0, l).unwrap();
    assert_eq!((g.player(P0).hand.len(), g.player(P0).pool.total()), before, "nothing drawn, nothing paid");
    assert!(!g.perm(l).tapped);
}

#[test]
fn magus_of_the_moon_removes_the_person_draw_ability() {
    let mut g = tbl();
    let l = ready_person_minas(&mut g);
    perm(&mut g, P1, "Magus of the Moon");
    assert!(sim_core::engine::mana::moon_land(&g, l));
    assert!(no_draw_label(&g, l), "a Mountain has no draw ability");
    let hand = g.player(P0).hand.len();
    let _ = sim_core::human::human::use_land(&mut g, P0, l).unwrap();
    assert_eq!(g.player(P0).hand.len(), hand);
    assert!(!g.perm(l).tapped);
}

#[test]
fn a_person_may_draw_from_an_empty_library_and_lose() {
    // the whole main-phase loop: the draw is attempted, and the player loses before priority comes back
    let mut g = tbl();
    let l = ready_person_minas(&mut g);
    g.player_mut(P0).library.clear();
    let mut activated = false;
    g.humans.add(
        P0,
        std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
            if r.kind == "priority" {
                if activated {
                    Answer::Action(json!({"do": "pass"}))
                } else {
                    activated = true;
                    Answer::Action(json!({"do": "use", "land": 0}))
                }
            } else if r.prompt.contains("which ability?") {
                Answer::Pick(0)
            } else {
                Answer::Decline
            }
        })),
    );
    sim_core::human::human::human_main(&mut g, P0, false).unwrap();
    assert!(g.player(P0).decked, "the draw was attempted");
    assert!(!g.player(P0).alive, "and a player who drew from an empty library has lost");
    assert!(g.perm(l).tapped);
}

#[test]
fn cacophonys_temporary_legendary_supertype_expires() {
    // Cacophony Unleashed becomes a legendary 6/6 creature until end of turn: the supertype goes with the rest
    let mut g = tbl();
    let c = perm(&mut g, P0, "Cacophony Unleashed");
    assert!(g.is_creature(c));
    assert!(sim_core::impls::legends::legendary(&g, c));
    turn::end_step(&mut g, P0).unwrap();
    assert!(!g.is_creature(c), "the animation ended");
    assert!(!sim_core::impls::legends::legendary(&g, c), "so did the legendary supertype of the same effect");
}

#[test]
fn sisay_loses_cacophonys_colour_after_the_effect_ends() {
    let mut g = tbl();
    let s = perm(&mut g, P0, "Sisay, Weatherlight Captain");
    let c = perm(&mut g, P0, "Cacophony Unleashed");
    assert!(sim_core::impls::legends::legendary(&g, c));
    assert_eq!(epow(&g, s), 3);
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(epow(&g, s), 2, "Cacophony is no longer a legendary permanent");
}

#[test]
fn a_card_made_legendary_by_an_effect_is_legendary() {
    // DataKey::Legendary on a card-backed permanent (Anikthea's tokens, a Kaldra-style effect), not only on a token
    let mut g = tbl();
    let c = minas(&mut g);
    let m = perm(&mut g, P0, "Mother of Runes");
    assert!(!sim_core::impls::legends::legendary(&g, m));
    assert!(turn::land_enters_tapped(&g, P0, c));
    g.perm_mut(m).data.set(DataKey::Legendary, Val::Bool(true));
    assert!(sim_core::impls::legends::legendary(&g, m));
    assert!(!turn::land_enters_tapped(&g, P0, c), "and Minas Tirith sees it");
}

fn ready_minas(g: &mut Game) -> PermId {
    let l = play_land(g, "Minas Tirith");
    g.perm_mut(l).tapped = false;
    lands(g, P0, "Plains", 2, false);
    l
}

fn attacked_with(g: &mut Game, n: u32) {
    let st = g.turn_stamp();
    let attackers: Vec<PermId> = (0..n).map(|_| token(g, P0, 2)).collect();
    let pl = g.player_mut(P0);
    pl.attacking = Some(st);
    pl.attackers = attackers;
}

fn draw_options(g: &mut Game, l: PermId, post: Option<bool>) -> Vec<sim_core::hooks::Opt> {
    let f = g.registry.get(g.perm(l).card()).unwrap().land_options.unwrap();
    f(g, l, P0, post).unwrap()
}

#[test]
fn minas_tirith_draws_after_attacking_with_two_creatures() {
    let mut g = tbl();
    let l = ready_minas(&mut g);
    attacked_with(&mut g, 2);
    let o = draw_options(&mut g, l, Some(true));
    assert_eq!(o.len(), 1);
    let hand = g.player(P0).hand.len();
    assert!(sim_core::ai::act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P0).hand.len(), hand + 1);
    assert!(g.perm(l).tapped, "it taps as a cost");
    assert!(g.player(P0).lands.iter().filter(|&&x| g.perm(x).tapped).count() >= 3, "and {{1}}{{W}} was paid");
    assert!(draw_options(&mut g, l, Some(true)).is_empty(), "tapped: not again");
}

#[test]
fn minas_tirith_does_not_draw_without_two_attackers() {
    let mut g = tbl();
    let l = ready_minas(&mut g);
    assert!(draw_options(&mut g, l, Some(true)).is_empty(), "nobody attacked");
    attacked_with(&mut g, 1);
    assert!(draw_options(&mut g, l, Some(true)).is_empty(), "one attacker is not two");
    // two attacked, but on an earlier turn
    attacked_with(&mut g, 2);
    g.player_mut(P0).attacking = None;
    assert!(draw_options(&mut g, l, Some(true)).is_empty());
}

#[test]
fn minas_tirith_needs_the_mana() {
    let mut g = tbl();
    let l = play_land(&mut g, "Minas Tirith");
    g.perm_mut(l).tapped = false;
    attacked_with(&mut g, 3);
    assert!(draw_options(&mut g, l, Some(true)).is_empty(), "no other land to pay {{1}}{{W}}");
    lands(&mut g, P0, "Swamp", 2, false);
    assert!(draw_options(&mut g, l, Some(true)).is_empty(), "no white mana but its own");
}

/// everything a person is offered on the land: the ability probe's table (`abilities_of`, `land_abilities`, `land_alts`)
fn offered_labels(g: &mut Game, l: PermId) -> Vec<String> {
    let mut v: Vec<String> = sim_core::human::human::abilities_of(g, P0, l).unwrap().into_iter().map(|a| a.0).collect();
    v.extend(sim_core::human::abilities::land_abilities(g, P0, l).into_iter().map(|a| a.label));
    v.extend(sim_core::human::alt::land_alts(g, P0, l).unwrap().into_iter().map(|a| a.0));
    v
}

fn draw_labels(g: &mut Game, l: PermId) -> Vec<String> {
    offered_labels(g, l).into_iter().filter(|x| x.contains("draw a card")).collect()
}

/// the person's Minas Tirith draw: Some(why) when it can't be done now
fn use_draw(g: &mut Game, l: PermId) -> Option<String> {
    let ab = sim_core::human::abilities::land_abilities(g, P0, l);
    let a = ab.iter().find(|a| a.label.contains("draw a card"));
    assert!(a.is_some(), "the draw is a land ability: {:?}", ab.iter().map(|a| &a.label).collect::<Vec<_>>());
    (a.unwrap().act)(g, P0, l).unwrap()
}

#[test]
fn a_person_uses_minas_tirith_from_their_pool() {
    let mut g = tbl();
    person(&mut g, |_| None);
    let l = play_land(&mut g, "Minas Tirith");
    g.perm_mut(l).tapped = false;
    attacked_with(&mut g, 2);
    g.player_mut(P0).pool.add('W', 2);
    let labels = draw_labels(&mut g, l);
    assert_eq!(labels.len(), 1, "{labels:?}");
    let hand = g.player(P0).hand.len();
    assert_eq!(use_draw(&mut g, l), None);
    assert_eq!(g.player(P0).hand.len(), hand + 1);
    assert_eq!(g.player(P0).pool.total(), 0, "the {{1}}{{W}} came out of the pool");
    assert!(g.perm(l).tapped, "it taps as a cost");
}

#[test]
fn a_person_is_offered_minas_tirith_draw_and_told_why_before_attacking() {
    let mut g = tbl();
    person(&mut g, |_| None);
    let l = play_land(&mut g, "Minas Tirith");
    g.perm_mut(l).tapped = false;
    g.player_mut(P0).pool.add('W', 2);
    // the ability probe's own check: with nobody attacked, the land still lists something
    assert!(!sim_core::human::abilities::land_abilities(&g, P0, l).is_empty());
    assert_eq!(draw_labels(&mut g, l).len(), 1);
    let hand = g.player(P0).hand.len();
    let why = use_draw(&mut g, l).expect("nobody has attacked");
    assert!(why.contains("attacked with two or more creatures"), "{why}");
    attacked_with(&mut g, 1);
    assert!(use_draw(&mut g, l).expect("one attacker").contains("two or more"));
    attacked_with(&mut g, 2);
    let _ = std::mem::take(&mut g.player_mut(P0).pool);
    let why = use_draw(&mut g, l).expect("no mana");
    assert!(why.contains("Can't activate Minas Tirith"), "{why}");
    g.player_mut(P0).pool.add('W', 2);
    g.perm_mut(l).tapped = true;
    assert!(use_draw(&mut g, l).expect("tapped").contains("tapped"));
    assert_eq!(g.player(P0).hand.len(), hand, "none of them drew");
    assert_eq!(g.player(P0).pool.total(), 2, "or paid");
}

#[test]
fn a_person_sees_minas_tirith_draw_once() {
    let mut g = tbl();
    person(&mut g, |_| None);
    let l = play_land(&mut g, "Minas Tirith");
    g.perm_mut(l).tapped = false;
    g.player_mut(P0).pool.add('W', 2);
    assert_eq!(draw_labels(&mut g, l).len(), 1, "before attacking");
    attacked_with(&mut g, 2);
    assert_eq!(
        draw_labels(&mut g, l).len(),
        1,
        "after two attackers: not listed twice: {:?}",
        offered_labels(&mut g, l)
    );
}

// ------------------------------------------------------------------ The Seriema
fn main1(g: &mut Game) {
    g.active = Some(P0);
    g.step = Step::Main1;
}

fn library(g: &mut Game, names: &[&str]) {
    let cs: Vec<CardId> = names.iter().map(|n| card(g, n)).collect();
    g.player_mut(P0).library = cs;
}

fn in_hand(g: &Game, name: &str) -> bool {
    g.player(P0).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

#[test]
fn the_seriema_fetches_a_legendary_creature_when_it_enters() {
    let mut g = tbl();
    library(&mut g, &["Sol Ring", "Tymna the Weaver", "Mother of Runes"]);
    perm(&mut g, P0, "The Seriema");
    assert!(in_hand(&g, "Tymna the Weaver"));
    assert!(!in_hand(&g, "Sol Ring") && !in_hand(&g, "Mother of Runes"), "only a legendary creature");
    assert_eq!(g.player(P0).library.len(), 2);
}

#[test]
fn the_seriema_finds_nothing_without_a_legendary_creature_card() {
    let mut g = tbl();
    library(&mut g, &["Sol Ring", "The One Ring", "Mother of Runes"]);
    let hand = g.player(P0).hand.len();
    perm(&mut g, P0, "The Seriema");
    assert_eq!(g.player(P0).hand.len(), hand, "a legendary artifact is not a legendary creature");
}

#[test]
fn a_person_chooses_the_legendary_creature_the_seriema_fetches() {
    let mut g = tbl();
    library(&mut g, &["Tymna the Weaver", "Kambal, Consul of Allocation", "Sol Ring"]);
    let h = person(&mut g, |r| choice_with(r, "Kambal"));
    perm(&mut g, P0, "The Seriema");
    assert!(
        in_hand(&g, "Kambal, Consul of Allocation"),
        "asked: {:?}",
        h.asked().iter().map(|r| &r.prompt).collect::<Vec<_>>()
    );
    assert!(!in_hand(&g, "Tymna the Weaver"));
    let shown = h.asked().into_iter().find(|r| r.choices.iter().any(|c| c.contains("Kambal"))).unwrap();
    assert!(shown.choices.iter().all(|c| !c.contains("Sol Ring")), "{:?}", shown.choices);
}

fn seriema(g: &mut Game) -> PermId {
    library(g, &[]);
    let s = perm(g, P0, "The Seriema");
    main1(g);
    s
}

fn options_of(g: &mut Game, s: PermId, post: Option<bool>) -> Vec<sim_core::hooks::Opt> {
    let f = g.registry.get(g.perm(s).cd.unwrap()).unwrap().options.unwrap();
    f(g, s, P0, post).unwrap()
}

fn run(g: &mut Game, o: &sim_core::hooks::Opt) -> bool {
    sim_core::ai::act::perform(g, P0, o.act.as_ref().unwrap()).unwrap()
}

fn charges(g: &Game, s: PermId) -> i64 {
    g.perm(s).data.int(sim_core::state::DataKey::Charge)
}

#[test]
fn stationing_taps_a_creature_for_charge_counters_equal_to_its_power() {
    let mut g = tbl();
    let s = seriema(&mut g);
    let t = token(&mut g, P0, 3);
    g.perm_mut(t).sick = true; // summoning sick: it can't attack, so the AI taps it
    let o = options_of(&mut g, s, Some(false));
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, &o[0]));
    assert_eq!(charges(&g, s), 3);
    assert!(g.perm(t).tapped);
    assert!(!g.is_creature(s), "3 counters is not 7");
    assert!(options_of(&mut g, s, Some(false)).is_empty(), "no untapped creature left");
}

#[test]
fn seven_charge_counters_make_it_a_5_5_flying_artifact_creature() {
    let mut g = tbl();
    let s = seriema(&mut g);
    for pw in [4, 3] {
        let t = token(&mut g, P0, pw);
        g.perm_mut(t).sick = true;
    }
    while let Some(o) = options_of(&mut g, s, Some(false)).first().cloned() {
        assert!(run(&mut g, &o));
    }
    assert_eq!(charges(&g, s), 7);
    assert!(g.is_creature(s));
    assert_eq!((epow(&g, s), sim_core::engine::values::etgh(&g, s)), (5, 5));
    assert!(sim_core::dsl::has_kw(&g, s, "flying"));
    assert!(g.db.get(g.perm(s).card()).types.has(Types::ARTIFACT), "still an artifact");
}

#[test]
fn six_charge_counters_are_not_enough() {
    let mut g = tbl();
    let s = seriema(&mut g);
    let t = token(&mut g, P0, 6);
    g.perm_mut(t).sick = true;
    let o = options_of(&mut g, s, Some(false));
    assert!(run(&mut g, &o[0]));
    assert_eq!(charges(&g, s), 6);
    assert!(!g.is_creature(s));
    assert!(!sim_core::dsl::has_kw(&g, s, "flying"));
}

#[test]
fn stationing_is_only_a_sorcery_and_needs_another_untapped_creature() {
    let mut g = tbl();
    let s = seriema(&mut g);
    let t = token(&mut g, P0, 3);
    g.perm_mut(t).sick = true;
    let o = options_of(&mut g, s, Some(false)).remove(0);
    g.step = Step::Combat;
    assert!(!run(&mut g, &o), "not in combat");
    g.step = Step::Main1;
    g.active = Some(P1);
    assert!(!run(&mut g, &o), "not on an opponent's turn");
    assert!(options_of(&mut g, s, None).is_empty(), "no instant-speed option");
    g.active = Some(P0);
    g.perm_mut(t).tapped = true;
    assert!(!run(&mut g, &o), "the creature is already tapped");
    assert_eq!(charges(&g, s), 0);
    assert!(options_of(&mut g, s, Some(false)).is_empty());
}

#[test]
fn the_ai_keeps_creatures_that_can_attack_unless_the_taps_finish_it() {
    let mut g = tbl();
    let s = seriema(&mut g);
    token(&mut g, P0, 3); // not summoning sick: it would attack
    assert!(options_of(&mut g, s, Some(false)).is_empty());
    token(&mut g, P0, 4); // 3 + 4 reaches 7: now it taps
    assert!(!options_of(&mut g, s, Some(false)).is_empty());
}

#[test]
fn the_seriema_gives_tapped_legendary_creatures_indestructible_at_7() {
    let mut g = tbl();
    let s = seriema(&mut g);
    let tymna = perm(&mut g, P0, "Tymna the Weaver");
    let mom = perm(&mut g, P0, "Mother of Runes");
    let theirs = perm(&mut g, P1, "Kambal, Consul of Allocation");
    for m in [tymna, mom, theirs] {
        g.perm_mut(m).tapped = true;
    }
    assert!(!indestructible(&g, tymna), "under 7 it grants nothing");
    g.perm_mut(s).data.set(sim_core::state::DataKey::Charge, sim_core::state::Val::Int(7));
    assert!(indestructible(&g, tymna), "a tapped legendary creature of yours");
    assert!(!indestructible(&g, mom), "not legendary");
    assert!(!indestructible(&g, theirs), "an opponent's");
    assert!(!indestructible(&g, s), "other creatures only");
    g.perm_mut(tymna).tapped = false;
    assert!(!indestructible(&g, tymna), "untapped");
}

#[test]
fn a_person_chooses_the_creature_to_station_with() {
    let mut g = tbl();
    let s = seriema(&mut g);
    let a = token(&mut g, P0, 2);
    let b = token(&mut g, P0, 5);
    let h = person(&mut g, |r| choice_with(r, "power 5"));
    let o = options_of(&mut g, s, Some(false));
    assert_eq!(o.len(), 1, "one way to station, the person picks the creature");
    assert!(run(&mut g, &o[0]));
    assert!(g.perm(b).tapped && !g.perm(a).tapped);
    assert_eq!(charges(&g, s), 5);
    let q = h.asked().into_iter().find(|r| r.kind == "choose").unwrap();
    assert_eq!(q.choices.len(), 3, "both creatures and cancel: {:?}", q.choices);
}

#[test]
fn a_person_can_cancel_stationing_and_nothing_is_tapped() {
    let mut g = tbl();
    let s = seriema(&mut g);
    let a = token(&mut g, P0, 2);
    person(&mut g, |_| None);
    let o = options_of(&mut g, s, Some(false));
    assert!(!run(&mut g, &o[0]));
    assert!(!g.perm(a).tapped);
    assert_eq!(charges(&g, s), 0);
}

// ------------------------------------------------------------------ Witch Enchanter // Witch-Blessed Meadow
const WITCH: &str = "Witch Enchanter // Witch-Blessed Meadow";

fn on_bf(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

#[test]
fn witch_enchanter_destroys_an_artifact_or_enchantment_an_opponent_controls() {
    let mut g = tbl();
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P0, "Mind Stone");
    perm(&mut g, P1, "Mother of Runes");
    perm(&mut g, P0, WITCH);
    assert!(!on_bf(&g, P1, "Sol Ring"), "the opponent's artifact is destroyed");
    assert!(on_bf(&g, P0, "Mind Stone"), "not your own");
    assert!(on_bf(&g, P1, "Mother of Runes"), "not a creature");
}

#[test]
fn witch_enchanter_destroys_an_enchantment_too() {
    let mut g = tbl();
    perm(&mut g, P1, "Rhystic Study");
    perm(&mut g, P0, WITCH);
    assert!(!on_bf(&g, P1, "Rhystic Study"));
}

#[test]
fn witch_enchanter_with_nothing_to_destroy_does_nothing() {
    let mut g = tbl();
    let mine = perm(&mut g, P0, "Mind Stone");
    perm(&mut g, P1, "Mother of Runes");
    perm(&mut g, P0, WITCH);
    assert!(g.perm(mine).on_bf);
}

#[test]
fn a_person_chooses_what_witch_enchanter_destroys() {
    let mut g = tbl();
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Rhystic Study");
    let h = person(&mut g, |r| choice_with(r, "Rhystic"));
    perm(&mut g, P0, WITCH);
    assert!(
        !on_bf(&g, P1, "Rhystic Study"),
        "asked: {:?}",
        h.asked().iter().map(|r| (&r.prompt, &r.choices)).collect::<Vec<_>>()
    );
    assert!(on_bf(&g, P1, "Sol Ring"));
}

fn meadow_hand(g: &mut Game) -> CardId {
    hand(g, P0, &[WITCH])[0]
}

fn hand_options(g: &mut Game, c: CardId, post: Option<bool>) -> Vec<sim_core::hooks::Opt> {
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    f(g, c, P0, post).unwrap()
}

#[test]
fn the_ai_plays_the_meadow_as_a_land_when_short_of_lands() {
    let mut g = tbl();
    main1(&mut g);
    g.player_mut(P0).hand.clear();
    let c = meadow_hand(&mut g);
    let o = hand_options(&mut g, c, Some(false));
    assert_eq!(o.len(), 1);
    assert!(sim_core::ai::act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert!(!g.player(P0).hand.contains(&c));
    let l = *g.player(P0).lands.last().unwrap();
    assert_eq!(g.perm(l).card(), c);
    assert!(hand_options(&mut g, c, Some(false)).is_empty(), "the land drop is used and the card is gone");
}

#[test]
fn the_meadow_taps_for_white() {
    let mut g = tbl();
    main1(&mut g);
    g.player_mut(P0).hand.clear();
    let c = meadow_hand(&mut g);
    let o = hand_options(&mut g, c, Some(false));
    sim_core::ai::act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap();
    let l = *g.player(P0).lands.last().unwrap();
    g.perm_mut(l).tapped = false;
    assert!(sim_core::engine::mana::can_pay(&mut g, P0, 0, "W", false));
    assert!(!sim_core::engine::mana::can_pay(&mut g, P0, 0, "B", false));
}

#[test]
fn a_person_pays_3_life_for_an_untapped_meadow() {
    let mut g = tbl();
    main1(&mut g);
    person(&mut g, |r| if r.prompt.contains("pay 3 life") { Some(0) } else { None });
    let c = meadow_hand(&mut g);
    let life = g.player(P0).life;
    let ways = sim_core::human::alt::hand_alts(&mut g, P0, c).unwrap();
    let (_, o) = ways.into_iter().find(|(l, _)| l.contains("as a land")).expect("a person can play it as a land");
    assert!(sim_core::human::alt::run(&mut g, P0, c, &o).unwrap().is_none());
    let l = *g.player(P0).lands.last().unwrap();
    assert_eq!(g.perm(l).card(), c);
    assert!(!g.perm(l).tapped);
    assert_eq!(g.player(P0).life, life - 3);
}

#[test]
fn a_person_who_does_not_pay_gets_a_tapped_meadow_and_keeps_their_life() {
    let mut g = tbl();
    main1(&mut g);
    person(&mut g, |r| if r.prompt.contains("pay 3 life") { Some(1) } else { None });
    let c = meadow_hand(&mut g);
    let life = g.player(P0).life;
    let ways = sim_core::human::alt::hand_alts(&mut g, P0, c).unwrap();
    let (_, o) = ways.into_iter().find(|(l, _)| l.contains("as a land")).unwrap();
    assert!(sim_core::human::alt::run(&mut g, P0, c, &o).unwrap().is_none());
    let l = *g.player(P0).lands.last().unwrap();
    assert!(g.perm(l).tapped);
    assert_eq!(g.player(P0).life, life);
}

#[test]
fn a_person_can_play_the_meadow_with_other_lands_in_hand_and_four_lands_out() {
    // the AI's heuristics (fewer than 4 lands, no other land in hand) do not bind a person
    let mut g = tbl();
    main1(&mut g);
    person(&mut g, |_| Some(1));
    lands(&mut g, P0, "Plains", 5, false);
    hand(&mut g, P0, &["Plains"]);
    let c = meadow_hand(&mut g);
    assert!(hand_options(&mut g, c, Some(false)).len() == 1);
}

#[test]
fn the_meadow_is_not_offered_after_the_land_drop_or_off_turn() {
    let mut g = tbl();
    main1(&mut g);
    person(&mut g, |_| Some(1));
    let c = meadow_hand(&mut g);
    assert_eq!(hand_options(&mut g, c, Some(false)).len(), 1);
    assert!(hand_options(&mut g, c, None).is_empty(), "not at instant speed");
    g.player_mut(P0).lands_played = 1;
    assert!(hand_options(&mut g, c, Some(false)).is_empty(), "the land drop is spent");
    g.player_mut(P0).lands_played = 0;
    g.active = Some(P1);
    assert!(hand_options(&mut g, c, Some(false)).is_empty(), "not on an opponent's turn");
}

#[test]
fn the_database_has_gandalf_the_white_and_sunpearl_kirin() {
    let g = tbl();
    for name in ["Gandalf the White", "Sunpearl Kirin"] {
        let id = card(&g, name);
        assert_eq!(&*g.db.get(id).name, name);
    }
}
