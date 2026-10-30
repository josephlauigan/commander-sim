//! --- phase4-eval: the learned evaluation's model file, mapping and ordering (`ai/learned.rs`)

use sim_core::ai::learned::{self, PER, RAW, Raw};
use sim_core::ai::search::WIN_FLOOR;

const MODEL: &str = include_str!("../../../../data/eval/model.toml");

/// a seat's numbers: life, board, power, hand, lands, threat (strength from `strength`'s weights)
fn seat(v: &mut [f32; RAW], k: usize, life: f32, board: f32, power: f32, hand: f32, lands: f32, threat: f32) {
    let o = k * PER;
    v[o + learned::ALIVE] = 1.0;
    v[o + learned::LIFE] = life;
    v[o + learned::BOARD] = board;
    v[o + learned::POWER] = power;
    v[o + learned::HAND] = hand;
    v[o + learned::LANDS] = lands;
    v[o + learned::THREAT] = threat;
    v[o + learned::CREATURES] = (power / 2.0).round();
    v[o + learned::DISTANCE] = 6.0;
    v[o + learned::HAND_WORTH] = 2.0 * hand;
    v[o + learned::STRENGTH] = 0.25 * life.clamp(0.0, 60.0) + board + 0.35 * power + 0.9 * hand + 0.6 * lands;
}

/// me (seat 0) with `mine`, three opponents with `theirs`, at own turn 6
fn position(mine: [f32; 6], theirs: [f32; 6]) -> Raw {
    let mut v = [0.0f32; RAW];
    seat(&mut v, 0, mine[0], mine[1], mine[2], mine[3], mine[4], mine[5]);
    for k in 1..4 {
        seat(&mut v, k, theirs[0], theirs[1], theirs[2], theirs[3], theirs[4], theirs[5]);
    }
    let s = v[learned::STRENGTH] as f64;
    let so = v[PER + learned::STRENGTH] as f64;
    let old = (s - 0.6 * so - 0.4 * so) as f32;
    Raw { v, seat: 0, nseats: 4, turns: 6, old }
}

fn value(r: &Raw) -> f64 {
    let m = learned::model();
    m.scale(m.logit(&learned::features(r).expect("opponents alive")))
}

#[test]
fn the_embedded_model_loads() {
    let m = learned::parse_model(MODEL).expect("data/eval/model.toml checks");
    assert_eq!(m.features, learned::names());
    assert!(m.meta.positions > 0, "the model is fitted");
}

#[test]
fn bad_model_files_are_refused() {
    let ok = MODEL.to_string();
    let bad = |from: &str, to: &str| ok.replacen(from, to, 1);
    let first_sd = ok.lines().find(|l| l.starts_with("sd = [")).expect("sd line").to_string();
    let cases = [
        (bad("\"me_life\"", "\"my_life\""), "features"),
        (bad("sd = [", "sd = [1.0, "), "numbers each"),
        (bad("bias = ", "bias = nan # "), "finite"),
        (bad(&first_sd, &format!("sd = [0.0,{}", &first_sd[first_sd.find(',').expect("comma") + 1..])), "positive"),
        (ok.replace("\nb = ", "\nb = -"), "slope"),
        (bad("[map]", "[mapping]"), "model file"),
    ];
    for (text, why) in cases {
        let e = learned::parse_model(&text).expect_err(why);
        assert!(e.contains(why), "expected {why:?} in {e:?}");
    }
}

#[test]
fn the_mapping_is_monotone_and_below_a_win() {
    let m = learned::model();
    let mut last = f64::NEG_INFINITY;
    for i in -400..=400 {
        let z = i as f64 / 10.0;
        let v = m.scale(z);
        assert!(v >= last, "monotone at z {z}");
        assert!(v < WIN_FLOOR && v > -100.0, "z {z} maps to {v}");
        last = v;
    }
    assert!(m.scale(40.0) > m.scale(-40.0));
}

#[test]
fn hand_built_positions_order() {
    let ahead = position([40.0, 25.0, 12.0, 5.0, 8.0, 15.0], [10.0, 3.0, 2.0, 1.0, 3.0, 2.0]);
    let even = position([30.0, 10.0, 6.0, 4.0, 5.0, 6.0], [30.0, 10.0, 6.0, 4.0, 5.0, 6.0]);
    let behind = position([10.0, 3.0, 2.0, 1.0, 3.0, 2.0], [40.0, 25.0, 12.0, 5.0, 8.0, 15.0]);
    let (a, e, b) = (value(&ahead), value(&even), value(&behind));
    assert!(a > e && e > b, "ahead {a:.2}, even {e:.2}, behind {b:.2}");
    // a win chance near a quarter (four even seats) maps near today's even position
    let m = learned::model();
    let p = 1.0 / (1.0 + (-m.logit(&learned::features(&even).expect("opps"))).exp());
    assert!((0.1..0.45).contains(&p), "even position: win chance {p:.3}");
}
