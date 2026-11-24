//! `--swap-commander "OLD=>NEW"`: a paired A/B run whose variant has another COMMANDER. `--swap` edits the card list
//! only; the seat's commander comes from the deck record, so a commander change needs its own option.
//!
//! The commander pair rides in the swap list as its first entry (`run` in main.rs puts it there), so the model-status
//! line, the preflight, the coverage block and the explain breakdown all see the new commander like a swapped-in card.
//! What is different is the list and the seat: the deck's list holds the commander as one of its 100 cards, and the
//! seat's library is that list without the commander. The variant puts NEW at OLD's position, so the 99 cards keep
//! their positions and both lists get the same library permutation from the same seed (as `swap_in_place` does for
//! `--swap`). The `--swap` pairs are applied to the 99 of the variant afterwards, so a card swapped in may be the old
//! commander.

use crate::args::Args;
use crate::play::Sim;
use cardgen::data::FormatDef;
use cardgen::decks::{card_problems, structural_problems};
use cardgen::json::J;
use std::path::Path;

/// "OLD=>NEW": one pair, both names given
pub fn parse(raw: &str) -> Result<(String, String), String> {
    let bad = || format!("--swap-commander must look like 'Old Commander=>New Commander': {raw}");
    let (old, new) = raw.split_once("=>").ok_or_else(bad)?;
    let (old, new) = (old.trim(), new.trim());
    if old.is_empty() || new.is_empty() || new.contains("=>") {
        return Err(bad());
    }
    if old == new {
        return Err(format!("--swap-commander: {old} is already the commander"));
    }
    Ok((old.to_string(), new.to_string()))
}

/// the new commander of a run, if it has `--swap-commander` (the pair is the first swap)
pub fn new_commander<'a>(a: &Args, swaps: &'a [(String, String)]) -> Option<&'a str> {
    a.swap_commander.as_ref().and_then(|_| swaps.first()).map(|s| s.1.as_str())
}

/// A plain `--swap` edits the 99 only, but `swap_in_place` and `apply_swaps` also match the commander's own slot in the
/// list, so a pair naming the deck's commander (out or in) would run a seat with a doubled or missing commander. Refused
/// before any game, with the option that does change a commander.
pub fn plain_swap_problem(commanders: &[&str], swaps: &[(String, String)]) -> Option<String> {
    swaps.iter().flat_map(|(o, i)| [o, i]).find(|c| commanders.contains(&c.as_str())).map(|c| {
        format!("--swap: '{c}' is the deck's commander: use --swap-commander OLD=>NEW to change the commander")
    })
}

/// the commanders of the variant seat, for reports (`base` when the run changes none)
pub fn commanders_of(base: &[String], new: Option<&str>) -> Vec<String> {
    match new {
        Some(n) => vec![n.to_string()],
        None => base.to_vec(),
    }
}

/// The variant list: the base with NEW at the commander's position and the `--swap` pairs applied in place to the
/// 99 (the commander's slot is never matched by a swap). NEW may not be in the 99 once the swaps are applied.
pub fn variant_list(
    base: &[String],
    deck: &str,
    old: &str,
    new: &str,
    swaps: &[(String, String)],
) -> Result<Vec<String>, String> {
    let ci = base
        .iter()
        .position(|c| c == old)
        .ok_or_else(|| format!("--swap-commander: '{old}' is not in the current {deck} list"))?;
    let mut var = base.to_vec();
    for (out, inn) in swaps {
        let j = var.iter().enumerate().position(|(i, c)| i != ci && c == out).ok_or_else(|| {
            if out == old {
                format!(
                    "--swap: '{out}' is the commander being replaced, not one of the 99: it is not a card to swap out \
                     (it can be swapped IN: --swap \"Some Card=>{out}\")"
                )
            } else {
                format!("'{out}' is not in the current {deck} list")
            }
        })?;
        var[j] = inn.clone();
    }
    if var.iter().enumerate().any(|(i, c)| i != ci && c == new) {
        return Err(format!(
            "--swap-commander: {new} is already in the 99 of the {deck} list once the swaps are applied (a commander \
             deck is singleton); swap it out with --swap too"
        ));
    }
    var[ci] = new.to_string();
    Ok(var)
}

/// What the variant list breaks that the base list does not, by cardgen's deck checks under the deck's format: size,
/// singleton, the commander present, "can be your commander" (a legendary creature, or text saying so), every card
/// within the new commander's colour identity, and the format's bans and legality. `recs` is the Scryfall record of a
/// card name.
pub fn new_problems(
    fmt: &FormatDef,
    base: &[String],
    var: &[String],
    old: &str,
    new: &str,
    recs: &dyn Fn(&str) -> Option<J>,
) -> Vec<String> {
    let all = |list: &[String], cmd: &str| {
        let mut p = structural_problems(list, &[cmd], 100);
        p.extend(card_problems(fmt, list, &[cmd], None, None, recs).0);
        p
    };
    let before = all(base, old);
    all(var, new).into_iter().filter(|p| !before.contains(p)).collect()
}

/// `new_problems` with the repository's formats.toml and Scryfall cache
fn catalog_problems(
    root: &Path,
    format_key: &str,
    base: &[String],
    var: &[String],
    old: &str,
    new: &str,
) -> Result<Vec<String>, String> {
    let project = cardgen::data::Project::new(root);
    let formats = cardgen::data::load_formats(&project)?;
    let fmt = cardgen::decks::format_of(&formats, format_key)?;
    let cache = cardgen::scryfall::Cache::load(&project.data("scryfall_cache.json"))?;
    Ok(new_problems(fmt, base, var, old, new, &|n| cache.get(n).cloned()))
}

/// Every check on a commander change, before any game: OLD is the deck's commander, the deck has one commander, NEW
/// has an engine record and fits the deck's colours, the variant list is legal under the deck's format (Commander
/// legality, "can be your commander", bans, singleton) and is admitted at the table. Returns the variant list.
pub fn check(sim: &Sim, deck: &str, old: &str, new: &str, swaps: &[(String, String)]) -> Result<Vec<String>, String> {
    let rec = sim.engine.decks.get(deck).ok_or_else(|| format!("no deck {deck:?}"))?;
    if rec.commander != old {
        return Err(format!("--swap-commander: '{old}' is not {deck}'s commander ({})", rec.commander));
    }
    if let Some(c2) = &rec.commander2 {
        return Err(format!("--swap-commander: {deck} has two commanders ({old} and {c2}); not supported"));
    }
    if sim.engine.db.id(new).is_none() {
        return Err(format!("--swap-commander: {new} is not in the engine's card data (data/cards.json)"));
    }
    let base = &sim.mine[deck];
    let var = variant_list(base, deck, old, new, swaps)?;
    let root = crate::decks::root()?;
    let problems = catalog_problems(&root, &sim.format_for(deck).key, base, &var, old, new)?;
    if !problems.is_empty() {
        return Err(format!(
            "--swap-commander: the {deck} list with {new} as its commander is not legal: {}",
            problems.join("; ")
        ));
    }
    // the table's format must admit the variant list (a list-based ban list applies to a deck of another format)
    let mut raw = rec.clone();
    raw.cards = var.clone();
    raw.commander = new.to_string();
    if let Err(why) = sim.format_for(deck).admits(&raw, sim.engine.format_of(deck)) {
        return Err(format!("--swap-commander: the variant cannot be played at this table: {}", why.join("; ")));
    }
    Ok(var)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::play::RunOpts;
    use cardgen::jo;
    use sim_core::settings::AiMode;

    fn v(x: &[&str]) -> Vec<String> {
        x.iter().map(|s| s.to_string()).collect()
    }

    fn pairs(x: &[(&str, &str)]) -> Vec<(String, String)> {
        x.iter().map(|(a, b)| (a.to_string(), b.to_string())).collect()
    }

    #[test]
    fn the_pair_parses_to_one_old_and_one_new() {
        assert_eq!(parse(" A, B => C ").unwrap(), ("A, B".to_string(), "C".to_string()));
        for bad in ["A", "A=>", "=>B", "A=>B=>C", "A->B", "A=>A"] {
            assert!(parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn a_plain_swap_naming_the_decks_commander_is_refused_either_way() {
        let cmd = ["Old"];
        assert!(plain_swap_problem(&cmd, &pairs(&[("A", "B")])).is_none());
        for p in [("Old", "B"), ("A", "Old")] {
            let e = plain_swap_problem(&cmd, &pairs(&[("X", "Y"), p])).unwrap();
            assert!(e.contains("'Old' is the deck's commander") && e.contains("--swap-commander"), "{e}");
        }
        // a second commander counts too
        assert!(plain_swap_problem(&["Old", "Two"], &pairs(&[("Two", "B")])).is_some());
    }

    #[test]
    fn the_new_commander_takes_the_old_ones_slot_and_the_99_keep_their_positions() {
        let base = v(&["A", "Old", "C", "D"]);
        assert_eq!(variant_list(&base, "d", "Old", "New", &[]).unwrap(), ["A", "New", "C", "D"]);
        // --swap pairs apply in place to the 99: the old commander may come in as a card
        let swaps = pairs(&[("C", "Old")]);
        assert_eq!(variant_list(&base, "d", "Old", "New", &swaps).unwrap(), ["A", "New", "Old", "D"]);
        // a swap of a card the list does not have, and of the old commander as a card out, are refused
        assert!(variant_list(&base, "d", "Old", "New", &pairs(&[("Z", "Q")])).is_err());
        let e = variant_list(&base, "d", "Old", "New", &pairs(&[("Old", "Q")])).unwrap_err();
        assert!(e.contains("commander being replaced"), "{e}");
        assert!(variant_list(&base, "d", "Nope", "New", &[]).is_err());
    }

    #[test]
    fn a_new_commander_already_in_the_99_is_a_singleton_clash_unless_a_swap_removes_it() {
        let base = v(&["A", "Old", "New", "D"]);
        let e = variant_list(&base, "d", "Old", "New", &[]).unwrap_err();
        assert!(e.contains("already in the 99"), "{e}");
        // swapping it out with --swap first is fine, and the swap does not touch the commander's slot
        let swaps = pairs(&[("New", "X")]);
        assert_eq!(variant_list(&base, "d", "Old", "New", &swaps).unwrap(), ["A", "New", "X", "D"]);
        // a swap that brings it in is the clash
        let swaps = pairs(&[("D", "New")]);
        assert!(variant_list(&v(&["A", "Old", "C", "D"]), "d", "Old", "New", &swaps).is_err());
    }

    fn rec(name: &str, ident: &[&str], tl: &str, text: &str, scry: &str) -> J {
        jo! {
            "name" => name,
            "color_identity" => J::Arr(ident.iter().map(|c| J::from(*c)).collect()),
            "type_line" => tl,
            "oracle_text" => text,
            "legalities" => jo!{"commander" => scry},
        }
    }

    fn fmt(key: &str) -> FormatDef {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let all = cardgen::data::load_formats(&cardgen::data::Project::new(root)).unwrap();
        all.into_iter().find(|f| f.key == key).unwrap()
    }

    /// a 100-card list: the commander and 99 basics-free filler
    fn list(cmd: &str) -> Vec<String> {
        std::iter::once(cmd.to_string()).chain((1..100).map(|i| format!("Filler {i}"))).collect()
    }

    fn problems(f: &FormatDef, new: &str, table: &[J]) -> Vec<String> {
        let (base, mut var) = (list("Old"), list("Old"));
        var[0] = new.to_string();
        let recs = |n: &str| match n {
            "Old" => Some(rec("Old", &["W", "U"], "Legendary Creature", "", "legal")),
            n if n.starts_with("Filler") => Some(rec(n, &["W"], "Creature", "", "legal")),
            n => table.iter().find(|r| r.s_or("name") == n).cloned(),
        };
        new_problems(f, &base, &var, "Old", new, &recs)
    }

    #[test]
    fn a_legal_commander_in_the_decks_colours_passes() {
        let new = rec("New", &["W", "U"], "Legendary Creature - Elf", "", "legal");
        assert!(problems(&fmt("commander"), "New", &[new]).is_empty());
    }

    #[test]
    fn illegal_banned_and_out_of_identity_commanders_are_each_named() {
        let c = fmt("commander");
        let banned = rec("Banned One", &["W"], "Legendary Creature", "", "banned");
        assert!(problems(&c, "Banned One", &[banned]).iter().any(|p| p.contains("banned in Commander")));
        let nl = rec("Silver Border", &["W"], "Legendary Creature", "", "not_legal");
        assert!(problems(&c, "Silver Border", &[nl]).iter().any(|p| p.contains("not legal in Commander")));
        let red = rec("Red One", &["R"], "Legendary Creature", "", "legal");
        // the 99 are white: a commander of another colour leaves them outside its identity
        let p = problems(&c, "Red One", &[red]);
        assert!(p.iter().any(|p| p.contains("outside colour identity R")), "{p:?}");
        // the Cali list bans its names whatever Scryfall says
        let nadu = rec("Nadu, Winged Wisdom", &["G"], "Legendary Creature", "", "banned");
        let p = problems(&fmt("cali"), "Nadu, Winged Wisdom", &[nadu]);
        assert!(p.iter().any(|p| p.contains("banned in Cali Commander")), "{p:?}");
    }

    #[test]
    fn only_a_legendary_creature_or_a_card_that_says_so_can_be_the_commander() {
        let c = fmt("commander");
        let rock = rec("Sol Ring", &[], "Artifact", "{T}: Add {C}{C}.", "legal");
        let p = problems(&c, "Sol Ring", &[rock]);
        assert!(p.iter().any(|p| p.contains("is not a legendary creature")), "{p:?}");
        let pw = rec(
            "Walker",
            &["W"],
            "Legendary Planeswalker - Walker",
            "+1: Draw a card.\nWalker can be your commander.",
            "legal",
        );
        assert!(problems(&c, "Walker", &[pw]).is_empty());
        let plain_pw = rec("Plain Walker", &["W"], "Legendary Planeswalker", "+1: Draw a card.", "legal");
        assert!(!problems(&c, "Plain Walker", &[plain_pw]).is_empty());
    }

    #[test]
    fn problems_the_base_list_already_has_are_not_blamed_on_the_change() {
        let c = fmt("commander");
        // both lists hold a card Scryfall calls banned; only the new commander's own problems are reported
        let (mut base, mut var) = (list("Old"), list("Old"));
        base[5] = "Banned Card".into();
        var[5] = "Banned Card".into();
        var[0] = "New".into();
        let recs = |n: &str| match n {
            "Old" | "New" => Some(rec(n, &["W"], "Legendary Creature", "", "legal")),
            "Banned Card" => Some(rec(n, &["W"], "Creature", "", "banned")),
            n => Some(rec(n, &["W"], "Creature", "", "legal")),
        };
        assert!(new_problems(&c, &base, &var, "Old", "New", &recs).is_empty());
    }

    fn sim() -> Sim {
        Sim::load(RunOpts { ai: AiMode::Adaptive, temp: 1.0, jobs: 2 }).unwrap()
    }

    /// the repository's data: the arzakon deck (Cali Commander, WUBRG, commander Arzakon)
    #[test]
    fn check_refuses_a_wrong_old_commander_and_an_unknown_or_illegal_new_one() {
        let s = sim();
        let k = "Arzakon";
        let e = check(&s, "arzakon", "Sheoldred, the Apocalypse", "The Ur-Dragon", &[]).unwrap_err();
        assert!(e.contains(k) && e.contains("is not arzakon's commander"), "{e}");
        let e = check(&s, "arzakon", k, "No Such Card", &[]).unwrap_err();
        assert!(e.contains("not in the engine's card data"), "{e}");
        // not a legendary creature
        let e = check(&s, "arzakon", k, "Swiftfoot Boots", &[]).unwrap_err();
        assert!(e.contains("is not a legendary creature"), "{e}");
        // Atraxa is WUBG: the red cards of the list are outside her identity
        let e = check(&s, "arzakon", k, "Atraxa, Praetors' Voice", &[]).unwrap_err();
        assert!(e.contains("outside colour identity"), "{e}");
        // a commander already in the 99
        let card = s.mine["arzakon"].iter().find(|c| *c != k && c.as_str() != "Sol Ring").unwrap().clone();
        let e = check(&s, "arzakon", k, &card, &[]).unwrap_err();
        assert!(e.contains("already in the 99") || e.contains("not legal"), "{e}");
    }

    #[test]
    fn check_accepts_a_legal_five_colour_commander_and_combines_with_swaps() {
        let s = sim();
        let k = "Arzakon";
        let base = &s.mine["arzakon"];
        let var = check(&s, "arzakon", k, "The Ur-Dragon", &[]).unwrap();
        let ci = base.iter().position(|c| c == k).unwrap();
        assert_eq!(var[ci], "The Ur-Dragon");
        assert!(base.iter().zip(&var).enumerate().all(|(i, (b, v))| i == ci || b == v));
        // a swap whose card in is the old commander, applied to the 99 of the variant
        let out = base.iter().find(|c| c.as_str() != k && !s.engine.db.get(s.engine.db.id(c).unwrap()).land).unwrap();
        let swaps = pairs(&[(out, k)]);
        let var = check(&s, "arzakon", k, "The Ur-Dragon", &swaps).unwrap();
        let j = base.iter().position(|c| c == out).unwrap();
        assert_eq!((var[ci].as_str(), var[j].as_str()), ("The Ur-Dragon", k));
        // the old commander comes back as a card of the 99 in the same run: The Ur-Dragon commands, Arzakon takes the
        // slot of a card of the list
        let golem = "Meteor Golem";
        assert!(base.iter().any(|c| c == golem) && !base.iter().any(|c| c == "The Ur-Dragon"));
        let var = check(&s, "arzakon", k, "The Ur-Dragon", &pairs(&[(golem, k)])).unwrap();
        let j = base.iter().position(|c| c == golem).unwrap();
        assert_eq!((var[ci].as_str(), var[j].as_str()), ("The Ur-Dragon", k));
        // a swap that brings the new commander into the 99 too is the singleton clash: refused
        let err = check(&s, "arzakon", k, "The Ur-Dragon", &pairs(&[(golem, "The Ur-Dragon")])).unwrap_err();
        assert!(err.contains("already in the 99"), "{err}");
    }

    /// the library of a seat is its list without the commander's slot: the variant's 99 sit where the base's do, so
    /// the same seed deals both lists the same library (the paired-run property of `--swap`)
    #[test]
    fn the_variant_seat_has_the_same_library_as_the_base_seat() {
        let s = sim();
        let k = "Arzakon";
        let base = s.seat("arzakon", &s.mine["arzakon"]).unwrap();
        let var_list = check(&s, "arzakon", k, "The Ur-Dragon", &[]).unwrap();
        let var = s.seat_with_commander("arzakon", &var_list, Some("The Ur-Dragon")).unwrap();
        assert_eq!(base.commander, s.engine.db.id(k).unwrap());
        assert_eq!(var.commander, s.engine.db.id("The Ur-Dragon").unwrap());
        let library = |seat: &sim_core::engine::turn::Seat| {
            let mut lib: Vec<_> = seat.cards.to_vec();
            let i = lib.iter().position(|&c| c == seat.commander).unwrap();
            lib.remove(i);
            lib
        };
        assert_eq!(library(&base), library(&var));
        assert_eq!(library(&base).len(), 99);
        // with a swap in the 99 the libraries differ at that one position only
        let out = s.mine["arzakon"].iter().find(|c| c.as_str() != k && s.engine.db.id(c).is_some()).unwrap().clone();
        let var_list = check(&s, "arzakon", k, "The Ur-Dragon", &pairs(&[(&out, "Swiftfoot Boots")])).unwrap();
        let var = s.seat_with_commander("arzakon", &var_list, Some("The Ur-Dragon")).unwrap();
        let diff = library(&base).iter().zip(library(&var)).filter(|(a, b)| *a != b).count();
        assert_eq!(diff, 1);
        // the old commander back as a card of the 99 in the same run (Arzakon takes Meteor Golem's slot): one position
        // differs, the length is 99
        let golem = "Meteor Golem";
        let var_list = check(&s, "arzakon", k, "The Ur-Dragon", &pairs(&[(golem, k)])).unwrap();
        let var = s.seat_with_commander("arzakon", &var_list, Some("The Ur-Dragon")).unwrap();
        assert_eq!(var.commander, s.engine.db.id("The Ur-Dragon").unwrap());
        assert_eq!(library(&var).len(), 99);
        let diff = library(&base).iter().zip(library(&var)).filter(|(a, b)| *a != b).count();
        assert_eq!(diff, 1);
    }
}
