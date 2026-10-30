//! The Tier 5 winota-boros-humans-cheat cards (impls/t45_winota.rs and the hooks marked `// --- Tiers 4/5 (winota)`)
//! on hand-built positions. Each expectation follows the card's Oracle text.

use sim_core::ai::pool;
use sim_core::engine::cast::{self, castable};
use sim_core::engine::combat::can_block;
use sim_core::engine::repl::trigger_copies as total_trigger_copies;
use sim_core::engine::life::lose_life;
use sim_core::engine::mana::land_colors_now;
use sim_core::engine::removal::apply_removal;
use sim_core::engine::values::{epow, etgh, no_sac};
use sim_core::engine::zones;
use sim_core::hooks::{Action, Opt};
use sim_core::ids::{PermId, PlayerId};
use sim_core::impls::t45_winota as w;
use sim_core::state::{Ctx, DataKey, Game, Step, Val};
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);
const P2: PlayerId = PlayerId(2);

fn tbl() -> Game {
    table(&[
        "winota-boros-humans-cheat",
        "urza-mono-blue-artifacts",
        "kinnan-simic-mana-combo",
        "yawgmoth-mono-black-aristocrats",
    ])
}

fn my_turn(g: &mut Game, p: PlayerId, step: Step) {
    g.active = Some(p);
    g.step = step;
}

fn run(g: &mut Game, o: Opt, p: PlayerId) -> bool {
    match o.act.unwrap() {
        Action::Plan { f, arg } => f(g, p, arg).unwrap(),
        Action::Ability { src, f, arg } => f(g, src, p, arg).unwrap(),
        other => panic!("unexpected action {other:?}"),
    }
}

fn perm_opts(g: &mut Game, m: PermId, p: PlayerId, post: Option<bool>) -> Vec<Opt> {
    let f = g.registry.get(g.perm(m).cd.unwrap()).and_then(|i| i.options).expect("options");
    f(g, m, p, post).unwrap()
}

fn on(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

fn lose_all_cards(g: &mut Game, p: PlayerId) {
    g.player_mut(p).hand.clear();
}

// ------------------------------------------------------------------ Breath of Fury
#[test]
fn breath_of_fury_sacrifices_the_creature_moves_and_gives_a_combat() {
    let mut g = tbl();
    my_turn(&mut g, P0, Step::Combat);
    let orni = perm(&mut g, P0, "Ornithopter");
    let pest = perm(&mut g, P0, "Signal Pest");
    let b = perm(&mut g, P0, "Breath of Fury");
    g.perm_mut(b).attached = Some(orni);
    g.perm_mut(orni).tapped = true;
    g.perm_mut(pest).tapped = true;
    g.player_mut(P0).combat_no = 1;
    let f = g.registry.get(card(&g, "Breath of Fury")).unwrap().combat_damage.unwrap();
    f(&mut g, b, P0, orni, P1, 1).unwrap();
    assert!(!on(&g, orni), "the enchanted creature is sacrificed");
    assert_eq!(g.perm(b).attached, Some(pest), "the Aura moves to another creature you control");
    assert!(!g.perm(pest).tapped, "your creatures untap");
    assert_eq!(g.player(P0).extra_combats, 1, "an additional combat");
    f(&mut g, b, P0, pest, P1, 1).unwrap();
    assert!(on(&g, pest), "once per combat: the Aura moved during this combat's damage");
    // the next combat: nothing to move to, so no untap and no combat
    g.player_mut(P0).combat_no = 2;
    f(&mut g, b, P0, pest, P1, 1).unwrap();
    assert!(!on(&g, pest) && !on(&g, b), "sacrificed, and the Aura goes with it");
    assert_eq!(g.player(P0).extra_combats, 1);
    // another player's creature dealing damage does nothing
    let o = perm(&mut g, P1, "Ornithopter");
    let b2 = perm(&mut g, P0, "Breath of Fury");
    let m = perm(&mut g, P0, "Memnite");
    g.perm_mut(b2).attached = Some(m);
    f(&mut g, b2, P1, o, P0, 1).unwrap();
    assert!(on(&g, o) && on(&g, m));
}

// ------------------------------------------------------------------ Flare of Fortitude
#[test]
fn flare_of_fortitude_locks_life_and_protects_permanents_this_turn() {
    let mut g = tbl();
    lose_all_cards(&mut g, P1);
    lands(&mut g, P1, "Plains", 4, false);
    let m = perm(&mut g, P1, "Ornithopter");
    hand(&mut g, P1, &["Flare of Fortitude"]);
    assert!(w::flare_last_chance(&mut g, P1).unwrap(), "cast against lethal damage");
    assert!(g.player(P1).life_locked);
    let life = g.player(P1).life;
    lose_life(&mut g, P1, 50, Some(P0), "combat", None).unwrap();
    assert_eq!(g.player(P1).life, life, "life total can't change");
    assert!(sim_core::dsl::has_kw(&g, m, "hexproof") && sim_core::dsl::has_kw(&g, m, "indestructible"));
    // the next turn: over
    sim_core::engine::turn::step_start(&mut g, P2).unwrap();
    assert!(!g.player(P1).life_locked, "until end of turn only");
}

#[test]
fn flare_of_fortitude_can_be_cast_by_sacrificing_a_nontoken_white_creature() {
    let mut g = tbl();
    lose_all_cards(&mut g, P0);
    let cheap = perm(&mut g, P0, "Mother of Runes");
    let win = perm(&mut g, P0, "Winota, Joiner of Forces");
    hand(&mut g, P0, &["Flare of Fortitude"]);
    assert_eq!(w::flare_fodder(&g, P0, "Flare of Fortitude", Some(win)), Some(cheap));
    assert_eq!(w::flare_fodder(&g, P0, "Flawless Maneuver", None), None, "only Flare has the cost");
    // a destroy spell at Winota: Flare answers it, paid with Mother of Runes (no lands)
    let saved = pool::protect(&mut g, P0, win, "destroy", Some(P1), None).unwrap();
    assert!(saved, "Flare protects Winota");
    assert!(!on(&g, cheap), "Mother of Runes was the cost (or the protector)");
    assert!(sim_core::engine::values::indestructible(&g, win));
}

// ------------------------------------------------------------------ High Noon
#[test]
fn high_noon_allows_one_spell_a_turn_and_sacrifices_for_five_damage() {
    let mut g = tbl();
    perm(&mut g, P0, "High Noon");
    let c = hand(&mut g, P1, &["Sol Ring"])[0];
    assert!(castable(&g, P1, c, "hand"));
    let other = take(&mut g, P1, "Arcane Signet");
    cast::on_cast(&mut g, P1, other).unwrap();
    assert!(!castable(&g, P1, c, "hand"), "a second spell this turn is not allowed");
    let mine = take(&mut g, P0, "Sol Ring");
    assert!(castable(&g, P0, mine, "hand"), "each player counts their own");
    // {4}{R}, sacrifice: 5 damage, lethal to a player at 5
    let mut g = tbl();
    let n = perm(&mut g, P0, "High Noon");
    lands(&mut g, P0, "Mountain", 5, false);
    assert!(perm_opts(&mut g, n, P0, Some(false)).is_empty(), "nothing worth 5 damage");
    g.player_mut(P1).life = 5;
    let o = perm_opts(&mut g, n, P0, Some(false)).pop().expect("lethal");
    assert!(run(&mut g, o, P0));
    assert!(!on(&g, n), "sacrificed");
    assert!(g.player(P1).life <= 0 || !g.player(P1).alive);
}

// ------------------------------------------------------------------ Windcrag Siege
#[test]
fn windcrag_siege_picks_mardu_for_attack_triggers_and_jeskai_otherwise() {
    let mut g = tbl();
    let s = perm(&mut g, P0, "Windcrag Siege");
    assert_eq!(g.perm(s).data.get(DataKey::Mode), Some(&Val::Str("mardu")), "Winota's deck: Mardu");
    assert_eq!(total_trigger_copies(&g, P0, "attack", None), 1, "attack triggers trigger an additional time");
    assert_eq!(total_trigger_copies(&g, P0, "etb", None), 0);
    assert_eq!(total_trigger_copies(&g, P1, "attack", None), 0, "only yours");
    // Jeskai: a hasty lifelink Goblin at your upkeep
    let j = perm(&mut g, P2, "Windcrag Siege");
    assert_eq!(g.perm(j).data.get(DataKey::Mode), Some(&Val::Str("jeskai")), "Kinnan: no attack triggers");
    let f = g.registry.get(card(&g, "Windcrag Siege")).unwrap().upkeep.unwrap();
    let before = g.player(P2).perms.len();
    f(&mut g, j, P2).unwrap();
    let t = *g.player(P2).perms.last().unwrap();
    assert_eq!(g.player(P2).perms.len(), before + 1);
    assert!(g.perm(t).token && epow(&g, t) == 1);
    assert!(sim_core::dsl::has_kw(&g, t, "haste") && sim_core::dsl::has_kw(&g, t, "lifelink"));
    f(&mut g, j, P1).unwrap();
    assert_eq!(g.player(P2).perms.len(), before + 1, "your upkeep only");
    f(&mut g, s, P0).unwrap();
    assert!(!g.player(P0).perms.iter().any(|&m| g.perm(m).token), "Mardu makes no Goblin");
}

// ------------------------------------------------------------------ Alexios, Deimos of Kosmos
#[test]
fn alexios_changes_hands_each_upkeep_grows_and_cant_attack_its_owner() {
    let mut g = tbl();
    let a = perm(&mut g, P0, "Alexios, Deimos of Kosmos");
    assert!(no_sac(&g, a), "can't be sacrificed");
    zones::die(&mut g, a, "sac").unwrap();
    assert!(on(&g, a));
    g.perm_mut(a).tapped = true;
    let f = g.registry.get(card(&g, "Alexios, Deimos of Kosmos")).unwrap().upkeep.unwrap();
    my_turn(&mut g, P1, Step::Start);
    f(&mut g, a, P1).unwrap();
    assert_eq!(g.perm(a).owner, P1, "the upkeep's player gains control");
    assert!(!g.perm(a).tapped && g.perm(a).plus == 1, "untapped, a +1/+1 counter");
    assert!(sim_core::engine::combat::has_haste(&g, a), "haste this turn");
    assert!(g.perm(a).data.get(DataKey::MustAttack).is_some(), "attacks each combat");
    assert_eq!(sim_core::impls::t0_zurgo::goaded_by(&g, a), Some(P0), "its owner goads it: it attacks someone else");
    my_turn(&mut g, P0, Step::Start);
    g.player_mut(P0).turns += 1;
    f(&mut g, a, P0).unwrap();
    assert_eq!(g.perm(a).owner, P0);
    assert_eq!(g.perm(a).plus, 2);
    assert_eq!(sim_core::impls::t0_zurgo::goaded_by(&g, a), None, "with its owner it is not goaded");
}

// ------------------------------------------------------------------ Angrath's Marauders
#[test]
fn angraths_marauders_doubles_your_damage() {
    let mut g = tbl();
    perm(&mut g, P0, "Angrath's Marauders");
    let life = g.player(P1).life;
    lose_life(&mut g, P1, 3, Some(P0), "burn", None).unwrap();
    assert_eq!(g.player(P1).life, life - 6, "3 damage becomes 6");
    let life0 = g.player(P0).life;
    lose_life(&mut g, P0, 3, Some(P1), "burn", None).unwrap();
    assert_eq!(g.player(P0).life, life0 - 3, "an opponent's damage isn't doubled");
    let t = token(&mut g, P1, 3); // a 3/3
    apply_removal(&mut g, Some(P0), t, "dmg2", None).unwrap();
    assert!(!on(&g, t), "2 damage becomes 4: the 3/3 dies");
    assert_eq!(sim_core::engine::combat::fiery(&g, P0), 2);
}

// ------------------------------------------------------------------ Blinkmoth Nexus
#[test]
fn blinkmoth_nexus_animates_into_a_flier_to_attack() {
    let mut g = tbl();
    let l = lands(&mut g, P0, "Blinkmoth Nexus", 1, false)[0];
    lands(&mut g, P0, "Mountain", 2, false);
    g.player_mut(P0).land_turn = -1;
    let f = g.registry.get(card(&g, "Blinkmoth Nexus")).and_then(|i| i.land_options).expect("land options");
    let o = f(&mut g, l, P0, Some(false)).unwrap().pop().expect("animate before combat");
    assert!(run(&mut g, o, P0));
    let m = *g.player(P0).perms.last().unwrap();
    assert!(g.is_creature(m) && g.perm(m).fly && epow(&g, m) == 1 && etgh(&g, m) == 1, "a 1/1 flier");
}

// ------------------------------------------------------------------ Boromir, Warden of the Tower
#[test]
fn boromir_counters_free_spells_and_saves_creatures() {
    let mut g = tbl();
    let b = perm(&mut g, P0, "Boromir, Warden of the Tower");
    let f = g.registry.get(card(&g, "Boromir, Warden of the Tower")).unwrap().cast_counter.unwrap();
    let c = card(&g, "Sol Ring");
    let free = Ctx { mana_spent: 0, ..Ctx::default() };
    let paid = Ctx { mana_spent: 1, ..Ctx::default() };
    assert!(f(&g, b, P1, c, &free), "an opponent's spell with no mana spent is countered");
    assert!(!f(&g, b, P1, c, &paid), "mana was spent");
    assert!(!f(&g, b, P0, c, &free), "not your own spells");
    let win = perm(&mut g, P0, "Winota, Joiner of Forces");
    assert!(pool::protect(&mut g, P0, win, "destroy", Some(P1), None).unwrap(), "sacrificed to save Winota");
    assert!(!on(&g, b));
    assert!(sim_core::engine::values::indestructible(&g, win));
}

#[test]
fn boromir_compiled_sacrifice_tempts_with_the_ring() {
    let g = tbl();
    let d = g.db.get(card(&g, "Boromir, Warden of the Tower"));
    let txt = format!("{:?}", d.abilities);
    assert!(txt.contains("ring_tempt"), "the Ring tempts you is compiled: {txt}");
}

// ------------------------------------------------------------------ Gingerbrute
#[test]
fn gingerbrute_can_only_be_blocked_by_hasty_creatures() {
    let mut g = tbl();
    lands(&mut g, P0, "Mountain", 1, false);
    let gb = perm(&mut g, P0, "Gingerbrute");
    my_turn(&mut g, P0, Step::Main1);
    assert!(perm_opts(&mut g, gb, P0, Some(false)).is_empty(), "no blocker: not worth it");
    let wall = token(&mut g, P1, 2);
    let o = perm_opts(&mut g, gb, P0, Some(false)).pop().expect("evasion before combat");
    assert!(run(&mut g, o, P0));
    assert!(!can_block(&g, wall, gb), "a creature without haste can't block it");
    let hasty = token(&mut g, P1, 1);
    g.perm_mut(hasty).eot_kw.push("haste");
    assert!(can_block(&g, hasty, gb), "a creature with haste can");
}

// ------------------------------------------------------------------ Greymond, Avacyn's Stalwart
#[test]
fn greymond_grants_abilities_to_humans_and_pumps_with_four() {
    let mut g = tbl();
    let gr = perm(&mut g, P0, "Greymond, Avacyn's Stalwart");
    let t = perm(&mut g, P0, "Thalia, Guardian of Thraben");
    let orni = perm(&mut g, P0, "Ornithopter");
    assert!(sim_core::dsl::has_kw(&g, t, "first strike") && sim_core::dsl::has_kw(&g, t, "lifelink"));
    assert!(!sim_core::dsl::has_kw(&g, t, "vigilance"), "two of the three");
    assert!(!sim_core::dsl::has_kw(&g, orni, "lifelink"), "not a Human");
    let (p0, t0) = (epow(&g, t), etgh(&g, t));
    perm(&mut g, P0, "Grand Abolisher");
    assert_eq!(epow(&g, t), p0, "three Humans: no bonus");
    perm(&mut g, P0, "Drannith Magistrate");
    assert_eq!((epow(&g, t), etgh(&g, t)), (p0 + 2, t0 + 2), "four Humans: +2/+2");
    assert_eq!(epow(&g, orni), 0, "not a Human");
    let o = perm(&mut g, P1, "Grand Abolisher");
    assert!(!sim_core::dsl::has_kw(&g, o, "first strike"), "an opponent's Human gets nothing");
    let _ = gr;
}

// ------------------------------------------------------------------ Lena, Selfless Champion
#[test]
fn lena_makes_a_soldier_per_nontoken_creature_and_saves_smaller_creatures() {
    let mut g = tbl();
    perm(&mut g, P0, "Ornithopter");
    perm(&mut g, P0, "Winota, Joiner of Forces");
    for n in ["Mother of Runes", "Giver of Runes", "Thalia, Guardian of Thraben"] {
        perm(&mut g, P0, n);
    }
    token(&mut g, P0, 1);
    let tokens = |g: &Game| g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf && g.perm(m).token).count();
    let before = tokens(&g);
    let lena = perm(&mut g, P0, "Lena, Selfless Champion");
    assert_eq!(tokens(&g), before + 6, "Ornithopter, Winota, three Humans and Lena: six Soldiers");
    let big = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).name == "Winota, Joiner of Forces").unwrap();
    let small = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).name == "Ornithopter").unwrap();
    assert!(!w::lena_saves(&g, "Lena, Selfless Champion", Some(lena), big), "Winota's power is not less than Lena's");
    assert!(w::lena_saves(&g, "Lena, Selfless Champion", Some(lena), small));
    assert!(w::lena_saves(&g, "Selfless Spirit", None, big), "other protectors are unchanged");
    // a wipe: Lena is sacrificed, the small creatures live
    let saved = w::lena_wipe(&mut g, P0, "Lena, Selfless Champion", Some(lena), "destroy").unwrap();
    assert_eq!(saved, Some(true));
    assert!(!on(&g, lena));
    assert!(sim_core::engine::values::indestructible(&g, small));
    assert!(!sim_core::engine::values::indestructible(&g, big), "power 4 is not less than 3");
}

// ------------------------------------------------------------------ Myrel, Shield of Argive
#[test]
fn myrel_stops_opponents_spells_and_abilities_on_your_turn() {
    let mut g = tbl();
    perm(&mut g, P0, "Myrel, Shield of Argive");
    let c = hand(&mut g, P1, &["Sol Ring"])[0];
    let pest = perm(&mut g, P1, "Signal Pest");
    my_turn(&mut g, P0, Step::Main1);
    assert!(!castable(&g, P1, c, "hand"), "your turn: opponents can't cast spells");
    assert!(sim_core::cardcode::ability_locked(&g, pest, P1), "nor activate creatures' abilities");
    let mine = perm(&mut g, P0, "Signal Pest");
    assert!(!sim_core::cardcode::ability_locked(&g, mine, P0), "yours are fine");
    my_turn(&mut g, P1, Step::Main1);
    assert!(castable(&g, P1, c, "hand"), "their own turn");
    assert!(!sim_core::cardcode::ability_locked(&g, pest, P1));
}

// ------------------------------------------------------------------ Needleverge Pathway
#[test]
fn needleverge_pathway_is_played_as_the_colour_the_hand_needs() {
    let mut g = tbl();
    lose_all_cards(&mut g, P0);
    lands(&mut g, P0, "Mountain", 2, false);
    hand(&mut g, P0, &["Swords to Plowshares", "Mother of Runes"]);
    let c = take(&mut g, P0, "Needleverge Pathway // Pillarverge Pathway");
    sim_core::engine::turn::play_land_card(&mut g, P0, c, "plays").unwrap();
    let l = *g.player(P0).lands.last().unwrap();
    let cols = land_colors_now(&g, P0, l);
    assert!(cols.has('W') && !cols.has('R'), "white needed, red already made: the {{W}} face");
    let mut g = tbl();
    lose_all_cards(&mut g, P0);
    lands(&mut g, P0, "Plains", 2, false);
    hand(&mut g, P0, &["Lightning Bolt"]);
    let c = take(&mut g, P0, "Needleverge Pathway // Pillarverge Pathway");
    sim_core::engine::turn::play_land_card(&mut g, P0, c, "plays").unwrap();
    let l = *g.player(P0).lands.last().unwrap();
    let cols = land_colors_now(&g, P0, l);
    assert!(cols.has('R') && !cols.has('W'), "the {{R}} face");
}

// ------------------------------------------------------------------ Phoenix Chick
#[test]
fn phoenix_chick_returns_attacking_when_three_attack() {
    let mut g = tbl();
    my_turn(&mut g, P0, Step::Combat);
    let c = take(&mut g, P0, "Phoenix Chick");
    g.player_mut(P0).gy.push(c);
    lands(&mut g, P0, "Mountain", 2, false);
    let atk: Vec<PermId> = (0..3).map(|_| token(&mut g, P0, 1)).collect();
    assert!(w::gy_attack(&mut g, P0, &atk[..2]).unwrap().is_empty(), "two attackers: no");
    let back = w::gy_attack(&mut g, P0, &atk).unwrap();
    assert_eq!(back.len(), 1, "three attackers and {{R}}{{R}}: it returns");
    let m = back[0];
    assert!(g.perm(m).tapped && g.perm(m).plus == 1, "tapped and attacking with a +1/+1 counter");
    assert!(!g.player(P0).gy.contains(&c));
}

// ------------------------------------------------------------------ Phyrexian Revoker
#[test]
fn phyrexian_revoker_stops_the_named_cards_abilities() {
    let mut g = tbl();
    let r = perm(&mut g, P0, "Phyrexian Revoker");
    let named = card(&g, "Signal Pest");
    g.perm_mut(r).data.set(DataKey::Imprint, Val::Card(named));
    assert!(sim_core::impls::t6_stax::needle_stops(&g, "Signal Pest"));
    assert!(!sim_core::impls::t6_stax::needle_stops(&g, "Sol Ring"));
    let pest = perm(&mut g, P1, "Signal Pest");
    assert!(sim_core::cardcode::ability_locked(&g, pest, P1));
    zones::die(&mut g, r, "destroy").unwrap();
    assert!(!sim_core::impls::t6_stax::needle_stops(&g, "Signal Pest"), "gone with it");
}

// ------------------------------------------------------------------ Soulless Jailer
#[test]
fn soulless_jailer_stops_graveyard_casting_and_reanimation() {
    let mut g = tbl();
    let c = card(&g, "Swords to Plowshares");
    let cr = card(&g, "Ornithopter");
    assert!(!w::jailed(&g));
    let j = perm(&mut g, P1, "Soulless Jailer");
    assert!(w::jailed(&g));
    let f = g.registry.get(card(&g, "Soulless Jailer")).unwrap().can_cast.unwrap();
    assert!(!f(&g, j, P0, c, "gy") && !f(&g, j, P0, c, "exile"), "noncreature spells from graveyards or exile");
    assert!(f(&g, j, P0, c, "hand"), "from hand is fine");
    assert!(f(&g, j, P0, cr, "gy"), "creature spells may be cast");
    // Phoenix Chick can't return
    let chick = take(&mut g, P0, "Phoenix Chick");
    g.player_mut(P0).gy.push(chick);
    lands(&mut g, P0, "Mountain", 2, false);
    let atk: Vec<PermId> = (0..3).map(|_| token(&mut g, P0, 1)).collect();
    assert!(w::gy_attack(&mut g, P0, &atk).unwrap().is_empty());
}

// ------------------------------------------------------------------ Sundering Eruption
#[test]
fn sundering_eruption_destroys_a_land_and_grounds_blockers() {
    let mut g = tbl();
    let tower = lands(&mut g, P1, "Command Tower", 1, false)[0];
    let island = take(&mut g, P1, "Island");
    g.player_mut(P1).library.push(island);
    let n1 = g.player(P1).lands.len();
    let c = take(&mut g, P0, "Sundering Eruption // Volcanic Fissure");
    cast::resolve(&mut g, P0, c, &Ctx::default(), "hand").unwrap();
    assert!(!g.player(P1).lands.contains(&tower), "the nonbasic is destroyed");
    assert_eq!(g.player(P1).lands.len(), n1, "its controller fetched a basic");
    assert!(g.perm(*g.player(P1).lands.last().unwrap()).tapped, "tapped");
    let wall = token(&mut g, P1, 2);
    let bird = perm(&mut g, P1, "Ornithopter");
    let a = token(&mut g, P0, 2);
    assert!(!can_block(&g, wall, a), "creatures without flying can't block this turn");
    assert!(can_block(&g, bird, a), "fliers can");
    g.round += 1;
    sim_core::engine::turn::step_start(&mut g, P1).unwrap();
    assert!(can_block(&g, wall, a), "the next turn they can");
}

#[test]
fn sundering_eruption_is_cast_before_combat_into_ground_blockers() {
    let mut g = tbl();
    let c = card(&g, "Sundering Eruption // Volcanic Fissure");
    let prio = g.registry.get(c).unwrap().prio.unwrap();
    my_turn(&mut g, P0, Step::Main1);
    token(&mut g, P0, 2);
    token(&mut g, P0, 2);
    assert_eq!(prio(&g, P0, c), 0, "no blockers: held");
    token(&mut g, P1, 3);
    assert!(prio(&g, P0, c) > 0, "ground blockers before combat: cast it");
    g.step = Step::Main2;
    assert_eq!(prio(&g, P0, c), 0, "after combat: held");
}

// ------------------------------------------------------------------ combos
#[test]
fn rionya_and_combat_celebrant_is_a_combo_before_combat() {
    let mut g = tbl();
    let ready = |g: &Game| (w::COMBOS[0].ready)(g, P0).0;
    my_turn(&mut g, P0, Step::Main1);
    perm(&mut g, P0, "Rionya, Fire Dancer");
    assert!(!ready(&g), "Rionya alone");
    perm(&mut g, P0, "Combat Celebrant");
    assert!(ready(&g), "Rionya + Celebrant before combat");
    g.step = Step::Main2;
    assert!(!ready(&g), "after combat");
    assert!(sim_core::impls::combos::all().iter().any(|c| c.name == w::COMBOS[0].name), "in the catalogue");
}
