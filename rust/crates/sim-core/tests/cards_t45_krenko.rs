//! The Tier 4 krenko-mono-red-goblins cards (impls/t45_krenko.rs and the compiled patterns under
//! `// --- Tiers 4/5 (krenko)`) on hand-built positions. Each expectation follows the card's Oracle text.

use sim_core::cardcode;
use sim_core::dsl::{Fired, fire, has_kw};
use sim_core::engine::{cast, combat, life, removal, values::epow, zones};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::{combos, t4, t45_krenko};
use sim_core::state::{Ctx, DataKey, Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const KRENKO: &str = "krenko-mono-red-goblins";

fn krenko() -> Game {
    table(&[KRENKO, "seph", "sauron"])
}

fn opp_life(g: &Game) -> i32 {
    g.player(P1).life + g.player(P2).life
}

fn goblins(g: &Game, p: PlayerId) -> usize {
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| g.perm(m).on_bf && g.is_creature(m) && sim_core::engine::values::has_type(g, m, "goblin"))
        .count()
}

fn resolve(g: &mut Game, p: PlayerId, name: &str) {
    let c = take(g, p, name);
    cast::resolve(g, p, c, &Ctx::default(), "hand").unwrap();
}

fn opts(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.perm(src).cd.unwrap()).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn run(g: &mut Game, o: &Opt, p: PlayerId) -> bool {
    match o.act.clone().unwrap() {
        Action::Ability { src, f, arg } => f(g, src, p, arg).unwrap(),
        Action::Plan { f, arg } => f(g, p, arg).unwrap(),
        other => panic!("unexpected action {other:?}"),
    }
}

// ------------------------------------------------------------------ compiled: burn and the ritual
#[test]
fn war_strike_and_massive_raid_count_goblins_and_creatures() {
    let mut g = krenko();
    t4::goblins(&mut g, P0, 4).unwrap();
    let before = opp_life(&g);
    resolve(&mut g, P0, "Goblin War Strike");
    assert_eq!(before - opp_life(&g), 4, "4 damage: the Goblins you control");
    perm(&mut g, P0, "Arcane Signet"); // not a creature: no extra damage
    token(&mut g, P0, 2); // a non-Goblin creature counts for Massive Raid
    let before = opp_life(&g);
    resolve(&mut g, P0, "Massive Raid");
    assert_eq!(before - opp_life(&g), 5, "5 damage: the creatures you control");
}

#[test]
fn brightstone_ritual_adds_mana_per_goblin_on_the_battlefield() {
    let mut g = krenko();
    t4::goblins(&mut g, P0, 2).unwrap();
    t4::goblins(&mut g, P1, 1).unwrap();
    let before = g.player(P0).floating;
    resolve(&mut g, P0, "Brightstone Ritual");
    let f = g.player(P0).floating;
    assert_eq!(f.any + f.r - before.any - before.r, 3, "every player's Goblins");
}

#[test]
fn hobgoblin_bandit_lord_pings_for_the_goblins_that_entered_this_turn() {
    let mut g = krenko();
    lands(&mut g, P0, "Mountain", 1, false);
    let hb = perm(&mut g, P0, "Hobgoblin Bandit Lord");
    let toks = t4::goblins(&mut g, P0, 3).unwrap();
    zones::die(&mut g, toks[0], "sac").unwrap(); // sacrificed: still entered this turn
    assert_eq!(t45_krenko::entered_this_turn(&g, P0, "goblin"), 4, "the Lord and three tokens");
    assert_eq!(epow(&g, toks[1]), 2, "other Goblins you control get +1/+1");
    let idx =
        g.db.get(g.perm(hb).cd.unwrap())
            .abilities
            .iter()
            .position(|a| matches!(a, sim_core::dsl::model::Ability::Activated { .. }))
            .unwrap();
    let before = opp_life(&g);
    assert!(sim_core::dsl::activate(&mut g, P0, hb, idx as u16).unwrap());
    assert_eq!(before - opp_life(&g), 4);
    // the next turn counts again from zero
    g.round += 1;
    assert_eq!(t45_krenko::entered_this_turn(&g, P0, "goblin"), 0);
}

// ------------------------------------------------------------------ General Kreat, Goblin King
#[test]
fn general_kreat_makes_an_attacking_goblin_and_pings_on_entries() {
    let mut g = krenko();
    let k = perm(&mut g, P0, "General Kreat, the Boltbringer");
    let other = token(&mut g, P0, 2); // a non-Goblin token entering still pings
    assert_eq!(opp_life(&g), 80 - 2, "1 damage to each opponent");
    let n = g.player(P0).perms.len();
    fire(
        &mut g,
        "attack",
        Fired { player: Some(P0), attackers: vec![other], defender: Some(P1), ..Default::default() },
    )
    .unwrap();
    assert_eq!(g.player(P0).perms.len(), n, "no Goblin attacked");
    fire(&mut g, "attack", Fired { player: Some(P0), attackers: vec![k], defender: Some(P1), ..Default::default() })
        .unwrap();
    let new = *g.player(P0).perms.last().unwrap();
    assert!(g.perm(new).token && g.perm(new).tapped, "a tapped (and attacking) Goblin token");
}

#[test]
fn goblin_king_pumps_every_goblin_and_gives_mountainwalk() {
    let mut g = krenko();
    let gob = t4::goblins(&mut g, P1, 1).unwrap()[0];
    let mine = t4::goblins(&mut g, P0, 1).unwrap()[0];
    let king = perm(&mut g, P0, "Goblin King");
    assert_eq!(epow(&g, gob), 2, "other Goblins: an opponent's too");
    assert_eq!(epow(&g, king), 2, "not itself");
    let blocker = token(&mut g, P1, 3);
    assert!(!cardcode::evasion_blocked(&g, blocker, mine), "no Mountain: blockable");
    lands(&mut g, P1, "Mountain", 1, false);
    assert!(cardcode::evasion_blocked(&g, blocker, mine), "mountainwalk");
    assert!(!cardcode::evasion_blocked(&g, blocker, king), "the King itself has no mountainwalk");
}

// ------------------------------------------------------------------ Boggart Shenanigans, Mana Echoes, Tremors
#[test]
fn boggart_shenanigans_pings_when_another_goblin_dies() {
    let mut g = krenko();
    perm(&mut g, P0, "Boggart Shenanigans");
    let gob = t4::goblins(&mut g, P0, 1).unwrap()[0];
    let other = token(&mut g, P0, 1);
    zones::die(&mut g, other, "sac").unwrap();
    assert_eq!(opp_life(&g), 80, "not a Goblin");
    zones::die(&mut g, gob, "sac").unwrap();
    assert_eq!(opp_life(&g), 79, "1 damage to a player");
    let opp = t4::goblins(&mut g, P1, 1).unwrap()[0];
    zones::die(&mut g, opp, "sac").unwrap();
    assert_eq!(opp_life(&g), 79, "not an opponent's Goblin");
}

#[test]
fn mana_echoes_adds_colourless_per_creature_sharing_a_type() {
    let mut g = krenko();
    perm(&mut g, P0, "Mana Echoes");
    perm(&mut g, P0, "Goblin Lackey");
    let c0 = g.player(P0).floating.c;
    t4::goblins(&mut g, P0, 3).unwrap();
    assert_eq!(g.player(P0).floating.c - c0, 3 * 4, "each of three Goblins: four Goblins share a type");
    let c1 = g.player(P0).floating.c;
    token(&mut g, P1, 2); // an opponent's typeless token: shares nothing
    assert_eq!(g.player(P0).floating.c, c1);
}

#[test]
fn impact_tremors_sees_krenkos_tokens() {
    let mut g = krenko();
    perm(&mut g, P0, "Impact Tremors");
    t4::goblins(&mut g, P0, 3).unwrap();
    assert_eq!(g.player(P1).life, 37);
    assert_eq!(g.player(P2).life, 37);
}

// ------------------------------------------------------------------ Goblin Chirurgeon, Hexing Squelcher
#[test]
fn goblin_chirurgeon_sacrifices_a_goblin_to_regenerate_krenko() {
    let mut g = krenko();
    let k = perm(&mut g, P0, "Krenko, Mob Boss");
    perm(&mut g, P0, "Goblin Chirurgeon");
    let tok = t4::goblins(&mut g, P0, 1).unwrap()[0];
    zones::die(&mut g, k, "destroy").unwrap();
    assert!(g.perm(k).on_bf && g.perm(k).tapped, "regenerated: tapped");
    assert!(!g.perm(tok).on_bf, "the token was sacrificed");
}

#[test]
fn hexing_squelcher_ward_costs_two_life() {
    let mut g = krenko();
    perm(&mut g, P0, "Hexing Squelcher");
    let k = perm(&mut g, P0, "Krenko, Mob Boss");
    let bolt = card(&g, "Lightning Bolt");
    removal::apply_removal(&mut g, Some(P1), k, "destroy", Some(bolt)).unwrap();
    assert_eq!(g.player(P1).life, 38, "ward: pay 2 life");
    assert!(!g.perm(k).on_bf);
    let k2 = perm(&mut g, P0, "Goblin King");
    g.player_mut(P1).life = 4;
    removal::apply_removal(&mut g, Some(P1), k2, "destroy", Some(bolt)).unwrap();
    assert!(g.perm(k2).on_bf, "can't afford the life: countered");
}

// ------------------------------------------------------------------ Howlsquad Heavy
#[test]
fn howlsquad_heavy_speed_haste_combat_token_and_max_speed_mana() {
    let mut g = krenko();
    let h = perm(&mut g, P0, "Howlsquad Heavy");
    assert_eq!(g.player(P0).speed, 1, "start your engines");
    let gob = t4::goblins(&mut g, P0, 1).unwrap()[0];
    g.perm_mut(gob).sick = true;
    assert!(combat::has_haste(&g, gob), "other Goblins you control have haste");
    life::lose_life(&mut g, P1, 1, Some(P0), "triggers", None).unwrap();
    life::lose_life(&mut g, P2, 1, Some(P0), "triggers", None).unwrap();
    assert_eq!(g.player(P0).speed, 2, "once a turn");
    g.active = Some(P1);
    life::lose_life(&mut g, P2, 1, Some(P0), "triggers", None).unwrap();
    assert_eq!(g.player(P0).speed, 2, "only on your turn");
    assert_eq!(cardcode::dyn_mana_perm(&g, P0, h), Some(0), "below max speed: no mana");
    g.player_mut(P0).speed = 4;
    assert_eq!(cardcode::dyn_mana_perm(&g, P0, h), Some(2), "{{R}} per Goblin");
    // the beginning of combat on its controller's turn
    g.active = Some(P0);
    g.step = Step::Main1;
    let n = goblins(&g, P0);
    let f = g.registry.get(card(&g, "Howlsquad Heavy")).unwrap().each_combat.unwrap();
    f(&mut g, h, P0).unwrap();
    assert_eq!(goblins(&g, P0), n + 1);
    let t = *g.player(P0).perms.last().unwrap();
    assert!(g.perm(t).data.get(DataKey::MustAttack).is_some(), "it attacks this combat if able");
    f(&mut g, h, P1).unwrap();
    assert_eq!(goblins(&g, P0), n + 1, "not on an opponent's combat");
}

// ------------------------------------------------------------------ untappers and Marvin
#[test]
fn patriars_seal_untaps_a_tapped_krenko() {
    let mut g = krenko();
    lands(&mut g, P0, "Mountain", 2, false);
    let k = perm(&mut g, P0, "Krenko, Mob Boss");
    let seal = perm(&mut g, P0, "Patriar's Seal");
    assert!(opts(&mut g, seal, P0, None).is_empty(), "Krenko is untapped");
    g.perm_mut(k).tapped = true;
    let o = opts(&mut g, seal, P0, None);
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, &o[0], P0));
    assert!(!g.perm(k).tapped && g.perm(seal).tapped);
}

#[test]
fn thousand_year_elixir_gives_tap_abilities_haste_and_untaps() {
    let mut g = krenko();
    lands(&mut g, P0, "Mountain", 2, false);
    let k = perm(&mut g, P0, "Krenko, Mob Boss");
    g.perm_mut(k).sick = true;
    assert!(opts(&mut g, k, P0, None).is_empty(), "summoning sick");
    let e = perm(&mut g, P0, "Thousand-Year Elixir");
    assert!(!opts(&mut g, k, P0, None).is_empty(), "as though it had haste");
    g.perm_mut(k).tapped = true;
    let o = opts(&mut g, e, P0, None);
    assert!(run(&mut g, &o[0], P0));
    assert!(!g.perm(k).tapped);
}

#[test]
fn marvin_has_krenkos_ability() {
    let mut g = krenko();
    perm(&mut g, P0, "Krenko, Mob Boss");
    let m = perm(&mut g, P0, "Marvin, Murderous Mimic");
    let o = opts(&mut g, m, P0, None);
    let o = o.iter().find(|o| o.label.starts_with("Marvin (Krenko)")).expect("Krenko's ability");
    let n = goblins(&g, P0);
    assert!(run(&mut g, o, P0));
    assert_eq!(goblins(&g, P0), 2 * n, "a Goblin per Goblin");
    assert!(g.perm(m).tapped);
}

// ------------------------------------------------------------------ Sting, Treasure Nabber, Warren Instigator
#[test]
fn sting_gives_first_strike_against_goblins() {
    let mut g = krenko();
    let a = token(&mut g, P0, 2);
    let s = perm(&mut g, P0, "Sting, the Glinting Dagger");
    g.perm_mut(s).attached = Some(a);
    let gob = t4::goblins(&mut g, P1, 1).unwrap()[0];
    let other = token(&mut g, P1, 1);
    assert!(t45_krenko::sting_first_strike(&g, a, gob));
    assert!(!t45_krenko::sting_first_strike(&g, a, other));
    assert!(!t45_krenko::sting_first_strike(&g, gob, a), "not the Goblin's");
    assert!(!has_kw(&g, a, "first strike"));
}

#[test]
fn treasure_nabber_takes_an_artifact_tapped_for_mana() {
    let mut g = krenko();
    let nab = perm(&mut g, P0, "Treasure Nabber");
    let ring = perm(&mut g, P1, "Sol Ring");
    let f = g.registry.get(card(&g, "Treasure Nabber")).unwrap().mana_tapped.unwrap();
    g.perm_mut(ring).tapped = true;
    f(&mut g, nab, P1, ring, 2).unwrap();
    assert_eq!(g.perm(ring).owner, P0, "until the end of your next turn");
    assert!(g.perm(ring).tapped);
    let mine = perm(&mut g, P0, "Arcane Signet");
    f(&mut g, nab, P0, mine, 1).unwrap();
    assert_eq!(g.perm(mine).owner, P0, "your own artifacts don't trigger it");
}

#[test]
fn warren_instigator_puts_a_goblin_from_hand_onto_the_battlefield() {
    let mut g = krenko();
    let wi = perm(&mut g, P0, "Warren Instigator");
    assert!(has_kw(&g, wi, "double strike"));
    hand(&mut g, P0, &["Goblin Matron", "Skirk Prospector", "Lightning Bolt"]);
    let f = g.registry.get(card(&g, "Warren Instigator")).unwrap().combat_damage.unwrap();
    f(&mut g, wi, P0, wi, P1, 2).unwrap();
    let names: Vec<&str> = g.player(P0).perms.iter().map(|&m| &*g.perm(m).name).collect();
    assert!(names.contains(&"Goblin Matron"), "the most expensive Goblin creature card");
    let matron = card(&g, "Goblin Matron");
    assert!(!g.player(P0).hand.contains(&matron), "it left the hand");
}

// ------------------------------------------------------------------ combo
#[test]
fn krenko_umbral_mantle_ashnods_altar_combo_is_ready_with_a_payoff() {
    let mut g = krenko();
    let i = combos::combo_index("Krenko + Umbral Mantle").expect("in the catalogue");
    perm(&mut g, P0, "Krenko, Mob Boss");
    perm(&mut g, P0, "Umbral Mantle");
    perm(&mut g, P0, "Ashnod's Altar");
    t4::goblins(&mut g, P0, 2).unwrap();
    assert!(!combos::ready(&g, P0, i), "no payoff and no haste");
    perm(&mut g, P0, "Impact Tremors");
    assert!(combos::ready(&g, P0, i));
    let mut g = krenko();
    perm(&mut g, P0, "Krenko, Mob Boss");
    perm(&mut g, P0, "Staff of Domination");
    perm(&mut g, P0, "Mana Echoes");
    perm(&mut g, P0, "Goblin Warchief");
    t4::goblins(&mut g, P0, 1).unwrap();
    assert!(combos::ready(&g, P0, i), "hasty Goblins before combat");
    g.step = Step::Main2;
    assert!(!combos::ready(&g, P0, i), "after combat the haste doesn't win");
}
