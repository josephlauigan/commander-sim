//! Karn, Gilded Guardian ({2/W}{2/U}{2/B}{2/R}{2/G}, Legendary Artifact Creature - Golem 5/5, vigilance, trample). Oracle
//! (data/scryfall_cache.json): "When Karn enters, draw a card for each color among other artifacts you control." Not
//! Karn, Legacy Reforged (karn.rs, karn_cards.rs, karn_engines.rs).
//!
//! The body and the cost are the card's data ({2/X} symbols: `engine::hybrid`; the override block in
//! data/card_overrides.toml carries no cost, so `cardinfo::hybrid` keeps them). The enter trigger is card code:
//!
//! - **A triggered ability that uses the stack** (a hook not marked at once), so an opponent's Elesh Norn blanks it, Gandalf
//!   the White and your own Elesh Norn add triggers (each instance counts at its own resolution), and the table may
//!   respond (a person is asked nothing: there is no choice to make). The model is `impls/dockside.rs`.
//! - **The count is taken at resolution** (CR 608.2h): a response that removes or adds an artifact changes it. It is the
//!   number of colours in the union of the colours of the artifacts the controller has then (`chars::colors`, so a
//!   Transguild Courier is five, Leyline of the Guildpact's owner's artifacts are five, a copy keeps the copied card's
//!   colours, a token its token colour), not the number of artifacts: two red artifacts are one colour. Karn himself is
//!   not counted ("other"), nor are phased-out permanents or face-down ones (a 2/2 with no types, CR 708.2). The
//!   Treasure, Food, Clue and Blood tokens are counters on the player in this engine and are colourless anyway. A count of
//!   0 draws nothing. A Cursed Mirror that copies Karn is a Karn too: it counts the original (it is another
//!   artifact), and the legend rule keeps one of them.
//!
//! AI (PROVISIONAL, never tuned by a sim; no plan row is added): the cast priority below, read by the generic cast logic.
//! A 5/5 vigilance trample body is worth casting with no draw, and each colour among the controller's artifacts adds to it.
//!
//! Blinking him (E4): `reenter_bonus` is what re-entering is worth to the AI, read at the top of every blink chooser
//! (`t2::blink_value`, `common::blink_worth`, `mine::blink_worth`, the Cloudshift DSL blink, Flickerwisp's `wisp_worth`) and
//! by the protection pool. It returns None for every permanent that is not p's own Karn card (so every other card, deck and
//! copy of Karn keeps today's value), Some(0.0) when a blink gains nothing (no colour, an opposing Elesh Norn, Doomsday's
//! pile is being played) and Some(negative) when it is harmful (it would deck p: `DECKING`; a stolen Karn goes back to his
//! owner, who draws: `STOLEN`). The value replaces the generic "has an enters hook" value (it is not added to it):
//! `PER_CARD` for each card that stays in hand and `PER_DISCARD` for each one that would be discarded at the cleanup step
//! (only when the draw is at p's own end step and p has a maximum hand size). The draws are counted before the blink, with
//! `enters_trigger_times` (Gandalf, a Norn of p's own double it, an opposing one blanks it). The guard leaves
//! `LIBRARY_FLOOR` cards in the library, except in p's own main phase with Thassa's Oracle castable in hand and the draws
//! no larger than the library, and the library within the Oracle's reach: the draw then does not lose first (`oracle_prio`
//! casts the Oracle once the library is that low).
//!
//! Limits: a Cursed Mirror or Metamorph never copies a commander, so there is no copy to blink; the AI does not cast him
//! at instant speed under Gandalf; the Oracle exception is only for p's own main phase (at an end step the next draw comes
//! before any Oracle can be cast); an opponent's Flickerwisp on him still sends him to the command zone.

use crate::cards::{CardDb, Colors, Types};
use crate::engine::chars::{card_types, colors};
use crate::engine::zones::draw;
use crate::flow::Res;
use crate::hooks::{Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game, Step};

pub const KARN_GILDED: &str = "Karn, Gilded Guardian";

/// PROVISIONAL: the priority (0-90 scale of `ai::decks::deck_prio`) with no coloured artifact to count: the body alone
const PRIO_BASE: i32 = 40;
/// PROVISIONAL: what each colour among the controller's artifacts adds (five colours: 80)
const PRIO_STEP: i32 = 8;

/// is the permanent an artifact now (a card's type, an artifact token, Liquimetal Torque's mark)? Face-down ones are
/// not: they have no types. The same test as `impls/dockside.rs`.
fn artifact(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    card_types(g, m).has(Types::ARTIFACT) || crate::impls::t5::is_artifact(g, m) || x.ttypes.contains(&"artifact")
}

/// the colours among the artifacts `o` controls now, leaving out `karn` (None: no permanent of Karn's, as when the AI
/// values the card in hand). Phased-out and face-down permanents are not counted.
fn colour_set(g: &Game, o: PlayerId, karn: Option<PermId>) -> Colors {
    let q = g.player(o);
    let mut seen: Vec<PermId> = Vec::with_capacity(q.perms.len() + q.lands.len());
    let mut all = Colors::NONE;
    for &m in q.perms.iter().chain(q.lands.iter()) {
        if Some(m) == karn || seen.contains(&m) {
            continue; // an animated land can sit in both lists
        }
        seen.push(m);
        let x = g.perm(m);
        if !x.on_bf || x.phased || x.data.truthy(DataKey::Facedown) {
            continue;
        }
        if artifact(g, m) {
            all = all.union(colors(g, m));
        }
    }
    all
}

/// "a card for each color among other artifacts you control": the number of colours among the artifacts `o` controls
/// other than the permanent `karn`, as the game is now (0 to 5).
pub fn colours_among_other_artifacts(g: &Game, o: PlayerId, karn: PermId) -> u32 {
    colour_set(g, o, Some(karn)).count()
}

/// "When Karn enters, draw a card for each color among other artifacts you control." The trigger goes on the stack (the
/// table may respond, Elesh Norn blanks it, Gandalf and your Norn repeat it); the colours are counted as it resolves.
fn karn_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = crate::engine::hooks::trigger_controller(g, src);
    if !crate::engine::stack::trigger_window(
        g,
        o,
        Some(src),
        "draw a card for each color among other artifacts",
        Some(3.0),
    )? {
        return Ok(()); // countered
    }
    let n = colours_among_other_artifacts(g, o, src);
    crate::glog!(g, "  Karn, Gilded Guardian: {} colours among the other artifacts", n);
    draw(g, o, n, false)
}

/// cards that must stay in the library after the draws (more would leave the next draw step with too little margin)
pub const LIBRARY_FLOOR: usize = 3;
/// the value of a blink that would deck p: far below any gain
pub const DECKING: f64 = -20.0;
/// the value of blinking a Karn p controls but does not own: he returns to his owner, who draws
pub const STOLEN: f64 = -5.0;
/// what a card that stays in hand is worth (`mine::etb_value`'s Draw rate)
pub const PER_CARD: f64 = 1.5;
/// what a card that would be discarded at the cleanup step is worth (the worst card is thrown away, but it was a choice)
pub const PER_DISCARD: f64 = 0.25;
/// cards in hand p keeps at the cleanup step
const HAND_SIZE: usize = 7;

/// what blinking p's permanent `m` is worth to p's AI when it is p's own Karn (by the physical card, so a copy of Karn is
/// not one), None when it is not (the caller keeps its own value): see the module doc for the three-way result.
/// `own_end_step`: the draw lands in p's own end step, before the cleanup step's discard. Reads the game only (no RNG).
pub fn reenter_bonus(g: &Game, p: PlayerId, m: PermId, own_end_step: bool) -> Option<f64> {
    let x = g.perm(m);
    if x.token || x.owner != p {
        return None;
    }
    let (cd, card) = (x.cd?, x.phys.or(x.cd)?);
    if &*g.db.get(cd).name != KARN_GILDED || &*g.db.get(card).name != KARN_GILDED {
        return None;
    }
    if x.orig != p {
        return Some(STOLEN);
    }
    let n = colours_among_other_artifacts(g, p, m) as usize
        * crate::engine::hooks::enters_trigger_times(g, p, None, Some(m)) as usize;
    if n == 0 {
        return Some(0.0);
    }
    let capped = own_end_step && !crate::engine::turn::no_max_hand(g, p);
    Some(draw_worth(g, p, n, capped))
}

/// the worth of `n` draws to p now (the tail of `reenter_bonus`, shared with `copy_draw_value`): `DECKING` if they leave
/// the library at or under `LIBRARY_FLOOR` (the Oracle exception as in `oracle_covers`; also under a Doomsday pile: a
/// draw that takes the pile down to the floor is no protection), 0 under a Doomsday pile, else `PER_CARD` for each card
/// kept and `PER_DISCARD` for each one discarded at the cleanup step when `capped`
fn draw_worth(g: &Game, p: PlayerId, n: usize, capped: bool) -> f64 {
    let library = g.player(p).library.len();
    if library <= n + LIBRARY_FLOOR && !oracle_covers(g, p, n, library) {
        return DECKING;
    }
    if crate::impls::doomsday::active(g, p) {
        return 0.0; // the pile is planned: the choosers never blink him for it
    }
    let kept = if capped { n.min(HAND_SIZE.saturating_sub(crate::engine::values::hand_proper(g, p).len())) } else { n };
    PER_CARD * kept as f64 + PER_DISCARD * (n - kept) as f64
}

/// a token copy of Karn entering (The Fire Crystal): its trigger draws a card for each colour among the other artifacts,
/// and the real Karn is one of them (all five colours), so it counts every colour among p's artifacts as they are now,
/// times the instances of the trigger (`enters_trigger_times`: 0 under an opposing Elesh Norn). The cards come before
/// p's turn, so none is discarded (uncapped). `karn` is the permanent the copy stands in for. Reads the game only.
pub fn copy_draw_value(g: &Game, p: PlayerId, karn: PermId) -> f64 {
    let n = colour_set(g, p, None).count() as usize
        * crate::engine::hooks::enters_trigger_times(g, p, None, Some(karn)) as usize;
    if n == 0 { 0.0 } else { draw_worth(g, p, n, false) }
}

/// `reenter_bonus` with the timing read from the game: the draw is in p's own end step now
pub fn reenter_now(g: &Game, p: PlayerId, m: PermId) -> Option<f64> {
    reenter_bonus(g, p, m, g.active == Some(p) && g.step == Step::End)
}

/// the Oracle line: p's own main phase, Thassa's Oracle in hand and castable now, the `n` draws no larger than the
/// library (a draw from an empty library loses at the next state check, before any Oracle), and the library within the
/// Oracle's reach after them (`combos::oracle_prio`: devotion plus the Oracle's own two)
fn oracle_covers(g: &Game, p: PlayerId, n: usize, library: usize) -> bool {
    if g.active != Some(p) || !matches!(g.step, Step::Main1 | Step::Main2) || n > library {
        return false;
    }
    let Some(o) = g.db.id("Thassa's Oracle").filter(|c| g.player(p).hand.contains(c)) else { return false };
    let d = g.db.get(o);
    crate::engine::cast::castable(g, p, o, "hand")
        && crate::engine::mana::can_pay_spell(g, p, o, d.generic, &d.pips, false)
        && library - n <= crate::impls::combos::oracle_devotion(g, p) + 2
}

/// the AI's cast priority (see the module doc)
fn karn_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    PRIO_BASE + PRIO_STEP * colour_set(g, p, None).count() as i32
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, KARN_GILDED)?;
    c.etb = Some(karn_etb);
    c.prio = Some(karn_prio);
    Ok(())
}
