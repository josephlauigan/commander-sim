//! The white cards of the cloud deck (impls/bulk_white.rs) on hand-built positions. Seat 0 plays the cloud deck, seat 1
//! the seph deck (seat 1 only holds targets and plays removal).

use sim_core::ai::brain;
use sim_core::engine::turn;
use sim_core::engine::values::{epow, etgh, untargetable};
use sim_core::engine::{cast, zones};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["cloud", "seph"])
}

fn options(g: &mut Game, p: PlayerId, post: bool) -> Vec<Opt> {
    brain::main_options(g, p, post).unwrap()
}

fn option_labels(g: &mut Game, post: bool) -> Vec<String> {
    options(g, P0, post).into_iter().map(|o| o.label).collect()
}

fn do_option(g: &mut Game, label: &str, post: bool) {
    let o = options(g, P0, post)
        .into_iter()
        .find(|o| o.label.starts_with(label))
        .unwrap_or_else(|| panic!("no option {label:?}"));
    match o.act.unwrap() {
        Action::Plan { f, arg } => assert!(f(g, P0, arg).unwrap(), "{label} did nothing"),
        Action::Ability { src, f, arg } => assert!(f(g, src, P0, arg).unwrap(), "{label} did nothing"),
        other => panic!("unexpected action {other:?}"),
    }
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn to_gy(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = take(g, p, name);
    g.player_mut(p).gy.push(c);
    c
}

fn tokens_named(g: &Game, p: PlayerId, ty: &str) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).token && g.perm(m).ttypes.iter().any(|t| &**t == ty)).count()
}

fn cast_spell(g: &mut Game, p: PlayerId, name: &str, target: Option<PermId>) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx { target, ..Ctx::default() }).unwrap();
    c
}

// ------------------------------------------------------------------ Exalted Angel
#[test]
fn exalted_angel_is_cast_face_down_when_it_cannot_be_cast_and_turned_up_next_turn() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 3, false);
    hand(&mut g, P0, &["Exalted Angel"]);
    assert!(option_labels(&mut g, false).contains(&"Exalted Angel (face down)".to_string()));
    do_option(&mut g, "Exalted Angel (face down)", false);
    let m = on_bf(&g, P0, "Exalted Angel").expect("a face-down creature");
    assert_eq!((epow(&g, m), etgh(&g, m)), (2, 2), "a 2/2");
    assert!(g.perm(m).neutered && !g.perm(m).lifelink && !g.perm(m).fly, "no abilities face down");
    assert_eq!(sim_core::cardcode::aura_ward(&g, m), 0, "morph has no ward");
    assert!(!option_labels(&mut g, false).iter().any(|l| l.contains("turn face up")), "no mana left");
    // the next turn, with four lands
    let _ = lands(&mut g, P0, "Plains", 4, false);
    g.perm_mut(m).sick = false;
    do_option(&mut g, "Exalted Angel (turn face up)", false);
    assert_eq!((epow(&g, m), etgh(&g, m)), (4, 5));
    assert!(!g.perm(m).neutered);
    let c = sim_core::engine::values::card_name(&g, m);
    assert_eq!(c, Some("Exalted Angel"));
}

#[test]
fn exalted_angel_is_cast_normally_with_six_lands() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 6, false);
    hand(&mut g, P0, &["Exalted Angel"]);
    assert!(!option_labels(&mut g, false).contains(&"Exalted Angel (face down)".to_string()));
}

#[test]
fn exalted_angel_gains_the_damage_it_deals_only_face_up() {
    let mut g = game();
    let a = perm(&mut g, P0, "Exalted Angel");
    g.player_mut(P0).life = 20;
    sim_core::engine::combat::resolve_combat(&mut g, P0, &[a], P1, &[]).unwrap();
    assert_eq!(g.player(P1).life, 36, "4 damage");
    assert_eq!(g.player(P0).life, 24, "you gain that much life");
}

// ------------------------------------------------------------------ Forum Familiar
#[test]
fn forum_familiar_face_down_has_ward_two_and_bounces_a_permanent_when_turned_up() {
    let mut g = game();
    let helper = perm(&mut g, P0, "Ennis, Debate Moderator");
    lands(&mut g, P0, "Plains", 5, false);
    hand(&mut g, P0, &["Forum Familiar"]);
    assert!(option_labels(&mut g, false).iter().any(|l| l == "Forum Familiar (face down)"));
    do_option(&mut g, "Forum Familiar (face down)", false);
    let fam = on_bf(&g, P0, "Forum Familiar").unwrap();
    assert!(g.perm(fam).neutered);
    assert_eq!(sim_core::cardcode::aura_ward(&g, fam), 2, "disguise: ward {{2}} face down");
    do_option(&mut g, "Forum Familiar (turn face up)", false);
    assert!(!g.perm(fam).neutered && g.perm(fam).plus == 1, "a +1/+1 counter");
    assert_eq!((epow(&g, fam), etgh(&g, fam)), (2, 2));
    assert_eq!(sim_core::cardcode::aura_ward(&g, fam), 0, "no ward face up");
    assert!(!g.perm(helper).on_bf, "Ennis went home");
    assert!(in_hand(&g, P0, "Ennis, Debate Moderator"));
}

// ------------------------------------------------------------------ Unyielding Gatekeeper
#[test]
fn gatekeeper_exiles_an_opposing_bomb_and_gives_a_detective() {
    let mut g = game();
    let bomb = perm(&mut g, P1, "Grave Titan");
    lands(&mut g, P0, "Plains", 5, false);
    hand(&mut g, P0, &["Unyielding Gatekeeper"]);
    do_option(&mut g, "Unyielding Gatekeeper (face down)", false);
    do_option(&mut g, "Unyielding Gatekeeper (turn face up)", false);
    assert!(!g.perm(bomb).on_bf);
    assert!(g.player(P1).exile.iter().any(|&c| &*g.db.get(c).name == "Grave Titan"));
    assert_eq!(tokens_named(&g, P1, "detective"), 1, "they get a 2/2 Detective");
    let d = g
        .player(P1)
        .perms
        .iter()
        .copied()
        .find(|&m| g.perm(m).token && g.perm(m).ttypes.iter().any(|t| &**t == "detective"))
        .unwrap();
    assert_eq!((epow(&g, d), etgh(&g, d)), (2, 2));
}

#[test]
fn gatekeeper_blinks_its_own_permanent_and_it_returns_tapped() {
    let mut g = game();
    let cloud = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    g.perm_mut(cloud).is_cmd = true;
    lands(&mut g, P0, "Plains", 5, false);
    hand(&mut g, P0, &["Unyielding Gatekeeper"]);
    do_option(&mut g, "Unyielding Gatekeeper (face down)", false);
    do_option(&mut g, "Unyielding Gatekeeper (turn face up)", false);
    let back = on_bf(&g, P0, "Cloud, Midgar Mercenary").expect("Cloud came back");
    assert!(g.perm(back).tapped, "returned tapped");
    assert!(g.perm(back).is_cmd);
    assert_eq!(tokens_named(&g, P0, "detective"), 0, "no Detective for your own permanent");
}

// ------------------------------------------------------------------ Exit Specialist
#[test]
fn exit_specialist_bounces_a_creature_when_turned_up_and_dodges_big_blockers() {
    let mut g = game();
    let victim = perm(&mut g, P1, "Grave Titan");
    lands(&mut g, P0, "Island", 5, false);
    hand(&mut g, P0, &["Exit Specialist"]);
    do_option(&mut g, "Exit Specialist (face down)", false);
    assert_eq!(sim_core::cardcode::aura_ward(&g, on_bf(&g, P0, "Exit Specialist").unwrap()), 2);
    do_option(&mut g, "Exit Specialist (turn face up)", false);
    assert!(!g.perm(victim).on_bf && in_hand(&g, P1, "Grave Titan"));
    // blocking
    let spec = on_bf(&g, P0, "Exit Specialist").unwrap();
    let big = perm(&mut g, P1, "Grave Titan");
    let small = token(&mut g, P1, 2);
    assert!(!sim_core::engine::combat::can_block(&g, big, spec), "power 6 can't block it");
    assert!(sim_core::engine::combat::can_block(&g, small, spec), "power 2 can");
}

// ------------------------------------------------------------------ Ennis
#[test]
fn ennis_exiles_cloud_and_he_returns_at_the_end_step_with_a_counter_for_ennis() {
    let mut g = game();
    let cloud = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    g.perm_mut(cloud).is_cmd = true;
    let c = cast_spell(&mut g, P0, "Ennis, Debate Moderator", None);
    let _ = c;
    let ennis = on_bf(&g, P0, "Ennis, Debate Moderator").unwrap();
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert!(!g.perm(cloud).on_bf, "Cloud was exiled");
    assert!(on_bf(&g, P0, "Cloud, Midgar Mercenary").is_none());
    turn::end_step(&mut g, P0).unwrap();
    let back = on_bf(&g, P0, "Cloud, Midgar Mercenary").expect("Cloud returns at the end step");
    assert!(g.perm(back).is_cmd);
    assert_eq!(g.perm(ennis).plus, 1, "cards were put into exile this turn: a counter");
}

#[test]
fn ennis_gets_no_counter_when_nothing_was_exiled() {
    let mut g = game();
    let _ = cast_spell(&mut g, P0, "Ennis, Debate Moderator", None);
    let ennis = on_bf(&g, P0, "Ennis, Debate Moderator").unwrap();
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.perm(ennis).plus, 0);
}

// ------------------------------------------------------------------ Stormfront Riders
#[test]
fn stormfront_riders_returns_two_creatures_and_makes_a_soldier_for_each() {
    let mut g = game();
    let a = perm(&mut g, P0, "Armory Automaton");
    let b = perm(&mut g, P0, "Helitrooper");
    let _ = cast_spell(&mut g, P0, "Stormfront Riders", None);
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert!(!g.perm(a).on_bf && !g.perm(b).on_bf, "both went home");
    assert!(in_hand(&g, P0, "Armory Automaton") && in_hand(&g, P0, "Helitrooper"));
    assert!(on_bf(&g, P0, "Stormfront Riders").is_some(), "Riders stays");
    assert_eq!(tokens_named(&g, P0, "soldier"), 2);
}

#[test]
fn stormfront_riders_sends_cloud_to_the_command_zone_without_a_soldier() {
    let mut g = game();
    let cloud = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    g.perm_mut(cloud).is_cmd = true;
    let other = perm(&mut g, P0, "Helitrooper");
    let _ = cast_spell(&mut g, P0, "Stormfront Riders", None);
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert!(!g.perm(cloud).on_bf && !g.perm(other).on_bf);
    assert!(g.player(P0).cmd_in_zone, "Cloud is in the command zone");
    assert_eq!(tokens_named(&g, P0, "soldier"), 1, "only the creature that reached the hand");
}

#[test]
fn stormfront_riders_makes_a_soldier_whenever_a_creature_is_bounced_to_hand() {
    let mut g = game();
    g.skip_etb = true;
    let _riders = perm(&mut g, P0, "Stormfront Riders");
    g.skip_etb = false;
    let h = perm(&mut g, P0, "Helitrooper");
    zones::bounce(&mut g, h).unwrap();
    assert_eq!(tokens_named(&g, P0, "soldier"), 1);
    // an opponent's creature returned to its owner's hand: not ours
    let t = perm(&mut g, P1, "Grave Titan");
    zones::bounce(&mut g, t).unwrap();
    assert_eq!(tokens_named(&g, P0, "soldier"), 1);
    // a noncreature permanent doesn't count
    let art = perm(&mut g, P0, "Sword of Kaldra");
    zones::bounce(&mut g, art).unwrap();
    assert_eq!(tokens_named(&g, P0, "soldier"), 1);
}

// ------------------------------------------------------------------ Call a Surprise Witness, Recommission
#[test]
fn call_a_surprise_witness_returns_a_cheap_creature_with_flying() {
    let mut g = game();
    to_gy(&mut g, P0, "Helitrooper");
    to_gy(&mut g, P0, "Grave Titan");
    lands(&mut g, P0, "Plains", 2, false);
    hand(&mut g, P0, &["Call a Surprise Witness"]);
    do_option(&mut g, "Call a Surprise Witness", true);
    let m = on_bf(&g, P0, "Helitrooper").expect("the 3-or-less creature came back");
    assert!(g.perm(m).fly, "a flying counter");
    assert!(in_gy(&g, P0, "Call a Surprise Witness"));
    assert!(g.player(P0).gy.iter().all(|&c| &*g.db.get(c).name != "Helitrooper"));
}

#[test]
fn call_a_surprise_witness_ignores_expensive_creatures_and_artifacts() {
    let mut g = game();
    to_gy(&mut g, P0, "Stormfront Riders"); // mana value 5
    to_gy(&mut g, P0, "Sword of Kaldra");
    lands(&mut g, P0, "Plains", 2, false);
    hand(&mut g, P0, &["Call a Surprise Witness"]);
    assert!(!option_labels(&mut g, true).iter().any(|l| l.starts_with("Call a Surprise Witness")));
}

#[test]
fn recommission_returns_a_creature_with_an_extra_counter_or_an_equipment() {
    let mut g = game();
    to_gy(&mut g, P0, "Armory Automaton");
    lands(&mut g, P0, "Plains", 2, false);
    hand(&mut g, P0, &["Recommission"]);
    do_option(&mut g, "Recommission", true);
    let m = on_bf(&g, P0, "Armory Automaton").unwrap();
    assert_eq!(g.perm(m).plus, 1, "an additional +1/+1 counter");
    // an artifact with mana value 3 or less
    let mut g = game();
    to_gy(&mut g, P0, "Veteran's Powerblade");
    lands(&mut g, P0, "Plains", 2, false);
    hand(&mut g, P0, &["Recommission"]);
    do_option(&mut g, "Recommission", true);
    let m = on_bf(&g, P0, "Veteran's Powerblade").expect("the Equipment came back");
    assert_eq!(g.perm(m).plus, 0, "no counter on an artifact");
}

// ------------------------------------------------------------------ Auras and protection
#[test]
fn shardmage_s_rescue_answers_removal_with_hexproof_and_a_counter() {
    let mut g = game();
    let cloud = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    g.perm_mut(cloud).is_cmd = true;
    lands(&mut g, P0, "Plains", 1, false);
    hand(&mut g, P0, &["Shardmage's Rescue"]);
    let swords = take(&mut g, P1, "Swords to Plowshares");
    sim_core::engine::removal::apply_removal(&mut g, Some(P1), cloud, "exile", Some(swords)).unwrap();
    assert!(g.perm(cloud).on_bf, "Cloud is still here");
    let aura = on_bf(&g, P0, "Shardmage's Rescue").expect("the Aura entered");
    assert_eq!(g.perm(aura).attached, Some(cloud));
    assert_eq!((epow(&g, cloud), etgh(&g, cloud)), (3, 2), "+1/+1");
    assert!(untargetable(&g, cloud), "hexproof the turn it entered");
}

#[test]
fn shardmage_s_rescue_hexproof_ends_with_the_turn() {
    let mut g = game();
    let h = perm(&mut g, P0, "Helitrooper");
    lands(&mut g, P0, "Plains", 1, false);
    let _ = cast_spell(&mut g, P0, "Shardmage's Rescue", Some(h));
    assert!(untargetable(&g, h));
    g.round += 1; // a later turn
    assert!(!untargetable(&g, h));
    assert_eq!(epow(&g, h), 2, "+1/+1 stays (a 1/2)");
}

#[test]
fn cho_manno_s_blessing_gives_protection_from_the_spells_colour() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = game();
    let cloud = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    g.perm_mut(cloud).is_cmd = true;
    lands(&mut g, P0, "Plains", 2, false);
    hand(&mut g, P0, &["Cho-Manno's Blessing"]);
    let doom = take(&mut g, P1, "Bitter Triumph");
    sim_core::engine::removal::apply_removal(&mut g, Some(P1), cloud, "destroy", Some(doom)).unwrap();
    assert!(g.perm(cloud).on_bf);
    let aura = on_bf(&g, P0, "Cho-Manno's Blessing").unwrap();
    assert_eq!(g.perm(aura).attached, Some(cloud));
    assert!(sim_core::engine::values::protected_from(&g, cloud, sim_core::cards::Colors::from_letters("B")));
    assert!(!sim_core::engine::values::protected_from(&g, cloud, sim_core::cards::Colors::from_letters("R")));
    assert_eq!(epow(&g, cloud), 2, "no stat change");
}

#[test]
fn patriot_gives_hexproof_in_response_to_removal() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = game();
    let _pat = perm(&mut g, P0, "Patriot, Shield Wielder");
    let cloud = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    g.perm_mut(cloud).is_cmd = true;
    lands(&mut g, P0, "Plains", 2, false);
    let swords = take(&mut g, P1, "Swords to Plowshares");
    sim_core::engine::removal::apply_removal(&mut g, Some(P1), cloud, "exile", Some(swords)).unwrap();
    assert!(g.perm(cloud).on_bf);
    assert!(g.perm(_pat).tapped);
    assert_eq!(epow(&g, cloud), 4, "+2/+0 until end of turn");
}

#[test]
fn patriot_pumps_the_biggest_attacker_before_combat() {
    let mut g = game();
    let pat = perm(&mut g, P0, "Patriot, Shield Wielder");
    let big = perm(&mut g, P0, "Exalted Angel");
    lands(&mut g, P0, "Plains", 2, false);
    let labels = option_labels(&mut g, false);
    assert!(labels.iter().any(|l| l.starts_with("Patriot:")), "{labels:?}");
    do_option(&mut g, "Patriot:", false);
    assert_eq!(epow(&g, big), 6);
    assert!(untargetable(&g, big));
    assert!(g.perm(pat).tapped);
}

#[test]
fn ultimate_magic_holy_saves_the_board_from_a_wrath() {
    let _hand = sim_core::ai::force_hand(Some(true)); // --- phase4-hand-off: a test of the hand AI
    let mut g = game();
    let a = perm(&mut g, P0, "Exalted Angel");
    let b = perm(&mut g, P0, "Helitrooper");
    let _c = perm(&mut g, P0, "Armory Automaton");
    lands(&mut g, P0, "Plains", 3, false);
    hand(&mut g, P0, &["Ultimate Magic: Holy"]);
    let c = take(&mut g, P1, "Wrath of God");
    g.player_mut(P1).hand.push(c);
    cast::cast_card(&mut g, P1, c, "hand", Ctx::default()).unwrap();
    assert!(g.perm(a).on_bf && g.perm(b).on_bf, "indestructible: both survive Wrath of God");
    assert!(in_gy(&g, P0, "Ultimate Magic: Holy"));
}

#[test]
fn ultimate_magic_holy_foretold_prevents_damage_to_you() {
    let mut g = game();
    lands(&mut g, P0, "Plains", 5, false);
    hand(&mut g, P0, &["Ultimate Magic: Holy"]);
    // foretell it (a later turn: it may be cast)
    let labels = option_labels(&mut g, true);
    assert!(labels.iter().any(|l| l == "Ultimate Magic: Holy (foretell)"), "{labels:?}");
    do_option(&mut g, "Ultimate Magic: Holy (foretell)", true);
    g.player_mut(P0).turns += 1;
    let before = g.player(P0).life;
    let h = hand_card(&g, P0, "Ultimate Magic: Holy");
    assert!(sim_core::impls::bulk_white::cast_holy(&mut g, P0, h).unwrap());
    sim_core::engine::life::lose_life(&mut g, P0, 5, Some(P1), "burn", None).unwrap();
    assert_eq!(g.player(P0).life, before, "damage is prevented this turn");
}

fn hand_card(g: &Game, p: PlayerId, name: &str) -> CardId {
    g.player(p).hand.iter().copied().find(|&c| &*g.db.get(c).name == name).unwrap()
}

// ------------------------------------------------------------------ Bronze Guardian
#[test]
fn bronze_guardian_has_power_equal_to_the_artifacts_and_gives_the_others_ward() {
    let mut g = game();
    let guard = perm(&mut g, P0, "Bronze Guardian");
    assert_eq!(epow(&g, guard), 1, "itself is an artifact");
    let sword = perm(&mut g, P0, "Sword of Kaldra");
    let _ = perm(&mut g, P0, "Helm of Kaldra");
    assert_eq!(epow(&g, guard), 3);
    assert_eq!(etgh(&g, guard), 5);
    assert_eq!(sim_core::cardcode::aura_ward(&g, sword), 2, "other artifacts have ward {{2}}");
    assert_eq!(sim_core::cardcode::aura_ward(&g, guard), 0, "its own ward is printed (the card's)");
    let creature = perm(&mut g, P0, "Helitrooper");
    assert_eq!(sim_core::cardcode::aura_ward(&g, creature), 0);
    // double strike and ward {2} on itself are the card's
    assert_eq!(g.db.get(g.perm(guard).cd.unwrap()).ward, 2);
    assert!(g.db.get(g.perm(guard).cd.unwrap()).has_kw("double strike"));
}

// ------------------------------------------------------------------ removal and the rest
#[test]
fn harsh_annotation_destroys_and_gives_the_controller_an_inkling() {
    let mut g = game();
    let t = perm(&mut g, P1, "Grave Titan");
    let _ = cast_spell(&mut g, P0, "Harsh Annotation", Some(t));
    assert!(!g.perm(t).on_bf);
    assert_eq!(tokens_named(&g, P1, "inkling"), 1);
    let ink = g
        .player(P1)
        .perms
        .iter()
        .copied()
        .find(|&m| g.perm(m).token && g.perm(m).ttypes.iter().any(|t| &**t == "inkling"))
        .unwrap();
    assert_eq!((epow(&g, ink), etgh(&g, ink)), (1, 1));
    assert!(g.perm(ink).fly);
}

#[test]
fn lay_down_arms_needs_enough_plains_and_gives_three_life() {
    let mut g = game();
    let t = perm(&mut g, P1, "Grave Titan"); // mana value 6
    lands(&mut g, P0, "Plains", 5, false);
    assert!(
        !sim_core::engine::removal::legal_targets(&g, P0, "exile", "c", false, Some(card(&g, "Lay Down Arms")))
            .contains(&t)
    );
    lands(&mut g, P0, "Plains", 1, false);
    assert!(
        sim_core::engine::removal::legal_targets(&g, P0, "exile", "c", false, Some(card(&g, "Lay Down Arms")))
            .contains(&t)
    );
    let life = g.player(P1).life;
    let _ = cast_spell(&mut g, P0, "Lay Down Arms", Some(t));
    assert!(!g.perm(t).on_bf);
    assert!(g.player(P1).exile.iter().any(|&c| &*g.db.get(c).name == "Grave Titan"));
    assert_eq!(g.player(P1).life, life + 3);
}

#[test]
fn fate_of_the_sun_cryst_costs_two_less_for_a_tapped_creature() {
    let mut g = game();
    let t = perm(&mut g, P1, "Grave Titan");
    lands(&mut g, P0, "Plains", 3, false);
    hand(&mut g, P0, &["Fate of the Sun-Cryst"]);
    // untapped: not offered for three mana
    assert!(!option_labels(&mut g, false).iter().any(|l| l.starts_with("Fate of the Sun-Cryst")));
    g.perm_mut(t).tapped = true;
    let labels = option_labels(&mut g, false);
    assert!(labels.iter().any(|l| l.starts_with("Fate of the Sun-Cryst")), "{labels:?}");
    do_option(&mut g, "Fate of the Sun-Cryst", false);
    assert!(!g.perm(t).on_bf);
    assert!(in_gy(&g, P0, "Fate of the Sun-Cryst"));
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 0, "paid {{2}}{{W}}");
}

#[test]
fn fate_of_the_sun_cryst_destroys_a_nonland_permanent_at_full_cost() {
    let mut g = game();
    let art = perm(&mut g, P1, "Sol Ring");
    let _ = cast_spell(&mut g, P0, "Fate of the Sun-Cryst", Some(art));
    assert!(!g.perm(art).on_bf);
}

#[test]
fn repel_the_darkness_taps_two_creatures_and_draws() {
    let mut g = game();
    let a = perm(&mut g, P1, "Grave Titan");
    let b = token(&mut g, P1, 3);
    let c = token(&mut g, P1, 1);
    let hand_before = g.player(P0).hand.len();
    let _ = cast_spell(&mut g, P0, "Repel the Darkness", None);
    assert!(g.perm(a).tapped && g.perm(b).tapped, "the two biggest");
    assert!(!g.perm(c).tapped);
    assert_eq!(g.player(P0).hand.len(), hand_before + 1, "drew a card (the spell left the hand)");
}

#[test]
fn recruitment_officer_finds_a_cheap_creature_in_the_top_four() {
    let mut g = game();
    let off = perm(&mut g, P0, "Recruitment Officer");
    lands(&mut g, P0, "Plains", 4, false);
    // top of the library is the end of the list: Helitrooper in the top four
    let h = take(&mut g, P0, "Helitrooper");
    let titan = take(&mut g, P0, "Stormfront Riders");
    g.player_mut(P0).library.push(titan);
    g.player_mut(P0).library.push(h);
    g.player_mut(P0).library.push(titan);
    let len = g.player(P0).library.len();
    let labels = option_labels(&mut g, true);
    assert!(labels.iter().any(|l| l.starts_with("Recruitment Officer")), "{labels:?}");
    let _ = off;
    do_option(&mut g, "Recruitment Officer", true);
    assert!(in_hand(&g, P0, "Helitrooper"));
    assert_eq!(g.player(P0).library.len(), len - 1, "the rest went to the bottom");
}

// ------------------------------------------------------------------ free-equip and backup carriers
fn ready(g: &mut Game, m: PermId) -> PermId {
    g.perm_mut(m).sick = false;
    m
}

fn wearing(g: &Game, e: PermId, m: PermId) -> bool {
    g.perm(e).attached == Some(m)
}

#[test]
fn ardenn_moves_every_equipment_onto_cloud_at_combat() {
    let mut g = game();
    g.active = Some(P0);
    let cl = perm(&mut g, P0, "Cloud, Midgar Mercenary");
    ready(&mut g, cl);
    let _ard = perm(&mut g, P0, "Ardenn, Intrepid Archaeologist");
    let bear = perm(&mut g, P0, "Kor Duelist");
    let a = perm(&mut g, P0, "Hero's Blade");
    let b = perm(&mut g, P0, "Mask of Memory");
    g.perm_mut(b).attached = Some(bear);
    sim_core::cardcode::combat_start(&mut g, P0).unwrap();
    assert!(wearing(&g, a, cl) && wearing(&g, b, cl));
}

#[test]
fn balan_has_double_strike_only_with_two_equipment() {
    let mut g = game();
    let bal = perm(&mut g, P0, "Balan, Wandering Knight");
    let a = perm(&mut g, P0, "Hero's Blade");
    let b = perm(&mut g, P0, "Mask of Memory");
    assert!(sim_core::engine::combat::first_strike(&g, bal));
    assert!(!sim_core::engine::combat::double_strike(&g, bal));
    g.perm_mut(a).attached = Some(bal);
    assert!(!sim_core::engine::combat::double_strike(&g, bal));
    g.perm_mut(b).attached = Some(bal);
    assert!(sim_core::engine::combat::double_strike(&g, bal));
}

#[test]
fn balan_attaches_all_equipment_for_two_mana_when_cloud_is_away() {
    let mut g = game();
    g.active = Some(P0);
    lands(&mut g, P0, "Plains", 2, false);
    let bal = rperm(&mut g, "Balan, Wandering Knight");
    let other = perm(&mut g, P0, "Kor Duelist");
    let a = perm(&mut g, P0, "Hero's Blade");
    let b = perm(&mut g, P0, "Mask of Memory");
    g.perm_mut(b).attached = Some(other);
    let o = option_labels(&mut g, false);
    assert!(o.iter().any(|l| l == "Balan: attach all Equipment"), "{o:?}");
    do_option(&mut g, "Balan: attach all Equipment", false);
    assert!(wearing(&g, a, bal) && wearing(&g, b, bal));
    assert!(sim_core::engine::combat::double_strike(&g, bal));
}

#[test]
fn balan_leaves_the_equipment_for_cloud_when_he_can_attack() {
    let mut g = game();
    g.active = Some(P0);
    lands(&mut g, P0, "Plains", 2, false);
    rperm(&mut g, "Cloud, Midgar Mercenary");
    rperm(&mut g, "Balan, Wandering Knight");
    perm(&mut g, P0, "Hero's Blade");
    let o = option_labels(&mut g, false);
    assert!(!o.iter().any(|l| l.starts_with("Balan")), "{o:?}");
}

#[test]
fn kor_duelist_has_double_strike_when_equipped() {
    let mut g = game();
    let d = perm(&mut g, P0, "Kor Duelist");
    let e = perm(&mut g, P0, "Hero's Blade");
    assert!(!sim_core::engine::combat::double_strike(&g, d));
    g.perm_mut(e).attached = Some(d);
    assert!(sim_core::engine::combat::double_strike(&g, d));
}

fn skyhunter_attack(g: &mut Game, sky: PermId) {
    sim_core::engine::hooks::fire_trigger(
        g,
        sim_core::hooks::Event::Attack,
        sim_core::hooks::Call::Attack { p: P0, atk: vec![sky], d: P1 },
    )
    .unwrap();
}

#[test]
fn armored_skyhunter_puts_an_equipment_from_the_top_six_onto_the_best_holder() {
    let mut g = game();
    g.active = Some(P0);
    let cl = rperm(&mut g, "Cloud, Midgar Mercenary");
    let sky = rperm(&mut g, "Armored Skyhunter");
    let before = g.player(P0).library.len();
    let top: Vec<CardId> = ["Swamp", "Plains", "Plains", "Hero's Blade", "Plains", "Plains", "Plains"]
        .iter()
        .map(|n| card(&g, n))
        .collect();
    g.player_mut(P0).library.extend(top);
    skyhunter_attack(&mut g, sky);
    let e = g.player(P0).perms.iter().copied().find(|&m| &*g.perm(m).name == "Hero's Blade").expect("on battlefield");
    assert!(wearing(&g, e, cl));
    // six looked at, five went to the bottom, the seventh card is still on top
    assert_eq!(g.player(P0).library.len(), before + 6);
    assert_eq!(&*g.db.get(*g.player(P0).library.last().unwrap()).name, "Swamp");
}

#[test]
fn armored_skyhunter_with_nothing_to_take_bottoms_all_six() {
    let mut g = game();
    g.active = Some(P0);
    let sky = rperm(&mut g, "Armored Skyhunter");
    let before = g.player(P0).library.len();
    let top: Vec<CardId> = (0..6).map(|_| card(&g, "Plains")).collect();
    g.player_mut(P0).library.extend(top);
    skyhunter_attack(&mut g, sky);
    assert_eq!(g.player(P0).library.len(), before + 6);
    assert_eq!(&*g.db.get(g.player(P0).library[0]).name, "Plains");
}

fn rperm(g: &mut Game, name: &str) -> PermId {
    let m = perm(g, P0, name);
    ready(g, m)
}
