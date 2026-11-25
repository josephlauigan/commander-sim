//! Arzakon ({W}{U}{B}{R}{G}, Legendary Planeswalker, loyalty 4) against its Oracle text (data/scryfall_cache.json):
//! "+2: Arzakon deals 3 damage to any other target. -3: Create a Black Lotus token. (It's a {0} artifact with "{T},
//! Sacrifice this token: Add three mana of any one color.") -7: Each player shuffles their hand and graveyard into their
//! library, then draws seven cards. Arzakon can be your commander." The expected results come from that text and the
//! comprehensive rules, not from what the engine did before. Card code: impls/arzakon.rs.

mod play_common;
use play_common::*;
use sim_core::ai::act::perform;
use sim_core::engine::mana::{self, total_mana};
use sim_core::engine::values::{commander_out, epow, etgh};
use sim_core::engine::{cast, combat, stack, zones};
use sim_core::hooks::Opt;
use sim_core::human::Target;
use sim_core::human::human::{AbilityKind, abilities_of, human_main};
use sim_core::impls::arzakon::plus_targets;
use sim_core::impls::model_gaps_a4::{Tgt, ping, ping_legal};
use sim_core::state::Ctx;
use std::sync::{Arc, Mutex};

const ARZ: &str = "Arzakon";
const LOTUS: &str = "Black Lotus";
const KINNAN: &str = "Kinnan, Bonder Prodigy";
const IKORIA: &str = "Invasion of Ikoria // Zilortha, Apex of Ikoria";

/// P0 (Arzakon's seat, Arzakon its commander in the command zone), P1 and P2
fn tbl() -> Game {
    let mut g = table(&["arzakon", "yshtola", "karn"]);
    set_commander(&mut g, P0, ARZ);
    g
}

/// a table where P0's library and hand hold known nonland cards (so a draw is a draw), and Arzakon is out at loyalty `l`
fn walker(g: &mut Game, l: i32) -> PermId {
    let w = perm(g, P0, ARZ);
    g.perm_mut(w).loyalty = Some(l);
    g.perm_mut(w).is_cmd = true;
    w
}

fn options(g: &mut Game, w: PermId) -> Vec<Opt> {
    let f = g.registry.get(g.perm(w).cd.unwrap()).and_then(|i| i.options).expect("Arzakon has options");
    f(g, w, P0, Some(false)).unwrap()
}

fn labels(g: &mut Game, w: PermId) -> Vec<String> {
    options(g, w).into_iter().map(|o| o.label).collect()
}

fn find(g: &mut Game, w: PermId, word: &str) -> Option<Opt> {
    options(g, w).into_iter().find(|o| o.label.contains(word))
}

fn ai_use(g: &mut Game, w: PermId, word: &str) {
    let a = find(g, w, word).unwrap_or_else(|| panic!("no AI option {word:?}: {:?}", labels(g, w))).act.unwrap();
    assert!(perform(g, P0, &a).unwrap());
}

/// the person's loyalty ability (no question asked): None, or why not
fn human_use(g: &mut Game, w: PermId, word: &str) -> Option<String> {
    let (_, ability) = abilities_of(g, P0, w).unwrap().into_iter().find(|a| a.0.starts_with(word)).unwrap();
    match ability {
        AbilityKind::Extra(f) => f(g, P0, w).unwrap(),
        _ => panic!("expected the practice-mode card implementation"),
    }
}

fn colour_lands(g: &mut Game, p: PlayerId) {
    for b in ["Plains", "Island", "Swamp", "Mountain", "Forest"] {
        lands(g, p, b, 1, false);
    }
}

fn untap(g: &mut Game, p: PlayerId) {
    for l in g.player(p).lands.clone() {
        g.perm_mut(l).tapped = false;
    }
}

fn lotus_tokens(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).token && &*g.perm(m).name == LOTUS && g.perm(m).on_bf)
        .collect()
}

fn burn(g: &mut Game, w: PermId, n: i32) {
    sim_core::human::choices::apply_damage(g, P1, Target::Perm(w), n, "Lightning Bolt", "burn").unwrap();
}

// ================================================================== 1. the command zone
#[test]
fn it_is_cast_from_the_command_zone_with_four_loyalty_and_a_tax_of_two() {
    let mut g = tbl();
    colour_lands(&mut g, P0);
    let cmd = g.player(P0).cmd;
    assert_eq!(&*g.db.get(cmd).name, ARZ);
    let (gn, pi) = mana::cost_of(&g, P0, cmd);
    assert_eq!(gn + pi.len() as u32, 5, "{{W}}{{U}}{{B}}{{R}}{{G}}");
    assert!(cast::cast_card(&mut g, P0, cmd, "cmd", Ctx::default()).unwrap());
    let w = perms_named(&g, P0, ARZ)[0];
    assert_eq!(g.perm(w).loyalty, Some(4));
    assert!(g.perm(w).is_cmd, "the commander on the battlefield");
    assert!(!g.player(P0).cmd_in_zone);
    assert_eq!(g.player(P0).tax, 2);
    assert!(commander_out(&g, P0));
}

#[test]
fn it_returns_to_the_command_zone_from_burn_its_own_minus_three_and_combat_and_costs_two_more_each_time() {
    let mut g = tbl();
    colour_lands(&mut g, P0);
    lands(&mut g, P0, "Plains", 10, false);
    let cmd = g.player(P0).cmd;
    let cast_it = |g: &mut Game, n: u32| -> PermId {
        untap(g, P0);
        let (gn, pi) = mana::cost_of(g, P0, cmd);
        assert_eq!(gn + pi.len() as u32, 5 + 2 * n, "cast {n}: the commander tax");
        assert!(cast::cast_card(g, P0, cmd, "cmd", Ctx::default()).unwrap());
        assert_eq!(g.player(P0).tax, 2 * (n + 1));
        perms_named(g, P0, ARZ)[0]
    };
    // burn
    let w = cast_it(&mut g, 0);
    burn(&mut g, w, 4);
    assert!(!g.perm(w).on_bf && g.player(P0).cmd_in_zone, "four damage: gone to the command zone");
    assert!(!g.player(P0).gy.contains(&cmd));
    // its own -3 at loyalty 3 (the token still appears)
    let w = cast_it(&mut g, 1);
    g.perm_mut(w).loyalty = Some(3);
    assert_eq!(human_use(&mut g, w, "-3"), None);
    assert!(!g.perm(w).on_bf && g.player(P0).cmd_in_zone, "-3 at loyalty 3: 0 loyalty, back to the command zone");
    assert_eq!(lotus_tokens(&g, P0).len(), 1, "the ability still resolves");
    // combat damage
    let w = cast_it(&mut g, 2);
    let attacker = token(&mut g, P1, 6);
    g.active = Some(P1);
    combat::resolve_combat(&mut g, P1, &[attacker], P0, &[]).unwrap();
    assert!(!g.perm(w).on_bf && g.player(P0).cmd_in_zone, "combat damage in excess of its loyalty");
    assert_eq!(g.player(P0).life, 40, "the attacker went after Arzakon");
    // and it is still castable, at +6
    let _ = cast_it(&mut g, 3);
}

#[test]
fn a_countered_arzakon_returns_to_the_command_zone() {
    let mut g = tbl();
    colour_lands(&mut g, P0);
    hand(&mut g, P1, &["Counterspell"]);
    pool(&mut g, P1, &[('U', 2)]);
    seat(&mut g, P1, vec![act(json!({"do": "cast", "card": 0})), by_text("Arzakon"), pass()]);
    let cmd = g.player(P0).cmd;
    let _ = cast::cast_card(&mut g, P0, cmd, "cmd", Ctx::default()).unwrap();
    assert!(perms_named(&g, P0, ARZ).is_empty(), "countered");
    assert!(g.player(P0).cmd_in_zone, "a commander spell that is countered goes back to the command zone");
    assert_eq!(g.player(P0).tax, 2, "the tax was paid");
}

#[test]
fn a_planeswalker_commander_deals_no_commander_damage() {
    // CR 903.10a: only combat damage dealt by a commander creature counts, and Arzakon is never a creature
    let mut g = tbl();
    let w = walker(&mut g, 4);
    assert!(!g.is_creature(w));
    assert_eq!(g.player(P1).cmd_dmg[0], 0);
}

// ================================================================== 2. +2
#[test]
fn plus_two_kills_an_opponent_at_three_life() {
    let mut g = tbl();
    let w = walker(&mut g, 4);
    g.player_mut(P1).life = 3;
    perm(&mut g, P2, KINNAN); // a tempting creature: lethal comes first
    ai_use(&mut g, w, "+2");
    assert!(!g.player(P1).alive, "three damage to a player at 3 life");
    assert_eq!(g.perm(w).loyalty, Some(6));
}

#[test]
fn plus_two_kills_a_two_two_commander_rather_than_going_to_the_face() {
    let mut g = tbl();
    let w = walker(&mut g, 4);
    let k = perm(&mut g, P1, KINNAN);
    g.perm_mut(k).is_cmd = true;
    assert_eq!((epow(&g, k), etgh(&g, k)), (2, 2));
    ai_use(&mut g, w, "+2");
    assert!(!g.perm(k).on_bf, "Kinnan took three");
    assert_eq!(g.player(P1).life, 40, "not the face");
    assert_eq!(g.perm(w).loyalty, Some(6));
}

#[test]
fn plus_two_goes_to_the_face_when_nothing_is_worth_killing() {
    let mut g = tbl();
    let w = walker(&mut g, 4);
    g.player_mut(P1).life = 30;
    ai_use(&mut g, w, "+2");
    assert_eq!(g.player(P1).life + g.player(P2).life, 30 + 40 - 3, "one opponent's face took three");
    assert_eq!(g.player(P0).life, 40, "never its own controller");
}

#[test]
fn plus_two_hurts_a_planeswalker_it_does_not_destroy_it() {
    // guards against `apply_removal(.., "dmg3")`, which would destroy a planeswalker whatever its loyalty
    let mut g = tbl();
    let w = walker(&mut g, 4);
    let pw = perm(&mut g, P1, "Teferi, Hero of Dominaria");
    g.perm_mut(pw).loyalty = Some(5);
    ping(&mut g, P0, w, Tgt::Perm(pw), 3).unwrap();
    assert!(g.perm(pw).on_bf, "alive");
    assert_eq!(g.perm(pw).loyalty, Some(2), "5 - 3");
    ping(&mut g, P0, w, Tgt::Perm(pw), 3).unwrap();
    assert!(!g.perm(pw).on_bf, "3 more is enough");
}

#[test]
fn the_ai_aims_at_a_planeswalker_it_can_kill() {
    let mut g = tbl();
    let w = walker(&mut g, 4);
    let pw = perm(&mut g, P1, "Teferi, Hero of Dominaria");
    g.perm_mut(pw).loyalty = Some(3);
    let o = find(&mut g, w, "+2").unwrap();
    assert!(o.label.contains("Teferi"), "{}", o.label);
    ai_use(&mut g, w, "+2");
    assert!(!g.perm(pw).on_bf, "three loyalty, three damage");
}

#[test]
fn a_creature_with_hexproof_or_protection_is_not_a_target_and_never_itself() {
    let mut g = tbl();
    let w = walker(&mut g, 4);
    let hex = perm(&mut g, P1, "Sigarda, Host of Herons");
    let open = perm(&mut g, P1, KINNAN);
    let legal = plus_targets(&g, P0, w);
    assert!(!legal.contains(&Tgt::Perm(hex)), "hexproof");
    assert!(legal.contains(&Tgt::Perm(open)));
    assert!(!legal.contains(&Tgt::Perm(w)), "'any OTHER target'");
    assert!(legal.contains(&Tgt::Player(P0)), "a person may point it at themselves: legal, never the AI's choice");
    assert!(!ping_legal(&g, P0, w, Tgt::Perm(hex)));
    // protection from the source's colours (Arzakon is all five): protection from red
    let pro = perm(&mut g, P2, "Kor Firewalker");
    assert!(!plus_targets(&g, P0, w).contains(&Tgt::Perm(pro)), "protection from red: Arzakon is red");
}

#[test]
fn a_target_that_leaves_in_response_takes_no_damage_and_the_loyalty_is_paid_anyway() {
    let mut g = tbl();
    let w = walker(&mut g, 4);
    let k = perm(&mut g, P1, KINNAN);
    g.perm_mut(k).is_cmd = true;
    let backup = perm(&mut g, P1, "Serra Angel"); // 4 toughness: not a +2 target itself
    hand(&mut g, P1, &["Swords to Plowshares"]);
    pool(&mut g, P1, &[('W', 1)]);
    let h = seat(&mut g, P1, vec![act(json!({"do": "cast", "card": 0})), by_text("Kinnan"), pass()]);
    ai_use(&mut g, w, "+2");
    assert!(h.told_kind("invalid").is_empty(), "{:?}", h.told());
    assert!(!g.perm(k).on_bf, "the response exiled it");
    assert_eq!(g.player(P1).life, 42, "no new target: only Swords' two life, no damage to the face");
    assert_eq!(g.player(P2).life, 40, "and none to the other face");
    assert!(g.perm(backup).on_bf, "and no other creature");
    assert_eq!(g.perm(w).loyalty, Some(6), "the loyalty was paid");
}

#[test]
fn a_battle_can_be_hit_and_loses_defense_counters() {
    let mut g = tbl();
    let w = walker(&mut g, 4);
    let spell = take(&mut g, P1, IKORIA);
    stack_resolve(&mut g, spell);
    let battle = g.player(P1).perms.iter().copied().find(|&m| g.perm(m).cd == Some(spell)).unwrap();
    let before = sim_core::impls::card_models3::defense(&g, battle);
    assert!(before >= 3, "{before}");
    assert!(plus_targets(&g, P0, w).contains(&Tgt::Perm(battle)), "any other target includes a battle");
    ping(&mut g, P0, w, Tgt::Perm(battle), 3).unwrap();
    assert_eq!(sim_core::impls::card_models3::defense(&g, battle), before - 3);
    assert!(g.perm(battle).on_bf, "not yet defeated");
}

fn stack_resolve(g: &mut Game, spell: sim_core::ids::CardId) {
    cast::resolve(g, P1, spell, &Ctx { x: 1, ..Default::default() }, "hand").unwrap();
}

// ================================================================== 3. -3
#[test]
fn minus_three_makes_a_black_lotus_token_that_pays_three_of_one_colour() {
    let mut g = tbl();
    let w = walker(&mut g, 7);
    assert_eq!(human_use(&mut g, w, "-3"), None);
    assert_eq!(g.perm(w).loyalty, Some(4));
    let t = lotus_tokens(&g, P0);
    assert_eq!(t.len(), 1);
    let t = t[0];
    let c = g.db.get(g.perm(t).cd.unwrap());
    assert!(c.types.has(sim_core::cards::Types::ARTIFACT) && c.cmc == 0, "a {{0}} artifact");
    assert!(sim_core::engine::values::colors_of(&g, t).is_empty(), "colourless");
    assert_eq!(total_mana(&g, P0, false), 3);
    // one colour only: {U}{U}{U} yes, {U}{B}{R} no
    assert!(!mana::can_pay(&g, P0, 0, "UBR", false), "all three mana are one colour");
    assert!(mana::can_pay(&g, P0, 0, "UUU", false));
    assert!(mana::pay(&mut g, P0, 0, "UUU", false).unwrap());
    assert!(!g.perm(t).on_bf, "sacrificed as it was tapped");
    assert_eq!(total_mana(&g, P0, false), 0);
}

#[test]
fn the_token_counts_as_an_artifact_for_mox_opal_and_tezzeret() {
    let mut g = tbl();
    let w = walker(&mut g, 7);
    perm(&mut g, P0, "Sol Ring");
    let opal = perm(&mut g, P0, "Mox Opal");
    let tez = perm(&mut g, P0, "Tezzeret, Cruel Captain");
    let l = g.perm(tez).loyalty.unwrap();
    assert_eq!(total_mana(&g, P0, false), 2 /* Sol Ring */, "no metalcraft with two artifacts: Opal is off");
    assert_eq!(human_use(&mut g, w, "-3"), None);
    assert_eq!(g.perm(tez).loyalty, Some(l + 1), "an artifact entered under your control");
    let _ = opal;
    // Sol Ring 2 + Opal 1 (three artifacts now: metalcraft) + the token's 3
    assert_eq!(total_mana(&g, P0, false), 2 + 1 + 3);
}

#[test]
fn the_token_is_not_a_card_so_the_cali_ban_list_never_sees_it() {
    // the ban list is checked against deck lists by name; a token made in a game is not in any list
    let mut g = tbl();
    g.settings = Arc::new({
        let mut s = (*g.settings).clone();
        s.rules = sim_core::format::Rules { life: 30, first_player_draws: false };
        s
    });
    let w = walker(&mut g, 7);
    ai_use(&mut g, w, "-3");
    assert_eq!(lotus_tokens(&g, P0).len(), 1);
    assert_eq!(total_mana(&g, P0, false), 3);
}

// ================================================================== 4. -7
fn library_len(g: &Game, p: PlayerId) -> usize {
    g.player(p).library.len()
}

#[test]
fn minus_seven_shuffles_hands_and_graveyards_away_and_everyone_draws_seven() {
    let mut g = tbl();
    let w = walker(&mut g, 8);
    hand(&mut g, P0, &["Brainstorm"]);
    hand(&mut g, P1, &["Counterspell", "Swords to Plowshares", "Sol Ring"]);
    let c = card(&g, "Lightning Bolt");
    g.player_mut(P0).gy.push(c);
    let titan = card(&g, "Grave Titan");
    g.player_mut(P1).gy.push(titan);
    let totals: Vec<usize> =
        [P0, P1, P2].iter().map(|&p| library_len(&g, p) + g.player(p).hand.len() + g.player(p).gy.len()).collect();
    assert_eq!(human_use(&mut g, w, "-7"), None);
    assert_eq!(g.perm(w).loyalty, Some(1));
    for (i, &p) in [P0, P1, P2].iter().enumerate() {
        assert_eq!(g.player(p).hand.len(), 7, "{p:?} draws seven");
        assert!(g.player(p).gy.is_empty(), "graveyard shuffled in");
        assert_eq!(library_len(&g, p) + g.player(p).hand.len(), totals[i], "no card lost or made");
    }
}

#[test]
fn minus_seven_returns_a_card_taken_by_opposition_agent_to_its_owner() {
    let mut g = tbl();
    let w = walker(&mut g, 8);
    let c = card(&g, "Mana Leak"); // in none of the three decks
    g.player_mut(P0).hand.push(c); // P0 holds P1's card (Opposition Agent style)
    g.player_mut(P0).stolen.insert(c, P1);
    let before = library_len(&g, P1);
    assert_eq!(human_use(&mut g, w, "-7"), None);
    let owner_has = g.player(P1).hand.contains(&c) || g.player(P1).library.contains(&c);
    let thief_has = g.player(P0).hand.contains(&c) || g.player(P0).library.contains(&c);
    assert!(owner_has && !thief_has, "it goes to its owner's library (and may be drawn)");
    assert!(library_len(&g, P1) + g.player(P1).hand.len() > before, "P1's library got it");
}

#[test]
fn minus_seven_sends_a_commander_in_hand_to_the_command_zone() {
    // CR 903.9b: a commander that would be put into a library goes to the command zone instead
    let mut g = tbl();
    let w = walker(&mut g, 8);
    let c2 = card(&g, "Kyodai, Soul of Kamigawa");
    g.player_mut(P1).cmd = c2; // P1's commander is in P1's hand
    g.player_mut(P1).cmd_in_zone = false;
    g.player_mut(P1).library.retain(|&x| x != c2);
    g.player_mut(P1).hand.push(c2);
    assert_eq!(human_use(&mut g, w, "-7"), None);
    assert!(g.player(P1).cmd_in_zone, "back in the command zone");
    assert!(!g.player(P1).hand.contains(&c2) && !g.player(P1).library.contains(&c2));
    assert_eq!(g.player(P1).hand.len(), 7);
}

#[test]
fn timetwister_still_shuffles_and_draws_seven_and_goes_to_the_graveyard() {
    let mut g = tbl();
    let tt = hand(&mut g, P0, &["Timetwister"])[0];
    let bolt = card(&g, "Lightning Bolt");
    g.player_mut(P0).gy.push(bolt);
    lands(&mut g, P0, "Island", 3, false);
    assert!(cast::cast_card(&mut g, P0, tt, "hand", Ctx::default()).unwrap());
    for p in [P0, P1, P2] {
        assert_eq!(g.player(p).hand.len(), 7, "{p:?}");
    }
    assert_eq!(g.player(P0).gy, vec![tt], "Timetwister alone is in the graveyard; the old graveyard was shuffled in");
}

// ================================================================== 5. the AI's rules
#[test]
fn plus_two_beats_minus_three_when_there_is_a_kill() {
    let mut g = tbl();
    let w = walker(&mut g, 6);
    let k = perm(&mut g, P1, KINNAN);
    g.perm_mut(k).is_cmd = true;
    let mut o = options(&mut g, w);
    o.sort_by(|a, b| b.utility.total_cmp(&a.utility));
    assert!(o[0].label.contains("+2"), "{:?}", o.iter().map(|x| (&x.label, x.utility)).collect::<Vec<_>>());
}

#[test]
fn minus_three_is_chosen_when_its_mana_lets_a_tutor_be_cast() {
    let mut g = tbl();
    let w = walker(&mut g, 6);
    hand(&mut g, P0, &["Demonic Tutor"]); // {1}{B}: payable only with the Lotus (no lands)
    let bank = {
        let mut g2 = tbl();
        let w2 = walker(&mut g2, 6);
        find(&mut g2, w2, "-3").map(|o| o.utility)
    };
    let o = find(&mut g, w, "-3").expect("the Lotus makes the tutor castable");
    assert!(o.utility > bank.unwrap_or(0.0) + 1.0, "{} vs banking {:?}", o.utility, bank);
}

#[test]
fn minus_three_is_not_used_at_three_loyalty_for_a_cheap_play_and_is_banked_only_with_loyalty_to_spare() {
    let mut g = tbl();
    let w = walker(&mut g, 3);
    hand(&mut g, P0, &["Ponder"]);
    assert!(find(&mut g, w, "-3").is_none(), "{:?}", labels(&mut g, w));
    // loyalty 5: the token is banked (3.6)
    let mut g = tbl();
    let w = walker(&mut g, 5);
    let o = find(&mut g, w, "-3").expect("banked");
    assert!(o.utility > 2.0, "{}", o.utility);
    // loyalty 4: after 1, a little
    let mut g = tbl();
    let w = walker(&mut g, 4);
    let o = find(&mut g, w, "-3").expect("banked, small");
    assert!(o.utility < 1.5, "{}", o.utility);
    // loyalty 3, the table about to kill it anyway
    let mut g = tbl();
    let w = walker(&mut g, 3);
    token(&mut g, P1, 6);
    assert!(find(&mut g, w, "-3").is_some(), "dying anyway: take the Lotus");
}

/// the -3 and the +2 as the AI scores them: (the -3's utility if offered, the +2's)
fn minus_and_plus(g: &mut Game, w: PermId) -> (Option<f64>, f64) {
    let plus = find(g, w, "+2").expect("the +2 is always on offer").utility;
    (find(g, w, "-3").map(|o| o.utility), plus)
}

// PROVISIONAL rule (JD, 2026-11-24, option a): the -3 that leaves loyalty 1 or less is taken only for a line, a tutor, the
// commander or one of the plan's engines
#[test]
fn at_four_loyalty_the_ai_takes_the_plus_two_over_a_minus_three_that_only_pays_for_an_arcane_signet() {
    let mut g = tbl();
    let w = walker(&mut g, 4);
    hand(&mut g, P0, &["Arcane Signet"]); // {2}: payable only with the Lotus (no lands)
    let (minus, plus) = minus_and_plus(&mut g, w);
    assert!(minus.is_none_or(|m| m < plus), "-3 {minus:?} against +2 {plus}");
}

#[test]
fn at_four_loyalty_the_ai_still_takes_the_minus_three_for_a_catalogued_line_the_lotus_pays_for() {
    let mut g = tbl();
    let w = walker(&mut g, 4);
    // Heliod, Sun-Crowned out, a Walking Ballista in hand and two Plains: the line needs {W} and {2}{2} for the Ballista (X = 2),
    // five mana, which two lands cannot pay and two lands with the Lotus's three can (a catalogued line, combos::line_enabled_by;
    // the Ballista itself costs nothing at X = 0, so it enables nothing on its own)
    perm(&mut g, P0, "Heliod, Sun-Crowned");
    lands(&mut g, P0, "Plains", 2, false);
    hand(&mut g, P0, &["Walking Ballista"]);
    let (minus, plus) = minus_and_plus(&mut g, w);
    assert!(minus.is_some_and(|m| m > 13.0), "the line is worth 14: -3 {minus:?} against +2 {plus}");
}

#[test]
fn at_four_loyalty_the_ai_still_takes_the_minus_three_for_a_tutor_the_lotus_pays_for() {
    let mut g = tbl();
    let w = walker(&mut g, 4);
    hand(&mut g, P0, &["Demonic Tutor"]);
    let (minus, plus) = minus_and_plus(&mut g, w);
    assert!(minus.is_some_and(|m| m > plus), "-3 {minus:?} against +2 {plus}");
}

#[test]
fn at_four_loyalty_the_ai_still_takes_the_minus_three_for_one_of_the_plans_engines() {
    let mut g = tbl();
    let w = walker(&mut g, 4);
    hand(&mut g, P0, &["Mystic Remora"]); // {U}, in data/plans/arzakon.toml's engines (Rhystic Study left the list 2026-11-24)
    let (minus, plus) = minus_and_plus(&mut g, w);
    assert!(minus.is_some_and(|m| m > plus), "-3 {minus:?} against +2 {plus}");
}

#[test]
fn at_five_and_six_loyalty_the_minus_three_for_an_arcane_signet_is_as_before() {
    for l in [5, 6] {
        let mut g = tbl();
        let w = walker(&mut g, l);
        hand(&mut g, P0, &["Arcane Signet"]);
        let (minus, plus) = minus_and_plus(&mut g, w);
        assert!(minus.is_some_and(|m| m > plus), "loyalty {l}: -3 {minus:?} against +2 {plus}");
    }
}

fn wheel_position() -> (Game, PermId) {
    let mut g = tbl();
    let w = walker(&mut g, 8);
    lands(&mut g, P0, "Plains", 3, false);
    g.player_mut(P0).hand.clear();
    hand(&mut g, P1, &["Brainstorm", "Ponder", "Counterspell", "Sol Ring", "Mana Crypt"]);
    (g, w)
}

#[test]
fn minus_seven_is_offered_with_an_empty_hand_against_a_full_one() {
    let (mut g, w) = wheel_position();
    let o = find(&mut g, w, "-7").expect("offered");
    assert!(o.utility > 5.0, "{}", o.utility);
}

#[test]
fn minus_seven_is_not_offered_against_a_draw_punisher_or_with_even_hands_or_with_a_breach_line() {
    for punisher in [
        "Narset, Parter of Veils",
        "Sheoldred, the Apocalypse",
        "Orcish Bowmasters",
        "Spirit of the Labyrinth",
        "Notion Thief",
    ] {
        let (mut g, w) = wheel_position();
        perm(&mut g, P1, punisher);
        assert!(find(&mut g, w, "-7").is_none(), "{punisher}: {:?}", labels(&mut g, w));
    }
    // even hands
    let (mut g, w) = wheel_position();
    g.player_mut(P1).hand.truncate(1);
    assert!(find(&mut g, w, "-7").is_none(), "a gap under three cards");
    // a full hand of ours
    let (mut g, w) = wheel_position();
    hand(&mut g, P0, &["Brainstorm", "Ponder", "Counterspell"]);
    assert!(find(&mut g, w, "-7").is_none(), "three nonland cards in our hand");
    // Underworld Breach to fuel
    let (mut g, w) = wheel_position();
    hand(&mut g, P0, &["Underworld Breach"]);
    assert!(find(&mut g, w, "-7").is_none(), "Breach in hand");
    let (mut g, w) = wheel_position();
    perm(&mut g, P0, "Underworld Breach");
    assert!(find(&mut g, w, "-7").is_none(), "Breach on the battlefield");
    // no mana to use the new cards
    let (mut g, w) = wheel_position();
    for l in g.player(P0).lands.clone() {
        g.perm_mut(l).tapped = true;
    }
    assert!(find(&mut g, w, "-7").is_none(), "no mana left");
    // a punisher on our own side is not an opponent's
    let (mut g, w) = wheel_position();
    perm(&mut g, P0, "Orcish Bowmasters");
    assert!(find(&mut g, w, "-7").is_some());
}

#[test]
fn a_walker_acts_once_a_turn() {
    let mut g = tbl();
    let w = walker(&mut g, 4);
    ai_use(&mut g, w, "+2");
    assert!(labels(&mut g, w).is_empty(), "one loyalty ability a turn");
    // a new turn
    g.round += 1;
    assert!(!labels(&mut g, w).is_empty());
}

// ================================================================== 6. the look-ahead
#[test]
fn the_look_ahead_does_not_count_a_black_lotus_token_as_a_mana_rock() {
    let mut g = tbl();
    let w = walker(&mut g, 7);
    let ring = perm(&mut g, P0, "Sol Ring");
    assert_eq!(human_use(&mut g, w, "-3"), None);
    let t = lotus_tokens(&g, P0)[0];
    assert_eq!(sim_core::ai::search::output_cards(&g, t), 0.0, "spent once, not every turn");
    assert!(sim_core::ai::search::output_cards(&g, ring) > 0.0, "a Sol Ring is a rock");
}

// ================================================================== 7. Cali
/// a data dir whose decks.json is `edit(real decks.json)` (every other file symlinked), loaded as the engine
fn engine_with(name: &str, edit: impl FnOnce(&mut serde_json::Value)) -> Result<sim_core::run::Engine, String> {
    let real = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");
    let dir = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("cards_arzakon_{name}"));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for e in std::fs::read_dir(&real).unwrap() {
        let e = e.unwrap();
        if e.file_name() != "decks.json" {
            std::os::unix::fs::symlink(e.path(), dir.join(e.file_name())).unwrap();
        }
    }
    let mut v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(real.join("decks.json")).unwrap()).unwrap();
    edit(&mut v);
    std::fs::write(dir.join("decks.json"), serde_json::to_string(&v).unwrap()).unwrap();
    sim_core::run::Engine::load(&dir)
}

#[test]
fn a_cali_deck_with_arzakon_as_its_commander_loads_and_is_admitted() {
    // the Cali ban list (data/formats.toml) names Black Lotus the card, not Arzakon, and is read from deck lists only
    let e = engine_with("commander", |v| {
        let d = v["mine"].as_array_mut().unwrap().iter_mut().find(|d| d["key"] == "arzakon").unwrap();
        d["commander"] = json!("Arzakon");
    })
    .unwrap();
    assert!(e.admits("arzakon", "cali").is_ok());
    assert_eq!(e.format_of("arzakon").key, "cali");
    let banned = std::fs::read_to_string(data_dir().join("formats.toml")).unwrap();
    assert!(banned.contains("\"Black Lotus\"") && !banned.contains("\"Arzakon\""));
}

#[test]
fn a_cali_table_makes_and_spends_a_lotus_token_without_a_refusal() {
    let mut g = table_in(&["arzakon", "yshtola"], 7, sim_core::format::Rules { life: 30, first_player_draws: false });
    set_commander(&mut g, P0, ARZ);
    assert_eq!(g.player(P0).life, 30);
    let w = walker(&mut g, 7);
    hand(&mut g, P0, &["Demonic Tutor"]);
    ai_use(&mut g, w, "-3");
    assert_eq!(lotus_tokens(&g, P0).len(), 1);
    assert!(mana::pay(&mut g, P0, 0, "BBB", false).unwrap(), "spent for three of one colour");
    assert!(lotus_tokens(&g, P0).is_empty());
}

// ================================================================== 8. "control your commander"
#[test]
fn free_with_the_commander_out_only_while_arzakon_is_on_the_battlefield() {
    let mut g = tbl();
    let fg = card(&g, "Fierce Guardianship");
    assert!(!commander_out(&g, P0) && !stack::free_counter(&g, P0, fg), "Arzakon in the command zone");
    colour_lands(&mut g, P0);
    let cmd = g.player(P0).cmd;
    assert!(cast::cast_card(&mut g, P0, cmd, "cmd", Ctx::default()).unwrap());
    assert!(commander_out(&g, P0) && stack::free_counter(&g, P0, fg), "Arzakon out");
    let w = perms_named(&g, P0, ARZ)[0];
    zones::die(&mut g, w, "destroy".into()).unwrap();
    assert!(!commander_out(&g, P0), "back in the command zone: off again");
}

#[test]
fn jeskas_will_takes_both_modes_with_arzakon_out() {
    let opp_hand = ["Brainstorm", "Ponder", "Counterspell"];
    // the exile mode plays a land it finds on top, which would add mana here: put nonland cards on top so the count depends on
    // the modes only, not on the shuffled list
    fn nonland_on_top(g: &mut Game) {
        let mut lib = std::mem::take(&mut g.player_mut(P0).library);
        lib.sort_by_key(|&c| !g.db.get(c).land); // stable: lands first (the bottom), nonland cards last (the top)
        g.player_mut(P0).library = lib;
    }
    let mut g = tbl();
    nonland_on_top(&mut g);
    hand(&mut g, P1, &opp_hand);
    sim_core::engine::tutors::jeskas_will(&mut g, P0).unwrap();
    assert_eq!(total_mana(&g, P0, false), 0, "Arzakon in the command zone: one mode only (the AI took the exile)");
    let mut g = tbl();
    nonland_on_top(&mut g);
    walker(&mut g, 4);
    hand(&mut g, P1, &opp_hand);
    sim_core::engine::tutors::jeskas_will(&mut g, P0).unwrap();
    assert_eq!(total_mana(&g, P0, false), 3, "both modes: {{R}} for each card in the opponent's hand");
}

// ================================================================== 9. practice mode
fn use_index(g: &mut Game, w: PermId, sign_cost: &str) -> (usize, usize) {
    let i = g.player(P0).perms.iter().position(|&x| x == w).unwrap();
    let ab = abilities_of(g, P0, w).unwrap();
    let k = ab
        .iter()
        .position(|a| a.0.starts_with(sign_cost))
        .unwrap_or_else(|| panic!("{:?}", ab.iter().map(|a| &a.0).collect::<Vec<_>>()));
    (i, k)
}

#[test]
fn a_person_chooses_the_plus_two_target_and_cancelling_costs_nothing() {
    let mut g = tbl();
    let w = walker(&mut g, 4);
    let angel = perm(&mut g, P1, "Serra Angel");
    let (i, k) = use_index(&mut g, w, "+2");
    // cancel: nothing is paid
    let h = seat(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pick(k), cancel(), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert_eq!(g.perm(w).loyalty, Some(4), "cancelled: nothing paid");
    let asked: Vec<_> = h.asked().into_iter().filter(|r| r.prompt.contains("Arzakon +2")).collect();
    assert_eq!(asked.len(), 1);
    let choices = &asked[0].choices;
    assert!(!choices.iter().any(|c| c.starts_with("Arzakon (you")), "never Arzakon itself: {choices:?}");
    assert!(
        choices.iter().any(|c| c.starts_with("Arzakon (player") && c.contains("(you)")),
        "the person themselves: {choices:?}"
    );
    assert!(choices.iter().any(|c| c.contains("Serra Angel")), "{choices:?}");
    // the person's pick, not the AI's: Serra Angel (4 toughness) takes 3 and survives, marked
    let mut g = tbl();
    let w = walker(&mut g, 4);
    let angel2 = perm(&mut g, P1, "Serra Angel");
    let _ = angel;
    let (i, k) = use_index(&mut g, w, "+2");
    seat(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pick(k), by_text("player, 40 life"), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert_eq!(g.perm(w).loyalty, Some(6));
    let lost: i32 = [P0, P1, P2].iter().map(|&p| 40 - g.player(p).life).sum();
    assert_eq!(lost, 3, "someone's face took three");
    assert!(g.perm(angel2).on_bf);
}

#[test]
fn a_person_uses_minus_three_and_minus_seven_without_a_question() {
    let mut g = tbl();
    let w = walker(&mut g, 8);
    let (i, k) = use_index(&mut g, w, "-3");
    let h = seat(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pick(k), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert_eq!(g.perm(w).loyalty, Some(5));
    assert_eq!(lotus_tokens(&g, P0).len(), 1);
    let asked: Vec<String> = h
        .asked()
        .iter()
        .map(|r| r.prompt.clone())
        .filter(|p| p.contains("Arzakon") && !p.contains("which ability"))
        .collect();
    assert!(asked.is_empty(), "no question of its own: {asked:?}");
    let mut g = tbl();
    let w = walker(&mut g, 8);
    let (i, k) = use_index(&mut g, w, "-7");
    seat(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pick(k), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert_eq!(g.perm(w).loyalty, Some(1));
    assert_eq!(g.player(P1).hand.len(), 7);
}

#[test]
fn a_rings_of_brighthearth_copy_of_the_persons_plus_two_asks_for_a_new_target() {
    let mut g = tbl();
    let w = walker(&mut g, 4);
    perm(&mut g, P0, "Rings of Brighthearth");
    g.player_mut(P0).pool.add('C', 2);
    let n = Arc::new(Mutex::new(0usize));
    let n2 = n.clone();
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if &*r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        if r.prompt.starts_with("Rings of Brighthearth: pay") {
            return Answer::Pick(0);
        }
        if r.prompt.contains("Arzakon +2") {
            let mut n = n2.lock().unwrap();
            *n += 1;
            let word = if *n == 1 { "Serra Angel" } else { "Grave Titan" };
            return r.choices.iter().position(|c| c.contains(word)).map_or(Answer::Decline, Answer::Pick);
        }
        Answer::Decline
    }));
    g.humans.add(P0, h.clone());
    let a = perm(&mut g, P1, "Serra Angel");
    let t = perm(&mut g, P1, "Grave Titan");
    g.perm_mut(a).eot_pt = (0, -2); // 4 - 2: two toughness, three damage kills
    g.perm_mut(t).eot_pt = (0, -3); // 6 - 3
    assert_eq!(human_use(&mut g, w, "+2"), None);
    assert!(h.asked().iter().any(|r| r.prompt.starts_with("Rings of Brighthearth: pay")));
    assert_eq!(*n.lock().unwrap(), 2, "asked once for the ability, once for the copy");
    assert!(!g.perm(a).on_bf && !g.perm(t).on_bf, "the copy chose its own target");
    assert_eq!(g.perm(w).loyalty, Some(6), "loyalty paid once");
}

// ================================================================== the AI path is never used for a person
#[test]
fn a_person_walker_is_not_driven_by_the_ai_options() {
    let mut g = tbl();
    let w = walker(&mut g, 4);
    let list = abilities_of(&mut g, P0, w).unwrap();
    let names: Vec<&str> = list.iter().map(|a| a.0.as_str()).collect();
    assert!(
        names.iter().any(|n| n.starts_with("+2"))
            && names.iter().any(|n| n.starts_with("-3"))
            && names.iter().any(|n| n.starts_with("-7")),
        "{names:?}"
    );
}
