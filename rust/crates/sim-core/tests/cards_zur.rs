//! zur.py's card code (`impls/zur.rs`): the Auras that lock a permanent down, The Eternal Wanderer, Prayer of
//! Binding, Rootborn Defenses, Azorius Guildmage's tap, Zur's untargeted Auras and the protection responses. Ported
//! from the Python suite's tests/test_zur.py (the practice-mode tests wait for phase 9); the seat is the Y'shtola
//! deck, which runs most of these cards.

use sim_core::cardcode;
use sim_core::engine::{cast, combat, mana, removal, turn, values, zones};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::zur;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn table3() -> Game {
    table(&["yshtola", "veyran", "sauron"])
}

/// your commander onto the battlefield, ready
fn cmd(g: &mut Game, p: PlayerId) -> PermId {
    let c = g.player(p).cmd;
    let m = zones::enter(g, p, c, zones::Enter { sick: false, ..Default::default() }).unwrap();
    g.perm_mut(m).is_cmd = true;
    g.player_mut(p).cmd_in_zone = false;
    m
}

/// cast a card from hand (already paid for), with the target chosen as it's cast; the permanent it became
fn cast_at(g: &mut Game, p: PlayerId, name: &str, target: Option<PermId>) -> Option<PermId> {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx { target, ..Ctx::default() }).unwrap();
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).cd == Some(c))
}

fn has_card_on_bf(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == name))
}

/// a permanent's own options hook
fn opts(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.perm(src).cd.unwrap()).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    match o.act.clone() {
        Some(Action::Ability { src, f, arg }) => f(g, src, p, arg).unwrap(),
        _ => panic!("not an ability"),
    }
}

// ------------------------------------------------------------------ the lock Auras
#[test]
fn arrest_stops_attacks_blocks_and_abilities_until_it_leaves() {
    let mut g = table3();
    let gm = perm(&mut g, P1, "Harmonic Prodigy");
    let a = cast_at(&mut g, P0, "Arrest", Some(gm)).expect("Arrest on the battlefield");
    assert_eq!(g.perm(a).attached, Some(gm));
    assert!(cardcode::locked(&g, gm, "pacify"));
    let y = cmd(&mut g, P0);
    assert!(!combat::can_block(&g, gm, y));
    assert!(cardcode::ability_locked(&g, gm, P1));
    zones::die(&mut g, a, "destroy").unwrap(); // the Aura goes: the creature is free again
    assert!(!cardcode::locked(&g, gm, "pacify"));
    assert!(!cardcode::ability_locked(&g, gm, P1));
}

#[test]
fn luminous_bonds_leaves_abilities_alone() {
    let mut g = table3();
    let m = perm(&mut g, P1, "Guttersnipe");
    cast_at(&mut g, P0, "Luminous Bonds", Some(m));
    assert!(cardcode::locked(&g, m, "pacify"));
    assert!(!cardcode::locked(&g, m, "noact"));
    assert!(!cardcode::locked(&g, m, "frozen"));
}

#[test]
fn encrust_stops_mana_and_untapping() {
    let mut g = table3();
    let rock = perm(&mut g, P1, "Sol Ring");
    let before = mana::total_mana(&g, P1, false);
    cast_at(&mut g, P0, "Encrust", Some(rock));
    assert_eq!(mana::total_mana(&g, P1, false), before - 2, "its mana ability can't be activated");
    let cr = perm(&mut g, P2, "Orcish Bowmasters");
    g.perm_mut(cr).tapped = true;
    cast_at(&mut g, P0, "Encrust", Some(cr)); // (a second copy, from the card database)
    turn::step_start(&mut g, P2).unwrap();
    assert!(g.perm(cr).tapped, "doesn't untap");
}

#[test]
fn a_locked_creature_is_worth_less() {
    let mut g = table3();
    let m = perm(&mut g, P1, "Niv-Mizzet, Parun");
    let before = values::pval(&g, m);
    cast_at(&mut g, P0, "Arrest", Some(m));
    assert!(values::pval(&g, m) < before * 0.5, "{} vs {before}", values::pval(&g, m));
    assert!((zur::lock_factor(&g, m) - 0.3).abs() < 1e-9);
    let ps = card(&g, "Prison Sentence");
    assert_eq!(removal::legal_targets(&g, P0, "arrest", "c", false, Some(ps)), vec![], "not twice");
}

#[test]
fn no_target_no_aura() {
    let mut g = table3();
    let c = hand(&mut g, P0, &["Bound in Silence"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert!(g.player(P0).gy.contains(&c));
    assert!(!has_card_on_bf(&g, P0, "Bound in Silence"));
}

// ------------------------------------------------------------------ Zur's untargeted Auras
#[test]
fn a_fetched_lock_aura_ignores_hexproof() {
    let mut g = table3();
    let z = perm(&mut g, P0, "Zur the Enchanter");
    let big = perm(&mut g, P1, "Niv-Mizzet, Parun");
    g.perm_mut(big).eot_kw.push("hexproof");
    assert!(values::untargetable(&g, big));
    assert_eq!(zur::lock_host(&g, P0, "Arrest", true, &[]), None, "a cast Arrest can't target it");
    assert_eq!(zur::lock_host(&g, P0, "Arrest", false, &[]), Some(big));
    let arrest = take(&mut g, P0, "Arrest");
    zur::put_enchantment(&mut g, P0, arrest, z).unwrap();
    assert!(cardcode::locked(&g, big, "pacify"));
}

#[test]
fn an_aura_with_nothing_to_lock_goes_to_the_graveyard() {
    let mut g = table3();
    let z = perm(&mut g, P0, "Zur the Enchanter");
    let bonds = take(&mut g, P0, "Luminous Bonds");
    zur::put_enchantment(&mut g, P0, bonds, z).unwrap();
    assert!(g.player(P0).gy.contains(&bonds));
    assert!(!has_card_on_bf(&g, P0, "Luminous Bonds"));
}

#[test]
fn lock_worth_by_kind() {
    let mut g = table3();
    let rock = perm(&mut g, P1, "Sol Ring");
    let cr = perm(&mut g, P1, "Guttersnipe");
    assert!(zur::activated(&g, rock));
    assert!(!zur::activated(&g, cr));
    assert_eq!(zur::lock_worth(&g, P0, rock, "pacify"), 0.0, "not a creature");
    assert_eq!(zur::lock_worth(&g, P0, rock, "encrust"), 0.0, "a Sol Ring is worth 0 to remove (as in the Python)");
    let v = values::pval(&g, cr);
    let p = values::epow(&g, cr).min(4) as f64;
    assert!((zur::lock_worth(&g, P0, cr, "arrest") - v * (0.55 + 0.08 * p)).abs() < 1e-9);
    assert!((zur::lock_worth(&g, P0, cr, "arrest") - 2.84).abs() < 1e-9, "the Python's number");
    assert_eq!(zur::lock_host(&g, P0, "Encrust", true, &[]), Some(cr));
}

// ------------------------------------------------------------------ the other cards
#[test]
fn bastion_protector_guards_only_the_commander() {
    let mut g = table3();
    let y = cmd(&mut g, P0);
    let bp = perm(&mut g, P0, "Bastion Protector");
    assert_eq!((values::epow(&g, y), values::etgh(&g, y)), (4, 6)); // Y'shtola is 2/4
    assert!(values::indestructible(&g, y));
    assert!(!values::indestructible(&g, bp));
}

#[test]
fn the_eternal_wanderer() {
    let mut g = table3();
    let w = perm(&mut g, P0, "The Eternal Wanderer");
    assert_eq!(g.perm(w).loyalty, Some(5));
    let big = perm(&mut g, P1, "Niv-Mizzet, Parun");
    zur::wanderer_exile(&mut g, P0, big).unwrap();
    assert!(!g.perm(big).on_bf);
    turn::end_step(&mut g, P2).unwrap(); // not its owner's end step
    assert!(!has_card_on_bf(&g, P1, "Niv-Mizzet, Parun"));
    turn::end_step(&mut g, P1).unwrap();
    assert!(has_card_on_bf(&g, P1, "Niv-Mizzet, Parun"));
    assert!(g.zur_due.is_empty());
    // 0: a 2/2 double-strike Samurai
    let o = opts(&mut g, w, P0, Some(false));
    let zero = o.iter().find(|x| x.label == "The Eternal Wanderer 0 (Samurai)").unwrap().clone();
    assert!(run(&mut g, P0, &zero));
    let sam: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token).collect();
    assert_eq!(sam.len(), 1);
    assert!(sim_core::dsl::has_kw(&g, sam[0], "double strike"));
    assert!(opts(&mut g, w, P0, Some(false)).is_empty(), "one ability a turn");
    // -4: each player keeps one creature
    let y = cmd(&mut g, P0);
    perm(&mut g, P0, "Esper Sentinel");
    perm(&mut g, P1, "Guttersnipe");
    zur::wanderer_ult(&mut g, P0).unwrap();
    let mine: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&m| g.is_creature(m)).collect();
    assert_eq!(mine, vec![y], "you keep your commander");
    assert_eq!(g.player(P1).perms.iter().filter(|&&m| g.is_creature(m)).count(), 1);
}

#[test]
fn the_wanderer_plus_one_exiles_the_best_target() {
    let mut g = table3();
    let w = perm(&mut g, P0, "The Eternal Wanderer");
    let big = perm(&mut g, P1, "Niv-Mizzet, Parun");
    let o = opts(&mut g, w, P0, Some(false));
    assert_eq!(o[0].label, "The Eternal Wanderer +1 (exile Niv-Mizzet, Parun)");
    assert!(run(&mut g, P0, &o[0]));
    assert!(!g.perm(big).on_bf);
    assert_eq!(g.perm(w).loyalty, Some(6));
    assert_eq!(g.zur_due.len(), 1);
}

#[test]
fn prayer_of_binding_until_it_leaves() {
    let mut g = table3();
    let big = perm(&mut g, P1, "Niv-Mizzet, Parun");
    let life = g.player(P0).life;
    let pr = perm(&mut g, P0, "Prayer of Binding");
    assert!(!g.perm(big).on_bf);
    assert_eq!(g.player(P0).life, life + 2);
    zones::die(&mut g, pr, "destroy").unwrap();
    assert!(has_card_on_bf(&g, P1, "Niv-Mizzet, Parun"));
}

#[test]
fn disenchant_targets() {
    let mut g = table3();
    let big = perm(&mut g, P1, "Niv-Mizzet, Parun");
    let rock = perm(&mut g, P1, "Sol Ring");
    let dis = removal::legal_targets(&g, P0, "destroy", "ae", false, Some(card(&g, "Disenchant")));
    assert!(dis.contains(&rock));
    assert!(!dis.contains(&big));
}

#[test]
fn rootborn_defenses_against_a_wipe() {
    let mut g = table3();
    lands(&mut g, P0, "Plains", 3, false);
    let y = cmd(&mut g, P0);
    perm(&mut g, P0, "Restoration Angel");
    hand(&mut g, P0, &["Rootborn Defenses"]);
    assert_eq!(zur::zur_wipe_response(&mut g, P0, "destroy").unwrap(), Some("indes"));
    assert!(values::indestructible(&g, y));
}

#[test]
fn rootborn_defenses_answers_removal_and_populates() {
    let mut g = table3();
    lands(&mut g, P0, "Plains", 3, false);
    let y = cmd(&mut g, P0);
    let t = token(&mut g, P0, 2);
    hand(&mut g, P0, &["Rootborn Defenses"]);
    assert_eq!(values::pval(&g, y), 4.0, "worth protecting (4 or more), as in the Python");
    assert!(zur::zur_protect(&mut g, P0, y, "destroy", Some(P1), None).unwrap());
    assert!(values::indestructible(&g, y));
    let toks: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token).collect();
    assert_eq!(toks.len(), 2, "populate copied the token");
    assert!(values::indestructible(&g, t));
}

#[test]
fn azorius_guildmage_taps_the_blocker_that_would_win() {
    let mut g = table3();
    lands(&mut g, P0, "Plains", 1, false);
    lands(&mut g, P0, "Island", 2, false);
    let gm = perm(&mut g, P0, "Azorius Guildmage");
    let y = cmd(&mut g, P0);
    let blocker = token(&mut g, P1, 5);
    assert_eq!(zur::guildmage_target(&g, P0), Some(blocker));
    assert!(opts(&mut g, gm, P0, Some(true)).is_empty(), "only before combat");
    let o = opts(&mut g, gm, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, P0, &o[0]));
    assert!(g.perm(blocker).tapped);
    let _ = y;
}

#[test]
fn shroud_stops_your_own_targeting() {
    let mut g = table3();
    let y = cmd(&mut g, P0);
    assert!(!zur::untargetable_by_you(&g, y));
    let gr = perm(&mut g, P0, "Lightning Greaves");
    g.perm_mut(gr).attached = Some(y);
    assert!(zur::untargetable_by_you(&g, y));
}
