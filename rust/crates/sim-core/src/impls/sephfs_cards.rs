//! Cards of the Sephiroth FS deck (`sephfs`, mono-black wipe, drain and steal) that the compiled Oracle text and the
//! tags don't cover. Verdicts are in data/card_overrides.toml; each function's doc comment says how the card is
//! modeled and what is left out. Tests: tests/cards_sephfs.rs. The deck's plan is in `sephfs.rs`.
//!
//! - **Extinction Event**: choose odd or even (the AI picks the parity that costs it least), exile every creature of
//!   that mana value (tokens are mana value 0, even).
//! - **Blood on the Snow**, **Necromantic Selection**, **Living Death**: the three hand-written wipes. Blood's X is the
//!   snow mana spent: none of this deck's lands or mana rocks is snow, so X = 0 (its return clause finds no card).
//! - **Grave Betrayal**, **Dread Slaver**: take what dies. Betrayal returns the card at the next end step with a
//!   +1/+1 counter; Slaver returns what its combat damage killed at once.
//! - **Kaya's Ghostform**, **Rabid Attack**, **Pestilence**, **Grave Researcher // Reanimate**.
//!
//! The "black Zombie in addition" clauses (Betrayal, Slaver, Necromantic Selection) are not modeled: nothing in these
//! games reads the creature type of a creature that has already been returned.

use super::common::{AURA, AuraSpec};
use crate::cards::CardDb;
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, pay};
use crate::engine::values::{etgh, indestructible, no_damage, pval};
use crate::engine::zones::{Enter, Zone, die, draw, enter, exile_perm, leave, to_zone_card};
use crate::flow::Res;
use crate::hooks::{Action, Opt, Registry, Src};
use crate::ids::{CardId, PermId, PlayerId};
use crate::state::{Ctx, DataKey, Game, Val};
use crate::sym::Sym;

pub const KEY: &str = "sephfs";
pub const SEPH: &str = "Sephiroth, Fabled SOLDIER // Sephiroth, One-Winged Angel";

fn owner(g: &Game, m: PermId) -> PlayerId {
    g.perm(m).owner
}

fn live(g: &Game, m: PermId) -> bool {
    g.perm(m).on_bf && !g.perm(m).phased
}

fn stat(g: &mut Game, p: PlayerId, name: &'static str, n: i64) {
    g.player_mut(p).stat(name, n);
}

/// the mana value of a permanent (a token or a copy without a card: 0)
pub fn cmc_of(g: &Game, m: PermId) -> u32 {
    g.perm(m).cd.map_or(0, |c| g.db.get(c).cmc)
}

pub fn is_seph(g: &Game, m: PermId) -> bool {
    g.perm(m).cd.is_some_and(|c| &*g.db.get(c).name == SEPH)
}

pub fn flipped(g: &Game, m: PermId) -> bool {
    g.perm(m).data.truthy(DataKey::Flipped)
}

/// p's unflipped Sephiroth on the battlefield, if any
pub fn seph_unflipped(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p).perms.iter().copied().find(|&m| live(g, m) && is_seph(g, m) && !flipped(g, m) && g.perm(m).owner == p)
}

/// every creature on the battlefield with its controller
pub fn all_creatures(g: &Game) -> Vec<(PlayerId, PermId)> {
    g.players
        .iter()
        .filter(|q| q.alive)
        .flat_map(|q| q.perms.iter().map(move |&m| (q.id, m)))
        .filter(|&(_, m)| live(g, m) && g.is_creature(m))
        .collect()
}

/// destroy or sacrifice these creatures. An unflipped Sephiroth goes last: the wipe kills everything at once, so his
/// drain sees the others die (the engine kills one at a time).
fn kill_all(g: &mut Game, ms: Vec<PermId>, cause: Sym) -> Res {
    let (last, first): (Vec<PermId>, Vec<PermId>) = ms.into_iter().partition(|&m| is_seph(g, m) && !flipped(g, m));
    for m in first.into_iter().chain(last) {
        if live(g, m) {
            die(g, m, cause)?;
        }
    }
    Ok(())
}

/// what a creature card is worth to p as a steal (0: not a creature)
pub fn steal_value(g: &Game, c: CardId) -> f64 {
    let d = g.db.get(c);
    if !d.creature {
        return 0.0;
    }
    let mut v = 0.7 * d.pow as f64 + 0.35 * d.tgh as f64 + 0.5 * d.cmc as f64 + 0.8 * d.bomb as f64;
    if d.has_kw("flying") || d.has_kw("menace") || d.has_kw("trample") {
        v += 1.0;
    }
    if d.has_kw("deathtouch") || d.has_kw("lifelink") {
        v += 0.5;
    }
    v
}

/// the creature cards in any graveyard p could take, best first: (value, card, whose graveyard)
pub fn steal_targets(g: &Game, p: PlayerId) -> Vec<(f64, CardId, PlayerId)> {
    let mut v = vec![];
    for q in g.players.iter().filter(|q| q.alive || q.id == p) {
        for &c in &q.gy {
            let s = steal_value(g, c);
            if s >= 3.0 {
                v.push((s, c, q.id));
            }
        }
    }
    v.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    v
}

// ================================================================== Extinction Event
/// the parity (0 even, 1 odd) the caster exiles, with what the opponents and the caster lose by it. Our unflipped
/// Sephiroth counts double: a flipped one is only a body.
pub fn parity_choice(g: &Game, p: PlayerId) -> (u32, f64, f64) {
    let mut best = (0u32, f64::NEG_INFINITY, 0.0, 0.0);
    for par in 0..2u32 {
        let (mut ol, mut ml) = (0.0, 0.0);
        for (q, m) in all_creatures(g) {
            if cmc_of(g, m) % 2 != par {
                continue;
            }
            let mut v = pval(g, m);
            if q == p {
                if is_seph(g, m) && !flipped(g, m) {
                    v += 4.0;
                }
                ml += v;
            } else {
                ol += v;
            }
        }
        let score = ol - 1.2 * ml;
        if score > best.1 {
            best = (par, score, ol, ml);
        }
    }
    (best.0, best.2, best.3)
}

fn event_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let (par, _, _) = parity_choice(g, p);
    crate::glog!(g, "    Extinction Event: {} creatures are exiled", if par == 0 { "even" } else { "odd" });
    for (_, m) in all_creatures(g) {
        if live(g, m) && cmc_of(g, m) % 2 == par {
            exile_perm(g, m)?;
        }
    }
    stat(g, p, "sf_wipes", 1);
    Ok("gy")
}

// ================================================================== the destroy wipes
/// destroy every creature; the creature cards that reached a graveyard by it, with their owners
fn destroy_all(g: &mut Game) -> Res<Vec<(CardId, PlayerId)>> {
    let ms: Vec<PermId> = all_creatures(g).into_iter().map(|x| x.1).collect();
    let cards: Vec<(CardId, PlayerId)> =
        ms.iter().filter(|&&m| !g.perm(m).token).filter_map(|&m| g.perm(m).cd.map(|c| (c, g.perm(m).orig))).collect();
    kill_all(g, ms, "destroy")?;
    Ok(cards.into_iter().filter(|&(c, o)| g.player(o).gy.contains(&c)).collect())
}

/// put the card c from owner o's graveyard onto the battlefield under p's control
fn take_from_gy(g: &mut Game, p: PlayerId, c: CardId, o: PlayerId) -> Res<Option<PermId>> {
    let Some(i) = g.player(o).gy.iter().position(|&x| x == c) else { return Ok(None) };
    g.player_mut(o).gy.remove(i);
    let how = Enter { orig: (o != p).then_some(o), ..Enter::default() };
    Ok(Some(enter(g, p, c, how)?))
}

/// **Necromantic Selection**: destroy all creatures, then return a creature card put into a graveyard this way under
/// your control (the best one), and exile the spell.
fn necro_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let died = destroy_all(g)?;
    stat(g, p, "sf_wipes", 1);
    if let Some(&(c, o)) = died.iter().max_by(|a, b| steal_value(g, a.0).total_cmp(&steal_value(g, b.0))) {
        if let Some(m) = take_from_gy(g, p, c, o)? {
            crate::glog!(g, "    Necromantic Selection returns {} under {}'s control", g.perm(m).name, g.player(p).name);
            stat(g, p, "sf_steals", 1);
        }
    }
    Ok("exile")
}

/// **Blood on the Snow**: destroy all creatures (the AI never takes the planeswalker mode), then return a creature or
/// planeswalker card with mana value X or less from your graveyard, X being the snow mana spent: none here, so X = 0.
fn blood_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    destroy_all(g)?;
    stat(g, p, "sf_wipes", 1);
    let own: Vec<CardId> = g
        .player(p)
        .gy
        .iter()
        .copied()
        .filter(|&c| {
            let d = g.db.get(c);
            (d.creature || d.types.has(crate::cards::Types::PLANESWALKER)) && d.cmc == 0
        })
        .collect();
    if let Some(&c) = own.first() {
        take_from_gy(g, p, c, p)?;
    }
    Ok("gy")
}

/// **Living Death**: each player exiles the creature cards from their graveyard, then sacrifices all their
/// creatures, then puts the exiled cards onto the battlefield (under their owner's control).
fn living_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let mut exiled: Vec<(PlayerId, CardId)> = vec![];
    for i in 0..g.players.len() {
        let q = PlayerId(i as u8);
        if !g.player(q).alive {
            continue;
        }
        let (cre, rest): (Vec<CardId>, Vec<CardId>) = std::mem::take(&mut g.player_mut(q).gy)
            .into_iter()
            .partition(|&c| g.db.get(c).creature);
        g.player_mut(q).gy = rest;
        exiled.extend(cre.into_iter().map(|c| (q, c)));
    }
    let ms: Vec<PermId> = all_creatures(g).into_iter().map(|x| x.1).collect();
    kill_all(g, ms, "sac")?;
    for (q, c) in exiled {
        if g.player(q).alive {
            enter(g, q, c, Enter::default())?;
        }
    }
    stat(g, p, "sf_wipes", 1);
    Ok("gy")
}

// ================================================================== Grave Betrayal
/// **Grave Betrayal**: whenever a creature you don't control dies, return it under your control with a +1/+1 counter
/// at the beginning of the next end step. The death is noted when the trigger resolves (the card is in its owner's
/// graveyard by then); a token has no card to return. If the enchantment leaves before the end step, the creature
/// stays where it is (the delayed effect is kept on the enchantment).
fn betrayal_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let p = owner(g, src);
    let x = g.perm(m);
    if x.owner == p || x.token || !g.is_creature(m) || !g.perm(src).on_bf {
        return Ok(());
    }
    let (Some(c), o) = (x.cd, x.orig) else { return Ok(()) };
    if !g.player(o).gy.contains(&c) {
        return Ok(()); // a commander in the command zone, or exiled
    }
    let mut v = match g.perm(src).data.get(DataKey::Betrayed) {
        Some(Val::List(v)) => v.clone(),
        _ => vec![],
    };
    v.push(Val::Card(c));
    v.push(Val::Player(o));
    g.perm_mut(src).data.set(DataKey::Betrayed, Val::List(v));
    Ok(())
}

fn betrayal_end(g: &mut Game, src: Src, _active: PlayerId) -> Res {
    let p = owner(g, src);
    let Some(Val::List(v)) = g.perm(src).data.get(DataKey::Betrayed).cloned() else { return Ok(()) };
    g.perm_mut(src).data.set(DataKey::Betrayed, Val::None);
    for pair in v.chunks(2) {
        let (Some(Val::Card(c)), Some(Val::Player(o))) = (pair.first(), pair.get(1)) else { continue };
        if !g.player(p).alive {
            return Ok(());
        }
        let Some(i) = g.player(*o).gy.iter().position(|&x| x == *c) else { continue };
        g.player_mut(*o).gy.remove(i);
        let how = Enter { orig: (*o != p).then_some(*o), plus: 1, ..Enter::default() };
        let n = enter(g, p, *c, how)?;
        crate::glog!(g, "    Grave Betrayal returns {} under {}'s control with a +1/+1 counter", g.perm(n).name, g.player(p).name);
        stat(g, p, "sf_steals", 1);
        stat(g, p, "sf_betrayals", 1);
    }
    Ok(())
}

// ================================================================== Dread Slaver
/// **Dread Slaver** (3/5): whenever a creature dealt damage by it this turn dies, return it under your control. The
/// engine only kills what the damage kills, so combat marks the victim (`DataKey::Slaver`, set in combat.rs) and this
/// returns it when it dies.
fn slaver_dies(g: &mut Game, src: Src, m: PermId, _cause: Sym) -> Res {
    let p = owner(g, src);
    if g.perm(m).data.get(DataKey::Slaver) != Some(&Val::Player(p)) || g.perm(m).token || !g.perm(src).on_bf {
        return Ok(());
    }
    g.perm_mut(m).data.set(DataKey::Slaver, Val::None);
    let (Some(c), o) = (g.perm(m).cd, g.perm(m).orig) else { return Ok(()) };
    if let Some(n) = take_from_gy(g, p, c, o)? {
        crate::glog!(g, "    Dread Slaver returns {} under {}'s control", g.perm(n).name, g.player(p).name);
        stat(g, p, "sf_steals", 1);
        stat(g, p, "sf_slaver", 1);
    }
    Ok(())
}

// ================================================================== Rabid Attack
/// **Rabid Attack**: until end of turn, your creatures (the AI targets all of them) get +1/+0 and "when this creature
/// dies, draw a card". The draw is run from engine/zones.rs `die_triggers` (the `Rabid` mark).
fn rabid_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    let st = g.turn_stamp();
    for (q, m) in all_creatures(g) {
        if q == p {
            g.perm_mut(m).eot_pt.0 += 1;
            g.perm_mut(m).data.set(DataKey::Rabid, Val::Stamp(st));
        }
    }
    Ok("gy")
}

/// a creature marked by Rabid Attack died: its controller draws a card
pub fn rabid_dies(g: &mut Game, p: PlayerId) -> Res {
    if g.player(p).alive {
        stat(g, p, "sf_rabid_draws", 1);
        draw(g, p, 1, false)?;
    }
    Ok(())
}

// ================================================================== Kaya's Ghostform
/// **Kaya's Ghostform** (Aura, enchant creature or planeswalker you control): when the enchanted permanent dies or is
/// put into exile, return that card to the battlefield under your control. Only creatures are enchanted here. When the
/// host leaves, `host_dies` marks it (`DataKey::Ghostform`, the Aura's controller) and `die_rest` / `exile_perm` return
/// the card instead of putting it into the graveyard or exile. A commander returns from the command zone's place.
fn ghostform_mark(g: &mut Game, p: PlayerId, _a: PermId, host: PermId) -> Res {
    g.perm_mut(host).data.set(DataKey::Ghostform, Val::Player(p));
    Ok(())
}

static GHOSTFORM: AuraSpec = AuraSpec { host_dies: Some(ghostform_mark), host_exiled: true, ..AURA };

/// the host m, marked by a Ghostform, left: its card returns under p's control (true when it did)
pub fn ghostform_return(g: &mut Game, m: PermId, p: PlayerId) -> Res<bool> {
    g.perm_mut(m).data.set(DataKey::Ghostform, Val::None);
    let x = g.perm(m).clone();
    let Some(c) = x.cd else { return Ok(false) };
    if x.token || !g.player(p).alive {
        return Ok(false);
    }
    let cmd = g.player(x.orig).is_commander(c);
    let how = Enter { orig: (x.orig != p).then_some(x.orig), ..Enter::default() };
    let n = enter(g, p, c, how)?;
    if cmd {
        g.player_mut(x.orig).set_cmd_zone(c, false);
        g.perm_mut(n).is_cmd = true;
    }
    crate::glog!(g, "    Kaya's Ghostform returns {} to the battlefield", g.perm(n).name);
    stat(g, p, "sf_ghostform", 1);
    Ok(true)
}

// ================================================================== Pestilence
/// the damage-proof or indestructible creatures Pestilence cannot kill
fn pest_kills(g: &Game, m: PermId, i: i32) -> bool {
    g.is_creature(m) && live(g, m) && etgh(g, m) <= i && !no_damage(g, m) && !indestructible(g, m)
}

/// what n activations of Pestilence are worth to p: (value, kills an opponent)
pub fn pest_value(g: &Game, p: PlayerId, n: i32) -> (f64, bool) {
    let dv = super::common::death_value(g, p, None);
    let (mut v, mut lethal) = (0.0, false);
    for (q, m) in all_creatures(g) {
        if !pest_kills(g, m, n) {
            continue;
        }
        let pv = pval(g, m);
        if q == p {
            // our unflipped Sephiroth must not die; anything else is a loss (but its death still drains)
            v -= if is_seph(g, m) && !flipped(g, m) { 12.0 } else { 1.2 * pv };
        } else {
            v += pv;
        }
        v += dv;
    }
    for q in g.opps(p) {
        if g.player(q).alive {
            v += 0.5 * n as f64;
            if g.player(q).life <= n {
                lethal = true;
            }
        }
    }
    v -= 0.5 * n as f64; // our own life
    if g.player(p).life <= n {
        v -= 100.0;
    }
    if lethal {
        v += 20.0;
    }
    (v, lethal)
}

fn pest_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || p != owner(g, src) || !live(g, src) || crate::human::is_human(g, p) {
        return Ok(vec![]);
    }
    let mut best: Option<(f64, i32)> = None;
    for n in 1..=8 {
        if !can_pay(g, p, 0, &"B".repeat(n as usize), false) {
            break;
        }
        let (v, _) = pest_value(g, p, n);
        if best.is_none_or(|b| v > b.0) {
            best = Some((v, n));
        }
    }
    let Some((v, n)) = best else { return Ok(vec![]) };
    if v < 1.5 {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: (v / 2.0).clamp(1.0, 9.0),
        label: format!("Pestilence x{n}"),
        act: Some(Action::Ability { src, f: pest_go, arg: n as i64 }),
    }])
}

/// activate Pestilence n times: each time {B}, 1 damage to each creature and each player. Damage stays on a creature
/// for the turn, so activation i kills what has toughness i or less.
fn pest_go(g: &mut Game, src: PermId, p: PlayerId, n: i64) -> Res<bool> {
    if !live(g, src) {
        return Ok(false);
    }
    let mut done = 0;
    for i in 1..=n as i32 {
        if !pay(g, p, 0, "B", false)? {
            break;
        }
        done += 1;
        crate::glog!(g, "  {} activates Pestilence ({} damage so far)", g.player(p).name, i);
        for q in g.players.iter().filter(|q| q.alive).map(|q| q.id).collect::<Vec<_>>() {
            lose_life(g, q, 1, Some(p), "burn", None)?;
        }
        let dead: Vec<PermId> = all_creatures(g).into_iter().map(|x| x.1).filter(|&m| pest_kills(g, m, i)).collect();
        kill_all(g, dead, "destroy")?;
        if g.over {
            break;
        }
    }
    stat(g, p, "sf_pestilence", done);
    Ok(done > 0)
}

/// at the beginning of the end step, if no creatures are on the battlefield, sacrifice Pestilence (any player's end
/// step)
fn pest_end(g: &mut Game, src: Src, _active: PlayerId) -> Res {
    if live(g, src) && all_creatures(g).is_empty() {
        crate::glog!(g, "    Pestilence is sacrificed (no creatures)");
        leave(g, src)?;
        to_zone_card(g, src, Zone::Gy);
    }
    Ok(())
}

// ================================================================== Grave Researcher // Reanimate
const RESEARCHER: &str = "Grave Researcher // Reanimate";

/// at the beginning of your upkeep, surveil 1, then with three or more creature cards in your graveyard it becomes
/// prepared
fn researcher_upkeep(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != owner(g, src) || !live(g, src) {
        return Ok(());
    }
    super::kefka::surveil(g, p, 1)?;
    if g.player(p).gy.iter().filter(|&&c| g.db.get(c).creature).count() >= 3 {
        g.perm_mut(src).data.set(DataKey::Prepared, Val::Bool(true));
    }
    Ok(())
}

fn researcher_options(g: &mut Game, src: Src, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post.is_none() || p != owner(g, src) || g.active != Some(p) || !live(g, src) || !g.perm(src).data.truthy(DataKey::Prepared) {
        return Ok(vec![]);
    }
    let life = g.player(p).life;
    let Some(&(v, _, _)) = steal_targets(g, p).iter().find(|&&(_, c, _)| life - g.db.get(c).cmc as i32 >= 8) else {
        return Ok(vec![]);
    };
    if !can_pay(g, p, 0, "B", false) {
        return Ok(vec![]);
    }
    Ok(vec![Opt {
        utility: v * 0.9,
        label: "Reanimate (Grave Researcher, prepared copy)".into(),
        act: Some(Action::Ability { src, f: researcher_go, arg: 0 }),
    }])
}

fn researcher_go(g: &mut Game, src: PermId, p: PlayerId, _arg: i64) -> Res<bool> {
    let life = g.player(p).life;
    let Some(&(_, c, o)) = steal_targets(g, p).iter().find(|&&(_, c, _)| life - g.db.get(c).cmc as i32 >= 8) else {
        return Ok(false);
    };
    if !g.perm(src).data.truthy(DataKey::Prepared) || !pay(g, p, 0, "B", false)? {
        return Ok(false);
    }
    g.perm_mut(src).data.set(DataKey::Prepared, Val::Bool(false));
    super::mine::cast_copy_with(g, p, None, Some("Reanimate"), true, 6.0, &mut |g| {
        if g.player(o).gy.contains(&c) {
            let mv = g.db.get(c).cmc as i32;
            if let Some(m) = take_from_gy(g, p, c, o)? {
                crate::glog!(g, "    Reanimate (prepared copy) takes {}", g.perm(m).name);
                lose_life(g, p, mv, Some(p), "life", None)?;
                stat(g, p, "sf_steals", 1);
                stat(g, p, "sf_reanimate", 1);
            }
        }
        Ok(())
    })?;
    Ok(true)
}

// ================================================================== registration
pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "Extinction Event")?.resolve = Some(event_resolve);
    r.card(db, "Necromantic Selection")?.resolve = Some(necro_resolve);
    r.card(db, "Blood on the Snow")?.resolve = Some(blood_resolve);
    r.card(db, "Living Death")?.resolve = Some(living_resolve);
    r.card(db, "Rabid Attack")?.resolve = Some(rabid_resolve);
    r.card(db, "Rewrite Regrets")?.resolve = Some(regrets_resolve);
    r.card(db, "Presumed Dead")?.resolve = Some(presumed_resolve);
    let c = r.card(db, BRAIDS)?;
    c.etb = Some(braids_etb);
    c.upkeep = Some(braids_lore);
    let c = r.card(db, "Grave Betrayal")?;
    c.dies = Some(betrayal_dies);
    c.end_step = Some(betrayal_end);
    r.card(db, "Dread Slaver")?.dies = Some(slaver_dies);
    super::common::aura(r, db, "Kaya's Ghostform", &GHOSTFORM)?;
    let c = r.card(db, "Pestilence")?;
    c.options = Some(pest_options);
    c.end_step = Some(pest_end);
    let c = r.card(db, RESEARCHER)?;
    c.upkeep = Some(researcher_upkeep);
    c.options = Some(researcher_options);
    Ok(())
}

/// combat: `killer` dealt the damage that kills `victim`; a Dread Slaver marks it for its controller
pub fn slaver_mark(g: &mut Game, killer: PermId, victim: PermId) {
    if g.perm(killer).cd.is_some_and(|c| &*g.db.get(c).name == "Dread Slaver") {
        let p = owner(g, killer);
        g.perm_mut(victim).data.set(DataKey::Slaver, Val::Player(p));
    }
}

// ================================================================== Rewrite Regrets
/// the best card p's graveyard gives back to Rewrite Regrets: a creature (mana value 6 or less), else a planeswalker
pub fn regrets_target(g: &Game, p: PlayerId) -> Option<CardId> {
    let gy = &g.player(p).gy;
    let best = gy
        .iter()
        .copied()
        .filter(|&c| g.db.get(c).creature && g.db.get(c).cmc <= 6)
        .max_by(|&a, &b| steal_value(g, a).total_cmp(&steal_value(g, b)));
    best.or_else(|| {
        gy.iter().copied().find(|&c| {
            let d = g.db.get(c);
            d.types.has(crate::cards::Types::PLANESWALKER) && d.cmc <= 6
        })
    })
}

/// **Rewrite Regrets**: return the best creature (else planeswalker) card with mana value 6 or less from your
/// graveyard to the battlefield, then Empower Jace 2 (the Jace token of the Alela deck: +2 loyalty, created first
/// when you have none; its -1 and -3 are not used by this deck's plan, so it is only a body that soaks attacks).
fn regrets_resolve(g: &mut Game, p: PlayerId, _c: CardId, _ctx: &Ctx) -> Res<Sym> {
    if let Some(c) = regrets_target(g, p)
        && let Some(m) = take_from_gy(g, p, c, p)?
    {
        crate::glog!(g, "    Rewrite Regrets returns {}", g.perm(m).name);
        stat(g, p, "sf_regrets", 1);
    }
    if g.db.id(super::alela::JACE).is_some() {
        super::alela::empower(g, p, 2);
    }
    Ok("gy")
}

// ================================================================== Presumed Dead
/// **Presumed Dead**: until end of turn the target creature gets +2/+0 and "when this creature dies, return it to the
/// battlefield under its owner's control and suspect it" (`ret_dies_suspect`, run by `sheoldred::returns_when_dies`
/// from `zones::die_rest`). The target comes in `ctx.target`.
fn presumed_resolve(g: &mut Game, _p: PlayerId, _c: CardId, ctx: &Ctx) -> Res<Sym> {
    if let Some(t) = ctx.target.filter(|&t| live(g, t) && g.is_creature(t)) {
        g.perm_mut(t).eot_pt.0 += 2;
        super::sheoldred::grant_eot(g, t, "ret_dies_suspect");
        crate::glog!(g, "    Presumed Dead on {}", g.perm(t).name);
    }
    Ok("gy")
}

const PRESUMED: &str = "Presumed Dead";

fn cast_presumed(g: &mut Game, p: PlayerId, t: PermId) -> Res<bool> {
    let Some(c) = g.player(p).hand.iter().copied().find(|&c| &*g.db.get(c).name == PRESUMED) else { return Ok(false) };
    if !crate::engine::cast::castable(g, p, c, "hand") || !can_pay(g, p, 1, "B", false) || !pay(g, p, 1, "B", false)? {
        return Ok(false);
    }
    crate::engine::cast::cast_card(g, p, c, "hand", Ctx { target: Some(t), ..Ctx::default() })?;
    stat(g, p, "sf_presumed", 1);
    Ok(true)
}

/// in response to destroy / damage removal (or a wipe) aimed at p's creature m: Presumed Dead when it is Sephiroth
/// or worth 2 or more. The removal still resolves; the creature comes back suspected (menace, can't block).
pub fn presumed_protect(g: &mut Game, p: PlayerId, m: PermId, kind: Sym) -> Res<bool> {
    if !(kind == "destroy" || kind.starts_with("dmg") || kind.starts_with("shrink") || kind == "sac")
        || !live(g, m)
        || !g.is_creature(m)
        || g.perm(m).token
        || g.perm(m).eot_kw.contains(&"ret_dies_suspect")
        || (!is_seph(g, m) && pval(g, m) < 2.0)
    {
        return Ok(false);
    }
    cast_presumed(g, p, m)
}

/// the main-phase play: Presumed Dead before combat on the creature that attacks best (menace, +2 power, and it
/// survives a block), when an opposing creature could kill it. Plan arg: the permanent.
fn presumed_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let t = PermId(arg as u32);
    if !live(g, t) || !g.is_creature(t) {
        return Ok(false);
    }
    cast_presumed(g, p, t)
}

pub fn presumed_option(g: &Game, p: PlayerId) -> Option<Opt> {
    let c = g.player(p).hand.iter().copied().find(|&c| &*g.db.get(c).name == PRESUMED)?;
    if g.active != Some(p) || !crate::engine::cast::castable(g, p, c, "hand") || !can_pay(g, p, 1, "B", false) {
        return None;
    }
    if !g.opps(p).any(|q| g.player(q).perms.iter().any(|&m| live(g, m) && g.is_creature(m))) {
        return None; // nothing blocks: no need
    }
    let t = g
        .player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| {
            live(g, m) && g.is_creature(m) && !g.perm(m).tapped && !g.perm(m).sick && !is_seph(g, m) && !g.perm(m).token
        })
        .max_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b)))?;
    (pval(g, t) >= 2.5).then(|| Opt {
        utility: 4.0 + pval(g, t) * 0.3,
        label: format!("Presumed Dead on {}", g.perm(t).name),
        act: Some(Action::Plan { f: presumed_go, arg: t.0 as i64 }),
    })
}

// ================================================================== Braids's Frightful Return
pub const BRAIDS: &str = "Braids's Frightful Return";

/// a creature p would sacrifice to chapter I: a token, else a creature worth under 1.8 that is not the commander
fn braids_fodder(g: &Game, p: PlayerId) -> Option<PermId> {
    g.player(p)
        .perms
        .iter()
        .copied()
        .filter(|&m| live(g, m) && g.is_creature(m) && !g.perm(m).is_cmd && !is_seph(g, m) && pval(g, m) < 1.8)
        .min_by(|&a, &b| pval(g, a).total_cmp(&pval(g, b)))
}

/// the chapter read ahead starts on: II when the graveyard holds a creature worth returning, I with spare fodder and
/// cards in an opponent's hand, otherwise III (which sacrifices the Saga at once)
fn braids_start(g: &Game, p: PlayerId) -> i64 {
    if g.player(p).gy.iter().any(|&c| steal_value(g, c) >= 3.0) {
        2
    } else if braids_fodder(g, p).is_some() && g.opps(p).any(|q| !g.player(q).hand.is_empty()) {
        1
    } else {
        3
    }
}

/// one chapter of Braids's Frightful Return (the trigger window is the chapter's commit point)
fn braids_chapter(g: &mut Game, src: PermId, p: PlayerId, n: i64) -> Res {
    match n {
        1 => {
            // you may sacrifice a creature; if you do, each opponent discards a card
            if let Some(f) = braids_fodder(g, p)
                && crate::engine::stack::trigger_window(g, p, Some(src), "chapter I: sacrifice, each opponent discards", Some(3.0))?
                && live(g, f)
            {
                die(g, f, "sac")?;
                for q in g.opps(p).collect::<Vec<_>>() {
                    crate::engine::cast::discard_worst(g, q, 1)?;
                }
            }
        }
        2 => {
            let best = g
                .player(p)
                .gy
                .iter()
                .copied()
                .filter(|&c| g.db.get(c).creature)
                .max_by(|&a, &b| steal_value(g, a).total_cmp(&steal_value(g, b)));
            if let Some(c) = best
                && crate::engine::stack::trigger_window(g, p, Some(src), "chapter II: creature card to hand", Some(3.0))?
                && let Some(i) = g.player(p).gy.iter().position(|&x| x == c)
            {
                g.player_mut(p).gy.remove(i);
                g.player_mut(p).hand.push(c);
                crate::glog!(g, "    Braids's Frightful Return returns {} to hand", g.db.get(c).name);
                stat(g, p, "sf_braids_returns", 1);
            }
        }
        _ => {
            // target opponent (the one with the most nonland, nontoken permanents) may sacrifice one; if not, loses 2 and you draw
            let spare = |g: &Game, q: PlayerId| -> Option<PermId> {
                g.player(q)
                    .perms
                    .iter()
                    .copied()
                    .filter(|&m| live(g, m) && !g.perm(m).token && g.perm(m).cd.is_some_and(|c| !g.db.get(c).land))
                    .min_by_key(|&m| cmc_of(g, m))
            };
            let count = |g: &Game, q: PlayerId| g.player(q).perms.iter().filter(|&&m| live(g, m) && !g.perm(m).token).count();
            let Some(q) = g.opps(p).filter(|&q| g.player(q).alive).max_by_key(|&q| count(g, q)) else { return Ok(()) };
            if !crate::engine::stack::trigger_window(g, p, Some(src), "chapter III: sacrifice or lose 2 and they draw", Some(3.0))? {
                return Ok(());
            }
            if crate::human::is_human(g, q) {
                // their choice of any nonland, nontoken permanent; lose 2 and you draw only if they have none
                let all: Vec<PermId> = g
                    .player(q)
                    .perms
                    .iter()
                    .copied()
                    .filter(|&m| live(g, m) && !g.perm(m).token && g.perm(m).cd.is_some_and(|c| !g.db.get(c).land))
                    .collect();
                if !all.is_empty() {
                    crate::human::choices::sacrifice_permanent(
                        g,
                        q,
                        &all,
                        "Braids's Frightful Return: sacrifice a nonland, nontoken permanent",
                    )?;
                    return Ok(());
                }
                lose_life(g, q, 2, Some(p), "other", None)?;
                draw(g, p, 1, false)?;
                stat(g, p, "sf_braids_drains", 1);
                return Ok(());
            }
            // the opponent gives up a mana-value 0 or 1 permanent, not a better one
            match spare(g, q).filter(|&m| cmc_of(g, m) <= 1) {
                Some(m) => die(g, m, "sac")?,
                None => {
                    lose_life(g, q, 2, Some(p), "other", None)?;
                    draw(g, p, 1, false)?;
                    stat(g, p, "sf_braids_drains", 1);
                }
            }
        }
    }
    Ok(())
}

/// **Braids's Frightful Return** (read ahead Saga): enters on the chapter `braids_start` picks, with that many lore
/// counters; skipped chapters don't trigger. Sacrificed after III. Lore counters are added at the upkeep (`braids_lore`)
/// rather than after the draw step, and proliferate does not add one (a rare case in this deck).
fn braids_etb(g: &mut Game, src: Src, _p: PlayerId, m: PermId) -> Res {
    if m != src {
        return Ok(());
    }
    let o = g.perm(src).owner;
    let n = braids_start(g, o);
    g.perm_mut(src).data.set(DataKey::Lore, Val::Int(n));
    crate::glog!(g, "    {} reads ahead to chapter {n}", g.perm(src).name);
    braids_chapter(g, src, o, n)?;
    if n >= 3 && live(g, src) {
        die(g, src, "sac")?;
    }
    Ok(())
}

fn braids_lore(g: &mut Game, src: Src, p: PlayerId) -> Res {
    if p != g.perm(src).owner || !live(g, src) || g.perm(src).data.get(DataKey::Lore).is_none() {
        return Ok(());
    }
    let n = g.perm(src).data.int(DataKey::Lore) + 1;
    g.perm_mut(src).data.set(DataKey::Lore, Val::Int(n));
    braids_chapter(g, src, p, n)?;
    if n >= 3 && live(g, src) {
        die(g, src, "sac")?;
    }
    Ok(())
}
