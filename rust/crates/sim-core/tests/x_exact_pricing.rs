//! The AI's {X} and alternative-cost pricing is exact: the mana it plans is the mana the cost pipeline charges. One
//! block per site: Hangarback Walker and Chalice of the Void (`CardImpl::auto_x`), Twinflame's strive, the combo mana
//! count and payment, the Damping Sphere in `seq_cost_x`, the planner's X tutors (its own unit tests in
//! src/ai/planner.rs), harmonize, and the Chulane/Kinnan Ballista lines (which do not cast the Ballista).

mod play_common;

use sim_core::ai::brain;
use sim_core::engine::mana::{self, cost_of, cost_of_with, cost_of_x, seq_cost, seq_cost_x};
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::combos;
use sim_core::impls::karn_engines::hangar_x;
use sim_core::impls::t6_stax::chalice_x;
use sim_core::state::{DataKey, Game};
use sim_core::testkit::*;

const P: PlayerId = PlayerId(0);
const Q: PlayerId = PlayerId(1);

fn game() -> Game {
    play_common::table(&["yshtola", "kefka", "sauron"])
}

fn run(g: &mut Game, o: &Opt) -> bool {
    sim_core::ai::act::perform(g, P, o.act.as_ref().expect("a play")).unwrap()
}

fn tapped(g: &Game) -> u32 {
    g.player(P).lands.iter().filter(|&&m| g.perm(m).tapped).count() as u32
}

fn body(g: &Game, c: CardId) -> PermId {
    g.player(P).perms.iter().copied().find(|&m| g.perm(m).cd == Some(c)).expect("on the battlefield")
}

/// the opponents' libraries are all one cheap card, so Chalice of the Void knows what to name
fn bolts(g: &mut Game, name: &str) {
    let one = card(g, name);
    g.player_mut(Q).library = vec![one; 20];
}

// ------------------------------------------------------------------ 1: Hangarback Walker and Chalice of the Void
#[test]
fn hangarback_under_two_reducers_is_cast_free_at_x_one_and_keeps_its_counter() {
    let mut g = game();
    lands(&mut g, P, "Wastes", 2, false);
    perm(&mut g, P, "Etherium Sculptor");
    perm(&mut g, P, "Etherium Sculptor");
    let c = hand(&mut g, P, &["Hangarback Walker"])[0];
    assert_eq!(
        hangar_x(&g, P),
        1,
        "X = 1 costs {{2}} less two times {{1}}, X = 2 costs {{2}}: the lands pay either, one is chosen"
    );
    assert_eq!(cost_of(&g, P, c), (0, String::new()));
    assert!(brain::do_cast(&mut g, P, c, None).unwrap());
    assert_eq!(tapped(&g), 0);
    let m = body(&g, c);
    assert!(g.perm(m).on_bf);
    assert_eq!(g.perm(m).plus, 1, "a cast that cost nothing still has its X: the Walker enters with a counter");
}

#[test]
fn chalice_under_two_reducers_records_its_x_when_the_cast_costs_nothing() {
    let mut g = game();
    perm(&mut g, P, "Etherium Sculptor");
    perm(&mut g, P, "Etherium Sculptor");
    lands(&mut g, P, "Wastes", 0, false);
    bolts(&mut g, "Lightning Bolt");
    let c = hand(&mut g, P, &["Chalice of the Void"])[0];
    assert_eq!(chalice_x(&g, P), Some(1), "X = 1 costs nothing here, and it names the Bolts");
    assert_eq!(cost_of(&g, P, c), (0, String::new()));
    assert!(brain::do_cast(&mut g, P, c, None).unwrap());
    assert_eq!(g.perm(body(&g, c)).data.int(DataKey::Charge), 1);
}

#[test]
fn chalice_pays_the_reduction_then_the_trinisphere_minimum() {
    let mut g = game();
    lands(&mut g, P, "Wastes", 3, false);
    perm(&mut g, P, "Etherium Sculptor");
    perm(&mut g, Q, "Trinisphere");
    bolts(&mut g, "Lightning Bolt");
    let c = hand(&mut g, P, &["Chalice of the Void"])[0];
    assert_eq!(chalice_x(&g, P), Some(1));
    // {X}{X} at X = 1 is {2}, the Sculptor makes it {1}, Trinisphere raises the whole cost to three
    assert_eq!(cost_of(&g, P, c), (3, String::new()));
    assert!(brain::do_cast(&mut g, P, c, None).unwrap());
    assert_eq!(tapped(&g), 3);
    assert_eq!(g.perm(body(&g, c)).data.int(DataKey::Charge), 1);
}

#[test]
fn chalice_under_tamiyo_costs_nothing_at_x_zero_even_when_it_can_pay_for_no_x() {
    let mut g = game();
    lands(&mut g, P, "Wastes", 1, false);
    g.player_mut(P).emblems.push("tamiyo");
    let c = hand(&mut g, P, &["Chalice of the Void"])[0];
    assert_eq!(chalice_x(&g, P), None, "one land pays for no X");
    assert_eq!(cost_of(&g, P, c), (0, String::new()), "the emblem makes the cast free: not priced out of reach");
    assert!(mana::x_free(&g, P, c));
    assert!(brain::do_cast(&mut g, P, c, None).unwrap());
    assert_eq!(g.perm(body(&g, c)).data.int(DataKey::Charge), 0);
    assert_eq!(tapped(&g), 0);
}

#[test]
fn a_person_is_priced_at_x_zero_as_practice_mode_charges_x_itself() {
    for sculptor in [false, true] {
        let mut g = game();
        play_common::seat(&mut g, P, vec![]);
        lands(&mut g, P, "Wastes", 6, false);
        if sculptor {
            perm(&mut g, P, "Etherium Sculptor");
        }
        bolts(&mut g, "Lightning Bolt");
        for name in ["Hangarback Walker", "Chalice of the Void"] {
            let c = card(&g, name);
            assert_eq!(cost_of(&g, P, c), cost_of_x(&g, P, c, 0), "{name} {sculptor}");
        }
    }
}

#[test]
fn x_is_capped_by_what_the_pipeline_lets_the_ai_pay() {
    // Hangarback: a third of six lands is X = 2, but three Spheres of Resistance make X = 2 cost {7}: it casts X = 1 for {5}
    let mut g = game();
    lands(&mut g, P, "Wastes", 6, false);
    for _ in 0..3 {
        perm(&mut g, Q, "Sphere of Resistance");
    }
    let c = hand(&mut g, P, &["Hangarback Walker"])[0];
    assert_eq!(hangar_x(&g, P), 1);
    assert_eq!(cost_of(&g, P, c), (5, String::new()));
    assert!(brain::do_cast(&mut g, P, c, None).unwrap());
    assert_eq!(tapped(&g), 5);
    assert_eq!(g.perm(body(&g, c)).plus, 1);
    // Chalice: three lands and a Sculptor pay {4} less {1} for X = 2 (the old half of the mana said X = 1)
    let mut g = game();
    lands(&mut g, P, "Wastes", 3, false);
    perm(&mut g, P, "Etherium Sculptor");
    bolts(&mut g, "Mind Stone");
    let c = hand(&mut g, P, &["Chalice of the Void"])[0];
    assert_eq!(chalice_x(&g, P), Some(2));
    assert_eq!(cost_of(&g, P, c), (3, String::new()));
    assert!(brain::do_cast(&mut g, P, c, None).unwrap());
    assert_eq!(g.perm(body(&g, c)).data.int(DataKey::Charge), 2);
    assert_eq!(tapped(&g), 3);
    // and with taxes it never picks an X it cannot pay and then does not cast: two Spheres, four lands, X = 1 for {4}
    let mut g = game();
    lands(&mut g, P, "Wastes", 4, false);
    perm(&mut g, Q, "Sphere of Resistance");
    perm(&mut g, Q, "Sphere of Resistance");
    bolts(&mut g, "Mind Stone");
    let c = hand(&mut g, P, &["Chalice of the Void"])[0];
    assert_eq!(chalice_x(&g, P), Some(1), "X = 2 is {{6}}");
    assert!(brain::do_cast(&mut g, P, c, None).unwrap());
}

// ------------------------------------------------------------------ 2: Twinflame (strive)
fn twinflame_game(titans: usize, extra: &[(&str, PlayerId)], r_lands: usize) -> (Game, CardId) {
    let mut g = game();
    lands(&mut g, P, "Mountain", r_lands, false);
    for &(name, who) in extra {
        perm(&mut g, who, name);
    }
    for _ in 0..titans {
        perm(&mut g, P, "Grave Titan");
    }
    let c = hand(&mut g, P, &["Twinflame"])[0];
    (g, c)
}

fn twinflame_opts(g: &mut Game, c: CardId) -> Vec<Opt> {
    g.registry.get(c).unwrap().hand_options.unwrap()(g, c, P, Some(false)).unwrap()
}

fn targets(o: &Opt) -> usize {
    o.label.split(|ch: char| !ch.is_ascii_digit()).find(|s| !s.is_empty()).unwrap().parse().unwrap()
}

#[test]
fn twinflame_strive_counts_every_symbol_for_trinisphere() {
    // {1}{R} and {2}{R} for the second target: {3}{R}{R}, five mana (already above the minimum of three)
    let (mut g, c) = twinflame_game(2, &[("Trinisphere", Q)], 6);
    let oo = twinflame_opts(&mut g, c);
    let o = oo.iter().find(|o| o.label == "Twinflame (2 targets)").expect("two targets");
    assert!(run(&mut g, o));
    assert_eq!(tapped(&g), 5);
}

#[test]
fn twinflame_with_trinisphere_and_a_ruby_medallion() {
    for (titans, taps) in [(1, 3), (2, 4)] {
        let (mut g, c) = twinflame_game(titans, &[("Trinisphere", Q), ("Ruby Medallion", P)], 6);
        let oo = twinflame_opts(&mut g, c);
        assert_eq!(oo.len(), 1, "{titans}");
        assert_eq!(targets(&oo[0]), titans);
        assert!(run(&mut g, &oo[0]));
        // one target: {1}{R} less {1} is {R}, raised to three; two: {3}{R}{R} less {1}, already above three
        assert_eq!(tapped(&g), taps, "{titans}");
    }
}

#[test]
fn twinflame_pays_what_the_pipeline_prices_for_any_target_count_and_board() {
    for titans in 1..=3 {
        for board in 0..6 {
            let extra: &[(&str, PlayerId)] = match board {
                0 => &[],
                1 => &[("Ruby Medallion", P)],
                2 => &[("Ruby Medallion", P), ("Ruby Medallion", P)],
                3 => &[("Sphere of Resistance", Q)],
                4 => &[("Trinisphere", Q)],
                _ => &[("Trinisphere", Q), ("Ruby Medallion", P), ("Sphere of Resistance", Q)],
            };
            let (mut g, c) = twinflame_game(titans, extra, 12);
            let oo = twinflame_opts(&mut g, c);
            assert!(!oo.is_empty(), "{titans}/{board}");
            let n = targets(&oo[0]);
            let (gn, pips) = cost_of_with(&g, P, c, 2 * (n as i32 - 1), &format!("R{}", "R".repeat(n - 1)));
            assert!(run(&mut g, &oo[0]));
            assert_eq!(tapped(&g), gn + pips.len() as u32, "{titans} Titans, board {board}, {n} targets");
        }
    }
}

// ------------------------------------------------------------------ 3: combos (mana_need and the pay loop), seq_cost_x
fn heliod() -> &'static combos::Combo {
    combos::all().iter().find(|c| c.name == "Heliod, Sun-Crowned + Walking Ballista").unwrap()
}

fn heliod_game(lands_n: usize) -> (Game, CardId, usize) {
    let mut g = game();
    perm(&mut g, P, "Heliod, Sun-Crowned");
    perm(&mut g, P, "Etherium Sculptor");
    lands(&mut g, P, "Plains", 1, false);
    lands(&mut g, P, "Wastes", lands_n - 1, false);
    let c = hand(&mut g, P, &["Walking Ballista"])[0];
    let i = combos::combo_index("Heliod, Sun-Crowned + Walking Ballista").unwrap();
    (g, c, i)
}

#[test]
fn the_heliod_line_runs_on_exactly_the_mana_it_counted() {
    let (mut g, c, i) = heliod_game(3);
    let (gn, pips) = combos::mana_need(&g, P, heliod(), &[c]);
    assert_eq!(
        (gn, pips.as_str()),
        (2, "W"),
        "{{X}}{{X}} at X = 1 is {{2}}, less the Sculptor's {{1}}, plus the loop's {{1}}{{W}}"
    );
    assert!(combos::ready_to_go(&g, P, i));
    assert!(combos::attempt(&mut g, P, i as i64).unwrap());
    assert!(g.over && g.winner == Some(P), "the line wins");
    assert_eq!(tapped(&g), cost_of_x(&g, P, c, 2).0 + 1 + 1, "the Ballista, the loop's {{1}} and its {{W}}");
    let m = body(&g, c);
    assert_eq!(g.perm(m).plus, 1, "the Ballista enters with a counter (half of the mana its cast paid for X)");
}

#[test]
fn one_land_fewer_and_the_line_is_not_affordable_nothing_is_paid() {
    let (mut g, c, i) = heliod_game(2);
    assert!(!combos::ready_to_go(&g, P, i));
    assert!(!combos::attempt(&mut g, P, i as i64).unwrap());
    assert_eq!(tapped(&g), 0);
    assert!(g.player(P).hand.contains(&c), "the Ballista stays in hand");
    assert!(!g.over);
}

#[test]
fn seq_cost_x_with_zero_x_is_seq_cost_and_prices_x_through_the_pipeline() {
    let mut g = game();
    perm(&mut g, P, "Etherium Sculptor");
    let ballista = card(&g, "Walking Ballista");
    let ring = card(&g, "Sol Ring");
    assert_eq!(seq_cost_x(&g, P, &[(ring, 0), (ballista, 0)]), seq_cost(&g, P, &[ring, ballista]));
    let (rg, rp) = cost_of(&g, P, ring);
    let (bg, bp) = cost_of_x(&g, P, ballista, 2);
    assert_eq!(seq_cost_x(&g, P, &[(ring, 0), (ballista, 2)]), (rg + bg, format!("{rp}{bp}")));
}

#[test]
fn damping_sphere_in_a_line_is_priced_through_the_pipeline_with_trinisphere() {
    let mut g = game();
    perm(&mut g, Q, "Damping Sphere");
    perm(&mut g, Q, "Trinisphere");
    let (ring, bolt) = (card(&g, "Sol Ring"), card(&g, "Lightning Bolt"));
    // the Bolt after the Ring: {R} and {1} for the Sphere is two mana, raised to three by Trinisphere (the Sphere's
    // tax is not added on top of the minimum)
    let mut after = g.clone();
    let st = after.turn_stamp();
    after.player_mut(P).turn_casts = Some((st, vec![ring]));
    let exact_bolt = cost_of(&after, P, bolt);
    assert_eq!(exact_bolt, (2, "R".into()));
    let (rg, rp) = cost_of(&g, P, ring);
    assert_eq!(seq_cost(&g, P, &[ring, bolt]), (rg + exact_bolt.0, format!("{rp}R")));
    assert_eq!(seq_cost(&g, P, &[ring, bolt]), (5, "R".into()));
    // a land is no spell: nothing for the Sphere to count
    let tomb = card(&g, "Ancient Tomb");
    assert_eq!(seq_cost(&g, P, &[tomb, bolt]), (cost_of(&g, P, tomb).0 + 2, "R".into()), "{:?}", cost_of(&g, P, tomb));
}

#[test]
fn damping_sphere_after_casts_already_made_this_turn_still_counts_them_once() {
    let mut g = game();
    perm(&mut g, Q, "Damping Sphere");
    let (ring, bolt) = (card(&g, "Sol Ring"), card(&g, "Lightning Bolt"));
    let st = g.turn_stamp();
    g.player_mut(P).turn_casts = Some((st, vec![ring]));
    // one spell already cast: the Ring in the line costs {1} more, the Bolt after it {2} more
    assert_eq!(seq_cost(&g, P, &[ring, bolt]), (1 + 1 + 2, "R".into()));
}

// ------------------------------------------------------------------ 5: harmonize
#[test]
fn harmonize_under_trinisphere_a_sphere_and_two_medallions_floors_exactly() {
    for x_power in 1..=6 {
        let mut g = game();
        lands(&mut g, P, "Forest", 5, false);
        perm(&mut g, Q, "Trinisphere");
        perm(&mut g, Q, "Sphere of Resistance");
        perm(&mut g, P, "Emerald Medallion");
        perm(&mut g, P, "Emerald Medallion");
        token(&mut g, P, x_power);
        let c = card(&g, "Nature's Rhythm");
        g.player_mut(P).gy.push(c);
        let target = card(&g, "Consecrated Sphinx");
        g.player_mut(P).library = vec![target];
        let f = g.registry.get(c).unwrap().gy_options.unwrap();
        let oo = f(&mut g, c, P, Some(false)).unwrap();
        let paid = (0..=64).rev().find(|&x| {
            let (gn, pips) = cost_of_with(&g, P, c, x, "GGGG");
            mana::can_pay_spell(&g, P, c, gn, &pips, false)
        });
        let Some(paid) = paid else {
            assert!(oo.is_empty());
            continue;
        };
        let (gn, _) = cost_of_with(&g, P, c, paid, "GGGG");
        if oo.is_empty() {
            continue; // (the plan wants X of at least 3: harmonize_x + the creature's power)
        }
        assert!(run(&mut g, &oo[0]));
        assert_eq!(tapped(&g), gn + 4, "power {x_power}");
    }
}

// ------------------------------------------------------------------ 4: the Chulane and Kinnan Ballista lines
#[test]
fn the_tender_line_with_the_aura_and_a_ballista_in_hand_pays_only_for_the_aura() {
    // The loop's unbounded mana pays for the Ballista, as `combos::mana_sink` models every unbounded-mana line: the line
    // reserves mana for the Aura alone. Bloom Tender out, Freed from the Real and the Ballista in hand, 2 Islands + 1
    // Forest (3 lands + the Tender's {G} = 4 mana): {2}{U} for the Aura, ready, and it wins.
    let mut g = table(&["kinnan-simic-mana-combo", "seph", "sauron"]);
    perm(&mut g, P, "Bloom Tender");
    lands(&mut g, P, "Island", 2, false);
    lands(&mut g, P, "Forest", 1, false);
    let hh = hand(&mut g, P, &["Freed from the Real", "Walking Ballista"]);
    let (aura, ballista) = (hh[0], hh[1]);
    let i = combos::combo_index("Aura + Bloom Tender").unwrap();
    let cmb = combos::all()[i];
    let (ok, _, casts) = (cmb.ready)(&g, P);
    assert!(ok);
    assert_eq!(casts, vec![aura], "the Aura is the line's only cast: the Ballista is paid by the loop");
    let (ag, ap) = cost_of(&g, P, aura);
    assert_eq!(combos::mana_need(&g, P, cmb, &casts), (ag, ap), "mana_need is the Aura's price alone");
    assert!(combos::ready_to_go(&g, P, i));
    assert!(combos::attempt(&mut g, P, i as i64).unwrap());
    assert!(g.over && g.winner == Some(P));
    assert_eq!(tapped(&g), ag + 1, "only the Aura was paid");
    assert!(g.player(P).hand.contains(&ballista), "the Ballista was not cast with the lands");
}

#[test]
fn a_ballista_only_thrasios_finds_is_not_cast_and_the_xcost_is_mana() {
    let mut g = table(&["kinnan-simic-mana-combo", "seph", "sauron"]);
    perm(&mut g, P, "Freed from the Real");
    perm(&mut g, P, "Bloom Tender");
    perm(&mut g, P, "Thrasios, Triton Hero");
    let ballista = card(&g, "Walking Ballista");
    g.player_mut(P).library.insert(0, ballista);
    let i = combos::combo_index("Aura + Bloom Tender").unwrap();
    let (ok, _, casts) = (combos::all()[i].ready)(&g, P);
    assert!(ok && casts.is_empty(), "the outlet is in the library: nothing to cast");
    for part in ["Aura + Bloom Tender", "Nyxbloom Ancient", "Badgermole Cub + a blue"] {
        let cmb = combos::all()[combos::combo_index(part).unwrap()];
        assert_eq!(cmb.xcost, &[("Walking Ballista", 2)], "{part}: the mana spent on X, as the Heliod line's");
    }
}
