//! Practice mode: targets and activated abilities for the human seat (Python's tests/test_play_abilities.py). Targets
//! follow the rules (your own permanents, players for burn; hexproof and protection respected), are chosen before
//! paying, and can be cancelled. Equip and card abilities (planeswalkers) work from the permanent.

mod play_common;
use play_common::*;
use sim_core::control::Human;
use sim_core::engine::zones;
use sim_core::human::human::{abilities_of, human_main};
use sim_core::human::{Target, choices, legal, mana};
use sim_core::sym::Sym;

/// an answer that picks the choice describing x (targets are listed by description)
fn pick_target(g: &Game, p: PlayerId, x: Target) -> F {
    let label = legal::describe_target(g, p, x);
    Box::new(move |r| {
        Answer::Pick(
            r.choices.iter().position(|c| *c == label).unwrap_or_else(|| panic!("{label:?} not in {:?}", r.choices)),
        )
    })
}

fn tap_all(g: &mut Game, p: PlayerId) {
    while let Some(s) = mana::sources(g, p).first() {
        let id = s.id as i64;
        mana::tap(g, p, id, None).unwrap();
    }
}

// ------------------------------------------------------------------ Targets
#[test]
fn what_a_removal_spell_can_target() {
    let mut g = table(&["sauron", "veyran"]);
    let c = hand(&mut g, P0, &["Terminate"])[0];
    let mine = perm(&mut g, P0, "Orcish Bowmasters");
    let theirs = perm(&mut g, P1, "Guttersnipe");
    let cloaked = perm(&mut g, P1, "Young Pyromancer");
    let cloak = perm(&mut g, P1, "Whispersilk Cloak");
    g.perm_mut(cloak).attached = Some(cloaked);
    let tg = legal::spell_targets(&g, P0, c).unwrap();
    assert!(tg.contains(&Target::Perm(mine)));
    assert!(tg.contains(&Target::Perm(theirs)));
    assert!(!tg.contains(&Target::Perm(cloaked)));
    assert!(!tg.contains(&Target::Player(P1))); // Terminate can't target players
}

#[test]
fn burn_can_target_players() {
    let mut g = table(&["veyran", "sauron"]);
    let bolt = card(&g, "Lightning Bolt");
    let tg = legal::spell_targets(&g, P0, bolt).unwrap();
    assert!(tg.contains(&Target::Player(P1)));
    assert!(tg.contains(&Target::Player(P0)));
    g.player_mut(P1).life_locked = true; // Teferi's Protection
    assert!(!legal::spell_targets(&g, P0, bolt).unwrap().contains(&Target::Player(P1)));
}

#[test]
fn cast_with_a_target() {
    let mut g = table(&["sauron", "veyran"]);
    hand(&mut g, P0, &["Terminate"]);
    lands(&mut g, P0, "Swamp", 1, false);
    lands(&mut g, P0, "Mountain", 1, false);
    tap_all(&mut g, P0);
    let gs = perm(&mut g, P1, "Guttersnipe");
    let pt = pick_target(&g, P0, Target::Perm(gs));
    let h = seat(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), pt, pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(!g.player(P1).perms.contains(&gs));
    assert_eq!(&*h.asked()[1].kind, "target");
}

#[test]
fn cancelling_a_target_costs_nothing() {
    let mut g = table(&["sauron", "veyran"]);
    hand(&mut g, P0, &["Terminate"]);
    lands(&mut g, P0, "Swamp", 1, false);
    lands(&mut g, P0, "Mountain", 1, false);
    tap_all(&mut g, P0);
    perm(&mut g, P1, "Guttersnipe");
    seat(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), cancel(), pass()]);
    human_main(&mut g, P0, false).unwrap();
    let t = card(&g, "Terminate");
    assert!(g.player(P0).hand.contains(&t));
    assert_eq!(pool_total(&g, P0), 2);
}

#[test]
fn bloodchiefs_thirst_kicker() {
    let mut g = table(&["sauron", "veyran"]);
    let c = hand(&mut g, P0, &["Bloodchief's Thirst"])[0];
    lands(&mut g, P0, "Swamp", 1, false);
    tap_all(&mut g, P0);
    perm(&mut g, P1, "Llanowar Elves");
    assert_eq!(legal::check_cast(&mut g, P0, c, "hand"), None); // {B} unkicked, at a mana value 1 creature
    let big = perm(&mut g, P1, "Sheoldred, the Apocalypse");
    let pt = pick_target(&g, P0, Target::Perm(big));
    let h = seat(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), pt, pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(g.player(P1).perms.contains(&big)); // mana value 4 needs the kicker: {2}{B} more
    assert!(h.told()[0].1.contains("Missing {2}{B}"), "{:?}", h.told());
}

// ------------------------------------------------------------------ Abilities
#[test]
fn equip() {
    let mut g = table(&["sauron", "veyran"]);
    zones::amass(&mut g, P0, 3).unwrap();
    let army = sim_core::engine::values::army_of(&g, P0).unwrap();
    let sword = perm(&mut g, P0, "Sword of Feast and Famine");
    lands(&mut g, P0, "Swamp", 2, false);
    tap_all(&mut g, P0);
    let i = g.player(P0).perms.iter().position(|&x| x == sword).unwrap();
    let pt = pick_target(&g, P0, Target::Perm(army));
    seat(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pick(0), pt, pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert_eq!(g.perm(sword).attached, Some(army));
    assert_eq!(pool_total(&g, P0), 0);
}

#[test]
fn equip_needs_mana_and_your_main_phase() {
    let mut g = table(&["sauron", "veyran"]);
    zones::amass(&mut g, P0, 3).unwrap();
    let army = sim_core::engine::values::army_of(&g, P0).unwrap();
    let sword = perm(&mut g, P0, "Sword of Feast and Famine");
    assert!(legal::check_equip(&g, P0, sword, army).unwrap().contains("Missing {2}"));
    g.active = Some(P1);
    assert_eq!(legal::check_equip(&g, P0, sword, army).as_deref(), Some("It isn't your turn."));
}

#[test]
fn a_planeswalker_ability() {
    let mut g = table(&["sauron", "veyran"]);
    let vr = perm(&mut g, P0, "Vraska, Betrayal's Sting");
    let abil = abilities_of(&mut g, P0, vr).unwrap();
    assert!(!abil.is_empty(), "Vraska offers loyalty abilities");
    let start = g.perm(vr).loyalty.unwrap();
    let i = g.player(P0).perms.iter().position(|&x| x == vr).unwrap();
    seat(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pick(0), pass()]); // 0: draw, lose 1, proliferate
    human_main(&mut g, P0, false).unwrap();
    assert_eq!((g.perm(vr).loyalty, g.player(P0).life), (Some(start + 1), 39)); // proliferate counts Vraska too
}

#[test]
fn nothing_to_activate() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P0, "Orcish Bowmasters");
    let h = seat(&mut g, P0, vec![act(json!({"do": "use", "perm": 0})), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(h.told()[0].1.contains("no ability"), "{:?}", h.told());
}

// ------------------------------------------------------------------ Table refs
#[test]
fn choices_point_at_the_table() {
    let mut g = table(&["sauron", "veyran"]);
    let gs = perm(&mut g, P1, "Guttersnipe");
    let c = hand(&mut g, P0, &["Terminate"])[0];
    let gl = legal::describe_target(&g, P0, Target::Perm(gs));
    let labels = vec![
        gl.clone(),
        legal::describe_target(&g, P0, Target::Player(P1)),
        choices::card_label(&g, c),
        format!("{gl} (X = 3)"),
        "a mode".to_string(),
    ];
    let refs = choices::table_refs(&g, P0, &labels);
    let idx = g.player(P1).perms.iter().position(|&x| x == gs).unwrap();
    assert_eq!(refs[0], json!({"seat": "veyran", "perm": idx}));
    assert_eq!(refs[1], json!({"player": "veyran"}));
    assert_eq!(refs[2], json!({"hand": 0}));
    assert_eq!(refs[3], refs[0]);
    assert!(refs[4].is_null());
}

/// a person at the browser: live, answers the first choice, keeps what it was asked
struct Browser(std::sync::Mutex<Vec<sim_core::control::Request>>);
impl Human for Browser {
    fn ask(&self, _g: &Game, _seat: PlayerId, req: sim_core::control::Request) -> Answer {
        self.0.lock().unwrap().push(req);
        Answer::Pick(0)
    }
    fn tell(&self, _seat: PlayerId, _kind: Sym, _text: &str) {}
    fn live(&self) -> bool {
        true
    }
}

#[test]
fn the_browser_gets_them() {
    let mut g = table(&["sauron", "veyran"]);
    perm(&mut g, P1, "Guttersnipe");
    let b = std::sync::Arc::new(Browser(Default::default()));
    g.humans.add(P0, b.clone());
    let m = g.player(P1).perms[0];
    let label = legal::describe_target(&g, P0, Target::Perm(m));
    choices::choose(&g, P0, "target", "pick", &[label], Some("cancel"), None).unwrap();
    let got = b.0.lock().unwrap();
    assert_eq!(got[0].data["refs"], json!([{"seat": "veyran", "perm": 0}]));
}

// ------------------------------------------------------------------ Overload
#[test]
fn only_overloaded_with_nothing_to_target() {
    let mut g = table(&["sauron", "veyran"]);
    hand(&mut g, P0, &["Vandalblast"]);
    pool(&mut g, P0, &[('R', 1), ('C', 4)]);
    let h = seat(&mut g, P0, vec![act(json!({"do": "cast", "card": 0})), pick(0), pass()]);
    human_main(&mut g, P0, false).unwrap();
    let asked = h.asked();
    assert!(asked[1].prompt.contains("only be cast overloaded"), "{}", asked[1].prompt);
    assert_eq!(asked[1].choices.len(), 2); // overloaded, or cancel
    let v = card(&g, "Vandalblast");
    assert!(g.player(P0).gy.contains(&v));
}

// ------------------------------------------------------------------ compiled abilities
#[test]
fn wayfarers_bauble_from_the_pool() {
    // JD (10/21): two {C} in the pool, Wayfarer's Bauble on the battlefield: its compiled ability wasn't offered
    let mut g = table(&["karn", "veyran"]);
    lands(&mut g, P0, "Wastes", 2, false);
    let b = perm(&mut g, P0, "Wayfarer's Bauble");
    tap_all(&mut g, P0);
    let abil = abilities_of(&mut g, P0, b).unwrap();
    assert!(abil.iter().any(|a| a.0.starts_with("{2}, {T}, sacrifice it")), "{:?}", abil.iter().map(|a| &a.0).collect::<Vec<_>>());
    let i = g.player(P0).perms.iter().position(|&x| x == b).unwrap();
    // use the Bauble, its ability (0), the basic land to fetch (0: the only kind, Wastes), then pass
    seat(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pick(0), pick(0), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(!g.perm(b).on_bf, "the Bauble is sacrificed");
    assert_eq!(g.player(P0).lands.len(), 3, "a basic land is fetched");
    assert!(g.land(*g.player(P0).lands.last().unwrap()).tapped, "tapped");
}

#[test]
fn a_compiled_ability_without_the_mana_says_so() {
    let mut g = table(&["karn", "veyran"]);
    let b = perm(&mut g, P0, "Wayfarer's Bauble");
    let i = g.player(P0).perms.iter().position(|&x| x == b).unwrap();
    let h = seat(&mut g, P0, vec![act(json!({"do": "use", "perm": i})), pick(0), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(g.perm(b).on_bf, "nothing paid, nothing sacrificed");
    assert!(!h.told().is_empty(), "the person is told why");
}
