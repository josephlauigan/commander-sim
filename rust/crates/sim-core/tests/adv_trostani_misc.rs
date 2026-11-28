//! Independent Batch C adversarial tests. Oracle text comes from cardgen find --name NAME --full.
//! The Gargaroth override is an in-memory scratch database: no data file or global test database is changed.

mod play_common;
use play_common::*;
use sim_core::ai;
use sim_core::ai::act::perform;
use sim_core::engine::cast::cast_card;
use sim_core::engine::combat;
use sim_core::engine::repl;
use sim_core::engine::zones::exile_perm;
use sim_core::hooks::Opt;
use sim_core::settings::Profile;
use sim_core::state::{Ctx, DataKey, Step, Val};
use std::sync::{Arc, OnceLock};

fn with_keys(keys: &[&str]) -> Game {
    let mut g = table(keys);
    let mut settings = (*g.settings).clone();
    settings.profile = Profile::Loose;
    g.settings = Arc::new(settings);
    g
}

fn tbl() -> Game {
    with_keys(&["trostani", "seph"])
}

fn fixed() -> Game {
    static DB: OnceLock<Arc<sim_core::cards::CardDb>> = OnceLock::new();
    let db = DB.get_or_init(|| {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../data/cards.json");
        let mut raw = sim_core::export::load_cards(&path).unwrap();
        let row = raw.iter_mut().find(|c| c.name == "Elder Gargaroth").unwrap();
        row.tags.shift_remove("tok");
        row.tags.shift_remove("tokp");
        Arc::new(sim_core::cards::CardDb::from_raw(&raw).unwrap())
    });
    let mut g = tbl();
    g.db = db.clone();
    g
}

fn attack(g: &mut Game, m: PermId) {
    combat::attack_triggers(g, P0, &[m], P1).unwrap();
}

fn token_count(g: &Game) -> usize {
    g.player(P0).perms.iter().filter(|&&m| g.perm(m).token).count()
}

fn shot(g: &mut Game) -> Option<Opt> {
    ai::brain::main_options(g, P0, false)
        .unwrap()
        .into_iter()
        .find(|o| o.label.starts_with("Aetherflux Reservoir: pay 50"))
}

fn reservoir(life: i32) -> Game {
    let mut g = tbl();
    perm(&mut g, P0, "Aetherflux Reservoir");
    g.player_mut(P0).life = life;
    g
}

fn use_opt(g: &mut Game, o: &Opt) -> bool {
    perform(g, P0, o.act.as_ref().unwrap()).unwrap()
}

fn castle(g: &mut Game) -> PermId {
    lands(g, P0, "Plains", 4, false);
    let l = lands(g, P0, "Castle Ardenvale", 1, false)[0];
    g.active = Some(P1);
    g.step = Step::End;
    l
}

fn castle_option(g: &mut Game) -> Opt {
    sim_core::cardcode::land_options(g, P0, None)
        .unwrap()
        .into_iter()
        .find(|o| o.label.starts_with("Castle Ardenvale:"))
        .expect("four other Plains pay for a Human")
}

// Defender still forbids attacks after a power increase and haste (CR 702.3b).
#[test]
fn wall_with_power_and_haste_stays_home() {
    let mut g = tbl();
    let w = perm(&mut g, P0, "Wall of Omens");
    repl::put_plus(&mut g, w, 8);
    g.perm_mut(w).sick = true;
    g.perm_mut(w).eot_kw.push("haste");
    combat::combat(&mut g, P0).unwrap();
    assert_eq!(g.player(P1).life, 40);
    assert!(!g.perm(w).tapped);
}

#[test]
fn wall_human_attacker_filter_still_enforces_defender() {
    let mut g = tbl();
    let w = perm(&mut g, P0, "Wall of Omens");
    repl::put_plus(&mut g, w, 8);
    let other = token(&mut g, P0, 4);
    let c = sim_core::human::combat::attack_candidates(&g, P0);
    assert!(!c.contains(&w));
    assert!(c.contains(&other));
}

#[test]
fn defender_filter_does_not_stop_an_ordinary_attacker_in_another_deck() {
    let mut g = tbl();
    g.player_mut(P0).key = sim_core::sym::intern("karn_reforged");
    token(&mut g, P0, 5);
    combat::combat(&mut g, P0).unwrap();
    assert_eq!(g.player(P1).life, 35);
}

// Gargaroth has only an attack/block trigger, no ETB token ability.
#[test]
#[ignore = "C7: the Gargaroth data override is applied by the coordinator"]
fn gargaroth_unmodified_export_must_not_make_an_etb_beast() {
    let mut g = tbl();
    perm(&mut g, P0, "Elder Gargaroth");
    assert_eq!(token_count(&g), 0, "Oracle text has no enters trigger");
}

#[test]
fn gargaroth_scratch_override_removes_only_the_spurious_etb_token() {
    let mut g = fixed();
    let a = perm(&mut g, P0, "Elder Gargaroth");
    assert_eq!(token_count(&g), 0);
    attack(&mut g, a);
    assert_eq!(token_count(&g), 1, "alone: choose one Beast on attack");
}

// CR 113.7a: an ability on the stack exists independently of its source.
#[test]
fn gargaroth_attack_trigger_survives_its_source_leaving() {
    let mut g = fixed();
    let a = perm(&mut g, P0, "Elder Gargaroth");
    token(&mut g, P0, 1);
    let before = g.player(P0).hand.len();
    g.resolving += 1; // hold the attack's triggers; remove the source before resolving them
    attack(&mut g, a);
    assert!(!g.trig_queue.is_empty());
    exile_perm(&mut g, a).unwrap();
    g.resolving -= 1;
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert_eq!(g.player(P0).hand.len(), before + 1, "source removal does not counter the draw trigger");
}

#[test]
fn gargaroth_attack_with_roaming_throne_gets_two_triggers() {
    let mut g = fixed();
    let throne = perm(&mut g, P0, "Roaming Throne");
    g.perm_mut(throne).data.set(DataKey::Ctype, Val::Str("beast"));
    let a = perm(&mut g, P0, "Elder Gargaroth");
    let before = g.player(P0).hand.len();
    attack(&mut g, a);
    assert_eq!(g.player(P0).hand.len(), before + 2);
}

// Roaming Throne adds one instance of another Beast's triggered ability, including "blocks".
#[test]
#[ignore = "C4 (T8): pending JD"]
fn gargaroth_block_with_roaming_throne_gets_two_triggers() {
    let mut g = fixed();
    let throne = perm(&mut g, P0, "Roaming Throne");
    g.perm_mut(throne).data.set(DataKey::Ctype, Val::Str("beast"));
    let a = perm(&mut g, P0, "Elder Gargaroth");
    let atk = token(&mut g, P1, 2);
    let before = g.player(P0).hand.len();
    sim_core::cardcode::blocks_hooks(&mut g, P1, &[atk], P0, &mut vec![(atk, a)]).unwrap();
    assert_eq!(g.player(P0).hand.len(), before + 2, "one original block trigger plus one Throne instance");
}

#[test]
fn gargaroth_blocking_several_attackers_triggers_once() {
    let mut g = fixed();
    let a = perm(&mut g, P0, "Elder Gargaroth");
    token(&mut g, P0, 1);
    let x = token(&mut g, P1, 1);
    let y = token(&mut g, P1, 1);
    let before = g.player(P0).hand.len();
    // A legal multi-block assignment after an effect allows blocking an additional creature.
    sim_core::cardcode::blocks_hooks(&mut g, P1, &[x, y], P0, &mut vec![(x, a), (y, a)]).unwrap();
    assert_eq!(g.player(P0).hand.len(), before + 1, "blocks, rather than blocks a creature");
}

#[test]
fn panharmonicon_does_not_double_gargaroths_attack_trigger() {
    let mut g = fixed();
    perm(&mut g, P0, "Panharmonicon");
    let a = perm(&mut g, P0, "Elder Gargaroth");
    token(&mut g, P0, 1);
    let before = g.player(P0).hand.len();
    attack(&mut g, a);
    assert_eq!(g.player(P0).hand.len(), before + 1);
}

#[test]
fn gargaroth_rhox_gain_reaches_felidar_from_34() {
    let mut g = fixed();
    perm(&mut g, P0, "Rhox Faithmender");
    perm(&mut g, P0, "Felidar Sovereign");
    let a = perm(&mut g, P0, "Elder Gargaroth");
    g.player_mut(P0).life = 34;
    attack(&mut g, a);
    assert_eq!(g.player(P0).life, 40, "Rhox doubles the chosen 3 life");
}

// A locked life total cannot increase. The AI must not choose life to reach Felidar's threshold when no gain occurs.
#[test]
fn gargaroth_does_not_choose_prevented_life_gain() {
    let mut g = fixed();
    perm(&mut g, P0, "Felidar Sovereign");
    let a = perm(&mut g, P0, "Elder Gargaroth");
    g.player_mut(P0).life = 38;
    g.player_mut(P0).life_locked = true;
    let before = g.player(P0).hand.len();
    attack(&mut g, a);
    assert_eq!(g.player(P0).life, 38);
    assert_eq!(g.player(P0).hand.len(), before + 1);
}

// CR 603.3c, 700.2: choose a modal trigger's mode once when putting it on the stack.
#[test]
fn gargaroth_human_is_asked_for_one_mode_per_trigger() {
    let mut g = fixed();
    let a = perm(&mut g, P0, "Elder Gargaroth");
    let before = g.player(P0).hand.len();
    let h = Arc::new(ScriptHuman::from_fn(|r| {
        if r.kind == "mode" { Answer::Pick(2) } else { Answer::Action(json!({"do":"pass"})) }
    }));
    g.humans.add(P0, h.clone());
    attack(&mut g, a);
    assert_eq!(g.player(P0).hand.len(), before + 1);
    assert_eq!(h.asked_kind("mode").len(), 1, "one attack trigger has one mode choice");
}

#[test]
fn reservoir_exactly_50_is_not_an_ai_shot() {
    let mut g = reservoir(50);
    assert!(shot(&mut g).is_none());
}

#[test]
fn reservoir_at_51_pays_exactly_50_despite_rhox() {
    let mut g = reservoir(51);
    perm(&mut g, P0, "Rhox Faithmender");
    let o = shot(&mut g).unwrap();
    assert!(use_opt(&mut g, &o));
    assert_eq!(g.player(P0).life, 1, "life payment is not life gain");
    assert!(g.player(P1).life <= 0, "50 damage is lethal; the outer action loop checks state-based actions");
}

#[test]
fn reservoir_cannot_pay_when_its_controller_has_locked_life() {
    let mut g = reservoir(100);
    g.player_mut(P0).life_locked = true;
    assert!(shot(&mut g).is_none());
}

#[test]
fn reservoir_cannot_target_ring_protection() {
    let mut g = reservoir(60);
    g.player_mut(P1).ring_prot = true;
    assert!(shot(&mut g).is_none());
}

// Metropolis Reformer: "You have hexproof." Opponents cannot target that player (CR 702.11c).
#[test]
fn reservoir_must_not_offer_a_shot_at_a_hexproof_player() {
    let mut g = reservoir(60);
    perm(&mut g, P1, "Metropolis Reformer");
    assert!(sim_core::engine::values::player_hexproof(&g, P1), "fixture has player hexproof");
    assert!(shot(&mut g).is_none(), "hexproof player is not a legal opponent target");
}

// Chasm prevents all damage to its controller; a Reservoir shot cannot kill that player.
// The approved AI rule only spends 50 life for a kill/win, so this is an AI failure, not an illegal activation.
#[test]
fn reservoir_must_not_pay_for_a_shot_prevented_by_chasm() {
    let mut g = reservoir(60);
    lands(&mut g, P1, "Glacial Chasm", 1, false);
    assert!(sim_core::engine::life::prevents_damage(&g, P1, Some(P0)));
    assert!(shot(&mut g).is_none(), "50 damage is prevented, so it does not kill");
}

// Platinum Angel: "You can't lose the game". Damage to its controller does not eliminate that opponent.
// The approved AI rule spends life only for a kill/win.
#[test]
fn reservoir_must_not_pay_for_a_nonkill_against_platinum_angel() {
    let mut g = reservoir(60);
    perm(&mut g, P1, "Platinum Angel");
    assert!(shot(&mut g).is_none(), "the opponent cannot lose to this shot");
}

#[test]
fn reservoir_option_rechecks_protection_before_paying() {
    let mut g = reservoir(60);
    let o = shot(&mut g).unwrap();
    g.player_mut(P1).ring_prot = true;
    assert!(!use_opt(&mut g, &o));
    assert_eq!(g.player(P0).life, 60);
}

#[test]
fn reservoir_two_copies_do_not_allow_a_second_payment_from_51() {
    // Keep a third player alive so the first shot doesn't finish the game.
    let mut g = with_keys(&["trostani", "seph", "sheoldred"]);
    g.player_mut(P0).life = 51;
    perm(&mut g, P0, "Aetherflux Reservoir");
    perm(&mut g, P0, "Aetherflux Reservoir");
    let o = shot(&mut g).unwrap();
    assert!(use_opt(&mut g, &o));
    assert_eq!(g.player(P0).life, 1);
    assert!(shot(&mut g).is_none());
}

#[test]
fn reservoir_karn_reforged_can_take_a_winning_shot() {
    let mut g = reservoir(60);
    g.player_mut(P0).key = sim_core::sym::intern("karn_reforged");
    let o = shot(&mut g).expect("the new rule is global, including Karn");
    assert!(use_opt(&mut g, &o));
    assert!(g.player(P1).life <= 0, "50 damage is lethal; the outer action loop checks state-based actions");
}

// Felidar wins at upkeep with >=40 life. The design explicitly says hold the Reservoir shot when that win is certain.
#[test]
fn reservoir_holds_life_for_felidar_with_no_opponent_turn_left() {
    let mut g = with_keys(&["trostani", "seph", "sheoldred"]);
    perm(&mut g, P0, "Aetherflux Reservoir");
    perm(&mut g, P0, "Felidar Sovereign");
    g.player_mut(P0).life = 51;
    g.active = Some(P2);
    g.step = Step::End;
    ai::end_of_turn_window(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).life, 51, "a nonwinning shot must not sacrifice the certain upkeep win");
}

// Test of Endurance has the same upkeep win condition at 50 life; preserve that threshold.
#[test]
fn reservoir_holds_life_for_test_of_endurance() {
    let mut g = with_keys(&["trostani", "seph", "sheoldred"]);
    perm(&mut g, P0, "Aetherflux Reservoir");
    perm(&mut g, P0, "Test of Endurance");
    g.player_mut(P0).life = 51;
    g.active = Some(P2);
    g.step = Step::End;
    ai::end_of_turn_window(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).life, 51, "retain 50 life for the next upkeep instead of killing only one opponent");
}

#[test]
fn sunblast_payment_uses_lands_and_spares_ready_dorks() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 6, false);
    let bird = perm(&mut g, P0, "Birds of Paradise");
    let enemy = token(&mut g, P1, 6);
    g.perm_mut(enemy).tapped = true;
    let c = hand(&mut g, P0, &["Sunblast Angel"])[0];
    assert!(ai::pool::generic_prio(&g, P0, c) > 0);
    assert!(perform(&mut g, P0, &sim_core::hooks::Action::Cast { card: c, zone: None }).unwrap());
    assert!(g.perm(bird).on_bf && !g.perm(bird).tapped);
    assert!(!g.perm(enemy).on_bf);
    assert!(g.player(P0).lands.iter().all(|&m| g.perm(m).tapped), "all six lands paid for the Angel");
}

#[test]
fn sunblast_cannot_fund_itself_with_summoning_sick_dorks() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 3, false);
    for _ in 0..3 {
        let b = perm(&mut g, P0, "Birds of Paradise");
        g.perm_mut(b).sick = true;
    }
    let c = hand(&mut g, P0, &["Sunblast Angel"])[0];
    assert_eq!(ai::pool::generic_prio(&g, P0, c), 0);
    assert!(!sim_core::engine::mana::can_pay_spell(&g, P0, c, 4, "WW", false));
}

#[test]
fn sunblast_destruction_respects_regeneration() {
    let mut g = tbl();
    let m = token(&mut g, P1, 3);
    g.perm_mut(m).tapped = true;
    g.player_mut(P1).regen_turn = Some(g.turn_stamp());
    perm(&mut g, P0, "Sunblast Angel");
    assert!(g.perm(m).on_bf, "destroy allows regeneration");
}

#[test]
fn sunblast_indestructible_tapped_creature_survives() {
    let mut g = tbl();
    let m = token(&mut g, P1, 3);
    g.perm_mut(m).tapped = true;
    g.perm_mut(m).eot_kw.push("indestructible");
    perm(&mut g, P0, "Sunblast Angel");
    assert!(g.perm(m).on_bf);
}

#[test]
fn castle_doubling_season_makes_two_humans() {
    let mut g = tbl();
    castle(&mut g);
    perm(&mut g, P0, "Doubling Season");
    let o = castle_option(&mut g);
    assert!(use_opt(&mut g, &o));
    assert_eq!(token_count(&g), 2);
    for &m in &g.player(P0).perms {
        if g.perm(m).token {
            assert!(g.perm(m).ttypes.contains(&"human"));
            assert_eq!(sim_core::engine::values::epow(&g, m), 1);
        }
    }
}

#[test]
fn castle_rechecks_a_destroyed_land_before_payment() {
    let mut g = tbl();
    let l = castle(&mut g);
    let o = castle_option(&mut g);
    exile_perm(&mut g, l).unwrap();
    assert!(!use_opt(&mut g, &o));
    assert_eq!(token_count(&g), 0);
    assert!(g.player(P0).lands.iter().all(|&m| !g.perm(m).tapped));
}

#[test]
fn castle_rechecks_mana_and_does_not_create_a_free_token() {
    let mut g = tbl();
    let l = castle(&mut g);
    let o = castle_option(&mut g);
    let plains = g.player(P0).lands[0];
    g.perm_mut(plains).tapped = true;
    assert!(!use_opt(&mut g, &o));
    assert_eq!(token_count(&g), 0);
    assert!(!g.perm(l).tapped);
}

#[test]
fn castle_end_step_window_ignores_tapped_mana_sources() {
    let mut g = tbl();
    let l = castle(&mut g);
    let plains = g.player(P0).lands[0];
    g.perm_mut(plains).tapped = true;
    ai::end_of_turn_window(&mut g, P0).unwrap();
    assert_eq!(token_count(&g), 0);
    assert!(!g.perm(l).tapped);
}

fn protection(g: &mut Game, m: PermId) -> bool {
    let sw = card(g, "Swords to Plowshares");
    ai::protect_response(g, P0, m, "exile", Some(P1), Some(sw)).unwrap()
}

#[test]
fn cloudshift_scope_requires_the_exact_seat_key() {
    for key in ["trostani-x", "Trostani", "not-trostani", "karn_reforged"] {
        let mut g = fixed();
        g.player_mut(P0).key = sim_core::sym::intern(key);
        let a = perm(&mut g, P0, "Elder Gargaroth");
        lands(&mut g, P0, "Plains", 1, false);
        let c = hand(&mut g, P0, &["Cloudshift"])[0];
        assert!(!protection(&mut g, a), "unexpected protection for key {key}");
        assert!(g.player(P0).hand.contains(&c));
    }
}

#[test]
fn cloudshift_saves_a_commander_and_preserves_commander_identity() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Trostani, Selesnya's Voice");
    g.perm_mut(t).is_cmd = true;
    lands(&mut g, P0, "Plains", 1, false);
    hand(&mut g, P0, &["Cloudshift"]);
    assert!(protection(&mut g, t));
    assert!(!g.perm(t).on_bf);
    let returned = perms_named(&g, P0, "Trostani, Selesnya's Voice");
    assert_eq!(returned.len(), 1);
    assert!(g.perm(returned[0]).is_cmd);
}

#[test]
fn cloudshift_does_not_spend_protection_on_a_token() {
    let mut g = tbl();
    let t = token(&mut g, P0, 20);
    lands(&mut g, P0, "Plains", 1, false);
    let c = hand(&mut g, P0, &["Cloudshift"])[0];
    assert!(!protection(&mut g, t));
    assert!(g.player(P0).hand.contains(&c));
}

// Cloudshift exiles its target. CR 111.8: a token that leaves the battlefield cannot return.
// Human players may legally choose tokens even though doing so is normally bad play.
#[test]
#[ignore = "C5: pending JD"]
fn cloudshift_human_targeting_a_token_exiles_it_permanently() {
    let mut g = tbl();
    let t = token(&mut g, P0, 3);
    lands(&mut g, P0, "Plains", 1, false);
    pool(&mut g, P0, &[('W', 1)]); // human casting spends manually floated mana
    let c = hand(&mut g, P0, &["Cloudshift"])[0];
    let h = Arc::new(ScriptHuman::from_fn(|r| {
        if r.kind == "target" { Answer::Pick(0) } else { Answer::Action(json!({"do":"pass"})) }
    }));
    g.humans.add(P0, h);
    assert_eq!(sim_core::human::legal::check_cast(&mut g, P0, c, "hand"), None);
    assert_eq!(sim_core::human::human::cast(&mut g, P0, c, "hand").unwrap(), None);
    assert!(g.player(P0).gy.contains(&c), "the spell resolved");
    assert!(!g.perm(t).on_bf, "the token is exiled and cannot return");
    assert_eq!(token_count(&g), 0);
}

// Cloudshift requires "target creature you control". Without one, casting must be rejected (CR 601.2c).
#[test]
#[ignore = "C2: pending JD"]
fn cloudshift_human_without_a_creature_cannot_spend_the_card() {
    let mut g = tbl();
    pool(&mut g, P0, &[('W', 1)]);
    let c = hand(&mut g, P0, &["Cloudshift"])[0];
    seat(&mut g, P0, vec![]);
    let problem = sim_core::human::legal::check_cast(&mut g, P0, c, "hand").expect("no creature is a legal target");
    assert!(problem.contains("no legal target"), "{problem}");
    assert!(g.player(P0).hand.contains(&c));
    assert!(!g.player(P0).gy.contains(&c));
}

#[test]
fn beast_whisperer_does_not_draw_for_a_creature_put_onto_the_battlefield() {
    let mut g = tbl();
    perm(&mut g, P0, "Beast Whisperer");
    let before = g.player(P0).hand.len();
    perm(&mut g, P0, "Wall of Omens");
    assert_eq!(g.player(P0).hand.len(), before + 1, "only the Wall's enters draw, no cast trigger");
}

#[test]
fn eladamris_call_without_creatures_does_not_take_a_noncreature() {
    let mut g = tbl();
    g.player_mut(P0).library = vec![card(&g, "Plains")];
    lands(&mut g, P0, "Forest", 1, false);
    lands(&mut g, P0, "Plains", 1, false);
    // lands() removes matching basics from the library; restore the one-card library for the actual search.
    g.player_mut(P0).library = vec![card(&g, "Plains")];
    let c = hand(&mut g, P0, &["Eladamri's Call"])[0];
    assert!(cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap());
    assert!(g.player(P0).hand.is_empty());
    assert_eq!(g.player(P0).library.len(), 1);
}

#[test]
fn kodamas_reach_with_one_basic_puts_only_one_land_into_play() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 3, false);
    let c = hand(&mut g, P0, &["Kodama's Reach"])[0];
    g.player_mut(P0).library = vec![card(&g, "Plains")];
    assert!(cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap());
    assert_eq!(g.player(P0).lands.len(), 4);
    assert!(g.perm(*g.player(P0).lands.last().unwrap()).tapped);
    assert!(g.player(P0).hand.is_empty());
}

// Sphere counts enchantments you control; phased-out permanents are treated as nonexistent (CR 702.26b).
#[test]
#[ignore = "C8 (T9): pending JD"]
fn sphere_counts_itself_but_not_a_phased_enchantment() {
    let mut g = tbl();
    perm(&mut g, P0, "Sphere of Safety");
    let d = perm(&mut g, P0, "Doubling Season");
    g.perm_mut(d).phased = true;
    assert_eq!(combat::attack_restrictions(&g, P1, P0).0, 1);
}

#[test]
fn thalia_tax_ends_when_she_leaves() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Thalia, the Survivor");
    let c = card(&g, "Swords to Plowshares");
    assert_eq!(sim_core::engine::mana::cost_of(&g, P1, c).0, 1);
    exile_perm(&mut g, t).unwrap();
    assert_eq!(sim_core::engine::mana::cost_of(&g, P1, c).0, 0);
}

#[test]
fn archon_and_sphere_attack_taxes_add() {
    let mut g = tbl();
    perm(&mut g, P0, "Archon of Absolution");
    perm(&mut g, P0, "Sphere of Safety");
    perm(&mut g, P0, "Doubling Season");
    assert_eq!(combat::attack_restrictions(&g, P1, P0).0, 3, "Archon 1 plus Sphere 2");
}
