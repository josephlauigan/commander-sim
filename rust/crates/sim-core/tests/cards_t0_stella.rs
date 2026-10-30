//! The Stella Lee, Wild Card precon's cards (impls/t0_stella.rs) on hand-built positions. Seat 0 plays the precon,
//! seat 1 the seph deck.

use sim_core::ai::act;
use sim_core::engine::hooks::fire_trigger;
use sim_core::engine::{cast, mana, values};
use sim_core::hooks::{Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["stella-lee-quick-draw", "seph"])
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

fn in_exile(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).exile.iter().any(|&c| &*g.db.get(c).name == name)
}

fn perm_opts(g: &mut Game, m: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.perm(m).cd.unwrap()).and_then(|i| i.options).expect("the card has options");
    f(g, m, p, post).unwrap()
}

fn hand_opts(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).and_then(|i| i.hand_options).expect("the card has hand options");
    f(g, c, p, post).unwrap()
}

fn gy_opts(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).and_then(|i| i.gy_options).expect("the card has graveyard options");
    f(g, c, p, post).unwrap()
}

fn perform(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn tokens_of(g: &Game, p: PlayerId, pow: i32) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).token && g.perm(m).pow == pow).count()
}

fn fill_gy(g: &mut Game, p: PlayerId, names: &[&str]) {
    for n in names {
        let c = take(g, p, n);
        g.player_mut(p).gy.push(c);
    }
}

// ======================================================== storm, Elemental Eruption, Crackling Spellslinger
#[test]
fn elemental_eruption_makes_a_dragon_and_a_copy_for_each_earlier_spell() {
    let mut g = game();
    cast_spell(&mut g, P0, "Opt", Ctx::default());
    cast_spell(&mut g, P0, "Preordain", Ctx::default());
    cast_spell(&mut g, P0, "Elemental Eruption", Ctx::default());
    assert_eq!(tokens_of(&g, P0, 4), 3, "the Dragon and two copies' Dragons");
    let d = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token).unwrap();
    assert!(g.perm(d).fly && g.perm(d).data.truthy(DataKey::Prowess), "flying and prowess");
    assert!(in_gy(&g, P0, "Elemental Eruption"));
}

#[test]
fn crackling_spellslinger_gives_the_next_spell_storm_once() {
    let mut g = game();
    let c = hand(&mut g, P0, &["Crackling Spellslinger", "Opt", "Ponder"]);
    cast::cast_card(&mut g, P0, c[0], "hand", Ctx::default()).unwrap();
    assert!(g.player(P0).t0s.storm_next.is_some());
    let n = g.player(P0).hand.len();
    cast::cast_card(&mut g, P0, c[1], "hand", Ctx::default()).unwrap();
    // Opt draws, and its storm copy (one spell before it) draws too
    assert_eq!(g.player(P0).hand.len(), n - 1 + 2);
    assert!(g.player(P0).t0s.storm_next.is_none());
    let n = g.player(P0).hand.len();
    cast::cast_card(&mut g, P0, c[2], "hand", Ctx::default()).unwrap();
    assert!(g.player(P0).hand.len() <= n, "Ponder has no storm: it draws one card and leaves the hand");
}

#[test]
fn spellslinger_not_cast_does_nothing() {
    let mut g = game();
    perm(&mut g, P0, "Crackling Spellslinger"); // put onto the battlefield, not cast
    assert!(g.player(P0).t0s.storm_next.is_none());
}

// ======================================================== Kaza, Thunderclap Drake, Octavia
#[test]
fn kaza_makes_the_next_instant_or_sorcery_cheaper_by_your_wizards() {
    let mut g = game();
    let k = perm(&mut g, P0, "Kaza, Roil Chaser");
    perm(&mut g, P0, "Goblin Electromancer");
    let e = hand(&mut g, P0, &["Elemental Eruption", "Opt"]);
    let before = mana::cost_of(&g, P0, e[0]).0;
    let opts = perm_opts(&mut g, k, P0, Some(false));
    assert!(perform(&mut g, P0, &opts[0]));
    assert!(g.perm(k).tapped);
    // Kaza and the Electromancer are Wizards; the Electromancer also makes instants and sorceries cost {1} less
    assert_eq!(mana::cost_of(&g, P0, e[0]).0, before - 2);
    cast::cast_card(&mut g, P0, e[1], "hand", Ctx::default()).unwrap();
    assert_eq!(mana::cost_of(&g, P0, e[0]).0, before, "used up by the next spell");
}

#[test]
fn thunderclap_drake_copies_the_next_spell_once_per_commander_cast() {
    let mut g = game();
    let d = perm(&mut g, P0, "Thunderclap Drake");
    lands(&mut g, P0, "Island", 4, false);
    let cmd = g.player(P0).cmd;
    g.player_mut(P0).add_tax(cmd); // cast once from the command zone
    let c = hand(&mut g, P0, &["Big Score", "Preordain"]);
    let opts = perm_opts(&mut g, d, P0, Some(false));
    assert_eq!(opts.len(), 1, "worth it with a spell and the mana");
    assert!(perform(&mut g, P0, &opts[0]));
    assert!(on_bf(&g, P0, "Thunderclap Drake").is_none(), "sacrificed");
    let n = g.player(P0).hand.len();
    cast::cast_card(&mut g, P0, c[1], "hand", Ctx::default()).unwrap();
    // Preordain: draws 1; its one copy draws 1 more (the original left the hand)
    assert_eq!(g.player(P0).hand.len(), n - 1 + 2);
    assert!(g.player(P0).t0s.drake_next.is_none());
}

#[test]
fn octavia_costs_eight_less_and_sets_a_creature_to_8_8() {
    let mut g = game();
    let o = hand(&mut g, P0, &["Octavia, Living Thesis"])[0];
    assert_eq!(mana::cost_of(&g, P0, o).0, 8);
    fill_gy(
        &mut g,
        P0,
        &["Opt", "Ponder", "Preordain", "Big Score", "Deep Analysis", "Dig Through Time", "Pongify", "Chaos Warp"],
    );
    assert_eq!(mana::cost_of(&g, P0, o).0, 0, "eight instants and sorceries");
    perm(&mut g, P0, "Octavia, Living Thesis");
    let t = token(&mut g, P0, 2);
    cast_spell(&mut g, P0, "Opt", Ctx::default());
    assert_eq!(values::epow(&g, t), 8, "base power 8");
    assert_eq!(values::etgh(&g, t), 8, "base toughness 8");
    // a copy triggers it too; the creature is not changed twice
    let opt = card(&g, "Opt");
    cast::copy_spell(&mut g, P0, opt, None, false).unwrap();
    assert_eq!(values::epow(&g, t), 8);
}

// ======================================================== Stella Lee, Arcane Bombardment
#[test]
fn stella_exiles_on_the_second_spell_and_copies_the_third() {
    let mut g = game();
    let s = perm(&mut g, P0, "Stella Lee, Wild Card");
    let top = card(&g, "Mountain");
    g.player_mut(P0).library.push(top);
    cast_spell(&mut g, P0, "Opt", Ctx::default());
    let lib = g.player(P0).library.len();
    let h = g.player(P0).hand.len();
    cast_spell(&mut g, P0, "Preordain", Ctx::default());
    assert!(
        g.player(P0).impulse_long.iter().any(|x| x.0 == top) || g.player(P0).hand.contains(&top),
        "the top card is playable"
    );
    assert!(g.player(P0).library.len() < lib, "the top card was exiled");
    assert!(!g.perm(s).tapped);
    let h2 = g.player(P0).hand.len();
    let _ = h;
    cast_spell(&mut g, P0, "Ponder", Ctx::default());
    assert!(g.perm(s).tapped, "the third spell is copied by tapping Stella");
    assert!(g.player(P0).hand.len() > h2 - 1);
}

#[test]
fn arcane_bombardment_casts_copies_of_what_it_exiled() {
    let mut g = game();
    let b = perm(&mut g, P0, "Arcane Bombardment");
    fill_gy(&mut g, P0, &["Opt"]);
    let n = g.player(P0).hand.len();
    cast_spell(&mut g, P0, "Preordain", Ctx::default());
    assert!(in_exile(&g, P0, "Opt"), "the graveyard's only instant or sorcery");
    assert!(g.player(P0).hand.len() >= n + 2, "Preordain and the copy of Opt both draw");
    assert_eq!(g.player(P0).t0s.bombard.iter().find(|e| e.0 == b).unwrap().1.len(), 1);
    // only the first spell each turn triggers it
    cast_spell(&mut g, P0, "Ponder", Ctx::default());
    assert_eq!(g.player(P0).t0s.bombard[0].1.len(), 1);
}

// ======================================================== sorceries
#[test]
fn baral_bounces_the_opponents_best_and_casts_a_cheap_spell_free() {
    let mut g = game();
    let s = perm(&mut g, P1, "Sheoldred, the Apocalypse");
    hand(&mut g, P0, &["Ponder"]);
    cast_spell(&mut g, P0, "Baral's Expertise", Ctx::default());
    assert!(!g.perm(s).on_bf, "Sheoldred went back to the hand");
    assert!(g.player(P1).hand.contains(&card(&g, "Sheoldred, the Apocalypse")));
    assert!(in_gy(&g, P0, "Ponder"), "Ponder was cast free");
}

#[test]
fn curse_of_the_swine_exiles_x_creatures_and_leaves_boars() {
    let mut g = game();
    let s = perm(&mut g, P1, "Sheoldred, the Apocalypse");
    cast_spell(&mut g, P0, "Curse of the Swine", Ctx { x: 1, ..Ctx::default() });
    assert!(!g.perm(s).on_bf);
    assert!(in_exile(&g, P1, "Sheoldred, the Apocalypse"));
    assert_eq!(tokens_of(&g, P1, 2), 1, "a 2/2 Boar for its controller");
}

#[test]
fn curse_of_the_swine_is_offered_for_a_good_target() {
    let mut g = game();
    lands(&mut g, P0, "Island", 5, false);
    perm(&mut g, P1, "Sheoldred, the Apocalypse");
    let c = hand(&mut g, P0, &["Curse of the Swine"])[0];
    let opts = hand_opts(&mut g, c, P0, Some(false));
    assert_eq!(opts.len(), 1);
    assert!(perform(&mut g, P0, &opts[0]));
    assert!(in_exile(&g, P1, "Sheoldred, the Apocalypse"));
}

#[test]
fn epic_experiment_casts_the_spells_it_exiled_free() {
    let mut g = game();
    let (a, b, l) = (card(&g, "Opt"), card(&g, "Ponder"), card(&g, "Mountain"));
    take(&mut g, P0, "Opt");
    take(&mut g, P0, "Ponder");
    g.player_mut(P0).library.extend([l, b, a]); // Opt on top
    let n = g.player(P0).library.len();
    cast_spell(&mut g, P0, "Epic Experiment", Ctx { x: 3, ..Ctx::default() });
    assert!(in_gy(&g, P0, "Opt") && in_gy(&g, P0, "Ponder"), "both were cast free");
    assert!(in_gy(&g, P0, "Mountain"), "the uncast card goes to the graveyard");
    assert!(g.player(P0).library.len() <= n - 3 + 2, "three cards exiled (Opt and Ponder draw back)");
}

#[test]
fn epic_experiment_does_not_cast_what_costs_more_than_x() {
    let mut g = game();
    let a = take(&mut g, P0, "Deep Analysis");
    g.player_mut(P0).library.push(a);
    cast_spell(&mut g, P0, "Epic Experiment", Ctx { x: 1, ..Ctx::default() });
    assert!(in_gy(&g, P0, "Deep Analysis"));
    assert_eq!(g.player(P0).hand.len(), 0, "not cast: nothing drawn");
}

#[test]
fn finale_of_promise_casts_an_instant_and_a_sorcery_and_exiles_them() {
    let mut g = game();
    fill_gy(&mut g, P0, &["Opt", "Ponder"]);
    cast_spell(&mut g, P0, "Finale of Promise", Ctx { x: 1, ..Ctx::default() });
    assert!(in_exile(&g, P0, "Opt") && in_exile(&g, P0, "Ponder"), "exiled instead of the graveyard");
    assert!(!in_gy(&g, P0, "Opt") && !in_gy(&g, P0, "Ponder"));
    assert!(g.player(P0).hand.len() >= 2, "Opt drew a card and Ponder drew a card");
}

#[test]
fn finale_of_promise_x_ten_copies_each_spell_twice() {
    let mut g = game();
    fill_gy(&mut g, P0, &["Opt"]);
    cast_spell(&mut g, P0, "Finale of Promise", Ctx { x: 10, ..Ctx::default() });
    assert_eq!(g.player(P0).hand.len(), 3, "Opt and its two copies each drew a card");
}

#[test]
fn finale_of_revelation_draws_x_and_exiles_itself() {
    let mut g = game();
    cast_spell(&mut g, P0, "Finale of Revelation", Ctx { x: 4, ..Ctx::default() });
    assert_eq!(g.player(P0).hand.len(), 4);
    assert!(in_exile(&g, P0, "Finale of Revelation") && !in_gy(&g, P0, "Finale of Revelation"));
}

#[test]
fn finale_of_revelation_x_ten_shuffles_untaps_and_removes_the_hand_limit() {
    let mut g = game();
    fill_gy(&mut g, P0, &["Opt", "Ponder"]);
    let ls = lands(&mut g, P0, "Island", 6, true);
    cast_spell(&mut g, P0, "Finale of Revelation", Ctx { x: 10, ..Ctx::default() });
    assert_eq!(g.player(P0).hand.len(), 10);
    assert!(!in_gy(&g, P0, "Opt") || g.player(P0).gy.len() <= 1, "the graveyard went into the library");
    assert_eq!(ls.iter().filter(|&&l| !g.perm(l).tapped).count(), 5, "up to five lands untapped");
    assert!(g.player(P0).t0s.nomax);
}

#[test]
fn rousing_refrain_adds_red_mana_and_suspends_itself() {
    let mut g = game();
    for _ in 0..3 {
        let c = take(&mut g, P1, "Swamp");
        g.player_mut(P1).hand.push(c);
    }
    let c = cast_spell(&mut g, P0, "Rousing Refrain", Ctx::default());
    assert_eq!(g.player(P0).floating.r, 3);
    assert!(g.player(P0).exile.contains(&c) && g.player(P0).suspended.contains(&(c, 3)));
    for _ in 0..3 {
        sim_core::cardcode::suspend_upkeep(&mut g, P0).unwrap();
    }
    assert!(g.player(P0).floating.r >= 6, "cast free from suspend: more mana");
}

#[test]
fn rousing_refrain_can_be_suspended_for_two() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 2, false);
    for _ in 0..3 {
        let c = take(&mut g, P1, "Swamp");
        g.player_mut(P1).hand.push(c);
    }
    let c = hand(&mut g, P0, &["Rousing Refrain"])[0];
    let opts = hand_opts(&mut g, c, P0, Some(false));
    assert!(perform(&mut g, P0, &opts[0]));
    assert!(g.player(P0).suspended.contains(&(c, 3)) && g.player(P0).exile.contains(&c));
}

#[test]
fn volcanic_torrent_cascades_then_damages_by_spells_cast() {
    let mut g = game();
    let big = perm(&mut g, P1, "Sheoldred, the Apocalypse"); // 4/5
    let small = token(&mut g, P1, 2); // 2/2
    cast_spell(&mut g, P0, "Opt", Ctx::default());
    cast_spell(&mut g, P0, "Ponder", Ctx::default());
    cast_spell(&mut g, P0, "Volcanic Torrent", Ctx::default());
    let x = cast::casts_this_turn(&g, P0);
    assert!(x >= 3, "two spells and the Torrent");
    assert!(!g.perm(small).on_bf, "{x} damage kills the 2/2");
    assert!(g.perm(big).on_bf == (x < 5), "x = {x}, Sheoldred {}/{}", values::epow(&g, big), values::etgh(&g, big));
}

// ======================================================== creatures and permanents
#[test]
fn bloodthirsty_adversary_pays_for_counters_and_copies() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 5, false);
    fill_gy(&mut g, P0, &["Opt"]);
    let n = g.player(P0).hand.len();
    cast_spell(&mut g, P0, "Bloodthirsty Adversary", Ctx::default());
    let m = on_bf(&g, P0, "Bloodthirsty Adversary").unwrap();
    assert_eq!(g.perm(m).plus, 1, "paid {{2}}{{R}} once");
    assert!(in_exile(&g, P0, "Opt"));
    assert_eq!(g.player(P0).hand.len(), n + 1, "the copy of Opt drew a card");
}

#[test]
fn bloodthirsty_adversary_pays_nothing_without_targets() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 5, false);
    cast_spell(&mut g, P0, "Bloodthirsty Adversary", Ctx::default());
    let m = on_bf(&g, P0, "Bloodthirsty Adversary").unwrap();
    assert_eq!(g.perm(m).plus, 0);
}

#[test]
fn cursed_mirror_copies_the_best_creature_until_end_of_turn() {
    let mut g = game();
    perm(&mut g, P1, "Sheoldred, the Apocalypse");
    cast_spell(&mut g, P0, "Cursed Mirror", Ctx::default());
    let m = on_bf(&g, P0, "Cursed Mirror").unwrap();
    assert!(g.is_creature(m), "a creature copy this turn");
    assert!(values::epow(&g, m) >= 4 && !g.perm(m).sick, "a 4/5 with haste");
    fire_trigger(&mut g, Event::EndStep, Call::Player { p: P0 }).unwrap();
    assert!(!g.is_creature(m), "an artifact again");
}

#[test]
fn cursed_mirror_stays_an_artifact_with_nothing_to_copy() {
    let mut g = game();
    cast_spell(&mut g, P0, "Cursed Mirror", Ctx::default());
    let m = on_bf(&g, P0, "Cursed Mirror").unwrap();
    assert!(!g.is_creature(m));
}

#[test]
fn forger_s_foundry_exiles_the_spell_its_mana_paid_for_and_casts_it_later() {
    let mut g = game();
    let f = perm(&mut g, P0, "Forger's Foundry");
    lands(&mut g, P0, "Island", 5, false);
    let o = hand(&mut g, P0, &["Opt"])[0];
    // the Foundry's mana is spent while paying for Opt
    g.pay_for = Some(o);
    let tapped = g.registry.get(g.perm(f).cd.unwrap()).and_then(|i| i.mana_tapped).unwrap();
    tapped(&mut g, f, P0, f, 1).unwrap();
    g.pay_for = None;
    cast::cast_card(&mut g, P0, o, "hand", Ctx::default()).unwrap();
    assert!(g.player(P0).exile.contains(&o) && !g.player(P0).gy.contains(&o), "exiled with the Foundry");
    let n = g.player(P0).hand.len();
    let opts = perm_opts(&mut g, f, P0, Some(false));
    assert_eq!(opts.len(), 1);
    assert!(perform(&mut g, P0, &opts[0]));
    assert!(g.perm(f).tapped && g.player(P0).hand.len() == n + 1, "Opt cast free: it drew");
}

#[test]
fn forger_s_foundry_ignores_a_spell_its_mana_did_not_pay_for() {
    let mut g = game();
    perm(&mut g, P0, "Forger's Foundry");
    let o = cast_spell(&mut g, P0, "Opt", Ctx::default());
    assert!(g.player(P0).gy.contains(&o));
}

#[test]
fn leyline_dowser_mills_and_keeps_an_instant_or_sorcery() {
    let mut g = game();
    let d = perm(&mut g, P0, "Leyline Dowser");
    lands(&mut g, P0, "Island", 3, false);
    let l = token(&mut g, P0, 1);
    let _ = l;
    let top = take(&mut g, P0, "Opt");
    g.player_mut(P0).library.push(top);
    let opts = perm_opts(&mut g, d, P0, Some(true));
    assert_eq!(opts.len(), 1);
    assert!(perform(&mut g, P0, &opts[0]));
    assert!(g.perm(d).tapped);
    assert!(g.player(P0).hand.contains(&top), "the milled instant went to the hand");
}

#[test]
fn leyline_dowser_untaps_by_tapping_a_legendary_creature() {
    let mut g = game();
    let d = perm(&mut g, P0, "Leyline Dowser");
    lands(&mut g, P0, "Island", 6, false);
    let s = perm(&mut g, P0, "Stella Lee, Wild Card");
    let opts = perm_opts(&mut g, d, P0, Some(true));
    assert!(perform(&mut g, P0, &opts[0]));
    assert!(g.perm(s).tapped, "Stella was tapped to untap the Dowser");
}

#[test]
fn midnight_clock_counts_hours_and_wheels_at_twelve() {
    let mut g = game();
    let c = perm(&mut g, P0, "Midnight Clock");
    hand(&mut g, P0, &["Opt", "Ponder"]);
    fire_trigger(&mut g, Event::Upkeep, Call::Player { p: P1 }).unwrap();
    assert_eq!(g.perm(c).data.int(DataKey::T0Stella), 1, "each upkeep");
    g.perm_mut(c).data.set(DataKey::T0Stella, Val::Int(11));
    fire_trigger(&mut g, Event::Upkeep, Call::Player { p: P0 }).unwrap();
    assert!(!g.perm(c).on_bf && in_exile(&g, P0, "Midnight Clock"), "exiled");
    assert_eq!(g.player(P0).hand.len(), 7, "a new hand of seven");
}

#[test]
fn midnight_clock_is_wound_with_spare_mana_and_an_empty_hand() {
    let mut g = game();
    let c = perm(&mut g, P0, "Midnight Clock");
    lands(&mut g, P0, "Island", 3, false);
    g.perm_mut(c).data.set(DataKey::T0Stella, Val::Int(5));
    let opts = perm_opts(&mut g, c, P0, Some(true));
    assert!(perform(&mut g, P0, &opts[0]));
    assert_eq!(g.perm(c).data.int(DataKey::T0Stella), 6);
}

#[test]
fn pteramander_adapts_for_less_with_spells_in_the_graveyard() {
    let mut g = game();
    let m = perm(&mut g, P0, "Pteramander");
    lands(&mut g, P0, "Island", 4, false);
    assert!(perm_opts(&mut g, m, P0, Some(true)).is_empty(), "{{7}}{{U}} is out of reach");
    fill_gy(&mut g, P0, &["Opt", "Ponder", "Preordain", "Big Score", "Deep Analysis"]);
    let opts = perm_opts(&mut g, m, P0, Some(true));
    assert_eq!(opts.len(), 1, "{{2}}{{U}} with five in the graveyard");
    assert!(perform(&mut g, P0, &opts[0]));
    assert_eq!(g.perm(m).plus, 4);
    assert!(perm_opts(&mut g, m, P0, Some(true)).is_empty(), "already has counters");
}

#[test]
fn lock_and_load_draws_for_each_other_spell_and_can_be_plotted() {
    let mut g = game();
    cast_spell(&mut g, P0, "Opt", Ctx::default());
    cast_spell(&mut g, P0, "Preordain", Ctx::default());
    let n = g.player(P0).hand.len();
    cast_spell(&mut g, P0, "Lock and Load", Ctx::default());
    assert_eq!(g.player(P0).hand.len(), n + 3, "one, plus two other spells");
    // plot it, then cast it free on a later turn
    let mut g = game();
    lands(&mut g, P0, "Island", 4, false);
    let c = hand(&mut g, P0, &["Lock and Load"])[0];
    let opts = hand_opts(&mut g, c, P0, Some(true));
    assert!(perform(&mut g, P0, &opts[0]));
    assert_eq!(sim_core::impls::t0_stella::StellaState::default(), Default::default());
    assert!(g.player(P0).hand.contains(&c), "a plotted card stays marked in the hand");
    assert!(hand_opts(&mut g, c, P0, Some(true)).is_empty(), "not on the turn it was plotted");
    g.player_mut(P0).turns += 1;
    cast_spell(&mut g, P0, "Opt", Ctx::default());
    cast_spell(&mut g, P0, "Preordain", Ctx::default());
    let opts = hand_opts(&mut g, c, P0, Some(false));
    assert_eq!(opts.len(), 1);
    assert!(perform(&mut g, P0, &opts[0]));
    assert!(in_gy(&g, P0, "Lock and Load"));
}

#[test]
fn pyretic_charge_discards_the_hand_draws_four_and_pumps() {
    let mut g = game();
    let t = token(&mut g, P0, 2);
    hand(&mut g, P0, &["Opt", "Ponder", "Preordain"]);
    cast_spell(&mut g, P0, "Pyretic Charge", Ctx::default());
    assert_eq!(g.player(P0).hand.len(), 4);
    assert!(in_gy(&g, P0, "Opt") && in_gy(&g, P0, "Ponder"));
    assert_eq!(values::epow(&g, t), 5, "+1/+0 for each of the three discarded cards");
}

#[test]
fn pyretic_charge_is_plotted_for_four() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 4, false);
    let c = hand(&mut g, P0, &["Pyretic Charge"])[0];
    let opts = hand_opts(&mut g, c, P0, Some(true));
    assert!(perform(&mut g, P0, &opts[0]));
    assert!(g.player(P0).impulse_long.iter().any(|x| x.0 == c));
}

#[test]
fn radical_idea_can_be_jump_started() {
    let mut g = game();
    lands(&mut g, P0, "Island", 2, false);
    let r = take(&mut g, P0, "Radical Idea");
    g.player_mut(P0).gy.push(r);
    hand(&mut g, P0, &["Ponder"]);
    let opts = gy_opts(&mut g, r, P0, Some(true));
    assert_eq!(opts.len(), 1);
    assert!(perform(&mut g, P0, &opts[0]));
    assert!(in_exile(&g, P0, "Radical Idea") && !in_gy(&g, P0, "Radical Idea"), "exiled afterwards");
    assert!(in_gy(&g, P0, "Ponder"), "the discarded card");
    assert_eq!(g.player(P0).hand.len(), 1, "and a card drawn");
}

#[test]
fn radical_idea_needs_a_card_to_discard() {
    let mut g = game();
    lands(&mut g, P0, "Island", 2, false);
    let r = take(&mut g, P0, "Radical Idea");
    g.player_mut(P0).gy.push(r);
    assert!(gy_opts(&mut g, r, P0, Some(true)).is_empty());
}

#[test]
fn shark_typhoon_makes_a_shark_for_each_noncreature_spell_and_cycles() {
    let mut g = game();
    perm(&mut g, P0, "Shark Typhoon");
    cast_spell(&mut g, P0, "Deep Analysis", Ctx::default());
    let s = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token).expect("a Shark");
    assert_eq!((g.perm(s).pow, g.perm(s).tgh, g.perm(s).fly), (4, 4, true), "mana value 4");
    // cycling: X = spare mana
    let mut g = game();
    lands(&mut g, P0, "Island", 6, false);
    g.active = Some(P1);
    let c = hand(&mut g, P0, &["Shark Typhoon"])[0];
    let opts = hand_opts(&mut g, c, P0, None);
    assert_eq!(opts.len(), 1);
    assert!(perform(&mut g, P0, &opts[0]));
    let s = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token).expect("a Shark");
    assert_eq!((g.perm(s).pow, g.perm(s).tgh), (4, 4), "X = 4");
    assert!(g.player(P0).gy.contains(&c));
    assert_eq!(g.player(P0).hand.len(), 1, "drew a card");
}

#[test]
fn smoldering_stagecoach_crews_scales_and_gives_cascade() {
    let mut g = game();
    let v = perm(&mut g, P0, "Smoldering Stagecoach");
    let t = token(&mut g, P0, 2);
    g.perm_mut(t).sick = true;
    fill_gy(&mut g, P0, &["Opt", "Ponder", "Preordain"]);
    let crew = g.registry.get(g.perm(v).cd.unwrap()).and_then(|i| i.crew).unwrap();
    crew(&mut g, v, P0).unwrap();
    assert!(g.is_creature(v) && g.perm(t).tapped, "crewed by the summoning-sick token");
    assert_eq!((values::epow(&g, v), values::etgh(&g, v)), (3, 5), "power: three instants and sorceries");
    let attack = g.registry.get(g.perm(v).cd.unwrap()).and_then(|i| i.attack).unwrap();
    attack(&mut g, v, P0, &[v], P1).unwrap();
    assert!(g.player(P0).t0s.cascade_instant.is_some() && g.player(P0).t0s.cascade_sorcery.is_some());
    // a sorcery with mana value 4 cascades into a cheaper card on top of the library
    let top = take(&mut g, P0, "Preordain");
    g.player_mut(P0).gy.retain(|&c| c != top);
    g.player_mut(P0).library.push(top);
    let da = card(&g, "Deep Analysis");
    cast_spell(&mut g, P0, "Deep Analysis", Ctx::default());
    let _ = da;
    assert!(g.player(P0).t0s.cascade_sorcery.is_none(), "used by the sorcery");
    assert!(g.player(P0).t0s.cascade_instant.is_some(), "the next instant still has it");
    assert!(in_gy(&g, P0, "Preordain") && !g.player(P0).library.contains(&top), "Preordain was cascaded into");
}

#[test]
fn winged_boots_give_flying_and_ward_four() {
    let mut g = game();
    let b = perm(&mut g, P0, "Winged Boots");
    let t = token(&mut g, P0, 2);
    assert!(!sim_core::dsl::has_kw(&g, t, "flying"));
    assert_eq!(sim_core::cardcode::aura_ward(&g, t), 0);
    g.perm_mut(b).attached = Some(t);
    assert!(sim_core::dsl::has_kw(&g, t, "flying"));
    assert_eq!(sim_core::cardcode::aura_ward(&g, t), 4);
}

#[test]
fn the_commander_trace_pieces_are_registered() {
    let g = game();
    for n in ["Stella Lee, Wild Card", "Kaza, Roil Chaser", "Octavia, Living Thesis"] {
        assert!(g.registry.get(card(&g, n)).is_some(), "{n} has card code");
    }
}

// ======================================================== Mizzix's Mastery (casual combos)
#[test]
fn mizzix_s_mastery_is_overloaded_with_guttersnipe_out() {
    let mut g = game();
    g.active = Some(P0);
    lands(&mut g, P0, "Mountain", 8, false);
    perm(&mut g, P0, "Guttersnipe");
    fill_gy(&mut g, P0, &["Opt", "Ponder", "Preordain", "Think Twice", "Serum Visions"]);
    let c = hand(&mut g, P0, &["Mizzix's Mastery"])[0];
    let opts = hand_opts(&mut g, c, P0, Some(false));
    let o = opts.iter().find(|o| o.label.contains("overload")).expect("the overload is offered");
    assert!(opts.iter().all(|x| x.utility <= o.utility), "and preferred to the single copy");
    let life = g.player(P1).life;
    assert!(perform(&mut g, P0, &o.clone()));
    assert!(g.player(P1).life <= life - 2 * 6, "Guttersnipe: the Mastery and five copies");
    assert!(in_exile(&g, P0, "Opt") && in_exile(&g, P0, "Serum Visions"), "the copied cards are exiled");
}

#[test]
fn mizzix_s_mastery_copies_one_spell_without_the_overload_mana() {
    let mut g = game();
    g.active = Some(P0);
    lands(&mut g, P0, "Mountain", 4, false);
    fill_gy(&mut g, P0, &["Treasure Cruise"]);
    let c = hand(&mut g, P0, &["Mizzix's Mastery"])[0];
    let opts = hand_opts(&mut g, c, P0, Some(false));
    assert!(opts.iter().all(|o| !o.label.contains("overload")));
    assert!(
        opts.iter().any(|o| o.label.contains("one spell")),
        "{:?}",
        opts.iter().map(|o| &o.label).collect::<Vec<_>>()
    );
}

#[test]
fn mizzix_s_mastery_is_not_offered_to_veyran_s_own_brain() {
    let mut g = table(&["veyran", "seph"]);
    g.active = Some(P0);
    lands(&mut g, P0, "Mountain", 8, false);
    for n in ["Opt", "Ponder", "Preordain", "Consider"] {
        if let Some(c) = g.player(P0).library.iter().copied().find(|&c| &*g.db.get(c).name == n) {
            g.player_mut(P0).library.retain(|&x| x != c);
            g.player_mut(P0).gy.push(c);
        }
    }
    let c = card(&g, "Mizzix's Mastery");
    g.player_mut(P0).hand.push(c);
    assert!(hand_opts(&mut g, c, P0, Some(false)).is_empty());
}

#[test]
fn galvanic_iteration_is_cast_with_a_spell_to_copy() {
    let mut g = game();
    lands(&mut g, P0, "Island", 7, false);
    let c = hand(&mut g, P0, &["Galvanic Iteration"])[0];
    assert_eq!(sim_core::ai::decks::deck_prio(&g, P0, c), 0, "nothing to copy: held");
    hand(&mut g, P0, &["Deep Analysis"]);
    assert!(sim_core::ai::decks::deck_prio(&g, P0, c) > 46, "before the draw spell it copies");
}
