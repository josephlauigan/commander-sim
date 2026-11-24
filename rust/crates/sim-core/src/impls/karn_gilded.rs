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

use crate::cards::{CardDb, Colors, Types};
use crate::engine::chars::{card_types, colors};
use crate::engine::zones::draw;
use crate::flow::Res;
use crate::hooks::{Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game};

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
