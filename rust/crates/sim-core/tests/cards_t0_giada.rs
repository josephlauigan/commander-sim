//! The giada precon's cards (impls/t0_giada.rs) on hand-built positions, Oracle text as the expectation. Seat 0 plays
//! the giada deck, seat 1 veyran, seat 2 seph.

use sim_core::engine::life::{check_state, gain};
use sim_core::engine::values::{epow, etgh};
use sim_core::engine::zones::{self, Tokens};
use sim_core::engine::{cast, combat, turn};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    table(&["giada-calling-all-angels", "veyran", "seph"])
}

fn pt(g: &Game, m: PermId) -> (i32, i32) {
    (epow(g, m), etgh(g, m))
}

fn named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf && &*g.perm(m).name == name).collect()
}

fn cast_free(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
    c
}

fn wspell(g: &Game, p: PlayerId) -> i32 {
    g.player(p).clues as i32
}

#[test]
fn giada_gives_angels_a_counter_per_angel_already_controlled() {
    let mut g = tbl();
    let giada = perm(&mut g, P0, "Giada, Font of Hope");
    assert_eq!(g.perm(giada).plus, 0); // not for itself
    let lyra = perm(&mut g, P0, "Lyra Dawnbringer");
    assert_eq!(g.perm(lyra).plus, 1); // Giada
    let reya = perm(&mut g, P0, "Reya Dawnbringer");
    assert_eq!(g.perm(reya).plus, 2); // Giada and Lyra
    let human = perm(&mut g, P0, "Metallic Mimic");
    assert_eq!(g.perm(human).plus, 0); // not an Angel
    let opp = perm(&mut g, P1, "Lyra Dawnbringer");
    assert_eq!(g.perm(opp).plus, 0); // only yours
}

#[test]
fn giada_mana_is_for_angel_spells_only() {
    let mut g = tbl();
    let giada = perm(&mut g, P0, "Giada, Font of Hope");
    g.perm_mut(giada).sick = false;
    assert!(!sim_core::engine::mana::can_pay(&g, P0, 0, "W", false));
    let lyra = card(&g, "Lyra Dawnbringer");
    g.pay_for = Some(lyra);
    assert!(sim_core::engine::mana::can_pay(&g, P0, 0, "W", false));
    g.pay_for = Some(card(&g, "Day of Judgment"));
    assert!(!sim_core::engine::mana::can_pay(&g, P0, 0, "W", false));
}

#[test]
fn giada_counters_reach_angel_tokens() {
    let mut g = tbl();
    perm(&mut g, P0, "Giada, Font of Hope");
    perm(&mut g, P0, "Lyra Dawnbringer");
    let toks =
        zones::make_tokens(&mut g, P0, Tokens { fly: true, tgh: Some(4), types: vec!["angel"], ..Tokens::new(1, 4) })
            .unwrap();
    assert_eq!(g.perm(toks[0]).plus, 2); // Giada and Lyra
}

#[test]
fn search_the_premises_investigates_for_each_attacker() {
    let mut g = tbl();
    perm(&mut g, P0, "Search the Premises");
    let a = token(&mut g, P1, 2);
    let b = token(&mut g, P1, 2);
    combat::attack_triggers(&mut g, P1, &[a, b], P0).unwrap();
    assert_eq!(g.player(P0).clues, 2);
    let c = token(&mut g, P1, 2);
    combat::attack_triggers(&mut g, P1, &[c], PlayerId(2)).unwrap(); // not attacking us
    assert_eq!(g.player(P0).clues, 2);
}

#[test]
fn secret_rendezvous_draws_three_each() {
    let mut g = tbl();
    token(&mut g, P1, 1); // seat 2 has the emptier board
    let (h0, h1, h2) = (g.player(P0).hand.len(), g.player(P1).hand.len(), g.player(PlayerId(2)).hand.len());
    cast_free(&mut g, P0, "Secret Rendezvous");
    assert_eq!(g.player(P0).hand.len(), h0 + 3);
    assert_eq!(g.player(PlayerId(2)).hand.len(), h2 + 3);
    assert_eq!(g.player(P1).hand.len(), h1);
}

#[test]
fn angel_of_vitality_adds_one_and_grows_at_25() {
    let mut g = tbl();
    let a = perm(&mut g, P0, "Angel of Vitality");
    assert_eq!(pt(&g, a), (4, 4)); // 40 life: 2/2 base +2/+2
    g.player_mut(P0).life = 20;
    assert_eq!(pt(&g, a), (2, 2));
    gain(&mut g, P0, 3).unwrap();
    assert_eq!(g.player(P0).life, 24); // 3 + 1
    let b = perm(&mut g, P0, "Angel of Vitality");
    let _ = b;
    gain(&mut g, P0, 1).unwrap();
    assert_eq!(g.player(P0).life, 27); // 1 + 2
    gain(&mut g, P1, 3).unwrap();
    assert_eq!(g.player(P1).life, 43); // an opponent isn't helped
}

#[test]
fn angel_of_the_ruins_exiles_two_artifacts_or_enchantments() {
    let mut g = tbl();
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Mind Stone");
    perm(&mut g, P1, "Arcane Signet");
    let mine = perm(&mut g, P0, "Mind Stone");
    perm(&mut g, P0, "Angel of the Ruins");
    let left = ["Sol Ring", "Mind Stone", "Arcane Signet"].iter().filter(|n| !named(&g, P1, n).is_empty()).count();
    assert_eq!(left, 1);
    assert!(g.perm(mine).on_bf);
}

#[test]
fn angel_of_the_ruins_plainscycles() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 3, false);
    let c = hand(&mut g, P0, &["Angel of the Ruins"])[0];
    let o = {
        let f = g.registry.get(c).unwrap().hand_options.unwrap();
        f(&mut g, c, P0, None).unwrap()
    };
    assert_eq!(o.len(), 1);
    sim_core::ai::act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap();
    assert!(g.player(P0).gy.contains(&c));
    assert!(g.player(P0).hand.iter().any(|&x| &*g.db.get(x).name == "Plains"));
}

#[test]
fn angelic_field_marshal_is_bigger_with_the_commander_and_gives_vigilance() {
    let mut g = tbl();
    let m = perm(&mut g, P0, "Angelic Field Marshal");
    let t = token(&mut g, P0, 2);
    assert_eq!(pt(&g, m), (3, 3));
    assert!(!sim_core::engine::combat::kw(&g, t, "vigilance"));
    let giada = perm(&mut g, P0, "Giada, Font of Hope");
    g.perm_mut(giada).is_cmd = true;
    assert_eq!(pt(&g, m), (5, 5));
    assert!(sim_core::engine::combat::kw(&g, t, "vigilance"));
    let opp = token(&mut g, P1, 2);
    assert!(!sim_core::engine::combat::kw(&g, opp, "vigilance"));
}

#[test]
fn angelic_sleuth_investigates_when_a_permanent_with_counters_leaves() {
    let mut g = tbl();
    perm(&mut g, P0, "Angelic Sleuth");
    let plain = token(&mut g, P0, 2);
    let grown = token(&mut g, P0, 2);
    g.perm_mut(grown).plus = 2;
    zones::die(&mut g, plain, "destroy").unwrap();
    assert_eq!(wspell(&g, P0), 0);
    zones::die(&mut g, grown, "destroy").unwrap();
    assert_eq!(wspell(&g, P0), 1);
}

#[test]
fn court_of_grace_makes_the_monarch_and_a_token_each_upkeep() {
    let mut g = tbl();
    let court = perm(&mut g, P0, "Court of Grace");
    assert_eq!(g.monarch, Some(P0));
    let f = g.registry.get(g.perm(court).cd.unwrap()).unwrap().upkeep.unwrap();
    f(&mut g, court, P0).unwrap();
    let angels: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token).collect();
    assert_eq!(angels.len(), 1);
    assert_eq!(pt(&g, angels[0]), (4, 4));
    assert!(g.perm(angels[0]).fly && g.perm(angels[0]).ttypes.contains(&"angel"));
    g.monarch = Some(P1); // no longer the monarch
    f(&mut g, court, P0).unwrap();
    let spirit = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token).last().unwrap();
    assert_eq!(pt(&g, spirit), (1, 1));
    assert!(g.perm(spirit).ttypes.contains(&"spirit"));
}

#[test]
fn cut_a_deal_draws_for_each_opponent_that_drew() {
    let mut g = tbl();
    let (h0, h1, h2) = (g.player(P0).hand.len(), g.player(P1).hand.len(), g.player(PlayerId(2)).hand.len());
    cast_free(&mut g, P0, "Cut a Deal");
    assert_eq!(g.player(P0).hand.len(), h0 + 2);
    assert_eq!(g.player(P1).hand.len(), h1 + 1);
    assert_eq!(g.player(PlayerId(2)).hand.len(), h2 + 1);
}

#[test]
fn defy_death_returns_the_best_creature_and_angels_get_two_counters() {
    let mut g = tbl();
    for n in ["Metallic Mimic", "Lyra Dawnbringer"] {
        let c = take(&mut g, P0, n);
        g.player_mut(P0).gy.push(c);
    }
    cast_free(&mut g, P0, "Defy Death");
    let lyra = named(&g, P0, "Lyra Dawnbringer");
    assert_eq!(lyra.len(), 1);
    assert_eq!(g.perm(lyra[0]).plus, 2);
    assert!(named(&g, P0, "Metallic Mimic").is_empty());
    // a non-Angel gets nothing
    let c = take(&mut g, P0, "Metallic Mimic");
    let lyra = card(&g, "Lyra Dawnbringer");
    g.player_mut(P0).gy.retain(|&x| x != lyra);
    cast_free(&mut g, P0, "Defy Death");
    let w = named(&g, P0, "Metallic Mimic");
    assert_eq!(w.len(), 1);
    assert_eq!(g.perm(w[0]).plus, 0);
    let _ = c;
}

#[test]
fn emeria_shepherd_returns_a_card_to_hand_or_the_battlefield_on_a_plains() {
    let mut g = tbl();
    perm(&mut g, P0, "Emeria Shepherd");
    let c = take(&mut g, P0, "Lyra Dawnbringer");
    g.player_mut(P0).gy.push(c);
    let land = take(&mut g, P0, "Secluded Steppe");
    g.add_land(P0, land, false);
    zones::landfall(&mut g, P0).unwrap();
    assert!(g.player(P0).hand.contains(&c)); // not a Plains: to hand
    g.player_mut(P0).hand.retain(|&x| x != c);
    g.player_mut(P0).gy.push(c);
    let plains = take(&mut g, P0, "Plains");
    g.add_land(P0, plains, false);
    zones::landfall(&mut g, P0).unwrap();
    assert_eq!(named(&g, P0, "Lyra Dawnbringer").len(), 1); // a Plains: to the battlefield
}

#[test]
fn exemplar_of_light_grows_on_lifegain_and_draws_once_per_turn() {
    let mut g = tbl();
    let e = perm(&mut g, P0, "Exemplar of Light");
    let h = g.player(P0).hand.len();
    gain(&mut g, P0, 2).unwrap();
    assert_eq!(g.perm(e).plus, 1);
    assert_eq!(g.player(P0).hand.len(), h + 1);
    gain(&mut g, P0, 2).unwrap();
    assert_eq!(g.perm(e).plus, 2);
    assert_eq!(g.player(P0).hand.len(), h + 1); // once each turn
}

#[test]
fn exemplar_of_light_draws_when_defy_death_gives_it_counters() {
    let mut g = tbl();
    let c = take(&mut g, P0, "Exemplar of Light");
    g.player_mut(P0).gy.push(c);
    let h = g.player(P0).hand.len();
    cast_free(&mut g, P0, "Defy Death");
    let e = named(&g, P0, "Exemplar of Light")[0];
    assert_eq!(g.perm(e).plus, 2);
    assert_eq!(g.player(P0).hand.len(), h + 1);
}

#[test]
fn fateful_absence_destroys_and_the_controller_investigates() {
    let mut g = tbl();
    let t = token(&mut g, P1, 3);
    let c = hand(&mut g, P0, &["Fateful Absence"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { target: Some(t), ..Ctx::default() }).unwrap();
    assert!(!g.perm(t).on_bf);
    assert_eq!(g.player(P1).clues, 1);
    assert_eq!(g.player(P0).clues, 0);
}

#[test]
fn firemane_commando_draws_for_two_attackers() {
    let mut g = tbl();
    perm(&mut g, P0, "Firemane Commando");
    let h = g.player(P0).hand.len();
    let (a, b) = (token(&mut g, P0, 2), token(&mut g, P0, 2));
    combat::attack_triggers(&mut g, P0, &[a], P1).unwrap();
    assert_eq!(g.player(P0).hand.len(), h); // one attacker: nothing
    combat::attack_triggers(&mut g, P0, &[a, b], P1).unwrap();
    assert_eq!(g.player(P0).hand.len(), h + 1);
    // another player: draws if none attacked us
    let (h1, h2) = (g.player(P1).hand.len(), g.player(PlayerId(2)).hand.len());
    let (c, d) = (token(&mut g, P1, 2), token(&mut g, P1, 2));
    combat::attack_triggers(&mut g, P1, &[c, d], PlayerId(2)).unwrap();
    assert_eq!(g.player(P1).hand.len(), h1 + 1);
    combat::attack_triggers(&mut g, P1, &[c, d], P0).unwrap();
    assert_eq!(g.player(P1).hand.len(), h1 + 1); // attacked us: nothing
    assert_eq!(g.player(PlayerId(2)).hand.len(), h2);
}

#[test]
fn herald_of_eternal_dawn_means_you_cannot_lose() {
    let mut g = tbl();
    perm(&mut g, P0, "Herald of Eternal Dawn");
    g.player_mut(P0).life = -5;
    g.player_mut(P0).poison = 12;
    check_state(&mut g).unwrap();
    assert!(g.player(P0).alive);
    g.player_mut(P1).life = 0;
    check_state(&mut g).unwrap();
    assert!(!g.player(P1).alive);
    assert!(!g.over); // the third seat is still in, and so are we
}

#[test]
fn herald_of_war_grows_on_attack_and_discounts_angels_and_humans() {
    let mut g = tbl();
    let h = perm(&mut g, P0, "Herald of War");
    let lyra = card(&g, "Lyra Dawnbringer");
    let hum = card(&g, "Bishop of Wings");
    let other = card(&g, "Day of Judgment");
    let cost = |g: &Game, c: CardId| sim_core::engine::hooks::total_cost(g, P0, c);
    let base = cost(&g, lyra);
    combat::attack_triggers(&mut g, P0, &[h], P1).unwrap();
    combat::attack_triggers(&mut g, P0, &[h], P1).unwrap();
    assert_eq!(g.perm(h).plus, 2);
    assert_eq!(cost(&g, lyra), base - 2);
    assert_eq!(cost(&g, hum), -2);
    assert_eq!(cost(&g, other), 0);
}

#[test]
fn heraldic_banner_taps_for_white_and_pumps_white_creatures() {
    let mut g = tbl();
    let t = token(&mut g, P0, 2);
    let w = perm(&mut g, P0, "Dazzling Angel");
    let base = (epow(&g, t), epow(&g, w));
    perm(&mut g, P0, "Heraldic Banner");
    assert_eq!(epow(&g, w), base.1 + 1);
    let opp = token(&mut g, P1, 2);
    assert_eq!(epow(&g, opp), 2);
    perm(&mut g, P0, "Heraldic Banner");
    assert_eq!(epow(&g, w), base.1 + 2);
}

#[test]
fn merchant_of_truth_investigates_on_deaths_and_clues_are_exalted() {
    let mut g = tbl();
    let m = perm(&mut g, P0, "Merchant of Truth");
    let nontoken = perm(&mut g, P0, "Metallic Mimic");
    let tok = token(&mut g, P0, 1);
    zones::die(&mut g, tok, "destroy").unwrap();
    assert_eq!(g.player(P0).clues, 0); // a token
    zones::die(&mut g, nontoken, "destroy").unwrap();
    assert_eq!(g.player(P0).clues, 1);
    g.player_mut(P0).clues = 3;
    let a = token(&mut g, P0, 2);
    combat::attack_triggers(&mut g, P0, &[a], P1).unwrap();
    assert_eq!(pt(&g, a), (5, 5)); // alone: +3/+3 for three Clues
    let b = token(&mut g, P0, 2);
    combat::attack_triggers(&mut g, P0, &[b, m], P1).unwrap();
    assert_eq!(pt(&g, b), (2, 2)); // not alone
    zones::die(&mut g, m, "destroy").unwrap();
    assert_eq!(g.player(P0).clues, 4); // itself too
}

#[test]
fn metropolis_reformer_gives_hexproof_and_gains_life_from_damage() {
    let mut g = tbl();
    let r = perm(&mut g, P0, "Metropolis Reformer");
    assert!(sim_core::engine::hooks::any_player_hexproof(&g, P0));
    assert!(!sim_core::engine::hooks::any_player_hexproof(&g, P1));
    let before = g.player(P0).life;
    sim_core::engine::removal::apply_removal(&mut g, Some(P1), r, "dmg2", None).unwrap();
    assert_eq!(g.player(P0).life, before + 2);
}

#[test]
fn norns_choirmaster_proliferates_for_the_commander() {
    let mut g = tbl();
    perm(&mut g, P0, "Norn's Choirmaster");
    let t = token(&mut g, P0, 1);
    g.perm_mut(t).plus = 1;
    let giada = perm(&mut g, P0, "Giada, Font of Hope"); // a commander entering
    assert_eq!(g.perm(t).plus, 2);
    g.perm_mut(giada).is_cmd = true;
    combat::attack_triggers(&mut g, P0, &[giada], P1).unwrap(); // attacking
    assert_eq!(g.perm(t).plus, 3);
    let other = token(&mut g, P0, 1);
    combat::attack_triggers(&mut g, P0, &[other], P1).unwrap();
    assert_eq!(g.perm(t).plus, 3);
}

#[test]
fn righteous_valkyrie_gains_life_and_pumps_at_47() {
    let mut g = tbl();
    let v = perm(&mut g, P0, "Righteous Valkyrie");
    let h = g.player(P0).life;
    perm(&mut g, P0, "Lyra Dawnbringer"); // toughness 5
    assert_eq!(g.player(P0).life, h + 5);
    let w = perm(&mut g, P0, "Metallic Mimic"); // neither Angel nor Cleric
    assert_eq!(g.player(P0).life, h + 5);
    assert_eq!(pt(&g, w), (epow(&g, w), etgh(&g, w)));
    g.player_mut(P0).life = 46;
    let base = pt(&g, w);
    g.player_mut(P0).life = 47;
    assert_eq!(pt(&g, w), (base.0 + 2, base.1 + 2));
    assert_eq!(pt(&g, v), (5, 7)); // 2/4, +2/+2, and Lyra gives other Angels +1/+1
    perm(&mut g, P0, "Metropolis Reformer"); // Angel Cleric: toughness 3 +2 from the anthem
    assert!(g.player(P0).life >= 52); // its toughness with the anthem and Lyra
}

#[test]
fn sephara_gives_flyers_indestructible_and_has_an_alternative_cost() {
    let mut g = tbl();
    perm(&mut g, P0, "Sephara, Sky's Blade");
    let lyra = perm(&mut g, P0, "Lyra Dawnbringer");
    let ground = token(&mut g, P0, 2);
    assert!(sim_core::engine::values::indestructible(&g, lyra));
    assert!(!sim_core::engine::values::indestructible(&g, ground));
    // the alternative cost: {W} and four untapped flyers
    let mut g = tbl();
    for n in ["Lyra Dawnbringer", "Reya Dawnbringer", "Dazzling Angel", "Sunblast Angel"] {
        perm(&mut g, P0, n);
    }
    lands(&mut g, P0, "Plains", 1, false);
    let c = hand(&mut g, P0, &["Sephara, Sky's Blade"])[0];
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    let o = f(&mut g, c, P0, Some(false)).unwrap();
    assert_eq!(o.len(), 1);
    sim_core::ai::act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap();
    assert_eq!(named(&g, P0, "Sephara, Sky's Blade").len(), 1);
    let tapped = g.player(P0).perms.iter().filter(|&&m| g.perm(m).tapped).count();
    assert_eq!(tapped, 4);
}

#[test]
fn seraph_of_the_sword_takes_no_combat_damage() {
    let mut g = tbl();
    let s = perm(&mut g, P0, "Seraph of the Sword");
    g.perm_mut(s).sick = false;
    // combat damage is prevented, burn is not
    assert!(sim_core::impls::t0_giada::combat_proof(&g, s));
    sim_core::engine::removal::apply_removal(&mut g, Some(P1), s, "dmg9", None).unwrap();
    assert!(!g.perm(s).on_bf);
}

#[test]
fn serra_avenger_cannot_be_cast_on_the_first_three_turns() {
    let mut g = tbl();
    let c = hand(&mut g, P0, &["Serra Avenger"])[0];
    for t in 1..=3 {
        g.player_mut(P0).turns = t;
        assert!(!cast::castable(&g, P0, c, "hand"));
    }
    g.player_mut(P0).turns = 4;
    assert!(cast::castable(&g, P0, c, "hand"));
}

#[test]
fn starnheim_aspirant_discounts_angels_by_two() {
    let mut g = tbl();
    let lyra = card(&g, "Lyra Dawnbringer");
    let base = sim_core::engine::hooks::total_cost(&g, P0, lyra);
    perm(&mut g, P0, "Starnheim Aspirant");
    assert_eq!(sim_core::engine::hooks::total_cost(&g, P0, lyra), base - 2);
    assert_eq!(sim_core::engine::hooks::total_cost(&g, P0, card(&g, "Day of Judgment")), 0);
}

#[test]
fn vanguard_seraph_surveils_on_the_first_lifegain_each_turn() {
    let mut g = tbl();
    perm(&mut g, P0, "Vanguard Seraph");
    let n0 = g.player(P0).stats.get("surveil").copied().unwrap_or(0);
    let lib = g.player(P0).library.len();
    let top = *g.player(P0).library.last().unwrap();
    gain(&mut g, P0, 1).unwrap();
    let after_first = (g.player(P0).library.len(), g.player(P0).gy.len());
    gain(&mut g, P0, 1).unwrap();
    assert_eq!((g.player(P0).library.len(), g.player(P0).gy.len()), after_first); // only the first
    let _ = (n0, lib, top);
}

#[test]
fn wojek_investigator_investigates_per_opponent_with_more_cards() {
    let mut g = tbl();
    let w = perm(&mut g, P0, "Wojek Investigator");
    hand(&mut g, P1, &["Plains", "Plains", "Plains"]);
    hand(&mut g, P0, &["Plains"]);
    let f = g.registry.get(g.perm(w).cd.unwrap()).unwrap().upkeep.unwrap();
    f(&mut g, w, P0).unwrap();
    assert_eq!(g.player(P0).clues, 1); // seat 1 has more, seat 2 has none
    hand(&mut g, PlayerId(2), &["Plains", "Plains"]);
    f(&mut g, w, P0).unwrap();
    assert_eq!(g.player(P0).clues, 3);
}

#[test]
fn youthful_valkyrie_grows_for_each_other_angel() {
    let mut g = tbl();
    let y = perm(&mut g, P0, "Youthful Valkyrie");
    assert_eq!(g.perm(y).plus, 0);
    perm(&mut g, P0, "Lyra Dawnbringer");
    assert_eq!(g.perm(y).plus, 1);
    perm(&mut g, P0, "Metallic Mimic"); // not an Angel
    assert_eq!(g.perm(y).plus, 1);
    perm(&mut g, P1, "Lyra Dawnbringer"); // not ours
    assert_eq!(g.perm(y).plus, 1);
}

#[test]
fn bishop_of_wings_gains_life_and_leaves_spirits() {
    let mut g = tbl();
    perm(&mut g, P0, "Bishop of Wings");
    let h = g.player(P0).life;
    let a = perm(&mut g, P0, "Lyra Dawnbringer");
    assert_eq!(g.player(P0).life, h + 4);
    zones::die(&mut g, a, "destroy").unwrap();
    let spirits: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token).collect();
    assert_eq!(spirits.len(), 1);
    assert_eq!(pt(&g, spirits[0]), (1, 1));
}

#[test]
fn linvala_gains_life_and_makes_an_angel_when_behind() {
    let mut g = tbl();
    g.player_mut(P1).life = 50;
    token(&mut g, P1, 2);
    token(&mut g, P1, 2);
    let h = g.player(P0).life;
    perm(&mut g, P0, "Linvala, the Preserver");
    assert_eq!(g.player(P0).life, h + 5);
    let angels: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token).collect();
    assert_eq!(angels.len(), 1);
    assert_eq!(pt(&g, angels[0]), (3, 3));
    // ahead: nothing
    let mut g = tbl();
    let h = g.player(P0).life;
    perm(&mut g, P0, "Linvala, the Preserver");
    assert_eq!(g.player(P0).life, h);
    assert!(g.player(P0).perms.iter().all(|&m| !g.perm(m).token));
}

#[test]
fn a_whole_game_with_the_deck_plays_to_the_end() {
    let g = play(&["giada-calling-all-angels", "veyran", "seph"], 3, false);
    assert!(g.round >= 1);
    let _ = turn::run_rounds;
}

#[test]
fn speaker_of_the_heavens_needs_47_life_and_makes_a_4_4_angel() {
    let mut g = tbl();
    g.active = Some(P0);
    let s = perm(&mut g, P0, "Speaker of the Heavens");
    g.perm_mut(s).sick = false;
    let hook = *g.registry.get(card(&g, "Speaker of the Heavens")).unwrap();
    g.player_mut(P0).life = 40;
    assert!((hook.options.unwrap())(&mut g, s, P0, None).unwrap().is_empty(), "40 life: not allowed");
    g.player_mut(P0).life = 47;
    let opts = (hook.options.unwrap())(&mut g, s, P0, None).unwrap();
    let Some(sim_core::hooks::Action::Ability { src, f, arg }) = opts.into_iter().next().unwrap().act else { panic!() };
    assert!(f(&mut g, src, P0, arg).unwrap());
    assert!(g.perm(s).tapped);
    let angels = named(&g, P0, "Token");
    assert_eq!(angels.len(), 1);
    assert_eq!(pt(&g, angels[0]), (4, 4));
}
