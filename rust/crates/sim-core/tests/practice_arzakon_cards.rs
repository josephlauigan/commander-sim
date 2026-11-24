//! Practice mode (2026-11-23): Voltaic Key ({1}, {T}: untap target artifact) and Gleaming Splendor ({2}{W}: two target
//! players each draw a card), cards the Arzakon change brought into the owned decks, are listed for a person, who
//! chooses their targets (the AI's use of both is unchanged).

mod play_common;
use play_common::*;
use sim_core::human::human::abilities_of;
use std::sync::{Arc, Mutex};

/// a person at seat 0 whose first action is `action` and who picks the ability whose label starts with `ab`; `pick`
/// answers every other question (None: the first choice, or yes). Runs their main phase
fn person(
    g: &mut Game,
    action: Value,
    ab: &'static str,
    pick: impl Fn(&Request) -> Option<usize> + Send + Sync + 'static,
) -> Arc<ScriptHuman> {
    let first = Mutex::new(true);
    let h = Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            let mut f = first.lock().unwrap();
            let a = if *f { action.clone() } else { json!({"do": "pass"}) };
            *f = false;
            return Answer::Action(a);
        }
        if r.prompt.ends_with(": which ability?") {
            return Answer::Pick(
                r.choices
                    .iter()
                    .position(|c| c.starts_with(ab))
                    .unwrap_or_else(|| panic!("no ability {ab:?} in {:?}", r.choices)),
            );
        }
        match pick(r) {
            Some(k) => Answer::Pick(k),
            None if r.choices.is_empty() => Answer::Yes,
            None => Answer::Pick(0),
        }
    }));
    g.humans.add(P0, h.clone());
    sim_core::human::human::human_main(g, P0, false).unwrap();
    h
}

/// `person` using permanent `m`
fn use_perm(
    g: &mut Game,
    m: PermId,
    ab: &'static str,
    pick: impl Fn(&Request) -> Option<usize> + Send + Sync + 'static,
) -> Arc<ScriptHuman> {
    let i = g.player(P0).perms.iter().position(|&x| x == m).unwrap();
    person(g, json!({"do": "use", "perm": i}), ab, pick)
}

fn asked(h: &ScriptHuman, text: &str) -> bool {
    h.asked().iter().any(|r| r.prompt.contains(text))
}

fn prompts(h: &ScriptHuman) -> Vec<String> {
    h.asked().iter().map(|r| r.prompt.clone()).collect()
}

fn labels(g: &mut Game, m: PermId) -> Vec<String> {
    abilities_of(g, P0, m).unwrap().into_iter().map(|a| a.0).collect()
}

// ------------------------------------------------------------------ Voltaic Key
fn key_table() -> (Game, PermId, PermId) {
    let mut g = table(&["arzakon", "seph"]);
    let key = perm(&mut g, P0, "Voltaic Key");
    let ring = perm(&mut g, P0, "Sol Ring");
    g.perm_mut(ring).tapped = true;
    (g, key, ring)
}

#[test]
fn voltaic_key_is_listed_and_untaps_the_artifact_the_person_picks() {
    let (mut g, key, ring) = key_table();
    let ls = labels(&mut g, key);
    assert!(ls.iter().any(|x| x.starts_with("{1}, {T}: untap target artifact")), "{ls:?}");
    pool(&mut g, P0, &[('C', 1)]);
    let h = use_perm(&mut g, key, "{1}, {T}", |r| {
        if r.prompt.contains("Voltaic Key: untap which artifact?") {
            r.choices.iter().position(|c| c.contains("Sol Ring"))
        } else {
            None
        }
    });
    assert!(asked(&h, "Voltaic Key: untap which artifact?"), "{:?}", prompts(&h));
    assert!(!g.perm(ring).tapped, "the Sol Ring the person named is untapped");
    assert!(g.perm(key).tapped, "the Key was tapped for the cost");
    assert_eq!(pool_total(&g, P0), 0, "{{1}} was paid");
}

#[test]
fn voltaic_key_can_untap_an_opponents_artifact() {
    let (mut g, key, _ring) = key_table();
    let theirs = perm(&mut g, P1, "Arcane Signet");
    g.perm_mut(theirs).tapped = true;
    pool(&mut g, P0, &[('C', 1)]);
    use_perm(&mut g, key, "{1}, {T}", |r| {
        if r.prompt.contains("Voltaic Key: untap which artifact?") {
            r.choices.iter().position(|c| c.contains("Arcane Signet"))
        } else {
            None
        }
    });
    assert!(!g.perm(theirs).tapped, "any artifact may be the target");
}

#[test]
fn voltaic_key_offers_only_tapped_artifacts_and_itself() {
    let (mut g, key, _ring) = key_table();
    perm(&mut g, P0, "Arcane Signet"); // untapped: nothing to untap
    pool(&mut g, P0, &[('C', 1)]);
    let h = use_perm(&mut g, key, "{1}, {T}", |r| {
        if r.prompt.contains("Voltaic Key: untap which artifact?") {
            assert_eq!(r.choices.len(), 3, "Sol Ring, the Key itself and cancel: {:?}", r.choices);
            assert!(r.choices.iter().all(|c| !c.contains("Arcane Signet")), "{:?}", r.choices);
        }
        None
    });
    assert!(asked(&h, "Voltaic Key: untap which artifact?"));
}

#[test]
fn voltaic_key_needs_one_mana_and_an_untapped_key() {
    let (mut g, key, ring) = key_table();
    let h = use_perm(&mut g, key, "{1}, {T}", |_| None);
    assert!(h.told_kind("invalid").iter().any(|t| t.contains("Voltaic Key")), "{:?}", h.told());
    assert!(!g.perm(key).tapped, "refused: the Key stays untapped");
    assert!(g.perm(ring).tapped);
}

#[test]
fn voltaic_key_cancelling_the_target_costs_nothing() {
    let (mut g, key, ring) = key_table();
    pool(&mut g, P0, &[('C', 1)]);
    use_perm(&mut g, key, "{1}, {T}", |r| {
        if r.prompt.contains("Voltaic Key: untap which artifact?") { Some(r.choices.len() - 1) } else { None }
    });
    assert!(!g.perm(key).tapped && g.perm(ring).tapped);
    assert_eq!(pool_total(&g, P0), 1, "nothing was paid");
}

// ------------------------------------------------------------------ Gleaming Splendor
fn treasures(g: &Game, p: PlayerId) -> u32 {
    g.player(p).treasures as u32
}

#[test]
fn gleaming_splendor_asks_for_two_players_and_both_draw() {
    let mut g = table(&["yshtola", "seph", "arzakon"]);
    let s = perm(&mut g, P0, "Gleaming Splendor");
    let ls = labels(&mut g, s);
    assert!(ls.iter().any(|x| x.starts_with("{2}{W}: two target players each draw a card")), "{ls:?}");
    pool(&mut g, P0, &[('W', 1), ('C', 2)]);
    let (h0, h1, h2) = (g.player(P0).hand.len(), g.player(P1).hand.len(), g.player(P2).hand.len());
    let h = use_perm(&mut g, s, "{2}{W}", |r| {
        // the first target is the person, the second the third seat
        if r.prompt.contains("the first target player draws") {
            Some(0)
        } else if r.prompt.contains("the second target player draws") {
            Some(1)
        } else {
            None
        }
    });
    assert!(asked(&h, "Gleaming Splendor: the first target player"), "{:?}", prompts(&h));
    assert!(asked(&h, "Gleaming Splendor: the second target player"), "{:?}", prompts(&h));
    assert_eq!(g.player(P0).hand.len(), h0 + 1, "the person drew");
    assert_eq!(g.player(P2).hand.len(), h2 + 1, "the second target drew");
    assert_eq!(g.player(P1).hand.len(), h1, "the player left out did not");
    assert_eq!(pool_total(&g, P0), 0, "{{2}}{{W}} was paid");
}

#[test]
fn gleaming_splendor_in_a_two_player_game_has_nothing_to_choose() {
    let mut g = table(&["yshtola", "seph"]);
    let s = perm(&mut g, P0, "Gleaming Splendor");
    pool(&mut g, P0, &[('W', 1), ('C', 2)]);
    let (h0, h1) = (g.player(P0).hand.len(), g.player(P1).hand.len());
    let h = use_perm(&mut g, s, "{2}{W}", |_| None);
    assert!(
        h.asked().iter().all(|r| !r.prompt.starts_with("Gleaming Splendor:") || r.prompt.ends_with("which ability?")),
        "{:?}",
        prompts(&h)
    );
    assert_eq!((g.player(P0).hand.len(), g.player(P1).hand.len()), (h0 + 1, h1 + 1));
}

#[test]
fn gleaming_splendors_treasure_comes_for_an_opponents_second_draw_of_the_turn() {
    let mut g = table(&["yshtola", "seph"]);
    let s = perm(&mut g, P0, "Gleaming Splendor");
    pool(&mut g, P0, &[('W', 1), ('C', 2)]);
    sim_core::engine::zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!(treasures(&g, P0), 0, "their first draw gives nothing");
    use_perm(&mut g, s, "{2}{W}", |_| None);
    assert_eq!(treasures(&g, P0), 1, "the ability's draw is their second card of the turn");
}

#[test]
fn gleaming_splendor_needs_its_mana() {
    let mut g = table(&["yshtola", "seph"]);
    let s = perm(&mut g, P0, "Gleaming Splendor");
    pool(&mut g, P0, &[('C', 3)]); // no white
    let h0 = g.player(P0).hand.len();
    let h = use_perm(&mut g, s, "{2}{W}", |_| None);
    assert!(h.told_kind("invalid").iter().any(|t| t.contains("Gleaming Splendor")), "{:?}", h.told());
    assert_eq!(g.player(P0).hand.len(), h0);
}
