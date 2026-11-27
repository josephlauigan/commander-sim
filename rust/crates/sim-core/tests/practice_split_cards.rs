//! Practice mode: Temple Bell, Castle Locthwain (the Sheoldred list) and Woe Strider (the Marchesa list) are offered to a
//! person, who decides when to use them, and using them has the printed effect. Found by `practice_ability_probe` after
//! the Marchesa / Sheoldred split. The AI's Temple Bell is unchanged: it only taps while a draw punisher is out.

mod play_common;
use play_common::*;
use sim_core::human::human::abilities_of;
use sim_core::state::Step;

fn main_table(key: &str) -> Game {
    let mut g = table(&[key, "seph"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g
}

fn hand_len(g: &Game, p: PlayerId) -> usize {
    g.player(p).hand.len()
}

// ------------------------------------------------------------------ Temple Bell
#[test]
fn a_person_is_offered_temple_bell_with_no_draw_punisher_out_and_each_player_draws() {
    let mut g = main_table("sheoldred");
    let bell = perm(&mut g, P0, "Temple Bell");
    seat(&mut g, P0, vec![]);
    assert_eq!(abilities_of(&mut g, P0, bell).unwrap().len(), 1, "offered whenever it is untapped");
    let (mine, theirs) = (hand_len(&g, P0), hand_len(&g, P1));
    let i = g.player(P0).perms.iter().position(|&x| x == bell).unwrap();
    main(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pick(0)]);
    assert!(g.perm(bell).tapped);
    assert_eq!(hand_len(&g, P0), mine + 1, "you draw");
    assert_eq!(hand_len(&g, P1), theirs + 1, "and so does the opponent");
}

#[test]
fn a_tapped_temple_bell_is_not_offered_to_a_person() {
    let mut g = main_table("sheoldred");
    let bell = perm(&mut g, P0, "Temple Bell");
    g.perm_mut(bell).tapped = true;
    seat(&mut g, P0, vec![]);
    assert!(abilities_of(&mut g, P0, bell).unwrap().is_empty());
}

/// the Sheoldred plan's `plays` list names the Bell (2026-11-26), so the AI taps it only at the end of an opponent's
/// turn and only while a draw punisher is out: giving a person the ability must not change that
#[test]
fn the_ai_seats_temple_bell_waits_for_a_draw_punisher() {
    let mut g = table(&["sheoldred", "seph"]);
    let bell = perm(&mut g, P0, "Temple Bell");
    let f = g.registry.get(g.perm(bell).card()).unwrap().options.unwrap();
    assert!(f(&mut g, bell, P0, None).unwrap().is_empty(), "no punisher");
    perm(&mut g, P0, "Underworld Dreams");
    assert_eq!(f(&mut g, bell, P0, None).unwrap().len(), 1, "an opponent's end step with a punisher out");
    for post in [Some(false), Some(true)] {
        assert!(f(&mut g, bell, P0, post).unwrap().is_empty(), "not in a main phase unless it kills: {post:?}");
    }
    // the same hook for the person's seat is offered, unconditionally
    seat(&mut g, P0, vec![]);
    assert_eq!(f(&mut g, bell, P0, None).unwrap().len(), 1);
}

// ------------------------------------------------------------------ Castle Locthwain
#[test]
fn a_person_uses_castle_locthwain_to_draw_and_lose_life_equal_to_the_hand() {
    let mut g = main_table("sheoldred");
    let l = lands(&mut g, P0, "Castle Locthwain", 1, false)[0];
    g.player_mut(P0).life = 40;
    g.player_mut(P0).hand.clear();
    seat(&mut g, P0, vec![]);
    assert_eq!(sim_core::human::alt::land_alts(&mut g, P0, l).unwrap().len(), 1, "offered to the person");
    pool(&mut g, P0, &[('B', 3)]);
    main(&mut g, P0, vec![act(json!({"do": "use", "land": 0})), pick(0)]);
    assert!(g.perm(l).tapped);
    assert_eq!(pool_total(&g, P0), 0, "{{1}}{{B}}{{B}} paid from your pool");
    assert_eq!(hand_len(&g, P0), 1, "drew a card");
    assert_eq!(g.player(P0).life, 39, "lost life equal to the one card in hand");
}

#[test]
fn castle_locthwain_loses_life_for_the_whole_hand() {
    let mut g = main_table("sheoldred");
    lands(&mut g, P0, "Castle Locthwain", 1, false);
    g.player_mut(P0).life = 40;
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Sol Ring", "Mind Stone"]);
    seat(&mut g, P0, vec![]);
    pool(&mut g, P0, &[('B', 3)]);
    main(&mut g, P0, vec![act(json!({"do": "use", "land": 0})), pick(0)]);
    assert_eq!(hand_len(&g, P0), 3);
    assert_eq!(g.player(P0).life, 37, "three cards in hand after the draw");
}

// ------------------------------------------------------------------ Woe Strider
/// Woe Strider's enter trigger makes a 0/1 Goat: the other creature to sacrifice
fn strider_and_goat(g: &mut Game) -> (PermId, PermId) {
    let strider = perm(g, P0, "Woe Strider");
    let goat = *g.player(P0).perms.iter().find(|&&m| m != strider && g.is_creature(m)).expect("the Goat token");
    (strider, goat)
}

#[test]
fn a_person_sacrifices_another_creature_to_woe_strider_and_scries() {
    let mut g = main_table("marchesa");
    let (strider, goat) = strider_and_goat(&mut g);
    seat(&mut g, P0, vec![]);
    assert_eq!(abilities_of(&mut g, P0, strider).unwrap().len(), 1, "listed");
    let i = g.player(P0).perms.iter().position(|&x| x == strider).unwrap();
    let top = g.player(P0).library.last().copied().unwrap();
    // the creature (the only choice), then the scry: put the card on the bottom
    let h = main(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pick(0), pick(0), pick(1)]);
    let asked = h.asked();
    assert!(
        asked.iter().any(|r| r.prompt.contains("sacrifice which creature")
            && r.choices.iter().any(|c| c.contains("Goat"))
            && !r.choices.iter().any(|c| c.contains("Woe Strider"))),
        "only the Goat is offered, not Woe Strider itself: {asked:?}"
    );
    assert!(asked.iter().any(|r| r.prompt.starts_with("Scry")), "scry 1: {asked:?}");
    assert!(!g.perm(goat).on_bf, "the other creature was sacrificed");
    assert!(g.perm(strider).on_bf, "Woe Strider stays");
    assert_eq!(g.player(P0).library.first().copied(), Some(top), "the scried card went to the bottom");
}

#[test]
fn woe_strider_cannot_sacrifice_itself() {
    let mut g = main_table("marchesa");
    let (strider, goat) = strider_and_goat(&mut g);
    sim_core::engine::zones::die(&mut g, goat, "sac").unwrap();
    seat(&mut g, P0, vec![]);
    let i = g.player(P0).perms.iter().position(|&x| x == strider).unwrap();
    let h = main(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pick(0)]);
    assert!(g.perm(strider).on_bf, "no other creature: nothing is sacrificed");
    assert!(h.asked().iter().all(|r| !r.prompt.contains("sacrifice which")), "nothing to choose");
}

// ------------------------------------------------------------------ Sign in Blood (new on the Sheoldred list)
/// a person casts Sign in Blood with {B}{B} from the pool and picks the target player
fn cast_sign_in_blood(g: &mut Game, target_seat: usize) -> Vec<String> {
    let c = hand(g, P0, &["Sign in Blood"])[0];
    let i = g.player(P0).hand.iter().position(|&x| x == c).unwrap();
    pool(g, P0, &[('B', 2)]);
    let h = main(g, P0, vec![act(json!({"do": "cast", "card": i})), pick(target_seat)]);
    h.asked().iter().map(|r| format!("{}: {:?}", r.prompt, r.choices)).collect()
}

#[test]
fn a_person_picks_the_player_who_draws_two_and_loses_two_with_sign_in_blood() {
    let mut g = main_table("sheoldred");
    g.player_mut(P0).life = 40;
    g.player_mut(P1).life = 40;
    let (mine, theirs) = (hand_len(&g, P0), hand_len(&g, P1));
    let asked = cast_sign_in_blood(&mut g, 1);
    assert!(asked.iter().any(|a| a.to_lowercase().contains("target")), "the target player is asked: {asked:?}");
    assert_eq!((hand_len(&g, P1), g.player(P1).life), (theirs + 2, 38), "the opponent drew two and lost 2");
    assert_eq!((hand_len(&g, P0), g.player(P0).life), (mine, 40), "the caster is untouched (the spell left the hand)");
}

#[test]
fn a_person_can_still_target_themselves_with_sign_in_blood() {
    let mut g = main_table("sheoldred");
    g.player_mut(P0).life = 40;
    let mine = hand_len(&g, P0);
    cast_sign_in_blood(&mut g, 0);
    assert_eq!((hand_len(&g, P0), g.player(P0).life), (mine + 2, 38), "one card cast, two drawn");
}

#[test]
fn the_ai_seat_still_targets_itself_with_sign_in_blood() {
    let mut g = main_table("sheoldred");
    g.player_mut(P0).life = 40;
    lands(&mut g, P0, "Swamp", 2, false);
    let mine = hand_len(&g, P0);
    let c = hand(&mut g, P0, &["Sign in Blood"])[0];
    // an AI seat's cast carries no target player: it draws for itself
    sim_core::engine::cast::cast_card(&mut g, P0, c, "hand", Default::default()).unwrap();
    assert_eq!((hand_len(&g, P0), g.player(P0).life), (mine + 2, 38));
}

// ------------------------------------------------------------------ Black Market Connections (new on both lists)
fn bmc_trigger(g: &mut Game, m: PermId) {
    let f = g.registry.get(g.perm(m).card()).unwrap().upkeep.unwrap();
    f(g, m, P0).unwrap();
}

#[test]
fn a_person_chooses_the_modes_of_black_market_connections() {
    let mut g = main_table("sheoldred");
    let m = perm(&mut g, P0, "Black Market Connections");
    g.player_mut(P0).life = 40;
    let cards = hand_len(&g, P0);
    let before = g.player(P0).perms.len();
    // the first mode (forced): Buy Information; then Sell Contraband: no, Hire a Mercenary: yes
    let h = seat(&mut g, P0, vec![pick(1), pick(1), pick(0)]);
    bmc_trigger(&mut g, m);
    assert!(h.asked().iter().any(|r| r.prompt.contains("choose a mode")), "the modes are asked");
    assert_eq!(hand_len(&g, P0), cards + 1, "Buy Information: a card");
    assert_eq!(g.player(P0).life, 40 - 2 - 3, "Buy Information and Hire a Mercenary");
    assert_eq!(g.player(P0).perms.len(), before + 1, "the 3/2 token only: no Treasure");
}

#[test]
fn the_ai_black_market_connections_still_takes_every_mode_it_can() {
    let mut g = main_table("sheoldred");
    let m = perm(&mut g, P0, "Black Market Connections");
    g.player_mut(P0).life = 40;
    let cards = hand_len(&g, P0);
    bmc_trigger(&mut g, m);
    assert_eq!(hand_len(&g, P0), cards + 1);
    assert_eq!(g.player(P0).life, 40 - 1 - 2 - 3, "Treasure, card and 3/2 above 20 life");
}

// ------------------------------------------------------------------ Bloodgift Demon (new on the Sheoldred list)
#[test]
fn a_person_picks_who_draws_and_loses_life_to_bloodgift_demon() {
    let mut g = main_table("sheoldred");
    let m = perm(&mut g, P0, "Bloodgift Demon");
    g.player_mut(P1).life = 40;
    let theirs = hand_len(&g, P1);
    let mine = hand_len(&g, P0);
    let f = g.registry.get(g.perm(m).card()).unwrap().upkeep.unwrap();
    let h = seat(&mut g, P0, vec![pick(1)]);
    f(&mut g, m, P0).unwrap();
    assert!(h.asked().iter().any(|r| r.prompt.starts_with("Bloodgift Demon")), "the target is asked");
    assert_eq!((hand_len(&g, P1), g.player(P1).life), (theirs + 1, 39), "the opponent drew and lost 1");
    assert_eq!(hand_len(&g, P0), mine, "the caster did not");
}

#[test]
fn the_ai_bloodgift_demon_still_draws_for_itself_above_ten_life() {
    let mut g = main_table("sheoldred");
    let m = perm(&mut g, P0, "Bloodgift Demon");
    g.player_mut(P0).life = 40;
    let mine = hand_len(&g, P0);
    let f = g.registry.get(g.perm(m).card()).unwrap().upkeep.unwrap();
    f(&mut g, m, P0).unwrap();
    assert_eq!((hand_len(&g, P0), g.player(P0).life), (mine + 1, 39));
    g.player_mut(P0).life = 10;
    f(&mut g, m, P0).unwrap();
    assert_eq!((hand_len(&g, P0), g.player(P0).life), (mine + 1, 10), "not at 10 life or less");
}
