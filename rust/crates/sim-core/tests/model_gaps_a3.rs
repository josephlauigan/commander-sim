//! model-gaps-a3: token lands leave no card behind, Astral Dragon's land and Aura copies, ward on every targeting,
//! Mystic Forge's top-of-library counterspell, Peter Parker's Camera and Marvin with hand-written abilities.

mod play_common;
use play_common::*;
use sim_core::control::Answer;
use sim_core::engine::turn::remove_land;
use sim_core::engine::zones::Zone;
use sim_core::impls::{fixes, lands as land_fx, rules};
use std::sync::Arc;

fn tbl() -> Game {
    table(&["karn", "kefka", "sauron"])
}

/// a token copy of a land card (a Mirrorworks or Astral Dragon copy), listed in the controller's lands
fn token_land(g: &mut Game, p: PlayerId, name: &str) -> PermId {
    let c = card(g, name);
    let t = g.add_land(p, c, false);
    g.perm_mut(t).token = true;
    t
}

/// how many copies of card `name` sit in p's graveyard, hand, library and exile together
fn copies_elsewhere(g: &Game, p: PlayerId, name: &str) -> usize {
    let c = card(g, name);
    let pl = g.player(p);
    let n = |z: &Vec<sim_core::ids::CardId>| z.iter().filter(|&&x| x == c).count();
    n(&pl.gy) + n(&pl.hand) + n(&pl.library) + pl.exile.iter().filter(|&&x| x == c).count()
}

// ======================================================== item 1: token lands
#[test]
fn a_token_land_destroyed_or_sacrificed_leaves_no_card() {
    let mut g = tbl();
    let l = lands(&mut g, P0, "Treasure Vault", 1, false)[0]; // (a real one, taken from the library)
    let before = copies_elsewhere(&g, P0, "Treasure Vault");
    let t = token_land(&mut g, P0, "Treasure Vault");
    fixes::destroy_land(&mut g, P0, t).unwrap();
    assert!(!g.player(P0).lands.contains(&t), "destroyed");
    assert_eq!(copies_elsewhere(&g, P0, "Treasure Vault"), before, "a token ceases to exist: no card in the graveyard");
    let t = token_land(&mut g, P0, "Treasure Vault");
    land_fx::sac_land(&mut g, P0, t).unwrap();
    assert_eq!(copies_elsewhere(&g, P0, "Treasure Vault"), before);
    // a real Vault does go to the graveyard
    fixes::destroy_land(&mut g, P0, l).unwrap();
    assert_eq!(copies_elsewhere(&g, P0, "Treasure Vault"), before + 1);
}

#[test]
fn a_token_land_bounced_exiled_or_tucked_leaves_no_card() {
    let mut g = tbl();
    let l = lands(&mut g, P0, "Darksteel Citadel", 1, false)[0];
    let before = copies_elsewhere(&g, P0, "Darksteel Citadel");
    for z in [Zone::Hand, Zone::Exile, Zone::Gy, Zone::Lib, Zone::Top] {
        let t = token_land(&mut g, P0, "Darksteel Citadel");
        assert!(!remove_land(&mut g, P0, t, Some(z)), "no card went anywhere");
        assert!(!g.player(P0).lands.contains(&t) && !g.perm(t).on_bf);
    }
    assert_eq!(copies_elsewhere(&g, P0, "Darksteel Citadel"), before);
    assert!(remove_land(&mut g, P0, l, Some(Zone::Hand)));
    assert_eq!(copies_elsewhere(&g, P0, "Darksteel Citadel"), before + 1, "a real land card goes to the hand");
    assert!(g.player(P0).hand.contains(&card(&g, "Darksteel Citadel")));
}

#[test]
fn mass_sacrifice_of_lands_leaves_no_card_for_a_token() {
    let mut g = tbl();
    lands(&mut g, P1, "Wastes", 2, false);
    token_land(&mut g, P1, "Treasure Vault");
    token_land(&mut g, P1, "Darksteel Citadel");
    let before = (copies_elsewhere(&g, P1, "Treasure Vault"), copies_elsewhere(&g, P1, "Darksteel Citadel"));
    let src = perm(&mut g, P0, "Liliana, Dreadhorde General");
    (rules::LDG_ULT.eff)(&mut g, P0, src).unwrap();
    // Oracle: a permanent of each permanent type. Treasure Vault and Darksteel Citadel are artifact lands, so one of them
    // is the opponent's artifact and a land is the other pick: at most two lands stay, and one of them is an artifact
    let kept = g.player(P1).lands.len();
    assert!((1..=2).contains(&kept), "one permanent of each type (artifact, land): {kept} lands");
    assert!(
        g.player(P1)
            .lands
            .iter()
            .any(|&l| g.db.get(g.perm(l).card()).types.has(sim_core::cards::Types::ARTIFACT)),
        "an artifact land stays as the artifact"
    );
    assert_eq!((copies_elsewhere(&g, P1, "Treasure Vault"), copies_elsewhere(&g, P1, "Darksteel Citadel")), before);
}

// ======================================================== item 2: Astral Dragon copies a land or an Aura
/// a permanent put onto the battlefield without its enters trigger (the test calls it)
fn quiet_perm(g: &mut Game, p: PlayerId, name: &str) -> PermId {
    g.skip_etb = true;
    let m = perm(g, p, name);
    g.skip_etb = false;
    m
}

fn astral_etb(g: &mut Game, p: PlayerId, src: PermId) {
    let c = g.perm(src).cd.unwrap();
    let f = g.registry.get(c).unwrap().etb.unwrap();
    f(g, src, p, src).unwrap();
}

/// a person's answers: pick the choice with this word, otherwise say yes (the trigger window)
fn pick_word(word: &'static str) -> F {
    Box::new(move |r| match r.choices.iter().position(|x| x.contains(word)) {
        Some(i) => Answer::Pick(i),
        None => Answer::Yes,
    })
}

fn astral_land_copies(g: &Game, p: PlayerId, name: &str) -> Vec<PermId> {
    g.player(p)
        .lands
        .iter()
        .copied()
        .filter(|&l| g.perm(l).token && &*g.db.get(g.perm(l).card()).name == name)
        .collect()
}

#[test]
fn astral_dragon_copies_a_land_that_is_also_a_dragon_creature() {
    let mut g = tbl();
    lands(&mut g, P0, "Wastes", 2, false);
    let vault = lands(&mut g, P0, "Treasure Vault", 1, false)[0];
    let dragon = quiet_perm(&mut g, P0, "Astral Dragon");
    seat(&mut g, P0, vec![pick_word("Treasure Vault"), pick_word("Treasure Vault"), pick_word("Treasure Vault")]);
    let (nl, np) = (g.player(P0).lands.len(), g.player(P0).perms.len());
    astral_etb(&mut g, P0, dragon);
    let toks = astral_land_copies(&g, P0, "Treasure Vault");
    assert_eq!(toks.len(), 2, "two token copies of the land, in the lands");
    assert_eq!(g.player(P0).lands.len(), nl + 2);
    assert_eq!(g.player(P0).perms.len(), np + 2, "listed as creatures too");
    for &t in &toks {
        assert!(g.is_creature(t) && sim_core::engine::values::has_type(&g, t, "dragon"));
        assert_eq!((sim_core::engine::values::epow(&g, t), sim_core::engine::values::etgh(&g, t)), (3, 3));
        assert!(g.perm(t).fly && g.perm(t).sick, "flying, summoning sick");
    }
    assert!(g.player(P0).lands.contains(&vault) && !g.is_creature(vault));
    // summoning sick: no mana from them yet; next turn they tap for {C}
    let before = sim_core::engine::mana::total_mana(&g, P0, false);
    for &t in &toks {
        g.perm_mut(t).sick = false;
    }
    assert_eq!(
        sim_core::engine::mana::total_mana(&g, P0, false),
        before + 2,
        "each taps for {{C}} once it is not sick"
    );
    // a creature: it attacks (the opponent has no blocker)
    g.humans = Default::default();
    let life = g.player(P1).life + g.player(P2).life;
    sim_core::engine::combat::combat(&mut g, P0).unwrap();
    assert!(g.player(P1).life + g.player(P2).life < life, "the Dragon lands attack");
    assert!(toks.iter().any(|&t| g.perm(t).tapped), "attacking taps them");
    // it dies as a creature: no card, no land left
    let gy = g.player(P0).gy.len();
    sim_core::engine::zones::die(&mut g, toks[0], "dmg").unwrap();
    assert!(!g.player(P0).lands.contains(&toks[0]) && !g.player(P0).perms.contains(&toks[0]));
    assert_eq!(g.player(P0).gy.len(), gy, "a token leaves no card");
    // and as a land it can be destroyed (Strip Mine, Wasteland)
    fixes::destroy_land(&mut g, P0, toks[1]).unwrap();
    assert!(!g.perm(toks[1]).on_bf && !g.player(P0).perms.contains(&toks[1]));
    assert_eq!(g.player(P0).gy.len(), gy);
}

#[test]
fn astral_dragon_copying_an_aura_makes_aura_creatures_that_go_to_the_graveyard() {
    let mut g = tbl();
    let aura = card(&g, "Arrest");
    let a0 = quiet_perm(&mut g, P0, "Arrest"); // (the Aura to copy: what it enchants does not matter here)
    let dragon = quiet_perm(&mut g, P0, "Astral Dragon");
    seat(&mut g, P0, (0..12).map(|_| pick_word("Arrest")).collect());
    let toks = g.player(P0).perms.len();
    let gy0 = g.player(P0).gy.iter().filter(|&&c| &*g.db.get(c).name == "Arrest").count();
    astral_etb(&mut g, P0, dragon);
    assert_eq!(g.player(P0).perms.len(), toks, "both Aura creature tokens are gone (303.4d)");
    let entered = g.entered.iter().filter(|e| g.perm(e.1).token && g.perm(e.1).cd == Some(aura)).count();
    assert_eq!(entered, 2, "both Aura creature tokens did enter (their enters triggers saw them)");
    let _ = a0;
    assert_eq!(
        g.player(P0).gy.iter().filter(|&&c| &*g.db.get(c).name == "Arrest").count(),
        gy0,
        "tokens cease to exist"
    );
}

#[test]
fn the_ai_still_copies_a_nonland_permanent() {
    let mut g = tbl();
    lands(&mut g, P0, "Treasure Vault", 1, false);
    perm(&mut g, P0, "Sol Ring");
    let dragon = quiet_perm(&mut g, P0, "Astral Dragon");
    astral_etb(&mut g, P0, dragon);
    assert!(astral_land_copies(&g, P0, "Treasure Vault").is_empty());
    let sol = card(&g, "Sol Ring");
    assert_eq!(g.player(P0).perms.iter().filter(|&&m| g.perm(m).token && g.perm(m).cd == Some(sol)).count(), 2);
}

// ======================================================== item 3: ward on every targeting
use sim_core::engine::{control, removal};

#[test]
fn ward_counters_a_bounce_spell_unless_the_mana_is_paid() {
    let mut g = tbl();
    let ft = perm(&mut g, P1, "Frost Titan"); // ward {2}
    let bolt = card(&g, "Lightning Bolt");
    // no mana: the bounce is countered
    removal::apply_removal(&mut g, Some(P0), ft, "bounce", Some(bolt)).unwrap();
    assert!(g.perm(ft).on_bf, "ward {{2}} unpaid: the spell is countered");
    // with the mana the AI pays and the bounce happens
    lands(&mut g, P0, "Wastes", 2, false);
    removal::apply_removal(&mut g, Some(P0), ft, "bounce", Some(bolt)).unwrap();
    assert!(!g.perm(ft).on_bf, "paid: bounced");
    assert!(g.player(P0).lands.iter().all(|&l| g.perm(l).tapped), "{{2}} was tapped for the ward");
}

#[test]
fn ward_counters_a_targeted_ability_too() {
    // an ability (no spell card): Hexing Squelcher's "pay 2 life" on a destroy ability at 4 life, and Frost Titan's mana
    let mut g = tbl();
    let ft = perm(&mut g, P1, "Frost Titan");
    removal::apply_removal(&mut g, Some(P0), ft, "destroy", None).unwrap();
    assert!(g.perm(ft).on_bf, "an ability that targets Frost Titan needs the ward {{2}} paid as well");
    lands(&mut g, P0, "Wastes", 2, false);
    removal::apply_removal(&mut g, Some(P0), ft, "destroy", None).unwrap();
    assert!(!g.perm(ft).on_bf);
    let sq = perm(&mut g, P1, "Hexing Squelcher");
    let life = g.player(P0).life;
    removal::apply_removal(&mut g, Some(P0), sq, "dmg9", None).unwrap();
    assert_eq!(g.player(P0).life, life - 2, "the ability's controller paid 2 life");
    assert!(!g.perm(sq).on_bf);
}

#[test]
fn ward_applies_to_a_steal_and_never_to_your_own_permanent() {
    let mut g = tbl();
    let ft = perm(&mut g, P1, "Frost Titan");
    assert!(!control::steal(&mut g, P0, ft, control::Duration::Permanent).unwrap(), "countered: no mana");
    assert_eq!(g.perm(ft).owner, P1);
    lands(&mut g, P0, "Wastes", 2, false);
    assert!(control::steal(&mut g, P0, ft, control::Duration::Permanent).unwrap());
    assert_eq!(g.perm(ft).owner, P0, "ward paid: the steal happens");
    // its new controller targeting it pays nothing
    assert!(sim_core::engine::ward::pay_ward(&mut g, P0, ft).unwrap());
}

#[test]
fn a_person_is_asked_whether_to_pay_a_ward() {
    // a person pays or declines the ward: the page asks, a "no" counters the effect
    let mut g = tbl();
    let ft = perm(&mut g, P1, "Frost Titan");
    lands(&mut g, P0, "Wastes", 2, false);
    seat_live(&mut g, P0, (0..8).map(|_| pick(1)).collect());
    assert!(!sim_core::engine::ward::pay_ward(&mut g, P0, ft).unwrap(), "the person declined to pay");
    assert!(g.player(P0).lands.iter().all(|&l| !g.perm(l).tapped));
    g.humans = Default::default();
    seat_live(&mut g, P0, (0..8).map(|_| pick(0)).collect());
    assert!(sim_core::engine::ward::pay_ward(&mut g, P0, ft).unwrap(), "the person paid");
    assert!(g.player(P0).lands.iter().all(|&l| g.perm(l).tapped));
}

// ======================================================== item 4: Mystic Forge, a counterspell on top
/// P1 casts a big spell; P0 has a Mystic Forge and Pact of Negation on top of the library (no cards in hand)
fn forge_game(seed: u64) -> Game {
    let mut g = table_with(&["karn", "kefka", "sauron"], seed, 40);
    perm(&mut g, P0, "Mystic Forge");
    lands(&mut g, P0, "Island", 5, false);
    g.player_mut(P0).hand.clear();
    let pact = take(&mut g, P0, "Pact of Negation");
    g.player_mut(P0).library.push(pact);
    lands(&mut g, P1, "Wastes", 8, false);
    g.active = Some(P1);
    g
}

#[test]
fn the_ai_counters_from_the_top_of_the_library_under_a_mystic_forge() {
    let pact_id = |g: &Game| card(g, "Pact of Negation");
    let mut countered = 0;
    for seed in 0..40 {
        let mut g = forge_game(seed);
        let pact = pact_id(&g);
        assert_eq!(sim_core::engine::stack::counter_on_top(&g, P0), Some(pact), "the Forge covers it");
        let ugin = hand(&mut g, P1, &["Ugin, the Spirit Dragon"])[0];
        sim_core::engine::cast::cast_card(&mut g, P1, ugin, "hand".into(), sim_core::state::Ctx::default()).unwrap();
        if g.player(P0).gy.contains(&pact) || g.player(P0).exile.contains(&pact) {
            countered += 1;
            assert!(!g.player(P0).library.contains(&pact) && !g.player(P0).hand.contains(&pact));
            assert!(!has_perm(&g, P1, "Ugin, the Spirit Dragon"), "countered");
        }
    }
    assert!(countered > 0, "the Pact on top was cast from the library in at least one game");
}

#[test]
fn without_a_forge_the_top_card_is_not_a_counterspell_option() {
    let mut g = forge_game(1);
    let forge = perms_named(&g, P0, "Mystic Forge")[0];
    sim_core::engine::zones::die(&mut g, forge, "destroy").unwrap();
    assert_eq!(sim_core::engine::stack::counter_on_top(&g, P0), None);
}

#[test]
fn a_person_can_cast_the_counterspell_from_the_top_in_response() {
    let mut g = forge_game(3);
    let ugin = hand(&mut g, P1, &["Ugin, the Spirit Dragon"])[0];
    let pact = card(&g, "Pact of Negation");
    // the legality check accepts the top card as a counterspell source (cost problem aside: Pact costs {0})
    let why = sim_core::human::legal::check_counter(&g, P0, pact, ugin);
    assert!(why.is_none(), "{why:?}");
    // and the page offers it
    let abs = sim_core::human::karn::perm_abilities(&g, P0, perms_named(&g, P0, "Mystic Forge")[0]);
    assert!(abs.iter().any(|a| a.label.contains("Pact of Negation")), "an ability to cast it from the top");
    assert!(sim_core::human::human::can_respond(&g, P0), "the response window opens for the top card");
}

// ======================================================== item 5: Peter Parker's Camera copies hand-written abilities
type Rule = (&'static str, Box<dyn Fn(&sim_core::control::Request) -> Answer + Send>);

/// a person who answers by what is asked: each rule is used once, the first whose text is in the prompt (or the kind);
/// anything else is passed or declined
fn scripted(rules: Vec<Rule>) -> ScriptHuman {
    let mut rules: Vec<Option<Rule>> = rules.into_iter().map(Some).collect();
    ScriptHuman::from_fn(move |r| {
        for slot in rules.iter_mut() {
            if slot.as_ref().is_some_and(|(k, _)| r.prompt.contains(k) || &*r.kind == *k) {
                let (_, f) = slot.take().unwrap();
                return f(r);
            }
        }
        if r.kind == "priority" { Answer::Action(json!({"do": "pass"})) } else { Answer::Decline }
    })
}

fn rule(key: &'static str, f: impl Fn(&sim_core::control::Request) -> Answer + Send + 'static) -> Rule {
    (key, Box::new(f))
}

fn first(_: &sim_core::control::Request) -> Answer {
    Answer::Pick(0)
}

fn index_of(g: &Game, p: PlayerId, m: PermId) -> usize {
    g.player(p).perms.iter().position(|&x| x == m).unwrap()
}

fn films(g: &Game, cam: PermId) -> i64 {
    g.perm(cam).data.int(sim_core::state::DataKey::Counters)
}

#[test]
fn a_hand_written_ability_split_from_its_cost_is_run_again_for_a_camera_copy() {
    let mut g = tbl();
    lands(&mut g, P0, "Island", 2, false);
    let cam = perm(&mut g, P0, "Peter Parker's Camera");
    let cd = card(&g, "Barad-dûr");
    let n = std::cell::Cell::new(0);
    let worth = |w: f64, n: &std::cell::Cell<i32>, g: &mut Game| {
        sim_core::engine::stack::ability_run_w(g, P0, None, Some(cd), "amass", None, None, w, |_g, _| {
            n.set(n.get() + 1);
            Ok(())
        })
        .unwrap()
    };
    // worth 0: the AI never spends a film counter on it
    assert!(worth(0.0, &n, &mut g));
    assert_eq!((n.get(), films(&g, cam), g.perm(cam).tapped), (1, 3, false));
    // worth 3: the copy resolves first, the effect ran twice, the Camera was paid for once
    n.set(0);
    assert!(worth(3.0, &n, &mut g));
    assert_eq!(n.get(), 2, "the copy and the original");
    assert_eq!((films(&g, cam), g.perm(cam).tapped), (2, true));
    assert!(g.player(P0).lands.iter().all(|&l| g.perm(l).tapped), "{{2}} paid");
}

#[test]
fn the_ai_camera_copies_its_expedition_map() {
    let mut copied = 0;
    for seed in 0..20 {
        let mut g = table_with(&["sauron", "kefka", "karn"], seed, 40);
        lands(&mut g, P0, "Island", 2, false);
        perm(&mut g, P0, "Sol Ring");
        let cam = perm(&mut g, P0, "Peter Parker's Camera");
        let map = perm(&mut g, P0, "Expedition Map");
        let tomb = card(&g, "Ancient Tomb");
        g.player_mut(P0).library.push(tomb);
        g.player_mut(P0).hand.clear();
        g.active = Some(P0);
        g.step = sim_core::state::Step::Main1;
        let cd = g.perm(map).cd.unwrap();
        let f = g.registry.get(cd).unwrap().options.unwrap();
        let o = f(&mut g, map, P0, Some(false)).unwrap();
        let Some(act) = o.first().and_then(|x| x.act.as_ref()) else { continue };
        sim_core::ai::act::perform(&mut g, P0, act).unwrap();
        let lands_in_hand = g.player(P0).hand.iter().filter(|&&c| g.db.get(c).land).count();
        assert!(!g.perm(map).on_bf, "sacrificed");
        if g.perm(cam).tapped {
            copied += 1;
            assert_eq!(lands_in_hand, 2, "the copy fetched a second land");
            assert_eq!(films(&g, cam), 2);
        } else {
            assert_eq!(lands_in_hand, 1);
        }
    }
    assert!(copied > 0, "the AI used the Camera on its Expedition Map (a land tutor) in some game");
}

#[test]
fn a_person_copies_expedition_map_mind_stone_and_rogues_passage_with_the_camera() {
    // Expedition Map: two lands to the hand
    let mut g = tbl();
    let cam = perm(&mut g, P0, "Peter Parker's Camera");
    let map = perm(&mut g, P0, "Expedition Map");
    g.player_mut(P0).hand.clear();
    pool(&mut g, P0, &[('C', 4)]);
    let (ci, mi) = (index_of(&g, P0, cam), index_of(&g, P0, map));
    let h = Arc::new(scripted(vec![
        rule("You activate", move |_| Answer::Action(json!({"do": "use", "perm": ci}))),
        rule("which ability", first),
        rule("which ability", first),
        rule("copy which", first),
        rule("Search for a land", first),
        rule("Search for a land", first),
    ]));
    g.humans.add(P0, h.clone());
    let why = sim_core::human::human::apply(&mut g, P0, &json!({"do": "use", "perm": mi})).unwrap();
    assert!(why.is_none(), "{why:?}");
    assert_eq!(g.player(P0).hand.iter().filter(|&&c| g.db.get(c).land).count(), 2, "the copy and the original");
    assert_eq!((films(&g, cam), g.perm(cam).tapped), (2, true));
    assert_eq!(pool_total(&g, P0), 0);

    // Mind Stone: two cards
    let mut g = tbl();
    let cam = perm(&mut g, P0, "Peter Parker's Camera");
    let stone = perm(&mut g, P0, "Mind Stone");
    g.player_mut(P0).hand.clear();
    pool(&mut g, P0, &[('C', 3)]);
    let (ci, si) = (index_of(&g, P0, cam), index_of(&g, P0, stone));
    g.humans.add(
        P0,
        Arc::new(scripted(vec![
            rule("You activate", move |_| Answer::Action(json!({"do": "use", "perm": ci}))),
            rule("which ability", first),
            rule("which ability", first),
            rule("copy which", first),
        ])),
    );
    let why = sim_core::human::human::apply(&mut g, P0, &json!({"do": "use", "perm": si})).unwrap();
    assert!(why.is_none(), "{why:?}");
    assert_eq!(g.player(P0).hand.len(), 2, "drew for the copy and for the original");
    assert!(!g.perm(stone).on_bf);

    // Rogue's Passage: the copy may take a new target, so two creatures can't be blocked
    let mut g = tbl();
    let cam = perm(&mut g, P0, "Peter Parker's Camera");
    let passage = lands(&mut g, P0, "Rogue's Passage", 1, false)[0];
    let a = perm(&mut g, P0, "Serra Angel");
    let b = perm(&mut g, P0, "Grave Titan");
    pool(&mut g, P0, &[('C', 6)]);
    let ci = index_of(&g, P0, cam);
    g.humans.add(
        P0,
        Arc::new(scripted(vec![
            rule("You activate", move |_| Answer::Action(json!({"do": "use", "perm": ci}))),
            rule("which ability", first),
            rule("copy which", first),
            // the first choice is the Serra Angel (for the original), the copy takes Grave Titan
            rule("which creature can't be blocked", |r| {
                Answer::Pick(r.choices.iter().position(|c| c.contains("Serra")).unwrap())
            }),
            rule("(copy)", |r| Answer::Pick(r.choices.iter().position(|c| c.contains("Grave Titan")).unwrap())),
        ])),
    );
    let ab = sim_core::human::abilities::land_abilities(&g, P0, passage);
    let act = &ab.iter().find(|x| x.label.contains("can't be blocked")).expect("Passage's ability").act;
    act(&mut g, P0, passage).unwrap();
    let st = g.turn_stamp();
    let unbl = |g: &Game, m: PermId| {
        g.perm(m).data.get(sim_core::state::DataKey::Unbl) == Some(&sim_core::state::Val::Stamp(st))
    };
    assert!(unbl(&g, a) && unbl(&g, b), "both can't be blocked this turn");
    assert!(g.perm(cam).tapped);
}

// ======================================================== item 6: Marvin borrows hand-written creature abilities
const MARVIN: &str = "Marvin, Murderous Mimic";

fn marvin_opts(g: &mut Game, marvin: PermId, post: Option<bool>) -> Vec<sim_core::hooks::Opt> {
    let c = g.perm(marvin).cd.unwrap();
    let f = g.registry.get(c).unwrap().options.unwrap();
    f(g, marvin, P0, post).unwrap()
}

fn ready(g: &mut Game, m: PermId) {
    let x = g.perm_mut(m);
    x.sick = false;
    x.tapped = false;
}

#[test]
fn marvin_uses_kefkas_8_ability_and_nothing_transforms() {
    let mut g = tbl();
    g.active = Some(P0);
    g.step = sim_core::state::Step::Main1;
    let marvin = perm(&mut g, P0, MARVIN);
    let kefka = perm(&mut g, P0, "Kefka, Court Mage // Kefka, Ruler of Ruin");
    ready(&mut g, marvin);
    assert!(marvin_opts(&mut g, marvin, Some(false)).iter().all(|o| !o.label.contains("Kefka")), "no {{8}}");
    lands(&mut g, P0, "Swamp", 8, false);
    let o = marvin_opts(&mut g, marvin, Some(false));
    let k = o.iter().find(|o| o.label.contains("Kefka")).expect("Marvin offers Kefka's ability");
    lands(&mut g, P1, "Island", 3, false);
    lands(&mut g, P2, "Island", 3, false);
    let opp_perms = g.player(P1).perms.len() + g.player(P1).lands.len();
    assert!(sim_core::ai::act::perform(&mut g, P0, k.act.as_ref().unwrap()).unwrap());
    let after = g.player(P1).perms.len() + g.player(P1).lands.len();
    assert!(after < opp_perms, "an opponent sacrificed a permanent");
    // rule 201.5: "transform Kefka" names the creature with the ability (Marvin), and Marvin is not a double-faced card (701.28c)
    assert!(!g.perm(kefka).data.truthy(sim_core::state::DataKey::Ruin), "Kefka does not transform");
    assert!(!g.perm(marvin).data.truthy(sim_core::state::DataKey::Ruin), "not Marvin");
    assert_eq!(sim_core::engine::values::epow(&g, marvin), 2);
    // Kefka, still untransformed, keeps giving Marvin the ability; once transformed it has no activated ability
    lands(&mut g, P0, "Swamp", 8, false);
    assert!(marvin_opts(&mut g, marvin, Some(false)).iter().any(|o| o.label.contains("Kefka")), "still offered");
    g.perm_mut(kefka).data.set(sim_core::state::DataKey::Ruin, sim_core::state::Val::Bool(true));
    assert!(marvin_opts(&mut g, marvin, Some(false)).iter().all(|o| !o.label.contains("Kefka")));
}

#[test]
fn marvin_pays_for_priests_and_timberwatchs_hand_written_abilities() {
    let mut g = tbl();
    g.active = Some(P0);
    g.step = sim_core::state::Step::Main1;
    let marvin = perm(&mut g, P0, MARVIN);
    perm(&mut g, P0, "Priest of Forgotten Gods");
    let (a, b) = (perm(&mut g, P0, "Caged Zombie"), perm(&mut g, P0, "Llanowar Elves"));
    for m in [marvin, a, b] {
        ready(&mut g, m);
    }
    let o = marvin_opts(&mut g, marvin, Some(true));
    let pr = o.iter().find(|o| o.label.contains("Priest")).expect("Marvin offers the Priest's ability");
    let life = g.player(P1).life;
    assert!(sim_core::ai::act::perform(&mut g, P0, pr.act.as_ref().unwrap()).unwrap());
    assert!(g.perm(marvin).tapped, "Marvin paid the {{T}}");
    assert!(!g.perm(a).on_bf && !g.perm(b).on_bf, "two other creatures sacrificed");
    assert_eq!(g.player(P1).life, life - 2);

    // Timberwatch Elf: {T}: target creature gets +X/+X
    let mut g = tbl();
    g.active = Some(P0);
    g.step = sim_core::state::Step::Main1;
    let marvin = perm(&mut g, P0, MARVIN);
    let tw = perm(&mut g, P0, "Timberwatch Elf");
    let t = perm(&mut g, P0, "Serra Angel");
    for m in [marvin, tw, t] {
        ready(&mut g, m);
    }
    g.perm_mut(tw).tapped = true; // the Elf itself is tapped: it is Marvin who uses the ability
    let before = sim_core::engine::values::epow(&g, t);
    let o = marvin_opts(&mut g, marvin, Some(false));
    let o = o.iter().find(|o| o.label.contains("Timberwatch")).expect("Marvin offers Timberwatch's ability");
    assert!(sim_core::ai::act::perform(&mut g, P0, o.act.as_ref().unwrap()).unwrap());
    assert!(g.perm(marvin).tapped);
    assert!(sim_core::engine::values::epow(&g, t) > before, "+X/+X");
}

#[test]
fn marvin_is_a_sacrifice_outlet_for_a_carrion_feeder_and_taps_for_a_dorks_mana() {
    let mut g = tbl();
    let marvin = perm(&mut g, P0, MARVIN);
    perm(&mut g, P0, "Carrion Feeder");
    let fodder = perm(&mut g, P0, "Llanowar Elves");
    assert!(sim_core::impls::common::outlets(&g, P0).contains(&marvin), "Marvin has the sacrifice ability");
    sim_core::impls::common::sac_through(&mut g, P0, marvin, fodder).unwrap();
    assert!(!g.perm(fodder).on_bf);
    assert_eq!(g.perm(marvin).plus, 1, "Carrion Feeder's +1/+1 counter went on Marvin");

    // mana: Marvin taps for the {G} of a Llanowar Elves (which is itself tapped)
    let mut g = tbl();
    let marvin = perm(&mut g, P0, MARVIN);
    let elf = perm(&mut g, P0, "Llanowar Elves");
    ready(&mut g, marvin);
    g.perm_mut(elf).tapped = true;
    let units = sim_core::engine::mana::mana_units(&g, P0, false);
    assert!(units.iter().any(|u| u.src == sim_core::engine::mana::Source::Perm(marvin)), "Marvin makes mana");
    assert!(sim_core::engine::mana::can_pay(&g, P0, 0, "G", false), "{{G}} from Marvin");
    sim_core::engine::mana::pay(&mut g, P0, 0, "G", false).unwrap();
    assert!(g.perm(marvin).tapped);
    // summoning sick: no mana
    let mut g = tbl();
    let marvin = perm(&mut g, P0, MARVIN);
    perm(&mut g, P0, "Llanowar Elves");
    g.perm_mut(marvin).sick = true;
    let units = sim_core::engine::mana::mana_units(&g, P0, false);
    assert!(!units.iter().any(|u| u.src == sim_core::engine::mana::Source::Perm(marvin)));
}

#[test]
fn marvin_has_quirion_rangers_untap_ability() {
    use sim_core::engine::mana::{Source, mana_units};
    let mut g = tbl();
    let marvin = perm(&mut g, P0, MARVIN);
    let druid = perm(&mut g, P0, "Circle of Dreams Druid");
    perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Llanowar Elves");
    lands(&mut g, P0, "Forest", 1, false);
    for m in [marvin, druid] {
        ready(&mut g, m);
    }
    g.perm_mut(druid).tapped = true; // the Druid made its mana already
    let quirion =
        |g: &Game| mana_units(g, P0, false).iter().any(|u| matches!(u.src, Source::Quirion(s, _) if s == marvin));
    assert!(!quirion(&g), "no Quirion Ranger");
    perm(&mut g, P0, "Quirion Ranger");
    assert!(quirion(&g), "Marvin returns a Forest to untap the Druid");
}
