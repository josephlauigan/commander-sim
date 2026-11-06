//! Phase C3 (casual-tier speed): one scripted position per fix, each wrong before the fix.
//!
//! - a tax or cap on the chosen defender no longer leaves the rest of the army at home;
//! - infect and toxic attackers are poison threats: blocks, danger, removal value and kill checks;
//! - infect damage is combat damage dealt to the player (lifelink, triggers, commander damage);
//! - an outside deck casts its commander the turn it can pay for it.

use sim_core::ai::brain;
use sim_core::engine::combat::{self, poison_hit, poison_share};
use sim_core::engine::values::pval;
use sim_core::ids::{PermId, PlayerId};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const ATRAXA: &str = "atraxa-infect";

// ------------------------------------------------------------------ a tax no longer idles the army

#[test]
fn the_creatures_a_tax_keeps_off_one_player_attack_another() {
    for seed in 1..=6 {
        let mut g = table_with(&[ATRAXA, "seph", "cloud"], seed, 40);
        perm(&mut g, P1, "Ghostly Prison");
        g.player_mut(P1).life = 5; // the tempting target: one paid attacker kills it
        lands(&mut g, P0, "Plains", 2, false); // {2} a creature: one attacker paid for
        let army: Vec<PermId> = (0..3).map(|_| token(&mut g, P0, 6)).collect();
        g.log = Some(vec![]);
        combat::combat(&mut g, P0).unwrap();
        let log = g.log.take().unwrap_or_default().join("\n");
        let attacked = army.iter().filter(|&&a| g.perm(a).tapped).count();
        assert_eq!(attacked, 3, "seed {seed}: all three attack, one into the tax, two elsewhere\n{log}");
        assert!(g.player(P2).life < 40 || !g.player(P1).alive, "seed {seed}: damage landed\n{log}");
    }
}

#[test]
fn a_tax_cuts_the_power_that_counts_towards_a_kill() {
    let mut g = table(&[ATRAXA, "seph", "cloud"]);
    perm(&mut g, P1, "Ghostly Prison");
    lands(&mut g, P0, "Plains", 2, false);
    // three 6-power attackers, mana for one: 6 can be sent at the taxed player, 18 at the others
    assert_eq!(combat::sendable_power(&g, P0, P1, &[6, 6, 6]), 6);
    assert_eq!(combat::sendable_power(&g, P0, P2, &[6, 6, 6]), 18);
}

// ------------------------------------------------------------------ poison is a threat

#[test]
fn infect_and_toxic_are_poison_not_life() {
    let mut g = table(&[ATRAXA, "seph"]);
    let agent = perm(&mut g, P0, "Blighted Agent");
    let rat = perm(&mut g, P0, "Blightbelly Rat");
    let plain = token(&mut g, P0, 3);
    assert_eq!(poison_hit(&g, agent), 1, "infect 1/1");
    assert_eq!(poison_hit(&g, rat), 1, "toxic 1");
    assert_eq!(poison_hit(&g, plain), 0);
    g.player_mut(P1).poison = 8;
    assert!((poison_share(&g, P1, &[agent, rat]) - 1.0).abs() < 1e-9, "two of the two counters P1 has left");
    assert_eq!(poison_share(&g, P1, &[plain]), 0.0);
}

#[test]
fn a_player_one_hit_from_ten_poison_chump_blocks_an_infect_attacker() {
    // Ichor Rats 2/5 (pumped) against a 1/1 token: the token cannot kill it and the 2 life it would stop is nothing,
    // but P1 has 8 poison counters
    let mut g = table(&[ATRAXA, "seph"]);
    let rats = perm(&mut g, P0, "Ichor Rats");
    g.perm_mut(rats).eot_pt = (0, 4);
    let chump = token(&mut g, P1, 1);
    g.player_mut(P1).poison = 8;
    let mut blocked = 0;
    for seed in 0..20 {
        let mut rng = sim_core::rng::Rng::from_u64(seed);
        let assign = combat::ai_blocks(&g, P0, &[rats], P1, &[], &mut rng);
        if assign.contains(&(rats, chump)) {
            blocked += 1;
        }
    }
    assert!(blocked >= 19, "the chump block is made when ten counters are two hits away: {blocked} of 20");
    // with no poison on P1 the same attack is let through (the old, and still right, behaviour)
    g.player_mut(P1).poison = 0;
    let mut through = 0;
    for seed in 0..20 {
        let mut rng = sim_core::rng::Rng::from_u64(seed);
        if combat::ai_blocks(&g, P0, &[rats], P1, &[], &mut rng).is_empty() {
            through += 1;
        }
    }
    assert!(through >= 17, "no poison, no chump: {through} of 20");
}

#[test]
fn the_chump_probability_follows_the_poison_share() {
    let g = table(&[ATRAXA, "seph"]);
    let life_only = sim_core::ai::chump_prob(&g, P1, 3);
    let with_poison = sim_core::ai::chump_prob_poison(&g, P1, 3, 1.0);
    assert!(life_only < 0.05 && with_poison > 0.99, "{life_only} vs {with_poison}");
}

#[test]
fn an_infect_creature_is_worth_more_to_remove_as_the_poison_mounts() {
    let mut g = table(&[ATRAXA, "seph"]);
    let agent = perm(&mut g, P0, "Blighted Agent"); // 1/1 unblockable infect
    let before = pval(&g, agent);
    g.player_mut(P1).poison = 6;
    let after = pval(&g, agent);
    assert!(after > before + 0.5, "{before} -> {after}");
    // and more than a plain 1/1 of the same size
    let vanilla = perm(&mut g, P0, "Llanowar Elves");
    assert!(after > pval(&g, vanilla), "{after} vs {}", pval(&g, vanilla));
}

#[test]
fn the_danger_from_infect_attackers_is_the_poison_they_bring() {
    // 4 power of infect against 40 life is nothing; against a player on 6 poison it is the game
    let mut g = table(&[ATRAXA, "seph"]);
    perm(&mut g, P0, "Skithiryx, the Blight Dragon"); // 4/4 flying infect
    let calm = brain::Situation::new(&g, P1).danger;
    g.player_mut(P1).poison = 6;
    let poisoned = brain::Situation::new(&g, P1).danger;
    assert!((0.3..0.5).contains(&calm), "four counters of ten: {calm}"); // 0.1 when it was read as 4 life of 40
    assert!(poisoned > 0.8, "{poisoned}");
}

#[test]
fn ten_poison_is_a_kill_for_the_attacker_s_target_choice() {
    // P1 is two counters from ten and P2 has a full life total: an infect attacker with enough poison goes for P1
    let mut g = table(&[ATRAXA, "seph", "cloud"]);
    perm(&mut g, P0, "Skithiryx, the Blight Dragon"); // 4 poison a hit
    g.player_mut(P1).poison = 7;
    let mut at_p1 = 0;
    for seed in 0..30 {
        let mut g2 = g.clone();
        g2.rng = sim_core::rng::Rng::from_u64(seed);
        if sim_core::ai::choose_defender(&mut g2, P0) == P1 {
            at_p1 += 1;
        }
    }
    assert!(at_p1 >= 27, "{at_p1} of 30 at the player Skithiryx kills");
}

// ------------------------------------------------------------------ infect damage is combat damage

#[test]
fn infect_damage_is_combat_damage_dealt_so_lifelink_gains() {
    let mut g = table(&[ATRAXA, "seph"]);
    let stinger = perm(&mut g, P0, "Plague Stinger"); // 1/1 flying infect
    g.perm_mut(stinger).lifelink = true;
    let (mine, theirs) = (g.player(P0).life, g.player(P1).life);
    combat::resolve_combat(&mut g, P0, &[stinger], P1, &[stinger]).unwrap();
    assert_eq!(g.player(P1).poison, 1);
    assert_eq!(g.player(P1).life, theirs, "no life lost");
    assert_eq!(g.player(P0).life, mine + 1, "lifelink on damage dealt as poison");
}

// ------------------------------------------------------------------ the commander comes down on curve

#[test]
fn an_outside_deck_wants_its_commander_first() {
    let mut g = table(&[ATRAXA, "seph"]);
    g.player_mut(P0).turns = 4;
    for l in ["Forest", "Plains", "Island", "Swamp"] {
        lands(&mut g, P0, l, 1, false);
    }
    let opts = brain::main_options(&mut g, P0, false).unwrap();
    let cmdr = opts.iter().find(|o| o.label.starts_with("Atraxa, Praetors")).expect("the commander is offered");
    assert!(cmdr.utility >= 9.0 - 1e-9, "a first cast carries the bonus: {}", cmdr.utility);
    // a recast pays the tax and loses it
    g.player_mut(P0).tax = 2;
    let again = brain::main_options(&mut g, P0, false).unwrap();
    if let Some(o) = again.iter().find(|o| o.label.starts_with("Atraxa, Praetors")) {
        assert!(o.utility < cmdr.utility, "{} vs {}", o.utility, cmdr.utility);
    }
}

// ------------------------------------------------------------------ the look-ahead's counter from the library top

/// A Mystic Forge lets a colourless counterspell (Pact of Negation) be cast from the top of the library: `pick_counter`
/// offers it, and the look-ahead's copy cast it with `cast_counter`, which took it out of the hand and panicked
/// (seed 500000 Thanos Tier 5, game 111).
#[test]
fn a_counterspell_on_top_of_the_library_is_cast_from_there() {
    let mut g = table(&["sauron", "cloud"]);
    perm(&mut g, P0, "Mystic Forge");
    lands(&mut g, P0, "Island", 5, false);
    let pact = card(&g, "Pact of Negation");
    g.player_mut(P0).library.push(pact);
    let spell = card(&g, "Lightning Bolt");
    assert_eq!(sim_core::engine::stack::pick_counter(&g, P0, spell), Some(pact), "offered from the top");
    let hand = g.player(P0).hand.len();
    let lib = g.player(P0).library.len();
    assert!(sim_core::engine::stack::cast_counter(&mut g, P0, pact, Some(spell)).unwrap());
    assert_eq!(g.player(P0).hand.len(), hand, "the hand is untouched");
    assert_eq!(g.player(P0).library.len(), lib - 1, "taken off the top");
    assert!(g.player(P0).gy.contains(&pact));
}

/// a counterspell that is in neither the hand nor on top is not cast (and nothing panics)
#[test]
fn a_counterspell_the_copy_does_not_hold_is_not_cast() {
    let mut g = table(&["sauron", "cloud"]);
    lands(&mut g, P0, "Island", 5, false);
    let pact = card(&g, "Pact of Negation");
    let spell = card(&g, "Lightning Bolt");
    assert!(!sim_core::engine::stack::cast_counter(&mut g, P0, pact, Some(spell)).unwrap());
    assert!(!g.player(P0).gy.contains(&pact));
}

// ------------------------------------------------------------------ huge life gains in one turn

/// Two huge gains in one turn (Exquisite Blood off a doubled Dragon army, seed 500000 Y'shtola Tier 5, game 1953) overflowed
/// the "life gained this turn" counter; it saturates at the life cap like the life total does.
#[test]
fn two_huge_life_gains_in_one_turn_do_not_overflow() {
    let mut g = table(&["sauron", "cloud"]);
    let cap = sim_core::engine::life::LIFE_CAP;
    sim_core::engine::life::gain(&mut g, P0, cap).unwrap();
    sim_core::engine::life::gain(&mut g, P0, cap).unwrap();
    assert!(g.player(P0).life <= cap);
    assert_eq!(g.player(P0).gained_turn.map(|x| x.1), Some(cap));
}
