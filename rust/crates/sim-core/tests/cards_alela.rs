//! Avery's Alela deck (impls/alela.rs, Python's cards/impl/alela.py) on hand-built positions: Python's
//! tests/test_alela.py (all of it), test_my_cards.py's two Last Gasp tests (seated with Alela's deck), and the deck
//! plan's choices (Alela's priorities with the Faerie bonus, the hand plays' timing).

use sim_core::ai::{act, decks};
use sim_core::engine::hooks::fire_trigger;
use sim_core::engine::{cast, mana, removal, stack, turn, values, zones};
use sim_core::hooks::{Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::alela;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

/// Alela from the command zone onto the battlefield (Python's `alela(g, p)`)
fn alela_out(g: &mut Game, p: PlayerId) -> PermId {
    let c = g.player(p).cmd;
    let m = zones::enter(g, p, c, zones::Enter::default()).unwrap();
    g.perm_mut(m).is_cmd = true;
    g.player_mut(p).cmd_in_zone = false;
    m
}

fn names(g: &Game, p: PlayerId) -> Vec<&'static str> {
    g.player(p).perms.iter().map(|&m| g.perm(m).name).collect()
}

/// a hand card's `hand_options`
fn hand_opts(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).and_then(|i| i.hand_options).expect("the card has hand options");
    f(g, c, p, post).unwrap()
}

fn perm_opts(g: &mut Game, m: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.perm(m).cd.unwrap()).and_then(|i| i.options).expect("the card has options");
    f(g, m, p, post).unwrap()
}

fn perform(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

/// a 1/1 flying token, not summoning sick (Python's `token(g, p, 1, fly=True)`)
fn flier(g: &mut Game, p: PlayerId) -> PermId {
    let spec = zones::Tokens { sick: false, fly: true, tgh: Some(1), ..zones::Tokens::new(1, 1) };
    zones::make_tokens(g, p, spec).unwrap()[0]
}

fn on_cast(g: &mut Game, p: PlayerId, name: &str) {
    let c = card(g, name);
    cast::on_cast(g, p, c).unwrap();
}

fn upkeep(g: &mut Game, p: PlayerId) {
    fire_trigger(g, Event::Upkeep, Call::Player { p }).unwrap();
}

/// a card put on top of p's library (Python's `p.library.append(card(n))`)
fn on_top(g: &mut Game, p: PlayerId, name: &str) {
    let c = card(g, name);
    g.player_mut(p).library.push(c);
}

// ======================================================== Faeries
#[test]
fn artifact_spell_makes_a_faerie_with_alela_out() {
    let mut g = table(&["alela", "veyran"]);
    alela_out(&mut g, P0);
    lands(&mut g, P0, "Plains", 2, false);
    let c = hand(&mut g, P0, &["Mind Stone"])[0];
    mana::pay(&mut g, P0, 2, "", false).unwrap();
    cast::cast_card(&mut g, P0, c, "hand", Default::default()).unwrap();
    let toks: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token).collect();
    assert_eq!(toks.len(), 1);
    assert!(g.perm(toks[0]).fly);
    assert_eq!(values::epow(&g, toks[0]), 2, "Alela's +1/+0 to other fliers");
}

// ======================================================== the Jace token
#[test]
fn empower_creates_then_grows() {
    let mut g = table(&["alela", "veyran"]);
    alela::empower(&mut g, P0, 2);
    let j = alela::jace_token(&g, P0).unwrap();
    assert_eq!(g.perm(j).loyalty, Some(2));
    alela::empower(&mut g, P0, 1);
    assert_eq!(alela::jace_token(&g, P0), Some(j));
    assert_eq!(g.perm(j).loyalty, Some(3));
}

#[test]
fn keeper_empowers_two() {
    let mut g = table(&["alela", "veyran"]);
    perm(&mut g, P0, "Keeper of the Quiet Hour");
    let j = alela::jace_token(&g, P0).unwrap();
    assert_eq!(g.perm(j).loyalty, Some(2));
}

#[test]
fn annex_enters_untapped_with_a_planeswalker() {
    let mut g = table(&["alela", "veyran"]);
    lands(&mut g, P0, "Plains", 3, false);
    let annex = card(&g, "Fatehold Annex");
    assert!(turn::land_enters_tapped(&g, P0, annex));
    alela::empower(&mut g, P0, 1);
    assert!(!turn::land_enters_tapped(&g, P0, annex));
}

#[test]
fn the_token_leaves_no_card_behind() {
    let mut g = table(&["alela", "veyran"]);
    alela::empower(&mut g, P0, 1);
    let j = alela::jace_token(&g, P0).unwrap();
    let gy = g.player(P0).gy.len();
    zones::bounce(&mut g, j).unwrap();
    assert!(alela::jace_token(&g, P0).is_none());
    assert_eq!(g.player(P0).gy.len(), gy);
    let jc = card(&g, alela::JACE);
    assert!(!g.player(P0).hand.contains(&jc));
}

#[test]
fn minus_three_draws() {
    let mut g = table(&["alela", "veyran"]);
    alela::empower(&mut g, P0, 3);
    let j = alela::jace_token(&g, P0).unwrap();
    let o = perm_opts(&mut g, j, P0, Some(false));
    let best = o.iter().fold(None::<&Opt>, |b, x| match b {
        Some(y) if y.utility >= x.utility => Some(y),
        _ => Some(x),
    });
    let best = best.unwrap().clone();
    assert!(best.label.contains("-3"), "{}", best.label);
    let n = g.player(P0).hand.len();
    perform(&mut g, P0, &best);
    assert_eq!(g.player(P0).hand.len(), n + 1);
    assert!(alela::jace_token(&g, P0).is_none(), "loyalty 0: gone");
}

#[test]
fn jace_surveils_only_while_far_from_the_draw() {
    let mut g = table(&["alela", "veyran"]);
    alela::empower(&mut g, P0, 2);
    let j = alela::jace_token(&g, P0).unwrap();
    let o = perm_opts(&mut g, j, P0, Some(true));
    assert_eq!(o.len(), 1, "-1 only (no Plan for All Outcomes); -3 can't be paid");
    perm(&mut g, P0, "Plan for All Outcomes");
    let o = perm_opts(&mut g, j, P0, Some(true));
    assert!(o.iter().all(|x| !x.label.contains("-1")), "the Plan will grow it to the -3");
}

#[test]
fn plan_empowers_on_first_noncreature_spell_only() {
    let mut g = table(&["alela", "veyran"]);
    perm(&mut g, P0, "Plan for All Outcomes");
    on_cast(&mut g, P0, "Night's Whisper");
    on_cast(&mut g, P0, "Sol Ring");
    let j = alela::jace_token(&g, P0).unwrap();
    assert_eq!(g.perm(j).loyalty, Some(1));
}

// ======================================================== removal and control
#[test]
fn multiply_by_zero_kills_unless_counters() {
    let mut g = table(&["alela", "veyran"]);
    let a = perm(&mut g, P1, "Grave Titan");
    let b = perm(&mut g, P1, "Serra Angel");
    g.perm_mut(b).plus = 1;
    let t = removal::legal_targets(&g, P0, "zero", "c", false, None);
    assert!(t.contains(&a));
    assert!(!t.contains(&b));
    removal::apply_removal(&mut g, Some(P0), a, "zero", None).unwrap();
    assert!(!g.perm(a).on_bf);
}

#[test]
fn prophesied_end_draws_unless_attacking() {
    let mut g = table(&["alela", "veyran"]);
    let spell = card(&g, "Prophesied End");
    let a = perm(&mut g, P1, "Grave Titan");
    let n = g.player(P1).hand.len();
    removal::apply_removal(&mut g, Some(P0), a, "destroy", Some(spell)).unwrap();
    assert_eq!(g.player(P1).hand.len(), n + 1);
    let b = perm(&mut g, P1, "Serra Angel");
    g.in_combat = vec![b];
    removal::apply_removal(&mut g, Some(P0), b, "destroy", Some(spell)).unwrap();
    g.in_combat.clear();
    assert_eq!(g.player(P1).hand.len(), n + 1);
}

#[test]
fn plan_puts_the_permanent_on_top() {
    let mut g = table(&["alela", "veyran"]);
    let t = perm(&mut g, P1, "Grave Titan");
    perm(&mut g, P0, "Plan for All Outcomes");
    assert!(!g.perm(t).on_bf);
    let top = *g.player(P1).library.last().unwrap();
    assert_eq!(&*g.db.get(top).name, "Grave Titan");
}

#[test]
fn karmic_justice_answers_an_opponents_destroy() {
    let mut g = table(&["alela", "veyran"]);
    perm(&mut g, P0, "Karmic Justice");
    let stone = perm(&mut g, P0, "Mind Stone");
    let t = perm(&mut g, P1, "Grave Titan");
    let abrade = card(&g, "Abrade");
    removal::apply_removal(&mut g, Some(P1), stone, "destroy", Some(abrade)).unwrap();
    assert!(!g.perm(t).on_bf);
}

#[test]
fn karmic_justice_ignores_creatures_and_your_own() {
    let mut g = table(&["alela", "veyran"]);
    perm(&mut g, P0, "Karmic Justice");
    let c = perm(&mut g, P0, "Serra Angel");
    let stone = perm(&mut g, P0, "Mind Stone");
    let t = perm(&mut g, P1, "Grave Titan");
    removal::apply_removal(&mut g, Some(P1), c, "destroy", None).unwrap();
    removal::apply_removal(&mut g, Some(P0), stone, "destroy", None).unwrap();
    assert!(g.perm(t).on_bf);
}

#[test]
fn ray_of_command_steals_and_returns_tapped() {
    let mut g = table(&["alela", "veyran"]);
    lands(&mut g, P0, "Island", 4, false);
    let t = perm(&mut g, P1, "Grave Titan");
    g.perm_mut(t).tapped = true;
    let c = hand(&mut g, P0, &["Ray of Command"])[0];
    let o = hand_opts(&mut g, c, P0, Some(false));
    assert!(!o.is_empty() && perform(&mut g, P0, &o[0]));
    assert_eq!(g.perm(t).owner, P0);
    assert!(!g.perm(t).tapped);
    turn::end_step(&mut g, P0).unwrap();
    assert_eq!(g.perm(t).owner, P1);
    assert!(g.perm(t).tapped);
}

#[test]
fn malice_destroys_a_nonblack_creature() {
    let mut g = table(&["alela", "veyran"]);
    lands(&mut g, P0, "Swamp", 4, false);
    let t = perm(&mut g, P1, "Consecrated Sphinx");
    perm(&mut g, P1, "Grave Titan"); // black
    let c = hand(&mut g, P0, &["Spite // Malice"])[0];
    let o = hand_opts(&mut g, c, P0, Some(false));
    assert_eq!(o.len(), 1);
    perform(&mut g, P0, &o[0]);
    assert!(!g.perm(t).on_bf);
}

#[test]
fn muddle_counters_only_instants_and_sorceries() {
    let g = table(&["alela", "veyran"]);
    let m = card(&g, "Muddle the Mixture");
    assert!(stack::counter_ok(&g, m, card(&g, "Counterspell")));
    assert!(!stack::counter_ok(&g, m, card(&g, "Rhystic Study")));
}

#[test]
fn last_gasp_targets() {
    let mut g = table(&["alela", "sauron"]);
    let bow = perm(&mut g, P1, "Orcish Bowmasters"); // black
    let mimic = perm(&mut g, P1, "Metallic Mimic"); // artifact
    let lg = card(&g, "Last Gasp");
    let t: Vec<PermId> = removal::legal_targets(&g, P0, "shrink3", "c", false, Some(lg))
        .into_iter()
        .filter(|&m| !g.perm(m).token)
        .collect();
    assert_eq!(t, vec![bow, mimic]);
}

#[test]
fn last_gasp_ignores_indestructible() {
    let mut g = table(&["alela", "veyran"]);
    let snipe = perm(&mut g, P1, "Guttersnipe");
    g.player_mut(P1).regen_turn = Some(g.turn_stamp());
    let lg = card(&g, "Last Gasp");
    removal::apply_removal(&mut g, Some(P0), snipe, "shrink3", Some(lg)).unwrap();
    let gs = card(&g, "Guttersnipe");
    assert!(g.player(P1).gy.contains(&gs));
}

// ======================================================== Opposition
#[test]
fn opposition_taps_their_mana_in_their_upkeep() {
    let mut g = table(&["alela", "veyran"]);
    perm(&mut g, P0, "Opposition");
    flier(&mut g, P0);
    flier(&mut g, P0);
    lands(&mut g, P1, "Island", 3, false);
    g.active = Some(P1);
    upkeep(&mut g, P1);
    assert_eq!(g.player(P1).lands.iter().filter(|&&l| g.land(l).tapped).count(), 2);
}

#[test]
fn opposition_taps_attackers_at_their_combat() {
    let mut g = table(&["alela", "veyran"]);
    perm(&mut g, P0, "Opposition");
    flier(&mut g, P0);
    let t = perm(&mut g, P1, "Grave Titan");
    g.active = Some(P1);
    alela::opposition_precombat(&mut g, P0).unwrap();
    assert!(g.perm(t).tapped);
}

#[test]
fn opposition_taps_blocking_fliers_on_your_turn() {
    // your beginning of combat: a summoning-sick creature taps the opponent's flier
    let mut g = table(&["alela", "veyran"]);
    perm(&mut g, P0, "Opposition");
    let sick = flier(&mut g, P0);
    g.perm_mut(sick).sick = true;
    let angel = perm(&mut g, P1, "Serra Angel");
    fire_trigger(&mut g, Event::Crew, Call::Player { p: P0 }).unwrap();
    assert!(g.perm(angel).tapped && g.perm(sick).tapped);
}

// ======================================================== energy
#[test]
fn static_prison_pays_then_goes() {
    let mut g = table(&["alela", "veyran"]);
    let t = perm(&mut g, P1, "Grave Titan");
    let sp = perm(&mut g, P0, "Static Prison");
    assert!(!g.perm(t).on_bf);
    assert_eq!(g.player(P0).energy, 2);
    upkeep(&mut g, P0);
    upkeep(&mut g, P0);
    assert!(g.perm(sp).on_bf);
    upkeep(&mut g, P0);
    assert!(!g.perm(sp).on_bf);
    assert!(names(&g, P1).contains(&"Grave Titan"));
}

#[test]
fn aether_hub_colour_costs_energy() {
    let mut g = table(&["alela", "veyran"]);
    let hub = card(&g, "Aether Hub");
    let l = g.add_land(P0, hub, false);
    let f = g.registry.get(hub).unwrap().land_etb.unwrap();
    f(&mut g, P0, l).unwrap();
    assert_eq!(mana::land_colors_now(&g, P0, l), g.player(P0).ident);
    mana::pay(&mut g, P0, 0, "W", false).unwrap();
    assert_eq!(g.player(P0).energy, 0);
    g.land_mut(l).tapped = false;
    assert!(mana::land_colors_now(&g, P0, l).is_empty());
}

// ======================================================== creatures
#[test]
fn plumecreed_counts_faerie_tokens() {
    let mut g = table(&["alela", "veyran"]);
    perm(&mut g, P0, "Plumecreed Mentor");
    let page = perm(&mut g, P0, "Page, Loose Leaf");
    flier(&mut g, P0);
    assert_eq!(g.perm(page).plus, 1);
}

#[test]
fn jackdaw_returns_a_cheaper_creature() {
    let mut g = table(&["alela", "veyran"]);
    let i = card(&g, "Initiates of the Ebon Hand");
    g.player_mut(P0).gy.push(i);
    let j = perm(&mut g, P0, "Jackdaw Savior");
    zones::die(&mut g, j, "destroy").unwrap();
    assert!(names(&g, P0).contains(&"Initiates of the Ebon Hand"));
}

#[test]
fn malcator_golem_at_end_step_after_three_artifacts() {
    let mut g = table(&["alela", "veyran"]);
    perm(&mut g, P0, "Malcator, Purity Overseer");
    let golems = |g: &Game| {
        g.player(P0).perms.iter().filter(|&&m| g.perm(m).token && g.perm(m).ttypes.contains(&"golem")).count()
    };
    assert_eq!(golems(&g), 1);
    perm(&mut g, P0, "Mind Stone");
    perm(&mut g, P0, "Sol Ring");
    fire_trigger(&mut g, Event::EndStep, Call::Player { p: P0 }).unwrap();
    assert_eq!(golems(&g), 2, "its own Golem was the third artifact");
}

#[test]
fn airlift_chaplain_keeps_a_small_creature() {
    let mut g = table(&["alela", "veyran"]);
    for n in ["Island", "Island", "Initiates of the Ebon Hand"] {
        on_top(&mut g, P0, n);
    }
    let ch = perm(&mut g, P0, "Airlift Chaplain");
    let hand_names: Vec<&str> = g.player(P0).hand.iter().map(|&c| &*g.db.get(c).name).collect();
    assert_eq!(hand_names, vec!["Initiates of the Ebon Hand"]);
    assert_eq!(g.perm(ch).plus, 0);
}

#[test]
fn airlift_chaplain_counter_when_nothing_fits() {
    let mut g = table(&["alela", "veyran"]);
    for n in ["Island", "Island", "Island"] {
        on_top(&mut g, P0, n);
    }
    let ch = perm(&mut g, P0, "Airlift Chaplain");
    assert!(g.player(P0).hand.is_empty());
    assert_eq!(g.perm(ch).plus, 1);
}

#[test]
fn skycoach_crews_with_a_sick_creature() {
    let mut g = table(&["alela", "veyran"]);
    let sc = perm(&mut g, P0, "Strixhaven Skycoach");
    let c = take(&mut g, P0, "Plumecreed Mentor");
    zones::enter(&mut g, P0, c, zones::Enter::default()).unwrap(); // summoning sick
    fire_trigger(&mut g, Event::Crew, Call::Player { p: P0 }).unwrap();
    assert!(g.is_creature(sc));
    assert_eq!(values::epow(&g, sc), 3);
}

#[test]
fn stirring_hopesinger_counters_on_a_creature_spell_target() {
    let mut g = table(&["alela", "veyran"]);
    let h = perm(&mut g, P0, "Stirring Hopesinger");
    on_cast(&mut g, P0, "Last Gasp");
    assert_eq!(g.perm(h).plus, 1);
    on_cast(&mut g, P0, "Night's Whisper");
    assert_eq!(g.perm(h).plus, 1);
}

#[test]
fn desperate_futurescribe_pumps_another_creature() {
    let mut g = table(&["alela", "veyran"]);
    perm(&mut g, P0, "Desperate Futurescribe");
    let q = perm(&mut g, P0, "Plumecreed Mentor");
    fire_trigger(&mut g, Event::Crew, Call::Player { p: P0 }).unwrap();
    assert_eq!(g.perm(q).eot_pt, (1, 1), "no scry this turn: until end of turn");
    g.player_mut(P0).scry_turn = Some(g.turn_stamp());
    fire_trigger(&mut g, Event::Crew, Call::Player { p: P0 }).unwrap();
    assert_eq!(g.perm(q).plus, 1, "scried: a counter");
}

#[test]
fn reconstructed_thopter_unearths_once() {
    let mut g = table(&["alela", "veyran"]);
    lands(&mut g, P0, "Island", 2, false);
    let c = card(&g, "Reconstructed Thopter");
    g.player_mut(P0).gy.push(c);
    let f = g.registry.get(c).unwrap().gy_options.unwrap();
    let o = f(&mut g, c, P0, Some(false)).unwrap();
    assert_eq!(o.len(), 1);
    assert!(perform(&mut g, P0, &o[0]));
    let m = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).cd == Some(c)).unwrap();
    assert!(!g.perm(m).sick);
    fire_trigger(&mut g, Event::EndStep, Call::Player { p: P0 }).unwrap();
    assert!(!g.perm(m).on_bf);
    assert!(g.player(P0).exile.contains(&c));
}

// ======================================================== spells, sagas, Venser
#[test]
fn helping_hand_returns_tapped() {
    let mut g = table(&["alela", "veyran"]);
    let pm = card(&g, "Plumecreed Mentor");
    g.player_mut(P0).gy.push(pm);
    let hh = card(&g, "Helping Hand");
    alela::helping_hand(&mut g, P0, hh, &Default::default()).unwrap();
    let m = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).name == "Plumecreed Mentor").unwrap();
    assert!(g.perm(m).tapped);
}

#[test]
fn daydream_blinks_with_a_counter() {
    let mut g = table(&["alela", "veyran"]);
    perm(&mut g, P0, "Keeper of the Quiet Hour");
    let dd = card(&g, "Daydream");
    alela::daydream(&mut g, P0, dd, &Default::default()).unwrap();
    let k = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).name == "Keeper of the Quiet Hour").unwrap();
    assert_eq!(g.perm(k).plus, 1);
    let j = alela::jace_token(&g, P0).unwrap();
    assert_eq!(g.perm(j).loyalty, Some(4), "empowered again");
}

#[test]
fn ajani_chapters() {
    let mut g = table(&["alela", "veyran"]);
    let t = perm(&mut g, P1, "Grave Titan");
    let aj = perm(&mut g, P0, "Ajani Fells the Godsire");
    assert!(!g.perm(t).on_bf);
    upkeep(&mut g, P0);
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).token && g.perm(m).ttypes.contains(&"cat")));
    upkeep(&mut g, P0);
    assert!(!g.perm(aj).on_bf);
}

#[test]
fn venser_emblem_exiles_per_spell() {
    let mut g = table(&["alela", "veyran"]);
    g.player_mut(P0).emblems.push("venser");
    let t = perm(&mut g, P1, "Grave Titan");
    on_cast(&mut g, P0, "Sol Ring");
    assert!(!g.perm(t).on_bf);
}

#[test]
fn venser_minus_one_before_a_big_attack() {
    let mut g = table(&["alela", "veyran"]);
    let v = perm(&mut g, P0, "Venser, the Sojourner");
    for n in ["Pileated Provisioner", "Grave Titan"] {
        perm(&mut g, P0, n);
    }
    let l = g.perm(v).loyalty.unwrap();
    fire_trigger(&mut g, Event::Crew, Call::Player { p: P0 }).unwrap();
    assert_eq!(g.perm(v).loyalty, Some(l - 1));
    assert_eq!(g.player(P0).unbl_all, Some(g.turn_stamp()));
}

// ======================================================== the plan
#[test]
fn alela_adds_a_faerie_to_artifacts_and_enchantments() {
    let mut g = table(&["alela", "veyran"]);
    let stone = card(&g, "Mind Stone");
    let base = decks::deck_prio(&g, P0, stone);
    alela_out(&mut g, P0);
    assert_eq!(decks::deck_prio(&g, P0, stone), 90.min(base + 10));
    let ws = take(&mut g, P0, "Wispdrinker Vampire");
    zones::enter(&mut g, P0, ws, zones::Enter::default()).unwrap();
    assert_eq!(alela::faerie_bonus(&g, P0), 16);
    let cmd = g.player(P0).cmd;
    assert!(decks::deck_prio(&g, P0, cmd) >= 85);
}

#[test]
fn card_prios_of_the_99() {
    let mut g = table(&["alela", "veyran"]);
    let op = card(&g, "Opposition");
    perm(&mut g, P0, "Plumecreed Mentor");
    let f = g.registry.get(op).unwrap().prio.unwrap();
    assert_eq!(f(&g, P0, op), 39);
    let sp = card(&g, "Static Prison");
    let f = g.registry.get(sp).unwrap().prio.unwrap();
    assert_eq!(f(&g, P0, sp), 0, "nothing to exile");
    let kj = card(&g, "Karmic Justice");
    assert_eq!((g.registry.get(kj).unwrap().prio.unwrap())(&g, P0, kj), 30);
    let hh = card(&g, "Helping Hand");
    assert_eq!((g.registry.get(hh).unwrap().prio.unwrap())(&g, P0, hh), 0, "nothing to return");
}

#[test]
fn fatehold_charm_draws_at_the_end_of_the_turn_before_yours() {
    let mut g = table(&["alela", "veyran"]);
    lands(&mut g, P0, "Plains", 1, false);
    lands(&mut g, P0, "Island", 1, false);
    let c = hand(&mut g, P0, &["Fatehold Charm"])[0];
    assert!(hand_opts(&mut g, c, P0, None).is_empty(), "not on your own turn");
    g.active = Some(P1);
    let o = hand_opts(&mut g, c, P0, None);
    assert_eq!(o.len(), 1);
    let n = g.player(P0).hand.len();
    assert!(perform(&mut g, P0, &o[0]));
    assert_eq!(g.player(P0).hand.len(), n, "the Charm out, a card in");
    let j = alela::jace_token(&g, P0).unwrap();
    assert_eq!(g.perm(j).loyalty, Some(2));
}

#[test]
fn emry_costs_less_with_artifacts_and_casts_from_the_graveyard() {
    let mut g = table(&["alela", "veyran"]);
    let emry = card(&g, "Emry, Lurker of the Loch");
    perm(&mut g, P0, "Mind Stone");
    g.player_mut(P0).treasures = 1;
    assert_eq!(sim_core::cardcode::self_cost(&g, P0, emry), -2);
    let e = perm(&mut g, P0, "Emry, Lurker of the Loch");
    lands(&mut g, P0, "Plains", 2, false);
    let ring = card(&g, "Sol Ring");
    g.player_mut(P0).gy.push(ring);
    let o = perm_opts(&mut g, e, P0, Some(true)); // (its enter trigger milled four: other artifacts may show up)
    let o = o.into_iter().find(|x| x.label == "Emry: cast Sol Ring from the graveyard").expect("Sol Ring");
    assert!(perform(&mut g, P0, &o));
    assert!(g.perm(e).tapped);
    assert!(names(&g, P0).contains(&"Sol Ring"));
}
