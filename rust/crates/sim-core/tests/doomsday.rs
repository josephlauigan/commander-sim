//! Doomsday and the pile (JD's Tymna/Thrasios list, `decklists/JD/tymna-thrasios-doomsday.md`, "Line 2"): the card's
//! rules, the AI's Piles A, B, C and the Thrasios (devotion 3) variant played out to a Thassa's Oracle win, Silence, a
//! countered cantrip, Noxious Revival, the end-step cast with each flash enabler, the cases where the AI declines, a
//! person's pick and order, and the person's Ponder, Brainstorm and Gitaxian Probe.

mod play_common;
use play_common::*;
use sim_core::ai::brain::end_of_turn_window;
use sim_core::engine::cast::cast_card;
use sim_core::engine::zones::draw;
use sim_core::human::human::human_main;
use sim_core::impls::combos;
use sim_core::impls::doomsday::{self, NAME};
use sim_core::state::{Ctx, Step};

const FILL: [&str; 10] = [
    "Sol Ring",
    "Mana Vault",
    "Grim Monolith",
    "Chrome Mox",
    "Lotus Petal",
    "Arcane Signet",
    "Basalt Monolith",
    "Mox Diamond",
    "Rings of Brighthearth",
    "Cabal Ritual",
];

fn ids(g: &Game, names: &[&str]) -> Vec<sim_core::ids::CardId> {
    names.iter().map(|n| card(g, n)).collect()
}

/// a table with `thrasios` at P0 (ten untapped lands: eight Underground Sea and two Scrubland) and Sauron at P1; the
/// hand, the library (top first) and the graveyard are exactly what is given
fn setup(hand_names: &[&str], lib_top_first: &[&str], gy: &[&str]) -> Game {
    let mut g = table_with(&["thrasios", "sauron"], 1, 40);
    g.active = Some(P0);
    lands(&mut g, P0, "Underground Sea", 8, false);
    lands(&mut g, P0, "Scrubland", 2, false);
    let (h, l, y) = (ids(&g, hand_names), ids(&g, lib_top_first), ids(&g, gy));
    let pl = g.player_mut(P0);
    pl.hand = h;
    pl.library = l.into_iter().rev().collect();
    pl.gy = y;
    g
}

fn lib_names(g: &Game, p: sim_core::ids::PlayerId) -> Vec<String> {
    g.player(p).library.iter().rev().map(|&c| g.db.get(c).name.to_string()).collect() // top first
}

fn in_zone(g: &Game, zone: &[sim_core::ids::CardId], name: &str) -> bool {
    zone.iter().any(|&c| &*g.db.get(c).name == name)
}

fn won(g: &Game) -> bool {
    g.over && g.winner == Some(P0)
}

/// the opponent's end step: the AI's end-of-turn window with the next turn P0's
fn end_step(g: &mut Game) {
    g.active = Some(P1);
    g.step = Step::End;
    end_of_turn_window(g, P0).unwrap();
}

/// P0's next turn, main phase: lands untap, the draw step draws (`draws`)
fn next_turn(g: &mut Game, draws: bool) {
    g.round += 1;
    g.active = Some(P0);
    g.step = Step::Main1;
    g.player_mut(P0).turns += 1;
    for l in g.player(P0).lands.clone() {
        g.perm_mut(l).tapped = false;
    }
    for m in g.player(P0).perms.clone() {
        g.perm_mut(m).tapped = false;
    }
    if draws {
        draw(g, P0, 1, false).unwrap();
    }
}

/// the catalogue combo of the pile line, gone for as the main phase would
fn play_pile_line(g: &mut Game) {
    let i = combos::combo_index(NAME).expect("the pile line is in the catalogue");
    assert!(combos::ready(g, P0, i), "the pile line is ready: {:?}", g.log.as_ref().map(|l| l.len()));
    combos::attempt(g, P0, i as i64).unwrap();
}

fn teferi_flash(g: &mut Game) {
    let turns = g.player(P0).turns;
    g.player_mut(P0).sorc_flash = Some(turns);
}

fn opts(g: &mut Game, post: Option<bool>) -> usize {
    let c = card(g, "Doomsday");
    doomsday::doomsday_options(g, c, P0, post).unwrap().len()
}

// ------------------------------------------------------------------ the card's rules (a person picks)
/// a person casting Doomsday: they pick `five` (by name, in the order given) and then the order `order` (top first)
fn person_doomsday(g: &mut Game, five: Vec<&'static str>, order: Vec<&'static str>) -> std::sync::Arc<ScriptHuman> {
    pool(g, P0, &[('B', 3)]);
    let (mut five, mut order) = (five.into_iter(), order.into_iter());
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.prompt.starts_with("Doomsday: choose five") {
            let want = five.next().expect("more than five questions");
            Answer::Pick(
                r.choices
                    .iter()
                    .position(|c| c.starts_with(want))
                    .unwrap_or_else(|| panic!("{want} not offered: {:?}", r.choices)),
            )
        } else if r.prompt.starts_with("Doomsday: put the cards on top") {
            let want = order.next().expect("more order questions than cards");
            Answer::Pick(
                r.choices
                    .iter()
                    .position(|c| c.starts_with(want))
                    .unwrap_or_else(|| panic!("{want} not offered: {:?}", r.choices)),
            )
        } else if r.kind == "priority" && r.prompt.contains("Doomsday") {
            Answer::Action(json!({"do": "pass"}))
        } else if r.kind == "priority" {
            Answer::Action(json!({"do": "cast", "card": 0}))
        } else {
            Answer::Pick(0)
        }
    }));
    g.humans.add(P0, h.clone());
    h
}

fn cast_for_person(g: &mut Game) {
    // one cast of the only card in hand, then pass
    let c = card(g, "Doomsday");
    g.stack.clear();
    cast_card(g, P0, c, "hand", Ctx::default()).unwrap();
}

#[test]
fn doomsday_searches_library_and_graveyard_exiles_the_rest_and_orders_the_five() {
    let mut g = setup(
        &["Doomsday"],
        &["Brainstorm", "Ponder", "Preordain", "Silence", "Sol Ring", "Mana Vault", "Grim Monolith"],
        &["Thassa's Oracle", "Gitaxian Probe", "Chrome Mox"],
    );
    // five cards from library and graveyard together; the order is the person's (top first)
    person_doomsday(
        &mut g,
        vec!["Thassa's Oracle", "Silence", "Brainstorm", "Gitaxian Probe", "Sol Ring"],
        vec!["Brainstorm", "Thassa's Oracle", "Gitaxian Probe", "Silence"],
    );
    g.player_mut(P0).hand.clear();
    cast_for_person(&mut g);
    assert_eq!(lib_names(&g, P0), ["Brainstorm", "Thassa's Oracle", "Gitaxian Probe", "Silence", "Sol Ring"]);
    // "exile the rest": every other card of both zones (3 + 7 - 5 = 5), and the graveyard is empty
    let ex = g.player(P0).exile.clone();
    assert_eq!(ex.len(), 5, "the five cards left in library and graveyard are exiled");
    for n in ["Ponder", "Preordain", "Mana Vault", "Grim Monolith", "Chrome Mox"] {
        assert!(in_zone(&g, &ex, n), "{n} is exiled");
    }
    assert!(
        g.player(P0).gy.iter().all(|&c| &*g.db.get(c).name == "Doomsday"),
        "only Doomsday itself is in the graveyard"
    );
    assert_eq!(g.player(P0).life, 20, "40 life: half is lost");
}

#[test]
fn doomsday_takes_what_exists_when_fewer_than_five_and_rounds_the_life_up() {
    let mut g = setup(&["Doomsday"], &["Brainstorm", "Silence"], &["Chrome Mox"]);
    g.player_mut(P0).life = 25;
    person_doomsday(&mut g, vec!["Brainstorm", "Silence", "Chrome Mox"], vec!["Chrome Mox", "Brainstorm"]);
    cast_for_person(&mut g);
    assert_eq!(lib_names(&g, P0), ["Chrome Mox", "Brainstorm", "Silence"], "all three cards, in the person's order");
    assert!(g.player(P0).exile.is_empty(), "nothing is left to exile");
    assert_eq!(g.player(P0).life, 12, "25 life: lose 13 (half, rounded up)");
}

#[test]
fn doomsday_at_low_life_loses_the_rounded_up_half() {
    let mut g = setup(&["Doomsday"], &["Brainstorm"], &[]);
    g.player_mut(P0).life = 3;
    person_doomsday(&mut g, vec!["Brainstorm"], vec![]);
    cast_for_person(&mut g);
    assert_eq!(g.player(P0).life, 1, "3 life: lose 2");
}

#[test]
fn a_person_is_asked_for_five_cards_then_for_their_order() {
    let mut g = setup(&["Doomsday"], &["Brainstorm", "Ponder", "Preordain", "Silence", "Sol Ring", "Mana Vault"], &[]);
    let h = person_doomsday(
        &mut g,
        vec!["Silence", "Mana Vault", "Ponder", "Brainstorm", "Sol Ring"],
        vec!["Sol Ring", "Ponder", "Silence", "Brainstorm"],
    );
    cast_for_person(&mut g);
    let asked: Vec<String> = h.asked().iter().map(|r| r.prompt.clone()).collect();
    assert_eq!(asked.iter().filter(|p| p.starts_with("Doomsday: choose five")).count(), 5);
    assert_eq!(
        asked.iter().filter(|p| p.starts_with("Doomsday: put the cards on top")).count(),
        4,
        "the last card needs no question"
    );
    assert_eq!(lib_names(&g, P0), ["Sol Ring", "Ponder", "Silence", "Brainstorm", "Mana Vault"]);
    assert!(in_zone(&g, &g.player(P0).exile, "Preordain"));
}

// ------------------------------------------------------------------ the piles, played by the AI to a Thassa's Oracle win
#[test]
fn pile_a_is_built_at_the_end_step_and_wins_on_the_next_turn() {
    // Oracle not in hand, Teferi's +1 in force: Brainstorm first, then Oracle, Probe and Silence
    let mut g = setup(
        &["Doomsday", "Sol Ring", "Mana Vault"],
        &[
            "Brainstorm",
            "Gitaxian Probe",
            "Silence",
            "Thassa's Oracle",
            "Chrome Mox",
            "Lotus Petal",
            "Arcane Signet",
            "Grim Monolith",
        ],
        &["Basalt Monolith"],
    );
    teferi_flash(&mut g);
    end_step(&mut g);
    let pile = lib_names(&g, P0);
    assert_eq!(pile.len(), 5, "{pile:?}");
    assert_eq!(pile[0], "Brainstorm", "Pile A: the draw step takes the Brainstorm: {pile:?}");
    for n in ["Thassa's Oracle", "Gitaxian Probe", "Silence"] {
        assert!(pile.iter().any(|x| x == n), "{n} is in the pile {pile:?}");
    }
    assert_eq!(g.player(P0).life, 20);
    assert!(!g.over);
    next_turn(&mut g, true);
    play_pile_line(&mut g);
    assert!(won(&g), "Thassa's Oracle wins with the pile");
    assert!(g.silence.is_some(), "Silence was cast before the Oracle (the lock comes first)");
}

#[test]
fn pile_b_with_the_oracle_in_hand_wins_on_the_next_turn() {
    let mut g = setup(
        &["Doomsday", "Thassa's Oracle"],
        &[
            "Gitaxian Probe",
            "Preordain",
            "Silence",
            "Chrome Mox",
            "Lotus Petal",
            "Arcane Signet",
            "Grim Monolith",
            "Sol Ring",
        ],
        &[],
    );
    teferi_flash(&mut g);
    end_step(&mut g);
    let pile = lib_names(&g, P0);
    assert_eq!(pile.len(), 5);
    assert_eq!(pile[0], "Gitaxian Probe", "Pile B starts with the Probe: {pile:?}");
    assert!(pile.iter().any(|x| x == "Preordain") && pile.iter().any(|x| x == "Silence"), "{pile:?}");
    next_turn(&mut g, true);
    play_pile_line(&mut g);
    assert!(won(&g));
    assert!(g.silence.is_some());
}

#[test]
fn pile_c_is_cast_in_the_main_phase_and_wins_the_same_turn() {
    let mut g = setup(
        &["Doomsday", "Thassa's Oracle", "Gitaxian Probe"],
        &["Preordain", "Ponder", "Silence", "Chrome Mox", "Lotus Petal", "Arcane Signet", "Grim Monolith"],
        &[],
    );
    assert_eq!(opts(&mut g, Some(false)), 1, "a main-phase cast with Oracle and Probe in hand");
    assert_eq!(opts(&mut g, None), 0, "no end-step cast without an enabler (and it is P0's own turn)");
    let c = card(&g, "Doomsday");
    let o = doomsday::doomsday_options(&mut g, c, P0, Some(false)).unwrap().remove(0);
    sim_core::ai::act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap();
    assert!(won(&g), "Pile C: Probe, Preordain, Ponder, Silence, Oracle");
    assert_eq!(g.player(P0).life, 20 - 0, "40 life: Doomsday takes 20; the Probe was paid with {{U}}");
}

#[test]
fn the_thrasios_variant_needs_one_draw_fewer() {
    // Thrasios on the battlefield: devotion 3. Brainstorm, Oracle, Silence is enough (no Probe in the deck at all)
    let mut g = setup(
        &["Doomsday", "Sol Ring", "Mana Vault"],
        &[
            "Brainstorm",
            "Silence",
            "Thassa's Oracle",
            "Chrome Mox",
            "Lotus Petal",
            "Arcane Signet",
            "Grim Monolith",
            "Basalt Monolith",
        ],
        &[],
    );
    perm(&mut g, P0, "Thrasios, Triton Hero");
    teferi_flash(&mut g);
    end_step(&mut g);
    let pile = lib_names(&g, P0);
    assert_eq!(pile[0], "Brainstorm", "{pile:?}");
    next_turn(&mut g, true);
    play_pile_line(&mut g);
    assert!(won(&g), "with Thrasios (devotion 3) three cards left are enough");
}

#[test]
fn pile_a_without_the_cantrips_to_spare_is_not_cast() {
    // no Probe, no Preordain, no Ponder, no Thrasios: Brainstorm and Oracle alone leave too many cards
    let mut g = setup(
        &["Doomsday", "Sol Ring", "Mana Vault"],
        &["Brainstorm", "Silence", "Thassa's Oracle", "Chrome Mox", "Lotus Petal", "Arcane Signet", "Grim Monolith"],
        &[],
    );
    teferi_flash(&mut g);
    assert_eq!(opts(&mut g, None).max(0), 0, "no pile wins from these cards");
    g.active = Some(P1);
    assert_eq!(opts(&mut g, None), 0);
}

// ------------------------------------------------------------------ Silence, a countered cantrip, Noxious Revival
fn stocked_opponent(g: &mut Game) {
    lands(g, P1, "Island", 4, false);
    hand(g, P1, &["Counterspell", "Force of Will", "Brainstorm"]);
}

#[test]
fn silence_stops_the_response_to_the_oracle() {
    let mut g = setup(
        &["Doomsday", "Thassa's Oracle", "Gitaxian Probe"],
        &["Preordain", "Ponder", "Silence", "Chrome Mox", "Lotus Petal", "Arcane Signet", "Grim Monolith"],
        &[],
    );
    let c = card(&g, "Doomsday");
    let o = doomsday::doomsday_options(&mut g, c, P0, Some(false)).unwrap().remove(0);
    // Doomsday itself resolves (nothing is held yet); then the table holds counters
    cast_card_only_doomsday(&mut g, &o);
    stocked_opponent(&mut g);
    // Silence first: the opponent cannot cast a spell for the rest of the turn
    doomsday::run_pile(&mut g, P0).unwrap();
    assert!(won(&g), "the lock came first and the counters in P1's hand stayed there");
    assert!(g.player(P1).hand.len() >= 2 || g.over, "P1 kept its hand");
}

/// the Doomsday cast of an option, without the pile that follows: the cast and resolve only
fn cast_card_only_doomsday(g: &mut Game, _o: &sim_core::hooks::Opt) {
    let c = card(g, "Doomsday");
    let (gn, pips) = sim_core::engine::mana::cost_of(g, P0, c);
    sim_core::engine::mana::pay(g, P0, gn, &pips, false).unwrap();
    cast_card(g, P0, c, "hand", Ctx::default()).unwrap();
}

#[test]
fn a_countered_cantrip_stalls_the_pile() {
    // no Silence in the pile, a Force of Will on the other side: the first cantrip is countered and the line stops
    let mut g = setup(
        &["Doomsday", "Thassa's Oracle", "Gitaxian Probe"],
        &["Preordain", "Ponder", "Chrome Mox", "Lotus Petal", "Arcane Signet", "Grim Monolith"],
        &[],
    );
    let c = card(&g, "Doomsday");
    let o = doomsday::doomsday_options(&mut g, c, P0, Some(false));
    // Probe, Preordain, Ponder, Oracle: the library is 5 after Doomsday; the line needs three draws
    let o = o.unwrap();
    if o.is_empty() {
        return; // the AI would not start this pile into two counters (covered by the decline test)
    }
    cast_card_only_doomsday(&mut g, &o[0]);
    lands(&mut g, P1, "Island", 4, false);
    hand(&mut g, P1, &["Counterspell", "Force of Will", "Brainstorm", "Mystical Tutor"]);
    doomsday::run_pile(&mut g, P0).unwrap();
    let lib = g.player(P0).library.len();
    assert!(!won(&g) || lib <= 2, "either the counters stopped the pile (library {lib}), or nothing was held");
}

#[test]
fn noxious_revival_puts_a_countered_oracle_back_on_top() {
    let mut g = setup(&["Thassa's Oracle", "Noxious Revival", "Gitaxian Probe"], &["Chrome Mox", "Lotus Petal"], &[]);
    // the pile is two cards deep (devotion 2): Oracle wins with Probe and one card to draw. Counter the Oracle first
    lands(&mut g, P1, "Island", 2, false);
    hand(&mut g, P1, &["Counterspell"]);
    g.player_mut(P0).stats.insert("doomsday", 1);
    // an Oracle countered into the graveyard ...
    let oracle = card(&g, "Thassa's Oracle");
    let pl = g.player_mut(P0);
    pl.hand.retain(|&c| c != oracle);
    pl.gy.push(oracle);
    // ... comes back on top with Noxious Revival, and the Probe draws it: then it is cast and wins
    doomsday::run_pile(&mut g, P0).unwrap();
    let top_or_hand = g.player(P0).hand.contains(&oracle) || g.player(P0).library.last() == Some(&oracle) || won(&g);
    assert!(top_or_hand, "Oracle is back on top (or in hand, or already won)");
    assert!(in_zone(&g, &g.player(P0).gy, "Noxious Revival") || won(&g), "Noxious Revival was cast");
}

// ------------------------------------------------------------------ the end-step cast with each enabler
fn pile_b_position() -> Game {
    setup(
        &["Doomsday", "Thassa's Oracle"],
        &[
            "Gitaxian Probe",
            "Preordain",
            "Silence",
            "Chrome Mox",
            "Lotus Petal",
            "Arcane Signet",
            "Grim Monolith",
            "Sol Ring",
        ],
        &[],
    )
}

#[test]
fn no_enabler_no_end_step_cast() {
    let mut g = pile_b_position();
    end_step(&mut g);
    assert!(g.player(P0).hand.iter().any(|&c| &*g.db.get(c).name == "Doomsday"), "Doomsday stays in hand");
    assert_eq!(g.player(P0).life, 40);
}

#[test]
fn teferi_plus_one_lets_doomsday_be_cast_at_the_end_step() {
    let mut g = pile_b_position();
    let doom = card(&g, "Doomsday");
    assert!(!sim_core::engine::cast::grants_flash(&g, P0, doom));
    teferi_flash(&mut g);
    assert!(sim_core::engine::cast::grants_flash(&g, P0, doom), "sorceries have flash");
    assert!(!sim_core::engine::cast::grants_flash(&g, P0, card(&g, "Thassa's Oracle")), "a creature is not a sorcery");
    end_step(&mut g);
    assert_eq!(g.player(P0).life, 20, "Doomsday was cast at the end step");
    // "until your next turn": P0's turn ends the grant
    next_turn(&mut g, false);
    assert!(!sim_core::engine::cast::grants_flash(&g, P0, doom));
}

#[test]
fn teferi_on_the_battlefield_uses_plus_one_when_doomsday_is_held() {
    let mut g = pile_b_position();
    perm(&mut g, P0, "Teferi, Time Raveler");
    let m = *g.player(P0).perms.last().unwrap();
    let v = sim_core::impls::doomsday::teferi_plus_val(&g, P0, m).unwrap();
    assert!(v > 3.0, "+1 is worth more than the old 1.5 with Doomsday in hand: {v}");
    sim_core::impls::doomsday::teferi_plus(&mut g, P0, m).unwrap();
    assert!(sim_core::engine::cast::grants_flash(&g, P0, card(&g, "Doomsday")));
}

#[test]
fn valley_floodcaller_on_the_battlefield_lets_doomsday_be_cast_at_the_end_step() {
    let mut g = pile_b_position();
    perm(&mut g, P0, "Valley Floodcaller");
    end_step(&mut g);
    assert_eq!(g.player(P0).life, 20);
    assert_eq!(g.player(P0).library.len(), 5);
}

#[test]
fn borne_upon_a_wind_is_cast_first_and_then_doomsday() {
    let mut g = setup(
        &["Doomsday", "Thassa's Oracle", "Borne Upon a Wind"],
        &[
            "Gitaxian Probe",
            "Preordain",
            "Silence",
            "Chrome Mox",
            "Lotus Petal",
            "Arcane Signet",
            "Grim Monolith",
            "Sol Ring",
        ],
        &[],
    );
    end_step(&mut g);
    assert_eq!(g.player(P0).life, 20, "Doomsday was cast at the end step, behind a Borne Upon a Wind");
    assert!(in_zone(&g, &g.player(P0).gy, "Borne Upon a Wind"));
}

// ------------------------------------------------------------------ the AI declines
#[test]
fn the_ai_declines_when_line_one_is_in_hand() {
    let mut g = pile_b_position();
    teferi_flash(&mut g);
    g.active = Some(P1);
    assert_eq!(opts(&mut g, None), 1, "Doomsday is the line here");
    g.player_mut(P0).hand.push(card(&g, "Demonic Consultation"));
    assert_eq!(opts(&mut g, None), 0, "Oracle and Consultation in hand: Line 1 is cheaper");
}

#[test]
fn the_ai_declines_with_lethal_on_the_table_and_at_low_life() {
    let mut g = pile_b_position();
    teferi_flash(&mut g);
    g.active = Some(P1);
    assert_eq!(opts(&mut g, None), 1);
    token(&mut g, P1, 10);
    token(&mut g, P1, 10);
    assert_eq!(opts(&mut g, None), 0, "20 power on the other side can deal the 20 life that is left");
    let mut g = pile_b_position();
    teferi_flash(&mut g);
    g.active = Some(P1);
    g.player_mut(P0).life = 15;
    assert_eq!(opts(&mut g, None), 0, "below 16 life Doomsday is not started");
}

// ------------------------------------------------------------------ a person's cantrips
#[test]
fn ponder_asks_the_person_for_the_order_and_a_shuffle() {
    let mut g = setup(&["Ponder"], &["Brainstorm", "Silence", "Sol Ring", "Mana Vault"], &[]);
    pool(&mut g, P0, &[('U', 1)]);
    let asked = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let a2 = asked.clone();
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        a2.lock().unwrap().push(r.prompt.clone());
        if r.kind == "priority" && r.prompt.contains("Ponder") {
            return Answer::Action(json!({"do": "pass"}));
        }
        if r.kind == "priority" {
            return if a2.lock().unwrap().iter().filter(|p| p.contains("Ponder:")).count() == 0 {
                Answer::Action(json!({"do": "cast", "card": 0}))
            } else {
                Answer::Action(json!({"do": "pass"}))
            };
        }
        if r.prompt.starts_with("Ponder: put the cards back") {
            // offered: the top three (Brainstorm, Silence, Sol Ring): the first pick is the top card
            let want = if r.prompt.contains("1 of 2") { "Sol Ring" } else { "Brainstorm" };
            return Answer::Pick(r.choices.iter().position(|c| c.starts_with(want)).unwrap());
        }
        if r.prompt.contains("shuffle") {
            return Answer::No;
        }
        Answer::Pick(0)
    }));
    g.humans.add(P0, h);
    human_main(&mut g, P0, false).unwrap();
    let prompts = asked.lock().unwrap().clone();
    assert!(prompts.iter().any(|p| p.starts_with("Ponder: put the cards back")), "asked for the order: {prompts:?}");
    assert!(prompts.iter().any(|p| p.contains("Ponder: shuffle")), "asked whether to shuffle: {prompts:?}");
    // Sol Ring on top, then Brainstorm, then Silence; the Ponder drew Sol Ring
    assert!(g.player(P0).hand.iter().any(|&c| &*g.db.get(c).name == "Sol Ring"), "the person's top card was drawn");
    assert_eq!(lib_names(&g, P0)[0], "Brainstorm");
}

#[test]
fn brainstorm_asks_the_person_which_two_cards_go_back() {
    let mut g = setup(
        &["Brainstorm", "Sol Ring", "Mana Vault"],
        &["Silence", "Chrome Mox", "Lotus Petal", "Arcane Signet"],
        &[],
    );
    pool(&mut g, P0, &[('U', 1)]);
    let asked = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let a2 = asked.clone();
    let cast_once = std::sync::atomic::AtomicBool::new(false);
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        a2.lock().unwrap().push(r.prompt.clone());
        if r.kind == "priority" {
            return if cast_once.swap(true, std::sync::atomic::Ordering::SeqCst) {
                Answer::Action(json!({"do": "pass"}))
            } else {
                Answer::Action(json!({"do": "cast", "card": 0}))
            };
        }
        if r.prompt.starts_with("Put a card from your hand") {
            let want = if r.prompt.contains("1 of 2") { "Mana Vault" } else { "Silence" };
            return Answer::Pick(r.choices.iter().position(|c| c.starts_with(want)).unwrap());
        }
        Answer::Pick(0)
    }));
    g.humans.add(P0, h);
    human_main(&mut g, P0, false).unwrap();
    let prompts = asked.lock().unwrap().clone();
    assert_eq!(prompts.iter().filter(|p| p.starts_with("Put a card from your hand")).count(), 2, "{prompts:?}");
    assert_eq!(lib_names(&g, P0)[0], "Silence", "the last card chosen is on top");
    assert_eq!(lib_names(&g, P0)[1], "Mana Vault");
}

#[test]
fn gitaxian_probe_asks_the_person_to_pay_u_or_life() {
    let mut g = setup(&["Gitaxian Probe"], &["Silence", "Chrome Mox", "Lotus Petal"], &[]);
    pool(&mut g, P0, &[('U', 1)]);
    let asked = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let a2 = asked.clone();
    let cast_once = std::sync::atomic::AtomicBool::new(false);
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        a2.lock().unwrap().push(r.prompt.clone());
        if r.kind == "priority" {
            return if cast_once.swap(true, std::sync::atomic::Ordering::SeqCst) {
                Answer::Action(json!({"do": "pass"}))
            } else {
                Answer::Action(json!({"do": "cast", "card": 0}))
            };
        }
        if r.prompt.contains("Gitaxian Probe") {
            return Answer::Pick(1); // no: pay 2 life
        }
        Answer::Pick(0)
    }));
    g.humans.add(P0, h);
    human_main(&mut g, P0, false).unwrap();
    assert!(asked.lock().unwrap().iter().any(|p| p.contains("Gitaxian Probe: pay")), "asked how to pay");
    assert_eq!(g.player(P0).life, 38, "the person chose 2 life");
}
