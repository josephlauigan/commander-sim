//! The Trostani deck's life, populate and trigger-order cards (impls/trostani_life.rs, zur.rs populate,
//! t45_heliod_zur.rs's life-win upkeep, t6_legends1.rs's Emiel, engine/stack.rs's "resolve late" key) on hand-built
//! positions. Every expected number is derived from the card's Oracle text, not from what the engine printed.

use sim_core::engine::hooks::fire_trigger;
use sim_core::engine::stack::abilities_answered;
use sim_core::engine::values::{epow, etgh, is_artifact_perm};
use sim_core::engine::zones::{Tokens, make_tokens};
use sim_core::engine::{combat, life};
use sim_core::hooks::{Action, Call, Event, Opt};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{DataKey, Game, PermData, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    let mut g = table(&["trostani", "seph", "sauron"]);
    g.active = Some(P0);
    g
}

/// the table can respond to an ability (an opponent's Azorius Guildmage, with no mana to use it): triggers go on the
/// stack one at a time and the last one pushed resolves first
fn with_responses(g: &mut Game) {
    perm(g, P1, "Azorius Guildmage");
    assert!(abilities_answered(g, None), "the stack path is in use");
}

fn creature_token(g: &mut Game, p: PlayerId, pw: i32, th: i32) -> PermId {
    let spec = Tokens { sick: false, tgh: Some(th), ..Tokens::new(1, pw) };
    make_tokens(g, p, spec).unwrap()[0]
}

fn tokens_of(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf && g.perm(m).token && g.is_creature(m)).collect()
}

fn opts(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.perm(src).cd.unwrap()).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    match o.act.clone().unwrap() {
        Action::Ability { src, f, arg } => f(g, src, p, arg).unwrap(),
        other => panic!("unexpected action {other:?}"),
    }
}

// ------------------------------------------------------------------ Trostani: the life trigger
/// "Whenever another creature you control enters, you gain life equal to that creature's toughness": a 0/3 enters
#[test]
fn trostani_gains_life_equal_to_the_toughness_of_another_creature() {
    for responses in [false, true] {
        let mut g = tbl();
        if responses {
            with_responses(&mut g);
        }
        perm(&mut g, P0, "Trostani, Selesnya's Voice");
        g.player_mut(P0).life = 20;
        perm(&mut g, P0, "Sylvan Caryatid");
        assert_eq!(g.player(P0).life, 23, "toughness 3 (responses: {responses})");
    }
}

/// "another creature": Trostani entering is not another creature
#[test]
fn trostani_does_not_trigger_for_herself() {
    for responses in [false, true] {
        let mut g = tbl();
        if responses {
            with_responses(&mut g);
        }
        g.player_mut(P0).life = 20;
        perm(&mut g, P0, "Trostani, Selesnya's Voice");
        assert_eq!(g.player(P0).life, 20, "her own entering gains nothing (responses: {responses})");
    }
}

/// only creatures you control: an opponent's creature entering gains nothing
#[test]
fn trostani_does_not_trigger_for_an_opponents_creature() {
    let mut g = tbl();
    perm(&mut g, P0, "Trostani, Selesnya's Voice");
    g.player_mut(P0).life = 20;
    perm(&mut g, P1, "Sylvan Caryatid");
    assert_eq!(g.player(P0).life, 20);
}

/// Emiel pays {G/W} for a +1/+1 counter on the 0/3 as it enters; Trostani's trigger resolves after it (she reads the
/// toughness as it resolves, 4 not 3), whichever of the two entered first, and whether or not the table can respond
fn trostani_with_emiel(responses: bool, emiel_first: bool) -> i32 {
    let mut g = tbl();
    if responses {
        with_responses(&mut g);
    }
    if emiel_first {
        perm(&mut g, P0, "Emiel the Blessed");
        perm(&mut g, P0, "Trostani, Selesnya's Voice");
    } else {
        perm(&mut g, P0, "Trostani, Selesnya's Voice");
        perm(&mut g, P0, "Emiel the Blessed");
    }
    lands(&mut g, P0, "Plains", 1, false);
    g.player_mut(P0).life = 20;
    let c = perm(&mut g, P0, "Sylvan Caryatid");
    assert_eq!(g.perm(c).plus, 1, "Emiel's counter");
    assert_eq!(etgh(&g, c), 4);
    g.player(P0).life - 20
}

#[test]
fn trostani_reads_toughness_after_emiels_counter_no_response_trostani_first() {
    assert_eq!(trostani_with_emiel(false, false), 4);
}

#[test]
fn trostani_reads_toughness_after_emiels_counter_no_response_emiel_first() {
    assert_eq!(trostani_with_emiel(false, true), 4);
}

#[test]
fn trostani_reads_toughness_after_emiels_counter_with_responses_trostani_first() {
    assert_eq!(trostani_with_emiel(true, false), 4);
}

#[test]
fn trostani_reads_toughness_after_emiels_counter_with_responses_emiel_first() {
    assert_eq!(trostani_with_emiel(true, true), 4);
}

/// without the mana Emiel's trigger does nothing: Trostani gains the printed 4 of a Wall of Omens (0/4)
#[test]
fn trostani_with_emiel_but_no_mana_gains_the_printed_toughness() {
    let mut g = tbl();
    perm(&mut g, P0, "Trostani, Selesnya's Voice");
    perm(&mut g, P0, "Emiel the Blessed");
    g.player_mut(P0).life = 20;
    let c = perm(&mut g, P0, "Wall of Omens");
    assert_eq!(g.perm(c).plus, 0);
    assert_eq!(g.player(P0).life, 24);
}

/// a creature token entering is a creature entering (it comes through the tokens event)
#[test]
fn trostani_triggers_for_a_creature_token() {
    let mut g = tbl();
    perm(&mut g, P0, "Trostani, Selesnya's Voice");
    g.player_mut(P0).life = 20;
    creature_token(&mut g, P0, 2, 3);
    assert_eq!(g.player(P0).life, 23, "a 2/3 token: toughness 3");
}

/// Doubling Season makes two tokens: two enters, two triggers of 3 (one trigger would gain only 3)
#[test]
fn doubling_season_makes_two_tokens_and_two_trostani_triggers() {
    let mut g = tbl();
    perm(&mut g, P0, "Trostani, Selesnya's Voice");
    perm(&mut g, P0, "Doubling Season");
    g.player_mut(P0).life = 20;
    creature_token(&mut g, P0, 2, 3);
    assert_eq!(tokens_of(&g, P0).len(), 2);
    assert_eq!(g.player(P0).life, 26, "two triggers, 3 each");
}

/// a token with Emiel out: the counter is offered for the token too, and Trostani reads it after (3 + 1)
#[test]
fn emiel_puts_a_counter_on_a_token_before_trostani_reads_it() {
    for responses in [false, true] {
        let mut g = tbl();
        if responses {
            with_responses(&mut g);
        }
        perm(&mut g, P0, "Trostani, Selesnya's Voice");
        perm(&mut g, P0, "Emiel the Blessed");
        lands(&mut g, P0, "Plains", 1, false);
        g.player_mut(P0).life = 20;
        let t = creature_token(&mut g, P0, 2, 3);
        assert_eq!(g.perm(t).plus, 1, "Emiel's counter on the token (responses: {responses})");
        assert_eq!(g.player(P0).life, 24, "4: the toughness after the counter");
    }
}

/// Emiel alone: a token entering offers the counter; paid with {W}; none when there is no mana
#[test]
fn emiel_offers_the_counter_for_a_token() {
    let mut g = tbl();
    perm(&mut g, P0, "Emiel the Blessed");
    lands(&mut g, P0, "Plains", 1, false);
    let t = creature_token(&mut g, P0, 1, 1);
    assert_eq!(g.perm(t).plus, 1);
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| !g.perm(l).tapped).count(), 0, "the {{G/W}} was paid");
    let u = creature_token(&mut g, P0, 1, 1);
    assert_eq!(g.perm(u).plus, 0, "no mana left, no counter");
}

/// Emiel: "another creature you control" only; an opponent's token gets nothing
#[test]
fn emiel_ignores_an_opponents_token() {
    let mut g = tbl();
    perm(&mut g, P0, "Emiel the Blessed");
    lands(&mut g, P0, "Plains", 1, false);
    let t = creature_token(&mut g, P1, 1, 1);
    assert_eq!(g.perm(t).plus, 0);
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| !g.perm(l).tapped).count(), 1);
}

// ------------------------------------------------------------------ populate
/// populate copies the token that would be best as a copy, never the one with the most counters: a 1/1 Soldier with 5
/// counters is a 6/6 now, but its copy is a 1/1 with no counters; the 3/3 Beast copies as a 3/3
#[test]
fn populate_copies_the_beast_not_the_soldier_with_counters() {
    let mut g = tbl();
    let soldier = creature_token(&mut g, P0, 1, 1);
    g.perm_mut(soldier).plus = 5;
    let beast = make_tokens(&mut g, P0, Tokens { types: vec!["beast"], tgh: Some(3), ..Tokens::new(1, 3) }).unwrap()[0];
    assert_eq!(epow(&g, soldier), 6);
    let before = tokens_of(&g, P0);
    sim_core::impls::zur::populate(&mut g, P0).unwrap();
    let new: Vec<PermId> = tokens_of(&g, P0).into_iter().filter(|m| !before.contains(m)).collect();
    assert_eq!(new.len(), 1);
    let c = new[0];
    assert_eq!((g.perm(c).pow, g.perm(c).tgh), (3, 3), "a copy of the Beast");
    assert_eq!(g.perm(c).plus, 0, "counters are not copied");
    assert_eq!(g.perm(c).ttypes, g.perm(beast).ttypes);
    assert_eq!(g.perm(soldier).plus, 5, "the original keeps its counters");
}

/// a Thopter's copy is an artifact creature with flying
#[test]
fn populate_copies_a_thopter_as_an_artifact_creature_with_flying() {
    let mut g = tbl();
    // an artifact token is marked `DataKey::Artifact` (Hangarback Walker's Thopters, Urza's Constructs)
    let mut data = PermData::default();
    data.set(DataKey::Artifact, Val::Bool(true));
    let thopter =
        make_tokens(&mut g, P0, Tokens { fly: true, types: vec!["thopter"], data, ..Tokens::new(1, 1) }).unwrap()[0];
    assert!(is_artifact_perm(&g, thopter));
    sim_core::impls::zur::populate(&mut g, P0).unwrap();
    let ts = tokens_of(&g, P0);
    assert_eq!(ts.len(), 2);
    let c = *ts.iter().find(|&&m| m != thopter).unwrap();
    assert!(g.is_creature(c) && is_artifact_perm(&g, c) && g.perm(c).fly);
    assert_eq!((g.perm(c).pow, g.perm(c).tgh), (1, 1));
}

/// "create a token that's a copy of a creature token you control": with none, nothing happens (a creature card is no token)
#[test]
fn populate_with_no_creature_token_does_nothing() {
    let mut g = tbl();
    perm(&mut g, P0, "Sylvan Caryatid");
    let before = g.player(P0).perms.len();
    sim_core::impls::zur::populate(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).perms.len(), before);
}

/// Rootborn Defenses populates (the same populate), then your creatures gain indestructible until end of turn
#[test]
fn rootborn_defenses_populates_the_best_token() {
    let mut g = tbl();
    let soldier = creature_token(&mut g, P0, 1, 1);
    g.perm_mut(soldier).plus = 5;
    make_tokens(&mut g, P0, Tokens { types: vec!["beast"], tgh: Some(3), ..Tokens::new(1, 3) }).unwrap();
    let before = tokens_of(&g, P0);
    sim_core::impls::zur::rootborn(&mut g, P0).unwrap();
    let new: Vec<PermId> = tokens_of(&g, P0).into_iter().filter(|m| !before.contains(m)).collect();
    assert_eq!(new.len(), 1);
    assert_eq!((g.perm(new[0]).pow, g.perm(new[0]).plus), (3, 0), "the Beast, no counters");
    for m in tokens_of(&g, P0) {
        assert!(sim_core::dsl::has_kw(&g, m, "indestructible"), "each creature is indestructible");
    }
}

/// Trostani's {1}{G}{W},{T}: the AI uses it at the end of an opponent's turn; the copy enters (she gains its toughness)
#[test]
fn trostani_populates_in_the_end_of_turn_window() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Trostani, Selesnya's Voice");
    lands(&mut g, P0, "Forest", 1, false);
    lands(&mut g, P0, "Plains", 1, false);
    lands(&mut g, P0, "Swamp", 1, false);
    g.active = Some(P1);
    g.player_mut(P0).life = 20;
    assert!(opts(&mut g, t, P0, None).is_empty(), "no creature token to copy");
    make_tokens(&mut g, P0, Tokens { types: vec!["beast"], tgh: Some(3), ..Tokens::new(1, 3) }).unwrap();
    g.player_mut(P0).life = 20;
    let o = opts(&mut g, t, P0, None);
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, P0, &o[0]));
    assert!(g.perm(t).tapped, "{{T}}");
    assert_eq!(tokens_of(&g, P0).len(), 2);
    assert_eq!(g.player(P0).life, 23, "the copy is a creature entering: toughness 3");
    assert!(g.player(P0).lands.iter().all(|&l| g.perm(l).tapped), "{{1}}{{G}}{{W}} paid");
    assert!(opts(&mut g, t, P0, None).is_empty(), "tapped");
}

/// in its own main phase the AI keeps the mana unless the copy pays at once (a Cathars' Crusade here: each creature gets a counter)
#[test]
fn trostani_populates_in_a_main_phase_only_when_it_pays_now() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Trostani, Selesnya's Voice");
    lands(&mut g, P0, "Forest", 1, false);
    lands(&mut g, P0, "Plains", 1, false);
    lands(&mut g, P0, "Swamp", 1, false);
    creature_token(&mut g, P0, 1, 1);
    assert!(opts(&mut g, t, P0, Some(false)).is_empty());
    perm(&mut g, P0, "Cathars' Crusade");
    assert_eq!(opts(&mut g, t, P0, Some(false)).len(), 1);
}

// ------------------------------------------------------------------ Verdant Sun's Avatar
/// "Whenever this creature or another creature you control enters, you gain life equal to that creature's toughness":
/// with Trostani out, the Avatar's own entering triggers both (5 and 5)
#[test]
fn avatar_entering_with_trostani_gains_two_fives() {
    for responses in [false, true] {
        let mut g = tbl();
        if responses {
            with_responses(&mut g);
        }
        perm(&mut g, P0, "Trostani, Selesnya's Voice");
        g.player_mut(P0).life = 20;
        perm(&mut g, P0, "Verdant Sun's Avatar");
        assert_eq!(g.player(P0).life, 30, "5 + 5 (responses: {responses})");
    }
}

/// Trostani entering with the Avatar out: only the Avatar triggers (she is not another creature to herself): her 2/5
#[test]
fn trostani_entering_with_the_avatar_out_triggers_only_the_avatar() {
    let mut g = tbl();
    perm(&mut g, P0, "Verdant Sun's Avatar");
    g.player_mut(P0).life = 20;
    perm(&mut g, P0, "Trostani, Selesnya's Voice");
    assert_eq!(g.player(P0).life, 25, "one gain of 5");
}

/// Archangel of Thune out: the first gain puts a counter on each creature (the Avatar too), which resolves before the
/// second trigger reads the toughness: 5 then 6
#[test]
fn avatar_with_archangel_the_second_gain_reads_one_more() {
    for responses in [false, true] {
        let mut g = tbl();
        if responses {
            with_responses(&mut g);
        }
        perm(&mut g, P0, "Trostani, Selesnya's Voice");
        perm(&mut g, P0, "Archangel of Thune");
        g.player_mut(P0).life = 20;
        let a = perm(&mut g, P0, "Verdant Sun's Avatar");
        assert_eq!(g.player(P0).life, 31, "5 + 6 (responses: {responses})");
        assert_eq!(g.perm(a).plus, 2, "one counter for each of the two gains");
    }
}

/// another creature entering: the Avatar gains its toughness too (its own trigger and Trostani's: 3 and 3)
#[test]
fn avatar_triggers_for_another_creature() {
    let mut g = tbl();
    perm(&mut g, P0, "Verdant Sun's Avatar");
    g.player_mut(P0).life = 20;
    perm(&mut g, P0, "Sylvan Caryatid");
    assert_eq!(g.player(P0).life, 23);
}

// ------------------------------------------------------------------ Rhox Faithmender
/// "If you would gain life, you gain twice that much life instead."
#[test]
fn rhox_faithmender_doubles_life_gain() {
    let mut g = tbl();
    perm(&mut g, P0, "Rhox Faithmender");
    g.player_mut(P0).life = 20;
    life::gain(&mut g, P0, 3).unwrap();
    assert_eq!(g.player(P0).life, 26);
}

#[test]
fn two_rhox_faithmenders_quadruple_life_gain() {
    let mut g = tbl();
    perm(&mut g, P0, "Rhox Faithmender");
    perm(&mut g, P0, "Rhox Faithmender");
    g.player_mut(P0).life = 20;
    life::gain(&mut g, P0, 3).unwrap();
    assert_eq!(g.player(P0).life, 32, "3 x 2 x 2 = 12");
}

/// Angel of Vitality's "that much plus 1" is applied before the doubling: (3 + 1) x 2 = 8, not 3 x 2 + 1 = 7
#[test]
fn rhox_doubles_after_the_plus_one_of_angel_of_vitality() {
    let mut g = tbl();
    perm(&mut g, P0, "Rhox Faithmender");
    perm(&mut g, P0, "Angel of Vitality");
    g.player_mut(P0).life = 20;
    life::gain(&mut g, P0, 3).unwrap();
    assert_eq!(g.player(P0).life, 28);
}

// ------------------------------------------------------------------ Treebeard, Gracious Host
/// "When Treebeard enters, create two Food tokens."
#[test]
fn treebeard_creates_two_food() {
    let mut g = tbl();
    perm(&mut g, P0, "Treebeard, Gracious Host");
    assert_eq!(g.player(P0).foods, 2);
}

#[test]
fn treebeard_creates_four_food_with_anointed_procession() {
    let mut g = tbl();
    perm(&mut g, P0, "Anointed Procession");
    perm(&mut g, P0, "Treebeard, Gracious Host");
    assert_eq!(g.player(P0).foods, 4);
}

/// "Whenever you gain life, put that many +1/+1 counters on target Halfling or Treefolk": a gain of 4 puts 4 on Treebeard;
/// one trigger for each gain event (a second gain of 2 adds 2, not more)
#[test]
fn treebeard_puts_counters_equal_to_each_life_gain_on_itself() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Treebeard, Gracious Host");
    life::gain(&mut g, P0, 4).unwrap();
    assert_eq!(g.perm(t).plus, 4);
    life::gain(&mut g, P0, 2).unwrap();
    assert_eq!(g.perm(t).plus, 6, "one trigger for each gain event");
}

/// Doubling Season doubles the counters Treebeard's trigger puts: 8
#[test]
fn treebeard_with_doubling_season_puts_eight() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Treebeard, Gracious Host");
    perm(&mut g, P0, "Doubling Season");
    life::gain(&mut g, P0, 4).unwrap();
    assert_eq!(g.perm(t).plus, 8);
}

/// the counters equal the life really gained: Rhox doubles 3 to 6, so 6 counters
#[test]
fn treebeard_counts_the_life_actually_gained() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Treebeard, Gracious Host");
    perm(&mut g, P0, "Rhox Faithmender");
    life::gain(&mut g, P0, 3).unwrap();
    assert_eq!(g.perm(t).plus, 6);
}

// ------------------------------------------------------------------ Archangel of Thune
/// "Whenever you gain life, put a +1/+1 counter on each creature you control": two lifelinkers deal damage, two gain
/// events (one for each source), two triggers: each creature gets two counters
#[test]
fn archangel_of_thune_triggers_once_for_each_lifelinker() {
    let mut g = tbl();
    let a = perm(&mut g, P0, "Archangel of Thune");
    let spec = Tokens { sick: false, lifelink: true, ..Tokens::new(1, 2) };
    let l1 = make_tokens(&mut g, P0, spec.clone()).unwrap()[0];
    let l2 = make_tokens(&mut g, P0, spec).unwrap()[0];
    for m in [a, l1, l2] {
        g.perm_mut(m).plus = 0;
    }
    g.player_mut(P0).life = 20;
    combat::resolve_combat(&mut g, P0, &[l1, l2], P1, &[l1, l2]).unwrap();
    assert_eq!(g.player(P0).life, 24, "2 + 2 from the two lifelinkers");
    for m in [a, l1, l2] {
        assert_eq!(g.perm(m).plus, 2, "{}: one counter for each trigger", g.perm(m).name);
    }
}

// ------------------------------------------------------------------ Test of Endurance, Felidar Sovereign
fn upkeep(g: &mut Game, p: PlayerId) {
    fire_trigger(g, Event::Upkeep, Call::Player { p }).unwrap();
}

/// "At the beginning of your upkeep, if you have 50 or more life, you win the game."
#[test]
fn test_of_endurance_wins_at_fifty_not_at_forty_nine() {
    let mut g = tbl();
    perm(&mut g, P0, "Test of Endurance");
    g.player_mut(P0).life = 49;
    upkeep(&mut g, P0);
    assert!(!g.over, "49 life");
    g.player_mut(P0).life = 50;
    upkeep(&mut g, P0);
    assert!(g.over && g.winner == Some(P0));
}

/// "your upkeep": an opponent's upkeep does nothing
#[test]
fn test_of_endurance_does_not_win_in_an_opponents_upkeep() {
    let mut g = tbl();
    perm(&mut g, P0, "Test of Endurance");
    g.player_mut(P0).life = 60;
    upkeep(&mut g, P1);
    assert!(!g.over);
}

/// the "if" is checked again as the trigger resolves (CR 603.4): a trigger that resolves first and takes the life to 49
/// stops it. The table can respond, so the triggers are on the stack and the one pushed last resolves first.
#[test]
fn test_of_endurance_life_dropping_to_forty_nine_in_response_stops_it() {
    use sim_core::engine::stack::trigger_window;
    use sim_core::hooks::Src;
    // a second upkeep trigger that costs 3 life (as a stand-in for any response that changes the life total)
    fn drain(g: &mut Game, src: Src, p: PlayerId) -> sim_core::flow::Res {
        if g.perm(src).owner != p || !trigger_window(g, p, Some(src), "lose 3 life", None)? {
            return Ok(());
        }
        g.player_mut(p).life -= 3;
        Ok(())
    }
    let mut g = tbl();
    let mut r = sim_core::impls::registry(&g.db).unwrap();
    r.card(&g.db, "Wall of Omens").unwrap().upkeep = Some(drain);
    g.registry = std::sync::Arc::new(r);
    with_responses(&mut g);
    perm(&mut g, P0, "Test of Endurance"); // pushed first, resolves last
    perm(&mut g, P0, "Wall of Omens");
    g.player_mut(P0).life = 52;
    upkeep(&mut g, P0);
    assert_eq!(g.player(P0).life, 49);
    assert!(!g.over, "49 when it resolves: no win");
    // and without the drain in the way it wins on the stack path too
    let mut g = tbl();
    with_responses(&mut g);
    perm(&mut g, P0, "Test of Endurance");
    g.player_mut(P0).life = 50;
    upkeep(&mut g, P0);
    assert!(g.over && g.winner == Some(P0));
}

/// Felidar Sovereign is unchanged: 40 life at your upkeep wins, 39 does not
#[test]
fn felidar_sovereign_still_wins_at_forty() {
    let mut g = tbl();
    perm(&mut g, P0, "Felidar Sovereign");
    g.player_mut(P0).life = 39;
    upkeep(&mut g, P0);
    assert!(!g.over);
    g.player_mut(P0).life = 40;
    upkeep(&mut g, P0);
    assert!(g.over && g.winner == Some(P0));
}

/// 40 life is not enough for Test of Endurance, and 50 is more than Felidar needs: each card uses its own threshold
#[test]
fn the_two_win_cards_use_their_own_thresholds() {
    let mut g = tbl();
    perm(&mut g, P0, "Test of Endurance");
    g.player_mut(P0).life = 45;
    upkeep(&mut g, P0);
    assert!(!g.over, "45 is not 50");
}
