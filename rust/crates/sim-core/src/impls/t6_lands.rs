//! Tier 6 lands batch (rust/agents/t6-batches.md, `lands`): utility lands of the Tier 6 decks. Cards only: what
//! needs a new engine feature is listed in rust/agents/engine-needs-lands.md.
//!
//! Lands are not permanents: their abilities are offered through the `land_options` / `hand_options` slots, their
//! mana through `land_cols`, `dyn_mana_land` and `on_tap_land` (see impls/lands.rs).

use super::lands::{can_pay_without, pay_without};
use crate::cards::{CardDb, Colors, Types};
use crate::engine::stack::ability_window_card;
use crate::engine::tutors::card_worth;
use crate::engine::zones::max_by;
use crate::hooks::{Action, Opt};
use crate::ids::{CardId, PermId};
use crate::tag::Tag;
use crate::engine::life::lose_life;
use crate::engine::mana::{can_pay, pay, total_mana};
use crate::engine::removal::apply_removal;
use crate::engine::values::{etgh, pval, untargetable};
use crate::flow::Res;
use crate::hooks::Registry;
use crate::ids::{LandId, PlayerId};
use crate::state::{DataKey, Game};

fn name_of(g: &Game, l: LandId) -> &str {
    &g.db.get(g.land(l).cd).name
}

// ================================================================== City of Brass
/// **City of Brass**: {T}: any colour (tag c=A); whenever it becomes tapped it deals 1 damage to you.
fn brass_tap(g: &mut Game, p: PlayerId, _l: LandId, _n: u32) -> Res {
    lose_life(g, p, 1, Some(p), "other", Some(true))
}

// ================================================================== City of Traitors
/// **City of Traitors**: {T}: {C}{C} (tag amt=2). When you play another land, sacrifice it. The engine calls
/// `land_etb` for the land just played, so every land without code of its own gets this check (`traitors_check`).
fn traitors_check(g: &mut Game, p: PlayerId, l: LandId) -> Res {
    let mine: Vec<LandId> = g
        .player(p)
        .lands
        .iter()
        .copied()
        .filter(|&x| x != l && name_of(g, x) == "City of Traitors")
        .collect();
    for c in mine {
        if g.player(p).lands.contains(&c) {
            super::lands::sac_land(g, p, c)?;
            crate::glog!(g, "    {} sacrifices City of Traitors (another land was played)", g.player(p).name);
        }
    }
    Ok(())
}

// ================================================================== Gemstone Caverns
/// **Gemstone Caverns**: {T}: Add {C}; with a luck counter on it, one mana of any colour instead. The opening-hand
/// clause (begin the game with it on the battlefield with a luck counter, exiling a card) needs the engine: see
/// engine-needs-lands.md. The luck counter is `DataKey::Counters` (an Int) on the land.
fn caverns_cols(g: &Game, p: PlayerId, l: LandId) -> Colors {
    if g.land(l).data.int(DataKey::Counters) > 0 { g.player(p).ident } else { Colors::NONE }
}

// ================================================================== the "{1},{T}: any colour" lands
/// **The Mycosynth Gardens, Talon Gates of Madara, Abstergo Entertainment**: {T}: Add {C}. {1},{T}: Add one mana of any
/// colour. A filter for one mana: the land makes any colour only when another untapped land (not one of these) pays
/// the {1}; the count of mana stays right (the land is one mana, the payer another, as if the payer made colourless
/// mana and the land its own colourless mana). With nothing else to pay, colourless only
const ANY_FILTER: [&str; 3] = ["The Mycosynth Gardens", "Talon Gates of Madara", "Abstergo Entertainment"];

fn any_filter_cols(g: &Game, p: PlayerId, l: LandId) -> Colors {
    let payer = g.player(p).lands.iter().any(|&x| x != l && !g.land(x).tapped && !ANY_FILTER.contains(&name_of(g, x)));
    if payer { g.player(p).ident } else { Colors::NONE }
}

// ================================================================== Abstergo Entertainment
/// **Abstergo Entertainment**: {3},{T}, exile it: return up to one target historic card (artifact, legendary, Saga)
/// from your graveyard to your hand, then exile all graveyards. The AI does it with 3 spare mana when a historic card
/// in its graveyard is worth at least 30 as a card in hand
fn historic(g: &Game, c: CardId) -> bool {
    let d = g.db.get(c);
    d.types.has(Types::ARTIFACT) || d.tag(Tag::Leg) || d.has_subtype("saga")
}

fn abstergo_target(g: &Game, p: PlayerId) -> Option<CardId> {
    let cs: Vec<CardId> = g.player(p).gy.iter().copied().filter(|&c| historic(g, c)).collect();
    max_by(&cs, |c| card_worth(g, p, c, false)).filter(|&c| card_worth(g, p, c, false) >= 30.0)
}

fn abstergo(g: &mut Game, l: LandId, p: PlayerId, post: Option<bool>) -> Res<Vec<Opt>> {
    if post == Some(false) || g.land(l).tapped || !can_pay_without(g, p, l, 3, "") {
        return Ok(vec![]);
    }
    let Some(t) = abstergo_target(g, p) else { return Ok(vec![]) };
    let u = card_worth(g, p, t, false) / 30.0;
    Ok(vec![Opt {
        utility: u,
        label: "Abstergo Entertainment".into(),
        act: Some(Action::Plan { f: abstergo_go, arg: l.0 as i64 }),
    }])
}

fn abstergo_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let l = LandId(arg as u32);
    if !g.player(p).lands.contains(&l) || g.land(l).tapped || !pay_without(g, p, l, 3, "")? {
        return Ok(false);
    }
    let cd = g.land(l).cd;
    crate::engine::turn::remove_land(g, p, l); // {T} and exile it as costs
    g.player_mut(p).exile.push(cd);
    crate::glog!(g, "  {} exiles Abstergo Entertainment", g.player(p).name);
    let t = abstergo_target(g, p);
    if ability_window_card(g, p, cd, "return a historic card, exile all graveyards", None, None)? {
        if let Some(c) = t.filter(|c| g.player(p).gy.contains(c)) {
            let i = g.player(p).gy.iter().position(|&x| x == c).unwrap();
            g.player_mut(p).gy.remove(i);
            g.player_mut(p).hand.push(c);
            crate::glog!(g, "    {} returns {} to hand", g.player(p).name, g.db.get(c).name);
        }
        for i in 0..g.players.len() {
            super::common::exile_gy(g, PlayerId(i as u8), Some("Abstergo Entertainment"));
        }
    }
    Ok(true)
}

// ================================================================== Eiganjo, Seat of the Empire (and Razorgrass Ambush)
/// The damage-to-an-attacker-or-blocker answers from hand (Eiganjo's channel, Razorgrass Ambush): the best creature in
/// `targets` that `dmg` kills and that is worth at least 4 (pval), or None
fn shot_target(g: &Game, who: PlayerId, targets: &[PermId], dmg: i32) -> Option<PermId> {
    let cs: Vec<PermId> = targets
        .iter()
        .copied()
        .filter(|&m| {
            let x = g.perm(m);
            x.on_bf && x.owner != who && g.is_creature(m) && !untargetable(g, m) && etgh(g, m) <= dmg
        })
        .collect();
    max_by(&cs, |m| pval(g, m)).filter(|&m| pval(g, m) >= 4.0)
}

/// **Eiganjo, Seat of the Empire**: Channel, {2}{W}, discard it: 4 damage to target attacking or blocking creature;
/// {1} less for each legendary creature you control. The AI channels it at an attacker of an opponent's attack on it
/// (after blocks) or at a blocker of its own attack, when 4 damage kills a creature worth at least 4
pub fn eiganjo_cost(g: &Game, p: PlayerId) -> u32 {
    let leg = g
        .player(p)
        .perms
        .iter()
        .filter(|&&m| g.is_creature(m) && !g.perm(m).phased && g.perm(m).cd.is_some_and(|c| g.db.get(c).tag(Tag::Leg)))
        .count() as u32;
    2u32.saturating_sub(leg)
}

/// pay and discard Eiganjo; false when it can't be done (the caller then does nothing)
fn eiganjo_channel(g: &mut Game, c: CardId, who: PlayerId, t: PermId) -> Res<bool> {
    let gn = eiganjo_cost(g, who);
    if !g.player(who).hand.contains(&c) || !can_pay(g, who, gn, "W", false) || !pay(g, who, gn, "W", false)? {
        return Ok(false);
    }
    let i = g.player(who).hand.iter().position(|&x| x == c).unwrap();
    g.player_mut(who).hand.remove(i);
    g.player_mut(who).gy.push(c);
    crate::glog!(g, "  {} channels Eiganjo: 4 damage to {}", g.player(who).name, g.perm(t).name);
    if ability_window_card(g, who, c, "4 damage to an attacking or blocking creature", None, Some(t))? && g.perm(t).on_bf {
        apply_removal(g, Some(who), t, "dmg4", None)?;
    }
    Ok(true)
}

fn eiganjo_defend(
    g: &mut Game,
    c: CardId,
    d: PlayerId,
    p: PlayerId,
    atk: &mut Vec<PermId>,
    _assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    if !can_pay(g, d, eiganjo_cost(g, d), "W", false) {
        return Ok(());
    }
    let mine: Vec<PermId> = atk.iter().copied().filter(|&a| g.perm(a).owner == p).collect();
    let Some(t) = shot_target(g, d, &mine, 4) else { return Ok(()) };
    if eiganjo_channel(g, c, d, t)? && !g.perm(t).on_bf {
        atk.retain(|&a| a != t);
    }
    Ok(())
}

fn eiganjo_blocks(
    g: &mut Game,
    c: CardId,
    p: PlayerId,
    _atk: &[PermId],
    _d: PlayerId,
    assign: &mut Vec<(PermId, PermId)>,
) -> Res {
    if !can_pay(g, p, eiganjo_cost(g, p), "W", false) {
        return Ok(());
    }
    let blockers: Vec<PermId> = assign.iter().map(|x| x.1).collect();
    let Some(t) = shot_target(g, p, &blockers, 4) else { return Ok(()) };
    if eiganjo_channel(g, c, p, t)? && !g.perm(t).on_bf {
        assign.retain(|x| x.1 != t);
    }
    Ok(())
}

// ================================================================== Deserted Temple
/// What untapping this (tapped) land adds to p's mana: total mana with it untapped minus now
fn land_untap_gain(g: &mut Game, p: PlayerId, t: LandId) -> i32 {
    let before = total_mana(g, p, false) as i32;
    g.land_mut(t).tapped = false;
    let after = total_mana(g, p, false) as i32;
    g.land_mut(t).tapped = true;
    after - before
}

/// the same for a tapped permanent
fn perm_untap_gain(g: &mut Game, p: PlayerId, m: PermId) -> i32 {
    let before = total_mana(g, p, false) as i32;
    g.perm_mut(m).tapped = false;
    let after = total_mana(g, p, false) as i32;
    g.perm_mut(m).tapped = true;
    after - before
}

/// **Deserted Temple**: {T}: Add {C}. {1},{T}: untap target land. A mana engine: the AI untaps a tapped land when that
/// gives at least 3 more mana (the {1} and the Temple's own mana are the cost), e.g. Gaea's Cradle or City of Traitors
fn temple(g: &mut Game, l: LandId, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if g.land(l).tapped || !can_pay_without(g, p, l, 1, "") {
        return Ok(vec![]);
    }
    let mut best: Option<(LandId, i32)> = None;
    for t in g.player(p).lands.clone() {
        if t == l || !g.land(t).tapped {
            continue;
        }
        let d = land_untap_gain(g, p, t);
        if d >= 3 && best.is_none_or(|b| d > b.1) {
            best = Some((t, d));
        }
    }
    let Some((t, d)) = best else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: 0.5 * (d - 2) as f64,
        label: format!("Deserted Temple untaps {}", name_of(g, t)),
        act: Some(Action::Plan { f: temple_go, arg: pack(l.0, t.0) }),
    }])
}

fn pack(a: u32, b: u32) -> i64 {
    ((b as i64) << 32) | a as i64
}

fn unpack(arg: i64) -> (u32, u32) {
    ((arg & 0xffff_ffff) as u32, (arg >> 32) as u32)
}

fn temple_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (l, t) = unpack(arg);
    let (l, t) = (LandId(l), LandId(t));
    if !g.player(p).lands.contains(&l) || !g.player(p).lands.contains(&t) || g.land(l).tapped || !g.land(t).tapped {
        return Ok(false);
    }
    if !pay_without(g, p, l, 1, "")? {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    crate::glog!(g, "  {} uses Deserted Temple: untaps {}", g.player(p).name, name_of(g, t));
    if ability_window_card(g, p, g.land(l).cd, "untap a land", None, None)? && g.player(p).lands.contains(&t) {
        g.land_mut(t).tapped = false;
    }
    Ok(true)
}

// ================================================================== Minamo, School at Water's Edge
/// **Minamo, School at Water's Edge**: {T}: Add {U}. {U},{T}: untap target legendary permanent (a legendary land or
/// permanent). The AI untaps a tapped legendary land or permanent when that gives at least 3 more mana (the {U} and
/// Minamo's own mana are the cost): Gaea's Cradle, a legendary mana creature. Arg: Minamo's land id, then the
/// target's kind (1 land, 2 permanent) and id packed above it
fn minamo(g: &mut Game, l: LandId, p: PlayerId, _post: Option<bool>) -> Res<Vec<Opt>> {
    if g.land(l).tapped || !can_pay_without(g, p, l, 0, "U") {
        return Ok(vec![]);
    }
    let mut best: Option<(i64, i32, String)> = None; // (arg, gain, label)
    for t in g.player(p).lands.clone() {
        if t != l && g.land(t).tapped && g.db.get(g.land(t).cd).tag(Tag::Leg) {
            let d = land_untap_gain(g, p, t);
            if d >= 3 && best.as_ref().is_none_or(|b| d > b.1) {
                best = Some((pack(l.0, t.0), d, name_of(g, t).to_string()));
            }
        }
    }
    for m in g.player(p).perms.clone() {
        let x = g.perm(m);
        if x.on_bf && x.tapped && !x.phased && x.cd.is_some_and(|c| g.db.get(c).tag(Tag::Leg)) {
            let d = perm_untap_gain(g, p, m);
            if d >= 3 && best.as_ref().is_none_or(|b| d > b.1) {
                best = Some((pack(l.0, m.0 | 1 << 31), d, g.perm(m).name.to_string()));
            }
        }
    }
    let Some((arg, d, n)) = best else { return Ok(vec![]) };
    Ok(vec![Opt {
        utility: 0.5 * (d - 2) as f64,
        label: format!("Minamo untaps {n}"),
        act: Some(Action::Plan { f: minamo_go, arg }),
    }])
}

fn minamo_go(g: &mut Game, p: PlayerId, arg: i64) -> Res<bool> {
    let (l, t) = unpack(arg);
    let l = LandId(l);
    if !g.player(p).lands.contains(&l) || g.land(l).tapped || !pay_without(g, p, l, 0, "U")? {
        return Ok(false);
    }
    g.land_mut(l).tapped = true;
    let perm = t & (1 << 31) != 0;
    let t = t & !(1 << 31);
    crate::glog!(g, "  {} uses Minamo: untaps a legendary permanent", g.player(p).name);
    if ability_window_card(g, p, g.land(l).cd, "untap a legendary permanent", None, None)? {
        if perm {
            let m = PermId(t);
            if g.perm(m).on_bf && g.perm(m).owner == p {
                g.perm_mut(m).tapped = false;
                g.bf_ver += 1;
            }
        } else if g.player(p).lands.contains(&LandId(t)) {
            g.land_mut(LandId(t)).tapped = false;
        }
    }
    Ok(true)
}

// CODE-HERE

pub fn register(r: &mut Registry, db: &CardDb) -> Result<(), String> {
    r.card(db, "City of Brass")?.on_tap_land = Some(brass_tap);
    // City of Traitors: land_etb on every land that has none yet (the City itself included: playing a second City
    // sacrifices the first, and the new one stays)
    let lands: Vec<String> = db.iter().filter(|d| d.land).map(|d| d.name.to_string()).collect();
    for n in lands {
        let c = r.card(db, &n)?;
        if c.land_etb.is_none() {
            c.land_etb = Some(traitors_check);
        }
    }
    r.card(db, "Gemstone Caverns")?.land_cols = Some(caverns_cols);
    for n in ANY_FILTER {
        r.card(db, n)?.land_cols = Some(any_filter_cols);
    }
    r.card(db, "Abstergo Entertainment")?.land_options = Some(abstergo);
    let e = r.card(db, "Eiganjo, Seat of the Empire")?;
    e.hand_defend = Some(eiganjo_defend);
    e.hand_blocks = Some(eiganjo_blocks);
    r.card(db, "Deserted Temple")?.land_options = Some(temple);
    r.card(db, "Minamo, School at Water's Edge")?.land_options = Some(minamo);
    // REGISTER-HERE
    Ok(())
}
