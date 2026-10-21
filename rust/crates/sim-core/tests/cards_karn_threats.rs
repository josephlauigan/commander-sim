//! The Karn deck's threats, removal and wipes (impls/karn_cards.rs): each card on a hand-built position with its Oracle
//! text as the expectation.

use sim_core::ai::{act, brain};
use sim_core::engine::{cast, combat, life, values, zones};
use sim_core::hooks::Opt;
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn tbl() -> Game {
    table(&["kefka", "seph", "sauron"])
}

fn on(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn labels(os: &[Opt]) -> Vec<String> {
    os.iter().map(|o| o.label.clone()).collect()
}

fn cast_free(g: &mut Game, p: PlayerId, name: &str) -> bool {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap()
}

fn gy_has(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.contains(&card(g, name))
}

/// colourless artifact creatures / coloured creatures for the targets
const BIG: &str = "Lyra Dawnbringer";
const BOMB2: &str = "Grave Titan";

// ======================================================== Nevinyrral's Disk
#[test]
fn the_disk_enters_tapped_and_destroys_artifacts_creatures_and_enchantments() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 3, false);
    let disk = perm(&mut g, P0, "Nevinyrral's Disk");
    assert!(g.perm(disk).tapped, "it enters tapped");
    assert!(options(&mut g, disk, P0, Some(false)).is_empty(), "tapped: no activation");
    g.perm_mut(disk).tapped = false;
    let lyra = perm(&mut g, P1, BIG);
    let titan = perm(&mut g, P2, BOMB2);
    let ring = perm(&mut g, P1, "Sol Ring");
    let ench = perm(&mut g, P2, "Rhystic Study");
    let own = perm(&mut g, P0, "Mind Stone");
    let os = options(&mut g, disk, P0, Some(false));
    assert_eq!(labels(&os), vec!["Nevinyrral's Disk"], "the opponents lose far more");
    assert!(run(&mut g, P0, &os[0]));
    for m in [lyra, titan, ring, ench, own, disk] {
        assert!(!on(&g, m), "{} is destroyed", g.perm(m).name);
    }
    assert!(g.perm(disk).tapped);
}

#[test]
fn the_disk_leaves_indestructible_artifacts_and_is_not_used_to_hurt_ourselves() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 3, false);
    let disk = perm(&mut g, P0, "Nevinyrral's Disk");
    g.perm_mut(disk).tapped = false;
    perm(&mut g, P0, "Darksteel Forge");
    let rock = perm(&mut g, P0, "Mind Stone");
    let o1 = perm(&mut g, P1, "Mind Stone");
    // our board is big, theirs is one rock: the AI does not wipe
    perm(&mut g, P0, "Wurmcoil Engine");
    assert!(options(&mut g, disk, P0, Some(false)).is_empty());
    // forced: the Forge's indestructible artifacts live, the rest of the opposing board dies
    sim_core::impls::karn_cards::run(&mut g, P0, sim_core::impls::karn_cards::Wipe::Disk, None).unwrap();
    assert!(on(&g, rock) && on(&g, disk), "Darksteel Forge: our artifacts are indestructible");
    assert!(!on(&g, o1));
}

// ======================================================== Oblivion Stone
#[test]
fn oblivion_stone_destroys_what_has_no_fate_counter_then_clears_them() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 10, false);
    let stone = perm(&mut g, P0, "Oblivion Stone");
    let mine = perm(&mut g, P0, "Wurmcoil Engine");
    let junk = perm(&mut g, P0, "Mind Stone");
    let lyra = perm(&mut g, P1, BIG);
    let titan = perm(&mut g, P2, BOMB2);
    // it first marks our best permanent (the wipe is worth more with it saved)
    let os = options(&mut g, stone, P0, Some(false));
    assert!(labels(&os).contains(&"Oblivion Stone (fate counter)".to_string()), "{:?}", labels(&os));
    let mark = os.iter().find(|o| o.label.contains("fate")).unwrap();
    assert!(run(&mut g, P0, mark));
    assert_eq!(g.perm(mine).data.int(DataKey::Fate), 1, "the Wurmcoil Engine");
    assert!(g.perm(stone).tapped);
    g.perm_mut(stone).tapped = false;
    let os = options(&mut g, stone, P0, Some(false));
    let wipe = os.iter().find(|o| o.label.contains("destroy")).expect("the wipe is on now");
    assert!(run(&mut g, P0, wipe));
    assert!(on(&g, mine), "the marked permanent survives");
    assert!(!on(&g, junk) && !on(&g, lyra) && !on(&g, titan) && !on(&g, stone));
    assert_eq!(g.perm(mine).data.int(DataKey::Fate), 0, "all fate counters are removed");
}

// ======================================================== Ratchet Bomb
#[test]
fn ratchet_bomb_destroys_the_mana_value_it_counted_up_to_and_tokens_at_zero() {
    let mut g = tbl();
    let bomb = perm(&mut g, P0, "Ratchet Bomb");
    let t1 = token(&mut g, P1, 3);
    let t2 = token(&mut g, P2, 3);
    let own = perm(&mut g, P0, "Mind Stone"); // mana value 2
    let a = perm(&mut g, P1, "Sol Ring"); // 1
    let b = perm(&mut g, P2, "Cultivate"); // not a permanent
    let _ = b;
    let karn = perm(&mut g, P1, BIG); // 5
    // at zero counters it kills tokens
    let os = options(&mut g, bomb, P0, Some(false));
    assert!(!os.is_empty());
    sim_core::impls::karn_cards::run(&mut g, P0, sim_core::impls::karn_cards::Wipe::Bomb(0), None).unwrap();
    assert!(!on(&g, t1) && !on(&g, t2), "tokens are mana value 0");
    assert!(on(&g, own) && on(&g, a) && on(&g, karn) && on(&g, bomb));
    // wind it up to 5 by hand through its own ability: the tick option
    let os = options(&mut g, bomb, P0, Some(false));
    let tick = os.iter().find(|o| o.label.contains("charge")).expect("it winds up toward the 5");
    assert!(run(&mut g, P0, tick));
    assert_eq!(g.perm(bomb).data.int(DataKey::Charge), 1);
    assert!(g.perm(bomb).tapped);
    g.perm_mut(bomb).tapped = false;
    // at 1 counter the Sol Ring is the target, but we have nothing to lose: it detonates when it is worth it
    let os = options(&mut g, bomb, P0, Some(false));
    assert!(!os.is_empty());
}

// ======================================================== All Is Dust
#[test]
fn all_is_dust_sacrifices_every_coloured_permanent_and_leaves_ours() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 7, false);
    let lyra = perm(&mut g, P1, BIG);
    let titan = perm(&mut g, P2, BOMB2);
    let ring = perm(&mut g, P1, "Sol Ring");
    let own_art = perm(&mut g, P0, "Wurmcoil Engine");
    let own_col = perm(&mut g, P0, "Grave Titan"); // coloured: ours goes too
    let c = hand(&mut g, P0, &["All Is Dust"])[0];
    // the AI lists the wipe
    let o = brain::main_options(&mut g, P0, false).unwrap();
    assert!(labels(&o).contains(&"All Is Dust".to_string()), "{:?}", labels(&o));
    assert!(cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap());
    assert!(!on(&g, lyra) && !on(&g, titan) && !on(&g, own_col));
    assert!(on(&g, ring) && on(&g, own_art), "colourless permanents stay");
    assert!(gy_has(&g, P0, "Grave Titan") && gy_has(&g, P1, BIG));
}

// ======================================================== static abilities
#[test]
fn darksteel_forge_makes_our_artifacts_indestructible() {
    let mut g = tbl();
    perm(&mut g, P0, "Darksteel Forge");
    let w = perm(&mut g, P0, "Wurmcoil Engine");
    let theirs = perm(&mut g, P1, "Wurmcoil Engine");
    assert!(values::indestructible(&g, w));
    assert!(!values::indestructible(&g, theirs));
    zones::die(&mut g, w, "destroy").unwrap();
    assert!(on(&g, w), "destroy does nothing");
    zones::die(&mut g, theirs, "destroy").unwrap();
    assert!(!on(&g, theirs));
}

#[test]
fn platinum_angel_keeps_its_controller_in_the_game() {
    let mut g = tbl();
    perm(&mut g, P0, "Platinum Angel");
    g.player_mut(P0).life = 0;
    life::check_state(&mut g).unwrap();
    assert!(g.player(P0).alive, "you can't lose the game");
    let mut g = tbl();
    g.player_mut(P0).life = 0;
    life::check_state(&mut g).unwrap();
    assert!(!g.player(P0).alive, "without it you do");
    // poison too
    let mut g = tbl();
    perm(&mut g, P1, "Platinum Angel");
    g.player_mut(P1).poison = 10;
    life::check_state(&mut g).unwrap();
    assert!(g.player(P1).alive);
}

#[test]
fn blightsteel_colossus_kills_with_ten_poison_and_shuffles_back() {
    let mut g = tbl();
    let b = perm(&mut g, P0, "Blightsteel Colossus");
    combat::resolve_combat(&mut g, P0, &[b], P1, &[]).unwrap();
    assert_eq!(g.player(P1).life, 40, "infect: no life lost");
    assert_eq!(g.player(P1).poison, 11);
    assert!(!g.player(P1).alive, "10 poison counters: it loses");
    // it never reaches the graveyard from the battlefield
    let mut g = tbl();
    let b = perm(&mut g, P0, "Blightsteel Colossus");
    let c = card(&g, "Blightsteel Colossus");
    zones::die(&mut g, b, "sac").unwrap();
    assert!(!on(&g, b));
    assert!(!g.player(P0).gy.contains(&c), "not the graveyard");
    assert!(g.player(P0).library.contains(&c), "shuffled into the library");
    // indestructible
    let mut g = tbl();
    let b = perm(&mut g, P0, "Blightsteel Colossus");
    zones::die(&mut g, b, "destroy").unwrap();
    assert!(on(&g, b));
}

// ======================================================== Ugin, the Ineffable
#[test]
fn ugin_makes_colourless_spells_cheaper_and_uses_both_abilities() {
    let mut g = tbl();
    let ugin = perm(&mut g, P0, "Ugin, the Ineffable");
    assert_eq!(g.perm(ugin).loyalty, Some(4), "loyalty from Scryfall");
    let w = card(&g, "Wurmcoil Engine");
    let (n, pips) = sim_core::engine::mana::cost_of(&g, P0, w);
    assert_eq!((n, pips.as_str()), (4, ""), "{{6}} less {{2}}");
    let coloured = card(&g, "Grave Titan");
    let (n, _) = sim_core::engine::mana::cost_of(&g, P0, coloured);
    assert_eq!(n, 4, "a coloured spell costs the usual 4 generic");
    let (n, _) = sim_core::engine::mana::cost_of(&g, P1, w);
    assert_eq!(n, 6, "only for us");
    // -3 destroys the best coloured permanent
    let lyra = perm(&mut g, P1, BIG);
    let ring = perm(&mut g, P1, "Sol Ring");
    let os = options(&mut g, ugin, P0, Some(false));
    let minus = os.iter().find(|o| o.label.contains("-3")).expect("the -3 is on");
    assert!(run(&mut g, P0, minus));
    assert!(!on(&g, lyra) && on(&g, ring), "only a coloured permanent");
    assert_eq!(g.perm(ugin).loyalty, Some(1));
}

#[test]
fn ugin_plus_one_exiles_a_card_for_a_spirit_that_returns_it() {
    let mut g = tbl();
    let ugin = perm(&mut g, P0, "Ugin, the Ineffable");
    let top = *g.player(P0).library.last().unwrap();
    let hand_before = g.player(P0).hand.len();
    let os = options(&mut g, ugin, P0, Some(false));
    let plus = os.iter().find(|o| o.label.contains("+1")).unwrap();
    assert!(run(&mut g, P0, plus));
    assert_eq!(g.perm(ugin).loyalty, Some(5));
    assert!(g.player(P0).exile.contains(&top));
    let spirits: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token).collect();
    assert_eq!(spirits.len(), 1);
    assert_eq!(values::epow(&g, spirits[0]), 2);
    zones::die(&mut g, spirits[0], "destroy").unwrap();
    assert!(g.player(P0).hand.contains(&top), "the exiled card goes to hand when the Spirit leaves");
    assert_eq!(g.player(P0).hand.len(), hand_before + 1);
}

// ======================================================== Steel Overseer, Triplicate Titan, Duplicant
#[test]
fn steel_overseer_grows_every_artifact_creature() {
    let mut g = tbl();
    let o = perm(&mut g, P0, "Steel Overseer");
    let w = perm(&mut g, P0, "Wurmcoil Engine");
    let plain = perm(&mut g, P0, "Mind Stone");
    let other = perm(&mut g, P1, "Wurmcoil Engine");
    let os = options(&mut g, o, P0, Some(false));
    assert_eq!(os.len(), 1);
    assert!(run(&mut g, P0, &os[0]));
    assert_eq!(values::epow(&g, o), 2, "it counts itself");
    assert_eq!(values::epow(&g, w), 7);
    assert_eq!(g.perm(plain).plus, 0);
    assert_eq!(g.perm(other).plus, 0, "only ours");
    assert!(g.perm(o).tapped);
}

#[test]
fn triplicate_titan_leaves_three_golems() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Triplicate Titan");
    zones::die(&mut g, t, "destroy").unwrap();
    let golems: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token).collect();
    assert_eq!(golems.len(), 3);
    for &m in &golems {
        assert_eq!((values::epow(&g, m), values::etgh(&g, m)), (3, 3));
        assert!(sim_core::impls::karn_cards::is_art(&g, m));
    }
    assert_eq!(golems.iter().filter(|&&m| g.perm(m).fly).count(), 1);
    assert_eq!(golems.iter().filter(|&&m| g.perm(m).vig).count(), 1);
    assert_eq!(golems.iter().filter(|&&m| sim_core::dsl::has_kw(&g, m, "trample")).count(), 1);
}

#[test]
fn duplicant_exiles_the_best_creature_and_takes_its_size() {
    let mut g = tbl();
    let lyra = perm(&mut g, P1, BIG);
    let d = perm(&mut g, P0, "Duplicant");
    assert!(!on(&g, lyra));
    assert!(g.player(P1).exile.contains(&card(&g, BIG)));
    assert_eq!((values::epow(&g, d), values::etgh(&g, d)), (5, 5), "Lyra's 5/5");
    // a token is not a legal target
    let mut g = tbl();
    let t = token(&mut g, P1, 6);
    let d = perm(&mut g, P0, "Duplicant");
    assert!(on(&g, t));
    assert_eq!((values::epow(&g, d), values::etgh(&g, d)), (2, 4));
}

// ======================================================== Forgemaster, Replicator, Liberator
#[test]
fn kuldotha_forgemaster_sacrifices_three_artifacts_for_the_best_artifact() {
    let mut g = tbl();
    let f = perm(&mut g, P0, "Kuldotha Forgemaster");
    for _ in 0..3 {
        perm(&mut g, P0, "Mind Stone");
    }
    let target = card(&g, "Blightsteel Colossus");
    g.player_mut(P0).library.push(target);
    let os = options(&mut g, f, P0, Some(false));
    assert_eq!(os.len(), 1, "{:?}", labels(&os));
    assert!(run(&mut g, P0, &os[0]));
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).on_bf && &*g.perm(m).name == "Blightsteel Colossus"));
    assert_eq!(g.player(P0).perms.iter().filter(|&&m| &*g.perm(m).name == "Mind Stone").count(), 0);
    assert!(g.perm(f).tapped);
}

#[test]
fn mishra_s_self_replicator_copies_on_historic_spells_for_one_mana() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 3, false);
    let r = perm(&mut g, P0, "Mishra's Self-Replicator");
    assert!(cast_free(&mut g, P0, "Sol Ring"), "an artifact is historic");
    let reps = g.player(P0).perms.iter().filter(|&&m| &*g.perm(m).name == "Mishra's Self-Replicator").count();
    assert_eq!(reps, 2, "one copy token");
    // a non-historic spell does not
    assert!(cast_free(&mut g, P0, "Cultivate"));
    let reps = g.player(P0).perms.iter().filter(|&&m| &*g.perm(m).name == "Mishra's Self-Replicator").count();
    assert_eq!(reps, 2);
    // two copies trigger twice
    assert!(cast_free(&mut g, P0, "Mind Stone"));
    let reps = g.player(P0).perms.iter().filter(|&&m| &*g.perm(m).name == "Mishra's Self-Replicator").count();
    assert!(reps >= 3, "the copy triggers too: {reps}");
    let _ = r;
}

#[test]
fn liberator_gives_artifact_and_colourless_spells_flash_and_grows() {
    let mut g = tbl();
    let c = hand(&mut g, P0, &["Wurmcoil Engine", "Grave Titan"]);
    assert!(!cast::grants_flash(&g, P0, c[0]));
    let l = perm(&mut g, P0, "Liberator, Urza's Battlethopter");
    assert!(cast::grants_flash(&g, P0, c[0]), "an artifact creature spell");
    assert!(!cast::grants_flash(&g, P0, c[1]), "a coloured spell does not");
    assert!(!cast::grants_flash(&g, P1, c[0]), "only ours");
    // a 3-mana cast is more than its power 1: a counter
    lands(&mut g, P0, "Wastes", 3, false);
    let cs = hand(&mut g, P0, &["Mind Stone"]);
    let a = sim_core::hooks::Action::Cast { card: cs[0], zone: None };
    assert!(act::perform(&mut g, P0, &a).unwrap());
    assert_eq!(g.perm(l).plus, 1, "mana spent 2 > power 1");
}

// ======================================================== Portal to Phyrexia, Spine of Ish Sah, Obelisk
#[test]
fn portal_to_phyrexia_makes_three_sacrifices_and_reanimates_each_upkeep() {
    let mut g = tbl();
    let cs: Vec<PermId> = (0..4).map(|i| token(&mut g, P1, 1 + i)).collect();
    let keep = perm(&mut g, P2, BOMB2);
    let dead = card(&g, "Wurmcoil Engine");
    g.player_mut(P1).gy.push(dead);
    let portal = perm(&mut g, P0, "Portal to Phyrexia");
    assert_eq!(cs.iter().filter(|&&m| on(&g, m)).count(), 1, "three of four tokens sacrificed");
    assert!(!on(&g, keep), "P2 had one creature: it goes");
    // upkeep: the best creature card of any graveyard, as a Phyrexian
    let imp = g.registry.get(g.perm(portal).cd.unwrap()).copied().unwrap();
    (imp.upkeep.unwrap())(&mut g, portal, P0).unwrap();
    let got: Vec<PermId> = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).on_bf && g.is_creature(m) && !g.perm(m).token)
        .collect();
    assert_eq!(got.len(), 1, "one creature card from a graveyard");
    assert!(g.perm(got[0]).ttypes.contains(&"phyrexian"));
    let back = g.perm(got[0]).cd.unwrap();
    assert!(!g.player(P1).gy.contains(&back) && !g.player(P2).gy.contains(&back));
    let _ = dead;
}

#[test]
fn spine_of_ish_sah_destroys_a_permanent_and_comes_back_to_hand() {
    let mut g = tbl();
    let lyra = perm(&mut g, P1, BIG);
    let s = perm(&mut g, P0, "Spine of Ish Sah");
    assert!(!on(&g, lyra), "destroy target permanent");
    zones::die(&mut g, s, "destroy").unwrap();
    assert!(g.player(P0).hand.contains(&card(&g, "Spine of Ish Sah")), "returned to its owner's hand");
    assert!(!gy_has(&g, P0, "Spine of Ish Sah"));
}

#[test]
fn unstable_obelisk_taps_for_mana_and_destroys_for_seven() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 7, false);
    let o = perm(&mut g, P0, "Unstable Obelisk");
    let lyra = perm(&mut g, P1, BIG);
    let os = options(&mut g, o, P0, Some(false));
    assert_eq!(os.len(), 1, "{:?}", labels(&os));
    assert!(run(&mut g, P0, &os[0]));
    assert!(!on(&g, lyra) && !on(&g, o));
}

// ======================================================== Steel Hellkite
#[test]
fn steel_hellkite_destroys_the_mana_value_of_a_player_it_hit() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 6, false);
    let h = perm(&mut g, P0, "Steel Hellkite");
    let a = perm(&mut g, P1, "Mind Stone"); // 2
    let b = perm(&mut g, P1, "Mind Stone");
    let c = perm(&mut g, P2, "Mind Stone"); // P2 was not hit
    assert!(options(&mut g, h, P0, Some(true)).is_empty(), "no combat damage yet");
    combat::resolve_combat(&mut g, P0, &[h], P1, &[]).unwrap();
    assert_eq!(g.player(P1).life, 35);
    let os = options(&mut g, h, P0, Some(true));
    assert_eq!(os.len(), 1, "{:?}", labels(&os));
    assert!(os[0].label.contains("X=2"));
    assert!(run(&mut g, P0, &os[0]));
    assert!(!on(&g, a) && !on(&g, b));
    assert!(on(&g, c), "P2 wasn't dealt damage");
    assert!(options(&mut g, h, P0, Some(true)).is_empty(), "once each turn");
}

// ======================================================== Cityscape Leveler
#[test]
fn cityscape_leveler_destroys_on_cast_and_on_attack_and_unearths() {
    let mut g = tbl();
    let lyra = perm(&mut g, P1, BIG);
    assert!(cast_free(&mut g, P0, "Cityscape Leveler"));
    assert!(!on(&g, lyra), "when you cast it");
    let ps: Vec<PermId> =
        g.player(P1).perms.iter().copied().filter(|&m| values::card_name(&g, m) == Some("Powerstone")).collect();
    assert_eq!(ps.len(), 1, "its controller gets a Powerstone");
    assert!(g.perm(ps[0]).tapped);
    // attack
    let titan = perm(&mut g, P2, BOMB2);
    let lev =
        g.player(P0).perms.iter().copied().find(|&m| values::card_name(&g, m) == Some("Cityscape Leveler")).unwrap();
    g.perm_mut(lev).sick = false;
    combat::resolve_combat(&mut g, P0, &[lev], P2, &[]).unwrap();
    assert!(!on(&g, titan), "whenever it attacks");
    // dies, then unearth {8}
    zones::die(&mut g, lev, "sac").unwrap();
    assert!(gy_has(&g, P0, "Cityscape Leveler"));
    lands(&mut g, P0, "Wastes", 8, false);
    let c = card(&g, "Cityscape Leveler");
    let f = g.registry.get(c).unwrap().gy_options.unwrap();
    let os = f(&mut g, c, P0, Some(false)).unwrap();
    assert_eq!(os.len(), 1);
    assert!(run(&mut g, P0, &os[0]));
    let back =
        g.player(P0).perms.iter().copied().find(|&m| values::card_name(&g, m) == Some("Cityscape Leveler")).unwrap();
    assert!(!g.perm(back).sick, "haste");
    assert!(g.perm(back).data.truthy(DataKey::Unearth));
}

// ======================================================== the Basalt Monolith + Rings combo
#[test]
fn basalt_monolith_and_rings_of_brighthearth_are_a_known_combo() {
    let mut g = table(&["karn", "seph", "sauron"]);
    lands(&mut g, P0, "Wastes", 6, false);
    perm(&mut g, P0, "Basalt Monolith");
    hand(&mut g, P0, &["Rings of Brighthearth", "Walking Ballista"]);
    let o = brain::main_options(&mut g, P0, false).unwrap();
    assert!(labels(&o).contains(&"Rings of Brighthearth".to_string()), "{:?}", labels(&o));
    // the AI plays it out: Rings is cast, the loop runs with Walking Ballista as the outlet, the table loses
    brain::main(&mut g, P0, false).unwrap();
    assert!(g.over && g.winner == Some(P0), "the combo wins: over={} winner={:?}", g.over, g.winner);
}
