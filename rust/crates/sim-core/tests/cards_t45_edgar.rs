//! The Tier 4 edgar-markov-mardu-vampires cards (impls/t45_edgar.rs and the compiled patterns under
//! `// --- Tiers 4/5 (edgar)`) on hand-built positions. Each expectation follows the card's Oracle text.

use sim_core::dsl::{Fired, fire, has_kw};
use sim_core::engine::hooks::fire_trigger;
use sim_core::engine::values::{epow, etgh, has_type};
use sim_core::engine::{cast, life, mana, zones};
use sim_core::hooks::{Action, Call, Event, Opt};
use sim_core::ids::{CardId, LandId, PermId, PlayerId};
use sim_core::impls::{combos, t45_edgar};
use sim_core::state::{Ctx, DataKey, Game};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);
const EDGAR: &str = "edgar-markov-mardu-vampires";

fn edgar() -> Game {
    let mut g = table(&[EDGAR, "seph", "sauron"]);
    g.active = Some(P0);
    g
}

fn opp_life(g: &Game) -> i32 {
    g.player(P1).life + g.player(P2).life
}

fn vampires(g: &Game, p: PlayerId) -> Vec<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).on_bf && g.is_creature(m) && has_type(g, m, "vampire"))
        .collect()
}

/// a 1/1 black Vampire token, not summoning sick
fn vamp(g: &mut Game, p: PlayerId) -> PermId {
    let spec = zones::Tokens { sick: false, types: vec!["vampire"], ..zones::Tokens::new(1, 1) };
    zones::make_tokens(g, p, spec).unwrap()[0]
}

fn opts(g: &mut Game, src: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.perm(src).cd.unwrap()).unwrap().options.unwrap();
    f(g, src, p, post).unwrap()
}

fn land_opts(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.land(l).card()).unwrap().land_options.unwrap();
    f(g, l, p, post).unwrap()
}

fn hand_opts(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(c).unwrap().hand_options.unwrap();
    f(g, c, p, post).unwrap()
}

fn run(g: &mut Game, p: PlayerId, o: &Opt) -> bool {
    match o.act.clone().unwrap() {
        Action::Ability { src, f, arg } => f(g, src, p, arg).unwrap(),
        Action::Plan { f, arg } => f(g, p, arg).unwrap(),
        other => panic!("unexpected action {other:?}"),
    }
}

fn resolve(g: &mut Game, p: PlayerId, name: &str, ctx: Ctx) {
    let c = take(g, p, name);
    cast::resolve(g, p, c, &ctx, "hand").unwrap();
}

fn end_step(g: &mut Game, p: PlayerId) {
    fire_trigger(g, Event::EndStep, Call::Player { p }).unwrap();
}

fn upkeep(g: &mut Game, p: PlayerId) {
    fire_trigger(g, Event::Upkeep, Call::Player { p }).unwrap();
}

// ------------------------------------------------------------------ Edgar Markov
#[test]
fn edgar_eminence_makes_a_vampire_from_the_command_zone_and_the_battlefield() {
    let mut g = edgar();
    assert!(g.player(P0).cmd_in_zone, "Edgar starts in the command zone");
    let seer = card(&g, "Viscera Seer");
    t45_edgar::eminence(&mut g, P0, seer).unwrap();
    assert_eq!(vampires(&g, P0).len(), 1, "a Vampire spell: a 1/1 Vampire");
    let t = vampires(&g, P0)[0];
    assert!(g.perm(t).token && epow(&g, t) == 1 && etgh(&g, t) == 1);
    let ring = card(&g, "Sol Ring");
    t45_edgar::eminence(&mut g, P0, ring).unwrap();
    assert_eq!(vampires(&g, P0).len(), 1, "not a Vampire spell");
    g.player_mut(P0).cmd_in_zone = false;
    t45_edgar::eminence(&mut g, P0, seer).unwrap();
    assert_eq!(vampires(&g, P0).len(), 1, "Edgar neither in the command zone nor on the battlefield");
    perm(&mut g, P0, "Edgar Markov");
    let n = vampires(&g, P0).len();
    t45_edgar::eminence(&mut g, P0, seer).unwrap();
    assert_eq!(vampires(&g, P0).len(), n + 1, "Edgar on the battlefield");
    let ed = card(&g, "Edgar Markov");
    t45_edgar::eminence(&mut g, P0, ed).unwrap();
    assert_eq!(vampires(&g, P0).len(), n + 1, "another Vampire spell only");
}

#[test]
fn edgar_attack_and_cordial_vampire_counter_every_vampire() {
    let mut g = edgar();
    let e = perm(&mut g, P0, "Edgar Markov");
    let v = vamp(&mut g, P0);
    let other = token(&mut g, P0, 2); // not a Vampire
    fire(&mut g, "attack", Fired { player: Some(P0), attackers: vec![e], defender: Some(P1), ..Default::default() })
        .unwrap();
    assert_eq!((g.perm(e).plus, g.perm(v).plus, g.perm(other).plus), (1, 1, 0), "each Vampire you control");
    let c = perm(&mut g, P0, "Cordial Vampire");
    let x = token(&mut g, P1, 1);
    zones::die(&mut g, x, "destroy").unwrap();
    assert_eq!((g.perm(v).plus, g.perm(c).plus, g.perm(other).plus), (2, 1, 0), "another creature died");
}

#[test]
fn indulgent_aristocrat_and_drana_compile_their_counters() {
    let g = edgar();
    let a = format!("{:?}", g.db.get(card(&g, "Indulgent Aristocrat")).abilities);
    assert!(a.contains("Activated") && a.contains("sac: Some(\"creature\")") && a.contains("vampire"), "{a}");
    let d = format!("{:?}", g.db.get(card(&g, "Drana, Liberator of Malakir")).abilities);
    assert!(d.contains("combat_damage") && d.contains("attacking: Some(true)"), "{d}");
}

#[test]
fn malakir_bloodwitch_drains_per_vampire() {
    let mut g = edgar();
    vamp(&mut g, P0);
    vamp(&mut g, P0);
    let before = opp_life(&g);
    perm(&mut g, P0, "Malakir Bloodwitch");
    assert_eq!(before - opp_life(&g), 2 * 3, "each opponent loses 3: two tokens and itself");
    assert_eq!(g.player(P0).life, 46, "you gain the life lost");
}

// ------------------------------------------------------------------ Edgar, Charmed Groom // Edgar Markov's Coffin
#[test]
fn charmed_groom_pumps_returns_as_the_coffin_and_transforms_back() {
    let mut g = edgar();
    let v = vamp(&mut g, P0);
    let gr = perm(&mut g, P0, "Edgar, Charmed Groom // Edgar Markov's Coffin");
    assert_eq!((epow(&g, v), epow(&g, gr)), (2, 4), "other Vampires +1/+1");
    zones::die(&mut g, gr, "destroy").unwrap();
    let cof = *g.player(P0).perms.iter().find(|&&m| g.perm(m).on_bf && &*g.perm(m).name != "Token").unwrap();
    assert!(!g.is_creature(cof), "it returns as Edgar Markov's Coffin, not a creature");
    assert_eq!(epow(&g, v), 1, "the Coffin has no anthem");
    let n = vampires(&g, P0).len();
    upkeep(&mut g, P0);
    upkeep(&mut g, P0);
    assert_eq!(vampires(&g, P0).len(), n + 2, "a 1/1 Vampire each upkeep");
    let t = *vampires(&g, P0).last().unwrap();
    assert!(g.perm(t).lifelink, "with lifelink");
    assert!(!g.is_creature(cof));
    upkeep(&mut g, P0);
    assert!(g.is_creature(cof), "the third bloodline counter transforms it back");
    assert_eq!(epow(&g, v), 2);
}

// ------------------------------------------------------------------ the drain cards
#[test]
fn bloodletter_doubles_opponents_life_loss_on_your_turn() {
    let mut g = edgar();
    perm(&mut g, P0, "Bloodletter of Aclazotz");
    life::lose_life(&mut g, P1, 3, Some(P0), "drain", None).unwrap();
    assert_eq!(g.player(P1).life, 34, "twice 3");
    g.active = Some(P1);
    life::lose_life(&mut g, P2, 3, Some(P0), "drain", None).unwrap();
    assert_eq!(g.player(P2).life, 37, "not your turn");
    g.active = Some(P0);
    life::lose_life(&mut g, P0, 3, Some(P0), "life", None).unwrap();
    assert_eq!(g.player(P0).life, 37, "you are not your opponent");
}

#[test]
fn bloodthirsty_conqueror_and_vito_turn_life_changes_around() {
    let mut g = edgar();
    perm(&mut g, P0, "Bloodthirsty Conqueror");
    life::lose_life(&mut g, P1, 3, Some(P0), "drain", None).unwrap();
    assert_eq!(g.player(P0).life, 43, "you gain that much");
    let mut g = edgar();
    perm(&mut g, P0, "Vito, Thorn of the Dusk Rose");
    let before = opp_life(&g);
    life::gain(&mut g, P0, 2).unwrap();
    assert_eq!(before - opp_life(&g), 2, "target opponent loses that much");
}

#[test]
fn the_drain_loops_are_in_the_combo_catalogue() {
    let i = combos::combo_index("Bloodthirsty Conqueror").unwrap();
    let mut g = edgar();
    assert!(!(combos::all()[i].ready)(&g, P0).0);
    perm(&mut g, P0, "Vito, Thorn of the Dusk Rose");
    assert!(!(combos::all()[i].ready)(&g, P0).0, "one half only");
    hand(&mut g, P0, &["Exquisite Blood"]);
    let (ok, keys, cast) = (combos::all()[i].ready)(&g, P0);
    assert!(ok && keys.len() == 1 && cast.len() == 1, "Vito out, Exquisite Blood to cast");
    let mut g = edgar();
    perm(&mut g, P0, "Sanguine Bond");
    perm(&mut g, P0, "Bloodthirsty Conqueror");
    assert!((combos::all()[i].ready)(&g, P0).0, "Sanguine Bond + Bloodthirsty Conqueror");
    let mut g = edgar();
    perm(&mut g, P0, "Sanguine Bond");
    perm(&mut g, P0, "Exquisite Blood");
    assert!(!(combos::all()[i].ready)(&g, P0).0, "the plain pair is combos.rs's own entry");
}

#[test]
fn knight_of_the_ebon_legion_grows_after_a_big_hit() {
    let mut g = edgar();
    let k = perm(&mut g, P0, "Knight of the Ebon Legion");
    life::lose_life(&mut g, P1, 3, Some(P0), "drain", None).unwrap();
    end_step(&mut g, P0);
    assert_eq!(g.perm(k).plus, 0, "3 life");
    life::lose_life(&mut g, P2, 4, Some(P0), "drain", None).unwrap();
    end_step(&mut g, P0);
    assert_eq!(g.perm(k).plus, 1, "a player lost 4 or more this turn");
}

#[test]
fn twilight_prophet_reveals_and_drains_with_the_blessing() {
    let mut g = edgar();
    let tp = perm(&mut g, P0, "Twilight Prophet");
    let top = take(&mut g, P0, "Bolas's Citadel");
    g.player_mut(P0).library.push(top);
    let before = opp_life(&g);
    upkeep(&mut g, P0);
    assert_eq!(opp_life(&g), before, "no city's blessing yet");
    lands(&mut g, P0, "Swamp", 9, false);
    upkeep(&mut g, P0);
    assert!(g.perm(tp).data.truthy(DataKey::Blessing));
    assert!(g.player(P0).hand.contains(&top), "the top card to hand");
    assert_eq!(before - opp_life(&g), 2 * 6, "each opponent loses 6 (its mana value)");
    assert_eq!(g.player(P0).life, 46, "you gain 6");
}

// ------------------------------------------------------------------ counters
#[test]
fn blade_of_the_bloodchief_counts_deaths_double_on_a_vampire() {
    let mut g = edgar();
    let b = perm(&mut g, P0, "Blade of the Bloodchief");
    let v = vamp(&mut g, P0);
    let h = token(&mut g, P0, 2);
    g.perm_mut(b).attached = Some(v);
    let x = token(&mut g, P1, 1);
    zones::die(&mut g, x, "destroy").unwrap();
    assert_eq!(g.perm(v).plus, 2, "two on a Vampire");
    g.perm_mut(b).attached = Some(h);
    let x = token(&mut g, P1, 1);
    zones::die(&mut g, x, "destroy").unwrap();
    assert_eq!(g.perm(h).plus, 1, "one on another creature");
}

#[test]
fn vampire_socialite_counters_once_an_opponent_lost_life() {
    let mut g = edgar();
    let v = vamp(&mut g, P0);
    perm(&mut g, P0, "Vampire Socialite");
    assert_eq!(g.perm(v).plus, 0, "no opponent lost life");
    let mut g = edgar();
    let v = vamp(&mut g, P0);
    life::lose_life(&mut g, P1, 1, Some(P0), "drain", None).unwrap();
    let s = perm(&mut g, P0, "Vampire Socialite");
    assert_eq!((g.perm(v).plus, g.perm(s).plus), (1, 0), "each other Vampire");
    let w = vamp(&mut g, P0);
    assert_eq!(g.perm(w).plus, 1, "an additional counter as it enters");
    let k = perm(&mut g, P0, "Knight of the Ebon Legion");
    assert_eq!(g.perm(k).plus, 1, "cards too");
}

#[test]
fn forerunner_stacks_a_vampire_and_pumps_on_vampires_entering() {
    let mut g = edgar();
    let f = perm(&mut g, P0, "Forerunner of the Legion");
    let top = *g.player(P0).library.last().unwrap();
    let d = g.db.get(top);
    assert!(d.has_subtype("vampire") || d.has_kw("changeling"), "a Vampire card on top ({})", d.name);
    let before = epow(&g, f);
    vamp(&mut g, P0);
    assert_eq!(epow(&g, f), before + 1, "target creature gets +1/+1 (your best, the Forerunner)");
}

// ------------------------------------------------------------------ steals, wipes
#[test]
fn captivating_vampire_taps_five_to_steal() {
    let mut g = edgar();
    let c = perm(&mut g, P0, "Captivating Vampire");
    for _ in 0..3 {
        vamp(&mut g, P0);
    }
    let big = perm(&mut g, P1, "Bloodthirsty Conqueror");
    assert!(opts(&mut g, c, P0, None).is_empty(), "four Vampires");
    vamp(&mut g, P0);
    let o = opts(&mut g, c, P0, None);
    assert_eq!(o.len(), 1, "{:?}", o.iter().map(|x| &x.label).collect::<Vec<_>>());
    assert!(run(&mut g, P0, &o[0]));
    assert_eq!(g.perm(big).owner, P0, "gained control");
    assert!(has_type(&g, big, "vampire"));
    assert_eq!(vampires(&g, P0).iter().filter(|&&m| g.perm(m).tapped).count(), 5, "five tapped");
}

#[test]
fn new_blood_taps_a_vampire_and_steals() {
    let mut g = edgar();
    let c = hand(&mut g, P0, &["New Blood"])[0];
    let big = perm(&mut g, P1, "Bloodthirsty Conqueror");
    assert!(!cast::additional_cost(&mut g, P0, c, true).unwrap(), "no untapped Vampire");
    let v = vamp(&mut g, P0);
    assert!(cast::additional_cost(&mut g, P0, c, true).unwrap());
    cast::additional_cost(&mut g, P0, c, false).unwrap();
    assert!(g.perm(v).tapped, "the Vampire is tapped as the cost");
    g.player_mut(P0).hand.clear();
    resolve(&mut g, P0, "New Blood", Ctx { target: Some(big), ..Ctx::default() });
    assert_eq!(g.perm(big).owner, P0);
    assert!(has_type(&g, big, "vampire"));
}

#[test]
fn olivias_wrath_spares_vampires() {
    let mut g = edgar();
    vamp(&mut g, P0);
    vamp(&mut g, P0);
    let mine = token(&mut g, P0, 3);
    let small = token(&mut g, P1, 2);
    let theirs = vamp(&mut g, P1);
    resolve(&mut g, P0, "Olivia's Wrath", Ctx::default());
    assert!(!g.perm(small).on_bf, "a non-Vampire 2/2 gets -2/-2");
    assert!(g.perm(mine).on_bf && etgh(&g, mine) == 1, "a 3/3 survives as a 1/1");
    assert!(g.perm(theirs).on_bf, "Vampires are spared, anyone's");
}

#[test]
fn damn_destroys_a_target_or_overloaded_every_creature() {
    let mut g = edgar();
    let a = token(&mut g, P1, 3);
    let b = token(&mut g, P2, 3);
    resolve(&mut g, P0, "Damn", Ctx { target: Some(a), ..Ctx::default() });
    assert!(!g.perm(a).on_bf && g.perm(b).on_bf, "destroy target creature");
    let mut g = edgar();
    let c = hand(&mut g, P0, &["Damn"])[0];
    lands(&mut g, P0, "Plains", 4, false);
    let theirs: Vec<PermId> = (0..4).map(|_| perm(&mut g, P1, "Bloodthirsty Conqueror")).collect();
    let mine = token(&mut g, P0, 1);
    let o = hand_opts(&mut g, c, P0, Some(false));
    assert_eq!(o.len(), 1, "the overload is worth it");
    assert!(run(&mut g, P0, &o[0]));
    assert!(theirs.iter().all(|&m| !g.perm(m).on_bf) && !g.perm(mine).on_bf, "each creature");
}

// ------------------------------------------------------------------ Sorin, Emeritus, Markov Baron, Clavileño
#[test]
fn sorin_three_abilities() {
    let mut g = edgar();
    let s = perm(&mut g, P0, "Sorin, Imperious Bloodlord");
    let v = vamp(&mut g, P0);
    let o = opts(&mut g, s, P0, Some(false));
    let first = o.iter().find(|x| matches!(x.act, Some(Action::Ability { arg: 0, .. }))).unwrap().clone();
    assert!(run(&mut g, P0, &first));
    assert!(has_kw(&g, v, "deathtouch") && has_kw(&g, v, "lifelink") && g.perm(v).plus == 1);
    let mut g = edgar();
    let s = perm(&mut g, P0, "Sorin, Imperious Bloodlord");
    let v = vamp(&mut g, P0);
    let t = perm(&mut g, P1, "Knight of the Ebon Legion");
    let o = opts(&mut g, s, P0, Some(false));
    let second = o.iter().find(|x| matches!(x.act, Some(Action::Ability { arg: 1, .. }))).unwrap().clone();
    assert!(run(&mut g, P0, &second));
    assert!(!g.perm(v).on_bf && !g.perm(t).on_bf, "sacrifice a Vampire: 3 damage kills the Knight");
    assert_eq!(g.player(P0).life, 43, "gain 3 life");
    let mut g = edgar();
    let s = perm(&mut g, P0, "Sorin, Imperious Bloodlord");
    g.perm_mut(s).loyalty = Some(4);
    hand(&mut g, P0, &["Malakir Bloodwitch"]);
    let o = opts(&mut g, s, P0, Some(false));
    let third = o.iter().find(|x| matches!(x.act, Some(Action::Ability { arg: 2, .. }))).unwrap().clone();
    assert!(run(&mut g, P0, &third));
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).on_bf && &*g.perm(m).name == "Malakir Bloodwitch"));
}

#[test]
fn emeritus_of_woe_casts_a_prepared_demonic_tutor() {
    let mut g = edgar();
    let e = perm(&mut g, P0, "Emeritus of Woe // Demonic Tutor");
    assert!(g.perm(e).data.truthy(DataKey::Prepared), "enters prepared");
    lands(&mut g, P0, "Swamp", 2, false);
    let n = g.player(P0).hand.len();
    let o = opts(&mut g, e, P0, Some(false));
    assert_eq!(o.len(), 1);
    assert!(run(&mut g, P0, &o[0]));
    assert_eq!(g.player(P0).hand.len(), n + 1, "Demonic Tutor finds a card");
    assert!(!g.perm(e).data.truthy(DataKey::Prepared));
    for _ in 0..2 {
        let x = token(&mut g, P1, 1);
        zones::die(&mut g, x, "destroy").unwrap();
    }
    end_step(&mut g, P0);
    assert!(g.perm(e).data.truthy(DataKey::Prepared), "two creatures died this turn");
}

#[test]
fn markov_baron_is_cast_for_madness() {
    let mut g = edgar();
    lands(&mut g, P0, "Swamp", 3, false);
    let c = take(&mut g, P0, "Markov Baron");
    g.player_mut(P0).gy.push(c);
    t45_edgar::madness(&mut g, P0, &[c]).unwrap();
    assert!(g.player(P0).perms.iter().any(|&m| g.perm(m).on_bf && g.perm(m).cd == Some(c)), "cast for {{2}}{{B}}");
}

#[test]
fn clavileno_makes_an_attacker_a_demon_that_pays_off_on_death() {
    let mut g = edgar();
    perm(&mut g, P0, "Clavileño, First of the Blessed");
    let v = vamp(&mut g, P0);
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![v], d: P1 }).unwrap();
    assert!(has_type(&g, v, "demon") && g.perm(v).data.truthy(DataKey::DemonGift));
    let n = g.player(P0).hand.len();
    zones::die(&mut g, v, "destroy").unwrap();
    assert_eq!(g.player(P0).hand.len(), n + 1, "draw a card");
    let t = *g.player(P0).perms.last().unwrap();
    assert!(g.perm(t).tapped && epow(&g, t) == 4 && etgh(&g, t) == 3 && g.perm(t).fly, "a tapped 4/3 flyer");
    assert!(has_type(&g, t, "vampire") && has_type(&g, t, "demon"));
}

// ------------------------------------------------------------------ Voldaren Estate
#[test]
fn voldaren_estate_colours_only_vampire_spells_and_makes_blood() {
    let mut g = edgar();
    let l = lands(&mut g, P0, "Voldaren Estate", 1, false)[0];
    assert!(mana::land_colors_now(&g, P0, l).count() == 0, "{{C}} for anything else");
    g.pay_for = Some(card(&g, "Malakir Bloodwitch"));
    assert!(mana::land_colors_now(&g, P0, l).count() > 0, "any colour for a Vampire spell");
    g.pay_for = None;
    lands(&mut g, P0, "Swamp", 2, false);
    for _ in 0..3 {
        vamp(&mut g, P0);
    }
    let o = land_opts(&mut g, l, P0, None);
    let blood = o.iter().find(|x| x.label.contains("Blood token")).expect("{2} with three Vampires").clone();
    assert!(run(&mut g, P0, &blood));
    assert_eq!(g.player(P0).bloods, 1);
    assert!(g.land(l).tapped);
}
