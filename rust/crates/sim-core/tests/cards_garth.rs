//! Garth One-Eye (impls/garth.rs): "{T}: Choose a card name that hasn't been chosen from among Disenchant,
//! Braingeyser, Terror, Shivan Dragon, Regrowth, and Black Lotus. Create a copy of the card with the chosen name. You
//! may cast the copy. (You still pay its costs.)"

use sim_core::engine::mana::{cost_of, pay, total_mana};
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::garth::{self as gar, NAMES};
use sim_core::state::{DataKey, Game, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

const DISENCHANT: usize = 0;
const BRAINGEYSER: usize = 1;
const TERROR: usize = 2;
const SHIVAN: usize = 3;
const REGROWTH: usize = 4;
const LOTUS: usize = 5;

fn tbl() -> Game {
    let mut g =
        table(&["urdragon", "miirym-temur-dragons", "urza-mono-blue-artifacts", "zur-esper-enchantment-control"]);
    g.player_mut(P0).hand.clear();
    g
}

/// the Ur-Dragon out of the command zone (its eminence would change a Dragon's cost)
fn no_eminence(g: &mut Game) {
    let c = card(g, "The Ur-Dragon");
    g.player_mut(P0).set_cmd_zone(c, false);
}

fn garth(g: &mut Game) -> PermId {
    perm(g, P0, "Garth One-Eye")
}

fn on(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

/// every card of p's that is a token on the battlefield
fn tokens(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p).perms.iter().copied().filter(|&m| g.perm(m).on_bf && g.perm(m).token).collect()
}

fn untapped(g: &Game, ls: &[PermId]) -> usize {
    ls.iter().filter(|&&l| !g.perm(l).tapped).count()
}

/// is the card anywhere p can see it (hand, graveyard, exile, library)? A copy is nowhere.
fn somewhere(g: &Game, p: PlayerId, c: CardId) -> bool {
    let pl = g.player(p);
    pl.hand.contains(&c) || pl.gy.contains(&c) || pl.exile.contains(&c)
}

fn options(g: &mut Game, m: PermId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(card(g, "Garth One-Eye")).unwrap().options.unwrap();
    f(g, m, P0, post).unwrap()
}

fn run(g: &mut Game, o: &Opt) -> bool {
    match o.act.as_ref().unwrap() {
        Action::Ability { src, f, arg } => f(g, *src, P0, *arg).unwrap(),
        other => panic!("unexpected action {other:?}"),
    }
}

fn labelled<'a>(opts: &'a [Opt], what: &str) -> Option<&'a Opt> {
    opts.iter().find(|o| o.label.contains(what))
}

// ======================================================== the six copies
#[test]
fn terror_destroys_a_nonartifact_nonblack_creature_and_costs_two() {
    let mut g = tbl();
    let ls = lands(&mut g, P0, "Swamp", 3, false);
    let m = garth(&mut g);
    let hoof = perm(&mut g, P1, "Craterhoof Behemoth");
    assert!(gar::activate(&mut g, m, P0, TERROR, hoof.0).unwrap());
    assert!(!on(&g, hoof), "destroyed");
    assert!(g.perm(m).tapped, "Garth taps");
    assert_eq!(untapped(&g, &ls), 1, "{{1}}{{B}} is paid");
    assert!(!somewhere(&g, P0, card(&g, "Terror")), "the copy is not a card: it goes nowhere");
}

#[test]
fn terror_cannot_target_a_black_or_an_artifact_creature() {
    let mut g = tbl();
    lands(&mut g, P0, "Swamp", 3, false);
    let m = garth(&mut g);
    let titan = perm(&mut g, P1, "Grave Titan"); // black
    let golem = perm(&mut g, P1, "Solemn Simulacrum"); // artifact creature
    for t in [titan, golem] {
        assert!(!gar::activate(&mut g, m, P0, TERROR, t.0).unwrap());
        assert!(on(&g, t) && !g.perm(m).tapped, "nothing happens, and Garth stays untapped");
    }
    assert!(gar::unchosen(&g, m).contains(&TERROR), "the name is still available");
}

#[test]
fn disenchant_destroys_an_artifact_or_enchantment_and_costs_two() {
    let mut g = tbl();
    let ls = lands(&mut g, P0, "Plains", 3, false);
    let m = garth(&mut g);
    let ring = perm(&mut g, P1, "Sol Ring");
    assert!(gar::activate(&mut g, m, P0, DISENCHANT, ring.0).unwrap());
    assert!(!on(&g, ring));
    assert_eq!(untapped(&g, &ls), 1, "{{1}}{{W}} is paid");
    // a creature is not a legal target
    g.perm_mut(m).tapped = false;
    let hoof = perm(&mut g, P1, "Craterhoof Behemoth");
    g.perm_mut(m).data.set(DataKey::Garth, Val::Int(0));
    assert!(!gar::activate(&mut g, m, P0, DISENCHANT, hoof.0).unwrap());
    assert!(on(&g, hoof));
}

#[test]
fn braingeyser_draws_x_and_the_x_is_paid() {
    let mut g = tbl();
    let ls = lands(&mut g, P0, "Island", 6, false);
    let m = garth(&mut g);
    let lib = g.player(P0).library.len();
    assert!(gar::activate(&mut g, m, P0, BRAINGEYSER, 4).unwrap());
    assert_eq!(g.player(P0).hand.len(), 4, "draws X cards");
    assert_eq!(g.player(P0).library.len(), lib - 4);
    assert_eq!(untapped(&g, &ls), 0, "X=4 and {{U}}{{U}}: six mana");
    // X cannot exceed the mana
    let mut g = tbl();
    lands(&mut g, P0, "Island", 4, false);
    let m = garth(&mut g);
    assert!(!gar::activate(&mut g, m, P0, BRAINGEYSER, 3).unwrap(), "X=3 needs five mana");
    assert!(g.player(P0).hand.is_empty() && !g.perm(m).tapped);
}

#[test]
fn regrowth_returns_a_card_from_the_graveyard_to_hand_and_costs_two() {
    let mut g = tbl();
    let ls = lands(&mut g, P0, "Forest", 3, false);
    let m = garth(&mut g);
    let hoof = card(&g, "Craterhoof Behemoth");
    g.player_mut(P0).gy.push(hoof);
    assert!(gar::activate(&mut g, m, P0, REGROWTH, 0).unwrap());
    assert!(g.player(P0).hand.contains(&hoof) && !g.player(P0).gy.contains(&hoof));
    assert_eq!(untapped(&g, &ls), 1, "{{1}}{{G}} is paid");
    assert!(!g.player(P0).gy.contains(&card(&g, "Regrowth")) && !g.player(P0).hand.contains(&card(&g, "Regrowth")));
    // an empty graveyard: the copy would do nothing, so it is not made
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 3, false);
    let m = garth(&mut g);
    assert!(!gar::activate(&mut g, m, P0, REGROWTH, 0).unwrap() && !g.perm(m).tapped);
}

#[test]
fn shivan_dragon_is_a_five_five_flier_that_costs_mana_less_the_eminence() {
    let mut g = tbl();
    let c = card(&g, "Shivan Dragon");
    assert_eq!(cost_of(&g, P0, c), (3, "RR".to_string()), "{{4}}{{R}}{{R}} less {{1}}: The Ur-Dragon's eminence");
    let ls = lands(&mut g, P0, "Mountain", 6, false);
    let m = garth(&mut g);
    assert!(gar::activate(&mut g, m, P0, SHIVAN, 0).unwrap());
    assert_eq!(untapped(&g, &ls), 1, "five mana, not free and not six");
    let toks = tokens(&g, P0);
    let d = toks.iter().copied().find(|&t| &*g.perm(t).name == "Shivan Dragon").expect("a Shivan Dragon token");
    assert_eq!((g.perm(d).pow, g.perm(d).tgh), (5, 5));
    assert!(sim_core::dsl::has_kw(&g, d, "flying"));
    // without the eminence it costs the full six
    let mut g = tbl();
    no_eminence(&mut g);
    let ls = lands(&mut g, P0, "Mountain", 5, false);
    let m = garth(&mut g);
    assert!(!gar::activate(&mut g, m, P0, SHIVAN, 0).unwrap(), "five mana is not enough");
    assert_eq!(untapped(&g, &ls), 5);
}

#[test]
fn black_lotus_is_a_token_that_makes_three_mana_and_is_sacrificed() {
    let mut g = tbl();
    let m = garth(&mut g);
    assert!(gar::activate(&mut g, m, P0, LOTUS, 0).unwrap(), "it costs nothing");
    let lotus: Vec<PermId> = tokens(&g, P0).into_iter().filter(|&t| &*g.perm(t).name == "Black Lotus").collect();
    assert_eq!(lotus.len(), 1);
    assert_eq!(total_mana(&g, P0, false), 3);
    assert!(pay(&mut g, P0, 3, "", false).unwrap());
    assert!(!on(&g, lotus[0]), "sacrificed for its mana");
    assert_eq!(total_mana(&g, P0, false), 0);
}

// ======================================================== names
#[test]
fn a_garth_chooses_each_name_once_but_a_new_garth_chooses_it_again() {
    let mut g = tbl();
    let m = garth(&mut g);
    assert!(gar::activate(&mut g, m, P0, LOTUS, 0).unwrap());
    g.perm_mut(m).tapped = false;
    assert!(!gar::activate(&mut g, m, P0, LOTUS, 0).unwrap(), "Black Lotus is chosen");
    assert_eq!(gar::unchosen(&g, m), vec![DISENCHANT, BRAINGEYSER, TERROR, SHIVAN, REGROWTH]);
    assert_eq!(tokens(&g, P0).len(), 1, "one Lotus");
    // Garth dies and comes back: a new object
    sim_core::engine::zones::die(&mut g, m, "destroy").unwrap();
    let m2 = garth(&mut g);
    assert_eq!(gar::unchosen(&g, m2).len(), NAMES.len());
    assert!(gar::activate(&mut g, m2, P0, LOTUS, 0).unwrap(), "chosen again by the new Garth");
    assert_eq!(tokens(&g, P0).len(), 2);
}

#[test]
fn a_tapped_or_summoning_sick_garth_does_nothing() {
    let mut g = tbl();
    let m = garth(&mut g);
    g.perm_mut(m).tapped = true;
    assert!(!gar::activate(&mut g, m, P0, LOTUS, 0).unwrap());
    g.perm_mut(m).tapped = false;
    g.perm_mut(m).sick = true;
    assert!(!gar::activate(&mut g, m, P0, LOTUS, 0).unwrap());
}

// ======================================================== the copy is not free, and ceases to exist uncast
#[test]
fn an_uncast_copy_ceases_to_exist() {
    let mut g = tbl();
    lands(&mut g, P0, "Swamp", 2, false);
    let hoof = perm(&mut g, P1, "Craterhoof Behemoth");
    let terror = card(&g, "Terror");
    // no mana: it cannot be cast
    for l in g.player(P0).lands.clone() {
        g.perm_mut(l).tapped = true;
    }
    assert!(!gar::cast_copy(&mut g, P0, terror, Some(hoof), 0).unwrap());
    assert!(on(&g, hoof) && !somewhere(&g, P0, terror) && !somewhere(&g, P1, terror));
    // its target left before it could be cast
    for l in g.player(P0).lands.clone() {
        g.perm_mut(l).tapped = false;
    }
    sim_core::engine::zones::die(&mut g, hoof, "destroy").unwrap();
    assert!(!gar::cast_copy(&mut g, P0, terror, Some(hoof), 0).unwrap());
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| g.perm(l).tapped).count(), 0, "no mana spent");
    assert!(!somewhere(&g, P0, terror));
}

#[test]
fn the_copy_is_cast_so_it_counts_as_a_spell() {
    let mut g = tbl();
    lands(&mut g, P0, "Plains", 2, false);
    let m = garth(&mut g);
    let ring = perm(&mut g, P1, "Sol Ring");
    let before = g.player(P0).spells_this_turn;
    assert!(gar::activate(&mut g, m, P0, DISENCHANT, ring.0).unwrap());
    assert_eq!(g.player(P0).spells_this_turn, before + 1);
}

// ======================================================== the AI
#[test]
fn the_ai_taps_garth_for_terror_when_a_creature_is_worth_killing() {
    let mut g = tbl();
    lands(&mut g, P0, "Swamp", 2, false);
    let m = garth(&mut g);
    let hoof = perm(&mut g, P1, "Craterhoof Behemoth");
    let opts = options(&mut g, m, Some(false));
    let o = labelled(&opts, "Terror").expect("Terror is offered");
    assert!(o.utility > 0.0 && o.label.contains("Craterhoof"), "{}", o.label);
    assert!(run(&mut g, o));
    assert!(!on(&g, hoof));
}

#[test]
fn the_ai_offers_terror_at_the_end_of_the_turn_but_not_the_sorcery_speed_names() {
    let mut g = tbl();
    lands(&mut g, P0, "Swamp", 2, false);
    lands(&mut g, P0, "Mountain", 6, false);
    lands(&mut g, P0, "Forest", 1, false);
    let m = garth(&mut g);
    perm(&mut g, P1, "Craterhoof Behemoth");
    let hoof_gy = card(&g, "Craterhoof Behemoth");
    g.player_mut(P0).gy.push(hoof_gy);
    let opts = options(&mut g, m, None);
    assert!(labelled(&opts, "Terror").is_some(), "an instant works in the end-of-turn window");
    for sorcery in ["Shivan", "Regrowth", "Braingeyser", "Lotus"] {
        assert!(labelled(&opts, sorcery).is_none(), "{sorcery} needs a main phase");
    }
    let main = options(&mut g, m, Some(true));
    assert!(labelled(&main, "Shivan").is_some() && labelled(&main, "Regrowth").is_some());
}

#[test]
fn the_ai_taps_garth_for_black_lotus_when_its_mana_makes_a_spell_castable() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 5, false);
    let m = garth(&mut g);
    // {5}{G}{G}: seven mana, five lands and a Lotus's three make it castable
    hand(&mut g, P0, &["Craterhoof Behemoth"]);
    let opts = options(&mut g, m, Some(false));
    let o = labelled(&opts, "Black Lotus").expect("Black Lotus is offered");
    assert!(o.utility >= 5.0, "worth a lot: {}", o.utility);
    assert!(run(&mut g, o));
    assert_eq!(total_mana(&g, P0, false), 5 + 3);
    // with nothing to enable it is the last resort
    let mut g = tbl();
    let m = garth(&mut g);
    let opts = options(&mut g, m, Some(false));
    let o = labelled(&opts, "Black Lotus").expect("still offered");
    assert!(o.utility < 1.0, "the last choice: {}", o.utility);
}

#[test]
fn the_ai_does_not_tap_garth_when_no_choice_can_be_cast_and_used() {
    let mut g = tbl();
    let m = garth(&mut g);
    perm(&mut g, P1, "Craterhoof Behemoth");
    perm(&mut g, P1, "Sol Ring");
    // Black Lotus is the only name that costs nothing: with it chosen, no mana means no choice
    g.perm_mut(m).data.set(DataKey::Garth, Val::Int(1 << LOTUS));
    assert!(options(&mut g, m, Some(false)).is_empty(), "no mana: nothing is castable");
    assert!(options(&mut g, m, None).is_empty());
    // mana, but nothing worth the copy: an empty board, an empty graveyard, no X of 3
    let mut g2 = tbl();
    lands(&mut g2, P0, "Swamp", 2, false);
    let m2 = garth(&mut g2);
    g2.perm_mut(m2).data.set(DataKey::Garth, Val::Int(1 << LOTUS));
    assert!(options(&mut g2, m2, Some(false)).is_empty(), "no target, no graveyard card, no X, no Dragon mana");
    // black and artifact creatures are not Terror targets, so Terror is not offered for them
    perm(&mut g2, P1, "Grave Titan");
    perm(&mut g2, P1, "Solemn Simulacrum");
    assert!(labelled(&options(&mut g2, m2, Some(false)), "Terror").is_none());
    // a tapped Garth is not offered at all
    g2.perm_mut(m2).tapped = true;
    g2.perm_mut(m2).data.set(DataKey::Garth, Val::Int(0));
    assert!(options(&mut g2, m2, Some(false)).is_empty());
}

#[test]
fn the_ai_brings_back_a_good_card_and_draws_with_spare_mana() {
    let mut g = tbl();
    lands(&mut g, P0, "Forest", 3, false);
    lands(&mut g, P0, "Island", 5, false);
    let m = garth(&mut g);
    // a land in the graveyard is not worth Regrowth
    let forest = card(&g, "Forest");
    g.player_mut(P0).gy.push(forest);
    let opts = options(&mut g, m, Some(false));
    assert!(labelled(&opts, "Regrowth").is_none());
    let b = labelled(&opts, "Braingeyser for").expect("eight mana: X=6 with UU");
    assert!(b.label.contains("for 6"), "{}", b.label);
    let hoof = card(&g, "Craterhoof Behemoth");
    g.player_mut(P0).gy.push(hoof);
    let opts = options(&mut g, m, Some(false));
    assert!(labelled(&opts, "Regrowth (Craterhoof Behemoth)").is_some());
    // too little mana for X=3: no Braingeyser
    let mut g = tbl();
    lands(&mut g, P0, "Island", 4, false);
    let m = garth(&mut g);
    assert!(labelled(&options(&mut g, m, Some(false)), "Braingeyser").is_none());
}
