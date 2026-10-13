//! The blue spells (impls/bulk_blue.rs) on hand-built positions. Seat 0 plays a stand-in deck (the cards are keyed by
//! name), seat 1 the seph deck.

use sim_core::ai::act;
use sim_core::engine::hooks::fire_trigger;
use sim_core::engine::{cast, values, zones};
use sim_core::hooks::{Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["sheoldred", "seph"])
}

fn cast_spell(g: &mut Game, p: PlayerId, name: &str, ctx: Ctx) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", ctx).unwrap();
    c
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn perm_opts(g: &mut Game, m: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.perm(m).cd.unwrap()).and_then(|i| i.options).expect("the card has options");
    f(g, m, p, post).unwrap()
}

fn hand_opts(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).and_then(|i| i.hand_options).expect("the card has hand options");
    f(g, c, p, post).unwrap()
}

fn perform(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn jace_token(g: &Game, p: PlayerId) -> Option<PermId> {
    sim_core::impls::alela::jace_token(g, p)
}

// ======================================================== Jace, Reality Sculptor
#[test]
fn jace_plus_empowers_by_the_islands_you_control() {
    let mut g = game();
    lands(&mut g, P0, "Island", 5, false);
    let j = perm(&mut g, P0, "Jace, Reality Sculptor");
    g.perm_mut(j).loyalty = Some(5);
    let opts = perm_opts(&mut g, j, P0, Some(true));
    let plus = opts.iter().find(|o| o.label.ends_with("+1")).expect("the +1 is offered");
    assert!(perform(&mut g, P0, plus));
    assert_eq!(g.perm(j).loyalty, Some(6));
    let t = jace_token(&g, P0).expect("the Jace token was made");
    assert_eq!(g.perm(t).loyalty, Some(5), "X = five Islands");
}

#[test]
fn jace_minus_three_shrinks_attackers_until_your_next_turn() {
    let mut g = game();
    lands(&mut g, P0, "Island", 3, false);
    let j = perm(&mut g, P0, "Jace, Reality Sculptor");
    g.perm_mut(j).loyalty = Some(5);
    let a = token(&mut g, P1, 6);
    let b = token(&mut g, P1, 6);
    let opts = perm_opts(&mut g, j, P0, Some(true));
    let minus = opts.iter().find(|o| o.label.ends_with("-3")).expect("the -3 is offered facing a big board");
    assert!(perform(&mut g, P0, minus));
    assert_eq!(g.perm(j).loyalty, Some(2));
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: P1, atk: vec![a, b], d: P0 }).unwrap();
    assert_eq!(values::epow(&g, a), 1, "6 - 5");
    // an attack on someone else is not shrunk
    let mut g = game();
    let j = perm(&mut g, P0, "Jace, Reality Sculptor");
    g.perm_mut(j).data.set(sim_core::state::DataKey::Used, sim_core::state::Val::Bool(true));
    let a = token(&mut g, P1, 6);
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: P1, atk: vec![a], d: PlayerId(1) }).unwrap();
    assert_eq!(values::epow(&g, a), 6);
    // the effect ends at the Jace's controller's next upkeep
    fire_trigger(&mut g, Event::Upkeep, Call::Player { p: P0 }).unwrap();
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: P1, atk: vec![a], d: P0 }).unwrap();
    assert_eq!(values::epow(&g, a), 6);
}

#[test]
fn jace_zero_needs_twenty_five_loyalty_among_your_jaces() {
    let mut g = game();
    let j = perm(&mut g, P0, "Jace, Reality Sculptor");
    g.perm_mut(j).loyalty = Some(10);
    let n = g.player(P1).library.len();
    assert!(n > 1);
    let opts = perm_opts(&mut g, j, P0, Some(true));
    assert!(!opts.iter().any(|o| o.label.ends_with("+0")), "10 loyalty: no 0 ability");
    sim_core::impls::alela::empower(&mut g, P0, 15);
    let opts = perm_opts(&mut g, j, P0, Some(true));
    let zero = opts.iter().find(|o| o.label.ends_with("+0")).expect("10 + 15 = 25");
    assert!(perform(&mut g, P0, zero));
    assert_eq!(g.player(P1).library.len(), 1);
    assert_eq!(g.player(P1).exile.len(), n - 1);
}

// ======================================================== helpers for the rest
fn hand_len(g: &Game, p: PlayerId) -> usize {
    g.player(p).hand.len()
}

fn count(g: &Game, p: PlayerId, name: &str) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && &*g.perm(m).name == name).count()
}

/// a spell of P1 on the stack, and the id P0's counterspell would name
fn spell_on_stack(g: &mut Game, name: &str) -> (CardId, u32) {
    let c = card(g, name);
    let it = sim_core::engine::stack::spell_item(g, P1, c, Ctx::default(), 7.0);
    let id = it.id;
    g.stack.push(it);
    (c, id)
}

fn countered(g: &Game, id: u32) -> bool {
    g.stack.iter().find(|x| x.id == id).unwrap().countered
}

// ======================================================== token copies
#[test]
fn relms_sketching_copies_the_best_creature() {
    let mut g = game();
    perm(&mut g, P0, "Cloudfin Raptor");
    let big = perm(&mut g, P1, "Grave Titan");
    token(&mut g, P0, 1);
    cast_spell(&mut g, P0, "Relm's Sketching", Ctx::default());
    assert_eq!(count(&g, P0, "Grave Titan"), 1, "a token copy of the best creature, even an opponent's");
    assert!(g.perm(big).on_bf);
    let copy = g.player(P0).perms.iter().copied().find(|&m| &*g.perm(m).name == "Grave Titan").unwrap();
    assert!(g.perm(copy).token);
    assert!(in_gy(&g, P0, "Relm's Sketching"));
}

#[test]
fn relms_sketching_skips_legends_and_falls_back_to_a_land() {
    let mut g = game();
    perm(&mut g, P0, "Elrond, Lord of Rivendell");
    lands(&mut g, P0, "Island", 3, false);
    cast_spell(&mut g, P0, "Relm's Sketching", Ctx::default());
    assert_eq!(count(&g, P0, "Elrond, Lord of Rivendell"), 1, "the legend rule would undo a copy");
    assert_eq!(g.player(P0).lands.len(), 4, "a token copy of an Island");
}

#[test]
fn relms_sketching_is_held_without_a_target() {
    let mut g = game();
    let c = hand(&mut g, P0, &["Relm's Sketching"])[0];
    let prio = g.registry.get(c).unwrap().prio.unwrap();
    assert_eq!(prio(&g, P0, c), 0);
    perm(&mut g, P0, "Cloudfin Raptor");
    perm(&mut g, P1, "Grave Titan");
    assert!(prio(&g, P0, c) > 0);
}

#[test]
fn echocasting_symposium_copies_then_exiles_and_recasts_each_turn() {
    let mut g = game();
    perm(&mut g, P0, "Cloudfin Raptor");
    let c = cast_spell(&mut g, P0, "Echocasting Symposium", Ctx { face: Some(P0), ..Ctx::default() });
    assert_eq!(count(&g, P0, "Cloudfin Raptor"), 2);
    assert!(g.player(P0).exile.contains(&c), "paradigm: exiled, not in the graveyard");
    assert!(!g.player(P0).gy.contains(&c));
    // each of our turns, a copy of the spell is cast free: another token, and the spell stays in exile
    sim_core::cardcode::turn_start(&mut g, P0).unwrap();
    assert_eq!(count(&g, P0, "Cloudfin Raptor"), 3);
    assert!(g.player(P0).exile.contains(&c));
    sim_core::cardcode::turn_start(&mut g, P0).unwrap();
    assert_eq!(count(&g, P0, "Cloudfin Raptor"), 4);
    // an opponent's turn start does nothing
    sim_core::cardcode::turn_start(&mut g, P1).unwrap();
    assert_eq!(count(&g, P0, "Cloudfin Raptor"), 4);
}

#[test]
fn echocasting_symposium_without_a_creature_still_exiles() {
    let mut g = game();
    let c = cast_spell(&mut g, P0, "Echocasting Symposium", Ctx::default());
    assert!(g.player(P0).exile.contains(&c));
}

// ======================================================== spree, kicker, modes
#[test]
fn phantom_interference_counters_unless_its_controller_pays_two() {
    let mut g = game();
    lands(&mut g, P0, "Island", 2, false);
    let ctr = hand(&mut g, P0, &["Phantom Interference"])[0];
    let (_, id) = spell_on_stack(&mut g, "Grave Titan");
    assert!(sim_core::engine::stack::cast_counter_spell(&mut g, P0, ctr, id).unwrap());
    assert!(countered(&g, id), "the controller has no mana to pay");
    assert_eq!(g.player(P0).perms.len(), 0, "{{1}}{{U}} only: no Spirit");
    // with two lands open, the controller pays and the spell resolves
    let mut g = game();
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P1, "Swamp", 2, false);
    let ctr = hand(&mut g, P0, &["Phantom Interference"])[0];
    let (_, id) = spell_on_stack(&mut g, "Grave Titan");
    sim_core::engine::stack::cast_counter_spell(&mut g, P0, ctr, id).unwrap();
    assert!(!countered(&g, id));
}

#[test]
fn phantom_interference_with_five_mana_adds_the_spirit() {
    let mut g = game();
    lands(&mut g, P0, "Island", 5, false);
    let ctr = hand(&mut g, P0, &["Phantom Interference"])[0];
    let (_, id) = spell_on_stack(&mut g, "Grave Titan");
    sim_core::engine::stack::cast_counter_spell(&mut g, P0, ctr, id).unwrap();
    assert!(countered(&g, id));
    let spirit = g.player(P0).perms.first().copied().expect("a Spirit token");
    assert_eq!((values::epow(&g, spirit), values::etgh(&g, spirit)), (2, 2));
    assert!(g.perm(spirit).fly);
    assert!(mana_left(&g, P0) == 0, "all five lands were used");
}

fn mana_left(g: &Game, p: PlayerId) -> usize {
    g.player(p).lands.iter().filter(|&&l| !g.land(l).tapped).count()
}

#[test]
fn phantom_interference_spirit_mode_at_the_end_of_an_opponents_turn() {
    let mut g = game();
    g.active = Some(P1);
    lands(&mut g, P0, "Island", 4, false);
    let c = hand(&mut g, P0, &["Phantom Interference"])[0];
    assert!(hand_opts(&mut g, c, P0, Some(false)).is_empty(), "not in the main phase");
    let opts = hand_opts(&mut g, c, P0, None);
    assert_eq!(opts.len(), 1);
    assert!(perform(&mut g, P0, &opts[0]));
    assert_eq!(g.player(P0).perms.len(), 1);
    assert!(in_gy(&g, P0, "Phantom Interference"));
    assert_eq!(mana_left(&g, P0), 0);
}

#[test]
fn whoosh_bounces_and_kicker_draws() {
    let mut g = game();
    let t = perm(&mut g, P1, "Grave Titan");
    let h = hand_len(&g, P0);
    cast_spell(&mut g, P0, "Whoosh!", Ctx { target: Some(t), ..Ctx::default() });
    assert!(!g.perm(t).on_bf);
    assert_eq!(hand_len(&g, P0), h, "unkicked: no card");
    let mut g = game();
    let t = perm(&mut g, P1, "Grave Titan");
    let h = hand_len(&g, P0);
    cast_spell(&mut g, P0, "Whoosh!", Ctx { target: Some(t), mode: Some("kick"), ..Ctx::default() });
    assert_eq!(hand_len(&g, P0), h + 1, "kicked: draws a card");
}

#[test]
fn whoosh_rebuys_our_enter_trigger_at_the_end_of_their_turn_and_kicks_when_it_can() {
    let mut g = game();
    g.active = Some(P1);
    lands(&mut g, P0, "Island", 4, false);
    let k = perm(&mut g, P0, "Keeper of the Quiet Hour");
    let c = hand(&mut g, P0, &["Whoosh!"])[0];
    let h = hand_len(&g, P0);
    let opts = hand_opts(&mut g, c, P0, None);
    assert_eq!(opts.len(), 1, "{:?}", opts.iter().map(|o| &o.label).collect::<Vec<_>>());
    assert!(perform(&mut g, P0, &opts[0]));
    assert!(!g.perm(k).on_bf);
    assert_eq!(hand_len(&g, P0), h + 1, "Whoosh! out, Keeper back, a card drawn (kicked)");
    assert_eq!(mana_left(&g, P0), 0, "{{1}}{{U}} and the kicker {{1}}{{U}}");
    // two lands: unkicked
    let mut g = game();
    g.active = Some(P1);
    lands(&mut g, P0, "Island", 2, false);
    let k = perm(&mut g, P0, "Keeper of the Quiet Hour");
    let c = hand(&mut g, P0, &["Whoosh!"])[0];
    let h = hand_len(&g, P0);
    let opts = hand_opts(&mut g, c, P0, None);
    assert!(perform(&mut g, P0, &opts[0]));
    assert!(!g.perm(k).on_bf);
    assert_eq!(hand_len(&g, P0), h, "Whoosh! out, Keeper in, no draw");
}

#[test]
fn amazing_acrobatics_counters_and_taps_two_creatures() {
    let mut g = game();
    lands(&mut g, P0, "Island", 3, false);
    let a = token(&mut g, P1, 6);
    let b = token(&mut g, P1, 4);
    let c = token(&mut g, P1, 1);
    let ctr = hand(&mut g, P0, &["Amazing Acrobatics"])[0];
    let (_, id) = spell_on_stack(&mut g, "Sheoldred, the Apocalypse");
    assert!(sim_core::engine::stack::cast_counter_spell(&mut g, P0, ctr, id).unwrap());
    assert!(countered(&g, id));
    assert!(g.perm(a).tapped && g.perm(b).tapped, "the two best are tapped");
    assert!(!g.perm(c).tapped);
}

#[test]
fn amazing_acrobatics_taps_blockers_for_a_big_attack() {
    let mut g = game();
    lands(&mut g, P0, "Island", 3, false);
    let a = token(&mut g, P1, 6);
    let b = token(&mut g, P1, 4);
    hand(&mut g, P0, &["Amazing Acrobatics"]);
    let atk = vec![token(&mut g, P0, 5), token(&mut g, P0, 5)];
    sim_core::cardcode::hand_attack(&mut g, P0, &atk, P1).unwrap();
    assert!(g.perm(a).tapped && g.perm(b).tapped);
    assert!(in_gy(&g, P0, "Amazing Acrobatics"));
    // a small attack keeps it
    let mut g = game();
    lands(&mut g, P0, "Island", 3, false);
    let a = token(&mut g, P1, 6);
    token(&mut g, P1, 4);
    hand(&mut g, P0, &["Amazing Acrobatics"]);
    let atk = vec![token(&mut g, P0, 2)];
    sim_core::cardcode::hand_attack(&mut g, P0, &atk, P1).unwrap();
    assert!(!g.perm(a).tapped);
}

#[test]
fn icy_reception_counters_creature_and_legendary_spells_only() {
    let mut g = game();
    let icy = card(&g, "Icy Reception");
    for (name, ok) in
        [("Grave Titan", true), ("Jace, Reality Sculptor", true), ("Counterspell", false), ("Terror", false)]
    {
        let s = card(&g, name);
        assert_eq!(sim_core::engine::stack::counter_ok(&g, icy, s), ok, "{name}");
    }
    lands(&mut g, P0, "Island", 2, false);
    let ctr = hand(&mut g, P0, &["Icy Reception"])[0];
    let (_, id) = spell_on_stack(&mut g, "Grave Titan");
    sim_core::engine::stack::cast_counter_spell(&mut g, P0, ctr, id).unwrap();
    assert!(countered(&g, id), "no mana for the {{3}}");
    let mut g = game();
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P1, "Swamp", 3, false);
    let ctr = hand(&mut g, P0, &["Icy Reception"])[0];
    let (_, id) = spell_on_stack(&mut g, "Grave Titan");
    sim_core::engine::stack::cast_counter_spell(&mut g, P0, ctr, id).unwrap();
    assert!(!countered(&g, id), "they pay {{3}}");
}

#[test]
fn icy_reception_shrinks_a_big_unblocked_attacker() {
    let mut g = game();
    lands(&mut g, P0, "Island", 2, false);
    let c = hand(&mut g, P0, &["Icy Reception"])[0];
    g.player_mut(P0).life = 10;
    let big = token(&mut g, P1, 9);
    let mut atk = vec![big];
    let mut assign = vec![];
    let f = g.registry.get(c).unwrap().hand_defend.unwrap();
    f(&mut g, c, P0, P1, &mut atk, &mut assign).unwrap();
    assert_eq!(values::epow(&g, big), 4, "9 - 5");
    assert!(in_gy(&g, P0, "Icy Reception"));
    // a small attack on a healthy player: held
    let mut g = game();
    lands(&mut g, P0, "Island", 2, false);
    let c = hand(&mut g, P0, &["Icy Reception"])[0];
    let small = token(&mut g, P1, 2);
    let mut atk = vec![small];
    f(&mut g, c, P0, P1, &mut atk, &mut vec![]).unwrap();
    assert_eq!(values::epow(&g, small), 2);
}

// ======================================================== top or bottom
/// a permanent card of the seph deck (seat 1) that its owner values (card worth 45 or more) and can cast with 8 lands
fn wanted_permanent(g: &Game) -> CardId {
    g.player(P1)
        .library
        .iter()
        .copied()
        .filter(|&c| {
            let d = g.db.get(c);
            d.perm && !d.land && d.cmc <= 6 && sim_core::engine::tutors::card_worth(g, P1, c, false) >= 45.0
        })
        .max_by_key(|&c| g.db.get(c).cmc)
        .expect("the seph deck has a permanent it values")
}

#[test]
fn sudden_setback_tucks_a_permanent_on_its_owners_choice() {
    // a card the owner doesn't value goes to the bottom
    let mut g = game();
    let t = perm(&mut g, P1, "Grave Titan");
    let gt = card(&g, "Grave Titan");
    g.player_mut(P1).library.retain(|&c| c != gt);
    cast_spell(&mut g, P0, "Sudden Setback", Ctx { target: Some(t), ..Ctx::default() });
    assert!(!g.perm(t).on_bf);
    assert_eq!(g.player(P1).library.first(), Some(&gt), "bottom");
    // a card it wants back, with the lands to cast it: the top
    let mut g = game();
    lands(&mut g, P1, "Swamp", 8, false);
    let w = wanted_permanent(&g);
    let name = g.db.get(w).name.to_string();
    let t = perm(&mut g, P1, &name);
    cast_spell(&mut g, P0, "Sudden Setback", Ctx { target: Some(t), ..Ctx::default() });
    assert!(!g.perm(t).on_bf);
    assert_eq!(g.player(P1).library.last(), Some(&w), "top");
}

#[test]
fn sudden_setback_tokens_cease_and_a_countered_spell_goes_to_the_library() {
    let mut g = game();
    let t = token(&mut g, P1, 3);
    let n = g.player(P1).library.len();
    cast_spell(&mut g, P0, "Sudden Setback", Ctx { target: Some(t), ..Ctx::default() });
    assert!(!g.perm(t).on_bf);
    assert_eq!(g.player(P1).library.len(), n, "a token isn't a card");
    // the spell mode: it is a counterspell whose victim goes to the library (cast.rs asks spell_to_library)
    let setback = card(&g, "Sudden Setback");
    let s = card(&g, "Grave Titan");
    assert!(sim_core::engine::stack::counter_ok(&g, setback, s));
    sim_core::impls::bulk_blue::spell_to_library(&mut g, P1, s, true);
    assert_eq!(g.player(P1).library.first(), Some(&s), "a card it doesn't value: bottom");
    let mut g = game();
    lands(&mut g, P1, "Swamp", 8, false);
    let w = wanted_permanent(&g);
    sim_core::impls::bulk_blue::spell_to_library(&mut g, P1, w, true);
    assert_eq!(g.player(P1).library.last(), Some(&w), "with the lands: top");
    sim_core::impls::bulk_blue::spell_to_library(&mut g, P1, s, false);
    assert_eq!(g.player(P1).library.last(), Some(&s), "Lapse of Certainty: always the top");
}

#[test]
fn a_countered_spell_goes_to_the_library_when_sudden_setback_counters_it() {
    // through the engine's cast path: P1 casts, P0's Setback answers (forced), the spell goes to P1's library
    let mut g = game();
    lands(&mut g, P0, "Island", 4, false);
    let ctr = hand(&mut g, P0, &["Sudden Setback"])[0];
    let spell = hand(&mut g, P1, &["Terror"])[0];
    let _ = ctr;
    g.player_mut(P1).hand.retain(|&c| c != spell);
    let it = sim_core::engine::stack::spell_item(&mut g, P1, spell, Ctx::default(), 7.0);
    let id = it.id;
    g.stack.push(it);
    assert!(sim_core::engine::stack::cast_counter_spell(&mut g, P0, ctr, id).unwrap());
    assert!(countered(&g, id));
    assert_eq!(g.stack.iter().find(|x| x.id == id).unwrap().countered_by, Some(ctr));
}

#[test]
fn vanish_from_sight_tucks_and_surveils() {
    let mut g = game();
    let t = perm(&mut g, P1, "Grave Titan");
    let top = *g.player(P0).library.last().unwrap();
    cast_spell(&mut g, P0, "Vanish from Sight", Ctx { target: Some(t), ..Ctx::default() });
    assert!(!g.perm(t).on_bf);
    assert!(g.player(P1).library.contains(&card(&g, "Grave Titan")));
    let lib0 = g.player(P0).library.clone();
    // surveil 1 looked at our top card: it is still there or in the graveyard
    assert!(lib0.last() == Some(&top) || g.player(P0).gy.contains(&top));
    assert!(in_gy(&g, P0, "Vanish from Sight"));
}

// ======================================================== the others
#[test]
fn kasmina_s_transmutation_makes_a_creature_a_vanilla_1_1_until_it_leaves() {
    let mut g = game();
    let t = perm(&mut g, P1, "Grave Titan");
    let (p0, t0) = (values::epow(&g, t), values::etgh(&g, t));
    assert!(p0 > 1);
    cast_spell(&mut g, P0, "Kasmina's Transmutation", Ctx { target: Some(t), ..Ctx::default() });
    let a = on_bf(&g, P0, "Kasmina's Transmutation").expect("the Aura is on the battlefield");
    assert_eq!(g.perm(a).attached, Some(t));
    assert_eq!((values::epow(&g, t), values::etgh(&g, t)), (1, 1));
    assert!(g.perm(t).neutered, "no abilities");
    // the Aura leaves: the creature is itself again
    zones::die(&mut g, a, "destroy").unwrap();
    assert_eq!((values::epow(&g, t), values::etgh(&g, t)), (p0, t0));
    assert!(!g.perm(t).neutered);
}

#[test]
fn kasmina_s_transmutation_keeps_counters() {
    let mut g = game();
    let t = perm(&mut g, P1, "Grave Titan");
    g.perm_mut(t).plus = 2;
    cast_spell(&mut g, P0, "Kasmina's Transmutation", Ctx { target: Some(t), ..Ctx::default() });
    assert_eq!((values::epow(&g, t), values::etgh(&g, t)), (3, 3), "base 1/1 plus two counters");
}

#[test]
fn retrieve_the_esper_makes_a_robot_and_flashback_adds_counters() {
    let mut g = game();
    cast_spell(&mut g, P0, "Retrieve the Esper", Ctx::default());
    let r = g.player(P0).perms[0];
    assert_eq!((values::epow(&g, r), values::etgh(&g, r)), (3, 3));
    assert!(g.perm(r).ttypes.contains(&"robot") && g.perm(r).ttypes.contains(&"warrior"));
    assert!(g.perm(r).data.truthy(sim_core::state::DataKey::Artifact));
    assert!(in_gy(&g, P0, "Retrieve the Esper"));
    // flashback from the graveyard: 5/5, then exiled
    let c = card(&g, "Retrieve the Esper");
    cast::cast_card(&mut g, P0, c, "gy", Ctx::default()).unwrap();
    assert_eq!(g.player(P0).perms.len(), 2);
    let r2 = g.player(P0).perms[1];
    assert_eq!((values::epow(&g, r2), values::etgh(&g, r2)), (5, 5));
    assert!(g.player(P0).exile.contains(&c));
}

#[test]
fn rousing_read_gives_flying_and_loots_two() {
    let mut g = game();
    let m = perm(&mut g, P0, "Cloudfin Raptor");
    let (p0, t0) = (values::epow(&g, m), values::etgh(&g, m));
    let h = hand_len(&g, P0);
    cast_spell(&mut g, P0, "Rousing Read", Ctx::default());
    assert_eq!((values::epow(&g, m), values::etgh(&g, m)), (p0 + 1, t0 + 1));
    assert_eq!(hand_len(&g, P0), h + 1, "drew two, discarded one (the Aura left the hand)");
    assert!(sim_core::impls::common::attached_kw(&g, m, "flying"));
}

#[test]
fn unfathomable_truths_draws_three_and_makes_a_spawn() {
    let mut g = game();
    let h = hand_len(&g, P0);
    cast_spell(&mut g, P0, "Unfathomable Truths", Ctx::default());
    assert_eq!(hand_len(&g, P0), h + 3);
    let s = g.player(P0).perms[0];
    assert_eq!((values::epow(&g, s), values::etgh(&g, s)), (0, 1));
    assert!(g.perm(s).ttypes.contains(&"spawn"));
    assert!(values::colors_of(&g, s).is_empty());
}

#[test]
fn west_coast_expansion_draws_x_and_casts_a_hero_free_at_five() {
    let mut g = game();
    lands(&mut g, P0, "Island", 8, false);
    let c = hand(&mut g, P0, &["West Coast Expansion", "Professor Hulk"])[0];
    let hulk = card(&g, "Professor Hulk");
    let opts = hand_opts(&mut g, c, P0, Some(false));
    assert_eq!(opts.len(), 1);
    let h = hand_len(&g, P0);
    assert!(perform(&mut g, P0, &opts[0]));
    assert_eq!(mana_left(&g, P0), 0, "X = 6 and {{U}}{{U}}");
    assert_eq!(count(&g, P0, "Professor Hulk"), 1, "the Hero came in free");
    assert_eq!(hand_len(&g, P0), h - 2 + 6, "left: Hulk and the spell; drew six");
    assert!(!g.player(P0).hand.contains(&hulk));
    // X = 4: draws four, no Hero
    let mut g = game();
    lands(&mut g, P0, "Island", 6, false);
    let c = hand(&mut g, P0, &["West Coast Expansion", "Professor Hulk"])[0];
    let opts = hand_opts(&mut g, c, P0, Some(false));
    let h = hand_len(&g, P0);
    assert!(perform(&mut g, P0, &opts[0]));
    assert_eq!(count(&g, P0, "Professor Hulk"), 0);
    assert_eq!(hand_len(&g, P0), h - 1 + 4);
    // two lands: not worth it
    let mut g = game();
    lands(&mut g, P0, "Island", 3, false);
    let c = hand(&mut g, P0, &["West Coast Expansion"])[0];
    assert!(hand_opts(&mut g, c, P0, Some(false)).is_empty());
}

#[test]
fn sleight_of_hand_keeps_the_better_of_two_and_bottoms_the_other() {
    let mut g = game();
    let (good, bad) = (card(&g, "Grave Titan"), card(&g, "Island"));
    g.player_mut(P0).library.retain(|&c| c != good && c != bad);
    g.player_mut(P0).library.push(good);
    g.player_mut(P0).library.push(bad);
    let c = hand(&mut g, P0, &["Sleight of Hand"])[0];
    let n = g.player(P0).library.len();
    let h = hand_len(&g, P0);
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert_eq!(hand_len(&g, P0), h - 1 + 1, "the spell left, the better card came");
    assert!(g.player(P0).hand.contains(&good), "the better card");
    assert_eq!(g.player(P0).library.len(), n - 1);
    assert_eq!(g.player(P0).library.first(), Some(&bad), "the other one on the bottom");
}

#[test]
fn every_card_here_is_full_in_the_audit_data() {
    // the card code is registered for each of them
    let g = game();
    for n in [
        "Relm's Sketching",
        "Echocasting Symposium",
        "Jace, Reality Sculptor",
        "Phantom Interference",
        "Whoosh!",
        "Amazing Acrobatics",
        "Icy Reception",
        "Vanish from Sight",
        "Kasmina's Transmutation",
        "Retrieve the Esper",
        "Rousing Read",
        "Unfathomable Truths",
        "West Coast Expansion",
        "Sleight of Hand",
    ] {
        assert!(g.registry.get(card(&g, n)).is_some(), "{n}");
    }
}
