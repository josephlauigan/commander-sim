//! Practice mode (2026-11-04, Agent T2, rust/agents/practice-followups.md): triggers later in a game whose target or
//! card the AI picked for the person. Each test seats a scripted person, fires the trigger the way the engine does and
//! checks that the person was asked and that the answer is what happened.

mod play_common;
use play_common::*;
use sim_core::engine::combat::combat;
use sim_core::engine::hooks::fire_trigger;
use sim_core::engine::turn::upkeep;
use sim_core::engine::zones::die;
use sim_core::hooks::{Call, Event};
use sim_core::human::combat::attack_candidates;
use sim_core::ids::CardId;
use sim_core::state::{DataKey, Val};
use std::sync::Arc;

const SEPH: &str = "Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel";
const VENAT: &str = "Venat, Heart of Hydaelyn // Hydaelyn, the Mothercrystal";

/// seat 0 as a person who answers `f` first and otherwise passes priority and declines
fn person(g: &mut Game, f: impl Fn(&Request) -> Option<Answer> + Send + 'static) -> Arc<ScriptHuman> {
    let h = Arc::new(ScriptHuman::from_fn(move |r| {
        f(r).unwrap_or_else(|| {
            if &*r.kind == "priority" { Answer::Action(json!({"do": "pass"})) } else { Answer::Decline }
        })
    }));
    g.humans.add(P0, h.clone());
    h
}

/// pick the first choice containing `word`
fn text(r: &Request, word: &str) -> Option<Answer> {
    Some(Answer::Pick(
        r.choices.iter().position(|c| c.contains(word)).unwrap_or_else(|| panic!("no {word:?} in {:?}", r.choices)),
    ))
}

fn asked(h: &ScriptHuman) -> Vec<String> {
    h.asked().iter().filter(|r| &*r.kind != "priority").map(|r| format!("{}: {}", r.kind, r.prompt)).collect()
}

/// the person was asked this exactly once: the engine probes a queued trigger up to its first window, so a question
/// put before the window would come twice
fn asked_once(h: &ScriptHuman, what: &str) {
    let n = asked(h).iter().filter(|l| l.contains(what)).count();
    assert_eq!(n, 1, "{what:?} asked {n} times: {:?}", asked(h));
}

fn put_gy(g: &mut Game, p: PlayerId, names: &[&str]) {
    for n in names {
        let c = take(g, p, n);
        g.player_mut(p).gy.push(c);
    }
}

fn on_bf(g: &Game, p: PlayerId, name: &str) -> bool {
    g.player(p).perms.iter().any(|&m| g.perm(m).on_bf && &*g.perm(m).name == name)
}

fn name_of(g: &Game, p: PlayerId) -> String {
    g.player(p).name.to_string()
}

// ================================================================== "target player / opponent" drains
#[test]
fn blood_artist_drains_the_player_you_pick() {
    let mut g = table(&["sheoldred", "thrasios", "yshtola"]);
    perm(&mut g, P0, "Blood Artist");
    let elves = perm(&mut g, P0, "Llanowar Elves");
    g.player_mut(P1).life = 10; // the AI's pick would not be the table's healthiest
    let n2 = name_of(&g, P2);
    let h = person(&mut g, move |r| r.prompt.contains("Blood Artist").then(|| text(r, &n2)).flatten());
    die(&mut g, elves, "sac".into()).unwrap();
    asked_once(&h, "Blood Artist: target player");
    assert_eq!((g.player(P1).life, g.player(P2).life, g.player(P0).life), (10, 39, 41));
}

#[test]
fn blood_artist_may_target_you() {
    let mut g = table(&["sheoldred", "thrasios"]);
    perm(&mut g, P0, "Blood Artist");
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let me = format!("{} (player, 40 life) (you)", name_of(&g, P0));
    let h = person(&mut g, move |r| r.prompt.contains("Blood Artist").then(|| text(r, &me)).flatten());
    die(&mut g, elves, "sac".into()).unwrap();
    asked_once(&h, "Blood Artist: target player");
    assert_eq!((g.player(P1).life, g.player(P0).life), (40, 40), "you lose 1 and gain 1");
}

#[test]
fn al_bhed_salvagers_drain_the_opponent_you_pick() {
    let mut g = table(&["sheoldred", "thrasios", "yshtola"]);
    perm(&mut g, P0, "Al Bhed Salvagers");
    let elves = perm(&mut g, P0, "Llanowar Elves");
    g.player_mut(P1).life = 10; // the AI drains the weakest opponent
    let n2 = name_of(&g, P2);
    let h = person(&mut g, move |r| r.prompt.contains("Al Bhed Salvagers").then(|| text(r, &n2)).flatten());
    die(&mut g, elves, "sac".into()).unwrap();
    asked_once(&h, "Al Bhed Salvagers: target opponent");
    assert_eq!((g.player(P1).life, g.player(P2).life, g.player(P0).life), (10, 39, 41));
}

#[test]
fn sephiroth_drains_the_opponent_you_pick() {
    let mut g = table(&["sheoldred", "thrasios", "yshtola"]);
    perm(&mut g, P0, SEPH);
    let elves = perm(&mut g, P0, "Llanowar Elves");
    g.player_mut(P1).life = 10;
    let n2 = name_of(&g, P2);
    let h = person(&mut g, move |r| r.prompt.contains("Sephiroth: target opponent").then(|| text(r, &n2)).flatten());
    die(&mut g, elves, "sac".into()).unwrap();
    asked_once(&h, "Sephiroth: target opponent");
    assert_eq!((g.player(P1).life, g.player(P2).life, g.player(P0).life), (10, 39, 41));
}

// ================================================================== "target creature / card"
#[test]
fn armored_skyhunter_puts_the_card_you_pick_onto_the_creature_you_pick() {
    let mut g = table(&["cloud", "seph"]);
    g.active = Some(P0);
    let sky = perm(&mut g, P0, "Armored Skyhunter");
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let top: Vec<CardId> = ["Plains", "Bonesplitter", "Ethereal Armor", "Plains", "Plains", "Plains"]
        .iter()
        .map(|n| take(&mut g, P0, n))
        .collect();
    g.player_mut(P0).library.extend(top);
    let h = person(&mut g, |r| {
        if r.prompt.contains("put which Aura or Equipment") {
            text(r, "Bonesplitter")
        } else if r.prompt.contains("attach the Equipment to which creature") {
            text(r, "Llanowar")
        } else {
            None
        }
    });
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![sky], d: P1 }).unwrap();
    asked_once(&h, "Armored Skyhunter: put which Aura or Equipment");
    let bone = g.player(P0).perms.iter().copied().find(|&m| &*g.perm(m).name == "Bonesplitter").expect("in play");
    assert_eq!(g.perm(bone).attached, Some(elves), "the person picked the creature");
    assert!(!on_bf(&g, P0, "Ethereal Armor"), "the Aura stayed in the library");
    assert!(
        g.player(P0).library[..6].iter().any(|&c| &*g.db.get(c).name == "Ethereal Armor"),
        "the rest went to the bottom"
    );
}

#[test]
fn armored_skyhunter_may_take_nothing() {
    let mut g = table(&["cloud", "seph"]);
    g.active = Some(P0);
    let sky = perm(&mut g, P0, "Armored Skyhunter");
    let top: Vec<CardId> = ["Plains", "Bonesplitter", "Plains", "Plains", "Plains", "Plains"]
        .iter()
        .map(|n| take(&mut g, P0, n))
        .collect();
    g.player_mut(P0).library.extend(top);
    let h = person(&mut g, |_| None); // "none"
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![sky], d: P1 }).unwrap();
    asked_once(&h, "Armored Skyhunter: put which");
    assert!(!on_bf(&g, P0, "Bonesplitter"));
}

#[test]
fn court_of_ardenvale_returns_the_card_you_pick() {
    let mut g = table(&["yshtola", "seph"]);
    g.active = Some(P0);
    perm(&mut g, P0, "Court of Ardenvale");
    put_gy(&mut g, P0, &["Mind Stone", "Llanowar Elves", "Sol Ring"]);
    let h = person(&mut g, |r| r.prompt.contains("Court of Ardenvale").then(|| text(r, "Llanowar")).flatten());
    upkeep(&mut g, P0).unwrap();
    asked_once(&h, "Court of Ardenvale: return which permanent card");
    // the monarch (the Court makes you one) gets it on the battlefield
    assert!(
        on_bf(&g, P0, "Llanowar Elves") || g.player(P0).hand.iter().any(|&c| &*g.db.get(c).name == "Llanowar Elves")
    );
    assert!(g.player(P0).gy.iter().any(|&c| &*g.db.get(c).name == "Mind Stone"), "the others stayed");
}

#[test]
fn desperate_futurescribe_pumps_the_creature_you_pick() {
    let mut g = table(&["alela", "seph"]);
    g.active = Some(P0);
    perm(&mut g, P0, "Desperate Futurescribe");
    let big = perm(&mut g, P0, "Sakura-Tribe Elder");
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let h = person(&mut g, |r| r.prompt.contains("Desperate Futurescribe").then(|| text(r, "Llanowar")).flatten());
    fire_trigger(&mut g, Event::Crew, Call::Player { p: P0 }).unwrap();
    asked_once(&h, "Desperate Futurescribe: which other creature");
    assert_eq!(g.perm(elves).eot_pt, (1, 1));
    assert_eq!(g.perm(big).eot_pt, (0, 0));
}

#[test]
fn jackdaw_savior_returns_the_creature_card_you_pick() {
    let mut g = table(&["alela", "seph"]);
    g.active = Some(P0);
    let jack = perm(&mut g, P0, "Jackdaw Savior");
    put_gy(&mut g, P0, &["Llanowar Elves", "Birds of Paradise"]);
    let h = person(&mut g, |r| r.prompt.contains("Jackdaw Savior").then(|| text(r, "Llanowar")).flatten());
    die(&mut g, jack, "destroy".into()).unwrap();
    asked_once(&h, "Jackdaw Savior: return which creature card");
    assert!(on_bf(&g, P0, "Llanowar Elves") && !on_bf(&g, P0, "Birds of Paradise"));
}

#[test]
fn helitrooper_gives_flying_to_the_attacker_you_pick() {
    let mut g = table(&["cloud", "seph"]);
    let heli = perm(&mut g, P0, "Helitrooper");
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let big = perm(&mut g, P0, "Craterhoof Behemoth");
    let all: Vec<usize> = attack_candidates(&g, P0)
        .iter()
        .enumerate()
        .filter(|(_, m)| [heli, elves, big].contains(m))
        .map(|(i, _)| i)
        .collect();
    let h = person(&mut g, move |r| {
        if &*r.kind == "attack" {
            Some(Answer::Picks(all.clone()))
        } else if r.prompt.contains("Helitrooper") {
            text(r, "Llanowar")
        } else {
            None
        }
    });
    combat(&mut g, P0).unwrap();
    asked_once(&h, "Helitrooper: which attacking creature gains flying");
    assert!(g.perm(elves).eot_kw.contains(&"flying"), "the AI would have picked the Behemoth");
    assert!(!g.perm(big).eot_kw.contains(&"flying"));
}

#[test]
fn karmic_justice_destroys_the_permanent_you_pick() {
    let mut g = table(&["alela", "seph"]);
    perm(&mut g, P0, "Karmic Justice");
    let rock = perm(&mut g, P0, "Mind Stone");
    let best = perm(&mut g, P1, "Craterhoof Behemoth");
    let elves = perm(&mut g, P1, "Llanowar Elves");
    let h = person(&mut g, |r| r.prompt.contains("Karmic Justice").then(|| text(r, "Llanowar")).flatten());
    g.destroyer = Some(P1);
    die(&mut g, rock, "destroy".into()).unwrap();
    g.destroyer = None;
    asked_once(&h, "Karmic Justice: destroy which");
    assert!(!g.perm(elves).on_bf, "the person picked the Elves");
    assert!(g.perm(best).on_bf);
}

#[test]
fn karmic_justice_may_destroy_nothing() {
    let mut g = table(&["alela", "seph"]);
    perm(&mut g, P0, "Karmic Justice");
    let rock = perm(&mut g, P0, "Mind Stone");
    let best = perm(&mut g, P1, "Craterhoof Behemoth");
    person(&mut g, |_| None); // every question declined
    g.destroyer = Some(P1);
    die(&mut g, rock, "destroy".into()).unwrap();
    assert!(g.perm(best).on_bf);
}

#[test]
fn hydaelyns_blessing_goes_on_the_creature_you_pick() {
    let mut g = table(&["yshtola", "seph"]);
    let venat = perm(&mut g, P0, VENAT);
    g.perm_mut(venat).data.set(DataKey::Hydaelyn, Val::Bool(true));
    let big = perm(&mut g, P0, "Craterhoof Behemoth");
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let h = person(&mut g, |r| r.prompt.contains("Blessing of Light").then(|| text(r, "Llanowar")).flatten());
    fire_trigger(&mut g, Event::Crew, Call::Player { p: P0 }).unwrap();
    asked_once(&h, "Blessing of Light: which creature");
    assert_eq!(g.perm(elves).plus, 1);
    assert_eq!(g.perm(big).plus, 0);
}

#[test]
fn zur_searches_for_the_enchantment_you_pick() {
    let mut g = table(&["sheoldred", "seph"]);
    g.active = Some(P0);
    let zur = perm(&mut g, P0, "Zur the Enchanter");
    let lib: Vec<CardId> = ["Necropotence", "Propaganda"].iter().map(|n| take(&mut g, P0, n)).collect();
    g.player_mut(P0).library.extend(lib);
    let h =
        person(&mut g, |r| if r.prompt.contains("Zur the Enchanter: search") { text(r, "Propaganda") } else { None });
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![zur], d: P1 }).unwrap();
    asked_once(&h, "Zur the Enchanter: search");
    assert!(on_bf(&g, P0, "Propaganda"), "the AI would have taken Necropotence first");
    assert!(!on_bf(&g, P0, "Necropotence"));
}

#[test]
fn zur_may_find_nothing() {
    let mut g = table(&["sheoldred", "seph"]);
    let zur = perm(&mut g, P0, "Zur the Enchanter");
    let c = take(&mut g, P0, "Propaganda");
    g.player_mut(P0).library.push(c);
    person(&mut g, |_| None);
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![zur], d: P1 }).unwrap();
    assert!(!on_bf(&g, P0, "Propaganda"));
}

#[test]
fn zur_enchants_the_creature_you_pick_with_an_aura() {
    let mut g = table(&["sheoldred", "seph"]);
    let zur = perm(&mut g, P0, "Zur the Enchanter");
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let c = take(&mut g, P0, "Ethereal Armor");
    g.player_mut(P0).library.push(c);
    let h = person(&mut g, |r| {
        if r.prompt.contains("Zur the Enchanter: search") {
            text(r, "Ethereal Armor")
        } else if r.prompt.contains("Zur the Enchanter: which creature") {
            text(r, "Llanowar")
        } else {
            None
        }
    });
    fire_trigger(&mut g, Event::Attack, Call::Attack { p: P0, atk: vec![zur], d: P1 }).unwrap();
    asked_once(&h, "which creature does Ethereal Armor enchant");
    let aura = g.player(P0).perms.iter().copied().find(|&m| &*g.perm(m).name == "Ethereal Armor").expect("in play");
    assert_eq!(g.perm(aura).attached, Some(elves));
}

#[test]
fn reya_dawnbringer_returns_the_creature_card_you_pick() {
    let mut g = table(&["galadriel", "seph"]);
    g.active = Some(P0);
    perm(&mut g, P0, "Reya Dawnbringer");
    put_gy(&mut g, P0, &["Llanowar Elves", "Craterhoof Behemoth"]);
    let h = person(&mut g, |r| {
        if r.choices.iter().any(|c| c == "yes") {
            text(r, "yes") // "you may return ..."
        } else if r.prompt.contains("Reya Dawnbringer") {
            text(r, "Llanowar")
        } else {
            None
        }
    });
    upkeep(&mut g, P0).unwrap();
    asked_once(&h, "Reya Dawnbringer: return which card");
    assert!(on_bf(&g, P0, "Llanowar Elves"), "the person picked the Elves");
    assert!(!on_bf(&g, P0, "Craterhoof Behemoth"), "the AI would have taken the Behemoth");
}

// ================================================================== "you may" the compiled card did not ask
fn mask_table() -> (Game, PermId) {
    let mut g = table(&["cloud", "seph"]);
    let hero = perm(&mut g, P0, "Llanowar Elves");
    let mask = perm(&mut g, P0, "Mask of Memory");
    g.perm_mut(mask).attached = Some(hero);
    (g, hero)
}

#[test]
fn mask_of_memory_may_be_declined() {
    let (mut g, hero) = mask_table();
    let i = attack_candidates(&g, P0).iter().position(|&m| m == hero).unwrap();
    let h = person(&mut g, move |r| {
        if &*r.kind == "attack" {
            Some(Answer::Picks(vec![i]))
        } else if r.prompt.contains("Mask of Memory") {
            text(r, "no")
        } else {
            None
        }
    });
    let hand_before = g.player(P0).hand.len();
    combat(&mut g, P0).unwrap();
    asked_once(&h, "Mask of Memory: draw");
    assert_eq!(g.player(P0).hand.len(), hand_before, "no cards drawn, none discarded");
}

#[test]
fn mask_of_memory_draws_two_and_discards_the_card_you_pick() {
    let (mut g, hero) = mask_table();
    hand(&mut g, P0, &["Sol Ring"]);
    let i = attack_candidates(&g, P0).iter().position(|&m| m == hero).unwrap();
    let h = person(&mut g, move |r| {
        if &*r.kind == "attack" {
            Some(Answer::Picks(vec![i]))
        } else if r.prompt.contains("Mask of Memory") {
            text(r, "yes")
        } else if r.prompt.contains("Discard") {
            text(r, "Sol Ring")
        } else {
            None
        }
    });
    combat(&mut g, P0).unwrap();
    asked_once(&h, "Mask of Memory: draw");
    assert!(g.player(P0).gy.iter().any(|&c| &*g.db.get(c).name == "Sol Ring"), "the person chose the discard");
    assert_eq!(g.player(P0).hand.len(), 2, "one card in hand, plus two drawn, minus one discarded");
}

// ================================================================== Adeline: one token per opponent, player or walker
#[test]
fn adeline_tokens_attack_the_walker_you_pick() {
    let mut g = table(&["cloud", "sheoldred", "thrasios"]);
    let adeline = perm(&mut g, P0, "Adeline, Resplendent Cathar");
    let walker = perm(&mut g, P2, "Chandra, Torch of Defiance");
    let loyalty = g.perm(walker).loyalty.unwrap();
    let i = attack_candidates(&g, P0).iter().position(|&m| m == adeline).unwrap();
    let (n1, n2) = (name_of(&g, P1), name_of(&g, P2));
    let h = person(&mut g, move |r| {
        if &*r.kind == "attack" {
            Some(Answer::Picks(vec![i]))
        } else if r.prompt.starts_with("Attack which player") {
            text(r, &n1)
        } else if r.prompt.contains(&format!("Adeline: the token for {n2}")) {
            text(r, "planeswalker")
        } else {
            None
        }
    });
    combat(&mut g, P0).unwrap();
    asked_once(&h, "Adeline: the token for");
    assert!(g.perm(walker).loyalty.unwrap() < loyalty, "the token went after the planeswalker");
    assert_eq!(g.player(P2).life, 40, "and not the player");
    assert!(g.player(P1).life < 40, "Adeline and the other token hit their opponent");
}

// ================================================================== asks already (probes of the rest)
#[test]
fn cavalier_of_dawn_asks_for_its_target_and_the_card_to_return() {
    let mut g = table(&["cloud", "seph"]);
    let best = perm(&mut g, P1, "Craterhoof Behemoth");
    let elves = perm(&mut g, P1, "Llanowar Elves");
    put_gy(&mut g, P0, &["Sol Ring", "Mind Stone"]);
    let h = person(&mut g, |r| {
        if r.prompt.contains("choose a target") {
            text(r, "Llanowar")
        } else if r.prompt.contains("return which card from your graveyard") {
            text(r, "Mind Stone")
        } else {
            None
        }
    });
    let cav = perm(&mut g, P0, "Cavalier of Dawn");
    asked_once(&h, "Cavalier of Dawn: choose a target");
    assert!(!g.perm(elves).on_bf && g.perm(best).on_bf, "the person picked the Elves");
    die(&mut g, cav, "destroy".into()).unwrap();
    asked_once(&h, "Cavalier of Dawn: return which card from your graveyard");
    assert!(g.player(P0).hand.iter().any(|&c| &*g.db.get(c).name == "Mind Stone"));
}

// ================================================================== The Wasp (fixed earlier today): asked once, not twice
#[test]
fn the_wasp_asks_once_when_it_enters() {
    let mut g = table(&["thrasios", "seph"]);
    let ant = perm(&mut g, P0, "Ant-Man, Elusive Avenger");
    let h = person(&mut g, |r| r.prompt.contains("The Wasp").then(|| text(r, "Ant-Man")).flatten());
    perm(&mut g, P0, "The Wasp, Winsome Avenger");
    asked_once(&h, "The Wasp: which Hero gains hexproof");
    assert!(g.perm(ant).eot_kw.contains(&"hexproof"));
}

#[test]
fn the_wasp_asks_once_when_it_attacks() {
    let mut g = table(&["thrasios", "seph"]);
    let wasp = perm(&mut g, P0, "The Wasp, Winsome Avenger");
    let elves = perm(&mut g, P1, "Llanowar Elves");
    perm(&mut g, P1, "Birds of Paradise");
    let i = attack_candidates(&g, P0).iter().position(|&m| m == wasp).unwrap();
    let h = person(&mut g, move |r| {
        if &*r.kind == "attack" {
            Some(Answer::Picks(vec![i]))
        } else if r.prompt.contains("The Wasp: tap which") {
            text(r, "Llanowar")
        } else {
            None
        }
    });
    combat(&mut g, P0).unwrap();
    asked_once(&h, "The Wasp: tap which creature");
    assert!(g.perm(elves).tapped);
}

// ================================================================== the Swords: asks already
fn sword_swing(sword: &str) -> (Game, PermId, PermId) {
    let mut g = table(&["cloud", "seph"]);
    let hero = perm(&mut g, P0, "Llanowar Elves");
    let s = perm(&mut g, P0, sword);
    g.perm_mut(s).attached = Some(hero);
    (g, hero, s)
}

#[test]
fn sword_of_fire_and_ice_asks_where_the_two_damage_goes() {
    let (mut g, hero, _) = sword_swing("Sword of Fire and Ice");
    let victim = perm(&mut g, P1, "Birds of Paradise");
    let i = attack_candidates(&g, P0).iter().position(|&m| m == hero).unwrap();
    let h = person(&mut g, move |r| {
        if &*r.kind == "attack" {
            Some(Answer::Picks(vec![i]))
        } else if r.prompt.contains("deal 2 damage") {
            text(r, "Birds of Paradise")
        } else {
            None
        }
    });
    let hand_before = g.player(P0).hand.len();
    combat(&mut g, P0).unwrap();
    asked_once(&h, "deal 2 damage to which target");
    assert!(!g.perm(victim).on_bf, "the person aimed it at the Birds");
    assert_eq!(g.player(P0).hand.len(), hand_before + 1, "and drew a card");
}

#[test]
fn sword_of_hearth_and_home_asks_which_creature_and_which_basic() {
    let (mut g, hero, _) = sword_swing("Sword of Hearth and Home");
    let other = perm(&mut g, P0, "Birds of Paradise");
    let i = attack_candidates(&g, P0).iter().position(|&m| m == hero).unwrap();
    let lands_before = g.player(P0).lands.len();
    let h = person(&mut g, move |r| {
        if &*r.kind == "attack" {
            Some(Answer::Picks(vec![i]))
        } else if r.prompt.contains("choose a target") {
            text(r, "Birds of Paradise")
        } else if r.prompt.contains("Search for a basic land") {
            text(r, "Plains")
        } else {
            None
        }
    });
    combat(&mut g, P0).unwrap();
    asked_once(&h, "Sword of Hearth and Home: choose a target");
    asked_once(&h, "Search for a basic land");
    assert_eq!(g.player(P0).lands.len(), lands_before + 1);
    assert!(!g.perm(other).on_bf && on_bf(&g, P0, "Birds of Paradise"), "the Birds were exiled and came back");
}
