//! The AI's {X} spells are priced by `mana::cost_of_x`, not by `cost_of` plus "all the spare mana": a generic reducer
//! (a Medallion, Etherium Sculptor) takes its {1} off the generic part that includes X, so the AI casts with a bigger X
//! for the same mana; a tax comes after it; K'rrik's life is charged for the {B} the mana leaves unpaid.
//!
//! Each site: a position with N lands, the card in hand, the X the AI chooses and the mana it spends, with no reducer
//! (the old result, unchanged) and with one. The helpers `brain::max_x` / `brain::pay_x` are the shared search and
//! payment; `cost_of_x` itself is tested in tests/cost_of_x.rs.

use sim_core::ai::brain::{self, Situation};
use sim_core::ai::{act, pool};
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

/// Y'shtola's seat first: it plays the X drains; the other seats are the opponents
fn game() -> Game {
    table(&["yshtola", "kefka", "sauron"])
}

fn hopts(g: &mut Game, c: CardId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).and_then(|i| i.hand_options).expect("the card has hand options");
    f(g, c, P0, post).unwrap()
}

fn go(g: &mut Game, o: &Opt) -> bool {
    act::perform(g, P0, o.act.as_ref().expect("a play")).unwrap()
}

fn tapped(g: &Game, ls: &[PermId]) -> usize {
    ls.iter().filter(|&&l| g.perm(l).tapped).count()
}

fn only(opts: Vec<Opt>) -> Opt {
    assert_eq!(opts.len(), 1, "one play: {:?}", opts.iter().map(|o| &o.label).collect::<Vec<_>>());
    opts.into_iter().next().unwrap()
}

/// the numbers in a label, in order
fn nums(label: &str) -> Vec<i32> {
    label.split(|c: char| !c.is_ascii_digit()).filter(|s| !s.is_empty()).map(|s| s.parse().unwrap()).collect()
}

fn krrik(g: &mut Game) {
    perm(g, P0, "K'rrik, Son of Yawgmoth");
}

// ------------------------------------------------------------------ Y'shtola: Exsanguinate, Debt to the Deathless
fn drain(g: &mut Game, name: &str) -> (Opt, CardId) {
    g.player_mut(P0).turns = 8; // late: a drain with no kill is worth it from X = 4
    let c = hand(g, P0, &[name])[0];
    (only(hopts(g, c, Some(false))), c)
}

#[test]
fn exsanguinate_without_a_reducer_is_x_four_for_six_mana() {
    let mut g = game();
    let ls = lands(&mut g, P0, "Swamp", 6, false);
    let (o, _) = drain(&mut g, "Exsanguinate");
    assert_eq!(o.label, "Exsanguinate (X = 4)");
    assert!(go(&mut g, &o));
    assert_eq!(tapped(&g, &ls), 6);
    assert_eq!(g.player(P1).life, 36);
}

#[test]
fn exsanguinate_with_jet_medallion_takes_one_more_x_for_the_same_mana() {
    let mut g = game();
    perm(&mut g, P0, "Jet Medallion");
    let ls = lands(&mut g, P0, "Swamp", 6, false);
    let (o, _) = drain(&mut g, "Exsanguinate");
    assert_eq!(o.label, "Exsanguinate (X = 5)", "{{5}}{{B}}{{B}} less {{1}} is six mana");
    assert!(go(&mut g, &o));
    assert_eq!(tapped(&g, &ls), 6);
    assert_eq!((g.player(P1).life, g.player(P0).life), (35, 50), "each opponent loses 5, the caster gains the 10");
}

#[test]
fn exsanguinate_with_a_medallion_and_a_tax_after_it() {
    let mut g = game();
    perm(&mut g, P0, "Jet Medallion");
    perm(&mut g, P1, "Sphere of Resistance");
    let ls = lands(&mut g, P0, "Swamp", 6, false);
    let (o, _) = drain(&mut g, "Exsanguinate");
    assert_eq!(
        o.label, "Exsanguinate (X = 4)",
        "the {{1}} off and the {{1}} tax on: the same as no reducer and no tax"
    );
    assert!(go(&mut g, &o));
    assert_eq!(tapped(&g, &ls), 6);
}

#[test]
fn exsanguinate_with_krrik_pays_life_for_the_black_pips_and_all_the_mana_goes_to_x() {
    let mut g = game();
    krrik(&mut g);
    let ls = lands(&mut g, P0, "Swamp", 4, false);
    let (o, _) = drain(&mut g, "Exsanguinate");
    assert_eq!(o.label, "Exsanguinate (X = 4)");
    assert!(go(&mut g, &o));
    assert_eq!(tapped(&g, &ls), 4, "{{4}} from the mana, {{B}}{{B}} from life");
    assert_eq!(g.player(P0).life, 44, "40, less 2 life for each {{B}}, plus the 8 drained");
    assert_eq!(g.player(P1).life, 36);
}

#[test]
fn debt_to_the_deathless_with_and_without_jet_medallion() {
    // {X}{W}{W}{B}{B}: eight lands are X = 4 bare, X = 5 with Jet Medallion ({5}{W}{W}{B}{B} less {1})
    for (reducer, x) in [(false, 4), (true, 5)] {
        let mut g = game();
        if reducer {
            perm(&mut g, P0, "Jet Medallion");
        }
        lands(&mut g, P0, "Plains", 2, false);
        lands(&mut g, P0, "Swamp", 6, false);
        let (o, _) = drain(&mut g, "Debt to the Deathless");
        assert!(o.label.contains(&format!("X = {x}")), "{}", o.label);
        assert!(go(&mut g, &o));
        assert!(g.player(P0).lands.iter().all(|&l| g.perm(l).tapped), "all eight lands, reducer: {reducer}");
    }
}

// ------------------------------------------------------------------ the generic cast: X spells of outside decks
/// Secure the Wastes ({X}{W}): X 1/1 Warriors. Cast by `brain::do_cast`, X = all the mana the cost leaves
fn wastes(g: &mut Game) -> CardId {
    hand(g, P0, &["Secure the Wastes"])[0]
}

#[test]
fn a_tokx_spell_is_x_four_for_five_mana_without_a_reducer() {
    let mut g = game();
    let ls = lands(&mut g, P0, "Plains", 5, false);
    let c = wastes(&mut g);
    assert!(brain::do_cast(&mut g, P0, c, None).unwrap());
    assert_eq!(tapped(&g, &ls), 5);
    assert_eq!(g.player(P0).perms.len(), 4, "four Warriors");
}

#[test]
fn a_tokx_spell_with_pearl_medallion_is_x_five_for_five_mana() {
    let mut g = game();
    perm(&mut g, P0, "Pearl Medallion");
    let ls = lands(&mut g, P0, "Plains", 5, false);
    let before = g.player(P0).perms.len();
    let c = wastes(&mut g);
    assert!(brain::do_cast(&mut g, P0, c, None).unwrap());
    assert_eq!(tapped(&g, &ls), 5, "{{5}}{{W}} less {{1}}");
    assert_eq!(g.player(P0).perms.len() - before, 5, "five Warriors: the reducer applied to the generic part with X");
}

#[test]
fn a_tokx_spell_with_a_medallion_and_a_tax_is_as_before() {
    let mut g = game();
    perm(&mut g, P0, "Pearl Medallion");
    perm(&mut g, P1, "Sphere of Resistance");
    let ls = lands(&mut g, P0, "Plains", 5, false);
    let before = g.player(P0).perms.len();
    let c = wastes(&mut g);
    assert!(brain::do_cast(&mut g, P0, c, None).unwrap());
    assert_eq!(tapped(&g, &ls), 5);
    assert_eq!(g.player(P0).perms.len() - before, 4, "the {{1}} tax comes after the reduction");
}

#[test]
fn a_tokx_spell_that_cannot_pay_its_base_cost_is_not_cast() {
    let mut g = game();
    let ls = lands(&mut g, P0, "Plains", 0, false);
    let c = wastes(&mut g);
    assert!(!brain::do_cast(&mut g, P0, c, None).unwrap());
    assert!(ls.is_empty() && g.player(P0).hand.contains(&c));
}

/// Mind Twist ({X}{B}, an Xtutor card cast with no planned X: X = all the spare mana): the opponent discards X at random
fn twist(g: &mut Game, hand_size: usize) -> CardId {
    let filler = vec!["Island"; hand_size];
    hand(g, P1, &filler);
    hand(g, P0, &["Mind Twist"])[0]
}

#[test]
fn an_xtutor_spell_with_no_planned_x_takes_all_the_spare_mana() {
    let mut g = game();
    let ls = lands(&mut g, P0, "Swamp", 3, false);
    let c = twist(&mut g, 8);
    let n = g.player(P1).hand.len();
    assert!(brain::do_cast(&mut g, P0, c, None).unwrap());
    assert_eq!(tapped(&g, &ls), 3);
    assert_eq!(n - g.player(P1).hand.len(), 2, "X = 2: {{2}}{{B}}");
}

#[test]
fn an_xtutor_spell_with_jet_medallion_has_one_more_x() {
    let mut g = game();
    perm(&mut g, P0, "Jet Medallion");
    let ls = lands(&mut g, P0, "Swamp", 3, false);
    let c = twist(&mut g, 8);
    let n = g.player(P1).hand.len();
    assert!(brain::do_cast(&mut g, P0, c, None).unwrap());
    assert_eq!(tapped(&g, &ls), 3);
    assert_eq!(n - g.player(P1).hand.len(), 3, "X = 3: {{3}}{{B}} less {{1}}");
}

#[test]
fn an_xtutor_spell_with_krrik_charges_the_life_for_the_black_pip() {
    let mut g = game();
    krrik(&mut g);
    let ls = lands(&mut g, P0, "Swamp", 3, false);
    let c = twist(&mut g, 8);
    let n = g.player(P1).hand.len();
    assert!(brain::do_cast(&mut g, P0, c, None).unwrap());
    assert_eq!(tapped(&g, &ls), 3);
    assert_eq!(n - g.player(P1).hand.len(), 3, "X = 3: the {{B}} is paid with life");
    assert_eq!(g.player(P0).life, 38);
}

#[test]
fn a_planned_xtutor_x_is_priced_with_the_reducer_too() {
    // Green Sun's Zenith ({X}{G}), the planner's X = 3 (`plan_x`): {3}{G} less {1} is three mana
    let mut g = game();
    perm(&mut g, P0, "Emerald Medallion");
    let ls = lands(&mut g, P0, "Forest", 3, false);
    let c = hand(&mut g, P0, &["Green Sun's Zenith"])[0];
    g.player_mut(P0).plan_x = Some(3);
    assert!(brain::do_cast(&mut g, P0, c, None).unwrap());
    assert_eq!(tapped(&g, &ls), 3);
    assert_eq!(g.last_x, 3);
}

#[test]
fn a_planned_xtutor_x_without_a_reducer_still_costs_four() {
    let mut g = game();
    let ls = lands(&mut g, P0, "Forest", 3, false);
    let c = hand(&mut g, P0, &["Green Sun's Zenith"])[0];
    g.player_mut(P0).plan_x = Some(3);
    assert!(!brain::do_cast(&mut g, P0, c, None).unwrap(), "three mana is not {{3}}{{G}}");
    assert_eq!(tapped(&g, &ls), 0);
    let mut g = game();
    let ls = lands(&mut g, P0, "Forest", 4, false);
    let c = hand(&mut g, P0, &["Green Sun's Zenith"])[0];
    g.player_mut(P0).plan_x = Some(3);
    assert!(brain::do_cast(&mut g, P0, c, None).unwrap());
    assert_eq!(tapped(&g, &ls), 4);
}

// ------------------------------------------------------------------ the card's own X: The Meathook Massacre
fn meathook_game(lands_n: usize, reducer: bool, krrik_out: bool) -> (Game, Vec<PermId>, PermId) {
    let mut g = game();
    if reducer {
        perm(&mut g, P0, "Jet Medallion");
    }
    if krrik_out {
        krrik(&mut g);
    }
    for t in 1..=8 {
        token(&mut g, P1, t); // toughness 1 to 8: every extra X kills one more
    }
    let ls = lands(&mut g, P0, "Swamp", lands_n, false);
    let c = hand(&mut g, P0, &["The Meathook Massacre"])[0];
    assert!(brain::do_cast(&mut g, P0, c, None).unwrap());
    let m = g.player(P0).perms.iter().copied().find(|&m| &*g.perm(m).name == "The Meathook Massacre").unwrap();
    (g, ls, m)
}

#[test]
fn meathook_is_x_four_for_six_mana_without_a_reducer() {
    let (g, ls, m) = meathook_game(6, false, false);
    assert_eq!(g.perm(m).data.int(DataKey::CastX), 4);
    assert_eq!(tapped(&g, &ls), 6);
}

#[test]
fn meathook_with_jet_medallion_is_x_five_for_six_mana() {
    let (g, ls, m) = meathook_game(6, true, false);
    assert_eq!(g.perm(m).data.int(DataKey::CastX), 5);
    assert_eq!(tapped(&g, &ls), 6);
}

#[test]
fn meathook_with_krrik_pays_life_and_puts_all_the_mana_in_x() {
    let (g, ls, m) = meathook_game(4, false, true);
    assert_eq!(g.perm(m).data.int(DataKey::CastX), 4);
    assert_eq!(tapped(&g, &ls), 4);
    let died = 8 - g.player(P1).perms.iter().filter(|&&t| g.perm(t).on_bf).count() as i32;
    assert_eq!(died, 4, "toughness 1 to 4 die");
    assert_eq!(g.player(P0).life, 40 - 4 + died, "4 life for the {{B}}{{B}}, 1 gained for each creature that died");
}

// ------------------------------------------------------------------ Crackle with Power
#[test]
fn crackle_with_power_prices_three_x_through_the_reducer() {
    // {X}{X}{X}{R}{R}: ten mana is X = 2 (eight) bare, X = 3 (nine, less {1}, plus {R}{R}) with Ruby Medallion
    for (reducer, x, spent) in [(false, 2, 8), (true, 3, 10)] {
        let mut g = game();
        if reducer {
            perm(&mut g, P0, "Ruby Medallion");
        }
        let ls = lands(&mut g, P0, "Mountain", 10, false);
        hand(&mut g, P0, &["Crackle with Power"]);
        let s = Situation::new(&g, P0);
        let opts = brain::extra_options(&mut g, P0, &s, false, true);
        let o = opts.iter().find(|o| o.label.starts_with("Crackle with Power")).expect("the AI offers Crackle");
        assert_eq!(o.label, format!("Crackle with Power X={x}"));
        assert!(go(&mut g, o));
        assert_eq!(tapped(&g, &ls), spent, "reducer: {reducer}");
    }
}

// ------------------------------------------------------------------ the Stella Lee precon's X spells
#[test]
fn curse_of_the_swine() {
    // {X}{U}{U}: four targets worth exiling; X is the most the mana pays, at most four
    for (reducer, x, spent) in [(false, 3, 5), (true, 4, 5)] {
        let mut g = table(&["stella-lee-quick-draw", "seph"]);
        if reducer {
            perm(&mut g, P0, "Sapphire Medallion");
        }
        for n in ["Shivan Dragon", "Serra Angel", "Baneslayer Angel", "Inferno Titan"] {
            perm(&mut g, P1, n);
        }
        let ls = lands(&mut g, P0, "Island", 5, false);
        let c = hand(&mut g, P0, &["Curse of the Swine"])[0];
        let o = only(hopts(&mut g, c, Some(false)));
        assert_eq!(o.label, format!("Curse of the Swine (X = {x})"), "reducer: {reducer}");
        assert!(go(&mut g, &o));
        assert_eq!(tapped(&g, &ls), spent);
    }
}

#[test]
fn epic_experiment() {
    // {X}{U}{R}, X at most seven: six mana is X = 4, with Sapphire Medallion X = 5
    for (reducer, x) in [(false, 4), (true, 5)] {
        let mut g = table(&["stella-lee-quick-draw", "seph"]);
        if reducer {
            perm(&mut g, P0, "Sapphire Medallion");
        }
        let mut ls = lands(&mut g, P0, "Island", 3, false);
        ls.extend(lands(&mut g, P0, "Mountain", 3, false));
        let c = hand(&mut g, P0, &["Epic Experiment"])[0];
        let o = only(hopts(&mut g, c, Some(false)));
        assert_eq!(o.label, format!("Epic Experiment (X = {x})"), "reducer: {reducer}");
        assert!(go(&mut g, &o));
        assert_eq!(tapped(&g, &ls), 6);
    }
}

#[test]
fn finale_of_revelation() {
    // {X}{U}{U}: six Islands, X = 4; with Sapphire Medallion X = 5. It draws X cards
    for (reducer, x) in [(false, 4), (true, 5)] {
        let mut g = table(&["stella-lee-quick-draw", "seph"]);
        if reducer {
            perm(&mut g, P0, "Sapphire Medallion");
        }
        let ls = lands(&mut g, P0, "Island", 6, false);
        let c = hand(&mut g, P0, &["Finale of Revelation"])[0];
        let o = only(hopts(&mut g, c, Some(true)));
        assert_eq!(o.label, format!("Finale of Revelation (X = {x})"), "reducer: {reducer}");
        assert!(go(&mut g, &o));
        assert_eq!(tapped(&g, &ls), 6);
        assert_eq!(g.player(P0).hand.len(), x as usize, "draws X");
    }
}

#[test]
fn finale_of_promise() {
    // {X}{R}{R}: a mana value 4 sorcery in the graveyard needs X = 4: five Mountains reach it only with Ruby Medallion
    for (reducer, has_option) in [(false, false), (true, true)] {
        let mut g = table(&["stella-lee-quick-draw", "seph"]);
        if reducer {
            perm(&mut g, P0, "Ruby Medallion");
        }
        let ls = lands(&mut g, P0, "Mountain", 5, false);
        let tgt = card(&g, "Fact or Fiction"); // {3}{U}: mana value 4
        g.player_mut(P0).gy.push(tgt);
        let c = hand(&mut g, P0, &["Finale of Promise"])[0];
        let opts = hopts(&mut g, c, Some(false));
        if !has_option {
            assert!(opts.is_empty(), "{:?}", opts.iter().map(|o| &o.label).collect::<Vec<_>>());
        } else {
            let o = only(opts);
            assert_eq!(o.label, "Finale of Promise (X = 4)");
            assert!(go(&mut g, &o));
            assert_eq!(tapped(&g, &ls), 5, "{{4}}{{R}}{{R}} less {{1}}");
        }
    }
}

// ------------------------------------------------------------------ the other X spells
#[test]
fn torment_of_hailfire() {
    for (reducer, x) in [(false, 4), (true, 5)] {
        let mut g = game();
        if reducer {
            perm(&mut g, P0, "Jet Medallion");
        }
        g.player_mut(P0).turns = 8;
        let ls = lands(&mut g, P0, "Swamp", 6, false);
        let c = hand(&mut g, P0, &["Torment of Hailfire"])[0];
        let o = only(hopts(&mut g, c, Some(false)));
        assert_eq!(o.label, format!("Torment of Hailfire (X = {x})"), "reducer: {reducer}");
        assert!(go(&mut g, &o));
        assert_eq!(tapped(&g, &ls), 6);
    }
}

#[test]
fn torment_of_hailfire_with_krrik_charges_the_life() {
    let mut g = game();
    krrik(&mut g);
    g.player_mut(P0).turns = 8;
    let ls = lands(&mut g, P0, "Swamp", 4, false);
    let c = hand(&mut g, P0, &["Torment of Hailfire"])[0];
    let o = only(hopts(&mut g, c, Some(false)));
    assert_eq!(o.label, "Torment of Hailfire (X = 4)");
    assert!(go(&mut g, &o));
    assert_eq!(tapped(&g, &ls), 4);
    assert_eq!(g.player(P0).life, 36);
}

#[test]
fn comet_storm_prices_x_and_the_kicks_together() {
    // {X}{R}{R}, multikicker {1}: seven mana is X + kicks = 5 bare, 6 with Ruby Medallion; either way all seven are spent
    for (reducer, total) in [(false, 5), (true, 6)] {
        let mut g = game();
        if reducer {
            perm(&mut g, P0, "Ruby Medallion");
        }
        g.player_mut(P1).life = 3;
        let ls = lands(&mut g, P0, "Mountain", 7, false);
        let c = hand(&mut g, P0, &["Comet Storm"])[0];
        let o = only(hopts(&mut g, c, Some(false)));
        let n = nums(&o.label);
        assert_eq!(n[0] + n[1], total, "{} (reducer: {reducer})", o.label);
        assert!(go(&mut g, &o));
        assert_eq!(tapped(&g, &ls), 7);
    }
}

#[test]
fn wild_hypothesis() {
    // {X}{G}: four Forests, X = 3; with Emerald Medallion X = 4
    for (reducer, x) in [(false, 3), (true, 4)] {
        let mut g = game();
        if reducer {
            perm(&mut g, P0, "Emerald Medallion");
        }
        let ls = lands(&mut g, P0, "Forest", 4, false);
        let c = hand(&mut g, P0, &["Wild Hypothesis"])[0];
        let o = only(hopts(&mut g, c, Some(false)));
        assert_eq!(o.label, format!("Wild Hypothesis (X = {x})"), "reducer: {reducer}");
        assert!(go(&mut g, &o));
        assert_eq!(tapped(&g, &ls), 4);
    }
}

#[test]
fn wan_shi_tong() {
    // {X}{U}{U}, flash, cast at the end of the turn before: six Islands, X = 4; with Sapphire Medallion X = 5
    for (reducer, x) in [(false, 4), (true, 5)] {
        let mut g = game();
        if reducer {
            perm(&mut g, P0, "Sapphire Medallion");
        }
        let ls = lands(&mut g, P0, "Island", 6, false);
        let c = hand(&mut g, P0, &["Wan Shi Tong, Librarian"])[0];
        let o = only(hopts(&mut g, c, None));
        assert_eq!(o.label, format!("Wan Shi Tong X={x}"), "reducer: {reducer}");
        assert!(go(&mut g, &o));
        assert_eq!(tapped(&g, &ls), 6);
        let m = g.player(P0).perms.iter().copied().find(|&m| &*g.perm(m).name == "Wan Shi Tong, Librarian").unwrap();
        assert_eq!(g.perm(m).plus, x, "X +1/+1 counters");
    }
}

#[test]
fn west_coast_expansion() {
    // {X}{U}{U}: X >= 2 on a small hand: six Islands, X = 4; with Sapphire Medallion X = 5
    for (reducer, x) in [(false, 4), (true, 5)] {
        let mut g = game();
        if reducer {
            perm(&mut g, P0, "Sapphire Medallion");
        }
        let ls = lands(&mut g, P0, "Island", 6, false);
        let c = hand(&mut g, P0, &["West Coast Expansion"])[0];
        let o = only(hopts(&mut g, c, Some(false)));
        assert_eq!(o.label, format!("West Coast Expansion X={x}"), "reducer: {reducer}");
        assert!(go(&mut g, &o));
        assert_eq!(tapped(&g, &ls), 6);
    }
}

#[test]
fn astral_cornucopia() {
    // {X}{X}{X} (an artifact): eleven mana is X = 3 bare, X = 4 with Etherium Sculptor's {1} off artifact spells
    for (reducer, x, spent) in [(false, 3, 9), (true, 4, 11)] {
        let mut g = game();
        if reducer {
            perm(&mut g, P0, "Etherium Sculptor");
        }
        let ls = lands(&mut g, P0, "Mountain", 11, false);
        let c = hand(&mut g, P0, &["Astral Cornucopia"])[0];
        let o = only(hopts(&mut g, c, Some(false)));
        assert_eq!(o.label, format!("cast Astral Cornucopia (X = {x})"), "reducer: {reducer}");
        assert!(go(&mut g, &o));
        assert_eq!(tapped(&g, &ls), spent);
        let m = g.player(P0).perms.iter().copied().find(|&m| &*g.perm(m).name == "Astral Cornucopia").unwrap();
        assert_eq!(g.perm(m).data.int(DataKey::Charge), x as i64);
    }
}

#[test]
fn everflowing_chalice_kicks() {
    // multikicker {2}: five mana is two kicks bare, three ({6} less {1}) with Etherium Sculptor, two with a {1} tax
    for (setup, k, spent) in [("none", 2, 4), ("sculptor", 3, 5), ("tax", 2, 5)] {
        // the tax: two kicks cost {4} and the tax {1}
        let mut g = game();
        match setup {
            "sculptor" => {
                perm(&mut g, P0, "Etherium Sculptor");
            }
            "tax" => {
                perm(&mut g, P1, "Sphere of Resistance");
            }
            _ => {}
        }
        let ls = lands(&mut g, P0, "Mountain", 5, false);
        let c = hand(&mut g, P0, &["Everflowing Chalice"])[0];
        let o = only(hopts(&mut g, c, Some(false)));
        assert_eq!(o.label, format!("Everflowing Chalice x{k}"), "{setup}");
        assert!(go(&mut g, &o));
        assert_eq!(tapped(&g, &ls), spent, "{setup}");
    }
}

// ------------------------------------------------------------------ the Shatterskull Smashing, Heliod's Intervention,
// Stargaze, Disintegrate, White Sun's Twilight, Mockingbird, Agadeem's Awakening, Nature's Rhythm, Krasis, Primal Might,
// Braingeyser through Garth
#[test]
fn hydroid_krasis_and_primal_might_from_the_pool_options() {
    // Krasis {X}{G}{U}: seven mana, X = 5 (at least 4 is offered); Sapphire Medallion: X = 6
    for (reducer, x) in [(false, 5), (true, 6)] {
        let mut g = game();
        if reducer {
            perm(&mut g, P0, "Sapphire Medallion");
        }
        let mut ls = lands(&mut g, P0, "Forest", 4, false);
        ls.extend(lands(&mut g, P0, "Island", 3, false));
        hand(&mut g, P0, &["Hydroid Krasis"]);
        let s = Situation::new(&g, P0);
        let opts = pool::spell_options(&g, P0, &s, Some(false));
        let o = opts.iter().find(|o| o.label.starts_with("Hydroid Krasis")).expect("the pool AI offers Krasis");
        assert_eq!(o.label, format!("Hydroid Krasis X={x}"), "reducer: {reducer}");
        assert!(go(&mut g, o));
        assert_eq!(tapped(&g, &ls), 7);
    }
}

#[test]
fn primal_might_from_the_pool_options() {
    // {X}{G}: a 1/1 fights a 5/5: it needs X = 4, four Forests pay for it only with Emerald Medallion
    for (reducer, has_option) in [(false, false), (true, true)] {
        let mut g = game();
        if reducer {
            perm(&mut g, P0, "Emerald Medallion");
        }
        token(&mut g, P0, 1);
        let dragon = perm(&mut g, P1, "Shivan Dragon");
        let ls = lands(&mut g, P0, "Forest", 4, false);
        hand(&mut g, P0, &["Primal Might"]);
        let s = Situation::new(&g, P0);
        let opts = pool::spell_options(&g, P0, &s, Some(false));
        let o = opts.iter().find(|o| o.label.starts_with("Primal Might"));
        assert_eq!(o.is_some(), has_option, "reducer: {reducer}");
        if let Some(o) = o {
            assert!(go(&mut g, o));
            assert_eq!(tapped(&g, &ls), 4);
            assert!(!g.perm(dragon).on_bf, "X = 4: the fight kills the Dragon");
        }
    }
}

#[test]
fn shatterskull_smashing() {
    // {X}{R}{R}: a 5/5 needs X = 5: six Mountains pay for it only with Ruby Medallion ({5}{R}{R} less {1})
    for (reducer, has_option) in [(false, false), (true, true)] {
        let mut g = game();
        if reducer {
            perm(&mut g, P0, "Ruby Medallion");
        }
        let dragon = perm(&mut g, P1, "Shivan Dragon");
        let ls = lands(&mut g, P0, "Mountain", 6, false);
        let c = hand(&mut g, P0, &["Shatterskull Smashing"])[0];
        let opts = hopts(&mut g, c, Some(false));
        let o = opts.iter().find(|o| o.label.starts_with("Shatterskull Smashing (X"));
        assert_eq!(o.is_some(), has_option, "reducer: {reducer}");
        if let Some(o) = o {
            assert_eq!(o.label, "Shatterskull Smashing (X = 5)");
            assert!(go(&mut g, o));
            assert_eq!(tapped(&g, &ls), 6);
            assert!(!g.perm(dragon).on_bf);
        }
    }
}

#[test]
fn heliods_intervention() {
    // {X}{W}{W}: four opposing artifacts worth destroying; five Plains are X = 3, with Pearl Medallion X = 4
    for (reducer, x) in [(false, 3), (true, 4)] {
        let mut g = game();
        if reducer {
            perm(&mut g, P0, "Pearl Medallion");
        }
        let arts: Vec<PermId> = ["Rhystic Study", "Smothering Tithe", "Mystic Remora", "Basalt Monolith"]
            .iter()
            .map(|n| perm(&mut g, P1, n))
            .collect();
        let ls = lands(&mut g, P0, "Plains", 5, false);
        let c = hand(&mut g, P0, &["Heliod's Intervention"])[0];
        let opts = hopts(&mut g, c, Some(false));
        let o = opts.iter().find(|o| o.label.contains("destroy")).expect("destroy mode");
        assert_eq!(o.label, format!("Heliod's Intervention: destroy {x}"), "reducer: {reducer}");
        assert!(go(&mut g, o));
        assert_eq!(tapped(&g, &ls), 5);
        assert_eq!(arts.iter().filter(|&&m| !g.perm(m).on_bf).count(), x as usize);
    }
}

#[test]
fn stargaze() {
    // {X}{B}{B}, X at most five: six Swamps are X = 4, with Jet Medallion X = 5
    for (reducer, x) in [(false, 4), (true, 5)] {
        let mut g = game();
        if reducer {
            perm(&mut g, P0, "Jet Medallion");
        }
        let ls = lands(&mut g, P0, "Swamp", 6, false);
        let c = hand(&mut g, P0, &["Stargaze"])[0];
        let o = only(hopts(&mut g, c, Some(false)));
        assert_eq!(o.label, format!("Stargaze (X={x})"), "reducer: {reducer}");
        assert!(go(&mut g, &o));
        assert_eq!(tapped(&g, &ls), 6);
        assert_eq!(g.player(P0).life, 40 - x, "it loses X life");
    }
}

#[test]
fn stargaze_with_krrik_counts_the_life_for_the_pips_in_its_floor() {
    let mut g = game();
    krrik(&mut g);
    let ls = lands(&mut g, P0, "Swamp", 4, false);
    let c = hand(&mut g, P0, &["Stargaze"])[0];
    let o = only(hopts(&mut g, c, Some(false)));
    assert_eq!(o.label, "Stargaze (X=4)");
    assert!(go(&mut g, &o));
    assert_eq!(tapped(&g, &ls), 4);
    assert_eq!(g.player(P0).life, 32, "4 for the {{B}}{{B}} and 4 for X");
    // at 15 life K'rrik's floor (15) lets it pay no pip with life: four mana are X = 2, and only X costs life
    let mut g = game();
    krrik(&mut g);
    g.player_mut(P0).life = 15;
    let ls = lands(&mut g, P0, "Swamp", 4, false);
    let c = hand(&mut g, P0, &["Stargaze"])[0];
    let o = only(hopts(&mut g, c, Some(false)));
    assert_eq!(o.label, "Stargaze (X=2)");
    assert!(go(&mut g, &o));
    assert_eq!((tapped(&g, &ls), g.player(P0).life), (4, 13));
}

#[test]
fn disintegrate() {
    // {X}{R}: an opponent at six is killed with X = 6: six Mountains pay for it only with Ruby Medallion
    for (reducer, has_option) in [(false, false), (true, true)] {
        let mut g = game();
        if reducer {
            perm(&mut g, P0, "Ruby Medallion");
        }
        g.player_mut(P1).life = 6;
        let ls = lands(&mut g, P0, "Mountain", 6, false);
        let c = hand(&mut g, P0, &["Disintegrate"])[0];
        let opts = hopts(&mut g, c, Some(false));
        let o = opts.iter().find(|o| o.label.starts_with("Disintegrate the face"));
        assert_eq!(o.is_some(), has_option, "reducer: {reducer}");
        if let Some(o) = o {
            assert!(go(&mut g, o));
            assert_eq!(tapped(&g, &ls), 6);
            assert!(!g.player(P1).alive);
        }
    }
}

#[test]
fn white_suns_twilight() {
    // {X}{W}{W}, X at most four unless a wipe is worth more: five Plains are X = 3, with Pearl Medallion X = 4
    for (reducer, x) in [(false, 3), (true, 4)] {
        let mut g = game();
        if reducer {
            perm(&mut g, P0, "Pearl Medallion");
        }
        let ls = lands(&mut g, P0, "Plains", 5, false);
        let c = hand(&mut g, P0, &["White Sun's Twilight"])[0];
        let o = only(hopts(&mut g, c, Some(false)));
        assert_eq!(o.label, format!("White Sun's Twilight X={x}"), "reducer: {reducer}");
        assert!(go(&mut g, &o));
        assert_eq!(tapped(&g, &ls), 5);
        assert_eq!(g.player(P0).life, 40 + x, "it gains X");
    }
}

#[test]
fn mockingbird() {
    // {X}{U}: a mana value 6 creature to copy needs X = 5: five Islands pay for it only with Sapphire Medallion
    for (reducer, lands_n, has_option) in [(false, 5, false), (false, 6, true), (true, 5, true)] {
        let mut g = game();
        if reducer {
            perm(&mut g, P0, "Sapphire Medallion");
        }
        perm(&mut g, P1, "Grave Titan");
        let ls = lands(&mut g, P0, "Island", lands_n, false);
        let c = hand(&mut g, P0, &["Mockingbird"])[0];
        let opts = hopts(&mut g, c, Some(false));
        assert_eq!(!opts.is_empty(), has_option, "reducer: {reducer}, {lands_n} lands");
        if has_option {
            let o = only(opts);
            assert_eq!(o.label, "Mockingbird (X = 5, copy Grave Titan)");
            assert!(go(&mut g, &o));
            assert_eq!(tapped(&g, &ls), lands_n);
        }
    }
}

#[test]
fn agadeems_awakening() {
    // {X}{B}{B}{B}: Grave Titan from the graveyard needs X = 6: eight Swamps pay for it only with Jet Medallion
    for (reducer, has_option) in [(false, false), (true, true)] {
        let mut g = game();
        if reducer {
            perm(&mut g, P0, "Jet Medallion");
        }
        let ls = lands(&mut g, P0, "Swamp", 8, false);
        let titan = card(&g, "Grave Titan");
        g.player_mut(P0).gy.push(titan);
        let c = hand(&mut g, P0, &["Agadeem's Awakening // Agadeem, the Undercrypt"])[0];
        let opts = hopts(&mut g, c, Some(false));
        let o = opts.iter().find(|o| o.label.starts_with("Agadeem's Awakening (X"));
        assert_eq!(o.is_some(), has_option, "reducer: {reducer}");
        if let Some(o) = o {
            assert_eq!(o.label, "Agadeem's Awakening (X = 6)");
            assert!(go(&mut g, o));
            assert_eq!(tapped(&g, &ls), 8);
        }
    }
}

#[test]
fn natures_rhythm_harmonize() {
    // {X}{G}{G}{G}{G}, harmonize: tapping a 6-power creature takes 6 off the generic cost. Four Forests: X = 6 bare, 7 with
    // Emerald Medallion; the creature put onto the battlefield is the one with mana value X or less
    for (reducer, avenger_in) in [(false, false), (true, true)] {
        let mut g = game();
        if reducer {
            perm(&mut g, P0, "Emerald Medallion");
        }
        token(&mut g, P0, 6);
        let ls = lands(&mut g, P0, "Forest", 4, false);
        let avenger = card(&g, "Avenger of Zendikar"); // mana value 7
        g.player_mut(P0).library = vec![avenger];
        let c = hand(&mut g, P0, &["Nature's Rhythm"])[0];
        g.player_mut(P0).hand.retain(|&x| x != c);
        g.player_mut(P0).gy.push(c);
        let f = g.registry.get(c).and_then(|i| i.gy_options).unwrap();
        let opts = f(&mut g, c, P0, Some(false)).unwrap();
        let o = only(opts);
        assert!(go(&mut g, &o));
        assert_eq!(tapped(&g, &ls), 4);
        let out = g.player(P0).perms.iter().any(|&m| &*g.perm(m).name == "Avenger of Zendikar");
        assert_eq!(out, avenger_in, "reducer: {reducer}");
    }
}

#[test]
fn braingeyser_through_garth() {
    // {X}{U}{U}: eight Islands are X = 6 bare, X = 7 with Sapphire Medallion
    for (reducer, x) in [(false, 6), (true, 7)] {
        let mut g =
            table(&["urdragon", "miirym-temur-dragons", "urza-mono-blue-artifacts", "zur-esper-enchantment-control"]);
        g.player_mut(P0).hand.clear();
        let cmd = card(&g, "The Ur-Dragon");
        g.player_mut(P0).set_cmd_zone(cmd, false);
        if reducer {
            perm(&mut g, P0, "Sapphire Medallion");
        }
        let ls = lands(&mut g, P0, "Island", 8, false);
        let m = perm(&mut g, P0, "Garth One-Eye");
        let f = g.registry.get(card(&g, "Garth One-Eye")).unwrap().options.unwrap();
        let opts = f(&mut g, m, P0, Some(false)).unwrap();
        let o = opts.iter().find(|o| o.label.starts_with("Garth One-Eye: Braingeyser")).expect("Braingeyser");
        assert_eq!(o.label, format!("Garth One-Eye: Braingeyser for {x}"), "reducer: {reducer}");
        assert!(go(&mut g, o));
        assert_eq!(tapped(&g, &ls), 8);
        assert_eq!(g.player(P0).hand.len(), x as usize, "draws X");
    }
}

#[test]
fn meathook_priority_counts_the_x_the_reducer_allows() {
    // three 5-toughness creatures die only at X = 5: six Swamps are X = 4 bare (the plain priority), X = 5 with Jet Medallion
    for (reducer, want) in [(false, 44), (true, 68)] {
        let mut g = game();
        if reducer {
            perm(&mut g, P0, "Jet Medallion");
        }
        for _ in 0..3 {
            token(&mut g, P1, 5);
        }
        lands(&mut g, P0, "Swamp", 6, false);
        let c = hand(&mut g, P0, &["The Meathook Massacre"])[0];
        let prio = g.registry.get(c).and_then(|i| i.prio).expect("a priority");
        assert_eq!(prio(&g, P0, c), want, "reducer: {reducer}");
    }
}
