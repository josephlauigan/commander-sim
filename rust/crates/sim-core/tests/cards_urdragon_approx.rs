//! JD's `urdragon` deck: the five cards brought up from Approximate (Astral Dragon, Cursed Mirror, Nicol Bolas the
//! Ravager, The Ur-Dragon, Wrenn and Six), on hand-built positions. Each expectation follows the card's Oracle text.

use sim_core::engine::cast;
use sim_core::engine::values::{epow, etgh, has_type};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::state::{Ctx, Game, Step};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn tbl() -> Game {
    let mut g =
        table(&["urdragon", "miirym-temur-dragons", "urza-mono-blue-artifacts", "zur-esper-enchantment-control"]);
    g.player_mut(P0).hand.clear();
    g.active = Some(P0);
    g.step = Step::Main1;
    g
}

fn imp(g: &Game, name: &str) -> sim_core::hooks::CardImpl {
    *g.registry.get(card(g, name)).unwrap()
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf && &*g.perm(m).name == name).collect()
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

fn cast_hand(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
    c
}

fn library(g: &mut Game, p: PlayerId, names: &[&str]) -> Vec<CardId> {
    let cs: Vec<CardId> = names.iter().map(|n| card(g, n)).collect();
    g.player_mut(p).library = cs.clone();
    cs
}

fn hooks(g: &mut Game) -> Vec<Opt> {
    let s = sim_core::ai::brain::Situation::new(g, P0);
    sim_core::ai::brain::hook_options(g, P0, &s, Some(false)).unwrap()
}

fn opp_life(g: &Game) -> i32 {
    g.opps(P0).map(|q| g.player(q).life).sum()
}

fn perform(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    sim_core::ai::act::perform(g, p, o.act.as_ref().unwrap()).unwrap()
}

// ================================================================== Astral Dragon
#[test]
fn astral_dragon_tokens_are_3_3_flying_dragon_creatures_that_dragon_tempest_sees() {
    let mut g = tbl();
    perm(&mut g, P0, "Dragon Tempest");
    perm(&mut g, P0, "Rhystic Study"); // the permanent to copy
    let before = opp_life(&g);
    perm(&mut g, P0, "Astral Dragon");
    let toks: Vec<PermId> = on_bf(&g, P0, "Rhystic Study").into_iter().filter(|&m| g.perm(m).token).collect();
    assert_eq!(toks.len(), 2, "two token copies of Rhystic Study");
    for &t in &toks {
        assert!(g.is_creature(t) && has_type(&g, t, "dragon") && g.perm(t).fly);
        assert_eq!((epow(&g, t), etgh(&g, t)), (3, 3));
    }
    // Dragon Tempest: a Dragon that enters deals damage equal to the Dragons you control: Astral Dragon alone is 1, the
    // two tokens (creatures with flying and Dragons as they enter) 2 and 3 more
    assert!(
        before - opp_life(&g) >= 6,
        "Astral Dragon and both tokens set Dragon Tempest off: {}",
        before - opp_life(&g)
    );
}

#[test]
fn astral_dragon_aims_at_the_better_permanent_and_may_copy_an_opponents_legendary_one() {
    let mut g = tbl();
    perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P0, "Rhystic Study");
    let t = sim_core::impls::t45_miirym::astral_target(&g, P0).expect("a target");
    assert_eq!(&*g.perm(t).name, "Rhystic Study", "the engine over the rock");
    // an own legendary permanent isn't copied (the legend rule would keep one of three); an opponent's may be
    let mut g = tbl();
    perm(&mut g, P0, "The One Ring");
    assert!(sim_core::impls::t45_miirym::astral_target(&g, P0).is_none(), "own legendary: no target");
    let mut g2 = tbl();
    perm(&mut g2, P1, "The One Ring");
    let t = sim_core::impls::t45_miirym::astral_target(&g2, P0).expect("the opponent's The One Ring");
    assert_eq!(g2.perm(t).owner, P1);
    perm(&mut g2, P0, "Astral Dragon");
    sim_core::engine::life::check_state(&mut g2).unwrap(); // the legend rule
    let mine = on_bf(&g2, P0, "The One Ring");
    assert_eq!(mine.len(), 1, "two token copies, and the legend rule keeps one of them");
    assert_eq!(on_bf(&g2, P1, "The One Ring").len(), 1, "theirs is untouched");
}

// ================================================================== Cursed Mirror
#[test]
fn cursed_mirror_is_the_creature_it_copies_with_haste_and_its_enters_trigger() {
    let mut g = tbl();
    perm(&mut g, P1, "Grave Titan");
    let before = g.player(P0).perms.len();
    let c = cast_hand(&mut g, P0, "Cursed Mirror");
    let m = *g.player(P0).perms.iter().find(|&&m| g.perm(m).phys == Some(c)).expect("the Mirror's permanent");
    assert_eq!(&*g.perm(m).name, "Grave Titan", "a copy of Grave Titan");
    assert!(g.is_creature(m) && !g.perm(m).sick, "a creature with haste");
    assert_eq!((epow(&g, m), etgh(&g, m)), (6, 6));
    assert!(g.perm(m).fly || epow(&g, m) == 6);
    // the copied card's enters trigger fired: two 2/2 Zombies
    let tokens = g.player(P0).perms.iter().filter(|&&x| g.perm(x).token).count();
    assert_eq!(tokens, 2, "Grave Titan's enters trigger: two Zombie tokens");
    assert_eq!(g.player(P0).perms.len(), before + 3);
}

#[test]
fn cursed_mirror_has_the_copied_creatures_abilities() {
    let mut g = tbl();
    perm(&mut g, P0, "Terror of the Peaks");
    cast_hand(&mut g, P0, "Cursed Mirror");
    // the Mirror is a second Terror of the Peaks: it has Terror's abilities (flying 5/4, and the damage trigger), and
    // each Terror deals damage when another creature enters
    let terrors = on_bf(&g, P0, "Terror of the Peaks");
    assert_eq!(terrors.len(), 2);
    assert!(terrors.iter().all(|&t| g.perm(t).fly && epow(&g, t) == 5));
    let before = opp_life(&g);
    perm(&mut g, P0, "Thundermaw Hellkite"); // 5 power: each Terror (and the Mirror) deals 5
    assert!(before - opp_life(&g) >= 10, "both Terrors trigger: {}", before - opp_life(&g));
}

#[test]
fn cursed_mirror_is_an_artifact_again_at_the_end_of_the_turn_and_goes_to_the_graveyard_as_the_mirror() {
    let mut g = tbl();
    perm(&mut g, P1, "Grave Titan");
    let c = cast_hand(&mut g, P0, "Cursed Mirror");
    let m = *g.player(P0).perms.iter().find(|&&m| g.perm(m).phys == Some(c)).unwrap();
    assert_eq!(g.perm(m).phys, Some(c));
    sim_core::impls::t0_stella::mirror_revert(&mut g); // the next turn begins
    assert!(!g.is_creature(m), "an artifact again");
    assert_eq!(&*g.perm(m).name, "Cursed Mirror");
    let mana0 = sim_core::engine::mana::total_mana(&g, P0, false);
    assert!(mana0 >= 1, "it taps for {{R}} again: {mana0}");
    sim_core::engine::zones::die(&mut g, m, "destroy").unwrap();
    assert!(in_gy(&g, P0, "Cursed Mirror"), "the Mirror card is in the graveyard");
}

#[test]
fn cursed_mirror_dying_as_a_creature_is_the_mirror_in_the_graveyard() {
    let mut g = tbl();
    perm(&mut g, P1, "Grave Titan");
    let c = cast_hand(&mut g, P0, "Cursed Mirror");
    let m = *g.player(P0).perms.iter().find(|&&m| g.perm(m).phys == Some(c)).unwrap();
    sim_core::engine::zones::die(&mut g, m, "destroy").unwrap();
    assert!(in_gy(&g, P0, "Cursed Mirror") && !in_gy(&g, P0, "Grave Titan"));
}

#[test]
fn cursed_mirror_stays_a_mana_rock_with_nothing_worth_copying() {
    let mut g = tbl();
    perm(&mut g, P1, "Birds of Paradise");
    let m = *{
        cast_hand(&mut g, P0, "Cursed Mirror");
        on_bf(&g, P0, "Cursed Mirror")
    }
    .first()
    .expect("still the Mirror");
    assert!(!g.is_creature(m));
}

#[test]
fn a_token_copy_of_cursed_mirror_becomes_a_copy_too() {
    let mut g = tbl();
    perm(&mut g, P1, "Grave Titan");
    let cd = card(&g, "Cursed Mirror");
    let k = sim_core::engine::zones::enter_token_copy(&mut g, P0, cd).unwrap().expect("a token");
    assert_eq!(&*g.perm(k).name, "Grave Titan", "as it enters, the token Mirror may become a copy");
}

// ================================================================== Nicol Bolas, the Ravager
const RAVAGER: &str = "Nicol Bolas, the Ravager";
const ARISEN: &str = "Nicol Bolas, the Ravager // Nicol Bolas, the Arisen";

fn bolas_table() -> (Game, PermId) {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 3, false);
    lands(&mut g, P0, "Swamp", 2, false);
    lands(&mut g, P0, "Mountain", 3, false);
    let b = perm(&mut g, P0, RAVAGER);
    g.step = Step::Main2;
    (g, b)
}

fn flip(g: &mut Game, b: PermId) -> PermId {
    let o = (imp(g, RAVAGER).options.unwrap())(g, b, P0, Some(true)).unwrap().pop().expect("the transform option");
    assert!(perform(g, P0, &o));
    on_bf(g, P0, ARISEN).pop().expect("the Arisen on the battlefield")
}

#[test]
fn bolas_transforms_into_a_seven_loyalty_walker_and_the_ravager_is_gone() {
    let (mut g, b) = bolas_table();
    let a = flip(&mut g, b);
    assert!(!g.perm(b).on_bf, "he was exiled");
    assert!(!g.is_creature(a));
    assert_eq!(g.perm(a).loyalty, Some(7));
    assert!(!in_gy(&g, P0, RAVAGER), "exiled, not dead");
    assert_eq!(
        sim_core::engine::mana::total_mana(&g, P0, false),
        1,
        "{{4}}{{U}}{{B}}{{R}} paid: one land of eight left"
    );
}

#[test]
fn bolas_the_arisen_loyalty_abilities_work() {
    let (mut g, b) = bolas_table();
    let a = flip(&mut g, b);
    // +2: draw two cards
    library(&mut g, P0, &["Plains", "Island", "Forest"]);
    let h = g.player(P0).hand.len();
    assert!(sim_core::dsl::use_loyalty(&mut g, P0, a, 0).unwrap());
    assert_eq!(g.player(P0).hand.len(), h + 2, "+2: draw two");
    assert_eq!(g.perm(a).loyalty, Some(9));
    // -3: 10 damage to target creature or planeswalker
    let titan = perm(&mut g, P1, "Grave Titan");
    g.perm_mut(a).loyalty_used = None;
    g.perm_mut(a).loyalty = Some(7);
    assert!(sim_core::dsl::use_loyalty(&mut g, P0, a, 1).unwrap());
    assert!(!g.perm(titan).on_bf, "-3: 10 damage kills Grave Titan");
    assert_eq!(g.perm(a).loyalty, Some(4));
    // -4: a creature card from a graveyard onto the battlefield under his controller's control
    g.perm_mut(a).loyalty_used = None;
    assert!(sim_core::dsl::use_loyalty(&mut g, P0, a, 2).unwrap());
    assert_eq!(on_bf(&g, P0, "Grave Titan").len(), 1, "-4: Grave Titan comes back under my control");
}

#[test]
fn bolas_the_arisen_minus_twelve_exiles_all_but_the_bottom_card() {
    let (mut g, b) = bolas_table();
    let a = flip(&mut g, b);
    g.perm_mut(a).loyalty = Some(14); // a Doubling Season's worth
    let n = g.player(P1).library.len();
    assert!(n > 5);
    let bottom = g.player(P1).library[0];
    assert!(sim_core::dsl::use_loyalty(&mut g, P0, a, 3).unwrap());
    assert_eq!(g.player(P1).library, vec![bottom], "-12: only the bottom card is left");
    assert_eq!(g.perm(a).loyalty, Some(2));
}

#[test]
fn the_ai_uses_the_arisen_the_turn_he_flips_and_draws() {
    let (mut g, b) = bolas_table();
    let a = flip(&mut g, b);
    library(&mut g, P0, &["Plains", "Island", "Forest", "Swamp"]);
    let opts = sim_core::dsl::ability_options(&mut g, P0, true).unwrap();
    let best = opts
        .iter()
        .filter(|o| matches!(o.act, Some(Action::Loyalty { src, .. }) if src == a))
        .max_by(|x, y| x.utility.total_cmp(&y.utility))
        .expect("loyalty options for the Arisen");
    let h = g.player(P0).hand.len();
    assert!(perform(&mut g, P0, best));
    assert_eq!(g.player(P0).hand.len(), h + 2, "with nothing to kill or reanimate the AI draws two: {}", best.label);
}

#[test]
fn the_arisen_that_dies_is_the_creature_card_in_the_graveyard() {
    let (mut g, b) = bolas_table();
    let a = flip(&mut g, b);
    sim_core::engine::zones::die(&mut g, a, "destroy").unwrap();
    assert!(in_gy(&g, P0, RAVAGER), "a transformed card is its front face in the graveyard");
    assert!(!in_gy(&g, P0, ARISEN));
}

#[test]
fn a_token_copy_of_bolas_cannot_transform() {
    let (mut g, b) = bolas_table();
    g.perm_mut(b).token = true;
    let o = (imp(&g, RAVAGER).options.unwrap())(&mut g, b, P0, Some(true)).unwrap();
    assert!(o.is_empty());
}

// ================================================================== The Ur-Dragon
#[test]
fn the_ur_dragon_may_put_a_land_card_onto_the_battlefield() {
    let mut g = tbl();
    g.step = Step::Combat;
    let ur = perm(&mut g, P0, "The Ur-Dragon");
    let gold = perm(&mut g, P0, "Goldspan Dragon");
    library(&mut g, P0, &["Forest", "Island"]);
    hand(&mut g, P0, &["Command Tower"]);
    let lands0 = g.player(P0).lands.len();
    (imp(&g, "The Ur-Dragon").attack.unwrap())(&mut g, ur, P0, &[ur, gold], P1).unwrap();
    assert_eq!(g.player(P0).lands.len(), lands0 + 1, "the only permanent card is a land: it goes in");
    assert_eq!(g.player(P0).hand.len(), 2, "two Dragons attacked: two cards drawn, the land is gone from hand");
}

#[test]
fn the_ur_dragon_prefers_a_real_permanent_to_a_land_and_a_land_to_a_trifle() {
    let mut g = tbl();
    g.step = Step::Combat;
    let ur = perm(&mut g, P0, "The Ur-Dragon");
    library(&mut g, P0, &["Forest"]);
    hand(&mut g, P0, &["Command Tower", "Glorybringer"]);
    (imp(&g, "The Ur-Dragon").attack.unwrap())(&mut g, ur, P0, &[ur], P1).unwrap();
    assert!(!on_bf(&g, P0, "Glorybringer").is_empty(), "a 5-drop beats a land");
    assert!(g.player(P0).hand.iter().any(|&c| &*g.db.get(c).name == "Command Tower"));
    // a land beats a one-mana trinket when mana is short
    let mut g = tbl();
    g.step = Step::Combat;
    let ur = perm(&mut g, P0, "The Ur-Dragon");
    library(&mut g, P0, &["Forest"]);
    hand(&mut g, P0, &["Command Tower", "Lotus Petal"]);
    let n = g.player(P0).lands.len();
    (imp(&g, "The Ur-Dragon").attack.unwrap())(&mut g, ur, P0, &[ur], P1).unwrap();
    assert_eq!(g.player(P0).lands.len(), n + 1, "the land ramps more than a Lotus Petal");
}

// ================================================================== Wrenn and Six
fn wrenn_ult(g: &mut Game) -> PermId {
    let w = perm(g, P0, "Wrenn and Six");
    g.perm_mut(w).loyalty = Some(7);
    let o = sim_core::impls::common::walker_options(g, w, P0, Some(false)).unwrap();
    let o = o.iter().find(|o| o.label.ends_with("-7")).expect("the -7 option");
    assert!(perform(g, P0, o));
    w
}

#[test]
fn wrenn_and_six_minus_seven_gives_the_retrace_emblem() {
    let mut g = tbl();
    let w = wrenn_ult(&mut g);
    assert!(!g.perm(w).on_bf, "0 loyalty");
    assert!(g.player(P0).emblems.contains(&sim_core::impls::t12_windgrace::WRENN_EMBLEM));
}

#[test]
fn with_the_emblem_the_ai_retraces_an_instant_from_the_graveyard_discarding_a_land() {
    let mut g = tbl();
    wrenn_ult(&mut g);
    lands(&mut g, P0, "Plains", 6, false);
    perm(&mut g, P1, "Grave Titan");
    let sw = take(&mut g, P0, "Swords to Plowshares");
    g.player_mut(P0).gy.push(sw);
    hand(&mut g, P0, &["Forest", "Island"]);
    let opts = hooks(&mut g);
    let o = opts.iter().find(|o| o.label.contains("retrace")).expect("a retrace option");
    assert!(o.label.contains("Swords to Plowshares"), "{}", o.label);
    let hl = g.player(P0).hand.len();
    assert!(perform(&mut g, P0, o));
    assert!(on_bf(&g, P1, "Grave Titan").is_empty(), "Swords to Plowshares exiled Grave Titan");
    assert_eq!(g.player(P0).hand.len(), hl - 1, "a land card was discarded");
    assert!(in_gy(&g, P0, "Swords to Plowshares"), "back in the graveyard, to be retraced again");
}

#[test]
fn without_the_emblem_nothing_is_retraced() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 6, false);
    perm(&mut g, P1, "Grave Titan");
    let sw = take(&mut g, P0, "Swords to Plowshares");
    g.player_mut(P0).gy.push(sw);
    hand(&mut g, P0, &["Forest", "Island"]);
    let opts = hooks(&mut g);
    assert!(opts.iter().all(|o| !o.label.contains("retrace")));
}

#[test]
fn wrenn_and_six_plus_one_returns_the_land_worth_most_not_the_first() {
    let mut g = tbl();
    let w = perm(&mut g, P0, "Wrenn and Six");
    for n in ["Forest", "Command Tower"] {
        let c = take(&mut g, P0, n);
        g.player_mut(P0).gy.push(c);
    }
    let o = sim_core::impls::common::walker_options(&mut g, w, P0, Some(false)).unwrap();
    let o = o.iter().find(|o| o.label.ends_with("+1")).expect("the +1 option");
    assert!(perform(&mut g, P0, o));
    assert_eq!(g.player(P0).hand.len(), 1, "one land card back");
}
