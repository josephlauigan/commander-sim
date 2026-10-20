//! JD's Tymna/Thrasios deck (key `thrasios`): Thrasios's {4} as a mana sink, the Basalt Monolith + Rings of
//! Brighthearth loop with Walking Ballista (on the battlefield, castable, or found with Thrasios) and Worldly Tutor
//! fetching the Ballista.

use sim_core::ai::act;
use sim_core::engine::mana::total_mana;
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PermId, PlayerId};

use sim_core::impls::combos;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const THRASIOS: &str = "Thrasios, Triton Hero";

fn tbl() -> Game {
    let mut g = table(&["thrasios", "seph", "veyran"]);
    g.active = Some(P0);
    g
}

fn ready(g: &Game) -> bool {
    combos::ready(g, P0, combos::combo_index("Rings of Brighthearth").unwrap())
}

fn set_library(g: &mut Game, top: &[&str]) {
    // the last name is the top card; plenty of filler below
    let filler = card(g, "Forest");
    let mut lib: Vec<CardId> = vec![filler; 20];
    lib.extend(top.iter().map(|n| card(g, n)));
    g.player_mut(P0).library = lib;
}

fn options(g: &mut Game, src: PermId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, P0, post).unwrap()
}

fn ballista_in_play(g: &mut Game) {
    let c = take(g, P0, "Walking Ballista");
    let b = sim_core::engine::zones::enter(
        g,
        P0,
        c,
        sim_core::engine::zones::Enter { plus: 2, sick: false, ..Default::default() },
    )
    .unwrap();
    assert!(g.perm(b).on_bf);
}

fn has_thrasios_opt(os: &[Opt]) -> bool {
    os.iter().any(|o| o.label.starts_with("Thrasios:"))
}

// ---------------------------------------------------------------- (a) Thrasios as a mana sink
#[test]
fn thrasios_is_offered_with_spare_mana_at_the_end_of_the_turn_before_its_own() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 4, false);
    let t = perm(&mut g, P0, THRASIOS);
    set_library(&mut g, &["Island"]);
    assert!(has_thrasios_opt(&options(&mut g, t, None)), "eot window, {{4}} open");
    // not with less than {4}
    let mut g2 = tbl();
    lands(&mut g2, P0, "Forest", 3, false);
    let t2 = perm(&mut g2, P0, THRASIOS);
    set_library(&mut g2, &["Island"]);
    assert!(!has_thrasios_opt(&options(&mut g2, t2, None)));
}

#[test]
fn thrasios_is_used_in_the_second_main_phase_after_everything_else() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 5, false);
    let t = perm(&mut g, P0, THRASIOS);
    set_library(&mut g, &["Island"]);
    let os = options(&mut g, t, Some(true));
    let o = os.iter().find(|o| o.label.starts_with("Thrasios:")).expect("offered in main 2");
    assert!(o.utility < 0.0 && o.utility > -5.0, "a low utility, above 'stop': {}", o.utility);
    assert!(options(&mut g, t, Some(false)).is_empty(), "never in the first main phase");
}

#[test]
fn thrasios_puts_a_land_onto_the_battlefield_and_draws_anything_else() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 8, false);
    let t = perm(&mut g, P0, THRASIOS);
    set_library(&mut g, &["Island"]);
    let (lands0, hand0) = (g.player(P0).lands.len(), g.player(P0).hand.len());
    let o = options(&mut g, t, None).into_iter().find(|o| o.label.starts_with("Thrasios:")).unwrap();
    assert!(act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P0).lands.len(), lands0 + 1, "a land goes onto the battlefield");
    set_library(&mut g, &["Elspeth, Sun's Champion", "Sun Titan"]); // whichever the scry keeps is drawn
    let o = options(&mut g, t, None).into_iter().find(|o| o.label.starts_with("Thrasios:")).unwrap();
    assert!(act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P0).hand.len(), hand0 + 1, "anything else is drawn");
    assert_eq!(total_mana(&g, P0, false), 0);
}

// ---------------------------------------------------------------- (b) Basalt + Rings + Ballista
#[test]
fn basalt_and_rings_win_through_a_ballista_on_the_battlefield_or_in_hand() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 2, false);
    perm(&mut g, P0, "Basalt Monolith");
    perm(&mut g, P0, "Rings of Brighthearth");
    assert!(!ready(&g), "infinite colourless mana alone kills no one");
    hand(&mut g, P0, &["Walking Ballista"]);
    assert!(ready(&g), "castable for any X");
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 2, false);
    perm(&mut g, P0, "Basalt Monolith");
    perm(&mut g, P0, "Rings of Brighthearth");
    ballista_in_play(&mut g);
    assert!(ready(&g), "on the battlefield");
}

#[test]
fn the_first_loop_needs_five_mana_in_all_two_more_than_basalt_untapped_or_five_if_it_is_tapped() {
    let mut g = tbl();
    let b = perm(&mut g, P0, "Basalt Monolith");
    perm(&mut g, P0, "Rings of Brighthearth");
    hand(&mut g, P0, &["Walking Ballista"]);
    assert!(!ready(&g), "Basalt alone: {{C}}{{C}}{{C}}, the untap and the copy cost {{5}}");
    lands(&mut g, P0, "Forest", 2, false);
    assert!(ready(&g), "untapped Basalt plus two");
    g.perm_mut(b).tapped = true;
    assert!(!ready(&g), "tapped: two lands are not enough");
    lands(&mut g, P0, "Forest", 3, false);
    assert!(ready(&g), "tapped Basalt and five lands");
}

#[test]
fn rings_alone_with_a_different_monolith_is_not_the_loop() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 5, false);
    perm(&mut g, P0, "Grim Monolith");
    perm(&mut g, P0, "Rings of Brighthearth");
    hand(&mut g, P0, &["Walking Ballista"]);
    assert!(!ready(&g));
}

#[test]
fn without_a_ballista_in_hand_thrasios_digs_for_it() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 2, false);
    perm(&mut g, P0, "Basalt Monolith");
    perm(&mut g, P0, "Rings of Brighthearth");
    set_library(&mut g, &["Walking Ballista", "Island"]);
    assert!(!ready(&g), "no Thrasios, Ballista deep in the library: nothing to win with");
    perm(&mut g, P0, THRASIOS);
    assert!(ready(&g), "Thrasios draws the library for the Ballista");
    // no Ballista left in the library (and none in hand): no win
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 2, false);
    perm(&mut g, P0, "Basalt Monolith");
    perm(&mut g, P0, "Rings of Brighthearth");
    perm(&mut g, P0, THRASIOS);
    let ballista = card(&g, "Walking Ballista");
    g.player_mut(P0).library.retain(|&c| c != ballista);
    assert!(!ready(&g));
}

#[test]
fn the_ai_goes_for_the_loop_and_wins() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 2, false);
    perm(&mut g, P0, "Basalt Monolith");
    perm(&mut g, P0, "Rings of Brighthearth");
    ballista_in_play(&mut g);
    let i = combos::combo_index("Rings of Brighthearth").unwrap();
    assert!(combos::attempt(&mut g, P0, i as i64).unwrap());
    assert!(g.over && g.winner == Some(P0), "the combo wins");
}

// ---------------------------------------------------------------- (c) Worldly Tutor fetches the Ballista
#[test]
fn worldly_tutor_fetches_walking_ballista_when_basalt_and_rings_are_out() {
    let mut g = tbl();
    perm(&mut g, P0, "Basalt Monolith");
    perm(&mut g, P0, "Rings of Brighthearth");
    set_library(&mut g, &[]);
    let ballista = card(&g, "Walking Ballista");
    g.player_mut(P0).library.insert(3, ballista);
    let c = hand(&mut g, P0, &["Worldly Tutor"])[0];
    let f = g.registry.get(c).unwrap().resolve.unwrap();
    let ctx = sim_core::state::Ctx::default();
    f(&mut g, P0, c, &ctx).unwrap();
    assert_eq!(g.player(P0).library.last(), Some(&ballista), "the Ballista is on top, the next draw");
}
