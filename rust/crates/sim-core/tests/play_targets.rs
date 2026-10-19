//! Practice mode: the person chooses the targets, modes and costs of spells that resolve by their own code
//! (src/human/specs.rs). Each test casts through the person's cast flow with scripted answers and checks both what
//! was asked and that the answer was applied.

mod play_common;
use play_common::*;
use sim_core::human::human::human_main;
use std::sync::Arc;

const P3: PlayerId = PlayerId(3);

/// pick the first choice that contains `word`
fn pick_text(word: String) -> F {
    Box::new(move |r| {
        if r.choices.is_empty() {
            return Answer::Action(json!({"do": "pass"})); // the cast was refused: the test then shows why
        }
        let i = r
            .choices
            .iter()
            .position(|x| x.contains(&word))
            .unwrap_or_else(|| panic!("no choice with {word:?} in {:?} ({})", r.choices, r.prompt));
        Answer::Pick(i)
    })
}

fn cast_first() -> F {
    act(json!({"do": "cast", "card": 0}))
}

fn has_asked(h: &ScriptHuman, start: &str) -> bool {
    h.asked().iter().any(|r| r.prompt.starts_with(start))
}

#[test]
fn lethal_throwdown_asks_for_the_target_and_the_sacrifice() {
    let mut g = table(&["sheoldred", "veyran"]);
    hand(&mut g, P0, &["Lethal Throwdown"]);
    pool(&mut g, P0, &[('B', 2)]);
    let _keep = token(&mut g, P0, 1);
    let fodder = token(&mut g, P0, 3);
    let victim = token(&mut g, P1, 2);
    let h = seat(&mut g, P0, vec![cast_first(), pick_text("2/".into()), pick_text("you, 3/".into()), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(has_asked(&h, "Lethal Throwdown: choose a target"), "asked for the target");
    assert!(has_asked(&h, "Lethal Throwdown: sacrifice which creature"), "asked for the sacrifice");
    assert!(!g.perm(victim).on_bf, "the target was destroyed");
    assert!(!g.perm(fodder).on_bf, "the chosen creature was sacrificed");
    assert_eq!(g.player(P0).perms.iter().filter(|&&m| g.perm(m).on_bf).count(), 1, "the other one stayed");
    assert!(g.player(P0).gy.contains(&card(&g, "Lethal Throwdown")));
}

#[test]
fn splatter_technique_can_draw_four() {
    let mut g = table(&["kefka", "veyran"]);
    hand(&mut g, P0, &["Splatter Technique"]);
    pool(&mut g, P0, &[('U', 2), ('R', 2), ('C', 1)]);
    let mine = token(&mut g, P0, 1);
    let h = seat(&mut g, P0, vec![cast_first(), pick_text("draw four".into()), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(has_asked(&h, "Splatter Technique: choose a mode"));
    assert_eq!(g.player(P0).hand.len(), 4, "four cards drawn, and the spell is gone");
    assert!(g.perm(mine).on_bf, "the wipe mode was not used");
}

#[test]
fn comet_storm_takes_a_target_for_each_kick() {
    let mut g = table(&["sauron", "veyran", "seph", "kefka"]);
    hand(&mut g, P0, &["Comet Storm"]);
    pool(&mut g, P0, &[('R', 2), ('C', 3)]);
    for p in [P1, P2, P3] {
        g.player_mut(p).life = 40;
    }
    let (n1, n2) = (g.player(P1).name.to_string(), g.player(P2).name.to_string());
    let h = seat(
        &mut g,
        P0,
        vec![
            cast_first(),
            pick_text("kick once".into()),
            pick_text("X = 2".into()),
            pick_text(format!("{n1} (player")),
            pick_text(format!("{n2} (player")),
            pass(),
        ],
    );
    human_main(&mut g, P0, false).unwrap();
    assert!(has_asked(&h, "Comet Storm: choose target 2 of 2"));
    assert_eq!(g.player(P1).life, 38);
    assert_eq!(g.player(P2).life, 38);
    assert_eq!(g.player(P3).life, 40, "not chosen");
}

#[test]
fn peer_into_the_abyss_can_target_an_opponent() {
    let mut g = table(&["sheoldred", "veyran"]);
    hand(&mut g, P0, &["Peer into the Abyss"]);
    pool(&mut g, P0, &[('B', 3), ('C', 4)]);
    let n1 = g.player(P1).name.to_string();
    let (lib1, life1, hand0) = (g.player(P1).library.len(), g.player(P1).life, g.player(P0).hand.len());
    let h = seat(&mut g, P0, vec![cast_first(), pick_text(format!("{n1} (player")), pass()]);
    human_main(&mut g, P0, false).unwrap();
    assert!(has_asked(&h, "Peer into the Abyss: choose a target (player)"));
    assert_eq!(g.player(P1).hand.len(), lib1.div_ceil(2), "the opponent drew half their library");
    assert_eq!(g.player(P1).life, life1 - (life1 + 1) / 2, "and lost half their life");
    assert_eq!(g.player(P0).hand.len(), hand0 - 1, "the caster drew nothing (only the spell left their hand)");
}

#[test]
fn inferno_titan_divides_its_damage_point_by_point() {
    let mut g = table(&["sheoldred", "veyran"]);
    let a = token(&mut g, P1, 1);
    let b = token(&mut g, P1, 1);
    let n1 = g.player(P1).name.to_string();
    let life1 = g.player(P1).life;
    let mut seen = 0;
    let h = Arc::new(ScriptHuman::from_fn(move |r: &Request| {
        if !r.prompt.contains("of 3 of the damage") {
            return Answer::Yes; // the trigger's own "do it?" window
        }
        seen += 1;
        let tokens: Vec<usize> =
            r.choices.iter().enumerate().filter(|(_, c)| c.contains(&format!("({n1}, 1/"))).map(|(i, _)| i).collect();
        let face = r.choices.iter().position(|c| c.contains("(player") && c.starts_with(&n1)).unwrap();
        Answer::Pick(match seen {
            1 => tokens[0],
            2 => tokens[1],
            _ => face,
        })
    }));
    g.humans.add(P0, h.clone());
    perm(&mut g, P0, "Inferno Titan");
    let asked = h.asked();
    assert_eq!(asked.iter().filter(|r| r.prompt.contains("of 3 of the damage")).count(), 3, "one question per point");
    assert!(!g.perm(a).on_bf && !g.perm(b).on_bf, "one point each killed the two tokens");
    assert_eq!(g.player(P1).life, life1 - 1, "and the last point went to the player");
}
