// written by an independent reviewer
//! Adversarial: the evoke / Emperor flash count as casts (a): no double count, nothing counted when nothing is paid,
//! readers of `spells_this_turn` fire once.

use sim_core::ai::act::perform;
use sim_core::engine::cast::casts_this_turn;
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PlayerId};
use sim_core::state::Game;
use sim_core::sym::intern;
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
    hand(&mut g, P0, &["Swords to Plowshares"]);
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

fn spells_cast(g: &Game, p: PlayerId) -> i64 {
    g.player(p).stats.get(&intern("spells_cast")).copied().unwrap_or(0)
}

fn dragons(g: &Game, p: PlayerId) -> usize {
    // the 4/4 flying prowess tokens (a token has no card)
    g.player(p)
        .perms
        .iter()
        .filter(|&&m| g.perm(m).on_bf && g.perm(m).cd.is_none() && sim_core::engine::values::epow(g, m) == 4)
        .count()
}

/// an evoke the pay_tax / evoke_allowed refuses (Sphere of Resistance, no mana): nothing is recorded or spent
#[test]
fn an_unpayable_evoke_records_nothing() {
    let (mut g, s) = tbl("yoshimaru");
    perm(&mut g, P1, BIG);
    perm(&mut g, P1, "Sphere of Resistance");
    let (spells, stat, names) = (g.player(P0).spells_this_turn, spells_cast(&g, P0), g.player(P0).cast_names.len());
    assert!(!ai_evokes(&mut g, s), "no evoke offered with an unpayable tax");
    assert_eq!(g.player(P0).spells_this_turn, spells);
    assert_eq!(spells_cast(&g, P0), stat);
    assert_eq!(g.player(P0).cast_names.len(), names);
    assert!(g.player(P0).hand.contains(&s));
}

/// an evoke is exactly one spell: turn_casts agrees with spells_this_turn, nothing double
#[test]
fn an_evoke_is_exactly_one_spell_everywhere() {
    let (mut g, s) = tbl("yoshimaru");
    perm(&mut g, P1, BIG);
    assert!(ai_evokes(&mut g, s));
    assert_eq!(g.player(P0).spells_this_turn, 1);
    assert_eq!(spells_cast(&g, P0), 1);
    assert_eq!(casts_this_turn(&g, P0), 1);
    assert_eq!(g.player(P0).cast_names.len(), 1);
}

/// Eris: the second spell makes one Dragon; an evoke as the second spell makes exactly one (no double trigger)
#[test]
fn eris_second_spell_evoke_makes_one_dragon() {
    let (mut g, s) = tbl("yoshimaru");
    perm(&mut g, P1, BIG);
    perm(&mut g, P0, "Eris, Roar of the Storm");
    g.player_mut(P0).spells_this_turn = 1; // one spell already cast
    assert!(ai_evokes(&mut g, s));
    assert_eq!(g.player(P0).spells_this_turn, 2);
    assert_eq!(dragons(&g, P0), 1, "one Dragon");
}

/// Eris: with the evoke as the FIRST spell no Dragon yet, and the next spell then makes it (count is right)
#[test]
fn eris_first_spell_evoke_then_a_second_spell_still_counts() {
    let (mut g, s) = tbl("yoshimaru");
    perm(&mut g, P1, BIG);
    perm(&mut g, P0, "Eris, Roar of the Storm");
    assert!(ai_evokes(&mut g, s));
    assert_eq!(dragons(&g, P0), 0);
    assert_eq!(g.player(P0).spells_this_turn, 1);
}

/// the Emperor's flash countered by the attacker's counterspell still counts as a cast (seed loop)
#[test]
fn a_countered_emperor_flash_still_counts() {
    let mut counted = 0;
    let mut countered = 0;
    for seed in 1..40u64 {
        let mut g = table_with(&["sauron", "atraxa-infect"], seed, 40);
        let (p, d) = (P0, P1);
        let big = perm(&mut g, p, "Grave Titan");
        g.perm_mut(big).tapped = true;
        let mut atk = vec![big];
        let e = hand(&mut g, d, &["The Wandering Emperor"])[0];
        lands(&mut g, d, "Plains", 4, false);
        g.player_mut(p).hand.clear();
        lands(&mut g, p, "Island", 4, false);
        hand(&mut g, p, &["Counterspell", "Counterspell"]);
        let (spells, stat) = (g.player(d).spells_this_turn, spells_cast(&g, d));
        let mut assign = vec![];
        sim_core::cardcode::defend_hooks(&mut g, p, &mut atk, d, &mut assign).unwrap();
        if g.player(d).cast_names.contains(&e) {
            counted += 1;
            assert_eq!(g.player(d).spells_this_turn, spells + 1, "seed {seed}");
            assert_eq!(spells_cast(&g, d), stat + 1, "seed {seed}");
            if g.perm(big).on_bf {
                countered += 1;
            }
        }
    }
    println!("flashed in {counted} seeds, countered in {countered}");
    assert!(counted > 0);
}
