//! "it gains ..." in compiled Oracle text names the creature the text is about, not every creature you control.

use cardgen::dsl_parse::compile_card;
use cardgen::json::{J, dumps};

/// the compiled abilities as compact JSON
fn compiled(name: &str, oracle: &str, types: &str) -> String {
    let (abil, _) = compile_card(name, oracle, types, None);
    dumps(&J::Arr(abil.into_iter().map(J::Obj).collect()), None, false)
}

#[test]
fn atarka_grants_double_strike_to_the_attacking_dragon_only() {
    let c = compiled(
        "Atarka, World Render",
        "Flying, trample\nWhenever a Dragon you control attacks, it gains double strike until end of turn.",
        "C",
    );
    assert!(c.contains(r#""effects": [{"do": "grant", "keyword": "double strike", "what": {"sel": "event"}}]"#), "{c}");
    assert!(c.contains(r#""source": "you_each""#) && c.contains(r#""subtype": "dragon""#), "{c}");
    assert!(!c.contains(r#""sel": "all""#), "no grant to every creature: {c}");
}

#[test]
fn seasoned_hallowblade_grants_indestructible_to_itself_only() {
    let c = compiled(
        "Seasoned Hallowblade",
        "Discard a card: Tap this creature. It gains indestructible until end of turn. (Damage and effects that say \"destroy\" don't destroy it.)",
        "C",
    );
    assert!(c.contains(r#"{"do": "grant", "keyword": "indestructible", "what": {"sel": "self"}}"#), "{c}");
}

/// "That creature gains first strike" (Squire's Lightblade) is about the creature it is attached to, not the creature
/// of an attack trigger: no grant is compiled for it
#[test]
fn a_that_creature_sentence_outside_an_attack_trigger_is_not_read_as_the_event() {
    let c = compiled(
        "Squire's Lightblade",
        "Flash\nWhen this Equipment enters, attach it to target creature you control. That creature gains first strike until end of turn.\nEquipped creature gets +1/+0.\nEquip {3}",
        "E",
    );
    assert!(!c.contains("first strike"), "{c}");
}
