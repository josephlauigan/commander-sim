//! Sauron's cards on hand-built positions: Python's tests/test_my_cards.py, classes Sauron, SauronBreach and
//! SauronMarchesa. Each expectation comes from the card's Oracle text (quoted where the Python quotes it).
//!
//! Some tests use cards no current decklist runs (Witch-king, Bringer of Ruin; Brush Off; Grapeshot; Lion's Eye
//! Diamond; Laboratory Maniac): they are still in the card database, and they check branches of the Breach line, so
//! they run. SauronMarchesa is ignored: it needs marchesa.py's card code (Marchesa is in Jodah's list now, ported in
//! phase 6); `cargo test -- --ignored` runs it.

use sim_core::ai::{act, brain, decks};
use sim_core::engine::{cast, removal, stack, turn, values, zones};
use sim_core::hooks::Opt;
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::mine;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn on_cast(g: &mut Game, p: PlayerId, name: &str) {
    let c = card(g, name);
    cast::on_cast(g, p, c).unwrap();
}

fn army(g: &Game, p: PlayerId) -> PermId {
    values::army_of(g, p).expect("an Army")
}

fn options(g: &mut Game, p: PlayerId) -> Vec<Opt> {
    brain::main_options(g, p, false).unwrap()
}

fn utility(opts: &[Opt], label: &str) -> f64 {
    opts.iter().find(|o| o.label == label).unwrap_or_else(|| panic!("no option {label:?}")).utility
}

/// make the option with this label
fn take_option(g: &mut Game, p: PlayerId, label: &str) -> bool {
    let opts = options(g, p);
    let o = opts.iter().find(|o| o.label == label).unwrap_or_else(|| panic!("no option {label:?}"));
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn tapped_lands(g: &Game, p: PlayerId) -> usize {
    g.player(p).lands.iter().filter(|&&l| g.land(l).tapped).count()
}

// ======================================================== Sauron
#[test]
fn sauron_amasses_when_an_opponent_casts() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Sauron, the Dark Lord");
    on_cast(&mut g, P1, "Lightning Bolt");
    on_cast(&mut g, P1, "Think Twice");
    assert_eq!(values::epow(&g, army(&g, P0)), 2);
}

#[test]
fn kaervek() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Kaervek the Merciless");
    on_cast(&mut g, P1, "Counterspell"); // mana value 2
    assert_eq!(g.player(P1).life, 38);
}

#[test]
fn witch_king_takes_the_least_power() {
    // "defending player sacrifices a creature with the least power among creatures they control"
    let mut g = table(&["sauron", "veyran"]);
    let w = perm(&mut g, P0, "Witch-king, Bringer of Ruin");
    let snipe = perm(&mut g, P1, "Guttersnipe");
    perm(&mut g, P1, "Kessig Flamebreather"); // power 2 and 1
    sim_core::engine::combat::attack_triggers(&mut g, P0, &[w], P1).unwrap();
    assert_eq!(g.player(P1).perms, vec![snipe]);
}

#[test]
fn orcish_bowmasters() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Orcish Bowmasters");
    assert_eq!(g.player(P1).life, 39); // enters: 1 damage, then amass 1
    assert!(values::army_of(&g, P0).is_some());
    zones::draw(&mut g, P1, 1, true).unwrap(); // the first card of a draw step doesn't count
    assert_eq!(g.player(P1).life, 39);
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!(g.player(P1).life, 38);
}

#[test]
fn rhystic_study() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Rhystic Study");
    on_cast(&mut g, P1, "Lightning Bolt"); // the caster has no mana to pay {1}
    assert_eq!(g.player(P0).hand.len(), 1);
}

#[test]
fn champions_helm_goes_on_a_legend() {
    // "Equipped creature gets +2/+2. As long as equipped creature is legendary, it has hexproof. Equip {1}"
    let mut g = table(&["sauron", "veyran"]);
    lands(&mut g, P0, "Island", 3, false);
    let sauron = perm(&mut g, P0, "Sauron, the Dark Lord");
    zones::amass(&mut g, P0, 3).unwrap();
    perm(&mut g, P0, "Champion's Helm");
    assert!(take_option(&mut g, P0, "equip Champion's Helm to Sauron, the Dark Lord")); // no Sword needed
    assert_eq!(values::epow(&g, sauron), 9);
    assert!(values::untargetable(&g, sauron));
    assert_eq!(tapped_lands(&g, P0), 1);
}

#[test]
fn sauron_casts_sheoldred_early() {
    let mut g = table(&["sauron", "veyran"]);
    lands(&mut g, P0, "Swamp", 4, false);
    hand(&mut g, P0, &["Sheoldred, the Apocalypse", "Night's Whisper"]);
    let opts = options(&mut g, P0);
    assert!(utility(&opts, "Sheoldred, the Apocalypse") > utility(&opts, "Night's Whisper"));
}

#[test]
fn slaughter_pact_hits_nonblack_only() {
    let mut g = table(&["sauron", "seph"]);
    perm(&mut g, P1, "Blood Artist");
    let birds = perm(&mut g, P1, "Birds of Paradise");
    let pact = card(&g, "Slaughter Pact");
    assert_eq!(removal::legal_targets(&g, P0, "destroy", "c", false, Some(pact)), vec![birds]);
}

#[test]
fn slaughter_pact_is_paid_at_the_next_upkeep() {
    let mut g = table(&["sauron", "veyran"]);
    lands(&mut g, P0, "Swamp", 3, false);
    on_cast(&mut g, P0, "Slaughter Pact");
    turn::upkeep(&mut g, P0).unwrap();
    assert!(g.player(P0).alive);
    assert_eq!(tapped_lands(&g, P0), 3);
    assert!(g.player(P0).pact_debts.is_empty());
}

#[test]
fn slaughter_pact_unpaid_loses_the_game() {
    let mut g = table(&["sauron", "veyran", "seph"]);
    lands(&mut g, P0, "Island", 3, false); // no black mana
    on_cast(&mut g, P0, "Slaughter Pact");
    turn::upkeep(&mut g, P0).unwrap();
    assert!(!g.player(P0).alive);
}

#[test]
fn the_ai_casts_slaughter_pact_only_when_it_can_pay() {
    let mut g = table(&["sauron", "veyran"]);
    hand(&mut g, P0, &["Slaughter Pact"]);
    perm(&mut g, P1, "Guttersnipe");
    assert!(brain::removal_options(&g, P0, &brain::Situation::new(&g, P0)).is_empty());
    lands(&mut g, P0, "Swamp", 3, true); // tapped now, untapped at the upkeep
    assert_eq!(brain::removal_options(&g, P0, &brain::Situation::new(&g, P0)).len(), 1);
}

#[test]
fn urabrask_on_your_upkeep() {
    // "At the beginning of your upkeep, exile the top card of your library. You may play it this turn."
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Urabrask, Heretic Praetor");
    let top = *g.player(P0).library.last().unwrap();
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand, vec![top]);
    assert_eq!(g.player(P0).impulse, vec![top]);
}

#[test]
fn urabrask_replaces_an_opponents_draw() {
    // "At the beginning of each opponent's upkeep, the next time they would draw a card this turn, instead they
    //  exile the top card of their library. They may play it this turn."  Exiling isn't drawing: Sheoldred doesn't
    //  trigger, and the card is gone at the end of the turn if it isn't played.
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Urabrask, Heretic Praetor");
    perm(&mut g, P0, "Sheoldred, the Apocalypse");
    g.active = Some(P1);
    turn::upkeep(&mut g, P1).unwrap();
    let top = *g.player(P1).library.last().unwrap();
    zones::draw(&mut g, P1, 1, true).unwrap();
    assert_eq!(g.player(P1).impulse, vec![top]);
    assert_eq!(g.player(P1).life, 40);
    zones::draw(&mut g, P1, 1, false).unwrap(); // only the first draw is replaced
    assert_eq!(g.player(P1).life, 38);
    turn::end_step(&mut g, P1).unwrap();
    assert!(!g.player(P1).hand.contains(&top));
    assert!(g.player(P1).exile.contains(&top));
}

#[test]
fn kefka_enters() {
    // "each player discards a card. Then you draw a card for each card type among cards discarded this way"
    let mut g = table(&["sauron", "veyran", "seph"]);
    hand(&mut g, P0, &["Sol Ring"]);
    hand(&mut g, P1, &["Island"]);
    hand(&mut g, P2, &["Grave Titan"]);
    let kefka = perm(&mut g, P0, "Kefka, Court Mage // Kefka, Ruler of Ruin");
    assert!(!g.perm(kefka).fly); // the front face doesn't fly
    assert_eq!((g.player(P1).hand.len(), g.player(P2).hand.len()), (0, 0));
    assert_eq!(g.player(P0).hand.len(), 3); // artifact, land, creature
}

#[test]
fn kefka_transforms() {
    let mut g = table(&["sauron", "veyran"]);
    let kefka = perm(&mut g, P0, "Kefka, Court Mage // Kefka, Ruler of Ruin");
    lands(&mut g, P0, "Island", 8, false);
    lands(&mut g, P1, "Island", 5, false);
    token(&mut g, P1, 1);
    assert!(take_option(&mut g, P0, "Kefka: {8}, transform"));
    assert!(g.player(P1).perms.is_empty()); // the opponent sacrificed the token
    let k = g.perm(kefka);
    assert_eq!((values::epow(&g, kefka), values::etgh(&g, kefka), k.fly), (5, 7, true));
    let n = g.player(P0).hand.len();
    sim_core::engine::life::lose_life(&mut g, P1, 3, Some(P0), "other", None).unwrap(); // on your turn: draw that many
    assert_eq!(g.player(P0).hand.len(), n + 3);
    g.active = Some(P1);
    sim_core::engine::life::lose_life(&mut g, P1, 2, Some(P0), "other", None).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 3);
}

#[test]
fn brush_off_is_cheaper_against_instants_and_sorceries() {
    let mut g = table(&["sauron", "veyran"]);
    let brush = hand(&mut g, P0, &["Brush Off"])[0];
    lands(&mut g, P0, "Island", 2, false);
    let bolt = card(&g, "Lightning Bolt");
    let titan = card(&g, "Grave Titan");
    assert_eq!(stack::pick_counter(&g, P0, bolt), Some(brush)); // {1}{U}
    assert_eq!(stack::pick_counter(&g, P0, titan), None); // {2}{U}{U}
    assert!(stack::cast_counter(&mut g, P0, brush, Some(bolt)).unwrap());
    assert_eq!(tapped_lands(&g, P0), 2);
}

#[test]
fn sauron_values_consecrated_sphinx() {
    // its priority follows what it will draw: two per opponent draw, so three opponents make it the better card,
    // and it's worth less with one opponent left
    let mut g = table(&["sauron", "veyran", "veyran", "veyran"]);
    lands(&mut g, P0, "Island", 5, false);
    lands(&mut g, P0, "Swamp", 2, false);
    hand(&mut g, P0, &["Consecrated Sphinx", "Night's Whisper"]);
    let u = options(&mut g, P0);
    assert!(utility(&u, "Consecrated Sphinx") > utility(&u, "Night's Whisper"));
    let mut g1 = table(&["sauron", "veyran"]);
    lands(&mut g1, P0, "Island", 5, false);
    lands(&mut g1, P0, "Swamp", 2, false);
    hand(&mut g1, P0, &["Consecrated Sphinx", "Night's Whisper"]);
    let u1 = options(&mut g1, P0);
    assert!(utility(&u1, "Consecrated Sphinx") < utility(&u, "Consecrated Sphinx"));
}

#[test]
fn diabolic_intent_needs_a_creature_to_sacrifice() {
    // "As an additional cost to cast this spell, sacrifice a creature." It used to be free outside Sephiroth.
    let mut g = table(&["sauron", "veyran"]);
    lands(&mut g, P0, "Swamp", 2, false);
    let c = hand(&mut g, P0, &["Diabolic Intent"])[0];
    assert!(!brain::do_cast(&mut g, P0, c, None).unwrap());
    perm(&mut g, P0, "Orcish Bowmasters"); // its Army comes too
    let a = army(&g, P0);
    g.perm_mut(a).plus = 5;
    assert!(brain::do_cast(&mut g, P0, c, None).unwrap());
    let names: Vec<&str> = g.player(P0).perms.iter().map(|&m| g.perm(m).name).collect();
    assert_eq!(names, ["Orc Army"]); // the cheapest creature went, not the Army
    assert_eq!(g.player(P0).hand.len(), 1); // the tutored card
}

#[test]
fn diabolic_intent_never_sacrifices_the_army() {
    // the AI used to sacrifice the Army (often with ten or more counters) when it was the only creature,
    // frequently to fetch the Sword that goes on it
    let mut g = table(&["sauron", "veyran"]);
    lands(&mut g, P0, "Swamp", 2, false);
    let c = hand(&mut g, P0, &["Diabolic Intent"])[0];
    zones::amass(&mut g, P0, 6).unwrap();
    assert!(!brain::do_cast(&mut g, P0, c, None).unwrap());
    assert!(values::army_of(&g, P0).is_some());
}

#[test]
fn phyrexian_arena() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Phyrexian Arena");
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!((g.player(P0).hand.len(), g.player(P0).life), (1, 39));
}

// ======================================================== SauronBreach
// Underworld Breach + Brain Freeze / Grapeshot (storm: a copy per spell cast before it this turn)

/// Sauron against three Veyrans whose libraries hold `opp_library` cards (the rest milled), no hands; six Islands in
/// Sauron's graveyard for escape fuel
fn board(islands: usize, mountains: usize, opp_library: usize) -> Game {
    let mut g = table(&["sauron", "veyran", "veyran", "veyran"]);
    for q in 1..4 {
        let pl = g.player_mut(PlayerId(q));
        pl.hand.clear();
        let rest = pl.library.split_off(opp_library.min(pl.library.len()));
        pl.gy.extend(rest);
    }
    let island = card(&g, "Island");
    let s = g.player_mut(P0);
    s.hand.clear();
    s.gy = vec![island; 6]; // fuel for escape
    lands(&mut g, P0, "Island", islands, false);
    lands(&mut g, P0, "Mountain", mountains, false);
    g
}

fn opp_libraries(g: &Game) -> Vec<usize> {
    g.players[1..].iter().map(|q| q.library.len()).collect()
}

fn stat(g: &Game, p: PlayerId, name: &str) -> i64 {
    g.player(p).stats.get(name).copied().unwrap_or(0)
}

#[test]
fn brain_freeze_copies() {
    let mut g = board(2, 0, 20);
    let bf = hand(&mut g, P0, &["Brain Freeze"])[0];
    let ctx = Ctx { storm: Some(2), mill: vec![P1, P1, P1], ..Ctx::default() };
    cast::cast_card(&mut g, P0, bf, "hand", ctx).unwrap();
    assert_eq!(g.player(P1).library.len(), 20 - 9);
}

#[test]
fn grapeshot_kills_the_lowest_life_first() {
    let mut g = board(0, 2, 20);
    g.player_mut(P1).life = 3;
    let gs = hand(&mut g, P0, &["Grapeshot"])[0];
    cast::cast_card(&mut g, P0, gs, "hand", Ctx { storm: Some(4), ..Ctx::default() }).unwrap();
    assert!(!g.player(P1).alive);
    assert_eq!(g.players[2..].iter().map(|q| 40 - q.life).sum::<i32>(), 2);
}

#[test]
fn the_line_waits_for_enough_mana() {
    let mut g = board(6, 6, 20);
    hand(&mut g, P0, &["Underworld Breach", "Brain Freeze"]);
    assert!(!mine::breach_line(&mut g, P0, false).unwrap()); // 12 mana: two opponents milled, not three
    let mut g = board(8, 8, 20);
    hand(&mut g, P0, &["Underworld Breach", "Brain Freeze"]);
    assert!(mine::breach_line(&mut g, P0, false).unwrap());
    let mut g = board(4, 12, 20);
    hand(&mut g, P0, &["Underworld Breach", "Brain Freeze"]);
    assert!(!mine::breach_line(&mut g, P0, false).unwrap()); // 16 mana, but only four blue for Brain Freeze
}

#[test]
fn the_line_mills_the_table() {
    let mut g = board(8, 8, 20);
    hand(&mut g, P0, &["Underworld Breach", "Brain Freeze"]);
    assert!(mine::breach_line(&mut g, P0, true).unwrap());
    assert_eq!(opp_libraries(&g), [0, 0, 0]);
    assert!(stat(&g, P0, "breach_escapes") >= 5);
}

#[test]
fn lotus_petal_makes_it_infinite() {
    // each Petal escape ({0} plus three graveyard cards) makes one mana of any colour: the {U} for the next Brain
    // Freeze, whose copies mill you for more fuel; from two blue sources it mills out the table
    let mut g = board(2, 1, 60);
    lands(&mut g, P0, "Swamp", 1, false);
    hand(&mut g, P0, &["Underworld Breach", "Brain Freeze", "Lotus Petal"]);
    assert!(mine::breach_line(&mut g, P0, false).unwrap());
    mine::breach_line(&mut g, P0, true).unwrap();
    assert_eq!(opp_libraries(&g), [0, 0, 0]);
}

#[test]
fn rituals_alone_run_out_of_blue() {
    // Dark Ritual and Cabal Ritual make only black mana: storm grows, but each Brain Freeze still needs a real {U}
    let mut g = board(3, 1, 60);
    lands(&mut g, P0, "Swamp", 1, false);
    hand(&mut g, P0, &["Underworld Breach", "Brain Freeze", "Dark Ritual", "Cabal Ritual"]);
    assert!(!mine::breach_line(&mut g, P0, false).unwrap());
}

#[test]
fn lions_eye_diamond_makes_it_infinite() {
    // each Diamond escape ({0} plus three graveyard cards) discards the hand and makes three blue: a whole Brain
    // Freeze; from one Island and two Mountains it mills out the table, which the same lands alone can't
    let mut g = board(1, 2, 60);
    hand(&mut g, P0, &["Underworld Breach", "Brain Freeze"]);
    assert!(!mine::breach_line(&mut g, P0, false).unwrap());
    let mut g = board(1, 2, 60);
    hand(&mut g, P0, &["Underworld Breach", "Brain Freeze", "Lion's Eye Diamond"]);
    assert!(mine::breach_line(&mut g, P0, false).unwrap());
    mine::breach_line(&mut g, P0, true).unwrap();
    assert_eq!(opp_libraries(&g), [0, 0, 0]);
    assert!(stat(&g, P0, "led_used") >= 2);
}

/// board(2, 2, 60) plus two Swamps, Sauron's library cut to 30
fn labman_board() -> Game {
    let mut g = board(2, 2, 60);
    lands(&mut g, P0, "Swamp", 2, false);
    g.player_mut(P0).library.truncate(30);
    g
}

#[test]
fn laboratory_maniac_mills_yourself_out() {
    // with Laboratory Maniac out, every Brain Freeze copy mills you; Lotus Petal escapes pay each next Brain Freeze
    // (the copies refill the graveyard), and an escaped Night's Whisper draws from the empty library to win.
    // Milling three 60-card libraries from the same mana fails
    let mut g = labman_board();
    hand(&mut g, P0, &["Underworld Breach", "Brain Freeze", "Lotus Petal", "Night's Whisper"]);
    assert!(!mine::breach_line(&mut g, P0, false).unwrap());
    let mut g = labman_board();
    hand(&mut g, P0, &["Underworld Breach", "Brain Freeze", "Lotus Petal", "Night's Whisper"]);
    perm(&mut g, P0, "Laboratory Maniac");
    assert!(mine::breach_line(&mut g, P0, false).unwrap());
    mine::breach_line(&mut g, P0, true).unwrap();
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn the_line_casts_laboratory_maniac_from_hand() {
    let mut g = board(4, 2, 60);
    lands(&mut g, P0, "Swamp", 2, false);
    g.player_mut(P0).library.truncate(30);
    hand(&mut g, P0, &["Laboratory Maniac", "Underworld Breach", "Brain Freeze", "Lotus Petal", "Night's Whisper"]);
    assert!(mine::breach_line(&mut g, P0, false).unwrap());
    mine::breach_line(&mut g, P0, true).unwrap();
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn birgi_pays_for_each_brain_freeze() {
    // Birgi adds {R} for each spell cast: with her out, a Petal escape makes the {U} and Birgi the {1}. Holding her
    // in hand doesn't stop a line that works without casting her first (the dry run tries both)
    const BIRGI: &str = "Birgi, God of Storytelling // Harnfel, Horn of Bounty";
    let mut g = board(1, 2, 60);
    perm(&mut g, P0, BIRGI);
    hand(&mut g, P0, &["Underworld Breach", "Brain Freeze", "Lotus Petal"]);
    mine::breach_line(&mut g, P0, true).unwrap();
    assert_eq!(opp_libraries(&g), [0, 0, 0]);
    assert!(g.player(P0).floating.r >= 5); // Birgi's spare red
    let mut g = board(1, 2, 60);
    hand(&mut g, P0, &[BIRGI, "Underworld Breach", "Brain Freeze", "Lotus Petal"]);
    assert!(mine::breach_line(&mut g, P0, false).unwrap());
}

#[test]
fn jeskas_will_never_spends_the_blue_brain_freeze_needs() {
    // Jeska's Will makes red for each card in an opponent's hand, but paying for it can eat the Petal's {U}: the dry
    // run falls back to the plain line, so holding it never stops a line that works without it
    let mut g = board(0, 4, 60);
    g.player_mut(P0).gy.clear();
    let island = card(&g, "Island");
    for q in 1..4 {
        g.player_mut(PlayerId(q)).hand = vec![island; 7];
    }
    hand(&mut g, P0, &["Jeska's Will", "Underworld Breach", "Brain Freeze", "Lotus Petal"]);
    assert!(mine::breach_line(&mut g, P0, false).unwrap());
    mine::breach_line(&mut g, P0, true).unwrap();
    assert_eq!(opp_libraries(&g), [0, 0, 0]);
}

#[test]
fn laboratory_maniac_turns_an_empty_draw_into_a_win() {
    let mut g = table(&["sauron", "veyran"]);
    g.player_mut(P0).library.clear();
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert!(g.player(P0).decked);
    let mut g = table(&["sauron", "veyran"]);
    g.player_mut(P0).library.clear();
    perm(&mut g, P0, "Laboratory Maniac");
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn the_pieces_are_held_for_the_line() {
    // (Python also checks Lion's Eye Diamond: see the test below)
    let mut g = board(8, 8, 20);
    let cs = hand(&mut g, P0, &["Brain Freeze", "Underworld Breach"]);
    assert_eq!(decks::sauron_prio(&g, P0, cs[0]), 0);
    assert_eq!(decks::sauron_prio(&g, P0, cs[1]), 0); // gc_prio_sauron
}

#[test]
fn lions_eye_diamond_is_held_for_the_line() {
    let mut g = board(8, 8, 20);
    let led = hand(&mut g, P0, &["Brain Freeze", "Underworld Breach", "Lion's Eye Diamond"])[2];
    assert_eq!(decks::sauron_prio(&g, P0, led), 0);
}

#[test]
fn tutors_find_the_missing_piece() {
    let mut g = board(4, 4, 20);
    hand(&mut g, P0, &["Underworld Breach"]);
    let bf = card(&g, "Brain Freeze");
    g.player_mut(P0).library.push(bf);
    assert_eq!(decks::tutor_pick(&g, P0, "any"), Some(bf));
}

// ======================================================== SauronMarchesa
// Marchesa, the Black Rose with Sauron's Army (she was in Sauron's deck from 10-03): dethrone for the Army, with
// Mauhúr's extra counter, and the return of a creature with a counter

#[test]
#[ignore = "needs marchesa.py's card code (Marchesa is in Jodah's list now: phase 6)"]
fn the_army_gets_dethrone_and_mauhurs_extra_counter() {
    let mut g = table(&["sauron", "veyran", "seph"]);
    perm(&mut g, P0, "Marchesa, the Black Rose");
    perm(&mut g, P0, "Mauhúr, Uruk-hai Captain");
    zones::amass(&mut g, P0, 2).unwrap();
    let a = army(&g, P0);
    g.perm_mut(a).sick = false;
    let before = g.perm(a).plus;
    g.player_mut(P1).life = 45; // the life leader
    sim_core::engine::combat::attack_triggers(&mut g, P0, &[a], P1).unwrap();
    assert_eq!(g.perm(a).plus, before + 2); // dethrone 1, Mauhúr 1
}

#[test]
#[ignore = "needs marchesa.py's card code (Marchesa is in Jodah's list now: phase 6)"]
fn a_creature_with_a_counter_returns() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Marchesa, the Black Rose");
    let b = perm(&mut g, P0, "Orcish Bowmasters");
    g.perm_mut(b).plus = 1;
    zones::die(&mut g, b, "destroy").unwrap();
    turn::end_step(&mut g, P0).unwrap();
    let cd = g.perm(b).cd;
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).cd == cd));
}

// ======================================================== the Ring and the rest of mine.py's Sauron cards (Rust-only)
#[test]
fn the_ring_tempts_and_makes_the_army_the_ring_bearer() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Orcish Bowmasters");
    let a = army(&g, P0);
    mine::ring_tempt(&mut g, P0).unwrap();
    assert_eq!(mine::ring_level(&g, P0), 1);
    assert_eq!(mine::ring_bearer(&g, P0), Some(a)); // the Army first
    assert!(mine::is_legendary(&g, a)); // the Ring: your Ring-bearer is legendary
    for _ in 0..5 {
        mine::ring_tempt(&mut g, P0).unwrap();
    }
    assert_eq!(mine::ring_level(&g, P0), 4);
}

#[test]
fn ringsight_tempts_sauron_into_a_new_hand() {
    // Sauron, the Dark Lord: whenever the Ring tempts you, you may discard your hand and draw four (the AI does with
    // three or fewer cards)
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Sauron, the Dark Lord");
    hand(&mut g, P0, &["Island", "Swamp"]);
    mine::ring_tempt(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), 4);
    assert_eq!(g.player(P0).gy.len(), 2);
}

#[test]
fn vraska_turns_a_creature_into_a_treasure() {
    let mut g = table(&["sauron", "veyran"]);
    let v = perm(&mut g, P0, "Vraska, Betrayal's Sting");
    assert_eq!(g.perm(v).loyalty, Some(6));
    perm(&mut g, P1, "Guttersnipe");
    assert!(take_option(&mut g, P0, "Vraska -2 -> Guttersnipe"));
    assert!(g.player(P1).perms.is_empty());
    assert_eq!(g.player(P1).treasures, 1);
    assert_eq!(g.perm(v).loyalty, Some(4));
    assert!(!options(&mut g, P0).iter().any(|o| o.label.starts_with("Vraska"))); // once per turn
}

#[test]
fn bloodsoaked_insight_takes_an_opponents_top_three() {
    let mut g = table(&["sauron", "veyran"]);
    lands(&mut g, P0, "Swamp", 4, false);
    let c = hand(&mut g, P0, &["Bloodsoaked Insight // Sanguine Morass"])[0];
    sim_core::engine::life::lose_life(&mut g, P1, 3, Some(P0), "other", None).unwrap(); // {5} - 3
    let top: Vec<_> = g.player(P1).library.iter().rev().take(3).copied().collect();
    assert!(take_option(&mut g, P0, "Bloodsoaked Insight"));
    assert!(g.player(P0).gy.contains(&c));
    for x in top {
        assert!(g.player(P0).hand.contains(&x));
    }
    assert_eq!(tapped_lands(&g, P0), 4); // {2} + {1}{B}
}

// ======================================================== Sauron never decks himself after the Breach line
// (a Python bug, fixed in Rust only: spare Brain Freeze copies milled Sauron, then his own draws lost him the game
// before the milled-out table drew)

#[test]
fn spare_brain_freeze_copies_never_mill_sauron() {
    // three cards an opponent: the second Brain Freeze has four copies and the table needs one; three are spare
    let mut g = board(8, 8, 3);
    hand(&mut g, P0, &["Underworld Breach", "Brain Freeze"]);
    let before = g.player(P0).library.len();
    assert!(mine::breach_line(&mut g, P0, true).unwrap());
    assert_eq!(opp_libraries(&g), [0, 0, 0]);
    assert_eq!(g.player(P0).library.len(), before);
}

#[test]
fn sauron_wont_draw_four_from_a_short_library() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Sauron, the Dark Lord");
    hand(&mut g, P0, &["Island", "Swamp"]);
    g.player_mut(P0).library.truncate(3);
    mine::ring_tempt(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), 2);
    assert_eq!(g.player(P0).library.len(), 3);
    assert!(g.player(P0).alive);
}

#[test]
fn call_of_the_ring_wont_draw_from_an_empty_library() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Call of the Ring");
    token(&mut g, P0, 2);
    hand(&mut g, P0, &["Island", "Swamp", "Forest", "Plains"]); // four cards: Sauron's draw-four is off anyway
    g.player_mut(P0).library.clear();
    mine::ring_tempt(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).life, 40);
    assert!(g.player(P0).alive);
}

#[test]
fn the_ring_bearer_stays_home_when_its_loot_would_deck_sauron() {
    let mut g = table(&["sauron", "veyran"]);
    let b = token(&mut g, P0, 3);
    g.player_mut(P0).ring_level = 2;
    g.player_mut(P0).ring_bearer = Some(b);
    g.player_mut(P0).library.clear();
    assert!(mine::ring_loot_decks(&g, P0, b));
    sim_core::engine::combat::combat(&mut g, P0).unwrap();
    assert!(g.player(P0).alive);
    assert!(!g.perm(b).tapped);
    // with a card left the loot is safe, and it attacks
    let mut g = table(&["sauron", "veyran"]);
    let b = token(&mut g, P0, 3);
    g.player_mut(P0).ring_level = 2;
    g.player_mut(P0).ring_bearer = Some(b);
    g.player_mut(P0).library.truncate(1);
    assert!(!mine::ring_loot_decks(&g, P0, b));
}

// ======================================================== a finished Breach line isn't offered again
// (a Python bug, fixed in Rust only: once every opponent's library was empty, the dry run still counted the line as
// finishing the table, and the AI kept choosing the 15.0 "Underworld Breach line" option, which did nothing, until the
// main phase's action cap)

#[test]
fn a_finished_breach_line_is_not_offered() {
    let mut g = board(8, 8, 20);
    hand(&mut g, P0, &["Underworld Breach", "Brain Freeze"]);
    let line = |g: &mut Game| mine::breach_options(g, P0, true).unwrap().len();
    assert_eq!(line(&mut g), 1); // the table can be milled out
    assert!(mine::breach_line(&mut g, P0, true).unwrap());
    assert_eq!(opp_libraries(&g), [0, 0, 0]);
    assert!(g.players[1..].iter().all(|q| q.alive)); // they lose on their own draws
    assert!(!mine::breach_line(&mut g, P0, false).unwrap()); // nothing left to do
    assert_eq!(line(&mut g), 0);
    let opts = brain::main_options(&mut g, P0, true).unwrap();
    assert!(!opts.iter().any(|o| o.label == "Underworld Breach line"));
    // the same with Breach still in hand and the libraries already empty
    let mut g = board(8, 8, 0);
    hand(&mut g, P0, &["Underworld Breach", "Brain Freeze"]);
    assert_eq!(line(&mut g), 0);
}
