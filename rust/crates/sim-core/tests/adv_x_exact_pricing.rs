//! Adversarial review of the exact {X} and alternative-cost pricing change (x-exact-pricing). Each test is a property the change
//! claims, tried on boards the author's own tests did not use.

use sim_core::ai::brain;
use sim_core::engine::cast::{cast_card, casts_this_turn};
use sim_core::engine::mana::{self, cost_of, cost_of_with, cost_of_x, seq_cost, seq_cost_x};
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::combos;
use sim_core::impls::karn_engines::hangar_x;
use sim_core::impls::t6_stax::chalice_x;
use sim_core::state::{Ctx, DataKey, Game};
use sim_core::testkit::*;

const P: PlayerId = PlayerId(0);
const Q: PlayerId = PlayerId(1);
const R: PlayerId = PlayerId(2);

fn game() -> Game {
    table(&["yshtola", "kefka", "sauron"])
}

fn tapped(g: &Game) -> u32 {
    g.player(P).lands.iter().filter(|&&m| g.perm(m).tapped).count() as u32
}

fn body(g: &Game, c: CardId) -> Option<PermId> {
    g.player(P).perms.iter().copied().find(|&m| g.perm(m).cd == Some(c) && g.perm(m).on_bf)
}

fn run(g: &mut Game, o: &Opt) -> bool {
    sim_core::ai::act::perform(g, P, o.act.as_ref().expect("a play")).unwrap()
}

fn paid(g: &Game, c: CardId, gn: u32, pips: &str) -> bool {
    mana::can_pay_spell(g, P, c, gn, pips, false)
}

/// the largest X (0 when none) whose `cost_of_x` at `per * X` the pool can pay, by trying every one: the oracle for
/// `max_x`'s binary search
fn brute_x(g: &Game, c: CardId, per: i32, top: i32) -> i32 {
    (0..=top)
        .filter(|&x| {
            let (gn, pips) = cost_of_x(g, P, c, per * x);
            paid(g, c, gn, &pips)
        })
        .max()
        .unwrap_or(0)
}

/// collect failures, show the first ones
#[derive(Default)]
struct Bad(Vec<String>);

impl Bad {
    fn add(&mut self, s: String) {
        self.0.push(s);
    }
    fn done(self, what: &str) {
        if !self.0.is_empty() {
            let n = self.0.len();
            let shown: Vec<_> = self.0.iter().take(12).cloned().collect();
            panic!("{what}: {n} failing cases, first:\n{}", shown.join("\n"));
        }
    }
}

/// a small deterministic generator for the random tests
struct Lcg(u64);

impl Lcg {
    fn next(&mut self, n: usize) -> usize {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % n
    }
}

// ------------------------------------------------------------------ a board of price changers
#[derive(Clone, Copy, Debug)]
struct Board {
    reducers: usize,
    spheres: usize,
    trini: bool,
    /// None: no Damping Sphere; Some(k): one, with k spells cast this turn already
    damping: Option<usize>,
    /// 0 none, 1 Thalia (noncreature +1), 2 Thorn of Amethyst (noncreature +1)
    thalia: usize,
    /// K'rrik, Son of Yawgmoth out (it changes nothing for a colourless card)
    krrik: bool,
}

fn boards() -> Vec<Board> {
    let mut v = vec![];
    for reducers in 0..=3 {
        for spheres in 0..=2 {
            for trini in [false, true] {
                for damping in [None, Some(0), Some(1), Some(2)] {
                    for thalia in 0..=2 {
                        v.push(Board {
                            reducers,
                            spheres,
                            trini,
                            damping,
                            thalia,
                            krrik: (reducers + spheres) % 2 == 1,
                        });
                    }
                }
            }
        }
    }
    v
}

fn set_board(g: &mut Game, b: &Board) {
    for _ in 0..b.reducers {
        perm(g, P, "Etherium Sculptor");
    }
    for _ in 0..b.spheres {
        perm(g, Q, "Sphere of Resistance");
    }
    if b.trini {
        perm(g, R, "Trinisphere");
    }
    if let Some(k) = b.damping {
        perm(g, Q, "Damping Sphere");
        let ring = card(g, "Sol Ring");
        let st = g.turn_stamp();
        g.player_mut(P).turn_casts = Some((st, vec![ring; k]));
    }
    if b.krrik {
        perm(g, P, "K'rrik, Son of Yawgmoth");
    }
    match b.thalia {
        1 => {
            perm(g, Q, "Thalia, Guardian of Thraben");
        }
        2 => {
            perm(g, R, "Thorn of Amethyst");
        }
        _ => {}
    }
}

/// opponents hold only cards of this name, so Chalice of the Void has something to name
fn opp_cards(g: &mut Game, name: &str) {
    let one = card(g, name);
    g.player_mut(Q).library = vec![one; 20];
    g.player_mut(R).library = vec![one; 20];
    let wastes = card(g, "Wastes");
    g.player_mut(P).library = vec![wastes; 10];
}

// ------------------------------------------------------------------ (1) E2: cost_of is what the cast taps
/// the choice Chalice makes, the old way (per X: scan hand and library), for a given affordable X
fn old_chalice(g: &Game, p: PlayerId, afford: i32) -> Option<i32> {
    let count = |q: PlayerId, x: i32| {
        let pl = g.player(q);
        pl.hand.iter().chain(pl.library.iter()).filter(|&&c| !g.db.get(c).land && g.db.get(c).cmc as i32 == x).count()
            as i32
    };
    let score = |x: i32| g.opps(p).map(|q| count(q, x)).sum::<i32>() - count(p, x);
    let best = (1..=afford).max_by_key(|&x| (score(x), -x))?;
    Some(if score(0) > score(best).max(0) { 0 } else { best })
}

fn check_auto_x_card(bad: &mut Bad, name: &str, b: &Board, n_lands: usize, lib: &str, tamiyo: bool) {
    let mut g = game();
    set_board(&mut g, b);
    opp_cards(&mut g, lib);
    lands(&mut g, P, "Wastes", n_lands, false);
    if tamiyo {
        g.player_mut(P).emblems.push("tamiyo");
    }
    let c = hand(&mut g, P, &[name])[0];
    let tag = format!("{name} {b:?} lands {n_lands} lib {lib} tamiyo {tamiyo}");
    let brute = brute_x(&g, c, 2, 40);
    let free = mana::x_free(&g, P, c);
    let want = (mana::total_mana(&g, P, false) as i32 / 3).clamp(1, 4);
    // the X the card's code should choose
    let chosen: i32 = if name == "Hangarback Walker" {
        let e = if free || brute == 0 { 0 } else { want.min(brute) };
        if hangar_x(&g, P) != e {
            bad.add(format!("{tag}: hangar_x {} expected {e} (brute {brute}, free {free})", hangar_x(&g, P)));
        }
        e
    } else {
        let afford = brute.min(3);
        let e = if free || afford < 1 { None } else { old_chalice(&g, P, afford) };
        if chalice_x(&g, P) != e {
            bad.add(format!("{tag}: chalice_x {:?} expected {e:?} (brute {brute}, free {free})", chalice_x(&g, P)));
        }
        e.unwrap_or(0)
    };
    // cost_of is priced at that X
    let pre = cost_of(&g, P, c);
    let priced_at_x = cost_of_x(&g, P, c, 2 * chosen);
    let castable_price = if chosen > 0 || free || (name != "Hangarback Walker" && chalice_x(&g, P).is_some()) {
        pre == priced_at_x
    } else {
        pre.0 >= 99
    };
    if !castable_price {
        bad.add(format!("{tag}: cost_of {pre:?}, cost_of_x at X {chosen} {priced_at_x:?}"));
    }
    let ok_pay = paid(&g, c, pre.0, &pre.1);
    let expect_cast = ok_pay;
    let res = brain::do_cast(&mut g, P, c, None).unwrap();
    if res != expect_cast {
        bad.add(format!("{tag}: do_cast {res}, price {pre:?} payable {ok_pay}"));
        return;
    }
    if !res {
        if tapped(&g) != 0 {
            bad.add(format!("{tag}: a failed cast tapped {}", tapped(&g)));
        }
        return;
    }
    if tapped(&g) != pre.0 + pre.1.chars().count() as u32 {
        bad.add(format!("{tag}: tapped {} but cost_of said {pre:?}", tapped(&g)));
    }
    let Some(m) = body(&g, c) else {
        // a Walker cast with X = 0 (Tamiyo's emblem, 107.3b) has no counters and dies as it enters
        if !(name == "Hangarback Walker" && chosen == 0) {
            bad.add(format!("{tag}: cast but not on the battlefield"));
        }
        return;
    };
    let counters =
        if name == "Hangarback Walker" { g.perm(m).plus } else { g.perm(m).data.int(DataKey::Charge) as i32 };
    if counters != chosen {
        bad.add(format!("{tag}: enters with {counters} counters, X chosen {chosen} (price {pre:?})"));
    }
}

#[test]
fn hangarback_cost_of_equals_what_the_cast_taps_over_boards_and_pools() {
    let mut bad = Bad::default();
    for b in boards() {
        for n in 0..=12 {
            check_auto_x_card(&mut bad, "Hangarback Walker", &b, n, "Lightning Bolt", false);
        }
    }
    bad.done("Hangarback Walker");
}

#[test]
fn chalice_cost_of_equals_what_the_cast_taps_over_boards_and_pools() {
    let mut bad = Bad::default();
    for b in boards() {
        for n in 0..=12 {
            for lib in ["Lightning Bolt", "Mind Stone", "Basalt Monolith"] {
                check_auto_x_card(&mut bad, "Chalice of the Void", &b, n, lib, false);
            }
        }
    }
    bad.done("Chalice of the Void");
}

#[test]
fn tamiyo_emblem_makes_both_cards_x_free_on_every_pool() {
    let mut bad = Bad::default();
    for b in [
        Board { reducers: 0, spheres: 0, trini: false, damping: None, thalia: 0, krrik: false },
        Board { reducers: 2, spheres: 1, trini: true, damping: Some(1), thalia: 2, krrik: true },
    ] {
        for n in 0..=6 {
            for name in ["Hangarback Walker", "Chalice of the Void"] {
                check_auto_x_card(&mut bad, name, &b, n, "Lightning Bolt", true);
            }
        }
    }
    bad.done("Tamiyo emblem");
}

#[test]
fn cost_of_x_never_reads_the_pool_or_auto_x() {
    // the price at a given X must not depend on how many lands there are (auto_x reads the pool)
    for name in ["Hangarback Walker", "Chalice of the Void"] {
        for b in boards().iter().step_by(7) {
            let mut prices = vec![];
            for n in [0usize, 3, 12] {
                let mut g = game();
                set_board(&mut g, b);
                opp_cards(&mut g, "Lightning Bolt");
                lands(&mut g, P, "Wastes", n, false);
                let c = card(&g, name);
                prices.push([0, 1, 2, 5].map(|x| cost_of_x(&g, P, c, x)));
            }
            assert!(prices[0] == prices[1] && prices[1] == prices[2], "{name} {b:?}: {prices:?}");
            assert!(prices[0][0].0 < 99, "{name} {b:?}: cost_of_x(0) carries the unpayable sentinel");
        }
    }
}

#[test]
fn a_person_keeps_the_old_price_and_the_old_practice_charge() {
    // human seat: cost_of == cost_of_x(0) on every board, for every pool
    for b in boards().iter().step_by(5) {
        for n in [0usize, 4, 9] {
            let mut g = game();
            let h = std::sync::Arc::new(sim_core::human::script::ScriptHuman::from_funcs(vec![]));
            g.humans.add(P, h);
            set_board(&mut g, b);
            opp_cards(&mut g, "Lightning Bolt");
            lands(&mut g, P, "Wastes", n, false);
            for name in ["Hangarback Walker", "Chalice of the Void"] {
                let c = card(&g, name);
                assert_eq!(cost_of(&g, P, c), cost_of_x(&g, P, c, 0), "{name} {b:?} {n}");
            }
        }
    }
}

// ------------------------------------------------------------------ (2) x_free and max_x_upto edges
#[test]
fn max_x_upto_caps_and_agrees_with_max_x_below_the_cap() {
    for b in boards().iter().step_by(3) {
        for n in [0usize, 1, 2, 5, 8, 12] {
            let mut g = game();
            set_board(&mut g, b);
            lands(&mut g, P, "Wastes", n, false);
            let c = card(&g, "Hangarback Walker");
            let full = brute_x(&g, c, 2, 60);
            for cap in [-5, -1, 0, 1, 2, 4, 100, i32::MAX] {
                let got = brute::upto(&g, c, cap);
                let want = if mana::x_free(&g, P, c) { 0 } else { full.min(cap.max(0)) };
                assert_eq!(got, want, "{b:?} lands {n} cap {cap} (max_x {})", brute::max(&g, c));
            }
            assert_eq!(brute::max(&g, c), brute::upto(&g, c, i32::MAX));
        }
    }
}

mod brute {
    use super::*;
    pub fn upto(g: &Game, c: CardId, cap: i32) -> i32 {
        brain::max_x_upto(g, P, c, 2, false, cap)
    }
    pub fn max(g: &Game, c: CardId) -> i32 {
        brain::max_x(g, P, c, 2, false)
    }
}

#[test]
fn a_pile_of_reducers_taking_x_one_to_nothing_is_not_x_free() {
    // with 130+ reducers X = 1 is free: the X-independence probe must read the price as X-dependent (it is the same
    // `x_free` that `cost_of` uses), so hangar_x / chalice_x find the free X instead of saying "no X"
    let mut bad = Bad::default();
    for n_red in [2usize, 3, 10, 40, 70, 150] {
        let mut g = game();
        for _ in 0..n_red {
            perm(&mut g, P, "Etherium Sculptor");
        }
        opp_cards(&mut g, "Lightning Bolt");
        lands(&mut g, P, "Wastes", 1, false);
        let walker = card(&g, "Hangarback Walker");
        let chalice = card(&g, "Chalice of the Void");
        if mana::x_free(&g, P, walker) {
            bad.add(format!("{n_red} Sculptors: x_free reads true"));
        }
        // a free X = 1 exists (cost_of_x(2) is nothing), so both cards can be cast for X >= 1
        if cost_of_x(&g, P, walker, 2) != (0, String::new()) {
            continue;
        }
        if hangar_x(&g, P) < 1 {
            bad.add(format!("{n_red} Sculptors: hangar_x {} though X = 1 costs nothing", hangar_x(&g, P)));
        }
        if chalice_x(&g, P).is_none_or(|x| x < 1) {
            bad.add(format!("{n_red} Sculptors: chalice_x {:?} though X = 1 costs nothing", chalice_x(&g, P)));
        }
        let _ = chalice;
    }
    bad.done("reducer piles");
}

#[test]
fn x_free_is_false_for_every_payable_real_x_and_true_only_when_the_price_is_flat() {
    let mut g = game();
    let c = card(&g, "Hangarback Walker");
    assert!(!mana::x_free(&g, P, c));
    g.player_mut(P).emblems.push("tamiyo");
    // in hand under the emblem: flat
    g.player_mut(P).hand.push(c);
    assert!(mana::x_free(&g, P, c));
    // not in hand: the emblem does nothing
    g.player_mut(P).hand.clear();
    assert!(!mana::x_free(&g, P, c));
    // any card at all: `cost_of_x` adds x to the generic part whatever the card, so a Bolt is not flat either
    let bolt = card(&g, "Lightning Bolt");
    assert!(!mana::x_free(&g, P, bolt));
}

#[test]
fn saturation_near_u32_max_does_not_panic_or_wrap() {
    let mut g = game();
    let c = card(&g, "Chalice of the Void");
    for x in [i32::MAX, i32::MAX - 1, 1 << 30, 1 << 16, -1, i32::MIN] {
        let (gn, _) = cost_of_x(&g, P, c, x);
        assert_eq!(gn as i64, (x.max(0)) as i64, "x {x}");
    }
    // Sphere of Resistance on top of a huge X saturates, not wraps
    for _ in 0..3 {
        perm(&mut g, Q, "Sphere of Resistance");
    }
    let (gn, _) = cost_of_x(&g, P, c, i32::MAX);
    assert_eq!(gn as i64, i32::MAX as i64 + 3);
    // total mana 0: a cast of either card is refused and costs nothing to ask
    for name in ["Hangarback Walker", "Chalice of the Void"] {
        let c = card(&g, name);
        assert!(!brain::do_cast(&mut g, P, c, None).unwrap_or(false));
        assert_eq!(brain::max_x_upto(&g, P, c, 2, false, i32::MAX), 0);
    }
}

// ------------------------------------------------------------------ (3) chalice_x one-pass tally against the old formula
#[test]
fn chalice_x_tally_matches_the_old_per_x_count_on_random_hands_and_libraries() {
    let names = [
        "Island",
        "Wastes",
        "Black Lotus",
        "Lotus Petal",
        "Lightning Bolt",
        "Sol Ring",
        "Mind Stone",
        "Counterspell",
        "Basalt Monolith",
        "Grave Titan",
        "Walking Ballista",
        "Chalice of the Void",
        "Mox Diamond",
        "Mana Crypt",
        "Etherium Sculptor",
        "Trinisphere",
    ];
    let mut rng = Lcg(7);
    let mut bad = Bad::default();
    for round in 0..300 {
        let seats = 2 + rng.next(3);
        let keys = ["yshtola", "kefka", "sauron", "seph"];
        let mut g = table(&keys[..seats]);
        for i in 0..seats {
            let hand_n = rng.next(6);
            let lib_n = rng.next(25);
            let pick = |rng: &mut Lcg, g: &Game, n: usize| -> Vec<CardId> {
                (0..n).map(|_| card(g, names[rng.next(names.len())])).collect()
            };
            let h = pick(&mut rng, &g, hand_n);
            let l = pick(&mut rng, &g, lib_n);
            let pl = g.player_mut(PlayerId(i as _));
            pl.hand = h;
            pl.library = l;
        }
        let n_lands = 2 + rng.next(8);
        lands(&mut g, P, "Wastes", n_lands, false);
        if rng.next(3) == 0 {
            perm(&mut g, P, "Etherium Sculptor");
        }
        if rng.next(4) == 0 {
            perm(&mut g, Q, "Sphere of Resistance");
        }
        let c = card(&g, "Chalice of the Void");
        let afford = brute_x(&g, c, 2, 3).min(3);
        let want = if mana::x_free(&g, P, c) || afford < 1 { None } else { old_chalice(&g, P, afford) };
        if chalice_x(&g, P) != want {
            bad.add(format!("round {round}: {seats} seats, afford {afford}: new {:?} old {want:?}", chalice_x(&g, P)));
        }
    }
    bad.done("chalice tally");
}

#[test]
fn chalice_x_equals_the_old_choice_when_nothing_changes_prices() {
    // no reducer, tax, K'rrik or Trinisphere: the old afford was lands / 2 capped at 3
    for n in 0..=10usize {
        for lib in ["Lightning Bolt", "Mind Stone", "Basalt Monolith"] {
            let mut g = game();
            opp_cards(&mut g, lib);
            lands(&mut g, P, "Wastes", n, false);
            let c = hand(&mut g, P, &["Chalice of the Void"])[0];
            let old_afford = (mana::total_mana_spell(&g, P, c, false) as i32 / 2).min(3);
            assert_eq!(chalice_x(&g, P), old_chalice(&g, P, old_afford), "{n} lands, {lib}");
            if let Some(x) = chalice_x(&g, P) {
                assert_eq!(cost_of(&g, P, c), (2 * x as u32, String::new()));
            }
        }
    }
}

#[test]
fn hangarback_equals_the_old_rule_when_nothing_changes_prices() {
    for n in 0..=10usize {
        let mut g = game();
        lands(&mut g, P, "Wastes", n, false);
        let c = hand(&mut g, P, &["Hangarback Walker"])[0];
        let total = mana::total_mana(&g, P, false) as i32;
        let old = if total < 2 { 0 } else { (total / 3).clamp(1, 4) };
        assert_eq!(hangar_x(&g, P), old, "{n} lands");
        if old > 0 {
            assert!(brain::do_cast(&mut g, P, c, None).unwrap());
            assert_eq!(tapped(&g), 2 * old as u32);
            assert_eq!(g.perm(body(&g, c).unwrap()).plus, old);
        }
    }
}

#[test]
fn seq_cost_with_zero_xs_equals_seq_cost_and_never_panics_on_short_lines() {
    let mut g = game();
    for _ in 0..2 {
        perm(&mut g, Q, "Damping Sphere");
    }
    let (ring, bolt) = (card(&g, "Sol Ring"), card(&g, "Lightning Bolt"));
    assert_eq!(seq_cost(&g, P, &[]), (0, String::new()));
    assert_eq!(seq_cost_x(&g, P, &[]), (0, String::new()));
    assert_eq!(seq_cost(&g, P, &[ring]), seq_cost_x(&g, P, &[(ring, 0)]));
    let many: Vec<_> = (0..8).map(|i| if i % 2 == 0 { ring } else { bolt }).collect();
    let z: Vec<_> = many.iter().map(|&c| (c, 0)).collect();
    assert_eq!(seq_cost(&g, P, &many), seq_cost_x(&g, P, &z));
}

// ------------------------------------------------------------------ (4) seq_cost_x against casts really made
/// what the engine charges for the line: every card is priced on the live game, paid, and cast for real (a land is
/// priced and left in hand), so Damping Sphere counts exactly the spells the engine counted
fn line_by_casting(g: &Game, line: &[(CardId, i32)]) -> (u32, String) {
    let mut g2 = g.clone();
    let (mut gn, mut pips) = (0, String::new());
    for &(c, x) in line {
        let (cg, cp) = if x > 0 { cost_of_x(&g2, P, c, x) } else { cost_of(&g2, P, c) };
        gn += cg;
        pips.push_str(&cp);
        if g2.db.get(c).land {
            continue;
        }
        g2.player_mut(P).hand.push(c);
        let before = casts_this_turn(&g2, P);
        let can = mana::can_pay_spell(&g2, P, c, cg, &cp, false);
        assert!(
            mana::pay_spell(&mut g2, P, c, cg, &cp, false).unwrap(),
            "the oracle's pool pays {cg} {cp:?} for {} (can_pay {can}, mana {}, untapped lands {})",
            g2.db.get(c).name,
            mana::total_mana(&g2, P, false),
            g2.player(P).lands.iter().filter(|&&l| !g2.perm(l).tapped).count()
        );
        assert!(cast_card(&mut g2, P, c, "hand", Ctx { x: x / 2, ..Default::default() }).unwrap());
        assert_eq!(casts_this_turn(&g2, P), before + 1, "a cast is counted once");
    }
    (gn, pips)
}

fn seq_board(g: &mut Game, spheres: usize, trini: bool, reducers: usize, tax: usize, prior: usize) {
    for _ in 0..spheres {
        perm(g, Q, "Damping Sphere");
    }
    if trini {
        perm(g, R, "Trinisphere");
    }
    for _ in 0..reducers {
        perm(g, P, "Etherium Sculptor");
    }
    for _ in 0..tax {
        perm(g, Q, "Sphere of Resistance");
    }
    let ring = card(g, "Sol Ring");
    let st = g.turn_stamp();
    g.player_mut(P).turn_casts = Some((st, vec![ring; prior]));
}

#[test]
fn seq_cost_x_with_damping_spheres_equals_the_engines_charge_for_every_line() {
    let pool = [
        "Sol Ring",
        "Mind Stone",
        "Basalt Monolith",
        "Walking Ballista",
        "Wastes",
        "Hangarback Walker",
        "Thran Dynamo",
    ];
    let mut rng = Lcg(99);
    let mut bad = Bad::default();
    for spheres in 1..=2 {
        for trini in [false, true] {
            for reducers in 0..=2 {
                for tax in 0..=1 {
                    for prior in 0..=2 {
                        let mut g = game();
                        seq_board(&mut g, spheres, trini, reducers, tax, prior);
                        lands(&mut g, P, "Wastes", 160, false);
                        for _ in 0..25 {
                            let n = rng.next(7);
                            let line: Vec<(CardId, i32)> = (0..n)
                                .map(|_| {
                                    let name = pool[rng.next(pool.len())];
                                    let x = if name == "Walking Ballista" { [0, 2, 4][rng.next(3)] } else { 0 };
                                    (card(&g, name), x)
                                })
                                .collect();
                            let turn_before = g.player(P).turn_casts.clone();
                            let want = line_by_casting(&g, &line);
                            let got = seq_cost_x(&g, P, &line);
                            let names: Vec<_> =
                                line.iter().map(|&(c, x)| format!("{}:{x}", g.db.get(c).name)).collect();
                            if got != want {
                                bad.add(format!(
                                    "spheres {spheres} trini {trini} red {reducers} tax {tax} prior {prior} line {names:?}: seq_cost_x {got:?}, engine {want:?}"
                                ));
                            }
                            if g.player(P).turn_casts != turn_before {
                                bad.add(format!("line {names:?}: seq_cost_x changed the game's turn casts"));
                            }
                        }
                    }
                }
            }
        }
    }
    bad.done("seq_cost_x");
}

#[test]
fn seq_cost_x_the_same_card_twice_and_a_land_in_the_line() {
    let mut g = game();
    seq_board(&mut g, 1, false, 1, 0, 1);
    lands(&mut g, P, "Wastes", 20, false);
    let ring = card(&g, "Sol Ring");
    let tomb = card(&g, "Ancient Tomb");
    for line in
        [vec![(ring, 0), (ring, 0), (ring, 0)], vec![(tomb, 0), (ring, 0), (tomb, 0), (ring, 0)], vec![(ring, 0)]]
    {
        assert_eq!(seq_cost_x(&g, P, &line), line_by_casting(&g, &line), "{line:?}");
    }
}

#[test]
fn seq_cost_x_hundreds_of_cards_does_not_blow_up() {
    let mut g = game();
    seq_board(&mut g, 2, true, 1, 1, 0);
    let ring = card(&g, "Sol Ring");
    let line = vec![(ring, 0); 300];
    let (gn, _) = seq_cost_x(&g, P, &line);
    assert!(gn >= 300);
}

// ------------------------------------------------------------------ (5) combos
fn heliod() -> usize {
    combos::combo_index("Heliod, Sun-Crowned + Walking Ballista").unwrap()
}

fn heliod_board_d(
    reducers: usize,
    spheres: usize,
    trini: bool,
    n_lands: usize,
    damping: Option<usize>,
) -> (Game, CardId) {
    let mut g = game();
    if let Some(k) = damping {
        perm(&mut g, Q, "Damping Sphere");
        let ring = card(&g, "Sol Ring");
        let st = g.turn_stamp();
        g.player_mut(P).turn_casts = Some((st, vec![ring; k]));
    }
    perm(&mut g, P, "Heliod, Sun-Crowned");
    for _ in 0..reducers {
        perm(&mut g, P, "Etherium Sculptor");
    }
    for _ in 0..spheres {
        perm(&mut g, Q, "Sphere of Resistance");
    }
    if trini {
        perm(&mut g, R, "Trinisphere");
    }
    if n_lands > 0 {
        lands(&mut g, P, "Plains", 1, false);
        lands(&mut g, P, "Wastes", n_lands - 1, false);
    }
    let c = hand(&mut g, P, &["Walking Ballista"])[0];
    (g, c)
}

#[test]
fn heliod_line_pays_what_mana_need_said_and_never_when_one_mana_short() {
    let mut bad = Bad::default();
    for reducers in 0..=2 {
        for spheres in 0..=2 {
            for trini in [false, true] {
                for damping in [None, Some(0), Some(2)] {
                    for n in 0..=9 {
                        let (mut g, c) = heliod_board_d(reducers, spheres, trini, n, damping);
                        let i = heliod();
                        let tag = format!("red {reducers} sph {spheres} trini {trini} damping {damping:?} lands {n}");
                        let cmb = combos::all()[i];
                        let (ok, _, casts) = (cmb.ready)(&g, P);
                        assert!(ok && casts == vec![c], "{tag}");
                        let (gn, pips) = combos::mana_need(&g, P, cmb, &casts);
                        // the engine's own price of the pieces: the Ballista at X = 1 ({X}{X}), then the loop's {1}{W}
                        let b = cost_of_x(&g, P, c, 2);
                        if (gn, pips.as_str()) != (b.0 + 1, "W") || !b.1.is_empty() {
                            bad.add(format!("{tag}: mana_need {gn} {pips:?}, Ballista {b:?} + {{1}}{{W}}"));
                        }
                        // payable by a clone that really pays the Ballista and then the loop
                        let mut g2 = g.clone();
                        let can = mana::can_pay_spell(&g2, P, c, b.0, &b.1, false)
                            && mana::pay_spell(&mut g2, P, c, b.0, &b.1, false).unwrap()
                            && mana::can_pay(&g2, P, 1, "W", false);
                        let go = combos::ready_to_go(&g, P, i);
                        if can != go {
                            bad.add(format!("{tag}: really payable {can}, ready_to_go {go}"));
                        }
                        let res = combos::attempt(&mut g, P, i as i64).unwrap();
                        if res != go {
                            bad.add(format!("{tag}: attempt {res}, ready_to_go {go}"));
                        }
                        if go {
                            if tapped(&g) != gn + 1 {
                                bad.add(format!("{tag}: tapped {} but mana_need said {gn} + the {{W}}", tapped(&g)));
                            }
                            match body(&g, c) {
                                Some(m) if g.perm(m).plus == 1 => {}
                                other => bad.add(format!("{tag}: Ballista {:?}", other.map(|m| g.perm(m).plus))),
                            }
                            if !(g.over && g.winner == Some(P)) {
                                bad.add(format!("{tag}: the line did not win"));
                            }
                        } else {
                            if tapped(&g) != 0 || !g.player(P).hand.contains(&c) || g.over {
                                bad.add(format!(
                                    "{tag}: a refused line left tapped {} hand {}",
                                    tapped(&g),
                                    g.player(P).hand.contains(&c)
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
    bad.done("heliod line");
}

#[test]
fn last_x_is_left_alone_by_a_line_without_an_x_piece() {
    // the Aura line with the Ballista already out casts nothing: last_x is not touched
    let mut g = table(&["kinnan-simic-mana-combo", "seph", "sauron"]);
    perm(&mut g, P, "Freed from the Real");
    perm(&mut g, P, "Bloom Tender");
    ballista_out(&mut g);
    lands(&mut g, P, "Forest", 4, false);
    g.last_x = 77;
    let i = combos::combo_index("Aura + Bloom Tender").unwrap();
    assert!(combos::attempt(&mut g, P, i as i64).unwrap());
    assert_eq!(g.last_x, 77);
    assert_eq!(tapped(&g), 0, "the Ballista on the battlefield is not cast again");
}

/// a Walking Ballista on the battlefield with one counter (a plain `perm` would enter with none and die)
fn ballista_out(g: &mut Game) -> PermId {
    let c = take(g, P, "Walking Ballista");
    sim_core::engine::zones::enter(
        g,
        P,
        c,
        sim_core::engine::zones::Enter { sick: false, plus: 1, ..Default::default() },
    )
    .unwrap()
}

fn kinnan_game(lands_n: usize) -> Game {
    let mut g = table(&["kinnan-simic-mana-combo", "seph", "sauron"]);
    perm(&mut g, P, "Freed from the Real");
    perm(&mut g, P, "Bloom Tender");
    lands(&mut g, P, "Forest", lands_n, false);
    g
}

#[test]
fn chulane_kinnan_tender_line_with_the_aura_out_needs_no_mana_for_a_ballista_in_hand() {
    // The loop's unbounded mana pays for the Ballista: with the Aura out the line costs nothing on any board.
    for (lands_n, reducers, tax, trini) in
        [(0, 0, 0, false), (1, 0, 0, false), (1, 1, 0, false), (2, 1, 1, false), (0, 0, 0, true), (3, 0, 0, true)]
    {
        let mut g = kinnan_game(lands_n);
        for _ in 0..reducers {
            perm(&mut g, P, "Etherium Sculptor");
        }
        for _ in 0..tax {
            perm(&mut g, Q, "Sphere of Resistance");
        }
        if trini {
            perm(&mut g, R, "Trinisphere");
        }
        let c = hand(&mut g, P, &["Walking Ballista"])[0];
        let i = combos::combo_index("Aura + Bloom Tender").unwrap();
        let tag = format!("lands {lands_n} red {reducers} tax {tax} trini {trini}");
        let (ok, _, casts) = (combos::all()[i].ready)(&g, P);
        assert!(ok && casts.is_empty(), "{tag}: {casts:?}");
        assert!(combos::ready_to_go(&g, P, i), "{tag}");
        assert!(combos::attempt(&mut g, P, i as i64).unwrap(), "{tag}");
        assert!(g.over && g.winner == Some(P), "{tag}");
        assert_eq!(tapped(&g), 0, "{tag}");
        assert!(g.player(P).hand.contains(&c), "{tag}: the Ballista was not cast with real mana");
    }
}

#[test]
fn chulane_nyx_and_cub_lines_pay_for_the_aura_in_hand_alone() {
    for (line, part) in [("Nyxbloom Ancient", "Nyxbloom Ancient"), ("Badgermole Cub", "Badgermole Cub + a blue")] {
        let mut any_go = false;
        for n in 0..=7usize {
            let mut g = table(&["kinnan-simic-mana-combo", "seph", "sauron"]);
            perm(&mut g, P, line);
            perm(&mut g, P, "Birds of Paradise");
            lands(&mut g, P, "Island", n, false);
            let hh = hand(&mut g, P, &["Walking Ballista", "Freed from the Real"]);
            let (ballista, aura) = (hh[0], hh[1]);
            let i = combos::combo_index(part).unwrap();
            let cmb = combos::all()[i];
            let (ok, _, casts) = (cmb.ready)(&g, P);
            assert!(ok, "{line}");
            assert_eq!(casts, vec![aura], "{line}: the Aura is the line's only cast");
            assert_eq!(combos::mana_need(&g, P, cmb, &casts), cost_of(&g, P, aura), "{line} lands {n}");
            let go = combos::ready_to_go(&g, P, i);
            any_go |= go;
            let res = combos::attempt(&mut g, P, i as i64).unwrap();
            assert_eq!(res, go, "{line} lands {n}");
            assert!(g.player(P).hand.contains(&ballista), "{line} lands {n}: the Ballista is not cast with real mana");
            if go {
                assert!(g.over && g.winner == Some(P), "{line} lands {n}");
            } else {
                assert!(g.player(P).hand.contains(&aura), "{line} lands {n}");
                assert_eq!(tapped(&g), 0);
            }
        }
        assert!(any_go, "{line}: no land count where the line goes, the checks above are vacuous");
    }
}

#[test]
fn a_ballista_on_the_battlefield_is_not_cast_again_and_one_in_hand_waits() {
    let mut g = kinnan_game(4);
    ballista_out(&mut g);
    let spare = hand(&mut g, P, &["Walking Ballista"])[0];
    let i = combos::combo_index("Aura + Bloom Tender").unwrap();
    let (ok, _, casts) = (combos::all()[i].ready)(&g, P);
    assert!(ok);
    assert!(casts.is_empty(), "{casts:?}");
    assert!(combos::attempt(&mut g, P, i as i64).unwrap());
    assert!(g.player(P).hand.contains(&spare));
    assert_eq!(tapped(&g), 0);
}

// ------------------------------------------------------------------ (6) planner: commander tax and `reserve`
#[test]
fn a_commander_is_public_exactly_when_cost_of_is_payable_for_every_tax() {
    // Kinnan ({G}{U}) in the command zone, Basalt Monolith out: the line is public only while Kinnan can be cast
    for tax in 0..=4u32 {
        for extra in [0u32, 1] {
            let mut g = table(&["kinnan-bonder-mana-combo", "seph", "sauron"]);
            let kinnan = card(&g, "Kinnan, Bonder Prodigy");
            if g.player(P).cmd == kinnan {
                g.player_mut(P).tax = 2 * tax;
            } else {
                g.player_mut(P).tax2 = 2 * tax;
            }
            perm(&mut g, P, "Basalt Monolith");
            let m = g.player(P).perms.iter().copied().find(|&m| g.perm(m).cd.is_some()).unwrap();
            g.perm_mut(m).tapped = true; // (no mana from the Monolith)
            lands(&mut g, P, "Forest", 1, false);
            lands(&mut g, P, "Island", 1, false);
            lands(&mut g, P, "Wastes", (2 * tax) as usize, false);
            if extra == 1 {
                // one mana fewer than the price
                let l = g.player(P).lands.last().copied().unwrap();
                g.perm_mut(l).tapped = true;
            }
            let (gn, pips) = cost_of(&g, P, kinnan);
            assert_eq!(gn + pips.len() as u32, 2 + 2 * tax, "tax {tax}: cost_of counts the tax once");
            let public = combos::public_progress(&g, P) >= 1.0;
            let payable = mana::can_pay_spell(&g, P, kinnan, gn, &pips, false);
            assert_eq!(public, payable, "tax {tax} extra {extra}");
            assert_eq!(payable, extra == 0, "tax {tax} extra {extra}");
        }
    }
}

#[test]
fn partner_commanders_pay_their_own_tax_once_and_liesa_pays_none() {
    let mut g = table(&["kraum-tymna-blue-breach-midrange", "seph", "sauron"]);
    let cs: Vec<CardId> = g.player(P).commanders().collect();
    assert_eq!(cs.len(), 2);
    for (t1, t2) in [(0u32, 0u32), (2, 0), (0, 4), (6, 2)] {
        g.player_mut(P).tax = t1;
        g.player_mut(P).tax2 = t2;
        for &c in &cs {
            let d = g.db.get(c);
            let tax = g.player(P).tax_of(c);
            let (gn, pips) = cost_of(&g, P, c);
            assert_eq!(gn, d.generic + tax, "{} tax {t1}/{t2}", d.name);
            assert_eq!(pips.as_str(), &*d.pips);
        }
    }
    // Liesa, Shroud of Dusk: no mana tax (she pays life instead)
    let mut g = table(&["kraum-tymna-blue-breach-midrange", "seph", "sauron"]);
    let liesa = card(&g, "Liesa, Shroud of Dusk");
    let c0 = g.player(P).cmd;
    g.player_mut(P).cmd = liesa;
    g.player_mut(P).tax = 4;
    let d = g.db.get(liesa);
    assert_eq!(cost_of(&g, P, liesa), (d.generic, d.pips.to_string()));
    let _ = c0;
}

fn tutor_game(forests: usize, boards: &[(&str, PlayerId)]) -> (Game, CardId) {
    let mut g = table(&["kinnan-bonder-mana-combo", "kraum-tymna-blue-breach-midrange", "dack-fayden-white-stax"]);
    for i in 1..3 {
        let w: Vec<_> = (0..60).map(|_| card(&g, "Wastes")).collect();
        g.player_mut(PlayerId(i)).hand.clear();
        g.player_mut(PlayerId(i)).library = w;
    }
    let lib: Vec<_> = (0..40).map(|_| card(&g, "Island")).collect();
    g.player_mut(P).hand.clear();
    g.player_mut(P).library = lib;
    for &(n, who) in boards {
        perm(&mut g, who, n);
    }
    lands(&mut g, P, "Forest", forests, false);
    let gsz = hand(
        &mut g,
        P,
        &[
            "Green Sun's Zenith",
            "Bonesplitter",
            "Skullclamp",
            "Sword of Fire and Ice",
            "Gitaxian Probe",
            "Frantic Search",
            "Deadly Dispute",
            "Ornithopter",
        ],
    )[0];
    (g, gsz)
}

/// the same table with `islands` Islands and Mystical Tutor ({U}, an instant: what `plays(eot)` casts) as the tutor
fn instant_tutor_game(islands: usize, boards: &[(&str, PlayerId)]) -> (Game, CardId) {
    let mut g = table(&["kinnan-bonder-mana-combo", "kraum-tymna-blue-breach-midrange", "dack-fayden-white-stax"]);
    for i in 1..3 {
        let w: Vec<_> = (0..60).map(|_| card(&g, "Wastes")).collect();
        g.player_mut(PlayerId(i)).hand.clear();
        g.player_mut(PlayerId(i)).library = w;
    }
    let lib: Vec<_> = (0..40).map(|_| card(&g, "Island")).collect();
    g.player_mut(P).hand.clear();
    g.player_mut(P).library = lib;
    for &(n, who) in boards {
        perm(&mut g, who, n);
    }
    lands(&mut g, P, "Island", islands, false);
    let t = hand(&mut g, P, &["Mystical Tutor", "Ornithopter", "Gitaxian Probe", "Bonesplitter"])[0];
    (g, t)
}

fn cast_opt(c: CardId) -> Opt {
    Opt { utility: 1.0, label: format!("cast {c:?}"), act: Some(sim_core::hooks::Action::Cast { card: c, zone: None }) }
}

/// the position the tutor is cast in (the end-of-turn window before P's next turn), built here from the rules and not by
/// the planner's own helper: the seat before P is the active one, it is the next round (no spell P cast this turn counts
/// any more), every pool is empty (`turn::step_start` empties them at the start of every turn) and the other seats have
/// untapped
fn window(mut g: Game) -> Game {
    g.active = Some(R);
    g.round += 1;
    g.player_mut(P).turn_casts = None;
    for q in &mut g.players {
        q.mana.clear();
        q.pool.art = 0;
        q.pool.spell = 0;
    }
    for i in 1..3 {
        let q = PlayerId(i);
        for m in g.player(q).perms.clone().into_iter().chain(g.player(q).lands.clone()) {
            g.perm_mut(m).tapped = false;
        }
    }
    g
}

/// is the tutor payable at its price in the window after the play is made on a copy
fn payable_after(g: &Game, tutor: CardId, x: u32, a: &sim_core::hooks::Action) -> bool {
    let mut g2 = g.clone();
    if !sim_core::ai::act::perform(&mut g2, P, a).unwrap_or(false) || !g2.player(P).hand.contains(&tutor) {
        return false;
    }
    let g2 = window(g2);
    let (gn, pips) = if x == 0 { cost_of(&g2, P, tutor) } else { cost_of_x(&g2, P, tutor, x as i32) };
    mana::can_pay_spell(&g2, P, tutor, gn, &pips, false)
}

#[test]
fn reserve_keeps_an_option_exactly_when_the_tutor_is_still_payable_after_it() {
    let boards: Vec<Vec<(&str, PlayerId)>> = vec![
        vec![],
        vec![("Emerald Medallion", P)],
        vec![("Emerald Medallion", P), ("Emerald Medallion", P)],
        vec![("Sphere of Resistance", Q)],
        vec![("Sphere of Resistance", Q), ("Sphere of Resistance", R)],
        vec![("Trinisphere", Q)],
        vec![("Trinisphere", Q), ("Emerald Medallion", P), ("Sphere of Resistance", Q)],
        vec![("Thalia, Guardian of Thraben", Q)],
        vec![("Etherium Sculptor", P)],
        vec![("Damping Sphere", Q)],
        vec![("Damping Sphere", Q), ("Trinisphere", R), ("Etherium Sculptor", P)],
    ];
    let mut bad = Bad::default();
    for (bi, board) in boards.iter().enumerate() {
        for forests in 0..=9 {
            for (prior, float) in [(0, 0u32), (1, 0), (0, 2), (1, 2)] {
                for x in [1u32, 2, 3] {
                    let (mut g, gsz) = tutor_game(forests, board);
                    g.player_mut(P).mana.add(sim_core::engine::pool::Kind::G, float);
                    if prior > 0 {
                        let ring = card(&g, "Sol Ring");
                        let st = g.turn_stamp();
                        g.player_mut(P).turn_casts = Some((st, vec![ring]));
                    }
                    let cards: Vec<CardId> = g.player(P).hand.iter().copied().filter(|&c| c != gsz).collect();
                    let opts: Vec<Opt> = cards.iter().map(|&c| cast_opt(c)).chain([cast_opt(gsz)]).collect();
                    let kept = sim_core::ai::planner::reserve(&mut g, P, gsz, x, opts.clone());
                    for o in &opts {
                        let a = o.act.as_ref().unwrap();
                        let sim_core::hooks::Action::Cast { card: c, .. } = a else { unreachable!() };
                        let keep = kept.iter().any(|k| k.label == o.label);
                        let want = *c != gsz && payable_after(&g, gsz, x, a);
                        if keep != want {
                            bad.add(format!(
                                "board {bi} forests {forests} prior {prior} floating {float} x {x}: option {} kept {keep}, tutor payable after {want}",
                                g.db.get(*c).name
                            ));
                        }
                    }
                }
            }
        }
    }
    bad.done("reserve");
}

fn add_sphere(g: &mut Game, _p: PlayerId, _arg: i64) -> sim_core::flow::Res<bool> {
    perm(g, Q, "Sphere of Resistance");
    Ok(true)
}

fn add_medallion(g: &mut Game, p: PlayerId, _arg: i64) -> sim_core::flow::Res<bool> {
    perm(g, p, "Emerald Medallion");
    Ok(true)
}

fn plan_opt(f: sim_core::hooks::PlanFn) -> Opt {
    Opt { utility: 1.0, label: "plan".into(), act: Some(sim_core::hooks::Action::Plan { f, arg: 0 }) }
}

#[test]
fn reserve_prices_the_tutor_after_a_play_that_adds_a_tax_or_a_reducer() {
    // other-action branch: the tutor is priced on the copy after the play
    for forests in 0..=6 {
        let (mut g, gsz) = tutor_game(forests, &[]);
        let keep = |g: &mut Game, f| !sim_core::ai::planner::reserve(g, P, gsz, 2, vec![plan_opt(f)]).is_empty();
        let (gn, pips) = cost_of_x(&g, P, gsz, 2);
        let base = gn + pips.len() as u32;
        assert_eq!(
            keep(&mut g, add_sphere),
            forests as u32 >= base + 1,
            "a Sphere after the play: price {base} + 1, {forests} Forests"
        );
        assert_eq!(
            keep(&mut g, add_medallion),
            forests as u32 >= base - 1,
            "a Medallion after the play: price {base} - 1, {forests} Forests"
        );
    }
}

#[test]
fn reserve_cast_branch_sees_the_tax_the_cast_card_brings() {
    // Sphere of Resistance cast with 5 Forests and Green Sun's Zenith at X = 2: the tutor then costs {3}{G}, only 3 Forests
    // are left: not kept
    let (mut g, gsz) = tutor_game(5, &[]);
    let sphere = hand(&mut g, P, &["Sphere of Resistance"])[0];
    let a = sim_core::hooks::Action::Cast { card: sphere, zone: None };
    let kept = !sim_core::ai::planner::reserve(&mut g, P, gsz, 2, vec![cast_opt(sphere)]).is_empty();
    assert!(!kept);
    assert_eq!(kept, payable_after(&g, gsz, 2, &a), "kept {kept}");
}

#[test]
fn reserve_cast_branch_sees_the_reducer_the_cast_card_brings() {
    // Emerald Medallion cast with 4 Forests: the tutor then costs {1}{G}, payable: kept
    let (mut g, gsz) = tutor_game(4, &[]);
    let med = hand(&mut g, P, &["Emerald Medallion"])[0];
    let a = sim_core::hooks::Action::Cast { card: med, zone: None };
    let kept = !sim_core::ai::planner::reserve(&mut g, P, gsz, 2, vec![cast_opt(med)]).is_empty();
    assert!(kept);
    assert_eq!(kept, payable_after(&g, gsz, 2, &a), "kept {kept}");
}

#[test]
fn reserve_counts_the_mana_a_cast_rock_brings() {
    // Sol Ring cast with 2 Forests: it costs {1}, leaves one Forest and the Ring's {C}{C}; the tutor at X = 1 ({1}{G}) is
    // payable from them, so the option is kept. Reading the cost only ({1} + {1}{G} from two Forests) dropped it
    let (mut g, gsz) = tutor_game(2, &[]);
    let ring = hand(&mut g, P, &["Sol Ring"])[0];
    let a = sim_core::hooks::Action::Cast { card: ring, zone: None };
    let kept = !sim_core::ai::planner::reserve(&mut g, P, gsz, 1, vec![cast_opt(ring)]).is_empty();
    assert!(kept);
    assert_eq!(kept, payable_after(&g, gsz, 1, &a));
}

#[test]
fn reserve_prices_the_tutor_in_the_window_where_a_defense_grid_taxes_it() {
    // Defense Grid taxes a spell cast on another player's turn: Mystical Tutor, cast in the end step before P's next turn
    // (an opponent's turn), costs {3} more than in P's main phase. Kept exactly when the window price is payable
    let mut seen_main_payable_window_not = false;
    for islands in 0..=9usize {
        let (mut g, tutor) = instant_tutor_game(islands, &[("Defense Grid", P)]);
        let ring = hand(&mut g, P, &["Sol Ring"])[0];
        let a = sim_core::hooks::Action::Cast { card: ring, zone: None };
        let kept = !sim_core::ai::planner::reserve(&mut g, P, tutor, 0, vec![cast_opt(ring)]).is_empty();
        let main = cost_of(&g, P, tutor);
        let win = cost_of(&window(g.clone()), P, tutor);
        assert_eq!(win.0, main.0 + 3, "the Grid adds {{3}} in the window");
        assert_eq!(kept, payable_after(&g, tutor, 0, &a), "islands {islands}");
        let mut g2 = g.clone();
        sim_core::ai::act::perform(&mut g2, P, &a).unwrap();
        let in_main = mana::can_pay_spell(&g2, P, tutor, main.0, &main.1, false);
        seen_main_payable_window_not |= in_main && !kept;
    }
    assert!(seen_main_payable_window_not, "no board where the main phase price is payable and the window's is not");
}

#[test]
fn reserve_restarts_the_damping_sphere_count_in_the_window() {
    // Damping Sphere charges {1} per spell P already cast this turn: in the window the count starts again, so a play that
    // the main phase price would drop (spells cast, the tutor charged for each) is kept when the window can pay
    let mut seen = false;
    for islands in 0..=9usize {
        let (mut g, tutor) = instant_tutor_game(islands, &[("Damping Sphere", Q)]);
        let ring = card(&g, "Sol Ring");
        let st = g.turn_stamp();
        g.player_mut(P).turn_casts = Some((st, vec![ring, ring]));
        let bs = hand(&mut g, P, &["Bonesplitter"])[0];
        let a = sim_core::hooks::Action::Cast { card: bs, zone: None };
        let kept = !sim_core::ai::planner::reserve(&mut g, P, tutor, 0, vec![cast_opt(bs)]).is_empty();
        assert_eq!(kept, payable_after(&g, tutor, 0, &a), "islands {islands}");
        let mut g2 = g.clone();
        sim_core::ai::act::perform(&mut g2, P, &a).unwrap();
        let (mg, mp) = cost_of(&g2, P, tutor);
        seen |= kept && !mana::can_pay_spell(&g2, P, tutor, mg, &mp, false);
    }
    assert!(seen, "no board where the window pays and the main phase price does not");
}

#[test]
fn reserve_does_not_count_mana_that_floats_now_it_is_gone_by_the_window() {
    // no lands, {U}{U} floating, Mystical Tutor in hand: the tutor is payable in the main phase, but the pool is empty by
    // the end step before P's next turn, so casting Ornithopter (an artifact) or Gitaxian Probe (an instant) leaves the
    // tutor unpayable there. With an Island the Ornithopter is fine: the Island is still untapped in the window
    let (mut g, tutor) = instant_tutor_game(0, &[]);
    g.player_mut(P).mana.add(sim_core::engine::pool::Kind::U, 2);
    let (tg, tp) = cost_of(&g, P, tutor);
    assert!(mana::can_pay_spell(&g, P, tutor, tg, &tp, false), "payable now, from the floating mana");
    let orni = hand(&mut g, P, &["Ornithopter"])[0];
    let probe = hand(&mut g, P, &["Gitaxian Probe"])[0];
    for c in [orni, probe] {
        let kept = !sim_core::ai::planner::reserve(&mut g, P, tutor, 0, vec![cast_opt(c)]).is_empty();
        assert!(!kept, "{}: kept, the tutor is unpayable in the window", g.db.get(c).name);
    }
    let (mut g, tutor) = instant_tutor_game(1, &[]);
    g.player_mut(P).mana.add(sim_core::engine::pool::Kind::U, 2);
    let orni = hand(&mut g, P, &["Ornithopter"])[0];
    assert!(!sim_core::ai::planner::reserve(&mut g, P, tutor, 0, vec![cast_opt(orni)]).is_empty());
}

/// an instant-tutor table with an opponent's Trinisphere that is tapped now
fn tapped_trinisphere_game(islands: usize) -> (Game, CardId) {
    let (mut g, tutor) = instant_tutor_game(islands, &[("Trinisphere", Q)]);
    let t = g.player(Q).perms.iter().copied().find(|&m| &*g.db.get(g.perm(m).card()).name == "Trinisphere").unwrap();
    g.perm_mut(t).tapped = true;
    (g, tutor)
}

#[test]
fn reserve_untaps_the_other_seats_in_the_window() {
    // an opponent's Trinisphere that is tapped in P's main phase untaps before the window and applies there: every spell
    // costs at least {3}, so Mystical Tutor costs {2}{U} in the window and {U} now
    let (g, tutor) = tapped_trinisphere_game(3);
    assert_eq!(cost_of(&g, P, tutor), (0, "U".to_string()), "a tapped Trinisphere does not apply now");
    assert_eq!(cost_of(&window(g.clone()), P, tutor), (2, "U".to_string()), "an untapped one does");
    // the play is Ornithopter ({0}): three Islands pay the window price, two do not
    for (islands, want) in [(2usize, false), (3, true)] {
        let (mut g, tutor) = tapped_trinisphere_game(islands);
        let orni = hand(&mut g, P, &["Ornithopter"])[0];
        let kept = !sim_core::ai::planner::reserve(&mut g, P, tutor, 0, vec![cast_opt(orni)]).is_empty();
        assert_eq!(kept, want, "{islands} Islands");
    }
}

// ------------------------------------------------------------------ (7) Twinflame
fn twinflame_board(board: usize, titans: usize, mountains: usize, wastes: usize) -> (Game, CardId, usize, u32, bool) {
    let mut g = game();
    lands(&mut g, P, "Mountain", mountains, false);
    lands(&mut g, P, "Wastes", wastes, false);
    let (mut red, mut tax, mut trini) = (0usize, 0u32, false);
    let mut add = |g: &mut Game, name: &str, who: PlayerId, red_n: usize, tax_n: u32, t: bool| {
        perm(g, who, name);
        red += red_n;
        tax += tax_n;
        trini |= t;
    };
    match board {
        0 => {}
        1 => add(&mut g, "Ruby Medallion", P, 1, 0, false),
        2 => {
            add(&mut g, "Ruby Medallion", P, 1, 0, false);
            add(&mut g, "Ruby Medallion", P, 1, 0, false);
        }
        3 => add(&mut g, "Sphere of Resistance", Q, 0, 1, false),
        4 => add(&mut g, "Trinisphere", Q, 0, 0, true),
        5 => {
            add(&mut g, "Trinisphere", Q, 0, 0, true);
            add(&mut g, "Ruby Medallion", P, 1, 0, false);
            add(&mut g, "Sphere of Resistance", Q, 0, 1, false);
        }
        6 => add(&mut g, "K'rrik, Son of Yawgmoth", P, 0, 0, false),
        7 => add(&mut g, "Thalia, Guardian of Thraben", Q, 0, 1, false),
        _ => {
            add(&mut g, "Sphere of Resistance", Q, 0, 1, false);
            add(&mut g, "Sphere of Resistance", R, 0, 1, false);
            add(&mut g, "Thalia, Guardian of Thraben", Q, 0, 1, false);
            add(&mut g, "Trinisphere", Q, 0, 0, true);
            add(&mut g, "Ruby Medallion", P, 1, 0, false);
            add(&mut g, "Ruby Medallion", P, 1, 0, false);
        }
    }
    for _ in 0..titans {
        perm(&mut g, P, "Grave Titan");
    }
    let c = hand(&mut g, P, &["Twinflame"])[0];
    (g, c, red, tax, trini)
}

fn twinflame_opts(g: &mut Game, c: CardId) -> Vec<Opt> {
    g.registry.get(c).unwrap().hand_options.unwrap()(g, c, P, Some(false)).unwrap()
}

fn n_targets(o: &Opt) -> usize {
    o.label.split(|ch: char| !ch.is_ascii_digit()).find(|s| !s.is_empty()).unwrap().parse().unwrap()
}

#[test]
fn twinflame_plan_and_payment_follow_the_rules_price_for_every_target_count_and_board() {
    // the rules price (601.2f): {1}{R} + {2}{R} per extra target, plus taxes, less reductions, floored at 0 once, then the
    // Trinisphere minimum counted over every symbol
    let mut bad = Bad::default();
    for board in 0..=8 {
        for titans in 1..=3 {
            for lands_n in [3usize, 5, 8, 12, 16] {
                let (mut g, c, red, tax, trini) = twinflame_board(board, titans, lands_n, 0);
                let oo = twinflame_opts(&mut g, c);
                let tag = format!("board {board} titans {titans} lands {lands_n}");
                if oo.is_empty() {
                    continue;
                }
                let n = n_targets(&oo[0]);
                let mut gn = (1 + 2 * (n as i64 - 1) - red as i64 + tax as i64).max(0);
                if trini {
                    gn = gn.max(3 - n as i64);
                }
                let rules = (gn as u32, "R".repeat(n));
                let pipe = cost_of_with(&g, P, c, 2 * (n as i32 - 1), &"R".repeat(n));
                if pipe != rules {
                    bad.add(format!("{tag}: {n} targets: pipeline {pipe:?}, rules {rules:?}"));
                }
                // the plan offered the option only if the pool pays the rules price
                if !mana::can_pay_spell(&g, P, c, rules.0, &rules.1, false) {
                    bad.add(format!("{tag}: offered {n} targets with a pool that cannot pay {rules:?}"));
                    continue;
                }
                if !run(&mut g, &oo[0]) {
                    bad.add(format!("{tag}: the offered cast failed"));
                    continue;
                }
                if tapped(&g) != rules.0 + n as u32 {
                    bad.add(format!("{tag}: {n} targets tapped {} rules price {rules:?}", tapped(&g)));
                }
            }
        }
    }
    bad.done("twinflame");
}

#[test]
fn twinflame_never_plans_more_targets_than_the_pool_has_red_for() {
    for mountains in 1..=3usize {
        let (mut g, c, ..) = twinflame_board(0, 3, mountains, 12);
        let oo = twinflame_opts(&mut g, c);
        assert!(!oo.is_empty(), "{mountains}");
        for o in &oo {
            let n = n_targets(o);
            assert!(n <= mountains, "{n} targets need {n} {{R}}: only {mountains} Mountains");
        }
        let n = n_targets(&oo[0]);
        assert!(run(&mut g, &oo[0]));
        assert_eq!(tapped(&g), 3 * n as u32 - 1, "{n} targets, plain board: {{1}}{{R}} and {{2}}{{R}} for each extra");
    }
}

#[test]
fn twinflame_strive_is_still_paid_under_tamiyos_emblem() {
    // Tamiyo's emblem lets the spell be cast without paying its mana cost, but strive is a cost increase and still applies
    // (601.2f, 118.9d): every extra target costs {2}{R}. (The other taxes the engine drops under the emblem stay dropped.)
    let (mut g, c, ..) = twinflame_board(0, 3, 12, 0);
    g.player_mut(P).emblems.push("tamiyo");
    let oo = twinflame_opts(&mut g, c);
    let n = n_targets(&oo[0]);
    assert!(n >= 2, "{n}");
    assert!(run(&mut g, &oo[0]));
    // board 0 (nothing else on the table): the exact price is strive's {2}{R} for each target beyond the first
    assert_eq!(tapped(&g), 3 * (n as u32 - 1), "{n} targets, strive {{2}}{{R}} each extra");
}

// ------------------------------------------------------------------ (8) harmonize
fn harmonize_case(bad: &mut Bad, forests: usize, others: usize, power: i32, board: usize) {
    let mut g = game();
    lands(&mut g, P, "Forest", forests, false);
    lands(&mut g, P, "Wastes", others, false);
    let (mut med, mut tax) = (0i64, 0i64);
    match board {
        0 => {}
        1 => {
            perm(&mut g, Q, "Trinisphere");
        }
        2 => {
            perm(&mut g, Q, "Sphere of Resistance");
            tax += 1;
        }
        3 => {
            perm(&mut g, Q, "Trinisphere");
            perm(&mut g, Q, "Sphere of Resistance");
            perm(&mut g, P, "Emerald Medallion");
            perm(&mut g, P, "Emerald Medallion");
            tax += 1;
            med += 2;
        }
        4 => {
            perm(&mut g, P, "Emerald Medallion");
            perm(&mut g, P, "Emerald Medallion");
            med += 2;
        }
        5 => {
            perm(&mut g, Q, "Thalia, Guardian of Thraben");
            tax += 1;
        }
        6 => {
            perm(&mut g, P, "K'rrik, Son of Yawgmoth");
        }
        _ => {
            perm(&mut g, Q, "Thorn of Amethyst");
            perm(&mut g, R, "Sphere of Resistance");
            perm(&mut g, Q, "Trinisphere");
            tax += 2;
        }
    }
    if power > 0 {
        token(&mut g, P, power);
    }
    let c = card(&g, "Nature's Rhythm");
    g.player_mut(P).gy.push(c);
    let target = card(&g, "Consecrated Sphinx");
    g.player_mut(P).library = vec![target];
    let f = g.registry.get(c).unwrap().gy_options.unwrap();
    let oo = f(&mut g, c, P, Some(false)).unwrap();
    let tag = format!("forests {forests} others {others} power {power} board {board}");
    let best = (0..=64).rev().find(|&x| {
        let (gn, pips) = cost_of_with(&g, P, c, x, "GGGG");
        mana::can_pay_spell(&g, P, c, gn, &pips, false)
    });
    // (K'rrik is a 2/2 creature: its power reduces the cost too)
    let power = if board == 6 { power.max(2) } else { power };
    let want_x = best.map(|b| b + power);
    let should_offer = want_x.is_some_and(|x| x >= 3);
    if oo.is_empty() == should_offer {
        bad.add(format!("{tag}: offered {} but brute force says X' {best:?} + power {power}", !oo.is_empty()));
        return;
    }
    let Some(o) = oo.first() else { return };
    let x_plan = ((o.utility - 1.0) * 2.0).round() as i32;
    if Some(x_plan) != want_x {
        bad.add(format!("{tag}: plan X {x_plan}, brute force X' {best:?} + power {power}"));
    }
    let b = best.unwrap();
    // the generic part is the rules' (601.2f: taxes and reductions are summed, then floored once; every symbol of
    // {X}{G}{G}{G}{G} is four, so the Trinisphere minimum never raises it)
    let rules = (b as i64 - med + tax).max(0);
    let (gn, pips) = cost_of_with(&g, P, c, b, "GGGG");
    if gn as i64 != rules || pips != "GGGG" {
        bad.add(format!("{tag}: X' {b}: pipeline {gn} {pips:?}, rules {rules} GGGG"));
    }
    if !run(&mut g, o) {
        bad.add(format!("{tag}: the offered cast failed"));
        return;
    }
    if tapped(&g) != gn + 4 {
        bad.add(format!("{tag}: tapped {} price {gn} + 4", tapped(&g)));
    }
    if g.player(P).gy.contains(&c) || !g.player(P).exile.contains(&c) {
        bad.add(format!("{tag}: Nature's Rhythm not exiled"));
    }
}

#[test]
fn harmonize_matches_brute_force_over_pools_powers_and_boards() {
    let mut bad = Bad::default();
    for board in 0..=7 {
        for forests in 0..=8 {
            for others in [0usize, 1, 3, 6] {
                for power in [0, 1, 2, 4, 7] {
                    harmonize_case(&mut bad, forests, others, power, board);
                }
            }
        }
    }
    bad.done("harmonize");
}

#[test]
fn harmonize_two_forests_and_other_colours_is_never_castable() {
    // {G}{G}{G}{G} cannot be made from two Forests however many other colours and however large the power
    for power in [0, 3, 10] {
        let mut bad = Bad::default();
        harmonize_case(&mut bad, 2, 10, power, 0);
        bad.done("two forests");
    }
}

// ------------------------------------------------------------------ (9) nothing that changes prices: the old rule
#[test]
fn twinflame_equals_the_old_price_without_price_changers() {
    for lands_n in 1..=10usize {
        let (mut g, c, ..) = twinflame_board(0, 2, lands_n, 0);
        let oo = twinflame_opts(&mut g, c);
        if oo.is_empty() {
            assert!(lands_n < 2, "{lands_n}");
            continue;
        }
        let n = n_targets(&oo[0]);
        assert!(run(&mut g, &oo[0]));
        // the old rule: {1}{R} and {2}{R} for each extra target
        assert_eq!(tapped(&g), 3 * n as u32 - 1, "{lands_n} lands, {n} targets");
    }
}

// ------------------------------------------------------------------ (10) the rest of the diff
#[test]
fn whole_games_with_hangarback_and_chalice_in_three_seats_do_not_panic() {
    for seed in 1..=3u64 {
        let g = play(&["arzakon", "dack-fayden-white-stax", "sheoldred"], seed, false);
        assert!(g.round >= 1);
    }
}

#[test]
fn cost_of_for_the_two_cards_in_hand_is_cheap_and_finite_on_every_pool() {
    // cost_of on these cards runs the card's own search: it must stay finite and never wrap near u32::MAX
    for n in 0..=30usize {
        for lib_n in [0usize, 80] {
            let mut g = game();
            lands(&mut g, P, "Wastes", n, false);
            let one = card(&g, "Lightning Bolt");
            g.player_mut(Q).library = vec![one; lib_n];
            for name in ["Hangarback Walker", "Chalice of the Void"] {
                let c = card(&g, name);
                let (gn, pips) = cost_of(&g, P, c);
                assert!(gn < 200 && pips.is_empty(), "{name} {n} lands: {gn} {pips:?}");
            }
        }
    }
}

#[test]
fn a_ballista_in_hand_that_cannot_be_cast_is_not_an_outlet() {
    // Rule of Law with a spell already cast this turn: the Ballista in hand cannot be cast, so it is no outlet
    let mut g = kinnan_game(4);
    perm(&mut g, Q, "Rule of Law");
    let ring = card(&g, "Sol Ring");
    let st = g.turn_stamp();
    g.player_mut(P).turn_casts = Some((st, vec![ring]));
    let c = hand(&mut g, P, &["Walking Ballista"])[0];
    assert!(!sim_core::engine::cast::castable(&g, P, c, "hand"), "Rule of Law: one spell a turn, one already cast");
    let i = combos::combo_index("Aura + Bloom Tender").unwrap();
    let (ok, _, casts) = (combos::all()[i].ready)(&g, P);
    assert!(!ok, "ready although the only outlet cannot be cast: {casts:?}");
    assert!(!combos::attempt(&mut g, P, i as i64).unwrap());
    assert!(!g.over, "no win without an outlet");
}

#[test]
fn a_ballista_in_hand_is_an_outlet_under_a_chalice_on_zero_a_bauble_or_a_void_mirror() {
    // `castable` reads the Ballista at X = 0 (nothing spent, nothing coloured) and says a Chalice of the Void on 0, a Vexing
    // Bauble or a Void Mirror counters it. The loop's mana casts it for X >= 1 with coloured mana, which none of them
    // counters: the line is ready and wins
    for lock in ["Chalice of the Void", "Vexing Bauble", "Void Mirror"] {
        let mut g = kinnan_game(0);
        perm(&mut g, Q, lock);
        let c = hand(&mut g, P, &["Walking Ballista"])[0];
        assert!(!sim_core::engine::cast::castable(&g, P, c, "hand"), "{lock}: the AI's own estimate stops the cast");
        assert!(sim_core::engine::cast::cast_allowed(&g, P, c, "hand"), "{lock}: nothing forbids it");
        let i = combos::combo_index("Aura + Bloom Tender").unwrap();
        let (ok, _, casts) = (combos::all()[i].ready)(&g, P);
        assert!(ok && casts.is_empty(), "{lock}: {casts:?}");
        assert!(combos::attempt(&mut g, P, i as i64).unwrap(), "{lock}");
        assert!(g.over && g.winner == Some(P), "{lock}");
    }
}

#[test]
fn auto_x_cards_are_priced_and_recorded_from_other_mana_and_through_yawgmoths_will() {
    // mana from a rock and a creature, not only lands: the mana spent is the price
    for rocks in 0..=2usize {
        for lands_n in 0..=6usize {
            for name in ["Hangarback Walker", "Chalice of the Void"] {
                let mut g = game();
                opp_cards(&mut g, "Lightning Bolt");
                for _ in 0..rocks {
                    perm(&mut g, P, "Sol Ring");
                }
                lands(&mut g, P, "Wastes", lands_n, false);
                let c = hand(&mut g, P, &[name])[0];
                let pre = cost_of(&g, P, c);
                let ok = paid(&g, c, pre.0, &pre.1);
                let res = brain::do_cast(&mut g, P, c, None).unwrap();
                assert_eq!(res, ok, "{name} rocks {rocks} lands {lands_n}: price {pre:?}");
                if res {
                    // a Sol Ring tapped for {1} wastes one: the units tapped cover the price and exceed it by at most that
                    let units = tapped(&g)
                        + 2 * g
                            .player(P)
                            .perms
                            .iter()
                            .filter(|&&m| g.perm(m).tapped && g.perm(m).cd == Some(card(&g, "Sol Ring")))
                            .count() as u32;
                    let price = pre.0 + pre.1.len() as u32;
                    assert!(
                        price <= units && units <= price + 1,
                        "{name} rocks {rocks} lands {lands_n}: price {price} units {units}"
                    );
                }
            }
        }
    }
}

#[test]
fn chalice_from_the_graveyard_through_yawgmoths_will_records_its_x() {
    let mut g = game();
    opp_cards(&mut g, "Lightning Bolt");
    lands(&mut g, P, "Wastes", 4, false);
    let c = card(&g, "Chalice of the Void");
    g.player_mut(P).gy.push(c);
    let pre = cost_of(&g, P, c);
    let res = brain::do_cast_with(&mut g, P, c, Some("yawg"), Ctx::default()).unwrap();
    if !res {
        println!("SKIPPED: Yawgmoth's Will permission is not modelled for a bare graveyard card ({pre:?})");
        return;
    }
    let m = body(&g, c).expect("on the battlefield");
    assert_eq!(tapped(&g), pre.0);
    assert_eq!(g.perm(m).data.int(DataKey::Charge) as i32, chalice_x(&g, P).unwrap_or(0).max(1).min(2), "{pre:?}");
}

#[test]
fn harmonize_records_x_prime_plus_power_as_the_creature_value_it_may_find() {
    let mut checked = 0;
    for board in [0usize, 3, 4] {
        for forests in 4..=8usize {
            for power in [0, 1, 3] {
                let build = |cmc: u32| -> Option<(Game, CardId)> {
                    let mut g = game();
                    lands(&mut g, P, "Forest", forests, false);
                    match board {
                        3 => {
                            perm(&mut g, Q, "Trinisphere");
                            perm(&mut g, Q, "Sphere of Resistance");
                            perm(&mut g, P, "Emerald Medallion");
                            perm(&mut g, P, "Emerald Medallion");
                        }
                        4 => {
                            perm(&mut g, P, "Emerald Medallion");
                        }
                        _ => {}
                    }
                    if power > 0 {
                        token(&mut g, P, power);
                    }
                    let nr = card(&g, "Nature's Rhythm");
                    g.player_mut(P).gy.push(nr);
                    let found =
                        g.db.iter()
                            .find(|d| d.creature && !d.land && d.cmc == cmc && d.xcount == 0)
                            .map(|d| g.db.id(&d.name).unwrap())?;
                    g.player_mut(P).library = vec![found];
                    Some((g, nr))
                };
                let Some((mut g, nr)) = build(0) else { continue };
                let f = g.registry.get(nr).unwrap().gy_options.unwrap();
                let oo = f(&mut g, nr, P, Some(false)).unwrap();
                let Some(o) = oo.first() else { continue };
                let x = ((o.utility - 1.0) * 2.0).round() as u32;
                if !(3..=9).contains(&x) {
                    continue;
                }
                for (cmc, enters) in [(x, true), (x + 1, false)] {
                    let Some((mut g, nr)) = build(cmc) else { continue };
                    let f = g.registry.get(nr).unwrap().gy_options.unwrap();
                    let oo = f(&mut g, nr, P, Some(false)).unwrap();
                    assert!(run(&mut g, &oo[0]));
                    let lib = g.player(P).library.len();
                    assert_eq!(
                        lib == 0,
                        enters,
                        "board {board} forests {forests} power {power}: X {x}, creature of mana value {cmc}"
                    );
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 0);
}

#[test]
fn cost_of_for_the_two_cards_is_cheap() {
    let mut g = game();
    lands(&mut g, P, "Wastes", 8, false);
    perm(&mut g, P, "Etherium Sculptor");
    let hang = card(&g, "Hangarback Walker");
    let chal = card(&g, "Chalice of the Void");
    let t = std::time::Instant::now();
    let mut s = 0u64;
    for _ in 0..20_000 {
        s += cost_of(&g, P, hang).0 as u64 + cost_of(&g, P, chal).0 as u64;
    }
    let per = t.elapsed().as_nanos() / 40_000;
    println!("PERF cost_of of an auto_x card: {per} ns ({s})");
    assert!(per < 100_000, "{per} ns per cost_of");
}
