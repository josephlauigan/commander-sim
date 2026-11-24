//! Dockside Extortionist ({1}{R}, Creature - Goblin Pirate 1/2). Oracle (data/scryfall_cache.json): "When this creature
//! enters, create X Treasure tokens, where X is the number of artifacts and enchantments your opponents control."
//! Banned in Commander, legal in Cali Commander by the format's own ban list (data/formats.toml `[cali]`).
//!
//! The body is the card's data; the enter trigger is card code:
//!
//! - **A triggered ability that uses the stack** (a hook not marked at once), so an opponent's Elesh Norn blanks it,
//!   Panharmonicon and the other doublers repeat it (each instance counts again at its own resolution), and the table may
//!   respond (a person is asked nothing: there is no choice to make).
//! - **X is counted at resolution** (CR 608.2h: an effect that counts uses the game as it is then, not as the trigger was
//!   put on the stack), so a response that destroys or adds an artifact or enchantment changes it. The count is every
//!   artifact and enchantment permanent a living opponent of the controller has (the controller as the trigger was put on
//!   the stack, `trigger_controller`): artifact creatures, enchantment creatures and artifact lands once each, tokens
//!   (Treasure, Food, Clue and Blood are counters on the player in this engine, `Player::treasures` and so on; a Construct
//!   or Thopter is a permanent with the `artifact` type), phased-out permanents and face-down ones (a 2/2 with no types,
//!   CR 708.2) not. The controller's own are never counted. X = 0 makes no tokens.
//! - **The Treasures** are `zones::add_treasure` (the engine's one Treasure maker): they are artifacts for every count
//!   (Mox Opal's metalcraft, Tinker, Urza, Karn ...), tap for one mana of any colour, and Academy Manufactor, Xorn,
//!   Chatterfang and the token doublers apply. They come in untapped.
//!
//! AI (PROVISIONAL, never tuned by a sim; the rest of a deck's plan can still override it with a `[[prio]]` row): the cast
//! priority below, read by the generic cast logic. A 1/2 for two is no body, so the card is cast for its Treasures: it
//! waits while no opponent has an artifact or an enchantment (a later turn may give more), and is cast from one
//! onwards, sooner the more there are.

use crate::cards::{CardDb, Types};
use crate::engine::chars::card_types;
use crate::engine::zones::add_treasure;
use crate::flow::Res;
use crate::hooks::{Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game};

pub const DOCKSIDE: &str = "Dockside Extortionist";

/// priority (0-90 scale of `ai::decks::deck_prio`) when no opponent has an artifact or an enchantment: held
const HOLD: i32 = 0;
/// PROVISIONAL: the priority with one artifact or enchantment across the table; each more adds `PRIO_STEP`
const PRIO_BASE: i32 = 52;
const PRIO_STEP: i32 = 5;
/// PROVISIONAL: the most it adds (six or more: 77)
const PRIO_MAX_COUNT: u32 = 6;

/// is the permanent an artifact now (a card's type, an artifact token, Liquimetal Torque's mark)? Face-down ones are
/// not: they have no types.
fn artifact(g: &Game, m: PermId) -> bool {
    let x = g.perm(m);
    card_types(g, m).has(Types::ARTIFACT) || crate::impls::t5::is_artifact(g, m) || x.ttypes.contains(&"artifact")
}

fn enchantment(g: &Game, m: PermId) -> bool {
    card_types(g, m).has(Types::ENCHANTMENT) || g.perm(m).ttypes.contains(&"enchantment")
}

/// X for a Dockside Extortionist controlled by `o`: the artifacts and enchantments the living opponents of `o` control
/// (each permanent once, even an artifact enchantment; the Treasure, Food, Clue and Blood tokens each once). An opponent
/// under Teferi's Protection (`life_locked` and `ring_prot`, both set by it) controls nothing: it is all phased out.
pub fn opposing_artifacts_and_enchantments(g: &Game, o: PlayerId) -> u32 {
    let mut n: u32 = 0;
    for q in g.players.iter().filter(|q| q.alive && q.id != o) {
        if q.life_locked && q.ring_prot {
            continue; // Teferi's Protection: all it controls is phased out, its token counters and lands too (702.26)
        }
        let mut seen: Vec<PermId> = Vec::with_capacity(q.perms.len() + q.lands.len());
        for &m in q.perms.iter().chain(q.lands.iter()) {
            if seen.contains(&m) {
                continue; // an animated land can sit in both lists
            }
            seen.push(m);
            let x = g.perm(m);
            if !x.on_bf || x.phased || x.data.truthy(DataKey::Facedown) {
                continue;
            }
            if artifact(g, m) || enchantment(g, m) {
                n = n.saturating_add(1);
            }
        }
        n = n.saturating_add(q.treasures).saturating_add(q.foods).saturating_add(q.clues).saturating_add(q.bloods);
    }
    n
}

/// "When this creature enters, create X Treasure tokens." The trigger goes on the stack (the table may respond, Elesh
/// Norn blanks it, a doubler repeats it); X is counted as it resolves.
fn dockside_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = crate::engine::hooks::trigger_controller(g, src);
    if !crate::engine::stack::trigger_window(g, o, Some(src), "create X Treasures", Some(3.0))? {
        return Ok(()); // countered
    }
    let x = opposing_artifacts_and_enchantments(g, o);
    crate::glog!(g, "  Dockside Extortionist: X = {} (the artifacts and enchantments the opponents control)", x);
    add_treasure(g, o, x.min(i32::MAX as u32) as i32)
}

/// the AI's cast priority (see the module doc; the controller's opponents are `p`'s)
fn dockside_prio(g: &Game, p: PlayerId, _c: CardId) -> i32 {
    match opposing_artifacts_and_enchantments(g, p) {
        0 => HOLD,
        n => PRIO_BASE + PRIO_STEP * (n.min(PRIO_MAX_COUNT) as i32 - 1),
    }
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    let c = r.card(db, DOCKSIDE)?;
    c.etb = Some(dockside_etb);
    c.prio = Some(dockside_prio);
    Ok(())
}
