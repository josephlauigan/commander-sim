//! Practice mode: triggers and abilities the AI used to decide for the person (trigger-fixes task). Each test seats a
//! person at P0, shows they are asked, and checks that their answer is what happens.

mod play_common;
use play_common::*;
use std::sync::Arc;

/// a person who answers with `pick` (None: the first choice, or yes); at a priority, pass
fn person(pick: impl Fn(&Request) -> Option<usize> + Send + Sync + 'static) -> Arc<ScriptHuman> {
    Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        match pick(r) {
            Some(k) => Answer::Pick(k),
            None if r.choices.is_empty() => Answer::Yes,
            None => Answer::Pick(0),
        }
    }))
}

/// the index of the first choice containing `word`
fn with(r: &Request, word: &str) -> Option<usize> {
    r.choices.iter().position(|c| c.contains(word))
}

fn asked(h: &ScriptHuman, text: &str) -> bool {
    h.asked().iter().any(|r| r.prompt.contains(text))
}

fn count(g: &Game, p: PlayerId, name: &str) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).name.contains(name)).count()
}

// ------------------------------------------------------------------ Felidar Retreat
#[test]
fn felidar_retreat_person_picks_the_counters_mode() {
    let mut g = table(&["alela", "seph"]);
    perm(&mut g, P0, "Felidar Retreat");
    let bear = perm(&mut g, P0, "Llanowar Elves");
    let h = person(|r| if r.prompt.contains("Felidar Retreat") { with(r, "+1/+1") } else { None });
    g.humans.add(P0, h.clone());
    sim_core::engine::zones::landfall(&mut g, P0).unwrap();
    assert!(asked(&h, "Felidar Retreat: choose a mode"), "the person was asked");
    assert_eq!(count(&g, P0, "Cat"), 0, "no Cat token: the person chose the counters");
    assert_eq!(g.perm(bear).plus, 1, "a +1/+1 counter on the creature");
    assert!(g.perm(bear).eot_kw.contains(&"vigilance"), "and vigilance until end of turn");
}

#[test]
fn felidar_retreat_person_picks_the_cat() {
    let mut g = table(&["alela", "seph"]);
    perm(&mut g, P0, "Felidar Retreat");
    let bear = perm(&mut g, P0, "Llanowar Elves");
    let h = person(|r| if r.prompt.contains("Felidar Retreat") { with(r, "Cat") } else { None });
    g.humans.add(P0, h.clone());
    sim_core::engine::zones::landfall(&mut g, P0).unwrap();
    assert!(asked(&h, "Felidar Retreat: choose a mode"));
    assert_eq!(
        g.player(P0).perms.iter().filter(|&&m| g.perm(m).token && g.perm(m).pow == 2).count(),
        1,
        "the 2/2 Cat token"
    );
    assert_eq!(g.perm(bear).plus, 0);
}

// ------------------------------------------------------------------ Sun Titan
#[test]
fn sun_titan_returns_the_card_the_person_picks() {
    let mut g = table(&["thrasios", "seph"]);
    let st = perm(&mut g, P0, "Sun Titan");
    let sol = take(&mut g, P0, "Sol Ring");
    let elves = take(&mut g, P0, "Llanowar Elves");
    g.player_mut(P0).gy.extend([sol, elves]);
    let h = person(|r| if r.prompt.contains("Sun Titan") { with(r, "Sol Ring") } else { None });
    g.humans.add(P0, h.clone());
    sim_core::engine::zones::sun_titan(&mut g, P0, st).unwrap();
    assert!(asked(&h, "Sun Titan: return which"), "the person was asked");
    assert!(has_perm(&g, P0, "Sol Ring"), "the picked card returned");
    assert!(!has_perm(&g, P0, "Llanowar Elves"), "the AI's pick (the creature) stays");
    assert!(g.player(P0).gy.contains(&elves));
}

#[test]
fn sun_titan_person_may_return_nothing() {
    let mut g = table(&["thrasios", "seph"]);
    let st = perm(&mut g, P0, "Sun Titan");
    let elves = take(&mut g, P0, "Llanowar Elves");
    g.player_mut(P0).gy.push(elves);
    let h = person(|r| if r.prompt.contains("Sun Titan") { with(r, "none") } else { None });
    g.humans.add(P0, h.clone());
    sim_core::engine::zones::sun_titan(&mut g, P0, st).unwrap();
    assert!(asked(&h, "Sun Titan: return which"));
    assert!(!has_perm(&g, P0, "Llanowar Elves"), "the person chose none");
    assert!(g.player(P0).gy.contains(&elves));
}

#[test]
fn sun_titan_ai_still_takes_the_best() {
    let mut g = table(&["thrasios", "seph"]);
    let st = perm(&mut g, P0, "Sun Titan");
    let sol = take(&mut g, P0, "Sol Ring");
    let elves = take(&mut g, P0, "Llanowar Elves");
    g.player_mut(P0).gy.extend([sol, elves]);
    sim_core::engine::zones::sun_titan(&mut g, P0, st).unwrap();
    assert!(has_perm(&g, P0, "Llanowar Elves"), "the AI's pick is the creature");
}

// ------------------------------------------------------------------ Mirrorworks
#[test]
fn mirrorworks_asks_the_person_before_paying() {
    let mut g = table(&["karn", "seph"]);
    g.active = Some(P0);
    perm(&mut g, P0, "Mirrorworks");
    lands(&mut g, P0, "Wastes", 2, false);
    let h = Arc::new(ScriptHuman::from_fn(|r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        if r.prompt.contains("Mirrorworks") { Answer::Pick(1) } else { Answer::Pick(0) }
    }));
    g.humans.add(P0, h.clone());
    perm(&mut g, P0, "Sol Ring");
    assert!(asked(&h, "Mirrorworks: pay {2}"), "the person was asked");
    assert_eq!(count(&g, P0, "Sol Ring"), 1, "they said no: no copy");
    assert!(g.player(P0).lands.iter().all(|&l| !g.perm(l).tapped), "nothing was paid");
}

#[test]
fn mirrorworks_pays_and_copies_when_the_person_says_yes() {
    let mut g = table(&["karn", "seph"]);
    g.active = Some(P0);
    perm(&mut g, P0, "Mirrorworks");
    lands(&mut g, P0, "Wastes", 2, false);
    let h = Arc::new(ScriptHuman::from_fn(|r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        if r.prompt.contains("Mirrorworks") { Answer::Pick(0) } else { Answer::Pick(0) }
    }));
    g.humans.add(P0, h.clone());
    perm(&mut g, P0, "Sol Ring");
    assert!(asked(&h, "Mirrorworks: pay {2}"));
    assert_eq!(count(&g, P0, "Sol Ring"), 2, "the card and its token copy");
    assert!(g.player(P0).lands.iter().all(|&l| g.perm(l).tapped), "{{2}} was paid");
}

// ------------------------------------------------------------------ Garth One-Eye ("you may cast the copy")
/// a person at seat 0 who uses Garth (the perm at index `i`), names `card`, and answers the rest with `pick`
fn garth_person(
    i: usize,
    card: &'static str,
    pick: impl Fn(&Request) -> Option<usize> + Send + Sync + 'static,
) -> Arc<ScriptHuman> {
    let first = std::sync::Mutex::new(true);
    Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            let mut f = first.lock().unwrap();
            let a = if *f { json!({"do": "use", "perm": i}) } else { json!({"do": "pass"}) };
            *f = false;
            return Answer::Action(a);
        }
        if r.prompt.contains("choose a card name") {
            return Answer::Pick(r.choices.iter().position(|c| c.contains(card)).unwrap());
        }
        match pick(r) {
            Some(k) => Answer::Pick(k),
            None if r.choices.is_empty() => Answer::Yes,
            None => Answer::Pick(0),
        }
    }))
}

#[test]
fn garths_copy_is_not_cast_when_the_person_declines() {
    let mut g = table(&["urdragon", "seph"]);
    let garth = perm(&mut g, P0, "Garth One-Eye");
    let i = g.player(P0).perms.iter().position(|&m| m == garth).unwrap();
    pool(&mut g, P0, &[('R', 2), ('C', 4)]);
    let h = garth_person(i, "Shivan", |r| {
        if r.prompt.contains("cast the copy") { with(r, "cast Shivan").map(|_| r.choices.len() - 1) } else { None }
    });
    g.humans.add(P0, h.clone());
    sim_core::human::human::human_main(&mut g, P0, false).unwrap();
    assert!(asked(&h, "Garth One-Eye: cast the copy of Shivan Dragon"), "the person was asked");
    assert!(!has_perm(&g, P0, "Shivan Dragon"), "declined: no Shivan Dragon");
    assert_eq!(pool_total(&g, P0), 6, "nothing was paid");
    assert!(sim_core::impls::garth::chosen(&g, garth, 3), "the name stays used");
    assert!(g.perm(garth).tapped);
}

#[test]
fn garths_copy_is_cast_when_the_person_accepts() {
    let mut g = table(&["urdragon", "seph"]);
    let garth = perm(&mut g, P0, "Garth One-Eye");
    let i = g.player(P0).perms.iter().position(|&m| m == garth).unwrap();
    pool(&mut g, P0, &[('R', 2), ('C', 4)]);
    let h = garth_person(i, "Shivan", |_| None);
    g.humans.add(P0, h.clone());
    sim_core::human::human::human_main(&mut g, P0, false).unwrap();
    assert!(asked(&h, "Garth One-Eye: cast the copy of Shivan Dragon"));
    assert!(has_perm(&g, P0, "Shivan Dragon"), "accepted: cast");
}

#[test]
fn garths_braingeyser_x_counts_untapped_lands_and_taps_them() {
    let mut g = table(&["urdragon", "seph"]);
    let garth = perm(&mut g, P0, "Garth One-Eye");
    let i = g.player(P0).perms.iter().position(|&m| m == garth).unwrap();
    let ls = lands(&mut g, P0, "Island", 5, false); // {X}{U}{U} with X = 3: five untapped Islands, an empty pool
    let them = sim_core::human::legal::describe_target(&g, P0, sim_core::human::Target::Player(P1));
    let h = garth_person(i, "Braingeyser", move |r| {
        if r.prompt.contains("choose X") {
            assert_eq!(r.choices.last().map(String::as_str), Some("X = 3"), "X tops out at 3: {:?}", r.choices);
            r.choices.iter().position(|c| c == "X = 3")
        } else if r.prompt.contains("which player draws") {
            r.choices.iter().position(|c| *c == them)
        } else {
            None
        }
    });
    g.humans.add(P0, h.clone());
    let theirs = g.player(P1).hand.len();
    sim_core::human::human::human_main(&mut g, P0, false).unwrap();
    assert!(asked(&h, "choose X"), "{:?}", h.told());
    assert_eq!(g.player(P1).hand.len(), theirs + 3, "the opponent drew X = 3");
    assert!(ls.iter().all(|&l| g.perm(l).tapped), "the Islands were tapped to pay");
    assert_eq!(pool_total(&g, P0), 0);
}

// ================================================================== planeswalkers and Blightspeaker (walker-fixes)
/// a person at seat 0 who uses the walker (the perm at index `i`), picks the ability whose label starts with `ab` (e.g.
/// "-3:"), and answers the rest with `pick` (None: the first choice, or yes)
fn walker_person(
    i: usize,
    ab: &'static str,
    pick: impl Fn(&Request) -> Option<usize> + Send + Sync + 'static,
) -> Arc<ScriptHuman> {
    let first = std::sync::Mutex::new(true);
    Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            let mut f = first.lock().unwrap();
            let a = if *f { json!({"do": "use", "perm": i}) } else { json!({"do": "pass"}) };
            *f = false;
            return Answer::Action(a);
        }
        if r.prompt.ends_with("which ability?") {
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

fn idx(g: &Game, m: PermId) -> usize {
    g.player(P0).perms.iter().position(|&x| x == m).unwrap()
}

/// the person uses walker `m`'s ability `ab` and answers with `pick`; the game's main phase is run
fn use_walker(
    g: &mut Game,
    m: PermId,
    ab: &'static str,
    pick: impl Fn(&Request) -> Option<usize> + Send + Sync + 'static,
) -> Arc<ScriptHuman> {
    let h = walker_person(idx(g, m), ab, pick);
    g.humans.add(P0, h.clone());
    sim_core::human::human::human_main(g, P0, false).unwrap();
    h
}

/// the labels of everything the person can activate on `m`
fn labels(g: &mut Game, m: PermId) -> Vec<String> {
    sim_core::human::human::abilities_of(g, P0, m).unwrap().into_iter().map(|a| a.0).collect()
}

fn lists(g: &mut Game, m: PermId, want: &[&str]) {
    let ls = labels(g, m);
    for w in want {
        assert!(ls.iter().any(|l| l.starts_with(w)), "{w} is listed: {ls:?}");
    }
    assert_eq!(ls.len(), want.len(), "every loyalty ability and no more: {ls:?}");
}

fn exiled(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).exile.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

// ------------------------------------------------------------------ Karn Liberated
#[test]
fn karn_liberated_lists_all_three_abilities() {
    let mut g = table(&["karn", "seph"]);
    let k = perm(&mut g, P0, "Karn Liberated");
    lists(&mut g, k, &["+4:", "-3:", "-14:"]);
}

#[test]
fn karn_liberated_plus_exiles_from_the_player_the_person_names() {
    let mut g = table(&["karn", "seph"]);
    let k = perm(&mut g, P0, "Karn Liberated");
    hand(&mut g, P1, &["Sol Ring", "Forest", "Shivan Dragon"]);
    let (mine, theirs) = (g.player(P0).hand.len(), g.player(P1).hand.len());
    let h = use_walker(&mut g, k, "+4:", |r| {
        if r.prompt.contains("which player exiles") {
            r.choices.iter().position(|c| !c.contains("(you)"))
        } else {
            None
        }
    });
    assert!(asked(&h, "Karn Liberated +4: which player exiles a card"), "the person chose the player");
    assert_eq!(g.player(P1).hand.len(), theirs - 1, "the opponent exiled a card from their hand");
    assert_eq!(g.player(P1).exile.len(), 1, "exiled, not discarded");
    assert_eq!(g.player(P0).hand.len(), mine);
    assert_eq!(g.perm(k).loyalty, Some(10));
}

#[test]
fn karn_liberated_plus_on_yourself_asks_which_card_you_exile() {
    let mut g = table(&["karn", "seph"]);
    let k = perm(&mut g, P0, "Karn Liberated");
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Sol Ring", "Llanowar Elves", "Mind Stone"]);
    let h = use_walker(&mut g, k, "+4:", |r| {
        if r.prompt.contains("which player exiles") {
            r.choices.iter().position(|c| c.contains("(you)"))
        } else if r.prompt.contains("exile which card from your hand") {
            r.choices.iter().position(|c| c.contains("Llanowar"))
        } else {
            None
        }
    });
    assert!(asked(&h, "Karn Liberated: exile which card from your hand?"), "the target player chooses");
    assert!(exiled(&g, P0, "Llanowar Elves"), "the card the person picked");
    assert!(in_hand(&g, P0, "Sol Ring") && in_hand(&g, P0, "Mind Stone"));
}

#[test]
fn karn_liberated_minus_three_exiles_the_permanent_the_person_picks() {
    let mut g = table(&["karn", "seph"]);
    let k = perm(&mut g, P0, "Karn Liberated");
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Shivan Dragon");
    let island = lands(&mut g, P1, "Island", 1, false)[0];
    let h = use_walker(&mut g, k, "-3:", |r| {
        if r.prompt.contains("exile which permanent") {
            r.choices.iter().position(|c| c.starts_with("Island"))
        } else {
            None
        }
    });
    assert!(asked(&h, "Karn Liberated -3: exile which permanent?"));
    assert!(!g.perm(island).on_bf, "the land (a permanent the AI's code never chose) is gone");
    assert!(exiled(&g, P1, "Island"), "exiled");
    assert!(has_perm(&g, P1, "Sol Ring") && has_perm(&g, P1, "Shivan Dragon"), "the others stay");
    assert_eq!(g.perm(k).loyalty, Some(3));
}

// ------------------------------------------------------------------ Ugin, the Ineffable
#[test]
fn ugin_lists_both_abilities_and_shows_the_exiled_card() {
    let mut g = table(&["karn", "seph"]);
    let u = perm(&mut g, P0, "Ugin, the Ineffable");
    lists(&mut g, u, &["+1:", "-3:"]);
    let top = *g.player(P0).library.last().unwrap();
    let name = g.db.get(top).name.to_string();
    let h = use_walker(&mut g, u, "+1:", |_| None);
    assert!(
        h.told().iter().any(|t| t.1.contains("you look at") && t.1.contains(&name)),
        "told the card: {:?}",
        h.told()
    );
    assert!(has_perm(&g, P0, "Spirit") || g.player(P0).perms.iter().any(|&m| g.perm(m).token), "the Spirit");
}

#[test]
fn ugin_minus_three_destroys_the_coloured_permanent_the_person_picks() {
    let mut g = table(&["karn", "seph"]);
    let u = perm(&mut g, P0, "Ugin, the Ineffable");
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Shivan Dragon");
    perm(&mut g, P1, "Llanowar Elves");
    let seen = Arc::new(std::sync::Mutex::new(vec![]));
    let s2 = seen.clone();
    let h = use_walker(&mut g, u, "-3:", move |r| {
        if r.prompt.contains("Ugin, the Ineffable -3") {
            *s2.lock().unwrap() = r.choices.clone();
            r.choices.iter().position(|c| c.starts_with("Llanowar"))
        } else {
            None
        }
    });
    assert!(asked(&h, "Ugin, the Ineffable -3: destroy which permanent?"));
    let offered = seen.lock().unwrap().clone();
    assert!(
        !offered.iter().any(|c| c.starts_with("Sol Ring")),
        "a colourless permanent isn't a legal target: {offered:?}"
    );
    assert!(!has_perm(&g, P1, "Llanowar Elves"), "the one the person picked is destroyed");
    assert!(has_perm(&g, P1, "Shivan Dragon"), "the AI's pick (the best creature) stays");
}

// ------------------------------------------------------------------ Karn, Scion of Urza
fn silver(g: &mut Game, p: PlayerId, names: &[&str]) {
    for n in names {
        let c = card(g, n);
        g.player_mut(p).exile.push(c);
        g.player_mut(p).exile.last_mut().unwrap().silver = true;
    }
}

#[test]
fn karn_scion_lists_all_three_and_minus_one_returns_the_card_the_person_picks() {
    let mut g = table(&["karn", "seph"]);
    let k = perm(&mut g, P0, "Karn, Scion of Urza");
    lists(&mut g, k, &["+1:", "-1:", "-2:"]);
    // the AI would take the better of the two; the person takes either, so one of the two runs differs from the AI's pick
    for pick_name in ["Llanowar Elves", "Sol Ring"] {
        let mut g = table(&["karn", "seph"]);
        let k = perm(&mut g, P0, "Karn, Scion of Urza");
        silver(&mut g, P0, &["Llanowar Elves", "Sol Ring"]);
        g.player_mut(P0).hand.clear();
        let h = use_walker(&mut g, k, "-1:", move |r| {
            if r.prompt.contains("Karn, Scion of Urza -1") {
                r.choices.iter().position(|c| c.starts_with(pick_name))
            } else {
                None
            }
        });
        assert!(asked(&h, "Karn, Scion of Urza -1: put which card with a silver counter"));
        assert!(in_hand(&g, P0, pick_name), "{pick_name} is in hand");
        assert_eq!(g.player(P0).hand.len(), 1);
        assert_eq!(g.player(P0).exile.silver().len(), 1, "the other card stays in exile");
    }
}

#[test]
fn karn_scion_plus_and_minus_two_still_work() {
    let mut g = table(&["karn", "seph"]);
    let k = perm(&mut g, P0, "Karn, Scion of Urza");
    let n = g.player(P0).hand.len();
    use_walker(&mut g, k, "+1:", |_| None);
    assert_eq!(g.player(P0).hand.len(), n + 1, "one card to hand");
    assert_eq!(g.player(P0).exile.silver().len(), 1, "the other exiled with a silver counter");
    let mut g = table(&["karn", "seph"]);
    let k = perm(&mut g, P0, "Karn, Scion of Urza");
    use_walker(&mut g, k, "-2:", |_| None);
    assert!(
        g.player(P0).perms.iter().any(|&m| g.perm(m).data.truthy(sim_core::state::DataKey::Construct)),
        "a Construct"
    );
}

// ------------------------------------------------------------------ Karn, the Great Creator
#[test]
fn karn_great_creator_plus_animates_the_artifact_the_person_picks_until_their_next_turn() {
    let mut g = table(&["karn", "seph"]);
    let k = perm(&mut g, P0, "Karn, the Great Creator");
    lists(&mut g, k, &["+1:", "-2:"]);
    let sol = perm(&mut g, P0, "Sol Ring");
    let stone = perm(&mut g, P0, "Mind Stone");
    let h = use_walker(&mut g, k, "+1:", |r| {
        if r.prompt.contains("Karn, the Great Creator +1") {
            r.choices.iter().position(|c| c.starts_with("Mind Stone"))
        } else {
            None
        }
    });
    assert!(asked(&h, "Karn, the Great Creator +1: which noncreature artifact"));
    assert!(g.is_creature(stone), "Mind Stone is a creature");
    assert_eq!((g.perm(stone).pow, g.perm(stone).tgh), (2, 2), "power and toughness equal to its mana value");
    assert!(!g.is_creature(sol), "the other artifact isn't");
    sim_core::human::walkers::karn_revert(&mut g, P1);
    assert!(g.is_creature(stone), "an opponent's turn doesn't end it");
    sim_core::human::walkers::karn_revert(&mut g, P0);
    assert!(!g.is_creature(stone), "your next turn ends it");
}

#[test]
fn karn_great_creator_minus_two_takes_the_artifact_card_the_person_picks() {
    let mut g = table(&["karn", "seph"]);
    let k = perm(&mut g, P0, "Karn, the Great Creator");
    for n in ["Sol Ring", "Mind Stone"] {
        let c = card(&g, n);
        g.player_mut(P0).exile.push(c);
    }
    g.player_mut(P0).hand.clear();
    let h = use_walker(&mut g, k, "-2:", |r| {
        if r.prompt.contains("Karn, the Great Creator -2") {
            r.choices.iter().position(|c| c.starts_with("Mind Stone"))
        } else {
            None
        }
    });
    assert!(asked(&h, "Karn, the Great Creator -2: put which artifact card"));
    assert!(in_hand(&g, P0, "Mind Stone") && !in_hand(&g, P0, "Sol Ring"), "the picked card (the AI takes the first)");
    assert!(exiled(&g, P0, "Sol Ring"));
}

// ------------------------------------------------------------------ Teferi, Time Raveler
#[test]
fn teferi_minus_three_bounces_the_permanent_the_person_picks_and_draws() {
    let mut g = table(&["thanos", "seph"]);
    let t = perm(&mut g, P0, "Teferi, Time Raveler");
    lists(&mut g, t, &["+1:", "-3:"]);
    perm(&mut g, P1, "Shivan Dragon");
    perm(&mut g, P1, "Llanowar Elves");
    let (mine, theirs) = (g.player(P0).hand.len(), g.player(P1).hand.len());
    let h = use_walker(&mut g, t, "-3:", |r| {
        if r.prompt.contains("Teferi, Time Raveler -3") {
            r.choices.iter().position(|c| c.starts_with("Llanowar"))
        } else {
            None
        }
    });
    assert!(asked(&h, "Teferi, Time Raveler -3: return which"));
    assert!(!has_perm(&g, P1, "Llanowar Elves") && has_perm(&g, P1, "Shivan Dragon"), "the picked one is bounced");
    assert_eq!(g.player(P1).hand.len(), theirs + 1);
    assert_eq!(g.player(P0).hand.len(), mine + 1, "and Teferi's player drew");
}

#[test]
fn teferi_minus_three_may_have_no_target_and_still_draws() {
    let mut g = table(&["thanos", "seph"]);
    let t = perm(&mut g, P0, "Teferi, Time Raveler");
    perm(&mut g, P1, "Shivan Dragon");
    let mine = g.player(P0).hand.len();
    use_walker(&mut g, t, "-3:", |r| {
        if r.prompt.contains("Teferi, Time Raveler -3") { Some(r.choices.len() - 1) } else { None }
    });
    assert!(has_perm(&g, P1, "Shivan Dragon"), "nothing bounced: the person chose no target");
    assert_eq!(g.player(P0).hand.len(), mine + 1);
}

// ------------------------------------------------------------------ Wrenn and Six
#[test]
fn wrenn_and_six_plus_returns_the_land_the_person_picks() {
    let mut g = table(&["urdragon", "seph"]);
    let w = perm(&mut g, P0, "Wrenn and Six");
    lists(&mut g, w, &["+1:", "-1:", "-7:"]);
    for n in ["Forest", "Island"] {
        let c = card(&g, n);
        g.player_mut(P0).gy.push(c);
    }
    let h = use_walker(&mut g, w, "+1:", |r| {
        if r.prompt.contains("Wrenn and Six +1") {
            r.choices.iter().position(|c| c.starts_with("Island"))
        } else {
            None
        }
    });
    assert!(asked(&h, "Wrenn and Six +1: return which land"));
    assert!(in_hand(&g, P0, "Island"), "the picked land");
    assert!(g.player(P0).gy.iter().any(|&c| &*g.db.get(c).name == "Forest"), "the other stays");
}

#[test]
fn wrenn_and_six_minus_one_hits_the_target_the_person_picks() {
    let mut g = table(&["urdragon", "seph"]);
    let w = perm(&mut g, P0, "Wrenn and Six");
    perm(&mut g, P1, "Llanowar Elves");
    let life = g.player(P1).life;
    let h = use_walker(&mut g, w, "-1:", |r| {
        if r.prompt.contains("Wrenn and Six -1") {
            r.choices.iter().position(|c| c.contains("player") && !c.contains("(you)"))
        } else {
            None
        }
    });
    assert!(asked(&h, "Wrenn and Six -1: 1 damage to which target?"));
    assert_eq!(g.player(P1).life, life - 1, "the player the person picked");
    assert!(has_perm(&g, P1, "Llanowar Elves"), "the AI's pick (the 1-toughness creature) lives");
}

// ------------------------------------------------------------------ Venser, the Sojourner
#[test]
fn venser_lists_all_three_abilities() {
    let mut g = table(&["alela", "seph"]);
    let v = perm(&mut g, P0, "Venser, the Sojourner");
    lists(&mut g, v, &["+2:", "-1:", "-8:"]);
}

#[test]
fn venser_plus_exiles_the_permanent_the_person_picks_until_the_end_step() {
    let mut g = table(&["alela", "seph"]);
    let v = perm(&mut g, P0, "Venser, the Sojourner");
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P0, "Mind Stone");
    let h = use_walker(&mut g, v, "+2:", |r| {
        if r.prompt.contains("Venser, the Sojourner +2") {
            r.choices.iter().position(|c| c.starts_with("Mind Stone"))
        } else {
            None
        }
    });
    assert!(asked(&h, "Venser, the Sojourner +2: exile which permanent you own?"));
    assert!(!has_perm(&g, P0, "Mind Stone") && exiled(&g, P0, "Mind Stone"), "exiled");
    assert!(has_perm(&g, P0, "Sol Ring"));
    sim_core::cardcode::delayed_end_step(&mut g, P0).unwrap();
    assert!(has_perm(&g, P0, "Mind Stone"), "back at the beginning of the next end step");
}

#[test]
fn venser_minus_one_makes_the_persons_creatures_unblockable() {
    let mut g = table(&["alela", "seph"]);
    let v = perm(&mut g, P0, "Venser, the Sojourner");
    use_walker(&mut g, v, "-1:", |_| None);
    assert_eq!(g.player(P0).unbl_all, Some(g.turn_stamp()));
    assert_eq!(g.perm(v).loyalty, Some(2));
}

// ------------------------------------------------------------------ Blightspeaker
#[test]
fn blightspeaker_asks_which_player_loses_life() {
    let mut g = table(&["kefka", "seph"]);
    let b = perm(&mut g, P0, "Blightspeaker");
    lists(&mut g, b, &["{T}:", "{4}, {T}:"]);
    let (mine, theirs) = (g.player(P0).life, g.player(P1).life);
    let h = use_walker(&mut g, b, "{T}:", |r| {
        if r.prompt.contains("Blightspeaker: which player") {
            r.choices.iter().position(|c| c.contains("(you)"))
        } else {
            None
        }
    });
    assert!(asked(&h, "Blightspeaker: which player loses 1 life?"), "the person chose the player");
    assert_eq!(g.player(P0).life, mine - 1, "the person drained themselves");
    assert_eq!(g.player(P1).life, theirs);
    assert!(g.perm(b).tapped);
}

#[test]
fn blightspeaker_search_asks_which_rebel() {
    let mut g = table(&["kefka", "seph"]);
    let b = perm(&mut g, P0, "Blightspeaker");
    for n in ["Ramosian Sergeant", "Ramosian Lieutenant"] {
        let c = card(&g, n);
        g.player_mut(P0).library.push(c);
    }
    pool(&mut g, P0, &[('C', 4)]);
    let h = use_walker(&mut g, b, "{4}, {T}:", |r| {
        if r.prompt.contains("Blightspeaker: put which Rebel") {
            r.choices.iter().position(|c| c.starts_with("Ramosian Lieutenant"))
        } else {
            None
        }
    });
    assert!(
        asked(&h, "Blightspeaker: put which Rebel (mana value 3 or less)"),
        "{:?}",
        h.asked().iter().map(|r| r.prompt.clone()).collect::<Vec<_>>()
    );
    assert!(
        has_perm(&g, P0, "Ramosian Lieutenant") && !has_perm(&g, P0, "Ramosian Sergeant"),
        "the Rebel the person named"
    );
}
