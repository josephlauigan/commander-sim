//! Practice mode: a fetch land stays on the battlefield for a person and is an ability they activate when they choose
//! (`human/fetch.rs`), paying what the card says; they pick the land from the legal ones. The AI still cracks at once.

mod play_common;
use play_common::*;
use sim_core::state::Step;

fn land_named(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).lands.iter().copied().find(|&l| g.db.get(g.perm(l).card()).name.as_ref() == name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| g.db.get(c).name.as_ref() == name)
}

/// a table with the library holding exactly these cards (the search finds them), P0 holding the fetch land
fn setup(fetch: &str, library: &[&str]) -> Game {
    let mut g = table(&["thanos", "seph"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    hand(&mut g, P0, &[fetch]);
    let cs: Vec<_> = library.iter().map(|n| card(&g, n)).collect();
    g.player_mut(P0).library = cs;
    g
}

fn play_land() -> F {
    act(json!({"do": "land", "card": 0}))
}

fn use_land(i: usize) -> F {
    act(json!({"do": "use", "land": i}))
}

#[test]
fn a_person_keeps_the_fetch_land_uncracked_as_it_enters() {
    let mut g = setup("Polluted Delta", &["Island", "Swamp", "Forest"]);
    let h = main(&mut g, P0, vec![play_land()]);
    assert!(land_named(&g, P0, "Polluted Delta").is_some(), "the fetch land is still on the battlefield");
    assert!(!in_gy(&g, P0, "Polluted Delta"));
    assert_eq!(g.player(P0).life, 40, "no life paid yet");
    assert_eq!(g.player(P0).library.len(), 3, "nothing searched");
    assert!(h.asked_kind("search").is_empty());
}

#[test]
fn the_person_activates_it_pays_a_life_and_picks_the_land() {
    let mut g = setup("Polluted Delta", &["Island", "Swamp", "Forest", "Watery Grave", "Mountain"]);
    let h = main(&mut g, P0, vec![play_land(), use_land(0), pick(0), by_text("Swamp")]);
    let s = h.asked_kind("search");
    assert_eq!(s.len(), 1, "one search");
    // Polluted Delta: Island or Swamp card: the basics and the Island/Swamp dual, not Forest or Mountain
    let c = s[0].choices.join(" | ");
    assert!(c.contains("Island") && c.contains("Swamp") && c.contains("Watery Grave"), "{c}");
    assert!(!c.contains("Forest") && !c.contains("Mountain"), "{c}");
    assert!(in_gy(&g, P0, "Polluted Delta") && land_named(&g, P0, "Polluted Delta").is_none());
    assert_eq!(g.player(P0).life, 39, "1 life paid");
    let sw = land_named(&g, P0, "Swamp").expect("the person took the Swamp");
    assert!(!g.perm(sw).tapped, "a true fetch land puts the land onto the battlefield untapped");
    assert_eq!(g.player(P0).library.len(), 4);
}

#[test]
fn it_can_be_cracked_later_and_may_find_nothing() {
    let mut g = setup("Polluted Delta", &["Island", "Swamp"]);
    // find nothing: the last choice is "find nothing"
    let h = main(
        &mut g,
        P0,
        vec![
            play_land(),
            use_land(0),
            pick(0),
            Box::new(|r| Answer::Pick(r.choices.len() - 1)),
        ],
    );
    assert!(h.asked_kind("search").len() == 1);
    assert!(in_gy(&g, P0, "Polluted Delta"), "sacrificed all the same");
    assert_eq!(g.player(P0).lands.len(), 0);
    assert_eq!(g.player(P0).life, 39);
}

#[test]
fn evolving_wilds_pays_no_life_and_the_land_enters_tapped() {
    let mut g = setup("Evolving Wilds", &["Island", "Swamp", "Watery Grave"]);
    let h = main(&mut g, P0, vec![play_land(), use_land(0), pick(0), by_text("Island")]);
    let c = h.asked_kind("search")[0].choices.join(" | ");
    assert!(!c.contains("Watery Grave"), "basic land cards only: {c}");
    assert_eq!(g.player(P0).life, 40);
    let i = land_named(&g, P0, "Island").unwrap();
    assert!(g.perm(i).tapped, "it enters tapped");
}

#[test]
fn terminal_moraine_costs_two_mana() {
    let mut g = setup("Terminal Moraine", &["Island", "Swamp"]);
    let h = main(&mut g, P0, vec![play_land(), use_land(0), pick(0)]);
    assert!(land_named(&g, P0, "Terminal Moraine").is_some(), "not paid: still there");
    assert!(told_last(&h).contains("Can't activate"), "{}", told_last(&h));
    let mut g = setup("Terminal Moraine", &["Island", "Swamp"]);
    pool(&mut g, P0, &[('G', 2)]);
    main(&mut g, P0, vec![play_land(), use_land(0), pick(0), pick(0)]);
    assert!(in_gy(&g, P0, "Terminal Moraine"));
    assert_eq!(pool_total(&g, P0), 0, "{{2}} paid");
    assert!(land_named(&g, P0, "Island").is_some());
}

#[test]
fn fabled_passage_untaps_the_land_with_four_lands() {
    // three other lands: the new one is the fourth
    let mut g = setup("Fabled Passage", &["Island", "Swamp"]);
    lands(&mut g, P0, "Forest", 3, false);
    main(&mut g, P0, vec![play_land(), use_land(3), pick(0), pick(0)]);
    let l = land_named(&g, P0, "Island").unwrap();
    assert!(!g.perm(l).tapped, "four lands: untapped");
    // two other lands: it stays tapped
    let mut g = setup("Fabled Passage", &["Island", "Swamp"]);
    lands(&mut g, P0, "Forest", 2, false);
    main(&mut g, P0, vec![play_land(), use_land(2), pick(0), pick(0)]);
    let l = land_named(&g, P0, "Island").unwrap();
    assert!(g.perm(l).tapped, "three lands: tapped");
}

#[test]
fn a_fetched_shock_land_asks_whether_to_pay_for_it() {
    let mut g = setup("Polluted Delta", &["Watery Grave", "Island"]);
    let h = main(&mut g, P0, vec![play_land(), use_land(0), pick(0), by_text("Watery Grave"), pick(0)]);
    assert!(h.asked().iter().any(|r| r.prompt.contains("pay 2 life")), "asked about the shock");
    assert_eq!(g.player(P0).life, 37, "1 for the fetch land and 2 for the shock land");
    let l = land_named(&g, P0, "Watery Grave").unwrap();
    assert!(!g.perm(l).tapped);
}

#[test]
fn a_tapped_fetch_land_cannot_be_activated() {
    let mut g = setup("Polluted Delta", &["Island"]);
    main(&mut g, P0, vec![play_land()]);
    let l = land_named(&g, P0, "Polluted Delta").unwrap();
    g.perm_mut(l).tapped = true;
    let h = main(&mut g, P0, vec![use_land(0), pick(0)]);
    assert!(land_named(&g, P0, "Polluted Delta").is_some());
    assert!(told_last(&h).contains("tapped"), "{}", told_last(&h));
}

#[test]
fn the_ai_still_cracks_a_fetch_land_at_once() {
    let mut g = setup("Polluted Delta", &["Island", "Swamp"]);
    let c = take(&mut g, P0, "Polluted Delta");
    g.player_mut(P0).hand.retain(|&x| x != c);
    sim_core::engine::turn::play_land_card(&mut g, P0, c, "plays").unwrap();
    assert!(in_gy(&g, P0, "Polluted Delta"), "cracked as it entered");
    assert_eq!(g.player(P0).lands.len(), 1);
    assert_eq!(g.player(P0).life, 39);
}
