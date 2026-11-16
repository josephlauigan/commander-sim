//! Lifelink on damage that is not combat damage (CR 702.15b: any damage dealt by a source with lifelink makes its
//! controller gain that much life, to a player, a creature or a planeswalker), on the sites where a permanent a deck can
//! give lifelink to deals damage: Walking Ballista, Triskelion, Orcish Bowmasters, Siege-Gang Commander's ping and a
//! compiled permanent's damage (Firemane Avenger), each for the AI's path and the person's.

mod play_common;
use play_common::*;
use sim_core::ai::act::perform;
use sim_core::engine::zones;
use sim_core::hooks::Opt;
use sim_core::human::human::{AbilityKind, abilities_of};
use sim_core::state::Ctx;

fn tbl() -> Game {
    let mut g = table(&["heliod-mono-white-stax", "seph", "sauron"]);
    g.active = Some(P0);
    g
}

/// a Walking Ballista with `counters` +1/+1 counters, not summoning sick
fn ballista(g: &mut Game, counters: i32) -> PermId {
    let c = take(g, P0, "Walking Ballista");
    g.last_x = counters * 2;
    let b = zones::enter(g, P0, c, zones::Enter { was_cast: true, sick: false, ..Default::default() }).unwrap();
    assert!(g.perm(b).on_bf);
    assert_eq!(g.perm(b).plus, counters);
    b
}

fn options(g: &mut Game, src: PermId, p: PlayerId) -> Vec<Opt> {
    let cd = g.perm(src).cd.unwrap();
    let f = g.registry.get(cd).and_then(|i| i.options).expect("the card has options");
    f(g, src, p, Some(false)).unwrap()
}

/// the AI uses the option of `src` whose label contains `word`
fn ai_uses(g: &mut Game, src: PermId, word: &str) {
    let o = options(g, src, P0).into_iter().find(|o| o.label.contains(word)).unwrap_or_else(|| panic!("no {word}"));
    assert!(perform(g, P0, o.act.as_ref().unwrap()).unwrap());
}

/// a person at P0 who aims every damage at the first opponent player (`who`: None) or at the choice containing `who`,
/// and passes priority
fn person(g: &mut Game, who: Option<&'static str>) -> std::sync::Arc<ScriptHuman> {
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        let p = r.prompt.to_lowercase();
        if p.contains("remove how many") {
            return Answer::Pick(0); // one counter
        }
        if p.contains("damage") {
            return match who {
                Some(w) => r.choices.iter().position(|c| c.contains(w)).map_or(Answer::Decline, Answer::Pick),
                None => r
                    .choices
                    .iter()
                    .position(|c| c.contains("(player,") && !c.contains("(you)"))
                    .map_or(Answer::Decline, Answer::Pick),
            };
        }
        Answer::Decline
    }));
    g.humans.add(P0, h.clone());
    h
}

/// the person uses the ability of `m` whose label contains `word`; the reason it could not, if any
fn use_ability(g: &mut Game, m: PermId, word: &str) -> Option<String> {
    let list = abilities_of(g, P0, m).unwrap();
    let Some((_, kind)) = list.into_iter().find(|(l, _)| l.contains(word)) else {
        panic!("no ability with {word:?} on {}", g.perm(m).name)
    };
    let AbilityKind::Extra(act) = kind else { panic!("not an extra ability") };
    act(g, P0, m).unwrap()
}

// ------------------------------------------------------------------ Walking Ballista
#[test]
fn ballista_ping_with_lifelink_gains_one_per_ping_for_the_ai() {
    let mut g = tbl();
    let b = ballista(&mut g, 3);
    g.player_mut(P1).life = 3; // lethal: the AI pings the player
    ai_uses(&mut g, b, "ping");
    assert_eq!(g.player(P1).life, 2);
    assert_eq!(g.player(P0).life, 40, "no lifelink, no gain");
    g.perm_mut(b).lifelink = true;
    ai_uses(&mut g, b, "ping");
    assert_eq!(g.player(P1).life, 1);
    assert_eq!(g.player(P0).life, 41, "one damage, one life");
}

#[test]
fn ballista_ping_with_lifelink_gains_for_a_person() {
    let mut g = tbl();
    person(&mut g, None);
    let b = ballista(&mut g, 2);
    g.perm_mut(b).lifelink = true;
    assert_eq!(use_ability(&mut g, b, "remove a +1/+1 counter"), None);
    assert_eq!(g.player(P1).life, 39);
    assert_eq!(g.player(P0).life, 41);
}

#[test]
fn a_ballista_at_its_last_counter_still_gains_what_it_dealt() {
    let mut g = tbl();
    person(&mut g, None);
    let b = ballista(&mut g, 1);
    g.perm_mut(b).lifelink = true;
    assert_eq!(use_ability(&mut g, b, "remove a +1/+1 counter"), None);
    assert!(!g.perm(b).on_bf, "0/0: it died");
    assert_eq!(g.player(P1).life, 39);
    assert_eq!(g.player(P0).life, 41, "last known information: it had lifelink");
}

#[test]
fn lifelink_damage_to_a_creature_counts_all_of_it() {
    let mut g = tbl();
    person(&mut g, Some("Grave Titan"));
    let b = ballista(&mut g, 1);
    g.perm_mut(b).lifelink = true;
    let big = perm(&mut g, P1, "Grave Titan");
    assert_eq!(use_ability(&mut g, b, "remove a +1/+1 counter"), None);
    assert!(g.perm(big).on_bf, "1 damage does not kill a 6/6");
    assert_eq!(g.player(P0).life, 41, "but it was dealt, so it gains");
}

// ------------------------------------------------------------------ Triskelion
#[test]
fn triskelion_ping_with_lifelink_gains_for_the_ai() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Triskelion");
    g.perm_mut(t).lifelink = true;
    g.player_mut(P1).life = 3;
    ai_uses(&mut g, t, "ping");
    assert_eq!(g.player(P1).life, 2);
    assert_eq!(g.player(P0).life, 41);
}

#[test]
fn triskelion_ping_with_lifelink_gains_for_a_person() {
    let mut g = tbl();
    person(&mut g, None);
    let t = perm(&mut g, P0, "Triskelion");
    g.perm_mut(t).lifelink = true;
    assert_eq!(use_ability(&mut g, t, "remove a +1/+1 counter"), None);
    assert_eq!(g.player(P1).life, 39);
    assert_eq!(g.player(P0).life, 41);
}

#[test]
fn triskelion_with_a_grant_drains_through_sanguine_bond() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Triskelion");
    g.perm_mut(t).lifelink = true;
    perm(&mut g, P0, "Sanguine Bond");
    g.player_mut(P1).life = 3;
    ai_uses(&mut g, t, "ping");
    assert_eq!(g.player(P0).life, 41, "the ping's lifelink");
    assert_eq!(g.player(P1).life + g.player(P2).life, 3 + 40 - 2, "1 damage, then Sanguine Bond: an opponent loses 1 more");
}

// ------------------------------------------------------------------ Orcish Bowmasters
#[test]
fn bowmasters_with_lifelink_gains_on_entering_for_the_ai() {
    let mut g = tbl();
    let b = perm(&mut g, P0, "Orcish Bowmasters");
    let (mine, theirs) = (g.player(P0).life, g.player(P1).life + g.player(P2).life);
    g.perm_mut(b).lifelink = true;
    zones::etb_once(&mut g, P0, b).unwrap();
    assert_eq!(g.player(P1).life + g.player(P2).life, theirs - 1, "its ping");
    assert_eq!(g.player(P0).life, mine + 1, "lifelink");
}

#[test]
fn bowmasters_with_lifelink_gains_on_entering_for_a_person() {
    let mut g = tbl();
    let b = perm(&mut g, P0, "Orcish Bowmasters");
    person(&mut g, None);
    let mine = g.player(P0).life;
    g.perm_mut(b).lifelink = true;
    zones::etb_once(&mut g, P0, b).unwrap();
    assert_eq!(g.player(P0).life, mine + 1);
}

#[test]
fn bowmasters_with_lifelink_gains_on_an_opponents_extra_draw_for_the_ai() {
    let mut g = tbl();
    let b = perm(&mut g, P0, "Orcish Bowmasters");
    let mine = g.player(P0).life;
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!(g.player(P0).life, mine, "no lifelink, no gain");
    g.perm_mut(b).lifelink = true;
    let theirs = g.player(P1).life;
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!(g.player(P1).life, theirs - 1);
    assert_eq!(g.player(P0).life, mine + 1);
}

#[test]
fn bowmasters_with_lifelink_gains_on_an_opponents_extra_draw_for_a_person() {
    let mut g = tbl();
    person(&mut g, None);
    let b = perm(&mut g, P0, "Orcish Bowmasters");
    g.perm_mut(b).lifelink = true;
    let mine = g.player(P0).life;
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!(g.player(P0).life, mine + 1);
}

// ------------------------------------------------------------------ Siege-Gang Commander
#[test]
fn siege_gang_ping_with_lifelink_gains() {
    use sim_core::impls::model_gaps_a4::{Tgt, ping};
    let mut g = tbl();
    let s = perm(&mut g, P0, "Siege-Gang Commander");
    ping(&mut g, P0, s, Tgt::Player(P1), 2).unwrap();
    assert_eq!(g.player(P0).life, 40);
    g.perm_mut(s).lifelink = true;
    ping(&mut g, P0, s, Tgt::Player(P1), 2).unwrap();
    assert_eq!(g.player(P1).life, 36);
    assert_eq!(g.player(P0).life, 42);
}

// ------------------------------------------------------------------ a compiled permanent's damage
#[test]
fn a_compiled_permanent_damage_ability_with_lifelink_gains() {
    let mut g = tbl();
    let fa = perm(&mut g, P0, "Firemane Avenger");
    let t1 = token(&mut g, P0, 1);
    let t2 = token(&mut g, P0, 1);
    g.perm_mut(fa).lifelink = true;
    let theirs = g.player(P1).life + g.player(P2).life;
    sim_core::engine::combat::attack_triggers(&mut g, P0, &[fa, t1, t2], P1).unwrap();
    assert_eq!(g.player(P1).life + g.player(P2).life, theirs - 3, "3 damage");
    assert_eq!(g.player(P0).life, 46, "its own 3, and 3 more from lifelink");
}

#[test]
fn a_compiled_permanent_damage_ability_with_lifelink_gains_for_a_person() {
    let mut g = tbl();
    person(&mut g, None);
    let fa = perm(&mut g, P0, "Firemane Avenger");
    let t1 = token(&mut g, P0, 1);
    let t2 = token(&mut g, P0, 1);
    g.perm_mut(fa).lifelink = true;
    sim_core::engine::combat::attack_triggers(&mut g, P0, &[fa, t1, t2], P1).unwrap();
    assert_eq!(g.player(P1).life, 37);
    assert_eq!(g.player(P0).life, 46);
}

#[test]
fn compiled_damage_that_kills_a_creature_gains_the_damage_dealt_for_a_person() {
    let mut g = tbl();
    person(&mut g, Some("Mother of Runes"));
    let fa = perm(&mut g, P0, "Firemane Avenger");
    let t1 = token(&mut g, P0, 1);
    let t2 = token(&mut g, P0, 1);
    g.perm_mut(fa).lifelink = true;
    let mom = perm(&mut g, P1, "Mother of Runes");
    sim_core::engine::combat::attack_triggers(&mut g, P0, &[fa, t1, t2], P1).unwrap();
    assert!(!g.perm(mom).on_bf, "3 damage kills a 1/1");
    assert_eq!(g.player(P0).life, 46, "the 3 damage dealt, not the 1 toughness");
}

// ------------------------------------------------------------------ coverage rows (green before and after)
#[test]
fn a_spell_gains_nothing_with_a_lifelinker_out() {
    let mut g = tbl();
    let k = token(&mut g, P0, 2);
    g.perm_mut(k).lifelink = true;
    let axe = take(&mut g, P0, "Lava Axe");
    let before = (g.player(P0).life, g.player(P1).life + g.player(P2).life);
    sim_core::dsl::resolve_spell(&mut g, P0, axe, &Ctx::default()).unwrap();
    assert_eq!(g.player(P1).life + g.player(P2).life, before.1 - 5, "Lava Axe: 5 damage");
    assert_eq!(g.player(P0).life, before.0, "a spell has no lifelink");
}

#[test]
fn prevented_damage_gains_nothing() {
    let mut g = tbl();
    let b = ballista(&mut g, 2);
    g.perm_mut(b).lifelink = true;
    g.player_mut(P1).life = 2;
    lands(&mut g, P1, "Glacial Chasm", 1, false);
    ai_uses(&mut g, b, "ping");
    assert_eq!(g.player(P1).life, 2, "Glacial Chasm prevents all damage to its controller");
    assert_eq!(g.player(P0).life, 40, "nothing was dealt: nothing gained");
}
