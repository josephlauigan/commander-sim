//! Sauron's speed and backup-win cards (brief: rust/agents/sauron-threats.md). The cards are in no decklist; the lead
//! tests them with `--swap`, so every AI decision here is gated on the Sauron seat (`key == "sauron"`) or works for
//! any deck holding the card.
//!
//! - **Grishnákh, Brash Instigator**: amass Orcs 2, then (a reflexive trigger) gain control of the best nonlegendary
//!   creature an opponent controls with power at most the Army's, until end of turn: untapped, hasty (`marchesa::steal`
//!   returns it at the end step).
//! - **Dreadhorde Invasion**: the compiled upkeep (lose 1, amass Zombies 1) plus the lifelink clause: a Zombie token with
//!   power 6 or more gains lifelink as it attacks. The Army is a Zombie (amass Zombies makes it one), so with Invasion out
//!   every Army counts; the one gap is the turn before Invasion's first upkeep, when an Orc Army that never amassed
//!   Zombies is not one yet.
//! - **Lazotep Reaver** (amass Zombies 1 on entering) and **Grim Initiate** (first strike; dies: amass Zombies 1) are
//!   compiled abilities; `Zones::amass` puts the counters on the Army you control whatever its type (701.47a).
//! - **Torment of Hailfire** ({X}{B}{B}): X times, each opponent loses 3 life unless they sacrifice a nonland permanent or
//!   discard a card. Each opponent takes the cheapest way out each time (`torment_pick`): the least valuable card or
//!   permanent, or the 3 life while their life total makes that the lesser loss. The AI casts it when it kills, or late.
//! - **Dualcaster Mage** (flash; enters: copy target instant or sorcery spell) is flashed in over a worthwhile spell
//!   (an X drain or Torment of Hailfire at X >= 3, burn, draw; the copy keeps X). There is no combo detection (JD: no
//!   infinite combos in Sauron); with no spell worth copying it is a plain 2/2 the AI casts late.
//! - **Exsanguinate** is yshtola.rs's X drain (its option now also runs for the Sauron seat); Talisman of
//!   Indulgence, Rakdos Signet and Fellwar Stone are modeled elsewhere (mine.rs, the rock tags, mana.rs).

use crate::cards::CardDb;
use crate::engine::cast::{cast_card, copy_spell};
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, cost_of, pay, total_mana};
use crate::engine::stack::trigger_window;
use crate::engine::tutors::card_worth;
use crate::engine::values::{army_of, epow, pval, untargetable};
use crate::engine::zones::{amass, die, discard_cards, max_by};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, Game};
use crate::sym::{Sym, intern};

pub const GRISHNAKH: &str = "Grishnákh, Brash Instigator";
pub const INVASION: &str = "Dreadhorde Invasion";
pub const TORMENT: &str = "Torment of Hailfire";
pub const DUALCASTER: &str = "Dualcaster Mage";

// ================================================================== Grishnákh, Brash Instigator
/// the creature Grishnákh's trigger takes: the most valuable nonlegendary opposing creature with power at most
/// `max_pow` (the amassed Army's power)
pub fn grishnakh_victim(g: &Game, p: PlayerId, max_pow: i32) -> Option<PermId> {
    let cands: Vec<PermId> = g
        .opps(p)
        .flat_map(|q| g.player(q).perms.iter().copied())
        .filter(|&m| {
            g.is_creature(m)
                && !g.perm(m).phased
                && !untargetable(g, m)
                && !crate::impls::mine::is_legendary(g, m)
                && epow(g, m) <= max_pow
        })
        .collect();
    max_by(&cands, |m| pval(g, m))
}

fn grishnakh_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    if !trigger_window(g, o, Some(src), "amass Orcs 2", None)? {
        return Ok(());
    }
    amass(g, o, 2)?;
    let Some(a) = army_of(g, o) else { return Ok(()) };
    let Some(t) = grishnakh_victim(g, o, epow(g, a)) else { return Ok(()) };
    let name = format!("gain control of {} until end of turn", g.perm(t).name);
    if trigger_window(g, o, Some(src), &name, None)? && g.perm(t).on_bf && !g.perm(t).phased {
        crate::impls::marchesa::steal(g, o, t, true);
    }
    Ok(())
}

// ================================================================== Dreadhorde Invasion
fn is_zombie_token(g: &Game, p: PlayerId, m: PermId) -> bool {
    let x = g.perm(m);
    x.token && x.owner == p && (x.army || crate::engine::values::has_type(g, m, "zombie"))
}

/// whenever a Zombie token you control with power 6 or greater attacks, it gains lifelink until end of turn
fn invasion_attack(g: &mut Game, src: Src, p: PlayerId, atk: &[PermId], _d: PlayerId) -> Res<Vec<PermId>> {
    if g.perm(src).owner != p || g.perm(src).phased {
        return Ok(vec![]);
    }
    let life = intern("lifelink");
    for &m in atk {
        if !is_zombie_token(g, p, m) || epow(g, m) < 6 || g.perm(m).eot_kw.contains(&life) {
            continue;
        }
        if trigger_window(g, p, Some(src), &format!("{} gains lifelink", g.perm(m).name), None)? {
            g.perm_mut(m).eot_kw.push(life);
        }
    }
    Ok(vec![])
}

// ================================================================== Torment of Hailfire
/// what an opponent gives up to Torment of Hailfire's process: the card/permanent and its worth, or the 3 life
#[derive(Clone, Copy, Debug, PartialEq)]
enum Give {
    Card(CardId),
    Perm(PermId),
    Life,
}

/// how much 3 life is worth to a player at this life total (a card is worth `card_worth / 10`, a permanent `pval`)
fn life_cost(life: i32) -> f64 {
    if life <= 3 { f64::INFINITY } else { 3.0 * (25.0 / life as f64) }
}

/// the cheapest way out for q: (the item, its worth), None when nothing is left to give
fn cheapest(g: &Game, q: PlayerId, skip_c: &[CardId], skip_m: &[PermId]) -> Option<(Give, f64)> {
    let pl = g.player(q);
    let mut best: Option<(Give, f64)> = None;
    for &c in &pl.hand {
        if skip_c.contains(&c) {
            continue;
        }
        let w = card_worth(g, q, c, false) / 10.0;
        if best.is_none_or(|b| w < b.1) {
            best = Some((Give::Card(c), w));
        }
    }
    for &m in &pl.perms {
        if skip_m.contains(&m) || g.perm(m).phased {
            continue;
        }
        let w = pval(g, m) + if g.perm(m).is_cmd { 6.0 } else { 0.0 };
        if best.is_none_or(|b| w < b.1) {
            best = Some((Give::Perm(m), w));
        }
    }
    best
}

/// one iteration of the process for q at `life` with the given items already given up: what q does
fn torment_pick(g: &Game, q: PlayerId, life: i32, skip_c: &[CardId], skip_m: &[PermId]) -> (Give, f64) {
    match cheapest(g, q, skip_c, skip_m) {
        Some((item, w)) if w < life_cost(life) || life <= 3 => (item, w),
        _ => (Give::Life, life_cost(life).min(30.0)),
    }
}

/// the whole process for q, played on paper: (life lost, the worth of what was given up)
fn torment_sim(g: &Game, q: PlayerId, x: i32) -> (i32, f64) {
    let (mut life, mut lost, mut worth) = (g.player(q).life, 0, 0.0);
    let (mut sc, mut sm): (Vec<CardId>, Vec<PermId>) = (vec![], vec![]);
    for _ in 0..x {
        if life <= 0 {
            break;
        }
        match torment_pick(g, q, life, &sc, &sm) {
            (Give::Card(c), w) => {
                sc.push(c);
                worth += w;
            }
            (Give::Perm(m), w) => {
                sm.push(m);
                worth += w;
            }
            (Give::Life, _) => {
                life -= 3;
                lost += 3;
            }
        }
    }
    (lost, worth)
}

/// {X}{B}{B}: repeat X times: each opponent loses 3 life unless they sacrifice a nonland permanent or discard a card
fn torment_resolve(g: &mut Game, p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    let x = ctx.x.max(0);
    crate::glog!(g, "    Torment of Hailfire (X = {x})");
    for _ in 0..x {
        for q in g.opps(p).collect::<Vec<_>>() {
            if !g.player(q).alive || g.over {
                continue;
            }
            let life = g.player(q).life;
            match torment_pick(g, q, life, &[], &[]) {
                (Give::Card(c), _) => {
                    crate::glog!(g, "      {} discards {}", g.player(q).name, g.db.get(c).name);
                    discard_cards(g, q, &[c])?;
                }
                (Give::Perm(m), _) => {
                    crate::glog!(g, "      {} sacrifices {}", g.player(q).name, g.perm(m).name);
                    die(g, m, "sac")?;
                }
                (Give::Life, _) => {
                    lose_life(g, q, 3, Some(p), "drain", None)?;
                }
            }
        }
        crate::engine::life::check_state(g)?;
        if g.over {
            break;
        }
    }
    Ok("gy")
}

fn late(g: &Game, p: PlayerId) -> bool {
    g.player(p).turns >= 8 || total_mana(g, p, false) >= 9
}

/// the AI's X for Torment of Hailfire: all the mana it has
fn torment_x(g: &Game, p: PlayerId, c: CardId) -> Option<i32> {
    let (gn, pips) = cost_of(g, p, c);
    if !can_pay(g, p, gn, &pips, false) {
        return None;
    }
    let x = total_mana(g, p, false) as i32 - gn as i32 - pips.chars().count() as i32;
    (x >= 1).then_some(x)
}

/// Torment of Hailfire as a Sauron main-phase play: a finisher (it kills an opponent) or a late-game strip of
/// every opponent's cards and permanents
fn torment_options(g: &mut Game, c: CardId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || g.active != Some(p) || g.player(p).key != "sauron" || !g.player(p).hand.contains(&c) {
        return Ok(vec![]);
    }
    let Some(x) = torment_x(g, p, c) else { return Ok(vec![]) };
    let (mut kills, mut loss, mut worth) = (0, 0, 0.0);
    for q in g.opps(p).collect::<Vec<_>>() {
        if g.player(q).life_locked {
            continue;
        }
        let (l, w) = torment_sim(g, q, x);
        if g.player(q).life - l <= 0 {
            kills += 1;
        }
        loss += l;
        worth += w;
    }
    if kills == 0 && (x < 4 || !late(g, p)) {
        return Ok(vec![]);
    }
    let u = 1.5 + 0.1 * loss as f64 + 0.3 * worth + 7.0 * kills as f64;
    Ok(vec![Opt {
        utility: u,
        label: format!("Torment of Hailfire (X = {x})"),
        act: Some(Action::Plan { f: torment_go, arg: c.0 as i64 }),
    }])
}

fn torment_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let c = CardId(arg as u16);
    if !g.player(p).hand.contains(&c) {
        return Ok(false);
    }
    let Some(x) = torment_x(g, p, c) else { return Ok(false) };
    let (gn, pips) = cost_of(g, p, c);
    if !can_pay(g, p, gn + x as u32, &pips, false) {
        return Ok(false);
    }
    pay(g, p, gn + x as u32, &pips, false)?;
    cast_card(g, p, c, "hand", Ctx { x, ..Ctx::default() })?;
    Ok(true)
}

// ================================================================== Dualcaster Mage
fn dualcaster_perm(g: &Game, p: PlayerId, cd: CardId) -> Option<PermId> {
    g.player(p).perms.iter().rev().copied().find(|&m| g.perm(m).cd == Some(cd) && g.perm(m).on_bf)
}

/// how much a copy of `spell` (cast by anyone) is worth to p
fn copy_worth(g: &Game, p: PlayerId, spell: CardId) -> f64 {
    let d = g.db.get(spell);
    let x = match &g.cur_cast {
        Some((cc, ctx, _)) if *cc == spell => ctx.x,
        _ => 0,
    };
    match &*d.name {
        "Exsanguinate" | TORMENT => {
            if g.cur_cast.as_ref().is_some_and(|c| c.0 == spell && g.player(p).key == "sauron") {
                x as f64 * 2.0
            } else {
                0.0
            }
        }
        _ => crate::impls::mine::spell_copy_value(g, p, spell),
    }
}

/// a spell was cast: flash Dualcaster Mage in from hand to copy it when the copy is worth {1}{R}{R}. `caster` is the
/// player who cast it (a copy is made for p either way).
fn dualcaster_copy(g: &mut Game, c: CardId, p: PlayerId, spell: CardId) -> Res {
    let sd = g.db.get(spell);
    if g.in_rtf || !g.player(p).hand.contains(&c) || !(sd.instant || sd.sorcery) || spell == c {
        return Ok(());
    }
    if copy_worth(g, p, spell) < 5.0 || !can_pay(g, p, 1, "RR", false) {
        return Ok(());
    }
    g.in_rtf = true;
    let r = dualcaster_copy_go(g, c, p, spell);
    g.in_rtf = false;
    r
}

fn dualcaster_copy_go(g: &mut Game, c: CardId, p: PlayerId, spell: CardId) -> Res {
    let ctx = g.cur_cast.as_ref().filter(|x| x.0 == spell).map(|x| x.1.clone());
    pay(g, p, 1, "RR", false)?;
    crate::glog!(g, "  {} flashes in Dualcaster Mage: copy {}", g.player(p).name, g.db.get(spell).name);
    if !cast_card(g, p, c, "hand", Ctx::default())? {
        return Ok(()); // countered
    }
    let Some(m) = dualcaster_perm(g, p, c) else { return Ok(()) };
    if trigger_window(g, p, Some(m), &format!("copy {}", g.db.get(spell).name), Some(5.0))? {
        copy_spell(g, p, spell, ctx.as_ref(), false)?;
    }
    Ok(())
}

// ================================================================== Sauron's cast priorities
/// ais.sauron_prio for the cards of this module (None: not one of them). The amass creatures and Dreadhorde Invasion are
/// early plays (rocks, at 80, still go first); Grishnákh waits for a creature worth taking; Dualcaster Mage is
/// held for a spell worth copying, and cast as a plain 2/2 only late.
pub fn sauron_extra_prio(g: &Game, p: PlayerId, c: CardId) -> Option<i32> {
    let pl = g.player(p);
    let early = pl.turns <= 4;
    Some(match &*g.db.get(c).name {
        INVASION => {
            if early {
                70
            } else {
                52
            }
        }
        "Lazotep Reaver" => {
            if early {
                67
            } else {
                46
            }
        }
        "Grim Initiate" => {
            if early {
                65
            } else {
                44
            }
        }
        GRISHNAKH => {
            // the Army it makes is 2 power (more on top of an existing one): a victim with power up to that
            let pow = army_of(g, p).map_or(0, |a| epow(g, a)) + 2;
            if grishnakh_victim(g, p, pow).is_some_and(|m| pval(g, m) >= 3.0) {
                64
            } else if early {
                56
            } else {
                44
            }
        }
        DUALCASTER => {
            if pl.turns < 6 {
                0
            } else {
                36
            }
        }
        _ => return None,
    })
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, GRISHNAKH)?.etb = Some(grishnakh_etb);
    r.card(db, INVASION)?.attack = Some(invasion_attack);
    let c = r.card(db, TORMENT)?;
    c.resolve = Some(torment_resolve);
    c.hand_options = Some(torment_options);
    let c = r.card(db, DUALCASTER)?;
    c.hand_cast = Some(dualcaster_copy);
    c.hand_opp_cast = Some(dualcaster_copy);
    Ok(())
}
