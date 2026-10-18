//! Tier 6 lands batch (impls/t6_lands.rs) on hand-built positions. Each expectation follows the card's Oracle text.

use sim_core::engine::{mana, turn};
use sim_core::ids::PlayerId;
use sim_core::state::{DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn pod() -> Game {
    table(&["lathril-golgari-elves", "veyran"])
}

fn play_land(g: &mut Game, p: PlayerId, name: &str) {
    let c = take(g, p, name);
    turn::play_land_card(g, p, c, "plays").unwrap();
}

fn names(g: &Game, p: PlayerId) -> Vec<String> {
    g.player(p).lands.iter().map(|&l| g.db.get(g.land(l).cd).name.to_string()).collect()
}

// ------------------------------------------------------------------ City of Brass
#[test]
fn city_of_brass_deals_one_damage_each_time_it_is_tapped() {
    let mut g = pod();
    lands(&mut g, P0, "City of Brass", 1, false);
    let life = g.player(P0).life;
    assert!(mana::pay(&mut g, P0, 1, "", false).unwrap());
    assert_eq!(g.player(P0).life, life - 1, "1 damage when it becomes tapped");
    assert!(g.player(P0).lands.iter().all(|&l| g.land(l).tapped));
}

// ------------------------------------------------------------------ City of Traitors
#[test]
fn city_of_traitors_is_sacrificed_when_you_play_another_land() {
    let mut g = pod();
    lands(&mut g, P0, "City of Traitors", 1, false);
    assert_eq!(mana::total_mana(&g, P0, false), 2, "{{T}}: {{C}}{{C}}");
    play_land(&mut g, P0, "Forest");
    assert_eq!(names(&g, P0), ["Forest"]);
    assert!(g.player(P0).gy.contains(&card(&g, "City of Traitors")));
    // the City itself being played does not sacrifice it, another land afterwards does; the opponent's lands do not
    play_land(&mut g, P0, "City of Traitors");
    assert_eq!(names(&g, P0), ["Forest", "City of Traitors"]);
    play_land(&mut g, P1, "Island");
    assert_eq!(names(&g, P0), ["Forest", "City of Traitors"]);
}

// ------------------------------------------------------------------ Gemstone Caverns
#[test]
fn gemstone_caverns_makes_colourless_unless_it_has_a_luck_counter() {
    let mut g = pod();
    let l = lands(&mut g, P0, "Gemstone Caverns", 1, false)[0];
    let u = mana::mana_units(&g, P0, false);
    assert_eq!(u.len(), 1);
    assert!(u[0].cols.is_empty(), "{{T}}: Add {{C}}");
    g.land_mut(l).data.set(DataKey::Counters, Val::Int(1)); // a luck counter
    let u = mana::mana_units(&g, P0, false);
    assert_eq!(u[0].cols, g.player(P0).ident, "with a luck counter: one mana of any colour");
    assert_eq!(u[0].amt, 1);
}

// ------------------------------------------------------------------ Abstergo Entertainment
#[test]
fn abstergo_taps_for_colourless_and_filters_one_mana_into_any_colour() {
    let mut g = pod();
    lands(&mut g, P0, "Abstergo Entertainment", 1, false);
    let u = mana::mana_units(&g, P0, false);
    assert!(u.len() == 1 && u[0].cols.is_empty(), "alone: {{T}}: Add {{C}} only (nothing pays the {{1}})");
    lands(&mut g, P0, "Forest", 1, false);
    let u = mana::mana_units(&g, P0, false);
    assert_eq!(u.len(), 2);
    assert_eq!(u.iter().map(|x| x.amt).sum::<u32>(), 2, "one mana per land");
    assert!(u.iter().any(|x| x.cols == g.player(P0).ident), "{{1}},{{T}}: one mana of any colour");
}

#[test]
fn abstergo_exiles_itself_returns_a_historic_card_and_exiles_all_graveyards() {
    let mut g = pod();
    let a = lands(&mut g, P0, "Abstergo Entertainment", 1, false)[0];
    lands(&mut g, P0, "Forest", 3, false);
    let sol = take(&mut g, P0, "Sol Ring");
    let bear = take(&mut g, P0, "Llanowar Elves");
    g.player_mut(P0).gy.extend([sol, bear]);
    let junk = take(&mut g, P1, "Llanowar Elves");
    g.player_mut(P1).gy.push(junk);
    let o = sim_core::cardcode::land_options(&mut g, P0, None).unwrap();
    assert_eq!(o.iter().map(|x| x.label.as_str()).collect::<Vec<_>>(), ["Abstergo Entertainment"]);
    assert!(sim_core::ai::act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert!(!g.player(P0).lands.contains(&a), "exiled as a cost");
    assert!(g.player(P0).exile.contains(&card(&g, "Abstergo Entertainment")));
    assert!(g.player(P0).hand.contains(&sol), "the historic card returns");
    assert!(g.player(P0).gy.is_empty() && g.player(P1).gy.is_empty(), "all graveyards exiled");
    assert!(g.player(P0).exile.contains(&bear) && g.player(P1).exile.contains(&junk));
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.land(l).tapped).count(), 3, "{{3}} paid");
}

// ------------------------------------------------------------------ Eiganjo, Seat of the Empire
#[test]
fn eiganjo_channel_deals_four_to_an_attacker_and_costs_one_less_per_legendary_creature() {
    let mut g = pod();
    let e = hand(&mut g, P0, &["Eiganjo, Seat of the Empire"])[0];
    lands(&mut g, P0, "Plains", 3, false);
    let a = perm(&mut g, P1, "Craterhoof Behemoth"); // 5/5: 4 damage doesn't kill it
    let b = perm(&mut g, P1, "Llanowar Elves"); // too small to be worth it
    let big = perm(&mut g, P1, "Kaalia of the Vast"); // 2 toughness, pval 7
    let mut atk = vec![a, b, big];
    let mut assign = vec![];
    for &m in &atk {
        g.perm_mut(m).tapped = true;
    }
    sim_core::cardcode::defend_hooks(&mut g, P1, &mut atk, P0, &mut assign).unwrap();
    assert!(g.perm(a).on_bf && g.perm(b).on_bf, "too big / too small to be worth 4 damage");
    assert!(!g.perm(big).on_bf && !atk.contains(&big), "Kaalia is killed and leaves the attack");
    assert!(!g.player(P0).hand.contains(&e) && g.player(P0).gy.contains(&e), "discarded as the cost");
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.land(l).tapped).count(), 3, "{{2}}{{W}}");
}

#[test]
fn eiganjo_costs_one_less_for_each_legendary_creature() {
    let mut g = pod();
    assert_eq!(sim_core::impls::t6_lands::eiganjo_cost(&g, P0), 2);
    perm(&mut g, P0, "Isshin, Two Heavens as One");
    assert_eq!(sim_core::impls::t6_lands::eiganjo_cost(&g, P0), 1);
}

#[test]
fn eiganjo_channel_also_hits_a_blocker_of_its_own_attack() {
    let mut g = pod();
    let e = hand(&mut g, P0, &["Eiganjo, Seat of the Empire"])[0];
    lands(&mut g, P0, "Plains", 3, false);
    let atk = perm(&mut g, P0, "Craterhoof Behemoth");
    let blk = perm(&mut g, P1, "Kaalia of the Vast");
    let mut assign = vec![(atk, blk)];
    let mut a = vec![atk];
    sim_core::cardcode::defend_hooks(&mut g, P0, &mut a, P1, &mut assign).unwrap();
    assert!(!g.perm(blk).on_bf && assign.is_empty(), "the blocker takes 4 damage and is removed from the blocks");
    assert!(g.player(P0).gy.contains(&e));
}

// ------------------------------------------------------------------ Deserted Temple
#[test]
fn deserted_temple_untaps_a_land_that_makes_enough_mana_to_pay_for_it() {
    let mut g = pod();
    let t = lands(&mut g, P0, "Deserted Temple", 1, false)[0];
    lands(&mut g, P0, "Forest", 1, false);
    let c = lands(&mut g, P0, "City of Traitors", 1, true)[0]; // {C}{C}, tapped
    let o = sim_core::cardcode::land_options(&mut g, P0, None).unwrap();
    assert!(o.is_empty(), "untapping the City nets 2 - 1 - 1 = 0 mana: not worth it");
    lands(&mut g, P0, "Forest", 1, false);
    let _ = (t, c);
    // a land worth 3: City of Traitors is only 2, so use a tapped Gaea's Cradle with three creatures
    let cradle = lands(&mut g, P0, "Gaea's Cradle", 1, true)[0];
    for _ in 0..3 {
        token(&mut g, P0, 1);
    }
    let o = sim_core::cardcode::land_options(&mut g, P0, None).unwrap();
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].label, "Deserted Temple untaps Gaea's Cradle");
    assert!(sim_core::ai::act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert!(!g.land(cradle).tapped, "the Cradle is untapped");
    assert!(g.land(t).tapped, "{{T}}");
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.land(l).tapped).count(), 3, "Temple + the {{1}} + the City");
}

// ------------------------------------------------------------------ Minamo, School at Water's Edge
#[test]
fn minamo_untaps_a_legendary_land_for_u_and_its_own_tap() {
    let mut g = pod();
    let m = lands(&mut g, P0, "Minamo, School at Water's Edge", 1, false)[0];
    lands(&mut g, P0, "Island", 2, false);
    let cradle = lands(&mut g, P0, "Gaea's Cradle", 1, true)[0];
    for _ in 0..3 {
        token(&mut g, P0, 1);
    }
    let o = sim_core::cardcode::land_options(&mut g, P0, None).unwrap();
    assert_eq!(o.len(), 1);
    assert_eq!(o[0].label, "Minamo untaps Gaea's Cradle");
    assert!(sim_core::ai::act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert!(!g.land(cradle).tapped && g.land(m).tapped);
    assert_eq!(
        g.player(P0).lands.iter().filter(|&&l| g.land(l).tapped).count(),
        2,
        "Minamo and the Island that paid {{U}}"
    );
}

#[test]
fn minamo_does_nothing_when_the_untap_does_not_pay_for_itself() {
    let mut g = pod();
    lands(&mut g, P0, "Minamo, School at Water's Edge", 1, false);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Gaea's Cradle", 1, true); // no creatures: it makes nothing
    assert!(sim_core::cardcode::land_options(&mut g, P0, None).unwrap().is_empty());
}

// ------------------------------------------------------------------ Emergence Zone
#[test]
fn emergence_zone_sacrifices_itself_and_marks_the_turn_for_flash() {
    use sim_core::impls::t6_lands as t6;
    let mut g = pod();
    let z = lands(&mut g, P0, "Emergence Zone", 1, false)[0];
    lands(&mut g, P0, "Forest", 1, false);
    assert!(t6::emergence_go(&mut g, P0, z.0 as i64).unwrap());
    assert!(!g.player(P0).lands.contains(&z), "sacrificed");
    assert!(g.player(P0).gy.contains(&card(&g, "Emergence Zone")));
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.land(l).tapped).count(), 1, "the {{1}} came from the Forest");
    let flag = sim_core::sym::intern(t6::FLASH_FLAG);
    assert_eq!(g.player(P0).flag_turn.get(&flag), Some(&g.turn_stamp()), "flash this turn");
}

// ------------------------------------------------------------------ Forbidden Orchard
#[test]
fn forbidden_orchard_gives_an_opponent_a_spirit_each_time_it_is_tapped_for_mana() {
    let mut g = pod();
    lands(&mut g, P0, "Forbidden Orchard", 1, false);
    let before = g.player(P1).perms.len();
    assert!(mana::pay(&mut g, P0, 1, "", false).unwrap());
    let toks: Vec<_> = g.player(P1).perms.iter().copied().filter(|&m| g.perm(m).token).collect();
    assert_eq!(g.player(P1).perms.len(), before + 1);
    let s = toks[0];
    assert_eq!((g.perm(s).pow, g.perm(s).tgh), (1, 1));
    assert!(g.perm(s).colors.is_empty(), "colorless");
    assert!(g.perm(s).ttypes.contains(&sim_core::sym::intern("spirit")));
    assert!(g.player(P0).perms.iter().all(|&m| !g.perm(m).token), "the tapper gets nothing");
}

// ------------------------------------------------------------------ Petrified Hamlet
#[test]
fn petrified_hamlet_names_the_opponents_utility_land() {
    use sim_core::impls::t6_lands as t6;
    let mut g = pod();
    lands(&mut g, P1, "Forest", 2, false);
    lands(&mut g, P1, "Mutavault", 1, false);
    play_land(&mut g, P0, "Petrified Hamlet");
    assert_eq!(t6::hamlet_names(&g), [card(&g, "Mutavault")]);
}

#[test]
fn petrified_hamlet_names_nothing_without_a_land_with_abilities_and_still_sacrifices_a_city() {
    use sim_core::impls::t6_lands as t6;
    let mut g = pod();
    lands(&mut g, P1, "Forest", 2, false);
    lands(&mut g, P0, "City of Traitors", 1, false);
    play_land(&mut g, P0, "Petrified Hamlet");
    assert!(t6::hamlet_names(&g).is_empty());
    assert_eq!(names(&g, P0), ["Petrified Hamlet"], "playing the Hamlet sacrifices the City");
}

// ------------------------------------------------------------------ The Mycosynth Gardens
#[test]
fn mycosynth_gardens_is_colourless_alone_and_filters_with_another_land() {
    let mut g = pod();
    lands(&mut g, P0, "The Mycosynth Gardens", 1, false);
    let u = mana::mana_units(&g, P0, false);
    assert!(u.len() == 1 && u[0].cols.is_empty());
    lands(&mut g, P0, "Forest", 1, false);
    let u = mana::mana_units(&g, P0, false);
    assert_eq!(u.iter().map(|x| x.amt).sum::<u32>(), 2);
    assert!(u.iter().any(|x| x.cols == g.player(P0).ident), "{{1}},{{T}}: one mana of any colour");
    // two filters and no other land: still colourless (nothing real pays the {1})
    let mut g = pod();
    lands(&mut g, P0, "The Mycosynth Gardens", 1, false);
    lands(&mut g, P0, "Abstergo Entertainment", 1, false);
    assert!(mana::mana_units(&g, P0, false).iter().all(|x| x.cols.is_empty()));
}

// ------------------------------------------------------------------ Fomori Vault
#[test]
fn fomori_vault_discards_a_card_and_takes_the_best_of_the_top_x_cards() {
    let mut g = pod();
    lands(&mut g, P0, "Fomori Vault", 1, false);
    lands(&mut g, P0, "Forest", 3, false);
    for n in ["Sol Ring", "Mana Vault", "Grim Monolith"] {
        perm(&mut g, P0, n);
    }
    hand(&mut g, P0, &["Forest"]);
    let top: Vec<_> = ["Llanowar Elves", "Birds of Paradise", "Craterhoof Behemoth", "Eternal Witness"]
        .iter()
        .map(|n| take(&mut g, P0, n))
        .collect();
    g.player_mut(P0).library.extend(&top); // the last is the top
    let before = g.player(P0).library.len();
    let o = sim_core::cardcode::land_options(&mut g, P0, None).unwrap();
    assert_eq!(o.iter().map(|x| x.label.as_str()).collect::<Vec<_>>(), ["Fomori Vault"]);
    assert!(sim_core::ai::act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    let pl = g.player(P0);
    assert_eq!(pl.hand.len(), 1, "discarded the Forest, took one card");
    assert!(pl.gy.contains(&card(&g, "Forest")));
    let got = pl.hand[0];
    assert!(top[1..].contains(&got), "one of the top 3 (X = 3 artifacts)");
    assert_eq!(pl.library.len(), before - 1, "the other two went to the bottom");
    assert!(pl.library.last() == Some(&top[0]), "the 4th card is still on top");
    assert!(g.land(g.player(P0).lands[0]).tapped, "{{T}}: the Vault is tapped");
}

#[test]
fn fomori_vault_needs_artifacts() {
    let mut g = pod();
    lands(&mut g, P0, "Fomori Vault", 1, false);
    lands(&mut g, P0, "Forest", 3, false);
    perm(&mut g, P0, "Sol Ring");
    hand(&mut g, P0, &["Forest"]);
    assert!(sim_core::cardcode::land_options(&mut g, P0, None).unwrap().is_empty(), "X = 1: not worth a card");
}

// ------------------------------------------------------------------ Mount Doom
#[test]
fn mount_doom_costs_one_life_to_tap_and_is_not_tapped_at_one_life() {
    let mut g = pod();
    lands(&mut g, P0, "Mount Doom", 1, false);
    let life = g.player(P0).life;
    assert!(mana::pay(&mut g, P0, 0, "B", false).unwrap());
    assert_eq!(g.player(P0).life, life - 1, "pay 1 life");
    let mut g = pod();
    lands(&mut g, P0, "Mount Doom", 1, false);
    g.player_mut(P0).life = 1;
    assert_eq!(mana::total_mana(&g, P0, false), 0, "never tapped for mana at 1 life");
}

#[test]
fn mount_doom_pings_every_opponent_for_one() {
    let mut g = table(&["lathril-golgari-elves", "veyran", "sauron"]);
    lands(&mut g, P0, "Mount Doom", 1, false);
    lands(&mut g, P0, "Swamp", 1, false);
    lands(&mut g, P0, "Mountain", 1, false);
    lands(&mut g, P0, "Plains", 1, false);
    let (l1, l2) = (g.player(P1).life, g.player(PlayerId(2)).life);
    let o = sim_core::cardcode::land_options(&mut g, P0, None).unwrap();
    let ping = o.iter().find(|x| x.label == "Mount Doom pings").expect("ping offered at the end of the turn");
    assert!(sim_core::ai::act::perform(&mut g, P0, ping.act.as_ref().unwrap()).unwrap());
    assert_eq!((g.player(P1).life, g.player(PlayerId(2)).life), (l1 - 1, l2 - 1));
}

#[test]
fn mount_doom_wipes_all_but_two_creatures_for_a_legendary_artifact() {
    let mut g = pod();
    g.active = Some(P0);
    let md = lands(&mut g, P0, "Mount Doom", 1, false)[0];
    lands(&mut g, P0, "Swamp", 4, false);
    lands(&mut g, P0, "Mountain", 4, false);
    let art = perm(&mut g, P0, "Sword of the Animist"); // not legendary: no wipe
    assert!(
        sim_core::cardcode::land_options(&mut g, P0, Some(false))
            .unwrap()
            .iter()
            .all(|x| x.label != "Mount Doom destroys all but two creatures")
    );
    let _ = art;
    let mine = [
        perm(&mut g, P0, "Craterhoof Behemoth"),
        perm(&mut g, P0, "Kaalia of the Vast"),
        perm(&mut g, P0, "Llanowar Elves"),
    ];
    let theirs = [
        perm(&mut g, P1, "Archon of Cruelty"),
        perm(&mut g, P1, "Sheoldred, the Apocalypse"),
        perm(&mut g, P1, "Hullbreaker Horror"),
    ];
    let sol = perm(&mut g, P0, "The One Ring"); // a legendary artifact
    let o = sim_core::cardcode::land_options(&mut g, P0, Some(false)).unwrap();
    let w = o.iter().find(|x| x.label == "Mount Doom destroys all but two creatures").expect("wipe offered");
    assert!(sim_core::ai::act::perform(&mut g, P0, w.act.as_ref().unwrap()).unwrap());
    assert!(!g.player(P0).lands.contains(&md), "Mount Doom is sacrificed");
    assert!(!g.perm(sol).on_bf, "and a legendary artifact");
    assert!(g.perm(mine[0]).on_bf && g.perm(mine[1]).on_bf, "my two best creatures are chosen");
    assert!(!g.perm(mine[2]).on_bf, "the rest are destroyed");
    assert!(theirs.iter().all(|&m| !g.perm(m).on_bf));
}

// ------------------------------------------------------------------ Razorgrass Ambush // Razorgrass Field
const RAZOR: &str = "Razorgrass Ambush // Razorgrass Field";

#[test]
fn razorgrass_ambush_deals_three_to_an_attacker_worth_killing() {
    let mut g = pod();
    let c = hand(&mut g, P0, &[RAZOR])[0];
    lands(&mut g, P0, "Plains", 2, false);
    let big = perm(&mut g, P1, "Craterhoof Behemoth"); // 5 toughness: 3 damage does nothing
    let kaalia = perm(&mut g, P1, "Kaalia of the Vast");
    let mut atk = vec![big, kaalia];
    let mut assign = vec![];
    sim_core::cardcode::defend_hooks(&mut g, P1, &mut atk, P0, &mut assign).unwrap();
    assert!(g.perm(big).on_bf && !g.perm(kaalia).on_bf && !atk.contains(&kaalia));
    assert!(g.player(P0).gy.contains(&c) && !g.player(P0).hand.contains(&c), "an instant: to the graveyard");
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.land(l).tapped).count(), 2, "{{1}}{{W}}");
}

#[test]
fn razorgrass_ambush_is_held_when_nothing_is_worth_three_damage() {
    let mut g = pod();
    let c = hand(&mut g, P0, &[RAZOR])[0];
    lands(&mut g, P0, "Plains", 2, false);
    let elf = perm(&mut g, P1, "Llanowar Elves");
    let mut atk = vec![elf];
    sim_core::cardcode::defend_hooks(&mut g, P1, &mut atk, P0, &mut vec![]).unwrap();
    assert!(g.perm(elf).on_bf && g.player(P0).hand.contains(&c));
}

#[test]
fn razorgrass_field_is_played_as_a_land_that_taps_for_w_and_enters_tapped_unless_3_life_is_paid() {
    let mut g = pod();
    g.active = Some(P0);
    let c = hand(&mut g, P0, &[RAZOR])[0];
    lands(&mut g, P0, "Plains", 2, false);
    let o = hand_opts(&mut g, P0, Some(false));
    let o = o.iter().find(|x| x.label == format!("play {RAZOR} as a land")).expect("land option");
    let life = g.player(P0).life;
    assert!(sim_core::ai::act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(!g.player(P0).hand.contains(&c));
    let l = *g.player(P0).lands.last().unwrap();
    assert_eq!(g.land(l).cd, c);
    assert_eq!(g.player(P0).lands_played, 1, "it was the land drop");
    // few lands on the battlefield and a comfortable life: the AI pays 3 life only when that makes mana it can use
    assert_eq!(g.land(l).tapped, g.player(P0).life == life, "tapped exactly when no life was paid");
    assert!(g.player(P0).life == life || g.player(P0).life == life - 3);
    g.land_mut(l).tapped = false;
    let u = mana::mana_units(&g, P0, false);
    assert!(u.iter().any(|x| x.cols.has('W') && x.src == mana::Source::Land(l)), "{{T}}: Add {{W}}");
    // no second land drop
    let c2 = hand(&mut g, P0, &[RAZOR])[0];
    let o = hand_opts(&mut g, P0, Some(false));
    assert!(o.iter().all(|x| !x.label.contains("as a land")), "land drop used");
    let _ = c2;
}

/// the hand cards' options (brain.hook_options' loop over the hand)
fn hand_opts(g: &mut Game, p: PlayerId, post: Option<bool>) -> Vec<sim_core::hooks::Opt> {
    let mut o = vec![];
    for c in g.player(p).hand.clone() {
        if let Some(f) = g.registry.get(c).and_then(|i| i.hand_options) {
            o.extend(f(g, c, p, post).unwrap());
        }
    }
    o
}

// ------------------------------------------------------------------ Shatterskull Smashing // Shatterskull, the Hammer Pass
const SMASH: &str = "Shatterskull Smashing";

#[test]
fn shatterskull_smashing_divides_x_damage_between_two_creatures() {
    let mut g = pod();
    g.active = Some(P0);
    let c = hand(&mut g, P0, &[SMASH])[0];
    lands(&mut g, P0, "Mountain", 9, false);
    let kaalia = perm(&mut g, P1, "Kaalia of the Vast"); // toughness 2
    let sheo = perm(&mut g, P1, "Sheoldred, the Apocalypse"); // toughness 5
    let o = hand_opts(&mut g, P0, Some(false));
    let o = o.iter().find(|x| x.label.starts_with("Shatterskull Smashing")).expect("offered with a kill");
    assert_eq!(
        o.label, "Shatterskull Smashing (X = 6)",
        "X = 6 deals twice X = 12: the smallest X that kills both (2 + 5)"
    );
    assert!(sim_core::ai::act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(!g.perm(kaalia).on_bf && !g.perm(sheo).on_bf, "the damage is divided as lethal to each");
    assert!(g.player(P0).gy.contains(&c));
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.land(l).tapped).count(), 8, "{{6}}{{R}}{{R}}");
}

#[test]
fn shatterskull_smashing_x_of_six_or_more_deals_twice_x() {
    use sim_core::impls::t6_lands as t6;
    assert_eq!((t6::smash_damage(5), t6::smash_damage(6), t6::smash_damage(8)), (5, 12, 16));
}

#[test]
fn shatterskull_smashing_is_held_without_a_worthwhile_kill() {
    let mut g = pod();
    g.active = Some(P0);
    hand(&mut g, P0, &[SMASH]);
    lands(&mut g, P0, "Mountain", 6, false);
    perm(&mut g, P1, "Llanowar Elves");
    assert!(hand_opts(&mut g, P0, Some(false)).iter().all(|x| !x.label.starts_with("Shatterskull Smashing")));
}

#[test]
fn shatterskull_the_hammer_pass_is_played_as_a_land_that_taps_for_r() {
    let mut g = pod();
    g.active = Some(P0);
    let c = hand(&mut g, P0, &[SMASH])[0];
    lands(&mut g, P0, "Mountain", 2, false);
    let o = hand_opts(&mut g, P0, Some(false));
    let o = o.iter().find(|x| x.label.contains("as a land")).expect("land option");
    assert!(sim_core::ai::act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    let l = *g.player(P0).lands.last().unwrap();
    assert_eq!(g.land(l).cd, c);
    g.land_mut(l).tapped = false;
    assert!(mana::mana_units(&g, P0, false).iter().any(|x| x.src == mana::Source::Land(l) && x.cols.has('R')));
}

// ------------------------------------------------------------------ Talon Gates of Madara
#[test]
fn talon_gates_phases_out_the_best_opposing_creature_when_it_enters() {
    let mut g = pod();
    let k = perm(&mut g, P1, "Kaalia of the Vast");
    let e = perm(&mut g, P1, "Llanowar Elves");
    lands(&mut g, P0, "City of Traitors", 1, false);
    play_land(&mut g, P0, "Talon Gates of Madara");
    assert!(g.perm(k).phased && !g.perm(e).phased, "up to one target creature phases out");
    assert_eq!(names(&g, P0), ["Talon Gates of Madara"], "the City of Traitors check still runs");
}

#[test]
fn talon_gates_put_onto_the_battlefield_for_four_costs_no_land_drop() {
    let mut g = pod();
    g.active = Some(P0);
    let c = hand(&mut g, P0, &["Talon Gates of Madara"])[0];
    lands(&mut g, P0, "Forest", 4, false);
    let k = perm(&mut g, P1, "Kaalia of the Vast");
    assert!(hand_opts(&mut g, P0, Some(false)).is_empty(), "the land drop is still open: play it normally");
    g.player_mut(P0).lands_played = 1;
    let o = hand_opts(&mut g, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert!(sim_core::ai::act::perform(&mut g, P0, o[0].act.as_ref().unwrap()).unwrap());
    assert!(!g.player(P0).hand.contains(&c) && g.player(P0).lands.len() == 5);
    assert!(g.player(P0).lands.iter().take(4).all(|&l| g.land(l).tapped), "{{4}} paid");
    assert!(g.perm(k).phased);
    assert_eq!(g.player(P0).lands_played, 1, "not a land play");
}

#[test]
fn talon_gates_phases_a_big_attacker_out_of_the_attack_for_four() {
    let mut g = pod();
    let c = hand(&mut g, P0, &["Talon Gates of Madara"])[0];
    lands(&mut g, P0, "Forest", 4, false);
    let k = perm(&mut g, P1, "Kaalia of the Vast");
    let e = perm(&mut g, P1, "Llanowar Elves");
    let mut atk = vec![k, e];
    sim_core::cardcode::defend_hooks(&mut g, P1, &mut atk, P0, &mut vec![]).unwrap();
    assert!(g.perm(k).phased && atk == [e]);
    assert!(!g.player(P0).hand.contains(&c) && g.player(P0).lands.len() == 5);
}
