//! Practice-mode audit of the Karn deck (JD, 10/21): every card with an activated ability in its Oracle text (beyond
//! "{T}: Add ...") must offer it to the person. `--nocapture` prints the table.

mod play_common;
use play_common::*;
use sim_core::human::human::abilities_of;
use sim_core::human::{abilities, alt};
use sim_core::testkit::{data_dir, deck};

/// the Oracle lines that are activated abilities other than plain mana abilities
fn activated_lines(oracle: &str) -> Vec<String> {
    oracle
        .lines()
        .filter_map(|l| {
            let (cost, eff) = l.split_once(": ")?;
            let costish = cost.contains('{') || cost.starts_with("Sacrifice") || cost.starts_with("Equip");
            (costish && !cost.contains('"') && !eff.starts_with("Add ")).then(|| l.to_string())
        })
        .collect()
}

/// the position each card's "activate only if" needs (a power-4 creature, three same-named lands, three artifacts)
fn condition(g: &mut sim_core::state::Game, n: &str) {
    match n {
        "Bonders' Enclave" => {
            token(g, P0, 4);
        }
        "Endless Atlas" => {
            lands(g, P0, "Wastes", 3, false);
        }
        "Inventors' Fair" | "Kuldotha Forgemaster" => {
            for a in ["Sol Ring", "Mana Vault", "Thought Vessel"] {
                perm(g, P0, a);
            }
        }
        "Buried Ruin" => {
            let c = card(g, "Sol Ring");
            g.player_mut(P0).gy.push(c);
        }
        _ => {}
    }
}

/// Every Oracle line that is an activated ability must be offered (run with `--nocapture` for the table).
#[test]
fn karn_deck_abilities_in_practice() {
    let cache: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(data_dir().join("scryfall_cache.json")).unwrap()).unwrap();
    let mut names: Vec<String> = deck("karn").cards.clone();
    names.sort();
    names.dedup();
    let mut missing = 0;
    for n in names {
        let Some(oracle) = cache[n.to_lowercase()]["oracle_text"].as_str() else { continue };
        let want = activated_lines(oracle);
        if want.is_empty() {
            continue;
        }
        let mut g = table(&["karn", "veyran"]);
        seat(&mut g, P0, vec![]);
        g.player_mut(P0).pool.add('C', 20); // enough to pay any of them
        g.active = Some(P0);
        condition(&mut g, &n);
        let c = card(&g, &n);
        let offered: Vec<String> = if g.db.get(c).land {
            let l = lands(&mut g, P0, &n, 1, false)[0];
            let mut v: Vec<String> = abilities::land_abilities(&g, P0, l).into_iter().map(|a| a.label).collect();
            v.extend(alt::land_alts(&mut g, P0, l).unwrap().into_iter().map(|a| a.0));
            v
        } else {
            let m = if n == "Walking Ballista" {
                let c = card(&g, &n);
                sim_core::engine::zones::enter(
                    &mut g,
                    P0,
                    c,
                    sim_core::engine::zones::Enter { sick: false, plus: 1, ..Default::default() },
                )
                .unwrap() // it enters with no counters otherwise, and a 0/0 dies
            } else {
                perm(&mut g, P0, &n)
            };
            abilities_of(&mut g, P0, m).unwrap().into_iter().map(|a| a.0).collect()
        };
        let flag = if offered.len() < want.len() { "MISSING" } else { "ok" };
        if flag != "ok" {
            missing += 1;
        }
        println!("{flag}\t{n}\n\toracle: {want:?}\n\toffered: {offered:?}");
    }
    println!("cards with fewer offered abilities than Oracle lists: {missing}");
    assert_eq!(missing, 0, "a Karn card offers fewer activated abilities than its Oracle text has");
}
