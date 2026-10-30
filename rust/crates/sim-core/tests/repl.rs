//! Phase 3, the replacement step (`engine::repl`): the step and its table give what the old helpers gave (their
//! bodies are copied here as `old_*`, reading names) on tricky boards: Doubling Season + Hardened Scales +
//! Vorinclex, Melira with infect and wither, Rest in Peace and a dying creature with a dies trigger, Solemnity with an
//! undying creature, damage prevention. Seat 0 plays the sauron deck, seat 1 the seph deck.

use sim_core::cardcode;
use sim_core::dsl;
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

/// the old `add_counters` amount (before `put_plus`'s Solemnity and Vorinclex)
fn old_add(g: &Game, m: PermId, n: i32) -> i32 {
    let x = g.perm(m);
    let orcish = x.army || (x.cd.is_some() && (values::has_type(g, m, "orc") || values::has_type(g, m, "goblin")));
    let mauhur = g.player(x.owner).perms.iter().any(|&y| name(g, y) == Some("Mauhúr, Uruk-hai Captain"));
    let n = if n > 0 && orcish && mauhur { n + 1 } else { n };
    if n > 0 && g.is_creature(m) { n + old_scales(g, x.owner) } else { n }
}

/// the old ability-language `counters` amount for actor p on m
fn old_dsl_counters(g: &Game, p: PlayerId, m: PermId, n: i32) -> i32 {
    let k = n * dsl::counter_mult(g, p);
    let extra = if k > 0 && g.is_creature(m) { old_scales(g, g.perm(m).owner) * dsl::counter_mult(g, p) } else { 0 };
    old_vorinclex(g, p, k + extra)
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
        assert_eq!(cardcode::scales(g, p), old_scales(g, p));
        assert_eq!(values::melira(g, p), old_melira(g, p));
        for n in [-5, -3, -1, 1, 2, 3, 7] {
            assert_eq!(cardcode::vorinclex(g, p, n), old_vorinclex(g, p, n), "vorinclex {p:?} {n}");
            for q in [P0, P1] {
                for &m in &g.player(q).perms {
                    let new = repl::counters(g, p, On::Perm(m), Ctr::Plus, n, Path::DSL);
                    let old = if old_solemnity(g) { 0 } else { old_dsl_counters(g, p, m, n) };
                    assert_eq!(new, old, "dsl counters {p:?} on {} {n}", g.perm(m).name);
                    let new = repl::counters(g, q, On::Perm(m), Ctr::Plus, n, Path::MAUHUR | Path::SCALES);
                    assert_eq!(new, if n > 0 { old_add(g, m, n) } else { n }, "add {} {n}", g.perm(m).name);
                }
            }
            let poison = repl::counters(g, p, On::Player(P1), Ctr::Poison, n.abs(), Path::POISON);
            assert_eq!(poison, if old_melira(g, P1) { 0 } else { old_vorinclex(g, p, n.abs()) });
        }
    }
    assert_eq!(repl::solemnity(g), old_solemnity(g));
    assert_eq!(zones::hushed(g), old_hushed(g));
}

// ------------------------------------------------------------------ the table
#[test]
fn the_table_is_read_once_per_card() {
    let g = game();
    for (n, fx) in [
        ("Hardened Scales", ReplFx::Scales),
        ("Vorinclex, Monstrous Raider", ReplFx::Vorinclex),
        ("Doubling Season", ReplFx::DoublingSeason),
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
    // worked by hand: seat 0's ability language puts 1 counter on its Elves: (1 + 2 Scales) x 2 Doubling Season, x 2
    // (its Vorinclex), / 2 (seat 1's)
    assert_eq!(repl::counters(&g, P0, On::Perm(elves), Ctr::Plus, 1, Path::DSL), 6);
    // seat 1 puts 3 on its own Elves: no Scales of its own, x 2 (its Vorinclex) after seat 0's halving: 3 / 2 * 2
    assert_eq!(repl::counters(&g, P1, On::Perm(theirs), Ctr::Plus, 3, Path::DSL), 2);
    // add_counters: Scales, then put_plus's Vorinclex by the owner (Doubling Season by name doesn't apply)
    let before = g.perm(elves).plus;
    cardcode::add_counters(&mut g, elves, 1);
    assert_eq!(g.perm(elves).plus - before, old_vorinclex(&g, P0, 1 + 2));
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
    assert!(values::melira(&g, P1) && !values::melira(&g, P0));
    assert!(!zones::minus_counter(&mut g, mine, 1), "no -1/-1 counter on Melira's side");
    assert_eq!(g.perm(mine).plus, 0);
    assert_eq!(repl::counters(&g, P0, On::Player(P1), Ctr::Poison, 3, Path::POISON), 0);
    assert_eq!(repl::counters(&g, P1, On::Player(P0), Ctr::Poison, 3, Path::POISON), 3);
    // a phased-out Melira stops nothing
    let melira = g.player(P1).perms.iter().copied().find(|&m| g.perm(m).name.contains("Melira")).unwrap();
    g.perm_mut(melira).phased = true;
    agree(&g);
    assert!(!values::melira(&g, P1));
}

#[test]
fn solemnity_and_an_undying_creature() {
    let mut g = game();
    perm(&mut g, P0, "Solemnity");
    agree(&g);
    let bear = perm(&mut g, P0, "Butcher Ghoul"); // undying
    cardcode::put_plus(&mut g, bear, 2);
    assert_eq!(g.perm(bear).plus, 0, "Solemnity: no counters");
    zones::die(&mut g, bear, "destroy").unwrap();
    // undying returns it with no counter (Solemnity), so it can return again
    let back = g.player(P0).perms.iter().copied().find(|&m| name(&g, m) == Some("Butcher Ghoul"));
    let back = back.expect("undying returned it");
    assert_eq!(g.perm(back).plus, 0);
    assert!(g.perm(back).undying);
    // loyalty counters still go on
    assert_eq!(repl::counters(&g, P0, On::Player(P1), Ctr::Loyalty, 2, Path::SOLEMN), 2);
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
    assert!(zones::hushed(&g));
    g.perm_mut(h).neutered = true;
    agree(&g);
    assert!(!zones::hushed(&g));
}

// ------------------------------------------------------------------ damage
#[test]
fn damage_prevention_cho_manno_and_tajic() {
    let mut g = game();
    let cho = perm(&mut g, P0, "Cho-Manno, Revolutionary");
    let elves = perm(&mut g, P0, "Llanowar Elves");
    assert!(values::no_damage(&g, cho) && !values::no_damage(&g, elves));
    g.perm_mut(cho).neutered = true;
    assert!(!values::no_damage(&g, cho), "abilities lost: no prevention");
    g.perm_mut(cho).neutered = false;
    let tajic = perm(&mut g, P0, "Tajic, Legion's Edge");
    assert!(cardcode::tajic_protects(&g, elves) && !cardcode::tajic_protects(&g, tajic));
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
    assert_eq!(dsl::token_mult(&g, P0), 4);
}
