//! Phase 3, the replacement step (`engine::repl`). Stage A: the step's readers give what the old helpers gave (their
//! bodies are copied here as `old_*`, reading names) on tricky boards: Doubling Season + Hardened Scales +
//! Vorinclex, Melira with infect and wither, Rest in Peace and a dying creature with a dies trigger, Solemnity with an
//! undying creature, damage prevention. Stage C: the counters step's coverage and order fixes, worked by hand.
//! Seat 0 plays the sauron deck, seat 1 the seph deck.

use sim_core::engine::repl::{self, Ctr, On, Path, ReplFx};
use sim_core::engine::{life, values, zones};
use sim_core::ids::{PermId, PlayerId};
use sim_core::state::Game;
use sim_core::testkit::*;

const P0: PlayerId = PlayerId(0);
const P1: PlayerId = PlayerId(1);

fn game() -> Game {
    table(&["sauron", "seph"])
}

fn name(g: &Game, m: PermId) -> Option<&str> {
    values::card_name(g, m)
}

// ------------------------------------------------------------------ the old helpers, as they were
fn old_scales(g: &Game, p: PlayerId) -> i32 {
    let named = |y: &PermId| name(g, *y) == Some("Hardened Scales");
    g.player(p).perms.iter().filter(|y| g.perm(**y).on_bf && !g.perm(**y).phased && named(y)).count() as i32
}

fn old_vorinclex(g: &Game, actor: PlayerId, n: i32) -> i32 {
    let mut k = n.abs();
    for pl in &g.players {
        for &y in &pl.perms {
            let x = g.perm(y);
            if x.on_bf && !x.phased && name(g, y) == Some("Vorinclex, Monstrous Raider") {
                k = if pl.id == actor { k * 2 } else { k / 2 };
            }
        }
    }
    k * n.signum()
}

fn old_solemnity(g: &Game) -> bool {
    g.players.iter().any(|q| {
        q.perms.iter().any(|&m| {
            let x = g.perm(m);
            x.on_bf && !x.phased && !x.neutered && name(g, m) == Some("Solemnity")
        })
    })
}

fn old_melira(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| name(g, m) == Some("Melira, Sylvok Outcast") && !g.perm(m).phased)
        || old_solemnity(g)
}

fn old_hushed(g: &Game) -> bool {
    g.players.iter().filter(|q| q.alive).flat_map(|q| q.perms.iter()).any(|&m| {
        let x = g.perm(m);
        !x.phased && !x.neutered && name(g, m) == Some("Hushbringer")
    })
}

/// every old counters reader agrees with the step on this table
fn agree(g: &Game) {
    for p in [P0, P1] {
        assert_eq!(repl::scales(g, p), old_scales(g, p));
        assert_eq!(repl::melira(g, p), old_melira(g, p));
        for n in [-5, -3, -1, 1, 2, 3, 7] {
            assert_eq!(repl::vorinclex(g, p, n), old_vorinclex(g, p, n), "vorinclex {p:?} {n}");
        }
    }
    assert_eq!(repl::solemnity(g), old_solemnity(g));
    assert_eq!(repl::hushed(g), old_hushed(g));
}

// ------------------------------------------------------------------ the table
#[test]
fn the_table_is_read_once_per_card() {
    let g = game();
    for (n, fx) in [
        ("Hardened Scales", ReplFx::Scales),
        ("Vorinclex, Monstrous Raider", ReplFx::Vorinclex),
        ("Solemnity", ReplFx::Solemnity),
        ("Melira, Sylvok Outcast", ReplFx::Melira),
        ("Mauhúr, Uruk-hai Captain", ReplFx::Mauhur),
        ("Cho-Manno, Revolutionary", ReplFx::NoDamage),
        ("Tajic, Legion's Edge", ReplFx::Tajic),
        ("Academy Manufactor", ReplFx::Manufactor),
        ("Hushbringer", ReplFx::Hushbringer),
        ("Laboratory Maniac", ReplFx::LabMan),
        ("Jace, Wielder of Mysteries", ReplFx::LabMan),
        ("Spelunking", ReplFx::Spelunking),
    ] {
        assert_eq!(g.db.get(card(&g, n)).repl, Some(fx), "{n}");
    }
    assert_eq!(g.db.get(card(&g, "Llanowar Elves")).repl, None);
    for nm in repl::NAMED.iter() {
        assert!(g.db.id(nm.name).is_some(), "{} is not in the card pool", nm.name);
        assert_eq!(repl::ev_of(nm.fx), nm.ev);
    }
}

// ------------------------------------------------------------------ counters
#[test]
fn doubling_season_scales_and_vorinclex_agree_with_the_old_helpers() {
    let mut g = game();
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let theirs = perm(&mut g, P1, "Llanowar Elves");
    agree(&g);
    perm(&mut g, P0, "Hardened Scales");
    agree(&g);
    perm(&mut g, P0, "Doubling Season"); // its compiled `counters` doubler
    g.dsl_on = true;
    agree(&g);
    perm(&mut g, P0, "Vorinclex, Monstrous Raider");
    agree(&g);
    perm(&mut g, P1, "Vorinclex, Monstrous Raider"); // doubles for seat 1, halves for seat 0
    perm(&mut g, P0, "Hardened Scales");
    agree(&g);
    // Stage C, worked by hand. Seat 0 puts 1 on its Elves: two Scales (+1, +1), its Doubling Season (x 2), its
    // Vorinclex (x 2), seat 1's (/ 2). Seat 0 orders them: halve first (0), then +2, x 2, x 2 = 8
    assert_eq!(repl::counters(&g, P0, On::Perm(elves), Ctr::Plus, 1, Path::ALL), 8);
    // seat 1 puts 3 on its own Elves: no Scales or Doubling Season of its own; double (its Vorinclex) then halve: 3
    assert_eq!(repl::counters(&g, P1, On::Perm(theirs), Ctr::Plus, 3, Path::ALL), 3);
    // seat 1 puts 1 on seat 0's Elves: seat 0's Scales and Doubling Season apply (the controller's), the
    // Vorinclexes swap roles: still 8
    assert_eq!(repl::counters(&g, P1, On::Perm(elves), Ctr::Plus, 1, Path::ALL), 8);
    // seat 0 puts 1 on seat 1's Elves: no doubling of seat 0's; double then halve: 1
    assert_eq!(repl::counters(&g, P0, On::Perm(theirs), Ctr::Plus, 1, Path::ALL), 1);
    let before = g.perm(elves).plus;
    repl::put_plus(&mut g, elves, 1);
    assert_eq!(g.perm(elves).plus - before, 8, "put_plus goes through every effect");
}

#[test]
fn doubling_season_doubles_loyalty_once_as_a_planeswalker_enters() {
    let mut g = game();
    perm(&mut g, P0, "Doubling Season");
    let karn = perm(&mut g, P0, "Karn Liberated"); // starting loyalty 6
    assert_eq!(g.perm(karn).loyalty, Some(12));
    perm(&mut g, P0, "Vorinclex, Monstrous Raider");
    let other = perm(&mut g, P1, "Karn Liberated"); // seat 1: no Doubling Season, seat 0's Vorinclex halves
    assert_eq!(g.perm(other).loyalty, Some(3));
}

#[test]
fn melira_stops_poison_and_minus_counters_from_infect_and_wither() {
    let mut g = game();
    let mine = perm(&mut g, P1, "Llanowar Elves");
    perm(&mut g, P1, "Melira, Sylvok Outcast");
    agree(&g);
    assert!(repl::melira(&g, P1) && !repl::melira(&g, P0));
    assert!(!zones::minus_counter(&mut g, mine, 1), "no -1/-1 counter on Melira's side");
    assert_eq!(g.perm(mine).plus, 0);
    assert_eq!(repl::counters(&g, P0, On::Player(P1), Ctr::Poison, 3, Path::POISON), 0);
    assert_eq!(repl::counters(&g, P1, On::Player(P0), Ctr::Poison, 3, Path::POISON), 3);
    // a phased-out Melira stops nothing
    let melira = g.player(P1).perms.iter().copied().find(|&m| g.perm(m).name.contains("Melira")).unwrap();
    g.perm_mut(melira).phased = true;
    agree(&g);
    assert!(!repl::melira(&g, P1));
}

#[test]
fn solemnity_and_an_undying_creature() {
    let mut g = game();
    perm(&mut g, P0, "Solemnity");
    agree(&g);
    let bear = perm(&mut g, P0, "Butcher Ghoul"); // undying
    repl::put_plus(&mut g, bear, 2);
    assert_eq!(g.perm(bear).plus, 0, "Solemnity: no counters");
    zones::die(&mut g, bear, "destroy").unwrap();
    // undying returns it with no counter (Solemnity), so it can return again
    let back = g.player(P0).perms.iter().copied().find(|&m| name(&g, m) == Some("Butcher Ghoul"));
    let back = back.expect("undying returned it");
    assert_eq!(g.perm(back).plus, 0);
    assert!(g.perm(back).undying);
    // loyalty counters still go on a planeswalker
    let karn = perm(&mut g, P0, "Karn Liberated");
    assert_eq!(g.perm(karn).loyalty, Some(6));
    assert_eq!(repl::counters(&g, P0, On::Perm(karn), Ctr::Loyalty, 2, Path::ALL), 2);
    assert_eq!(repl::counters(&g, P0, On::Player(P1), Ctr::Poison, 2, Path::ALL), 0, "players get none");
}

// ------------------------------------------------------------------ graveyard and triggers
#[test]
fn rest_in_peace_exiles_a_dying_creature_and_its_dies_trigger_does_not_happen() {
    let mut g = game();
    let sim = perm(&mut g, P0, "Solemn Simulacrum"); // when it dies, you may draw a card
    perm(&mut g, P1, "Rest in Peace");
    assert!(repl::no_graveyard(&g, P0) && repl::no_graveyard(&g, P1));
    let cd = g.perm(sim).cd.unwrap();
    let hand = g.player(P0).hand.len();
    zones::die(&mut g, sim, "destroy").unwrap();
    assert!(g.player(P0).exile.contains(&cd) && !g.player(P0).gy.contains(&cd));
    assert_eq!(g.player(P0).hand.len(), hand, "no dies trigger: it never died");
}

#[test]
fn hushbringer_is_read_through_the_table() {
    let mut g = game();
    agree(&g);
    let h = perm(&mut g, P1, "Hushbringer");
    agree(&g);
    assert!(repl::hushed(&g));
    g.perm_mut(h).neutered = true;
    agree(&g);
    assert!(!repl::hushed(&g));
}

// ------------------------------------------------------------------ damage
#[test]
fn damage_prevention_cho_manno_and_tajic() {
    let mut g = game();
    let cho = perm(&mut g, P0, "Cho-Manno, Revolutionary");
    let elves = perm(&mut g, P0, "Llanowar Elves");
    assert!(repl::no_damage(&g, cho) && !repl::no_damage(&g, elves));
    g.perm_mut(cho).neutered = true;
    assert!(!repl::no_damage(&g, cho), "abilities lost: no prevention");
    g.perm_mut(cho).neutered = false;
    let tajic = perm(&mut g, P0, "Tajic, Legion's Edge");
    assert!(repl::tajic_prevents(&g, elves) && !repl::tajic_prevents(&g, tajic));
    // life: gain goes through the step (no gain effects out: unchanged)
    let before = g.player(P0).life;
    life::gain(&mut g, P0, 3).unwrap();
    assert_eq!(g.player(P0).life, before + 3);
}

// ------------------------------------------------------------------ tokens
#[test]
fn token_doublers_through_the_step() {
    let mut g = game();
    g.dsl_on = true;
    assert_eq!(repl::tokens(&g, P0, 2), 2);
    perm(&mut g, P0, "Anointed Procession");
    perm(&mut g, P0, "Doubling Season");
    assert_eq!(repl::tokens(&g, P0, 2), 8);
    assert_eq!(repl::tokens(&g, P1, 2), 2);
    assert_eq!(repl::token_mult(&g, P0), 4);
}

// ================================================================== Stage C: coverage and order
#[test]
fn doubling_season_doubles_every_placement_on_its_controllers_permanents() {
    let mut g = game();
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let theirs = perm(&mut g, P1, "Llanowar Elves");
    perm(&mut g, P0, "Doubling Season");
    // card code's counters (`put_plus`) used to miss it: now twice as many
    repl::put_plus(&mut g, elves, 1);
    assert_eq!(g.perm(elves).plus, 2);
    // by the permanent's controller, not the actor: seat 1's effect on seat 0's Elves doubles, seat 0's on seat 1's not
    assert_eq!(repl::put_counters(&mut g, P1, elves, 1), 2);
    assert_eq!(repl::put_counters(&mut g, P0, theirs, 1), 1);
}

#[test]
fn the_controller_orders_scales_and_the_doublers() {
    let mut g = game();
    let elves = perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Hardened Scales");
    perm(&mut g, P0, "Doubling Season");
    // Scales first, then the doubling: (1 + 1) x 2 = 4, not 1 x 2 + 1 = 3 (616.1: the controller picks)
    assert_eq!(repl::counters(&g, P0, On::Perm(elves), Ctr::Plus, 1, Path::ALL), 4);
    // an opponent's Vorinclex halves: halve first, then Scales: 2 / 2 + 1 = 2 x 2 = 4 (not (2 + 1) / 2 x 2 = 2)
    perm(&mut g, P1, "Vorinclex, Monstrous Raider");
    assert_eq!(repl::counters(&g, P0, On::Perm(elves), Ctr::Plus, 2, Path::ALL), 4);
    // -1/-1 counters on your own creature: the fewest (Doubling Season after the halving: 3 / 2 x 2 = 2, not 3)
    assert_eq!(repl::counters(&g, P0, On::Perm(elves), Ctr::Plus, -3, Path::ALL), -2);
}

#[test]
fn a_person_orders_the_replacement_effects() {
    use sim_core::control::Answer;
    use sim_core::human::script::ScriptHuman;
    let mut g = game();
    let elves = perm(&mut g, P0, "Llanowar Elves");
    perm(&mut g, P0, "Hardened Scales");
    perm(&mut g, P0, "Doubling Season");
    // the choices come best first: "Hardened Scales, then Doubling Season: 4", "Doubling Season, then ...: 3"
    let h = std::sync::Arc::new(ScriptHuman::new(vec![Answer::Pick(1)]));
    g.humans.add(P0, h.clone());
    repl::put_plus(&mut g, elves, 1);
    assert_eq!(g.perm(elves).plus, 3, "the person picked doubling first");
    let asked = h.asked_kind("replacement_order");
    assert_eq!(asked.len(), 1);
    assert_eq!(
        asked[0].choices,
        ["Hardened Scales, then Doubling Season: 4", "Doubling Season, then Hardened Scales: 3"]
    );
}

#[test]
fn abilities_lost_stop_the_replacement_effects() {
    let mut g = game();
    let elves = perm(&mut g, P0, "Llanowar Elves");
    let theirs = perm(&mut g, P1, "Llanowar Elves");
    let fx = [
        perm(&mut g, P0, "Hardened Scales"),
        perm(&mut g, P0, "Doubling Season"),
        perm(&mut g, P0, "Vorinclex, Monstrous Raider"),
        perm(&mut g, P1, "Melira, Sylvok Outcast"),
    ];
    assert_eq!(repl::counters(&g, P0, On::Perm(elves), Ctr::Plus, 1, Path::ALL), 8);
    assert_eq!(repl::counters(&g, P0, On::Perm(theirs), Ctr::Plus, -1, Path::ALL), 0, "Melira");
    for m in fx {
        g.perm_mut(m).neutered = true; // Darksteel Mutation, Tishana's Tidebinder ...
    }
    assert_eq!(repl::counters(&g, P0, On::Perm(elves), Ctr::Plus, 1, Path::ALL), 1);
    assert_eq!(repl::counters(&g, P0, On::Perm(theirs), Ctr::Plus, -1, Path::ALL), -1);
    assert!(repl::poison_ok(&g, P1));
}

#[test]
fn melira_stops_wither_and_infect_counters_and_infect_poison() {
    let mut g = game();
    let mine = perm(&mut g, P1, "Llanowar Elves");
    perm(&mut g, P1, "Melira, Sylvok Outcast");
    // wither and infect put -1/-1 counters with `put_plus`, which used to ignore Melira
    repl::put_plus(&mut g, mine, -2);
    assert_eq!(g.perm(mine).plus, 0);
    // Blightsteel's infect damage to a player is poison counters: Melira stops them (it used to add them)
    let colossus = perm(&mut g, P0, "Blightsteel Colossus");
    sim_core::impls::karn_cards::infect_hit(&mut g, colossus, P1, 3).unwrap();
    assert_eq!(g.player(P1).poison, 0);
    // and Vorinclex doubles the ones its controller gives
    perm(&mut g, P0, "Vorinclex, Monstrous Raider");
    sim_core::impls::karn_cards::infect_hit(&mut g, colossus, P0, 3).unwrap(); // (its own controller: doubled)
    assert_eq!(g.player(P0).poison, 6);
}

#[test]
fn undying_and_entering_counters_get_doubling_season_and_scales() {
    let mut g = game();
    perm(&mut g, P0, "Doubling Season");
    perm(&mut g, P0, "Hardened Scales");
    let ghoul = perm(&mut g, P0, "Butcher Ghoul"); // undying
    zones::die(&mut g, ghoul, "destroy").unwrap();
    let back = g.player(P0).perms.iter().copied().find(|&m| name(&g, m) == Some("Butcher Ghoul")).unwrap();
    assert_eq!(g.perm(back).plus, 4, "undying's counter: (1 + Scales) x 2");
    let karn = perm(&mut g, P0, "Karn Liberated");
    assert_eq!(g.perm(karn).loyalty, Some(12), "once, through the compiled doubler");
}
