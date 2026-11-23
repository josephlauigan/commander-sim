// written by an independent reviewer
//! Adversarial: the ward predicate (b) of the Solitude evoke and enters trigger.

use sim_core::ai::act::perform;
use sim_core::engine::values::pval;
use sim_core::engine::ward::ward_affordable;
use sim_core::hooks::Opt;
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const SOL: &str = "Solitude";
const MIIRYM: &str = "Miirym, Sentinel Wyrm";
const SHELOB: &str = "Shelob, Child of Ungoliant";
const BIG: &str = "Grave Titan";

fn tbl(key: &str) -> (Game, CardId) {
    let mut g = table(&[key, "seph"]);
    g.active = Some(P0);
    g.player_mut(P1).hand.clear();
    let s = hand(&mut g, P0, &[SOL])[0];
    hand(&mut g, P0, &["Swords to Plowshares"]);
    (g, s)
}

fn evoke_opts(g: &mut Game, c: CardId) -> Vec<Opt> {
    let f = g.registry.get(c).and_then(|i| i.hand_options).expect("hand options");
    f(g, c, P0, None).unwrap()
}

fn ai_evokes(g: &mut Game, c: CardId) -> bool {
    let opts = evoke_opts(g, c);
    let Some(o) = opts.iter().find(|o| o.label.contains("evoke Solitude")) else { return false };
    perform(g, P0, o.act.as_ref().unwrap()).unwrap()
}

fn tapped_lands(g: &Game, p: PlayerId) -> usize {
    g.player(p).lands.iter().filter(|&&l| g.perm(l).tapped).count()
}

/// the evoke tax (Sphere of Resistance, {1}) and ward {2} compete for two lands: the evoke is offered (ward alone
/// is affordable) but after the tax is paid the ward cannot be: what happens?
#[test]
fn tax_and_ward_compete_for_the_last_mana() {
    let (mut g, s) = tbl("yoshimaru");
    let m = perm(&mut g, P1, MIIRYM);
    perm(&mut g, P1, "Sphere of Resistance");
    lands(&mut g, P0, "Plains", 2, false);
    let offered = !evoke_opts(&mut g, s).is_empty();
    println!("offered with tax 1 + ward 2 on 2 lands: {offered}");
    if offered {
        assert!(ai_evokes(&mut g, s));
        println!("miirym on bf after: {}, tapped lands {}", g.perm(m).on_bf, tapped_lands(&g, P0));
        assert!(!g.perm(m).on_bf, "the evoke was offered into a ward it could not pay after the tax: Miirym survived");
    }
}

/// the same with 3 lands: it is fine
#[test]
fn tax_and_ward_with_three_lands() {
    let (mut g, s) = tbl("yoshimaru");
    let m = perm(&mut g, P1, MIIRYM);
    perm(&mut g, P1, "Sphere of Resistance");
    lands(&mut g, P0, "Plains", 3, false);
    assert!(ai_evokes(&mut g, s));
    assert!(!g.perm(m).on_bf);
}

/// a warded unaffordable creature outranks a pval < 6 creature: no evoke is offered, nothing spent
#[test]
fn unaffordable_ward_creature_plus_a_small_one_no_evoke() {
    let (mut g, s) = tbl("yoshimaru");
    perm(&mut g, P1, SHELOB);
    let t = token(&mut g, P1, 2);
    println!("pval small {}", pval(&g, t));
    assert!(pval(&g, t) < 6.0);
    assert!(evoke_opts(&mut g, s).is_empty());
    assert!(g.player(P0).hand.contains(&s));
}

/// the enters trigger: warded Shelob (unaffordable), a lesser creature worth >= 2: that one is exiled
#[test]
fn enters_trigger_falls_through_to_the_next_best_creature() {
    let mut used = None;
    for name in ["Giver of Runes", "Archangel of Thune", "Restoration Angel", "Adeline, Resplendent Cathar"] {
        let mut g = table(&["yoshimaru", "seph"]);
        let sh = perm(&mut g, P1, SHELOB);
        let t = perm(&mut g, P1, name);
        let (v, w) = (pval(&g, t), pval(&g, sh));
        println!("{name}: pval {v} (Shelob {w})");
        if v >= 2.0 && v < w {
            perm(&mut g, P0, SOL);
            assert!(g.perm(sh).on_bf, "Shelob left alone (ward {{2}}, tapped out)");
            assert!(!g.perm(t).on_bf, "{name} is exiled (pval {v})");
            used = Some(name);
            break;
        }
    }
    assert!(used.is_some(), "no candidate creature between 2.0 and Shelob's pval");
}

/// a phased-out or hexproof creature of top value is skipped by best_opp_creature already
#[test]
fn phased_out_warded_creature_does_not_hide_the_valid_target() {
    let (mut g, s) = tbl("yoshimaru");
    let sh = perm(&mut g, P1, SHELOB);
    g.perm_mut(sh).phased = true;
    let b = perm(&mut g, P1, BIG);
    assert!(ai_evokes(&mut g, s));
    assert!(!g.perm(b).on_bf);
}

/// Sauron's ward: no legend to sacrifice: no target
#[test]
fn sauron_unaffordable_without_a_legend_affordable_with() {
    let (mut g, _s) = tbl("yoshimaru");
    let sau = perm(&mut g, P1, "Sauron, the Dark Lord");
    println!("no legend: {}", ward_affordable(&g, P0, sau, 0, ""));
    assert!(!ward_affordable(&g, P0, sau, 0, ""), "no legend");
    let _ = perm(&mut g, P0, "Yoshimaru, Ever Faithful");
    assert!(ward_affordable(&g, P0, sau, 0, ""), "a legendary creature to sacrifice");
    assert!(ward_affordable(&g, P1, sau, 0, ""), "own permanent");
}

/// Sauron's own creature does not have its ward cost both from card ward and Sauron tag: one check each
#[test]
fn valgavoth_needs_three_nonland_permanents() {
    let (mut g, _s) = tbl("yoshimaru");
    let v = perm(&mut g, P1, "Valgavoth, Terror Eater");
    assert!(!ward_affordable(&g, P0, v, 0, ""), "nothing to sacrifice");
    for _ in 0..3 {
        token(&mut g, P0, 1);
    }
    assert!(ward_affordable(&g, P0, v, 0, ""));
}

/// ward {2} with `extra_generic`: extra counts
#[test]
fn extra_generic_adds_to_the_ward() {
    let (mut g, _s) = tbl("yoshimaru");
    let m = perm(&mut g, P1, MIIRYM);
    lands(&mut g, P0, "Plains", 3, false);
    assert!(ward_affordable(&g, P0, m, 1, ""));
    assert!(!ward_affordable(&g, P0, m, 2, ""));
}

/// Controller semantics: a creature P1 stole (it sits in P0's perms, owner P0) is not an opposing target at all
#[test]
fn perm_owner_is_the_controller() {
    let (mut g, _s) = tbl("yoshimaru");
    let m = perm(&mut g, P0, MIIRYM);
    assert_eq!(g.perm(m).owner, P0);
    assert!(ward_affordable(&g, P0, m, 0, ""));
    let _: PermId = m;
}

/// Sauron in the evoke: the AI (tapped out, no legend) must not pick Sauron over a plain creature
#[test]
fn evoke_skips_sauron_without_a_legend() {
    let (mut g, s) = tbl("yoshimaru");
    let sau = perm(&mut g, P1, "Sauron, the Dark Lord");
    let b = perm(&mut g, P1, BIG);
    println!("pval sauron {} big {}", pval(&g, sau), pval(&g, b));
    assert!(ai_evokes(&mut g, s));
    assert!(g.perm(sau).on_bf, "Sauron untouched (ward would counter)");
    assert!(!g.perm(b).on_bf);
}
