//! Real Oracle text frozen in a tracked fixture; no network and no generated catalog required.
use cardgen::data::Project;
use cardgen::knowledge::{self, FindError, Index, Query};
use cardgen::roles::{self, Assignment, Classifier, IDS, RULES, Taxonomy};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

const TAXONOMY: &str = include_str!("../../../../data/knowledge/roles.toml");
fn index() -> Index {
    Index::load(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/roles_oracle.json")).unwrap()
}
fn card<'a>(i: &'a Index, name: &str) -> &'a cardgen::knowledge::Card {
    let hits = i.lookup(name);
    assert_eq!(hits.len(), 1, "fixture lookup {name}");
    &i.cards[hits[0] as usize]
}
fn classify(i: &Index, name: &str) -> Assignment {
    Classifier::new().unwrap().classify(card(i, name)).unwrap()
}

#[test]
fn every_rule_has_real_positive_and_decoy_oracle_text() {
    // One independently chosen positive and negative per rule id, including reminder and incidental-effect bait.
    let cases = [
        ("alternate-win", "Revel in Riches", "Demonic Tutor"),
        ("mana", "Ashnod's Altar", "The Pride of Hull Clade"),
        ("land-ramp", "Natural Connection", "Evolving Wilds"),
        ("fixing", "Windswept Heath", "Loyal Warhound"),
        ("library-search", "Demonic Tutor", "Furnace Host Charger"),
        ("counter-spell", "Counterspell", "Tomakul Honor Guard"),
        ("mass-removal", "Toxic Deluge", "Thundermaw Hellkite"),
        ("single-removal", "Skywhaler's Shot", "Metallic Mastery"),
        ("exile-return", "Ephemerate", "Reanimate"),
        ("protect", "Heroic Intervention", "Invisible Stalker"),
        ("card-advantage", "Consecrated Sphinx", "Smash"),
        ("filter", "Ponder", "Skywhaler's Shot"),
        ("opponent-discard", "Mind Twist", "Deepcavern Imp"),
        ("graveyard-hate", "Relic of Progenitus", "Eternal Witness"),
        ("graveyard-reuse", "Eternal Witness", "Reanimate"),
        ("graveyard-battlefield", "Agadeem's Awakening // Agadeem, the Undercrypt", "Ephemerate"),
        ("fill-graveyard", "Deadbridge Chant", "Deadly Dispute"),
        ("copy", "Twinflame", "Emeritus of Conflict // Lightning Bolt"),
        ("create-token", "Undercity Dire Rat", "Ugin's Mastery"),
        ("team-pump", "Obelisk of Urd", "Heroic Intervention"),
        ("sacrifice-cost", "Carrion Feeder", "Mogg Fanatic"),
        ("life-loss", "Exsanguinate", "Lightning Bolt"),
        ("gain-life", "Polluted Bonds", "Sephiroth's Intervention"),
        ("counters", "Arbor Armament", "Spark Double"),
        ("extra-phase", "Time Warp", "Ogre Berserker"),
        ("grant-evasion", "Silverblade Paladin", "Invisible Stalker"),
        ("restriction", "Propaganda", "Mask of Intolerance"),
    ];
    let i = index();
    let classifier = Classifier::new().unwrap();
    assert_eq!(cases.len(), RULES.len());
    for rule in RULES {
        let (_, positive, negative) = cases.iter().find(|(id, _, _)| *id == rule.id).unwrap();
        let yes = classifier.classify(card(&i, positive)).unwrap();
        let no = classifier.classify(card(&i, negative)).unwrap();
        assert_eq!(
            yes.evidence.get(rule.role).map(|e| e.rule.as_str()),
            Some(rule.id),
            "{} {positive}: {yes:?}",
            rule.id
        );
        assert!(!no.roles.iter().any(|r| r == rule.role), "{} bait {negative}: {no:?}", rule.id);
    }
}

#[test]
fn face_indices_keyword_and_cost_conventions() {
    let i = index();
    for (name, expected) in [
        ("Deadly Dispute", vec!["draw", "tokens"]),
        ("Lion's Eye Diamond", vec!["ramp", "fixing"]),
        ("Maze Skullbomb", vec![]),
        ("K'rrik, Son of Yawgmoth", vec!["ramp", "counters", "lifegain"]),
        ("Evolving Wilds", vec!["fixing"]),
        ("The Pride of Hull Clade", vec!["draw"]),
        ("Spark Double", vec!["copy"]),
        ("Arbor Armament", vec!["counters"]),
    ] {
        assert_eq!(classify(&i, name).roles, expected, "{name}");
    }
    let mdfc = classify(&i, "Agadeem's Awakening // Agadeem, the Undercrypt");
    assert_eq!(mdfc.roles, ["reanimate"]);
    assert_eq!(mdfc.evidence["reanimate"].face, Some(0));
    let prepare = classify(&i, "Emeritus of Conflict // Lightning Bolt");
    assert_eq!(prepare.roles, ["removal"]);
    assert_eq!(prepare.evidence["removal"].face, Some(1));
    let pw = classify(&i, "Elspeth, Sun's Champion");
    assert!(!pw.roles.contains(&"anthem".into()), "ultimate cannot supply a role");
}

#[test]
fn reminder_and_self_name_substitution_is_case_sensitive_and_whole_word() {
    let i = index();
    let c = card(&i, "Scion of the Ur-Dragon");
    assert_eq!(
        roles::normalize(c, "Scion of the Ur-Dragon (counter it unless (pay)) becomes a copy."),
        "~  becomes a copy."
    );
    let c = card(&i, "K'rrik, Son of Yawgmoth");
    assert_eq!(
        roles::normalize(c, "K'rrik, Son of Yawgmoth K'rrik K'rrik's K'rrikish k'rrik"),
        "~ ~ ~'s K'rrikish k'rrik"
    );
}

#[test]
fn taxonomy_guards_cover_every_rule_evidence_and_role_ids_are_exact() {
    let t = Taxonomy::parse(TAXONOMY).unwrap();
    assert_eq!(t.role.len(), IDS.len());
    let i = index();
    let classifier = Classifier::new().unwrap();
    for c in &i.cards {
        let a = classifier.classify(c).unwrap();
        assert!(a.roles.len() <= 4);
        assert_eq!(a.primary, a.roles.first().cloned().unwrap_or_default());
        for r in &a.roles {
            let e = &a.evidence[r];
            let lines = c.lines();
            let (face, line) = lines[e.line.unwrap()];
            assert_eq!(face, e.face);
            let guard = &t.role.iter().find(|d| &d.id == r).unwrap().guard;
            assert!(fancy_regex::Regex::new(guard).unwrap().is_match(line).unwrap(), "{} {r} {line}", c.raw.name);
        }
    }
    assert!(Taxonomy::parse(&TAXONOMY.replace("version = 1", "version = 2")).is_err());
}

#[test]
fn routing_is_explicit_and_does_not_hide_unknown_lines() {
    let i = index();
    assert!(!classify(&i, "Counterspell").needs_llm);
    assert!(!classify(&i, "Ogre Berserker").needs_llm);
    assert!(classify(&i, "Skywhaler's Shot").needs_llm, "only R+L role matched");
    assert!(classify(&i, "Ugin's Mastery").needs_llm, "unparsed manifest effect");
}

#[test]
fn override_wins_even_over_certain_and_invalid_overrides_are_errors() {
    let i = index();
    let map = roles::generate(&i, "[[card]]\nname='Counterspell'\nroles=['draw']\nnote='test override'\n").unwrap();
    let a = &map[&card(&i, "Counterspell").raw.oracle_id];
    assert_eq!(a.roles, ["draw"]);
    assert_eq!(a.evidence["draw"].source, "override");
    assert!(!a.needs_llm);
    for text in [
        "[[card]]\nname='Counterspell'\nroles=['bad']\nnote='test'",
        "[[card]]\nname='Counterspell'\nroles=['draw','draw']\nnote='test'",
        "[[card]]\nname='Counterspell'\nroles=[]\nnote=''",
        "[[card]]\nname='missing'\nroles=[]\nnote='test'",
    ] {
        assert!(roles::generate(&i, text).is_err(), "{text}");
    }
    let cleared = roles::generate(&i, "[[card]]\nname='Counterspell'\nroles=[]\nnote='test'\n").unwrap();
    assert!(cleared[&card(&i, "Counterspell").raw.oracle_id].roles.is_empty());
}

struct Env(PathBuf);
impl Env {
    fn new() -> Self {
        static N: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "roles-test-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::SeqCst)
        ));
        let dir = root.join("data/knowledge");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/roles_oracle.json"),
            dir.join("oracle.json"),
        )
        .unwrap();
        std::fs::write(dir.join("roles.toml"), TAXONOMY).unwrap();
        std::fs::write(dir.join("roles_overrides.toml"), "").unwrap();
        std::fs::write(
            root.join("data/decks.json"),
            r#"{"mine":[{"key":"test","commander":"Counterspell","cards":["Counterspell"]}]}"#,
        )
        .unwrap();
        std::fs::write(
            root.join("data/cards.json"),
            r#"{"cards":[{"name":"Counterspell","source":"manual","tags":{"ctr":"any"}}]}"#,
        )
        .unwrap();
        Self(root)
    }
    fn project(&self) -> Project {
        Project::new(&self.0)
    }
    fn args(&self, args: &[&str]) -> Vec<String> {
        args.iter()
            .map(|s| s.to_string())
            .chain(["--data-dir".into(), self.0.join("data/knowledge").display().to_string()])
            .collect()
    }
    fn command(&self, args: &[&str]) -> Result<(String, bool), String> {
        roles::command(&self.project(), &self.args(args))
    }
    fn find(&self, args: &[&str]) -> Result<String, FindError> {
        knowledge::find(&self.project(), &self.args(args))
    }
}
impl Drop for Env {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn deterministic_generation_find_side_table_and_additive_json() {
    let e = Env::new();
    let before: Value = serde_json::from_str(&e.find(&["--name", "Counterspell", "--json"]).unwrap()).unwrap();
    assert!(before[0].get("roles").is_none());
    assert!(e.find(&["--role", "draw"]).unwrap_err().to_string().contains("cardgen roles --rules"));
    e.command(&["--rules"]).unwrap();
    let path = e.0.join("data/knowledge/roles.json");
    let first = std::fs::read(&path).unwrap();
    e.command(&["--rules"]).unwrap();
    assert_eq!(first, std::fs::read(&path).unwrap());
    let after: Value = serde_json::from_str(&e.find(&["--name", "Counterspell", "--json"]).unwrap()).unwrap();
    assert_eq!(after[0]["roles"], json!(["counter"]));
    assert_eq!(after[0]["primary"], "counter");
    for (key, value) in before[0].as_object().unwrap() {
        assert_eq!(after[0].get(key), Some(value));
    }
    let both: Value =
        serde_json::from_str(&e.find(&["--role", "ramp", "--role", "sac-outlet", "--json"]).unwrap()).unwrap();
    assert!(both.as_array().unwrap().iter().any(|c| c["name"] == "Ashnod's Altar"));
    let mut i = index();
    let map = roles::load(&path).unwrap();
    i.attach_roles(&map).unwrap();
    let q = Query { roles: vec!["ramp".into(), "sac-outlet".into()], ..Query::default() };
    assert!(
        i.search(&q, None).unwrap().0.iter().all(|n| q.roles.iter().all(|r| i.roles(*n).unwrap().roles.contains(r)))
    );
    let mut stale = map;
    stale.pop_first();
    assert!(i.attach_roles(&stale).is_err());
    assert!(i.roles(i.lookup("Counterspell")[0]).unwrap().roles.contains(&"counter".into()));
}

#[test]
fn usage_errors_and_validation_reject_missed_thresholds() {
    let e = Env::new();
    let err = e.find(&["--role", "not-a-role"]).unwrap_err();
    assert!(matches!(err, FindError::Usage(_)));
    for role in IDS {
        assert!(err.to_string().contains(role));
    }
    for args in
        [vec![], vec!["--rules", "--stats"], vec!["--show"], vec!["--gold", "x", "--rules"], vec!["--rules", "--json"]]
    {
        assert!(e.command(&args).is_err(), "{args:?}");
    }
    e.command(&["--rules"]).unwrap();
    let i = index();
    let id = &card(&i, "Counterspell").raw.oracle_id;
    let gold = e.0.join("gold.toml");
    std::fs::write(
        &gold,
        format!("[[card]]\nname='Counterspell'\noracle_id='{id}'\nroles=['counter']\nprimary='counter'\n"),
    )
    .unwrap();
    let args = ["--validate", "--gold", gold.to_str().unwrap(), "--json"];
    let (report, passed) = e.command(&args).unwrap();
    assert!(passed);
    let v: Value = serde_json::from_str(&report).unwrap();
    assert_eq!(v["hand_micro"]["tp"], 1);
    assert_eq!(v["certain_micro"]["tp"], 1);
    std::fs::write(
        &gold,
        format!("[[card]]\nname='Counterspell'\noracle_id='{id}'\nroles=[]\nprimary=''\nnot_roles=['counter']\n"),
    )
    .unwrap();
    let (report, passed) = e.command(&args).unwrap();
    assert!(!passed);
    let v: Value = serde_json::from_str(&report).unwrap();
    assert_eq!(v["certain_micro"]["fp"], 1);
    assert_eq!(v["certain_not_roles"].as_array().unwrap().len(), 1);
}

#[test]
fn functional_context_does_not_confuse_untap_landfall_or_incidental_costs() {
    let i = index();
    for (name, role) in [
        ("Acrobatic Leap", "removal"),
        ("Arachnoid Adaptation", "removal"),
        ("Caustic Crawler", "wipe"),
        ("Silverblade Paladin", "anthem"),
        ("Court of Ambition", "stax"),
        ("Decoy Gambit", "stax"),
        ("Ob Nixilis, the Adversary", "stax"),
        ("Beseech the Mirror", "ramp"),
        ("Beseech the Mirror", "draw"),
        ("Fblthp, Lost on the Range", "selection"),
    ] {
        assert!(!classify(&i, name).roles.iter().any(|r| r == role), "{name} -> {role}");
    }
    for (name, role) in [
        ("Abduction", "removal"),
        ("Narcolepsy", "removal"),
        ("Loyal Warhound", "ramp"),
        ("Chaos Warp", "removal"),
        ("Overkill", "removal"),
        ("Nibelheim Aflame", "wipe"),
        ("Mystic Confluence", "draw"),
        ("Ad Nauseam", "draw"),
        ("Underworld Breach", "recursion"),
        ("Otawara, Soaring City", "removal"),
        ("Rooftop Assassin", "lifegain"),
        ("Thanos, the Mad Titan", "lifegain"),
        ("Deadbridge Chant", "recursion"),
        ("Deadbridge Chant", "reanimate"),
    ] {
        assert!(classify(&i, name).roles.iter().any(|r| r == role), "{name} -> {role}");
    }
}

#[test]
fn prominence_puts_main_effects_before_their_riders() {
    let i = index();
    for (name, expected) in [
        ("Ashnod's Altar", vec!["ramp", "sac-outlet"]),
        ("Mana Drain", vec!["counter", "ramp"]),
        ("Twinflame", vec!["copy", "tokens"]),
        ("Sinister Monolith", vec!["drain", "draw", "lifegain"]),
        ("Akroma's Will", vec!["protection", "anthem", "evasion"]),
        ("Whispersilk Cloak", vec!["evasion", "protection"]),
    ] {
        let got = classify(&i, name);
        assert_eq!(got.roles, expected, "{name}");
        assert_eq!(got.primary, expected[0], "{name}");
    }
}

#[test]
fn recall_translation_excludes_self_punishment_and_preserves_effect_tags() {
    let (roles, reason) = roles::tag_roles("vaultping", &Value::Bool(true));
    assert!(roles.is_empty());
    assert!(reason.contains("own controller"));
    assert_eq!(roles::tag_roles("bartist", &Value::Bool(true)).0, ["drain"]);
    assert_eq!(roles::tag_roles("fill", &Value::String("dispute".into())).0, ["self-mill"]);
    assert_eq!(roles::tag_roles("xtutor", &Value::Bool(true)).0, ["tutor"]);
    assert_eq!(roles::tag_roles("prot", &Value::String("blink".into())).0, ["blink"]);
}

#[test]
fn audited_catalog_patterns_and_real_decoys() {
    let i = index();
    let classifier = Classifier::new().unwrap();
    let cases: &[(&str, &[&str])] = &[
        ("Springjack Pasture", &["fixing", "ramp", "sac-outlet", "tokens"]),
        ("Zurgo Stormrender", &["drain", "draw", "tokens"]),
        ("Ghirapur Orrery", &["draw", "ramp"]),
        ("Alabaster Wall", &["protection"]),
        ("Pyretic Charge", &["anthem", "draw"]),
        ("Staff of Titania", &["ramp", "tokens"]),
        ("Curse of Vengeance", &["draw", "lifegain"]),
        ("Marchesa's Emissary", &["counters"]),
        ("Heralds of Tzeentch", &["draw"]),
        ("Scavenger Regent // Exude Toxin", &["wipe"]),
        ("Windfall", &["draw"]),
        ("Baleful Beholder", &["anthem", "evasion", "removal"]),
        ("Isolation at Orthanc", &["removal"]),
        ("He Who Hungers", &["discard", "recursion", "sac-outlet"]),
        ("Riding the Dilu Horse", &["evasion"]),
        ("Baylen, the Haymaker", &["counters", "draw", "fixing", "ramp"]),
        ("Gigapede", &["recursion"]),
        ("Fatal Mutation", &["removal"]),
        ("Lord Windgrace", &["draw", "ramp"]),
        ("Serra Aviary", &["anthem"]),
        ("Gandalf, Spark Starter", &["removal"]),
        ("Saheeli, the Gifted", &["ramp", "tokens"]),
        ("Struggle for Sanity", &[]),
        ("Mirror of Life Trapping", &["blink"]),
        ("Decree of Pain", &["draw", "wipe"]),
        ("Orzhov Racketeers", &["discard", "tokens"]),
        ("Fearless Swashbuckler", &["draw"]),
        ("Aether Barrier", &["stax"]),
        ("Land Equilibrium", &["stax"]),
        ("Insatiable Frugivore", &["anthem", "evasion"]),
        ("Double Negative", &["counter"]),
        ("Marrow-Gnawer", &["anthem", "evasion", "sac-outlet", "tokens"]),
        ("Hunting Drake", &["removal"]),
        ("Argothian Elder", &["ramp"]),
        ("Collective Inferno", &["finisher"]),
        ("Naga Vitalist", &["fixing", "ramp"]),
        ("Vedalken Infuser", &["counters"]),
        ("Deathgazer", &["removal"]),
        ("Summoned Dromedary", &["recursion"]),
        ("Become the Pilot", &["evasion", "removal"]),
        ("Tributary Instructor", &["counters", "draw"]),
        ("Flux", &["selection"]),
        ("Magus of the Scroll", &["removal"]),
        ("Debt to the Kami", &["removal"]),
        ("Sword of Hearth and Home", &["blink", "fixing", "protection", "ramp"]),
        ("Void Beckoner", &["counters"]),
        ("Hugs, Grisly Guardian", &["draw", "ramp"]),
        ("Unassuming Sage", &[]),
        ("Ruin Processor", &["lifegain"]),
        ("Trolls of Tel-Jilad", &["protection"]),
        ("Óin the Brave", &["selection"]),
        ("Kinjalli's Sunwing", &["stax"]),
        ("Blot Out", &["removal"]),
        ("Niv-Mizzet, Guildpact", &["draw", "lifegain", "removal"]),
        ("Flame Wave", &["wipe"]),
        ("Peace and Quiet", &["removal"]),
        ("Venerated Rotpriest", &["counters"]),
        ("Dread Defiler", &["drain"]),
        ("Skeletal Scrying", &["draw"]),
        ("Astarion, the Decadent", &["drain", "lifegain"]),
        ("Overgrown Arch", &["lifegain", "selection"]),
        ("Shellshock", &["wipe"]),
        ("Missy", &["drain", "draw", "reanimate"]),
        ("Sealed Fate", &[]),
        ("Vow of Lightning", &["stax"]),
        ("Minds Aglow", &["draw"]),
        ("Depth Defiler", &["draw", "removal"]),
    ];
    for (name, expected) in cases {
        let a = classifier.classify(card(&i, name)).unwrap();
        let mut got: Vec<_> = a.roles.iter().map(String::as_str).collect();
        got.sort();
        assert_eq!(&got, expected, "{name}: {a:?}");
    }
}

#[test]
fn exact_certainty_templates_and_incidental_near_matches() {
    let i = index();
    for (name, role, certain) in [
        ("Fyndhorn Elves", "ramp", true),
        ("Delighted Halfling", "ramp", false),
        ("Counterspell", "counter", true),
        ("Demonic Tutor", "tutor", true),
        ("Diabolic Intent", "tutor", true),
        ("Vampiric Tutor", "tutor", false),
        ("Tidings", "draw", true),
        ("Precognitive Perception", "draw", true),
        ("Cosmos Elixir", "draw", false),
        ("Rhox War Monk", "lifegain", true),
        ("Bre of Clan Stoutarm", "lifegain", false),
    ] {
        assert_eq!(classify(&i, name).evidence[role].certain, certain, "{name} {role}");
    }
    for name in [
        "Unfulfilled Desires",
        "Immersturm Predator",
        "My Precious // Allure of Power",
        "Springjack Pasture",
        "Rashida Scalebane",
    ] {
        assert!(classify(&i, name).needs_llm, "ambiguous/risky roles must be routed: {name}");
    }
}

#[test]
fn certainty_audit_secondary_effects_have_real_positive_and_negative_text() {
    let i = index();
    for (name, role, present) in [
        ("Tablet of Discovery", "draw", true),
        ("Harsh Deceiver", "draw", false),
        ("Extraction Integrant", "wipe", false),
        ("Teferi's Response", "removal", true),
        ("Beseech the Mirror", "removal", false),
        ("Coastal Discovery", "counters", true),
        ("Bloodcrazed Paladin", "counters", false),
        ("Pearl-Ear, Imperial Advisor", "ramp", true),
        ("Moltensteel Dragon", "ramp", false),
        ("Minamo's Meddling", "discard", true),
        ("Wild Guess", "discard", false),
        ("Render Silent", "stax", true),
        ("Decree of Pain", "stax", false),
        ("Muddle the Mixture", "tutor", true),
        ("Mental Journey", "tutor", false),
        ("Illicit Shipment", "copy", true),
        ("Emeritus of Conflict // Lightning Bolt", "copy", false),
        ("Night Nurse, Healer of Heroes", "recursion", true),
        ("Fact or Fiction", "recursion", false),
        ("Statute of Denial", "selection", false),
        ("Ponder", "selection", true),
    ] {
        assert_eq!(classify(&i, name).roles.iter().any(|r| r == role), present, "{name} {role}");
    }
}

#[test]
fn audit_is_seeded_gold_excluding_legal_without_replacement_and_independent_of_side_tables() {
    let e = Env::new();
    let i = index();
    let excluded = card(&i, "Counterspell");
    let gold = format!(
        "[[card]]\nname='Counterspell'\noracle_id='{}'\nroles=['counter']\nprimary='counter'\n",
        excluded.raw.oracle_id
    );
    std::fs::write(e.0.join("data/knowledge/roles_gold.toml"), &gold).unwrap();
    // No roles.json exists: audit must classify fresh and avoid writing any artifacts.
    let (a, ok) = e.command(&["--audit", "--seed", "0", "--n", "80"]).unwrap();
    assert!(ok);
    assert_eq!(a, e.command(&["--audit", "--seed", "0", "--n", "80"]).unwrap().0);
    assert_ne!(a, e.command(&["--audit", "--seed", "1", "--n", "80"]).unwrap().0);
    let sampled: Vec<_> = a.lines().filter(|l| l.starts_with('[')).collect();
    assert_eq!(sampled.len(), 80);
    let mut ids = std::collections::BTreeSet::new();
    for row in sampled {
        let id = row.split(" | ").nth(1).unwrap();
        assert!(ids.insert(id));
        assert_ne!(id, excluded.raw.oracle_id);
        assert_eq!(i.cards[i.by_oracle_id(id).unwrap() as usize].legal, knowledge::Legal::Legal);
    }
    let certain = e.command(&["--audit", "--rule", "mana", "--certain", "--n", "999"]).unwrap().0;
    assert!(certain.contains("ramp: mana certain=true"));
    assert!(certain.contains("Oracle face="));
    assert!(!e.0.join("data/knowledge/roles.json").exists());
    for args in [
        vec!["--audit", "--n", "0"],
        vec!["--audit", "--n", "-1"],
        vec!["--rules", "--seed", "0"],
        vec!["--audit", "--seed", "0", "--seed", "1"],
        vec!["--audit", "--rule", "missing"],
        vec!["--audit", "--json"],
        vec!["--stats", "--certain"],
    ] {
        assert!(e.command(&args).is_err(), "{args:?}");
    }
}

#[test]
fn rules_final_gate_includes_llm_domain_roles_and_cannot_be_improved_by_overrides() {
    let e = Env::new();
    e.command(&["--rules"]).unwrap();
    let i = index();
    let gold = format!(
        "[[card]]\nname='Counterspell'\noracle_id='{}'\nroles=['counter','finisher']\nprimary='counter'\n",
        card(&i, "Counterspell").raw.oracle_id
    );
    std::fs::write(e.0.join("data/knowledge/roles_gold.toml"), gold).unwrap();
    // Other old gates all pass: the missing LLM-domain role must now fail rules-final recall.
    let (out, passed) = e.command(&["--validate", "--json"]).unwrap();
    let v: Value = serde_json::from_str(&out).unwrap();
    assert!(!passed);
    assert_eq!(v["rules_final_cards"], 1);
    assert_eq!(v["rules_final_micro"], json!({"tp":1,"fp":0,"missed":1}));
    assert_eq!(v["failures"], json!(["rules-final recall < 0.85"]));
    std::fs::write(
        e.0.join("data/knowledge/roles_overrides.toml"),
        "[[card]]\nname='Counterspell'\nroles=['counter','finisher']\nnote='cannot fix rule score'\n",
    )
    .unwrap();
    e.command(&["--rules"]).unwrap();
    let (out, passed) = e.command(&["--validate", "--json"]).unwrap();
    assert!(!passed);
    assert_eq!(serde_json::from_str::<Value>(&out).unwrap()["rules_final_micro"], v["rules_final_micro"]);
}
