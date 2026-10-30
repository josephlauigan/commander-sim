//! The Tier 1 tatyova-simic-landfall and teysa-orzhov-aristocrats cards (impls/t12_tatyova_teysa.rs and the compiled
//! patterns under `// --- Tiers 1/2 (tatyova-teysa)`) on hand-built positions. Each expectation follows the card's
//! Oracle text.

use sim_core::dsl::{Fired, fire, has_kw};
use sim_core::engine::pool::Kind;
use sim_core::engine::cast;
use sim_core::engine::repl::trigger_copies as total_trigger_copies;
use sim_core::engine::values::{colors_of, epow, etgh, indestructible};
use sim_core::engine::zones;
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, DataKey, Game, Step, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const TATYOVA: &str = "tatyova-simic-landfall";
const TEYSA: &str = "teysa-orzhov-aristocrats";

fn tatyova() -> Game {
    table(&[TATYOVA, "seph", "sauron"])
}

fn teysa() -> Game {
    table(&[TEYSA, "seph", "sauron"])
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn tokens(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf && g.perm(m).token).collect()
}

fn to_gy(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = take(g, p, name);
    g.player_mut(p).gy.push(c);
    c
}

fn run(g: &mut Game, o: Opt, p: PlayerId) -> bool {
    match o.act.unwrap() {
        Action::Plan { f, arg } => f(g, p, arg).unwrap(),
        Action::Ability { src, f, arg } => f(g, src, p, arg).unwrap(),
        other => panic!("unexpected action {other:?}"),
    }
}

fn resolve(g: &mut Game, p: PlayerId, name: &str) {
    let c = take(g, p, name);
    cast::resolve(g, p, c, &Ctx::default(), "hand").unwrap();
}

fn options(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).and_then(|i| i.options).expect("options");
    f(g, src, p, post).unwrap()
}

fn hand_opts(g: &mut Game, c: CardId, p: PlayerId) -> Vec<Opt> {
    let f = g.registry.get(c).and_then(|i| i.hand_options).expect("hand options");
    f(g, c, p, Some(false)).unwrap()
}

/// a card put on top of p's library
fn on_top(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = take(g, p, name);
    g.player_mut(p).library.push(c);
    c
}

// ------------------------------------------------------------------ Summer Bloom, Urban Evolution
#[test]
fn summer_bloom_and_urban_evolution_add_land_drops() {
    let mut g = tatyova();
    g.player_mut(P0).lands_played = 1;
    hand(&mut g, P0, &["Forest", "Forest", "Forest", "Forest"]);
    resolve(&mut g, P0, "Summer Bloom");
    assert_eq!(g.player(P0).lands.len(), 3, "up to three additional lands, played at once");
    assert_eq!(g.player(P0).lands_played, 4);
    let n = g.player(P0).hand.len();
    resolve(&mut g, P0, "Urban Evolution");
    assert_eq!(g.player(P0).lands.len(), 4, "one more land drop");
    assert_eq!(g.player(P0).hand.len(), n + 3 - 1, "three cards drawn, a Forest played");
}

// ------------------------------------------------------------------ Beanstalk Giant // Fertile Footsteps
#[test]
fn beanstalk_giant_is_as_big_as_the_lands_and_its_adventure_ramps() {
    let mut g = tatyova();
    lands(&mut g, P0, "Forest", 3, false);
    let c = hand(&mut g, P0, &["Beanstalk Giant // Fertile Footsteps"])[0];
    let o = hand_opts(&mut g, c, P0).pop().expect("the adventure");
    assert!(run(&mut g, o, P0));
    assert_eq!(g.player(P0).lands.len(), 4, "a basic land onto the battlefield");
    assert!(!g.perm(*g.player(P0).lands.last().unwrap()).tapped, "untapped");
    assert!(g.player(P0).hand.contains(&c) && g.player(P0).adv_done.contains(&c), "the Giant is still to cast");
    assert!(hand_opts(&mut g, c, P0).is_empty(), "one adventure");
    g.player_mut(P0).hand.retain(|&x| x != c);
    g.player_mut(P0).library.push(c);
    let m = perm(&mut g, P0, "Beanstalk Giant // Fertile Footsteps");
    assert_eq!((epow(&g, m), etgh(&g, m)), (4, 4));
    lands(&mut g, P0, "Forest", 2, false);
    assert_eq!((epow(&g, m), etgh(&g, m)), (6, 6));
}

// ------------------------------------------------------------------ Bonny Pall, Clearcutter
#[test]
fn bonny_pall_makes_beau_and_puts_a_land_from_the_graveyard_when_attacking() {
    let mut g = tatyova();
    lands(&mut g, P0, "Forest", 5, false);
    let b = perm(&mut g, P0, "Bonny Pall, Clearcutter");
    let beau = on_bf(&g, P0, "Beau").expect("Beau");
    assert!(g.perm(beau).token && colors_of(&g, beau).has('U'));
    assert_eq!((epow(&g, beau), etgh(&g, beau)), (5, 5), "as big as the lands");
    let land = to_gy(&mut g, P0, "Island");
    let n = g.player(P0).hand.len();
    fire(&mut g, "attack", Fired { player: Some(P0), attackers: vec![b], defender: Some(P1), ..Default::default() })
        .unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1, "a card drawn");
    assert!(!g.player(P0).gy.contains(&land), "the graveyard land is put onto the battlefield");
    assert_eq!(g.player(P0).lands.len(), 6);
    assert_eq!(epow(&g, beau), 6);
}

// ------------------------------------------------------------------ Roiling Regrowth, Entish Restoration
#[test]
fn roiling_regrowth_and_entish_restoration_trade_a_land_for_more() {
    let mut g = tatyova();
    lands(&mut g, P0, "Forest", 4, false);
    resolve(&mut g, P0, "Roiling Regrowth");
    assert_eq!(g.player(P0).lands.len(), 5, "one sacrificed, two found");
    assert!(g.player(P0).gy.iter().any(|&c| &*g.db.get(c).name == "Forest"), "the sacrificed land");
    resolve(&mut g, P0, "Entish Restoration");
    assert_eq!(g.player(P0).lands.len(), 6, "no creature with power 4: two");
    perm(&mut g, P0, "Rampaging Baloths");
    resolve(&mut g, P0, "Entish Restoration");
    assert_eq!(g.player(P0).lands.len(), 8, "a creature with power 4 or more: three");
    let tapped = g.player(P0).lands.iter().filter(|&&l| g.perm(l).tapped).count();
    assert_eq!(tapped, 5, "the found lands enter tapped (a tapped land is sacrificed first)");
}

// ------------------------------------------------------------------ Loot, Exuberant Explorer
#[test]
fn loot_adds_a_land_drop_and_digs_for_a_creature() {
    let mut g = tatyova();
    let lo = perm(&mut g, P0, "Loot, Exuberant Explorer");
    assert_eq!(sim_core::cardcode::extra_lands(&g, P0), 1);
    assert_eq!(sim_core::cardcode::extra_lands(&g, P1), 0, "only its controller's");
    lands(&mut g, P0, "Forest", 7, false);
    let big = on_top(&mut g, P0, "Avenger of Zendikar");
    for _ in 0..5 {
        on_top(&mut g, P0, "Island");
    }
    let o = options(&mut g, lo, P0, Some(false)).pop().expect("Loot's ability");
    assert!(run(&mut g, o, P0));
    assert!(g.perm(lo).tapped);
    assert!(on_bf(&g, P0, "Avenger of Zendikar").is_some(), "mana value 7 with seven lands");
    assert!(!g.player(P0).library.contains(&big));
    let bottom: Vec<&str> = g.player(P0).library[..5].iter().map(|&c| &*g.db.get(c).name).collect();
    assert!(bottom.iter().all(|&n| n == "Island"), "the rest on the bottom");
}

// ------------------------------------------------------------------ Mossborn Hydra
#[test]
fn mossborn_hydra_enters_with_a_counter_and_doubles_on_landfall() {
    let mut g = tatyova();
    let h = perm(&mut g, P0, "Mossborn Hydra");
    assert!(g.perm(h).on_bf && g.perm(h).plus == 1);
    zones::landfall(&mut g, P0).unwrap();
    assert_eq!(g.perm(h).plus, 2);
    zones::landfall(&mut g, P0).unwrap();
    assert_eq!(g.perm(h).plus, 4);
    zones::landfall(&mut g, P1).unwrap();
    assert_eq!(g.perm(h).plus, 4, "only its controller's lands");
}

// ------------------------------------------------------------------ Wrenn and Seven
#[test]
fn wrenn_and_seven_abilities() {
    let mut g = tatyova();
    let w = perm(&mut g, P0, "Wrenn and Seven");
    let ab = g.registry.get(card(&g, "Wrenn and Seven")).and_then(|i| i.walker).unwrap();
    assert!(g.db.get(card(&g, "Wrenn and Seven")).abilities.is_empty(), "no compiled ability as well");
    for n in ["Forest", "Coiling Oracle", "Island", "Eternal Witness"] {
        on_top(&mut g, P0, n);
    }
    let (h, gy) = (g.player(P0).hand.len(), g.player(P0).gy.len());
    (ab[0].eff)(&mut g, P0, w).unwrap();
    assert_eq!(g.player(P0).hand.len(), h + 2, "the lands to hand");
    assert_eq!(g.player(P0).gy.len(), gy + 2, "the rest to the graveyard");
    // 0: the lands in hand onto the battlefield tapped
    let l = g.player(P0).lands.len();
    (ab[1].eff)(&mut g, P0, w).unwrap();
    assert_eq!(g.player(P0).lands.len(), l + 2);
    assert!(g.player(P0).lands.iter().all(|&x| g.perm(x).tapped));
    // -3: the Treefolk
    lands(&mut g, P0, "Forest", 4, false);
    (ab[2].eff)(&mut g, P0, w).unwrap();
    let t = on_bf(&g, P0, "Treefolk").expect("Treefolk");
    assert_eq!((epow(&g, t), etgh(&g, t)), (6, 6));
    assert!(has_kw(&g, t, "reach"));
    // -8: the permanent cards back, no maximum hand size
    (ab[3].eff)(&mut g, P0, w).unwrap();
    assert!(g.player(P0).gy.iter().all(|&c| !g.db.get(c).perm && !g.db.get(c).land));
    assert!(g.player(P0).hand.iter().any(|&c| &*g.db.get(c).name == "Eternal Witness"));
}

// ------------------------------------------------------------------ Black Market
#[test]
fn black_market_counts_deaths_and_makes_black_mana() {
    let mut g = teysa();
    let bm = perm(&mut g, P0, "Black Market");
    let a = token(&mut g, P0, 1);
    let b = token(&mut g, P1, 1);
    zones::die(&mut g, a, "sac").unwrap();
    zones::die(&mut g, b, "destroy").unwrap();
    assert_eq!(g.perm(bm).data.get(DataKey::Charge), Some(&Val::Int(2)), "any creature");
    let before = g.player(P0).mana.get(Kind::B);
    sim_core::engine::hooks::fire_trigger(
        &mut g,
        sim_core::hooks::Event::Main1,
        sim_core::hooks::Call::Player { p: P0 },
    )
    .unwrap();
    sim_core::engine::stack::flush_triggers(&mut g).unwrap();
    assert_eq!(g.player(P0).mana.get(Kind::B), before + 2);
}

// ------------------------------------------------------------------ Drivnod, Carnage Dominus
#[test]
fn drivnod_doubles_death_triggers_and_gets_indestructible() {
    let mut g = teysa();
    let d = perm(&mut g, P0, "Drivnod, Carnage Dominus");
    perm(&mut g, P0, "Blood Artist");
    assert_eq!(total_trigger_copies(&g, P0, "dies", None), 1);
    assert_eq!(total_trigger_copies(&g, P1, "dies", None), 0);
    let t = token(&mut g, P0, 1);
    let life = g.player(P1).life + g.player(P2).life;
    zones::die(&mut g, t, "sac").unwrap();
    assert_eq!(life - g.player(P1).life - g.player(P2).life, 2, "Blood Artist triggers twice");
    for n in ["Doomed Traveler", "Viscera Seer", "Carrion Feeder"] {
        to_gy(&mut g, P0, n);
    }
    lands(&mut g, P0, "Swamp", 2, false);
    let o = options(&mut g, d, P0, Some(false)).pop().expect("the indestructible counter");
    assert!(run(&mut g, o, P0));
    assert!(indestructible(&g, d));
    assert_eq!(g.player(P0).exile.len(), 3, "three creature cards exiled");
}

// ------------------------------------------------------------------ Hallowed Spiritkeeper, afterlife
#[test]
fn hallowed_spiritkeeper_counts_creature_cards_and_afterlife_makes_spirits() {
    let mut g = teysa();
    let h = perm(&mut g, P0, "Hallowed Spiritkeeper");
    for n in ["Doomed Traveler", "Viscera Seer"] {
        to_gy(&mut g, P0, n);
    }
    to_gy(&mut g, P0, "Swamp");
    zones::die(&mut g, h, "destroy").unwrap();
    assert_eq!(tokens(&g, P0).len(), 3, "two creature cards and itself");
    for t in tokens(&g, P0) {
        assert!(has_kw(&g, t, "flying") || g.perm(t).fly);
        assert!(colors_of(&g, t).has('W'), "white Spirits");
    }
    let mut g = teysa();
    let m = perm(&mut g, P0, "Ministrant of Obligation");
    zones::die(&mut g, m, "destroy").unwrap();
    let ts = tokens(&g, P0);
    assert_eq!(ts.len(), 2, "afterlife 2");
    assert!(ts.iter().all(|&t| colors_of(&g, t).has('W') && colors_of(&g, t).has('B') && g.perm(t).fly));
    let e = perm(&mut g, P0, "Orzhov Enforcer");
    zones::die(&mut g, e, "destroy").unwrap();
    assert_eq!(tokens(&g, P0).len(), 3, "afterlife 1");
}

// ------------------------------------------------------------------ Indebted Spirit
#[test]
fn indebted_spirit_bestowed_gives_one_one_and_afterlife() {
    let mut g = teysa();
    lands(&mut g, P0, "Plains", 3, false);
    let host = perm(&mut g, P0, "Cruel Celebrant");
    let c = hand(&mut g, P0, &["Indebted Spirit"])[0];
    let (pw, tg) = (epow(&g, host), etgh(&g, host));
    let o = hand_opts(&mut g, c, P0).pop().expect("bestow");
    assert!(run(&mut g, o, P0));
    assert_eq!((epow(&g, host), etgh(&g, host)), (pw + 1, tg + 1));
    zones::die(&mut g, host, "destroy").unwrap();
    assert!(on_bf(&g, P0, "Indebted Spirit").is_some(), "it becomes a creature");
    let ts = tokens(&g, P0);
    assert_eq!(ts.len(), 1, "the host's afterlife 1");
    assert!(colors_of(&g, ts[0]).has('B'));
    // the Spirit itself has afterlife 1
    let s = on_bf(&g, P0, "Indebted Spirit").unwrap();
    zones::die(&mut g, s, "destroy").unwrap();
    assert_eq!(tokens(&g, P0).len(), 2);
}

// ------------------------------------------------------------------ Luminous Broodmoth
#[test]
fn luminous_broodmoth_returns_creatures_without_flying() {
    let mut g = teysa();
    perm(&mut g, P0, "Luminous Broodmoth");
    let m = perm(&mut g, P0, "Viscera Seer");
    zones::die(&mut g, m, "sac").unwrap();
    let back = on_bf(&g, P0, "Viscera Seer").expect("it returns");
    assert!(g.perm(back).fly, "with a flying counter");
    zones::die(&mut g, back, "sac").unwrap();
    assert!(on_bf(&g, P0, "Viscera Seer").is_none(), "it has flying now: it stays dead");
    let t = token(&mut g, P0, 1);
    zones::die(&mut g, t, "sac").unwrap();
    let o = perm(&mut g, P1, "Viscera Seer");
    zones::die(&mut g, o, "sac").unwrap();
    assert!(on_bf(&g, P1, "Viscera Seer").is_none(), "only your creatures");
}

// ------------------------------------------------------------------ Teysa, Orzhov Scion
#[test]
fn teysa_orzhov_scion_spirits_and_exile() {
    let mut g = teysa();
    let t = perm(&mut g, P0, "Teysa, Orzhov Scion");
    assert!(g.db.get(card(&g, "Teysa, Orzhov Scion")).abilities.is_empty(), "the one-sacrifice compile is gone");
    let b = perm(&mut g, P0, "Viscera Seer"); // black
    zones::die(&mut g, b, "sac").unwrap();
    let ts = tokens(&g, P0);
    assert_eq!(ts.len(), 1, "a black creature died: a Spirit");
    assert!(colors_of(&g, ts[0]).has('W') && !colors_of(&g, ts[0]).has('B') && g.perm(ts[0]).fly);
    zones::die(&mut g, ts[0], "sac").unwrap();
    assert!(tokens(&g, P0).is_empty(), "a white Spirit dying makes none");
    // the exile: three white creatures
    let target = perm(&mut g, P1, "Avenger of Zendikar");
    g.step = Step::Main1;
    assert!(options(&mut g, t, P0, Some(false)).is_empty(), "no white creatures to sacrifice");
    for _ in 0..3 {
        sim_core::impls::t12_tatyova_teysa::indebted_afterlife(&mut g, P0).unwrap(); // white and black Spirits
    }
    let o = options(&mut g, t, P0, Some(false)).pop().expect("the exile");
    assert!(run(&mut g, o, P0));
    assert!(!g.perm(target).on_bf && g.player(P1).exile.contains(&card(&g, "Avenger of Zendikar")));
    assert_eq!(tokens(&g, P0).len(), 3, "three black Spirits died: three white Spirits");
}

// ------------------------------------------------------------------ whole games
/// both decks play whole games with the new card code (no panic, no stuck loop); the lines naming the new cards are
/// printed for a look (`--nocapture`)
#[test]
fn whole_games_with_the_new_cards() {
    let keys = [TATYOVA, TEYSA, "isshin-mardu-attack-triggers", "lathril-golgari-elves"];
    let marks = [
        "Fertile Footsteps", "Beau", "Summer Bloom", "Wrenn and Seven", "Loot", "Entish", "Roiling", "Black Market",
        "Drivnod", "Indebted", "Broodmoth", "Orzhov Scion", "Treefolk", "Mossborn", "afterlife",
    ];
    let mut seen = 0;
    for seed in 1..=8 {
        let g = play(&keys, seed, true);
        for l in g.log.as_deref().unwrap_or(&[]) {
            if marks.iter().any(|m| l.contains(m)) {
                seen += 1;
                if seen <= 60 {
                    println!("{l}");
                }
            }
        }
    }
    assert!(seen > 0, "the new cards show up in eight games");
}
