//! Practice mode: the Ur-Dragon deck's three loops as table shortcuts for a person (`human/urdragon.rs`, `loop_abilities`):
//! offered only while the catalogue's `ready` holds, cancellable, and the result is what the loop makes (hasty Astral
//! Dragons, 3/3 flying Dragon tokens, hasty token Garths). Seat 0 is the person, seat 1 holds the answers.

mod play_common;
use play_common::*;
use sim_core::engine::zones::{self, Enter};
use sim_core::state::Step;
use std::sync::{Arc, Mutex};

const MIRROR: &str = "Astral Dragon + Cursed Mirror";
const MIIRYM: &str = "Miirym + Astral Dragon + Doubling Season";
const GARTH: &str = "Garth One-Eye + Twinflame + Spark Double + Doubling Season";

type Rule = (&'static str, Box<dyn Fn(&Request) -> Answer + Send>);

/// a person who answers by what is asked: each rule is used once, the first whose text is in the prompt; anything else
/// is passed or declined. Everything asked is recorded as "kind: prompt".
fn scripted(rules: Vec<Rule>, log: Arc<Mutex<Vec<String>>>) -> ScriptHuman {
    let mut rules: Vec<Option<Rule>> = rules.into_iter().map(Some).collect();
    ScriptHuman::from_fn(move |r| {
        log.lock().unwrap().push(format!("{}: {}", r.kind, r.prompt));
        for slot in rules.iter_mut() {
            if slot.as_ref().is_some_and(|(k, _)| r.prompt.contains(k)) {
                let (_, f) = slot.take().unwrap();
                return f(r);
            }
        }
        if r.kind == "priority" { Answer::Action(json!({"do": "pass"})) } else { Answer::Decline }
    })
}

fn rule(key: &'static str, f: impl Fn(&Request) -> Answer + Send + 'static) -> Rule {
    (key, Box::new(f))
}

fn text(word: &'static str) -> impl Fn(&Request) -> Answer + Send + 'static {
    move |r| {
        Answer::Pick(
            r.choices.iter().position(|c| c.contains(word)).unwrap_or_else(|| panic!("no {word:?} in {:?}", r.choices)),
        )
    }
}

fn seat_script(g: &mut Game, rules: Vec<Rule>) -> Arc<Mutex<Vec<String>>> {
    let log = Arc::new(Mutex::new(vec![]));
    g.humans.add(P0, Arc::new(scripted(rules, log.clone())));
    log
}

fn table_for(seed: u64) -> Game {
    let mut g = table_with(&["urdragon", "seph"], seed, 40);
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    // the Ur-Dragon out of the command zone: its eminence would take {1} off Astral Dragon
    let c = card(&g, "The Ur-Dragon");
    g.player_mut(P0).set_cmd_zone(c, false);
    g
}

/// the labels of the loop abilities p's permanent m offers
fn offered(g: &Game, m: PermId) -> Vec<String> {
    assert!(sim_core::human::is_human(g, P0), "seat the person first");
    sim_core::human::abilities::permanent_abilities(g, P0, m)
        .into_iter()
        .map(|a| a.label)
        .filter(|l| l.starts_with("start the loop: ") && !l.contains("no payoff"))
        .filter(|l| [MIRROR, MIIRYM, GARTH].iter().any(|n| l.contains(n)))
        .collect()
}

fn offers(g: &Game, m: PermId, part: &str) -> bool {
    offered(g, m).iter().any(|l| l.contains(part))
}

fn index_of(g: &Game, m: PermId) -> usize {
    g.player(P0).perms.iter().position(|&x| x == m).unwrap()
}

/// the person uses permanent m's "start the loop" ability and picks the answer to "how far?" containing `how`
fn start(g: &mut Game, m: PermId, how: &'static str) -> Arc<Mutex<Vec<String>>> {
    let log = seat_script(
        g,
        vec![rule("which ability", text("start the loop")), rule("the opponents may answer", text(how))],
    );
    let i = index_of(g, m);
    sim_core::human::human::apply(g, P0, &json!({"do": "use", "perm": i})).unwrap();
    log
}

fn count(g: &Game, name: &str, f: impl Fn(&Game, PermId) -> bool) -> usize {
    g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf && &*g.perm(m).name == name && f(g, m)).count()
}

fn asked(log: &Arc<Mutex<Vec<String>>>, what: &str) -> bool {
    log.lock().unwrap().iter().any(|l| l.contains(what))
}

// ============================================================ Astral Dragon + Cursed Mirror
/// the Mirror out as the artifact, Astral Dragon in hand, {6}{U}{U} in the pool (seated afterwards: the setup is the
/// AI's)
fn mirror_table(seed: u64) -> (Game, PermId) {
    let mut g = table_for(seed);
    let mirror = perm(&mut g, P0, "Cursed Mirror");
    hand(&mut g, P0, &["Astral Dragon"]);
    pool_up(&mut g, &[('U', 2), ('R', 6)]);
    (g, mirror)
}

#[test]
fn the_mirror_loop_is_offered_when_live() {
    let (g, mirror) = mirror_table(1);
    assert!(offers(&g, mirror, MIRROR), "{:?}", offered(&g, mirror));
}

#[test]
fn the_mirror_loop_is_not_offered_with_a_piece_missing() {
    // no Astral Dragon in hand
    let (mut g, mirror) = mirror_table(1);
    g.player_mut(P0).hand.clear();
    assert!(!offers(&g, mirror, MIRROR));

    // too little mana
    let (mut g, mirror) = mirror_table(1);
    g.player_mut(P0).pool.empty();
    pool_up(&mut g, &[('U', 2), ('R', 5)]);
    assert!(!offers(&g, mirror, MIRROR), "seven mana of eight");

    // no Cursed Mirror on the battlefield (in hand is not enough): there is no piece to carry the ability, and the
    // Dragon alone offers nothing
    let mut g = table_for(1);
    hand(&mut g, P0, &["Astral Dragon", "Cursed Mirror"]);
    pool_up(&mut g, &[('U', 2), ('R', 6)]);
    let dragon = perm(&mut g, P0, "Astral Dragon");
    assert!(!offers(&g, dragon, MIRROR));

    // no combat still to come
    let (mut g, mirror) = mirror_table(1);
    g.step = Step::Main2;
    assert!(!offers(&g, mirror, MIRROR));
}

#[test]
fn the_mirror_loop_makes_enough_hasty_dragons_to_kill_the_table() {
    let (mut g, mirror) = mirror_table(1);
    let log = start(&mut g, mirror, "enough hasty Dragons");
    assert!(asked(&log, "Astral Dragon + Cursed Mirror: cast Astral Dragon"), "{:?}", log.lock().unwrap());
    assert_eq!(g.winner, None, "the attack is the person's: the loop only makes the attackers");
    assert!(has_perm(&g, P0, "Astral Dragon") && has_perm(&g, P0, "Cursed Mirror"));
    let hasty = count(&g, "Astral Dragon", |g, m| g.perm(m).token && !g.perm(m).sick && g.perm(m).fly);
    assert!(hasty >= 30, "enough to kill two opponents at 40 life through blockers ({hasty})");
    assert!(g.player(P0).hand.is_empty(), "Astral Dragon was cast");
    // no prompt per token: the Project Image choice appears once at most (the cast Dragon is the loop's own)
    assert!(!asked(&log, "create two 3/3 flying Dragon copies"), "{:?}", log.lock().unwrap());
}

#[test]
fn the_mirror_loop_goes_round_as_many_times_as_asked() {
    for (how, tokens) in [("once around", 2), ("twice around", 6), ("three times", 14)] {
        let (mut g, mirror) = mirror_table(1);
        start(&mut g, mirror, how);
        let n = count(&g, "Astral Dragon", |g, m| g.perm(m).token);
        assert_eq!(n, tokens, "{how}");
    }
}

#[test]
fn the_mirror_tokens_are_mirrors_again_next_turn() {
    let (mut g, mirror) = mirror_table(1);
    start(&mut g, mirror, "once around");
    let t = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token && g.perm(m).on_bf).unwrap();
    assert_eq!(g.perm(t).exc.revert_to, Some(card(&g, "Cursed Mirror")));
    sim_core::impls::t0_stella::mirror_revert(&mut g);
    assert_eq!(&*g.perm(t).name, "Cursed Mirror", "the copy wore off");
}

#[test]
fn cancelling_the_mirror_loop_changes_nothing() {
    let (mut g, mirror) = mirror_table(1);
    seat_script(
        &mut g,
        vec![rule("which ability", text("start the loop")), rule("the opponents may answer", |_| Answer::Decline)],
    );
    let i = index_of(&g, mirror);
    sim_core::human::human::apply(&mut g, P0, &json!({"do": "use", "perm": i})).unwrap();
    assert_eq!(g.player(P0).hand.len(), 1, "Astral Dragon is still in hand");
    assert!(!has_perm(&g, P0, "Astral Dragon"));
    assert_eq!(pool_total(&g, P0), 8, "no mana spent");
}

#[test]
fn a_counterspell_on_astral_dragon_stops_the_mirror_loop() {
    let mut countered = 0;
    for seed in 0..16u64 {
        let (mut g, mirror) = mirror_table(seed);
        lands(&mut g, P1, "Island", 2, false);
        hand(&mut g, P1, &["Counterspell"]);
        start(&mut g, mirror, "enough hasty Dragons");
        if !has_perm(&g, P0, "Astral Dragon") {
            countered += 1;
            assert!(has_perm(&g, P0, "Cursed Mirror"), "the Mirror stays");
            assert_eq!(count(&g, "Astral Dragon", |g, m| g.perm(m).token), 0);
        }
    }
    assert!(countered >= 1, "the AI counters a combo piece sometimes");
}

// ============================================================ Miirym + Astral Dragon + Doubling Season
fn miirym_table(seed: u64) -> (Game, PermId) {
    let mut g = table_for(seed);
    perm(&mut g, P0, "Doubling Season");
    let miirym = perm(&mut g, P0, "Miirym, Sentinel Wyrm");
    hand(&mut g, P0, &["Astral Dragon"]);
    pool_up(&mut g, &[('U', 2), ('R', 6)]);
    (g, miirym)
}

#[test]
fn the_miirym_loop_is_offered_when_live_and_not_with_a_piece_missing() {
    let (g, miirym) = miirym_table(1);
    assert!(offers(&g, miirym, MIIRYM), "{:?}", offered(&g, miirym));
    let season = perms_named(&g, P0, "Doubling Season")[0];
    assert!(offers(&g, season, MIIRYM), "on every piece");

    let (mut g, miirym) = miirym_table(1);
    g.player_mut(P0).hand.clear();
    assert!(!offers(&g, miirym, MIIRYM), "no Astral Dragon");

    let (mut g, miirym) = miirym_table(1);
    g.player_mut(P0).pool.empty();
    pool_up(&mut g, &[('U', 2), ('R', 5)]);
    assert!(!offers(&g, miirym, MIIRYM), "seven mana of eight");

    // no Doubling Season
    let mut g = table_for(1);
    let miirym = perm(&mut g, P0, "Miirym, Sentinel Wyrm");
    hand(&mut g, P0, &["Astral Dragon"]);
    pool_up(&mut g, &[('U', 2), ('R', 6)]);
    assert!(!offers(&g, miirym, MIIRYM));

    // no Miirym
    let mut g = table_for(1);
    let season = perm(&mut g, P0, "Doubling Season");
    hand(&mut g, P0, &["Astral Dragon"]);
    pool_up(&mut g, &[('U', 2), ('R', 6)]);
    assert!(!offers(&g, season, MIIRYM));
}

#[test]
fn the_miirym_loop_fills_the_board_with_sick_flying_dragons() {
    let (mut g, miirym) = miirym_table(1);
    let log = start(&mut g, miirym, "fill the battlefield");
    assert_eq!(g.winner, None, "the tokens are summoning sick: the next combat is the kill");
    let n = count(&g, "Token", |g, m| g.perm(m).token);
    assert!(n >= 100, "a board that grows without bound ({n} tokens)");
    let t = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token && g.perm(m).on_bf).unwrap();
    assert!(g.perm(t).fly && g.perm(t).sick);
    assert!(has_perm(&g, P0, "Astral Dragon"));
    assert!(!asked(&log, "create two 3/3 flying Dragon copies"), "Project Image is the loop's own step");
}

#[test]
fn the_miirym_loop_can_stop_at_a_number() {
    let (mut g, miirym) = miirym_table(1);
    start(&mut g, miirym, "about ten");
    let n = g.player(P0).perms.iter().filter(|&&m| g.perm(m).token && g.perm(m).on_bf && g.perm(m).fly).count();
    assert!((10..40).contains(&n), "about ten Dragons plus Miirym's copy ({n})");
}

#[test]
fn a_counterspell_on_astral_dragon_stops_the_miirym_loop() {
    let mut countered = 0;
    for seed in 0..16u64 {
        let (mut g, miirym) = miirym_table(seed);
        lands(&mut g, P1, "Island", 2, false);
        hand(&mut g, P1, &["Counterspell"]);
        start(&mut g, miirym, "fill the battlefield");
        if !has_perm(&g, P0, "Astral Dragon") {
            countered += 1;
            assert_eq!(g.player(P0).perms.iter().filter(|&&m| g.perm(m).token).count(), 0, "no flood");
        }
    }
    assert!(countered >= 1, "the AI counters a combo piece sometimes");
}

// ============================================================ Garth + Twinflame + Spark Double + Doubling Season
/// Garth, Spark Double (a copy of Garth) and Doubling Season out; Twinflame in hand; seven mana in the pool
fn garth_table(seed: u64) -> (Game, PermId) {
    let mut g = table_for(seed);
    let garth = perm(&mut g, P0, "Garth One-Eye");
    let sd = take(&mut g, P0, "Spark Double");
    zones::enter(&mut g, P0, sd, Enter { plus: 1, ..Enter::default() }).unwrap();
    perm(&mut g, P0, "Doubling Season");
    hand(&mut g, P0, &["Twinflame"]);
    pool_up(&mut g, &[('R', 6), ('G', 1)]);
    (g, garth)
}

#[test]
fn the_garth_loop_is_offered_when_live() {
    let (g, garth) = garth_table(1);
    assert!(offers(&g, garth, GARTH), "{:?}", offered(&g, garth));
    let season = perms_named(&g, P0, "Doubling Season")[0];
    assert!(offers(&g, season, GARTH));
}

#[test]
fn the_garth_loop_is_not_offered_with_a_piece_missing() {
    let (mut g, garth) = garth_table(1);
    g.player_mut(P0).hand.clear();
    assert!(!offers(&g, garth, GARTH), "no Twinflame");

    // six mana: the second Twinflame's strive can't be paid
    let (mut g, garth) = garth_table(1);
    g.player_mut(P0).pool.empty();
    pool_up(&mut g, &[('R', 5), ('G', 1)]);
    assert!(!offers(&g, garth, GARTH), "six mana of seven");

    // no Spark Double
    let mut g = table_for(1);
    let garth = perm(&mut g, P0, "Garth One-Eye");
    perm(&mut g, P0, "Doubling Season");
    hand(&mut g, P0, &["Twinflame"]);
    pool_up(&mut g, &[('R', 6), ('G', 1)]);
    assert!(!offers(&g, garth, GARTH));

    // no Doubling Season
    let mut g = table_for(1);
    let garth = perm(&mut g, P0, "Garth One-Eye");
    let sd = take(&mut g, P0, "Spark Double");
    zones::enter(&mut g, P0, sd, Enter { plus: 1, ..Enter::default() }).unwrap();
    hand(&mut g, P0, &["Twinflame"]);
    pool_up(&mut g, &[('R', 6), ('G', 1)]);
    assert!(!offers(&g, garth, GARTH));

    // no combat still to come
    let (mut g, garth) = garth_table(1);
    g.step = Step::Main2;
    assert!(!offers(&g, garth, GARTH));
}

#[test]
fn the_garth_loop_makes_enough_hasty_garths_to_kill_the_table() {
    let (mut g, garth) = garth_table(1);
    let log = start(&mut g, garth, "enough hasty Garths");
    assert!(asked(&log, "Garth + Twinflame + Spark Double"), "{:?}", log.lock().unwrap());
    assert_eq!(g.winner, None, "the attack is the person's");
    let hasty = count(&g, "Garth One-Eye", |g, m| !g.perm(m).sick && !g.perm(m).tapped);
    assert!(hasty >= 20, "hasty Garths to attack with ({hasty})");
    // the tokens are not legendary, so the legend rule left them all alone, and they are exiled at the end step
    let tok = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token && g.perm(m).on_bf).unwrap();
    assert!(g.perm(tok).temp && g.perm(tok).exc.not_legendary);
    assert!(g.player(P0).hand.iter().any(|&c| &*g.db.get(c).name == "Twinflame"), "Regrowth returned Twinflame");
}

#[test]
fn the_garth_loop_goes_round_as_many_times_as_asked() {
    let mut last = 0;
    for how in ["once around", "twice around"] {
        let (mut g, garth) = garth_table(1);
        start(&mut g, garth, how);
        let n = count(&g, "Garth One-Eye", |g, m| g.perm(m).token);
        assert!(n > last, "{how}: {n} token Garths after {last}");
        last = n;
    }
}

#[test]
fn a_counterspell_on_twinflame_stops_the_garth_loop() {
    let mut countered = 0;
    for seed in 0..16u64 {
        let (mut g, garth) = garth_table(seed);
        lands(&mut g, P1, "Island", 2, false);
        hand(&mut g, P1, &["Counterspell"]);
        start(&mut g, garth, "enough hasty Garths");
        if count(&g, "Garth One-Eye", |g, m| g.perm(m).token) == 0 {
            countered += 1;
        }
    }
    assert!(countered >= 1, "the AI counters a combo piece sometimes");
}

/// P0's mana pool, and the person seated (the readiness checks pay from a person's pool)
fn pool_up(g: &mut Game, mana: &[(char, u32)]) {
    pool(g, P0, mana);
    seat_script(g, vec![]);
}
