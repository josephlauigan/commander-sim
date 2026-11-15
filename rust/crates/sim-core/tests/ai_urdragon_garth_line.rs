//! The Ur-Dragon deck's card-code fixes, guarded to the `urdragon` seat: The Ur-Dragon's free drop never puts Tiamat
//! onto the battlefield (its "if you cast it" search would be lost), Twinflame is held while the Garth line is alive,
//! and Spark Double copies Garth when the rest of the line is near. Every other seat plays as before.
mod play_common;
use play_common::*;
use sim_core::ai::brain::main_options;
use sim_core::engine::zones;
use sim_core::ids::CardId;
use sim_core::impls::t45_urdragon::ur_pick;
use sim_core::rng::Rng;
use sim_core::state::Step;

// ------------------------------------------------------------------ the free drop: never Tiamat
fn permanent_cards(g: &Game) -> Vec<CardId> {
    deck("urdragon")
        .cards
        .iter()
        .map(|n| card(g, n))
        .filter(|&c| {
            let d = g.db.get(c);
            d.land || (d.perm && !d.has_subtype("aura"))
        })
        .collect()
}

/// The cards `ur_attack` offers `ur_pick`: lands and non-Aura permanents of the hand.
fn offered(g: &Game, hand: &[CardId]) -> Vec<CardId> {
    hand.iter()
        .copied()
        .filter(|&c| g.db.get(c).land || (g.db.get(c).perm && !g.db.get(c).has_subtype("aura")))
        .collect()
}

#[test]
fn ur_pick_never_returns_tiamat_over_500_seeded_states() {
    let mut rng = Rng::from_u64(20261115);
    let mut seen = 0;
    for i in 0..500 {
        let mut g = table(&["urdragon", "seph"]);
        let tiamat = card(&g, "Tiamat");
        let pool: Vec<CardId> = permanent_cards(&g).into_iter().filter(|&c| c != tiamat).collect();
        let mut hand_cards = vec![tiamat];
        for _ in 0..rng.below(8) {
            hand_cards.push(*rng.choice(&pool).unwrap());
        }
        for _ in 0..rng.below(9) {
            lands(&mut g, P0, "Plains", 1, false);
        }
        let cs = offered(&g, &hand_cards);
        let without: Vec<CardId> = cs.iter().copied().filter(|&c| c != tiamat).collect();
        let pick = ur_pick(&g, P0, &cs);
        assert_ne!(pick, Some(tiamat), "state {i}: hand {:?} offered Tiamat", names(&g, &cs));
        assert_eq!(pick, ur_pick(&g, P0, &without), "state {i}: Tiamat changed the pick from {:?}", names(&g, &cs));
        seen += 1;
    }
    assert_eq!(seen, 500);
}

fn names(g: &Game, cs: &[CardId]) -> Vec<String> {
    cs.iter().map(|&c| g.db.get(c).name.to_string()).collect()
}

#[test]
fn ur_pick_with_only_tiamat_picks_nothing_and_with_a_land_picks_the_land() {
    let g = table(&["urdragon", "seph"]);
    let tiamat = card(&g, "Tiamat");
    assert_eq!(ur_pick(&g, P0, &[tiamat]), None);
    let land = card(&g, "Plains");
    assert_eq!(ur_pick(&g, P0, &[tiamat, land]), Some(land));
}

#[test]
fn ur_pick_for_another_seat_still_picks_the_biggest_permanent_tiamat() {
    // guard off: Tiamat is the highest `body_worth` of Tiamat and a one-drop, no land
    let g = table(&["aragorn", "seph"]);
    let (tiamat, elves) = (card(&g, "Tiamat"), card(&g, "Llanowar Elves"));
    assert_eq!(ur_pick(&g, P0, &[tiamat, elves]), Some(tiamat));
}
