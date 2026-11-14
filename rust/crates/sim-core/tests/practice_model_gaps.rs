//! Practice mode: abilities of the owned decks' cards a person could not use, or could not choose for, found by the
//! trigger probe once it had the preconditions (model-gaps B1, 2026-11-05). Each test seats a person at P0, shows they
//! are asked, and checks that their answer is what happens. Also: the abilities model-gaps A1 gave Mishra's
//! Factory / Foundry, Steel Hellkite, Tezzeret's emblem and Liquimetal Torque, used by a person with their own choices.

mod play_common;
use play_common::*;
use sim_core::engine::values::{epow, etgh};
use sim_core::state::{DataKey, Val};
use std::sync::{Arc, Mutex};

#[derive(Clone, Copy)]
enum Use {
    Perm(PermId),
    Land(PermId),
}

/// a person at seat 0 who uses `what`, picks the ability whose label starts with `ab`, and answers the rest with `pick`
/// (None: the first choice, or yes); every other priority is a pass
fn person_uses(
    what: Use,
    ab: &'static str,
    pick: impl Fn(&Request) -> Option<usize> + Send + Sync + 'static,
) -> Arc<ScriptHuman> {
    let first = Mutex::new(true);
    Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            let mut f = first.lock().unwrap();
            let a = match (*f, what) {
                (true, Use::Perm(i)) => json!({"do": "use", "perm": i.0}),
                (true, Use::Land(i)) => json!({"do": "use", "land": i.0}),
                _ => json!({"do": "pass"}),
            };
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
    }))
}

fn perm_idx(g: &Game, m: PermId) -> usize {
    g.player(P0).perms.iter().position(|&x| x == m).unwrap()
}

fn land_idx(g: &Game, l: PermId) -> usize {
    g.player(P0).lands.iter().position(|&x| x == l).unwrap()
}

/// seat the person and run their main phase
fn run(
    g: &mut Game,
    what: Use,
    ab: &'static str,
    pick: impl Fn(&Request) -> Option<usize> + Send + Sync + 'static,
) -> Arc<ScriptHuman> {
    let what = match what {
        Use::Perm(m) => Use::Perm(PermId(perm_idx(g, m) as u32)),
        Use::Land(l) => Use::Land(PermId(land_idx(g, l) as u32)),
    };
    let h = person_uses(what, ab, pick);
    g.humans.add(P0, h.clone());
    sim_core::human::human::human_main(g, P0, false).unwrap();
    h
}

fn asked(h: &ScriptHuman, text: &str) -> bool {
    h.asked().iter().any(|r| r.prompt.contains(text))
}

fn prompts(h: &ScriptHuman) -> Vec<String> {
    h.asked().iter().map(|r| r.prompt.clone()).collect()
}

fn labels(g: &mut Game, p: PlayerId, m: PermId) -> Vec<String> {
    sim_core::human::human::abilities_of(g, p, m).unwrap().into_iter().map(|a| a.0).collect()
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

// ------------------------------------------------------------------ Loran of the Third Path
#[test]
fn loran_is_listed_and_asks_which_opponent_draws() {
    let mut g = table(&["yshtola", "seph", "thanos"]);
    let l = perm(&mut g, P0, "Loran of the Third Path");
    let ls = labels(&mut g, P0, l);
    assert!(ls.iter().any(|x| x.starts_with("{T}: you and target opponent each draw a card")), "{ls:?}");
    let (mine, p1, p2) = (g.player(P0).hand.len(), g.player(P1).hand.len(), g.player(P2).hand.len());
    let h = run(&mut g, Use::Perm(l), "{T}:", |r| {
        if r.prompt.contains("Loran: which opponent draws a card?") {
            r.choices.iter().position(|c| c.contains("Thanos"))
        } else {
            None
        }
    });
    assert!(asked(&h, "Loran: which opponent draws a card?"), "{:?}", prompts(&h));
    assert_eq!(g.player(P0).hand.len(), mine + 1, "the person drew");
    assert_eq!(g.player(P2).hand.len(), p2 + 1, "the opponent the person named drew");
    assert_eq!(g.player(P1).hand.len(), p1, "the other opponent did not");
    assert!(g.perm(l).tapped);
}

#[test]
fn loran_cannot_draw_while_tapped() {
    let mut g = table(&["yshtola", "seph"]);
    let l = perm(&mut g, P0, "Loran of the Third Path");
    g.perm_mut(l).tapped = true;
    let mine = g.player(P0).hand.len();
    let h = run(&mut g, Use::Perm(l), "{T}:", |_| None);
    assert!(h.told_kind("invalid").iter().any(|t| t.contains("tapped")), "{:?}", h.told());
    assert_eq!(g.player(P0).hand.len(), mine);
}

// ------------------------------------------------------------------ Maelstrom of the Spirit Dragon
fn maelstrom_table() -> (Game, PermId) {
    let mut g = table(&["urdragon", "seph"]);
    let l = lands(&mut g, P0, "Maelstrom of the Spirit Dragon", 1, false)[0];
    pool(&mut g, P0, &[('C', 4)]);
    (g, l)
}

#[test]
fn maelstrom_searches_for_the_dragon_the_person_names() {
    let (mut g, l) = maelstrom_table();
    let ls = sim_core::human::abilities::land_abilities(&g, P0, l);
    assert!(ls.iter().any(|a| a.label.contains("search your library for a Dragon card")));
    let h = run(&mut g, Use::Land(l), "{4}, {T}, sacrifice it", |r| {
        if r.prompt.contains("search for a Dragon card") {
            r.choices.iter().position(|c| c.starts_with("Dragonlord Dromoka"))
        } else {
            None
        }
    });
    assert!(asked(&h, "Maelstrom of the Spirit Dragon: search for a Dragon card"), "{:?}", prompts(&h));
    assert!(in_hand(&g, P0, "Dragonlord Dromoka"), "the Dragon the person named is in hand");
    assert!(!g.player(P0).lands.contains(&l), "the land was sacrificed");
    assert!(in_gy(&g, P0, "Maelstrom of the Spirit Dragon"));
    assert_eq!(pool_total(&g, P0), 0, "{{4}} was paid");
}

#[test]
fn maelstrom_offers_only_dragons_and_may_find_nothing() {
    let (mut g, l) = maelstrom_table();
    let hand = g.player(P0).hand.len();
    let h = run(&mut g, Use::Land(l), "{4}, {T}, sacrifice it", |r| {
        if r.prompt.contains("search for a Dragon card") {
            assert!(
                r.choices.iter().all(|c| !c.starts_with("Sol Ring") && !c.starts_with("Llanowar")),
                "{:?}",
                r.choices
            );
            r.choices.iter().position(|c| c.contains("find nothing"))
        } else {
            None
        }
    });
    assert!(asked(&h, "search for a Dragon card"));
    assert_eq!(g.player(P0).hand.len(), hand, "nothing found");
    assert!(!g.player(P0).lands.contains(&l), "the cost was paid all the same");
}

#[test]
fn maelstrom_needs_four_mana() {
    let (mut g, l) = maelstrom_table();
    g.player_mut(P0).pool = Default::default();
    pool(&mut g, P0, &[('C', 3)]);
    let h = run(&mut g, Use::Land(l), "{4}, {T}, sacrifice it", |_| None);
    assert!(h.told_kind("invalid").iter().any(|t| t.contains("Maelstrom")), "{:?}", h.told());
    assert!(g.player(P0).lands.contains(&l), "refused: the land stays");
}

// ------------------------------------------------------------------ Ramosian Revivalist
fn revivalist_table() -> (Game, PermId) {
    let mut g = table(&["galadriel", "seph"]);
    let r = perm(&mut g, P0, "Ramosian Revivalist");
    for n in ["Ramosian Sergeant", "Ramosian Lieutenant"] {
        let c = take(&mut g, P0, n);
        g.player_mut(P0).gy.push(c);
    }
    pool(&mut g, P0, &[('C', 6)]);
    (g, r)
}

#[test]
fn revivalist_targets_the_rebel_the_person_names_before_paying() {
    let (mut g, r) = revivalist_table();
    let h = run(&mut g, Use::Perm(r), "{6}, {T}", |q| {
        if q.prompt.contains("Ramosian Revivalist: return which Rebel") {
            q.choices.iter().position(|c| c.starts_with("Ramosian Sergeant"))
        } else {
            None
        }
    });
    assert!(asked(&h, "Ramosian Revivalist: return which Rebel permanent card?"), "{:?}", prompts(&h));
    assert!(has_perm(&g, P0, "Ramosian Sergeant"), "the Rebel the person named is back");
    assert!(!has_perm(&g, P0, "Ramosian Lieutenant") && in_gy(&g, P0, "Ramosian Lieutenant"));
    assert_eq!(pool_total(&g, P0), 0, "{{6}} was paid");
    assert!(g.perm(r).tapped);
}

#[test]
fn revivalist_cancelled_target_costs_nothing() {
    let (mut g, r) = revivalist_table();
    let h = run(&mut g, Use::Perm(r), "{6}, {T}", |q| {
        if q.prompt.contains("Ramosian Revivalist: return which Rebel") {
            q.choices.iter().position(|c| c == "cancel")
        } else {
            None
        }
    });
    assert!(asked(&h, "Ramosian Revivalist: return which Rebel"));
    assert_eq!(pool_total(&g, P0), 6, "nothing paid");
    assert!(!g.perm(r).tapped);
    assert!(in_gy(&g, P0, "Ramosian Sergeant") && in_gy(&g, P0, "Ramosian Lieutenant"));
}

// ------------------------------------------------------------------ model-gaps A1: the person uses the new abilities
/// an animated Mishra's Factory / Foundry (a token standing in for the land) and a second Assembly-Worker
fn two_workers(g: &mut Game, name: &str) -> (PermId, PermId) {
    let l = lands(g, P0, name, 1, false)[0];
    let m = sim_core::impls::lands::animate(g, P0, l, 2, 2, &[], false);
    g.perm_mut(m).ttypes.push("artifact");
    g.perm_mut(m).ttypes.push("assembly-worker");
    g.perm_mut(m).sick = false;
    let w = token(g, P0, 2);
    g.perm_mut(w).ttypes.push("artifact");
    g.perm_mut(w).ttypes.push("assembly-worker");
    (m, w)
}

#[test]
fn factory_pump_asks_which_assembly_worker() {
    let mut g = table(&["karn", "seph"]);
    let (f, w) = two_workers(&mut g, "Mishra's Factory");
    let h = run(&mut g, Use::Perm(f), "{T}: target Assembly-Worker", |r| {
        if r.prompt.contains("Mishra's Factory: which Assembly-Worker") {
            r.choices.iter().position(|c| c.starts_with("Artifact Assembly-Worker token"))
        } else {
            None
        }
    });
    assert!(asked(&h, "Mishra's Factory: which Assembly-Worker gets +1/+1?"), "{:?}", prompts(&h));
    assert_eq!((epow(&g, w), etgh(&g, w)), (3, 3), "the token the person named");
    assert_eq!((epow(&g, f), etgh(&g, f)), (2, 2), "not the Factory");
    assert!(g.perm(f).tapped);
}

#[test]
fn foundry_pump_only_offers_attacking_workers_and_pays_one() {
    let mut g = table(&["karn", "seph"]);
    let (f, w) = two_workers(&mut g, "Mishra's Foundry");
    g.in_combat.push(w); // only the token attacks
    pool(&mut g, P0, &[('C', 1)]);
    let h = run(&mut g, Use::Perm(f), "{1}, {T}: target attacking", |r| {
        if r.prompt.contains("Mishra's Foundry: which attacking Assembly-Worker") {
            assert_eq!(r.choices.len(), 2, "the token and cancel: {:?}", r.choices);
        }
        None
    });
    assert!(asked(&h, "Mishra's Foundry: which attacking Assembly-Worker gets +2/+2?"), "{:?}", prompts(&h));
    assert_eq!((epow(&g, w), etgh(&g, w)), (4, 4));
    assert_eq!(pool_total(&g, P0), 0);
    assert!(g.perm(f).tapped);
}

#[test]
fn steel_hellkite_x_asks_x_and_destroys_that_mana_value() {
    let mut g = table(&["karn", "seph"]);
    let hk = perm(&mut g, P0, "Steel Hellkite");
    let (stone, ring) = (perm(&mut g, P1, "Mind Stone"), perm(&mut g, P1, "Sol Ring"));
    let st = g.turn_stamp();
    g.perm_mut(hk).data.set(DataKey::Hit, Val::List(vec![Val::Stamp(st), Val::Player(P1)]));
    pool(&mut g, P0, &[('C', 3)]);
    let h = run(&mut g, Use::Perm(hk), "{X}: destroy", |r| {
        if r.prompt.contains("Steel Hellkite: X = ?") { r.choices.iter().position(|c| c == "X = 2") } else { None }
    });
    assert!(asked(&h, "Steel Hellkite: X = ?"), "{:?}", prompts(&h));
    assert!(!g.perm(stone).on_bf, "Mind Stone has mana value 2");
    assert!(g.perm(ring).on_bf, "Sol Ring has mana value 1");
    assert_eq!(pool_total(&g, P0), 1, "X = 2 was paid");
}

#[test]
fn liquimetal_torque_asks_which_permanent_becomes_an_artifact() {
    let mut g = table(&["karn", "seph"]);
    let t = perm(&mut g, P0, "Liquimetal Torque");
    let (elves, birds) = (perm(&mut g, P0, "Llanowar Elves"), perm(&mut g, P0, "Birds of Paradise"));
    let h = run(&mut g, Use::Perm(t), "{T}: target nonland permanent", |r| {
        if r.prompt.contains("Liquimetal Torque: which permanent") {
            r.choices.iter().position(|c| c.starts_with("Birds of Paradise"))
        } else {
            None
        }
    });
    assert!(asked(&h, "Liquimetal Torque: which permanent becomes an artifact?"), "{:?}", prompts(&h));
    assert!(sim_core::impls::t5::is_artifact(&g, birds), "the permanent the person named");
    assert!(!sim_core::impls::t5::is_artifact(&g, elves));
}

#[test]
fn tezzerets_emblem_asks_which_artifact_gets_the_counters() {
    let mut g = table(&["karn", "seph"]);
    g.player_mut(P0).emblems.push(sim_core::impls::t6_godo::TEZ_EMBLEM);
    let (stone, ring) = (perm(&mut g, P0, "Mind Stone"), perm(&mut g, P0, "Sol Ring"));
    let h = person_uses(Use::Perm(PermId(0)), "", |r| {
        if r.prompt.contains("Tezzeret's emblem") {
            r.choices.iter().position(|c| c.starts_with("Sol Ring"))
        } else {
            None
        }
    });
    g.humans.add(P0, h.clone());
    sim_core::impls::t6_godo::tez_emblem(&mut g, P0).unwrap();
    assert!(asked(&h, "Tezzeret's emblem: which artifact gets three +1/+1 counters?"), "{:?}", prompts(&h));
    assert!(g.is_creature(ring) && epow(&g, ring) == 3, "the artifact the person named is a 3/3 Robot");
    assert!(!g.is_creature(stone) && g.perm(stone).plus == 0);
}
