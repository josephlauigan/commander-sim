//! `cardgen audit --paths` on the committed data: the known doubles show, a declared split is not a double, and a
//! card with only AI code has none.

use cardgen::audit::{Audit, engine_db};
use cardgen::data::Project;
use cardgen::export::Inputs;
use cardgen::paths::{CardPaths, Class, all_paths, paths_ok};
use sim_core::hooks::SlotClass;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::OnceLock;

struct Fixture {
    all: BTreeMap<String, CardPaths>,
    /// the same, with a `paths_ok` added in memory for Ruby Medallion
    declared: BTreeMap<String, CardPaths>,
    /// cards whose only code is AI slots, and exactly `prio`
    prio_only: Vec<String>,
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
        ok2.insert("Ruby Medallion".into(), "a test: declared on purpose".into());
        let declared = all_paths(&db, &reg, &au, &inp, &ok2);
        let prio_only = db
            .iter()
            .filter(|d| reg.get(d.id).is_some_and(|i| i.slots() == ["prio"] && i.slots_of(SlotClass::Rule).is_empty()))
            .map(|d| d.name.to_string())
            .collect();
        Fixture { all, declared, prio_only }
    })
}

fn card(name: &str) -> &'static CardPaths {
    fixture().all.get(name).unwrap_or_else(|| panic!("{name}: no paths"))
}

fn has_double(c: &CardPaths, class: Class, what: &str) -> bool {
    c.doubles.iter().any(|(k, w)| *k == class && w.starts_with(what))
}

#[test]
fn medallions_and_starfield_mystic_are_hard_cost_doubles() {
    for n in ["Ruby Medallion", "Jet Medallion", "Pearl Medallion", "Starfield Mystic"] {
        let c = card(n);
        assert_eq!(c.class, Some(Class::Hard), "{n}: {c:?}");
        assert!(has_double(c, Class::Hard, "cost:"), "{n}: {:?}", c.doubles);
    }
}

#[test]
fn laelia_is_a_hard_trigger_double() {
    let c = card("Laelia, the Blade Reforged");
    assert_eq!(c.class, Some(Class::Hard), "{c:?}");
    assert!(has_double(c, Class::Hard, "trigger: triggered:attack + attack"), "{:?}", c.doubles);
}

#[test]
fn hexing_squelcher_is_uncounterable_twice() {
    let c = card("Hexing Squelcher");
    assert!(has_double(c, Class::Hard, "uncounterable:"), "{:?}", c.doubles);
}

#[test]
fn paths_ok_makes_a_double_a_declared_split() {
    let c = &fixture().declared["Ruby Medallion"];
    assert_eq!(c.class, Some(Class::SplitOk), "{c:?}");
    assert!(c.paths_ok.is_some());
    // the double is still listed, only its class changes
    assert!(has_double(c, Class::Hard, "cost:"));
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
