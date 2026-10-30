//! --- phase4-values: the old card values beside the derived ones (`ai/values.rs`) for every card of every pool deck
//! (tiers t1-t5), with the Spearman rank correlation per tier, and the worst disagreements.
//!     AI_VALUES=0 cargo run --release -p sim-core --example values_diag [all]   (all: every card, not only the worst)
//!
//! State: each tier's decks seated four at a time (two tables cover all five), round and turn 3, three of each deck's
//! own lands on the battlefield, an empty hand. Cards a plan's key cards, prio rules or card code's own priority
//! decide are left out (both arms read those the same way).

use sim_core::ai::{plans, pool, values};
use sim_core::engine::tutors::card_worth;
use sim_core::ids::{CardId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

/// average ranks (ties share their mean rank)
fn ranks(v: &[f64]) -> Vec<f64> {
    let mut idx: Vec<usize> = (0..v.len()).collect();
    idx.sort_by(|&a, &b| v[a].total_cmp(&v[b]));
    let mut r = vec![0.0; v.len()];
    let mut i = 0;
    while i < idx.len() {
        let mut j = i;
        while j + 1 < idx.len() && v[idx[j + 1]] == v[idx[i]] {
            j += 1;
        }
        let avg = (i + j) as f64 / 2.0 + 1.0;
        for k in i..=j {
            r[idx[k]] = avg;
        }
        i = j + 1;
    }
    r
}

fn spearman(a: &[f64], b: &[f64]) -> f64 {
    let (ra, rb) = (ranks(a), ranks(b));
    let n = ra.len() as f64;
    let (ma, mb) = (ra.iter().sum::<f64>() / n, rb.iter().sum::<f64>() / n);
    let cov: f64 = ra.iter().zip(&rb).map(|(x, y)| (x - ma) * (y - mb)).sum();
    let va: f64 = ra.iter().map(|x| (x - ma).powi(2)).sum();
    let vb: f64 = rb.iter().map(|y| (y - mb).powi(2)).sum();
    if va == 0.0 || vb == 0.0 { 0.0 } else { cov / (va * vb).sqrt() }
}

struct Row {
    tier: &'static str,
    deck: String,
    name: String,
    old: f64,
    new: f64,
    worth_old: f64,
    worth_new: f64,
    bomb: f64,
    threat: f64,
    perm: bool,
    parts: [f64; 8],
}

fn same_both_arms(g: &Game, p: PlayerId, c: CardId) -> bool {
    let pl = g.player(p);
    let cfg = plans::config(pl.key);
    let d = g.db.get(c);
    pl.is_commander(c)
        || d.land
        || cfg.key_cards.iter().any(|x| *x.0 == *d.name)
        || cfg.prio_fn.is_some_and(|f| f(g, p, c).is_some())
        || g.registry.get(c).is_some_and(|i| i.prio.is_some())
}

fn main() {
    assert!(
        std::env::var("AI_VALUES").as_deref() == Ok("0"),
        "run with AI_VALUES=0: `generic_prio` then gives the old table, `derived_prio` the derived value"
    );
    let all = std::env::args().nth(1).as_deref() == Some("all");
    let mut rows: Vec<Row> = vec![];
    for t in ["t1", "t2", "t3", "t4", "t5"] {
        let decks = tier(t);
        let mut done: Vec<&str> = vec![];
        for table_keys in [&decks[0..4], &decks[1..5]] {
            let mut g = table(table_keys);
            g.round = 3;
            for i in 0..4 {
                let p = PlayerId(i as u8);
                g.player_mut(p).turns = 3;
                let lands: Vec<String> = {
                    let pl = g.player(p);
                    let mut v: Vec<String> = vec![];
                    for &c in &pl.library {
                        let d = g.db.get(c);
                        if d.land && v.len() < 3 {
                            v.push(d.name.to_string());
                        }
                    }
                    v
                };
                for l in lands {
                    lands_of(&mut g, p, &l);
                }
            }
            for (i, key) in table_keys.iter().enumerate() {
                if done.contains(key) {
                    continue;
                }
                done.push(key);
                let p = PlayerId(i as u8);
                let mut seen: Vec<CardId> = vec![];
                let lib = g.player(p).library.clone();
                for c in lib {
                    if seen.contains(&c) || same_both_arms(&g, p, c) {
                        continue;
                    }
                    seen.push(c);
                    let d = g.db.get(c);
                    rows.push(Row {
                        tier: t,
                        deck: key.to_string(),
                        name: d.name.to_string(),
                        old: pool::generic_prio(&g, p, c) as f64,
                        new: pool::derived_prio(&g, p, c) as f64,
                        worth_old: card_worth(&g, p, c, false),
                        worth_new: values::card_worth(&g, p, c),
                        bomb: d.bomb as f64,
                        threat: values::threat(&g, c),
                        perm: d.perm,
                        parts: values::worth_parts(&g, p, c, true),
                    });
                }
            }
        }
    }
    println!("tier  cards  prio rho  worth rho  bomb rho (permanents)  bombs: old>=5 derived>=5 both");
    for t in ["t1", "t2", "t3", "t4", "t5"] {
        let r: Vec<&Row> = rows.iter().filter(|x| x.tier == t).collect();
        let col = |f: &dyn Fn(&Row) -> f64, rs: &[&Row]| rs.iter().map(|x| f(x)).collect::<Vec<f64>>();
        let perms: Vec<&Row> = r.iter().copied().filter(|x| x.perm).collect();
        let ob = perms.iter().filter(|x| x.bomb >= 5.0).count();
        let nb = perms.iter().filter(|x| x.threat >= values::BOMB).count();
        let both = perms.iter().filter(|x| x.bomb >= 5.0 && x.threat >= values::BOMB).count();
        println!(
            "{t}    {:5}  {:8.3}  {:9.3}  {:21.3}  {ob} {nb} {both}",
            r.len(),
            spearman(&col(&|x| x.old, &r), &col(&|x| x.new, &r)),
            spearman(&col(&|x| x.worth_old, &r), &col(&|x| x.worth_new, &r)),
            spearman(&col(&|x| x.bomb, &perms), &col(&|x| x.threat, &perms)),
        );
    }
    // the worst disagreements: the largest gaps in rank within each tier
    let mut gaps: Vec<(f64, usize)> = vec![];
    for t in ["t1", "t2", "t3", "t4", "t5"] {
        let ix: Vec<usize> = (0..rows.len()).filter(|&i| rows[i].tier == t).collect();
        let ro = ranks(&ix.iter().map(|&i| rows[i].old).collect::<Vec<_>>());
        let rn = ranks(&ix.iter().map(|&i| rows[i].new).collect::<Vec<_>>());
        let n = ix.len() as f64;
        for (k, &i) in ix.iter().enumerate() {
            gaps.push(((rn[k] - ro[k]) / n, i));
        }
    }
    gaps.sort_by(|a, b| b.0.abs().total_cmp(&a.0.abs()));
    let show = if all { gaps.len() } else { 40 };
    println!(
        "\nworst disagreements (rank gap as a share of the tier; parts: mana cards removal damage board engine role fit)"
    );
    println!(
        "{:>6}  {:<4} {:<34} {:<32} {:>4} {:>4}  {:>5} {:>5}  {:>4} {:>5}  parts",
        "gap", "tier", "deck", "card", "old", "new", "w.old", "w.new", "bomb", "threat"
    );
    for &(gap, i) in gaps.iter().take(show) {
        let r = &rows[i];
        let parts: Vec<String> = r.parts.iter().map(|x| format!("{x:.1}")).collect();
        println!(
            "{gap:+6.2}  {:<4} {:<34} {:<32} {:4.0} {:4.0}  {:5.0} {:5.0}  {:4.0} {:5.1}  {}",
            r.tier,
            &r.deck[..r.deck.len().min(34)],
            &r.name[..r.name.len().min(32)],
            r.old,
            r.new,
            r.worth_old,
            r.worth_new,
            r.bomb,
            r.threat,
            parts.join(" ")
        );
    }
    // the bomb reads: where the tag and the derived threat disagree about "a bomb"
    println!("\nbomb disagreements (old bomb >= 5 xor derived threat >= {}):", values::BOMB);
    let mut seen: Vec<&str> = vec![];
    for r in rows.iter().filter(|r| r.perm && ((r.bomb >= 5.0) != (r.threat >= values::BOMB))) {
        if seen.contains(&r.name.as_str()) {
            continue;
        }
        seen.push(&r.name);
        println!("  {:<32} bomb {:2.0}  threat {:4.1}", r.name, r.bomb, r.threat);
    }
}

/// one of p's own lands onto the battlefield, untapped
fn lands_of(g: &mut Game, p: PlayerId, name: &str) {
    lands(g, p, name, 1, false);
}
