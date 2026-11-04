//! Practice-mode gaps, Agent A (rust/agents/practice-gaps.md): the Ur-Dragon, Kefka, Sauron and Thanos cards whose
//! abilities or choices the person did not get. Each test seats a scripted person and checks that the card asks and
//! that the answer is what happens.

mod play_common;
use play_common::*;
use sim_core::human::human::abilities_of;
use sim_core::state::{DataKey, Step, Val};
use std::sync::{Arc, Mutex};

type Log = Arc<Mutex<Vec<String>>>;

/// seat 0 as a person: the first priority is `first` (an action), later priorities pass; every other request goes to
/// `f` (None: the first choice). Everything asked is recorded as "kind: prompt [choices]".
fn person(g: &mut Game, first: Option<Value>, f: impl Fn(&Request) -> Option<Answer> + Send + 'static) -> Log {
    let log: Log = Arc::new(Mutex::new(vec![]));
    let l2 = log.clone();
    let first = Mutex::new(first);
    g.humans.add(
        P0,
        Arc::new(ScriptHuman::from_fn(move |r: &Request| {
            l2.lock().unwrap().push(format!("{}: {} {:?}", r.kind, r.prompt, r.choices));
            if &*r.kind == "priority" {
                return match first.lock().unwrap().take() {
                    Some(a) => Answer::Action(a),
                    None => Answer::Action(json!({"do": "pass"})),
                };
            }
            if let Some(a) = f(r) {
                return a;
            }
            if &*r.kind == "attack" || &*r.kind == "block" { Answer::Picks(vec![]) } else { Answer::Pick(0) }
        })),
    );
    log
}

/// play the main phase of seat 0 once
fn run(g: &mut Game) {
    let _ = sim_core::human::human::human_main(g, P0, false);
}

fn asked(log: &Log, word: &str) -> bool {
    log.lock().unwrap().iter().any(|l| l.contains(word))
}

fn dump(log: &Log) -> String {
    log.lock().unwrap().join("\n")
}

/// the choice containing `word`
fn text(r: &Request, word: &str) -> Answer {
    Answer::Pick(
        r.choices.iter().position(|c| c.contains(word)).unwrap_or_else(|| panic!("no {word:?} in {:?}", r.choices)),
    )
}

/// seat 0 in its first main phase
fn main_table(key: &str) -> Game {
    let mut g = table(&[key, "seph"]);
    g.active = Some(P0);
    g.step = Step::Main1;
    g
}

fn use_perm(g: &Game, m: PermId) -> Value {
    json!({"do": "use", "perm": g.player(P0).perms.iter().position(|&x| x == m).unwrap()})
}

fn cast_last(g: &Game) -> Value {
    json!({"do": "cast", "card": g.player(P0).hand.len() - 1})
}

fn ability_count(g: &mut Game, m: PermId) -> usize {
    abilities_of(g, P0, m).unwrap().len()
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).gy.iter().any(|&c| &*g.db.get(c).name == name)
}

// ------------------------------------------------------------------ Garth One-Eye
#[test]
fn garth_is_listed_with_no_mana_and_chooses_a_name_and_a_target() {
    let mut g = main_table("urdragon");
    let garth = perm(&mut g, P0, "Garth One-Eye");
    let elves = perm(&mut g, P1, "Llanowar Elves");
    seat(&mut g, P0, vec![]); // a person sits down; the pool is empty
    assert_eq!(ability_count(&mut g, garth), 1, "listed whenever the rules allow, not only when the AI wants it");
    pool(&mut g, P0, &[('B', 1), ('C', 1)]);
    let first = Some(use_perm(&g, garth));
    let log = person(&mut g, first, |r| {
        if r.prompt.contains("choose a card name") {
            Some(text(r, "Terror"))
        } else if r.prompt.contains("choose a target") {
            Some(text(r, "Llanowar"))
        } else {
            None
        }
    });
    run(&mut g);
    assert!(asked(&log, "choose a card name"), "{}", dump(&log));
    assert!(!g.player(P1).perms.contains(&elves), "the Terror copy destroyed the target: {}", dump(&log));
    assert!(g.perm(garth).tapped);
    assert!(sim_core::impls::garth::chosen(&g, garth, 2), "Terror is used up");
    assert_eq!(pool_total(&g, P0), 0, "{{1}}{{B}} paid for the copy");
    assert!(!in_gy(&g, P0, "Terror"), "the copy is not a card: it never reaches the graveyard");
    // the second time Terror is no longer on the list
    g.perm_mut(garth).tapped = false;
    let first = Some(use_perm(&g, garth));
    let log = person(&mut g, first, |r| {
        if r.prompt.contains("choose a card name") {
            assert!(!r.choices.iter().any(|c| c.contains("Terror")), "{:?}", r.choices);
            Some(text(r, "Black Lotus"))
        } else {
            None
        }
    });
    run(&mut g);
    assert!(asked(&log, "choose a card name"));
    assert!(sim_core::impls::garth::chosen(&g, garth, 5));
}

#[test]
fn garth_with_nothing_to_pay_with_uses_the_name_up() {
    let mut g = main_table("urdragon");
    let garth = perm(&mut g, P0, "Garth One-Eye");
    let first = Some(use_perm(&g, garth));
    let log =
        person(&mut g, first, |r| if r.prompt.contains("choose a card name") { Some(text(r, "Shivan")) } else { None });
    run(&mut g);
    assert!(asked(&log, "choose a card name"), "{}", dump(&log));
    assert!(g.perm(garth).tapped);
    assert!(sim_core::impls::garth::chosen(&g, garth, 3), "Shivan Dragon is chosen even though it can't be cast");
    assert!(!has_perm(&g, P0, "Shivan Dragon"), "no mana in the pool: the copy ceases to exist");
}

#[test]
fn garth_casts_a_shivan_dragon_copy_from_the_pool() {
    let mut g = main_table("urdragon");
    let garth = perm(&mut g, P0, "Garth One-Eye");
    pool(&mut g, P0, &[('R', 2), ('C', 4)]);
    let first = Some(use_perm(&g, garth));
    let _ =
        person(&mut g, first, |r| if r.prompt.contains("choose a card name") { Some(text(r, "Shivan")) } else { None });
    run(&mut g);
    assert!(has_perm(&g, P0, "Shivan Dragon"), "the copy was cast and entered");
}

// ------------------------------------------------------------------ Kefka, Court Mage
#[test]
fn kefka_transforms_for_eight_at_sorcery_speed() {
    let mut g = main_table("kefka");
    let kefka = perm(&mut g, P0, "Kefka, Court Mage // Kefka, Ruler of Ruin");
    lands(&mut g, P1, "Swamp", 5, false);
    seat(&mut g, P0, vec![]);
    assert_eq!(ability_count(&mut g, kefka), 1, "listed with an empty pool");
    // not at sorcery speed: refused
    g.step = Step::Combat;
    pool(&mut g, P0, &[('C', 8)]);
    let first = Some(use_perm(&g, kefka));
    let _ = person(&mut g, first, |_| None);
    run(&mut g);
    assert!(!sim_core::impls::kefka::flipped(&g, P0), "refused outside a main phase");
    assert_eq!(pool_total(&g, P0), 8);
    g.step = Step::Main1;
    let before = g.player(P1).lands.len() + g.player(P1).perms.len();
    let first = Some(use_perm(&g, kefka));
    let log = person(&mut g, first, |_| None);
    run(&mut g);
    assert!(sim_core::impls::kefka::flipped(&g, P0), "{}", dump(&log));
    assert_eq!(pool_total(&g, P0), 0, "{{8}} paid");
    assert_eq!(g.player(P1).lands.len() + g.player(P1).perms.len(), before - 1, "an opponent sacrificed a permanent");
    assert_eq!(ability_count(&mut g, kefka), 0, "Kefka, Ruler of Ruin has no activated ability");
}

#[test]
fn kefka_does_nothing_without_the_mana() {
    let mut g = main_table("kefka");
    let kefka = perm(&mut g, P0, "Kefka, Court Mage // Kefka, Ruler of Ruin");
    let first = Some(use_perm(&g, kefka));
    let _ = person(&mut g, first, |_| None);
    run(&mut g);
    assert!(!sim_core::impls::kefka::flipped(&g, P0), "no mana: not activated");
}

// ------------------------------------------------------------------ Nicol Bolas, the Ravager
#[test]
fn nicol_bolas_transforms_at_sorcery_speed() {
    let mut g = main_table("urdragon");
    let bolas = perm(&mut g, P0, "Nicol Bolas, the Ravager");
    seat(&mut g, P0, vec![]);
    assert_eq!(ability_count(&mut g, bolas), 1);
    pool(&mut g, P0, &[('U', 1), ('B', 1), ('R', 1), ('C', 4)]);
    let first = Some(use_perm(&g, bolas));
    let log = person(&mut g, first, |_| None);
    run(&mut g);
    assert_eq!(pool_total(&g, P0), 0, "{{4}}{{U}}{{B}}{{R}} paid: {}", dump(&log));
    let walker = g.player(P0).perms.iter().copied().find(|&m| g.perm(m).name.contains("Arisen"));
    assert!(
        walker.is_some(),
        "Bolas came back as the Arisen: {:?}",
        g.player(P0).perms.iter().map(|&m| g.perm(m).name.to_string()).collect::<Vec<_>>()
    );
    let walker = walker.unwrap();
    assert_eq!(ability_count(&mut g, walker), 4, "the Arisen's loyalty abilities are offered to the person");
    let hand = g.player(P0).hand.len();
    let first = Some(use_perm(&g, walker));
    let _ = person(&mut g, first, |r| if r.prompt.contains("which ability") { Some(text(r, "+2")) } else { None });
    run(&mut g);
    assert_eq!(g.player(P0).hand.len(), hand + 2, "+2: draw two cards");
    assert_eq!(g.perm(walker).loyalty, Some(9));
}

// ------------------------------------------------------------------ Thanos, the Mad Titan
#[test]
fn thanos_power_up_asks_odd_or_even() {
    let mut g = main_table("thanos");
    let thanos = perm(&mut g, P0, "Thanos, the Mad Titan");
    let elves = perm(&mut g, P1, "Llanowar Elves"); // mana value 1: odd
    let ours = perm(&mut g, P0, "Sakura-Tribe Elder"); // mana value 2: even
    seat(&mut g, P0, vec![]);
    assert_eq!(ability_count(&mut g, thanos), 1, "listed with an empty pool");
    let (gn, pips) = sim_core::impls::t6_thanos::cost(&g, thanos);
    let mut mana: Vec<(char, u32)> = pips.chars().map(|c| (c, 1)).collect();
    mana.push(('C', gn));
    pool(&mut g, P0, &mana);
    let first = Some(use_perm(&g, thanos));
    let log = person(&mut g, first, |r| if r.prompt.contains("odd or even") { Some(text(r, "odd")) } else { None });
    run(&mut g);
    assert!(asked(&log, "odd or even"), "{}", dump(&log));
    assert!(!g.player(P1).perms.contains(&elves), "the odd creature died");
    assert!(g.player(P0).perms.contains(&ours), "the even creature lived");
    assert!(sim_core::impls::t6_thanos::used(&g, thanos));
    // once only
    let first = Some(use_perm(&g, thanos));
    let log = person(&mut g, first, |_| None);
    run(&mut g);
    assert!(!asked(&log, "(destroy each other creature with a mana value"), "{}", dump(&log));
}

#[test]
fn thanos_power_up_can_pick_even() {
    let mut g = main_table("thanos");
    let thanos = perm(&mut g, P0, "Thanos, the Mad Titan");
    let elves = perm(&mut g, P1, "Llanowar Elves");
    let ours = perm(&mut g, P0, "Sakura-Tribe Elder");
    let (gn, pips) = sim_core::impls::t6_thanos::cost(&g, thanos);
    let mut mana: Vec<(char, u32)> = pips.chars().map(|c| (c, 1)).collect();
    mana.push(('C', gn));
    pool(&mut g, P0, &mana);
    let first = Some(use_perm(&g, thanos));
    let _ = person(&mut g, first, |r| if r.prompt.contains("odd or even") { Some(text(r, "even")) } else { None });
    run(&mut g);
    assert!(g.player(P1).perms.contains(&elves));
    assert!(!g.player(P0).perms.contains(&ours));
}

// ------------------------------------------------------------------ Ranger-Captain of Eos
#[test]
fn ranger_captain_sacrifices_to_silence_the_table() {
    let mut g = main_table("thanos");
    let ranger = perm(&mut g, P0, "Ranger-Captain of Eos");
    let c = g.perm(ranger).card();
    seat(&mut g, P0, vec![]);
    assert_eq!(ability_count(&mut g, ranger), 1);
    let first = Some(use_perm(&g, ranger));
    let _ = person(&mut g, first, |_| None);
    run(&mut g);
    assert!(!g.player(P0).perms.contains(&ranger), "sacrificed");
    assert!(g.player(P0).gy.contains(&c));
    assert!(sim_core::impls::partials::silence_active(&g, P1), "opponents can't cast noncreature spells this turn");
    assert!(!sim_core::impls::partials::silence_active(&g, P0));
}

// ------------------------------------------------------------------ Scion of the Ur-Dragon
#[test]
fn scion_searches_for_a_dragon_the_person_picks() {
    let mut g = main_table("urdragon");
    let scion = perm(&mut g, P0, "Scion of the Ur-Dragon");
    seat(&mut g, P0, vec![]);
    assert_eq!(ability_count(&mut g, scion), 1, "listed with an empty pool");
    pool(&mut g, P0, &[('C', 2)]);
    let first = Some(use_perm(&g, scion));
    let picked: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let p2 = picked.clone();
    let log = person(&mut g, first, move |r| {
        if &*r.kind == "search" {
            let last = r.choices.len() - 2; // the last Dragon, not the first
            *p2.lock().unwrap() = Some(r.choices[last].clone());
            Some(Answer::Pick(last))
        } else {
            None
        }
    });
    run(&mut g);
    let name = picked.lock().unwrap().clone().unwrap_or_else(|| panic!("never asked: {}", dump(&log)));
    assert!(asked(&log, "search your library for a Dragon permanent card"), "{}", dump(&log));
    let short = name.split(" {").next().unwrap().to_string();
    assert!(in_gy(&g, P0, &short), "{short} went to the graveyard: {}", dump(&log));
    assert!(sim_core::impls::t6_scion::is_copy(&g, scion), "Scion became a copy of it");
    assert_eq!(g.perm(scion).name.to_string(), short);
    assert_eq!(pool_total(&g, P0), 0);
}

// ------------------------------------------------------------------ Dragon's Hoard
#[test]
fn dragons_hoard_draws_by_removing_a_gold_counter() {
    let mut g = main_table("urdragon");
    let hoard = perm(&mut g, P0, "Dragon's Hoard");
    g.perm_mut(hoard).data.set(DataKey::T45Dragon, Val::Int(1));
    seat(&mut g, P0, vec![]);
    assert!(ability_count(&mut g, hoard) >= 1);
    let n = g.player(P0).hand.len();
    let first = Some(use_perm(&g, hoard));
    let _ =
        person(&mut g, first, |r| if r.prompt.contains("which ability") { Some(text(r, "draw a card")) } else { None });
    run(&mut g);
    assert_eq!(g.player(P0).hand.len(), n + 1);
    assert!(g.perm(hoard).tapped);
    assert_eq!(sim_core::impls::t45_urdragon::gold(&g, hoard), 0);
}

// ------------------------------------------------------------------ Faerie Mastermind
#[test]
fn faerie_mastermind_makes_each_player_draw() {
    let mut g = main_table("sauron");
    let fm = perm(&mut g, P0, "Faerie Mastermind");
    seat(&mut g, P0, vec![]);
    assert_eq!(ability_count(&mut g, fm), 1, "listed with an empty pool");
    pool(&mut g, P0, &[('U', 1), ('C', 3)]);
    let (a, b) = (g.player(P0).hand.len(), g.player(P1).hand.len());
    let first = Some(use_perm(&g, fm));
    let _ = person(&mut g, first, |_| None);
    run(&mut g);
    assert_eq!(g.player(P0).hand.len(), a + 1);
    assert_eq!(g.player(P1).hand.len(), b + 1);
    assert_eq!(pool_total(&g, P0), 0);
}

// ================================================================== choices
fn all_mana() -> Vec<(char, u32)> {
    vec![('W', 10), ('U', 10), ('B', 10), ('R', 10), ('G', 10), ('C', 10)]
}

fn not_you(r: &Request) -> Answer {
    Answer::Pick(r.choices.iter().position(|c| !c.contains("(you)")).unwrap())
}

// ------------------------------------------------------------------ Alpharael, Dreaming Acolyte
#[test]
fn alpharael_asks_for_the_artifact_or_two_cards() {
    let mut g = main_table("kefka");
    hand(&mut g, P0, &["Sol Ring", "Plains", "Island"]);
    let log = person(&mut g, None, |r| {
        if r.prompt.contains("unless you discard an artifact") { Some(text(r, "Sol Ring")) } else { None }
    });
    perm(&mut g, P0, "Alpharael, Dreaming Acolyte");
    assert!(asked(&log, "unless you discard an artifact"), "{}", dump(&log));
    assert!(in_gy(&g, P0, "Sol Ring"));
    assert_eq!(g.player(P0).gy.len(), 1, "only the artifact");
    assert_eq!(g.player(P0).hand.len(), 4, "three, two drawn, one discarded");
}

#[test]
fn alpharael_lets_the_person_pick_which_two_cards() {
    let mut g = main_table("kefka");
    hand(&mut g, P0, &["Sol Ring", "Plains", "Island"]);
    let log = person(&mut g, None, |r| {
        if r.prompt.contains("unless you discard an artifact") {
            Some(text(r, "discard 2 cards"))
        } else if r.prompt.contains("1 of 2") {
            Some(text(r, "Plains"))
        } else if r.prompt.contains("2 of 2") {
            Some(text(r, "Island"))
        } else {
            None
        }
    });
    perm(&mut g, P0, "Alpharael, Dreaming Acolyte");
    assert!(in_gy(&g, P0, "Plains") && in_gy(&g, P0, "Island"), "{}", dump(&log));
    assert!(!in_gy(&g, P0, "Sol Ring"));
    assert_eq!(g.player(P0).hand.len(), 3);
}

// ------------------------------------------------------------------ Glimpse the Unthinkable
#[test]
fn glimpse_the_unthinkable_mills_the_player_the_person_names() {
    let mut g = main_table("kefka");
    hand(&mut g, P0, &["Glimpse the Unthinkable"]);
    pool(&mut g, P0, &all_mana());
    let (a, b) = (g.player(P0).library.len(), g.player(P1).library.len());
    let first = Some(cast_last(&g));
    let log = person(&mut g, first, |r| if r.prompt.contains("choose a target") { Some(not_you(r)) } else { None });
    run(&mut g);
    assert!(asked(&log, "Glimpse the Unthinkable: choose a target"), "{}", dump(&log));
    assert_eq!(g.player(P1).library.len(), b - 10);
    assert_eq!(g.player(P0).library.len(), a);
}

#[test]
fn glimpse_the_unthinkable_can_mill_yourself() {
    let mut g = main_table("kefka");
    hand(&mut g, P0, &["Glimpse the Unthinkable"]);
    pool(&mut g, P0, &all_mana());
    let (a, b) = (g.player(P0).library.len(), g.player(P1).library.len());
    let first = Some(cast_last(&g));
    let _ = person(&mut g, first, |r| if r.prompt.contains("choose a target") { Some(text(r, "(you)")) } else { None });
    run(&mut g);
    assert_eq!(g.player(P0).library.len(), a - 10);
    assert_eq!(g.player(P1).library.len(), b);
}

// ------------------------------------------------------------------ Seize the Spoils
#[test]
fn seize_the_spoils_discards_the_card_the_person_names() {
    let mut g = main_table("kefka");
    hand(&mut g, P0, &["Plains", "Swamp", "Seize the Spoils"]);
    pool(&mut g, P0, &all_mana());
    let first = Some(cast_last(&g));
    let log = person(&mut g, first, |r| {
        if r.prompt.contains("discard a card as an additional cost") { Some(text(r, "Swamp")) } else { None }
    });
    run(&mut g);
    assert!(asked(&log, "discard a card as an additional cost"), "{}", dump(&log));
    assert!(in_gy(&g, P0, "Swamp") && !in_gy(&g, P0, "Plains"));
    assert_eq!(g.player(P0).hand.len(), 3, "Plains plus the two drawn");
    assert_eq!(g.player(P0).treasures, 1, "and a Treasure");
}

// ------------------------------------------------------------------ Twinned Vision (the flashback's discard)
#[test]
fn twinned_vision_flashback_discards_the_card_the_person_names() {
    let mut g = main_table("kefka");
    let tv = take(&mut g, P0, "Twinned Vision");
    g.player_mut(P0).gy.push(tv);
    hand(&mut g, P0, &["Plains", "Swamp"]);
    pool(&mut g, P0, &[('U', 2), ('C', 1)]);
    let first = Some(json!({"do": "cast", "zone": "gy", "card": 0}));
    let log = person(&mut g, first, |r| {
        if r.prompt.contains("which?") {
            Some(text(r, "flashback"))
        } else if r.prompt.contains("flashback): discard a card") {
            Some(text(r, "Swamp"))
        } else {
            None
        }
    });
    run(&mut g);
    assert!(asked(&log, "flashback): discard a card"), "{}", dump(&log));
    assert!(in_gy(&g, P0, "Swamp") && !in_gy(&g, P0, "Plains"));
    assert_eq!(g.player(P0).hand.len(), 3, "Plains and the two cards it drew");
}

// ------------------------------------------------------------------ Chandra's Ignition and Nibelheim Aflame
/// Elves (1 power) is the person's, Grave Titan (6/6) too; the opponent has a Grave Titan and a Birds of Paradise
fn sweep_table() -> (Game, PermId, PermId, PermId) {
    let mut g = main_table("sauron");
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let my_titan = perm(&mut g, P0, "Grave Titan");
    let their_titan = perm(&mut g, P1, "Grave Titan");
    let _ = perm(&mut g, P1, "Birds of Paradise");
    (g, elves, my_titan, their_titan)
}

#[test]
fn chandras_ignition_uses_the_creature_the_person_chooses() {
    let (mut g, elves, my_titan, their_titan) = sweep_table();
    hand(&mut g, P0, &["Chandra's Ignition"]);
    pool(&mut g, P0, &all_mana());
    let life = g.player(P1).life;
    let first = Some(cast_last(&g));
    let log =
        person(&mut g, first, |r| if r.prompt.contains("choose a target") { Some(text(r, "Llanowar")) } else { None });
    run(&mut g);
    assert!(asked(&log, "Chandra's Ignition: choose a target (creature you control)"), "{}", dump(&log));
    assert!(g.player(P0).perms.contains(&elves) && g.player(P0).perms.contains(&my_titan));
    assert!(g.player(P1).perms.contains(&their_titan), "1 damage doesn't kill a 6/6");
    assert!(!has_perm(&g, P1, "Birds of Paradise"), "but it kills the 0/1");
    assert_eq!(g.player(P1).life, life - 1, "each opponent takes damage equal to the chosen creature's power");
}

#[test]
fn chandras_ignition_with_the_big_creature_kills_the_other_titan() {
    let (mut g, _, my_titan, their_titan) = sweep_table();
    hand(&mut g, P0, &["Chandra's Ignition"]);
    pool(&mut g, P0, &all_mana());
    let life = g.player(P1).life;
    let first = Some(cast_last(&g));
    let log =
        person(
            &mut g,
            first,
            |r| if r.prompt.contains("choose a target") { Some(text(r, "Grave Titan")) } else { None },
        );
    run(&mut g);
    assert!(!g.player(P1).perms.contains(&their_titan));
    assert!(g.player(P0).perms.contains(&my_titan), "the source isn't 'each other creature'");
    assert!(g.player(P1).life < life);
}

#[test]
fn nibelheim_aflame_uses_the_creature_the_person_chooses() {
    let (mut g, elves, my_titan, their_titan) = sweep_table();
    hand(&mut g, P0, &["Nibelheim Aflame"]);
    pool(&mut g, P0, &all_mana());
    let first = Some(cast_last(&g));
    let log =
        person(&mut g, first, |r| if r.prompt.contains("choose a target") { Some(text(r, "Llanowar")) } else { None });
    run(&mut g);
    assert!(asked(&log, "Nibelheim Aflame: choose a target (creature you control)"), "{}", dump(&log));
    assert!(g.player(P0).perms.contains(&elves) && g.player(P0).perms.contains(&my_titan));
    assert!(g.player(P1).perms.contains(&their_titan));
    assert!(!has_perm(&g, P1, "Birds of Paradise"));
}

// ------------------------------------------------------------------ Slip Out the Back
#[test]
fn slip_out_the_back_phases_out_the_target_the_person_chooses() {
    let mut g = main_table("sauron");
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let birds = perm(&mut g, P0, "Birds of Paradise");
    hand(&mut g, P0, &["Slip Out the Back"]);
    pool(&mut g, P0, &[('U', 1)]);
    let first = Some(cast_last(&g));
    let log =
        person(&mut g, first, |r| if r.prompt.contains("choose a target") { Some(text(r, "Birds")) } else { None });
    run(&mut g);
    assert!(asked(&log, "Slip Out the Back: choose a target"), "{}", dump(&log));
    assert!(g.perm(birds).phased, "it phases out");
    assert!(!g.perm(elves).phased);
    assert!(in_gy(&g, P0, "Slip Out the Back"));
}

// ------------------------------------------------------------------ Foreboding Ruins
fn ruins_tapped(g: &Game) -> bool {
    let l = g
        .player(P0)
        .lands
        .iter()
        .copied()
        .find(|&l| &*g.perm(l).name == "Foreboding Ruins")
        .expect("on the battlefield");
    g.perm(l).tapped
}

#[test]
fn foreboding_ruins_asks_whether_to_reveal() {
    let mut g = main_table("sauron");
    hand(&mut g, P0, &["Foreboding Ruins", "Swamp"]);
    let first = Some(json!({"do": "land", "card": 0}));
    let log =
        person(
            &mut g,
            first,
            |r| if r.prompt.contains("reveal a card") { Some(text(r, "reveal Swamp")) } else { None },
        );
    run(&mut g);
    assert!(asked(&log, "Foreboding Ruins: reveal a card"), "{}", dump(&log));
    assert!(!ruins_tapped(&g), "revealed: untapped");

    let mut g = main_table("sauron");
    hand(&mut g, P0, &["Foreboding Ruins", "Swamp"]);
    let first = Some(json!({"do": "land", "card": 0}));
    let _ = person(&mut g, first, |r| {
        if r.prompt.contains("reveal a card") { Some(Answer::Pick(r.choices.len() - 1)) } else { None }
    });
    run(&mut g);
    assert!(ruins_tapped(&g), "not revealed: tapped");
}

// ------------------------------------------------------------------ Cavern of Souls
#[test]
fn cavern_of_souls_asks_for_a_creature_type() {
    for key in ["urdragon", "thanos"] {
        let mut g = main_table(key);
        hand(&mut g, P0, &["Cavern of Souls"]);
        let first = Some(json!({"do": "land", "card": 0}));
        let log = person(&mut g, first, |r| {
            if r.prompt.contains("choose a creature type") { Some(Answer::Pick(r.choices.len() - 1)) } else { None }
        });
        run(&mut g);
        assert!(asked(&log, "Cavern of Souls: choose a creature type"), "{key}: {}", dump(&log));
        let l = g.player(P0).lands.iter().copied().find(|&l| &*g.perm(l).name == "Cavern of Souls").unwrap();
        assert!(matches!(g.perm(l).data.get(DataKey::Ctype), Some(Val::Str(_))), "{key}: the type is kept on the land");
    }
}

// ------------------------------------------------------------------ Decimate
#[test]
fn decimate_destroys_the_four_targets_the_person_chooses() {
    let mut g = main_table("urdragon");
    let _ = perm(&mut g, P1, "Sol Ring");
    let elves = perm(&mut g, P1, "Llanowar Elves");
    let birds = perm(&mut g, P1, "Birds of Paradise");
    let _ = perm(&mut g, P1, "Propaganda");
    let forest = lands(&mut g, P1, "Forest", 1, false)[0];
    let swamp = lands(&mut g, P1, "Swamp", 1, false)[0];
    hand(&mut g, P0, &["Decimate"]);
    pool(&mut g, P0, &all_mana());
    let first = Some(cast_last(&g));
    let log = person(&mut g, first, |r| {
        if r.prompt.contains("(artifact)") {
            Some(text(r, "Sol Ring"))
        } else if r.prompt.contains("(creature)") {
            Some(text(r, "Birds"))
        } else if r.prompt.contains("(enchantment)") {
            Some(text(r, "Propaganda"))
        } else if r.prompt.contains("(land)") {
            Some(text(r, "Swamp"))
        } else {
            None
        }
    });
    run(&mut g);
    assert!(asked(&log, "Decimate: choose a target (land)"), "{}", dump(&log));
    assert!(!has_perm(&g, P1, "Sol Ring") && !has_perm(&g, P1, "Propaganda"));
    assert!(!g.player(P1).perms.contains(&birds), "the chosen creature died");
    assert!(g.player(P1).perms.contains(&elves), "the other creature lived");
    assert!(!g.player(P1).lands.contains(&swamp), "the chosen land was destroyed");
    assert!(g.player(P1).lands.contains(&forest));
    assert!(in_gy(&g, P0, "Decimate"));
}

// ------------------------------------------------------------------ Kindred Dominance
#[test]
fn kindred_dominance_asks_for_the_creature_type() {
    let mut g = main_table("urdragon");
    let elves = perm(&mut g, P0, "Llanowar Elves"); // Elf Druid
    let birds = perm(&mut g, P1, "Birds of Paradise"); // Bird
    let elder = perm(&mut g, P1, "Sakura-Tribe Elder"); // Snake Shaman
    hand(&mut g, P0, &["Kindred Dominance"]);
    pool(&mut g, P0, &all_mana());
    let first = Some(cast_last(&g));
    let log =
        person(
            &mut g,
            first,
            |r| if r.prompt.contains("choose a creature type") { Some(text(r, "Bird")) } else { None },
        );
    run(&mut g);
    assert!(asked(&log, "Kindred Dominance: choose a creature type"), "{}", dump(&log));
    assert!(g.player(P1).perms.contains(&birds), "Birds are of the chosen type");
    assert!(!g.player(P0).perms.contains(&elves) && !g.player(P1).perms.contains(&elder));
}

// ------------------------------------------------------------------ Nature's Lore and Rampant Growth
#[test]
fn natures_lore_finds_a_forest_card_untapped() {
    let mut g = main_table("urdragon");
    hand(&mut g, P0, &["Nature's Lore"]);
    pool(&mut g, P0, &all_mana());
    let first = Some(cast_last(&g));
    let log = person(&mut g, first, |_| None);
    run(&mut g);
    assert!(asked(&log, "Nature's Lore: search your library for a Forest card"), "{}", dump(&log));
    assert_eq!(g.player(P0).lands.len(), 1);
    let l = g.player(P0).lands[0];
    assert!(!g.perm(l).tapped, "untapped");
    assert!(in_gy(&g, P0, "Nature's Lore"));
}

#[test]
fn rampant_growth_finds_a_basic_land_tapped() {
    let mut g = main_table("urdragon");
    hand(&mut g, P0, &["Rampant Growth"]);
    pool(&mut g, P0, &all_mana());
    let first = Some(cast_last(&g));
    let log = person(&mut g, first, |_| None);
    run(&mut g);
    assert!(asked(&log, "Search for a basic land"), "{}", dump(&log));
    assert_eq!(g.player(P0).lands.len(), 1);
    assert!(g.perm(g.player(P0).lands[0]).tapped);
}
