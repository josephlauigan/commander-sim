//! The Scion line for JD's `scion` deck (rust/agents/scion-goldfish.md, Part 3): Prismari's storm with Underworld
//! Breach. Framework: `combos::Combo` / `combos::Step` (each step answerable as the real play is).
//!
//! **Scion as Prismari + Underworld Breach + Manamorphose + Brain Freeze (at the table).** Scion's {2} puts Prismari,
//! the Inspiration into the graveyard and Scion becomes a copy of it (skipped when Prismari itself is out): every
//! instant and sorcery cast this turn has storm. Cast Underworld Breach ({1}{R}): each nonland card in the graveyard
//! has escape (its mana cost plus exiling three other cards from the graveyard). Cast Manamorphose ({1}{R/G}: two mana
//! of any colours, draw a card): storm copies it once per spell before it, so it makes 2 x (storm + 1) mana for 2.
//! Cast Brain Freeze ({1}{U}) at yourself: its own storm and Prismari's give 2 x storm + 1 copies of "mill three",
//! the fuel for the escapes. Then escape Manamorphose and Brain Freeze again and again: each round costs 4 and makes
//! 2 x (storm + 1) of any colour, the storm count grows by two a round, and the Brain Freeze copies mill yourself for
//! fuel and then every opponent's library empty. Mana up front: {2} (Scion), {1}{R} (Breach, unless out), {1}{R/G}
//! (Manamorphose); its own mana pays for Brain Freeze. A Dark Ritual, Cabal Ritual or Rite of Flame in the graveyard
//! is escaped for more mana when Manamorphose's draws would empty the library; they make no blue, so they can't run
//! the loop alone (Brain Freeze needs {U} every round), which is why Manamorphose is the piece.
//!
//! Not catalogued (they make mana or cards, or the cards don't support a sure kill): Scion as Ancient Copper Dragon (a
//! d20 of Treasures on combat damage), Scion as Korvold (a card per sacrifice), Prismari + Brain Freeze without Breach
//! (3 x (2 x storm + 1) cards: a self-mill for Thassa's Oracle needs a storm count near 12 on turn 3), Flash
//! Photography or Mockingbird copying Scion as Prismari (a lasting storm engine), and Prismari with Breach and Dark
//! Ritual alone (no blue for the next Brain Freeze).

use super::combos::{Combo, QUIET, Ready, Step, StepKind, no_keys, no_src};
use super::t6_combos_godo::{bf, in_hand, in_lib, ok_ready};
use super::t6_scion::{PRISMARI, SCION, activate, prismari_on, scion_of};
use crate::engine::cast::{cast_card, castable};
use crate::engine::mana::{can_pay, cost_of, pay};
use crate::flow::Res;
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game};

const BREACH: &str = "Underworld Breach";
const MANAMORPHOSE: &str = "Manamorphose";
const BRAIN_FREEZE: &str = "Brain Freeze";
const RITUALS: [&str; 3] = ["Dark Ritual", "Cabal Ritual", "Rite of Flame"];
const LINE: [&str; 6] = [BREACH, MANAMORPHOSE, BRAIN_FREEZE, "Dark Ritual", "Cabal Ritual", "Rite of Flame"];

/// fuel the graveyard keeps for the next round (two escapes at most, three cards each, and a margin)
const FUEL: usize = 9;

fn no() -> Ready {
    (false, vec![], vec![])
}

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

fn in_gy(g: &Game, p: PlayerId, name: &str) -> Option<CardId> {
    g.player(p).gy.iter().copied().find(|&c| cname(g, c) == name)
}

/// graveyard cards an escape may exile (not the line's own cards)
fn fuel(g: &Game, p: PlayerId) -> usize {
    g.player(p).gy.iter().filter(|&&c| !LINE.contains(&cname(g, c))).count()
}

/// spells cast this turn by everyone: the storm count of the next spell
fn storm_now(g: &Game) -> i32 {
    g.players.iter().map(|q| crate::engine::cast::casts_this_turn(g, q.id)).sum()
}

/// the copies of "mill three" the next Brain Freeze makes: its own storm, and Prismari's
fn freeze_copies(g: &Game, p: PlayerId) -> usize {
    let s = storm_now(g) as usize;
    s + 1 + if prismari_on(g, p) { s } else { 0 }
}

/// cast a card of this name from hand, paying for it (the normal counter window)
fn cast_from_hand(g: &mut Game, p: PlayerId, name: &str, ctx: Ctx) -> Res<bool> {
    let Some(c) = in_hand(g, p, name) else { return Ok(false) };
    if !castable(g, p, c, "hand") {
        return Ok(false);
    }
    let (gn, pips) = cost_of(g, p, c);
    if !can_pay(g, p, gn, &pips, false) {
        return Ok(false);
    }
    pay(g, p, gn, &pips, false)?;
    cast_card(g, p, c, "hand", ctx)
}

/// escape a card of this name with Underworld Breach: its mana cost, and three other graveyard cards exiled (the
/// line's own cards last); then the real cast
fn escape(g: &mut Game, p: PlayerId, name: &str, ctx: Ctx) -> Res<bool> {
    let Some(c) = in_gy(g, p, name) else { return Ok(false) };
    if bf(g, p, BREACH).is_none() || !castable(g, p, c, "gy") {
        return Ok(false);
    }
    let (gn, pips) = cost_of(g, p, c);
    let mut others: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&x| x != c).collect();
    others.sort_by_key(|&x| LINE.contains(&cname(g, x)));
    if others.len() < 3 || !can_pay(g, p, gn, &pips, false) {
        return Ok(false);
    }
    pay(g, p, gn, &pips, false)?;
    for &x in others.iter().take(3) {
        let pl = g.player_mut(p);
        if let Some(i) = pl.gy.iter().position(|&y| y == x) {
            pl.gy.remove(i);
            pl.exile.push(x);
        }
    }
    crate::glog!(g, "  {} escapes {} with Underworld Breach", g.player(p).name, name);
    cast_card(g, p, c, "escape", ctx)
}

/// from hand if it is there, else escaped from the graveyard
fn hand_or_escape(g: &mut Game, p: PlayerId, name: &str, ctx: Ctx) -> Res<bool> {
    if in_hand(g, p, name).is_some() { cast_from_hand(g, p, name, ctx) } else { escape(g, p, name, ctx) }
}

/// whom each Brain Freeze copy mills: yourself until the graveyard holds the fuel for the next round (keeping cards
/// to draw), then each opponent until its library is empty
fn freeze_targets(g: &Game, p: PlayerId) -> Vec<PlayerId> {
    let n = freeze_copies(g, p);
    // an escaped Brain Freeze exiles three of the fuel before it resolves
    let escaping = if in_hand(g, p, BRAIN_FREEZE).is_some() { 0 } else { 3 };
    let need = (FUEL + escaping).saturating_sub(fuel(g, p)).div_ceil(3);
    let spare = g.player(p).library.len().saturating_sub(2 * n + 4) / 3; // Manamorphose's draws come out of it
    let mut whom: Vec<PlayerId> = std::iter::repeat_n(p, need.min(spare)).collect();
    for q in g.opps(p).collect::<Vec<_>>() {
        whom.extend(std::iter::repeat_n(q, g.player(q).library.len().div_ceil(3)));
    }
    whom.truncate(n); // copies past the list mill the opponent with the fewest cards (Brain Freeze's default)
    whom
}

fn freeze(g: &mut Game, p: PlayerId) -> Res<bool> {
    let whom = freeze_targets(g, p);
    hand_or_escape(g, p, BRAIN_FREEZE, Ctx { mill: whom, ..Ctx::default() })
}

/// a mana spell for the next round: Manamorphose while the library can take its draws, else a ritual
fn mana_round(g: &mut Game, p: PlayerId) -> Res<bool> {
    let draws = freeze_copies(g, p); // Manamorphose's copies: as many as Brain Freeze's
    if g.player(p).library.len() > draws + 2 && escape(g, p, MANAMORPHOSE, Ctx::default())? {
        return Ok(true);
    }
    for r in RITUALS {
        if escape(g, p, r, Ctx::default())? {
            return Ok(true);
        }
    }
    Ok(false)
}

// ------------------------------------------------------------------ the steps
fn become_prismari(g: &mut Game, p: PlayerId) -> Res<bool> {
    if prismari_on(g, p) {
        return Ok(true);
    }
    let Some(s) = scion_of(g, p) else { return Ok(false) };
    Ok(activate(g, s, p, &[PRISMARI])? && prismari_on(g, p))
}

fn cast_breach(g: &mut Game, p: PlayerId) -> Res<bool> {
    if bf(g, p, BREACH).is_some() {
        return Ok(true);
    }
    Ok(cast_from_hand(g, p, BREACH, Ctx::default())? && bf(g, p, BREACH).is_some())
}

fn breach_keys(g: &Game, p: PlayerId) -> Vec<PermId> {
    bf(g, p, BREACH).into_iter().collect()
}

fn first_manamorphose(g: &mut Game, p: PlayerId) -> Res<bool> {
    hand_or_escape(g, p, MANAMORPHOSE, Ctx::default())
}

/// escape Manamorphose (or a ritual) and Brain Freeze, round after round, until every opponent's library is empty
fn table_loop(g: &mut Game, p: PlayerId) -> Res<bool> {
    for _ in 0..40 {
        if g.opps(p).all(|q| g.player(q).library.is_empty()) {
            return Ok(true);
        }
        if g.over || !g.player(p).alive || !prismari_on(g, p) {
            return Ok(g.over);
        }
        if !can_pay(g, p, 6, "", false) {
            mana_round(g, p)?; // more mana first, only when it runs low: its draws and escapes eat the library and fuel
        }
        if !freeze(g, p)? {
            let libs: Vec<usize> = g.opps(p).map(|q| g.player(q).library.len()).collect();
            crate::glog!(
                g,
                "    the loop runs out: library {}, fuel {}, mana {}, opponents' libraries {libs:?}",
                g.player(p).library.len(),
                fuel(g, p),
                crate::engine::mana::total_mana(g, p, false)
            );
            return Ok(false);
        }
    }
    Ok(g.opps(p).all(|q| g.player(q).library.is_empty()))
}

/// the opponents' libraries are empty: they lose at their next draw
fn table_finish(g: &mut Game, p: PlayerId) -> Res {
    crate::ai::win(g, p, "combo", Some(false))
}

const fn step(label: &'static str, kind: StepKind, run: fn(&mut Game, PlayerId) -> Res<bool>) -> Step {
    Step { label, kind, src: no_src, pay: (0, ""), run, keys: no_keys }
}

const PRISMARI_STEPS: [Step; 6] = [
    step("Scion's {2}: Prismari into the graveyard, Scion becomes a copy of it", StepKind::Engine, become_prismari),
    step("cast Underworld Breach", StepKind::Spell, cast_breach),
    Step {
        label: "Underworld Breach stays in play",
        kind: StepKind::Permanent,
        src: no_src,
        pay: (0, ""),
        run: super::combos::done,
        keys: breach_keys,
    },
    step("cast Manamorphose (storm copies make the mana)", StepKind::Spell, first_manamorphose),
    step("Brain Freeze with two storm triggers, at yourself for fuel", StepKind::Spell, freeze),
    step("escape Manamorphose and Brain Freeze until every opponent's library is empty", StepKind::Spell, table_loop),
];

/// ready: a Prismari (or Scion with Prismari in the library), Breach out or castable, Manamorphose and Brain Freeze
/// in hand or graveyard (with the fuel to escape the ones in the graveyard), and the mana up front
fn ready_prismari(g: &Game, p: PlayerId) -> Ready {
    let act = !prismari_on(g, p);
    if act && (scion_of(g, p).is_none() || !in_lib(g, p, PRISMARI)) {
        return no();
    }
    let breach_out = bf(g, p, BREACH).is_some();
    let breach_hand = in_hand(g, p, BREACH).filter(|&c| castable(g, p, c, "hand"));
    if !breach_out && breach_hand.is_none() {
        return no();
    }
    let mm_hand = in_hand(g, p, MANAMORPHOSE);
    let (mm, bfz) = (mm_hand.or(in_gy(g, p, MANAMORPHOSE)), in_hand(g, p, BRAIN_FREEZE).or(in_gy(g, p, BRAIN_FREEZE)));
    let (Some(mm), Some(_)) = (mm, bfz) else { return no() };
    let from_gy = usize::from(mm_hand.is_none()) + usize::from(in_hand(g, p, BRAIN_FREEZE).is_none());
    if fuel(g, p) < 3 * from_gy || g.player(p).library.len() < 12 {
        return no();
    }
    let (mg, mp) = cost_of(g, p, mm);
    let mut gn = mg + if act { 2 } else { 0 };
    let mut pips = mp;
    if let Some(b) = breach_hand.filter(|_| !breach_out) {
        let (bg, bp) = cost_of(g, p, b);
        gn += bg;
        pips.push_str(&bp);
    }
    if !can_pay(g, p, gn, &pips, false) {
        return no();
    }
    ok_ready()
}

/// the combos of this module (chained after the others by `combos::all`)
pub static COMBOS: [Combo; 1] = [Combo {
    name: "Scion as Prismari + Underworld Breach + Manamorphose + Brain Freeze (at the table)",
    groups: &[QUIET, &[SCION, PRISMARI], &[BREACH], &[MANAMORPHOSE], &[BRAIN_FREEZE]],
    ready: ready_prismari,
    mana: (0, ""),
    finish: Some(table_finish),
    xcost: &[],
    steps: &PRISMARI_STEPS,
}];
