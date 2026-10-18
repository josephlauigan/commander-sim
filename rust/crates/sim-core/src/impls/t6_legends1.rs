//! Tier 6 "legends1" batch: Sisay's legends (Derevi, Emiel, Selvala, Agatha's Soul Cauldron, Deadpool, Ioreth,
//! Marvin, Shang-Chi, Tyvar, Loki's Scepter). Cards only: no shared engine code is changed here (what the engine
//! must still learn is in rust/agents/engine-needs-legends1.md). Each card's doc comment says how it is modeled.

use crate::engine::stack::trigger_window;
use crate::engine::values::{epow, pval, untargetable};
use crate::engine::zones::max_by;
use crate::flow::Res;
use crate::hooks::{Event, Registry, Src};
use crate::ids::{PermId, PlayerId};
use crate::state::Game;
use crate::tag::Tag;

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

// ================================================================== Derevi, Empyrial Tactician
/// what untapping the tapped permanent m is worth to p: the mana it makes (a rock, a dork, a land that taps for
/// more than one), or half its power value for another creature
fn untap_gain(g: &Game, p: PlayerId, m: PermId) -> f64 {
    let x = g.perm(m);
    let Some(c) = x.cd else { return 0.0 };
    if !x.tapped || !live(g, m) {
        return 0.0;
    }
    let d = g.db.get(c);
    if let Some(rock) = d.tags.str(Tag::Rock) {
        let a = rock.split_once(':').map_or(rock, |s| s.0);
        return crate::cardcode::dyn_mana_perm(g, p, m).unwrap_or_else(|| a.parse().unwrap_or(1)) as f64;
    }
    if d.tags.str(Tag::Dork).is_some() {
        return crate::cardcode::dyn_mana_perm(g, p, m).unwrap_or(1) as f64;
    }
    if g.is_creature(m) && !x.sick { 0.25 * pval(g, m) } else { 0.0 }
}

/// the tapped land of p that untapping pays the most: (land, mana it makes)
fn untap_land(g: &Game, p: PlayerId) -> Option<(crate::ids::LandId, f64)> {
    g.player(p)
        .lands
        .iter()
        .copied()
        .filter(|&l| g.land(l).tapped)
        .map(|l| {
            let amt = crate::cardcode::dyn_mana_land(g, p, l)
                .map_or_else(|| g.db.get(g.land(l).cd).tags.int(Tag::Amt).unwrap_or(1), |n| n as i32);
            (l, amt as f64)
        })
        .fold(None, |b: Option<(crate::ids::LandId, f64)>, x| match b {
            Some(y) if y.1 >= x.1 => Some(y),
            _ => Some(x),
        })
}

/// Derevi's "tap or untap target permanent": untap the tapped mana source (or creature) of the most worth; else,
/// on p's own turn, tap the best untapped creature an opponent could block with. Returns false if nothing was worth it.
fn derevi_pick(g: &mut Game, src: PermId, p: PlayerId) -> Res<bool> {
    let mine: Vec<PermId> = g.player(p).perms.clone();
    let bp = max_by(&mine, |m| untap_gain(g, p, m)).filter(|&m| untap_gain(g, p, m) > 0.0);
    let bl = untap_land(g, p);
    let pv = bp.map_or(0.0, |m| untap_gain(g, p, m));
    let lv = bl.map_or(0.0, |x| x.1);
    let opp = if g.active == Some(p) {
        let cs: Vec<PermId> = g
            .opps(p)
            .flat_map(|q| g.player(q).perms.clone())
            .filter(|&m| live(g, m) && g.is_creature(m) && !g.perm(m).tapped && !untargetable(g, m))
            .collect();
        max_by(&cs, |m| pval(g, m)).filter(|&m| epow(g, m) >= 2)
    } else {
        None
    };
    if pv > 0.0 && pv >= lv {
        let m = bp.unwrap();
        if !trigger_window(g, p, Some(src), "untap target permanent", None)? {
            return Ok(false);
        }
        g.perm_mut(m).tapped = false;
        crate::glog!(g, "    Derevi untaps {}", g.perm(m).name);
        return Ok(true);
    }
    if let Some((l, v)) = bl.filter(|x| x.1 > 0.0 && lv > 0.0) {
        let _ = v;
        if !trigger_window(g, p, Some(src), "untap target permanent", None)? {
            return Ok(false);
        }
        g.land_mut(l).tapped = false;
        crate::glog!(g, "    Derevi untaps a land");
        return Ok(true);
    }
    if let Some(m) = opp {
        if !trigger_window(g, p, Some(src), "tap target permanent", None)? {
            return Ok(false);
        }
        g.perm_mut(m).tapped = true;
        crate::glog!(g, "    Derevi taps {}", g.perm(m).name);
        return Ok(true);
    }
    Ok(false)
}

/// When Derevi enters: tap or untap target permanent
fn derevi_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m != src || p != owner(g, src) {
        return Ok(());
    }
    derevi_pick(g, src, p)?;
    Ok(())
}

/// whenever a creature you control deals combat damage to a player: tap or untap target permanent
fn derevi_hit(g: &mut Game, src: Src, p: PlayerId, _a: PermId, d: PlayerId, _dmg: i32) -> Res {
    if p != owner(g, src) || d == p || !live(g, src) {
        return Ok(());
    }
    derevi_pick(g, src, p)?;
    Ok(())
}

pub fn register(r: &mut Registry, db: &crate::cards::CardDb) -> Result<(), String> {
    let c = r.card(db, "Derevi, Empyrial Tactician")?;
    c.etb = Some(derevi_etb);
    c.combat_damage = Some(derevi_hit);
    let _ = Event::Etb;
    Ok(())
}
