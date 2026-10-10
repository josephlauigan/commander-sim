//! Veyran's cards and plan on hand-built positions: Python's tests/test_my_cards.py, class Veyran, and the AI's
//! Alania tests of tests/test_play_veyran.py; then tests of the card code ported for this deck (Thousand-Year Storm,
//! the prepared back faces, Ral, the card selection spells, Deflecting Swat ...) and of the plan's choices.
//!
//! Not ported: a person's choices (practice mode: the rest of test_play_veyran.py, Alania's 'you choose').
//! Twinflame and Thunderdrum Soloist are in no decklist now; their tests run anyway (the card code is ported).

use sim_core::ai::{self, act, brain, decks, veyran};
use sim_core::engine::{cast, combat, life, removal, turn, values};
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn lives(g: &Game) -> Vec<i32> {
    g.players.iter().map(|p| p.life).collect()
}

fn named(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| values::card_name(g, m) == Some(name))
}

fn options(g: &mut Game, p: PlayerId, post: bool) -> Vec<Opt> {
    brain::main_options(g, p, post).unwrap()
}

fn find_opt<'a>(opts: &'a [Opt], label: &str) -> &'a Opt {
    opts.iter().find(|o| o.label == label).unwrap_or_else(|| panic!("no option {label:?} in {:?}", labels(opts)))
}

fn labels(opts: &[Opt]) -> Vec<String> {
    opts.iter().map(|o| o.label.clone()).collect()
}

fn take_option(g: &mut Game, p: PlayerId, label: &str) -> bool {
    let opts = options(g, p, false);
    let o = find_opt(&opts, label);
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn magecraft(g: &mut Game, p: PlayerId, name: &str) {
    let c = card(g, name);
    cast::magecraft(g, p, Some(c), false).unwrap();
}

fn on_cast(g: &mut Game, p: PlayerId, name: &str) {
    let c = card(g, name);
    cast::on_cast(g, p, c).unwrap();
}

fn tapped(g: &Game, p: PlayerId) -> usize {
    g.player(p).lands.iter().filter(|&&l| g.land(l).tapped).count()
}

fn tokens(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).token).collect()
}

// ======================================================== Veyran (test_my_cards.Veyran)
#[test]
fn magecraft_pings_each_opponent() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    perm(&mut g, P0, "Guttersnipe");
    magecraft(&mut g, P0, "Lightning Bolt");
    assert_eq!(lives(&g), vec![40, 38, 38]);
}

#[test]
fn veyran_doubles_magecraft() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    perm(&mut g, P0, "Veyran, Voice of Duality");
    perm(&mut g, P0, "Guttersnipe");
    magecraft(&mut g, P0, "Lightning Bolt");
    assert_eq!(lives(&g), vec![40, 36, 36]);
}

#[test]
fn archmage_emeritus_with_veyran_draws_two() {
    let mut g = table(&["veyran", "seph"]);
    perm(&mut g, P0, "Archmage Emeritus");
    perm(&mut g, P0, "Veyran, Voice of Duality");
    magecraft(&mut g, P0, "Think Twice");
    assert_eq!(g.player(P0).hand.len(), 2);
}

#[test]
fn rite_of_the_dragoncaller() {
    let mut g = table(&["veyran", "seph"]);
    perm(&mut g, P0, "Rite of the Dragoncaller");
    magecraft(&mut g, P0, "Lightning Bolt");
    let toks: Vec<(i32, i32, bool)> =
        tokens(&g, P0).iter().map(|&m| (g.perm(m).pow, g.perm(m).tgh, g.perm(m).fly)).collect();
    assert_eq!(toks, vec![(5, 5, true)]);
}

#[test]
fn aetherflux_reservoir() {
    // gain 1 life for each spell you've cast before it this turn
    let mut g = table(&["veyran", "seph"]);
    perm(&mut g, P0, "Aetherflux Reservoir");
    for name in ["Lightning Bolt", "Think Twice", "Counterspell"] {
        g.player_mut(P0).spells_this_turn += 1;
        on_cast(&mut g, P0, name);
    }
    assert_eq!(g.player(P0).life, 46);
}

#[test]
fn jin_gitaxias_copies_the_first_spell() {
    let mut g = table(&["veyran", "seph"]);
    perm(&mut g, P0, "Jin-Gitaxias, Progress Tyrant");
    on_cast(&mut g, P0, "Quick Study"); // the copy resolves: draw two
    assert_eq!(g.player(P0).hand.len(), 2);
    on_cast(&mut g, P0, "Quick Study"); // once each turn
    assert_eq!(g.player(P0).hand.len(), 2);
}

#[test]
fn emeritus_of_ideation_draws_nothing_on_entering() {
    let mut g = table(&["veyran", "seph"]);
    perm(&mut g, P0, "Emeritus of Ideation // Ancestral Recall");
    assert_eq!(g.player(P0).hand, vec![]);
}

fn twinflame(n_lands: usize, creatures: &[&str], spells: &[&str]) -> (Game, Vec<Opt>) {
    let mut g = table(&["veyran", "seph", "sauron"]);
    lands(&mut g, P0, "Mountain", n_lands, false);
    for c in creatures {
        perm(&mut g, P0, c);
    }
    let mut h = vec!["Twinflame"];
    h.extend(spells);
    hand(&mut g, P0, &h);
    let opts: Vec<Opt> = options(&mut g, P0, false).into_iter().filter(|o| o.label.starts_with("Twinflame")).collect();
    (g, opts)
}

const BURN: [&str; 2] = ["Lightning Bolt", "Burst Lightning"];

#[test]
fn twinflame_copies_with_haste_until_the_end_step() {
    let (mut g, opts) = twinflame(4, &["Guttersnipe", "Veyran, Voice of Duality"], &BURN);
    act::perform(&mut g, P0, opts[0].act.as_ref().unwrap()).unwrap();
    let copies = tokens(&g, P0);
    assert_eq!(copies.iter().map(|&m| g.perm(m).name).collect::<Vec<_>>(), vec!["Guttersnipe"]);
    assert!(!g.perm(copies[0]).sick); // haste
    assert_eq!(tapped(&g, P0), 2); // {1}{R}: mana left for the burn spells
    turn::end_step(&mut g, P0).unwrap();
    assert!(tokens(&g, P0).is_empty()); // exiled at the end step
}

#[test]
fn twinflame_strive_cost() {
    let (mut g, opts) = twinflame(8, &["Guttersnipe", "Kessig Flamebreather", "Thunderdrum Soloist"], &BURN);
    let o = find_opt(&opts, "Twinflame (2 targets)");
    act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap();
    assert_eq!(tapped(&g, P0), 5); // {1}{R} + {2}{R}
}

#[test]
fn twinflame_skips_legends_and_idle_copies() {
    let (_, opts) = twinflame(4, &["Veyran, Voice of Duality"], &BURN);
    assert!(opts.is_empty()); // a copy of a legend dies to the legend rule
    let (_, opts) = twinflame(2, &["Guttersnipe"], &BURN);
    assert!(opts.is_empty()); // no mana left to trigger the copy
}

#[test]
fn unsummon_returns_a_creature() {
    let mut g = table(&["veyran", "sauron"]);
    lands(&mut g, P0, "Island", 1, false);
    hand(&mut g, P0, &["Unsummon"]);
    let k = perm(&mut g, P1, "Kaervek the Merciless");
    let s = brain::Situation::new(&g, P0);
    let opts = brain::removal_options(&g, P0, &s);
    let o = find_opt(&opts, "Unsummon -> Kaervek the Merciless");
    act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap();
    assert!(!g.perm(k).on_bf);
    assert!(g.player(P1).hand.contains(&card(&g, "Kaervek the Merciless")));
}

// ======================================================== Alania, Divergent Storm (test_play_veyran.Alania, the AI)
fn bolt(g: &mut Game, target: PlayerId) {
    let c = hand(g, P0, &["Lightning Bolt"])[0];
    cast::cast_card(g, P0, c, "hand", Ctx { face: Some(target), ..Ctx::default() }).unwrap();
}

fn opp_hands(g: &Game) -> usize {
    g.player(P1).hand.len() + g.player(P2).hand.len()
}

#[test]
fn alania_copies_the_first_instant_once() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    perm(&mut g, P0, "Alania, Divergent Storm");
    let before = opp_hands(&g);
    bolt(&mut g, P1);
    assert_eq!(opp_hands(&g), before + 1); // one opponent drew for the copy
    assert_eq!(g.player(P1).life + g.player(P2).life, 80 - 6); // the Bolt and its copy
    let c = hand(&mut g, P0, &["Burst Lightning"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { face: Some(P1), ..Ctx::default() }).unwrap();
    assert_eq!(opp_hands(&g), before + 1); // not the first instant: no copy
}

#[test]
fn alania_veyran_and_harmonic_prodigy_each_add_a_copy() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    for n in ["Alania, Divergent Storm", "Veyran, Voice of Duality", "Harmonic Prodigy"] {
        perm(&mut g, P0, n);
    }
    let before = opp_hands(&g);
    bolt(&mut g, P1);
    assert_eq!(opp_hands(&g), before + 3);
}

#[test]
fn alania_copies_the_first_sorcery_too_and_not_a_counterspell() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    perm(&mut g, P0, "Alania, Divergent Storm");
    let before = opp_hands(&g);
    let c = hand(&mut g, P0, &["Stock Up"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert_eq!(opp_hands(&g), before + 1);
    let ctr = hand(&mut g, P0, &["Counterspell"])[0];
    g.log = Some(vec![]);
    cast::on_cast(&mut g, P0, ctr).unwrap(); // the first instant: a counterspell
    assert!(!g.log.as_ref().unwrap().iter().any(|x| x.contains("Alania")));
}

#[test]
fn alania_counts_a_spell_cast_before_it() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    bolt(&mut g, P1);
    perm(&mut g, P0, "Alania, Divergent Storm");
    let before = opp_hands(&g);
    let c = hand(&mut g, P0, &["Burst Lightning"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { face: Some(P1), ..Ctx::default() }).unwrap();
    assert_eq!(opp_hands(&g), before);
}

// ======================================================== Veyran's card code
#[test]
fn thousand_year_storm_copies_per_earlier_spell() {
    // "Whenever you cast an instant or sorcery spell, copy it for each other instant and sorcery spell you've cast
    //  before it this turn. You may choose new targets for the copies."
    let mut g = table(&["veyran", "seph", "sauron"]);
    perm(&mut g, P0, "Thousand-Year Storm");
    let qs = hand(&mut g, P0, &["Quick Study"])[0];
    cast::cast_card(&mut g, P0, qs, "hand", Ctx::default()).unwrap(); // the first: no copy
    assert_eq!(g.player(P0).hand.len(), 2);
    bolt(&mut g, P1); // the second: one copy
    assert_eq!(g.player(P1).life + g.player(P2).life, 80 - 6);
    // with Veyran, twice as many copies
    let mut g = table(&["veyran", "seph", "sauron"]);
    perm(&mut g, P0, "Thousand-Year Storm");
    perm(&mut g, P0, "Veyran, Voice of Duality");
    let qs = hand(&mut g, P0, &["Quick Study"])[0];
    cast::cast_card(&mut g, P0, qs, "hand", Ctx::default()).unwrap();
    bolt(&mut g, P1);
    assert_eq!(g.player(P1).life + g.player(P2).life, 80 - 9); // the Bolt and two copies
}

#[test]
fn a_prepared_copy_is_a_cast_spell() {
    // Emeritus of Ideation enters prepared: a copy of Ancestral Recall for {U} draws three, with magecraft
    let mut g = table(&["veyran", "seph", "sauron"]);
    lands(&mut g, P0, "Island", 1, false);
    let em = perm(&mut g, P0, "Emeritus of Ideation // Ancestral Recall");
    perm(&mut g, P0, "Guttersnipe");
    assert!(g.perm(em).data.truthy(DataKey::Prepared));
    let opts = options(&mut g, P0, false);
    let o = find_opt(&opts, "Ancestral Recall (prepared copy)");
    assert!((o.utility - (1.2 + 5.0)).abs() < 1e-9); // one payoff out
    assert!(take_option(&mut g, P0, "Ancestral Recall (prepared copy)"));
    assert_eq!(g.player(P0).hand.len(), 3);
    assert_eq!(lives(&g), vec![40, 38, 38]); // Guttersnipe: the copy was cast
    assert!(!g.perm(em).data.truthy(DataKey::Prepared));
    assert_eq!(g.player(P0).spells_this_turn, 1);
}

#[test]
fn thousand_year_storm_copies_a_prepared_copy() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    lands(&mut g, P0, "Island", 1, false);
    perm(&mut g, P0, "Thousand-Year Storm");
    perm(&mut g, P0, "Emeritus of Ideation // Ancestral Recall");
    let qs = hand(&mut g, P0, &["Quick Study"])[0];
    cast::cast_card(&mut g, P0, qs, "hand", Ctx::default()).unwrap(); // a spell before it
    let n = g.player(P0).hand.len();
    assert!(take_option(&mut g, P0, "Ancestral Recall (prepared copy)"));
    assert_eq!(g.player(P0).hand.len(), n + 6); // the copy and Storm's copy of it
}

#[test]
fn emeritus_of_ideation_prepares_again_when_it_attacks() {
    // "Whenever this creature attacks, you may exile eight cards from your graveyard. If you do, it becomes prepared."
    let mut g = table(&["veyran", "seph"]);
    let em = perm(&mut g, P0, "Emeritus of Ideation // Ancestral Recall");
    g.perm_mut(em).data.set(DataKey::Prepared, Val::Bool(false));
    for n in ["Island", "Mountain", "Think Twice", "Deduce", "Quick Study", "Stock Up", "Flow State", "Abrade", "Unsummon"] {
        let c = take(&mut g, P0, n);
        g.player_mut(P0).gy.push(c);
    }
    combat::attack_triggers(&mut g, P0, &[em], P1).unwrap();
    assert!(g.perm(em).data.truthy(DataKey::Prepared));
    assert_eq!(g.player(P0).gy.len(), 1);
    assert_eq!(g.player(P0).exile.len(), 8);
    // lands and permanents go first: an instant or sorcery is what's left
    let left = g.player(P0).gy[0];
    assert!(g.db.get(left).instant || g.db.get(left).sorcery);
}

#[test]
fn ral_storm_conduit_copies_the_next_spell() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    let ral = perm(&mut g, P0, "Ral, Storm Conduit");
    lands(&mut g, P0, "Island", 3, false);
    let qs = hand(&mut g, P0, &["Quick Study"])[0]; // draw two: a copy is worth 3
    let opts = options(&mut g, P0, false);
    assert!(labels(&opts).contains(&"Ral, Storm Conduit +2 (scry 1)".to_string()));
    let o = find_opt(&opts, "Ral, Storm Conduit -2 (copy)");
    assert_eq!(o.utility, 1.5 + 0.5 * (3.0 + 1.0)); // (Ral pings: one payoff out)
    act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap();
    assert_eq!(g.perm(ral).loyalty, Some(2));
    assert_eq!(g.player(P0).ral_copy, Some(g.turn_stamp()));
    assert!(options(&mut g, P0, false).iter().all(|o| !o.label.starts_with("Ral, Storm Conduit"))); // once a turn
    let life = g.player(P1).life + g.player(P2).life;
    cast::cast_card(&mut g, P0, qs, "hand", Ctx::default()).unwrap();
    assert_eq!(g.player(P0).hand.len(), 4); // Quick Study and its copy
    assert_eq!(g.player(P0).ral_copy, None);
    // Ral's magecraft pings the most threatening opponent, for the spell and for its copy
    assert_eq!(g.player(P1).life + g.player(P2).life, life - 2);
}

#[test]
fn expressive_iteration_flow_state_and_stock_up() {
    let mut g = table(&["veyran", "seph"]);
    let top3: Vec<CardId> = g.player(P0).library.iter().rev().take(3).copied().collect();
    let c = hand(&mut g, P0, &["Expressive Iteration"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    let pl = g.player(P0);
    assert_eq!(pl.hand.len(), 2); // one to hand, one exiled and playable this turn
    assert_eq!(pl.impulse.len(), 1);
    assert!(top3.contains(&pl.library[0])); // the last to the bottom
    // Flow State: one card (two with an instant and a sorcery in the graveyard)
    let mut g = table(&["veyran", "seph"]);
    let c = hand(&mut g, P0, &["Flow State"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert_eq!(g.player(P0).hand.len(), 1);
    let mut g = table(&["veyran", "seph"]);
    for n in ["Lightning Bolt", "Light Up the Stage"] {
        let x = take(&mut g, P0, n);
        g.player_mut(P0).gy.push(x);
    }
    let c = hand(&mut g, P0, &["Flow State"])[0];
    let lib = g.player(P0).library.len();
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert_eq!(g.player(P0).hand.len(), 2);
    assert_eq!(g.player(P0).library.len(), lib - 2);
    // Stock Up: two of the top five
    let mut g = table(&["veyran", "seph"]);
    let c = hand(&mut g, P0, &["Stock Up"])[0];
    let lib = g.player(P0).library.len();
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert_eq!(g.player(P0).hand.len(), 2);
    assert_eq!(g.player(P0).library.len(), lib - 2);
}

#[test]
fn prismari_charm_picks_its_mode() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    g.player_mut(P1).life = 1;
    g.player_mut(P2).life = 1;
    let c = hand(&mut g, P0, &["Prismari Charm"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    life::check_state(&mut g).unwrap();
    assert!(!g.player(P1).alive && !g.player(P2).alive); // two players at 1: ping both
    // nothing to ping or bounce: surveil 2 and draw
    let mut g = table(&["veyran", "seph"]);
    let c = hand(&mut g, P0, &["Prismari Charm"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert_eq!(g.player(P0).hand.len(), 1);
}

#[test]
fn visions_of_beyond_draws_three_with_a_big_graveyard() {
    let mut g = table(&["veyran", "seph"]);
    let c = hand(&mut g, P0, &["Visions of Beyond"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert_eq!(g.player(P0).hand.len(), 1);
    let mut g = table(&["veyran", "seph"]);
    let rest = { let lib = &mut g.player_mut(P1).library; lib.split_off(lib.len() - 20) };
    g.player_mut(P1).gy.extend(rest);
    let c = hand(&mut g, P0, &["Visions of Beyond"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert_eq!(g.player(P0).hand.len(), 3);
}

#[test]
fn deflecting_swat_turns_removal_back_on_its_caster() {
    // "If you control a commander, you may cast this spell without paying its mana cost. You may choose new targets
    //  for target spell or ability."
    let mut g = table(&["veyran", "seph"]);
    let archon = perm(&mut g, P1, "Archon of Cruelty"); // (first: its enter trigger would take Veyran)
    let vey = perm(&mut g, P0, "Veyran, Voice of Duality");
    g.perm_mut(vey).is_cmd = true;
    g.player_mut(P0).cmd_in_zone = false;
    hand(&mut g, P0, &["Deflecting Swat"]);
    let stp = card(&g, "Swords to Plowshares");
    removal::apply_removal(&mut g, Some(P1), vey, "exile", Some(stp)).unwrap();
    assert!(g.perm(vey).on_bf);
    assert!(!g.perm(archon).on_bf); // the Swords hit the caster's own Archon
    assert!(g.player(P0).gy.contains(&card(&g, "Deflecting Swat")));
}

#[test]
fn slip_out_the_back_saves_a_key_creature() {
    let mut g = table(&["veyran", "seph"]);
    lands(&mut g, P0, "Island", 1, false);
    let vey = perm(&mut g, P0, "Veyran, Voice of Duality");
    hand(&mut g, P0, &["Slip Out the Back"]);
    let stp = card(&g, "Swords to Plowshares");
    assert!(ai::protect_response(&mut g, P0, vey, "exile", Some(P1), Some(stp)).unwrap());
    assert!(g.perm(vey).phased);
    // a wipe isn't targeted: no answer
    let mut g = table(&["veyran", "seph"]);
    lands(&mut g, P0, "Island", 1, false);
    let vey = perm(&mut g, P0, "Veyran, Voice of Duality");
    hand(&mut g, P0, &["Slip Out the Back"]);
    assert!(!ai::protect_response(&mut g, P0, vey, "wipe", Some(P1), Some(stp)).unwrap());
}

#[test]
fn desolate_lighthouse_loots_at_the_end_of_an_opponents_turn() {
    let mut g = table(&["veyran", "seph"]);
    lands(&mut g, P0, "Desolate Lighthouse", 1, false);
    lands(&mut g, P0, "Steam Vents", 3, false);
    hand(&mut g, P0, &["Island"]);
    assert!(sim_core::cardcode::land_options(&mut g, P0, Some(false)).unwrap().is_empty()); // your main phase: no
    g.active = Some(P1);
    let opts = sim_core::cardcode::land_options(&mut g, P0, None).unwrap();
    let o = find_opt(&opts, "Desolate Lighthouse loot");
    assert!(act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P0).hand.len(), 1); // drew one, discarded one
    assert_eq!(g.player(P0).gy.len(), 1);
    assert_eq!(tapped(&g, P0), 4);
}

#[test]
fn docent_of_perfection_transforms_with_three_wizards() {
    // "Whenever you cast an instant or sorcery spell, create a 1/1 blue Human Wizard creature token. Then if you
    //  control three or more Wizards, transform Docent of Perfection." Final Iteration: Wizards you control get +2/+1
    //  and have flying.
    let mut g = table(&["veyran", "seph"]);
    perm(&mut g, P0, "Docent of Perfection // Final Iteration");
    let emer = perm(&mut g, P0, "Emeritus of Conflict // Lightning Bolt"); // a Human Wizard, 2/2
    assert_eq!(values::epow(&g, emer), 2);
    for _ in 0..2 {
        on_cast(&mut g, P0, "Think Twice");
    }
    life::check_state(&mut g).unwrap();
    assert_eq!(tokens(&g, P0).len(), 2);
    assert_eq!((values::epow(&g, emer), values::etgh(&g, emer)), (4, 3));
    assert!(combat::kw(&g, emer, "flying"));
    let d = named(&g, P0, "Docent of Perfection // Final Iteration").unwrap();
    assert!(g.perm(d).data.truthy(DataKey::Final));
    // kept once transformed: the Wizards dying doesn't turn it back
    for t in tokens(&g, P0) {
        sim_core::engine::zones::die(&mut g, t, "destroy").unwrap();
    }
    assert_eq!(values::epow(&g, emer), 4);
}

#[test]
fn fiery_emancipation_triples_damage() {
    let mut g = table(&["veyran", "seph"]);
    perm(&mut g, P0, "Fiery Emancipation");
    perm(&mut g, P0, "Guttersnipe");
    magecraft(&mut g, P0, "Lightning Bolt");
    assert_eq!(g.player(P1).life, 40 - 6);
}

// ======================================================== the plan's choices
#[test]
fn veyran_priorities() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    let prio = |g: &Game, n: &str| veyran::veyran_prio(g, P0, card(g, n));
    assert_eq!(prio(&g, "Veyran, Voice of Duality"), 70);
    assert_eq!(prio(&g, "Displacer Kitten"), 74);
    assert_eq!(prio(&g, "Guttersnipe"), 72);
    assert_eq!(prio(&g, "Thousand-Year Storm"), 57);
    assert_eq!(prio(&g, "Fiery Emancipation"), 40);
    perm(&mut g, P0, "Guttersnipe");
    assert_eq!(prio(&g, "Fiery Emancipation"), 64); // with a pinger out
    assert_eq!(prio(&g, "Propaganda"), 52); // no opposing creatures yet
    perm(&mut g, P1, "Grave Titan");
    assert_eq!(prio(&g, "Propaganda"), 52 + 10); // a 6/6 and two 2/2 Zombies
    assert_eq!(decks::deck_prio(&g, P0, card(&g, "Guttersnipe")), 72);
}

#[test]
fn veyran_goes_for_the_kitten_and_firesinger_combo() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    lands(&mut g, P0, "Mountain", 3, false);
    perm(&mut g, P0, "Displacer Kitten");
    perm(&mut g, P0, "Blazing Firesinger // Seething Song");
    let opts = options(&mut g, P0, false);
    assert!(!labels(&opts).contains(&"go for the combo".to_string())); // no payoff yet
    perm(&mut g, P0, "Guttersnipe");
    let opts = options(&mut g, P0, false);
    let o = find_opt(&opts, "go for the combo");
    assert!(o.utility > 4.0);
    assert!(veyran::veyran_try_combo(&mut g, P0).unwrap());
    assert!(g.over && g.winner == Some(P0));
}

#[test]
fn veyran_fires_aetherflux_at_51_life() {
    let mut g = table(&["veyran", "seph", "sauron"]);
    perm(&mut g, P0, "Aetherflux Reservoir");
    g.player_mut(P0).life = 51;
    assert!(labels(&options(&mut g, P0, false)).contains(&"Aetherflux shot".to_string()));
    assert!(veyran::aether_check(&mut g, P0).unwrap());
    assert_eq!(g.player(P0).life, 1);
    assert_eq!(g.player(P1).life.min(g.player(P2).life), -10);
}

#[test]
fn veyran_wishes_for_the_combo_half_it_lacks() {
    let mut g = table(&["veyran", "seph"]);
    perm(&mut g, P0, "Blazing Firesinger // Seething Song");
    // Displacer Kitten isn't in Veyran's list now: nothing to fetch for the combo, so the next creature of the list
    let pick = decks::tutor_pick(&g, P0, "cre2").map(|c| g.db.get(c).name.to_string());
    assert!(pick.is_some());
    assert_ne!(pick.as_deref(), Some("Displacer Kitten"));
}

#[test]
fn veyran_engine_payoff_milestone() {
    let mut g = table(&["veyran", "seph"]);
    assert!(!ai::engine_payoff(&g, P0));
    perm(&mut g, P0, "Murmuring Mystic");
    assert!(ai::engine_payoff(&g, P0));
    assert!(!veyran::payoff(&g, P0));
}

#[test]
fn whole_games_with_veyran_finish() {
    for seed in [1, 2, 3] {
        let g = play(&["veyran", "seph", "sauron"], seed, false);
        assert!(g.over || g.round >= 20, "seed {seed}");
    }
}
