//! Scion of the Ur-Dragon's dragon line for JD's `scion` deck (rust/agents/scion-goldfish.md, Part 3), and Prismari,
//! the Inspiration. Oracle text (data/scryfall_cache.json):
//!
//! - **Scion of the Ur-Dragon**: "{2}: Search your library for a Dragon permanent card and put it into your graveyard.
//!   If you do, Scion of the Ur-Dragon becomes a copy of that card until end of turn. Then shuffle." No {T}: it works
//!   the turn Scion enters. Rulings: once it is a copy it has lost this ability for the turn, but activations already
//!   on the stack still resolve (several can be stacked first); taking the card out of the graveyard does not end
//!   the copy.
//! - **Prismari, the Inspiration** ({5}{U}{R}, 7/7 flying Elder Dragon): "Ward—Pay 5 life. Instant and sorcery spells
//!   you cast have storm."
//!
//! The copy: Mirage Mirror's machinery (`alela::set_face`): the permanent's card becomes the Dragon's (name, body,
//! keywords, card code), so the Dragon's hooks fire for it (Ancient Copper Dragon's Treasures, Korvold's sacrifice and
//! draws, Prismari's storm) and Scion's own ability (registered on Scion's card) is gone. Scion's card stays in `phys`
//! (it is still the commander: it goes to the command zone if it leaves). `copy_revert` ends the copy as the next
//! turn starts (after the end step and its window, the cleanup in effect).
//!
//! Prismari's storm: a cast trigger of each Prismari (or Scion as Prismari): an instant or sorcery its controller
//! casts is copied once per spell cast before it this turn. A storm card (Brain Freeze) gets this as a second storm
//! trigger: its copies mill 3 each, on the targets after the ones its own storm uses.
//!
//! The Scion AI (`scion_options`, deck `scion` only; The Ur-Dragon pool deck keeps its Rivaz use): before combat on its
//! own turn, with {2} spare and Scion able to attack, become Ancient Copper Dragon (a d20 of Treasures when it
//! connects); with Copper gone and a Treasure to feed it, Korvold. Prismari is fetched only as a step of the storm line
//! (`t6_combos_scion.rs`), where its storm wins the game.

use crate::cards::CardDb;
use crate::engine::cast::copy_spell;
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::{ability_window, trigger_window};
use crate::engine::tutors::shuffle_library;
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{DataKey, Game, Val};
use crate::sym::intern;

pub const SCION: &str = "Scion of the Ur-Dragon";
pub const PRISMARI: &str = "Prismari, the Inspiration";
pub const COPPER: &str = "Ancient Copper Dragon";
pub const KORVOLD: &str = "Korvold, Fae-Cursed King";

/// the Dragons the AI fetches, by option argument
const FETCH: [&str; 3] = [COPPER, KORVOLD, PRISMARI];

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn cname(g: &Game, c: CardId) -> &str {
    &g.db.get(c).name
}

/// is permanent m a Scion that is a copy right now
pub fn is_copy(g: &Game, m: PermId) -> bool {
    g.perm(m).data.truthy(DataKey::ScionCopy)
}

/// p's Scion of the Ur-Dragon on the battlefield as itself (with its ability)
pub fn scion_of(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && g.perm(m).cd.is_some_and(|c| cname(g, c) == SCION))
}

/// p controls a Prismari (the card, or Scion as a copy of it): its instants and sorceries have storm
pub fn prismari_on(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| live(g, m) && g.perm(m).cd.is_some_and(|c| cname(g, c) == PRISMARI))
}

/// a Dragon permanent card of this name in p's library
fn dragon_in_lib(g: &Game, p: PlayerId, name: &str) -> Option<CardId> {
    g.player(p).library.iter().copied().find(|&c| {
        let d = g.db.get(c);
        &*d.name == name && d.perm && super::t45_urdragon::dragon_card(g, c)
    })
}

/// Scion (permanent m) becomes a copy of card cd until end of turn; a second copy replaces the first (the newest
/// copy effect wins), and Scion's card stays in `phys`
pub fn become_copy(g: &mut Game, m: PermId, cd: CardId) {
    let own = if is_copy(g, m) { g.perm(m).phys } else { g.perm(m).cd };
    crate::impls::alela::set_face(g, m, cd);
    let x = g.perm_mut(m);
    x.phys = own;
    x.data.set(DataKey::ScionCopy, Val::Bool(true));
    crate::glog!(g, "    Scion of the Ur-Dragon becomes a copy of {}", cname(g, cd));
}

/// engine/turn.rs, at the start of every turn: Scion's copies end ("until end of turn")
pub fn copy_revert(g: &mut Game) {
    for i in 0..g.perms.len() {
        let m = PermId(i as u32);
        if !g.perm(m).data.truthy(DataKey::ScionCopy) {
            continue;
        }
        g.perm_mut(m).data.remove(DataKey::ScionCopy);
        if !g.perm(m).on_bf {
            continue;
        }
        if let Some(own) = g.perm(m).phys {
            crate::impls::alela::set_face(g, m, own);
            g.perm_mut(m).phys = None;
        }
    }
}

/// Activate Scion's {2} once per name in `dragons`, all put on the stack before any resolves (so all resolve even
/// though the first to resolve takes the ability away); they resolve last first, each searching for its Dragon
/// (if it is still in the library), putting it into the graveyard and making Scion a copy of it, then shuffling.
/// False if Scion can't activate (not itself, or the mana for every activation isn't there).
pub fn activate(g: &mut Game, src: PermId, p: PlayerId, dragons: &[&str]) -> Res<bool> {
    let k = dragons.len() as u32;
    let own = g.perm(src).cd.is_some_and(|c| cname(g, c) == SCION);
    if k == 0 || !live(g, src) || g.perm(src).owner != p || !own || !can_pay(g, p, 2 * k, "", false) {
        return Ok(false);
    }
    pay(g, p, 2 * k, "", false)?;
    for name in dragons.iter().rev() {
        crate::glog!(g, "  {} activates Scion of the Ur-Dragon: search for {name}", g.player(p).name);
        let label = format!("search for {name}, become a copy of it");
        if !ability_window(g, p, Some(src), &label, None, None)? || g.over {
            continue; // countered (Stifle)
        }
        if let Some(c) = dragon_in_lib(g, p, name) {
            let lib = &mut g.player_mut(p).library;
            if let Some(i) = lib.iter().position(|&x| x == c) {
                lib.remove(i);
            }
            g.player_mut(p).gy.push(c);
            g.player_mut(p).stat(intern(&format!("scion|act|{name}")), 1);
            if live(g, src) {
                become_copy(g, src, c);
            }
        }
        shuffle_library(g, p);
    }
    Ok(true)
}

/// the AI's Scion activation (deck `scion`): precombat on its own turn, Scion untapped and able to attack, {2}
/// spare: Ancient Copper Dragon (its combat damage makes a d20 of Treasures); with Copper gone and a Treasure for
/// Korvold's attack trigger to sacrifice, Korvold (a card per sacrifice). Prismari only as a step of the storm line.
pub fn scion_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    let x = g.perm(src);
    if post != Some(false)
        || g.active != Some(p)
        || x.owner != p
        || !live(g, src)
        || x.sick
        || x.tapped
        || is_copy(g, src)
        || !can_pay(g, p, 2, "", false)
        || g.opps(p).next().is_none()
    {
        return Ok(vec![]);
    }
    let pick = if dragon_in_lib(g, p, COPPER).is_some() {
        Some((0, 4.0))
    } else if dragon_in_lib(g, p, KORVOLD).is_some() && g.player(p).treasures >= 1 {
        Some((1, 2.0))
    } else {
        None
    };
    let Some((i, u)) = pick else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: u,
        label: format!("Scion of the Ur-Dragon: become {}", FETCH[i]),
        act: Some(Action::Ability { src, f: scion_go, arg: i as i64 }),
    }])
}

fn scion_go(g: &mut Game, src: PermId, p: PlayerId, arg: i64) -> Res<bool> {
    let name = FETCH[(arg as usize).min(FETCH.len() - 1)];
    activate(g, src, p, &[name])
}

/// Prismari: an instant or sorcery its controller casts has storm (one copy per spell cast before it this turn)
fn prismari_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let p = g.perm(src).owner;
    let d = g.db.get(c);
    if caster != p || !(d.instant || d.sorcery) || !live(g, src) {
        return Ok(());
    }
    let n = crate::impls::mine::storm_count(g);
    if n <= 0 {
        return Ok(());
    }
    let ctx = match &g.cur_cast {
        Some((cc, ctx, _)) if *cc == c => Some(ctx.clone()),
        _ => None,
    };
    let name = cname(g, c).to_string();
    if !trigger_window(g, p, Some(src), &format!("storm: copy {name} {n} time(s)"), Some(6.0))? {
        return Ok(());
    }
    crate::glog!(g, "    Prismari's storm copies {name} {n} time(s)");
    if g.db.get(c).tags.has(crate::tag::Tag::Stormmill) {
        // a second storm trigger on Brain Freeze: n copies, three cards each, on the targets after the n + 1 its own
        // storm mills (one copy carries the count, as the card's own storm does)
        let mut cx = ctx.unwrap_or_default();
        cx.mill = cx.mill.iter().copied().skip(n as usize + 1).collect();
        cx.storm = Some(n - 1);
        return copy_spell(g, p, c, Some(&cx), false);
    }
    for _ in 0..n {
        copy_spell(g, p, c, ctx.as_ref(), false)?;
        if g.over || !g.player(p).alive {
            break;
        }
    }
    Ok(())
}

/// "Ward—Pay 5 life" on Prismari (or Scion as a copy of it): the actor pays 5 life, or (at 5 life or less) the
/// spell is countered (false)
pub fn prismari_ward(g: &mut Game, actor: PlayerId, m: PermId) -> Res<bool> {
    if !g.perm(m).cd.is_some_and(|c| cname(g, c) == PRISMARI) {
        return Ok(true);
    }
    if g.player(actor).life <= 5 {
        return Ok(false);
    }
    crate::glog!(g, "    {} pays 5 life for ward (Prismari)", g.player(actor).name);
    lose_life(g, actor, 5, Some(actor), "life", Some(false))?;
    Ok(true)
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    if db.id(PRISMARI).is_some() {
        r.card(db, PRISMARI)?.cast = Some(prismari_cast);
    }
    Ok(())
}
