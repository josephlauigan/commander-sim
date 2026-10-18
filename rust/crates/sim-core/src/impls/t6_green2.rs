//! Tier 6 batch `green2`: the green and small utility cards of the Rograkh & Thrasios and Kinnan decks
//! (rust/agents/t6-batches.md). Cards only: no engine file is touched; what needs the engine is listed in
//! rust/agents/engine-needs-green2.md.
//!
//! - Chatterstorm: the Squirrel token is the compiled spell; storm is `t0_stella::storm_copies` on the cast.
//! - Cloud of Faeries: ETB untaps two lands; cycling {2} is a hand option.

use crate::cards::CardDb;
use crate::engine::mana::{can_pay, pay};
use crate::engine::stack::trigger_window;
use crate::engine::zones::{discard_cards, draw};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::Game;
use crate::sym::Sym;

fn pname(g: &Game, p: PlayerId) -> Sym {
    g.player(p).name
}

// ================================================================== Chatterstorm
/// Storm: when you cast it, copy it for each spell cast before it this turn
fn chatterstorm_cast(g: &mut Game, p: PlayerId, c: CardId) -> Res {
    crate::impls::t0_stella::storm_copies(g, p, c)
}

// ================================================================== Cloud of Faeries
/// When this creature enters, untap up to two lands: the AI untaps two tapped lands of its own (it always takes the
/// full two when it has them).
fn cloud_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    let o = g.perm(src).owner;
    if m != src || !trigger_window(g, o, Some(src), "untap up to two lands", Some(3.0))? {
        return Ok(());
    }
    let tapped: Vec<_> = g.player(o).lands.iter().copied().filter(|&l| g.land(l).tapped).take(2).collect();
    for l in &tapped {
        g.land_mut(*l).tapped = false;
    }
    crate::glog!(g, "    {} untaps {} land(s) (Cloud of Faeries)", pname(g, o), tapped.len());
    Ok(())
}

/// Cycling {2}: the AI cycles it in an opponent's end step or after combat when it has the two mana to spare.
fn cloud_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if !g.player(p).hand.contains(&c) || !(post.is_none() && g.active != Some(p) || post == Some(true)) {
        return Ok(vec![]);
    }
    if !can_pay(g, p, 2, "", false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 0.9,
        label: "cycle Cloud of Faeries".into(),
        act: Some(Action::Plan { f: cloud_cycle, arg: c.0 as i64 }),
    }])
}

fn cloud_cycle(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) || !can_pay(g, p, 2, "", false) {
        return Ok(false);
    }
    pay(g, p, 2, "", false)?;
    crate::glog!(g, "  {} cycles Cloud of Faeries", pname(g, p));
    discard_cards(g, p, &[c])?;
    draw(g, p, 1, false)?;
    Ok(true)
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Chatterstorm")?.self_cast = Some(chatterstorm_cast);
    let c = r.card(db, "Cloud of Faeries")?;
    c.etb = Some(cloud_etb);
    c.hand_options = Some(cloud_options);
    Ok(())
}
