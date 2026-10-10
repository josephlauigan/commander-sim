//! Jodah's deck on hand-built positions: Python's tests/test_jodah.py (Jodah's anthem and legend cascade, Kaldra,
//! the legends with abilities, the spells, the audit's protection and plan, the rework and swap candidates) and
//! tests/test_my_cards.py's Marchesa class (marchesa.py's cards), plus tests of Jodah's plan choices.
//!
//! Skipped: the VenatTymna class's Venat tests (Venat, Heart of Hydaelyn is not in data/cards.json: no list runs it,
//! and the Python fetches its card data on demand), the Marchesa class's Nim Deathmantle tests (Sephiroth's card code,
//! mine.py, ported with Sephiroth) and its Tezzeret's Gambit / Last Gasp tests (tag-only cards of the Y'shtola and
//! Alela decks, not Jodah's). The Burglar Rat's second enter trigger (Marchesa's return) waits for t3.py's port
//! (ignored). One assertion is left out: Unclaimed Territory's named type cached on the land
//! (`L.data['ctype']`), which the Rust works out each time instead.

use sim_core::ai::{act, brain, decks};
use sim_core::engine::{cast, combat, hooks, life, mana, removal, turn, values, zones};
use sim_core::hooks::{Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::{jodah as J, marchesa as M};
use sim_core::state::{Ctx, DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

/// Jodah from the command zone onto the battlefield (summoning sick, as Python's `enter`)
fn jodah(g: &mut Game, p: PlayerId) -> PermId {
    let c = g.player(p).cmd;
    let m = zones::enter(g, p, c, zones::Enter::default()).unwrap();
    g.perm_mut(m).is_cmd = true;
    g.player_mut(p).cmd_in_zone = false;
    m
}

fn names(g: &Game, p: PlayerId) -> Vec<String> {
    g.player(p).perms.iter().map(|&m| g.perm(m).name.to_string()).collect()
}

fn has_perm(g: &Game, p: PlayerId, name: &str) -> bool {
    names(g, p).iter().any(|n| n == name)
}

fn on(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.perm(m).on_bf && g.perm(m).owner == p
}

fn stat(g: &Game, p: PlayerId, name: &str) -> i64 {
    g.player(p).stats.get(name).copied().unwrap_or(0)
}

/// a permanent's `options` hook (Python's `E.CI.HOOKS[name]['options'](g, src, p, s, post)`)
fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

/// a card's `hand_options` hook
fn hand_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    f(g, c, p, post).unwrap()
}

fn labels(o: &[Opt]) -> Vec<String> {
    o.iter().map(|x| x.label.clone()).collect()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

/// Python's `E.CI.fire(g, event, p)`
fn fire(g: &mut Game, e: Event, p: PlayerId) {
    hooks::fire_trigger(g, e, Call::Player { p }).unwrap();
}

fn cast_from_hand(g: &mut Game, p: PlayerId, name: &str) -> bool {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap()
}

fn swords(g: &Game) -> Option<CardId> {
    Some(card(g, "Swords to Plowshares"))
}

fn set_library(g: &mut Game, p: PlayerId, names: &[&str]) {
    let cs: Vec<CardId> = names.iter().map(|n| card(g, n)).collect();
    g.player_mut(p).library = cs;
}

fn basics(g: &mut Game, p: PlayerId) {
    for n in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        lands(g, p, n, 1, false);
    }
}

fn pt(g: &Game, m: PermId) -> (i32, i32) {
    (values::epow(g, m), values::etgh(g, m))
}

// ======================================================== Jodah
#[test]
fn anthem_counts_legends() {
    let mut g = table(&["jodah", "veyran"]);
    let j = jodah(&mut g, P0);
    perm(&mut g, P0, "Lyra Dawnbringer");
    assert_eq!(values::epow(&g, j), 5 + 2); // two legends: +2/+2 each
    perm(&mut g, P0, "Solemn Simulacrum"); // not legendary: no bonus, no count
    assert_eq!(values::epow(&g, j), 7);
}

#[test]
fn cascade_from_hand_finds_a_cheaper_legend() {
    let mut g = table(&["jodah", "veyran"]);
    jodah(&mut g, P0);
    set_library(&mut g, P0, &["Island", "Lyra Dawnbringer", "Mind Stone", "King Darien XLVIII"]);
    lands(&mut g, P0, "Plains", 3, false);
    lands(&mut g, P0, "Swamp", 2, false);
    lands(&mut g, P0, "Island", 2, false);
    let c = hand(&mut g, P0, &["Dakkon Blackblade"])[0];
    mana::pay(&mut g, P0, 2, "WUUB", false).unwrap();
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert!(has_perm(&g, P0, "King Darien XLVIII")); // first legend below 6 from the top
    assert!(!has_perm(&g, P0, "Lyra Dawnbringer"));
}

#[test]
fn a_cascade_that_finds_nothing_says_so() {
    let mut g = table(&["jodah", "veyran"]);
    jodah(&mut g, P0);
    set_library(&mut g, P0, &["Island", "Lyra Dawnbringer", "Mind Stone"]); // no legend below mana value 2
    g.log = Some(vec![]);
    cast_from_hand(&mut g, P0, "Wrenn and Six");
    let mut lib: Vec<&str> = g.player(P0).library.iter().map(|&c| &*g.db.get(c).name).collect();
    lib.sort();
    assert_eq!(lib, ["Island", "Lyra Dawnbringer", "Mind Stone"]);
    let log = g.log.as_ref().unwrap();
    assert!(log.iter().any(|l| l.contains("finds no legendary nonland card with mana value below 2")));
}

#[test]
fn no_cascade_from_a_free_cast() {
    let mut g = table(&["jodah", "veyran"]);
    jodah(&mut g, P0);
    set_library(&mut g, P0, &["Mirri, Weatherlight Duelist", "King Darien XLVIII"]);
    let lyra = card(&g, "Lyra Dawnbringer");
    cast::cast_card(&mut g, P0, lyra, "lib", Ctx::default()).unwrap();
    assert!(!has_perm(&g, P0, "King Darien XLVIII"));
}

// ======================================================== Kaldra
#[test]
fn helm_assembles_kaldra() {
    let mut g = table(&["jodah", "veyran"]);
    lands(&mut g, P0, "Plains", 2, false);
    let pieces: Vec<PermId> =
        ["Sword of Kaldra", "Shield of Kaldra", "Helm of Kaldra"].iter().map(|n| perm(&mut g, P0, n)).collect();
    let o = options(&mut g, pieces[2], P0, Some(false));
    assert!(!o.is_empty() && run(&mut g, P0, &o[0]));
    let k = g
        .player(P0)
        .perms
        .iter()
        .copied()
        .find(|&m| g.perm(m).token && g.perm(m).data.truthy(DataKey::Kaldra))
        .unwrap();
    assert!(pieces.iter().all(|&e| g.perm(e).attached == Some(k)));
    assert_eq!(values::epow(&g, k), 9);
    assert!(values::indestructible(&g, k));
}

#[test]
fn sword_exiles_what_it_damages() {
    let mut g = table(&["jodah", "veyran"]);
    let sw = perm(&mut g, P0, "Sword of Kaldra");
    let a = perm(&mut g, P0, "Lyra Dawnbringer");
    g.perm_mut(sw).attached = Some(a);
    let b = perm(&mut g, P1, "Grave Titan");
    assert!(J::kaldra_exile(&mut g, a, b).unwrap());
    assert!(!g.perm(b).on_bf);
    assert!(g.player(P1).exile.contains(&card(&g, "Grave Titan")));
}

#[test]
fn shield_makes_the_kaldra_pieces_indestructible() {
    let mut g = table(&["jodah", "veyran"]);
    let sw = perm(&mut g, P0, "Sword of Kaldra");
    zones::die(&mut g, sw, "destroy").unwrap();
    assert!(!g.perm(sw).on_bf); // no Shield: the Sword is destroyed
    let sw = perm(&mut g, P0, "Sword of Kaldra");
    perm(&mut g, P0, "Shield of Kaldra");
    zones::die(&mut g, sw, "destroy").unwrap();
    assert!(on(&g, P0, sw));
}

// ======================================================== the legends
#[test]
fn blackblade_counts_lands() {
    let mut g = table(&["jodah", "veyran"]);
    lands(&mut g, P0, "Plains", 5, false);
    let bb = perm(&mut g, P0, "Blackblade Reforged");
    let m = perm(&mut g, P0, "Lyra Dawnbringer");
    g.perm_mut(bb).attached = Some(m);
    assert_eq!(values::epow(&g, m), 5 + 5);
}

#[test]
fn blackblade_equips_a_legend_for_three() {
    let mut g = table(&["jodah", "veyran"]);
    lands(&mut g, P0, "Plains", 3, false);
    let bb = perm(&mut g, P0, "Blackblade Reforged");
    let m = perm(&mut g, P0, "Lyra Dawnbringer");
    let o = options(&mut g, bb, P0, Some(false));
    assert_eq!(labels(&o), ["equip Blackblade Reforged to Lyra Dawnbringer"]);
    assert!(run(&mut g, P0, &o[0]));
    assert_eq!(g.perm(bb).attached, Some(m));
    assert!(options(&mut g, bb, P0, Some(false)).is_empty()); // already on one of yours
}

#[test]
fn korlash_counts_swamps() {
    let mut g = table(&["jodah", "veyran"]);
    lands(&mut g, P0, "Swamp", 2, false);
    lands(&mut g, P0, "Plains", 2, false);
    let k = perm(&mut g, P0, "Korlash, Heir to Blackblade");
    assert_eq!(values::epow(&g, k), 2);
}

#[test]
fn szadek_mills_instead_of_damage() {
    let mut g = table(&["jodah", "veyran"]);
    let s = perm(&mut g, P0, "Szadek, Lord of Secrets");
    g.perm_mut(s).sick = false;
    let (lib, life) = (g.player(P1).library.len(), g.player(P1).life);
    combat::resolve_combat(&mut g, P0, &[s], P1, &[]).unwrap();
    assert_eq!(g.player(P1).life, life);
    assert_eq!(g.player(P1).library.len(), lib - 5);
    assert_eq!(g.perm(s).plus, 5);
}

#[test]
fn dromoka_stops_opponents_on_your_turn() {
    let mut g = table(&["jodah", "veyran"]);
    perm(&mut g, P0, "Dragonlord Dromoka");
    let bolt = card(&g, "Lightning Bolt");
    g.active = Some(P0);
    assert!(!cast::castable(&g, P1, bolt, "hand"));
    g.active = Some(P1);
    assert!(cast::castable(&g, P1, bolt, "hand"));
}

#[test]
fn mirri_tapped_caps_attackers() {
    let mut g = table(&["jodah", "veyran"]);
    let m = perm(&mut g, P0, "Mirri, Weatherlight Duelist");
    g.perm_mut(m).tapped = true;
    let f = g.registry.get(card(&g, "Mirri, Weatherlight Duelist")).unwrap().attack_cap.unwrap();
    assert_eq!(f(&g, m, P1, P0), Some(1));
}

#[test]
fn mirri_lets_each_opponent_block_with_one_creature() {
    let mut g = table(&["jodah", "veyran"]);
    let m = perm(&mut g, P0, "Mirri, Weatherlight Duelist");
    let a = token(&mut g, P0, 4);
    let (b1, b2) = (token(&mut g, P1, 1), token(&mut g, P1, 2));
    let f = g.registry.get(card(&g, "Mirri, Weatherlight Duelist")).unwrap().blocks.unwrap();
    let mut assign = vec![(m, b1), (a, b2)];
    f(&mut g, m, P0, &[m, a], P1, &mut assign).unwrap();
    assert_eq!(assign, vec![(a, b2)]); // the biggest attacker keeps its blocker
}

#[test]
fn carth_adds_loyalty() {
    let mut g = table(&["jodah", "veyran"]);
    perm(&mut g, P0, "Carth the Lion");
    let w = perm(&mut g, P0, "Mordenkainen");
    let o = options(&mut g, w, P0, Some(false));
    let plus = o.iter().find(|x| x.label.contains("+3")).unwrap().clone(); // +2 becomes +3
    let before = g.perm(w).loyalty.unwrap();
    run(&mut g, P0, &plus);
    assert_eq!(g.perm(w).loyalty.unwrap(), before + 3);
}

#[test]
fn caparocti_taps_two_to_discover() {
    let mut g = table(&["jodah", "veyran"]);
    let c = perm(&mut g, P0, "Caparocti Sunborn");
    let a = perm(&mut g, P0, "Mind Stone");
    let b = perm(&mut g, P0, "Arcane Signet");
    set_library(&mut g, P0, &["Lyra Dawnbringer", "Island", "Fellwar Stone"]);
    let f = g.registry.get(card(&g, "Caparocti Sunborn")).unwrap().attack.unwrap();
    f(&mut g, c, P0, &[c], P1).unwrap();
    assert!(g.perm(a).tapped && g.perm(b).tapped);
    assert!(has_perm(&g, P0, "Fellwar Stone")); // discover 3: cast free
}

// ======================================================== the spells
#[test]
fn memory_jar_wheels_and_returns() {
    let mut g = table(&["jodah", "veyran"]);
    lands(&mut g, P0, "Plains", 3, false);
    let jar = perm(&mut g, P0, "Memory Jar");
    hand(&mut g, P0, &["Lyra Dawnbringer"]);
    let o = options(&mut g, jar, P0, Some(false));
    assert!(!o.is_empty() && run(&mut g, P0, &o[0]));
    assert_eq!(g.player(P0).hand.len(), 7);
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand, vec![card(&g, "Lyra Dawnbringer")]);
}

#[test]
fn profane_tutor_suspend_then_free() {
    let mut g = table(&["jodah", "veyran"]);
    lands(&mut g, P0, "Swamp", 2, false);
    let c = hand(&mut g, P0, &["Profane Tutor"])[0];
    let o = hand_options(&mut g, c, P0, Some(false));
    run(&mut g, P0, &o[0]);
    assert!(g.player(P0).exile.contains(&c));
    let n = g.player(P0).hand.len();
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), n);
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1); // cast free: a tutored card
}

#[test]
fn dissipate_exiles_and_desertion_steals() {
    let g = table(&["jodah", "veyran"]);
    let (dis, des) = (g.db.get(card(&g, "Dissipate")), g.db.get(card(&g, "Desertion")));
    assert!(dis.tag(sim_core::tag::Tag::Ctrexile));
    assert_eq!(des.tags.str(sim_core::tag::Tag::Ctr), Some("any"));
}

#[test]
fn court_returns_a_permanent() {
    let mut g = table(&["jodah", "veyran"]);
    perm(&mut g, P0, "Court of Ardenvale");
    assert_eq!(g.monarch, Some(P0));
    let ms = card(&g, "Mind Stone");
    g.player_mut(P0).gy.push(ms);
    fire(&mut g, Event::Upkeep, P0);
    assert!(has_perm(&g, P0, "Mind Stone"));
}

#[test]
fn fyndhorn_elder_makes_two() {
    let mut g = table(&["jodah", "veyran"]);
    perm(&mut g, P0, "Fyndhorn Elder");
    assert_eq!(mana::total_mana(&g, P0, false), 2);
}

#[test]
fn cartographers_survey_puts_two_lands_in() {
    let mut g = table(&["jodah", "veyran"]);
    set_library(&mut g, P0, &["Lyra Dawnbringer", "Forest", "Command Tower", "Mind Stone", "Plains"]);
    let c = card(&g, "Cartographer's Survey");
    let f = g.registry.get(c).unwrap().resolve.unwrap();
    f(&mut g, P0, c, &Ctx::default()).unwrap();
    let ls: Vec<&str> = g.player(P0).lands.iter().map(|&l| &*g.db.get(g.land(l).cd).name).collect();
    assert_eq!(ls.len(), 2);
    assert!(ls.contains(&"Command Tower")); // the land making the most colours first
    assert!(g.player(P0).lands.iter().all(|&l| g.land(l).tapped));
    assert_eq!(g.player(P0).library.len(), 3);
}

// ======================================================== the audit: Plaza of Heroes, Jodah first, the wish list
#[test]
fn plaza_any_colour_for_legendary_spells_only() {
    let mut g = table(&["jodah", "veyran"]);
    for n in ["Frontier Bivouac", "Forest", "Plaza of Heroes", "Sandsteppe Citadel", "Crumbling Necropolis"] {
        lands(&mut g, P0, n, 1, false);
    }
    g.pay_for = Some(g.player(P0).cmd);
    assert!(mana::can_pay(&g, P0, 0, "WUBRG", false)); // Plaza makes the fifth colour for Jodah
    g.pay_for = Some(card(&g, "Toxic Deluge"));
    assert!(!mana::can_pay(&g, P0, 0, "WUBRG", false));
    g.pay_for = None;
}

#[test]
fn jodah_before_legends() {
    let mut g = table(&["jodah", "veyran"]);
    g.active = Some(P0);
    basics(&mut g, P0);
    let lyra = hand(&mut g, P0, &["Lyra Dawnbringer"])[0];
    assert!(J::jodah_prio(&g, P0, lyra) <= 15); // cast Jodah first, then Lyra cascades
    let cmd = g.player(P0).cmd;
    assert!(J::jodah_prio(&g, P0, cmd) > 80);
    jodah(&mut g, P0);
    assert!(J::jodah_prio(&g, P0, lyra) > 40);
    assert_eq!(decks::deck_prio(&g, P0, lyra), J::jodah_prio(&g, P0, lyra)); // the deck's priority is Jodah's
}

#[test]
fn tutor_finds_fixing_early_and_a_big_legend_later() {
    let mut g = table(&["jodah", "veyran"]);
    lands(&mut g, P0, "Forest", 2, false);
    for n in ["Coalition Relic", "Sisay's Ring", "Razia, Boros Archangel"] {
        let c = card(&g, n);
        g.player_mut(P0).library.push(c);
    }
    assert_eq!(decks::tutor_pick(&g, P0, "any"), Some(card(&g, "Coalition Relic")));
    lands(&mut g, P0, "Plains", 3, false);
    jodah(&mut g, P0);
    assert_eq!(decks::tutor_pick(&g, P0, "any"), Some(card(&g, "Razia, Boros Archangel")));
}

#[test]
fn instant_tutors_find_toxic_deluge_under_pressure() {
    let mut g = table(&["jodah", "veyran"]);
    jodah(&mut g, P0);
    set_library(&mut g, P0, &["Toxic Deluge", "Demonic Tutor", "Force of Will"]);
    assert_eq!(decks::tutor_pick(&g, P0, "is"), Some(card(&g, "Demonic Tutor")));
    token(&mut g, P1, 8);
    token(&mut g, P1, 7);
    assert_eq!(decks::tutor_pick(&g, P0, "is"), Some(card(&g, "Toxic Deluge"))); // 15 power across the table
}

#[test]
fn the_third_kaldra_piece_comes_first() {
    let mut g = table(&["jodah", "veyran"]);
    perm(&mut g, P0, "Sword of Kaldra");
    hand(&mut g, P0, &["Shield of Kaldra"]);
    lands(&mut g, P0, "Plains", 5, false);
    assert_eq!(decks::tutor_pick(&g, P0, "any"), Some(card(&g, "Helm of Kaldra")));
}

// ======================================================== lands
#[test]
fn unclaimed_territory_names_the_commonest_creature_type() {
    let mut g = table(&["jodah", "veyran"]);
    assert_eq!(J::territory_type(&g, P0), "human"); // Jodah, Grand Abolisher, Mirri ... are Humans
    let gs = table(&["sauron", "veyran"]);
    assert_eq!(J::territory_type(&gs, P0), "orc"); // Sauron's deck keeps naming Orc
    for n in ["Unclaimed Territory", "Island", "Swamp"] {
        lands(&mut g, P0, n, 1, false);
    }
    g.pay_for = Some(card(&g, "Grand Abolisher")); // a Human: any colour
    assert!(mana::can_pay(&g, P0, 0, "WUB", false));
    g.pay_for = Some(card(&g, "Lyra Dawnbringer")); // an Angel: colourless only
    assert!(!mana::can_pay(&g, P0, 0, "WUB", false));
    assert!(mana::can_pay(&g, P0, 1, "UB", false));
    g.pay_for = Some(card(&g, "Toxic Deluge")); // not a creature spell
    assert!(!mana::can_pay(&g, P0, 0, "WUB", false));
    g.pay_for = None;
}

#[test]
fn vivid_creek_makes_any_colour_twice() {
    let mut g = table(&["jodah", "veyran"]);
    let l = lands(&mut g, P0, "Vivid Creek", 1, false)[0];
    for _ in 0..2 {
        assert!(mana::can_pay(&g, P0, 0, "R", false));
        mana::pay(&mut g, P0, 0, "R", false).unwrap();
        g.land_mut(l).tapped = false;
    }
    assert!(!mana::can_pay(&g, P0, 0, "R", false)); // the counters are gone: {U} only
    assert!(mana::can_pay(&g, P0, 0, "U", false));
    mana::pay(&mut g, P0, 0, "U", false).unwrap();
    assert_eq!(g.land(l).data.int(DataKey::Charge), 0);
}

// ======================================================== protecting Jodah (audit/jodah, 10-09)
/// Jodah on the battlefield, the opponent's turn
fn protection_table() -> (Game, PermId) {
    let mut g = table(&["jodah", "veyran"]);
    let j = jodah(&mut g, P0);
    g.active = Some(P1);
    (g, j)
}

#[test]
fn heroic_intervention_answers_targeted_removal() {
    let (mut g, j) = protection_table();
    lands(&mut g, P0, "Forest", 2, false);
    hand(&mut g, P0, &["Heroic Intervention"]);
    let s = swords(&g);
    removal::apply_removal(&mut g, Some(P1), j, "exile", s).unwrap();
    assert!(on(&g, P0, j));
    assert!(values::untargetable(&g, j)); // hexproof for the rest of the turn
    assert_eq!(stat(&g, P0, "jprot_Heroic Intervention"), 1);
}

#[test]
fn flawless_maneuver_is_free_but_only_stops_destroy() {
    let (mut g, j) = protection_table();
    hand(&mut g, P0, &["Flawless Maneuver"]);
    let s = swords(&g);
    removal::apply_removal(&mut g, Some(P1), j, "exile", s).unwrap();
    assert!(!on(&g, P0, j)); // indestructible doesn't stop exile
    let j = jodah(&mut g, P0);
    removal::apply_removal(&mut g, Some(P1), j, "destroy", s).unwrap();
    assert!(on(&g, P0, j)); // no lands: cast free with Jodah out
    assert_eq!(stat(&g, P0, "jprot_Flawless Maneuver"), 1);
}

#[test]
fn protection_is_held_not_cast_for_value() {
    let (mut g, _) = protection_table();
    lands(&mut g, P0, "Plains", 3, false);
    for n in ["Flawless Maneuver", "Tamiyo's Safekeeping", "Teferi's Protection"] {
        let c = hand(&mut g, P0, &[n])[0];
        let s = brain::Situation::new(&g, P0);
        assert_eq!(brain::card_utility(&g, P0, &s, c), None, "{n}");
    }
}

#[test]
fn safekeeping_keeps_jodah_through_a_wrath() {
    let (mut g, j) = protection_table();
    let other = perm(&mut g, P0, "Lyra Dawnbringer");
    lands(&mut g, P0, "Forest", 1, false);
    hand(&mut g, P0, &["Tamiyo's Safekeeping"]);
    assert_eq!(sim_core::ai::wipe_response(&mut g, P0, "destroy", P1).unwrap(), None); // Jodah alone is indestructible
    for m in g.player(P0).perms.clone() {
        zones::die(&mut g, m, "destroy").unwrap();
    }
    assert!(on(&g, P0, j));
    assert!(!on(&g, P0, other));
    assert_eq!(stat(&g, P0, "jprot_wipe_Tamiyo's Safekeeping"), 1);
}

#[test]
fn teferis_protection_against_an_exile_wipe() {
    let (mut g, _) = protection_table();
    lands(&mut g, P0, "Plains", 3, false);
    hand(&mut g, P0, &["Teferi's Protection"]);
    assert_eq!(sim_core::ai::wipe_response(&mut g, P0, "exile", P1).unwrap(), Some("all"));
}

#[test]
fn plaza_of_heroes_saves_jodah() {
    let (mut g, j) = protection_table();
    lands(&mut g, P0, "Plaza of Heroes", 1, false);
    lands(&mut g, P0, "Swamp", 3, false);
    let s = swords(&g);
    removal::apply_removal(&mut g, Some(P1), j, "destroy", s).unwrap();
    assert!(on(&g, P0, j));
    let plaza = card(&g, "Plaza of Heroes");
    assert!(!g.player(P0).lands.iter().any(|&l| g.land(l).cd == plaza));
    assert!(g.player(P0).exile.contains(&plaza));
    assert!(g.player(P0).lands.iter().all(|&l| g.land(l).tapped));
}

#[test]
fn runes_and_swat() {
    let (mut g, j) = protection_table();
    perm(&mut g, P0, "Giver of Runes");
    let s = swords(&g);
    removal::apply_removal(&mut g, Some(P1), j, "exile", s).unwrap();
    assert!(on(&g, P0, j));
    let t = perm(&mut g, P1, "Grave Titan");
    hand(&mut g, P0, &["Deflecting Swat"]);
    removal::apply_removal(&mut g, Some(P1), j, "exile", s).unwrap(); // Giver is tapped: Swat, free, sends it back
    assert!(on(&g, P0, j));
    assert!(!on(&g, P1, t));
}

#[test]
fn equipment_goes_to_jodah() {
    let (mut g, j) = protection_table();
    g.active = Some(P0);
    lands(&mut g, P0, "Plains", 3, false);
    let other = perm(&mut g, P0, "Lyra Dawnbringer");
    let boots = perm(&mut g, P0, "Swiftfoot Boots");
    g.perm_mut(boots).attached = Some(other);
    let o = J::jodah_options(&mut g, P0, false);
    assert_eq!(labels(&o), ["equip Swiftfoot Boots to Jodah"]);
    run(&mut g, P0, &o[0]);
    assert_eq!(g.perm(boots).attached, Some(j));
    assert!(values::untargetable(&g, j));
    let coat = perm(&mut g, P0, "Mithril Coat"); // attaches to Jodah as it enters
    assert_eq!(g.perm(coat).attached, Some(j));
    assert!(values::indestructible(&g, j));
    // the plan offers the move in the main phase
    let o = brain::main_options(&mut g, P0, false).unwrap();
    assert!(!o.iter().any(|x| x.label.starts_with("equip Swiftfoot Boots to Jodah"))); // already there
}

#[test]
fn bolt_bend_costs_r_with_power_four_and_redirects() {
    let (mut g, j) = protection_table();
    assert_eq!(J::prot_cost(&g, P0, "Bolt Bend"), (0, "R")); // Jodah is a 5/5
    zones::leave(&mut g, j).unwrap();
    perm(&mut g, P0, "Mirri, Weatherlight Duelist");
    assert_eq!(J::prot_cost(&g, P0, "Bolt Bend"), (3, "R")); // a 3/2 only: full price
    let j = jodah(&mut g, P0);
    let t = perm(&mut g, P1, "Grave Titan");
    lands(&mut g, P0, "Mountain", 1, false);
    let bend = hand(&mut g, P0, &["Bolt Bend"])[0];
    let s = swords(&g);
    removal::apply_removal(&mut g, Some(P1), j, "exile", s).unwrap();
    assert!(on(&g, P0, j));
    assert!(!on(&g, P1, t)); // the Swords hit the caster's Titan
    assert!(g.player(P0).gy.contains(&bend));
    let l = g.player(P0).lands[0];
    assert!(g.land(l).tapped); // paid {R}
    assert_eq!(stat(&g, P0, "jprot_Bolt Bend"), 1);
}

#[test]
fn bolt_bend_guards_a_key_legend_while_jodah_is_away() {
    let (mut g, j) = protection_table();
    zones::leave(&mut g, j).unwrap();
    g.player_mut(P0).cmd_in_zone = true;
    let wanderer = perm(&mut g, P0, "Maelstrom Wanderer"); // a 7/5 bomb
    let t = perm(&mut g, P1, "Grave Titan");
    lands(&mut g, P0, "Mountain", 1, false);
    hand(&mut g, P0, &["Bolt Bend"]);
    let s = swords(&g);
    removal::apply_removal(&mut g, Some(P1), wanderer, "destroy", s).unwrap();
    assert!(on(&g, P0, wanderer));
    assert!(!on(&g, P1, t));
    assert_eq!(stat(&g, P0, "jprot_Bolt Bend (legend)"), 1);
}

#[test]
fn no_protection_against_shrink_from_indestructible_only_cards() {
    // -X/-X: indestructible doesn't help, so Flawless Maneuver stays in hand
    let (mut g, j) = protection_table();
    hand(&mut g, P0, &["Flawless Maneuver"]);
    let s = swords(&g);
    assert!(!J::jodah_protect(&mut g, P0, j, "shrink6", Some(P1), s).unwrap());
    assert_eq!(g.player(P0).hand.len(), 1);
}

#[test]
fn holding_mana_for_protection_is_off_by_default() {
    // jodah_hold is switch 'hold' (JODAH_AI): the default set ('first,tutor') leaves it off
    let (mut g, _) = protection_table();
    lands(&mut g, P0, "Forest", 2, false);
    hand(&mut g, P0, &["Heroic Intervention"]);
    assert_eq!(J::jodah_hold(&g, P0), (None, 0.0));
}

// ======================================================== the rework candidates
#[test]
fn command_beacon_puts_jodah_in_hand_and_skips_the_tax() {
    let mut g = table(&["jodah", "veyran"]);
    for n in ["Plains", "Island", "Swamp", "Mountain", "Forest", "Command Beacon"] {
        lands(&mut g, P0, n, 1, false);
    }
    g.player_mut(P0).tax = 2; // castable from the zone (7 mana is not there): the Beacon makes it castable now
    let o = sim_core::impls::lands::land_options(&mut g, P0, Some(false)).unwrap();
    assert_eq!(labels(&o), ["Command Beacon: Jodah, the Unifier to hand"]);
    run(&mut g, P0, &o[0]);
    let cmd = g.player(P0).cmd;
    assert!(g.player(P0).hand.contains(&cmd));
    assert!(!g.player(P0).cmd_in_zone);
    assert_eq!(mana::cost_of(&g, P0, cmd), (0, "WUBRG".to_string())); // no commander tax from hand
    assert!(brain::do_cast(&mut g, P0, cmd, None).unwrap());
    let j = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).cd == Some(cmd)).unwrap();
    assert!(g.perm(j).is_cmd);
    assert_eq!(g.player(P0).tax, 2); // a cast from hand doesn't add tax
    assert_eq!((stat(&g, P0, "jr_beacon"), stat(&g, P0, "jr_jodah_from_hand")), (1, 1));
    zones::die(&mut g, j, "destroy").unwrap();
    assert!(g.player(P0).cmd_in_zone); // back to the command zone
}

#[test]
fn command_beacon_waits_while_the_tax_is_small() {
    let mut g = table(&["jodah", "veyran"]);
    for n in ["Plains", "Island", "Swamp", "Mountain", "Forest", "Forest", "Forest", "Command Beacon"] {
        lands(&mut g, P0, n, 1, false);
    }
    g.player_mut(P0).tax = 2; // 7 lands pay for Jodah with tax: keep the land
    assert!(sim_core::impls::lands::land_options(&mut g, P0, Some(false)).unwrap().is_empty());
    g.player_mut(P0).tax = 4;
    assert_eq!(sim_core::impls::lands::land_options(&mut g, P0, Some(false)).unwrap().len(), 1);
}

#[test]
fn a_discarded_commander_goes_to_the_command_zone() {
    let mut g = table(&["jodah", "veyran"]);
    let cmd = g.player(P0).cmd;
    g.player_mut(P0).cmd_in_zone = false;
    g.player_mut(P0).hand.push(cmd);
    zones::discard_cards(&mut g, P0, &[cmd]).unwrap();
    assert!(g.player(P0).cmd_in_zone);
    assert!(!g.player(P0).gy.contains(&cmd));
}

#[test]
fn nexus_cascades_the_first_spell_only() {
    let mut g = table(&["jodah", "veyran"]);
    perm(&mut g, P0, "Maelstrom Nexus");
    set_library(&mut g, P0, &["Fact or Fiction", "Island", "Arcane Signet"]);
    cast_from_hand(&mut g, P0, "Coalition Relic"); // mana value 3: Arcane Signet (2), not the 4
    assert!(has_perm(&g, P0, "Arcane Signet"));
    cast_from_hand(&mut g, P0, "Star Compass"); // the second spell: no cascade
    assert_eq!(stat(&g, P0, "jr_nexus"), 1);
    assert_eq!(g.player(P0).library.len(), 2);
}

#[test]
fn nexus_passes_on_a_counterspell() {
    let mut g = table(&["jodah", "veyran"]);
    perm(&mut g, P0, "Maelstrom Nexus");
    set_library(&mut g, P0, &["Arcane Denial"]);
    cast_from_hand(&mut g, P0, "Coalition Relic");
    assert_eq!(g.player(P0).library, vec![card(&g, "Arcane Denial")]);
}

#[test]
fn nexus_and_jodah_both_cascade_a_legend_from_hand() {
    let mut g = table(&["jodah", "veyran"]);
    jodah(&mut g, P0);
    perm(&mut g, P0, "Maelstrom Nexus");
    set_library(&mut g, P0, &["Moss Diamond", "King Darien XLVIII"]); // Jodah finds the legend, Nexus the rock
    cast_from_hand(&mut g, P0, "Lyra Dawnbringer");
    assert!(has_perm(&g, P0, "King Darien XLVIII"));
    assert!(has_perm(&g, P0, "Moss Diamond"));
    assert_eq!((stat(&g, P0, "jodah_cascades"), stat(&g, P0, "jr_nexus")), (1, 1));
}

#[test]
fn wanderer_cascades_twice() {
    let mut g = table(&["jodah", "veyran"]);
    set_library(&mut g, P0, &["Moss Diamond", "Island", "Lyra Dawnbringer"]);
    cast_from_hand(&mut g, P0, "Maelstrom Wanderer");
    assert!(has_perm(&g, P0, "Lyra Dawnbringer"));
    assert!(has_perm(&g, P0, "Moss Diamond"));
    assert_eq!(stat(&g, P0, "jr_wanderer"), 2);
}

#[test]
fn sisay_grows_with_colours_and_fetches_below_its_power() {
    let mut g = table(&["jodah", "veyran"]);
    let s = perm(&mut g, P0, "Sisay, Weatherlight Captain");
    assert_eq!(values::epow(&g, s), 2);
    perm(&mut g, P0, "Dakkon Blackblade"); // white, blue, black: +3/+3
    assert_eq!(values::epow(&g, s), 5);
    set_library(&mut g, P0, &["Lyra Dawnbringer", "Mirri, Weatherlight Duelist", "Caparocti Sunborn"]);
    basics(&mut g, P0);
    g.player_mut(P0).cmd_in_zone = false; // (Jodah gone for good: no Jodah first)
    let o = options(&mut g, s, P0, Some(false));
    assert_eq!(labels(&o), ["Sisay: Caparocti Sunborn"]); // the dearest below 5 (Lyra is 5)
    run(&mut g, P0, &o[0]);
    assert!(has_perm(&g, P0, "Caparocti Sunborn"));
    assert_eq!(stat(&g, P0, "jr_sisay"), 1);
}

#[test]
fn sisay_waits_while_jodah_is_castable() {
    let mut g = table(&["jodah", "veyran"]);
    let s = perm(&mut g, P0, "Sisay, Weatherlight Captain");
    set_library(&mut g, P0, &["Blackblade Reforged"]);
    basics(&mut g, P0);
    assert!(options(&mut g, s, P0, Some(false)).is_empty());
}

#[test]
fn shalai_gives_hexproof_to_jodah_and_you_but_not_herself() {
    let mut g = table(&["jodah", "veyran"]);
    let j = jodah(&mut g, P0);
    let w = perm(&mut g, P0, "Teferi, Hero of Dominaria");
    let sh = perm(&mut g, P0, "Shalai, Voice of Plenty");
    assert!(values::untargetable(&g, j));
    assert!(values::untargetable(&g, w)); // planeswalkers too
    assert!(!values::untargetable(&g, sh));
    assert!(values::player_hexproof(&g, P0));
    assert!(!values::player_hexproof(&g, P1));
    let s = swords(&g);
    let tg = removal::legal_targets(&g, P1, "exile", "cp", false, s);
    assert!(!tg.contains(&j) && !tg.contains(&w) && tg.contains(&sh));
    removal::apply_removal(&mut g, Some(P1), j, "exile", s).unwrap();
    assert!(on(&g, P0, j));
    let life = g.player(P0).life;
    let bolt = hand(&mut g, P1, &["Lightning Bolt"])[0];
    cast::cast_card(&mut g, P1, bolt, "hand", Ctx { face: Some(P0), ..Ctx::default() }).unwrap();
    assert_eq!(g.player(P0).life, life); // burn aimed at a hexproof player does nothing
    removal::apply_removal(&mut g, Some(P1), sh, "exile", s).unwrap();
    assert!(!values::untargetable(&g, j)); // Shalai gone: Jodah is exposed again
}

#[test]
fn shalai_counters_with_mana_left_late() {
    let mut g = table(&["jodah", "veyran"]);
    let j = jodah(&mut g, P0);
    let sh = perm(&mut g, P0, "Shalai, Voice of Plenty");
    lands(&mut g, P0, "Forest", 2, false);
    lands(&mut g, P0, "Plains", 4, false);
    assert!(options(&mut g, sh, P0, Some(false)).is_empty()); // not in the first main phase
    let o = options(&mut g, sh, P0, Some(true));
    assert_eq!(labels(&o), ["Shalai: +1/+1 counters"]);
    let (p0, s0) = (values::epow(&g, j), values::epow(&g, sh));
    run(&mut g, P0, &o[0]);
    assert_eq!((values::epow(&g, j), values::epow(&g, sh)), (p0 + 1, s0 + 1));
    assert_eq!(stat(&g, P0, "jr_shalai_counters"), 1);
}

#[test]
fn feed_the_cycle_costs_1bb_and_answers_an_attack() {
    let mut g = table(&["jodah", "veyran"]);
    let c = g.db.get(card(&g, "Feed the Cycle"));
    assert_eq!((c.generic, &*c.pips), (1, "BB"));
    assert_eq!(c.tags.str(sim_core::tag::Tag::Rem), Some("destroy"));
    assert_eq!(c.tags.str(sim_core::tag::Tag::Tgt), Some("cp"));
    g.active = Some(P1);
    let lyra = perm(&mut g, P1, "Lyra Dawnbringer");
    hand(&mut g, P0, &["Feed the Cycle"]);
    lands(&mut g, P0, "Swamp", 2, false);
    assert!(!sim_core::ai::attack_response(&mut g, P0, P1, &[lyra]).unwrap()); // {1}{B}: one mana short
    lands(&mut g, P0, "Swamp", 1, false);
    assert!(sim_core::ai::attack_response(&mut g, P0, P1, &[lyra]).unwrap());
    assert!(!on(&g, P1, lyra));
}

// ======================================================== Tymna the Weaver
#[test]
fn tymna_pays_a_life_per_opponent_hit() {
    let mut g = table(&["jodah", "veyran", "seph"]);
    let t = perm(&mut g, P0, "Tymna the Weaver");
    let a = token(&mut g, P0, 3);
    let n = g.player(P0).hand.len();
    fire(&mut g, Event::Main2, P0); // nobody hit: nothing
    assert_eq!(g.player(P0).hand.len(), n);
    combat::resolve_combat(&mut g, P0, &[t], P1, &[]).unwrap();
    combat::resolve_combat(&mut g, P0, &[a], P2, &[]).unwrap();
    let life = g.player(P0).life; // (Tymna's lifelink: +2)
    fire(&mut g, Event::Main2, P0);
    assert_eq!((g.player(P0).life, g.player(P0).hand.len(), stat(&g, P0, "jr_tymna_draws")), (life - 2, n + 2, 2));
    fire(&mut g, Event::Main2, P0); // once a turn
    assert_eq!(g.player(P0).hand.len(), n + 2);
}

#[test]
fn tymna_keeps_low_life() {
    let mut g = table(&["jodah", "veyran", "seph"]);
    perm(&mut g, P0, "Tymna the Weaver");
    let a = token(&mut g, P0, 3);
    combat::resolve_combat(&mut g, P0, &[a], P1, &[]).unwrap();
    g.player_mut(P0).life = 10; // Necropotence's floor: 10 at least
    let n = g.player(P0).hand.len();
    fire(&mut g, Event::Main2, P0);
    assert_eq!((g.player(P0).life, g.player(P0).hand.len()), (10, n));
}

// ======================================================== Garland, Royal Kidnapper
const GARLAND: &str = "Garland, Royal Kidnapper";

#[test]
fn garland_crowns_the_opponent_with_the_best_creature_and_steals_it() {
    let mut g = table(&["jodah", "veyran", "seph"]);
    let lyra = perm(&mut g, P1, "Lyra Dawnbringer");
    let s = perm(&mut g, P2, "Solemn Simulacrum");
    assert_eq!(pt(&g, lyra), (5, 5));
    perm(&mut g, P0, GARLAND);
    assert_eq!(g.monarch, Some(P1)); // Veyran has the creature worth taking
    assert!(on(&g, P0, lyra));
    assert!(!g.player(P1).perms.contains(&lyra));
    assert_eq!(pt(&g, lyra), (7, 7)); // +2/+2: controlled, not owned
    assert!(on(&g, P2, s));
    assert_eq!(stat(&g, P0, "jr_garland_steals"), 1);
}

#[test]
fn garland_control_returns_when_that_opponent_loses_the_monarch() {
    let mut g = table(&["jodah", "veyran", "seph"]);
    let lyra = perm(&mut g, P1, "Lyra Dawnbringer");
    perm(&mut g, P0, GARLAND);
    let a = token(&mut g, P2, 2);
    combat::resolve_combat(&mut g, P2, &[a], P1, &[a]).unwrap(); // Sephiroth takes the crown from Veyran
    assert_eq!(g.monarch, Some(P2));
    assert!(on(&g, P1, lyra));
    assert!(!g.player(P0).perms.contains(&lyra));
    assert_eq!(pt(&g, lyra), (5, 5));
    assert_eq!(stat(&g, P0, "jr_garland_back"), 1);
}

#[test]
fn garland_an_opponent_taking_the_monarch_from_you_triggers_a_steal() {
    let mut g = table(&["jodah", "veyran", "seph"]);
    perm(&mut g, P0, GARLAND); // no creatures yet: nothing to take
    assert_eq!(g.monarch, Some(P1));
    sim_core::cardcode::become_monarch(&mut g, P0).unwrap(); // (you take it back: no trigger for you)
    assert_eq!(stat(&g, P0, "jr_garland_steals"), 0);
    let s = perm(&mut g, P2, "Solemn Simulacrum");
    let a = token(&mut g, P2, 1);
    combat::resolve_combat(&mut g, P2, &[a], P0, &[a]).unwrap(); // Sephiroth hits you and takes the crown
    assert_eq!(g.monarch, Some(P2));
    assert!(on(&g, P0, s)); // the more valuable of Sephiroth's creatures
    assert_eq!(stat(&g, P0, "jr_garland_steals"), 1);
}

#[test]
fn garland_stolen_creatures_cant_be_sacrificed() {
    let mut g = table(&["jodah", "veyran", "seph"]);
    let t = token(&mut g, P1, 1);
    let gar = perm(&mut g, P0, GARLAND);
    assert!(on(&g, P0, t));
    assert_eq!(pt(&g, t), (3, 3));
    zones::die(&mut g, t, "sac").unwrap();
    assert!(on(&g, P0, t));
    assert!(cast::sac_fodder(&g, P0, "creature", Some(gar)).is_none());
    zones::edict(&mut g, P0, false).unwrap(); // not the stolen 1/1: Garland has to go
    assert!(on(&g, P0, t));
    assert!(!on(&g, P0, gar));
    assert_eq!(pt(&g, t), (1, 1)); // Garland gone: no bonus, still yours
    zones::edict(&mut g, P0, false).unwrap(); // ... and it can be sacrificed again
    assert!(!on(&g, P0, t));
}

#[test]
fn garland_the_steal_outlasts_garland_and_ends_with_the_owner() {
    let mut g = table(&["jodah", "veyran", "seph"]);
    let lyra = perm(&mut g, P1, "Lyra Dawnbringer");
    let gar = perm(&mut g, P0, GARLAND);
    zones::die(&mut g, gar, "destroy").unwrap();
    assert!(on(&g, P0, lyra)); // "for as long as they're the monarch"
    g.player_mut(P1).life = 0;
    life::check_state(&mut g).unwrap(); // Veyran leaves the game: so does its card
    assert!(!g.player(P0).perms.contains(&lyra));
    assert!(!g.garland);
}

// ======================================================== Marchesa, the Black Rose (test_my_cards.Marchesa)
const MARCHESA: &str = "Marchesa, the Black Rose";

#[test]
fn marchesa_dethrone_on_the_life_leader() {
    let mut g = table(&["jodah", "veyran", "sauron"]);
    let a = perm(&mut g, P0, MARCHESA);
    let b = perm(&mut g, P0, "Burglar Rat");
    g.player_mut(P1).life = 41;
    combat::attack_triggers(&mut g, P0, &[a, b], P2).unwrap(); // not the most life: nothing
    assert_eq!((g.perm(a).plus, g.perm(b).plus), (0, 0));
    combat::attack_triggers(&mut g, P0, &[a, b], P1).unwrap(); // the Rat has dethrone from Marchesa
    assert_eq!((g.perm(a).plus, g.perm(b).plus), (1, 1));
}

#[test]
fn marchesa_a_countered_creature_returns_at_the_next_end_step() {
    let mut g = table(&["jodah", "veyran"]);
    hand(&mut g, P1, &["Island", "Island"]);
    perm(&mut g, P0, MARCHESA);
    let rat = perm(&mut g, P0, "Burglar Rat"); // enters: each opponent discards
    g.perm_mut(rat).plus = 1;
    zones::die(&mut g, rat, "destroy").unwrap();
    let rc = card(&g, "Burglar Rat");
    assert!(g.player(P0).gy.contains(&rc));
    turn::end_step(&mut g, P1).unwrap(); // anyone's end step
    assert!(!g.player(P0).gy.contains(&rc));
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).cd == Some(rc) && g.perm(m).plus == 0)); // back, no counters
}

#[test]
#[ignore = "needs t3.py's Burglar Rat (each opponent discards as it enters), another agent's port"]
fn marchesa_a_returning_creature_enters_again() {
    let mut g = table(&["jodah", "veyran"]);
    hand(&mut g, P1, &["Island", "Island"]);
    perm(&mut g, P0, MARCHESA);
    let rat = perm(&mut g, P0, "Burglar Rat");
    g.perm_mut(rat).plus = 1;
    zones::die(&mut g, rat, "destroy").unwrap();
    turn::end_step(&mut g, P1).unwrap();
    assert_eq!(g.player(P1).hand.len(), 0); // its enter trigger twice
}

#[test]
fn marchesa_no_counter_no_return() {
    let mut g = table(&["jodah", "veyran"]);
    perm(&mut g, P0, MARCHESA);
    let rat = perm(&mut g, P0, "Burglar Rat");
    zones::die(&mut g, rat, "destroy").unwrap();
    turn::end_step(&mut g, P0).unwrap();
    assert!(g.player(P0).gy.contains(&card(&g, "Burglar Rat")));
}

#[test]
fn marchesa_exiled_creatures_do_not_return() {
    let mut g = table(&["jodah", "veyran"]);
    perm(&mut g, P0, MARCHESA);
    let rat = perm(&mut g, P0, "Burglar Rat");
    g.perm_mut(rat).plus = 2;
    zones::exile_perm(&mut g, rat).unwrap();
    turn::end_step(&mut g, P0).unwrap();
    let rc = card(&g, "Burglar Rat");
    assert!(g.player(P0).exile.contains(&rc));
    assert!(!g.player(P0).perms.iter().any(|&m| g.perm(m).cd == Some(rc)));
}

#[test]
fn marchesa_a_wipe_returns_everything_with_counters_marchesa_included() {
    // the creatures die at the same time as Marchesa, so her ability sees them all
    let mut g = table(&["jodah", "veyran"]);
    let a = perm(&mut g, P0, MARCHESA);
    let rat = perm(&mut g, P0, "Burglar Rat");
    let bird = perm(&mut g, P0, "Thrummingbird");
    perm(&mut g, P0, "Carrion Feeder");
    let ps = &mut g.player_mut(P0).perms;
    ps.retain(|&m| m != a);
    ps.insert(0, a); // Marchesa dies first
    for m in [a, rat, bird] {
        g.perm_mut(m).plus = 1;
    }
    removal::apply_wipe(&mut g, P1, "destroy", None, &Ctx::default()).unwrap();
    assert!(!g.player(P0).perms.iter().any(|&m| g.is_creature(m)));
    turn::end_step(&mut g, P1).unwrap();
    let mut back: Vec<String> =
        g.player(P0).perms.iter().filter(|&&m| g.is_creature(m)).map(|&m| g.perm(m).name.to_string()).collect();
    back.sort();
    assert_eq!(back, ["Burglar Rat", MARCHESA, "Thrummingbird"]); // not Carrion Feeder
}

#[test]
fn marchesa_a_stolen_creature_returns_under_your_control() {
    let mut g = table(&["jodah", "veyran"]);
    perm(&mut g, P0, MARCHESA);
    let snipe = perm(&mut g, P1, "Guttersnipe");
    M::steal(&mut g, P0, snipe, true);
    combat::attack_triggers(&mut g, P0, &[snipe], P1).unwrap(); // dethrone: a +1/+1 counter
    zones::die(&mut g, snipe, "sac").unwrap();
    let gc = card(&g, "Guttersnipe");
    assert!(g.player(P1).gy.contains(&gc)); // its owner's graveyard
    turn::end_step(&mut g, P0).unwrap();
    let back = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).cd == Some(gc)).unwrap();
    assert_eq!(g.perm(back).orig, P1);
    assert!(!g.player(P1).perms.contains(&back));
}

#[test]
fn accursed_marauder() {
    // 2/1: "each player sacrifices a nontoken creature of their choice" - Marchesa's player gives up one she returns
    let mut g = table(&["jodah", "veyran"]);
    perm(&mut g, P0, MARCHESA);
    let rat = perm(&mut g, P0, "Burglar Rat");
    g.perm_mut(rat).plus = 1;
    perm(&mut g, P1, "Guttersnipe");
    token(&mut g, P1, 1);
    let mar = perm(&mut g, P0, "Accursed Marauder");
    assert_eq!((g.perm(mar).pow, g.perm(mar).tgh), (2, 1));
    assert!(g.player(P0).gy.contains(&card(&g, "Burglar Rat")));
    assert!(on(&g, P0, mar));
    assert!(g.player(P1).gy.contains(&card(&g, "Guttersnipe"))); // the token doesn't count
    assert_eq!(g.player(P1).perms.iter().filter(|&&m| g.perm(m).token).count(), 1);
}

#[test]
fn two_notion_thieves_do_not_loop() {
    // 614.5: each replacement applies once; the draw goes to the other Thief's player and back
    let mut g = table(&["yshtola", "sauron"]);
    perm(&mut g, P0, "Notion Thief");
    perm(&mut g, P1, "Notion Thief");
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!((g.player(P0).hand.len(), g.player(P1).hand.len()), (0, 1));
}

#[test]
fn notion_thief_flashes_in_at_the_end_of_an_opponents_turn() {
    let mut g = table(&["yshtola", "sauron"]);
    g.active = Some(P1);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Swamp", 2, false);
    let c = hand(&mut g, P0, &["Notion Thief"])[0];
    assert!(hand_options(&mut g, c, P0, Some(false)).is_empty()); // only in the end-of-turn window
    let o = hand_options(&mut g, c, P0, None);
    assert_eq!(labels(&o), ["Notion Thief (flash)"]);
    run(&mut g, P0, &o[0]);
    assert!(has_perm(&g, P0, "Notion Thief"));
}

#[test]
fn coalition_relic_charge() {
    let mut g = table(&["jodah", "veyran"]);
    let relic = perm(&mut g, P0, "Coalition Relic");
    fire(&mut g, Event::EndStep, P0);
    assert!(g.perm(relic).tapped);
    g.perm_mut(relic).tapped = false;
    g.player_mut(P0).floating.any = 0;
    fire(&mut g, Event::Upkeep, P0);
    assert_eq!(mana::total_mana(&g, P0, false), 2); // the charge counter plus its tap
}

#[test]
fn marchesa_makes_a_returning_creature_cheap_to_sacrifice() {
    let mut g = table(&["jodah", "veyran"]);
    let rat = perm(&mut g, P0, "Burglar Rat");
    g.perm_mut(rat).plus = 1;
    let full = zones::sac_worth(&g, rat);
    perm(&mut g, P0, MARCHESA);
    assert!(zones::sac_worth(&g, rat) < full);
    // Burglar Rat's enter trigger: each opponent with cards in hand discards (0.8 each)
    hand(&mut g, P1, &["Island"]);
    let rc = card(&g, "Burglar Rat");
    assert_eq!(M::card_etb_value(&g, P0, Some(rc)), 0.8);
}

#[test]
fn marchesa_returns_herself_with_a_counter() {
    let mut g = table(&["jodah", "veyran"]);
    let a = perm(&mut g, P0, MARCHESA);
    assert!(g.marchesa_on); // the engine checks deaths for her trigger from here on
    g.perm_mut(a).plus = 1;
    zones::die(&mut g, a, "destroy").unwrap();
    assert_eq!(g.marchesa_due.len(), 1);
    turn::end_step(&mut g, P1).unwrap();
    assert!(has_perm(&g, P0, MARCHESA));
    assert_eq!(stat(&g, P0, "marchesa_returns"), 1);
}
