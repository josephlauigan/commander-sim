//! Tier 4 card code (impls/t4.rs, Python's cards/impl/t4.py) on hand-built positions: ninjutsu and Yuriko's reveal,
//! Krenko's Goblins, Heliod, Sanguine Bond and Exquisite Blood, Isochron Scepter, Consecrated Sphinx, Aluren, Prosper
//! and the Treasure makers. The Python suite has no tests of these cards beyond Consecrated Sphinx's draw; each
//! expectation follows the Python card code.

use sim_core::cardcode;
use sim_core::engine::{hooks, life, turn, zones};
use sim_core::hooks::{Action, Call, Event, Opt};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::t4;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

const YURIKO: &str = "yuriko-dimir-ninjas";
const KRENKO: &str = "krenko-mono-red-goblins";
const CHULANE: &str = "chulane-bant-value-combo";
const PROSPER: &str = "prosper-rakdos-exile-treasure";
const HELIOD: &str = "heliod-mono-white-stax";

fn named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).name == name).collect()
}

fn tokens(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).token).collect()
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

/// a permanent's own options hook
fn opts(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.perm(src).cd.unwrap()).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

/// run an option's ability
fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    match o.act.clone() {
        Some(Action::Ability { src, f, arg }) => f(g, src, p, arg).unwrap(),
        _ => panic!("not an ability"),
    }
}

fn combat_damage(g: &mut Game, p: PlayerId, a: PermId, d: PlayerId, dmg: i32) {
    hooks::fire_trigger(g, Event::CombatDamage, Call::CombatDamage { p, a, d, dmg }).unwrap();
}

/// put a card on top of p's library
fn on_top(g: &mut Game, p: PlayerId, name: &str) {
    let c = take(g, p, name);
    g.player_mut(p).library.push(c);
}

// ------------------------------------------------------------------ ninjutsu and Yuriko
#[test]
fn ninjutsu_swaps_an_unblocked_attacker_for_a_ninja() {
    let mut g = table(&[YURIKO, "seph"]);
    g.player_mut(P0).cmd_in_zone = false; // Yuriko elsewhere: only the Ninja in hand
    lands(&mut g, P0, "Island", 1, false);
    hand(&mut g, P0, &["Moon-Circuit Hacker"]);
    let t = token(&mut g, P0, 1);
    let mut atk = vec![t];
    cardcode::ninjutsu(&mut g, P0, &mut atk, P1, &mut []).unwrap();
    let hacker = named(&g, P0, "Moon-Circuit Hacker");
    assert_eq!(atk, hacker, "the Ninja attacks in the token's place");
    assert!(g.perm(hacker[0]).tapped && !g.perm(hacker[0]).sick);
    assert!(!g.perm(t).on_bf, "the token went back (and ceased to exist)");
    assert!(!in_hand(&g, P0, "Moon-Circuit Hacker"));
    assert_eq!(g.player(P0).stats.get("ninjutsu"), Some(&1));
}

#[test]
fn a_blocked_attacker_or_no_mana_means_no_ninjutsu() {
    let mut g = table(&[YURIKO, "seph"]);
    g.player_mut(P0).cmd_in_zone = false;
    hand(&mut g, P0, &["Moon-Circuit Hacker"]);
    let t = token(&mut g, P0, 1);
    let mut atk = vec![t];
    cardcode::ninjutsu(&mut g, P0, &mut atk, P1, &mut []).unwrap();
    assert_eq!(atk, vec![t], "no Island: can't pay {{U}}");
    lands(&mut g, P0, "Island", 1, false);
    let b = token(&mut g, P1, 1);
    cardcode::ninjutsu(&mut g, P0, &mut atk, P1, &mut [(t, b)]).unwrap();
    assert_eq!(atk, vec![t], "blocked");
}

#[test]
fn yuriko_comes_from_the_command_zone() {
    let mut g = table(&[YURIKO, "seph"]);
    lands(&mut g, P0, "Island", 1, false);
    lands(&mut g, P0, "Swamp", 1, false);
    let t = token(&mut g, P0, 1);
    let mut atk = vec![t];
    cardcode::ninjutsu(&mut g, P0, &mut atk, P1, &mut []).unwrap();
    let y = named(&g, P0, "Yuriko, the Tiger's Shadow");
    assert_eq!(atk, y);
    assert!(g.perm(y[0]).is_cmd && !g.player(P0).cmd_in_zone);
}

#[test]
fn yuriko_reveals_the_top_card_and_drains_its_mana_value() {
    let mut g = table(&[YURIKO, "seph", "veyran"]);
    perm(&mut g, P0, "Yuriko, the Tiger's Shadow");
    let ninja = perm(&mut g, P0, "Mistblade Shinobi");
    on_top(&mut g, P0, "Treasure Cruise");
    let n = g.player(P0).hand.len();
    combat_damage(&mut g, P0, ninja, P1, 1);
    assert!(in_hand(&g, P0, "Treasure Cruise"));
    assert_eq!(g.player(P0).hand.len(), n + 1);
    assert_eq!(g.player(P1).life, 40 - 8);
    assert_eq!(g.player(PlayerId(2)).life, 40 - 8);
    // a non-Ninja hit reveals nothing
    let t = token(&mut g, P0, 1);
    combat_damage(&mut g, P0, t, P1, 1);
    assert_eq!(g.player(P0).hand.len(), n + 1);
}

#[test]
fn ninjutsu_costs_and_silver_fur_master() {
    let mut g = table(&[YURIKO, "seph"]);
    hand(&mut g, P0, &["Ink-Eyes, Servant of Oni", "Mistblade Shinobi", "Brainstorm"]);
    assert_eq!(cardcode::ninjutsu_costs(&g, P0), vec![(3, "BB".to_string()), (0, "U".to_string())]);
    perm(&mut g, P0, "Silver-Fur Master");
    assert_eq!(cardcode::ninjutsu_costs(&g, P0), vec![(2, "BB".to_string()), (0, "U".to_string())]);
    assert_eq!(t4::ninjutsu_cost(&g, P0, "Satoru Umezawa"), (1, "UB"), "Satoru's grant: {{2}}{{U}}{{B}}, one less");
}

#[test]
fn thousand_faced_shadow_copies_another_attacker() {
    let mut g = table(&[YURIKO, "seph"]);
    g.player_mut(P0).cmd_in_zone = false;
    lands(&mut g, P0, "Island", 4, false);
    hand(&mut g, P0, &["Thousand-Faced Shadow"]);
    let t = token(&mut g, P0, 1);
    let other = perm(&mut g, P0, "Moon-Circuit Hacker");
    let mut atk = vec![t, other];
    cardcode::ninjutsu(&mut g, P0, &mut atk, P1, &mut [(other, other)]).unwrap();
    assert_eq!(atk.len(), 3, "the Shadow and its token copy of the Hacker joined: {atk:?}");
    let copies: Vec<PermId> = named(&g, P0, "Moon-Circuit Hacker").into_iter().filter(|&m| g.perm(m).token).collect();
    assert_eq!(copies.len(), 1);
    assert!(g.perm(copies[0]).tapped && atk.contains(&copies[0]));
}

#[test]
fn satoru_digs_three_deep_on_ninjutsu() {
    let mut g = table(&[YURIKO, "seph"]);
    g.player_mut(P0).cmd_in_zone = false;
    perm(&mut g, P0, "Satoru Umezawa");
    lands(&mut g, P0, "Island", 1, false);
    hand(&mut g, P0, &["Moon-Circuit Hacker"]);
    let lib = g.player(P0).library.len();
    let h = g.player(P0).hand.len();
    let t = token(&mut g, P0, 1);
    let mut atk = vec![t];
    cardcode::ninjutsu(&mut g, P0, &mut atk, P1, &mut []).unwrap();
    assert_eq!(g.player(P0).hand.len(), h, "the Hacker left the hand, Satoru's card came in");
    assert_eq!(g.player(P0).library.len(), lib - 1);
}

#[test]
fn yuriko_s_tutor_wishes_and_cast_priorities() {
    let mut g = table(&[YURIKO, "seph"]);
    let wish: Vec<String> = cardcode::yuriko_wish(&g, P0).iter().map(|&c| g.db.get(c).name.to_string()).collect();
    assert_eq!(wish[0], "Rhystic Study", "no way to put a card back: card advantage");
    hand(&mut g, P0, &["Brainstorm"]);
    let wish = cardcode::yuriko_wish(&g, P0);
    assert_eq!(&*g.db.get(wish[0]).name, "Draco", "Yuriko in the command zone and Brainstorm in hand");
    let c = card(&g, "Changeling Outcast");
    assert_eq!(cardcode::yuriko_prio(&g, P0, c), Some(80));
    assert_eq!(cardcode::yuriko_prio(&g, P0, card(&g, "Rhystic Study")), None);
}

// ------------------------------------------------------------------ Krenko and the Goblins
#[test]
fn krenko_makes_a_goblin_per_goblin() {
    let mut g = table(&[KRENKO, "seph"]);
    let k = perm(&mut g, P0, "Krenko, Mob Boss");
    t4::goblins(&mut g, P0, 2).unwrap();
    let o = opts(&mut g, k, P0, None);
    assert_eq!(o.len(), 1);
    assert!((o[0].utility - (3.5 + 0.4 * 3.0)).abs() < 1e-9, "three Goblins at the end of the turn before yours");
    assert!(run(&mut g, P0, &o[0]));
    assert!(g.perm(k).tapped);
    assert_eq!(tokens(&g, P0).len(), 5);
    assert!(opts(&mut g, k, P0, None).is_empty(), "tapped");
}

#[test]
fn goblin_warchief_discounts_goblins_and_gives_them_haste() {
    let mut g = table(&[KRENKO, "seph"]);
    let gob = t4::goblins(&mut g, P0, 1).unwrap()[0];
    let theirs = t4::goblins(&mut g, P1, 1).unwrap()[0];
    assert!(!cardcode::granted_kw(&g, gob, "haste"));
    let w = perm(&mut g, P0, "Goblin Warchief");
    assert!(cardcode::granted_kw(&g, gob, "haste"));
    assert!(cardcode::granted_kw(&g, w, "haste"), "Warchief hastes itself (Chieftain doesn't)");
    assert!(!cardcode::granted_kw(&g, theirs, "haste"));
    assert_eq!(hooks::total_cost(&g, P0, card(&g, "Goblin Ringleader")), -1);
    assert_eq!(hooks::total_cost(&g, P1, card(&g, "Goblin Ringleader")), 0);
    assert_eq!(hooks::total_cost(&g, P0, card(&g, "Lightning Bolt")), 0);
}

#[test]
fn goblin_etb_and_death_triggers() {
    let mut g = table(&[KRENKO, "seph"]);
    perm(&mut g, P0, "Siege-Gang Commander");
    assert_eq!(tokens(&g, P0).len(), 3);
    let m = perm(&mut g, P0, "Mogg War Marshal");
    assert_eq!(tokens(&g, P0).len(), 4);
    zones::die(&mut g, m, "destroy").unwrap();
    assert_eq!(tokens(&g, P0).len(), 5);
    perm(&mut g, P0, "Impact Tremors");
    perm(&mut g, P0, "Beetleback Chief");
    assert_eq!(g.player(P1).life, 40 - 1, "the Chief (tokens made by card code don't fire enter hooks)");
}

#[test]
fn goblin_chainwhirler_pings_each_opponent_and_their_x_1s() {
    let mut g = table(&[KRENKO, "seph", "veyran"]);
    let small = token(&mut g, P1, 1);
    let big = token(&mut g, P1, 3);
    let mine = token(&mut g, P0, 1);
    perm(&mut g, P0, "Goblin Chainwhirler");
    assert_eq!(g.player(P1).life, 39);
    assert_eq!(g.player(P2).life, 39);
    assert!(!g.perm(small).on_bf && g.perm(big).on_bf && g.perm(mine).on_bf);
}

#[test]
fn krenko_tin_street_kingpin_and_piledriver_attack() {
    let mut g = table(&[KRENKO, "seph"]);
    let k = perm(&mut g, P0, "Krenko, Tin Street Kingpin");
    let pd = perm(&mut g, P0, "Goblin Piledriver");
    let atk = vec![k, pd];
    hooks::fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: atk.clone(), d: P1 }).unwrap();
    assert_eq!(g.perm(k).plus, 1);
    assert_eq!(tokens(&g, P0).len(), 2, "Goblins equal to its power (2)");
    assert_eq!(g.perm(pd).eot_pt, (2, 0), "+2/+0 for the other attacking Goblin");
}

// ------------------------------------------------------------------ Chulane, Aluren, Consecrated Sphinx
#[test]
fn aluren_makes_small_creatures_free() {
    let mut g = table(&[CHULANE, "seph"]);
    perm(&mut g, P0, "Aluren");
    assert_eq!(hooks::total_cost(&g, P1, card(&g, "Coiling Oracle")), -99);
    assert_eq!(hooks::total_cost(&g, P0, card(&g, "Craterhoof Behemoth")), 0);
}

#[test]
fn consecrated_sphinx_draws_two_when_an_opponent_draws() {
    let mut g = table(&[CHULANE, "seph"]);
    perm(&mut g, P0, "Consecrated Sphinx");
    let n = g.player(P0).hand.len();
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 2);
    zones::draw(&mut g, P0, 1, false).unwrap(); // your own draw: nothing
    assert_eq!(g.player(P0).hand.len(), n + 3);
}

#[test]
fn chulane_draws_and_puts_a_land_on_creature_casts() {
    let mut g = table(&[CHULANE, "seph"]);
    perm(&mut g, P0, "Chulane, Teller of Tales");
    hand(&mut g, P0, &["Forest", "Craterhoof Behemoth"]);
    let c = card(&g, "Craterhoof Behemoth");
    sim_core::engine::cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert_eq!(g.player(P0).lands.len(), 1, "Chulane put the Forest down");
    assert!(!in_hand(&g, P0, "Forest"));
    assert_eq!(g.player(P0).hand.len(), 1, "and drew a card");
}

#[test]
fn kor_skyfisher_returns_an_enters_permanent_or_itself() {
    let mut g = table(&[CHULANE, "seph"]);
    let k = perm(&mut g, P0, "Kor Skyfisher");
    assert!(!g.perm(k).on_bf && in_hand(&g, P0, "Kor Skyfisher"), "nothing better: itself");
    let mut g = table(&[CHULANE, "seph"]);
    perm(&mut g, P0, "Coiling Oracle");
    let k = perm(&mut g, P0, "Kor Skyfisher");
    assert!(g.perm(k).on_bf && in_hand(&g, P0, "Coiling Oracle"));
}

// ------------------------------------------------------------------ Prosper and the Treasure makers
#[test]
fn prosper_exiles_at_the_end_step_and_makes_treasure_for_exiled_casts() {
    let mut g = table(&[PROSPER, "seph"]);
    perm(&mut g, P0, "Prosper, Tome-Bound");
    on_top(&mut g, P0, "Lightning Bolt");
    turn::end_step(&mut g, P0).unwrap();
    let bolt = card(&g, "Lightning Bolt");
    assert_eq!(g.player(P0).impulse_long, vec![(bolt, 2)]);
    assert!(in_hand(&g, P0, "Lightning Bolt"));
    let t = g.player(P0).treasures;
    hooks::fire_trigger(&mut g, Event::Cast, Call::Cast { caster: P0, c: bolt }).unwrap();
    assert_eq!(g.player(P0).treasures, t + 1);
}

#[test]
fn mahadi_makes_a_treasure_per_death_this_turn() {
    let mut g = table(&[PROSPER, "seph"]);
    perm(&mut g, P0, "Mahadi, Emporium Master");
    for _ in 0..2 {
        let t = token(&mut g, P1, 1);
        zones::die(&mut g, t, "destroy").unwrap();
    }
    let t = g.player(P0).treasures;
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).treasures, t + 2);
}

#[test]
fn dark_confidant_costs_life_equal_to_mana_value() {
    let mut g = table(&[PROSPER, "seph"]);
    perm(&mut g, P0, "Dark Confidant");
    on_top(&mut g, P0, "Bolas's Citadel");
    hooks::fire_trigger(&mut g, Event::Upkeep, Call::Player { p: P0 }).unwrap();
    assert!(in_hand(&g, P0, "Bolas's Citadel"));
    assert_eq!(g.player(P0).life, 40 - 6);
}

#[test]
fn sanguine_bond_and_exquisite_blood_drain_but_stop() {
    let mut g = table(&[PROSPER, "seph", "veyran"]);
    perm(&mut g, P0, "Sanguine Bond");
    life::gain(&mut g, P0, 3).unwrap();
    assert_eq!(g.player(P0).life, 43);
    assert_eq!(g.player(P1).life + g.player(P2).life, 80 - 3, "one opponent loses 3");
    perm(&mut g, P0, "Exquisite Blood");
    life::gain(&mut g, P0, 1).unwrap();
    assert!(!g.over || g.player(P0).alive, "the loop is bounded");
    assert!(g.player(P0).life > 44, "Blood gave the drained life back: {}", g.player(P0).life);
}

#[test]
fn xorn_adds_a_treasure_and_fireweaver_pings() {
    let mut g = table(&[PROSPER, "seph"]);
    perm(&mut g, P0, "Xorn");
    perm(&mut g, P0, "Reckless Fireweaver");
    let t = g.player(P0).treasures;
    zones::make_artifact_tokens(&mut g, P0, "Treasure", 1).unwrap();
    assert_eq!(g.player(P0).treasures, t + 2);
    assert_eq!(g.player(P1).life, 39);
}

#[test]
fn chandra_minus_three_kills_a_creature_worth_it() {
    let mut g = table(&[PROSPER, "seph"]);
    let c = perm(&mut g, P0, "Chandra, Torch of Defiance");
    perm(&mut g, P1, "Grave Titan"); // toughness 6: too big
    let o = opts(&mut g, c, P0, Some(false));
    assert!(o.iter().all(|x| !x.label.contains("-3")), "{:?}", o.iter().map(|x| &x.label).collect::<Vec<_>>());
    assert_eq!(o.len(), 2, "+1 (exile the top) and +1 (RR)");
    let mid = perm(&mut g, P1, "Orcish Bowmasters");
    let worth = sim_core::engine::values::pval(&g, mid) >= 4.0;
    let o = opts(&mut g, c, P0, Some(false));
    let minus = o.iter().find(|x| x.label.contains("-3")).cloned();
    assert_eq!(minus.is_some(), worth);
    if let Some(m) = minus {
        assert!(run(&mut g, P0, &m));
        assert!(!g.perm(mid).on_bf);
        assert_eq!(g.perm(c).loyalty, Some(4 - 3));
    }
}

// ------------------------------------------------------------------ Heliod and Isochron Scepter
#[test]
fn heliod_is_a_creature_with_devotion_five_and_grows_your_best_creature() {
    let mut g = table(&[HELIOD, "seph"]);
    let h = perm(&mut g, P0, "Heliod, Sun-Crowned");
    assert!(!g.is_creature(h), "devotion 1");
    assert!(cardcode::granted_kw(&g, h, "indestructible"));
    perm(&mut g, P0, "Thalia, Guardian of Thraben");
    perm(&mut g, P0, "Mother of Runes");
    assert!(!g.is_creature(h), "devotion 3");
    perm(&mut g, P0, "Grand Abolisher");
    assert!(g.is_creature(h), "devotion 5");
    assert_eq!(sim_core::engine::values::epow(&g, h), 5);
    let cre: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&m| m != h).collect();
    let best = zones::max_by(&cre, |m| sim_core::engine::values::pval(&g, m)).unwrap();
    life::gain(&mut g, P0, 1).unwrap();
    assert_eq!(g.perm(best).plus, 1, "the best creature");
    assert_eq!(g.perm(h).plus, 0, "never Heliod itself");
}

#[test]
fn isochron_scepter_imprints_and_casts_copies() {
    let mut g = table(&[HELIOD, "seph"]);
    hand(&mut g, P0, &["Swords to Plowshares"]);
    let sc = perm(&mut g, P0, "Isochron Scepter");
    assert!(g.player(P0).exile.contains(&card(&g, "Swords to Plowshares")));
    lands(&mut g, P0, "Plains", 3, false);
    let big = perm(&mut g, P1, "Grave Titan");
    let o = opts(&mut g, sc, P0, Some(false));
    assert_eq!(o.len(), 1, "{:?}", o.iter().map(|x| &x.label).collect::<Vec<_>>());
    assert!(run(&mut g, P0, &o[0]));
    assert!(!g.perm(big).on_bf, "the copy exiled the Titan");
    assert!(g.perm(sc).tapped);
    let stp = card(&g, "Swords to Plowshares");
    assert!(!g.player(P0).gy.contains(&stp), "the copy ceases to exist");
}

#[test]
fn scepter_waits_for_dramatic_reversal() {
    let mut g = table(&["kinnan-simic-mana-combo", "seph"]);
    let sc = card(&g, "Isochron Scepter");
    let prio = g.registry.get(sc).unwrap().prio.unwrap();
    assert!(g.player(P0).library.contains(&card(&g, "Dramatic Reversal")));
    assert_eq!(prio(&g, P0, sc), 0, "the Reversal is still in the library");
    hand(&mut g, P0, &["Dramatic Reversal"]);
    assert_eq!(prio(&g, P0, sc), 62);
}

// ------------------------------------------------------------------ whole games
#[test]
fn tier_four_games_run_and_yuriko_uses_ninjutsu() {
    let t4 = tier("t4");
    let mut ninjutsu = 0;
    for seed in 0..20u64 {
        let k = seed as usize;
        let keys = [YURIKO, t4[k % 5], t4[(k + 1) % 5], t4[(k + 2) % 5]];
        let keys: Vec<&str> =
            if keys[1..].contains(&YURIKO) { vec![YURIKO, KRENKO, CHULANE, PROSPER] } else { keys.to_vec() };
        let g = play(&keys, 4_000 + seed, false);
        assert!(g.over || g.round >= 20 || g.stopped, "seed {seed}");
        ninjutsu += g.player(P0).stats.get("ninjutsu").copied().unwrap_or(0);
    }
    assert!(ninjutsu > 0, "Yuriko never used ninjutsu in 20 games");
}
