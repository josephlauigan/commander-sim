//! Practice mode: your decks' activated abilities by the rules (play/cards.py), Python's tests/test_play_cards.py. The
//! card code offers them to the AI only when its plan says so, with its own targets; the human seat gets them
//! whenever the card has them, with the reason when one can't be used, and picks the targets.

mod play_common;
use play_common::*;
use sim_core::engine::{cast, zones};
use sim_core::human::human::{human_main, respond};
use sim_core::state::{DataKey, Val};

/// activate an ability of permanent m in p's main phase, answering the follow-up questions
fn use_perm(g: &mut Game, p: PlayerId, m: PermId, answers: Vec<F>) -> std::sync::Arc<ScriptHuman> {
    let idx = g.player(p).perms.iter().position(|&x| x == m).unwrap();
    let mut a = vec![act(json!({"do": "use", "perm": idx}))];
    a.extend(answers);
    a.push(pass());
    let h = seat(g, p, a);
    human_main(g, p, false).unwrap();
    h
}

fn gy_push(g: &mut Game, p: PlayerId, name: &str) {
    let c = card(g, name);
    g.player_mut(p).gy.push(c);
}

// ------------------------------------------------------------------ Walkers
#[test]
fn vraska_minus_two_on_your_pick() {
    let mut g = table(&["sauron", "veyran"]);
    let vr = perm(&mut g, P0, "Vraska, Betrayal's Sting");
    let small = perm(&mut g, P1, "Guttersnipe");
    perm(&mut g, P1, "Archmage Emeritus");
    use_perm(&mut g, P0, vr, vec![by_text("-2"), by_text("Guttersnipe")]);
    assert!(!g.player(P1).perms.contains(&small));
    assert_eq!((g.perm(vr).loyalty, g.player(P1).treasures), (Some(4), 1));
}

#[test]
fn once_per_turn_and_sorcery_speed() {
    let mut g = table(&["sauron", "veyran"]);
    let vr = perm(&mut g, P0, "Vraska, Betrayal's Sting");
    let idx = g.player(P0).perms.iter().position(|&x| x == vr).unwrap();
    let h = use_perm(&mut g, P0, vr, vec![by_text("0:"), act(json!({"do": "use", "perm": idx})), by_text("0:")]);
    assert!(told_last(&h).contains("already used a loyalty ability"), "{}", told_last(&h));
    g.active = Some(P1);
    let h = use_perm(&mut g, P0, vr, vec![by_text("0:")]);
    assert!(told_last(&h).contains("isn't your turn"), "{}", told_last(&h));
}

#[test]
fn ral_zarek_returns_your_pick() {
    let mut g = table(&["sauron", "veyran"]);
    let rz = perm(&mut g, P0, "Ral Zarek, Guest Lecturer");
    gy_push(&mut g, P0, "Orcish Bowmasters");
    gy_push(&mut g, P0, "Metallic Mimic");
    use_perm(&mut g, P0, rz, vec![by_text("-2"), by_text("Metallic Mimic")]);
    assert!(has_perm(&g, P0, "Metallic Mimic"));
    let bow = card(&g, "Orcish Bowmasters");
    assert!(g.player(P0).gy.contains(&bow));
}

// ------------------------------------------------------------------ Permanents
#[test]
fn mind_stone_whenever() {
    let mut g = table(&["seph", "veyran"]);
    let ms = perm(&mut g, P0, "Mind Stone");
    pool(&mut g, P0, &[('C', 1)]);
    let n = g.player(P0).hand.len();
    use_perm(&mut g, P0, ms, vec![pick(0)]);
    assert!(!g.player(P0).perms.contains(&ms));
    assert_eq!(g.player(P0).hand.len(), n + 1);
}

#[test]
fn mind_stone_needs_mana() {
    let mut g = table(&["seph", "veyran"]);
    let ms = perm(&mut g, P0, "Mind Stone");
    let h = use_perm(&mut g, P0, ms, vec![pick(0)]);
    assert!(told_last(&h).contains("Can't activate Mind Stone"), "{}", told_last(&h));
    assert!(g.player(P0).perms.contains(&ms));
}

#[test]
fn prepared_lightning_bolt() {
    let mut g = table(&["veyran", "seph"]);
    let em = perm(&mut g, P0, "Emeritus of Conflict // Lightning Bolt");
    g.perm_mut(em).data.set(DataKey::Prepared, Val::Bool(true));
    pool(&mut g, P0, &[('R', 1)]);
    use_perm(&mut g, P0, em, vec![pick(0), by_text("Sephiroth")]);
    assert_eq!(g.player(P1).life, 37);
    assert!(!g.perm(em).data.truthy(DataKey::Prepared));
}

#[test]
fn aetherflux_reservoir() {
    let mut g = table(&["veyran", "seph"]);
    let ax = perm(&mut g, P0, "Aetherflux Reservoir");
    g.player_mut(P0).life = 60;
    use_perm(&mut g, P0, ax, vec![pick(0), by_text("Sephiroth")]);
    assert_eq!((g.player(P0).life, g.player(P1).alive), (10, false));
}

#[test]
fn aetherflux_needs_fifty_life() {
    let mut g = table(&["veyran", "seph"]);
    let ax = perm(&mut g, P0, "Aetherflux Reservoir");
    let h = use_perm(&mut g, P0, ax, vec![pick(0)]);
    assert!(told_last(&h).contains("You need 50 life"), "{}", told_last(&h));
}

#[test]
fn triskelion() {
    let mut g = table(&["seph", "veyran"]);
    let t = perm(&mut g, P0, "Triskelion");
    g.perm_mut(t).plus = 3;
    let gs = perm(&mut g, P1, "Kessig Flamebreather");
    use_perm(&mut g, P0, t, vec![pick(0), by_text("Kessig")]);
    assert_eq!(g.perm(t).plus, 2);
    assert!(g.player(P1).perms.contains(&gs)); // 1/3: it survives one damage
}

// ------------------------------------------------------------------ Lands
#[test]
fn strip_mine_any_land() {
    let mut g = table(&["seph", "veyran"]);
    lands(&mut g, P0, "Strip Mine", 1, false);
    lands(&mut g, P1, "Island", 2, false);
    seat(&mut g, P0, vec![act(json!({"do": "use", "land": 0})), pick(0), by_text("Island"), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert_eq!((g.player(P0).lands.len(), g.player(P1).lands.len()), (0, 1));
}

#[test]
fn desolate_lighthouse_loots() {
    let mut g = table(&["veyran", "seph"]);
    let l = lands(&mut g, P0, "Desolate Lighthouse", 1, false)[0];
    pool(&mut g, P0, &[('U', 1), ('R', 2)]);
    hand(&mut g, P0, &["Counterspell"]);
    let n = g.player(P0).hand.len();
    seat(&mut g, P0, vec![act(json!({"do": "use", "land": 0})), pick(0), pick(0), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert_eq!((g.player(P0).hand.len(), g.player(P0).gy.len()), (n, 1));
    assert!(g.perm(l).tapped);
}

// ------------------------------------------------------------------ Citadel
#[test]
fn cast_the_top_card_for_life() {
    let mut g = table(&["seph", "veyran"]);
    let cit = perm(&mut g, P0, "Bolas's Citadel");
    let top = card(&g, "Mind Stone");
    g.player_mut(P0).library.push(top);
    let life = g.player(P0).life;
    use_perm(&mut g, P0, cit, vec![by_text("Mind Stone")]);
    assert!(has_perm(&g, P0, "Mind Stone"));
    assert_eq!(g.player(P0).life, life - 2);
}

#[test]
fn play_the_top_land() {
    let mut g = table(&["seph", "veyran"]);
    let cit = perm(&mut g, P0, "Bolas's Citadel");
    let top = card(&g, "Command Tower");
    g.player_mut(P0).library.push(top);
    let n = g.player(P0).lands.len();
    use_perm(&mut g, P0, cit, vec![by_text("play Command Tower")]);
    assert_eq!(g.player(P0).lands.len(), n + 1);
}

#[test]
fn sacrifice_ten() {
    let mut g = table(&["seph", "veyran"]);
    let cit = perm(&mut g, P0, "Bolas's Citadel");
    for _ in 0..9 {
        let mut spec = zones::Tokens::new(1, 1);
        spec.sick = false;
        zones::make_tokens(&mut g, P0, spec).unwrap();
    }
    let mut a: Vec<F> = vec![by_text("sacrifice ten")];
    a.extend((0..9).map(|_| pick(0)));
    use_perm(&mut g, P0, cit, a);
    assert_eq!(g.player(P1).life, 30);
    assert!(!g.player(P0).perms.contains(&cit));
}

// ------------------------------------------------------------------ Mistrise Village
#[test]
fn next_spell_cant_be_countered() {
    let mut g = table(&["veyran", "seph"]);
    lands(&mut g, P0, "Mistrise Village", 1, false);
    pool(&mut g, P0, &[('U', 1)]);
    seat(&mut g, P0, vec![act(json!({"do": "use", "land": 0})), pick(0), pass()]);
    human_main(&mut g, P0, false).unwrap();
    let c = card(&g, "Think Twice");
    cast::on_cast(&mut g, P0, c).unwrap();
    assert!(sim_core::engine::stack::mistrised(&g, c));
    let other = card(&g, "Counterspell");
    assert!(!sim_core::engine::stack::mistrised(&g, other)); // only the next spell
}

// ------------------------------------------------------------------ Responding
#[test]
fn no_sorcery_speed_play_in_response() {
    let mut g = table(&["sauron", "veyran"]);
    let vr = perm(&mut g, P0, "Vraska, Betrayal's Sting");
    let idx = g.player(P0).perms.iter().position(|&x| x == vr).unwrap();
    let h = seat(&mut g, P0, vec![act(json!({"do": "use", "perm": idx})), by_text("0:"), pass()]);
    respond(&mut g, P0, "Veyran casts Opt", None, None).unwrap();
    assert!(told_last(&h).contains("not in response"), "{}", told_last(&h));
}
