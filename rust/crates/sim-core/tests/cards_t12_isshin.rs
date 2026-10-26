//! The Tier 1 isshin-mardu-attack-triggers cards (impls/t12_isshin.rs and the compiled patterns under
//! `// --- Tiers 1/2 (isshin)`) on hand-built positions. Seat 0 plays the Isshin deck. Each expectation follows the
//! card's Oracle text.

use sim_core::dsl::has_kw;
use sim_core::engine::combat;
use sim_core::engine::hooks::fire_trigger;
use sim_core::engine::values::{epow, etgh};
use sim_core::hooks::{Action, Call, Event, Sacrificed};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const DECK: &str = "isshin-mardu-attack-triggers";

fn game() -> Game {
    let mut g = table(&[DECK, "seph", "cloud"]);
    for p in [P0, P1, P2] {
        let cs: Vec<_> = (0..30).map(|_| card(&g, "Plains")).collect();
        g.player_mut(p).library.extend(cs); // something to draw
    }
    g
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn lives(g: &Game) -> (i32, i32, i32) {
    (g.player(P0).life, g.player(P1).life, g.player(P2).life)
}

// ======================================================== compiled: Fervent Charge, Ingris, Mishra, Severina
#[test]
fn fervent_charge_pumps_each_attacker() {
    let mut g = game();
    perm(&mut g, P0, "Fervent Charge");
    let (a, b) = (token(&mut g, P0, 1), token(&mut g, P0, 3));
    let c = token(&mut g, P0, 2);
    combat::attack_triggers(&mut g, P0, &[a, b], P1).unwrap();
    assert_eq!((epow(&g, a), etgh(&g, a)), (3, 3), "the 1/1 attacker is 3/3");
    assert_eq!(epow(&g, b), 5, "the 3/3 attacker is 5/5");
    assert_eq!(epow(&g, c), 2, "a creature that didn't attack isn't pumped");
}

#[test]
fn ingris_pings_each_opponent_per_attacker() {
    let mut g = game();
    perm(&mut g, P0, "Ingris Stingerquill");
    let atk = [token(&mut g, P0, 1), token(&mut g, P0, 1)];
    let (l0, l1, l2) = lives(&g);
    combat::attack_triggers(&mut g, P0, &atk, P1).unwrap();
    assert_eq!(lives(&g), (l0, l1 - 2, l2 - 2), "1 damage to each opponent per attacker");
}

#[test]
fn mishra_drains_by_the_number_of_attackers() {
    let mut g = game();
    let m = perm(&mut g, P0, "Mishra, Claimed by Gix");
    let atk = [m, token(&mut g, P0, 1), token(&mut g, P0, 1)];
    let (l0, l1, l2) = lives(&g);
    combat::attack_triggers(&mut g, P0, &atk, P1).unwrap();
    assert_eq!(lives(&g), (l0 + 3, l1 - 3, l2 - 3), "X = 3 attackers: each opponent loses 3, you gain 3");
    assert_eq!(sim_core::dsl::x_fallbacks(), 0, "X is named, not the stand-in 3");
}

#[test]
fn severina_drains_by_the_other_attackers() {
    let mut g = game();
    let s = perm(&mut g, P0, "Commissar Severina Raine");
    let atk = [s, token(&mut g, P0, 1), token(&mut g, P0, 1)];
    let (l0, l1, l2) = lives(&g);
    combat::attack_triggers(&mut g, P0, &atk, P1).unwrap();
    assert_eq!(lives(&g), (l0, l1 - 2, l2 - 2), "two other attackers: each opponent loses 2");
    let t = token(&mut g, P0, 1);
    let (_, l1, _) = lives(&g);
    combat::attack_triggers(&mut g, P0, &[t], P1).unwrap();
    assert_eq!(g.player(P1).life, l1, "Severina not attacking: no trigger");
}

// ======================================================== Curse of Opulence
#[test]
fn curse_of_opulence_makes_gold_when_its_player_is_attacked() {
    let mut g = game();
    g.player_mut(P2).life = 50;
    let curse = perm(&mut g, P0, "Curse of Opulence");
    assert_eq!(g.perm(curse).data.get(DataKey::T12Isshin), Some(&Val::Player(P2)), "it enchants the leader");
    let t1 = token(&mut g, P1, 2);
    let (tr0, tr1) = (g.player(P0).treasures, g.player(P1).treasures);
    combat::attack_triggers(&mut g, P1, &[t1], P2).unwrap();
    assert_eq!((g.player(P0).treasures, g.player(P1).treasures), (tr0 + 1, tr1 + 1), "you and the attacker");
    combat::attack_triggers(&mut g, P1, &[t1], P0).unwrap();
    assert_eq!(g.player(P0).treasures, tr0 + 1, "attacking another player: nothing");
    let t0 = token(&mut g, P0, 2);
    combat::attack_triggers(&mut g, P0, &[t0], P2).unwrap();
    assert_eq!((g.player(P0).treasures, g.player(P1).treasures), (tr0 + 2, tr1 + 1), "your own attack: one Gold");
}

// ======================================================== Reconnaissance
#[test]
fn reconnaissance_untaps_your_attackers_on_your_turn() {
    let mut g = game();
    let t = token(&mut g, P0, 2);
    assert!(!has_kw(&g, t, "vigilance"));
    perm(&mut g, P0, "Reconnaissance");
    assert!(has_kw(&g, t, "vigilance"), "your attackers come out of combat untapped");
    let theirs = token(&mut g, P1, 2);
    assert!(!has_kw(&g, theirs, "vigilance"), "not an opponent's creature");
    g.active = Some(P1);
    assert!(!has_kw(&g, t, "vigilance"), "only on your turn (your creatures attack)");
}

// ======================================================== Adriana, Captain of the Guard
#[test]
fn adriana_gives_melee_per_opponent_attacked() {
    let mut g = game();
    let a = perm(&mut g, P0, "Adriana, Captain of the Guard");
    let t = token(&mut g, P0, 1);
    let p = epow(&g, a);
    g.atk_targets = vec![(a, P1), (t, P2)];
    combat::attack_triggers(&mut g, P0, &[a, t], P1).unwrap();
    assert_eq!(epow(&g, a), p + 2, "two opponents attacked: +2/+2");
    assert_eq!((epow(&g, t), etgh(&g, t)), (3, 3), "the other attacker has melee too");
    let u = token(&mut g, P0, 1);
    g.atk_targets = vec![(u, P1)];
    combat::attack_triggers(&mut g, P0, &[u], P1).unwrap();
    assert_eq!(epow(&g, u), 2, "Adriana needn't attack: one opponent, +1/+1");
}

// ======================================================== Alesha, Who Laughs at Fate
#[test]
fn alesha_raid_returns_a_creature_with_mana_value_up_to_her_power() {
    let mut g = game();
    let al = perm(&mut g, P0, "Alesha, Who Laughs at Fate");
    for name in ["Hellrider", "Captain Lannery Storm"] {
        let c = take(&mut g, P0, name);
        g.player_mut(P0).gy.push(c);
    }
    fire_trigger(&mut g, Event::EndStep, Call::Player { p: P0 }).unwrap();
    assert!(on_bf(&g, P0, "Captain Lannery Storm").is_none(), "no attack this turn: no raid");
    combat::attack_triggers(&mut g, P0, &[al], P1).unwrap();
    assert_eq!(epow(&g, al), 3, "the attack counter (compiled)");
    fire_trigger(&mut g, Event::EndStep, Call::Player { p: P0 }).unwrap();
    assert!(on_bf(&g, P0, "Captain Lannery Storm").is_some(), "mana value 3 <= power 3 returns");
    assert!(on_bf(&g, P0, "Hellrider").is_none(), "mana value 4 stays");
}

// ======================================================== Breena, the Demagogue
#[test]
fn breena_rewards_attacks_on_an_opponent_with_more_life() {
    let mut g = game();
    let b = perm(&mut g, P0, "Breena, the Demagogue");
    g.player_mut(P2).life = 50;
    let t1 = token(&mut g, P1, 2);
    let h1 = g.player(P1).hand.len();
    combat::attack_triggers(&mut g, P1, &[t1], P2).unwrap();
    assert_eq!(g.player(P1).hand.len(), h1 + 1, "the attacking player draws");
    assert_eq!(g.perm(b).plus, 2, "two +1/+1 counters on your creature");
    g.player_mut(P2).life = 40;
    combat::attack_triggers(&mut g, P1, &[t1], P2).unwrap();
    assert_eq!(g.perm(b).plus, 2, "equal life: nothing");
    combat::attack_triggers(&mut g, P1, &[t1], P0).unwrap();
    assert_eq!(g.perm(b).plus, 2, "attacking you: nothing");
}

// ======================================================== Captain Lannery Storm
#[test]
fn lannery_storm_grows_when_you_sacrifice_a_treasure() {
    let mut g = game();
    let c = perm(&mut g, P0, "Captain Lannery Storm");
    let tr = g.player(P0).treasures;
    combat::attack_triggers(&mut g, P0, &[c], P1).unwrap();
    assert_eq!(g.player(P0).treasures, tr + 1, "attack: a Treasure (compiled)");
    let p = epow(&g, c);
    fire_trigger(&mut g, Event::Sacrifice, Call::Sacrifice { p: P0, what: Sacrificed::Token("Treasure") }).unwrap();
    assert_eq!(epow(&g, c), p + 1, "+1/+0");
    fire_trigger(&mut g, Event::Sacrifice, Call::Sacrifice { p: P0, what: Sacrificed::Token("Clue") }).unwrap();
    fire_trigger(&mut g, Event::Sacrifice, Call::Sacrifice { p: P1, what: Sacrificed::Token("Treasure") }).unwrap();
    assert_eq!(epow(&g, c), p + 1, "not a Clue, not an opponent's Treasure");
}

// ======================================================== Firebender Ascension
#[test]
fn firebender_ascension_counts_attack_triggers_and_copies_from_four() {
    let mut g = game();
    let fa = perm(&mut g, P0, "Firebender Ascension");
    let c = perm(&mut g, P0, "Captain Lannery Storm");
    let tr = g.player(P0).treasures;
    combat::attack_triggers(&mut g, P0, &[c], P1).unwrap();
    assert_eq!(g.perm(fa).data.get(DataKey::T12Isshin), Some(&Val::Int(1)), "one quest counter");
    assert_eq!(g.player(P0).treasures, tr + 1, "no copy yet");
    g.perm_mut(fa).data.set(DataKey::T12Isshin, Val::Int(3));
    let tr = g.player(P0).treasures;
    combat::attack_triggers(&mut g, P0, &[c], P1).unwrap();
    assert_eq!(g.player(P0).treasures, tr + 2, "the fourth counter copies the Treasure trigger");
    let t = token(&mut g, P0, 2);
    combat::attack_triggers(&mut g, P0, &[t], P1).unwrap();
    assert_eq!(g.perm(fa).data.get(DataKey::T12Isshin), Some(&Val::Int(4)), "a vanilla attacker: no counter");
}

// ======================================================== Hammers of Moradin
#[test]
fn hammers_of_moradin_taps_a_creature_of_each_opponent() {
    let mut g = game();
    let h = perm(&mut g, P0, "Hammers of Moradin");
    let (small, big) = (token(&mut g, P1, 1), token(&mut g, P1, 4));
    let other = token(&mut g, P2, 2);
    combat::attack_triggers(&mut g, P0, &[h], P1).unwrap();
    assert!(g.perm(big).tapped && !g.perm(small).tapped, "the best blocker of each opponent");
    assert!(g.perm(other).tapped, "the other opponent's too");
}

// ======================================================== Otharri, Suns' Glory
#[test]
fn otharri_makes_rebels_per_experience_and_returns_from_the_graveyard() {
    let mut g = game();
    let o = perm(&mut g, P0, "Otharri, Suns' Glory");
    let new = combat::attack_triggers(&mut g, P0, &[o], P1).unwrap();
    assert_eq!((g.player(P0).experience, new.len()), (1, 1), "an experience counter, one attacking Rebel");
    let new = combat::attack_triggers(&mut g, P0, &[o], P1).unwrap();
    assert_eq!((g.player(P0).experience, new.len()), (2, 2), "two counters: two Rebels");
    assert!(new.iter().all(|&m| g.perm(m).tapped && epow(&g, m) == 2), "2/2s, tapped and attacking");
    for m in new {
        g.perm_mut(m).tapped = false;
    }
    sim_core::engine::zones::die(&mut g, o, "destroy").unwrap();
    let c = card(&g, "Otharri, Suns' Glory");
    assert!(g.player(P0).gy.contains(&c));
    lands(&mut g, P0, "Plateau", 4, false);
    let f = g.registry.get(c).and_then(|i| i.gy_options).unwrap();
    assert!(f(&mut g, c, P0, Some(false)).unwrap().is_empty(), "not before combat");
    let opts = f(&mut g, c, P0, Some(true)).unwrap();
    let Some(Action::Plan { f: run, arg }) = opts[0].act.clone() else { panic!("a plan") };
    assert!(run(&mut g, P0, arg).unwrap());
    let back = on_bf(&g, P0, "Otharri, Suns' Glory").expect("back on the battlefield");
    assert!(g.perm(back).tapped, "it returns tapped");
}

// ======================================================== Raiyuu, Storm's Edge
#[test]
fn raiyuu_untaps_a_lone_warrior_and_adds_a_combat() {
    let mut g = game();
    perm(&mut g, P0, "Raiyuu, Storm's Edge");
    let al = perm(&mut g, P0, "Alesha, Who Laughs at Fate");
    let t = token(&mut g, P0, 2);
    g.player_mut(P0).combat_no = 1;
    g.perm_mut(al).tapped = true;
    combat::attack_triggers(&mut g, P0, &[al, t], P1).unwrap();
    assert!(g.perm(al).tapped && g.player(P0).extra_combats == 0, "two attackers: not alone");
    combat::attack_triggers(&mut g, P0, &[al], P1).unwrap();
    assert!(!g.perm(al).tapped, "a Warrior attacking alone untaps");
    assert_eq!(g.player(P0).extra_combats, 1, "first combat: an additional one");
    g.player_mut(P0).combat_no = 2;
    g.perm_mut(t).tapped = true;
    combat::attack_triggers(&mut g, P0, &[al], P1).unwrap();
    assert_eq!(g.player(P0).extra_combats, 1, "the second combat adds none");
    combat::attack_triggers(&mut g, P0, &[t], P1).unwrap();
    assert!(g.perm(t).tapped, "a token that isn't a Samurai or Warrior stays tapped");
}

// ======================================================== Tectonic Giant
#[test]
fn tectonic_giant_burns_each_opponent() {
    let mut g = game();
    let tg = perm(&mut g, P0, "Tectonic Giant");
    let (l0, l1, l2) = lives(&g);
    combat::attack_triggers(&mut g, P0, &[tg], P1).unwrap();
    assert_eq!(lives(&g), (l0, l1 - 3, l2 - 3), "3 damage to each opponent");
}
