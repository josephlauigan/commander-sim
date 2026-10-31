//! Card models, part 2 (10-31): JD's last three Partial cards (Shifting Woodland, Redirect Lightning, Peter Parker's
//! Camera) and the pool's Partial and Unmodeled cards. Each test checks the card's Oracle behaviour, and the AI's use
//! of it where that matters. JD's decks sit only at their own tiers' tables.

use sim_core::ids::{CardId, PermId, PlayerId};
use sim_core::impls::t6_green2 as green2;
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

/// Thanos at a t6 table
fn thanos_table() -> Game {
    let mut g = table(&["thanos", "rograkh-thrasios-creature-combo", "sisay-legends-toolbox"]);
    g.active = Some(P0);
    g
}

fn fill_gy(g: &mut Game, p: PlayerId, names: &[&str]) -> Vec<CardId> {
    let cs = hand(g, p, names);
    for &c in &cs {
        sim_core::engine::zones::discard_cards(g, p, &[c]).unwrap();
    }
    cs
}

fn named(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p)
        .perms
        .iter()
        .chain(g.player(p).lands.iter())
        .copied()
        .find(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

// ======================================================== Thanos: Shifting Woodland
const WOODLAND: &str = "Shifting Woodland";

fn woodland(g: &mut Game) -> PermId {
    let c = take(g, P0, WOODLAND);
    g.add_land(P0, c, false)
}

#[test]
fn woodland_becomes_a_copy_itself_with_no_enter_triggers() {
    let mut g = thanos_table();
    let l = woodland(&mut g);
    let gy = fill_gy(&mut g, P0, &["Eternal Witness", "Opt", "Sol Ring", "Forest"]);
    let (hand0, perms0) = (g.player(P0).hand.len(), g.player(P0).perms.len());
    assert!(green2::woodland_become(&mut g, P0, l, gy[0]));
    assert_eq!(&*g.perm(l).name, "Eternal Witness");
    assert!(g.is_creature(l), "a creature now");
    let d = g.db.get(gy[0]);
    assert_eq!((g.perm(l).pow, g.perm(l).tgh), (d.pow, d.tgh), "the copied card's body");
    assert!(!g.player(P0).lands.contains(&l) && g.player(P0).perms.contains(&l), "no longer a land");
    assert_eq!(g.player(P0).perms.len(), perms0 + 1, "no new permanent: the land changed");
    assert_eq!(g.player(P0).hand.len(), hand0, "Eternal Witness's enter trigger does not fire");
    assert!(g.player(P0).gy.contains(&gy[0]), "the copied card stays in the graveyard");
    // until end of turn: the next turn it is the Woodland again, a land
    sim_core::impls::t6_green2::woodland_revert(&mut g);
    assert_eq!(&*g.perm(l).name, WOODLAND);
    assert!(!g.is_creature(l) && g.player(P0).lands.contains(&l) && !g.player(P0).perms.contains(&l));
    assert_eq!(g.perm(l).phys, None);
}

#[test]
fn woodland_copies_any_permanent_card_but_not_a_spell_or_a_planeswalker() {
    let mut g = thanos_table();
    let l = woodland(&mut g);
    let gy = fill_gy(&mut g, P0, &["Opt", "Sol Ring"]);
    assert!(!green2::woodland_become(&mut g, P0, l, gy[0]), "an instant is not a permanent card");
    assert!(green2::woodland_become(&mut g, P0, l, gy[1]), "an artifact card is");
    assert_eq!(&*g.perm(l).name, "Sol Ring");
    assert!(!g.is_creature(l));
    let jace = take(&mut g, P0, "Jace, Wielder of Mysteries");
    assert!(!green2::woodland_can_copy(&g, P0, jace), "a planeswalker copy would have no loyalty");
}

#[test]
fn woodland_copy_that_dies_puts_the_woodland_in_the_graveyard() {
    let mut g = thanos_table();
    let l = woodland(&mut g);
    let gy = fill_gy(&mut g, P0, &["Eternal Witness", "Opt", "Sol Ring", "Forest"]);
    assert!(green2::woodland_become(&mut g, P0, l, gy[0]));
    sim_core::engine::zones::die(&mut g, l, "destroy").unwrap();
    let wood = card(&g, WOODLAND);
    assert!(g.player(P0).gy.contains(&wood), "the physical card is the Woodland");
    assert_eq!(g.player(P0).gy.iter().filter(|&&c| c == gy[0]).count(), 1, "the Witness card is still one card");
    sim_core::impls::t6_green2::woodland_revert(&mut g);
    assert!(!g.player(P0).lands.contains(&l), "a dead copy does not come back as a land");
}

#[test]
fn the_ai_has_woodland_copy_oboro_and_keeps_it_untapped() {
    let mut g = thanos_table();
    let l = woodland(&mut g);
    lands(&mut g, P0, "Forest", 4, false);
    fill_gy(&mut g, P0, &["Oboro Breezecaller", "Opt", "Sol Ring", "Forest"]);
    let f = g.registry.get(g.perm(l).card()).and_then(|i| i.land_options).expect("land option");
    let opts = f(&mut g, l, P0, Some(false)).unwrap();
    assert_eq!(opts.len(), 1);
    assert!(sim_core::ai::act::perform(&mut g, P0, opts[0].act.as_ref().unwrap()).unwrap());
    assert_eq!(named(&g, P0, "Oboro Breezecaller"), Some(l));
    assert!(!g.perm(l).tapped, "the four Forests paid: the copy is untapped and can use its ability");
    assert!(g.player(P0).lands.iter().filter(|&&x| g.perm(x).tapped).count() == 4);
    let _ = P1;
}

// ======================================================== the stack: abilities as objects
/// the index of card `cd`'s first compiled activated ability
fn activated_idx(g: &Game, cd: CardId) -> u16 {
    g.db.get(cd)
        .abilities
        .iter()
        .position(|a| matches!(a, sim_core::dsl::model::Ability::Activated { .. }))
        .expect("a compiled activated ability") as u16
}

/// a spell of p's on the stack, by hand (its caster's code resolves it): (stack id, card)
fn spell_on_stack(g: &mut Game, p: PlayerId, name: &str, imp: f64) -> (u32, CardId) {
    let c = take(g, p, name);
    let it = sim_core::engine::stack::spell_item(g, p, c, sim_core::state::Ctx::default(), imp);
    let id = it.id;
    g.stack.push(it);
    (id, c)
}

fn quiet_hands(g: &mut Game) {
    for p in [P0, P1, PlayerId(2)] {
        g.player_mut(p).hand.clear();
    }
}

// ======================================================== Thanos: Redirect Lightning
const REDIRECT: &str = "Redirect Lightning";

#[test]
fn redirect_lightning_turns_an_ability_on_its_controllers_own_permanent() {
    let mut g = thanos_table();
    quiet_hands(&mut g);
    hand(&mut g, P0, &[REDIRECT]);
    lands(&mut g, P0, "Mountain", 3, false);
    let ring = perm(&mut g, P0, "Smothering Tithe");
    assert!(
        sim_core::engine::values::pval(&g, ring) >= 4.0,
        "worth protecting: {}",
        sim_core::engine::values::pval(&g, ring)
    );
    let bronto = perm(&mut g, P1, "Thrashing Brontodon");
    let theirs = perm(&mut g, P1, "Sol Ring");
    lands(&mut g, P1, "Forest", 1, false);
    let cd = g.perm(bronto).card();
    let idx = activated_idx(&g, cd);
    assert_eq!(sim_core::dsl::aim(&g, P1, cd, idx, Some(bronto)), Some(ring), "aimed at the best artifact");
    let life = g.player(P0).life;
    assert!(sim_core::dsl::activate(&mut g, P1, bronto, idx).unwrap());
    assert!(g.perm(ring).on_bf, "Smothering Tithe survives");
    assert!(!g.perm(theirs).on_bf, "the Brontodon destroyed its controller's own Sol Ring");
    assert!(g.player(P0).gy.contains(&card(&g, REDIRECT)));
    assert_eq!(g.player(P0).life, life, "{{2}} paid rather than 5 life");
}

#[test]
fn redirect_lightning_pays_life_when_mana_is_short() {
    let mut g = thanos_table();
    quiet_hands(&mut g);
    hand(&mut g, P0, &[REDIRECT]);
    lands(&mut g, P0, "Mountain", 1, false);
    let ring = perm(&mut g, P0, "Smothering Tithe");
    let bronto = perm(&mut g, P1, "Thrashing Brontodon");
    perm(&mut g, P1, "Sol Ring");
    lands(&mut g, P1, "Forest", 1, false);
    let life = g.player(P0).life;
    let idx = activated_idx(&g, g.perm(bronto).card());
    assert!(sim_core::dsl::activate(&mut g, P1, bronto, idx).unwrap());
    assert!(g.perm(ring).on_bf);
    assert_eq!(g.player(P0).life, life - 5);
}

#[test]
fn an_ability_with_no_other_legal_target_is_not_redirected() {
    let mut g = thanos_table();
    quiet_hands(&mut g);
    hand(&mut g, P0, &[REDIRECT]);
    lands(&mut g, P0, "Mountain", 3, false);
    let ring = perm(&mut g, P0, "Smothering Tithe");
    let bronto = perm(&mut g, P1, "Thrashing Brontodon");
    lands(&mut g, P1, "Forest", 1, false);
    let idx = activated_idx(&g, g.perm(bronto).card());
    assert!(sim_core::dsl::activate(&mut g, P1, bronto, idx).unwrap());
    assert!(!g.perm(ring).on_bf, "no other artifact or enchantment: the Tithe is destroyed");
    assert!(g.player(P0).hand.contains(&card(&g, REDIRECT)), "Redirect Lightning is kept");
}

#[test]
fn an_ability_on_the_stack_reports_its_target_as_it_leaves() {
    // the target is chosen as the ability goes on the stack and comes back with it (unchanged: nobody redirects)
    let mut g = thanos_table();
    quiet_hands(&mut g);
    let bronto = perm(&mut g, P1, "Thrashing Brontodon");
    lands(&mut g, P1, "Forest", 1, false);
    let cd = g.perm(bronto).card();
    let ring = perm(&mut g, P0, "Smothering Tithe");
    let idx = activated_idx(&g, cd);
    let ctx = sim_core::state::Ctx { target: Some(ring), dsl_ab: Some((cd, idx)), ..Default::default() };
    // the stack round runs only when someone could answer: P0 holds a redirect it can't pay for
    hand(&mut g, P0, &[REDIRECT]);
    let r = sim_core::engine::stack::ability_window_ex(&mut g, P1, Some(bronto), "ability", None, ctx).unwrap();
    assert!(r.resolves && r.target == Some(ring) && r.copies == 0);
}

#[test]
fn redirect_lightning_turns_a_counterspell_on_another_spell() {
    let mut g = thanos_table();
    quiet_hands(&mut g);
    hand(&mut g, P0, &[REDIRECT]);
    lands(&mut g, P0, "Mountain", 3, false);
    lands(&mut g, P1, "Island", 2, false);
    let (other, _) = spell_on_stack(&mut g, PlayerId(2), "Demonic Tutor", 4.0);
    let (mine, _) = spell_on_stack(&mut g, P0, "Ad Nauseam", 8.0);
    let ctr = hand(&mut g, P1, &["Counterspell"])[0];
    assert!(sim_core::engine::stack::cast_counter_spell(&mut g, P1, ctr, mine).unwrap());
    let countered = |g: &Game, id: u32| g.stack.iter().find(|x| x.id == id).unwrap().countered;
    assert!(!countered(&g, mine), "P0's spell is not countered");
    assert!(countered(&g, other), "the Counterspell countered the other spell instead");
    assert!(g.player(P0).gy.contains(&card(&g, REDIRECT)));
}

#[test]
fn redirect_lightning_is_kept_when_the_counter_has_no_other_spell_to_hit() {
    let mut g = thanos_table();
    quiet_hands(&mut g);
    hand(&mut g, P0, &[REDIRECT]);
    lands(&mut g, P0, "Mountain", 3, false);
    lands(&mut g, P1, "Island", 2, false);
    let (mine, _) = spell_on_stack(&mut g, P0, "Ad Nauseam", 8.0);
    let ctr = hand(&mut g, P1, &["Counterspell"])[0];
    assert!(sim_core::engine::stack::cast_counter_spell(&mut g, P1, ctr, mine).unwrap());
    assert!(g.stack.iter().find(|x| x.id == mine).unwrap().countered);
    assert!(g.player(P0).hand.contains(&card(&g, REDIRECT)));
}

// ======================================================== urdragon: Peter Parker's Camera
const CAMERA: &str = "Peter Parker's Camera";

fn urdragon_table() -> Game {
    let mut g = table(&["urdragon", "zur-esper-enchantment-control", "miirym-temur-dragons"]);
    g.active = Some(P0);
    g
}

#[test]
fn the_camera_copies_an_activated_tutor_ability() {
    let mut g = urdragon_table();
    quiet_hands(&mut g);
    let cam = perm(&mut g, P0, CAMERA);
    assert_eq!(g.perm(cam).data.int(sim_core::state::DataKey::Counters), 3, "three film counters");
    let shaman = perm(&mut g, P0, "Fauna Shaman");
    lands(&mut g, P0, "Forest", 3, false);
    hand(&mut g, P0, &["Llanowar Elves"]); // the creature card to discard
    let creatures = |g: &Game| g.player(P0).library.iter().filter(|&&c| g.db.get(c).creature).count();
    let before = creatures(&g);
    let idx = activated_idx(&g, g.perm(shaman).card());
    assert!(sim_core::dsl::activate(&mut g, P0, shaman, idx).unwrap());
    assert_eq!(before - creatures(&g), 2, "the copy found a creature card too");
    assert_eq!(g.player(P0).hand.iter().filter(|&&c| g.db.get(c).creature).count(), 2);
    assert!(g.perm(cam).tapped);
    assert_eq!(g.perm(cam).data.int(sim_core::state::DataKey::Counters), 2, "a film counter used");
}

#[test]
fn the_camera_is_not_used_without_mana_for_it() {
    let mut g = urdragon_table();
    quiet_hands(&mut g);
    let cam = perm(&mut g, P0, CAMERA);
    let shaman = perm(&mut g, P0, "Fauna Shaman");
    lands(&mut g, P0, "Forest", 1, false);
    hand(&mut g, P0, &["Llanowar Elves"]);
    let idx = activated_idx(&g, g.perm(shaman).card());
    assert!(sim_core::dsl::activate(&mut g, P0, shaman, idx).unwrap());
    assert_eq!(g.player(P0).hand.iter().filter(|&&c| g.db.get(c).creature).count(), 1, "one search");
    assert!(!g.perm(cam).tapped);
}

// ======================================================== the pool: Wan Shi Tong, Librarian (t6 kinnan-bonder, rograkh-thrasios)
#[test]
fn wan_shi_tong_answers_an_opponents_real_search() {
    let mut g = table(&["rograkh-thrasios-creature-combo", "sisay-legends-toolbox", "kinnan-bonder-mana-combo"]);
    g.active = Some(P1);
    quiet_hands(&mut g);
    let w = perm(&mut g, P0, "Wan Shi Tong, Librarian");
    let p = sim_core::engine::values::epow(&g, w);
    let h = g.player(P0).hand.len();
    lands(&mut g, P1, "Swamp", 2, false);
    let tutor = hand(&mut g, P1, &["Demonic Tutor"])[0];
    assert!(sim_core::engine::cast::cast_card(&mut g, P1, tutor, "hand", sim_core::state::Ctx::default()).unwrap());
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert_eq!(sim_core::engine::values::epow(&g, w), p + 1, "a +1/+1 counter");
    assert_eq!(g.player(P0).hand.len(), h + 1, "and a card");
    // its controller's own search does nothing
    g.active = Some(P0);
    lands(&mut g, P0, "Island", 2, false);
    sim_core::engine::tutors::shuffle_library(&mut g, P0);
    sim_core::engine::life::check_state(&mut g).unwrap();
    assert_eq!(sim_core::engine::values::epow(&g, w), p + 1);
}

// ======================================================== the pool: Flesh Duplicate (t6 kinnan-bonder, rograkh-thrasios)
#[test]
fn flesh_duplicate_copies_with_vanishing_three() {
    let mut g = table(&["rograkh-thrasios-creature-combo", "sisay-legends-toolbox", "kinnan-bonder-mana-combo"]);
    g.active = Some(P0);
    let big = perm(&mut g, P1, "Craterhoof Behemoth");
    let _ = big;
    let dup = perm(&mut g, P0, "Flesh Duplicate");
    assert_eq!(&*g.perm(dup).name, "Craterhoof Behemoth", "a copy of the best creature");
    assert_eq!(g.perm(dup).data.int(sim_core::state::DataKey::Vanish), 3, "three time counters");
    sim_core::impls::t6_copies::vanishing_upkeep(&mut g, P1).unwrap();
    assert_eq!(g.perm(dup).data.int(sim_core::state::DataKey::Vanish), 3, "only its controller's upkeep");
    for left in [2, 1] {
        sim_core::impls::t6_copies::vanishing_upkeep(&mut g, P0).unwrap();
        assert_eq!(g.perm(dup).data.int(sim_core::state::DataKey::Vanish), left);
        assert!(g.perm(dup).on_bf);
    }
    sim_core::impls::t6_copies::vanishing_upkeep(&mut g, P0).unwrap();
    assert!(!g.perm(dup).on_bf, "the last counter: sacrificed");
    assert!(g.player(P0).gy.contains(&card(&g, "Flesh Duplicate")), "the Duplicate card goes to the graveyard");
    assert!(g.perm(big).on_bf);
}

// ======================================================== the pool: Phantasmal Image (t6 rograkh-thrasios)
#[test]
fn phantasmal_image_is_sacrificed_when_targeted() {
    let mut g = table(&["rograkh-thrasios-creature-combo", "sisay-legends-toolbox", "kinnan-bonder-mana-combo"]);
    g.active = Some(P0);
    quiet_hands(&mut g);
    perm(&mut g, P1, "Craterhoof Behemoth");
    let img = perm(&mut g, P0, "Phantasmal Image");
    assert_eq!(&*g.perm(img).name, "Craterhoof Behemoth");
    assert!(sim_core::impls::t6_copies::is_image(&g, img));
    // an exile effect that would not destroy it anyway: the Image is sacrificed as it becomes the target
    sim_core::engine::removal::apply_removal(&mut g, Some(P1), img, "bounce", None).unwrap();
    assert!(!g.perm(img).on_bf);
    assert!(g.player(P0).gy.contains(&card(&g, "Phantasmal Image")), "sacrificed: in the graveyard, not the hand");
}

#[test]
fn only_a_phantasmal_image_is_sacrificed_when_targeted() {
    let mut g = table(&["rograkh-thrasios-creature-combo", "sisay-legends-toolbox", "kinnan-bonder-mana-combo"]);
    g.active = Some(P0);
    perm(&mut g, P1, "Craterhoof Behemoth");
    let img = perm(&mut g, P0, "Phantasmal Image");
    let elf = perm(&mut g, P0, "Llanowar Elves");
    assert!(!sim_core::impls::t6_copies::image_targeted(&mut g, elf).unwrap(), "not an Image");
    assert!(g.perm(img).on_bf);
}

// ======================================================== the pool: Mnemonic Betrayal (t6 kraum-tymna, rograkh-silas)
#[test]
fn mnemonic_betrayal_returns_the_uncast_cards_to_their_owners_graveyards() {
    let mut g = table(&["kraum-tymna-blue-breach-midrange", "sisay-legends-toolbox", "kinnan-bonder-mana-combo"]);
    g.active = Some(P0);
    quiet_hands(&mut g);
    let theirs = fill_gy(&mut g, P1, &["Swords to Plowshares", "Forest"]);
    lands(&mut g, P0, "Island", 2, false);
    lands(&mut g, P0, "Swamp", 2, false);
    let mb = hand(&mut g, P0, &["Mnemonic Betrayal"])[0];
    assert!(sim_core::engine::cast::cast_card(&mut g, P0, mb, "hand", sim_core::state::Ctx::default()).unwrap());
    assert!(g.player(P0).hand.contains(&theirs[0]), "the nonland card can be cast this turn");
    assert!(g.player(P1).exile.contains(&theirs[1]), "the land is exiled");
    assert!(g.player(P0).exile.contains(&mb), "Betrayal exiles itself");
    sim_core::impls::t6_blue::betrayal_return(&mut g);
    assert!(!g.player(P0).hand.contains(&theirs[0]));
    assert!(g.player(P1).gy.contains(&theirs[0]), "back in its owner's graveyard");
    assert!(!g.player(P0).stolen.contains_key(&theirs[0]));
}
