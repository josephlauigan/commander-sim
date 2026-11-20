//! Elesh Norn, Mother of Machines: "If a permanent entering the battlefield causes a triggered ability of a permanent
//! you control to trigger, that ability triggers an additional time. Permanents entering the battlefield don't cause
//! abilities of permanents your opponents control to trigger." The first half adds to every other "additional time"
//! effect (1 + the number of effects, CR 603.2d); the second blanks every enters-caused trigger of an opponent's
//! permanents (enters, landfall, tokens entering), but not graveyard abilities, dies or leaves triggers, or what
//! happens as a permanent enters (a name chosen, counters, an Aura's attachment, devour).

use sim_core::engine::{turn, zones};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const ELESH: &str = "Elesh Norn, Mother of Machines";
const DELNEY: &str = "Delney, Streetwise Lookout";
const PANHARM: &str = "Panharmonicon";
const VOCALIST: &str = "Starfield Vocalist";
const Y: &str = "Yoshimaru, Ever Faithful";

fn tbl() -> Game {
    table(&["heliod-mono-white-stax", "seph", "sauron"])
}

fn brago() -> Game {
    table(&["brago-azorius-blink-control", "seph", "sauron"])
}

fn land(g: &mut Game, p: PlayerId, name: &str) -> PermId {
    let c = take(g, p, name);
    turn::play_land_card(g, p, c, "plays").unwrap();
    *g.player(p).lands.last().unwrap()
}

/// the cards a Plains played from the library takes off it besides itself (Icetill Explorer's landfall mills one)
fn milled_by_a_land(g: &mut Game) -> usize {
    let c = take(g, P0, "Plains");
    let lib = g.player(P0).library.len();
    turn::play_land_card(g, P0, c, "plays").unwrap();
    lib - g.player(P0).library.len()
}

fn tokens(g: &Game, p: PlayerId) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).token).count()
}

/// the life p gains from a Soul Warden (compiled: another creature enters, gain 1) when a creature enters
fn warden_gain(g: &mut Game, p: PlayerId) -> i32 {
    let life = g.player(p).life;
    perm(g, p, "Mother of Runes");
    g.player(p).life - life
}

/// a second Elesh under the same controller, as with Mirror Gallery ("the legend rule doesn't apply"): its name is
/// changed so the rule doesn't see two of one name, and nothing it sets off as it enters is kept
fn second_elesh(g: &mut Game) -> PermId {
    g.resolving += 1;
    let m = perm(g, P0, ELESH);
    g.perm_mut(m).name = sim_core::sym::intern("Elesh Norn, Mother of Machines (second)");
    g.resolving -= 1;
    g.trig_queue.clear();
    m
}

/// Yoshimaru's counters (a card-code enters hook) after a legendary creature enters
fn yoshimaru_counters(g: &mut Game, y: PermId) -> i32 {
    let before = g.perm(y).plus;
    perm(g, P0, "Tymna the Weaver");
    g.perm(y).plus - before
}

// ------------------------------------------------------------------ her first half adds to the other doublers
#[test]
fn one_elesh_makes_two_triggers_of_a_compiled_enters_ability() {
    let mut g = tbl();
    perm(&mut g, P0, "Soul Warden");
    assert_eq!(warden_gain(&mut g, P0), 1, "control: one trigger");
    perm(&mut g, P0, ELESH);
    assert_eq!(warden_gain(&mut g, P0), 2, "1 + Elesh");
}

#[test]
fn two_elesh_make_three_triggers() {
    let mut g = tbl();
    perm(&mut g, P0, ELESH);
    perm(&mut g, P0, "Soul Warden");
    second_elesh(&mut g);
    assert_eq!(warden_gain(&mut g, P0), 3, "1 + two Elesh Norns");
}

#[test]
fn elesh_and_a_vocalist_make_three_triggers() {
    let mut g = tbl();
    perm(&mut g, P0, ELESH);
    perm(&mut g, P0, VOCALIST);
    let y = perm(&mut g, P0, Y);
    assert_eq!(yoshimaru_counters(&mut g, y), 3, "1 + Elesh + Vocalist");
}

#[test]
fn elesh_and_panharmonicon_make_three_triggers_of_a_legendary_creature() {
    let mut g = tbl();
    perm(&mut g, P0, ELESH);
    perm(&mut g, P0, PANHARM);
    let y = perm(&mut g, P0, Y);
    assert_eq!(yoshimaru_counters(&mut g, y), 3, "1 + Elesh + Panharmonicon");
}

#[test]
fn elesh_and_panharmonicon_make_three_triggers_of_a_compiled_ability() {
    let mut g = tbl();
    perm(&mut g, P0, ELESH);
    perm(&mut g, P0, PANHARM);
    perm(&mut g, P0, "Soul Warden");
    assert_eq!(warden_gain(&mut g, P0), 3, "1 + Elesh + Panharmonicon");
}

#[test]
fn elesh_and_panharmonicon_make_three_triggers_of_an_engine_enters_tag() {
    let mut g = brago();
    perm(&mut g, P0, ELESH);
    perm(&mut g, P0, PANHARM);
    let hand = g.player(P0).hand.len();
    perm(&mut g, P0, "Mulldrifter"); // the engine's enters tag: draw two, once per trigger
    assert_eq!(g.player(P0).hand.len(), hand + 6, "1 + Elesh + Panharmonicon = 3 triggers of 2 cards");
}

#[test]
fn elesh_and_delney_make_three_triggers() {
    let mut g = tbl();
    perm(&mut g, P0, ELESH);
    perm(&mut g, P0, DELNEY);
    let y = perm(&mut g, P0, Y);
    assert_eq!(yoshimaru_counters(&mut g, y), 3, "1 + Elesh + Delney");
}

#[test]
fn elesh_makes_a_card_code_enters_hook_trigger_twice() {
    let mut g = tbl();
    perm(&mut g, P0, ELESH);
    let y = perm(&mut g, P0, Y);
    assert_eq!(yoshimaru_counters(&mut g, y), 2, "1 + Elesh");
}

#[test]
fn elesh_makes_an_engine_enters_tag_trigger_twice() {
    let mut g = brago();
    perm(&mut g, P0, ELESH);
    let hand = g.player(P0).hand.len();
    perm(&mut g, P0, "Mulldrifter");
    assert_eq!(g.player(P0).hand.len(), hand + 4, "1 + Elesh = 2 triggers of 2 cards");
}

#[test]
fn elesh_entering_doubles_the_triggers_of_her_own_entry_and_blanks_the_opponents() {
    let mut g = tbl();
    perm(&mut g, P0, "Soul Warden");
    perm(&mut g, P1, "Soul Warden");
    let (mine, theirs) = (g.player(P0).life, g.player(P1).life);
    perm(&mut g, P0, ELESH);
    assert_eq!(g.player(P0).life - mine, 2, "her own entry sets off your Soul Warden twice");
    assert_eq!(g.player(P1).life - theirs, 0, "and none of the opponent's");
}

#[test]
fn elesh_makes_a_landfall_hook_trigger_twice() {
    let mut g = tbl();
    perm(&mut g, P0, ELESH);
    perm(&mut g, P0, "Scute Swarm"); // an Event::Landfall hook: a 1/1 Insect
    let before = tokens(&g, P0);
    land(&mut g, P0, "Plains");
    assert_eq!(tokens(&g, P0) - before, 2, "1 + Elesh");
}

#[test]
fn elesh_makes_a_compiled_landfall_ability_trigger_twice() {
    let mut g = tbl();
    perm(&mut g, P0, ELESH);
    perm(&mut g, P0, "Icetill Explorer"); // compiled: landfall, mill a card
    assert_eq!(milled_by_a_land(&mut g), 2, "1 + Elesh");
}

#[test]
fn elesh_makes_the_land_enters_trigger_of_a_legendary_land_twice() {
    let mut g = tbl();
    perm(&mut g, P0, ELESH);
    let y = perm(&mut g, P0, Y);
    land(&mut g, P0, "Minas Tirith");
    assert_eq!(g.perm(y).plus, 2, "the land-enters path: 1 + Elesh");
}

#[test]
fn your_own_elesh_does_not_blank_your_triggers() {
    let mut g = tbl();
    perm(&mut g, P0, "Soul Warden");
    perm(&mut g, P0, "Scute Swarm");
    perm(&mut g, P0, ELESH);
    assert_eq!(warden_gain(&mut g, P0), 2);
    let before = tokens(&g, P0);
    land(&mut g, P0, "Plains");
    assert_eq!(tokens(&g, P0) - before, 2);
}

// ------------------------------------------------------------------ her second half blanks the opponents'
#[test]
fn an_opponents_elesh_blanks_a_compiled_enters_trigger() {
    let mut g = tbl();
    perm(&mut g, P0, "Soul Warden");
    assert_eq!(warden_gain(&mut g, P0), 1, "control");
    perm(&mut g, P1, ELESH);
    assert_eq!(warden_gain(&mut g, P0), 0, "a permanent entering causes nothing to trigger");
}

#[test]
fn an_opponents_elesh_blanks_a_card_code_enters_hook() {
    let mut g = tbl();
    let y = perm(&mut g, P0, Y);
    assert_eq!(yoshimaru_counters(&mut g, y), 1, "control");
    perm(&mut g, P1, ELESH);
    assert_eq!(yoshimaru_counters(&mut g, y), 0);
}

#[test]
fn an_opponents_elesh_blanks_a_landfall_hook() {
    let mut g = tbl();
    perm(&mut g, P0, "Scute Swarm");
    let before = tokens(&g, P0);
    land(&mut g, P0, "Plains");
    assert_eq!(tokens(&g, P0) - before, 1, "control");
    perm(&mut g, P1, ELESH);
    let before = tokens(&g, P0);
    land(&mut g, P0, "Plains");
    assert_eq!(tokens(&g, P0) - before, 0);
}

#[test]
fn an_opponents_elesh_blanks_a_compiled_landfall_ability() {
    let mut g = tbl();
    perm(&mut g, P0, "Icetill Explorer");
    assert_eq!(milled_by_a_land(&mut g), 1, "control");
    perm(&mut g, P1, ELESH);
    assert_eq!(milled_by_a_land(&mut g), 0);
}

#[test]
fn an_opponents_elesh_blanks_the_land_enters_hook() {
    let mut g = tbl();
    let y = perm(&mut g, P0, Y);
    perm(&mut g, P1, ELESH);
    land(&mut g, P0, "Minas Tirith");
    assert_eq!(g.perm(y).plus, 0);
}

#[test]
fn an_opponents_elesh_blanks_felidar_retreat() {
    let mut g = tbl();
    perm(&mut g, P0, "Felidar Retreat");
    let before = tokens(&g, P0);
    land(&mut g, P0, "Plains");
    assert_eq!(tokens(&g, P0) - before, 1, "control: a 2/2 Cat");
    perm(&mut g, P1, ELESH);
    let before = tokens(&g, P0);
    land(&mut g, P0, "Plains");
    assert_eq!(tokens(&g, P0) - before, 0);
}

#[test]
fn an_opponents_elesh_blanks_a_tokens_enter_hook() {
    let mut g = table(&["alela", "cloud", "sauron"]);
    perm(&mut g, P0, "Plumecreed Mentor");
    let page = perm(&mut g, P0, "Page, Loose Leaf");
    let flier = |g: &mut Game| {
        let spec = zones::Tokens { sick: false, fly: true, tgh: Some(1), ..zones::Tokens::new(1, 1) };
        zones::make_tokens(g, P0, spec).unwrap();
    };
    flier(&mut g);
    assert_eq!(g.perm(page).plus, 1, "control: a flier enters, a counter on a creature without flying");
    perm(&mut g, P1, ELESH);
    flier(&mut g);
    assert_eq!(g.perm(page).plus, 1, "no second counter");
}

#[test]
fn an_opponents_elesh_does_not_blank_a_graveyard_landfall_ability() {
    let mut g = tbl();
    perm(&mut g, P1, ELESH);
    let b = card(&g, "Bloodghast");
    g.player_mut(P0).gy.push(b);
    land(&mut g, P0, "Plains");
    assert!(!g.player(P0).gy.contains(&b), "Bloodghast returns: it is a card's ability, not a permanent's");
}

#[test]
fn an_opponents_elesh_does_not_blank_a_dies_trigger() {
    let mut g = tbl();
    perm(&mut g, P1, ELESH);
    perm(&mut g, P0, "Cruel Celebrant"); // whenever it or another creature of yours dies, each opponent loses 1
    let t = token(&mut g, P0, 1);
    let life = g.player(P1).life;
    zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!(life - g.player(P1).life, 1);
}

#[test]
fn an_opponents_elesh_does_not_blank_a_leaves_trigger() {
    let mut g = tbl();
    perm(&mut g, P1, ELESH);
    perm(&mut g, P0, "Dour Port-Mage"); // whenever another creature you control leaves without dying, draw a card
    let t = token(&mut g, P0, 1);
    let hand = g.player(P0).hand.len();
    zones::bounce(&mut g, t).unwrap();
    assert_eq!(g.player(P0).hand.len(), hand + 1);
}

// ------------------------------------------------------------------ and what happens as a permanent enters stays
#[test]
fn an_opponents_elesh_does_not_stop_disruptor_flute_naming_a_card() {
    let mut g = tbl();
    perm(&mut g, P1, ELESH);
    perm(&mut g, P1, "Tezzeret, Cruel Captain");
    g.flutes.clear();
    let flute = perm(&mut g, P0, "Disruptor Flute");
    assert!(g.flutes.iter().any(|&(m, _)| m == flute), "it names a card as it enters");
}

#[test]
fn an_opponents_elesh_does_not_stop_devour() {
    let mut g = tbl();
    perm(&mut g, P1, ELESH);
    token(&mut g, P0, 1);
    token(&mut g, P0, 1);
    let m = perm(&mut g, P0, "Mycoloth"); // devour 2
    assert_eq!(tokens(&g, P0), 0, "both Tokens are eaten");
    assert_eq!(g.perm(m).plus, 4, "two +1/+1 counters for each");
}

#[test]
fn an_opponents_elesh_does_not_stop_an_aura_attaching() {
    let mut g = tbl();
    perm(&mut g, P1, ELESH);
    let a = perm(&mut g, P0, "Arrest");
    assert!(g.perm(a).attached.is_some(), "Arrest attaches as it enters");
}

#[test]
fn an_opponents_elesh_does_not_stop_an_as_it_enters_hook() {
    let mut g = tbl();
    perm(&mut g, P1, ELESH);
    g.entering_spent = 6;
    let c = perm(&mut g, P0, "Chalice of the Void"); // enters with X charge counters: half the mana spent
    assert_eq!(g.perm(c).data.get(DataKey::Charge), Some(&Val::Int(3)));
}

// ------------------------------------------------------------------ a real enters trigger run at once is blanked
#[test]
fn an_opponents_elesh_blanks_an_enters_trigger_that_runs_at_once() {
    let mut g = tbl();
    perm(&mut g, P1, ELESH);
    g.player_mut(P0).library.clear();
    perm(&mut g, P0, "Thassa's Oracle"); // when it enters, look at the top X: win with an empty library
    assert!(!g.over, "no enters trigger, no win");
}

#[test]
fn thassas_oracle_wins_without_an_elesh() {
    let mut g = tbl();
    g.player_mut(P0).library.clear();
    perm(&mut g, P0, "Thassa's Oracle");
    assert!(g.over, "control");
}

// ------------------------------------------------------------------ a doubler or blanker that is not in play
#[test]
fn a_phased_out_elesh_neither_doubles_nor_blanks() {
    let mut g = tbl();
    perm(&mut g, P0, "Soul Warden");
    let mine = perm(&mut g, P0, ELESH);
    perm(&mut g, P1, "Soul Warden");
    let theirs = perm(&mut g, P1, ELESH);
    g.perm_mut(mine).phased = true;
    g.perm_mut(theirs).phased = true;
    assert_eq!(warden_gain(&mut g, P0), 1, "no extra trigger of yours, and the opponent's phased Elesh blanks nothing");
    let life = g.player(P1).life;
    perm(&mut g, P1, "Mother of Runes");
    assert_eq!(g.player(P1).life - life, 1);
}

#[test]
fn an_elesh_that_has_lost_its_abilities_neither_doubles_nor_blanks() {
    let mut g = tbl();
    perm(&mut g, P0, "Soul Warden");
    let mine = perm(&mut g, P0, ELESH);
    perm(&mut g, P0, "Scute Swarm");
    let theirs = perm(&mut g, P1, ELESH);
    g.perm_mut(mine).neutered = true;
    g.perm_mut(theirs).neutered = true;
    assert_eq!(warden_gain(&mut g, P0), 1);
    let before = tokens(&g, P0);
    land(&mut g, P0, "Plains");
    assert_eq!(tokens(&g, P0) - before, 1, "a Landfall hook of yours: once, and not blanked");
}

// ------------------------------------------------------------------ Hushbringer
#[test]
fn hushbringer_and_elesh_on_a_creature_entering_give_no_triggers() {
    let mut g = tbl();
    perm(&mut g, P0, "Soul Warden");
    perm(&mut g, P0, ELESH);
    perm(&mut g, P1, "Hushbringer");
    assert_eq!(warden_gain(&mut g, P0), 0, "a creature entering sets off nothing, twice or not");
}

// ================================================================== only an enters-caused trigger is blanked or added to
// what an opponent's Elesh leaves alone: the replacement and as-enters parts of a card, and a Saga's lore counter
fn elesh_opp(g: &mut Game) {
    perm(g, P1, ELESH);
}

#[test]
fn an_opponents_elesh_does_not_make_moss_diamond_enter_untapped() {
    let mut g = tbl();
    elesh_opp(&mut g);
    let m = perm(&mut g, P0, "Moss Diamond");
    assert!(g.perm(m).tapped, "enters tapped is not a trigger");
}

#[test]
fn an_opponents_elesh_does_not_remove_the_mind_stones_indestructible() {
    let mut g = tbl();
    elesh_opp(&mut g);
    let m = perm(&mut g, P0, "The Mind Stone");
    assert!(g.perm(m).data.truthy(DataKey::Indestr));
}

#[test]
fn an_opponents_elesh_does_not_stop_phyrexian_revoker_naming_a_card() {
    let mut g = tbl();
    elesh_opp(&mut g);
    perm(&mut g, P1, "Tezzeret, Cruel Captain");
    let r = perm(&mut g, P0, "Phyrexian Revoker");
    assert!(g.perm(r).data.get(DataKey::Imprint).is_some(), "as it enters, choose a name");
}

#[test]
fn an_opponents_elesh_does_not_stop_freestrider_commandos_counters() {
    let mut g = tbl();
    elesh_opp(&mut g);
    let m = perm(&mut g, P0, "Freestrider Commando");
    assert_eq!(g.perm(m).plus, 2, "enters with counters is a replacement");
}

#[test]
fn an_opponents_elesh_does_not_stop_a_creature_entering_prepared() {
    let mut g = tbl();
    elesh_opp(&mut g);
    let a = perm(&mut g, P0, "Skycoach Conductor // All Aboard");
    let b = perm(&mut g, P0, "Studious First-Year");
    assert!(g.perm(a).data.truthy(DataKey::Prepared));
    assert!(g.perm(b).data.truthy(DataKey::Prepared));
}

#[test]
fn an_opponents_elesh_does_not_take_tome_of_legends_page_counter() {
    let mut g = tbl();
    elesh_opp(&mut g);
    let t = perm(&mut g, P0, "Tome of Legends");
    assert_eq!(g.perm(t).data.int(DataKey::Counters), 1, "it enters with a page counter");
}

#[test]
fn your_own_elesh_does_not_copy_what_happens_as_a_permanent_enters() {
    let mut g = tbl();
    perm(&mut g, P0, ELESH);
    let f = perm(&mut g, P0, "Freestrider Commando");
    assert_eq!(g.perm(f).plus, 2, "one replacement, not an additional trigger");
    let t = perm(&mut g, P0, "Tome of Legends");
    assert_eq!(g.perm(t).data.int(DataKey::Counters), 1);
}

fn chapters(g: &Game) -> i64 {
    g.player(P0).stats.get(&sim_core::sym::intern("t0a_chapters")).copied().unwrap_or(0)
}

#[test]
fn a_sagas_lore_counter_is_a_game_rule_not_an_enters_trigger_so_elesh_neither_blanks_nor_doubles_it() {
    // CR 714.3a: the lore counter is put as it enters; chapter I triggers because it was put (714.2b)
    let mut g = tbl();
    elesh_opp(&mut g);
    perm(&mut g, P0, "The Binding of the Titans");
    assert_eq!(chapters(&g), 1, "an opponent's Elesh does not blank chapter I");
    let mut g = tbl();
    perm(&mut g, P0, ELESH);
    perm(&mut g, P0, "The Binding of the Titans");
    assert_eq!(chapters(&g), 1, "your own Elesh does not run it twice");
}

#[test]
fn panharmonicon_adds_a_trigger_to_a_treasure_entering() {
    let mut g = tbl();
    perm(&mut g, P0, PANHARM);
    perm(&mut g, P0, "Reckless Fireweaver");
    let before = g.player(P1).life;
    zones::add_treasure(&mut g, P0, 1).unwrap();
    assert_eq!(before - g.player(P1).life, 2, "an artifact token entering: 1 + Panharmonicon");
}

#[test]
fn an_opponents_elesh_blanks_weftstalker_ardent_on_a_treasure() {
    let mut g = tbl();
    perm(&mut g, P0, "Weftstalker Ardent");
    let before = g.player(P1).life;
    zones::add_treasure(&mut g, P0, 1).unwrap();
    assert_eq!(before - g.player(P1).life, 1, "control");
    elesh_opp(&mut g);
    let before = g.player(P1).life;
    zones::add_treasure(&mut g, P0, 1).unwrap();
    assert_eq!(before - g.player(P1).life, 0);
}

// ------------------------------------------------------------------ ported from the adversarial review
#[test]
fn opponent_elesh_blanks_mithril_coat_attachment() {
    let mut g = tbl();
    let legend = perm(&mut g, P0, "Yoshimaru, Ever Faithful");
    elesh_opp(&mut g);
    let coat = perm(&mut g, P0, "Mithril Coat");
    assert_eq!(g.perm(coat).attached, None, "Mithril Coat's When-enters trigger is blanked; target was {legend:?}");
}

#[test]
fn blanked_mithril_coat_must_not_prompt_for_target() {
    use sim_core::control::Answer;
    use sim_core::human::script::ScriptHuman;
    use std::sync::Arc;
    let mut g = tbl();
    perm(&mut g, P0, Y);
    perm(&mut g, P0, "Tymna the Weaver");
    elesh_opp(&mut g);
    let h = Arc::new(ScriptHuman::from_fn(|r| {
        if r.kind == "target" { Answer::Pick(0) } else { Answer::Action(serde_json::json!({"do": "pass"})) }
    }));
    h.set_autopass("all");
    g.humans.add(P0, h.clone());
    perm(&mut g, P0, "Mithril Coat");
    assert_eq!(h.asked().iter().filter(|r| r.prompt.contains("Mithril Coat")).count(), 0);
}

#[test]
fn opponent_elesh_does_not_remove_genesis_hydra_as_enters_counters() {
    let mut g = tbl();
    elesh_opp(&mut g);
    g.last_x = 4;
    let c = take(&mut g, P0, "Genesis Hydra");
    let hydra = zones::enter(&mut g, P0, c, zones::Enter { was_cast: true, ..Default::default() }).unwrap();
    assert_eq!(g.perm(hydra).plus, 4, "enters with X counters is a replacement, not an enters trigger");
}

#[test]
fn opponent_elesh_blanks_artifact_token_enter_trigger() {
    let mut g = tbl();
    perm(&mut g, P0, "Reckless Fireweaver");
    elesh_opp(&mut g);
    let before = g.player(P1).life;
    zones::add_treasure(&mut g, P0, 1).unwrap();
    assert_eq!(before - g.player(P1).life, 0, "artifact enters trigger must be blanked");
}

#[test]
fn own_elesh_doubles_artifact_token_enter_trigger() {
    let mut g = tbl();
    perm(&mut g, P0, ELESH);
    perm(&mut g, P0, "Reckless Fireweaver");
    let before = g.player(P1).life;
    zones::add_treasure(&mut g, P0, 1).unwrap();
    assert_eq!(before - g.player(P1).life, 2, "an artifact entering triggers an additional time");
}

fn aura_draw_blank(name: &str) {
    let mut g = tbl();
    let host = perm(&mut g, P0, "Mother of Runes");
    elesh_opp(&mut g);
    let before = g.player(P0).hand.len();
    g.attach_to = Some(host);
    let aura = perm(&mut g, P0, name);
    assert_eq!(g.perm(aura).attached, Some(host), "attachment is unaffected");
    assert_eq!(g.player(P0).hand.len(), before, "{name}'s enters trigger must be blanked");
}

#[test]
fn opponent_elesh_blanks_angelic_gift_draw() {
    aura_draw_blank("Angelic Gift");
}
#[test]
fn opponent_elesh_blanks_feather_of_flight_draw() {
    aura_draw_blank("Feather of Flight");
}
#[test]
fn opponent_elesh_blanks_sages_reverie_draw() {
    aura_draw_blank("Sage's Reverie");
}
#[test]
fn opponent_elesh_blanks_unquestioned_authority_draw() {
    aura_draw_blank("Unquestioned Authority");
}
#[test]
fn opponent_elesh_blanks_rousing_read_draw() {
    aura_draw_blank("Rousing Read");
}

#[test]
fn opponent_elesh_blanks_cartouche_token() {
    let mut g = tbl();
    let host = perm(&mut g, P0, "Mother of Runes");
    elesh_opp(&mut g);
    g.attach_to = Some(host);
    let aura = perm(&mut g, P0, "Cartouche of Solidarity");
    assert_eq!(g.perm(aura).attached, Some(host));
    assert_eq!(tokens(&g, P0), 0);
}

fn aura_exile_blank(name: &str) {
    let mut g = tbl();
    let host = perm(&mut g, P0, "Mother of Runes");
    let elesh = perm(&mut g, P1, ELESH);
    g.attach_to = Some(host);
    let aura = perm(&mut g, P0, name);
    assert_eq!(g.perm(aura).attached, Some(host));
    assert!(g.perm(elesh).on_bf, "{name}'s triggered exile must be blanked");
}

#[test]
fn opponent_elesh_blanks_chains_of_custody_exile() {
    aura_exile_blank("Chains of Custody");
}
#[test]
fn opponent_elesh_blanks_sheltered_by_ghosts_exile() {
    aura_exile_blank("Sheltered by Ghosts");
}

#[test]
fn opponent_elesh_blanks_mantle_of_ancients_return() {
    let mut g = tbl();
    let host = perm(&mut g, P0, "Mother of Runes");
    elesh_opp(&mut g);
    let equipment = take(&mut g, P0, "Mask of Memory");
    g.player_mut(P0).gy.push(equipment);
    g.attach_to = Some(host);
    perm(&mut g, P0, "Mantle of the Ancients");
    assert!(g.player(P0).gy.contains(&equipment), "Mantle's return trigger must be blanked");
}

#[test]
fn opponent_elesh_blanks_abduction_untap_but_not_control() {
    let mut g = tbl();
    let elesh = perm(&mut g, P1, ELESH);
    let boots = perm(&mut g, P1, "Swiftfoot Boots");
    g.perm_mut(boots).attached = Some(elesh);
    let target = perm(&mut g, P1, "Grave Titan");
    g.perm_mut(target).tapped = true;
    let aura = perm(&mut g, P0, "Abduction");
    assert_eq!(g.perm(aura).attached, Some(target));
    assert_eq!(g.perm(target).owner, P0);
    assert!(g.perm(target).tapped, "Abduction's When-enters untap trigger is blanked");
}

#[test]
fn opponent_elesh_blanks_meathook_massacre() {
    let mut g = tbl();
    let elesh = perm(&mut g, P1, ELESH);
    let target = perm(&mut g, P1, "Mother of Runes");
    lands(&mut g, P0, "Swamp", 8, false);
    let c = take(&mut g, P0, "The Meathook Massacre");
    zones::enter(&mut g, P0, c, zones::Enter { was_cast: true, ..Default::default() }).unwrap();
    assert!(g.perm(target).on_bf, "Meathook's enters -X/-X is blanked");
    assert!(g.perm(elesh).on_bf);
}

#[test]
fn opponent_elesh_blanks_socialites_own_enters_trigger_but_keeps_the_replacement() {
    let mut g = tbl();
    elesh_opp(&mut g);
    let old = perm(&mut g, P0, "Viscera Seer");
    let st = g.turn_stamp();
    g.player_mut(P1).lost_turn = Some((st, 1));
    let socialite = perm(&mut g, P0, "Vampire Socialite");
    assert_eq!(g.perm(old).plus, 0, "Socialite's own When-enters trigger is blanked");
    assert!(g.perm(socialite).on_bf);
}

#[test]
fn opponent_elesh_keeps_socialite_token_replacement() {
    let mut g = tbl();
    elesh_opp(&mut g);
    perm(&mut g, P0, "Vampire Socialite");
    let st = g.turn_stamp();
    g.player_mut(P1).lost_turn = Some((st, 1));
    let spec = zones::Tokens { types: vec!["vampire"], ..zones::Tokens::new(1, 1) };
    let t = zones::make_tokens(&mut g, P0, spec).unwrap()[0];
    assert_eq!(g.perm(t).plus, 1, "each other Vampire enters with a counter is a replacement");
}

#[test]
fn tome_of_legends_commander_page_is_blanked_by_an_opponents_elesh() {
    use sim_core::engine::hooks::fire_trigger;
    use sim_core::hooks::{Call, Event};
    let mut g = tbl();
    let t = perm(&mut g, P0, "Tome of Legends");
    let cmd = perm(&mut g, P0, "Mother of Runes");
    g.perm_mut(cmd).is_cmd = true;
    fire_trigger(&mut g, Event::Etb, Call::Etb { p: P0, m: cmd }).unwrap();
    assert_eq!(g.perm(t).data.int(DataKey::Counters), 2, "control: your commander entering adds a page");
    elesh_opp(&mut g);
    fire_trigger(&mut g, Event::Etb, Call::Etb { p: P0, m: cmd }).unwrap();
    assert_eq!(g.perm(t).data.int(DataKey::Counters), 2, "an opponent's Elesh blanks that trigger");
}

// ================================================================== round 3 (ported from the adversarial review)
/// a permanent put onto the battlefield with no enters event (a fixture for firing one by hand)
fn raw(g: &mut Game, p: PlayerId, name: &str) -> PermId {
    let cd = card(g, name);
    let m = g.new_perm(p, Some(cd), "", 0, 0);
    g.perm_mut(m).on_bf = true;
    g.player_mut(p).perms.push(m);
    g.hooks.push(m);
    m
}

/// your commander enters: the enters event, fired by hand
fn commander_event(g: &mut Game) {
    use sim_core::engine::hooks::fire_trigger;
    use sim_core::hooks::{Call, Event};
    let m = raw(g, P0, "Mother of Runes"); // a creature: Panharmonicon counts it (Heliod is not one without devotion)
    g.perm_mut(m).is_cmd = true;
    fire_trigger(g, Event::Etb, Call::Etb { p: P0, m }).unwrap();
}

fn pages(g: &Game, m: PermId) -> i64 {
    g.perm(m).data.int(DataKey::Counters)
}

fn throne_centaur(g: &mut Game) {
    let throne = perm(g, P0, "Roaming Throne");
    g.perm_mut(throne).data.set(DataKey::Ctype, Val::Str("centaur"));
}

// ------------------------------------------------------------------ Tome of Legends: a replacement and a trigger
#[test]
fn tomes_commander_trigger_still_doubles_with_panharmonicon() {
    let mut g = tbl();
    perm(&mut g, P0, PANHARM);
    let tome = perm(&mut g, P0, "Tome of Legends");
    commander_event(&mut g);
    assert_eq!(pages(&g, tome), 3, "its page as it enters, then 1 + Panharmonicon for the commander");
}

#[test]
fn tomes_commander_trigger_still_doubles_with_a_vocalist() {
    let mut g = tbl();
    perm(&mut g, P0, VOCALIST);
    let tome = perm(&mut g, P0, "Tome of Legends");
    commander_event(&mut g);
    assert_eq!(pages(&g, tome), 3);
}

#[test]
fn tomes_blanking_is_decided_when_the_commander_enters_not_when_the_trigger_resolves() {
    let mut g = tbl();
    let tome = perm(&mut g, P0, "Tome of Legends");
    g.resolving = 1;
    commander_event(&mut g);
    raw(&mut g, P1, ELESH);
    g.resolving = 0;
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert_eq!(pages(&g, tome), 2, "an Elesh that came later cannot erase a trigger that already happened");
}

#[test]
fn a_blanked_tome_trigger_does_not_come_back_when_elesh_leaves() {
    let mut g = tbl();
    let tome = perm(&mut g, P0, "Tome of Legends");
    let elesh = perm(&mut g, P1, ELESH);
    g.resolving = 1;
    commander_event(&mut g);
    zones::bounce(&mut g, elesh).unwrap();
    g.resolving = 0;
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert_eq!(pages(&g, tome), 1, "the commander entered under Elesh: no page");
}

#[test]
fn tomes_commander_trigger_adds_up_with_elesh_panharmonicon_and_a_vocalist() {
    let mut g = tbl();
    for s in [ELESH, PANHARM, VOCALIST] {
        perm(&mut g, P0, s);
    }
    let tome = perm(&mut g, P0, "Tome of Legends");
    commander_event(&mut g);
    assert_eq!(pages(&g, tome), 5, "one page as it enters + 1 + three additional triggers");
}

// ------------------------------------------------------------------ setup is not a triggered ability: no source copies
#[test]
fn freestrider_is_not_copied_by_roaming_throne() {
    let mut g = tbl();
    throne_centaur(&mut g);
    let m = perm(&mut g, P0, "Freestrider Commando");
    assert_eq!(g.perm(m).plus, 2, "enters WITH counters is not a triggered ability");
}

#[test]
fn freestrider_is_not_copied_by_delney() {
    use sim_core::engine::hooks::fire_trigger;
    use sim_core::hooks::{Call, Event};
    let mut g = tbl();
    perm(&mut g, P0, DELNEY);
    let m = raw(&mut g, P0, "Freestrider Commando");
    g.perm_mut(m).eot_pt = (-2, 0);
    fire_trigger(&mut g, Event::Etb, Call::Etb { p: P0, m }).unwrap();
    assert_eq!(g.perm(m).plus, 2, "Delney must not copy a replacement");
}

#[test]
fn freestrider_is_not_copied_by_roaming_throne_under_an_opponents_elesh() {
    let mut g = tbl();
    elesh_opp(&mut g);
    throne_centaur(&mut g);
    let m = perm(&mut g, P0, "Freestrider Commando");
    assert_eq!(g.perm(m).plus, 2);
}

#[test]
fn a_creature_saga_still_triggers_twice_under_roaming_throne() {
    // chapter I of a creature Saga is a triggered ability of a creature: Throne's additional trigger is legal
    let mut g = tbl();
    let t = perm(&mut g, P0, "Roaming Throne");
    g.perm_mut(t).data.set(DataKey::Ctype, Val::Str("dragon"));
    perm(&mut g, P0, "Summon: Bahamut");
    let n = g.player(P0).stats.get(&sim_core::sym::intern("bahamut_chapters")).copied().unwrap_or(0);
    assert_eq!(n, 2, "chapter I: 1 + Throne");
}

// ------------------------------------------------------------------ End-Raze Forerunners: the creatures there now
#[test]
fn end_raze_pumps_the_other_creatures_you_control_once_per_trigger() {
    let mut g = tbl();
    perm(&mut g, P0, ELESH);
    let other = perm(&mut g, P0, "Mother of Runes");
    let end = perm(&mut g, P0, "End-Raze Forerunners");
    assert_eq!(g.perm(other).eot_pt, (4, 4), "+2/+2 for each of two triggers");
    assert_eq!(g.perm(end).eot_pt, (0, 0), "other creatures only");
    let later = perm(&mut g, P0, "Mother of Runes");
    assert_eq!(g.perm(later).eot_pt, (0, 0), "the recipients are fixed as it resolves");
}

#[test]
fn an_opponents_elesh_blanks_the_end_raze_pump() {
    let mut g = tbl();
    let other = perm(&mut g, P0, "Mother of Runes");
    elesh_opp(&mut g);
    perm(&mut g, P0, "End-Raze Forerunners");
    assert_eq!(g.perm(other).eot_pt, (0, 0));
}

#[test]
fn end_raze_two_triggers_add_toughness_not_only_power() {
    let mut g = tbl();
    perm(&mut g, P0, VOCALIST);
    let other = perm(&mut g, P0, "Mother of Runes");
    let end = perm(&mut g, P0, "End-Raze Forerunners");
    assert_eq!(sim_core::engine::chars::toughness(&g, other), 5, "1 + 4");
    assert_eq!(sim_core::engine::chars::power(&g, end), 3, "the source is not pumped");
}

#[test]
fn end_raze_other_creatures_gain_vigilance_and_trample() {
    let mut g = tbl();
    let other = perm(&mut g, P0, "Mother of Runes");
    perm(&mut g, P0, "End-Raze Forerunners");
    assert!(sim_core::dsl::has_kw(&g, other, "vigilance"));
    assert!(sim_core::dsl::has_kw(&g, other, "trample"));
}

// ------------------------------------------------------------------ a static hidden in a trigger's hook
#[test]
fn righteous_valkyries_anthem_survives_an_opponents_elesh() {
    let mut g = tbl();
    g.player_mut(P0).life = 50;
    let other = perm(&mut g, P0, "Mother of Runes");
    elesh_opp(&mut g);
    perm(&mut g, P0, "Righteous Valkyrie");
    assert_eq!(sim_core::engine::chars::power(&g, other), 3, "+2/+2 above starting life + 7 is a static");
}

#[test]
fn boons_blessing_counter_anthem_survives_an_opponents_elesh() {
    let mut g = tbl();
    let other = perm(&mut g, P0, "Mother of Runes");
    elesh_opp(&mut g);
    let boon = perm(&mut g, P0, "Boon of the Spirit Realm");
    assert_eq!(pages(&g, boon), 0, "its constellation trigger is blanked");
    g.perm_mut(boon).data.set(DataKey::Counters, Val::Int(2));
    assert_eq!(sim_core::engine::chars::power(&g, other), 3, "the anthem is a static");
}

// ------------------------------------------------------------------ Invasion of Ikoria
#[test]
fn an_opponents_elesh_blanks_invasion_of_ikorias_search_but_not_its_defense() {
    let mut g = tbl();
    elesh_opp(&mut g);
    let creature = take(&mut g, P0, "Elvish Mystic");
    g.player_mut(P0).gy.push(creature);
    let spell = take(&mut g, P0, "Invasion of Ikoria // Zilortha, Apex of Ikoria");
    sim_core::engine::cast::resolve(&mut g, P0, spell, &sim_core::state::Ctx { x: 1, ..Default::default() }, "hand")
        .unwrap();
    let battle = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).cd == Some(spell)).unwrap();
    assert_eq!(sim_core::impls::card_models3::defense(&g, battle), 6, "defense counters are setup");
    assert!(sim_core::impls::card_models3::protector(&g, battle).is_some(), "the protector is chosen as it enters");
    assert!(g.player(P0).gy.contains(&creature), "the Siege's enters search is blanked");
}

// ------------------------------------------------------------------ lands
fn land_drop(g: &mut Game, p: PlayerId, name: &str) -> PermId {
    let c = take(g, p, name);
    let before = g.perms.len();
    turn::play_land_card(g, p, c, "plays").unwrap();
    (before..g.perms.len()).map(|i| PermId(i as u32)).find(|&m| g.perm(m).cd == Some(c)).unwrap()
}

#[test]
fn an_opponents_elesh_blanks_bojuka_bogs_exile() {
    let mut g = tbl();
    elesh_opp(&mut g);
    let c = take(&mut g, P1, "Grave Titan");
    g.player_mut(P1).gy.push(c);
    land_drop(&mut g, P0, "Bojuka Bog");
    assert!(g.player(P1).gy.contains(&c));
}

#[test]
fn an_opponents_elesh_blanks_aether_hubs_energy() {
    let mut g = tbl();
    elesh_opp(&mut g);
    let before = g.player(P0).energy;
    land_drop(&mut g, P0, "Aether Hub");
    assert_eq!(g.player(P0).energy, before);
}

#[test]
fn an_opponents_elesh_blanks_lotus_fields_sacrifice() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 2, false);
    elesh_opp(&mut g);
    land_drop(&mut g, P0, "Lotus Field");
    assert_eq!(g.player(P0).lands.len(), 3);
}

#[test]
fn an_opponents_elesh_blanks_mortuary_mires_return() {
    let mut g = tbl();
    elesh_opp(&mut g);
    let c = take(&mut g, P0, "Mother of Runes");
    g.player_mut(P0).gy.push(c);
    land_drop(&mut g, P0, "Mortuary Mire");
    assert!(g.player(P0).gy.contains(&c));
}

#[test]
fn an_opponents_elesh_blanks_mystic_sanctuarys_return() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 3, false);
    elesh_opp(&mut g);
    let c = take(&mut g, P0, "Counterspell");
    g.player_mut(P0).gy.push(c);
    land_drop(&mut g, P0, "Mystic Sanctuary");
    assert!(g.player(P0).gy.contains(&c), "the return is a trigger");
}

#[test]
fn an_opponents_elesh_blanks_the_hideaway_of_mosswort_bridge_and_windbrisk_heights() {
    for name in ["Mosswort Bridge", "Windbrisk Heights"] {
        let mut g = tbl();
        elesh_opp(&mut g);
        let land = land_drop(&mut g, P0, name);
        assert!(!g.perm(land).data.truthy(DataKey::Hidden), "{name}: Hideaway is an enters trigger");
    }
}

#[test]
fn an_opponents_elesh_blanks_petrified_hamlets_naming() {
    let mut g = tbl();
    lands(&mut g, P1, "War Room", 1, false);
    elesh_opp(&mut g);
    let land = land_drop(&mut g, P0, "Petrified Hamlet");
    assert!(!g.perm(land).data.truthy(DataKey::Imprint), "it says When, not As");
}

#[test]
fn an_opponents_elesh_blanks_talon_gates_phasing() {
    let mut g = tbl();
    let elesh = perm(&mut g, P1, ELESH);
    land_drop(&mut g, P0, "Talon Gates of Madara");
    assert!(!g.perm(elesh).phased);
}

#[test]
fn an_opponents_elesh_blanks_a_bouncelands_return_and_glacial_chasms_sacrifice() {
    for name in ["Izzet Boilerworks", "Glacial Chasm"] {
        let mut g = tbl();
        lands(&mut g, P0, "Island", 1, false);
        elesh_opp(&mut g);
        land_drop(&mut g, P0, name);
        assert_eq!(g.player(P0).lands.len(), 2, "{name}");
    }
}

// the lands' own enters rules stay
#[test]
fn an_opponents_elesh_leaves_urzas_sagas_lore_counter() {
    let mut g = tbl();
    elesh_opp(&mut g);
    let l = land_drop(&mut g, P0, "Urza's Saga");
    assert!(g.player(P0).sagas.iter().any(|s| s.0 == l), "a Saga's lore counter is put as it enters");
}

#[test]
fn an_opponents_elesh_leaves_mystic_sanctuary_untapped() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 3, false);
    elesh_opp(&mut g);
    let l = land_drop(&mut g, P0, "Mystic Sanctuary");
    assert!(!g.perm(l).tapped, "the untapped entry is a replacement");
}

#[test]
fn an_opponents_elesh_leaves_city_of_traitors_sacrificed_when_you_play_another_land() {
    // "when you play another land" is caused by a land being played, not by one entering
    let mut g = tbl();
    elesh_opp(&mut g);
    let city = land_drop(&mut g, P0, "City of Traitors");
    land_drop(&mut g, P0, "Plains");
    assert!(!g.player(P0).lands.contains(&city));
}

#[test]
fn an_opponents_elesh_leaves_cavern_of_souls_naming_a_type() {
    use sim_core::control::Answer;
    use sim_core::human::script::ScriptHuman;
    use std::sync::Arc;
    let mut g = tbl();
    elesh_opp(&mut g);
    g.humans.add(P0, Arc::new(ScriptHuman::from_fn(|_| Answer::Pick(0))));
    let l = land_drop(&mut g, P0, "Cavern of Souls");
    assert!(g.perm(l).data.get(DataKey::Ctype).is_some(), "as it enters, choose a creature type");
}
