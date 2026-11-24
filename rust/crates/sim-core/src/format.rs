//! The rules a format changes. `Settings.rules` carries them; the engine and the cards read them (a player's
//! `start_life`, the starting player's first draw), so a format is data, not a branch on a name. A `Format` is the
//! record data/decks.json carries for each format (written from data/formats.toml).

use crate::export::RawDeck;

/// the format-dependent rules a game is played under
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rules {
    /// every player's starting life total (Commander 40)
    pub life: i32,
    /// whether the starting player (seat 0) draws on its first turn. False skips that draw step (CR 103.8a).
    pub first_player_draws: bool,
}

impl Rules {
    /// Commander as this simulator has always played it. `first_player_draws: true` is deliberately the old 4-seat
    /// rule even for a 2-seat Commander game (CR 103.8a would skip it): do not "fix" it, every Commander trace and
    /// number depends on it.
    pub const COMMANDER: Rules = Rules { life: 40, first_player_draws: true };
}

impl Default for Rules {
    fn default() -> Rules {
        Rules::COMMANDER
    }
}

// ------------------------------------------------------------------------------------------ the AI's table profile

/// whose catalogued lines the hold-up-or-go gates (`t6_table::go_ok`) apply to
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GoRules {
    /// Tier 6 seats only (`t6_plans::is_t6`): today's play
    #[default]
    T6,
    /// every seat
    All,
}

/// What the shared AI reads of the table it plays at (`Settings.table`). Commander's values are today's play exactly;
/// a format record can change them (`[cali.ai]` in data/formats.toml). `deny_unknown_fields`: a field this reader does
/// not know is an error. `duel` is not a data field: the loader derives it from the format's seats (`formats_table`),
/// never from the live player count, so a 4-player endgame at a Commander table plays as it always has.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct TableAi {
    /// the look-ahead scores positions with the learned evaluation (fitted on 4-seat, 40-life games) for generic decks
    pub learned_eval: bool,
    /// whose catalogued lines wait for the table to be unable to stop them
    pub go_rules: GoRules,
    /// one opponent at the table (the format has 2 seats): Silence and Orim's Chant are equal cover
    #[serde(skip)]
    pub duel: bool,
}

impl TableAi {
    /// Commander as the AI has always played it
    pub const COMMANDER: TableAi = TableAi { learned_eval: true, go_rules: GoRules::T6, duel: false };

    /// The field names `TABLE_AI` may name.
    pub const FIELDS: [&'static str; 3] = ["learned_eval", "go_rules", "duel"];

    /// This profile with the fields `spec` names turned off, that is back at Commander's value (a measurement arm:
    /// env `TABLE_AI`). `spec` is `commander` (every field) or a comma list of field names; empty changes nothing.
    pub fn turned_off(self, spec: &str) -> Result<TableAi, String> {
        let spec = spec.trim();
        if spec.is_empty() {
            return Ok(self);
        }
        let mut out = self;
        let all = spec == "commander";
        let names: Vec<&str> = if all { Self::FIELDS.to_vec() } else { spec.split(',').map(str::trim).collect() };
        for n in names {
            match n {
                "learned_eval" => out.learned_eval = Self::COMMANDER.learned_eval,
                "go_rules" => out.go_rules = Self::COMMANDER.go_rules,
                "duel" => out.duel = Self::COMMANDER.duel,
                _ => {
                    return Err(format!(
                        "bad TABLE_AI {spec:?}: `commander` or a comma list of {}",
                        Self::FIELDS.join(", ")
                    ));
                }
            }
        }
        Ok(out)
    }

    /// the profile as a report names it: `Commander's (4-player)`, or each field's value
    pub fn describe(&self) -> String {
        if *self == Self::COMMANDER {
            return "Commander's (4-player)".into();
        }
        format!(
            "learned eval {}, go rules {}, duel cover {}",
            if self.learned_eval { "on" } else { "off" },
            if self.go_rules == GoRules::All { "every seat" } else { "Tier 6 only" },
            if self.duel { "on" } else { "off" }
        )
    }
}

impl Default for TableAi {
    fn default() -> TableAi {
        TableAi::COMMANDER
    }
}

// ------------------------------------------------------------------------------------------ the format record

/// the key of the format a deck plays when its record names none
pub const DEFAULT_FORMAT: &str = "commander";

/// how a format decides which cards are banned (cardgen reads this; sim-core only checks a list format's list)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Bans {
    /// Scryfall's `legalities.commander`
    #[default]
    Scryfall,
    /// the format's own `banned` list replaces Commander's
    List,
}

/// A game format: a record of data/decks.json "formats" (written from data/formats.toml). `deny_unknown_fields`: a
/// field this reader does not know is an error, not silently ignored.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Format {
    /// its key in the formats object; filled in by `Engine::load`, not stored in the record
    #[serde(skip)]
    pub key: String,
    /// what messages call it ("Cali Commander")
    pub name: String,
    /// players at its table: 2 or 4 (the pod builders' shapes; the engine itself never reads it)
    pub seats: usize,
    pub life: i32,
    pub first_player_draws: bool,
    #[serde(default)]
    pub bans: Bans,
    /// the ban list when `bans` is `List`: front-face names
    #[serde(default)]
    pub banned: Vec<String>,
    /// what the AI reads of the table (`[<key>.ai]`); an old decks.json without it reads as Commander's. Its `duel`
    /// is filled in by `formats_table` from `seats`
    #[serde(default)]
    pub ai: TableAi,
}

/// a card name's front face, lower case: how ban lists compare names
fn front_face(name: &str) -> String {
    name.split(" // ").next().unwrap_or("").trim().to_lowercase()
}

impl Format {
    /// Commander as the engine has always played it: what an old decks.json without `formats` reads as, and what
    /// `[commander]` must equal
    pub fn commander() -> Format {
        Format {
            key: DEFAULT_FORMAT.into(),
            name: "Commander".into(),
            seats: 4,
            life: Rules::COMMANDER.life,
            first_player_draws: Rules::COMMANDER.first_player_draws,
            bans: Bans::Scryfall,
            banned: vec![],
            ai: TableAi::COMMANDER,
        }
    }

    /// what the engine reads of it
    pub fn rules(&self) -> Rules {
        Rules { life: self.life, first_player_draws: self.first_player_draws }
    }

    /// the load-time rules: seats 2 or 4, life at least 1, a list format has a list (no duplicates, no `//`), and
    /// `[commander]` is exactly Commander
    pub fn validate(&self) -> Result<(), String> {
        let k = &self.key;
        if self.name.trim().is_empty() {
            return Err(format!("format {k:?}: name must not be empty"));
        }
        if self.seats != 2 && self.seats != 4 {
            return Err(format!("format {k:?}: seats must be 2 or 4, not {}", self.seats));
        }
        if self.life < 1 {
            return Err(format!("format {k:?}: life must be at least 1, not {}", self.life));
        }
        match self.bans {
            Bans::List if self.banned.is_empty() => {
                return Err(format!("format {k:?}: bans = \"list\" needs a non-empty banned list"));
            }
            Bans::Scryfall if !self.banned.is_empty() => {
                return Err(format!("format {k:?}: a banned list needs bans = \"list\""));
            }
            _ => {}
        }
        for (i, b) in self.banned.iter().enumerate() {
            if b.trim().is_empty() || b.contains("//") {
                return Err(format!("format {k:?}: banned name {b:?} must be a front-face name"));
            }
            if self.banned[..i].iter().any(|o| front_face(o) == front_face(b)) {
                return Err(format!("format {k:?}: banned name {b:?} is listed twice"));
            }
        }
        if k == DEFAULT_FORMAT {
            let c = Format::commander();
            if (self.seats, self.rules(), self.bans, self.ai) != (c.seats, c.rules(), c.bans, c.ai) {
                return Err(format!(
                    "format {k:?} must stay {} seats, {} life, first player draws, Scryfall bans and \
                     Commander's AI profile (learned_eval on, go_rules t6; the engine's Commander)",
                    c.seats,
                    Rules::COMMANDER.life
                ));
            }
        }
        Ok(())
    }

    /// The deck's own format key (`DEFAULT_FORMAT` if its record names none).
    pub fn key_of(deck: &RawDeck) -> &str {
        deck.format.as_deref().unwrap_or(DEFAULT_FORMAT)
    }

    /// May `deck`, whose own format is `own`, sit at a table of this format? Its own format, or a Commander deck
    /// with no card on a list-based ban list (names compared by front face, lower case). `Err` says why not, one
    /// reason per line.
    pub fn admits(&self, deck: &RawDeck, own: &Format) -> Result<(), Vec<String>> {
        if own.key == self.key {
            return Ok(());
        }
        if own.key != DEFAULT_FORMAT {
            return Err(vec![format!("{} plays {} (format {:?}), not {}", deck.key, own.name, own.key, self.name)]);
        }
        if self.bans == Bans::List {
            let bad: Vec<&str> = deck
                .cards
                .iter()
                .map(String::as_str)
                .filter(|c| self.banned.iter().any(|b| front_face(b) == front_face(c)))
                .collect();
            if !bad.is_empty() {
                let mut seen: Vec<&str> = vec![];
                for c in bad {
                    if !seen.contains(&c) {
                        seen.push(c);
                    }
                }
                return Err(vec![format!("{} holds cards banned in {}: {}", deck.key, self.name, seen.join(", "))]);
            }
        }
        Ok(())
    }
}

/// The formats table of a decks.json: its `formats` with each record's key filled in, plus the built-in Commander when
/// the file has none (an old file reads as Commander only). Every record is validated and every deck's format must be in
/// the table.
pub fn formats_table<'a>(
    raw: std::collections::BTreeMap<String, Format>,
    decks: impl IntoIterator<Item = &'a RawDeck>,
) -> Result<std::collections::BTreeMap<String, Format>, String> {
    let mut formats = raw;
    formats.entry(DEFAULT_FORMAT.to_string()).or_insert_with(Format::commander);
    for (k, f) in formats.iter_mut() {
        f.key = k.clone();
        f.ai.duel = f.seats == 2;
        f.validate()?;
    }
    for d in decks {
        let k = Format::key_of(d);
        if !formats.contains_key(k) {
            return Err(format!("deck {}: unknown format {k:?} (data/decks.json formats)", d.key));
        }
    }
    Ok(formats)
}

#[cfg(test)]
mod format_data_tests {
    use super::*;
    use crate::export::{DecksFile, load_decks};
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    fn data(f: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data").join(f)
    }

    fn file() -> DecksFile {
        load_decks(&data("decks.json")).unwrap()
    }

    fn deck(key: &str, format: Option<&str>, cards: &[&str]) -> RawDeck {
        RawDeck {
            tier: None,
            key: key.into(),
            name: key.into(),
            commander: "X".into(),
            commander2: None,
            ident: None,
            tiers: vec![],
            format: format.map(String::from),
            cards: cards.iter().map(|c| c.to_string()).collect(),
        }
    }

    fn cali() -> Format {
        let mut f = file().formats.remove("cali").unwrap();
        f.key = "cali".into();
        f
    }

    /// `[commander]` as decks.json carries it is exactly the engine's Commander: editing the data cannot change
    /// every 4-player game
    #[test]
    fn commander_in_decks_json_is_the_engines_commander() {
        let f = file();
        let mut c = f.formats["commander"].clone();
        c.key = "commander".into();
        assert_eq!(c, Format::commander());
        assert_eq!(c.rules(), Rules::COMMANDER);
        assert_eq!(c.seats, 4);
        assert_eq!(c.bans, Bans::Scryfall);
        let t = formats_table(f.formats, f.mine.iter().chain(&f.pool)).unwrap();
        assert_eq!(t.keys().collect::<Vec<_>>(), ["cali", "commander"]);
        assert_eq!(t["cali"].rules(), Rules { life: 30, first_player_draws: false });
        assert_eq!((t["cali"].key.as_str(), t["cali"].seats, t["cali"].bans), ("cali", 2, Bans::List));
    }

    /// decks.json `formats` equals data/formats.toml, field by field
    #[test]
    fn decks_json_formats_equal_formats_toml() {
        let src: toml::Table = std::fs::read_to_string(data("formats.toml")).unwrap().parse().unwrap();
        let got = file().formats;
        assert_eq!(got.keys().collect::<Vec<_>>(), {
            let mut k: Vec<&String> = src.keys().collect();
            k.sort();
            k
        });
        for (k, v) in &src {
            let f = &got[k];
            assert_eq!(v["name"].as_str(), Some(f.name.as_str()), "{k}");
            assert_eq!(v["seats"].as_integer(), Some(f.seats as i64), "{k}");
            assert_eq!(v["life"].as_integer(), Some(f.life as i64), "{k}");
            assert_eq!(v["first_player_draws"].as_bool(), Some(f.first_player_draws), "{k}");
            assert_eq!(v["bans"].as_str(), Some(if f.bans == Bans::List { "list" } else { "scryfall" }), "{k}");
            let banned: Vec<&str> = v
                .get("banned")
                .and_then(|b| b.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str()).collect())
                .unwrap_or_default();
            assert_eq!(banned, f.banned.iter().map(String::as_str).collect::<Vec<_>>(), "{k}");
        }
        assert_eq!(got["cali"].banned.len(), 18);
    }

    #[test]
    fn an_old_decks_json_without_formats_reads_as_commander_only() {
        let f: DecksFile = serde_json::from_str(r#"{"version":1,"mine":[],"pool":[]}"#).unwrap();
        assert!(f.formats.is_empty());
        let t = formats_table(f.formats, []).unwrap();
        assert_eq!(t.keys().collect::<Vec<_>>(), ["commander"]);
        assert_eq!(t["commander"], Format::commander());
    }

    #[test]
    fn a_record_with_a_field_this_reader_does_not_know_is_an_error() {
        let e =
            serde_json::from_str::<Format>(r#"{"name":"X","seats":2,"life":30,"first_player_draws":false,"speed":1}"#)
                .unwrap_err();
        assert!(e.to_string().contains("unknown field `speed`"), "{e}");
        // `bans` and `banned` may be left out (Scryfall's, no list)
        let f: Format = serde_json::from_str(r#"{"name":"X","seats":2,"life":30,"first_player_draws":false}"#).unwrap();
        assert_eq!((f.bans, f.banned.len()), (Bans::Scryfall, 0));
    }

    #[test]
    fn the_load_rules_refuse_a_bad_record() {
        let ok = cali();
        assert!(ok.validate().is_ok());
        let bad = |f: &dyn Fn(&mut Format)| {
            let mut x = ok.clone();
            f(&mut x);
            x.validate().unwrap_err()
        };
        assert!(bad(&|f| f.name = " ".into()).contains("name must not be empty"));
        assert!(bad(&|f| f.seats = 3).contains("seats must be 2 or 4"));
        assert!(bad(&|f| f.seats = 0).contains("seats must be 2 or 4"));
        assert!(bad(&|f| f.life = 0).contains("life must be at least 1"));
        assert!(bad(&|f| f.banned.clear()).contains("non-empty banned list"));
        assert!(bad(&|f| f.banned.push("balance".into())).contains("listed twice"));
        assert!(bad(&|f| f.banned.push("Leech Collector // Bloodletting".into())).contains("front-face"));
        assert!(bad(&|f| f.bans = Bans::Scryfall).contains("needs bans"));
        // [commander] cannot move
        let c = Format::commander();
        for f in [
            Format { life: 30, ..c.clone() },
            Format { seats: 2, ..c.clone() },
            Format { first_player_draws: false, ..c.clone() },
        ] {
            assert!(f.validate().unwrap_err().contains("must stay 4 seats, 40 life"), "{f:?}");
        }
        // an unknown format on a deck
        let d = deck("x", Some("calli"), &[]);
        let e = formats_table(BTreeMap::new(), [&d]).unwrap_err();
        assert!(e.contains("deck x: unknown format \"calli\""), "{e}");
        let mut raw = BTreeMap::new();
        raw.insert("odd".to_string(), Format { seats: 3, ..ok });
        assert!(formats_table(raw, []).unwrap_err().contains("seats must be 2 or 4"));
    }

    #[test]
    fn a_table_admits_its_own_decks_and_commander_decks_without_a_banned_card() {
        let (cali, mut cmd) = (cali(), Format::commander());
        cmd.key = "commander".into();
        let cali_deck = deck("k", Some("cali"), &["Sol Ring"]);
        // its own format
        assert!(cali.admits(&cali_deck, &cali).is_ok());
        assert!(cmd.admits(&deck("c", None, &["Sol Ring"]), &cmd).is_ok());
        // a Cali deck at a Commander table: refused, naming the format
        let e = cmd.admits(&cali_deck, &cali).unwrap_err();
        assert_eq!(e.len(), 1);
        assert!(e[0].contains("Cali Commander") && e[0].contains("k plays"), "{e:?}");
        // a Commander deck at a Cali table: fine unless it holds a banned card (front face, lower case)
        assert!(cali.admits(&deck("c", None, &["Sol Ring", "Mana Crypt"]), &cmd).is_ok());
        let e = cali
            .admits(&deck("c", None, &["Sol Ring", "the tabernacle at pendrell vale", "Karakas", "Karakas"]), &cmd)
            .unwrap_err();
        assert_eq!(e.len(), 1);
        assert!(e[0].contains("banned in Cali Commander: the tabernacle at pendrell vale, Karakas"), "{e:?}");
    }
}
