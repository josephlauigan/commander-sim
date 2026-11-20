//! P5a-2b C3: event copies are additive and belong to each listener's controller.
use sim_core::cardcode;
use sim_core::control::Answer;
use sim_core::engine::{cast, combat, hooks, turn, zones};
use sim_core::hooks::{Action, Call, Event};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game, Val};
use sim_core::testkit::*;
use std::sync::Arc;

const A: PlayerId = PlayerId(0);
const B: PlayerId = PlayerId(1);
const C: PlayerId = PlayerId(2);
const PAN: &str = "Panharmonicon";
const VOC: &str = "Starfield Vocalist";
const E: &str = "Elesh Norn, Mother of Machines";
const GW: &str = "Ancient Greenwarden";

fn tbl() -> Game {
    table(&["miirym-temur-dragons", "seph", "sauron", "cloud"])
}
fn play_land(g: &mut Game, name: &str) -> PermId {
    let c = take(g, A, name);
    let l = PermId(g.perms.len() as u32);
    turn::play_land_card(g, A, c, "plays").unwrap();
    l
}
fn life_sum(g: &Game) -> i32 {
    g.opps(A).map(|q| g.player(q).life).sum()
}
fn human(g: &mut Game) -> Arc<sim_core::human::script::ScriptHuman> {
    let h = Arc::new(sim_core::human::script::ScriptHuman::from_fn(|r| {
        if r.kind == "choose" || r.kind == "target" {
            Answer::Pick(0)
        } else {
            Answer::Action(serde_json::json!({"do":"pass"}))
        }
    }));
    h.set_autopass("all");
    g.humans.add(A, h.clone());
    h
}

// 20: source and event copies add, never multiply each other.
#[test]
fn terror_token_panharmonicon_and_throne() {
    for throne in [false, true] {
        let mut g = tbl();
        perm(&mut g, A, PAN);
        if throne {
            let t = perm(&mut g, A, "Roaming Throne");
            assert_eq!(g.perm(t).data.get(DataKey::Ctype), Some(&Val::Str("dragon")));
        }
        perm(&mut g, A, "Terror of the Peaks");
        let before = life_sum(&g);
        token(&mut g, A, 5);
        assert_eq!(before - life_sum(&g), if throne { 15 } else { 10 });
    }
}

// 21, 22, 23: inline watchers, both creature entry routes.
#[test]
fn uprising_draws_twice_for_token_and_nontoken() {
    for tok in [false, true] {
        let mut g = tbl();
        perm(&mut g, A, PAN);
        perm(&mut g, A, "Garruk's Uprising");
        let n = g.player(A).hand.len();
        if tok {
            token(&mut g, A, 4);
        } else {
            perm(&mut g, A, "Serra Angel");
        }
        assert_eq!(g.player(A).hand.len() - n, 2);
    }
}
#[test]
fn warleaders_call_event_copies_add() {
    for elesh in [false, true] {
        for tok in [false, true] {
            let mut g = tbl();
            perm(&mut g, A, PAN);
            if elesh {
                perm(&mut g, A, E);
            }
            perm(&mut g, A, "Warleader's Call");
            let n = g.player(B).life;
            if tok {
                token(&mut g, A, 1);
            } else {
                perm(&mut g, A, "Llanowar Elves");
            }
            assert_eq!(n - g.player(B).life, if elesh { 3 } else { 2 });
        }
    }
}
fn shards(copier: &str, tok: bool) {
    let mut g = tbl();
    perm(&mut g, A, copier);
    perm(&mut g, A, "Aura Shards");
    let a = perm(&mut g, B, "Myr Battlesphere");
    let b = perm(&mut g, B, "Steel Hellkite");
    if tok {
        token(&mut g, A, 1);
    } else {
        perm(&mut g, A, "Llanowar Elves");
    }
    assert!(!g.perm(a).on_bf && !g.perm(b).on_bf);
}
#[test]
fn shards_panharmonicon_destroys_two() {
    for tok in [false, true] {
        shards(PAN, tok);
    }
}
#[test]
fn shards_own_elesh_control_destroys_two() {
    shards(E, false);
}

// 24: TokensEnter hooks see creature characteristics, not the artifact-token None convention.
#[test]
fn token_hooks_get_event_copies() {
    for (listener, copier, n, damage) in [("Impact Tremors", PAN, 2, 4), ("Purphoros, God of the Forge", VOC, 1, 4)] {
        let mut g = tbl();
        perm(&mut g, A, copier);
        perm(&mut g, A, listener);
        let before = g.player(B).life;
        zones::make_tokens(&mut g, A, zones::Tokens::new(n, 1)).unwrap();
        assert_eq!(before - g.player(B).life, damage, "{listener}");
    }
}
#[test]
fn valkyrie_own_elesh_angel_token_gains_eight() {
    let mut g = tbl();
    perm(&mut g, A, E);
    perm(&mut g, A, "Righteous Valkyrie");
    let before = g.player(A).life;
    zones::make_tokens(&mut g, A, zones::Tokens { types: vec!["angel"], ..zones::Tokens::new(1, 4) }).unwrap();
    assert_eq!(g.player(A).life - before, 8);
}

// 25: run-now T hooks; the first Oracle instance is countered.
#[test]
fn oracle_own_elesh_second_window_wins() {
    let mut g = tbl();
    perm(&mut g, A, E);
    g.skip_etb = true;
    let o = perm(&mut g, A, "Thassa's Oracle");
    g.skip_etb = false;
    hand(&mut g, B, &["Tishana's Tidebinder"]);
    lands(&mut g, B, "Island", 3, false);
    g.player_mut(A).library.clear();
    hooks::fire_trigger(&mut g, Event::Etb, Call::Etb { p: A, m: o }).unwrap();
    assert!(!g.player(B).hand.contains(&card(&g, "Tishana's Tidebinder")));
    assert!(g.over && g.winner == Some(A));
}
#[test]
fn ertai_vocalist_destroys_two_creatures() {
    let mut g = tbl();
    perm(&mut g, A, VOC);
    let a = perm(&mut g, B, "Grave Titan");
    let b = perm(&mut g, C, "Serra Angel");
    perm(&mut g, A, "Ertai Resurrected");
    assert!(!g.perm(a).on_bf && !g.perm(b).on_bf);
}

// 26: mixed hooks copy only their trigger portion.
fn gift(copier: &str, expected: usize) {
    let mut g = tbl();
    perm(&mut g, A, copier);
    let host = token(&mut g, A, 1);
    let c = hand(&mut g, A, &["Angelic Gift"])[0];
    let before = g.player(A).hand.len();
    cast::cast_card(&mut g, A, c, "hand", Ctx { target: Some(host), ..Default::default() }).unwrap();
    assert_eq!(g.player(A).hand.len(), before - 1 + expected);
}
#[test]
fn angelic_gift_vocalist_draws_two() {
    gift(VOC, 2);
}
#[test]
fn angelic_gift_panharmonicon_control_draws_one() {
    gift(PAN, 1);
}
#[test]
fn socialite_panharmonicon_puts_two_counters_on_other_vampires() {
    let mut g = tbl();
    perm(&mut g, A, PAN);
    let v = perm(&mut g, A, "Wispdrinker Vampire");
    sim_core::engine::life::lose_life(&mut g, B, 1, Some(A), "drain", None).unwrap();
    let before = g.perm(v).plus;
    perm(&mut g, A, "Vampire Socialite");
    assert_eq!(g.perm(v).plus - before, 2);
}
#[test]
fn prison_sentence_own_elesh_scries_two_twice() {
    let mut g = tbl();
    perm(&mut g, A, E);
    let target = perm(&mut g, B, "Serra Angel");
    let h = human(&mut g);
    let c = hand(&mut g, A, &["Prison Sentence"])[0];
    cast::cast_card(&mut g, A, c, "hand", Ctx { target: Some(target), ..Default::default() }).unwrap();
    assert_eq!(h.asked().iter().filter(|r| r.prompt.starts_with("Scry:")).count(), 4);
}
#[test]
fn wan_shi_tong_own_elesh_x_four() {
    let mut g = tbl();
    perm(&mut g, A, E);
    lands(&mut g, A, "Island", 6, false);
    let c = hand(&mut g, A, &["Wan Shi Tong, Librarian"])[0];
    let before = g.player(A).hand.len();
    assert!(sim_core::impls::t6_legends2::wan_go(&mut g, A, c.0 as i64).unwrap());
    let w = g.player(A).perms.iter().copied().find(|&m| g.perm(m).cd == Some(c)).unwrap();
    assert_eq!(g.perm(w).plus, 8);
    assert_eq!(g.player(A).hand.len(), before - 1 + 4);
}

// 27: land hooks are repeated by land_etb, never again in their bodies.
#[test]
fn bog_own_elesh_exiles_two_graveyards() {
    let mut g = tbl();
    perm(&mut g, A, E);
    for q in [B, C] {
        let c = take(&mut g, q, "Grave Titan");
        g.player_mut(q).gy.push(c);
    }
    play_land(&mut g, "Bojuka Bog");
    assert!(g.player(B).gy.is_empty() && g.player(C).gy.is_empty());
}
#[test]
fn lotus_field_greenwarden_sacrifices_four() {
    let mut g = tbl();
    perm(&mut g, A, GW);
    lands(&mut g, A, "Forest", 6, false);
    play_land(&mut g, "Lotus Field");
    assert_eq!(g.player(A).lands.len(), 3);
    assert_eq!(g.player(A).gy.len(), 4);
}
#[test]
fn aether_hub_vocalist_gives_two_energy() {
    let mut g = tbl();
    perm(&mut g, A, VOC);
    play_land(&mut g, "Aether Hub");
    assert_eq!(g.player(A).energy, 2);
}
#[test]
fn hideaway_keeps_both_cards_and_can_play_either() {
    for name in ["Mosswort Bridge", "Windbrisk Heights"] {
        let mut g = tbl();
        perm(&mut g, A, GW);
        token(&mut g, A, 10);
        lands(&mut g, A, "Forest", 2, false);
        lands(&mut g, A, "Plains", 2, false);
        let cards = [card(&g, "Grave Titan"), card(&g, "Serra Angel")];
        let plains = card(&g, "Plains");
        g.player_mut(A).library = vec![cards[1], plains, plains, plains, cards[0], plains, plains, plains];
        let l = play_land(&mut g, name);
        let Some(Val::List(hidden)) = g.perm(l).data.get(DataKey::Hidden) else { panic!("two hidden cards: {name}") };
        assert_eq!(hidden.len(), 2);
        assert!(cards.iter().all(|c| hidden.contains(&Val::Card(*c))));
        g.perm_mut(l).tapped = false;
        g.player_mut(A).atk_count = (Some(g.turn_stamp()), 3);
        let opts = cardcode::land_options(&mut g, A, Some(true)).unwrap();
        let opts: Vec<_> = opts.into_iter().filter(|o| o.label.starts_with(name)).collect();
        assert_eq!(opts.len(), 2);
        for c in cards {
            let mut copy = g.clone();
            let o = opts.iter().find(|o| o.label.contains(&*copy.db.get(c).name)).unwrap();
            let Some(Action::Plan { f, arg }) = o.act else { panic!("hideaway action") };
            assert!(f(&mut copy, A, arg).unwrap());
            assert!(copy.player(A).perms.iter().any(|&m| copy.perm(m).cd == Some(c)));
            let remaining = copy.perm(l).data.get(DataKey::Hidden);
            assert!(!matches!(remaining, Some(Val::List(xs)) if xs.contains(&Val::Card(c))));
            assert!(remaining.is_some_and(|v| v.truthy()), "the other card remains linked");
        }
    }
}

// 28: self-return and self-sacrifice cannot remove the entering land twice.
#[test]
fn bouncelands_and_chasm_repeat_safely() {
    for (name, copier, others, remaining, hand_count) in [
        ("Azorius Chancery", E, 3, 2, 2),
        ("Azorius Chancery", E, 1, 0, 2),
        ("Azorius Chancery", E, 0, 0, 1),
        ("Simic Growth Chamber", GW, 3, 2, 2),
        ("Glacial Chasm", GW, 3, 2, 0),
        ("Glacial Chasm", GW, 0, 0, 0),
    ] {
        let mut g = tbl();
        perm(&mut g, A, copier);
        lands(&mut g, A, "Forest", others, false);
        play_land(&mut g, name);
        assert_eq!(g.player(A).lands.len(), remaining, "{name}, others={others}");
        assert_eq!(g.player(A).hand.len(), hand_count, "{name}, others={others}");
        if name == "Glacial Chasm" {
            assert_eq!(g.player(A).gy.len(), if others == 0 { 1 } else { 2 });
        }
    }
}

// 29: compiled lands use the max of each copier's landfall/etb contribution.
#[test]
fn temple_greenwarden_scries_twice() {
    let mut g = tbl();
    perm(&mut g, A, GW);
    let h = human(&mut g);
    play_land(&mut g, "Temple of Mystery");
    assert_eq!(h.asked().iter().filter(|r| r.prompt.starts_with("Scry:")).count(), 2);
}
fn falls(copier: &str, expected: i32) {
    let mut g = tbl();
    perm(&mut g, A, copier);
    let before = g.player(A).life;
    play_land(&mut g, "Thornwood Falls");
    assert_eq!(g.player(A).life - before, expected);
}
#[test]
fn falls_greenwarden_gains_two() {
    falls(GW, 2);
}
#[test]
fn falls_panharmonicon_control_gains_one() {
    falls(PAN, 1);
}

// 30: fresh targets and searches for every instance.
#[test]
fn sanctuary_own_elesh_returns_two_instants() {
    let mut g = tbl();
    perm(&mut g, A, E);
    lands(&mut g, A, "Island", 3, false);
    let cards = [card(&g, "Opt"), card(&g, "Counterspell")];
    g.player_mut(A).gy.extend(cards);
    play_land(&mut g, "Mystic Sanctuary");
    assert!(g.player(A).gy.is_empty());
    assert!(cards.iter().all(|c| g.player(A).library.iter().rev().take(2).any(|x| x == c)));
}
#[test]
fn talon_gates_vocalist_phases_two_creatures() {
    let mut g = tbl();
    perm(&mut g, A, VOC);
    let a = perm(&mut g, B, "Grave Titan");
    let b = perm(&mut g, C, "Serra Angel");
    play_land(&mut g, "Talon Gates of Madara");
    assert!(g.perm(a).phased && g.perm(b).phased);
}
#[test]
fn ikoria_own_elesh_searches_twice() {
    let mut g = tbl();
    perm(&mut g, A, E);
    g.player_mut(A).library = vec![card(&g, "Serra Angel"), card(&g, "Grave Titan")];
    let c = hand(&mut g, A, &["Invasion of Ikoria // Zilortha, Apex of Ikoria"])[0];
    let before = g.player(A).perms.len();
    cast::cast_card(&mut g, A, c, "hand", Ctx { x: 6, ..Default::default() }).unwrap();
    // Battle + two creatures + Grave Titan's four Zombies (its ETB also doubles).
    assert_eq!(g.player(A).perms.len() - before, 7);
}

// 31: controls must already be green at the base.
#[test]
fn nissa_greenwarden_control_gets_two_loyalty() {
    let mut g = tbl();
    perm(&mut g, A, GW);
    let n = perm(&mut g, A, "Nissa of Shadowed Boughs");
    let before = g.perm(n).loyalty.unwrap();
    play_land(&mut g, "Forest");
    assert_eq!(g.perm(n).loyalty.unwrap() - before, 2);
}
#[test]
fn yoshimaru_greenwarden_vocalist_control_gets_three() {
    let mut g = tbl();
    perm(&mut g, A, GW);
    perm(&mut g, A, VOC);
    let y = perm(&mut g, A, "Yoshimaru, Ever Faithful");
    play_land(&mut g, "Minas Tirith");
    assert_eq!(g.perm(y).plus, 3);
}
#[test]
fn treasure_does_not_trigger_tremors_control() {
    let mut g = tbl();
    perm(&mut g, A, PAN);
    perm(&mut g, A, "Impact Tremors");
    let before = life_sum(&g);
    zones::add_treasure(&mut g, A, 1).unwrap();
    assert_eq!(life_sum(&g), before);
    hooks::fire_trigger(&mut g, Event::TokensEnter, Call::Tokens { p: A, toks: vec![] }).unwrap();
    assert_eq!(life_sum(&g), before);
}
#[test]
fn setup_is_never_copied_control() {
    let mut g = tbl();
    for copier in [PAN, VOC, E] {
        perm(&mut g, A, copier);
    }
    let c = perm(&mut g, A, "Freestrider Commando");
    assert_eq!(g.perm(c).plus, 2);
    let m = perm(&mut g, A, "Moss Diamond");
    assert!(g.perm(m).tapped);
}

// T2 39-40: Scute itself only listens to its controller's land. A probe accepting either player's land isolates
// dispatch counts; changing real Scute's Oracle condition would make the requested opponent-Scute example illegal.
fn landfall_counts(copier: Option<PlayerId>, expected: [i32; 2]) {
    let mut g = tbl();
    if let Some(p) = copier {
        perm(&mut g, p, GW);
    }
    for p in [A, B] {
        perm(&mut g, p, "Scute Swarm");
    }
    let mut registry = sim_core::hooks::Registry::new(&g.db);
    for d in g.db.iter() {
        let c = card(&g, &d.name);
        if let Some(i) = g.registry.get(c) {
            registry.on(&g.db, &d.name, *i).unwrap();
        }
    }
    registry.card(&g.db, "Scute Swarm").unwrap().landfall = Some(|g, s, _| {
        let p = g.perm(s).owner;
        g.player_mut(p).life += 1;
        Ok(())
    });
    g.registry = Arc::new(registry);
    let before = [g.player(A).life, g.player(B).life];
    play_land(&mut g, "Forest");
    assert_eq!([g.player(A).life - before[0], g.player(B).life - before[1]], expected);
    assert_eq!(g.resolving, 0);
}
#[test]
fn land_players_greenwarden_only_repeats_its_own_listener() {
    landfall_counts(Some(A), [2, 1]);
}
#[test]
fn opponents_greenwarden_repeats_the_opponents_listener() {
    landfall_counts(Some(B), [1, 2]);
}
#[test]
fn real_opponent_scute_does_not_hear_another_players_land_control() {
    let mut g = tbl();
    perm(&mut g, A, GW);
    perm(&mut g, B, "Scute Swarm");
    play_land(&mut g, "Forest");
    assert_eq!(g.player(B).perms.len(), 1);
}

// 41: Aurelia is a real opponent listener; check both directions and both controllers having copiers.
#[test]
fn attack_copies_belong_to_each_listener_controller() {
    for owner in [A, B] {
        let mut g = tbl();
        perm(&mut g, owner, "Isshin, Two Heavens as One");
        perm(&mut g, A, "Aurelia, the Law Above");
        perm(&mut g, B, "Aurelia, the Law Above");
        let atk = [token(&mut g, A, 1), token(&mut g, A, 1), token(&mut g, A, 1)];
        let before = [g.player(A).hand.len(), g.player(B).hand.len()];
        combat::attack_triggers(&mut g, A, &atk, B).unwrap();
        assert_eq!(g.player(A).hand.len() - before[0], if owner == A { 2 } else { 1 });
        assert_eq!(g.player(B).hand.len() - before[1], if owner == B { 2 } else { 1 });
    }
}
#[test]
fn no_copier_landfall_control_runs_once_per_controller() {
    landfall_counts(None, [1, 1]);
}

#[test]
fn wispdrinker_event_and_source_extras_add_on_both_paths() {
    for tok in [false, true] {
        let mut g = tbl();
        perm(&mut g, A, PAN);
        perm(&mut g, A, "Delney, Streetwise Lookout");
        perm(&mut g, A, "Wispdrinker Vampire");
        let before = [g.player(A).life, g.player(B).life];
        if tok {
            token(&mut g, A, 1);
        } else {
            perm(&mut g, A, "Llanowar Elves");
        }
        assert_eq!([g.player(A).life - before[0], before[1] - g.player(B).life], [3, 3]);
    }
}

#[test]
fn haliya_queue_adapter_event_and_source_copies_are_counted_once() {
    let mut g = tbl();
    perm(&mut g, A, PAN);
    perm(&mut g, A, "Delney, Streetwise Lookout");
    let h = perm(&mut g, A, "Haliya, Guided by Light");
    g.perm_mut(h).plus = -1;
    let before = g.player(A).life;
    token(&mut g, A, 1);
    assert_eq!(g.player(A).life - before, 3);
}

#[test]
fn bog_does_not_retarget_an_empty_graveyard_with_a_recursion_engine() {
    let mut g = tbl();
    perm(&mut g, A, E);
    perm(&mut g, B, "Meren of Clan Nel Toth");
    let titan = take(&mut g, B, "Grave Titan");
    let bloodghast = take(&mut g, C, "Bloodghast");
    g.player_mut(B).gy.push(titan);
    g.player_mut(C).gy.push(bloodghast);
    play_land(&mut g, "Bojuka Bog");
    assert!(g.player(B).gy.is_empty() && g.player(C).gy.is_empty());
}

#[test]
fn isshin_repeats_myriad_granted_by_blade_of_selves() {
    let mut g = tbl();
    perm(&mut g, A, "Isshin, Two Heavens as One");
    let a = token(&mut g, A, 2);
    let blade = perm(&mut g, A, "Blade of Selves");
    g.perm_mut(blade).attached = Some(a);
    let new = combat::attack_triggers(&mut g, A, &[a], B).unwrap();
    assert_eq!(new.len(), 4, "two other opponents, twice");
}
