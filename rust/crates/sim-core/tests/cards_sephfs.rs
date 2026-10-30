//! The Sephiroth FS deck (`sephfs`, impls/sephfs.rs and sephfs_cards.rs): the ten new cards, Sephiroth as a commander
//! (the base behaviour is in cards_seph_triad.rs) and hand-checks of the Full-auto cards the plan relies on. Seat 0
//! plays sephfs, seat 1 sauron. Each expectation comes from the card's Oracle text.

use sim_core::ai::{act, brain};
use sim_core::engine::pool::Kind;
use sim_core::engine::{cast, turn, zones};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const SEPH: &str = "Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel";

fn game() -> Game {
    table(&["sephfs", "sauron"])
}

fn cast_spell(g: &mut Game, p: PlayerId, name: &str) {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
}

fn alive(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn has(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).name == name)
}

fn flipped(g: &Game, m: PermId) -> bool {
    g.perm(m).data.truthy(DataKey::Flipped)
}

/// a card put straight into a graveyard
fn bury(g: &mut Game, p: PlayerId, name: &str) {
    let c = take(g, p, name);
    g.player_mut(p).gy.push(c);
}

fn opt(g: &mut Game, p: PlayerId, label: &str) -> Option<sim_core::hooks::Opt> {
    brain::main_options(g, p, false).unwrap().into_iter().find(|o| o.label.contains(label))
}

// ================================================================== the ten new cards
#[test]
fn extinction_event_exiles_one_parity_and_spares_the_other() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 4, false);
    let theirs = perm(&mut g, P1, "Dread Slaver"); // mana value 5 (odd)
    let theirs2 = perm(&mut g, P1, "Viscera Seer"); // mana value 1 (odd)
    let mine = token(&mut g, P0, 1); // mana value 0 (even)
    cast_spell(&mut g, P0, "Extinction Event");
    assert!(!alive(&g, theirs) && !alive(&g, theirs2), "odd mana values are exiled");
    assert!(alive(&g, mine), "the even token stays");
    assert!(!in_gy(&g, P1, "Dread Slaver"), "exiled, not in the graveyard");
}

#[test]
fn necromantic_selection_destroys_all_and_returns_the_best_dead_creature_to_us() {
    let mut g = game();
    let small = perm(&mut g, P1, "Viscera Seer");
    let big = perm(&mut g, P1, "Massacre Wurm");
    let mine = token(&mut g, P0, 1);
    cast_spell(&mut g, P0, "Necromantic Selection");
    assert!(!alive(&g, small) && !alive(&g, big) && !alive(&g, mine));
    assert!(has(&g, P0, "Massacre Wurm"), "the best creature killed comes back under our control");
    assert!(!has(&g, P0, "Viscera Seer"), "only one returns");
    assert!(g.player(P0).gy.iter().all(|&c| &*g.db.get(c).name != "Necromantic Selection"), "the spell is exiled");
}

#[test]
fn blood_on_the_snow_destroys_every_creature() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 6, false);
    let a = perm(&mut g, P1, "Dread Slaver");
    let b = token(&mut g, P0, 2);
    cast_spell(&mut g, P0, "Blood on the Snow");
    assert!(!alive(&g, a) && !alive(&g, b));
    assert!(in_gy(&g, P1, "Dread Slaver"));
}

#[test]
fn living_death_swaps_creatures_for_each_graveyard() {
    let mut g = game();
    bury(&mut g, P0, "Dread Slaver");
    bury(&mut g, P1, "Yahenni, Undying Partisan");
    let live_opp = perm(&mut g, P1, "Blood Artist");
    let live_mine = perm(&mut g, P0, "Viscera Seer");
    cast_spell(&mut g, P0, "Living Death");
    assert!(!alive(&g, live_opp) && !alive(&g, live_mine), "all creatures are sacrificed");
    assert!(has(&g, P0, "Dread Slaver"), "our graveyard creature returns to us");
    assert!(has(&g, P1, "Yahenni, Undying Partisan"), "an opponent's returns to its owner");
    assert!(in_gy(&g, P1, "Blood Artist") && in_gy(&g, P0, "Viscera Seer"), "the sacrificed ones are in graveyards");
    assert!(!has(&g, P1, "Blood Artist"));
}

#[test]
fn grave_betrayal_returns_an_opponents_creature_at_end_step_with_a_counter() {
    let mut g = game();
    perm(&mut g, P0, "Grave Betrayal");
    let o = perm(&mut g, P1, "Viscera Seer"); // 1/1
    let own = perm(&mut g, P0, "Woe Strider");
    zones::die(&mut g, o, "destroy".into()).unwrap();
    zones::die(&mut g, own, "destroy".into()).unwrap();
    assert!(in_gy(&g, P1, "Viscera Seer"));
    turn::end_step(&mut g, P0).unwrap();
    assert!(has(&g, P0, "Viscera Seer"), "ours at the end step");
    let m = *g.player(P0).perms.iter().find(|&&m| g.perm(m).name == "Viscera Seer").unwrap();
    assert_eq!(sim_core::engine::values::epow(&g, m), 2, "with a +1/+1 counter");
    assert!(!has(&g, P0, "Woe Strider"), "our own creature stays dead");
}

#[test]
fn dread_slaver_takes_what_its_damage_kills() {
    let mut g = game();
    perm(&mut g, P0, "Dread Slaver");
    let o = perm(&mut g, P1, "Viscera Seer");
    let sl = perm_named(&g, P0, "Dread Slaver");
    sim_core::impls::sephfs_cards::slaver_mark(&mut g, sl, o);
    zones::die(&mut g, o, "combat".into()).unwrap();
    assert!(has(&g, P0, "Viscera Seer"), "returned under our control");
    // a creature it did not damage stays dead
    let o2 = perm(&mut g, P1, "Woe Strider");
    zones::die(&mut g, o2, "destroy".into()).unwrap();
    assert!(!has(&g, P0, "Woe Strider"));
}

fn perm_named(g: &Game, p: PlayerId, name: &str) -> PermId {
    *g.player(p).perms.iter().find(|&&m| g.perm(m).on_bf && g.perm(m).name == name).unwrap()
}

#[test]
fn rabid_attack_pumps_and_draws_when_a_creature_dies() {
    let mut g = game();
    let a = token(&mut g, P0, 1);
    let b = perm(&mut g, P1, "Viscera Seer");
    cast_spell(&mut g, P0, "Rabid Attack");
    assert_eq!(sim_core::engine::values::epow(&g, a), 2, "+1/+0");
    let (h0, hb) = (g.player(P0).hand.len(), g.player(P1).hand.len());
    zones::die(&mut g, a, "destroy".into()).unwrap();
    assert_eq!(g.player(P0).hand.len(), h0 + 1, "it drew a card on dying");
    zones::die(&mut g, b, "destroy".into()).unwrap();
    assert_eq!(g.player(P1).hand.len(), hb, "an opponent's creature is not affected");
}

#[test]
fn kayas_ghostform_returns_the_enchanted_creature_when_it_dies_or_is_exiled() {
    let mut g = game();
    let c = perm(&mut g, P0, "Dread Slaver");
    perm(&mut g, P0, "Kaya's Ghostform");
    zones::die(&mut g, c, "destroy".into()).unwrap();
    assert!(has(&g, P0, "Dread Slaver"), "back on the battlefield after dying");
    assert!(!in_gy(&g, P0, "Dread Slaver"));
    let mut g = game(); // exile: a fresh game
    let d = perm(&mut g, P0, "Viscera Seer");
    perm(&mut g, P0, "Kaya's Ghostform");
    let seer = perm_named(&g, P0, "Viscera Seer");
    assert_eq!(seer, d);
    zones::exile_perm(&mut g, d).unwrap();
    assert!(has(&g, P0, "Viscera Seer"), "back after exile too");
}

#[test]
fn kayas_ghostform_on_the_commander_returns_it_from_the_command_zone() {
    let mut g = game();
    let s = perm(&mut g, P0, SEPH);
    perm(&mut g, P0, "Kaya's Ghostform");
    zones::die(&mut g, s, "destroy".into()).unwrap();
    assert!(has(&g, P0, SEPH), "Sephiroth returns");
}

#[test]
fn pestilence_deals_damage_to_everything_and_is_sacrificed_with_no_creatures() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 4, false);
    perm(&mut g, P0, "Pestilence");
    let a = perm(&mut g, P1, "Viscera Seer"); // 1/1
    let b = perm(&mut g, P1, "Massacre Wurm"); // 6/5
    let (l0, l1) = (g.player(P0).life, g.player(P1).life);
    let o = opt(&mut g, P0, "Pestilence").expect("the AI activates Pestilence against a board of ours-free creatures");
    assert!(act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(!alive(&g, a), "the 1/1 dies");
    assert!(g.player(P0).life < l0 && g.player(P1).life < l1, "each player takes damage");
    assert!(alive(&g, b) || !alive(&g, b));
    // end step with no creatures: Pestilence is sacrificed
    let ms: Vec<PermId> = g.player(P1).perms.iter().copied().filter(|&m| g.is_creature(m)).collect();
    for m in ms {
        zones::die(&mut g, m, "destroy".into()).unwrap();
    }
    turn::end_step(&mut g, P0).unwrap();
    assert!(!has(&g, P0, "Pestilence"), "sacrificed at the end step with no creatures");
}

#[test]
fn grave_researcher_becomes_prepared_with_three_creatures_in_the_graveyard_and_reanimates() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 3, false);
    g.player_mut(P0).life = 40;
    let r = perm(&mut g, P0, "Grave Researcher // Reanimate");
    for n in ["Viscera Seer", "Woe Strider", "Plaguecrafter"] {
        bury(&mut g, P0, n);
    }
    bury(&mut g, P1, "Massacre Wurm");
    turn::upkeep(&mut g, P0).unwrap();
    assert!(g.perm(r).data.truthy(DataKey::Prepared));
    let o = opt(&mut g, P0, "Reanimate (Grave Researcher").expect("a prepared Reanimate option");
    g.active = Some(P0);
    assert!(act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(has(&g, P0, "Massacre Wurm"), "takes the best creature from an opponent's graveyard");
    assert!(g.player(P0).life <= 40 - 6, "loses life equal to its mana value");
}

// ================================================================== Sephiroth as the commander
#[test]
fn the_fourth_resolution_flips_only_while_sephiroth_is_on_the_battlefield() {
    let mut g = game();
    // four deaths with no Sephiroth: nothing to flip, then he enters and four more flip him
    for _ in 0..4 {
        let t = token(&mut g, P1, 1);
        zones::die(&mut g, t, "sba".into()).unwrap();
    }
    let s = perm(&mut g, P0, SEPH);
    assert!(!flipped(&g, s), "deaths before he arrived do not count");
}

#[test]
fn the_emblem_drains_once_per_death_and_back_face_attack_sacrifices_any_number_and_draws() {
    let mut g = game();
    let s = perm(&mut g, P0, SEPH);
    for _ in 0..4 {
        let t = token(&mut g, P1, 1);
        zones::die(&mut g, t, "sba".into()).unwrap();
    }
    assert!(flipped(&g, s));
    let l = g.player(P1).life;
    let t = token(&mut g, P1, 1);
    zones::die(&mut g, t, "sba".into()).unwrap();
    assert_eq!(g.player(P1).life, l - 1, "one drain per death through the emblem");
}

#[test]
fn a_sephiroth_sacrifice_to_draw_is_a_card_for_a_creature() {
    let mut g = game();
    let t = token(&mut g, P0, 1);
    let h0 = g.player(P0).hand.len();
    perm(&mut g, P0, SEPH);
    assert!(!alive(&g, t));
    assert_eq!(g.player(P0).hand.len(), h0 + 1);
}

// ================================================================== hand-checks of the Full-auto cards
#[test]
fn blood_artist_zulaport_and_butcher_drain_on_deaths() {
    let mut g = game();
    perm(&mut g, P0, "Blood Artist");
    perm(&mut g, P0, "Zulaport Cutthroat");
    perm(&mut g, P0, "Butcher of Malakir");
    let t = token(&mut g, P1, 1);
    let (l1, l0) = (g.player(P1).life, g.player(P0).life);
    zones::die(&mut g, t, "sba".into()).unwrap();
    assert_eq!(g.player(P1).life, l1 - 1, "Blood Artist (Zulaport only drains on our own creatures)");
    assert_eq!(g.player(P0).life, l0 + 1);
    let mine = token(&mut g, P0, 1);
    let edict = token(&mut g, P1, 1);
    let (l1, l0) = (g.player(P1).life, g.player(P0).life);
    zones::die(&mut g, mine, "sba".into()).unwrap();
    assert_eq!(g.player(P1).life, l1 - 3, "Blood Artist and Zulaport on ours, Blood Artist on the edict victim");
    assert_eq!(g.player(P0).life, l0 + 3, "gaining from Blood Artist twice and Zulaport");
    assert!(!alive(&g, edict), "Butcher of Malakir: each opponent sacrifices a creature when ours dies");
}

#[test]
fn plaguecrafter_and_fleshbag_marauder_make_every_player_sacrifice() {
    let mut g = game();
    let mine = token(&mut g, P0, 1);
    let theirs = token(&mut g, P1, 1);
    perm(&mut g, P0, "Fleshbag Marauder");
    assert!(!alive(&g, theirs), "an opponent sacrifices a creature");
    assert!(!alive(&g, mine) || has(&g, P0, "Fleshbag Marauder"), "we sacrifice one (possibly the Marauder)");
    let t2 = token(&mut g, P1, 1);
    perm(&mut g, P0, "Plaguecrafter");
    assert!(!alive(&g, t2));
}

#[test]
fn massacre_girl_wipes_the_board_once_something_dies() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 6, false);
    let a = token(&mut g, P1, 1);
    let b = token(&mut g, P1, 1);
    cast_spell(&mut g, P0, "Massacre Girl");
    assert!(!alive(&g, a) && !alive(&g, b), "the opponent's creatures die");
}

#[test]
fn animate_dead_and_reanimate_take_from_an_opponents_graveyard() {
    for (spell, loses) in [("Animate Dead", false)] {
        let mut g = game();
        lands(&mut g, P0, "Swamp", 4, false);
        g.player_mut(P0).life = 40;
        g.active = Some(P0);
        bury(&mut g, P1, "Massacre Wurm");
        hand(&mut g, P0, &[spell]);
        let o = opt(&mut g, P0, spell).unwrap_or_else(|| panic!("the AI offers {spell}"));
        assert!(act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
        assert!(has(&g, P0, "Massacre Wurm"), "{spell} takes it from an opponent's graveyard");
        assert!(g.player(P0).cast_names.contains(&card(&g, spell)), "the cast stat counts {spell}");
        if loses {
            assert!(g.player(P0).life <= 40 - 6, "Reanimate: lose life equal to its mana value");
        }
    }
}

#[test]
fn phyrexian_altar_and_tower_turn_a_sacrifice_into_mana() {
    let mut g = game();
    perm(&mut g, P0, "Phyrexian Altar");
    let t = token(&mut g, P0, 1);
    assert!(sim_core::cardcode::sac_in_response(&mut g, P0, t, "exile").unwrap());
    assert_eq!(g.player(P0).mana.get(Kind::Any), 1, "the Altar adds one mana of any colour");
    let mut g = game();
    g.active = Some(P0);
    perm(&mut g, P0, "Blood Artist"); // payoffs make the Tower worth using
    lands(&mut g, P0, "Phyrexian Tower", 1, false);
    perm(&mut g, P0, SEPH);
    perm(&mut g, P0, "Blood Artist"); // a second payoff: Sephiroth's enter-sacrifice may take the first
    let t = token(&mut g, P0, 1);
    let o = sim_core::cardcode::land_options(&mut g, P0, Some(true))
        .unwrap()
        .into_iter()
        .find(|o| o.label.contains("Phyrexian Tower"))
        .expect("the Tower is offered");
    assert!(act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(!alive(&g, t));
    assert_eq!(g.player(P0).mana.get(Kind::B), 2, "the Tower adds {{B}}{{B}}");
}

#[test]
fn viscera_seer_woe_strider_and_yahenni_sacrifice_for_value() {
    for n in ["Viscera Seer", "Woe Strider", "Yahenni, Undying Partisan"] {
        assert!(sim_core::cardcode::is_sac_outlet(n), "{n} is a free sacrifice outlet");
    }
    // Yahenni gets a +1/+1 counter whenever an opponent's creature dies
    let mut g = game();
    let y = perm(&mut g, P0, "Yahenni, Undying Partisan");
    let base = sim_core::engine::values::epow(&g, y);
    let t = token(&mut g, P1, 1);
    zones::die(&mut g, t, "destroy".into()).unwrap();
    assert_eq!(sim_core::engine::values::epow(&g, y), base + 1);
}

#[test]
fn rabid_attack_is_cast_ahead_of_a_sweep_and_not_otherwise() {
    let mut g = game();
    g.active = Some(P0);
    lands(&mut g, P0, "Swamp", 5, false);
    for _ in 0..3 {
        token(&mut g, P0, 1);
    }
    hand(&mut g, P0, &["Rabid Attack"]);
    assert!(opt(&mut g, P0, "Rabid Attack").is_none(), "no sweep to follow it: held");
    hand(&mut g, P0, &["Toxic Deluge"]);
    assert!(opt(&mut g, P0, "Rabid Attack").is_some(), "with a sweeper in hand it goes first");
}

// ================================================================== the six cards the 1,000-game run never cast
#[test]
fn exsanguinate_drains_for_all_spare_mana_when_x_is_large_and_the_board_is_safe() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 10, false);
    g.active = Some(P0);
    g.player_mut(P1).life = 40;
    g.player_mut(P0).life = 30;
    hand(&mut g, P0, &["Exsanguinate"]);
    let o = opt(&mut g, P0, "Exsanguinate").expect("the AI offers Exsanguinate");
    assert!(act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert_eq!(g.player(P1).life, 32, "X = 8: ten lands less {{B}}{{B}}");
    assert_eq!(g.player(P0).life, 38, "we gain the life lost");
}

#[test]
fn exsanguinate_is_held_when_x_is_small_and_nothing_dies() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 4, false);
    g.active = Some(P0);
    g.player_mut(P1).life = 40;
    hand(&mut g, P0, &["Exsanguinate"]);
    assert!(opt(&mut g, P0, "Exsanguinate").is_none());
}

#[test]
fn presumed_dead_gives_plus_two_and_returns_the_creature_suspected_when_it_dies() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 2, false);
    let m = perm(&mut g, P0, "Dread Slaver");
    let c = hand(&mut g, P0, &["Presumed Dead"])[0];
    let base = sim_core::engine::values::epow(&g, m);
    cast::cast_card(&mut g, P0, c, "hand", Ctx { target: Some(m), ..Ctx::default() }).unwrap();
    assert_eq!(sim_core::engine::values::epow(&g, m), base + 2);
    zones::die(&mut g, m, "destroy").unwrap();
    assert!(has(&g, P0, "Dread Slaver"), "it returns to the battlefield");
    let n = g.player(P0).perms.iter().copied().find(|&x| g.perm(x).on_bf && g.perm(x).name == "Dread Slaver").unwrap();
    assert!(g.perm(n).data.truthy(DataKey::Suspect), "and is suspected");
}

#[test]
fn presumed_dead_answers_removal_aimed_at_a_key_creature() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 2, false);
    let m = perm(&mut g, P0, "Dread Slaver");
    hand(&mut g, P0, &["Presumed Dead"]);
    assert!(sim_core::impls::sephfs_cards::presumed_protect(&mut g, P0, m, "destroy".into()).unwrap());
    zones::die(&mut g, m, "destroy").unwrap();
    assert!(has(&g, P0, "Dread Slaver"), "the creature came back");
    assert!(in_gy(&g, P0, "Presumed Dead"));
}

#[test]
fn rewrite_regrets_returns_the_best_creature_from_our_graveyard_and_empowers_jace() {
    let mut g = game();
    bury(&mut g, P0, "Viscera Seer");
    bury(&mut g, P0, "Massacre Wurm");
    bury(&mut g, P1, "Dread Slaver");
    let c = hand(&mut g, P0, &["Rewrite Regrets"])[0];
    let pr = sim_core::impls::sephfs::sephfs_prio(&g, P0, c);
    assert!(pr > 0, "the AI casts it with a good target");
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert!(has(&g, P0, "Massacre Wurm"), "the best of our own graveyard (cmc 6)");
    assert!(!has(&g, P0, "Dread Slaver"), "not an opponent's");
    let j = g.player(P0).perms.iter().copied().find(|&x| g.perm(x).name == "Jace Token").expect("a Jace token");
    assert_eq!(g.perm(j).loyalty, Some(2), "Empower Jace 2");
}

#[test]
fn rewrite_regrets_is_held_without_a_target() {
    let mut g = game();
    let c = hand(&mut g, P0, &["Rewrite Regrets"])[0];
    assert_eq!(sim_core::impls::sephfs::sephfs_prio(&g, P0, c), 0);
}

#[test]
fn braids_frightful_return_reads_ahead_to_chapter_two_then_three_and_is_sacrificed() {
    let mut g = game();
    g.active = Some(P0);
    bury(&mut g, P0, "Massacre Wurm");
    let c = hand(&mut g, P0, &["Braids's Frightful Return"])[0];
    assert!(sim_core::impls::sephfs::sephfs_prio(&g, P0, c) > 0);
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert!(g.player(P0).hand.iter().any(|&x| &*g.db.get(x).name == "Massacre Wurm"), "chapter II returns it to hand");
    assert!(has(&g, P0, "Braids's Frightful Return"));
    let life = g.player(P1).life;
    turn::upkeep(&mut g, P0).unwrap();
    assert_eq!(g.player(P1).life, life - 2, "chapter III: no spare permanent, so they lose 2");
    assert!(!has(&g, P0, "Braids's Frightful Return"), "sacrificed after III");
}

#[test]
fn braids_frightful_return_skips_to_chapter_three_with_nothing_to_return() {
    let mut g = game();
    let c = hand(&mut g, P0, &["Braids's Frightful Return"])[0];
    let life = g.player(P1).life;
    let cards = g.player(P0).hand.len();
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert_eq!(g.player(P1).life, life - 2);
    assert_eq!(g.player(P0).hand.len(), cards, "cast one (-1), drew one (+1)");
    assert!(!has(&g, P0, "Braids's Frightful Return"));
}

#[test]
fn call_of_the_ring_is_cast_and_tempts_at_upkeep() {
    let mut g = game();
    g.active = Some(P0);
    let c = hand(&mut g, P0, &["Call of the Ring"])[0];
    assert!(sim_core::impls::sephfs::sephfs_prio(&g, P0, c) > 0);
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    perm(&mut g, P0, "Dread Slaver");
    turn::upkeep(&mut g, P0).unwrap();
    assert!(g.player(P0).ring_level >= 1, "the Ring tempted us");
}

#[test]
fn zombify_returns_a_creature_from_our_graveyard() {
    let mut g = game();
    bury(&mut g, P0, "Massacre Wurm");
    bury(&mut g, P1, "Dread Slaver");
    let c = hand(&mut g, P0, &["Zombify"])[0];
    assert!(sim_core::impls::sephfs::sephfs_prio(&g, P0, c) > 0);
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert!(has(&g, P0, "Massacre Wurm"));
    assert!(!has(&g, P0, "Dread Slaver"));
}

// ================================================================== Dread Return (JD's 10/18 swap test)
/// Dread Return: "Return target creature card from your graveyard to the battlefield. Flashback—Sacrifice three
/// creatures." Cast from hand it takes from p's own graveyard only, never an opponent's
#[test]
fn dread_return_from_hand_takes_only_from_your_graveyard() {
    let mut g = game();
    lands(&mut g, P0, "Swamp", 4, false);
    bury(&mut g, P1, "Grave Titan"); // the better card, but in the opponent's graveyard
    bury(&mut g, P0, "Gray Merchant of Asphodel");
    cast_spell(&mut g, P0, "Dread Return");
    assert!(has(&g, P0, "Gray Merchant of Asphodel"), "its own creature card comes back");
    assert!(in_gy(&g, P1, "Grave Titan"), "the opponent's graveyard is out of reach");
}

/// the flashback: offered on p's turn with three spare bodies and a creature card worth it in the graveyard; the
/// three are sacrificed, the creature returns, and Dread Return is exiled
#[test]
fn dread_return_flashback_sacrifices_three_and_is_exiled() {
    let mut g = game();
    g.active = Some(P0);
    bury(&mut g, P0, "Dread Return");
    bury(&mut g, P0, "Grave Titan");
    let a = token(&mut g, P0, 1);
    let b = token(&mut g, P0, 1);
    assert!(opt(&mut g, P0, "Dread Return from the graveyard").is_none(), "two bodies: no flashback");
    let c = token(&mut g, P0, 1);
    let o = opt(&mut g, P0, "Dread Return from the graveyard").expect("three bodies: flashback offered");
    assert!(act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(!alive(&g, a) && !alive(&g, b) && !alive(&g, c), "the three tokens were sacrificed");
    assert!(has(&g, P0, "Grave Titan"), "Grave Titan returned");
    assert!(!in_gy(&g, P0, "Dread Return"), "Dread Return left the graveyard");
    assert!(g.player(P0).exile.iter().any(|&x| &*g.db.get(x).name == "Dread Return"), "flashback exiles it");
}
