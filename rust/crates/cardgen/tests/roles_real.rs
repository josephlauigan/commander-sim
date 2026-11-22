//! Opt-in checks over the generated local catalog. Fixtures cover normal CI without this file.
use cardgen::data::Project;
use cardgen::knowledge::Index;
use cardgen::llm;
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
    let mut invalid = vec![];
    for c in index.cards.iter().filter(|c| roles::is_deck_card(c)) {
        let lines = c.lines();
        for (r, e) in &a[&c.raw.oracle_id].evidence {
            if e.source == roles::Source::Rule {
                let guard = &taxonomy.role.iter().find(|t| &t.id == r).unwrap().guard;
                if !fancy_regex::Regex::new(guard)
                    .unwrap()
                    .is_match(&roles::normalize(c, lines[e.line.unwrap()].1))
                    .unwrap()
                {
                    invalid.push(format!("{} {r}", c.raw.name));
                }
            }
        }
    }
    assert!(invalid.is_empty(), "invalid evidence guards: {invalid:#?}");
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

#[test]
#[ignore = "needs data/knowledge/oracle.json (run cardgen bulk-sync)"]
fn llm_stage_prompt_lines_budget_and_empty_cache_merge_over_the_catalog() {
    let p = project();
    let mut index = Index::load(&p.data("knowledge/oracle.json")).unwrap();
    let rules = roles::generate(&index, "").unwrap();
    let taxonomy = Taxonomy::parse(&std::fs::read_to_string(p.data("knowledge/roles.toml")).unwrap()).unwrap();
    // every line a rule cites is a line the LLM is shown, so the two stages agree on what is readable
    let (mut routed, mut no_lines, mut sizes) = (0usize, 0usize, vec![]);
    for c in index.cards.iter().filter(|c| roles::is_deck_card(c)) {
        let lines = roles::prompt_lines(c);
        for e in rules[&c.raw.oracle_id].evidence.values() {
            assert!(
                lines.iter().any(|(i, _)| Some(*i) == e.line),
                "{}: a rule cites a line the LLM never sees",
                c.raw.name
            );
        }
        if c.legal == cardgen::knowledge::Legal::Legal && rules[&c.raw.oracle_id].needs_llm {
            routed += 1;
            no_lines += usize::from(lines.is_empty());
            sizes.push(llm::card_text(c).len());
        }
    }
    sizes.sort_unstable();
    let worst_batch: usize = sizes.iter().rev().take(10).sum();
    let system = llm::estimate_tokens(&llm::system_prompt(&taxonomy));
    let typical = sizes.iter().sum::<usize>() / sizes.len().max(1);
    println!(
        "routed legal cards {routed} ({no_lines} without readable lines); card text avg {typical} chars, max {}; \
         system prompt {system} tokens; the ten longest cards together {} tokens; typical batch of ten {} tokens",
        sizes.last().unwrap_or(&0),
        worst_batch / 4,
        typical * 10 / 4
    );
    assert!(system + worst_batch / 4 + 10 * 160 < 24_000, "a batch of the ten longest cards must fit the 32K context");
    // with nothing cached every routed card keeps its rule roles and is flagged; nothing else changes
    let (merged, _, stats) =
        llm::merge_table(&index, &taxonomy, &rules, &Default::default(), &Default::default(), None, &Default::default(), false).unwrap();
    for (id, a) in rules.iter() {
        let m = &merged[id];
        assert_eq!((&m.roles, &m.evidence, &m.route, m.needs_llm), (&a.roles, &a.evidence, &a.route, a.needs_llm));
    }
    assert_eq!(stats.with_answer + stats.llm_failed, stats.routed);
    index.attach_roles(&merged).unwrap();
}
