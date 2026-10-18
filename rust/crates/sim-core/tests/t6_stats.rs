//! The Tier 6 report counters (`sim_core::t6stats`): scripted positions, one counter each. The counters are plain
//! additions to a Tier 6 player's `stats`; a player of another tier keeps none of them.

use sim_core::engine::{cast, removal, stack};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::combos::{self, Combo, Step, StepKind};
use sim_core::state::{Ctx, Game, StackItem, StackKind};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn stat(g: &Game, p: PlayerId, key: &str) -> i64 {
    g.player(p).stats.get(key).copied().unwrap_or(0)
}

fn keys_with(g: &Game, p: PlayerId, prefix: &str) -> Vec<String> {
    let mut v: Vec<String> = g.player(p).stats.keys().filter(|k| k.starts_with(prefix)).map(|k| k.to_string()).collect();
    v.sort();
    v
}

#[test]
fn the_t6_flag_is_set_for_the_pool_decks_and_godo_only() {
    let g = table(&["godo", "kinnan-bonder-mana-combo", "seph", "sauron"]);
    assert_eq!(g.players.iter().map(|p| p.t6).collect::<Vec<_>>(), [true, true, false, false]);
    for k in sim_core::t6stats::T6_KEYS {
        assert!(sim_core::t6stats::is_t6_key(k));
    }
    assert!(!sim_core::t6stats::is_t6_key("cloud"));
}

/// a spell of p on the stack, as if cast
fn on_stack(g: &mut Game, p: PlayerId, name: &str) -> u32 {
    let c = hand(g, p, &[name])[0];
    let i = g.player(p).hand.iter().position(|&x| x == c).unwrap();
    g.player_mut(p).hand.remove(i);
    let id = g.stack_pushes + 1;
    g.stack_pushes = id;
    g.stack.push(StackItem {
        id,
        controller: p,
        card: Some(c),
        ctx: Ctx::default(),
        zone: "hand",
        imp: 5.0,
        aff: vec![],
        generic: true,
        kind: StackKind::Spell,
        name: name.to_string(),
        passed: vec![],
        countered: false,
        countered_by: None,
    });
    id
}

#[test]
fn a_resolved_counterspell_is_counted_with_the_card_and_whose_spell_it_hit() {
    let mut g = table(&["kinnan-bonder-mana-combo", "godo"]);
    lands(&mut g, P0, "Island", 3, false);
    let ctr = hand(&mut g, P0, &["Counterspell"])[0];
    let id = on_stack(&mut g, P1, "Grave Titan");
    assert!(stack::cast_counter_spell(&mut g, P0, ctr, id).unwrap());
    assert!(g.stack.iter().all(|x| x.id != id || x.countered), "the Titan was countered");
    assert_eq!(stat(&g, P0, "ix|counter|Counterspell|godo"), 1);
    assert_eq!(g.last_counter, Some(ctr));
    assert_eq!(keys_with(&g, P0, "ix|"), ["ix|counter|Counterspell|godo"]);
    assert!(keys_with(&g, P1, "ix|").is_empty());
}

#[test]
fn a_player_of_another_tier_keeps_no_counters() {
    let mut g = table(&["seph", "godo"]);
    lands(&mut g, P0, "Island", 3, false);
    let ctr = hand(&mut g, P0, &["Counterspell"])[0];
    let id = on_stack(&mut g, P1, "Grave Titan");
    assert!(stack::cast_counter_spell(&mut g, P0, ctr, id).unwrap());
    assert!(keys_with(&g, P0, "ix|").is_empty());
    assert_eq!(stat(&g, P0, "spells_countered") + stat(&g, P1, "spells_countered"), 1, "the old counters still count");
}

#[test]
fn removal_is_counted_with_the_spell_and_the_owner_of_the_permanent() {
    let mut g = table(&["rograkh-silas-turbo-breach", "godo"]);
    let t = perm(&mut g, P1, "Grave Titan");
    let swords = card(&g, "Swords to Plowshares");
    removal::apply_removal(&mut g, Some(P0), t, "exile", Some(swords)).unwrap();
    assert_eq!(stat(&g, P0, "ix|removal|Swords to Plowshares|godo"), 1);
    // removing your own permanent is not interaction
    let own = perm(&mut g, P0, "Grave Titan");
    removal::apply_removal(&mut g, Some(P0), own, "exile", Some(swords)).unwrap();
    assert_eq!(keys_with(&g, P0, "ix|removal"), ["ix|removal|Swords to Plowshares|godo"]);
    // a card with no spell (an ability) is counted as "ability"
    let t2 = perm(&mut g, P1, "Grave Titan");
    removal::apply_removal(&mut g, Some(P0), t2, "exile", None).unwrap();
    assert_eq!(stat(&g, P0, "ix|removal|ability|godo"), 1);
}

#[test]
fn a_cast_stax_piece_is_counted_and_an_ordinary_spell_is_not() {
    let mut g = table(&["dack-fayden-white-stax", "godo"]);
    let (tri, titan) = (card(&g, "Trinisphere"), card(&g, "Grave Titan"));
    cast::on_cast(&mut g, P0, titan).unwrap();
    assert!(keys_with(&g, P0, "ix|stax").is_empty());
    cast::on_cast(&mut g, P0, tri).unwrap();
    assert_eq!(stat(&g, P0, "ix|stax|Trinisphere"), 1);
}

// ------------------------------------------------------------ combos: tries, stops, wins
fn yes(_g: &mut Game, _p: PlayerId) -> sim_core::flow::Res<bool> {
    Ok(true)
}

fn no_src(_g: &Game, _p: PlayerId) -> Option<PermId> {
    None
}

fn no_keys(_g: &Game, _p: PlayerId) -> Vec<PermId> {
    vec![]
}

fn synthetic(kind: StepKind) -> Combo {
    let st: &'static [Step] =
        Box::leak(Box::new([Step { label: "the step", kind, src: no_src, pay: (0, ""), run: yes, keys: no_keys }]));
    Combo { name: "synthetic", groups: &[], ready: |_, _| (true, vec![], vec![]), mana: (0, ""), finish: None, xcost: &[], steps: st }
}

#[test]
fn a_combo_stopped_by_ertai_is_counted_with_the_answering_card() {
    let mut g = table(&["godo", "seph"]);
    g.active = Some(P0);
    lands(&mut g, P1, "Island", 2, false);
    lands(&mut g, P1, "Swamp", 2, false);
    hand(&mut g, P1, &["Ertai Resurrected"]);
    combos::run_steps(&mut g, P0, &synthetic(StepKind::Trigger)).unwrap();
    assert_eq!(g.winner, None);
    assert_eq!(stat(&g, P0, "stop|synthetic|Ertai Resurrected"), 1);
    assert_eq!(stat(&g, P0, "combo_stopped"), 1, "the old counter is unchanged");
    assert_eq!(stat(&g, P0, "win|synthetic"), 0);
}

#[test]
fn an_unanswered_combo_is_counted_as_a_win_of_its_line() {
    let mut g = table(&["godo", "seph"]);
    g.active = Some(P0);
    combos::run_steps(&mut g, P0, &synthetic(StepKind::Trigger)).unwrap();
    assert_eq!(g.winner, Some(P0));
    assert_eq!(stat(&g, P0, "win|synthetic"), 1);
    assert!(keys_with(&g, P0, "stop|").is_empty());
}

#[test]
fn going_for_a_real_line_is_counted_once() {
    let mut g = table(&["godo", "seph"]);
    g.active = Some(P0);
    lands(&mut g, P0, "Mountain", 11, false);
    let i = combos::combo_index("equip {5}").unwrap();
    assert!(combos::ready(&g, P0, i));
    combos::attempt(&mut g, P0, i as i64).unwrap();
    assert_eq!(g.winner, Some(P0));
    let name = combos::all()[i].name;
    assert_eq!(stat(&g, P0, &format!("go|{name}")), 1);
    assert_eq!(stat(&g, P0, &format!("win|{name}")), 1);
    // a second try the same turn is refused and not counted
    combos::attempt(&mut g, P0, i as i64).unwrap();
    assert_eq!(stat(&g, P0, &format!("go|{name}")), 1);
}

#[test]
fn a_removal_spell_on_a_key_piece_names_itself_as_the_stopper() {
    let mut stopped = 0;
    for seed in 0..8u64 {
        let mut g = table_with(&["godo", "seph"], seed, 40);
        g.active = Some(P0);
        token(&mut g, P0, 3);
        lands(&mut g, P1, "Plains", 1, false);
        hand(&mut g, P1, &["Swords to Plowshares"]);
        let combo = {
            let mut c = synthetic(StepKind::Permanent);
            let st: &'static [Step] = Box::leak(Box::new([Step {
                label: "the token",
                kind: StepKind::Permanent,
                src: no_src,
                pay: (0, ""),
                run: yes,
                keys: |g, p| g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf).collect(),
            }]));
            c.steps = st;
            c
        };
        combos::run_steps(&mut g, P0, &combo).unwrap();
        if g.winner.is_none() {
            stopped += 1;
            assert_eq!(stat(&g, P0, "stop|synthetic|Swords to Plowshares"), 1, "seed {seed}");
        }
    }
    assert!(stopped >= 6, "{stopped}");
}

#[test]
fn a_mulligan_is_counted() {
    // the opening shuffle decides how many mulligans a hand needs; mull_n is that number (0-3), "mulls" the old
    // counter that only knows two and three
    let mut seen = [0u32; 4];
    for seed in 0..200u64 {
        let mut g = table(&["godo", "seph"]);
        let mut r = sim_core::rng::Rng::named(&format!("lib:{seed}:godo"));
        sim_core::engine::turn::mulligan(&mut g, P0, &mut r);
        let n = stat(&g, P0, "mull_n");
        assert!((0..=3).contains(&n));
        assert_eq!(g.player(P0).hand.len() as i64, 7 - (n - 1).max(0), "a hand of seven, less what was bottomed (n={n})");
        if n >= 2 {
            assert_eq!(stat(&g, P0, "mulls"), n);
        }
        seen[n as usize] += 1;
    }
    assert!(seen[0] > 0 && seen[1] > 0, "{seen:?}");
}
