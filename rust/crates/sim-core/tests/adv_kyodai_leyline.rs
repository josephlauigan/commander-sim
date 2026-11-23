//! Adversarial probes for Kyodai, Leyline of the Guildpact, hybrid costs and Shape::Alt (written by Sol, 2026-11-22). Oracle
//! expectations were read from data/scryfall_cache.json. A probe marked `#[ignore = "FAILS: ..."]` is a known gap that is
//! recorded in the cards' notes in data/card_overrides.toml (it is not a regression); remove the ignore when the gap is fixed.
mod play_common;
use play_common::*;
use sim_core::cards::Colors;
use sim_core::engine::control::{self, Duration};
use sim_core::engine::mana::{self, Shape, Source};
use sim_core::engine::values::{colors_of, epow, etgh, indestructible};
use sim_core::engine::{cast, chars, life, removal, stack, zones};
use sim_core::ids::CardId;
use sim_core::state::Ctx;
use std::sync::{Arc, Mutex};
const KYODAI: &str = "Kyodai, Soul of Kamigawa";
const LEYLINE: &str = "Leyline of the Guildpact";
const ASK: &str = "Kyodai: which permanent gains indestructible?";
fn game() -> Game {
    table(&["thanos", "seph"])
}
fn leyline(g: &mut Game, p: PlayerId) -> PermId {
    perm(g, p, LEYLINE)
}
fn paid_cast(g: &mut Game, p: PlayerId, c: CardId, zone: &'static str) {
    let (gn, pi) = mana::cost_of(g, p, c);
    assert!(mana::pay_spell(g, p, c, gn, &pi, false).unwrap(), "payment for {}: {gn} {pi}", g.db.get(c).name);
    assert!(cast::cast_card(g, p, c, zone.into(), Ctx::default()).unwrap(), "cast {}", g.db.get(c).name);
}
fn choose(g: &mut Game, p: PlayerId, word: &'static str) -> Arc<ScriptHuman> {
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if r.prompt.contains(ASK) {
            Answer::Pick(r.choices.iter().position(|c| c.contains(word)).expect("target offered"))
        } else if &*r.kind == "priority" {
            Answer::Action(json!({"do":"pass"}))
        } else {
            Answer::Decline
        }
    }));
    g.humans.add(p, h.clone());
    h
}
/// Kyodai's target must be announced before anyone responds to the triggered ability.
#[test]
#[ignore = "FAILS: trigger chooses target after the response window"]
fn kyodai_announces_target_before_priority() {
    let mut g = game();
    perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Sol Ring");
    let events = Arc::new(Mutex::new(Vec::new()));
    let seen = events.clone();
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if r.prompt.contains(ASK) {
            seen.lock().unwrap().push("target");
            Answer::Pick(0)
        } else if &*r.kind == "priority" {
            seen.lock().unwrap().push("priority");
            Answer::Action(json!({"do":"pass"}))
        } else {
            Answer::Decline
        }
    }));
    h.set_autopass("all");
    g.humans.add(P0, h);
    perm(&mut g, P0, KYODAI);
    let got = events.lock().unwrap();
    assert_eq!(got.first(), Some(&"target"), "target must precede response window: {got:?}");
}
/// A chosen target leaving makes the ability fizzle, without selecting a replacement on resolution.
#[test]
#[ignore = "FAILS: departed target is replaced on resolution"]
fn kyodai_removed_target_does_not_retarget() {
    let mut g = game();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let ring = perm(&mut g, P0, "Sol Ring");
    hand(&mut g, P0, &["Swords to Plowshares"]);
    pool(&mut g, P0, &[('W', 1)]);
    let mut fired = false;
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if &*r.kind == "priority" && !fired {
            fired = true;
            return Answer::Action(json!({"do":"cast","card":0}));
        }
        if r.prompt.contains("Swords") && &*r.kind == "target" {
            return Answer::Pick(r.choices.iter().position(|c| c.contains("Elves")).unwrap());
        }
        if r.prompt.contains(ASK) {
            return Answer::Pick(
                r.choices
                    .iter()
                    .position(|c| c.contains("Elves"))
                    .unwrap_or_else(|| r.choices.iter().position(|c| c.contains("Sol Ring")).unwrap()),
            );
        }
        if &*r.kind == "priority" { Answer::Action(json!({"do":"pass"})) } else { Answer::Decline }
    }));
    h.set_autopass("all");
    g.humans.add(P0, h);
    perm(&mut g, P0, KYODAI);
    assert!(!g.perm(elf).on_bf, "response exiled intended target");
    assert!(!indestructible(&g, ring), "illegal original target cannot be replaced by Sol Ring");
}
/// A resolved grant depends on controlling Kyodai, not on Kyodai retaining its abilities.
#[test]
#[ignore = "FAILS: ability loss incorrectly ends an already resolved grant"]
fn kyodai_ability_loss_keeps_resolved_grant() {
    let mut g = game();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let k = perm(&mut g, P0, KYODAI);
    assert!(indestructible(&g, elf));
    g.perm_mut(k).neutered = true;
    assert!(indestructible(&g, elf), "ability loss does not end a resolved duration effect");
}
/// Losing control ends the original duration permanently even if control is regained.
#[test]
#[ignore = "FAILS: ended grant resumes after control is regained"]
fn kyodai_control_round_trip_does_not_revive_grant() {
    let mut g = game();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let k = perm(&mut g, P0, KYODAI);
    assert!(control::gain_control(&mut g, P1, k, Duration::Permanent));
    assert!(!indestructible(&g, elf));
    assert!(control::gain_control(&mut g, P0, k, Duration::Permanent));
    assert!(!indestructible(&g, elf), "ended duration must not restart");
}
/// Two separate resolved grants on one target are independent.
#[test]
fn two_kyodai_grants_one_target_survives_first_source_leaving() {
    let mut g = game();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let a = perm(&mut g, P0, KYODAI);
    perm(&mut g, P1, "Sol Ring");
    choose(&mut g, P1, "Elves");
    let b = perm(&mut g, P1, KYODAI);
    zones::exile_perm(&mut g, a).unwrap();
    assert!(indestructible(&g, elf), "second grant remains");
    zones::exile_perm(&mut g, b).unwrap();
    assert!(!indestructible(&g, elf));
}
/// Indestructible stops destroy and lethal damage, but not sacrifice, exile or zero toughness.
#[test]
fn kyodai_indestructible_removal_matrix() {
    for cause in ["destroy", "damage", "sacrifice", "exile", "zero"] {
        let mut g = game();
        let elf = perm(&mut g, P0, "Llanowar Elves");
        perm(&mut g, P0, KYODAI);
        match cause {
            "destroy" => {
                let c = card(&g, "Murder");
                removal::apply_removal(&mut g, Some(P1), elf, "destroy", Some(c)).unwrap();
            }
            "damage" => {
                let dealt = life::removal_damage(&mut g, P1, None, elf, 100, None).unwrap();
                assert_eq!(dealt, 100);
            }
            "sacrifice" => zones::die(&mut g, elf, "sac".into()).unwrap(),
            "exile" => zones::exile_perm(&mut g, elf).unwrap(),
            _ => {
                g.perm_mut(elf).eot_pt = (0, -10);
                life::check_state(&mut g).unwrap();
            }
        }
        assert_eq!(g.perm(elf).on_bf, matches!(cause, "destroy" | "damage"), "{cause}");
    }
}
/// Command-zone recasts add two mana tax each time and create fresh grants.
#[test]
fn kyodai_four_commander_casts_tax_and_grants() {
    let mut g = game();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    lands(&mut g, P0, "Plains", 12, false);
    let c = g.player(P0).cmd;
    let mut ids = Vec::new();
    for n in 0..4 {
        for l in g.player(P0).lands.clone() {
            g.perm_mut(l).tapped = false;
        }
        let (gn, pi) = mana::cost_of(&g, P0, c);
        assert_eq!(gn + pi.len() as u32, 4 + 2 * n, "commander total cost");
        paid_cast(&mut g, P0, c, "cmd");
        let k = perms_named(&g, P0, KYODAI)[0];
        assert!(!ids.contains(&k));
        ids.push(k);
        assert!(indestructible(&g, elf));
        zones::die(&mut g, k, "sac".into()).unwrap();
        assert!(g.player(P0).cmd_in_zone);
        assert!(!indestructible(&g, elf));
    }
}
/// Flash permits casting Kyodai during an opponent's end step.
#[test]
fn kyodai_cast_in_opponents_end_step() {
    let mut g = game();
    g.active = Some(P1);
    g.step = sim_core::state::Step::End;
    lands(&mut g, P0, "Plains", 4, false);
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let c = hand(&mut g, P0, &[KYODAI])[0];
    assert!(cast::castable(&g, P0, c, "hand"));
    paid_cast(&mut g, P0, c, "hand");
    assert!(indestructible(&g, elf));
}
/// Each WUBRG activation adds five power/toughness and consumes five lands once.
#[test]
fn kyodai_many_pumps_with_leyline_lands() {
    let mut g = game();
    let k = perm(&mut g, P0, KYODAI);
    leyline(&mut g, P0);
    lands(&mut g, P0, "Island", 50, false);
    for n in 1..=10 {
        let list = sim_core::human::human::abilities_of(&mut g, P0, k).unwrap();
        let (_, sim_core::human::human::AbilityKind::Hook(a)) =
            list.into_iter().find(|(l, _)| l.contains("pump")).unwrap()
        else {
            panic!("pump")
        };
        assert!(sim_core::ai::act::perform(&mut g, P0, &a).unwrap());
        assert_eq!((epow(&g, k), etgh(&g, k)), (3 + 5 * n, 3 + 5 * n));
    }
    assert!(!mana::can_pay(&g, P0, 0, "WUBRG", false));
}
/// Colours and types follow the current controller's Leyline on either seat.
#[test]
fn leyline_controller_changes_and_both_seats() {
    for p in [P0, P1] {
        let q = if p == P0 { P1 } else { P0 };
        let mut g = game();
        leyline(&mut g, p);
        let elf = perm(&mut g, q, "Llanowar Elves");
        let land = lands(&mut g, q, "Island", 1, false)[0];
        assert_eq!(colors_of(&g, elf), Colors::from_letters("G"));
        control::gain_control(&mut g, p, elf, Duration::Permanent);
        control::gain_control(&mut g, p, land, Duration::Permanent);
        assert_eq!(colors_of(&g, elf), Colors::from_letters("WUBRG"));
        assert_eq!(chars::land_types(&g, land), Colors::from_letters("WUBRG"));
        control::gain_control(&mut g, q, elf, Duration::Permanent);
        assert_eq!(colors_of(&g, elf), Colors::from_letters("G"));
    }
}
/// Elder counts its controller's colours, with and without Leyline, on either seat.
#[test]
fn elder_pt_and_mana_on_both_seats() {
    for p in [P0, P1] {
        let mut g = game();
        let e = perm(&mut g, p, "Faeburrow Elder");
        assert_eq!((epow(&g, e), etgh(&g, e)), (2, 2));
        let u = mana::mana_units(&g, p, false).into_iter().find(|u| u.src == Source::Perm(e)).unwrap();
        assert_eq!(u.amt, 2);
        leyline(&mut g, p);
        assert_eq!((epow(&g, e), etgh(&g, e)), (5, 5));
        let u = mana::mana_units(&g, p, false).into_iter().find(|u| u.src == Source::Perm(e)).unwrap();
        assert_eq!((u.amt, u.cols), (5, Colors::from_letters("WUBRG")));
    }
}
/// Permanent recolouring never changes devotion; Leyline's GU symbol contributes one blue.
#[test]
#[ignore = "FAILS: Leyline GU hybrid contributes zero blue devotion"]
fn leyline_devotion_is_printed_symbols_only() {
    let mut g = game();
    perm(&mut g, P0, "Serra Angel");
    perm(&mut g, P0, "Sol Ring");
    let l = leyline(&mut g, P0);
    assert_eq!(sim_core::impls::rules2::devotion(&g, P0, 'U'), 1, "only Leyline's GU symbol contributes blue");
    zones::bounce(&mut g, l).unwrap();
    assert_eq!(sim_core::impls::rules2::devotion(&g, P0, 'U'), 0);
}
/// A hybrid symbol contributes devotion to each of its colours, regardless of payment.
#[test]
#[ignore = "FAILS: devotion ignores second half of hybrid symbols"]
fn hybrid_devotion_counts_both_halves() {
    let mut g = game();
    perm(&mut g, P0, "Kitchen Finks");
    assert_eq!(sim_core::impls::rules2::devotion(&g, P0, 'G'), 2);
    assert_eq!(sim_core::impls::rules2::devotion(&g, P0, 'W'), 2, "two GW symbols contribute two white devotion");
}
/// All five protections prevent damage from a Leyline-recoloured nonland source.
#[test]
fn leyline_damage_respects_all_five_protections() {
    for c in "WUBRG".chars() {
        let mut g = game();
        leyline(&mut g, P0);
        let src = perm(&mut g, P0, "Llanowar Elves");
        let t = token(&mut g, P1, 2);
        g.perm_mut(t).eot_kw.push(sim_core::sym::intern(&format!("pro_{c}")));
        assert_eq!(life::creature_damage(&g, P0, Some(src), t, 1), 0, "protection from {c}");
    }
}
/// Leyline affects permanents, so a green spell must not become red for Firewalker.
#[test]
fn leyline_does_not_recolour_spells_for_firewalker() {
    let mut g = game();
    leyline(&mut g, P0);
    perm(&mut g, P1, "Kor Firewalker");
    lands(&mut g, P0, "Forest", 1, false);
    let c = hand(&mut g, P0, &["Llanowar Elves"])[0];
    let before = g.player(P1).life;
    paid_cast(&mut g, P0, c, "hand");
    assert_eq!(g.player(P1).life, before);
}
/// Hybrid spells remain both colours on the stack, including when paid with one colour.
#[test]
fn hybrid_spell_colours_for_blast_counterspells() {
    let mut g = game();
    let c = hand(&mut g, P0, &["Yorion, Sky Nomad"])[0];
    lands(&mut g, P0, "Plains", 5, false);
    let _ = mana::cost_of(&g, P0, c);
    for n in ["Pyroblast", "Red Elemental Blast"] {
        assert!(stack::counter_ok(&g, card(&g, n), c), "Yorion is blue when paid WW: {n}");
    }
    leyline(&mut g, P0);
    assert!(
        !stack::counter_ok(&g, card(&g, "Pyroblast"), card(&g, "Llanowar Elves")),
        "Leyline does not colour spells"
    );
}
/// Every assignment of each Oracle hybrid symbol must be payable, preserving generic mana.
#[test]
fn hybrid_all_assignments_across_catalog_cards() {
    let oracle: Value =
        serde_json::from_str(&std::fs::read_to_string(data_dir().join("scryfall_cache.json")).unwrap()).unwrap();
    for name in [
        LEYLINE,
        "Kitchen Finks",
        "Deathrite Shaman",
        "Manamorphose",
        "Dovin, Hand of Control",
        "Yorion, Sky Nomad",
        "Vibrance",
        "Steel of the Godhead",
    ] {
        let d = &oracle[name.to_lowercase()];
        let cost = d["mana_cost"].as_str().expect("Oracle cost cached");
        let symbols: Vec<&str> = cost.split('{').skip(1).map(|s| s.split('}').next().unwrap()).collect();
        let hybrid: Vec<Vec<char>> = symbols
            .iter()
            .filter(|s| s.contains('/'))
            .map(|s| s.chars().filter(|c| c.is_ascii_alphabetic()).collect())
            .collect();
        let generic: u32 = symbols.iter().filter_map(|s| s.parse::<u32>().ok()).sum();
        for mask in 0..1usize << hybrid.len() {
            let mut g = game();
            let c = hand(&mut g, P0, &[name])[0];
            for (i, pair) in hybrid.iter().enumerate() {
                let land = match pair[(mask >> i) & 1] {
                    'W' => "Plains",
                    'U' => "Island",
                    'B' => "Swamp",
                    'R' => "Mountain",
                    'G' => "Forest",
                    _ => unreachable!(),
                };
                lands(&mut g, P0, land, 1, false);
            }
            if name == "Steel of the Godhead" {
                token(&mut g, P0, 2);
            }
            lands(&mut g, P0, "Wastes", generic as usize, false);
            let (gn, pi) = mana::cost_of(&g, P0, c);
            assert!(mana::pay_spell(&mut g, P0, c, gn, &pi, false).unwrap(), "{name}, assignment {mask}: {gn} {pi}");
            assert!(cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap(), "cast {name}, assignment {mask}");
        }
    }
}
/// Petal, treasure, Chrome Mox imprint and rainbow lands can fund the hybrid cost.
#[test]
fn hybrid_nonland_mana_sources() {
    for kind in ["petal", "treasure", "mox", "rainbow"] {
        let mut g = game();
        let c = hand(&mut g, P0, &[LEYLINE])[0];
        match kind {
            "petal" => {
                for _ in 0..4 {
                    perm(&mut g, P0, "Lotus Petal");
                }
            }
            "treasure" => g.player_mut(P0).treasures = 4,
            "mox" => {
                for _ in 0..4 {
                    hand(&mut g, P0, &[LEYLINE]);
                    perm(&mut g, P0, "Chrome Mox");
                }
            }
            _ => {
                lands(&mut g, P0, "City of Brass", 4, false);
            }
        }
        let (gn, pi) = mana::cost_of(&g, P0, c);
        assert!(mana::pay_spell(&mut g, P0, c, gn, &pi, false).unwrap(), "{kind}: {pi}");
    }
}
/// Hybrid assignments work on graveyard/exile casting paths as on hand casts.
#[test]
fn hybrid_graveyard_and_exile_cast_paths() {
    for zone in ["gy", "exile"] {
        let mut g = game();
        let c = card(&g, "Kitchen Finks");
        if zone == "gy" {
            perm(&mut g, P0, "Muldrotha, the Gravetide");
            g.player_mut(P0).gy.push(c);
        } else {
            g.player_mut(P0).exile.push(c);
        }
        lands(&mut g, P0, "Plains", 3, false);
        paid_cast(&mut g, P0, c, zone);
        assert!(has_perm(&g, P0, "Kitchen Finks"));
    }
}
/// Vibrance checks all mana spent: RRGGG satisfies both RR and GG.
#[test]
#[ignore = "FAILS: Vibrance ignores colours used for generic mana and cannot trigger both abilities"]
fn vibrance_generic_colours_can_trigger_both_abilities() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 2, false);
    lands(&mut g, P0, "Forest", 3, false);
    let c = hand(&mut g, P0, &["Vibrance"])[0];
    let before = g.player(P0).life;
    let opp = g.player(P1).life;
    paid_cast(&mut g, P0, c, "hand");
    assert_eq!(g.player(P1).life, opp - 3);
    assert_eq!(g.player(P0).life, before + 2, "GG spent toward generic also triggers green ability");
}
/// Pricing another game/search branch cannot alter an already-paid spell's resolution.
#[test]
#[ignore = "FAILS: cost estimate in another game overwrites actual paid colours"]
fn vibrance_pricing_other_game_does_not_rewrite_paid_cast() {
    let mut g = game();
    lands(&mut g, P0, "Mountain", 5, false);
    let c = hand(&mut g, P0, &["Vibrance"])[0];
    let (gn, pi) = mana::cost_of(&g, P0, c);
    assert!(mana::pay_spell(&mut g, P0, c, gn, &pi, false).unwrap());
    let mut other = game();
    lands(&mut other, P0, "Forest", 5, false);
    let _ = mana::cost_of(&other, P0, c);
    let before = g.player(P0).life;
    let opp = g.player(P1).life;
    assert!(cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap());
    assert_eq!(g.player(P1).life, opp - 3, "other game's estimate cannot change red-paid Vibrance");
    assert_eq!(g.player(P0).life, before);
}
/// Moving a paid game state to a new thread must not change its meaning.
#[test]
#[ignore = "FAILS: paid colours are thread-local rather than part of the game"]
fn vibrance_paid_state_is_thread_independent() {
    let mut g = game();
    lands(&mut g, P0, "Forest", 5, false);
    let c = hand(&mut g, P0, &["Vibrance"])[0];
    let (gn, pi) = mana::cost_of(&g, P0, c);
    assert!(mana::pay_spell(&mut g, P0, c, gn, &pi, false).unwrap());
    let before = g.player(P0).life;
    let after = std::thread::spawn(move || {
        cast::cast_card(&mut g, P0, c, "hand", Ctx::default()).unwrap();
        g.player(P0).life
    })
    .join()
    .unwrap();
    assert_eq!(after, before + 2, "GG payment must survive moving the game to another thread");
}
/// Workshop's gained basic-type abilities do not inherit its printed artifact restriction.
#[test]
#[ignore = "FAILS: printed Workshop restriction incorrectly applies to gained basic-type mana"]
fn workshop_leyline_ability_pays_nonartifact_spell() {
    let mut g = game();
    leyline(&mut g, P0);
    lands(&mut g, P0, "Mishra's Workshop", 1, false);
    let c = hand(&mut g, P0, &["Llanowar Elves"])[0];
    let (gn, pi) = mana::cost_of(&g, P0, c);
    assert!(
        mana::pay_spell(&mut g, P0, c, gn, &pi, false).unwrap(),
        "Workshop's Forest ability has no artifact restriction"
    );
}
/// A Karoo retains BG and gains each basic type's one-mana ability.
#[test]
#[ignore = "FAILS: Karoo does not gain the Leyline mana alternative"]
fn karoo_leyline_can_make_blue() {
    let mut g = game();
    leyline(&mut g, P0);
    lands(&mut g, P0, "Golgari Rot Farm", 1, false);
    assert!(mana::pay(&mut g, P0, 0, "U", false).unwrap(), "Rot Farm has an Island ability");
    assert_eq!(g.player(P0).mana.total(), 0);
}
/// A zero-creature Cradle still has basic-type abilities under Leyline.
#[test]
#[ignore = "FAILS: zero-creature Cradle loses its gained basic-type ability"]
fn empty_cradle_still_has_leyline_ability() {
    let mut g = game();
    leyline(&mut g, P0);
    lands(&mut g, P0, "Gaea's Cradle", 1, false);
    assert!(mana::pay(&mut g, P0, 0, "U", false).unwrap(), "zero creatures must not remove Island ability");
}
/// Coffers can use its Island ability without the printed ability's two-mana activation.
#[test]
#[ignore = "FAILS: zero-yield Coffers loses its gained basic-type ability"]
fn coffers_leyline_free_blue_with_no_other_sources() {
    let mut g = game();
    leyline(&mut g, P0);
    lands(&mut g, P0, "Cabal Coffers", 1, false);
    assert!(mana::pay(&mut g, P0, 0, "U", false).unwrap(), "Island ability requires no 2-mana activation");
}
/// Damping Sphere replaces CC but not the separate single-blue ability.
#[test]
#[ignore = "FAILS: Damping Sphere removes gained single-colour ability"]
fn sphere_preserves_tombs_single_blue_ability() {
    let mut g = game();
    leyline(&mut g, P0);
    lands(&mut g, P0, "Ancient Tomb", 1, false);
    perm(&mut g, P1, "Damping Sphere");
    assert!(mana::pay(&mut g, P0, 0, "U", false).unwrap(), "Sphere only replaces two or more mana");
    assert_eq!(g.player(P0).life, 40);
}
/// Nyxbloom triples the gained Island ability: UUU without Tomb damage.
#[test]
#[ignore = "FAILS: Alt gained basic-type ability is not tripled"]
fn nyxbloom_triples_tombs_blue_ability_without_pain() {
    let mut g = game();
    leyline(&mut g, P0);
    lands(&mut g, P0, "Ancient Tomb", 1, false);
    perm(&mut g, P0, "Nyxbloom Ancient");
    assert!(mana::pay(&mut g, P0, 0, "UUU", false).unwrap(), "Nyxbloom triples selected Island ability");
    assert_eq!(g.player(P0).life, 40);
}
/// Crypt Ghast sees the gained Swamp type: one Tomb makes UB without Tomb damage.
#[test]
#[ignore = "FAILS: Alt ability omits Crypt Ghast additional black mana"]
fn crypt_ghast_adds_black_to_leyline_blue() {
    let mut g = game();
    leyline(&mut g, P0);
    lands(&mut g, P0, "Ancient Tomb", 1, false);
    perm(&mut g, P0, "Crypt Ghast");
    assert!(mana::pay(&mut g, P0, 0, "UB", false).unwrap(), "tap Swamp for U, Ghast adds B");
    assert_eq!(g.player(P0).life, 40);
}
/// At one/two life Tomb's gained ability works; invalid payments spend nothing and cannot double-use it.
#[test]
fn tomb_low_life_and_atomic_failed_payments() {
    for hp in [1, 2, 40] {
        let mut g = game();
        leyline(&mut g, P0);
        let t = lands(&mut g, P0, "Ancient Tomb", 1, false)[0];
        g.player_mut(P0).life = hp;
        for pi in ["WU", "CCW", "CW"] {
            assert!(!mana::pay(&mut g, P0, 0, pi, false).unwrap(), "cannot double-use Tomb for {pi}");
            assert!(!g.perm(t).tapped);
            assert_eq!(g.player(P0).life, hp);
        }
        assert!(mana::pay(&mut g, P0, 0, "U", false).unwrap());
        assert_eq!(g.player(P0).life, hp);
    }
}
/// An inactive Leyline leaves legacy payment plans identical over deterministic random land mixes/costs.
#[test]
fn absent_leyline_random_plans_match_unhooked_control() {
    let mut seed = 0x5eed_u64;
    for board in 0..32 {
        let mut g = game();
        let mut dormant = g.clone();
        let ley = leyline(&mut dormant, P0);
        dormant.perm_mut(ley).phased = true;
        for _ in 0..5 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let name = ["Ancient Tomb", "Island", "Forest", "City of Brass", "Golgari Rot Farm", "Wastes"]
                [(seed >> 32) as usize % 6];
            lands(&mut g, P0, name, 1, false);
            lands(&mut dormant, P0, name, 1, false);
        }
        let a = mana::mana_units(&g, P0, false);
        let b = mana::mana_units(&dormant, P0, false);
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(&b) {
            assert_eq!((x.cols, x.amt, x.shape), (y.cols, y.amt, y.shape));
            assert!(!matches!(y.shape, Shape::Alt(_)));
        }
        for _ in 0..80 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let gn = (seed >> 32) as u32 % 9;
            let pi = ["", "W", "U", "B", "R", "G", "CC", "WU", "GG", "WUBRG"][(seed >> 16) as usize % 10];
            assert_eq!(
                mana::plan_pay(&g, &a, gn, pi),
                mana::plan_pay(&dormant, &b, gn, pi),
                "board {board}, cost {gn} {pi}"
            );
        }
    }
}

/// An actual mulligan may keep Leyline or bottom it; only a kept copy can begin on the battlefield.
#[test]
fn actual_mulligan_kept_or_bottomed_leyline() {
    for bottom in [false, true] {
        let mut g = game();
        let ley = card(&g, LEYLINE);
        let island = card(&g, "Island");
        g.player_mut(P0).library = vec![island; 6];
        g.player_mut(P0).library.push(ley);
        let mut count = 0;
        let h = Arc::new(ScriptHuman::from_fn(move |r| {
            if &*r.kind == "mulligan" {
                count += 1;
                return Answer::Pick(if count <= if bottom { 2 } else { 1 } { 1 } else { 0 });
            }
            if r.prompt.contains("bottom") {
                return Answer::Pick(r.choices.iter().position(|c| c.contains("Leyline")).unwrap());
            }
            Answer::Pick(0)
        }));
        g.humans.add(P0, h);
        sim_core::human::choices::mulligan(&mut g, P0, &mut sim_core::rng::Rng::named("adv-mull")).unwrap();
        sim_core::impls::card_models::opening_hand_person(&mut g, P0).unwrap();
        assert_eq!(perms_named(&g, P0, LEYLINE).len(), usize::from(!bottom));
        if bottom {
            assert!(g.player(P0).library.contains(&ley));
        }
    }
}
/// Face-down nonland creatures and tokens become all colours; a face-down object's hidden card gives no exemption.
#[test]
fn leyline_face_down_creature_and_token() {
    let mut g = game();
    leyline(&mut g, P0);
    let c = card(&g, "Whipcorder");
    let face = sim_core::impls::galadriel::enter_face_down(&mut g, P0, c).unwrap();
    let tok = token(&mut g, P0, 2);
    for m in [face, tok] {
        assert_eq!(colors_of(&g, m), Colors::from_letters("WUBRG"));
    }
}
/// Leyline's layer-five effect overrides the copied object's colour exception and disappears when the source phases.
#[test]
fn leyline_copy_exception_and_phase() {
    let mut g = game();
    let ley = leyline(&mut g, P0);
    let c = card(&g, "Serra Angel");
    let m = zones::enter_token_copy_with(
        &mut g,
        P0,
        c,
        Some(sim_core::state::CopyExc { colors: Some(Colors::from_letters("R")), ..Default::default() }),
    )
    .unwrap()
    .unwrap();
    assert_eq!(colors_of(&g, m), Colors::from_letters("WUBRG"));
    g.perm_mut(ley).phased = true;
    assert_eq!(colors_of(&g, m), Colors::from_letters("R"));
}
/// With Blood Moon entering first, a later Leyline adds all basic types to the Mountain.
#[test]
#[ignore = "FAILS: Blood Moon always wins regardless of timestamp"]
fn later_leyline_adds_types_after_blood_moon() {
    let mut g = game();
    let l = lands(&mut g, P0, "Strip Mine", 1, false)[0];
    perm(&mut g, P1, "Blood Moon");
    leyline(&mut g, P0);
    assert_eq!(
        chars::land_types(&g, l),
        Colors::from_letters("WUBRG"),
        "later additive type effect applies after Moon"
    );
    assert!(mana::pay(&mut g, P0, 0, "U", false).unwrap());
}
/// Additive Urborg/Yavimaya/Leyline type effects commute and affect the appropriate controllers.
#[test]
fn additive_land_types_all_orderings() {
    for order in [[0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]] {
        let mut g = game();
        for i in order {
            match i {
                0 => {
                    leyline(&mut g, P0);
                }
                1 => {
                    lands(&mut g, P1, "Urborg, Tomb of Yawgmoth", 1, false);
                }
                _ => {
                    lands(&mut g, P1, "Yavimaya, Cradle of Growth", 1, false);
                }
            }
        }
        let a = lands(&mut g, P0, "Island", 1, false)[0];
        let b = lands(&mut g, P1, "Island", 1, false)[0];
        assert_eq!(chars::land_types(&g, a), Colors::from_letters("WUBRG"));
        assert_eq!(chars::land_types(&g, b), Colors::from_letters("UBG"));
    }
}
/// An opponent gaining hexproof in response invalidates the chosen target; the trigger cannot retarget a different permanent.
#[test]
#[ignore = "FAILS: illegal hexproof target is replaced on resolution"]
fn kyodai_hexproof_response_does_not_retarget() {
    let mut g = game();
    let ring = perm(&mut g, P0, "Sol Ring");
    perm(&mut g, P1, "Birds of Paradise");
    hand(&mut g, P1, &["Blossoming Defense"]);
    pool(&mut g, P1, &[('G', 2)]);
    let h = Arc::new(ScriptHuman::from_fn(|r| {
        if r.prompt.contains(ASK) {
            Answer::Pick(
                r.choices
                    .iter()
                    .position(|c| c.contains("Birds"))
                    .unwrap_or_else(|| r.choices.iter().position(|c| c.contains("Sol Ring")).unwrap()),
            )
        } else if &*r.kind == "priority" {
            Answer::Action(json!({"do":"pass"}))
        } else {
            Answer::Decline
        }
    }));
    h.set_autopass("all");
    g.humans.add(P0, h);
    let mut fired = false;
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if &*r.kind == "priority" && !fired {
            fired = true;
            Answer::Action(json!({"do":"cast","card":0}))
        } else if &*r.kind == "target" {
            Answer::Pick(0)
        } else if &*r.kind == "priority" {
            Answer::Action(json!({"do":"pass"}))
        } else {
            Answer::Decline
        }
    }));
    h.set_autopass("all");
    g.humans.add(P1, h);
    perm(&mut g, P0, KYODAI);
    assert!(g.player(P1).hand.is_empty(), "response was cast");
    assert!(!indestructible(&g, ring), "hexproof target cannot be replaced by Ring");
}
/// Blinking Kyodai in response creates a second trigger; the original announced target remains an announced target even though its duration ends.
#[test]
#[ignore = "FAILS: original trigger never announces a target before blink response"]
fn kyodai_blink_in_response_has_two_announcements_one_grant() {
    let mut g = game();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Sol Ring");
    hand(&mut g, P0, &["Ephemerate"]);
    pool(&mut g, P0, &[('W', 1)]);
    let mut fired = false;
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if &*r.kind == "priority" && !fired {
            fired = true;
            return Answer::Action(json!({"do":"cast","card":0}));
        }
        if r.prompt.contains("Ephemerate") && &*r.kind == "target" {
            return Answer::Pick(r.choices.iter().position(|c| c.contains("Kyodai")).unwrap());
        }
        if r.prompt.contains(ASK) {
            return Answer::Pick(r.choices.iter().position(|c| c.contains("Elves")).unwrap());
        }
        if &*r.kind == "priority" { Answer::Action(json!({"do":"pass"})) } else { Answer::Decline }
    }));
    h.set_autopass("all");
    g.humans.add(P0, h.clone());
    let old = perm(&mut g, P0, KYODAI);
    assert!(!g.perm(old).on_bf);
    assert!(indestructible(&g, elf));
    assert_eq!(
        h.asked().iter().filter(|r| r.prompt.contains(ASK)).count(),
        2,
        "each trigger chooses a target before responses"
    );
}
/// The legend rule ignores indestructible, keeps one Kyodai, and only that object's grant persists.
#[test]
fn kyodai_legend_copy_retains_only_survivors_grant() {
    let mut g = game();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let old = perm(&mut g, P0, KYODAI);
    let cd = card(&g, KYODAI);
    let copy = zones::enter_token_copy(&mut g, P0, cd).unwrap().unwrap();
    life::check_state(&mut g).unwrap();
    assert_eq!(usize::from(g.perm(old).on_bf) + usize::from(g.perm(copy).on_bf), 1);
    let survivor = if g.perm(old).on_bf { old } else { copy };
    assert!(indestructible(&g, elf));
    zones::exile_perm(&mut g, survivor).unwrap();
    assert!(!indestructible(&g, elf));
}
/// Damage is attached to Tomb's printed CC ability, not to any gained basic-type ability, also for a person's tap.
#[test]
fn tomb_manual_low_life_blue_then_no_second_tap() {
    let mut g = game();
    leyline(&mut g, P0);
    lands(&mut g, P0, "Ancient Tomb", 1, false);
    g.player_mut(P0).life = 1;
    seat(&mut g, P0, vec![]);
    let s = sim_core::human::mana::sources(&g, P0);
    assert_eq!(s.len(), 1);
    let id = s[0].id as i64;
    assert_eq!(sim_core::human::mana::tap(&mut g, P0, id, Some('U')).unwrap(), None);
    assert_eq!(pool_text(&g, P0), "{U}");
    assert_eq!(g.player(P0).life, 1);
    assert!(sim_core::human::mana::tap(&mut g, P0, id, Some('U')).unwrap().is_some());
    assert_eq!(pool_total(&g, P0), 1);
}

/// Ghast's extra mana is specifically black, so one Leyline-fixed Island cannot pay UU.
#[test]
#[ignore = "FAILS: Crypt Ghast black bonus is treated as arbitrary coloured mana"]
fn crypt_ghast_bonus_is_black_not_free_colour() {
    let mut g = game();
    leyline(&mut g, P0);
    lands(&mut g, P0, "Island", 1, false);
    perm(&mut g, P0, "Crypt Ghast");
    assert!(mana::can_pay(&g, P0, 0, "UB", false));
    assert!(!mana::can_pay(&g, P0, 0, "UU", false), "extra mana is B, not a second arbitrary colour");
}
/// A feasible payment is Tomb's single U plus Cradle's GGG; land insertion order must not prevent it.
#[test]
#[ignore = "FAILS: greedy Alt choice loses feasible Tomb U plus Cradle GGG payment"]
fn alt_planner_preserves_large_own_ability_for_generic_cost() {
    let mut g = game();
    leyline(&mut g, P0);
    for _ in 0..3 {
        token(&mut g, P0, 1);
    }
    lands(&mut g, P0, "Gaea's Cradle", 1, false);
    lands(&mut g, P0, "Ancient Tomb", 1, false);
    assert!(mana::pay(&mut g, P0, 3, "U", false).unwrap(), "Tomb U plus Cradle GGG pays 3U");
    assert_eq!(g.player(P0).life, 40);
}
/// A person's selected U ability is tripled by Nyxbloom just like its other mana abilities.
#[test]
#[ignore = "FAILS: manual gained basic-type ability is not tripled"]
fn nyxbloom_manual_tomb_blue_makes_three() {
    let mut g = game();
    leyline(&mut g, P0);
    lands(&mut g, P0, "Ancient Tomb", 1, false);
    perm(&mut g, P0, "Nyxbloom Ancient");
    seat(&mut g, P0, vec![]);
    let s = sim_core::human::mana::sources(&g, P0);
    assert_eq!(s.len(), 1);
    sim_core::human::mana::tap(&mut g, P0, s[0].id as i64, Some('U')).unwrap();
    assert_eq!(pool_total(&g, P0), 3, "selected Island ability produces UUU");
    assert_eq!(g.player(P0).life, 40);
}
/// A copied Leyline moving to the other player fixes only its new controller's permanents.
#[test]
fn copied_leyline_control_change_updates_effects() {
    let mut g = game();
    let original = leyline(&mut g, P0);
    let copy = perm(&mut g, P0, "Copy Enchantment");
    let a = perm(&mut g, P0, "Llanowar Elves");
    let b = perm(&mut g, P1, "Llanowar Elves");
    assert_eq!(g.perm(copy).cd, g.perm(original).cd);
    control::gain_control(&mut g, P1, copy, Duration::Permanent);
    zones::exile_perm(&mut g, original).unwrap();
    assert_eq!(colors_of(&g, a), Colors::from_letters("G"));
    assert_eq!(colors_of(&g, b), Colors::from_letters("WUBRG"));
}
/// The two colours of a hybrid spell both participate in protection checks even when paid with only blue.
#[test]
fn hybrid_spell_protection_includes_unpaid_half() {
    let mut g = game();
    let t = perm(&mut g, P0, "Llanowar Elves");
    g.perm_mut(t).eot_kw.push("pro_W");
    let c = card(&g, "Steel of the Godhead");
    lands(&mut g, P0, "Island", 3, false);
    assert_eq!(mana::cost_of(&g, P0, c).1, "U");
    assert!(
        sim_core::engine::values::protected_from(&g, t, g.db.get(c).colors),
        "white half of W/U makes Steel a white source"
    );
}
/// A failed hybrid payment must leave lands, treasures and life intact.
#[test]
fn hybrid_insufficient_mixed_sources_spend_nothing() {
    let mut g = game();
    lands(&mut g, P0, "Forest", 1, false);
    perm(&mut g, P0, "Lotus Petal");
    g.player_mut(P0).treasures = 1;
    let c = hand(&mut g, P0, &[LEYLINE])[0];
    let (gn, pi) = mana::cost_of(&g, P0, c);
    assert!(!mana::pay_spell(&mut g, P0, c, gn, &pi, false).unwrap());
    assert_eq!(g.player(P0).treasures, 1);
    assert!(!g.perm(g.player(P0).lands[0]).tapped);
    assert!(has_perm(&g, P0, "Lotus Petal"));
    assert_eq!(g.player(P0).life, 40);
}

/// A departed source uses its last battlefield colour for protection; a later Leyline cannot recolour it.
#[test]
#[ignore = "FAILS: departed source is recoloured by later Leyline instead of using last known information"]
fn departed_ballista_does_not_gain_later_leylines_colours() {
    let mut g = game();
    let b = perm(&mut g, P0, "Walking Ballista");
    g.perm_mut(b).plus = 2;
    let t = perm(&mut g, P1, "Kor Firewalker");
    assert_eq!(colors_of(&g, b), Colors::NONE);
    zones::exile_perm(&mut g, b).unwrap();
    leyline(&mut g, P0);
    assert_eq!(
        life::creature_damage(&g, P0, Some(b), t, 1),
        1,
        "colourless departed Ballista's pending ping is not red"
    );
}
/// A departed source's last-known all-colour status persists after Leyline leaves.
#[test]
#[ignore = "FAILS: departed source loses last known all-colour status after Leyline leaves"]
fn departed_ballista_keeps_last_known_leyline_colours() {
    let mut g = game();
    let ley = leyline(&mut g, P0);
    let b = perm(&mut g, P0, "Walking Ballista");
    g.perm_mut(b).plus = 2;
    let t = perm(&mut g, P1, "Kor Firewalker");
    assert_eq!(colors_of(&g, b), Colors::from_letters("WUBRG"));
    zones::exile_perm(&mut g, b).unwrap();
    zones::exile_perm(&mut g, ley).unwrap();
    assert_eq!(
        life::creature_damage(&g, P0, Some(b), t, 1),
        0,
        "departed Ballista was red when it last existed on battlefield"
    );
}

/// Phasing out makes the tracked Kyodai absent; an ended for-as-long-as duration cannot restart on phase-in.
#[test]
#[ignore = "FAILS: ended duration resumes when source phases in"]
fn kyodai_phase_in_does_not_restart_ended_duration() {
    let mut g = game();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let k = perm(&mut g, P0, KYODAI);
    assert!(indestructible(&g, elf));
    g.perm_mut(k).phased = true;
    assert!(!indestructible(&g, elf));
    g.perm_mut(k).phased = false;
    assert!(!indestructible(&g, elf), "phasing in cannot restart an ended duration");
}
/// A grant already resolved on a target survives that target gaining hexproof/protection or changing controller.
#[test]
fn kyodai_resolved_grant_does_not_recheck_target_legality() {
    let mut g = game();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let k = perm(&mut g, P0, KYODAI);
    g.perm_mut(elf).eot_kw.extend(["hexproof", "pro_W"]);
    control::gain_control(&mut g, P1, elf, Duration::Permanent);
    assert!(indestructible(&g, elf), "only controlling Kyodai defines the duration, not continued targetability");
    zones::exile_perm(&mut g, k).unwrap();
    assert!(!indestructible(&g, elf));
}

/// A person casting REB can target a permanent that Leyline has made blue, just as the AI can.
#[test]
#[ignore = "FAILS: human target filter ignores Leyline permanent colours"]
fn person_red_elemental_blast_sees_leyline_blue_permanent() {
    let mut g = game();
    leyline(&mut g, P0);
    let angel = perm(&mut g, P0, "Serra Angel");
    let reb = card(&g, "Red Elemental Blast");
    assert!(removal::legal_targets(&g, P1, "destroy", "blue", false, Some(reb)).contains(&angel));
    let targets = sim_core::human::legal::spell_targets(&g, P1, reb).unwrap();
    assert!(
        targets.iter().any(|t| matches!(t,sim_core::human::Target::Perm(m) if *m==angel)),
        "human targets must read permanent's current blue colour"
    );
}
/// Leyline makes Oblivion Ring green too, so it cannot target an opponent's pro-green permanent.
#[test]
#[ignore = "FAILS: O-Ring reads printed source colours instead of Leyline colours"]
fn leyline_oblivion_ring_source_respects_protection_from_green() {
    let mut g = game();
    leyline(&mut g, P0);
    let angel = perm(&mut g, P1, "Serra Angel");
    g.perm_mut(angel).eot_kw.push("pro_G");
    let ring = perm(&mut g, P0, "Oblivion Ring");
    assert_eq!(colors_of(&g, ring), Colors::from_letters("WUBRG"));
    assert!(g.perm(angel).on_bf, "pro-green stops an all-colour Ring's triggered ability");
}
/// A creature made green by Leyline can pay Natural Order's additional sacrifice cost.
#[test]
fn leyline_green_creature_satisfies_natural_order_cost() {
    let mut g = game();
    leyline(&mut g, P0);
    let angel = perm(&mut g, P0, "Serra Angel");
    assert!(matches!(cast::sac_fodder(&g,P0,"green creature",None),Some(cast::Fodder::Perm(m)) if m==angel));
}
