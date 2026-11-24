//! Adversarial checks for 5af7fd67 on the cardgen side ({2/X} in `conv_cost`, `hybrid`, `mana_value`; the export is
//! deterministic and leaves the checked-in data as it is). Reviewer's file: source and data are unchanged. No network.

use cardgen::autotag::conv_cost;
use cardgen::cardinfo::{hybrid_of_cost, mana_value};
use cardgen::cd::Cd;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn mv(name: &str, mc: &str) -> i64 {
    let cd = Cd::new(name, "AC", &conv_cost(mc), "");
    mana_value(&cd, &hybrid_of_cost(mc))
}

#[test]
fn mixed_costs_keep_the_generic_part_and_every_symbol() {
    assert_eq!(conv_cost("{2}{2/W}"), "2W");
    assert_eq!(conv_cost("{2/W}{W}"), "WW");
    assert_eq!(conv_cost("{3}{W/U}{2/B}"), "3WB");
    assert_eq!(hybrid_of_cost("{3}{W/U}{2/B}"), ["WU", "B2"]);
    assert_eq!(mv("a", "{2}{2/W}"), 4);
    assert_eq!(mv("b", "{2/W}{W}"), 3);
    assert_eq!(mv("c", "{3}{W/U}{2/B}"), 6);
    assert_eq!(mv("d", "{X}{2/R}"), 2, "X counts 0");
}

#[test]
fn symbols_that_are_not_a_two_hybrid_are_unchanged() {
    // Phyrexian, Phyrexian hybrid, colourless, snow, X, plain hybrid and a split cost
    assert_eq!(conv_cost("{W/P}"), "W");
    assert_eq!(conv_cost("{G/W/P}"), "G");
    assert_eq!(conv_cost("{2}{C}{S}"), "3", "{{C}} counts one; snow adds nothing (as before)");
    assert_eq!(conv_cost("{X}{X}{G}"), "G");
    assert_eq!(conv_cost("{R/G}{R/G}"), "RR");
    assert_eq!(conv_cost("{1}{W} // {2/U}"), "1W", "only the front face of a split cost");
    assert!(hybrid_of_cost("{W/P}{G/W/P}{C}{S}{X}").is_empty());
    assert_eq!(
        hybrid_of_cost("{1}{W} // {2/U}"),
        ["U2"],
        "(hybrid_of_cost reads whatever it is given; `hybrid` strips the face)"
    );
}

#[test]
fn the_export_is_the_checked_in_data_byte_for_byte() {
    let project = cardgen::data::Project::new(root());
    let out = std::env::temp_dir().join(format!("adv_hybrid_export_{}", std::process::id()));
    std::fs::create_dir_all(&out).unwrap();
    let (cards, _decks) = cardgen::export::run(&project, &out, true).expect("offline export");
    assert!(cards > 3000);
    for f in ["cards.json", "decks.json"] {
        let a = std::fs::read(out.join(f)).unwrap();
        let b = std::fs::read(root().join("data").join(f)).unwrap();
        assert!(a == b, "{f} differs from data/{f} after a fresh offline export");
    }
    let _ = std::fs::remove_dir_all(&out);
}
