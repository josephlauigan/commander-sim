//! Lifelink on damage that is not combat damage (CR 702.15b: any damage dealt by a source with lifelink makes its
//! controller gain that much life, to a player, a creature or a planeswalker), on the sites where a permanent a deck can
//! give lifelink to deals damage: Walking Ballista, Triskelion, Orcish Bowmasters, Siege-Gang Commander's ping and a
//! compiled permanent's damage (Firemane Avenger), each for the AI's path and the person's.

mod play_common;
use play_common::*;
use sim_core::ai::act::perform;
use sim_core::engine::{life, zones};
use sim_core::hooks::Opt;
use sim_core::human::human::{AbilityKind, abilities_of};
use sim_core::human::{Target, choices};
use sim_core::state::Ctx;

fn tbl() -> Game {
    let mut g = table(&["heliod-mono-white-stax", "seph", "sauron"]);
    g.active = Some(P0);
    g
}

/// a Walking Ballista with `counters` +1/+1 counters, not summoning sick
fn ballista(g: &mut Game, counters: i32) -> PermId {
    let c = take(g, P0, "Walking Ballista");
    g.last_x = counters * 2;
    let b = zones::enter(g, P0, c, zones::Enter { was_cast: true, sick: false, ..Default::default() }).unwrap();
    assert!(g.perm(b).on_bf);
    assert_eq!(g.perm(b).plus, counters);
    b
}

fn options(g: &mut Game, src: PermId, p: PlayerId) -> Vec<Opt> {
    let cd = g.perm(src).cd.unwrap();
    let f = g.registry.get(cd).and_then(|i| i.options).expect("the card has options");
    f(g, src, p, Some(false)).unwrap()
}

/// the AI uses the option of `src` whose label contains `word`
fn ai_uses(g: &mut Game, src: PermId, word: &str) {
    let o = options(g, src, P0).into_iter().find(|o| o.label.contains(word)).unwrap_or_else(|| panic!("no {word}"));
    assert!(perform(g, P0, o.act.as_ref().unwrap()).unwrap());
}

/// a person at P0 who aims every damage at the first opponent player (`who`: None) or at the choice containing `who`,
/// and passes priority
fn person(g: &mut Game, who: Option<&'static str>) -> std::sync::Arc<ScriptHuman> {
    let h = std::sync::Arc::new(ScriptHuman::from_fn(move |r| {
        if r.kind == "priority" {
            return Answer::Action(json!({"do": "pass"}));
        }
        let p = r.prompt.to_lowercase();
        if p.contains("remove how many") {
            return Answer::Pick(0); // one counter
        }
        if p.contains("damage") {
            return match who {
                Some(w) => r.choices.iter().position(|c| c.contains(w)).map_or(Answer::Decline, Answer::Pick),
                None => r
                    .choices
                    .iter()
                    .position(|c| c.contains("(player,") && !c.contains("(you)"))
                    .map_or(Answer::Decline, Answer::Pick),
            };
        }
        Answer::Decline
    }));
    g.humans.add(P0, h.clone());
    h
}

/// the person uses the ability of `m` whose label contains `word`; the reason it could not, if any
fn use_ability(g: &mut Game, m: PermId, word: &str) -> Option<String> {
    let list = abilities_of(g, P0, m).unwrap();
    let Some((_, kind)) = list.into_iter().find(|(l, _)| l.contains(word)) else {
        panic!("no ability with {word:?} on {}", g.perm(m).name)
    };
    let AbilityKind::Extra(act) = kind else { panic!("not an extra ability") };
    act(g, P0, m).unwrap()
}

// ------------------------------------------------------------------ Walking Ballista
#[test]
fn ballista_ping_with_lifelink_gains_one_per_ping_for_the_ai() {
    let mut g = tbl();
    let b = ballista(&mut g, 3);
    g.player_mut(P1).life = 3; // lethal: the AI pings the player
    ai_uses(&mut g, b, "ping");
    assert_eq!(g.player(P1).life, 2);
    assert_eq!(g.player(P0).life, 40, "no lifelink, no gain");
    g.perm_mut(b).lifelink = true;
    ai_uses(&mut g, b, "ping");
    assert_eq!(g.player(P1).life, 1);
    assert_eq!(g.player(P0).life, 41, "one damage, one life");
}

#[test]
fn ballista_ping_with_lifelink_gains_for_a_person() {
    let mut g = tbl();
    person(&mut g, None);
    let b = ballista(&mut g, 2);
    g.perm_mut(b).lifelink = true;
    assert_eq!(use_ability(&mut g, b, "remove a +1/+1 counter"), None);
    assert_eq!(g.player(P1).life, 39);
    assert_eq!(g.player(P0).life, 41);
}

#[test]
fn a_ballista_at_its_last_counter_still_gains_what_it_dealt() {
    let mut g = tbl();
    person(&mut g, None);
    let b = ballista(&mut g, 1);
    g.perm_mut(b).lifelink = true;
    assert_eq!(use_ability(&mut g, b, "remove a +1/+1 counter"), None);
    assert!(!g.perm(b).on_bf, "0/0: it died");
    assert_eq!(g.player(P1).life, 39);
    assert_eq!(g.player(P0).life, 41, "last known information: it had lifelink");
}

#[test]
fn lifelink_damage_to_a_creature_counts_all_of_it() {
    let mut g = tbl();
    person(&mut g, Some("Grave Titan"));
    let b = ballista(&mut g, 1);
    g.perm_mut(b).lifelink = true;
    let big = perm(&mut g, P1, "Grave Titan");
    assert_eq!(use_ability(&mut g, b, "remove a +1/+1 counter"), None);
    assert!(g.perm(big).on_bf, "1 damage does not kill a 6/6");
    assert_eq!(g.player(P0).life, 41, "but it was dealt, so it gains");
}

// ------------------------------------------------------------------ Triskelion
#[test]
fn triskelion_ping_with_lifelink_gains_for_the_ai() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Triskelion");
    g.perm_mut(t).lifelink = true;
    g.player_mut(P1).life = 3;
    ai_uses(&mut g, t, "ping");
    assert_eq!(g.player(P1).life, 2);
    assert_eq!(g.player(P0).life, 41);
}

#[test]
fn triskelion_ping_with_lifelink_gains_for_a_person() {
    let mut g = tbl();
    person(&mut g, None);
    let t = perm(&mut g, P0, "Triskelion");
    g.perm_mut(t).lifelink = true;
    assert_eq!(use_ability(&mut g, t, "remove a +1/+1 counter"), None);
    assert_eq!(g.player(P1).life, 39);
    assert_eq!(g.player(P0).life, 41);
}

#[test]
fn triskelion_with_a_grant_drains_through_sanguine_bond() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Triskelion");
    g.perm_mut(t).lifelink = true;
    perm(&mut g, P0, "Sanguine Bond");
    g.player_mut(P1).life = 3;
    ai_uses(&mut g, t, "ping");
    assert_eq!(g.player(P0).life, 41, "the ping's lifelink");
    assert_eq!(
        g.player(P1).life + g.player(P2).life,
        3 + 40 - 2,
        "1 damage, then Sanguine Bond: an opponent loses 1 more"
    );
}

// ------------------------------------------------------------------ Orcish Bowmasters
#[test]
fn bowmasters_with_lifelink_gains_on_entering_for_the_ai() {
    let mut g = tbl();
    let b = perm(&mut g, P0, "Orcish Bowmasters");
    let (mine, theirs) = (g.player(P0).life, g.player(P1).life + g.player(P2).life);
    g.perm_mut(b).lifelink = true;
    zones::etb_once(&mut g, P0, b).unwrap();
    assert_eq!(g.player(P1).life + g.player(P2).life, theirs - 1, "its ping");
    assert_eq!(g.player(P0).life, mine + 1, "lifelink");
}

#[test]
fn bowmasters_with_lifelink_gains_on_entering_for_a_person() {
    let mut g = tbl();
    let b = perm(&mut g, P0, "Orcish Bowmasters");
    person(&mut g, None);
    let mine = g.player(P0).life;
    g.perm_mut(b).lifelink = true;
    zones::etb_once(&mut g, P0, b).unwrap();
    assert_eq!(g.player(P0).life, mine + 1);
}

#[test]
fn bowmasters_with_lifelink_gains_on_an_opponents_extra_draw_for_the_ai() {
    let mut g = tbl();
    let b = perm(&mut g, P0, "Orcish Bowmasters");
    let mine = g.player(P0).life;
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!(g.player(P0).life, mine, "no lifelink, no gain");
    g.perm_mut(b).lifelink = true;
    let theirs = g.player(P1).life;
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!(g.player(P1).life, theirs - 1);
    assert_eq!(g.player(P0).life, mine + 1);
}

#[test]
fn bowmasters_with_lifelink_gains_on_an_opponents_extra_draw_for_a_person() {
    let mut g = tbl();
    person(&mut g, None);
    let b = perm(&mut g, P0, "Orcish Bowmasters");
    g.perm_mut(b).lifelink = true;
    let mine = g.player(P0).life;
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!(g.player(P0).life, mine + 1);
}

// ------------------------------------------------------------------ Siege-Gang Commander
#[test]
fn siege_gang_ping_with_lifelink_gains() {
    use sim_core::impls::model_gaps_a4::{Tgt, ping};
    let mut g = tbl();
    let s = perm(&mut g, P0, "Siege-Gang Commander");
    ping(&mut g, P0, s, Tgt::Player(P1), 2).unwrap();
    assert_eq!(g.player(P0).life, 40);
    g.perm_mut(s).lifelink = true;
    ping(&mut g, P0, s, Tgt::Player(P1), 2).unwrap();
    assert_eq!(g.player(P1).life, 36);
    assert_eq!(g.player(P0).life, 42);
}

// ------------------------------------------------------------------ a compiled permanent's damage
#[test]
fn a_compiled_permanent_damage_ability_with_lifelink_gains() {
    let mut g = tbl();
    let fa = perm(&mut g, P0, "Firemane Avenger");
    let t1 = token(&mut g, P0, 1);
    let t2 = token(&mut g, P0, 1);
    g.perm_mut(fa).lifelink = true;
    let theirs = g.player(P1).life + g.player(P2).life;
    sim_core::engine::combat::attack_triggers(&mut g, P0, &[fa, t1, t2], P1).unwrap();
    assert_eq!(g.player(P1).life + g.player(P2).life, theirs - 3, "3 damage");
    assert_eq!(g.player(P0).life, 46, "its own 3, and 3 more from lifelink");
}

#[test]
fn a_compiled_permanent_damage_ability_with_lifelink_gains_for_a_person() {
    let mut g = tbl();
    person(&mut g, None);
    let fa = perm(&mut g, P0, "Firemane Avenger");
    let t1 = token(&mut g, P0, 1);
    let t2 = token(&mut g, P0, 1);
    g.perm_mut(fa).lifelink = true;
    sim_core::engine::combat::attack_triggers(&mut g, P0, &[fa, t1, t2], P1).unwrap();
    assert_eq!(g.player(P1).life, 37);
    assert_eq!(g.player(P0).life, 46);
}

#[test]
fn compiled_damage_that_kills_a_creature_gains_the_damage_dealt_for_a_person() {
    let mut g = tbl();
    person(&mut g, Some("Mother of Runes"));
    let fa = perm(&mut g, P0, "Firemane Avenger");
    let t1 = token(&mut g, P0, 1);
    let t2 = token(&mut g, P0, 1);
    g.perm_mut(fa).lifelink = true;
    let mom = perm(&mut g, P1, "Mother of Runes");
    sim_core::engine::combat::attack_triggers(&mut g, P0, &[fa, t1, t2], P1).unwrap();
    assert!(!g.perm(mom).on_bf, "3 damage kills a 1/1");
    assert_eq!(g.player(P0).life, 46, "the 3 damage dealt, not the 1 toughness");
}

// ------------------------------------------------------------------ the Dragons (Akroma's Will's lifelink reaches them)
#[test]
fn terror_of_the_peaks_with_lifelink_gains_the_damage_of_its_trigger() {
    let mut g = tbl();
    let t = perm(&mut g, P0, "Terror of the Peaks");
    token(&mut g, P0, 1);
    assert_eq!(g.player(P0).life, 40, "no lifelink, no gain");
    g.perm_mut(t).lifelink = true;
    let theirs = g.player(P1).life + g.player(P2).life;
    token(&mut g, P0, 3);
    assert_eq!(g.player(P1).life + g.player(P2).life, theirs - 3, "3 damage to an opponent");
    assert_eq!(g.player(P0).life, 43);
}

#[test]
fn terror_of_the_peaks_with_lifelink_gains_for_a_person() {
    let mut g = tbl();
    person(&mut g, None);
    let t = perm(&mut g, P0, "Terror of the Peaks");
    g.perm_mut(t).lifelink = true;
    token(&mut g, P0, 3);
    assert_eq!(g.player(P1).life, 37);
    assert_eq!(g.player(P0).life, 43);
}

#[test]
fn scourge_of_valkas_with_lifelink_gains_the_damage_of_its_trigger() {
    let mut g = tbl();
    let s = perm(&mut g, P0, "Scourge of Valkas");
    g.perm_mut(s).lifelink = true;
    let theirs = g.player(P1).life + g.player(P2).life;
    let c = take(&mut g, P0, "Scourge of Valkas");
    zones::enter(&mut g, P0, c, zones::Enter { sick: false, ..Default::default() }).unwrap();
    assert_eq!(g.player(P1).life + g.player(P2).life, theirs - 4, "each Scourge deals 2 (two Dragons)");
    // "it deals X damage": the Dragon that entered is the source of both triggers' damage, and it has no lifelink
    assert_eq!(g.player(P0).life, 40, "the first Scourge's lifelink is not the entering Dragon's");
}

#[test]
fn balefire_dragon_with_lifelink_gains_for_each_creature_it_damages() {
    let mut g = tbl();
    let b = perm(&mut g, P0, "Balefire Dragon");
    g.perm_mut(b).lifelink = true;
    g.perm_mut(b).pow = 5;
    g.perm_mut(b).tgh = 5;
    body(&mut g, P1, 1, 1);
    body(&mut g, P1, 1, 1);
    fight(&mut g, &[b], &[]);
    assert_eq!(g.player(P1).perms.len(), 0, "both creatures died");
    assert_eq!(g.player(P0).life, 40 + 5 + 5 + 5, "5 to the player, 5 to each of two creatures");
}

// ------------------------------------------------------------------ the sites the guard first listed as open
#[test]
fn fate_unraveler_with_lifelink_gains_the_damage_it_deals_when_an_opponent_draws() {
    let mut g = tbl();
    let f = perm(&mut g, P0, "Fate Unraveler");
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!(g.player(P0).life, 40, "no lifelink, no gain");
    g.perm_mut(f).lifelink = true;
    let theirs = g.player(P1).life;
    zones::draw(&mut g, P1, 1, false).unwrap();
    assert_eq!(g.player(P1).life, theirs - 1, "1 damage to the player who drew");
    assert_eq!(g.player(P0).life, 41, "lifelink");
}

#[test]
fn myr_battlesphere_with_lifelink_gains_the_damage_it_deals_when_it_attacks() {
    let mut g = tbl();
    let b = perm(&mut g, P0, "Myr Battlesphere");
    g.perm_mut(b).lifelink = true;
    let theirs = g.player(P1).life;
    sim_core::engine::combat::attack_triggers(&mut g, P0, &[b], P1).unwrap();
    assert_eq!(g.player(P1).life, theirs - 4, "it taps its four Myr: 4 damage");
    assert_eq!(g.player(P0).life, 44, "lifelink");
}

// ------------------------------------------------------------------ coverage rows (green before and after)
#[test]
fn a_spell_gains_nothing_with_a_lifelinker_out() {
    let mut g = tbl();
    let k = token(&mut g, P0, 2);
    g.perm_mut(k).lifelink = true;
    let axe = take(&mut g, P0, "Lava Axe");
    let before = (g.player(P0).life, g.player(P1).life + g.player(P2).life);
    sim_core::dsl::resolve_spell(&mut g, P0, axe, &Ctx::default()).unwrap();
    assert_eq!(g.player(P1).life + g.player(P2).life, before.1 - 5, "Lava Axe: 5 damage");
    assert_eq!(g.player(P0).life, before.0, "a spell has no lifelink");
}

#[test]
fn prevented_damage_gains_nothing() {
    let mut g = tbl();
    let b = ballista(&mut g, 2);
    g.perm_mut(b).lifelink = true;
    g.player_mut(P1).life = 2;
    lands(&mut g, P1, "Glacial Chasm", 1, false);
    ai_uses(&mut g, b, "ping");
    assert_eq!(g.player(P1).life, 2, "Glacial Chasm prevents all damage to its controller");
    assert_eq!(g.player(P0).life, 40, "nothing was dealt: nothing gained");
}

// ------------------------------------------------------------------ combat damage (resolve_blocks with chosen blocks)
/// a vanilla creature token with power `p` and toughness `t`, not summoning sick
fn body(g: &mut Game, who: PlayerId, p: i32, t: i32) -> PermId {
    let m = token(g, who, p.max(1));
    g.perm_mut(m).pow = p;
    g.perm_mut(m).tgh = t;
    m
}

/// `m` has keyword `kw` until end of turn
fn give(g: &mut Game, m: PermId, kw: &str) {
    g.perm_mut(m).eot_kw.push(sim_core::sym::intern(kw));
}

/// P0 attacks P1 with `atk`; `blocks` are (attacker, blocker)
fn fight(g: &mut Game, atk: &[PermId], blocks: &[(PermId, PermId)]) {
    let mut tot = 0;
    sim_core::engine::combat::resolve_blocks(g, P0, atk.to_vec(), P1, blocks.to_vec(), &mut tot).unwrap();
}

#[test]
fn an_unblocked_lifelink_attacker_gains_once() {
    let mut g = tbl();
    let a = body(&mut g, P0, 3, 3);
    g.perm_mut(a).lifelink = true;
    fight(&mut g, &[a], &[]);
    assert_eq!(g.player(P1).life, 37);
    assert_eq!(g.player(P0).life, 43, "once, not twice");
}

#[test]
fn lifelink_gains_the_damage_dealt_when_fiery_emancipation_triples_it() {
    let mut g = tbl();
    perm(&mut g, P0, "Fiery Emancipation");
    let a = body(&mut g, P0, 3, 3);
    g.perm_mut(a).lifelink = true;
    fight(&mut g, &[a], &[]);
    assert_eq!(g.player(P1).life, 31, "3 damage, tripled");
    assert_eq!(g.player(P0).life, 49, "a source that dealt 9 damage gains 9");
}

#[test]
fn blocked_lifelink_attacker_gains_damage_dealt_to_the_blocker() {
    let mut g = tbl();
    let a = body(&mut g, P0, 3, 3);
    g.perm_mut(a).lifelink = true;
    let b = body(&mut g, P1, 4, 4);
    fight(&mut g, &[a], &[(a, b)]);
    assert!(!g.perm(a).on_bf, "4 damage kills a 3/3");
    assert!(g.perm(b).on_bf);
    assert_eq!(g.player(P0).life, 43, "3 damage to the blocker, though it only had 4 toughness to spare");
    assert_eq!(g.player(P1).life, 40, "blocked: nothing to the player");
}

#[test]
fn a_lifelink_attacker_gains_all_its_damage_from_a_blocker_it_overkills() {
    let mut g = tbl();
    let a = body(&mut g, P0, 5, 5);
    g.perm_mut(a).lifelink = true;
    let b = body(&mut g, P1, 1, 1);
    fight(&mut g, &[a], &[(a, b)]);
    assert!(!g.perm(b).on_bf);
    assert_eq!(g.player(P0).life, 45, "all 5, not the 1 toughness");
}

#[test]
fn a_trampling_lifelink_attacker_gains_for_the_blocker_and_the_player() {
    let mut g = tbl();
    let a = body(&mut g, P0, 5, 5);
    g.perm_mut(a).lifelink = true;
    give(&mut g, a, "trample");
    let b = body(&mut g, P1, 2, 2);
    fight(&mut g, &[a], &[(a, b)]);
    assert_eq!(g.player(P1).life, 37, "3 tramples over");
    assert_eq!(g.player(P0).life, 45, "2 to the blocker and 3 to the player: all 5");
}

#[test]
fn a_lifelink_blocker_gains() {
    let mut g = tbl();
    let a = body(&mut g, P0, 3, 3);
    let b = body(&mut g, P1, 2, 2);
    g.perm_mut(b).lifelink = true;
    fight(&mut g, &[a], &[(a, b)]);
    assert!(!g.perm(b).on_bf);
    assert_eq!(g.player(P1).life, 42, "the blocker dealt 2 damage to the attacker");
    assert_eq!(g.player(P0).life, 40);
}

#[test]
fn a_first_striker_that_kills_its_blocker_leaves_the_blocker_nothing_to_gain() {
    let mut g = tbl();
    let a = body(&mut g, P0, 3, 3);
    give(&mut g, a, "first strike");
    let b = body(&mut g, P1, 2, 2);
    g.perm_mut(b).lifelink = true;
    fight(&mut g, &[a], &[(a, b)]);
    assert!(!g.perm(b).on_bf && g.perm(a).on_bf, "first strike: the blocker died before it struck");
    assert_eq!(g.player(P1).life, 40, "it dealt no damage, so it gained nothing");
}

#[test]
fn a_lifelink_attacker_on_a_planeswalker_gains() {
    let mut g = tbl();
    person(&mut g, None);
    let w = perm(&mut g, P1, "Jace, the Mind Sculptor");
    let a = body(&mut g, P0, 3, 3);
    g.perm_mut(a).lifelink = true;
    let l = g.perm(w).loyalty.unwrap();
    g.human_walker_atk = vec![(a, w)];
    fight(&mut g, &[a], &[]);
    assert_eq!(g.perm(w).loyalty, Some(l - 3), "the damage went to the planeswalker");
    assert_eq!(g.player(P0).life, 43);
    assert_eq!(g.player(P1).life, 40);
}

#[test]
fn a_lifelink_attacker_gains_all_its_damage_on_a_planeswalker_it_overkills() {
    let mut g = tbl();
    person(&mut g, None);
    let w = perm(&mut g, P1, "Jace, the Mind Sculptor");
    g.perm_mut(w).loyalty = Some(1);
    let a = body(&mut g, P0, 4, 4);
    g.perm_mut(a).lifelink = true;
    g.human_walker_atk = vec![(a, w)];
    fight(&mut g, &[a], &[]);
    assert!(!g.perm(w).on_bf);
    assert_eq!(g.player(P0).life, 44, "4 damage dealt to a planeswalker with 1 loyalty");
}

#[test]
fn a_lifelink_attacker_on_a_battle_gains() {
    use sim_core::impls::card_models3 as cm3;
    let mut g = table(&["kinnan-bonder-mana-combo", "rograkh-thrasios-creature-combo", "sisay-legends-toolbox"]);
    g.active = Some(P0);
    token(&mut g, P1, 2);
    let c = hand(&mut g, P0, &["Invasion of Ikoria // Zilortha, Apex of Ikoria"])[0];
    assert!(sim_core::engine::cast::cast_card(&mut g, P0, c, "hand", Ctx { x: 3, ..Ctx::default() }).unwrap(), "cast");
    let w = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).on_bf && cm3::defense(&g, m) > 0).unwrap();
    let d = cm3::protector(&g, w).unwrap();
    let a = body(&mut g, P0, 4, 4);
    g.perm_mut(a).lifelink = true;
    let (life, defense) = (g.player(P0).life, cm3::defense(&g, w));
    let mut tot = 0;
    sim_core::engine::combat::resolve_blocks(&mut g, P0, vec![a], d, vec![], &mut tot).unwrap();
    assert_eq!(cm3::defense(&g, w), (defense - 4).max(0), "the battle lost 4 defense");
    assert_eq!(g.player(P0).life, life + 4);
}

#[test]
fn a_lifelink_attacker_into_glacial_chasm_gains_nothing() {
    let mut g = tbl();
    lands(&mut g, P1, "Glacial Chasm", 1, false);
    let a = body(&mut g, P0, 3, 3);
    g.perm_mut(a).lifelink = true;
    fight(&mut g, &[a], &[]);
    assert_eq!(g.player(P1).life, 40, "prevented");
    assert_eq!(g.player(P0).life, 40, "nothing dealt, nothing gained");
}

// ------------------------------------------------------------------ the guard (decks.json: yours and the pools)
/// which permanents a card that grants lifelink reaches. Cards that give it only to tokens (Teysa Karlov, Archon of Sun's
/// Grace, Dreadhorde Invasion) are left out: no deck list names a token
#[derive(Clone, Copy)]
enum Reach {
    Any,
    Black,
    Angels,
}

const GRANTERS: &[(&str, Reach)] = &[
    ("Sorin, Vengeful Bloodlord", Reach::Any),
    ("Heliod, Sun-Crowned", Reach::Any),
    ("Vito, Thorn of the Dusk Rose", Reach::Any),
    ("Akroma's Will", Reach::Any),
    ("Vault of the Archangel", Reach::Any),
    ("Felidar Umbra", Reach::Any),
    ("Sheltered by Ghosts", Reach::Any),
    ("On Serra's Wings", Reach::Any),
    ("Lyra Dawnbringer", Reach::Angels),
    ("Scion of Draco", Reach::Black),
];

/// creatures and planeswalkers that deal damage other than combat damage, and read lifelink at the site (a compiled
/// permanent's damage reads it too: `dsl::run`'s "damage")
const READS_LIFELINK: &[&str] = &[
    "Walking Ballista",
    "Triskelion",
    "Orcish Bowmasters",
    "Siege-Gang Commander",
    "Terror of the Peaks",
    "Scourge of Valkas",
    "Dragon Tempest",
    "Drakuseth, Maw of Flames",
    "Balefire Dragon",
    "Fate Unraveler",
    "Myr Battlesphere",
    "Sorin, Vengeful Bloodlord",
    "Aurelia, the Law Above",
    "Firemane Avenger",
    "Niv-Mizzet, the Firemind",
];

/// ... and do not (yet): their damage sites are card code that does not read lifelink. A deck that pairs one with a
/// granter that reaches it fails the test below, unless the pair is in `KNOWN_OPEN`
const IGNORES_LIFELINK: &[&str] = &[
    "Mayhem Devil",
    "Midnight Reaper",
    "Syr Konrad, the Grim",
    "Vindictive Vampire",
    "Kaervek the Merciless",
    "Smaug the Magnificent",
    "Glorybringer",
    "Thundermaw Hellkite",
    "Goblin Chainwhirler",
    "Goblin Sharpshooter",
    "Jagged-Scar Archers",
];

/// pairs awaiting a decision (convert the site, or accept it): none; the two found when the guard was written are converted
const KNOWN_OPEN: &[(&str, &str)] = &[];

fn reaches(reach: Reach, card: &str) -> bool {
    let db = sim_core::testkit::db();
    let d = db.get(db.id(card).unwrap_or_else(|| panic!("{card} is not in cards.json")));
    match reach {
        Reach::Any => true,
        Reach::Black => d.pips.contains('B'),
        Reach::Angels => d.subtypes.iter().any(|t| &**t == "angel"),
    }
}

fn pairs() -> Vec<(String, &'static str, &'static str)> {
    let mut out = vec![];
    for d in sim_core::testkit::decks().iter().filter(|d| d.tier.as_deref() != Some("retired")) {
        for &(granter, reach) in GRANTERS {
            if !d.cards.iter().any(|c| c == granter) {
                continue;
            }
            for &o in IGNORES_LIFELINK {
                if d.cards.iter().any(|c| c == o) && reaches(reach, o) {
                    out.push((d.key.clone(), granter, o));
                }
            }
        }
    }
    out
}

#[test]
fn no_deck_pairs_a_lifelink_granter_with_a_damage_site_that_ignores_lifelink() {
    let bad: Vec<String> = pairs()
        .into_iter()
        .filter(|(k, _, o)| !KNOWN_OPEN.contains(&(k.as_str(), *o)))
        .map(|(k, g, o)| format!("{k}: {g} can give {o} lifelink, but its damage site does not read it"))
        .collect();
    assert!(bad.is_empty(), "convert the site (`life::lifelink`) or list the pair in KNOWN_OPEN: {bad:#?}");
}

#[test]
fn every_known_open_pair_is_still_open() {
    let now = pairs();
    for &(k, o) in KNOWN_OPEN {
        assert!(now.iter().any(|(dk, _, dc)| dk == k && *dc == o), "{k} / {o} is no longer a pair: remove it");
    }
}

#[test]
fn the_guard_lists_name_real_cards_and_overlap_nowhere() {
    let db = sim_core::testkit::db();
    for n in GRANTERS.iter().map(|g| g.0).chain(READS_LIFELINK.iter().copied()).chain(IGNORES_LIFELINK.iter().copied())
    {
        assert!(db.id(n).is_some(), "{n} is not in cards.json");
    }
    assert!(READS_LIFELINK.iter().all(|n| !IGNORES_LIFELINK.contains(n)));
}

// ------------------------------------------------------------------ fix round: what lifelink follows
/// P0's `s` deals `n` damage to `t` as a permanent's ability (the person's damage function, with last known lifelink)
fn deal(g: &mut Game, s: PermId, t: Target, n: i32) -> i32 {
    let by = choices::by(g, Some(s));
    choices::apply_damage_by(g, P0, t, n, "test permanent", "burn", by).unwrap()
}

#[test]
fn bloodletter_doubles_life_loss_but_not_damage_or_lifelink() {
    let mut g = tbl();
    perm(&mut g, P0, "Bloodletter of Aclazotz");
    let s = body(&mut g, P0, 2, 2);
    g.perm_mut(s).lifelink = true;
    deal(&mut g, s, Target::Player(P1), 1);
    assert_eq!(g.player(P1).life, 38, "one damage causes two life lost");
    assert_eq!(g.player(P0).life, 41, "only one damage was dealt");
}

#[test]
fn a_life_total_lock_does_not_prevent_damage_or_lifelink() {
    let mut g = tbl();
    let s = body(&mut g, P0, 2, 2);
    g.perm_mut(s).lifelink = true;
    g.player_mut(P1).life_locked = true; // a life-total lock, no protection
    deal(&mut g, s, Target::Player(P1), 3);
    assert_eq!(g.player(P1).life, 40, "the locked life total does not change");
    assert_eq!(g.player(P0).life, 43, "the damage was dealt");
}

#[test]
fn life_lock_combat_regression_still_gains_life() {
    let mut g = tbl();
    let s = body(&mut g, P0, 3, 3);
    g.perm_mut(s).lifelink = true;
    g.player_mut(P1).life_locked = true;
    fight(&mut g, &[s], &[]);
    assert_eq!((g.player(P1).life, g.player(P0).life), (40, 43));
}

#[test]
fn indestructible_does_not_make_an_unpayable_ward_deal_damage() {
    let mut g = tbl();
    let s = body(&mut g, P0, 2, 2);
    g.perm_mut(s).lifelink = true;
    let t = perm(&mut g, P1, "Sauron, the Dark Lord");
    give(&mut g, t, "indestructible");
    assert!(sim_core::engine::values::ward_legends(&g, P0).is_empty(), "the ward (sacrifice a legend) can't be paid");
    assert_eq!(deal(&mut g, s, Target::Perm(t), 10), 0, "ward counters the ability");
    assert_eq!(g.player(P0).life, 40);
}

#[test]
fn noncombat_lifelink_damage_to_a_creature_on_an_arachnogenesis_turn_gains() {
    let mut g = tbl();
    let s = body(&mut g, P0, 2, 2);
    g.perm_mut(s).lifelink = true;
    let t = body(&mut g, P1, 1, 4);
    g.fog_spiders = Some(g.turn_stamp()); // prevents COMBAT damage by non-Spiders this turn
    assert_eq!(deal(&mut g, s, Target::Perm(t), 1), 1);
    assert_eq!(g.player(P0).life, 41, "a ping is not combat damage: the fog does not stop it");
}

#[test]
fn source_damage_prevention_gains_nothing_for_a_ballista_ping_to_a_player() {
    let mut g = tbl();
    person(&mut g, None);
    let b = ballista(&mut g, 2);
    g.perm_mut(b).lifelink = true;
    let spider = perm(&mut g, P1, "Old Fat Spider Can't See Me");
    g.perm_mut(spider).data.set(sim_core::state::DataKey::Prevent, sim_core::state::Val::Perm(b));
    assert!(sim_core::engine::repl::source_prevented(&g, b));
    assert_eq!(use_ability(&mut g, b, "remove a +1/+1 counter"), None);
    assert_eq!(g.perm(b).plus, 1, "the activation was exercised");
    assert_eq!(g.player(P0).life, 40, "all the damage the Ballista would deal is prevented: nothing to gain");
}

// ------------------------------------------------------------------ fix round: combat gain events
const HELIOD: &str = "Heliod, Sun-Crowned";

/// P0's `a` attacks P1, blocked by `b` or not
fn duel(g: &mut Game, a: PermId, b: Option<PermId>) {
    fight(g, &[a], &b.map(|b| (a, b)).into_iter().collect::<Vec<_>>());
}

/// the +1/+1 counters on all of p's permanents (Heliod puts one on a creature for each gain event)
fn counters(g: &Game, p: PlayerId) -> i32 {
    g.player(p).perms.iter().map(|&m| g.perm(m).plus).sum()
}

#[test]
fn fog_prevents_blocked_lifelink_damage() {
    let mut g = tbl();
    let a = body(&mut g, P0, 3, 4);
    let b = body(&mut g, P1, 2, 4);
    g.perm_mut(a).lifelink = true;
    g.perm_mut(b).lifelink = true;
    g.fog = Some(g.turn_stamp());
    duel(&mut g, a, Some(b));
    assert_eq!((g.player(P0).life, g.player(P1).life), (40, 40), "a fog prevents all combat damage: no gain");
}

#[test]
fn trample_is_one_gain_event_for_one_source_in_one_step() {
    let mut g = tbl();
    perm(&mut g, P0, HELIOD);
    let a = body(&mut g, P0, 5, 5);
    let b = body(&mut g, P1, 2, 2);
    g.perm_mut(a).lifelink = true;
    give(&mut g, a, "trample");
    duel(&mut g, a, Some(b));
    assert_eq!(g.player(P0).life, 45);
    assert_eq!(counters(&g, P0), 1, "one 5-life gain, not a 2-life gain and a 3-life gain");
}

#[test]
fn blocked_double_strike_has_two_gain_events() {
    let mut g = tbl();
    perm(&mut g, P0, HELIOD);
    let a = body(&mut g, P0, 2, 5);
    let b = body(&mut g, P1, 0, 10);
    g.perm_mut(a).lifelink = true;
    give(&mut g, a, "double strike");
    duel(&mut g, a, Some(b));
    assert_eq!(counters(&g, P0), 2, "two damage steps cause two gains");
}

#[test]
fn indestructible_blocker_survives_first_strike_and_is_hit_again() {
    let mut g = tbl();
    let a = body(&mut g, P0, 3, 5);
    let b = body(&mut g, P1, 0, 2);
    g.perm_mut(a).lifelink = true;
    give(&mut g, a, "double strike");
    give(&mut g, b, "indestructible");
    duel(&mut g, a, Some(b));
    assert!(g.perm(b).on_bf);
    assert_eq!(g.player(P0).life, 46, "3 in each step to an indestructible blocker");
}

#[test]
fn pariah_redirected_combat_damage_still_gains_life() {
    let mut g = tbl();
    let s = body(&mut g, P0, 3, 3);
    g.perm_mut(s).lifelink = true;
    let t = body(&mut g, P1, 1, 10);
    let a = perm(&mut g, P1, "Pariah");
    g.perm_mut(a).attached = Some(t);
    assert!(g.auras.contains(&a));
    duel(&mut g, s, None);
    assert_eq!(g.player(P1).life, 40, "redirected");
    assert_eq!(g.player(P0).life, 43, "the damage was dealt, to the enchanted creature");
}

#[test]
fn a_double_striking_deathtouch_lifelinker_gains_only_its_first_hit() {
    let mut g = tbl();
    let a = body(&mut g, P0, 1, 1);
    let b = body(&mut g, P1, 0, 5);
    g.perm_mut(a).lifelink = true;
    g.perm_mut(a).dt = true;
    give(&mut g, a, "double strike");
    duel(&mut g, a, Some(b));
    assert!(!g.perm(b).on_bf, "deathtouch: 1 damage kills the 0/5 in the first step");
    assert_eq!(g.player(P0).life, 41, "the blocker is gone for the second step");
}

#[test]
fn a_double_striking_lifelink_blocker_gains_twice() {
    let mut g = tbl();
    perm(&mut g, P1, HELIOD);
    let a = body(&mut g, P0, 1, 10);
    let b = body(&mut g, P1, 2, 5);
    g.perm_mut(b).lifelink = true;
    give(&mut g, b, "double strike");
    duel(&mut g, a, Some(b));
    assert_eq!(g.player(P1).life, 44, "2 damage in each of two steps");
    assert_eq!(counters(&g, P1), 2, "two gain events");
}

#[test]
fn unblocked_double_strike_on_a_planeswalker_is_two_gains() {
    let mut g = tbl();
    person(&mut g, None);
    perm(&mut g, P0, HELIOD);
    let a = body(&mut g, P0, 2, 5);
    g.perm_mut(a).lifelink = true;
    give(&mut g, a, "double strike");
    let w = perm(&mut g, P1, "Jace, the Mind Sculptor");
    g.perm_mut(w).loyalty = Some(10);
    g.human_walker_atk = vec![(a, w)];
    duel(&mut g, a, None);
    assert_eq!(g.perm(w).loyalty, Some(6), "2 damage in each of two steps");
    assert_eq!(g.player(P0).life, 44);
    assert_eq!(counters(&g, P0), 2, "two gain events");
}

#[test]
fn nonpositive_damage_does_not_gain_life_or_loyalty() {
    let mut g = tbl();
    let s = body(&mut g, P0, 2, 2);
    g.perm_mut(s).lifelink = true;
    let w = perm(&mut g, P1, "Jace, the Mind Sculptor");
    let l = g.perm(w).loyalty;
    deal(&mut g, s, Target::Perm(w), 0);
    deal(&mut g, s, Target::Perm(w), -1);
    assert_eq!((g.perm(w).loyalty, g.player(P0).life), (l, 40), "0 damage is no damage (CR 120.8)");
}

// ------------------------------------------------------------------ coverage rows (green before and after the fix round)
#[test]
fn a_lifelink_blocker_killed_in_first_strike_does_not_hit_back() {
    let mut g = tbl();
    let a = body(&mut g, P0, 3, 5);
    let b = body(&mut g, P1, 2, 2);
    give(&mut g, a, "double strike");
    g.perm_mut(b).lifelink = true;
    duel(&mut g, a, Some(b));
    assert_eq!(g.player(P1).life, 40, "it died in the first step and dealt nothing");
}

#[test]
fn no_lifelink_permanent_damage_gains_zero() {
    let mut g = tbl();
    let s = body(&mut g, P0, 2, 2);
    let t = body(&mut g, P1, 1, 6);
    deal(&mut g, s, Target::Perm(t), 3);
    deal(&mut g, s, Target::Player(P1), 2);
    assert_eq!((g.player(P0).life, g.player(P1).life), (40, 38));
}

#[test]
fn unblocked_double_strike_causes_two_gains() {
    let mut g = tbl();
    perm(&mut g, P0, HELIOD);
    let a = body(&mut g, P0, 2, 5);
    g.perm_mut(a).lifelink = true;
    give(&mut g, a, "double strike");
    duel(&mut g, a, None);
    assert_eq!(g.player(P0).life, 44);
    assert_eq!(counters(&g, P0), 2, "two damage steps, two gain events");
}

#[test]
fn wither_lifelink_gains_damage_dealt_as_counters() {
    let mut g = tbl();
    let a = body(&mut g, P0, 2, 5);
    let b = body(&mut g, P1, 1, 5);
    g.perm_mut(a).lifelink = true;
    give(&mut g, a, "wither");
    duel(&mut g, a, Some(b));
    assert_eq!(g.perm(b).plus, -2);
    assert_eq!(g.player(P0).life, 42);
}

#[test]
fn grace_floor_preserves_full_damage_for_lifelink() {
    let mut g = tbl();
    let s = body(&mut g, P0, 2, 2);
    g.perm_mut(s).lifelink = true;
    g.player_mut(P1).life = 1;
    let stamp = g.turn_stamp();
    g.player_mut(P1).flag_turn.insert("stax_grace", stamp);
    assert_eq!(deal(&mut g, s, Target::Player(P1), 3), 3);
    assert_eq!((g.player(P1).life, g.player(P0).life), (1, 43));
}

#[test]
fn serra_floor_preserves_full_damage_for_lifelink() {
    let mut g = tbl();
    let s = body(&mut g, P0, 2, 2);
    g.perm_mut(s).lifelink = true;
    body(&mut g, P1, 2, 2);
    g.player_mut(P1).life = 1;
    g.player_mut(P1).emblems.push("serra");
    assert_eq!(deal(&mut g, s, Target::Player(P1), 3), 3);
    assert_eq!((g.player(P1).life, g.player(P0).life), (1, 43));
}

#[test]
fn unlife_poison_still_gains_the_damage() {
    let mut g = tbl();
    let s = body(&mut g, P0, 2, 2);
    g.perm_mut(s).lifelink = true;
    perm(&mut g, P1, "Phyrexian Unlife");
    g.player_mut(P1).life = 0;
    assert_eq!(deal(&mut g, s, Target::Player(P1), 3), 3);
    assert_eq!((g.player(P1).life, g.player(P1).poison, g.player(P0).life), (0, 3, 43));
}

#[test]
fn pariah_redirected_noncombat_damage_still_gains_life() {
    let mut g = tbl();
    let s = body(&mut g, P0, 3, 3);
    g.perm_mut(s).lifelink = true;
    let t = body(&mut g, P1, 1, 10);
    let a = perm(&mut g, P1, "Pariah");
    g.perm_mut(a).attached = Some(t);
    assert!(g.auras.contains(&a));
    deal(&mut g, s, Target::Player(P1), 3);
    assert_eq!((g.player(P1).life, g.player(P0).life), (40, 43));
}

#[test]
fn two_lifelink_sources_have_separate_gain_events() {
    let mut g = tbl();
    perm(&mut g, P0, HELIOD);
    let a = body(&mut g, P0, 2, 4);
    let b = body(&mut g, P0, 2, 4);
    g.perm_mut(a).lifelink = true;
    g.perm_mut(b).lifelink = true;
    fight(&mut g, &[a, b], &[]);
    assert_eq!(g.player(P0).life, 44);
    assert_eq!(counters(&g, P0), 2, "one gain event for each source");
}

#[test]
fn converted_sites_without_lifelink_gain_no_extra_life() {
    for source in [
        "Walking Ballista",
        "Triskelion",
        "Orcish Bowmasters",
        "Terror of the Peaks",
        "Siege-Gang Commander",
        "Firemane Avenger",
    ] {
        let mut g = tbl();
        let s = if source == "Walking Ballista" { ballista(&mut g, 2) } else { perm(&mut g, P0, source) };
        let before = g.player(P1).life + g.player(P2).life;
        match source {
            "Walking Ballista" | "Triskelion" => {
                g.player_mut(P1).life = 1;
                ai_uses(&mut g, s, "ping");
                assert_eq!(g.player(P1).life, 0, "{source} actually pinged");
            }
            "Orcish Bowmasters" => zones::draw(&mut g, P1, 1, false).unwrap(),
            "Terror of the Peaks" => {
                body(&mut g, P0, 3, 3);
            }
            "Siege-Gang Commander" => {
                use sim_core::impls::model_gaps_a4::{Tgt, ping};
                ping(&mut g, P0, s, Tgt::Player(P1), 2).unwrap();
            }
            "Firemane Avenger" => {
                let a = body(&mut g, P0, 1, 1);
                let b = body(&mut g, P0, 1, 1);
                sim_core::engine::combat::attack_triggers(&mut g, P0, &[s, a, b], P1).unwrap();
            }
            _ => unreachable!(),
        }
        if !matches!(source, "Walking Ballista" | "Triskelion") {
            let dealt = match source {
                "Orcish Bowmasters" => 1,
                "Siege-Gang Commander" => 2,
                _ => 3,
            };
            assert_eq!(g.player(P1).life + g.player(P2).life, before - dealt, "{source} actually dealt damage");
        }
        assert_eq!(g.player(P0).life, if source == "Firemane Avenger" { 43 } else { 40 }, "{source}: no lifelink gain");
    }
}

// ------------------------------------------------------------------ fix round 2: the entering Dragon is the source
#[test]
fn scourge_of_valkas_gains_when_the_entering_dragon_has_lifelink() {
    let mut g = tbl();
    let s = perm(&mut g, P0, "Scourge of Valkas");
    let d = perm(&mut g, P0, "Shivan Dragon");
    g.perm_mut(d).lifelink = true;
    assert!(!life::has_lifelink(&g, s));
    let theirs = g.player(P1).life + g.player(P2).life;
    let cd = g.perm(s).cd.unwrap();
    let etb = g.registry.get(cd).and_then(|i| i.etb).expect("Scourge has an enter hook");
    etb(&mut g, s, P0, d).unwrap(); // the Dragon `d` entered: it deals X damage, X = the Dragons P0 controls
    assert_eq!(g.player(P1).life + g.player(P2).life, theirs - 2, "two Dragons: 2 damage");
    assert_eq!(g.player(P0).life, 42, "the entering Dragon dealt it and has lifelink; the Scourge has none");
}

#[test]
fn dragon_tempest_gains_when_the_entering_dragon_has_lifelink() {
    let mut g = tbl();
    perm(&mut g, P0, "Shivan Dragon");
    perm(&mut g, P0, "Sorin, Vengeful Bloodlord"); // your creatures have lifelink on your turn
    perm(&mut g, P0, "Dragon Tempest");
    let theirs = g.player(P1).life + g.player(P2).life;
    let c = take(&mut g, P0, "Shivan Dragon");
    let d = zones::enter(&mut g, P0, c, zones::Enter { sick: false, ..Default::default() }).unwrap();
    assert!(life::has_lifelink(&g, d));
    assert_eq!(g.player(P1).life + g.player(P2).life, theirs - 2, "it deals X damage, X = 2 Dragons");
    assert_eq!(g.player(P0).life, 42, "the enchantment has no lifelink; the Dragon that entered does");
}

// ------------------------------------------------------------------ fix round 2: one damage event, saturation
#[test]
fn drakuseths_attack_trigger_is_one_damage_event_for_lifelink() {
    let mut g = tbl();
    perm(&mut g, P0, HELIOD);
    let d = perm(&mut g, P0, "Drakuseth, Maw of Flames");
    g.perm_mut(d).lifelink = true;
    let theirs = g.player(P1).life + g.player(P2).life;
    let cd = g.perm(d).cd.unwrap();
    let attack = g.registry.get(cd).and_then(|i| i.attack).expect("Drakuseth has an attack hook");
    attack(&mut g, d, P0, &[d], P1).unwrap();
    assert_eq!(g.player(P1).life + g.player(P2).life, theirs - 10, "4, 3 and 3 damage to players");
    assert_eq!(g.player(P0).life, 50, "all of it is gained");
    assert_eq!(counters(&g, P0), 1, "one source, one damage event: one life gain (Heliod counts the events)");
}

#[test]
fn a_double_striking_wither_lifelink_blocker_gains_both_hits_when_the_first_is_not_lethal() {
    let mut g = tbl();
    let a = body(&mut g, P0, 3, 4);
    let b = body(&mut g, P1, 2, 5);
    g.perm_mut(b).lifelink = true;
    give(&mut g, b, "double strike");
    give(&mut g, b, "wither");
    duel(&mut g, a, Some(b));
    // 2 wither damage in the first step takes the attacker to 1/2, not to 0 toughness: it takes the second hit too
    assert_eq!(g.player(P1).life, 44, "2 damage in each of two steps, 4 in all");
}
