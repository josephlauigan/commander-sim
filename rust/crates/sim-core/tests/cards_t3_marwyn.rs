//! Tier 3 deck `marwyn-mono-green-elves` (impls/t3_marwyn.rs and the `// --- Tier 3 (marwyn)` patterns of the ability
//! language): each card on a small position, its Oracle text as the expectation.

use sim_core::ai::act;
use sim_core::engine::mana::{cost_of, total_mana};
use sim_core::engine::values::{epow, etgh};
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    table(&["marwyn-mono-green-elves", "aurelia-boros-extra-combats", "korvold-jund-sacrifice", "atraxa-infect"])
}

fn hand_opts(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    f(g, c, p, post).unwrap()
}

fn perm_opts(g: &mut Game, m: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(m).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, m, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn set_library(g: &mut Game, p: PlayerId, names: &[&str]) {
    let cs: Vec<CardId> = names.iter().map(|n| card(g, n)).collect();
    g.player_mut(p).library = cs;
}

/// a library of twelve Forests with this card on top (the AI doesn't cast a card that draws from a thin library)
fn set_top(g: &mut Game, p: PlayerId, top: &str) {
    let mut names = vec!["Forest"; 12];
    names.push(top);
    set_library(g, p, &names);
}
fn count_named(g: &Game, p: PlayerId, name: &str) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && &*g.perm(m).name == name).count()
}

// ======================================================== the ability language
#[test]
fn emerald_medallion_discounts_green_spells_only() {
    let mut g = tbl();
    let elf = take(&mut g, P0, "Elvish Archdruid"); // {1}{G}{G}
    let ring = take(&mut g, P0, "Sol Ring"); // {1}
    assert_eq!(cost_of(&g, P0, elf).0, 1);
    perm(&mut g, P0, "Emerald Medallion");
    assert_eq!(cost_of(&g, P0, elf), (0, "GG".to_string()), "a green spell costs {{1}} less");
    assert_eq!(cost_of(&g, P0, ring).0, 1, "a colourless spell does not");
    assert_eq!(cost_of(&g, P1, elf).0, 1, "an opponent's green spell does not");
}

#[test]
fn joraga_warcaller_pumps_other_elves_per_counter() {
    let mut g = tbl();
    let j = perm(&mut g, P0, "Joraga Warcaller");
    let e = perm(&mut g, P0, "Elvish Mystic");
    assert_eq!(epow(&g, e), 1, "no counters: no bonus");
    g.perm_mut(j).plus = 2;
    assert_eq!((epow(&g, e), etgh(&g, e)), (3, 3), "two counters: +2/+2");
    assert_eq!(epow(&g, j), 3, "not itself (1/1 with two counters)");
    let o = perm(&mut g, P1, "Elvish Mystic");
    assert_eq!(epow(&g, o), 1, "an opponent's Elf gets nothing");
}

#[test]
fn joraga_warcaller_is_multikicked_with_the_spare_mana() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 5, false);
    let c = hand(&mut g, P0, &["Joraga Warcaller"])[0];
    let o = hand_opts(&mut g, c, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert!(o[0].label.contains("2 extra"), "{{G}} + two kicks of {{1}}{{G}}: {}", o[0].label);
    assert!(run(&mut g, P0, &o[0]));
    let j = g.player(P0).perms.iter().copied().find(|&m| &*g.perm(m).name == "Joraga Warcaller").unwrap();
    assert_eq!(g.perm(j).plus, 2, "a +1/+1 counter for each kick");
    assert_eq!(total_mana(&g, P0, false), 0);
}

#[test]
fn immaculate_magistrate_counts_elves() {
    let mut g = tbl();
    let m = perm(&mut g, P0, "Immaculate Magistrate");
    perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Elvish Mystic");
    let t = perm(&mut g, P0, "Craterhoof Behemoth");
    let opts = sim_core::dsl::ability_options(&mut g, P0, true).unwrap();
    let o = opts.iter().find(|o| o.label.contains("Immaculate")).expect("its {T} ability is offered");
    assert!(run(&mut g, P0, o));
    assert!(g.perm(m).tapped);
    assert_eq!(g.perm(t).plus, 3, "three Elves: three counters on the target (Craterhoof)");
}

#[test]
fn dwynen_gains_life_per_attacking_elf() {
    let mut g = tbl();
    let d = perm(&mut g, P0, "Dwynen, Gilt-Leaf Daen");
    let a = perm(&mut g, P0, "Llanowar Elves");
    let b = perm(&mut g, P0, "Elvish Mystic");
    let h = token(&mut g, P0, 2); // no Elf
    assert_eq!(epow(&g, b), 2, "other Elves get +1/+1");
    let life = g.player(P0).life;
    sim_core::engine::combat::attack_triggers(&mut g, P0, &[d, a, b, h], P1).unwrap();
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert_eq!(g.player(P0).life, life + 3, "three attacking Elves (Dwynen too), the token is no Elf");
}

// ======================================================== mana: Elvish Guidance, Three Tree City, Wirewood Lodge
#[test]
fn elvish_guidance_adds_g_per_elf_on_the_battlefield() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 1, false);
    perm(&mut g, P0, "Elvish Guidance");
    assert_eq!(total_mana(&g, P0, false), 1, "no Elves: the land alone");
    perm(&mut g, P0, "Dwynen, Gilt-Leaf Daen");
    perm(&mut g, P1, "Llanowar Elves");
    // Dwynen (no mana) and an opponent's Elf: two more
    assert_eq!(total_mana(&g, P0, false), 3);
}

#[test]
fn three_tree_city_taps_for_its_type_less_the_activation() {
    let mut g = tbl();
    let c = take(&mut g, P0, "Three Tree City");
    let l = g.add_land(P0, c, false);
    let f = g.registry.get(c).unwrap().land_etb.unwrap();
    f(&mut g, P0, l).unwrap();
    assert_eq!(total_mana(&g, P0, false), 1, "no Elves: {{C}}");
    for n in ["Dwynen, Gilt-Leaf Daen", "Joraga Warcaller", "Immaculate Magistrate", "Galadhrim Brigade"] {
        perm(&mut g, P0, n);
    }
    assert_eq!(total_mana(&g, P0, false), 2, "four Elves: four mana for {{2}}, net two");
}

#[test]
fn wirewood_lodge_untaps_a_big_elf() {
    let mut g = tbl();
    let c = take(&mut g, P0, "Wirewood Lodge");
    g.add_land(P0, c, false);
    let priest = perm(&mut g, P0, "Priest of Titania");
    perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Elvish Mystic");
    let lodge_only = |g: &Game| total_mana(g, P0, false);
    let all = lodge_only(&g);
    assert_eq!(all, 1 + 3 + 1 + 1, "Lodge {{C}}, Priest 3, two Elves");
    g.perm_mut(priest).tapped = true;
    assert_eq!(lodge_only(&g), 2 + 1 + 1, "the Priest tapped: the Lodge untaps it for {{G}}, net two");
}

// ======================================================== Genesis Wave
#[test]
fn genesis_wave_puts_permanents_of_x_or_less_onto_the_battlefield() {
    let mut g = tbl();
    set_library(&mut g, P0, &["Forest", "Craterhoof Behemoth", "Harmonize", "Elvish Archdruid", "Sol Ring"]);
    let c = take(&mut g, P0, "Genesis Wave");
    let f = g.registry.get(c).unwrap().resolve.unwrap();
    let lands_before = g.player(P0).lands.len();
    let dest = f(&mut g, P0, c, &Ctx { x: 4, ..Ctx::default() }).unwrap();
    assert_eq!(dest, "gy");
    assert_eq!(count_named(&g, P0, "Sol Ring"), 1);
    assert_eq!(count_named(&g, P0, "Elvish Archdruid"), 1);
    assert_eq!(g.player(P0).lands.len(), lands_before, "the Forest was the fifth card, not revealed");
    let gy: Vec<&str> = g.player(P0).gy.iter().map(|&c| &*g.db.get(c).name).collect();
    assert!(gy.contains(&"Harmonize") && gy.contains(&"Craterhoof Behemoth"), "the rest go to the graveyard: {gy:?}");
    assert_eq!(g.player(P0).library.len(), 1);
}

// ======================================================== Dionus, Elvish Archdruid
#[test]
fn dionus_untaps_an_elf_tapped_on_your_turn_once() {
    let mut g = tbl();
    perm(&mut g, P0, "Dionus, Elvish Archdruid");
    let e = perm(&mut g, P0, "Llanowar Elves");
    assert!(sim_core::engine::mana::pay(&mut g, P0, 0, "G", false).unwrap());
    assert!(!g.perm(e).tapped, "untapped by Dionus");
    assert_eq!(g.perm(e).plus, 1, "with a +1/+1 counter");
    assert!(sim_core::engine::mana::pay(&mut g, P0, 0, "G", false).unwrap());
    assert!(g.perm(e).tapped, "only once each turn");
    assert_eq!(g.perm(e).plus, 1);
    // not on an opponent's turn
    let o = perm(&mut g, P1, "Dionus, Elvish Archdruid");
    let _ = o;
    let e1 = perm(&mut g, P1, "Llanowar Elves");
    assert!(sim_core::engine::mana::pay(&mut g, P1, 0, "G", false).unwrap());
    assert!(g.perm(e1).tapped, "Dionus only works during its controller's turn");
}

// ======================================================== Eladamri, Realmwalker
#[test]
fn eladamri_casts_creatures_from_the_top_and_cheats_one_in() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 3, false);
    let el = perm(&mut g, P0, "Eladamri, Korvecdal");
    set_top(&mut g, P0, "Elvish Visionary");
    let o = perm_opts(&mut g, el, P0, Some(false));
    assert!(o.iter().any(|o| o.label.contains("Elvish Visionary from the top")), "{:?}", labels(&o));
    set_top(&mut g, P0, "Harmonize");
    let o = perm_opts(&mut g, el, P0, Some(false));
    assert!(!o.iter().any(|o| o.label.contains("from the top")), "not a noncreature spell");
    // the activated ability: {G}, {T}, tap two creatures: Craterhoof from the hand
    perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Elvish Mystic");
    hand(&mut g, P0, &["Craterhoof Behemoth"]);
    let o = perm_opts(&mut g, el, P0, Some(false));
    let a = o.iter().find(|o| o.label.contains("Eladamri puts Craterhoof")).expect("the ability is offered");
    assert!(run(&mut g, P0, a));
    assert_eq!(count_named(&g, P0, "Craterhoof Behemoth"), 1);
    assert!(g.perm(el).tapped);
}

fn labels(o: &[Opt]) -> Vec<String> {
    o.iter().map(|o| o.label.clone()).collect()
}

#[test]
fn realmwalker_casts_only_its_type_from_the_top() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 4, false);
    perm(&mut g, P0, "Llanowar Elves");
    let r = perm(&mut g, P0, "Realmwalker");
    set_top(&mut g, P0, "Elvish Visionary");
    let o = perm_opts(&mut g, r, P0, Some(false));
    assert!(o.iter().any(|o| o.label.contains("from the top")), "an Elf (the type it named): {:?}", labels(&o));
    set_top(&mut g, P0, "Craterhoof Behemoth");
    let o = perm_opts(&mut g, r, P0, Some(false));
    assert!(o.is_empty(), "a Beast is not its type");
}

// ======================================================== Galadhrim Ambush, Galadhrim Brigade
#[test]
fn galadhrim_ambush_fogs_non_elves_and_makes_blockers() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 4, false);
    hand(&mut g, P0, &["Galadhrim Ambush"]);
    g.active = Some(P1);
    let atk: Vec<PermId> = (0..3).map(|_| token(&mut g, P1, 3)).collect();
    let elves_before = count_named(&g, P0, "Elf Warrior");
    assert!(sim_core::impls::t3_marwyn::ambush_response(&mut g, P0, P1, &atk).unwrap());
    let toks = g.player(P0).perms.iter().filter(|&&m| g.perm(m).token && g.perm(m).on_bf).count();
    assert!(toks >= 3 + elves_before, "three Elf Warriors");
    assert!(sim_core::impls::t3_marwyn::elf_fog(&g, atk[0]), "a non-Elf attacker deals no combat damage");
    let e = perm(&mut g, P1, "Llanowar Elves");
    assert!(!sim_core::impls::t3_marwyn::elf_fog(&g, e), "an Elf still does");
}

#[test]
fn galadhrim_brigade_squads_with_the_spare_mana() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 7, false);
    let c = hand(&mut g, P0, &["Galadhrim Brigade"])[0];
    let o = hand_opts(&mut g, c, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, P0, &o[0]));
    assert_eq!(count_named(&g, P0, "Galadhrim Brigade"), 3, "{{2}}{{G}} + two squads: the card and two copies");
    let b = g.player(P0).perms.iter().copied().find(|&m| &*g.perm(m).name == "Galadhrim Brigade").unwrap();
    assert_eq!(epow(&g, b), 4, "each gets +1/+1 from the other two");
}

// ======================================================== Legolas's Quick Reflexes
#[test]
fn legolas_untaps_a_mana_elf_that_then_kills_a_creature() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 1, false);
    let m = perm(&mut g, P0, "Marwyn, the Nurturer");
    g.perm_mut(m).plus = 3; // a 4/4 that taps for four
    g.perm_mut(m).tapped = true;
    let big = perm(&mut g, P1, "Craterhoof Behemoth"); // 5/5: too big
    let t = perm(&mut g, P1, "Elvish Archdruid"); // 2/2
    let c = hand(&mut g, P0, &["Legolas's Quick Reflexes"])[0];
    let o = hand_opts(&mut g, c, P0, Some(false));
    assert_eq!(o.len(), 1, "a tapped mana creature that can kill");
    assert!(run(&mut g, P0, &o[0]));
    assert!(!g.perm(t).on_bf, "4 damage kills the Archdruid");
    assert!(g.perm(big).on_bf);
    assert!(g.perm(m).tapped, "tapped again for mana");
    assert_eq!(g.player(P0).floating.g, 4, "the mana floats");
}

// ======================================================== Yavimaya, Yeva
#[test]
fn yavimaya_makes_every_land_a_forest_for_forestwalk() {
    let mut g = tbl();
    perm(&mut g, P0, "Elvish Champion");
    let e = perm(&mut g, P0, "Llanowar Elves");
    let b = token(&mut g, P1, 1);
    lands(&mut g, P1, "Mountain", 2, false);
    assert!(sim_core::engine::combat::can_block(&g, b, e), "no Forest: it can be blocked");
    let y = take(&mut g, P0, "Yavimaya, Cradle of Growth");
    g.add_land(P0, y, false);
    assert!(!sim_core::engine::combat::can_block(&g, b, e), "the Mountains are Forests: forestwalk");
}

#[test]
fn yeva_gives_green_creatures_flash() {
    let mut g = tbl();
    let elf = take(&mut g, P0, "Elvish Archdruid");
    let ring = take(&mut g, P0, "Sol Ring");
    assert!(!sim_core::engine::cast::grants_flash(&g, P0, elf));
    perm(&mut g, P0, "Yeva, Nature's Herald");
    assert!(sim_core::engine::cast::grants_flash(&g, P0, elf), "a green creature spell");
    assert!(!sim_core::engine::cast::grants_flash(&g, P0, ring), "not an artifact");
    assert!(!sim_core::engine::cast::grants_flash(&g, P1, elf), "not for an opponent");
}

// ======================================================== Umbral Mantle
#[test]
fn umbral_mantle_combo_needs_a_big_mana_elf_and_ezuri() {
    let mut g = tbl();
    let ready = |g: &Game| (sim_core::impls::t3_marwyn::COMBOS[0].ready)(g, P0).0;
    perm(&mut g, P0, "Umbral Mantle");
    let m = perm(&mut g, P0, "Marwyn, the Nurturer");
    g.perm_mut(m).plus = 2; // a 3/3: taps for three, {3} untaps it as a 5/5
    assert!(!ready(&g), "no Ezuri: no win");
    perm(&mut g, P0, "Ezuri, Renegade Leader");
    assert!(ready(&g), "Mantle + Marwyn + Ezuri before combat");
    g.step = Step::Main2;
    assert!(!ready(&g), "after combat the overruns hit nothing");
    g.step = Step::Main1;
    g.perm_mut(m).plus = 0;
    assert!(!ready(&g), "a 1/1 Marwyn and no other mana: no loop");
}
