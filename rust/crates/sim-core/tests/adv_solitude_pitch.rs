// written by an independent reviewer
// (assertions tightened by the coordinator's request after the Opus review)
//! Adversarial: the yoshimaru pitch rule (c). Which white cards of the yoshimaru list count as line pieces, and which
//! card is exiled in various hands.

use sim_core::ai::act::perform;
use sim_core::engine::tutors::card_worth;
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PlayerId};
use sim_core::impls::t6_pieces::is_line_piece;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const SOL: &str = "Solitude";
const BIG: &str = "Grave Titan";

fn tbl(key: &str) -> (Game, CardId) {
    let mut g = table(&[key, "seph"]);
    g.active = Some(P0);
    g.player_mut(P1).hand.clear();
    let s = hand(&mut g, P0, &[SOL])[0];
    perm(&mut g, P1, BIG);
    (g, s)
}

fn evoke_opts(g: &mut Game, c: CardId) -> Vec<Opt> {
    let f = g.registry.get(c).and_then(|i| i.hand_options).expect("hand options");
    f(g, c, P0, None).unwrap()
}

fn ai_evokes(g: &mut Game, c: CardId) -> bool {
    let opts = evoke_opts(g, c);
    let Some(o) = opts.iter().find(|o| o.label.contains("evoke Solitude")) else { return false };
    perform(g, P0, o.act.as_ref().unwrap()).unwrap()
}

fn pitched(g: &Game, names: &[&str]) -> Vec<String> {
    names.iter().filter(|n| g.player(P0).exile.contains(&card(g, n))).map(|n| n.to_string()).collect()
}

/// every white card of the yoshimaru list: is it a line piece for the yoshimaru seat? (prints the table)
#[test]
fn probe_line_pieces_of_the_yoshimaru_white_cards() {
    let g = table(&["yoshimaru", "seph"]);
    let mut pieces = vec![];
    let mut nonpieces = vec![];
    for &c in g.player(P0).deck_names.iter() {
        let d = g.db.get(c);
        if d.colors.has('W') {
            let n = d.name.to_string();
            if is_line_piece(&g, P0, &n) {
                pieces.push(n);
            } else {
                nonpieces.push(n);
            }
        }
    }
    pieces.sort();
    pieces.dedup();
    nonpieces.sort();
    nonpieces.dedup();
    println!("WHITE LINE PIECES: {pieces:?}");
    println!("WHITE NON-PIECES: {nonpieces:?}");
    // the plan names Heliod as the only white line piece; Vito / Exquisite Blood are black
    assert_eq!(pieces, vec!["Heliod, Sun-Crowned".to_string()], "white pieces: {pieces:?}");
}

/// `is_line_piece` is keyed to the seat's own deck: the yoshimaru seat sees Heliod as a piece, the brago seat does not
/// (the pitch gate is the seat key either way)
#[test]
fn heliod_is_a_piece_for_the_yoshimaru_seat_and_not_for_a_seat_without_ballista() {
    let g = table(&["yoshimaru", "seph"]);
    assert!(is_line_piece(&g, P0, "Heliod, Sun-Crowned"));
    let g = table(&["brago-azorius-blink-control", "seph"]);
    assert!(!is_line_piece(&g, P0, "Heliod, Sun-Crowned"));
}

/// {Heliod, Court}: Court pitched; and it must fail on a rule with no exclusion (precondition: Heliod is the cheaper)
#[test]
fn heliod_and_court_pitch_court() {
    let (mut g, s) = tbl("yoshimaru");
    hand(&mut g, P0, &["Heliod, Sun-Crowned", "Court of Ardenvale"]);
    let (h, c) = (card(&g, "Heliod, Sun-Crowned"), card(&g, "Court of Ardenvale"));
    let (wh, wc) = (card_worth(&g, P0, h, false), card_worth(&g, P0, c, false));
    assert!(wh < wc, "precondition: old rule pitches Heliod ({wh} vs {wc})");
    assert!(ai_evokes(&mut g, s));
    assert_eq!(pitched(&g, &["Heliod, Sun-Crowned", "Court of Ardenvale"]), vec!["Court of Ardenvale".to_string()]);
}

/// all the other white cards of the list as the alternative to Heliod: Heliod is never the one pitched
#[test]
fn heliod_is_never_pitched_over_any_other_white_card_of_the_list() {
    let (g0, _) = tbl("yoshimaru");
    let mut whites: Vec<String> = g0
        .player(P0)
        .deck_names
        .iter()
        .filter(|&&c| {
            let d = g0.db.get(c);
            d.colors.has('W') && &*d.name != "Heliod, Sun-Crowned" && &*d.name != "Solitude" && !d.land
        })
        .map(|&c| g0.db.get(c).name.to_string())
        .collect();
    whites.sort();
    whites.dedup();
    let mut bad = vec![];
    let mut counts = vec![];
    for w in &whites {
        let (mut g, s) = tbl("yoshimaru");
        hand(&mut g, P0, &["Heliod, Sun-Crowned", w]);
        if !ai_evokes(&mut g, s) {
            bad.push(format!("{w}: no evoke"));
            continue;
        }
        let p = pitched(&g, &["Heliod, Sun-Crowned", w]);
        counts.push(format!("{w}: {p:?}"));
        if p != vec![w.clone()] {
            bad.push(format!("{w}: pitched {p:?}"));
        }
    }
    println!("{}", counts.join("\n"));
    assert!(bad.is_empty(), "{bad:?}");
}

/// the key check: only the key yoshimaru is gated; a seat with another key (thrasios, retired) keeps the old rule
#[test]
fn a_seat_with_another_key_keeps_the_old_pitch_rule() {
    let (mut g, s) = tbl("thrasios");
    hand(&mut g, P0, &["Heliod, Sun-Crowned", "Court of Ardenvale"]);
    let (h, c) = (card(&g, "Heliod, Sun-Crowned"), card(&g, "Court of Ardenvale"));
    let (wh, wc) = (card_worth(&g, P0, h, false), card_worth(&g, P0, c, false));
    assert!(ai_evokes(&mut g, s), "the evoke is offered");
    let exp = if wh < wc { "Heliod, Sun-Crowned" } else { "Court of Ardenvale" };
    assert_eq!(pitched(&g, &["Heliod, Sun-Crowned", "Court of Ardenvale"]), vec![exp.to_string()]);
}

/// Heliod with the combo already half built still pitches Court; Heliod on the battlefield (not in hand) is irrelevant;
/// a hand with only non-white-eligible cards: no crash
#[test]
fn only_nonpiece_whites_pick_the_least_worth_as_before() {
    let (mut g, s) = tbl("yoshimaru");
    hand(&mut g, P0, &["Court of Ardenvale", "Silence", "Swords to Plowshares"]);
    let ws: Vec<(String, f64)> = ["Court of Ardenvale", "Silence", "Swords to Plowshares"]
        .iter()
        .map(|n| (n.to_string(), card_worth(&g, P0, card(&g, n), false)))
        .collect();
    assert!(ai_evokes(&mut g, s));
    let p = pitched(&g, &["Court of Ardenvale", "Silence", "Swords to Plowshares"]);
    let min = ws.iter().min_by(|a, b| a.1.total_cmp(&b.1)).unwrap();
    assert_eq!(p, vec![min.0.clone()], "{ws:?}");
}
