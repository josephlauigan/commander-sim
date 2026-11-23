//! Enduring Tenacity ({2}{B}{B}, Enchantment Creature, 4/3: "Whenever you gain life, target opponent loses that much
//! life. When Enduring Tenacity dies, if it was a creature, return it to the battlefield under its owner's control. It's
//! an enchantment. (It's not a creature.)") in impls/t45_edgar.rs. Cards go onto the battlefield or into hand by name,
//! whatever the deck lists hold.

use sim_core::cards::Types;
use sim_core::engine::combat::blockable;
use sim_core::engine::values::ptypes;
use sim_core::engine::{cast, life, removal, zones};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::combos;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const TEN: &str = "Enduring Tenacity";
const SHEO: &str = "Sheoldred, the Apocalypse";

fn game() -> Game {
    table(&["sheoldred", "seph"])
}

fn game3() -> Game {
    table(&["sheoldred", "seph", "cloud"])
}

fn life_of(g: &Game, p: PlayerId) -> i32 {
    g.player(p).life
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn gy_count(g: &Game, p: PlayerId, name: &str) -> usize {
    g.player(p).gy.iter().filter(|&&c| &*g.db.get(c).name == name).count()
}

/// Enduring Tenacity as it dies the first time: back as a new permanent that is an enchantment and no creature
fn returned(g: &mut Game) -> PermId {
    let t = perm(g, P0, TEN);
    zones::die(g, t, "destroy").unwrap();
    on_bf(g, P0, TEN).expect("it returns")
}

// ------------------------------------------------------------------ the life gain trigger
#[test]
fn tenacity_drains_the_target_opponent_by_the_life_gained() {
    let mut g = game();
    perm(&mut g, P0, TEN);
    life::gain(&mut g, P0, 3).unwrap();
    assert_eq!((life_of(&g, P0), life_of(&g, P1)), (43, 37));
}

#[test]
fn tenacity_picks_one_opponent_and_the_one_it_kills() {
    let mut g = game3();
    perm(&mut g, P0, TEN);
    life::gain(&mut g, P0, 2).unwrap();
    assert_eq!(life_of(&g, P1) + life_of(&g, P2), 78, "one opponent loses it, not both");
    let mut g = game3();
    perm(&mut g, P0, TEN);
    g.player_mut(P2).life = 4;
    life::gain(&mut g, P0, 5).unwrap();
    assert_eq!(life_of(&g, P2), -1, "the opponent it kills is the target");
    assert_eq!(life_of(&g, P1), 40);
}

#[test]
fn tenacity_ignores_an_opponents_life_gain() {
    let mut g = game();
    perm(&mut g, P0, TEN);
    life::gain(&mut g, P1, 4).unwrap();
    assert_eq!((life_of(&g, P0), life_of(&g, P1)), (40, 44));
}

#[test]
fn tenacity_drains_for_a_sheoldred_draw() {
    let mut g = game();
    perm(&mut g, P0, SHEO);
    perm(&mut g, P0, TEN);
    zones::draw(&mut g, P0, 1, false).unwrap();
    assert_eq!(life_of(&g, P0), 42, "Sheoldred: gain 2 on your own draw");
    assert_eq!(life_of(&g, P1), 38, "and Enduring Tenacity makes an opponent lose that 2");
}

#[test]
fn tenacity_drains_for_a_lifelink_hit() {
    let mut g = game();
    perm(&mut g, P0, TEN);
    let src = token(&mut g, P0, 3);
    g.perm_mut(src).lifelink = true;
    life::lifelink(&mut g, src, true, 3).unwrap();
    assert_eq!((life_of(&g, P0), life_of(&g, P1)), (43, 37));
}

#[test]
fn tenacity_drains_for_gray_merchant() {
    let mut g = game();
    perm(&mut g, P0, TEN);
    let c = hand(&mut g, P0, &["Gray Merchant of Asphodel"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    // devotion to black: Tenacity {B}{B} and the Merchant {B}{B}: the Merchant drains 4, and the 4 gained drain 4 more
    assert_eq!(life_of(&g, P0), 44);
    assert_eq!(life_of(&g, P1), 40 - 4 - 4);
}

// ------------------------------------------------------------------ the death clause
#[test]
fn tenacity_is_an_enchantment_creature_before_it_dies() {
    let mut g = game();
    let t = perm(&mut g, P0, TEN);
    assert!(g.is_creature(t));
    let ty = ptypes(&g, t);
    assert!(ty.has(Types::ENCHANTMENT) && ty.has(Types::CREATURE));
}

#[test]
fn tenacity_returns_as_an_enchantment_that_is_not_a_creature() {
    let mut g = game();
    let old = perm(&mut g, P0, TEN);
    zones::die(&mut g, old, "destroy").unwrap();
    assert!(!g.perm(old).on_bf);
    let back = on_bf(&g, P0, TEN).expect("it returns to the battlefield under its owner's control");
    assert_ne!(back, old, "a new object");
    assert!(!g.is_creature(back), "It's an enchantment. (It's not a creature.)");
    assert!(ptypes(&g, back).has(Types::ENCHANTMENT));
    assert_eq!(gy_count(&g, P0, TEN), 0, "no longer in the graveyard");
    assert_eq!(g.perm(back).plus, 0, "no counters");
}

#[test]
fn the_returned_tenacity_is_no_creature_for_combat_removal_and_counts() {
    let mut g = game();
    let back = returned(&mut g);
    let attacker = token(&mut g, P1, 2);
    assert!(!blockable(&g, P0, attacker), "it can't block");
    let creatures = g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf && g.is_creature(m)).count();
    assert_eq!(creatures, 0, "not counted by an edict or a creature count");
    let targets = removal::legal_targets(&g, P1, "destroy", "c", false, None);
    assert!(!targets.contains(&back), "creature removal can't target it");
    let targets = removal::legal_targets(&g, P1, "destroy", "ce", false, None);
    assert!(targets.contains(&back), "enchantment removal can");
    assert_eq!(
        g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf && ptypes(&g, m).has(Types::ENCHANTMENT)).count(),
        1,
        "and it still counts as an enchantment"
    );
}

#[test]
fn the_returned_tenacity_still_drains_on_life_gain() {
    let mut g = game();
    returned(&mut g);
    life::gain(&mut g, P0, 5).unwrap();
    assert_eq!(life_of(&g, P1), 35, "its trigger works as an enchantment");
}

#[test]
fn a_second_death_as_an_enchantment_does_not_return_it() {
    let mut g = game();
    let back = returned(&mut g);
    zones::die(&mut g, back, "destroy").unwrap();
    assert!(on_bf(&g, P0, TEN).is_none(), "it was not a creature when it died");
    assert_eq!(gy_count(&g, P0, TEN), 1);
}

#[test]
fn exile_does_not_return_it() {
    let mut g = game();
    let t = perm(&mut g, P0, TEN);
    zones::exile_perm(&mut g, t).unwrap();
    assert!(on_bf(&g, P0, TEN).is_none());
    assert_eq!(gy_count(&g, P0, TEN), 0);
}

#[test]
fn sacrificing_it_returns_it_too() {
    let mut g = game();
    let t = perm(&mut g, P0, TEN);
    zones::die(&mut g, t, "sac").unwrap();
    let back = on_bf(&g, P0, TEN).expect("it dies, so it returns");
    assert!(!g.is_creature(back));
}

#[test]
fn only_the_creature_death_counts_for_creature_triggers() {
    // Blood Artist: "whenever Blood Artist or another creature dies, target player loses 1 life and you gain 1 life"
    let mut g = game();
    perm(&mut g, P0, "Blood Artist");
    let t = perm(&mut g, P0, TEN);
    let before = life_of(&g, P1);
    zones::die(&mut g, t, "destroy").unwrap();
    assert!(life_of(&g, P1) < before, "its first death was a creature dying");
    let back = on_bf(&g, P0, TEN).unwrap();
    let before = life_of(&g, P1);
    zones::die(&mut g, back, "destroy").unwrap();
    assert_eq!(life_of(&g, P1), before, "its second death was an enchantment dying: no creature died");
}

#[test]
fn a_dead_tenacity_returns_to_its_owner_not_the_thief() {
    let mut g = game();
    let t = perm(&mut g, P0, TEN);
    let o = g.perm(t).owner;
    zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!(g.perm(on_bf(&g, P0, TEN).unwrap()).owner, o);
}

// ------------------------------------------------------------------ the drain loop
#[test]
fn tenacity_is_a_gain_drain_in_the_exquisite_blood_loop() {
    let i = combos::combo_index("Bloodthirsty Conqueror").unwrap();
    let mut g = game();
    assert!(!(combos::all()[i].ready)(&g, P0).0);
    perm(&mut g, P0, TEN);
    assert!(!(combos::all()[i].ready)(&g, P0).0, "one half only");
    hand(&mut g, P0, &["Exquisite Blood"]);
    let (ok, keys, cast) = (combos::all()[i].ready)(&g, P0);
    assert!(ok && keys.len() == 1 && cast.len() == 1, "Tenacity out, Exquisite Blood to cast");
    let mut g = game();
    perm(&mut g, P0, TEN);
    perm(&mut g, P0, "Exquisite Blood");
    assert!((combos::all()[i].ready)(&g, P0).0, "both out");
    // the returned enchantment is still half of it
    let mut g = game();
    returned(&mut g);
    perm(&mut g, P0, "Exquisite Blood");
    assert!((combos::all()[i].ready)(&g, P0).0, "as an enchantment it still drains");
}

#[test]
fn tenacity_and_exquisite_blood_drain_in_a_loop() {
    let mut g = game();
    perm(&mut g, P0, TEN);
    perm(&mut g, P0, "Exquisite Blood");
    life::gain(&mut g, P0, 3).unwrap();
    assert!(life_of(&g, P1) <= 40 - 3 * 2, "each gain drains and each drain gains again, up to the loop guard");
    assert!(life_of(&g, P0) > 43);
}

#[test]
fn vito_is_unchanged_by_the_shared_drain() {
    let mut g = table(&["sheoldred", "seph", "cloud"]);
    perm(&mut g, P0, "Vito, Thorn of the Dusk Rose");
    life::gain(&mut g, P0, 2).unwrap();
    assert_eq!(life_of(&g, P1) + life_of(&g, P2), 78);
    assert!(g.is_creature(on_bf(&g, P0, "Vito, Thorn of the Dusk Rose").unwrap()));
}

#[test]
fn tenacity_has_card_code_and_the_card_data() {
    let g = game();
    assert!(g.registry.get(card(&g, TEN)).is_some());
    let d = g.db.get(card(&g, TEN));
    assert_eq!((d.pow, d.tgh, d.cmc), (4, 3, 4));
    assert!(d.types.has(Types::ENCHANTMENT) && d.types.has(Types::CREATURE));
}

// ------------------------------------------------------------------ the AI casts it like Vito, a turn later
#[test]
fn the_ai_casts_tenacity_from_hand_with_four_lands_and_not_with_three() {
    for (n, castable) in [(3, false), (4, true)] {
        let mut g = game();
        lands(&mut g, P0, "Swamp", n, false);
        hand(&mut g, P0, &[TEN]);
        let opts = sim_core::ai::brain::main_options(&mut g, P0, false).unwrap();
        assert_eq!(
            opts.iter().any(|o| o.label.contains(TEN)),
            castable,
            "{n} lands: {:?}",
            opts.iter().map(|o| &o.label).collect::<Vec<_>>()
        );
    }
    let mut g = game();
    lands(&mut g, P0, "Swamp", 4, false);
    hand(&mut g, P0, &[TEN]);
    sim_core::ai::brain::main(&mut g, P0, false).unwrap();
    assert!(on_bf(&g, P0, TEN).is_some(), "the main phase casts it");
}
