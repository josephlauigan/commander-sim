//! P5a-2b source doublers: each original gets its extras once (CR 603.2d).
use sim_core::engine::hooks::fire_trigger;
use sim_core::engine::{combat, life, turn, zones};
use sim_core::hooks::{Call, Event};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::{combos, rules, t45_miirym};
use sim_core::state::{DataKey, Game, Step, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const DELNEY: &str = "Delney, Streetwise Lookout";
const CLOUD: &str = "Cloud, Midgar Mercenary";

fn tbl() -> Game {
    table(&["heliod-mono-white-stax", "seph", "sauron", "cloud"])
}
fn dragons() -> Game {
    table(&["miirym-temur-dragons", "seph", "sauron", "cloud"])
}
fn throne(g: &mut Game) {
    let t = perm(g, P0, "Roaming Throne");
    assert_eq!(g.perm(t).data.get(DataKey::Ctype), Some(&Val::Str("dragon")));
}
fn attach(g: &mut Game, m: PermId, name: &str) {
    let e = perm(g, P0, name);
    g.perm_mut(e).attached = Some(m);
}
fn opp_life(g: &Game) -> Vec<i32> {
    g.opps(P0).map(|q| g.player(q).life).collect()
}
fn main2(g: &mut Game) {
    g.step = Step::Main2;
    fire_trigger(g, Event::Main2, Call::Player { p: P0 }).unwrap();
}
fn tymna_position() -> Game {
    let mut g = tbl();
    perm(&mut g, P0, DELNEY);
    let t = perm(&mut g, P0, "Tymna the Weaver");
    combat::resolve_combat(&mut g, P0, &[t], P1, &[]).unwrap();
    combat::resolve_combat(&mut g, P0, &[t], PlayerId(2), &[]).unwrap();
    g
}

// Design 8: two original instances, each pays two and draws two.
#[test]
fn tymna_delney_pays_and_draws_twice() {
    let mut g = tymna_position();
    let (life, cards) = (g.player(P0).life, g.player(P0).hand.len());
    main2(&mut g);
    assert_eq!((g.player(P0).life, g.player(P0).hand.len()), (life - 4, cards + 4));
}

// Design 9 control: resuming the copy at Main2 cannot reopen the beginning-of-phase trigger.
// The absolute four-card count is asserted separately by design 8.
#[test]
fn tymna_main2_search_copy_does_not_draw_again() {
    let mut g = tymna_position();
    main2(&mut g);
    let mut copy = g.clone();
    let (life, lib) = (copy.player(P0).life, copy.player(P0).library.len());
    // No mana: the main phase cannot spend any of Tymna's newly drawn cards.
    turn::continue_turn(&mut copy, P0, Step::Main2).unwrap();
    assert_eq!((copy.player(P0).life, copy.player(P0).library.len()), (life, lib));
}

fn adeline_tokens() -> usize {
    let mut g = tbl();
    perm(&mut g, P0, DELNEY);
    let a = perm(&mut g, P0, "Adeline, Resplendent Cathar");
    combat::attack_triggers(&mut g, P0, &[a], P1).unwrap().len()
}
#[test]
fn adeline_delney_makes_six_attacking_tokens() {
    assert_eq!(adeline_tokens(), 6);
}
#[test]
fn animist_on_adeline_is_not_delneys_trigger() {
    let mut g = tbl();
    perm(&mut g, P0, DELNEY);
    let a = perm(&mut g, P0, "Adeline, Resplendent Cathar");
    attach(&mut g, a, "Sword of the Animist");
    combat::attack_triggers(&mut g, P0, &[a], P1).unwrap();
    assert_eq!(g.player(P0).lands.len(), 1);
}
#[test]
fn animist_on_cloud_searches_twice() {
    let mut g = tbl();
    let cl = perm(&mut g, P0, CLOUD);
    attach(&mut g, cl, "Sword of the Animist");
    combat::attack_triggers(&mut g, P0, &[cl], P1).unwrap();
    assert_eq!(g.player(P0).lands.len(), 2);
}
#[test]
fn bowmasters_delney_extra_draw_amasses_and_pings_twice() {
    let mut g = tbl();
    perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, "Orcish Bowmasters");
    let army = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).army).unwrap();
    let (life, plus) = (g.player(P1).life, g.perm(army).plus);
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!((g.player(P1).life, g.perm(army).plus), (life - 2, plus + 2));
}
#[test]
fn terror_throne_token_deals_ten() {
    let mut g = dragons();
    throne(&mut g);
    perm(&mut g, P0, "Terror of the Peaks");
    let before: i32 = opp_life(&g).iter().sum();
    zones::make_tokens(&mut g, P0, zones::Tokens::new(1, 5)).unwrap();
    assert_eq!(before - opp_life(&g).iter().sum::<i32>(), 10);
}
#[test]
fn terror_throne_nontoken_control_deals_six() {
    let mut g = dragons();
    throne(&mut g);
    perm(&mut g, P0, "Terror of the Peaks");
    let before_enter: i32 = opp_life(&g).iter().sum();
    perm(&mut g, P0, "Thrakkus the Butcher");
    assert_eq!(before_enter - opp_life(&g).iter().sum::<i32>(), 6);
}
#[test]
fn champion_throne_draws_two_per_champion() {
    for champions in [1, 2] {
        let mut g = dragons();
        throne(&mut g);
        for _ in 0..champions {
            perm(&mut g, P0, "Dragonborn Champion");
        }
        let cards = g.player(P0).hand.len();
        life::lose_life(&mut g, P1, 5, Some(P0), "triggers", None).unwrap();
        assert_eq!(g.player(P0).hand.len() - cards, 2 * champions);
    }
}
#[test]
fn wrathful_throne_damage_deals_six() {
    let mut g = dragons();
    throne(&mut g);
    perm(&mut g, P0, "Wrathful Red Dragon");
    let other = perm(&mut g, P0, "Goldspan Dragon");
    let before: i32 = opp_life(&g).iter().sum();
    sim_core::impls::t0_giada::damaged(&mut g, other, 3).unwrap();
    assert_eq!(before - opp_life(&g).iter().sum::<i32>(), 6);
}
#[test]
fn wrathful_throne_wipe_and_planner_count_six_chunks() {
    let mut g = dragons();
    throne(&mut g);
    perm(&mut g, P0, "Wrathful Red Dragon");
    perm(&mut g, P0, "Goldspan Dragon");
    hand(&mut g, P0, &["Blasphemous Act"]);
    for q in [P1, PlayerId(2), PlayerId(3)] {
        g.player_mut(q).life = 26;
    }
    assert_eq!(t45_miirym::wrathful_wipe(&g, 13), vec![(P0, 13); 6]);
    let cmb = combos::all().iter().find(|c| c.name == "Wrathful Red Dragon + Blasphemous Act").unwrap();
    assert!((cmb.ready)(&g, P0).0);
}
#[test]
fn goldspan_targeted_throne_makes_two_treasures() {
    let mut g = dragons();
    throne(&mut g);
    let gold = perm(&mut g, P0, "Goldspan Dragon");
    rules::goldspan_targeted(&mut g, gold).unwrap();
    assert_eq!(g.player(P0).treasures, 2);
}
#[test]
fn goldspan_targeted_without_throne_control_makes_one() {
    let mut g = dragons();
    let gold = perm(&mut g, P0, "Goldspan Dragon");
    rules::goldspan_targeted(&mut g, gold).unwrap();
    assert_eq!(g.player(P0).treasures, 1);
}
fn sword_discards(cloud: bool, swords: usize) -> usize {
    let mut g = tbl();
    let a = if cloud { perm(&mut g, P0, CLOUD) } else { token(&mut g, P0, 2) };
    for _ in 0..swords {
        attach(&mut g, a, "Sword of Feast and Famine");
    }
    hand(&mut g, P1, &["Plains", "Plains", "Plains", "Plains"]);
    combat::resolve_combat(&mut g, P0, &[a], P1, &[]).unwrap();
    4 - g.player(P1).hand.len()
}
#[test]
fn sword_on_cloud_discards_twice() {
    assert_eq!(sword_discards(true, 1), 2);
}
#[test]
fn sword_on_other_creature_control_discards_once() {
    assert_eq!(sword_discards(false, 1), 1);
}
#[test]
fn two_swords_each_discard_once() {
    assert_eq!(sword_discards(false, 2), 2);
}
#[test]
fn oracle_delney_second_window_wins_after_first_is_countered() {
    let mut g = tbl();
    perm(&mut g, P0, DELNEY);
    g.skip_etb = true;
    let oracle = perm(&mut g, P0, "Thassa's Oracle");
    g.skip_etb = false;
    hand(&mut g, P1, &["Tishana's Tidebinder"]);
    lands(&mut g, P1, "Island", 3, false);
    g.player_mut(P0).library.clear();
    fire_trigger(&mut g, Event::Etb, Call::Etb { p: P0, m: oracle }).unwrap();
    assert!(!g.player(P1).hand.contains(&card(&g, "Tishana's Tidebinder")));
    assert!(g.over && g.winner == Some(P0));
}
#[test]
fn fireweaver_delney_one_treasure_pings_twice() {
    let mut g = tbl();
    perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, "Reckless Fireweaver");
    let before = opp_life(&g);
    zones::add_treasure(&mut g, P0, 1).unwrap();
    assert_eq!(opp_life(&g), before.into_iter().map(|l| l - 2).collect::<Vec<_>>());
}
#[test]
fn wispdrinker_delney_small_nontoken_drains_twice() {
    let mut g = tbl();
    perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, "Wispdrinker Vampire");
    let (life, before) = (g.player(P0).life, opp_life(&g));
    perm(&mut g, P0, "Llanowar Elves");
    assert_eq!(opp_life(&g), before.into_iter().map(|l| l - 2).collect::<Vec<_>>());
    assert_eq!(g.player(P0).life, life + 6);
}
#[test]
fn wispdrinker_delney_small_token_drains_twice() {
    let mut g = tbl();
    perm(&mut g, P0, DELNEY);
    perm(&mut g, P0, "Wispdrinker Vampire");
    let (life, before) = (g.player(P0).life, opp_life(&g));
    token(&mut g, P0, 1);
    assert_eq!(opp_life(&g), before.into_iter().map(|l| l - 2).collect::<Vec<_>>());
    assert_eq!(g.player(P0).life, life + 6);
}
