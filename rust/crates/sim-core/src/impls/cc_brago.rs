//! Casual combos (brago): the unbounded-mana lines of `brago-azorius-blink-control` (Tier 2), which win through Agent
//! of Treachery blinked again and again until it has taken every permanent.
//!
//! - Peregrine Drake + Archaeomancer + Ghostly Flicker (its `Combo` entry stays in impls/t12_brago_meren.rs, which
//!   now uses `flicker_ready` and `FLICKER_STEPS` from here): Flicker blinks Drake and Archaeomancer, five lands
//!   untap and Flicker comes back to hand, two mana up each time (four or more lands: one up).
//! - Displacer Kitten + Archaeomancer + a mana enters creature + a flicker spell (new, `COMBOS`): cast Cloudshift
//!   ({W}) or Momentary Blink ({1}{W}) on Archaeomancer, Kitten's trigger blinks Peregrine Drake (five lands untap)
//!   or Cloud of Faeries (two), Archaeomancer returns the spell; Ghostly Flicker works as well (it blinks the mana
//!   creature with Archaeomancer, and Kitten blinks it again). Ephemerate is left out: cast from hand it is exiled
//!   (rebound), so Archaeomancer can't return it; Eerie Interlude returns its creatures only at the end step.
//!
//! Agent of Treachery is the finish: on the battlefield, in hand (cast with the loop's mana: nothing is paid for it
//! up front), or still in the library when a creature or artifact that draws as it enters (Mulldrifter, Cloudblazer,
//! Wall of Omens, Cryogen Relic) is on the battlefield: blinked with the loop, it draws the library down to the
//! Agent. The line has three answer windows: the flicker spell that starts it (counters), the Agent's cast (counters,
//! when it is cast), and each opponent's one try at instant-speed removal on a piece.

use crate::engine::cast::{cast_card, castable};
use crate::engine::mana::{can_pay, cost_of, pay};
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::impls::combos::{Combo, Ready, Step, StepKind, no_src};
use crate::state::{Ctx, Game};

const DECK: &str = "brago-azorius-blink-control";
const AGENT: &str = "Agent of Treachery";
const ARCH: &str = "Archaeomancer";
const DRAKE: &str = "Peregrine Drake";
const FAERIES: &str = "Cloud of Faeries";
const KITTEN: &str = "Displacer Kitten";
const FLICKER: &str = "Ghostly Flicker";
const CLOUDSHIFT: &str = "Cloudshift";
const BLINK: &str = "Momentary Blink";
/// what draws as it enters, blinked with the loop to dig for Agent of Treachery
const DRAWERS: &[&str] = &["Mulldrifter", "Cloudblazer", "Wall of Omens", "Cryogen Relic"];

// ------------------------------------------------------------------ helpers
fn named(g: &Game, c: CardId, name: &str) -> bool {
    *g.db.get(c).name == *name
}

/// p's live permanent of this name (its own: a stolen one would go home when blinked)
fn mine(g: &Game, p: PlayerId, name: &str) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| {
        let x = g.perm(m);
        x.on_bf && !x.phased && x.orig == p && x.cd.is_some_and(|c| named(g, c, name))
    })
}

fn held(g: &Game, p: PlayerId, name: &str) -> Option<CardId> {
    g.player(p).hand.iter().copied().find(|&c| named(g, c, name))
}

/// a spell in hand that p can cast now
fn castable_held(g: &Game, p: PlayerId, name: &str) -> Option<CardId> {
    held(g, p, name).filter(|&c| castable(g, p, c, "hand"))
}

fn lands(g: &Game, p: PlayerId) -> u32 {
    g.player(p).lands.len() as u32
}

/// how p gets Agent of Treachery: the permanent, the card in hand, or the library (dug for with a drawer)
enum Finish {
    Board(PermId),
    Hand(CardId),
    Library(CardId, PermId),
}

fn finish(g: &Game, p: PlayerId) -> Option<Finish> {
    if let Some(a) = mine(g, p, AGENT) {
        return Some(Finish::Board(a));
    }
    if let Some(c) = held(g, p, AGENT) {
        return castable(g, p, c, "hand").then_some(Finish::Hand(c));
    }
    let c = g.player(p).library.iter().copied().find(|&c| named(g, c, AGENT))?;
    let d = DRAWERS.iter().find_map(|n| mine(g, p, n))?;
    Some(Finish::Library(c, d))
}

fn finish_keys(f: &Finish) -> Vec<PermId> {
    match *f {
        Finish::Board(a) => vec![a],
        Finish::Hand(_) => vec![],
        Finish::Library(_, d) => vec![d],
    }
}

/// the spell that starts the line: paid for and cast as any spell (it can be countered); true if it resolved
fn cast_start(g: &mut Game, p: PlayerId, c: CardId) -> Res<bool> {
    let (gn, pips) = cost_of(g, p, c);
    if !g.player(p).hand.contains(&c) || !castable(g, p, c, "hand") || !can_pay(g, p, gn, &pips, false) {
        return Ok(false);
    }
    g.pay_for = Some(c);
    let ok = pay(g, p, gn, &pips, false)?;
    g.pay_for = None;
    if !ok {
        return Ok(false);
    }
    g.combo_spell = true;
    let r = cast_card(g, p, c, "hand", Ctx::default());
    g.combo_spell = false;
    r
}

/// the loop's mana pays for Agent of Treachery (dug out of the library first if need be); its cast can be countered
fn get_agent(g: &mut Game, p: PlayerId) -> Res<bool> {
    let c = match finish(g, p) {
        None => return Ok(false),
        Some(Finish::Board(_)) => return Ok(true),
        Some(Finish::Hand(c)) => c,
        Some(Finish::Library(c, d)) => {
            let lib = &mut g.player_mut(p).library;
            let Some(i) = lib.iter().position(|&x| x == c) else { return Ok(false) };
            lib.remove(i);
            g.player_mut(p).hand.push(c);
            crate::glog!(
                g,
                "    {} blinks {} again and again: draws down to {AGENT}",
                g.player(p).name,
                g.perm(d).name
            );
            c
        }
    };
    if !castable(g, p, c, "hand") {
        return Ok(false);
    }
    g.combo_spell = true;
    let r = cast_card(g, p, c, "hand", Ctx::default());
    g.combo_spell = false;
    Ok(r? && mine(g, p, AGENT).is_some())
}

const AGENT_STEP: Step = Step {
    label: "cast Agent of Treachery with the loop's mana",
    kind: StepKind::Engine,
    src: no_src,
    pay: (0, ""),
    run: get_agent,
    keys: crate::impls::combos::no_keys,
};

// ------------------------------------------------------------------ Peregrine Drake + Archaeomancer + Ghostly Flicker
/// Drake, Archaeomancer and four or more lands (Flicker costs three: four lands untapped is one mana up each time),
/// Ghostly Flicker castable in hand, and a way to Agent of Treachery
pub fn flicker_ready(g: &Game, p: PlayerId) -> Ready {
    let (Some(d), Some(a)) = (mine(g, p, DRAKE), mine(g, p, ARCH)) else { return (false, vec![], vec![]) };
    let Some(f) = castable_held(g, p, FLICKER) else { return (false, vec![], vec![]) };
    if lands(g, p) < 4 {
        return (false, vec![], vec![]);
    }
    let Some(fin) = finish(g, p) else { return (false, vec![], vec![]) };
    let mut keys = finish_keys(&fin);
    keys.extend([d, a]);
    (true, keys, vec![f])
}

fn flicker_cast(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some(f) = held(g, p, FLICKER) else { return Ok(false) };
    cast_start(g, p, f)
}

fn flicker_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    [DRAKE, ARCH, AGENT].iter().filter_map(|n| mine(g, p, n)).collect()
}

pub const FLICKER_STEPS: &[Step] = &[
    Step {
        label: "Ghostly Flicker on Peregrine Drake and Archaeomancer",
        kind: StepKind::Spell,
        src: no_src,
        pay: (0, ""),
        run: flicker_cast,
        keys: crate::impls::combos::no_keys,
    },
    AGENT_STEP,
    Step {
        label: "the loop: blink Agent of Treachery until it has every permanent",
        kind: StepKind::Permanent,
        src: no_src,
        pay: (0, ""),
        run: crate::impls::combos::done,
        keys: flicker_keys,
    },
];

// ------------------------------------------------------------------ Displacer Kitten + Archaeomancer
/// the mana creature Kitten blinks and the flicker spell to loop, best first: Peregrine Drake with any of the three
/// spells (a land up at two lands with Cloudshift), Cloud of Faeries with Cloudshift (one up). Kitten's trigger
/// resolves first and blinks the mana creature, so a Ghostly Flicker aimed at it too finds a new object: it untaps once
/// per loop like the others (Cloud of Faeries with Flicker loses one)
fn kitten_pick(g: &Game, p: PlayerId) -> Option<(PermId, CardId)> {
    let n = lands(g, p).min(5);
    let mut best: Option<(i32, PermId, CardId)> = None;
    for (mana_name, untap) in [(DRAKE, n), (FAERIES, n.min(2))] {
        let Some(m) = mine(g, p, mana_name) else { continue };
        for spell in [CLOUDSHIFT, BLINK, FLICKER] {
            let Some(c) = castable_held(g, p, spell) else { continue };
            let (gn, pips) = cost_of(g, p, c);
            let cost = (gn as usize + pips.len()) as i32;
            let net = untap as i32 - cost;
            if net > 0 && best.is_none_or(|b| net > b.0) {
                best = Some((net, m, c));
            }
        }
    }
    best.map(|b| (b.1, b.2))
}

fn kitten_ready(g: &Game, p: PlayerId) -> Ready {
    let (Some(k), Some(a)) = (mine(g, p, KITTEN), mine(g, p, ARCH)) else { return (false, vec![], vec![]) };
    let Some((m, c)) = kitten_pick(g, p) else { return (false, vec![], vec![]) };
    let Some(fin) = finish(g, p) else { return (false, vec![], vec![]) };
    let mut keys = finish_keys(&fin);
    keys.extend([k, a, m]);
    (true, keys, vec![c])
}

fn kitten_cast(g: &mut Game, p: PlayerId) -> Res<bool> {
    let Some((_, c)) = kitten_pick(g, p) else { return Ok(false) };
    cast_start(g, p, c)
}

fn kitten_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    [KITTEN, ARCH, DRAKE, FAERIES, AGENT].iter().filter_map(|n| mine(g, p, n)).collect()
}

const KITTEN_STEPS: &[Step] = &[
    Step {
        label: "a flicker spell on Archaeomancer (Displacer Kitten blinks the mana creature)",
        kind: StepKind::Spell,
        src: no_src,
        pay: (0, ""),
        run: kitten_cast,
        keys: crate::impls::combos::no_keys,
    },
    AGENT_STEP,
    Step {
        label: "the loop: Kitten blinks Agent of Treachery until it has every permanent",
        kind: StepKind::Permanent,
        src: no_src,
        pay: (0, ""),
        run: crate::impls::combos::done,
        keys: kitten_keys,
    },
];

pub static COMBOS: [Combo; 1] = [Combo {
    name: "Displacer Kitten + Archaeomancer + Peregrine Drake or Cloud of Faeries + a flicker spell (+ Agent of Treachery)",
    groups: &[&[KITTEN], &[ARCH], &[DRAKE, FAERIES], &[CLOUDSHIFT, BLINK, FLICKER], &[AGENT]],
    ready: kitten_ready,
    mana: (0, ""),
    finish: None,
    xcost: &[],
    steps: KITTEN_STEPS,
}];

// ------------------------------------------------------------------ Cloudshift
/// in hand or p's own on the battlefield
fn around(g: &Game, p: PlayerId, name: &str) -> bool {
    held(g, p, name).is_some() || mine(g, p, name).is_some()
}

/// Cloudshift ({W}, exile target creature you control, then return it): a plain spell only on a creature whose enter
/// effect is worth repeating (it needs a target: it used to be cast on an empty board), and kept for the loop while
/// Displacer Kitten and Archaeomancer are both in hand or on the battlefield. (the brago deck only:
/// `hold`, read by ai/brain.rs)
fn cloudshift_hold(g: &Game, p: PlayerId) -> bool {
    if around(g, p, KITTEN) && around(g, p, ARCH) {
        return true;
    }
    let best = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| g.perm(m).on_bf && !g.perm(m).phased && g.perm(m).orig == p && g.is_creature(m))
        .map(|m| crate::impls::mine::blink_worth(g, p, m))
        .fold(0.0, f64::max);
    best < 3.0
}

/// ai/brain.rs card_utility: the brago deck keeps c in hand for now
pub fn hold(g: &Game, p: PlayerId, c: CardId) -> bool {
    g.player(p).key == DECK && named(g, c, CLOUDSHIFT) && cloudshift_hold(g, p)
}
