//! Hybrid mana costs (engine/hybrid.rs): a card's `pips` hold the first colour of each {A/B} symbol, its `hybrid` field
//! the pairs, and `cost_of` pays the other colour when the first can't be paid. Leyline of the Guildpact
//! ({G/W}{G/U}{B/G}{R/G}) is the card this was built for; Kitchen Finks and Vibrance check the general rule.

mod play_common;
use play_common::*;
use sim_core::engine::mana::{can_pay_spell, cost_of, pay_spell};
use sim_core::engine::{stack, zones};
use sim_core::ids::CardId;
use sim_core::state::{Ctx, StackItem, StackKind};

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    let mut g = table(&["sheoldred", "seph"]);
    g.active = Some(P0);
    g.player_mut(P0).hand.clear();
    g
}

fn sorted(s: &str) -> String {
    let mut v: Vec<char> = s.chars().collect();
    v.sort();
    v.into_iter().collect()
}

/// p's lands, one of each name given, untapped
fn sources(g: &mut Game, names: &[&str]) {
    for n in names {
        lands(g, P0, n, 1, false);
    }
}

fn untapped(g: &Game) -> usize {
    g.player(P0).lands.iter().filter(|&&l| !g.perm(l).tapped).count()
}

// ======================================================== the card data
#[test]
fn leyline_colours_are_wubrg_in_hand() {
    let g = tbl();
    let d = g.db.get(card(&g, "Leyline of the Guildpact"));
    assert_eq!(&*d.hybrid, "GWGUBGRG", "the four symbols in cost order");
    for ch in ['W', 'U', 'B', 'R', 'G'] {
        assert!(d.colors.has(ch), "Leyline is {ch}");
    }
    assert_eq!(d.cmc, 4);
}

#[test]
fn non_hybrid_cards_have_no_hybrid_field_and_keep_their_colours() {
    let g = tbl();
    let bolt = g.db.get(card(&g, "Lightning Bolt"));
    assert_eq!(&*bolt.hybrid, "");
    assert_eq!(bolt.colors, sim_core::cards::Colors::from_letters("R"));
    let finks = g.db.get(card(&g, "Kitchen Finks"));
    assert_eq!(&*finks.hybrid, "GWGW");
    assert!(finks.colors.has('G') && finks.colors.has('W') && !finks.colors.has('U'));
    // {2/W} and Phyrexian symbols are not two-colour hybrid
    assert_eq!(&*g.db.get(card(&g, "Gitaxian Probe")).hybrid, "");
}

// ======================================================== paying Leyline
#[test]
fn leyline_paid_wubr() {
    let mut g = tbl();
    sources(&mut g, &["Plains", "Island", "Swamp", "Mountain"]);
    let c = hand(&mut g, P0, &["Leyline of the Guildpact"])[0];
    let (gn, pips) = cost_of(&g, P0, c);
    assert_eq!(
        (gn, sorted(&pips)),
        (0, sorted("WUBR")),
        "{{G/W}}{{G/U}} paid with W and U; {{B/G}}{{R/G}} with B and R"
    );
    assert!(can_pay_spell(&g, P0, c, gn, &pips, false));
    assert!(pay_spell(&mut g, P0, c, gn, &pips, false).unwrap());
    assert_eq!(untapped(&g), 0);
}

#[test]
fn leyline_paid_gggg() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 4, false);
    let c = hand(&mut g, P0, &["Leyline of the Guildpact"])[0];
    let (gn, pips) = cost_of(&g, P0, c);
    assert_eq!((gn, pips.as_str()), (0, "GGGG"));
    assert!(pay_spell(&mut g, P0, c, gn, &pips, false).unwrap());
    assert_eq!(untapped(&g), 0);
}

#[test]
fn leyline_mixed() {
    let mut g = tbl();
    sources(&mut g, &["Forest", "Plains", "Swamp", "Island"]);
    let c = hand(&mut g, P0, &["Leyline of the Guildpact"])[0];
    let (gn, pips) = cost_of(&g, P0, c);
    // W for {G/W}, U for {G/U}, B for {B/G}, the Forest's G for {R/G}
    assert_eq!((gn, sorted(&pips)), (0, sorted("WUBG")));
    assert!(pay_spell(&mut g, P0, c, gn, &pips, false).unwrap());
    assert_eq!(untapped(&g), 0);
}

#[test]
fn leyline_mixed_with_two_forests_and_two_off_colour_lands() {
    let mut g = tbl();
    sources(&mut g, &["Forest", "Forest", "Plains", "Mountain"]);
    let c = hand(&mut g, P0, &["Leyline of the Guildpact"])[0];
    let (gn, pips) = cost_of(&g, P0, c);
    assert_eq!(gn, 0);
    assert!(can_pay_spell(&g, P0, c, gn, &pips, false), "{pips}");
    assert!(pay_spell(&mut g, P0, c, gn, &pips, false).unwrap());
    assert_eq!(untapped(&g), 0);
}

#[test]
fn leyline_unpayable_ggg_plus_c() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 3, false);
    lands(&mut g, P0, "Wastes", 1, false);
    let c = hand(&mut g, P0, &["Leyline of the Guildpact"])[0];
    let (gn, pips) = cost_of(&g, P0, c);
    assert_eq!((gn, pips.as_str()), (0, "GGBR"), "no way to pay it: the printed first colours stay");
    assert!(!can_pay_spell(&g, P0, c, gn, &pips, false));
    assert!(!pay_spell(&mut g, P0, c, gn, &pips, false).unwrap());
    assert_eq!(untapped(&g), 4, "nothing was spent");
}

#[test]
fn default_colours_are_kept_when_they_can_be_paid() {
    let mut g = tbl();
    sources(&mut g, &["Forest", "Forest", "Swamp", "Mountain"]);
    let c = hand(&mut g, P0, &["Leyline of the Guildpact"])[0];
    assert_eq!(
        cost_of(&g, P0, c),
        (0, "GGBR".to_string()),
        "{{G/W}}{{G/U}}{{B/G}}{{R/G}} at the printed first colours"
    );
}

// ======================================================== other hybrid cards
#[test]
fn kitchen_finks_paid_ww() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 3, false);
    let c = hand(&mut g, P0, &["Kitchen Finks"])[0];
    let (gn, pips) = cost_of(&g, P0, c);
    assert_eq!((gn, pips.as_str()), (1, "WW"));
    assert!(pay_spell(&mut g, P0, c, gn, &pips, false).unwrap());
    assert_eq!(untapped(&g), 0);
}

#[test]
fn kitchen_finks_keeps_gg_with_forests_and_takes_one_of_each_with_one_forest() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 3, false);
    let c = hand(&mut g, P0, &["Kitchen Finks"])[0];
    assert_eq!(cost_of(&g, P0, c), (1, "GG".to_string()));
    let mut g = tbl();
    sources(&mut g, &["Forest", "Plains", "Plains"]);
    let c = hand(&mut g, P0, &["Kitchen Finks"])[0];
    let (gn, pips) = cost_of(&g, P0, c);
    assert_eq!((gn, sorted(&pips)), (1, sorted("GW")));
    assert!(pay_spell(&mut g, P0, c, gn, &pips, false).unwrap());
}

#[test]
fn a_hybrid_card_with_a_plain_pip_keeps_it() {
    // Essenceknit Scholar {B}{B/G}{G}: with Swamp, Forest, Forest the {B/G} is paid by G and the plain pips stay
    let mut g = tbl();
    sources(&mut g, &["Swamp", "Forest", "Forest"]);
    let c = hand(&mut g, P0, &["Essenceknit Scholar"])[0];
    let (gn, pips) = cost_of(&g, P0, c);
    assert_eq!((gn, sorted(&pips)), (0, sorted("BGG")));
    assert!(pay_spell(&mut g, P0, c, gn, &pips, false).unwrap());
}

#[test]
fn non_hybrid_costs_are_unchanged() {
    let mut g = tbl();
    let bolt = hand(&mut g, P0, &["Lightning Bolt"])[0];
    assert_eq!(cost_of(&g, P0, bolt), (0, "R".to_string()), "unpayable or not, a plain cost is as printed");
    lands(&mut g, P0, "Mountain", 1, false);
    assert_eq!(cost_of(&g, P0, bolt), (0, "R".to_string()));
}

// ======================================================== colours of hybrid cards
#[test]
fn hybrid_cards_have_both_colours() {
    let g = tbl();
    let has = |name: &str, cols: &str| {
        let d = g.db.get(card(&g, name));
        let want = sim_core::cards::Colors::from_letters(cols);
        assert_eq!(d.colors, want, "{name}");
    };
    has("Deathrite Shaman", "BG");
    has("Fiend Artisan", "BG");
    has("Yorion, Sky Nomad", "WU");
    has("Vibrance", "RG");
    has("Privileged Position", "GW");
    has("Blade Historian", "RW");
    has("Steel of the Godhead", "WU");
}

#[test]
fn fow_can_pitch_leyline() {
    let mut g = table(&["sheoldred", "seph"]);
    g.active = Some(P1);
    g.player_mut(P0).hand.clear();
    let ley = hand(&mut g, P0, &["Leyline of the Guildpact"])[0];
    assert!(stack::blue_card(&g, ley), "Leyline is blue (and every other colour) in hand");
    let fow = hand(&mut g, P0, &["Force of Will"])[0];
    let bolt = hand(&mut g, P1, &["Lightning Bolt"])[0];
    let i = g.player(P1).hand.iter().position(|&x| x == bolt).unwrap();
    g.player_mut(P1).hand.remove(i);
    let id = g.stack_pushes + 1;
    g.stack_pushes = id;
    g.stack.push(StackItem {
        id,
        controller: P1,
        card: Some(bolt),
        ctx: Ctx::default(),
        zone: "hand",
        imp: 6.0,
        aff: vec![],
        generic: true,
        kind: StackKind::Spell,
        name: "Lightning Bolt".to_string(),
        passed: vec![],
        countered: false,
        countered_by: None,
    });
    assert_eq!(stack::pick_counter(&g, P0, bolt), Some(fow), "the pitch card is Leyline");
    assert!(stack::pay_counter(&mut g, P0, fow, Some(bolt)).unwrap());
    assert!(g.player(P0).exile.contains(&ley), "Leyline was exiled to pay for Force of Will");
}

// ======================================================== a person pays from the pool
#[test]
fn person_pool_pays_hybrid() {
    let mut g = tbl();
    seat(&mut g, P0, vec![]);
    let c = hand(&mut g, P0, &["Leyline of the Guildpact"])[0];
    pool(&mut g, P0, &[('W', 1), ('U', 1), ('B', 1), ('R', 1)]);
    let (gn, pips) = cost_of(&g, P0, c);
    assert_eq!((gn, sorted(&pips)), (0, sorted("WUBR")), "{{W}}{{U}}{{B}}{{R}} in the pool pays it");
    assert!(can_pay_spell(&g, P0, c, gn, &pips, false));
    assert!(pay_spell(&mut g, P0, c, gn, &pips, false).unwrap());
    assert_eq!(pool_total(&g, P0), 0, "all four came out of the pool");
}

#[test]
fn person_pool_with_gggg_keeps_the_printed_colours_where_it_can() {
    let mut g = tbl();
    seat(&mut g, P0, vec![]);
    let c = hand(&mut g, P0, &["Leyline of the Guildpact"])[0];
    pool(&mut g, P0, &[('G', 4)]);
    let (gn, pips) = cost_of(&g, P0, c);
    assert_eq!((gn, pips.as_str()), (0, "GGGG"));
    assert!(pay_spell(&mut g, P0, c, gn, &pips, false).unwrap());
}

// ======================================================== Vibrance reads the colours it was paid with
fn vibrance_cast_with(g: &mut Game, land: &str) -> CardId {
    lands(g, P0, land, 5, false);
    let c = hand(g, P0, &["Vibrance"])[0];
    let (gn, pips) = cost_of(g, P0, c);
    assert!(can_pay_spell(g, P0, c, gn, &pips, false));
    pay_spell(g, P0, c, gn, &pips, false).unwrap();
    let i = g.player(P0).hand.iter().position(|&x| x == c).unwrap();
    g.player_mut(P0).hand.remove(i);
    zones::enter(g, P0, c, zones::Enter { was_cast: true, sick: false, ..Default::default() }).unwrap();
    c
}

#[test]
fn vibrance_paid_with_red_deals_3() {
    let mut g = tbl();
    let life = g.player(P1).life;
    let c = vibrance_cast_with(&mut g, "Mountain");
    assert_eq!(sim_core::engine::hybrid::priced(&g, c).as_deref(), Some("RR"));
    assert_eq!(g.player(P1).life, life - 3, "{{R}}{{R}} was spent: 3 damage");
}

#[test]
fn vibrance_paid_with_green_finds_a_land_and_gains_2() {
    let mut g = tbl();
    let (life, opp) = (g.player(P0).life, g.player(P1).life);
    let c = vibrance_cast_with(&mut g, "Forest");
    assert_eq!(sim_core::engine::hybrid::priced(&g, c).as_deref(), Some("GG"), "no red: the hybrid is paid with green");
    assert_eq!(g.player(P0).life, life + 2, "{{G}}{{G}} was spent: 2 life");
    assert_eq!(g.player(P1).life, opp, "and no damage");
}

// ======================================================== the paid-colour record belongs to its game
#[test]
fn the_paid_colour_record_is_per_game() {
    let mut g = tbl();
    let c = vibrance_cast_with(&mut g, "Mountain");
    assert_eq!(sim_core::engine::hybrid::priced(&g, c).as_deref(), Some("RR"));
    let mut other = tbl();
    lands(&mut other, P0, "Forest", 5, false);
    let _ = cost_of(&other, P0, c); // another game prices the same card id at GG
    assert_eq!(sim_core::engine::hybrid::priced(&other, c).as_deref(), Some("GG"));
    assert_eq!(sim_core::engine::hybrid::priced(&g, c).as_deref(), Some("RR"), "the first game's record is its own");
    assert_eq!(sim_core::engine::hybrid::priced(&tbl(), c), None, "a fresh game has no record");
}

// ======================================================== devotion counts both halves of a hybrid symbol (CR 700.5)
#[test]
fn blade_historian_counts_once_for_red_and_white_together_and_in_full_for_each() {
    use sim_core::impls::rules2::{devotion, devotion_to};
    let mut g = tbl();
    perm(&mut g, P0, "Blade Historian"); // {2}{R/W}{R/W}{R/W}{R/W}
    assert_eq!(devotion(&g, P0, 'R'), 4);
    assert_eq!(devotion(&g, P0, 'W'), 4, "the second half counts too");
    assert_eq!(devotion_to(&g, P0, "RW"), 4, "devotion to red and white together counts each symbol once (Iroas)");
    assert_eq!(devotion_to(&g, P0, "WR"), 4);
    assert_eq!(devotion_to(&g, P0, "G"), 0);
    perm(&mut g, P0, "Goblin Guide"); // {R}
    assert_eq!(devotion_to(&g, P0, "RW"), 5);
    assert_eq!(devotion(&g, P0, 'W'), 4);
}

#[test]
fn devotion_to_a_card_counts_hybrid_pairs_by_either_half() {
    let g = tbl();
    let finks = g.db.get(card(&g, "Kitchen Finks"));
    assert_eq!(
        (finks.devotion_to("G"), finks.devotion_to("W"), finks.devotion_to("GW"), finks.devotion_to("U")),
        (2, 2, 2, 0)
    );
    let leyline = g.db.get(card(&g, "Leyline of the Guildpact"));
    assert_eq!(
        (leyline.devotion_to("U"), leyline.devotion_to("W"), leyline.devotion_to("B"), leyline.devotion_to("R")),
        (1, 1, 1, 1)
    );
    assert_eq!(leyline.devotion_to("G"), 4, "G is in every one of the four symbols");
    assert_eq!(leyline.devotion_to("WUBRG"), 4, "four symbols, each counted once");
    let bolt = g.db.get(card(&g, "Lightning Bolt"));
    assert_eq!((bolt.devotion_to("R"), bolt.devotion_to("RW")), (1, 1));
}
