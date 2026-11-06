//! Phase C2 (ai-rules, 2026-11-05): AI paths that did not follow the Oracle text now do. Each test builds a small
//! position, lets the AI's code act and checks the result against the card's Oracle text.
//!
//! Council's Judgment, Liliana of the Veil -6, Liliana Dreadhorde General -9, Chain of Vapor's copy chain, Maelstrom of
//! the Spirit Dragon, Loran of the Third Path, Ramosian Revivalist and Forensic Gadgeteer's discount.

mod play_common;
use play_common::*;
use sim_core::engine::cast::resolve;
use sim_core::hooks::{Action, Opt};
use sim_core::state::Ctx;
use std::sync::Arc;

/// run an option the way the AI would
fn run(g: &mut Game, o: Opt, p: PlayerId) -> bool {
    match o.act.expect("an action") {
        Action::Plan { f, arg } => f(g, p, arg).unwrap(),
        Action::Ability { src, f, arg } => f(g, src, p, arg).unwrap(),
        other => panic!("unexpected action {other:?}"),
    }
}

fn on(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

/// the option slot of a battlefield card's code
fn options_of(g: &mut Game, m: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(m).cd.unwrap();
    let f = g.registry.get(c).and_then(|i| i.options).expect("options");
    f(g, m, p, post).unwrap()
}

/// the option slot of a land's code
fn land_options_of(g: &mut Game, l: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(l).card();
    let f = g.registry.get(c).and_then(|i| i.land_options).expect("land_options");
    f(g, l, p, post).unwrap()
}

fn cast_resolve(g: &mut Game, p: PlayerId, name: &str, target: Option<PermId>) {
    let c = take(g, p, name);
    resolve(g, p, c, &Ctx { target, ..Ctx::default() }, "hand".into()).unwrap();
}

// ================================================================== 1. Council's Judgment
#[test]
fn judgment_an_ai_opponent_may_vote_for_its_own_permanent() {
    // two players: the opponent's only legal votes are its own permanents (the caster's are not legal), so it votes
    // for one of them (the one it minds losing least); the caster votes for the best of them: a tie, both are exiled
    let mut g = table(&["kefka", "seph"]);
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P1, "Doubling Season");
    perm(&mut g, P1, "Mind Stone");
    cast_resolve(&mut g, P0, "Council's Judgment", None);
    assert!(on(&g, P0, "Sol Ring"), "the caster's own permanent can't be voted for");
    assert!(!on(&g, P1, "Doubling Season"), "the caster's vote");
    assert!(!on(&g, P1, "Mind Stone"), "the opponent's vote for its own permanent: tied for the most, so exiled too");
}

#[test]
fn judgment_an_ai_opponent_votes_for_another_players_permanent_when_it_can() {
    // three players: P1 and P2 each have a permanent; each votes for the other's rather than its own; the caster's
    // vote is for the better one; that one has two votes and is the only one exiled
    let mut g = table(&["kefka", "seph", "cloud"]);
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P1, "Mind Stone");
    perm(&mut g, P2, "Doubling Season");
    cast_resolve(&mut g, P0, "Council's Judgment", None);
    assert!(!on(&g, P2, "Doubling Season"), "the better permanent: the caster's vote and P1's");
    assert!(on(&g, P1, "Mind Stone"), "one vote only (P2's) against two");
    assert!(on(&g, P0, "Sol Ring"));
}

#[test]
fn judgment_only_the_permanent_with_the_most_votes_is_exiled() {
    // Mind Stone gets the caster's vote and P2's (the only other-player candidate for each); Sol Ring gets P1's
    let mut g = table(&["kefka", "seph", "cloud"]);
    perm(&mut g, P1, "Mind Stone");
    perm(&mut g, P2, "Sol Ring");
    cast_resolve(&mut g, P0, "Council's Judgment", None);
    assert!(!on(&g, P1, "Mind Stone"), "two votes");
    assert!(on(&g, P2, "Sol Ring"), "one vote");
}

// ================================================================== 2. Liliana of the Veil -6
fn lotv(g: &mut Game) {
    let src = perm(g, P0, "Liliana of the Veil");
    (sim_core::impls::rules::LOTV_ULT.eff)(g, P0, src).unwrap();
}

#[test]
fn liliana_of_the_veil_the_piles_hold_the_lands_too() {
    // six Swamps and a Sol Ring: the piles are split by worth, so one pile holds about half the lands, and the
    // opponent loses one pile, lands included
    let mut g = table(&["kefka", "seph"]);
    lands(&mut g, P1, "Swamp", 6, false);
    perm(&mut g, P1, "Sol Ring");
    lotv(&mut g);
    let left = g.player(P1).lands.len();
    assert!(left < 6, "a pile of lands was sacrificed ({left} left)");
    assert!(left >= 2, "the chooser kept the better pile ({left} left)");
    assert!(on(&g, P1, "Sol Ring") || left <= 3, "Sol Ring went only with the pile that lost the lands");
}

#[test]
fn liliana_of_the_veil_the_ai_chooser_keeps_the_pile_worth_more() {
    // one big permanent and four lands: the caster splits so that no pile has both; the chooser sacrifices the
    // pile of lands and keeps the big permanent, or sacrifices the big permanent with some lands; it never keeps
    // the poorer pile
    let mut g = table(&["kefka", "seph"]);
    lands(&mut g, P1, "Swamp", 4, false);
    perm(&mut g, P1, "Doubling Season");
    perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Mind Stone");
    let before = 4 + 3;
    lotv(&mut g);
    let after = g.player(P1).lands.len() + g.player(P1).perms.len();
    assert!(after < before && after >= 2, "one pile sacrificed, one kept: {after} of {before}");
}

#[test]
fn liliana_of_the_veil_the_person_sees_the_lands_in_the_piles() {
    let h = Arc::new(ScriptHuman::from_fn(|r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        Answer::Pick(0)
    }));
    let mut g = table(&["kefka", "seph"]);
    g.humans.add(P0, h.clone());
    lands(&mut g, P0, "Swamp", 4, false);
    perm(&mut g, P0, "Sol Ring");
    g.player_mut(P0).treasures = 2;
    let src = perm(&mut g, P1, "Liliana of the Veil");
    (sim_core::impls::rules::LOTV_ULT.eff)(&mut g, P1, src).unwrap();
    let r = h.asked().into_iter().find(|r| r.prompt.contains("Liliana of the Veil")).expect("the person was asked");
    assert_eq!(r.choices.len(), 2);
    let all = r.choices.join(" | ");
    assert!(all.contains("Swamp"), "lands are in the piles: {all}");
    assert!(all.contains("Treasure token"), "tokens are permanents too: {all}");
    assert!(all.contains("Sol Ring"));
    // the person chose pile 1 to sacrifice: what it lists is gone
    assert_eq!(r.choices[0].contains("Sol Ring"), !on(&g, P0, "Sol Ring"), "{all}");
    assert_eq!(r.choices[0].contains("Treasure"), g.player(P0).treasures < 2, "{all}");
    assert_eq!(r.choices[0].contains("Swamp"), g.player(P0).lands.len() < 4, "{all}");
}

// ================================================================== 3. Liliana, Dreadhorde General -9
fn ldg(g: &mut Game) {
    let src = perm(g, P0, "Liliana, Dreadhorde General");
    (sim_core::impls::rules::LDG_ULT.eff)(g, P0, src).unwrap();
}

#[test]
fn liliana_dreadhorde_an_artifact_creature_counts_as_both_and_the_ai_keeps_the_best_set() {
    // Ornithopter is an artifact creature and Birds of Paradise a creature: the chooser keeps Ornithopter for the
    // artifact and Birds for the creature (one permanent per type, a permanent may serve two), not just one creature
    let mut g = table(&["kefka", "seph"]);
    perm(&mut g, P1, "Ornithopter");
    perm(&mut g, P1, "Birds of Paradise");
    perm(&mut g, P1, "Llanowar Elves");
    lands(&mut g, P1, "Swamp", 3, false);
    ldg(&mut g);
    assert!(on(&g, P1, "Ornithopter"), "kept as the artifact");
    let creatures = ["Birds of Paradise", "Llanowar Elves"].iter().filter(|n| on(&g, P1, n)).count();
    assert_eq!(creatures, 1, "and one other creature, the best");
    assert_eq!(g.player(P1).lands.len(), 1, "one land");
}

#[test]
fn liliana_dreadhorde_one_permanent_may_serve_two_types_and_the_rest_go() {
    let mut g = table(&["kefka", "seph"]);
    perm(&mut g, P1, "Ornithopter"); // artifact creature: the only creature and an artifact
    perm(&mut g, P1, "Sol Ring"); // artifact
    perm(&mut g, P1, "Mind Stone"); // artifact
    perm(&mut g, P1, "Doubling Season"); // enchantment
    ldg(&mut g);
    assert!(on(&g, P1, "Ornithopter"), "the only creature");
    assert!(on(&g, P1, "Doubling Season"), "the only enchantment");
    let arts = ["Sol Ring", "Mind Stone"].iter().filter(|n| on(&g, P1, n)).count();
    assert_eq!(arts, 1, "one artifact more: Ornithopter is kept as the creature, the artifact slot takes another");
    assert!(on(&g, P1, "Mind Stone"), "the most valuable artifact");
}

#[test]
fn liliana_dreadhorde_a_lone_artifact_creature_covers_both_types() {
    let mut g = table(&["kefka", "seph"]);
    perm(&mut g, P1, "Ornithopter");
    lands(&mut g, P1, "Swamp", 2, false);
    ldg(&mut g);
    assert!(on(&g, P1, "Ornithopter"), "kept for both types");
    assert_eq!(g.player(P1).lands.len(), 1);
}

// ================================================================== 4. Chain of Vapor
fn chain_script(land_word: &'static str, target_word: &'static str, copy: bool) -> Arc<ScriptHuman> {
    Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        if r.prompt.contains("sacrifice a land to copy") {
            // (a yes/no question: the choices are yes, no)
            let k = r.choices.iter().position(|c| c.to_lowercase().contains(if copy { "yes" } else { "no" }));
            return k.map_or(if copy { Answer::Yes } else { Answer::No }, Answer::Pick);
        }
        if r.prompt.contains("which land") {
            return Answer::Pick(r.choices.iter().position(|c| c.contains(land_word)).unwrap_or(0));
        }
        if r.prompt.contains("returns which") {
            return Answer::Pick(r.choices.iter().position(|c| c.contains(target_word)).unwrap_or(0));
        }
        Answer::Pick(0)
    }))
}

#[test]
fn chain_of_vapor_the_copys_new_target_is_offered_a_copy_too() {
    // P0 (AI) bounces P1's Sol Ring; P1 copies it at P2's Mind Stone; P2 is offered a copy of its own and sacrifices
    // a land to copy it at P0's Birds of Paradise: the chain goes round
    let mut g = table(&["kefka", "seph", "cloud"]);
    let h1 = chain_script("Forest", "Mind Stone", true);
    let h2 = chain_script("Island", "Birds", true);
    g.humans.add(P1, h1.clone());
    g.humans.add(P2, h2.clone());
    lands(&mut g, P0, "Island", 3, false);
    lands(&mut g, P1, "Forest", 2, false);
    lands(&mut g, P2, "Island", 2, false);
    let sol = perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P2, "Mind Stone");
    perm(&mut g, P0, "Birds of Paradise");
    cast_resolve(&mut g, P0, "Chain of Vapor", Some(sol));
    assert!(in_hand(&g, P1, "Sol Ring"), "the original bounce");
    assert!(in_hand(&g, P2, "Mind Stone"), "P1's copy");
    assert!(h2.asked().iter().any(|r| r.prompt.contains("sacrifice a land to copy")), "P2 was offered a copy");
    assert_eq!(g.player(P2).lands.len(), 1, "P2 sacrificed a land");
    assert!(in_hand(&g, P0, "Birds of Paradise"), "P2's copy came back to the caster");
}

#[test]
fn chain_of_vapor_the_chain_ends_when_the_next_player_declines() {
    let mut g = table(&["kefka", "seph", "cloud"]);
    let h1 = chain_script("Forest", "Mind Stone", true);
    let h2 = chain_script("Island", "Birds", false);
    g.humans.add(P1, h1.clone());
    g.humans.add(P2, h2.clone());
    lands(&mut g, P1, "Forest", 2, false);
    lands(&mut g, P2, "Island", 2, false);
    let sol = perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P2, "Mind Stone");
    perm(&mut g, P0, "Birds of Paradise");
    cast_resolve(&mut g, P0, "Chain of Vapor", Some(sol));
    assert!(h2.asked().iter().any(|r| r.prompt.contains("sacrifice a land to copy")));
    assert_eq!(g.player(P2).lands.len(), 2, "P2 declined: no land sacrificed");
    assert!(on(&g, P0, "Birds of Paradise"));
}

#[test]
fn chain_of_vapor_an_ai_copies_at_a_key_permanent_and_the_chain_continues_to_the_next_ai() {
    // P0 bounces P1's Sol Ring. P1 (AI, six lands) copies at P0's best permanent (value 5 or more); P0 (AI, six
    // lands left) is then offered a copy and answers at P1's best permanent
    let mut g = table(&["kefka", "seph"]);
    lands(&mut g, P0, "Island", 6, false);
    lands(&mut g, P1, "Swamp", 6, false);
    let sol = perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P1, "Sheoldred, the Apocalypse");
    perm(&mut g, P0, "Rhystic Study");
    cast_resolve(&mut g, P0, "Chain of Vapor", Some(sol));
    assert!(in_hand(&g, P1, "Sol Ring"));
    assert!(g.player(P1).lands.len() < 6, "P1 sacrificed a land to copy");
    assert!(in_hand(&g, P0, "Rhystic Study"), "P1's copy returned P0's key permanent");
    assert!(g.player(P0).lands.len() < 6, "P0 was offered a copy of the copy and took it");
    assert!(in_hand(&g, P1, "Sheoldred, the Apocalypse"), "and returned P1's key permanent");
}

#[test]
fn chain_of_vapor_an_ai_with_few_lands_does_not_copy() {
    let mut g = table(&["kefka", "seph"]);
    lands(&mut g, P1, "Swamp", 4, false);
    let sol = perm(&mut g, P1, "Sol Ring");
    perm(&mut g, P0, "Rhystic Study");
    cast_resolve(&mut g, P0, "Chain of Vapor", Some(sol));
    assert_eq!(g.player(P1).lands.len(), 4, "no land to spare");
    assert!(on(&g, P0, "Rhystic Study"));
}

// ================================================================== 5. Maelstrom of the Spirit Dragon
fn dragon_table() -> Game {
    let mut g = table(&["ur-dragon-five-color-dragons", "urza-mono-blue-artifacts"]);
    g.player_mut(P0).hand.clear();
    g
}

#[test]
fn maelstrom_the_ai_fetches_a_dragon_at_the_end_of_the_turn_before_its_own() {
    let mut g = dragon_table();
    let m = lands(&mut g, P0, "Maelstrom of the Spirit Dragon", 1, false)[0];
    lands(&mut g, P0, "Mountain", 6, false);
    let dragon = take(&mut g, P0, "Goldspan Dragon");
    g.player_mut(P0).library = vec![dragon];
    let o = land_options_of(&mut g, m, P0, None);
    assert_eq!(o.len(), 1, "an option at the end-of-turn window");
    assert!(run(&mut g, o.into_iter().next().unwrap(), P0));
    assert!(in_hand(&g, P0, "Goldspan Dragon"), "the Dragon is in hand");
    assert!(!g.player(P0).lands.contains(&m), "Maelstrom was sacrificed");
    assert_eq!(g.player(P0).lands.len(), 6);
    assert!(g.player(P0).lands.iter().filter(|&&l| g.perm(l).tapped).count() >= 4, "{{4}} was paid");
}

#[test]
fn maelstrom_the_ai_needs_a_land_to_spare_and_a_dragon_worth_fetching() {
    let mut g = dragon_table();
    let m = lands(&mut g, P0, "Maelstrom of the Spirit Dragon", 1, false)[0];
    lands(&mut g, P0, "Mountain", 4, false);
    let dragon = take(&mut g, P0, "Goldspan Dragon");
    g.player_mut(P0).library = vec![dragon];
    assert!(land_options_of(&mut g, m, P0, None).is_empty(), "five lands: none to spare");
    lands(&mut g, P0, "Mountain", 2, false);
    assert_eq!(land_options_of(&mut g, m, P0, None).len(), 1);
    assert!(land_options_of(&mut g, m, P0, Some(false)).is_empty(), "not before combat");
    g.player_mut(P0).library.clear();
    assert!(land_options_of(&mut g, m, P0, None).is_empty(), "no Dragon in the library");
}

#[test]
fn maelstrom_in_the_second_main_phase_only_with_plenty_of_lands() {
    let mut g = dragon_table();
    let m = lands(&mut g, P0, "Maelstrom of the Spirit Dragon", 1, false)[0];
    lands(&mut g, P0, "Mountain", 5, false);
    let dragon = take(&mut g, P0, "Goldspan Dragon");
    g.player_mut(P0).library = vec![dragon];
    assert!(land_options_of(&mut g, m, P0, Some(true)).is_empty(), "six lands: not in its own turn");
    lands(&mut g, P0, "Mountain", 1, false);
    let o = land_options_of(&mut g, m, P0, Some(true));
    assert_eq!(o.len(), 1, "seven lands, second main phase");
    assert!(o[0].utility < land_options_of(&mut g, m, P0, None)[0].utility, "a lower play than at the end of the turn");
}

// ================================================================== 6. Loran of the Third Path
#[test]
fn loran_the_ai_draws_with_the_opponent_a_card_helps_least_and_never_a_hexproof_one() {
    let mut g = table(&["kefka", "seph", "cloud"]);
    let loran = perm(&mut g, P0, "Loran of the Third Path");
    g.perm_mut(loran).sick = false;
    g.player_mut(P0).hand.clear();
    lands(&mut g, P0, "Plains", 6, false);
    lands(&mut g, P1, "Swamp", 6, false);
    lands(&mut g, P2, "Island", 6, false);
    let filler = card(&g, "Sol Ring");
    g.player_mut(P1).hand = vec![filler; 7]; // a full hand: one more is discarded
    g.player_mut(P2).hand.clear(); // an empty hand: an extra card is worth a lot
    let o = options_of(&mut g, loran, P0, None);
    assert_eq!(o.len(), 1);
    match o[0].act.as_ref().unwrap() {
        Action::Ability { arg, .. } => assert_eq!(*arg, P1.0 as i64, "the opponent with the full hand"),
        other => panic!("{other:?}"),
    }
    // only at the end-of-turn window: in its own turn Loran would stay tapped through every opponent's turn
    assert!(options_of(&mut g, loran, P0, Some(false)).is_empty());
    assert!(options_of(&mut g, loran, P0, Some(true)).is_empty());
}

#[test]
fn loran_the_ai_skips_a_draw_that_helps_the_opponent_more_than_itself() {
    let mut g = table(&["kefka", "seph"]);
    let loran = perm(&mut g, P0, "Loran of the Third Path");
    g.perm_mut(loran).sick = false;
    lands(&mut g, P1, "Swamp", 6, false);
    lands(&mut g, P0, "Plains", 2, false);
    let filler = card(&g, "Sol Ring");
    g.player_mut(P0).hand = vec![filler; 5];
    g.player_mut(P1).hand.clear();
    assert!(options_of(&mut g, loran, P0, None).is_empty(), "the opponent would gain more from the card");
    g.player_mut(P0).hand.clear();
    lands(&mut g, P0, "Plains", 4, false);
    assert_eq!(options_of(&mut g, loran, P0, None).len(), 1, "both with empty hands and mana");
}

#[test]
fn loran_not_with_a_library_about_to_run_out() {
    let mut g = table(&["kefka", "seph"]);
    let loran = perm(&mut g, P0, "Loran of the Third Path");
    g.perm_mut(loran).sick = false;
    lands(&mut g, P0, "Plains", 4, false);
    lands(&mut g, P1, "Swamp", 4, false);
    g.player_mut(P0).library.truncate(5);
    assert!(options_of(&mut g, loran, P0, None).is_empty());
}

// ================================================================== 7. Ramosian Revivalist
fn revivalist_table() -> (Game, PermId) {
    let mut g = table(&["galadriel", "seph"]);
    let rev = perm(&mut g, P0, "Ramosian Revivalist");
    g.perm_mut(rev).sick = false;
    lands(&mut g, P0, "Plains", 6, false);
    g.player_mut(P0).gy.clear();
    (g, rev)
}

#[test]
fn revivalist_the_ai_fixes_its_target_when_it_activates() {
    let (mut g, rev) = revivalist_table();
    let lt = card(&g, "Ramosian Lieutenant");
    g.player_mut(P0).gy.push(lt);
    let o = options_of(&mut g, rev, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert!(o[0].label.contains("Ramosian Lieutenant"), "{}", o[0].label);
    // a better Rebel arrives in the graveyard after the option was made: the ability still returns its target
    let cmd = card(&g, "Ramosian Commander");
    g.player_mut(P0).gy.push(cmd);
    assert!(run(&mut g, o.into_iter().next().unwrap(), P0));
    assert!(on(&g, P0, "Ramosian Lieutenant"), "the target chosen on activation came back");
    assert!(!on(&g, P0, "Ramosian Commander"));
    assert!(g.player(P0).gy.contains(&cmd));
    assert!(g.perm(rev).tapped);
}

#[test]
fn revivalist_cannot_be_activated_for_a_card_that_has_left_the_graveyard() {
    let (mut g, rev) = revivalist_table();
    let lt = card(&g, "Ramosian Lieutenant");
    g.player_mut(P0).gy.push(lt);
    let o = options_of(&mut g, rev, P0, Some(false));
    g.player_mut(P0).gy.clear(); // exiled in response to nothing: the target is gone before it was chosen
    assert!(!run(&mut g, o.into_iter().next().unwrap(), P0), "no target: nothing is paid, nothing happens");
    assert!(!g.perm(rev).tapped);
    assert!(!on(&g, P0, "Ramosian Lieutenant"));
}

// ================================================================== 8. Forensic Gadgeteer
#[test]
fn gadgeteer_the_ai_eats_a_food_for_one_mana() {
    let mut g = table(&["kefka", "seph"]);
    g.player_mut(P0).foods = 1;
    g.player_mut(P0).life = 10;
    lands(&mut g, P0, "Swamp", 1, false);
    assert!(sim_core::impls::common::food_options(&mut g, P0, Some(true)).unwrap().is_empty(), "{{2}} without it");
    perm(&mut g, P0, "Forensic Gadgeteer");
    let o = sim_core::impls::common::food_options(&mut g, P0, Some(true)).unwrap();
    assert_eq!(o.len(), 1, "{{1}} with it");
    assert!(run(&mut g, o.into_iter().next().unwrap(), P0));
    assert_eq!(g.player(P0).life, 13);
    assert_eq!(g.player(P0).foods, 0);
}

#[test]
fn gadgeteer_never_reduces_a_cost_below_one_mana() {
    let mut g = table(&["kefka", "seph"]);
    perm(&mut g, P0, "Forensic Gadgeteer");
    perm(&mut g, P0, "Forensic Gadgeteer");
    let art = sim_core::impls::t45_urza_yawgmoth::artifact_cost;
    assert_eq!(art(&g, P0, 2, 0), 1, "two Gadgeteers still leave one mana");
    assert_eq!(art(&g, P0, 1, 0), 1, "a Blood's {{1}} stays");
    assert_eq!(art(&g, P0, 0, 0), 0);
}

#[test]
fn gadgeteer_the_blood_rummage_is_priced_by_the_same_rule() {
    // the Blood token costs {1}: the Gadgeteer can't reduce it, so it still needs one mana
    let mut g = table(&["sheoldred", "seph"]);
    perm(&mut g, P0, "Forensic Gadgeteer");
    g.player_mut(P0).bloods = 1;
    let junk = card(&g, "Swamp");
    g.player_mut(P0).hand = vec![junk, junk];
    let seeker = perm(&mut g, P0, "Restless Bloodseeker // Bloodsoaked Reveler");
    let ls = lands(&mut g, P0, "Swamp", 7, true); // seven lands: a spare land card is worth little; all tapped
    let before = g.player(P0).hand.len();
    let o: Vec<Opt> = options_of(&mut g, seeker, P0, Some(true));
    assert!(o.iter().all(|x| !x.label.contains("rummage")), "no mana: no rummage");
    g.perm_mut(ls[0]).tapped = false;
    let o: Vec<Opt> = options_of(&mut g, seeker, P0, Some(true));
    let r = o.into_iter().find(|x| x.label.contains("rummage")).expect("one mana pays it");
    assert!(run(&mut g, r, P0));
    assert_eq!(g.player(P0).bloods, 0);
    assert_eq!(g.player(P0).hand.len(), before, "discard one, draw one");
}

#[test]
fn gadgeteer_equip_and_other_artifact_abilities_cost_one_less() {
    let mut g = table(&["kefka", "seph"]);
    let sword = perm(&mut g, P0, "Bonesplitter");
    let bird = perm(&mut g, P0, "Birds of Paradise");
    let base = sim_core::impls::cloud::equip_cost(&g, P0, sword, bird, 2);
    assert_eq!(base, 2);
    perm(&mut g, P0, "Forensic Gadgeteer");
    assert_eq!(sim_core::impls::cloud::equip_cost(&g, P0, sword, bird, 2), 1, "equip is an artifact's ability");
    assert_eq!(sim_core::impls::cloud::equip_cost(&g, P0, sword, bird, 1), 1, "not below one mana");
}

#[test]
fn gadgeteer_a_retrofitter_foundry_servo_costs_one() {
    let mut g = table(&["urza-mono-blue-artifacts", "seph"]);
    let f = perm(&mut g, P0, "Retrofitter Foundry");
    lands(&mut g, P0, "Island", 1, false);
    assert!(options_of(&mut g, f, P0, Some(false)).is_empty(), "a Servo costs {{2}}");
    perm(&mut g, P0, "Forensic Gadgeteer");
    let o = options_of(&mut g, f, P0, Some(false));
    assert_eq!(o.len(), 1, "{{1}} with it");
    assert!(run(&mut g, o.into_iter().next().unwrap(), P0));
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).token && g.perm(m).ttypes.contains(&"servo")));
}

#[test]
fn gadgeteer_a_compiled_artifact_ability_costs_one_less() {
    // Basalt Monolith's {3}: untap it, from the ability language: {2} with a Gadgeteer, two Islands pay it
    let mut g = table(&["urza-mono-blue-artifacts", "seph"]);
    let mono = perm(&mut g, P0, "Basalt Monolith");
    g.perm_mut(mono).tapped = true;
    lands(&mut g, P0, "Island", 2, false);
    let untap = |g: &mut Game| -> usize {
        sim_core::dsl::ability_options(g, P0, true)
            .unwrap()
            .iter()
            .filter(|o| o.label.contains("Basalt Monolith"))
            .count()
    };
    assert_eq!(untap(&mut g), 0, "{{3}} is more than two Islands");
    perm(&mut g, P0, "Forensic Gadgeteer");
    assert!(untap(&mut g) >= 1, "{{2}} with it");
}

// ================================================================== follow-up: Liliana's tokens for a person, the sweep
#[test]
fn liliana_dreadhorde_the_person_may_keep_a_token_as_the_artifact() {
    let h = Arc::new(ScriptHuman::from_fn(|r: &Request| {
        if &*r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        Answer::Pick(r.choices.iter().position(|c| c.contains("Treasure")).unwrap_or(0))
    }));
    let mut g = table(&["kefka", "seph"]);
    g.humans.add(P0, h.clone());
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P0, "Mind Stone");
    g.player_mut(P0).treasures = 3;
    g.player_mut(P0).clues = 2;
    let src = perm(&mut g, P1, "Liliana, Dreadhorde General");
    (sim_core::impls::rules::LDG_ULT.eff)(&mut g, P1, src).unwrap();
    let r = h.asked().into_iter().find(|r| r.prompt.contains("artifact you keep")).expect("asked for the artifact");
    let all = r.choices.join(" | ");
    assert!(all.contains("Treasure token") && all.contains("Clue token"), "tokens are artifacts to choose: {all}");
    assert_eq!(r.choices.iter().filter(|c| c.contains("Treasure")).count(), 1, "a kind is one choice: {all}");
    assert_eq!(g.player(P0).treasures, 1, "one Treasure kept");
    assert_eq!(g.player(P0).clues, 0, "the Clues were sacrificed");
    assert!(!on(&g, P0, "Sol Ring") && !on(&g, P0, "Mind Stone"));
}

#[test]
fn gadgeteer_endless_atlas_costs_one_not_two() {
    // Endless Atlas {2},{T}: draw a card. Three Islands (it needs three lands with the same name), two of them tapped
    let mut g = table(&["kefka", "seph"]);
    let atlas = perm(&mut g, P0, "Endless Atlas");
    let ls = lands(&mut g, P0, "Island", 3, false);
    g.perm_mut(ls[0]).tapped = true;
    g.perm_mut(ls[1]).tapped = true;
    assert!(options_of(&mut g, atlas, P0, Some(false)).is_empty(), "one untapped land is not {{2}}");
    perm(&mut g, P0, "Forensic Gadgeteer");
    let o = options_of(&mut g, atlas, P0, Some(false));
    assert_eq!(o.len(), 1, "{{1}} with the Gadgeteer");
    let hand = g.player(P0).hand.len();
    assert!(run(&mut g, o.into_iter().next().unwrap(), P0));
    assert_eq!(g.player(P0).hand.len(), hand + 1);
    assert!(g.perm(ls[2]).tapped, "the untapped Island paid");
}

#[test]
fn gadgeteer_midnight_clock_keeps_its_blue_pip() {
    // Midnight Clock {2}{U}: with a Gadgeteer {1}{U}; two Islands pay it, one does not
    let mut g = table(&["stella-lee-quick-draw", "seph"]);
    let clock = perm(&mut g, P0, "Midnight Clock");
    let ls = lands(&mut g, P0, "Island", 3, false);
    g.perm_mut(ls[0]).tapped = true;
    g.active = Some(P0);
    assert!(options_of(&mut g, clock, P0, Some(false)).is_empty(), "two Islands can't pay {{2}}{{U}}");
    perm(&mut g, P0, "Forensic Gadgeteer");
    assert_eq!(options_of(&mut g, clock, P0, Some(false)).len(), 1, "{{1}}{{U}} with it");
}
