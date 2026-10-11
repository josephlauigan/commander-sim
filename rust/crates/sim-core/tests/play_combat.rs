//! Practice mode: combat declarations for the human seat (Python's tests/test_play_combat.py). Attackers are any
//! creatures that can legally attack; you pick whom to attack; when attacked, you pick blockers. The engine resolves
//! the damage.

mod play_common;
use play_common::*;
use sim_core::engine::combat::{combat, resolve_combat, split_attack};
use sim_core::human::combat::{SPLIT, attack_candidates, human_attack, human_blocks};
use sim_core::sym::intern;

fn sick(g: &mut Game, m: PermId, v: bool) -> PermId {
    g.perm_mut(m).sick = v;
    m
}

fn idx<T: PartialEq>(v: &[T], x: &T) -> usize {
    v.iter().position(|y| y == x).unwrap()
}

fn life(g: &Game, p: PlayerId) -> i32 {
    g.player(p).life
}

fn tapped(g: &Game, m: PermId) -> bool {
    g.perm(m).tapped
}

fn alive(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.player(p).perms.contains(&m)
}

// ---- Attacking
#[test]
fn who_can_attack() {
    let mut g = table(&["sauron", "veyran"]);
    let arch = perm(&mut g, P0, "Jace's Archivist"); // the AI never attacks with it; you may
    let new = perm(&mut g, P0, "Orcish Bowmasters");
    sick(&mut g, new, true);
    let t = perm(&mut g, P0, "Grave Titan");
    g.perm_mut(t).tapped = true;
    let c = attack_candidates(&g, P0);
    assert!(c.contains(&arch));
    assert!(!c.contains(&new));
    assert!(!c.contains(&t));
}

#[test]
fn attack_the_only_opponent() {
    let mut g = table(&["sauron", "veyran"]);
    let t = perm(&mut g, P0, "Grave Titan");
    let i = idx(&attack_candidates(&g, P0), &t);
    seat(&mut g, P0, vec![picks(&[i])]);
    combat(&mut g, P0).unwrap();
    assert_eq!(life(&g, P1), 40 - 6);
    assert!(tapped(&g, t));
}

#[test]
fn choose_the_defending_player() {
    let mut g = table(&["sauron", "veyran", "seph"]);
    perm(&mut g, P0, "Grave Titan");
    let h = seat(&mut g, P0, vec![picks(&[0]), pick(1)]);
    combat(&mut g, P0).unwrap();
    assert_eq!((life(&g, P1), life(&g, P2)), (40, 34));
    let kinds: Vec<String> = h.asked().iter().map(|r| r.kind.to_string()).collect();
    assert_eq!(kinds, ["attack", "target"]);
}

#[test]
fn split_attackers_between_players() {
    let mut g = table(&["sauron", "veyran", "seph"]);
    let t = perm(&mut g, P0, "Grave Titan");
    let b = perm(&mut g, P0, "Orcish Bowmasters");
    let c = attack_candidates(&g, P0);
    let h = seat(&mut g, P0, vec![picks(&[idx(&c, &t), idx(&c, &b)]), pick(2), pick(0), pick(1)]); // Titan at Veyran, Bowmasters at Sephiroth
    let (lv, lx) = (life(&g, P1), life(&g, P2)); // (Bowmasters' enter damage already dealt)
    combat(&mut g, P0).unwrap();
    assert_eq!((life(&g, P1), life(&g, P2)), (lv - 6, lx - 1));
    assert!(tapped(&g, t) && tapped(&g, b));
    assert!(h.asked()[1].choices.iter().any(|x| x == SPLIT));
}

#[test]
fn split_to_one_player_is_one_attack() {
    let mut g = table(&["sauron", "veyran", "seph"]);
    let t = perm(&mut g, P0, "Grave Titan");
    let b = perm(&mut g, P0, "Orcish Bowmasters");
    let c = attack_candidates(&g, P0);
    seat(&mut g, P0, vec![picks(&[idx(&c, &t), idx(&c, &b)]), pick(2), pick(1), pick(1)]);
    assert_eq!(human_attack(&mut g, P0, 1).unwrap(), Some(vec![(P2, vec![t, b])]));
}

#[test]
fn a_single_attacker_isnt_offered_a_split() {
    let mut g = table(&["sauron", "veyran", "seph"]);
    perm(&mut g, P0, "Grave Titan");
    let h = seat(&mut g, P0, vec![picks(&[0]), pick(0)]);
    human_attack(&mut g, P0, 1).unwrap();
    assert!(!h.asked()[1].choices.iter().any(|x| x == SPLIT));
}

#[test]
fn no_attack() {
    let mut g = table(&["sauron", "veyran"]);
    let t = perm(&mut g, P0, "Grave Titan");
    seat(&mut g, P0, vec![picks(&[])]);
    combat(&mut g, P0).unwrap();
    assert_eq!(life(&g, P1), 40);
    assert!(!tapped(&g, t));
}

// ---- the AI's attackers may go at different players (split_attack)
#[test]
fn an_attacker_avoids_a_bad_block() {
    let mut g = table(&["sauron", "veyran", "seph"]);
    let a = perm(&mut g, P0, "Orcish Bowmasters"); // 1/1
    perm(&mut g, P1, "Grave Titan"); // Veyran's 6/6 would eat it
    sick(&mut g, a, false);
    assert_eq!(split_attack(&g, P0, P1, &[a], &[]), vec![(P2, vec![a])]);
}

#[test]
fn one_blocker_stops_only_one_attacker() {
    let mut g = table(&["sauron", "veyran", "seph"]);
    let a = perm(&mut g, P0, "Orcish Bowmasters");
    let b = perm(&mut g, P0, "Orcish Bowmasters");
    perm(&mut g, P1, "Niv-Mizzet, Parun"); // one blocker that eats a 1/1
    let groups = split_attack(&g, P0, P1, &[a, b], &[]);
    let n = |q: PlayerId| groups.iter().find(|x| x.0 == q).map_or(0, |x| x.1.len());
    assert_eq!((n(P1), n(P2)), (1, 1));
}

#[test]
fn finish_off_a_player_who_cant_block() {
    let mut g = table(&["sauron", "veyran", "seph"]);
    let t = perm(&mut g, P0, "Grave Titan");
    perm(&mut g, P1, "Grave Titan"); // Veyran can trade with it; Sephiroth has nothing
    g.player_mut(P2).life = 5;
    assert_eq!(split_attack(&g, P0, P1, &[t], &[]), vec![(P2, vec![t])]);
}

#[test]
fn nothing_moves_without_a_reason() {
    let mut g = table(&["sauron", "veyran", "seph"]);
    let t = perm(&mut g, P0, "Grave Titan");
    assert_eq!(split_attack(&g, P0, P1, &[t], &[]), vec![(P1, vec![t])]);
}

#[test]
fn the_ai_attacks_two_players_in_one_combat() {
    let mut g = table(&["sauron", "veyran", "seph"]);
    let t = perm(&mut g, P0, "Grave Titan");
    let b = perm(&mut g, P0, "Orcish Bowmasters");
    perm(&mut g, P2, "Grave Titan"); // Sephiroth can eat the Bowmasters
    let lv = life(&g, P1);
    g.forced_attack = Some((P2, intern("filtered"))); // the plan names Sephiroth
    combat(&mut g, P0).unwrap();
    assert!(life(&g, P1) < lv); // the Bowmasters went at Veyran instead
    assert!(tapped(&g, t) && tapped(&g, b));
}

// ---- Blocking
#[test]
fn block() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Grave Titan");
    let a = perm(&mut g, P1, "Guttersnipe");
    let h = seat(&mut g, P0, vec![pass(), pick(0)]);
    g.active = Some(P1);
    resolve_combat(&mut g, P1, &[a], P0, &[]).unwrap();
    assert_eq!(life(&g, P0), 40);
    assert!(!alive(&g, P1, a));
    let kinds: Vec<String> = h.asked().iter().map(|r| r.kind.to_string()).collect();
    assert_eq!(kinds, ["priority", "block"]);
}

#[test]
fn no_block() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Grave Titan");
    let a = perm(&mut g, P1, "Guttersnipe");
    seat(&mut g, P0, vec![pass(), cancel()]);
    g.active = Some(P1);
    resolve_combat(&mut g, P1, &[a], P0, &[]).unwrap();
    assert_eq!(life(&g, P0), 38);
}

#[test]
fn unblockable_attackers_are_not_asked_about() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Grave Titan");
    let a = perm(&mut g, P1, "Guttersnipe");
    let h = seat(&mut g, P0, vec![pass()]);
    g.active = Some(P1);
    resolve_combat(&mut g, P1, &[a], P0, &[a]).unwrap();
    let kinds: Vec<String> = h.asked().iter().map(|r| r.kind.to_string()).collect();
    assert_eq!(kinds, ["priority"]);
    assert_eq!(life(&g, P0), 38);
}

// ---- with a person at the browser, combat requests say where the creatures are, so the table can show them
#[test]
fn attackers_point_at_your_creatures() {
    let mut g = table(&["sauron", "veyran"]);
    let t = perm(&mut g, P0, "Grave Titan");
    let h = seat_live(&mut g, P0, vec![picks(&[0])]);
    let out = human_attack(&mut g, P0, 1).unwrap();
    let i = idx(&g.player(P0).perms, &t);
    assert_eq!(h.asked()[0].data["refs"], json!([{"seat": "sauron", "perm": i}]));
    assert_eq!(out, Some(vec![(P1, vec![t])]));
}

#[test]
fn blocks_show_the_attackers() {
    let mut g = table(&["veyran", "sauron"]);
    let a = perm(&mut g, P0, "Guttersnipe");
    let b = perm(&mut g, P1, "Orcish Bowmasters");
    let h = seat_live(&mut g, P1, vec![cancel()]);
    human_blocks(&mut g, P0, &[a], P1, &[]).unwrap();
    let d = h.asked()[0].data.clone();
    let want = json!({"seat": "veyran", "perm": idx(&g.player(P0).perms, &a)});
    assert_eq!(d["attacker"], want);
    assert_eq!(d["attackers"], json!([want]));
    let mine = json!({"seat": "sauron", "perm": idx(&g.player(P1).perms, &b)});
    assert!(d["refs"].as_array().unwrap().contains(&mine)); // (and the Orc Army it made)
}
