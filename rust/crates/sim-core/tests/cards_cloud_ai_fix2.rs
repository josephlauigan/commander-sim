//! Tests for the cloud-ai-fix2 task: one section per fix.

use sim_core::ids::PlayerId;
use sim_core::settings::AiMode;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const CLOUD: &str = "Cloud, Midgar Mercenary";

fn game() -> Game {
    let mut g = table(&["cloud", "seph"]);
    g.player_mut(P0).hand.clear();
    set_ai(&mut g, AiMode::Lookahead);
    g
}

fn set_library(g: &mut Game, p: PlayerId, names: &[&str]) {
    let cs: Vec<_> = names.iter().map(|n| card(g, n)).collect();
    g.player_mut(p).library = cs;
}

const LIB: [&str; 7] = [
    "Sword of Fire and Ice",
    "Sword of Hearth and Home",
    "Sword of Feast and Famine",
    "Champion's Helm",
    "Colossus Hammer",
    "Sword of Kaldra",
    "Wrecking Ball Arm",
];

fn fetched(g: &Game) -> Vec<String> {
    g.player(P0).hand.iter().map(|&c| g.db.get(c).name.to_string()).collect()
}

// ---- fix 1: engine-aware tutor
#[test]
fn tutor_takes_a_big_piece_with_puresteel_metalcraft() {
    let mut g = game();
    set_library(&mut g, P0, &LIB);
    lands(&mut g, P0, "Plains", 2, false);
    for n in ["Sol Ring", "Mind Stone"] {
        perm(&mut g, P0, n);
    }
    perm(&mut g, P0, "Puresteel Paladin");
    perm(&mut g, P0, CLOUD);
    let got = fetched(&g);
    assert_eq!(got.len(), 1, "{got:?}");
    assert!(["Sword of Kaldra", "Wrecking Ball Arm", "Colossus Hammer"].contains(&got[0].as_str()), "{got:?}");
}

#[test]
fn tutor_takes_a_cheap_sword_without_an_engine() {
    let mut g = game();
    set_library(&mut g, P0, &LIB);
    lands(&mut g, P0, "Plains", 2, false);
    for n in ["Sol Ring", "Mind Stone"] {
        perm(&mut g, P0, n);
    }
    perm(&mut g, P0, CLOUD);
    assert_eq!(fetched(&g), vec!["Sword of Fire and Ice".to_string()]);
}

#[test]
fn tutor_takes_a_big_piece_with_ardenn_out() {
    let mut g = game();
    set_library(&mut g, P0, &LIB);
    lands(&mut g, P0, "Plains", 3, false);
    perm(&mut g, P0, "Ardenn, Intrepid Archaeologist");
    perm(&mut g, P0, CLOUD);
    let got = fetched(&g);
    assert_eq!(got.len(), 1, "{got:?}");
    assert!(["Sword of Kaldra", "Wrecking Ball Arm", "Colossus Hammer"].contains(&got[0].as_str()), "{got:?}");
}

#[test]
fn enlightened_tutor_uses_the_same_pick() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = game();
    set_library(
        &mut g,
        P0,
        &["Sword of Fire and Ice", "Sol Ring", "Sword of Feast and Famine", "Swords to Plowshares"],
    );
    lands(&mut g, P0, "Plains", 3, false);
    let want = sim_core::ai::tutor_pick(&g, P0, "ae").map(|c| g.db.get(c).name.to_string());
    assert_eq!(want.as_deref(), Some("Sword of Fire and Ice"));
    perm(&mut g, P0, "Sword of Fire and Ice");
    let want = sim_core::ai::tutor_pick(&g, P0, "ae").map(|c| g.db.get(c).name.to_string());
    assert_eq!(want.as_deref(), Some("Sword of Feast and Famine"), "skips the piece on the battlefield");
}

// ---- fix 2: don't strand equipment
fn cast_labels(g: &mut Game) -> Vec<String> {
    sim_core::ai::brain::main_options(g, P0, false).unwrap().iter().map(|o| o.label.clone()).collect()
}

#[test]
fn equipment_is_held_without_a_carrier() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = game();
    lands(&mut g, P0, "Plains", 3, false);
    hand(&mut g, P0, &["Sword of Fire and Ice", "Mind Stone"]);
    g.player_mut(P0).turns = 3;
    // Cloud (2) + the Sword (3) don't fit in 3 mana: the Sword waits
    let l = cast_labels(&mut g);
    assert!(!l.iter().any(|x| x.contains("Sword of Fire and Ice")), "{l:?}");
    assert!(l.iter().any(|x| x.contains("Mind Stone")), "{l:?}");
}

#[test]
fn equipment_is_cast_with_a_carrier_or_as_the_only_use() {
    // Cloud and the Sword fit together
    let mut g = game();
    lands(&mut g, P0, "Plains", 6, false);
    hand(&mut g, P0, &["Sword of Fire and Ice", "Mind Stone"]);
    g.player_mut(P0).turns = 3;
    let l = cast_labels(&mut g);
    assert!(l.iter().any(|x| x.contains("Sword of Fire and Ice")), "{l:?}");
    // a creature is out
    let mut g = game();
    lands(&mut g, P0, "Plains", 3, false);
    perm(&mut g, P0, "Kor Duelist");
    hand(&mut g, P0, &["Sword of Fire and Ice", "Mind Stone"]);
    g.player_mut(P0).turns = 3;
    let l = cast_labels(&mut g);
    assert!(l.iter().any(|x| x.contains("Sword of Fire and Ice")), "{l:?}");
    // nothing else to do with the mana
    let mut g = game();
    lands(&mut g, P0, "Plains", 3, false);
    hand(&mut g, P0, &["Sword of Fire and Ice"]);
    g.player_mut(P0).turns = 3;
    let l = cast_labels(&mut g);
    assert!(l.iter().any(|x| x.contains("Sword of Fire and Ice")), "{l:?}");
}

// ---- fix 3: protection
const P1: PlayerId = PlayerId(1);

#[test]
fn plaza_of_heroes_protects_cloud() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 3, false);
    let pl = lands(&mut g, P0, "Plaza of Heroes", 1, false)[0];
    let cl = perm(&mut g, P0, CLOUD);
    let saved = sim_core::impls::cloud::cloud_protect(&mut g, P0, cl, "exile", Some(P1), None).unwrap();
    assert!(saved);
    assert!(sim_core::engine::values::untargetable(&g, cl));
    assert!(sim_core::engine::values::indestructible(&g, cl));
    assert!(!g.perm(pl).on_bf && !g.player(P0).lands.contains(&pl));
    assert!(g.player(P0).exile.contains(&g.perm(pl).card()));
    assert!(g.player(P0).lands.iter().all(|&l| g.perm(l).tapped), "paid {{3}} with the other lands");
}

#[test]
fn plaza_needs_three_other_mana() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 2, false);
    let pl = lands(&mut g, P0, "Plaza of Heroes", 1, false)[0];
    let cl = perm(&mut g, P0, CLOUD);
    let saved = sim_core::impls::cloud::cloud_protect(&mut g, P0, cl, "exile", Some(P1), None).unwrap();
    assert!(!saved);
    assert!(g.perm(pl).on_bf);
}

#[test]
fn formation_answers_a_destroy_wipe() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    for n in ["Unbreakable Formation", "Rootborn Defenses"] {
        let mut g = game();
        lands(&mut g, P0, "Plains", 3, false);
        let cl = perm(&mut g, P0, CLOUD);
        let k = perm(&mut g, P0, "Kor Duelist");
        let e = perm(&mut g, P0, "Sword of Fire and Ice");
        g.perm_mut(e).attached = Some(k);
        hand(&mut g, P0, &[n]);
        let r = sim_core::ai::wipe_response(&mut g, P0, "destroy".into(), P1).unwrap();
        assert_eq!(r.as_deref(), Some("indes"), "{n}");
        assert!(sim_core::engine::values::indestructible(&g, cl), "{n}");
        assert!(!g.player(P0).hand.iter().any(|&c| *g.db.get(c).name == *n), "{n} was cast");
        // an exile wipe is not answered by indestructible
        let mut g = game();
        lands(&mut g, P0, "Plains", 3, false);
        perm(&mut g, P0, CLOUD);
        hand(&mut g, P0, &[n]);
        assert!(sim_core::ai::wipe_response(&mut g, P0, "exile".into(), P1).unwrap().is_none());
    }
}

#[test]
fn giver_of_runes_answers_removal_on_cloud() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 3, false);
    let cl = perm(&mut g, P0, CLOUD);
    g.perm_mut(cl).is_cmd = true;
    let gv = perm(&mut g, P0, "Giver of Runes");
    let sp = card(&g, "Swords to Plowshares");
    let saved = sim_core::impls::cloud::cloud_protect(&mut g, P0, cl, "exile", Some(P1), Some(sp)).unwrap();
    assert!(saved, "Giver of Runes should answer");
    assert!(g.perm(gv).tapped);
}

// ---- fix 4: Cloud into open removal mana
fn open_removal(g: &mut Game) {
    let _ = &g;
    lands(g, P1, "Swamp", 4, false);
    // a fixed, removal-heavy library (independent of Sauron's current list), then a full hand of unknowns from it
    let mix = ["Go for the Throat", "Island", "Infernal Grasp", "Swamp", "Terminate", "Mountain", "Bitter Triumph", "Counterspell"];
    let lib: Vec<_> = (0..40).map(|i| card(g, mix[i % mix.len()])).collect();
    g.player_mut(P1).library = lib;
    for _ in 0..7 {
        let c = g.player_mut(P1).library.pop().unwrap();
        g.player_mut(P1).hand.push(c);
    }
}

fn cloud_bonus(g: &mut Game) -> f64 {
    let opts = sim_core::ai::brain::main_options(g, P0, false).unwrap();
    let o = opts.iter().find(|o| o.label == CLOUD).expect("Cloud castable");
    sim_core::impls::cloud::search_bonus(g, P0, o)
}

#[test]
fn giver_goes_first_into_open_removal() {
    assert!(sim_core::impls::cloud::PROTECT_FIRST);
    let mut g = sauron_table();
    open_removal(&mut g);
    assert!(sim_core::ai::brain::removal_risk(&g, P0) >= 0.3, "setup: removal likely");
    lands(&mut g, P0, "Plains", 2, false);
    hand(&mut g, P0, &["Giver of Runes"]);
    g.player_mut(P0).turns = 2;
    let plain = {
        // the same board with no Giver in hand
        let mut h = sauron_table();
        open_removal(&mut h);
        lands(&mut h, P0, "Plains", 2, false);
        h.player_mut(P0).turns = 2;
        cloud_bonus(&mut h)
    };
    let with_giver = cloud_bonus(&mut g);
    assert!(with_giver < plain, "Cloud waits for Giver: {with_giver} vs {plain}");
    let opts = sim_core::ai::brain::main_options(&mut g, P0, false).unwrap();
    let o = opts.iter().find(|o| o.label == "Giver of Runes").expect("Giver castable");
    assert!(sim_core::impls::cloud::search_bonus(&g, P0, o) > with_giver);
}

#[test]
fn recast_waits_for_protection_after_a_death() {
    let mut g = sauron_table();
    open_removal(&mut g);
    lands(&mut g, P0, "Plains", 5, false);
    g.player_mut(P0).turns = 6;
    g.player_mut(P0).tax = 2;
    let bare = cloud_bonus(&mut g);
    // Giver untapped: protection can follow
    let gv = perm(&mut g, P0, "Giver of Runes");
    g.perm_mut(gv).sick = false;
    let boots = cloud_bonus(&mut g);
    assert!(boots > bare, "{boots} vs {bare}");
}

fn sauron_table() -> Game {
    let mut g = table(&["cloud", "sauron"]);
    g.player_mut(P0).hand.clear();
    set_ai(&mut g, AiMode::Lookahead);
    g
}

// ---- fix 5: no ping-pong
#[test]
fn worn_equipment_moves_only_to_a_clearly_better_holder() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = game();
    lands(&mut g, P0, "Plains", 4, false);
    let cl = perm(&mut g, P0, CLOUD);
    g.perm_mut(cl).noatk = true; // out of the picture as a holder, but on the battlefield: moves are wanted
    let kd = perm(&mut g, P0, "Kor Duelist");
    let bg = perm(&mut g, P0, "Bronze Guardian");
    let mask = perm(&mut g, P0, "Mask of Memory");
    g.perm_mut(mask).attached = Some(kd);
    let offered = |g: &Game| sim_core::dsl::equip_options(g, P0).iter().any(|o| o.label == "equip Mask of Memory");
    assert!(!offered(&g), "equal holders: no move");
    g.perm_mut(bg).plus += 8;
    assert!(offered(&g), "a much bigger holder: move");
}

// ---- fix 6: mulligan
#[test]
fn cloud_keeps_only_playable_hands() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let (mut kept, mut sent_back) = (0, 0);
    for seed in 0..300u64 {
        let mut g = table(&["cloud", "seph"]);
        let mut r = sim_core::rng::Rng::from_u64(seed);
        sim_core::engine::turn::mulligan(&mut g, P0, &mut r);
        let pl = g.player(P0);
        if pl.stats.contains_key("mulls") {
            sent_back += 1;
            continue;
        }
        kept += 1;
        let lands = pl.hand.iter().filter(|&&c| g.db.get(c).land).count();
        assert!((2..=5).contains(&lands), "kept {lands} lands");
        assert!(sim_core::impls::cloud::keep_hand(&g, P0));
    }
    assert!(kept > 100 && sent_back > 0, "{kept} kept, {sent_back} mulliganed");
}

#[test]
fn keep_rule_by_hand() {
    let mut g = game();
    let keep = |g: &mut Game, names: &[&str]| {
        g.player_mut(P0).hand.clear();
        hand(g, P0, names);
        sim_core::impls::cloud::keep_hand(g, P0)
    };
    assert!(keep(
        &mut g,
        &["Plains", "Plains", "Plains", "Swords to Plowshares", "Sol Ring", "Sword of Fire and Ice", "Kor Duelist"]
    ));
    assert!(keep(
        &mut g,
        &[
            "Plains",
            "Plains",
            "Sol Ring",
            "Sword of Fire and Ice",
            "Sword of Hearth and Home",
            "Champion's Helm",
            "Kor Duelist"
        ]
    ));
    assert!(!keep(
        &mut g,
        &[
            "Plains",
            "Plains",
            "Sword of Fire and Ice",
            "Sword of Hearth and Home",
            "Champion's Helm",
            "Sword of Kaldra",
            "Colossus Hammer"
        ]
    ));
    assert!(!keep(
        &mut g,
        &[
            "Plains",
            "Sol Ring",
            "Sword of Fire and Ice",
            "Sword of Hearth and Home",
            "Champion's Helm",
            "Kor Duelist",
            "Colossus Hammer"
        ]
    ));
    assert!(!keep(&mut g, &["Plains", "Plains", "Plains", "Plains", "Plains", "Plains", "Sol Ring"]));
}

// ---- fix 7: wipes and edicts are logged
fn log_of(g: &mut Game, f: impl FnOnce(&mut Game)) -> Vec<String> {
    g.log = Some(vec![]);
    f(g);
    g.log.take().unwrap_or_default()
}

#[test]
fn wipes_and_edicts_name_what_dies() {
    let mut g = game();
    let cl = perm(&mut g, P0, CLOUD);
    let kd = perm(&mut g, P0, "Kor Duelist");
    let _ = (cl, kd);
    let l = log_of(&mut g, |g| {
        sim_core::engine::removal::apply_wipe(g, P1, "destroy".into(), None, &Default::default()).unwrap();
    });
    assert!(l.iter().any(|x| x.contains(CLOUD) && x.contains("removed by the wipe")), "{l:?}");
    assert!(l.iter().any(|x| x.contains("Kor Duelist") && x.contains("removed by the wipe")), "{l:?}");
    // an edict
    let mut g = game();
    perm(&mut g, P0, "Kor Duelist");
    let l = log_of(&mut g, |g| sim_core::engine::zones::edict(g, P0, false).unwrap());
    assert!(l.iter().any(|x| x.contains("sacrifices Kor Duelist (edict)")), "{l:?}");
}
