//! Flash grants (plan row 14): a one-turn player grant (Borne Upon a Wind, Emergence Zone, Scout's Warning) and a
//! permanent static grant (High Fae Trickster, Valley Floodcaller) read by the cast-timing check
//! (`cast::grants_flash`, `human::legal::instant_speed`) and by the AI's end-of-turn window.

use sim_core::ai::brain;
use sim_core::engine::cast;
use sim_core::human::legal;
use sim_core::ids::{CardId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    let mut g = table(&["rograkh-silas-turbo-breach", "seph"]);
    g.active = Some(P0);
    g
}

fn cast_free(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
    c
}

fn gy_has(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

#[test]
fn borne_upon_a_wind_makes_sorceries_and_creatures_instant_speed_for_that_turn_only() {
    let mut g = game();
    let (sorc, crea) = (take(&mut g, P0, "Sleight of Hand"), take(&mut g, P0, "Aven Fisher"));
    g.active = Some(P1);
    assert!(!legal::instant_speed(&g, P0, sorc) && !legal::instant_speed(&g, P0, crea));
    cast_free(&mut g, P0, "Borne Upon a Wind");
    assert!(legal::instant_speed(&g, P0, sorc) && legal::instant_speed(&g, P0, crea));
    assert!(cast::grants_flash(&g, P0, sorc));
    assert!(!legal::instant_speed(&g, P1, sorc), "only the caster");
    assert!(!cast::grants_flash(&g, P0, take(&mut g.clone(), P0, "Island")), "never a land");
    g.round += 1; // the next turn: gone
    assert!(!legal::instant_speed(&g, P0, sorc));
}

#[test]
fn floodcaller_gives_noncreature_spells_flash_trickster_all() {
    let mut g = game();
    let (sorc, crea) = (take(&mut g, P0, "Sleight of Hand"), take(&mut g, P0, "Aven Fisher"));
    perm(&mut g, P0, "Valley Floodcaller");
    assert!(legal::instant_speed(&g, P0, sorc));
    assert!(!legal::instant_speed(&g, P0, crea));
    perm(&mut g, P0, "High Fae Trickster");
    assert!(legal::instant_speed(&g, P0, crea));
}

#[test]
fn scouts_warning_covers_only_the_next_creature_card() {
    let mut g = game();
    let (sorc, crea) = (take(&mut g, P0, "Sleight of Hand"), hand(&mut g, P0, &["Aven Fisher"])[0]);
    cast_free(&mut g, P0, "Scout's Warning");
    assert!(legal::instant_speed(&g, P0, crea));
    assert!(!legal::instant_speed(&g, P0, sorc), "creature cards only");
    cast::cast_card(&mut g, P0, crea, "hand", Ctx::default()).unwrap();
    let c2 = take(&mut g, P0, "Aven Fisher");
    assert!(!legal::instant_speed(&g, P0, c2), "the grant is spent");
}

#[test]
fn ai_casts_a_sorcery_and_a_creature_at_the_end_of_the_opponents_turn_with_trickster() {
    let mut g = game();
    lands(&mut g, P0, "Island", 6, false);
    perm(&mut g, P0, "High Fae Trickster");
    hand(&mut g, P0, &["Sleight of Hand", "Aven Fisher"]);
    g.active = Some(P1);
    brain::end_of_turn_window(&mut g, P0).unwrap();
    assert!(gy_has(&g, P0, "Sleight of Hand"), "sorcery cast at instant speed");
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == "Aven Fisher")));
}

#[test]
fn ai_holds_nothing_back_without_a_grant() {
    let mut g = game();
    lands(&mut g, P0, "Island", 6, false);
    hand(&mut g, P0, &["Sleight of Hand", "Aven Fisher"]);
    g.active = Some(P1);
    brain::end_of_turn_window(&mut g, P0).unwrap();
    assert!(!gy_has(&g, P0, "Sleight of Hand"));
}

#[test]
fn emergence_zone_is_offered_to_the_ai_at_the_end_of_an_opponents_turn_holding_a_creature() {
    let mut g = game();
    lands(&mut g, P0, "Emergence Zone", 1, false);
    lands(&mut g, P0, "Island", 5, false);
    hand(&mut g, P0, &["Aven Fisher"]);
    g.active = Some(P1);
    let opts = sim_core::cardcode::land_options(&mut g, P0, None).unwrap();
    assert!(opts.iter().any(|o| o.label == "Emergence Zone"));
}
