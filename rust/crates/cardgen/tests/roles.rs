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
        ("mass-removal", "Toxic Deluge", "Lightning Bolt"),
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
    assert!(roles::tag_roles("fill", &Value::String("dispute".into())).0.is_empty());
    assert!(roles::tag_roles("xtutor", &Value::Bool(true)).0.is_empty());
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
        ("Lord Windgrace", &["draw", "ramp", "recursion"]),
        ("Serra Aviary", &["anthem"]),
        ("Gandalf, Spark Starter", &["removal"]),
        ("Saheeli, the Gifted", &["ramp", "tokens"]),
        ("Struggle for Sanity", &[]),
        ("Mirror of Life Trapping", &["blink"]),
        ("Decree of Pain", &["draw", "wipe"]),
        ("Orzhov Racketeers", &["discard", "tokens"]),
        ("Fearless Swashbuckler", &["draw"]), // C8: haste alone is not anthem.
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
        ("Venerated Rotpriest", &[]),
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

#[test]
fn convention_clauses_c1_through_c4_use_real_oracle_text() {
    let i = index();
    for (name, role, present) in [
        ("Gold-Forged Thopteryx", "protection", true),
        ("Coppercoat Vanguard", "protection", true),
        ("Coppercoat Vanguard", "anthem", true),
        ("Combat Research", "protection", true),
        ("Tomakul Honor Guard", "protection", false),
        ("Whip of Erebos", "lifegain", true),
        ("Whip of Erebos", "anthem", false),
        ("Tireless Tracker", "draw", true),
        ("Tireless Tracker", "tokens", true),
        ("Trail of Evidence", "draw", true),
        ("Thraben Inspector", "tokens", true),
        ("Thraben Inspector", "draw", false),
        ("Hard Evidence", "draw", false),
        ("Hard Evidence", "tokens", true),
        ("Gilded Goose", "tokens", false),
        ("Gilded Goose", "lifegain", true),
        ("Voldaren Epicure", "tokens", false),
        ("Restless Anchorage", "tokens", false),
        ("Restless Anchorage", "counters", true),
        ("Veronica, Dissident Scribe", "tokens", false),
        ("Veronica, Dissident Scribe", "draw", true),
        ("Smothering Tithe", "tokens", true),
        ("Smothering Tithe", "ramp", true),
        ("Inspiring Roar", "counters", true),
        ("Inspiring Roar", "anthem", true),
        ("Shalai, Voice of Plenty", "anthem", true),
        ("Travel Preparations", "anthem", false),
        ("Venerated Rotpriest", "counters", false),
        ("Aether Hub", "counters", false),
        ("Meren of Clan Nel Toth", "counters", false),
        ("Radstorm", "counters", true),
        ("Nuclear Fallout", "counters", false),
        ("Mikaeus, the Unhallowed", "anthem", true),
        ("Pollenbright Druid", "counters", true),
        ("Dreadhorde Invasion", "tokens", true),
        ("Dreadhorde Invasion", "counters", true),
        ("Lazotep Reaver", "tokens", true),
        ("Lazotep Reaver", "counters", false),
        ("Refurbish", "recursion", true),
        ("Refurbish", "reanimate", false),
        ("Resurgent Belief", "recursion", true),
        ("Resurgent Belief", "reanimate", false),
        ("Dance of the Manse", "recursion", true),
        ("Dance of the Manse", "reanimate", false),
        ("Repair and Recharge", "recursion", true),
        ("Repair and Recharge", "reanimate", false),
        ("Repair and Recharge", "tokens", false),
        ("Moment of Reckoning", "recursion", true),
        ("Moment of Reckoning", "reanimate", true),
        ("Zombify", "recursion", false),
        ("Zombify", "reanimate", true),
        ("Turntimber Symbiosis // Turntimber, Serpentine Wood", "reanimate", false),
    ] {
        assert_eq!(classify(&i, name).roles.iter().any(|r| r == role), present, "{name} {role}");
    }
}

#[test]
fn round_three_precision_audit_regressions() {
    let i = index();
    let cases: &[(&str, &[&str])] = &[
        ("Moldgraf Millipede", &["self-mill"]),
        ("Armed Response", &["removal"]),
        ("Mount Doom", &["drain", "fixing", "wipe"]),
        ("Guan Yu, Sainted Warrior", &["recursion"]),
        ("Summon: Knights of Round", &["anthem", "counters", "protection", "tokens"]),
        ("Aeronaut Admiral", &["anthem", "evasion"]),
        ("It That Betrays", &["reanimate", "removal"]),
        ("Frost Titan", &["removal"]),
        ("Kyren Negotiations", &["drain"]),
        ("Clamavus", &["anthem"]),
        ("Vraska, the Cutting Glare", &["removal"]),
        ("Enigma Sphinx", &["draw", "recursion"]),
        ("Pull Through the Weft", &["ramp", "recursion"]),
        ("Confessor", &["lifegain"]),
        ("Polymorphist's Jest", &["wipe"]),
        ("Lady Sun", &["removal"]),
        ("Darksteel Mutation", &["removal"]),
        ("Instill Furor", &["removal"]),
        ("Seismic Shudder", &["wipe"]),
        ("Lush Growth", &["fixing"]),
        ("Dack Fayden, Helping Hand", &[]),
        ("Turntimber Symbiosis // Turntimber, Serpentine Wood", &["selection"]),
        ("Twisted Fates", &["anthem", "counters", "removal"]),
        ("Titania's Song", &["stax"]),
        ("Sigurd, Jarl of Ravensthorpe", &["counters", "lifegain"]),
        ("Moment of Reckoning", &["reanimate", "recursion", "removal"]),
        ("Soulquake", &["recursion", "wipe"]),
        ("Willow Satyr", &["removal"]),
        ("Syr Konrad, the Grim", &["drain", "self-mill"]),
        ("Tenured Oilcaster", &["self-mill"]),
        ("Tawnos, Solemn Survivor", &["copy", "sac-outlet", "self-mill", "tokens"]),
        ("Highland Berserker", &["anthem"]),
        ("Broodlord", &["counters"]),
        ("Blossom Dryad", &["ramp"]),
        ("Icefeather Aven", &["removal"]),
        ("Duel for Dominance", &["counters", "removal"]),
        ("Loki, the Deceiver", &["copy", "draw", "tokens"]),
        ("Legerdemain", &["removal"]),
        ("Shadow of Doubt", &["stax"]),
        ("Widespread Panic", &["stax"]),
        ("Midsummer Revel", &["tokens"]),
        ("Withering Wisps", &["wipe"]),
        ("Nashi, Moon Sage's Scion", &["draw"]),
        ("Lifegift", &["lifegain"]),
        ("The Lord of Pain", &["drain", "stax"]),
        ("Grisly Transformation", &["evasion"]), // C15: an aura cantrip is not draw.
        ("Golgari Charm", &["protection", "removal", "wipe"]),
        ("Bitter Reunion", &["draw"]), // C8: haste alone is not anthem.
        ("Dog Umbra", &["protection", "removal"]),
        ("Meletis Charlatan", &["copy"]),
        ("Chrome Steed", &[]),
        ("Gods' Eye, Gate to the Reikai", &["tokens"]),
        ("Abyssal Nightstalker", &["discard"]),
        ("Acclaimed Contender", &["selection"]),
        ("Keldon Strike Team", &["tokens"]), // C8: haste alone is not anthem.
        ("Grismold, the Dreadsower", &["counters", "tokens"]),
        ("Cybernetica Datasmith", &["draw", "tokens"]),
        ("Adéwalé, Breaker of Chains", &["recursion", "selection"]),
        ("Inspiring Paladin", &["anthem"]),
        ("Parallax Nexus", &[]),
        ("Tymaret, the Murder King", &["drain", "recursion", "sac-outlet"]),
        ("Celestial Mantle", &["lifegain"]),
        ("Fire Shrine Keeper", &["removal"]),
        ("Destined Confrontation", &["wipe"]),
        ("Dream Leash", &["removal"]),
        ("Zhao, the Moon Slayer", &["stax"]),
        ("Rising Waters", &["stax"]),
        ("Library", &["draw", "fixing", "tokens"]),
        ("Phyrexian Boon", &["removal"]),
        ("Ragged Recluse // Odious Witch", &["drain", "lifegain"]),
        ("The Flesh Is Weak", &["anthem", "counters", "wipe"]),
    ];
    let classifier = Classifier::new().unwrap();
    for (name, expected) in cases {
        let a = classifier.classify(card(&i, name)).unwrap();
        let mut got: Vec<_> = a.roles.iter().map(String::as_str).collect();
        got.sort();
        assert_eq!(&got, expected, "{name}: {a:?}");
    }
}

#[test]
fn non_deck_cards_remain_searchable_but_are_excluded_from_role_catalog_and_audits() {
    let e = Env::new();
    let mut i = index();
    let map = roles::generate(&i, "").unwrap();
    for name in ["Spinny Ride", "Unhinged Beast Hunt"] {
        let c = card(&i, name);
        assert!(!roles::is_deck_card(c));
        assert!(!map.contains_key(&c.raw.oracle_id));
    }
    let stats = roles::stats(&i, &map);
    assert_eq!(stats.skipped, 2);
    assert_eq!(stats.total, i.cards.len() - 2);
    i.attach_roles(&map).unwrap();
    for name in ["Spinny Ride", "Unhinged Beast Hunt"] {
        assert!(i.roles(i.lookup(name)[0]).unwrap().roles.is_empty());
    }
    e.command(&["--rules"]).unwrap();
    assert!(e.command(&["--stats"]).unwrap().0.contains("SKIPPED: 2"));
    assert!(e.command(&["--show", "Spinny Ride"]).unwrap().0.contains("excluded from roles"));
    assert!(e.find(&["--name", "Spinny Ride"]).is_ok());
    let audit = roles::audit(&i, "card=[]", 611, 9999, None, false).unwrap();
    assert!(!audit.contains("Spinny Ride"));
    assert!(!audit.contains("Unhinged Beast Hunt"));
    assert!(roles::generate(&i, "[[card]]\nname='Spinny Ride'\nroles=[]\nnote='invalid'\n").is_err());
    let mut stale = map.clone();
    let removed = card(&i, "Counterspell").raw.oracle_id.clone();
    stale.remove(&removed);
    stale.insert(card(&i, "Spinny Ride").raw.oracle_id.clone(), Assignment::default());
    assert!(i.attach_roles(&stale).is_err(), "equal count must not mask a missing deck card");
    assert_eq!(i.roles(i.lookup("Counterspell")[0]).unwrap().roles, ["counter"]);
}

fn rulings_index() -> Index {
    let e = Env::new();
    let path = e.0.join("data/knowledge/oracle.json");
    let mut doc: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    let extra: Value = serde_json::from_str(include_str!("fixtures/roles_rulings.json")).unwrap();
    doc["cards"].as_array_mut().unwrap().extend(extra["cards"].as_array().unwrap().iter().cloned());
    doc["meta"]["count"] = json!(doc["cards"].as_array().unwrap().len());
    std::fs::write(&path, doc.to_string()).unwrap();
    Index::load(&path).unwrap()
}

#[test]
fn approved_rulings_and_shared_effect_vocabulary_use_real_text() {
    let i = rulings_index();
    for (name, yes, no) in [
        ("Twinflame", vec!["copy", "tokens"], vec![]),
        ("Smothering Tithe", vec!["tokens", "ramp"], vec![]),
        ("Revel in Riches", vec!["tokens", "ramp"], vec![]),
        ("Undercity Dire Rat", vec!["tokens"], vec!["ramp"]),
        ("Leech Collector // Bloodletting", vec!["lifegain", "drain"], vec!["copy"]),
        ("Syphon Mind", vec!["draw", "discard"], vec![]),
        ("Blightning", vec!["discard"], vec![]),
        ("Rise of the Eldrazi", vec!["draw"], vec![]),
        ("Elder Owyn Lyons", vec!["protection", "recursion"], vec![]),
        ("Skeletal Swarming", vec!["anthem"], vec![]),
        ("Grave Exchange", vec!["recursion", "removal"], vec![]),
        ("Aura Shards", vec!["removal"], vec![]),
        ("Hatchet Bully", vec!["removal"], vec!["counters"]),
        ("Laboratory Maniac", vec!["finisher"], vec!["draw"]),
        ("Together Forever", vec!["counters"], vec![]),
        ("Imi Statue", vec!["stax"], vec![]),
        ("Keldon Flamesage", vec!["draw"], vec!["selection"]),
        ("Noxious Revival", vec!["recursion"], vec![]),
        ("Sedgemoor Witch", vec!["tokens"], vec!["lifegain"]),
        ("Chainer, Dementia Master", vec!["reanimate"], vec!["wipe"]),
        ("Cruel Grimnarch", vec!["discard"], vec!["stax"]),
    ] {
        let a = classify(&i, name);
        for role in yes {
            assert!(a.roles.iter().any(|r| r == role), "{name}: missing {role}: {a:?}");
        }
        for role in no {
            assert!(!a.roles.iter().any(|r| r == role), "{name}: unexpected {role}: {a:?}");
        }
    }
    let copy = classify(&i, "Twinflame");
    assert_eq!(copy.primary, "copy");
    let prepare = classify(&i, "Leech Collector // Bloodletting");
    assert_eq!(prepare.evidence["lifegain"].face, Some(0));
    assert_eq!(prepare.evidence["lifegain"].line, Some(0));
    assert!(roles::tag_roles("fill", &json!("dispute")).0.is_empty());
    assert!(roles::tag_roles("fill", &json!("dispute")).1.contains("additional-cost"));
    assert!(roles::tag_roles("xtutor", &json!(true)).0.is_empty());
    assert_eq!(roles::tag_roles("fill", &json!("entomb")).0, ["self-mill"]);
}

#[test]
fn rejected_hits_and_same_line_roles_record_routing_without_changing_mana_exemptions() {
    use roles::RouteReason::*;
    let i = rulings_index();
    for name in ["Victimize", "Dawn's Reflection"] {
        let a = classify(&i, name);
        assert!(a.needs_llm && a.route.contains(&RejectedHit), "{name}: {a:?}");
    }
    let a = classify(&i, "Twinflame");
    assert!(a.route.contains(&SameLine), "{a:?}");
    let mana = classify(&i, "Phyrexian Altar");
    assert!(!mana.route.contains(&SameLine), "{mana:?}");
    let clue = classify(&i, "Tireless Tracker");
    assert!(!clue.route.contains(&SameLine), "{clue:?}");
    let s = roles::stats(&i, &roles::generate(&i, "").unwrap());
    assert!(s.legal_route_causes["RejectedHit"] > 0);
    assert!(s.legal_route_causes["SameLine"] > 0);
}

#[test]
fn metadata_rejects_stale_taxonomy_rules_oracle_count_and_optional_find_omits_roles() {
    let e = Env::new();
    e.command(&["--rules"]).unwrap();
    let path = e.0.join("data/knowledge/roles.json");
    let good = std::fs::read_to_string(&path).unwrap();
    for field in ["taxonomy", "rules_fnv", "oracle_fnv", "count"] {
        let mut doc: Value = serde_json::from_str(&good).unwrap();
        doc["meta"][field] = if field == "count" { json!(0) } else { json!("stale") };
        std::fs::write(&path, doc.to_string()).unwrap();
        let map = roles::load(&path).unwrap();
        assert!(index().attach_roles(&map).unwrap_err().contains("cardgen roles --rules"));
        assert!(e.find(&["--role", "draw"]).is_err());
        let out = knowledge::find_with_notes(&e.project(), &e.args(&["--name", "Counterspell", "--json"])).unwrap();
        let rows: Value = serde_json::from_str(&out.stdout()).unwrap();
        assert!(rows[0].get("roles").is_none() && rows[0].get("primary").is_none());
        assert_eq!(out.stderr().lines().count(), 1);
        assert!(out.stderr().contains("cardgen roles --rules"));
    }
    let (out, passed) = e.command(&["--lint"]).unwrap();
    assert!(!passed && out.contains("UNDOCUMENTED")); // a small fixture cannot satisfy catalog coverage
}

fn round5_synthetic(typ: &str, text: &str, mana: &[&str]) -> Assignment {
    let e = Env::new();
    let path = e.0.join("data/knowledge/oracle.json");
    std::fs::write(
        &path,
        json!({
            "meta": {"schema": knowledge::SCHEMA, "count": 1},
            "cards": [{"name": "Specimen", "oracle_id": "synthetic", "type_line": typ,
                "oracle_text": text, "produced_mana": mana}]
        })
        .to_string(),
    )
    .unwrap();
    let i = Index::load(&path).unwrap();
    Classifier::new().unwrap().classify(&i.cards[0]).unwrap()
}

#[test]
fn roleless_anthem_hits_cannot_excuse_an_unread_team_effect() {
    // Unnatural Growth's Oracle line: a real function absent from the accepted roles.
    let a = round5_synthetic(
        "Enchantment",
        concat!(
            "At the beginning of each combat, double the power and toughness of each creature ",
            "you control until end of turn."
        ),
        &[],
    );
    assert!(a.roles.is_empty() && a.route.contains(&roles::RouteReason::NoRole), "{a:?}");
}

#[test]
fn variable_mana_is_not_excused_as_a_one_mana_land() {
    // Gaea's Cradle.
    let a = round5_synthetic("Land", "{T}: Add {G} for each creature you control.", &["G"]);
    assert!(a.route.contains(&roles::RouteReason::NoRole), "{a:?}");
    assert!(a.route.contains(&roles::RouteReason::RejectedHit), "{a:?}");
}

#[test]
fn rejected_loot_needs_an_accepted_selection_role() {
    // Credit Voucher.
    let a = round5_synthetic(
        "Artifact",
        concat!(
            "{2}, {T}, Sacrifice this artifact: Shuffle any number of cards from your hand into your library, ",
            "then draw that many cards."
        ),
        &[],
    );
    assert!(a.route.contains(&roles::RouteReason::NoRole), "{a:?}");
    assert!(a.route.contains(&roles::RouteReason::RejectedHit), "{a:?}");
}

#[test]
fn ignore_prefixes_do_not_read_later_sentences() {
    // Cabal Therapy, then Plumb the Forbidden (only their first Oracle lines).
    let a = round5_synthetic(
        "Sorcery",
        concat!("Choose a nonland card name. Target player reveals their hand and discards all cards with that name."),
        &[],
    );
    assert!(a.route.contains(&roles::RouteReason::NoRole), "{a:?}");
    assert!(a.route.contains(&roles::RouteReason::UnknownLine), "{a:?}");
    let a = round5_synthetic(
        "Instant",
        concat!(
            "As an additional cost to cast this spell, you may sacrifice one or more creatures. ",
            "When you do, copy this spell for each creature sacrificed this way."
        ),
        &[],
    );
    assert!(a.route.contains(&roles::RouteReason::NoRole), "{a:?}");
    assert!(a.route.contains(&roles::RouteReason::UnknownLine), "{a:?}");
}

#[test]
fn negative_toughness_auras_are_not_boilerplate() {
    // Dead Weight and Feebleness shaped auras: a negative toughness is real removal, a plain pump is boilerplate.
    for text in ["Enchant creature\nEnchanted creature gets -2/-2.", "Enchant creature\nEnchanted creature gets +2/-1."]
    {
        let a = round5_synthetic("Enchantment — Aura", text, &[]);
        assert!(a.route.contains(&roles::RouteReason::NoRole) || !a.roles.is_empty(), "{text}: {a:?}");
    }
    for text in ["Enchant creature\nEnchanted creature gets +1/+1.", "Enchant creature\nEnchanted creature gets +2/-0."]
    {
        let a = round5_synthetic("Enchantment — Aura", text, &[]);
        assert!(!a.route.contains(&roles::RouteReason::NoRole), "{text}: {a:?}");
    }
}

#[test]
fn two_produced_colors_require_fixing_or_review() {
    // Two independently ordinary tap abilities must still be checked together.
    let a = round5_synthetic("Land", "{T}: Add {G}.\n{T}: Add {W}.", &["G", "W"]);
    assert!(a.route.contains(&roles::RouteReason::NoRole), "{a:?}");
    for text in ["Flying, vigilance", "Ward {2}", "Equip {3}", "{T}: Add {C}."] {
        let a = round5_synthetic("Land", text, &[]);
        assert!(!a.route.contains(&roles::RouteReason::NoRole), "{text}: {a:?}");
    }
}

#[test]
fn approved_c7_through_c13_and_general_audit_vocabulary() {
    let i = rulings_index();
    for (name, yes, no) in [
        ("Viridian Acolyte", vec!["fixing"], vec!["ramp"]),
        ("Chromatic Star", vec!["fixing"], vec!["ramp", "draw"]),
        ("Chasm Guide", vec![], vec!["anthem"]),
        ("Rally the Monastery", vec!["tokens", "removal"], vec!["anthem"]),
        ("Infernal Scarring", vec![], vec!["draw"]),
        ("Llanowar Sentinel", vec![], vec!["tutor"]),
        ("Mercy Killing", vec!["removal"], vec!["tokens"]),
        ("Court of Cunning", vec!["draw"], vec![]),
        ("Salt Vampire", vec!["lifegain"], vec!["draw"]),
        ("Animist's Might", vec!["removal"], vec!["anthem", "ramp"]),
        ("Thud", vec!["removal"], vec!["drain"]),
        ("Riptide Gearhulk", vec!["removal"], vec![]),
        ("Desynchronization", vec!["wipe"], vec![]),
        ("Quirion Elves", vec!["ramp", "fixing"], vec![]),
        ("Thriving Grove", vec!["fixing"], vec!["ramp"]),
        ("Cosmic Rebirth", vec!["recursion", "reanimate"], vec![]),
        ("Practiced Scrollsmith", vec!["recursion"], vec![]),
        ("Ugin's Binding", vec!["removal", "wipe"], vec!["recursion"]),
        ("Aerial Assault", vec!["removal"], vec!["evasion"]),
    ] {
        let a = classify(&i, name);
        for role in yes {
            assert!(a.roles.iter().any(|r| r == role), "{name}: missing {role}: {a:?}");
        }
        for role in no {
            assert!(!a.roles.iter().any(|r| r == role), "{name}: unexpected {role}: {a:?}");
        }
    }
}

#[test]
fn approved_c14_through_c18_use_real_oracle_text() {
    let i = rulings_index();
    for (name, yes, no) in [
        // C14: 1 damage to each creature of one group sweeps it; a single targeted effect does not.
        ("Thundermaw Hellkite", vec!["wipe"], vec!["removal", "evasion", "drain"]),
        ("Breath of Darigaaz", vec!["wipe"], vec!["drain"]),
        // C15: an aura cantrip is not draw; a vehicle sacrificing itself to draw two still is.
        ("Ritual of Steel", vec![], vec!["draw"]),
        ("Daredevil Dragster", vec!["draw"], vec![]),
        // C16: one chosen colour is ramp, not fixing (a second colour beside it still fixes).
        ("Throne of Eldraine", vec!["ramp", "draw"], vec!["fixing"]),
        ("Coldsteel Heart", vec!["ramp"], vec!["fixing"]),
        ("Thriving Grove", vec!["fixing"], vec!["ramp"]),
        // C17: Scroll Rack is a hand filter.
        ("Scroll Rack", vec!["selection"], vec!["draw"]),
        // C18: Treasure makers are tokens and ramp, never fixing.
        ("Smothering Tithe", vec!["tokens", "ramp"], vec!["fixing"]),
        ("Dockside Extortionist", vec!["tokens", "ramp"], vec!["fixing"]),
    ] {
        let a = classify(&i, name);
        for role in yes {
            assert!(a.roles.iter().any(|r| r == role), "{name}: missing {role}: {a:?}");
        }
        for role in no {
            assert!(!a.roles.iter().any(|r| r == role), "{name}: unexpected {role}: {a:?}");
        }
    }
    // The wipe is decided by the rules alone, and Treasure's any-colour mana does not route its maker.
    assert!(!classify(&i, "Thundermaw Hellkite").needs_llm);
    for name in ["Smothering Tithe", "Dockside Extortionist"] {
        let a = classify(&i, name);
        assert!(!a.route.contains(&roles::RouteReason::NoRole), "{name}: {a:?}");
    }
}

#[test]
fn partial_sweeps_cantrips_and_chosen_colour_mana_general_shapes() {
    // C14: any group of creatures, any enter or activated sweep.
    for text in [
        "When this creature enters, it deals 1 damage to each attacking creature.",
        "{R}: This creature deals 1 damage to each tapped creature.",
    ] {
        let a = round5_synthetic("Creature — Elemental", text, &[]);
        assert!(a.roles.contains(&"wipe".into()), "{text}: {a:?}");
    }
    let a =
        round5_synthetic("Creature — Elemental", "When this creature enters, it deals 1 damage to any target.", &[]);
    assert!(!a.roles.contains(&"wipe".into()), "{a:?}");
    // C15: an equipment cantrip and a self-sacrifice draw replace themselves; a repeating draw does not.
    for text in ["When this Equipment enters, draw a card.", "{2}, Sacrifice this artifact: Draw a card."] {
        let typ = if text.contains("Equipment") { "Artifact — Equipment" } else { "Artifact" };
        assert!(!round5_synthetic(typ, text, &[]).roles.contains(&"draw".into()), "{text}");
    }
    let a = round5_synthetic("Artifact — Equipment", "Whenever equipped creature attacks, draw a card.", &[]);
    assert!(a.roles.contains(&"draw".into()), "{a:?}");
    // C16: one mana of the chosen colour on a land is not fixing either.
    let a = round5_synthetic(
        "Artifact",
        "As this artifact enters, choose a color.\n{T}: Add one mana of the chosen color.",
        &[],
    );
    assert!(a.roles.contains(&"ramp".into()) && !a.roles.contains(&"fixing".into()), "{a:?}");
    // C17: the draw-that-many wording of the same hand filter.
    let a = round5_synthetic(
        "Artifact",
        "{1}, {T}: Exile any number of cards from your hand face down, then draw that many cards.",
        &[],
    );
    assert!(a.roles.contains(&"selection".into()) && !a.roles.contains(&"draw".into()), "{a:?}");
    // C18: a Treasure maker whose tokens make WUBRG is not routed, but a real mana source with the same tokens is.
    let maker = "Whenever a creature dies, create a Treasure token. (It's an artifact with \"{T}, Sacrifice this \
                 token: Add one mana of any color.\")";
    let a = round5_synthetic("Enchantment", maker, &["W", "U", "B", "R", "G"]);
    assert!(!a.route.contains(&roles::RouteReason::NoRole) && !a.roles.contains(&"fixing".into()), "{a:?}");
}

#[test]
fn treasure_pair_is_not_a_same_line_route() {
    let a = round5_synthetic("Enchantment", "Whenever an opponent draws a card, create a Treasure token.", &[]);
    assert!(a.roles.contains(&"tokens".into()) && a.roles.contains(&"ramp".into()), "{a:?}");
    assert!(!a.route.contains(&roles::RouteReason::SameLine), "{a:?}");
}

#[test]
fn metadata_only_reader_and_override_fingerprint_invalidation() {
    let e = Env::new();
    e.command(&["--rules"]).unwrap();
    let path = e.0.join("data/knowledge/roles.json");
    let map = roles::load(&path).unwrap();
    assert_eq!(roles::load_meta(&path).unwrap(), map.meta);
    let prefix = format!("{{\"meta\":{},\"cards\":invalid table", serde_json::to_string(&map.meta).unwrap());
    std::fs::write(&path, prefix).unwrap();
    assert_eq!(roles::load_meta(&path).unwrap(), map.meta); // intentionally never reads the table
    assert!(roles::load(&path).is_err());
    std::fs::write(
        e.0.join("data/knowledge/roles_overrides.toml"),
        "[[card]]\nname='Counterspell'\nroles=[]\nnote='changed'\n",
    )
    .unwrap();
    let changed = Index::load(&e.0.join("data/knowledge/oracle.json")).unwrap();
    assert!(map.check_meta(&changed).is_err());
    assert_ne!(roles::rules_hash(b""), roles::rules_hash(b"[[card]]\nname='Counterspell'\nroles=[]\nnote='changed'\n"));
    assert_eq!(roles::rules_hash(b""), roles::rules_hash(b"# comment\n\n")); // comments carry no content
}

#[test]
fn validation_scores_the_supplied_assignments() {
    let e = Env::new();
    let i = Index::load(&e.0.join("data/knowledge/oracle.json")).unwrap();
    let mut map = roles::generate(&i, "").unwrap();
    let id = &card(&i, "Counterspell").raw.oracle_id;
    let gold = format!("[[card]]\nname='Counterspell'\noracle_id='{id}'\nroles=['counter']\nprimary='counter'\n");
    let taxonomy = Taxonomy::parse(TAXONOMY).unwrap();
    let before = roles::validate(&e.project(), &i, &map, &gold, &taxonomy).unwrap();
    assert_eq!(before.gold_micro.tp, 1);
    map.get_mut(id).unwrap().roles.clear();
    map.get_mut(id).unwrap().primary.clear();
    map.get_mut(id).unwrap().evidence.clear();
    let after = roles::validate(&e.project(), &i, &map, &gold, &taxonomy).unwrap();
    assert_eq!(after.gold_micro.tp, 0);
    assert_eq!(after.gold_micro.missed, 1);
    assert_eq!(after.hand_micro.missed, 1);
}

#[test]
fn copy_never_cites_a_prepare_becomes_prepared_line() {
    let mut doc: Value = serde_json::from_str(include_str!("fixtures/roles_oracle.json")).unwrap();
    let record = doc["cards"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|c| c["name"] == "Emeritus of Conflict // Lightning Bolt")
        .unwrap();
    record["card_faces"][1]["oracle_text"] = json!("Copy target instant or sorcery spell.");
    let e = Env::new();
    let path = e.0.join("data/knowledge/oracle.json");
    std::fs::write(&path, doc.to_string()).unwrap();
    let i = Index::load(&path).unwrap();
    let c = card(&i, "Emeritus of Conflict // Lightning Bolt");
    let a = Classifier::new().unwrap().classify(c).unwrap();
    let evidence = &a.evidence["copy"];
    assert_eq!(evidence.face, Some(1));
    assert!(!c.lines()[evidence.line.unwrap()].1.contains("becomes prepared"));
}
