//! Practice mode, simplifications removed (2026-11-04, part 2): Urza's {5} and Triple Triad give a permission for the
//! turn, Mirage Mirror copies lands with abilities, Sylvan Library offers the cards really drawn this turn, the Ur-Dragon
//! loops ask once for a batch of tokens, and (AI-affecting) Ragavan's exiled card and Goldspan Dragon's targeting.

mod play_common;
use play_common::*;
use sim_core::ids::CardId;
use sim_core::state::{Ctx, Step};
use std::sync::{Arc, Mutex};

/// a person who answers `rule`'s choice when it returns one (the first choice, or yes, otherwise); at a priority, the
/// scripted actions in order, then pass
fn person(actions: Vec<Value>, rule: impl Fn(&Request) -> Option<usize> + Send + Sync + 'static) -> Arc<ScriptHuman> {
    let acts = Mutex::new(actions.into_iter());
    Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(acts.lock().unwrap().next().unwrap_or(json!({"do": "pass"})));
        }
        if let Some(k) = rule(r) {
            return Answer::Pick(k);
        }
        if r.choices.is_empty() { Answer::Yes } else { Answer::Pick(0) }
    }))
}

fn with(r: &Request, word: &str) -> Option<usize> {
    r.choices.iter().position(|c| c.contains(word))
}

#[allow(dead_code)]
fn asked(h: &ScriptHuman, text: &str) -> bool {
    h.asked().iter().any(|r| r.prompt.contains(text))
}

/// the library is exactly `names`, the last one on top
fn library(g: &mut Game, p: PlayerId, names: &[&str]) -> Vec<CardId> {
    let cs: Vec<CardId> = names.iter().map(|n| take(g, p, n)).collect();
    g.player_mut(p).library = cs.clone();
    cs
}

fn in_exile(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).exile.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn main_phase(g: &mut Game) {
    g.active = Some(P0);
    g.step = Step::Main1;
}

fn from_exile(g: &mut Game, c: CardId) -> Option<String> {
    sim_core::human::human::apply(g, P0, &json!({"do": "cast", "zone": "ex", "card": c.0})).unwrap()
}

// ------------------------------------------------------------------ 1. Urza's {5} and Triple Triad: until end of turn
/// seat 0 is a person with Urza; its {5} is paid and the top card (the last of `lib`) is exiled with a permission
fn urza_top(
    lib: &[&str],
    answers: Vec<Value>,
    rule: impl Fn(&Request) -> Option<usize> + Send + Sync + 'static,
) -> (Game, Vec<CardId>) {
    let mut g = table(&["urdragon", "seph"]);
    let u = perm(&mut g, P0, "Urza, Lord High Artificer");
    let cs = library(&mut g, P0, lib);
    g.humans.add(P0, person(answers, rule));
    // (one card: the shuffle can't move it)
    sim_core::impls::t5::urza_top(&mut g, u, P0).unwrap();
    (g, cs)
}

#[test]
fn urzas_land_is_played_as_the_land_drop_in_the_main_phase() {
    let (mut g, cs) = urza_top(&["Island"], vec![], |_| None);
    let isle = cs[0];
    assert!(g.player(P0).free_play.contains(&isle), "a permission, not played at once");
    assert_eq!(g.player(P0).lands_played, 0);
    // not in the opponent's turn: a land is a main-phase land drop
    g.active = Some(P1);
    g.step = Step::Main1;
    let r = sim_core::human::human::apply(&mut g, P0, &json!({"do": "land", "zone": "ex", "card": isle.0})).unwrap();
    assert!(r.is_some(), "not your turn: {r:?}");
    main_phase(&mut g);
    let r = sim_core::human::human::apply(&mut g, P0, &json!({"do": "land", "zone": "ex", "card": isle.0})).unwrap();
    assert_eq!(r, None);
    assert_eq!(g.player(P0).lands_played, 1, "it was the land drop");
    assert!(g.player(P0).lands.iter().any(|&l| &*g.perm(l).name == "Island"));
    assert!(!in_exile(&g, P0, "Island"));
}

#[test]
fn urzas_land_needs_a_land_drop_and_is_still_offered_after_a_failed_try() {
    let (mut g, cs) = urza_top(&["Island"], vec![], |_| None);
    main_phase(&mut g);
    g.player_mut(P0).lands_played = 1;
    let r = sim_core::human::human::apply(&mut g, P0, &json!({"do": "land", "zone": "ex", "card": cs[0].0})).unwrap();
    assert!(r.is_some_and(|w| w.contains("already played a land")));
    assert!(in_exile(&g, P0, "Island"), "still in exile, still permitted");
    assert!(g.player(P0).free_play.contains(&cs[0]));
}

#[test]
fn urzas_aura_like_animate_dead_is_cast_in_the_main_phase_and_not_lost() {
    let (mut g, cs) = urza_top(&["Animate Dead"], vec![], |r| with(r, "Grave Titan"));
    let titan = take(&mut g, P1, "Grave Titan");
    g.player_mut(P1).gy.push(titan);
    main_phase(&mut g);
    assert!(g.player(P0).free_play.contains(&cs[0]), "kept for the main phase (it was lost before)");
    assert_eq!(from_exile(&mut g, cs[0]), None);
    assert!(has_perm(&g, P0, "Grave Titan"), "reanimated");
    assert!(in_gy(&g, P0, "Animate Dead"), "and the spell goes to its owner's graveyard");
}

#[test]
fn urzas_counterspell_is_held_for_a_spell_and_the_permission_ends_with_the_turn() {
    // the person answers the opponent's Grave Titan with the free Counterspell from exile
    let mut g = table(&["urdragon", "seph"]);
    let u = perm(&mut g, P0, "Urza, Lord High Artificer");
    let ctr = library(&mut g, P0, &["Counterspell"])[0];
    let acts = vec![json!({"do": "cast", "zone": "ex", "card": ctr.0})];
    let h = person(acts, |_| None);
    g.humans.add(P0, h.clone());
    sim_core::impls::t5::urza_top(&mut g, u, P0).unwrap();
    assert!(in_exile(&g, P0, "Counterspell"), "not wasted at once: nothing to counter now");
    g.active = Some(P1);
    let titan = hand(&mut g, P1, &["Grave Titan"])[0];
    let resolved = sim_core::engine::cast::cast_card(&mut g, P1, titan, "hand", Ctx::default()).unwrap_or(true);
    assert!(!resolved, "countered: {:?}", h.told());
    assert!(!has_perm(&g, P1, "Grave Titan"));
    assert!(in_gy(&g, P0, "Counterspell"), "the spell is in its owner's graveyard");
    assert!(!in_exile(&g, P0, "Counterspell"));
}

#[test]
fn urzas_permission_ends_at_the_end_of_the_turn() {
    let (mut g, cs) = urza_top(&["Sol Ring"], vec![], |_| None);
    main_phase(&mut g);
    assert!(sim_core::human::legal::check_cast_exile(&g, P0, cs[0]).is_none());
    sim_core::engine::turn::end_step(&mut g, P0).unwrap();
    assert!(g.player(P0).free_play.is_empty());
    assert!(sim_core::human::legal::check_cast_exile(&g, P0, cs[0]).is_some());
    assert!(in_exile(&g, P0, "Sol Ring"), "unplayed, it stays in exile");
}

// ------------------------------------------------------------------ 2. Mirage Mirror copies lands with abilities
use sim_core::human::human::{AbilityKind, abilities_of, apply};

/// run the ability of m whose label contains `word`; the reason it can't be done, if any
fn use_ability(g: &mut Game, p: PlayerId, m: PermId, word: &str) -> Option<String> {
    let list = abilities_of(g, p, m).unwrap();
    let Some((_, kind)) = list.into_iter().find(|(l, _)| l.contains(word)) else {
        panic!("no ability with {word:?} on {}", g.perm(m).name)
    };
    let AbilityKind::Extra(act) = kind else { panic!("not an extra ability") };
    act(g, p, m).unwrap()
}

#[test]
fn mirage_mirror_copies_a_fetch_land_and_cracking_it_sends_the_mirror_to_the_graveyard() {
    let mut g = table(&["alela", "seph"]);
    let mm = perm(&mut g, P0, "Mirage Mirror");
    let mirror_card = g.perm(mm).card();
    let fetch = lands(&mut g, P0, "Verdant Catacombs", 1, false)[0];
    let fetch_card = g.perm(fetch).card();
    library(&mut g, P0, &["Swamp"]);
    pool(&mut g, P0, &[('C', 2)]);
    let h = person(vec![], |r| if r.prompt.contains("Mirage Mirror") { with(r, "Verdant Catacombs") } else { None });
    g.humans.add(P0, h.clone());
    assert_eq!(use_ability(&mut g, P0, mm, "becomes a copy"), None, "a fetch land is a legal choice now");
    assert_eq!(g.perm(mm).name, "Verdant Catacombs");
    let i = g.player(P0).lands.iter().position(|&l| l == mm).expect("the Mirror is in the land list");
    // the copy's ability works: pay 1 life, sacrifice it, search
    main_phase(&mut g);
    let life = g.player(P0).life;
    assert_eq!(apply(&mut g, P0, &json!({"do": "use", "land": i})).unwrap(), None, "{:?}", h.told());
    assert_eq!(g.player(P0).life, life - 1);
    assert!(!g.perm(mm).on_bf, "sacrificed");
    assert!(g.player(P0).gy.contains(&mirror_card), "the Mirage Mirror card is in the graveyard");
    assert!(!g.player(P0).gy.contains(&fetch_card), "not a second Verdant Catacombs");
    assert!(g.perm(fetch).on_bf, "the real fetch land is untouched");
    assert!(g.player(P0).lands.iter().any(|&l| &*g.perm(l).name == "Swamp"), "and it found a Swamp");
}

#[test]
fn mirage_mirror_copies_your_own_legendary_land_and_the_legend_rule_has_you_keep_one() {
    let mut g = table(&["alela", "seph"]);
    let mm = perm(&mut g, P0, "Mirage Mirror");
    let mirror_card = g.perm(mm).card();
    let cradle = lands(&mut g, P0, "Gaea's Cradle", 1, false)[0];
    pool(&mut g, P0, &[('C', 2)]);
    // keep the Mirror (the second choice listed is not assumed: pick by the label's text)
    let h = person(vec![], |r| {
        if r.prompt.contains("Mirage Mirror") {
            with(r, "Gaea's Cradle")
        } else if r.prompt.contains("legend rule") {
            Some(r.choices.len() - 1)
        } else {
            None
        }
    });
    g.humans.add(P0, h.clone());
    assert_eq!(use_ability(&mut g, P0, mm, "becomes a copy"), None);
    assert!(asked(&h, "legend rule"), "two Gaea's Cradle: {:?}", h.asked());
    let cradles =
        g.player(P0).lands.iter().filter(|&&l| g.perm(l).on_bf && &*g.perm(l).name == "Gaea's Cradle").count();
    assert_eq!(cradles, 1, "you keep one");
    let gone = if g.perm(cradle).on_bf { mm } else { cradle };
    assert!(!g.perm(gone).on_bf);
    if gone == mm {
        assert!(in_gy(&g, P0, "Mirage Mirror"), "the Mirror, not a second Cradle, is in the graveyard");
        assert!(g.player(P0).gy.contains(&mirror_card));
    }
    assert!(g.player(P0).lands.iter().all(|&l| g.perm(l).on_bf), "no dead permanent left in the land list");
}

// ------------------------------------------------------------------ 3. Sylvan Library: the cards really drawn this turn
#[test]
fn sylvan_library_offers_the_cards_drawn_this_turn_not_the_last_cards_in_hand() {
    let mut g = table(&["thrasios", "seph"]);
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    library(&mut g, P0, &["Mind Stone", "Arcane Signet", "Thran Dynamo"]); // Thran Dynamo on top
    // the draw step's card (no Sylvan Library out yet), then a tutored card, then Sylvan Library's two extra draws
    sim_core::engine::zones::draw(&mut g, P0, 1, true).unwrap();
    let ring = take(&mut g, P0, "Sol Ring");
    g.player_mut(P0).hand.push(ring); // put into the hand, not drawn
    let h = person(vec![], |_| None);
    g.humans.add(P0, h.clone());
    sim_core::human::triggers_t1::sylvan_library(&mut g, P0).unwrap();
    let req = h.asked().into_iter().find(|r| r.prompt.contains("choose two cards")).expect("asked which cards");
    let all = req.choices.join(" | ");
    assert!(all.contains("Thran Dynamo"), "the draw step's card is offered: {all}");
    assert!(all.contains("Arcane Signet") && all.contains("Mind Stone"), "and the two extra cards: {all}");
    assert!(!all.contains("Sol Ring"), "a card that was not drawn is not offered: {all}");
}

#[test]
fn drawn_cards_are_forgotten_at_the_next_turn() {
    let mut g = table(&["thrasios", "seph"]);
    g.active = Some(P0);
    library(&mut g, P0, &["Mind Stone", "Arcane Signet"]);
    sim_core::engine::zones::draw(&mut g, P0, 2, false).unwrap();
    assert_eq!(sim_core::human::triggers_t1::drawn_in_hand(&g, P0).len(), 2);
    g.round += 1; // a new turn
    assert!(sim_core::human::triggers_t1::drawn_in_hand(&g, P0).is_empty(), "nothing drawn yet this turn");
    library(&mut g, P0, &["Sol Ring"]);
    sim_core::engine::zones::draw(&mut g, P0, 1, true).unwrap();
    assert_eq!(sim_core::human::triggers_t1::drawn_in_hand(&g, P0), vec![card(&g, "Sol Ring")]);
}

// ------------------------------------------------------------------ 4a. Loop shortcuts: one question for a batch of tokens
fn count_asked(h: &ScriptHuman, text: &str) -> usize {
    h.asked().iter().filter(|r| r.prompt.contains(text)).count()
}

#[test]
fn terror_of_the_peaks_asks_once_for_a_batch_of_tokens_when_the_target_is_the_same() {
    let mut g = table(&["urdragon", "seph"]);
    perm(&mut g, P0, "Terror of the Peaks");
    let h = person(vec![], |r| {
        if r.prompt.contains("which target for all") || r.prompt.contains("damage to which target") {
            r.choices.iter().position(|c| c.contains("Sephiroth") || c.contains(&*"Seph"))
        } else {
            None // "yes, the same target" is the first choice
        }
    });
    g.humans.add(P0, h.clone());
    let life = g.player(P1).life;
    sim_core::engine::zones::make_tokens(&mut g, P0, sim_core::engine::zones::Tokens::new(4, 2)).unwrap();
    assert_eq!(count_asked(&h, "Aim all of them at the same target"), 1, "{:?}", h.asked());
    assert_eq!(count_asked(&h, "damage to which target"), 0, "no per-token question");
    assert_eq!(g.player(P1).life, life - 4 * 2, "all four triggers hit the chosen player");
}

#[test]
fn terror_of_the_peaks_still_asks_per_token_when_the_person_wants_each_target() {
    let mut g = table(&["urdragon", "seph"]);
    perm(&mut g, P0, "Terror of the Peaks");
    let h = person(vec![], |r| {
        if r.prompt.contains("Aim all of them") {
            with(r, "I'll choose each")
        } else if r.prompt.contains("damage to which target") {
            r.choices.iter().position(|c| c.contains("Seph"))
        } else {
            None
        }
    });
    g.humans.add(P0, h.clone());
    sim_core::engine::zones::make_tokens(&mut g, P0, sim_core::engine::zones::Tokens::new(3, 2)).unwrap();
    assert_eq!(count_asked(&h, "Aim all of them at the same target"), 1);
    assert_eq!(count_asked(&h, "damage to which target"), 3, "one question per token: {:?}", h.asked());
}

#[test]
fn a_single_token_is_not_asked_whether_to_use_the_same_target() {
    let mut g = table(&["urdragon", "seph"]);
    perm(&mut g, P0, "Terror of the Peaks");
    let h = person(vec![], |r| r.prompt.contains("damage to which target").then(|| with(r, "Seph")).flatten());
    g.humans.add(P0, h.clone());
    sim_core::engine::zones::make_tokens(&mut g, P0, sim_core::engine::zones::Tokens::new(1, 2)).unwrap();
    assert_eq!(count_asked(&h, "Aim all of them"), 0);
    assert_eq!(count_asked(&h, "damage to which target"), 1);
}

#[test]
fn the_mirror_loop_asks_once_for_the_terror_of_the_peaks_target() {
    let mut g = table_with(&["urdragon", "seph"], 1, 40);
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    let ur = card(&g, "The Ur-Dragon");
    g.player_mut(P0).set_cmd_zone(ur, false);
    let mirror = perm(&mut g, P0, "Cursed Mirror");
    perm(&mut g, P0, "Terror of the Peaks");
    hand(&mut g, P0, &["Astral Dragon"]);
    let mana = g.player_mut(P0);
    mana.pool.add('U', 2);
    mana.pool.add('R', 6);
    let h = person(vec![], |r| {
        if r.prompt.contains("which ability") {
            with(r, "start the loop")
        } else if r.prompt.contains("the opponents may answer") {
            with(r, "once around")
        } else if r.prompt.contains("which target for all") {
            with(r, "Seph")
        } else {
            None
        }
    });
    g.humans.add(P0, h.clone());
    let i = g.player(P0).perms.iter().position(|&m| m == mirror).unwrap();
    let life = g.player(P1).life;
    sim_core::human::human::apply(&mut g, P0, &json!({"do": "use", "perm": i})).unwrap();
    assert!(count_asked(&h, "start the loop") + count_asked(&h, "which ability") > 0, "the loop was started");
    assert!(count_asked(&h, "Aim all of them at the same target") == 1, "{:?}", h.asked());
    assert_eq!(count_asked(&h, "damage to which target"), 0, "no per-token target question: {:?}", h.asked());
    assert!(g.player(P1).life < life, "Terror's triggers hit the chosen player ({} -> {})", life, g.player(P1).life);
}

// ------------------------------------------------------------------ 4b. The Garth loop: Black Lotus and Braingeyser
fn garth_table() -> (Game, PermId) {
    use sim_core::engine::zones::{self, Enter};
    let mut g = table_with(&["urdragon", "seph"], 1, 40);
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    let ur = card(&g, "The Ur-Dragon");
    g.player_mut(P0).set_cmd_zone(ur, false);
    let garth = perm(&mut g, P0, "Garth One-Eye");
    let sd = take(&mut g, P0, "Spark Double");
    zones::enter(&mut g, P0, sd, Enter { plus: 1, ..Enter::default() }).unwrap();
    perm(&mut g, P0, "Doubling Season");
    hand(&mut g, P0, &["Twinflame"]);
    pool(&mut g, P0, &[('R', 6), ('G', 1)]);
    (g, garth)
}

fn garth_person(tap: &'static str, brain: &'static str) -> Arc<ScriptHuman> {
    person(vec![], move |r| {
        if r.prompt.contains("which ability") {
            with(r, "start the loop")
        } else if r.prompt.contains("the opponents may answer") {
            with(r, "once around")
        } else if r.prompt.contains("How many tap instead") {
            with(r, tap)
        } else if r.prompt.contains("one of the") && r.prompt.contains("Braingeyser") {
            with(r, brain)
        } else if r.prompt.contains("draws X cards") {
            with(r, "X = 4")
        } else if r.prompt.contains("which colour") {
            with(r, "{U}")
        } else {
            None
        }
    })
}

#[test]
fn the_garth_loop_plays_out_black_lotus_and_braingeyser_for_the_garths_that_tap() {
    let (mut g, garth) = garth_table();
    let h = garth_person("none of the new Garths attack", "you draw");
    g.humans.add(P0, h.clone());
    let i = g.player(P0).perms.iter().position(|&m| m == garth).unwrap();
    let (hand0, lib0) = (g.player(P0).hand.len(), g.player(P0).library.len());
    let pool0 = pool_total(&g, P0);
    sim_core::human::human::apply(&mut g, P0, &json!({"do": "use", "perm": i})).unwrap();
    assert!(asked(&h, "How many tap instead"), "{:?}", h.asked());
    assert!(asked(&h, "Braingeyser"), "{:?}", h.asked());
    // the Twinflame / Regrowth steps use mana and the hand: compare the cards drawn by Braingeyser (X = 4)
    assert_eq!(g.player(P0).library.len(), lib0 - 4, "Braingeyser drew four cards");
    assert!(g.player(P0).hand.len() >= hand0 + 4 - 1, "the cards are in the hand");
    assert!(pool_total(&g, P0) > pool0, "Black Lotus mana is in the pool: {}", pool_text(&g, P0));
    assert!(pool_text(&g, P0).contains('U'), "in the chosen colour: {}", pool_text(&g, P0));
    let tapped = g.player(P0).perms.iter().filter(|&&m| g.perm(m).token && g.perm(m).tapped).count();
    assert!(tapped >= 2, "the Garths that made the Lotuses and the Braingeyser are tapped ({tapped})");
}

#[test]
fn the_garth_loop_leaves_every_new_garth_to_attack_when_the_person_says_none() {
    let (mut g, garth) = garth_table();
    let h = garth_person("none: all", "no Braingeyser");
    g.humans.add(P0, h.clone());
    let i = g.player(P0).perms.iter().position(|&m| m == garth).unwrap();
    let lib0 = g.player(P0).library.len();
    sim_core::human::human::apply(&mut g, P0, &json!({"do": "use", "perm": i})).unwrap();
    assert!(asked(&h, "How many tap instead"));
    assert_eq!(g.player(P0).library.len(), lib0, "no Braingeyser");
    assert!(!pool_text(&g, P0).contains('U'));
    let tapped = g.player(P0).perms.iter().filter(|&&m| g.perm(m).token && g.perm(m).tapped).count();
    assert_eq!(tapped, 1, "only the Garth that chose Regrowth in the loop's steps is tapped");
}
