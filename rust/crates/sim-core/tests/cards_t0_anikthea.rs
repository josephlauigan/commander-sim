//! The Anikthea precon's cards (impls/t0_anikthea.rs): each card on a hand-built position, with its Oracle text as the
//! expectation.

use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    table(&["anikthea-enduring-enchantments", "galadriel", "veyran"])
}

fn on(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

/// the beginning of p's precombat main phase for one permanent (a lore counter after the draw step)
fn main1(g: &mut Game, m: PermId, p: PlayerId) {
    let c = g.perm(m).cd.unwrap();
    let f = g.registry.get(c).unwrap().main1.unwrap();
    f(g, m, p).unwrap();
}

fn names(g: &Game, p: PlayerId) -> Vec<String> {
    g.player(p).perms.iter().map(|&m| g.perm(m).name.to_string()).collect()
}

fn count_tokens(g: &Game, p: PlayerId, ty: &str) -> usize {
    g.player(p).perms.iter().filter(|&&m| g.perm(m).token && g.perm(m).ttypes.contains(&ty)).count()
}

fn put_gy(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = take(g, p, name);
    g.player_mut(p).gy.push(c);
    c
}

// ======================================================== Sagas
#[test]
fn eldest_reborn_chapters_one_to_three() {
    let mut g = tbl();
    let small = token(&mut g, P1, 1);
    let big = perm(&mut g, P1, "Grave Titan");
    let before = g.player(P1).perms.len();
    hand(&mut g, P1, &["Sol Ring", "Mind Stone"]);
    let dead = put_gy(&mut g, P1, "Kraum, Ludevic's Opus");
    let s = perm(&mut g, P0, "The Eldest Reborn");
    assert_eq!(g.player(P1).perms.len(), before - 1, "chapter I: the opponent sacrificed one creature");
    assert!(on(&g, big) && !on(&g, small), "the weakest creature was the one sacrificed");
    let hand_before = g.player(P1).hand.len();
    main1(&mut g, s, P0);
    assert_eq!(g.player(P1).hand.len(), hand_before - 1, "chapter II: a discard");
    assert!(on(&g, s));
    main1(&mut g, s, P0);
    assert!(!on(&g, s), "sacrificed after chapter III");
    assert!(!g.player(P1).gy.contains(&dead), "chapter III took a creature card from a graveyard");
    assert!(
        names(&g, P0).iter().any(|n| n.starts_with("Kraum") || n == "Grave Titan"),
        "under P0's control: {:?}",
        names(&g, P0)
    );
}

#[test]
fn helvault_exiles_until_it_leaves_then_avacyn() {
    let mut g = tbl();
    let big = perm(&mut g, P1, "Grave Titan");
    let s = perm(&mut g, P0, "Battle at the Helvault");
    assert!(!on(&g, big), "chapter I exiled the best permanent");
    main1(&mut g, s, P0);
    main1(&mut g, s, P0);
    assert!(!on(&g, s), "sacrificed after chapter III");
    assert_eq!(count_tokens(&g, P0, "angel"), 1, "Avacyn");
    let av = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token).unwrap();
    assert_eq!((g.perm(av).pow, g.perm(av).tgh), (8, 8));
    assert!(g.perm(av).vig && g.perm(av).fly);
    assert!(g.perm(av).data.truthy(DataKey::Indestr));
    assert!(
        g.player(P1).perms.iter().any(|&m| g.perm(m).cd == Some(card(&g, "Grave Titan"))),
        "the exiled Titan came back when the Saga was sacrificed"
    );
}

#[test]
fn bretagard_copies_each_token_name_once() {
    let mut g = tbl();
    let s = perm(&mut g, P0, "Battle for Bretagard");
    assert_eq!(count_tokens(&g, P0, "human"), 1);
    main1(&mut g, s, P0);
    assert_eq!(count_tokens(&g, P0, "elf"), 1);
    main1(&mut g, s, P0);
    assert_eq!(count_tokens(&g, P0, "human"), 2, "a copy of the Human Warrior");
    assert_eq!(count_tokens(&g, P0, "elf"), 2, "a copy of the Elf Warrior");
    assert!(!on(&g, s));
}

#[test]
fn bretagard_copies_a_treasure_kind_too() {
    let mut g = tbl();
    g.player_mut(P0).treasures = 3;
    let s = perm(&mut g, P0, "Battle for Bretagard");
    main1(&mut g, s, P0);
    main1(&mut g, s, P0);
    assert_eq!(g.player(P0).treasures, 4, "one more Treasure, not one per Treasure");
}

#[test]
fn love_song_reads_ahead_to_chapter_two() {
    let mut g = tbl();
    let n = g.player(P0).hand.len();
    let s = perm(&mut g, P0, "Love Song of Night and Day");
    assert_eq!(count_tokens(&g, P0, "bird"), 1, "chapter II first: a Bird");
    assert_eq!(g.player(P0).hand.len(), n, "chapter I skipped");
    main1(&mut g, s, P0);
    assert!(!on(&g, s));
    let bird = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).token).unwrap();
    assert_eq!(g.perm(bird).plus, 1, "chapter III: a +1/+1 counter");
}

#[test]
fn binding_the_old_gods_destroys_fetches_and_gives_deathtouch() {
    let mut g = tbl();
    let big = perm(&mut g, P1, "Grave Titan");
    let mine = token(&mut g, P0, 2);
    let s = perm(&mut g, P0, "Binding the Old Gods");
    assert!(!on(&g, big), "chapter I destroyed the Titan");
    let lands_before = g.player(P0).lands.len();
    main1(&mut g, s, P0);
    assert_eq!(g.player(P0).lands.len(), lands_before + 1, "chapter II: a Forest");
    main1(&mut g, s, P0);
    assert!(g.perm(mine).eot_kw.contains(&"deathtouch"), "chapter III");
    assert!(!on(&g, s));
}

#[test]
fn mending_of_dominaria_returns_lands_and_shuffles() {
    let mut g = tbl();
    let sol = take(&mut g, P0, "Sol Ring");
    g.player_mut(P0).library = vec![sol; 20];
    let s = perm(&mut g, P0, "The Mending of Dominaria");
    put_gy(&mut g, P0, "Forest");
    put_gy(&mut g, P0, "Plains");
    put_gy(&mut g, P0, "Sol Ring");
    let lands_before = g.player(P0).lands.len();
    main1(&mut g, s, P0);
    main1(&mut g, s, P0);
    assert_eq!(g.player(P0).lands.len(), lands_before + 2, "both lands returned");
    let saga = card(&g, "The Mending of Dominaria");
    assert_eq!(
        g.player(P0).gy,
        vec![saga],
        "the graveyard was shuffled into the library; the sacrificed Saga came after"
    );
    assert!(!on(&g, s));
}

#[test]
fn binding_of_the_titans_mills_exiles_and_returns() {
    let mut g = tbl();
    let lib = g.player(P1).library.len();
    let s = perm(&mut g, P0, "The Binding of the Titans");
    assert_eq!(g.player(P1).library.len(), lib - 3, "chapter I: each player mills three");
    let c = put_gy(&mut g, P1, "Grave Titan");
    let life = g.player(P0).life;
    main1(&mut g, s, P0);
    assert!(g.player(P1).exile.contains(&c), "chapter II exiled the creature card");
    assert!(g.player(P0).life > life, "and gained life for it");
    let mine = put_gy(&mut g, P0, "Kraum, Ludevic's Opus");
    main1(&mut g, s, P0);
    assert!(g.player(P0).hand.contains(&mine), "chapter III");
}

// ======================================================== constellation and enchantment engines
fn lib_of(g: &mut Game, p: PlayerId, top: &[&str]) {
    let filler = card(g, "Sol Ring");
    let mut lib: Vec<CardId> = vec![filler; 20];
    lib.extend(top.iter().map(|n| card(g, n)));
    g.player_mut(p).library = lib;
}

#[test]
fn composer_of_spring_puts_a_land_in_tapped_and_a_creature_with_six() {
    let mut g = tbl();
    perm(&mut g, P0, "Composer of Spring");
    hand(&mut g, P0, &["Forest", "Grave Titan"]);
    let n = g.player(P0).lands.len();
    perm(&mut g, P0, "Abundance");
    assert_eq!(g.player(P0).lands.len(), n + 1, "a land from the hand");
    assert!(g.land(*g.player(P0).lands.last().unwrap()).tapped);
    assert!(g.player(P0).hand.contains(&card(&g, "Grave Titan")), "only five enchantments' worth: no creature");
    for nm in ["Mirari's Wake", "Sandwurm Convergence", "Khalni Heart Expedition", "Boon of the Spirit Realm"] {
        perm(&mut g, P0, nm);
    }
    hand(&mut g, P0, &["Forest"]);
    perm(&mut g, P0, "Cunning Rhetoric");
    assert!(
        g.player(P0).perms.iter().any(|&m| g.perm(m).cd == Some(card(&g, "Grave Titan"))),
        "with six enchantments the creature card goes in"
    );
}

#[test]
fn nessian_wanderer_takes_a_land_from_the_top_three() {
    let mut g = tbl();
    perm(&mut g, P0, "Nessian Wanderer");
    lib_of(&mut g, P0, &["Forest", "Sol Ring", "Sol Ring"]);
    let lib = g.player(P0).library.len();
    perm(&mut g, P0, "Abundance");
    assert!(g.player(P0).hand.contains(&card(&g, "Forest")));
    assert_eq!(g.player(P0).library.len(), lib - 1);
}

#[test]
fn doomwake_giant_shrinks_opposing_creatures() {
    let mut g = tbl();
    let t = token(&mut g, P1, 1); // a 1/1
    perm(&mut g, P0, "Doomwake Giant");
    assert!(!on(&g, t), "the 1/1 died to -1/-1 when the Giant itself entered");
}

#[test]
fn ondu_copies_an_enchantment_once_a_turn() {
    let mut g = tbl();
    perm(&mut g, P0, "Ondu Spiritdancer");
    perm(&mut g, P0, "Mirari's Wake");
    perm(&mut g, P0, "Sandwurm Convergence");
    let copies = |g: &Game, n: &str| {
        g.player(P0).perms.iter().filter(|&&m| g.perm(m).token && g.perm(m).cd == Some(card(g, n))).count()
    };
    assert_eq!(copies(&g, "Mirari's Wake"), 1);
    assert_eq!(copies(&g, "Sandwurm Convergence"), 0, "once each turn");
}

#[test]
fn boon_counts_blessings_and_pumps_creatures() {
    let mut g = tbl();
    let c = token(&mut g, P0, 2);
    let b = perm(&mut g, P0, "Boon of the Spirit Realm");
    assert_eq!(g.perm(b).data.int(DataKey::Counters), 1, "it counts itself");
    perm(&mut g, P0, "Mirari's Wake");
    assert_eq!(g.perm(b).data.int(DataKey::Counters), 2);
    // +2 from the counters, +1 from Mirari's Wake
    assert_eq!(sim_core::engine::values::epow(&g, c), 2 + 2 + 1);
}

#[test]
fn khalni_counts_landfall_and_fetches_two_lands() {
    use sim_core::hooks::Action;
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 2, false);
    let k = perm(&mut g, P0, "Khalni Heart Expedition");
    for _ in 0..3 {
        sim_core::engine::zones::landfall(&mut g, P0).unwrap();
    }
    assert_eq!(g.perm(k).data.int(DataKey::Counters), 3);
    let c = g.perm(k).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    let o = f(&mut g, k, P0, Some(false)).unwrap();
    assert_eq!(o.len(), 1);
    let n = g.player(P0).lands.len();
    let Some(Action::Ability { f, arg, src }) = o[0].act.clone() else { panic!() };
    assert!(f(&mut g, src, P0, arg).unwrap());
    assert_eq!(g.player(P0).lands.len(), n + 2);
    assert!(!on(&g, k), "sacrificed");
}

#[test]
fn mirari_wake_adds_a_mana_per_land() {
    let mut g = tbl();
    let ls = lands(&mut g, P0, "Forest", 3, false);
    let before = sim_core::engine::mana::total_mana(&g, P0, false);
    perm(&mut g, P0, "Mirari's Wake");
    assert_eq!(sim_core::engine::mana::total_mana(&g, P0, false), before + ls.len() as u32);
}

#[test]
fn heliod_is_a_creature_only_with_five_devotion() {
    let mut g = tbl();
    let h = perm(&mut g, P0, "Heliod, God of the Sun");
    assert!(!g.is_creature(h));
    for _ in 0..2 {
        perm(&mut g, P0, "Archon of Sun's Grace"); // {2}{W}{W} each: devotion 1 + 4
    }
    assert!(g.is_creature(h), "devotion {}", sim_core::impls::rules2::devotion(&g, P0, 'W'));
}

#[test]
fn erebos_draws_when_a_creature_dies() {
    let mut g = tbl();
    perm(&mut g, P0, "Erebos, Bleak-Hearted");
    let t = token(&mut g, P0, 1);
    let n = g.player(P0).hand.len();
    let life = g.player(P0).life;
    sim_core::engine::zones::die(&mut g, t, "destroy").unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1);
    assert_eq!(g.player(P0).life, life - 2);
}

#[test]
fn narci_draws_on_sacrificed_enchantment_and_drains_on_final_chapter() {
    let mut g = tbl();
    perm(&mut g, P0, "Narci, Fable Singer");
    let s = perm(&mut g, P0, "The Eldest Reborn"); // mana value 5
    let (n, l0, l1) = (g.player(P0).hand.len(), g.player(P0).life, g.player(P1).life);
    main1(&mut g, s, P0);
    main1(&mut g, s, P0); // chapter III, then the sacrifice
    assert_eq!(g.player(P1).life, l1 - 5);
    assert_eq!(g.player(P0).life, l0 + 5);
    assert_eq!(g.player(P0).hand.len(), n + 1, "the sacrificed Saga drew a card");
}

#[test]
fn greater_tanuki_channels_for_a_land() {
    use sim_core::hooks::Action;
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 3, false);
    let c = hand(&mut g, P0, &["Greater Tanuki"])[0];
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    let o = f(&mut g, c, P0, Some(false)).unwrap();
    assert_eq!(o.len(), 1);
    let n = g.player(P0).lands.len();
    let Some(Action::Plan { f, arg }) = o[0].act.clone() else { panic!() };
    assert!(f(&mut g, P0, arg).unwrap());
    assert_eq!(g.player(P0).lands.len(), n + 1);
    assert!(g.player(P0).gy.contains(&c));
}

#[test]
fn nyxborn_behemoth_costs_less_for_enchantments() {
    let mut g = tbl();
    let c = card(&g, "Nyxborn Behemoth");
    let f = g.registry.get(c).unwrap().self_cost.unwrap();
    assert_eq!(f(&g, P0, c), 0);
    perm(&mut g, P0, "Mirari's Wake"); // mana value 5
    perm(&mut g, P0, "Abundance"); // 4
    assert_eq!(f(&g, P0, c), -9);
}

#[test]
fn sandwurm_convergence_stops_flyers() {
    let mut g = tbl();
    perm(&mut g, P0, "Sandwurm Convergence");
    let fl = perm(&mut g, P1, "Archon of Sun's Grace"); // flying
    let gr = token(&mut g, P1, 2);
    let kept = sim_core::engine::combat::attack_limits(&mut g, P1, P0, vec![fl, gr]).unwrap();
    assert_eq!(kept, vec![gr]);
}

#[test]
fn anikthea_makes_a_zombie_copy_and_gives_menace() {
    let mut g = tbl();
    let c = put_gy(&mut g, P0, "Doomwake Giant");
    put_gy(&mut g, P0, "Mirari's Wake");
    let a = perm(&mut g, P0, "Anikthea, Hand of Erebos");
    assert!(g.player(P0).exile.contains(&c) || g.player(P0).exile.iter().any(|&x| x == card(&g, "Mirari's Wake")));
    let toks: Vec<PermId> = g.player(P0).perms.iter().copied().filter(|&m| g.perm(m).token).collect();
    assert_eq!(toks.len(), 1);
    let t = toks[0];
    assert!(g.is_creature(t) && g.perm(t).ttypes.contains(&"zombie"));
    assert_eq!((g.perm(t).pow, g.perm(t).tgh), (3, 3));
    assert!(sim_core::engine::combat::kw(&g, t, "menace"), "other enchantment creatures have menace");
    assert!(on(&g, a));
}

#[test]
fn abundance_replaces_the_draw() {
    let mut g = tbl();
    perm(&mut g, P0, "Abundance");
    // few lands in play: it chooses land
    lib_of(&mut g, P0, &["Forest", "Sol Ring", "Sol Ring"]);
    let n = g.player(P0).hand.len();
    sim_core::engine::zones::draw(&mut g, P0, 1, false).unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1);
    assert!(g.player(P0).hand.contains(&card(&g, "Forest")), "the land under two Sol Rings");
    assert_eq!(g.player(P0).library.last(), Some(&card(&g, "Sol Ring")));
    assert_eq!(g.player(P0).library.len(), 20 + 2, "the two revealed Sol Rings are now on the bottom");
}

#[test]
fn cacophony_wipes_nonenchantment_creatures_and_becomes_a_god() {
    let mut g = tbl();
    let victim = token(&mut g, P1, 3);
    let mine = perm(&mut g, P0, "Doomwake Giant"); // an enchantment creature survives
    let c = hand(&mut g, P0, &["Cacophony Unleashed"])[0];
    sim_core::engine::cast::cast_card(&mut g, P0, c, "hand", sim_core::state::Ctx::default()).unwrap();
    assert!(!on(&g, victim));
    assert!(on(&g, mine));
    let cac = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).cd == Some(c)).unwrap();
    assert!(g.is_creature(cac));
    assert_eq!((g.perm(cac).pow, g.perm(cac).tgh), (6, 6));
}

#[test]
fn bouncelands_are_in_the_card_data() {
    let g = tbl();
    for n in ["Orzhov Basilica", "Golgari Rot Farm"] {
        assert!(g.db.get(card(&g, n)).tag(sim_core::tag::Tag::Bounceland), "{n}");
    }
}

// ======================================================== Ghoulish Impetus, Demon, Rhetoric, Calix
fn run_ability(g: &mut Game, a: &sim_core::hooks::Opt, p: PlayerId) -> bool {
    match a.act.as_ref().unwrap() {
        sim_core::hooks::Action::Ability { src, f, arg } => f(g, *src, p, *arg).unwrap(),
        _ => panic!("not an ability"),
    }
}

fn options_of(g: &mut Game, m: PermId, p: PlayerId) -> Vec<sim_core::hooks::Opt> {
    let c = g.perm(m).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, m, p, Some(false)).unwrap()
}

#[test]
fn ghoulish_impetus_pumps_and_returns_when_the_creature_dies() {
    let mut g = tbl();
    let victim = perm(&mut g, P1, "Grave Titan");
    let (p0, t0) = (g.perm(victim).pow, g.perm(victim).tgh);
    let a = perm(&mut g, P0, "Ghoulish Impetus");
    assert_eq!(g.perm(a).attached, Some(victim), "it enchants the opposing creature");
    assert_eq!(sim_core::engine::values::epow(&g, victim), p0 + 1);
    assert_eq!(sim_core::engine::values::etgh(&g, victim), t0 + 1);
    sim_core::engine::zones::die(&mut g, victim, "destroy").unwrap();
    let cd = card(&g, "Ghoulish Impetus");
    assert!(g.player(P0).gy.contains(&cd), "it fell off with the creature");
    let other = perm(&mut g, P1, "Grave Titan");
    sim_core::cardcode::delayed_end_step(&mut g, P0).unwrap();
    assert!(!g.player(P0).gy.contains(&cd));
    let back = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).cd == Some(cd)).expect("back on the battlefield");
    assert_eq!(g.perm(back).attached, Some(other), "and it enchants a creature again");
}

#[test]
fn ghoulish_impetus_goads_while_attached() {
    let mut g = tbl();
    let victim = perm(&mut g, P1, "Grave Titan");
    g.perm_mut(victim).sick = false;
    let a = perm(&mut g, P0, "Ghoulish Impetus");
    assert_eq!(g.perm(a).attached, Some(victim));
    // it must attack, and not the Aura's controller, even if the AI would keep it home
    let mut groups: Vec<(PlayerId, Vec<PermId>)> = vec![];
    sim_core::impls::t0_zurgo::goad_attackers(&mut g, P1, &mut groups, P0).unwrap();
    assert_eq!(groups.len(), 1);
    assert_ne!(groups[0].0, P0, "a player other than the goader");
    assert_eq!(groups[0].1, vec![victim]);
    // the goad lasts as long as the Aura stays, across the goader's turns
    g.player_mut(P0).turns += 2;
    let mut groups: Vec<(PlayerId, Vec<PermId>)> = vec![(P0, vec![victim])];
    sim_core::impls::t0_zurgo::goad_attackers(&mut g, P1, &mut groups, P0).unwrap();
    assert!(groups.iter().all(|(q, _)| *q != P0), "redirected away from the goader");
    // without the Aura it is free again
    sim_core::engine::zones::die(&mut g, a, "destroy").unwrap();
    let mut groups: Vec<(PlayerId, Vec<PermId>)> = vec![];
    sim_core::impls::t0_zurgo::goad_attackers(&mut g, P1, &mut groups, P0).unwrap();
    assert!(groups.is_empty());
}

#[test]
fn demon_casts_an_enchantment_for_life_once_a_turn() {
    let mut g = tbl();
    g.active = Some(P0);
    let d = perm(&mut g, P0, "Demon of Fate's Design");
    hand(&mut g, P0, &["Sigil of the Empty Throne"]);
    let life = g.player(P0).life;
    let opts = options_of(&mut g, d, P0);
    assert_eq!(opts.len(), 1);
    assert!(run_ability(&mut g, &opts[0], P0));
    assert_eq!(g.player(P0).life, life - 5, "life equal to its mana value");
    assert!(names(&g, P0).contains(&"Sigil of the Empty Throne".to_string()));
    hand(&mut g, P0, &["Omen of the Sun"]);
    assert!(options_of(&mut g, d, P0).is_empty(), "once during each of your turns");
}

#[test]
fn cunning_rhetoric_takes_the_attackers_top_card_and_plays_it() {
    let mut g = tbl();
    g.active = Some(P0);
    let r = perm(&mut g, P0, "Cunning Rhetoric");
    lib_of(&mut g, P1, &["Mountain", "Lightning Bolt"]);
    let f = g.registry.get(g.perm(r).cd.unwrap()).unwrap().attack.unwrap();
    f(&mut g, r, P1, &[], P0).unwrap();
    let bolt = card(&g, "Lightning Bolt");
    assert!(g.player(P0).hand.contains(&bolt), "exiled from its library, playable by you");
    assert_eq!(g.player(P1).library.last(), Some(&card(&g, "Mountain")));
    lands(&mut g, P0, "Forest", 1, false);
    let opts = options_of(&mut g, r, P0);
    assert_eq!(opts.len(), 1, "no red mana, but any colour will do: {{R}} paid with a Forest");
    assert!(run_ability(&mut g, &opts[0], P0));
    assert!(!g.player(P0).hand.contains(&bolt));
}

#[test]
fn calix_plus_one_takes_an_enchantment_from_the_top_four() {
    let mut g = tbl();
    let c = perm(&mut g, P0, "Calix, Destiny's Hand");
    lib_of(&mut g, P0, &["Forest", "Forest", "Abundance", "Forest"]);
    let ab = g.registry.get(g.perm(c).cd.unwrap()).unwrap().walker.unwrap();
    (ab[0].eff)(&mut g, P0, c).unwrap();
    assert!(g.player(P0).hand.contains(&card(&g, "Abundance")));
    assert_eq!(g.player(P0).library.len(), 20 + 3, "the rest went to the bottom");
}

#[test]
fn calix_minus_three_exiles_until_the_linked_enchantment_leaves() {
    let mut g = tbl();
    let c = perm(&mut g, P0, "Calix, Destiny's Hand");
    let ench = perm(&mut g, P0, "Sandwurm Convergence");
    let big = perm(&mut g, P1, "Grave Titan");
    let ab = g.registry.get(g.perm(c).cd.unwrap()).unwrap().walker.unwrap();
    (ab[1].eff)(&mut g, P0, c).unwrap();
    assert!(!on(&g, big), "exiled");
    sim_core::engine::zones::leave(&mut g, ench).unwrap();
    assert!(g.player(P1).perms.iter().any(|&m| g.perm(m).cd == Some(card(&g, "Grave Titan"))), "it came back");
}

#[test]
fn calix_link_adds_no_hooks_to_shared_enchantments() {
    // Sythis's deck plays the same enchantments; Calix must not change them
    let g = tbl();
    for name in ["Enchantress's Presence", "Sandwurm Convergence", "Mirari's Wake"] {
        let hook = g.registry.get(card(&g, name)).and_then(|i| i.leaves);
        assert!(hook.is_none(), "{name} got a leaves hook");
    }
}

#[test]
fn calix_minus_seven_returns_every_enchantment_card() {
    let mut g = tbl();
    let c = perm(&mut g, P0, "Calix, Destiny's Hand");
    put_gy(&mut g, P0, "Sandwurm Convergence");
    put_gy(&mut g, P0, "Mirari's Wake");
    put_gy(&mut g, P0, "Forest");
    let ab = g.registry.get(g.perm(c).cd.unwrap()).unwrap().walker.unwrap();
    (ab[2].eff)(&mut g, P0, c).unwrap();
    let n = names(&g, P0);
    assert!(n.contains(&"Sandwurm Convergence".to_string()) && n.contains(&"Mirari's Wake".to_string()));
    assert!(g.player(P0).gy.contains(&card(&g, "Forest")), "the land stays");
}

#[test]
fn a_bounceland_played_with_no_other_land_returns_itself() {
    let mut g = tbl();
    let c = take(&mut g, P0, "Orzhov Basilica");
    sim_core::engine::turn::play_land_card(&mut g, P0, c, "plays").unwrap();
    assert!(g.player(P0).lands.is_empty(), "it must return a land, and it is the only one");
    assert!(g.player(P0).hand.contains(&c));
    // with another land out, that land goes back instead of the bounceland
    lands(&mut g, P0, "Forest", 1, false);
    let c2 = g.player_mut(P0).hand.iter().copied().find(|&x| x == c).unwrap();
    g.player_mut(P0).hand.retain(|&x| x != c2);
    sim_core::engine::turn::play_land_card(&mut g, P0, c2, "plays").unwrap();
    assert_eq!(g.player(P0).lands.len(), 1);
    assert_eq!(&*g.db.get(g.land(g.player(P0).lands[0]).cd).name, "Orzhov Basilica");
}

#[test]
fn tainted_field_needs_a_swamp_for_coloured_mana_and_sungrass_prairie_is_one_mana() {
    use sim_core::engine::mana::land_colors_now;
    let mut g = tbl();
    let tf = lands(&mut g, P0, "Tainted Field", 1, false)[0];
    assert!(land_colors_now(&g, P0, tf).count() == 0, "no Swamp: colourless only");
    lands(&mut g, P0, "Swamp", 1, false);
    let c = land_colors_now(&g, P0, tf);
    assert!(c.has('W') && c.has('B'));
    let sp = lands(&mut g, P0, "Sungrass Prairie", 1, false)[0];
    let u = sim_core::engine::mana::mana_units(&g, P0, false);
    let unit = u.iter().find(|x| x.src == sim_core::engine::mana::Source::Land(sp)).unwrap();
    assert_eq!(unit.amt, 1, "{{1}},{{T}}: add {{G}}{{W}} nets one mana");
}
