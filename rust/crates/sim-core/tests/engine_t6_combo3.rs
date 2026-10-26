//! Engine feature `combo3`: the declared steps of Thassa's Oracle + Consultation / Pact and Kinnan + Basalt Monolith, the
//! missing cEDH lines (Breach loop, Ad Nauseam, Hullbreaker Horror, the Kinnan outlets) and the Godo leftovers
//! (impls/t6_combos_extra.rs). Seat 0 plays the combo, seat 1 holds the answers.

use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::combos;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl(keys: &[&str]) -> Game {
    let mut g = table(keys);
    g.active = Some(P0);
    g
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn go(g: &mut Game, part: &str) -> bool {
    let i = combos::combo_index(part).unwrap_or_else(|| panic!("no combo {part}"));
    assert!(combos::ready(g, P0, i), "{part} is ready");
    combos::attempt(g, P0, i as i64).unwrap()
}

fn not_ready(g: &Game, part: &str) -> bool {
    let i = combos::combo_index(part).unwrap();
    !combos::ready(g, P0, i)
}

fn blue_black(g: &mut Game, p: PlayerId) {
    lands(g, p, "Island", 2, false);
    lands(g, p, "Swamp", 2, false);
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

// ============================================================ Thassa's Oracle + Consultation / Pact
const ORACLE: &str = "Oracle + Demonic Consultation";

fn oracle_table(lib: &[&str]) -> Game {
    let mut g = tbl(&["kraum-tymna-blue-breach-midrange", "seph"]);
    blue_black(&mut g, P0);
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation"]);
    let lib: Vec<_> = lib.iter().map(|n| card(&g, n)).collect();
    g.player_mut(P0).library = lib;
    g
}

#[test]
fn oracle_with_consultation_has_three_declared_steps_and_wins() {
    let i = combos::combo_index(ORACLE).unwrap();
    assert_eq!(combos::all()[i].steps.len(), 3);
    let mut g = oracle_table(&["Sol Ring", "Counterspell", "Brainstorm"]);
    go(&mut g, ORACLE);
    assert_eq!(g.winner, Some(P0));
    assert!(g.player(P0).library.is_empty(), "Consultation exiled the library");
}

#[test]
fn a_tidebinder_on_the_oracle_trigger_stops_the_line_after_the_library_is_gone() {
    let mut g = oracle_table(&["Sol Ring", "Counterspell", "Brainstorm"]);
    blue_black(&mut g, P1);
    hand(&mut g, P1, &["Tishana's Tidebinder"]);
    go(&mut g, ORACLE);
    assert_eq!(g.winner, None, "the trigger was countered");
    assert!(on_bf(&g, P0, "Thassa's Oracle").is_some());
    assert!(g.player(P0).library.is_empty());
}

#[test]
fn a_counterspell_on_consultation_leaves_the_library_alone() {
    let mut countered = 0;
    for seed in 0..16u64 {
        let mut g = table_with(&["kraum-tymna-blue-breach-midrange", "seph"], seed, 40);
        g.active = Some(P0);
        blue_black(&mut g, P0);
        hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation"]);
        let lib: Vec<_> = ["Sol Ring", "Brainstorm"].iter().map(|n| card(&g, n)).collect();
        g.player_mut(P0).library = lib;
        lands(&mut g, P1, "Island", 2, false);
        hand(&mut g, P1, &["Counterspell"]);
        go(&mut g, ORACLE);
        if g.winner.is_none() {
            countered += 1;
            assert_eq!(g.player(P0).library.len(), 2, "a counter stopped the line before the exile or on the Oracle");
        }
    }
    assert!(countered >= 1, "the AI counters a combo piece sometimes");
}

// the AI checks a line against what the engine charges (taxes included), not the printed costs: Oracle + Consultation
// is {U}{U} + {B} printed, and a tax makes it 4, so on three lands it waits instead of stopping half-cast
fn taxed_oracle(tax: &str) -> Game {
    let mut g = tbl(&["kraum-tymna-blue-breach-midrange", "seph"]);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Swamp", 1, false);
    hand(&mut g, P0, &["Thassa's Oracle", "Demonic Consultation"]);
    let lib: Vec<_> = ["Sol Ring", "Brainstorm"].iter().map(|n| card(&g, n)).collect();
    g.player_mut(P0).library = lib;
    perm(&mut g, P1, tax);
    g
}

fn oracle_waits_then_goes(tax: &str) {
    let i = combos::combo_index(ORACLE).unwrap() as i64;
    let mut g = taxed_oracle(tax);
    assert!(!combos::attempt(&mut g, P0, i).unwrap(), "{tax}: the line isn't started on three lands");
    assert_eq!(g.player(P0).hand.len(), 2, "{tax}: both pieces still in hand");
    assert_eq!(g.player(P0).library.len(), 2, "{tax}: the library is untouched");
    lands(&mut g, P0, "Swamp", 1, false);
    combos::attempt(&mut g, P0, i).unwrap();
    assert_eq!(g.winner, Some(P0), "{tax}: a fourth land pays the tax and the line wins");
}

#[test]
fn damping_sphere_taxes_the_second_piece_of_the_oracle_line() {
    oracle_waits_then_goes("Damping Sphere");
}

#[test]
fn thorn_of_amethyst_taxes_consultation_in_the_oracle_line() {
    oracle_waits_then_goes("Thorn of Amethyst");
}

// ============================================================ Kinnan + Basalt Monolith and its outlets
const KB: &str = "Kinnan + Basalt Monolith";

fn kinnan_table() -> Game {
    let mut g = tbl(&["kinnan-bonder-mana-combo", "seph"]);
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    perm(&mut g, P0, "Basalt Monolith");
    g
}

#[test]
fn kinnan_basalt_has_steps_and_wins_with_the_outlet_paid() {
    let i = combos::combo_index(KB).unwrap();
    assert_eq!(combos::all()[i].steps.len(), 2);
    let mut g = kinnan_table();
    lands(&mut g, P0, "Island", 1, false);
    lands(&mut g, P0, "Forest", 1, false);
    // unbounded colourless mana is no win by itself (#100): it needs a mana sink, here Urza
    assert!(not_ready(&g, KB), "no mana sink");
    perm(&mut g, P0, "Urza, Lord High Artificer");
    go(&mut g, KB);
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn a_guildmage_on_kinnans_activation_stops_the_loop() {
    let mut g = kinnan_table();
    perm(&mut g, P0, "Urza, Lord High Artificer");
    lands(&mut g, P0, "Island", 1, false);
    lands(&mut g, P0, "Forest", 1, false);
    lands(&mut g, P1, "Island", 3, false);
    perm(&mut g, P1, "Azorius Guildmage");
    go(&mut g, KB);
    assert_eq!(g.winner, None, "the outlet was countered");
}

#[test]
fn thrasios_draws_the_deck_when_kinnans_ability_cant_be_paid() {
    let mut g = kinnan_table();
    assert!(not_ready(&g, "Thrasios (draw"), "no Thrasios");
    perm(&mut g, P0, "Thrasios, Triton Hero");
    go(&mut g, "Thrasios (draw");
    assert_eq!(g.winner, Some(P0));
    let mut g = kinnan_table();
    perm(&mut g, P0, "Thrasios, Triton Hero");
    lands(&mut g, P1, "Island", 3, false);
    perm(&mut g, P1, "Azorius Guildmage");
    go(&mut g, "Thrasios (draw");
    assert_eq!(g.winner, None, "Thrasios's ability was countered");
}

#[test]
fn mirage_mirror_turns_the_loop_into_blue_green() {
    let mut g = kinnan_table();
    perm(&mut g, P0, "Mirage Mirror");
    go(&mut g, "Mirage Mirror");
    assert_eq!(g.winner, Some(P0));
    let mut g = kinnan_table();
    perm(&mut g, P0, "Mirage Mirror");
    lands(&mut g, P1, "Island", 3, false);
    perm(&mut g, P1, "Azorius Guildmage");
    go(&mut g, "Mirage Mirror");
    assert_eq!(g.winner, None, "the Mirror's ability was countered");
}

// ============================================================ Underworld Breach + LED + Brain Freeze
const BREACH_O: &str = "Brain Freeze (Thassa";
const BREACH_T: &str = "Brain Freeze (at the table)";

fn breach_table(oracle: bool) -> Game {
    let mut g = tbl(&["kraum-tymna-blue-breach-midrange", "seph"]);
    lands(&mut g, P0, "Mountain", 1, false);
    lands(&mut g, P0, "Island", 1, false);
    hand(&mut g, P0, &["Underworld Breach", "Lion's Eye Diamond", "Brain Freeze", "Brainstorm", "Ponder", "Preordain"]);
    if oracle {
        hand(&mut g, P0, &["Thassa's Oracle"]);
    }
    g
}

#[test]
fn the_breach_loop_empties_the_library_and_escapes_the_oracle() {
    let mut g = breach_table(true);
    assert!(not_ready(&g, BREACH_T), "the Oracle is in hand: that line is the other one");
    go(&mut g, BREACH_O);
    assert_eq!(g.winner, Some(P0), "an escaped Thassa's Oracle on an empty library");
    assert!(g.player(P0).library.is_empty());
    assert_eq!(g.wintype, Some("combo"));
}

#[test]
fn the_breach_loop_without_the_oracle_brain_freezes_the_table() {
    let mut g = breach_table(false);
    assert!(not_ready(&g, BREACH_O));
    go(&mut g, BREACH_T);
    assert_eq!(g.winner, Some(P0));
    assert!(g.player(P1).library.is_empty(), "every copy past the fuel milled the opponent");
}

#[test]
fn the_breach_loop_needs_fuel_and_red() {
    let mut g = tbl(&["kraum-tymna-blue-breach-midrange", "seph"]);
    lands(&mut g, P0, "Island", 2, false);
    hand(&mut g, P0, &["Underworld Breach", "Lion's Eye Diamond", "Brain Freeze", "Brainstorm", "Ponder", "Preordain"]);
    assert!(not_ready(&g, BREACH_T), "no {{R}} for Breach");
    let mut g = breach_table(false);
    let db = g.db.clone();
    g.player_mut(P0).hand.retain(|&c| !["Brainstorm", "Ponder", "Preordain"].contains(&&*db.get(c).name));
    assert!(not_ready(&g, BREACH_T), "three cards to exile are needed to escape");
}

#[test]
fn a_counterspell_on_a_loop_spell_stops_the_breach_line() {
    let mut stopped = 0;
    for seed in 0..16u64 {
        let mut g = table_with(&["kraum-tymna-blue-breach-midrange", "seph"], seed, 40);
        g.active = Some(P0);
        lands(&mut g, P0, "Mountain", 1, false);
        lands(&mut g, P0, "Island", 1, false);
        hand(
            &mut g,
            P0,
            &[
                "Underworld Breach",
                "Lion's Eye Diamond",
                "Brain Freeze",
                "Brainstorm",
                "Ponder",
                "Preordain",
                "Thassa's Oracle",
            ],
        );
        lands(&mut g, P1, "Island", 2, false);
        hand(&mut g, P1, &["Counterspell"]);
        go(&mut g, BREACH_O);
        if g.winner.is_none() {
            stopped += 1;
            assert!(!g.player(P0).library.is_empty() || on_bf(&g, P0, "Thassa's Oracle").is_none());
        }
    }
    assert!(stopped >= 1, "the AI counters a loop spell sometimes");
}

// ============================================================ Ad Nauseam
const AD: &str = "Ad Nauseam into a win";

fn ad_table(top: &[&str]) -> Game {
    ad_table_with(top, &[])
}

fn ad_table_with(top: &[&str], bottom: &[&str]) -> Game {
    let mut g = tbl(&["kraum-tymna-blue-breach-midrange", "seph"]);
    lands(&mut g, P0, "Swamp", 3, false);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Mountain", 3, false);
    lands(&mut g, P0, "Island", 2, false);
    hand(&mut g, P0, &["Ad Nauseam"]);
    let tops: Vec<_> = top.iter().map(|n| take(&mut g, P0, n)).collect();
    g.player_mut(P0).library.extend(tops); // the last one is the top
    let bots: Vec<_> = bottom.iter().map(|n| take(&mut g, P0, n)).collect();
    g.player_mut(P0).library.splice(0..0, bots);
    g
}

#[test]
fn ad_nauseam_draws_into_the_oracle_line_and_wins() {
    let mut g = ad_table(&["Thassa's Oracle", "Demonic Consultation"]);
    go(&mut g, AD);
    assert_eq!(g.winner, Some(P0));
    assert!(g.player(P0).life < 40, "Ad Nauseam cost life");
}

#[test]
fn ad_nauseam_digs_past_a_full_hand_in_the_win_attempt() {
    // a traced game: eight cards in hand, the hand cap stopped Ad Nauseam at 0 cards and the line stopped
    // under three lands: five cards to dig, past the cap of 10 with eight cards in hand
    let mut g = ad_table(&["Thassa's Oracle", "Demonic Consultation", "Plains", "Plains", "Plains"]);
    let full = ["Brainstorm", "Opt", "Ponder", "Preordain", "Gitaxian Probe", "Mental Misstep", "Swan Song", "Dispel"];
    hand(&mut g, P0, &full);
    go(&mut g, AD);
    assert_eq!(g.winner, Some(P0), "the win attempt digs for the line whatever the hand size");
}

#[test]
fn ad_nauseam_that_finds_nothing_ends_without_a_win() {
    let mut g = ad_table_with(&[], &["Thassa's Oracle", "Demonic Consultation"]); // at the very bottom: out of reach
    go(&mut g, AD);
    assert_eq!(g.winner, None);
    assert!(in_gy(&g, P0, "Ad Nauseam"));
}

#[test]
fn ad_nauseam_needs_life_and_a_line_in_the_deck() {
    let mut g = ad_table(&["Thassa's Oracle", "Demonic Consultation"]);
    g.player_mut(P0).life = 25;
    assert!(not_ready(&g, AD), "above 25 life");
}

// ============================================================ Hullbreaker Horror
const HORROR: &str = "Hullbreaker Horror + Kinnan";

fn horror_table() -> Game {
    let mut g = tbl(&["kinnan-bonder-mana-combo", "seph"]);
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    perm(&mut g, P0, "Hullbreaker Horror");
    perm(&mut g, P0, "Sol Ring");
    hand(&mut g, P0, &["Mox Amber"]);
    g
}

#[test]
fn horror_with_sol_ring_out_and_mox_amber_in_hand_loops() {
    let mut g = horror_table();
    go(&mut g, HORROR);
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn horror_can_be_flashed_in_and_lotus_petal_is_a_partner() {
    let mut g = tbl(&["kinnan-bonder-mana-combo", "seph"]);
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    perm(&mut g, P0, "Mox Amber");
    hand(&mut g, P0, &["Hullbreaker Horror", "Lotus Petal"]);
    lands(&mut g, P0, "Island", 7, false);
    go(&mut g, HORROR);
    assert_eq!(g.winner, Some(P0));
    assert!(on_bf(&g, P0, "Hullbreaker Horror").is_some());
    let mut g = tbl(&["kinnan-bonder-mana-combo", "seph"]);
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    perm(&mut g, P0, "Sol Ring");
    hand(&mut g, P0, &["Hullbreaker Horror", "Mox Amber"]);
    lands(&mut g, P0, "Island", 2, false);
    assert!(not_ready(&g, HORROR), "Horror costs seven; two lands and a Kinnan-doubled Sol Ring make six");
}

#[test]
fn a_tidebinder_on_the_horror_trigger_stops_the_loop() {
    let mut g = horror_table();
    blue_black(&mut g, P1);
    hand(&mut g, P1, &["Tishana's Tidebinder"]);
    go(&mut g, HORROR);
    assert_eq!(g.winner, None);
}

#[test]
fn two_rocks_that_dont_pair_are_no_loop() {
    let mut g = tbl(&["kinnan-bonder-mana-combo", "seph"]);
    perm(&mut g, P0, "Kinnan, Bonder Prodigy");
    perm(&mut g, P0, "Hullbreaker Horror");
    perm(&mut g, P0, "Lotus Petal");
    hand(&mut g, P0, &["Fellwar Stone"]);
    assert!(not_ready(&g, HORROR), "Petal and Stone are both in the second list");
}

// ============================================================ Godo: Welder + Engineer, the rebuild, Basalt + a copied Kinnan
fn godo_tbl() -> Game {
    tbl(&["godo", "seph"])
}

fn mountains(g: &mut Game, n: usize) {
    lands(g, P0, "Mountain", n, false);
}

fn worn_by_godo(g: &Game) -> bool {
    let (Some(h), Some(_)) = (on_bf(g, P0, "Helm of the Host"), on_bf(g, P0, "Godo, Bandit Warlord")) else {
        return false;
    };
    g.perm(h).attached.is_some_and(|t| &*g.perm(t).name == "Godo, Bandit Warlord")
}

const WELDER: &str = "Goblin Welder + Goblin Engineer";

fn welder_table() -> Game {
    let mut g = godo_tbl();
    let w = perm(&mut g, P0, "Goblin Welder");
    g.perm_mut(w).sick = false;
    let rock = perm(&mut g, P0, "Sol Ring");
    g.perm_mut(rock).tapped = true; // a spent rock to give up
    hand(&mut g, P0, &["Goblin Engineer"]);
    mountains(&mut g, 7);
    g
}

#[test]
fn welder_and_engineer_put_hammer_and_helm_out_then_godo_wins() {
    let mut g = welder_table();
    assert!(not_ready(&g, WELDER), "Engineer {{1}}{{R}} and Godo {{5}}{{R}} are eight");
    mountains(&mut g, 1);
    go(&mut g, WELDER);
    assert_eq!(g.winner, Some(P0));
    assert!(worn_by_godo(&g), "Godo fetched Hammer, the Welder returned Helm, Hammer attached it");
    assert!(on_bf(&g, P0, "Hammer of Nazahn").is_some());
    assert!(on_bf(&g, P0, "Sol Ring").is_none(), "the spent rock was given up");
}

#[test]
fn welder_line_needs_the_welder_untapped_and_a_spare_artifact() {
    let mut g = welder_table();
    mountains(&mut g, 1);
    let w = on_bf(&g, P0, "Goblin Welder").unwrap();
    g.perm_mut(w).tapped = true;
    assert!(not_ready(&g, WELDER));
    let mut g = welder_table();
    mountains(&mut g, 1);
    let r = on_bf(&g, P0, "Sol Ring").unwrap();
    sim_core::engine::zones::die(&mut g, r, "sac").unwrap();
    assert!(not_ready(&g, WELDER), "nothing to sacrifice");
}

#[test]
fn a_tidebinder_on_an_enter_trigger_stops_the_welder_line() {
    let mut g = welder_table();
    mountains(&mut g, 1);
    blue_black(&mut g, P1);
    hand(&mut g, P1, &["Tishana's Tidebinder"]);
    go(&mut g, WELDER);
    assert_eq!(g.winner, None);
    assert!(on_bf(&g, P0, "Goblin Engineer").is_some());
    assert!(on_bf(&g, P0, "Helm of the Host").is_none(), "an enter trigger was countered: no Helm came out");
}

#[test]
fn a_guildmage_on_the_welders_ability_stops_the_line_after_godo() {
    let mut g = welder_table();
    mountains(&mut g, 1);
    lands(&mut g, P1, "Island", 3, false);
    perm(&mut g, P1, "Azorius Guildmage");
    go(&mut g, WELDER);
    assert_eq!(g.winner, None);
    assert!(on_bf(&g, P0, "Godo, Bandit Warlord").is_some());
    assert!(on_bf(&g, P0, "Helm of the Host").is_none(), "the swap was countered");
}

// ---- the rebuild with Godo already out
/// Godo and Hammer out, Helm back in the library (the first trigger was answered)
fn rebuild_table() -> Game {
    let mut g = godo_tbl();
    perm(&mut g, P0, "Hammer of Nazahn");
    perm(&mut g, P0, "Godo, Bandit Warlord"); // his enter trigger fetches Helm ...
    let helm = on_bf(&g, P0, "Helm of the Host").unwrap();
    sim_core::engine::zones::die(&mut g, helm, "sac").unwrap(); // ... which is put back
    let c = g.player(P0).gy.iter().copied().find(|&c| &*g.db.get(c).name == "Helm of the Host").unwrap();
    g.player_mut(P0).gy.retain(|&x| x != c);
    g.player_mut(P0).library.push(c);
    g
}

#[test]
fn rebuild_with_cursed_mirror_or_sculpting_steel() {
    let mut g = rebuild_table();
    hand(&mut g, P0, &["Cursed Mirror"]);
    mountains(&mut g, 3);
    go(&mut g, "Godo already out, Hammer out: Cursed Mirror");
    assert_eq!(g.winner, Some(P0));
    assert!(worn_by_godo(&g));
    let mut g = rebuild_table();
    perm(&mut g, P0, "Cursed Mirror");
    hand(&mut g, P0, &["Sculpting Steel"]);
    mountains(&mut g, 3);
    go(&mut g, "Godo already out, Hammer out: Cursed Mirror");
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn rebuild_with_jaxis_and_with_a_copy_spell() {
    let mut g = rebuild_table();
    hand(&mut g, P0, &["Jaxis, the Troublemaker"]);
    mountains(&mut g, 3);
    go(&mut g, "Jaxis blitzes a copy of Godo");
    assert_eq!(g.winner, Some(P0));
    let mut g = rebuild_table();
    hand(&mut g, P0, &["Twinflame"]);
    mountains(&mut g, 2);
    go(&mut g, "a token copy spell on Godo");
    assert_eq!(g.winner, Some(P0));
    assert!(worn_by_godo(&g));
}

#[test]
fn the_rebuild_needs_godo_out_and_hammer_out() {
    let mut g = godo_tbl();
    hand(&mut g, P0, &["Cursed Mirror"]);
    mountains(&mut g, 3);
    assert!(not_ready(&g, "Godo already out, Hammer out: Cursed Mirror"), "no Godo on the battlefield");
}

#[test]
fn ertai_on_the_copys_enter_trigger_stops_the_rebuild() {
    let mut g = rebuild_table();
    hand(&mut g, P0, &["Cursed Mirror"]);
    mountains(&mut g, 3);
    blue_black(&mut g, P1);
    hand(&mut g, P1, &["Tishana's Tidebinder"]);
    go(&mut g, "Godo already out, Hammer out: Cursed Mirror");
    assert_eq!(g.winner, None);
    assert!(on_bf(&g, P0, "Helm of the Host").is_none());
}

// ---- Basalt Monolith + a copied Kinnan
const KCOPY: &str = "Godo + Basalt Monolith + a copied Kinnan";

fn kinnan_copy_table() -> Game {
    let mut g = tbl(&["godo", "kinnan-bonder-mana-combo"]);
    perm(&mut g, P0, "Basalt Monolith");
    perm(&mut g, P1, "Kinnan, Bonder Prodigy");
    hand(&mut g, P0, &["Heat Shimmer"]);
    mountains(&mut g, 3);
    g
}

#[test]
fn heat_shimmer_on_an_opposing_kinnan_with_basalt_funds_godo() {
    let mut g = kinnan_copy_table();
    go(&mut g, KCOPY);
    assert_eq!(g.winner, Some(P0));
    assert!(worn_by_godo(&g), "equipped from the loop's mana");
}

#[test]
fn the_kinnan_copy_needs_a_kinnan_across_the_table_and_a_copier() {
    let mut g = kinnan_copy_table();
    let k = on_bf(&g, P1, "Kinnan, Bonder Prodigy").unwrap();
    sim_core::engine::zones::die(&mut g, k, "sac").unwrap();
    assert!(not_ready(&g, KCOPY));
    let mut g = kinnan_copy_table();
    g.player_mut(P0).hand.clear();
    assert!(not_ready(&g, KCOPY));
}

#[test]
fn cursed_mirror_also_copies_the_kinnan() {
    let mut g = kinnan_copy_table();
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Cursed Mirror"]);
    go(&mut g, KCOPY);
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn removal_on_the_kinnan_copy_stops_the_loop() {
    let mut stopped = 0;
    for seed in 0..8u64 {
        let mut g = table_with(&["godo", "kinnan-bonder-mana-combo"], seed, 40);
        g.active = Some(P0);
        perm(&mut g, P0, "Basalt Monolith");
        perm(&mut g, P1, "Kinnan, Bonder Prodigy");
        hand(&mut g, P0, &["Heat Shimmer"]);
        mountains(&mut g, 3);
        lands(&mut g, P1, "Plains", 1, false);
        hand(&mut g, P1, &["Swords to Plowshares"]);
        go(&mut g, KCOPY);
        if g.winner.is_none() {
            stopped += 1;
        }
    }
    assert!(stopped >= 5, "instant-speed removal on the copy stops the line ({stopped}/8)");
}

#[test]
fn a_counterspell_on_heat_shimmer_stops_the_kinnan_copy() {
    let mut countered = 0;
    for seed in 0..16u64 {
        let mut g = table_with(&["godo", "kinnan-bonder-mana-combo"], seed, 40);
        g.active = Some(P0);
        perm(&mut g, P0, "Basalt Monolith");
        perm(&mut g, P1, "Kinnan, Bonder Prodigy");
        hand(&mut g, P0, &["Heat Shimmer"]);
        mountains(&mut g, 3);
        lands(&mut g, P1, "Island", 2, false);
        hand(&mut g, P1, &["Counterspell"]);
        go(&mut g, KCOPY);
        if g.winner.is_none() {
            countered += 1;
            assert!(on_bf(&g, P0, "Godo, Bandit Warlord").is_none());
        }
    }
    assert!(countered >= 1);
}

// ============================================================ the new lines register no pieces
#[test]
fn the_new_lines_are_quiet_so_other_decks_do_not_change() {
    for n in [
        "Underworld Breach",
        "Lion's Eye Diamond",
        "Brain Freeze",
        "Ad Nauseam",
        "Sol Ring",
        "Mox Amber",
        "Goblin Welder",
        "Goblin Engineer",
        "Hullbreaker Horror",
        "Mirage Mirror",
        "Thrasios, Triton Hero",
    ] {
        assert!(!combos::is_piece_name(n), "{n} is no combo piece");
    }
    assert!(combos::is_piece_name("Thassa's Oracle"), "the base combos keep their pieces");
    assert!(combos::is_piece_name("Kinnan, Bonder Prodigy"));
}
