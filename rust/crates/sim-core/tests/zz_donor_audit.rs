mod play_common;
use play_common::*;

#[test]
fn audit() {
    let g0 = table(&["karn", "kefka", "sauron"]);
    let names: Vec<String> = g0
        .db
        .iter()
        .filter(|d| d.creature && g0.registry.get(g0.db.id(&d.name).unwrap()).is_some_and(|i| i.options.is_some()))
        .map(|d| d.name.to_string())
        .collect();
    let mut hookonly = vec![];
    let mut haspath = vec![];
    for n in names {
        let r = std::panic::catch_unwind(|| {
            let mut g = table(&["karn", "kefka", "sauron"]);
            let m = perm(&mut g, P0, &n);
            let own = sim_core::human::cards::abilities(&g, P0, m).is_some()
                || !sim_core::dsl::human_abilities(&g, P0, m).is_empty()
                || !sim_core::human::abilities::permanent_abilities(&g, P0, m).is_empty()
                || !sim_core::human::karn::perm_abilities(&g, P0, m).is_empty()
                || !sim_core::human::cloud::perm_abilities(&g, P0, m).is_empty();
            own
        });
        match r {
            Ok(true) => haspath.push(n),
            Ok(false) => hookonly.push(n),
            Err(_) => hookonly.push(format!("{n} (panic)")),
        }
    }
    println!("HAS PATH ({}): {:?}", haspath.len(), haspath);
    println!("HOOK ONLY ({}): {:?}", hookonly.len(), hookonly);
}
