//! AI logic fixes (2026-11-04, agent L, rust/agents/fixes-2026-11-04.md): the AI's play follows the Oracle text of
//! Resplendent Angel, Iron Man, Blood Artist, Adeline, Armored Skyhunter and Zur the Enchanter. The sims are not run
//! for this batch, so each test checks the AI's behaviour directly.

use sim_core::engine::combat::combat;
use sim_core::engine::hooks::fire_trigger;
use sim_core::engine::values::pval;
use sim_core::engine::zones::{amass, die, draw};
use sim_core::flow::Stop;
use sim_core::hooks::{Action, Call, Event};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

// ------------------------------------------------------------------ Resplendent Angel
#[test]
fn resplendent_angel_ai_pump_is_plus_two_plus_two_and_lifelink() {
    let mut g = table(&["thrasios", "seph"]);
    let a = perm(&mut g, P0, "Resplendent Angel");
    lands(&mut g, P0, "Plains", 6, false);
    let f = g.registry.get(g.perm(a).cd.unwrap()).unwrap().options.unwrap();
    let opts = f(&mut g, a, P0, Some(false)).unwrap();
    let o = opts.iter().find(|o| o.label.contains("Resplendent Angel")).expect("the AI offers the pump");
    let Some(Action::Ability { src, f, arg }) = o.act.clone() else { panic!("not an ability") };
    assert!(f(&mut g, src, P0, arg).unwrap());
    assert_eq!(g.perm(a).eot_pt, (2, 2), "Oracle: +2/+2 until end of turn (it was +2/+0)");
    assert!(g.perm(a).eot_kw.contains(&"lifelink"));
}

// ------------------------------------------------------------------ Iron Man, Armored Avenger
#[test]
fn iron_man_ai_counter_goes_on_its_best_creature_not_only_the_army() {
    // no Army at all: the counter still goes on a creature (it went nowhere before)
    let mut g = table(&["sauron", "seph"]);
    let im = perm(&mut g, P0, "Iron Man, Armored Avenger");
    let tok = token(&mut g, P0, 1);
    draw(&mut g, P0, 1, false).unwrap();
    assert_eq!(g.perm(im).plus + g.perm(tok).plus, 1, "exactly one counter, on a creature");
    let best = if pval(&g, im) >= pval(&g, tok) { im } else { tok };
    assert_eq!(g.perm(best).plus, 1, "on the creature the AI values most");
}

#[test]
fn iron_man_ai_counter_prefers_a_better_creature_to_the_army() {
    let mut g = table(&["sauron", "seph"]);
    perm(&mut g, P0, "Iron Man, Armored Avenger");
    amass(&mut g, P0, 1).unwrap();
    let army = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).army).expect("an Orc Army");
    let big = perm(&mut g, P0, "Craterhoof Behemoth");
    assert!(pval(&g, big) > pval(&g, army), "precondition: {} vs {}", pval(&g, big), pval(&g, army));
    let (a0, b0) = (g.perm(army).plus, g.perm(big).plus);
    draw(&mut g, P0, 1, false).unwrap();
    assert_eq!(g.perm(big).plus, b0 + 1, "the counter goes on the best creature");
    assert_eq!(g.perm(army).plus, a0, "and not on the Army");
}

#[test]
fn iron_man_ai_counter_still_feeds_the_army_when_it_is_the_best() {
    let mut g = table(&["sauron", "seph"]);
    perm(&mut g, P0, "Iron Man, Armored Avenger");
    amass(&mut g, P0, 6).unwrap();
    let army = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).army).expect("an Orc Army");
    let tok = token(&mut g, P0, 1);
    let before = g.perm(army).plus;
    draw(&mut g, P0, 1, false).unwrap();
    assert_eq!(g.perm(army).plus, before + 1);
    assert_eq!(g.perm(tok).plus, 0);
}

// ------------------------------------------------------------------ Blood Artist
fn drain_after_sacrificing_a_token(artists: usize) -> (i32, i32) {
    let mut g = table(&["sheoldred", "seph"]);
    for _ in 0..artists {
        perm(&mut g, P0, "Blood Artist");
    }
    let t = token(&mut g, P0, 1);
    let (me, opp) = (g.player(P0).life, g.player(P1).life);
    die(&mut g, t, "sac".into()).unwrap();
    (me - g.player(P0).life, opp - g.player(P1).life)
}

#[test]
fn blood_artist_drains_once_per_artist() {
    // each Artist's trigger: target player loses 1 life and you gain 1
    assert_eq!(drain_after_sacrificing_a_token(1), (-1, 1));
    assert_eq!(drain_after_sacrificing_a_token(2), (-2, 2), "two Artists drained once before");
    assert_eq!(drain_after_sacrificing_a_token(3), (-3, 3));
}

#[test]
fn blood_artist_ignores_a_permanent_that_is_not_a_creature() {
    let mut g = table(&["sheoldred", "seph"]);
    perm(&mut g, P0, "Blood Artist");
    let ring = perm(&mut g, P0, "Sol Ring");
    let (me, opp) = (g.player(P0).life, g.player(P1).life);
    die(&mut g, ring, "destroy".into()).unwrap();
    assert_eq!((g.player(P0).life, g.player(P1).life), (me, opp), "Oracle: a creature dies");
}

// ------------------------------------------------------------------ Adeline, Resplendent Cathar
#[test]
fn adeline_ai_sends_a_token_at_each_opponent() {
    let mut g = table(&["cloud", "sheoldred", "thrasios"]);
    g.active = Some(P0);
    let adeline = perm(&mut g, P0, "Adeline, Resplendent Cathar");
    combat(&mut g, P0).unwrap();
    let tokens: Vec<PermId> = g
        .atk_targets
        .iter()
        .map(|x| x.0)
        .filter(|&m| g.perm(m).token && g.perm(m).owner == P0 && &*g.perm(m).name != "Adeline, Resplendent Cathar")
        .collect();
    assert!(g.atk_targets.iter().any(|x| x.0 == adeline), "precondition: the AI attacked with Adeline");
    assert_eq!(tokens.len(), 2, "one Human token for each of the two opponents");
    let mut defenders: Vec<PlayerId> =
        tokens.iter().map(|&t| g.atk_targets.iter().find(|x| x.0 == t).unwrap().1).collect();
    defenders.sort_by_key(|q| q.0);
    assert_eq!(defenders, vec![P1, P2], "a token attacks each opponent, not both the main defender");
    assert!(g.player(P1).life < 40 && g.player(P2).life < 40, "and both opponents were hit");
}

// ------------------------------------------------------------------ Armored Skyhunter
#[test]
fn armored_skyhunter_ai_probe_run_changes_nothing() {
    let mut g = table(&["cloud", "seph"]);
    g.active = Some(P0);
    let sky = perm(&mut g, P0, "Armored Skyhunter");
    // the end of the library is its top: the top six hold the Bonesplitter, the seventh card is the Aura
    let names = ["Ethereal Armor", "Plains", "Plains", "Plains", "Plains", "Bonesplitter", "Plains"];
    let top: Vec<CardId> = names.iter().map(|n| take(&mut g, P0, n)).collect();
    g.player_mut(P0).library.extend(top);
    let before = g.player(P0).library.clone();
    let imp = *g.registry.get(g.perm(sky).cd.unwrap()).unwrap();
    // the engine's probe run: it must stop at the trigger's first window with the library untouched
    g.trig_probe = true;
    let r = (imp.attack.unwrap())(&mut g, sky, P0, &[sky], P1);
    g.trig_probe = false;
    assert!(matches!(r, Err(Stop::Probe(_))), "the probe stops at the window");
    assert_eq!(g.player(P0).library, before, "nothing was popped or moved by the probe run");
    assert!(on_bf(&g, P0, "Bonesplitter").is_none());
    // the real run looks at the top six once
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![sky], d: P1 }).unwrap();
    assert!(on_bf(&g, P0, "Bonesplitter").is_some(), "the Equipment among the top six came out");
    assert!(on_bf(&g, P0, "Ethereal Armor").is_none(), "the cards below the top six were not looked at");
    let lib = &g.player(P0).library;
    assert_eq!(lib.len(), before.len() - 1);
    assert_eq!(lib.last(), before.get(before.len() - 7), "the seventh card is the new top of the library");
}

// ------------------------------------------------------------------ Zur the Enchanter
fn zur_attacks_with(lib: &[&str]) -> (Game, PermId) {
    let mut g = table(&["sheoldred", "seph"]);
    g.active = Some(P0);
    let zur = perm(&mut g, P0, "Zur the Enchanter");
    let cs: Vec<CardId> = lib.iter().map(|n| take(&mut g, P0, n)).collect();
    g.player_mut(P0).library.clear(); // only the cards under test to find
    g.player_mut(P0).library.extend(cs);
    (g, zur)
}

#[test]
fn zur_ai_puts_an_aura_for_an_opponents_creature_on_their_creature() {
    let (mut g, zur) = zur_attacks_with(&["Arrest"]);
    let hoof = perm(&mut g, P1, "Craterhoof Behemoth");
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![zur], d: P1 }).unwrap();
    let arrest = on_bf(&g, P0, "Arrest").expect("Arrest was fetched");
    assert_eq!(
        g.perm(arrest).attached,
        Some(hoof),
        "Arrest (Oracle: enchant creature, the opponent's here) is not on Zur"
    );
}

#[test]
fn zur_ai_leaves_an_opponent_aura_in_the_library_without_a_legal_host() {
    let (mut g, zur) = zur_attacks_with(&["Arrest"]);
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![zur], d: P1 }).unwrap();
    assert!(on_bf(&g, P0, "Arrest").is_none(), "nothing to enchant: not put onto the battlefield");
    assert!(g.player(P0).library.iter().any(|&c| &*g.db.get(c).name == "Arrest"), "still in the library");
}

#[test]
fn zur_ai_still_enchants_itself_with_an_aura_for_its_own_creature() {
    let (mut g, zur) = zur_attacks_with(&["Ethereal Armor"]);
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![zur], d: P1 }).unwrap();
    let armor = on_bf(&g, P0, "Ethereal Armor").expect("fetched");
    assert_eq!(g.perm(armor).attached, Some(zur));
}
