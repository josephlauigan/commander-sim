//! Practice mode (2026-11-04 gap fixes): the Cloud and Karn cards whose ability the person could not activate, or whose
//! choice the AI made for them. Each test asks the person and checks that the answer is what happens.

mod play_common;
use play_common::*;
use sim_core::human::human::human_main;
use sim_core::human::{Target, legal};
use std::sync::{Arc, Mutex};

/// how the person answers: for a question whose prompt contains `key`, the choice whose text contains `word`
type Rule = (String, String);

fn ans_word(key: &str, word: &str) -> Rule {
    (key.to_string(), word.to_string())
}

/// the target described as x, for questions whose prompt contains `key`
fn ans_target(g: &Game, key: &str, x: Target) -> Rule {
    (key.to_string(), legal::describe_target(g, P0, x))
}

/// the person takes `action` at the first priority, answers by `rules` (the first that fits; the first choice when
/// none does) and passes at every other priority
fn play(g: &mut Game, action: Value, rules: Vec<Rule>) -> Arc<ScriptHuman> {
    let first = Mutex::new(Some(action));
    let h = Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if &*r.kind == "priority" {
            return match first.lock().unwrap().take() {
                Some(a) => Answer::Action(a),
                None => Answer::Action(json!({"do": "pass"})),
            };
        }
        for (key, word) in &rules {
            if r.prompt.contains(key.as_str())
                && let Some(i) = r.choices.iter().position(|c| c.contains(word.as_str()))
            {
                return Answer::Pick(i);
            }
        }
        Answer::Pick(0)
    }));
    g.humans.add(P0, h.clone());
    human_main(g, P0, false).unwrap();
    h
}

fn use_perm(g: &mut Game, m: PermId, rules: Vec<Rule>) -> Arc<ScriptHuman> {
    let i = g.player(P0).perms.iter().position(|&x| x == m).unwrap();
    play(g, json!({"do": "use", "perm": i}), rules)
}

fn use_land(g: &mut Game, l: PermId, rules: Vec<Rule>) -> Arc<ScriptHuman> {
    let i = g.player(P0).lands.iter().position(|&x| x == l).unwrap();
    play(g, json!({"do": "use", "land": i}), rules)
}

/// cast the i-th card of the hand
fn cast_hand(g: &mut Game, i: usize, rules: Vec<Rule>) -> Arc<ScriptHuman> {
    play(g, json!({"do": "cast", "card": i}), rules)
}

fn cloud_table() -> Game {
    table(&["cloud", "seph"])
}

fn karn_table() -> Game {
    table(&["karn", "cloud"])
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_exile(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).exile.iter().any(|&c| &*g.db.get(c).name == name)
}

fn asked(h: &ScriptHuman, word: &str) -> bool {
    h.asked().iter().any(|r| r.prompt.contains(word))
}

fn has_kw(g: &Game, m: PermId, kw: &str) -> bool {
    g.perm(m).eot_kw.iter().any(|k| *k == kw)
}

fn set_library(g: &mut Game, p: PlayerId, names: &[&str]) {
    let cs: Vec<_> = names.iter().map(|n| card(g, n)).collect();
    g.player_mut(p).library = cs;
}

fn to_gy(g: &mut Game, p: PlayerId, names: &[&str]) {
    for n in names {
        let c = take(g, p, n);
        g.player_mut(p).gy.push(c);
    }
}

// ------------------------------------------------------------------ Cloud: abilities
#[test]
fn balan_attaches_all_equipment_for_two_mana_at_any_time() {
    let mut g = cloud_table();
    let b = perm(&mut g, P0, "Balan, Wandering Knight");
    let e1 = perm(&mut g, P0, "Bonesplitter");
    let e2 = perm(&mut g, P0, "Lightning Greaves");
    pool(&mut g, P0, &[('W', 1), ('G', 1)]);
    use_perm(&mut g, b, vec![]);
    assert_eq!(g.perm(e1).attached, Some(b));
    assert_eq!(g.perm(e2).attached, Some(b));
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn cid_returns_the_equipment_the_person_picks() {
    let mut g = cloud_table();
    let cid = perm(&mut g, P0, "Cid, Freeflier Pilot");
    to_gy(&mut g, P0, &["Bonesplitter", "Skullclamp"]);
    pool(&mut g, P0, &[('C', 2)]);
    let h = use_perm(&mut g, cid, vec![ans_word("Cid: return which", "Skullclamp")]);
    assert!(asked(&h, "Cid: return which"));
    assert!(in_hand(&g, P0, "Skullclamp") && in_gy(&g, P0, "Bonesplitter"));
    assert!(g.perm(cid).tapped);
}

#[test]
fn giver_of_runes_gives_the_chosen_protection() {
    let mut g = cloud_table();
    let giver = perm(&mut g, P0, "Giver of Runes");
    let other = perm(&mut g, P0, "Llanowar Elves");
    let tg = ans_target(&g, "which other creature", Target::Perm(other));
    let h = use_perm(&mut g, giver, vec![tg, ans_word("protection from what", "red")]);
    assert!(asked(&h, "protection from what"));
    assert!(has_kw(&g, other, "pro_R"), "{:?}", g.perm(other).eot_kw);
    assert!(g.perm(giver).tapped);
}

#[test]
fn stoneforge_mystic_puts_the_equipment_the_person_picks_onto_the_battlefield() {
    let mut g = cloud_table();
    let s = perm(&mut g, P0, "Stoneforge Mystic");
    hand(&mut g, P0, &["Bonesplitter", "Skullclamp"]);
    pool(&mut g, P0, &[('W', 1), ('G', 1)]);
    let h = use_perm(&mut g, s, vec![ans_word("put which Equipment", "Skullclamp")]);
    assert!(asked(&h, "put which Equipment"));
    assert!(g.player(P0).perms.iter().any(|&m| &*g.perm(m).name == "Skullclamp"));
    assert!(in_hand(&g, P0, "Bonesplitter"));
    assert!(g.perm(s).tapped);
}

#[test]
fn zack_fair_sacrifices_to_protect_the_creature_the_person_picks() {
    let mut g = cloud_table();
    let z = perm(&mut g, P0, "Zack Fair");
    let other = perm(&mut g, P0, "Llanowar Elves");
    pool(&mut g, P0, &[('C', 1)]);
    let tg = ans_target(&g, "Zack Fair", Target::Perm(other));
    use_perm(&mut g, z, vec![tg]);
    assert!(!g.perm(z).on_bf, "sacrificed");
    assert!(has_kw(&g, other, "indestructible"));
}

#[test]
fn plaza_of_heroes_exiles_itself_to_protect_a_legendary_creature() {
    let mut g = cloud_table();
    let l = lands(&mut g, P0, "Plaza of Heroes", 1, false)[0];
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    pool(&mut g, P0, &[('C', 3)]);
    let tg = ans_target(&g, "Plaza of Heroes", Target::Perm(cl));
    use_land(&mut g, l, vec![tg]);
    assert!(in_exile(&g, P0, "Plaza of Heroes"));
    assert!(has_kw(&g, cl, "hexproof") && has_kw(&g, cl, "indestructible"));
}

// ------------------------------------------------------------------ Karn: abilities
#[test]
fn expedition_map_finds_the_land_the_person_picks() {
    let mut g = karn_table();
    let m = perm(&mut g, P0, "Expedition Map");
    set_library(&mut g, P0, &["Mishra's Workshop", "Gaea's Cradle", "Island"]);
    pool(&mut g, P0, &[('C', 2)]);
    let h = use_perm(&mut g, m, vec![ans_word("Search", "Gaea's Cradle")]);
    assert!(!h.asked_kind("search").is_empty());
    assert!(in_hand(&g, P0, "Gaea's Cradle") && !g.perm(m).on_bf);
}

#[test]
fn urzas_cave_fetches_the_land_the_person_picks_tapped() {
    let mut g = karn_table();
    let l = lands(&mut g, P0, "Urza's Cave", 1, false)[0];
    set_library(&mut g, P0, &["Mishra's Workshop", "Gaea's Cradle", "Island"]);
    pool(&mut g, P0, &[('C', 3)]);
    use_land(&mut g, l, vec![ans_word("Search", "Mishra's Workshop")]);
    assert!(in_gy(&g, P0, "Urza's Cave"));
    let w =
        g.player(P0).lands.iter().copied().find(|&x| &*g.db.get(g.perm(x).card()).name == "Mishra's Workshop").unwrap();
    assert!(g.perm(w).tapped);
}

#[test]
fn urzas_saga_makes_a_construct_from_chapter_two() {
    let mut g = karn_table();
    let l = lands(&mut g, P0, "Urza's Saga", 1, false)[0];
    pool(&mut g, P0, &[('C', 2)]);
    // chapter I: the Construct ability is not there yet
    g.player_mut(P0).sagas.push((l, 1));
    let h = use_land(&mut g, l, vec![]);
    assert!(h.told_kind("invalid").iter().any(|t| t.contains("lore counter II")), "{:?}", h.told());
    assert!(!g.perm(l).tapped);
    g.player_mut(P0).sagas[0].1 = 2;
    use_land(&mut g, l, vec![]);
    assert!(g.perm(l).tapped);
    assert_eq!(pool_total(&g, P0), 0);
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).data.truthy(sim_core::state::DataKey::Construct)));
}

#[test]
fn urzas_tome_draws_then_the_person_exiles_a_historic_card_or_discards() {
    let mut g = karn_table();
    let t = perm(&mut g, P0, "Urza's Tome");
    to_gy(&mut g, P0, &["Sol Ring", "Grim Monolith"]);
    hand(&mut g, P0, &["Island"]);
    pool(&mut g, P0, &[('C', 3)]);
    let before = g.player(P0).hand.len();
    let h = use_perm(&mut g, t, vec![ans_word("exile a historic card", "Grim Monolith")]);
    assert!(asked(&h, "exile a historic card"));
    assert!(in_exile(&g, P0, "Grim Monolith") && in_gy(&g, P0, "Sol Ring"));
    assert_eq!(g.player(P0).hand.len(), before + 1, "drew a card, discarded nothing");
    // with nothing historic in the graveyard it asks what to discard
    let mut g = karn_table();
    let t = perm(&mut g, P0, "Urza's Tome");
    hand(&mut g, P0, &["Island", "Swamp"]);
    pool(&mut g, P0, &[('C', 3)]);
    let h = use_perm(&mut g, t, vec![ans_word("discard which", "Swamp")]);
    assert!(asked(&h, "discard which"));
    assert!(in_gy(&g, P0, "Swamp"));
}

// ------------------------------------------------------------------ Cloud: choices
#[test]
fn akromas_will_asks_the_mode() {
    let mut g = cloud_table();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    hand(&mut g, P0, &["Akroma's Will"]);
    pool(&mut g, P0, &[('W', 1), ('C', 3)]);
    let h = cast_hand(&mut g, 0, vec![ans_word("choose a mode", "lifelink")]);
    assert!(asked(&h, "choose a mode"));
    assert!(has_kw(&g, elf, "lifelink") && has_kw(&g, elf, "pro_colors"));
    assert!(!has_kw(&g, elf, "flying"), "mode two only");
}

#[test]
fn akromas_will_offers_both_with_a_commander() {
    let mut g = cloud_table();
    let cmd = perm(&mut g, P0, "Llanowar Elves");
    g.perm_mut(cmd).is_cmd = true; // (a commander on the battlefield)
    hand(&mut g, P0, &["Akroma's Will"]);
    pool(&mut g, P0, &[('W', 1), ('C', 3)]);
    let h = cast_hand(&mut g, 0, vec![ans_word("choose a mode", "both")]);
    assert!(h.asked().iter().any(|r| r.choices.iter().any(|c| c == "both")));
    assert!(has_kw(&g, cmd, "flying") && has_kw(&g, cmd, "lifelink"));
}

#[test]
fn winds_of_abandon_exiles_the_creature_the_person_targets() {
    let mut g = cloud_table();
    let a = perm(&mut g, P1, "Llanowar Elves");
    let b = perm(&mut g, P1, "Birds of Paradise");
    hand(&mut g, P0, &["Winds of Abandon"]);
    pool(&mut g, P0, &[('W', 1), ('C', 1)]);
    let tg = ans_target(&g, "creature you don't control", Target::Perm(b));
    let lands_before = g.player(P1).lands.len();
    let h = cast_hand(&mut g, 0, vec![tg]);
    assert!(asked(&h, "creature you don't control"));
    assert!(!g.perm(b).on_bf && g.perm(a).on_bf);
    assert_eq!(g.player(P1).lands.len(), lands_before + 1, "its controller got a basic land");
}

#[test]
fn winds_of_abandon_overloads_when_the_person_chooses_and_can_pay() {
    let mut g = cloud_table();
    let a = perm(&mut g, P1, "Llanowar Elves");
    let b = perm(&mut g, P1, "Birds of Paradise");
    hand(&mut g, P0, &["Winds of Abandon"]);
    pool(&mut g, P0, &[('W', 2), ('C', 4)]);
    let h = cast_hand(&mut g, 0, vec![ans_word("how?", "overload")]);
    assert!(asked(&h, "how?"));
    assert!(!g.perm(a).on_bf && !g.perm(b).on_bf);
    assert_eq!(pool_total(&g, P0), 0, "{{4}}{{W}}{{W}} paid");
}

#[test]
fn galadriels_dismissal_phases_out_the_target_creature() {
    let mut g = cloud_table();
    let a = perm(&mut g, P0, "Llanowar Elves");
    let b = perm(&mut g, P0, "Birds of Paradise");
    hand(&mut g, P0, &["Galadriel's Dismissal"]);
    pool(&mut g, P0, &[('W', 1)]);
    let tg = ans_target(&g, "choose a target (creature)", Target::Perm(a));
    cast_hand(&mut g, 0, vec![tg]);
    assert!(g.perm(a).phased && !g.perm(b).phased);
}

#[test]
fn galadriels_dismissal_kicked_phases_out_each_creature_of_a_player() {
    let mut g = cloud_table();
    let a = perm(&mut g, P0, "Llanowar Elves");
    let b = perm(&mut g, P0, "Birds of Paradise");
    let c = perm(&mut g, P1, "Sol Ring");
    hand(&mut g, P0, &["Galadriel's Dismissal"]);
    pool(&mut g, P0, &[('W', 2), ('C', 2)]);
    let me = ans_target(&g, "choose a target (player)", Target::Player(P0));
    let h = cast_hand(&mut g, 0, vec![ans_word("kicker?", "kicker"), me]);
    assert!(asked(&h, "kicker?"));
    assert!(g.perm(a).phased && g.perm(b).phased && !g.perm(c).phased);
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn kor_outfitter_attaches_the_equipment_to_the_creature_the_person_picks() {
    let mut g = cloud_table();
    let e = perm(&mut g, P0, "Bonesplitter");
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let _birds = perm(&mut g, P0, "Birds of Paradise");
    hand(&mut g, P0, &["Kor Outfitter"]);
    pool(&mut g, P0, &[('W', 2)]);
    let te = ans_target(&g, "which Equipment", Target::Perm(e));
    let tc = ans_target(&g, "which creature", Target::Perm(elf));
    let h = cast_hand(&mut g, 0, vec![te, tc]);
    assert!(asked(&h, "attach it to which creature"));
    assert_eq!(g.perm(e).attached, Some(elf));
}

#[test]
fn forge_anew_returns_the_equipment_the_person_picks() {
    let mut g = cloud_table();
    to_gy(&mut g, P0, &["Bonesplitter", "Skullclamp"]);
    hand(&mut g, P0, &["Forge Anew"]);
    pool(&mut g, P0, &[('W', 1), ('C', 2)]);
    let h = cast_hand(&mut g, 0, vec![ans_word("Forge Anew: return which", "Skullclamp")]);
    assert!(asked(&h, "Forge Anew: return which"));
    assert!(g.player(P0).perms.iter().any(|&m| &*g.perm(m).name == "Skullclamp"));
    assert!(in_gy(&g, P0, "Bonesplitter"));
}

// ------------------------------------------------------------------ Karn: choices
#[test]
fn pithing_needle_names_the_card_the_person_picks() {
    let mut g = karn_table();
    perm(&mut g, P1, "Mishra's Bauble");
    hand(&mut g, P0, &["Pithing Needle"]);
    pool(&mut g, P0, &[('C', 1)]);
    let h = cast_hand(&mut g, 0, vec![ans_word("choose a card name", "Mishra's Bauble")]);
    assert!(asked(&h, "choose a card name"));
    assert!(sim_core::impls::t6_stax::needle_stops(&g, "Mishra's Bauble"));
    assert!(!sim_core::impls::t6_stax::needle_stops(&g, "Sol Ring"));
}

#[test]
fn soul_guide_lantern_exiles_the_card_the_person_picks_from_any_graveyard() {
    let mut g = karn_table();
    to_gy(&mut g, P0, &["Island"]);
    to_gy(&mut g, P1, &["Grave Titan", "Sol Ring"]);
    hand(&mut g, P0, &["Soul-Guide Lantern"]);
    pool(&mut g, P0, &[('C', 1)]);
    let h = cast_hand(&mut g, 0, vec![ans_word("exile which card", "Island")]);
    assert!(asked(&h, "exile which card"));
    assert!(in_exile(&g, P0, "Island") && in_gy(&g, P1, "Grave Titan"));
}

#[test]
fn spine_of_ish_sah_destroys_the_permanent_the_person_picks() {
    let mut g = karn_table();
    let a = perm(&mut g, P1, "Sol Ring");
    let b = perm(&mut g, P1, "Arcane Signet");
    hand(&mut g, P0, &["Spine of Ish Sah"]);
    pool(&mut g, P0, &[('C', 7)]);
    let tg = ans_target(&g, "destroy which permanent", Target::Perm(b));
    let h = cast_hand(&mut g, 0, vec![tg]);
    assert!(asked(&h, "destroy which permanent"));
    assert!(g.perm(a).on_bf && !g.perm(b).on_bf);
}
