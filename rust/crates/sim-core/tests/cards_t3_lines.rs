//! Tier 3's combo and alternate-win lines (impls/t3_lines.rs) on hand-built positions; each expectation follows the
//! cards' Oracle text.

use sim_core::hooks::{Action, Opt};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::{combos, t3_lines};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const AURELIA: &str = "aurelia-boros-extra-combats";
const KORVOLD: &str = "korvold-jund-sacrifice";
const MARWYN: &str = "marwyn-mono-green-elves";
const TERGRID: &str = "tergrid-mono-black-disruption";
const ATRAXA: &str = "atraxa-infect";

fn tbl(first: &str) -> Game {
    let others: Vec<&str> = [AURELIA, KORVOLD, MARWYN, ATRAXA].into_iter().filter(|k| *k != first).take(3).collect();
    let mut keys = vec![first];
    keys.extend(others);
    let mut g = table(&keys);
    for p in g.players.iter_mut() {
        p.cmd_in_zone = false;
        p.hand.clear();
    }
    g
}

fn idx(part: &str) -> usize {
    combos::combo_index(part).unwrap_or_else(|| panic!("no combo named like {part:?}"))
}

// ======================================================== Helm of the Host on Aurelia / Combat Celebrant
fn helm_opts(g: &mut Game, helm: PermId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(helm).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, helm, P0, post).unwrap()
}

#[test]
fn aurelia_puts_helm_of_the_host_on_aurelia_before_combat() {
    let mut g = tbl(AURELIA);
    let helm = perm(&mut g, P0, "Helm of the Host");
    let valk = perm(&mut g, P0, "Youthful Valkyrie");
    let aurelia = perm(&mut g, P0, "Aurelia, the Warleader");
    g.perm_mut(helm).attached = Some(valk);
    lands(&mut g, P0, "Plains", 4, false);
    assert!(helm_opts(&mut g, helm, Some(false)).is_empty(), "four mana: no equip {{5}}");
    lands(&mut g, P0, "Plains", 1, false);
    let o = helm_opts(&mut g, helm, Some(false));
    assert_eq!(o.len(), 1);
    match o[0].act {
        Some(Action::Equip { target, n, .. }) => assert_eq!((target, n), (aurelia, 5)),
        _ => panic!("expected an equip"),
    }
    assert!(helm_opts(&mut g, helm, Some(true)).is_empty(), "after combat the copy is a turn away");
    g.perm_mut(helm).attached = Some(aurelia);
    assert!(helm_opts(&mut g, helm, Some(false)).is_empty(), "already on Aurelia");
}

#[test]
fn helm_goes_on_combat_celebrant_without_aurelia_and_only_for_the_aurelia_deck() {
    let mut g = tbl(AURELIA);
    let helm = perm(&mut g, P0, "Helm of the Host");
    let cel = perm(&mut g, P0, "Combat Celebrant");
    lands(&mut g, P0, "Plains", 5, false);
    let o = helm_opts(&mut g, helm, Some(false));
    assert!(matches!(o[0].act, Some(Action::Equip { target, .. }) if target == cel));
    g.player_mut(P0).key = "korvold-jund-sacrifice";
    assert!(helm_opts(&mut g, helm, Some(false)).is_empty());
}

#[test]
fn helm_on_aurelia_makes_a_combat_per_copy() {
    // the engine's own line: each hasty copy of Aurelia attacks for the first time and adds a combat
    let mut g = tbl(AURELIA);
    let helm = perm(&mut g, P0, "Helm of the Host");
    let aurelia = perm(&mut g, P0, "Aurelia, the Warleader");
    g.perm_mut(helm).attached = Some(aurelia);
    for q in 1..4 {
        g.players[q].life = 200;
    }
    sim_core::engine::combat::combat(&mut g, P0).unwrap();
    let copies = g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).token).count();
    assert!(copies >= 5, "{copies} copies: one per combat, the combats keep coming");
}

// ======================================================== Chatterfang + Pitiless Plunderer
fn chatter_board(g: &mut Game) {
    perm(g, P0, "Chatterfang, Squirrel General");
    perm(g, P0, "Pitiless Plunderer");
}

#[test]
fn chatterfang_plunderer_needs_an_outlet_a_payoff_and_a_creature() {
    let mut g = tbl(KORVOLD);
    let i = idx("Chatterfang + Pitiless Plunderer");
    chatter_board(&mut g);
    perm(&mut g, P0, "Viscera Seer");
    assert!(!combos::ready(&g, P0, i), "no payoff");
    perm(&mut g, P0, "Zulaport Cutthroat");
    assert!(!combos::ready(&g, P0, i), "nothing to sacrifice first");
    token(&mut g, P0, 1);
    assert!(combos::ready(&g, P0, i), "a creature dies: Treasure, Squirrel; the Squirrel dies: again");
}

#[test]
fn goblin_bombardment_is_outlet_and_payoff() {
    let mut g = tbl(KORVOLD);
    let i = idx("Chatterfang + Pitiless Plunderer");
    chatter_board(&mut g);
    perm(&mut g, P0, "Goblin Bombardment");
    token(&mut g, P0, 1);
    assert!(combos::ready(&g, P0, i), "each Squirrel sacrificed to Bombardment deals 1 and makes the next");
}

#[test]
fn with_korvold_out_the_loop_must_kill_before_the_library_runs_out() {
    let mut g = tbl(KORVOLD);
    let i = idx("Chatterfang + Pitiless Plunderer");
    chatter_board(&mut g);
    perm(&mut g, P0, "Viscera Seer");
    perm(&mut g, P0, "Blood Artist"); // one target per death: 120 life needs 120 Squirrels, 120 draws
    token(&mut g, P0, 1);
    perm(&mut g, P0, "Korvold, Fae-Cursed King");
    let lib: Vec<_> = g.player(P0).library.iter().copied().take(60).collect();
    g.player_mut(P0).library = lib;
    assert!(!combos::ready(&g, P0, i), "Korvold's draws deck p first");
    perm(&mut g, P0, "Zulaport Cutthroat"); // each opponent too: 40 Squirrels
    assert!(combos::ready(&g, P0, i), "Zulaport + Blood Artist kill in 40 draws");
}

// ======================================================== Staff of Domination + a big Elf
#[test]
fn staff_and_a_five_mana_elf_with_ezuri_in_the_library() {
    let mut g = tbl(MARWYN);
    let i = idx("Staff of Domination + an Elf");
    perm(&mut g, P0, "Staff of Domination");
    perm(&mut g, P0, "Priest of Titania");
    for n in ["Llanowar Elves", "Elvish Mystic", "Fyndhorn Elves"] {
        perm(&mut g, P0, n);
    }
    // four Elves on the battlefield (the Atraxa, Korvold and Aurelia seats have none out): Priest taps for 4
    assert!(!combos::ready(&g, P0, i), "Priest taps for four: {{3}} + {{1}} eats it all");
    perm(&mut g, P0, "Elvish Visionary");
    let ezuri_in_lib = g.player(P0).library.iter().any(|&c| &*g.db.get(c).name == "Ezuri, Renegade Leader");
    assert!(ezuri_in_lib);
    assert!(combos::ready(&g, P0, i), "five Elves: unbounded mana, the Staff draws Ezuri");
    g.player_mut(P0).library.clear();
    assert!(!combos::ready(&g, P0, i), "no finisher left to find");
}

// ======================================================== Aurelia: the Sword and Aggravated Assault
#[test]
fn aurelia_equips_sword_of_feast_and_famine() {
    let mut g = tbl(AURELIA);
    let sword = perm(&mut g, P0, "Sword of Feast and Famine");
    let aurelia = perm(&mut g, P0, "Aurelia, the Warleader");
    perm(&mut g, P0, "Mother of Runes");
    lands(&mut g, P0, "Plains", 2, false);
    let c = g.perm(sword).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    let opts: Vec<Opt> = f(&mut g, sword, P0, Some(false)).unwrap();
    assert_eq!(opts.len(), 1);
    match opts[0].act {
        Some(Action::Equip { target, n, .. }) => {
            assert_eq!(target, aurelia, "on the flier");
            assert_eq!(n, 2);
        }
        _ => panic!("expected an equip"),
    }
    let prio = g.registry.get(c).unwrap().prio.unwrap();
    assert!(prio(&g, P0, c) > 0, "cast by the Aurelia deck");
}

#[test]
fn aggravated_assault_adds_a_combat_and_loops_with_the_sword() {
    let mut g = tbl(AURELIA);
    perm(&mut g, P0, "Aggravated Assault");
    let aurelia = perm(&mut g, P0, "Aurelia, the Warleader");
    perm(&mut g, P0, "Hero of Bladehold");
    lands(&mut g, P0, "Mountain", 2, false);
    lands(&mut g, P0, "Plains", 3, false);
    assert!(t3_lines::assault_again(&mut g, P0, 2, &[]).unwrap(), "seven power, five mana: one more combat");
    assert!(g.player(P0).lands.iter().all(|&l| g.land(l).tapped), "{{3}}{{R}}{{R}} paid");
    for l in g.player(P0).lands.clone() {
        g.land_mut(l).tapped = false;
    }
    assert!(!t3_lines::assault_again(&mut g, P0, 3, &[]).unwrap(), "no Sword hit: once a turn");
    let sword = perm(&mut g, P0, "Sword of Feast and Famine");
    g.perm_mut(sword).attached = Some(aurelia);
    assert!(t3_lines::assault_again(&mut g, P0, 3, &[aurelia]).unwrap(), "the Sword connected: again");
    assert!(!t3_lines::assault_again(&mut g, P0, 4, &[aurelia]).unwrap(), "no mana left");
}

#[test]
fn assault_is_the_aurelia_decks_alone() {
    let mut g = tbl(KORVOLD);
    perm(&mut g, P0, "Aggravated Assault");
    lands(&mut g, P0, "Mountain", 5, false);
    token(&mut g, P0, 8);
    assert!(!t3_lines::assault_again(&mut g, P0, 1, &[]).unwrap());
}

// ======================================================== Tergrid: Torment of Hailfire
#[test]
fn tergrid_casts_torment_of_hailfire_for_the_kill() {
    let mut g = table(&[TERGRID, KORVOLD, MARWYN, ATRAXA]);
    for p in g.players.iter_mut() {
        p.cmd_in_zone = false;
        p.hand.clear();
    }
    let t = hand(&mut g, P0, &["Torment of Hailfire"])[0];
    lands(&mut g, P0, "Swamp", 8, false);
    for q in 1..4 {
        g.players[q].life = 9;
    }
    let f = g.registry.get(t).unwrap().hand_options.unwrap();
    let opts = f(&mut g, t, P0, Some(false)).unwrap();
    assert_eq!(opts.len(), 1, "X = 6: each opponent at 9 with nothing to give dies");
}
