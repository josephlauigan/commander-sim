//! **K'rrik, Son of Yawgmoth** (`sheoldred` seat): {4}{B/P}{B/P}{B/P}, 2/2 lifelink.
//!
//! - "For each {B} in a cost, you may pay 2 life rather than pay that mana." While p controls K'rrik, the black pips of
//!   a black spell p casts from hand or the command zone can be paid with life. The AI (`mana::cost_of`) drops a black
//!   pip it cannot pay with mana, paying 2 life for it only while its life stays at or above `LIFE_FLOOR` afterwards;
//!   `mana::phyrexian_life` counts the life, which `brain::do_cast` pays with the other Phyrexian life. A person gets
//!   every black pip as a mana-or-life choice (`human::legal::phyrexian`), with no floor.
//!   Not covered: costs of abilities, and removal spells (`rem_cost` uses the printed cost).
//! - "Whenever you cast a black spell, put a +1/+1 counter on K'rrik" (`krrik_cast`). He is not on the battlefield
//!   while he is cast, so his own cast gives no counter.
//!
//! The floor of 15: Sheoldred's plan drains and gains, so life is a resource, but the table is tier 3-4 (Sheoldred
//! sits at 40 and the opponents' attacks, drains and Sheoldred's own triggers on a draw cost several points a round);
//! 15 keeps the AI out of one-round kill range and above the 10 the existing Phyrexian rule uses, while each
//! paid pip is worth a whole mana. Tests: tests/cards_krrik.rs.

use crate::engine::repl;
use crate::cards::CardDb;
use crate::flow::Res;
use crate::hooks::{Registry, Src};
use crate::ids::{CardId, PlayerId};
use crate::state::Game;

pub const KRRIK: &str = "K'rrik, Son of Yawgmoth";
/// the AI pays life for {B} only while its life stays at or above this afterwards
pub const LIFE_FLOOR: i32 = 15;

/// p controls K'rrik
pub fn out(g: &Game, p: PlayerId) -> bool {
    g.player(p).perms.iter().any(|&m| {
        let x = g.perm(m);
        x.on_bf && !x.phased && x.owner == p && x.cd.is_some_and(|c| &*g.db.get(c).name == KRRIK)
    })
}

/// the colours of c's pips p may pay with 2 life: the printed Phyrexian symbols, and with K'rrik out every {B} of a
/// black spell
pub fn life_pips(g: &Game, p: PlayerId, c: CardId) -> String {
    let d = g.db.get(c);
    if !d.pips.contains('B') || d.land || !out(g, p) {
        return d.phyrexian.to_string();
    }
    let mut s: String = d.phyrexian.chars().filter(|&x| x != 'B').collect();
    s.extend(d.pips.chars().filter(|&x| x == 'B'));
    s
}

/// cost_of: drop black pips of c that the mana can't pay, as long as p keeps `LIFE_FLOOR` life
pub fn drop_black(g: &Game, p: PlayerId, c: CardId, gn: i32, pips: &mut String) {
    let d = g.db.get(c);
    if !d.pips.contains('B') || d.land || !out(g, p) {
        return;
    }
    let printed = d.pips.matches('B').count() as i32;
    loop {
        let missing = printed - pips.matches('B').count() as i32;
        if !pips.contains('B')
            || g.player(p).life - 2 * (missing + 1) < LIFE_FLOOR
            || crate::engine::mana::can_pay(g, p, gn.max(0) as u32, pips, false)
        {
            return;
        }
        *pips = pips.replacen('B', "", 1);
    }
}

/// whenever p casts a black spell, a +1/+1 counter on K'rrik
fn krrik_cast(g: &mut Game, src: Src, caster: PlayerId, c: CardId) -> Res {
    let x = g.perm(src);
    if caster != x.owner || !x.on_bf || x.phased || !g.db.get(c).colors.has('B') {
        return Ok(());
    }
    repl::put_plus(g, src, 1);
    crate::glog!(g, "    K'rrik grows (a black spell was cast)");
    Ok(())
}

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, KRRIK)?.cast = Some(krrik_cast);
    Ok(())
}
