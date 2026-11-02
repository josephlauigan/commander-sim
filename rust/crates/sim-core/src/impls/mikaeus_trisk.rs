//! The combo-catalogue entry "Mikaeus + Triskelion" (one of Sephiroth's old loops, impls/mine.rs `LOOPS[0]`, read
//! through `mine::loops_now` for any deck): Mikaeus, the Unhallowed returns every creature with a +1/+1 counter and
//! Triskelion pings its counters off, so it kills alone. It ran in the retired `ghave` deck's catalogue slice; Sheoldred
//! holds both pieces now, so the entry stays. The rest of that slice (Ghave's altar loops, the Finks and Nim lines) held
//! pieces no deck in the simulator plays and went with the key (2026-11-02).

use crate::ids::{PermId, PlayerId};
use crate::impls::combos::{Combo, Ready, loop_payoff};
use crate::state::Game;

const MIKAEUS: &[&str] = &["Mikaeus, the Unhallowed"];
const TRISKELION: &[&str] = &["Triskelion"];
const SEPHIROTH: &str = "Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel";

const NOT_READY: fn() -> Ready = || (false, vec![], vec![]);

fn on_bf(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        x.on_bf
            && !x.phased
            && !x.neutered
            && x.cd.is_some_and(|c| &*g.db.get(c).name == name)
            && !crate::engine::values::stopped(g, &g.db.get(x.card()).name)
    })
}

/// a death payoff out (the catalogue's drain payoffs, or Sephiroth, Fabled SOLDIER's drain)
fn payoff(g: &Game, p: PlayerId) -> Option<PermId> {
    loop_payoff(g, p).or_else(|| on_bf(g, p, SEPHIROTH))
}

/// Sephiroth's loop 0 (impls/mine.rs `LOOPS[0]`) when it kills: alone, or with a death payoff out
fn mikaeus_trisk(g: &Game, p: PlayerId) -> Ready {
    let Some((_, keys, kills)) = super::mine::loops_now(g, p).into_iter().find(|x| x.0 == 0) else {
        return NOT_READY();
    };
    if !(kills || payoff(g, p).is_some()) {
        return NOT_READY();
    }
    (true, keys, vec![])
}

pub static COMBOS: [Combo; 1] = [Combo {
    name: "Mikaeus + Triskelion",
    groups: &[MIKAEUS, TRISKELION],
    ready: mikaeus_trisk,
    mana: (0, ""),
    finish: None,
    xcost: &[],
    steps: &[],
}];
