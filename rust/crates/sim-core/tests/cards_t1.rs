//! Tier 1 card code (impls/t1.rs, Python's cards/impl/t1.py) on hand-built positions: Teysa's copies and keyword
//! grants, Isshin's double attack triggers, Druid Class's levels, the Aura tutors of the Light-Paws deck, Heritage
//! Druid, Lathril's drain and the landfall payoffs. Each expectation follows the Python card code.

use sim_core::cardcode;
use sim_core::engine::{combat, mana, zones};
use sim_core::hooks::Action;
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::t1;
use sim_core::state::{DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

const TEYSA: &str = "teysa-orzhov-aristocrats";
const ISSHIN: &str = "isshin-mardu-attack-triggers";
const LATHRIL: &str = "lathril-golgari-elves";
const LIGHT_PAWS: &str = "light-paws-aura-voltron";
const TATYOVA: &str = "tatyova-simic-landfall";

fn tokens(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).token).collect()
}

fn named(g: &Game, p: PlayerId, name: &str) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).name == name).count()
}

/// run the first option a battlefield permanent's `options` hook offers p
fn use_option(g: &mut Game, p: PlayerId, src: PermId, post: Option<bool>) -> Option<bool> {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).and_then(|i| i.options).unwrap();
    let opts = f(g, src, p, post).unwrap();
    let o = opts.into_iter().next()?;
    match o.act {
        Some(Action::Ability { src, f, arg }) => Some(f(g, src, p, arg).unwrap()),
        _ => panic!("not an ability"),
    }
}

// ------------------------------------------------------------------ Teysa Karlov
#[test]
fn teysa_doubles_dies_triggers() {
    for (teysa, warriors) in [(false, 1), (true, 2)] {
        let mut g = table(&[TEYSA, "seph"]);
        if teysa {
            perm(&mut g, P0, "Teysa Karlov");
        }
        perm(&mut g, P0, "Prowess of the Fair");
        let elf = perm(&mut g, P0, "Elvish Mystic");
        zones::die(&mut g, elf, "destroy").unwrap();
        assert_eq!(tokens(&g, P0).len(), warriors, "Teysa out: {teysa}");
    }
}

#[test]
fn teysa_gives_your_creature_tokens_vigilance_and_lifelink() {
    let mut g = table(&[TEYSA, "seph"]);
    let mine = t1::elf_tokens(&mut g, P0, 1).unwrap()[0];
    let theirs = t1::elf_tokens(&mut g, P1, 1).unwrap()[0];
    let card = perm(&mut g, P0, "Elvish Mystic");
    assert!(!cardcode::granted_kw(&g, mine, "lifelink"));
    perm(&mut g, P0, "Teysa Karlov");
    assert!(cardcode::granted_kw(&g, mine, "lifelink"));
    assert!(cardcode::granted_kw(&g, mine, "vigilance"));
    assert!(!cardcode::granted_kw(&g, mine, "flying"));
    assert!(!cardcode::granted_kw(&g, theirs, "lifelink")); // an opponent's token
    assert!(!cardcode::granted_kw(&g, card, "lifelink")); // not a token
}

// ------------------------------------------------------------------ Isshin, Two Heavens as One
#[test]
fn isshin_doubles_attack_triggers() {
    for (isshin, loss) in [(false, 2), (true, 4)] {
        let mut g = table(&[ISSHIN, "seph"]);
        if isshin {
            perm(&mut g, P0, "Isshin, Two Heavens as One");
        }
        let hr = perm(&mut g, P0, "Hellrider");
        let tok = token(&mut g, P0, 2);
        combat::attack_triggers(&mut g, P0, &[hr, tok], P1).unwrap();
        assert_eq!(g.player(P1).life, 40 - loss, "Isshin out: {isshin}");
    }
}

#[test]
fn brimaz_s_cat_joins_the_attack() {
    let mut g = table(&[ISSHIN, "seph"]);
    perm(&mut g, P0, "Isshin, Two Heavens as One");
    let b = perm(&mut g, P0, "Brimaz, King of Oreskos");
    let new = combat::attack_triggers(&mut g, P0, &[b], P1).unwrap();
    assert_eq!(new.len(), 2); // a Cat per trigger, both attacking
    assert!(new.iter().all(|&m| g.perm(m).tapped && g.perm(m).token));
}

// ------------------------------------------------------------------ Druid Class
#[test]
fn druid_class_gains_life_on_landfall_and_level_two_adds_a_land_drop() {
    let mut g = table(&[TATYOVA, "seph"]);
    let dc = perm(&mut g, P0, "Druid Class");
    assert_eq!(cardcode::extra_lands(&g, P0), 0);
    zones::landfall(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).life, 41);
    g.perm_mut(dc).data.set(DataKey::Level, Val::Int(2));
    assert_eq!(cardcode::extra_lands(&g, P0), 1);
    perm(&mut g, P0, "Ancient Greenwarden"); // landfall triggers twice
    zones::landfall(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).life, 43);
}

#[test]
fn land_drops_and_land_sources() {
    let mut g = table(&[TATYOVA, "seph"]);
    perm(&mut g, P0, "Azusa, Lost but Seeking");
    perm(&mut g, P0, "Exploration");
    perm(&mut g, P1, "Exploration"); // only its controller's
    assert_eq!(cardcode::extra_lands(&g, P0), 3);
    assert!(!cardcode::lands_from_top(&g, P0) && !cardcode::lands_from_gy(&g, P0));
    perm(&mut g, P0, "Oracle of Mul Daya");
    perm(&mut g, P0, "Crucible of Worlds");
    assert!(cardcode::lands_from_top(&g, P0) && cardcode::lands_from_gy(&g, P0));
    assert!(!cardcode::lands_from_top(&g, P1));
}

// ------------------------------------------------------------------ landfall payoffs
#[test]
fn landfall_payoffs() {
    let mut g = table(&[TATYOVA, "seph"]);
    lands(&mut g, P0, "Forest", 4, false);
    perm(&mut g, P0, "Tireless Provisioner");
    perm(&mut g, P0, "Tireless Tracker");
    let t0 = g.player(P0).treasures;
    zones::landfall(&mut g, P0).unwrap();
    assert_eq!(g.player(P0).treasures, t0 + 1);
    assert_eq!(g.player(P0).clues, 1);
}

#[test]
fn avenger_of_zendikar_makes_plants_and_grows_them() {
    let mut g = table(&[TATYOVA, "seph"]);
    lands(&mut g, P0, "Forest", 5, false);
    perm(&mut g, P0, "Avenger of Zendikar");
    let plants = tokens(&g, P0);
    assert_eq!(plants.len(), 5);
    zones::landfall(&mut g, P0).unwrap();
    assert!(plants.iter().all(|&m| g.perm(m).plus == 1));
}

#[test]
fn scute_swarm_copies_itself_with_six_lands() {
    let mut g = table(&[TATYOVA, "seph"]);
    lands(&mut g, P0, "Forest", 5, false);
    perm(&mut g, P0, "Scute Swarm");
    zones::landfall(&mut g, P0).unwrap(); // five lands: an Insect
    assert_eq!(named(&g, P0, "Scute Swarm"), 1);
    assert_eq!(tokens(&g, P0).len(), 1);
    lands(&mut g, P0, "Forest", 1, false);
    zones::landfall(&mut g, P0).unwrap(); // six: a copy of itself
    assert_eq!(named(&g, P0, "Scute Swarm"), 2);
}

#[test]
fn zendikar_resurgent_doubles_land_mana() {
    let mut g = table(&[TATYOVA, "seph"]);
    lands(&mut g, P0, "Forest", 2, false);
    assert_eq!(mana::total_mana(&g, P0, false), 2);
    perm(&mut g, P0, "Zendikar Resurgent");
    assert_eq!(mana::total_mana(&g, P0, false), 4);
}

// ------------------------------------------------------------------ Lathril and the Elves
#[test]
fn heritage_druid_taps_three_other_elves_for_ggg() {
    let mut g = table(&[LATHRIL, "seph"]);
    let hd = perm(&mut g, P0, "Heritage Druid");
    let elves = t1::elf_tokens(&mut g, P0, 3).unwrap(); // summoning sick: they still pay
    assert_eq!(mana::total_mana(&g, P0, false), 3);
    assert!(mana::pay(&mut g, P0, 0, "GGG", false).unwrap());
    assert!(!g.perm(hd).tapped);
    assert!(elves.iter().all(|&m| g.perm(m).tapped));
}

#[test]
fn priest_of_titania_counts_every_elf() {
    let mut g = table(&[LATHRIL, "seph"]);
    perm(&mut g, P0, "Priest of Titania");
    t1::elf_tokens(&mut g, P0, 2).unwrap();
    t1::elf_tokens(&mut g, P1, 2).unwrap();
    assert_eq!(t1::count_type(&g, P0, "elf", true), 5);
    assert_eq!(mana::total_mana(&g, P0, false), 5);
}

#[test]
fn lathril_taps_ten_elves_to_drain_ten() {
    let mut g = table(&[LATHRIL, "seph", "veyran"]);
    let l = perm(&mut g, P0, "Lathril, Blade of the Elves");
    let elves = t1::elf_tokens(&mut g, P0, 9).unwrap();
    assert_eq!(use_option(&mut g, P0, l, Some(false)), None); // nine Elves: not enough
    let more = t1::elf_tokens(&mut g, P0, 2).unwrap();
    assert_eq!(use_option(&mut g, P0, l, Some(false)), Some(true));
    assert_eq!(g.player(P1).life, 30);
    assert_eq!(g.player(PlayerId(2)).life, 30);
    assert_eq!(g.player(P0).life, 50);
    assert!(g.perm(l).tapped);
    let tapped = elves.iter().chain(&more).filter(|&&m| g.perm(m).tapped).count();
    assert_eq!(tapped, 10);
}

#[test]
fn eyeblight_cullers_dies_into_three_warriors_and_a_drain() {
    let mut g = table(&[LATHRIL, "seph"]);
    let m = perm(&mut g, P0, "Eyeblight Cullers");
    zones::die(&mut g, m, "destroy").unwrap();
    assert_eq!(tokens(&g, P0).len(), 3);
    assert_eq!(g.player(P1).life, 38);
    assert_eq!(g.player(P0).life, 42);
}

// ------------------------------------------------------------------ Light-Paws and the Auras
fn auras_in_hand(g: &Game, p: PlayerId) -> Vec<sim_core::ids::CardId> {
    g.player(p).hand.iter().copied().filter(|&c| g.db.get(c).has_subtype("aura")).collect()
}

#[test]
fn heliod_s_pilgrim_finds_an_aura() {
    let mut g = table(&[LIGHT_PAWS, "seph"]);
    let n = g.player(P0).library.len();
    perm(&mut g, P0, "Heliod's Pilgrim");
    // Python, ported as is: the card's compiled search (any card) runs as well as t1's Aura search
    assert_eq!(auras_in_hand(&g, P0).len(), 1);
    assert_eq!(g.player(P0).hand.len(), 2);
    assert_eq!(g.player(P0).library.len(), n - 3); // the Pilgrim and the two cards found
}

#[test]
fn three_dreams_finds_three_auras_with_different_names() {
    let mut g = table(&[LIGHT_PAWS, "seph"]);
    cardcode::tutor_auras(&mut g, P0, 3).unwrap();
    let mut got = auras_in_hand(&g, P0);
    assert_eq!(got.len(), 3);
    got.sort();
    got.dedup();
    assert_eq!(got.len(), 3);
}

#[test]
#[ignore = "needs common.AURA (cardcode::aura_known / aura_own_fits, ported with common.py)"]
fn light_paws_fetches_an_aura_onto_itself_when_an_aura_is_cast() {
    let mut g = table(&[LIGHT_PAWS, "seph"]);
    let lp = perm(&mut g, P0, "Light-Paws, Emperor's Voice");
    lands(&mut g, P0, "Plains", 3, false);
    let a = hand(&mut g, P0, &["Ethereal Armor"])[0];
    sim_core::engine::cast::cast_card(&mut g, P0, a, "hand", sim_core::state::Ctx::default()).unwrap();
    let on_lp = g.auras.iter().filter(|&&x| g.perm(x).attached == Some(lp) && g.perm(x).on_bf).count();
    assert_eq!(on_lp, 2); // the cast Aura and the one Light-Paws fetched
}

#[test]
fn kor_spiritdancer_draws_on_aura_casts_and_turns_its_size_rule_on() {
    let mut g = table(&[LIGHT_PAWS, "seph"]);
    assert!(!g.selfpt);
    perm(&mut g, P0, "Kor Spiritdancer");
    assert!(g.selfpt);
    let n = g.player(P0).hand.len();
    let a = card(&g, "Ethereal Armor");
    sim_core::engine::cast::on_cast(&mut g, P0, a).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1);
    let b = card(&g, "Sol Ring");
    sim_core::engine::cast::on_cast(&mut g, P0, b).unwrap(); // not an Aura
    assert_eq!(g.player(P0).hand.len(), n + 1);
}

/// Python, ported as is: Eidolon enters as a 0/0 and dies to the state check before its own enters hook turns the
/// size rule on. Its rule counts its controller's creatures and Auras.
#[test]
fn eidolon_dies_alone_and_counts_creatures_and_auras() {
    let mut g = table(&[LIGHT_PAWS, "seph"]);
    let e = perm(&mut g, P0, "Eidolon of Countless Battles");
    assert!(!g.perm(e).on_bf);
    assert_eq!(cardcode::self_pt(&g, e), (0, 0)); // the rule is off
    perm(&mut g, P0, "Kor Spiritdancer");
    token(&mut g, P0, 1);
    assert_eq!(cardcode::self_pt(&g, e), (2, 2));
}

/// with the size rule already on (Kor Spiritdancer), Eidolon survives and gets +1/+1 per creature and Aura
#[test]
#[ignore = "needs common.attached_bonus (cardcode::attached_bonus, ported with common.py) to add cardcode::self_pt"]
fn eidolon_survives_once_the_size_rule_is_on() {
    let mut g = table(&[LIGHT_PAWS, "seph"]);
    perm(&mut g, P0, "Kor Spiritdancer");
    let e = perm(&mut g, P0, "Eidolon of Countless Battles");
    token(&mut g, P0, 1);
    assert!(g.perm(e).on_bf);
    assert_eq!(cardcode::self_pt(&g, e), (3, 3));
}
