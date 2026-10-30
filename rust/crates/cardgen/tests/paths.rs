//! `cardgen audit --paths` on the committed data: no hard double is left undeclared, the doubles Phase 2 removed stay
//! gone, a declared split is not a double (and without its `paths_ok` it is one again), and a card with only AI code
//! has none.

use cardgen::audit::{Audit, engine_db};
use cardgen::data::Project;
use cardgen::export::Inputs;
use cardgen::paths::{CardPaths, Class, all_paths, check, load_known, paths_ok, refusal};
use sim_core::hooks::SlotClass;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::OnceLock;

struct Fixture {
    all: BTreeMap<String, CardPaths>,
    /// the same, with Hexing Squelcher's `paths_ok` taken out in memory
    undeclared: BTreeMap<String, CardPaths>,
    /// cards whose only code is AI slots, and exactly `prio`
    prio_only: Vec<String>,
    /// data/paths_known.txt
    known: BTreeMap<String, Class>,
}

fn fixture() -> &'static Fixture {
    static F: OnceLock<Fixture> = OnceLock::new();
    F.get_or_init(|| {
        let p = Project::new(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.."));
        let inp = Inputs::load(&p).unwrap();
        let built = inp.build();
        let au = Audit::new(&inp, &built.cards).unwrap();
        let (db, reg) = engine_db(&inp).unwrap();
        let ok = paths_ok(&inp);
        let all = all_paths(&db, &reg, &au, &inp, &ok);
        let mut ok2 = ok.clone();
        ok2.remove("Hexing Squelcher");
        let undeclared = all_paths(&db, &reg, &au, &inp, &ok2);
        let prio_only = db
            .iter()
            .filter(|d| reg.get(d.id).is_some_and(|i| i.slots() == ["prio"] && i.slots_of(SlotClass::Rule).is_empty()))
            .map(|d| d.name.to_string())
            .collect();
        let known = load_known(&p).unwrap();
        Fixture { all, undeclared, prio_only, known }
    })
}

fn card(name: &str) -> &'static CardPaths {
    fixture().all.get(name).unwrap_or_else(|| panic!("{name}: no paths"))
}

fn has_double(c: &CardPaths, class: Class, what: &str) -> bool {
    c.doubles.iter().any(|(k, w)| *k == class && w.starts_with(what))
}

#[test]
fn no_hard_double_is_left_undeclared() {
    let hard: Vec<&String> =
        fixture().all.iter().filter(|(_, c)| c.class == Some(Class::Hard)).map(|(n, _)| n).collect();
    assert!(hard.is_empty(), "hard doubles without paths_ok: {hard:?}");
}

#[test]
fn the_cost_reducers_run_on_the_compiled_static_alone() {
    for n in ["Ruby Medallion", "Jet Medallion", "Pearl Medallion", "Foundry Inspector", "Etherium Sculptor"] {
        let c = card(n);
        assert!(c.rule.is_empty() && c.class.is_none(), "{n}: {c:?}");
        assert!(c.dsl.iter().any(|l| l == "static:cost"), "{n}: {:?}", c.dsl);
    }
    // Starfield Mystic: the discount compiled, its +1/+1 counter card code (a declared split)
    let c = card("Starfield Mystic");
    assert!(!c.rule.contains(&"cost"), "{c:?}");
    assert_eq!(c.class, Some(Class::SplitOk), "{c:?}");
}

#[test]
fn laelia_runs_on_card_code_alone() {
    let c = card("Laelia, the Blade Reforged");
    assert!(c.dsl.is_empty() && c.doubles.is_empty() && c.class.is_none(), "{c:?}");
}

#[test]
fn hexing_squelcher_is_uncounterable_twice() {
    let c = &fixture().undeclared["Hexing Squelcher"];
    assert_eq!(c.class, Some(Class::Hard), "{c:?}");
    assert!(has_double(c, Class::Hard, "uncounterable:"), "{:?}", c.doubles);
}

#[test]
fn paths_ok_makes_a_double_a_declared_split() {
    let c = card("Hexing Squelcher");
    assert_eq!(c.class, Some(Class::SplitOk), "{c:?}");
    assert!(c.paths_ok.is_some());
    // the double is still listed, only its class changes
    assert!(has_double(c, Class::Hard, "uncounterable:"));
}

#[test]
fn a_card_with_only_ai_code_has_no_double() {
    let f = fixture();
    assert!(!f.prio_only.is_empty(), "no card has only a prio slot");
    for n in &f.prio_only {
        if let Some(c) = f.all.get(n) {
            assert!(c.rule.is_empty() && c.group.is_none(), "{n}: {c:?}");
            assert!(c.doubles.is_empty() && c.class.is_none(), "{n}: {:?}", c.doubles);
        }
    }
}

#[test]
fn string_literals_skip_comments_and_char_literals() {
    let src =
        "let a = \"Sol Ring\"; // \"not this\"\nlet c = '\"'; fn f<'a>(x: &'a str) {} /* \"nor\" */ g(\"x\\\"y\");";
    assert_eq!(cardgen::paths::string_literals(src), ["Sol Ring", "x\\\"y"]);
}

/// `cardgen export`'s guard on the committed data (so `cargo test` fails on a double even if nobody re-exports): no
/// card runs through two paths unless declared (`paths_ok`) or a known double of the same class, and no known-list
/// line is stale
#[test]
fn the_export_guard_passes_the_committed_data() {
    let f = fixture();
    let (new, stale) = check(&f.all, &f.known);
    assert!(new.is_empty(), "{}", refusal(&new));
    assert!(stale.is_empty(), "data/paths_known.txt only shrinks: delete the line of each card that is no longer that double (or no longer exists): {stale:?}");
    // the known list holds no hard double and no declared split
    assert!(f.known.values().all(|c| matches!(c, Class::Shadowed | Class::Competing | Class::Split)));
}

#[test]
fn the_export_guard_refuses_an_undeclared_double() {
    let f = fixture();
    // Hexing Squelcher without its paths_ok: a hard double the guard names, with its paths and the fixes
    let (new, _) = check(&f.undeclared, &f.known);
    let names: Vec<&str> = new.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(names, ["Hexing Squelcher"]);
    let msg = refusal(&new);
    for want in ["Hexing Squelcher [hard]: uncounterable:", "rule: ", "paths_ok = \"why\"", "dsl = []", "nothing was written"] {
        assert!(msg.contains(want), "missing {want:?}:\n{msg}");
    }
    // a hard double is refused even when the known list names the card
    let mut known = f.known.clone();
    known.insert("Hexing Squelcher".into(), Class::Hard);
    assert_eq!(check(&f.undeclared, &known).0.len(), 1);
    // a known double that changes class is a new double
    let name = f.known.iter().find(|(_, c)| **c == Class::Split).map(|(n, _)| n).expect("a known split");
    let mut known = f.known.clone();
    known.insert(name.clone(), Class::Competing);
    let (new, stale) = check(&f.all, &known);
    assert_eq!(new.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), [name.as_str()]);
    assert_eq!(stale.len(), 1);
}
