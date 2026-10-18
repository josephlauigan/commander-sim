//! Tier 6 batch `blue` (impls/t6_blue.rs) on hand-built positions. Seat 0 plays the Rograkh & Silas deck, seat 1
//! the seph deck.

use sim_core::engine::cast;
use sim_core::ids::{CardId, PlayerId};
use sim_core::impls::t6_blue as t6;
use sim_core::state::{Ctx, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    let mut g = table(&["rograkh-silas-turbo-breach", "seph"]);
    g.active = Some(P0);
    g
}

fn cast_free(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = hand(g, p, &[name])[0];
    cast::cast_card(g, p, c, "hand", Ctx::default()).unwrap();
    c
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

// ======================================================== Borne Upon a Wind
#[test]
fn borne_upon_a_wind_draws_and_grants_flash_this_turn() {
    let mut g = game();
    let before = g.player(P0).hand.len();
    cast_free(&mut g, P0, "Borne Upon a Wind");
    assert_eq!(g.player(P0).hand.len(), before + 1, "the card left the hand, one was drawn");
    assert!(in_gy(&g, P0, "Borne Upon a Wind"));
    assert!(t6::flash_turn_grant(&g, P0));
    assert!(!t6::flash_turn_grant(&g, P1), "only the caster");
}

// ======================================================== Mnemonic Betrayal
fn put_gy(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = take(g, p, name);
    g.player_mut(p).gy.push(c);
    c
}

#[test]
fn mnemonic_betrayal_takes_opposing_graveyards_for_the_turn() {
    let mut g = game();
    let bolt = put_gy(&mut g, P1, "Lightning Bolt");
    let land = put_gy(&mut g, P1, "Island");
    let mine = put_gy(&mut g, P0, "Opt");
    cast_free(&mut g, P0, "Mnemonic Betrayal");
    assert!(g.player(P1).gy.is_empty(), "the opposing graveyard is exiled");
    assert!(g.player(P1).exile.contains(&land), "a land can't be cast: it stays in exile");
    assert!(g.player(P0).hand.contains(&bolt), "the spell can be cast from among them");
    assert!(g.player(P0).agent_ids.contains(&bolt), "with mana of any type");
    assert_eq!(g.player(P0).stolen.get(&bolt), Some(&P1), "it still belongs to its owner");
    assert!(g.player(P0).impulse.contains(&bolt), "only this turn");
    assert!(g.player(P0).gy.contains(&mine), "own graveyard untouched");
    assert!(g.player(P0).exile.iter().any(|&c| &*g.db.get(c).name == "Mnemonic Betrayal"), "Betrayal is exiled");
}

// ======================================================== Timetwister
#[test]
fn timetwister_wheels_every_player_for_seven() {
    let mut g = game();
    let a = put_gy(&mut g, P0, "Opt");
    let b = put_gy(&mut g, P1, "Lightning Bolt");
    hand(&mut g, P1, &["Island", "Island"]);
    let total: usize = [P0, P1].iter().map(|&q| g.player(q).library.len() + g.player(q).hand.len() + g.player(q).gy.len()).sum();
    cast_free(&mut g, P0, "Timetwister");
    assert_eq!(g.player(P0).hand.len(), 7);
    assert_eq!(g.player(P1).hand.len(), 7);
    assert_eq!(g.player(P0).gy.len(), 1, "only Timetwister itself is in the graveyard");
    assert!(in_gy(&g, P0, "Timetwister"));
    assert!(g.player(P1).gy.is_empty());
    let now: usize = [P0, P1].iter().map(|&q| g.player(q).library.len() + g.player(q).hand.len() + g.player(q).gy.len()).sum();
    assert_eq!(total, now, "no card is lost");
    let all = |g: &Game, q: PlayerId, c: CardId| g.player(q).library.contains(&c) || g.player(q).hand.contains(&c);
    assert!(all(&g, P0, a) && all(&g, P1, b), "the graveyard cards were shuffled in");
}

// ======================================================== Demonic Counsel
fn lib_with(g: &mut Game, p: PlayerId, name: &str) -> CardId {
    let c = take(g, p, name);
    g.player_mut(p).library.push(c);
    c
}

#[test]
fn demonic_counsel_finds_a_demon_without_delirium() {
    let mut g = game();
    let demon = lib_with(&mut g, P0, "Doom Whisperer");
    let other = lib_with(&mut g, P0, "Grave Titan");
    cast_free(&mut g, P0, "Demonic Counsel");
    assert!(g.player(P0).hand.contains(&demon));
    assert!(!g.player(P0).hand.contains(&other), "only a Demon without delirium");
    assert!(in_gy(&g, P0, "Demonic Counsel"));
}

#[test]
fn demonic_counsel_without_a_demon_finds_nothing() {
    let mut g = game();
    lib_with(&mut g, P0, "Grave Titan");
    let n = g.player(P0).hand.len();
    cast_free(&mut g, P0, "Demonic Counsel");
    assert_eq!(g.player(P0).hand.len(), n, "the card left, nothing came");
}

#[test]
fn demonic_counsel_with_delirium_finds_any_card() {
    let mut g = game();
    for n in ["Island", "Grave Titan", "Lightning Bolt", "Sol Ring"] {
        put_gy(&mut g, P0, n);
    }
    assert!(sim_core::impls::t6_green2::delirium(&g, P0));
    let n = g.player(P0).hand.len();
    cast_free(&mut g, P0, "Demonic Counsel");
    assert_eq!(g.player(P0).hand.len(), n + 1, "one card was tutored");
}

// ======================================================== Rain of Filth
#[test]
fn rain_of_filth_sets_the_turn_flag_for_the_caster() {
    let mut g = game();
    assert!(!t6::rain_of_filth_active(&g, P0));
    cast_free(&mut g, P0, "Rain of Filth");
    assert!(t6::rain_of_filth_active(&g, P0));
    assert!(!t6::rain_of_filth_active(&g, P1));
    assert!(in_gy(&g, P0, "Rain of Filth"));
}

// ======================================================== Sevinne's Reclamation
fn on_bf(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

#[test]
fn sevinnes_reclamation_returns_a_cheap_permanent() {
    let mut g = game();
    put_gy(&mut g, P0, "Sol Ring");
    put_gy(&mut g, P0, "Grave Titan"); // mana value 6: not allowed
    put_gy(&mut g, P0, "Opt");
    cast_free(&mut g, P0, "Sevinne's Reclamation");
    assert!(on_bf(&g, P0, "Sol Ring"));
    assert!(!on_bf(&g, P0, "Grave Titan"));
    assert!(in_gy(&g, P0, "Grave Titan") && in_gy(&g, P0, "Opt"));
    assert!(in_gy(&g, P0, "Sevinne's Reclamation"), "hard-cast it goes to the graveyard (flashback later)");
}

#[test]
fn sevinnes_reclamation_flashback_is_copied_and_exiled() {
    let mut g = game();
    put_gy(&mut g, P0, "Sol Ring");
    put_gy(&mut g, P0, "Fellwar Stone");
    let rec = put_gy(&mut g, P0, "Sevinne's Reclamation");
    cast::cast_card(&mut g, P0, rec, "gy", Ctx::default()).unwrap();
    assert!(on_bf(&g, P0, "Sol Ring") && on_bf(&g, P0, "Fellwar Stone"), "the copy returns a second card");
    assert!(g.player(P0).exile.contains(&rec), "flashback exiles it");
    assert!(!g.player(P0).gy.contains(&rec));
}

#[test]
fn sevinnes_reclamation_can_return_a_land() {
    let mut g = game();
    let before = g.player(P0).lands.len();
    put_gy(&mut g, P0, "Island");
    cast_free(&mut g, P0, "Sevinne's Reclamation");
    assert_eq!(g.player(P0).lands.len(), before + 1);
}
// ======================================================== Exhibition Tidecaller
fn lib_len(g: &Game, p: PlayerId) -> usize {
    g.player(p).library.len()
}

#[test]
fn tidecaller_mills_three_for_a_cheap_spell() {
    let mut g = game();
    perm(&mut g, P0, "Exhibition Tidecaller");
    let before = lib_len(&g, P1);
    cast_free(&mut g, P0, "Opt"); // mana value 1
    assert_eq!(before - lib_len(&g, P1), 3, "target player mills three");
}

#[test]
fn tidecaller_mills_ten_when_five_mana_was_spent() {
    let mut g = game();
    perm(&mut g, P0, "Exhibition Tidecaller");
    let before = lib_len(&g, P1);
    cast_free(&mut g, P0, "Dark Petition"); // mana value 5
    assert_eq!(before - lib_len(&g, P1), 10);
}

#[test]
fn tidecaller_ignores_creature_spells_and_opposing_spells() {
    let mut g = game();
    perm(&mut g, P0, "Exhibition Tidecaller");
    let (a, b) = (lib_len(&g, P0), lib_len(&g, P1));
    cast_free(&mut g, P0, "Drift of Phantasms");
    cast_free(&mut g, P1, "Opt");
    assert_eq!(lib_len(&g, P0), a);
    assert!(b - lib_len(&g, P1) < 3, "only Opt's own scry/draw touched the library");
}

// ======================================================== Voice of Victory
#[test]
fn voice_of_victory_mobilizes_two_warriors_that_go_at_the_end_step() {
    use sim_core::engine::combat;
    use sim_core::state::DataKey;
    let mut g = game();
    let v = perm(&mut g, P0, "Voice of Victory");
    g.atk_targets = vec![(v, P1)];
    g.perm_mut(v).tapped = true;
    g.in_combat = vec![v];
    let new = combat::attack_triggers(&mut g, P0, &[v], P1).unwrap();
    assert_eq!(new.len(), 2, "mobilize 2");
    for &w in &new {
        assert!(g.perm(w).token && g.perm(w).tapped && g.perm(w).warrior);
        assert_eq!((g.perm(w).pow, g.perm(w).tgh), (1, 1));
        assert!(g.perm(w).data.truthy(DataKey::EotSac), "sacrificed at the next end step");
        assert_eq!(g.atk_targets.iter().find(|x| x.0 == w).map(|x| x.1), Some(P1));
    }
}

#[test]
fn voice_of_victory_stops_opponents_casting_on_its_controllers_turn_only() {
    let mut g = game();
    perm(&mut g, P0, "Voice of Victory");
    let bolt = hand(&mut g, P1, &["Lightning Bolt"])[0];
    let opt = hand(&mut g, P0, &["Opt"])[0];
    g.active = Some(P0);
    assert!(!cast::castable(&g, P1, bolt, "hand"), "opponents can't cast spells during my turn");
    assert!(cast::castable(&g, P0, opt, "hand"), "I can");
    g.active = Some(P1);
    assert!(cast::castable(&g, P1, bolt, "hand"), "on their own turn they can");
}

// ======================================================== High Fae Trickster
#[test]
fn high_fae_trickster_grants_flash_to_every_spell_of_its_controller() {
    let mut g = game();
    assert!(!t6::static_flash(&g, P0, true) && !t6::static_flash(&g, P0, false));
    let t = perm(&mut g, P0, "High Fae Trickster");
    assert!(t6::static_flash(&g, P0, true), "creature spells too");
    assert!(t6::static_flash(&g, P0, false));
    assert!(!t6::static_flash(&g, P1, false), "only its controller");
    g.perm_mut(t).phased = true;
    assert!(!t6::static_flash(&g, P0, false), "not while phased out");
}

// ======================================================== Drift of Phantasms
#[test]
fn drift_of_phantasms_transmutes_for_a_card_of_the_same_mana_value() {
    let mut g = game();
    g.player_mut(P0).library.clear();
    lands(&mut g, P0, "Island", 3, false);
    let c = hand(&mut g, P0, &["Drift of Phantasms"])[0];
    let three = lib_with(&mut g, P0, "Mnemonic Betrayal"); // mana value 3
    let two = lib_with(&mut g, P0, "Brainstorm"); // mana value 1: not a match
    let f = g.registry.get(c).and_then(|i| i.hand_options).expect("transmute option");
    let opts = f(&mut g, c, P0, Some(false)).unwrap();
    assert_eq!(opts.len(), 1);
    let n = g.player(P0).hand.len();
    assert!(sim_core::ai::act::perform(&mut g, P0, opts[0].act.as_ref().unwrap()).unwrap());
    assert!(in_gy(&g, P0, "Drift of Phantasms"), "discarded");
    assert!(g.player(P0).hand.contains(&three), "found the mana value 3 card");
    assert!(!g.player(P0).hand.contains(&two));
    assert_eq!(g.player(P0).hand.len(), n, "one out, one in");
    assert_eq!(g.player(P0).lands.iter().filter(|&&l| !g.land(l).tapped).count(), 0, "{{1}}{{U}}{{U}} paid");
}

#[test]
fn drift_of_phantasms_transmutes_only_at_sorcery_speed() {
    let mut g = game();
    lands(&mut g, P0, "Island", 3, false);
    let c = hand(&mut g, P0, &["Drift of Phantasms"])[0];
    lib_with(&mut g, P0, "Mnemonic Betrayal");
    let f = g.registry.get(c).and_then(|i| i.hand_options).unwrap();
    assert!(f(&mut g, c, P0, None).unwrap().is_empty(), "not in the end-of-turn window");
    g.active = Some(P1);
    assert!(f(&mut g, c, P0, Some(false)).unwrap().is_empty(), "not on the opponent's turn");
}
