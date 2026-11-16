//! Sorin, Vengeful Bloodlord and Heliod, Sun-Crowned: the card code (AI paths) and the person's paths. Each
//! expectation follows the card's Oracle text (data/scryfall_cache.json).

mod play_common;
use play_common::*;
use sim_core::dsl::has_kw;
use sim_core::engine::values::subtypes;
use sim_core::engine::{life, zones};
use sim_core::hooks::Opt;
use sim_core::human::human::{AbilityKind, abilities_of};
use sim_core::state::Step;

const SORIN: &str = "Sorin, Vengeful Bloodlord";

fn tbl() -> Game {
    let mut g = table(&["heliod-mono-white-stax", "seph", "sauron"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g
}

/// the options of the card code of `src` (none when its card has no `options` hook)
fn options(g: &mut Game, src: PermId, p: PlayerId) -> Vec<Opt> {
    let cd = g.perm(src).cd.unwrap();
    match g.registry.get(cd).and_then(|i| i.options) {
        Some(f) => f(g, src, p, Some(false)).unwrap(),
        None => vec![],
    }
}

fn labels(o: &[Opt]) -> Vec<String> {
    o.iter().map(|x| x.label.clone()).collect()
}

/// the AI uses the option of `src` whose label ends with `end`
fn ai_uses(g: &mut Game, src: PermId, end: &str) {
    let o = options(g, src, P0);
    let found = o.iter().find(|o| o.label.ends_with(end));
    assert!(found.is_some(), "no option ending {end:?} in {:?}", labels(&o));
    assert!(sim_core::ai::act::perform(g, P0, found.unwrap().act.as_ref().unwrap()).unwrap());
}

fn creature_card(g: &mut Game, name: &str) -> sim_core::ids::CardId {
    let c = card(g, name);
    g.player_mut(P0).gy.push(c);
    c
}

/// a person at P0 who answers a question by the first choice that contains `word` (decline otherwise) and passes priority
fn person_picking(g: &mut Game, word: &'static str) -> std::sync::Arc<ScriptHuman> {
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        match r.choices.iter().position(|c| c.contains(word)) {
            Some(i) => Answer::Pick(i),
            None => Answer::Decline,
        }
    }));
    g.humans.add(P0, h.clone());
    h
}

/// the person uses the ability of `m` whose label contains `word`; the reason it could not, if any
fn use_ability(g: &mut Game, m: PermId, word: &str) -> Option<String> {
    let list = abilities_of(g, P0, m).unwrap();
    let all: Vec<String> = list.iter().map(|x| x.0.clone()).collect();
    let found = list.into_iter().find(|(l, _)| l.contains(word));
    assert!(found.is_some(), "{} lists no ability with {word:?}: {all:?}", g.perm(m).name);
    let AbilityKind::Extra(act) = found.unwrap().1 else { panic!("not an extra ability") };
    act(g, P0, m).unwrap()
}

// ------------------------------------------------------------------ Sorin: the static
#[test]
fn sorin_gives_creatures_and_planeswalkers_lifelink_on_your_turn_only() {
    let mut g = tbl();
    let s = perm(&mut g, P0, SORIN);
    let mine = token(&mut g, P0, 2);
    let theirs = token(&mut g, P1, 2);
    assert!(has_kw(&g, mine, "lifelink"), "your creature, on your turn");
    assert!(has_kw(&g, s, "lifelink"), "Sorin is a planeswalker you control");
    assert!(!has_kw(&g, theirs, "lifelink"), "not an opponent's creature");
    g.active = Some(P1);
    assert!(!has_kw(&g, mine, "lifelink"), "not on an opponent's turn");
    assert!(!has_kw(&g, s, "lifelink"));
}

// ------------------------------------------------------------------ Sorin: +2
#[test]
fn sorin_plus_two_hits_a_player_or_planeswalker_and_gains_one() {
    let mut g = tbl();
    let s = perm(&mut g, P0, SORIN);
    let prize = perm(&mut g, P1, "Mother of Runes"); // a valuable creature with 1 toughness: not a legal target
    let (theirs, mine) = (g.player(P1).life + g.player(P2).life, g.player(P0).life);
    ai_uses(&mut g, s, "+2");
    assert!(g.perm(prize).on_bf, "a creature is no target of Sorin's +2");
    assert_eq!(g.player(P1).life + g.player(P2).life, theirs - 1, "1 damage to an opponent");
    assert_eq!(g.player(P0).life, mine + 1, "Sorin is a planeswalker you control: lifelink");
    assert_eq!(g.perm(s).loyalty, Some(6));
}

#[test]
fn sorin_plus_two_finishes_a_planeswalker_with_one_loyalty() {
    let mut g = tbl();
    let s = perm(&mut g, P0, SORIN);
    let w = perm(&mut g, P1, "Jace, the Mind Sculptor");
    g.perm_mut(w).loyalty = Some(1);
    ai_uses(&mut g, s, "+2");
    assert!(!g.perm(w).on_bf, "1 damage kills a planeswalker with 1 loyalty");
}

// ------------------------------------------------------------------ Sorin: -X
#[test]
fn sorin_minus_x_returns_a_creature_card_of_mana_value_x_as_a_vampire() {
    let mut g = tbl();
    let s = perm(&mut g, P0, SORIN);
    creature_card(&mut g, "Mother of Runes");
    let big = creature_card(&mut g, "Serra Angel");
    ai_uses(&mut g, s, "-1");
    let mom = perms_named(&g, P0, "Mother of Runes");
    assert_eq!(mom.len(), 1, "the 1-mana creature came back");
    assert!(subtypes(&g, mom[0]).iter().any(|t| &**t == "vampire"), "a Vampire in addition");
    assert!(subtypes(&g, mom[0]).iter().any(|t| &**t == "human"), "and still a Human");
    assert_eq!(g.perm(s).loyalty, Some(3), "loyalty X = 1 lower");
    assert!(g.player(P0).gy.contains(&big), "Serra Angel has mana value 5");
}

#[test]
fn sorin_minus_zero_returns_a_creature_card_with_mana_value_zero() {
    let mut g = tbl();
    let s = perm(&mut g, P0, SORIN);
    creature_card(&mut g, "Ornithopter");
    ai_uses(&mut g, s, "+0");
    assert_eq!(perms_named(&g, P0, "Ornithopter").len(), 1);
    assert_eq!(g.perm(s).loyalty, Some(4), "X = 0 costs nothing");
}

#[test]
fn the_ai_does_not_return_a_creature_whose_mana_value_is_more_than_sorins_loyalty() {
    let mut g = tbl();
    let s = perm(&mut g, P0, SORIN);
    creature_card(&mut g, "Serra Angel");
    let o = options(&mut g, s, P0);
    assert!(o.iter().all(|x| !x.label.contains("-5")), "{:?}", labels(&o));
    assert!(o.iter().any(|x| x.label.ends_with("+2")), "{:?}: the +2 is there", labels(&o));
}

// ------------------------------------------------------------------ Sorin: coverage (green before and after)
#[test]
fn a_spell_gains_nothing_with_sorin_out() {
    let mut g = tbl();
    perm(&mut g, P0, SORIN);
    let axe = take(&mut g, P0, "Lava Axe");
    let (theirs, mine) = (g.player(P1).life + g.player(P2).life, g.player(P0).life);
    sim_core::dsl::resolve_spell(&mut g, P0, axe, &sim_core::state::Ctx::default()).unwrap();
    assert_eq!(g.player(P1).life + g.player(P2).life, theirs - 5, "Lava Axe: 5 damage");
    assert_eq!(g.player(P0).life, mine, "a spell has no lifelink, Sorin or not");
}

#[test]
fn sorin_gives_no_lifelink_on_an_opponents_turn() {
    let mut g = tbl();
    perm(&mut g, P0, SORIN);
    g.active = Some(P1);
    let blocker = token(&mut g, P0, 3);
    let attacker = token(&mut g, P1, 3);
    let mut tot = 0;
    sim_core::engine::combat::resolve_blocks(&mut g, P1, vec![attacker], P0, vec![(attacker, blocker)], &mut tot)
        .unwrap();
    assert_eq!(g.player(P0).life, 40, "no lifelink on the opponent's turn: nothing gained");
}

// ------------------------------------------------------------------ Sorin: the person's paths
#[test]
fn a_person_is_offered_sorins_abilities_and_asked_the_plus_two_target() {
    let mut g = tbl();
    let h = person_picking(&mut g, "Jace");
    let s = perm(&mut g, P0, SORIN);
    let w = perm(&mut g, P1, "Jace, the Mind Sculptor");
    let bear = perm(&mut g, P1, "Mother of Runes");
    let list: Vec<String> = abilities_of(&mut g, P0, s).unwrap().into_iter().map(|a| a.0).collect();
    assert!(list.iter().any(|l| l.starts_with("+2")), "{list:?}");
    assert!(list.iter().any(|l| l.starts_with("-X")), "{list:?}");
    let mine = g.player(P0).life;
    let loyalty = g.perm(w).loyalty.unwrap();
    assert_eq!(use_ability(&mut g, s, "+2"), None);
    let asked = h.asked();
    let ask = asked.iter().find(|r| r.prompt.contains("Sorin"));
    assert!(ask.is_some(), "the target is asked: {asked:?}");
    let ask = ask.unwrap();
    assert!(ask.choices.iter().any(|c| c.contains("Jace")), "a planeswalker is a legal target: {:?}", ask.choices);
    assert!(ask.choices.iter().any(|c| c.contains("player")), "a player is: {:?}", ask.choices);
    assert!(!ask.choices.iter().any(|c| c.contains("Mother of Runes")), "a creature is not: {:?}", ask.choices);
    assert_eq!(g.perm(w).loyalty, Some(loyalty - 1));
    assert!(g.perm(bear).on_bf);
    assert_eq!(g.perm(s).loyalty, Some(6));
    assert_eq!(g.player(P0).life, mine + 1, "lifelink");
}

#[test]
fn a_person_is_asked_which_creature_sorin_minus_x_returns() {
    let mut g = tbl();
    let h = person_picking(&mut g, "Mother of Runes");
    let s = perm(&mut g, P0, SORIN);
    creature_card(&mut g, "Ornithopter");
    creature_card(&mut g, "Mother of Runes");
    creature_card(&mut g, "Serra Angel");
    assert_eq!(use_ability(&mut g, s, "-X"), None);
    let asked = h.asked();
    let ask = asked.iter().find(|r| r.prompt.contains("Sorin"));
    assert!(ask.is_some(), "the creature card is asked: {asked:?}");
    let ask = ask.unwrap();
    assert!(ask.choices.iter().any(|c| c.contains("Ornithopter")), "{:?}", ask.choices);
    assert!(ask.choices.iter().any(|c| c.contains("Mother of Runes")), "{:?}", ask.choices);
    assert!(!ask.choices.iter().any(|c| c.contains("Serra Angel")), "mana value 5 is more than 4 loyalty");
    assert_eq!(perms_named(&g, P0, "Mother of Runes").len(), 1);
    assert!(subtypes(&g, perms_named(&g, P0, "Mother of Runes")[0]).iter().any(|t| &**t == "vampire"));
    assert_eq!(g.perm(s).loyalty, Some(3), "X is the card's mana value: 1");
}

#[test]
fn a_person_is_told_why_sorin_minus_x_cannot_be_used_with_no_creature_card() {
    let mut g = tbl();
    person_picking(&mut g, "nothing");
    let s = perm(&mut g, P0, SORIN);
    let why = use_ability(&mut g, s, "-X");
    assert!(why.is_some(), "a reason is given");
    let why = why.unwrap();
    assert!(why.contains("graveyard"), "{why}");
    assert_eq!(g.perm(s).loyalty, Some(4), "nothing was paid");
}

// ------------------------------------------------------------------ Heliod, Sun-Crowned
const HELIOD: &str = "Heliod, Sun-Crowned";

fn counters(g: &Game, m: PermId) -> i32 {
    g.perm(m).plus
}

#[test]
fn heliods_counter_can_go_on_heliod_or_an_enchantment() {
    let mut g = tbl();
    let h = perm(&mut g, P0, HELIOD);
    assert!(!g.is_creature(h), "devotion 1: not a creature");
    sim_core::engine::life::gain(&mut g, P0, 1).unwrap();
    assert_eq!(counters(&g, h), 1, "with no creature, a creature or enchantment: Heliod, an enchantment");
    let tg = perm(&mut g, P0, "Mother of Runes");
    sim_core::engine::life::gain(&mut g, P0, 1).unwrap();
    assert_eq!(counters(&g, tg), 1, "a creature first");
    assert_eq!(counters(&g, h), 1);
}

#[test]
fn heliods_counter_goes_on_a_ballista_before_other_creatures() {
    let mut g = tbl();
    perm(&mut g, P0, HELIOD);
    perm(&mut g, P0, "Grave Titan");
    let c = take(&mut g, P0, "Walking Ballista");
    g.last_x = 2;
    let b = sim_core::engine::zones::enter(
        &mut g,
        P0,
        c,
        sim_core::engine::zones::Enter { was_cast: true, sick: false, ..Default::default() },
    )
    .unwrap();
    let before = counters(&g, b);
    sim_core::engine::life::gain(&mut g, P0, 1).unwrap();
    assert_eq!(counters(&g, b), before + 1, "one more ping");
}

#[test]
fn a_person_is_asked_heliods_counter_target() {
    let mut g = tbl();
    let h = person_picking(&mut g, "Rule of Law");
    perm(&mut g, P0, HELIOD);
    let law = perm(&mut g, P0, "Rule of Law");
    let mine = perm(&mut g, P0, "Mother of Runes");
    let theirs = perm(&mut g, P1, "Mother of Runes");
    sim_core::engine::life::gain(&mut g, P0, 1).unwrap();
    let asked = h.asked();
    let ask = asked.iter().find(|r| r.prompt.contains("Heliod"));
    assert!(ask.is_some(), "the target is asked: {asked:?}");
    let ask = ask.unwrap();
    assert!(ask.choices.iter().any(|c| c.contains("Rule of Law")), "an enchantment: {:?}", ask.choices);
    assert!(ask.choices.iter().any(|c| c.contains("Mother of Runes")), "a creature: {:?}", ask.choices);
    assert!(ask.choices.iter().any(|c| c.contains("Heliod")), "Heliod himself: {:?}", ask.choices);
    assert_eq!(ask.choices.len(), 3, "only your own creatures and enchantments: {:?}", ask.choices);
    assert_eq!(counters(&g, law), 1);
    assert_eq!(counters(&g, mine), 0);
    assert_eq!(counters(&g, theirs), 0);
}

#[test]
fn a_person_is_offered_a_heliods_lifelink_grant() {
    let mut g = tbl();
    let h = person_picking(&mut g, "Mother of Runes");
    let hel = perm(&mut g, P0, HELIOD);
    let other = perm(&mut g, P0, "Mother of Runes");
    let list: Vec<String> = abilities_of(&mut g, P0, hel).unwrap().into_iter().map(|a| a.0).collect();
    assert!(list.iter().any(|l| l.contains("{1}{W}") && l.contains("lifelink")), "{list:?}");
    let why = use_ability(&mut g, hel, "lifelink");
    assert!(why.is_some(), "no mana in the pool: a reason is given");
    let why = why.unwrap();
    assert!(why.contains("mana") || why.contains("pay") || why.contains("{1}"), "{why}");
    g.player_mut(P0).pool.add('W', 2);
    assert_eq!(use_ability(&mut g, hel, "lifelink"), None);
    assert_eq!(g.player(P0).pool.total(), 0, "{{1}}{{W}} came out of the pool");
    let asked = h.asked();
    let ask = asked.iter().find(|r| r.prompt.contains("Heliod"));
    assert!(ask.is_some(), "the target is asked: {asked:?}");
    let ask = ask.unwrap();
    assert!(!ask.choices.iter().any(|c| c.contains("Heliod")), "another creature: {:?}", ask.choices);
    assert!(has_kw(&g, other, "lifelink"), "until end of turn");
}

#[test]
fn the_ai_grants_lifelink_with_spare_mana_before_combat() {
    let mut g = tbl();
    let hel = perm(&mut g, P0, HELIOD);
    let big = token(&mut g, P0, 4);
    lands(&mut g, P0, "Plains", 2, false);
    let o = options(&mut g, hel, P0);
    assert!(o.iter().any(|x| x.label.contains("lifelink")), "{:?}", labels(&o));
    ai_uses(&mut g, hel, &o.iter().find(|x| x.label.contains("lifelink")).unwrap().label.clone());
    assert!(has_kw(&g, big, "lifelink"));
    assert!(options(&mut g, hel, P0).iter().all(|x| !x.label.contains("lifelink")), "once a turn");
}

/// Heliod and a creature of power `pow` on P0's table, with `plains` untapped Plains: the AI's options of Heliod
fn heliod_options_with(pow: i32, plains: usize, f: impl Fn(&mut Game, PermId)) -> Vec<Opt> {
    let mut g = tbl();
    let hel = perm(&mut g, P0, HELIOD);
    let big = token(&mut g, P0, pow);
    f(&mut g, big);
    lands(&mut g, P0, "Plains", plains, false);
    options(&mut g, hel, P0)
}

fn grants(o: &[Opt]) -> bool {
    o.iter().any(|x| x.label.contains("lifelink"))
}

#[test]
fn the_ai_does_not_grant_lifelink_to_a_creature_of_power_two() {
    let o = heliod_options_with(2, 2, |_, _| {});
    assert!(!grants(&o), "not worth {{1}}{{W}}: {:?}", labels(&o));
}

#[test]
fn the_ai_does_not_grant_lifelink_without_the_mana() {
    let o = heliod_options_with(4, 0, |_, _| {});
    assert!(!grants(&o), "no mana: {:?}", labels(&o));
}

#[test]
fn the_ai_does_not_grant_lifelink_to_a_creature_that_has_it() {
    let o = heliod_options_with(4, 2, |g, m| g.perm_mut(m).lifelink = true);
    assert!(!grants(&o), "it has lifelink already: {:?}", labels(&o));
}

#[test]
fn the_ai_does_not_grant_lifelink_to_a_creature_that_cannot_attack() {
    let o = heliod_options_with(4, 2, |g, m| g.perm_mut(m).sick = true);
    assert!(!grants(&o), "summoning sick: {:?}", labels(&o));
}

// ------------------------------------------------------------------ fix round
/// `m` has keyword `kw` until end of turn
fn give(g: &mut Game, m: PermId, kw: &str) {
    g.perm_mut(m).eot_kw.push(sim_core::sym::intern(kw));
}

/// a vanilla creature of power `power` and toughness `toughness`, not summoning sick
fn body(g: &mut Game, p: PlayerId, power: i32, toughness: i32) -> PermId {
    let m = token(g, p, power.max(1));
    g.perm_mut(m).pow = power;
    g.perm_mut(m).tgh = toughness;
    m
}

/// a Walking Ballista with `n` +1/+1 counters, not summoning sick
fn ballista(g: &mut Game, n: i32) -> PermId {
    let c = take(g, P0, "Walking Ballista");
    g.last_x = 2 * n;
    let b = zones::enter(g, P0, c, zones::Enter { was_cast: true, sick: false, ..Default::default() }).unwrap();
    assert_eq!(g.perm(b).plus, n);
    b
}

#[test]
fn ballista_reads_lifelink_at_resolution_if_it_is_still_on_the_battlefield() {
    let mut g = tbl();
    let hel = perm(&mut g, P0, HELIOD);
    let b = ballista(&mut g, 2);
    let hel_i = g.player(P0).perms.iter().position(|&x| x == hel).unwrap();
    g.player_mut(P0).pool.add('W', 2);
    let mut granted = false;
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" {
            if !granted {
                granted = true;
                return Answer::Action(json!({"do": "use", "perm": hel_i}));
            }
            return Answer::Action(json!({"do": "pass"}));
        }
        if r.prompt.contains("remove how many") {
            return Answer::Pick(0);
        }
        let word = if r.prompt.contains("which ability") {
            "lifelink"
        } else if r.prompt.contains("Heliod") {
            "Walking Ballista"
        } else {
            "(player,"
        };
        r.choices
            .iter()
            .position(|c| c.contains(word) && (word != "(player," || !c.contains("(you)")))
            .map_or(Answer::Decline, Answer::Pick)
    }));
    h.set_autopass("all");
    g.humans.add(P0, h);
    assert_eq!(use_ability(&mut g, b, "remove a +1/+1 counter"), None);
    assert!(g.perm(b).on_bf);
    assert!(life::has_lifelink(&g, b), "the grant resolved before the ping");
    assert_eq!(g.player(P0).life, 41, "lifelink is read when the ping deals its damage");
}

#[test]
fn heliod_cannot_save_a_ballista_that_paid_its_last_counter() {
    let mut g = tbl();
    let hel = perm(&mut g, P0, HELIOD);
    let b = ballista(&mut g, 1);
    g.perm_mut(b).lifelink = true;
    // aim the ping at an opponent; aim Heliod's gain trigger at the Ballista if it is wrongly still there
    let p = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        let word = if r.prompt.contains("Heliod") { "Walking Ballista" } else { "(player," };
        r.choices
            .iter()
            .position(|c| c.contains(word) && (word != "(player," || !c.contains("(you)")))
            .map_or(Answer::Decline, Answer::Pick)
    }));
    g.humans.add(P0, p);
    assert_eq!(use_ability(&mut g, b, "remove a +1/+1 counter"), None);
    assert_eq!(g.player(P0).life, 41, "the ping still resolves, with last known information");
    assert!(!g.perm(b).on_bf, "the 0/0 dies before the ping and Heliod's trigger resolve");
    assert_eq!(g.perm(hel).plus, 1, "Heliod is the only legal counter target");
}

fn prot_white(g: &mut Game, m: PermId) {
    give(g, m, "pro_W");
    assert!(sim_core::engine::values::protected_from(g, m, sim_core::cards::Colors::from_letters("W")));
}

#[test]
fn heliod_counter_cannot_target_a_creature_with_protection_from_white() {
    let mut g = tbl();
    person_picking(&mut g, "Mother of Runes");
    let h = perm(&mut g, P0, HELIOD);
    let m = perm(&mut g, P0, "Mother of Runes");
    prot_white(&mut g, m);
    life::gain(&mut g, P0, 1).unwrap();
    assert_eq!(g.perm(m).plus, 0, "a creature with protection from white is no legal target");
    assert_eq!(g.perm(h).plus, 1, "Heliod himself is");
}

#[test]
fn heliod_asks_for_a_target_once_per_trigger_not_during_probe() {
    let mut g = tbl();
    let h = person_picking(&mut g, "Heliod");
    perm(&mut g, P0, HELIOD);
    life::gain(&mut g, P0, 1).unwrap();
    let asks = h.asked().into_iter().filter(|r| r.kind == "target" && r.prompt.contains("Heliod")).count();
    assert_eq!(asks, 1, "one trigger, one target selection");
}

#[test]
fn heliod_ai_does_not_target_a_protected_creature() {
    let mut g = tbl();
    let h = perm(&mut g, P0, HELIOD);
    let t = body(&mut g, P0, 4, 4);
    prot_white(&mut g, t);
    lands(&mut g, P0, "Plains", 2, false);
    let opts = options(&mut g, h, P0);
    assert!(opts.iter().all(|o| !o.label.contains("lifelink")), "{:?}", labels(&opts));
}

#[test]
fn heliod_rechecks_protection_when_the_counter_trigger_resolves() {
    let mut g = tbl();
    perm(&mut g, P0, HELIOD);
    let m = perm(&mut g, P0, "Mother of Runes");
    let giver = perm(&mut g, P0, "Giver of Runes");
    let giver_i = g.player(P0).perms.iter().position(|&x| x == giver).unwrap();
    let mut protected = false;
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" {
            if !protected {
                protected = true;
                return Answer::Action(json!({"do": "use", "perm": giver_i}));
            }
            return Answer::Action(json!({"do": "pass"}));
        }
        let word = if r.prompt.contains("which ability") {
            "protection"
        } else if r.prompt.contains("protection from what") {
            "white"
        } else {
            "Mother of Runes"
        };
        r.choices.iter().position(|c| c.contains(word)).map_or(Answer::Decline, Answer::Pick)
    }));
    h.set_autopass("all");
    g.humans.add(P0, h);
    life::gain(&mut g, P0, 1).unwrap();
    assert!(sim_core::engine::values::protected_from(&g, m, sim_core::cards::Colors::from_letters("W")));
    assert_eq!(g.perm(m).plus, 0, "the only target became illegal");
}

#[test]
fn the_ai_grant_does_nothing_if_the_target_gained_protection_before_it_resolved() {
    let mut g = tbl();
    let hel = perm(&mut g, P0, HELIOD);
    let t = body(&mut g, P0, 4, 4);
    lands(&mut g, P0, "Plains", 2, false);
    let o = options(&mut g, hel, P0);
    let found = o.iter().find(|x| x.label.contains("lifelink"));
    assert!(found.is_some(), "the grant is offered: {:?}", labels(&o));
    prot_white(&mut g, t); // in response
    let _ = sim_core::ai::act::perform(&mut g, P0, found.unwrap().act.as_ref().unwrap()).unwrap(); // not done, or done to nothing
    assert!(!has_kw(&g, t, "lifelink"), "the target is no longer legal: nothing happens");
}

#[test]
fn a_persons_grant_rechecks_its_target_on_resolution() {
    let mut g = tbl();
    let hel = perm(&mut g, P0, HELIOD);
    let m = perm(&mut g, P0, "Mother of Runes");
    let giver = perm(&mut g, P0, "Giver of Runes");
    let giver_i = g.player(P0).perms.iter().position(|&x| x == giver).unwrap();
    g.player_mut(P0).pool.add('W', 2);
    let mut protected = false;
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" {
            if !protected {
                protected = true;
                return Answer::Action(json!({"do": "use", "perm": giver_i}));
            }
            return Answer::Action(json!({"do": "pass"}));
        }
        let word = if r.prompt.contains("which ability") {
            "protection"
        } else if r.prompt.contains("protection from what") {
            "white"
        } else {
            "Mother of Runes"
        };
        r.choices.iter().position(|c| c.contains(word)).map_or(Answer::Decline, Answer::Pick)
    }));
    h.set_autopass("all");
    g.humans.add(P0, h);
    assert_eq!(use_ability(&mut g, hel, "lifelink"), None);
    assert!(sim_core::engine::values::protected_from(&g, m, sim_core::cards::Colors::from_letters("W")));
    assert!(!has_kw(&g, m, "lifelink"), "protection from white in response: the grant has no legal target");
}

#[test]
fn the_ai_keeps_a_held_answers_mana_instead_of_granting_lifelink() {
    let mut g = table(&["zur-esper-enchantment-control", "seph", "sauron"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g.player_mut(P0).turns = 6;
    let hel = perm(&mut g, P0, HELIOD);
    token(&mut g, P0, 4);
    hand(&mut g, P0, &["Counterspell"]);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Plains", 1, false);
    let o = options(&mut g, hel, P0);
    assert!(!grants(&o), "{{1}}{{W}} on top of the held Counterspell ({{U}}{{U}}) would tap it out: {:?}", labels(&o));
    lands(&mut g, P0, "Plains", 1, false); // a fourth land: both are affordable
    assert!(grants(&options(&mut g, hel, P0)), "with mana to spare the grant is offered");
}
