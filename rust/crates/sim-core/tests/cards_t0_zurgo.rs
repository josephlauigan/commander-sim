//! Tier 0 zurgo-mardu-surge card code (impls/t0_zurgo.rs) on hand-built positions: myriad, mobilize, goad, fabricate,
//! hideaway and lieutenant, token doubling and replacement, and the cards that use them. Each expectation follows the
//! card's Oracle text.

use sim_core::cardcode;
use sim_core::engine::hooks::run_hook;
use sim_core::engine::{combat, zones};
use sim_core::hooks::{Action, Call, Event};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const ZURGO: &str = "zurgo-mardu-surge";

/// the zurgo deck against two others (three seats, so there is "another opponent" for myriad)
fn pod() -> Game {
    table(&[ZURGO, "seph", "sauron"])
}

fn tokens(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).token && g.perm(m).on_bf).collect()
}

/// p attacks d with these creatures: every attacker's defender recorded, then the attack triggers run
fn attack(g: &mut Game, p: PlayerId, atk: &[PermId], d: PlayerId) -> Vec<PermId> {
    g.atk_targets = atk.iter().map(|&m| (m, d)).collect();
    for &m in atk {
        g.perm_mut(m).tapped = true;
    }
    g.in_combat = atk.to_vec();
    combat::attack_triggers(g, p, atk, d).unwrap()
}

fn token_with(g: &mut Game, p: PlayerId, name: &str) -> PermId {
    let m = token(g, p, 1);
    g.perm_mut(m).ttypes = vec![sim_core::sym::intern(name)];
    m
}

fn card_hook(g: &Game, name: &str) -> sim_core::hooks::CardImpl {
    *g.registry.get(card(g, name)).unwrap_or_else(|| panic!("no card code for {name}"))
}

// ------------------------------------------------------------------ Zurgo Stormrender
#[test]
fn zurgo_attack_makes_a_tapped_attacking_warrior_that_goes_at_the_end_step() {
    let mut g = pod();
    let z = perm(&mut g, P0, "Zurgo Stormrender");
    let new = attack(&mut g, P0, &[z], P1);
    assert_eq!(new.len(), 1, "mobilize 1");
    let w = new[0];
    assert!(g.perm(w).token && g.perm(w).tapped && g.perm(w).warrior);
    assert_eq!((g.perm(w).pow, g.perm(w).tgh), (1, 1));
    assert!(g.perm(w).data.truthy(DataKey::EotSac), "sacrificed at the next end step");
    assert_eq!(g.atk_targets.iter().find(|x| x.0 == w).map(|x| x.1), Some(P1));
}

#[test]
fn zurgo_draws_when_an_attacking_token_leaves_and_drains_otherwise() {
    let mut g = pod();
    perm(&mut g, P0, "Zurgo Stormrender");
    let t = token(&mut g, P0, 1);
    let hand0 = g.player(P0).hand.len();
    let (l1, l2) = (g.player(P1).life, g.player(P2).life);
    zones::die(&mut g, t, "sac").unwrap(); // not attacking: each opponent loses 1
    assert_eq!((g.player(P1).life, g.player(P2).life), (l1 - 1, l2 - 1));
    assert_eq!(g.player(P0).hand.len(), hand0);
    let t = token(&mut g, P0, 1);
    g.in_combat = vec![t];
    zones::die(&mut g, t, "destroy").unwrap(); // attacking: a card
    assert_eq!(g.player(P0).hand.len(), hand0 + 1);
    assert_eq!(g.player(P1).life, l1 - 1);
    // an opponent's token doesn't count, a nontoken creature neither
    let theirs = token(&mut g, P1, 1);
    zones::die(&mut g, theirs, "destroy").unwrap();
    let c = perm(&mut g, P0, "Selfless Spirit");
    zones::die(&mut g, c, "destroy").unwrap();
    assert_eq!(g.player(P0).hand.len(), hand0 + 1);
    assert_eq!(g.player(P1).life, l1 - 1);
}

#[test]
fn zurgo_is_cast_early_by_the_pool_ai() {
    let mut g = pod();
    let c = card(&g, "Zurgo Stormrender");
    g.player_mut(P0).turns = 2;
    assert!(sim_core::ai::pool::generic_prio(&g, P0, c) >= 80, "the commander of a live deck comes first");
}

// ------------------------------------------------------------------ myriad
#[test]
fn goldlust_triad_myriad_copies_attack_the_other_opponents_and_are_exiled() {
    let mut g = pod();
    let t = perm(&mut g, P0, "Goldlust Triad");
    let new = attack(&mut g, P0, &[t], P1);
    assert_eq!(new.len(), 1, "one other opponent");
    let c = new[0];
    assert!(g.perm(c).token && g.perm(c).tapped && g.perm(c).cd == g.perm(t).cd);
    assert_eq!(g.atk_targets.iter().find(|x| x.0 == c).map(|x| x.1), Some(P2), "attacks the other opponent");
    assert!(g.perm(c).data.truthy(DataKey::MyriadTok));
    sim_core::impls::t0_zurgo::end_of_combat(&mut g, P0).unwrap();
    assert!(!g.perm(c).on_bf, "exiled at end of combat");
}

#[test]
fn myriad_copies_go_to_the_right_groups_and_alone_it_does_nothing() {
    let mut g = table(&[ZURGO, "seph"]);
    let t = perm(&mut g, P0, "Goldlust Triad");
    assert!(attack(&mut g, P0, &[t], P1).is_empty(), "no other opponent, no copy");
    let mut g = pod();
    let t = perm(&mut g, P0, "Goldlust Triad");
    let new = attack(&mut g, P0, &[t], P1);
    let mut groups = vec![(P1, vec![t])];
    sim_core::impls::t0_zurgo::route_new_attackers(&mut g, P0, &[t, new[0]], &new, P1, &mut groups).unwrap();
    assert_eq!(groups, vec![(P1, vec![t]), (P2, vec![new[0]])]);
}

#[test]
fn legion_loyalty_gives_every_creature_myriad_and_tokens_copy_as_tokens() {
    let mut g = pod();
    perm(&mut g, P0, "Legion Loyalty");
    let a = token_with(&mut g, P0, "goblin");
    g.perm_mut(a).fly = true;
    let b = perm(&mut g, P0, "Selfless Spirit");
    let new = attack(&mut g, P0, &[a, b], P1);
    assert_eq!(new.len(), 2);
    let copy = new.iter().copied().find(|&m| g.perm(m).cd.is_none()).unwrap();
    assert!(g.perm(copy).fly && g.perm(copy).ttypes == vec!["goblin"]);
    assert_eq!(g.atk_targets.iter().find(|x| x.0 == copy).map(|x| x.1), Some(P2));
}

#[test]
fn blade_of_selves_gives_the_equipped_creature_myriad() {
    let mut g = pod();
    let blade = perm(&mut g, P0, "Blade of Selves");
    let a = perm(&mut g, P0, "Selfless Spirit");
    let b = perm(&mut g, P0, "Ophiomancer");
    g.perm_mut(blade).attached = Some(a);
    let new = attack(&mut g, P0, &[a, b], P1);
    assert_eq!(new.len(), 1);
    assert_eq!(g.perm(new[0]).cd, g.perm(a).cd);
}

#[test]
fn myriad_never_copies_a_legendary_creature() {
    let mut g = pod();
    perm(&mut g, P0, "Legion Loyalty");
    let z = perm(&mut g, P0, "Zurgo Stormrender");
    let new = attack(&mut g, P0, &[z], P1);
    assert!(new.iter().all(|&m| g.perm(m).warrior), "only the Warrior token of mobilize");
}

#[test]
fn ironwill_forger_gives_an_attacking_nonlegendary_creature_myriad_with_the_commander_out() {
    let mut g = pod();
    let f = perm(&mut g, P0, "Ironwill Forger");
    let big = perm(&mut g, P0, "Siege-Gang Commander");
    g.atk_targets = vec![(big, P1)];
    cardcode::combat_start(&mut g, P0).unwrap();
    assert!(!g.perm(big).eot_kw.contains(&"myriad"), "no commander, no lieutenant");
    let z = perm(&mut g, P0, "Zurgo Stormrender");
    g.perm_mut(z).is_cmd = true;
    g.atk_targets = vec![(z, P1), (big, P1), (f, P1)];
    cardcode::combat_start(&mut g, P0).unwrap();
    assert!(g.perm(big).eot_kw.contains(&"myriad") || g.perm(f).eot_kw.contains(&"myriad"));
    assert!(!g.perm(z).eot_kw.contains(&"myriad"), "nonlegendary only");
}

// ------------------------------------------------------------------ Divine Visitation, Kaya
#[test]
fn divine_visitation_turns_tokens_into_angels() {
    let mut g = pod();
    let spec = zones::Tokens::new(2, 1);
    assert_eq!(zones::make_tokens(&mut g, P0, spec).unwrap().len(), 2);
    perm(&mut g, P0, "Divine Visitation");
    let made = zones::make_tokens(&mut g, P0, zones::Tokens::new(3, 1)).unwrap();
    assert_eq!(made.len(), 3, "that many");
    for m in made {
        let x = g.perm(m);
        assert_eq!((x.pow, x.tgh), (4, 4));
        assert!(x.fly && x.vig && x.ttypes == vec!["angel"]);
    }
    let theirs = zones::make_tokens(&mut g, P1, zones::Tokens::new(1, 1)).unwrap()[0];
    assert_eq!(g.perm(theirs).pow, 1, "only your own tokens");
}

fn kaya_ability(g: &Game, i: usize) -> sim_core::impls::common::WalkerAb {
    card_hook(g, "Kaya, Geist Hunter").walker.unwrap()[i]
}

#[test]
fn kaya_minus_two_doubles_tokens_until_end_of_turn() {
    let mut g = pod();
    let k = perm(&mut g, P0, "Kaya, Geist Hunter");
    assert_eq!(zones::make_tokens(&mut g, P0, zones::Tokens::new(2, 1)).unwrap().len(), 2);
    assert_eq!(kaya_ability(&g, 1).delta, -2);
    (kaya_ability(&g, 1).eff)(&mut g, P0, k).unwrap();
    assert_eq!(zones::make_tokens(&mut g, P0, zones::Tokens::new(2, 1)).unwrap().len(), 4);
    zones::add_treasure(&mut g, P0, 1).unwrap();
    assert_eq!(g.player(P0).treasures, 2, "Treasures too");
    assert_eq!(zones::make_tokens(&mut g, P1, zones::Tokens::new(1, 1)).unwrap().len(), 1, "only your tokens");
    g.round += 1; // next turn
    assert_eq!(zones::make_tokens(&mut g, P0, zones::Tokens::new(2, 1)).unwrap().len(), 2);
}

#[test]
fn kaya_plus_one_gives_deathtouch_and_a_counter_on_a_token_and_the_ultimate_exiles_graveyards() {
    let mut g = pod();
    let k = perm(&mut g, P0, "Kaya, Geist Hunter");
    let c = perm(&mut g, P0, "Selfless Spirit");
    let t = token(&mut g, P0, 2);
    assert!(!g.perm(c).dt);
    (kaya_ability(&g, 0).eff)(&mut g, P0, k).unwrap();
    assert!(g.perm(c).dt && g.perm(t).dt);
    assert_eq!(g.perm(t).plus, 1, "the token got the counter");
    assert_eq!(g.perm(c).plus, 0);
    sim_core::impls::t0_zurgo::end_cleanup(&mut g, P0);
    assert!(!g.perm(c).dt && !g.perm(t).dt, "until end of turn");
    let gy0: Vec<_> = (0..3).map(|_| card(&g, "Sol Ring")).collect();
    g.player_mut(P0).gy.extend(&gy0);
    let c2 = card(&g, "Abrade");
    g.player_mut(P1).gy.push(c2);
    let before = tokens(&g, P0).len();
    (kaya_ability(&g, 2).eff)(&mut g, P0, k).unwrap();
    assert!(g.player(P0).gy.is_empty() && g.player(P1).gy.is_empty());
    assert_eq!(tokens(&g, P0).len(), before + 4, "a Spirit for each card exiled");
}

// ------------------------------------------------------------------ the other creatures
#[test]
fn angel_of_invention_fabricates_two_servos() {
    let mut g = pod();
    perm(&mut g, P0, "Selfless Spirit");
    let a = perm(&mut g, P0, "Angel of Invention");
    let servos: Vec<_> = tokens(&g, P0).into_iter().filter(|&m| g.perm(m).ttypes.contains(&"servo")).collect();
    assert_eq!(servos.len(), 2);
    assert_eq!(g.perm(a).plus, 0);
    assert!(g.perm(a).fly && g.perm(a).vig && g.perm(a).lifelink);
    for s in servos {
        assert_eq!(sim_core::engine::values::epow(&g, s), 2, "the anthem");
    }
}

#[test]
fn chittering_witch_makes_a_rat_for_each_opponent_and_can_sacrifice_for_minus_two() {
    let mut g = pod();
    let w = perm(&mut g, P0, "Chittering Witch");
    let rats: Vec<_> = tokens(&g, P0).into_iter().filter(|&m| g.perm(m).ttypes.contains(&"rat")).collect();
    assert_eq!(rats.len(), 2, "two opponents");
    let victim = perm(&mut g, P1, "Solemn Simulacrum");
    let _ = victim;
    let small = perm(&mut g, P1, "Ophiomancer"); // 2/2
    lands(&mut g, P0, "Swamp", 2, false);
    let hook = card_hook(&g, "Chittering Witch");
    let opts = (hook.options.unwrap())(&mut g, w, P0, Some(true)).unwrap();
    assert!(!opts.is_empty(), "a Rat for the 2/2");
    let Some(Action::Ability { src, f, arg }) = opts.into_iter().next().unwrap().act else { panic!() };
    assert!(f(&mut g, src, P0, arg).unwrap());
    assert!(!g.perm(small).on_bf || g.perm(small).eot_pt.1 <= -2, "-2/-2");
    assert_eq!(
        tokens(&g, P0).iter().filter(|&&m| g.perm(m).ttypes.contains(&"rat")).count(),
        1,
        "a Rat was sacrificed"
    );
}

#[test]
fn twilight_drover_grows_on_any_token_leaving_and_makes_spirits() {
    let mut g = pod();
    let d = perm(&mut g, P0, "Twilight Drover");
    let mine = token(&mut g, P0, 1);
    let theirs = token(&mut g, P1, 1);
    zones::die(&mut g, mine, "destroy").unwrap();
    zones::exile_perm(&mut g, theirs).unwrap();
    assert_eq!(g.perm(d).plus, 2, "any creature token, however it left");
    lands(&mut g, P0, "Plains", 3, false);
    let hook = card_hook(&g, "Twilight Drover");
    let opts = (hook.options.unwrap())(&mut g, d, P0, Some(true)).unwrap();
    let Some(Action::Ability { src, f, arg }) = opts.into_iter().next().unwrap().act else { panic!() };
    assert!(f(&mut g, src, P0, arg).unwrap());
    assert_eq!(g.perm(d).plus, 1);
    let spirits: Vec<_> = tokens(&g, P0).into_iter().filter(|&m| g.perm(m).ttypes.contains(&"spirit")).collect();
    assert_eq!(spirits.len(), 2);
    assert!(spirits.iter().all(|&m| g.perm(m).fly));
}

#[test]
fn grenzo_goads_or_exiles_the_top_card() {
    let mut g = pod();
    let gr = perm(&mut g, P0, "Grenzo, Havoc Raiser");
    let a = perm(&mut g, P0, "Selfless Spirit");
    let big = perm(&mut g, P1, "Sun Titan");
    let call = Call::CombatDamage { p: P0, a, d: P1, dmg: 1 };
    // no mana: goad their creature
    run_hook(&mut g, gr, Event::CombatDamage, &call).unwrap();
    assert!(g.perm(big).data.get(DataKey::Goaded).is_some(), "goaded");
    // the goaded creature attacks, and not its goader
    g.player_mut(P1).turns = 3;
    g.perm_mut(big).sick = false;
    let mut groups = vec![(P0, vec![big])];
    sim_core::impls::t0_zurgo::goad_attackers(&mut g, P1, &mut groups, P0).unwrap();
    assert_eq!(groups, vec![(P2, vec![big])], "another player if able");
    // it must attack even if the AI would not send it
    let mut groups: Vec<(PlayerId, Vec<PermId>)> = vec![];
    sim_core::impls::t0_zurgo::goad_attackers(&mut g, P1, &mut groups, P0).unwrap();
    assert_eq!(groups.len(), 1);
    // the goad ends when its player's next turn starts
    g.player_mut(P0).turns += 1;
    let mut groups: Vec<(PlayerId, Vec<PermId>)> = vec![];
    sim_core::impls::t0_zurgo::goad_attackers(&mut g, P1, &mut groups, P0).unwrap();
    assert!(groups.is_empty());
}

#[test]
fn grenzo_exile_mode_takes_the_top_card_to_cast_this_turn() {
    let mut g = pod();
    let gr = perm(&mut g, P0, "Grenzo, Havoc Raiser");
    let a = perm(&mut g, P0, "Selfless Spirit");
    lands(&mut g, P0, "Plains", 4, false);
    let top = card(&g, "Sol Ring");
    g.player_mut(P1).library.push(top);
    let n = g.player(P1).library.len();
    run_hook(&mut g, gr, Event::CombatDamage, &Call::CombatDamage { p: P0, a, d: P1, dmg: 1 }).unwrap();
    assert_eq!(g.player(P1).library.len(), n - 1);
    assert!(g.player(P0).hand.contains(&top) && g.player(P0).agent_ids.contains(&top), "castable with any mana");
    assert!(g.player(P0).impulse.contains(&top), "until end of turn");
}

#[test]
fn mindblade_render_draws_once_per_combat_when_a_warrior_hurts_an_opponent() {
    let mut g = pod();
    let m = perm(&mut g, P0, "Mindblade Render");
    let z = perm(&mut g, P0, "Zurgo Stormrender"); // a Warrior
    let other = perm(&mut g, P0, "Selfless Spirit");
    let (hand, life) = (g.player(P0).hand.len(), g.player(P0).life);
    run_hook(&mut g, m, Event::CombatDamage, &Call::CombatDamage { p: P0, a: other, d: P1, dmg: 2 }).unwrap();
    assert_eq!(g.player(P0).hand.len(), hand, "not a Warrior");
    run_hook(&mut g, m, Event::CombatDamage, &Call::CombatDamage { p: P0, a: z, d: P1, dmg: 3 }).unwrap();
    run_hook(&mut g, m, Event::CombatDamage, &Call::CombatDamage { p: P0, a: z, d: P2, dmg: 3 }).unwrap();
    assert_eq!(g.player(P0).hand.len(), hand + 1, "once");
    assert_eq!(g.player(P0).life, life - 1);
    // damage dealt to its controller doesn't count
    let (hand, life) = (g.player(P0).hand.len(), g.player(P0).life);
    let w = perm(&mut g, P1, "Zurgo Stormrender");
    g.player_mut(P0).combat_no = 7;
    run_hook(&mut g, m, Event::CombatDamage, &Call::CombatDamage { p: P1, a: w, d: P0, dmg: 3 }).unwrap();
    assert_eq!((g.player(P0).hand.len(), g.player(P0).life), (hand, life));
}

#[test]
fn myr_battlesphere_taps_myr_for_pump_and_damage() {
    let mut g = pod();
    let b = perm(&mut g, P0, "Myr Battlesphere");
    let myr: Vec<_> = tokens(&g, P0);
    assert_eq!(myr.len(), 4, "four Myr");
    g.perm_mut(myr[0]).tapped = true; // one already attacked
    let life = g.player(P1).life;
    attack(&mut g, P0, &[b], P1);
    assert_eq!(g.player(P1).life, life - 3);
    assert_eq!(g.perm(b).eot_pt.0, 3);
    assert!(myr.iter().all(|&m| g.perm(m).tapped));
}

#[test]
fn redoubled_stormsinger_copies_tokens_that_entered_this_turn() {
    let mut g = pod();
    let old = token_with(&mut g, P0, "goblin");
    g.turn_enter = g.enter_no; // the old token is from an earlier turn
    let s = perm(&mut g, P0, "Redoubled Stormsinger");
    let fresh = token_with(&mut g, P0, "spirit");
    g.perm_mut(fresh).fly = true;
    let new = attack(&mut g, P0, &[s], P1);
    assert_eq!(new.len(), 1);
    let c = new[0];
    assert!(g.perm(c).fly && g.perm(c).ttypes == vec!["spirit"] && g.perm(c).tapped);
    assert!(g.perm(c).data.truthy(DataKey::EotSac));
    let _ = old;
}

#[test]
fn neriv_exiles_cards_for_differently_named_tokens_and_plays_them_after_a_commander_attacks() {
    let mut g = pod();
    let n = perm(&mut g, P0, "Neriv, Crackling Vanguard");
    token_with(&mut g, P0, "goblin");
    token_with(&mut g, P0, "goblin");
    token_with(&mut g, P0, "spirit");
    zones::add_treasure(&mut g, P0, 1).unwrap();
    let lib = g.player(P0).library.len();
    attack(&mut g, P0, &[n], P1);
    assert_eq!(g.player(P0).library.len(), lib - 3, "goblin, spirit, Treasure");
    assert_eq!(g.player(P0).neriv.len(), 3);
    assert!(g.player(P0).neriv.iter().all(|c| g.player(P0).exile.contains(c)), "no commander attacked");
    // a commander attacks: they come to the hand for the turn
    let z = perm(&mut g, P0, "Zurgo Stormrender");
    g.perm_mut(z).is_cmd = true;
    attack(&mut g, P0, &[z], P1);
    assert!(g.player(P0).neriv.iter().all(|c| g.player(P0).hand.contains(c)));
    sim_core::impls::t0_zurgo::end_cleanup(&mut g, P0);
    assert!(g.player(P0).neriv.iter().all(|c| g.player(P0).exile.contains(c) && !g.player(P0).hand.contains(c)));
}

#[test]
fn ainok_strike_leader_makes_a_goblin_attacking_each_opponent() {
    let mut g = pod();
    let a = perm(&mut g, P0, "Ainok Strike Leader");
    let new = attack(&mut g, P0, &[a], P1);
    assert_eq!(new.len(), 2);
    let dests: Vec<_> = new.iter().map(|&m| g.atk_targets.iter().find(|x| x.0 == m).unwrap().1).collect();
    assert!(dests.contains(&P1) && dests.contains(&P2));
    assert!(new.iter().all(|&m| g.perm(m).tapped && g.perm(m).ttypes == vec!["goblin"]));
    // the commander attacking alone also triggers it
    let mut g = pod();
    perm(&mut g, P0, "Ainok Strike Leader");
    let z = perm(&mut g, P0, "Zurgo Stormrender");
    g.perm_mut(z).is_cmd = true;
    let new = attack(&mut g, P0, &[z], P1);
    assert_eq!(new.iter().filter(|&&m| g.perm(m).ttypes == vec!["goblin"]).count(), 2);
    // and a plain creature attacking alone doesn't
    let mut g = pod();
    perm(&mut g, P0, "Ainok Strike Leader");
    let o = perm(&mut g, P0, "Selfless Spirit");
    assert!(attack(&mut g, P0, &[o], P1).is_empty());
}

#[test]
fn within_range_drains_for_the_creatures_attacking_each_opponent() {
    let mut g = pod();
    perm(&mut g, P0, "Within Range");
    assert_eq!(tokens(&g, P0).len(), 2, "two Warriors");
    let a = perm(&mut g, P0, "Selfless Spirit");
    let b = perm(&mut g, P0, "Ophiomancer");
    let c = perm(&mut g, P0, "Viscera Seer");
    let (l1, l2) = (g.player(P1).life, g.player(P2).life);
    g.atk_targets = vec![(a, P1), (b, P1), (c, P2)];
    for m in [a, b, c] {
        g.perm_mut(m).tapped = true;
    }
    let mut groups = vec![(P1, vec![a, b]), (P2, vec![c])];
    sim_core::impls::t0_zurgo::route_new_attackers(&mut g, P0, &[a, b, c], &[], P1, &mut groups).unwrap();
    assert_eq!((g.player(P1).life, g.player(P2).life), (l1 - 2, l2 - 1));
}

#[test]
fn commanders_insignia_counts_commander_casts() {
    let mut g = pod();
    let c = perm(&mut g, P0, "Selfless Spirit");
    perm(&mut g, P0, "Commander's Insignia");
    let base = sim_core::engine::values::epow(&g, c);
    g.player_mut(P0).tax = 4; // cast twice from the command zone
    assert_eq!(sim_core::engine::values::epow(&g, c), base + 2);
    g.player_mut(P0).tax = 0;
    assert_eq!(sim_core::engine::values::epow(&g, c), base);
}

// ------------------------------------------------------------------ the spells
#[test]
fn shadow_summoning_makes_two_tapped_flying_spirits() {
    let mut g = pod();
    let c = card(&g, "Shadow Summoning");
    let f = card_hook(&g, "Shadow Summoning").resolve.unwrap();
    f(&mut g, P0, c, &Ctx::default()).unwrap();
    let t = tokens(&g, P0);
    assert_eq!(t.len(), 2);
    assert!(t.iter().all(|&m| g.perm(m).tapped && g.perm(m).fly && g.perm(m).pow == 1));
}

#[test]
fn will_of_the_mardu_makes_warriors_and_deals_damage() {
    // without a commander: the better mode (their 3/3 dies to three creatures' damage)
    let mut g = pod();
    for _ in 0..3 {
        token(&mut g, P0, 1);
    }
    let big = perm(&mut g, P1, "Sun Titan");
    let weak = perm(&mut g, P1, "Ophiomancer");
    let before = creature_count(&g, P0);
    let c = card(&g, "Will of the Mardu");
    let f = card_hook(&g, "Will of the Mardu").resolve.unwrap();
    f(&mut g, P0, c, &Ctx::default()).unwrap();
    let tokens_made = creature_count(&g, P0) > before;
    assert!(g.perm(big).on_bf);
    assert_ne!(tokens_made, !g.perm(weak).on_bf, "exactly one mode without the commander");
    // with the commander: both
    let mut g = pod();
    let z = perm(&mut g, P0, "Zurgo Stormrender");
    g.perm_mut(z).is_cmd = true;
    token(&mut g, P0, 1);
    let theirs: Vec<_> = (0..3).map(|_| perm(&mut g, P1, "Ophiomancer")).collect();
    let before = creature_count(&g, P0);
    let c = card(&g, "Will of the Mardu");
    f(&mut g, P0, c, &Ctx::default()).unwrap();
    assert_eq!(creature_count(&g, P0), before + 3, "Warriors equal to the creatures the player with most controls");
    assert_eq!(theirs.iter().filter(|&&m| !g.perm(m).on_bf).count(), 1, "damage equal to our creatures kills one");
}

fn creature_count(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.is_creature(m) && g.perm(m).on_bf).count()
}

#[test]
fn eliminate_the_competition_sacrifices_x_to_destroy_x() {
    let mut g = pod();
    lands(&mut g, P0, "Swamp", 5, false);
    let hand = hand(&mut g, P0, &["Eliminate the Competition"]);
    let c = hand[0];
    let t1 = token(&mut g, P0, 1);
    let t2 = token(&mut g, P0, 1);
    let big1 = perm(&mut g, P1, "Sun Titan");
    let big2 = perm(&mut g, P2, "Siege-Gang Commander");
    let hook = card_hook(&g, "Eliminate the Competition");
    let opts = (hook.hand_options.unwrap())(&mut g, c, P0, Some(false)).unwrap();
    assert!(!opts.is_empty());
    let Some(Action::Plan { f, arg }) = opts.into_iter().next().unwrap().act else { panic!() };
    assert!(f(&mut g, P0, arg).unwrap());
    let sacked = [t1, t2].iter().filter(|&&m| !g.perm(m).on_bf).count();
    let destroyed = [big1, big2].iter().filter(|&&m| !g.perm(m).on_bf).count();
    assert!(sacked >= 1 && sacked == destroyed, "X creatures sacrificed ({sacked}), X destroyed ({destroyed})");
    // not worth casting with nothing to sacrifice
    let mut g = pod();
    lands(&mut g, P0, "Swamp", 5, false);
    let c = hand_card(&mut g, P0, "Eliminate the Competition");
    perm(&mut g, P1, "Sun Titan");
    let opts = (hook.hand_options.unwrap())(&mut g, c, P0, Some(false)).unwrap();
    assert!(opts.is_empty());
}

fn hand_card(g: &mut Game, p: PlayerId, name: &str) -> sim_core::ids::CardId {
    hand(g, p, &[name])[0]
}

// ------------------------------------------------------------------ Windbrisk Heights
#[test]
fn windbrisk_heights_hides_a_card_and_plays_it_after_a_three_creature_attack() {
    let mut g = pod();
    let l = lands(&mut g, P0, "Windbrisk Heights", 1, false)[0];
    lands(&mut g, P0, "Plains", 1, false);
    let lib = g.player(P0).library.len();
    cardcode::land_etb(&mut g, P0, l).unwrap();
    assert_eq!(g.player(P0).library.len(), lib - 1, "one card hidden, the other three back on the bottom");
    let Some(Val::Card(c)) = g.land(l).data.get(DataKey::Hidden).cloned() else { panic!("nothing hidden") };
    let hook = card_hook(&g, "Windbrisk Heights");
    let opts = (hook.land_options.unwrap())(&mut g, l, P0, Some(true)).unwrap();
    assert!(opts.is_empty(), "not attacked with three");
    g.player_mut(P0).atk_count = (Some(g.turn_stamp()), 3);
    let opts = (hook.land_options.unwrap())(&mut g, l, P0, Some(true)).unwrap();
    assert_eq!(opts.len(), 1);
    let Some(Action::Plan { f, arg }) = opts.into_iter().next().unwrap().act else { panic!() };
    assert!(f(&mut g, P0, arg).unwrap());
    assert!(g.land(l).data.get(DataKey::Hidden) == Some(&Val::None) || g.land(l).data.get(DataKey::Hidden).is_none());
    assert!(g.land(l).tapped);
    let _ = c;
}

#[test]
fn zurgo_can_be_cast_from_the_command_zone_with_three_lands() {
    let mut g = pod();
    lands(&mut g, P0, "Swamp", 1, false);
    lands(&mut g, P0, "Battlefield Forge", 1, false);
    lands(&mut g, P0, "Dragonskull Summit", 1, false);
    let c = card(&g, "Zurgo Stormrender");
    assert!(g.player(P0).cmd_zone_of(c));
    assert!(sim_core::ai::brain::do_cast(&mut g, P0, c, None).unwrap(), "RWB from Swamp, Forge, Summit");
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).cd == Some(c)));
}

#[test]
fn zurgo_is_cast_from_a_trace_board() {
    let mut g = pod();
    lands(&mut g, P0, "Plains", 1, false);
    lands(&mut g, P0, "Swamp", 1, false);
    lands(&mut g, P0, "Battlefield Forge", 1, false);
    lands(&mut g, P0, "Dragonskull Summit", 1, false);
    perm(&mut g, P0, "Talisman of Conviction");
    perm(&mut g, P0, "Idol of Oblivion");
    let c = card(&g, "Zurgo Stormrender");
    assert!(sim_core::ai::brain::do_cast(&mut g, P0, c, None).unwrap());
}

// ------------------------------------------------------------------ Bone Devourer, Thalisse
#[test]
fn bone_devourer_enters_with_counters_for_deaths_and_draws_that_many() {
    let mut g = pod();
    g.died_count = (Some(g.turn_stamp()), 3);
    let c = take(&mut g, P0, "Bone Devourer");
    let m = zones::enter(&mut g, P0, c, zones::Enter::default()).unwrap();
    assert_eq!(g.perm(m).plus, 3, "one counter per creature that died this turn");
    let (hand0, life0) = (g.player(P0).hand.len(), g.player(P0).life);
    zones::die(&mut g, m, "destroy").unwrap();
    assert_eq!(g.player(P0).hand.len(), hand0 + 3, "draws X");
    assert_eq!(g.player(P0).life, life0 - 3, "loses X");
}

#[test]
fn bone_devourer_with_no_deaths_enters_plain_and_dies_for_nothing() {
    let mut g = pod();
    g.died_count = (None, 0);
    let c = take(&mut g, P0, "Bone Devourer");
    let m = zones::enter(&mut g, P0, c, zones::Enter::default()).unwrap();
    assert_eq!(g.perm(m).plus, 0);
    let hand0 = g.player(P0).hand.len();
    zones::die(&mut g, m, "destroy").unwrap();
    assert_eq!(g.player(P0).hand.len(), hand0);
}

#[test]
fn thalisse_makes_one_spirit_per_token_created_this_turn() {
    let mut g = pod();
    let t = perm(&mut g, P0, "Thalisse, Reverent Medium");
    let spec = zones::Tokens { fly: true, types: vec!["spirit"], ..zones::Tokens::new(2, 1) };
    zones::make_tokens(&mut g, P0, spec).unwrap();
    zones::make_artifact_tokens(&mut g, P0, "Treasure", 1).unwrap();
    assert_eq!(g.player(P0).tok_made.1, 3);
    let before = tokens(&g, P0).len();
    run_hook(&mut g, t, Event::EndStep, &Call::Player { p: P1 }).unwrap();
    assert_eq!(tokens(&g, P0).len(), before + 3, "X = 3 tokens created, whoever's end step it is");
    // an opponent's tokens don't count
    assert_eq!(g.player(P1).tok_made.1, 0);
}

#[test]
fn filter_lands_need_a_matching_land_and_the_temple_needs_five_lands() {
    use sim_core::engine::mana::{land_colors_now, mana_units};
    let mut g = pod();
    let fh = lands(&mut g, P0, "Fetid Heath", 1, false)[0];
    assert_eq!(land_colors_now(&g, P0, fh).count(), 0, "alone it makes colourless only");
    lands(&mut g, P0, "Plains", 1, false);
    let c = land_colors_now(&g, P0, fh);
    assert!(c.has('W') && c.has('B'), "with a Plains to pay the filter cost");
    let mut g = pod();
    let t = lands(&mut g, P0, "Temple of the False God", 1, false)[0];
    lands(&mut g, P0, "Plains", 3, false);
    let amt = |g: &Game| {
        mana_units(g, P0, false).iter().find(|u| u.src == sim_core::engine::mana::Source::Land(t)).unwrap().amt
    };
    assert_eq!(amt(&g), 0, "four lands");
    lands(&mut g, P0, "Plains", 1, false);
    assert_eq!(amt(&g), 2, "five lands");
}
