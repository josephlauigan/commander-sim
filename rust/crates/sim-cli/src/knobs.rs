//! The AI's tunable values as data. The fitted knobs of t3, t5 and t6 (`data/knobs/tN.toml`, `Defaults`) are the
//! defaults since 2026-11-02; `--knobs builtin` drops them, and a knobs file (`--knobs FILE`, written by `--fit`)
//! replaces them for a run. A file overrides the hand-set values
//! in sim-core (each deck's style, the counter thresholds, the removal eagerness, the look-ahead's weights). A value
//! the file leaves out keeps its built-in one, so an empty file, or one that repeats the built-in values, changes
//! nothing (tests below, and `audit/realism/knobs-identity-*.txt`).
//!
//! ```toml
//! tier = "t6"            # optional: the tier it was fitted on
//! [table]
//! temp_scale = 1.0       # every deck's temperature times this (as --temp)
//! cthresh_default = 6    # outside decks' counter threshold (importance a spell needs to be countered)
//! instant_extra = 0      # extra importance instant removal needs before it is used proactively
//! t6_moxd_lands = 3      # Tier 6: lands (hand + battlefield, the discard included) needed to cast Mox Diamond
//! [ai]
//! planner = false        # --- ai-planner: the Tier 6 line planner off (the step 1 AI; env PLANNER=0 too)
//! [cthresh]
//! sauron = 6              # one of your decks' counter threshold
//! [search]               # the look-ahead's knobs (no effect with --ai adaptive)
//! combo_w = 12.0
//! [style.kinnan-bonder-mana-combo]
//! temp = 1.0
//! aggression = 0.6
//! caution = 0.5
//! [fit]                  # what --fit wrote (read by people, ignored by the program)
//! ```

use sim_core::ai::Style;
use sim_core::settings::Settings;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct StyleKnob {
    pub temp: Option<f64>,
    pub aggression: Option<f64>,
    pub caution: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SearchKnobs {
    pub combo_w: Option<f64>,
    pub ramp_w: Option<f64>,
    pub hold_w: Option<f64>,
    pub top_k: Option<usize>,
    pub rollouts: Option<u32>,
    pub horizon: Option<u32>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Knobs {
    pub tier: Option<String>,
    pub temp_scale: Option<f64>,
    pub cthresh_default: Option<i32>,
    pub instant_extra: Option<i32>,
    pub t6_moxd_lands: Option<usize>,
    /// --- ai-planner: `[ai] planner`
    pub planner: Option<bool>,
    pub cthresh: BTreeMap<String, i32>,
    pub search: SearchKnobs,
    pub style: BTreeMap<String, StyleKnob>,
}

/// the range each style value may take (aggression and caution are weights in [0, 1] in the built-in tables)
pub const TEMP_RANGE: (f64, f64) = (0.05, 3.0);
pub const UNIT_RANGE: (f64, f64) = (0.0, 1.0);
pub const CTHRESH_RANGE: (i32, i32) = (0, 20);
pub const INSTANT_EXTRA_RANGE: (i32, i32) = (0, 10);

fn float(v: &toml::Value, what: &str, (lo, hi): (f64, f64)) -> Result<f64, String> {
    let x = match v {
        toml::Value::Float(f) => *f,
        toml::Value::Integer(i) => *i as f64,
        _ => return Err(format!("{what} must be a number")),
    };
    if !(lo..=hi).contains(&x) {
        return Err(format!("{what} = {x} is outside {lo}..{hi}"));
    }
    Ok(x)
}

fn int(v: &toml::Value, what: &str, (lo, hi): (i32, i32)) -> Result<i32, String> {
    let x = v.as_integer().ok_or_else(|| format!("{what} must be a whole number"))?;
    if x < lo as i64 || x > hi as i64 {
        return Err(format!("{what} = {x} is outside {lo}..{hi}"));
    }
    Ok(x as i32)
}

fn table<'a>(v: &'a toml::Value, what: &str) -> Result<&'a toml::Table, String> {
    v.as_table().ok_or_else(|| format!("{what} must be a table ([{what}])"))
}

fn only(t: &toml::Table, what: &str, known: &[&str]) -> Result<(), String> {
    match t.keys().find(|k| !known.contains(&k.as_str())) {
        Some(k) => Err(format!("[{what}] has an unknown key `{k}` (known: {})", known.join(", "))),
        None => Ok(()),
    }
}

/// a knobs file; `decks`: the deck keys it may name ([cthresh] and [style.*])
pub fn parse(text: &str, file: &str, decks: &[&str]) -> Result<Knobs, String> {
    let at = |e: String| format!("{file}: {e}");
    let t: toml::Table = text.parse().map_err(|e| at(format!("not valid TOML: {e}")))?;
    let mut k = Knobs::default();
    let deck = |key: &str, section: &str| -> Result<(), String> {
        if decks.contains(&key) {
            Ok(())
        } else {
            Err(format!("[{section}] names `{key}`, which is not a deck (data/decks.json)"))
        }
    };
    for (key, v) in &t {
        match key.as_str() {
            "tier" => k.tier = Some(v.as_str().ok_or_else(|| at("tier must be a string".into()))?.to_string()),
            "note" | "fit" => {}
            "table" => {
                let tb = table(v, "table").map_err(at)?;
                only(tb, "table", &["temp_scale", "cthresh_default", "instant_extra", "t6_moxd_lands"]).map_err(at)?;
                if let Some(x) = tb.get("temp_scale") {
                    k.temp_scale = Some(float(x, "temp_scale", TEMP_RANGE).map_err(at)?);
                }
                if let Some(x) = tb.get("cthresh_default") {
                    k.cthresh_default = Some(int(x, "cthresh_default", CTHRESH_RANGE).map_err(at)?);
                }
                if let Some(x) = tb.get("instant_extra") {
                    k.instant_extra = Some(int(x, "instant_extra", INSTANT_EXTRA_RANGE).map_err(at)?);
                }
                if let Some(x) = tb.get("t6_moxd_lands") {
                    k.t6_moxd_lands = Some(int(x, "t6_moxd_lands", (1, 6)).map_err(at)? as usize);
                }
            }
            // --- ai-planner
            "ai" => {
                let tb = table(v, "ai").map_err(at)?;
                only(tb, "ai", &["planner"]).map_err(at)?;
                if let Some(x) = tb.get("planner") {
                    k.planner = Some(x.as_bool().ok_or_else(|| at("planner must be true or false".into()))?);
                }
            }
            "cthresh" => {
                for (d, x) in table(v, "cthresh").map_err(at)? {
                    deck(d, "cthresh").map_err(at)?;
                    k.cthresh.insert(d.clone(), int(x, &format!("cthresh.{d}"), CTHRESH_RANGE).map_err(at)?);
                }
            }
            "search" => {
                let tb = table(v, "search").map_err(at)?;
                only(tb, "search", &["combo_w", "ramp_w", "hold_w", "top_k", "rollouts", "horizon"]).map_err(at)?;
                let s = &mut k.search;
                for (name, slot) in [("combo_w", &mut s.combo_w), ("ramp_w", &mut s.ramp_w), ("hold_w", &mut s.hold_w)] {
                    if let Some(x) = tb.get(name) {
                        *slot = Some(float(x, name, (0.0, 1000.0)).map_err(at)?);
                    }
                }
                if let Some(x) = tb.get("top_k") {
                    s.top_k = Some(int(x, "top_k", (1, 50)).map_err(at)? as usize);
                }
                if let Some(x) = tb.get("rollouts") {
                    s.rollouts = Some(int(x, "rollouts", (1, 200)).map_err(at)? as u32);
                }
                if let Some(x) = tb.get("horizon") {
                    s.horizon = Some(int(x, "horizon", (1, 10)).map_err(at)? as u32);
                }
            }
            "style" => {
                for (d, sv) in table(v, "style").map_err(at)? {
                    deck(d, "style").map_err(at)?;
                    let tb = table(sv, &format!("style.{d}")).map_err(at)?;
                    only(tb, &format!("style.{d}"), &["temp", "aggression", "caution"]).map_err(at)?;
                    let get = |name: &str, r| tb.get(name).map(|x| float(x, &format!("style.{d}.{name}"), r)).transpose();
                    let sk = StyleKnob {
                        temp: get("temp", TEMP_RANGE).map_err(at)?,
                        aggression: get("aggression", UNIT_RANGE).map_err(at)?,
                        caution: get("caution", UNIT_RANGE).map_err(at)?,
                    };
                    k.style.insert(d.clone(), sk);
                }
            }
            _ => {
                return Err(at(format!(
                    "unknown key `{key}` (known: tier, note, [table], [ai], [cthresh], [search], [style.<deck>], [fit])"
                )));
            }
        }
    }
    Ok(k)
}

impl Knobs {
    /// these knobs on top of a run's settings (values left out keep what the settings hold)
    pub fn apply(&self, st: &mut Settings) {
        if let Some(x) = self.temp_scale {
            st.temp_scale = x;
        }
        if let Some(x) = self.cthresh_default {
            st.cthresh_default = x;
        }
        if let Some(x) = self.instant_extra {
            st.instant_extra = x;
        }
        if let Some(x) = self.t6_moxd_lands {
            st.t6_moxd_lands = x;
        }
        if let Some(x) = self.planner {
            st.planner = x; // --- ai-planner
        }
        for (d, x) in &self.cthresh {
            st.cthresh.insert(d.clone(), *x);
        }
        let s = &self.search;
        let ss = &mut st.search;
        if let Some(x) = s.combo_w {
            ss.combo_w = x;
        }
        if let Some(x) = s.ramp_w {
            ss.ramp_w = x;
        }
        if let Some(x) = s.hold_w {
            ss.hold_w = x;
        }
        if let Some(x) = s.top_k {
            ss.top_k = x;
        }
        if let Some(x) = s.rollouts {
            ss.rollouts = x;
        }
        if let Some(x) = s.horizon {
            ss.horizon = x;
        }
        for (d, sk) in &self.style {
            let base = st.styles.get(d).copied().unwrap_or_else(|| sim_core::ai::base_style(d));
            let style = Style {
                temp: sk.temp.unwrap_or(base.temp),
                aggression: sk.aggression.unwrap_or(base.aggression),
                caution: sk.caution.unwrap_or(base.caution),
            };
            st.styles.insert(d.clone(), style);
        }
    }

    /// the file `parse` reads back; `header` lines become comments at the top, `fit` lines the [fit] table
    pub fn to_toml(&self, header: &[String], fit: &[(String, String)]) -> String {
        let mut o = String::new();
        for h in header {
            o.push_str(&format!("# {h}\n"));
        }
        if !header.is_empty() {
            o.push('\n');
        }
        if let Some(t) = &self.tier {
            o.push_str(&format!("tier = \"{t}\"\n\n"));
        }
        let mut tb = vec![];
        if let Some(x) = self.temp_scale {
            tb.push(format!("temp_scale = {x:.4}"));
        }
        if let Some(x) = self.cthresh_default {
            tb.push(format!("cthresh_default = {x}"));
        }
        if let Some(x) = self.instant_extra {
            tb.push(format!("instant_extra = {x}"));
        }
        if let Some(x) = self.t6_moxd_lands {
            tb.push(format!("t6_moxd_lands = {x}"));
        }
        if !tb.is_empty() {
            o.push_str(&format!("[table]\n{}\n\n", tb.join("\n")));
        }
        if let Some(x) = self.planner {
            o.push_str(&format!("[ai]\nplanner = {x}\n\n")); // --- ai-planner
        }
        if !self.cthresh.is_empty() {
            o.push_str("[cthresh]\n");
            for (d, x) in &self.cthresh {
                o.push_str(&format!("{d} = {x}\n"));
            }
            o.push('\n');
        }
        let s = &self.search;
        let mut sl = vec![];
        for (n, x) in [("combo_w", s.combo_w), ("ramp_w", s.ramp_w), ("hold_w", s.hold_w)] {
            if let Some(x) = x {
                sl.push(format!("{n} = {x:.4}"));
            }
        }
        if let Some(x) = s.top_k {
            sl.push(format!("top_k = {x}"));
        }
        if let Some(x) = s.rollouts {
            sl.push(format!("rollouts = {x}"));
        }
        if let Some(x) = s.horizon {
            sl.push(format!("horizon = {x}"));
        }
        if !sl.is_empty() {
            o.push_str(&format!("[search]\n{}\n\n", sl.join("\n")));
        }
        for (d, sk) in &self.style {
            o.push_str(&format!("[style.{d}]\n"));
            for (n, x) in [("temp", sk.temp), ("aggression", sk.aggression), ("caution", sk.caution)] {
                if let Some(x) = x {
                    o.push_str(&format!("{n} = {x:.4}\n"));
                }
            }
            o.push('\n');
        }
        if !fit.is_empty() {
            o.push_str("[fit]\n");
            for (k, v) in fit {
                o.push_str(&format!("{k} = {v}\n"));
            }
        }
        o
    }
}

/// The fitted knobs that are the defaults (JD, 2026-11-02): `data/knobs/tN.toml`, one file per tier whose fit beat the
/// built-in values (t3, t5, t6). A run that gets `--knobs FILE` uses the file instead; `--knobs builtin` uses neither.
///
/// - **Styles** (`[style.<deck>]`): every pool deck gets the style of its own tier's file, whatever table it sits at.
///   Decks in no file (yours, and the tiers with no file) keep their built-in style.
/// - **Table knobs** (`[table]`) are whole-table settings, so one tier's file supplies them: the table's tier, the tier
///   most of the table's seats belong to (`majority_tier`). Every mode names its opponents' pool (`--pool tN`, your deck
///   against tN, a tN+1 deck tested into tN: all tN), so the rule was never ambiguous; a tie between tiers (no mode
///   makes one) goes to the lowest tier. A tier with no file gives the built-in table knobs.
#[derive(Debug, Clone, Default)]
pub struct Defaults {
    by_tier: BTreeMap<String, Knobs>,
}

/// the tier most of these seats' decks belong to (a tie: the lowest tier); None when none of them is a pool deck
pub fn majority_tier<'a>(tiers: impl IntoIterator<Item = &'a str>) -> Option<String> {
    let mut n: BTreeMap<&str, usize> = BTreeMap::new();
    for t in tiers {
        *n.entry(t).or_default() += 1;
    }
    let top = n.values().copied().max()?;
    n.into_iter().find(|(_, c)| *c == top).map(|(t, _)| t.to_string())
}

impl Defaults {
    /// the built-in values everywhere (`--knobs builtin`)
    pub fn builtin() -> Defaults {
        Defaults::default()
    }

    /// every `tN.toml` in `dir` (data/knobs); `pool`: (deck key, tier) of the pool decks, which a file's styles must
    /// name and which must sit in the file's own tier
    pub fn load(dir: &std::path::Path, pool: &[(&str, &str)], decks: &[&str]) -> Result<Defaults, String> {
        let mut by_tier = BTreeMap::new();
        let rd = std::fs::read_dir(dir)
            .map_err(|e| format!("{}: {e} (the default knobs; --knobs builtin skips them)", dir.display()))?;
        let mut paths: Vec<_> = rd.flatten().map(|f| f.path()).collect();
        paths.sort();
        for p in paths {
            let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("").to_string();
            if p.extension().is_none_or(|x| x != "toml") {
                continue;
            }
            let file = p.display().to_string();
            let text = std::fs::read_to_string(&p).map_err(|e| format!("{file}: {e}"))?;
            by_tier.insert(stem.clone(), Self::check(&text, &stem, &file, pool, decks)?);
        }
        Ok(Defaults { by_tier })
    }

    /// parse one default file and check it is a pure tier file (styles and table knobs only, its own tier's decks)
    pub fn check(text: &str, tier: &str, file: &str, pool: &[(&str, &str)], decks: &[&str]) -> Result<Knobs, String> {
        let k = parse(text, file, decks)?;
        if k.tier.as_deref() != Some(tier) {
            return Err(format!("{file}: tier = {:?}, but this is the default file for {tier}", k.tier));
        }
        if !k.cthresh.is_empty() || k.search != SearchKnobs::default() || k.planner.is_some() {
            return Err(format!("{file}: a default file holds [table] and [style.*] only"));
        }
        for d in k.style.keys() {
            if !pool.iter().any(|(key, t)| key == d && *t == tier) {
                return Err(format!("{file}: [style.{d}] is not a {tier} pool deck"));
            }
        }
        Ok(k)
    }

    /// the tiers that have a default file
    #[cfg(test)]
    pub fn tiers(&self) -> Vec<&str> {
        self.by_tier.keys().map(|t| t.as_str()).collect()
    }

    /// the defaults on top of a run's settings; `table_tier`: the tier whose `[table]` knobs apply (None: built-in)
    pub fn apply(&self, st: &mut Settings, table_tier: Option<&str>) {
        for (tier, k) in &self.by_tier {
            let mut k = k.clone();
            if Some(tier.as_str()) != table_tier {
                (k.temp_scale, k.cthresh_default, k.instant_extra, k.t6_moxd_lands) = (None, None, None, None);
            }
            k.apply(st);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::settings::{AiMode, Profile};

    const DECKS: [&str; 3] = ["sauron", "kinnan-bonder-mana-combo", "sisay-legends-toolbox"];

    fn p(s: &str) -> Result<Knobs, String> {
        parse(s, "k.toml", &DECKS)
    }

    #[test]
    fn an_empty_file_changes_nothing() {
        let base = Settings::new(Profile::Loose, AiMode::Adaptive, 1.0);
        let mut st = base.clone();
        p("").unwrap().apply(&mut st);
        assert_eq!(format!("{st:?}"), format!("{base:?}"));
    }

    // --- ai-planner
    #[test]
    fn the_planner_knob_is_read_applied_and_written() {
        let k = p("[ai]\nplanner = false\n").unwrap();
        assert_eq!(k.planner, Some(false));
        let mut st = Settings::new(Profile::Loose, AiMode::Adaptive, 1.0);
        st.planner = true;
        k.apply(&mut st);
        assert!(!st.planner);
        assert_eq!(p(&k.to_toml(&[], &[])).unwrap(), k);
        assert!(p("[ai]\nplanner = 1\n").is_err());
        assert!(p("[ai]\nplaner = false\n").is_err());
    }

    #[test]
    fn a_complete_file_is_read_and_applied() {
        let k = p("tier = \"t6\"\nnote = \"x\"\n[table]\ntemp_scale = 0.8\ncthresh_default = 5\ninstant_extra = 1\n\
                   [cthresh]\nsauron = 7\n[search]\ncombo_w = 10\ntop_k = 4\n\
                   [style.kinnan-bonder-mana-combo]\naggression = 0.9\n[fit]\nloss = 1.5\n")
        .unwrap();
        let mut st = Settings::new(Profile::Loose, AiMode::Adaptive, 1.0);
        k.apply(&mut st);
        assert_eq!((st.temp_scale, st.cthresh_default, st.instant_extra), (0.8, 5, 1));
        assert_eq!(st.counter_threshold("sauron"), Some(7));
        assert_eq!((st.search.combo_w, st.search.top_k), (10.0, 4));
        let base = sim_core::ai::base_style("kinnan-bonder-mana-combo");
        let s = st.styles["kinnan-bonder-mana-combo"];
        assert_eq!((s.temp, s.aggression, s.caution), (base.temp, 0.9, base.caution));
    }

    #[test]
    fn written_knobs_read_back_the_same() {
        let mut k = Knobs { tier: Some("t6".into()), cthresh_default: Some(5), instant_extra: Some(0), ..Default::default() };
        k.style.insert(
            "sisay-legends-toolbox".into(),
            StyleKnob { temp: Some(1.25), aggression: Some(0.5), caution: Some(0.125) },
        );
        let text = k.to_toml(&["fitted".into()], &[("loss".into(), "2.0".into())]);
        assert_eq!(p(&text).unwrap(), k);
    }

    #[test]
    fn bad_files_are_refused_with_a_reason() {
        for (text, want) in [
            ("[tabel]\n", "unknown key `tabel`"),
            ("[table]\ncthresh = 5\n", "unknown key `cthresh`"),
            ("[table]\ninstant_extra = 1.5\n", "whole number"),
            ("[table]\ncthresh_default = 99\n", "outside"),
            ("[style.nobody]\ntemp = 1\n", "not a deck"),
            ("[style.sauron]\nboldness = 1\n", "unknown key `boldness`"),
            ("[style.sauron]\naggression = 1.5\n", "outside"),
            ("[cthresh]\nnobody = 6\n", "not a deck"),
            ("[search]\ndepth = 3\n", "unknown key `depth`"),
            ("x = [", "not valid TOML"),
        ] {
            let e = p(text).unwrap_err();
            assert!(e.contains(want), "{text:?}: {e}");
        }
    }
}

/// the fitted knobs as defaults (`data/knobs/tN.toml`), against the repository's decks
#[cfg(test)]
mod default_tests {
    use super::*;
    use crate::play::{RunOpts, Sim};
    use sim_core::settings::{AiMode, Profile};

    fn sim() -> Sim {
        Sim::load(RunOpts { ai: AiMode::Lookahead, temp: 1.0, jobs: 2 }).unwrap()
    }

    fn builtin() -> Settings {
        Settings::new(Profile::Loose, AiMode::Lookahead, 1.0)
    }

    /// the table knobs of a run's settings
    fn table(st: &Settings) -> (i32, i32, usize, f64) {
        (st.cthresh_default, st.instant_extra, st.t6_moxd_lands, st.temp_scale)
    }

    /// a tier's default file, read again by the test
    fn file(sim: &Sim, tier: &str) -> Knobs {
        let path = crate::decks::root().unwrap().join("data").join("knobs").join(format!("{tier}.toml"));
        parse(&std::fs::read_to_string(&path).unwrap(), tier, &sim.deck_keys()).unwrap()
    }

    #[test]
    fn exactly_t3_t5_and_t6_have_default_files() {
        assert_eq!(sim().defaults.tiers(), ["t3", "t5", "t6"]);
    }

    #[test]
    fn a_pool_deck_gets_the_style_of_its_own_tiers_file_at_any_table() {
        let sim = sim();
        for tier in ["t3", "t5", "t6"] {
            let f = file(&sim, tier);
            assert!(f.style.len() >= 5, "{tier}");
            for table_tier in [Some(tier), Some("t4"), Some("t6"), None] {
                let st = sim.settings_in(Profile::Loose, table_tier);
                for (deck, want) in &f.style {
                    let got = st.styles[deck];
                    let base = sim_core::ai::base_style(deck);
                    assert_eq!(got.temp, want.temp.unwrap_or(base.temp), "{deck} at {table_tier:?}");
                    assert_eq!(got.aggression, want.aggression.unwrap_or(base.aggression), "{deck} at {table_tier:?}");
                    assert_eq!(got.caution, want.caution.unwrap_or(base.caution), "{deck} at {table_tier:?}");
                }
            }
        }
    }

    #[test]
    fn a_t4_deck_and_yours_stay_built_in() {
        let sim = sim();
        let st = sim.settings_vs(Profile::Loose, &sim.pool_keys("t4"));
        for tier in ["t0", "t1", "t2", "t4"] {
            for d in sim.pool_keys(tier) {
                assert!(!st.styles.contains_key(&d), "{d}");
            }
        }
        for d in sim.mine.keys() {
            assert!(!st.styles.contains_key(d), "{d}");
        }
        // and a table of t4 decks has the built-in table knobs
        assert_eq!(table(&st), table(&builtin()));
    }

    #[test]
    fn the_table_takes_the_knobs_of_the_tier_most_seats_belong_to() {
        let sim = sim();
        let fit = |t: &str| {
            let k = file(&sim, t);
            (
                k.cthresh_default.unwrap(),
                k.instant_extra.unwrap(),
                k.t6_moxd_lands.unwrap_or(builtin().t6_moxd_lands),
                k.temp_scale.unwrap_or(1.0),
            )
        };
        for t in ["t3", "t5", "t6"] {
            assert_eq!(table(&sim.settings_vs(Profile::Loose, &sim.pool_keys(t))), fit(t), "{t}");
        }
        assert_ne!(fit("t3"), fit("t5"));
        // the decks of a tier are not the table's tier when the others are more: 3 t5 decks and 2 t3 decks give t5
        let mix = |a: &str, na: usize, b: &str, nb: usize| {
            let mut v: Vec<String> = sim.pool_keys(a).into_iter().take(na).collect();
            v.extend(sim.pool_keys(b).into_iter().take(nb));
            v
        };
        assert_eq!(sim.table_tier(&mix("t5", 3, "t3", 2)).as_deref(), Some("t5"));
        assert_eq!(table(&sim.settings_vs(Profile::Loose, &mix("t5", 3, "t3", 2))), fit("t5"));
        assert_eq!(table(&sim.settings_vs(Profile::Loose, &mix("t3", 3, "t5", 2))), fit("t3"));
        // a tier with no file, and no pool decks at all: the built-in table knobs
        assert_eq!(table(&sim.settings_vs(Profile::Loose, &mix("t4", 3, "t3", 2))), table(&builtin()));
        assert_eq!(table(&sim.settings_vs(Profile::Loose, &[])), table(&builtin()));
        // a tie goes to the lowest tier
        assert_eq!(majority_tier(["t5", "t3", "t5", "t3"]).as_deref(), Some("t3"));
        assert_eq!(majority_tier(["t6", "t5", "t5"]).as_deref(), Some("t5"));
        assert_eq!(majority_tier([]), None);
    }

    #[test]
    fn knobs_builtin_gives_the_built_in_values() {
        let mut sim = sim();
        sim.defaults = Defaults::builtin();
        for tier in ["t3", "t5", "t6"] {
            let st = sim.settings_vs(Profile::Loose, &sim.pool_keys(tier));
            // (the debug text would differ in map order)
            assert_eq!(table(&st), table(&builtin()), "{tier}");
            assert!(st.styles.is_empty(), "{tier}");
            assert_eq!(st.cthresh, builtin().cthresh, "{tier}");
            assert_eq!(st.search.combo_w, builtin().search.combo_w, "{tier}");
        }
    }

    #[test]
    fn knobs_file_replaces_the_defaults() {
        let mut sim = sim();
        let keys = sim.deck_keys();
        sim.knobs = Some(parse("[table]\ncthresh_default = 7\n", "k.toml", &keys).unwrap());
        let st = sim.settings_vs(Profile::Loose, &sim.pool_keys("t3"));
        // the file's value, and nothing of t3's default file: its instant_extra and its styles
        assert_eq!(st.cthresh_default, 7);
        assert_eq!(st.instant_extra, builtin().instant_extra);
        assert!(st.styles.is_empty());
        // a file that styles a deck is the only source of that deck's style
        sim.knobs = Some(parse("[style.atraxa-infect]\ntemp = 1.5\n", "k.toml", &sim.deck_keys()).unwrap());
        let st = sim.settings_vs(Profile::Loose, &sim.pool_keys("t5"));
        assert_eq!(st.styles["atraxa-infect"].temp, 1.5);
        assert_eq!(st.styles.len(), 1);
        assert_eq!(table(&st), table(&builtin()));
    }

    #[test]
    fn a_default_file_is_refused_when_it_strays() {
        let pool = [("atraxa-infect", "t3"), ("urza-mono-blue-artifacts", "t5")];
        let decks = ["atraxa-infect", "urza-mono-blue-artifacts"];
        let ok = "tier = \"t3\"\n[table]\ninstant_extra = 2\n[style.atraxa-infect]\ntemp = 1.0\n";
        assert!(Defaults::check(ok, "t3", "t3.toml", &pool, &decks).is_ok());
        for (text, want) in [
            ("tier = \"t5\"\n", "default file for t3"),
            ("tier = \"t3\"\n[cthresh]\natraxa-infect = 5\n", "[table] and [style.*] only"),
            ("tier = \"t3\"\n[style.urza-mono-blue-artifacts]\ntemp = 1.0\n", "not a t3 pool deck"),
        ] {
            let e = Defaults::check(text, "t3", "t3.toml", &pool, &decks).unwrap_err();
            assert!(e.contains(want), "{text:?}: {e}");
        }
    }
}
