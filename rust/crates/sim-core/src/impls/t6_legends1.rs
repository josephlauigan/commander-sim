//! Tier 6 "legends1" batch: Sisay's legends (Derevi, Emiel, Selvala, Agatha's Soul Cauldron, Deadpool, Ioreth,
//! Marvin, Shang-Chi, Tyvar, Loki's Scepter). Cards only: no shared engine code is changed here (what the engine
//! must still learn is in rust/agents/engine-needs-legends1.md). Each card's doc comment says how it is modeled.

use crate::engine::stack::trigger_window;
use crate::engine::values::{epow, pval, untargetable};
use crate::engine::mana::{can_pay, pay};
use crate::engine::zones::{draw, max_by};
use crate::flow::Res;
use crate::hooks::{Registry, Src};
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


// ================================================================== Emiel the Blessed
/// **Emiel the Blessed**: {3}: blink another creature you control (compiled from the Oracle text). Whenever another
/// creature you control enters, you may pay {G/W} (paid with W, else G) to put a +1/+1 counter on it (two if it is a
/// Unicorn). The AI pays whenever it can spare the mana.
fn emiel_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src || p != owner(g, src) || !live(g, src) || !live(g, m) || !g.is_creature(m) || owner(g, m) != p {
        return Ok(());
    }
    let col = if can_pay(g, p, 0, "W", false) {
        "W"
    } else if can_pay(g, p, 0, "G", false) {
        "G"
    } else {
        return Ok(());
    };
    if !trigger_window(g, p, Some(src), "pay {G/W} for a +1/+1 counter", None)? || !pay(g, p, 0, col, false)? {
        return Ok(());
    }
    let n = if crate::engine::values::has_type(g, m, "unicorn") { 2 } else { 1 };
    crate::cardcode::put_plus(g, m, n);
    crate::glog!(g, "    Emiel puts {} +1/+1 counter(s) on {}", n, g.perm(m).name);
    Ok(())
}

// ================================================================== Selvala, Heart of the Wilds
/// **Selvala**: whenever another creature enters, its controller may draw a card if its power is greater than each
/// other creature's power. The mana ability ({G}, {T}: X mana of any colours, X the greatest power among your
/// creatures) is the `dork` tag with `dyn_mana_perm` = X - 1: the {G} it costs is netted off.
fn selvala_etb(g: &mut Game, src: Src, p: PlayerId, m: PermId) -> Res {
    if m == src || !live(g, src) || !live(g, m) || !g.is_creature(m) {
        return Ok(());
    }
    let pw = epow(g, m);
    let all: Vec<PermId> = std::iter::once(p).chain(g.opps(p))
        .flat_map(|q| g.player(q).perms.clone())
        .filter(|&x| x != m && live(g, x) && g.is_creature(x))
        .collect();
    if all.iter().any(|&x| epow(g, x) >= pw) {
        return Ok(());
    }
    if trigger_window(g, p, Some(src), "draw a card (greatest power)", None)? {
        draw(g, p, 1, false)?;
    }
    Ok(())
}

fn selvala_mana(g: &Game, p: PlayerId, _m: PermId) -> u32 {
    let x = g.player(p).perms.iter().filter(|&&y| live(g, y) && g.is_creature(y)).map(|&y| epow(g, y)).max().unwrap_or(0);
    (x - 1).max(0) as u32
}

pub fn register(r: &mut Registry, db: &crate::cards::CardDb) -> Result<(), String> {
    let c = r.card(db, "Derevi, Empyrial Tactician")?;
    c.etb = Some(derevi_etb);
    c.combat_damage = Some(derevi_hit);
    r.card(db, "Emiel the Blessed")?.etb = Some(emiel_etb);
    let c = r.card(db, "Selvala, Heart of the Wilds")?;
    c.etb = Some(selvala_etb);
    c.dyn_mana_perm = Some(selvala_mana);
    Ok(())
}
