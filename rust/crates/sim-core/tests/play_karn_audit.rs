//! Practice-mode audit of the Karn deck (JD, 10/21): every card with an activated ability in its Oracle text (beyond
//! "{T}: Add ...") must offer it to the person. Run with `--ignored --nocapture` for the table.

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

#[test]
#[ignore]
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
        let c = card(&g, &n);
        let offered: Vec<String> = if g.db.get(c).land {
            let l = lands(&mut g, P0, &n, 1, false)[0];
            let mut v: Vec<String> = abilities::land_abilities(&g, P0, l).into_iter().map(|a| a.label).collect();
            v.extend(alt::land_alts(&mut g, P0, l).unwrap().into_iter().map(|a| a.0));
            v
        } else {
            let m = perm(&mut g, P0, &n);
            abilities_of(&mut g, P0, m).unwrap().into_iter().map(|a| a.0).collect()
        };
        let flag = if offered.len() < want.len() { "MISSING" } else { "ok" };
        if flag != "ok" {
            missing += 1;
        }
        println!("{flag}\t{n}\n\toracle: {want:?}\n\toffered: {offered:?}");
    }
    println!("cards with fewer offered abilities than Oracle lists: {missing}");
}
