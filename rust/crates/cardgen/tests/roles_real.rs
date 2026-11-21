//! Opt-in checks over the generated local catalog. Fixtures cover normal CI without this file.
use cardgen::data::Project;
use cardgen::knowledge::Index;
use cardgen::roles::{self, Taxonomy};
use std::path::PathBuf;

fn project() -> Project {
    Project::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..").canonicalize().unwrap())
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn catalog_is_deterministic_and_evidence_is_valid() {
    let p = project();
    let mut index = Index::load(&p.data("knowledge/oracle.json")).unwrap();
    let overrides = std::fs::read_to_string(p.data("knowledge/roles_overrides.toml")).unwrap();
    let a = roles::generate(&index, &overrides).unwrap();
    let b = roles::generate(&index, &overrides).unwrap();
    assert_eq!(roles::bytes(&a).unwrap(), roles::bytes(&b).unwrap());
    index.attach_roles(&a).unwrap();
    let taxonomy = Taxonomy::parse(&std::fs::read_to_string(p.data("knowledge/roles.toml")).unwrap()).unwrap();
    for c in &index.cards {
        let lines = c.lines();
        for (r, e) in &a[&c.raw.oracle_id].evidence {
            if e.source == "rule" {
                let guard = &taxonomy.role.iter().find(|t| &t.id == r).unwrap().guard;
                assert!(
                    fancy_regex::Regex::new(guard).unwrap().is_match(lines[e.line.unwrap()].1).unwrap(),
                    "{} {r}",
                    c.raw.name
                );
            }
        }
    }
    let stats = roles::stats(&index, &a);
    println!("{}", stats.text());
}

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn validation_thresholds() {
    let p = project();
    let index = Index::load(&p.data("knowledge/oracle.json")).unwrap();
    let map = roles::generate(&index, "").unwrap();
    let taxonomy = Taxonomy::parse(&std::fs::read_to_string(p.data("knowledge/roles.toml")).unwrap()).unwrap();
    let gold = std::fs::read_to_string(p.data("knowledge/roles_gold.toml")).unwrap();
    let report = roles::validate(&p, &index, &map, &gold, &taxonomy).unwrap();
    assert!(report.passed, "{}", report.text());
}
