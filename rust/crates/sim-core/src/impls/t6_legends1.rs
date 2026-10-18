//! Tier 6 "legends1" batch: Sisay's legends (Derevi, Emiel, Selvala, Agatha's Soul Cauldron, Deadpool, Ioreth,
//! Marvin, Shang-Chi, Tyvar, Loki's Scepter). Cards only: no shared engine code is changed here (what the engine
//! must still learn is in rust/agents/engine-needs-legends1.md). Each card's doc comment says how it is modeled.

use crate::engine::stack::trigger_window;
use crate::engine::values::{epow, pval, untargetable};
use crate::engine::mana::{can_pay, pay};
use crate::engine::zones::{die, draw, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
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

// ================================================================== Agatha's Soul Cauldron
/// the creature card in an opponent's graveyard worth exiling most: (owner, card)
fn cauldron_pick(g: &Game, p: PlayerId) -> Option<(PlayerId, crate::ids::CardId)> {
    let mut best: Option<(PlayerId, crate::ids::CardId, f64)> = None;
    for q in g.opps(p) {
        for &c in &g.player(q).gy {
            if !g.db.get(c).creature {
                continue;
            }
            let w = crate::engine::tutors::card_worth(g, q, c, true);
            if best.is_none_or(|b| w > b.2) {
                best = Some((q, c, w));
            }
        }
    }
    best.map(|b| (b.0, b.1))
}

/// **Agatha's Soul Cauldron**: {T}: exile target card from a graveyard; when a creature card is exiled this way, put a
/// +1/+1 counter on target creature you control. The AI exiles the best creature card from an opponent's graveyard
/// (denying reanimation and recursion) onto its best creature, in the second main phase or the end-of-turn window.
/// Not modeled: spending mana as any colour on creature abilities, and the exiled creatures' abilities for creatures
/// with +1/+1 counters (see engine-needs-legends1.md).
fn cauldron_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post == Some(false) || !live(g, src) || owner(g, src) != p || g.perm(src).tapped {
        return Ok(vec![]);
    }
    if cauldron_pick(g, p).is_none() || !g.player(p).perms.iter().any(|&m| live(g, m) && g.is_creature(m)) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 1.0,
        label: "exile a creature card from a graveyard (Cauldron)".into(),
        act: Some(Action::Ability { src, f: cauldron_go, arg: 0 }),
    }])
}

fn cauldron_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || g.perm(src).tapped {
        return Ok(false);
    }
    let Some((q, c)) = cauldron_pick(g, p) else { return Ok(false) };
    let mine: Vec<PermId> = g.player(p).perms.iter().copied().filter(|&m| live(g, m) && g.is_creature(m)).collect();
    let Some(t) = max_by(&mine, |m| pval(g, m)) else { return Ok(false) };
    g.perm_mut(src).tapped = true;
    g.player_mut(q).gy.retain(|&x| x != c);
    g.player_mut(q).exile.push(c);
    crate::cardcode::add_counters(g, t, 1);
    crate::glog!(g, "  Agatha's Soul Cauldron exiles {}, +1/+1 counter on {}", g.db.get(c).name, g.perm(t).name);
    Ok(true)
}

// ================================================================== Deadpool, Trading Card
/// **Deadpool, Trading Card**: at the beginning of your upkeep you lose 3 life (compiled from the Oracle text);
/// {3}, sacrifice it: each other player draws a card. The AI sacrifices it only to stop the drain from killing it
/// (life 6 or less) with {3} spare. Not modeled: "as it enters, you may exchange its text box and another creature's"
/// (see engine-needs-legends1.md).
fn deadpool_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post == Some(true) || !live(g, src) || owner(g, src) != p || g.player(p).life > 6 || !can_pay(g, p, 3, "", false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: 4.0,
        label: "sacrifice Deadpool (each other player draws)".into(),
        act: Some(Action::Ability { src, f: deadpool_go, arg: 0 }),
    }])
}

fn deadpool_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    if !live(g, src) || owner(g, src) != p || !pay(g, p, 3, "", false)? {
        return Ok(false);
    }
    crate::glog!(g, "  {} sacrifices Deadpool", g.player(p).name);
    die(g, src, "sac")?;
    for q in g.opps(p).collect::<Vec<_>>() {
        draw(g, q, 1, false)?;
    }
    Ok(true)
}

pub fn register(r: &mut Registry, db: &crate::cards::CardDb) -> Result<(), String> {
    let c = r.card(db, "Derevi, Empyrial Tactician")?;
    c.etb = Some(derevi_etb);
    c.combat_damage = Some(derevi_hit);
    r.card(db, "Emiel the Blessed")?.etb = Some(emiel_etb);
    let c = r.card(db, "Selvala, Heart of the Wilds")?;
    c.etb = Some(selvala_etb);
    c.dyn_mana_perm = Some(selvala_mana);
    r.card(db, "Agatha's Soul Cauldron")?.options = Some(cauldron_options);
    r.card(db, "Deadpool, Trading Card")?.options = Some(deadpool_options);
    Ok(())
}
