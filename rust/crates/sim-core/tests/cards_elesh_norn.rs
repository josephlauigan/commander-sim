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
