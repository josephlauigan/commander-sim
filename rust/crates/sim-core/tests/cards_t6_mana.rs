//! Tier 6 "mana" batch (impls/t6_mana.rs): each card on a hand-built position with its Oracle text as the
//! expectation.

use sim_core::ai::act;
use sim_core::engine::pool::Kind;
use sim_core::hooks::Opt;
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn tbl() -> Game {
    table(&["rograkh-silas-turbo-breach", "godo", "dack-fayden-white-stax", "kinnan-bonder-mana-combo"])
}

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn on(g: &Game, p: PlayerId, m: PermId) -> bool {
    g.perm(m).on_bf && g.player(p).perms.contains(&m)
}

// ======================================================== Chromatic Star
#[test]
fn chromatic_star_is_not_a_free_rock_and_cracks_for_a_card() {
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 2, false);
    let s = perm(&mut g, P0, "Chromatic Star");
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 2, "Star adds no mana by itself");
    assert!(options(&mut g, s, P0, Some(false)).is_empty(), "not before combat");
    let o = options(&mut g, s, P0, Some(true));
    assert_eq!(o.len(), 1);
    let before = g.player(P0).hand.len();
    assert!(run(&mut g, P0, &o[0]));
    assert!(!on(&g, P0, s), "sacrificed");
    assert!(g.player(P0).gy.contains(&card(&g, "Chromatic Star")));
    assert_eq!(g.player(P0).hand.len(), before + 1, "drew a card");
    assert_eq!(g.player(P0).mana.get(Kind::Any), 1, "one mana of any colour");
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 2 - 1 + 1, "paid {{1}}, got one mana back");
}

#[test]
fn chromatic_star_destroyed_still_draws() {
    let mut g = tbl();
    let s = perm(&mut g, P0, "Chromatic Star");
    let before = g.player(P0).hand.len();
    sim_core::engine::zones::die(&mut g, s, "destroy").unwrap();
    assert_eq!(g.player(P0).hand.len(), before + 1);
    let _ = P1;
}

// ======================================================== Desperate Ritual
#[test]
fn desperate_ritual_adds_rrr_for_two_mana() {
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 2, false);
    let c = hand(&mut g, P0, &["Desperate Ritual"])[0];
    assert!(sim_core::engine::mana::pay(&mut g, P0, 1, "R", false).unwrap(), "{{1}}{{R}}");
    assert!(sim_core::engine::cast::cast_card(&mut g, P0, c, "hand", Default::default()).unwrap());
    assert_eq!(g.player(P0).mana.get(Kind::R), 3, "RRR");
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 3, "both Mountains were tapped for the cost");
    assert!(g.player(P0).gy.contains(&c), "an instant goes to the graveyard");
    assert!(sim_core::engine::mana::can_pay(&g, P0, 0, "RRR", false));
}

// ======================================================== Rite of Flame
#[test]
fn rite_of_flame_adds_rr_and_one_more_per_copy_in_a_graveyard() {
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 1, false);
    let c = hand(&mut g, P0, &["Rite of Flame"])[0];
    assert!(sim_core::engine::mana::pay(&mut g, P0, 0, "R", false).unwrap());
    assert!(sim_core::engine::cast::cast_card(&mut g, P0, c, "hand", Default::default()).unwrap());
    assert_eq!(g.player(P0).mana.get(Kind::R), 2, "RR with no other copy");
    // two more copies (other players' graveyards count too) make RRRR
    let again = card(&g, "Rite of Flame");
    g.player_mut(P1).gy.push(again);
    g.player_mut(P0).gy.push(again);
    g.player_mut(P0).mana.set(Kind::R, 0);
    let c2 = card(&g, "Rite of Flame");
    let mut ctx = sim_core::state::Ctx::default();
    ctx.owner = None;
    let f = g.registry.get(c2).unwrap().resolve.unwrap();
    let dest = f(&mut g, P0, c2, &ctx).unwrap();
    assert_eq!(&*dest, "gy");
    assert_eq!(g.player(P0).mana.get(Kind::R), 2 + 3, "the copy cast first plus two more in graveyards");
}

// ======================================================== Manifold Key
#[test]
fn manifold_key_untaps_a_sol_ring_for_one() {
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 1, false);
    let key = perm(&mut g, P0, "Manifold Key");
    let ring = perm(&mut g, P0, "Sol Ring");
    hand(&mut g, P0, &["Rite of Flame"]);
    g.perm_mut(ring).tapped = true;
    let o = options(&mut g, key, P0, Some(false));
    assert_eq!(o.len(), 1, "untap offer only; no creature to make unblockable");
    assert!(run(&mut g, P0, &o[0]));
    assert!(!g.perm(ring).tapped, "Sol Ring untapped");
    assert!(g.perm(key).tapped, "{{T}} is part of the cost");
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 2, "Mountain paid {{1}}; Sol Ring makes {{C}}{{C}}");
    assert!(options(&mut g, key, P0, Some(false)).is_empty(), "the Key is tapped");
}

#[test]
fn manifold_key_does_not_untap_a_one_mana_rock() {
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 2, false);
    let key = perm(&mut g, P0, "Manifold Key");
    let rock = perm(&mut g, P0, "Arcane Signet");
    hand(&mut g, P0, &["Rite of Flame"]);
    g.perm_mut(rock).tapped = true;
    assert!(options(&mut g, key, P0, Some(false)).is_empty(), "{{1}} for {{1}} is no gain");
}

#[test]
fn manifold_key_makes_the_best_attacker_unblockable_for_three() {
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 3, false);
    let key = perm(&mut g, P0, "Manifold Key");
    let big = token(&mut g, P0, 5);
    token(&mut g, P1, 2); // a blocker
    let o = options(&mut g, key, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, P0, &o[0]));
    let st = g.turn_stamp();
    assert_eq!(g.perm(big).data.get(sim_core::state::DataKey::Unbl), Some(&sim_core::state::Val::Stamp(st)));
    assert!(g.perm(key).tapped);
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 0, "paid {{3}}");
}

// ======================================================== Honor-Worn Shaku
#[test]
fn shaku_taps_a_legendary_permanent_to_untap_for_more_mana() {
    use sim_core::engine::mana::{can_pay, pay, total_mana};
    let mut g = tbl();
    let shaku = perm(&mut g, P0, "Honor-Worn Shaku");
    assert_eq!(total_mana(&g, P0, false), 1, "just {{C}} with no legendary permanent");
    let apple = perm(&mut g, P0, "Apple of Eden, Isu Relic");
    assert_eq!(total_mana(&g, P0, false), 2, "tap the legendary Apple to untap Shaku: {{C}}{{C}}");
    assert!(can_pay(&g, P0, 2, "", false));
    assert!(pay(&mut g, P0, 2, "", false).unwrap());
    assert!(g.perm(shaku).tapped && g.perm(apple).tapped, "both tapped");
    assert_eq!(total_mana(&g, P0, false), 0);
}

#[test]
fn shaku_uses_one_mana_when_only_one_is_needed() {
    use sim_core::engine::mana::{pay, total_mana};
    let mut g = tbl();
    let shaku = perm(&mut g, P0, "Honor-Worn Shaku");
    let apple = perm(&mut g, P0, "Apple of Eden, Isu Relic");
    assert!(pay(&mut g, P0, 1, "", false).unwrap());
    assert!(g.perm(shaku).tapped);
    assert!(!g.perm(apple).tapped, "the Apple is tapped only for the extra mana");
    assert_eq!(total_mana(&g, P0, false), 0);
}

#[test]
fn shaku_does_not_tap_a_creature_that_can_attack() {
    use sim_core::engine::mana::total_mana;
    let mut g = tbl();
    perm(&mut g, P0, "Honor-Worn Shaku");
    perm(&mut g, P0, "Jaxis, the Troublemaker"); // legendary, not summoning sick: a blocker and an attacker (Godo would now fetch the legendary Helm, which Shaku counts)
    assert_eq!(total_mana(&g, P0, false), 1);
}

// ======================================================== Metalworker
#[test]
fn metalworker_makes_cc_for_each_artifact_card_in_hand() {
    use sim_core::engine::mana::{pay, total_mana};
    let mut g = tbl();
    let mw = perm(&mut g, P0, "Metalworker");
    assert_eq!(total_mana(&g, P0, false), 0, "no artifact card revealed, no mana");
    hand(&mut g, P0, &["Sol Ring", "Manifold Key", "Rite of Flame"]);
    assert_eq!(total_mana(&g, P0, false), 4, "two artifact cards: {{C}}{{C}} each");
    assert!(pay(&mut g, P0, 4, "", false).unwrap());
    assert!(g.perm(mw).tapped);
}

#[test]
fn metalworker_is_summoning_sick_like_any_creature() {
    use sim_core::engine::mana::total_mana;
    let mut g = tbl();
    let c = take(&mut g, P0, "Metalworker");
    let m = sim_core::engine::zones::enter(&mut g, P0, c, Default::default()).unwrap();
    hand(&mut g, P0, &["Sol Ring"]);
    assert!(g.perm(m).sick);
    assert_eq!(total_mana(&g, P0, false), 0);
}

// ======================================================== Treasonous Ogre
#[test]
fn treasonous_ogre_pays_three_life_per_red_mana_after_lands() {
    use sim_core::engine::mana::{can_pay, pay, total_mana};
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 2, false);
    let ogre = perm(&mut g, P0, "Treasonous Ogre");
    assert_eq!(g.player(P0).life, 40);
    assert_eq!(total_mana(&g, P0, false), 2 + 10, "two lands and ten payments of 3 life (40 down to 10)");
    // a small cost ({1}{R}) is paid by the two lands: no life lost
    assert!(pay(&mut g, P0, 1, "R", false).unwrap());
    assert_eq!(g.player(P0).life, 40);
    // both Mountains are tapped now ({1}{R}): RRR is all the Ogre
    assert!(can_pay(&g, P0, 0, "RRR", false));
    assert!(pay(&mut g, P0, 0, "RRR", false).unwrap());
    assert_eq!(g.player(P0).life, 40 - 3 * 3, "three mana from the Ogre");
    assert!(!g.perm(ogre).tapped, "paying life does not tap the Ogre");
}

#[test]
fn treasonous_ogre_stops_paying_near_the_life_floor_and_gives_no_colour_but_red() {
    use sim_core::engine::mana::{can_pay, total_mana};
    let mut g = tbl();
    perm(&mut g, P0, "Treasonous Ogre");
    g.player_mut(P0).life = 12;
    assert_eq!(total_mana(&g, P0, false), 0, "12 life is below the floor of 10 plus one payment");
    g.player_mut(P0).life = 19;
    assert_eq!(total_mana(&g, P0, false), 3);
    assert!(can_pay(&g, P0, 0, "RRR", false));
    assert!(!can_pay(&g, P0, 0, "U", false), "only red");
}

#[test]
fn treasonous_ogre_has_no_proactive_floating_mana_ability() {
    use sim_core::dsl;
    let mut g = tbl();
    perm(&mut g, P0, "Treasonous Ogre");
    let o = dsl::ability_options(&mut g, P0, true).unwrap();
    assert!(o.iter().all(|x| !x.label.contains("Treasonous")), "no pay-3-life option for the AI");
}

// ======================================================== _____ Goblin
#[test]
fn blank_goblin_adds_red_when_it_enters() {
    let mut g = tbl();
    assert_eq!(g.player(P0).mana.get(Kind::R), 0);
    perm(&mut g, P0, "_____ Goblin");
    assert_eq!(g.player(P0).mana.get(Kind::R), 4, "four unique vowels on the sticker");
    assert_eq!(g.player(P1).mana.get(Kind::R), 0);
}

// ======================================================== Ignoble Hierarch
#[test]
fn ignoble_hierarch_taps_for_b_r_or_g_and_has_exalted() {
    use sim_core::engine::mana::{can_pay, total_mana};
    let mut g = tbl();
    let c = take(&mut g, P0, "Ignoble Hierarch");
    let m = sim_core::engine::zones::enter(&mut g, P0, c, Default::default()).unwrap();
    assert_eq!(total_mana(&g, P0, false), 0, "summoning sick");
    g.perm_mut(m).sick = false;
    assert_eq!(total_mana(&g, P0, false), 1);
    for col in ["B", "R", "G"] {
        assert!(can_pay(&g, P0, 0, col, false), "{col}");
    }
    assert!(!can_pay(&g, P0, 0, "W", false) && !can_pay(&g, P0, 0, "U", false));
    assert!(g.db.get(c).has_kw("exalted"), "exalted is the engine's keyword (cardcode::keyword_attack)");
}

// ======================================================== Jeweled Amulet
#[test]
fn jeweled_amulet_stores_a_mana_of_the_noted_type_for_later() {
    use sim_core::engine::mana::{can_pay, pay, total_mana};
    use sim_core::state::{DataKey, Val};
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 2, false);
    let am = perm(&mut g, P0, "Jeweled Amulet");
    assert_eq!(total_mana(&g, P0, false), 2, "an uncharged Amulet makes no mana");
    assert!(options(&mut g, am, P0, Some(false)).is_empty(), "second main or end of turn only");
    let o = options(&mut g, am, P0, None);
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, P0, &o[0]));
    assert!(g.perm(am).tapped);
    assert!(matches!(g.perm(am).data.get(DataKey::Charge), Some(Val::Str(s)) if &**s == "W"), "noted white");
    assert_eq!(total_mana(&g, P0, false), 1, "one Plains paid {{1}}");
    assert!(options(&mut g, am, P0, None).is_empty(), "no second counter, and it is tapped");
    // next turn: everything untaps, and the Amulet makes {W}
    g.perm_mut(am).tapped = false;
    for l in g.player(P0).lands.clone() {
        g.perm_mut(l).tapped = false;
    }
    assert_eq!(total_mana(&g, P0, false), 3, "two Plains and the Amulet");
    assert!(pay(&mut g, P0, 2, "", false).unwrap(), "the lands pay first");
    assert!(!g.perm(am).tapped);
    assert!(can_pay(&g, P0, 0, "W", false), "the Amulet's own {{W}} is left");
    assert!(pay(&mut g, P0, 0, "W", false).unwrap());
    assert!(g.perm(am).tapped);
    assert!(!matches!(g.perm(am).data.get(DataKey::Charge), Some(Val::Str(_))), "the counter is gone");
    assert!(!can_pay(&g, P0, 0, "W", false));
}

// ======================================================== Grinding Station
#[test]
fn grinding_station_untaps_when_an_artifact_enters_and_mills_by_sacrificing_one() {
    use sim_core::dsl;
    let mut g = tbl();
    let gs = perm(&mut g, P0, "Grinding Station");
    let ring = perm(&mut g, P0, "Sol Ring");
    // a tapped Station untaps as any artifact enters, an opponent's too
    g.perm_mut(gs).tapped = true;
    perm(&mut g, P1, "Sol Ring");
    assert!(!g.perm(gs).tapped, "an opponent's artifact entering untaps it");
    // a non-artifact entering does nothing
    g.perm_mut(gs).tapped = true;
    perm(&mut g, P0, "Solemn Simulacrum"); // an artifact creature: untaps
    assert!(!g.perm(gs).tapped);
    g.perm_mut(gs).tapped = true;
    perm(&mut g, P0, "Mountain");
    assert!(g.perm(gs).tapped, "a land is not an artifact");
    // {T}, sacrifice an artifact: the target player mills three
    g.perm_mut(gs).tapped = false;
    let libs = |g: &Game| g.players.iter().map(|q| q.library.len()).sum::<usize>();
    let lib = libs(&g);
    let idx = dsl::ability_options(&mut g, P0, true).unwrap();
    let o = idx.iter().find(|x| x.label.contains("Grinding Station")).expect("the compiled mill ability is offered");
    assert!(run(&mut g, P0, o));
    assert!(g.perm(gs).tapped);
    assert_eq!(
        g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf && &*g.perm(m).name == "Sol Ring").count()
            + g.player(P0)
                .perms
                .iter()
                .filter(|&&m| g.perm(m).on_bf && &*g.perm(m).name == "Solemn Simulacrum")
                .count(),
        1,
        "one artifact was sacrificed"
    );
    assert_eq!(libs(&g), lib - 3, "three cards milled from one library");
    let _ = ring;
}

// ======================================================== Monumental Henge
fn lib_top(g: &mut Game, p: PlayerId, names: &[&str]) {
    // the first name ends on top
    for n in names.iter().rev() {
        let c = card(g, n);
        g.player_mut(p).library.push(c);
    }
}

#[test]
fn monumental_henge_digs_five_for_a_historic_card() {
    let mut g = tbl();
    let ls = lands(&mut g, P0, "Monumental Henge", 1, false);
    lands(&mut g, P0, "Plains", 4, false);
    lib_top(&mut g, P0, &["Mountain", "Forest", "Sol Ring", "Island", "Swamp"]);
    let size = g.player(P0).library.len();
    let hand0 = g.player(P0).hand.len();
    let f = g.registry.get(g.perm(ls[0]).card()).unwrap().land_options.unwrap();
    assert!(f(&mut g, ls[0], P0, Some(false)).unwrap().is_empty(), "not before combat");
    let o = f(&mut g, ls[0], P0, None).unwrap();
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, P0, &o[0]));
    assert!(g.perm(ls[0]).tapped, "{{T}} is part of the cost");
    assert_eq!(g.player(P0).hand.len(), hand0 + 1);
    assert!(g.player(P0).hand.contains(&card(&g, "Sol Ring")), "the artifact is historic");
    assert_eq!(g.player(P0).library.len(), size - 1);
    // the other four went to the bottom
    let bottom: Vec<&str> = g.player(P0).library.iter().take(4).map(|&c| &*g.db.get(c).name).collect();
    for n in ["Mountain", "Forest", "Island", "Swamp"] {
        assert!(bottom.contains(&n), "{n} is on the bottom");
    }
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), 0, "{{2}}{{W}}{{W}} paid by the four Plains");
}

#[test]
fn monumental_henge_is_not_offered_without_a_historic_card_on_top() {
    let mut g = tbl();
    let ls = lands(&mut g, P0, "Monumental Henge", 1, false);
    lands(&mut g, P0, "Plains", 4, false);
    lib_top(&mut g, P0, &["Mountain", "Forest", "Island", "Swamp", "Mountain"]);
    let f = g.registry.get(g.perm(ls[0]).card()).unwrap().land_options.unwrap();
    assert!(f(&mut g, ls[0], P0, None).unwrap().is_empty());
    // a legendary creature counts as historic too
    g.player_mut(P0).library.pop();
    lib_top(&mut g, P0, &["Godo, Bandit Warlord"]);
    assert_eq!(f(&mut g, ls[0], P0, None).unwrap().len(), 1);
}

#[test]
fn monumental_henge_enters_tapped_without_a_plains() {
    let mut g = tbl();
    let c = hand(&mut g, P0, &["Monumental Henge"])[0];
    assert!(sim_core::engine::turn::land_enters_tapped(&g, P0, c), "no Plains");
    lands(&mut g, P0, "Plains", 1, false);
    assert!(!sim_core::engine::turn::land_enters_tapped(&g, P0, c), "a Plains");
}

// ======================================================== Apple of Eden, Isu Relic
#[test]
fn apple_of_eden_takes_a_hand_plays_it_and_the_owner_draws() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 6, false);
    let apple = perm(&mut g, P0, "Apple of Eden, Isu Relic");
    // P1's hand: two spells and a land
    g.player_mut(P1).hand.clear();
    let theirs = hand(&mut g, P1, &["Sol Ring", "Solemn Simulacrum", "Mountain"]);
    let lib1 = g.player(P1).library.len();
    let o = options(&mut g, apple, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, P0, &o[0]));
    assert_eq!(g.player(P0).life, 36, "paid 4 life");
    assert!(!on(&g, P0, apple) && g.player(P0).gy.contains(&card(&g, "Apple of Eden, Isu Relic")), "sacrificed");
    // Sol Ring and Solemn were played with white mana (any type): they are on P0's side, their owner's cards
    let names: Vec<_> = g.player(P0).perms.iter().map(|&m| g.perm(m).name.to_string()).collect();
    assert!(names.iter().any(|n| n == "Sol Ring"), "{names:?}");
    // P1 kept the land and drew one card for each spell played
    assert!(g.player(P1).hand.contains(&theirs[2]), "the land stays");
    let drawn = lib1 - g.player(P1).library.len();
    assert_eq!(drawn, 2, "one draw per spell played (Sol Ring and Solemn Simulacrum)");
    assert!(g.player(P0).stolen.is_empty() && g.player(P0).agent_ids.is_empty(), "nothing is left held");
    assert!(g.player(P0).hand.iter().all(|c| !theirs.contains(c)), "unplayed cards went home");
}

#[test]
fn apple_of_eden_needs_life_and_a_hand_worth_taking() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 6, false);
    let apple = perm(&mut g, P0, "Apple of Eden, Isu Relic");
    g.player_mut(P1).hand.clear();
    g.player_mut(P2).hand.clear();
    g.player_mut(PlayerId(3)).hand.clear();
    assert!(options(&mut g, apple, P0, Some(false)).is_empty(), "no hands");
    hand(&mut g, P1, &["Sol Ring", "Solemn Simulacrum"]);
    assert_eq!(options(&mut g, apple, P0, Some(false)).len(), 1);
    g.player_mut(P0).life = 15;
    assert!(options(&mut g, apple, P0, Some(false)).is_empty(), "not at 15 life");
}

#[test]
fn a_land_tapped_for_more_than_the_payment_uses_floats_the_rest() {
    // Gaea's Cradle with four creatures paid one {G} of a {W}{U}{B}{R}{G} activation and the other three were lost
    let mut g = sim_core::testkit::table(&["sisay-legends-toolbox", "seph"]);
    let p = sim_core::ids::PlayerId(0);
    for _ in 0..4 {
        sim_core::testkit::token(&mut g, p, 1);
    }
    sim_core::testkit::lands(&mut g, p, "Gaea's Cradle", 1, false);
    sim_core::testkit::lands(&mut g, p, "Plains", 1, false);
    assert_eq!(sim_core::engine::mana::total_mana(&g, p, false), 5);
    assert!(sim_core::engine::mana::pay(&mut g, p, 0, "WG", false).unwrap());
    assert_eq!(g.player(p).mana.get(Kind::G), 3, "Cradle made four green, one was spent");
    assert!(sim_core::engine::mana::can_pay(&g, p, 0, "GGG", false), "the three float for the next payment");
}
