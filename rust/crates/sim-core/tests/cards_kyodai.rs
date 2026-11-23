//! Kyodai, Soul of Kamigawa ({3}{W}, Legendary Creature - Dragon Spirit 3/3): "Flash. Flying. When Kyodai enters,
//! another target permanent gains indestructible for as long as you control Kyodai. {W}{U}{B}{R}{G}: Kyodai gets +5/+5
//! until end of turn." The enter trigger is card code (`impls/kyodai.rs`); flash, flying and the pump are compiled.
//! The expected results come from the Oracle text above, not from what the engine did before.

mod play_common;
use play_common::*;
use sim_core::engine::control::{self, Duration};
use sim_core::engine::values::{epow, etgh, indestructible};
use sim_core::engine::{cast, removal, zones};
use sim_core::human::human::{AbilityKind, abilities_of};
use sim_core::human::legal;
use sim_core::state::Ctx;
use std::sync::Arc;

const KYODAI: &str = "Kyodai, Soul of Kamigawa";
const ELESH: &str = "Elesh Norn, Mother of Machines";
const ASK: &str = "Kyodai: which permanent gains indestructible?";

fn tbl() -> Game {
    table(&["thanos", "seph"])
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf
}

/// a person at P0 who answers Kyodai's question with the first choice containing each word in turn (the last word
/// again once they run out), and passes priority otherwise; `f` may answer first
fn person(
    g: &mut Game,
    words: &'static [&'static str],
    f: impl FnMut(&Request) -> Option<Answer> + Send + 'static,
) -> Arc<ScriptHuman> {
    let mut f = f;
    let mut n = 0usize;
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        if let Some(a) = f(r) {
            return a;
        }
        if r.prompt.contains(ASK) {
            let word = words[n.min(words.len() - 1)];
            n += 1;
            return Answer::Pick(
                r.choices
                    .iter()
                    .position(|c| c.contains(word))
                    .unwrap_or_else(|| panic!("no {word:?}: {:?}", r.choices)),
            );
        }
        if &*r.kind == "priority" { Answer::Action(json!({"do": "pass"})) } else { Answer::Decline }
    }));
    g.humans.add(P0, h.clone());
    h
}

fn asked_kyodai(h: &ScriptHuman) -> Vec<Request> {
    h.asked().into_iter().filter(|r| r.prompt.contains(ASK)).collect()
}

/// try the compiled "pump" ability (paid from the pool, as a person pays); whether it was done
fn pump(g: &mut Game, p: PlayerId, m: PermId) -> bool {
    let list = abilities_of(g, p, m).unwrap();
    let labels: Vec<String> = list.iter().map(|(l, _)| l.clone()).collect();
    let Some((_, AbilityKind::Hook(a))) = list.into_iter().find(|(l, _)| l.contains("pump")) else {
        panic!("the pump is not offered: {labels:?}")
    };
    sim_core::ai::act::perform(g, p, &a).unwrap()
}

// ------------------------------------------------------------------ the enter trigger
#[test]
fn kyodai_etb_target_gains_indestructible() {
    let mut g = tbl();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let theirs = perm(&mut g, P1, "Llanowar Elves");
    let k = perm(&mut g, P0, KYODAI);
    assert!(indestructible(&g, elf), "the only other permanent of the AI gains it");
    assert!(!indestructible(&g, theirs), "the AI never picks an opponent's permanent");
    assert!(!indestructible(&g, k), "another permanent: not Kyodai");
    // it really survives destroy effects
    let doom = card(&g, "Murder");
    removal::apply_removal(&mut g, Some(P1), elf, "destroy", Some(doom)).unwrap();
    assert!(live(&g, elf), "indestructible: destroy does nothing");
}

#[test]
fn ai_picks_a_plan_engine_then_the_most_wanted_permanent_then_a_land() {
    // the thanos plan's engines: The One Ring, Rhystic Study, Mystic Remora, Urza, Crucible of Worlds
    let mut g = tbl();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let rhystic = perm(&mut g, P0, "Rhystic Study");
    perm(&mut g, P0, KYODAI);
    assert!(indestructible(&g, rhystic), "the plan's engine");
    assert!(!indestructible(&g, elf));

    // no land is chosen while a nonland permanent can be
    let mut g = tbl();
    let plains = lands(&mut g, P0, "Plains", 2, false);
    let elf = perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, KYODAI);
    assert!(indestructible(&g, elf) && !plains.iter().any(|&l| indestructible(&g, l)));

    // only lands of its own and an opponent's creature: one of its own lands, never the opponent's creature
    let mut g = tbl();
    let plains = lands(&mut g, P0, "Plains", 2, false);
    let theirs = perm(&mut g, P1, "Llanowar Elves");
    perm(&mut g, P0, KYODAI);
    assert_eq!(plains.iter().filter(|&&l| indestructible(&g, l)).count(), 1, "one of its own lands");
    assert!(!indestructible(&g, theirs));
}

#[test]
fn indestructible_ends_when_kyodai_dies() {
    let mut g = tbl();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let k = perm(&mut g, P0, KYODAI);
    assert!(indestructible(&g, elf));
    zones::die(&mut g, k, "destroy".into()).unwrap();
    assert!(!live(&g, k));
    assert!(!indestructible(&g, elf), "for as long as you control Kyodai: he is gone");
}

#[test]
fn ends_when_control_of_kyodai_changes() {
    let mut g = tbl();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let k = perm(&mut g, P0, KYODAI);
    assert!(indestructible(&g, elf));
    assert!(control::gain_control(&mut g, P1, k, Duration::Permanent));
    assert!(!indestructible(&g, elf), "you no longer control Kyodai");
}

#[test]
fn kyodai_removed_in_response_grants_nothing() {
    // the trigger is on the stack; in response the person exiles Kyodai: "for as long as you control Kyodai" had
    // already ended, so the ability does nothing (CR 611.2b) and no target is chosen
    let mut g = tbl();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let swords = hand(&mut g, P0, &["Swords to Plowshares"])[0];
    pool(&mut g, P0, &[('W', 1)]);
    let mut cast_it = false;
    let h = person(&mut g, &["Elves"], move |r| {
        if &*r.kind == "priority" && !cast_it {
            cast_it = true;
            return Some(Answer::Action(json!({"do": "cast", "card": 0})));
        }
        if &*r.kind == "target" && r.prompt.contains("Swords") {
            return Some(Answer::Pick(r.choices.iter().position(|c| c.contains("Kyodai")).unwrap()));
        }
        None
    });
    h.set_autopass("all"); // a person is given priority on their own trigger only with this setting
    let k = perm(&mut g, P0, KYODAI);
    assert!(g.player(P0).exile.contains(&swords) || g.player(P0).gy.contains(&swords), "Swords was cast");
    assert!(!live(&g, k), "Kyodai was exiled in response");
    assert!(!indestructible(&g, elf), "nothing was granted");
    assert!(asked_kyodai(&h).is_empty(), "no target was chosen for an ability that does nothing");
}

#[test]
fn ephemerate_old_target_loses_new_target_gains() {
    let mut g = tbl();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let ring = perm(&mut g, P0, "Sol Ring");
    let h = person(&mut g, &["Elves", "Sol Ring"], |_| None);
    let k = perm(&mut g, P0, KYODAI);
    assert!(indestructible(&g, elf) && !indestructible(&g, ring), "the first trigger: the person picked the Elves");
    // Ephemerate: a new object comes back, its trigger asks again; the old object's grant ended with it
    let k2 = sim_core::impls::t2::blink(&mut g, P0, k).unwrap().expect("Kyodai came back");
    assert_ne!(k, k2, "a new object");
    assert!(live(&g, k2) && !live(&g, k));
    assert!(!indestructible(&g, elf), "the old target loses it");
    assert!(indestructible(&g, ring), "the new target gains it");
    assert_eq!(asked_kyodai(&h).len(), 2, "asked again for the new object");
}

#[test]
fn cannot_target_itself() {
    let mut g = tbl();
    perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P1, "Sol Ring");
    let h = person(&mut g, &["Elves"], |_| None);
    perm(&mut g, P0, KYODAI);
    let qs = asked_kyodai(&h);
    assert_eq!(qs.len(), 1);
    assert!(!qs[0].choices.iter().any(|c| c.contains("Kyodai")), "another permanent: {:?}", qs[0].choices);
    assert!(qs[0].choices.iter().any(|c| c.contains("Sol Ring")), "an opponent's permanent is a legal target");
}

#[test]
fn cannot_target_shrouded_greaves_creature() {
    // shroud stops every spell and ability, its controller's too: Lightning Greaves gives it
    let mut g = tbl();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let bird = perm(&mut g, P0, "Birds of Paradise");
    let greaves = perm(&mut g, P0, "Lightning Greaves");
    g.perm_mut(greaves).attached = Some(bird);
    let h = person(&mut g, &["Elves"], |_| None);
    perm(&mut g, P0, KYODAI);
    let qs = asked_kyodai(&h);
    assert_eq!(qs.len(), 1);
    assert!(
        !qs[0].choices.iter().any(|c| c.contains("Birds")),
        "the Greaves creature is not a choice: {:?}",
        qs[0].choices
    );
    assert!(indestructible(&g, elf) && !indestructible(&g, bird));
}

#[test]
fn hexproof_shroud_and_protection_from_white_are_not_legal_choices() {
    let mut g = tbl();
    let ok = perm(&mut g, P1, "Llanowar Elves");
    let hexproof = perm(&mut g, P1, "Birds of Paradise");
    g.perm_mut(hexproof).eot_kw.push("hexproof");
    let shroud = perm(&mut g, P1, "Sol Ring");
    g.perm_mut(shroud).eot_kw.push("shroud");
    let pro = perm(&mut g, P1, "Mana Vault");
    g.perm_mut(pro).eot_kw.push("pro_W");
    let own_hexproof = perm(&mut g, P0, "Mother of Runes");
    g.perm_mut(own_hexproof).eot_kw.push("hexproof"); // hexproof only stops opponents
    let h = person(&mut g, &["Elves"], |_| None);
    perm(&mut g, P0, KYODAI);
    let qs = asked_kyodai(&h);
    assert_eq!(qs.len(), 1);
    let cs = &qs[0].choices;
    for gone in ["Birds", "Sol Ring", "Mana Vault"] {
        assert!(!cs.iter().any(|c| c.contains(gone)), "{gone} is not a legal target: {cs:?}");
    }
    assert!(cs.iter().any(|c| c.contains("Llanowar")), "{cs:?}");
    assert!(cs.iter().any(|c| c.contains("Mother of Runes")), "your own hexproof permanent is legal: {cs:?}");
    assert!(indestructible(&g, ok), "the person's pick, an opponent's permanent");
}

#[test]
fn no_other_legal_permanent_the_trigger_does_nothing() {
    let mut g = tbl();
    let h = person(&mut g, &["Elves"], |_| None);
    let k = perm(&mut g, P0, KYODAI);
    assert!(asked_kyodai(&h).is_empty(), "no target: nothing is asked");
    assert!(g.stack.is_empty() && !indestructible(&g, k), "and nothing is on the stack or granted");
    // the AI: the same
    let mut g = tbl();
    let k = perm(&mut g, P0, KYODAI);
    assert!(!indestructible(&g, k));
}

#[test]
fn person_picks_target() {
    // each in turn, so the answer is honoured whichever the AI would have chosen
    for pick in ["Elves", "Rhystic Study", "Sol Ring"] {
        let mut g = tbl();
        let all = [
            ("Elves", perm(&mut g, P0, "Llanowar Elves")),
            ("Rhystic Study", perm(&mut g, P0, "Rhystic Study")),
            ("Sol Ring", perm(&mut g, P1, "Sol Ring")),
        ];
        let words: &'static [&'static str] = match pick {
            "Elves" => &["Elves"],
            "Rhystic Study" => &["Rhystic"],
            _ => &["Sol Ring"],
        };
        let h = person(&mut g, words, |_| None);
        perm(&mut g, P0, KYODAI);
        assert_eq!(asked_kyodai(&h).len(), 1, "asked once");
        for (n, m) in all {
            assert_eq!(indestructible(&g, m), n == pick, "{pick}: {n}");
        }
    }
}

#[test]
fn elesh_norn_blanks_trigger() {
    let mut g = tbl();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P1, ELESH);
    let h = person(&mut g, &["Elves"], |_| None);
    perm(&mut g, P0, KYODAI);
    assert!(!indestructible(&g, elf), "an opponent's Elesh Norn: Kyodai's enter trigger does not trigger");
    assert!(asked_kyodai(&h).is_empty());
}

#[test]
fn panharmonicon_two_targets() {
    // a creature entering triggers an additional time: two instances, two targets, both indestructible
    let mut g = tbl();
    perm(&mut g, P0, "Panharmonicon");
    let ours: Vec<PermId> = ["Llanowar Elves", "Sol Ring", "Mana Vault"].iter().map(|n| perm(&mut g, P0, n)).collect();
    perm(&mut g, P0, KYODAI);
    let n = ours.iter().filter(|&&m| indestructible(&g, m)).count();
    assert_eq!(n, 2, "two triggers, two different targets");
    // a person picks each target
    let mut g = tbl();
    perm(&mut g, P0, "Panharmonicon");
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let ring = perm(&mut g, P0, "Sol Ring");
    let vault = perm(&mut g, P0, "Mana Vault");
    let h = person(&mut g, &["Elves", "Sol Ring"], |_| None);
    perm(&mut g, P0, KYODAI);
    assert_eq!(asked_kyodai(&h).len(), 2, "asked for each trigger");
    assert!(indestructible(&g, elf) && indestructible(&g, ring) && !indestructible(&g, vault));
    // both on the same permanent is one grant
    let mut g = tbl();
    perm(&mut g, P0, "Panharmonicon");
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let h = person(&mut g, &["Elves"], |_| None);
    perm(&mut g, P0, KYODAI);
    assert_eq!(asked_kyodai(&h).len(), 2);
    assert!(indestructible(&g, elf));
}

#[test]
fn recast_from_command_zone_new_grant() {
    let mut g = tbl();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    lands(&mut g, P0, "Plains", 8, false);
    let cmd = g.player(P0).cmd;
    assert_eq!(g.db.get(cmd).name.as_ref(), KYODAI);
    assert!(cast::cast_card(&mut g, P0, cmd, "cmd", Ctx::default()).unwrap());
    let k = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).cd == Some(cmd) && live(&g, m)).unwrap();
    assert!(indestructible(&g, elf), "cast: the enter trigger");
    zones::die(&mut g, k, "destroy".into()).unwrap();
    assert!(!indestructible(&g, elf), "he died");
    assert!(g.player(P0).cmd_in_zone, "back in the command zone");
    assert!(cast::cast_card(&mut g, P0, cmd, "cmd", Ctx::default()).unwrap(), "recast (with the tax)");
    assert!(g.player(P0).tax > 0);
    assert!(indestructible(&g, elf), "the new Kyodai's trigger makes a new grant");
}

// ------------------------------------------------------------------ phasing (Teferi's Protection)
#[test]
fn kyodai_phasing_out_turns_the_grant_off_while_he_is_gone() {
    // the engine reads a static ability only from a permanent that is phased in, so the grant is off while Kyodai is
    // phased out (a phased-out permanent is treated as though it doesn't exist) and is back when he phases in
    let mut g = tbl();
    let elf = perm(&mut g, P0, "Llanowar Elves");
    let k = perm(&mut g, P0, KYODAI);
    assert!(indestructible(&g, elf));
    g.perm_mut(k).phased = true;
    assert!(!indestructible(&g, elf), "while Kyodai is phased out");
    g.perm_mut(k).phased = false;
    assert!(
        indestructible(&g, elf),
        "phased back in: the grant resumes (see the report: the rule may end it for good)"
    );
}

// ------------------------------------------------------------------ flash, flying and the pump
#[test]
fn flash_is_real() {
    let mut g = tbl();
    let k = hand(&mut g, P0, &["Kyodai, Soul of Kamigawa", "Llanowar Elves"]);
    g.active = Some(P1); // the opponent's turn
    g.step = sim_core::state::Step::Main1;
    lands(&mut g, P0, "Plains", 4, false);
    assert_eq!(
        legal::check_cast(&mut g, P0, k[1], "hand").is_some(),
        true,
        "a creature without flash waits for your turn"
    );
    pool(&mut g, P0, &[('W', 1), ('C', 3)]);
    assert_eq!(legal::check_cast(&mut g, P0, k[0], "hand"), None, "Kyodai can be cast at instant speed");
    assert!(g.db.get(k[0]).tag(sim_core::tag::Tag::Flash));
}

#[test]
fn flying_is_real() {
    let mut g = tbl();
    let k = perm(&mut g, P0, KYODAI);
    assert!(g.perm(k).fly || sim_core::dsl::has_kw(&g, k, "flying"));
    assert_eq!((epow(&g, k), etgh(&g, k)), (3, 3));
}

#[test]
fn pump_costs_wubrg_and_gives_plus_five_until_end_of_turn_stacking() {
    let mut g = tbl();
    person(&mut g, &["Elves"], |_| None); // a person pays from the pool
    let k = perm(&mut g, P0, KYODAI);
    assert_eq!((epow(&g, k), etgh(&g, k)), (3, 3));
    let list = abilities_of(&mut g, P0, k).unwrap();
    assert!(list.iter().any(|(l, _)| l.starts_with("{W}{U}{B}{R}{G}: ")), "the cost is {{W}}{{U}}{{B}}{{R}}{{G}}");
    // one colour short: it can't be done and costs nothing
    pool(&mut g, P0, &[('W', 1), ('U', 1), ('B', 1), ('R', 1), ('C', 3)]);
    assert!(!pump(&mut g, P0, k), "no green: {{W}}{{U}}{{B}}{{R}}{{G}} can't be paid with colourless");
    assert_eq!((epow(&g, k), etgh(&g, k)), (3, 3), "nothing happened");
    assert_eq!(pool_total(&g, P0), 7, "and nothing was spent");
    // exactly {W}{U}{B}{R}{G}
    pool(&mut g, P0, &[('G', 1)]);
    assert!(pump(&mut g, P0, k));
    assert_eq!((epow(&g, k), etgh(&g, k)), (8, 8), "+5/+5");
    assert_eq!(pool_total(&g, P0), 3, "five coloured mana paid, the three colourless left");
    // each activation adds +5/+5
    pool(&mut g, P0, &[('W', 1), ('U', 1), ('B', 1), ('R', 1), ('G', 1)]);
    assert!(pump(&mut g, P0, k));
    assert_eq!((epow(&g, k), etgh(&g, k)), (13, 13), "stacks per activation");
    // until end of turn: the engine keeps it as the turn's bonus and clears it when the next turn starts
    assert_eq!(g.perm(k).eot_pt, (10, 10));
    sim_core::engine::turn::step_start(&mut g, P1).unwrap();
    assert_eq!((epow(&g, k), etgh(&g, k)), (3, 3), "gone once the turn is over");
}

// ------------------------------------------------------------------ with Leyline of the Guildpact
// Oracle: Leyline makes lands every basic land type (so each taps for any one colour) and every nonland permanent
// all colours (so Faeburrow Elder taps for one mana of each of the five colours, and counts five colours for its P/T).

#[test]
fn pump_paid_by_five_islands_under_leyline_but_not_without_it() {
    // without Leyline five Islands make {U} only: {W}{U}{B}{R}{G} can't be paid and nothing happens
    let mut g = tbl();
    let k = perm(&mut g, P0, KYODAI);
    lands(&mut g, P0, "Island", 5, false);
    assert!(!pump(&mut g, P0, k), "five Islands can't pay {{W}}{{U}}{{B}}{{R}}{{G}}");
    assert_eq!((epow(&g, k), etgh(&g, k)), (3, 3));
    // with Leyline each Island is every basic type: five lands, five colours
    perm(&mut g, P0, "Leyline of the Guildpact");
    assert!(pump(&mut g, P0, k), "five lands that each tap for any colour pay it");
    assert_eq!((epow(&g, k), etgh(&g, k)), (8, 8), "+5/+5");
}

#[test]
fn pump_paid_by_elder_under_leyline_gives_8_8() {
    // Leyline: every nonland permanent is all five colours, so Faeburrow Elder taps for {W}{U}{B}{R}{G} alone
    let mut g = tbl();
    let k = perm(&mut g, P0, KYODAI);
    let elder = perm(&mut g, P0, "Faeburrow Elder");
    perm(&mut g, P0, "Leyline of the Guildpact");
    assert_eq!(
        (epow(&g, elder), etgh(&g, elder)),
        (5, 5),
        "+1/+1 for each of the five colours among permanents you control"
    );
    assert!(pump(&mut g, P0, k), "the Elder's five mana pay the pump");
    assert_eq!((epow(&g, k), etgh(&g, k)), (8, 8));
    assert!(g.perm(elder).tapped, "the Elder was tapped for the mana");
}
