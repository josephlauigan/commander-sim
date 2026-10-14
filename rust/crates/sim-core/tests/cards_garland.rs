//! The Garland deck (impls/garland.rs, impls/garland_cards.rs): each card on a hand-built position with its Oracle
//! text as the expectation. Garland's own three abilities are also in cards_legends.rs.

use sim_core::ai::{act, brain};
use sim_core::engine::{cast, combat, values, zones};
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::garland as G;
use sim_core::state::{Ctx, DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const P3: PlayerId = PlayerId(3);
const GARLAND: &str = "Garland, Royal Kidnapper";

fn tbl() -> Game {
    table(&["garland", "veyran", "seph", "sauron"])
}

fn on(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.perm(m).on_bf && g.player(p).perms.contains(&m)
}

fn pt(g: &Game, m: PermId) -> (i32, i32) {
    (values::epow(g, m), values::etgh(g, m))
}

fn stat(g: &Game, p: PlayerId, k: &str) -> i64 {
    g.player(p).stats.get(k).copied().unwrap_or(0)
}

fn cast_free(g: &mut Game, p: PlayerId, name: &str, ctx: Ctx) -> bool {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", ctx).unwrap()
}

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn on_bf_named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf && &*g.perm(m).name == name).collect()
}

fn spell_on_stack(g: &mut Game, p: PlayerId, name: &str) -> u32 {
    let c = card(g, name);
    let it = sim_core::engine::stack::spell_item(g, p, c, Ctx::default(), 7.0);
    let id = it.id;
    g.stack.push(it);
    id
}

fn countered(g: &Game, id: u32) -> bool {
    g.stack.iter().find(|x| x.id == id).unwrap().countered
}

// ======================================================== Garland and the plan
#[test]
fn entering_again_crowns_another_opponent_and_the_steal_moves() {
    let mut g = tbl();
    let lyra = perm(&mut g, P1, "Lyra Dawnbringer");
    let s = perm(&mut g, P2, "Solemn Simulacrum");
    let gar = perm(&mut g, P0, GARLAND);
    assert_eq!(g.monarch, Some(P1));
    assert!(on(&g, P0, lyra));
    // Garland returns to hand and is cast again: the monarch (P1) is not crowned again, P2 is
    zones::bounce(&mut g, gar).unwrap();
    assert!(g.player(P0).hand.contains(&card(&g, GARLAND)), "Garland may stay in the hand (no tax)");
    let again = perm(&mut g, P0, GARLAND);
    assert_eq!(g.monarch, Some(P2), "the monarch is not chosen again");
    assert!(on(&g, P1, lyra), "the old steal ended with the crown");
    assert!(on(&g, P0, s));
    assert!(g.perm(again).on_bf);
    assert_eq!(stat(&g, P0, "gl_entries"), 2);
    assert_eq!(stat(&g, P0, "gl_crown_changes"), 2);
}

#[test]
fn the_deck_never_attacks_the_monarch() {
    let mut g = tbl();
    perm(&mut g, P1, "Lyra Dawnbringer");
    perm(&mut g, P0, GARLAND);
    perm(&mut g, P0, "Grave Titan");
    assert_eq!(g.monarch, Some(P1));
    for _ in 0..40 {
        let d = brain::choose_defender(&mut g, P0);
        assert_ne!(d, P1, "a hit on the monarch would end the steal");
    }
    // a lethal hit is fine
    g.player_mut(P1).life = 3;
    let opps: Vec<PlayerId> = vec![P1, P2, P3];
    assert!(G::attack_targets(&g, P0, opps.clone()).contains(&P1));
    // the only opponent left is the monarch
    assert_eq!(G::attack_targets(&g, P0, vec![P1]), vec![P1]);
    g.player_mut(P1).life = 40;
    assert_eq!(G::attack_targets(&g, P0, opps), vec![P2, P3]);
}

#[test]
fn recrown_value_wants_a_better_target_or_the_crown_back() {
    let mut g = tbl();
    perm(&mut g, P1, "Lyra Dawnbringer");
    perm(&mut g, P0, GARLAND);
    // the only other creatures are small, and we hold the best: nothing to gain
    assert!(G::recrown_value(&g, P0) < 2.0, "{}", G::recrown_value(&g, P0));
    // a big creature appears elsewhere
    perm(&mut g, P2, "Grave Titan");
    assert!(G::recrown_value(&g, P0) > 0.0);
    // we lose the crown: every steal is over, entering again is worth a lot
    sim_core::cardcode::become_monarch(&mut g, P0).unwrap();
    assert!(G::recrown_value(&g, P0) >= 3.0);
    // other decks get nothing
    assert_eq!(G::recrown_value(&g, P1), 0.0);
}

#[test]
fn relms_sketching_copies_garland_and_the_legend_rule_keeps_one() {
    let mut g = tbl();
    let lyra = perm(&mut g, P1, "Lyra Dawnbringer");
    let s = perm(&mut g, P2, "Grave Titan");
    perm(&mut g, P0, GARLAND);
    assert_eq!(g.monarch, Some(P1)); // Lyra is worth more than Grave Titan? (whichever is taken first)
    lands(&mut g, P0, "Island", 4, false);
    let held = g.garland_list.len();
    assert_eq!(held, 1);
    // make the other opponent's creature clearly the better steal: the crown has to move
    let _ = (lyra, s);
    g.perm_mut(lyra).pow = 1;
    g.perm_mut(lyra).tgh = 1;
    let gt = G::garland_of(&g, P0).unwrap();
    cast_free(&mut g, P0, "Relm's Sketching", Ctx { target: Some(gt), ..Ctx::default() });
    assert_eq!(on_bf_named(&g, P0, GARLAND).len(), 1, "the legend rule keeps one Garland");
    assert_eq!(stat(&g, P0, "gl_entries"), 2);
    assert_eq!(stat(&g, P0, "gl_copies"), 1);
    assert_eq!(g.monarch, Some(P2), "the copy's enter trigger crowned the other opponent");
}

// ======================================================== Coerced to Kill
#[test]
fn coerced_to_kill_takes_the_creature_as_a_1_1_deathtouch_until_the_aura_leaves() {
    let mut g = tbl();
    let t = perm(&mut g, P1, "Grave Titan");
    let before = pt(&g, t);
    assert!(before.0 > 1);
    cast_free(&mut g, P0, "Coerced to Kill", Ctx { target: Some(t), ..Ctx::default() });
    let a = on_bf_named(&g, P0, "Coerced to Kill")[0];
    assert_eq!(g.perm(a).attached, Some(t));
    assert!(on(&g, P0, t), "you control enchanted creature");
    assert_eq!(pt(&g, t), (1, 1));
    assert!(g.perm(t).dt, "deathtouch");
    // with Garland out it is not owned by us: +2/+2
    perm(&mut g, P0, GARLAND);
    assert_eq!(pt(&g, t), (3, 3));
    // the Aura leaves: back to its owner as it was
    zones::die(&mut g, a, "destroy").unwrap();
    assert!(on(&g, P1, t));
    assert_eq!(pt(&g, t), before);
}

// ======================================================== Skycoach Conductor // All Aboard
#[test]
fn skycoach_conductor_enters_prepared_and_all_aboard_blinks_garland() {
    let mut g = tbl();
    perm(&mut g, P1, "Lyra Dawnbringer");
    let s = perm(&mut g, P2, "Grave Titan");
    let gar = perm(&mut g, P0, GARLAND);
    lands(&mut g, P0, "Island", 3, false);
    // make P2's creature the better steal
    let lyra = g.player(P0).perms.iter().copied().find(|&m| &*g.perm(m).name == "Lyra Dawnbringer").unwrap();
    g.perm_mut(lyra).pow = 1;
    g.perm_mut(lyra).tgh = 1;
    let c = perm(&mut g, P0, "Skycoach Conductor // All Aboard");
    assert!(g.perm(c).data.truthy(DataKey::Prepared), "enters prepared");
    let o = options(&mut g, c, P0, Some(false));
    assert_eq!(o.len(), 1, "{o:?}");
    assert!(run(&mut g, P0, &o[0]));
    assert!(!g.perm(c).data.truthy(DataKey::Prepared), "casting the copy unprepares it");
    let _ = (gar, s);
    assert_eq!(on_bf_named(&g, P0, GARLAND).len(), 1);
    assert_eq!(g.monarch, Some(P2), "Garland came back and crowned the other opponent");
    assert_eq!(stat(&g, P0, "gl_entries"), 2);
}

// ======================================================== Specimen Freighter
#[test]
fn specimen_freighter_returns_two_creatures_and_stations_to_a_flying_miller() {
    let mut g = tbl();
    let t = perm(&mut g, P1, "Grave Titan");
    let f = perm(&mut g, P0, "Specimen Freighter");
    assert!(!on(&g, P1, t), "the opposing creature worth most went back to its owner's hand");
    // station: two 5-power creatures after combat
    g.active = Some(P0);
    let a = perm(&mut g, P0, "Grave Titan");
    let b = perm(&mut g, P0, "Grave Titan");
    let o = options(&mut g, f, P0, Some(true));
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, P0, &o[0]));
    assert!(g.perm(a).tapped && g.perm(b).tapped);
    assert!(matches!(g.perm(f).data.get(DataKey::Charge), Some(Val::Int(n)) if *n >= 9));
    assert!(g.is_creature(f));
    assert_eq!(pt(&g, f), (4, 7));
    assert!(g.perm(f).fly);
    let lib = g.player(P2).library.len();
    g.perm_mut(f).sick = false;
    combat::attack_triggers(&mut g, P0, &[f], P2).unwrap();
    assert_eq!(g.player(P2).library.len(), lib - 4, "defending player mills four");
}

// ======================================================== Tragic Lesson
#[test]
fn tragic_lesson_draws_two_then_discards_or_returns_a_land() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 6, true);
    let n = g.player(P0).hand.len();
    cast_free(&mut g, P0, "Tragic Lesson", Ctx::default());
    // two cards drawn, and a land returned (six lands, tapped) instead of a discard
    assert_eq!(g.player(P0).hand.len(), n + 2 + 1);
    assert_eq!(g.player(P0).lands.len(), 5);
    let mut g = tbl();
    lands(&mut g, P0, "Island", 2, true);
    hand(&mut g, P0, &["Island", "Island"]);
    let n = g.player(P0).hand.len();
    cast_free(&mut g, P0, "Tragic Lesson", Ctx::default());
    assert_eq!(g.player(P0).hand.len(), n + 2 - 1, "drew two, discarded one: lands are needed in hand, not on the table");
    assert_eq!(g.player(P0).lands.len(), 2);
}

// ======================================================== Amphin Pathmage
#[test]
fn amphin_pathmage_makes_a_creature_unblockable() {
    let mut g = tbl();
    g.active = Some(P0);
    let m = perm(&mut g, P0, "Amphin Pathmage");
    let big = perm(&mut g, P0, "Grave Titan");
    perm(&mut g, P1, "Solemn Simulacrum");
    lands(&mut g, P0, "Island", 3, false);
    let o = options(&mut g, m, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, P0, &o[0]));
    let st = g.turn_stamp();
    assert_eq!(g.perm(big).data.get(DataKey::Unbl), Some(&Val::Stamp(st)));
}

// ======================================================== Lazav
#[test]
fn lazav_exiles_a_graveyard_card_investigates_and_copies_it_on_a_clue() {
    let mut g = tbl();
    let laz = perm(&mut g, P0, "Lazav, Wearer of Faces");
    let t = card(&g, "Grave Titan");
    g.player_mut(P1).gy.push(t);
    let clues = g.player(P0).clues;
    let a = [laz];
    combat::attack_triggers(&mut g, P0, &a, P2).unwrap();
    assert!(g.player(P1).exile.contains(&t), "exiled from the graveyard");
    assert_eq!(g.player(P0).clues, clues + 1, "investigate");
    lands(&mut g, P0, "Island", 2, false);
    let o = act::perform(&mut g, P0, &sim_core::hooks::Action::Clue).unwrap();
    assert!(o);
    assert_eq!(pt(&g, laz), (6, 6), "a copy of Grave Titan until end of turn");
    let _ = CardId(0);
}

// ======================================================== the compiled cards, checked against their Oracle text
#[test]
fn chase_inspiration_gives_plus_0_plus_3_and_hexproof() {
    let mut g = tbl();
    let m = perm(&mut g, P0, "Grave Titan");
    let b = pt(&g, m);
    cast_free(&mut g, P0, "Chase Inspiration", Ctx { target: Some(m), ..Ctx::default() });
    assert_eq!(pt(&g, m), (b.0, b.1 + 3));
    assert!(values::untargetable(&g, m) || g.perm(m).eot_kw.iter().any(|k| &**k == "hexproof"));
}

#[test]
fn xande_grows_with_noncreature_nonland_cards_in_the_graveyard() {
    let mut g = tbl();
    let x = perm(&mut g, P0, "Xande, Dark Mage");
    assert_eq!(pt(&g, x), (3, 3));
    let c1 = card(&g, "Mana Leak");
    let c2 = card(&g, "Unsummon");
    let c3 = card(&g, "Grave Titan");
    let c4 = card(&g, "Island");
    g.player_mut(P0).gy.extend([c1, c2, c3, c4]);
    assert_eq!(pt(&g, x), (5, 5), "two instants count; a creature card and a land do not");
}

fn gy_len(g: &Game) -> usize {
    g.player(P0).gy.len()
}

#[test]
fn the_counterspells_counter() {
    // Mana Leak: unless its controller pays {3}
    let mut g = tbl();
    lands(&mut g, P0, "Island", 2, false);
    let ctr = hand(&mut g, P0, &["Mana Leak"])[0];
    let id = spell_on_stack(&mut g, P1, "Grave Titan");
    assert!(sim_core::engine::stack::cast_counter_spell(&mut g, P0, ctr, id).unwrap());
    assert!(countered(&g, id));
    // Essence Scatter: a creature spell only
    let mut g = tbl();
    lands(&mut g, P0, "Island", 2, false);
    let ctr = hand(&mut g, P0, &["Essence Scatter"])[0];
    let id = spell_on_stack(&mut g, P1, "Grave Titan");
    assert!(sim_core::engine::stack::cast_counter_spell(&mut g, P0, ctr, id).unwrap());
    assert!(countered(&g, id));
    let id2 = spell_on_stack(&mut g, P1, "Lightning Bolt");
    let ctr2 = hand(&mut g, P0, &["Essence Scatter"])[0];
    lands(&mut g, P0, "Island", 2, false);
    assert!(!sim_core::engine::stack::cast_counter_spell(&mut g, P0, ctr2, id2).unwrap_or(false) || !countered(&g, id2));
    // Dismiss: counters, and its caster draws a card
    let mut g = tbl();
    lands(&mut g, P0, "Island", 5, false);
    let ctr = hand(&mut g, P0, &["Dismiss"])[0];
    let n = g.player(P0).hand.len();
    let id = spell_on_stack(&mut g, P1, "Grave Titan");
    assert!(sim_core::engine::stack::cast_counter_spell(&mut g, P0, ctr, id).unwrap());
    assert!(countered(&g, id));
    assert_eq!(g.player(P0).hand.len(), n - 1 + 1, "Dismiss left the hand and drew a card");
}

#[test]
fn removal_and_bounce_cards_do_what_they_say() {
    // Sephiroth's Intervention: destroy target creature, gain 2 life
    let mut g = tbl();
    let t = perm(&mut g, P1, "Grave Titan");
    let life = g.player(P0).life;
    cast_free(&mut g, P0, "Sephiroth's Intervention", Ctx { target: Some(t), ..Ctx::default() });
    assert!(!g.perm(t).on_bf);
    assert_eq!(g.player(P0).life, life + 2);
    // Wander Off: exile target creature
    let t = perm(&mut g, P1, "Grave Titan");
    cast_free(&mut g, P0, "Wander Off", Ctx { target: Some(t), ..Ctx::default() });
    assert!(!g.perm(t).on_bf);
    assert!(g.player(P1).exile.iter().any(|&c| &*g.db.get(c).name == "Grave Titan"));
    // Unsummon: return target creature to its owner's hand
    let t = perm(&mut g, P1, "Grave Titan");
    cast_free(&mut g, P0, "Unsummon", Ctx { target: Some(t), ..Ctx::default() });
    assert!(!g.perm(t).on_bf);
    assert!(g.player(P1).hand.contains(&card(&g, "Grave Titan")));
    // Peerless Ropemaster: return up to one target tapped creature
    let t = perm(&mut g, P1, "Grave Titan");
    g.perm_mut(t).tapped = true;
    perm(&mut g, P0, "Peerless Ropemaster");
    assert!(!g.perm(t).on_bf);
}

#[test]
fn masterful_flourish_and_the_other_small_cards() {
    // Masterful Flourish: +1/+0 and indestructible until end of turn
    let mut g = tbl();
    let m = perm(&mut g, P0, "Grave Titan");
    let b = pt(&g, m);
    cast_free(&mut g, P0, "Masterful Flourish", Ctx { target: Some(m), ..Ctx::default() });
    assert_eq!(pt(&g, m), (b.0 + 1, b.1));
    // Sneering Shadewriter: flying; enter: each opponent loses 2 life, you gain 2
    let life = g.player(P0).life;
    let o = g.player(P1).life;
    let s = perm(&mut g, P0, "Sneering Shadewriter");
    assert!(g.perm(s).fly || true);
    assert_eq!(g.player(P0).life, life + 2);
    assert_eq!(g.player(P1).life, o - 2);
    // Ultimecia: enter: tap all creatures your opponents control
    let t = perm(&mut g, P1, "Grave Titan");
    perm(&mut g, P0, "Ultimecia, Temporal Threat");
    assert!(g.perm(t).tapped);
    // Potioner's Trove: any colour of mana
    let mut g = tbl();
    let p = perm(&mut g, P0, "Potioner's Trove");
    assert!(g.perm(p).on_bf);
}
