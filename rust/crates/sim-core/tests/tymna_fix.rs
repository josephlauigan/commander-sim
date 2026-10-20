//! tymna-fix: the AI problems the first thrasios / kraum run found (Lagrella, Witherbloom Charm, the protection
//! spells, commanders skipping the search, Thrasios's mana sink).

use sim_core::ai::decks::{deck_prio, protect_response, wipe_response};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::PlayerId;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl(key: &str) -> Game {
    table(&[key, "veyran", "seph", "galadriel"])
}

fn on(g: &Game, m: sim_core::ids::PermId) -> bool {
    g.perm(m).on_bf
}

#[test]
fn lagrella_is_cast_when_opponents_have_creatures_and_not_valued_zero() {
    for key in ["thrasios"] {
        let mut g = tbl(key);
        let c = hand(&mut g, P0, &["Lagrella, the Magpie"])[0];
        let empty = deck_prio(&g, P0, c);
        assert!(empty > 0, "{key}: a body even without a target");
        perm(&mut g, P1, "Lyra Dawnbringer");
        assert!(deck_prio(&g, P0, c) > empty, "{key}: a target to exile raises it");
    }
}

fn cast_charm(g: &mut Game, mode: i32, target: Option<sim_core::ids::PermId>) {
    let c = hand(g, P0, &["Witherbloom Charm"])[0];
    sim_core::engine::cast::cast_card(g, P0, c, "hand", Ctx { x: mode, target, ..Default::default() }).unwrap();
}

#[test]
fn witherbloom_charm_has_its_three_modes() {
    let mut g = tbl("thrasios");
    // mode 2: gain 5 life
    let life = g.player(P0).life;
    cast_charm(&mut g, 2, None);
    assert_eq!(g.player(P0).life, life + 5);
    // mode 3: destroy a nonland permanent with mana value 2 or less, not a bigger one
    let small = perm(&mut g, P1, "Birds of Paradise");
    let big = perm(&mut g, P1, "Lyra Dawnbringer");
    cast_charm(&mut g, 3, Some(big));
    assert!(on(&g, big), "mana value 5 is not a legal target");
    cast_charm(&mut g, 3, Some(small));
    assert!(!on(&g, small));
    // mode 1: sacrifice a token, draw two
    let t = token(&mut g, P0, 1);
    let hand_before = g.player(P0).hand.len();
    cast_charm(&mut g, 1, None);
    assert!(!on(&g, t), "the token was sacrificed");
    assert_eq!(g.player(P0).hand.len(), hand_before + 2);
}

#[test]
fn witherbloom_charm_is_offered_as_a_hand_play() {
    let mut g = tbl("thrasios");
    lands(&mut g, P0, "Swamp", 1, false);
    lands(&mut g, P0, "Forest", 1, false);
    token(&mut g, P0, 1);
    let c = hand(&mut g, P0, &["Witherbloom Charm"])[0];
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    let o: Vec<Opt> = f(&mut g, c, P0, Some(false)).unwrap();
    assert!(o.iter().any(|x| x.label.contains("draw two")), "{:?}", o.iter().map(|x| &x.label).collect::<Vec<_>>());
    assert!(o.iter().all(|x| matches!(x.act, Some(Action::Plan { .. }))));
}

#[test]
fn the_partner_decks_answer_removal_and_wipes_with_their_white_spells() {
    for key in ["thrasios"] {
        // Shardmage's Rescue: hexproof for a creature that targeted removal is aimed at
        let mut g = tbl(key);
        let tymna = perm(&mut g, P0, "Tymna the Weaver");
        g.perm_mut(tymna).is_cmd = true;
        lands(&mut g, P0, "Plains", 1, false);
        hand(&mut g, P0, &["Shardmage's Rescue"]);
        assert!(protect_response(&mut g, P0, tymna, "exile", Some(P1), None).unwrap(), "{key}: Rescue answers");
        assert!(on_bf(&g, P0, "Shardmage's Rescue").is_some());

        // Ultimate Magic: Holy: destroy removal on Tymna, and a destroy wipe
        let mut g = tbl(key);
        let tymna = perm(&mut g, P0, "Tymna the Weaver");
        g.perm_mut(tymna).is_cmd = true;
        lands(&mut g, P0, "Plains", 3, false);
        hand(&mut g, P0, &["Ultimate Magic: Holy"]);
        assert!(protect_response(&mut g, P0, tymna, "destroy", Some(P1), None).unwrap(), "{key}: Holy answers");
        assert!(in_gy(&g, P0, "Ultimate Magic: Holy"));

        let mut g = tbl(key);
        for _ in 0..3 {
            perm(&mut g, P0, "Lyra Dawnbringer");
        }
        lands(&mut g, P0, "Plains", 3, false);
        hand(&mut g, P0, &["Ultimate Magic: Holy"]);
        let r = wipe_response(&mut g, P0, "destroy", P1).unwrap();
        assert_eq!(r, Some("indes"), "{key}: Holy saves the board from a wipe");
    }
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<sim_core::ids::PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}
