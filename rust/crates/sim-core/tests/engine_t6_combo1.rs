//! Engine feature `combo1`: combos with declared steps (`combos::Step`), each step with its own answer window, and
//! the Godo lines (impls/t6_combos_godo.rs; SPEC "Combo tests" 1, 2, 3, 4, 6, 7). Seat 0 plays the Godo deck; seat 1
//! holds the answers.

use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::combos::{self, Combo, Step, StepKind};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    let mut g = table(&["godo", "seph"]);
    g.active = Some(P0);
    g
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn go(g: &mut Game, part: &str) -> bool {
    let i = combos::combo_index(part).unwrap_or_else(|| panic!("no combo {part}"));
    assert!(combos::ready(g, P0, i), "{part} is ready");
    combos::attempt(g, P0, i as i64).unwrap()
}

fn not_ready(g: &Game, part: &str) -> bool {
    let i = combos::combo_index(part).unwrap();
    !combos::ready(g, P0, i)
}

fn mountains(g: &mut Game, n: usize) {
    lands(g, P0, "Mountain", n, false);
}

fn blue_black(g: &mut Game, p: PlayerId) {
    lands(g, p, "Island", 2, false);
    lands(g, p, "Swamp", 2, false);
}

// ============================================================ the framework, with synthetic steps
fn yes(_g: &mut Game, _p: PlayerId) -> sim_core::flow::Res<bool> {
    Ok(true)
}

fn no_src(_g: &Game, _p: PlayerId) -> Option<PermId> {
    None
}

fn ready_now(_g: &Game, _p: PlayerId) -> combos::Ready {
    (true, vec![], vec![])
}

fn synthetic(kind: StepKind, keys: fn(&Game, PlayerId) -> Vec<PermId>) -> Combo {
    let st: &'static [Step] =
        Box::leak(Box::new([Step { label: "the step", kind, src: no_src, pay: (0, ""), run: yes, keys }]));
    Combo { name: "synthetic", groups: &[], ready: ready_now, mana: (0, ""), finish: None, xcost: &[], steps: st }
}

fn no_keys(_g: &Game, _p: PlayerId) -> Vec<PermId> {
    vec![]
}

#[test]
fn a_trigger_step_without_an_answer_resolves_and_the_player_wins() {
    let mut g = tbl();
    combos::run_steps(&mut g, P0, &synthetic(StepKind::Trigger, no_keys)).unwrap();
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn a_trigger_step_is_answered_by_the_ability_answer_path() {
    let mut g = tbl();
    blue_black(&mut g, P1);
    hand(&mut g, P1, &["Ertai Resurrected"]);
    combos::run_steps(&mut g, P0, &synthetic(StepKind::Trigger, no_keys)).unwrap();
    assert_eq!(g.winner, None, "Ertai countered the trigger: the combo stopped at its step");
    assert!(on_bf(&g, P1, "Ertai Resurrected").is_some());
}

#[test]
fn an_ability_step_is_answered_by_a_guildmage() {
    let mut g = tbl();
    lands(&mut g, P1, "Island", 3, false);
    perm(&mut g, P1, "Azorius Guildmage");
    combos::run_steps(&mut g, P0, &synthetic(StepKind::Ability, no_keys)).unwrap();
    assert_eq!(g.winner, None, "the activated ability was countered");
    // a trigger is not an activated ability: the Guildmage can't answer it
    let mut g = tbl();
    lands(&mut g, P1, "Island", 3, false);
    perm(&mut g, P1, "Azorius Guildmage");
    combos::run_steps(&mut g, P0, &synthetic(StepKind::Trigger, no_keys)).unwrap();
    assert_eq!(g.winner, Some(P0));
}

fn keys_of_token(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf).collect()
}

#[test]
fn a_permanent_step_lets_opponents_remove_the_key_piece() {
    let mut stopped = 0;
    for seed in 0..8u64 {
        let mut g = table_with(&["godo", "seph"], seed, 40);
        g.active = Some(P0);
        token(&mut g, P0, 3);
        lands(&mut g, P1, "Plains", 1, false);
        hand(&mut g, P1, &["Swords to Plowshares"]);
        combos::run_steps(&mut g, P0, &synthetic(StepKind::Permanent, keys_of_token)).unwrap();
        if g.winner.is_none() {
            stopped += 1;
        }
    }
    assert!(stopped >= 6, "instant-speed removal on the key piece stops the combo ({stopped}/8)");
    let mut g = tbl();
    combos::run_steps(&mut g, P0, &synthetic(StepKind::Permanent, no_keys)).unwrap();
    assert_eq!(g.winner, Some(P0), "no key piece to answer, nobody to answer it");
}

#[test]
fn combos_without_steps_keep_their_single_window() {
    let i = combos::combo_index("Sanguine Bond").unwrap();
    assert!(combos::all()[i].steps.is_empty());
    assert!(combos::all().len() > 14, "the Tier 6 modules are chained after the base combos");
    assert_eq!(combos::all()[i].name, "Sanguine Bond + Exquisite Blood");
}

// ============================================================ Godo + Helm of the Host
#[test]
fn spec_1_godo_with_the_equip_paid_wins_off_eleven_mana() {
    let mut g = tbl();
    mountains(&mut g, 10);
    assert!(not_ready(&g, "equip {5}"), "ten mana is one short");
    mountains(&mut g, 1);
    go(&mut g, "equip {5}");
    assert_eq!(g.winner, Some(P0));
    let helm = on_bf(&g, P0, "Helm of the Host").unwrap();
    assert!(g.perm(helm).attached.is_some(), "the equip was paid and resolved");
}

#[test]
fn spec_2_godo_with_hammer_out_attaches_helm_for_free() {
    let mut g = tbl();
    perm(&mut g, P0, "Hammer of Nazahn");
    mountains(&mut g, 6);
    assert!(not_ready(&g, "equip {5}"), "the equip line steps aside for the cheaper one");
    go(&mut g, "Hammer of Nazahn out");
    assert_eq!(g.winner, Some(P0));
    let (helm, godo) = (on_bf(&g, P0, "Helm of the Host").unwrap(), on_bf(&g, P0, "Godo, Bandit Warlord").unwrap());
    assert_eq!(g.perm(helm).attached, Some(godo));
}

#[test]
fn godo_with_an_untapped_brass_squire() {
    let mut g = tbl();
    let sq = perm(&mut g, P0, "Brass Squire");
    g.perm_mut(sq).sick = true;
    mountains(&mut g, 6);
    assert!(not_ready(&g, "Brass Squire attaches"), "summoning sick");
    g.perm_mut(sq).sick = false;
    go(&mut g, "Brass Squire attaches");
    assert_eq!(g.winner, Some(P0));
    assert!(g.perm(sq).tapped, "the Squire's {{T}} was paid");
}

#[test]
fn godo_with_magnetic_theft() {
    let mut g = tbl();
    hand(&mut g, P0, &["Magnetic Theft"]);
    mountains(&mut g, 6);
    assert!(not_ready(&g, "Magnetic Theft attaches"), "needs {{R}} more");
    mountains(&mut g, 1);
    go(&mut g, "Magnetic Theft attaches");
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn spec_3_camera_copies_the_enter_trigger_the_copy_fetches_hammer_the_original_helm() {
    let mut g = tbl();
    perm(&mut g, P0, "Peter Parker's Camera");
    mountains(&mut g, 7);
    assert!(not_ready(&g, "Camera copies"), "Godo 6 + the Camera's {{2}}");
    mountains(&mut g, 1);
    go(&mut g, "Camera copies");
    assert_eq!(g.winner, Some(P0));
    let (helm, godo) = (on_bf(&g, P0, "Helm of the Host").unwrap(), on_bf(&g, P0, "Godo, Bandit Warlord").unwrap());
    assert!(on_bf(&g, P0, "Hammer of Nazahn").is_some(), "the copy fetched Hammer");
    assert_eq!(g.perm(helm).attached, Some(godo), "Hammer attached the original's Helm");
}

#[test]
fn spec_7_hammer_first_then_a_copy_spell_on_godo() {
    for (spell, lands_n) in [("Twinflame", 8), ("Molten Duplication", 8), ("Heat Shimmer", 9), ("Electroduplicate", 9)]
    {
        let mut g = tbl();
        hand(&mut g, P0, &[spell]);
        mountains(&mut g, lands_n - 1);
        assert!(not_ready(&g, "a copy spell on Godo"), "{spell}: one mana short");
        mountains(&mut g, 1);
        go(&mut g, "a copy spell on Godo");
        assert_eq!(g.winner, Some(P0), "{spell}");
        let helm = on_bf(&g, P0, "Helm of the Host").expect(spell);
        assert!(on_bf(&g, P0, "Hammer of Nazahn").is_some());
        let t = g.perm(helm).attached.expect("Hammer attached Helm with no equip payment");
        assert_eq!(&*g.perm(t).name, "Godo, Bandit Warlord");
        assert_eq!(g.player(P0).library.iter().filter(|&&c| &*g.db.get(c).name == "Helm of the Host").count(), 0);
    }
}

#[test]
fn hammer_first_then_cursed_mirror_or_sculpting_steel_as_godo() {
    let mut g = tbl();
    hand(&mut g, P0, &["Cursed Mirror"]);
    mountains(&mut g, 9);
    go(&mut g, "Cursed Mirror / Sculpting Steel");
    assert_eq!(g.winner, Some(P0));
    assert!(on_bf(&g, P0, "Helm of the Host").is_some());
    let mut g = tbl();
    perm(&mut g, P0, "Cursed Mirror");
    hand(&mut g, P0, &["Sculpting Steel"]);
    mountains(&mut g, 9);
    go(&mut g, "Cursed Mirror / Sculpting Steel");
    assert_eq!(g.winner, Some(P0));
}

#[test]
fn hammer_first_then_jaxis_copies_godo() {
    let mut g = tbl();
    let j = perm(&mut g, P0, "Jaxis, the Troublemaker");
    g.perm_mut(j).sick = false;
    mountains(&mut g, 7);
    go(&mut g, "Jaxis copies Godo");
    assert_eq!(g.winner, Some(P0));
    let mut g = tbl();
    hand(&mut g, P0, &["Jaxis, the Troublemaker"]);
    mountains(&mut g, 8);
    assert!(not_ready(&g, "Jaxis copies Godo"), "blitz {{1}}{{R}} plus {{R}} on top of Godo is nine");
    mountains(&mut g, 1);
    go(&mut g, "Jaxis copies Godo");
    assert_eq!(g.winner, Some(P0));
}

// ============================================================ Dualcaster Mage + a copy spell
#[test]
fn spec_4_dualcaster_with_twinflame_needs_no_commander() {
    let mut g = tbl();
    token(&mut g, P0, 1);
    hand(&mut g, P0, &["Twinflame", "Dualcaster Mage"]);
    mountains(&mut g, 4);
    assert!(not_ready(&g, "Dualcaster Mage + a copy spell"), "{{1}}{{R}} + {{1}}{{R}}{{R}} is five");
    mountains(&mut g, 1);
    go(&mut g, "Dualcaster Mage + a copy spell");
    assert_eq!(g.winner, Some(P0));
    let mut g = tbl();
    hand(&mut g, P0, &["Twinflame", "Dualcaster Mage"]);
    mountains(&mut g, 5);
    assert!(not_ready(&g, "Dualcaster Mage + a copy spell"), "a creature to target is needed");
}

// ============================================================ opponents with answers
#[test]
fn a_counterspell_on_godo_stops_the_line_at_the_cast() {
    // whether the AI counters is a roll: over a few seeds it does at least once, and then nothing else happens
    let mut countered = 0;
    for seed in 0..16u64 {
        let mut g = table_with(&["godo", "seph"], seed, 40);
        g.active = Some(P0);
        perm(&mut g, P0, "Hammer of Nazahn");
        mountains(&mut g, 6);
        lands(&mut g, P1, "Island", 2, false);
        hand(&mut g, P1, &["Counterspell"]);
        go(&mut g, "Hammer of Nazahn out");
        if on_bf(&g, P0, "Godo, Bandit Warlord").is_none() {
            countered += 1;
            assert_eq!(g.winner, None, "Godo was countered: the combo stopped at the cast");
            assert!(on_bf(&g, P0, "Helm of the Host").is_none());
        } else {
            assert_eq!(g.winner, Some(P0), "Godo resolved and nothing else answered");
        }
    }
    assert!(countered >= 1, "the AI counters Godo sometimes");
}

#[test]
fn ertai_on_godos_enter_trigger_stops_the_line_at_the_trigger() {
    let mut g = tbl();
    perm(&mut g, P0, "Hammer of Nazahn");
    mountains(&mut g, 6);
    blue_black(&mut g, P1);
    hand(&mut g, P1, &["Ertai Resurrected"]);
    go(&mut g, "Hammer of Nazahn out");
    assert_eq!(g.winner, None);
    assert!(on_bf(&g, P0, "Godo, Bandit Warlord").is_some(), "Godo resolved");
    assert!(on_bf(&g, P0, "Helm of the Host").is_none(), "but the trigger was countered");
}

#[test]
fn a_guildmage_on_the_equip_ability_stops_the_line_at_the_equip() {
    let mut g = tbl();
    mountains(&mut g, 11);
    lands(&mut g, P1, "Island", 3, false);
    perm(&mut g, P1, "Azorius Guildmage");
    go(&mut g, "equip {5}");
    assert_eq!(g.winner, None);
    let helm = on_bf(&g, P0, "Helm of the Host").unwrap();
    assert_eq!(g.perm(helm).attached, None, "the equip was countered");
}

#[test]
fn a_guildmage_on_the_squires_ability_stops_the_line() {
    let mut g = tbl();
    perm(&mut g, P0, "Brass Squire");
    let sq = on_bf(&g, P0, "Brass Squire").unwrap();
    g.perm_mut(sq).sick = false;
    mountains(&mut g, 6);
    lands(&mut g, P1, "Island", 3, false);
    perm(&mut g, P1, "Azorius Guildmage");
    go(&mut g, "Brass Squire attaches");
    assert_eq!(g.winner, None);
    assert_eq!(g.perm(on_bf(&g, P0, "Helm of the Host").unwrap()).attached, None);
}

#[test]
fn removal_on_godo_at_instant_speed_stops_the_line_before_combat() {
    let mut stopped = 0;
    for seed in 0..8u64 {
        let mut g = table_with(&["godo", "seph"], seed, 40);
        g.active = Some(P0);
        perm(&mut g, P0, "Hammer of Nazahn");
        mountains(&mut g, 6);
        lands(&mut g, P1, "Plains", 1, false);
        hand(&mut g, P1, &["Swords to Plowshares"]);
        go(&mut g, "Hammer of Nazahn out");
        if g.winner.is_none() {
            stopped += 1;
            assert!(on_bf(&g, P0, "Godo, Bandit Warlord").is_none(), "Swords exiled Godo");
        }
    }
    assert!(stopped >= 6, "the answer is held back only 5% of the time ({stopped}/8)");
}

#[test]
fn a_counterspell_on_the_copy_spell_stops_the_dualcaster_loop_before_it_starts() {
    let mut countered = 0;
    for seed in 0..16u64 {
        let mut g = table_with(&["godo", "seph"], seed, 40);
        g.active = Some(P0);
        token(&mut g, P0, 1);
        hand(&mut g, P0, &["Twinflame", "Dualcaster Mage"]);
        mountains(&mut g, 5);
        lands(&mut g, P1, "Island", 2, false);
        hand(&mut g, P1, &["Counterspell"]);
        go(&mut g, "Dualcaster Mage + a copy spell");
        if on_bf(&g, P0, "Dualcaster Mage").is_none() {
            countered += 1;
            assert_eq!(g.winner, None, "the original spell was countered: the loop never starts");
        } else {
            assert_eq!(g.winner, Some(P0));
        }
    }
    assert!(countered >= 1, "the AI counters the copy spell sometimes");
}

#[test]
fn ertai_on_dualcasters_enter_trigger_stops_the_loop() {
    let mut g = tbl();
    token(&mut g, P0, 1);
    hand(&mut g, P0, &["Twinflame", "Dualcaster Mage"]);
    mountains(&mut g, 5);
    blue_black(&mut g, P1);
    hand(&mut g, P1, &["Ertai Resurrected"]);
    go(&mut g, "Dualcaster Mage + a copy spell");
    assert_eq!(g.winner, None);
    assert!(on_bf(&g, P0, "Dualcaster Mage").is_some(), "it resolved, its trigger did not");
}

#[test]
fn the_godo_deck_is_offered_the_line_from_the_command_zone() {
    let mut g = tbl();
    perm(&mut g, P0, "Hammer of Nazahn");
    mountains(&mut g, 6);
    let o = sim_core::cardcode::combo_options(&mut g, P0, 0.0, Some(false)).unwrap();
    let labels: Vec<String> = o.iter().map(|x| x.label.clone()).collect();
    assert_eq!(labels, ["combo: Godo + Helm of the Host (Hammer of Nazahn out)"], "{labels:?}");
}
