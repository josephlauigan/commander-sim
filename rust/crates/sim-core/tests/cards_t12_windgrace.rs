//! The Tier 2 lord-windgrace-jund-lands cards (impls/t12_windgrace.rs and the compiled patterns under
//! `// --- Tiers 1/2 (windgrace)`) and the Dark Depths + Thespian's Stage combo, on hand-built positions. Each
//! expectation follows the card's Oracle text.

use sim_core::cardcode;
use sim_core::dsl::{ability_options, has_kw};
use sim_core::engine::hooks::run_hook;
use sim_core::engine::mana::{mana_units, total_mana};
use sim_core::engine::values::{epow, etgh, indestructible};
use sim_core::engine::{cast, turn, zones};
use sim_core::hooks::{Action, Call, Event, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::combos;
use sim_core::state::{Ctx, DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const WINDGRACE: &str = "lord-windgrace-jund-lands";

fn windgrace() -> Game {
    table(&[WINDGRACE, "seph", "sauron"])
}

fn named(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf && &*g.perm(m).name == name).collect()
}

fn land_named(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).lands.iter().copied().find(|&l| &*g.db.get(g.perm(l).card()).name == name)
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn in_hand(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).hand.iter().any(|&c| &*g.db.get(c).name == name)
}

fn opts(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.perm(src).cd.unwrap()).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn land_opts(g: &mut Game, l: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.perm(l).card()).unwrap().land_options.unwrap();
    f(g, l, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    match o.act.clone().unwrap() {
        Action::Ability { src, f, arg } => f(g, src, p, arg).unwrap(),
        Action::Plan { f, arg } => f(g, p, arg).unwrap(),
        other => panic!("unexpected action {other:?}"),
    }
}

/// the option whose label contains this
fn pick(o: &[Opt], part: &str) -> Opt {
    o.iter()
        .find(|x| x.label.contains(part))
        .unwrap_or_else(|| panic!("{:?}", o.iter().map(|x| &x.label).collect::<Vec<_>>()))
        .clone()
}

/// a card put on top of p's library
fn on_top(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = take(g, p, name);
    g.player_mut(p).library.push(c);
    c
}

fn resolve(g: &mut Game, p: PlayerId, name: &str, ctx: Ctx) {
    let c = take(g, p, name);
    cast::resolve(g, p, c, &ctx, "hand").unwrap();
}

/// take every land card out of p's hand (or library)
fn no_lands(g: &mut Game, p: PlayerId, library: bool) {
    let db = g.db.clone();
    let pl = g.player_mut(p);
    let v = if library { &mut pl.library } else { &mut pl.hand };
    v.retain(|&c| !db.get(c).land);
}

fn lage(g: &Game, p: PlayerId) -> Option<PermId> {
    named(g, p, "Marit Lage").first().copied()
}

// ------------------------------------------------------------------ bounce lands, Lotus Field
#[test]
fn gruul_turf_and_rakdos_carnarium_return_a_land_and_tap_for_two() {
    for (name, a, b) in [("Gruul Turf", 'R', 'G'), ("Rakdos Carnarium", 'B', 'R')] {
        let mut g = windgrace();
        let forest = lands(&mut g, P0, "Forest", 1, false)[0];
        let c = take(&mut g, P0, name);
        turn::play_land_card(&mut g, P0, c, "plays").unwrap();
        assert!(!g.player(P0).lands.contains(&forest) && in_hand(&g, P0, "Forest"), "{name} returns a land");
        let l = land_named(&g, P0, name).unwrap();
        assert!(g.perm(l).tapped, "{name} enters tapped");
        g.perm_mut(l).tapped = false;
        let u = mana_units(&g, P0, false);
        assert!(u.iter().any(|w| w.amt == 2 && w.cols.has(a) && w.cols.has(b)), "{name}: {{T}}: two mana");
    }
}

#[test]
fn lotus_field_sacrifices_two_lands_and_taps_for_three_of_one_colour() {
    let mut g = windgrace();
    lands(&mut g, P0, "Forest", 2, true);
    lands(&mut g, P0, "Swamp", 1, false);
    let c = take(&mut g, P0, "Lotus Field");
    turn::play_land_card(&mut g, P0, c, "plays").unwrap();
    assert_eq!(g.player(P0).lands.len(), 2, "two of the three others are sacrificed");
    assert_eq!(g.player(P0).gy.iter().filter(|&&x| &*g.db.get(x).name == "Forest").count(), 2, "the tapped ones");
    let l = land_named(&g, P0, "Lotus Field").unwrap();
    assert!(g.perm(l).tapped);
    g.perm_mut(l).tapped = false;
    assert!(mana_units(&g, P0, false).iter().any(|w| w.amt == 3 && w.cols.has('G')), "three mana of one colour");
    // with no other land it sacrifices itself
    let mut g = windgrace();
    let c = take(&mut g, P0, "Lotus Field");
    turn::play_land_card(&mut g, P0, c, "plays").unwrap();
    assert!(g.player(P0).lands.is_empty() && in_gy(&g, P0, "Lotus Field"));
}

// ------------------------------------------------------------------ Dark Depths, Thespian's Stage
#[test]
fn dark_depths_makes_no_mana_and_its_last_ice_counter_makes_marit_lage() {
    let mut g = windgrace();
    let d = lands(&mut g, P0, "Dark Depths", 1, false)[0];
    assert_eq!(total_mana(&g, P0, false), 0, "no mana ability");
    lands(&mut g, P0, "Forest", 3, false);
    g.perm_mut(d).data.set(DataKey::IceRemoved, Val::Int(9));
    g.active = Some(P1); // an opponent's end of turn
    let o = land_opts(&mut g, d, P0, None);
    assert!(run(&mut g, P0, &pick(&o, "remove an ice counter")));
    let m = lage(&g, P0).expect("Marit Lage");
    assert_eq!((epow(&g, m), etgh(&g, m)), (20, 20));
    assert!(g.perm(m).fly && indestructible(&g, m), "flying and indestructible");
    assert!(!g.player(P0).lands.contains(&d) && in_gy(&g, P0, "Dark Depths"), "sacrificed");
}

#[test]
fn dark_depths_counts_down_one_counter_per_activation() {
    let mut g = windgrace();
    let d = lands(&mut g, P0, "Dark Depths", 1, false)[0];
    lands(&mut g, P0, "Forest", 6, false);
    g.active = Some(P1);
    for _ in 0..2 {
        let o = land_opts(&mut g, d, P0, None);
        assert!(run(&mut g, P0, &pick(&o, "remove an ice counter")));
    }
    assert_eq!(sim_core::impls::t12_windgrace::ice(&g, d), 8);
    assert!(lage(&g, P0).is_none() && land_opts(&mut g, d, P0, None).is_empty(), "no mana left for a third");
}

#[test]
fn dark_depths_and_thespians_stage_combo_makes_marit_lage() {
    let mut g = windgrace();
    lands(&mut g, P0, "Dark Depths", 1, false);
    lands(&mut g, P0, "Thespian's Stage", 1, false);
    lands(&mut g, P0, "Forest", 1, false);
    let i = combos::combo_index("Dark Depths + Thespian's Stage").unwrap();
    assert!(!combos::ready(&g, P0, i), "{{2}} besides the Stage is needed");
    lands(&mut g, P0, "Forest", 1, false);
    assert!(combos::ready(&g, P0, i));
    combos::attempt(&mut g, P0, i as i64).unwrap();
    assert!(lage(&g, P0).is_some(), "Marit Lage");
    assert!(in_gy(&g, P0, "Dark Depths") && in_gy(&g, P0, "Thespian's Stage"), "the legend rule and the sacrifice");
    assert!(g.player(P0).lands.iter().all(|&l| g.perm(l).tapped), "the Stage and {{2}} paid");
}

#[test]
fn crop_rotation_finds_the_missing_half_and_keeps_dark_depths() {
    let mut g = windgrace();
    lands(&mut g, P0, "Dark Depths", 1, false);
    lands(&mut g, P0, "Forest", 1, true);
    on_top(&mut g, P0, "Thespian's Stage");
    resolve(&mut g, P0, "Crop Rotation", Ctx::default());
    assert!(land_named(&g, P0, "Thespian's Stage").is_some() && land_named(&g, P0, "Dark Depths").is_some());
    assert!(in_gy(&g, P0, "Forest"));
}

// ------------------------------------------------------------------ Elvish Reclaimer
#[test]
fn elvish_reclaimer_grows_with_three_lands_in_the_graveyard() {
    let mut g = windgrace();
    let r = perm(&mut g, P0, "Elvish Reclaimer");
    assert_eq!((epow(&g, r), etgh(&g, r)), (1, 2));
    for _ in 0..3 {
        let c = take(&mut g, P0, "Forest");
        g.player_mut(P0).gy.push(c);
    }
    assert_eq!((epow(&g, r), etgh(&g, r)), (3, 4));
}

#[test]
fn elvish_reclaimer_sacrifices_a_tapped_land_for_the_missing_combo_half() {
    let mut g = windgrace();
    let r = perm(&mut g, P0, "Elvish Reclaimer");
    lands(&mut g, P0, "Dark Depths", 1, false);
    lands(&mut g, P0, "Forest", 2, false);
    lands(&mut g, P0, "Swamp", 1, true);
    on_top(&mut g, P0, "Thespian's Stage");
    let o = opts(&mut g, r, P0, Some(false));
    assert!(run(&mut g, P0, &pick(&o, "Thespian's Stage")), "at once: it completes the combo");
    assert!(g.perm(r).tapped, "{{T}}");
    assert!(g.player(P0).gy.iter().any(|&c| g.db.get(c).basic), "a tapped basic sacrificed (paid {{2}} first)");
    assert_eq!(g.player(P0).lands.len(), 4);
    let s = land_named(&g, P0, "Thespian's Stage").expect("the Stage");
    assert!(g.perm(s).tapped, "onto the battlefield tapped");
    assert!(land_named(&g, P0, "Dark Depths").is_some(), "never a key land");
}

// ------------------------------------------------------------------ extra land drops
#[test]
fn icetill_explorer_and_mina_and_denn_give_land_drops_and_icetill_the_graveyard() {
    let mut g = windgrace();
    perm(&mut g, P0, "Icetill Explorer");
    assert_eq!(cardcode::extra_lands(&g, P0), 1);
    assert!(cardcode::lands_from_gy(&g, P0), "you may play lands from your graveyard");
    assert!(!cardcode::lands_from_gy(&g, P1));
    perm(&mut g, P0, "Mina and Denn, Wildborn");
    assert_eq!(cardcode::extra_lands(&g, P0), 2);
}

#[test]
fn mina_and_denn_return_a_land_and_replay_it_for_landfall() {
    let mut g = windgrace();
    let m = perm(&mut g, P0, "Mina and Denn, Wildborn");
    perm(&mut g, P0, "Tannuk, Memorial Ensign");
    lands(&mut g, P0, "Mountain", 2, false);
    lands(&mut g, P0, "Forest", 2, false);
    lands(&mut g, P0, "Swamp", 1, true);
    g.player_mut(P0).lands_played = 1;
    no_lands(&mut g, P0, false);
    let life = g.player(P1).life;
    let o = opts(&mut g, m, P0, Some(false));
    assert!(run(&mut g, P0, &pick(&o, "Mina and Denn")));
    assert_eq!(g.player(P0).lands.len(), 5, "a tapped land returned and played again");
    assert!(g.player(P0).lands.iter().any(|&l| !g.perm(l).tapped), "untapped");
    assert_eq!(g.player(P0).lands_played, 2, "the second land drop");
    assert_eq!(g.player(P1).life, life - 1, "Tannuk's landfall");
    assert!(g.player(P0).perms.iter().any(|&x| has_kw(&g, x, "trample")), "target creature gains trample");
}

// ------------------------------------------------------------------ Lumra
#[test]
fn lumra_is_as_big_as_your_lands_and_returns_every_land_card() {
    let mut g = windgrace();
    lands(&mut g, P0, "Forest", 3, false);
    for n in ["Swamp", "Mountain"] {
        let c = take(&mut g, P0, n);
        g.player_mut(P0).gy.push(c);
    }
    for n in ["Sol Ring", "Eternal Witness", "Explore", "Cultivate"] {
        on_top(&mut g, P0, n);
    }
    let l = perm(&mut g, P0, "Lumra, Bellow of the Woods");
    assert_eq!(g.player(P0).lands.len(), 5, "both land cards back");
    assert!(g.player(P0).lands.iter().filter(|&&x| g.perm(x).tapped).count() == 2, "tapped");
    assert!(in_gy(&g, P0, "Sol Ring") && in_gy(&g, P0, "Cultivate"), "milled four");
    assert_eq!((epow(&g, l), etgh(&g, l)), (5, 5), "power and toughness: the lands you control");
}

// ------------------------------------------------------------------ Six
#[test]
fn six_attacks_mills_three_and_takes_a_land() {
    let mut g = windgrace();
    let six = perm(&mut g, P0, "Six");
    on_top(&mut g, P0, "Sol Ring");
    on_top(&mut g, P0, "Forest");
    on_top(&mut g, P0, "Eternal Witness");
    let hand = g.player(P0).hand.len();
    let f = g.registry.get(card(&g, "Six")).unwrap().attack.unwrap();
    f(&mut g, six, P0, &[six], P1).unwrap();
    assert_eq!(g.player(P0).hand.len(), hand + 1);
    assert!(in_hand(&g, P0, "Forest") && in_gy(&g, P0, "Sol Ring") && in_gy(&g, P0, "Eternal Witness"));
}

#[test]
fn six_gives_retrace_to_permanent_cards_in_the_graveyard() {
    let mut g = windgrace();
    let six = perm(&mut g, P0, "Six");
    lands(&mut g, P0, "Forest", 4, false);
    let w = take(&mut g, P0, "Eternal Witness");
    g.player_mut(P0).gy.push(w);
    assert!(opts(&mut g, six, P0, Some(false)).is_empty(), "no land card to discard");
    hand(&mut g, P0, &["Swamp"]);
    let o = opts(&mut g, six, P0, Some(false));
    assert!(run(&mut g, P0, &pick(&o, "Eternal Witness")));
    assert!(!named(&g, P0, "Eternal Witness").is_empty(), "cast from the graveyard");
    assert!(in_gy(&g, P0, "Swamp"), "the land card discarded");
    g.active = Some(P1);
    let c = take(&mut g, P0, "Sol Ring");
    g.player_mut(P0).gy.push(c);
    hand(&mut g, P0, &["Swamp"]);
    assert!(opts(&mut g, six, P0, Some(false)).is_empty(), "only during your turn");
}

// ------------------------------------------------------------------ Soul of Windgrace
#[test]
fn soul_of_windgrace_takes_a_land_from_any_graveyard() {
    let mut g = windgrace();
    let c = take(&mut g, P1, "Island");
    g.player_mut(P1).gy.push(c);
    perm(&mut g, P0, "Soul of Windgrace");
    let l = land_named(&g, P0, "Island").expect("under your control");
    assert!(g.perm(l).tapped && !g.player(P1).gy.contains(&c));
}

#[test]
fn soul_of_windgrace_abilities_need_a_land_card_to_discard() {
    let mut g = windgrace();
    let s = perm(&mut g, P0, "Soul of Windgrace");
    lands(&mut g, P0, "Swamp", 2, false);
    lands(&mut g, P0, "Mountain", 2, false);
    lands(&mut g, P0, "Forest", 2, false);
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Sol Ring"]);
    let mine = |g: &mut Game| {
        ability_options(g, P0, true)
            .unwrap()
            .into_iter()
            .filter(|o| matches!(o.act, Some(Action::Dsl { src, .. }) if src == s))
            .count()
    };
    assert_eq!(mine(&mut g), 0, "Sol Ring is no land card");
    hand(&mut g, P0, &["Forest"]);
    assert!(mine(&mut g) > 0);
}

// ------------------------------------------------------------------ Spelunking
#[test]
fn spelunking_draws_puts_a_land_in_and_lands_enter_untapped() {
    let mut g = windgrace();
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Overgrown Tomb"]);
    on_top(&mut g, P0, "Sol Ring");
    perm(&mut g, P0, "Spelunking");
    assert!(in_hand(&g, P0, "Sol Ring"), "draw a card");
    let l = land_named(&g, P0, "Overgrown Tomb").expect("then a land from hand");
    assert!(!g.perm(l).tapped);
    let t = lands(&mut g, P0, "Golgari Rot Farm", 1, true)[0];
    assert!(!g.perm(t).tapped, "lands you control enter untapped, even put onto the battlefield tapped");
    let o = lands(&mut g, P1, "Forest", 1, true)[0];
    assert!(g.perm(o).tapped, "only yours");
}

// ------------------------------------------------------------------ Tannuk
#[test]
fn tannuk_pings_each_landfall_and_draws_on_the_second() {
    let mut g = windgrace();
    perm(&mut g, P0, "Tannuk, Memorial Ensign");
    let (life, hand0) = (g.player(P1).life, g.player(P0).hand.len());
    zones::landfall(&mut g, P0).unwrap();
    assert_eq!((g.player(P1).life, g.player(P0).hand.len()), (life - 1, hand0));
    zones::landfall(&mut g, P0).unwrap();
    assert_eq!((g.player(P1).life, g.player(P0).hand.len()), (life - 2, hand0 + 1), "the second time: draw");
    zones::landfall(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).hand.len(), hand0 + 1, "only the second");
}

// ------------------------------------------------------------------ Nissa of Shadowed Boughs
#[test]
fn nissa_gains_loyalty_on_landfall_and_her_minus_five_cheats_a_creature() {
    let mut g = windgrace();
    let n = perm(&mut g, P0, "Nissa of Shadowed Boughs");
    let _ = opts(&mut g, n, P0, Some(false)); // loyalty set
    let l0 = g.perm(n).loyalty.unwrap();
    zones::landfall(&mut g, P0).unwrap();
    assert_eq!(g.perm(n).loyalty, Some(l0 + 1));
    lands(&mut g, P0, "Forest", 7, false);
    let c = take(&mut g, P0, "Avenger of Zendikar");
    g.player_mut(P0).gy.push(c);
    g.perm_mut(n).loyalty = Some(5);
    let o = opts(&mut g, n, P0, Some(false));
    assert!(run(&mut g, P0, &pick(&o, "-5")));
    let a = named(&g, P0, "Avenger of Zendikar");
    assert_eq!(a.len(), 1, "mana value 7, seven lands: from the graveyard");
    assert_eq!(g.perm(a[0]).plus, 2, "two +1/+1 counters");
}

#[test]
fn nissa_plus_one_untaps_a_land_and_animates_it_to_attack() {
    let mut g = windgrace();
    let n = perm(&mut g, P0, "Nissa of Shadowed Boughs");
    let t = lands(&mut g, P0, "Forest", 1, true)[0];
    let o = opts(&mut g, n, P0, Some(false));
    assert!(run(&mut g, P0, &pick(&o, "+1")));
    let e = named(&g, P0, "Forest");
    assert_eq!(e.len(), 1, "no blockers: it becomes a 3/3 creature");
    assert!(!g.perm(e[0]).tapped && !g.player(P0).lands.contains(&t));
    assert_eq!((epow(&g, e[0]), etgh(&g, e[0])), (3, 3));
    assert!(has_kw(&g, e[0], "haste") && has_kw(&g, e[0], "menace"));
}

// ------------------------------------------------------------------ Constant Mists
#[test]
fn constant_mists_fogs_a_big_attack_with_buyback() {
    let mut g = windgrace();
    let c = hand(&mut g, P0, &["Constant Mists"])[0];
    lands(&mut g, P0, "Forest", 8, false);
    let atk: Vec<PermId> = (0..3).map(|_| token(&mut g, P1, 6)).collect();
    let f = g.registry.get(c).unwrap().hand_defend.unwrap();
    let mut a = atk.clone();
    f(&mut g, c, P0, P1, &mut a, &mut vec![]).unwrap();
    assert_eq!(g.fog, Some(g.turn_stamp()), "all combat damage prevented this turn");
    assert!(g.player(P0).hand.contains(&c), "buyback: back to hand");
    assert_eq!(g.player(P0).lands.len(), 7, "a land sacrificed");
}

#[test]
fn constant_mists_holds_for_a_small_attack_and_goes_to_the_graveyard_without_spare_lands() {
    let mut g = windgrace();
    let c = hand(&mut g, P0, &["Constant Mists"])[0];
    lands(&mut g, P0, "Forest", 3, false);
    let f = g.registry.get(c).unwrap().hand_defend.unwrap();
    let mut a = vec![token(&mut g, P1, 2)];
    f(&mut g, c, P0, P1, &mut a, &mut vec![]).unwrap();
    assert!(g.fog.is_none() && g.player(P0).hand.contains(&c), "2 damage isn't worth it");
    let mut a: Vec<PermId> = (0..4).map(|_| token(&mut g, P1, 5)).collect();
    f(&mut g, c, P0, P1, &mut a, &mut vec![]).unwrap();
    assert!(g.fog.is_some() && g.player(P0).gy.contains(&c) && g.player(P0).lands.len() == 3, "no buyback");
}

// ------------------------------------------------------------------ Realms Uncharted
#[test]
fn realms_uncharted_finds_four_different_lands_two_to_hand() {
    let mut g = windgrace();
    no_lands(&mut g, P0, true);
    for n in ["Forest", "Forest", "Swamp", "Mountain", "Overgrown Tomb"] {
        on_top(&mut g, P0, n);
    }
    let hand0 = g.player(P0).hand.len();
    resolve(&mut g, P0, "Realms Uncharted", Ctx::default());
    assert_eq!(g.player(P0).hand.len(), hand0 + 2);
    let gy: Vec<CardId> = g.player(P0).gy.iter().copied().filter(|&c| g.db.get(c).land).collect();
    assert_eq!(gy.len(), 2, "the opponent's two");
    let mut names: Vec<&str> =
        g.player(P0).hand.iter().chain(&gy).filter(|&&c| g.db.get(c).land).map(|&c| &*g.db.get(c).name).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), 4, "different names");
}

// ------------------------------------------------------------------ Scapeshift
#[test]
fn scapeshift_trades_lands_for_dark_depths_and_thespians_stage() {
    let mut g = windgrace();
    lands(&mut g, P0, "Forest", 3, false);
    no_lands(&mut g, P0, true);
    for n in ["Swamp", "Dark Depths", "Thespian's Stage", "Mountain"] {
        on_top(&mut g, P0, n);
    }
    resolve(&mut g, P0, "Scapeshift", Ctx::default());
    assert_eq!(g.player(P0).lands.len(), 3, "three sacrificed, three found");
    assert!(land_named(&g, P0, "Dark Depths").is_some() && land_named(&g, P0, "Thespian's Stage").is_some());
    assert!(g.player(P0).lands.iter().all(|&l| g.perm(l).tapped), "tapped");
    assert_eq!(g.player(P0).gy.iter().filter(|&&c| &*g.db.get(c).name == "Forest").count(), 3);
}

// ------------------------------------------------------------------ Worldsoul's Rage
#[test]
fn worldsouls_rage_deals_x_and_puts_x_lands_in() {
    let mut g = windgrace();
    g.player_mut(P0).hand.clear();
    hand(&mut g, P0, &["Forest", "Swamp", "Mountain"]);
    let c = take(&mut g, P0, "Overgrown Tomb");
    g.player_mut(P0).gy.push(c);
    let lives: Vec<i32> = [P1, PlayerId(2)].iter().map(|&q| g.player(q).life).collect();
    resolve(&mut g, P0, "Worldsoul's Rage", Ctx { x: 3, ..Ctx::default() });
    let lost: i32 = [P1, PlayerId(2)].iter().zip(&lives).map(|(&q, &l)| l - g.player(q).life).sum();
    assert_eq!(lost, 3, "X damage to an opponent (no creature worth it)");
    assert_eq!(g.player(P0).lands.len(), 3, "up to X land cards");
    assert!(g.player(P0).lands.iter().all(|&l| g.perm(l).tapped));
    assert!(land_named(&g, P0, "Overgrown Tomb").is_some(), "from the graveyard too");
}

#[test]
fn run_hook_is_used_for_tannuk_too() {
    // the hook path the engine takes (fire_trigger -> run_hook) reaches the hand-written landfall
    let mut g = windgrace();
    let t = perm(&mut g, P0, "Tannuk, Memorial Ensign");
    let life = g.player(P1).life;
    run_hook(&mut g, t, Event::Landfall, &Call::Player { p: P0 }).unwrap();
    assert_eq!(g.player(P1).life, life - 1);
}
