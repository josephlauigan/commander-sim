//! Your Galadriel deck (impls/galadriel.rs, Python's cards/impl/galadriel.py) on hand-built positions: Python's
//! tests/test_galadriel.py, plus the deck plan's choices (cast priorities, protection, wipe answers, the answers when
//! attacked, the tappers before an opponent's combat).
//!
//! Skipped: the practice-mode tests (a person choosing Alliance's mode, Lin Sivvi's X, the searcher's card, the
//! Whipcorder morph, Mirror Entity's X, Austere Command's modes, Mentor of the Meek's payment, the creature type, the
//! Secluded Courtyard mana in a person's pool): practice mode comes with phase 9. Mirror Entity and the morph are
//! checked here through their effects (`mirror`, `enter_face_down`, `turn_face_up`) instead.

use sim_core::ai::{act, brain, decks};
use sim_core::engine::{cast, combat, mana, removal, turn, values, zones};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::galadriel as gal;
use sim_core::state::{Ctx, DataKey, Game, Step, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

/// Galadriel from the command zone onto the battlefield (Python's `galadriel(g, p)`)
fn galadriel(g: &mut Game, p: PlayerId) -> PermId {
    let c = g.player(p).cmd;
    let m = zones::enter(g, p, c, zones::Enter::default()).unwrap();
    g.perm_mut(m).is_cmd = true;
    g.player_mut(p).cmd_in_zone = false;
    m
}

/// the options a permanent's `options` hook offers p (post: Some(true) in the main phase)
fn opts(g: &mut Game, m: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.perm(m).cd.unwrap()).and_then(|i| i.options).expect("the card has options");
    f(g, m, p, post).unwrap()
}

fn perform(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

fn named(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).name == name)
}

fn set_ctype(g: &mut Game, m: PermId, t: &'static str) {
    g.perm_mut(m).data.set(DataKey::Ctype, Val::Str(t));
}

// ======================================================== Alliance
#[test]
fn each_mode_once_a_turn() {
    let mut g = table(&["galadriel", "veyran", "sauron"]);
    let gal = galadriel(&mut g, P0);
    for n in ["Llanowar Elves", "Fyndhorn Elves", "Elvish Mystic", "Jhovall Queen"] {
        perm(&mut g, P0, n);
    }
    let mut ch = gal::chosen(&g, gal);
    ch.sort();
    assert_eq!(ch, vec!["counters", "draw", "mana"], "the fourth: nothing left");
    assert_eq!(g.player(P0).milestone.get("alliance"), Some(&g.player(P0).turns));
}

#[test]
fn panharmonicon_doubles_alliance() {
    let mut g = table(&["galadriel", "veyran", "sauron"]);
    let gal = galadriel(&mut g, P0);
    perm(&mut g, P0, "Panharmonicon");
    perm(&mut g, P0, "Llanowar Elves");
    assert_eq!(gal::chosen(&g, gal).len(), 2);
}

#[test]
fn panharmonicon_doubles_cathars_crusade() {
    let mut g = table(&["galadriel", "veyran", "sauron"]);
    perm(&mut g, P0, "Cathars' Crusade");
    perm(&mut g, P0, "Panharmonicon");
    let a = perm(&mut g, P0, "Jhovall Queen");
    perm(&mut g, P0, "Llanowar Elves");
    assert_eq!(g.perm(a).plus, 4, "two for itself, two for the Elves");
}

#[test]
fn alliance_takes_mana_in_your_main_phase_for_a_spell_within_reach() {
    // a 4-drop in hand with one land: {G}{G}{G} (3.2) beats scry and draw (2.6) and one creature's counter
    let mut g = table(&["galadriel", "veyran"]);
    lands(&mut g, P0, "Forest", 1, false);
    hand(&mut g, P0, &["Elspeth, Sun's Champion"]); // mana value 6: out of reach
    assert_eq!(gal::alliance_value(&g, P0, "mana"), 0.6);
    hand(&mut g, P0, &["Ramosian Captain"]); // mana value 4: within three
    assert_eq!(gal::alliance_value(&g, P0, "mana"), 3.2);
    assert_eq!(gal::alliance_value(&g, P0, "draw"), 2.6);
    g.step = Step::Combat;
    assert_eq!(gal::alliance_value(&g, P0, "mana"), 0.2);
    let gl = galadriel(&mut g, P0);
    g.step = Step::Main1;
    perm(&mut g, P0, "Llanowar Elves");
    assert_eq!(gal::chosen(&g, gl), vec!["mana"]);
    assert_eq!(g.player(P0).floating.g, 3);
}

// ======================================================== Rebels
#[test]
fn a_searcher_finds_a_rebel_within_its_cap() {
    let mut g = table(&["galadriel", "veyran", "sauron"]);
    lands(&mut g, P0, "Plains", 3, false);
    let m = perm(&mut g, P0, "Ramosian Sergeant");
    let before = g.player(P0).perms.clone();
    let o = opts(&mut g, m, P0, Some(true));
    assert!(!o.is_empty() && perform(&mut g, P0, &o[0]));
    let new: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|x| !before.contains(x)).collect();
    assert_eq!(new.len(), 1);
    let d = g.db.get(g.perm(new[0]).cd.unwrap());
    assert!(d.has_subtype("rebel") && d.cmc <= 2, "{}", d.name);
    assert!(g.perm(m).tapped);
}

#[test]
fn searchers_wait_for_an_opponents_end_step_on_their_own_turn() {
    // the end-of-turn window (post None) on your own turn: no activation
    let mut g = table(&["galadriel", "veyran"]);
    lands(&mut g, P0, "Plains", 3, false);
    let m = perm(&mut g, P0, "Ramosian Sergeant");
    assert!(opts(&mut g, m, P0, None).is_empty());
    g.active = Some(P1);
    assert_eq!(opts(&mut g, m, P0, None).len(), 1);
}

#[test]
fn maskwood_makes_every_creature_a_rebel() {
    let mut g = table(&["galadriel", "veyran", "sauron"]);
    let elves = card(&g, "Llanowar Elves");
    assert!(!gal::card_is(&g, P0, elves, "rebel"));
    perm(&mut g, P0, "Maskwood Nexus");
    assert!(gal::card_is(&g, P0, elves, "rebel"));
    let f = perm(&mut g, P0, "Fyndhorn Elves");
    assert!(values::has_type(&g, f, "rebel"));
}

#[test]
fn lin_sivvi_fetches_with_x_and_the_best_rebel() {
    // the AI's side of Python's practice test: X is the fetched card's mana value, at most the mana available
    let mut g = table(&["galadriel", "veyran", "sauron"]);
    let lin = perm(&mut g, P0, "Lin Sivvi, Defiant Hero");
    lands(&mut g, P0, "Plains", 3, false);
    let o = opts(&mut g, lin, P0, Some(true));
    assert_eq!(o.len(), 1);
    assert!(o[0].label.starts_with("Lin Sivvi (X = "), "{}", o[0].label);
    let before = g.player(P0).perms.clone();
    assert!(perform(&mut g, P0, &o[0]));
    assert!(g.perm(lin).tapped);
    let new: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|x| !before.contains(x)).collect();
    assert_eq!(new.len(), 1);
    let d = g.db.get(g.perm(new[0]).cd.unwrap());
    assert!(d.has_subtype("rebel") && d.cmc <= 3, "{}", d.name);
}

#[test]
fn revivalist_returns_a_rebel() {
    let mut g = table(&["galadriel", "veyran", "sauron"]);
    lands(&mut g, P0, "Plains", 6, false);
    let m = perm(&mut g, P0, "Ramosian Revivalist");
    let c = take(&mut g, P0, "Ramosian Captain");
    g.player_mut(P0).gy.push(c);
    let o = opts(&mut g, m, P0, Some(true));
    assert!(!o.is_empty() && perform(&mut g, P0, &o[0]));
    assert!(g.player(P0).perms.iter().any(|&x| g.perm(x).cd == Some(c)));
}

#[test]
fn rebel_values_follow_the_board() {
    let mut g = table(&["galadriel", "sauron"]);
    let law = card(&g, "Lawbringer");
    assert_eq!(gal::rebel_value(&g, P0, law), 1.5, "no red threat");
    let lin = card(&g, "Lin Sivvi, Defiant Hero");
    assert_eq!(gal::rebel_value(&g, P0, lin), 9.0);
    let cmdr = card(&g, "Ramosian Commander");
    assert_eq!(gal::rebel_value(&g, P0, cmdr), 7.0);
    perm(&mut g, P0, "Ramosian Sergeant");
    perm(&mut g, P0, "Ramosian Lieutenant");
    assert!((gal::rebel_value(&g, P0, cmdr) - 5.8).abs() < 1e-9, "a second searcher out: -1.2");
}

// ======================================================== creature types
#[test]
fn kindred_discovery_names_a_type() {
    let mut g = table(&["galadriel", "veyran", "sauron"]);
    let kd = perm(&mut g, P0, "Kindred Discovery");
    let t = gal::ctype(&g, kd).unwrap();
    assert!(t == "human" || t == "rebel", "{t}");
    assert!(g.selfpt);
    let n = g.player(P0).hand.len();
    perm(&mut g, P0, "Ramosian Sergeant"); // a Human Rebel
    assert_eq!(g.player(P0).hand.len(), n + 1);
}

#[test]
fn sauron_still_names_orc() {
    let mut g = table(&["sauron", "veyran"]);
    let kd = perm(&mut g, P0, "Kindred Discovery");
    assert_eq!(gal::ctype(&g, kd), Some("orc"));
}

#[test]
fn door_of_destinies_and_the_banner() {
    let mut g = table(&["galadriel", "veyran", "sauron"]);
    let door = perm(&mut g, P0, "Door of Destinies");
    set_ctype(&mut g, door, "rebel");
    let ban = perm(&mut g, P0, "Vanquisher's Banner");
    set_ctype(&mut g, ban, "rebel");
    let r = perm(&mut g, P0, "Ramosian Sergeant");
    let n = g.player(P0).hand.len();
    let c = hand(&mut g, P0, &["Ramosian Lieutenant"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert_eq!(g.perm(door).data.int(DataKey::Charge), 1);
    assert_eq!(g.player(P0).hand.len(), n + 1, "drew one (the Banner)");
    assert_eq!(values::epow(&g, r), 1 + 1 + 1, "base 1, the Door's counter, the Banner");
}

#[test]
fn patchwork_banner_pumps_the_named_type() {
    let mut g = table(&["galadriel", "veyran"]);
    let b = perm(&mut g, P0, "Patchwork Banner");
    set_ctype(&mut g, b, "elf");
    let e = perm(&mut g, P0, "Llanowar Elves");
    let q = perm(&mut g, P0, "Jhovall Queen");
    assert_eq!(values::epow(&g, e), g.perm(e).pow + 1, "an Elf: +1/+1");
    assert_eq!(values::epow(&g, q), g.perm(q).pow, "not an Elf");
}

#[test]
fn secluded_courtyard_colours_only_for_the_type() {
    let mut g = table(&["galadriel", "veyran", "sauron"]);
    let cd = card(&g, "Secluded Courtyard");
    let l = g.add_land(P0, cd, false);
    g.land_mut(l).data.set(DataKey::Ctype, Val::Str("elf"));
    g.pay_for = Some(card(&g, "Llanowar Elves"));
    assert!(mana::land_colors_now(&g, P0, l).has('G'));
    g.pay_for = Some(card(&g, "Counterspell"));
    assert!(mana::land_colors_now(&g, P0, l).is_empty());
    g.pay_for = None;
}

// ======================================================== other cards
#[test]
fn crackdown_keeps_big_nonwhite_creatures_tapped() {
    let mut g = table(&["galadriel", "sauron"]);
    perm(&mut g, P0, "Crackdown");
    let big = perm(&mut g, P1, "Grave Titan");
    g.perm_mut(big).tapped = true;
    let small = perm(&mut g, P1, "Orcish Bowmasters");
    g.perm_mut(small).tapped = true;
    turn::step_start(&mut g, P1).unwrap();
    assert!(g.perm(big).tapped);
    assert!(!g.perm(small).tapped);
}

#[test]
fn cho_manno_takes_no_damage() {
    let mut g = table(&["galadriel", "veyran"]);
    let cho = perm(&mut g, P0, "Cho-Manno, Revolutionary");
    removal::apply_removal(&mut g, Some(P1), cho, "dmg3", None).unwrap();
    assert!(g.perm(cho).on_bf);
}

#[test]
fn knight_of_the_holy_nimbus_regenerates_unless_paid() {
    let mut g = table(&["galadriel", "veyran"]);
    let k = perm(&mut g, P0, "Knight of the Holy Nimbus");
    zones::die(&mut g, k, "destroy").unwrap();
    assert!(g.perm(k).on_bf && g.perm(k).tapped, "the opponent had no mana");
    let k2 = perm(&mut g, P0, "Knight of the Holy Nimbus");
    lands(&mut g, P1, "Island", 2, false);
    zones::die(&mut g, k2, "destroy").unwrap();
    assert!(!g.perm(k2).on_bf, "paid {{2}}");
}

#[test]
fn defiant_vanguard_kills_what_it_blocks() {
    let mut g = table(&["sauron", "galadriel"]);
    let (s, p) = (P0, P1);
    let t = perm(&mut g, s, "Grave Titan");
    let vg = perm(&mut g, p, "Defiant Vanguard");
    combat::resolve_combat(&mut g, s, &[t], p, &[]).unwrap();
    assert!(!g.perm(t).on_bf);
    assert!(!g.perm(vg).on_bf);
}

#[test]
fn amrou_seekers_blockers() {
    let mut g = table(&["galadriel", "veyran"]);
    let a = perm(&mut g, P0, "Amrou Seekers");
    let gs = perm(&mut g, P1, "Guttersnipe");
    assert!(!combat::can_block(&g, gs, a));
    let q = perm(&mut g, P0, "Jhovall Queen");
    assert!(combat::can_block(&g, q, a));
}

#[test]
fn abduction_steals_and_returns_on_death() {
    let mut g = table(&["galadriel", "sauron"]);
    let t = perm(&mut g, P1, "Grave Titan");
    g.perm_mut(t).tapped = true;
    perm(&mut g, P0, "Abduction");
    assert_eq!(g.perm(t).owner, P0);
    assert!(g.perm(t).on_bf && !g.perm(t).tapped);
    let cd = g.perm(t).cd;
    zones::die(&mut g, t, "destroy").unwrap();
    assert!(g.player(P1).perms.iter().any(|&x| g.perm(x).cd == cd), "back under its owner's control");
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert!(named(&g, P0, "Abduction").is_none(), "the Aura fell off");
}

#[test]
fn abduction_leaving_gives_the_creature_back() {
    let mut g = table(&["galadriel", "sauron"]);
    let t = perm(&mut g, P1, "Grave Titan");
    let a = perm(&mut g, P0, "Abduction");
    assert_eq!(g.perm(t).owner, P0);
    zones::leave(&mut g, a).unwrap();
    assert_eq!(g.perm(t).owner, P1);
    assert!(g.player(P1).perms.contains(&t));
}

#[test]
fn bribery() {
    let mut g = table(&["galadriel", "sauron"]);
    let c = hand(&mut g, P0, &["Bribery"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert!(g.player(P0).perms.iter().any(|&x| g.is_creature(x) && g.perm(x).orig == P1));
}

#[test]
fn bribery_takes_the_biggest_threat() {
    let g = table(&["galadriel", "sauron"]);
    let (q, c) = gal::bribery_pick(&g, P0);
    assert_eq!(q, Some(P1));
    let c = c.unwrap();
    let best = g
        .player(P1)
        .library
        .iter()
        .filter(|&&x| g.db.get(x).creature)
        .map(|&x| {
            let d = g.db.get(x);
            (d.bomb * 2) as f64 + d.cmc as f64 + if d.tag(sim_core::tag::Tag::Fly) { 3.0 } else { 0.0 }
        })
        .fold(f64::MIN, f64::max);
    let d = g.db.get(c);
    let v = (d.bomb * 2) as f64 + d.cmc as f64 + if d.tag(sim_core::tag::Tag::Fly) { 3.0 } else { 0.0 };
    assert_eq!(v, best);
}

#[test]
fn eerie_interlude_returns_at_the_end_step() {
    // Python casts it with ctx {'targets': [m]} (practice mode's choice); the AI picks by enter value, so the
    // exile is driven directly here
    let mut g = table(&["galadriel", "veyran"]);
    let m = perm(&mut g, P0, "Recruiter of the Guard");
    let cd = g.perm(m).cd;
    gal::interlude(&mut g, P0, &[m]).unwrap();
    assert!(!g.perm(m).on_bf);
    turn::end_step(&mut g, P0).unwrap();
    assert!(g.player(P0).perms.iter().any(|&x| g.perm(x).cd == cd));
}

#[test]
fn voice_of_resurgence_token() {
    let mut g = table(&["galadriel", "veyran"]);
    perm(&mut g, P0, "Voice of Resurgence");
    perm(&mut g, P0, "Jhovall Queen");
    let bolt = card(&g, "Lightning Bolt");
    cast::on_cast(&mut g, P1, bolt).unwrap(); // during your turn
    let tok = g.player(P0).perms.iter().copied().find(|&x| g.perm(x).token).unwrap();
    assert_eq!(values::epow(&g, tok), 3);
}

#[test]
fn voice_of_resurgence_dies_into_a_token() {
    let mut g = table(&["galadriel", "veyran"]);
    let v = perm(&mut g, P0, "Voice of Resurgence");
    zones::die(&mut g, v, "destroy").unwrap();
    let tok = g.player(P0).perms.iter().copied().find(|&x| g.perm(x).token).unwrap();
    assert_eq!(values::epow(&g, tok), 1, "it counts itself");
}

#[test]
fn mangara_draws_on_the_second_spell() {
    let mut g = table(&["galadriel", "veyran"]);
    perm(&mut g, P0, "Mangara, the Diplomat");
    let n = g.player(P0).hand.len();
    g.player_mut(P1).spells_this_turn = 2;
    let bolt = card(&g, "Lightning Bolt");
    cast::on_cast(&mut g, P1, bolt).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1);
}

#[test]
fn tocasia_draws_once_a_turn() {
    let mut g = table(&["galadriel", "veyran"]);
    perm(&mut g, P0, "Tocasia's Welcome");
    let n = g.player(P0).hand.len();
    perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Fyndhorn Elves");
    assert_eq!(g.player(P0).hand.len(), n + 1);
}

#[test]
fn mana_springleaf_drum_and_elvish_archdruid() {
    let mut g = table(&["galadriel", "veyran"]);
    let d = perm(&mut g, P0, "Springleaf Drum");
    assert_eq!(sim_core::cardcode::dyn_mana_perm(&g, P0, d), Some(0), "no creature to tap");
    let q = perm(&mut g, P0, "Jhovall Queen");
    assert_eq!(sim_core::cardcode::dyn_mana_perm(&g, P0, d), Some(1));
    assert!(mana::pay(&mut g, P0, 1, "", false).unwrap());
    assert!(g.perm(q).tapped);
    let a = perm(&mut g, P0, "Elvish Archdruid");
    perm(&mut g, P0, "Farhaven Elf");
    assert_eq!(sim_core::cardcode::dyn_mana_perm(&g, P0, a), Some(2));
}

#[test]
fn return_to_dust_takes_two_in_your_main_phase() {
    let mut g = table(&["galadriel", "veyran"]);
    let a = perm(&mut g, P1, "Sol Ring");
    let b = perm(&mut g, P1, "Mind Stone");
    let c = hand(&mut g, P0, &["Return to Dust"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx { target: Some(a), ..Ctx::default() }).unwrap();
    assert!(!g.perm(a).on_bf && !g.perm(b).on_bf);
}

#[test]
fn unbreakable_formation_addendum() {
    let mut g = table(&["galadriel", "veyran"]);
    let m = perm(&mut g, P0, "Jhovall Queen");
    let c = hand(&mut g, P0, &["Unbreakable Formation"])[0];
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    assert_eq!(g.perm(m).plus, 1);
    assert!(values::indestructible(&g, m));
    g.step = Step::Combat; // outside your main phase: no addendum
    gal::formation(&mut g, P0, false);
    assert_eq!(g.perm(m).plus, 1);
}

#[test]
fn elspeth_emblem_flies() {
    let mut g = table(&["galadriel", "veyran"]);
    let m = perm(&mut g, P0, "Jhovall Queen");
    gal::elspeth_emblem(&mut g, P0);
    assert!(sim_core::dsl::has_kw(&g, m, "flying"));
    assert_eq!(values::epow(&g, m), 4 + 2);
}

#[test]
fn mirror_entity_makes_creatures_x_x_of_every_type() {
    let mut g = table(&["galadriel", "veyran", "sauron"]);
    perm(&mut g, P0, "Mirror Entity");
    let q = perm(&mut g, P0, "Jhovall Queen");
    gal::mirror(&mut g, P0, 5).unwrap();
    assert_eq!((values::epow(&g, q), values::etgh(&g, q)), (5, 5));
    assert!(values::has_type(&g, q, "goblin"));
}

#[test]
fn whipcorder_face_down_and_up() {
    let mut g = table(&["galadriel", "veyran", "sauron"]);
    let c = take(&mut g, P0, "Whipcorder");
    let m = gal::enter_face_down(&mut g, P0, c).unwrap();
    assert!(g.perm(m).neutered && g.perm(m).data.truthy(DataKey::Facedown));
    gal::turn_face_up(&mut g, P0, m);
    assert!(!g.perm(m).neutered && g.perm(m).data.get(DataKey::Facedown).is_none());
}

#[test]
fn planar_genesis_puts_a_land_onto_the_battlefield_tapped() {
    let mut g = table(&["galadriel", "veyran"]);
    for n in ["Island", "Counterspell", "Forest", "Cultivate"] {
        let c = take(&mut g, P0, n);
        g.player_mut(P0).library.push(c);
    }
    let c = hand(&mut g, P0, &["Planar Genesis"])[0];
    let lib = g.player(P0).library.len();
    cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
    let l = *g.player(P0).lands.last().unwrap();
    assert_eq!(&*g.db.get(g.land(l).cd).name, "Forest", "the first land of the top four");
    assert!(g.land(l).tapped);
    assert_eq!(g.player(P0).library.len(), lib - 1);
}

// ======================================================== the plan
#[test]
fn galadriel_casts_the_engine_first() {
    let mut g = table(&["galadriel", "veyran"]);
    let c = |g: &Game, n: &str| card(g, n);
    let sgt = c(&g, "Ramosian Sergeant");
    assert_eq!(decks::deck_prio(&g, P0, sgt), 73, "no searcher out yet: +10");
    perm(&mut g, P0, "Ramosian Lieutenant");
    assert_eq!(decks::deck_prio(&g, P0, sgt), 63);
    let cs = c(&g, "Counterspell");
    assert_eq!(decks::deck_prio(&g, P0, cs), 0, "held for its window");
    let f = c(&g, "Unbreakable Formation");
    assert_eq!(decks::deck_prio(&g, P0, f), 0);
    let ring = c(&g, "Sol Ring");
    assert_eq!(decks::deck_prio(&g, P0, ring), 82);
    let cmd = g.player(P0).cmd;
    assert_eq!(decks::deck_prio(&g, P0, cmd), 66);
    g.player_mut(P0).turns = 3;
    assert_eq!(decks::deck_prio(&g, P0, cmd), 84);
    let pan = c(&g, "Panharmonicon");
    assert_eq!(decks::deck_prio(&g, P0, pan), 50, "one creature, no Galadriel");
    perm(&mut g, P0, "Llanowar Elves");
    assert_eq!(decks::deck_prio(&g, P0, pan), 72);
}

#[test]
fn galadriel_abduction_wants_a_real_threat() {
    let mut g = table(&["galadriel", "sauron"]);
    let ab = card(&g, "Abduction");
    assert_eq!(decks::deck_prio(&g, P0, ab), 0);
    let t = perm(&mut g, P1, "Grave Titan");
    let v = values::pval(&g, t);
    assert_eq!(decks::deck_prio(&g, P0, ab), 80.min((40.0 + 6.0 * v) as i32));
}

#[test]
fn unbreakable_formation_saves_a_key_creature_from_destroy() {
    let mut g = table(&["galadriel", "sauron"]);
    let m = perm(&mut g, P0, "Grave Titan");
    lands(&mut g, P0, "Plains", 3, false);
    hand(&mut g, P0, &["Unbreakable Formation"]);
    assert!(values::pval(&g, m) >= 4.0);
    assert!(!decks::protect_response(&mut g, P0, m, "exile", Some(P1), None).unwrap(), "not against exile");
    assert!(decks::protect_response(&mut g, P0, m, "destroy", Some(P1), None).unwrap());
    assert!(values::indestructible(&g, m));
}

#[test]
fn eerie_interlude_answers_exile() {
    let mut g = table(&["galadriel", "sauron"]);
    let t = perm(&mut g, P0, "Grave Titan");
    lands(&mut g, P0, "Plains", 3, false);
    hand(&mut g, P0, &["Eerie Interlude"]);
    assert!(gal::galadriel_protect(&mut g, P0, t, "exile", Some(P1), None).unwrap());
    assert!(!g.perm(t).on_bf);
    assert_eq!(g.eot_returns.len(), 1);
    turn::end_step(&mut g, P0).unwrap();
    assert!(named(&g, P0, "Grave Titan").is_some(), "back at the end step");
}

#[test]
fn wipe_answers_indestructible_against_destroy() {
    let mut g = table(&["galadriel", "sauron"]);
    perm(&mut g, P0, "Grave Titan");
    let q = perm(&mut g, P0, "Jhovall Queen");
    lands(&mut g, P0, "Plains", 3, false);
    hand(&mut g, P0, &["Unbreakable Formation"]);
    assert_eq!(decks::wipe_response(&mut g, P0, "exile", P1).unwrap(), None, "no Eerie Interlude to cast");
    assert_eq!(decks::wipe_response(&mut g, P0, "destroy", P1).unwrap(), Some("indes"));
    assert!(values::indestructible(&g, q));
}

#[test]
fn ballista_squad_shoots_an_attacker() {
    let mut g = table(&["sauron", "galadriel"]);
    let (s, p) = (P0, P1);
    perm(&mut g, p, "Ballista Squad");
    lands(&mut g, p, "Plains", 8, false);
    let t = perm(&mut g, s, "Grave Titan");
    let a = gal::attack_answers(&g, p, s, &[t]);
    assert_eq!(a.len(), 1);
    assert!(brain::attack_response(&mut g, p, s, &[t]).unwrap());
    assert!(!g.perm(t).on_bf);
    assert_eq!(g.player(p).stats.get("attack_removal"), Some(&1));
}

#[test]
fn tappers_tap_the_attacker_before_an_opponents_combat() {
    let mut g = table(&["sauron", "galadriel"]);
    let (s, p) = (P0, P1);
    perm(&mut g, p, "Whipcorder");
    lands(&mut g, p, "Plains", 1, false);
    let t = perm(&mut g, s, "Grave Titan");
    gal::precombat(&mut g, p).unwrap();
    assert!(g.perm(t).tapped);
}

#[test]
fn lawbringer_exiles_a_red_threat() {
    let mut g = table(&["galadriel", "veyran"]);
    let lb = perm(&mut g, P0, "Lawbringer");
    let t = perm(&mut g, P1, "Guttersnipe");
    let o = opts(&mut g, lb, P0, Some(true));
    if values::pval(&g, t) >= 3.0 {
        assert_eq!(o.len(), 1);
        assert!(perform(&mut g, P0, &o[0]));
        assert!(!g.perm(t).on_bf && !g.perm(lb).on_bf);
    } else {
        assert!(o.is_empty());
    }
    assert!(opts(&mut g, lb, P0, Some(false)).is_empty(), "not before combat");
}

#[test]
fn maskwood_makes_a_shapeshifter_at_an_opponents_end_step() {
    let mut g = table(&["galadriel", "veyran"]);
    let mw = perm(&mut g, P0, "Maskwood Nexus");
    lands(&mut g, P0, "Plains", 3, false);
    assert!(opts(&mut g, mw, P0, None).is_empty(), "not on your own turn");
    g.active = Some(P1);
    let o = opts(&mut g, mw, P0, None);
    assert_eq!(o.len(), 1);
    assert!(matches!(o[0].act, Some(Action::Ability { .. })));
    assert!(perform(&mut g, P0, &o[0]));
    let tok = g.player(P0).perms.iter().copied().find(|&x| g.perm(x).token).unwrap();
    assert!(values::has_type(&g, tok, "rebel") && values::has_type(&g, tok, "elf"));
}
