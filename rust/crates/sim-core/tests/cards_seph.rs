//! Sephiroth's cards and plan on hand-built positions: Python's tests/test_my_cards.py, classes Sephiroth, NewCards
//! (Ephemerate), Bahamut and SephFlicker, and test_audit_fixes.Muldrotha; then tests of the plan's choices. Each
//! expectation comes from the card's Oracle text (quoted where the Python quotes it).
//!
//! Not ported: Sephiroth, Planet's Heir and Avacyn's Pilgrim (in no decklist now); a person's choices (practice mode,
//! test_play_seph.py, Muldrotha's practice-mode cast). Ignored until the other phase-6 modules are merged (`cargo test
//! -- --ignored` runs them): the flicker engines' own card code (Soulherder, Conjurer's Closet, Teleportation Circle,
//! Flickering Hound, Restoration Angel's enter trigger: t2.rs), Ephemerate's rebound (common.rs), Karn's Bastion and
//! proliferate (lands.rs, common.rs), and Marchesa (marchesa.rs).

use sim_core::ai::{self, act, brain, decks, seph};
use sim_core::engine::{cast, combat, hooks, life, removal, turn, values, zones};
use sim_core::hooks::{Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::mine;
use sim_core::state::{Ctx, DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const B: &str = "Summon: Bahamut";

fn lives(g: &Game) -> Vec<i32> {
    g.players.iter().map(|p| p.life).collect()
}

fn named(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| values::card_name(g, m) == Some(name))
}

fn on_bf(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

fn tokens(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).token).count()
}

fn options(g: &mut Game, p: PlayerId, post: bool) -> Vec<Opt> {
    brain::main_options(g, p, post).unwrap()
}

fn labels(opts: &[Opt]) -> Vec<String> {
    opts.iter().map(|o| o.label.clone()).collect()
}

fn take_option(g: &mut Game, p: PlayerId, label: &str) -> bool {
    let opts = options(g, p, false);
    let o = opts.iter().find(|o| o.label == label).unwrap_or_else(|| panic!("no option {label:?} in {opts:?}"));
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn lore(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.int(DataKey::Lore)
}

/// Python's `E.CI.fire(g, 'main1', p)`
fn main1(g: &mut Game, p: PlayerId, n: usize) {
    for _ in 0..n {
        hooks::fire_trigger(g, Event::Main1, Call::Player { p }).unwrap();
    }
}

// ======================================================== Sephiroth (test_my_cards.Sephiroth)
#[test]
fn sheoldred_the_apocalypse() {
    // "Whenever you draw a card, you gain 2 life. Whenever an opponent draws a card, they lose 2 life."
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Sheoldred, the Apocalypse");
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!(lives(&g), vec![40, 38]); // the opponent's draw gains you nothing
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert_eq!(lives(&g), vec![42, 38]);
}

#[test]
fn massacre_wurm() {
    // "When this enters, creatures your opponents control get -2/-2 until end of turn. Whenever a creature an opponent
    //  controls dies, that player loses 2 life."
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P1, "Guttersnipe");
    let mystic = perm(&mut g, P1, "Murmuring Mystic"); // 2/2 dies, 1/5 lives
    perm(&mut g, P0, "Massacre Wurm");
    assert_eq!(g.player(P1).perms, vec![mystic]);
    assert_eq!(g.player(P1).life, 38); // one death, 2 life
}

#[test]
fn archon_of_cruelty_enters() {
    // opponent sacrifices a creature, discards a card and loses 3; you draw a card and gain 3
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P1, "Guttersnipe");
    hand(&mut g, P1, &["Lightning Bolt", "Think Twice"]);
    perm(&mut g, P0, "Archon of Cruelty");
    assert_eq!((g.player(P0).life, g.player(P0).hand.len()), (43, 1));
    let v = g.player(P1);
    assert_eq!((v.life, v.hand.len(), v.perms.len()), (37, 1, 0));
}

#[test]
fn gray_merchant_drains_devotion() {
    let mut g = table(&["seph", "veyran", "sauron"]);
    perm(&mut g, P0, "Sheoldred, Whispering One"); // {5}{B}{B}: devotion 2 + Gray Merchant's 2
    perm(&mut g, P0, "Gray Merchant of Asphodel");
    assert_eq!(lives(&g), vec![48, 36, 36]);
}

#[test]
fn blood_artist() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Blood Artist");
    let s = perm(&mut g, P1, "Guttersnipe");
    zones::die(&mut g, s, "destroy").unwrap();
    assert_eq!(lives(&g), vec![41, 39]);
}

#[test]
fn elesh_norn_grand_cenobite() {
    let mut g = table(&["seph", "veyran"]);
    let mystic = perm(&mut g, P1, "Murmuring Mystic");
    let artist = perm(&mut g, P0, "Blood Artist");
    perm(&mut g, P0, "Elesh Norn, Grand Cenobite");
    assert_eq!(values::etgh(&g, mystic), 3); // 1/5 gets -2/-2
    assert_eq!((values::epow(&g, artist), values::etgh(&g, artist)), (2, 3)); // 0/1 gets +2/+2
}

#[test]
fn grave_titan() {
    let mut g = table(&["seph", "veyran"]);
    let t = perm(&mut g, P0, "Grave Titan");
    assert_eq!(tokens(&g, P0), 2); // enters with two Zombies
    combat::attack_triggers(&mut g, P0, &[t], P1).unwrap();
    assert_eq!(tokens(&g, P0), 4); // and two more when it attacks
}

#[test]
fn atraxa_takes_one_card_of_each_type() {
    let mut g = table(&["seph", "veyran"]);
    let lib = &g.player(P0).library;
    let top: Vec<CardId> = lib[lib.len() - 10..].to_vec();
    perm(&mut g, P0, "Atraxa, Grand Unifier");
    let h = &g.player(P0).hand;
    assert!(h.iter().all(|c| top.contains(c)));
    let mut kinds: Vec<_> = h.iter().map(|&c| format!("{:?}", g.db.get(c).types)).collect();
    let n = kinds.len();
    kinds.sort();
    kinds.dedup();
    assert_eq!(kinds.len(), n);
}

#[test]
fn consecrated_sphinx() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Consecrated Sphinx");
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!(g.player(P0).hand.len(), 2);
}

// --- the four loops: Mikaeus + Triskelion, Mikaeus or Melira + Kitchen Finks, Nim Deathmantle + Ashnod's Altar + Grave
// Titan
fn loops_with(cards: &[&str], opp: &[&str]) -> (Game, Vec<(&'static str, bool)>) {
    let mut g = table(&["seph", "veyran", "sauron"]);
    for c in cards {
        perm(&mut g, P0, c);
    }
    for c in opp {
        perm(&mut g, P1, c);
    }
    let l = loop_names(&g);
    (g, l)
}

fn loop_names(g: &Game) -> Vec<(&'static str, bool)> {
    mine::seph_loops(g, P0).into_iter().map(|(i, _, kills)| (mine::LOOPS[i].name, kills)).collect()
}

#[test]
fn mikaeus_triskelion_kills_alone() {
    let (_, l) = loops_with(&["Mikaeus, the Unhallowed", "Triskelion", "Viscera Seer"], &[]);
    assert_eq!(l, vec![("Mikaeus + Triskelion", true)]);
}

#[test]
fn finks_loops_need_a_payoff() {
    for enabler in ["Mikaeus, the Unhallowed", "Melira, Sylvok Outcast"] {
        let (_, l) = loops_with(&[enabler, "Kitchen Finks", "Ashnod's Altar"], &[]);
        assert_eq!(l.iter().map(|x| x.1).collect::<Vec<_>>(), vec![false]);
        let (_, l) = loops_with(&[enabler, "Kitchen Finks", "Ashnod's Altar", "Zulaport Cutthroat"], &[]);
        assert_eq!(l.iter().map(|x| x.1).collect::<Vec<_>>(), vec![true]);
        let (_, l) = loops_with(&[enabler, "Kitchen Finks", "Altar of Dementia"], &[]); // mills everyone
        assert_eq!(l.iter().map(|x| x.1).collect::<Vec<_>>(), vec![true]);
    }
}

#[test]
fn deathmantle_loop() {
    let base = ["Nim Deathmantle", "Ashnod's Altar", "Grave Titan"];
    let (_, l) = loops_with(&base, &[]);
    assert_eq!(l.iter().map(|x| x.1).collect::<Vec<_>>(), vec![false]);
    let (_, l) = loops_with(&[base[0], base[1], base[2], "Blood Artist"], &[]);
    assert_eq!(l.iter().map(|x| x.1).collect::<Vec<_>>(), vec![true]);
    let (_, l) = loops_with(&[base[0], base[1], base[2], "Triskelion"], &[]);
    assert_eq!(l.iter().map(|x| x.1).collect::<Vec<_>>(), vec![true]); // infinite mana keeps returning Triskelion
}

#[test]
fn finks_loops_need_an_outlet() {
    let (_, l) =
        loops_with(&["Mikaeus, the Unhallowed", "Kitchen Finks", "Melira, Sylvok Outcast", "Blood Artist"], &[]);
    assert_eq!(l, vec![]);
}

#[test]
fn mikaeus_triskelion_needs_no_outlet() {
    // undying returns it with four counters: two pings at itself, two at opponents, and it dies a 2/2 with 2 damage
    let (_, l) = loops_with(&["Mikaeus, the Unhallowed", "Triskelion"], &[]);
    assert_eq!(l, vec![("Mikaeus + Triskelion", true)]);
}

#[test]
fn a_bigger_triskelion_needs_an_outlet() {
    // Elesh Norn, Grand Cenobite makes it a counterless 4/4, and Nim Deathmantle on it a 6/6: all four pings (or more)
    // go to itself, so only a free sacrifice outlet keeps it looping
    let (_, l) = loops_with(&["Mikaeus, the Unhallowed", "Triskelion", "Elesh Norn, Grand Cenobite"], &[]);
    assert_eq!(l, vec![]);
    let (_, l) =
        loops_with(&["Mikaeus, the Unhallowed", "Triskelion", "Elesh Norn, Grand Cenobite", "Viscera Seer"], &[]);
    assert_eq!(l, vec![("Mikaeus + Triskelion", true)]);
    let (mut g, _) = loops_with(&["Mikaeus, the Unhallowed", "Triskelion", "Nim Deathmantle"], &[]);
    let nim = named(&g, P0, "Nim Deathmantle").unwrap();
    g.perm_mut(nim).attached = named(&g, P0, "Triskelion");
    assert_eq!(loop_names(&g), vec![]);
}

#[test]
fn mikaeus_alone_wants_just_triskelion() {
    let (g, _) = loops_with(&["Mikaeus, the Unhallowed"], &[]);
    assert_eq!(mine::loop_need(&g, P0, &[]), vec!["Triskelion"]);
    let (g, _) = loops_with(&["Mikaeus, the Unhallowed", "Elesh Norn, Grand Cenobite", "Triskelion"], &[]);
    assert_eq!(mine::loop_need(&g, P0, &[]), mine::OUTLETS.to_vec());
}

#[test]
fn rest_in_peace_stops_the_loops() {
    let (_, l) = loops_with(&["Mikaeus, the Unhallowed", "Triskelion", "Viscera Seer"], &["Rest in Peace"]);
    assert_eq!(l, vec![]);
}

#[test]
fn the_ai_goes_for_a_killing_loop() {
    let (mut g, _) = loops_with(&["Mikaeus, the Unhallowed", "Triskelion", "Viscera Seer"], &[]);
    assert!(take_option(&mut g, P0, "loop: Mikaeus + Triskelion"));
    assert!(g.over);
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn a_loop_without_payoff_is_infinite_life() {
    let (mut g, _) = loops_with(&["Melira, Sylvok Outcast", "Kitchen Finks", "Ashnod's Altar"], &[]);
    let ls = mine::seph_loops(&g, P0);
    assert_eq!(ls.len(), 1);
    let (i, keys, kills) = ls[0].clone();
    mine::run_loop(&mut g, P0, i, &keys, kills).unwrap();
    assert!(!g.over);
    assert!(g.player(P0).life > 1000);
}

#[test]
fn aura_shards_clears_artifacts_and_enchantments() {
    let (mut g, _) = loops_with(&["Melira, Sylvok Outcast", "Kitchen Finks", "Ashnod's Altar", "Aura Shards"], &[]);
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Rite of the Dragoncaller");
    let snipe = perm(&mut g, P1, "Guttersnipe");
    let ls = mine::seph_loops(&g, P0);
    assert_eq!(ls.len(), 1);
    let (i, keys, kills) = ls[0].clone();
    mine::run_loop(&mut g, P0, i, &keys, kills).unwrap();
    assert_eq!(g.player(P1).perms, vec![snipe]);
}

#[test]
fn tutors_find_the_last_piece() {
    let (g, _) = loops_with(&["Mikaeus, the Unhallowed", "Viscera Seer"], &[]);
    assert_eq!(seph::seph_tutor_target(&g, P0), Some("Triskelion"));
}

#[test]
fn reanimation_takes_the_last_piece() {
    let (mut g, _) = loops_with(&["Melira, Sylvok Outcast", "Viscera Seer", "Blood Artist"], &[]);
    let finks = take(&mut g, P0, "Kitchen Finks");
    g.player_mut(P0).gy.push(finks);
    assert_eq!(decks::rean_targets(&g, P0, "animate")[0].1, finks);
}

#[test]
fn a_piece_that_completes_a_loop_is_cast_first() {
    let (mut g, _) = loops_with(&["Mikaeus, the Unhallowed", "Viscera Seer"], &[]);
    let t = hand(&mut g, P0, &["Triskelion"])[0];
    assert_eq!(seph::seph_prio(&g, P0, t), 85);
}

#[test]
fn kitchen_finks_persist() {
    let mut g = table(&["seph", "veyran"]);
    let finks = perm(&mut g, P0, "Kitchen Finks");
    assert_eq!(g.player(P0).life, 42); // enters: gain 2
    zones::die(&mut g, finks, "sac").unwrap();
    let back = named(&g, P0, "Kitchen Finks").unwrap();
    assert_eq!((g.perm(back).plus, g.player(P0).life), (-1, 44)); // persist: back with a -1/-1 counter
    zones::die(&mut g, back, "sac").unwrap();
    assert!(named(&g, P0, "Kitchen Finks").is_none());
}

#[test]
fn melira_keeps_persist_going() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Melira, Sylvok Outcast");
    let mut finks = perm(&mut g, P0, "Kitchen Finks");
    for _ in 0..3 {
        zones::die(&mut g, finks, "sac").unwrap();
        finks = named(&g, P0, "Kitchen Finks").unwrap();
        assert_eq!(g.perm(finks).plus, 0);
    }
    assert!(!zones::minus_counter(&mut g, finks, 1)); // no -1/-1 counters on your creatures
    assert!(values::melira(&g, P0));
}

#[test]
fn finks_at_zero_toughness_under_melira_stays_dead() {
    // Kitchen Finks (3/2) enters under an opponent's Elesh Norn (-2/-2): it dies at once, and under Melira persist
    // returns it without a counter every time, a mandatory loop
    let mut g = table(&["seph", "veyran"]);
    let melira = perm(&mut g, P0, "Melira, Sylvok Outcast");
    g.perm_mut(melira).plus = 2; // 4/4: she survives the Norn
    perm(&mut g, P1, "Elesh Norn, Grand Cenobite");
    perm(&mut g, P0, "Kitchen Finks");
    assert!(on_bf(&g, melira));
    assert!(named(&g, P0, "Kitchen Finks").is_none());
    assert!(g.player(P0).gy.contains(&card(&g, "Kitchen Finks")));
}

#[test]
fn teferis_protection_against_a_wipe() {
    let mut g = table(&["sauron", "seph"]);
    lands(&mut g, P1, "Plains", 3, false);
    hand(&mut g, P1, &["Teferi's Protection"]);
    perm(&mut g, P1, "Grave Titan");
    perm(&mut g, P1, "Archon of Cruelty");
    assert_eq!(ai::wipe_response(&mut g, P1, "destroy", P0).unwrap(), Some("all"));
    assert!(g.player(P1).life_locked);
}

#[test]
fn sephiroth_casts_necropotence_while_healthy() {
    // Necropotence keeps life above what the table could hit you for (necro_floor: the biggest opposing board plus 6,
    // at least 10); it's cast only when the life above that floor buys two or more cards
    let mut g = table(&["seph", "veyran"]);
    let necro = card(&g, "Necropotence");
    assert!(seph::seph_prio(&g, P0, necro) > 0);
    g.player_mut(P0).life = 11;
    assert_eq!(seph::seph_prio(&g, P0, necro), 0);
    g.player_mut(P0).life = 30;
    for _ in 0..3 {
        perm(&mut g, P1, "Grave Titan");
    }
    assert_eq!(seph::seph_prio(&g, P0, necro), 0); // three Titans: the floor is above 30
}

// ======================================================== Ephemerate (test_my_cards.NewCards)
#[test]
fn ephemerate_fizzles_targeted_removal() {
    // "Exile target creature you control, then return it to the battlefield under its owner's control. Rebound"
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Plains", 1, false);
    hand(&mut g, P0, &["Ephemerate"]);
    let grave = perm(&mut g, P0, "Grave Titan"); // enters: two Zombies
    let stp = card(&g, "Swords to Plowshares");
    removal::apply_removal(&mut g, Some(P1), grave, "exile", Some(stp)).unwrap();
    assert!(named(&g, P0, "Grave Titan").is_some());
    assert_eq!(tokens(&g, P0), 4); // entered again: two more Zombies
    let eph = card(&g, "Ephemerate");
    assert!(g.player(P0).exile.contains(&eph));
    assert_eq!(g.player(P0).rebound, vec![eph]);
}

#[test]
fn ephemerate_answers_theft() {
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Plains", 1, false);
    hand(&mut g, P0, &["Ephemerate"]);
    let grave = perm(&mut g, P0, "Grave Titan");
    let zc = card(&g, "Zealous Conscripts");
    assert!(ai::protect_response(&mut g, P0, grave, "steal", Some(P1), Some(zc)).unwrap());
    assert!(named(&g, P0, "Grave Titan").is_some());
}

#[test]
fn ephemerate_rebound_blinks_again() {
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Plains", 1, false);
    hand(&mut g, P0, &["Ephemerate"]);
    let grave = perm(&mut g, P0, "Grave Titan");
    assert!(mine::ephemerate_cast(&mut g, P0, grave, "value").unwrap());
    let eph = card(&g, "Ephemerate");
    let f = g.registry.get(eph).and_then(|i| i.rebound).expect("Ephemerate's rebound (common.rs)");
    f(&mut g, P0, eph).unwrap();
    assert_eq!(tokens(&g, P0), 6); // three Grave Titan entries
    assert!(g.player(P0).gy.contains(&eph));
}

#[test]
fn ephemerate_value_blink_offered() {
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Plains", 1, false);
    hand(&mut g, P0, &["Ephemerate"]);
    perm(&mut g, P0, "Grave Titan");
    assert!(labels(&options(&mut g, P0, true)).contains(&"Ephemerate (blink Grave Titan)".to_string()));
}

// ======================================================== Summon: Bahamut (test_my_cards.Bahamut)
// "(As this Saga enters and after your draw step, add a lore counter. Sacrifice after IV.) I, II - Destroy up to one
// target nonland permanent. III - Draw two cards. IV - Mega Flare - This creature deals damage equal to the total mana
// value of other permanents you control to each opponent." 9/9 flying
#[test]
fn chapter_one_destroys_the_biggest_threat() {
    let mut g = table(&["seph", "veyran"]);
    let snipe = perm(&mut g, P1, "Guttersnipe");
    perm(&mut g, P1, "Veyran, Voice of Duality");
    lands(&mut g, P1, "Island", 1, false);
    let b = perm(&mut g, P0, B);
    assert_eq!(lore(&g, b), 1);
    let x = g.perm(b);
    assert!(x.fly && (x.pow, x.tgh) == (9, 9));
    assert_eq!(g.player(P1).perms, vec![snipe]); // Veyran, not Guttersnipe; lands are never targets
    assert_eq!(g.player(P1).lands.len(), 1);
}

#[test]
fn chapter_two_after_your_draw_step() {
    let mut g = table(&["seph", "veyran"]);
    let b = perm(&mut g, P0, B);
    let snipe = perm(&mut g, P1, "Guttersnipe");
    main1(&mut g, P1, 1); // an opponent's main phase adds nothing
    assert_eq!((lore(&g, b), g.player(P1).perms.clone()), (1, vec![snipe]));
    turn::step_start(&mut g, P0).unwrap(); // upkeep, draw step, then the lore counter
    assert_eq!(lore(&g, b), 2);
    assert_eq!(g.player(P1).perms, vec![]);
}

#[test]
fn chapter_three_draws_two() {
    let mut g = table(&["seph", "veyran"]);
    let b = perm(&mut g, P0, B);
    main1(&mut g, P0, 1);
    let n = g.player(P0).hand.len();
    main1(&mut g, P0, 1);
    assert_eq!((lore(&g, b), g.player(P0).hand.len()), (3, n + 2));
}

#[test]
fn mega_flare_then_sacrificed() {
    let mut g = table(&["seph", "veyran", "sauron"]);
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P0, "Grave Titan");
    lands(&mut g, P0, "Swamp", 3, false); // 1 + 6 (Zombies and lands: 0)
    let b = perm(&mut g, P0, B);
    main1(&mut g, P0, 2);
    main1(&mut g, P0, 1);
    assert_eq!(lives(&g), vec![40, 33, 33]);
    assert!(!on_bf(&g, b));
    assert!(g.player(P0).gy.contains(&card(&g, B))); // sacrificed: back in the graveyard to reanimate
}

#[test]
fn mega_flare_kills() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Grave Titan");
    g.player_mut(P1).life = 6;
    perm(&mut g, P0, B);
    main1(&mut g, P0, 3);
    assert!(!g.player(P1).alive);
    assert!(g.over && g.winner == Some(P0));
}

#[test]
fn reanimated_at_chapter_one() {
    let mut g = table(&["seph", "veyran"]);
    let vey = perm(&mut g, P1, "Veyran, Voice of Duality");
    lands(&mut g, P0, "Swamp", 2, false);
    hand(&mut g, P0, &["Animate Dead"]);
    let titan = take(&mut g, P0, "Grave Titan");
    let bah = take(&mut g, P0, B);
    g.player_mut(P0).gy.extend([titan, bah]);
    assert_eq!(decks::rean_targets(&g, P0, "animate")[0].1, bah); // above Grave Titan
    assert!(seph::seph_reanimate(&mut g, P0).unwrap());
    assert_eq!(lore(&g, named(&g, P0, B).unwrap()), 1);
    assert!(!on_bf(&g, vey));
}

#[test]
fn entomb_target() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Sol Ring");
    let bah = card(&g, B);
    if !g.player(P0).library.contains(&bah) {
        g.player_mut(P0).library.push(bah);
    }
    ai::seph_fill_resolve(&mut g, P0, "entomb", &Ctx::default()).unwrap();
    assert_eq!(g.player(P0).gy, vec![card(&g, "Archon of Cruelty")]); // the one bomb above it on a small board
    g.player_mut(P0).gy.clear(); // then above the other 8s (Sheoldred, Elesh Norn)
    ai::seph_fill_resolve(&mut g, P0, "entomb", &Ctx::default()).unwrap();
    assert_eq!(g.player(P0).gy, vec![bah]);
}

#[test]
fn blink_restarts_at_chapter_one() {
    let mut g = table(&["seph", "veyran"]);
    let b = perm(&mut g, P0, B);
    main1(&mut g, P0, 2); // chapter III done: IV next
    let vey = perm(&mut g, P1, "Veyran, Voice of Duality");
    lands(&mut g, P0, "Plains", 1, false);
    hand(&mut g, P0, &["Ephemerate"]);
    assert!(mine::ephemerate_cast(&mut g, P0, b, "value").unwrap());
    let n = named(&g, P0, B).unwrap();
    assert_ne!(n, b);
    assert_eq!(lore(&g, n), 1); // a new object: chapter I again
    assert!(!on_bf(&g, vey));
    main1(&mut g, P0, 2);
    assert!(on_bf(&g, n)); // not sacrificed: the count started over
}

#[test]
fn kitten_blinks_it_for_a_target() {
    let mut g = table(&["seph", "veyran"]);
    let b = perm(&mut g, P0, B);
    perm(&mut g, P0, "Displacer Kitten");
    let vey = perm(&mut g, P1, "Veyran, Voice of Duality");
    let stp = card(&g, "Swords to Plowshares");
    cast::on_cast(&mut g, P0, stp).unwrap();
    assert!(!on_bf(&g, b));
    assert!(!on_bf(&g, vey));
}

#[test]
fn no_blink_before_a_lethal_mega_flare() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Grave Titan");
    g.player_mut(P1).life = 6;
    let b = perm(&mut g, P0, B);
    main1(&mut g, P0, 2);
    perm(&mut g, P1, "Veyran, Voice of Duality");
    assert_eq!(mine::blink_worth(&g, P0, b), 0.0);
}

#[test]
fn hardcast_needs_nine_mana() {
    let mut g = table(&["seph", "veyran"]);
    g.player_mut(P0).cmd_in_zone = false;
    hand(&mut g, P0, &[B]);
    lands(&mut g, P0, "Swamp", 8, false);
    assert!(!seph::seph_hardcast(&mut g, P0).unwrap());
    lands(&mut g, P0, "Swamp", 1, false);
    assert!(seph::seph_hardcast(&mut g, P0).unwrap());
    assert_eq!(lore(&g, named(&g, P0, B).unwrap()), 1);
}

// ======================================================== the flicker package (test_my_cards.SephFlicker)
fn end_step(g: &mut Game, p: PlayerId) {
    hooks::fire_trigger(g, Event::EndStep, Call::Player { p }).unwrap();
}

#[test]
fn soulherder_flickers_the_commander_at_your_end_step() {
    let mut g = table(&["seph", "veyran"]);
    let herd = perm(&mut g, P0, "Soulherder");
    perm(&mut g, P0, "Grave Titan");
    let atx = perm(&mut g, P0, "Atraxa, Grand Unifier");
    g.perm_mut(atx).is_cmd = true;
    end_step(&mut g, P1); // an opponent's end step: nothing
    assert!(on_bf(&g, atx));
    let n = g.player(P0).hand.len();
    end_step(&mut g, P0);
    assert!(!on_bf(&g, atx)); // Atraxa (8) over Grave Titan (3)
    let new = named(&g, P0, "Atraxa, Grand Unifier").unwrap();
    assert!(g.perm(new).is_cmd); // still the commander
    assert!(g.player(P0).hand.len() > n); // its reveal-ten enter trigger again
    assert_eq!(g.perm(herd).plus, 1); // a creature was exiled from the battlefield
}

#[test]
fn closet_restarts_bahamut_for_another_destroy() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Conjurer's Closet");
    let b = perm(&mut g, P0, B);
    main1(&mut g, P0, 1); // chapter II: nothing to destroy yet
    let vey = perm(&mut g, P1, "Veyran, Voice of Duality");
    end_step(&mut g, P0);
    assert!(!on_bf(&g, b));
    assert_eq!(lore(&g, named(&g, P0, B).unwrap()), 1); // a new object at chapter I
    assert!(!on_bf(&g, vey));
}

#[test]
fn no_restart_before_a_strong_mega_flare() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Conjurer's Closet");
    let titan = perm(&mut g, P0, "Grave Titan"); // Mega Flare: 5 + 6 = 11
    let b = perm(&mut g, P0, B);
    main1(&mut g, P0, 2);
    perm(&mut g, P1, "Veyran, Voice of Duality");
    end_step(&mut g, P0);
    assert!(on_bf(&g, b)); // Mega Flare next turn instead
    assert_eq!(lore(&g, b), 3);
    assert!(!on_bf(&g, titan)); // the next best enter effect: two more Zombies
}

#[test]
fn counters_worth_more_than_the_enter_effect() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Conjurer's Closet");
    let titan = perm(&mut g, P0, "Grave Titan");
    g.perm_mut(titan).plus = 6;
    end_step(&mut g, P0);
    assert!(on_bf(&g, titan)); // two Zombies aren't worth six +1/+1 counters
    assert_eq!(g.perm(titan).plus, 6);
}

#[test]
fn restoration_angel_saves_a_bomb_from_removal() {
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Plains", 4, false);
    hand(&mut g, P0, &["Restoration Angel"]);
    let archon = perm(&mut g, P0, "Archon of Cruelty");
    let stp = card(&g, "Swords to Plowshares");
    removal::apply_removal(&mut g, Some(P1), archon, "exile", Some(stp)).unwrap();
    assert!(!on_bf(&g, archon));
    assert!(named(&g, P0, "Archon of Cruelty").is_some()); // blinked: the Swords fizzled
    assert!(named(&g, P0, "Restoration Angel").is_some());
    assert_eq!(g.player(P0).stats.get("resto_protection").copied(), Some(1));
}

#[test]
fn restoration_angel_blinks_the_best_enter_effect() {
    let mut g = table(&["seph", "veyran"]);
    let titan = perm(&mut g, P0, "Grave Titan");
    let archon = perm(&mut g, P0, "Archon of Cruelty");
    let atx = perm(&mut g, P0, "Atraxa, Grand Unifier");
    g.perm_mut(atx).is_cmd = true;
    perm(&mut g, P0, "Restoration Angel");
    assert!(on_bf(&g, atx)); // a Phyrexian Angel: not a legal target
    assert!(on_bf(&g, titan));
    assert!(!on_bf(&g, archon)); // Archon's drain over Grave Titan's Zombies
}

#[test]
fn teleportation_circle_flickers_the_best_creature() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Teleportation Circle");
    perm(&mut g, P0, "Grave Titan");
    let archon = perm(&mut g, P0, "Archon of Cruelty");
    end_step(&mut g, P1); // an opponent's end step: nothing
    assert!(on_bf(&g, archon));
    end_step(&mut g, P0);
    assert!(!on_bf(&g, archon)); // Archon's drain over Grave Titan's Zombies
    assert!(named(&g, P0, "Archon of Cruelty").is_some());
    assert_eq!(g.player(P0).stats.get("flicker Teleportation Circle").copied(), Some(1));
}

#[test]
fn teleportation_circle_untaps_a_rock_with_no_creature_worth_it() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Teleportation Circle");
    let rock = perm(&mut g, P0, "Sol Ring");
    g.perm_mut(rock).tapped = true;
    end_step(&mut g, P0);
    assert!(!on_bf(&g, rock));
    assert!(!g.perm(named(&g, P0, "Sol Ring").unwrap()).tapped); // back untapped for the opponents' turns
    let mut g2 = table(&["seph", "veyran"]);
    perm(&mut g2, P0, "Teleportation Circle");
    let rock2 = perm(&mut g2, P0, "Sol Ring");
    end_step(&mut g2, P0);
    assert!(on_bf(&g2, rock2)); // an untapped rock gains nothing: left alone
}

#[test]
fn flickering_hound_flickers_on_your_creature_casts() {
    let mut g = table(&["seph", "veyran"]);
    let hound = perm(&mut g, P0, "Flickering Hound");
    perm(&mut g, P0, "Grave Titan");
    let archon = perm(&mut g, P0, "Archon of Cruelty");
    let cast_ev = |g: &mut Game, p: PlayerId, n: &str| {
        let c = card(g, n);
        hooks::fire_trigger(g, Event::Cast, Call::Cast { caster: p, c }).unwrap();
    };
    cast_ev(&mut g, P1, "Grave Titan"); // an opponent's creature spell: nothing
    assert!(on_bf(&g, archon));
    cast_ev(&mut g, P0, "Swords to Plowshares"); // your noncreature spell: nothing
    assert!(on_bf(&g, archon));
    cast_ev(&mut g, P0, "Viscera Seer"); // your creature spell: Archon blinks
    assert!(!on_bf(&g, archon));
    assert!(named(&g, P0, "Archon of Cruelty").is_some());
    assert!(on_bf(&g, hound)); // never itself
    assert_eq!(g.player(P0).stats.get("flicker Flickering Hound").copied(), Some(1));
}

#[test]
fn flicker_priorities() {
    let mut g = table(&["seph", "veyran"]);
    let (circle, resto, closet) =
        (card(&g, "Teleportation Circle"), card(&g, "Restoration Angel"), card(&g, "Conjurer's Closet"));
    assert_eq!(seph::seph_prio(&g, P0, circle), 30);
    assert_eq!(seph::seph_prio(&g, P0, closet), 30);
    perm(&mut g, P0, "Archon of Cruelty");
    assert_eq!(seph::seph_prio(&g, P0, closet), 50); // a creature worth flickering
    assert_eq!(seph::seph_prio(&g, P0, circle), 50);
    assert_eq!(seph::seph_prio(&g, P0, resto), 0); // your own turn: wait for the end of an opponent's
    g.active = Some(P1);
    assert_eq!(seph::seph_prio(&g, P0, resto), 30 + 6 * 5);
    let mut g2 = table(&["seph", "veyran"]);
    g2.active = Some(P1);
    assert_eq!(seph::seph_prio(&g2, P0, resto), 25); // nothing to blink: a 3/4 flash flier
    perm(&mut g2, P0, "Sheoldred, Whispering One");
    assert_eq!(seph::seph_prio(&g2, P0, resto), 0); // held to protect the bomb
}

#[test]
fn bahamut_is_a_saga_proliferate_can_count() {
    // CI.SAGA for common.proliferate and Karn's Bastion (their own tests come with lands.rs and common.rs): toward
    // chapters II and III, and IV only when Mega Flare is strong
    let mut g = table(&["seph", "veyran"]);
    let b = perm(&mut g, P0, B);
    assert!(sim_core::cardcode::saga_wants_lore(&g, P0, b));
    main1(&mut g, P0, 2);
    assert_eq!(lore(&g, b), 3);
    assert!(!sim_core::cardcode::saga_wants_lore(&g, P0, b)); // Mega Flare would deal 0
    g.player_mut(P1).life = 6;
    perm(&mut g, P0, "Grave Titan"); // now it kills
    assert!(sim_core::cardcode::saga_wants_lore(&g, P0, b));
    sim_core::cardcode::saga_proliferated(&mut g, P0, b).unwrap();
    assert!(!g.player(P1).alive);
    assert!(!on_bf(&g, b));
    assert_eq!(g.player(P0).stats.get("bahamut_proliferated").copied(), Some(1));
}

#[test]
fn bastion_proliferates_bahamut_and_the_loop_counters() {
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Karn's Bastion", 1, false);
    lands(&mut g, P0, "Swamp", 4, false);
    let trisk = perm(&mut g, P0, "Triskelion"); // three +1/+1 counters
    let finks = perm(&mut g, P0, "Kitchen Finks");
    g.perm_mut(finks).plus = -1; // persisted: a -1/-1 counter
    let bastion = |g: &mut Game| -> Vec<Opt> {
        sim_core::cardcode::land_options(g, P0, Some(false))
            .unwrap()
            .into_iter()
            .filter(|o| o.label == "Karn's Bastion")
            .collect()
    };
    assert!(bastion(&mut g).is_empty()); // counters alone: at the end of the turn before yours
    let b = perm(&mut g, P0, B);
    let vey = perm(&mut g, P1, "Veyran, Voice of Duality");
    let opts = bastion(&mut g); // a chapter: in your main phase too
    assert_eq!(opts.len(), 1);
    assert!(act::perform(&mut g, P0, opts[0].act.as_ref().unwrap()).unwrap());
    assert_eq!(lore(&g, b), 2); // chapter II triggers
    assert!(!on_bf(&g, vey));
    assert_eq!((g.perm(trisk).plus, g.perm(finks).plus), (4, -1)); // the loop's counters: more pings
    assert_eq!(g.player(P0).stats.get("bahamut_proliferated").copied(), Some(1));
}

// ======================================================== Nim Deathmantle (test_my_cards.Marchesa)
#[test]
fn nim_deathmantle_leaves_marchesa_returns_alone() {
    // Nim Deathmantle doesn't pay {4} for a creature Marchesa returns for free
    let mut g = table(&["jodah", "veyran"]);
    lands(&mut g, P0, "Swamp", 4, false);
    perm(&mut g, P0, "Marchesa, the Black Rose");
    perm(&mut g, P0, "Nim Deathmantle");
    let rat = perm(&mut g, P0, "Burglar Rat");
    g.perm_mut(rat).plus = 1;
    zones::die(&mut g, rat, "destroy").unwrap();
    assert!(!g.player(P0).lands.iter().any(|&l| g.land(l).tapped));
}

#[test]
#[ignore = "needs Mayhem Devil's (t3.rs) and Dauthi Voidwalker's (common.rs) card code: un-ignore after the merge"]
fn nim_deathmantle_card_exiled_while_paying() {
    // paying {4} with Treasures sets off Mayhem Devil; its ping kills Dark Confidant, and the state-based check that
    // follows runs Dauthi Voidwalker's sweep: the card has left the graveyard, so nothing returns (the mana is spent)
    let mut g = table(&["seph", "veyran", "veyran"]);
    g.player_mut(P0).treasures = 4;
    perm(&mut g, P0, "Nim Deathmantle");
    perm(&mut g, P1, "Mayhem Devil");
    perm(&mut g, P2, "Dauthi Voidwalker");
    perm(&mut g, P2, "Dark Confidant");
    let titan = perm(&mut g, P0, "Grave Titan");
    zones::die(&mut g, titan, "destroy").unwrap();
    assert_eq!(g.player(P0).treasures, 0);
    let c = card(&g, "Grave Titan");
    assert!(g.player(P0).exile.contains(&c));
    assert!(named(&g, P0, "Grave Titan").is_none());
}

#[test]
fn nim_deathmantle_returns_a_bomb_and_equips_it() {
    // "Whenever a nontoken creature is put into your graveyard from the battlefield, you may pay {4}. If you do,
    //  return that card to the battlefield and attach Nim Deathmantle to it."
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Swamp", 4, false);
    let nim = perm(&mut g, P0, "Nim Deathmantle");
    let archon = perm(&mut g, P0, "Archon of Cruelty");
    zones::die(&mut g, archon, "destroy").unwrap();
    let back = named(&g, P0, "Archon of Cruelty").expect("returned");
    assert_eq!(g.perm(nim).attached, Some(back));
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.land(l).tapped).count(), 4);
    // a 1/1 without an enter effect isn't worth {4}
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Swamp", 4, false);
    perm(&mut g, P0, "Nim Deathmantle");
    let seer = perm(&mut g, P0, "Viscera Seer");
    zones::die(&mut g, seer, "destroy").unwrap();
    assert!(named(&g, P0, "Viscera Seer").is_none());
}

// ======================================================== Muldrotha (test_audit_fixes.Muldrotha)
fn muldrotha_table() -> (Game, PermId) {
    let mut g = table(&["seph", "veyran", "sauron"]);
    let m = perm(&mut g, P0, "Muldrotha, the Gravetide");
    (g, m)
}

#[test]
fn muldrotha_one_permanent_of_each_type_from_the_graveyard() {
    let (mut g, _) = muldrotha_table();
    let (sol, stone, tower) =
        (take(&mut g, P0, "Sol Ring"), take(&mut g, P0, "Mind Stone"), take(&mut g, P0, "Command Tower"));
    g.player_mut(P0).gy.extend([sol, stone, tower]);
    assert_eq!(mine::muld_types(&g, P0, sol), vec!["A"]);
    mine::muld_mark(&mut g, P0, "A");
    assert_eq!(mine::muld_types(&g, P0, stone), Vec::<&str>::new()); // artifact used this turn
    assert_eq!(mine::muld_types(&g, P0, tower), vec!["L"]);
    g.active = Some(P1);
    assert_eq!(mine::muld_types(&g, P0, tower), Vec::<&str>::new()); // only during your turns
}

#[test]
fn muldrotha_the_ai_casts_from_the_graveyard() {
    let (mut g, _) = muldrotha_table();
    lands(&mut g, P0, "Swamp", 2, false);
    let sol = take(&mut g, P0, "Sol Ring");
    g.player_mut(P0).gy.push(sol);
    assert!(labels(&options(&mut g, P0, false)).iter().any(|l| l.contains("Sol Ring")));
    assert!(take_option(&mut g, P0, "Sol Ring from the graveyard (Muldrotha)"));
    assert!(named(&g, P0, "Sol Ring").is_some());
    assert_eq!(mine::muld_used(&g, P0), vec!["A"]);
}

#[test]
fn muldrotha_the_ai_plays_a_land_from_the_graveyard() {
    let (mut g, _) = muldrotha_table();
    let tower = take(&mut g, P0, "Command Tower");
    g.player_mut(P0).gy.push(tower);
    turn::play_land(&mut g, P0).unwrap();
    assert!(g.player(P0).lands.iter().any(|&l| g.land(l).cd == tower));
    assert!(mine::muld_used(&g, P0).contains(&"L"));
}

// ======================================================== Sephiroth's other cards
#[test]
fn triskelion_enters_with_three_counters_and_pings() {
    // "Triskelion enters with three +1/+1 counters on it. Remove a +1/+1 counter from Triskelion: It deals 1 damage
    //  to any target."
    let mut g = table(&["seph", "veyran"]);
    let t = perm(&mut g, P0, "Triskelion");
    assert_eq!(g.perm(t).plus, 3);
    g.player_mut(P1).life = 2;
    let opts = options(&mut g, P0, false);
    let o = opts.iter().find(|o| o.label == "Triskelion ping").expect("a lethal ping");
    assert_eq!(o.utility, 8.0);
    act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap();
    act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap();
    life::check_state(&mut g).unwrap();
    assert!(!g.player(P1).alive);
    assert_eq!(g.perm(t).plus, 1);
}

#[test]
fn displacer_kitten_flickers_the_best_enter_effect() {
    // "Whenever you cast a noncreature spell, exile up to one target nonland permanent you control. Return that
    //  permanent to the battlefield under its owner's control."
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Displacer Kitten");
    let titan = perm(&mut g, P0, "Grave Titan");
    let archon = perm(&mut g, P0, "Archon of Cruelty");
    let life = g.player(P1).life;
    let stp = card(&g, "Swords to Plowshares");
    cast::on_cast(&mut g, P0, stp).unwrap();
    assert!(on_bf(&g, titan));
    assert!(!on_bf(&g, archon)); // Archon (6) over Grave Titan (3)
    assert!(named(&g, P0, "Archon of Cruelty").is_some());
    assert_eq!(g.player(P1).life, life - 3); // its drain again
    // a creature spell doesn't trigger it
    let seer = card(&g, "Viscera Seer");
    let n = named(&g, P0, "Archon of Cruelty").unwrap();
    cast::on_cast(&mut g, P0, seer).unwrap();
    assert!(on_bf(&g, n));
}

#[test]
fn strip_mine_takes_a_key_land() {
    let mut g = table(&["seph", "sauron"]);
    let strip = lands(&mut g, P0, "Strip Mine", 1, false)[0];
    let tomb = lands(&mut g, P1, "Urborg, Tomb of Yawgmoth", 1, false)[0];
    let opts = sim_core::cardcode::land_options(&mut g, P0, Some(false)).unwrap();
    let o = opts.iter().find(|o| o.label.starts_with("Strip Mine -> ")).expect("Strip Mine's option");
    assert_eq!(o.label, "Strip Mine -> Urborg, Tomb of Yawgmoth");
    assert!(act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(!g.player(P0).lands.contains(&strip));
    assert!(!g.player(P1).lands.contains(&tomb));
    assert!(g.player(P0).gy.contains(&card(&g, "Strip Mine")));
}

#[test]
fn necromancy_flashes_in_for_an_enter_effect() {
    // cast at instant speed at the end of an opponent's turn for an enter effect (Archon of Cruelty): sacrificed at
    // cleanup
    let mut g = table(&["seph", "veyran"]);
    g.active = Some(P1);
    lands(&mut g, P0, "Swamp", 3, false);
    hand(&mut g, P0, &["Necromancy"]);
    let atx = take(&mut g, P0, "Archon of Cruelty");
    g.player_mut(P0).gy.push(atx);
    let n = g.player(P0).hand.len();
    let s = brain::Situation::new(&g, P0);
    let mut opts = brain::hook_options(&mut g, P0, &s, None).unwrap();
    opts.retain(|o| o.label.starts_with("Necromancy (flash)"));
    assert_eq!(opts.len(), 1);
    assert!(act::perform(&mut g, P0, opts[0].act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P0).hand.len(), n); // Necromancy cast, a card from Archon's trigger
    assert_eq!(g.player(P1).life, 37); // and its drain
    assert!(named(&g, P0, "Archon of Cruelty").is_none()); // sacrificed
    assert!(g.player(P0).gy.contains(&atx) && g.player(P0).gy.contains(&card(&g, "Necromancy")));
}

// ======================================================== the plan's choices
#[test]
fn seph_reanimates_a_bomb_with_the_cheapest_spell() {
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Swamp", 4, false);
    hand(&mut g, P0, &["Animate Dead", "Necromancy"]);
    let archon = take(&mut g, P0, "Archon of Cruelty");
    g.player_mut(P0).gy.push(archon);
    let opts = options(&mut g, P0, false);
    assert!(labels(&opts).contains(&"reanimate".to_string()));
    assert!(take_option(&mut g, P0, "reanimate"));
    assert!(named(&g, P0, "Archon of Cruelty").is_some());
    assert!(g.player(P0).gy.contains(&card(&g, "Animate Dead"))); // the cheaper one ({1}{B})
    assert!(g.player(P0).hand.contains(&card(&g, "Necromancy")));
    assert_eq!(g.player(P0).stats.get("rean_cast").copied(), Some(1));
}

#[test]
fn seph_fills_the_graveyard_before_reanimating() {
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Swamp", 3, false);
    hand(&mut g, P0, &["Entomb", "Animate Dead"]);
    let labels = labels(&options(&mut g, P0, false));
    assert!(labels.contains(&"fill graveyard".to_string()));
    assert!(take_option(&mut g, P0, "fill graveyard"));
    assert!(seph::own_bomb_in_gy(&g, P0));
}

#[test]
fn seph_tutors_for_what_the_plan_misses() {
    let mut g = table(&["seph", "veyran"]);
    // a reanimation spell but nothing to reanimate: Entomb first
    hand(&mut g, P0, &["Animate Dead"]);
    assert_eq!(seph::seph_tutor_target(&g, P0), Some("Entomb"));
    // a bomb in the graveyard but no spell: the reanimation spells
    let mut g = table(&["seph", "veyran"]);
    let archon = take(&mut g, P0, "Archon of Cruelty");
    g.player_mut(P0).gy.push(archon);
    assert_eq!(seph::seph_tutor_target(&g, P0), Some("Reanimate")); // life above 22
    g.player_mut(P0).life = 20;
    assert_eq!(seph::seph_tutor_target(&g, P0), Some("Animate Dead"));
    // the tutor picks it
    assert_eq!(decks::tutor_pick(&g, P0, "any"), Some(card(&g, "Animate Dead")));
}

#[test]
fn seph_hardcasts_the_commander() {
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Command Tower", 7, false);
    let labels = labels(&options(&mut g, P0, false));
    assert!(labels.contains(&"hardcast bomb".to_string()));
    assert!(take_option(&mut g, P0, "hardcast bomb"));
    assert!(named(&g, P0, "Atraxa, Grand Unifier").is_some());
    assert!(g.player(P0).first_bomb.is_some());
}

#[test]
fn seph_dredges_with_a_reanimation_spell_and_no_bomb() {
    let mut g = table(&["seph", "veyran"]);
    let imp = take(&mut g, P0, "Stinkweed Imp");
    g.player_mut(P0).gy.push(imp);
    assert!(!ai::seph_dredge(&mut g, P0).unwrap()); // no way to reanimate: draw
    hand(&mut g, P0, &["Animate Dead"]);
    let lib = g.player(P0).library.len();
    assert!(ai::seph_dredge(&mut g, P0).unwrap());
    assert!(g.player(P0).hand.contains(&imp));
    assert_eq!(g.player(P0).library.len(), lib - 5);
}

#[test]
fn seph_saves_a_bomb_with_heroic_intervention_and_galadriels_dismissal() {
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Forest", 2, false);
    hand(&mut g, P0, &["Heroic Intervention"]);
    let archon = perm(&mut g, P0, "Archon of Cruelty");
    assert!(ai::protect_response(&mut g, P0, archon, "destroy", Some(P1), None).unwrap());
    assert_eq!(g.player(P0).stats.get("hi_used").copied(), Some(1));
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Plains", 1, false);
    hand(&mut g, P0, &["Galadriel's Dismissal"]);
    let archon = perm(&mut g, P0, "Archon of Cruelty");
    assert!(ai::protect_response(&mut g, P0, archon, "exile", Some(P1), None).unwrap());
    assert!(g.perm(archon).phased);
}

#[test]
fn seph_answers_a_wipe_with_galadriels_dismissal_kicked() {
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Plains", 4, false); // kicked: {2}{W} more
    hand(&mut g, P0, &["Galadriel's Dismissal"]);
    let titan = perm(&mut g, P0, "Grave Titan");
    let archon = perm(&mut g, P0, "Archon of Cruelty");
    assert_eq!(ai::wipe_response(&mut g, P0, "destroy", P1).unwrap(), Some("all"));
    assert!(g.perm(titan).phased && g.perm(archon).phased);
}

#[test]
fn seph_sacrifices_a_bomb_to_exile_under_conquerors_flail() {
    // Conqueror's Flail: no spells in response; a free sacrifice outlet still saves the card for reanimation
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Viscera Seer");
    let archon = perm(&mut g, P0, "Archon of Cruelty");
    g.active = Some(P1); // the opponent's turn, its Flail on one of its creatures
    let snipe = perm(&mut g, P1, "Guttersnipe");
    let flail = perm(&mut g, P1, "Conqueror's Flail");
    g.perm_mut(flail).attached = Some(snipe);
    assert!(values::silenced(&g, P0));
    assert!(ai::protect_response(&mut g, P0, archon, "exile", Some(P1), None).unwrap());
    assert!(g.player(P0).gy.contains(&card(&g, "Archon of Cruelty")));
    assert_eq!(g.player(P0).stats.get("sac_saves").copied(), Some(1));
}

#[test]
fn seph_milestones_at_the_end_step() {
    let mut g = table(&["seph", "veyran"]);
    perm(&mut g, P0, "Archon of Cruelty");
    lands(&mut g, P0, "Island", 2, false);
    hand(&mut g, P0, &["Counterspell"]);
    ai::seph_end_milestones(&mut g, P0);
    let ms = &g.player(P0).milestone;
    assert_eq!(ms.get("bomb_by").copied(), Some(1));
    assert_eq!(ms.get("int_up:1").copied(), Some(1));
    assert_eq!(ms.get("int_held:1").copied(), Some(1));
}

#[test]
fn seph_bval_counts_mega_flares_reach() {
    let mut g = table(&["seph", "veyran"]);
    let b = card(&g, B);
    assert_eq!(seph::seph_bval(&g, P0, b), 8.0);
    perm(&mut g, P0, "Grave Titan"); // 6 mana value: +0.4
    assert!((seph::seph_bval(&g, P0, b) - 8.4).abs() < 1e-9);
}

#[test]
fn whole_games_with_sephiroth_finish() {
    for seed in [1, 2, 3] {
        let g = play(&["seph", "veyran", "sauron"], seed, false);
        assert!(g.over || g.round >= 20, "seed {seed}");
    }
}
