//! AI audit phase 2, sheoldred and titania: Rooftop Assassin's damage condition, Seizan, Dark Deal, Necrosquito's
//! priority, Fblthp's {X}{G}{G} and Silhana Starfletcher's green mana.

use sim_core::engine::{cast, mana};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::sheoldred::sheoldred_prio;
use sim_core::state::{Ctx, Game};
use sim_core::tag::Tag;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn cast_hand(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
    c
}

#[test]
fn rooftop_assassin_needs_a_damaged_creature() {
    let mut g = table(&["sheoldred", "seph"]);
    let victim = perm(&mut g, P1, "Birds of Paradise");
    lands(&mut g, P0, "Swamp", 4, false);
    cast_hand(&mut g, P0, "Rooftop Assassin");
    assert!(g.perm(victim).on_bf, "an undamaged creature survives the Assassin");
    assert!(on_bf(&g, P0, "Rooftop Assassin").is_some());
}

#[test]
fn rooftop_assassin_kills_a_creature_dealt_damage_this_turn() {
    let mut g = table(&["sheoldred", "seph"]);
    let victim = perm(&mut g, P1, "Birds of Paradise");
    let other = perm(&mut g, P1, "Llanowar Elves");
    g.mark_damaged(victim);
    lands(&mut g, P0, "Swamp", 4, false);
    cast_hand(&mut g, P0, "Rooftop Assassin");
    assert!(!g.perm(victim).on_bf, "the damaged creature is destroyed");
    assert!(g.perm(other).on_bf, "the undamaged one is not");
}

#[test]
fn rooftop_assassin_damage_mark_expires_with_the_turn() {
    let mut g = table(&["sheoldred", "seph"]);
    let victim = perm(&mut g, P1, "Birds of Paradise");
    g.mark_damaged(victim);
    assert!(g.was_damaged_now(victim));
    g.round += 1;
    assert!(!g.was_damaged_now(victim));
}

#[test]
fn rooftop_assassin_is_cast_for_a_damaged_target() {
    let mut g = table(&["sheoldred", "seph"]);
    let c = hand(&mut g, P0, &["Rooftop Assassin"])[0];
    let victim = perm(&mut g, P1, "Atraxa, Grand Unifier");
    let idle = sheoldred_prio(&g, P0, c);
    g.mark_damaged(victim);
    assert!(sheoldred_prio(&g, P0, c) > idle + 20);
}

#[test]
fn seizan_comes_down_for_underworld_dreams_without_sheoldred() {
    let mut g = table(&["sheoldred", "seph"]);
    let c = hand(&mut g, P0, &["Seizan, Perverter of Truth"])[0];
    assert!(sheoldred_prio(&g, P0, c) < 60);
    perm(&mut g, P0, "Underworld Dreams");
    assert_eq!(sheoldred_prio(&g, P0, c), 80);
}

#[test]
fn necrosquito_is_a_better_cast_with_sacrifice_fodder() {
    let mut g = table(&["sheoldred", "seph"]);
    let c = hand(&mut g, P0, &["Necrosquito"])[0];
    let base = sheoldred_prio(&g, P0, c);
    assert!(base >= 55, "not last among the four-drops, got {base}");
    perm(&mut g, P0, "Carnage Altar");
    token(&mut g, P0, 1);
    assert!(sheoldred_prio(&g, P0, c) > base + 5);
}

#[test]
fn silhana_starfletcher_taps_for_green() {
    let g = table(&["titania", "seph"]);
    let d = g.db.get(card(&g, "Silhana Starfletcher"));
    assert_eq!(d.tags.str(Tag::Dork), Some("G"));
}

#[test]
fn fblthp_costs_x_gg_and_fetches_one_forest() {
    let mut g = table(&["titania", "seph"]);
    let f = card(&g, "Fblthp, Knows the Way");
    assert_eq!(g.db.get(f).cmc, 3, "{{1}}{{G}}{{G}}: X = 1");
    lands(&mut g, P0, "Forest", 2, false);
    let c = hand(&mut g, P0, &["Fblthp, Knows the Way"])[0];
    assert!(!mana::can_pay(&g, P0, 1, "GG", false), "two lands can't pay X=1");
    let _ = c;
    lands(&mut g, P0, "Forest", 1, false);
    let before = g.player(P0).hand.iter().filter(|&&x| &*g.db.get(x).name == "Forest").count();
    cast_hand_existing(&mut g, c);
    let after = g.player(P0).hand.iter().filter(|&&x| &*g.db.get(x).name == "Forest").count();
    assert_eq!(after, before + 1);
}

fn cast_hand_existing(g: &mut Game, c: CardId) {
    cast::cast_card(g, P0, c, "hand", Ctx::default()).unwrap();
}

#[test]
fn ad_nauseam_for_sheoldred_also_goes_off_in_its_own_second_main() {
    use sim_core::ai::plans;
    use sim_core::state::Step;
    let mut g = table(&["sheoldred", "seph"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    assert!(!plans::adnaus_cast_now(&g, P0));
    g.step = Step::Main2;
    assert!(plans::adnaus_cast_now(&g, P0), "empty hand: room for four");
    let mut g2 = table(&["seph", "sheoldred"]);
    g2.active = Some(P0);
    g2.step = Step::Main2;
    assert!(!plans::adnaus_cast_now(&g2, P0), "other decks keep waiting for the opposing end step");
}

#[test]
fn memory_jar_waits_for_a_punisher() {
    let mut g = table(&["sheoldred", "seph"]);
    let c = hand(&mut g, P0, &["Memory Jar"])[0];
    assert!(sheoldred_prio(&g, P0, c) < 45);
    perm(&mut g, P0, "Sheoldred, the Apocalypse");
    assert!(sheoldred_prio(&g, P0, c) > 80);
}

#[test]
fn dark_deal_is_allowed_with_three_other_cards() {
    use sim_core::hooks::Action;
    let mut g = table(&["sheoldred", "seph"]);
    perm(&mut g, P0, "Sheoldred, the Apocalypse");
    lands(&mut g, P0, "Swamp", 3, false);
    g.active = Some(P0);
    g.player_mut(P1).hand = (0..5).map(|_| card(&g, "Terror")).collect();
    let d = hand(&mut g, P0, &["Dark Deal", "Terror", "Terror", "Terror"])[0];
    let f = g.registry.get(d).unwrap().hand_options.unwrap();
    let opts = f(&mut g, d, P0, Some(false)).unwrap();
    assert_eq!(opts.len(), 1);
    assert!(matches!(opts[0].act, Some(Action::Plan { .. })));
    hand(&mut g, P0, &["Terror"]);
    assert!(f(&mut g, d, P0, Some(false)).unwrap().is_empty(), "four other cards: too many to throw away");
}

#[test]
fn worldly_tutor_fetches_a_mana_creature_early_and_a_big_body_late() {
    use sim_core::ai::decks::tutor_pick_named;
    let mut g = table(&["titania", "seph"]);
    lands(&mut g, P0, "Forest", 2, false);
    let early = tutor_pick_named(&g, P0, "cre").unwrap();
    assert!(
        matches!(&*g.db.get(early).name, "Llanowar Elves" | "Paradise Druid" | "Bugenhagen, Wise Elder" | "Silhana Starfletcher"),
        "got {}",
        g.db.get(early).name
    );
    lands(&mut g, P0, "Forest", 5, false);
    let late = tutor_pick_named(&g, P0, "cre").unwrap();
    assert!(matches!(&*g.db.get(late).name, "Ghalta the Unstoppable" | "Annoyed Altisaur" | "Cactarantula"));
}

#[test]
fn arachnogenesis_answers_three_small_attackers() {
    use sim_core::impls::titania::attack_fog;
    let mut g = table(&["titania", "seph"]);
    lands(&mut g, P0, "Forest", 3, false);
    hand(&mut g, P0, &["Arachnogenesis"]);
    let atk: Vec<PermId> = (0..3).map(|_| token(&mut g, P1, 1)).collect();
    assert!(attack_fog(&mut g, P0, P1, &atk).unwrap());
    let mut g = table(&["titania", "seph"]);
    lands(&mut g, P0, "Forest", 3, false);
    hand(&mut g, P0, &["Arachnogenesis"]);
    let atk: Vec<PermId> = (0..2).map(|_| token(&mut g, P1, 1)).collect();
    assert!(!attack_fog(&mut g, P0, P1, &atk).unwrap(), "two 1-power attackers against 40 life: held");
}
