//! Trostani deck: Felidar Retreat's AI. Oracle text: {3}{W} Enchantment. "Landfall -- Whenever a land you control enters,
//! choose one -- Create a 2/2 white Cat Beast creature token. -- Put a +1/+1 counter on each creature you control.
//! Those creatures gain vigilance until end of turn."
//!
//! Before the fix the card was tagged only `landfall2`, which the derived cast priority reads as nothing: its priority
//! was 0 in every position, so the AI never cast it ("cast 0.0% of the games it was drawn" in the t3 and t4 baselines),
//! and the AI's landfall always made the Cat. The numbers below come from the Oracle text: a land entering makes one Cat
//! (two with Doubling Season or Anointed Procession, which double tokens) or one counter on each creature (two each with
//! Doubling Season, which doubles counters). Card code: impls/trostani_misc.rs; the landfall site: engine/zones.rs.

mod play_common;
use play_common::*;
use sim_core::ai;
use sim_core::engine::zones::landfall;
use sim_core::state::Step;
use std::sync::Arc;

const DECK: &str = "trostani";
const RETREAT: &str = "Felidar Retreat";

fn tbl() -> Game {
    let mut g = table(&[DECK, "seph"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g
}

fn prio(g: &Game, c: sim_core::ids::CardId) -> i32 {
    ai::pool::generic_prio(g, P0, c)
}

/// Retreat in hand with four Plains out (so it is payable) and `n_lands` lands in hand
fn hand_with_lands(g: &mut Game, n_lands: usize) -> sim_core::ids::CardId {
    lands(g, P0, "Plains", 4, false);
    let c = hand(g, P0, &[RETREAT])[0];
    for _ in 0..n_lands {
        hand(g, P0, &["Forest"]);
    }
    c
}

fn tokens_named(g: &Game, p: PlayerId, ty: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).token && g.perm(m).ttypes.contains(&ty)).collect()
}

/// creatures that are not tokens made by the landfall: n 1/1 stand-ins (vanilla tokens of power 1)
fn creatures(g: &mut Game, n: usize) -> Vec<PermId> {
    (0..n).map(|_| token(g, P0, 1)).collect()
}

fn retreat_option(g: &mut Game) -> Option<sim_core::hooks::Opt> {
    ai::brain::main_options(g, P0, false).unwrap().into_iter().find(|o| o.label.contains(RETREAT))
}

// ================================================================== the cast rule

#[test]
fn never_cast_reproduction_a_land_in_hand_gives_a_priority() {
    // four Plains for {3}{W}, a Forest waiting in hand: the next land drop is a landfall. On the unmodified engine the
    // priority is 0 (derived worth 0) and the main-phase options hold no Retreat at all.
    let mut g = tbl();
    let c = hand_with_lands(&mut g, 1);
    assert!(prio(&g, c) > 0, "a land to play and an enchantment that rewards it: priority above 0");
    assert!(retreat_option(&mut g).is_some(), "the AI's main-phase options offer the cast");
}

#[test]
fn the_ai_casts_retreat_before_passing_with_a_land_in_hand() {
    // the cast beats "stop": the AI spends its turn's mana on the enchantment
    let mut g = tbl();
    hand_with_lands(&mut g, 2);
    let opts = ai::brain::main_options(&mut g, P0, false).unwrap();
    let best = opts.iter().max_by(|a, b| a.utility.total_cmp(&b.utility)).unwrap();
    assert!(best.label.contains(RETREAT), "best option is the cast, not {:?}", best.label);
    let stop = opts.iter().find(|o| o.label == "stop").map_or(f64::MIN, |o| o.utility);
    assert!(retreat_option(&mut g).unwrap().utility > stop);
}

#[test]
fn no_land_in_hand_and_none_in_the_library_means_no_cast() {
    // landfall has nothing to trigger on: not cast (and the empty library gives no land drop to expect)
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 4, false);
    let c = hand(&mut g, P0, &[RETREAT])[0];
    g.player_mut(P0).library.clear();
    assert_eq!(prio(&g, c), 0);
    assert!(retreat_option(&mut g).is_none());
}

#[test]
fn an_empty_hand_is_still_cast_while_the_library_may_give_lands() {
    // a land drop next turn is likely when a fifth or more of the library is land; Trostani's deck runs 36
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 4, false);
    let c = hand(&mut g, P0, &[RETREAT])[0];
    let lib = &g.player(P0).library;
    let share = lib.iter().filter(|&&x| g.db.get(x).land).count() * 5 >= lib.len();
    assert!(share, "the deck's library holds a fifth of lands or more");
    let p = prio(&g, c);
    assert!(p > 0, "worth casting for the land drops to come");
    // ...but below the case with a land waiting in hand
    hand(&mut g, P0, &["Forest"]);
    assert!(prio(&g, c) > p, "a land in hand is worth more than the chance of one");
}

#[test]
fn more_lands_waiting_creatures_and_engines_raise_the_priority() {
    // each case against the same bare position: four Plains, Retreat and one Forest in hand
    let bare = {
        let mut g = tbl();
        let c = hand_with_lands(&mut g, 1);
        prio(&g, c)
    };
    let with = |setup: &dyn Fn(&mut Game)| {
        let mut g = tbl();
        let c = hand_with_lands(&mut g, 1);
        setup(&mut g);
        prio(&g, c)
    };
    assert!(
        with(&|g| hand(g, P0, &["Forest", "Forest"]).iter().for_each(|_| ())) > bare,
        "three lands waiting beat one"
    );
    assert!(with(&|g| creatures(g, 3).iter().for_each(|_| ())) > bare, "creatures to put counters on");
    assert!(with(&|g| drop(perm(g, P0, "Doubling Season"))) > bare, "Doubling Season doubles both modes");
    assert!(with(&|g| drop(perm(g, P0, "Trostani, Selesnya's Voice"))) > bare, "Trostani gains life for each Cat");
    assert!(with(&|g| drop(perm(g, P0, "Anointed Procession"))) > bare, "Procession doubles the Cat");
    // and it stays below the multipliers' own priority (60) however good the position
    let mut g = tbl();
    let c = hand_with_lands(&mut g, 3);
    creatures(&mut g, 4);
    perm(&mut g, P0, "Doubling Season");
    assert!(prio(&g, c) < 60, "capped below the plan's multipliers: {}", prio(&g, c));
}

#[test]
fn an_extra_land_drop_counts_when_no_land_is_in_hand() {
    // Exploration: an extra land drop each turn. With a library of lands it is cast and ranks above the library's chance
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 4, false);
    let c = hand(&mut g, P0, &[RETREAT])[0];
    let plain = prio(&g, c);
    perm(&mut g, P0, "Exploration");
    assert!(prio(&g, c) > plain, "an extra land drop to come");
}

// ================================================================== the mode choice (the AI's landfall)

fn retreat_out(g: &mut Game) {
    perm(g, P0, RETREAT);
}

#[test]
fn with_no_creature_the_ai_makes_a_cat_beast() {
    // counters on nothing is nothing: the only useful mode is the Cat. 2/2, white, Cat Beast (Oracle)
    let mut g = tbl();
    retreat_out(&mut g);
    landfall(&mut g, P0).unwrap();
    let cats = tokens_named(&g, P0, "cat");
    assert_eq!(cats.len(), 1);
    assert_eq!((g.perm(cats[0]).pow, g.perm(cats[0]).tgh), (2, 2));
    assert!(tokens_named(&g, P0, "beast").contains(&cats[0]), "a Cat Beast");
    assert!(g.perm(cats[0]).colors.has('W'), "white");
}

#[test]
fn with_three_creatures_the_ai_still_makes_the_cat() {
    // In counter-units a 2/2 is two counters, plus one for being a body (Craterhoof, blink and populate want bodies):
    // the Cat is worth 3. Three creatures give three counters: a tie, and a tie is the Cat.
    let mut g = tbl();
    retreat_out(&mut g);
    let cs = creatures(&mut g, 3);
    landfall(&mut g, P0).unwrap();
    assert_eq!(tokens_named(&g, P0, "cat").len(), 1);
    assert!(cs.iter().all(|&m| g.perm(m).plus == 0), "no counters");
}

#[test]
fn with_four_creatures_the_ai_puts_a_counter_on_each_with_vigilance() {
    // four counters against one Cat worth three
    let mut g = tbl();
    retreat_out(&mut g);
    let cs = creatures(&mut g, 4);
    landfall(&mut g, P0).unwrap();
    assert!(tokens_named(&g, P0, "cat").is_empty(), "no Cat: the counters mode");
    for &m in &cs {
        assert_eq!(g.perm(m).plus, 1, "one +1/+1 counter on each creature");
        assert!(g.perm(m).eot_kw.contains(&"vigilance"), "and vigilance until end of turn");
    }
}

#[test]
fn doubling_season_doubles_both_modes() {
    // Season doubles tokens and counters. Two creatures: two counters on each (four) against two Cats (4 + 1 each side
    // of the body... the Cats are worth 2 x 2 + 1 = 5): Cats. Three creatures: six against five: counters, two each.
    let mut g = tbl();
    retreat_out(&mut g);
    let cs = creatures(&mut g, 2); // made first: Season would double the stand-ins too
    perm(&mut g, P0, "Doubling Season");
    landfall(&mut g, P0).unwrap();
    assert_eq!(tokens_named(&g, P0, "cat").len(), 2, "Season doubles the Cat");
    assert!(cs.iter().all(|&m| g.perm(m).plus == 0));

    let mut g = tbl();
    retreat_out(&mut g);
    let cs = creatures(&mut g, 3);
    perm(&mut g, P0, "Doubling Season");
    landfall(&mut g, P0).unwrap();
    assert!(tokens_named(&g, P0, "cat").is_empty());
    assert!(cs.iter().all(|&m| g.perm(m).plus == 2), "Season doubles the one counter to two on each");
}

#[test]
fn anointed_procession_doubles_only_the_cat_so_it_takes_more_creatures_to_prefer_counters() {
    // Procession doubles tokens, not counters: two Cats (worth 5) against one counter on each creature. Five
    // creatures: a tie, the Cats (two of them); six creatures: the counters.
    let mut g = tbl();
    retreat_out(&mut g);
    creatures(&mut g, 5);
    perm(&mut g, P0, "Anointed Procession");
    landfall(&mut g, P0).unwrap();
    assert_eq!(tokens_named(&g, P0, "cat").len(), 2, "two Cats");
    let mut g = tbl();
    retreat_out(&mut g);
    let cs = creatures(&mut g, 6);
    perm(&mut g, P0, "Anointed Procession");
    landfall(&mut g, P0).unwrap();
    assert!(tokens_named(&g, P0, "cat").is_empty());
    assert!(cs.iter().all(|&m| g.perm(m).plus == 1));
}

#[test]
fn wildwood_scourge_makes_the_counters_worth_more() {
    // Wildwood Scourge: whenever +1/+1 counters are put on another non-Hydra creature you control, it gets one too. Three
    // creatures are a tie (the Cat); three that include the Scourge are four counters' worth: the counters.
    let mut g = tbl();
    retreat_out(&mut g);
    // a Scourge cast for X = 1 (it enters with one counter; with X = 0 it would die as a 0/0)
    let c = take(&mut g, P0, "Wildwood Scourge");
    let sc = sim_core::engine::zones::enter(
        &mut g,
        P0,
        c,
        sim_core::engine::zones::Enter { x: 1, was_cast: true, sick: false, ..Default::default() },
    )
    .unwrap();
    let cs = creatures(&mut g, 2);
    landfall(&mut g, P0).unwrap();
    assert!(tokens_named(&g, P0, "cat").is_empty(), "the counters mode");
    assert!(cs.iter().all(|&m| g.perm(m).plus == 1));
    assert_eq!(g.perm(sc).plus, 4, "its X counter, one from Retreat, and one grown for each of the two others");
}

#[test]
fn cathars_crusade_makes_the_cat_better() {
    // Crusade: whenever a creature you control enters, a +1/+1 counter on each creature you control. The Cat enters, so
    // the Cat mode gives counters on everything as well. Even with eight creatures it is the Cat.
    let mut g = tbl();
    retreat_out(&mut g);
    let cs = creatures(&mut g, 8);
    perm(&mut g, P0, "Cathars' Crusade");
    landfall(&mut g, P0).unwrap();
    assert_eq!(tokens_named(&g, P0, "cat").len(), 1, "the Cat");
    assert!(cs.iter().all(|&m| g.perm(m).plus >= 1), "and Crusade put a counter on each creature");
}

// ================================================================== a person's prompt is unchanged

fn person(pick: impl Fn(&Request) -> Option<usize> + Send + Sync + 'static) -> Arc<ScriptHuman> {
    Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        match pick(r) {
            Some(k) => Answer::Pick(k),
            None => Answer::Pick(0),
        }
    }))
}

#[test]
fn a_person_is_asked_the_same_prompt_and_both_modes_work() {
    for (word, counters) in [("Cat", false), ("+1/+1", true)] {
        let mut g = tbl();
        retreat_out(&mut g);
        let c = creatures(&mut g, 1)[0]; // one creature: the AI would pick the Cat, so the person's choice is theirs
        let h = person(move |r| r.choices.iter().position(|x| x.contains(word)));
        g.humans.add(P0, h.clone());
        landfall(&mut g, P0).unwrap();
        let asked: Vec<_> =
            h.asked().iter().filter(|r| r.prompt.contains("Felidar Retreat: choose a mode")).cloned().collect();
        assert_eq!(asked.len(), 1, "the person is asked once");
        assert_eq!(asked[0].choices.len(), 2);
        assert!(asked[0].choices[0].starts_with("Create a 2/2 white Cat Beast"));
        assert!(asked[0].choices[1].starts_with("Put a +1/+1 counter on each creature you control"));
        assert_eq!(g.perm(c).plus, i32::from(counters));
        assert_eq!(tokens_named(&g, P0, "cat").len(), usize::from(!counters));
    }
}
